//! S-195 — the value of a plain-variable argument of a dynamic call is the one it has
//! at the SEND (criterion wp195-harness/s195-criterio-invio.md). `push_dyn_args` sends a
//! variable as a deferred place read at the bind; that is exact unless a LATER argument
//! of the same call writes the variable (`$o->m($i, $i = 5)` must pass 1, not 5). After
//! the argument list is emitted, each such send whose tail of ops touches the variable
//! is rewritten in place (same address) into the step-less `PushArgPlace`, which the VM
//! turns into a place carrying a snapshot (`ArgPlaceStep::Snap`).
//!
//! `touches` is an OPEN list on purpose: an op missing from it leaves that shape with
//! today's bind-time read (the old divergence), it cannot introduce a new behaviour.
use std::rc::Rc;

use super::FnCompiler;
use crate::bytecode::{Addr, Const, DimBase, FieldBase, Op};
use crate::hir::Slot;

/// Can `op` write local `s`, or take a reference / deferred place to it?
fn touches(op: &Op, s: Slot) -> bool {
    let d = |b: &DimBase| matches!(b, DimBase::Local(x) if *x == s);
    let f = |b: &FieldBase| matches!(b, FieldBase::Local(x) if *x == s);
    match op {
        Op::StoreSlot(x) | Op::ConcatAssignSlot(x) | Op::PushRef(x) | Op::PushRefArg(x) | Op::IterInitRef(x) => *x == s,
        Op::IncDecSlot { slot, .. }
        | Op::StaticAlias { slot, .. }
        | Op::CallBuiltinRef { slot, .. }
        | Op::CallBuiltinRefSpread { slot, .. }
        | Op::CallHostBuiltinRef { slot, .. } => *slot == s,
        Op::AssignPath { base, .. } | Op::AssignOpPath { base, .. } | Op::IncDecPath { base, .. } | Op::UnsetPath { base, .. } => d(base),
        Op::BindRef { target, source } => d(target) || d(source),
        Op::FieldAssign { base, .. }
        | Op::FieldAssignOp { base, .. }
        | Op::FieldIncDec { base, .. }
        | Op::FieldUnset { base, .. }
        | Op::MakeRef { base, .. }
        | Op::PushArgPlace { base, .. }
        | Op::BindRefTo { base, .. }
        | Op::BindRefToChecked { base, .. } => f(base),
        Op::MakeClosure { captures, .. } => captures.iter().any(|c| c.by_ref && c.src == s),
        Op::CallHostBuiltinOut { out_slot, out_slot2, .. } => *out_slot == Some(s) || *out_slot2 == Some(s),
        Op::CallHostBuiltinScanf { out_slots, .. } => out_slots.iter().any(|o| *o == Some(s)),
        Op::Eval | Op::Include { .. } | Op::StoreVarDyn | Op::BindGlobalDyn | Op::GlobalsDynAssign => true,
        _ => false,
    }
}

impl<'a> FnCompiler<'a> {
    /// `sent`: address and slot of every plain-variable send of ONE argument list, in
    /// order, called once the whole list is emitted. The sends of the list itself are
    /// not hazards for one another (they bind together); anything else that touches the
    /// variable after its send is.
    pub(super) fn snapshot_hazard_sends(&mut self, sent: &[(Addr, Slot)]) {
        for (k, &(at, slot)) in sent.iter().enumerate() {
            let from = at as usize + 1;
            let hazard = self.ops[from..].iter().enumerate().any(|(j, op)| {
                touches(op, slot) && !sent[k + 1..].iter().any(|&(b, _)| b as usize == from + j)
            });
            if hazard {
                let name = self.slot_names.get(slot as usize).cloned().unwrap_or_default();
                let name = self.konst(Const::Str(php_types::PhpStr::new(name)));
                self.ops[at as usize] = Op::PushArgPlace { base: FieldBase::Local(slot), steps: Rc::from(Vec::new()), name };
            }
        }
    }
}
