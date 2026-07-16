//! Statement tail: try/switch/match + assignment/lvalue/isset emission. Split from compile/mod.rs.
use super::*;

/// The static-property NAME for the rmw/read wrappers: a compile-time literal
/// (`self::$arr[k]`) or an expression resolved at run time
/// (`self::${$n}[k]`, DebugClassLoader), evaluated exactly once.
pub(super) enum SpName {
    Lit(Box<[u8]>),
    Dyn(Expr),
}

/// The (class, name) parts of a static-property-rooted place, for the
/// rmw/read wrappers — `None` for every other base.
fn static_place_parts(base: &PlaceBase) -> Option<(ClassRef, SpName)> {
    match base {
        PlaceBase::StaticProp { class, name } => {
            Some((class.clone(), SpName::Lit(name.clone())))
        }
        PlaceBase::StaticPropDyn { class, name } => {
            Some((class.clone(), SpName::Dyn((**name).clone())))
        }
        _ => None,
    }
}

impl<'a> super::FnCompiler<'a> {
    /// Lower a mixed property/index place (`$o->a[$k]`, `$this->x->y`, …) into a
    /// [`FieldBase`] plus a [`FieldStep`] list, emitting each `Index` step's key
    /// expression in source order (consumed at run time beneath the value). The
    /// in-place vs copy-on-write distinction between object and array steps is the
    /// VM's job; the compiler only records the shape.
    /// Compile `try { body } catch (...) { } [finally { }]` (EXC). The body's op
    /// range becomes a *catch* region (→ a `CatchMatch`-per-clause / `Rethrow`
    /// dispatch) and, when a `finally` is present, the body+catches range also
    /// becomes a *finally* region (→ the finally body, re-raising at `EndFinally`)
    /// — so normal, caught, and propagating exits all run `finally`, and nesting
    /// works via re-raise. EXC-2 scope: a `return`/`break`/`continue`/`goto`
    /// crossing a `finally` is out of slice (falls back to the evaluator).
    pub(super) fn try_stmt(&mut self, body: &[Stmt], catches: &[CatchClause], finally: &[Stmt]) -> R<()> {
        let has_finally = !finally.is_empty();
        // `return`/`break`/`continue` and now `goto` crossing a finally are handled
        // (EXC-2b): `resolve_gotos` routes a goto that leaves a finally's protected
        // region through it. A goto confined to the finally body does not cross it.
        // Snapshot the scope depth outside the try (its own level) so the goto router
        // can tell an outside-the-body target from an inside-the-body one.
        let outer_scope_len = self.scope_path.len();
        let start = self.here();
        if has_finally {
            // Route control transfers in the body/catches through this finally.
            self.finally_scopes.push(FinallyScope { sites: Vec::new(), loop_depth: self.loops.len() });
        }
        self.block(body)?;
        let body_end = self.here();
        let after_body = self.emit(Op::Jump(Addr::MAX)); // normal completion → finally / after
        let catch_addr = self.here();
        if !catches.is_empty() {
            self.exc_regions.push(ExcRegion { start, end: body_end, target: catch_addr, is_finally: false });
        }
        // Catch dispatch: one `CatchMatch` per clause (body forward-referenced),
        // then `Rethrow` if none matched.
        let mut sites: Vec<(Addr, Vec<ClassId>, Vec<Box<[u8]>>, Option<crate::hir::Slot>)> = Vec::new();
        for c in catches {
            let (cids, cnames) = self.resolve_catch_types(&c.types);
            let at = self.emit(Op::CatchMatch {
                types: cids.clone().into(),
                names: cnames.clone().into(),
                var: c.var,
                body: Addr::MAX,
            });
            sites.push((at, cids, cnames, c.var));
        }
        if !catches.is_empty() {
            self.emit(Op::Rethrow);
        }
        let mut catch_end_jumps = Vec::new();
        for (i, c) in catches.iter().enumerate() {
            let body_at = self.here();
            let (at, cids, cnames, var) = &sites[i];
            self.patch(*at, Op::CatchMatch {
                types: cids.clone().into(),
                names: cnames.clone().into(),
                var: *var,
                body: body_at,
            });
            self.block(&c.body)?;
            catch_end_jumps.push(self.emit(Op::Jump(Addr::MAX)));
        }
        let finally_entry = self.here();
        let after;
        if has_finally {
            // Every parked control transfer (return/break/continue in the body or
            // catches) lands at the finally entry now that it is known (EXC-2b).
            let scope = self.finally_scopes.pop().expect("finally scope");
            for site in scope.sites {
                self.patch(site, Op::Jump(finally_entry));
            }
            // Covers the body, the catch dispatch, and the catch bodies — an
            // exception anywhere before `finally_entry` runs `finally` then
            // re-propagates. Pushed after the catch region so catches win first.
            self.exc_regions.push(ExcRegion {
                start,
                end: finally_entry,
                target: finally_entry,
                is_finally: true,
            });
            // Record the protected range for `goto`-through-finally routing (EXC-2b).
            self.goto_finally_meta.push((start..finally_entry, finally_entry, outer_scope_len));
            self.block(finally)?;
            // On normal completion `EndFinally` jumps to `after` (skipping the
            // trailing `Ret`); for a parked return it pushes the value and falls
            // through to that `Ret`; for a parked break/continue it jumps to the
            // loop target.
            let endf = self.emit(Op::EndFinally { after: Addr::MAX });
            self.emit(Op::Ret);
            after = self.here();
            self.patch(endf, Op::EndFinally { after });
        } else {
            after = self.here();
        }
        let normal_target = if has_finally { finally_entry } else { after };
        self.patch(after_body, Op::Jump(normal_target));
        for j in catch_end_jumps {
            self.patch(j, Op::Jump(normal_target));
        }
        Ok(())
    }

    /// Split a catch clause's class names into those resolvable at compile time
    /// (returned as [`ClassId`]s) and those not yet declared — left as names for
    /// the VM to resolve against the live class table at run time (step 57, Phase
    /// 2), so a `catch (E)` where `E` is provided by an `eval`/`include` still
    /// matches.
    pub(super) fn resolve_catch_types(&self, names: &[Box<[u8]>]) -> (Vec<ClassId>, Vec<Box<[u8]>>) {
        let mut ids = Vec::new();
        let mut unresolved = Vec::new();
        for n in names {
            match self.resolve_class(n) {
                Some(id) => ids.push(id),
                None => unresolved.push(n.clone()),
            }
        }
        (ids, unresolved)
    }

    pub(super) fn field_path(&mut self, place: &Place) -> R<(FieldBase, Vec<FieldStep>)> {
        let base = match place.base {
            PlaceBase::Local(s) => FieldBase::Local(s),
            PlaceBase::Global(s) => FieldBase::Global(s),
            PlaceBase::This => FieldBase::This,
            // Indexed static-property targets are rewritten into a temp before
            // reaching the field-path walker (see `static_prop_rmw`).
            PlaceBase::StaticProp { .. } | PlaceBase::StaticPropDyn { .. } => {
                return Err(CompileError::Unsupported("static property field path".into()))
            }
            // A class-constant base is read-only and materialised into a temp for
            // isset/empty before any field walk, so it never reaches here.
            PlaceBase::ClassConst { .. } => {
                return Err(CompileError::Unsupported("class constant field path".into()))
            }
            // A reference through a call result (`$x = &f()->p`): the place
            // operations rewrite `Value` through `value_base_rmw` before any
            // field walk, so only `assign_ref`'s direct `field_path` reaches
            // here. Materialise the temporary into a temp slot (deliberately
            // not freed — the `MakeRef`/`BindRefTo` runs after this walk) and
            // root the walk there: the reference reaches the real object
            // through its handle.
            PlaceBase::Value(ref e) => {
                let e = (**e).clone();
                let t = self.alloc_temp();
                self.expr(&e)?;
                self.emit(Op::StoreSlot(t));
                FieldBase::Local(t)
            }
            // `$o->p = &$_SERVER`, `&$_SERVER['k']`: reference paths rooted at
            // the VM-level superglobal store (monolog's WebProcessor).
            PlaceBase::Superglobal(i) => FieldBase::Superglobal(i),
        };
        let mut steps = Vec::with_capacity(place.steps.len());
        for step in &place.steps {
            match step {
                PlaceStep::Index(k) => {
                    self.expr(k)?;
                    steps.push(FieldStep::Index);
                }
                PlaceStep::Prop(name) => steps.push(FieldStep::Prop(name.clone())),
                // `->$n` / `->{expr}`: the name expression is emitted here (in source
                // order, consumed at run time beneath the value, like an index key)
                // and the VM resolves it to a property name (step 51).
                PlaceStep::PropDyn(name) => {
                    self.expr(name)?;
                    steps.push(FieldStep::PropDyn);
                }
                // `[]` autovivifies a fresh array and descends into it (the VM's
                // `field_write` recurses through an appended child), so an
                // intermediate `$a[][] = …` is valid here, not just as the last step.
                PlaceStep::Append => steps.push(FieldStep::Append),
            }
        }
        Ok((base, steps))
    }

    /// Compile a `switch`: the subject is evaluated once into a temp, each `case`
    /// is compared with loose `==`, and on a match control jumps to that case's
    /// body. Bodies are laid out in source order so execution falls through to the
    /// next case until a `break` (the switch is one `break`/`continue` level, both
    /// landing past its end). `default` runs when no case matches, at its source
    /// position in the fall-through chain.
    pub(super) fn switch(&mut self, subject: &Expr, cases: &[Case]) -> R<()> {
        let t = self.alloc_temp();
        self.expr(subject)?;
        self.emit(Op::StoreSlot(t));
        // Dispatch: compare against each non-default case, jump to its body.
        let mut test_jumps: Vec<(usize, Addr)> = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            if let Some(test) = &case.test {
                self.emit(Op::LoadSlot(t));
                self.expr(test)?;
                self.emit(Op::Binary(BinOp::Eq));
                test_jumps.push((i, self.emit(Op::JumpIfTrue(Addr::MAX))));
            }
        }
        // No case matched -> default (if any) or past the end.
        let no_match = self.emit(Op::Jump(Addr::MAX));
        // Bodies in source order (fall-through between consecutive cases).
        self.loops.push(LoopCtx::default());
        let mut body_addrs: Vec<Addr> = Vec::with_capacity(cases.len());
        let mut default_addr: Option<Addr> = None;
        for case in cases {
            let at = self.here();
            body_addrs.push(at);
            if case.test.is_none() {
                default_addr = Some(at);
            }
            self.block(&case.body)?;
        }
        let end = self.here();
        for (i, j) in test_jumps {
            self.patch(j, Op::JumpIfTrue(body_addrs[i]));
        }
        self.patch(no_match, Op::Jump(default_addr.unwrap_or(end)));
        self.free_temp();
        // `break` and (PHP) `continue` both leave the switch.
        self.close_loop(end, end);
        Ok(())
    }

    /// Compile a `match` expression: the subject is evaluated once into a temp,
    /// each arm condition compared with strict `===`; the first match's body is
    /// evaluated as the result (no fall-through). With no matching arm and no
    /// `default`, PHP throws `UnhandledMatchError`; lacking VM exceptions, this
    /// raises a fatal (catchable-match handling is deferred). Leaves the result.
    pub(super) fn match_expr(&mut self, subject: &Expr, arms: &[MatchArm]) -> R<()> {
        let t = self.alloc_temp();
        self.expr(subject)?;
        self.emit(Op::StoreSlot(t));
        let mut to_body: Vec<(usize, Addr)> = Vec::new();
        let mut default_arm: Option<usize> = None;
        for (i, arm) in arms.iter().enumerate() {
            if arm.conditions.is_empty() {
                default_arm = Some(i);
                continue;
            }
            for cond in &arm.conditions {
                self.emit(Op::LoadSlot(t));
                self.expr(cond)?;
                self.emit(Op::Binary(BinOp::Identical));
                to_body.push((i, self.emit(Op::JumpIfTrue(Addr::MAX))));
            }
        }
        let no_match = self.emit(Op::Jump(Addr::MAX));
        // Each arm body is an expression leaving one value, then jumps to the end.
        let mut body_addrs: Vec<Addr> = vec![0; arms.len()];
        let mut to_end: Vec<Addr> = Vec::new();
        for (i, arm) in arms.iter().enumerate() {
            body_addrs[i] = self.here();
            self.expr(&arm.body)?;
            to_end.push(self.emit(Op::Jump(Addr::MAX)));
        }
        let unhandled = self.here();
        self.emit(Op::MatchError(t));
        let end = self.here();
        for (i, j) in to_body {
            self.patch(j, Op::JumpIfTrue(body_addrs[i]));
        }
        let nm_target = default_arm.map(|i| body_addrs[i]).unwrap_or(unhandled);
        self.patch(no_match, Op::Jump(nm_target));
        for j in to_end {
            self.patch(j, Op::Jump(end));
        }
        self.free_temp();
        Ok(())
    }

    /// `$target = &$source`. A step-less pair (REF-1: bare variables /
    /// `$GLOBALS['x']`) binds via a single [`Op::BindRef`]. Otherwise (REF-4:
    /// array elements) the source cell is produced with [`Op::MakeRef`] and the
    /// target bound with [`Op::BindRefTo`], evaluating the target's index
    /// expressions before the source's — the tree-walker's order. References into
    /// object properties or an appended slot fall back to the evaluator.
    pub(super) fn assign_ref(&mut self, target: &Place, source: &Place) -> R<()> {
        // `$x =& $this` (e.g. React's promise GC pattern) is legal PHP. `$this` is
        // read-only, so bind the target to a fresh reference cell holding its
        // object value rather than promoting the frame's `$this` to a reference:
        // observationally equivalent to a true alias for all realistic code (only
        // a write *through* the alias back into `$this` would differ), while
        // leaving `$this`'s representation — read by every method body — untouched.
        if matches!(source.base, PlaceBase::This) && source.steps.is_empty() {
            let (tbase, tsteps) = self.field_path(target)?; // target index keys first…
            self.emit(Op::This); // …then $this's value on top for the bind
            self.emit(Op::BindRefTo { base: tbase, steps: tsteps.into() });
            return Ok(());
        }
        // A static-property side (either one) roots at a temp via the RMW
        // wrapper: the Ref cell minted inside the temp array is Rc-shared, so
        // it survives the write-back and the alias stays live in both the
        // bound variable and the stored static property
        // (`$exists = &self::$existsCache[$k]`, ClassExistenceResource).
        if let PlaceBase::StaticProp { class, name } = &target.base {
            let (class, name) = (class.clone(), SpName::Lit(name.clone()));
            let src = source.clone();
            return self.static_prop_rmw(&class, &name, &target.steps, false, move |s, local| {
                s.assign_ref(local, &src)
            });
        }
        if let PlaceBase::StaticProp { class, name } = &source.base {
            let (class, name) = (class.clone(), name.clone());
            // Bare `&Class::$sp` with a compile-time class: bind straight to
            // the property's live storage cell (true two-way alias).
            if source.steps.is_empty() && !self.is_runtime_class(&class) {
                let cls_target = self.resolve_target(&class)?.0;
                let (tbase, tsteps) = self.field_path(target)?;
                self.emit(Op::StaticPropRef { target: cls_target, name: name.clone() });
                self.emit(Op::BindRefTo { base: tbase, steps: tsteps.into() });
                return Ok(());
            }
            let tgt = target.clone();
            let name = SpName::Lit(name);
            return self.static_prop_rmw(&class, &name, &source.steps, false, move |s, local| {
                s.assign_ref(&tgt, local)
            });
        }
        if target.steps.is_empty() && source.steps.is_empty() {
            let t = dim_base(target)?;
            let s = dim_base(source)?;
            self.emit(Op::BindRef { target: t, source: s });
            return Ok(());
        }
        let (tbase, tsteps) = self.field_path(target)?; // pushes target keys…
        let (sbase, ssteps) = self.field_path(source)?; // …then source keys
        self.emit(Op::MakeRef { base: sbase, steps: ssteps.into() });
        self.emit(Op::BindRefTo { base: tbase, steps: tsteps.into() });
        Ok(())
    }

    pub(super) fn assign_ref_call(&mut self, target: &Place, call: &Expr) -> R<()> {
        // `$t = &$obj->m(...)`: a method's by-reference-ness is only known at run
        // time (dynamic dispatch), so emit the call and bind through the
        // run-time-checked `BindRefToChecked` (alias a returned reference, else
        // notice + copy). Target index keys are pushed first so the call result
        // lands on top for the bind.
        if let ExprKind::MethodCall { object, method, args, named, nullsafe } = &call.kind {
            if *nullsafe {
                return Err(CompileError::Unsupported(
                    "reference bind from a nullsafe method call".into(),
                ));
            }
            let (base, steps) = self.field_path(target)?;
            self.expr(object)?;
            let recv_class = match object.kind {
                ExprKind::This => self.cur_class,
                _ => None,
            };
            self.emit_method_call(method, args, named, recv_class)?;
            self.emit(Op::BindRefToChecked { base, steps: steps.into() });
            return Ok(());
        }
        // `$t = &$f(...)` / `$t = &(expr)(...)`: like a method call, the callee's
        // by-reference-ness is only known at run time (closure value / callable
        // string), so emit the dynamic call and bind through `BindRefToChecked`.
        if let ExprKind::CallDynamic { callee, args } = &call.kind {
            let (base, steps) = self.field_path(target)?;
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
            self.emit(Op::BindRefToChecked { base, steps: steps.into() });
            return Ok(());
        }
        let ExprKind::Call { name, args, named, .. } = &call.kind else {
            return Err(CompileError::Unsupported("reference assignment from a non-call".into()));
        };
        if !named.is_empty() {
            return Err(CompileError::Unsupported("reference call with named arguments".into()));
        }
        let Some(idx) = self.ctx.funcs.iter().position(|f| ascii_eq_ignore_case(&f.name, name)) else {
            return Err(CompileError::Unsupported(
                "reference assignment from a builtin / undefined call".into(),
            ));
        };
        let callee = &self.ctx.funcs[idx];
        if callee.params.iter().any(|p| p.variadic) {
            return Err(CompileError::Unsupported("reference call to a variadic function".into()));
        }
        if args.len() != callee.params.len() {
            return Err(CompileError::Unsupported("reference call arity mismatch".into()));
        }
        let by_ref: Vec<bool> = callee.params.iter().map(|p| p.by_ref).collect();
        let callee_by_ref = callee.by_ref;
        let pnames: Vec<Box<[u8]>> =
            callee.params.iter().map(|p| callee.slots[p.slot as usize].clone()).collect();
        let (base, steps) = self.field_path(target)?; // target index keys first…
        self.push_call_args(args, &by_ref, name, &pnames)?; // …then the call args…
        self.emit(Op::Call { func: idx as u32, argc: args.len() as u32 }); // …leaving the raw ref on top
        // Aliasing a non-reference-returning function copies the value and raises a
        // notice (D-13.5). A by-ref callee that returned a non-place already raised
        // its own "returned by reference" notice from inside `f`, so suppress here.
        if !callee_by_ref {
            let k = self.konst(Const::Str(
                b"Only variables should be assigned by reference"[..].into(),
            ));
            self.emit(Op::EmitNotice(k));
        }
        self.emit(Op::BindRefTo { base, steps: steps.into() });
        Ok(())
    }

    /// Compile an array-element write `$a[…][k] = rhs` / `$a[…][] = rhs`, rooted
    /// at a local (or `$GLOBALS`) slot, at any nesting depth — or a single-step
    /// object-property write `$o->p = rhs` / `$this->p = rhs` (OOP-1). Mixed
    /// property+index chains (`$o->a[$k] = …`) remain out of slice.
    pub(super) fn assign_place(&mut self, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.assign_place(p, rhs));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_place(p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.expr(rhs)?;
            self.emit(Op::PropSet { name });
            return Ok(());
        }
        // An object-property step, or an *intermediate* `[]` append (`$a[][] = …`),
        // routes through the general field walker — `Op::AssignPath` only models a
        // run of index keys plus an optional trailing append.
        if place_has_prop(place) || place_has_intermediate_append(place) {
            let (base, steps) = self.field_path(place)?;
            self.expr(rhs)?;
            self.emit(Op::FieldAssign { base, steps: steps.into() });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x'] = rhs`: a direct global write (no array steps).
                self.expr(rhs)?;
                self.emit(Op::Dup);
                self.emit(Op::StoreGlobal(s));
                return Ok(());
            }
        }
        if let PlaceBase::Superglobal(i) = place.base {
            if place.steps.is_empty() {
                // `$_SERVER = rhs`: a direct superglobal write (no array steps).
                self.expr(rhs)?;
                self.emit(Op::Dup);
                self.emit(Op::StoreSuperglobal(i));
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if nkeys == 0 && !append {
            return Err(CompileError::Unsupported("array write with no steps".into()));
        }
        self.expr(rhs)?;
        self.emit(Op::AssignPath { base, nkeys, append });
        Ok(())
    }

    /// Compile a single `unset(place)`: a `->prop` routes to `PropUnset`, a mixed
    /// object/array path to `FieldUnset`, a plain array element to `UnsetPath`, and
    /// an indexed static property is read-modify-written through a temp.
    pub(super) fn unset_place(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.unset_place(p));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_rmw(&class, &name, &place.steps, false, |s, p| {
                s.unset_place(p)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropUnset { name });
        } else if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldUnset { base, steps: steps.into() });
        } else {
            let base = dim_base(place)?;
            let nkeys = self.test_path_steps(place)?;
            self.emit(Op::UnsetPath { base, nkeys });
        }
        Ok(())
    }

    /// Read-modify-write wrapper for an indexed static-property target
    /// (`self::$arr[k] = v`, `unset(self::$arr[k])`). The property value is loaded
    /// into a temp, `core` runs over a `Local`-rooted place on that temp, then the
    /// temp is written back into the static property — value-correct for PHP
    /// arrays (copy-on-write). When `leaves_value` is set, `core` leaves the
    /// expression's result on the stack and it is preserved across the write-back.
    /// The property name may itself be dynamic ([`SpName::Dyn`]); it is then
    /// evaluated exactly once into its own temp.
    /// Dynamic class references (`$cls::$arr[k]`) are out of scope.
    pub(super) fn static_prop_rmw(
        &mut self,
        class: &ClassRef,
        name: &SpName,
        steps: &[PlaceStep],
        leaves_value: bool,
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        // A class only known at run time (an autoloaded name, `$cls`) reads and
        // writes through the *Dynamic ops — the class value is pushed for each.
        // A literal name on a compile-time class targets the resolved class
        // directly; a dynamic name goes through the *DynName ops.
        let runtime = self.is_runtime_class(class);
        let target = match name {
            SpName::Lit(_) if !runtime => Some(self.resolve_target(class)?.0),
            _ => None,
        };
        let name_slot = self.sp_name_slot(name)?;
        let t = self.alloc_temp();
        self.sp_get(class, target, name, name_slot)?;
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        if leaves_value {
            core(self, &local)?; // [result]
            let t2 = self.alloc_temp();
            self.emit(Op::StoreSlot(t2)); // []
            self.emit(Op::LoadSlot(t));
            self.sp_set(class, target, name, name_slot)?; // [arr]
            self.emit(Op::Pop);
            self.emit(Op::LoadSlot(t2)); // [result]
            self.free_temp(); // t2
        } else {
            core(self, &local)?; // []
            self.emit(Op::LoadSlot(t));
            self.sp_set(class, target, name, name_slot)?; // [arr]
            self.emit(Op::Pop);
        }
        self.free_temp(); // t
        if name_slot.is_some() {
            self.free_temp(); // dynamic-name temp
        }
        Ok(())
    }

    /// `return self::$arr[$k];` in a by-ref function (`function &f()`):
    /// `field_path` has no static base, so mint the reference through the rmw
    /// wrapper — the ref cell is created inside the temp copy and the
    /// write-back installs that copy (ref included) as the live property
    /// value, the same separation Zend performs (WP_Test_Stream's
    /// `&get_directory_ref`). A bare `return self::$prop;` aliases the live
    /// storage cell directly. Returns `false` when the place is not
    /// static-property-rooted, so the caller falls through to the plain path.
    pub(super) fn return_ref_static_prop(&mut self, place: &Place) -> R<bool> {
        let Some((class, name)) = static_place_parts(&place.base) else {
            return Ok(false);
        };
        if place.steps.is_empty() && !self.is_runtime_class(&class) {
            if let SpName::Lit(n) = &name {
                let target = self.resolve_target(&class)?.0;
                self.emit(Op::StaticPropRef { target, name: n.clone() });
                self.emit(Op::Ret);
                return Ok(true);
            }
        }
        self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
            let (base, steps) = s.field_path(p)?;
            s.emit(Op::MakeRef { base, steps: steps.into() });
            Ok(())
        })?;
        self.emit(Op::Ret);
        Ok(true)
    }

    /// Evaluate a dynamic static-property name exactly once into a temp; a
    /// literal name needs no slot.
    fn sp_name_slot(&mut self, name: &SpName) -> R<Option<u32>> {
        match name {
            SpName::Lit(_) => Ok(None),
            SpName::Dyn(e) => {
                let tn = self.alloc_temp();
                self.expr(e)?;
                self.emit(Op::StoreSlot(tn));
                Ok(Some(tn))
            }
        }
    }

    /// Emit the read of `class::$name` for the rmw/read wrappers, dispatching
    /// on literal vs dynamic name and compile-time vs runtime class.
    fn sp_get(
        &mut self,
        class: &ClassRef,
        target: Option<ClassTarget>,
        name: &SpName,
        name_slot: Option<u32>,
    ) -> R<()> {
        match (name, target) {
            (SpName::Lit(nm), Some(t)) => {
                self.emit(Op::StaticPropGet { target: t, name: nm.clone() });
            }
            (SpName::Lit(nm), None) => {
                self.push_class_value(class)?;
                self.emit(Op::StaticPropGetDynamic { name: nm.clone() });
            }
            (SpName::Dyn(_), _) => {
                self.push_class_value(class)?;
                self.emit(Op::LoadSlot(name_slot.expect("dynamic name slot")));
                self.emit(Op::StaticPropGetDynName);
            }
        }
        Ok(())
    }

    /// The write twin of [`Self::sp_get`]: consumes the value beneath the
    /// class/name operands and leaves the assigned value on the stack.
    fn sp_set(
        &mut self,
        class: &ClassRef,
        target: Option<ClassTarget>,
        name: &SpName,
        name_slot: Option<u32>,
    ) -> R<()> {
        match (name, target) {
            (SpName::Lit(nm), Some(t)) => {
                self.emit(Op::StaticPropSet { target: t, name: nm.clone() });
            }
            (SpName::Lit(nm), None) => {
                self.push_class_value(class)?;
                self.emit(Op::StaticPropSetDynamic { name: nm.clone() });
            }
            (SpName::Dyn(_), _) => {
                self.push_class_value(class)?;
                self.emit(Op::LoadSlot(name_slot.expect("dynamic name slot")));
                self.emit(Op::StaticPropSetDynName);
            }
        }
        Ok(())
    }

    /// Materialise an indexed static property into a temp for a *read-only*
    /// operation (`isset`/`empty`): load the value, run `core` over a
    /// `Local`-rooted place on the temp, then free the temp. No write-back — the
    /// property is not modified, so this also avoids a visibility-checked
    /// `StaticPropSet` on an out-of-scope read.
    pub(super) fn static_prop_read(
        &mut self,
        class: &ClassRef,
        name: &SpName,
        steps: &[PlaceStep],
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        let runtime = self.is_runtime_class(class);
        let target = match name {
            SpName::Lit(_) if !runtime => Some(self.resolve_target(class)?.0),
            _ => None,
        };
        let name_slot = self.sp_name_slot(name)?;
        let t = self.alloc_temp();
        self.sp_get(class, target, name, name_slot)?;
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        core(self, &local)?;
        self.free_temp(); // t
        if name_slot.is_some() {
            self.free_temp(); // dynamic-name temp
        }
        Ok(())
    }

    /// Materialise a class constant into a temp for a read-only `isset`/`empty`
    /// test on an index into it (`isset(self::TABLE[$k])`): evaluate the constant,
    /// run `core` over a `Local`-rooted place carrying the index steps, then free
    /// the temp. The constant is a value (never written), mirroring
    /// [`Self::static_prop_read`] but reading via [`Self::class_const`].
    pub(super) fn class_const_read(
        &mut self,
        class: &ClassRef,
        name: &[u8],
        steps: &[PlaceStep],
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        let t = self.alloc_temp();
        self.class_const(class, name)?;
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        core(self, &local)?;
        self.free_temp();
        Ok(())
    }

    /// Materialise a call/`new`/`clone` result into a temp and run `core` over a
    /// `Local`-rooted place on it (`f()->prop = v`, `f()['k'] ??= v`, …). No
    /// write-back: PHP treats the result as a temporary — a property write
    /// reaches the real object through its handle, while a pure index write
    /// lands in the discarded temp (`f()['k'] = 2` is legal and leaves `f()`'s
    /// value untouched), which is exactly what the temp gives us.
    pub(super) fn value_base_rmw(
        &mut self,
        value: &Expr,
        steps: &[PlaceStep],
        core: impl FnOnce(&mut Self, &Place) -> R<()>,
    ) -> R<()> {
        let t = self.alloc_temp();
        self.expr(value)?;
        self.emit(Op::StoreSlot(t));
        let local = Place {
            base: PlaceBase::Local(t),
            steps: steps.to_vec(),
        };
        core(self, &local)?;
        self.free_temp();
        Ok(())
    }

    /// Compile `$o->p ??= rhs` on a single property (magic-aware). Read via
    /// `__isset`; if already set, the existing value (`__get`) is the result and
    /// no write happens; if unset, assign `rhs` (`__set`) and yield it. Each magic
    /// op leaves its result for the next, so the composition works for both magic
    /// and declared properties.
    pub(super) fn assign_coalesce_place(&mut self, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.assign_coalesce_place(p, rhs));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_coalesce_place(p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            // stack: [obj]
            self.emit(Op::Dup); // [obj, obj]
            // Fetch gate (BP_VAR_IS): `__get` without `__isset` answers set —
            // a non-null `__get` value is the result and nothing is written; a
            // null one still assigns (oracle-pinned).
            self.emit(Op::PropIssetFetchGate { name: name.clone() }); // [obj, isset]
            let to_set = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → set; [obj]
            self.emit(Op::Dup); // [obj, obj]
            self.emit(Op::PropGet { name: name.clone() }); // set: existing value → [obj, value]
            let to_nn = self.emit(Op::JumpIfNotNull(Addr::MAX)); // null → popped; [obj]
            let set_at = self.here();
            self.patch(to_set, Op::JumpIfFalse(set_at));
            self.expr(rhs)?; // [obj, rhs]
            self.emit(Op::PropSet { name }); // [value]
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let nn_at = self.here();
            self.patch(to_nn, Op::JumpIfNotNull(nn_at)); // [obj, value]
            self.emit(Op::Swap); // [value, obj]
            self.emit(Op::Pop); // [value]
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            return Ok(());
        }
        // `$this->cache[$k] ??= rhs` — a prop-bearing path: evaluate every
        // dynamic key ONCE into temps, probe with FieldIsset, then read the
        // existing leaf or FieldAssign the rhs (ORM's lazy identity-map
        // caches: `$this->repositories[$name] ??= …`).
        if place_has_prop(place)
            && !matches!(
                place.base,
                PlaceBase::Value(_) | PlaceBase::StaticProp { .. } | PlaceBase::ClassConst { .. }
            )
            && place.steps.iter().all(|s| !matches!(s, PlaceStep::Append))
        {
            let base = match place.base {
                PlaceBase::Local(s) => FieldBase::Local(s),
                PlaceBase::Global(s) => FieldBase::Global(s),
                PlaceBase::This => FieldBase::This,
                PlaceBase::Superglobal(i) => FieldBase::Superglobal(i),
                _ => unreachable!("filtered above"),
            };
            let mut steps: Vec<FieldStep> = Vec::with_capacity(place.steps.len());
            let mut temps: Vec<Option<u32>> = Vec::with_capacity(place.steps.len());
            for step in &place.steps {
                match step {
                    PlaceStep::Index(k) => {
                        let t = self.alloc_temp();
                        self.expr(k)?;
                        self.emit(Op::StoreSlot(t));
                        steps.push(FieldStep::Index);
                        temps.push(Some(t));
                    }
                    PlaceStep::PropDyn(name) => {
                        let t = self.alloc_temp();
                        self.expr(name)?;
                        self.emit(Op::StoreSlot(t));
                        steps.push(FieldStep::PropDyn);
                        temps.push(Some(t));
                    }
                    PlaceStep::Prop(name) => {
                        steps.push(FieldStep::Prop(name.clone()));
                        temps.push(None);
                    }
                    PlaceStep::Append => unreachable!("filtered above"),
                }
            }
            let steps: Box<[FieldStep]> = steps.into();
            for t in temps.iter().flatten() {
                self.emit(Op::LoadSlot(*t));
            }
            self.emit(Op::FieldIsset { base, steps: steps.clone() });
            let to_assign = self.emit(Op::JumpIfFalse(Addr::MAX));
            // Set: read the existing leaf (base value, then walk the steps).
            match base {
                FieldBase::Local(s) => self.emit(Op::LoadSlot(s)),
                FieldBase::Global(s) => self.emit(Op::LoadGlobal(s)),
                FieldBase::This => self.emit(Op::This),
                FieldBase::Superglobal(i) => self.emit(Op::LoadSuperglobal(i)),
            };
            let mut ti = temps.iter();
            for step in steps.iter() {
                let t = ti.next().expect("temps parallel to steps");
                match step {
                    FieldStep::Prop(n) => {
                        self.emit(Op::PropGet { name: n.clone() });
                    }
                    FieldStep::PropDyn => {
                        self.emit(Op::LoadSlot(t.expect("dyn step has temp")));
                        self.emit(Op::PropGetDynamic);
                    }
                    FieldStep::Index => {
                        self.emit(Op::LoadSlot(t.expect("index step has temp")));
                        self.emit(Op::FetchDim);
                    }
                    FieldStep::Append => unreachable!("filtered above"),
                }
            }
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let assign_at = self.here();
            self.patch(to_assign, Op::JumpIfFalse(assign_at));
            for t in temps.iter().flatten() {
                self.emit(Op::LoadSlot(*t));
            }
            self.expr(rhs)?;
            self.emit(Op::FieldAssign { base, steps });
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            for _ in temps.iter().flatten() {
                self.free_temp();
            }
            return Ok(());
        }
        // `$a[k1][k2]… ??= rhs` on an all-index path rooted at a local /
        // `$GLOBALS` slot: evaluate each key once (into temps), then assign
        // only if the element is unset, yielding the existing or newly-stored
        // value (symfony's DeepClone leans on the 2-step static-prop form,
        // which static_prop_rmw funnels here).
        if !place.steps.is_empty()
            && place.steps.iter().all(|s| matches!(s, PlaceStep::Index(_)))
        {
            let n = place.steps.len() as u32;
            let base = dim_base(place)?;
            let mut temps = Vec::with_capacity(place.steps.len());
            for st in &place.steps {
                let PlaceStep::Index(k) = st else { unreachable!("all-index checked") };
                let t_key = self.alloc_temp();
                self.expr(k)?; // […, key]
                self.emit(Op::Dup);
                self.emit(Op::StoreSlot(t_key));
                temps.push(t_key);
            }
            // stack: [k1 … kn]
            self.emit(Op::IssetPath { base, nkeys: n }); // [bool]
            let to_assign = self.emit(Op::JumpIfFalse(Addr::MAX));
            // Set: yield the existing element value.
            match base {
                DimBase::Local(s) => self.emit(Op::LoadSlot(s)),
                DimBase::Global(s) => self.emit(Op::LoadGlobal(s)),
                DimBase::Superglobal(i) => self.emit(Op::LoadSuperglobal(i)),
            };
            for t in &temps {
                self.emit(Op::LoadSlot(*t)); // [baseval, key]
                self.emit(Op::FetchDim); // [value]
            }
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let assign_at = self.here();
            self.patch(to_assign, Op::JumpIfFalse(assign_at));
            for t in &temps {
                self.emit(Op::LoadSlot(*t)); // [k1 … kn]
            }
            self.expr(rhs)?; // [k1 … kn, rhs]
            self.emit(Op::AssignPath { base, nkeys: n, append: false }); // [value]
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            for _ in &temps {
                self.free_temp();
            }
            return Ok(());
        }
        Err(CompileError::Unsupported("`??=` on this place".into()))
    }

    /// Compile a compound element write `$a[…][k] op= rhs`.
    pub(super) fn assign_op_place(&mut self, op: crate::hir::BinOp, place: &Place, rhs: &Expr) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.assign_op_place(op, p, rhs));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.assign_op_place(op, p, rhs)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            // `$o->p op= rhs` as read-modify-write so a magic property routes
            // through `__get` then `__set` (each op leaves its result for the
            // next): [obj] → Dup → PropGet → rhs → Binary → PropSet → [result].
            self.emit(Op::Dup);
            self.emit(Op::PropGet { name: name.clone() });
            self.expr(rhs)?;
            self.emit(Op::Binary(op));
            self.emit(Op::PropSet { name });
            return Ok(());
        }
        if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.expr(rhs)?;
            self.emit(Op::FieldAssignOp { base, steps: steps.into(), op });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x'] op= rhs`.
                self.emit(Op::LoadGlobal(s));
                self.expr(rhs)?;
                self.emit(Op::Binary(op));
                self.emit(Op::Dup);
                self.emit(Op::StoreGlobal(s));
                return Ok(());
            }
        }
        if let PlaceBase::Superglobal(i) = place.base {
            if place.steps.is_empty() {
                // `$_SERVER op= rhs`.
                self.emit(Op::LoadSuperglobal(i));
                self.expr(rhs)?;
                self.emit(Op::Binary(op));
                self.emit(Op::Dup);
                self.emit(Op::StoreSuperglobal(i));
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append || nkeys == 0 {
            return Err(CompileError::Unsupported("`[]` has no value for reading".into()));
        }
        self.expr(rhs)?;
        self.emit(Op::AssignOpPath { base, nkeys, op });
        Ok(())
    }

    /// Compile `++`/`--` on an array element `$a[…][k]`.
    pub(super) fn incdec_place(&mut self, place: &Place, inc: bool, pre: bool) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.incdec_place(p, inc, pre));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_rmw(&class, &name, &place.steps, true, |s, p| {
                s.incdec_place(p, inc, pre)
            });
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropIncDec { name, inc, pre });
            return Ok(());
        }
        if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldIncDec { base, steps: steps.into(), inc, pre });
            return Ok(());
        }
        if let PlaceBase::Global(s) = place.base {
            if place.steps.is_empty() {
                // `$GLOBALS['x']++` / `--$GLOBALS['x']`.
                self.emit(Op::IncDecGlobal { slot: s, inc, pre });
                return Ok(());
            }
        }
        if let PlaceBase::Superglobal(i) = place.base {
            if place.steps.is_empty() {
                // `$_SERVER++` / `--$_SERVER` (degenerate, but mirror the global form).
                self.emit(Op::IncDecSuperglobal { idx: i, inc, pre });
                return Ok(());
            }
        }
        let base = dim_base(place)?;
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append || nkeys == 0 {
            return Err(CompileError::Unsupported("`[]` has no value for reading".into()));
        }
        self.emit(Op::IncDecPath { base, nkeys, inc, pre });
        Ok(())
    }

    /// Compile `isset($p0, $p1, …)` to a boolean: each place is tested in turn
    /// and the result short-circuits to `false` on the first absent one (so a
    /// later place's index expressions aren't evaluated), mirroring PHP.
    pub(super) fn isset(&mut self, places: &[Place]) -> R<()> {
        let last = places.len() - 1;
        let mut to_false = Vec::new();
        for (i, place) in places.iter().enumerate() {
            self.isset_one(place)?;
            if i != last {
                // [bi]: if false, jump to the shared false-result; else discard.
                to_false.push(self.emit(Op::JumpIfFalse(Addr::MAX)));
            }
        }
        if to_false.is_empty() {
            return Ok(()); // single place: its IssetPath bool is the result
        }
        let to_end = self.emit(Op::Jump(Addr::MAX));
        let false_at = self.here();
        let f = self.konst(Const::Bool(false));
        self.emit(Op::PushConst(f));
        let end = self.here();
        self.patch(to_end, Op::Jump(end));
        for j in to_false {
            self.patch(j, Op::JumpIfFalse(false_at));
        }
        Ok(())
    }

    /// Push a single place's `isset` boolean: a `->prop` via `PropIsset`, a mixed
    /// object/array path via `FieldIsset`, a plain array element via `IssetPath`,
    /// and an indexed static property via a read-only temp.
    pub(super) fn isset_one(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.isset_one(p));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_read(&class, &name, &place.steps, |s, p| s.isset_one(p));
        }
        if let PlaceBase::ClassConst { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.class_const_read(&class, &name, &place.steps, |s, p| s.isset_one(p));
        }
        if let Some(name) = self.prop_place(place)? {
            self.emit(Op::PropIsset { name });
        } else if self.prop_place_dyn(place)? {
            self.emit(Op::PropIssetDyn);
        } else if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldIsset { base, steps: steps.into() });
        } else {
            let base = dim_base(place)?;
            let nkeys = self.test_path_steps(place)?;
            self.emit(Op::IssetPath { base, nkeys });
        }
        Ok(())
    }

    /// Compile `empty($place)`. A single property is `__isset`-then-silent-`__get`
    /// (`empty` is `!isset || !truthy(value)`), so an unset magic property never
    /// warns or calls `__get`; other places use the array `EmptyPath`.
    pub(super) fn empty(&mut self, place: &Place) -> R<()> {
        if let PlaceBase::Value(e) = &place.base {
            let e = (**e).clone();
            return self.value_base_rmw(&e, &place.steps, |s, p| s.empty(p));
        }
        if let PlaceBase::ClassConst { class, name } = &place.base {
            let (class, name) = (class.clone(), name.clone());
            return self.class_const_read(&class, &name, &place.steps, |s, p| s.empty(p));
        }
        if let Some((class, name)) = static_place_parts(&place.base) {
            return self.static_prop_read(&class, &name, &place.steps, |s, p| s.empty(p));
        }
        if let Some(name) = self.prop_place(place)? {
            // stack: [obj]
            self.emit(Op::Dup); // [obj, obj]
            self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
            let to_true = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → empty=true; [obj]
            self.emit(Op::PropGetSilent { name }); // [value]
            self.emit(Op::Unary(crate::hir::UnOp::Not)); // [empty = !truthy(value)]
            let to_end = self.emit(Op::Jump(Addr::MAX));
            let true_at = self.here();
            self.patch(to_true, Op::JumpIfFalse(true_at));
            self.emit(Op::Pop); // drop the kept object
            let t = self.konst(Const::Bool(true));
            self.emit(Op::PushConst(t)); // [true]
            let end = self.here();
            self.patch(to_end, Op::Jump(end));
            return Ok(());
        }
        if place_has_prop(place) {
            let (base, steps) = self.field_path(place)?;
            self.emit(Op::FieldEmpty { base, steps: steps.into() });
            return Ok(());
        }
        let base = dim_base(place)?;
        let nkeys = self.test_path_steps(place)?;
        self.emit(Op::EmptyPath { base, nkeys });
        Ok(())
    }

    /// Like [`Self::push_index_steps`] but for a read-only test target
    /// (`isset` / `empty` / `unset`): pushes the index values and returns the
    /// key count. `[]` and `->prop` steps are not valid here.
    pub(super) fn test_path_steps(&mut self, place: &Place) -> R<u32> {
        let (nkeys, append) = self.push_index_steps(&place.steps)?;
        if append {
            return Err(CompileError::Unsupported("`[]` is not a readable place".into()));
        }
        Ok(nkeys)
    }

    /// Push each `Index` step's value (source order) and report `(nkeys, append)`:
    /// how many index values were pushed, and whether the final step is `[]`.
    /// A `Prop` step or a non-final `Append` is out of slice.
    pub(super) fn push_index_steps(&mut self, steps: &[PlaceStep]) -> R<(u32, bool)> {
        let mut nkeys = 0u32;
        let mut append = false;
        let last = steps.len().saturating_sub(1);
        for (i, step) in steps.iter().enumerate() {
            match step {
                PlaceStep::Index(k) => {
                    self.expr(k)?;
                    nkeys += 1;
                }
                PlaceStep::Append if i == last => append = true,
                PlaceStep::Append => {
                    return Err(CompileError::Unsupported("`[]` is only valid as the last step".into()))
                }
                PlaceStep::Prop(_) | PlaceStep::PropDyn(_) => {
                    return Err(CompileError::Unsupported("object property step".into()))
                }
            }
        }
        Ok((nkeys, append))
    }
}
