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

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use php_types::{convert, dtoa, numstr, ops, Diag, Diags, Key, PhpArray, PhpStr, PhpError, Zval};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::hir::{
    BinOp, CastKind, Expr, ExprKind, FnDecl, Line, Param, Place, PlaceBase, PlaceStep, Program,
    ScalarType, Slot, Stmt, StmtKind, TypeHint, UnOp,
};

/// A fresh frame of `n` independent value slots, all unset.
///
/// A slot is a plain [`Zval`]; a reference is a `Zval::Ref` holding a shared
/// `Rc<RefCell<Zval>>` (step 11d unified the slot-level binding of steps 11a–c
/// with element-level references onto the single `Zval::Ref` representation,
/// D-R10). All aliases of one reference share the cell, so a write through any
/// of them is visible to all (write-through, D-R3).
fn fresh_slots(n: usize) -> Vec<Zval> {
    vec![Zval::Undef; n]
}

/// Mutable view of the *active* frame's value slots: the callee's locals while a
/// user function runs, otherwise the script globals (D-12.1). Deliberately a
/// macro, not a method: it expands to a borrow that touches only the `locals`
/// and `globals` fields, so sibling fields (notably `diags`) stay independently
/// borrowable at the call site — a `frame_mut(&mut self)` method would borrow
/// *all* of `self` and conflict with a concurrently-held `&mut self.diags`.
macro_rules! frame_mut {
    ($self:ident) => {
        $self.locals.as_mut().unwrap_or(&mut $self.globals)
    };
}

/// `&mut Zval` for a place base: the active frame for a `Local` slot, the global
/// frame for a `$GLOBALS['literal']` `Global` slot (D-12.3). A macro for the
/// same disjoint-borrow reason as [`frame_mut!`].
macro_rules! slot_mut {
    ($self:ident, $base:expr) => {
        match $base {
            PlaceBase::Local(s) => &mut frame_mut!($self)[s as usize],
            PlaceBase::Global(s) => &mut $self.globals[s as usize],
        }
    };
}

/// A user-function argument as bound for a call: a plain value for a by-value
/// parameter, or a shared cell for a `&$x` by-reference parameter (D-R6).
enum Arg {
    Val(Zval),
    Ref(Rc<RefCell<Zval>>),
}

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
    /// Bytes written by `echo` / inline HTML / builtins, in order. This is the
    /// *pure* program output: diagnostics are not interleaved here (use
    /// [`Outcome::rendered`] for that).
    pub stdout: Vec<u8>,
    /// The CLI-faithful output stream (step 9): `stdout` with diagnostics and an
    /// uncaught fatal rendered *inline at their point of occurrence*, exactly as
    /// PHP's CLI SAPI emits them under `display_errors=1, html_errors=0`. This is
    /// the stream a `.phpt` `--EXPECT(F)--` section is compared against.
    pub rendered: Vec<u8>,
    /// Non-fatal diagnostics raised during execution, in order (side channel for
    /// fine-grained assertions; also rendered into [`Outcome::rendered`]).
    pub diags: Diags,
    /// An uncaught fatal error that aborted execution, if any (also rendered at
    /// the tail of [`Outcome::rendered`]).
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
        global_names: &program.slots,
        local_names: None,
        reg: registry,
        funcs: &program.functions,
        fn_index: &fn_index,
        file: &program.file,
        globals: fresh_slots(program.slots.len()),
        locals: None,
        fn_returns_ref: false,
        statics: vec![None; program.static_count],
        out: Vec::new(),
        rendered: Vec::new(),
        diags: Vec::new(),
        diags_rendered: 0,
        cur_line: 1,
    };

    let (fatal, return_value) = match ev.exec_stmts(&program.body) {
        Ok(Flow::Return(v)) => (None, v),
        Ok(_) => (None, Zval::Null),
        Err(e) => (Some(e), Zval::Null),
    };

    // Render any diagnostics still staged (defensive; statement/expression exits
    // already flush), then the uncaught fatal at the tail of the stream.
    ev.flush_diags();
    if let Some(err) = &fatal {
        ev.render_fatal(err);
    }

    Outcome {
        stdout: ev.out,
        rendered: ev.rendered,
        diags: ev.diags,
        fatal,
        return_value,
    }
}

struct Evaluator<'p> {
    /// Slot names for the global frame and (while a user function runs) the
    /// callee's local frame — used only for undefined-variable warnings. The
    /// active set is chosen by [`Evaluator::names`] (D-12.1).
    global_names: &'p [Box<[u8]>],
    local_names: Option<&'p [Box<[u8]>]>,
    reg: &'p Registry,
    /// Hoisted user functions and their name→index map (built in `run_with`).
    funcs: &'p [FnDecl],
    fn_index: &'p HashMap<Vec<u8>, usize>,
    /// Script file name, reproduced in rendered diagnostics (`... in <file>`).
    file: &'p [u8],
    /// The script-global frame (always present) and the active local overlay
    /// (`Some` while a user function runs). The active frame is reached by
    /// [`frame_mut!`] / [`Evaluator::frame`]; this is the only structural change
    /// of step 12-1 and the mechanism that lets `global $x` / `$GLOBALS` reach
    /// the global frame *by slot* from inside a function (D-12.1).
    globals: Vec<Zval>,
    locals: Option<Vec<Zval>>,
    /// True while the body of a `function &f()` runs: a plain `StmtKind::Return`
    /// (i.e. a non-lvalue or bare `return;`) then raises the by-ref-return Notice
    /// (step 13-2, D-13.4). Set/restored by `call_user_fn` like `locals`.
    fn_returns_ref: bool,
    /// Persistent `static` variable cells, indexed by each declaration's unique
    /// id; `None` until first initialised. Survives across calls for the whole
    /// run, giving `static $x` its persistence and cross-recursion sharing
    /// (step 15, D-15.1).
    statics: Vec<Option<Rc<RefCell<Zval>>>>,
    /// Pure program output (echo / inline HTML / builtins).
    out: Vec<u8>,
    /// The interleaved CLI stream: `out` plus diagnostics rendered at their point
    /// of occurrence. Built incrementally alongside `out` (see `emit`).
    rendered: Vec<u8>,
    /// All diagnostics raised, in order (the side channel). Leaf functions in
    /// `php_types` push here; `flush_diags` renders the not-yet-rendered tail
    /// into `rendered`, tracked by `diags_rendered`.
    diags: Diags,
    diags_rendered: usize,
    /// 1-based source line of the node currently executing, stamped onto every
    /// rendered diagnostic and the uncaught-fatal location. Updated at the top of
    /// `eval` / `exec_stmt`; on the error path it is intentionally *not* restored,
    /// so it still points at the throwing node when the fatal is rendered.
    cur_line: Line,
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
    // --- diagnostic rendering (step 9) ---

    /// Append `bytes` to the program output, first flushing any pending
    /// diagnostics so they land *before* the output they precede (PHP renders a
    /// diagnostic at the moment it is raised, ahead of the value being printed).
    fn emit(&mut self, bytes: &[u8]) {
        self.flush_diags();
        self.out.extend_from_slice(bytes);
        self.rendered.extend_from_slice(bytes);
    }

    /// Render every diagnostic raised since the last flush into `rendered`,
    /// stamped with the current line and file:
    /// `\n{Severity}: {message} in {file} on line {N}\n` (`main/main.c:1493`).
    fn flush_diags(&mut self) {
        while self.diags_rendered < self.diags.len() {
            let d = &self.diags[self.diags_rendered];
            let header = format!("\n{}: {} in ", d.severity(), d.message());
            self.rendered.extend_from_slice(header.as_bytes());
            self.rendered.extend_from_slice(self.file);
            let tail = format!(" on line {}\n", self.cur_line);
            self.rendered.extend_from_slice(tail.as_bytes());
            self.diags_rendered += 1;
        }
    }

    /// Render an uncaught fatal at the tail of `rendered`, matching the CLI
    /// display of an uncaught throwable (`Zend/zend_exceptions.c:756`). The stack
    /// trace is the top-level `#0 {main}` form; frames for fatals thrown inside
    /// user calls are not modelled (step 9 scope).
    fn render_fatal(&mut self, err: &PhpError) {
        let file = String::from_utf8_lossy(self.file);
        let block = format!(
            "\nFatal error: Uncaught {}: {} in {}:{}\nStack trace:\n#0 {{main}}\n  thrown in {} on line {}\n",
            err.class_name(),
            err.message(),
            file,
            self.cur_line,
            file,
            self.cur_line,
        );
        self.rendered.extend_from_slice(block.as_bytes());
    }

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

    /// Set the current line for diagnostics, run the statement, then flush any
    /// diagnostics it staged. On the error path `cur_line` is left pointing at the
    /// throwing node (see the field doc) for the fatal renderer.
    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<Flow, PhpError> {
        self.cur_line = stmt.line;
        let r = self.exec_stmt_inner(stmt);
        self.flush_diags();
        r
    }

    fn exec_stmt_inner(&mut self, stmt: &Stmt) -> Result<Flow, PhpError> {
        match &stmt.kind {
            StmtKind::Nop => {}

            StmtKind::InlineHtml(bytes) => self.emit(bytes),

            StmtKind::Echo(values) => {
                for e in values {
                    let z = self.eval(e)?;
                    let s = convert::to_zstr(&z, &mut self.diags);
                    // `emit` flushes the (possible) array-to-string warning ahead
                    // of the converted bytes, matching PHP's ordering.
                    self.emit(s.as_bytes());
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
                by_ref,
                body,
            } => return self.exec_foreach(iter, *key, *value, *by_ref, body),

            StmtKind::Switch { subject, cases } => return self.exec_switch(subject, cases),

            StmtKind::Unset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    self.unset_place(p.base, &steps);
                }
            }

            StmtKind::Global(bindings) => {
                // At global scope the named variable *is* the global, so `global`
                // is a no-op (no separate local frame to alias into). Inside a
                // function, alias each global cell into the local slot via a
                // shared `Zval::Ref`, promoting the global to a cell on first use
                // — this reuses the step 11d reference machinery (D-12.2). An
                // undefined global is promoted to a NULL cell, so a later write
                // through the alias *creates* the global (D-12.4).
                if self.locals.is_some() {
                    for b in bindings {
                        let cell = make_cell(&mut self.globals[b.global as usize]);
                        frame_mut!(self)[b.local as usize] = Zval::Ref(cell);
                    }
                }
            }

            StmtKind::StaticVar(bindings) => {
                // Alias each local slot to its persistent cell, creating and
                // initialising the cell on the first execution only (D-15.4).
                for b in bindings {
                    if self.statics[b.id].is_none() {
                        let init = match &b.init {
                            Some(e) => self.eval(e)?,
                            None => Zval::Null,
                        };
                        self.statics[b.id] = Some(Rc::new(RefCell::new(init)));
                    }
                    let cell = Rc::clone(self.statics[b.id].as_ref().unwrap());
                    frame_mut!(self)[b.slot as usize] = Zval::Ref(cell);
                }
            }

            StmtKind::Break(n) => return Ok(Flow::Break(*n)),
            StmtKind::Continue(n) => return Ok(Flow::Continue(*n)),
            StmtKind::Return(opt) => {
                // A plain return inside a `function &f()` means the operand was a
                // non-lvalue (or `return;`): PHP raises a Notice and falls back to
                // returning by value (D-13.4).
                if self.fn_returns_ref {
                    self.diags.push(Diag::Notice(
                        "Only variable references should be returned by reference".to_string(),
                    ));
                }
                let v = match opt {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                return Ok(Flow::Return(v));
            }
            StmtKind::ReturnRef(place) => {
                // Return a *reference* to the place: promote it to a shared cell
                // (reusing the step 11d/12 machinery) and hand the cell back as a
                // `Zval::Ref`, which `$y = &f()` aliases (D-13.2).
                let steps = self.resolve_steps(place)?;
                let cell = self.ref_source_cell(place.base, &steps)?;
                return Ok(Flow::Return(Zval::Ref(cell)));
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
        by_ref: bool,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        // A by-reference loop binds each element of the *source variable* in
        // place (step 11d-3). Over a non-variable it would have nothing to write
        // back to, so it degrades to by-value iteration (PHP tolerates this).
        if by_ref {
            if let ExprKind::Var(slot) = iter.kind {
                return self.exec_foreach_by_ref(slot, key, value, body);
            }
        }
        let collection = self.eval(iter)?;
        // Snapshot raw element clones: a plain value is frozen for the loop, but a
        // reference element keeps sharing its cell, so its value is read live at
        // bind time (this is what makes the lingering-reference gotcha work).
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
                self.slot_set(ks as usize, key_to_zval(&k));
            }
            self.slot_set(value as usize, v.deref_clone());
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
        }
        Ok(Flow::Normal)
    }

    /// `foreach ($var as [$k =>] &$value)`: bind each element of the source
    /// variable's array by reference (D-R13). The keys are snapshotted once; each
    /// element is promoted to a `Zval::Ref` and the value slot aliases its cell,
    /// so body writes land in the array. The value slot is intentionally *not*
    /// reset afterwards — it lingers as a reference to the last element (the PHP
    /// gotcha, oracle-verified).
    fn exec_foreach_by_ref(
        &mut self,
        slot: Slot,
        key: Option<Slot>,
        value: Slot,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        let keys: Vec<Key> = match self.slot_clone(slot as usize) {
            Zval::Array(a) => a.iter().map(|(k, _)| k.clone()).collect(),
            other => {
                self.diags.push(Diag::Warning(format!(
                    "foreach() argument must be of type array|object, {} given",
                    php_type_name(&other)
                )));
                return Ok(Flow::Normal);
            }
        };
        for k in keys {
            let step = [Step::Key(k.clone())];
            let cell = {
                let d = &mut self.diags;
                place_cell(&mut frame_mut!(self)[slot as usize], &step, d)?
            };
            if let Some(ks) = key {
                self.slot_set(ks as usize, key_to_zval(&k));
            }
            frame_mut!(self)[value as usize] = Zval::Ref(Rc::clone(&cell));
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
    fn call_user_fn(&mut self, idx: usize, argv: Vec<Arg>) -> Result<Zval, PhpError> {
        // `funcs` is `&'p [FnDecl]` (Copy): copying it out detaches the borrow
        // from `self`, so installing the local overlay below can mutate the
        // active frame freely.
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

        // Install the callee's local frame as the overlay; the global frame
        // stays put so `global $x` / `$GLOBALS` can reach it by slot (D-12.1).
        // Saving and restoring the previous overlay makes nested calls nest.
        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);

        let result = self.run_user_fn_body(f, argv);

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        result
    }

    /// Bind parameters into the (already installed) callee frame and execute the
    /// body. A by-value argument installs a fresh value slot; a by-reference
    /// argument shares the caller's cell (D-R6). A missing argument falls back to
    /// its default, evaluated in the new frame; falling off the end yields NULL.
    fn run_user_fn_body(&mut self, f: &'p FnDecl, argv: Vec<Arg>) -> Result<Zval, PhpError> {
        for (i, p) in f.params.iter().enumerate() {
            let binding = match argv.get(i) {
                // A by-value argument is coerced to the parameter's scalar hint
                // under weak typing; a failure is an uncaught TypeError (D-14.4).
                // By-reference arguments and defaults are bound as-is.
                Some(Arg::Val(v)) => {
                    let val = v.clone();
                    match &p.hint {
                        Some(hint) => match coerce_to_hint(val, hint, &mut self.diags) {
                            Ok(c) => c,
                            Err(given) => return Err(self.arg_type_error(f, i, p, hint, given)),
                        },
                        None => val,
                    }
                }
                Some(Arg::Ref(cell)) => Zval::Ref(Rc::clone(cell)),
                // Required params are guaranteed present by the caller's check.
                // A default is coerced to the hint too (`float $n = 0` → 0.0,
                // D-NEW-6); a valid constant default always coerces, so on the
                // unreachable failure we keep the evaluated value.
                None => {
                    let v = self.eval(p.default.as_ref().expect("required arg checked"))?;
                    match &p.hint {
                        Some(hint) => coerce_to_hint(v.clone(), hint, &mut self.diags).unwrap_or(v),
                        None => v,
                    }
                }
            };
            frame_mut!(self)[p.slot as usize] = binding;
        }
        let ret = match self.exec_stmts(&f.body)? {
            Flow::Return(v) => v,
            _ => Zval::Null,
        };
        // Coerce the return value to a scalar return type (weak). A by-reference
        // function returns a `Zval::Ref` to alias, so its return type stays
        // unenforced here (scope-out, D-14.5/D-13.7).
        match &f.ret_hint {
            Some(hint) if !f.by_ref => match coerce_to_hint(ret, hint, &mut self.diags) {
                Ok(v) => Ok(v),
                Err(given) => Err(self.return_type_error(f, hint, given)),
            },
            _ => Ok(ret),
        }
    }

    /// Build the uncaught `TypeError` for a return value that failed scalar
    /// coercion (D-14.5). The message format differs from the argument one: no
    /// call site, suffix `returned in <file>:<defline>`.
    fn return_type_error(&self, f: &FnDecl, hint: &TypeHint, given: &str) -> PhpError {
        PhpError::TypeError(format!(
            "{}(): Return value must be of type {}, {} returned in {}:{}",
            String::from_utf8_lossy(&f.name),
            hint.display_name(),
            given,
            String::from_utf8_lossy(self.file),
            f.line,
        ))
    }

    /// Build the uncaught `TypeError` for an argument that failed scalar
    /// coercion, matching PHP's exact message (D-14.4). `cur_line` is the call
    /// site's line; `f.line` is the definition line.
    fn arg_type_error(&self, f: &FnDecl, i: usize, p: &Param, hint: &TypeHint, given: &str) -> PhpError {
        let file = String::from_utf8_lossy(self.file);
        PhpError::TypeError(format!(
            "{}(): Argument #{} (${}) must be of type {}, {} given, \
             called in {} on line {} and defined in {}:{}",
            String::from_utf8_lossy(&f.name),
            i + 1,
            String::from_utf8_lossy(&f.slots[p.slot as usize]),
            hint.display_name(),
            given,
            file,
            self.cur_line,
            file,
            f.line,
        ))
    }

    /// Resolve a user-function call's arguments against its declaration: by-value
    /// params evaluate normally; a `&$x` param binds the argument variable's
    /// shared cell (promoting it). A non-variable argument to a by-ref param is
    /// an uncaught `Error` (PHP 8.x; oracle-verified message).
    fn eval_call_args(&mut self, idx: usize, args: &[Expr]) -> Result<Vec<Arg>, PhpError> {
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];
        let mut out = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let by_ref = f.params.get(i).is_some_and(|p| p.by_ref);
            if by_ref {
                match &a.kind {
                    ExprKind::Var(slot) => out.push(Arg::Ref(self.slot_cell(*slot as usize))),
                    _ => {
                        let p = &f.params[i];
                        return Err(PhpError::Error(format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(&f.name),
                            i + 1,
                            String::from_utf8_lossy(&f.slots[p.slot as usize]),
                        )));
                    }
                }
            } else {
                out.push(Arg::Val(self.eval(a)?));
            }
        }
        Ok(out)
    }

    /// Invoke a by-reference builtin (step 11c). Its first argument must be a
    /// variable: that variable's storage cell is bound and handed to the builtin
    /// as `&mut Zval`, so the builtin's mutation writes through to the caller.
    /// The remaining arguments are evaluated by value. A missing or non-variable
    /// first argument raises the shared `$array`-family errors (oracle-verified).
    fn call_ref_builtin(
        &mut self,
        f: BuiltinRefFn,
        name: &[u8],
        args: &[Expr],
    ) -> Result<Zval, PhpError> {
        let Some((first, rest_exprs)) = args.split_first() else {
            return Err(PhpError::ArgumentCountError(format!(
                "{}() expects at least 1 argument, 0 given",
                String::from_utf8_lossy(name)
            )));
        };
        let ExprKind::Var(slot) = first.kind else {
            return Err(PhpError::Error(format!(
                "{}(): Argument #1 ($array) could not be passed by reference",
                String::from_utf8_lossy(name)
            )));
        };
        // Evaluate the by-value tail before binding the cell (binding a variable
        // has no side effect, so this preserves left-to-right argument order).
        let mut rest = Vec::with_capacity(rest_exprs.len());
        for a in rest_exprs {
            rest.push(self.eval(a)?);
        }
        let cell = self.slot_cell(slot as usize);
        let mut guard = cell.borrow_mut();
        let target = &mut *guard;
        // Like value builtins: flush pending diagnostics, run, mirror fresh
        // output into `rendered`, then flush the builtin's own diagnostics.
        self.flush_diags();
        let pre = self.out.len();
        let mut ctx = Ctx {
            out: &mut self.out,
            diags: &mut self.diags,
        };
        let res = f(target, &rest, &mut ctx);
        let produced = self.out[pre..].to_vec();
        self.rendered.extend_from_slice(&produced);
        self.flush_diags();
        res
    }

    // --- expressions ---

    /// Evaluate `e`, stamping its line for any diagnostics it raises and flushing
    /// them into `rendered` before the value flows to its consumer. On success the
    /// enclosing line is restored; on the error path it is kept at the throwing
    /// node so the top-level fatal renderer reports the right location.
    fn eval(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        let prev = self.cur_line;
        self.cur_line = e.line;
        let r = self.eval_inner(e);
        self.flush_diags();
        if r.is_ok() {
            self.cur_line = prev;
        }
        r
    }

    fn eval_inner(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        match &e.kind {
            ExprKind::Null => Ok(Zval::Null),
            ExprKind::Bool(b) => Ok(Zval::Bool(*b)),
            ExprKind::Int(i) => Ok(Zval::Long(*i)),
            ExprKind::Float(f) => Ok(Zval::Double(*f)),
            ExprKind::Str(bytes) => Ok(Zval::Str(PhpStr::new(bytes.clone()))),

            ExprKind::Var(slot) => Ok(self.read_var(*slot)),
            ExprKind::GlobalVar(slot) => Ok(self.read_global_var(*slot)),

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
                self.slot_set(*slot as usize, v.clone());
                Ok(v)
            }

            ExprKind::AssignRef { target, source } => self.assign_ref(target, source),
            ExprKind::AssignRefCall { target, call } => self.assign_ref_call(target, call),

            ExprKind::AssignOp(op, slot, rhs) => {
                let cur = self.read_var(*slot);
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.slot_set(*slot as usize, res.clone());
                Ok(res)
            }

            ExprKind::AssignCoalesce(slot, rhs) => {
                let cur = self.slot_clone(*slot as usize);
                if matches!(cur, Zval::Null | Zval::Undef) {
                    let v = self.eval(rhs)?;
                    self.slot_set(*slot as usize, v.clone());
                    Ok(v)
                } else {
                    Ok(cur)
                }
            }

            ExprKind::IncDec { slot, inc, pre } => {
                let idx = *slot as usize;
                if matches!(self.slot_clone(idx), Zval::Undef) {
                    self.warn_undef(*slot);
                    self.slot_set(idx, Zval::Null);
                }
                let old = self.slot_clone(idx);
                // `increment`/`decrement` follow a `Zval::Ref` themselves, so the
                // mutation writes through any alias without a separate match here.
                let d = &mut self.diags;
                let target = &mut frame_mut!(self)[idx];
                if *inc {
                    ops::increment(target, d)?;
                } else {
                    ops::decrement(target, d)?;
                }
                Ok(if *pre { self.slot_clone(idx) } else { old })
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
                // A user-defined function shadows the builtin namespace (PHP
                // resolves both from one function table; you cannot redefine a
                // builtin, but a user function wins when present). User functions
                // bind by-reference parameters (step 11b), so their arguments are
                // resolved against the declaration rather than blindly evaluated.
                if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
                    let argv = self.eval_call_args(idx, args)?;
                    let result = self.call_user_fn(idx, argv)?;
                    // A by-reference function returns a `Zval::Ref`; in this
                    // (value) context it must be copied, not aliased — only
                    // `$y = &f()` keeps the cell (D-13.6).
                    return Ok(match result {
                        Zval::Ref(cell) => cell.borrow().clone(),
                        other => other,
                    });
                }
                // A by-reference builtin (array_push/sort/...) binds its first
                // argument's variable cell rather than a copy, so it is handled
                // before the arguments are evaluated by value (step 11c).
                if let Some(Builtin::RefFirst(f)) = self.reg.get(name.as_ref()).copied() {
                    return self.call_ref_builtin(f, name, args);
                }
                // Builtins otherwise take all arguments by value. Copy the fn
                // pointer out so the registry borrow ends before we borrow
                // `out`/`diags` mutably for the call context.
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                let f = match self.reg.get(name.as_ref()) {
                    Some(Builtin::Value(f)) => *f,
                    Some(Builtin::RefFirst(_)) => unreachable!("handled above"),
                    None => {
                        return Err(PhpError::Error(format!(
                            "Call to undefined function {}()",
                            String::from_utf8_lossy(name)
                        )))
                    }
                };
                // A builtin writes straight to `out` and may stage diagnostics.
                // Flush anything pending first, run it, then mirror its fresh
                // output into `rendered` and flush its own diagnostics after it
                // (builtins emit output then warn, never the reverse).
                self.flush_diags();
                let pre = self.out.len();
                let mut ctx = Ctx {
                    out: &mut self.out,
                    diags: &mut self.diags,
                };
                let res = f(&argv, &mut ctx);
                let produced = self.out[pre..].to_vec();
                self.rendered.extend_from_slice(&produced);
                self.flush_diags();
                res
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
                self.write_place(place.base, &steps, value.clone())?;
                Ok(value)
            }

            ExprKind::AssignOpPlace(op, place, rhs) => {
                let steps = self.resolve_steps(place)?;
                let cur = self.read_place_value(place.base, &steps)?;
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.write_place(place.base, &steps, res.clone())?;
                Ok(res)
            }

            ExprKind::AssignCoalescePlace(place, rhs) => {
                let steps = self.resolve_steps(place)?;
                match self.silent_get(place.base, &steps) {
                    Some(v) if !matches!(v, Zval::Null | Zval::Undef) => Ok(v),
                    _ => {
                        let value = self.eval(rhs)?;
                        self.write_place(place.base, &steps, value.clone())?;
                        Ok(value)
                    }
                }
            }

            ExprKind::Isset(places) => {
                for p in places {
                    let steps = self.resolve_steps(p)?;
                    let set = matches!(self.silent_get(p.base, &steps), Some(v) if !matches!(v, Zval::Null | Zval::Undef));
                    if !set {
                        return Ok(Zval::Bool(false));
                    }
                }
                Ok(Zval::Bool(true))
            }

            ExprKind::Empty(place) => {
                let steps = self.resolve_steps(place)?;
                let empty = match self.silent_get(place.base, &steps) {
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

    /// Read-only view of the active frame's value slots (see [`frame_mut!`]).
    fn frame(&self) -> &[Zval] {
        self.locals.as_deref().unwrap_or(&self.globals)
    }

    /// Slot names for the active frame (callee locals while a user function
    /// runs, else the script globals). The references are `'p`-lived, so this
    /// borrows nothing of `self` (D-12.1).
    fn names(&self) -> &'p [Box<[u8]>] {
        self.local_names.unwrap_or(self.global_names)
    }

    /// The current value held by a slot, dereferencing a reference (D-R2: reads
    /// are always by value, preserving copy semantics).
    fn slot_clone(&self, idx: usize) -> Zval {
        self.frame()[idx].deref_clone()
    }

    /// Like [`Evaluator::slot_clone`] but for a place base: reads the active
    /// frame for a `Local` slot, the global frame for a `Global` slot (D-12.3).
    fn base_clone(&self, base: PlaceBase) -> Zval {
        match base {
            PlaceBase::Local(s) => self.frame()[s as usize].deref_clone(),
            PlaceBase::Global(s) => self.globals[s as usize].deref_clone(),
        }
    }

    /// Write `v` into a slot. For a plain value this replaces it; for a
    /// reference it writes *through* the shared cell so every alias sees the new
    /// value (D-R3 write-through).
    fn slot_set(&mut self, idx: usize, v: Zval) {
        match &mut frame_mut!(self)[idx] {
            Zval::Ref(cell) => *cell.borrow_mut() = v,
            slot => *slot = v,
        }
    }

    /// Read a variable slot. An unset slot raises "Undefined variable $name"
    /// and yields NULL (the runtime equivalent of HIR `Var` access).
    fn read_var(&mut self, slot: Slot) -> Zval {
        match self.slot_clone(slot as usize) {
            Zval::Undef => {
                self.warn_undef(slot);
                Zval::Null
            }
            v => v,
        }
    }

    /// Read an expression in an isset-like context (the LHS of `??`): unset
    /// variables and missing array keys are silently treated as NULL, with no
    /// warning. Other expressions evaluate normally.
    fn eval_isset(&mut self, e: &Expr) -> Result<Zval, PhpError> {
        match &e.kind {
            ExprKind::Var(slot) => {
                let v = self.slot_clone(*slot as usize);
                Ok(if matches!(v, Zval::Undef) { Zval::Null } else { v })
            }
            ExprKind::GlobalVar(slot) => {
                let v = self.globals[*slot as usize].deref_clone();
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
        let name = String::from_utf8_lossy(&self.names()[slot as usize]);
        self.diags
            .push(Diag::Warning(format!("Undefined variable ${name}")));
    }

    /// Read a `$GLOBALS['x']` global slot. An unset global raises the distinct
    /// "Undefined global variable $name" warning (its name lives in the global
    /// table, not the active frame's) and yields NULL (D-12.3/D-12.5).
    fn read_global_var(&mut self, slot: Slot) -> Zval {
        match self.globals[slot as usize].deref_clone() {
            Zval::Undef => {
                let name = String::from_utf8_lossy(&self.global_names[slot as usize]);
                self.diags
                    .push(Diag::Warning(format!("Undefined global variable ${name}")));
                Zval::Null
            }
            v => v,
        }
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
            Zval::Null | Zval::Undef => {
                // PHP 8.1+: using null as an array offset is deprecated; it still
                // resolves to the empty-string key. (The `??`/`isset` paths go
                // through `coalesce_index`, which stays silent — step 9 scope.)
                self.diags.push(Diag::Deprecated(
                    "Using null as an array offset is deprecated, use an empty string instead"
                        .to_string(),
                ));
                Key::from_bytes(b"")
            }
            Zval::Array(_) => {
                return Err(PhpError::TypeError(
                    "Illegal offset type".to_string(),
                ))
            }
            Zval::Ref(c) => return self.coerce_key(&c.borrow()),
        })
    }

    /// Read `base[key]`. `silent` suppresses the missing-key / wrong-type
    /// warnings (used on the LHS of `??` and inside `isset`).
    fn read_index(&mut self, base: &Zval, key: &Zval, silent: bool) -> Result<Zval, PhpError> {
        match base {
            Zval::Array(a) => {
                let k = self.coerce_key(key)?;
                match a.get(&k) {
                    // Deref a reference element (D-R11): a normal read is by value.
                    Some(v) => Ok(v.deref_clone()),
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
    fn write_place(&mut self, base: PlaceBase, steps: &[Step], value: Zval) -> Result<(), PhpError> {
        if steps.is_empty() {
            // Write-through any reference cell, like `slot_set` (D-R3).
            match slot_mut!(self, base) {
                Zval::Ref(cell) => *cell.borrow_mut() = value,
                slot => *slot = value,
            }
            return Ok(());
        }
        let d = &mut self.diags;
        match slot_mut!(self, base) {
            Zval::Ref(cell) => {
                let z = &mut *cell.borrow_mut();
                write_into(z, steps, value, d)
            }
            other => write_into(other, steps, value, d),
        }
    }

    /// Bind `target` as a reference alias of `source` (`$target = &$source`,
    /// D-R4). The source slot is promoted to a shared cell on first use (a plain
    /// value becomes a `Ref`; binding an unset variable *defines* it as NULL, so
    /// no later undefined-variable warning fires). Returns the aliased value, as
    /// `$x = &$y` is an expression.
    fn assign_ref(&mut self, target: &Place, source: &Place) -> Result<Zval, PhpError> {
        // PHP evaluates the lvalue's index expressions before the RHS; resolve
        // the target's steps first, then the source's, for the same ordering.
        let tgt_steps = self.resolve_steps(target)?;
        let src_steps = self.resolve_steps(source)?;
        // Obtain (promoting) the shared cell the source designates, then bind the
        // target to it. Reading the cell yields the expression's value.
        let cell = self.ref_source_cell(source.base, &src_steps)?;
        let value = cell.borrow().clone();
        self.bind_ref_target(target.base, &tgt_steps, Rc::clone(&cell))?;
        Ok(value)
    }

    /// Bind `target` as a reference alias of the cell a by-reference function
    /// returns (`$y = &f()`, D-13.5). The call is invoked *raw* so its
    /// `Zval::Ref` result is aliased rather than copied. If the callee returned a
    /// plain value (it is not by-reference, or returned a non-place), bind a
    /// fresh cell holding that value.
    fn assign_ref_call(&mut self, target: &Place, call: &Expr) -> Result<Zval, PhpError> {
        let tgt_steps = self.resolve_steps(target)?;
        let (result, callee_by_ref) = self.eval_call_for_ref(call)?;
        let cell = match result {
            Zval::Ref(cell) => cell,
            other => {
                // Aliasing a non-reference result: PHP warns only when the callee
                // is not itself by-reference (a by-ref callee that returned a
                // non-place already emitted its own Notice — oracle F, D-13.5).
                if !callee_by_ref {
                    self.diags.push(Diag::Notice(
                        "Only variables should be assigned by reference".to_string(),
                    ));
                }
                Rc::new(RefCell::new(other))
            }
        };
        let value = cell.borrow().clone();
        self.bind_ref_target(target.base, &tgt_steps, Rc::clone(&cell))?;
        Ok(value)
    }

    /// Invoke an [`ExprKind::Call`] for a by-reference binding context, returning
    /// the *raw* result (a by-ref function's `Zval::Ref` is not dereferenced)
    /// together with whether the callee is declared by-reference (D-13.5).
    fn eval_call_for_ref(&mut self, call: &Expr) -> Result<(Zval, bool), PhpError> {
        let ExprKind::Call { name, args } = &call.kind else {
            // Lowering only builds `AssignRefCall` around a call; be defensive.
            return Ok((self.eval(call)?, false));
        };
        if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
            let by_ref = self.funcs[idx].by_ref;
            let argv = self.eval_call_args(idx, args)?;
            return Ok((self.call_user_fn(idx, argv)?, by_ref));
        }
        // A builtin never returns by reference; evaluate the whole call by value.
        Ok((self.eval(call)?, false))
    }

    /// The shared cell a reference *source* (`&$x`, `&$a[k]`) designates,
    /// promoting the location to a `Zval::Ref` on first use (D-R12). A bare
    /// variable goes through [`Evaluator::slot_cell`]; an element navigates and
    /// vivifies via [`place_cell`].
    fn ref_source_cell(
        &mut self,
        base: PlaceBase,
        steps: &[Step],
    ) -> Result<Rc<RefCell<Zval>>, PhpError> {
        if steps.is_empty() {
            Ok(make_cell(slot_mut!(self, base)))
        } else {
            let d = &mut self.diags;
            place_cell(slot_mut!(self, base), steps, d)
        }
    }

    /// Bind a reference *target* (`$x = …`, `$a[k] = …`, `$a[] = …`) to `cell`:
    /// a bare variable replaces its slot with `Zval::Ref`; an element writes the
    /// `Zval::Ref` into the place (auto-vivifying / appending as usual).
    fn bind_ref_target(
        &mut self,
        base: PlaceBase,
        steps: &[Step],
        cell: Rc<RefCell<Zval>>,
    ) -> Result<(), PhpError> {
        if steps.is_empty() {
            *slot_mut!(self, base) = Zval::Ref(cell);
            Ok(())
        } else {
            self.write_place(base, steps, Zval::Ref(cell))
        }
    }

    /// Obtain the shared cell backing a slot, promoting a plain value to a
    /// reference on first use. Binding a reference to an unset variable *defines*
    /// it as NULL (no later undefined-variable warning). Shared by `$x = &$y`
    /// and by-reference argument passing (D-R4/D-R6).
    fn slot_cell(&mut self, idx: usize) -> Rc<RefCell<Zval>> {
        make_cell(&mut frame_mut!(self)[idx])
    }

    /// Read the current value at a place (for compound assignment). Missing
    /// keys yield NULL with a warning; the base variable is read silently
    /// (it is about to be written / auto-vivified anyway).
    fn read_place_value(&mut self, base: PlaceBase, steps: &[Step]) -> Result<Zval, PhpError> {
        let mut cur = match self.base_clone(base) {
            Zval::Undef => Zval::Null,
            v => v,
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
    fn silent_get(&self, base: PlaceBase, steps: &[Step]) -> Option<Zval> {
        let mut cur = match self.base_clone(base) {
            Zval::Undef => return None,
            v => v,
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
    fn unset_place(&mut self, base: PlaceBase, steps: &[Step]) {
        if steps.is_empty() {
            // Drop *this* binding only: replacing it with a fresh value slot
            // releases this alias's share of any reference cell, leaving other
            // aliases untouched (D-R5).
            *slot_mut!(self, base) = Zval::Undef;
            return;
        }
        match slot_mut!(self, base) {
            Zval::Ref(cell) => {
                let z = &mut *cell.borrow_mut();
                unset_into(z, steps);
            }
            other => unset_into(other, steps),
        }
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
    // A reference target is written *through* its cell: descend into the
    // referenced value, whether for the final write (empty `steps`) or to keep
    // navigating (D-R3/D-R11). This makes `$a[0] = v` write through an alias
    // when `$a[0]` is a reference element.
    if let Zval::Ref(cell) = target {
        let inner = &mut *cell.borrow_mut();
        return write_into(inner, steps, value, diags);
    }
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
                // Overwrite a plain element, but write *through* an existing
                // reference element (the recursive call's top-of-fn deref).
                match arr.get_mut(k) {
                    Some(child) => write_into(child, rest, value, diags)?,
                    None => arr.insert(k.clone(), value),
                }
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

/// Promote `target` to a reference and return its shared cell. An existing
/// reference yields a clone of its cell; a plain value is wrapped (an unset slot
/// becomes a defined NULL first). The element/slot analogue of Zend's
/// `ZVAL_MAKE_REF` (D-R12).
fn make_cell(target: &mut Zval) -> Rc<RefCell<Zval>> {
    if let Zval::Ref(cell) = target {
        return Rc::clone(cell);
    }
    let init = match &*target {
        Zval::Undef => Zval::Null,
        other => other.clone(),
    };
    let cell = Rc::new(RefCell::new(init));
    *target = Zval::Ref(Rc::clone(&cell));
    cell
}

/// Navigate `steps` from `target`, auto-vivifying missing elements as NULL, and
/// promote the addressed location to a reference, returning its shared cell
/// (used by `$x = &$a[k]…`). A reference encountered along the path is followed
/// into its cell; a scalar base that cannot be indexed yields a detached cell so
/// the caller does not crash (the `ensure_array_mut` warning already fired).
fn place_cell(
    target: &mut Zval,
    steps: &[Step],
    diags: &mut Diags,
) -> Result<Rc<RefCell<Zval>>, PhpError> {
    let Some((first, rest)) = steps.split_first() else {
        return Ok(make_cell(target));
    };
    if let Zval::Ref(cell) = target {
        let inner = &mut *cell.borrow_mut();
        return place_cell(inner, steps, diags);
    }
    let Some(arr) = ensure_array_mut(target, diags) else {
        return Ok(Rc::new(RefCell::new(Zval::Null)));
    };
    let Step::Key(k) = first else {
        // `&$a[]` is not a valid reference source; treat defensively.
        return Ok(Rc::new(RefCell::new(Zval::Null)));
    };
    if !arr.contains_key(k) {
        arr.insert(k.clone(), Zval::Null);
    }
    let child = arr.get_mut(k).expect("key just inserted");
    place_cell(child, rest, diags)
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
        Zval::Ref(c) => coerce_key_silent(&c.borrow()),
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
/// Coerce `value` to scalar type `hint` under PHP's *weak* typing rules
/// (step 14). On success returns the coerced value (emitting the lossy-float
/// deprecations along the way); on failure returns the PHP type name of `value`
/// for the TypeError message. `null` satisfies a nullable hint verbatim.
fn coerce_to_hint(value: Zval, hint: &TypeHint, diags: &mut Diags) -> Result<Zval, &'static str> {
    // Follow a reference to its value first (defensive; bound args are plain).
    if let Zval::Ref(c) = &value {
        let inner = c.borrow().clone();
        return coerce_to_hint(inner, hint, diags);
    }
    if matches!(value, Zval::Null | Zval::Undef) {
        return if hint.nullable {
            Ok(Zval::Null)
        } else {
            Err("null")
        };
    }
    let given = php_type_name(&value);
    match hint.kind {
        ScalarType::Int => coerce_to_int(value, diags),
        ScalarType::Float => coerce_to_float(value),
        ScalarType::String => coerce_to_string(value, diags),
        ScalarType::Bool => coerce_to_bool(value, diags),
    }
    .ok_or(given)
}

/// Weak coercion to `int`: numeric strings must be *well formed* (stricter than
/// the `(int)` cast — `"12abc"` fails). A float / float-string that loses
/// precision emits a deprecation (D-14.6). `None` means a type error.
fn coerce_to_int(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Long(_) => Some(value),
        Zval::Bool(b) => Some(Zval::Long(b as i64)),
        Zval::Double(d) => Some(Zval::Long(convert::dval_to_lval_safe(d, diags))),
        Zval::Str(ref s) => {
            let info = numstr::parse_numeric_ex(s.as_bytes(), false)?;
            if info.trailing {
                return None;
            }
            match info.num {
                numstr::Num::Long(l) => Some(Zval::Long(l)),
                numstr::Num::Double(d) => {
                    let l = convert::dval_to_lval(d);
                    if !convert::is_long_compatible(d, l) {
                        diags.push(Diag::Deprecated(format!(
                            "Implicit conversion from float-string \"{}\" to int loses precision",
                            String::from_utf8_lossy(s.as_bytes())
                        )));
                    }
                    Some(Zval::Long(l))
                }
            }
        }
        _ => None,
    }
}

/// Weak coercion to `float`: numeric strings (incl. scientific) convert; others
/// are a type error.
fn coerce_to_float(value: Zval) -> Option<Zval> {
    match value {
        Zval::Double(_) => Some(value),
        Zval::Long(l) => Some(Zval::Double(l as f64)),
        Zval::Bool(b) => Some(Zval::Double(b as i64 as f64)),
        Zval::Str(ref s) => {
            let info = numstr::parse_numeric_ex(s.as_bytes(), false)?;
            if info.trailing {
                return None;
            }
            Some(Zval::Double(match info.num {
                numstr::Num::Long(l) => l as f64,
                numstr::Num::Double(d) => d,
            }))
        }
        _ => None,
    }
}

/// Weak coercion to `string`: any scalar stringifies; array / object are a type
/// error.
fn coerce_to_string(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Str(_) => Some(value),
        Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) => {
            Some(Zval::Str(convert::to_zstr(&value, diags)))
        }
        _ => None,
    }
}

/// Weak coercion to `bool`: any scalar converts; array / object are a type
/// error.
fn coerce_to_bool(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Bool(_) => Some(value),
        Zval::Long(_) | Zval::Double(_) | Zval::Str(_) => {
            Some(Zval::Bool(convert::to_bool(&value, diags)))
        }
        _ => None,
    }
}

fn php_type_name(v: &Zval) -> &'static str {
    match v {
        Zval::Undef | Zval::Null => "null",
        Zval::Bool(_) => "bool",
        Zval::Long(_) => "int",
        Zval::Double(_) => "float",
        Zval::Str(_) => "string",
        Zval::Array(_) => "array",
        Zval::Ref(c) => php_type_name(&c.borrow()),
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
        Zval::Ref(c) => match_case_repr(&c.borrow()),
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
