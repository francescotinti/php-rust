//! Expression evaluation: the `eval`/`eval_inner` dispatch (the core of the
//! tree-walker), instanceof checks and binary-operator application. Split out of
//! `eval.rs` (step 60); behaviour is unchanged.
use std::rc::Rc;

use php_types::{
    convert, ops, Closure, GenKey, PhpArray,
    PhpError, PhpStr, Zval,
};

use crate::builtin::Builtin;
use crate::hir::{
    BinOp, CastKind, ClassDecl, ClassId, ClassRef, Expr, ExprKind, FnDecl, StaticAssignOp, UnOp,
};

use super::*;

impl<'p> Evaluator<'p> {
    // --- expressions ---

    /// Evaluate `e`, stamping its line for any diagnostics it raises and flushing
    /// them into `rendered` before the value flows to its consumer. On success the
    /// enclosing line is restored; on the error path it is kept at the throwing
    /// node so the top-level fatal renderer reports the right location.
    pub(super) fn eval(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        let prev = self.cur_line;
        self.cur_line = e.line;
        let r = self.eval_inner(e);
        self.flush_diags();
        if r.is_ok() {
            self.cur_line = prev;
        }
        r
    }

    fn eval_inner(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        match &e.kind {
            ExprKind::Null => Ok(Zval::Null),
            ExprKind::Bool(b) => Ok(Zval::Bool(*b)),
            ExprKind::Int(i) => Ok(Zval::Long(*i)),
            ExprKind::Float(f) => Ok(Zval::Double(*f)),
            ExprKind::Str(bytes) => Ok(Zval::Str(PhpStr::new(bytes.clone()))),
            // A bare `NAME` that was not an engine constant: read it from the
            // `define()` table, else PHP 8's fatal `Error` (step 49c).
            ExprKind::Const(name) => match self.constants.get(name.as_ref()) {
                Some(v) => Ok(v.clone()),
                None => Err(PhpError::Error(format!(
                    "Undefined constant \"{}\"",
                    String::from_utf8_lossy(name)
                ))),
            },

            ExprKind::Var(slot) => Ok(self.read_var(*slot)),
            ExprKind::GlobalVar(slot) => Ok(self.read_global_var(*slot)),

            ExprKind::Binary(op, l, r) => {
                let a = self.eval(l)?;
                let b = self.eval(r)?;
                self.apply_binop(*op, a, b)
            }

            // Short-circuit logical operators always yield a clean bool.
            ExprKind::And(l, r) => {
                if !self.eval_bool(l)? {
                    Ok(Zval::Bool(false))
                } else {
                    Ok(Zval::Bool(self.eval_bool(r)?))
                }
            }
            ExprKind::Or(l, r) => {
                if self.eval_bool(l)? {
                    Ok(Zval::Bool(true))
                } else {
                    Ok(Zval::Bool(self.eval_bool(r)?))
                }
            }
            ExprKind::Xor(l, r) => {
                let a = self.eval_bool(l)?;
                let b = self.eval_bool(r)?;
                Ok(Zval::Bool(a ^ b))
            }

            ExprKind::Coalesce(l, r) => {
                let lv = self.eval_isset(l)?;
                if matches!(lv, Zval::Null | Zval::Undef) {
                    self.eval(r)
                } else {
                    Ok(lv)
                }
            }

            ExprKind::Unary(op, operand) => {
                let v = self.eval(operand)?;
                match op {
                    UnOp::Neg => ops::neg(&v, &mut self.diags),
                    // Unary `+` is the numeric coercion `1 * v` (same TypeError surface).
                    UnOp::Plus => ops::mul(&Zval::Long(1), &v, &mut self.diags),
                    UnOp::Not => Ok(Zval::Bool(!convert::to_bool(&v, &mut self.diags))),
                    UnOp::BitNot => ops::bw_not(&v, &mut self.diags),
                }
            }

            ExprKind::Cast(kind, operand) => {
                let v = self.eval(operand)?;
                Ok(match kind {
                    CastKind::Int => Zval::Long(convert::to_long_cast(&v, &mut self.diags)),
                    CastKind::Float => Zval::Double(convert::to_double(&v)),
                    // `(string)$obj` honours `__toString` (step 19-6); other values
                    // use the cast funnel (which warns on NaN).
                    CastKind::String if matches!(v, Zval::Object(_)) => {
                        Zval::Str(self.stringify(&v)?)
                    }
                    CastKind::String => Zval::Str(convert::to_zstr_cast(&v, &mut self.diags)),
                    CastKind::Bool => Zval::Bool(convert::to_bool(&v, &mut self.diags)),
                    CastKind::Array => array_cast(v),
                    CastKind::Object => self.object_cast(v),
                })
            }

            ExprKind::Assign(slot, rhs) => {
                let v = self.eval(rhs)?;
                self.slot_set(*slot as usize, v.clone());
                Ok(v)
            }

            ExprKind::AssignRef { target, source } => self.assign_ref(target, source),
            ExprKind::AssignRefCall { target, call } => self.assign_ref_call(target, call),

            ExprKind::AssignOp(op, slot, rhs) => {
                let cur = self.read_var(*slot);
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.slot_set(*slot as usize, res.clone());
                Ok(res)
            }

            ExprKind::AssignCoalesce(slot, rhs) => {
                let cur = self.slot_clone(*slot as usize);
                if matches!(cur, Zval::Null | Zval::Undef) {
                    let v = self.eval(rhs)?;
                    self.slot_set(*slot as usize, v.clone());
                    Ok(v)
                } else {
                    Ok(cur)
                }
            }

            ExprKind::IncDec { slot, inc, pre } => {
                let idx = *slot as usize;
                if matches!(self.slot_clone(idx), Zval::Undef) {
                    self.warn_undef(*slot);
                    self.slot_set(idx, Zval::Null);
                }
                let old = self.slot_clone(idx);
                // `increment`/`decrement` follow a `Zval::Ref` themselves, so the
                // mutation writes through any alias without a separate match here.
                let d = &mut self.diags;
                let target = &mut frame_mut!(self)[idx];
                if *inc {
                    ops::increment(target, d)?;
                } else {
                    ops::decrement(target, d)?;
                }
                Ok(if *pre { self.slot_clone(idx) } else { old })
            }

            ExprKind::IncDecPlace { place, inc, pre } => {
                let steps = self.resolve_steps(place)?;
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let mut val = match self.read_place_value(place.base, &steps)? {
                    Zval::Undef => Zval::Null,
                    v => v,
                };
                let old = val.clone();
                if *inc {
                    ops::increment(&mut val, &mut self.diags)?;
                } else {
                    ops::decrement(&mut val, &mut self.diags)?;
                }
                self.write_place(place.base, &steps, val.clone())?;
                Ok(if *pre { val } else { old })
            }

            ExprKind::Ternary {
                cond,
                then,
                otherwise,
            } => {
                let c = self.eval(cond)?;
                if convert::to_bool(&c, &mut self.diags) {
                    match then {
                        // Full ternary: evaluate the "then" branch.
                        Some(t) => self.eval(t),
                        // Short ternary `?:` returns the (truthy) condition itself.
                        None => Ok(c),
                    }
                } else {
                    self.eval(otherwise)
                }
            }

            ExprKind::Call { name, args, named } => {
                // A user-defined function shadows the builtin namespace (PHP
                // resolves both from one function table; you cannot redefine a
                // builtin, but a user function wins when present). User functions
                // bind by-reference parameters (step 11b), so their arguments are
                // resolved against the declaration rather than blindly evaluated.
                if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
                    let (argv, spread_named) = self.eval_call_args(idx, args)?;
                    // Named arguments — explicit (step 38) and from unpacking string
                    // keys (step 40) — are placed by parameter name after the
                    // positional ones.
                    let f: &'p FnDecl = &self.funcs[idx];
                    let argv = self.apply_named_args(f, argv, spread_named, named)?;
                    let result = self.call_user_fn(idx, argv)?;
                    // A by-reference function returns a `Zval::Ref`; in this
                    // (value) context it must be copied, not aliased — only
                    // `$y = &f()` keeps the cell (D-13.6).
                    return Ok(match result {
                        Zval::Ref(cell) => cell.borrow().clone(),
                        other => other,
                    });
                }
                // Named arguments to a builtin are a scope-out (step 38, D-38.2):
                // the registry carries no parameter-name metadata. User functions
                // are handled above.
                if !named.is_empty() {
                    return Err(PhpError::Error(format!(
                        "named arguments to builtin {}() are not supported",
                        String::from_utf8_lossy(name)
                    )));
                }
                // Higher-order builtins need to invoke a callback, so they are
                // run by the evaluator itself rather than the (pure) registry
                // (step 18, D-18.6). They take precedence over the registry.
                if let Some(res) = self.dispatch_higher_order(name, args) {
                    return res;
                }
                // Class-introspection builtins read the current object / class
                // table, so the evaluator answers them directly (step 20 coda).
                if let Some(res) = self.dispatch_class_introspection(name, args) {
                    return res;
                }
                // define()/constant()/defined() read the evaluator's runtime
                // constant table, so they are answered here (step 49c). Evaluate
                // arguments by value only once we know the name matches.
                if name.eq_ignore_ascii_case(b"define")
                    || name.eq_ignore_ascii_case(b"defined")
                    || name.eq_ignore_ascii_case(b"constant")
                {
                    let mut argv = Vec::with_capacity(args.len());
                    for a in args {
                        argv.push(self.eval(a)?);
                    }
                    if let Some(res) = self.call_constant_builtin(name, &argv) {
                        return res;
                    }
                }
                // A by-reference builtin (array_push/sort/...) binds its first
                // argument's variable cell rather than a copy, so it is handled
                // before the arguments are evaluated by value (step 11c).
                if let Some(Builtin::RefFirst(f)) = self.reg.get(name.as_ref()).copied() {
                    return self.call_ref_builtin(f, name, args);
                }
                // Builtins otherwise take all arguments by value. Copy the fn
                // pointer out so the registry borrow ends before we borrow
                // `out`/`diags` mutably for the call context.
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                let f = match self.reg.get(name.as_ref()) {
                    Some(Builtin::Value(f)) => *f,
                    Some(Builtin::RefFirst(_)) => unreachable!("handled above"),
                    None => {
                        return Err(PhpError::Error(format!(
                            "Call to undefined function {}()",
                            String::from_utf8_lossy(name)
                        )))
                    }
                };
                self.dispatch_value_builtin(f, &argv)
            }

            // A closure / arrow expression: snapshot its captures in the active
            // frame and build the `Zval::Closure` value (step 18, D-18.2/D-18.3).
            ExprKind::Closure {
                fn_idx,
                captures,
                bind_this,
            } => {
                let bound = self.bind_captures(captures);
                // A non-static closure captures the current `$this` (step 19-6).
                let bound_this = if *bind_this { self.cur_this.clone() } else { None };
                Ok(Zval::Closure(Rc::new(Closure {
                    fn_idx: *fn_idx,
                    captures: bound,
                    named: None,
                    bound_this,
                    id: self.next_id(),
                    info: Rc::clone(&self.closure_info[*fn_idx]),
                })))
            }

            // A first-class callable `name(...)` — a closure wrapping a name.
            ExprKind::FirstClassCallable(name) => {
                let info = self.first_class_info(name);
                Ok(Zval::Closure(Rc::new(Closure {
                    fn_idx: 0,
                    captures: Vec::new(),
                    named: Some(PhpStr::new(name.to_vec())),
                    bound_this: None,
                    id: self.next_id(),
                    info,
                })))
            }

            // A dynamic call `$f(...)` dispatched on the callee value (step 18,
            // D-18.5). Arguments are evaluated by value (left to right).
            ExprKind::CallDynamic { callee, args } => {
                let c = self.eval(callee)?.deref_clone();
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                self.call_value(c, argv)
            }

            // A spread `...$e` is only meaningful as a call argument, where the
            // dedicated argument-evaluation paths intercept it (step 40). Reaching
            // the generic evaluator means it appeared elsewhere.
            ExprKind::Spread(_) => Err(PhpError::Error(
                "Cannot use spread operator outside of function call".to_string(),
            )),

            ExprKind::Array(elems) => {
                let mut arr = PhpArray::new();
                for el in elems {
                    // PHP evaluates the key before the value.
                    match &el.key {
                        Some(ke) => {
                            let kv = self.eval(ke)?;
                            let key = self.coerce_key(&kv)?;
                            let v = self.eval(&el.value)?;
                            arr.insert(key, v);
                        }
                        None => {
                            let v = self.eval(&el.value)?;
                            arr.append(v).map_err(|_| array_occupied())?;
                        }
                    }
                }
                Ok(Zval::Array(Rc::new(arr)))
            }

            ExprKind::Index { base, index } => {
                let b = self.eval(base)?;
                let k = self.eval(index)?;
                self.read_index(&b, &k, false)
            }

            ExprKind::AssignPlace(place, rhs) => {
                // PHP evaluates the lvalue's offset expressions *before* the RHS
                // (so `$a[f()][g()] = h()` runs f, g, then h). Resolve the place
                // steps first to match — and stay consistent with AssignOpPlace.
                let steps = self.resolve_steps(place)?;
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let value = self.eval(rhs)?;
                self.write_place(place.base, &steps, value.clone())?;
                Ok(value)
            }

            ExprKind::AssignOpPlace(op, place, rhs) => {
                let steps = self.resolve_steps(place)?;
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let cur = self.read_place_value(place.base, &steps)?;
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.write_place(place.base, &steps, res.clone())?;
                Ok(res)
            }

            ExprKind::AssignCoalescePlace(place, rhs) => {
                let steps = self.resolve_steps(place)?;
                // Magic property: `__isset` decides, then `__get` (existing) or
                // `__set` (new), step 22, D-22.6.
                if let [Step::Prop(name)] = steps.as_slice() {
                    if let Zval::Object(o) = self.base_clone(place.base) {
                        if let Some(r) = self.magic_isset_bool(&o, name) {
                            return if r? {
                                self.prop_value_silent(&o, name)
                            } else {
                                let value = self.eval(rhs)?;
                                self.write_place(place.base, &steps, value.clone())?;
                                Ok(value)
                            };
                        }
                    }
                }
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                match self.silent_get(place.base, &steps) {
                    Some(v) if !matches!(v, Zval::Null | Zval::Undef) => Ok(v),
                    _ => {
                        let value = self.eval(rhs)?;
                        self.write_place(place.base, &steps, value.clone())?;
                        Ok(value)
                    }
                }
            }

            ExprKind::Isset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    if !self.place_isset(p.base, &steps)? {
                        return Ok(Zval::Bool(false));
                    }
                }
                Ok(Zval::Bool(true))
            }

            ExprKind::Empty(place) => {
                let steps = self.resolve_steps(place)?;
                let empty = self.place_empty(place.base, &steps)?;
                Ok(Zval::Bool(empty))
            }

            // `@expr` (step 48): evaluate `expr`, then drop any non-fatal
            // diagnostics it raised — error_reporting is silenced for the
            // duration. A thrown exception / engine `Error` is on the `Err`
            // channel and is NOT suppressed (PHP only silences warnings/notices).
            // Edge: a diagnostic already *rendered* mid-evaluation (the operand
            // emitted output) can't be unrendered — a minor scope-out (D-48.1).
            ExprKind::Suppress(e) => {
                let saved = self.diags.len();
                self.suppress_depth += 1;
                let r = self.eval(e);
                self.suppress_depth -= 1;
                // Drop the diagnostics raised while suppressed (they were never
                // rendered, since `flush_diags` was a no-op under `@`).
                self.diags.truncate(saved);
                r
            }

            // `print expr` (step 46): emit the stringified value (honouring
            // `__toString`, like echo), then evaluate to int(1).
            ExprKind::Print(e) => {
                let z = self.eval(e)?;
                let s = self.stringify(&z)?;
                self.emit(s.as_bytes());
                Ok(Zval::Long(1))
            }

            // `exit`/`die [arg]` (step 46): the argument follows PHP's
            // `string|int $status` union (see `exit_status`). Raised on the `Err`
            // channel so it is uncatchable and bypasses `finally`.
            ExprKind::Exit(arg) => {
                let code = match arg {
                    Some(e) => {
                        let v = self.eval(e)?;
                        self.exit_status(v)?
                    }
                    None => 0,
                };
                Err(PhpError::Exit(code))
            }

            ExprKind::Match { subject, arms } => {
                let subj = self.eval(subject)?;
                let mut default_body = None;
                for arm in arms {
                    if arm.conditions.is_empty() {
                        default_body = Some(&arm.body);
                        continue;
                    }
                    for c in &arm.conditions {
                        let cv = self.eval(c)?;
                        if ops::identical(&subj, &cv) {
                            return self.eval(&arm.body);
                        }
                    }
                }
                match default_body {
                    Some(b) => self.eval(b),
                    None => Err(PhpError::Error(format!(
                        "Unhandled match case {}",
                        match_case_repr(&subj)
                    ))),
                }
            }

            ExprKind::New { class, args, named } => self.eval_new(class, args, named),

            ExprKind::MethodCall {
                object,
                method,
                args,
                named,
                nullsafe,
            } => {
                let recv = self.eval(object)?.deref_clone();
                if *nullsafe && matches!(recv, Zval::Null | Zval::Undef) {
                    return Ok(Zval::Null);
                }
                let (argv, spread_named) = self.eval_value_args(args)?;
                // Methods on a closure value (`$c->bindTo(...)`) are built-in
                // (step 19-6), not user-class dispatch. Named args (and named
                // unpacking) there are out of scope (step 38 / 40).
                if let Zval::Closure(cl) = &recv {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.closure_method(cl, method, argv);
                }
                // Methods on a `Generator` value (`->current()`, `->next()`, …)
                // are built-in (step 39), dispatched ahead of user-class lookup.
                if let Zval::Generator(gs) = &recv {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.generator_method(Rc::clone(gs), method, argv);
                }
                self.call_method(recv, method, argv, spread_named, named)
            }

            ExprKind::PropGet {
                object,
                name,
                nullsafe,
            } => {
                let recv = self.eval(object)?.deref_clone();
                if *nullsafe && matches!(recv, Zval::Null | Zval::Undef) {
                    return Ok(Zval::Null);
                }
                self.read_property(&recv, name)
            }

            ExprKind::This => match &self.cur_this {
                Some(obj) => Ok(obj.clone()),
                None => Err(PhpError::Error(
                    "Using $this when not in object context".to_string(),
                )),
            },

            ExprKind::StaticCall {
                class,
                method,
                args,
                named,
            } => {
                let (argv, spread_named) = self.eval_value_args(args)?;
                self.call_static(class, method, argv, spread_named, named)
            }

            ExprKind::ClassConst { class, name } => self.eval_class_const(class, name),

            ExprKind::StaticProp { class, name } => {
                let cell = self.static_prop_cell(class, name)?;
                let v = cell.borrow().deref_clone();
                Ok(v)
            }

            ExprKind::StaticPropAssign {
                class,
                name,
                op,
                rhs,
            } => {
                let cell = self.static_prop_cell(class, name)?;
                match op {
                    StaticAssignOp::Plain => {
                        let v = self.eval(rhs)?;
                        *cell.borrow_mut() = v.clone();
                        Ok(v)
                    }
                    StaticAssignOp::Coalesce => {
                        let cur = cell.borrow().deref_clone();
                        if matches!(cur, Zval::Null | Zval::Undef) {
                            let v = self.eval(rhs)?;
                            *cell.borrow_mut() = v.clone();
                            Ok(v)
                        } else {
                            Ok(cur)
                        }
                    }
                    StaticAssignOp::Op(b) => {
                        let cur = cell.borrow().deref_clone();
                        let rv = self.eval(rhs)?;
                        let res = self.apply_binop(*b, cur, rv)?;
                        *cell.borrow_mut() = res.clone();
                        Ok(res)
                    }
                }
            }

            ExprKind::StaticPropIncDec {
                class,
                name,
                inc,
                pre,
            } => {
                let cell = self.static_prop_cell(class, name)?;
                let old = cell.borrow().deref_clone();
                {
                    let mut guard = cell.borrow_mut();
                    if *inc {
                        ops::increment(&mut guard, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut guard, &mut self.diags)?;
                    }
                }
                Ok(if *pre { cell.borrow().deref_clone() } else { old })
            }

            ExprKind::InstanceOf { expr, class } => {
                let v = self.eval(expr)?.deref_clone();
                let result = match &v {
                    Zval::Object(o) => match self.resolve_class_ref(class) {
                        Ok(target) => self.is_instance_of(o.borrow().class_id as usize, target),
                        // An unknown class on the RHS is simply not matched (PHP
                        // does not error here under the CLI without autoloading).
                        Err(_) => false,
                    },
                    // A `Generator` satisfies `Generator`, `Iterator`, and
                    // `Traversable` (the built-in interface chain), step 39-7.
                    Zval::Generator(_) => match class {
                        ClassRef::Named(name) => matches!(
                            name.to_ascii_lowercase().as_slice(),
                            b"generator" | b"iterator" | b"traversable"
                        ),
                        _ => false,
                    },
                    _ => false,
                };
                Ok(Zval::Bool(result))
            }

            // `throw <expr>` (step 20): evaluate the operand and unwind with
            // `PhpError::Thrown`, which propagates through every `?` until a
            // matching `catch` (or the top, where it renders as an uncaught fatal).
            ExprKind::Throw(e) => {
                let v = self.eval(e)?.deref_clone();
                Err(PhpError::Thrown(v))
            }

            // `yield [$k =>] [$v]` (step 39): suspend the running generator,
            // handing out the (key, value); the expression evaluates to the value
            // the next resume delivers (`send()` argument / NULL for `next()`).
            ExprKind::Yield { key, value } => {
                let value = match value {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                let key = match key {
                    Some(e) => GenKey::Keyed(self.eval(e)?),
                    None => GenKey::Auto,
                };
                self.gen_suspend(key, value)
            }

            // `yield from <iterator>` — delegated iteration (step 39-6).
            ExprKind::YieldFrom(e) => self.eval_yield_from(e),
        }
    }

    /// Whether an object of `class_id` is an instance of `target` (step 19-5,
    /// D-19.16): the class itself, any ancestor, or any implemented interface,
    /// transitively through interface inheritance.
    pub(super) fn is_instance_of(&self, class_id: ClassId, target: ClassId) -> bool {
        // `Stringable` is auto-implemented (step 24-1): any class with a
        // resolvable `__toString` satisfies it, even without an explicit
        // `implements Stringable`. PHP 8 adds this interface implicitly.
        if self.class_index.get(b"stringable".as_slice()) == Some(&target)
            && self.resolve_method(class_id, b"__toString").is_some()
        {
            return true;
        }
        let classes: &'p [ClassDecl] = self.classes;
        let mut cur = Some(class_id);
        while let Some(c) = cur {
            if c == target {
                return true;
            }
            if classes[c].interfaces.iter().any(|&i| self.iface_is_a(i, target)) {
                return true;
            }
            cur = classes[c].parent;
        }
        false
    }

    /// Whether `cid` is a `Throwable` (step 20): used to stamp `line`/`file` on
    /// exception instances at `new` time. The prelude guarantees `Throwable`
    /// exists, so the lookup is normally `Some`.
    pub(super) fn is_throwable(&self, cid: ClassId) -> bool {
        match self.class_index.get(b"throwable".as_slice()) {
            Some(&tid) => self.is_instance_of(cid, tid),
            None => false,
        }
    }

    /// Whether interface `i` is, or transitively extends, `target` (step 19-5).
    fn iface_is_a(&self, i: ClassId, target: ClassId) -> bool {
        if i == target {
            return true;
        }
        let classes: &'p [ClassDecl] = self.classes;
        classes[i].interfaces.iter().any(|&p| self.iface_is_a(p, target))
    }

    fn apply_binop(&mut self, op: BinOp, a: Zval, b: Zval) -> Result<Zval, PhpError> {
        // String concatenation honours `__toString` on object operands (step
        // 19-6); `ops::concat` (in `php_types`) cannot reach the evaluator.
        if matches!(op, BinOp::Concat) && (matches!(a, Zval::Object(_)) || matches!(b, Zval::Object(_)))
        {
            let mut out = self.stringify(&a)?.as_bytes().to_vec();
            out.extend_from_slice(self.stringify(&b)?.as_bytes());
            return Ok(Zval::Str(PhpStr::new(out)));
        }
        let d = &mut self.diags;
        match op {
            BinOp::Add => ops::add(&a, &b, d),
            BinOp::Sub => ops::sub(&a, &b, d),
            BinOp::Mul => ops::mul(&a, &b, d),
            BinOp::Div => ops::div(&a, &b, d),
            BinOp::Mod => ops::modulo(&a, &b, d),
            BinOp::Pow => ops::pow(&a, &b, d),
            BinOp::Concat => ops::concat(&a, &b, d),
            BinOp::BitAnd => ops::bw_and(&a, &b, d),
            BinOp::BitOr => ops::bw_or(&a, &b, d),
            BinOp::BitXor => ops::bw_xor(&a, &b, d),
            BinOp::Shl => ops::shl(&a, &b, d),
            BinOp::Shr => ops::shr(&a, &b, d),
            BinOp::Eq => Ok(Zval::Bool(ops::loose_eq(&a, &b))),
            BinOp::NotEq => Ok(Zval::Bool(!ops::loose_eq(&a, &b))),
            BinOp::Identical => Ok(Zval::Bool(ops::identical(&a, &b))),
            BinOp::NotIdentical => Ok(Zval::Bool(!ops::identical(&a, &b))),
            BinOp::Lt => Ok(Zval::Bool(ops::smaller(&a, &b))),
            BinOp::Le => Ok(Zval::Bool(ops::smaller_or_equal(&a, &b))),
            // `a > b` ⟺ `b < a`; `a >= b` ⟺ `b <= a`.
            BinOp::Gt => Ok(Zval::Bool(ops::smaller(&b, &a))),
            BinOp::Ge => Ok(Zval::Bool(ops::smaller_or_equal(&b, &a))),
            BinOp::Spaceship => Ok(Zval::Long(ops::compare(&a, &b) as i64)),
        }
    }
}
