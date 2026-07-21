//! VM coroutines logic, extracted from vm/mod.rs (no semantic change).
use super::*;

impl<'m> Vm<'m> {
    /// Build a `Generator` handle for a freshly-bound generator-body `frame`
    /// (GEN): park the frame under a fresh id and return the `Zval::Generator` the
    /// call expression evaluates to. The body does not run until the generator is
    /// first advanced (`NotStarted`). The parked frame lives in `self.generators`
    /// — no coroutine, no `unsafe`, unlike the tree-walker's `corosensei` driver.
    pub(super) fn make_generator(&mut self, mut frame: Frame<'m>) -> Zval {
        let id = self.next_id();
        frame.ext_mut().gen_id = Some(id);
        let func_name = frame.func.name.to_vec().into_boxed_slice();
        self.generators.insert(id, frame);
        Zval::Generator(Rc::new(RefCell::new(GenState {
            id,
            func_name,
            status: GenStatus::NotStarted,
            advanced: false,
            cur_key: Zval::Null,
            cur_val: Zval::Null,
            ret: Zval::Null,
            auto_key: 0,
        })))
    }

    /// Advance a generator one step (GEN): move its parked frame onto the call
    /// stack and run until it yields again, returns, or throws an uncaught
    /// exception. Mirrors `eval::resume_generator` for the status guards and
    /// auto-key resolution. `sent` is the value the suspended `yield` expression
    /// evaluates to (NULL for `next()`/`foreach`).
    pub(super) fn resume_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        sent: Zval,
    ) -> Result<(), PhpError> {
        let was_suspended = {
            let mut gs = gs_rc.borrow_mut();
            match gs.status {
                GenStatus::Running => {
                    return Err(PhpError::Error(
                        "Cannot resume an already running generator".to_string(),
                    ))
                }
                GenStatus::Done => return Ok(()),
                GenStatus::Suspended => {
                    gs.advanced = true;
                    gs.status = GenStatus::Running;
                    true
                }
                GenStatus::NotStarted => {
                    gs.status = GenStatus::Running;
                    false
                }
            }
        };
        let id = gs_rc.borrow().id;
        let frame = self.generators.remove(&id).expect("parked generator frame");
        let baseline = self.frames.len();
        self.frames.push(frame);
        if was_suspended {
            // The suspended `yield` expression evaluates to the sent value.
            self.frames[baseline].stack.push(sent);
        }
        // Run the body until it yields/returns; route its *own* exceptions through
        // `unwind` with the generator frame as the floor, so a `try` inside the
        // generator is honoured and an uncaught throw surfaces at the resume site.
        let outcome = loop {
            match self.run_loop(baseline) {
                Ok(exit) => break Ok(exit),
                Err(e) => match self.unwind(e, baseline) {
                    None => continue,
                    Some(e) => break Err(e),
                },
            }
        };
        match outcome {
            Ok(RunExit::Yielded { key, value }) => {
                // The frame was already parked by `Op::Yield` / `Op::YieldFrom`.
                let mut gs = gs_rc.borrow_mut();
                let resolved = match key {
                    GenKey::Auto => {
                        let k = Zval::Long(gs.auto_key);
                        gs.auto_key += 1;
                        k
                    }
                    GenKey::Keyed(Zval::Long(n)) => {
                        if n >= gs.auto_key {
                            gs.auto_key = n.wrapping_add(1);
                        }
                        Zval::Long(n)
                    }
                    GenKey::Keyed(z) | GenKey::Verbatim(z) => z,
                };
                gs.cur_key = resolved;
                gs.cur_val = value;
                gs.status = GenStatus::Suspended;
                Ok(())
            }
            Ok(RunExit::Returned(v)) => {
                // The body returned; `Op::Ret` already popped the generator frame.
                let mut gs = gs_rc.borrow_mut();
                gs.ret = v;
                gs.cur_key = Zval::Null;
                gs.cur_val = Zval::Null;
                gs.status = GenStatus::Done;
                Ok(())
            }
            Ok(RunExit::Suspended { .. }) => {
                // `Fiber::suspend` reached across a generator resume (a fiber
                // suspended from within a generator that is itself inside the
                // fiber). This pathological nesting is out of scope; fail cleanly.
                let mut gs = gs_rc.borrow_mut();
                gs.status = GenStatus::Done;
                Err(PhpError::Error(
                    "VM: cannot suspend a Fiber from within a Generator (unsupported nesting)"
                        .to_string(),
                ))
            }
            Err(e) => {
                // Uncaught inside the generator: `unwind` left the dead frame at
                // the baseline; drop it (noting what it held) and surface the
                // exception at the resumer.
                let dead = self.frames.pop().expect("dead generator frame");
                self.gc_note_frame(&dead);
                self.recycle_frame(dead);
                let (k, v) = {
                    let mut gs = gs_rc.borrow_mut();
                    gs.status = GenStatus::Done;
                    (
                        std::mem::replace(&mut gs.cur_key, Zval::Null),
                        std::mem::replace(&mut gs.cur_val, Zval::Null),
                    )
                };
                self.gc_note(&k);
                self.gc_note(&v);
                Err(e)
            }
        }
    }

    /// Prime a `NotStarted` generator to its first `yield` (GEN); a no-op
    /// otherwise. Mirrors `eval::ensure_started`.
    pub(super) fn ensure_started(&mut self, gs_rc: &Rc<RefCell<GenState>>) -> Result<(), PhpError> {
        if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
            self.resume_generator(gs_rc, Zval::Null)?;
        }
        Ok(())
    }

    /// Synthesize a plain `Exception` carrying `msg`, for the generator misuse
    /// errors PHP raises as `Exception` (rewind-after-run, getReturn-before-return)
    /// — the tree-walker raises these as `Error`; the VM matches real PHP. Falls
    /// back to an engine `Error` if the prelude has no `Exception`.
    pub(super) fn gen_exception(&mut self, msg: &str) -> PhpError {
        match self.class_index.get(&b"exception"[..]).copied() {
            Some(cid) => match self.synthesize_throwable(cid, msg) {
                Ok(obj) => PhpError::Thrown(obj),
                Err(e) => e,
            },
            None => PhpError::Error(msg.to_string()),
        }
    }

    /// Dispatch a built-in `Generator` method (GEN), returning the value to leave
    /// on the caller's stack. Mirrors `eval::generator_method`.
    pub(super) fn generator_method(
        &mut self,
        gs_rc: Rc<RefCell<GenState>>,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        match method.to_ascii_lowercase().as_slice() {
            b"current" => {
                self.ensure_started(&gs_rc)?;
                Ok(gs_rc.borrow().cur_val.clone())
            }
            b"key" => {
                self.ensure_started(&gs_rc)?;
                Ok(gs_rc.borrow().cur_key.clone())
            }
            b"next" => {
                self.ensure_started(&gs_rc)?;
                self.resume_generator(&gs_rc, Zval::Null)?;
                Ok(Zval::Null)
            }
            b"valid" => {
                self.ensure_started(&gs_rc)?;
                let valid = !matches!(gs_rc.borrow().status, GenStatus::Done);
                Ok(Zval::Bool(valid))
            }
            b"rewind" => {
                self.ensure_started(&gs_rc)?;
                if gs_rc.borrow().advanced {
                    return Err(self.gen_exception("Cannot rewind a generator that was already run"));
                }
                Ok(Zval::Null)
            }
            b"send" => {
                // Deliver `$value` as the suspended `yield`'s result and advance;
                // an unstarted generator is primed first (GEN-2). Returns the next
                // yielded value (NULL once the generator is done).
                let value = args.into_iter().next().unwrap_or(Zval::Null);
                if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
                    self.resume_generator(&gs_rc, Zval::Null)?;
                }
                self.resume_generator(&gs_rc, value)?;
                Ok(gs_rc.borrow().cur_val.clone())
            }
            b"getreturn" => {
                // PHP auto-primes: getReturn() on a fresh generator starts it (so a
                // body that returns before any yield exposes its value); before the
                // body has returned, it is an error (GEN-2).
                self.ensure_started(&gs_rc)?;
                if !matches!(gs_rc.borrow().status, GenStatus::Done) {
                    return Err(self
                        .gen_exception("Cannot get return value of a generator that hasn't returned"));
                }
                Ok(gs_rc.borrow().ret.clone())
            }
            other => Err(PhpError::Error(format!(
                "Call to undefined method Generator::{}()",
                String::from_utf8_lossy(other)
            ))),
        }
    }

    /// The current status of fiber `id` (GEN-4); a missing entry means the fiber
    /// has not been started yet.
    pub(super) fn fiber_status(&self, id: u32) -> Option<FiberStatus> {
        self.fibers.get(&id).map(|s| s.status)
    }

    /// Run a fiber's frame segment at `baseline` until it suspends, its callable
    /// returns, or it throws (GEN-4). Shared by `start`/`resume`. Returns the
    /// value to hand back to the caller (the `Fiber::suspend` value, or NULL on
    /// termination); an exception that escapes the fiber propagates to the caller.
    pub(super) fn drive_fiber(&mut self, id: u32, obj: &Zval, baseline: usize) -> Result<Zval, PhpError> {
        self.fiber_stack.push(FiberContext { id, baseline, obj: obj.clone() });
        // `@` is per-execution-context: park the caller's suppression state and
        // run the fiber body under its OWN (restored from its last suspend), so
        // `@$fiber->start()` does not silence diagnostics inside the fiber.
        // The caller's silence also masked `error_level` (BEGIN_SILENCE) —
        // unmask to its pre-`@` value for the fiber body, and re-apply the
        // fiber's own mask if it suspended inside a `@` of its own.
        let caller_depth = std::mem::take(&mut self.suppress_depth);
        let caller_marks = std::mem::take(&mut self.suppress_marks);
        let caller_silence = std::mem::take(&mut self.silence_saved);
        if let Some(&outer) = caller_silence.first() {
            self.error_level = outer;
        }
        if let Some(st) = self.fibers.get_mut(&id) {
            let (d, m, s) = std::mem::take(&mut st.suppress);
            self.suppress_depth = d;
            self.suppress_marks = m;
            self.silence_saved = s;
            if !self.silence_saved.is_empty() && self.error_level & !4437 != 0 {
                self.error_level &= 4437;
            }
        }
        let outcome = loop {
            match self.run_loop(baseline) {
                Ok(exit) => break Ok(exit),
                Err(e) => match self.unwind(e, baseline) {
                    None => continue,
                    Some(e) => break Err(e),
                },
            }
        };
        self.fiber_stack.pop();
        // Park the fiber's suppression state (survives a suspend inside `@`)
        // and restore the caller's — including the caller's silence mask on
        // `error_level` (its region is still open). A fiber suspended inside
        // its own `@` unmasks first (conditionally, END_SILENCE style), so an
        // error_reporting() write inside the fiber still propagates out.
        let fiber_depth = std::mem::replace(&mut self.suppress_depth, caller_depth);
        let fiber_marks = std::mem::replace(&mut self.suppress_marks, caller_marks);
        let fiber_silence = std::mem::replace(&mut self.silence_saved, caller_silence);
        if let Some(&outer) = fiber_silence.first() {
            if self.error_level & !4437 == 0 && outer & !4437 != 0 {
                self.error_level = outer;
            }
        }
        if !self.silence_saved.is_empty() && self.error_level & !4437 != 0 {
            self.error_level &= 4437;
        }
        if let Some(st) = self.fibers.get_mut(&id) {
            st.suppress = (fiber_depth, fiber_marks, fiber_silence);
        }
        match outcome {
            Ok(RunExit::Suspended { value }) => {
                // `Fiber::suspend` already parked frames[baseline..] into the entry.
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Suspended;
                }
                Ok(value)
            }
            Ok(RunExit::Returned(v)) => {
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Terminated;
                    st.ret = v;
                }
                Ok(Zval::Null)
            }
            Ok(RunExit::Yielded { .. }) => {
                unreachable!("a fiber callable does not `yield` at its own baseline")
            }
            Err(e) => {
                // The exception escaped the fiber: it terminates and the error
                // propagates out of start()/resume(). `unwind` left the dead
                // baseline frame; drop the whole segment, noting what it held.
                while self.frames.len() > baseline {
                    let dead = self.frames.pop().expect("fiber frames above baseline");
                    self.gc_note_frame(&dead);
                    self.recycle_frame(dead);
                }
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Terminated;
                }
                Err(e)
            }
        }
    }

    /// `$fiber->start(...$args)` (GEN-4): invoke the fiber's callable as a fresh
    /// frame and run it to the first suspend or to completion.
    pub(super) fn fiber_start(&mut self, obj: &Zval, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_start on a non-object"),
        };
        if self.fiber_status(id).is_some() {
            return Err(PhpError::Error(
                "Cannot start a fiber that has already been started".to_string(),
            ));
        }
        let callable = match obj {
            Zval::Object(o) => {
                let key = self.host_prop_key(o.borrow().class_id as usize, b"callable");
                o.borrow().props.get(key.as_slice()).cloned().unwrap_or(Zval::Null)
            }
            _ => Zval::Null,
        };
        self.fibers.insert(
            id,
            FiberState {
                status: FiberStatus::Running,
                parked: Vec::new(),
                ret: Zval::Null,
                suppress: (0, Vec::new(), Vec::new()),
            },
        );
        let baseline = self.frames.len();
        self.invoke_value(callable, args)?;
        if self.frames.len() != baseline + 1 {
            // A non-closure callable (builtin / generator function) did not push a
            // plain fiber frame; out of scope.
            while self.frames.len() > baseline {
                let dead = self.frames.pop().expect("frames above baseline");
                self.gc_note_frame(&dead);
                self.recycle_frame(dead);
            }
            self.fibers.remove(&id);
            return Err(PhpError::Error(
                "VM: fiber callable must be a closure or function (other callables unsupported)"
                    .to_string(),
            ));
        }
        self.drive_fiber(id, obj, baseline)
    }

    /// `$fiber->resume($value)` (GEN-4): restore the parked segment, deliver
    /// `$value` as the suspended `Fiber::suspend`'s result, and run on.
    pub(super) fn fiber_resume(&mut self, obj: &Zval, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_resume on a non-object"),
        };
        if self.fiber_status(id) != Some(FiberStatus::Suspended) {
            return Err(PhpError::Error(
                "Cannot resume a fiber that is not suspended".to_string(),
            ));
        }
        let value = args.into_iter().next().unwrap_or(Zval::Null);
        let parked = std::mem::take(&mut self.fibers.get_mut(&id).expect("fiber state").parked);
        let baseline = self.frames.len();
        self.frames.extend(parked);
        // The suspended `Fiber::suspend(...)` call evaluates to the resume value.
        self.frames.last_mut().expect("restored fiber frame").stack.push(value);
        self.fibers.get_mut(&id).expect("fiber state").status = FiberStatus::Running;
        self.drive_fiber(id, obj, baseline)
    }

    /// Dispatch a `Fiber` instance method (GEN-4), returning the value to leave on
    /// the caller's stack. `Fiber::suspend`/`getCurrent` are static and handled at
    /// the `StaticCall` site instead.
    pub(super) fn fiber_method(
        &mut self,
        obj: &Zval,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_method on a non-object"),
        };
        match method.to_ascii_lowercase().as_slice() {
            b"start" => self.fiber_start(obj, args),
            b"resume" => self.fiber_resume(obj, args),
            b"getreturn" => match self.fibers.get(&id) {
                Some(st) if st.status == FiberStatus::Terminated => Ok(st.ret.clone()),
                _ => Err(PhpError::Error(
                    "Cannot get fiber return value: The fiber has not returned".to_string(),
                )),
            },
            b"isstarted" => Ok(Zval::Bool(self.fiber_status(id).is_some())),
            b"issuspended" => {
                Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Suspended)))
            }
            b"isrunning" => Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Running))),
            b"isterminated" => {
                Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Terminated)))
            }
            b"throw" => Err(PhpError::Error(
                "VM: Fiber::throw() is not yet supported".to_string(),
            )),
            other => Err(PhpError::Error(format!(
                "Call to undefined method Fiber::{}()",
                String::from_utf8_lossy(other)
            ))),
        }
    }
}
