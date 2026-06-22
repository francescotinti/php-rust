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
    let mut vm = Vm {
        module,
        stdout: Vec::new(),
        diags: Diags::new(),
        frames: Vec::new(),
    };
    vm.frames.push(Frame::new(&module.main));
    let fatal = vm.run().err();
    VmOutcome {
        stdout: vm.stdout,
        diags: vm.diags,
        fatal,
    }
}

/// One activation record: the function being run, its instruction pointer, its
/// slot array (named locals) and its operand stack. This is the unit that would
/// be parked to suspend a generator — `ip` + `slots` + `stack`, all owned, no
/// Rust stack involved.
struct Frame<'m> {
    func: &'m Func,
    ip: usize,
    slots: Vec<Zval>,
    stack: Vec<Zval>,
}

impl<'m> Frame<'m> {
    fn new(func: &'m Func) -> Self {
        Frame {
            func,
            ip: 0,
            slots: vec![Zval::Undef; func.n_slots as usize],
            stack: Vec::new(),
        }
    }
}

/// The virtual machine: the module under execution plus the explicit call stack.
/// PHP function calls grow `frames` rather than the Rust stack, so deep PHP
/// recursion cannot overflow the host stack, and a frame is suspendable.
struct Vm<'m> {
    module: &'m Module,
    stdout: Vec<u8>,
    diags: Diags,
    frames: Vec<Frame<'m>>,
}

impl Vm<'_> {
    /// Run until the bottom frame returns, yielding the script's result value.
    fn run(&mut self) -> Result<Zval, PhpError> {
        loop {
            let top = self.frames.len() - 1;
            let ip = self.frames[top].ip;
            let op = self.frames[top].func.ops[ip].clone();
            // Default fall-through advance. Jumps overwrite `ip`; `Call` advances
            // the *caller* before pushing the callee; `Ret` discards this frame.
            self.frames[top].ip = ip + 1;

            match op {
                Op::PushConst(i) => {
                    let v = self.frames[top].func.consts[i as usize].to_zval();
                    self.frames[top].stack.push(v);
                }
                Op::Pop => {
                    self.frames[top].stack.pop();
                }
                Op::Dup => {
                    let v = self.frames[top].stack.last().expect("Dup on empty stack").clone();
                    self.frames[top].stack.push(v);
                }
                Op::LoadSlot(s) => {
                    // An unset local reads as NULL (the curated corpus is
                    // warning-free; the "Undefined variable" notice rides the
                    // diagnostics-ordering work). A reference slot is followed.
                    let v = read_slot(&self.frames[top].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSlot(s) => {
                    let v = self.frames[top].stack.pop().expect("StoreSlot on empty stack");
                    store_slot(&mut self.frames[top].slots[s as usize], v);
                }
                Op::IncDecSlot { slot, inc, pre } => {
                    let i = slot as usize;
                    if matches!(self.frames[top].slots[i], Zval::Undef) {
                        self.frames[top].slots[i] = Zval::Null;
                    }
                    let old = if pre { None } else { Some(self.frames[top].slots[i].clone()) };
                    {
                        let cell = &mut self.frames[top].slots[i];
                        if inc {
                            ops::increment(cell, &mut self.diags)?;
                        } else {
                            ops::decrement(cell, &mut self.diags)?;
                        }
                    }
                    let pushed = old.unwrap_or_else(|| self.frames[top].slots[i].clone());
                    self.frames[top].stack.push(pushed);
                }
                Op::Binary(b) => {
                    let rhs = self.frames[top].stack.pop().expect("Binary rhs");
                    let lhs = self.frames[top].stack.pop().expect("Binary lhs");
                    let r = apply_binop(b, &lhs, &rhs, &mut self.diags)?;
                    self.frames[top].stack.push(r);
                }
                Op::Unary(u) => {
                    let a = self.frames[top].stack.pop().expect("Unary operand");
                    let r = apply_unop(u, &a, &mut self.diags)?;
                    self.frames[top].stack.push(r);
                }
                Op::Cast(k) => {
                    let a = self.frames[top].stack.pop().expect("Cast operand");
                    let r = apply_cast(k, &a, &mut self.diags);
                    self.frames[top].stack.push(r);
                }
                Op::Jump(addr) => {
                    self.frames[top].ip = addr as usize;
                }
                Op::JumpIfFalse(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfFalse cond");
                    if !convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::JumpIfTrue(addr) => {
                    let c = self.frames[top].stack.pop().expect("JumpIfTrue cond");
                    if convert::to_bool(&c, &mut self.diags) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::Echo => {
                    let v = self.frames[top].stack.pop().expect("Echo operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.stdout.extend_from_slice(s.as_bytes());
                }
                Op::Print => {
                    let v = self.frames[top].stack.pop().expect("Print operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.stdout.extend_from_slice(s.as_bytes());
                    self.frames[top].stack.push(Zval::Long(1));
                }
                Op::Call { func, argc } => {
                    let callee = &self.module.functions[func as usize];
                    // Pop argc args (pushed left-to-right) and bind them to the
                    // callee's leading slots. The caller's `ip` is already past
                    // the Call, so it resumes correctly once the callee returns.
                    let n = argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("call argument"));
                    }
                    args.reverse();
                    let mut frame = Frame::new(callee);
                    for (i, a) in args.into_iter().enumerate() {
                        frame.slots[i] = a;
                    }
                    self.frames.push(frame);
                }
                Op::Ret => {
                    let ret = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    self.frames.pop();
                    match self.frames.last_mut() {
                        Some(caller) => caller.stack.push(ret),
                        None => return Ok(ret),
                    }
                }
                Op::Fatal(i) => {
                    let msg = match &self.frames[top].func.consts[i as usize] {
                        crate::bytecode::Const::Str(b) => String::from_utf8_lossy(b).into_owned(),
                        _ => "VM: unsupported construct".to_string(),
                    };
                    return Err(PhpError::Error(msg));
                }
                Op::Nop => {}
            }
        }
    }
}

/// Read a local cell's value, following a reference and mapping an unset slot to
/// NULL.
fn read_slot(cell: &Zval) -> Zval {
    match cell {
        Zval::Undef => Zval::Null,
        Zval::Ref(r) => r.borrow().clone(),
        other => other.clone(),
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

    #[test]
    fn user_function_call() {
        assert_eq!(
            vm_stdout(b"<?php function add($a, $b) { return $a + $b; } echo add(2, 3);"),
            b"5"
        );
    }

    #[test]
    fn void_return_and_multiple_calls() {
        // No explicit return -> implicit NULL; the calls run for their echo.
        assert_eq!(
            vm_stdout(b"<?php function greet() { echo 'hi'; } greet(); greet();"),
            b"hihi"
        );
    }

    #[test]
    fn recursion_uses_the_explicit_frame_stack() {
        assert_eq!(
            vm_stdout(b"<?php function fact($n) { if ($n <= 1) return 1; return $n * fact($n - 1); } echo fact(5);"),
            b"120"
        );
    }

    #[test]
    fn nested_calls_and_argument_order() {
        assert_eq!(
            vm_stdout(b"<?php function sub($a, $b) { return $a - $b; } echo sub(sub(10, 3), 2);"),
            b"5"
        );
    }
}
