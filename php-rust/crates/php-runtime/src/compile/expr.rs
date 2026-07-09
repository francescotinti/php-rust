//! Expression + call + place-read emission. Split from compile/mod.rs.
use super::*;

impl<'a> super::FnCompiler<'a> {
    pub(super) fn expr(&mut self, e: &Expr) -> R<()> {
        self.cur_line = e.line;
        match &e.kind {
            ExprKind::Null => {
                let k = self.konst(Const::Null);
                self.emit(Op::PushConst(k));
            }
            ExprKind::Bool(b) => {
                let k = self.konst(Const::Bool(*b));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Int(i) => {
                let k = self.konst(Const::Int(*i));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Float(f) => {
                let k = self.konst(Const::Float(*f));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Str(s) => {
                let k = self.konst(Const::Str(s.clone()));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Const { name, fallback } => {
                // A *user* constant (engine constants are folded at lowering): read
                // it from the VM's constant table at run time (B3). `fallback` is
                // the global name a namespaced unqualified constant falls back to.
                self.emit(Op::ConstFetch {
                    name: name.clone(),
                    fallback: fallback.clone(),
                });
            }
            ExprKind::Var(slot) => {
                // A source-level read warns on an undefined slot (PHP 8). The bare
                // name (no `$`) is taken from the slot table; a slot without a name
                // (shouldn't happen for a source var) degrades to a silent load.
                match self.slot_names.get(*slot as usize) {
                    Some(name) => {
                        let k = self.konst(Const::Str(name.clone()));
                        self.emit(Op::LoadVar { slot: *slot, name: k });
                    }
                    None => {
                        self.emit(Op::LoadSlot(*slot));
                    }
                }
            }
            ExprKind::GlobalVar(slot) => {
                // `$GLOBALS['x']` read — a resolved script-frame slot (step 12-3),
                // reachable from inside a function.
                self.emit(Op::LoadGlobal(*slot));
            }
            ExprKind::Superglobal(idx) => {
                // `$_SERVER` (&c.) read — resolved by name in the VM superglobal
                // store, so it reads correctly from any unit/frame.
                self.emit(Op::LoadSuperglobal(*idx));
            }
            ExprKind::GlobalsArray => {
                // Bare `$GLOBALS`: snapshot of the global symbol table.
                self.emit(Op::LoadGlobals);
            }
            ExprKind::GlobalsDynAssign { key, rhs } => {
                self.expr(key)?;
                self.expr(rhs)?;
                self.emit(Op::GlobalsDynAssign); // [key, v] -> [v]
            }
            ExprKind::Assign(slot, rhs) => {
                self.expr(rhs)?;
                self.emit(Op::Dup); // assignment is an expression valued by the RHS
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::ListAssign { temp, rhs, assigns } => {
                // `[$a,$b] = rhs`: stash rhs once, run each sub-assignment (which
                // reads `temp[key]`, or for a `&$x` target aliases the real source
                // element), then leave the stored rhs as the value. Element reads
                // are scalar-silent (see `in_list_assign`).
                self.expr(rhs)?;
                self.emit(Op::StoreSlot(*temp));
                let saved = std::mem::replace(&mut self.in_list_assign, true);
                for a in assigns {
                    self.expr(a)?;
                    self.emit(Op::Pop); // each sub-assignment's value is discarded
                }
                self.in_list_assign = saved;
                self.emit(Op::LoadSlot(*temp));
            }
            ExprKind::AssignOp(op, slot, rhs) => {
                self.emit(Op::LoadSlot(*slot));
                self.expr(rhs)?;
                self.emit(Op::Binary(*op));
                self.emit(Op::Dup);
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::IncDec { slot, inc, pre } => {
                self.emit(Op::IncDecSlot { slot: *slot, inc: *inc, pre: *pre });
            }
            ExprKind::Binary(op, a, b) => {
                // String concatenation stringifies each operand, honouring
                // `__toString` on object operands (OOP-3c).
                self.expr(a)?;
                if *op == BinOp::Concat {
                    self.emit(Op::Stringify);
                }
                self.expr(b)?;
                if *op == BinOp::Concat {
                    self.emit(Op::Stringify);
                }
                self.emit(Op::Binary(*op));
            }
            ExprKind::Unary(op, a) => {
                self.expr(a)?;
                self.emit(Op::Unary(*op));
            }
            ExprKind::Cast(kind, a) => {
                use crate::hir::CastKind;
                match kind {
                    // `(string)` honours `__toString` (OOP-3c). (The exotic
                    // `(string)NAN` coercion warning is not reproduced here.)
                    CastKind::String => {
                        self.expr(a)?;
                        self.emit(Op::Stringify);
                    }
                    CastKind::Int
                    | CastKind::Bool
                    | CastKind::Float
                    | CastKind::Array
                    | CastKind::Object => {
                        self.expr(a)?;
                        self.emit(Op::Cast(*kind));
                    }
                }
            }
            ExprKind::Suppress(e) => {
                // `@expr` (step 48): suppress diagnostics raised while evaluating
                // `expr`, leaving its value on the stack. Engine errors / thrown
                // objects still propagate (the unwind clears suppression).
                self.emit(Op::SuppressBegin);
                self.expr(e)?;
                self.emit(Op::SuppressEnd);
            }
            ExprKind::Exit(arg) => {
                // `exit` / `die` (step 46): evaluate the optional status, then
                // diverge via `Op::Exit` (raises `PhpError::Exit`, bypassing finally).
                match arg {
                    Some(e) => {
                        self.expr(e)?;
                        self.emit(Op::Exit { has_arg: true });
                    }
                    None => {
                        self.emit(Op::Exit { has_arg: false });
                    }
                }
            }
            ExprKind::And(a, b) => self.short_circuit(a, b, false)?,
            ExprKind::Or(a, b) => self.short_circuit(a, b, true)?,
            ExprKind::Xor(a, b) => {
                // `a xor b` evaluates both operands (no short-circuit) and yields a
                // bool: `(bool)a != (bool)b`. `!!x` coerces to bool, then loose
                // `!=` on the two bools is logical xor (eval: `Bool(a ^ b)`).
                self.expr(a)?;
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.expr(b)?;
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Unary(crate::hir::UnOp::Not));
                self.emit(Op::Binary(BinOp::NotEq));
            }
            ExprKind::Ternary { cond, then, otherwise } => {
                match then {
                    Some(then) => {
                        // cond ? then : otherwise
                        self.expr(cond)?;
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        self.expr(then)?;
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                    None => {
                        // cond ?: otherwise — evaluate cond once, reuse if truthy.
                        self.expr(cond)?;
                        self.emit(Op::Dup);
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.emit(Op::Pop); // discard the falsy cond copy
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                }
            }
            ExprKind::Print(a) => {
                self.expr(a)?;
                self.emit(Op::Stringify); // honour __toString (OOP-3c)
                self.emit(Op::Print);
            }
            ExprKind::Call { name, fallback, args, named } => {
                self.call(name, fallback.as_deref(), args, named)?
            }
            ExprKind::Array(elems) => {
                self.emit(Op::ArrayInit);
                for el in elems {
                    // `[...$src]` array spread (PHP 8.1): merge the source's elements
                    // (int keys renumbered, string keys preserved; Traversables too).
                    if let ExprKind::Spread(src) = &el.value.kind {
                        self.expr(src)?;
                        self.emit(Op::ArrayAppendSpread);
                        continue;
                    }
                    if let Some(k) = &el.key {
                        self.expr(k)?;
                    }
                    // A by-reference element (`['k' => &$v]`) pushes a *reference*
                    // to the source place instead of the value, so the array
                    // element aliases its cell; `Op::ArrayInsert` / `Op::ArrayPush`
                    // then store that `Ref` verbatim. A bare variable uses its
                    // slot; any other place (`&$o->p[$k]`) takes the `MakeRef`
                    // field-path route, exactly like a by-ref builtin argument.
                    match (el.by_ref, &el.value.kind) {
                        (true, ExprKind::Var(slot)) => {
                            self.emit(Op::PushRef(*slot));
                        }
                        (true, _) => {
                            let Some(place) = expr_field_place(&el.value) else {
                                return Err(CompileError::Unsupported(
                                    "by-reference array element of a non-place expression"
                                        .into(),
                                ));
                            };
                            let (base, steps) = self.field_path(&place)?;
                            self.emit(Op::MakeRef { base, steps: steps.into() });
                        }
                        _ => self.expr(&el.value)?,
                    }
                    if el.key.is_some() {
                        self.emit(Op::ArrayInsert);
                    } else {
                        self.emit(Op::ArrayPush);
                    }
                }
            }
            ExprKind::Index { base, index } => {
                // Part of an access chain: `$o?->m()['x']` skips the fetch too.
                let root = self.chain_enter();
                self.expr(base)?;
                self.chain_pause(|s| s.expr(index))?;
                self.emit(if self.in_list_assign { Op::FetchDimList } else { Op::FetchDim });
                self.chain_exit(root);
            }
            ExprKind::AssignPlace(place, rhs) => self.assign_place(place, rhs)?,
            ExprKind::AssignCoalescePlace(place, rhs) => self.assign_coalesce_place(place, rhs)?,
            ExprKind::AssignRef { target, source } => self.assign_ref(target, source)?,
            ExprKind::AssignRefCall { target, call } => self.assign_ref_call(target, call)?,
            ExprKind::Closure { fn_idx, captures, bind_this } => {
                // `closure_shift` is non-zero only for a trait body flattened from
                // another unit, whose closures were re-appended to this unit's table.
                let idx = (*fn_idx as i64 + self.closure_shift as i64) as u32;
                self.emit(Op::MakeClosure {
                    fn_idx: idx,
                    captures: captures.clone().into_boxed_slice(),
                    bind_this: *bind_this,
                });
            }
            ExprKind::FirstClassCallable(name) => {
                self.emit(Op::MakeFcc { name: name.clone() });
            }
            ExprKind::Throw(e) => {
                // PHP 8 `throw` is an expression that diverges; evaluate the
                // operand and raise. Any value the surrounding context expected is
                // never produced (the following ops are unreachable).
                self.expr(e)?;
                self.emit(Op::Throw);
            }
            ExprKind::CallDynamic { callee, args } => {
                // Push the callee, then the arguments; `CallValue` dispatches on the
                // callee at run time. With argument unpacking (`$f(...$a)`) the
                // arguments are built into a runtime array and expanded by
                // `CallValueArgs` (the value-callee analogue of `MethodCallDynamicArgs`).
                self.expr(callee)?;
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    self.build_args_array(args)?;
                    self.emit(Op::CallValueArgs);
                } else {
                    for a in args {
                        self.expr(a)?;
                    }
                    self.emit(Op::CallValue { argc: args.len() as u32 });
                }
            }
            ExprKind::Pipe { input, callable } => {
                // `input |> callable` == `callable(input)`, but the operands evaluate
                // left-to-right: push the input first, then the callable, then swap
                // so the stack is [callable, input] as `CallValue` expects.
                self.expr(input)?;
                self.expr(callable)?;
                self.emit(Op::Swap);
                self.emit(Op::CallValue { argc: 1 });
            }
            ExprKind::AssignOpPlace(op, place, rhs) => self.assign_op_place(*op, place, rhs)?,
            ExprKind::IncDecPlace { place, inc, pre } => self.incdec_place(place, *inc, *pre)?,
            ExprKind::Isset(places) => self.isset(places)?,
            ExprKind::Empty(place) => self.empty(place)?,
            ExprKind::Coalesce(a, b) => {
                // `$o->p ?? d`: a property is read isset-aware — `__isset` decides
                // and `__get` runs only when set, so an unset magic property never
                // warns or calls `__get`. Other operands (Var/Index) read silently.
                if let ExprKind::PropGet { object, name, nullsafe } = &a.kind {
                    if !nullsafe {
                        self.coalesce_load(object)?; // [obj] — silent: `$x->a->b ?? d`
                        self.emit(Op::Dup); // [obj, obj]
                        self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
                        let to_default = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → default; [obj]
                        self.emit(Op::PropGet { name: name.clone() }); // set → [value]
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let default_at = self.here();
                        self.patch(to_default, Op::JumpIfFalse(default_at));
                        self.emit(Op::Pop); // drop the kept object
                        self.expr(b)?; // [default]
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                        return Ok(());
                    }
                }
                // `$o->$n ?? d`: read the dynamic property *silently* (a missing or
                // null property both take the default, exactly as `??` requires) so
                // no "Undefined property" warning leaks.
                if let ExprKind::PropGetDyn { object, name, nullsafe } = &a.kind {
                    if !nullsafe {
                        self.coalesce_load(object)?; // [obj] — silent object read
                        self.expr(name)?; // [obj, name]
                        self.emit(Op::PropGetDynamicSilent); // [value-or-null]
                        let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                        self.expr(b)?; // [default]
                        let end = self.here();
                        self.patch(to_end, Op::JumpIfNotNull(end));
                        return Ok(());
                    }
                }
                // `$x[k] ?? d`: read the element isset-aware so a missing array key
                // OR an out-of-range/non-integer string offset takes the default
                // (a plain read of a string offset yields "", not null).
                if let ExprKind::Index { base, index } = &a.kind {
                    // The base is loaded isset-aware too: `$a[k] ?? d` must not
                    // warn when `$a` is an undefined variable / missing key /
                    // unset dynamic property — `??` suppresses those notices all
                    // the way down the access chain (bug: base used a plain read).
                    self.coalesce_load(base)?;
                    self.expr(index)?;
                    self.emit(Op::CoalesceFetchDim);
                    let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                    self.expr(b)?;
                    let end = self.here();
                    self.patch(to_end, Op::JumpIfNotNull(end));
                    return Ok(());
                }
                // Left read silently — `??` suppresses the undefined-variable
                // warning, so a plain `$x` uses the silent LoadSlot, not LoadVar.
                if let ExprKind::Var(slot) = &a.kind {
                    self.emit(Op::LoadSlot(*slot));
                } else {
                    self.expr(a)?;
                }
                let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                self.expr(b)?;
                let end = self.here();
                self.patch(to_end, Op::JumpIfNotNull(end));
            }
            ExprKind::AssignCoalesce(slot, rhs) => {
                self.emit(Op::LoadSlot(*slot));
                let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                self.expr(rhs)?;
                self.emit(Op::Dup); // the assignment yields the stored value
                self.emit(Op::StoreSlot(*slot));
                let end = self.here();
                self.patch(to_end, Op::JumpIfNotNull(end));
            }
            ExprKind::Match { subject, arms } => self.match_expr(subject, arms)?,
            ExprKind::New { class, args, named } => self.new_obj(class, args, named)?,
            ExprKind::This => {
                self.emit(Op::This);
            }
            ExprKind::Clone(e) => {
                self.expr(e)?;
                self.emit(Op::Clone);
            }
            ExprKind::Eval(e) => {
                self.expr(e)?;
                self.emit(Op::Eval);
            }
            ExprKind::Include { mode, path } => {
                self.expr(path)?;
                self.emit(Op::Include { mode: *mode });
            }
            ExprKind::PropGet { object, name, nullsafe } => {
                let root = self.chain_enter();
                self.expr(object)?;
                if *nullsafe {
                    // `$o?->p`: a null receiver keeps null and skips the read —
                    // and every further link of the enclosing chain (patched by
                    // the chain root).
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.nullsafe_chain.as_mut().expect("chain open").push(skip);
                }
                self.emit(Op::PropGet { name: name.clone() });
                self.chain_exit(root);
            }
            ExprKind::MethodCall { object, method, args, named, nullsafe } => {
                let root = self.chain_enter();
                self.expr(object)?;
                // A `$this->m(...)` receiver has a statically-known class, so the
                // method's by-reference parameters can be honoured (REF-2); any other
                // receiver is dynamic and stays by-value.
                let recv_class = match object.kind {
                    ExprKind::This => self.cur_class,
                    _ => None,
                };
                if *nullsafe {
                    // `$o?->m(...)`: a null receiver keeps null and skips the call
                    // (its arguments are not evaluated either) plus the rest of
                    // the chain.
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.nullsafe_chain.as_mut().expect("chain open").push(skip);
                }
                self.chain_pause(|s| s.emit_method_call(method, args, named, recv_class))?;
                // A method that returns by reference (`&m()`) yields a raw `Ref`;
                // in value context (everything except a `$x =& $o->m()` bind, which
                // compiles through `assign_ref_call` → `BindRefToChecked`) PHP hands
                // back a COPY, not an alias (REF-4b, mirrors the known-function
                // `DerefTop` above). `DerefTop` is a no-op for a non-reference
                // result, so it is safe to emit after every value-context call.
                self.emit(Op::DerefTop);
                self.chain_exit(root);
            }
            ExprKind::PropGetDyn { object, name, nullsafe } => {
                let root = self.chain_enter();
                self.expr(object)?;
                if *nullsafe {
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.nullsafe_chain.as_mut().expect("chain open").push(skip);
                }
                self.chain_pause(|s| s.expr(name))?; // [obj, name]
                self.emit(Op::PropGetDynamic);
                self.chain_exit(root);
            }
            ExprKind::MethodCallDyn { object, method, args, named, nullsafe } => {
                let root = self.chain_enter();
                self.expr(object)?;
                if *nullsafe {
                    let skip = self.emit(Op::JumpIfNull(Addr::MAX));
                    self.nullsafe_chain.as_mut().expect("chain open").push(skip);
                }
                self.chain_pause(|s| s.emit_method_call_dyn(method, args, named))?;
                // Value context: copy a by-reference return (see the static-call
                // arm above); `DerefTop` is a no-op for a plain result.
                self.emit(Op::DerefTop);
                self.chain_exit(root);
            }
            ExprKind::VarDyn(name) => {
                self.expr(name)?;
                self.emit(Op::LoadVarDyn);
            }
            ExprKind::VarDynAssign { name, rhs } => {
                self.expr(name)?;
                self.expr(rhs)?;
                self.emit(Op::StoreVarDyn);
            }
            ExprKind::InstanceOf { expr, class } => self.instance_of(expr, class)?,
            ExprKind::StaticCall { class, method, args, named } => {
                // `Closure::bind`/`fromCallable` are built-in statics with no compiled
                // class — must be handled before the runtime-class routing (a missing
                // `Closure` class would otherwise look "unknown").
                let closure_static =
                    matches!(class, ClassRef::Named(n) if n.eq_ignore_ascii_case(b"Closure"));
                if !closure_static && self.is_runtime_class(class) {
                    // `$cls::m()` / an unknown named class (PAR): the class reference
                    // is pushed beneath the arguments and resolved at run time.
                    self.push_class_value(class)?;
                    if !named.is_empty() {
                        // Named args ride the runtime args array (string keys =
                        // names); the dispatch binder resolves them (PHP 8.1).
                        self.build_args_array_named(args, named)?;
                        self.emit(Op::StaticCallDynamicArgs { method: method.clone() });
                    } else if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                        // Spread `$cls::m(...$a)` (Session A): args from a runtime array.
                        self.build_args_array(args)?;
                        self.emit(Op::StaticCallDynamicArgs { method: method.clone() });
                    } else {
                        // The class (hence callee) is only known at run time
                        // (`$cls::m()`, an autoloaded class): push by reference for a
                        // plain-variable argument so a run-time by-ref parameter can
                        // bind (SEND_VAR_EX); the binder decays it for by-value.
                        self.push_dyn_args(args)?;
                        self.emit(Op::StaticCallDynamic { method: method.clone(), argc: args.len() as u32 });
                    }
                } else if matches!(class, ClassRef::Named(n) if n.eq_ignore_ascii_case(b"Closure")) {
                    // `Closure::bind` / `Closure::fromCallable` are built-in statics
                    // — there is no compiled `Closure` class to resolve against.
                    if !named.is_empty()
                        || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)))
                    {
                        return Err(CompileError::Unsupported(
                            "named/spread arguments on `Closure::m()`".into(),
                        ));
                    }
                    self.push_value_args(args)?;
                    self.emit(Op::ClosureStatic { method: method.clone(), argc: args.len() as u32 });
                } else {
                    let (target, forwarding) = self.resolve_target(class)?;
                    if named.is_empty() {
                        if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                            // Spread `C::m(...$a)` (Session A): args from a runtime array.
                            self.build_args_array(args)?;
                            self.emit(Op::StaticCallArgs { target, method: method.clone(), forwarding });
                        } else {
                            let argc = args.len() as u32;
                            // For a statically-known class, honour the method's
                            // by-reference parameters (REF-2) exactly like a free
                            // function — pushing the caller's cell for a `&$p` slot.
                            // `static::` / an unresolved (autoloaded) method aren't
                            // known here, so a plain-variable argument is pushed by
                            // reference and the run-time binder decides (SEND_VAR_EX).
                            let resolved = match target {
                                ClassTarget::Class(cid) => {
                                    self.resolve_method_compile(cid, method).map(|r| (cid, r))
                                }
                                ClassTarget::Static => None,
                            };
                            match resolved {
                                Some((cid, (defc, midx))) => {
                                    let decl = &self.ctx.classes[defc].methods[midx].decl;
                                    if decl.params.iter().any(|p| p.by_ref) {
                                        let by_ref: Vec<bool> =
                                            decl.params.iter().map(|p| p.by_ref).collect();
                                        let pnames: Vec<Box<[u8]>> = decl
                                            .slots
                                            .iter()
                                            .take(decl.params.len())
                                            .cloned()
                                            .collect();
                                        let mut fname = self.ctx.classes[cid].name.to_vec();
                                        fname.extend_from_slice(b"::");
                                        fname.extend_from_slice(method);
                                        self.push_call_args(args, &by_ref, &fname, &pnames)?;
                                    } else {
                                        // Known callee with no by-ref params: values.
                                        self.push_value_args(args)?;
                                    }
                                }
                                None => self.push_dyn_args(args)?,
                            }
                            self.emit(Op::StaticCall { target, method: method.clone(), forwarding, argc });
                        }
                    } else {
                        // Named args: lay them into the resolved method's parameter
                        // slots at compile time when expressible; otherwise (an
                        // unresolved method, `static::`, a variadic/by-ref/unknown/
                        // colliding/missing name) build a runtime argument array
                        // (string keys = names) for `Op::StaticCallArgs`, whose
                        // binder resolves — or errors, catchably — at run time.
                        let layout = match target {
                            ClassTarget::Class(cid) => {
                                self.resolve_method_compile(cid, method).filter(|&(defc, midx)| {
                                    let fd = &self.ctx.classes[defc].methods[midx].decl;
                                    self.can_emit_named_layout(fd, args, named)
                                })
                            }
                            ClassTarget::Static => None,
                        };
                        match layout {
                            Some((defc, midx)) => {
                                let method_fd = &self.ctx.classes[defc].methods[midx].decl;
                                let n = method_fd.params.len() as u32;
                                self.emit_named_layout(method_fd, args, named)?;
                                self.emit(Op::StaticCall {
                                    target,
                                    method: method.clone(),
                                    forwarding,
                                    argc: n,
                                });
                            }
                            None => {
                                self.build_args_array_named(args, named)?;
                                self.emit(Op::StaticCallArgs {
                                    target,
                                    method: method.clone(),
                                    forwarding,
                                });
                            }
                        }
                    }
                }
            }
            ExprKind::StaticCallDyn { class, method, args, named } => {
                // `$cls::$m()` / `Class::$m()` / `self::$m()`: the method name is a
                // runtime value (the static analogue of `$obj->$m()`). Named or
                // spread arguments ride a runtime args array (string keys = names,
                // spreads flattened), like the instance-side `$obj->$m(...)`.
                let has_named = !named.is_empty();
                let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
                if self.is_runtime_class(class) {
                    // Runtime / unknown class: push it beneath the args, resolve at
                    // run time, dispatch non-forwarding (LSB = the resolved class).
                    self.push_class_value(class)?; // [classRef]
                    if has_named || has_spread {
                        self.build_args_array_named(args, named)?; // [classRef, argsArray]
                        self.expr(method)?; // [classRef, argsArray, method]
                        self.emit(Op::StaticCallDynamicMethodArgs);
                    } else {
                        self.push_dyn_args(args)?; // [classRef, arg0…]
                        self.expr(method)?; // [classRef, arg0…, method]
                        self.emit(Op::StaticCallDynamicMethod { argc: args.len() as u32 });
                    }
                } else {
                    // A compile-time class target (`Class`/`self`/`parent`/`static`):
                    // keep forwarding semantics ($this / LSB) as a static `StaticCall`
                    // would, only the method name is resolved at run time.
                    let (target, forwarding) = self.resolve_target(class)?;
                    if has_named || has_spread {
                        self.build_args_array_named(args, named)?; // [argsArray]
                        self.expr(method)?; // [argsArray, method]
                        self.emit(Op::StaticCallTargetDynamicMethodArgs { target, forwarding });
                    } else {
                        self.push_dyn_args(args)?; // [arg0…]
                        self.expr(method)?; // [arg0…, method]
                        self.emit(Op::StaticCallTargetDynamicMethod {
                            target,
                            forwarding,
                            argc: args.len() as u32,
                        });
                    }
                }
            }
            ExprKind::ParentHookCall { class, prop, set, args } => {
                // `parent::$prop::get()` / `parent::$prop::set($v)` (PHP 8.4). The
                // class resolves like any `::`-qualified op; a dynamic class
                // (`$cls::$prop::get()`) is not supported.
                if self.is_runtime_class(class) {
                    return Err(CompileError::Unsupported(
                        "parent hook call on a dynamic class".into(),
                    ));
                }
                let (target, _) = self.resolve_target(class)?;
                self.push_value_args(args)?;
                self.emit(Op::HookCall {
                    target,
                    prop: prop.clone(),
                    set: *set,
                    argc: args.len() as u32,
                });
            }
            ExprKind::ClassConst { class, name } => self.class_const(class, name)?,
            ExprKind::ClassConstDyn { class, name } => {
                self.push_class_value(class)?;
                self.expr(name)?;
                self.emit(Op::ClassConstDynamic);
            }
            ExprKind::StaticPropDyn { class, name } => {
                self.push_class_value(class)?;
                self.expr(name)?;
                self.emit(Op::StaticPropGetDynName);
            }
            ExprKind::StaticPropDynAssign { class, name, rhs } => {
                // rhs first, so the class+name pair sits on top for the
                // init-thunk re-run (mirrors StaticPropSetDynamic).
                self.expr(rhs)?;
                self.push_class_value(class)?;
                self.expr(name)?;
                self.emit(Op::StaticPropSetDynName);
            }
            ExprKind::StaticProp { class, name } => {
                if self.is_runtime_class(class) {
                    self.push_class_value(class)?;
                    self.emit(Op::StaticPropGetDynamic { name: name.clone() });
                } else {
                    let (target, _) = self.resolve_target(class)?;
                    self.emit(Op::StaticPropGet { target, name: name.clone() });
                }
            }
            ExprKind::StaticPropAssign { class, name, op, rhs } => {
                // `$cls::$p` (PAR): resolve the class at run time; the rhs is
                // pushed first so the class reference ends up on top.
                if self.is_runtime_class(class) {
                    match op {
                        StaticAssignOp::Plain => {
                            self.expr(rhs)?;
                            self.push_class_value(class)?;
                            self.emit(Op::StaticPropSetDynamic { name: name.clone() });
                        }
                        StaticAssignOp::Op(b) => {
                            self.expr(rhs)?;
                            self.push_class_value(class)?;
                            self.emit(Op::StaticPropOpSetDynamic { name: name.clone(), op: *b });
                        }
                        StaticAssignOp::Coalesce => {
                            // `$cls::$p ??= rhs`: the class reference is evaluated
                            // *once* into a temp and reused for the read and the
                            // conditional write (the rhs is evaluated only when the
                            // property is null).
                            let t = self.alloc_temp();
                            self.push_class_value(class)?;
                            self.emit(Op::StoreSlot(t));
                            self.emit(Op::LoadSlot(t));
                            self.emit(Op::StaticPropGetDynamic { name: name.clone() });
                            let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                            self.expr(rhs)?;
                            self.emit(Op::LoadSlot(t)); // class ref on top for the set
                            self.emit(Op::StaticPropSetDynamic { name: name.clone() });
                            let end = self.here();
                            self.patch(to_end, Op::JumpIfNotNull(end));
                            self.free_temp();
                        }
                    }
                    return Ok(());
                }
                let (target, _) = self.resolve_target(class)?;
                match op {
                    StaticAssignOp::Plain => {
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropSet { target, name: name.clone() });
                    }
                    StaticAssignOp::Op(b) => {
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropOpSet { target, name: name.clone(), op: *b });
                    }
                    StaticAssignOp::Coalesce => {
                        // `C::$p ??= rhs`: read, keep if non-null, else assign.
                        self.emit(Op::StaticPropGet { target, name: name.clone() });
                        let to_end = self.emit(Op::JumpIfNotNull(Addr::MAX));
                        self.expr(rhs)?;
                        self.emit(Op::StaticPropSet { target, name: name.clone() });
                        let end = self.here();
                        self.patch(to_end, Op::JumpIfNotNull(end));
                    }
                }
            }
            ExprKind::StaticPropIncDec { class, name, inc, pre } => {
                if self.is_runtime_class(class) {
                    // `$cls::$p++` (PAR): the class reference is resolved at run time.
                    self.push_class_value(class)?;
                    self.emit(Op::StaticPropIncDecDynamic { name: name.clone(), inc: *inc, pre: *pre });
                } else {
                    let (target, _) = self.resolve_target(class)?;
                    self.emit(Op::StaticPropIncDec { target, name: name.clone(), inc: *inc, pre: *pre });
                }
            }
            ExprKind::Yield { key, value } => {
                // `yield`, `yield $v`, `yield $k => $v` (GEN). Push the value (NULL
                // for a bare `yield`) and, if present, the key beneath it, then
                // suspend. `Op::Yield` leaves the `send()` value on the stack, so
                // the `yield` expression yields it (and `StmtKind::Expr` pops it).
                if let Some(k) = key {
                    self.expr(k)?;
                }
                match value {
                    Some(v) => self.expr(v)?,
                    None => {
                        let null = self.konst(Const::Null);
                        self.emit(Op::PushConst(null));
                    }
                }
                self.emit(Op::Yield { has_key: key.is_some() });
            }
            ExprKind::YieldFrom(delegate) => {
                // `yield from $x` (GEN-3): push the delegate, then the re-entrant
                // `YieldFrom` op drives the delegation and leaves its return value.
                self.expr(delegate)?;
                self.emit(Op::YieldFrom);
            }
            other => return Err(CompileError::Unsupported(expr_name(other))),
        }
        Ok(())
    }

    /// Compile `&&` (`want_true == false`) / `||` (`want_true == true`) to a
    /// boolean result via short-circuit jumps. Leaves `true`/`false` on the stack.
    pub(super) fn short_circuit(&mut self, a: &Expr, b: &Expr, want_true: bool) -> R<()> {
        // For `&&`: if either operand is falsy, result is false (jump to L_short).
        // For `||`: if either operand is truthy, result is true.
        let short = |s: &mut Self| {
            if want_true {
                s.emit(Op::JumpIfTrue(Addr::MAX))
            } else {
                s.emit(Op::JumpIfFalse(Addr::MAX))
            }
        };
        self.expr(a)?;
        let j1 = short(self);
        self.expr(b)?;
        let j2 = short(self);
        // Fell through: `&&` → true, `||` → false.
        let fallthrough = self.konst(Const::Bool(!want_true));
        self.emit(Op::PushConst(fallthrough));
        let to_end = self.emit(Op::Jump(Addr::MAX));
        let short_at = self.here();
        self.patch(j1, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        self.patch(j2, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        let shorted = self.konst(Const::Bool(want_true));
        self.emit(Op::PushConst(shorted));
        let end = self.here();
        self.patch(to_end, Op::Jump(end));
        Ok(())
    }

    /// Resolve a host builtin out-param argument to the slot the VM should write
    /// its produced value into: a plain variable's own slot, or — for a
    /// property/index target (`$this->pipes`) — a fresh temp whose value the
    /// caller assigns into `*place_dst` after the call. `None` when the argument
    /// was omitted; an unsupported target is a compile error.
    fn resolve_out_slot(
        &mut self,
        arg: Option<&Expr>,
        place_dst: &mut Option<Place>,
    ) -> Result<Option<crate::hir::Slot>, CompileError> {
        match arg {
            None => Ok(None),
            Some(e) => match &e.kind {
                ExprKind::Var(slot) => Ok(Some(*slot)),
                _ => match expr_field_place(e) {
                    Some(place) => {
                        *place_dst = Some(place);
                        Ok(Some(self.alloc_temp()))
                    }
                    None => Err(CompileError::Unsupported(
                        "host builtin out-param is not a plain variable".into(),
                    )),
                },
            },
        }
    }

    /// Compile a named function call `name(args...)`.
    ///
    /// Resolution mirrors the evaluator: a *user* function (matched
    /// ASCII-case-insensitively) shadows builtins; otherwise the name is looked
    /// up in the registry — a by-value builtin emits [`Op::CallBuiltin`], a
    /// by-reference-first builtin (`sort`, …) emits [`Op::CallBuiltinRef`]. A name
    /// absent from the registry (higher-order / class-introspection /
    /// `define`-family / undefined) is out of slice, so the script falls back to
    /// the tree-walker. Named/spread arguments and user by-ref/variadic params are
    /// likewise deferred.
    pub(super) fn call(
        &mut self,
        name: &[u8],
        fallback: Option<&[u8]>,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        // User functions shadow builtins — but only *hoisted* ones bind statically;
        // a conditional declaration is dispatched dynamically (callable only after
        // its `DeclareFn` runs). An unqualified call inside a namespace binds to the
        // namespaced `name` if hoisted here, otherwise to the global `fallback`
        // (PHP's two-step lookup), so the namespaced form always shadows the global.
        let hoisted = |nm: &[u8]| {
            self.ctx
                .funcs
                .iter()
                .enumerate()
                .find(|(i, f)| {
                    !self.ctx.conditional_fns.contains(i) && ascii_eq_ignore_case(&f.name, nm)
                })
                .map(|(i, _)| i)
        };
        let user_idx = hoisted(name).or_else(|| fallback.and_then(hoisted));
        // Builtins are always global, so resolve them — and the run-time dynamic
        // dispatch below — against the global-fallback name when present.
        let bname: &[u8] = fallback.unwrap_or(name);
        if let Some(idx) = user_idx {
            // Named arguments are resolved to parameter slots at compile time
            // (the callee is known), PAR.
            // A spread anywhere (`f(...$src)`, with or without named args) needs
            // run-time binding: integer keys become positional, string keys named.
            // By-value callees only (by-ref + spread is out of slice).
            if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                let callee = &self.ctx.funcs[idx];
                if callee.by_ref || callee.params.iter().any(|p| p.by_ref) {
                    return Err(CompileError::Unsupported(
                        "spread call to a by-reference function".into(),
                    ));
                }
                return self.emit_call_spread(idx, args, named);
            }
            if !named.is_empty() {
                return self.call_user_named(idx, args, named);
            }
            let callee = &self.ctx.funcs[idx];
            // Omitted optional args are filled by the callee's default prologue
            // (PAR); extra args are dropped by the binder.
            // Snapshot the by-ref mask so the immutable borrow of `callee` ends
            // before `push_call_args` borrows `self` mutably (REF-2).
            let by_ref: Vec<bool> = callee.params.iter().map(|p| p.by_ref).collect();
            let pnames: Vec<Box<[u8]>> =
                callee.params.iter().map(|p| callee.slots[p.slot as usize].clone()).collect();
            let returns_ref = callee.by_ref;
            self.push_call_args(args, &by_ref, name, &pnames)?;
            self.emit(Op::Call { func: idx as u32, argc: args.len() as u32 });
            // A `function &f()` used in value context yields a copy, not an alias
            // (REF-4b). `$y = &f()` takes the raw ref via `AssignRefCall` instead.
            if returns_ref {
                self.emit(Op::DerefTop);
            }
            return Ok(());
        }
        if !named.is_empty() {
            // Builtins with a known signature accept named arguments by
            // compile-time reordering into positionals (PHP resolves them by
            // the declared parameter names — `debug_backtrace(..., limit: 3)`).
            // Only a *contiguous* combined list is expressible (a hole would
            // need the builtin's default value); anything else keeps the error.
            if let Some(pnames) = builtin_param_names(bname) {
                if let Some(combined) = reorder_named_args(args, named, pnames) {
                    return self.call(name, fallback, &combined, &[]);
                }
            }
            return Err(CompileError::Unsupported("builtin call with named arguments".into()));
        }
        // Host builtins with a by-reference *output* parameter (`preg_match`'s
        // `&$matches` at index 2): checked BEFORE the plain host table because a
        // name can live in both (`preg_replace` — the plain entry serves dynamic
        // string-callable dispatch); capture the out-param's slot and let the VM
        // write the produced value back. The out-param is a *write* target, so its
        // argument is NOT evaluated by value — pushing it would read the (usually
        // undefined) variable and emit a spurious "Undefined variable" warning the
        // real PHP never raises. Push `null` in its place (the builtin ignores the
        // input value there).
        if let Some((canon, out_idx)) = crate::vm::host_builtin_out_param(bname) {
            // A builtin may have a *second* out-param (`exec`'s `&$result_code`).
            let out_idx2 = crate::vm::host_builtin_out_param_second(bname);
            // A property/index out-param (`proc_open(..., $this->pipes)`) is
            // written back through a temp slot after the call. Resolve each
            // out-param arg to its target slot (+ optional place).
            let mut out_place: Option<Place> = None;
            let mut out_place2: Option<Place> = None;
            let out_slot = self.resolve_out_slot(args.get(out_idx), &mut out_place)?;
            let out_slot2 = match out_idx2 {
                Some(i) => self.resolve_out_slot(args.get(i), &mut out_place2)?,
                None => None,
            };
            for (i, a) in args.iter().enumerate() {
                if i == out_idx || Some(i) == out_idx2 {
                    let k = self.konst(Const::Null);
                    self.emit(Op::PushConst(k));
                } else if matches!(a.kind, ExprKind::Spread(_)) {
                    return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
                } else {
                    self.expr(a)?;
                }
            }
            self.emit(Op::CallHostBuiltinOut {
                name: canon.into(),
                out_slot,
                out_index: out_idx as u32,
                out_slot2,
                out_index2: out_idx2.map(|i| i as u32).unwrap_or(u32::MAX),
                argc: args.len() as u32,
            });
            // Write back property/index out-params from their temps, in reverse
            // allocation order (LIFO temp discipline). Each leaves `[result]` on top.
            if let Some(place) = out_place2 {
                let rhs = Expr {
                    line: args[out_idx2.expect("out_place2 implies out_idx2")].line,
                    kind: ExprKind::Var(out_slot2.expect("temp allocated with out_place2")),
                };
                self.assign_place(&place, &rhs)?;
                self.emit(Op::Pop);
                self.free_temp();
            }
            if let Some(place) = out_place {
                let rhs = Expr {
                    line: args[out_idx].line,
                    kind: ExprKind::Var(out_slot.expect("temp allocated with out_place")),
                };
                self.assign_place(&place, &rhs)?; // [result, value]
                self.emit(Op::Pop); // [result]
                self.free_temp();
            }
            return Ok(());
        }
        // Evaluator-only *host* builtins (higher-order / class-introspection /
        // define-family, Sessions B–D) need the VM itself, so they are dispatched
        // VM-side via `Op::CallHostBuiltin` rather than the stateless registry.
        if let Some(canon) = crate::vm::host_builtin_canonical(bname) {
            // A spread (`json_encode(...$args)`) takes the same runtime-flatten
            // op as registry builtins; its VM handler routes host names too.
            if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                return self.emit_builtin_spread(canon, args);
            }
            self.push_value_args(args)?; // rejects spread (out of slice here)
            self.emit(Op::CallHostBuiltin { name: canon.into(), argc: args.len() as u32 });
            return Ok(());
        }
        // `array_multisort`: all arguments are by-reference (arrays sorted in
        // place, interleaved with by-value order/flag ints). Push every argument
        // by value and capture the writeback slot of each plain-variable array.
        if bname.eq_ignore_ascii_case(b"array_multisort") {
            let mut slots: Vec<Option<crate::hir::Slot>> = Vec::with_capacity(args.len());
            for a in args {
                if matches!(a.kind, ExprKind::Spread(_)) {
                    return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
                }
                slots.push(match &a.kind {
                    ExprKind::Var(slot) => Some(*slot),
                    _ => None,
                });
                self.expr(a)?;
            }
            self.emit(Op::CallArrayMultisort {
                arg_slots: slots.into_boxed_slice(),
                argc: args.len() as u32,
            });
            return Ok(());
        }
        // Host builtins with variadic by-reference output parameters (`sscanf`/
        // `fscanf`'s `...&$vars` from index 2): push the two fixed arguments by
        // value and capture each trailing out argument's slot (a non-variable
        // target becomes `None`, silently skipped at run time, D-54.1).
        if let Some(canon) = crate::vm::host_builtin_scanf(bname) {
            let fixed = args.len().min(2);
            self.push_value_args(&args[..fixed])?;
            let out_slots = args[fixed..]
                .iter()
                .map(|e| match &e.kind {
                    ExprKind::Var(slot) => Some(*slot),
                    _ => None,
                })
                .collect::<Vec<_>>();
            self.emit(Op::CallHostBuiltinScanf {
                name: canon.into(),
                argc: fixed as u32,
                out_slots: out_slots.into(),
            });
            return Ok(());
        }
        // By-reference-first host builtins (`usort`, …): first argument is an array
        // variable taken by reference, the rest by value (Session C).
        if let Some(canon) = crate::vm::host_builtin_ref_first(bname) {
            let Some((first, rest)) = args.split_first() else {
                return Err(CompileError::Unsupported(
                    "by-reference host builtin called with no arguments".into(),
                ));
            };
            match &first.kind {
                ExprKind::Var(slot) => {
                    let slot = *slot;
                    self.push_value_args(rest)?;
                    self.emit(Op::CallHostBuiltinRef {
                        name: canon.into(),
                        slot,
                        argc: rest.len() as u32,
                    });
                    return Ok(());
                }
                // Bare static property (`usort(self::$items, $cb)`): RMW through a
                // temp, mirroring the registry by-ref path.
                ExprKind::StaticProp { class, name: prop } => {
                    let canon: Box<[u8]> = canon.into();
                    return self.static_prop_rmw(class, prop, &[], true, |c, place| {
                        let slot = local_slot(place);
                        c.push_value_args(rest)?;
                        c.emit(Op::CallHostBuiltinRef {
                            name: canon,
                            slot,
                            argc: rest.len() as u32,
                        });
                        Ok(())
                    });
                }
                // An instance property or array element (`usort($this->q, $cb)`,
                // `usort($packages[$type], $cb)`): take a reference cell to the
                // place (single evaluation of the index keys) and *alias* it into a
                // temp slot, then run the by-ref host builtin against that slot —
                // the `ho_*` functions follow the ref and write the mutation
                // through to the place. `BindRefTo` overwrites the temp slot's
                // binding (unlike a plain `StoreSlot`, which would write *through*
                // a stale ref left in the reused temp from a previous call/loop
                // iteration, corrupting an unrelated place).
                _ => {
                    let canon: Box<[u8]> = canon.into();
                    // A prop/dim chain whose ROOT is itself a non-place
                    // expression (`reset($this->resultSetMapping()->aliasMap)`,
                    // ORM hydrators): evaluate the root once into a temp and
                    // treat `<tmp>->prop…` as the field place. Mutations
                    // through the ref reach the real object because objects
                    // are by-handle (an array-valued root would be a copy —
                    // PHP itself refuses those as by-ref arguments).
                    if expr_field_place(first).is_none() {
                        if let Some((root, steps)) = expr_rooted_field_chain(first) {
                            let root_tmp = self.alloc_temp();
                            self.expr(root)?;
                            // BindRefTo REPLACES the temp's binding (a plain
                            // StoreSlot would write through a stale ref left
                            // from a previous call/loop iteration).
                            self.emit(Op::BindRefTo {
                                base: FieldBase::Local(root_tmp),
                                steps: [].into(),
                            });
                            self.emit(Op::Pop);
                            let place = Place { base: PlaceBase::Local(root_tmp), steps };
                            let (base, fsteps) = self.field_path(&place)?;
                            self.emit(Op::MakeRef { base, steps: fsteps.into() });
                            let tmp = self.alloc_temp();
                            self.emit(Op::BindRefTo {
                                base: FieldBase::Local(tmp),
                                steps: [].into(),
                            });
                            self.emit(Op::Pop);
                            self.push_value_args(rest)?;
                            self.emit(Op::CallHostBuiltinRef {
                                name: canon,
                                slot: tmp,
                                argc: rest.len() as u32,
                            });
                            self.clear_temp_binding(tmp);
                            self.clear_temp_binding(root_tmp);
                            self.free_temp();
                            self.free_temp();
                            return Ok(());
                        }
                    }
                    let Some(place) = expr_field_place(first) else {
                        // `current(f(...))` / `key(<temp>)`: PHP 8 takes these
                        // BY VALUE (they only read the pointer, which for a
                        // temporary sits at the first element) — evaluate into
                        // a temp slot and call. The pointer-WRITING family
                        // (reset/end/next/prev) keeps the honest error.
                        if canon.as_ref() == b"current" || canon.as_ref() == b"key" {
                            let tmp = self.alloc_temp();
                            self.expr(first)?;
                            // BindRefTo REPLACES the temp's binding — a plain
                            // StoreSlot would write *through* a stale ref left
                            // in the reused temp by an earlier alias call.
                            self.emit(Op::BindRefTo {
                                base: FieldBase::Local(tmp),
                                steps: [].into(),
                            });
                            self.emit(Op::Pop);
                            self.push_value_args(rest)?;
                            self.emit(Op::CallHostBuiltinRef {
                                name: canon,
                                slot: tmp,
                                argc: rest.len() as u32,
                            });
                            self.free_temp();
                            return Ok(());
                        }
                        return Err(CompileError::Unsupported(
                            "by-reference host builtin whose first argument is not a plain variable"
                                .into(),
                        ));
                    };
                    let (base, steps) = self.field_path(&place)?;
                    self.emit(Op::MakeRef { base, steps: steps.into() }); // [ref]
                    let tmp = self.alloc_temp();
                    self.emit(Op::BindRefTo { base: FieldBase::Local(tmp), steps: [].into() }); // slot:=ref; [value]
                    self.emit(Op::Pop); // drop the aliased value
                    self.push_value_args(rest)?;
                    self.emit(Op::CallHostBuiltinRef { name: canon, slot: tmp, argc: rest.len() as u32 });
                    self.clear_temp_binding(tmp);
                    self.free_temp();
                    return Ok(());
                }
            }
        }
        // Builtins: classify by-value vs by-reference-first via the registry.
        match self.ctx.registry.get(bname) {
            Some(Builtin::Value(_)) => {
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    return self.emit_builtin_spread(bname, args);
                }
                self.push_value_args(args)?;
                self.emit(Op::CallBuiltin { name: bname.into(), argc: args.len() as u32 });
                Ok(())
            }
            Some(Builtin::RefFirst(_)) => self.call_ref_builtin(bname, args),
            None => {
                // An unknown name is NOT a compile error: PHP defers "Call to
                // undefined function" to run time (the function may be declared
                // conditionally, or be defined after this point). Push the name as a
                // string callable and dispatch dynamically — `invoke_named` raises
                // the catchable `Error` at the actual call site, after any output /
                // argument side effects, matching the tree-walker.
                if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
                }
                // An unqualified call inside a namespace whose target resolved to
                // neither a hoisted user function nor a builtin defers PHP's two-step
                // lookup (namespaced `name`, then global `fallback`) to run time — so
                // a function defined in another unit (autoloaded / included) binds.
                if let Some(fb) = fallback {
                    for a in args {
                        self.expr(a)?;
                    }
                    self.emit(Op::CallNsFallback {
                        name: name.into(),
                        fallback: fb.into(),
                        argc: args.len() as u32,
                    });
                    return Ok(());
                }
                let k = self.konst(Const::Str(name.into()));
                self.emit(Op::PushConst(k));
                for a in args {
                    self.expr(a)?;
                }
                self.emit(Op::CallValue { argc: args.len() as u32 });
                Ok(())
            }
        }
    }

    /// Compile a call to known user function `idx` that has named arguments
    /// (PAR): lay positional then named args into the parameter slots at compile
    /// time, pushing `Undef` for any skipped optional (the callee's default
    /// prologue then fills it), and emit a normal positional `Op::Call`. Falls
    /// back to the tree-walker for what the compile-time layout can't express:
    /// variadic / by-ref parameters, an unknown or duplicate name, a missing
    /// required argument, or a spread.
    pub(super) fn call_user_named(&mut self, idx: usize, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        let fd = &self.ctx.funcs[idx];
        let n = fd.params.len() as u32;
        let returns_ref = fd.by_ref;
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        // Fast path: lay the arguments into slots at compile time when expressible.
        if !has_spread && self.can_emit_named_layout(fd, args, named) {
            self.emit_named_layout(fd, args, named)?;
            self.emit(Op::Call { func: idx as u32, argc: n });
            if returns_ref {
                self.emit(Op::DerefTop);
            }
            return Ok(());
        }
        if has_spread {
            // Spread + named is the run-time spread path (handled elsewhere).
            return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
        }
        // Run-time named binding: a variadic / by-ref parameter, or a name that is
        // unknown / collides / routes into `...$rest` — all of which PHP resolves
        // (or errors) at run time rather than at compile time. Push positional args
        // (honouring by-ref), then one value per named arg (by-ref when its target
        // parameter is), and let `build_named_frame` bind them.
        let by_ref: Vec<bool> = fd.params.iter().map(|p| p.by_ref).collect();
        let fname: Box<[u8]> = fd.name.clone();
        let pnames: Vec<Box<[u8]>> =
            fd.params.iter().map(|p| fd.slots[p.slot as usize].clone()).collect();
        let named_by_ref: Vec<bool> = named
            .iter()
            .map(|(nm, _)| {
                fd.params
                    .iter()
                    .find(|p| fd.slots[p.slot as usize][..] == nm[..])
                    .is_some_and(|p| p.by_ref)
            })
            .collect();
        self.push_call_args(args, &by_ref, &fname, &pnames)?;
        for ((_, expr), &br) in named.iter().zip(&named_by_ref) {
            match (&expr.kind, br) {
                (ExprKind::Var(slot), true) => {
                    self.emit(Op::PushRef(*slot));
                }
                _ => self.expr(expr)?,
            }
        }
        self.emit(Op::CallNamed {
            func: idx as u32,
            positional: args.len() as u32,
            names: named.iter().map(|(nm, _)| nm.clone()).collect(),
        });
        if returns_ref {
            self.emit(Op::DerefTop);
        }
        Ok(())
    }

    /// Whether [`emit_named_layout`] can lay this named call into parameter slots
    /// at compile time. Mirrors its reject conditions without emitting: a variadic
    /// or by-reference parameter, a spread, too many positionals, an unknown or
    /// colliding name, or a missing required argument all force the run-time binder.
    pub(super) fn can_emit_named_layout(&self, fd: &FnDecl, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> bool {
        if fd.params.iter().any(|p| p.variadic || p.by_ref) {
            return false;
        }
        let n = fd.params.len();
        if args.len() > n || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
            return false;
        }
        let mut filled = vec![false; n];
        for slot in filled.iter_mut().take(args.len()) {
            *slot = true;
        }
        for (nm, _) in named {
            match fd
                .params
                .iter()
                .position(|p| fd.slots[p.slot as usize][..] == nm[..])
            {
                Some(pi) if !filled[pi] => filled[pi] = true,
                _ => return false, // unknown name or overwrite
            }
        }
        for p in &fd.params {
            if p.default.is_none() && !filled[p.slot as usize] {
                return false;
            }
        }
        true
    }

    /// Compile a spread function call `f(comp…, name: v, …)` (PAR): push one value
    /// per leading component — a positional value or a spread *source* (marked in
    /// `spreads`) — then the explicit named values, and let `Op::CallSpread` expand
    /// and bind them at run time.
    pub(super) fn emit_call_spread(
        &mut self,
        idx: usize,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        let returns_ref = self.ctx.funcs[idx].by_ref;
        let mut spreads = Vec::with_capacity(args.len());
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                spreads.push(true);
            } else {
                self.expr(a)?;
                spreads.push(false);
            }
        }
        for (_, expr) in named {
            self.expr(expr)?;
        }
        self.emit(Op::CallSpread {
            func: idx as u32,
            spreads: spreads.into(),
            names: named.iter().map(|(n, _)| n.clone()).collect(),
        });
        if returns_ref {
            self.emit(Op::DerefTop);
        }
        Ok(())
    }

    /// Compile a spread call into a by-value builtin `b(comp…)` (step 56b): push
    /// one value per leading component (a positional value, or a spread *source*
    /// marked in `spreads`), then let `Op::CallBuiltinSpread` flatten and run it.
    pub(super) fn emit_builtin_spread(&mut self, name: &[u8], args: &[Expr]) -> R<()> {
        let mut spreads = Vec::with_capacity(args.len());
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                spreads.push(true);
            } else {
                self.expr(a)?;
                spreads.push(false);
            }
        }
        self.emit(Op::CallBuiltinSpread { name: name.into(), spreads: spreads.into() });
        Ok(())
    }

    /// Lay named + positional arguments into `fd`'s parameter slots at compile
    /// time and emit them in slot order — pushing `Undef` for a skipped optional
    /// (the callee's default prologue fills it) — so a normal positional call op
    /// with `argc = fd.params.len()` can follow (PAR). Shared by named function,
    /// `new`, and static calls. Returns `Unsupported` for what the compile-time
    /// layout can't express: variadic / by-ref parameters, an unknown or
    /// duplicate name, a missing required argument, or a spread.
    pub(super) fn emit_named_layout(
        &mut self,
        fd: &FnDecl,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        if fd.params.iter().any(|p| p.variadic || p.by_ref) {
            return Err(CompileError::Unsupported(
                "named arguments with a variadic or by-reference parameter".into(),
            ));
        }
        let n = fd.params.len();
        if args.len() > n {
            return Err(CompileError::Unsupported(
                "named call with too many positional arguments".into(),
            ));
        }
        // Lay each argument into its parameter slot; `None` is a skipped optional.
        let mut slots: Vec<Option<&Expr>> = vec![None; n];
        for (i, a) in args.iter().enumerate() {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            slots[i] = Some(a);
        }
        for (nm, expr) in named {
            let pos = fd
                .params
                .iter()
                .position(|p| fd.slots[p.slot as usize][..] == nm[..]);
            match pos {
                Some(pi) if slots[pi].is_none() => slots[pi] = Some(expr),
                Some(_) => {
                    return Err(CompileError::Unsupported(
                        "named argument overwrites a positional one".into(),
                    ))
                }
                None => {
                    return Err(CompileError::Unsupported(format!(
                        "unknown named parameter ${}",
                        String::from_utf8_lossy(nm)
                    )))
                }
            }
        }
        // Every required (default-less) parameter must be supplied.
        for p in &fd.params {
            if p.default.is_none() && slots[p.slot as usize].is_none() {
                return Err(CompileError::Unsupported(
                    "named call missing a required argument".into(),
                ));
            }
        }
        // Emit in slot order; a gap pushes `Undef` for the default prologue.
        for s in slots {
            match s {
                Some(e) => self.expr(e)?,
                None => {
                    self.emit(Op::PushUndef);
                }
            }
        }
        Ok(())
    }

    /// Push each positional argument for a user call, honouring by-reference
    /// parameters (REF-2): a by-ref position whose argument is a plain variable
    /// is passed by [`Op::PushRef`] (the callee slot aliases the caller's cell);
    /// every other position is pushed by value. A by-ref position with a
    /// non-variable argument (e.g. a literal) is a *run-time* catchable `Error` in
    /// PHP — `fname(): Argument #N ($p) could not be passed by reference` — so it
    /// compiles to an [`Op::Fatal`] in place rather than a compile rejection.
    /// `fname` is the callee's display name and `pnames` its parameter names
    /// (indexed positionally) for that message.
    pub(super) fn push_call_args(
        &mut self,
        args: &[Expr],
        by_ref: &[bool],
        fname: &[u8],
        pnames: &[Box<[u8]>],
    ) -> R<()> {
        for (i, a) in args.iter().enumerate() {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            if by_ref.get(i).copied().unwrap_or(false) {
                match &a.kind {
                    ExprKind::Var(slot) => {
                        self.emit(Op::PushRef(*slot));
                    }
                    _ => {
                        // A non-variable *place* (`$a[$k]`, `$this->p` — sebastian/
                        // exporter's recursive by-ref descent) binds via `MakeRef`,
                        // exactly like a by-ref builtin argument. A true non-place
                        // (literal, call result) stays PHP's run-time Error.
                        if let Some(place) = expr_field_place(a) {
                            let (base, steps) = self.field_path(&place)?;
                            self.emit(Op::MakeRef { base, steps: steps.into() });
                            continue;
                        }
                        let pname = pnames.get(i).map(|n| n.as_ref()).unwrap_or(b"");
                        let msg = format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(fname),
                            i + 1,
                            String::from_utf8_lossy(pname),
                        );
                        let k = self.konst(Const::Str(msg.into_bytes().into()));
                        self.emit(Op::Fatal(k));
                        return Ok(());
                    }
                };
            } else {
                self.expr(a)?;
            }
        }
        Ok(())
    }

    /// Build a runtime argument array on the stack from `args`, expanding spreads
    /// (`...$src` via [`Op::ArrayAppendSpread`]) and pushing positional values
    /// (via [`Op::ArrayPush`]). Mirrors the `f(...$arr)` path (PAR-13) but feeds a
    /// dynamic-dispatch call (`$obj->m(...)`, `new C(...)`, `C::m(...)`, Session A)
    /// whose callee — hence parameter count — isn't known at compile time. Leaves
    /// the array on top of the stack.
    pub(super) fn build_args_array(&mut self, args: &[Expr]) -> R<()> {
        self.emit(Op::ArrayInit);
        for a in args {
            if let ExprKind::Spread(src) = &a.kind {
                self.expr(src)?;
                self.emit(Op::ArrayAppendSpread);
            } else {
                self.expr(a)?;
                self.emit(Op::ArrayPush);
            }
        }
        Ok(())
    }

    pub(super) fn emit_method_call(
        &mut self,
        method: &[u8],
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
        recv_class: Option<ClassId>,
    ) -> R<()> {
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        if !named.is_empty() {
            if has_spread {
                return Err(CompileError::Unsupported(
                    "method call mixing argument unpacking and named arguments".into(),
                ));
            }
            // Positional values first, then each named value (its label rides in the
            // op); the run-time binder maps names against the callee's params.
            self.push_value_args(args)?;
            for (_, expr) in named {
                self.expr(expr)?;
            }
            self.emit(Op::MethodCallNamed {
                method: method.into(),
                positional: args.len() as u32,
                names: named.iter().map(|(n, _)| n.clone()).collect(),
            });
        } else if has_spread {
            self.build_args_array(args)?;
            self.emit(Op::MethodCallArgs { method: method.into() });
        } else {
            // When the receiver's class is statically known (a `$this->m(...)` call
            // within a method body), honour the method's by-reference parameters
            // (REF-2) by pushing the caller's cell for a `&$p` slot — exactly like a
            // static `C::m()` call. A dynamic receiver (`$obj->m()`) stays by-value:
            // the callee, hence its by-ref params, is only known at run time.
            let mut resolved_here = false;
            if let Some(cid) = recv_class {
                if let Some((defc, midx)) = self.resolve_method_compile(cid, method) {
                    resolved_here = true;
                    let decl = &self.ctx.classes[defc].methods[midx].decl;
                    if decl.params.iter().any(|p| p.by_ref) {
                        let by_ref: Vec<bool> = decl.params.iter().map(|p| p.by_ref).collect();
                        let pnames: Vec<Box<[u8]>> =
                            decl.slots.iter().take(decl.params.len()).cloned().collect();
                        let mut fname = self.ctx.classes[cid].name.to_vec();
                        fname.extend_from_slice(b"::");
                        fname.extend_from_slice(method);
                        self.push_call_args(args, &by_ref, &fname, &pnames)?;
                    } else {
                        // Known callee with no by-ref params: plain values.
                        self.push_value_args(args)?;
                    }
                }
            }
            if !resolved_here {
                // Dynamic receiver (`$obj->m()`) or an unresolved method: the callee
                // — hence its by-ref params — is only known at run time, so push a
                // plain-variable argument by reference (SEND_VAR_EX); `bind_params`
                // decays it for a by-value parameter.
                self.push_dyn_args(args)?;
            }
            self.emit(Op::MethodCall { method: method.into(), argc: args.len() as u32 });
        }
        Ok(())
    }

    /// Emit a dynamic instance method call `$obj->$m(args)` / `$obj->{expr}(args)`
    /// (the receiver is already on the stack). The method-name expression is pushed
    /// last — on top of the positional args / args-array — so the VM pops the name,
    /// then dispatches on the same `[obj, args…]` layout as the static path (step
    /// 51). Named arguments on a dynamic call fall back to the evaluator.
    pub(super) fn emit_method_call_dyn(
        &mut self,
        method: &Expr,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> R<()> {
        let has_spread = args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_)));
        if !named.is_empty() {
            // Named args ride the runtime args array (string keys = names); the
            // handler splits and binds them like the spread path (PHP 8.1).
            self.build_args_array_named(args, named)?; // [obj, argsArray]
            self.expr(method)?; // [obj, argsArray, name]
            self.emit(Op::MethodCallDynamicArgs);
        } else if has_spread {
            self.build_args_array(args)?; // [obj, argsArray]
            self.expr(method)?; // [obj, argsArray, name]
            self.emit(Op::MethodCallDynamicArgs);
        } else {
            // `$obj->$m(args)`: the callee is only known at run time, so a
            // plain-variable argument is pushed by reference (SEND_VAR_EX) and the
            // binder decays it for a by-value parameter.
            self.push_dyn_args(args)?; // [obj, arg0…]
            self.expr(method)?; // [obj, arg0…, name]
            self.emit(Op::MethodCallDynamic { argc: args.len() as u32 });
        }
        Ok(())
    }

    /// Push each positional argument's value (source order); reject spreads.
    pub(super) fn push_value_args(&mut self, args: &[Expr]) -> R<()> {
        for a in args {
            if matches!(a.kind, ExprKind::Spread(_)) {
                return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
            }
            self.expr(a)?;
        }
        Ok(())
    }

    /// Push each positional argument for a **dynamic** call whose callee — hence
    /// which parameters are by-reference — isn't known at compile time (a
    /// non-`$this` receiver `$obj->m(…)`, `$obj->$m(…)`, `$cls::m(…)`, an
    /// autoloaded/`static::` static). Every plain-variable argument is passed by
    /// [`Op::PushRef`] (PHP's SEND_VAR_EX): the callee's cell aliases the caller's
    /// so a by-reference parameter can write through, and an undefined variable
    /// materialises rather than warning. The run-time `bind_params` decays the
    /// reference back to a value for a by-value parameter, so this is a no-op for
    /// the common case. Non-variable arguments push their value; spreads are
    /// rejected (the caller routes those through `build_args_array`).
    pub(super) fn push_dyn_args(&mut self, args: &[Expr]) -> R<()> {
        for a in args {
            match &a.kind {
                ExprKind::Spread(_) => {
                    return Err(CompileError::Unsupported("argument unpacking (spread)".into()));
                }
                ExprKind::Var(slot) => {
                    self.emit(Op::PushRef(*slot));
                }
                _ => {
                    self.expr(a)?;
                }
            }
        }
        Ok(())
    }

    /// Emit a by-reference-first builtin call (`sort`, `array_push`, …). As the
    /// evaluator requires, the first argument must be a plain variable: it is
    /// passed by reference via its slot, the rest by value.
    pub(super) fn call_ref_builtin(&mut self, name: &[u8], args: &[Expr]) -> R<()> {
        let Some((first, rest)) = args.split_first() else {
            return Err(CompileError::Unsupported(
                "by-reference builtin called with no arguments".into(),
            ));
        };
        match &first.kind {
            ExprKind::Var(slot) => {
                let slot = *slot;
                // A spread among the by-value rest (`array_push($a, ...$b)`): push
                // one value per component (spread *sources* marked) and flatten at
                // run time, mirroring the by-value `emit_builtin_spread` path.
                if rest.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
                    let mut spreads = Vec::with_capacity(rest.len());
                    for a in rest {
                        if let ExprKind::Spread(src) = &a.kind {
                            self.expr(src)?;
                            spreads.push(true);
                        } else {
                            self.expr(a)?;
                            spreads.push(false);
                        }
                    }
                    self.emit(Op::CallBuiltinRefSpread {
                        name: name.into(),
                        slot,
                        spreads: spreads.into(),
                    });
                    return Ok(());
                }
                self.push_value_args(rest)?;
                self.emit(Op::CallBuiltinRef { name: name.into(), slot, argc: rest.len() as u32 });
                Ok(())
            }
            // A bare static property as the by-reference argument
            // (`array_pop(self::$stack)`): read-modify-write through a temp — load
            // the property, run the builtin in place on the temp slot (leaving its
            // result), then write the mutated temp back into the property.
            ExprKind::StaticProp { class, name: prop } => {
                let nm: Box<[u8]> = name.into();
                self.static_prop_rmw(class, prop, &[], true, |c, place| {
                    let slot = local_slot(place);
                    c.push_value_args(rest)?;
                    c.emit(Op::CallBuiltinRef { name: nm, slot, argc: rest.len() as u32 });
                    Ok(())
                })
            }
            // An instance property or array element (`array_pop($this->q)`,
            // `sort($data['list'])`, `array_push($a[0], $x)`): produce a reference
            // cell to the place with `MakeRef` (single evaluation of the index
            // keys), push the rest by value, and let the builtin mutate the cell in
            // place (write-through). Mirrors how a by-ref *parameter* takes such an
            // argument; the plain-variable and static-property fast paths above
            // avoid the extra cell.
            _ => {
                if let Some(place) = expr_field_place(first) {
                    let (base, steps) = self.field_path(&place)?;
                    self.emit(Op::MakeRef { base, steps: steps.into() });
                    self.push_value_args(rest)?;
                    self.emit(Op::CallBuiltinRefCell {
                        name: name.into(),
                        argc: rest.len() as u32,
                    });
                    return Ok(());
                }
                // An *indexed* static property
                // (`array_unshift(self::$hooks[$class]['before'], $m)`): peel the
                // index chain down to the static-prop root and go through the same
                // read-modify-write temp as the bare static property above, with
                // the indexes as path steps — the builtin mutates a reference cell
                // into the temp's path, then the temp is written back.
                let mut steps_rev: Vec<PlaceStep> = Vec::new();
                let mut cur = first;
                while let ExprKind::Index { base, index } = &cur.kind {
                    steps_rev.push(PlaceStep::Index((**index).clone()));
                    cur = base;
                }
                if let ExprKind::StaticProp { class, name: prop } = &cur.kind {
                    steps_rev.reverse();
                    let nm: Box<[u8]> = name.into();
                    return self.static_prop_rmw(class, prop, &steps_rev, true, |c, place| {
                        let (base, psteps) = c.field_path(place)?;
                        c.emit(Op::MakeRef { base, steps: psteps.into() });
                        c.push_value_args(rest)?;
                        c.emit(Op::CallBuiltinRefCell { name: nm, argc: rest.len() as u32 });
                        Ok(())
                    });
                }
                Err(CompileError::Unsupported(
                    "by-reference builtin whose first argument is not a plain variable".into(),
                ))
            }
        }
    }

    /// Emit the run-time constructor invocation for `new static` / `new $cls` (the
    /// receiver is already duplicated on the stack). A spread (`...$a`) or a
    /// **named** argument builds a runtime argument array (string keys = names)
    /// and uses [`Op::InvokeCtorArgs`], whose handler binds by name against the
    /// run-time-resolved constructor; otherwise the values are pushed positionally
    /// for [`Op::InvokeCtor`].
    pub(super) fn emit_invoke_ctor(&mut self, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        if !named.is_empty() || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
            self.build_args_array_named(args, named)?;
            self.emit(Op::InvokeCtorArgs);
        } else {
            self.push_value_args(args)?;
            self.emit(Op::InvokeCtor { argc: args.len() as u32 });
        }
        Ok(())
    }

    /// [`Self::build_args_array`] plus trailing **named** arguments, inserted with
    /// their parameter name as string key (the encoding `Op::InvokeCtorArgs`
    /// decodes back into a named binding — see `split_args_from_array_value`).
    pub(super) fn build_args_array_named(&mut self, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        self.build_args_array(args)?;
        for (name, e) in named {
            let k = self.konst(Const::Str(name.clone()));
            self.emit(Op::PushConst(k));
            self.expr(e)?;
            self.emit(Op::ArrayInsert);
        }
        Ok(())
    }

    /// Compile `new ClassRef(args)` (no named / spread arguments). OOP-1 handled
    /// `Named`; OOP-2a adds `self` / `parent` (class id known at compile time) and
    /// `static` (the run-time LSB class). `Dynamic` stays out of slice.
    pub(super) fn new_obj(&mut self, class: &ClassRef, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        match class {
            ClassRef::Named(name) => match self.resolve_class(name) {
                // Known at compile time: allocate + run the resolved constructor.
                Some(cid) => self.new_obj_cid(cid, args, named),
                // Unknown at compile time: PHP resolves `new X` at run time and
                // raises a catchable `Error: Class "X" not found` only if it is
                // truly undefined (it may be declared conditionally or later). Push
                // the name and allocate dynamically, exactly like `new $cls`.
                None => {
                    let k = self.konst(Const::Str(name.clone()));
                    self.emit(Op::PushConst(k));
                    self.emit_dynamic_new(args, named)
                }
            },
            ClassRef::SelfClass => {
                let cid = self
                    .cur_class
                    .ok_or_else(|| CompileError::Unsupported("`new self` outside class context".into()))?;
                self.new_obj_cid(cid, args, named)
            }
            ClassRef::Parent => {
                let cid = self
                    .cur_class
                    .and_then(|c| self.ctx.classes[c].parent)
                    .ok_or_else(|| CompileError::Unsupported("`new parent` without a parent class".into()))?;
                self.new_obj_cid(cid, args, named)
            }
            ClassRef::Static => {
                // The actual class (hence the constructor) is only known at run
                // time, so allocate the LSB class and dispatch `__construct`
                // dynamically.
                self.emit(Op::AllocStatic);
                self.emit(Op::Dup);
                self.emit(Op::InitProps);
                self.emit(Op::Pop);
                // Fix line/file/trace on a Throwable after its defaults are set.
                self.emit(Op::StampThrowable);
                self.emit(Op::Dup);
                self.emit_invoke_ctor(args, named)?;
                self.emit(Op::Pop);
                Ok(())
            }
            ClassRef::Dynamic(expr) => {
                // `new $cls` (PAR): resolve the class at run time, then run the
                // constructor dynamically (like `new static`).
                self.expr(expr)?;
                self.emit_dynamic_new(args, named)
            }
        }
    }

    /// Emit the run-time `new` sequence for a class **value already on the stack**
    /// (a `new $cls` or a `new Name` whose class is unknown at compile time):
    /// resolve the class, init its properties, stamp a Throwable's location, and run
    /// the constructor dynamically, leaving the fresh object on the stack.
    pub(super) fn emit_dynamic_new(&mut self, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        self.emit(Op::AllocDynamic);
        self.emit(Op::Dup);
        self.emit(Op::InitProps);
        self.emit(Op::Pop);
        self.emit(Op::StampThrowable);
        self.emit(Op::Dup);
        self.emit_invoke_ctor(args, named)?;
        self.emit(Op::Pop);
        Ok(())
    }

    /// `new` of a class whose id is known at compile time: allocate, then run the
    /// compile-time-resolved constructor (if any) with the fresh object as `$this`.
    pub(super) fn new_obj_cid(&mut self, cid: ClassId, args: &[Expr], named: &[(Box<[u8]>, Expr)]) -> R<()> {
        let ctor = self.resolve_method_compile(cid, b"__construct");
        self.emit(Op::Alloc { class: cid });
        // Materialise non-constant property defaults before the constructor runs.
        // `InitProps` is a no-op (pushes NULL) for classes with none.
        self.emit(Op::Dup);
        self.emit(Op::InitProps);
        self.emit(Op::Pop);
        // After defaults are in place, fix a Throwable's line/file/trace at the
        // `new` site (a no-op for non-Throwables), before the constructor runs.
        self.emit(Op::StampThrowable);
        // Spread `new C(...$a)` (with or without named arguments), and any named
        // call the compile-time layout can't express (a variadic or by-ref
        // parameter, an unknown/colliding name, a missing required argument, a
        // ctor-less class): resolve the constructor at run time from the fresh
        // object's class (`InvokeCtorArgs`), whose binder handles variadics and
        // raises PHP's catchable errors ("Unknown named parameter", overwrite,
        // too-few) only if the `new` actually executes.
        let named_needs_runtime = !named.is_empty()
            && match ctor {
                Some((defc, midx)) => {
                    let fd = &self.ctx.classes[defc].methods[midx].decl;
                    !self.can_emit_named_layout(fd, args, named)
                }
                None => true,
            };
        if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) || named_needs_runtime {
            self.emit(Op::Dup);
            self.build_args_array_named(args, named)?;
            self.emit(Op::InvokeCtorArgs);
            self.emit(Op::Pop);
            return Ok(());
        }
        if let Some((defc, midx)) = ctor {
            self.emit(Op::Dup); // keep the instance as the result; the dup is the receiver
            let argc = if named.is_empty() {
                self.push_value_args(args)?;
                args.len() as u32
            } else {
                // Resolve named arguments against the constructor's parameters (PAR).
                let ctor_fd = &self.ctx.classes[defc].methods[midx].decl;
                let n = ctor_fd.params.len() as u32;
                self.emit_named_layout(ctor_fd, args, named)?;
                n
            };
            self.emit(Op::InvokeMethod { class: defc, method_idx: midx as u32, argc });
            self.emit(Op::Pop); // discard the constructor's return value
        }
        Ok(())
    }

    /// Compile `expr instanceof ClassRef`. `Named`/`self`/`parent` resolve to a
    /// compile-time id; `static` tests the run-time LSB class. A named class not
    /// known at compile time is resolved by name at run time (so a class later
    /// provided by `eval`/`include`, or a conditional declaration, is honoured) —
    /// an unresolvable name still tests false, as PHP does.
    pub(super) fn instance_of(&mut self, expr: &Expr, class: &ClassRef) -> R<()> {
        // Evaluate the operand first (PHP order), then test the class.
        match class {
            ClassRef::Named(name) => {
                self.expr(expr)?;
                match self.resolve_class(name) {
                    Some(cid) => self.emit(Op::InstanceOf { class: cid }),
                    None => match builtin_iface_for(name) {
                        // A built-in interface (Generator/Iterator/Traversable)
                        // has no ClassId; decide membership by runtime type.
                        Some(iface) => self.emit(Op::InstanceOfBuiltin(iface)),
                        None => {
                            // Unknown at compile time: push the name and resolve it
                            // against the live class table at run time.
                            let n = self.konst(Const::Str(name.clone()));
                            self.emit(Op::PushConst(n));
                            self.emit(Op::InstanceOfDynamic)
                        }
                    },
                };
            }
            ClassRef::SelfClass | ClassRef::Parent => {
                let (ClassTarget::Class(cid), _) = self.resolve_target(class)? else {
                    unreachable!("self/parent resolve to a class id")
                };
                self.expr(expr)?;
                self.emit(Op::InstanceOf { class: cid });
            }
            ClassRef::Static => {
                self.expr(expr)?;
                self.emit(Op::InstanceOfStatic);
            }
            ClassRef::Dynamic(cls) => {
                // `$x instanceof $cls` (PAR): evaluate the operand, then the class
                // reference, and test at run time.
                self.expr(expr)?;
                self.expr(cls)?;
                self.emit(Op::InstanceOfDynamic);
            }
        }
        Ok(())
    }

    /// Compile `ClassRef::name` — a class constant or the special `::class`.
    pub(super) fn class_const(&mut self, class: &ClassRef, name: &[u8]) -> R<()> {
        // `::class` is a compile-time constant. A *named* class yields its
        // fully-qualified name as a string even when the class is undefined (PHP
        // resolves the name, not the class; the lowerer already made it the FQN).
        if name.eq_ignore_ascii_case(b"class") {
            match class {
                ClassRef::Named(n) => {
                    let k = self.konst(Const::Str(n.clone()));
                    self.emit(Op::PushConst(k));
                }
                ClassRef::Dynamic(cexpr) => {
                    self.expr(cexpr)?;
                    self.emit(Op::ClassConstFromValue { name: name.into() });
                }
                _ => match self.resolve_target(class)?.0 {
                    ClassTarget::Class(cid) => {
                        let k = self.konst(Const::Str(self.ctx.classes[cid].name.clone()));
                        self.emit(Op::PushConst(k));
                    }
                    ClassTarget::Static => {
                        self.emit(Op::ClassNameStatic);
                    }
                },
            }
            return Ok(());
        }
        // A class constant `C::NAME`: `$cls::NAME` or an unknown named class resolve
        // the class from a run-time value (PHP: `Class "X" not found` if missing).
        if self.is_runtime_class(class) {
            self.push_class_value(class)?;
            self.emit(Op::ClassConstFromValue { name: name.into() });
            return Ok(());
        }
        let (target, _forwarding) = self.resolve_target(class)?;
        match target {
            ClassTarget::Class(cid) => match self.find_class_const(cid, name) {
                Some((decl, idx)) => {
                    self.emit(Op::ClassConst { class: decl, idx: idx as u32 });
                }
                // An enum case `E::Case` (Session A): materialise its singleton at
                // run time. Cases are matched case-sensitively (like PHP); a backed
                // case whose value did not const-fold is not materialisable and
                // falls through to the evaluator.
                None => match self.enum_case_index(cid, name) {
                    Some(case) => {
                        self.emit(Op::EnumCase { class: cid, case });
                    }
                    None => {
                        return Err(CompileError::Unsupported(format!(
                            "class constant `{}` (undefined here, or an enum case)",
                            String::from_utf8_lossy(name)
                        )))
                    }
                },
            },
            ClassTarget::Static => {
                self.emit(Op::ClassConstDyn { name: name.into() });
            }
        }
        Ok(())
    }

    /// Whether `class` is resolved at run time rather than compile time: a
    /// `$expr::` dynamic reference, or a named class unknown at compile time. PHP
    /// resolves an unknown named class at the point of use (throwing `Class "X"
    /// not found` if still missing), so we defer it to the same dynamic ops as
    /// `$cls::…` instead of failing to compile (step 50 follow-up).
    pub(super) fn is_runtime_class(&self, class: &ClassRef) -> bool {
        match class {
            ClassRef::Dynamic(_) => true,
            ClassRef::Named(n) => self.resolve_class(n).is_none(),
            _ => false,
        }
    }

    /// Push the run-time class value for a reference where [`Self::is_runtime_class`]
    /// holds: the evaluated `$expr`, or the (already fully-qualified) class name as
    /// a string the VM resolves via the class table.
    pub(super) fn push_class_value(&mut self, class: &ClassRef) -> R<()> {
        match class {
            ClassRef::Dynamic(e) => self.expr(e),
            ClassRef::Named(n) => {
                let k = self.konst(Const::Str(n.clone()));
                self.emit(Op::PushConst(k));
                Ok(())
            }
            _ => unreachable!("push_class_value on a compile-time class ref"),
        }
    }

    /// Resolve a `ClassRef` to a [`ClassTarget`] plus whether the call is
    /// *forwarding* (`self`/`parent`/`static` keep the caller's LSB class and
    /// `$this`; a named class rebinds them). `self`/`parent` collapse to a
    /// compile-time class id; `static` stays run-time.
    pub(super) fn resolve_target(&self, class: &ClassRef) -> R<(ClassTarget, bool)> {
        match class {
            ClassRef::Named(name) => {
                let cid = self.resolve_class(name).ok_or_else(|| {
                    CompileError::Unsupported(format!(
                        "reference to unknown class `{}`",
                        String::from_utf8_lossy(name)
                    ))
                })?;
                Ok((ClassTarget::Class(cid), false))
            }
            ClassRef::SelfClass => {
                let cid = self
                    .cur_class
                    .ok_or_else(|| CompileError::Unsupported("`self` outside class context".into()))?;
                Ok((ClassTarget::Class(cid), true))
            }
            ClassRef::Parent => {
                let cid = self
                    .cur_class
                    .and_then(|c| self.ctx.classes[c].parent)
                    .ok_or_else(|| CompileError::Unsupported("`parent` without a parent class".into()))?;
                Ok((ClassTarget::Class(cid), true))
            }
            ClassRef::Static => Ok((ClassTarget::Static, true)),
            ClassRef::Dynamic(_) => Err(CompileError::Unsupported("dynamic class reference".into())),
        }
    }

    /// Find a class constant by name at compile time, searching the class's own
    /// constants and parent chain, then (transitively) its interfaces. Returns the
    /// declaring class id and the constant's index in that class's `consts`
    /// (matching [`CompiledClass::consts`]). Case-sensitive, like PHP.
    pub(super) fn find_class_const(&self, cid: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
        let classes = self.ctx.classes;
        let mut c = Some(cid);
        while let Some(x) = c {
            if let Some(i) = classes[x].consts.iter().position(|k| k.name.as_ref() == name) {
                return Some((x, i));
            }
            c = classes[x].parent;
        }
        let mut c = Some(cid);
        while let Some(x) = c {
            for &i in &classes[x].interfaces {
                if let Some(r) = self.find_class_const(i, name) {
                    return Some(r);
                }
            }
            c = classes[x].parent;
        }
        None
    }

    /// The index of enum `cid`'s case `name` (case-sensitive, like PHP), if `cid`
    /// is an enum, the case exists, and it is *materialisable* by the VM — a pure
    /// case, or a backed case whose value const-folds (Session A). A backed case
    /// with a non-folding value returns `None` so `E::Case` falls back to the
    /// evaluator. The index matches [`CompiledClass::enum_cases`] (1:1 with source).
    pub(super) fn enum_case_index(&self, cid: ClassId, name: &[u8]) -> Option<u32> {
        let cd = &self.ctx.classes[cid];
        if !cd.is_enum {
            return None;
        }
        let i = cd.enum_cases.iter().position(|c| c.name.as_ref() == name)?;
        let case = &cd.enum_cases[i];
        let materialisable = match &case.value {
            None => true,
            Some(e) => const_eval_in_class(e, cid, self.ctx, 0).is_some(),
        };
        materialisable.then_some(i as u32)
    }

    /// Resolve a class name (case-insensitive) to its [`ClassId`].
    pub(super) fn resolve_class(&self, name: &[u8]) -> Option<ClassId> {
        self.ctx.class_index.get(&name.to_ascii_lowercase()).copied()
    }

    /// Resolve a method by name at compile time, walking the parent chain
    /// child→ancestor; returns the *defining* class id and the method's index in
    /// that class's `methods` (matching [`CompiledClass::methods`]).
    pub(super) fn resolve_method_compile(&self, start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
        let classes = self.ctx.classes;
        let mut cid = Some(start);
        while let Some(c) = cid {
            if let Some(i) = classes[c]
                .methods
                .iter()
                .position(|m| m.decl.name.eq_ignore_ascii_case(name))
            {
                return Some((c, i));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// If `place` is a single-step property access on `$this` or a local
    /// (`$this->p` / `$o->p`), push the object onto the stack and return the
    /// property name; otherwise return `None` so the caller falls through to the
    /// mixed field path (`field_path` / `FieldAssign`) or the array path. A
    /// `$GLOBALS`-rooted property (`$GLOBALS['x']->p`) returns `None` too: the
    /// `FieldBase::Global` field path handles it (the [`Op::PropSet`] fast path
    /// only roots at `$this` / a local slot).
    pub(super) fn prop_place(&mut self, place: &Place) -> R<Option<Box<[u8]>>> {
        if place.steps.len() != 1 {
            return Ok(None);
        }
        let PlaceStep::Prop(name) = &place.steps[0] else {
            return Ok(None);
        };
        match place.base {
            PlaceBase::This => {
                self.emit(Op::This);
            }
            PlaceBase::Local(s) => {
                self.emit(Op::LoadSlot(s));
            }
            // A `$GLOBALS`-rooted property write goes through the field path; an
            // indexed static-property target is rewritten before reaching here. A
            // class-constant test base is materialised into a temp before any
            // property step, so it never reaches here either.
            PlaceBase::Global(_)
            | PlaceBase::Superglobal(_)
            | PlaceBase::StaticProp { .. }
            | PlaceBase::ClassConst { .. }
            // Value bases are rewritten through a temp (`value_base_rmw`)
            // before any place operation, so they never reach here.
            | PlaceBase::Value(_) => return Ok(None),
        }
        Ok(Some(name.clone()))
    }

    /// The dynamic-name twin of [`Self::prop_place`]: a single `->{expr}` /
    /// `->$k` step on `$this` or a local. Pushes `[obj, name]` and reports
    /// `true`; the caller emits the `…Dyn` opcode.
    pub(super) fn prop_place_dyn(&mut self, place: &Place) -> R<bool> {
        if place.steps.len() != 1 {
            return Ok(false);
        }
        let PlaceStep::PropDyn(name) = &place.steps[0] else {
            return Ok(false);
        };
        let name = name.clone();
        match place.base {
            PlaceBase::This => {
                self.emit(Op::This);
            }
            PlaceBase::Local(s) => {
                self.emit(Op::LoadSlot(s));
            }
            PlaceBase::Global(_)
            | PlaceBase::Superglobal(_)
            | PlaceBase::StaticProp { .. }
            | PlaceBase::ClassConst { .. }
            | PlaceBase::Value(_) => return Ok(false),
        }
        self.expr(&name)?;
        Ok(true)
    }
}
