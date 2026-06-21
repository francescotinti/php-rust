//! Call machinery: user-function and method invocation (`call_user_fn`,
//! `run_user_fn_body`, `bind_params`), closures, named/spread argument handling,
//! and the generator runtime (`make_generator`/`resume_generator`/`gen_suspend`,
//! `yield from`). Split out of `eval.rs` (step 60); behaviour is unchanged.
use std::cell::RefCell;
use std::rc::Rc;

use corosensei::{Coroutine, Yielder};

use php_types::{
    convert, Closure, ClosureInfo, ClosureRender, Diag, GenKey, GenState, GenStatus, GenStep, Key, PhpArray,
    PhpError, PhpStr, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx};
use crate::hir::{
    Capture, Expr, ExprKind, FnDecl, Param, TypeHint,
};

use super::*;

impl<'p> Evaluator<'p> {
    /// Reject a call that would push the PHP call stack past [`MAX_CALL_DEPTH`],
    /// before recursing further on the native stack (see the constant's docs).
    /// `call_stack` already tracks every active function/method frame, so its
    /// length is the current call depth.
    pub(super) fn guard_call_depth(&self) -> Result<(), PhpError> {
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(PhpError::Error(format!(
                "Maximum call stack depth of {MAX_CALL_DEPTH} exceeded"
            )));
        }
        Ok(())
    }

    /// Invoke a hoisted user function: validate arity, set up a fresh local
    /// frame (its own slot table and slot names), bind parameters, run the body,
    /// then restore the caller's frame. Recursion uses the host (Rust) stack.
    pub(super) fn call_user_fn(&mut self, idx: usize, argv: Vec<Arg>) -> Result<Zval, PhpError> {
        self.guard_call_depth()?;
        // `funcs` is `&'p [FnDecl]` (Copy): copying it out detaches the borrow
        // from `self`, so installing the local overlay below can mutate the
        // active frame freely.
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];

        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
        // A required parameter must have a real argument at its index; named
        // arguments (step 38) can leave `Arg::Default` gaps, so the supplied
        // count is not enough — check each required slot directly.
        let missing_required = f
            .params
            .iter()
            .enumerate()
            .any(|(i, p)| {
                p.default.is_none()
                    && !p.variadic
                    && !matches!(argv.get(i), Some(Arg::Val(_) | Arg::Ref(_)))
            });
        if missing_required {
            let passed = argv
                .iter()
                .filter(|a| matches!(a, Arg::Val(_) | Arg::Ref(_)))
                .count();
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}(), {} passed and {} expected",
                String::from_utf8_lossy(&f.name),
                passed,
                expected,
            )));
        }

        // Install the callee's local frame as the overlay; the global frame
        // stays put so `global $x` / `$GLOBALS` can reach it by slot (D-12.1).
        // Saving and restoring the previous overlay makes nested calls nest.
        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);

        // Record a stack frame for the duration of the body (step 28); the
        // call-site line is the line currently executing in the caller.
        self.call_stack.push(CallFrame {
            class: None,
            function: f.name.to_vec(),
            is_static: false,
            line: self.cur_line as i64,
        });
        // A generator function does not run its body on call (step 39): bind the
        // arguments into the fresh frame, then hand that frame to a lazy
        // `Generator` value whose body runs on demand inside a coroutine.
        let result = if f.is_generator {
            match self.bind_params(f, argv) {
                Ok(()) => {
                    let frame = self.locals.take().expect("callee overlay installed");
                    Ok(self.make_generator(f, frame))
                }
                Err(e) => Err(e),
            }
        } else {
            self.run_user_fn_body(f, argv)
        };
        self.call_stack.pop();

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        result
    }

    /// Bind parameters into the (already installed) callee frame and execute the
    /// body. A by-value argument installs a fresh value slot; a by-reference
    /// argument shares the caller's cell (D-R6). A missing argument falls back to
    /// its default, evaluated in the new frame; falling off the end yields NULL.
    pub(super) fn run_user_fn_body(&mut self, f: &'p FnDecl, argv: Vec<Arg>) -> Result<Zval, PhpError> {
        self.bind_params(f, argv)?;
        let ret = match self.exec_stmts(&f.body)? {
            Flow::Return(v) => v,
            // An unresolved `goto` escaping the function body can only be the
            // unsupported "jump *into* a transparent block" case (D-45.1):
            // lowering already proved the label exists in scope and is not a
            // forbidden into-loop/switch/finally jump. Surface it instead of
            // silently returning null.
            Flow::Goto(label) => return Err(unsupported_goto(&label)),
            _ => Zval::Null,
        };
        // Coerce the return value to a scalar return type (weak). A by-reference
        // function returns a `Zval::Ref` to alias, so its return type stays
        // unenforced here (scope-out, D-14.5/D-13.7).
        let strict = self.strict;
        match &f.ret_hint {
            Some(hint) if !f.by_ref => match coerce_to_hint(ret, hint, &mut self.diags, strict) {
                Ok(v) => Ok(v),
                Err(given) => Err(self.return_type_error(f, hint, given)),
            },
            _ => Ok(ret),
        }
    }

    /// Bind a call's arguments into the (already installed) callee frame's
    /// leading parameter slots. Shared by ordinary calls ([`run_user_fn_body`])
    /// and generator construction ([`make_generator`]), which binds the frame but
    /// does not run the body. By-value arguments are coerced to scalar hints
    /// (weak); by-reference arguments share the caller's cell; gaps fall back to
    /// defaults evaluated in the new frame.
    fn bind_params(&mut self, f: &'p FnDecl, argv: Vec<Arg>) -> Result<(), PhpError> {
        let strict = self.strict;
        for (i, p) in f.params.iter().enumerate() {
            // A variadic `...$rest` (always last) collects every remaining
            // argument into a 0-indexed array (step 38-5).
            if p.variadic {
                let mut arr = PhpArray::new();
                for a in argv.iter().skip(i) {
                    match a {
                        // Positional tail entries take the next free int key.
                        Arg::Val(v) => {
                            let _ = arr.append(v.clone());
                        }
                        Arg::Ref(cell) => {
                            let _ = arr.append(cell.borrow().clone());
                        }
                        // A named-into-variadic entry keeps its string key (step 40-2).
                        Arg::Named(name, v) => {
                            arr.insert(Key::Str(PhpStr::new(name.clone())), v.clone());
                        }
                        Arg::Default => continue,
                    }
                }
                frame_mut!(self)[p.slot as usize] = Zval::Array(Rc::new(arr));
                break;
            }
            let binding = match argv.get(i) {
                // A by-value argument is coerced to the parameter's scalar hint
                // under weak typing; a failure is an uncaught TypeError (D-14.4).
                // By-reference arguments and defaults are bound as-is.
                Some(Arg::Val(v)) => {
                    let val = v.clone();
                    match &p.hint {
                        Some(hint) => match coerce_to_hint(val, hint, &mut self.diags, strict) {
                            Ok(c) => c,
                            Err(given) => return Err(self.arg_type_error(f, i, p, hint, given)),
                        },
                        None => val,
                    }
                }
                Some(Arg::Ref(cell)) => Zval::Ref(Rc::clone(cell)),
                // Required params are guaranteed present by the caller's check.
                // A default is coerced to the hint too (`float $n = 0` → 0.0,
                // D-NEW-6); a valid constant default always coerces, so on the
                // unreachable failure we keep the evaluated value. `Arg::Default`
                // is a gap left by named arguments (step 38) — same path as None.
                Some(Arg::Default) | None => {
                    let v = self.eval(p.default.as_ref().expect("required arg checked"))?;
                    match &p.hint {
                        Some(hint) => {
                            coerce_to_hint(v.clone(), hint, &mut self.diags, strict).unwrap_or(v)
                        }
                        None => v,
                    }
                }
                // `Arg::Named` is only ever appended past the variadic slot, so a
                // non-variadic parameter never sees one (step 40-2 invariant).
                Some(Arg::Named(..)) => {
                    unreachable!("named-into-variadic arg reached a fixed parameter slot")
                }
            };
            frame_mut!(self)[p.slot as usize] = binding;
        }
        Ok(())
    }

    // --- generators (step 39) ---

    /// Build the lazy `Generator` value a generator function returns. The
    /// argument-bound `frame` becomes the generator's initial locals; the body is
    /// cloned into an `Rc` so the `'static` coroutine can own it. The body does
    /// not run until the generator is first advanced.
    fn make_generator(&mut self, f: &FnDecl, frame: Vec<Zval>) -> Zval {
        let body_rc: Rc<FnDecl> = Rc::new(f.clone());
        let names_ptr: *const [Box<[u8]>] = body_rc.slots.as_slice();
        let ctx = GenCtx {
            locals: frame,
            local_names: names_ptr,
            // A generator defined in a method keeps its `$this` / class context
            // (captured here); a free-function generator has none.
            cur_this: self.cur_this.clone(),
            cur_class: self.cur_class,
            cur_static_class: self.cur_static_class,
            fn_returns_ref: false,
            gen_yielder: None,
        };
        let body_for_co = Rc::clone(&body_rc);
        let co = Coroutine::new(
            move |y: &Yielder<ResumeIn, YieldOut>, first: ResumeIn| -> Result<Zval, PhpError> {
                // SAFETY: see `GenDriverImpl::resume` — `first.ev` is the live
                // evaluator, valid for the whole body run; the reborrow as
                // `'static` is a lifetime extension that never escapes the call.
                let ev: &mut Evaluator<'static> =
                    unsafe { &mut *(first.ev as *mut Evaluator<'static>) };
                ev.gen_yielder = Some(y as *const Yielder<ResumeIn, YieldOut> as *const ());
                match ev.exec_stmts(&body_for_co.body) {
                    Ok(Flow::Return(v)) => Ok(v),
                    Ok(_) => Ok(Zval::Null),
                    Err(e) => Err(e),
                }
            },
        );
        let driver = GenDriverImpl {
            co,
            ctx,
            _body: body_rc,
        };
        let id = self.next_object_id;
        self.next_object_id += 1;
        Zval::Generator(Rc::new(RefCell::new(GenState {
            id,
            func_name: f.name.clone(),
            status: GenStatus::NotStarted,
            advanced: false,
            cur_key: Zval::Null,
            cur_val: Zval::Null,
            ret: Zval::Null,
            auto_key: 0,
            driver: Some(Box::new(driver)),
        })))
    }

    /// Drive a generator one step: resume its coroutine with `sent` (the value
    /// the suspended `yield` evaluates to), then record the outcome — a new
    /// `(key, value)` (resolving the auto-key) or completion (`getReturn` value /
    /// a propagated exception). The driver is taken out of [`GenState`] for the
    /// duration so a re-entrant resume of the *same* generator sees `Running` and
    /// errors cleanly (also upholding the reborrow's non-aliasing invariant).
    pub(super) fn resume_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        sent: Zval,
    ) -> Result<(), PhpError> {
        let mut driver = {
            let mut gs = gs_rc.borrow_mut();
            match gs.status {
                GenStatus::Running => {
                    return Err(PhpError::Error(
                        "Cannot resume an already running generator".to_string(),
                    ))
                }
                GenStatus::Done => return Ok(()),
                // Resuming an already-suspended generator advances it past its
                // first element, which disallows a later `rewind()` (step 39-7).
                GenStatus::Suspended => gs.advanced = true,
                GenStatus::NotStarted => {}
            }
            gs.status = GenStatus::Running;
            gs.driver
                .take()
                .expect("driver present while generator not done")
        };
        // The borrow on `gs_rc` is released here, so the body may legally call
        // back into other generators (and a re-entrant call on *this* one hits
        // the `Running` guard above instead of a RefCell double-borrow).
        let step = driver.resume(sent, self as *mut Self as *mut ());
        let mut gs = gs_rc.borrow_mut();
        match step {
            GenStep::Yielded { key, value } => {
                let resolved = match key {
                    GenKey::Auto => {
                        let k = Zval::Long(gs.auto_key);
                        gs.auto_key += 1;
                        k
                    }
                    GenKey::Keyed(z) => {
                        // An integer key `>=` the counter advances it, mirroring
                        // array append semantics (D-GEN auto-key).
                        if let Zval::Long(n) = &z {
                            if *n >= gs.auto_key {
                                gs.auto_key = n.wrapping_add(1);
                            }
                        }
                        z
                    }
                    // `yield from` keys are forwarded as-is and do not advance the
                    // outer counter (step 39-6).
                    GenKey::Verbatim(z) => z,
                };
                gs.cur_key = resolved;
                gs.cur_val = value;
                gs.status = GenStatus::Suspended;
                gs.driver = Some(driver);
            }
            GenStep::Returned(res) => {
                gs.status = GenStatus::Done;
                gs.cur_key = Zval::Null;
                gs.cur_val = Zval::Null;
                // `driver` is dropped here (not stored back), unwinding/freeing the
                // coroutine stack.
                match res {
                    Ok(v) => gs.ret = v,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// Suspend the active generator at a `yield`, handing out `(key, value)` and
    /// returning the value the next resume delivers (step 39). The single point
    /// the `yield` / `yield from` arms reach the running coroutine's `Yielder`.
    pub(super) fn gen_suspend(&mut self, key: GenKey, value: Zval) -> Result<Zval, PhpError> {
        let yptr = self.gen_yielder.ok_or_else(|| {
            PhpError::Error("Cannot use \"yield\" outside a generator".to_string())
        })?;
        // SAFETY: `gen_yielder` is set by the active generator body (in
        // `make_generator`'s coroutine) and read only while that body runs; the
        // `Yielder` lives for the coroutine's whole lifetime.
        let y = unsafe { &*(yptr as *const Yielder<ResumeIn, YieldOut>) };
        let resumed = y.suspend(YieldOut { key, value });
        Ok(resumed.sent)
    }

    /// `yield from <iterator>` (step 39-6): re-yield every element of the
    /// delegate *verbatim* (keys preserved, the outer auto-key counter
    /// untouched). For an array the expression evaluates to NULL; for a
    /// sub-generator it drives it (forwarding `send()` values in) and evaluates
    /// to its `return` value.
    pub(super) fn eval_yield_from(&mut self, iter: &Expr) -> Result<Zval, PhpError> {
        let src = self.eval(iter)?.deref_clone();
        match src {
            Zval::Array(a) => {
                let pairs: Vec<(Key, Zval)> =
                    a.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
                for (k, v) in pairs {
                    // A sent value is discarded when delegating to an array.
                    self.gen_suspend(GenKey::Verbatim(key_to_zval(&k)), v)?;
                }
                Ok(Zval::Null)
            }
            Zval::Generator(sub) => {
                self.ensure_started(&sub)?;
                loop {
                    let (k, v, done) = {
                        let g = sub.borrow();
                        (
                            g.cur_key.clone(),
                            g.cur_val.clone(),
                            matches!(g.status, GenStatus::Done),
                        )
                    };
                    if done {
                        break;
                    }
                    // Re-yield the sub-generator's current pair; forward the value
                    // the consumer sends back into the sub-generator.
                    let sent = self.gen_suspend(GenKey::Verbatim(k), v)?;
                    self.resume_generator(&sub, sent)?;
                }
                // The `yield from` expression evaluates to the delegate's return.
                let ret = sub.borrow().ret.clone();
                Ok(ret)
            }
            // `yield from` over a user `Traversable`/`Iterator` is a companion of
            // the (still scoped-out) generic `foreach` over objects; catalogue if
            // the corpus needs it (step 39 scope-out). The message matches PHP's
            // exactly (Zend/zend_generators.c).
            _ => Err(PhpError::Error(
                "Can use \"yield from\" only with arrays and Traversables".to_string(),
            )),
        }
    }

    /// Start a generator if it has not run yet (PHP starts lazily on the first
    /// `current`/`key`/`valid`/`next`/`foreach`).
    pub(super) fn ensure_started(&mut self, gs_rc: &Rc<RefCell<GenState>>) -> Result<(), PhpError> {
        if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
            self.resume_generator(gs_rc, Zval::Null)?;
        }
        Ok(())
    }

    /// Built-in methods on a `Generator` value (the `Iterator` interface plus
    /// `send`/`getReturn`), step 39. Dispatched like [`closure_method`], ahead of
    /// user-class method resolution.
    pub(super) fn generator_method(
        &mut self,
        gs_rc: Rc<RefCell<GenState>>,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"current") {
            self.ensure_started(&gs_rc)?;
            Ok(gs_rc.borrow().cur_val.clone())
        } else if method.eq_ignore_ascii_case(b"key") {
            self.ensure_started(&gs_rc)?;
            Ok(gs_rc.borrow().cur_key.clone())
        } else if method.eq_ignore_ascii_case(b"next") {
            self.ensure_started(&gs_rc)?;
            self.resume_generator(&gs_rc, Zval::Null)?;
            Ok(Zval::Null)
        } else if method.eq_ignore_ascii_case(b"valid") {
            self.ensure_started(&gs_rc)?;
            let done = matches!(gs_rc.borrow().status, GenStatus::Done);
            Ok(Zval::Bool(!done))
        } else if method.eq_ignore_ascii_case(b"rewind") {
            // Starts the generator (lazily). Rewinding one already advanced past
            // its first element is a fatal (step 39-7).
            self.ensure_started(&gs_rc)?;
            if gs_rc.borrow().advanced {
                return Err(PhpError::Error(
                    "Cannot rewind a generator that was already run".to_string(),
                ));
            }
            Ok(Zval::Null)
        } else if method.eq_ignore_ascii_case(b"send") {
            // Resume delivering `$value` as the result of the suspended `yield`
            // (step 39-4). An unstarted generator is primed first.
            let value = argv.into_iter().next().unwrap_or(Zval::Null);
            if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
                self.resume_generator(&gs_rc, Zval::Null)?;
            }
            self.resume_generator(&gs_rc, value)?;
            Ok(gs_rc.borrow().cur_val.clone())
        } else if method.eq_ignore_ascii_case(b"getReturn") {
            // PHP auto-primes here: getReturn() on a fresh generator starts it
            // (so one whose body returns before any yield exposes its value); if
            // it has not yet returned, that is an Error.
            self.ensure_started(&gs_rc)?;
            if !matches!(gs_rc.borrow().status, GenStatus::Done) {
                return Err(PhpError::Error(
                    "Cannot get return value of a generator that hasn't returned".to_string(),
                ));
            }
            Ok(gs_rc.borrow().ret.clone())
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Generator::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }

    /// Build the uncaught `TypeError` for a return value that failed scalar
    /// coercion (D-14.5). The message format differs from the argument one: no
    /// call site, suffix `returned in <file>:<defline>`.
    fn return_type_error(&self, f: &FnDecl, hint: &TypeHint, given: &str) -> PhpError {
        PhpError::TypeError(format!(
            "{}(): Return value must be of type {}, {} returned in {}:{}",
            String::from_utf8_lossy(&f.name),
            hint.display_name(),
            given,
            String::from_utf8_lossy(self.file),
            f.line,
        ))
    }

    /// Build the uncaught `TypeError` for an argument that failed scalar
    /// coercion, matching PHP's exact message (D-14.4). `cur_line` is the call
    /// site's line; `f.line` is the definition line.
    fn arg_type_error(&self, f: &FnDecl, i: usize, p: &Param, hint: &TypeHint, given: &str) -> PhpError {
        let file = String::from_utf8_lossy(self.file);
        PhpError::TypeError(format!(
            "{}(): Argument #{} (${}) must be of type {}, {} given, \
             called in {} on line {} and defined in {}:{}",
            String::from_utf8_lossy(&f.name),
            i + 1,
            String::from_utf8_lossy(&f.slots[p.slot as usize]),
            hint.display_name(),
            given,
            file,
            self.cur_line,
            file,
            f.line,
        ))
    }

    /// Resolve a user-function call's arguments against its declaration: by-value
    /// params evaluate normally; a `&$x` param binds the argument variable's
    /// shared cell (promoting it). A non-variable argument to a by-ref param is
    /// an uncaught `Error` (PHP 8.x; oracle-verified message).
    /// Evaluate a user function's call arguments into the positional `argv` plus
    /// the named arguments produced by unpacking string keys (step 40). Plain
    /// positional arguments (which lowering guarantees precede any spread) honour
    /// by-reference parameters; unpacked values are always by value.
    pub(super) fn eval_call_args(
        &mut self,
        idx: usize,
        args: &[Expr],
    ) -> Result<(Vec<Arg>, SpreadNamed), PhpError> {
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];
        let mut out: Vec<Arg> = Vec::with_capacity(args.len());
        let mut named: SpreadNamed = Vec::new();
        for a in args {
            if let ExprKind::Spread(inner) = &a.kind {
                let mut pos = Vec::new();
                self.expand_spread(inner, &mut pos, &mut named)?;
                out.extend(pos.into_iter().map(Arg::Val));
                continue;
            }
            // A plain positional binds at the next positional slot; only these
            // (never unpacked values) may target a by-reference parameter.
            let i = out.len();
            let by_ref = f.params.get(i).is_some_and(|p| p.by_ref);
            if by_ref {
                match &a.kind {
                    ExprKind::Var(slot) => out.push(Arg::Ref(self.slot_cell(*slot as usize))),
                    _ => {
                        let p = &f.params[i];
                        return Err(PhpError::Error(format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(&f.name),
                            i + 1,
                            String::from_utf8_lossy(&f.slots[p.slot as usize]),
                        )));
                    }
                }
            } else {
                out.push(Arg::Val(self.eval(a)?));
            }
        }
        Ok((out, named))
    }

    /// The catchable `Error` PHP raises for an unresolvable named argument; used
    /// for the scope-out targets (closures, `__call`, enum statics) that have no
    /// declared parameter list to bind names against (step 38).
    /// Reject any named argument — explicit (step 38) or produced by unpacking
    /// string keys (step 40) — for a target that has no parameter list to bind
    /// against (closures, generators, `__call`). Returns the unknown-parameter
    /// `Error` for the first offending name, or `None` if there are none.
    pub(super) fn reject_named(
        &self,
        named: &[(Box<[u8]>, Expr)],
        spread_named: &[(Box<[u8]>, Zval)],
    ) -> Option<PhpError> {
        let name = named
            .first()
            .map(|(n, _)| n.as_ref())
            .or_else(|| spread_named.first().map(|(n, _)| n.as_ref()))?;
        Some(PhpError::Error(format!(
            "Unknown named parameter ${}",
            String::from_utf8_lossy(name)
        )))
    }

    /// Place one already-built named argument into `argv` by parameter name
    /// (steps 38 / 40-2). A matching non-variadic parameter takes the value at
    /// its slot (filling earlier gaps with `Arg::Default`); an already-filled
    /// slot is an overwrite `Error`. With no matching parameter, a trailing
    /// variadic collects the value keyed by name ([`Arg::Named`]); otherwise the
    /// name is an unknown-parameter `Error`. Messages match PHP.
    fn place_named_arg(
        &self,
        argv: &mut Vec<Arg>,
        f: &FnDecl,
        name: &[u8],
        arg: Arg,
    ) -> Result<(), PhpError> {
        if let Some(j) = f
            .params
            .iter()
            .position(|p| !p.variadic && f.slots[p.slot as usize][..] == name[..])
        {
            // A positional argument already occupied this slot, or a duplicate
            // named argument targets it (an `Arg::Default` gap is not "previous").
            if matches!(argv.get(j), Some(Arg::Val(_) | Arg::Ref(_))) {
                return Err(PhpError::Error(format!(
                    "Named parameter ${} overwrites previous argument",
                    String::from_utf8_lossy(name)
                )));
            }
            if argv.len() <= j {
                argv.resize_with(j + 1, || Arg::Default);
            }
            argv[j] = arg;
            Ok(())
        } else if f.params.last().is_some_and(|p| p.variadic) {
            // No matching fixed parameter, but a trailing `...$rest` collects the
            // named argument keyed by its name (step 40-2). A by-reference value
            // is dereferenced — variadics collect by value.
            let val = match arg {
                Arg::Val(v) => v,
                Arg::Ref(cell) => cell.borrow().clone(),
                Arg::Default => return Ok(()),
                Arg::Named(_, v) => v,
            };
            argv.push(Arg::Named(name.into(), val));
            Ok(())
        } else {
            Err(PhpError::Error(format!(
                "Unknown named parameter ${}",
                String::from_utf8_lossy(name)
            )))
        }
    }

    /// Apply a call's named arguments to the positional `argv`: first the named
    /// arguments produced by string keys during unpacking (step 40, already
    /// evaluated, by value), then the explicit `name: value` arguments (step 38),
    /// each evaluated in the caller frame. A name targeting a by-reference
    /// parameter binds the caller's variable cell (step 38-4).
    pub(super) fn apply_named_args(
        &mut self,
        f: &'p FnDecl,
        mut argv: Vec<Arg>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Vec<Arg>, PhpError> {
        for (name, val) in spread_named {
            self.place_named_arg(&mut argv, f, &name, Arg::Val(val))?;
        }
        for (name, expr) in named {
            // A by-reference parameter binds the caller's variable cell when the
            // named value is a plain variable (mirrors `eval_call_args`); a
            // non-variable to a by-ref param is the same fatal as positionally.
            let target = f
                .params
                .iter()
                .position(|p| !p.variadic && f.slots[p.slot as usize][..] == name[..]);
            let arg = match target {
                Some(j) if f.params[j].by_ref => match &expr.kind {
                    ExprKind::Var(slot) => Arg::Ref(self.slot_cell(*slot as usize)),
                    _ => {
                        return Err(PhpError::Error(format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(&f.name),
                            j + 1,
                            String::from_utf8_lossy(name),
                        )))
                    }
                },
                _ => Arg::Val(self.eval(expr)?),
            };
            self.place_named_arg(&mut argv, f, name, arg)?;
        }
        Ok(argv)
    }

    /// Expand one unpacked value (`...$e`, step 40) into the positional `pos`
    /// stream and the `named` stream. Array/Traversable int keys append to
    /// `pos` in iteration order (the key value is ignored); string keys append
    /// to `named`. An int key after any string key already emitted during this
    /// call's unpacking is a catchable `Error`; a non-iterable is a `TypeError`.
    pub(super) fn expand_spread(
        &mut self,
        inner: &Expr,
        pos: &mut Vec<Zval>,
        named: &mut SpreadNamed,
    ) -> Result<(), PhpError> {
        // A positional value produced by unpacking is rejected once any named
        // (string-keyed) value has already been emitted, matching PHP.
        macro_rules! push_pos {
            ($v:expr) => {{
                if !named.is_empty() {
                    return Err(PhpError::Error(
                        "Cannot use positional argument after named argument during unpacking"
                            .to_string(),
                    ));
                }
                pos.push($v);
            }};
        }
        let value = self.eval(inner)?.deref_clone();
        match value {
            Zval::Array(arr) => {
                for (k, v) in arr.iter() {
                    match k {
                        Key::Int(_) => push_pos!(v.clone()),
                        Key::Str(s) => named.push((s.as_bytes().into(), v.clone())),
                    }
                }
                Ok(())
            }
            Zval::Generator(gs) => {
                self.ensure_started(&gs)?;
                loop {
                    let (k, v, done) = {
                        let g = gs.borrow();
                        (
                            g.cur_key.clone(),
                            g.cur_val.deref_clone(),
                            matches!(g.status, GenStatus::Done),
                        )
                    };
                    if done {
                        break;
                    }
                    match k {
                        Zval::Str(s) => named.push((s.as_bytes().into(), v)),
                        _ => push_pos!(v),
                    }
                    self.resume_generator(&gs, Zval::Null)?;
                }
                Ok(())
            }
            other => Err(PhpError::TypeError(format!(
                "Only arrays and Traversables can be unpacked, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// Invoke a by-reference builtin (step 11c). Its first argument must be a
    /// variable: that variable's storage cell is bound and handed to the builtin
    /// as `&mut Zval`, so the builtin's mutation writes through to the caller.
    /// The remaining arguments are evaluated by value. A missing or non-variable
    /// first argument raises the shared `$array`-family errors (oracle-verified).
    pub(super) fn call_ref_builtin(
        &mut self,
        f: BuiltinRefFn,
        name: &[u8],
        args: &[Expr],
    ) -> Result<Zval, PhpError> {
        let Some((first, rest_exprs)) = args.split_first() else {
            return Err(PhpError::ArgumentCountError(format!(
                "{}() expects at least 1 argument, 0 given",
                String::from_utf8_lossy(name)
            )));
        };
        let ExprKind::Var(slot) = first.kind else {
            return Err(PhpError::Error(format!(
                "{}(): Argument #1 ($array) could not be passed by reference",
                String::from_utf8_lossy(name)
            )));
        };
        // Evaluate the by-value tail before binding the cell (binding a variable
        // has no side effect, so this preserves left-to-right argument order).
        let mut rest = Vec::with_capacity(rest_exprs.len());
        for a in rest_exprs {
            rest.push(self.eval(a)?);
        }
        let cell = self.slot_cell(slot as usize);
        let mut guard = cell.borrow_mut();
        let target = &mut *guard;
        // Like value builtins: flush pending diagnostics, run, mirror fresh
        // output into `rendered`, then flush the builtin's own diagnostics.
        self.flush_diags();
        let pre = self.out.len();
        let mut ctx = Ctx {
            out: &mut self.out,
            diags: &mut self.diags,
        };
        let res = f(target, &rest, &mut ctx);
        let produced = self.out[pre..].to_vec();
        self.rendered.extend_from_slice(&produced);
        self.flush_diags();
        res
    }

    /// Run a by-value builtin, mirroring its fresh stdout into `rendered` and
    /// flushing its diagnostics, exactly like the `Call` dispatch path. Shared by
    /// direct calls and dynamic string-callable dispatch (step 18).
    pub(super) fn dispatch_value_builtin(
        &mut self,
        f: crate::builtin::BuiltinFn,
        argv: &[Zval],
    ) -> Result<Zval, PhpError> {
        self.flush_diags();
        let pre = self.out.len();
        let mut ctx = Ctx {
            out: &mut self.out,
            diags: &mut self.diags,
        };
        let res = f(argv, &mut ctx);
        // A builtin that writes to stdout (printf/vprintf) emits its warnings
        // while *formatting* the arguments — before the formatted result is
        // written. So render the diagnostics it raised first, then its output, to
        // match PHP's interleaving (e.g. "Array to string conversion" prints
        // ahead of the printf result, not after it).
        let produced = self.out[pre..].to_vec();
        self.flush_diags();
        self.rendered.extend_from_slice(&produced);
        res
    }

    /// Allocate the next monotonic object handle (the `#N` in `var_dump`).
    pub(super) fn next_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Build the render metadata for a first-class callable `name(...)`: the
    /// parameters are known only when it wraps a user function; a builtin target
    /// has no signature available, so its parameter list is empty (step 18-7
    /// scope-out).
    pub(super) fn first_class_info(&self, name: &[u8]) -> Rc<ClosureInfo> {
        let params = match self.fn_index.get(&name.to_ascii_lowercase()) {
            Some(&idx) => closure_params_for(&self.funcs[idx]),
            None => Vec::new(),
        };
        Rc::new(ClosureInfo {
            kind: ClosureRender::Function(PhpStr::new(name.to_vec())),
            params,
        })
    }

    /// Snapshot a closure's captured variables in the *active* frame (step 18,
    /// D-18.3): a by-value `use($x)` reads (and warns on undefined) the value; a
    /// by-reference `use(&$x)` shares the variable's cell as a `Zval::Ref`.
    pub(super) fn bind_captures(&mut self, captures: &[Capture]) -> Vec<(u32, Zval)> {
        let mut out = Vec::with_capacity(captures.len());
        for cap in captures {
            let val = if cap.by_ref {
                Zval::Ref(self.slot_cell(cap.src as usize))
            } else {
                self.read_var(cap.src)
            };
            out.push((cap.dst, val));
        }
        out
    }

    /// Invoke a runtime callable value (step 18, D-18.5): a closure runs its
    /// lowered body; a string names a user function or builtin; anything else is
    /// an uncaught `Error` ("Value of type X is not callable").
    pub(super) fn call_value(&mut self, callee: Zval, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                // A first-class callable dispatches like a string callable.
                Some(name) => self.call_named(name.as_bytes(), argv),
                None => self.call_closure(&cl, argv),
            },
            Zval::Str(s) => self.call_named(s.as_bytes(), argv),
            Zval::Ref(cell) => {
                let inner = cell.borrow().clone();
                self.call_value(inner, argv)
            }
            // An object is callable iff it defines `__invoke` (step 22, D-22.7).
            Zval::Object(ref o) => {
                let cid = o.borrow().class_id as usize;
                match self.resolve_method(cid, b"__invoke") {
                    Some((defc, m)) => {
                        self.invoke_method(Some(callee.clone()), defc, cid, m, b"__invoke", argv)
                    }
                    None => Err(PhpError::Error(format!(
                        "Object of type {} is not callable",
                        String::from_utf8_lossy(&self.classes[cid].name)
                    ))),
                }
            }
            other => Err(PhpError::Error(format!(
                "Value of type {} is not callable",
                other.error_type_name()
            ))),
        }
    }

    /// Dispatch a string callable to a user function (shadows builtins) or a
    /// by-value builtin. Arguments arrive by value (by-reference parameters in a
    /// dynamic string call are a scope-out, D-18.5).
    fn call_named(&mut self, name: &[u8], argv: Vec<Zval>) -> Result<Zval, PhpError> {
        if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
            let args: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
            let result = self.call_user_fn(idx, args)?;
            return Ok(match result {
                Zval::Ref(cell) => cell.borrow().clone(),
                other => other,
            });
        }
        // Constant builtins need the evaluator's `define()` table, so they are
        // dispatched here ahead of the stateless registry (step 49c).
        if let Some(result) = self.call_constant_builtin(name, &argv) {
            return result;
        }
        match self.reg.get(name).copied() {
            Some(Builtin::Value(f)) => self.dispatch_value_builtin(f, &argv),
            // String-calling a by-reference builtin (sort/array_push/…) is a
            // documented scope-out (it needs a variable, not a value).
            Some(Builtin::RefFirst(_)) => Err(PhpError::Error(format!(
                "{}(): cannot be called dynamically with a by-value argument",
                String::from_utf8_lossy(name)
            ))),
            None => Err(PhpError::Error(format!(
                "Call to undefined function {}()",
                String::from_utf8_lossy(name)
            ))),
        }
    }

    /// `define()` / `defined()` / `constant()` (step 49c). Returns `None` for any
    /// other name so the caller falls through to the normal registry. The
    /// engine-constant table ([`resolve_constant`]) is consulted alongside the
    /// runtime `define()` table so `defined('PHP_INT_MAX')` etc. answer `true`.
    pub(super) fn call_constant_builtin(
        &mut self,
        name: &[u8],
        argv: &[Zval],
    ) -> Option<Result<Zval, PhpError>> {
        let known = |n: &[u8], ev: &Self| {
            ev.constants.contains_key(n) || crate::lower::resolve_constant(n).is_some()
        };
        if name.eq_ignore_ascii_case(b"define") {
            let Some(name_arg) = argv.first() else {
                return Some(Err(PhpError::Error(
                    "define() expects at least 2 arguments, 0 given".into(),
                )));
            };
            let cname = convert::to_zstr_cast(name_arg, &mut self.diags)
                .as_bytes()
                .to_vec();
            let value = argv.get(1).cloned().unwrap_or(Zval::Null);
            // Redefining an existing (user or engine) constant warns and fails.
            if known(&cname, self) {
                self.diags.push(Diag::Warning(format!(
                    "Constant {} already defined",
                    String::from_utf8_lossy(&cname)
                )));
                return Some(Ok(Zval::Bool(false)));
            }
            self.constants.insert(cname, value);
            return Some(Ok(Zval::Bool(true)));
        }
        if name.eq_ignore_ascii_case(b"defined") {
            let Some(name_arg) = argv.first() else {
                return Some(Ok(Zval::Bool(false)));
            };
            let cname = convert::to_zstr_cast(name_arg, &mut self.diags);
            return Some(Ok(Zval::Bool(known(cname.as_bytes(), self))));
        }
        if name.eq_ignore_ascii_case(b"constant") {
            let Some(name_arg) = argv.first() else {
                return Some(Err(PhpError::Error(
                    "constant() expects exactly 1 argument, 0 given".into(),
                )));
            };
            let cname = convert::to_zstr_cast(name_arg, &mut self.diags)
                .as_bytes()
                .to_vec();
            if let Some(v) = self.constants.get(&cname) {
                return Some(Ok(v.clone()));
            }
            if let Some(z) = crate::lower::resolve_constant(&cname).and_then(const_literal_to_zval) {
                return Some(Ok(z));
            }
            return Some(Err(PhpError::Error(format!(
                "Undefined constant \"{}\"",
                String::from_utf8_lossy(&cname)
            ))));
        }
        None
    }

    /// Invoke a closure value: install its frame, bind the captured variables
    /// into their slots, then bind parameters and run the body via the shared
    /// [`Evaluator::run_user_fn_body`] (step 18, D-18.2).
    fn call_closure(&mut self, cl: &Closure, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        let closures: &'p [FnDecl] = self.closures;
        let f: &'p FnDecl = &closures[cl.fn_idx];

        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
        if argv.len() < required {
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}(), {} passed and {} expected",
                String::from_utf8_lossy(&f.name),
                argv.len(),
                expected,
            )));
        }

        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);
        // Install the closure's bound `$this` (step 19-6); a static / top-level
        // closure carries `None`, so `$this` inside it is the usual fatal.
        let saved_this = std::mem::replace(&mut self.cur_this, cl.bound_this.clone());

        // Bind captured variables into their closure-frame slots before params.
        for (slot, val) in &cl.captures {
            frame_mut!(self)[*slot as usize] = val.clone();
        }

        let args: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
        // A generator closure (its body contains `yield`) returns a lazy
        // Generator just like a generator function (step 39): bind params into
        // the frame (which already holds the captures), then hand it off.
        let result = if f.is_generator {
            match self.bind_params(f, args) {
                Ok(()) => {
                    let frame = self.locals.take().expect("closure overlay installed");
                    Ok(self.make_generator(f, frame))
                }
                Err(e) => Err(e),
            }
        } else {
            self.run_user_fn_body(f, args)
        };

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        self.cur_this = saved_this;
        result.map(|r| match r {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        })
    }

    /// Build a copy of a closure with a new bound `$this` (step 19-6, D-19.19),
    /// assigning it a fresh object handle.
    fn rebind_closure(&mut self, cl: &Closure, bound_this: Option<Zval>) -> Zval {
        Zval::Closure(Rc::new(Closure {
            fn_idx: cl.fn_idx,
            captures: cl.captures.clone(),
            named: cl.named.clone(),
            bound_this,
            id: self.next_id(),
            info: Rc::clone(&cl.info),
        }))
    }

    /// Built-in methods on a closure value (`$c->bindTo(...)`, `$c->call(...)`),
    /// step 19-6, D-19.19.
    pub(super) fn closure_method(
        &mut self,
        cl: &Closure,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bindTo") {
            let new_this = closure_this_arg(argv.into_iter().next());
            Ok(self.rebind_closure(cl, new_this))
        } else if method.eq_ignore_ascii_case(b"call") {
            // `$c->call($newThis, ...args)`: bind then invoke immediately.
            let mut it = argv.into_iter();
            let new_this = closure_this_arg(it.next());
            let rest: Vec<Zval> = it.collect();
            let bound = self.rebind_closure(cl, new_this);
            self.call_value(bound, rest)
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Closure::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }

    /// Static methods on the `Closure` class (`Closure::bind`,
    /// `Closure::fromCallable`), step 19-6, D-19.19.
    pub(super) fn closure_static(&mut self, method: &[u8], argv: Vec<Zval>) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bind") {
            let mut it = argv.into_iter();
            let target = it.next().map(|v| v.deref_clone());
            let new_this = closure_this_arg(it.next());
            match target {
                Some(Zval::Closure(cl)) => Ok(self.rebind_closure(&cl, new_this)),
                _ => Err(PhpError::Error(
                    "Closure::bind(): Argument #1 ($closure) must be of type Closure".to_string(),
                )),
            }
        } else if method.eq_ignore_ascii_case(b"fromCallable") {
            match argv.into_iter().next().map(|v| v.deref_clone()) {
                Some(Zval::Closure(cl)) => Ok(Zval::Closure(cl)),
                Some(Zval::Str(s)) => {
                    let info = self.first_class_info(s.as_bytes());
                    Ok(Zval::Closure(Rc::new(Closure {
                        fn_idx: 0,
                        captures: Vec::new(),
                        named: Some(s),
                        bound_this: None,
                        id: self.next_id(),
                        info,
                    })))
                }
                _ => Err(PhpError::Error(
                    "Closure::fromCallable(): Argument #1 ($callback) is not callable".to_string(),
                )),
            }
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Closure::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }
}
