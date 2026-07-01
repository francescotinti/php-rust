//! VM exceptions logic, extracted from vm/mod.rs (no semantic change).
use super::*;

impl<'m> Vm<'m> {
    /// Render an uncaught fatal at the tail of `rendered` (E1; mirrors
    /// `eval::render_fatal`). A user-thrown object carries its own class, message,
    /// line and trace; an engine error uses its variant name and the captured
    /// `fault_line`.
    pub(super) fn render_fatal(&mut self, err: &PhpError, fault_line: Line) {
        // The throwing-site file: taken from the throwable (its `file` prop was
        // stamped at the construct/fault site, so it reports the defining file
        // across includes); the bare-error fallback uses the executing frame.
        let module_file = String::from_utf8_lossy(&self.module.file).into_owned();
        // Prefer the throwable `unwind` synthesized for this fatal (its `trace` was
        // captured while the faulting frames were live); fall back to a user-thrown
        // object, then to the raw error with a bare `#0 {main}` trace.
        let throwable = self.uncaught_throwable.take().or_else(|| match err {
            PhpError::Thrown(v @ Zval::Object(_)) => Some(v.clone()),
            _ => None,
        });
        let (class, message, line, trace, file) = match throwable {
            Some(Zval::Object(o)) => {
                let b = o.borrow();
                let class = String::from_utf8_lossy(b.class_name.as_bytes()).into_owned();
                let message = match b.props.get(b"message") {
                    Some(Zval::Str(s)) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                    _ => String::new(),
                };
                let line = match b.props.get(b"line") {
                    Some(Zval::Long(n)) => *n,
                    _ => fault_line as i64,
                };
                let trace = match b.props.get(self.host_prop_key(b.class_id as usize, b"traceString").as_slice()) {
                    Some(Zval::Str(s)) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                    _ => "#0 {main}".to_string(),
                };
                let file = match b.props.get(b"file") {
                    Some(Zval::Str(s)) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                    _ => module_file.clone(),
                };
                (class, message, line, trace, file)
            }
            _ => (
                err.class_name().to_string(),
                err.message().to_string(),
                fault_line as i64,
                "#0 {main}".to_string(),
                self
                    .frames
                    .last()
                    .map(|_| String::from_utf8_lossy(self.frame_file(self.frames.len() - 1)).into_owned())
                    .unwrap_or_else(|| module_file.clone()),
            ),
        };
        // PHP omits the ": <message>" segment entirely when the throwable carries an
        // empty message, rendering "Uncaught Exception in F:L" rather than
        // "Uncaught Exception:  in F:L" (note the doubled space).
        let label = if message.is_empty() {
            format!("Uncaught {class}")
        } else {
            format!("Uncaught {class}: {message}")
        };
        // An *argument* TypeError names the call site in its message ("…called in F
        // on line N") and reports the *definition* site here as " and defined in
        // F:L"; every other throwable (including a return TypeError, whose message
        // ends "…returned") uses the standard " in F:L". `file`/`line` are the
        // throwable's, which for these errors are the definition site.
        let arg_type_err = class == "TypeError" && message.contains(", called in ");
        let head = if arg_type_err {
            format!("\nFatal error: {label} and defined in {file}:{line}\n")
        } else {
            format!("\nFatal error: {label} in {file}:{line}\n")
        };
        let block = format!("{head}Stack trace:\n{trace}\n  thrown in {file} on line {line}\n");
        self.rendered.extend_from_slice(block.as_bytes());
    }

    /// Find the innermost `try` whose protected range covers the in-flight
    /// exception and route control to its catch-dispatch block, popping frames
    /// with no handler. Returns `None` once control is parked at a `catch`, or
    /// `Some(e)` if the exception is uncatchable (a non-object throw, an engine
    /// error — EXC-3 — or `Exit`) or escapes uncaught down to `floor`.
    ///
    /// `floor` is the lowest frame index this unwind may inspect/route into: the
    /// search stops once the frame at `floor` has been checked, leaving that frame
    /// on the stack. Top-level passes `0` (so `main` is the floor, retained as
    /// today); a generator resume passes the parked frame's depth, so an
    /// exception uncaught inside the generator surfaces at the resume site (the
    /// resumer then pops the dead generator frame).
    pub(super) fn unwind(&mut self, e: PhpError, floor: usize) -> Option<PhpError> {
        log::debug!(
            target: "phpr::exc",
            "unwind: {} (floor {}, depth {})",
            e.class_name(),
            floor,
            self.frames.len()
        );
        // A throwable propagating out of an `@` abandons that suppression region
        // (its `Op::SuppressEnd` is skipped): drop the diagnostics raised under it
        // and reset, so a later `catch` resumes with suppression cleared (step 48;
        // `@` silences warnings, not engine errors / thrown objects).
        if let Some(&outer) = self.suppress_marks.first() {
            self.diags.truncate(outer);
            self.suppress_marks.clear();
            self.suppress_depth = 0;
        }
        // The in-flight Throwable object. A user `throw` of an object is itself
        // (EXC-1). An engine error (EXC-3a) is resolved to its prelude class by
        // name and a Throwable is synthesized carrying its message; if the class
        // is absent or can't be instantiated, the original error propagates.
        // `Exit` and a thrown non-object stay uncatchable.
        let obj = match &e {
            PhpError::Thrown(v) if matches!(v, Zval::Object(_)) => v.clone(),
            PhpError::Exit(_) | PhpError::Thrown(_) => return Some(e),
            engine => {
                let name = engine.class_name().to_ascii_lowercase();
                let msg = engine.message().to_owned();
                // An argument/return TypeError overrides its file/line with the
                // callee's definition site (step 14).
                let loc = engine.loc_override().map(|(f, l)| (f.to_vec(), l));
                match self.class_index.get(name.as_bytes()).copied() {
                    Some(cid) => match self.synthesize_throwable_at(cid, &msg, loc) {
                        Ok(v) => v,
                        Err(_) => return Some(e),
                    },
                    None => return Some(e),
                }
            }
        };
        loop {
            let top = self.frames.len() - 1;
            let faulting = self.frames[top].ip.saturating_sub(1);
            let region = self.frames[top]
                .func
                .exc_table
                .iter()
                .find(|r| faulting >= r.start as usize && faulting < r.end as usize)
                .copied();
            if let Some(r) = region {
                // Caught: any earlier "uncaught" trace stashed by a deeper unwind is
                // void (the exception is handled after all).
                self.uncaught_throwable = None;
                // Statement boundaries leave the operand stack at its baseline, so
                // clearing any partial-expression values restores it for the
                // handler. A catch region parks the exception on the stack for
                // `CatchMatch`; a finally region parks it in `pending_throw`, to be
                // re-raised at `EndFinally` (EXC-2).
                self.frames[top].stack.clear();
                if r.is_finally {
                    self.frames[top].pending_throw = Some(obj);
                } else {
                    // A new in-flight exception supersedes any exception parked by
                    // an earlier finally in this frame (e.g. a finally that threw).
                    self.frames[top].pending_throw = None;
                    self.frames[top].stack.push(obj);
                }
                self.frames[top].ip = r.target as usize;
                return None;
            }
            if top == floor {
                // The floor frame had no matching handler: propagate. Stash the
                // *synthesized* throwable — its `trace` was captured above, while the
                // frames (incl. the callee whose argument check failed) were still
                // live — so `render_fatal` can show the real stack even though the
                // frames are gone by then. Keep the deepest capture (store once); a
                // later catch clears it. The floor frame is left for the caller.
                if self.uncaught_throwable.is_none() {
                    self.uncaught_throwable = Some(obj);
                }
                return Some(e);
            }
            self.frames.pop();
        }
    }

    /// Convert an uncaught [`PhpError`] to the Throwable object a
    /// `set_exception_handler` receives: a thrown object is itself; an engine error
    /// is synthesized into its prelude class; `exit` / a non-object throw have no
    /// object (`None`). Mirrors the resolution in [`Self::unwind`].
    pub(super) fn error_to_throwable(&mut self, e: &PhpError) -> Option<Zval> {
        match e {
            PhpError::Thrown(v) if matches!(v, Zval::Object(_)) => Some(v.clone()),
            PhpError::Exit(_) | PhpError::Thrown(_) => None,
            engine => {
                let name = engine.class_name().to_ascii_lowercase();
                let msg = engine.message().to_owned();
                let cid = self.class_index.get(name.as_bytes()).copied()?;
                self.synthesize_throwable(cid, &msg).ok()
            }
        }
    }

    /// Route an uncaught throwable to the active `set_exception_handler` (Session 1b):
    /// flush pending diagnostics, then invoke the handler with the throwable. Returns
    /// `true` when a handler ran to completion (the script then ends with no fatal
    /// banner); `false` if there is no handler, the error is not a throwable, or the
    /// handler itself errored (the original fatal is reported instead).
    pub(super) fn handle_uncaught_exception(&mut self, e: &PhpError) -> bool {
        let Some(handler) = self.exception_handlers.last().cloned() else {
            return false;
        };
        let Some(obj) = self.error_to_throwable(e) else {
            return false;
        };
        let line = self.fatal_line;
        // Reached only after `final_flush` is set, so routing is skipped and this
        // never errs; the now-fallible signature is discharged with `let _`.
        let _ = self.flush_diags(line);
        self.call_callable(handler, vec![obj]).is_ok()
    }

    /// Build a Throwable object of `cid` carrying `message`, used to materialise
    /// an engine error (`TypeError`, `DivisionByZeroError`, …) so it can be
    /// offered to a `catch` (EXC-3a). Mirrors `eval::synthesize_throwable`:
    /// allocates the instance (prop defaults + `info` + id, via `alloc_object`)
    /// **without** running a constructor, then overwrites `message` directly.
    /// `line`/`file`/`trace` stay at their prelude defaults here — they are
    /// filled by the line-tracking (EXC-3b) and stack-trace (EXC-3c) steps.
    pub(super) fn synthesize_throwable(&mut self, cid: ClassId, message: &str) -> Result<Zval, PhpError> {
        self.synthesize_throwable_at(cid, message, None)
    }

    /// Like [`Self::synthesize_throwable`] but with an explicit `(file, line)`
    /// override for `getFile()`/`getLine()` — used by an argument/return TypeError,
    /// whose throwable reports the callee's *definition* site, not the faulting op.
    pub(super) fn synthesize_throwable_at(
        &mut self,
        cid: ClassId,
        message: &str,
        loc: Option<(Vec<u8>, u32)>,
    ) -> Result<Zval, PhpError> {
        let value = self.alloc_object(cid)?;
        // The line of the op that faulted (`ip-1` in the faulting frame), the
        // module's file, and the current stack trace — mirroring
        // `eval::synthesize_throwable` (EXC-3b/3c). An explicit `loc` overrides the
        // first two (the definition site of a type error).
        let (file, line) = match loc {
            Some((f, l)) => (f, l),
            None => {
                let top = self.frames.len() - 1;
                (self.frame_file(top).to_vec(), self.cur_line(top))
            }
        };
        let (trace, trace_string) = self.capture_trace();
        if let Zval::Object(o) = &value {
            let mut b = o.borrow_mut();
            b.props
                .set(b"message", Zval::Str(PhpStr::new(message.as_bytes().to_vec())));
            b.props.set(b"line", Zval::Long(line as i64));
            b.props
                .set(b"file", Zval::Str(PhpStr::new(file)));
            b.props.set(self.host_prop_key(cid, b"trace").as_slice(), trace);
            b.props
                .set(self.host_prop_key(cid, b"traceString").as_slice(), Zval::Str(PhpStr::new(trace_string)));
        }
        Ok(value)
    }
}
