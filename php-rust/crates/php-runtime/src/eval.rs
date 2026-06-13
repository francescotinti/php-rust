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

use std::collections::HashMap;
use std::rc::Rc;

use php_types::{convert, dtoa, ops, Diag, Diags, Key, PhpArray, PhpStr, PhpError, Zval};

use crate::builtin::{Ctx, Registry};
use crate::hir::{
    BinOp, CastKind, Expr, ExprKind, FnDecl, Place, PlaceStep, Program, Slot, Stmt, StmtKind, UnOp,
};

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
    // Index the hoisted user functions by ASCII-lowercased name (PHP function
    // names are case-insensitive).
    let fn_index: HashMap<Vec<u8>, usize> = program
        .functions
        .iter()
        .enumerate()
        .map(|(i, f)| (f.name.to_ascii_lowercase(), i))
        .collect();

    let mut ev = Evaluator {
        names: &program.slots,
        reg: registry,
        funcs: &program.functions,
        fn_index: &fn_index,
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
    /// Slot names for the *current* frame (script globals, or a callee's locals
    /// while a user function runs) — used only for undefined-variable warnings.
    names: &'p [Box<[u8]>],
    reg: &'p Registry,
    /// Hoisted user functions and their name→index map (built in `run_with`).
    funcs: &'p [FnDecl],
    fn_index: &'p HashMap<Vec<u8>, usize>,
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

            StmtKind::Foreach {
                iter,
                key,
                value,
                body,
            } => return self.exec_foreach(iter, *key, *value, body),

            StmtKind::Switch { subject, cases } => return self.exec_switch(subject, cases),

            StmtKind::Unset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    self.unset_place(p.slot, &steps);
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

    /// `foreach`: by-value iteration over an array snapshot (so mutating the
    /// source array in the body does not perturb the iteration — PHP's
    /// copy-on-write `foreach` semantics for arrays).
    fn exec_foreach(
        &mut self,
        iter: &Expr,
        key: Option<Slot>,
        value: Slot,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        let collection = self.eval(iter)?;
        let items: Vec<(Key, Zval)> = match collection {
            Zval::Array(a) => a.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            other => {
                self.diags.push(Diag::Warning(format!(
                    "foreach() argument must be of type array|object, {} given",
                    php_type_name(&other)
                )));
                return Ok(Flow::Normal);
            }
        };
        for (k, v) in items {
            if let Some(ks) = key {
                self.slots[ks as usize] = key_to_zval(&k);
            }
            self.slots[value as usize] = v;
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
        }
        Ok(Flow::Normal)
    }

    /// `switch`: loose-`==` matching, fall-through, and `default`. The case
    /// expressions are evaluated in source order until one matches (PHP's scan
    /// semantics); a `break`/`continue` at level 1 leaves the switch.
    fn exec_switch(&mut self, subject: &Expr, cases: &[crate::hir::Case]) -> Result<Flow, PhpError> {
        let subj = self.eval(subject)?;
        let mut start = None;
        for (i, c) in cases.iter().enumerate() {
            if let Some(test) = &c.test {
                let tv = self.eval(test)?;
                if ops::loose_eq(&subj, &tv) {
                    start = Some(i);
                    break;
                }
            }
        }
        // No matching case: fall back to `default`, wherever it sits.
        let start = match start.or_else(|| cases.iter().position(|c| c.test.is_none())) {
            Some(s) => s,
            None => return Ok(Flow::Normal),
        };
        for c in &cases[start..] {
            match self.exec_stmts(&c.body)? {
                Flow::Normal => {}
                // `break`/`continue` at this level both leave the switch
                // (a `switch` counts as one level for `continue`, per PHP).
                Flow::Break(1) | Flow::Continue(1) => return Ok(Flow::Normal),
                Flow::Break(n) => return Ok(Flow::Break(n - 1)),
                Flow::Continue(n) => return Ok(Flow::Continue(n - 1)),
                Flow::Return(v) => return Ok(Flow::Return(v)),
            }
        }
        Ok(Flow::Normal)
    }

    // --- user functions ---

    /// Invoke a hoisted user function: validate arity, set up a fresh local
    /// frame (its own slot table and slot names), bind parameters, run the body,
    /// then restore the caller's frame. Recursion uses the host (Rust) stack.
    fn call_user_fn(&mut self, idx: usize, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        // `funcs` is `&'p [FnDecl]` (Copy): copying it out detaches the borrow
        // from `self`, so the frame swap below can mutate `self.slots` freely.
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];

        let required = f.params.iter().filter(|p| p.default.is_none()).count();
        if argv.len() < required {
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}(), {} passed and {} expected",
                String::from_utf8_lossy(&f.name),
                argv.len(),
                expected,
            )));
        }

        let frame = vec![Zval::Undef; f.slots.len()];
        let saved_slots = std::mem::replace(&mut self.slots, frame);
        let saved_names = std::mem::replace(&mut self.names, f.slots.as_slice());

        let result = self.run_user_fn_body(f, argv);

        self.slots = saved_slots;
        self.names = saved_names;
        result
    }

    /// Bind parameters into the (already installed) callee frame and execute the
    /// body. A missing argument falls back to its default, evaluated in the new
    /// frame; falling off the end yields NULL.
    fn run_user_fn_body(&mut self, f: &'p FnDecl, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        for (i, p) in f.params.iter().enumerate() {
            let v = match argv.get(i) {
                Some(v) => v.clone(),
                // Required params are guaranteed present by the caller's check.
                None => self.eval(p.default.as_ref().expect("required arg checked"))?,
            };
            self.slots[p.slot as usize] = v;
        }
        match self.exec_stmts(&f.body)? {
            Flow::Return(v) => Ok(v),
            _ => Ok(Zval::Null),
        }
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
                // A user-defined function shadows the builtin namespace (PHP
                // resolves both from one function table; you cannot redefine a
                // builtin, but a user function wins when present).
                if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
                    return self.call_user_fn(idx, argv);
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

            ExprKind::Array(elems) => {
                let mut arr = PhpArray::new();
                for el in elems {
                    // PHP evaluates the key before the value.
                    match &el.key {
                        Some(ke) => {
                            let kv = self.eval(ke)?;
                            let key = self.coerce_key(&kv)?;
                            let v = self.eval(&el.value)?;
                            arr.insert(key, v);
                        }
                        None => {
                            let v = self.eval(&el.value)?;
                            arr.append(v).map_err(|_| array_occupied())?;
                        }
                    }
                }
                Ok(Zval::Array(Rc::new(arr)))
            }

            ExprKind::Index { base, index } => {
                let b = self.eval(base)?;
                let k = self.eval(index)?;
                self.read_index(&b, &k, false)
            }

            ExprKind::AssignPlace(place, rhs) => {
                // PHP evaluates the lvalue's offset expressions *before* the RHS
                // (so `$a[f()][g()] = h()` runs f, g, then h). Resolve the place
                // steps first to match — and stay consistent with AssignOpPlace.
                let steps = self.resolve_steps(place)?;
                let value = self.eval(rhs)?;
                self.write_place(place.slot, &steps, value.clone())?;
                Ok(value)
            }

            ExprKind::AssignOpPlace(op, place, rhs) => {
                let steps = self.resolve_steps(place)?;
                let cur = self.read_place_value(place.slot, &steps)?;
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.write_place(place.slot, &steps, res.clone())?;
                Ok(res)
            }

            ExprKind::AssignCoalescePlace(place, rhs) => {
                let steps = self.resolve_steps(place)?;
                match self.silent_get(place.slot, &steps) {
                    Some(v) if !matches!(v, Zval::Null | Zval::Undef) => Ok(v),
                    _ => {
                        let value = self.eval(rhs)?;
                        self.write_place(place.slot, &steps, value.clone())?;
                        Ok(value)
                    }
                }
            }

            ExprKind::Isset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    let set = matches!(self.silent_get(p.slot, &steps), Some(v) if !matches!(v, Zval::Null | Zval::Undef));
                    if !set {
                        return Ok(Zval::Bool(false));
                    }
                }
                Ok(Zval::Bool(true))
            }

            ExprKind::Empty(place) => {
                let steps = self.resolve_steps(place)?;
                let empty = match self.silent_get(place.slot, &steps) {
                    Some(v) => !convert::is_true_silent(&v),
                    None => true,
                };
                Ok(Zval::Bool(empty))
            }

            ExprKind::Match { subject, arms } => {
                let subj = self.eval(subject)?;
                let mut default_body = None;
                for arm in arms {
                    if arm.conditions.is_empty() {
                        default_body = Some(&arm.body);
                        continue;
                    }
                    for c in &arm.conditions {
                        let cv = self.eval(c)?;
                        if ops::identical(&subj, &cv) {
                            return self.eval(&arm.body);
                        }
                    }
                }
                match default_body {
                    Some(b) => self.eval(b),
                    None => Err(PhpError::Error(format!(
                        "Unhandled match case {}",
                        match_case_repr(&subj)
                    ))),
                }
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

    /// Read an expression in an isset-like context (the LHS of `??`): unset
    /// variables and missing array keys are silently treated as NULL, with no
    /// warning. Other expressions evaluate normally.
    fn eval_isset(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        match &e.kind {
            ExprKind::Var(slot) => {
                let v = self.slots[*slot as usize].clone();
                Ok(if matches!(v, Zval::Undef) { Zval::Null } else { v })
            }
            ExprKind::Index { base, index } => {
                let b = self.eval_isset(base)?;
                let k = self.eval(index)?;
                // Silent: an unset offset yields NULL so `??` falls through —
                // unlike a normal read, an out-of-range *string* offset is NOT
                // the empty string here (bug #69889).
                Ok(coalesce_index(&b, &k))
            }
            _ => self.eval(e),
        }
    }

    fn warn_undef(&mut self, slot: Slot) {
        let name = String::from_utf8_lossy(&self.names[slot as usize]);
        self.diags
            .push(Diag::Warning(format!("Undefined variable ${name}")));
    }

    // --- arrays ---

    /// Coerce a scalar to an array key, mirroring PHP's offset rules: int/bool
    /// stay integral, strings canonicalize (`"8"` → `Int(8)`), null becomes the
    /// `""` key, floats truncate (with a precision-loss deprecation), and an
    /// array offset is a `TypeError`.
    fn coerce_key(&mut self, v: &Zval) -> Result<Key, PhpError> {
        Ok(match v {
            Zval::Long(i) => Key::Int(*i),
            Zval::Bool(b) => Key::Int(*b as i64),
            Zval::Double(d) => {
                if d.fract() != 0.0 {
                    let repr = String::from_utf8_lossy(&dtoa::double_to_shortest(*d)).into_owned();
                    self.diags.push(Diag::Deprecated(format!(
                        "Implicit conversion from float {repr} to int loses precision"
                    )));
                }
                Key::Int(convert::dval_to_lval(*d))
            }
            Zval::Str(s) => Key::from_zstr(s),
            Zval::Null | Zval::Undef => Key::from_bytes(b""),
            Zval::Array(_) => {
                return Err(PhpError::TypeError(
                    "Illegal offset type".to_string(),
                ))
            }
        })
    }

    /// Read `base[key]`. `silent` suppresses the missing-key / wrong-type
    /// warnings (used on the LHS of `??` and inside `isset`).
    fn read_index(&mut self, base: &Zval, key: &Zval, silent: bool) -> Result<Zval, PhpError> {
        match base {
            Zval::Array(a) => {
                let k = self.coerce_key(key)?;
                match a.get(&k) {
                    Some(v) => Ok(v.clone()),
                    None => {
                        if !silent {
                            self.warn_undef_key(&k);
                        }
                        Ok(Zval::Null)
                    }
                }
            }
            Zval::Str(s) => self.read_string_offset(s, key, silent),
            other => {
                if !silent {
                    self.diags.push(Diag::Warning(format!(
                        "Trying to access array offset on value of type {}",
                        php_type_name(other)
                    )));
                }
                Ok(Zval::Null)
            }
        }
    }

    /// String offset read (`$s[i]`): integer index, negatives count from the
    /// end, out-of-range yields `""` (with a warning unless `silent`).
    fn read_string_offset(
        &mut self,
        s: &Rc<PhpStr>,
        key: &Zval,
        silent: bool,
    ) -> Result<Zval, PhpError> {
        if matches!(key, Zval::Array(_)) {
            return Err(PhpError::TypeError(
                "Cannot access offset of type array on string".to_string(),
            ));
        }
        let i = convert::to_long_cast(key, &mut self.diags);
        let len = s.len() as i64;
        let idx = if i < 0 { len + i } else { i };
        if idx < 0 || idx >= len {
            if !silent {
                self.diags
                    .push(Diag::Warning(format!("Uninitialized string offset {i}")));
            }
            Ok(Zval::Str(PhpStr::new(Vec::new())))
        } else {
            Ok(Zval::Str(PhpStr::new(vec![s.as_bytes()[idx as usize]])))
        }
    }

    fn warn_undef_key(&mut self, key: &Key) {
        let msg = match key {
            Key::Int(i) => format!("Undefined array key {i}"),
            Key::Str(s) => format!("Undefined array key \"{}\"", String::from_utf8_lossy(s.as_bytes())),
        };
        self.diags.push(Diag::Warning(msg));
    }

    /// Evaluate a place's index expressions into concrete steps (keys/append),
    /// before any mutation borrows the slot table.
    fn resolve_steps(&mut self, place: &Place) -> Result<Vec<Step>, PhpError> {
        let mut out = Vec::with_capacity(place.steps.len());
        for s in &place.steps {
            match s {
                PlaceStep::Index(e) => {
                    let v = self.eval(e)?;
                    out.push(Step::Key(self.coerce_key(&v)?));
                }
                PlaceStep::Append => out.push(Step::Append),
            }
        }
        Ok(out)
    }

    /// Write `value` to `slot` following the resolved steps, auto-vivifying
    /// intermediate arrays and copying-on-write shared ones.
    fn write_place(&mut self, slot: Slot, steps: &[Step], value: Zval) -> Result<(), PhpError> {
        if steps.is_empty() {
            self.slots[slot as usize] = value;
            return Ok(());
        }
        write_into(&mut self.slots[slot as usize], steps, value, &mut self.diags)
    }

    /// Read the current value at a place (for compound assignment). Missing
    /// keys yield NULL with a warning; the base variable is read silently
    /// (it is about to be written / auto-vivified anyway).
    fn read_place_value(&mut self, slot: Slot, steps: &[Step]) -> Result<Zval, PhpError> {
        let mut cur = match &self.slots[slot as usize] {
            Zval::Undef => Zval::Null,
            v => v.clone(),
        };
        for step in steps {
            let Step::Key(k) = step else {
                return Ok(Zval::Null);
            };
            cur = self.read_index(&cur, &key_to_zval(k), false)?;
        }
        Ok(cur)
    }

    /// Silent traversal used by `isset` / `empty` / `??=`: returns the value at
    /// the place if the whole path exists (value may be NULL), else `None`.
    fn silent_get(&self, slot: Slot, steps: &[Step]) -> Option<Zval> {
        let mut cur = match &self.slots[slot as usize] {
            Zval::Undef => return None,
            v => v.clone(),
        };
        for step in steps {
            let Step::Key(k) = step else { return None };
            match &cur {
                Zval::Array(a) => match a.get(k) {
                    Some(v) => cur = v.clone(),
                    None => return None,
                },
                Zval::Str(s) => {
                    // String offset isset: in-range integer key only.
                    match k {
                        Key::Int(i) => {
                            let len = s.len() as i64;
                            let idx = if *i < 0 { len + i } else { *i };
                            if idx < 0 || idx >= len {
                                return None;
                            }
                            cur = Zval::Str(PhpStr::new(vec![s.as_bytes()[idx as usize]]));
                        }
                        Key::Str(_) => return None,
                    }
                }
                _ => return None,
            }
        }
        Some(cur)
    }

    /// `unset($slot)` / `unset($a[k]...)`: drop a variable or array element.
    fn unset_place(&mut self, slot: Slot, steps: &[Step]) {
        if steps.is_empty() {
            self.slots[slot as usize] = Zval::Undef;
            return;
        }
        unset_into(&mut self.slots[slot as usize], steps);
    }
}

/// A resolved place step: an array key, or the append marker `[]`.
enum Step {
    Key(Key),
    Append,
}

/// Borrow `target` as a mutable array, auto-vivifying NULL/unset into a fresh
/// array (copy-on-write for shared arrays). A scalar value cannot be indexed:
/// a warning is raised and `None` returned so the caller aborts the write.
fn ensure_array_mut<'a>(target: &'a mut Zval, diags: &mut Diags) -> Option<&'a mut PhpArray> {
    match target {
        Zval::Null | Zval::Undef => {
            *target = Zval::Array(Rc::new(PhpArray::new()));
        }
        Zval::Array(_) => {}
        _ => {
            diags.push(Diag::Warning(
                "Cannot use a scalar value as an array".to_string(),
            ));
            return None;
        }
    }
    match target {
        Zval::Array(rc) => Some(Rc::make_mut(rc)),
        _ => unreachable!("target was just normalised to an array"),
    }
}

/// Recursively write `value` into `target` following `steps`.
fn write_into(
    target: &mut Zval,
    steps: &[Step],
    value: Zval,
    diags: &mut Diags,
) -> Result<(), PhpError> {
    let Some((first, rest)) = steps.split_first() else {
        *target = value;
        return Ok(());
    };
    let Some(arr) = ensure_array_mut(target, diags) else {
        return Ok(());
    };
    match first {
        Step::Key(k) => {
            if rest.is_empty() {
                arr.insert(k.clone(), value);
            } else {
                if !arr.contains_key(k) {
                    arr.insert(k.clone(), Zval::Array(Rc::new(PhpArray::new())));
                }
                let child = arr.get_mut(k).expect("key just inserted");
                write_into(child, rest, value, diags)?;
            }
        }
        Step::Append => {
            if rest.is_empty() {
                arr.append(value).map_err(|_| array_occupied())?;
            } else {
                let mut child = Zval::Array(Rc::new(PhpArray::new()));
                write_into(&mut child, rest, value, diags)?;
                arr.append(child).map_err(|_| array_occupied())?;
            }
        }
    }
    Ok(())
}

/// Recursively `unset` the element addressed by `steps`. A missing path is a
/// silent no-op (PHP semantics).
fn unset_into(target: &mut Zval, steps: &[Step]) {
    let (first, rest) = match steps.split_first() {
        Some(p) => p,
        None => return,
    };
    let Step::Key(k) = first else { return };
    if let Zval::Array(rc) = target {
        if rest.is_empty() {
            Rc::make_mut(rc).remove(k);
        } else if let Some(child) = Rc::make_mut(rc).get_mut(k) {
            unset_into(child, rest);
        }
    }
}

/// Silent `base[key]` read for the LHS of `??`: returns NULL when the offset is
/// not set, so the coalesce falls through. No warnings, no empty-string for an
/// out-of-range string offset (bug #69889). Mirrors PHP's `isset`-style rules.
fn coalesce_index(base: &Zval, key: &Zval) -> Zval {
    match base {
        Zval::Array(a) => match coerce_key_silent(key) {
            Some(k) => a.get(&k).cloned().unwrap_or(Zval::Null),
            None => Zval::Null,
        },
        Zval::Str(s) => match string_offset_silent(key) {
            Some(i) => {
                let len = s.len() as i64;
                let idx = if i < 0 { len + i } else { i };
                if idx < 0 || idx >= len {
                    Zval::Null
                } else {
                    Zval::Str(PhpStr::new(vec![s.as_bytes()[idx as usize]]))
                }
            }
            None => Zval::Null,
        },
        _ => Zval::Null,
    }
}

/// Coerce a value to an array key without diagnostics (for silent contexts).
/// An array offset is illegal → `None` (treated as not-set).
fn coerce_key_silent(v: &Zval) -> Option<Key> {
    match v {
        Zval::Long(i) => Some(Key::Int(*i)),
        Zval::Bool(b) => Some(Key::Int(*b as i64)),
        Zval::Double(d) => Some(Key::Int(convert::dval_to_lval(*d))),
        Zval::Str(s) => Some(Key::from_zstr(s)),
        Zval::Null | Zval::Undef => Some(Key::from_bytes(b"")),
        Zval::Array(_) => None,
    }
}

/// The integer offset of a string subscript in a silent context, or `None` when
/// the key is not a valid (integer-like) string offset — a non-numeric string
/// key is *not set*, so `isset`/`??` see it as absent.
fn string_offset_silent(v: &Zval) -> Option<i64> {
    match v {
        Zval::Long(i) => Some(*i),
        Zval::Bool(b) => Some(*b as i64),
        Zval::Double(d) => Some(convert::dval_to_lval(*d)),
        Zval::Str(s) => match Key::from_bytes(s.as_bytes()) {
            Key::Int(i) => Some(i),
            Key::Str(_) => None,
        },
        _ => None,
    }
}

/// An array key as a `Zval` (for `foreach` key binding and place re-reads).
fn key_to_zval(k: &Key) -> Zval {
    match k {
        Key::Int(i) => Zval::Long(*i),
        Key::Str(s) => Zval::Str(Rc::clone(s)),
    }
}

/// Lowercase type name used in runtime warning messages (distinct from
/// `gettype`'s capitalised names).
fn php_type_name(v: &Zval) -> &'static str {
    match v {
        Zval::Undef | Zval::Null => "null",
        Zval::Bool(_) => "bool",
        Zval::Long(_) => "int",
        Zval::Double(_) => "float",
        Zval::Str(_) => "string",
        Zval::Array(_) => "array",
    }
}

/// The subject rendering in an `UnhandledMatchError` message (PHP quotes
/// strings, prints scalars bare).
fn match_case_repr(v: &Zval) -> String {
    match v {
        Zval::Long(i) => i.to_string(),
        Zval::Bool(true) => "true".to_string(),
        Zval::Bool(false) => "false".to_string(),
        Zval::Null | Zval::Undef => "NULL".to_string(),
        Zval::Double(d) => String::from_utf8_lossy(&dtoa::double_to_shortest(*d)).into_owned(),
        Zval::Str(s) => format!("'{}'", String::from_utf8_lossy(s.as_bytes())),
        Zval::Array(_) => "of type array".to_string(),
    }
}

fn array_occupied() -> PhpError {
    PhpError::Error(
        "Cannot add element to the array as the next element is already occupied".to_string(),
    )
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
