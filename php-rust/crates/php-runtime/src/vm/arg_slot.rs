//! S-194 C4: a plain-variable argument of a dynamic call no longer promotes the variable to a
//! reference. [`Op::PushRefArg`] pushes the reference only when the slot already holds one; a
//! defined plain value is sent as a step-less [`Zval::ArgPlace`] naming the slot, which every
//! dispatch funnel already resolves against the callee's by-reference mask (a by-reference
//! parameter promotes the slot THEN, a by-value one reads it). The descriptor of a slot is the
//! same for every function, so the VM keeps one per slot index: a send is a reference-count bump,
//! not an allocation. Its name is empty: the "Undefined variable" warning belongs to the
//! undefined send, which keeps its own named descriptor (`push_undef_arg_place`).
//! Criterion: wp194-harness/s194-criterio-C.md p.25.
use super::*;

impl<'m> Vm<'m> {
    /// The shared step-less place of local `slot`.
    #[inline]
    pub(super) fn slot_place(&mut self, slot: Slot) -> Zval {
        match self.slot_places.get(slot as usize) {
            Some(p) => Zval::ArgPlace(Rc::clone(p)),
            None => self.slot_place_grow(slot),
        }
    }

    #[cold]
    #[inline(never)]
    fn slot_place_grow(&mut self, slot: Slot) -> Zval {
        while self.slot_places.len() <= slot as usize {
            let base = ArgPlaceBase::Local(self.slot_places.len() as u32);
            self.slot_places.push(Rc::new(ArgPlace { base, steps: Default::default(), keys: Default::default(), name: Default::default() }));
        }
        Zval::ArgPlace(Rc::clone(&self.slot_places[slot as usize]))
    }

    /// A callable object (`$o(...)`) or a `[$o, 'm']` callable reaches `dispatch_instance_call`
    /// without passing through `method_call`: resolve the deferred places here, as it does.
    pub(super) fn settle_instance_places(&mut self, top: usize, this: &Zval, method: &[u8], args: &mut [Zval]) -> Result<(), PhpError> {
        if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
            let callee = self.instance_arg_ref_target(top, this, method);
            self.materialize_arg_places(top, args, callee)?;
            let line = self.cur_line(top);
            self.flush_diags(line)?;
        }
        Ok(())
    }

    /// The value a by-value parameter reads from a step-less place of a DEFINED local; `None`
    /// for any other place (the general reader warns, walks or fails).
    #[inline]
    pub(super) fn slot_place_value(&self, top: usize, p: &ArgPlace) -> Option<Zval> {
        let ArgPlaceBase::Local(s) = p.base else { return None };
        let v = &self.frames[top].slots[s as usize];
        (p.steps.is_empty() && !matches!(v, Zval::Undef)).then(|| arrays::read_slot(v))
    }

    /// Fast admissions whose callee has no by-reference and no variadic parameter: replace, in
    /// place, the step-less places among the top `n` operands by the value of their slot. An
    /// undefined one stays (its read warns, on the general path). True = a place is left.
    #[inline]
    pub(super) fn settle_slot_places(&mut self, top: usize, n: usize) -> bool {
        let f = &mut self.frames[top];
        let len = f.stack.len();
        let mut left = false;
        for i in len - n..len {
            let Zval::ArgPlace(p) = &f.stack[i] else { continue };
            match p.base {
                ArgPlaceBase::Local(s) if p.steps.is_empty() && !matches!(f.slots[s as usize], Zval::Undef) => f.stack[i] = arrays::read_slot(&f.slots[s as usize]),
                _ => left = true,
            }
        }
        left
    }
}
