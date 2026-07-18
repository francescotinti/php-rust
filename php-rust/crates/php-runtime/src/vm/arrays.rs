//! VM arrays logic, extracted from vm/mod.rs (no semantic change).
use super::*;

/// Silently follow `keys` from `cell` without auto-vivifying anything, returning
/// the leaf value if the whole path exists (an unset variable or a missing key
/// at any level yields `None`). Backs `isset` / `empty`. A reference is followed;
/// a string base supports a final byte-offset step.
pub(super) fn silent_get_path(cell: &Zval, keys: &[Zval]) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
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
pub(super) fn unset_into(cell: &mut Zval, keys: &[Zval]) {
    match keys.split_first() {
        None => *cell = Zval::Undef,
        Some((k, rest)) => {
            if let Zval::Ref(rc) = cell {
                let mut inner = rc.borrow_mut();
                unset_into(&mut inner, keys);
                return;
            }
            if let Zval::Array(rc) = cell {
                if let Some(key) = coerce_key_silent(k) {
                    let arr = Rc::make_mut(rc);
                    if rest.is_empty() {
                        arr.remove(&key);
                    } else if let Some(child) = arr.get_mut(&key) {
                        unset_into(child, rest);
                    }
                }
            }
        }
    }
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
) -> Result<(), PhpError> {
    if let Zval::Ref(cell) = target {
        // A reference-BIND (`$arr['k'] =& $x`) REPLACES a leaf slot that
        // already holds a reference — writing through it instead would nest
        // the new ref inside the old cell (ORM's ArrayHydrator resultPointers
        // built a self-referential cell that way: infinite deref).
        if rebind && steps.is_empty() {
            dropped.push(std::mem::replace(target, value));
            return Ok(());
        }
        let inner = &mut *cell.borrow_mut();
        return field_write(inner, steps, keys, fs, value, diags, dropped, aa, rebind);
    }
    let Some((first, rest)) = steps.split_first() else {
        // Leaf overwrite: hand the displaced value back for GC noting.
        dropped.push(std::mem::replace(target, value));
        return Ok(());
    };
    match first {
        FieldStep::Prop(_) | FieldStep::PropDyn => {
            let owned;
            let name: &[u8] = match first {
                FieldStep::Prop(n) => n,
                _ => {
                    owned = prop_dyn_name(keys, diags);
                    &owned
                }
            };
            match target {
                Zval::Object(o) => {
                    // Own a handle so the leaf-defer branch can move the object into
                    // the pending op without holding a borrow of `target`.
                    let o = o.clone();
                    let (cid, is_enum, is_lazy) = {
                        let b = o.borrow();
                        (b.class_id as usize, b.info.is_enum_case, b.lazy.is_some())
                    };
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
                            || (!rebind
                                && (fs.prop_hooked(cid, name) || fs.prop_typed(cid, name)))
                            || is_lazy)
                    {
                        *aa = Some(AaOp::MagicSet(AaMagicSet {
                            obj: Zval::Object(o),
                            name: name.to_vec(),
                            value,
                        }));
                        return Ok(());
                    }
                    // An intermediate step whose raw slot is absent or
                    // inaccessible needs the VM (overloaded-property protocol /
                    // no-autoviv semantics) — defer the remaining path. Lazy
                    // wrappers keep the legacy raw walk (their realization has
                    // its own machinery); enum cases fall through to their
                    // dedicated immutability error below.
                    if !rest.is_empty() && !is_enum && !is_lazy {
                        let denied = fs.prop_key_read(cid, name).is_none();
                        let absent =
                            !o.borrow().props.contains(fs.prop_key(cid, name).as_ref());
                        if denied || absent {
                            *aa = Some(AaOp::MagicDescend(AaMagicDescend {
                                obj: Zval::Object(o),
                                name: name.to_vec(),
                                rest: rest.to_vec(),
                                keys: keys.collect(),
                                value,
                            }));
                            return Ok(());
                        }
                    }
                    let mut obj = o.borrow_mut();
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
                    } else {
                        if !obj.props.contains(key) {
                            obj.props.set(key, Zval::Array(Rc::new(PhpArray::new())));
                        }
                        let child = obj.props.get_mut(key).expect("property just inserted");
                        field_write(child, rest, keys, fs, value, diags, dropped, aa, rebind)?;
                    }
                }
                other => {
                    diags.push(Diag::Warning(format!(
                        "Attempt to assign property \"{}\" on {}",
                        String::from_utf8_lossy(name),
                        other.type_name_for_error()
                    )));
                }
            }
            Ok(())
        }
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
                return string_offset_write(target, &key, &value, diags).map(|_| ());
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
                return Ok(());
            }
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let k = coerce_key_diag(&key, diags)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if rest.is_empty() {
                // Overwrite a plain element, but write *through* an existing
                // reference element (the recursive call derefs at its top).
                match arr.get_mut(&k) {
                    Some(child) => field_write(child, rest, keys, fs, value, diags, dropped, aa, rebind)?,
                    None => arr.insert(k, value),
                }
            } else {
                if !arr.contains_key(&k) {
                    arr.insert(k.clone(), Zval::Array(Rc::new(PhpArray::new())));
                }
                let child = arr.get_mut(&k).expect("key just inserted");
                field_write(child, rest, keys, fs, value, diags, dropped, aa, rebind)?;
            }
            Ok(())
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
                    return Ok(());
                }
                // `$o->c[]->x = v` — appending then drilling has no PHP
                // equivalent through ArrayAccess; keep the object-as-array error.
                let Zval::Object(o) = target else { unreachable!() };
                return Err(PhpError::Error(format!(
                    "Cannot use object of type {} as array",
                    String::from_utf8_lossy(o.borrow().class_name.as_bytes())
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
                let mut child = Zval::Array(Rc::new(PhpArray::new()));
                field_write(&mut child, rest, keys, fs, value, diags, dropped, aa, rebind)?;
                arr.append(child).map_err(|_| occupied())?;
            }
            Ok(())
        }
    }
}

pub(super) fn field_get(cell: &Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>, fs: FieldScope) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
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

pub(super) fn field_unset(target: &mut Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>, fs: FieldScope) {
    if let Zval::Ref(rc) = target {
        field_unset(&mut rc.borrow_mut(), steps, keys, fs);
        return;
    }
    let Some((first, rest)) = steps.split_first() else {
        return;
    };
    match first {
        FieldStep::Prop(_) | FieldStep::PropDyn => {
            let owned;
            let name: &[u8] = match first {
                FieldStep::Prop(n) => n,
                _ => {
                    owned = prop_dyn_name(keys, &mut Diags::new());
                    &owned
                }
            };
            if let Zval::Object(o) = target {
                let key = fs.prop_key(o.borrow().class_id as usize, name);
                if rest.is_empty() {
                    // A declared TYPED property returns to *uninitialized* on
                    // unset (mirrors Op::PropUnset; doctrine unsets via a
                    // bound closure with a dynamic name).
                    let typed = {
                        let ob = o.borrow();
                        ob.info.type_of(key.as_ref()).is_some() || ob.info.type_of(name).is_some()
                    };
                    if typed {
                        o.borrow_mut().props.set(key.as_ref(), Zval::Undef);
                    } else {
                        o.borrow_mut().props.remove(key.as_ref());
                    }
                } else if let Some(child) = o.borrow_mut().props.get_mut(key.as_ref()) {
                    field_unset(child, rest, keys, fs);
                }
            }
        }
        FieldStep::Index => {
            let Some(key) = keys.next() else { return };
            if let Zval::Array(rc) = target {
                if let Some(k) = coerce_key_silent(&key) {
                    let arr = Rc::make_mut(rc);
                    if rest.is_empty() {
                        arr.remove(&k);
                    } else if let Some(child) = arr.get_mut(&k) {
                        field_unset(child, rest, keys, fs);
                    }
                }
            }
        }
        FieldStep::Append => {}
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
        return coerce_key_diag(&c.borrow(), diags);
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
        Zval::Ref(c) => coerce_key_silent(&c.borrow()),
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
        Zval::Array(a) => a.iter().map(|(k, v)| (key_to_zval(k), v.clone())).collect(),
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
/// $a[5]`). String-offset and other bases delegate to the silent [`read_dim`]
/// (no failing parity case needs the "Uninitialized string offset" warning yet).
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
        // A warning-ful READ of a string offset with a non-integral string or
        // container key is PHP 8's TypeError (isset/`??` stay silent-false).
        Zval::Str(_) => {
            let kd = key.deref_clone();
            match &kd {
                Zval::Str(k)
                    if std::str::from_utf8(k.as_bytes())
                        .ok()
                        .and_then(|t| t.trim().parse::<i64>().ok())
                        .is_none() =>
                {
                    Err(PhpError::TypeError(
                        "Cannot access offset of type string on string".to_string(),
                    ))
                }
                Zval::Array(_) | Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_) => {
                    Err(PhpError::TypeError(format!(
                        "Cannot access offset of type {} on string",
                        kd.type_name_for_error()
                    )))
                }
                _ => Ok(read_dim(base, key)),
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
        // Elements displaced by the write (e.g. `$a[0] = new X` overwriting the
        // old `$a[0]`) are collected here, then noted as possible GC roots once
        // the borrow of the base cell ends.
        let mut dropped = Vec::new();
        let mut aa = None;
        let result = path_apply(cell, &keys, last, &mut self.diags, &mut dropped, &mut aa);
        for d in &dropped {
            self.gc_note(d);
        }
        let mut result = result?;
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
                    .map(|o| String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned())
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
                        .map(|o| String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned())
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
                    let mut dropped2 = Vec::new();
                    let mut aa2 = None;
                    let r = path_apply(&mut val, &rest, *last, &mut self.diags, &mut dropped2, &mut aa2);
                    for d in &dropped2 {
                        self.gc_note(d);
                    }
                    r?;
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
        self.field_set_mode(base, top, steps, keys, value, false)
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
        let r = field_write(cell, steps, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa, rebind);
        for d in &dropped {
            self.gc_note(d);
        }
        r?;
        self.drain_aa_pending(aa, top, rebind)
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
    ) -> Result<(), PhpError> {
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        let mut root_val = Zval::Ref(root);
        let mut dropped = Vec::new();
        let mut aa = None;
        let r = field_write(&mut root_val, steps, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa, rebind);
        for d in &dropped {
            self.gc_note(d);
        }
        r?;
        self.drain_aa_pending(aa, top, rebind)
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
                                "Attempt to assign property \"{}\" on {}",
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
                    field_write(&mut val, &rest, &mut keys.into_iter(), fs2, value, &mut self.diags, &mut dropped, &mut aa2, rebind)?;
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
                        "Attempt to assign property \"{}\" on null",
                        String::from_utf8_lossy(&next)
                    )));
                }
                {
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
                field_write(&mut objz, &steps, &mut keys.into_iter(), fs2, value, &mut self.diags, &mut dropped, &mut aa2, rebind)?;
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
                    .map(|o| String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned())
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
                        .map(|o| String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned())
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
                    field_write(&mut val, &rest, &mut keys.into_iter(), fs, value, &mut self.diags, &mut dropped, &mut aa2, rebind)?;
                    for d in &dropped {
                        self.gc_note(d);
                    }
                    pending = aa2;
                }
            }
        }
        Ok(())
    }
}
