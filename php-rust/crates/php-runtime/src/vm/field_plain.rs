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
use super::arrays::coerce_key_silent;
use super::oop::{resolve_prop_access, PropAccess};
use super::Vm;
use crate::bytecode::{FieldBase, FieldStep};
use crate::hir::ClassId;
use php_types::{convert, Zval};

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
}
