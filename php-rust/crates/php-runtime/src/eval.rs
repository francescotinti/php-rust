//! Tree-walking evaluator over the HIR (plan step 4).
//!
//! This is the replacement for Zend's opcode VM (D-G9): instead of compiling to
//! bytecode and running `zend_execute`, we walk the resolved HIR directly and
//! `match` on enum variants. All value semantics (arithmetic, comparison, type
//! juggling, string conversion) are delegated to `php_types::ops` /
//! `php_types::convert`, the one faithfully-ported module (D-G11) — the
//! evaluator only orchestrates control flow and variable storage.
//!
//! Scope (Tier 1, step 4): echo, variables, assignments (incl. compound / `??=`
//! / inc-dec), `if`/`while`/`do-while`/`for`, ternary, `break`/`continue` with
//! levels, `return`. Arrays, functions, and OOP arrive in later steps.
//!
//! Diagnostics are *collected* into [`Outcome::diags`]; their exact rendering
//! and interleaving onto stdout is step 9, so for now the differential corpus
//! is curated to warning-free scripts.

use php_types::{convert, ops, Diag, Diags, Key, PhpArray, PhpStr, PhpError, Zval};

use crate::builtin::{Ctx, Registry};
use crate::hir::{BinOp, CastKind, Expr, ExprKind, Program, Slot, Stmt, StmtKind, UnOp};

/// Control-flow signal produced by executing a statement.
enum Flow {
    /// Fell off the end normally.
    Normal,
    /// `break N` still propagating (N levels remain).
    Break(u32),
    /// `continue N` still propagating (N levels remain).
    Continue(u32),
    /// `return expr` unwinding to the script top.
    Return(Zval),
}

/// The result of running a script.
#[derive(Debug)]
pub struct Outcome {
    /// Bytes written by `echo` / inline HTML, in order.
    pub stdout: Vec<u8>,
    /// Non-fatal diagnostics raised during execution (rendering is step 9).
    pub diags: Diags,
    /// An uncaught fatal error that aborted execution, if any.
    pub fatal: Option<PhpError>,
    /// Top-level `return` value (NULL if the script ran to completion).
    pub return_value: Zval,
}

/// Lower `source` and run it with no builtins. Convenience wrapper over [`run`].
pub fn run_source(name: &[u8], source: &[u8]) -> Result<Outcome, crate::LowerError> {
    let program = crate::lower_source(name, source)?;
    Ok(run(&program))
}

/// Lower `source` and run it with the given builtin registry.
pub fn run_source_with(
    name: &[u8],
    source: &[u8],
    registry: &Registry,
) -> Result<Outcome, crate::LowerError> {
    let program = crate::lower_source(name, source)?;
    Ok(run_with(&program, registry))
}

/// Execute a lowered program with no builtins registered.
pub fn run(program: &Program) -> Outcome {
    run_with(program, &Registry::new())
}

/// Execute a lowered program, resolving function calls against `registry`.
pub fn run_with(program: &Program, registry: &Registry) -> Outcome {
    let mut ev = Evaluator {
        names: &program.slots,
        reg: registry,
        slots: vec![Zval::Undef; program.slots.len()],
        out: Vec::new(),
        diags: Vec::new(),
    };

    let (fatal, return_value) = match ev.exec_stmts(&program.body) {
        Ok(Flow::Return(v)) => (None, v),
        Ok(_) => (None, Zval::Null),
        Err(e) => (Some(e), Zval::Null),
    };

    Outcome {
        stdout: ev.out,
        diags: ev.diags,
        fatal,
        return_value,
    }
}

struct Evaluator<'p> {
    names: &'p [Box<[u8]>],
    reg: &'p Registry,
    slots: Vec<Zval>,
    out: Vec<u8>,
    diags: Diags,
}

/// What a loop should do after running its body once.
enum LoopStep {
    /// Run another iteration (Normal fall-through or `continue` at this level).
    Iterate,
    /// Stop this loop (`break` at this level).
    Stop,
    /// A signal targeting an enclosing loop / the function — propagate it.
    Propagate(Flow),
}

impl<'p> Evaluator<'p> {
    // --- statements ---

    fn exec_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, PhpError> {
        for s in stmts {
            match self.exec_stmt(s)? {
                Flow::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow, PhpError> {
        match &stmt.kind {
            StmtKind::Nop => {}

            StmtKind::InlineHtml(bytes) => self.out.extend_from_slice(bytes),

            StmtKind::Echo(values) => {
                for e in values {
                    let z = self.eval(e)?;
                    let s = convert::to_zstr(&z, &mut self.diags);
                    self.out.extend_from_slice(s.as_bytes());
                }
            }

            StmtKind::Expr(e) => {
                self.eval(e)?;
            }

            StmtKind::Block(body) => return self.exec_stmts(body),

            StmtKind::If {
                cond,
                then,
                elseifs,
                otherwise,
            } => {
                if self.eval_bool(cond)? {
                    return self.exec_stmts(then);
                }
                for (econd, ebody) in elseifs {
                    if self.eval_bool(econd)? {
                        return self.exec_stmts(ebody);
                    }
                }
                return self.exec_stmts(otherwise);
            }

            StmtKind::While { cond, body } => {
                while self.eval_bool(cond)? {
                    match self.loop_step(body)? {
                        LoopStep::Iterate => {}
                        LoopStep::Stop => break,
                        LoopStep::Propagate(f) => return Ok(f),
                    }
                }
            }

            StmtKind::DoWhile { body, cond } => loop {
                match self.loop_step(body)? {
                    LoopStep::Iterate => {}
                    LoopStep::Stop => break,
                    LoopStep::Propagate(f) => return Ok(f),
                }
                if !self.eval_bool(cond)? {
                    break;
                }
            },

            StmtKind::For {
                init,
                cond,
                step,
                body,
            } => {
                for e in init {
                    self.eval(e)?;
                }
                loop {
                    if !self.eval_for_cond(cond)? {
                        break;
                    }
                    match self.loop_step(body)? {
                        LoopStep::Iterate => {}
                        LoopStep::Stop => break,
                        LoopStep::Propagate(f) => return Ok(f),
                    }
                    for e in step {
                        self.eval(e)?;
                    }
                }
            }

            StmtKind::Break(n) => return Ok(Flow::Break(*n)),
            StmtKind::Continue(n) => return Ok(Flow::Continue(*n)),
            StmtKind::Return(opt) => {
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                return Ok(Flow::Return(v));
            }
        }
        Ok(Flow::Normal)
    }

    /// Run a loop body once and translate its control-flow signal relative to
    /// *this* loop level.
    fn loop_step(&mut self, body: &[Stmt]) -> Result<LoopStep, PhpError> {
        Ok(match self.exec_stmts(body)? {
            Flow::Normal | Flow::Continue(1) => LoopStep::Iterate,
            Flow::Continue(n) => LoopStep::Propagate(Flow::Continue(n - 1)),
            Flow::Break(1) => LoopStep::Stop,
            Flow::Break(n) => LoopStep::Propagate(Flow::Break(n - 1)),
            Flow::Return(v) => LoopStep::Propagate(Flow::Return(v)),
        })
    }

    /// `for` condition list: every expression runs, the last one's truthiness
    /// controls the loop; an empty list means "always true".
    fn eval_for_cond(&mut self, cond: &[Expr]) -> Result<bool, PhpError> {
        let mut truthy = true;
        for c in cond {
            truthy = convert::to_bool(&self.eval(c)?, &mut self.diags);
        }
        Ok(truthy)
    }

    fn eval_bool(&mut self, e: &Expr) -> Result<bool, PhpError> {
        let v = self.eval(e)?;
        Ok(convert::to_bool(&v, &mut self.diags))
    }

    // --- expressions ---

    fn eval(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        match &e.kind {
            ExprKind::Null => Ok(Zval::Null),
            ExprKind::Bool(b) => Ok(Zval::Bool(*b)),
            ExprKind::Int(i) => Ok(Zval::Long(*i)),
            ExprKind::Float(f) => Ok(Zval::Double(*f)),
            ExprKind::Str(bytes) => Ok(Zval::Str(PhpStr::new(bytes.clone()))),

            ExprKind::Var(slot) => Ok(self.read_var(*slot)),

            ExprKind::Binary(op, l, r) => {
                let a = self.eval(l)?;
                let b = self.eval(r)?;
                self.apply_binop(*op, a, b)
            }

            // Short-circuit logical operators always yield a clean bool.
            ExprKind::And(l, r) => {
                if !self.eval_bool(l)? {
                    Ok(Zval::Bool(false))
                } else {
                    Ok(Zval::Bool(self.eval_bool(r)?))
                }
            }
            ExprKind::Or(l, r) => {
                if self.eval_bool(l)? {
                    Ok(Zval::Bool(true))
                } else {
                    Ok(Zval::Bool(self.eval_bool(r)?))
                }
            }
            ExprKind::Xor(l, r) => {
                let a = self.eval_bool(l)?;
                let b = self.eval_bool(r)?;
                Ok(Zval::Bool(a ^ b))
            }

            ExprKind::Coalesce(l, r) => {
                let lv = self.eval_isset(l)?;
                if matches!(lv, Zval::Null | Zval::Undef) {
                    self.eval(r)
                } else {
                    Ok(lv)
                }
            }

            ExprKind::Unary(op, operand) => {
                let v = self.eval(operand)?;
                match op {
                    UnOp::Neg => ops::neg(&v, &mut self.diags),
                    // Unary `+` is the numeric coercion `1 * v` (same TypeError surface).
                    UnOp::Plus => ops::mul(&Zval::Long(1), &v, &mut self.diags),
                    UnOp::Not => Ok(Zval::Bool(!convert::to_bool(&v, &mut self.diags))),
                    UnOp::BitNot => ops::bw_not(&v, &mut self.diags),
                }
            }

            ExprKind::Cast(kind, operand) => {
                let v = self.eval(operand)?;
                Ok(match kind {
                    CastKind::Int => Zval::Long(convert::to_long_cast(&v, &mut self.diags)),
                    CastKind::Float => Zval::Double(convert::to_double(&v)),
                    CastKind::String => Zval::Str(convert::to_zstr_cast(&v, &mut self.diags)),
                    CastKind::Bool => Zval::Bool(convert::to_bool(&v, &mut self.diags)),
                    CastKind::Array => array_cast(v),
                })
            }

            ExprKind::Assign(slot, rhs) => {
                let v = self.eval(rhs)?;
                self.slots[*slot as usize] = v.clone();
                Ok(v)
            }

            ExprKind::AssignOp(op, slot, rhs) => {
                let cur = self.read_var(*slot);
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.slots[*slot as usize] = res.clone();
                Ok(res)
            }

            ExprKind::AssignCoalesce(slot, rhs) => {
                let cur = self.slots[*slot as usize].clone();
                if matches!(cur, Zval::Null | Zval::Undef) {
                    let v = self.eval(rhs)?;
                    self.slots[*slot as usize] = v.clone();
                    Ok(v)
                } else {
                    Ok(cur)
                }
            }

            ExprKind::IncDec { slot, inc, pre } => {
                let idx = *slot as usize;
                if matches!(self.slots[idx], Zval::Undef) {
                    self.warn_undef(*slot);
                    self.slots[idx] = Zval::Null;
                }
                let old = self.slots[idx].clone();
                if *inc {
                    ops::increment(&mut self.slots[idx], &mut self.diags)?;
                } else {
                    ops::decrement(&mut self.slots[idx], &mut self.diags)?;
                }
                Ok(if *pre {
                    self.slots[idx].clone()
                } else {
                    old
                })
            }

            ExprKind::Ternary {
                cond,
                then,
                otherwise,
            } => {
                let c = self.eval(cond)?;
                if convert::to_bool(&c, &mut self.diags) {
                    match then {
                        // Full ternary: evaluate the "then" branch.
                        Some(t) => self.eval(t),
                        // Short ternary `?:` returns the (truthy) condition itself.
                        None => Ok(c),
                    }
                } else {
                    self.eval(otherwise)
                }
            }

            ExprKind::Call { name, args } => {
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                // Copy the fn pointer out so the registry borrow ends before we
                // borrow `out`/`diags` mutably for the call context.
                let f = match self.reg.get(name.as_ref()) {
                    Some(f) => *f,
                    None => {
                        return Err(PhpError::Error(format!(
                            "Call to undefined function {}()",
                            String::from_utf8_lossy(name)
                        )))
                    }
                };
                let mut ctx = Ctx {
                    out: &mut self.out,
                    diags: &mut self.diags,
                };
                f(&argv, &mut ctx)
            }
        }
    }

    fn apply_binop(&mut self, op: BinOp, a: Zval, b: Zval) -> Result<Zval, PhpError> {
        let d = &mut self.diags;
        match op {
            BinOp::Add => ops::add(&a, &b, d),
            BinOp::Sub => ops::sub(&a, &b, d),
            BinOp::Mul => ops::mul(&a, &b, d),
            BinOp::Div => ops::div(&a, &b, d),
            BinOp::Mod => ops::modulo(&a, &b, d),
            BinOp::Pow => ops::pow(&a, &b, d),
            BinOp::Concat => ops::concat(&a, &b, d),
            BinOp::BitAnd => ops::bw_and(&a, &b, d),
            BinOp::BitOr => ops::bw_or(&a, &b, d),
            BinOp::BitXor => ops::bw_xor(&a, &b, d),
            BinOp::Shl => ops::shl(&a, &b, d),
            BinOp::Shr => ops::shr(&a, &b, d),
            BinOp::Eq => Ok(Zval::Bool(ops::loose_eq(&a, &b))),
            BinOp::NotEq => Ok(Zval::Bool(!ops::loose_eq(&a, &b))),
            BinOp::Identical => Ok(Zval::Bool(ops::identical(&a, &b))),
            BinOp::NotIdentical => Ok(Zval::Bool(!ops::identical(&a, &b))),
            BinOp::Lt => Ok(Zval::Bool(ops::smaller(&a, &b))),
            BinOp::Le => Ok(Zval::Bool(ops::smaller_or_equal(&a, &b))),
            // `a > b` ⟺ `b < a`; `a >= b` ⟺ `b <= a`.
            BinOp::Gt => Ok(Zval::Bool(ops::smaller(&b, &a))),
            BinOp::Ge => Ok(Zval::Bool(ops::smaller_or_equal(&b, &a))),
            BinOp::Spaceship => Ok(Zval::Long(ops::compare(&a, &b) as i64)),
        }
    }

    /// Read a variable slot. An unset slot raises "Undefined variable $name"
    /// and yields NULL (the runtime equivalent of HIR `Var` access).
    fn read_var(&mut self, slot: Slot) -> Zval {
        match &self.slots[slot as usize] {
            Zval::Undef => {
                self.warn_undef(slot);
                Zval::Null
            }
            v => v.clone(),
        }
    }

    /// Read a slot in an isset-like context (the LHS of `??`): an unset slot is
    /// silently treated as NULL, with no warning.
    fn eval_isset(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        if let ExprKind::Var(slot) = &e.kind {
            let v = self.slots[*slot as usize].clone();
            return Ok(if matches!(v, Zval::Undef) { Zval::Null } else { v });
        }
        self.eval(e)
    }

    fn warn_undef(&mut self, slot: Slot) {
        let name = String::from_utf8_lossy(&self.names[slot as usize]);
        self.diags
            .push(Diag::Warning(format!("Undefined variable ${name}")));
    }
}

/// `(array)` cast: arrays pass through, null/undef become `[]`, and any scalar
/// becomes a single-element array keyed at `0`.
fn array_cast(v: Zval) -> Zval {
    match v {
        Zval::Array(_) => v,
        Zval::Null | Zval::Undef => Zval::Array(std::rc::Rc::new(PhpArray::new())),
        scalar => {
            let mut arr = PhpArray::new();
            arr.insert(Key::Int(0), scalar);
            Zval::Array(std::rc::Rc::new(arr))
        }
    }
}
