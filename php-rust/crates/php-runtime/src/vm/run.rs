//! The bounded VM dispatch loop (run_loop) — the hot Op match. Split from
//! vm/mod.rs verbatim (structural move only; hot-path unchanged).

use std::borrow::Cow;

use super::*;

/// A file op that a userland stream wrapper (`stream_wrapper_register`) can
/// service via its `stream_*` methods, if its first argument is a `UserStream`.
fn is_user_stream_op(name: &[u8]) -> bool {
    matches!(
        name,
        b"fread"
            | b"fwrite"
            | b"fputs"
            | b"feof"
            | b"fclose"
            | b"fgets"
            | b"stream_get_contents"
            | b"rewind"
            | b"fseek"
            | b"ftell"
    )
}

/// The resource handle if `v` (following a ref) is a userland-wrapper stream.
fn user_stream_rc(v: &Zval) -> Option<Rc<RefCell<php_types::Resource>>> {
    match v.deref_clone() {
        Zval::Resource(rc) if rc.borrow().as_user_stream().is_some() => Some(rc),
        _ => None,
    }
}

/// A path-taking builtin serviceable on a registered userland wrapper URL:
/// stat-family via the wrapper's `url_stat`, image/exif introspection via an
/// open/read-to-EOF/close of the wrapper stream (see `user_wrapper_path_op`).
pub(super) fn is_user_wrapper_path_op(name: &[u8]) -> bool {
    matches!(
        name,
        b"file_exists"
            | b"is_file"
            | b"is_dir"
            | b"is_readable"
            | b"is_writable"
            | b"is_writeable"
            | b"filesize"
            | b"filemtime"
            | b"stat"
            | b"getimagesize"
            | b"exif_read_data"
            | b"exif_imagetype"
    )
}

/// The URL bytes if `v` is a string whose scheme is a registered userland wrapper
/// (so `file_get_contents` routes through the wrapper instead of the filesystem).
pub(super) fn user_wrapper_url(
    v: &Zval,
    wrappers: &std::collections::HashMap<Vec<u8>, Vec<u8>>,
) -> Option<Vec<u8>> {
    let s = match v.deref_clone() {
        Zval::Str(s) => s.as_bytes().to_vec(),
        _ => return None,
    };
    let pos = s.windows(3).position(|w| w == b"://")?;
    if wrappers.contains_key(&s[..pos].to_ascii_lowercase()) {
        Some(s)
    } else {
        None
    }
}

/// WP-33 T1a: the identity class of an operand for the `===`/`!==` fast
/// path. Mirrors the arm structure of [`ops::identical`] exactly: two
/// operands of DIFFERENT classes are never identical (no coercion), same
/// class falls back to the generic path unless handled inline. `Ref`,
/// `ArgPlace` and `WeakHandle` return `None` (deref / special-cased there).
#[inline(always)]
fn ident_class(v: &Zval) -> Option<u8> {
    Some(match v {
        // `identical` treats Undef and Null as one identity class.
        Zval::Undef | Zval::Null => 0,
        Zval::Bool(_) => 1,
        Zval::Long(_) => 2,
        Zval::Double(_) => 3,
        Zval::Str(_) => 4,
        Zval::Array(_) => 5,
        Zval::Object(_) => 6,
        Zval::Resource(_) => 7,
        Zval::Closure(_) => 8,
        Zval::Generator(_) => 9,
        _ => return None,
    })
}

/// WP-33 T1a: monomorphic fast paths for the hottest operand tag pairs
/// (census-driven: Concat/NotIdentical (Str,Str) ~30M each per WP suite
/// run, Long arithmetic/compares, cross-class `===` constants). Tried
/// straight after the two pops in [`Vm::binary_value`]; `None` falls
/// through to the generic path (lazy realization, `__toString` rule,
/// GMP/BcMath overloads, division/shift error paths, numeric-string
/// loose equality) unchanged.
///
/// Semantics are inlined VERBATIM from ops.rs: Long overflow promotes to
/// Double recomputing on the ORIGINAL operands (never the wrapped value);
/// Double comparisons keep IEEE semantics (`Gt`/`Ge` written as the
/// swapped `smaller` forms, NaN-exact; `===` is IEEE `==`: -0.0===0.0,
/// NaN!==NaN). String LOOSE equality never fast-paths ("10"=="1e1" lives
/// in smart_streq only). None of the handled pairs can involve a lazy
/// object, an overload receiver, or a diagnostic.
#[inline(always)]
fn binary_fast(b: BinOp, lhs: &Zval, rhs: &Zval) -> Option<Zval> {
    use BinOp::*;
    Some(match (lhs, rhs) {
        (Zval::Long(l), Zval::Long(r)) => {
            let (l, r) = (*l, *r);
            match b {
                Add => match l.checked_add(r) {
                    Some(s) => Zval::Long(s),
                    None => Zval::Double(l as f64 + r as f64),
                },
                Sub => match l.checked_sub(r) {
                    Some(s) => Zval::Long(s),
                    None => Zval::Double(l as f64 - r as f64),
                },
                Mul => match l.checked_mul(r) {
                    Some(s) => Zval::Long(s),
                    None => Zval::Double(l as f64 * r as f64),
                },
                Lt => Zval::Bool(l < r),
                Le => Zval::Bool(l <= r),
                Gt => Zval::Bool(l > r),
                Ge => Zval::Bool(l >= r),
                Eq | Identical => Zval::Bool(l == r),
                NotEq | NotIdentical => Zval::Bool(l != r),
                Spaceship => Zval::Long(match l.cmp(&r) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                }),
                BitAnd => Zval::Long(l & r),
                BitOr => Zval::Long(l | r),
                BitXor => Zval::Long(l ^ r),
                // Div (zero / exact-division / MIN÷-1), Mod, Pow, Shl/Shr
                // (negative-shift errors), Concat → generic.
                _ => return None,
            }
        }
        (Zval::Double(l), Zval::Double(r)) => {
            let (l, r) = (*l, *r);
            match b {
                Add => Zval::Double(l + r),
                Sub => Zval::Double(l - r),
                Mul => Zval::Double(l * r),
                Lt => Zval::Bool(l < r),
                Le => Zval::Bool(l <= r),
                // `a > b` is `smaller(b, a)`: the swapped form is NaN-exact.
                Gt => Zval::Bool(r < l),
                Ge => Zval::Bool(r <= l),
                Identical => Zval::Bool(l == r),
                NotIdentical => Zval::Bool(l != r),
                // Eq/NotEq/Spaceship keep the generic compare() path.
                _ => return None,
            }
        }
        (Zval::Str(l), Zval::Str(r)) => match b {
            // Operands are already strings — `ops::concat`'s to_zstr is an
            // identity with no diagnostics; byte concat verbatim.
            Concat => {
                let (la, ra) = (l.as_bytes(), r.as_bytes());
                let mut out = Vec::with_capacity(la.len() + ra.len());
                out.extend_from_slice(la);
                out.extend_from_slice(ra);
                Zval::Str(PhpStr::new(out))
            }
            Identical => Zval::Bool(l.as_bytes() == r.as_bytes()),
            NotIdentical => Zval::Bool(l.as_bytes() != r.as_bytes()),
            // Loose ==/comparisons: numeric-string semantics stay in ONE
            // place (smart_streq / compare) — never fast-pathed.
            _ => return None,
        },
        (lhs, rhs) if matches!(b, Identical | NotIdentical) => {
            // Cross-class `===` is a constant: no coercion, no lazy init,
            // no overload (try_number_binop never handles ===/!==), and
            // ops::identical's cross-class arms are all `_ => false`.
            match (ident_class(lhs), ident_class(rhs)) {
                (Some(ca), Some(cb)) if ca != cb => Zval::Bool(b == NotIdentical),
                _ => return None,
            }
        }
        _ => return None,
    })
}

impl<'m> super::Vm<'m> {
    /// The bounded dispatch loop: runs until the frame at `baseline` returns
    /// ([`RunExit::Returned`]) or a generator at `baseline` suspends at a `yield`
    /// ([`RunExit::Yielded`]), or an op raises a `PhpError` (which the caller
    /// routes through [`Self::unwind`]). Frames above `baseline` (ordinary
    /// callees) return normally to their callers within this same loop.
    /// Pop rhs/lhs and evaluate a binary operator exactly as [`Op::Binary`]
    /// does — lazy-operand realization (init_trigger_compare), the
    /// string-vs-object `__toString` comparison rule, and the GMP/BcMath
    /// overloads of `apply_binop_ovl` — returning the result value. Extracted
    /// (WP-32) so [`Op::CmpJmp`] shares the identical semantics (diag order,
    /// TypeError paths) by construction instead of by duplication.
    fn binary_value(&mut self, top: usize, b: BinOp) -> Result<Zval, PhpError> {
        let rhs = self.frames[top].stack.pop().expect("Binary rhs");
        let lhs = self.frames[top].stack.pop().expect("Binary lhs");
        // WP-33 T1a: monomorphic tag-pair fast paths (census-driven); a
        // miss falls through to the generic path below unchanged.
        if let Some(r) = binary_fast(b, &lhs, &rhs) {
            return Ok(r);
        }
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
            // Tag peek only — no value clone (WP-32): the old
            // deref_clone here cloned (and dropped) both operands
            // on EVERY executed comparison just to read the tag.
            fn is_str_operand(v: &Zval) -> bool {
                match v {
                    Zval::Str(_) => true,
                    Zval::Ref(c) => matches!(&*c.borrow(), Zval::Str(_)),
                    _ => false,
                }
            }
            let l_str = is_str_operand(&lhs);
            let r_str = is_str_operand(&rhs);
            let to_str = |vm: &mut Self, v: Zval, other_is_str: bool| -> Result<Zval, PhpError> {
                if !other_is_str {
                    return Ok(v);
                }
                if let Some(o) = deref_object(&v) {
                    // `BcMath\Number` / `GMP` overload comparison
                    // themselves (their compare handler runs in
                    // `apply_binop_ovl`); they must NOT be pre-coerced
                    // to their `__toString` for a string comparison.
                    let cn = o.borrow().class_name.as_bytes().to_vec();
                    if cn == b"BcMath\\Number" || cn == b"GMP" {
                        return Ok(v);
                    }
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
        self.apply_binop_ovl(b, &lhs, &rhs)
    }

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
            // WP-31: `func` is `&'m Func` (Copy) — copying the reference out
            // unties the op from `self`, so the match runs on `&'m Op` with
            // ZERO per-instruction clone (the old `.clone()` copied the Op
            // struct plus an Rc bump per payload on every VM tick). Handlers
            // clone payloads only where they genuinely take ownership.
            let func = self.frames[top].func;
            let op = &func.ops[ip];
            // WP-33 T0: op census — compiled out unless the `op-census`
            // feature is on (even a never-taken branch here costs ~3%).
            #[cfg(feature = "op-census")]
            if self.census_on {
                super::census::census_record(
                    op,
                    &self.frames[top].stack,
                    &self.frames[top].slots,
                );
            }
            // Default fall-through advance. Jumps overwrite `ip`; `Call` advances
            // the *caller* before pushing the callee; `Ret` discards this frame.
            self.frames[top].ip = ip + 1;

            match op {
                Op::PushConst(i) => {
                    let v = self.frames[top].func.consts[*i as usize].to_zval();
                    self.frames[top].stack.push(v);
                }
                Op::ConstFetch { name, fallback } => {
                    // A user constant (B3). An unqualified constant inside a
                    // namespace is looked up as `CURNS\NAME` first, then the
                    // global `NAME` (step 50) — where the global step ALSO
                    // consults the engine-constant table: inside a namespace
                    // the lowering cannot fold `INI_ALL` eagerly (a namespaced
                    // `const INI_ALL` shadows it — ns_043), so the fold happens
                    // here on the fallback. An "Undefined constant" error
                    // reports the namespaced name.
                    let v = self
                        .constants
                        .get(&name[..])
                        .or_else(|| fallback.as_ref().and_then(|g| self.constants.get(&g[..])))
                        .cloned()
                        .or_else(|| {
                            fallback
                                .as_ref()
                                .and_then(|g| crate::lower::resolve_constant(g))
                                .and_then(super::calls::const_literal_to_zval)
                        })
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
                    let v = read_slot(&self.frames[top].slots[*s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::LoadVar { slot, name } => {
                    // A source-level `$x` read: an `Undef` slot raises the PHP 8
                    // "Undefined variable" warning (queued; flushed at the next
                    // emit point with the reading op's line) and yields NULL.
                    if matches!(self.frames[top].slots[*slot as usize], Zval::Undef) {
                        if let crate::bytecode::Const::Str(b) =
                            &self.frames[top].func.consts[*name as usize]
                        {
                            let msg = format!("Undefined variable ${}", String::from_utf8_lossy(b.as_bytes()));
                            self.diags.push(Diag::Warning(msg));
                        }
                    }
                    let v = read_slot(&self.frames[top].slots[*slot as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSlot(s) => {
                    // A slot aliasing a *typed property* (a registered typed
                    // reference) coerces/checks the value first (PHP's typed-ref
                    // assignment: "Cannot assign string to reference held by
                    // property C::$p of type int").
                    if !self.typed_refs.is_empty() {
                        if let Zval::Ref(cell) = &self.frames[top].slots[*s as usize] {
                            let cell = Rc::clone(cell);
                            let strict = self.frames[top].module.strict;
                            let v = self.frames[top].stack.pop().expect("StoreSlot value");
                            let v = self.typed_ref_assign(&cell, v, strict)?;
                            self.frames[top].stack.push(v);
                        }
                    }
                    let v = self.frames[top].stack.pop().expect("StoreSlot on empty stack");
                    let old = store_slot(&mut self.frames[top].slots[*s as usize], v);
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
                Op::BindGlobalDyn => {
                    // `global $$x`: alias the same-named local to the global
                    // cell resolved by the runtime name. An object name goes
                    // through the throwing conversion (`global ${new stdClass}`
                    // is Zend's catchable "could not be converted to string").
                    let nv = self.frames[top].stack.pop().expect("BindGlobalDyn name");
                    let nv = nv.deref_clone();
                    let name = match &nv {
                        Zval::Object(_) => self.vm_stringify(&nv)?.as_bytes().to_vec(),
                        other => {
                            convert::to_zstr_cast(other, &mut self.diags).as_bytes().to_vec()
                        }
                    };
                    self.bind_global_dyn(top, &name)?;
                }
                Op::StaticGuard { id, skip } => {
                    // First execution of this `static` declaration falls through to
                    // run the initialiser; every later one skips to the alias. A
                    // closure keys its own per-instance storage (fresh statics per
                    // closure object); everything else the program-global `statics`.
                    let exists = match self.frames[top].closure_id() {
                        Some(cid) => self.closure_statics.contains_key(&(cid, *id)),
                        None => self.statics[*id as usize].is_some(),
                    };
                    if exists {
                        self.frames[top].ip = *skip as usize;
                    }
                }
                Op::StaticStore { id } => {
                    let v = self.frames[top].stack.pop().expect("StaticStore on empty stack");
                    let cell = Rc::new(RefCell::new(v));
                    let old = match self.frames[top].closure_id() {
                        Some(cid) => self.closure_statics.insert((cid, *id), cell),
                        None => self.statics[*id as usize].replace(cell),
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
                    let cell = match self.frames[top].closure_id() {
                        Some(cid) => Rc::clone(
                            self.closure_statics
                                .get(&(cid, *id))
                                .expect("StaticAlias reached before its StaticStore"),
                        ),
                        None => Rc::clone(
                            self.statics[*id as usize]
                                .as_ref()
                                .expect("StaticAlias reached before its StaticStore"),
                        ),
                    };
                    // Rebinding the slot to the static cell drops whatever it held
                    // (`$x = new T; static $x;` discards the temporary T here).
                    let old = std::mem::replace(&mut self.frames[top].slots[*slot as usize], Zval::Ref(cell));
                    self.gc_note(&old);
                }
                Op::LoadGlobal(s) => {
                    // `$GLOBALS['x']` read: the global lives in the script frame.
                    let v = read_slot(&self.frames[0].slots[*s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreGlobal(s) => {
                    // `$GLOBALS['x'] = …`: write/create the global in the script frame.
                    let v = self.frames[top].stack.pop().expect("StoreGlobal on empty stack");
                    let old = store_slot(&mut self.frames[0].slots[*s as usize], v);
                    self.gc_note(&old);
                }
                Op::IncDecGlobal { slot, inc, pre } => {
                    let i = *slot as usize;
                    if matches!(self.frames[0].slots[i], Zval::Undef) {
                        self.frames[0].slots[i] = Zval::Null;
                    }
                    // Value snapshot + write-through (see IncDecSlot: a reference
                    // slot must yield the pre-increment VALUE and keep aliases).
                    let old = self.frames[0].slots[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), *inc)?;
                    // PHP raises the diagnostic *before* writing the result back, so a
                    // `set_error_handler` runs here (it may throw, unwinding this op, or
                    // mutate the variable — which the write-back below then overwrites).
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.frames[0].slots[i], newv.clone());
                    let pushed = if *pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::LoadSuperglobal(idx) => {
                    // `$_SERVER` (&c.) read: the value lives in the VM-level store,
                    // resolved by name — correct from any unit/frame. Silent like
                    // `LoadGlobal`.
                    let v = read_slot(&self.superglobals[*idx as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSuperglobal(idx) => {
                    let v = self.frames[top].stack.pop().expect("StoreSuperglobal on empty stack");
                    let old = store_slot(&mut self.superglobals[*idx as usize], v);
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
                        // Undef must be tested on the RAW slot (through a ref
                        // cell): read_slot maps unset to NULL, so testing its
                        // result kept every unset global in the snapshot as
                        // null (GetBookmark's `unset($GLOBALS['link'])`,
                        // WP-17).
                        let raw = &self.frames[0].slots[i];
                        let is_undef = match raw {
                            Zval::Undef => true,
                            Zval::Ref(c) => matches!(&*c.borrow(), Zval::Undef),
                            _ => false,
                        };
                        if is_undef {
                            continue;
                        }
                        let v = read_slot(raw);
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
                    let i = *idx as usize;
                    if matches!(self.superglobals[i], Zval::Undef) {
                        self.superglobals[i] = Zval::Null;
                    }
                    // Value snapshot + write-through (see IncDecSlot).
                    let old = self.superglobals[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), *inc)?;
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.superglobals[i], newv.clone());
                    let pushed = if *pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::PushUndef => {
                    self.frames[top].stack.push(Zval::Undef);
                }
                Op::FillDefault { slot, skip } => {
                    // Default-parameter prologue (PAR): skip the default if the
                    // argument was supplied (the slot is not `Undef`).
                    if !matches!(self.frames[top].slots[*slot as usize], Zval::Undef) {
                        self.frames[top].ip = *skip as usize;
                    }
                }
                Op::CoerceParam { slot, hint } => {
                    // Coerce a just-filled scalar-hinted default (step 14). A valid
                    // constant default always coerces; keep the value otherwise.
                    let v = self.frames[top].slots[*slot as usize].clone();
                    if let Ok(c) = coerce_to_hint(v, &hint, &mut self.diags, self.frames[top].module.strict) {
                        self.frames[top].slots[*slot as usize] = c;
                    }
                }
                Op::CheckArity { required, exactly } => {
                    let argc = self.frames[top].argc;
                    if argc < *required {
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
                        let qualifier = if *exactly { "exactly" } else { "at least" };
                        let msg = format!(
                            "Too few arguments to function {name}(), {argc} passed in {} on line {line} and {qualifier} {required} expected",
                            String::from_utf8_lossy(&self.module.file)
                        );
                        return Err(PhpError::ArgumentCountError(msg));
                    }
                }
                Op::IncDecSlot { slot, inc, pre } => {
                    let i = *slot as usize;
                    if matches!(self.frames[top].slots[i], Zval::Undef) {
                        self.frames[top].slots[i] = Zval::Null;
                    }
                    // Snapshot the VALUE (deref a reference slot): the postfix
                    // result is the pre-increment value, not the live cell —
                    // `$c++ === 0` on a by-ref captured `$c` compared the
                    // already-incremented cell. The write-back goes *through*
                    // the reference so aliases keep seeing the update.
                    let old = self.frames[top].slots[i].deref_clone();
                    let (newv, diags) = self.compute_incdec(old.clone(), *inc)?;
                    // Raise before write-back (see IncDecGlobal).
                    self.raise_diags(diags, self.cur_line(top))?;
                    let _ = store_slot(&mut self.frames[top].slots[i], newv.clone());
                    let pushed = if *pre { newv } else { old };
                    self.frames[top].stack.push(pushed);
                }
                Op::Binary(b) => {
                    let r = self.binary_value(top, *b)?;
                    self.frames[top].stack.push(r);
                }
                Op::CmpJmp { op, addr, when } => {
                    // Fused compare+branch (WP-32): identical semantics to
                    // Binary+JumpIfX by construction (shared binary_value;
                    // to_bool on a Bool is free and emits no diag) — minus
                    // the Zval::Bool stack round-trip and one dispatch.
                    let r = self.binary_value(top, *op)?;
                    if convert::to_bool(&r, &mut self.diags) == *when {
                        self.frames[top].ip = *addr as usize;
                    }
                }
                Op::Unary(u) => {
                    let a = self.frames[top].stack.pop().expect("Unary operand");
                    let r = self.apply_unop_ovl(*u, &a)?;
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
                    // are pure value conversions — EXCEPT SimpleXML, whose
                    // cast_object handler converts the node TEXT for (int)/
                    // (float) where a generic object casts to 1 (WP-17,
                    // export_wp's `(int) $item->post_id`).
                    let r = if matches!(k, CastKind::Object) {
                        self.object_cast(a)?
                    } else if matches!(k, CastKind::Int | CastKind::Float)
                        && deref_object(&a).is_some_and(|o| {
                            self.class_index.get(&b"simplexmlelement"[..]).is_some_and(|&t| {
                                is_instance_of(
                                    &self.classes,
                                    self.stringable_id,
                                    o.borrow().class_id as usize,
                                    t,
                                )
                            })
                        })
                    {
                        let s = self.vm_stringify(&a.deref_clone())?;
                        apply_cast(*k, &Zval::Str(s), &mut self.diags)
                    } else if matches!(k, CastKind::Int | CastKind::Float)
                        && deref_object(&a).is_some_and(|o| {
                            let cid = o.borrow().class_id as usize;
                            [&b"tidy"[..], &b"tidynode"[..]].iter().any(|n| {
                                self.class_index.get(*n).is_some_and(|&t| {
                                    is_instance_of(
                                        &self.classes,
                                        self.stringable_id,
                                        cid,
                                        t,
                                    )
                                })
                            })
                        })
                    {
                        // tidy_doc_cast_handler / tidy_node_cast_handler:
                        // numeric casts yield 0, silently.
                        match k {
                            CastKind::Int => Zval::Long(0),
                            _ => Zval::Double(0.0),
                        }
                    } else {
                        apply_cast(*k, &a, &mut self.diags)
                    };
                    self.frames[top].stack.push(r);
                }
                Op::Jump(addr) => {
                    self.frames[top].ip = *addr as usize;
                }
                Op::JumpIfFalse(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfFalse cond");
                    if !convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = *addr as usize;
                    }
                }
                Op::JumpIfTrue(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfTrue cond");
                    if convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = *addr as usize;
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
                                    let m = self.class_mod(defc);
                                    let mut frame = self.pooled_frame(callee, m);
                                    frame.this = Some(target.clone());
                                    frame.class = Some(defc);
                                    frame.static_class = Some(cid);
                                    frame.flags.set(FrameFlags::RET_STRINGIFY, true);
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
                        // A Closure / Generator has no `__toString`: the explicit
                        // cast throws like any such object (the infallible
                        // echo/concat funnel still warns — D-19.18).
                        Zval::Closure(_) => {
                            return Err(PhpError::Error(
                                "Object of class Closure could not be converted to string".into(),
                            ));
                        }
                        Zval::Generator(_) => {
                            return Err(PhpError::Error(
                                "Object of class Generator could not be converted to string".into(),
                            ));
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
                        self.frames[top].ip = *addr as usize;
                    } else {
                        self.frames[top].stack.pop();
                    }
                }
                Op::JumpIfNull(addr) => {
                    // Peek; the value is kept either way (nullsafe `?->`).
                    if matches!(self.frames[top].stack.last(), Some(Zval::Null | Zval::Undef)) {
                        self.frames[top].ip = *addr as usize;
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
                        let k = coerce_key_diag(&key, &mut self.diags)
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
                    // WP-33 T1b guard: Array base + Long/Str key (census:
                    // 98.8% of FetchDim in the WP suites) resolved with ONE
                    // canonical-key lookup, skipping the ArrayAccess probe
                    // and the diag-ful funnel. Key canonicalization matches
                    // coerce_key ("5" → Int(5) via Key::from_zstr); float
                    // keys (deprecation), Bool/Null/Ref keys and Ref/Str/
                    // object bases all fail the guard → generic. A MISS
                    // (key absent) also falls through so the "Undefined
                    // array key" warning keeps its single source of truth.
                    if let Zval::Array(a) = &base {
                        let k = match &key {
                            Zval::Long(i) => Some(Key::Int(*i)),
                            Zval::Str(s) => Some(Key::from_zstr(s)),
                            _ => None,
                        };
                        if let Some(v) = k.as_ref().and_then(|k| a.get(k)) {
                            let v = v.deref_clone();
                            // The pending-diag flush MUST stay on the hit
                            // path (trap #1): a warning staged by the
                            // preceding op surfaces AT this read, and a
                            // throwing error handler unwinds from HERE.
                            if self.diags_rendered < self.diags.len() {
                                let line = self.cur_line(top);
                                self.flush_diags(line)?;
                                let top = self.frames.len() - 1;
                                self.frames[top].stack.push(v);
                                continue;
                            }
                            self.frames[top].stack.push(v);
                            continue;
                        }
                    }
                    // `$o[$k]` on an ArrayAccess object dispatches `offsetGet` (step 51).
                    if let Some(recv) = self.as_arrayaccess(&base) {
                        self.enter_object_method(recv, b"offsetGet", vec![key], RetMode::Stack)?;
                        continue;
                    }
                    let v = read_dim_warn(&base, &key, &mut self.diags)?;
                    // Deliver the undefined-key warning AT the faulting read:
                    // Zend raises it synchronously, and a throwing user error
                    // handler (PHPUnit's expectWarning) must unwind from THIS
                    // statement — a deferred flush surfaced it at a later op,
                    // outside the test's expectation scope (WP-18, Tests_Locale).
                    if self.diags_rendered < self.diags.len() {
                        let line = self.cur_line(top);
                        self.flush_diags(line)?;
                        let top = self.frames.len() - 1;
                        self.frames[top].stack.push(v);
                        continue;
                    }
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
                    // WP-33 T1b guard (silent twin of the FetchDim guard):
                    // for an Array base with a Long/Str key the coalesce
                    // read is get-or-null with NO diagnostics by semantics,
                    // so hit AND miss resolve inline; other keys/bases →
                    // generic (float-key deprecation fires there via the
                    // shared coerce funnel — read_dim_nullable is silent,
                    // matching today's behavior).
                    if let Zval::Array(a) = &base {
                        let k = match &key {
                            Zval::Long(i) => Some(Key::Int(*i)),
                            Zval::Str(s) => Some(Key::from_zstr(s)),
                            _ => None,
                        };
                        if let Some(k) = k {
                            let v = a.get(&k).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
                            self.frames[top].stack.push(v);
                            continue;
                        }
                    }
                    // `$aa[$k] ?? $default` on an ArrayAccess object dispatches
                    // the protocol quietly: !offsetExists → null (the coalesce
                    // takes the default), else offsetGet (a null result also
                    // falls through to the default).
                    if let Some(recv) = self.as_arrayaccess(&base) {
                        let ex = self
                            .call_method_sync(recv.clone(), b"offsetExists", vec![key.clone()])?;
                        let v = if convert::is_true_silent(&ex.deref_clone()) {
                            self.call_method_sync(recv, b"offsetGet", vec![key])?.deref_clone()
                        } else {
                            Zval::Null
                        };
                        self.frames[top].stack.push(v);
                        continue;
                    }
                    self.frames[top].stack.push(read_dim_nullable(&base, &key));
                }
                Op::AssignPath { base, nkeys, append } => {
                    let value = self.frames[top].stack.pop().expect("AssignPath value");
                    let mut keys = self.pop_keys(top, *nkeys);
                    // `$o[$k] = v` / `$o[] = v` on an ArrayAccess object dispatches
                    // `offsetSet` (a single step only); the expression yields `v`.
                    if nkeys + *append as u32 == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(*base, top)) {
                            let key = if *append { Zval::Null } else { keys.pop().expect("set key") };
                            self.frames[top].stack.push(value.clone());
                            self.enter_object_method(recv, b"offsetSet", vec![key, value], RetMode::Discard)?;
                            continue;
                        }
                    }
                    let last = if *append {
                        Last::Append { value }
                    } else {
                        Last::Set { key: keys.pop().expect("AssignPath key"), value }
                    };
                    let result = self.path_op(*base, top, keys, last)?;
                    self.frames[top].stack.push(result);
                }
                Op::AssignOpPath { base, nkeys, op } => {
                    let rhs = self.frames[top].stack.pop().expect("AssignOpPath rhs");
                    let mut keys = self.pop_keys(top, *nkeys);
                    let key = keys.pop().expect("AssignOpPath key");
                    let result = self.path_op(*base, top, keys, Last::OpSet { key, op: *op, rhs })?;
                    self.frames[top].stack.push(result);
                }
                Op::IncDecPath { base, nkeys, inc, pre } => {
                    let mut keys = self.pop_keys(top, *nkeys);
                    let key = keys.pop().expect("IncDecPath key");
                    let result = self.path_op(*base, top, keys, Last::IncDec { key, inc: *inc, pre: *pre })?;
                    self.frames[top].stack.push(result);
                }
                Op::IssetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, *nkeys);
                    // `isset($o[$k])` on an ArrayAccess object is `offsetExists($k)`
                    // (a single step only; it does not call `offsetGet`).
                    if *nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(*base, top)) {
                            let key = keys.into_iter().next().expect("isset key");
                            self.enter_object_method(recv, b"offsetExists", vec![key], RetMode::Stack)?;
                            continue;
                        }
                    }
                    // Nested form: BP_VAR_IS walk (an intermediate ArrayAccess
                    // dispatches offsetExists/offsetGet; the protocol runs on
                    // an ArrayAccess leaf).
                    let set = match self.dim_is_walk(*base, top, &keys)? {
                        super::DimIsLeaf::Missing => false,
                        super::DimIsLeaf::Aa(recv, key) => {
                            let r = self.call_method_sync(recv, b"offsetExists", vec![key])?;
                            convert::is_true_silent(&r.deref_clone())
                        }
                        super::DimIsLeaf::Raw(v) => {
                            matches!(v, Some(v) if !matches!(v, Zval::Null | Zval::Undef))
                        }
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::EmptyPath { base, nkeys } => {
                    let keys = self.pop_keys(top, *nkeys);
                    // `empty($o[$k])` on an ArrayAccess object: `!offsetExists($k)`
                    // short-circuits to empty (so `offsetGet` — which may throw for
                    // an absent key, e.g. WeakMap — is skipped); otherwise
                    // `!truthy(offsetGet($k))`. A single step only.
                    if *nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(*base, top)) {
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
                    // Nested form: same BP_VAR_IS walk as `isset` above.
                    let empty = match self.dim_is_walk(*base, top, &keys)? {
                        super::DimIsLeaf::Missing => true,
                        super::DimIsLeaf::Aa(recv, key) => {
                            let exists = self
                                .call_method_sync(recv.clone(), b"offsetExists", vec![key.clone()])?;
                            if convert::is_true_silent(&exists.deref_clone()) {
                                let v = self.call_method_sync(recv, b"offsetGet", vec![key])?;
                                !convert::is_true_silent(&v.deref_clone())
                            } else {
                                true
                            }
                        }
                        super::DimIsLeaf::Raw(v) => match v {
                            Some(v) => !convert::is_true_silent(&v),
                            None => true,
                        },
                    };
                    self.frames[top].stack.push(Zval::Bool(empty));
                }
                Op::UnsetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, *nkeys);
                    // `unset($o[$k])` on an ArrayAccess object is `offsetUnset($k)`
                    // (a single step only).
                    if *nkeys == 1 {
                        if let Some(recv) = self.as_arrayaccess(self.base_cell(*base, top)) {
                            let key = keys.into_iter().next().expect("unset key");
                            self.enter_object_method(recv, b"offsetUnset", vec![key], RetMode::Discard)?;
                            continue;
                        }
                    }
                    if let Some((recv, key)) = self.dim_aa_leaf(*base, top, &keys) {
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
                            DimBase::Local(s) => std::mem::replace(&mut self.frames[top].slots[*s as usize], Zval::Undef),
                            DimBase::Global(s) => std::mem::replace(&mut self.frames[0].slots[*s as usize], Zval::Undef),
                            DimBase::Superglobal(i) => std::mem::replace(&mut self.superglobals[*i as usize], Zval::Undef),
                        })
                    } else {
                        let cell = match base {
                            DimBase::Local(s) => &mut self.frames[top].slots[*s as usize],
                            DimBase::Global(s) => &mut self.frames[0].slots[*s as usize],
                            DimBase::Superglobal(i) => &mut self.superglobals[*i as usize],
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
                    let cell = make_cell(ref_base_mut(
                        &mut self.frames,
                        &mut self.superglobals,
                        top,
                        *source,
                    ));
                    let value = cell.borrow().clone();
                    *ref_base_mut(&mut self.frames, &mut self.superglobals, top, *target) =
                        Zval::Ref(cell);
                    self.frames[top].stack.push(value);
                }
                Op::PushRef(slot) => {
                    // REF-2: promote the local to a shared cell and push the ref;
                    // the next `Op::Call` binds it into the by-ref callee slot.
                    let cell = make_cell(&mut self.frames[top].slots[*slot as usize]);
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
                    let bound_this = if *bind_this { self.frames[top].this.clone() } else { None };
                    let m = self.frames[top].module;
                    // A closure index always resolves in the unit that compiled the
                    // body — except a trait method flattened into a class from
                    // *another* unit: its closure indices point at the trait's unit,
                    // not the consumer's (cross-unit trait-closure relocation is not
                    // yet implemented). Surface a catchable error rather than panic.
                    let Some(func) = m.closures.get(*fn_idx as usize) else {
                        return Err(PhpError::Error(format!(
                            "closure from a trait used across files is not yet supported \
                             (closure #{fn_idx} of {} in unit '{}' [{} closures])",
                            String::from_utf8_lossy(&self.frames[top].func.name),
                            String::from_utf8_lossy(&m.file),
                            m.closures.len(),
                        )));
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
                    // `static::` inside the body keeps the CREATING frame's
                    // late-static-binding class (Child::getCb() returning a
                    // closure whose `static::` is Child, WP-17).
                    let lsb = self.frames[top].static_class.or(scope);
                    let cl = Closure {
                        fn_idx: *fn_idx as usize,
                        captures: bound,
                        named: None,
                        bound_this,
                        id,
                        info,
                        module_id,
                        scope,
                        is_static: !bind_this,
                        lsb,
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
                        .map(|f| closure_params(f))
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
                        lsb: None,
                    };
                    self.frames[top].stack.push(Zval::Closure(Rc::new(cl)));
                }
                Op::CallValue { argc } => {
                    let n = *argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("CallValue argument"));
                    }
                    args.reverse();
                    let callee = self.frames[top].stack.pop().expect("CallValue callee");
                    self.invoke_value(callee, args)?;
                }
                Op::CallNsFallback { name, fallback, argc } => {
                    let n = *argc as usize;
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
                Op::CallNsFallbackArgs { name, fallback } => {
                    // Spread on the two-step namespaced lookup: arguments from a
                    // runtime array, resolution deferred like `CallNsFallback`.
                    let argsval =
                        self.frames[top].stack.pop().expect("CallNsFallbackArgs array");
                    let args = args_from_array_value(argsval);
                    self.invoke_named_fallback(&name, &fallback, args)?;
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
                                store_slot(&mut self.frames[top].slots[*slot as usize], exc);
                            self.gc_note(&displaced);
                        } else {
                            // A capture-less catch drops the throwable right here.
                            self.gc_note(&exc);
                        }
                        self.frames[top].ip = *body as usize;
                    }
                    // else: fall through to the next CatchMatch / Rethrow.
                }
                Op::EndFinally { after } => {
                    // EXC-2/2b: resolve the finally's pending action. A propagating
                    // exception wins; then a parked return (push the value and fall
                    // through to the trailing `Ret`); then a parked break/continue
                    // (jump to its loop target); otherwise skip past the `try`.
                    if let Some(v) =
                        self.frames[top].ext_opt_mut().and_then(|e| e.pending_throw.take())
                    {
                        return Err(PhpError::Thrown(v));
                    }
                    match self.frames[top].ext_opt_mut().and_then(|e| e.pending_transfer.take()) {
                        Some(Transfer::Return(val)) => {
                            self.frames[top].stack.push(val);
                            // fall through to the `Ret` emitted right after this op
                        }
                        Some(Transfer::Jump(addr)) => {
                            self.frames[top].ip = addr as usize;
                        }
                        None => {
                            self.frames[top].ip = *after as usize;
                        }
                    }
                }
                Op::ParkReturn => {
                    let v = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    self.frames[top].ext_mut().pending_transfer = Some(Transfer::Return(v));
                }
                Op::ParkJump(addr) => {
                    self.frames[top].ext_mut().pending_transfer = Some(Transfer::Jump(*addr));
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
                    let cell = self.make_ref_cell(top, *base, &steps, keys)?;
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::PushArgPlace { base, steps, name } => {
                    // SEND_VAR_EX place argument (FETCH_DIM/OBJ_FUNC_ARG):
                    // capture the evaluated keys and defer the fetch to the
                    // dispatch binder, which W-fetches (by-ref param) or
                    // R-fetches.
                    let keys = self.pop_field_keys(top, &steps);
                    let pbase = match base {
                        FieldBase::Local(s) => ArgPlaceBase::Local(*s),
                        FieldBase::Global(s) => ArgPlaceBase::Global(*s),
                        FieldBase::Superglobal(i) => ArgPlaceBase::Superglobal(*i),
                        FieldBase::This => ArgPlaceBase::This,
                    };
                    let psteps: Box<[ArgPlaceStep]> = steps
                        .iter()
                        .map(|s| match s {
                            FieldStep::Index => ArgPlaceStep::Index,
                            FieldStep::Prop(n) => ArgPlaceStep::Prop(n.clone()),
                            FieldStep::Append => ArgPlaceStep::Append,
                            _ => unreachable!("PushArgPlace step"),
                        })
                        .collect();
                    let pname: Box<[u8]> = match &self.frames[top].func.consts[*name as usize] {
                        crate::bytecode::Const::Str(b) => Box::from(b.as_bytes()),
                        _ => Box::default(),
                    };
                    self.frames[top].stack.push(Zval::ArgPlace(Rc::new(ArgPlace {
                        base: pbase,
                        steps: psteps,
                        keys,
                        name: pname,
                    })));
                }
                Op::BindRefTo { base, steps } => {
                    // REF-4: pop the reference, bind the target place to its cell,
                    // and push the aliased value (the assignment's result).
                    // Binding a reference *into* a hooked property is forbidden.
                    if self.field_starts_at_hook(*base, top, &steps) {
                        if steps.len() > 1 {
                            // Navigating *into* the property: a `&get` hook's cell
                            // is an addressable root — bind the leaf inside it.
                            if let Some(root) = self.byref_hook_root(*base, top, &steps)? {
                                let top_val = self.frames[top].stack.pop().expect("BindRefTo value");
                                let cell = match top_val {
                                    Zval::Ref(rc) => rc,
                                    other => Rc::new(RefCell::new(other)),
                                };
                                let value = cell.borrow().clone();
                                let keys = self.pop_field_keys(top, &steps);
                                self.field_set_in_root(root, top, &steps[1..], keys, Zval::Ref(cell), true, false)?;
                                self.frames[top].stack.push(value);
                                continue;
                            }
                        } else {
                            // The write fetch of the rebind target runs a `&get`
                            // hook first (observable side effects, bug007) — then
                            // PHP still rejects rebinding the property slot.
                            let _ = self.byref_hook_root(*base, top, &steps)?;
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
                    let lazy_root = self.field_lazy_root(*base, top, &steps, &keys, true)?;
                    {
                        let target = match &lazy_root {
                            Some(r) => Some(r.clone()),
                            None => match base {
                                FieldBase::Local(s) => self.frames[top].slots.get(*s as usize).map(|v| v.deref_clone()),
                                FieldBase::Global(s) => self.frames[0].slots.get(*s as usize).map(|v| v.deref_clone()),
                                FieldBase::Superglobal(i) => self.superglobals.get(*i as usize).map(|v| v.deref_clone()),
                                FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                            },
                        };
                        self.bind_ref_typed_check(target.as_ref(), &steps, &mut keys, &cell)?;
                    }
                    let value = cell.borrow().clone();
                    if let Some(root) = lazy_root {
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, Zval::Ref(cell), true, false)?;
                    } else if steps.is_empty() {
                        // A step-less base is rebound directly (not written
                        // through), matching `eval::bind_ref_target`.
                        let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, *base)?;
                        *base_cell = Zval::Ref(cell);
                    } else {
                        self.field_set_mode(*base, top, &steps, keys, Zval::Ref(cell), true, false)?;
                    }
                    self.frames[top].stack.push(value);
                }
                Op::BindRefToChecked { base, steps } => {
                    // `$t = &m()` for a method/static call: the callee's by-ref-ness
                    // is only known now. A non-`Ref` source means the callee did not
                    // return by reference — raise the notice, then bind a copy.
                    if self.field_starts_at_hook(*base, top, &steps) {
                        if steps.len() == 1 {
                            // Run a `&get` hook's observable side effects before
                            // rejecting the rebind (mirrors `Op::BindRefTo`).
                            let _ = self.byref_hook_root(*base, top, &steps)?;
                        } else if let Some(root) = self.byref_hook_root(*base, top, &steps)? {
                            let top_val = self.frames[top].stack.pop().expect("BindRefToChecked value");
                            let cell = match top_val {
                                Zval::Ref(rc) => rc,
                                other => {
                                    self.diags.push(Diag::Notice(
                                        "Only variables should be assigned by reference".to_string(),
                                    ));
                                    let line = self.cur_line(top);
                                    self.flush_diags(line)?;
                                    Rc::new(RefCell::new(other))
                                }
                            };
                            let value = cell.borrow().clone();
                            let keys = self.pop_field_keys(top, &steps);
                            self.field_set_in_root(root, top, &steps[1..], keys, Zval::Ref(cell), true, false)?;
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
                            let line = self.cur_line(top);
                            self.flush_diags(line)?;
                            Rc::new(RefCell::new(other))
                        }
                    };
                    let value = cell.borrow().clone();
                    let keys = self.pop_field_keys(top, &steps);
                    if steps.is_empty() {
                        let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, *base)?;
                        *base_cell = Zval::Ref(cell);
                    } else {
                        self.field_set_mode(*base, top, &steps, keys, Zval::Ref(cell), true, false)?;
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
                            self.frames[top].ip = *end as usize;
                        } else {
                            store_slot(&mut self.frames[top].slots[*value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[*ks as usize], k);
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
                            None => self.frames[top].ip = *end as usize,
                            Some((k, v)) => {
                                store_slot(&mut self.frames[top].slots[*value as usize], v);
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[*ks as usize], k);
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
                                    self.frames[top].ip = *end as usize;
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
                                store_slot(&mut self.frames[top].slots[*value as usize], v.deref_clone());
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[*ks as usize], k.deref_clone());
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
                        None => self.frames[top].ip = *end as usize,
                        Some((k, v)) => {
                            // Deref at bind time: a reference element snapshots its
                            // cell and is read live here. `store_slot` writes
                            // *through* a value slot that is itself a reference (the
                            // lingering-ref gotcha), matching the tree-walker.
                            store_slot(&mut self.frames[top].slots[*value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[*ks as usize], k);
                            }
                        }
                    }
                }
                Op::IterInitRef(source) => {
                    // REF-3: snapshot the source's keys once; each step rebinds the
                    // live element/property by reference. A plain object binds each
                    // visible property by reference (ObjRefs); an array uses ByRef.
                    let src = self.frames[top].slots[*source as usize].deref_clone();
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
                    let keys = ref_array_keys(&self.frames[top].slots[*source as usize]);
                    self.frames[top].iters.push(IterState::ByRef { source: *source, keys, pos: 0 });
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
                            None => self.frames[top].ip = *end as usize,
                            Some((cell, keyname)) => {
                                if let Some(ks) = key {
                                    store_slot(&mut self.frames[top].slots[*ks as usize], Zval::Str(PhpStr::new(keyname.to_vec())));
                                }
                                self.frames[top].slots[*value as usize] = Zval::Ref(cell);
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
                        None => self.frames[top].ip = *end as usize,
                        Some((src, k)) => {
                            let cell = elem_cell(&mut self.frames[top].slots[src as usize], &k);
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[*ks as usize], key_to_zval(&k));
                            }
                            // Direct overwrite, *not* `store_slot`: on later
                            // iterations the value slot is itself a `Zval::Ref` to
                            // the previous element, and writing through it would
                            // corrupt that element (D-R13).
                            self.frames[top].slots[*value as usize] = Zval::Ref(cell);
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
                    let idx = *func as usize;
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
                    if let Some((key, lt)) = module.conditional_traits.get(*idx as usize) {
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
                    let cc = self.classes[*cid];
                    let key = cc.name.to_ascii_lowercase();
                    if self.class_index.contains_key(&key) {
                        let kind = if cc.enum_cases.is_empty() { "class" } else { "enum" };
                        return Err(PhpError::Error(format!(
                            "Cannot declare {} {}, because the name is already in use",
                            kind,
                            String::from_utf8_lossy(&cc.name)
                        )));
                    }
                    self.class_index.insert(key, *cid);
                    self.serializable_link_check(*cid)?;
                }
                Op::DeclareDeferred { idx } => {
                    // A late-bound declaration statement was reached (its
                    // supertype was unresolvable when the unit was lowered):
                    // bind it now — re-lower the snippet against the current
                    // class image, or throw PHP's `… "X" not found` Error.
                    self.run_deferred(*idx as usize, false)?;
                }
                Op::NewAnonDeferred { idx } => {
                    // Late-bound anonymous class: bind and instantiate at the
                    // expression's execution point, constructor arguments
                    // re-evaluated in this frame's bridged scope.
                    let v = self.run_deferred(*idx as usize, true)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(v);
                }
                Op::Call { func, argc } => {
                    let m = self.frames[top].module;
                    let callee = &m.functions[*func as usize];
                    // Pop argc args (pushed left-to-right) and bind them to the
                    // callee's leading slots. The caller's `ip` is already past
                    // the Call, so it resumes correctly once the callee returns.
                    let n = *argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("call argument"));
                    }
                    args.reverse();
                    let mut frame = self.pooled_frame(callee, m);
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
                    let callee = &m.functions[*func as usize];
                    let frame = if named.is_empty() {
                        let mut frame = self.pooled_frame(callee, m);
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
                    let pos = self.pop_keys(top, *positional);
                    let m = self.frames[top].module;
                    let callee = &m.functions[*func as usize];
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
                    let callee = &m.functions[*func as usize];
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
                    let mut args = self.pop_keys(top, *argc); // pops argc, source order
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
                    if *argc == 1
                        && (name[..] == *b"count" || name[..] == *b"sizeof")
                    {
                        if let Some(obj) = self.as_countable(&args[0]) {
                            let n = self.call_method_sync(obj, b"count", Vec::new())?;
                            self.frames[top].stack.push(n);
                            continue;
                        }
                    }
                    // A userland stream wrapper (stream_wrapper_register): a file
                    // op whose first argument is such a resource dispatches to the
                    // wrapper object's stream_* methods (VM re-entrant), never the
                    // pure value builtin. Only fires for UserStream resources, so
                    // ordinary file I/O is untouched.
                    if *argc >= 1 && is_user_stream_op(&name) {
                        if let Some(rc) = user_stream_rc(&args[0]) {
                            let line = self.cur_line(top);
                            self.flush_diags(line)?;
                            let result = self.user_stream_op(&name, rc, &args)?;
                            self.flush_diags(line)?;
                            self.frames[top].stack.push(result);
                            continue;
                        }
                    }
                    // `file_get_contents("scheme://…")` on a registered wrapper.
                    if *argc >= 1 && name[..] == *b"file_get_contents" {
                        if let Some(path) = user_wrapper_url(&args[0], &self.stream_wrappers) {
                            let line = self.cur_line(top);
                            self.flush_diags(line)?;
                            let result = self.user_wrapper_get_contents(&path)?;
                            self.flush_diags(line)?;
                            self.frames[top].stack.push(result);
                            continue;
                        }
                    }
                    // Stat-family / image / exif builtins on a wrapper URL:
                    // url_stat or read-to-EOF through the wrapper object.
                    if *argc >= 1 && is_user_wrapper_path_op(&name) {
                        if let Some(path) = user_wrapper_url(&args[0], &self.stream_wrappers) {
                            let line = self.cur_line(top);
                            self.flush_diags(line)?;
                            let result = self.user_wrapper_path_op(&name, &path, &args)?;
                            self.flush_diags(line)?;
                            self.frames[top].stack.push(result);
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
                    } else if value_builtin_string_coerces_deep(&name, &args) {
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
                    let args = self.pop_keys(top, *argc);
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
                    let rest = self.pop_keys(top, *argc);
                    let result = self.dispatch_host_builtin_ref(&name, *slot, rest)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinOut { name, out_slot, out_index, out_slot2, out_index2, argc } => {
                    // A host builtin with a by-reference output parameter
                    // (`preg_match`/`preg_match_all`'s `&$matches`): dispatch with all
                    // args by value, then write the produced out-value into `out_slot`.
                    // `exec` additionally writes `&$result_code` into `out_slot2`.
                    let _ = out_index2;
                    let mut args = self.pop_keys(top, *argc);
                    // `exec` *appends* to a pre-existing `&$output` array, so it
                    // needs that argument's current value (the compiler pushed a
                    // placeholder `null` there). Read it straight from the slot —
                    // no VarRead op, so an undefined variable warns nothing. Other
                    // out-param builtins ignore this argument.
                    if name[..] == *b"exec" {
                        if let Some(slot) = out_slot {
                            if let Some(a) = args.get_mut(*out_index as usize) {
                                *a = self.frames[top].slots[*slot as usize].deref_clone();
                            }
                        }
                    }
                    // Flush any diagnostic this builtin pushes at its call line
                    // (like `CallHostBuiltin` does) so e.g. stream_socket_client's
                    // connect Warning renders at the call, not the next statement.
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let (result, out_val, out_val2) =
                        self.dispatch_host_builtin_out(&name, args, *out_index as usize)?;
                    self.flush_diags(line)?;
                    let top = self.frames.len() - 1;
                    if let Some(slot) = out_slot {
                        match &mut self.frames[top].slots[*slot as usize] {
                            Zval::Ref(rc) => *rc.borrow_mut() = out_val,
                            cell => *cell = out_val,
                        }
                    }
                    if let (Some(slot), Some(v2)) = (out_slot2, out_val2) {
                        match &mut self.frames[top].slots[*slot as usize] {
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
                    let args = self.pop_keys(top, *argc);
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
                Op::CallArrayMultisort { arg_slots, argc } => {
                    // All arguments by value; the sorted arrays are written back
                    // into their captured variable slots.
                    let args = self.pop_keys(top, *argc);
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let (result, sorted) = self.ho_array_multisort(args)?;
                    let top = self.frames.len() - 1;
                    for (i, arr) in sorted.into_iter().enumerate() {
                        if let (Some(slot), Some(arr)) =
                            (arg_slots.get(i).copied().flatten(), arr)
                        {
                            match &mut self.frames[top].slots[slot as usize] {
                                Zval::Ref(rc) => *rc.borrow_mut() = arr,
                                cell => *cell = arr,
                            }
                        }
                    }
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRef { name, slot, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let rest = self.pop_keys(top, *argc);
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
                        let roots = ref_stringify_roots(&self.frames[top].slots[*slot as usize].clone());
                        self.compute_stringify(&roots, false)?
                    } else {
                        std::collections::HashMap::new()
                    };
                    let mut produced = Vec::new();
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[*slot as usize], &rest, &mut produced, &mut self.diags, &stringify);
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
                        let roots = ref_stringify_roots(&self.frames[top].slots[*slot as usize].clone());
                        self.compute_stringify(&roots, false)?
                    } else {
                        std::collections::HashMap::new()
                    };
                    let mut produced = Vec::new();
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[*slot as usize], &rest, &mut produced, &mut self.diags, &stringify);
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
                    let rest = self.pop_keys(top, *argc);
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
                    // A by-ref function that returned a plain value (the in-body
                    // notice already fired) still hands the caller a *reference*
                    // in Zend — so `$t = &f()` binds it silently instead of
                    // raising a second "assigned by reference" notice.
                    if func.by_ref && !func.is_generator && !matches!(ret, Zval::Ref(_)) {
                        ret = Zval::Ref(Rc::new(RefCell::new(ret)));
                    }
                    let ret_cell = self.frames[top].ret_cell.take();
                    let ret_bool = self.frames[top].flags.get(FrameFlags::RET_BOOL);
                    let ret_isset = self.frames[top].flags.get(FrameFlags::RET_ISSET);
                    let ret_stringify = self.frames[top].flags.get(FrameFlags::RET_STRINGIFY);
                    let ret_deref = self.frames[top].flags.get(FrameFlags::RET_DEREF);
                    let guard = self
                        .frames[top]
                        .ext_opt_mut()
                        .map(|e| std::mem::take(&mut e.guard_release))
                        .unwrap_or_default();
                    // A `clone`-driven `__clone` is finishing: revoke any remaining
                    // readonly re-init permission on the copy (PHP 8.3), so writes
                    // after the clone — or via a manual `__clone()` — fatal again.
                    if self.frames[top].flags.get(FrameFlags::CLONE_INIT) {
                        if let Some(Zval::Object(o)) = self.frames[top].this.clone() {
                            o.borrow_mut().readonly_clone_writable.clear();
                        }
                    }
                    let dead = self.frames.pop().expect("Ret pops the active frame");
                    if self.frames.is_empty() && !self.final_flush {
                        // The script `main` is returning: park its frame — the
                        // slots ARE the global variables, and Zend keeps them
                        // alive through the shutdown-function phase. Object
                        // destruction is unaffected (survivors are driven from
                        // `created`, not from this frame's drop).
                        self.retired_main = Some(dead);
                    } else {
                        // The returning frame's locals, leftover operands and `$this`
                        // release their references now: note any tracked objects so
                        // the next sweep reconsiders them (drives destruction of an
                        // object whose last reference was a returning function's local).
                        self.gc_note_frame(&dead);
                        self.recycle_frame(dead);
                    }
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
                    let key = if *has_key {
                        GenKey::Keyed(self.frames[top].stack.pop().expect("Yield key"))
                    } else {
                        GenKey::Auto
                    };
                    let gid = self.frames[top].gen_id().expect("Yield outside a generator frame");
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
                    if self.frames[top].ext().is_none_or(|e| e.yield_from.is_none()) {
                        let delegate = self.frames[top].stack.pop().expect("YieldFrom delegate");
                        match delegate.deref_clone() {
                            Zval::Array(_) => {
                                let entries = snapshot_entries(&delegate);
                                self.frames[top].ext_mut().yield_from =
                                    Some(YieldFromState::Array { entries, pos: 0 });
                            }
                            Zval::Generator(rc) => {
                                self.frames[top].ext_mut().yield_from =
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
                                        self.frames[top].ext_mut().yield_from =
                                            Some(YieldFromState::Gen { rc: Rc::clone(&rc), opaque: true });
                                        self.ensure_started(&rc)?;
                                    }
                                    TraversableSource::Entries(entries) => {
                                        self.frames[top].ext_mut().yield_from =
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
                        let sub = match self.frames[top].ext().and_then(|e| e.yield_from.as_ref()) {
                            Some(YieldFromState::Gen { rc, .. }) => Some(Rc::clone(rc)),
                            _ => None,
                        };
                        if let Some(rc) = sub {
                            self.resume_generator(&rc, sent)?;
                        }
                    }
                    // Take the next delegated `(key, value)`, or finish.
                    let step = match self.frames[top].ext_mut().yield_from.as_mut().unwrap() {
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
                                self.frames[top].gen_id().expect("YieldFrom outside a generator");
                            let frame = self.frames.pop().expect("generator frame to park");
                            self.generators.insert(gid, frame);
                            return Ok(RunExit::Yielded { key: GenKey::Verbatim(k), value: v });
                        }
                        None => {
                            // Delegation done: leave the delegate's return value (NULL
                            // for an array, the sub-generator's getReturn()) on the
                            // stack as the `yield from` expression's value.
                            let value = match self.frames[top].ext_mut().yield_from.take().unwrap() {
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
                    self.check_new_ctor_access(self.frames[top].class, *class)?;
                    let obj = self.alloc_object(*class)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocStatic => {
                    let cid = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" in the global scope".to_string())
                    })?;
                    self.check_new_ctor_access(self.frames[top].class, cid)?;
                    let obj = self.alloc_object(cid)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocDynamic => {
                    // `new $cls` (PAR): resolve the class reference at run time.
                    let classval = self.frames[top].stack.pop().expect("AllocDynamic class");
                    let cid = self.resolve_dynamic_class(&classval)?;
                    self.check_new_ctor_access(self.frames[top].class, cid)?;
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
                    let result = self.run_include(path_val, *mode)?;
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
                    // Opaque handle classes (GdImage) are uncloneable, like
                    // their internal PHP counterparts.
                    if crate::vm::is_opaque_handle_class(o.borrow().class_name.as_bytes()) {
                        let msg = format!(
                            "Trying to clone an uncloneable object of class {}",
                            String::from_utf8_lossy(o.borrow().class_name.as_bytes())
                        );
                        if let Some(cid) = self.class_index.get(&b"error"[..]).copied() {
                            let obj = self.synthesize_throwable(cid, &msg)?;
                            return Err(PhpError::Thrown(obj));
                        }
                        return Err(PhpError::Error(msg));
                    }
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
                            readonly_clone_writable: Vec::new(), typed_unset: Vec::new(),
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
                            readonly_clone_writable: Vec::new(), typed_unset: Vec::new(),
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
                        let m = self.class_mod(defc);
                        let mut frame = self.pooled_frame(callee, m);
                        frame.this = Some(clone_val);
                        frame.class = Some(defc);
                        frame.static_class = Some(cid);
                        frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
                        frame.flags.set(FrameFlags::CLONE_INIT, true);
                        self.frames.push(frame);
                        continue;
                    }
                }
                Op::PropGet { name, ic } => {
                    let obj = self.frames[top].stack.pop().expect("PropGet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    // INLINE CACHE (WP-29): the site's last cacheable
                    // resolution — a PUBLIC hook-free backed slot on this
                    // class — reads with zero hashing. A present non-`Undef`
                    // value is the only hit; everything else (absent = unset →
                    // `__get`, `Undef` = typed-uninit fatal, other class, lazy
                    // wrapper) falls through to the paths that own it.
                    if let Zval::Object(o) = &target {
                        if let Some((cid1, slot)) = ic.get() {
                            let b = o.borrow();
                            if b.class_id + 1 == cid1 && b.lazy.is_none() {
                                if let Some(v) = b.props.get_slot(slot) {
                                    if !matches!(v, Zval::Undef) {
                                        let v = v.deref_clone();
                                        drop(b);
                                        self.frames[top].stack.push(v);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // FAST PATH (WP-25): a present, initialized slot on a
                    // non-lazy instance of a hook-free all-public class reads
                    // straight off the table. A miss or `Undef` falls through —
                    // `__get`, the undefined-property warning and the
                    // typed-uninit fatal all live in the general path below.
                    if let Zval::Object(o) = &target {
                        let b = o.borrow();
                        if b.lazy.is_none() {
                            let ci = &self.classes[b.class_id as usize];
                            if ci.all_props_public && !ci.has_prop_hooks {
                                if let Some(v) = b.props.get(&name) {
                                    if !matches!(v, Zval::Undef) {
                                        // IC fill from the fast path too —
                                        // all-public classes NEVER reach the
                                        // general resolve, and without this
                                        // the site's cache stays cold forever
                                        // (every access re-pays slot_of).
                                        if let Some(pi) = ci.prop_info.get(&name[..]) {
                                            if let Some(i) = pi.slot {
                                                ic.fill(b.class_id, i);
                                            }
                                        }
                                        // deref_clone: a slot holding a Ref
                                        // reads as its inner value (same as
                                        // read_property).
                                        let v = v.deref_clone();
                                        drop(b);
                                        self.frames[top].stack.push(v);
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // A read of a lazy object initializes it first (PHP 8.4) —
                    // unless a hook/`__get` serves it; an initialized proxy then
                    // forwards the read to its real instance (transitively).
                    let target = self.lazy_prop_access(target, &name, cur, Some(false), (MagicKind::Get, b"__get"))?;
                    // Storage slot to read (the plain name for a dynamic/non-object
                    // target; a mangled key for an accessible private — set below).
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
                    let mut slot_idx: Option<u32> = None;
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
                        // Single resolution decides visibility AND the storage key
                        // (was check_prop_access + prop_storage_key = two walks).
                        let ocid = o.borrow().class_id as usize;
                        key = match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Slot { key: k, slot } => {
                                slot_idx = slot;
                                // IC fill: only a scope-independent outcome —
                                // PUBLIC, hook-free, backed (see PropIc).
                                if let (Some(i), Some(pi)) = (slot, prop_info(&self.classes, ocid, &name)) {
                                    if pi.visibility == crate::hir::Visibility::Public && pi.hooks.is_none() {
                                        ic.fill(ocid as u32, i);
                                    }
                                }
                                Cow::Borrowed(k)
                            }
                            PropAccess::Dynamic => Cow::Borrowed(&name[..]),
                            PropAccess::Denied { decl, vis } => {
                                return Err(prop_access_error(&self.classes, decl, &name, vis))
                            }
                        };
                        if let Some(err) = self.uninit_typed_read_at(o, &key, slot_idx, &name) {
                            return Err(err);
                        }
                    }
                    let v = read_property_at(&target, &key, slot_idx, &mut self.diags);
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
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
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
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
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
                        key = match resolve_prop_access(&self.classes, o.borrow().class_id as usize, &name, cur) {
                            PropAccess::Slot { key: k, .. } => Cow::Borrowed(k),
                            PropAccess::Dynamic => Cow::Borrowed(&name[..]),
                            PropAccess::Denied { decl, vis } => {
                                return Err(prop_access_error(&self.classes, decl, &name, vis))
                            }
                        };
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
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
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
                Op::PropSet { name, ic } => {
                    let mut value = self.frames[top].stack.pop().expect("PropSet value");
                    let obj = self.frames[top].stack.pop().expect("PropSet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    // A write to a lazy object initializes it first (PHP 8.4) —
                    // unless a set hook/`__set` serves it; a no-op during the
                    // object's own construction (it is no longer lazy then).
                    // Proxy forwarding is transitive (a reset instance re-triggers).
                    let target = if !self.frames[top].flags.get(FrameFlags::INIT_PROPS) {
                        self.lazy_prop_access(target, &name, cur, Some(true), (MagicKind::Set, b"__set"))?
                    } else {
                        target
                    };
                    // A `prop_init` thunk writes defaults directly: no `__set`, no
                    // visibility check (so a subclass can set an inherited private).
                    // The slot is the declared one (unconditional, mangled for a
                    // private) regardless of the running scope.
                    if self.frames[top].flags.get(FrameFlags::INIT_PROPS) {
                        let key = match object_class_id(&target) {
                            Some(ocid) => self.prop_decl_storage_key(ocid, &name),
                            None => Cow::Borrowed(&name[..]),
                        };
                        if let Some(old) = write_property(&target, &key, value.clone())? {
                            self.gc_note(&old);
                        }
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    // INLINE CACHE (WP-29): the WP-25 guards below, but
                    // slot-indexed — zero hashing on a monomorphic site. Only
                    // fills from a `plain_set_props` class, so a class-id
                    // match re-implies every per-class guard; the per-object
                    // ones (lazy, enum, present slot, Ref×typed_refs) are
                    // re-checked here.
                    if let Zval::Object(o) = &target {
                        if let Some((cid1, slot)) = ic.get() {
                            let hit = {
                                let b = o.borrow();
                                b.class_id + 1 == cid1
                                    && b.lazy.is_none()
                                    && !b.info.is_enum_case
                                    && match b.props.get_slot(slot) {
                                        Some(Zval::Ref(_)) => self.typed_refs.is_empty(),
                                        Some(_) => true,
                                        None => false,
                                    }
                            };
                            if hit {
                                if let Some(old) =
                                    write_property_at(&target, &name, Some(slot), value.clone())?
                                {
                                    self.gc_note(&old);
                                }
                                self.frames[top].stack.push(value);
                                continue;
                            }
                        }
                    }
                    // FAST PATH (WP-25): overwrite of a *present* slot on a
                    // non-lazy, non-enum instance of a class whose declared
                    // properties are all plain for writing (public, symmetric,
                    // untyped, non-readonly, hook-free): no visibility walk,
                    // no magic probe, no coercion. A miss falls through
                    // (dynamic-prop creation/deprecation, `__set`); a Ref slot
                    // with live typed refs falls through (typed_ref_assign).
                    if let Zval::Object(o) = &target {
                        let (fast, fcid) = {
                            let b = o.borrow();
                            let ok = b.lazy.is_none()
                                && !b.info.is_enum_case
                                && self.classes[b.class_id as usize].plain_set_props
                                && match b.props.get(&name) {
                                    Some(Zval::Ref(_)) => self.typed_refs.is_empty(),
                                    Some(_) => true,
                                    None => false,
                                };
                            (ok, b.class_id)
                        };
                        if fast {
                            // IC fill from the fast path too (see PropGet):
                            // plain_set_props classes never reach the general
                            // resolve, so without this the cache stays cold.
                            let slot = self.classes[fcid as usize]
                                .prop_info
                                .get(&name[..])
                                .and_then(|pi| pi.slot);
                            if let Some(i) = slot {
                                ic.fill(fcid, i);
                            }
                            if let Some(old) = write_property_at(&target, &name, slot, value.clone())? {
                                self.gc_note(&old);
                            }
                            self.frames[top].stack.push(value);
                            continue;
                        }
                    }
                    // Storage slot to write (plain name for a dynamic/non-object
                    // target; a mangled key for an accessible private — set below).
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
                    let mut slot_idx: Option<u32> = None;
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
                        let ocid = o.borrow().class_id as usize;
                        // Storage slot (mangled for an accessible private); per-instance
                        // state (props, readonly tracking) is keyed by it. A single
                        // resolution decides visibility (`Denied` errors here) and the
                        // slot: the write is either a declared slot or a dynamic
                        // creation — readonly / typed enforcement applies only to the
                        // former (a parent's private reached from a child scope is a
                        // *dynamic* write, untyped and unguarded).
                        let access = resolve_prop_access(&self.classes, ocid, &name, cur);
                        let declared_slot = matches!(access, PropAccess::Slot { .. });
                        key = match access {
                            PropAccess::Slot { key: k, slot } => {
                                slot_idx = slot;
                                // IC fill: only from a plain_set_props class
                                // (public, symmetric, untyped, non-readonly,
                                // hook-free in blocco — see PropIc).
                                if let Some(i) = slot {
                                    if self.classes[ocid].plain_set_props {
                                        ic.fill(ocid as u32, i);
                                    }
                                }
                                Cow::Borrowed(k)
                            }
                            PropAccess::Denied { decl, vis } => {
                                return Err(prop_access_error(&self.classes, decl, &name, vis))
                            }
                            PropAccess::Dynamic => Cow::Borrowed(&name[..]),
                        };
                        // PHP 8.4 asymmetric visibility: a declared slot whose set
                        // visibility excludes this scope cannot be assigned (the
                        // readonly path below keeps its own message shapes).
                        if declared_slot {
                            if let Some(err) = asym_write_error(&self.classes, cur, ocid, &name, "modify") {
                                return Err(err);
                            }
                        }
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
                                    let set_vis = prop_info(&self.classes, ocid, &name)
                                        .and_then(|pi| pi.set_visibility);
                                    if let Some(err) = readonly_write_error(&self.classes, cur, decl, &name, inited, set_vis) {
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
                            let b = o.borrow();
                            let cur_val = match slot_idx {
                                Some(i) => b.props.get_slot(i),
                                None => b.props.get(&key),
                            };
                            let cell = match cur_val {
                                Some(Zval::Ref(c)) => Some(Rc::clone(c)),
                                _ => None,
                            };
                            drop(b);
                            if let Some(cell) = cell {
                                let strict = self.frames[top].module.strict;
                                value = self.typed_ref_assign(&cell, value, strict)?;
                            }
                        }
                    }
                    if let Some(old) = write_property_at(&target, &key, slot_idx, value.clone())? {
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
                    // ONE resolution decides visibility, storage key AND slot
                    // (was check_prop_access + prop_storage_key = two walks,
                    // then two more name-hashes in read/write_property).
                    let mut slot_idx: Option<u32> = None;
                    let key: Cow<[u8]> = match object_class_id(&obj) {
                        Some(ocid) => match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Slot { key: k, slot } => {
                                slot_idx = slot;
                                Cow::Borrowed(k)
                            }
                            PropAccess::Dynamic => Cow::Borrowed(&name[..]),
                            PropAccess::Denied { decl, vis } => {
                                return Err(prop_access_error(&self.classes, decl, &name, vis))
                            }
                        },
                        None => Cow::Borrowed(&name[..]),
                    };
                    if let Some(err) = self.readonly_rmw_error(&obj, &key, &name) {
                        return Err(err);
                    }
                    // PHP 8.4 asymmetric visibility: a compound assignment is a
                    // write — denied from a scope the set visibility excludes.
                    if let Some(ocid) = object_class_id(&obj) {
                        if let Some(err) = asym_write_error(&self.classes, cur, ocid, &name, "modify") {
                            return Err(err);
                        }
                    }
                    if let Zval::Object(o) = &obj.deref_clone() {
                        if let Some(err) = self.uninit_typed_read_at(o, &key, slot_idx, &name) {
                            return Err(err);
                        }
                    }
                    let old = read_property_at(&obj, &key, slot_idx, &mut self.diags);
                    let mut result = self.apply_binop_ovl(*op, &old, &rhs)?;
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
                    if let Some(dropped) = write_property_at(&obj, &key, slot_idx, result.clone())? {
                        self.gc_note(&dropped);
                    }
                    self.frames[top].stack.push(result);
                }
                Op::PropIncDec { name, inc, pre, ic } => {
                    let obj = self.frames[top].stack.pop().expect("PropIncDec object");
                    let cur = self.frames[top].class;
                    let obj_d = obj.deref_clone();
                    // INLINE CACHE (WP-30): the RMW twin of the PropSet hit.
                    // Fills only from a `plain_set_props` class (below), so a
                    // class-id match re-implies every per-class check the slow
                    // path performs (readonly-RMW, asym write, typed-uninit,
                    // both hook probes, write coercion); the per-object state
                    // (lazy, enum case, present non-Undef slot, Ref×typed_refs)
                    // is re-checked here. An absent/Undef slot falls through
                    // (undefined-prop warning, dynamic creation, `__get`).
                    if let Zval::Object(o) = &obj_d {
                        if let Some((cid1, slot)) = ic.get() {
                            let hit = {
                                let b = o.borrow();
                                b.class_id + 1 == cid1
                                    && b.lazy.is_none()
                                    && !b.info.is_enum_case
                                    && match b.props.get_slot(slot) {
                                        None | Some(Zval::Undef) => false,
                                        Some(Zval::Ref(_)) => self.typed_refs.is_empty(),
                                        Some(_) => true,
                                    }
                            };
                            if hit {
                                let old = read_property_at(&obj_d, &name, Some(slot), &mut self.diags);
                                let mut newv = old.clone();
                                if *inc {
                                    ops::increment(&mut newv, &mut self.diags)?;
                                } else {
                                    ops::decrement(&mut newv, &mut self.diags)?;
                                }
                                if let Some(dropped) =
                                    write_property_at(&obj_d, &name, Some(slot), newv.clone())?
                                {
                                    self.gc_note(&dropped);
                                }
                                self.frames[top].stack.push(if *pre { newv } else { old });
                                continue;
                            }
                        }
                    }
                    // `++`/`--` initializes a lazy object like a compound assign.
                    let obj = self.lazy_prop_access(obj_d, &name, cur, None, (MagicKind::Get, b"__get"))?;
                    // ONE resolution (see PropOpSet above).
                    let mut slot_idx: Option<u32> = None;
                    let key: Cow<[u8]> = match object_class_id(&obj) {
                        Some(ocid) => match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Slot { key: k, slot } => {
                                slot_idx = slot;
                                // Fill (WP-30): only a plain_set_props class —
                                // scope-independent by construction (all props
                                // public, symmetric, untyped, non-readonly,
                                // hook-free); the slot proves it is declared.
                                if let (Some(i), Zval::Object(o)) = (slot, &obj) {
                                    let b = o.borrow();
                                    if b.lazy.is_none()
                                        && !b.info.is_enum_case
                                        && self.classes[b.class_id as usize].plain_set_props
                                    {
                                        ic.fill(b.class_id, i);
                                    }
                                }
                                Cow::Borrowed(k)
                            }
                            PropAccess::Dynamic => Cow::Borrowed(&name[..]),
                            PropAccess::Denied { decl, vis } => {
                                return Err(prop_access_error(&self.classes, decl, &name, vis))
                            }
                        },
                        None => Cow::Borrowed(&name[..]),
                    };
                    if let Some(err) = self.readonly_rmw_error(&obj, &key, &name) {
                        return Err(err);
                    }
                    // PHP 8.4 asymmetric visibility: `++`/`--` is a write —
                    // denied from a scope the set visibility excludes.
                    if let Some(ocid) = object_class_id(&obj) {
                        if let Some(err) = asym_write_error(&self.classes, cur, ocid, &name, "modify") {
                            return Err(err);
                        }
                    }
                    if let Zval::Object(o) = &obj.deref_clone() {
                        if let Some(err) = self.uninit_typed_read_at(o, &key, slot_idx, &name) {
                            return Err(err);
                        }
                    }
                    let old = read_property_at(&obj, &key, slot_idx, &mut self.diags);
                    let mut newv = old.clone();
                    if *inc {
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
                                self.frames[top].stack.push(if *pre { newv.clone() } else { old });
                                self.push_hook(func, obj.deref_clone(), oid, &name, Some(newv));
                                continue;
                            }
                        }
                    }
                    if let Some(ocid) = object_class_id(&obj) {
                        newv = self.coerce_typed_prop_write(ocid, &name, newv)?;
                    }
                    if let Some(dropped) = write_property_at(&obj, &key, slot_idx, newv.clone())? {
                        self.gc_note(&dropped);
                    }
                    self.frames[top].stack.push(if *pre { newv } else { old });
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
                                self.frames.last_mut().unwrap().flags.set(FrameFlags::RET_ISSET, true);
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
                        // Resolve like any property access: the accessing
                        // scope's OWN private wins over a subclass's same-name
                        // redeclaration (prop_vis_decl saw only the most-derived
                        // declaration, so `isset($this->data)` inside the parent
                        // read the child's slot and answered false).
                        match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Denied { .. } => false,
                            PropAccess::Slot { key, slot } => prop_isset_at(&target, &key, slot),
                            PropAccess::Dynamic => prop_isset(&target, &name),
                        }
                    } else {
                        prop_isset(&target, &name)
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::PropIssetFetchGate { name } => {
                    // The `??`/`??=` fetch gate: PropIsset semantics, plus the
                    // BP_VAR_IS fallback — no `__isset` but an applicable
                    // `__get` answers true (the follow-up PropGet routes to it).
                    let obj = self.frames[top].stack.pop().expect("PropIssetFetchGate object");
                    let cur = self.frames[top].class;
                    let target = self.lazy_prop_access(obj.deref_clone(), &name, cur, Some(false), (MagicKind::Isset, b"__isset"))?;
                    let set = if let Zval::Object(o) = &target {
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                self.frames.last_mut().unwrap().flags.set(FrameFlags::RET_ISSET, true);
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
                        let declared = match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Denied { .. } => false,
                            PropAccess::Slot { key, slot } => prop_isset_at(&target, &key, slot),
                            PropAccess::Dynamic => prop_isset(&target, &name),
                        };
                        declared
                            || self
                                .magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                                .is_some()
                    } else {
                        prop_isset(&target, &name)
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::PropIsset { name, ic } => {
                    let obj = self.frames[top].stack.pop().expect("PropIsset object");
                    let cur = self.frames[top].class;
                    let recv = obj.deref_clone();
                    // INLINE CACHE (WP-29): mirror of the PropGet IC — a
                    // present non-`Undef` PUBLIC slot on the cached class
                    // answers with zero hashing; anything else falls through.
                    if let Zval::Object(o) = &recv {
                        if let Some((cid1, slot)) = ic.get() {
                            let b = o.borrow();
                            if b.class_id + 1 == cid1 && b.lazy.is_none() {
                                if let Some(v) = b.props.get_slot(slot) {
                                    if !matches!(v, Zval::Undef) {
                                        let set = !matches!(v.deref_clone(), Zval::Null | Zval::Undef);
                                        drop(b);
                                        self.frames[top].stack.push(Zval::Bool(set));
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // FAST PATH (WP-25): mirror of the PropGet fast path — a
                    // present slot on a non-lazy instance of a hook-free
                    // all-public class answers directly. `Undef` still falls
                    // through (a typed-unset slot dispatches `__isset`), as
                    // does a miss.
                    if let Zval::Object(o) = &recv {
                        let b = o.borrow();
                        if b.lazy.is_none() {
                            let ci = &self.classes[b.class_id as usize];
                            if ci.all_props_public && !ci.has_prop_hooks {
                                if let Some(v) = b.props.get(&name) {
                                    if !matches!(v, Zval::Undef) {
                                        // IC fill from the fast path too (see
                                        // PropGet).
                                        if let Some(pi) = ci.prop_info.get(&name[..]) {
                                            if let Some(i) = pi.slot {
                                                ic.fill(b.class_id, i);
                                            }
                                        }
                                        let set = !matches!(v.deref_clone(), Zval::Null | Zval::Undef);
                                        drop(b);
                                        self.frames[top].stack.push(Zval::Bool(set));
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    // `isset()` initializes a lazy object (PHP 8.4,
                    // isset_initializes) — unless a get hook/`__isset` serves it.
                    let target = self.lazy_prop_access(recv, &name, cur, Some(false), (MagicKind::Isset, b"__isset"))?;
                    let set = if let Zval::Object(o) = &target {
                        // `isset($o->hooked)` runs the `get` hook and tests its result
                        // for being non-null (step 50). Hooks precede `__isset`.
                        let (oid, cid) = { let b = o.borrow(); (b.id, b.class_id as usize) };
                        if !self.hook_guarded(oid, &name) {
                            if let Some(func) = self.prop_hook(cid, &name, false) {
                                self.push_hook(func, target.clone(), oid, &name, None);
                                self.frames.last_mut().unwrap().flags.set(FrameFlags::RET_ISSET, true);
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
                        // Resolve like any property access: the accessing
                        // scope's OWN private wins over a subclass's same-name
                        // redeclaration (prop_vis_decl saw only the most-derived
                        // declaration, so `isset($this->data)` inside the parent
                        // read the child's slot and answered false).
                        match resolve_prop_access(&self.classes, ocid, &name, cur) {
                            PropAccess::Denied { .. } => false,
                            PropAccess::Slot { key, slot } => {
                                // IC fill: scope-independent outcomes only
                                // (public, hook-free, backed — see PropIc).
                                if let (Some(i), Some(pi)) = (slot, prop_info(&self.classes, ocid, &name)) {
                                    if pi.visibility == crate::hir::Visibility::Public && pi.hooks.is_none() {
                                        ic.fill(ocid as u32, i);
                                    }
                                }
                                prop_isset_at(&target, &key, slot)
                            }
                            PropAccess::Dynamic => prop_isset(&target, &name),
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
                    let mut key: Cow<[u8]> = Cow::Borrowed(&name[..]);
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
                        // A prelude-declared hook models a native virtual prop
                        // (XSLTProcessor::$maxTemplateDepth): Zend's internal
                        // handler omits the "hooked property" wording there.
                        let hcid = o.borrow().class_id as usize;
                        if let Some(h) = self
                            .prop_hook(hcid, &name, false)
                            .or_else(|| self.prop_hook(hcid, &name, true))
                        {
                            let native = h.file.as_ref() == b"prelude";
                            let ob = o.borrow();
                            let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                            let prop = String::from_utf8_lossy(&name).into_owned();
                            return Err(PhpError::Error(if native {
                                format!("Cannot unset {cls}::${prop}")
                            } else {
                                format!("Cannot unset hooked property {cls}::${prop}")
                            }));
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Unset, b"__unset")
                        {
                            let discard = Rc::new(RefCell::new(Zval::Null));
                            self.push_magic_prop(defc, midx, oid, MagicKind::Unset, target.clone(), &name, None, Some(discard), false);
                            continue;
                        }
                        check_prop_access(&self.classes, cur, o.borrow().class_id as usize, &name)?;
                        // `unset` takes the readonly *write* path (after the
                        // visibility check, so a private/protected one reports the
                        // access error first, matching PHP): an initialised
                        // property (or an uninitialised one from outside its
                        // set-visibility scope) fatals with the write-error
                        // shapes, "unset" for "modify". An uninitialised IN-scope
                        // unset is permitted — the lazy-ghost pattern (Symfony
                        // LazyClosure's ctor unsets `$this->service` so `__get`
                        // serves it later) — and falls through to the typed-unset
                        // marking below. During `__clone`, `unset` returns the
                        // property to the re-assignable uninitialised state (8.3).
                        let ocid = o.borrow().class_id as usize;
                        key = self.prop_storage_key(ocid, &name, cur);
                        // PHP 8.4 asymmetric visibility: `unset` from a scope the
                        // set visibility excludes is denied ("Cannot unset ...");
                        // an explicitly-unset slot dispatched `__unset` above.
                        if let Some(err) = asym_write_error(&self.classes, cur, ocid, &name, "unset") {
                            return Err(err);
                        }
                        if let Some(decl) = prop_readonly_decl(&self.classes, ocid, &name) {
                            if o.borrow().readonly_clone_writable(&key) {
                                o.borrow_mut().clear_readonly_init(&key);
                            } else {
                                let inited = o.borrow().is_readonly_init(&key);
                                let set_vis = prop_info(&self.classes, ocid, &name)
                                    .and_then(|pi| pi.set_visibility);
                                if let Some(err) =
                                    readonly_write_error(&self.classes, cur, decl, &name, inited, set_vis)
                                {
                                    let PhpError::Error(m) = err else { return Err(err) };
                                    return Err(PhpError::Error(
                                        m.replacen("Cannot modify", "Cannot unset", 1),
                                    ));
                                }
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
                        // A declared TYPED property keeps its `Undef` slot on
                        // unset (var_dump/reflection still render it
                        // `uninitialized`, and the lazy/readonly bookkeeping
                        // sees the slot), but is MARKED explicitly-unset:
                        // Zend clears the IS_PROP_UNINIT flag, which is what
                        // lets a later read dispatch `__get` (symfony
                        // Constraint's lazy groups) while a never-initialized
                        // read keeps the before-init fatal.
                        let ob = o.borrow();
                        let typed = ob.info.type_of(&key).is_some()
                            || ob.info.type_of(&name).is_some();
                        drop(ob);
                        if typed {
                            let mut ob = o.borrow_mut();
                            ob.props.set(&key, Zval::Undef);
                            ob.mark_typed_unset(&key);
                            continue;
                        }
                    }
                    prop_unset(&target, &key);
                }
                Op::MethodCall { method, argc, ic } => {
                    let args = self.pop_keys(top, *argc); // source order
                    let recv = self.frames[top].stack.pop().expect("MethodCall receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args, Some(&ic))?;
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
                        self.method_call(top, this, &method, args, None)?;
                    } else {
                        self.dispatch_instance_call_named(top, this, &method, args, named)?;
                    }
                }
                Op::MethodCallDynamic { argc } => {
                    // `$obj->$m(args)`: the method name sits on top, the positional
                    // args beneath it, the receiver at the bottom (step 51).
                    let nameval = self.frames[top].stack.pop().expect("MethodCallDynamic name");
                    let method = convert::to_zstr(&nameval, &mut self.diags).as_bytes().to_vec();
                    let args = self.pop_keys(top, *argc);
                    let recv = self.frames[top].stack.pop().expect("MethodCallDynamic receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args, None)?;
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
                        self.method_call(top, this, &method, args, None)?;
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
                    let pos = self.pop_keys(top, *positional);
                    let recv = self.frames[top].stack.pop().expect("MethodCallNamed receiver");
                    let this = recv.deref_clone();
                    self.dispatch_instance_call_named(top, this, &method, pos, named)?;
                }
                Op::InvokeMethod { class, method_idx, argc } => {
                    let args = self.pop_keys(top, *argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeMethod receiver");
                    let this = recv.deref_clone();
                    let lsb = object_class_id(&this).unwrap_or(*class);
                    let callee = &self.classes[*class].methods[*method_idx as usize].func;
                    let m = self.class_mod(*class);
                    let mut frame = self.pooled_frame(callee, m);
                    bind_params(&mut frame, args);
                    frame.this = Some(this);
                    frame.class = Some(*class);
                    frame.static_class = Some(lsb);
                    self.enter_callee(frame)?;
                }
                Op::InstanceOf { class } => {
                    let v = self.frames[top].stack.pop().expect("InstanceOf operand");
                    let result = match v.deref_clone() {
                        Zval::Object(o) => {
                            is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, *class)
                        }
                        // A generator has no ClassId but is-a Iterator/Traversable
                        // (now real prelude interfaces); nothing else among the
                        // value types satisfies these.
                        Zval::Generator(_) => {
                            let n = &self.classes[*class].name;
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
                Op::StaticCall { target, method, forwarding, argc, ic } => {
                    let mut args = self.pop_keys(top, *argc);
                    let start = self.target_class_id(*target, top)?;
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
                            // Decay a reference pushed by a dynamic call (SEND_VAR_EX);
                            // a deferred place argument reads by value (native callee).
                            if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
                                self.materialize_arg_places(top, &mut args, None)?;
                            }
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
                    self.dispatch_static_call(top, start, &method, *forwarding, args, Vec::new(), Some(&ic))?;
                }
                Op::HookCall { target, prop, set, argc } => {
                    // PHP 8.4 `parent::$prop::get()` / `parent::$prop::set($v)`.
                    let args = self.pop_keys(top, *argc);
                    let recv = self.frames[top].this.clone().ok_or_else(|| {
                        PhpError::Error(format!(
                            "Cannot call ::${}::{} outside object context",
                            String::from_utf8_lossy(&prop),
                            if *set { "set" } else { "get" },
                        ))
                    })?;
                    let oid = object_id(&recv);
                    let start = self.target_class_id(*target, top)?;
                    // A user `get`/`set` hook on the named class runs as a frame.
                    // Extra arguments are ignored — it is an ordinary user function.
                    if let Some(func) = self.prop_hook(start, &prop, *set) {
                        let set_value =
                            if *set { Some(args.into_iter().next().unwrap_or(Zval::Null)) } else { None };
                        if *set {
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
                    let expected = if *set { 1usize } else { 0 };
                    if args.len() != expected {
                        return Err(PhpError::Error(format!(
                            "{}::${}::{}() expects exactly {} argument{}, {} given",
                            String::from_utf8_lossy(&self.classes[start].name),
                            String::from_utf8_lossy(&prop),
                            if *set { "set" } else { "get" },
                            expected,
                            if expected == 1 { "" } else { "s" },
                            args.len(),
                        )));
                    }
                    let ocid = object_class_id(&recv).unwrap_or(start);
                    let key = self.prop_storage_key(ocid, &prop, Some(start));
                    if *set {
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
                    let args = self.pop_keys(top, *argc); // source order
                    let result = self.closure_static_method(&method, args)?;
                    self.frames[top].stack.push(result);
                }
                Op::StaticCallArgs { target, method, forwarding } => {
                    // Spread / named `C::m(...$a)` / `C::m(name: …)`: args from a
                    // runtime array — integer keys positional, string keys named.
                    let argsval = self.frames[top].stack.pop().expect("StaticCallArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let start = self.target_class_id(*target, top)?;
                    self.dispatch_static_call(top, start, &method, *forwarding, args, named, None)?;
                }
                Op::StaticCallDynamic { method, argc } => {
                    // `$cls::m()` (PAR): args are on top, the class reference beneath.
                    let args = self.pop_keys(top, *argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamic class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    // A dynamic class is non-forwarding, like a named class.
                    self.dispatch_static_call(top, start, &method, false, args, Vec::new(), None)?;
                }
                Op::StaticCallDynamicArgs { method } => {
                    // Spread `$cls::m(...$a)` (Session A): args array on top, the
                    // class reference beneath; string keys bind as named (PHP 8.1).
                    let argsval = self.frames[top].stack.pop().expect("StaticCallDynamicArgs array");
                    let (args, named) = split_args_from_array_value(argsval);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicArgs class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    self.dispatch_static_call(top, start, &method, false, args, named, None)?;
                }
                Op::StaticCallDynamicMethod { argc } => {
                    // `$cls::$m()`: method name on top, then args, then the class ref.
                    let mval = self.frames[top].stack.pop().expect("StaticCallDynamicMethod name");
                    let args = self.pop_keys(top, *argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicMethod class");
                    // PHP validates the class before the method name: `$a::$b()` with
                    // both invalid reports the class error first.
                    let start = self.resolve_dynamic_class(&classval)?;
                    let method = dyn_method_name(&mval)?;
                    // A dynamic class is non-forwarding, like a named static call.
                    self.dispatch_static_call(top, start, &method, false, args, Vec::new(), None)?;
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
                    self.dispatch_static_call(top, start, &method, false, args, named, None)?;
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
                    let start = self.target_class_id(*target, top)?;
                    self.dispatch_static_call(top, start, &method, *forwarding, args, named, None)?;
                }
                Op::StaticCallTargetDynamicMethod { target, forwarding, argc } => {
                    // `self::$m()` / `Class::$m()`: method name on top, then args; the
                    // class is a compile-time target, forwarding preserved.
                    let mval = self.frames[top]
                        .stack
                        .pop()
                        .expect("StaticCallTargetDynamicMethod name");
                    let method = dyn_method_name(&mval)?;
                    let args = self.pop_keys(top, *argc);
                    let start = self.target_class_id(*target, top)?;
                    self.dispatch_static_call(top, start, &method, *forwarding, args, Vec::new(), None)?;
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
                    if self.classes[*class].consts.get(*idx as usize).is_none() {
                        return Err(PhpError::Error(format!(
                            "VM: ClassConst out of range: {}::consts[{}] (len {}) in {} ({}) line {}",
                            String::from_utf8_lossy(&self.classes[*class].name),
                            idx,
                            self.classes[*class].consts.len(),
                            String::from_utf8_lossy(&self.frames[top].func.name),
                            String::from_utf8_lossy(&self.frames[top].func.file),
                            self.cur_line(top)
                        )));
                    }
                    let thunk = &self.classes[*class].consts[*idx as usize].func;
                    let mut frame = Frame::new(thunk, self.class_mod(*class));
                    frame.class = Some(*class);
                    frame.static_class = Some(*class);
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
                Op::ClassNameScope { parent } => {
                    // `self::class` / `parent::class` in a closure body: the scope
                    // class follows any `Closure::bind` rebinding.
                    let target =
                        if *parent { ClassTarget::ParentScope } else { ClassTarget::SelfScope };
                    let cid = self.target_class_id(target, top)?;
                    let name = self.classes[cid].name.to_vec();
                    self.frames[top].stack.push(Zval::Str(PhpStr::new(name)));
                }
                Op::EnumCase { class, case } => {
                    let obj = self.enum_case(*class, *case);
                    self.frames[top].stack.push(Zval::Object(obj));
                }
                Op::InvokeCtor { argc } => {
                    let mut args = self.pop_keys(top, *argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtor receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtor on a non-object");
                    match resolve_method_runtime(&self.classes, cid, b"__construct") {
                        Some((defc, midx)) => {
                            // Deferred place arguments (SEND_VAR_EX) resolve against
                            // the now-known constructor's by-ref mask.
                            if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
                                let cls = self.classes[defc];
                                let ctor = &cls.methods[midx].func;
                                self.materialize_arg_places(top, &mut args, Some(ctor))?;
                                let line = self.cur_line(top);
                                self.flush_diags(line)?;
                            }
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
                            frame.flags.set(FrameFlags::INIT_PROPS, true); // privileged default writes
                            self.frames.push(frame);
                        }
                        // No non-constant defaults: nothing to do, balance the stack.
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::StaticPropGet { target, name } => {
                    let cell = match self.ensure_static(*target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue, // init thunk scheduled; re-run after it
                    };
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSet { target, name } => {
                    let cell = match self.ensure_static(*target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let value = self.frames[top].stack.pop().expect("StaticPropSet value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropRef { target, name } => {
                    // `$x = &Class::$sp`: the property's storage cell itself,
                    // wrapped as a reference value — the alias is live both ways.
                    let cell = match self.ensure_static(*target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::StaticPropOpSet { target, name, op } => {
                    let cell = match self.ensure_static(*target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let rhs = self.frames[top].stack.pop().expect("StaticPropOpSet rhs");
                    let old = cell.borrow().deref_clone();
                    let result = self.apply_binop_ovl(*op, &old, &rhs)?;
                    *cell.borrow_mut() = result.clone();
                    self.frames[top].stack.push(result);
                }
                Op::StaticPropIncDec { target, name, inc, pre } => {
                    let cell = match self.ensure_static(*target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let old = cell.borrow().deref_clone();
                    let mut newv = old.clone();
                    if *inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if *pre { newv } else { old });
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
                    let result = self.apply_binop_ovl(*op, &old, &rhs)?;
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
                    if *inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if *pre { newv } else { old });
                }
                Op::FieldAssign { base, steps } => {
                    let value = self.frames[top].stack.pop().expect("FieldAssign value");
                    let keys = self.pop_field_keys(top, &steps);
                    // A path starting at a `&get` hooked property writes through
                    // the reference the hook returns (one hook run, no set hook).
                    if let Some(root) = self.byref_hook_root(*base, top, &steps)? {
                        self.field_set_in_root(root, top, &steps[1..], keys, value.clone(), false, false)?;
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    self.reject_indirect_hook(*base, top, &steps)?;
                    // A lazy base initializes/forwards first; the walk then roots
                    // at the realized object (PHP 8.4).
                    if let Some(root) = self.field_lazy_root(*base, top, &steps, &keys, true)? {
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, value.clone(), false, false)?;
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    self.field_set(*base, top, &steps, keys, value.clone())?;
                    self.frames[top].stack.push(value);
                }
                Op::FieldAssignOp { base, steps, op } => {
                    let rhs = self.frames[top].stack.pop().expect("FieldAssignOp rhs");
                    let keys = self.pop_field_keys(top, &steps);
                    if let Some(root) = self.byref_hook_root(*base, top, &steps)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&Zval::Ref(Rc::clone(&root)), &steps[1..], &mut keys.clone().into_iter(), fs)
                                .unwrap_or(Zval::Null)
                        };
                        let result = self.apply_binop_ovl(*op, &old, &rhs)?;
                        self.field_set_in_root(root, top, &steps[1..], keys, result.clone(), false, true)?;
                        self.frames[top].stack.push(result);
                        continue;
                    }
                    self.reject_indirect_hook(*base, top, &steps)?;
                    if let Some(root) = self.field_lazy_root(*base, top, &steps, &keys, true)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&root, &steps, &mut keys.clone().into_iter(), fs).unwrap_or(Zval::Null)
                        };
                        let result = self.apply_binop_ovl(*op, &old, &rhs)?;
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, result.clone(), false, true)?;
                        self.frames[top].stack.push(result);
                        continue;
                    }
                    let old = self.field_value(*base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let result = self.apply_binop_ovl(*op, &old, &rhs)?;
                    self.field_set_op(*base, top, &steps, keys, result.clone())?;
                    self.frames[top].stack.push(result);
                }
                Op::FieldIncDec { base, steps, inc, pre } => {
                    let keys = self.pop_field_keys(top, &steps);
                    if let Some(root) = self.byref_hook_root(*base, top, &steps)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&Zval::Ref(Rc::clone(&root)), &steps[1..], &mut keys.clone().into_iter(), fs)
                                .unwrap_or(Zval::Null)
                        };
                        let mut newv = old.clone();
                        if *inc {
                            ops::increment(&mut newv, &mut self.diags)?;
                        } else {
                            ops::decrement(&mut newv, &mut self.diags)?;
                        }
                        self.field_set_in_root(root, top, &steps[1..], keys, newv.clone(), false, true)?;
                        self.frames[top].stack.push(if *pre { newv } else { old });
                        continue;
                    }
                    self.reject_indirect_hook(*base, top, &steps)?;
                    if let Some(root) = self.field_lazy_root(*base, top, &steps, &keys, true)? {
                        let old = {
                            let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                            field_get(&root, &steps, &mut keys.clone().into_iter(), fs).unwrap_or(Zval::Null)
                        };
                        let mut newv = old.clone();
                        if *inc {
                            ops::increment(&mut newv, &mut self.diags)?;
                        } else {
                            ops::decrement(&mut newv, &mut self.diags)?;
                        }
                        self.field_set_in_root(Rc::new(RefCell::new(root)), top, &steps, keys, newv.clone(), false, true)?;
                        self.frames[top].stack.push(if *pre { newv } else { old });
                        continue;
                    }
                    let old = self.field_value(*base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let mut newv = old.clone();
                    if *inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    self.field_set_op(*base, top, &steps, keys, newv.clone())?;
                    self.frames[top].stack.push(if *pre { newv } else { old });
                }
                Op::FieldIsset { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    // Magic protocol at ANY property step along the path
                    // (`isset($o->magic['k'])`, gh18038 / bug40833; and a magic
                    // leaf one hop in: `isset($block->block_type->uses_context)`,
                    // WP_Block_Type) — see field_magic_probe.
                    if let Some(set) = self.field_magic_probe(*base, top, &steps, &keys, false)? {
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    // A final Index on an ArrayAccess object is the protocol:
                    // `isset($this->coll[0])` = offsetExists (no offsetGet),
                    // mirroring Op::IssetPath's single-step arm.
                    if let Some((recv, key)) = self.field_aa_leaf(*base, top, &steps, &keys) {
                        let r = self.call_method_sync(recv, b"offsetExists", vec![key])?;
                        let set = convert::is_true_silent(&r.deref_clone());
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    // Nested Index run on an ArrayAccess property
                    // (`isset($this->data['a']['b'])`): BP_VAR_IS walk.
                    if let Some(res) = self.field_aa_walk(*base, top, &steps, &keys)? {
                        let set = match res {
                            super::DimIsLeaf::Missing => false,
                            super::DimIsLeaf::Aa(recv, key) => {
                                let r =
                                    self.call_method_sync(recv, b"offsetExists", vec![key])?;
                                convert::is_true_silent(&r.deref_clone())
                            }
                            super::DimIsLeaf::Raw(v) => {
                                matches!(v, Some(v) if !matches!(v, Zval::Null | Zval::Undef))
                            }
                        };
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    // A lazy base initializes and the walk roots at the realized
                    // object (isset through a wrapper reads the instance).
                    if let Some(root) = self.field_lazy_root(*base, top, &steps, &keys, false)? {
                        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                        let set = matches!(
                            field_get(&root, &steps, &mut keys.into_iter(), fs),
                            Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                        );
                        self.frames[top].stack.push(Zval::Bool(set));
                        continue;
                    }
                    let set = matches!(
                        self.field_value(*base, top, &steps, keys),
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                    );
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::FieldEmpty { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    // Magic protocol at any property step (the empty() twin of
                    // FieldIsset's probe): `__isset` gates, `__get` supplies the
                    // value whose truthiness decides.
                    if let Some(empty) = self.field_magic_probe(*base, top, &steps, &keys, true)? {
                        self.frames[top].stack.push(Zval::Bool(empty));
                        continue;
                    }
                    // ArrayAccess leaf: !offsetExists short-circuits to empty,
                    // else !truthy(offsetGet) — mirrors Op::EmptyPath.
                    if let Some((recv, key)) = self.field_aa_leaf(*base, top, &steps, &keys) {
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
                    // Nested Index run on an ArrayAccess property: same
                    // BP_VAR_IS walk as Op::FieldIsset above.
                    if let Some(res) = self.field_aa_walk(*base, top, &steps, &keys)? {
                        let empty = match res {
                            super::DimIsLeaf::Missing => true,
                            super::DimIsLeaf::Aa(recv, key) => {
                                let exists = self.call_method_sync(
                                    recv.clone(),
                                    b"offsetExists",
                                    vec![key.clone()],
                                )?;
                                if convert::is_true_silent(&exists.deref_clone()) {
                                    let v =
                                        self.call_method_sync(recv, b"offsetGet", vec![key])?;
                                    !convert::is_true_silent(&v.deref_clone())
                                } else {
                                    true
                                }
                            }
                            super::DimIsLeaf::Raw(v) => match v {
                                Some(v) => !convert::is_true_silent(&v),
                                None => true,
                            },
                        };
                        self.frames[top].stack.push(Zval::Bool(empty));
                        continue;
                    }
                    // empty == !isset || !truthy(value): an unreachable/null leaf is
                    // empty; otherwise test the value's boolean.
                    let empty = match self.field_value(*base, top, &steps, keys) {
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
                        // The magic leaf may sit at the END of a longer path
                        // (`unset($this->list_table->undeclared)`, WP-18): read
                        // the prefix like Zend (its `__get`s included), then
                        // dispatch `__unset` on the leaf name.
                        let Some((last, prefix)) = steps.split_last() else {
                            break 'magic_unset false;
                        };
                        let prefix_keys: usize = prefix
                            .iter()
                            .filter(|s| matches!(s, FieldStep::PropDyn | FieldStep::Index))
                            .count();
                        let name: Vec<u8> = match last {
                            FieldStep::Prop(n) => n.to_vec(),
                            FieldStep::PropDyn => {
                                let Some(k) = keys.get(prefix_keys) else {
                                    break 'magic_unset false;
                                };
                                convert::to_zstr_cast(k, &mut self.diags).as_bytes().to_vec()
                            }
                            _ => break 'magic_unset false,
                        };
                        let base_val = if prefix.is_empty() {
                            match base {
                                FieldBase::Local(s) => {
                                    self.frames[top].slots.get(*s as usize).map(|v| v.deref_clone())
                                }
                                FieldBase::Global(s) => {
                                    self.frames[0].slots.get(*s as usize).map(|v| v.deref_clone())
                                }
                                FieldBase::Superglobal(i) => {
                                    self.superglobals.get(*i as usize).map(|v| v.deref_clone())
                                }
                                FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
                            }
                        } else {
                            self.field_value(*base, top, prefix, keys[..prefix_keys].to_vec())
                        };
                        let Some(mut v) = base_val else {
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
                    if let Some((recv, key)) = self.field_aa_leaf(*base, top, &steps, &keys) {
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
                            FieldBase::Local(s) => self.frames[top].slots.get(*s as usize).map(|v| v.deref_clone()),
                            FieldBase::Global(s) => self.frames[0].slots.get(*s as usize).map(|v| v.deref_clone()),
                            FieldBase::Superglobal(i) => self.superglobals.get(*i as usize).map(|v| v.deref_clone()),
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
                    if let Some(mut root) = self.field_lazy_root(*base, top, &steps, &keys, false)? {
                        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                        field_unset(&mut root, &steps, &mut keys.into_iter(), fs)?;
                        continue;
                    }
                    self.field_remove(*base, top, &steps, keys)?;
                }
                Op::Fatal(i) => {
                    let msg = match &self.frames[top].func.consts[*i as usize] {
                        crate::bytecode::Const::Str(b) => String::from_utf8_lossy(b.as_bytes()).into_owned(),
                        _ => "VM: unsupported construct".to_string(),
                    };
                    return Err(PhpError::Error(msg));
                }
                Op::EmitNotice(i) => {
                    if let crate::bytecode::Const::Str(b) = &self.frames[top].func.consts[*i as usize] {
                        let msg = String::from_utf8_lossy(b.as_bytes()).into_owned();
                        self.diags.push(Diag::Notice(msg));
                        // Flush at the emitting op's line — deferring to the next
                        // flush point would stamp a later statement's line on it.
                        let line = self.cur_line(top);
                        self.flush_diags(line)?;
                    }
                }
                Op::Exit { has_arg } => {
                    let code = if *has_arg {
                        let v = self.frames[top].stack.pop().expect("Exit status");
                        self.exit_status(v, top)?
                    } else {
                        0
                    };
                    return Err(PhpError::Exit(code));
                }
                Op::SuppressBegin => {
                    // Deliver everything queued BEFORE the `@` with normal
                    // semantics first, so the mark cleanly separates the
                    // suppressed region from the statement's earlier diagnostics.
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    self.suppress_marks.push(self.diags.len());
                    // Zend's BEGIN_SILENCE: save EG(error_reporting) and mask it
                    // down to the fatal-only bits (never silence fatals). The
                    // masked value is what error_reporting() reads inside the
                    // region — PHPUnit's handler declines diagnostics on it.
                    self.silence_saved.push(self.error_level);
                    if self.error_level & !4437 != 0 {
                        self.error_level &= 4437;
                    }
                    self.suppress_depth += 1;
                }
                Op::SuppressEnd => {
                    // Deliver the region's queued diagnostics while the level is
                    // still masked: Zend calls the user error handler even under
                    // `@`; only the default render is swallowed (gated by the
                    // masked error_reporting). The default-handler path still
                    // records error_get_last (monolog's StreamHandler appends
                    // error_get_last()['message'] after an @fopen).
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    // Zend's END_SILENCE: restore only if the current level is
                    // still fatal-only and the saved one was not — an
                    // error_reporting($x) inside the region survives it
                    // (bug27731).
                    if let Some(saved) = self.silence_saved.pop() {
                        if self.error_level & !4437 == 0 && saved & !4437 != 0 {
                            self.error_level = saved;
                        }
                    }
                    self.suppress_depth = self.suppress_depth.saturating_sub(1);
                    if let Some(saved) = self.suppress_marks.pop() {
                        self.diags.truncate(saved);
                        self.diags_rendered = self.diags_rendered.min(saved);
                    }
                }
                Op::MatchError(slot) => {
                    let subj = read_slot(&self.frames[top].slots[*slot as usize]);
                    return Err(PhpError::Error(format!(
                        "Unhandled match case {}",
                        match_case_repr(&subj)
                    )));
                }
                Op::Sweep { main } => {
                    // A destructor body's statement sweeps no-op — see
                    // Frame::in_destructor (handle-id release order).
                    if !self.frames[top].flags.get(FrameFlags::IN_DESTRUCTOR) {
                        self.gc_sweep(top, ip, *main)?;
                    }
                }
                Op::Nop => {}
            }
        }
    }

    /// Navigate a place (base + `steps`; `keys` already popped, source order) to
    /// its leaf, promote it to a shared cell, and return that cell — the body of
    /// [`Op::MakeRef`], shared with the dynamic-dispatch binder's by-reference
    /// materialization of a deferred [`Zval::ArgPlace`] (SEND_VAR_EX).
    pub(super) fn make_ref_cell(
        &mut self,
        top: usize,
        base: FieldBase,
        steps: &[FieldStep],
        keys: Vec<Zval>,
    ) -> Result<Rc<RefCell<Zval>>, PhpError> {
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
                        field_cell(&mut root_val, &steps[1..], &mut keys.into_iter(), fs)?
                    };
                    return Ok(cell);
                }
            }
            // A property whose set visibility excludes this scope hands
            // out a reference to a *copy* (PHP 8.4 asymmetric visibility).
            if let Some(cell) = self.asym_set_ref_copy(base, top, &steps)? {
                return Ok(cell);
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
                return Ok(cell);
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
                    field_cell(&mut root, &steps, &mut keys.into_iter(), fs)?
                } else {
                    let base_cell = field_base_mut(&mut self.frames, &mut self.superglobals, top, base)?;
                    if steps.is_empty() {
                        make_cell(base_cell)
                    } else {
                        field_cell(base_cell, &steps, &mut keys.into_iter(), fs)?
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
            Ok(cell)
    }
}
