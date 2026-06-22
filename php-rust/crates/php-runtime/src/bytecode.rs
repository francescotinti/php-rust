//! Bytecode: the instruction set the VM executes, and the program structures
//! that hold it (VM-migration Fase 2).
//!
//! # Why this exists
//!
//! Today the runtime *tree-walks* the [`crate::hir`] directly ([`crate::eval`]).
//! That is correct but structurally hostile to suspendable / non-structured
//! control flow: generators ride a stackful `corosensei` coroutine plus an
//! `unsafe` `*mut Evaluator` reborrow, and `goto` / `break N` propagate signal
//! enums up the Rust recursion. A flat instruction stream with an explicit
//! instruction pointer makes all of that ordinary: a generator is a frame whose
//! `ip` is parked at a `Yield`, a `goto` is a `Jump`.
//!
//! This module defines only the *instruction set* and the *program* it lives in.
//! The compiler (HIR → bytecode) is `crate::compile`; the dispatch loop and the
//! runtime frame (`ip` + operand stack + slots) are `crate::vm`.
//!
//! # Execution model: stack-based, slot-addressed locals
//!
//! The VM is a **stack machine** for expression evaluation, with **named locals
//! addressed by [`Slot`]** (the resolution the HIR already did — see
//! [`crate::hir::Program::slots`] / [`crate::hir::FnDecl::slots`]). This is the
//! CPython/JVM shape, deliberately *not* a register-allocated machine:
//!
//! - it makes the compiler a trivial post-order emit (no temporary-register
//!   allocator), so getting to behavioural parity with the tree-walker is fast
//!   and low-risk — the priority for the migration;
//! - it makes a generator a single saveable thing: park `ip`, keep `slots` and
//!   the operand `stack`, resume later. One coroutine, no native stack, no
//!   `unsafe`.
//!
//! Register allocation (collapsing the operand stack into flat registers à la
//! Lua/Zend) is a *later* optimisation that can be layered on without changing
//! this contract.
//!
//! ## Stack discipline (the invariant the compiler must uphold)
//!
//! - **Every compiled expression leaves exactly one value on the operand stack.**
//! - **Every compiled statement leaves the stack at the depth it found it.**
//!   (An expression-statement therefore compiles to `<expr>` followed by [`Op::Pop`].)
//!
//! Value semantics (arithmetic, comparison, type juggling, string conversion)
//! are *not* re-implemented here: the VM delegates [`Op::Binary`] / [`Op::Unary`]
//! / [`Op::Cast`] to `php_types::ops` / `php_types::convert`, exactly as the
//! tree-walker does. The bytecode only encodes *control* and *data movement*.
//!
//! # Scope of this first cut (Tier 1)
//!
//! The opcodes below cover the tree-walker's Tier-1 surface: echo/print,
//! literals, local read/write, compound and inc/dec assignment to a bare slot,
//! the binary/unary/cast operators, structured control flow (if / while /
//! do-while / for / ternary / short-circuit `&&` `||`, and `break N` /
//! `continue N`, all lowered to [`Op::Jump`] families at compile time), and
//! `return`. Everything else in the HIR maps to opcodes added as coverage is
//! ported — see [the extension map](#extension-map) — so this enum is expected
//! to grow; it is intentionally non-exhaustive of PHP today.
//!
//! ## Extension map
//!
//! HIR construct → planned opcode(s), added when that coverage is ported:
//!
//! - references (`$a = &$b`, by-ref params, `foreach &$v`) → slot/element
//!   reference fetch + alias ops (the `Zval::Ref(Rc<RefCell<_>>)` cell is reused
//!   verbatim from `php-types`);
//! - arrays / `Place` chains (`$a[k]`, `$o->p`, `$a[] = …`) → `NewArray`,
//!   `FetchDim` / `FetchProp` (read and write/append flavours), `WriteBack`;
//! - `??` / `??=` / `isset` / `empty` → null-aware peek-and-jump + a non-warning
//!   slot/place read;
//! - calls (`Call` / `CallDynamic` / `MethodCall` / `StaticCall` / `New`) →
//!   `Call*` ops with an argument-passing convention over the operand stack;
//! - closures / first-class callables → `MakeClosure(fn_idx)`;
//! - `match` / `switch` → jump tables built at compile time;
//! - `try`/`catch`/`finally`, `throw` → an exception-handler table per `Func`
//!   plus `Throw`;
//! - generators (`yield`, `yield from`) → `Yield` / `YieldFrom`, the payoff that
//!   retires `corosensei`;
//! - classes/enums/static props/consts → method bodies compile to [`Func`]s; the
//!   class metadata stays in the HIR [`ClassDecl`] table the VM consults.

use php_types::{PhpStr, Zval};

use crate::hir::{BinOp, CastKind, Line, Slot, UnOp};

/// Index into a [`Func`]'s instruction vector ([`Func::ops`]); also the form a
/// jump target takes. `u32` is plenty (PHP function bodies are tiny) and keeps
/// [`Op`] small.
pub type Addr = u32;

/// Index into a [`Func`]'s constant pool ([`Func::consts`]).
pub type ConstIdx = u32;

/// A compile-time literal, materialised into a [`Zval`] at run time by
/// [`Const::to_zval`]. Kept as a plain, `Clone`-cheap, structurally-comparable
/// value (no `Rc`) so a [`Func`] is `Clone`/`PartialEq` and can be cached by a
/// resident process, mirroring the HIR's own ownership discipline.
#[derive(Debug, Clone, PartialEq)]
pub enum Const {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// A byte string (PHP strings are byte strings, not UTF-8).
    Str(Box<[u8]>),
}

impl Const {
    /// Materialise this literal into a runtime value. The mapping mirrors the
    /// tree-walker's `const_literal_to_zval` (`eval`): `Int → Long`,
    /// `Float → Double`, `Str → Zval::Str(PhpStr::new(..))`.
    pub fn to_zval(&self) -> Zval {
        match self {
            Const::Null => Zval::Null,
            Const::Bool(b) => Zval::Bool(*b),
            Const::Int(i) => Zval::Long(*i),
            Const::Float(f) => Zval::Double(*f),
            Const::Str(b) => Zval::Str(PhpStr::new(b.clone())),
        }
    }
}

/// The storable cell a dimension write ([`Op::AssignDim`] / [`Op::AppendDim`])
/// is rooted at. Reads don't need this — they consume a base *value* off the
/// stack — but a write must reach back into a real cell to persist (and to
/// copy-on-write the array in place), so it names the slot directly. `Global`
/// targets the script (bottom) frame, for `$GLOBALS['x'][…] = …`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DimBase {
    Local(Slot),
    Global(Slot),
}

/// One VM instruction. Operands are immediate (slots, constant-pool indices,
/// jump addresses); runtime values flow through the frame's operand stack.
///
/// Unless stated otherwise, an op's stack effect is written as
/// `[before] -> [after]` over the *top* of the operand stack.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    // ----- constants & operand-stack housekeeping -----
    /// `[] -> [v]` — push `consts[idx]` materialised via [`Const::to_zval`].
    PushConst(ConstIdx),
    /// `[v] -> []` — discard the top value (statement-level result cleanup).
    Pop,
    /// `[v] -> [v, v]` — duplicate the top value. Used to let an assignment
    /// *expression* leave the assigned value while still storing it.
    Dup,

    // ----- locals (slot-addressed) -----
    /// `[] -> [v]` — push the value in local `slot`. Reading an unset slot
    /// follows PHP's "undefined variable" semantics in the VM (warning + NULL),
    /// matching the tree-walker.
    LoadSlot(Slot),
    /// `[v] -> []` — pop and store into local `slot`. To use an assignment as an
    /// expression, the compiler emits [`Op::Dup`] before this.
    StoreSlot(Slot),
    /// `++`/`--` on a bare local. `inc` selects increment vs decrement, `pre`
    /// selects whether the pushed result is the new value (prefix) or the old
    /// value (postfix). Stack: `[] -> [result]`. Semantics (string increment,
    /// `null++ == 1`, …) are delegated to `php_types`.
    IncDecSlot { slot: Slot, inc: bool, pre: bool },

    // ----- operators (semantics delegated to php_types::ops / ::convert) -----
    /// `[lhs, rhs] -> [result]` — pop rhs then lhs, push `lhs <op> rhs`.
    Binary(BinOp),
    /// `[v] -> [result]` — unary `-`, `+`, `!`, `~`.
    Unary(UnOp),
    /// `[v] -> [result]` — a type cast like `(int)$x`.
    Cast(CastKind),

    // ----- control flow (targets are resolved instruction addresses) -----
    /// Unconditional jump to `addr`. Encodes `goto`, loop back-edges, and the
    /// skip-arms of `if`/ternary/short-circuit.
    Jump(Addr),
    /// `[cond] -> []` — pop; jump to `addr` if the value is falsy (PHP truthiness).
    JumpIfFalse(Addr),
    /// `[cond] -> []` — pop; jump to `addr` if the value is truthy.
    JumpIfTrue(Addr),
    /// `[v] -> [v]` if `v` is *not* null/undefined (jump to `addr`, value kept);
    /// `[v] -> []` otherwise (fall through, value discarded). The primitive
    /// behind `??` and `??=`: the left operand is read silently, and the right is
    /// evaluated only when the left is null.
    JumpIfNotNull(Addr),

    // ----- output -----
    /// `[v] -> []` — pop, stringify (PHP string conversion), and emit to stdout.
    /// `echo a, b, c;` compiles to one `Echo` per operand.
    Echo,
    /// `[v] -> [int(1)]` — pop, stringify and emit, then push `int(1)`: `print`
    /// is an expression valued 1.
    Print,

    // ----- arrays & dimensions -----
    /// `[] -> [array()]` — push a fresh empty array. An array literal compiles to
    /// `ArrayInit` followed by one `ArrayPush` / `ArrayInsert` per element, so the
    /// growing array stays on the stack under the element operands.
    ArrayInit,
    /// `[array, v] -> [array]` — append `v` to the array (next integer key).
    ArrayPush,
    /// `[array, key, v] -> [array]` — insert `v` at `key` (key coerced per PHP).
    ArrayInsert,
    /// `[base, key] -> [v]` — read `base[key]` by value (array element or string
    /// offset); a missing key / non-subscriptable base yields NULL. Read context
    /// is silent in the proof slice (the undefined-key warning rides the
    /// diagnostics-ordering work, like the undefined-variable notice).
    FetchDim,
    /// `[key, v] -> [v]` — store `v` into `base[key]`, copy-on-writing the array
    /// in the rooted cell and auto-vivifying it from null/undefined. Leaves `v`
    /// (the assignment's value).
    AssignDim(DimBase),
    /// `[v] -> [v]` — append `v` to the array in the rooted cell (`$a[] = v`),
    /// auto-vivifying from null/undefined. Leaves `v`.
    AppendDim(DimBase),

    // ----- calls & frame control -----
    /// `[arg0, arg1, …, arg{argc-1}] -> [result]` — call user function
    /// `Module::functions[func]`. The `argc` arguments are popped (they were
    /// pushed left-to-right) and bound to the callee's leading slots; when the
    /// callee returns, its result is left on the caller's operand stack. The
    /// callee runs in its own pushed [`crate::vm`] frame, so this is *not* a Rust
    /// recursion — PHP recursion grows the explicit frame stack instead.
    Call { func: u32, argc: u32 },
    /// `[v] -> ` (frame ends) — pop the return value and unwind the current
    /// frame to the caller, which receives it on *its* operand stack. A function
    /// body with no explicit `return` ends with `PushConst(null); Ret`.
    Ret,

    /// Raise a fatal `Error` carrying `consts[idx]` (a string) as its message.
    /// Used for *stub* function bodies: the always-present PHP prelude (exception
    /// classes, the procedural date API) contains constructs not yet ported, so
    /// those functions compile to a single `Fatal` rather than sinking every
    /// script — the fatal fires only if such a function is actually called.
    Fatal(ConstIdx),

    /// No-op. Kept so a [`crate::hir::StmtKind::Nop`] / `Label` has a stable
    /// address to compile pass-throughs against without special-casing empty
    /// instruction ranges.
    Nop,
}

/// A compiled callable: the top-level script body, a user function, a closure
/// body, or (later) a method body. Self-contained — owns its instructions and
/// constant pool — so it can outlive the parser arena and be cached, mirroring
/// [`crate::hir::FnDecl`].
#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    /// Name as written (original case); empty for the script body / an anonymous
    /// closure. Calls match it ASCII-case-insensitively, like the HIR.
    pub name: Box<[u8]>,
    /// The instruction stream. Jump targets are indices into this vector.
    pub ops: Vec<Op>,
    /// Per-function constant pool, indexed by [`Op::PushConst`].
    pub consts: Vec<Const>,
    /// Size of the frame's slot array (named locals). The compiler copies this
    /// from the source [`crate::hir::FnDecl::slots`] length (or
    /// [`crate::hir::Program::slots`] for the script body).
    pub n_slots: u32,
    /// Number of formal parameters, occupying the leading `n_params` slots
    /// (`params[i].slot == i`, as the HIR guarantees).
    pub n_params: u32,
    /// `function &f()` — returns by reference (carried through for the by-ref
    /// call/return path, ported later).
    pub by_ref: bool,
    /// The body contains a `yield` — calling it produces a `Generator` rather
    /// than running the body. Drives generator setup once `Yield` is wired in.
    pub is_generator: bool,
    /// Source line of the declaration, for diagnostics / stack traces.
    pub line: Line,
}

/// A whole compiled program: the script body plus the flat function/closure
/// tables, indexed exactly as the source [`crate::hir::Program`] indexes them
/// (so a call resolved to `functions[i]` in the HIR maps to `functions[i]` here).
///
/// Class metadata is intentionally absent for now: when OOP is ported, method
/// bodies compile into [`Func`]s while the structural class table continues to
/// live in the HIR [`crate::hir::ClassDecl`] the VM consults at dispatch time.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    /// The top-level script body (the implicit `main`).
    pub main: Func,
    /// Top-level user-defined functions, hoisted — same index space as
    /// [`crate::hir::Program::functions`].
    pub functions: Vec<Func>,
    /// Anonymous / arrow-function bodies — same index space as
    /// [`crate::hir::Program::closures`].
    pub closures: Vec<Func>,
    /// Source file name, reproduced verbatim in diagnostics (`… in <file> on
    /// line N`), carried over from [`crate::hir::Program::file`].
    pub file: Box<[u8]>,
}
