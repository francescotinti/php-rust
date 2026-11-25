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
use super::arrays::coerce_key_silent;
use super::oop::{resolve_prop_access, PropAccess};
use super::*;

impl<'m> Vm<'m> {
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
}
