//! S-194 P1: one-pass answer to isset() / empty() of a mixed property/index path
//! (Op::FieldIsset, Op::FieldEmpty) when no protocol can be involved.
//!
//! The general machinery asks, in turn, the magic probe, the ArrayAccess leaf, the ArrayAccess
//! walk, the lazy root and the plain reader: up to six walks of the same path, the first of which
//! clones the value at every step. This walk borrows its way down once and answers only when the
//! verdict needs none of those: every Prop step lands on an object that is neither lazy nor a
//! proxy, of a class without property hooks, and the property is present and accessible (then
//! __isset / __get do not apply) or the class has neither __isset nor __get; every Index step
//! lands on an array, or on a value that has no elements. Anything else is None and the caller
//! goes on exactly as before: the walk only reads, so giving up half way leaves nothing behind.
//! Criterion: wp194-harness/s194-criterio-P.md p.3.
//!
//! S-194 P2 (same criterion, p.7): isset() / empty() / assignment with ONE key on an array
//! variable, without the key vector, the base clone and the general walkers. Both functions
//! decide before they touch the operand stack; false = the arm goes on as before.
//!
//! S-194 P3 (p.10): the write caches admit a private property of the scope's own class. A cache
//! hit writes by slot index; [`write_slot`] is that write, and [`Vm::write_stale_slot`] the
//! fallback when the slot vanished between the hit's check and the write (a typed coercion ran
//! a `__toString` that unset the property): it addresses the storage key the scope resolves,
//! which for a private is not the bare name.
//!
//! S-194 P4 (p.13): a readonly property is cached too (`PropIc::RO`) for the scopes that may
//! initialise it. The class facts are proved at the fill; the hit asks the instance what only
//! it knows ([`Vm::ro_uninit`]: the slot is uninitialised and no clone grant is pending) and
//! records the initialisation after the typed coercion succeeded ([`Vm::ro_mark_slot`]).
use super::arrays::coerce_key_silent;
use super::oop::{resolve_prop_access, PropAccess};
use super::*;

/// The slot write of a property cache hit: [`write_property_at`] without its by-name fallback
/// (`Err` hands the value back when the slot is gone).
#[inline]
pub(super) fn write_slot(o: &Rc<RefCell<Object>>, i: u32, value: Zval) -> Result<Option<Zval>, Zval> {
    // A slot holding a reference is written THROUGH, exactly like write_property_at.
    let cell = match o.borrow().props.get_slot(i) {
        Some(Zval::Ref(cell)) => Some(Rc::clone(cell)),
        _ => None,
    };
    if let Some(cell) = cell {
        return Ok(Some(std::mem::replace(&mut *cell.borrow_mut(), value)));
    }
    o.borrow_mut().props.replace_slot(i, value)
}

/// Record a readonly initialisation at `key` (`clone`: it consumes the grant `clone` gave).
pub(super) fn ro_mark_key(o: &Rc<RefCell<Object>>, key: &[u8], clone: bool) {
    let mut ob = o.borrow_mut();
    if clone {
        ob.consume_clone_writable(key);
    }
    ob.mark_readonly_init(key);
}

impl<'m> Vm<'m> {
    /// A cached readonly write may proceed: declared slot `slot` of `b` is uninitialised, was
    /// never initialised and holds no clone grant. Anything else is the general path's.
    #[inline(never)]
    pub(super) fn ro_uninit(&self, b: &Object, slot: u32) -> bool {
        matches!(b.props.get_slot(slot), Some(Zval::Undef))
            && self.classes[b.class_id as usize]
                .props_layout
                .key_at(slot)
                .is_some_and(|k| !b.is_readonly_init(k) && !b.readonly_clone_writable(k))
    }

    #[inline(never)]
    pub(super) fn ro_mark_slot(&self, o: &Rc<RefCell<Object>>, slot: u32) {
        let cid = o.borrow().class_id as usize;
        if let Some(k) = self.classes[cid].props_layout.key_at(slot) {
            o.borrow_mut().mark_readonly_init(k);
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn write_stale_slot(&self, recv: &Zval, name: &[u8], scope: Option<ClassId>, value: Zval) -> Result<Option<Zval>, PhpError> {
        let key: &[u8] = match object_class_id(recv).map(|cid| resolve_prop_access(&self.classes, cid, name, scope)) {
            Some(PropAccess::Slot { key, .. }) => &key[..],
            _ => name,
        };
        write_property(recv, key, value)
    }

    /// Some(verdict) = the Bool the op pushes (isset: set; empty: empty); None = not decided here.
    pub(super) fn field_is_plain(&self, base: FieldBase, top: usize, steps: &[FieldStep], keys: &[Zval], empty_mode: bool) -> Option<bool> {
        let cell = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize)?,
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize)?,
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize)?,
            FieldBase::This => self.frames[top].this.as_ref()?,
        };
        self.plain_walk(cell, steps, keys, self.frames[top].class, empty_mode)
    }

    fn plain_walk(&self, cell: &Zval, steps: &[FieldStep], keys: &[Zval], scope: Option<ClassId>, empty_mode: bool) -> Option<bool> {
        if let Zval::Ref(rc) = cell {
            let b = rc.try_borrow().ok()?;
            return self.plain_walk(&b, steps, keys, scope, empty_mode);
        }
        // A missing place: isset is false, empty is true.
        let missing = Some(empty_mode);
        let Some((first, rest)) = steps.split_first() else {
            return match cell {
                Zval::Null | Zval::Undef => missing,
                Zval::Bool(_) | Zval::Long(_) | Zval::Str(_) | Zval::Array(_) => Some(!empty_mode || !convert::is_true_silent(cell)),
                // empty() of an object or a float converts with diagnostics: not here.
                _ if empty_mode => None,
                _ => Some(true),
            };
        };
        match first {
            FieldStep::Prop(name) => {
                let Zval::Object(o) = cell else { return missing };
                let obj = o.try_borrow().ok()?;
                let cid = obj.class_id as usize;
                if obj.lazy.is_some() || obj.proxy_instance.is_some() || self.classes[cid].has_prop_hooks {
                    return None;
                }
                // Absent, unset or inaccessible: the protocol decides, unless the class has none.
                let no_magic = || (self.magic_method(cid, 2).is_none() && self.magic_method(cid, 4).is_none()).then_some(empty_mode);
                let key: &[u8] = match resolve_prop_access(&self.classes, cid, name, scope) {
                    PropAccess::Slot { key, .. } => &key[..],
                    PropAccess::Dynamic => &name[..],
                    PropAccess::Denied { .. } => return no_magic(),
                };
                match obj.props.get(key) {
                    None | Some(Zval::Undef) => no_magic(),
                    Some(v) => self.plain_walk(v, rest, keys, scope, empty_mode),
                }
            }
            FieldStep::Index => {
                let (k, krest) = keys.split_first()?;
                match cell {
                    Zval::Array(a) => match coerce_key_silent(k).and_then(|key| a.get(&key)) {
                        Some(v) => self.plain_walk(v, rest, krest, scope, empty_mode),
                        None => missing,
                    },
                    // String offsets and ArrayAccess have their own rules.
                    Zval::Str(_) | Zval::Object(_) => None,
                    _ => missing,
                }
            }
            _ => None,
        }
    }

    /// `[k] -> [bool]` for Op::IssetPath (`empty_mode` false) and Op::EmptyPath (true) with one key:
    /// the base is an array, directly or behind a reference, and the key an int or a string.
    #[inline(never)]
    pub(super) fn dim1_is_fast(&mut self, base: DimBase, top: usize, empty_mode: bool) -> bool {
        fn leaf(v: Option<&Zval>, empty_mode: bool) -> Option<bool> {
            match v {
                None | Some(Zval::Undef) => Some(empty_mode),
                Some(Zval::Ref(rc)) => leaf(Some(&*rc.try_borrow().ok()?), empty_mode),
                Some(v) if empty_mode => Some(!convert::is_true_silent(v)),
                Some(v) => Some(!matches!(v, Zval::Null)),
            }
        }
        let verdict = {
            let key = match self.frames[top].stack.last() {
                Some(Zval::Long(i)) => Key::Int(*i),
                Some(Zval::Str(s)) => Key::from_zstr(s),
                _ => return false,
            };
            let cell = match base {
                DimBase::Local(s) => &self.frames[top].slots[s as usize],
                DimBase::Global(s) => &self.frames[0].slots[s as usize],
                DimBase::Superglobal(i) => &self.superglobals[i as usize],
            };
            let v = match cell {
                Zval::Array(a) => leaf(a.get(&key), empty_mode),
                Zval::Ref(rc) => match rc.try_borrow().as_deref() {
                    Ok(Zval::Array(a)) => leaf(a.get(&key), empty_mode),
                    _ => return false,
                },
                _ => return false,
            };
            let Some(v) = v else { return false };
            v
        };
        *self.frames[top].stack.last_mut().expect("isset key") = Zval::Bool(verdict);
        true
    }

    /// `[k, v] -> [v]` for Op::AssignPath with one key, no append: the base slot already holds an
    /// array and the key is an int or a string. The write is AP1's (S-135): make_mut, set, note the
    /// displaced value.
    #[inline(never)]
    pub(super) fn dim1_assign_fast(&mut self, base: DimBase, top: usize) -> bool {
        let st = &self.frames[top].stack;
        let n = st.len();
        if n < 2 || !matches!(st[n - 2], Zval::Long(_) | Zval::Str(_)) {
            return false;
        }
        let is_arr = matches!(
            match base {
                DimBase::Local(s) => &self.frames[top].slots[s as usize],
                DimBase::Global(s) => &self.frames[0].slots[s as usize],
                DimBase::Superglobal(i) => &self.superglobals[i as usize],
            },
            Zval::Array(_)
        );
        if !is_arr {
            return false;
        }
        let st = &mut self.frames[top].stack;
        let value = st.pop().expect("AssignPath value");
        let vc = value.clone();
        let key = match std::mem::replace(st.last_mut().expect("AssignPath key"), value) {
            Zval::Long(i) => Key::Int(i),
            Zval::Str(s) => Key::from_zstr(&s),
            _ => unreachable!("peeked key"),
        };
        let lw = {
            let cell = match base {
                DimBase::Local(s) => &mut self.frames[top].slots[s as usize],
                DimBase::Global(s) => &mut self.frames[0].slots[s as usize],
                DimBase::Superglobal(i) => &mut self.superglobals[i as usize],
            };
            let Zval::Array(rc) = cell else { unreachable!("peeked array") };
            Rc::make_mut(rc).set_returning_displaced(key, vc)
        };
        match lw {
            LeafWrite::Done(Some(d)) => self.gc_note(&d),
            LeafWrite::Done(None) => {}
            LeafWrite::Busy(cell, val) => {
                ap1_busy_note();
                if let Ok(mut g) = cell.try_borrow_mut() {
                    let d = std::mem::replace(&mut *g, val);
                    drop(g);
                    self.gc_note(&d);
                }
            }
        }
        true
    }

    /// S-194 P3 census (instrumentation builds only): why a property write missed the site cache.
    #[cfg(feature = "op-census")]
    pub(super) fn prop_set_miss_why(&self, target: &Zval, name: &[u8], cur: Option<ClassId>, ic: &crate::bytecode::PropIc) {
        let why = (|| {
            let Zval::Object(o) = target else { return 0 };
            let (cid, lazy, present) = {
                let b = o.borrow();
                (b.class_id as usize, b.lazy.is_some() || b.info.is_enum_case, b.props.contains(name))
            };
            if lazy {
                return 1;
            }
            if self.prop_hook(cid, name, true).is_some() || self.is_virtual_hooked(cid, name) {
                return 2;
            }
            if self.magic_method(cid, 3).is_some() {
                return 3;
            }
            match resolve_prop_access(&self.classes, cid, name, cur) {
                PropAccess::Denied { .. } => 4,
                PropAccess::Dynamic => 5 + present as usize,
                PropAccess::Slot { key, slot: None } => 7 + (&key[..] == name) as usize,
                PropAccess::Slot { .. } => {
                    if prop_readonly_decl(&self.classes, cid, name).is_some() {
                        9
                    } else if asym_write_error(&self.classes, cur, cid, name, "modify").is_some() {
                        10
                    } else if ic.get(crate::bytecode::PropIc::scope_key(cur)).is_some() {
                        11
                    } else {
                        12
                    }
                }
            }
        })();
        super::census::PROP_SET_WHY[why].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
