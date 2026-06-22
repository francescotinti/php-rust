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

use std::rc::Rc;

use php_types::{ObjectInfo, PhpStr, Zval};

use crate::hir::{BinOp, CastKind, ClassId, Line, Slot, UnOp, Visibility};

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

    /// `[] -> [v]` — bind a reference between two bare locations (REF-1):
    /// `$a = &$b`. Promote `source` to a shared cell (a [`Zval::Ref`], `Undef`
    /// becoming a defined `Null`), alias `target` to the same `Rc`, and push the
    /// cell's current value (the assignment *expression* yields the aliased
    /// value). `global $x;` inside a function reuses this as
    /// `{target: Local(local), source: Global(global)}` followed by [`Op::Pop`]
    /// (D-12.2); at script scope `global` is a no-op the compiler omits.
    /// References into array elements / properties (`$x = &$a[0]`) are REF-4.
    BindRef { target: DimBase, source: DimBase },

    /// `[] -> [ref]` — push a [`Zval::Ref`] aliasing local `slot`, promoting the
    /// slot to a shared cell on first use (REF-2). The call mechanism binds this
    /// value into a by-reference parameter's callee slot, so the callee writes
    /// through to the caller's variable. Emitted only for a by-ref argument
    /// position whose argument is a plain variable.
    PushRef(Slot),

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
    /// `[v] -> [v]` — peek the top value; if it is null/undefined jump to `addr`,
    /// otherwise fall through. The value is *kept* either way (never popped). The
    /// primitive behind nullsafe `?->`: a null receiver keeps the null as the
    /// expression's result and skips the property/method access.
    JumpIfNull(Addr),

    // ----- output -----
    /// `[v] -> []` — pop, stringify (PHP string conversion), and emit to stdout.
    /// `echo a, b, c;` compiles to one `Echo` per operand.
    Echo,
    /// `[v] -> [int(1)]` — pop, stringify and emit, then push `int(1)`: `print`
    /// is an expression valued 1.
    Print,
    /// `[v] -> [string]` — convert the top value to a string honouring
    /// `__toString` on objects (OOP-3c): an object with `__toString` runs it (the
    /// stringified return flows back via `Ret`); an object without one is a fatal;
    /// any other value goes through ordinary PHP string conversion. The compiler
    /// inserts this at object→string sites (`echo`, `print`, `.` concat,
    /// `(string)`).
    Stringify,

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
    /// Write into an array path rooted at `base`, drilling through `nkeys` index
    /// values taken off the stack (pushed source-order, under the value). The
    /// final step is an append (`$a[…][] = v`) when `append`, else an index write
    /// (`$a[…][k] = v`, where `k` is the last of the `nkeys` keys). Every level is
    /// copy-on-written and auto-vivified from null/undefined/false. Stack:
    /// `[k0, …, k{nkeys-1}, v] -> [v]` (the assignment's value).
    AssignPath { base: DimBase, nkeys: u32, append: bool },
    /// Compound write `$a[…][k] op= rhs`: like [`Op::AssignPath`] but reads the
    /// current element (NULL if absent), applies `op`, and stores the result.
    /// `nkeys >= 1`; the last key is the element's. Stack:
    /// `[k0, …, k{nkeys-1}, rhs] -> [result]`.
    AssignOpPath { base: DimBase, nkeys: u32, op: BinOp },
    /// `++`/`--` on an array element `$a[…][k]`. Drills as above; `nkeys >= 1`.
    /// Stack: `[k0, …, k{nkeys-1}] -> [result]` (new value if `pre`, else old).
    IncDecPath { base: DimBase, nkeys: u32, inc: bool, pre: bool },
    /// `isset($a[…][k])` for one place: a *silent* read along the path with no
    /// auto-vivification. Pushes `true` iff every level exists and the leaf is
    /// not null. `nkeys == 0` tests a bare variable. Stack:
    /// `[k0, …, k{nkeys-1}] -> [bool]`. (`isset($a, $b)` chains these with
    /// short-circuit jumps.)
    IssetPath { base: DimBase, nkeys: u32 },
    /// `empty($a[…][k])`: like [`Op::IssetPath`] but pushes `true` when the path
    /// is absent *or* the leaf value is falsy. Stack: `[…keys] -> [bool]`.
    EmptyPath { base: DimBase, nkeys: u32 },
    /// `unset($a[…][k])` / `unset($x)`: silently remove the leaf element (or, with
    /// `nkeys == 0`, the variable itself). A missing intermediate level is a
    /// no-op. Stack: `[k0, …, k{nkeys-1}] -> []`.
    UnsetPath { base: DimBase, nkeys: u32 },

    // ----- calls & frame control -----
    /// `[arg0, arg1, …, arg{argc-1}] -> [result]` — call user function
    /// `Module::functions[func]`. The `argc` arguments are popped (they were
    /// pushed left-to-right) and bound to the callee's leading slots; when the
    /// callee returns, its result is left on the caller's operand stack. The
    /// callee runs in its own pushed [`crate::vm`] frame, so this is *not* a Rust
    /// recursion — PHP recursion grows the explicit frame stack instead.
    Call { func: u32, argc: u32 },
    /// `[arg0, …, arg{argc-1}] -> [result]` — call the by-value builtin named
    /// `name` (resolved in the [`crate::builtin::Registry`] at run time, as the
    /// tree-walker does). Arguments are popped into a `&[Zval]`; the builtin runs
    /// against a `Ctx { out, diags }` borrowed from the VM. Builtins that need the
    /// evaluator (higher-order, class-introspection, `define`/`defined`/`constant`)
    /// are *not* emitted — the compiler rejects them so the VM never sees them.
    CallBuiltin { name: Box<[u8]>, argc: u32 },
    /// `[rest0, …, rest{argc-1}] -> [result]` — call a by-reference-first builtin
    /// (`sort`, `array_push`, …): its first argument is the variable in `slot`,
    /// handed to the builtin as `&mut Zval` (write-through), and `argc` is the
    /// count of the remaining by-value arguments on the stack.
    CallBuiltinRef { name: Box<[u8]>, slot: Slot, argc: u32 },
    /// `[v] -> ` (frame ends) — pop the return value and unwind the current
    /// frame to the caller, which receives it on *its* operand stack. A function
    /// body with no explicit `return` ends with `PushConst(null); Ret`.
    Ret,

    // ----- foreach iteration -----
    /// `[iterable] -> []` — pop the iterable, snapshot it into a fresh iterator
    /// pushed on the frame's iterator stack. By-value `foreach` iterates a
    /// snapshot, so later mutation of the source array doesn't perturb the loop
    /// (PHP's copy-on-write semantics). A non-array iterates zero times for now.
    IterInit,
    /// Fetch the next element: bind it to `value` (and the key to `key`, if
    /// present) and fall through, or — when the iterator is exhausted — jump to
    /// `end` (which frees it via [`Op::IterPop`]). Operates on the top iterator.
    IterNext { value: Slot, key: Option<Slot>, end: Addr },
    /// Pop (free) the top iterator. Emitted at normal loop exhaustion and, by the
    /// compiler, on every `break`/`continue` path that leaves a `foreach`.
    IterPop,

    // ----- objects (OOP-1: instances, properties, methods, instanceof) -----
    /// `[] -> [obj]` — allocate a fresh instance of [`Module::classes`]`[class]`,
    /// its declared properties materialised from `prop_defaults`, with a fresh
    /// object id. Fatal if the class is non-instantiable (abstract / interface /
    /// enum) or could not be compiled ([`CompiledClass::ok`] false). The
    /// constructor, if any, is run by a following [`Op::InvokeMethod`].
    Alloc { class: ClassId },
    /// `[] -> [this]` — push the current frame's bound object. Fatal "Using $this
    /// when not in object context" if the frame has no `this`.
    This,
    /// `[obj] -> [value]` — read property `name` (deref-clone); a missing property
    /// (or a non-object receiver) warns and yields NULL, matching the tree-walker.
    PropGet { name: Box<[u8]> },
    /// `[obj, value] -> [value]` — write `value` into property `name` (created if
    /// absent), in place through the shared object cell. Leaves the assigned value.
    PropSet { name: Box<[u8]> },
    /// `[obj, rhs] -> [result]` — compound `$o->p op= rhs`: read the property
    /// (NULL if absent), apply `op`, store and leave the result.
    PropOpSet { name: Box<[u8]>, op: BinOp },
    /// `[obj] -> [result]` — `++`/`--` on property `name`; `pre` selects new vs old
    /// value, semantics delegated to `php_types`.
    PropIncDec { name: Box<[u8]>, inc: bool, pre: bool },
    /// `[obj] -> [bool]` — `isset($o->p)`: true iff the property exists and is not
    /// null (silent, no warning).
    PropIsset { name: Box<[u8]> },
    /// `[obj] -> []` — `unset($o->p)`: remove the property (no-op if absent).
    PropUnset { name: Box<[u8]> },
    /// `[obj, arg0, …, arg{argc-1}] -> [result]` — instance method call resolved
    /// at *run time* by walking the receiver's class `parent` chain
    /// (case-insensitive). The callee runs in a pushed frame with `$this` bound to
    /// the receiver; a missing method is a fatal (magic `__call` is OOP-3).
    MethodCall { method: Box<[u8]>, argc: u32 },
    /// `[obj, arg0, …, arg{argc-1}] -> [ret]` — like [`Op::MethodCall`] but the
    /// target method is resolved at *compile* time (`classes[class].methods[idx]`):
    /// used for the constructor, whose defining class and slot are known statically.
    InvokeMethod { class: ClassId, method_idx: u32, argc: u32 },
    /// `[value] -> [bool]` — `value instanceof classes[class]`: true if `value` is
    /// an object whose class is `class`, a subclass, or an implemented interface
    /// (transitively). A non-object yields `false`.
    InstanceOf { class: ClassId },
    /// `[value] -> [bool]` — `value instanceof static`: like [`Op::InstanceOf`]
    /// but the target is the running frame's late-static-binding class.
    InstanceOfStatic,

    // ----- OOP-2a: class context (self/parent/static), constants, static calls -----
    /// `[arg0, …, arg{argc-1}] -> [ret]` — `Class::m()` / `self::m()` /
    /// `parent::m()` / `static::m()`. The starting class comes from `target`; the
    /// method is resolved by walking its `parent` chain. The pushed frame's
    /// defining class is the resolver's, its LSB class is the caller's when
    /// `forwarding` (self/parent/static) else the start class, and `$this` is
    /// propagated per PHP's forwarding rules.
    StaticCall { target: ClassTarget, method: Box<[u8]>, forwarding: bool, argc: u32 },
    /// `[] -> [value]` — `Class::CONST` / `self::CONST` / `parent::CONST` resolved
    /// at compile time to its declaring class and constant index. Runs the
    /// constant's value *thunk* ([`CompiledConst::func`]) as a frame whose
    /// defining class is `class` (so a `self::OTHER` inside resolves), leaving the
    /// value on the caller's stack — constant expressions are pure, so re-running
    /// is sound (memoisation is a later optimisation).
    ClassConst { class: ClassId, idx: u32 },
    /// `[] -> [value]` — `static::CONST`: like [`Op::ClassConst`] but the constant
    /// is resolved at run time from the frame's LSB class (walking parents and
    /// interfaces).
    ClassConstDyn { name: Box<[u8]> },
    /// `[] -> [name]` — `static::class`: push the frame's LSB class name as a
    /// string. (`Class::class` / `self::class` / `parent::class` are folded to a
    /// [`Op::PushConst`] at compile time.)
    ClassNameStatic,
    /// `[] -> [obj]` — `new static`: allocate an instance of the frame's LSB class
    /// (its property defaults materialised, fresh id). The constructor is run by a
    /// following [`Op::InvokeCtor`] (the actual class — hence the ctor — is only
    /// known at run time).
    AllocStatic,
    /// `[obj, arg0, …, arg{argc-1}] -> [ret]` — run `obj`'s `__construct` if its
    /// class (or an ancestor) declares one, with `$this = obj`; otherwise push
    /// NULL. Used for `new static`, where the constructor can't be resolved at
    /// compile time. The instance itself is kept by the surrounding
    /// `AllocStatic; Dup; …; InvokeCtor; Pop` sequence.
    InvokeCtor { argc: u32 },
    /// `[obj] -> [ret]` — run `obj`'s class [`CompiledClass::prop_init`] thunk (if
    /// any) with `$this = obj`, materialising its non-constant property defaults;
    /// otherwise push NULL. Emitted as `Alloc; Dup; InitProps; Pop` so property
    /// defaults are set before the constructor runs. The class is read from the
    /// object at run time (so it serves `new static` too).
    InitProps,

    // ----- OOP-2b: static properties (visibility-checked, lazily initialised) -----
    /// `[] -> [value]` — read static property `target::$name` (deref-clone). The
    /// declaring class is resolved by walking the parent chain; the cell is
    /// lazily initialised (const default inline, non-const via its init thunk) and
    /// shared for the run. Visibility is enforced against the running frame's class.
    StaticPropGet { target: ClassTarget, name: Box<[u8]> },
    /// `[value] -> [value]` — write `value` into `target::$name` (through the
    /// shared cell); leaves the assigned value.
    StaticPropSet { target: ClassTarget, name: Box<[u8]> },
    /// `[rhs] -> [result]` — compound `target::$name op= rhs`.
    StaticPropOpSet { target: ClassTarget, name: Box<[u8]>, op: BinOp },
    /// `[] -> [result]` — `++`/`--` on `target::$name`.
    StaticPropIncDec { target: ClassTarget, name: Box<[u8]>, inc: bool, pre: bool },

    // ----- OOP-2c: mixed property/index write paths (`$o->a[$k]`, `$o->x->y`) -----
    /// `[keys…, value] -> [value]` — write `value` through `base` then `steps`
    /// (`Index` steps consume the pushed keys in source order). Objects navigate
    /// in place, arrays auto-vivify + copy-on-write (à la `write_into`).
    FieldAssign { base: FieldBase, steps: Box<[FieldStep]> },
    /// `[keys…, rhs] -> [result]` — compound `place op= rhs`: read the place (NULL
    /// if absent), apply `op`, write back, leave the result.
    FieldAssignOp { base: FieldBase, steps: Box<[FieldStep]>, op: BinOp },
    /// `[keys…] -> [result]` — `++`/`--` on a mixed place (read, apply, write back).
    FieldIncDec { base: FieldBase, steps: Box<[FieldStep]>, inc: bool, pre: bool },
    /// `[keys…] -> [bool]` — `isset()` of a mixed place: true iff every level
    /// exists and the leaf is non-null (silent).
    FieldIsset { base: FieldBase, steps: Box<[FieldStep]> },
    /// `[keys…] -> []` — `unset()` of a mixed place's leaf (silent no-op if absent).
    FieldUnset { base: FieldBase, steps: Box<[FieldStep]> },

    /// Raise a fatal `Error` carrying `consts[idx]` (a string) as its message.
    /// Used for *stub* function bodies: the always-present PHP prelude (exception
    /// classes, the procedural date API) contains constructs not yet ported, so
    /// those functions compile to a single `Fatal` rather than sinking every
    /// script — the fatal fires only if such a function is actually called.
    Fatal(ConstIdx),

    /// Release every tracked object the program can no longer reach
    /// (`Rc::strong_count == 1`), running `__destruct` on each, to a fixpoint
    /// (OOP-3d). Emitted by the compiler after each top-level (`main`) statement,
    /// mirroring the tree-walker's global-scope `sweep_destructors`; never inside a
    /// function/method body. A no-op when nothing is unreachable.
    Sweep,

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

/// The root of a mixed property/index write path ([`Op::FieldAssign`] &c.): a
/// local slot, a `$GLOBALS` slot, or `$this`. Unlike [`DimBase`] this admits
/// `This`, since a mixed path may begin at an object property of `$this`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldBase {
    Local(Slot),
    Global(Slot),
    This,
}

/// One step of a mixed write path ([`Op::FieldAssign`] &c.). `Index` consumes a
/// key from the operand stack (keys are pushed in source order, beneath the
/// value); `Prop` carries its name inline; `Append` is `[]` (final step only).
/// Objects are navigated in place (no copy-on-write); arrays auto-vivify and
/// copy-on-write, exactly as the tree-walker's `write_into`.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldStep {
    Index,
    Append,
    Prop(Box<[u8]>),
}

/// The class a `::`-qualified op ([`Op::StaticCall`], `instanceof static`) starts
/// from. `self`/`parent` and a named class are resolved to a concrete [`ClassId`]
/// at compile time; `static::` is the run-time late-static-binding class, read
/// from the executing frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClassTarget {
    /// A class id known at compile time (named class, `self`, or `parent`).
    Class(ClassId),
    /// `static::` — resolved at run time from the frame's LSB class.
    Static,
}

/// Whether a class can be instantiated, and if not, why — so [`Op::Alloc`] can
/// raise the same fatal PHP does (`Cannot instantiate {abstract class,interface,
/// enum} X`). Derived from [`crate::hir::ClassDecl`] at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instantiable {
    Yes,
    Abstract,
    Interface,
    Enum,
}

/// A method compiled onto a class: its name (matched case-insensitively at
/// dispatch) and body [`Func`]. The index in [`CompiledClass::methods`] matches
/// the source [`crate::hir::ClassDecl::methods`] order, so a compile-time method
/// resolution ([`Op::InvokeMethod`]) addresses the same slot.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledMethod {
    pub name: Box<[u8]>,
    /// Declared visibility, enforced against the calling frame's class (OOP-2b).
    pub visibility: Visibility,
    pub func: Func,
}

/// How a static property's initial value is produced. A constant default is
/// materialised inline on first access; a non-constant one (array / expression /
/// class constant) runs its thunk [`Func`] in the declaring class's context, the
/// result stored into the persistent cell (see [`Op::StaticPropGet`]).
#[derive(Debug, Clone, PartialEq)]
pub enum StaticInit {
    Const(Const),
    Thunk(Func),
}

/// A static property declared on a class (OOP-2b): its name, visibility, and how
/// its initial value is produced. The persistent cell lives in the VM, keyed by
/// (declaring class, name), and is created on first access.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledStaticProp {
    pub name: Box<[u8]>,
    pub visibility: Visibility,
    pub init: StaticInit,
}

/// A class constant compiled onto a class (same index space as the source
/// [`crate::hir::ClassDecl::consts`]): its name and a *thunk* [`Func`] whose body
/// evaluates the constant's value expression and returns it. Run on demand by
/// [`Op::ClassConst`] / [`Op::ClassConstDyn`] in the declaring class's context.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledConst {
    pub name: Box<[u8]>,
    pub func: Func,
}

/// Compile-time class metadata, in the same index space as
/// [`crate::hir::Program::classes`] / [`ClassId`] (a [`ClassRef::Named`] resolved
/// to `classes[i]` in the HIR maps to `classes[i]` here). The VM consults this at
/// `new` / property / method / `instanceof` dispatch.
///
/// [`ClassRef::Named`]: crate::hir::ClassRef::Named
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledClass {
    /// Name as written (original case).
    pub name: Box<[u8]>,
    /// The name as a shared [`PhpStr`], stamped into each instance's
    /// [`php_types::Object::class_name`] without re-allocating.
    pub class_name: Rc<PhpStr>,
    /// Superclass, resolved to its [`ClassId`] at lowering; `None` for a root.
    pub parent: Option<ClassId>,
    /// Implemented interfaces (resolved ids); `instanceof` walks them transitively.
    pub interfaces: Vec<ClassId>,
    /// Whether `new` on this class is allowed, and the fatal reason if not.
    pub instantiable: Instantiable,
    /// Effective instance properties, parent-first and flattened (a redeclared
    /// property keeps its inherited position with the most-derived default), each
    /// with its constant default materialised by [`Const::to_zval`].
    pub prop_defaults: Vec<(Box<[u8]>, Const)>,
    /// Declared-property visibility shape (for `var_dump`), shared by all instances.
    pub info: Rc<ObjectInfo>,
    /// Methods declared *on this class* (resolution walks `parent` at run time).
    pub methods: Vec<CompiledMethod>,
    /// Instance properties *declared directly on this class* with their
    /// visibility, in declaration order (OOP-2b). Visibility resolution
    /// (`$o->p` access checks) walks the parent chain looking at each class's own
    /// list; the *declaring* class is the one whose list contains the property.
    pub own_prop_vis: Vec<(Box<[u8]>, Visibility)>,
    /// Static properties declared *on this class* (OOP-2b); resolution walks the
    /// parent chain. The live cells are keyed by (declaring class, name) in the VM.
    pub static_props: Vec<CompiledStaticProp>,
    /// Thunk that materialises this class's *non-constant* instance-property
    /// defaults (`This; <expr>; PropSet; …`), run with `$this` = the new object by
    /// [`Op::InitProps`]. `None` when every default folded to a constant. Covers
    /// the flattened (parent-first) property set, so it is complete for the class.
    pub prop_init: Option<Func>,
    /// Class constants declared *on this class* (same index space as the source
    /// [`crate::hir::ClassDecl::consts`]); resolution walks `parent` then
    /// interfaces at run time.
    pub consts: Vec<CompiledConst>,
    /// `false` if the class could not be fully compiled (e.g. a non-constant
    /// property default): [`Op::Alloc`] on it fatals instead of producing a
    /// wrong instance, mirroring the function-stub discipline.
    pub ok: bool,
}

/// A whole compiled program: the script body plus the flat function / closure /
/// class tables, indexed exactly as the source [`crate::hir::Program`] indexes
/// them (so a call resolved to `functions[i]` in the HIR maps to `functions[i]`
/// here, and likewise for classes).
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
    /// Compiled class metadata — same index space as
    /// [`crate::hir::Program::classes`] / [`ClassId`].
    pub classes: Vec<CompiledClass>,
    /// Source file name, reproduced verbatim in diagnostics (`… in <file> on
    /// line N`), carried over from [`crate::hir::Program::file`].
    pub file: Box<[u8]>,
}
