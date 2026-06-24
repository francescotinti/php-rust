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
    if matches!(key, Zval::Array(_) | Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_)) {
        return None;
    }
    let i = convert::to_long_cast(key, &mut Diags::new());
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
pub(super) fn field_write(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
    value: Zval,
    diags: &mut Diags,
) -> Result<(), PhpError> {
    if let Zval::Ref(cell) = target {
        let inner = &mut *cell.borrow_mut();
        return field_write(inner, steps, keys, value, diags);
    }
    let Some((first, rest)) = steps.split_first() else {
        *target = value;
        return Ok(());
    };
    match first {
        FieldStep::Prop(name) => {
            match target {
                Zval::Object(o) => {
                    let mut obj = o.borrow_mut();
                    if obj.info.is_enum_case {
                        let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
                        let prop = String::from_utf8_lossy(name).into_owned();
                        return Err(PhpError::Error(if obj.props.contains(name) {
                            format!("Cannot modify readonly property {cls}::${prop}")
                        } else {
                            format!("Cannot create dynamic property {cls}::${prop}")
                        }));
                    }
                    if rest.is_empty() {
                        obj.props.set(name, value);
                    } else {
                        if !obj.props.contains(name) {
                            obj.props.set(name, Zval::Array(Rc::new(PhpArray::new())));
                        }
                        let child = obj.props.get_mut(name).expect("property just inserted");
                        field_write(child, rest, keys, value, diags)?;
                    }
                }
                other => {
                    diags.push(Diag::Warning(format!(
                        "Attempt to assign property \"{}\" on {}",
                        String::from_utf8_lossy(name),
                        other.error_type_name()
                    )));
                }
            }
            Ok(())
        }
        FieldStep::Index => {
            let key = keys.next().expect("field index key");
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if rest.is_empty() {
                // Overwrite a plain element, but write *through* an existing
                // reference element (the recursive call derefs at its top).
                match arr.get_mut(&k) {
                    Some(child) => field_write(child, rest, keys, value, diags)?,
                    None => arr.insert(k, value),
                }
            } else {
                if !arr.contains_key(&k) {
                    arr.insert(k.clone(), Zval::Array(Rc::new(PhpArray::new())));
                }
                let child = arr.get_mut(&k).expect("key just inserted");
                field_write(child, rest, keys, value, diags)?;
            }
            Ok(())
        }
        FieldStep::Append => {
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let occupied =
                || PhpError::Error("Cannot add element to the array as the next element is already occupied".to_string());
            if rest.is_empty() {
                arr.append(value).map_err(|_| occupied())?;
            } else {
                let mut child = Zval::Array(Rc::new(PhpArray::new()));
                field_write(&mut child, rest, keys, value, diags)?;
                arr.append(child).map_err(|_| occupied())?;
            }
            Ok(())
        }
    }
}

/// Silently read a mixed field path's value (OOP-2c), `None` if any level is
/// absent — backs compound/inc-dec (missing → NULL) and `isset`/field tests.
pub(super) fn field_get(cell: &Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
        return field_get(&rc.borrow(), steps, keys);
    }
    match steps.split_first() {
        None => match cell {
            Zval::Undef => None,
            other => Some(other.deref_clone()),
        },
        Some((first, rest)) => match first {
            FieldStep::Prop(name) => match cell {
                Zval::Object(o) => {
                    let obj = o.borrow();
                    match obj.props.get(name) {
                        Some(v) => field_get(v, rest, keys),
                        None => None,
                    }
                }
                _ => None,
            },
            FieldStep::Index => {
                let key = keys.next()?;
                match cell {
                    Zval::Array(a) => {
                        let k = coerce_key_silent(&key)?;
                        a.get(&k).and_then(|c| field_get(c, rest, keys))
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

/// Remove a mixed field path's leaf (OOP-2c). A missing intermediate level is a
/// silent no-op; arrays copy-on-write, objects mutate in place.
pub(super) fn field_unset(target: &mut Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>) {
    if let Zval::Ref(rc) = target {
        field_unset(&mut rc.borrow_mut(), steps, keys);
        return;
    }
    let Some((first, rest)) = steps.split_first() else {
        return;
    };
    match first {
        FieldStep::Prop(name) => {
            if let Zval::Object(o) = target {
                if rest.is_empty() {
                    o.borrow_mut().props.remove(name);
                } else if let Some(child) = o.borrow_mut().props.get_mut(name) {
                    field_unset(child, rest, keys);
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
                        field_unset(child, rest, keys);
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
pub(super) fn read_dim_warn(base: &Zval, key: &Zval, diags: &mut Diags) -> Zval {
    match base {
        Zval::Array(a) => match coerce_key_silent(key) {
            Some(k) => match a.get(&k) {
                Some(v) => v.deref_clone(),
                None => {
                    let msg = match &k {
                        Key::Int(i) => format!("Undefined array key {i}"),
                        Key::Str(s) => {
                            format!("Undefined array key \"{}\"", String::from_utf8_lossy(s.as_bytes()))
                        }
                    };
                    diags.push(Diag::Warning(msg));
                    Zval::Null
                }
            },
            None => Zval::Null,
        },
        Zval::Ref(rc) => read_dim_warn(&rc.borrow(), key, diags),
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
        };
        path_apply(cell, &keys, last, &mut self.diags)
    }

    /// Write `value` through a mixed field path. The base cell borrows
    /// `self.frames` and `&mut self.diags` a disjoint field, so the two coexist
    /// (the same split the array `path_op` relies on).
    pub(super) fn field_set(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
    ) -> Result<(), PhpError> {
        let cell = match base {
            FieldBase::Local(s) => &mut self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &mut self.frames[0].slots[s as usize],
            FieldBase::This => self.frames[top].this.as_mut().ok_or_else(|| {
                PhpError::Error("Using $this when not in object context".to_string())
            })?,
        };
        field_write(cell, steps, &mut keys.into_iter(), value, &mut self.diags)
    }
}
