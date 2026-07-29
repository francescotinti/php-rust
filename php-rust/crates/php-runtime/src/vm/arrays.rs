//! VM arrays logic, extracted from vm/mod.rs (no semantic change).
use super::*;

/// Silently follow `keys` from `cell` without auto-vivifying anything, returning
/// the leaf value if the whole path exists (an unset variable or a missing key
/// at any level yields `None`). Backs `isset` / `empty`. A reference is followed;
/// a string base supports a final byte-offset step.
pub(super) fn silent_get_path(cell: &Zval, keys: &[Zval]) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
        // Shared read-walk (Matsakis WP-71: nested shared borrows coexist;
        // no mut guard ever lives on this path). BORROW-OK: shared read.
        return silent_get_path(&rc.borrow(), keys);
    }
    match keys.split_first() {
        None => match cell {
            Zval::Undef => None,
            other => Some(other.clone()),
        },
        Some((k, rest)) => match cell {
            Zval::Array(a) => {
                let key = coerce_key_silent(k)?;
                a.get(&key).and_then(|child| silent_get_path(child, rest))
            }
            Zval::Str(s) if rest.is_empty() => {
                string_offset(s, k).map(|byte| Zval::Str(PhpStr::new(vec![byte])))
            }
            _ => None,
        },
    }
}

/// Outcome of [`silent_walk`]: the walk resolves the whole path by borrow
/// (`Missing` / `Verdict`) or stops at an OBJECT with keys left to consume
/// (`Object(handle, key_index)`) — the caller decides between the
/// ArrayAccess protocol and `Missing`. No user code runs inside the walk
/// (objects bail immediately), so holding `Ref` borrow guards across
/// levels is safe.
pub(super) enum SilentOut {
    Missing,
    Verdict(bool),
    Object(Zval, usize),
}

/// The zero-clone twin of [`silent_get_path`] for `isset` / `empty`: follow
/// `keys` from `cell` entirely by reference and fold the leaf into an
/// [`IsMode`] verdict — no intermediate or leaf `Zval` is cloned (an object
/// bails out with an `Rc` handle bump only). `depth` is the absolute index
/// of the next key so resumed walks report `Object` positions key-indexed.
pub(super) fn silent_walk(cell: &Zval, keys: &[Zval], depth: usize, mode: IsMode) -> SilentOut {
    if let Zval::Ref(rc) = cell {
        // BORROW-OK: shared read-walk (see silent_get_path).
        return silent_walk(&rc.borrow(), keys, depth, mode);
    }
    match keys.split_first() {
        None => match cell {
            Zval::Undef => SilentOut::Missing,
            v => SilentOut::Verdict(match mode {
                IsMode::Exists => !matches!(v, Zval::Null),
                IsMode::Truthy => convert::is_true_silent(v),
            }),
        },
        Some((k, rest)) => match cell {
            Zval::Object(_) => SilentOut::Object(cell.clone(), depth),
            Zval::Array(a) => match coerce_key_silent(k).and_then(|key| a.get(&key)) {
                Some(child) => silent_walk(child, rest, depth + 1, mode),
                None => SilentOut::Missing,
            },
            // A string offset can sit at ANY step (`isset($s[0][0])` chains
            // through one-byte strings — the per-key walk it replaces saw a
            // single-key slice at every level, so `silent_get_path`'s
            // final-step restriction never applied here).
            Zval::Str(s) => match string_offset(s, k) {
                Some(byte) if rest.is_empty() => SilentOut::Verdict(match mode {
                    IsMode::Exists => true,
                    IsMode::Truthy => {
                        convert::is_true_silent(&Zval::Str(PhpStr::new(vec![byte])))
                    }
                }),
                Some(byte) => {
                    silent_walk(&Zval::Str(PhpStr::new(vec![byte])), rest, depth + 1, mode)
                }
                None => SilentOut::Missing,
            },
            _ => SilentOut::Missing,
        },
    }
}

/// The in-bounds byte at a string offset (negatives count from the end), or
/// `None` if out of range — the existence test behind `isset($s[i])`.
pub(super) fn string_offset(s: &PhpStr, key: &Zval) -> Option<u8> {
    let kd = key.deref_clone();
    if matches!(kd, Zval::Array(_) | Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_)) {
        return None;
    }
    // A STRING key is a valid offset only when it parses as an integer ('01'
    // and '-2' do, 'url' and '1.5' do not): `isset($s['url'])` is false and
    // `empty($s['url'])` true — WP's style engine branches on exactly that to
    // tell an array background from a plain URL string (WP-18, themeJson).
    let i = match &kd {
        Zval::Str(k) => std::str::from_utf8(k.as_bytes())
            .ok()
            .and_then(|t| t.trim().parse::<i64>().ok())?,
        _ => convert::to_long_cast(&kd, &mut Diags::new()),
    };
    let len = s.len() as i64;
    let idx = if i < 0 { len + i } else { i };
    if idx < 0 || idx >= len {
        None
    } else {
        Some(s.as_bytes()[idx as usize])
    }
}

/// Remove the leaf of `keys` from `cell` (or, when `keys` is empty, unset the
/// variable itself by resetting it to `Undef`). A missing intermediate level is
/// a silent no-op; copy-on-write applies to each array touched.
pub(super) fn unset_into(cell: &mut Zval, keys: &[Zval], aa: &mut Option<UnsetAa>) {
    let mut walk = unset_into_walk(cell, keys, 0, aa);
    loop {
        walk = match walk {
            UnsetIntoWalk::Done => return,
            UnsetIntoWalk::IntoRef(rc, i) => match rc.try_borrow_mut() {
                Ok(mut g) => unset_into_walk(&mut g, keys, i, aa),
                Err(_) => {
                    // Every walker guard is already dropped (boundary
                    // discipline) — only a caller-held guard can trip this.
                    // Skip route, counted (M-70.2).
                    cell_skip_note();
                    UnsetIntoWalk::Done
                }
            },
        };
    }
}

/// M-71.1 (WP-71): boundary token of the plain unset-dim walk — same
/// discipline as [`WriteWalk`] (`$a[0] = &$a; unset($a[0][1])` closed the
/// cycle back onto the walk's own live guard and panicked).
enum UnsetIntoWalk {
    Done,
    IntoRef(Rc<RefCell<Zval>>, usize),
}

/// Walk `keys[i..]` from `cell` up to the next `Ref` boundary (see
/// [`UnsetIntoWalk`]); plain containers are crossed in place.
fn unset_into_walk(cell: &mut Zval, keys: &[Zval], i: usize, aa: &mut Option<UnsetAa>) -> UnsetIntoWalk {
    let Some(k) = keys.get(i) else {
        *cell = Zval::Undef;
        return UnsetIntoWalk::Done;
    };
    if let Zval::Ref(rc) = cell {
        return UnsetIntoWalk::IntoRef(Rc::clone(rc), i);
    }
    if let Zval::Array(rc) = cell {
        if let Some(key) = coerce_key_silent(k) {
            let arr = Rc::make_mut(rc);
            if i + 1 == keys.len() {
                arr.remove(&key);
            } else if let Some(child) = arr.get_mut(&key) {
                return unset_into_walk(child, keys, i + 1, aa);
            }
        }
    } else if matches!(cell, Zval::Object(_)) {
        // M-72.1: a dim step on an object defers to the VM drain (the
        // leaf/one-key AA forms are dispatched at the op level; what
        // reaches the walker is the nested form, `unset($c['a']['b'])`).
        if i + 1 == keys.len() {
            *aa = Some(UnsetAa::Unset { obj: cell.clone(), key: k.clone() });
        } else {
            *aa = Some(UnsetAa::Descend {
                obj: cell.clone(),
                key: k.clone(),
                rest: keys[i + 1..].iter().map(|_| FieldStep::Index).collect(),
                keys: keys[i + 1..].to_vec(),
            });
        }
    }
    UnsetIntoWalk::Done
}

/// Write `value` through a mixed field path (OOP-2c), the VM analogue of the
/// tree-walker's `write_into`: a reference is written through; an object property
/// is navigated *in place* (no copy-on-write, shared `Rc<RefCell>`); an array
/// element auto-vivifies and copy-on-writes. `Index` steps consume `keys` in
/// source order.
/// The property name of a [`FieldStep::PropDyn`] step: the next stack-sourced key
/// coerced to a string (step 51).
pub(super) fn prop_dyn_name(keys: &mut std::vec::IntoIter<Zval>, diags: &mut Diags) -> Box<[u8]> {
    let key = keys.next().expect("field prop-dyn name");
    convert::to_zstr(&key, diags).as_bytes().into()
}

/// A leaf dim-write that landed on an OBJECT: the caller (`Vm::field_set`) must
/// dispatch it through the ArrayAccess protocol (`offsetSet`) — a free function
/// cannot make the method call. `key: None` is the append form (`$o->c[] = v`
/// → `offsetSet(null, v)`).
pub(super) struct AaWrite {
    pub obj: Zval,
    pub key: Option<Zval>,
    pub value: Zval,
}

/// A dim-write step that landed on an OBJECT *mid-path*
/// (`$this->coll[0]->foo = 1`): the caller must `offsetGet(key)` and resume the
/// drill on the result (an object resumes through its handle; anything else is
/// PHP's "indirect modification has no effect" notice).
pub(super) struct AaDescend {
    pub obj: Zval,
    pub key: Zval,
    pub rest: Vec<FieldStep>,
    pub keys: Vec<Zval>,
    pub value: Zval,
}

/// A leaf *property* write (`$this->e->foo = v`) that landed on an OBJECT whose
/// `foo` is not a declared, accessible slot: the caller must run the full
/// `Op::PropSet` object-write semantics — dispatch `__set` (guarded against
/// re-entrant recursion) or, absent a magic setter, materialise a dynamic
/// property / enforce visibility. A free function cannot make the method call
/// nor consult the runtime recursion guard, so it defers here (bug40833:
/// `$this->e->foo = v` must invoke `e->__set('foo', v)`, not materialise `foo`).
pub(super) struct AaMagicSet {
    pub obj: Zval,
    pub name: Vec<u8>,
    pub value: Zval,
}

/// An intermediate *property* step (`$o->p->…` / `$o->p[…]…`) whose raw slot is
/// absent or inaccessible: Zend reads the step through the overloaded-property
/// protocol (`__get`, the lazy-holder pattern) and continues the write on its
/// result — or, with no magic getter, applies PHP's no-autoviv rules (dynamic
/// property deprecation, "Attempt to assign property on null", visibility
/// errors). All of those need the VM, so the walker defers the remaining path.
pub(super) struct AaMagicDescend {
    pub obj: Zval,
    pub name: Vec<u8>,
    pub rest: Vec<FieldStep>,
    pub keys: Vec<Zval>,
    pub value: Zval,
}

/// The pending ArrayAccess / magic dispatch a field-write walk produced (at
/// most one — a walk defers exactly one leaf that needs a method call).
pub(super) enum AaOp {
    Write(AaWrite),
    Descend(AaDescend),
    MagicSet(AaMagicSet),
    MagicDescend(AaMagicDescend),
}

pub(super) fn field_write(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
    value: Zval,
    diags: &mut Diags,
    dropped: &mut Vec<Zval>,
    aa: &mut Option<AaOp>,
    rebind: bool,
    rw: bool,
) -> Result<(), PhpError> {
    let mut walk = field_write_walk(target, steps, 0, keys, fs, value, diags, dropped, aa, rebind, rw)?;
    loop {
        walk = match walk {
            WriteWalk::Done => return Ok(()),
            WriteWalk::IntoRef(rc, i, value) => match rc.try_borrow_mut() {
                Ok(mut g) => field_write_walk(&mut g, steps, i, keys, fs, value, diags, dropped, aa, rebind, rw)?,
                Err(_) => {
                    // Every walker guard is already dropped (boundary
                    // discipline) — only a caller-held guard can trip this:
                    // the cell is mid-write on this very statement. Skip
                    // route, counted (M-70.2).
                    cell_skip_note();
                    WriteWalk::Done
                }
            },
            WriteWalk::IntoObj(o, i, value) => {
                field_write_prop_step(&o, steps, i, keys, fs, value, diags, dropped, aa, rebind, rw)?
            }
        };
    }
}

/// M-71.1 (WP-71): boundary token of the property/dim WRITE walk — the same
/// discipline as `CellWalk`/`PathWalk` (H-70.1): the walk stops at every cell
/// boundary (`Zval::Ref`, `Zval::Object`), hands the cloned handle and the
/// not-yet-consumed leaf value back to the [`field_write`] driver, and the
/// current guard drops BEFORE the next borrow — a reference cycle revisiting
/// a cell (`$o->self = $o; $o->self->self->y = 2`) never finds its own guard
/// alive.
enum WriteWalk {
    Done,
    /// The walk crossed into a `Zval::Ref` cell; `steps[i..]` remain.
    IntoRef(Rc<RefCell<Zval>>, usize, Zval),
    /// The next step (`steps[i]`, a Prop/PropDyn) writes on this handle.
    IntoObj(Rc<RefCell<Object>>, usize, Zval),
}

/// Walk `steps[i..]` from `target` up to the next cell boundary (see
/// [`WriteWalk`]). Plain containers (arrays nested by value) are crossed in
/// place — they hold no `RefCell`, so no guard outlives the return.
fn field_write_walk(
    target: &mut Zval,
    steps: &[FieldStep],
    i: usize,
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
    value: Zval,
    diags: &mut Diags,
    dropped: &mut Vec<Zval>,
    aa: &mut Option<AaOp>,
    rebind: bool,
    rw: bool,
) -> Result<WriteWalk, PhpError> {
    if let Zval::Ref(cell) = target {
        // A reference-BIND (`$arr['k'] =& $x`) REPLACES a leaf slot that
        // already holds a reference — writing through it instead would nest
        // the new ref inside the old cell (ORM's ArrayHydrator resultPointers
        // built a self-referential cell that way: infinite deref).
        if rebind && steps.len() == i {
            dropped.push(std::mem::replace(target, value));
            return Ok(WriteWalk::Done);
        }
        return Ok(WriteWalk::IntoRef(Rc::clone(cell), i, value));
    }
    let Some(first) = steps.get(i) else {
        // Leaf overwrite: hand the displaced value back for GC noting.
        dropped.push(std::mem::replace(target, value));
        return Ok(WriteWalk::Done);
    };
    let rest = &steps[i + 1..];
    match first {
        FieldStep::Prop(_) | FieldStep::PropDyn => match target {
            // Property steps run on the driver side (one scoped borrow per
            // step): hand the handle over, the current guard drops first.
            Zval::Object(o) => Ok(WriteWalk::IntoObj(Rc::clone(o), i, value)),
            other => {
                let owned;
                let name: &[u8] = match first {
                    FieldStep::Prop(n) => n,
                    _ => {
                        owned = prop_dyn_name(keys, diags);
                        &owned
                    }
                };
                // PHP 8: writing a property of a non-object is an Error —
                // a LEAF write is "assign", drilling further is "modify"
                // (zend_throw_non_object_error's verb split). The incdec
                // funnel shares this arm and PHP's dedicated
                // "increment/decrement" verb is a residual divergence.
                Err(PhpError::Error(format!(
                    "Attempt to {} property \"{}\" on {}",
                    if rest.is_empty() { "assign" } else { "modify" },
                    String::from_utf8_lossy(name),
                    other.type_name_for_error()
                )))
            }
        },
        FieldStep::Index => {
            let key = keys.next().expect("field index key");
            // A *string* base takes the single-byte offset write (`$s[0] = 'X'`,
            // zend_assign_to_string_offset), not the array path. An empty string
            // is still a string here (oracle: `$e=''; $e[0]='a'` → "a").
            if matches!(target, Zval::Str(_)) {
                if !rest.is_empty() {
                    return Err(PhpError::Error(
                        "Cannot use string offset as an array".to_string(),
                    ));
                }
                return string_offset_write(target, &key, &value, diags).map(|_| WriteWalk::Done);
            }
            if matches!(target, Zval::Object(_)) {
                // A dim-write on an object defers to ArrayAccess dispatch in
                // the VM caller: offsetSet at the leaf, offsetGet + resumed
                // drill mid-path (`$this->coll[0]->foo = 1`).
                if rest.is_empty() {
                    *aa = Some(AaOp::Write(AaWrite { obj: target.clone(), key: Some(key), value }));
                } else {
                    *aa = Some(AaOp::Descend(AaDescend {
                        obj: target.clone(),
                        key,
                        rest: rest.to_vec(),
                        keys: keys.collect(),
                        value,
                    }));
                }
                return Ok(WriteWalk::Done);
            }
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let k = coerce_key_diag(&key, diags)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if rest.is_empty() {
                // Overwrite a plain element, but write *through* an existing
                // reference element (the continued walk derefs at its top —
                // a ref element is a boundary token, so a cycle closing at
                // the leaf re-borrows only after this guard-free frame ends).
                match arr.get_mut(&k) {
                    Some(child) => field_write_walk(child, steps, i + 1, keys, fs, value, diags, dropped, aa, rebind, rw),
                    None => {
                        arr.insert(k, value);
                        Ok(WriteWalk::Done)
                    }
                }
            } else {
                if !arr.contains_key(&k) {
                    arr.insert(k.clone(), Zval::Array(Rc::new(PhpArray::new())));
                }
                let child = arr.get_mut(&k).expect("key just inserted");
                field_write_walk(child, steps, i + 1, keys, fs, value, diags, dropped, aa, rebind, rw)
            }
        }
        FieldStep::Append => {
            if matches!(target, Zval::Str(_)) {
                return Err(PhpError::Error(
                    "[] operator not supported for strings".to_string(),
                ));
            }
            if matches!(target, Zval::Object(_)) {
                if rest.is_empty() {
                    *aa = Some(AaOp::Write(AaWrite { obj: target.clone(), key: None, value }));
                    return Ok(WriteWalk::Done);
                }
                // `$o->c[]->x = v` — appending then drilling has no PHP
                // equivalent through ArrayAccess; keep the object-as-array error.
                let Zval::Object(o) = target else { unreachable!() };
                // BORROW-OK: shared try-read for the message only; a busy cell
                // (caller-held guard) falls back to an empty name rather than
                // panicking (boundary discipline, M-71.1).
                let cname = o
                    .try_borrow()
                    .map(|b| String::from_utf8_lossy(b.class_name.as_bytes()).into_owned())
                    .unwrap_or_default();
                return Err(PhpError::Error(format!(
                    "Cannot use object of type {cname} as array"
                )));
            }
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let occupied =
                || PhpError::Error("Cannot add element to the array as the next element is already occupied".to_string());
            if rest.is_empty() {
                arr.append(value).map_err(|_| occupied())?;
            } else {
                // A fresh child can never route back into an existing cell
                // (it contains no Ref/Object boundary), so a nested DRIVER
                // run is cycle-safe here.
                let mut child = Zval::Array(Rc::new(PhpArray::new()));
                field_write(&mut child, rest, keys, fs, value, diags, dropped, aa, rebind, rw)?;
                arr.append(child).map_err(|_| occupied())?;
            }
            Ok(WriteWalk::Done)
        }
    }
}

/// One property step (`steps[i]` is Prop/PropDyn) on an object HANDLE — the
/// driver-side half of the boundary discipline (M-71.1, mirrors
/// `field_prop_step`): the object is borrowed ONCE here and the borrow ends
/// when this function returns (the interior [`field_write_walk`] stops at
/// the next boundary).
fn field_write_prop_step(
    o: &Rc<RefCell<Object>>,
    steps: &[FieldStep],
    i: usize,
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
    value: Zval,
    diags: &mut Diags,
    dropped: &mut Vec<Zval>,
    aa: &mut Option<AaOp>,
    rebind: bool,
    rw: bool,
) -> Result<WriteWalk, PhpError> {
    let owned;
    let name: &[u8] = match &steps[i] {
        FieldStep::Prop(n) => n,
        _ => {
            owned = prop_dyn_name(keys, diags);
            &owned
        }
    };
    let rest = &steps[i + 1..];
    // Own a handle so the defer branches can move the object into the
    // pending op after the guard drops.
    let o = Rc::clone(o);
    let Ok(mut obj) = o.try_borrow_mut() else {
        // Only a caller-held guard can trip this (the driver dropped its own
        // before the step): the object is mid-write on this very statement.
        // Skip route, counted (M-70.2).
        cell_skip_note();
        return Ok(WriteWalk::Done);
    };
    let cid = obj.class_id as usize;
    let is_enum = obj.info.is_enum_case;
    let is_lazy = obj.lazy.is_some();
    // PHP 8.4 container-fetch guard: drilling INTO a readonly or
    // set-visibility-denied property whose value is not an object is
    // "Cannot indirectly modify …" (and a compound fetch of an
    // uninitialized TYPED slot is the uninit fatal), BEFORE any
    // magic-descend deferral — the slot is declared and get-visible here,
    // so Zend never reaches `__get` for it.
    if !rest.is_empty() && !is_enum && !is_lazy {
        let state = prop_slot_state(obj.props.get(fs.prop_key(cid, name).as_ref()));
        prop_indirect_guard(fs.classes, fs.scope, cid, name, state, rw, false)?;
    }
    // A leaf write on a property that is not a declared, accessible
    // slot defers to the VM caller (`prop_set_magic_or_dynamic`),
    // which dispatches `__set` (guarded against re-entrant recursion)
    // or, absent a magic setter, materialises a dynamic property /
    // enforces visibility. A declared accessible slot is written
    // directly below — PHP never invokes `__set` for an accessible
    // property. Enum cases keep their dedicated immutability error.
    // A *hooked* property (PHP 8.4) also defers: its set hook (or
    // the hooked write rules) must run, not a raw storage write.
    // So does a *lazy* object (uninitialized, or a forwarding
    // proxy): the write must trigger initialization / follow the
    // proxy, not land on the wrapper's raw storage.
    // A reference-BIND (`=&`) at the leaf must REPLACE the slot
    // with the incoming reference (and let the VM register the
    // typed-ref source afterwards); it must NOT be diverted to
    // `__set`/hook/coerce-as-value dispatch, which would drop the
    // aliasing (`typed_properties_071`, oss-fuzz hooked backing).
    if rest.is_empty()
        && !is_enum
        && (!fs.prop_is_declared_slot(cid, name)
            || (!rebind && (fs.prop_hooked(cid, name) || fs.prop_typed(cid, name)))
            || is_lazy)
    {
        drop(obj);
        *aa = Some(AaOp::MagicSet(AaMagicSet {
            obj: Zval::Object(o),
            name: name.to_vec(),
            value,
        }));
        return Ok(WriteWalk::Done);
    }
    // An intermediate step whose raw slot is absent or
    // inaccessible needs the VM (overloaded-property protocol /
    // no-autoviv semantics) — defer the remaining path. Lazy
    // wrappers keep the legacy raw walk (their realization has
    // its own machinery); enum cases fall through to their
    // dedicated immutability error below.
    if !rest.is_empty() && !is_enum && !is_lazy {
        let denied = fs.prop_key_read(cid, name).is_none();
        let absent = !obj.props.contains(fs.prop_key(cid, name).as_ref());
        if denied || absent {
            drop(obj);
            *aa = Some(AaOp::MagicDescend(AaMagicDescend {
                obj: Zval::Object(o),
                name: name.to_vec(),
                rest: rest.to_vec(),
                keys: keys.collect(),
                value,
            }));
            return Ok(WriteWalk::Done);
        }
    }
    let key = fs.prop_key(cid, name);
    let key = key.as_ref();
    if obj.info.is_enum_case {
        let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
        let prop = String::from_utf8_lossy(name).into_owned();
        return Err(PhpError::Error(if obj.props.contains(key) {
            format!("Cannot modify readonly property {cls}::${prop}")
        } else {
            format!("Cannot create dynamic property {cls}::${prop}")
        }));
    }
    if rest.is_empty() {
        if let Some(old) = obj.props.replace(key, value) {
            dropped.push(old);
        }
        Ok(WriteWalk::Done)
    } else {
        if !obj.props.contains(key) {
            obj.props.set(key, Zval::Array(Rc::new(PhpArray::new())));
        }
        let child = obj.props.get_mut(key).expect("property just inserted");
        // Continue INSIDE this guard only up to the next boundary; the
        // guard drops when the token returns (boundary discipline).
        field_write_walk(child, steps, i + 1, keys, fs, value, diags, dropped, aa, rebind, rw)
    }
}

pub(super) fn field_get(cell: &Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>, fs: FieldScope) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
        // BORROW-OK: shared read-walk (see silent_get_path).
        return field_get(&rc.borrow(), steps, keys, fs);
    }
    match steps.split_first() {
        None => match cell {
            Zval::Undef => None,
            other => Some(other.deref_clone()),
        },
        Some((first, rest)) => match first {
            FieldStep::Prop(_) | FieldStep::PropDyn => {
                let owned;
                let name: &[u8] = match first {
                    FieldStep::Prop(n) => n,
                    _ => {
                        owned = prop_dyn_name(keys, &mut Diags::new());
                        &owned
                    }
                };
                match cell {
                    Zval::Object(o) => {
                        // BORROW-OK: shared read-walk (see silent_get_path).
                        let obj = o.borrow();
                        // Denied (inaccessible declared) reads as absent here:
                        // this walker backs isset/empty/`??` only.
                        let key = fs.prop_key_read(obj.class_id as usize, name)?;
                        match obj.props.get(key.as_ref()) {
                            Some(v) => field_get(v, rest, keys, fs),
                            None => None,
                        }
                    }
                    _ => None,
                }
            }
            FieldStep::Index => {
                let key = keys.next()?;
                match cell {
                    Zval::Array(a) => {
                        let k = coerce_key_silent(&key)?;
                        a.get(&k).and_then(|c| field_get(c, rest, keys, fs))
                    }
                    Zval::Str(s) if rest.is_empty() => {
                        string_offset(s, &key).map(|byte| Zval::Str(PhpStr::new(vec![byte])))
                    }
                    _ => None,
                }
            }
            FieldStep::Append => None,
        },
    }
}

pub(super) fn field_unset(target: &mut Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>, fs: FieldScope, aa: &mut Option<UnsetAa>) -> Result<(), PhpError> {
    let mut walk = field_unset_walk(target, steps, 0, keys, fs, aa)?;
    loop {
        walk = match walk {
            UnsetWalk::Done => return Ok(()),
            UnsetWalk::IntoRef(rc, i) => match rc.try_borrow_mut() {
                Ok(mut g) => field_unset_walk(&mut g, steps, i, keys, fs, aa)?,
                Err(_) => {
                    // Every walker guard is already dropped (boundary
                    // discipline) — only a caller-held guard can trip this.
                    // Skip route, counted (M-70.2).
                    cell_skip_note();
                    UnsetWalk::Done
                }
            },
            UnsetWalk::IntoObj(o, i) => field_unset_prop_step(&o, steps, i, keys, fs, aa)?,
        };
    }
}

/// M-71.1 (WP-71): boundary token of the property/dim UNSET walk — same
/// discipline as [`WriteWalk`].
enum UnsetWalk {
    Done,
    IntoRef(Rc<RefCell<Zval>>, usize),
    IntoObj(Rc<RefCell<Object>>, usize),
}

/// M-72.1 (WP-72): the pending ArrayAccess dispatch an UNSET walk produced —
/// `unset($aa[k])` at the leaf is `offsetUnset`, mid-path is `offsetGet` +
/// resumed walk on the result (a non-object result is Zend's
/// indirect-modification notice and nothing is removed). Mirrors [`AaOp`]:
/// the walker parks, the VM drains — user code never runs under a walk guard.
pub(super) enum UnsetAa {
    Unset { obj: Zval, key: Zval },
    Descend { obj: Zval, key: Zval, rest: Vec<FieldStep>, keys: Vec<Zval> },
}

/// Walk `steps[i..]` from `target` up to the next cell boundary (see
/// [`UnsetWalk`]); plain containers are crossed in place — no guard
/// outlives the return.
fn field_unset_walk(
    target: &mut Zval,
    steps: &[FieldStep],
    i: usize,
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
    aa: &mut Option<UnsetAa>,
) -> Result<UnsetWalk, PhpError> {
    if let Zval::Ref(rc) = target {
        return Ok(UnsetWalk::IntoRef(Rc::clone(rc), i));
    }
    let Some(first) = steps.get(i) else {
        return Ok(UnsetWalk::Done);
    };
    match first {
        FieldStep::Prop(_) | FieldStep::PropDyn => match target {
            Zval::Object(o) => Ok(UnsetWalk::IntoObj(Rc::clone(o), i)),
            // Unsetting a property of a non-object is a silent no-op here
            // (the VM-level PropUnset op has its own diagnostics).
            _ => Ok(UnsetWalk::Done),
        },
        FieldStep::Index => {
            let Some(key) = keys.next() else { return Ok(UnsetWalk::Done) };
            if let Zval::Array(rc) = target {
                if let Some(k) = coerce_key_silent(&key) {
                    let arr = Rc::make_mut(rc);
                    if steps.len() == i + 1 {
                        arr.remove(&k);
                    } else if let Some(child) = arr.get_mut(&k) {
                        return field_unset_walk(child, steps, i + 1, keys, fs, aa);
                    }
                }
            } else if matches!(target, Zval::Object(_)) {
                // M-72.1: a dim step on an object defers to the VM drain —
                // ArrayAccess `offsetUnset` at the leaf, `offsetGet` +
                // resumed walk mid-path (a non-AA object errors there).
                if steps.len() == i + 1 {
                    *aa = Some(UnsetAa::Unset { obj: target.clone(), key });
                } else {
                    *aa = Some(UnsetAa::Descend {
                        obj: target.clone(),
                        key,
                        rest: steps[i + 1..].to_vec(),
                        keys: keys.collect(),
                    });
                }
            }
            Ok(UnsetWalk::Done)
        }
        FieldStep::Append => Ok(UnsetWalk::Done),
    }
}

/// One property step of the UNSET walk on an object HANDLE — the object is
/// borrowed ONCE here and the borrow ends when this function returns
/// (M-71.1, mirrors [`field_write_prop_step`]).
fn field_unset_prop_step(
    o: &Rc<RefCell<Object>>,
    steps: &[FieldStep],
    i: usize,
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
    aa: &mut Option<UnsetAa>,
) -> Result<UnsetWalk, PhpError> {
    let owned;
    let name: &[u8] = match &steps[i] {
        FieldStep::Prop(n) => n,
        _ => {
            owned = prop_dyn_name(keys, &mut Diags::new());
            &owned
        }
    };
    let rest_empty = steps.len() == i + 1;
    let Ok(mut obj) = o.try_borrow_mut() else {
        // Only a caller-held guard can trip this (the driver dropped its
        // own before the step). Skip route, counted (M-70.2).
        cell_skip_note();
        return Ok(UnsetWalk::Done);
    };
    let cid = obj.class_id as usize;
    // PHP 8.4 container-fetch guard, UNSET flavour: drilling into
    // a readonly / set-denied property errors on a non-object
    // value; an uninitialized slot is a silent no-op (Zend's
    // BP_VAR_UNSET read of an UNDEF slot).
    if !rest_empty {
        let state = prop_slot_state(obj.props.get(fs.prop_key(cid, name).as_ref()));
        prop_indirect_guard(fs.classes, fs.scope, cid, name, state, false, true)?;
    }
    let key = fs.prop_key(cid, name);
    if rest_empty {
        // M-72.1 leaf-op parity with Op::PropUnset: enum-case → visibility →
        // asym set-visibility → readonly. (`__unset`/hook dispatch runs at
        // the op level BEFORE the walker; the typed-ref release stays
        // op-level too — pre-existing walker gap.)
        if obj.info.is_enum_case {
            return Err(PhpError::Error(format!(
                "Cannot unset readonly property {}::${}",
                String::from_utf8_lossy(obj.class_name.as_bytes()),
                String::from_utf8_lossy(name)
            )));
        }
        check_prop_access(fs.classes, fs.scope, cid, name)?;
        if let Some(err) = asym_write_error(fs.classes, fs.scope, cid, name, "unset") {
            return Err(err);
        }
        if let Some(decl) = prop_readonly_decl(fs.classes, cid, name) {
            if obj.readonly_clone_writable(key.as_ref()) {
                obj.clear_readonly_init(key.as_ref());
            } else {
                let inited = obj.is_readonly_init(key.as_ref());
                let set_vis =
                    prop_info(fs.classes, cid, name).and_then(|pi| pi.set_visibility);
                if let Some(err) =
                    readonly_write_error(fs.classes, fs.scope, decl, name, inited, set_vis)
                {
                    let PhpError::Error(m) = err else { return Err(err) };
                    return Err(PhpError::Error(
                        m.replacen("Cannot modify", "Cannot unset", 1),
                    ));
                }
            }
        }
        // A declared TYPED property returns to *uninitialized* on
        // unset (mirrors Op::PropUnset; doctrine unsets via a
        // bound closure with a dynamic name).
        let typed =
            obj.info.type_of(key.as_ref()).is_some() || obj.info.type_of(name).is_some();
        if typed {
            obj.props.set(key.as_ref(), Zval::Undef);
            obj.mark_typed_unset(key.as_ref());
        } else {
            obj.props.remove(key.as_ref());
        }
        Ok(UnsetWalk::Done)
    } else if let Some(child) = obj.props.get_mut(key.as_ref()) {
        // Continue INSIDE this guard only up to the next boundary; the
        // guard drops when the token returns (boundary discipline).
        field_unset_walk(child, steps, i + 1, keys, fs, aa)
    } else {
        Ok(UnsetWalk::Done)
    }
}

/// Read a local cell's value, following a reference and mapping an unset slot to
/// NULL.
pub(super) fn read_slot(cell: &Zval) -> Zval {
    match cell {
        Zval::Undef => Zval::Null,
        Zval::Ref(r) => r.borrow().clone(),
        other => other.clone(),
    }
}

/// Coerce an index value to an array [`Key`] without raising diagnostics — the
/// proof slice reads and writes silently. Mirrors `eval::coerce_key` minus the
/// deprecation/warning pushes; `None` marks an illegal offset type
/// (array/object/closure/generator/resource).
/// [`coerce_key_silent`] plus PHP 8.1's lossy-float offset deprecation:
/// "Implicit conversion from float %s to int loses precision" fires in every
/// offset context (write, read, coalesce — oracle-pinned; WP's
/// `add_action($h, $cb, 9.5)` tests expect it). The isset/unset proof paths
/// keep the silent funnel (no diags in scope there; residual divergence).
pub(super) fn coerce_key_diag(v: &Zval, diags: &mut Diags) -> Option<Key> {
    if let Zval::Ref(c) = v {
        // H-72.5: a key can reach the walkers as a Ref, and these run INSIDE
        // a prop-step guard — a busy key cell (the key aliases a cell
        // mid-write on this very statement) is counted and treated as
        // un-coercible, never a panic.
        return match c.try_borrow() {
            Ok(g) => coerce_key_diag(&g, diags),
            Err(_) => {
                cell_skip_note();
                None
            }
        };
    }
    if let Zval::Double(d) = v {
        let l = convert::dval_to_lval(*d);
        if !convert::is_long_compatible(*d, l) {
            let rendered = convert::to_zstr(&Zval::Double(*d), diags);
            diags.push(Diag::Deprecated(format!(
                "Implicit conversion from float {} to int loses precision",
                String::from_utf8_lossy(rendered.as_bytes())
            )));
        }
        return Some(Key::Int(l));
    }
    coerce_key_silent(v)
}

pub(super) fn coerce_key_silent(v: &Zval) -> Option<Key> {
    match v {
        Zval::Long(i) => Some(Key::Int(*i)),
        Zval::Bool(b) => Some(Key::Int(*b as i64)),
        Zval::Double(d) => Some(Key::Int(convert::dval_to_lval(*d))),
        Zval::Str(s) => Some(Key::from_zstr(s)),
        Zval::Null | Zval::Undef => Some(Key::from_bytes(b"")),
        // H-72.5: try-routed for the same reason as coerce_key_diag — the
        // walkers call this inside a prop-step guard.
        Zval::Ref(c) => match c.try_borrow() {
            Ok(g) => coerce_key_silent(&g),
            Err(_) => {
                cell_skip_note();
                None
            }
        },
        _ => None,
    }
}

/// Snapshot an iterable into `(key, value)` pairs for `foreach`. An array (or a
/// reference to one) is copied element-wise — by-value `foreach` iterates this
/// snapshot, so the body mutating the source can't disturb the loop. Any other
/// value iterates zero times for now (object / Traversable support is OOP work).
///
/// Element values are cloned *shallowly* (`v.clone()`), so a reference element
/// keeps sharing its cell and is read live at bind time (see `IterNext`). This
/// is what reproduces PHP's lingering-reference gotcha — a `foreach (… as &$v)`
/// followed by `foreach (… as $v)` mutates the last element (D-R13) — and
/// mirrors the tree-walker (`eval::exec_foreach`).
pub(super) fn snapshot_entries(iterable: &Zval) -> Vec<(Zval, Zval)> {
    match iterable {
        Zval::Array(a) => a.iter().map(|(k, v)| (key_to_zval(&k), v.clone())).collect(),
        Zval::Ref(rc) => snapshot_entries(&rc.borrow()),
        _ => Vec::new(),
    }
}

/// Materialise an array [`Key`] as the [`Zval`] `foreach` binds to its key slot.
pub(super) fn key_to_zval(k: &Key) -> Zval {
    match k {
        Key::Int(i) => Zval::Long(*i),
        Key::Str(s) => Zval::Str(Rc::clone(s)),
    }
}

/// Read `base[key]` by value (silent). Array elements deref-clone; a string base
/// reads a byte offset; anything else (or a missing key) yields NULL.
pub(super) fn read_dim(base: &Zval, key: &Zval) -> Zval {
    match base {
        Zval::Array(a) => match coerce_key_silent(key) {
            Some(k) => a.get(&k).map(|v| v.deref_clone()).unwrap_or(Zval::Null),
            None => Zval::Null,
        },
        Zval::Str(s) => read_string_offset(s, key),
        Zval::Ref(rc) => read_dim(&rc.borrow(), key),
        _ => Zval::Null,
    }
}


/// Like [`read_dim`] but raises `Warning: Undefined array key K` when an array
/// key is absent (the warning-ful read context — `Op::FetchDim`, e.g. `echo
/// $a[5]`), and carries the string-offset read diagnostics (§3.7).
pub(super) fn read_dim_warn(base: &Zval, key: &Zval, diags: &mut Diags) -> Result<Zval, PhpError> {
    match base {
        Zval::Array(a) => match coerce_key_diag(key, diags) {
            Some(k) => match a.get(&k) {
                Some(v) => Ok(v.deref_clone()),
                None => {
                    let msg = match &k {
                        Key::Int(i) => format!("Undefined array key {i}"),
                        Key::Str(s) => {
                            format!("Undefined array key \"{}\"", String::from_utf8_lossy(s.as_bytes()))
                        }
                    };
                    diags.push(Diag::Warning(msg));
                    Ok(Zval::Null)
                }
            },
            None => Ok(Zval::Null),
        },
        Zval::Ref(rc) => read_dim_warn(&rc.borrow(), key, diags),
        // `$false[0]` & co.: PHP 7.4+ warns naming the value (false/null/…)
        // and yields null. Strings keep the silent byte-offset path below.
        Zval::Null | Zval::Undef | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_) => {
            diags.push(Diag::Warning(format!(
                "Trying to access array offset on {}",
                base.value_name_for_error()
            )));
            Ok(Zval::Null)
        }
        // A warning-ful READ of a string offset (oracle-pinned, §3.7): an
        // integral string key is silent; a leading-numeric one ("5abc") warns
        // `Illegal string offset` and uses its integer prefix; any other
        // string (incl. "1.5") or a container key is PHP 8's TypeError; a
        // float/bool/null key warns `String offset cast occurred` (no
        // float-precision deprecation on this path — zend_dval_to_lval);
        // out of range warns `Uninitialized string offset N` with the
        // pre-adjustment cast value and yields "". isset/`??` are separate
        // silent funnels ([`read_dim_nullable`], [`silent_get_path`]).
        Zval::Str(s) => {
            let kd = key.deref_clone();
            let i = match &kd {
                Zval::Str(k) => match php_types::numstr::parse_numeric_ex(k.as_bytes(), true) {
                    Some(info) if matches!(info.num, php_types::numstr::Num::Long(_)) => {
                        let php_types::numstr::Num::Long(l) = info.num else {
                            unreachable!()
                        };
                        if info.trailing {
                            diags.push(Diag::Warning(format!(
                                "Illegal string offset \"{}\"",
                                String::from_utf8_lossy(k.as_bytes())
                            )));
                        }
                        l
                    }
                    _ => {
                        return Err(PhpError::TypeError(
                            "Cannot access offset of type string on string".to_string(),
                        ))
                    }
                },
                Zval::Array(_) | Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_) => {
                    return Err(PhpError::TypeError(format!(
                        "Cannot access offset of type {} on string",
                        kd.type_name_for_error()
                    )))
                }
                Zval::Long(l) => *l,
                Zval::Resource(r) => {
                    let id = r.borrow().id;
                    diags.push(Diag::Warning(format!(
                        "Resource ID#{id} used as offset, casting to integer ({id})"
                    )));
                    id as i64
                }
                _ => {
                    diags.push(Diag::Warning("String offset cast occurred".to_string()));
                    convert::to_long_cast(&kd, &mut Diags::new())
                }
            };
            let len = s.len() as i64;
            let idx = if i < 0 { len + i } else { i };
            if idx < 0 || idx >= len {
                diags.push(Diag::Warning(format!("Uninitialized string offset {i}")));
                Ok(Zval::Str(PhpStr::new(Vec::new())))
            } else {
                Ok(Zval::Str(PhpStr::new(vec![s.as_bytes()[idx as usize]])))
            }
        }
        _ => Ok(read_dim(base, key)),
    }
}

/// [`read_dim_warn`] for a `list()` element read ([`crate::bytecode::Op::FetchDimList`]):
/// the undefined-key Warning stays, but a scalar base is SILENT — Zend's list
/// path raises no offset-on-scalar warning (`list($a) = null` is quiet).
pub(super) fn read_dim_warn_list(base: &Zval, key: &Zval, diags: &mut Diags) -> Zval {
    match base {
        Zval::Array(_) => read_dim_warn(base, key, diags).unwrap_or(Zval::Null),
        Zval::Ref(rc) => read_dim_warn_list(&rc.borrow(), key, diags),
        _ => read_dim(base, key),
    }
}

/// String byte-offset read `$s[i]` (silent): integer index, negatives count from
/// the end, out-of-range yields `""`.
pub(super) fn read_string_offset(s: &PhpStr, key: &Zval) -> Zval {
    match string_offset(s, key) {
        Some(byte) => Zval::Str(PhpStr::new(vec![byte])),
        None => Zval::Str(PhpStr::new(Vec::new())),
    }
}

/// Like [`read_dim`] but isset-aware for the `??` read context: a not-set leaf is
/// NULL rather than `""`. Arrays already yield NULL on a missing key; the
/// difference is a string offset that is out of range or non-integer, which
/// `isset($s[i])` reports as unset — so `$s[i] ?? d` takes the default.
pub(super) fn read_dim_nullable(base: &Zval, key: &Zval) -> Zval {
    match base {
        // Only an integer-valued key is a valid string offset for `isset`; a
        // non-numeric string key (`$s["str"]`) is unset → NULL.
        Zval::Str(s) => match coerce_key_silent(key) {
            Some(Key::Int(_)) => match string_offset(s, key) {
                Some(byte) => Zval::Str(PhpStr::new(vec![byte])),
                None => Zval::Null,
            },
            _ => Zval::Null,
        },
        Zval::Ref(rc) => read_dim_nullable(&rc.borrow(), key),
        _ => read_dim(base, key),
    }
}

/// `zend_assign_to_string_offset`: write `value`'s first byte at `key` into the
/// string held by `target` (oracle-pinned matrix): the offset casts with PHP's
/// warnings (float/bool/null → "String offset cast occurred", non-numeric
/// string → TypeError), a negative offset counts from the end (out of range →
/// "Illegal string offset" warning, no write), one past the end pads with
/// spaces, an empty value is an Error and a multi-byte one keeps its first
/// byte with a warning. Returns the single-byte string PHP yields as the
/// assignment expression's value (the *unchanged* target on the no-write path).
pub(super) fn string_offset_write(
    target: &mut Zval,
    key: &Zval,
    value: &Zval,
    diags: &mut Diags,
) -> Result<Zval, PhpError> {
    let Zval::Str(s) = target else {
        return Err(PhpError::Error("string_offset_write on a non-string".to_string()));
    };
    let off = match key.deref_clone() {
        Zval::Long(i) => i,
        Zval::Double(d) => {
            diags.push(Diag::Warning("String offset cast occurred".to_string()));
            d as i64
        }
        Zval::Bool(b) => {
            diags.push(Diag::Warning("String offset cast occurred".to_string()));
            b as i64
        }
        Zval::Null | Zval::Undef => {
            diags.push(Diag::Warning("String offset cast occurred".to_string()));
            0
        }
        Zval::Str(k) => match std::str::from_utf8(k.as_bytes())
            .ok()
            .and_then(|t| t.trim().parse::<i64>().ok())
        {
            Some(i) => i,
            None => {
                return Err(PhpError::TypeError(
                    "Cannot access offset of type string on string".to_string(),
                ))
            }
        },
        other => {
            return Err(PhpError::TypeError(format!(
                "Cannot access offset of type {} on string",
                other.type_name_for_error()
            )))
        }
    };
    let vbytes = convert::to_zstr_cast(value, diags).as_bytes().to_vec();
    if vbytes.is_empty() {
        return Err(PhpError::Error(
            "Cannot assign an empty string to a string offset".to_string(),
        ));
    }
    if vbytes.len() > 1 {
        diags.push(Diag::Warning(
            "Only the first byte will be assigned to the string offset".to_string(),
        ));
    }
    let mut bytes = s.as_bytes().to_vec();
    let idx = if off < 0 { bytes.len() as i64 + off } else { off };
    if idx < 0 {
        diags.push(Diag::Warning(format!("Illegal string offset {off}")));
        return Ok(target.clone());
    }
    let idx = idx as usize;
    if idx >= bytes.len() {
        bytes.resize(idx + 1, b' ');
    }
    bytes[idx] = vbytes[0];
    *target = Zval::Str(PhpStr::new(bytes));
    Ok(Zval::Str(PhpStr::new(vec![vbytes[0]])))
}

/// Ensure `cell` is an array, auto-vivifying from null/undefined/false; a
/// non-empty scalar cannot become an array.
pub(super) fn ensure_array(cell: &mut Zval) -> Result<(), PhpError> {
    match cell {
        Zval::Undef | Zval::Null | Zval::Bool(false) => {
            *cell = Zval::Array(Rc::new(PhpArray::new()));
            Ok(())
        }
        Zval::Array(_) => Ok(()),
        _ => Err(PhpError::Error(
            "Cannot use a scalar value as an array".to_string(),
        )),
    }
}

impl<'m> Vm<'m> {
    /// Run a path write/compound/incdec rooted at `base`, drilling through the
    /// intermediate `keys` and applying `last` at the leaf. Returns the value the
    /// expression evaluates to (assigned value / compound result / inc-dec value).
    pub(super) fn path_op(
        &mut self,
        base: DimBase,
        top: usize,
        keys: Vec<Zval>,
        last: Last,
    ) -> Result<Zval, PhpError> {
        let cell = match base {
            DimBase::Local(s) => &mut self.frames[top].slots[s as usize],
            DimBase::Global(s) => &mut self.frames[0].slots[s as usize],
            DimBase::Superglobal(i) => &mut self.superglobals[i as usize],
        };
        // The element displaced by the write (e.g. `$a[0] = new X` overwriting
        // the old `$a[0]`) lands here — at most one per path op — then is
        // noted as a possible GC root once the borrow of the base cell ends.
        let mut dropped = None;
        let mut aa = None;
        let mut refw = None;
        let result = path_apply(cell, &keys, last, &mut self.diags, &mut dropped, &mut aa, &mut refw);
        if let Some(d) = &dropped {
            self.gc_note(d);
        }
        let mut result = result?;
        // H-70.1 drain: a leaf write whose target ref-cell was mid-write on
        // this very statement (a reference cycle) parks here; every walk
        // guard is gone now, so the store proceeds — Zend's semantics for a
        // write through the collapsed pointer. At most one per path op.
        if let Some(rl) = refw.take() {
            match rl {
                super::RefLeaf::Set { cell, value } => {
                    match cell.try_borrow_mut() {
                        Ok(mut g) => {
                            let d = std::mem::replace(&mut *g, value);
                            drop(g);
                            self.gc_note(&d);
                        }
                        // H-71.3: declared unreachable (all guards gone).
                        Err(_) => super::drain_fail_note(),
                    }
                }
                super::RefLeaf::OpSet { cell, op, rhs } => {
                    // H-72.3: a busy cell is counted and SKIPPED — the drain
                    // never computes `op(Null, rhs)` into the cell (a
                    // fabricated value would be a deferred divergence).
                    let old = match cell.try_borrow() {
                        Ok(g) => Some(g.clone()),
                        Err(_) => {
                            // H-71.3: declared unreachable (all guards gone).
                            super::drain_fail_note();
                            None
                        }
                    };
                    if let Some(old) = old {
                        let new = super::apply_binop(op, &old, &rhs, &mut self.diags)?;
                        match cell.try_borrow_mut() {
                            Ok(mut g) => {
                                let d = std::mem::replace(&mut *g, new.clone());
                                drop(g);
                                self.gc_note(&d);
                            }
                            // H-71.3: declared unreachable (all guards gone).
                            Err(_) => super::drain_fail_note(),
                        }
                        result = new;
                    }
                }
                super::RefLeaf::IncDec { cell, inc, pre } => {
                    let vals = match cell.try_borrow_mut() {
                        Ok(mut g) => {
                            let old = g.clone();
                            if inc {
                                super::ops::increment(&mut g, &mut self.diags)?;
                            } else {
                                super::ops::decrement(&mut g, &mut self.diags)?;
                            }
                            Some((old, g.clone()))
                        }
                        Err(_) => {
                            // H-71.3: declared unreachable (all guards gone).
                            super::drain_fail_note();
                            None
                        }
                    };
                    if let Some((old, newv)) = vals {
                        result = if pre { newv } else { old };
                    }
                }
            }
        }
        // Drain the parked ArrayAccess dispatches: offsetSet at the leaf,
        // offsetGet→op→offsetSet for compound/incdec leaves, offsetGet +
        // resumed drill mid-path (`$ctx[0][$i] = v` on nested SplFixedArrays).
        let mut pending = aa;
        while let Some(op) = pending.take() {
            let obj_of = |op: &super::PathAa| match op {
                super::PathAa::Write(w) => w.obj.clone(),
                super::PathAa::Op { obj, .. }
                | super::PathAa::IncDec { obj, .. }
                | super::PathAa::Descend { obj, .. } => obj.clone(),
            };
            let obj = obj_of(&op);
            if !self.object_implements(&obj, b"arrayaccess") {
                let name = deref_object(&obj)
                    .and_then(|o| {
                            o.try_borrow().ok().map(|b| {
                                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                            })
                        })
                    .unwrap_or_default();
                return Err(PhpError::Error(format!("Cannot use object of type {name} as array")));
            }
            match op {
                super::PathAa::Write(AaWrite { obj, key, value }) => {
                    self.call_method_sync(obj, b"offsetSet", vec![key.unwrap_or(Zval::Null), value])?;
                }
                super::PathAa::Op { obj, key, op, rhs } => {
                    let old = self
                        .call_method_sync(obj.clone(), b"offsetGet", vec![key.clone()])?
                        .deref_clone();
                    #[cfg(feature = "op-census")]
                    if matches!(op, crate::hir::BinOp::Concat) {
                        crate::vm::census::census_concat_site(5, &old, &rhs);
                    }
                    let new = super::apply_binop(op, &old, &rhs, &mut self.diags)?;
                    self.call_method_sync(obj, b"offsetSet", vec![key, new.clone()])?;
                    result = new;
                }
                super::PathAa::IncDec { obj, key, inc, pre } => {
                    let old = self
                        .call_method_sync(obj.clone(), b"offsetGet", vec![key.clone()])?
                        .deref_clone();
                    let mut new = old.clone();
                    if inc {
                        super::ops::increment(&mut new, &mut self.diags)?;
                    } else {
                        super::ops::decrement(&mut new, &mut self.diags)?;
                    }
                    self.call_method_sync(obj, b"offsetSet", vec![key, new.clone()])?;
                    result = if pre { new } else { old };
                }
                super::PathAa::Descend { obj, key, rest, last } => {
                    let cname = deref_object(&obj)
                        .and_then(|o| {
                            o.try_borrow().ok().map(|b| {
                                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                            })
                        })
                        .unwrap_or_default();
                    let mut val =
                        self.call_method_sync(obj, b"offsetGet", vec![key])?.deref_clone();
                    if !matches!(val, Zval::Object(_)) {
                        // Writing through a by-value offsetGet result mutates a
                        // temporary — PHP's notice, and no write happens.
                        self.diags.push(Diag::Notice(format!(
                            "Indirect modification of overloaded element of {cname} has no effect"
                        )));
                        break;
                    }
                    let mut dropped2 = None;
                    let mut aa2 = None;
                    let mut refw2 = None;
                    let r = path_apply(&mut val, &rest, *last, &mut self.diags, &mut dropped2, &mut aa2, &mut refw2);
                    if let Some(d) = &dropped2 {
                        self.gc_note(d);
                    }
                    r?;
                    // H-70.1: the nested walk's guards are gone here; the
                    // nested result value is discarded (as `r` above), so
                    // Set-shape stores suffice for all three leaves.
                    if let Some(rl) = refw2.take() {
                        // H-72.3: a busy cell is counted and SKIPPED — never
                        // `op(Null, ·)` fabricated into the store.
                        let store = match rl {
                            super::RefLeaf::Set { cell, value } => Some((cell, value)),
                            super::RefLeaf::OpSet { cell, op, rhs } => {
                                let old = match cell.try_borrow() {
                                    Ok(g) => Some(g.clone()),
                                    Err(_) => {
                                        // H-71.3: declared unreachable.
                                        super::drain_fail_note();
                                        None
                                    }
                                };
                                match old {
                                    Some(old) => {
                                        let new = super::apply_binop(op, &old, &rhs, &mut self.diags)?;
                                        Some((cell, new))
                                    }
                                    None => None,
                                }
                            }
                            super::RefLeaf::IncDec { cell, inc, pre: _ } => {
                                let old = match cell.try_borrow() {
                                    Ok(g) => Some(g.clone()),
                                    Err(_) => {
                                        // H-71.3: declared unreachable.
                                        super::drain_fail_note();
                                        None
                                    }
                                };
                                match old {
                                    Some(mut new) => {
                                        if inc {
                                            super::ops::increment(&mut new, &mut self.diags)?;
                                        } else {
                                            super::ops::decrement(&mut new, &mut self.diags)?;
                                        }
                                        Some((cell, new))
                                    }
                                    None => None,
                                }
                            }
                        };
                        if let Some((cell, value)) = store {
                            match cell.try_borrow_mut() {
                                Ok(mut g) => {
                                    let d = std::mem::replace(&mut *g, value);
                                    drop(g);
                                    self.gc_note(&d);
                                }
                                // H-71.3: declared unreachable (all guards gone).
                                Err(_) => super::drain_fail_note(),
                            };
                        }
                    }
                    pending = aa2;
                }
            }
        }
        Ok(result)
    }

    pub(super) fn field_set(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
    ) -> Result<(), PhpError> {
        self.field_set_mode(base, top, steps, keys, value, false, false)
    }

    /// [`Self::field_set`] for a COMPOUND write (`+=`, `++`): the container
    /// fetch is Zend's BP_VAR_RW, so an uninitialized typed property along the
    /// path is the uninit fatal (see `prop_indirect_guard`).
    pub(super) fn field_set_op(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
    ) -> Result<(), PhpError> {
        self.field_set_mode(base, top, steps, keys, value, false, true)
    }

    /// [`Self::field_set`] with `rebind` (reference-bind leaf semantics: a
    /// leaf slot holding a reference is REPLACED, not written through).
    pub(super) fn field_set_mode(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
        rebind: bool,
        rw: bool,
    ) -> Result<(), PhpError> {
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        let cell = match base {
            FieldBase::Local(s) => &mut self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &mut self.frames[0].slots[s as usize],
            FieldBase::Superglobal(i) => &mut self.superglobals[i as usize],
            FieldBase::This => self.frames[top].this.as_mut().ok_or_else(|| {
                PhpError::Error("Using $this when not in object context".to_string())
            })?,
        };
        let mut dropped = Vec::new();
        let mut aa = None;
        let r = field_write(cell, steps, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa, rebind, rw);
        for d in &dropped {
            self.gc_note(d);
        }
        r?;
        self.drain_aa_pending(aa, top, rebind, rw)
    }

    /// Write `value` at `steps` inside `root` — an already-resolved path root,
    /// the cell a by-ref (`&get`) property hook returned — with the same
    /// deferred ArrayAccess dispatch as [`Self::field_set_mode`]. The property
    /// hook (and any set hook) is NOT consulted again: PHP writes through the
    /// reference (a `$o->hooked[] = v` runs `&get` once and no `set`).
    pub(super) fn field_set_in_root(
        &mut self,
        root: Rc<RefCell<Zval>>,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
        rebind: bool,
        rw: bool,
    ) -> Result<(), PhpError> {
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        let mut root_val = Zval::Ref(root);
        let mut dropped = Vec::new();
        let mut aa = None;
        let r = field_write(&mut root_val, steps, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa, rebind, rw);
        for d in &dropped {
            self.gc_note(d);
        }
        r?;
        self.drain_aa_pending(aa, top, rebind, rw)
    }

    /// Run the pending ArrayAccess / magic-set dispatches a [`field_write`]
    /// parked (possibly chained through `Descend`: `$this->coll[0]->foo = 1`,
    /// `$a->x[0][1]->y = v`, …). Extracted from [`Self::field_set_mode`] so
    /// hook-rooted writes ([`Self::field_set_in_root`]) share it.
    fn drain_aa_pending(
        &mut self,
        aa: Option<AaOp>,
        top: usize,
        rebind: bool,
        rw: bool,
    ) -> Result<(), PhpError> {
        let mut pending = aa;
        while let Some(op) = pending.take() {
            // A magic / dynamic property write (`$this->e->foo = v`) needs no
            // ArrayAccess; the array dispatches below do.
            if let AaOp::MagicSet(AaMagicSet { obj, name, value }) = op {
                self.prop_set_magic_or_dynamic(obj, &name, value, top)?;
                continue;
            }
            if let AaOp::MagicDescend(AaMagicDescend { obj, name, rest, keys, value }) = op {
                let o = deref_object(&obj).expect("MagicDescend carries an object");
                let (cid, cname) = {
                    // Drain entry: every walk guard is dead (H-70.1) and no
                    // user code has run yet in this dispatch.
                    // BORROW-OK: shared read for cid/name only.
                    let b = o.borrow();
                    (
                        b.class_id as usize,
                        String::from_utf8_lossy(b.class_name.as_bytes()).into_owned(),
                    )
                };
                // Everything `fs` answers is captured up front: the scope
                // borrow may not live across the `&mut self` calls below.
                let (denied_vis, declared, key_owned) = {
                    let fs =
                        FieldScope { classes: &self.classes, scope: self.frames[top].class };
                    (
                        fs.prop_denied_vis(cid, &name),
                        fs.prop_is_declared_slot(cid, &name),
                        fs.prop_key(cid, &name).into_owned(),
                    )
                };
                let prop = String::from_utf8_lossy(&name).into_owned();
                // The property name the assignment would fault on when the
                // step's value is not an object (`$o->p->NEXT = v` on null/…).
                let next_prop = match rest.first() {
                    Some(FieldStep::Prop(n)) => Some(n.to_vec()),
                    Some(FieldStep::PropDyn) => Some(
                        convert::to_zstr_cast(keys.first().unwrap_or(&Zval::Null), &mut self.diags)
                            .as_bytes()
                            .to_vec(),
                    ),
                    _ => None,
                };
                let cur = self.frames[top].class;
                if self.magic_applies(&o, &name, cur, MagicKind::Get, b"__get").is_some() {
                    // Guarded `__get`, then continue the write on its result:
                    // an object is a handle (the write lands); anything else is
                    // Zend's indirect-modification notice, then the write faults
                    // or silently mutates the discarded temporary.
                    // BORROW-OK: shared read pre-call (no guard, `__get` not entered).
                    let oid = o.borrow().id;
                    let gkey = (oid, MagicKind::Get, name.clone());
                    let ins = self.magic_guard.insert(gkey.clone());
                    let r = self.call_method_sync(
                        obj.clone(),
                        b"__get",
                        vec![Zval::Str(PhpStr::new(name.clone()))],
                    );
                    if ins {
                        self.magic_guard.remove(&gkey);
                    }
                    let mut val = r?.deref_clone();
                    if !matches!(val, Zval::Object(_)) {
                        self.diags.push(Diag::Notice(format!(
                            "Indirect modification of overloaded property {cname}::${prop} has no effect"
                        )));
                        // Attribute to the assignment's own line: the drain may
                        // outlive the op's flush point (mirrors BindRefTo).
                        let line = self.cur_line(top);
                        self.flush_diags(line)?;
                        if let Some(next) = next_prop {
                            return Err(PhpError::Error(format!(
                                "Attempt to {} property \"{}\" on {}",
                                if rest.len() > 1 { "modify" } else { "assign" },
                                String::from_utf8_lossy(&next),
                                val.type_name_for_error()
                            )));
                        }
                        break;
                    }
                    let mut dropped = Vec::new();
                    let mut aa2 = None;
                    let fs2 =
                        FieldScope { classes: &self.classes, scope: self.frames[top].class };
                    field_write(&mut val, &rest, &mut keys.into_iter(), fs2, value, &mut self.diags, &mut dropped, &mut aa2, rebind, rw)?;
                    for d in &dropped {
                        self.gc_note(d);
                    }
                    pending = aa2;
                    continue;
                }
                // No magic getter. An inaccessible declared property is the
                // visibility error; an absent one follows PHP's no-autoviv
                // rules — a next *property* step faults on null (the dynamic
                // creation still deprecates first on a non-dynamic class),
                // while an index step autovivifies an array and walks on.
                if let Some(vis) = denied_vis {
                    return Err(PhpError::Error(format!(
                        "Cannot access {vis} property {cname}::${prop}"
                    )));
                }
                if !declared && !self.allows_dynamic_props(cid) {
                    self.diags.push(Diag::Deprecated(format!(
                        "Creation of dynamic property {cname}::${prop} is deprecated"
                    )));
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                }
                if let Some(next) = next_prop {
                    return Err(PhpError::Error(format!(
                        "Attempt to {} property \"{}\" on null",
                        if rest.len() > 1 { "modify" } else { "assign" },
                        String::from_utf8_lossy(&next)
                    )));
                }
                {
                    // The flush_diags above may have run a user error-handler,
                    // but it has RETURNED — no guard survives a re-entry into
                    // the VM (M-71.3 statement-level invariant).
                    // BORROW-OK: scoped autoviv borrow, ends before the walk resumes.
                    let mut b = o.borrow_mut();
                    if !b.props.contains(&key_owned) {
                        b.props.set(&key_owned, Zval::Array(Rc::new(PhpArray::new())));
                    }
                }
                let mut steps: Vec<FieldStep> = Vec::with_capacity(rest.len() + 1);
                steps.push(FieldStep::Prop(name.clone().into_boxed_slice()));
                steps.extend(rest);
                let mut objz = Zval::Object(o.clone());
                let mut dropped = Vec::new();
                let mut aa2 = None;
                let fs2 = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                field_write(&mut objz, &steps, &mut keys.into_iter(), fs2, value, &mut self.diags, &mut dropped, &mut aa2, rebind, rw)?;
                for d in &dropped {
                    self.gc_note(d);
                }
                pending = aa2;
                continue;
            }
            let msg_obj = match &op {
                AaOp::Write(w) => &w.obj,
                AaOp::Descend(d) => &d.obj,
                AaOp::MagicSet(_) | AaOp::MagicDescend(_) => unreachable!("handled above"),
            };
            if !self.object_implements(msg_obj, b"arrayaccess") {
                let name = deref_object(msg_obj)
                    .and_then(|o| {
                            o.try_borrow().ok().map(|b| {
                                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                            })
                        })
                    .unwrap_or_default();
                return Err(PhpError::Error(format!("Cannot use object of type {name} as array")));
            }
            match op {
                AaOp::MagicSet(_) | AaOp::MagicDescend(_) => unreachable!("handled above"),
                AaOp::Write(AaWrite { obj, key, value }) => {
                    self.call_method_sync(obj, b"offsetSet", vec![key.unwrap_or(Zval::Null), value])?;
                }
                AaOp::Descend(AaDescend { obj, key, rest, keys, value }) => {
                    let cname = deref_object(&obj)
                        .and_then(|o| {
                            o.try_borrow().ok().map(|b| {
                                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                            })
                        })
                        .unwrap_or_default();
                    let mut val = self.call_method_sync(obj, b"offsetGet", vec![key])?.deref_clone();
                    if !matches!(val, Zval::Object(_)) {
                        // Writing through a by-value offsetGet result mutates a
                        // temporary — PHP's notice, and no write happens.
                        self.diags.push(Diag::Notice(format!(
                            "Indirect modification of overloaded element of {cname} has no effect"
                        )));
                        break;
                    }
                    let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                    let mut dropped = Vec::new();
                    let mut aa2 = None;
                    field_write(&mut val, &rest, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa2, rebind, rw)?;
                    for d in &dropped {
                        self.gc_note(d);
                    }
                    pending = aa2;
                }
            }
        }
        Ok(())
    }

    /// Run the pending ArrayAccess dispatches an UNSET walk parked (M-72.1),
    /// chaining through `Descend` (`unset($c['a']->b['x'])`). Every walker
    /// guard is dead by the time this runs — user code (`offsetGet` /
    /// `offsetUnset`) never executes under a walk borrow.
    pub(super) fn drain_unset_aa(&mut self, aa: Option<UnsetAa>, top: usize) -> Result<(), PhpError> {
        let mut pending = aa;
        while let Some(op) = pending.take() {
            let recv = match &op {
                UnsetAa::Unset { obj, .. } | UnsetAa::Descend { obj, .. } => obj,
            };
            if !self.object_implements(recv, b"arrayaccess") {
                let name = deref_object(recv)
                    .and_then(|o| {
                        o.try_borrow().ok().map(|b| {
                            String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                        })
                    })
                    .unwrap_or_default();
                return Err(PhpError::Error(format!(
                    "Cannot use object of type {name} as array"
                )));
            }
            match op {
                UnsetAa::Unset { obj, key } => {
                    self.call_method_sync(obj, b"offsetUnset", vec![key])?;
                }
                UnsetAa::Descend { obj, key, rest, keys } => {
                    let cname = deref_object(&obj)
                        .and_then(|o| {
                            o.try_borrow().ok().map(|b| {
                                String::from_utf8_lossy(b.class_name.as_bytes()).into_owned()
                            })
                        })
                        .unwrap_or_default();
                    let mut val =
                        self.call_method_sync(obj, b"offsetGet", vec![key])?.deref_clone();
                    if !matches!(val, Zval::Object(_)) {
                        // Unsetting through a by-value offsetGet result would
                        // mutate a temporary — Zend's notice, nothing removed.
                        self.diags.push(Diag::Notice(format!(
                            "Indirect modification of overloaded element of {cname} has no effect"
                        )));
                        let line = self.cur_line(top);
                        self.flush_diags(line)?;
                        break;
                    }
                    let fs =
                        FieldScope { classes: &self.classes, scope: self.frames[top].class };
                    let mut aa2 = None;
                    field_unset(&mut val, &rest, &mut keys.into_iter(), fs, &mut aa2)?;
                    pending = aa2;
                }
            }
        }
        Ok(())
    }
}
