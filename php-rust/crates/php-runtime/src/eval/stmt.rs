//! Statement execution and control flow: the `exec_stmt`/`exec_stmts` dispatch,
//! loops (`exec_foreach`, `loop_step`, `eval_for_cond`), `switch`/`match`, and
//! exception propagation (`handle_thrown`, `synthesize_throwable`,
//! `capture_trace`). Split out of `eval.rs` (step 60); behaviour is unchanged.
use std::cell::RefCell;
use std::rc::Rc;

use php_types::{
    convert, ops, Diag, GenState, GenStatus, Key, Object, PhpArray,
    PhpError, PhpStr, Zval,
};

use crate::hir::{
    ClassId, Expr, ExprKind, Slot,
    Stmt, StmtKind,
};

use super::*;

impl<'p> Evaluator<'p> {
    // --- statements ---

    pub(super) fn exec_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, PhpError> {
        // Index-based loop (step 45) so a `goto` can re-enter this block at a
        // different position. With no `goto` involved this walks the statements
        // exactly once, top to bottom, like the original `for`.
        let mut i = 0;
        while i < stmts.len() {
            match self.exec_stmt(&stmts[i])? {
                Flow::Normal => {}
                // A `goto` raised below: if its target label lives in *this*
                // block, jump there and keep executing; otherwise let it bubble
                // up to the enclosing block that owns the label.
                Flow::Goto(label) => {
                    match stmts
                        .iter()
                        .position(|s| matches!(&s.kind, StmtKind::Label(n) if **n == *label))
                    {
                        Some(j) => {
                            i = j;
                            continue;
                        }
                        None => return Ok(Flow::Goto(label)),
                    }
                }
                other => return Ok(other),
            }
            // Immediate destruction sweep at global-scope statement boundaries
            // (step 24-3): objects that just became unreachable (a discarded
            // temporary, an overwritten or unset variable, a returned-from local)
            // get their `__destruct` now. Destructor bodies run with a local
            // frame, so the `locals.is_none()` gate keeps this from re-entering.
            if self.locals.is_none() {
                self.sweep_destructors();
            }
            i += 1;
        }
        Ok(Flow::Normal)
    }

    /// Set the current line for diagnostics, run the statement, then flush any
    /// diagnostics it staged. On the error path `cur_line` is left pointing at the
    /// throwing node (see the field doc) for the fatal renderer.
    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow, PhpError> {
        self.cur_line = stmt.line;
        if self.trace_exec {
            // `{:?}` of the kind up to the first delimiter is its variant name —
            // cheap, and avoids enumerating every StmtKind by hand.
            let dbg = format!("{:?}", stmt.kind);
            let variant = dbg.split(|c: char| !c.is_alphanumeric()).next().unwrap_or("");
            eprintln!(
                "[exec]{:indent$}L{} {}",
                "",
                stmt.line,
                variant,
                indent = self.call_stack.len() * 2 + 1
            );
        }
        let r = self.exec_stmt_inner(stmt);
        self.flush_diags();
        r
    }

    fn exec_stmt_inner(&mut self, stmt: &Stmt) -> Result<Flow, PhpError> {
        match &stmt.kind {
            StmtKind::Nop => {}

            // `label:` is a pure marker — `exec_stmts` uses it as a jump target
            // (step 45); reaching it during normal fall-through does nothing.
            StmtKind::Label(_) => {}

            // `goto label;` raises a `Goto` flow that `exec_stmts` resolves to
            // the matching label in this or an enclosing block (step 45).
            StmtKind::Goto(label) => return Ok(Flow::Goto(label.clone())),

            StmtKind::InlineHtml(bytes) => self.emit(bytes),

            StmtKind::Echo(values) => {
                for e in values {
                    let z = self.eval(e)?;
                    // `__toString` is honoured for objects (step 19-6); other
                    // values use the ordinary string funnel.
                    let s = self.stringify(&z)?;
                    // `emit` flushes the (possible) array-to-string warning ahead
                    // of the converted bytes, matching PHP's ordering.
                    self.emit(s.as_bytes());
                }
            }

            StmtKind::Expr(e) => {
                self.eval(e)?;
            }

            StmtKind::Block(body) => return self.exec_stmts(body),

            StmtKind::If {
                cond,
                then,
                elseifs,
                otherwise,
            } => {
                if self.eval_bool(cond)? {
                    return self.exec_stmts(then);
                }
                for (econd, ebody) in elseifs {
                    if self.eval_bool(econd)? {
                        return self.exec_stmts(ebody);
                    }
                }
                return self.exec_stmts(otherwise);
            }

            StmtKind::While { cond, body } => {
                while self.eval_bool(cond)? {
                    match self.loop_step(body)? {
                        LoopStep::Iterate => {}
                        LoopStep::Stop => break,
                        LoopStep::Propagate(f) => return Ok(f),
                    }
                }
            }

            StmtKind::DoWhile { body, cond } => loop {
                match self.loop_step(body)? {
                    LoopStep::Iterate => {}
                    LoopStep::Stop => break,
                    LoopStep::Propagate(f) => return Ok(f),
                }
                if !self.eval_bool(cond)? {
                    break;
                }
            },

            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                for e in init {
                    self.eval(e)?;
                }
                loop {
                    if !self.eval_for_cond(cond)? {
                        break;
                    }
                    match self.loop_step(body)? {
                        LoopStep::Iterate => {}
                        LoopStep::Stop => break,
                        LoopStep::Propagate(f) => return Ok(f),
                    }
                    for e in step {
                        self.eval(e)?;
                    }
                }
            }

            StmtKind::Foreach {
                iter,
                key,
                value,
                by_ref,
                body,
            } => return self.exec_foreach(iter, *key, *value, *by_ref, body),

            StmtKind::Switch { subject, cases } => return self.exec_switch(subject, cases),

            StmtKind::Unset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    self.check_first_prop_write(p.base, &steps, MagicAccess::Unset, b"__unset")?;
                    self.unset_place(p.base, &steps)?;
                }
            }

            StmtKind::Global(bindings) => {
                // At global scope the named variable *is* the global, so `global`
                // is a no-op (no separate local frame to alias into). Inside a
                // function, alias each global cell into the local slot via a
                // shared `Zval::Ref`, promoting the global to a cell on first use
                // — this reuses the step 11d reference machinery (D-12.2). An
                // undefined global is promoted to a NULL cell, so a later write
                // through the alias *creates* the global (D-12.4).
                if self.locals.is_some() {
                    for b in bindings {
                        let cell = make_cell(&mut self.globals[b.global as usize]);
                        frame_mut!(self)[b.local as usize] = Zval::Ref(cell);
                    }
                }
            }

            StmtKind::StaticVar(bindings) => {
                // Alias each local slot to its persistent cell, creating and
                // initialising the cell on the first execution only (D-15.4).
                for b in bindings {
                    if self.statics[b.id].is_none() {
                        let init = match &b.init {
                            Some(e) => self.eval(e)?,
                            None => Zval::Null,
                        };
                        self.statics[b.id] = Some(Rc::new(RefCell::new(init)));
                    }
                    let cell = Rc::clone(self.statics[b.id].as_ref().unwrap());
                    frame_mut!(self)[b.slot as usize] = Zval::Ref(cell);
                }
            }

            StmtKind::Break(n) => return Ok(Flow::Break(*n)),
            StmtKind::Continue(n) => return Ok(Flow::Continue(*n)),
            StmtKind::Return(opt) => {
                // A plain return inside a `function &f()` means the operand was a
                // non-lvalue (or `return;`): PHP raises a Notice and falls back to
                // returning by value (D-13.4).
                if self.fn_returns_ref {
                    self.diags.push(Diag::Notice(
                        "Only variable references should be returned by reference".to_string(),
                    ));
                }
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                return Ok(Flow::Return(v));
            }
            StmtKind::ReturnRef(place) => {
                // Return a *reference* to the place: promote it to a shared cell
                // (reusing the step 11d/12 machinery) and hand the cell back as a
                // `Zval::Ref`, which `$y = &f()` aliases (D-13.2).
                let steps = self.resolve_steps(place)?;
                let cell = self.ref_source_cell(place.base, &steps)?;
                return Ok(Flow::Return(Zval::Ref(cell)));
            }

            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                // Run the protected body; a thrown exception (`Err(Thrown)`) is
                // offered to the catch clauses. Any other control flow (return /
                // break / continue) — or an uncaught throw — is carried in
                // `outcome` and resumes *after* `finally` runs.
                let outcome = match self.exec_stmts(body) {
                    Err(e) => self.handle_thrown(e, catches),
                    flow => flow,
                };
                // `exit`/`die` bypasses `finally` entirely (step 46): unlike a
                // thrown exception, a `return`, or a `break`, PHP does NOT run
                // `finally` on the way out of an `exit`. Propagate immediately.
                if matches!(&outcome, Err(PhpError::Exit(_))) {
                    return outcome;
                }
                if finally.is_empty() {
                    return outcome;
                }
                // `finally` always runs. Its own control flow (a return / throw /
                // break inside it) overrides the try/catch outcome; otherwise the
                // outcome (value, propagating signal, or re-thrown error) wins.
                match self.exec_stmts(finally)? {
                    Flow::Normal => return outcome,
                    other => return Ok(other),
                }
            }
        }
        Ok(Flow::Normal)
    }

    /// Offer a thrown exception to a `try`'s catch clauses (step 20). The first
    /// clause whose type matches by `instanceof` runs (binding `$e` if named);
    /// an unmatched throw propagates.
    ///
    /// Both user `throw`n objects and engine errors are catchable: an engine
    /// error (`PhpError::TypeError`, `DivisionByZeroError`, …) is resolved to its
    /// matching prelude class by name, and a Throwable object is *synthesized*
    /// (with its message) only if a clause actually binds it (step 20-3).
    fn handle_thrown(
        &mut self,
        e: PhpError,
        catches: &[crate::hir::CatchClause],
    ) -> Result<Flow, PhpError> {
        // The class id of the in-flight throwable: the object's own class for a
        // user throw, or the prelude class named by an engine error.
        let obj_cid = match &e {
            // `exit`/`die` is uncatchable (step 46): never offered to a `catch`,
            // it just keeps unwinding. The enclosing `try` still runs `finally`
            // on the way out (the generic `Err` path below `handle_thrown`).
            PhpError::Exit(_) => return Err(e),
            PhpError::Thrown(Zval::Object(o)) => o.borrow().class_id as usize,
            PhpError::Thrown(_) => return Err(e),
            engine => match self
                .class_index
                .get(engine.class_name().to_ascii_lowercase().as_bytes())
            {
                Some(&cid) => cid,
                None => return Err(e),
            },
        };
        for c in catches {
            for tname in &c.types {
                if let Some(&tid) = self.class_index.get(&tname.to_ascii_lowercase()) {
                    if self.is_instance_of(obj_cid, tid) {
                        if let Some(slot) = c.var {
                            let obj = match &e {
                                PhpError::Thrown(v) => v.clone(),
                                engine => self.synthesize_throwable(obj_cid, engine.message())?,
                            };
                            self.slot_set(slot as usize, obj);
                        }
                        return self.exec_stmts(&c.body);
                    }
                }
            }
        }
        Err(e)
    }

    /// Build a Throwable object of `class_id` carrying `message` (step 20-3), used
    /// to materialise an engine error (`TypeError`, `DivisionByZeroError`, …) when
    /// a `catch` binds it. Mirrors `eval_new`'s instance layout but sets the
    /// message/line/file directly instead of running a constructor.
    fn synthesize_throwable(&mut self, class_id: ClassId, message: &str) -> Result<Zval, PhpError> {
        let class_name = PhpStr::new(self.classes[class_id].name.to_vec());
        let props = self.collect_props(class_id)?;
        let info = self.class_shape(class_id);
        let id = self.next_id();
        let value = Zval::Object(Rc::new(RefCell::new(Object {
            class_id: class_id as u32,
            class_name,
            props,
            id,
            info,
        })));
        let (trace, trace_string) = self.capture_trace();
        if let Zval::Object(o) = &value {
            let mut b = o.borrow_mut();
            b.props
                .set(b"message", Zval::Str(PhpStr::new(message.as_bytes().to_vec())));
            b.props.set(b"line", Zval::Long(self.cur_line as i64));
            b.props
                .set(b"file", Zval::Str(PhpStr::new(self.file.to_vec())));
            b.props.set(b"trace", trace);
            b.props
                .set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
        }
        Ok(value)
    }

    /// Snapshot the current call stack as a Throwable's `(trace array, trace
    /// string)` (step 28). Frames are innermost-first; the final line is
    /// `#N {main}`. The array mirrors PHP's `getTrace()` shape (file / line /
    /// function / class / type / empty args).
    pub(super) fn capture_trace(&self) -> (Zval, Vec<u8>) {
        let file = self.file;
        let mut arr = PhpArray::new();
        let mut s: Vec<u8> = Vec::new();
        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            s.extend_from_slice(format!("#{i} ").as_bytes());
            s.extend_from_slice(file);
            s.extend_from_slice(format!("({}): ", frame.line).as_bytes());
            s.extend_from_slice(&frame_display(frame));
            s.extend_from_slice(b"()\n");

            let mut fr = PhpArray::new();
            fr.insert(Key::from_bytes(b"file"), Zval::Str(PhpStr::new(file.to_vec())));
            fr.insert(Key::from_bytes(b"line"), Zval::Long(frame.line));
            fr.insert(
                Key::from_bytes(b"function"),
                Zval::Str(PhpStr::new(frame.function.clone())),
            );
            if let Some(class) = &frame.class {
                fr.insert(Key::from_bytes(b"class"), Zval::Str(PhpStr::new(class.clone())));
                let ty: &[u8] = if frame.is_static { b"::" } else { b"->" };
                fr.insert(Key::from_bytes(b"type"), Zval::Str(PhpStr::new(ty.to_vec())));
            }
            fr.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(PhpArray::new())));
            let _ = arr.append(Zval::Array(Rc::new(fr)));
        }
        s.extend_from_slice(format!("#{} {{main}}", self.call_stack.len()).as_bytes());
        (Zval::Array(Rc::new(arr)), s)
    }

    /// Run a loop body once and translate its control-flow signal relative to
    /// *this* loop level.
    fn loop_step(&mut self, body: &[Stmt]) -> Result<LoopStep, PhpError> {
        Ok(match self.exec_stmts(body)? {
            Flow::Normal | Flow::Continue(1) => LoopStep::Iterate,
            Flow::Continue(n) => LoopStep::Propagate(Flow::Continue(n - 1)),
            Flow::Break(1) => LoopStep::Stop,
            Flow::Break(n) => LoopStep::Propagate(Flow::Break(n - 1)),
            Flow::Return(v) => LoopStep::Propagate(Flow::Return(v)),
            // A `goto` whose label was not found in this loop body (else
            // `exec_stmts` would have jumped) targets an enclosing scope: leave
            // the loop and keep searching outward (step 45). Jumping *into* a
            // loop is rejected at lowering, so this only ever exits a loop.
            Flow::Goto(l) => LoopStep::Propagate(Flow::Goto(l)),
        })
    }

    /// `for` condition list: every expression runs, the last one's truthiness
    /// controls the loop; an empty list means "always true".
    fn eval_for_cond(&mut self, cond: &[Expr]) -> Result<bool, PhpError> {
        let mut truthy = true;
        for c in cond {
            truthy = convert::to_bool(&self.eval(c)?, &mut self.diags);
        }
        Ok(truthy)
    }

    pub(super) fn eval_bool(&mut self, e: &Expr) -> Result<bool, PhpError> {
        let v = self.eval(e)?;
        Ok(convert::to_bool(&v, &mut self.diags))
    }

    /// `foreach`: by-value iteration over an array snapshot (so mutating the
    /// source array in the body does not perturb the iteration — PHP's
    /// copy-on-write `foreach` semantics for arrays).
    fn exec_foreach(
        &mut self,
        iter: &Expr,
        key: Option<Slot>,
        value: Slot,
        by_ref: bool,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        // A by-reference loop binds each element of the *source variable* in
        // place (step 11d-3). Over a non-variable it would have nothing to write
        // back to, so it degrades to by-value iteration (PHP tolerates this).
        if by_ref {
            if let ExprKind::Var(slot) = iter.kind {
                return self.exec_foreach_by_ref(slot, key, value, body);
            }
        }
        let collection = self.eval(iter)?;
        // Snapshot raw element clones: a plain value is frozen for the loop, but a
        // reference element keeps sharing its cell, so its value is read live at
        // bind time (this is what makes the lingering-reference gotcha work).
        let items: Vec<(Key, Zval)> = match collection {
            Zval::Array(a) => a.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            // A generator is driven live (step 39-2): unlike an array it is not
            // snapshotted — each iteration advances it.
            Zval::Generator(gs) => return self.foreach_generator(&gs, key, value, body),
            other => {
                self.diags.push(Diag::Warning(format!(
                    "foreach() argument must be of type array|object, {} given",
                    php_type_name(&other)
                )));
                return Ok(Flow::Normal);
            }
        };
        for (k, v) in items {
            if let Some(ks) = key {
                self.slot_set(ks as usize, key_to_zval(&k));
            }
            self.slot_set(value as usize, v.deref_clone());
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
        }
        Ok(Flow::Normal)
    }

    /// `foreach ($var as [$k =>] &$value)`: bind each element of the source
    /// variable's array by reference (D-R13). The keys are snapshotted once; each
    /// element is promoted to a `Zval::Ref` and the value slot aliases its cell,
    /// so body writes land in the array. The value slot is intentionally *not*
    /// reset afterwards — it lingers as a reference to the last element (the PHP
    /// gotcha, oracle-verified).
    fn exec_foreach_by_ref(
        &mut self,
        slot: Slot,
        key: Option<Slot>,
        value: Slot,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        let keys: Vec<Key> = match self.slot_clone(slot as usize) {
            Zval::Array(a) => a.iter().map(|(k, _)| k.clone()).collect(),
            other => {
                self.diags.push(Diag::Warning(format!(
                    "foreach() argument must be of type array|object, {} given",
                    php_type_name(&other)
                )));
                return Ok(Flow::Normal);
            }
        };
        for k in keys {
            let step = [Step::Key(k.clone())];
            let cell = {
                let d = &mut self.diags;
                place_cell(&mut frame_mut!(self)[slot as usize], &step, d)?
            };
            if let Some(ks) = key {
                self.slot_set(ks as usize, key_to_zval(&k));
            }
            frame_mut!(self)[value as usize] = Zval::Ref(Rc::clone(&cell));
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
        }
        Ok(Flow::Normal)
    }

    /// `foreach ($gen as [$k =>] $v)` over a generator (step 39-2). Drives it
    /// live — start, then on each iteration bind the current `(key, value)`, run
    /// the body, and advance — rather than snapshotting like an array. The
    /// generator's own key (already a `Zval`) is bound directly.
    fn foreach_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        key: Option<Slot>,
        value: Slot,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        self.ensure_started(gs_rc)?;
        loop {
            let (k, v, done) = {
                let gs = gs_rc.borrow();
                (
                    gs.cur_key.clone(),
                    gs.cur_val.clone(),
                    matches!(gs.status, GenStatus::Done),
                )
            };
            if done {
                break;
            }
            if let Some(ks) = key {
                self.slot_set(ks as usize, k);
            }
            self.slot_set(value as usize, v.deref_clone());
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
            self.resume_generator(gs_rc, Zval::Null)?;
        }
        Ok(Flow::Normal)
    }

    /// `switch`: loose-`==` matching, fall-through, and `default`. The case
    /// expressions are evaluated in source order until one matches (PHP's scan
    /// semantics); a `break`/`continue` at level 1 leaves the switch.
    fn exec_switch(&mut self, subject: &Expr, cases: &[crate::hir::Case]) -> Result<Flow, PhpError> {
        let subj = self.eval(subject)?;
        let mut start = None;
        for (i, c) in cases.iter().enumerate() {
            if let Some(test) = &c.test {
                let tv = self.eval(test)?;
                if ops::loose_eq(&subj, &tv) {
                    start = Some(i);
                    break;
                }
            }
        }
        // No matching case: fall back to `default`, wherever it sits.
        let start = match start.or_else(|| cases.iter().position(|c| c.test.is_none())) {
            Some(s) => s,
            None => return Ok(Flow::Normal),
        };
        for c in &cases[start..] {
            match self.exec_stmts(&c.body)? {
                Flow::Normal => {}
                // `break`/`continue` at this level both leave the switch
                // (a `switch` counts as one level for `continue`, per PHP).
                Flow::Break(1) | Flow::Continue(1) => return Ok(Flow::Normal),
                Flow::Break(n) => return Ok(Flow::Break(n - 1)),
                Flow::Continue(n) => return Ok(Flow::Continue(n - 1)),
                Flow::Return(v) => return Ok(Flow::Return(v)),
                // A `goto` whose label was not found inside this case body
                // targets an enclosing scope — leave the switch (jumping *into*
                // a switch is rejected at lowering). Step 45.
                Flow::Goto(l) => return Ok(Flow::Goto(l)),
            }
        }
        Ok(Flow::Normal)
    }

    // --- user functions ---

}
