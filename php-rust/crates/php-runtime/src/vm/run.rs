//! The bounded VM dispatch loop (run_loop) — the hot Op match. Split from
//! vm/mod.rs verbatim (structural move only; hot-path unchanged).

use super::*;

impl<'m> super::Vm<'m> {
    /// The bounded dispatch loop: runs until the frame at `baseline` returns
    /// ([`RunExit::Returned`]) or a generator at `baseline` suspends at a `yield`
    /// ([`RunExit::Yielded`]), or an op raises a `PhpError` (which the caller
    /// routes through [`Self::unwind`]). Frames above `baseline` (ordinary
    /// callees) return normally to their callers within this same loop.
    pub(super) fn run_loop(&mut self, baseline: usize) -> Result<RunExit, PhpError> {
        loop {
            // Defensive call-stack depth guard (mirrors `eval::guard_call_depth`):
            // surface a catchable PHP `Error` before runaway recursion exhausts
            // memory (pure PHP recursion is iterative here, growing `frames`) or
            // overflows the native stack (callback-nested `run_loop`s).
            if self.frames.len() > MAX_CALL_DEPTH {
                return Err(PhpError::Error(format!(
                    "Maximum call stack depth of {MAX_CALL_DEPTH} exceeded"
                )));
            }
            let top = self.frames.len() - 1;
            let ip = self.frames[top].ip;
            let op = self.frames[top].func.ops[ip].clone();
            // Default fall-through advance. Jumps overwrite `ip`; `Call` advances
            // the *caller* before pushing the callee; `Ret` discards this frame.
            self.frames[top].ip = ip + 1;

            match op {
                Op::PushConst(i) => {
                    let v = self.frames[top].func.consts[i as usize].to_zval();
                    self.frames[top].stack.push(v);
                }
                Op::ConstFetch { name, fallback } => {
                    // A user constant (B3): engine constants were folded at lowering.
                    // An unqualified constant inside a namespace is looked up as
                    // `CURNS\NAME` first, then the global `NAME` (step 50); an
                    // "Undefined constant" error reports the namespaced name.
                    let v = self
                        .constants
                        .get(&name[..])
                        .or_else(|| fallback.as_ref().and_then(|g| self.constants.get(&g[..])))
                        .cloned()
                        .ok_or_else(|| {
                            PhpError::Error(format!(
                                "Undefined constant \"{}\"",
                                String::from_utf8_lossy(&name)
                            ))
                        })?;
                    self.frames[top].stack.push(v);
                }
                Op::DefineConst { name } => {
                    // `const NAME = value;` — register the constant, warning and
                    // keeping the first value on redefinition (like `define()`).
                    let value = self.frames[top].stack.pop().expect("DefineConst value");
                    if self.constant_known(&name) {
                        self.diags.push(Diag::Warning(format!(
                            "Constant {} already defined, this will be an error in PHP 9",
                            String::from_utf8_lossy(&name)
                        )));
                    } else {
                        self.constants.insert(name.to_vec(), value);
                    }
                }
                Op::Pop => {
                    if let Some(v) = self.frames[top].stack.pop() {
                        self.gc_note(&v);
                    }
                }
                Op::Dup => {
                    let v = self.frames[top].stack.last().expect("Dup on empty stack").clone();
                    self.frames[top].stack.push(v);
                }
                Op::Swap => {
                    let st = &mut self.frames[top].stack;
                    let n = st.len();
                    st.swap(n - 1, n - 2);
                }
                Op::LoadSlot(s) => {
                    // An unset local reads as NULL (silent — used for compiler
                    // temporaries and PHP's warning-free contexts). A reference
                    // slot is followed. Source-level `$x` reads use `LoadVar`.
                    let v = read_slot(&self.frames[top].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::LoadVar { slot, name } => {
                    // A source-level `$x` read: an `Undef` slot raises the PHP 8
                    // "Undefined variable" warning (queued; flushed at the next
                    // emit point with the reading op's line) and yields NULL.
                    if matches!(self.frames[top].slots[slot as usize], Zval::Undef) {
                        if let crate::bytecode::Const::Str(b) =
                            &self.frames[top].func.consts[name as usize]
                        {
                            let msg = format!("Undefined variable ${}", String::from_utf8_lossy(b));
                            self.diags.push(Diag::Warning(msg));
                        }
                    }
                    let v = read_slot(&self.frames[top].slots[slot as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSlot(s) => {
                    // A slot aliasing a *typed property* (a registered typed
                    // reference) coerces/checks the value first (PHP's typed-ref
                    // assignment: "Cannot assign string to reference held by
                    // property C::$p of type int").
                    if !self.typed_refs.is_empty() {
                        if let Zval::Ref(cell) = &self.frames[top].slots[s as usize] {
                            let cell = Rc::clone(cell);
                            let strict = self.frames[top].module.strict;
                            let v = self.frames[top].stack.pop().expect("StoreSlot value");
                            let v = self.typed_ref_assign(&cell, v, strict)?;
                            self.frames[top].stack.push(v);
                        }
                    }
                    let v = self.frames[top].stack.pop().expect("StoreSlot on empty stack");
                    let old = store_slot(&mut self.frames[top].slots[s as usize], v);
                    self.gc_note(&old);
                }
                Op::LoadVarDyn => {
                    // `$$x` read: resolve the runtime name in the current frame.
                    let nv = self.frames[top].stack.pop().expect("LoadVarDyn name");
                    let name = convert::to_zstr_cast(&nv, &mut self.diags).as_bytes().to_vec();
                    let v = self.var_dyn_read(top, &name);
                    self.frames[top].stack.push(v);
                }
                Op::StoreVarDyn => {
                    // `$$x = rhs`: resolve/create by runtime name; the rhs stays
                    // on the stack as the assignment's value.
                    let rhs = self.frames[top].stack.pop().expect("StoreVarDyn value");
                    let nv = self.frames[top].stack.pop().expect("StoreVarDyn name");
                    let name = convert::to_zstr_cast(&nv, &mut self.diags).as_bytes().to_vec();
                    self.var_dyn_write(top, &name, rhs.clone())?;
                    self.frames[top].stack.push(rhs);
                }
                Op::StaticGuard { id, skip } => {
                    // First execution of this `static` declaration falls through to
                    // run the initialiser; every later one skips to the alias. A
                    // closure keys its own per-instance storage (fresh statics per
                    // closure object); everything else the program-global `statics`.
                    let exists = match self.frames[top].closure_id {
                        Some(cid) => self.closure_statics.contains_key(&(cid, id)),
                        None => self.statics[id as usize].is_some(),
                    };
                    if exists {
                        self.frames[top].ip = skip as usize;
                    }
                }
                Op::StaticStore { id } => {
                    let v = self.frames[top].stack.pop().expect("StaticStore on empty stack");
                    let cell = Rc::new(RefCell::new(v));
                    let old = match self.frames[top].closure_id {
                        Some(cid) => self.closure_statics.insert((cid, id), cell),
                        None => self.statics[id as usize].replace(cell),
                    };
                    if let Some(cell) = old {
                        if Rc::strong_count(&cell) == 1 {
                            let inner = cell.borrow();
                            self.gc_note(&inner);
                        }
                    }
                }
                Op::StaticAlias { slot, id } => {
                    // Alias the local slot to the persistent cell: reads/writes of
                    // the variable now go through it (the slot holds a `Zval::Ref`,
                    // followed by `read_slot`/`store_slot` like any reference).
                    let cell = match self.frames[top].closure_id {
                        Some(cid) => Rc::clone(
                            self.closure_statics
                                .get(&(cid, id))
                                .expect("StaticAlias reached before its StaticStore"),
                        ),
                        None => Rc::clone(
                            self.statics[id as usize]
                                .as_ref()
                                .expect("StaticAlias reached before its StaticStore"),
                        ),
                    };
                    // Rebinding the slot to the static cell drops whatever it held
                    // (`$x = new T; static $x;` discards the temporary T here).
                    let old = std::mem::replace(&mut self.frames[top].slots[slot as usize], Zval::Ref(cell));
                    self.gc_note(&old);
                }
                Op::LoadGlobal(s) => {
                    // `$GLOBALS['x']` read: the global lives in the script frame.
                    let v = read_slot(&self.frames[0].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreGlobal(s) => {
                    // `$GLOBALS['x'] = …`: write/create the global in the script frame.
                    let v = self.frames[top].stack.pop().expect("StoreGlobal on empty stack");
                    let old = store_slot(&mut self.frames[0].slots[s as usize], v);
                    self.gc_note(&old);
                }
                Op::IncDecGlobal { slot, inc, pre } => {
                    let i = slot as usize;
                    if matches!(self.frames[0].slots[i], Zval::Undef) {
                        self.frames[0].slots[i] = Zval::Null;
                    }
                    // Value snapshot + write-through (see IncDecSlot: a reference
                    // slot must yield the pre-increment VALUE and keep aliases).
                    let old = self.frames[0].slots[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), inc)?;
                    // PHP raises the diagnostic *before* writing the result back, so a
                    // `set_error_handler` runs here (it may throw, unwinding this op, or
                    // mutate the variable — which the write-back below then overwrites).
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.frames[0].slots[i], newv.clone());
                    let pushed = if pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::LoadSuperglobal(idx) => {
                    // `$_SERVER` (&c.) read: the value lives in the VM-level store,
                    // resolved by name — correct from any unit/frame. Silent like
                    // `LoadGlobal`.
                    let v = read_slot(&self.superglobals[idx as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSuperglobal(idx) => {
                    let v = self.frames[top].stack.pop().expect("StoreSuperglobal on empty stack");
                    let old = store_slot(&mut self.superglobals[idx as usize], v);
                    self.gc_note(&old);
                }
                Op::GlobalsDynAssign => {
                    // `$GLOBALS[$name] = v`: resolve-or-create the global slot.
                    let v = self.frames[top].stack.pop().expect("GlobalsDynAssign value");
                    let key = self.frames[top].stack.pop().expect("GlobalsDynAssign key");
                    let name = convert::to_zstr_cast(&key, &mut self.diags).as_bytes().to_vec();
                    let slot = self.global_slot_by_name(&name);
                    match &mut self.frames[0].slots[slot] {
                        Zval::Ref(rc) => *rc.borrow_mut() = v.clone(),
                        cell => *cell = v.clone(),
                    }
                    self.frames[top].stack.push(v);
                }
                Op::LoadGlobals => {
                    // Bare `$GLOBALS`: snapshot the script frame's named locals
                    // plus the seeded data superglobals (PHP 8.1 read-only-copy
                    // semantics; `$GLOBALS` itself excluded). The cross-unit
                    // registry (seed_globals) supersedes the main func's own
                    // slot names once includes/dynamic writes extended it.
                    let mut out = PhpArray::new();
                    let names: Vec<Vec<u8>> = if self.seed_globals.is_empty() {
                        self.frames[0].func.slot_names.iter().map(|n| n.to_vec()).collect()
                    } else {
                        self.seed_globals.iter().map(|n| n.to_vec()).collect()
                    };
                    for (i, name) in names.iter().enumerate() {
                        if name.is_empty() {
                            continue;
                        }
                        let v = read_slot(&self.frames[0].slots[i]);
                        if matches!(v, Zval::Undef) {
                            continue;
                        }
                        out.insert(Key::from_bytes(name), v);
                    }
                    for (i, name) in crate::bytecode::SUPERGLOBAL_NAMES.iter().enumerate() {
                        let v = read_slot(&self.superglobals[i]);
                        if matches!(v, Zval::Undef) {
                            continue;
                        }
                        out.insert(Key::from_bytes(name), v);
                    }
                    self.frames[top].stack.push(Zval::Array(Rc::new(out)));
                }
                Op::IncDecSuperglobal { idx, inc, pre } => {
                    let i = idx as usize;
                    if matches!(self.superglobals[i], Zval::Undef) {
                        self.superglobals[i] = Zval::Null;
                    }
                    // Value snapshot + write-through (see IncDecSlot).
                    let old = self.superglobals[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), inc)?;
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.superglobals[i], newv.clone());
                    let pushed = if pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::PushUndef => {
                    self.frames[top].stack.push(Zval::Undef);
                }
                Op::FillDefault { slot, skip } => {
                    // Default-parameter prologue (PAR): skip the default if the
                    // argument was supplied (the slot is not `Undef`).
                    if !matches!(self.frames[top].slots[slot as usize], Zval::Undef) {
                        self.frames[top].ip = skip as usize;
                    }
                }
                Op::CoerceParam { slot, hint } => {
                    // Coerce a just-filled scalar-hinted default (step 14). A valid
                    // constant default always coerces; keep the value otherwise.
                    let v = self.frames[top].slots[slot as usize].clone();
                    if let Ok(c) = coerce_to_hint(v, &hint, &mut self.diags, self.frames[top].module.strict) {
                        self.frames[top].slots[slot as usize] = c;
                    }
                }
                Op::CheckArity { required, exactly } => {
                    let argc = self.frames[top].argc;
                    if argc < required {
                        // `Class::method` for a method, bare name for a function.
                        let func_name = self.frames[top].func.name.clone();
                        let name = match self.frames[top].class {
                            Some(cid) => format!(
                                "{}::{}",
                                String::from_utf8_lossy(&self.classes[cid].name),
                                String::from_utf8_lossy(&func_name)
                            ),
                            None => String::from_utf8_lossy(&func_name).into_owned(),
                        };
                        // The message reports the *call site* line (the caller's
                        // current op), recovered from the EXC-3b line table.
                        let line = if self.frames.len() >= 2 {
                            self.cur_line(self.frames.len() - 2)
                        } else {
                            self.cur_line(top)
                        };
                        let qualifier = if exactly { "exactly" } else { "at least" };
                        let msg = format!(
                            "Too few arguments to function {name}(), {argc} passed in {} on line {line} and {qualifier} {required} expected",
                            String::from_utf8_lossy(&self.module.file)
                        );
                        return Err(PhpError::ArgumentCountError(msg));
                    }
                }
                Op::IncDecSlot { slot, inc, pre } => {
                    let i = slot as usize;
                    if matches!(self.frames[top].slots[i], Zval::Undef) {
                        self.frames[top].slots[i] = Zval::Null;
                    }
                    // Snapshot the VALUE (deref a reference slot): the postfix
                    // result is the pre-increment value, not the live cell —
                    // `$c++ === 0` on a by-ref captured `$c` compared the
                    // already-incremented cell. The write-back goes *through*
                    // the reference so aliases keep seeing the update.
                    let old = self.frames[top].slots[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), inc)?;
                    // Raise before write-back (see IncDecGlobal).
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.frames[top].slots[i], newv.clone());
                    let pushed = if pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::Binary(b) => {
                    let rhs = self.frames[top].stack.pop().expect("Binary rhs");
                    let lhs = self.frames[top].stack.pop().expect("Binary lhs");
                    // A *loose* comparison reads the whole property table, so it
                    // initializes a lazy operand (PHP 8.4, init_trigger_compare)
                    // and compares a proxy's real instance; `===`/`!==` compare
                    // handles and never initialize — and neither does comparing
                    // an object with itself (same handle short-circuits).
                    // Only an object-vs-object comparison reads property tables:
                    // a lazy operand initializes then (init_trigger_compare) —
                    // never for object-vs-scalar (the object simply compares
                    // greater), `===`, or a same-handle compare.
                    let cmp_op = matches!(
                        b,
                        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Spaceship
                    );
                    let both_objects = match (deref_object(&lhs), deref_object(&rhs)) {
                        (Some(a), Some(b)) => Some(Rc::ptr_eq(&a, &b)),
                        _ => None,
                    };
                    let (lhs, rhs) = if matches!(both_objects, Some(false))
                        && cmp_op
                        && (self.is_lazy_value(&lhs) || self.is_lazy_value(&rhs))
                    {
                        (self.realize_full(&lhs)?, self.realize_full(&rhs)?)
                    } else {
                        (lhs, rhs)
                    };
                    // A string-vs-object comparison converts the object through
                    // its `__toString` when it has one (PHP semantics; a lazy
                    // wrapper is NOT initialized by this — the hook is a method
                    // call on the wrapper).
                    let (lhs, rhs) = if cmp_op && both_objects.is_none() {
                        let l_str = matches!(lhs.deref_clone(), Zval::Str(_));
                        let r_str = matches!(rhs.deref_clone(), Zval::Str(_));
                        let to_str = |vm: &mut Self, v: Zval, other_is_str: bool| -> Result<Zval, PhpError> {
                            if !other_is_str {
                                return Ok(v);
                            }
                            if let Some(o) = deref_object(&v) {
                                let cid = o.borrow().class_id as usize;
                                if resolve_method_runtime(&vm.classes, cid, b"__toString").is_some() {
                                    return vm.call_method_sync(v, b"__toString", Vec::new());
                                }
                            }
                            Ok(v)
                        };
                        let lhs = to_str(self, lhs, r_str)?;
                        let rhs = to_str(self, rhs, l_str)?;
                        (lhs, rhs)
                    } else {
                        (lhs, rhs)
                    };
                    let r = apply_binop(b, &lhs, &rhs, &mut self.diags)?;
                    self.frames[top].stack.push(r);
                }
                Op::Unary(u) => {
                    let a = self.frames[top].stack.pop().expect("Unary operand");
                    let r = apply_unop(u, &a, &mut self.diags)?;
                    self.frames[top].stack.push(r);
                }
                Op::Cast(k) => {
                    let a = self.frames[top].stack.pop().expect("Cast operand");
                    // `(array)` does NOT initialize a lazy object (an
                    // uninitialized wrapper casts to its raw — mostly empty —
                    // view, init_trigger_array_cast), but an initialized proxy
                    // casts its real instance: follow the forwarding chain.
                    let a = if matches!(k, CastKind::Array) && self.is_lazy_value(&a) {
                        self.proxy_view(a)
                    } else {
                        a
                    };
                    // `(object)` needs the object table (stdClass alloc); the rest
                    // are pure value conversions.
                    let r = if matches!(k, CastKind::Object) {
                        self.object_cast(a)?
                    } else {
                        apply_cast(k, &a, &mut self.diags)
                    };
                    self.frames[top].stack.push(r);
                }
                Op::Jump(addr) => {
                    self.frames[top].ip = addr as usize;
                }
                Op::JumpIfFalse(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfFalse cond");
                    if !convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::JumpIfTrue(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfTrue cond");
                    if convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::Echo => {
                    let v = self.frames[top].stack.pop().expect("Echo operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.emit_str(top, s.as_bytes())?;
                }
                Op::Print => {
                    let v = self.frames[top].stack.pop().expect("Print operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.emit_str(top, s.as_bytes())?;
                    self.frames[top].stack.push(Zval::Long(1));
                }
                Op::Stringify => {
                    let v = self.frames[top].stack.pop().expect("Stringify operand");
                    let target = v.deref_clone();
                    match &target {
                        Zval::Object(o) => {
                            let cid = o.borrow().class_id as usize;
                            match resolve_method_runtime(&self.classes, cid, b"__toString") {
                                // __toString's (stringified) return flows back via Ret.
                                Some((defc, midx)) => {
                                    let callee = &self.classes[defc].methods[midx].func;
                                    let mut frame = Frame::new(callee, self.class_mod(defc));
                                    frame.this = Some(target.clone());
                                    frame.class = Some(defc);
                                    frame.static_class = Some(cid);
                                    frame.ret_stringify = true;
                                    self.frames.push(frame);
                                }
                                None => {
                                    let name = String::from_utf8_lossy(
                                        o.borrow().class_name.as_bytes(),
                                    )
                                    .into_owned();
                                    return Err(PhpError::Error(format!(
                                        "Object of class {name} could not be converted to string"
                                    )));
                                }
                            }
                        }
                        other => {
                            let s = convert::to_zstr(other, &mut self.diags);
                            self.frames[top].stack.push(Zval::Str(s));
                        }
                    }
                }
                Op::JumpIfNotNull(addr) => {
                    let keep = !matches!(
                        self.frames[top].stack.last(),
                        Some(Zval::Null | Zval::Undef)
                    );
                    if keep {
                        self.frames[top].ip = addr as usize;
                    } else {
                        self.frames[top].stack.pop();
                    }
                }
                Op::JumpIfNull(addr) => {
                    // Peek; the value is kept either way (nullsafe `?->`).
                    if matches!(self.frames[top].stack.last(), Some(Zval::Null | Zval::Undef)) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::ArrayInit => {
                    self.frames[top].stack.push(Zval::Array(Rc::new(PhpArray::new())));
                }
                Op::ArrayPush => {
                    let value = self.frames[top].stack.pop().expect("ArrayPush value");
                    let mut arr = self.frames[top].stack.pop().expect("ArrayPush array");
                    if let Zval::Array(rc) = &mut arr {
                        let _ = Rc::make_mut(rc).append(value);
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::ArrayInsert => {
                    let value = self.frames[top].stack.pop().expect("ArrayInsert value");
                    let key = self.frames[top].stack.pop().expect("ArrayInsert key");
                    let mut arr = self.frames[top].stack.pop().expect("ArrayInsert array");
                    if let Zval::Array(rc) = &mut arr {
                        let k = coerce_key_silent(&key)
                            .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
                        Rc::make_mut(rc).insert(k, value);
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::ArrayAppendSpread => {
                    let src = self.frames[top].stack.pop().expect("ArrayAppendSpread source");
                    // Collect the (int-key → append, string-key → insert) pairs to
                    // merge. A generator is driven to completion (its keys are
                    // re-yielded verbatim, so honour them like an array's).
                    let pairs: Vec<(Key, Zval)> = match src.deref_clone() {
                        Zval::Array(s) => {
                            s.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect()
                        }
                        Zval::Generator(rc) => {
                            let mut out = Vec::new();
                            self.ensure_started(&rc)?;
                            loop {
                                let (k, v, done) = {
                                    let g = rc.borrow();
                                    (g.cur_key.clone(), g.cur_val.clone(), matches!(g.status, GenStatus::Done))
                                };
                                if done {
                                    break;
                                }
                                let key = coerce_key_silent(&k).unwrap_or(Key::Int(0));
                                out.push((key, v));
                                self.resume_generator(&rc, Zval::Null)?;
                            }
                            out
                        }
                        obj @ Zval::Object(_)
                            if object_class_id(&obj).is_some_and(|c| self.is_traversable(c)) =>
                        {
                            self.collect_traversable(obj)?
                        }
                        _ => Vec::new(),
                    };
                    let mut arr = self.frames[top].stack.pop().expect("ArrayAppendSpread array");
                    if let Zval::Array(rc) = &mut arr {
                        let dest = Rc::make_mut(rc);
                        for (k, v) in pairs {
                            if matches!(k, Key::Int(_)) {
                                let _ = dest.append(v);
                            } else {
                                dest.insert(k, v);
                            }
                        }
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::FetchDim => {
                    let key = self.frames[top].stack.pop().expect("FetchDim key");
                    let base = self.frames[top].stack.pop().expect("FetchDim base");
                    // `$o[$k]` on an ArrayAccess object dispatches `offsetGet` (step 51).
                    if let Some(recv) = self.as_arrayaccess(&base) {
                        self.enter_object_method(recv, b"offsetGet", vec![key], RetMode::Stack)?;
                        continue;
                    }
                    let v = read_dim_warn(&base, &key, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::FetchDimList => {
                    // A `list()` element read: undefined-key warns, a scalar
                    // base stays silent (Zend's list path).
                    let key = self.frames[top].stack.pop().expect("FetchDimList key");
                    let base = self.frames[top].stack.pop().expect("FetchDimList base");
                    if let Some(recv) = self.as_arrayaccess(&base) {
                        self.enter_object_method(recv, b"offsetGet", vec![key], RetMode::Stack)?;
                        continue;
                    }
                    let v = read_dim_warn_list(&base, &key, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::CoalesceFetchDim => {
                    let key = self.frames[top].stack.pop().expect("CoalesceFetchDim key");
                    let base = self.frames[top].stack.pop().expect("CoalesceFetchDim base");
                    self.frames[top].stack.push(read_dim_nullable(&base, &key));
                }
                Op::AssignPath { base, nkeys, append } => {
                    let value = self.frames[top].stack.pop().expect("AssignPath value");
                    let mut keys = self.pop_keys(top, nkeys);
                    // `$o[$k] = v` / `$o[] = v` on an ArrayAccess object dispatches
                    // `offsetSet` (a single step only); the expression yields `v`.
                    if nkeys + append as u32 == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(base, top)) {
                            let key = if append { Zval::Null } else { keys.pop().expect("set key") };
                            self.frames[top].stack.push(value.clone());
                            self.enter_object_method(recv, b"offsetSet", vec![key, value], RetMode::Discard)?;
                            continue;
                        }
                    }
                    let last = if append {
                        Last::Append { value }
                    } else {
                        Last::Set { key: keys.pop().expect("AssignPath key"), value }
                    };
                    let result = self.path_op(base, top, keys, last)?;
                    self.frames[top].stack.push(result);
                }
                Op::AssignOpPath { base, nkeys, op } => {
                    let rhs = self.frames[top].stack.pop().expect("AssignOpPath rhs");
                    let mut keys = self.pop_keys(top, nkeys);
                    let key = keys.pop().expect("AssignOpPath key");
                    let result = self.path_op(base, top, keys, Last::OpSet { key, op, rhs })?;
                    self.frames[top].stack.push(result);
                }
                Op::IncDecPath { base, nkeys, inc, pre } => {
                    let mut keys = self.pop_keys(top, nkeys);
                    let key = keys.pop().expect("IncDecPath key");
                    let result = self.path_op(base, top, keys, Last::IncDec { key, inc, pre })?;
                    self.frames[top].stack.push(result);
                }
                Op::IssetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    // `isset($o[$k])` on an ArrayAccess object is `offsetExists($k)`
                    // (a single step only; it does not call `offsetGet`).
                    if nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(base, top)) {
                            let key = keys.into_iter().next().expect("isset key");
                            self.enter_object_method(recv, b"offsetExists", vec![key], RetMode::Stack)?;
                            continue;
                        }
                    }
                    // Nested form landing on an ArrayAccess (`isset($m['k'][2])`
                    // where the element holds one): protocol on the leaf.
                    if let Some((recv, key)) = self.dim_aa_leaf(base, top, &keys) {
                        let r = self.call_method_sync(recv, b"offsetExists", vec![key])?;
                        let set = convert::is_true_silent(&r.deref_clone());
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    let set = matches!(
                        silent_get_path(self.base_cell(base, top), &keys),
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                    );
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::EmptyPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    // `empty($o[$k])` on an ArrayAccess object: `!offsetExists($k)`
                    // short-circuits to empty (so `offsetGet` — which may throw for
                    // an absent key, e.g. WeakMap — is skipped); otherwise
                    // `!truthy(offsetGet($k))`. A single step only.
                    if nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(base, top)) {
                            let key = keys.into_iter().next().expect("empty key");
                            let exists =
                                self.call_method_sync(recv.clone(), b"offsetExists", vec![key.clone()])?;
                            let empty = if convert::is_true_silent(&exists) {
                                let v = self.call_method_sync(recv, b"offsetGet", vec![key])?;
                                !convert::is_true_silent(&v)
                            } else {
                                true
                            };
                            self.frames[top].stack.push(Zval::Bool(empty));
                            continue;
                        }
                    }
                    if let Some((recv, key)) = self.dim_aa_leaf(base, top, &keys) {
                        let exists =
                            self.call_method_sync(recv.clone(), b"offsetExists", vec![key.clone()])?;
                        let empty = if convert::is_true_silent(&exists.deref_clone()) {
                            let v = self.call_method_sync(recv, b"offsetGet", vec![key])?;
                            !convert::is_true_silent(&v.deref_clone())
                        } else {
                            true
                        };
                        self.frames[top].stack.push(Zval::Bool(empty));
                        continue;
                    }
                    let empty = match silent_get_path(self.base_cell(base, top), &keys) {
                        Some(v) => !convert::is_true_silent(&v),
                        None => true,
                    };
                    self.frames[top].stack.push(Zval::Bool(empty));
                }
                Op::UnsetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    // `unset($o[$k])` on an ArrayAccess object is `offsetUnset($k)`
                    // (a single step only).
                    if nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(base, top)) {
                            let key = keys.into_iter().next().expect("unset key");
                            self.enter_object_method(recv, b"offsetUnset", vec![key], RetMode::Discard)?;
                            continue;
                        }
                    }
                    if let Some((recv, key)) = self.dim_aa_leaf(base, top, &keys) {
                        self.call_method_sync(recv, b"offsetUnset", vec![key])?;
                        continue;
                    }
                    // `unset($x)` drops the variable's value: capture and note it
                    // (it may hold the last reference to an object with a
                    // destructor, which then runs at this point). A nested
                    // `unset($x[$k])` removes a deep element — left to the cascade
                    // / full scan.
                    let dropped = if keys.is_empty() {
                        Some(match base {
                            DimBase::Local(s) => std::mem::replace(&mut self.frames[top].slots[s as usize], Zval::Undef),
                            DimBase::Global(s) => std::mem::replace(&mut self.frames[0].slots[s as usize], Zval::Undef),
                            DimBase::Superglobal(i) => std::mem::replace(&mut self.superglobals[i as usize], Zval::Undef),
                        })
                    } else {
                        let cell = match base {
                            DimBase::Local(s) => &mut self.frames[top].slots[s as usize],
                            DimBase::Global(s) => &mut self.frames[0].slots[s as usize],
                            DimBase::Superglobal(i) => &mut self.superglobals[i as usize],
                        };
                        unset_into(cell, &keys);
                        None
                    };
                    if let Some(old) = dropped {
                        self.gc_note(&old);
                    }
                }
                Op::BindRef { target, source } => {
                    // REF-1: promote `source` to a shared cell, alias `target` to
                    // the same `Rc`, and push the cell's value (the assignment
                    // expression yields the aliased value). The two slot reads are
                    // sequential, so the borrows never overlap.
                    let cell = make_cell(ref_base_mut(&mut self.frames, top, source));
                    let value = cell.borrow().clone();
                    *ref_base_mut(&mut self.frames, top, target) = Zval::Ref(cell);
                    self.frames[top].stack.push(value);
                }
                Op::PushRef(slot) => {
                    // REF-2: promote the local to a shared cell and push the ref;
                    // the next `Op::Call` binds it into the by-ref callee slot.
                    let cell = make_cell(&mut self.frames[top].slots[slot as usize]);
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::MakeClosure { fn_idx, captures, bind_this } => {
                    let mut bound = Vec::with_capacity(captures.len());
                    for cap in captures.iter() {
                        let val = if cap.by_ref {
                            Zval::Ref(make_cell(&mut self.frames[top].slots[cap.src as usize]))
                        } else {
                            read_slot(&self.frames[top].slots[cap.src as usize])
                        };
                        bound.push((cap.dst, val));
                    }
                    let bound_this = if bind_this { self.frames[top].this.clone() } else { None };
                    let m = self.frames[top].module;
                    // A closure index always resolves in the unit that compiled the
                    // body — except a trait method flattened into a class from
                    // *another* unit: its closure indices point at the trait's unit,
                    // not the consumer's (cross-unit trait-closure relocation is not
                    // yet implemented). Surface a catchable error rather than panic.
                    let Some(func) = m.closures.get(fn_idx as usize) else {
                        return Err(PhpError::Error(
                            "closure from a trait used across files is not yet supported"
                                .to_string(),
                        ));
                    };
                    let info = Rc::new(ClosureInfo {
                        kind: ClosureRender::Closure {
                            name: PhpStr::new(func.name.to_vec()),
                            file: PhpStr::new(m.file.to_vec()),
                            line: func.line,
                        },
                        params: closure_params(func),
                    });
                    let id = self.next_id();
                    let module_id = self.module_id(m);
                    let scope = self.frames[top].class;
                    let cl = Closure {
                        fn_idx: fn_idx as usize,
                        captures: bound,
                        named: None,
                        bound_this,
                        id,
                        info,
                        module_id,
                        scope,
                        is_static: !bind_this,
                    };
                    self.frames[top].stack.push(Zval::Closure(Rc::new(cl)));
                }
                Op::MakeFcc { name } => {
                    // CLO-2: a first-class callable wraps a function *name*. A user
                    // function contributes its `[parameter]` dump; an internal
                    // (registry) callable has none.
                    let params = self
                        .module
                        .functions
                        .iter()
                        .find(|f| name_eq_ignore_case(&f.name, &name))
                        .map(closure_params)
                        .unwrap_or_default();
                    let info = Rc::new(ClosureInfo {
                        kind: ClosureRender::Function(PhpStr::new(name.to_vec())),
                        params,
                    });
                    let id = self.next_id();
                    let cl = Closure {
                        fn_idx: 0,
                        captures: Vec::new(),
                        named: Some(PhpStr::new(name.to_vec())),
                        bound_this: None,
                        id,
                        info,
                        module_id: 0,
                        scope: None,
                        is_static: false,
                    };
                    self.frames[top].stack.push(Zval::Closure(Rc::new(cl)));
                }
                Op::CallValue { argc } => {
                    let n = argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("CallValue argument"));
                    }
                    args.reverse();
                    let callee = self.frames[top].stack.pop().expect("CallValue callee");
                    self.invoke_value(callee, args)?;
                }
                Op::CallNsFallback { name, fallback, argc } => {
                    let n = argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("CallNsFallback argument"));
                    }
                    args.reverse();
                    self.invoke_named_fallback(&name, &fallback, args)?;
                }
                Op::CallValueArgs => {
                    // Spread `$f(...$a)`: the arguments are the values of a runtime
                    // array (the callee sits beneath it), expanded in order.
                    let argsval = self.frames[top].stack.pop().expect("CallValueArgs array");
                    let args = args_from_array_value(argsval);
                    let callee = self.frames[top].stack.pop().expect("CallValueArgs callee");
                    self.invoke_value(callee, args)?;
                }
                Op::Throw => {
                    let v = self.frames[top].stack.pop().expect("throw operand");
                    return Err(PhpError::Thrown(v.deref_clone()));
                }
                Op::Rethrow => {
                    let v = self.frames[top].stack.pop().expect("rethrow operand");
                    return Err(PhpError::Thrown(v));
                }
                Op::CatchMatch { types, names, var, body } => {
                    let exc = self.frames[top].stack.last().expect("in-flight exception").clone();
                    let caught = object_class_id(&exc).is_some_and(|ec| {
                        types
                            .iter()
                            .any(|&t| is_instance_of(&self.classes, self.stringable_id, ec, t))
                            // Classes declared after compile time (by eval/include) are
                            // resolved by name against the live class table (Phase 2).
                            || names.iter().any(|n| {
                                self.class_index
                                    .get(&n.to_ascii_lowercase())
                                    .is_some_and(|&t| {
                                        is_instance_of(&self.classes, self.stringable_id, ec, t)
                                    })
                            })
                    });
                    if caught {
                        self.frames[top].stack.pop();
                        if let Some(slot) = var {
                            let displaced =
                                store_slot(&mut self.frames[top].slots[slot as usize], exc);
                            self.gc_note(&displaced);
                        } else {
                            // A capture-less catch drops the throwable right here.
                            self.gc_note(&exc);
                        }
                        self.frames[top].ip = body as usize;
                    }
                    // else: fall through to the next CatchMatch / Rethrow.
                }
                Op::EndFinally { after } => {
                    // EXC-2/2b: resolve the finally's pending action. A propagating
                    // exception wins; then a parked return (push the value and fall
                    // through to the trailing `Ret`); then a parked break/continue
                    // (jump to its loop target); otherwise skip past the `try`.
                    if let Some(v) = self.frames[top].pending_throw.take() {
                        return Err(PhpError::Thrown(v));
                    }
                    match self.frames[top].pending_transfer.take() {
                        Some(Transfer::Return(val)) => {
                            self.frames[top].stack.push(val);
                            // fall through to the `Ret` emitted right after this op
                        }
                        Some(Transfer::Jump(addr)) => {
                            self.frames[top].ip = addr as usize;
                        }
                        None => {
                            self.frames[top].ip = after as usize;
                        }
                    }
                }
                Op::ParkReturn => {
                    let v = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    self.frames[top].pending_transfer = Some(Transfer::Return(v));
                }
                Op::ParkJump(addr) => {
                    self.frames[top].pending_transfer = Some(Transfer::Jump(addr));
                }
                Op::DerefTop => {
                    // REF-4b: copy a by-ref return used in value context.
                    if let Some(Zval::Ref(_)) = self.frames[top].stack.last() {
                        let v = self.frames[top].stack.pop().unwrap().deref_clone();
                        self.frames[top].stack.push(v);
                    }
                }
                Op::MakeRef { base, steps } => {
                    // REF-4: navigate to the place's leaf, promote it to a shared
                    // cell, and push a reference to it. Keys (for `Index` steps)
                    // were pushed in source order and sit on top of the stack.
                    let keys = self.pop_field_keys(top, &steps);
                    // First key retained past the cell walk (a dynamic-name
                    // typed-ref registration needs it after keys are consumed).
                    let keys_first_backup = keys.first().cloned();
                    // EXCEPTION checked FIRST (it must pre-empt hook/lazy
                    // machinery): a ref fetch under the wrapper's own
                    // suppressed `__get` guard stays on the wrapper's raw
                    // storage without initializing (gh20854's
                    // `return $this->x;` in the wrapper's `&__get`).
                    let guarded_ref_fetch = 'grf: {
                        let n: Box<[u8]> = match &steps[..] {
                            [FieldStep::Prop(n)] => n.clone(),
                            [FieldStep::PropDyn] => match keys.first().cloned() {
                                Some(k) => self.dyn_prop_name_value(&k)?,
                                None => break 'grf false,
                            },
                            _ => break 'grf false,
                        };
                        let target = match base {
                            FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                            FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                        };
                        let Some(o) = target.and_then(|v| deref_object(&v)) else { break 'grf false };
                        let (uninit, oid, cid) = {
                            let b = o.borrow();
                            (b.lazy.is_some() && b.proxy_instance.is_none(), b.id, b.class_id as usize)
                        };
                        uninit
                            && self.magic_guard.contains(&(oid, MagicKind::Get, n.to_vec()))
                            && resolve_method_runtime(&self.classes, cid, b"__get").is_some()
                    };
                    // A `&get` hook makes the property addressable: the reference
                    // the hook returns is the path's root (PHP 8.4).
                    if !guarded_ref_fetch {
                        if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                            let cell = if steps.len() == 1 {
                                root
                            } else {
                                let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                                let mut root_val = Zval::Ref(root);
                                field_cell(&mut root_val, &steps[1..], &mut keys.into_iter(), fs)
                            };
                            self.frames[top].stack.push(Zval::Ref(cell));
                            continue;
                        }
                    }
                    // A property whose set visibility excludes this scope hands
                    // out a reference to a *copy* (PHP 8.4 asymmetric visibility).
                    if let Some(cell) = self.asym_set_ref_copy(base, top, &steps) {
                        self.frames[top].stack.push(Zval::Ref(cell));
                        continue;
                    }
                    // Taking a reference *to* a hooked property is indirect modification.
                    self.reject_indirect_hook(base, top, &steps)?;
                    // A reference fetch of a lazy object's property initializes it
                    // first (PHP 8.4, fetch_ref_initializes) — a skipped/raw-set
                    // property does not — and a forwarding proxy binds the
                    // *instance*'s cell, so the walk roots there (unless the
                    // guarded-ref-fetch exception above applies).
                    let lazy_root = if guarded_ref_fetch {
                        None
                    } else {
                        self.field_lazy_root(base, top, &steps, &keys, false)?
                    };
                    // `&$o->magic` consults `__get` (gh21478-proxy-get-ref-
                    // forward, gh20875_proxy_get_no_init): the magic result
                    // binds as a fresh cell — an uninitialized proxy dispatches
                    // on the wrapper WITHOUT initializing. Under an ACTIVE
                    // guard the fetch degrades to an undefined READ (warning,
                    // detached NULL cell — nothing materializes).
                    let makeref_magic: Option<Rc<RefCell<Zval>>> = 'makeref_magic: {
                        let n: Box<[u8]> = match &steps[..] {
                            [FieldStep::Prop(n)] => n.clone(),
                            [FieldStep::PropDyn] => {
                                let Some(k) = keys.first().cloned() else { break 'makeref_magic None };
                                self.dyn_prop_name_value(&k)?
                            }
                            _ => break 'makeref_magic None,
                        };
                        let target = match base {
                            FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                            FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                        };
                        let Some(o) = target.and_then(|v| deref_object(&v)) else { break 'makeref_magic None };
                        let cur = self.frames[top].class;
                        let oid = o.borrow().id;
                        // For a forwarding wrapper the guard may be held on the
                        // instance's handle — honor both before dispatching.
                        let inst = deref_object(&self.proxy_view(Zval::Object(Rc::clone(&o))));
                        let inst_guard = inst
                            .as_ref()
                            .is_some_and(|io| self.magic_guard.contains(&(io.borrow().id, MagicKind::Get, n.to_vec())));
                        if !inst_guard && self.magic_applies(&o, &n, cur, MagicKind::Get, b"__get").is_some() {
                            let gkey = (oid, MagicKind::Get, n.to_vec());
                            let ins = self.magic_guard.insert(gkey.clone());
                            let r = self.call_method_sync(
                                Zval::Object(Rc::clone(&o)),
                                b"__get",
                                vec![Zval::Str(PhpStr::new(n.to_vec()))],
                            );
                            if ins {
                                self.magic_guard.remove(&gkey);
                            }
                            let v = r?;
                            break 'makeref_magic Some(Rc::new(RefCell::new(v.deref_clone())));
                        }
                        // `__get` guard-suppressed on a forwarding PROXY WRAPPER
                        // with an absent slot: Zend's ptr-fetch falls back to a
                        // READ — undefined-property warning naming the INSTANCE
                        // class, detached NULL cell, nothing materializes. (A
                        // plain object instead materializes with the creation
                        // deprecation — gh20854.)
                        let is_wrapper = {
                            let b = o.borrow();
                            b.lazy.is_some() && b.proxy_instance.is_some()
                        };
                        if is_wrapper {
                            let cid_o = o.borrow().class_id as usize;
                            let has_get = resolve_method_runtime(&self.classes, cid_o, b"__get").is_some();
                            let undeclared = matches!(
                                resolve_prop_access(&self.classes, cid_o, &n, cur),
                                PropAccess::Dynamic
                            );
                            let absent_on_inst = inst
                                .as_ref()
                                .is_some_and(|io| !io.borrow().props.contains(n.as_ref()));
                            let guarded = inst_guard
                                || self.magic_guard.contains(&(oid, MagicKind::Get, n.to_vec()));
                            if has_get && undeclared && absent_on_inst && guarded {
                                let icid = inst
                                    .as_ref()
                                    .map(|io| io.borrow().class_id as usize)
                                    .unwrap_or(cid_o);
                                self.diags.push(Diag::Warning(format!(
                                    "Undefined property: {}::${}",
                                    String::from_utf8_lossy(&self.classes[icid].name),
                                    String::from_utf8_lossy(&n),
                                )));
                                break 'makeref_magic Some(Rc::new(RefCell::new(Zval::Null)));
                            }
                        }
                        None
                    };
                    if let Some(cell) = makeref_magic {
                        self.frames[top].stack.push(Zval::Ref(cell));
                        continue;
                    }
                    // `&$o->undeclared` MATERIALIZES the property: on a class not
                    // allowing dynamic props that is the creation deprecation
                    // (gh20854's `&__get` returning an absent `$this->x`).
                    if let [FieldStep::Prop(n)] = &steps[..] {
                        let target = match &lazy_root {
                            Some(r) => Some(r.clone()),
                            None => match base {
                                FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                                FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                            },
                        };
                        if let Some(o) = target.as_ref().and_then(deref_object) {
                            let ocid = o.borrow().class_id as usize;
                            let dynamic = matches!(
                                resolve_prop_access(&self.classes, ocid, n, self.frames[top].class),
                                PropAccess::Dynamic
                            );
                            if dynamic
                                && !o.borrow().props.contains(n.as_ref())
                                && !self.allows_dynamic_props(ocid)
                            {
                                let cls = String::from_utf8_lossy(&self.classes[ocid].name).into_owned();
                                let prop = String::from_utf8_lossy(n).into_owned();
                                self.diags.push(Diag::Deprecated(format!(
                                    "Creation of dynamic property {cls}::${prop} is deprecated"
                                )));
                            }
                        }
                    }
                    // Taking a reference to an *uninitialized* non-nullable typed
                    // property is an error (zend_fetch_property_address,
                    // fetch_ref_skipped_prop_does_not_initialize).
                    if let [FieldStep::Prop(n)] = &steps[..] {
                        let target = match &lazy_root {
                            Some(r) => Some(r.clone()),
                            None => match base {
                                FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                                FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                            },
                        };
                        if let Some(o) = target.as_ref().and_then(deref_object) {
                            let cid = o.borrow().class_id as usize;
                            let key = self.prop_storage_key(cid, n, self.frames[top].class);
                            if matches!(o.borrow().props.get(&key), Some(Zval::Undef)) {
                                if let Some((decl, hint)) = prop_type_decl(&self.classes, cid, n) {
                                    if !hint.nullable {
                                        return Err(PhpError::Error(format!(
                                            "Cannot access uninitialized non-nullable property {}::${} by reference",
                                            String::from_utf8_lossy(&self.classes[decl].name),
                                            String::from_utf8_lossy(n),
                                        )));
                                    }
                                }
                            }
                        }
                    }
                    let cell = {
                        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                        if let Some(mut root) = lazy_root {
                            field_cell(&mut root, &steps, &mut keys.into_iter(), fs)
                        } else {
                            let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, base)?;
                            if steps.is_empty() {
                                make_cell(base_cell)
                            } else {
                                field_cell(base_cell, &steps, &mut keys.into_iter(), fs)
                            }
                        }
                    };
                    // A reference to a *typed* property keeps enforcing its type
                    // on writes through the alias (PHP's typed references).
                    match &steps[..] {
                        [FieldStep::Prop(name)] => {
                            let name = name.clone();
                            self.register_prop_typed_ref(base, top, &name, &cell);
                        }
                        [FieldStep::PropDyn] => {
                            if let Some(k) = keys_first_backup.as_ref() {
                                let name = self.dyn_prop_name_value(k)?;
                                self.register_prop_typed_ref(base, top, &name, &cell);
                            }
                        }
                        _ => {}
                    }
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::BindRefTo { base, steps } => {
                    // REF-4: pop the reference, bind the target place to its cell,
                    // and push the aliased value (the assignment's result).
                    // Binding a reference *into* a hooked property is forbidden.
                    if self.field_starts_at_hook(base, top, &steps) {
                        if steps.len() > 1 {
                            // Navigating *into* the property: a `&get` hook's cell
                            // is an addressable root — bind the leaf inside it.
                            if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                                let top_val = self.frames[top].stack.pop().expect("BindRefTo value");
                                let cell = match top_val {
                                    Zval::Ref(rc) => rc,
                                    other => Rc::new(RefCell::new(other)),
                                };
                                let value = cell.borrow().clone();
                                let keys = self.pop_field_keys(top, &steps);
                                self.field_set_in_root(root, top, &steps[1..], keys, Zval::Ref(cell), true)?;
                                self.frames[top].stack.push(value);
                                continue;
                            }
                        } else {
                            // The write fetch of the rebind target runs a `&get`
                            // hook first (observable side effects, bug007) — then
                            // PHP still rejects rebinding the property slot.
                            let _ = self.byref_hook_root(base, top, &steps)?;
                        }
                        return Err(PhpError::Error(
                            "Cannot assign by reference to overloaded object".to_string(),
                        ));
                    }
                    let top_val = self.frames[top].stack.pop().expect("BindRefTo value");
                    let cell = match top_val {
                        Zval::Ref(rc) => rc,
                        other => Rc::new(RefCell::new(other)),
                    };
                    let mut keys = self.pop_field_keys(top, &steps);
                    // A lazy base initializes/forwards; binding into a typed
                    // property validates the reference's value at bind time and
                    // registers the typed source (typed_properties_001).
                    let lazy_root = self.field_lazy_root(base, top, &steps, &keys, true)?;
                    {
                        let target = match &lazy_root {
                            Some(r) => Some(r.clone()),
                            None => match base {
                                FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                                FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                                FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                            },
                        };
                        self.bind_ref_typed_check(target.as_ref(), &steps, &mut keys, &cell)?;
                    }
                    let value = cell.borrow().clone();
                    if let Some(root) = lazy_root {
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, Zval::Ref(cell), true)?;
                    } else if steps.is_empty() {
                        // A step-less base is rebound directly (not written
                        // through), matching `eval::bind_ref_target`.
                        let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, base)?;
                        *base_cell = Zval::Ref(cell);
                    } else {
                        self.field_set_mode(base, top, &steps, keys, Zval::Ref(cell), true)?;
                    }
                    self.frames[top].stack.push(value);
                }
                Op::BindRefToChecked { base, steps } => {
                    // `$t = &m()` for a method/static call: the callee's by-ref-ness
                    // is only known now. A non-`Ref` source means the callee did not
                    // return by reference — raise the notice, then bind a copy.
                    if self.field_starts_at_hook(base, top, &steps) {
                        if steps.len() == 1 {
                            // Run a `&get` hook's observable side effects before
                            // rejecting the rebind (mirrors `Op::BindRefTo`).
                            let _ = self.byref_hook_root(base, top, &steps)?;
                        } else if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                            let top_val = self.frames[top].stack.pop().expect("BindRefToChecked value");
                            let cell = match top_val {
                                Zval::Ref(rc) => rc,
                                other => {
                                    self.diags.push(Diag::Notice(
                                        "Only variables should be assigned by reference".to_string(),
                                    ));
                                    Rc::new(RefCell::new(other))
                                }
                            };
                            let value = cell.borrow().clone();
                            let keys = self.pop_field_keys(top, &steps);
                            self.field_set_in_root(root, top, &steps[1..], keys, Zval::Ref(cell), true)?;
                            self.frames[top].stack.push(value);
                            continue;
                        }
                        return Err(PhpError::Error(
                            "Cannot assign by reference to overloaded object".to_string(),
                        ));
                    }
                    let top_val = self.frames[top].stack.pop().expect("BindRefToChecked value");
                    let cell = match top_val {
                        Zval::Ref(rc) => rc,
                        other => {
                            self.diags.push(Diag::Notice(
                                "Only variables should be assigned by reference".to_string(),
                            ));
                            Rc::new(RefCell::new(other))
                        }
                    };
                    let value = cell.borrow().clone();
                    let keys = self.pop_field_keys(top, &steps);
                    if steps.is_empty() {
                        let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, base)?;
                        *base_cell = Zval::Ref(cell);
                    } else {
                        self.field_set_mode(base, top, &steps, keys, Zval::Ref(cell), true)?;
                    }
                    self.frames[top].stack.push(value);
                }
                Op::IterInit => {
                    let iterable = self.frames[top].stack.pop().expect("IterInit iterable");
                    let deref = iterable.deref_clone();
                    // foreach over a lazy object initializes it first (PHP 8.4); a
                    // proxy then iterates its real instance's properties.
                    let deref = if deref_object(&deref).is_some_and(|o| o.borrow().lazy.is_some()) {
                        self.realize_full(&deref)?
                    } else {
                        deref
                    };
                    // A generator iterates live (no snapshot); an `Iterator` /
                    // `IteratorAggregate` object drives the protocol via the
                    // re-entrant state machine in `IterNext` (step 51); an array /
                    // plain object is snapshotted by value (GEN).
                    let scope = self.frames[top].class;
                    let it_state = match &deref {
                        Zval::Generator(gs) => IterState::Gen { rc: Rc::clone(gs), primed: false },
                        Zval::Object(o) if self.is_traversable(o.borrow().class_id as usize) => {
                            IterState::Object {
                                it: deref.clone(),
                                stage: ObjStage::Start,
                                pending: None,
                                cur_val: None,
                            }
                        }
                        // A plain (non-Traversable) object iterates its visible
                        // properties (declared first, then dynamic): the key set is
                        // fixed here, values are read live at each step.
                        Zval::Object(_) => {
                            IterState::ObjVals { obj: deref.clone(), scope, yielded: Vec::new() }
                        }
                        _ => IterState::ByVal { entries: snapshot_entries(&iterable), pos: 0 },
                    };
                    self.frames[top].iters.push(it_state);
                }
                Op::IterNext { value, key, end } => {
                    // A generator step: prime on the first visit, otherwise resume
                    // to the next yield, then bind the current `(key, value)` or
                    // jump to `end` when the generator is done (GEN).
                    let gen = match self.frames[top].iters.last_mut() {
                        Some(IterState::Gen { rc, primed }) => {
                            let rc = Rc::clone(rc);
                            let was_primed = *primed;
                            *primed = true;
                            Some((rc, was_primed))
                        }
                        _ => None,
                    };
                    if let Some((rc, was_primed)) = gen {
                        if was_primed {
                            self.resume_generator(&rc, Zval::Null)?;
                        } else {
                            self.ensure_started(&rc)?;
                        }
                        let (k, v, done) = {
                            let gs = rc.borrow();
                            (gs.cur_key.clone(), gs.cur_val.clone(), matches!(gs.status, GenStatus::Done))
                        };
                        if done {
                            self.frames[top].ip = end as usize;
                        } else {
                            store_slot(&mut self.frames[top].slots[value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], k);
                            }
                        }
                        continue;
                    }
                    // Plain-object foreach: yield the next not-yet-visited visible
                    // property, recomputed live (a property added in the body is
                    // reached, a removed one skipped); a hooked property reads
                    // through its `get` hook at this step.
                    if matches!(self.frames[top].iters.last(), Some(IterState::ObjVals { .. })) {
                        let pair = loop {
                            let (obj, entry) = {
                                let Some(IterState::ObjVals { obj, scope, yielded }) =
                                    self.frames[top].iters.last()
                                else {
                                    unreachable!("ObjVals iterator")
                                };
                                let Zval::Object(o) = obj else { break None };
                                let next = self
                                    .object_iter_entries(o, *scope)
                                    .into_iter()
                                    .find(|(d, _)| !yielded.iter().any(|y| y == d));
                                let Some(next) = next else { break None };
                                (obj.clone(), next)
                            };
                            let (display, entry) = entry;
                            if let Some(IterState::ObjVals { yielded, .. }) =
                                self.frames[top].iters.last_mut()
                            {
                                yielded.push(display.clone());
                            }
                            match entry {
                                PropIterEntry::Slot { key } => {
                                    let v = deref_object(&obj).and_then(|o| o.borrow().props.get(&key).cloned());
                                    if let Some(v) = v {
                                        if !matches!(v, Zval::Undef) {
                                            break Some((
                                                Zval::Str(PhpStr::new(display.to_vec())),
                                                v.deref_clone(),
                                            ));
                                        }
                                    }
                                    // Vanished since the entry list was built: skip.
                                }
                                PropIterEntry::Hook { name, view } => {
                                    // `push_hook` already derefs a `&get` return in
                                    // this value context (`ret_deref`).
                                    let v = self.run_iter_get_hook(&obj, &name, view, true)?;
                                    break Some((Zval::Str(PhpStr::new(display.to_vec())), v.deref_clone()));
                                }
                            }
                        };
                        match pair {
                            None => self.frames[top].ip = end as usize,
                            Some((k, v)) => {
                                store_slot(&mut self.frames[top].slots[value as usize], v);
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[ks as usize], k);
                                }
                            }
                        }
                        continue;
                    }
                    // Object iterator (Iterator / IteratorAggregate): a re-entrant
                    // state machine drives the protocol methods one per re-entry,
                    // each call's return captured via `ret_cell` (step 51).
                    let obj_stage = match self.frames[top].iters.last() {
                        Some(IterState::Object { stage, .. }) => Some(*stage),
                        _ => None,
                    };
                    if let Some(stage) = obj_stage {
                        match stage {
                            ObjStage::Start => {
                                let it = self.obj_iter_value(top);
                                let cid = object_class_id(&it).unwrap_or(0);
                                if self.is_aggregate(cid) {
                                    self.issue_iter_call(top, ip, b"getIterator", vec![], true, ObjStage::AfterAggregate)?;
                                } else {
                                    self.set_obj_stage(top, ObjStage::NeedRewind);
                                    self.frames[top].ip = ip;
                                }
                                continue;
                            }
                            ObjStage::AfterAggregate => {
                                let inner = self.take_obj_pending(top).deref_clone();
                                let it_obj_cid = object_class_id(&inner);
                                let new_state = match &inner {
                                    Zval::Generator(gs) => IterState::Gen { rc: Rc::clone(gs), primed: false },
                                    Zval::Object(_) if it_obj_cid.is_some_and(|c| self.is_traversable(c)) => {
                                        IterState::Object { it: inner.clone(), stage: ObjStage::Start, pending: None, cur_val: None }
                                    }
                                    _ => {
                                        let cls = String::from_utf8_lossy(&self.classes[object_class_id(&self.obj_iter_value(top)).unwrap_or(0)].name).into_owned();
                                        return Err(PhpError::Error(format!(
                                            "Objects returned by {cls}::getIterator() must be traversable or implement interface Iterator"
                                        )));
                                    }
                                };
                                *self.frames[top].iters.last_mut().expect("object iterator") = new_state;
                                self.frames[top].ip = ip; // re-run with the resolved iterator
                                continue;
                            }
                            ObjStage::NeedRewind => {
                                self.issue_iter_call(top, ip, b"rewind", vec![], false, ObjStage::NeedValid)?;
                                continue;
                            }
                            ObjStage::NeedValid => {
                                self.issue_iter_call(top, ip, b"valid", vec![], true, ObjStage::AfterValid)?;
                                continue;
                            }
                            ObjStage::AfterValid => {
                                let v = self.take_obj_pending(top);
                                let valid = convert::to_bool(&v, &mut self.diags);
                                if valid {
                                    self.set_obj_stage(top, ObjStage::NeedCurrent);
                                    self.frames[top].ip = ip;
                                } else {
                                    self.frames[top].ip = end as usize;
                                }
                                continue;
                            }
                            ObjStage::NeedCurrent => {
                                self.issue_iter_call(top, ip, b"current", vec![], true, ObjStage::AfterCurrent)?;
                                continue;
                            }
                            ObjStage::AfterCurrent => {
                                let v = self.take_obj_pending(top);
                                if let Some(IterState::Object { cur_val, stage, .. }) = self.frames[top].iters.last_mut() {
                                    *cur_val = Some(v);
                                    *stage = ObjStage::NeedKey;
                                }
                                self.frames[top].ip = ip;
                                continue;
                            }
                            ObjStage::NeedKey => {
                                self.issue_iter_call(top, ip, b"key", vec![], true, ObjStage::AfterKey)?;
                                continue;
                            }
                            ObjStage::AfterKey => {
                                let k = self.take_obj_pending(top);
                                let v = match self.frames[top].iters.last_mut() {
                                    Some(IterState::Object { cur_val, stage, .. }) => {
                                        *stage = ObjStage::NeedNext;
                                        cur_val.take().unwrap_or(Zval::Null)
                                    }
                                    _ => Zval::Null,
                                };
                                store_slot(&mut self.frames[top].slots[value as usize], v.deref_clone());
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[ks as usize], k.deref_clone());
                                }
                                continue; // ip is already past IterNext: run the body
                            }
                            ObjStage::NeedNext => {
                                self.issue_iter_call(top, ip, b"next", vec![], false, ObjStage::NeedValid)?;
                                continue;
                            }
                        }
                    }
                    // Read the cursor and bump it in a scoped borrow, then touch
                    // the slots — keeping the `iters` and `slots` borrows disjoint.
                    let pair = {
                        let it = self.frames[top].iters.last_mut().expect("IterNext without iterator");
                        let IterState::ByVal { entries, pos } = it else {
                            unreachable!("IterNext on a by-reference iterator");
                        };
                        if *pos >= entries.len() {
                            None
                        } else {
                            let pair = entries[*pos].clone();
                            *pos += 1;
                            Some(pair)
                        }
                    };
                    match pair {
                        None => self.frames[top].ip = end as usize,
                        Some((k, v)) => {
                            // Deref at bind time: a reference element snapshots its
                            // cell and is read live here. `store_slot` writes
                            // *through* a value slot that is itself a reference (the
                            // lingering-ref gotcha), matching the tree-walker.
                            store_slot(&mut self.frames[top].slots[value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], k);
                            }
                        }
                    }
                }
                Op::IterInitRef(source) => {
                    // REF-3: snapshot the source's keys once; each step rebinds the
                    // live element/property by reference. A plain object binds each
                    // visible property by reference (ObjRefs); an array uses ByRef.
                    let src = self.frames[top].slots[source as usize].deref_clone();
                    // A by-ref foreach over a lazy object initializes it first
                    // (PHP 8.4); a proxy then iterates its real instance.
                    let src = if deref_object(&src).is_some_and(|o| o.borrow().lazy.is_some()) {
                        self.realize_full(&src)?
                    } else {
                        src
                    };
                    if let Zval::Object(o) = &src {
                        if !self.is_traversable(o.borrow().class_id as usize) {
                            let scope = self.frames[top].class;
                            self.frames[top].iters.push(IterState::ObjRefs { obj: src.clone(), scope, yielded: Vec::new() });
                            continue;
                        }
                    }
                    let keys = ref_array_keys(&self.frames[top].slots[source as usize]);
                    self.frames[top].iters.push(IterState::ByRef { source, keys, pos: 0 });
                }
                Op::IterNextRef { value, key, end } => {
                    // A plain-object by-ref foreach: bind `$v` to the property's
                    // storage cell — or, for a hooked property, to the cell its
                    // `&get` hook returns (a by-value get hook is a fatal). The
                    // entry list is recomputed live, like `IterState::ObjVals`.
                    if matches!(self.frames[top].iters.last(), Some(IterState::ObjRefs { .. })) {
                        let bound = loop {
                            let (obj, entry) = {
                                let Some(IterState::ObjRefs { obj, scope, yielded }) =
                                    self.frames[top].iters.last()
                                else {
                                    unreachable!("ObjRefs iterator")
                                };
                                let Zval::Object(o) = obj else { break None };
                                let next = self
                                    .object_iter_entries(o, *scope)
                                    .into_iter()
                                    .find(|(d, _)| !yielded.iter().any(|y| y == d));
                                let Some(next) = next else { break None };
                                (obj.clone(), next)
                            };
                            let (display, entry) = entry;
                            if let Some(IterState::ObjRefs { yielded, .. }) =
                                self.frames[top].iters.last_mut()
                            {
                                yielded.push(display.clone());
                            }
                            match entry {
                                PropIterEntry::Slot { key: k } => {
                                    if let Some(o) = deref_object(&obj) {
                                        if o.borrow().props.get(&k).is_some() {
                                            let cell = prop_ref_cell(&o, &k);
                                            // A typed property's cell keeps
                                            // enforcing its type through `$v`.
                                            let cid = o.borrow().class_id as usize;
                                            if let Some((decl, hint)) = prop_type_decl(&self.classes, cid, &display) {
                                                self.register_typed_ref(&cell, &o, decl, &display, hint);
                                            }
                                            break Some((cell, display));
                                        }
                                    }
                                }
                                PropIterEntry::Hook { name, view } => {
                                    let by_ref = self
                                        .prop_hook_in(view, &name)
                                        .is_some_and(|f| f.by_ref);
                                    if !by_ref {
                                        // A by-value get hook has no addressable
                                        // storage to alias (PHP 8.4 wording).
                                        let decl = deref_object(&obj)
                                            .and_then(|o| prop_info(&self.classes, o.borrow().class_id as usize, &name))
                                            .map(|pi| pi.declaring_class)
                                            .unwrap_or(view);
                                        return Err(PhpError::Error(format!(
                                            "Cannot create reference to property {}::${}",
                                            String::from_utf8_lossy(&self.classes[decl].name),
                                            String::from_utf8_lossy(&name),
                                        )));
                                    }
                                    let v = self.run_iter_get_hook(&obj, &name, view, false)?;
                                    let cell = match v {
                                        Zval::Ref(rc) => rc,
                                        other => Rc::new(RefCell::new(other)),
                                    };
                                    break Some((cell, display));
                                }
                            }
                        };
                        match bound {
                            None => self.frames[top].ip = end as usize,
                            Some((cell, keyname)) => {
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[ks as usize], Zval::Str(PhpStr::new(keyname.to_vec())));
                                }
                                self.frames[top].slots[value as usize] = Zval::Ref(cell);
                            }
                        }
                        continue;
                    }
                    let next = {
                        let it = self.frames[top].iters.last_mut().expect("IterNextRef without iterator");
                        let IterState::ByRef { source, keys, pos } = it else {
                            unreachable!("IterNextRef on a by-value iterator");
                        };
                        if *pos >= keys.len() {
                            None
                        } else {
                            let k = keys[*pos].clone();
                            let src = *source;
                            *pos += 1;
                            Some((src, k))
                        }
                    };
                    match next {
                        None => self.frames[top].ip = end as usize,
                        Some((src, k)) => {
                            let cell = elem_cell(&mut self.frames[top].slots[src as usize], &k);
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], key_to_zval(&k));
                            }
                            // Direct overwrite, *not* `store_slot`: on later
                            // iterations the value slot is itself a `Zval::Ref` to
                            // the previous element, and writing through it would
                            // corrupt that element (D-R13).
                            self.frames[top].slots[value as usize] = Zval::Ref(cell);
                        }
                    }
                }
                Op::IterPop => {
                    // The iterator (and the object it holds, e.g. `foreach (new A
                    // as $v)`) is dropped here; note what it releases.
                    if let Some(it) = self.frames[top].iters.pop() {
                        self.gc_note_iter(&it);
                    }
                }
                Op::DeclareFn { func } => {
                    // A conditional `function` statement was reached: register it in
                    // the runtime table so it is callable by name from here on.
                    let m = self.frames[top].module;
                    let idx = func as usize;
                    let name = m.functions[idx].name.clone();
                    if self.is_name_callable(&name) {
                        return Err(PhpError::Error(format!(
                            "Cannot redeclare function {}()",
                            String::from_utf8_lossy(&name)
                        )));
                    }
                    self.linked_functions.insert(name.to_ascii_lowercase(), (m, idx));
                }
                Op::DeclareTrait { idx } => {
                    // A conditional trait declaration executed: register the
                    // lowered trait into the seed image so later units can
                    // `use` it (only the branch that RAN registers its variant).
                    let module = self.frames[top].module;
                    if let Some((key, lt)) = module.conditional_traits.get(idx as usize) {
                        if !self.seed_traits.iter().any(|(k, _)| k == key) {
                            self.seed_traits.push((key.clone(), lt.clone()));
                        }
                    }
                }
                Op::DeclareClass { class } => {
                    // A conditional class/interface/enum statement was reached:
                    // register its name in the runtime class index so it resolves by
                    // name (and `class_exists` reports it) from here on. The op's id
                    // was already relocated to the global table, so it indexes
                    // `self.classes` directly. A name already in use is the PHP fatal.
                    let cid = class;
                    let cc = self.classes[cid];
                    let key = cc.name.to_ascii_lowercase();
                    if self.class_index.contains_key(&key) {
                        let kind = if cc.enum_cases.is_empty() { "class" } else { "enum" };
                        return Err(PhpError::Error(format!(
                            "Cannot declare {} {}, because the name is already in use",
                            kind,
                            String::from_utf8_lossy(&cc.name)
                        )));
                    }
                    self.class_index.insert(key, cid);
                    self.serializable_link_check(cid)?;
                }
                Op::Call { func, argc } => {
                    let m = self.frames[top].module;
                    let callee = &m.functions[func as usize];
                    // Pop argc args (pushed left-to-right) and bind them to the
                    // callee's leading slots. The caller's `ip` is already past
                    // the Call, so it resumes correctly once the callee returns.
                    let n = argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("call argument"));
                    }
                    args.reverse();
                    let mut frame = Frame::new(callee, m);
                    bind_params(&mut frame, args);
                    self.enter_callee(frame)?;
                }
                Op::CallArgs { func } => {
                    // Spread call `f(...$arr)` (PAR): integer keys bind positionally
                    // (variadic/defaults compose via the binder), string keys as
                    // named arguments (PHP 8.1).
                    let argsval = self.frames[top].stack.pop().expect("CallArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let m = self.frames[top].module;
                    let callee = &m.functions[func as usize];
                    let frame = if named.is_empty() {
                        let mut frame = Frame::new(callee, m);
                        bind_params(&mut frame, args);
                        frame
                    } else {
                        let qn = String::from_utf8_lossy(&callee.name).into_owned();
                        build_named_frame(callee, m, &qn, args, named)?
                    };
                    self.enter_callee(frame)?;
                }
                Op::CallNamed { func, positional, names } => {
                    // Named function call bound at run time (unknown/overwrite/
                    // variadic/by-ref): pop named values (source order), then the
                    // positional values, and bind via `build_named_frame`.
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let named: Vec<(Box<[u8]>, Zval)> =
                        names.iter().cloned().zip(named_vals).collect();
                    let pos = self.pop_keys(top, positional);
                    let m = self.frames[top].module;
                    let callee = &m.functions[func as usize];
                    let qn = String::from_utf8_lossy(&callee.name).into_owned();
                    let frame = build_named_frame(callee, m, &qn, pos, named)?;
                    self.enter_callee(frame)?;
                }
                Op::CallSpread { func, spreads, names } => {
                    // Pop explicit named values (source order), then one value per
                    // leading component (a positional value or a spread source).
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let comp_vals = self.pop_keys(top, spreads.len() as u32);
                    let mut positional: Vec<Zval> = Vec::new();
                    let mut named: Vec<(Box<[u8]>, Zval)> = Vec::new();
                    let mut seen_named = false;
                    for (&is_spread, val) in spreads.iter().zip(comp_vals) {
                        if is_spread {
                            // Integer keys are positional, string keys named; a
                            // positional after a named (within the unpacking) is an
                            // error, a non-iterable a TypeError.
                            for (k, v) in self.spread_pairs(val)? {
                                match k {
                                    Key::Int(_) => {
                                        if seen_named {
                                            return Err(PhpError::Error("Cannot use positional argument after named argument during unpacking".to_string()));
                                        }
                                        positional.push(v);
                                    }
                                    Key::Str(s) => {
                                        named.push((s.as_bytes().to_vec().into_boxed_slice(), v));
                                        seen_named = true;
                                    }
                                }
                            }
                        } else {
                            if seen_named {
                                return Err(PhpError::Error("Cannot use positional argument after named argument".to_string()));
                            }
                            positional.push(val);
                        }
                    }
                    // Explicit named args always come last, so no positional can
                    // follow — no need to track `seen_named` past here.
                    for (label, v) in names.iter().cloned().zip(named_vals) {
                        named.push((label, v));
                    }
                    let m = self.frames[top].module;
                    let callee = &m.functions[func as usize];
                    let qn = String::from_utf8_lossy(&callee.name).into_owned();
                    let frame = build_named_frame(callee, m, &qn, positional, named)?;
                    self.enter_callee(frame)?;
                }
                Op::CallBuiltin { name, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::Value(f)) => *f,
                        // The compiler only emits CallBuiltin for value builtins.
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let mut args = self.pop_keys(top, argc); // pops argc, source order
                    // A whole-object exporter initializes a lazy argument first
                    // (PHP 8.4, init_trigger_var_export); the pure builtin then
                    // formats the realized instance.
                    if matches!(&name[..], b"var_export" | b"print_r") {
                        for a in &mut args {
                            if self.is_lazy_value(a) {
                                *a = self.realize_full(a)?;
                            }
                        }
                    }
                    // `var_dump` calls each debuggable object's `__debugInfo()`
                    // (PHP 8.4) *before* rendering — a lazy object initializes only
                    // if that method touches its state — and dumps the returned
                    // array under the object header. Results handed to the builtin
                    // via `var_dump_debug` (taken in `run_value_builtin`).
                    if name[..] == *b"var_dump" {
                        self.var_dump_debug = self.compute_debug_info(&args)?;
                    }
                    // `count($obj)`/`sizeof($obj)` on a Countable dispatches its
                    // user `count()` method (step 56); the builtin only handles
                    // arrays. A non-Countable object still TypeErrors in the
                    // builtin below, matching PHP.
                    if argc == 1
                        && (name[..] == *b"count" || name[..] == *b"sizeof")
                    {
                        if let Some(obj) = self.as_countable(&args[0]) {
                            let n = self.call_method_sync(obj, b"count", Vec::new())?;
                            self.frames[top].stack.push(n);
                            continue;
                        }
                    }
                    // A builtin that unconditionally string-coerces its (string)
                    // arguments gets each Stringable object's `__toString()`
                    // precomputed, so the pure builtin honors it (handed over via
                    // `stringify_args`, taken in `run_value_builtin`). The `deep`
                    // family (`implode`/`str_replace`) also coerces array
                    // *elements*, so its precompute recurses into array arguments.
                    if value_builtin_string_coerces(&name) {
                        self.stringify_args = self.compute_stringify(&args, false)?;
                    } else if value_builtin_string_coerces_deep(&name) {
                        self.stringify_args = self.compute_stringify(&args, true)?;
                    }
                    let line = self.cur_line(top);
                    let result = self.run_value_builtin(f, &args, line)?;
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinSpread { name, spreads } => {
                    // A registry builtin runs via its Value fn; a VM host
                    // builtin (`json_encode(...$args)`, Elastica's stringify)
                    // takes the flattened args through dispatch_host_builtin.
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::Value(f)) => Some(*f),
                        _ if host_builtin_canonical(&name).is_some() => None,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let comp_vals = self.pop_keys(top, spreads.len() as u32);
                    // Flatten components to positional values; int-keyed unpacks are
                    // positional, string-keyed are named (rejected for a builtin).
                    let mut args: Vec<Zval> = Vec::new();
                    let mut seen_named = false;
                    for (&is_spread, val) in spreads.iter().zip(comp_vals) {
                        if is_spread {
                            for (k, v) in self.spread_pairs(val)? {
                                match k {
                                    Key::Int(_) => {
                                        if seen_named {
                                            return Err(PhpError::Error("Cannot use positional argument after named argument during unpacking".to_string()));
                                        }
                                        args.push(v);
                                    }
                                    Key::Str(_) => seen_named = true,
                                }
                            }
                        } else {
                            if seen_named {
                                return Err(PhpError::Error("Cannot use positional argument after named argument".to_string()));
                            }
                            args.push(val);
                        }
                    }
                    if seen_named {
                        return Err(PhpError::Error(format!(
                            "{}() does not accept unknown named parameters",
                            String::from_utf8_lossy(&name)
                        )));
                    }
                    let result = match f {
                        Some(f) => {
                            let line = self.cur_line(top);
                            self.run_value_builtin(f, &args, line)?
                        }
                        None => self.dispatch_host_builtin(&name, args)?,
                    };
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltin { name, argc } => {
                    // An evaluator-only host builtin (Session B): it may invoke a
                    // user callable via `call_callable` (a nested `run_loop`).
                    let args = self.pop_keys(top, argc);
                    // Flush this builtin's own diagnostics at its call line, like
                    // `run_value_builtin` does for registry builtins — otherwise a
                    // warning it pushes (e.g. header()'s "headers already sent")
                    // renders at a later, wrong line.
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let result = self.dispatch_host_builtin(&name, args)?;
                    self.flush_diags(line)?;
                    // `pcntl_async_signals(true)`: host-builtin returns are the
                    // async delivery points (a self-directed `posix_kill` is
                    // observed here, before the next PHP statement).
                    if self.async_signals
                        && PENDING_SIGNALS.load(std::sync::atomic::Ordering::Relaxed) != 0
                    {
                        self.dispatch_pending_signals()?;
                    }
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinRef { name, slot, argc } => {
                    // A by-reference-first host builtin (`usort`, Session C): its
                    // array argument lives in `slot` of the caller frame and is
                    // written back in place; the callback may run a nested `run_loop`.
                    let rest = self.pop_keys(top, argc);
                    let result = self.dispatch_host_builtin_ref(&name, slot, rest)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinOut { name, out_slot, out_index, out_slot2, out_index2, argc } => {
                    // A host builtin with a by-reference output parameter
                    // (`preg_match`/`preg_match_all`'s `&$matches`): dispatch with all
                    // args by value, then write the produced out-value into `out_slot`.
                    // `exec` additionally writes `&$result_code` into `out_slot2`.
                    let _ = out_index2;
                    let mut args = self.pop_keys(top, argc);
                    // `exec` *appends* to a pre-existing `&$output` array, so it
                    // needs that argument's current value (the compiler pushed a
                    // placeholder `null` there). Read it straight from the slot —
                    // no VarRead op, so an undefined variable warns nothing. Other
                    // out-param builtins ignore this argument.
                    if name[..] == *b"exec" {
                        if let Some(slot) = out_slot {
                            if let Some(a) = args.get_mut(out_index as usize) {
                                *a = self.frames[top].slots[slot as usize].deref_clone();
                            }
                        }
                    }
                    let (result, out_val, out_val2) =
                        self.dispatch_host_builtin_out(&name, args, out_index as usize)?;
                    let top = self.frames.len() - 1;
                    if let Some(slot) = out_slot {
                        match &mut self.frames[top].slots[slot as usize] {
                            Zval::Ref(rc) => *rc.borrow_mut() = out_val,
                            cell => *cell = out_val,
                        }
                    }
                    if let (Some(slot), Some(v2)) = (out_slot2, out_val2) {
                        match &mut self.frames[top].slots[slot as usize] {
                            Zval::Ref(rc) => *rc.borrow_mut() = v2,
                            cell => *cell = v2,
                        }
                    }
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinScanf { name, argc, out_slots } => {
                    // `sscanf`/`fscanf` with variadic by-reference out-params: dispatch
                    // the two fixed value args, then assign each conversion into its
                    // slot. With no out slots the parsed array is returned instead.
                    let args = self.pop_keys(top, argc);
                    let scanned = self.dispatch_host_builtin_scanf(&name, args)?;
                    let top = self.frames.len() - 1;
                    let result = match scanned {
                        // `fscanf` at EOF: `false`, no assignments.
                        None => Zval::Bool(false),
                        Some(results) if out_slots.is_empty() => {
                            let mut arr = PhpArray::new();
                            for v in results {
                                let _ = arr.append(v.unwrap_or(Zval::Null));
                            }
                            Zval::Array(Rc::new(arr))
                        }
                        Some(results) => {
                            // Iterate over results (matching `eval::scanf_finish`): out
                            // vars beyond the result count are left unchanged.
                            let mut count = 0i64;
                            for (i, slot) in results.iter().enumerate() {
                                let Some(out) = out_slots.get(i) else { break };
                                let val = match slot {
                                    Some(v) => {
                                        count += 1;
                                        v.clone()
                                    }
                                    None => Zval::Null,
                                };
                                if let Some(s) = out {
                                    match &mut self.frames[top].slots[*s as usize] {
                                        Zval::Ref(rc) => *rc.borrow_mut() = val,
                                        cell => *cell = val,
                                    }
                                }
                            }
                            Zval::Long(count)
                        }
                    };
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRef { name, slot, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let rest = self.pop_keys(top, argc);
                    // Mirror `eval`'s ref-builtin rendering (E1): flush, run, append
                    // the builtin's output, then flush its own warnings.
                    // A by-value object argument reads through an initialized
                    // proxy (an uninitialized wrapper keeps its raw view —
                    // array_splice's internal to-array, convert_to_array).
                    let rest: Vec<Zval> = rest
                        .into_iter()
                        .map(|a| if self.is_lazy_value(&a) { self.proxy_view(a) } else { a })
                        .collect();
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    // Precompute __toString for a natural-sort's Stringable
                    // elements so the pure comparator can order them (empty for
                    // every other ref builtin).
                    let stringify = if ref_builtin_string_coerces(&name) {
                        let roots = ref_stringify_roots(&self.frames[top].slots[slot as usize].clone());
                        self.compute_stringify(&roots, false)?
                    } else {
                        std::collections::HashMap::new()
                    };
                    let mut produced = Vec::new();
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[slot as usize], &rest, &mut produced, &mut self.diags, &stringify);
                    self.write_output(&produced)?;
                    self.flush_diags(line)?;
                    let result = result?;
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRefSpread { name, slot, spreads } => {
                    // By-ref-first builtin whose by-value rest includes a spread
                    // (`array_push($a, ...$b)`): flatten the components (int-keyed
                    // pairs are positional; a string key is PHP's unknown-named
                    // error, as builtins accept none), then run like CallBuiltinRef.
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let comp_vals = self.pop_keys(top, spreads.len() as u32);
                    let mut rest: Vec<Zval> = Vec::new();
                    for (&is_spread, val) in spreads.iter().zip(comp_vals) {
                        if is_spread {
                            for (k, v) in self.spread_pairs(val)? {
                                match k {
                                    Key::Int(_) => rest.push(v),
                                    Key::Str(_) => {
                                        return Err(PhpError::Error(format!(
                                            "{}() does not accept unknown named parameters",
                                            String::from_utf8_lossy(&name)
                                        )))
                                    }
                                }
                            }
                        } else {
                            rest.push(val);
                        }
                    }
                    // A by-value object argument reads through an initialized
                    // proxy (an uninitialized wrapper keeps its raw view —
                    // array_splice's internal to-array, convert_to_array).
                    let rest: Vec<Zval> = rest
                        .into_iter()
                        .map(|a| if self.is_lazy_value(&a) { self.proxy_view(a) } else { a })
                        .collect();
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let stringify = if ref_builtin_string_coerces(&name) {
                        let roots = ref_stringify_roots(&self.frames[top].slots[slot as usize].clone());
                        self.compute_stringify(&roots, false)?
                    } else {
                        std::collections::HashMap::new()
                    };
                    let mut produced = Vec::new();
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[slot as usize], &rest, &mut produced, &mut self.diags, &stringify);
                    self.write_output(&produced)?;
                    self.flush_diags(line)?;
                    let result = result?;
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRefCell { name, argc } => {
                    // By-ref-first builtin on a non-variable place (`array_pop($o->q)`):
                    // the target is a `Zval::Ref` cell (from `MakeRef`) beneath the
                    // by-value rest args; mutate it in place, writing through to the
                    // property / array element.
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let rest = self.pop_keys(top, argc);
                    let cell = match self.frames[top].stack.pop().expect("CallBuiltinRefCell ref") {
                        Zval::Ref(rc) => rc,
                        other => Rc::new(RefCell::new(other)),
                    };
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let stringify = if ref_builtin_string_coerces(&name) {
                        let roots = ref_stringify_roots(&cell.borrow().clone());
                        self.compute_stringify(&roots, false)?
                    } else {
                        std::collections::HashMap::new()
                    };
                    let mut produced = Vec::new();
                    let result = {
                        let mut leaf = cell.borrow_mut();
                        builtin_ref_call(f, &mut leaf, &rest, &mut produced, &mut self.diags, &stringify)
                    };
                    self.write_output(&produced)?;
                    self.flush_diags(line)?;
                    let result = result?;
                    self.frames[top].stack.push(result);
                }
                Op::Ret => {
                    let mut ret = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    // Coerce the returned value to a scalar return hint (weak, or
                    // checked under strict_types) — step 14. A by-reference function
                    // returns an alias, so its return type stays unenforced; the
                    // init-thunk / magic path (`ret_cell`) carries no hint either.
                    let func = self.frames[top].func;
                    // A generator function's declared type (`: Generator`/`iterable`)
                    // describes the returned *generator*, not its internal `return`
                    // value — so it is never checked here (the body's `return` sets
                    // `getReturn`).
                    if let Some(hint) = func.ret_hint.clone().filter(|_| !func.is_generator) {
                        if !func.by_ref && self.frames[top].ret_cell.is_none() {
                            // The function's own unit governs its return check.
                            let strict = self.frames[top].module.strict;
                            match self.coerce_or_check_hint(ret, &hint, strict) {
                                Ok(c) => ret = c,
                                Err(given) => {
                                    return Err(self.return_type_error(func, &hint, &given))
                                }
                            }
                        }
                    }
                    let ret_cell = self.frames[top].ret_cell.take();
                    let ret_bool = self.frames[top].ret_bool;
                    let ret_isset = self.frames[top].ret_isset;
                    let ret_stringify = self.frames[top].ret_stringify;
                    let ret_deref = self.frames[top].ret_deref;
                    let guard = std::mem::take(&mut self.frames[top].guard_release);
                    // A `clone`-driven `__clone` is finishing: revoke any remaining
                    // readonly re-init permission on the copy (PHP 8.3), so writes
                    // after the clone — or via a manual `__clone()` — fatal again.
                    if self.frames[top].clone_init {
                        if let Some(Zval::Object(o)) = self.frames[top].this.clone() {
                            o.borrow_mut().readonly_clone_writable.clear();
                        }
                    }
                    let dead = self.frames.pop().expect("Ret pops the active frame");
                    // The returning frame's locals, leftover operands and `$this`
                    // release their references now: note any tracked objects so
                    // the next sweep reconsiders them (drives destruction of an
                    // object whose last reference was a returning function's local).
                    self.gc_note_frame(&dead);
                    drop(dead);
                    for key in guard {
                        self.magic_guard.remove(&key);
                    }
                    if let Some(cell) = ret_cell {
                        // Init thunk / discarded magic return: store into the cell;
                        // the caller already has (or re-reads) its own value.
                        *cell.borrow_mut() = ret;
                    } else {
                        let v = if ret_isset {
                            Zval::Bool(!matches!(ret.deref_clone(), Zval::Null))
                        } else if ret_bool {
                            Zval::Bool(convert::to_bool(&ret, &mut self.diags))
                        } else if ret_stringify {
                            Zval::Str(convert::to_zstr(&ret, &mut self.diags))
                        } else if ret_deref {
                            ret.deref_clone()
                        } else {
                            ret
                        };
                        // The frame that owned this bounded run has returned: hand
                        // the value back to whoever started it (the host, for the
                        // top-level run; `resume_generator`, for a generator body).
                        if self.frames.len() == baseline {
                            return Ok(RunExit::Returned(v));
                        }
                        self.frames
                            .last_mut()
                            .expect("a non-baseline Ret has a caller")
                            .stack
                            .push(v);
                    }
                }
                Op::Yield { has_key } => {
                    // Suspend the running generator frame (GEN). Pop the yielded
                    // value (and key), park the frame back under its handle id, and
                    // hand the key/value to `resume_generator`. `ip` is already
                    // past this op, so the resume continues after the `yield`.
                    let value = self.frames[top].stack.pop().expect("Yield value");
                    let key = if has_key {
                        GenKey::Keyed(self.frames[top].stack.pop().expect("Yield key"))
                    } else {
                        GenKey::Auto
                    };
                    let gid = self.frames[top]
                        .gen_id
                        .expect("Yield outside a generator frame");
                    debug_assert_eq!(top, baseline, "a generator yields at its own baseline");
                    let frame = self.frames.pop().expect("generator frame to park");
                    self.generators.insert(gid, frame);
                    return Ok(RunExit::Yielded { key, value });
                }
                Op::YieldFrom => {
                    // `yield from` (GEN-3): re-enters itself across resumes, driving
                    // one delegated step per visit. First visit sets up the cursor
                    // from the delegate on the stack; a re-visit pops the resume's
                    // sent value (forwarded into a sub-generator, ignored by arrays).
                    if self.frames[top].yield_from.is_none() {
                        let delegate = self.frames[top].stack.pop().expect("YieldFrom delegate");
                        match delegate.deref_clone() {
                            Zval::Array(_) => {
                                let entries = snapshot_entries(&delegate);
                                self.frames[top].yield_from =
                                    Some(YieldFromState::Array { entries, pos: 0 });
                            }
                            Zval::Generator(rc) => {
                                self.frames[top].yield_from =
                                    Some(YieldFromState::Gen { rc: Rc::clone(&rc), opaque: false });
                                self.ensure_started(&rc)?; // prime to its first yield
                            }
                            Zval::Object(_) if self.object_is_traversable(&delegate) => {
                                // A Traversable delegate (Symfony Finder pipes
                                // FilterIterators through `yield from`): resolve
                                // IteratorAggregate to its Iterator, then drain
                                // the protocol synchronously into entries.
                                // Divergence: PHP interleaves the delegate's
                                // steps with the consumer; phpr snapshots — the
                                // outcome matches for finite side-effect-free
                                // iterators (an infinite Iterator would hang
                                // here instead of streaming).
                                match self.traversable_entries(delegate.clone())? {
                                    TraversableSource::Gen(rc) => {
                                        self.frames[top].yield_from =
                                            Some(YieldFromState::Gen { rc: Rc::clone(&rc), opaque: true });
                                        self.ensure_started(&rc)?;
                                    }
                                    TraversableSource::Entries(entries) => {
                                        self.frames[top].yield_from =
                                            Some(YieldFromState::Array { entries, pos: 0 });
                                    }
                                }
                            }
                            other => {
                                return Err(PhpError::Error(format!(
                                    "Can use \"yield from\" only with arrays and Traversables, {} given",
                                    other.type_name_for_error()
                                )))
                            }
                        }
                    } else {
                        // Re-entry from a resume: the sent value is on the stack.
                        let sent = self.frames[top].stack.pop().expect("YieldFrom sent");
                        let sub = match &self.frames[top].yield_from {
                            Some(YieldFromState::Gen { rc, .. }) => Some(Rc::clone(rc)),
                            _ => None,
                        };
                        if let Some(rc) = sub {
                            self.resume_generator(&rc, sent)?;
                        }
                    }
                    // Take the next delegated `(key, value)`, or finish.
                    let step = match self.frames[top].yield_from.as_mut().unwrap() {
                        YieldFromState::Array { entries, pos } => {
                            if *pos < entries.len() {
                                let pair = entries[*pos].clone();
                                *pos += 1;
                                Some(pair)
                            } else {
                                None
                            }
                        }
                        YieldFromState::Gen { rc, .. } => {
                            let g = rc.borrow();
                            if matches!(g.status, GenStatus::Done) {
                                None
                            } else {
                                Some((g.cur_key.clone(), g.cur_val.clone()))
                            }
                        }
                    };
                    match step {
                        Some((k, v)) => {
                            // Re-enter this op on the next resume; park and re-yield
                            // verbatim (the outer auto-key counter is untouched).
                            self.frames[top].ip -= 1;
                            let gid =
                                self.frames[top].gen_id.expect("YieldFrom outside a generator");
                            let frame = self.frames.pop().expect("generator frame to park");
                            self.generators.insert(gid, frame);
                            return Ok(RunExit::Yielded { key: GenKey::Verbatim(k), value: v });
                        }
                        None => {
                            // Delegation done: leave the delegate's return value (NULL
                            // for an array, the sub-generator's getReturn()) on the
                            // stack as the `yield from` expression's value.
                            let value = match self.frames[top].yield_from.take().unwrap() {
                                YieldFromState::Array { .. } => Zval::Null,
                                // An aggregate-wrapped generator's return stays
                                // opaque (PHP: only a DIRECT generator delegate
                                // propagates getReturn()).
                                YieldFromState::Gen { opaque: true, .. } => Zval::Null,
                                YieldFromState::Gen { rc, .. } => rc.borrow().ret.clone(),
                            };
                            self.frames[top].stack.push(value);
                        }
                    }
                }
                Op::Alloc { class } => {
                    let obj = self.alloc_object(class)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocStatic => {
                    let cid = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                    })?;
                    let obj = self.alloc_object(cid)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocDynamic => {
                    // `new $cls` (PAR): resolve the class reference at run time.
                    let classval = self.frames[top].stack.pop().expect("AllocDynamic class");
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let obj = self.alloc_object(cid)?;
                    self.frames[top].stack.push(obj);
                }
                Op::StampThrowable => {
                    // Stamp line/file/trace on a `new`-constructed Throwable, after
                    // its property-init thunk ran (which would otherwise clobber
                    // `trace`), leaving the object on the stack (EXC-3b/3c).
                    if let Some(obj) = self.frames[top].stack.last().cloned() {
                        self.stamp_throwable_location(&obj);
                    }
                }
                Op::This => match &self.frames[top].this {
                    Some(t) => {
                        let v = t.clone();
                        self.frames[top].stack.push(v);
                    }
                    None => {
                        return Err(PhpError::Error(
                            "Using $this when not in object context".to_string(),
                        ))
                    }
                },
                Op::Eval => {
                    let code = self.frames[top].stack.pop().expect("eval code");
                    let code_str = convert::to_zstr(&code, &mut self.diags);
                    let mut src = b"<?php ".to_vec();
                    src.extend_from_slice(code_str.as_bytes());
                    let result = self.run_eval(&src)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::Include { mode } => {
                    let path_val = self.frames[top].stack.pop().expect("include path");
                    let result = self.run_include(path_val, mode)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::Clone => {
                    let src = self.frames[top].stack.pop().expect("Clone operand").deref_clone();
                    let Zval::Object(o) = &src else {
                        return Err(PhpError::TypeError(format!(
                            "clone(): Argument #1 ($object) must be of type object, {} given",
                            src.type_name_for_error()
                        )));
                    };
                    // Cloning a lazy object initializes it first (PHP 8.4,
                    // clone_initializes). An initialized proxy then clones as a
                    // NEW initialized proxy wrapping a clone of its real
                    // instance: the copy below runs on the instance and the
                    // wrapper is rebuilt around it afterwards.
                    if o.borrow().lazy.is_some() && o.borrow().proxy_instance.is_none() {
                        self.realize_lazy(&src)?;
                    }
                    let proxy_wrapper: Option<(u32, Rc<PhpStr>, Rc<ObjectInfo>)> = {
                        let b = o.borrow();
                        if matches!(b.lazy, Some(LazyKind::Proxy)) && b.proxy_instance.is_some() {
                            Some((b.class_id, Rc::clone(&b.class_name), Rc::clone(&b.info)))
                        } else {
                            None
                        }
                    };
                    let src = if proxy_wrapper.is_some() {
                        let inst = o.borrow().proxy_instance.clone().expect("initialized proxy");
                        (*inst).clone()
                    } else {
                        src
                    };
                    let Zval::Object(o) = &src else { unreachable!("proxy instance is an object") };
                    // Shallow copy: a fresh handle, properties cloned by value
                    // (nested objects share their handle, arrays copy on write).
                    let clone_rc = {
                        let b = o.borrow();
                        let mut props = b.props.clone();
                        // Detach object-internal (unshared) property references so
                        // the copy does not write back through them (bug27268/68262).
                        props.separate_cloned_internal_refs();
                        let obj = Object {
                            class_id: b.class_id,
                            class_name: Rc::clone(&b.class_name),
                            props,
                            id: self.next_id(),
                            info: Rc::clone(&b.info),
                            // A clone keeps the source's readonly props initialised.
                            readonly_init: b.readonly_init.clone(),
                            // Granted below if `__clone` runs (one re-init each).
                            readonly_clone_writable: Vec::new(),
                            // A clone is a concrete copy; lazy state is not carried.
                            lazy: None,
                            proxy_instance: None,
                        };
                        Rc::new(RefCell::new(obj))
                    };
                    self.created
                        .insert(clone_rc.borrow().id, Rc::clone(&clone_rc));
                    self.gc_track(&clone_rc);
                    // A clone inherits typed references (typed_properties_081):
                    // its property slots share the source's reference cells, so
                    // the copy becomes an additional owner of each registered
                    // typed source (the type outlives the original object).
                    if !self.typed_refs.is_empty() {
                        let src_ptr = Rc::as_ptr(o);
                        let inherited: Vec<TypedRefSource> = self
                            .typed_refs
                            .iter()
                            .filter(|t| {
                                t.cell.strong_count() > 0 && std::ptr::eq(t.obj.as_ptr(), src_ptr)
                            })
                            .map(|t| TypedRefSource {
                                cell: t.cell.clone(),
                                obj: Rc::downgrade(&clone_rc),
                                class_name: t.class_name.clone(),
                                prop: t.prop.clone(),
                                hint: t.hint.clone(),
                            })
                            .collect();
                        self.typed_refs.extend(inherited);
                    }
                    let cid = clone_rc.borrow().class_id as usize;
                    let clone_val = Zval::Object(clone_rc.clone());
                    // A proxy clones to a fresh initialized-proxy wrapper around
                    // the instance copy; `__clone` below still runs on the copy.
                    let pushed = if let Some((wcid, wname, winfo)) = proxy_wrapper {
                        let wrapper = Object {
                            class_id: wcid,
                            class_name: wname,
                            props: Props::new(),
                            id: self.next_id(),
                            info: winfo,
                            readonly_init: Vec::new(),
                            readonly_clone_writable: Vec::new(),
                            lazy: Some(LazyKind::Proxy),
                            proxy_instance: Some(Box::new(clone_val.clone())),
                        };
                        let wrc = Rc::new(RefCell::new(wrapper));
                        self.created.insert(wrc.borrow().id, Rc::clone(&wrc));
                        self.gc_track(&wrc);
                        Zval::Object(wrc)
                    } else {
                        clone_val.clone()
                    };
                    self.frames[top].stack.push(pushed);
                    // Run `__clone` on the copy if defined (return discarded), so it
                    // can deep-copy what it needs (PHP OOP).
                    if let Some((defc, midx)) = resolve_method_runtime(&self.classes, cid, b"__clone") {
                        // PHP 8.3 amendment: while `__clone` runs, each readonly
                        // property of the copy may be re-initialised once. Grant the
                        // permission now; the `__clone` frame revokes it on return so
                        // a manual `$o->__clone()` (no grant) still fatals on a write.
                        let writable: Vec<Box<[u8]>> = clone_rc
                            .borrow()
                            .props
                            .iter()
                            .map(|(n, _)| n.to_vec().into_boxed_slice())
                            .filter(|n| prop_readonly_decl(&self.classes, cid, n).is_some())
                            .collect();
                        clone_rc.borrow_mut().readonly_clone_writable = writable;
                        let callee = &self.classes[defc].methods[midx].func;
                        let mut frame = Frame::new(callee, self.class_mod(defc));
                        frame.this = Some(clone_val);
                        frame.class = Some(defc);
                        frame.static_class = Some(cid);
                        frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
                        frame.clone_init = true;
                        self.frames.push(frame);
                        continue;
                    }
                }
                Op::PropGet { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropGet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    // A read of a lazy object initializes it first (PHP 8.4) —
                    // unless a hook/`__get` serves it; an initialized proxy then
                    // forwards the read to its real instance (transitively).
                    let target = self.lazy_prop_access(target, &name, cur, Some(false), (MagicKind::Get, b"__get"))?;
                    // Storage slot to read (the plain name for a dynamic/non-object
                    // target; a mangled key for an accessible private — set below).
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        // A `get` hook takes precedence over `__get` and direct read
                        // (step 50). Skip it while a hook for this property is active
                        // (a backing read inside the hook).
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                continue;
                            }
                            // A virtual hooked property with no get hook is write-only.
                            if self.is_virtual_hooked(cid, &name) {
                                return Err(PhpError::Error(format!(
                                    "Property {}::${} is write-only",
                                    String::from_utf8_lossy(&self.classes[cid].name),
                                    String::from_utf8_lossy(&name),
                                )));
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            // __get's return *is* the read result (flows via Ret).
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                        check_prop_access(&self.classes, cur, o.borrow().class_id as usize, &name)?;
                        key = self.prop_storage_key(o.borrow().class_id as usize, &name, cur);
                        if let Some(err) = self.uninit_typed_read(o, &key, &name) {
                            return Err(err);
                        }
                    }
                    let v = read_property(&target, &key, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::PropGetSilent { name } => {
                    // Like PropGet but with no "Undefined property" warning and no
                    // visibility error (the read context of `empty()` / `??`).
                    let obj = self.frames[top].stack.pop().expect("PropGetSilent object");
                    let cur = self.frames[top].class;
                    // A silent read (`??`/`empty`) still initializes a lazy object
                    // (PHP 8.4, fetch_coalesce_initializes) — unless a hook/`__get`
                    // serves it; an initialized proxy forwards to its instance.
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Get, b"__get"))?;
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                        key = self.prop_storage_key(o.borrow().class_id as usize, &name, cur);
                    }
                    let mut sink = Diags::new();
                    let v = read_property(&target, &key, &mut sink);
                    self.frames[top].stack.push(v);
                }
                Op::PropGetDynamic => {
                    // `$o->$n` / `$o->{expr}`: pop the property name (coerced to a
                    // string) then read exactly like `Op::PropGet` (step 51).
                    let nameval = self.frames[top].stack.pop().expect("PropGetDynamic name");
                    let name = convert::to_zstr(&nameval, &mut self.diags).as_bytes().to_vec();
                    let obj = self.frames[top].stack.pop().expect("PropGetDynamic object");
                    let cur = self.frames[top].class;
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Get, b"__get"))?;
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                continue;
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                        check_prop_access(&self.classes, cur, o.borrow().class_id as usize, &name)?;
                        key = self.prop_storage_key(o.borrow().class_id as usize, &name, cur);
                        if let Some(err) = self.uninit_typed_read(o, &key, &name) {
                            return Err(err);
                        }
                    }
                    let v = read_property(&target, &key, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::PropGetDynamicSilent => {
                    // Like `Op::PropGetDynamic` but silent (the `??` read context).
                    let nameval = self.frames[top].stack.pop().expect("PropGetDynamicSilent name");
                    let name = convert::to_zstr(&nameval, &mut self.diags).as_bytes().to_vec();
                    let obj = self.frames[top].stack.pop().expect("PropGetDynamicSilent object");
                    let cur = self.frames[top].class;
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Get, b"__get"))?;
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                        key = self.prop_storage_key(o.borrow().class_id as usize, &name, cur);
                    }
                    let mut sink = Diags::new();
                    let v = read_property(&target, &key, &mut sink);
                    self.frames[top].stack.push(v);
                }
                Op::PropSet { name } => {
                    let mut value = self.frames[top].stack.pop().expect("PropSet value");
                    let obj = self.frames[top].stack.pop().expect("PropSet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    // A write to a lazy object initializes it first (PHP 8.4) —
                    // unless a set hook/`__set` serves it; a no-op during the
                    // object's own construction (it is no longer lazy then).
                    // Proxy forwarding is transitive (a reset instance re-triggers).
                    let target = if !self.frames[top].init_props {
                        self.lazy_prop_access(target, &name, cur, Some(true), (MagicKind::Set, b"__set"))?
                    } else {
                        target
                    };
                    // A `prop_init` thunk writes defaults directly: no `__set`, no
                    // visibility check (so a subclass can set an inherited private).
                    // The slot is the declared one (unconditional, mangled for a
                    // private) regardless of the running scope.
                    if self.frames[top].init_props {
                        let key = match object_class_id(&target) {
                            Some(ocid) => self.prop_decl_storage_key(ocid, &name),
                            None => name.to_vec(),
                        };
                        if let Some(old) = write_property(&target, &key, value.clone())? {
                            self.gc_note(&old);
                        }
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    // Storage slot to write (plain name for a dynamic/non-object
                    // target; a mangled key for an accessible private — set below).
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        // An enum case is immutable: every property is readonly and
                        // no dynamic property may be created (step 23).
                        {
                            let ob = o.borrow();
                            if ob.info.is_enum_case {
                                let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                                let prop = String::from_utf8_lossy(&name).into_owned();
                                return Err(PhpError::Error(if ob.props.contains(&name) {
                                    format!("Cannot modify readonly property {cls}::${prop}")
                                } else {
                                    format!("Cannot create dynamic property {cls}::${prop}")
                                }));
                            }
                        }
                        // A `set` hook takes precedence over `__set` and direct write
                        // (step 50); skipped while a hook for this property is active
                        // (a backing write inside the hook). The expression still
                        // yields the assigned value; the hook's own return is dropped.
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, true) {
                                self.frames[top].stack.push(value.clone());
                                self.push_hook(func, target.clone(), oid, &name, Some(value));
                                continue;
                            }
                            // A virtual hooked property with no set hook is read-only.
                            if self.is_virtual_hooked(cid, &name) {
                                return Err(PhpError::Error(format!(
                                    "Property {}::${} is read-only",
                                    String::from_utf8_lossy(&self.classes[cid].name),
                                    String::from_utf8_lossy(&name),
                                )));
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Set, b"__set")
                        {
                            // The expression yields the assigned value; __set's own
                            // return is discarded into a throwaway cell.
                            self.frames[top].stack.push(value.clone());
                            let discard = Rc::new(RefCell::new(Zval::Null));
                            self.push_magic_prop(defc, midx, oid, MagicKind::Set, target.clone(), &name, Some(value), Some(discard), false);
                            continue;
                        }
                        check_prop_access(&self.classes, cur, o.borrow().class_id as usize, &name)?;
                        let ocid = o.borrow().class_id as usize;
                        // Storage slot (mangled for an accessible private); per-instance
                        // state (props, readonly tracking) is keyed by it. `Denied`
                        // already errored above, so the write is either a declared
                        // slot or a dynamic creation — readonly / typed enforcement
                        // applies only to the former (a parent's private reached from
                        // a child scope is a *dynamic* write, untyped and unguarded).
                        let access = resolve_prop_access(&self.classes, ocid, &name, cur);
                        let declared_slot = matches!(access, PropAccess::Slot(_));
                        key = match access {
                            PropAccess::Slot(k) => k,
                            _ => name.to_vec(),
                        };
                        // Readonly write-once enforcement (after the visibility check,
                        // so a private/protected readonly reports the access error
                        // first, matching PHP). A permitted first initialisation is
                        // recorded so any later write fatals.
                        if declared_slot {
                            if let Some(decl) = prop_readonly_decl(&self.classes, ocid, &name) {
                                if o.borrow().readonly_clone_writable(&key) {
                                    // Permitted re-initialisation during `__clone` (8.3).
                                    let mut ob = o.borrow_mut();
                                    ob.consume_clone_writable(&key);
                                    ob.mark_readonly_init(&key);
                                } else {
                                    let inited = o.borrow().is_readonly_init(&key);
                                    if let Some(err) = readonly_write_error(&self.classes, cur, decl, &name, inited) {
                                        return Err(err);
                                    }
                                    o.borrow_mut().mark_readonly_init(&key);
                                }
                            }
                        }
                        // PHP 8.2: creating an *undeclared* property on a class that
                        // does not allow dynamic properties is deprecated (the
                        // property is still created). `__set`/hooks already
                        // short-circuited above. A *declared* property that was
                        // `unset` is absent from the store yet is not a dynamic
                        // creation when re-assigned — `declared_slot` covers it.
                        if !declared_slot
                            && !o.borrow().props.contains(&key)
                            && !self.allows_dynamic_props(ocid)
                        {
                            let cls = String::from_utf8_lossy(&self.classes[ocid].name).into_owned();
                            let prop = String::from_utf8_lossy(&name).into_owned();
                            self.diags.push(Diag::Deprecated(format!(
                                "Creation of dynamic property {cls}::${prop} is deprecated"
                            )));
                        }
                        // Typed-property write enforcement: coerce the value to the
                        // property's declared type (or TypeError). The assignment
                        // expression yields the coerced value.
                        if declared_slot {
                            value = self.coerce_typed_prop_write(ocid, &name, value)?;
                        }
                    }
                    // A declared slot written on a lazy wrapper (guarded writes
                    // during its own initializer) keeps declaration order.
                    if let Zval::Object(o) = &target {
                        self.lazy_ordered_insert(o, &key);
                    }
                    // A write through a reference slot honours ALL of the
                    // cell's typed sources (an aliased pair enforces both,
                    // typed_properties_002).
                    if !self.typed_refs.is_empty() {
                        if let Zval::Object(o) = &target {
                            let cell = match o.borrow().props.get(&key) {
                                Some(Zval::Ref(c)) => Some(Rc::clone(c)),
                                _ => None,
                            };
                            if let Some(cell) = cell {
                                let strict = self.frames[top].module.strict;
                                value = self.typed_ref_assign(&cell, value, strict)?;
                            }
                        }
                    }
                    if let Some(old) = write_property(&target, &key, value.clone())? {
                        self.gc_note(&old);
                    }
                    self.frames[top].stack.push(value);
                }
                Op::PropOpSet { name, op } => {
                    let rhs = self.frames[top].stack.pop().expect("PropOpSet rhs");
                    let obj = self.frames[top].stack.pop().expect("PropOpSet object");
                    let cur = self.frames[top].class;
                    // A compound read-modify-write initializes a lazy object (PHP
                    // 8.4, fetch_op_initializes) — unless a hook serves it.
                    let obj = self.lazy_prop_access(obj.deref_clone(), &name, cur, None, (MagicKind::Get, b"__get"))?;
                    let key = match object_class_id(&obj) {
                        Some(ocid) => {
                            check_prop_access(&self.classes, cur, ocid, &name)?;
                            self.prop_storage_key(ocid, &name, cur)
                        }
                        None => name.to_vec(),
                    };
                    if let Some(err) = self.readonly_rmw_error(&obj, &key, &name) {
                        return Err(err);
                    }
                    if let Zval::Object(o) = &obj.deref_clone() {
                        if let Some(err) = self.uninit_typed_read(o, &key, &name) {
                            return Err(err);
                        }
                    }
                    let old = read_property(&obj, &key, &mut self.diags);
                    let mut result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    // A `set` hook with no `get` hook (the backing read above is then
                    // the property's own value) handles the write: dispatch it like
                    // `$o->prop = result`, the compound's value being `result`.
                    if let Zval::Object(o) = &obj.deref_clone() {
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) && self.prop_hook(cid, &name, false).is_none() {
                            if let Some(func) = self.prop_hook(cid, &name, true) {
                                self.frames[top].stack.push(result.clone());
                                self.push_hook(func, obj.deref_clone(), oid, &name, Some(result));
                                continue;
                            }
                        }
                    }
                    if let Some(ocid) = object_class_id(&obj) {
                        result = self.coerce_typed_prop_write(ocid, &name, result)?;
                    }
                    if let Some(dropped) = write_property(&obj, &key, result.clone())? {
                        self.gc_note(&dropped);
                    }
                    self.frames[top].stack.push(result);
                }
                Op::PropIncDec { name, inc, pre } => {
                    let obj = self.frames[top].stack.pop().expect("PropIncDec object");
                    let cur = self.frames[top].class;
                    // `++`/`--` initializes a lazy object like a compound assign.
                    let obj = self.lazy_prop_access(obj.deref_clone(), &name, cur, None, (MagicKind::Get, b"__get"))?;
                    let key = match object_class_id(&obj) {
                        Some(ocid) => {
                            check_prop_access(&self.classes, cur, ocid, &name)?;
                            self.prop_storage_key(ocid, &name, cur)
                        }
                        None => name.to_vec(),
                    };
                    if let Some(err) = self.readonly_rmw_error(&obj, &key, &name) {
                        return Err(err);
                    }
                    if let Zval::Object(o) = &obj.deref_clone() {
                        if let Some(err) = self.uninit_typed_read(o, &key, &name) {
                            return Err(err);
                        }
                    }
                    let old = read_property(&obj, &key, &mut self.diags);
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    // A `set` hook (no `get` hook) handles the write; the inc/dec
                    // expression still yields the pre/post value.
                    if let Zval::Object(o) = &obj.deref_clone() {
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) && self.prop_hook(cid, &name, false).is_none() {
                            if let Some(func) = self.prop_hook(cid, &name, true) {
                                self.frames[top].stack.push(if pre { newv.clone() } else { old });
                                self.push_hook(func, obj.deref_clone(), oid, &name, Some(newv));
                                continue;
                            }
                        }
                    }
                    if let Some(ocid) = object_class_id(&obj) {
                        newv = self.coerce_typed_prop_write(ocid, &name, newv)?;
                    }
                    if let Some(dropped) = write_property(&obj, &key, newv.clone())? {
                        self.gc_note(&dropped);
                    }
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::PropIssetDyn => {
                    // Dynamic-name twin of `PropIsset`: pop the name, then fall
                    // into the same hook/`__isset`/visibility dispatch.
                    let name_v = self.frames[top].stack.pop().expect("PropIssetDyn name");
                    let name: Box<[u8]> =
                        convert::to_zstr_cast(&name_v, &mut self.diags).as_bytes().to_vec().into();
                    let obj = self.frames[top].stack.pop().expect("PropIssetDyn object");
                    let cur = self.frames[top].class;
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Isset, b"__isset"))?;
                    let set = if let Zval::Object(o) = &target {
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                self.frames.last_mut().unwrap().ret_isset = true;
                                continue;
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Isset, b"__isset")
                        {
                            self.push_magic_prop(defc, midx, oid, MagicKind::Isset, target.clone(), &name, None, None, true);
                            continue;
                        }
                        let ocid = o.borrow().class_id as usize;
                        match prop_vis_decl(&self.classes, ocid, &name) {
                            Some((vis, decl)) if !visible_from(&self.classes, cur, vis, decl) => false,
                            _ => {
                                let key = self.prop_storage_key(ocid, &name, cur);
                                prop_isset(&target, &key)
                            }
                        }
                    } else {
                        prop_isset(&target, &name)
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::PropIsset { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropIsset object");
                    let cur = self.frames[top].class;
                    // `isset()` initializes a lazy object (PHP 8.4,
                    // isset_initializes) — unless a get hook/`__isset` serves it.
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Isset, b"__isset"))?;
                    let set = if let Zval::Object(o) = &target {
                        // `isset($o->hooked)` runs the `get` hook and tests its result
                        // for being non-null (step 50). Hooks precede `__isset`.
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                self.frames.last_mut().unwrap().ret_isset = true;
                                continue;
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Isset, b"__isset")
                        {
                            // __isset's return (coerced to bool via ret_bool) is the
                            // result.
                            self.push_magic_prop(defc, midx, oid, MagicKind::Isset, target.clone(), &name, None, None, true);
                            continue;
                        }
                        // No magic: an inaccessible declared property reads as not-set.
                        let ocid = o.borrow().class_id as usize;
                        match prop_vis_decl(&self.classes, ocid, &name) {
                            Some((vis, decl)) if !visible_from(&self.classes, cur, vis, decl) => false,
                            _ => {
                                let key = self.prop_storage_key(ocid, &name, cur);
                                prop_isset(&target, &key)
                            }
                        }
                    } else {
                        prop_isset(&target, &name)
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::PropUnset { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropUnset object");
                    let cur = self.frames[top].class;
                    // `unset()` initializes a lazy object (PHP 8.4,
                    // unset_undefined_initializes) — unless `__unset`/a hook error
                    // serves it (hooked props fatal below without initializing).
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, None, (MagicKind::Unset, b"__unset"))?;
                    let mut key = name.to_vec();
                    if let Zval::Object(o) = &target {
                        // An enum case property is readonly — it cannot be unset.
                        if o.borrow().info.is_enum_case {
                            let ob = o.borrow();
                            let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                            let prop = String::from_utf8_lossy(&name).into_owned();
                            return Err(PhpError::Error(format!(
                                "Cannot unset readonly property {cls}::${prop}"
                            )));
                        }
                        // A hooked property has no plain backing to unset (step 50).
                        if self.prop_hook(o.borrow().class_id as usize, &name, false).is_some()
                            || self.prop_hook(o.borrow().class_id as usize, &name, true).is_some()
                        {
                            let ob = o.borrow();
                            let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                            let prop = String::from_utf8_lossy(&name).into_owned();
                            return Err(PhpError::Error(format!(
                                "Cannot unset hooked property {cls}::${prop}"
                            )));
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Unset, b"__unset")
                        {
                            let discard = Rc::new(RefCell::new(Zval::Null));
                            self.push_magic_prop(defc, midx, oid, MagicKind::Unset, target.clone(), &name, None, Some(discard), false);
                            continue;
                        }
                        check_prop_access(&self.classes, cur, o.borrow().class_id as usize, &name)?;
                        // A readonly property can never be unset (after the
                        // visibility check, so a private/protected one reports the
                        // access error first, matching PHP) — except during `__clone`,
                        // where an `unset` returns it to the re-assignable uninitialised
                        // state (8.3).
                        let ocid = o.borrow().class_id as usize;
                        key = self.prop_storage_key(ocid, &name, cur);
                        if let Some(decl) = prop_readonly_decl(&self.classes, ocid, &name) {
                            if o.borrow().readonly_clone_writable(&key) {
                                o.borrow_mut().clear_readonly_init(&key);
                            } else {
                                let cls = String::from_utf8_lossy(&self.classes[decl].name).into_owned();
                                let prop = String::from_utf8_lossy(&name).into_owned();
                                return Err(PhpError::Error(format!(
                                    "Cannot unset readonly property {cls}::${prop}"
                                )));
                            }
                        }
                        // `unset` deletes the property's typed-reference source:
                        // a still-live alias becomes an ordinary reference
                        // (init_handles_ref_source_types).
                        if !self.typed_refs.is_empty() {
                            let op_ptr = Rc::as_ptr(o);
                            let disp = php_types::prop_display_name(&key).to_vec();
                            self.typed_refs.retain(|t| {
                                !(std::ptr::eq(t.obj.as_ptr(), op_ptr) && t.prop.as_ref() == &disp[..])
                            });
                        }
                        // A declared TYPED property returns to the
                        // *uninitialized* state on unset (isInitialized false,
                        // read = "must not be accessed before initialization")
                        // rather than becoming an undefined dynamic prop —
                        // doctrine/persistence's TypedNoDefaultReflectionProperty
                        // models `setValue(null)` exactly this way.
                        let ob = o.borrow();
                        let typed = ob.info.type_of(&key).is_some()
                            || ob.info.type_of(&name).is_some();
                        drop(ob);
                        if typed {
                            o.borrow_mut().props.set(&key, Zval::Undef);
                            continue;
                        }
                    }
                    prop_unset(&target, &key);
                }
                Op::MethodCall { method, argc } => {
                    let args = self.pop_keys(top, argc); // source order
                    let recv = self.frames[top].stack.pop().expect("MethodCall receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args)?;
                }
                Op::MethodCallArgs { method } => {
                    // Spread `$obj->m(...$a)` (Session A): the arguments are the
                    // values of a runtime array (the receiver sits beneath it);
                    // string keys bind as named arguments (PHP 8.1).
                    let argsval = self.frames[top].stack.pop().expect("MethodCallArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let recv = self.frames[top].stack.pop().expect("MethodCallArgs receiver");
                    let this = recv.deref_clone();
                    if named.is_empty() {
                        self.method_call(top, this, &method, args)?;
                    } else {
                        self.dispatch_instance_call_named(top, this, &method, args, named)?;
                    }
                }
                Op::MethodCallDynamic { argc } => {
                    // `$obj->$m(args)`: the method name sits on top, the positional
                    // args beneath it, the receiver at the bottom (step 51).
                    let nameval = self.frames[top].stack.pop().expect("MethodCallDynamic name");
                    let method = convert::to_zstr(&nameval, &mut self.diags).as_bytes().to_vec();
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("MethodCallDynamic receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args)?;
                }
                Op::MethodCallDynamicArgs => {
                    // Spread `$obj->$m(...$a)`: name on top, args array beneath it;
                    // string keys bind as named arguments (PHP 8.1).
                    let nameval = self.frames[top].stack.pop().expect("MethodCallDynamicArgs name");
                    let method = convert::to_zstr(&nameval, &mut self.diags).as_bytes().to_vec();
                    let argsval = self.frames[top].stack.pop().expect("MethodCallDynamicArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let recv = self.frames[top].stack.pop().expect("MethodCallDynamicArgs receiver");
                    let this = recv.deref_clone();
                    if named.is_empty() {
                        self.method_call(top, this, &method, args)?;
                    } else {
                        self.dispatch_instance_call_named(top, this, &method, args, named)?;
                    }
                }
                Op::MethodCallNamed { method, positional, names } => {
                    // Named `$obj->m(p…, n: v, …)` (Session A): pop the named values
                    // (source order), then the positional values, then the receiver.
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let named: Vec<(Box<[u8]>, Zval)> =
                        names.iter().cloned().zip(named_vals).collect();
                    let pos = self.pop_keys(top, positional);
                    let recv = self.frames[top].stack.pop().expect("MethodCallNamed receiver");
                    let this = recv.deref_clone();
                    self.dispatch_instance_call_named(top, this, &method, pos, named)?;
                }
                Op::InvokeMethod { class, method_idx, argc } => {
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeMethod receiver");
                    let this = recv.deref_clone();
                    let lsb = object_class_id(&this).unwrap_or(class);
                    let callee = &self.classes[class].methods[method_idx as usize].func;
                    let mut frame = Frame::new(callee, self.class_mod(class));
                    bind_params(&mut frame, args);
                    frame.this = Some(this);
                    frame.class = Some(class);
                    frame.static_class = Some(lsb);
                    self.enter_callee(frame)?;
                }
                Op::InstanceOf { class } => {
                    let v = self.frames[top].stack.pop().expect("InstanceOf operand");
                    let result = match v.deref_clone() {
                        Zval::Object(o) => {
                            is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, class)
                        }
                        // A generator has no ClassId but is-a Iterator/Traversable
                        // (now real prelude interfaces); nothing else among the
                        // value types satisfies these.
                        Zval::Generator(_) => {
                            let n = &self.classes[class].name;
                            n.eq_ignore_ascii_case(b"Iterator")
                                || n.eq_ignore_ascii_case(b"Traversable")
                        }
                        _ => false,
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfStatic => {
                    let v = self.frames[top].stack.pop().expect("InstanceOfStatic operand");
                    let target = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                    })?;
                    let result = match v.deref_clone() {
                        Zval::Object(o) => {
                            is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, target)
                        }
                        _ => false,
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfDynamic => {
                    // `$x instanceof $cls` (PAR): an unknown class name (or a
                    // non-object operand) yields false — PHP does not error here.
                    let classval = self.frames[top].stack.pop().expect("InstanceOfDynamic class");
                    let operand = self.frames[top].stack.pop().expect("InstanceOfDynamic operand");
                    let result = match (object_class_id(&operand), self.class_id_from_value(&classval))
                    {
                        (Some(ocid), Some(tcid)) => is_instance_of(&self.classes, self.stringable_id, ocid, tcid),
                        // `Closure`/`Generator` have no `ClassId`; match by the
                        // operand's value type against the (string) class name
                        // (`$c instanceof Closure`, `$g instanceof Iterator`).
                        _ => match classval.deref_clone() {
                            Zval::Str(s) => {
                                let raw = s.as_bytes();
                                let lc = raw.strip_prefix(b"\\").unwrap_or(raw).to_ascii_lowercase();
                                match operand.deref_clone() {
                                    Zval::Closure(_) => lc == b"closure",
                                    Zval::Generator(_) => {
                                        matches!(&lc[..], b"generator" | b"iterator" | b"traversable")
                                    }
                                    _ => false,
                                }
                            }
                            _ => false,
                        },
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfBuiltin(_iface) => {
                    // Generator/Iterator/Traversable have no ClassId; a generator
                    // value satisfies all three, nothing else among the value
                    // types does (objects against these names already test false).
                    let v = self.frames[top].stack.pop().expect("InstanceOfBuiltin operand");
                    let result = matches!(v.deref_clone(), Zval::Generator(_));
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::StaticCall { target, method, forwarding, argc } => {
                    let args = self.pop_keys(top, argc);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                        })?,
                    };
                    // `Fiber::suspend` / `Fiber::getCurrent` are native static
                    // dispatch (GEN-4), handled before normal method resolution.
                    if self.fiber_class_id == Some(start) {
                        if method.eq_ignore_ascii_case(b"suspend") {
                            let (id, baseline) = match self.fiber_stack.last() {
                                Some(c) => (c.id, c.baseline),
                                None => {
                                    return Err(PhpError::Error(
                                        "Cannot suspend outside of a fiber".to_string(),
                                    ))
                                }
                            };
                            // Decay a reference pushed by a dynamic call (SEND_VAR_EX).
                            let value = args.into_iter().next().map(decay_arg).unwrap_or(Zval::Null);
                            // Park the whole fiber segment; it is restored by resume.
                            let parked = self.frames.split_off(baseline);
                            self.fibers.get_mut(&id).expect("running fiber state").parked = parked;
                            return Ok(RunExit::Suspended { value });
                        }
                        if method.eq_ignore_ascii_case(b"getcurrent") {
                            let cur = self
                                .fiber_stack
                                .last()
                                .map(|c| c.obj.clone())
                                .unwrap_or(Zval::Null);
                            self.frames[top].stack.push(cur);
                            continue;
                        }
                    }
                    self.dispatch_static_call(top, start, &method, forwarding, args, Vec::new())?;
                }
                Op::HookCall { target, prop, set, argc } => {
                    // PHP 8.4 `parent::$prop::get()` / `parent::$prop::set($v)`.
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].this.clone().ok_or_else(|| {
                        PhpError::Error(format!(
                            "Cannot call ::${}::{} outside object context",
                            String::from_utf8_lossy(&prop),
                            if set { "set" } else { "get" },
                        ))
                    })?;
                    let oid = object_id(&recv);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                        })?,
                    };
                    // A user `get`/`set` hook on the named class runs as a frame.
                    // Extra arguments are ignored — it is an ordinary user function.
                    if let Some(func) = self.prop_hook(start, &prop, set) {
                        let set_value =
                            if set { Some(args.into_iter().next().unwrap_or(Zval::Null)) } else { None };
                        if set {
                            // A user set hook discards its body return; the call yields NULL.
                            self.frames[top].stack.push(Zval::Null);
                        }
                        self.push_parent_hook(func, recv, oid, &prop, start, set_value);
                        continue;
                    }
                    // No user hook: the *implicit* hook reaches the backing store
                    // directly. The property must be declared on the named class.
                    if prop_info(&self.classes, start, &prop).is_none() {
                        return Err(PhpError::Error(format!(
                            "Undefined property {}::${}",
                            String::from_utf8_lossy(&self.classes[start].name),
                            String::from_utf8_lossy(&prop),
                        )));
                    }
                    // The implicit hook is an internal function with fixed arity.
                    let expected = if set { 1usize } else { 0 };
                    if args.len() != expected {
                        return Err(PhpError::Error(format!(
                            "{}::${}::{}() expects exactly {} argument{}, {} given",
                            String::from_utf8_lossy(&self.classes[start].name),
                            String::from_utf8_lossy(&prop),
                            if set { "set" } else { "get" },
                            expected,
                            if expected == 1 { "" } else { "s" },
                            args.len(),
                        )));
                    }
                    let ocid = object_class_id(&recv).unwrap_or(start);
                    let key = self.prop_storage_key(ocid, &prop, Some(start));
                    if set {
                        let v = args.into_iter().next().unwrap_or(Zval::Null);
                        write_property(&recv, &key, v.clone())?;
                        self.frames[top].stack.push(v);
                    } else {
                        if let Zval::Object(o) = &recv {
                            if let Some(err) = self.uninit_typed_read(o, &key, &prop) {
                                return Err(err);
                            }
                        }
                        let v = read_property(&recv, &key, &mut self.diags);
                        self.frames[top].stack.push(v);
                    }
                }
                Op::ClosureStatic { method, argc } => {
                    // `Closure::bind(...)` / `Closure::fromCallable(...)` (step 19-6).
                    let args = self.pop_keys(top, argc); // source order
                    let result = self.closure_static_method(&method, args)?;
                    self.frames[top].stack.push(result);
                }
                Op::StaticCallArgs { target, method, forwarding } => {
                    // Spread / named `C::m(...$a)` / `C::m(name: …)`: args from a
                    // runtime array — integer keys positional, string keys named.
                    let argsval = self.frames[top].stack.pop().expect("StaticCallArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                        })?,
                    };
                    self.dispatch_static_call(top, start, &method, forwarding, args, named)?;
                }
                Op::StaticCallDynamic { method, argc } => {
                    // `$cls::m()` (PAR): args are on top, the class reference beneath.
                    let args = self.pop_keys(top, argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamic class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    // A dynamic class is non-forwarding, like a named class.
                    self.dispatch_static_call(top, start, &method, false, args, Vec::new())?;
                }
                Op::StaticCallDynamicArgs { method } => {
                    // Spread `$cls::m(...$a)` (Session A): args array on top, the
                    // class reference beneath; string keys bind as named (PHP 8.1).
                    let argsval = self.frames[top].stack.pop().expect("StaticCallDynamicArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicArgs class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    self.dispatch_static_call(top, start, &method, false, args, named)?;
                }
                Op::StaticCallDynamicMethod { argc } => {
                    // `$cls::$m()`: method name on top, then args, then the class ref.
                    let mval = self.frames[top].stack.pop().expect("StaticCallDynamicMethod name");
                    let args = self.pop_keys(top, argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicMethod class");
                    // PHP validates the class before the method name: `$a::$b()` with
                    // both invalid reports the class error first.
                    let start = self.resolve_dynamic_class(&classval)?;
                    let method = dyn_method_name(&mval)?;
                    // A dynamic class is non-forwarding, like a named static call.
                    self.dispatch_static_call(top, start, &method, false, args, Vec::new())?;
                }
                Op::StaticCallDynamicMethodArgs => {
                    // `$cls::$m(...)` with named/spread args: method name on top,
                    // args array beneath (string keys = named), class ref below.
                    let mval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicMethodArgs name");
                    let argsval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicMethodArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicMethodArgs class");
                    // PHP validates the class before the method name.
                    let start = self.resolve_dynamic_class(&classval)?;
                    let method = dyn_method_name(&mval)?;
                    self.dispatch_static_call(top, start, &method, false, args, named)?;
                }
                Op::StaticCallTargetDynamicMethodArgs { target, forwarding } => {
                    // `self::$m(...)` / `Class::$m(...)` with named/spread args.
                    let mval = self.frames[top]
                        .stack
                        .pop()
                        .expect("StaticCallTargetDynamicMethodArgs name");
                    let method = dyn_method_name(&mval)?;
                    let argsval = self.frames[top]
                        .stack
                        .pop()
                        .expect("StaticCallTargetDynamicMethodArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                        })?,
                    };
                    self.dispatch_static_call(top, start, &method, forwarding, args, named)?;
                }
                Op::StaticCallTargetDynamicMethod { target, forwarding, argc } => {
                    // `self::$m()` / `Class::$m()`: method name on top, then args; the
                    // class is a compile-time target, forwarding preserved.
                    let mval = self.frames[top]
                        .stack
                        .pop()
                        .expect("StaticCallTargetDynamicMethod name");
                    let method = dyn_method_name(&mval)?;
                    let args = self.pop_keys(top, argc);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                        })?,
                    };
                    self.dispatch_static_call(top, start, &method, forwarding, args, Vec::new())?;
                }
                Op::ClassConstDynamic => {
                    // `C::{$expr}`: class beneath, runtime constant name on top.
                    let nv = self.frames[top].stack.pop().expect("ClassConstDynamic name");
                    let name = convert::to_zstr_cast(&nv, &mut self.diags).as_bytes().to_vec();
                    let classval =
                        self.frames[top].stack.pop().expect("ClassConstDynamic class");
                    let cid = self.resolve_dynamic_class(&classval)?;
                    // NB: the magic `::class` is compile-time only — a dynamic
                    // `C::{'class'}` is an Undefined constant in PHP.
                    if let Some((decl, idx)) = find_const_runtime(&self.classes, cid, &name) {
                        let thunk: &'m Func = &self.classes[decl].consts[idx].func;
                        let v = self.run_value_thunk(thunk, Some(decl))?;
                        self.frames[top].stack.push(v);
                    } else if let Some(ci) = self.enum_case_idx(cid, &name) {
                        let case = self.enum_case(cid, ci as u32);
                        self.frames[top].stack.push(Zval::Object(case));
                    } else {
                        return Err(PhpError::Error(format!(
                            "Undefined constant {}::{}",
                            String::from_utf8_lossy(&self.classes[cid].name),
                            String::from_utf8_lossy(&name)
                        )));
                    }
                }
                Op::ClassConst { class, idx } => {
                    // Run the constant's value thunk as a frame in its declaring
                    // class's context; its `Ret` leaves the value on the caller's
                    // stack.
                    if self.classes[class].consts.get(idx as usize).is_none() {
                        return Err(PhpError::Error(format!(
                            "VM: ClassConst out of range: {}::consts[{}] (len {}) in {} ({}) line {}",
                            String::from_utf8_lossy(&self.classes[class].name),
                            idx,
                            self.classes[class].consts.len(),
                            String::from_utf8_lossy(&self.frames[top].func.name),
                            String::from_utf8_lossy(&self.frames[top].func.file),
                            self.cur_line(top)
                        )));
                    }
                    let thunk = &self.classes[class].consts[idx as usize].func;
                    let mut frame = Frame::new(thunk, self.class_mod(class));
                    frame.class = Some(class);
                    frame.static_class = Some(class);
                    self.frames.push(frame);
                }
                Op::ClassConstDyn { name } => {
                    let start = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                    })?;
                    let Some((decl, idx)) = find_const_runtime(&self.classes, start, &name) else {
                        // An enum case is not a `consts` thunk: materialize its
                        // interned singleton (the runtime mirror of Op::EnumCase).
                        if let Some(ci) = self.classes[start]
                            .enum_cases
                            .iter()
                            .position(|c| c.name.as_ref() == name.as_ref())
                        {
                            let obj = self.enum_case(start, ci as u32);
                            self.frames[top].stack.push(Zval::Object(obj));
                            continue;
                        }
                        return Err(PhpError::Error(format!(
                            "Undefined constant {}::{}",
                            String::from_utf8_lossy(&self.classes[start].name),
                            String::from_utf8_lossy(&name)
                        )));
                    };
                    let thunk = &self.classes[decl].consts[idx].func;
                    let mut frame = Frame::new(thunk, self.class_mod(decl));
                    frame.class = Some(decl);
                    frame.static_class = Some(decl);
                    self.frames.push(frame);
                }
                Op::ClassConstFromValue { name } => {
                    let classval =
                        self.frames[top].stack.pop().expect("ClassConstFromValue class");
                    if name.eq_ignore_ascii_case(b"class") {
                        // `$x::class`: an object yields its class name; a string (or
                        // any non-object) is a TypeError in PHP 8.
                        match classval.deref_clone() {
                            Zval::Object(o) => {
                                let cls = self.classes[o.borrow().class_id as usize].name.to_vec();
                                self.frames[top].stack.push(Zval::Str(PhpStr::new(cls)));
                            }
                            // Engine objects held as dedicated variants still
                            // answer `$v::class` with their class name.
                            Zval::Closure(_) => {
                                self.frames[top].stack.push(Zval::Str(PhpStr::new(b"Closure".to_vec())));
                            }
                            Zval::Generator(_) => {
                                self.frames[top]
                                    .stack
                                    .push(Zval::Str(PhpStr::new(b"Generator".to_vec())));
                            }
                            other => {
                                return Err(PhpError::TypeError(format!(
                                    "Cannot use \"::class\" on {}",
                                    other.type_name_for_error()
                                )))
                            }
                        }
                    } else {
                        let cid = self.resolve_dynamic_class(&classval)?;
                        let Some((decl, idx)) = find_const_runtime(&self.classes, cid, &name) else {
                            // An enum case is not a `consts` thunk: materialize its
                            // interned singleton (the runtime mirror of Op::EnumCase).
                            if let Some(ci) = self.classes[cid]
                                .enum_cases
                                .iter()
                                .position(|c| c.name.as_ref() == name.as_ref())
                            {
                                let obj = self.enum_case(cid, ci as u32);
                                self.frames[top].stack.push(Zval::Object(obj));
                                continue;
                            }
                            return Err(PhpError::Error(format!(
                                "Undefined constant {}::{}",
                                String::from_utf8_lossy(&self.classes[cid].name),
                                String::from_utf8_lossy(&name)
                            )));
                        };
                        let thunk = &self.classes[decl].consts[idx].func;
                        let mut frame = Frame::new(thunk, self.class_mod(decl));
                        frame.class = Some(decl);
                        frame.static_class = Some(decl);
                        self.frames.push(frame);
                    }
                }
                Op::ClassNameStatic => {
                    let start = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                    })?;
                    let name = self.classes[start].name.to_vec();
                    self.frames[top].stack.push(Zval::Str(PhpStr::new(name)));
                }
                Op::EnumCase { class, case } => {
                    let obj = self.enum_case(class, case);
                    self.frames[top].stack.push(Zval::Object(obj));
                }
                Op::InvokeCtor { argc } => {
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtor receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtor on a non-object");
                    match resolve_method_runtime(&self.classes, cid, b"__construct") {
                        Some((defc, midx)) => {
                            let callee = &self.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee, self.class_mod(defc));
                            bind_params(&mut frame, args);
                            frame.this = Some(this);
                            frame.class = Some(defc);
                            frame.static_class = Some(cid);
                            // Coerce the arguments against the constructor's declared
                            // parameter types using the CALL SITE's strict mode, like
                            // `InvokeMethod` — a `new $cls(...)` (dynamic class) must
                            // weak-coerce just as `new C(...)` does (ORM hydrators
                            // instantiate DTOs dynamically: NewOperatorTest).
                            self.enter_callee(frame)?;
                        }
                        // No constructor: leave NULL so the surrounding `Pop` keeps
                        // the operand stack balanced (the instance is kept by `Dup`).
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::InvokeCtorArgs => {
                    // Spread / named `new C(...$a)` / `new C(name: …)` of a class (or
                    // constructor) resolved only at run time: arguments come from a
                    // runtime array — integer keys positional, string keys **named**
                    // (a spread of a string-keyed array binds by name too, PHP 8.1).
                    let argsval = self.frames[top].stack.pop().expect("InvokeCtorArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtorArgs receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtorArgs on a non-object");
                    match resolve_method_runtime(&self.classes, cid, b"__construct") {
                        Some((defc, midx)) => {
                            let callee = &self.classes[defc].methods[midx].func;
                            let mut frame = if named.is_empty() {
                                let mut frame = Frame::new(callee, self.class_mod(defc));
                                bind_params(&mut frame, args);
                                frame
                            } else {
                                let qn = format!(
                                    "{}::__construct",
                                    String::from_utf8_lossy(&self.classes[defc].name)
                                );
                                build_named_frame(callee, self.class_mod(defc), &qn, args, named)?
                            };
                            frame.this = Some(this);
                            frame.class = Some(defc);
                            frame.static_class = Some(cid);
                            // Coerce arguments with the call site's strict mode (see
                            // Op::InvokeCtor); the dynamic ctor path bypassed this.
                            self.enter_callee(frame)?;
                        }
                        // A named argument has no constructor to bind against: the
                        // catchable "Unknown named parameter" (positional arguments
                        // to a ctor-less class are silently discarded, as in PHP).
                        None if !named.is_empty() => return Err(unknown_named_param(&named)),
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::InitProps => {
                    let recv = self.frames[top].stack.pop().expect("InitProps receiver");
                    let cid = object_class_id(&recv).expect("InitProps on a non-object");
                    match &self.classes[cid].prop_init {
                        Some(func) => {
                            let mut frame = Frame::new(func, self.class_mod(cid));
                            frame.this = Some(recv.deref_clone());
                            frame.class = Some(cid);
                            frame.static_class = Some(cid);
                            frame.init_props = true; // privileged default writes
                            self.frames.push(frame);
                        }
                        // No non-constant defaults: nothing to do, balance the stack.
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::StaticPropGet { target, name } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue, // init thunk scheduled; re-run after it
                    };
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSet { target, name } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let value = self.frames[top].stack.pop().expect("StaticPropSet value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropOpSet { target, name, op } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let rhs = self.frames[top].stack.pop().expect("StaticPropOpSet rhs");
                    let old = cell.borrow().deref_clone();
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    *cell.borrow_mut() = result.clone();
                    self.frames[top].stack.push(result);
                }
                Op::StaticPropIncDec { target, name, inc, pre } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let old = cell.borrow().deref_clone();
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::StaticPropGetDynName => {
                    // [classRef, name]: peek both so a scheduled init thunk can
                    // re-run this op with its operands intact (PAR).
                    let n = self.frames[top].stack.len();
                    let nameval = self.frames[top].stack[n - 1].clone();
                    let name: Box<[u8]> =
                        convert::to_zstr_cast(&nameval, &mut self.diags).as_bytes().to_vec().into();
                    let classval = self.frames[top].stack[n - 2].clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // name
                    self.frames[top].stack.pop(); // class
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSetDynName => {
                    let n = self.frames[top].stack.len();
                    let nameval = self.frames[top].stack[n - 1].clone();
                    let name: Box<[u8]> =
                        convert::to_zstr_cast(&nameval, &mut self.diags).as_bytes().to_vec().into();
                    let classval = self.frames[top].stack[n - 2].clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // name
                    self.frames[top].stack.pop(); // class
                    let value = self.frames[top].stack.pop().expect("StaticPropSetDynName value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropGetDynamic { name } => {
                    // The class reference is on top; peek it so a scheduled init
                    // thunk can re-run this op without losing it (PAR).
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // remove the class reference
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSetDynamic { name } => {
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let value = self.frames[top].stack.pop().expect("StaticPropSetDynamic value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropOpSetDynamic { name, op } => {
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let rhs = self.frames[top].stack.pop().expect("StaticPropOpSetDynamic rhs");
                    let old = cell.borrow().deref_clone();
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    *cell.borrow_mut() = result.clone();
                    self.frames[top].stack.push(result);
                }
                Op::StaticPropIncDecDynamic { name, inc, pre } => {
                    // `$cls::$p++` (PAR): peek the class ref so a scheduled init
                    // thunk can re-run this op; pop it once the cell is ready.
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let old = cell.borrow().deref_clone();
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::FieldAssign { base, steps } => {
                    let value = self.frames[top].stack.pop().expect("FieldAssign value");
                    let keys = self.pop_field_keys(top, &steps);
                    // A path starting at a `&get` hooked property writes through
                    // the reference the hook returns (one hook run, no set hook).
                    if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                        self.field_set_in_root(root, top, &steps[1..], keys, value.clone(), false)?;
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    self.reject_indirect_hook(base, top, &steps)?;
                    // A lazy base initializes/forwards first; the walk then roots
                    // at the realized object (PHP 8.4).
                    if let Some(root) = self.field_lazy_root(base, top, &steps, &keys, true)? {
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, value.clone(), false)?;
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    self.field_set(base, top, &steps, keys, value.clone())?;
                    self.frames[top].stack.push(value);
                }
                Op::FieldAssignOp { base, steps, op } => {
                    let rhs = self.frames[top].stack.pop().expect("FieldAssignOp rhs");
                    let keys = self.pop_field_keys(top, &steps);
                    if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&Zval::Ref(Rc::clone(&root)), &steps[1..], &mut keys.clone().into_iter(), fs)
                                .unwrap_or(Zval::Null)
                        };
                        let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                        self.field_set_in_root(root, top, &steps[1..], keys, result.clone(), false)?;
                        self.frames[top].stack.push(result);
                        continue;
                    }
                    self.reject_indirect_hook(base, top, &steps)?;
                    if let Some(root) = self.field_lazy_root(base, top, &steps, &keys, true)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&root, &steps, &mut keys.clone().into_iter(), fs).unwrap_or(Zval::Null)
                        };
                        let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, result.clone(), false)?;
                        self.frames[top].stack.push(result);
                        continue;
                    }
                    let old = self.field_value(base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    self.field_set(base, top, &steps, keys, result.clone())?;
                    self.frames[top].stack.push(result);
                }
                Op::FieldIncDec { base, steps, inc, pre } => {
                    let keys = self.pop_field_keys(top, &steps);
                    if let Some(root) = self.byref_hook_root(base, top, &steps)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&Zval::Ref(Rc::clone(&root)), &steps[1..], &mut keys.clone().into_iter(), fs)
                                .unwrap_or(Zval::Null)
                        };
                        let mut newv = old.clone();
                        if inc {
                            ops::increment(&mut newv, &mut self.diags)?;
                        } else {
                            ops::decrement(&mut newv, &mut self.diags)?;
                        }
                        self.field_set_in_root(root, top, &steps[1..], keys, newv.clone(), false)?;
                        self.frames[top].stack.push(if pre { newv } else { old });
                        continue;
                    }
                    self.reject_indirect_hook(base, top, &steps)?;
                    if let Some(root) = self.field_lazy_root(base, top, &steps, &keys, true)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&root, &steps, &mut keys.clone().into_iter(), fs).unwrap_or(Zval::Null)
                        };
                        let mut newv = old.clone();
                        if inc {
                            ops::increment(&mut newv, &mut self.diags)?;
                        } else {
                            ops::decrement(&mut newv, &mut self.diags)?;
                        }
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, newv.clone(), false)?;
                        self.frames[top].stack.push(if pre { newv } else { old });
                        continue;
                    }
                    let old = self.field_value(base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    self.field_set(base, top, &steps, keys, newv.clone())?;
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::FieldIsset { base, steps } => {
                    let mut keys = self.pop_field_keys(top, &steps);
                    // Magic protocol at the path start (`isset($o->magic['k'])`,
                    // gh18038 / the bug40833 family): `__isset` decides, then
                    // `__get` supplies the value the rest of the path tests. An
                    // initialized proxy dispatches on its instance; an
                    // uninitialized wrapper dispatches on itself (no init).
                    let magic_set: Option<bool> = 'magic_isset: {
                        let (name, rest, key_used): (Vec<u8>, &[FieldStep], usize) = match steps.split_first() {
                            Some((FieldStep::Prop(n), rest)) => (n.to_vec(), rest, 0),
                            Some((FieldStep::PropDyn, rest)) => {
                                let Some(k) = keys.first() else { break 'magic_isset None };
                                (convert::to_zstr_cast(k, &mut self.diags).as_bytes().to_vec(), rest, 1)
                            }
                            _ => break 'magic_isset None,
                        };
                        let base_val = match base {
                            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
                            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
                            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
                            FieldBase::This => self.frames[top].this.as_ref(),
                        };
                        let Some(mut v) = base_val.map(|v| v.deref_clone()) else { break 'magic_isset None };
                        // Follow *initialized* proxy forwarding (no initialization).
                        for _ in 0..64 {
                            let next = self.proxy_redirect(v.clone());
                            let same = match (deref_object(&v), deref_object(&next)) {
                                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                                _ => true,
                            };
                            v = next;
                            if same {
                                break;
                            }
                        }
                        let Some(o) = deref_object(&v) else { break 'magic_isset None };
                        let cur = self.frames[top].class;
                        let has_isset =
                            self.magic_applies(&o, &name, cur, MagicKind::Isset, b"__isset").is_some();
                        let has_get =
                            self.magic_applies(&o, &name, cur, MagicKind::Get, b"__get").is_some();
                        if !has_isset && !has_get {
                            break 'magic_isset None;
                        }
                        let oid = o.borrow().id;
                        let name_z = Zval::Str(PhpStr::new(name.clone()));
                        let read_through_get = |vm: &mut Self, keys: Vec<Zval>| -> Result<bool, PhpError> {
                            let gkey = (oid, MagicKind::Get, name.clone());
                            let ins = vm.magic_guard.insert(gkey.clone());
                            let r = vm.call_method_sync(v.clone(), b"__get", vec![name_z.clone()]);
                            if ins {
                                vm.magic_guard.remove(&gkey);
                            }
                            let gv = r?.deref_clone();
                            let fs = FieldScope { classes: &vm.classes, scope: cur };
                            let mut it = keys.into_iter();
                            Ok(matches!(
                                field_get(&gv, rest, &mut it, fs),
                                Some(x) if !matches!(x, Zval::Null | Zval::Undef)
                            ))
                        };
                        let set = if has_isset {
                            let gkey = (oid, MagicKind::Isset, name.clone());
                            let ins = self.magic_guard.insert(gkey.clone());
                            let r = self.call_method_sync(v.clone(), b"__isset", vec![name_z.clone()]);
                            if ins {
                                self.magic_guard.remove(&gkey);
                            }
                            let mut set = convert::to_bool(&r?.deref_clone(), &mut self.diags);
                            if set && !rest.is_empty() {
                                set = if has_get {
                                    read_through_get(self, keys.split_off(key_used))?
                                } else {
                                    false
                                };
                            }
                            set
                        } else {
                            // `__isset` guarded (a re-entrant check inside it) or
                            // absent: read through `__get` for the offset test.
                            read_through_get(self, keys.split_off(key_used))?
                        };
                        Some(set)
                    };
                    if let Some(set) = magic_set {
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    // A final Index on an ArrayAccess object is the protocol:
                    // `isset($this->coll[0])` = offsetExists (no offsetGet),
                    // mirroring Op::IssetPath's single-step arm.
                    if let Some((recv, key)) = self.field_aa_leaf(base, top, &steps, &keys) {
                        let r = self.call_method_sync(recv, b"offsetExists", vec![key])?;
                        let set = convert::is_true_silent(&r.deref_clone());
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    // A lazy base initializes and the walk roots at the realized
                    // object (isset through a wrapper reads the instance).
                    if let Some(root) = self.field_lazy_root(base, top, &steps, &keys, false)? {
                        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                        let set = matches!(
                            field_get(&root, &steps, &mut keys.into_iter(), fs),
                            Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                        );
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    let set = matches!(
                        self.field_value(base, top, &steps, keys),
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                    );
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::FieldEmpty { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    // ArrayAccess leaf: !offsetExists short-circuits to empty,
                    // else !truthy(offsetGet) — mirrors Op::EmptyPath.
                    if let Some((recv, key)) = self.field_aa_leaf(base, top, &steps, &keys) {
                        let exists =
                            self.call_method_sync(recv.clone(), b"offsetExists", vec![key.clone()])?;
                        let empty = if convert::is_true_silent(&exists.deref_clone()) {
                            let v = self.call_method_sync(recv, b"offsetGet", vec![key])?;
                            !convert::is_true_silent(&v.deref_clone())
                        } else {
                            true
                        };
                        self.frames[top].stack.push(Zval::Bool(empty));
                        continue;
                    }
                    // empty == !isset || !truthy(value): an unreachable/null leaf is
                    // empty; otherwise test the value's boolean.
                    let empty = match self.field_value(base, top, &steps, keys) {
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef) => {
                            !convert::to_bool(&v, &mut self.diags)
                        }
                        _ => true,
                    };
                    self.frames[top].stack.push(Zval::Bool(empty));
                }
                Op::FieldUnset { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    // `unset($o->magic)` reached through the path walker (a
                    // dynamic name): `__unset` dispatches like Op::PropUnset —
                    // on the initialized-proxy instance, or the uninitialized
                    // wrapper without initializing (gh18038-010).
                    let magic_done: bool = 'magic_unset: {
                        let name: Vec<u8> = match steps.split_first() {
                            Some((FieldStep::Prop(n), [])) => n.to_vec(),
                            Some((FieldStep::PropDyn, [])) => {
                                let Some(k) = keys.first() else { break 'magic_unset false };
                                convert::to_zstr_cast(k, &mut self.diags).as_bytes().to_vec()
                            }
                            _ => break 'magic_unset false,
                        };
                        let base_val = match base {
                            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
                            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
                            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
                            FieldBase::This => self.frames[top].this.as_ref(),
                        };
                        let Some(mut v) = base_val.map(|v| v.deref_clone()) else {
                            break 'magic_unset false;
                        };
                        for _ in 0..64 {
                            let next = self.proxy_redirect(v.clone());
                            let same = match (deref_object(&v), deref_object(&next)) {
                                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                                _ => true,
                            };
                            v = next;
                            if same {
                                break;
                            }
                        }
                        let Some(o) = deref_object(&v) else { break 'magic_unset false };
                        let cur = self.frames[top].class;
                        if self.magic_applies(&o, &name, cur, MagicKind::Unset, b"__unset").is_none() {
                            break 'magic_unset false;
                        }
                        let oid = o.borrow().id;
                        let gkey = (oid, MagicKind::Unset, name.clone());
                        let ins = self.magic_guard.insert(gkey.clone());
                        let r = self.call_method_sync(v.clone(), b"__unset", vec![Zval::Str(PhpStr::new(name))]);
                        if ins {
                            self.magic_guard.remove(&gkey);
                        }
                        r?;
                        true
                    };
                    if magic_done {
                        continue;
                    }
                    // ArrayAccess leaf: `unset($this->coll[0])` = offsetUnset.
                    if let Some((recv, key)) = self.field_aa_leaf(base, top, &steps, &keys) {
                        self.call_method_sync(recv, b"offsetUnset", vec![key])?;
                        continue;
                    }
                    // Single-property paths mirror Op::PropUnset's declared-slot
                    // rules: the typed-reference source dies, a TYPED slot
                    // returns to uninitialized (typed_properties_002's
                    // `unset($obj->$a)`), a plain one is removed.
                    let single_name: Option<Box<[u8]>> = match &steps[..] {
                        [FieldStep::Prop(n)] => Some(n.clone()),
                        [FieldStep::PropDyn] => match keys.first().cloned() {
                            Some(k) => Some(self.dyn_prop_name_value(&k)?),
                            None => None,
                        },
                        _ => None,
                    };
                    if let Some(n) = single_name {
                        let target = match base {
                            FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
                            FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
                            FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                        };
                        let target = target.map(|v| self.proxy_view(v));
                        if let Some(o) = target.as_ref().and_then(deref_object) {
                            let cur = self.frames[top].class;
                            let ocid = o.borrow().class_id as usize;
                            let key = self.prop_storage_key(ocid, &n, cur);
                            if !self.typed_refs.is_empty() {
                                let op_ptr = Rc::as_ptr(&o);
                                let disp = php_types::prop_display_name(&key).to_vec();
                                self.typed_refs.retain(|t| {
                                    !(std::ptr::eq(t.obj.as_ptr(), op_ptr) && t.prop.as_ref() == &disp[..])
                                });
                            }
                            if prop_type_decl(&self.classes, ocid, &n).is_some() {
                                o.borrow_mut().props.set(&key, Zval::Undef);
                                continue;
                            }
                        }
                    }
                    // A lazy base initializes; the removal runs on the realized
                    // object (a guarded `unset($this->$name)` inside `__unset`).
                    if let Some(mut root) = self.field_lazy_root(base, top, &steps, &keys, false)? {
                        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                        field_unset(&mut root, &steps, &mut keys.into_iter(), fs);
                        continue;
                    }
                    self.field_remove(base, top, &steps, keys);
                }
                Op::Fatal(i) => {
                    let msg = match &self.frames[top].func.consts[i as usize] {
                        crate::bytecode::Const::Str(b) => String::from_utf8_lossy(b).into_owned(),
                        _ => "VM: unsupported construct".to_string(),
                    };
                    return Err(PhpError::Error(msg));
                }
                Op::EmitNotice(i) => {
                    if let crate::bytecode::Const::Str(b) = &self.frames[top].func.consts[i as usize] {
                        let msg = String::from_utf8_lossy(b).into_owned();
                        self.diags.push(Diag::Notice(msg));
                    }
                }
                Op::Exit { has_arg } => {
                    let code = if has_arg {
                        let v = self.frames[top].stack.pop().expect("Exit status");
                        self.exit_status(v, top)?
                    } else {
                        0
                    };
                    return Err(PhpError::Exit(code));
                }
                Op::SuppressBegin => {
                    self.suppress_marks.push(self.diags.len());
                    self.suppress_depth += 1;
                }
                Op::SuppressEnd => {
                    self.suppress_depth = self.suppress_depth.saturating_sub(1);
                    if let Some(saved) = self.suppress_marks.pop() {
                        // Drop the diagnostics raised under `@` (never rendered, as
                        // `flush_diags` was a no-op while suppressed) — but `@` does
                        // NOT hide them from error_get_last (monolog's StreamHandler
                        // appends error_get_last()['message'] after an @fopen): the
                        // last suppressed one is recorded like the default handler
                        // would have.
                        if let Some(d) = self.diags.get(saved..).and_then(|s| s.last()) {
                            let (errno, message) = match d {
                                Diag::Warning(m) => (2, m),
                                Diag::Notice(m) => (8, m),
                                Diag::Deprecated(m) => (8192, m),
                            };
                            let line = self.cur_line(top);
                            self.last_error = Some((errno, message.as_bytes().to_vec(), line));
                        }
                        self.diags.truncate(saved);
                    }
                }
                Op::MatchError(slot) => {
                    let subj = read_slot(&self.frames[top].slots[slot as usize]);
                    return Err(PhpError::Error(format!(
                        "Unhandled match case {}",
                        match_case_repr(&subj)
                    )));
                }
                Op::Sweep => {
                    self.gc_sweep(top, ip)?;
                }
                Op::Nop => {}
            }
        }
    }
}
