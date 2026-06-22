//! Bytecode VM: the dispatch loop that executes a [`crate::bytecode::Module`]
//! (VM-migration Fase 4, vertical proof slice).
//!
//! This is the eventual replacement for [`crate::eval`]'s tree-walk. Where the
//! evaluator recurses over the HIR, the VM advances an explicit instruction
//! pointer over a flat [`crate::bytecode::Op`] stream — the property that makes
//! generators (park the `ip`) and non-structured control flow (`Jump`) ordinary
//! instead of requiring a coroutine + `unsafe` reborrow.
//!
//! # Status: proof slice
//!
//! Runs a single frame (the script body); calls, references, arrays, OOP and
//! generators are out of slice (the compiler refuses them, so the VM never sees
//! their opcodes). Value semantics are delegated to `php_types::ops` /
//! `php_types::convert`, exactly as the tree-walker does — the VM only moves
//! data and steers control flow.

use php_types::{convert, ops, Diags, PhpError, Zval};

use crate::bytecode::{Func, Module, Op};
use crate::hir::{BinOp, CastKind, UnOp};

/// The result of running a [`Module`]: the bytes written to stdout, the
/// diagnostics raised, and the fatal that stopped execution (if any).
#[derive(Debug, Default)]
pub struct VmOutcome {
    pub stdout: Vec<u8>,
    pub diags: Diags,
    pub fatal: Option<PhpError>,
}

/// Compile-and-run is the caller's job ([`crate::compile`]); this takes the
/// already-compiled module and executes its `main`.
pub fn run_module(module: &Module) -> VmOutcome {
    let mut out = VmOutcome::default();
    match exec_func(&module.main, &mut out.stdout, &mut out.diags) {
        Ok(_) => {}
        Err(e) => out.fatal = Some(e),
    }
    out
}

/// Execute one frame to completion, returning its result value (the operand the
/// terminating [`Op::Ret`] left). A frame owns its slot array (named locals) and
/// its operand stack; both would be saved verbatim to suspend a generator.
fn exec_func(func: &Func, stdout: &mut Vec<u8>, diags: &mut Diags) -> Result<Zval, PhpError> {
    let mut slots: Vec<Zval> = vec![Zval::Undef; func.n_slots as usize];
    let mut stack: Vec<Zval> = Vec::new();
    let mut ip: usize = 0;

    loop {
        let op = &func.ops[ip];
        match op {
            Op::PushConst(i) => stack.push(func.consts[*i as usize].to_zval()),
            Op::Pop => {
                stack.pop();
            }
            Op::Dup => {
                let v = stack.last().expect("Dup on empty stack").clone();
                stack.push(v);
            }
            Op::LoadSlot(s) => {
                // An unset local reads as NULL (the curated proof corpus is
                // warning-free; the "Undefined variable" notice is wired in with
                // the diagnostics-ordering work). A reference slot is followed.
                let v = match &slots[*s as usize] {
                    Zval::Undef => Zval::Null,
                    Zval::Ref(cell) => cell.borrow().clone(),
                    other => other.clone(),
                };
                stack.push(v);
            }
            Op::StoreSlot(s) => {
                let v = stack.pop().expect("StoreSlot on empty stack");
                store_slot(&mut slots[*s as usize], v);
            }
            Op::IncDecSlot { slot, inc, pre } => {
                let cell = &mut slots[*slot as usize];
                if matches!(cell, Zval::Undef) {
                    *cell = Zval::Null;
                }
                if *pre {
                    if *inc {
                        ops::increment(cell, diags)?;
                    } else {
                        ops::decrement(cell, diags)?;
                    }
                    stack.push(cell.clone());
                } else {
                    let old = cell.clone();
                    if *inc {
                        ops::increment(cell, diags)?;
                    } else {
                        ops::decrement(cell, diags)?;
                    }
                    stack.push(old);
                }
            }
            Op::Binary(b) => {
                let rhs = stack.pop().expect("Binary rhs");
                let lhs = stack.pop().expect("Binary lhs");
                stack.push(apply_binop(*b, &lhs, &rhs, diags)?);
            }
            Op::Unary(u) => {
                let a = stack.pop().expect("Unary operand");
                stack.push(apply_unop(*u, &a, diags)?);
            }
            Op::Cast(k) => {
                let a = stack.pop().expect("Cast operand");
                stack.push(apply_cast(*k, &a, diags));
            }
            Op::Jump(addr) => {
                ip = *addr as usize;
                continue;
            }
            Op::JumpIfFalse(addr) => {
                let c = stack.pop().expect("JumpIfFalse cond");
                if !convert::to_bool(&c, diags) {
                    ip = *addr as usize;
                    continue;
                }
            }
            Op::JumpIfTrue(addr) => {
                let c = stack.pop().expect("JumpIfTrue cond");
                if convert::to_bool(&c, diags) {
                    ip = *addr as usize;
                    continue;
                }
            }
            Op::Echo => {
                let v = stack.pop().expect("Echo operand");
                let s = convert::to_zstr(&v, diags);
                stdout.extend_from_slice(s.as_bytes());
            }
            Op::Print => {
                let v = stack.pop().expect("Print operand");
                let s = convert::to_zstr(&v, diags);
                stdout.extend_from_slice(s.as_bytes());
                stack.push(Zval::Long(1));
            }
            Op::Ret => {
                return Ok(stack.pop().unwrap_or(Zval::Null));
            }
            Op::Nop => {}
        }
        ip += 1;
    }
}

/// Write `v` into a local cell. A reference slot writes *through* its shared
/// cell (so aliases see the update); a plain slot is overwritten. This mirrors
/// the tree-walker's write-through discipline (`Zval::Ref`, D-R3).
fn store_slot(cell: &mut Zval, v: Zval) {
    if let Zval::Ref(r) = cell {
        *r.borrow_mut() = v;
    } else {
        *cell = v;
    }
}

fn apply_binop(op: BinOp, a: &Zval, b: &Zval, d: &mut Diags) -> Result<Zval, PhpError> {
    use BinOp::*;
    Ok(match op {
        Add => ops::add(a, b, d)?,
        Sub => ops::sub(a, b, d)?,
        Mul => ops::mul(a, b, d)?,
        Div => ops::div(a, b, d)?,
        Mod => ops::modulo(a, b, d)?,
        Pow => ops::pow(a, b, d)?,
        Concat => ops::concat(a, b, d)?,
        BitAnd => ops::bw_and(a, b, d)?,
        BitOr => ops::bw_or(a, b, d)?,
        BitXor => ops::bw_xor(a, b, d)?,
        Shl => ops::shl(a, b, d)?,
        Shr => ops::shr(a, b, d)?,
        Eq => Zval::Bool(ops::loose_eq(a, b)),
        NotEq => Zval::Bool(!ops::loose_eq(a, b)),
        Identical => Zval::Bool(ops::identical(a, b)),
        NotIdentical => Zval::Bool(!ops::identical(a, b)),
        Lt => Zval::Bool(ops::smaller(a, b)),
        Le => Zval::Bool(ops::smaller_or_equal(a, b)),
        // `a > b` is `b < a`; `a >= b` is `b <= a`.
        Gt => Zval::Bool(ops::smaller(b, a)),
        Ge => Zval::Bool(ops::smaller_or_equal(b, a)),
        Spaceship => Zval::Long(ops::compare(a, b) as i64),
    })
}

fn apply_unop(op: UnOp, a: &Zval, d: &mut Diags) -> Result<Zval, PhpError> {
    use UnOp::*;
    Ok(match op {
        Neg => ops::neg(a, d)?,
        // Unary `+` is numeric identity-with-coercion; `0 + a` reproduces it
        // (incl. the PHP 8 TypeError on a non-numeric string). TODO: mirror
        // eval's exact path once the call/full-operator coverage is ported.
        Plus => ops::add(&Zval::Long(0), a, d)?,
        Not => Zval::Bool(!convert::to_bool(a, d)),
        BitNot => ops::bw_not(a, d)?,
    })
}

fn apply_cast(kind: CastKind, a: &Zval, d: &mut Diags) -> Zval {
    match kind {
        CastKind::Int => Zval::Long(convert::to_long_cast(a, d)),
        CastKind::String => Zval::Str(convert::to_zstr_cast(a, d)),
        CastKind::Bool => Zval::Bool(convert::to_bool(a, d)),
        // The compiler only emits Int/String/Bool casts in the proof slice.
        other => unreachable!("VM saw an unported cast {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::compile::compile_program;
    use crate::lower::lower_source;

    use super::run_module;

    /// Compile and run a PHP snippet through the bytecode VM, returning stdout.
    fn vm_stdout(src: &[u8]) -> Vec<u8> {
        let program = lower_source(b"test.php", src).expect("lower");
        let module = compile_program(&program).expect("compile");
        let out = run_module(&module);
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        out.stdout
    }

    #[test]
    fn echo_arithmetic() {
        assert_eq!(vm_stdout(b"<?php echo 1 + 2;"), b"3");
        assert_eq!(vm_stdout(b"<?php echo 2 * 3 + 4;"), b"10");
        assert_eq!(vm_stdout(b"<?php echo 'a' . 'b' . 'c';"), b"abc");
    }

    #[test]
    fn variables_and_compound_assign() {
        assert_eq!(vm_stdout(b"<?php $x = 5; $y = $x * 2; echo $y;"), b"10");
        assert_eq!(vm_stdout(b"<?php $x = 1; $x += 4; $x .= '!'; echo $x;"), b"5!");
    }

    #[test]
    fn inc_dec() {
        assert_eq!(vm_stdout(b"<?php $i = 0; echo $i++; echo $i; echo ++$i;"), b"012");
    }

    #[test]
    fn if_else() {
        assert_eq!(vm_stdout(b"<?php if (1 < 2) { echo 'a'; } else { echo 'b'; }"), b"a");
        assert_eq!(vm_stdout(b"<?php if (0) { echo 'a'; } elseif (1) { echo 'b'; } else { echo 'c'; }"), b"b");
    }

    #[test]
    fn while_loop() {
        assert_eq!(vm_stdout(b"<?php $i = 0; while ($i < 3) { echo $i; $i = $i + 1; }"), b"012");
    }

    #[test]
    fn for_loop_with_break_continue() {
        assert_eq!(
            vm_stdout(b"<?php for ($i = 0; $i < 5; $i++) { if ($i == 2) { continue; } if ($i == 4) { break; } echo $i; }"),
            b"013"
        );
    }

    #[test]
    fn short_circuit_and_ternary() {
        assert_eq!(vm_stdout(b"<?php echo (1 && 0) ? 'y' : 'n';"), b"n");
        assert_eq!(vm_stdout(b"<?php echo (1 || 0) ? 'y' : 'n';"), b"y");
        assert_eq!(vm_stdout(b"<?php echo 0 ?: 'fallback';"), b"fallback");
    }

    #[test]
    fn print_is_expression() {
        // `print` evaluates to int(1): "x" is printed, then 1 echoed.
        assert_eq!(vm_stdout(b"<?php echo print 'x';"), b"x1");
    }
}
