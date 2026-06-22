//! HIR → bytecode compiler (VM-migration Fase 3, vertical proof slice).
//!
//! Where [`crate::eval`] `match`es on an [`crate::hir::ExprKind`] to *execute*
//! it, this module `match`es to *emit* [`crate::bytecode::Op`]s. The two share
//! the same source of truth (the HIR) and the same value semantics
//! (`php_types::ops` / `convert`, invoked by the VM, not re-implemented here).
//!
//! # Status: Tier-1 proof slice
//!
//! This first cut compiles exactly the subset needed to prove the
//! compile→VM spine end-to-end: echo/print, scalar literals, local
//! read/write (incl. compound and inc/dec on a bare slot), the binary/unary
//! and Int/String/Bool casts, structured control flow (`if`/`while`/`do-while`/
//! `for`/ternary/short-circuit `&&` `||`, `break N` / `continue N`), and
//! `return`. Anything else returns [`CompileError::Unsupported`] with the HIR
//! variant name — the same "name the gap" discipline `lower` uses — so widening
//! coverage is a matter of turning `Unsupported` arms into emit arms.
//!
//! Calls, arrays, references, OOP and generators are deliberately out of slice;
//! `Module::functions` / `closures` are left empty until the call opcode lands.

use crate::bytecode::{Addr, Const, ConstIdx, Func, Module, Op};
use crate::hir::{Expr, ExprKind, Program, Stmt, StmtKind};

/// A construct the proof-slice compiler does not yet lower. Carries the HIR
/// variant name so the coverage gap is legible (mirrors `lower::LowerError`).
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    Unsupported(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Unsupported(what) => write!(f, "VM compile: unsupported {what} (not yet ported)"),
        }
    }
}

type R<T> = Result<T, CompileError>;

/// Compile a lowered [`Program`] into an executable [`Module`].
///
/// Only the script body (`main`) is compiled in the proof slice; the function
/// and closure tables are populated once the call opcode is wired in.
pub fn compile_program(program: &Program) -> R<Module> {
    let main = compile_func(b"", &program.body, program.slots.len() as u32, 0)?;
    Ok(Module {
        main,
        functions: Vec::new(),
        closures: Vec::new(),
        file: program.file.clone(),
    })
}

/// Compile one body (the script's, or later a function's) into a [`Func`].
fn compile_func(name: &[u8], body: &[Stmt], n_slots: u32, n_params: u32) -> R<Func> {
    let mut c = FnCompiler::default();
    c.block(body)?;
    // A body that runs off the end returns NULL (PHP's implicit return).
    let null = c.konst(Const::Null);
    c.emit(Op::PushConst(null));
    c.emit(Op::Ret);
    Ok(Func {
        name: name.into(),
        ops: c.ops,
        consts: c.consts,
        n_slots,
        n_params,
        by_ref: false,
        is_generator: false,
        line: 0,
    })
}

/// Per-function emit state: the growing instruction stream, the constant pool,
/// and the stack of enclosing loops (for `break N` / `continue N`).
#[derive(Default)]
struct FnCompiler {
    ops: Vec<Op>,
    consts: Vec<Const>,
    loops: Vec<LoopCtx>,
}

/// One enclosing loop's unresolved jump sites. `break` jumps land at the loop
/// exit; `continue` jumps land at the loop's step/condition re-entry. Both are
/// patched once those addresses are known.
#[derive(Default)]
struct LoopCtx {
    break_sites: Vec<Addr>,
    continue_sites: Vec<Addr>,
}

impl FnCompiler {
    /// Append `op`, returning its address.
    fn emit(&mut self, op: Op) -> Addr {
        let at = self.ops.len() as Addr;
        self.ops.push(op);
        at
    }

    /// The address the next emitted op will occupy.
    fn here(&self) -> Addr {
        self.ops.len() as Addr
    }

    /// Overwrite the op at `at` (used to back-patch a jump once its target is known).
    fn patch(&mut self, at: Addr, op: Op) {
        self.ops[at as usize] = op;
    }

    /// Intern a literal into the constant pool, returning its index.
    fn konst(&mut self, c: Const) -> ConstIdx {
        if let Some(i) = self.consts.iter().position(|e| *e == c) {
            return i as ConstIdx;
        }
        let i = self.consts.len() as ConstIdx;
        self.consts.push(c);
        i
    }

    fn block(&mut self, stmts: &[Stmt]) -> R<()> {
        for s in stmts {
            self.stmt(s)?;
        }
        Ok(())
    }

    fn stmt(&mut self, s: &Stmt) -> R<()> {
        match &s.kind {
            StmtKind::Nop => {}
            StmtKind::Echo(values) => {
                for e in values {
                    self.expr(e)?;
                    self.emit(Op::Echo);
                }
            }
            StmtKind::Expr(e) => {
                // Every expression leaves exactly one value; a statement must
                // restore the stack depth, so discard it.
                self.expr(e)?;
                self.emit(Op::Pop);
            }
            StmtKind::Block(body) => self.block(body)?,
            StmtKind::If { cond, then, elseifs, otherwise } => {
                // Collect the (cond, body) arms; the final `else` has no cond.
                let mut end_jumps: Vec<Addr> = Vec::new();
                self.cond_chain(cond, then, &mut end_jumps)?;
                for (c, b) in elseifs {
                    self.cond_chain(c, b, &mut end_jumps)?;
                }
                self.block(otherwise)?;
                let end = self.here();
                for j in end_jumps {
                    self.patch(j, Op::Jump(end));
                }
            }
            StmtKind::While { cond, body } => {
                let top = self.here();
                self.expr(cond)?;
                let exit = self.emit(Op::JumpIfFalse(Addr::MAX));
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                self.emit(Op::Jump(top));
                let end = self.here();
                self.patch(exit, Op::JumpIfFalse(end));
                self.close_loop(top, end);
            }
            StmtKind::DoWhile { body, cond } => {
                let top = self.here();
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                let cont = self.here();
                self.expr(cond)?;
                self.emit(Op::JumpIfTrue(top));
                let end = self.here();
                // `continue` in a do-while re-tests the condition.
                self.close_loop(cont, end);
            }
            StmtKind::For { init, cond, step, body } => {
                for e in init {
                    self.expr(e)?;
                    self.emit(Op::Pop);
                }
                let top = self.here();
                let exit = self.cond_list(cond)?;
                self.loops.push(LoopCtx::default());
                self.block(body)?;
                let cont = self.here();
                for e in step {
                    self.expr(e)?;
                    self.emit(Op::Pop);
                }
                self.emit(Op::Jump(top));
                let end = self.here();
                if let Some(exit) = exit {
                    self.patch(exit, Op::JumpIfFalse(end));
                }
                self.close_loop(cont, end);
            }
            StmtKind::Break(n) => self.loop_jump(*n, true)?,
            StmtKind::Continue(n) => self.loop_jump(*n, false)?,
            StmtKind::Return(opt) => {
                match opt {
                    Some(e) => self.expr(e)?,
                    None => {
                        let null = self.konst(Const::Null);
                        self.emit(Op::PushConst(null));
                    }
                }
                self.emit(Op::Ret);
            }
            other => return Err(CompileError::Unsupported(stmt_name(other))),
        }
        Ok(())
    }

    /// Emit one `if`/`elseif` arm: `cond`, a `JumpIfFalse` past the body, the
    /// body, and a `Jump` to the chain end (recorded for back-patching).
    fn cond_chain(&mut self, cond: &Expr, body: &[Stmt], end_jumps: &mut Vec<Addr>) -> R<()> {
        self.expr(cond)?;
        let skip = self.emit(Op::JumpIfFalse(Addr::MAX));
        self.block(body)?;
        end_jumps.push(self.emit(Op::Jump(Addr::MAX)));
        let after = self.here();
        self.patch(skip, Op::JumpIfFalse(after));
        Ok(())
    }

    /// Compile a `for`'s comma-separated condition list: all but the last are
    /// evaluated for side effects; the last drives the loop. Returns the address
    /// of the `JumpIfFalse` to back-patch, or `None` for an empty (infinite) list.
    fn cond_list(&mut self, conds: &[Expr]) -> R<Option<Addr>> {
        if conds.is_empty() {
            return Ok(None);
        }
        let (last, rest) = conds.split_last().unwrap();
        for e in rest {
            self.expr(e)?;
            self.emit(Op::Pop);
        }
        self.expr(last)?;
        Ok(Some(self.emit(Op::JumpIfFalse(Addr::MAX))))
    }

    /// Pop the just-compiled loop and resolve its `break`/`continue` jump sites.
    fn close_loop(&mut self, continue_target: Addr, break_target: Addr) {
        let ctx = self.loops.pop().expect("close_loop without an open loop");
        for at in ctx.break_sites {
            self.patch(at, Op::Jump(break_target));
        }
        for at in ctx.continue_sites {
            self.patch(at, Op::Jump(continue_target));
        }
    }

    /// Emit a `break N` / `continue N` as a placeholder `Jump`, registered with
    /// the N-th enclosing loop for back-patching. `level` is >= 1.
    fn loop_jump(&mut self, level: u32, is_break: bool) -> R<()> {
        let depth = self.loops.len();
        if level == 0 || (level as usize) > depth {
            // PHP rejects this at compile time; surface it the same way.
            let kw = if is_break { "break" } else { "continue" };
            return Err(CompileError::Unsupported(format!(
                "{kw} {level} with {depth} enclosing loop(s)"
            )));
        }
        let at = self.emit(Op::Jump(Addr::MAX));
        let idx = depth - level as usize;
        if is_break {
            self.loops[idx].break_sites.push(at);
        } else {
            self.loops[idx].continue_sites.push(at);
        }
        Ok(())
    }

    fn expr(&mut self, e: &Expr) -> R<()> {
        match &e.kind {
            ExprKind::Null => {
                let k = self.konst(Const::Null);
                self.emit(Op::PushConst(k));
            }
            ExprKind::Bool(b) => {
                let k = self.konst(Const::Bool(*b));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Int(i) => {
                let k = self.konst(Const::Int(*i));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Float(f) => {
                let k = self.konst(Const::Float(*f));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Str(s) => {
                let k = self.konst(Const::Str(s.clone()));
                self.emit(Op::PushConst(k));
            }
            ExprKind::Var(slot) => {
                self.emit(Op::LoadSlot(*slot));
            }
            ExprKind::Assign(slot, rhs) => {
                self.expr(rhs)?;
                self.emit(Op::Dup); // assignment is an expression valued by the RHS
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::AssignOp(op, slot, rhs) => {
                self.emit(Op::LoadSlot(*slot));
                self.expr(rhs)?;
                self.emit(Op::Binary(*op));
                self.emit(Op::Dup);
                self.emit(Op::StoreSlot(*slot));
            }
            ExprKind::IncDec { slot, inc, pre } => {
                self.emit(Op::IncDecSlot { slot: *slot, inc: *inc, pre: *pre });
            }
            ExprKind::Binary(op, a, b) => {
                self.expr(a)?;
                self.expr(b)?;
                self.emit(Op::Binary(*op));
            }
            ExprKind::Unary(op, a) => {
                self.expr(a)?;
                self.emit(Op::Unary(*op));
            }
            ExprKind::Cast(kind, a) => {
                use crate::hir::CastKind;
                match kind {
                    CastKind::Int | CastKind::String | CastKind::Bool => {
                        self.expr(a)?;
                        self.emit(Op::Cast(*kind));
                    }
                    // Float/Array/Object casts await the broader value-conversion
                    // and array opcodes.
                    other => return Err(CompileError::Unsupported(format!("cast {other:?}"))),
                }
            }
            ExprKind::And(a, b) => self.short_circuit(a, b, false)?,
            ExprKind::Or(a, b) => self.short_circuit(a, b, true)?,
            ExprKind::Ternary { cond, then, otherwise } => {
                match then {
                    Some(then) => {
                        // cond ? then : otherwise
                        self.expr(cond)?;
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        self.expr(then)?;
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                    None => {
                        // cond ?: otherwise — evaluate cond once, reuse if truthy.
                        self.expr(cond)?;
                        self.emit(Op::Dup);
                        let to_else = self.emit(Op::JumpIfFalse(Addr::MAX));
                        let to_end = self.emit(Op::Jump(Addr::MAX));
                        let else_at = self.here();
                        self.patch(to_else, Op::JumpIfFalse(else_at));
                        self.emit(Op::Pop); // discard the falsy cond copy
                        self.expr(otherwise)?;
                        let end = self.here();
                        self.patch(to_end, Op::Jump(end));
                    }
                }
            }
            ExprKind::Print(a) => {
                self.expr(a)?;
                self.emit(Op::Print);
            }
            other => return Err(CompileError::Unsupported(expr_name(other))),
        }
        Ok(())
    }

    /// Compile `&&` (`want_true == false`) / `||` (`want_true == true`) to a
    /// boolean result via short-circuit jumps. Leaves `true`/`false` on the stack.
    fn short_circuit(&mut self, a: &Expr, b: &Expr, want_true: bool) -> R<()> {
        // For `&&`: if either operand is falsy, result is false (jump to L_short).
        // For `||`: if either operand is truthy, result is true.
        let short = |s: &mut Self| {
            if want_true {
                s.emit(Op::JumpIfTrue(Addr::MAX))
            } else {
                s.emit(Op::JumpIfFalse(Addr::MAX))
            }
        };
        self.expr(a)?;
        let j1 = short(self);
        self.expr(b)?;
        let j2 = short(self);
        // Fell through: `&&` → true, `||` → false.
        let fallthrough = self.konst(Const::Bool(!want_true));
        self.emit(Op::PushConst(fallthrough));
        let to_end = self.emit(Op::Jump(Addr::MAX));
        let short_at = self.here();
        self.patch(j1, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        self.patch(j2, if want_true { Op::JumpIfTrue(short_at) } else { Op::JumpIfFalse(short_at) });
        let shorted = self.konst(Const::Bool(want_true));
        self.emit(Op::PushConst(shorted));
        let end = self.here();
        self.patch(to_end, Op::Jump(end));
        Ok(())
    }
}

/// HIR statement-variant name, for [`CompileError::Unsupported`].
fn stmt_name(k: &StmtKind) -> String {
    let n = match k {
        StmtKind::Echo(_) => "Echo",
        StmtKind::InlineHtml(_) => "InlineHtml",
        StmtKind::Expr(_) => "Expr",
        StmtKind::Block(_) => "Block",
        StmtKind::If { .. } => "If",
        StmtKind::While { .. } => "While",
        StmtKind::DoWhile { .. } => "DoWhile",
        StmtKind::For { .. } => "For",
        StmtKind::Foreach { .. } => "Foreach",
        StmtKind::Switch { .. } => "Switch",
        StmtKind::Unset(_) => "Unset",
        StmtKind::Global(_) => "Global",
        StmtKind::StaticVar(_) => "StaticVar",
        StmtKind::Break(_) => "Break",
        StmtKind::Continue(_) => "Continue",
        StmtKind::Return(_) => "Return",
        StmtKind::ReturnRef(_) => "ReturnRef",
        StmtKind::Try { .. } => "Try",
        StmtKind::Label(_) => "Label",
        StmtKind::Goto(_) => "Goto",
        StmtKind::Nop => "Nop",
    };
    format!("statement {n}")
}

/// HIR expression-variant name, for [`CompileError::Unsupported`].
fn expr_name(k: &ExprKind) -> String {
    let n = match k {
        ExprKind::Null => "Null",
        ExprKind::Bool(_) => "Bool",
        ExprKind::Int(_) => "Int",
        ExprKind::Float(_) => "Float",
        ExprKind::Str(_) => "Str",
        ExprKind::Const(_) => "Const",
        ExprKind::Var(_) => "Var",
        ExprKind::GlobalVar(_) => "GlobalVar",
        ExprKind::Binary(..) => "Binary",
        ExprKind::And(..) => "And",
        ExprKind::Or(..) => "Or",
        ExprKind::Xor(..) => "Xor",
        ExprKind::Coalesce(..) => "Coalesce",
        ExprKind::Unary(..) => "Unary",
        ExprKind::Cast(..) => "Cast",
        ExprKind::Assign(..) => "Assign",
        ExprKind::AssignRef { .. } => "AssignRef",
        ExprKind::AssignRefCall { .. } => "AssignRefCall",
        ExprKind::AssignOp(..) => "AssignOp",
        ExprKind::AssignCoalesce(..) => "AssignCoalesce",
        ExprKind::IncDec { .. } => "IncDec",
        ExprKind::IncDecPlace { .. } => "IncDecPlace",
        ExprKind::Ternary { .. } => "Ternary",
        ExprKind::Call { .. } => "Call",
        ExprKind::Closure { .. } => "Closure",
        ExprKind::FirstClassCallable(_) => "FirstClassCallable",
        ExprKind::CallDynamic { .. } => "CallDynamic",
        ExprKind::Spread(_) => "Spread",
        ExprKind::Array(_) => "Array",
        ExprKind::Index { .. } => "Index",
        ExprKind::AssignPlace(..) => "AssignPlace",
        ExprKind::AssignOpPlace(..) => "AssignOpPlace",
        ExprKind::AssignCoalescePlace(..) => "AssignCoalescePlace",
        ExprKind::Isset(_) => "Isset",
        ExprKind::Empty(_) => "Empty",
        ExprKind::Suppress(_) => "Suppress",
        ExprKind::Print(_) => "Print",
        ExprKind::Exit(_) => "Exit",
        ExprKind::Match { .. } => "Match",
        ExprKind::New { .. } => "New",
        ExprKind::MethodCall { .. } => "MethodCall",
        ExprKind::PropGet { .. } => "PropGet",
        ExprKind::This => "This",
        ExprKind::StaticCall { .. } => "StaticCall",
        ExprKind::ClassConst { .. } => "ClassConst",
        ExprKind::StaticProp { .. } => "StaticProp",
        ExprKind::StaticPropAssign { .. } => "StaticPropAssign",
        ExprKind::StaticPropIncDec { .. } => "StaticPropIncDec",
        ExprKind::InstanceOf { .. } => "InstanceOf",
        ExprKind::Throw(_) => "Throw",
        ExprKind::Yield { .. } => "Yield",
        ExprKind::YieldFrom(_) => "YieldFrom",
    };
    format!("expression {n}")
}
