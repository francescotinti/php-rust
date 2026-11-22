//! S-194 package A, step A3: compare-and-branch without an intermediate value.
//! `CmpJmpSC` has had this since S-171 (`long_cmp_i64`); here the same cure for
//! the slot/slot and stack forms, for a condition that is already a bool, and
//! for the fused loop back-edge. Every miss runs the EXACT former body.

use super::*;

impl<'m> Vm<'m> {
    /// `Op::CmpJmpSS`: two `Long` slots decide in place; anything else takes
    /// the former handler body out of line.
    #[inline(always)]
    pub(super) fn cmp_jmp_ss(
        &mut self,
        top: usize,
        func: &crate::bytecode::Func,
        op: BinOp,
        l: u16,
        r: u16,
    ) -> Result<bool, PhpError> {
        let fr = &self.frames[top];
        if let (Zval::Long(a), Zval::Long(b)) = (&fr.slots[l as usize], &fr.slots[r as usize]) {
            if let Some(j) = long_cmp_i64(op, *a, *b) {
                return Ok(j);
            }
        }
        self.cmp_jmp_ss_slow(top, func, op, l, r)
    }

    #[inline(never)]
    fn cmp_jmp_ss_slow(
        &mut self,
        top: usize,
        func: &crate::bytecode::Func,
        op: BinOp,
        l: u16,
        r: u16,
    ) -> Result<bool, PhpError> {
        let res = 'r: {
            {
                let fr = &self.frames[top];
                let lv = &fr.slots[l as usize];
                let rv = &fr.slots[r as usize];
                if !matches!(lv, Zval::Undef | Zval::Ref(_))
                    && !matches!(rv, Zval::Undef | Zval::Ref(_))
                {
                    if let Some(v) = binary_fast(op, lv, rv) {
                        break 'r v;
                    }
                }
            }
            let lhs = self.reg_load_slot(top, func, l);
            let rhs = self.reg_load_slot(top, func, r);
            self.binary_value_ab(op, lhs, rhs)?
        };
        Ok(convert::to_bool(&res, &mut self.diags))
    }

    /// `Op::CmpJmp`: both operands on the stack. Two `Long`s are compared
    /// and forgotten (a `Long` owns nothing); anything else is `binary_value`.
    #[inline(always)]
    pub(super) fn cmp_jmp_stack(&mut self, top: usize, op: BinOp) -> Result<bool, PhpError> {
        let st = &mut self.frames[top].stack;
        if let [.., Zval::Long(a), Zval::Long(b)] = st.as_slice() {
            if let Some(j) = long_cmp_i64(op, *a, *b) {
                std::mem::forget(st.pop());
                std::mem::forget(st.pop());
                return Ok(j);
            }
        }
        let r = self.binary_value(top, op)?;
        Ok(convert::to_bool(&r, &mut self.diags))
    }

    /// `Op::CmpJmpConst`: one operand on the stack, one literal.
    #[inline(always)]
    pub(super) fn cmp_jmp_const(
        &mut self,
        top: usize,
        func: &crate::bytecode::Func,
        op: BinOp,
        cidx: u32,
        const_lhs: bool,
    ) -> Result<bool, PhpError> {
        let st = &mut self.frames[top].stack;
        if let (Some(Zval::Long(x)), crate::bytecode::Const::Int(c)) = (st.last(), &func.consts[cidx as usize]) {
            let (a, b) = if const_lhs { (*c, *x) } else { (*x, *c) };
            if let Some(j) = long_cmp_i64(op, a, b) {
                std::mem::forget(st.pop());
                return Ok(j);
            }
        }
        let x = self.frames[top].stack.pop().expect("CmpJmpConst operand");
        let c = func.consts[cidx as usize].to_zval();
        let (lhs, rhs) = if const_lhs { (c, x) } else { (x, c) };
        let r = self.binary_value_ab(op, lhs, rhs)?;
        Ok(convert::to_bool(&r, &mut self.diags))
    }

    /// Fused loop back-edge of `Op::IncDecSlotJmp` (S-174): when the jump
    /// target is the loop-head compare and its fast path is in domain, the
    /// compare is decided here and `ip` goes straight to its outcome — one
    /// dispatch less per iteration. The `CmpJmpSC` arm is the S-174 body
    /// verbatim; the `CmpJmpSS` arm is the same for a limit held in a slot.
    /// Any miss leaves `ip` at the compare, which then runs as usual.
    #[cfg(not(feature = "op-census"))]
    #[inline(always)]
    pub(super) fn backedge_cmp(&mut self, top: usize, func: &crate::bytecode::Func, a: usize) {
        let fr = &mut self.frames[top];
        let (jump, when, target) = match func.ops.get(a) {
            Some(Op::CmpJmpSC { op, slot, cidx, addr, when }) => {
                match (&func.consts[*cidx as usize], &fr.slots[*slot as usize]) {
                    (crate::bytecode::Const::Int(c), Zval::Long(l)) => (long_cmp_i64(*op, *l, *c), *when, *addr as usize),
                    _ => return,
                }
            }
            Some(Op::CmpJmpSS { op, l, r, addr, when }) => match (&fr.slots[*l as usize], &fr.slots[*r as usize]) {
                (Zval::Long(x), Zval::Long(y)) => (long_cmp_i64(*op, *x, *y), *when, *addr as usize),
                _ => return,
            },
            _ => return,
        };
        if let Some(j) = jump {
            fr.ip = if j == when { target } else { a + 1 };
        }
    }

    /// `Op::JumpIfFalse` / `Op::JumpIfTrue`: a condition that is already a
    /// bool is read as it is (`to_bool` on a `Bool` emits no diagnostic).
    #[inline(always)]
    pub(super) fn pop_cond(&mut self, top: usize) -> bool {
        match self.frames[top].stack.pop().expect("jump condition") {
            Zval::Bool(b) => b,
            c => convert::to_bool(&c, &mut self.diags),
        }
    }
}
