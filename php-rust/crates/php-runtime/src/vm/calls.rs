//! VM calls logic, extracted from vm/mod.rs (no semantic change).
use super::*;

/// Invoke a by-reference-first builtin, handing it `&mut Zval` for the slot cell
/// (following a `Zval::Ref` so the write lands in the shared target).
pub(super) fn builtin_ref_call(
    f: BuiltinRefFn,
    cell: &mut Zval,
    rest: &[Zval],
    out: &mut Vec<u8>,
    diags: &mut Diags,
) -> Result<Zval, PhpError> {
    let mut ctx = Ctx { out, diags };
    if let Zval::Ref(rc) = cell {
        let mut guard = rc.borrow_mut();
        f(&mut guard, rest, &mut ctx)
    } else {
        f(cell, rest, &mut ctx)
    }
}

/// The fatal a call raises when a name isn't a callable VM builtin (defensive:
/// the compiler already filters these, so this is a safety net).
pub(super) fn undefined_builtin(name: &[u8]) -> PhpError {
    PhpError::Error(format!(
        "Call to undefined function {}()",
        String::from_utf8_lossy(name)
    ))
}

/// Flatten a runtime argument array into positional values for a spread call
/// (`...$arr` feeding a dynamic-dispatch call, Session A): keys are dropped and
/// each value deref-cloned. A non-array yields no arguments. Shared by the
/// `…Args` call opcodes and [`Op::CallArgs`].
/// Convert an engine constant's literal HIR value ([`crate::lower::resolve_constant`])
/// to a [`Zval`] for `constant()` (B3); `None` for a non-scalar form.
pub(super) fn const_literal_to_zval(kind: crate::hir::ExprKind) -> Option<Zval> {
    use crate::hir::ExprKind;
    Some(match kind {
        ExprKind::Null => Zval::Null,
        ExprKind::Bool(b) => Zval::Bool(b),
        ExprKind::Int(i) => Zval::Long(i),
        ExprKind::Float(f) => Zval::Double(f),
        ExprKind::Str(b) => Zval::Str(PhpStr::new(b.into_vec())),
        _ => return None,
    })
}

pub(super) fn args_from_array_value(v: Zval) -> Vec<Zval> {
    match v.deref_clone() {
        Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
        _ => Vec::new(),
    }
}

/// Pack call arguments into a 0-indexed list array — the second argument handed
/// to `__call` / `__callStatic` (OOP-3a), mirroring the tree-walker's `pack_args`.
pub(super) fn pack_args(args: Vec<Zval>) -> Zval {
    let mut arr = PhpArray::new();
    for a in args {
        let _ = arr.append(a);
    }
    Zval::Array(Rc::new(arr))
}

/// Bind positional `args` to a callee frame's leading parameter slots (PAR).
/// Omitted parameters are left `Undef` for the body's default prologue
/// ([`Op::FillDefault`]) to fill. For a **variadic** function the leading fixed
/// params are bound and every remaining argument is collected into an array in
/// the variadic slot (empty when there are none); otherwise surplus arguments
/// are dropped (PHP silently ignores them for a non-variadic function).
pub(super) fn bind_params(frame: &mut Frame, args: Vec<Zval>) {
    frame.argc = args.len() as u32;
    match frame.func.variadic_slot {
        None => {
            let n = frame.func.n_params as usize;
            // Snapshot arguments beyond the declared parameters: they have no slot,
            // so `func_get_args` (D1) could not otherwise recover them.
            if args.len() > n {
                frame.extra_args = args[n..].to_vec();
            }
            for (i, a) in args.into_iter().enumerate() {
                if i < n {
                    frame.slots[i] = a;
                }
            }
        }
        Some(vslot) => {
            let v = vslot as usize;
            let mut it = args.into_iter();
            for slot in frame.slots.iter_mut().take(v) {
                match it.next() {
                    Some(a) => *slot = a,
                    None => break, // omitted fixed params stay Undef (default prologue)
                }
            }
            let mut rest = PhpArray::new();
            for a in it {
                let _ = rest.append(a);
            }
            frame.slots[v] = Zval::Array(Rc::new(rest));
        }
    }
}

/// The catchable `Error` PHP raises for a named argument with no place to go —
/// reported for the first name. Used when the target has no parameter list to
/// bind against (a `Generator`/`Fiber`'s native methods).
pub(super) fn unknown_named_param(named: &[(Box<[u8]>, Zval)]) -> PhpError {
    match named.first() {
        Some((name, _)) => PhpError::Error(format!(
            "Unknown named parameter ${}",
            String::from_utf8_lossy(name)
        )),
        None => PhpError::Error("Unknown named parameter".to_string()),
    }
}

/// Build a callee frame for a method call with **named** (and positional)
/// arguments, binding by name against `callee.param_names` at run time (Session
/// A). Positional values fill the leading fixed slots; each named value targets
/// its matching fixed parameter (gaps stay `Undef` for the default prologue) or,
/// with no match and a trailing `...$rest`, is collected into the variadic array
/// keyed by its name (string key) — surplus positional args are collected too (int
/// keys). Mirrors the evaluator's errors: a duplicate/positional collision is an
/// overwrite `Error`, a name with no home (and no variadic) is unknown, and a
/// required parameter left unbound is an `ArgumentCountError`. `display_name` is
/// the `Class::method` used in that message; `frame.this`/`class`/`static_class`
/// are set by the caller.
pub(super) fn build_named_frame<'m>(
    callee: &'m Func,
    module: &'m Module,
    file: &[u8],
    line: Line,
    display_name: &str,
    positional: Vec<Zval>,
    named: Vec<(Box<[u8]>, Zval)>,
) -> Result<Frame<'m>, PhpError> {
    let n_params = callee.n_params as usize;
    let fixed = match callee.variadic_slot {
        Some(s) => s as usize,
        None => n_params,
    };
    let has_variadic = callee.variadic_slot.is_some();
    let passed = positional.len() + named.len();
    let mut frame = Frame::new(callee, module);
    let mut variadic = PhpArray::new();
    // Positional args fill the leading fixed slots; surplus goes to the variadic.
    for (i, a) in positional.into_iter().enumerate() {
        if i < fixed {
            frame.slots[i] = a;
        } else if has_variadic {
            let _ = variadic.append(a);
        }
    }
    // Named args target a fixed parameter by name, or collect into the variadic.
    for (name, val) in named {
        match callee.param_names[..fixed].iter().position(|pn| pn[..] == name[..]) {
            Some(j) if !matches!(frame.slots[j], Zval::Undef) => {
                return Err(PhpError::Error(format!(
                    "Named parameter ${} overwrites previous argument",
                    String::from_utf8_lossy(&name)
                )))
            }
            Some(j) => frame.slots[j] = val,
            None if has_variadic => {
                variadic.insert(Key::Str(PhpStr::new(name.to_vec())), val);
            }
            None => {
                return Err(PhpError::Error(format!(
                    "Unknown named parameter ${}",
                    String::from_utf8_lossy(&name)
                )))
            }
        }
    }
    if let Some(vs) = callee.variadic_slot {
        frame.slots[vs as usize] = Zval::Array(Rc::new(variadic));
    }
    // Every required (default-less, non-variadic) parameter must be bound.
    for (i, &required) in callee.param_required.iter().enumerate() {
        if required && matches!(frame.slots[i], Zval::Undef) {
            let required_count = callee.param_required.iter().filter(|&&r| r).count();
            let exactly = callee.param_required.iter().all(|&r| r);
            let qualifier = if exactly { "exactly" } else { "at least" };
            return Err(PhpError::ArgumentCountError(format!(
                "Too few arguments to function {display_name}(), {passed} passed in {} on line {line} and {qualifier} {required_count} expected",
                String::from_utf8_lossy(file)
            )));
        }
    }
    frame.argc = passed as u32;
    Ok(frame)
}

impl<'m> Vm<'m> {
    /// Dispatch a dynamic call on a runtime callee value (CLO + B1): an anonymous
    /// closure runs its body (binding captures then args); a named closure / FCC or
    /// a plain string names a user function / builtin; a `"Class::method"` string or
    /// a `[target, method]` array is a static / instance method callable; an object
    /// is callable via `__invoke`; a reference is followed. Anything else is an
    /// uncatchable "not callable" error. A pushed frame runs via the main loop; a
    /// builtin result is pushed on the current frame's stack.
    pub(super) fn invoke_value(&mut self, callee: Zval, args: Vec<Zval>) -> Result<(), PhpError> {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                None => self.push_closure_frame(&cl, args),
                Some(name) => {
                    let name = name.as_bytes().to_vec();
                    self.invoke_named(&name, args)
                }
            },
            Zval::Str(ref s) => {
                let bytes = s.as_bytes();
                // `"Class::method"` is a static method callable.
                if let Some(pos) = bytes.windows(2).position(|w| w == b"::") {
                    let cls = Zval::Str(PhpStr::new(bytes[..pos].to_vec()));
                    let method = bytes[pos + 2..].to_vec();
                    let cid = self.resolve_dynamic_class(&cls)?;
                    let top = self.frames.len() - 1;
                    self.dispatch_static_call(top, cid, &method, false, args)
                } else {
                    let name = bytes.to_vec();
                    self.invoke_named(&name, args)
                }
            }
            Zval::Array(ref a) => self.invoke_array_callable(a, args),
            Zval::Object(ref o) => {
                // An object is callable iff its class defines `__invoke` (D-22.7).
                let cid = o.borrow().class_id as usize;
                if resolve_method_runtime(&self.classes, cid, b"__invoke").is_some() {
                    let top = self.frames.len() - 1;
                    self.dispatch_instance_call(top, cid, callee.clone(), b"__invoke", args)
                } else {
                    Err(PhpError::Error(format!(
                        "Object of type {} is not callable",
                        String::from_utf8_lossy(&self.classes[cid].name)
                    )))
                }
            }
            Zval::Ref(rc) => {
                let inner = rc.borrow().clone();
                self.invoke_value(inner, args)
            }
            other => Err(PhpError::Error(format!(
                "Value of type {} is not callable",
                other.type_name_for_error()
            ))),
        }
    }

    /// A `[target, method]` array callable: `target` is an object (instance call)
    /// or a class-name string (static call); `method` is a string. A malformed
    /// array is an uncatchable "not callable" error.
    pub(super) fn invoke_array_callable(&mut self, arr: &PhpArray, args: Vec<Zval>) -> Result<(), PhpError> {
        let not_callable =
            || PhpError::Error("Value of type array is not callable".to_string());
        let elems: Vec<Zval> = arr.iter().map(|(_, v)| v.deref_clone()).collect();
        if elems.len() != 2 {
            return Err(not_callable());
        }
        let method = match &elems[1] {
            Zval::Str(s) => s.as_bytes().to_vec(),
            _ => return Err(not_callable()),
        };
        let top = self.frames.len() - 1;
        match &elems[0] {
            Zval::Object(_) => {
                let cid = object_class_id(&elems[0]).expect("object class id");
                self.dispatch_instance_call(top, cid, elems[0].clone(), &method, args)
            }
            Zval::Str(_) => {
                let cid = self.resolve_dynamic_class(&elems[0])?;
                self.dispatch_static_call(top, cid, &method, false, args)
            }
            _ => Err(not_callable()),
        }
    }

    /// Invoke a user callable (`$callable($args)`) from inside a host builtin and
    /// run it to completion, returning its value (B1). Resolves the callable via
    /// [`Self::invoke_value`], then — when that pushed a frame — drives a *nested*
    /// bounded [`Self::run_loop`] from the new baseline (mirrors `drive_fiber`):
    /// an exception caught inside the callable resumes there, an uncaught one
    /// unwinds out (its frames dropped) so it propagates through the host builtin.
    /// A value-builtin / generator-function callable pushes no frame — its result
    /// (or `Generator` handle) is taken straight off the caller's stack.
    pub(super) fn call_callable(&mut self, callee: Zval, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let baseline = self.frames.len();
        self.invoke_value(callee, args)?;
        if self.frames.len() == baseline {
            // No frame pushed: a value builtin (or a generator-function callable,
            // whose `Generator` handle is the call's value) left its result on the
            // calling frame's operand stack.
            return Ok(self.frames[baseline - 1]
                .stack
                .pop()
                .expect("host callable result on the caller stack"));
        }
        self.drive_to_return(baseline)
    }

    /// Normalise the `$newThis` argument of `bindTo`/`bind`/`call`: an object
    /// binds, `null` (or anything else) clears the binding (step 19-6).
    fn closure_this_arg(v: Option<Zval>) -> Option<Zval> {
        match v.map(|v| v.deref_clone()) {
            Some(o @ Zval::Object(_)) => Some(o),
            _ => None,
        }
    }

    /// Build a copy of `cl` with a new bound `$this`, optional new class scope,
    /// and a fresh object id (step 19-6, mirrors `eval::rebind_closure`).
    /// `new_scope` of `None` keeps the closure's current scope; `Some(s)` sets it
    /// (where `s` of `None` means *unscoped*). The defining module id is preserved.
    fn rebind_closure(
        &mut self,
        cl: &Closure,
        bound_this: Option<Zval>,
        new_scope: Option<Option<usize>>,
    ) -> Zval {
        let id = self.next_id();
        Zval::Closure(Rc::new(Closure {
            fn_idx: cl.fn_idx,
            captures: cl.captures.clone(),
            named: cl.named.clone(),
            bound_this,
            id,
            info: Rc::clone(&cl.info),
            module_id: cl.module_id,
            scope: new_scope.unwrap_or(cl.scope),
        }))
    }

    /// Resolve the `$newScope` argument of `Closure::bind`/`bindTo`. Returns
    /// `None` to *keep* the closure's current scope (argument omitted, or the
    /// sentinel string `"static"`); `Some(None)` for an explicit unscoped binding;
    /// and `Some(Some(class_id))` for an object (its class) or a class-name string.
    fn resolve_scope_arg(&self, v: Option<Zval>) -> Option<Option<usize>> {
        match v.map(|v| v.deref_clone()) {
            None => None,
            Some(Zval::Str(s)) if s.as_bytes().eq_ignore_ascii_case(b"static") => None,
            Some(Zval::Null) => Some(None),
            Some(Zval::Object(o)) => Some(Some(o.borrow().class_id as usize)),
            Some(Zval::Str(s)) => {
                let b = s.as_bytes();
                let lc = b.strip_prefix(b"\\").unwrap_or(b).to_ascii_lowercase();
                Some(self.class_index.get(&lc[..]).copied())
            }
            Some(_) => None,
        }
    }

    /// Built-in methods on a closure value: `$c->bindTo($newThis)` (rebind) and
    /// `$c->call($newThis, ...$args)` (rebind then invoke). Mirrors
    /// `eval::closure_method`.
    pub(super) fn closure_instance_method(
        &mut self,
        cl: &Closure,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bindTo") {
            let mut it = args.into_iter();
            let new_this = Self::closure_this_arg(it.next());
            let scope = self.resolve_scope_arg(it.next());
            Ok(self.rebind_closure(cl, new_this, scope))
        } else if method.eq_ignore_ascii_case(b"call") {
            // `$c->call($newThis, ...$args)` rebinds `$this` *and* the scope to the
            // class of `$newThis`, then invokes (step 19-6).
            let mut it = args.into_iter();
            let new_this = Self::closure_this_arg(it.next());
            let scope = Some(new_this.as_ref().and_then(object_class_id));
            let rest: Vec<Zval> = it.collect();
            let bound = self.rebind_closure(cl, new_this, scope);
            self.call_callable(bound, rest)
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Closure::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }

    /// Static methods on the `Closure` class: `Closure::bind($c, $newThis)` and
    /// `Closure::fromCallable($callable)`. Mirrors `eval::closure_static`.
    pub(super) fn closure_static_method(
        &mut self,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bind") {
            let mut it = args.into_iter();
            let target = it.next().map(|v| v.deref_clone());
            let new_this = Self::closure_this_arg(it.next());
            let scope = self.resolve_scope_arg(it.next());
            match target {
                Some(Zval::Closure(cl)) => Ok(self.rebind_closure(&cl, new_this, scope)),
                _ => Err(PhpError::Error(
                    "Closure::bind(): Argument #1 ($closure) must be of type Closure".to_string(),
                )),
            }
        } else if method.eq_ignore_ascii_case(b"fromCallable") {
            match args.into_iter().next().map(|v| v.deref_clone()) {
                // An existing closure passes through unchanged.
                Some(Zval::Closure(cl)) => Ok(Zval::Closure(cl)),
                // A function-name string becomes a named (first-class-callable)
                // closure, like `Op::MakeFcc`.
                Some(Zval::Str(s)) => {
                    let id = self.next_id();
                    let params = self
                        .module
                        .functions
                        .iter()
                        .find(|f| super::name_eq_ignore_case(&f.name, s.as_bytes()))
                        .map(super::closure_params)
                        .unwrap_or_default();
                    let info = Rc::new(ClosureInfo {
                        kind: ClosureRender::Function(s.clone()),
                        params,
                    });
                    Ok(Zval::Closure(Rc::new(Closure {
                        fn_idx: 0,
                        captures: Vec::new(),
                        named: Some(s),
                        bound_this: None,
                        id,
                        info,
                        module_id: 0,
                        scope: None,
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

    /// Drive a *nested* bounded [`Self::run_loop`] from `baseline` (the frame count
    /// before a callee frame was pushed) until that callee returns, propagating an
    /// uncaught exception out with its frames dropped. Shared by [`Self::call_callable`]
    /// (B1) and [`Self::vm_stringify`] (D2) — both run a freshly-pushed frame to its
    /// `Ret` from inside a host builtin.
    pub(super) fn drive_to_return(&mut self, baseline: usize) -> Result<Zval, PhpError> {
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
            Ok(RunExit::Returned(v)) => Ok(v),
            Ok(_) => unreachable!("a synchronously driven callee does not yield/suspend at its own baseline"),
            Err(e) => {
                self.frames.truncate(baseline);
                Err(e)
            }
        }
    }

    /// Dispatch a call to a function *name* (a string callable / first-class
    /// callable / named closure): a user function (case-insensitive, shadows
    /// builtins) installs a frame; a value builtin runs and pushes its result.
    pub(super) fn invoke_named(&mut self, name: &[u8], args: Vec<Zval>) -> Result<(), PhpError> {
        if let Some(idx) =
            self.module.functions.iter().position(|f| name_eq_ignore_case(&f.name, name))
        {
            let callee = &self.module.functions[idx];
            let mut frame = Frame::new(callee, self.module);
            bind_params(&mut frame, args);
            self.enter_callee(frame)?;
            return Ok(());
        }
        // A function declared by a linked eval/include unit (step 57): resolve it
        // and run its frame in the module that defined it.
        if let Some(&(fmod, idx)) = self.linked_functions.get(&name.to_ascii_lowercase()) {
            let callee = &fmod.functions[idx];
            let mut frame = Frame::new(callee, fmod);
            bind_params(&mut frame, args);
            self.enter_callee(frame)?;
            return Ok(());
        }
        match self.registry.get(name) {
            Some(Builtin::Value(f)) => {
                let f = *f;
                let line = self.cur_line(self.frames.len() - 1);
                let result = self.run_value_builtin(f, &args, line)?;
                let top = self.frames.len() - 1;
                self.frames[top].stack.push(result);
                Ok(())
            }
            Some(Builtin::RefFirst(_)) => Err(PhpError::Error(format!(
                "VM: dynamic call to by-reference builtin {}() is out of slice",
                String::from_utf8_lossy(name)
            ))),
            None => Err(PhpError::Error(format!(
                "Call to undefined function {}()",
                String::from_utf8_lossy(name)
            ))),
        }
    }

    /// Enter a freshly-built callee `frame`: if its body is a generator,
    /// materialise a `Generator` handle on the caller's operand stack instead of
    /// running it (GEN); otherwise push it to run. The caller is the current top
    /// frame, so this is called *before* `frame` is pushed.
    pub(super) fn enter_callee(&mut self, frame: Frame<'m>) -> Result<(), PhpError> {
        // The call site is the caller's current line, reported in an arg TypeError
        // (captured before the callee frame is pushed).
        let call_line = self.cur_line(self.frames.len() - 1);
        // Push the callee frame, then coerce/check each by-value argument against
        // its declared hint *within* that frame (step 14 / 16): PHP throws an
        // argument TypeError inside the function, so its stack trace shows this call
        // and "thrown in" reports the definition line. By-reference and variadic
        // slots are left untouched; an omitted (`Undef`) optional argument is
        // coerced later, when the default prologue fills it.
        self.frames.push(frame);
        let top = self.frames.len() - 1;
        let func = self.frames[top].func;
        if func.param_hints.iter().any(Option::is_some) {
            let strict = self.module.strict;
            for i in 0..func.n_params as usize {
                if Some(i as Slot) == func.variadic_slot {
                    continue;
                }
                if func.param_by_ref.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let Some(hint) = func.param_hints.get(i).cloned().flatten() else {
                    continue;
                };
                if matches!(self.frames[top].slots[i], Zval::Undef) {
                    continue;
                }
                let val = self.frames[top].slots[i].clone();
                match self.coerce_or_check_hint(val, &hint, strict) {
                    // The frame stays pushed on the error path so the unwind's trace
                    // capture includes this call; unwind then pops it.
                    Ok(c) => self.frames[top].slots[i] = c,
                    Err(given) => return Err(self.arg_type_error(func, i, &hint, &given, call_line)),
                }
            }
        }
        // A generator function materialises a `Generator` handle instead of running:
        // pop the (checked) frame back off and hand it to `make_generator`.
        if func.is_generator {
            let mut frame = self.frames.pop().expect("just-pushed generator frame");
            // Honour a return cell (e.g. an `IteratorAggregate::getIterator()` that
            // is itself a generator): the handle goes to the cell, not the stack.
            let ret_cell = frame.ret_cell.take();
            let gen = self.make_generator(frame);
            match ret_cell {
                Some(cell) => *cell.borrow_mut() = gen,
                None => {
                    let caller = self.frames.len() - 1;
                    self.frames[caller].stack.push(gen);
                }
            }
        }
        Ok(())
    }

    /// The catchable `TypeError` PHP raises for a return value that failed scalar
    /// coercion (step 14). The wording differs from the argument one: no call site,
    /// suffix `returned in <file>:<defline>`.
    pub(super) fn return_type_error(&self, f: &Func, hint: &TypeHint, given: &str) -> PhpError {
        PhpError::TypeError(format!(
            "{}(): Return value must be of type {}, {} returned in {}:{}",
            String::from_utf8_lossy(&f.name),
            hint.display_name(),
            given,
            String::from_utf8_lossy(&self.module.file),
            f.line,
        ))
    }

    /// The catchable `TypeError` PHP raises for an argument that failed scalar
    /// coercion, matching PHP's exact wording (step 14): the call site's file/line
    /// and the callee's definition file/line.
    fn arg_type_error(
        &self,
        f: &Func,
        i: usize,
        hint: &TypeHint,
        given: &str,
        call_line: Line,
    ) -> PhpError {
        let file = String::from_utf8_lossy(&self.module.file);
        let pname = f
            .param_names
            .get(i)
            .map(|n| String::from_utf8_lossy(n).into_owned())
            .unwrap_or_default();
        PhpError::TypeError(format!(
            "{}(): Argument #{} (${}) must be of type {}, {} given, \
             called in {} on line {} and defined in {}:{}",
            String::from_utf8_lossy(&f.name),
            i + 1,
            pname,
            hint.display_name(),
            given,
            file,
            call_line,
            file,
            f.line,
        ))
    }
}
