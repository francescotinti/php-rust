//! HIR → bytecode compiler (VM-migration Fase 3, vertical proof slice).
//!
//! Where the original tree-walker `match`ed on an [`crate::hir::ExprKind`] to
//! *execute* it, this module `match`es to *emit* [`crate::bytecode::Op`]s, sharing
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

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use php_types::{ObjectInfo, PhpStr, PropVis};

use crate::builtin::{Builtin, Registry};
use crate::bytecode::{
    Addr, BuiltinIface, ClassTarget, CompiledAttribute, CompiledClass, CompiledConst, CompiledEnumCase, CompiledMethod, CompiledStaticProp, Const,
    ConstIdx, DimBase, ExcRegion, FieldBase, FieldStep, Func, Instantiable, Module, Op, PropHooks, PropInfo, StaticInit,
};
use crate::hir::{
    BinOp, Case, CatchClause, ClassDecl, ClassId, ClassRef, Expr, ExprKind, FnDecl, HintKind, Line,
    MatchArm, Param, Place, PlaceBase, PlaceStep, Program, StaticAssignOp, Stmt, StmtKind, TypeHint,
    Visibility,
};

mod class;
mod func;
mod expr;
mod assign;
use class::*;
use func::*;

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

/// The program-wide context every body compiles against: the function table (for
/// call resolution), the builtin registry (for classifying call names), the class
/// table (for compile-time method/constructor resolution and walking parents),
/// and a case-insensitive name→[`ClassId`] index (for resolving
/// [`ClassRef::Named`]). Bundled so a body's compiler can borrow it whole.
struct ProgramCtx<'a> {
    funcs: &'a [FnDecl],
    /// Indices into `funcs` that are conditional declarations: they do NOT resolve
    /// a call by name at compile time (dispatched dynamically, callable only once
    /// their `DeclareFn` runs).
    conditional_fns: &'a HashSet<usize>,
    registry: &'a Registry,
    classes: &'a [ClassDecl],
    class_index: &'a HashMap<Vec<u8>, ClassId>,
}

/// Compile a lowered [`Program`] into an executable [`Module`].
///
/// Functions, classes and methods are compiled in the same index spaces as the
/// source [`Program`], so a call/`new`/method resolved to index `i` in the HIR
/// maps to index `i` here. Closures are still out of slice.
pub fn compile_program(program: &Program, registry: &Registry) -> R<Module> {
    compile_program_stubbed(program, registry, &[])
}


/// Like [`compile_program`], but a class whose index is marked in `stub_mask`
/// compiles to an inert stub. An `include`/`eval` unit is lowered against the
/// whole accumulated seed image (`program.classes = [seed…, new…]`), yet
/// `drive_unit` dedups every already-linked class by name — so fully compiling
/// the seed portion again is pure waste: O(seed) bytecode per included file,
/// *quadratic* memory/time across a Composer autoload storm (PHPUnit's
/// `preload()` requires ~1200 files → gigabytes of duplicated, leaked class
/// bytecode). The VM passes a mask of the classes its `class_index` already
/// links; new / conditional classes always compile in full.
pub fn compile_program_stubbed(
    program: &Program,
    registry: &Registry,
    stub_mask: &[bool],
) -> R<Module> {
    // Case-insensitive name→id index for resolving `ClassRef::Named`; the first
    // declaration of a name wins (PHP forbids redeclaration).
    let mut class_index: HashMap<Vec<u8>, ClassId> = HashMap::new();
    for (i, cd) in program.classes.iter().enumerate() {
        // A conditional declaration is not resolvable by name until its
        // `Op::DeclareClass` runs, so it stays out of the eager index (its
        // `new X` / static reference resolves dynamically at run time).
        if program.conditional_classes.contains(&i) {
            continue;
        }
        class_index.entry(cd.name.to_ascii_lowercase()).or_insert(i);
    }
    let ctx = ProgramCtx {
        funcs: &program.functions,
        conditional_fns: &program.conditional_fns,
        registry,
        classes: &program.classes,
        class_index: &class_index,
    };

    let mut functions = Vec::with_capacity(program.functions.len());
    for fd in &program.functions {
        // Function bodies compile *tolerantly*: the always-injected PHP prelude
        // (exception classes, date API) uses not-yet-ported constructs, so a
        // failure becomes a stub that fatals only if the function is called —
        // rather than making every script uncompilable. `main`, below, is not
        // tolerant: if the script body itself is unsupported, the VM can't run it.
        match compile_fndecl(fd, &ctx) {
            Ok(f) => functions.push(f),
            Err(e) => functions.push(stub_func(fd, &e)),
        }
    }
    // Closure bodies compile tolerantly (like functions): an unsupported body
    // becomes a stub that fatals only if the closure is actually invoked. Same
    // index space as `program.closures`, so `MakeClosure { fn_idx }` lines up.
    let closures = program
        .closures
        .iter()
        .map(|fd| compile_fndecl(fd, &ctx).unwrap_or_else(|e| stub_func(fd, &e)))
        .collect();
    let main = compile_body(b"", &program.file, &program.body, program.slots.len() as u32, &[], &program.slots, false, false, None, 0, &ctx, None, true, 0)?;
    // Classes are compiled tolerantly too (see `compile_class`); a seed class
    // the VM already links compiles to an inert stub (see the doc above).
    let classes = program
        .classes
        .iter()
        .enumerate()
        .map(|(cid, cd)| {
            if stub_mask.get(cid).copied().unwrap_or(false) {
                stub_class(cd)
            } else {
                compile_class(cid, cd, &ctx)
            }
        })
        .collect();

    // Top-level `const` attributes, compiled name→thunks (free-function context,
    // so `cur_class = None`).
    let const_attributes = program
        .const_attributes
        .iter()
        .map(|(name, attrs)| (name.clone(), compile_attrs(attrs, &ctx, None)))
        .collect();

    Ok(Module {
        main,
        functions,
        conditional_fns: program.conditional_fns.clone(),
        conditional_classes: program.conditional_classes.clone(),
        conditional_traits: program.conditional_traits.clone(),
        closures,
        classes,
        file: program.file.clone(),
        class_index,
        static_count: program.static_count,
        strict: program.strict,
        const_attributes,
    })
}















/// Constant-fold a property-default expression to a [`Const`]. OOP-1 only handles
/// scalar literals; anything else (array, constant ref, arithmetic) yields `None`
/// and makes its class an uninstantiable stub.
fn const_eval(e: &Expr) -> Option<Const> {
    match &e.kind {
        ExprKind::Null => Some(Const::Null),
        ExprKind::Bool(b) => Some(Const::Bool(*b)),
        ExprKind::Int(i) => Some(Const::Int(*i)),
        ExprKind::Float(f) => Some(Const::Float(*f)),
        ExprKind::Str(s) => Some(Const::Str(s.clone())),
        _ => None,
    }
}

/// If a parameter default is exactly a constant reference (a global `const` or a
/// `Class::CONST`), the constant's name as `ReflectionParameter::
/// getDefaultValueConstantName()` reports it — a class ref keeps its source text
/// (`self::bar`, `Foo2::bar`), not the resolved class. `None` for any other
/// default (a literal, an expression that merely contains a constant, …).
fn default_const_name(e: &Expr) -> Option<Box<[u8]>> {
    match &e.kind {
        ExprKind::Const { name, .. } => Some(name.clone()),
        ExprKind::ClassConst { class, name } => {
            let cls: &[u8] = match class {
                ClassRef::Named(n) => n,
                ClassRef::SelfClass => b"self",
                ClassRef::Parent => b"parent",
                ClassRef::Static => b"static",
                ClassRef::Dynamic(_) => return None,
            };
            let mut s = cls.to_vec();
            s.extend_from_slice(b"::");
            s.extend_from_slice(name);
            Some(s.into_boxed_slice())
        }
        _ => None,
    }
}


/// Like [`const_eval`] but able to resolve a class-constant reference
/// (`Self::C`, `Iface::C`, `Parent::C`) against the HIR class table, evaluated in
/// the *declaring* class's context. Used to const-fold enum case backing values
/// that name an inherited interface constant (e.g. `case C = I::A;`), which a
/// context-free fold cannot reach. Walks the parent chain then interfaces
/// (transitively), mirroring `find_class_const`. `depth` guards against a cyclic
/// constant definition (a PHP error in its own right).
fn const_eval_in_class(e: &Expr, cur: ClassId, ctx: &ProgramCtx, depth: u32) -> Option<Const> {
    if depth > 32 {
        return None;
    }
    if let ExprKind::ClassConst { class, name } = &e.kind {
        if name.eq_ignore_ascii_case(b"class") {
            return None; // `E::class` is the class name string; not folded here.
        }
        let target = match class {
            ClassRef::Named(n) => *ctx.class_index.get(&n.to_ascii_lowercase())?,
            ClassRef::SelfClass => cur,
            ClassRef::Parent => ctx.classes[cur].parent?,
            ClassRef::Static | ClassRef::Dynamic(_) => return None,
        };
        let (decl_cid, value) = find_const_decl(target, name, ctx)?;
        return const_eval_in_class(value, decl_cid, ctx, depth + 1);
    }
    const_eval(e)
}


/// Per-function emit state: the growing instruction stream, the constant pool,
/// the stack of enclosing loops (for `break N` / `continue N`), and the
/// program-wide [`ProgramCtx`] for resolving call / class targets.
struct FnCompiler<'a> {
    /// Inside a `list()` destructuring's element assignments: element reads
    /// compile to [`Op::FetchDimList`] (silent on a scalar base, unlike a plain
    /// `$x[k]` read — Zend's list path raises no offset-on-scalar warning).
    in_list_assign: bool,
    /// Pending `?->` short-circuit jump sites of the access chain being
    /// compiled: PHP's nullsafe skips to the end of the WHOLE chain
    /// (`$a?->b()->c()['d']` with null `$a` evaluates none of it), so every
    /// deref link routes its null-jump here and the chain's outermost link
    /// patches them all past itself. `None` outside a chain; detached
    /// (chain_pause) around non-chain subexpressions like call arguments.
    nullsafe_chain: Option<Vec<u32>>,
    ops: Vec<Op>,
    /// Source line of each emitted op, parallel to `ops` (EXC-3b). `emit` pushes
    /// the current line; `patch` overwrites an op in place and leaves this alone.
    lines: Vec<Line>,
    /// The line of the statement/expression currently being compiled; stamped
    /// onto every op `emit` appends. Updated at the top of `stmt`/`expr`.
    cur_line: Line,
    consts: Vec<Const>,
    loops: Vec<LoopCtx>,
    ctx: &'a ProgramCtx<'a>,
    /// The class whose method (or constant thunk) is being compiled, for
    /// resolving `self::` / `parent::` at compile time; `None` for the script
    /// body and free functions.
    cur_class: Option<ClassId>,
    /// Added to every emitted closure index (see [`FnDecl::closure_shift`]); 0
    /// except for a trait body flattened in from another unit.
    closure_shift: i32,
    /// True only for the top-level script body: a destruction sweep
    /// ([`Op::Sweep`]) is emitted after each of its statements, mirroring the
    /// tree-walker's global-scope sweep (OOP-3d). Never set for functions/methods.
    is_main: bool,
    /// True when compiling a `function &f()` body — a plain `return <expr>;` (or
    /// bare `return;`) then raises the "Only variable references should be returned
    /// by reference" notice (the operand is a non-lvalue). Set by `compile_body`.
    returns_ref: bool,
    /// Names of the HIR-allocated local slots (without the leading `$`), used to
    /// label the "Undefined variable" warning at a source-level `$x` read. Empty
    /// for bodies with no named locals (const / prop-init thunks).
    slot_names: &'a [Box<[u8]>],
    /// Number of named locals (HIR slots); compiler temporaries are allocated
    /// above this, so the frame's slot array is `n_locals + n_temps_max` wide.
    n_locals: u32,
    n_temps_cur: u32,
    n_temps_max: u32,
    /// Protected `try` regions accumulated while compiling this body (EXC); each
    /// is appended when its `try` finishes, so inner regions precede outer ones.
    exc_regions: Vec<ExcRegion>,
    /// Active `finally` scopes (EXC-2b), innermost last. A `return`/`break`/
    /// `continue` compiled while one is active is routed through the finally
    /// (parked, then a jump to the finally entry recorded for patching).
    finally_scopes: Vec<FinallyScope>,
    /// Resolved `label:` positions in this function body (step 45), by name: the
    /// op address and the block-scope path (innermost last) where the label sits,
    /// used to detect a `goto` *into* a transparent block (D-45.1).
    labels: HashMap<Vec<u8>, (Addr, Vec<u32>)>,
    /// `goto` jump sites awaiting their label, patched once the whole body is
    /// compiled (labels may be forward references). Each records the two-op site
    /// (a placeholder + a trailing slot for finally routing) and the goto's
    /// block-scope path.
    pending_gotos: Vec<(Box<[u8]>, Addr, Vec<u32>)>,
    /// Block-nesting path of the statement currently being compiled (a fresh id per
    /// `block()`), innermost last. A `goto` may only target a label whose scope is a
    /// prefix of the goto's (same block or an enclosing one); a deeper/divergent
    /// target is a jump *into* a transparent block (D-45.1).
    scope_path: Vec<u32>,
    /// Monotonic block-scope id source for `scope_path`.
    next_scope: u32,
    /// One entry per compiled `try` with a `finally`: the protected op range, the
    /// finally entry address, and the scope depth *outside* the try (its own level).
    /// `resolve_gotos` uses it to route a `goto` that leaves the protected region
    /// through the finally — the target is "outside" iff its scope depth is `<=` the
    /// outer depth (an address test breaks on a label marker at the try's boundary).
    goto_finally_meta: Vec<(std::ops::Range<Addr>, Addr, usize)>,
    /// Function-local `static $v` declarations seen in this body (name, cell id,
    /// initial value), collected for `Func::static_vars` / `getStaticVariables()`.
    static_vars: Vec<crate::bytecode::StaticVarDecl>,
}

/// One active `finally` block, while its protected body/catches are compiled
/// (EXC-2b). Collects the jump sites of control transfers that must run this
/// finally first; they are patched to the finally entry once it is known.
#[derive(Default)]
struct FinallyScope {
    /// `Op::Jump` sites that should land at the finally entry.
    sites: Vec<Addr>,
    /// `self.loops.len()` when this finally was entered: a `break`/`continue`
    /// whose target loop index is `< loop_depth` exits past this finally (so it
    /// is routed through it); a deeper target stays inside the protected body.
    loop_depth: usize,
}

/// One enclosing loop's unresolved jump sites. `break` jumps land at the loop
/// exit; `continue` jumps land at the loop's step/condition re-entry. Both are
/// patched once those addresses are known.
#[derive(Default)]
struct LoopCtx {
    break_sites: Vec<Addr>,
    continue_sites: Vec<Addr>,
    /// `Op::ParkJump` sites (break/continue that cross a finally): patched to the
    /// loop target like the plain sites, but with `ParkJump` so the jump runs
    /// after the finally (EXC-2b).
    parked_break_sites: Vec<Addr>,
    parked_continue_sites: Vec<Addr>,
    /// `true` for a `foreach`: a `break`/`continue` that leaves this loop must
    /// first emit an [`Op::IterPop`] to free the iterator (Zend's `FE_FREE`).
    has_iter: bool,
}

impl<'a> FnCompiler<'a> {
    fn new(
        ctx: &'a ProgramCtx<'a>,
        n_locals: u32,
        cur_class: Option<ClassId>,
        is_main: bool,
        slot_names: &'a [Box<[u8]>],
    ) -> Self {
        FnCompiler {
            in_list_assign: false,
            nullsafe_chain: None,
            ops: Vec::new(),
            lines: Vec::new(),
            cur_line: 0,
            consts: Vec::new(),
            loops: Vec::new(),
            ctx,
            cur_class,
            closure_shift: 0,
            is_main,
            returns_ref: false,
            slot_names,
            n_locals,
            n_temps_cur: 0,
            n_temps_max: 0,
            exc_regions: Vec::new(),
            finally_scopes: Vec::new(),
            labels: HashMap::new(),
            pending_gotos: Vec::new(),
            scope_path: Vec::new(),
            next_scope: 0,
            goto_finally_meta: Vec::new(),
            static_vars: Vec::new(),
        }
    }

    /// Reserve a scratch slot above the named locals (for `switch`/`match`
    /// subjects). Freed with [`Self::free_temp`] so siblings reuse the space.
    fn alloc_temp(&mut self) -> crate::hir::Slot {
        let s = self.n_locals + self.n_temps_cur;
        self.n_temps_cur += 1;
        self.n_temps_max = self.n_temps_max.max(self.n_temps_cur);
        s
    }

    fn free_temp(&mut self) {
        self.n_temps_cur -= 1;
    }

    /// Rebind a temp slot to `null`, dropping a `Zval::Ref` left in it by an
    /// alias trick (`reset($this->prop)` binds the temp to the property's
    /// cell). A freed temp is REUSED, and the common `StoreSlot` writes
    /// *through* a ref it finds in the slot — a stale alias residue would
    /// silently corrupt the aliased place (ORM's ArrayHydrator saw its
    /// `resultPointers` overwritten by an unrelated temp value this way).
    fn clear_temp_binding(&mut self, tmp: crate::hir::Slot) {
        let k = self.konst(Const::Null);
        self.emit(Op::PushConst(k));
        self.emit(Op::BindRefTo { base: FieldBase::Local(tmp), steps: [].into() });
        self.emit(Op::Pop);
    }

    /// Append `op`, returning its address. Records the current source line in the
    /// parallel `lines` table (EXC-3b).
    fn emit(&mut self, op: Op) -> Addr {
        let at = self.ops.len() as Addr;
        self.ops.push(op);
        self.lines.push(self.cur_line);
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

    /// Emit the default-parameter prologue (PAR): for each parameter with a
    /// default, `FillDefault` skips the default when the argument was supplied,
    /// else the default expression is evaluated and stored into the slot.
    /// Variadic / required parameters have no default and emit nothing.
    fn param_prologue(&mut self, params: &[Param]) -> R<()> {
        // Arity guard (PAR): a required param is a non-variadic one with no
        // default. "exactly" when there are no optional/variadic params at all.
        let required = params.iter().filter(|p| p.default.is_none() && !p.variadic).count() as u32;
        if required > 0 {
            let exactly = !params.iter().any(|p| p.default.is_some() || p.variadic);
            self.emit(Op::CheckArity { required, exactly });
        }
        for p in params {
            let Some(default) = &p.default else { continue };
            let fill = self.emit(Op::FillDefault { slot: p.slot, skip: Addr::MAX });
            self.expr(default)?;
            self.emit(Op::StoreSlot(p.slot));
            // A scalar-hinted default is coerced to its type (`float $n = 0` →
            // 0.0, D-NEW-6); an unhinted / non-scalar param keeps the value.
            // Only a scalar-hinted default needs coercion; a non-scalar hint is a
            // run-time check on passed args, not on a (constant) default value.
            if let Some(hint @ TypeHint { kind: HintKind::Scalar(_), .. }) = &p.hint {
                self.emit(Op::CoerceParam { slot: p.slot, hint: hint.clone() });
            }
            let here = self.here();
            self.patch(fill, Op::FillDefault { slot: p.slot, skip: here });
        }
        Ok(())
    }

    fn block(&mut self, stmts: &[Stmt]) -> R<()> {
        // Open a fresh block scope so a `goto` *into* this block (from a shallower
        // scope) can be detected at `resolve_gotos` (D-45.1). Every compound body
        // (if/else, try/catch/finally, loops, plain `{ }`) and the function root
        // funnel through here, so sibling statements share one scope.
        let id = self.next_scope;
        self.next_scope += 1;
        self.scope_path.push(id);
        for s in stmts {
            self.stmt(s)?;
            // At global scope, sweep unreachable objects after each statement
            // (OOP-3d); inside functions/methods the tree-walker does not.
            if self.is_main {
                self.emit(Op::Sweep);
            }
        }
        self.scope_path.pop();
        Ok(())
    }

    fn stmt(&mut self, s: &Stmt) -> R<()> {
        self.cur_line = s.line;
        match &s.kind {
            StmtKind::Nop => {}
            StmtKind::DeclareFn(idx) => {
                self.emit(Op::DeclareFn { func: *idx as u32 });
            }
            StmtKind::DeclareClass(idx) => {
                self.emit(Op::DeclareClass { class: *idx });
            }
            StmtKind::DeclareTrait(idx) => {
                self.emit(Op::DeclareTrait { idx: *idx as u32 });
            }
            StmtKind::Echo(values) => {
                for e in values {
                    self.expr(e)?;
                    self.emit(Op::Stringify); // honour __toString (OOP-3c)
                    self.emit(Op::Echo);
                }
            }
            StmtKind::ConstDecl(items) => {
                for (name, value) in items {
                    self.expr(value)?;
                    self.emit(Op::DefineConst { name: name.clone() });
                }
            }
            StmtKind::InlineHtml(bytes) => {
                // Raw text outside `<?php … ?>` (and the newline after a closing
                // tag): emitted verbatim, like `eval`'s `emit(bytes)`. Reuses the
                // string-constant + `Echo` path (a `Str` stringifies to itself).
                let k = self.konst(Const::Str(bytes.clone()));
                self.emit(Op::PushConst(k));
                self.emit(Op::Echo);
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
            StmtKind::Foreach { iter, key, value, by_ref, body } => {
                if *by_ref {
                    // REF-3: by-ref iteration needs an lvalue source to write back
                    // to. Over a plain variable we rebind each element live. Over a
                    // non-lvalue source (a temporary array, a function return) PHP
                    // degrades to by-value iteration, where writes to `$value` land
                    // nowhere observable — so that case falls through to the
                    // by-value loop below.
                    if let ExprKind::Var(slot) = iter.kind {
                        self.emit(Op::IterInitRef(slot));
                        let cont = self.here();
                        let fetch = self.emit(Op::IterNextRef { value: *value, key: *key, end: Addr::MAX });
                        self.loops.push(LoopCtx { has_iter: true, ..LoopCtx::default() });
                        self.block(body)?;
                        self.emit(Op::Jump(cont));
                        let exhaust = self.here();
                        self.patch(fetch, Op::IterNextRef { value: *value, key: *key, end: exhaust });
                        self.emit(Op::IterPop);
                        let after = self.here();
                        self.close_loop(cont, after);
                        return Ok(());
                    }
                    // Any OTHER lvalue source — a property (`$o->p`), array element,
                    // static/`$this` property — is written back to too: bind a
                    // reference to the place into a temp local, then iterate that by
                    // reference so an element REPLACEMENT (`$v = …`) writes through to
                    // the source array (Doctrine QueryBuilder::indexBy rewrites
                    // `$this->dqlParts['from']` elements via `foreach (… as &$f)`).
                    if let Some(place) = expr_field_place(iter) {
                        let (base, steps) = self.field_path(&place)?;
                        self.emit(Op::MakeRef { base, steps: steps.into() });
                        let tmp = self.alloc_temp();
                        self.emit(Op::BindRefTo { base: FieldBase::Local(tmp), steps: [].into() });
                        self.emit(Op::Pop);
                        self.emit(Op::IterInitRef(tmp));
                        let cont = self.here();
                        let fetch = self.emit(Op::IterNextRef { value: *value, key: *key, end: Addr::MAX });
                        self.loops.push(LoopCtx { has_iter: true, ..LoopCtx::default() });
                        self.block(body)?;
                        self.emit(Op::Jump(cont));
                        let exhaust = self.here();
                        self.patch(fetch, Op::IterNextRef { value: *value, key: *key, end: exhaust });
                        self.emit(Op::IterPop);
                        let after = self.here();
                        self.clear_temp_binding(tmp);
                        self.free_temp();
                        self.close_loop(cont, after);
                        return Ok(());
                    }
                }
                self.expr(iter)?;
                self.emit(Op::IterInit);
                let cont = self.here(); // `continue` re-fetches
                let fetch = self.emit(Op::IterNext { value: *value, key: *key, end: Addr::MAX });
                self.loops.push(LoopCtx { has_iter: true, ..LoopCtx::default() });
                self.block(body)?;
                self.emit(Op::Jump(cont));
                let exhaust = self.here();
                self.patch(fetch, Op::IterNext { value: *value, key: *key, end: exhaust });
                self.emit(Op::IterPop); // normal exhaustion frees the iterator
                let after = self.here(); // `break` lands here (after its own IterPop)
                self.close_loop(cont, after);
            }
            StmtKind::Break(n) => self.loop_jump(*n, true)?,
            StmtKind::Continue(n) => self.loop_jump(*n, false)?,
            StmtKind::Label(name) => {
                // `label:` (step 45) marks a jump target; the lowerer already
                // validated that no `goto` jumps *into* a loop/switch. Record the
                // scope so `resolve_gotos` can reject a jump into a transparent block.
                let here = self.here();
                self.labels.insert(name.to_vec(), (here, self.scope_path.clone()));
            }
            StmtKind::Goto(name) => {
                // Unconditional jump to a (possibly forward) label, patched once the
                // whole body is compiled. Two op slots are reserved: a `goto` that
                // crosses a `finally` is patched to `ParkJump(target)` + a jump to the
                // finally entry (the finally runs, then `EndFinally` performs the
                // parked jump, EXC-2b); a plain or into-block goto only uses the first.
                let site = self.emit(Op::Jump(Addr::MAX));
                self.emit(Op::Jump(Addr::MAX));
                self.pending_gotos.push((name.clone(), site, self.scope_path.clone()));
            }
            StmtKind::Return(opt) => {
                // A plain `return <expr>;` (or bare `return;`) inside a `function
                // &f()` means the operand is a non-lvalue: PHP raises a notice and
                // returns by value (D-13.4). The condition is known at compile time.
                if self.returns_ref {
                    let k = self.konst(Const::Str(
                        b"Only variable references should be returned by reference"[..].into(),
                    ));
                    self.emit(Op::EmitNotice(k));
                }
                match opt {
                    Some(e) => self.expr(e)?,
                    None => {
                        let null = self.konst(Const::Null);
                        self.emit(Op::PushConst(null));
                    }
                }
                // A `return` inside a try-with-finally runs the (innermost) finally
                // first: park the value, jump to the finally entry; `EndFinally`
                // then performs the return (EXC-2b).
                if !self.finally_scopes.is_empty() {
                    self.emit(Op::ParkReturn);
                    let jmp = self.emit(Op::Jump(Addr::MAX));
                    self.finally_scopes.last_mut().unwrap().sites.push(jmp);
                } else {
                    self.emit(Op::Ret);
                }
            }
            StmtKind::ReturnRef(place) => {
                // `function &f()` returning an lvalue (REF-4b): push a reference
                // to the place's cell and return it raw, so `$y = &f()` aliases
                // it. `field_path`/`MakeRef` handle index, property and `[]` steps.
                let (base, steps) = self.field_path(place)?;
                self.emit(Op::MakeRef { base, steps: steps.into() });
                self.emit(Op::Ret);
            }
            StmtKind::Unset(places) => {
                for place in places {
                    self.unset_place(place)?;
                }
            }
            StmtKind::Switch { subject, cases } => self.switch(subject, cases)?,
            StmtKind::Try { body, catches, finally } => self.try_stmt(body, catches, finally)?,
            StmtKind::Global(bindings) => {
                // REF-1. At script scope the named variable *is* the global
                // (main's frame is the global frame), so `global` is a no-op —
                // matching the tree-walker (D-12.2). Inside a function, alias each
                // local slot to its global-frame cell via a shared reference.
                if !self.is_main {
                    for b in bindings {
                        self.emit(Op::BindRef {
                            target: DimBase::Local(b.local),
                            source: DimBase::Global(b.global),
                        });
                        self.emit(Op::Pop); // statement: drop the BindRef value
                    }
                }
            }
            StmtKind::StaticVar(bindings) => {
                // `static $a = init, …` (step 15 VM port): per binding, guard the
                // one-time initialiser behind the persistent cell's existence, then
                // alias the local slot to that cell on every call. `id` is the
                // program-global static index; sharing it across calls (and across
                // recursion) gives `static` its persistence.
                for b in bindings {
                    let id = b.id as u32;
                    let guard = self.emit(Op::StaticGuard { id, skip: Addr::MAX });
                    match &b.init {
                        Some(e) => self.expr(e)?,
                        None => {
                            let null = self.konst(Const::Null);
                            self.emit(Op::PushConst(null));
                        }
                    }
                    self.emit(Op::StaticStore { id });
                    let alias = self.here();
                    self.patch(guard, Op::StaticGuard { id, skip: alias });
                    self.emit(Op::StaticAlias { slot: b.slot, id });
                    // Record the declared initial value for getStaticVariables()
                    // (used until the runtime cell exists). A non-const initialiser
                    // that does not fold reads as NULL before the function runs.
                    let init = match &b.init {
                        Some(e) => const_eval(e).map_or(StaticInit::Const(Const::Null), StaticInit::Const),
                        None => StaticInit::Const(Const::Null),
                    };
                    self.static_vars.push(crate::bytecode::StaticVarDecl { name: b.name.clone(), id, init });
                }
            }
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

    /// Patch every `goto` jump site to its label position, once the whole body is
    /// compiled (labels may be forward references). An unknown label should have
    /// been caught at lowering, but is surfaced defensively (step 45).
    fn resolve_gotos(&mut self) -> R<()> {
        let gotos = std::mem::take(&mut self.pending_gotos);
        for (name, site, goto_scope) in gotos {
            let Some((target, label_scope)) = self.labels.get(name.as_ref()).cloned() else {
                return Err(CompileError::Unsupported(format!(
                    "goto to undefined label '{}'",
                    String::from_utf8_lossy(&name)
                )));
            };
            // A `goto` may only target a label in its own block or an enclosing one
            // (the label scope must be a prefix of the goto scope). A deeper or
            // divergent target is a jump *into* a transparent block, which the
            // tree-walker scopes out — match it with the same run-time fatal (D-45.1),
            // raised in place so output before the goto still flushes.
            let into_block = label_scope.len() > goto_scope.len()
                || goto_scope[..label_scope.len()] != label_scope[..];
            if into_block {
                let k = self.konst(Const::Str(
                    format!(
                        "'goto' into a block is not supported (label '{}', D-45.1)",
                        String::from_utf8_lossy(&name)
                    )
                    .into_bytes()
                    .into(),
                ));
                self.patch(site, Op::Fatal(k));
                continue;
            }
            // If the goto leaves a `finally`'s protected region (the target sits
            // outside it), route it through that finally like break/continue: park
            // the target, jump to the finally entry; `EndFinally` performs the parked
            // jump afterwards (EXC-2b). The innermost crossed finally runs.
            // "goto inside the protected region" is decided by address (a real op);
            // "label outside the region" by scope, not address — a `label:` marker
            // just before the try shares the try's start address (it emits no op), so
            // an address test would wrongly read it as inside. The label is outside
            // the body iff its scope is at or above the try's own level.
            let crossed = self
                .goto_finally_meta
                .iter()
                .filter(|(protected, _entry, outer_len)| {
                    protected.contains(&site) && label_scope.len() <= *outer_len
                })
                .min_by_key(|(protected, _, _)| protected.end - protected.start)
                .map(|(_, entry, _)| *entry);
            match crossed {
                Some(finally_entry) => {
                    self.patch(site, Op::ParkJump(target));
                    self.patch(site + 1, Op::Jump(finally_entry));
                }
                None => self.patch(site, Op::Jump(target)),
            }
        }
        Ok(())
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
        // Sites that cross a `finally` resume at the loop target via `ParkJump`,
        // run by `EndFinally` after the finally completes (EXC-2b).
        for at in ctx.parked_break_sites {
            self.patch(at, Op::ParkJump(break_target));
        }
        for at in ctx.parked_continue_sites {
            self.patch(at, Op::ParkJump(continue_target));
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
        let idx = depth - level as usize;
        // Free the iterator of every `foreach` this jump leaves: for `break`,
        // the target loop itself (idx..depth); for `continue`, only the inner
        // loops fully exited (idx+1..depth) — we stay inside the target.
        let first = if is_break { idx } else { idx + 1 };
        let pops = self.loops[first..depth].iter().filter(|l| l.has_iter).count();
        for _ in 0..pops {
            self.emit(Op::IterPop);
        }
        // If this jump exits past an enclosing `finally` (the target loop sits
        // outside it), route it through that finally: park the loop target, then
        // jump to the finally entry; `EndFinally` performs the jump afterwards
        // (EXC-2b). Single-finally crossing; the innermost crossed finally runs.
        if let Some(scope_i) = self.finally_scopes.iter().rposition(|s| idx < s.loop_depth) {
            let park = self.emit(Op::ParkJump(Addr::MAX));
            if is_break {
                self.loops[idx].parked_break_sites.push(park);
            } else {
                self.loops[idx].parked_continue_sites.push(park);
            }
            let jmp = self.emit(Op::Jump(Addr::MAX));
            self.finally_scopes[scope_i].sites.push(jmp);
            return Ok(());
        }
        let at = self.emit(Op::Jump(Addr::MAX));
        if is_break {
            self.loops[idx].break_sites.push(at);
        } else {
            self.loops[idx].continue_sites.push(at);
        }
        Ok(())
    }

    /// Push the value of a `??` left-operand sub-expression using isset
    /// semantics: an undefined variable, a missing array element, or an unset
    /// dynamic property yields `null` *silently* (no "Undefined variable /
    /// index / property" warning), recursing through nested `$a[b][c]` chains.
    /// Mirrors PHP: `??` suppresses those notices along the whole access path.
    /// Non-access operands fall through to a normal read.
    fn coalesce_load(&mut self, a: &Expr) -> R<()> {
        match &a.kind {
            ExprKind::Var(slot) => {
                self.emit(Op::LoadSlot(*slot));
            }
            ExprKind::Index { base, index } => {
                self.coalesce_load(base)?;
                self.expr(index)?;
                self.emit(Op::CoalesceFetchDim);
            }
            ExprKind::PropGetDyn { object, name, nullsafe } if !nullsafe => {
                self.expr(object)?;
                self.expr(name)?;
                self.emit(Op::PropGetDynamicSilent);
            }
            ExprKind::PropGet { object, name, nullsafe } if !nullsafe => {
                // `$o->p[k] ?? d`: an unset `$o->p` must yield null silently (no
                // "Undefined property" warning, and `__get` runs only when
                // `__isset` says the property exists) — same gate the top-level
                // `$o->p ?? d` uses.
                self.expr(object)?; // [obj]
                self.emit(Op::Dup); // [obj, obj]
                self.emit(Op::PropIsset { name: name.clone() }); // [obj, isset]
                let to_null = self.emit(Op::JumpIfFalse(Addr::MAX)); // unset → null; [obj]
                self.emit(Op::PropGetSilent { name: name.clone() }); // [value]
                let to_end = self.emit(Op::Jump(Addr::MAX));
                let null_at = self.here();
                self.patch(to_null, Op::JumpIfFalse(null_at));
                self.emit(Op::Pop); // drop the kept object
                let k = self.konst(Const::Null);
                self.emit(Op::PushConst(k)); // [null]
                let end = self.here();
                self.patch(to_end, Op::Jump(end));
            }
            _ => {
                self.expr(a)?;
            }
        }
        Ok(())
    }

    /// Open a nullsafe chain context if none is active; returns whether this
    /// link is the chain's outermost one (the root, which patches the skips).
    fn chain_enter(&mut self) -> bool {
        if self.nullsafe_chain.is_none() {
            self.nullsafe_chain = Some(Vec::new());
            true
        } else {
            false
        }
    }

    /// Root of an access chain: patch every collected `?->` skip to *here*
    /// (past the whole chain) and close the context.
    fn chain_exit(&mut self, root: bool) {
        if root {
            if let Some(sites) = self.nullsafe_chain.take() {
                let end = self.here();
                for s in sites {
                    self.patch(s, Op::JumpIfNull(end));
                }
            }
        }
    }

    /// Compile a non-chain subexpression (call arguments, a dynamic name, an
    /// index) with the chain context detached, so a `?->` inside it forms its
    /// own chain instead of short-circuiting the enclosing one.
    fn chain_pause<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = self.nullsafe_chain.take();
        let r = f(self);
        self.nullsafe_chain = saved;
        r
    }


}

/// Whether a place contains an object-property step — routing it to the mixed
/// field-path opcodes (OOP-2c) rather than the array-only path opcodes.
fn place_has_prop(place: &Place) -> bool {
    place.steps.iter().any(|s| matches!(s, PlaceStep::Prop(_) | PlaceStep::PropDyn(_)))
}

/// Whether a place has an `[]` append anywhere but the last step (`$a[][] = …`):
/// the array-only `Op::AssignPath` can't express it, so it routes through the
/// general field walker (which autovivifies through the appended child).
fn place_has_intermediate_append(place: &Place) -> bool {
    let last = place.steps.len().saturating_sub(1);
    place
        .steps
        .iter()
        .enumerate()
        .any(|(i, s)| i != last && matches!(s, PlaceStep::Append))
}

/// Map a [`Place`]'s base to the VM's write-cell selector. Only a single-step
/// array write on a local / `$GLOBALS` slot is in slice; `$this` and deeper
/// chains are rejected so the VM never sees an opcode it can't honour.
/// Extract the local slot from a `Local`-rooted, step-less place. Used by the
/// static-property by-reference-builtin RMW path, which materialises the
/// property into a temp local before running the in-place builtin on it.
fn local_slot(place: &Place) -> crate::hir::Slot {
    match place.base {
        PlaceBase::Local(s) if place.steps.is_empty() => s,
        _ => unreachable!("static_prop_rmw builds a step-less Local place for the by-ref builtin"),
    }
}

/// Convert a read expression into a `FieldBase`-rooted [`Place`] (local/global/
/// `$this` base plus property/index steps), for taking a non-variable place by
/// reference (`MakeRef`) as a by-ref builtin's first argument. Returns `None` for
/// anything not rooted at an addressable location (a call result, a literal, a
/// static property — handled separately, &c.). Nullsafe property reads are
/// excluded: `?->` is not an assignable place.
/// Declared parameter names for host builtins commonly called with named
/// arguments (php.net signatures), for the compile-time named→positional
/// rewrite in [`FnCompiler::call`]. By-reference and variadic builtins stay
/// out — their argument handling is positional-only.
fn builtin_param_names(name: &[u8]) -> Option<&'static [&'static [u8]]> {
    Some(match name.to_ascii_lowercase().as_slice() {
        b"debug_backtrace" => &[b"options", b"limit"],
        b"strlen" => &[b"string"],
        b"assert" => &[b"assertion", b"description"],
        b"htmlentities" | b"htmlspecialchars" => {
            &[b"string", b"flags", b"encoding", b"double_encode"]
        }
        b"json_encode" => &[b"value", b"flags", b"depth"],
        b"json_decode" => &[b"json", b"associative", b"depth", b"flags"],
        b"array_slice" => &[b"array", b"offset", b"length", b"preserve_keys"],
        b"array_chunk" => &[b"array", b"length", b"preserve_keys"],
        b"array_keys" => &[b"array", b"filter_value", b"strict"],
        b"array_column" => &[b"array", b"column_key", b"index_key"],
        b"array_fill" => &[b"start_index", b"count", b"value"],
        b"array_unique" => &[b"array", b"flags"],
        b"in_array" => &[b"needle", b"haystack", b"strict"],
        b"array_search" => &[b"needle", b"haystack", b"strict"],
        b"count" => &[b"value", b"mode"],
        b"implode" => &[b"separator", b"array"],
        b"explode" => &[b"separator", b"string", b"limit"],
        b"substr" => &[b"string", b"offset", b"length"],
        b"substr_count" => &[b"haystack", b"needle", b"offset", b"length"],
        b"str_pad" => &[b"string", b"length", b"pad_string", b"pad_type"],
        b"str_repeat" => &[b"string", b"times"],
        b"str_split" => &[b"string", b"length"],
        b"strpos" | b"stripos" | b"strrpos" | b"strripos" => {
            &[b"haystack", b"needle", b"offset"]
        }
        b"str_contains" | b"str_starts_with" | b"str_ends_with" => &[b"haystack", b"needle"],
        b"trim" | b"ltrim" | b"rtrim" => &[b"string", b"characters"],
        b"number_format" => {
            &[b"num", b"decimals", b"decimal_separator", b"thousands_separator"]
        }
        b"round" => &[b"num", b"precision", b"mode"],
        b"intdiv" => &[b"num1", b"num2"],
        b"intval" => &[b"value", b"base"],
        b"range" => &[b"start", b"end", b"step"],
        b"date" | b"gmdate" => &[b"format", b"timestamp"],
        b"strtotime" => &[b"datetime", b"baseTimestamp"],
        b"microtime" => &[b"as_float"],
        b"hrtime" => &[b"as_number"],
        b"iterator_to_array" => &[b"iterator", b"preserve_keys"],
        b"mb_substr" => &[b"string", b"start", b"length", b"encoding"],
        b"mb_strtolower" | b"mb_strtoupper" => &[b"string", b"encoding"],
        b"preg_split" => &[b"pattern", b"subject", b"limit", b"flags"],
        b"preg_quote" => &[b"str", b"delimiter"],
        b"file_get_contents" => {
            &[b"filename", b"use_include_path", b"context", b"offset", b"length"]
        }
        b"file_put_contents" => &[b"filename", b"data", b"flags", b"context"],
        b"str_replace" | b"str_ireplace" => &[b"search", b"replace", b"subject"],
        b"strtr" => &[b"string", b"from", b"to"],
        b"dirname" => &[b"path", b"levels"],
        b"basename" => &[b"path", b"suffix"],
        b"pathinfo" => &[b"path", b"flags"],
        b"http_build_query" => &[b"data", b"numeric_prefix", b"arg_separator", b"encoding_type"],
        b"get_object_vars" => &[b"object"],
        b"get_class_methods" => &[b"object_or_class"],
        b"class_exists" | b"interface_exists" | b"enum_exists" | b"trait_exists" => {
            &[b"class", b"autoload"]
        }
        _ => return None,
    })
}

/// Combine positional and named arguments against a builtin's declared
/// parameter names into a purely positional list. `None` when a named argument
/// is unknown, collides with a positional, repeats, or would leave a hole (the
/// builtin's default is not expressible at compile time).
fn reorder_named_args(
    args: &[Expr],
    named: &[(Box<[u8]>, Expr)],
    pnames: &[&[u8]],
) -> Option<Vec<Expr>> {
    if args.len() > pnames.len() || args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_))) {
        return None;
    }
    let mut slots: Vec<Option<Expr>> = vec![None; pnames.len()];
    for (i, a) in args.iter().enumerate() {
        slots[i] = Some(a.clone());
    }
    for (nm, ex) in named {
        let p = pnames.iter().position(|pn| *pn == &nm[..])?;
        if slots[p].is_some() {
            return None; // collides with a positional / repeated name
        }
        slots[p] = Some(ex.clone());
    }
    let filled = slots.iter().take_while(|s| s.is_some()).count();
    if slots[filled..].iter().any(|s| s.is_some()) {
        return None; // a hole would need the builtin's default value
    }
    slots.truncate(filled);
    Some(slots.into_iter().map(|s| s.expect("prefix is Some")).collect())
}

/// Peel trailing `->prop` / `[idx]` steps off a chain whose ROOT is not itself
/// a writable place (a method call, `new`, …): `reset($this->rsm()->aliasMap)`.
/// Returns the root expression and the peeled steps (outermost last); `None`
/// when there is no step to peel (the argument really is a plain expression).
fn expr_rooted_field_chain(e: &Expr) -> Option<(&Expr, Vec<PlaceStep>)> {
    let mut steps_rev: Vec<PlaceStep> = Vec::new();
    let mut cur = e;
    loop {
        match &cur.kind {
            ExprKind::Index { base, index } => {
                steps_rev.push(PlaceStep::Index((**index).clone()));
                cur = base;
            }
            ExprKind::PropGet { object, name, nullsafe: false } => {
                steps_rev.push(PlaceStep::Prop(name.clone()));
                cur = object;
            }
            ExprKind::PropGetDyn { object, name, nullsafe: false } => {
                steps_rev.push(PlaceStep::PropDyn((**name).clone()));
                cur = object;
            }
            _ => break,
        }
    }
    if steps_rev.is_empty() {
        return None;
    }
    steps_rev.reverse();
    Some((cur, steps_rev))
}

fn expr_field_place(e: &Expr) -> Option<Place> {
    match &e.kind {
        ExprKind::Var(s) => Some(Place { base: PlaceBase::Local(*s), steps: Vec::new() }),
        ExprKind::GlobalVar(s) => Some(Place { base: PlaceBase::Global(*s), steps: Vec::new() }),
        ExprKind::Superglobal(i) => Some(Place { base: PlaceBase::Superglobal(*i), steps: Vec::new() }),
        ExprKind::This => Some(Place { base: PlaceBase::This, steps: Vec::new() }),
        ExprKind::Index { base, index } => {
            let mut place = expr_field_place(base)?;
            place.steps.push(PlaceStep::Index((**index).clone()));
            Some(place)
        }
        ExprKind::PropGet { object, name, nullsafe: false } => {
            let mut place = expr_field_place(object)?;
            place.steps.push(PlaceStep::Prop(name.clone()));
            Some(place)
        }
        ExprKind::PropGetDyn { object, name, nullsafe: false } => {
            let mut place = expr_field_place(object)?;
            place.steps.push(PlaceStep::PropDyn((**name).clone()));
            Some(place)
        }
        _ => None,
    }
}

fn dim_base(place: &Place) -> R<DimBase> {
    match place.base {
        PlaceBase::Local(s) => Ok(DimBase::Local(s)),
        PlaceBase::Global(s) => Ok(DimBase::Global(s)),
        PlaceBase::Superglobal(i) => Ok(DimBase::Superglobal(i)),
        PlaceBase::This => Err(CompileError::Unsupported("$this property write".into())),
        PlaceBase::StaticProp { .. } => {
            Err(CompileError::Unsupported("static property dim base".into()))
        }
        PlaceBase::ClassConst { .. } => {
            Err(CompileError::Unsupported("class constant dim base".into()))
        }
        PlaceBase::Value(_) => {
            Err(CompileError::Unsupported("call-result dim base".into()))
        }
    }
}

/// ASCII-case-insensitive byte-string equality — PHP resolves function names
/// case-insensitively in ASCII (`STRLEN` == `strlen`).
fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(x, y)| x.eq_ignore_ascii_case(y))
}



/// HIR expression-variant name, for [`CompileError::Unsupported`].
fn expr_name(k: &ExprKind) -> String {
    let n = match k {
        ExprKind::Null => "Null",
        ExprKind::Bool(_) => "Bool",
        ExprKind::Int(_) => "Int",
        ExprKind::Float(_) => "Float",
        ExprKind::Str(_) => "Str",
        ExprKind::Const { .. } => "Const",
        ExprKind::Var(_) => "Var",
        ExprKind::VarDyn(_) => "VarDyn",
        ExprKind::VarDynAssign { .. } => "VarDynAssign",
        ExprKind::ClassConstDyn { .. } => "ClassConstDyn",
        ExprKind::StaticPropDyn { .. } => "StaticPropDyn",
        ExprKind::StaticPropDynAssign { .. } => "StaticPropDynAssign",
        ExprKind::GlobalVar(_) => "GlobalVar",
        ExprKind::Superglobal(_) => "Superglobal",
        ExprKind::GlobalsArray => "GlobalsArray",
        ExprKind::GlobalsDynAssign { .. } => "GlobalsDynAssign",
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
        ExprKind::Pipe { .. } => "Pipe",
        ExprKind::Spread(_) => "Spread",
        ExprKind::Array(_) => "Array",
        ExprKind::Index { .. } => "Index",
        ExprKind::ListAssign { .. } => "ListAssign",
        ExprKind::AssignPlace(..) => "AssignPlace",
        ExprKind::AssignOpPlace(..) => "AssignOpPlace",
        ExprKind::AssignCoalescePlace(..) => "AssignCoalescePlace",
        ExprKind::Isset(_) => "Isset",
        ExprKind::Empty(_) => "Empty",
        ExprKind::Suppress(_) => "Suppress",
        ExprKind::Print(_) => "Print",
        ExprKind::Exit(_) => "Exit",
        ExprKind::Clone(_) => "Clone",
        ExprKind::Eval(_) => "Eval",
        ExprKind::Include { .. } => "Include",
        ExprKind::Match { .. } => "Match",
        ExprKind::New { .. } => "New",
        ExprKind::MethodCall { .. } => "MethodCall",
        ExprKind::MethodCallDyn { .. } => "MethodCallDyn",
        ExprKind::PropGet { .. } => "PropGet",
        ExprKind::PropGetDyn { .. } => "PropGetDyn",
        ExprKind::This => "This",
        ExprKind::StaticCall { .. } => "StaticCall",
        ExprKind::StaticCallDyn { .. } => "StaticCallDyn",
        ExprKind::ParentHookCall { .. } => "ParentHookCall",
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
