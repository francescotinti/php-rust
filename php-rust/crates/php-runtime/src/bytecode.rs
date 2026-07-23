//! Bytecode: the instruction set the VM executes, and the program structures
//! that hold it (VM-migration Fase 2).
//!
//! # Why this exists
//!
//! The runtime originally *tree-walked* the [`crate::hir`] directly. That was
//! correct but structurally hostile to suspendable / non-structured control flow:
//! generators rode a stackful `corosensei` coroutine plus an `unsafe`
//! `*mut Evaluator` reborrow, and `goto` / `break N` propagated signal enums up
//! the Rust recursion. A flat instruction stream with an explicit instruction
//! pointer makes all of that ordinary: a generator is a frame whose `ip` is parked
//! at a `Yield`, a `goto` is a `Jump`. The VM is now the sole engine — the
//! tree-walker (and `corosensei`, and the `unsafe` reborrow) have been deleted.
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
//!   retired `corosensei`;
//! - classes/enums/static props/consts → method bodies compile to [`Func`]s; the
//!   class metadata stays in the HIR [`ClassDecl`] table the VM consults.

use std::rc::Rc;

/// Fx-hashed (see vm/mod.rs): read on hot runtime paths — `class_index` on
/// class-name resolution, `prop_info` on every property access.
type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

use php_types::{ObjectInfo, PhpStr, Zval};

use crate::hir::{BinOp, Capture, CastKind, ClassId, IncludeMode, Line, Slot, TypeHint, UnOp, Visibility};

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
    /// A byte string (PHP strings are byte strings, not UTF-8). Stored as a
    /// prebuilt shared `ZStr` so `to_zval` (Op::PushConst — the hottest
    /// materialization) is a refcount bump instead of a byte copy.
    Str(php_types::ZStr),
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
            Const::Str(b) => Zval::Str(Rc::clone(b)),
        }
    }
}

/// Case-insensitive symbol hash (WP-29 B1): FxHash over the ASCII-lowercased
/// bytes. The lowercased copy lives on the stack (identifiers are short) so a
/// LOOKUP never allocates, and the hasher sees one slice write — per-byte
/// `write_u8` ran a full Fx round per byte and profiled SLOWER than the
/// legacy scan on small method tables. Build and lookup share this exact
/// procedure.
pub fn ci_hash(name: &[u8]) -> u64 {
    use std::hash::Hasher;
    let mut h = rustc_hash::FxHasher::default();
    if name.len() <= 64 {
        let mut buf = [0u8; 64];
        for (d, s) in buf.iter_mut().zip(name) {
            *d = s.to_ascii_lowercase();
        }
        h.write(&buf[..name.len()]);
    } else {
        h.write(&name.to_ascii_lowercase());
    }
    h.finish()
}

thread_local! {
    /// Epoch of validity for the [`PropIc`] inline caches. Class ids are
    /// GLOBAL per run but modules (and their ops, prelude included) are
    /// Rc-shared across unit links and — in a resident process — across
    /// requests, where the same numeric id can name a DIFFERENT class.
    /// Bumping the epoch at each `Vm` construction invalidates every cache
    /// in O(1). Starts at 1 and never returns 0 (0 = the empty-cache tag).
    static IC_EPOCH: std::cell::Cell<u32> = const { std::cell::Cell::new(1) };
}

/// Invalidate all [`PropIc`] caches (a new run's id space). Called by
/// `Vm::new`.
pub fn bump_ic_epoch() {
    IC_EPOCH.with(|e| {
        let next = e.get().wrapping_add(1);
        e.set(if next == 0 { 1 } else { next });
    });
}

#[inline]
fn ic_epoch() -> u32 {
    IC_EPOCH.with(|e| e.get())
}

/// Monomorphic per-op-site property cache (WP-29, lo Zend inline cache;
/// SCOPE-AWARE dal WP-35): `(epoch, class_id + 1, scope_id + 1 | 0, slot)`
/// dell'ultima risoluzione cache-abile; `class_id+1 == 0` = vuota, epoch ≠
/// corrente = stantia (id di un run precedente). Lo SCOPE chiamante fa
/// parte della CHIAVE: un hit vale solo per la stessa coppia
/// (classe receiver, scope) che ha riempito la cella, quindi anche gli
/// esiti private/protected sono cache-abili — `Closure::bind` che porta un
/// altro scope sul sito produce un MISS, mai un hit errato (la lezione
/// WP-29 "mai cachare visibilità non-public" valeva per celle keyed sulla
/// sola classe receiver). Le letture (GET/ISSET) fillano qualunque
/// visibilità purché il canale hook sia strutturalmente assente per
/// (classe, prop); le scritture restano `plain_set_props`-only.
///
/// La cella è `Rc`-condivisa: il clone dell'op deve puntare alla STESSA
/// cache perché i fill persistano sul sito. Lo stato resta invisibile
/// all'uguaglianza strutturale (`Func` è `PartialEq` per la unit-cache:
/// due compilazioni identiche DEVONO confrontare uguali qualunque cosa la
/// VM abbia cachato).
#[derive(Debug)]
pub struct PropIc(Rc<std::cell::Cell<(u32, u32, u32, u32)>>);

impl PropIc {
    /// The cached `(class_id + 1, slot)` when filled IN THIS RUN for
    /// exactly this calling scope (see [`PropIc::scope_key`]).
    #[inline]
    pub fn get(&self, scope_key: u32) -> Option<(u32, u32)> {
        let (epoch, cid1, sk, slot) = self.0.get();
        (cid1 != 0 && sk == scope_key && epoch == ic_epoch()).then_some((cid1, slot))
    }

    #[inline]
    pub fn fill(&self, class_id: u32, scope_key: u32, slot: u32) {
        self.0.set((ic_epoch(), class_id + 1, scope_key, slot));
    }

    /// Key form of a calling scope: `ClassId + 1`, `0` for no scope
    /// (global code / free functions).
    #[inline]
    pub fn scope_key(cur: Option<crate::hir::ClassId>) -> u32 {
        match cur {
            Some(c) => c as u32 + 1,
            None => 0,
        }
    }
}

impl Default for PropIc {
    fn default() -> Self {
        PropIc(Rc::new(std::cell::Cell::new((0, 0, 0, 0))))
    }
}

impl PartialEq for PropIc {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Clone for PropIc {
    fn clone(&self) -> Self {
        PropIc(Rc::clone(&self.0))
    }
}

/// Monomorphic per-op-site METHOD cache (WP-30, il gemello di [`PropIc`] per
/// il dispatch): `(epoch, receiver class_id + 1, defining ClassId, method
/// idx)` dell'ultima risoluzione cache-abile. Riempita SOLO per esiti
/// scope-indipendenti: vincitore **public** e — per le chiamate d'istanza —
/// nessun antenato proprio che dichiari un metodo `private` omonimo
/// (altrimenti `parent_private_rebind` renderebbe la risoluzione dipendente
/// dallo scope chiamante, e `Closure::bind` può portare QUALSIASI scope su
/// questo sito). Stesso contratto di PropIc: cella `Rc`-condivisa tra i
/// cloni dell'op, stato invisibile all'uguaglianza strutturale, epoch
/// per-run contro gli id stantii.
#[derive(Debug)]
pub struct MethodIc(Rc<std::cell::Cell<(u32, u32, u32, u32)>>);

impl MethodIc {
    /// The cached `(defining ClassId, method idx)` when filled IN THIS RUN
    /// for exactly this receiver class.
    #[inline]
    pub fn get(&self, cid: usize) -> Option<(usize, usize)> {
        let (epoch, cid1, defc, midx) = self.0.get();
        (cid1 as usize == cid + 1 && epoch == ic_epoch())
            .then_some((defc as usize, midx as usize))
    }

    #[inline]
    pub fn fill(&self, cid: usize, defc: usize, midx: usize) {
        self.0
            .set((ic_epoch(), cid as u32 + 1, defc as u32, midx as u32));
    }
}

impl Default for MethodIc {
    fn default() -> Self {
        MethodIc(Rc::new(std::cell::Cell::new((0, 0, 0, 0))))
    }
}

impl PartialEq for MethodIc {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Clone for MethodIc {
    fn clone(&self) -> Self {
        MethodIc(Rc::clone(&self.0))
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
    /// A data superglobal (`$_SERVER[$k] = …`), addressed by name via the
    /// VM-level superglobal store so it persists across units/frames. The `u8` is
    /// an index into [`SUPERGLOBAL_NAMES`].
    Superglobal(u8),
}

/// The fixed PHP data superglobals, indexed by the id carried in superglobal
/// HIR/bytecode nodes and the VM's superglobal store. `$GLOBALS` is excluded —
/// it has its own dedicated machinery (the script-frame slots).
pub const SUPERGLOBAL_NAMES: [&[u8]; 8] = [
    b"_SERVER", b"_GET", b"_POST", b"_ENV", b"_FILES", b"_COOKIE", b"_REQUEST", b"_SESSION",
];

/// The superglobal store index for `name` (`_SERVER` → 0, …), or `None` if the
/// name is not a data superglobal.
pub fn superglobal_index(name: &[u8]) -> Option<u8> {
    SUPERGLOBAL_NAMES.iter().position(|n| *n == name).map(|i| i as u8)
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
    /// `[] -> [v]` — like [`Op::LoadSlot`] but for a *source-level* variable read
    /// (`$x` in value position): when the slot is `Undef`, raise
    /// `Warning: Undefined variable $name` (`consts[name]` is the bare name) before
    /// yielding NULL. `LoadSlot` stays silent for compiler temporaries and contexts
    /// PHP does not warn in (`isset`/`??`/`empty`/`@`).
    LoadVar { slot: Slot, name: ConstIdx },
    /// `[] -> [Undef]` — push the `Undef` sentinel, used to leave a skipped
    /// optional parameter unbound in a named call (PAR) so the callee's default
    /// prologue fills it.
    PushUndef,
    /// `[v] -> []` — pop and store into local `slot`. To use an assignment as an
    /// expression, the compiler emits [`Op::Dup`] before this.
    StoreSlot(Slot),
    /// `[a, b] -> [b, a]` — swap the top two stack entries. Used to realise the
    /// pipe operator `$x |> $f`, whose operands evaluate left-to-right (input then
    /// callable) but must reach [`Op::CallValue`] in callee-then-arg stack order.
    Swap,

    // ----- globals (`$GLOBALS['literal']`, addressed in the script frame) -----
    /// `[] -> [v]` — push the value of global `slot` (a slot in the script/main
    /// frame, `frames[0]`). The read form of `$GLOBALS['x']`, reachable from
    /// inside a function (step 12-3). Follows a reference like [`Op::LoadSlot`].
    LoadGlobal(Slot),
    /// `[v] -> []` — pop and store into global `slot` (script frame). The write
    /// form of `$GLOBALS['x'] = …`; creates/overwrites the global. As with
    /// `StoreSlot`, the compiler emits [`Op::Dup`] first to value the assignment.
    StoreGlobal(Slot),
    /// `[] -> [v]` — `++`/`--` on global `slot` (`$GLOBALS['x']++`), pushing the
    /// pre- or post-value. The global analogue of [`Op::IncDecSlot`].
    IncDecGlobal { slot: Slot, inc: bool, pre: bool },
    // ----- data superglobals (`$_SERVER`, …, addressed by name) -----
    /// `[] -> [v]` — push the value of data superglobal `idx` (an index into
    /// [`SUPERGLOBAL_NAMES`]). Unlike [`Op::LoadGlobal`] this resolves by name in
    /// the VM-level superglobal store, so it reads correctly from any unit/frame
    /// (e.g. an included file). Silent like `LoadGlobal`.
    LoadSuperglobal(u8),
    /// `[v] -> []` — pop and store into data superglobal `idx`. The write form of
    /// `$_SERVER = …`; the compiler emits [`Op::Dup`] first to value the assignment.
    StoreSuperglobal(u8),
    /// `[] -> [v]` — `++`/`--` on data superglobal `idx`, pushing the pre/post-value.
    IncDecSuperglobal { idx: u8, inc: bool, pre: bool },
    /// `[base, key] -> [v]` — a `list()`-destructuring element read: like
    /// [`Op::FetchDim`] (undefined-key Warning included) but SILENT on a
    /// non-array scalar base — `list($a) = null` does not raise the
    /// "Trying to access array offset on null" warning `$null[0]` would.
    FetchDimList,
    /// `[] -> [arr]` — a *bare* `$GLOBALS` read: snapshot the script frame's
    /// named locals (by `slot_names`) plus the seeded data superglobals into a
    /// fresh array (PHP 8.1 read-only-copy semantics).
    LoadGlobals,
    /// `[key, v] -> [v]` — `$GLOBALS[$name] = v` with a runtime key: resolve
    /// (or create) the named global slot in the bottom frame and assign.
    GlobalsDynAssign,
    /// Default-parameter prologue (PAR): if `slot` already holds an argument
    /// (it is not `Undef`), jump to `skip` (past the default); otherwise fall
    /// through to evaluate the default expression and `StoreSlot` it. Emitted at
    /// function entry for each parameter that has a default.
    FillDefault { slot: Slot, skip: Addr },
    /// Coerce a just-filled default parameter value in `slot` to its scalar type
    /// `hint` (step 14, D-NEW-6): `function f(float $n = 0)` stores `0.0`, not `0`.
    /// Emitted in the prologue right after a hinted optional parameter's default is
    /// stored. Best-effort — a valid constant default always coerces, so on the
    /// unreachable failure the stored value is left as-is (no TypeError here).
    CoerceParam { slot: Slot, hint: TypeHint },
    /// Arity guard (PAR), emitted at function entry when there is at least one
    /// required parameter: if fewer than `required` arguments were passed, raise
    /// an `ArgumentCountError`. `exactly` selects the wording ("exactly N" when
    /// there are no optional/variadic params, else "at least N").
    CheckArity { required: u32, exactly: bool },
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

    /// `static $v = init;` first half (step 15 / VM port): if the program-global
    /// static cell `id` already exists, jump past the initialiser to `skip` (the
    /// matching [`Op::StaticAlias`]); otherwise fall through to run the
    /// initialiser once. `id` indexes `Vm::statics` (sized by `Module::static_count`).
    StaticGuard { id: u32, skip: Addr },
    /// `[init] -> []` — pop the just-evaluated initialiser and store it as static
    /// cell `id`'s first (and only) value. Reached only on the first execution.
    StaticStore { id: u32 },
    /// `[] -> []` — alias local `slot` to static cell `id` (`slot = Ref(cell)`), so
    /// reads/writes of the variable go through the persistent cell. Runs on every
    /// call (after the guard), giving `static $x` its cross-call persistence.
    StaticAlias { slot: Slot, id: u32 },

    /// `[] -> [ref]` — push a [`Zval::Ref`] aliasing local `slot`, promoting the
    /// slot to a shared cell on first use (REF-2). The call mechanism binds this
    /// value into a by-reference parameter's callee slot, so the callee writes
    /// through to the caller's variable. Emitted only for a by-ref argument
    /// position whose argument is a plain variable.
    PushRef(Slot),

    /// `[keys…] -> [ref]` — REF-4. Navigate a place (a local/global/`$this` base
    /// plus `Index` steps; the keys are on the stack in source order), promoting
    /// the addressed location to a shared cell, and push a [`Zval::Ref`] to it.
    /// With no steps this is the stepped generalisation of [`Op::PushRef`] over a
    /// `FieldBase`. The reference *source* of `$x = &$a[0]` and the value returned
    /// by `return $place;` in a `function &f()`.
    MakeRef { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[keys…] -> [argplace]` — SEND_VAR_EX for a *place* argument (Zend's
    /// FETCH_DIM/OBJ_FUNC_ARG). Pop one key per `Index` step (pushed in source
    /// order) and push a [`Zval::ArgPlace`] descriptor deferring the fetch of
    /// an `Index`/`Prop` path rooted at `base`: the dynamic-dispatch binder
    /// W-fetches (aliases) it for a by-reference parameter and R-fetches
    /// (value + warnings) otherwise. `name` is the const-table index of the
    /// root variable's bare name, for the R-branch's "Undefined variable"
    /// warning. Emitted only in argument position of a dynamic call — every
    /// dispatch funnel materializes it before use.
    PushArgPlace { base: FieldBase, steps: Rc<[FieldStep]>, name: u32 },
    /// `[keys…, ref] -> [v]` — REF-4. Pop a reference value, then bind the place
    /// (base + `Index` steps, keys beneath the ref in source order) to its shared
    /// cell: a step-less base is overwritten directly, a stepped leaf is written
    /// like a normal element assignment (so an existing reference element is
    /// written *through*, mirroring the tree-walker's `bind_ref_target`). A
    /// non-reference top-of-stack is wrapped in a fresh cell (the `$y = &f()`
    /// path where `f` is not by-reference). Pushes the aliased value as the
    /// assignment expression's result.
    BindRefTo { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[keys… ref] -> [value]` — like [`Op::BindRefTo`] but for `$t = &m()` where
    /// the by-reference-ness of the callee is only known at run time (a method /
    /// static call): when the source is **not** a [`Zval::Ref`] it raises the
    /// "Only variables should be assigned by reference" notice before copying. The
    /// free-function path emits that notice at compile time instead.
    BindRefToChecked { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[v] -> [v']` — if the top is a [`Zval::Ref`], replace it with a clone of
    /// its referent; otherwise leave it untouched (REF-4b). Emitted after a call
    /// to a `function &f()` used in a *value* context, so the reference it returns
    /// is copied rather than aliased — `$y = &f()` skips this and aliases instead.
    DerefTop,

    // ----- closures (CLO) -----
    /// `[] -> [closure]` — build a [`Zval::Closure`] over `closures[fn_idx]`. Each
    /// `Capture` is read in the *current* frame at this point: `use($x)` snapshots
    /// the value, `use(&$x)` shares the cell as a `Zval::Ref`. `bind_this` captures
    /// the current `$this` (a non-static closure in a method).
    MakeClosure { fn_idx: u32, captures: Rc<[Capture]>, bind_this: bool },
    /// `[] -> [closure]` — a first-class callable `name(...)` (CLO-2): a closure
    /// value wrapping the function *name* (dispatched like a string callable).
    MakeFcc { name: Rc<[u8]> },
    /// `[callee, args…] -> [result]` — a dynamic call `$f(...)` (CLO). Pop `argc`
    /// arguments (source order) and the callee beneath them, then dispatch on the
    /// callee value: an anonymous closure runs `closures[fn_idx]` (binding captures
    /// then params); a named closure / string names a user function or builtin.
    CallValue { argc: u32 },
    /// `[arg0, …, arg{argc-1}] -> [result]` — an unqualified named call inside a
    /// namespace whose target the compiler could not resolve statically (neither a
    /// hoisted user function nor a builtin). Pop `argc` arguments and dispatch by
    /// name at run time trying the namespaced `name` first, then the global
    /// `fallback` — exactly PHP's two-step lookup, so a function defined in another
    /// unit (autoloaded / included) still binds. When neither is defined the
    /// catchable "Call to undefined function `name`()" reports the namespaced name.
    CallNsFallback { name: Rc<[u8]>, fallback: Rc<[u8]>, argc: u32 },
    /// `[callee, argsArray] -> [result]` — a dynamic call with argument unpacking
    /// `$f(...$a)` (CLO). Pop the runtime argument array (its values become the
    /// positional arguments, in order) and the callee beneath it, then dispatch
    /// exactly like [`Op::CallValue`]. The spread variant of `CallValue`, mirroring
    /// [`Op::MethodCallDynamicArgs`] for `$obj->$m(...$a)`.
    CallValueArgs,
    /// `[argsArray] -> [ret]` — like [`Op::CallNsFallback`] but the arguments
    /// are the values of a runtime array (spread on a not-yet-loaded function
    /// inside a namespace: ParameterBag's `trigger_deprecation(...$dep)`).
    CallNsFallbackArgs { name: Rc<[u8]>, fallback: Rc<[u8]> },

    // ----- exceptions (EXC) -----
    /// `[exc] -> ` (diverges) — pop the operand and unwind with
    /// `PhpError::Thrown`. The protected-region table ([`Func::exc_table`])
    /// routes it to a matching `catch`, or it propagates to the caller.
    Throw,
    /// `[exc] -> ` (diverges) — re-raise the exception on top of the stack (no
    /// `catch` clause in the current region matched). Identical to [`Op::Throw`]
    /// but named for legibility at the end of a catch-dispatch sequence.
    Rethrow,
    /// `[exc] -> [exc] | []` — catch dispatch. The in-flight exception is on top:
    /// if its class is `instanceof` any of `types`, pop it (binding it into `var`
    /// if present) and jump to `body`; otherwise leave it and fall through to the
    /// next `CatchMatch` / `Rethrow`. `names` carries any caught class *not* known
    /// at compile time (declared later by `eval`/`include`), resolved by name
    /// against the live class table at run time (step 57, Phase 2).
    CatchMatch { types: Rc<[ClassId]>, names: Rc<[Box<[u8]>]>, var: Option<Slot>, body: Addr },
    /// `[] -> []` — the end of a `finally` block (EXC-2). Resolves the finally's
    /// pending action, in order: (1) a parked exception → re-raise it (resume
    /// unwinding to an outer handler); (2) a parked `return` → push its value and
    /// fall through to the function `Ret` that immediately follows; (3) a parked
    /// `break`/`continue` → jump to its loop target; (4) nothing → jump to `after`
    /// (the code past the `try`, skipping that trailing `Ret`).
    EndFinally { after: Addr },
    /// `[v] -> []` — park `v` as a pending `return` value while a `finally` runs
    /// (EXC-2b): a `return` inside a try-with-finally compiles to this plus a jump
    /// to the finally; [`Op::EndFinally`] performs the actual return afterwards.
    ParkReturn,
    /// `[] -> []` — park a pending `break`/`continue` whose loop target is `addr`
    /// (EXC-2b), while a `finally` runs. The `break`/`continue` inside a
    /// try-with-finally compiles to this plus a jump to the finally; the loop
    /// target is patched in like a normal break/continue site.
    ParkJump(Addr),

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
    /// `[lhs, rhs] -> []` — fused compare+branch (WP-32): evaluate the
    /// comparison `op` exactly as [`Op::Binary`] (same lazy-init, same
    /// string-vs-object `__toString` rule, same overloads/diags), then jump
    /// to `addr` when the boolean result equals `when`. Emitted ONLY when a
    /// condition's AST root is a comparison (compiler `cond_jump`), so the
    /// boolean is never observed as a value — no `Zval::Bool` round-trip.
    CmpJmp { op: BinOp, addr: Addr, when: bool },
    /// `[x] -> []` — [`Op::CmpJmp`] with one LITERAL operand inlined (WP-34,
    /// bigram PushConst→CmpJmp): `consts[cidx]` is the lhs when `const_lhs`
    /// (literal written on the left) else the rhs. Same compare funnel as
    /// `Binary`/`CmpJmp` (binary_value_ab) — a literal push has no effects,
    /// so eliding its dispatch and stack round-trip is unobservable.
    CmpJmpConst { op: BinOp, cidx: ConstIdx, addr: Addr, when: bool, const_lhs: bool },
    /// `[s1..sn] -> [s]` — join `n` already-stringified parts (WP-34): the
    /// compiler emits each part through `Stringify` (or as a Str literal), so
    /// the flattened chain's intermediate `Concat`s were pure — one
    /// allocation replaces n-1 left-associated reallocs. A non-Str part
    /// (unreachable by construction) folds through the pairwise funnel.
    ConcatN(u32),
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
    /// `[array, src] -> [array]` — merge `src`'s elements into the array on the
    /// stack (PAR): integer keys are re-indexed (appended), string keys inserted
    /// (overwriting). `src` is an array; a generator is driven to completion. Used
    /// to build the runtime argument array for a spread call `f(...$src)`.
    ArrayAppendSpread,
    /// `[argsArray] -> [ret]` — call user function `func` with arguments taken
    /// from a runtime array (PAR, `f(...$arr)`): the array's values are bound to
    /// the callee's parameters in order (string keys — named-via-spread — are not
    /// handled and fall back at compile time).
    CallArgs { func: u32 },
    /// `[base, key] -> [v]` — read `base[key]` by value (array element or string
    /// offset); a missing key / non-subscriptable base yields NULL. Read context
    /// is silent in the proof slice (the undefined-key warning rides the
    /// diagnostics-ordering work, like the undefined-variable notice).
    FetchDim,
    /// `[base, key] -> [v]` — like [`Op::FetchDim`] but isset-aware for the `??`
    /// read context: a not-set leaf (missing array key, or out-of-range /
    /// non-integer string offset) yields NULL rather than `""`, so `$x[k] ?? d`
    /// takes the default when the element is unset.
    CoalesceFetchDim,
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
    /// `[] -> []` — declare conditional function `functions[func]` (a `function`
    /// statement reached inside a branch/block): register it in the runtime
    /// function table so it becomes callable by name from here on. Re-declaring an
    /// already-defined function is the PHP "Cannot redeclare function" fatal.
    DeclareFn { func: u32 },
    /// `[] -> []` — declare conditional class/interface/enum `classes[class]` (a
    /// declaration statement reached inside a branch/block): register its name in
    /// the runtime class index so it resolves by name from here on. Re-declaring an
    /// already-defined name is the PHP "Cannot declare class … already in use" fatal.
    DeclareClass { class: ClassId },
    /// `[] -> []` — register [`Module::conditional_traits`]`[idx]` into the VM
    /// seed-trait image (a `trait` declared inside an executed branch), so a
    /// later unit's lowering can `use` it.
    DeclareTrait { idx: u32 },
    /// `[] -> []` — bind the late-bound class-like [`Module::deferred`]`[idx]`
    /// (its supertype was unresolvable when the unit was lowered): the VM
    /// re-lowers the snippet against the current class image, autoloading the
    /// supertype, and registers the declaration — or throws PHP's catchable
    /// `Error: Class|Interface|Trait "X" not found` (Zend late binding).
    DeclareDeferred { idx: u32 },
    /// `[] -> [instance]` — evaluate the late-bound anonymous-class expression
    /// [`Module::deferred`]`[idx]` (constructor arguments re-evaluate in the
    /// caller's bridged scope) and push the instance; or throw the same
    /// faithful `… not found` Error as [`Op::DeclareDeferred`].
    NewAnonDeferred { idx: u32 },
    /// `[arg0, …, arg{argc-1}] -> [result]` — call the by-value builtin named
    /// `name` (resolved in the [`crate::builtin::Registry`] at run time, as the
    /// tree-walker does). Arguments are popped into a `&[Zval]`; the builtin runs
    /// against a `Ctx { out, diags }` borrowed from the VM. Builtins that need the
    /// evaluator (higher-order, class-introspection, `define`/`defined`/`constant`)
    /// are *not* emitted — the compiler rejects them so the VM never sees them.
    CallBuiltin { name: Rc<[u8]>, argc: u32 },
    /// `f(comp…)` into a by-value builtin where at least one component is a spread
    /// `...$src` (step 56b): one value per leading component is on the stack;
    /// `spreads[i]` marks a spread source (expanded via `spread_pairs`) vs a plain
    /// positional value. The VM flattens them to a positional `&[Zval]` — a
    /// positional after a string-keyed (named) unpack raises the catchable Error
    /// "Cannot use positional argument after named argument during unpacking";
    /// a leftover named argument errors (builtins take no named args) — then runs
    /// the builtin exactly like [`Op::CallBuiltin`].
    CallBuiltinSpread { name: Rc<[u8]>, spreads: Rc<[bool]> },
    /// `[arg0, …, arg{argc-1}] -> [result]` — call an *evaluator-only* host builtin
    /// (Session B/C/D) that needs the VM itself: a higher-order builtin that invokes
    /// a user callable (`call_user_func`, `array_map`, …), class introspection, or
    /// the `define` family. Dispatched by [`crate::vm`]'s `dispatch_host_builtin`
    /// (which can run a nested `run_loop` via `call_callable`), not the stateless
    /// registry. `name` is the canonical lowercased builtin name.
    CallHostBuiltin { name: Rc<[u8]>, argc: u32 },
    /// `[rest0, …, rest{argc-1}] -> [result]` — call a by-reference-first *host*
    /// builtin (`usort`, `array_walk`, Session C): like [`Self::CallHostBuiltin`]
    /// but its first argument is the array variable in `slot`, read and written
    /// back in place (the callback may run a nested `run_loop`); `argc` is the
    /// count of the remaining by-value arguments on the stack.
    CallHostBuiltinRef { name: Rc<[u8]>, slot: Slot, argc: u32 },
    /// `[arg0, …, arg{argc-1}] -> [result]` — call a host builtin with a
    /// by-reference **output** parameter at `out_index` (`preg_match`/
    /// `preg_match_all`'s `&$matches`). All `argc` arguments are pushed by value
    /// (the out-param position included, harmlessly); the builtin returns
    /// `(result, out_value)` and the VM writes `out_value` into `out_slot` (a plain
    /// variable, following a reference) before pushing `result`. `out_slot` is
    /// `None` when the out-param argument was omitted (e.g. `preg_match($p,$s)`).
    ///
    /// A builtin with a **second** by-reference out-param (`exec`'s `&$output`
    /// at index 1 *and* `&$result_code` at index 2) sets `out_index2` to that
    /// index (`u32::MAX` when there is none) and `out_slot2` to its target; the
    /// builtin then returns `(result, out_value, Some(out_value2))`.
    CallHostBuiltinOut {
        name: Rc<[u8]>,
        out_slot: Option<Slot>,
        out_index: u32,
        out_slot2: Option<Slot>,
        out_index2: u32,
        argc: u32,
    },
    /// `[arg0, arg1] -> [result]` — call a host builtin with **variadic** by-reference
    /// output parameters (`sscanf`/`fscanf`'s `...&$vars`). The two fixed arguments
    /// (string/stream + format) are pushed by value; `argc` is how many were actually
    /// supplied (the VM raises an ArgumentCountError when < 2). Each variadic out
    /// argument becomes one entry in `out_slots` (`Some(slot)` for a plain variable,
    /// `None` for a non-variable target, which is silently skipped, D-54.1). With no
    /// out slots the builtin returns the parsed array; otherwise it assigns each slot
    /// and returns the successful-conversion count (`fscanf` returns `false` at EOF).
    CallHostBuiltinScanf { name: Rc<[u8]>, argc: u32, out_slots: Rc<[Option<Slot>]> },
    /// `[arg0, …, arg{argc-1}] -> [result]` — `array_multisort`, whose arguments
    /// are **all by-reference** (arrays sorted in place, interleaved with by-value
    /// sort-order/flag ints). Every argument is pushed by value; `arg_slots[i]` is
    /// the writeback slot for a plain-variable array argument (`None` for a literal
    /// or non-variable), and the VM stores each sorted array back into its slot.
    CallArrayMultisort { arg_slots: Rc<[Option<Slot>]>, argc: u32 },
    /// `[] -> [value]` — read a *user-defined* constant `name` (from `define()`),
    /// resolved at run time from the VM's constant table (B3). Engine constants
    /// (`PHP_INT_MAX`, …) are folded at lowering and never reach here; an unknown
    /// name is the catchable `Error` "Undefined constant \"name\"".
    ConstFetch { name: Rc<[u8]>, fallback: Option<Rc<[u8]>> },
    /// `[value] -> []` — declare a user constant `name` (a top-level / namespaced
    /// `const NAME = value`), step 51. Pops the value and registers it in the VM's
    /// constant table; redefining an existing constant warns and keeps the first
    /// value, exactly like `define()`.
    DefineConst { name: Rc<[u8]> },
    /// `[rest0, …, rest{argc-1}] -> [result]` — call a by-reference-first builtin
    /// (`sort`, `array_push`, …): its first argument is the variable in `slot`,
    /// handed to the builtin as `&mut Zval` (write-through), and `argc` is the
    /// count of the remaining by-value arguments on the stack.
    CallBuiltinRef { name: Rc<[u8]>, slot: Slot, argc: u32 },
    /// `[comp0, …, compN] -> [result]` — [`Op::CallBuiltinRef`] whose by-value
    /// *rest* arguments include a spread (`array_push($a, ...$b)`): one stack value
    /// per component, `spreads[i]` marking spread *sources* to flatten at run time
    /// (mirrors [`Op::CallBuiltinSpread`]). The by-ref first argument stays `slot`.
    CallBuiltinRefSpread { name: Rc<[u8]>, slot: Slot, spreads: Rc<[bool]> },
    /// `[ref, rest0, …, rest{argc-1}] -> [result]` — call a by-reference-first
    /// builtin whose first argument is a non-variable place (`array_pop($this->q)`,
    /// `sort($data['list'])`). The by-ref target is a [`Zval::Ref`] cell produced by
    /// [`Op::MakeRef`] and sitting beneath the `argc` by-value arguments; the
    /// builtin mutates the cell in place (write-through to the property / element).
    /// Registry `RefFirst` builtins only — host by-ref builtins (callback-driven)
    /// keep the slot/temp paths.
    CallBuiltinRefCell { name: Rc<[u8]>, argc: u32 },
    /// `[v] -> ` (frame ends) — pop the return value and unwind the current
    /// frame to the caller, which receives it on *its* operand stack. A function
    /// body with no explicit `return` ends with `PushConst(null); Ret`.
    Ret,
    /// `[value]` or `[key, value] -> [sent]` (GEN) — suspend the running
    /// generator frame at a `yield`. Pops the yielded value (and key, if
    /// `has_key`), parks the frame (with its `ip` already past this op), and
    /// returns control to whoever resumed the generator. On the next resume the
    /// `sent` value (the `send()` argument, NULL for `next()`/`foreach`) is pushed
    /// so the `yield` expression evaluates to it.
    Yield { has_key: bool },
    /// `[delegate] -> [returnValue]` (GEN) — `yield from`. Re-yields each element
    /// of an array or sub-generator verbatim (keys unchanged, the outer auto-key
    /// counter untouched). It re-enters itself across resumes — driving one
    /// delegated step per resume, forwarding `send()` into a sub-generator — until
    /// the delegate is exhausted, then leaves the delegate's return value (NULL for
    /// an array, the sub-generator's `getReturn()` otherwise) on the stack.
    YieldFrom,

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
    /// `foreach $src as &$v` (REF-3): snapshot the *keys* of the array in local
    /// `source` and push a by-reference iterator. Unlike [`Op::IterInit`] the
    /// source stays a live variable so each element can be rebound in place.
    IterInitRef(Slot),
    /// By-reference counterpart of [`Op::IterNext`]: promote the source's current
    /// element to a shared cell, alias the `value` slot to it (so body writes land
    /// in the array), bind the `key` slot if present, then fall through — or jump
    /// to `end` when exhausted. The `value` slot lingers as a reference to the
    /// last element after the loop (the documented PHP gotcha, D-R13).
    IterNextRef { value: Slot, key: Option<Slot>, end: Addr },
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
    /// `[obj] -> [clone]` — `clone $obj`: shallow-copy the object (new handle, each
    /// property cloned by value so nested objects are shared and arrays copy on
    /// write), push the copy, then run `__clone` on it if the class defines one
    /// (its return discarded). A non-object receiver is a catchable `Error`.
    Clone,
    /// `[code] -> [value]` — `eval($code)` (step 57): compile the popped string as
    /// a PHP unit at run time, execute it as its own module, and push its `return`
    /// value (or `null`). A compile/parse error yields `false`.
    Eval,
    /// `[path] -> [value]` — `include`/`require`(`_once`) the file named by the
    /// popped path (step 57, Phase 2): load and run it as its own module (reusing
    /// the eval machinery), pushing its top-level `return` value or `int(1)`. A
    /// missing/failed file fatals for `require*`, warns + pushes `false` for
    /// `include*`; a `_once` re-load pushes `true` without re-running.
    Include { mode: IncludeMode },
    /// `[obj] -> [value]` — read property `name` (deref-clone); a missing property
    /// (or a non-object receiver) warns and yields NULL, matching the tree-walker.
    PropGet { name: Rc<[u8]>, ic: PropIc },
    /// `[] -> [value]` — fused `$this->name` read (WP-34, bigram This→PropGet):
    /// same semantics as `This` + `PropGet` by construction (shared fallback),
    /// minus one dispatch and the receiver clone/push/pop round-trip on the
    /// IC-hit path. Emitted only for a non-nullsafe read whose base is `$this`.
    ThisPropGet { name: Rc<[u8]>, ic: PropIc },
    /// `[obj, value] -> [value]` — write `value` into property `name` (created if
    /// absent), in place through the shared object cell. Leaves the assigned value.
    PropSet { name: Rc<[u8]>, ic: PropIc },
    /// `[obj, rhs] -> [result]` — compound `$o->p op= rhs`: read the property
    /// (NULL if absent), apply `op`, store and leave the result.
    /// UNREACHED (WP-30 audit): compound prop assigns lower to
    /// `PropGet`+`PropSet` (compile/assign.rs `assign_op_place`), which carry
    /// the WP-29 ICs; no emit site constructs this variant.
    PropOpSet { name: Rc<[u8]>, op: BinOp },
    /// `[obj] -> [result]` — `++`/`--` on property `name`; `pre` selects new vs old
    /// value, semantics delegated to `php_types`.
    PropIncDec { name: Rc<[u8]>, inc: bool, pre: bool, ic: PropIc },
    /// `[obj] -> [bool]` — `isset($o->p)`: true iff the property exists and is not
    /// null (silent, no warning).
    PropIsset { name: Rc<[u8]>, ic: PropIc },
    /// `[obj] -> [bool]` — the fetch gate of `$o->p ?? d` / `$o->p ??= d`
    /// (zend read_property BP_VAR_IS): like [`Op::PropIsset`], EXCEPT that a
    /// class defining `__get` without `__isset` answers `true` for a missing
    /// property — the following `PropGet(Silent)` then routes to `__get`
    /// (oracle-pinned; `isset()`/`empty()` do NOT take this fallback).
    PropIssetFetchGate { name: Rc<[u8]> },
    /// `[obj, name] -> [bool]` — `isset($o->{expr})` / `isset($o->$k)`: the
    /// dynamic-name twin of [`Op::PropIsset`] (same hook/`__isset` dispatch).
    PropIssetDyn,
    /// `[name] -> [value]` — `$$x` / `${expr}` read: resolve the runtime NAME
    /// against the current frame (named slots, then the dynamic side-table;
    /// superglobals by name). Undefined -> warning + NULL, like `LoadVar`.
    LoadVarDyn,
    /// `[name, rhs] -> [rhs]` — `$$x = rhs`: resolve/create the variable by its
    /// runtime NAME and store (writing through a reference like `StoreSlot`).
    StoreVarDyn,
    /// `[name] -> []` — `global $$x`: dynamic-name form of the `global`
    /// binding. Resolves-or-creates the global cell by the runtime NAME
    /// (created as NULL, like Zend's global-fetch) and aliases the same-named
    /// local to it (named slot, else the dynamic side-table).
    BindGlobalDyn,
    /// `[classRef, name] -> [value]` — `C::{$expr}` (PHP 8.3): resolve the
    /// class (string/object, autoloading), then the constant by its runtime
    /// name through the parent/interface chain; `"class"` yields the class
    /// name, an enum case its singleton. Unknown -> catchable Error.
    ClassConstDynamic,
    /// `[obj] -> [v]` — read property `name` like [`Op::PropGet`] but *silently*:
    /// a missing property yields NULL with no "Undefined property" warning and no
    /// visibility error (the read context of `empty()` / `??`). A `__get` accessor
    /// still runs when present.
    PropGetSilent { name: Rc<[u8]> },
    /// `[obj, name] -> [value]` — dynamic property read `$o->$n` / `$o->{expr}`:
    /// the property name is popped from the stack (coerced to a string) and read
    /// exactly like [`Op::PropGet`] (warns + NULL if missing; hooks/`__get` apply),
    /// step 51.
    PropGetDynamic,
    /// `[obj, name] -> [value]` — dynamic property read like [`Op::PropGetDynamic`]
    /// but *silently* (no "Undefined property" warning), the read context of `??`
    /// on a dynamic name, step 51.
    PropGetDynamicSilent,
    /// `[] -> !` — raise `UnhandledMatchError` for a `match` with no matching arm
    /// and no `default`, formatting the subject in `slot` into the message
    /// ("Unhandled match case <repr>"). Like [`Op::Fatal`] but value-aware.
    MatchError(Slot),
    /// `[obj] -> []` — `unset($o->p)`: remove the property (no-op if absent).
    PropUnset { name: Rc<[u8]> },
    /// `[obj, arg0, …, arg{argc-1}] -> [result]` — instance method call resolved
    /// at *run time* by walking the receiver's class `parent` chain
    /// (case-insensitive). The callee runs in a pushed frame with `$this` bound to
    /// the receiver; a missing method is a fatal (magic `__call` is OOP-3).
    MethodCall { method: Rc<[u8]>, argc: u32, ic: MethodIc },
    /// `[] -> [result]` — fused zero-argument `$this->m()` (WP-36, bigram
    /// This→MethodCall): same semantics as `This` + `MethodCall{argc: 0}` by
    /// construction (the handler feeds the shared `method_call` funnel), minus
    /// one dispatch and the receiver clone/push/pop round-trip. Emitted only
    /// for a non-nullsafe call with no arguments whose base is `$this` — with
    /// no argument ops between the two, the unbound-`$this` error keeps its
    /// position in the side-effect order.
    ThisMethodCall { method: Rc<[u8]>, ic: MethodIc },
    /// `[obj, argsArray] -> [ret]` — like [`Op::MethodCall`] but the arguments are
    /// the values of a runtime array (spread call `$obj->m(...$a)`, Session A):
    /// string keys are dropped, values bound positionally. Resolves the method at
    /// run time exactly as [`Op::MethodCall`] (including `Generator`/`Fiber`).
    MethodCallArgs { method: Rc<[u8]> },
    /// `[obj, arg0, …, arg{argc-1}, name] -> [ret]` — dynamic instance method call
    /// `$obj->$m(args)` / `$obj->{expr}(args)`: the method-name string is popped
    /// from the top of the stack, then dispatched exactly like [`Op::MethodCall`]
    /// on the remaining `[obj, args…]`, step 51.
    MethodCallDynamic { argc: u32 },
    /// `[obj, argsArray, name] -> [ret]` — like [`Op::MethodCallDynamic`] but the
    /// arguments come from a runtime array (spread `$obj->$m(...$a)`), dispatched
    /// like [`Op::MethodCallArgs`] after popping the name, step 51.
    MethodCallDynamicArgs,
    /// `[obj, pos0, …, pos{positional-1}, named0, …, named{k-1}] -> [ret]` — an
    /// instance method call with **named arguments** `$obj->m(p…, n: v, …)`
    /// (Session A). The `positional` leading values fill the callee's first slots;
    /// each of the `k = names.len()` trailing values is bound by `names[i]` to the
    /// matching parameter (resolved at run time from the callee's `param_names`),
    /// with gaps left for the default prologue and a trailing `...$rest` collecting
    /// unmatched names (string keys). Mirrors the evaluator's named-binding errors
    /// (`ArgumentCountError`, unknown / overwriting name).
    MethodCallNamed { method: Rc<[u8]>, positional: u32, names: Rc<[Box<[u8]>]> },
    /// `[pos…, named…] -> [ret]` — call known user function `func` with named
    /// arguments bound at run time against the callee's `param_names` (the runtime
    /// binder, not the compile-time layout). Used when the compile-time layout
    /// can't express the call: a variadic / by-reference parameter, an unknown or
    /// colliding name (both catchable `Error`s in PHP, not compile errors), or a
    /// name routed into `...$rest`. `positional` values are pushed first, then one
    /// value per `names` entry (label order).
    CallNamed { func: u32, positional: u32, names: Rc<[Box<[u8]>]> },
    /// `[comp…, named…] -> [ret]` — call known user function `func` whose argument
    /// list contains a spread (`...$src`). Each leading component pushes one value:
    /// a positional value, or (where `spreads[i]`) a spread *source* expanded at
    /// run time — an array/Traversable whose integer keys become positional args
    /// and string keys become named ones. Trailing explicit named values follow,
    /// one per `names` entry. The binder enforces PHP's ordering (no positional
    /// after a named, the "during unpacking" error) and a non-iterable spread is a
    /// `TypeError`.
    CallSpread { func: u32, spreads: Rc<[bool]>, names: Rc<[Box<[u8]>]> },
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
    /// `[value, classRef] -> [bool]` — `value instanceof $cls` (PAR, dynamic
    /// class): pop the class reference (a name string, leading `\` stripped, or an
    /// object whose class is used) and the operand; an unknown class name yields
    /// `false` (PHP does not error here).
    InstanceOfDynamic,
    /// `[value] -> [bool]` — `value instanceof <built-in interface>` for an
    /// interface that has no `ClassId` (not registered in the prelude). Membership
    /// is decided by the operand's runtime `Zval` type: a `Zval::Generator`
    /// satisfies `Traversable`/`Iterator`/`Generator`; everything else is `false`.
    InstanceOfBuiltin(BuiltinIface),

    // ----- OOP-2a: class context (self/parent/static), constants, static calls -----
    /// `[arg0, …, arg{argc-1}] -> [ret]` — `Class::m()` / `self::m()` /
    /// `parent::m()` / `static::m()`. The starting class comes from `target`; the
    /// method is resolved by walking its `parent` chain. The pushed frame's
    /// defining class is the resolver's, its LSB class is the caller's when
    /// `forwarding` (self/parent/static) else the start class, and `$this` is
    /// propagated per PHP's forwarding rules.
    StaticCall { target: ClassTarget, method: Rc<[u8]>, forwarding: bool, argc: u32, ic: MethodIc },
    /// `[arg0, …] -> [ret]` — PHP 8.4 parent property-hook call
    /// `parent::$prop::get()` / `parent::$prop::set($v)` (`self`/`static`/named
    /// classes resolve the same way). Dispatches the `get`/`set` hook of `prop`
    /// declared on `target` against the executing frame's `$this`; if that class's
    /// property has no user hook the *implicit* hook reads/writes the backing store
    /// directly (validating the argument count). `set`'s single argument is the new
    /// value; a user hook's body return is discarded (the call yields the written
    /// value for the implicit set, otherwise NULL).
    HookCall { target: ClassTarget, prop: Rc<[u8]>, set: bool, argc: u32 },
    /// `[args…] -> [ret]` — a built-in static on the `Closure` class:
    /// `Closure::bind($c, $newThis)` or `Closure::fromCallable($callable)`. The
    /// `Closure` "class" has no compiled entry, so these are dispatched natively
    /// rather than through normal static-method resolution (step 19-6).
    ClosureStatic { method: Rc<[u8]>, argc: u32 },
    /// `[argsArray] -> [ret]` — like [`Op::StaticCall`] but the arguments are the
    /// values of a runtime array (spread call `C::m(...$a)`, Session A): string
    /// keys dropped, values bound positionally.
    StaticCallArgs { target: ClassTarget, method: Rc<[u8]>, forwarding: bool },
    /// `[classRef, arg0, …, arg{argc-1}] -> [ret]` — `$cls::m()` (PAR, dynamic
    /// class): the class reference sits beneath the arguments; it is resolved at
    /// run time (name string with leading `\` stripped, or an object's class) and
    /// the call dispatched non-forwarding (LSB = the resolved class), like a
    /// named static call. An unknown class is a catchable `Error`.
    StaticCallDynamic { method: Rc<[u8]>, argc: u32 },
    /// `[classRef, argsArray] -> [ret]` — like [`Op::StaticCallDynamic`] but the
    /// arguments are the values of a runtime array (spread call `$cls::m(...$a)`,
    /// Session A): the class reference sits beneath the array.
    StaticCallDynamicArgs { method: Rc<[u8]> },
    /// `[classRef, arg0, …, arg{argc-1}, method] -> [ret]` — `$cls::$m()` /
    /// `Class::$m()` (step 51): both the class reference (beneath the arguments) and
    /// the method name (on top) are runtime values. Pop the name, then the args, then
    /// resolve the class and dispatch non-forwarding, like [`Op::StaticCallDynamic`]
    /// with a runtime method. An unknown class/method is a catchable `Error`.
    StaticCallDynamicMethod { argc: u32 },
    /// `[arg0, …, arg{argc-1}, method] -> [ret]` — `self::$m()` / `parent::$m()` /
    /// `static::$m()` / `Class::$m()` where the class is a compile-time `target` but
    /// the method name is a runtime value on top of the stack. Keeps forwarding
    /// semantics (`$this` / LSB) like [`Op::StaticCall`]; only the method is dynamic.
    StaticCallTargetDynamicMethod { target: ClassTarget, forwarding: bool, argc: u32 },
    /// `[classRef, name] -> [value]` — `C::$$x` / `C::${expr}` read: both the
    /// class reference and the property NAME are runtime values. Operands are
    /// peeked (popped only on success) so a scheduled static-init thunk can
    /// re-run the op, like [`Op::StaticPropGetDynamic`].
    StaticPropGetDynName,
    /// `[rhs, classRef, name] -> [rhs]` — `C::$$x = rhs` with a runtime name.
    StaticPropSetDynName,
    /// `[classRef, argsArray, method] -> [ret]` — `$cls::$m(...)` with named or
    /// spread arguments: the args ride a runtime array (string keys = named,
    /// spreads flattened), the class ref and method name are runtime values.
    StaticCallDynamicMethodArgs,
    /// `[argsArray, method] -> [ret]` — `self::$m(...)` / `Class::$m(...)` with
    /// named or spread arguments: compile-time class `target`, runtime method
    /// name, args from a runtime array (string keys = named).
    StaticCallTargetDynamicMethodArgs { target: ClassTarget, forwarding: bool },
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
    ClassConstDyn { name: Rc<[u8]> },
    /// `[classRef] -> [value]` — `$cls::CONST` / `$cls::class` (PAR, dynamic
    /// class): pop the class reference and read its constant at run time. For
    /// `::class`, an object yields its class name and a string is a `TypeError`
    /// (PHP 8). Otherwise the class is resolved (unknown → `Error`) and the
    /// constant looked up (absent → "Undefined constant" `Error`).
    ClassConstFromValue { name: Rc<[u8]> },
    /// `[] -> [case]` — `E::Case` (Session A): push the interned singleton object
    /// for enum `class`'s `case`-th case (materialised on first use, with its
    /// read-only `name` — and, for a backed enum, `value` — property, then cached
    /// so `E::Case === E::Case`).
    EnumCase { class: ClassId, case: u32 },
    /// `[] -> [name]` — `static::class`: push the frame's LSB class name as a
    /// string. (`Class::class` / `self::class` / `parent::class` are folded to a
    /// [`Op::PushConst`] at compile time — except inside closures, whose scope
    /// is rebindable: see [`Op::ClassNameScope`].)
    ClassNameStatic,
    /// `[] -> [name]` — `self::class` (`parent: false`) / `parent::class`
    /// (`parent: true`) inside a CLOSURE body: push the frame's *scope* class
    /// name (or its parent's), following any `Closure::bind` rebinding.
    ClassNameScope { parent: bool },
    /// `[] -> [obj]` — `new static`: allocate an instance of the frame's LSB class
    /// (its property defaults materialised, fresh id). The constructor is run by a
    /// following [`Op::InvokeCtor`] (the actual class — hence the ctor — is only
    /// known at run time).
    AllocStatic,
    /// `[classRef] -> [obj]` — `new $cls` (PAR, dynamic class): pop the class
    /// reference (a name string, leading `\` stripped, or an object whose class is
    /// reused) and allocate an instance of it (defaults materialised, fresh id).
    /// An unknown class name is a catchable `Error`. The constructor is run by the
    /// following `Dup; …; InvokeCtor; Pop`, like `new static`.
    AllocDynamic,
    /// `[obj, arg0, …, arg{argc-1}] -> [ret]` — run `obj`'s `__construct` if its
    /// class (or an ancestor) declares one, with `$this = obj`; otherwise push
    /// NULL. Used for `new static`, where the constructor can't be resolved at
    /// compile time. The instance itself is kept by the surrounding
    /// `AllocStatic; Dup; …; InvokeCtor; Pop` sequence.
    InvokeCtor { argc: u32 },
    /// `[obj, argsArray] -> [ret]` — like [`Op::InvokeCtor`] but the constructor
    /// arguments are the values of a runtime array (spread `new C(...$a)` /
    /// `new $cls(...$a)` / `new static(...$a)`, Session A). The constructor is
    /// resolved at run time from the object's class; NULL is pushed when there is
    /// none, so it serves a ctor-less `new` too.
    InvokeCtorArgs,
    /// `[obj] -> [ret]` — run `obj`'s class [`CompiledClass::prop_init`] thunk (if
    /// any) with `$this = obj`, materialising its non-constant property defaults;
    /// otherwise push NULL. Emitted as `Alloc; Dup; InitProps; Pop` so property
    /// defaults are set before the constructor runs. The class is read from the
    /// object at run time (so it serves `new static` too).
    InitProps,
    /// `[obj] -> [obj]` — if the top-of-stack object is-a `Throwable`, stamp its
    /// `line`/`file`/`trace` at this `new` site (after `InitProps`, before the
    /// constructor), mirroring PHP (EXC-3b/3c). A no-op for non-Throwables. The
    /// object is left on the stack. Emitted right after `InitProps; Pop` so the
    /// stamp is not clobbered by the `$trace = []` property-init thunk.
    StampThrowable,

    // ----- OOP-2b: static properties (visibility-checked, lazily initialised) -----
    /// `[] -> [value]` — read static property `target::$name` (deref-clone). The
    /// declaring class is resolved by walking the parent chain; the cell is
    /// lazily initialised (const default inline, non-const via its init thunk) and
    /// shared for the run. Visibility is enforced against the running frame's class.
    StaticPropGet { target: ClassTarget, name: Rc<[u8]> },
    /// `[value] -> [value]` — write `value` into `target::$name` (through the
    /// shared cell); leaves the assigned value.
    StaticPropSet { target: ClassTarget, name: Rc<[u8]> },
    /// `[] -> [ref]` — push the static property's own storage cell as a
    /// reference value (`$x = &Class::$sp`): writes through the binding then
    /// hit the live cell every other static-prop op reads.
    StaticPropRef { target: ClassTarget, name: Rc<[u8]> },
    /// `[rhs] -> [result]` — compound `target::$name op= rhs`.
    StaticPropOpSet { target: ClassTarget, name: Rc<[u8]>, op: BinOp },
    /// `[] -> [result]` — `++`/`--` on `target::$name`.
    StaticPropIncDec { target: ClassTarget, name: Rc<[u8]>, inc: bool, pre: bool },
    /// `[classRef] -> [value]` — `$cls::$name` read (PAR, dynamic class): the
    /// class reference sits on top; it is resolved at run time, then the static
    /// property is read like [`Op::StaticPropGet`].
    StaticPropGetDynamic { name: Rc<[u8]> },
    /// `[value, classRef] -> [value]` — `$cls::$name = value` (PAR): the class
    /// reference is on top, the value beneath. Resolved at run time, then written.
    StaticPropSetDynamic { name: Rc<[u8]> },
    /// `[rhs, classRef] -> [result]` — `$cls::$name op= rhs` (PAR).
    StaticPropOpSetDynamic { name: Rc<[u8]>, op: BinOp },
    /// `[classRef] -> [result]` — `$cls::$name++` / `--` (PAR, dynamic class): the
    /// class reference is on top; resolved at run time, then the property is
    /// incremented/decremented like [`Op::StaticPropIncDec`] (`pre` selects new vs
    /// old value).
    StaticPropIncDecDynamic { name: Rc<[u8]>, inc: bool, pre: bool },

    // ----- OOP-2c: mixed property/index write paths (`$o->a[$k]`, `$o->x->y`) -----
    /// `[keys…, value] -> [value]` — write `value` through `base` then `steps`
    /// (`Index` steps consume the pushed keys in source order). Objects navigate
    /// in place, arrays auto-vivify + copy-on-write (à la `write_into`).
    FieldAssign { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[keys…, rhs] -> [result]` — compound `place op= rhs`: read the place (NULL
    /// if absent), apply `op`, write back, leave the result.
    FieldAssignOp { base: FieldBase, steps: Rc<[FieldStep]>, op: BinOp },
    /// `[keys…] -> [result]` — `++`/`--` on a mixed place (read, apply, write back).
    FieldIncDec { base: FieldBase, steps: Rc<[FieldStep]>, inc: bool, pre: bool },
    /// `[keys…] -> [bool]` — `isset()` of a mixed place: true iff every level
    /// exists and the leaf is non-null (silent).
    FieldIsset { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[keys…] -> [bool]` — `empty()` of a mixed place: true iff the leaf is
    /// unreachable/null or falsy (silent, like `FieldIsset`).
    FieldEmpty { base: FieldBase, steps: Rc<[FieldStep]> },
    /// `[keys…] -> []` — `unset()` of a mixed place's leaf (silent no-op if absent).
    FieldUnset { base: FieldBase, steps: Rc<[FieldStep]> },

    /// Raise a fatal `Error` carrying `consts[idx]` (a string) as its message.
    /// Used for *stub* function bodies: the always-present PHP prelude (exception
    /// classes, the procedural date API) contains constructs not yet ported, so
    /// those functions compile to a single `Fatal` rather than sinking every
    /// script — the fatal fires only if such a function is actually called.
    Fatal(ConstIdx),

    /// Queue an `E_NOTICE` diagnostic carrying `consts[idx]` (a string) as its
    /// message, then continue. Used for the run-time-shaped but compile-time-known
    /// by-reference notices ("Only variables should be assigned by reference" /
    /// "Only variable references should be returned by reference"), which PHP
    /// raises but does not abort on.
    EmitNotice(ConstIdx),

    /// `[] -> []` — enter an `@` error-suppression region (step 48): mark the
    /// current diagnostics length and raise the suppress depth so `flush_diags`
    /// renders nothing until the matching [`Op::SuppressEnd`].
    /// `[status?] -> !` — `exit` / `die` (step 46). With `has_arg`, pop the status:
    /// a string (or stringable object) is printed and the code is 0; an int / other
    /// scalar becomes the exit code (`% 256`). Raises `PhpError::Exit`, which
    /// propagates *uncatchably* and does NOT run `finally`.
    Exit { has_arg: bool },
    SuppressBegin,
    /// `[] -> []` — leave an `@` region: lower the suppress depth and drop every
    /// diagnostic raised since the matching [`Op::SuppressBegin`]. The suppressed
    /// expression's value is already on the stack and is untouched.
    SuppressEnd,

    /// Release every tracked object the program can no longer reach
    /// (`Rc::strong_count == 1`), running `__destruct` on each, to a fixpoint
    /// (OOP-3d). Emitted by the compiler after each top-level (`main`) statement,
    /// mirroring the tree-walker's global-scope `sweep_destructors`; never inside a
    /// function/method body. A no-op when nothing is unreachable.
    Sweep {
        /// True for global-scope statement boundaries: additionally re-examines
        /// the objects light (in-body) sweeps demoted since the last main sweep,
        /// closing the window for temp deaths the drop sites don't gc_note.
        main: bool,
    },

    /// No-op. Kept so a [`crate::hir::StmtKind::Nop`] / `Label` has a stable
    /// address to compile pass-throughs against without special-casing empty
    /// instruction ranges.
    Nop,
}

/// Where a hot op sources (or sinks) a value under the register-bytecode plan
/// (Leva B, REGISTER_BYTECODE_PLAN.md §4): operand sourcing on the HOT ops,
/// not a second ISA. `Stack` is the legacy form — pop/push a clone through the
/// operand stack; the direct forms read by borrow from a named slot, a
/// register temp (`Func::max_temps` slots past `n_slots`), or the function's
/// constant pool, skipping the stack round-trip. Cold ops stay stack-based
/// forever. Not consumed by any op yet — stage 2 wires it into Binary/CmpJmp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// Legacy: the value transits the frame's operand stack.
    Stack,
    /// A named local / parameter slot (`0..n_slots`).
    Slot(u16),
    /// A register temp slot (`n_slots..n_slots + max_temps`).
    Temp(u16),
    /// An index into the function's constant pool ([`Func::consts`]).
    Const(u16),
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
    /// Source file this function was defined in (carried from its `FnDecl::file`),
    /// so a stack trace / `getFile()` reports the *defining* file even across an
    /// `include`/autoload boundary. Empty for synthetic stubs / the `{main}` body
    /// (which fall back to the module file).
    pub file: Box<[u8]>,
    /// The `/** ... */` doc comment (carried from `FnDecl::doc`) —
    /// `ReflectionFunctionAbstract::getDocComment`.
    pub doc: Option<Box<[u8]>>,
    /// The instruction stream. Jump targets are indices into this vector.
    pub ops: Vec<Op>,
    /// Source line of each op, parallel to `ops` (same length). The VM reads
    /// `lines[ip-1]` to know the line of the instruction that just faulted, so a
    /// synthesized or `new`-constructed Throwable carries the right
    /// `getLine()`/`getFile()` (EXC-3b).
    pub lines: Vec<Line>,
    /// Per-function constant pool, indexed by [`Op::PushConst`].
    pub consts: Vec<Const>,
    /// Function-local `static $v` variables (name → cell id + initial value),
    /// for `getStaticVariables()`. Empty for synthetic thunks. Ids are relocated
    /// at module load alongside the `Op::Static*` cells.
    pub static_vars: Vec<StaticVarDecl>,
    /// Size of the frame's slot array (named locals). The compiler copies this
    /// from the source [`crate::hir::FnDecl::slots`] length (or
    /// [`crate::hir::Program::slots`] for the script body).
    pub n_slots: u32,
    /// Number of *register* temp slots past `n_slots` (Leva B,
    /// REGISTER_BYTECODE_PLAN.md §4): a "register" is an ordinary frame slot
    /// with a static index in `n_slots..n_slots + max_temps`, assigned by the
    /// register-lowering pass ([`crate::compile::reg_lower`]). The frame is
    /// sized `n_slots + max_temps`; recycle/drop machinery covers these slots
    /// like any other. Always 0 until the pass emits register forms (stage 2+)
    /// — distinct from the compiler's pinned temporaries, which are already
    /// folded INTO `n_slots`.
    pub max_temps: u32,
    /// Number of formal parameters, occupying the leading `n_params` slots
    /// (`params[i].slot == i`, as the HIR guarantees).
    pub n_params: u32,
    /// Name of each formal parameter (length `n_params`, parallel to the leading
    /// slots). Empty for the synthetic thunks (prop-init, constants). Used to bind
    /// **named arguments at run time** for a call whose callee isn't known at
    /// compile time — `$obj->m(name: …)`, Session A — where the compile-time
    /// layout ([`crate::compile`]'s `emit_named_layout`) can't be built.
    pub param_names: Box<[Box<[u8]>]>,
    /// Name of *every* named local (length `n_slots`, slot-indexed; temps are
    /// past this range). Carried so an `include` executed inside this function
    /// can bridge its variable scope by name (PHP: the included file shares the
    /// caller's symbol table). Empty for synthetic thunks.
    pub slot_names: Box<[Box<[u8]>]>,
    /// Whether each formal parameter is *required* (no default and non-variadic),
    /// parallel to `param_names`. The run-time named binder validates that every
    /// required parameter received an argument (raising `ArgumentCountError`).
    pub param_required: Box<[bool]>,
    /// Whether each formal parameter is declared by-reference (`&$x`), parallel to
    /// `param_names`. Read at run time when the callee isn't known at compile time
    /// — `array_walk` (Session C) passes the element by reference only when the
    /// callback's first parameter is by-reference, so element mutations propagate.
    pub param_by_ref: Box<[bool]>,
    /// The declared scalar type hint of each formal parameter (length `n_params`,
    /// parallel to `param_names`), or `None` for an unhinted / non-scalar
    /// parameter. The call binder coerces a by-value argument to its hint under
    /// weak typing (raising `TypeError` on failure), or checks it under
    /// `declare(strict_types=1)` (step 14 / 16).
    pub param_hints: Box<[Option<TypeHint>]>,
    /// Precomputed `param_hints.iter().any(is_some)` (WP-31): `enter_callee`
    /// consults it on every call — the per-call scan was pure waste for the
    /// overwhelmingly common hint-free function.
    pub has_hints: bool,
    /// Precomputed "nothing per-call to do beyond moving arguments" (WP-37,
    /// call-site specialization, safe subset): no hints, no by-reference
    /// parameter, no variadic, not a generator. `enter_callee` then just
    /// pushes the frame (no call-line / strict-mode capture — both feed
    /// only hint TypeErrors), and `bind_params` with EXACT arity takes the
    /// straight decay-into-slots loop. Defaults don't matter here: the
    /// fast paths engage only when every declared slot receives a value,
    /// so the callee's default prologue sees no `Undef` — same as today.
    pub simple_call: bool,
    /// The declared scalar return type hint (step 14), enforced on the returned
    /// value at [`Op::Ret`]. `None` for an absent / non-scalar return type, and
    /// left unenforced for a by-reference function (which returns an alias).
    pub ret_hint: Option<TypeHint>,
    /// The slot of a trailing `...$rest` variadic parameter (PAR), or `None`.
    /// When set, the call binder fills slots `0..variadic_slot` from the leading
    /// arguments and collects every remaining argument into an array in this
    /// slot (an empty array when there are no extras).
    pub variadic_slot: Option<Slot>,
    /// `function &f()` — returns by reference (carried through for the by-ref
    /// call/return path, ported later).
    pub by_ref: bool,
    /// Per-parameter default-value thunk (parallel to `param_names`), or `None` for
    /// a required / variadic / non-compilable default. Run in the function's class
    /// context by `ReflectionParameter::getDefaultValue()` (so an array, constant or
    /// `self::C` default evaluates as written). Not used by the call ABI, which has
    /// its own inline default prologue.
    pub param_defaults: Box<[Option<Func>]>,
    /// For each parameter whose default is exactly a constant reference, that
    /// constant's source name (`CONST_TEST_1`, `self::bar`) — else `None`. Backs
    /// `ReflectionParameter::isDefaultValueConstant()` /
    /// `getDefaultValueConstantName()`. Empty for synthetic thunks.
    pub param_default_const: Box<[Option<Box<[u8]>>]>,
    /// Whether each parameter is constructor-promoted (`public int $x`), for
    /// `ReflectionParameter::isPromoted()`. Empty for synthetic thunks.
    pub param_promoted: Box<[bool]>,
    /// `#[Attr]` attributes on each formal parameter (length `n_params`, parallel
    /// to `param_names`) — `ReflectionParameter::getAttributes()`. Each inner vec
    /// is empty for an unattributed parameter (the common case).
    pub param_attributes: Box<[Vec<CompiledAttribute>]>,
    /// Composite (union/intersection) declared type of each formal parameter
    /// (parallel to `param_names`), for `ReflectionParameter::getType()` →
    /// `ReflectionUnionType`/`ReflectionIntersectionType`. `None` for a single
    /// type (reflected through `param_hints`). Reflection-only.
    pub param_reflect_types: Box<[Option<crate::hir::ReflectType>]>,
    /// Composite (union/intersection) declared *return* type, for
    /// `getReturnType()`. `None` for a single type (via `ret_hint`).
    pub ret_reflect_type: Option<crate::hir::ReflectType>,
    /// The body contains a `yield` — calling it produces a `Generator` rather
    /// than running the body. Drives generator setup once `Yield` is wired in.
    pub is_generator: bool,
    /// Source line of the declaration, for diagnostics / stack traces.
    pub line: Line,
    /// Line of the closing `}` of the body, for `getEndLine` / the `@@` export span
    /// (set post-compile from `FnDecl::end_line`). 0 when unknown — the descriptor
    /// then falls back to the op-line span.
    pub end_line: Line,
    /// `#[Attr(args)]` attributes on the `function`/method declaration, retained
    /// for `ReflectionFunction`/`ReflectionMethod::getAttributes()`. Empty for
    /// closures, hooks, attribute thunks, and unattributed functions.
    pub attributes: Vec<CompiledAttribute>,
    /// Protected `try` regions, innermost first (EXC). On an in-flight exception
    /// the VM finds the first region whose `[start, end)` op range contains the
    /// faulting instruction and jumps to its `catch` (the catch-dispatch block).
    pub exc_table: Vec<ExcRegion>,
}

/// One protected `try` region: the half-open op range `[start, end)` it guards
/// and the `target` it routes an in-flight exception to (EXC). For a *catch*
/// region (`is_finally == false`) the target is the catch-dispatch block; for a
/// *finally* region the target is the `finally` body, entered with the exception
/// parked in the frame's pending slot (re-raised at [`Op::EndFinally`]). A
/// `try { } catch { } finally { }` emits both — the catch region (body only)
/// before the finally region (body + catches) so a linear scan tries catches
/// first. Regions are listed innermost-first.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExcRegion {
    pub start: Addr,
    pub end: Addr,
    pub target: Addr,
    pub is_finally: bool,
}

/// The root of a mixed property/index write path ([`Op::FieldAssign`] &c.): a
/// local slot, a `$GLOBALS` slot, or `$this`. Unlike [`DimBase`] this admits
/// `This`, since a mixed path may begin at an object property of `$this`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldBase {
    Local(Slot),
    Global(Slot),
    /// A data superglobal (`$_SERVER` &c.) rooting a reference path
    /// (`$o->p = &$_SERVER`, `&$_SERVER['k']`): resolves into the VM-level
    /// superglobal store by index, like [`DimBase::Superglobal`].
    Superglobal(u8),
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
    /// `->$n` / `->{expr}` — a dynamic property step whose name is taken from the
    /// operand stack at run time (pushed in source order, like an `Index` key),
    /// step 51.
    PropDyn,
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
    /// `self::` inside a CLOSURE body — resolved at run time from the frame's
    /// scope class (`frame.class`), because `Closure::bind`/`bindTo`/`call` can
    /// rebind a closure's scope after compilation. An unscoped closure is PHP's
    /// `Cannot access "self" when no class scope is active`.
    SelfScope,
    /// `parent::` inside a CLOSURE body — the run-time scope class's parent
    /// (`Cannot access "parent" when current class scope has no parent`).
    ParentScope,
}


/// A built-in PHP interface that has no `ClassId` because it is not registered in
/// the prelude. Membership is determined by the operand's runtime `Zval` type
/// rather than by the class table (see [`Op::InstanceOfBuiltin`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinIface {
    /// `Traversable` — the root iteration marker; satisfied by generators.
    Traversable,
    /// `Iterator` — satisfied by generators.
    Iterator,
    /// `Generator` — the concrete generator type.
    Generator,
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
    /// `static` method (for `ReflectionMethod::isStatic`).
    pub is_static: bool,
    /// `final` method (for `ReflectionMethod::isFinal`).
    pub is_final: bool,
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

/// A function-local `static $v = init;` variable, retained for
/// `ReflectionFunctionAbstract::getStaticVariables()`. `id` indexes `Vm::statics`
/// (relocated at module load like the `Op::Static*` cells); `init` is its declared
/// initial value, used when the persistent cell has not been created yet (the
/// function has never run). The current value wins once the cell exists.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticVarDecl {
    pub name: Box<[u8]>,
    pub id: u32,
    pub init: StaticInit,
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
    /// Declared visibility — `ReflectionClassConstant::isPublic` etc. and the
    /// `ReflectionClass::getConstants($filter)` visibility filter.
    pub visibility: Visibility,
    /// `final const` — `ReflectionClassConstant::isFinal`.
    pub is_final: bool,
    /// Attributes declared on the constant (`#[Foo] const X = …`) —
    /// `ReflectionClassConstant::getAttributes`. Empty for the common case.
    pub attributes: Vec<CompiledAttribute>,
}

/// One class attribute retained for reflection (mirrors one
/// [`crate::hir::HirAttribute`]): the attribute class name plus two thunks — one
/// that builds the attribute object (`newInstance()`) and one that yields its
/// argument array (`getArguments()`). Both run in the *attributed* class's context.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledAttribute {
    pub name: Box<[u8]>,
    pub new_thunk: Func,
    pub args_thunk: Func,
}

/// One enum `case` the VM can materialise (Session A): its name and, for a backed
/// enum, the folded backing value (`None` for a pure case — only a `name`
/// property). A backed case whose value did not const-fold is *omitted* from
/// [`CompiledClass::enum_cases`], so `E::Case` for it falls back to the evaluator.
/// The list order matches [`Op::EnumCase`]'s `case` index.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledEnumCase {
    pub name: Box<[u8]>,
    pub value: Option<Const>,
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
    /// The unit file that declared the class and the `class` keyword's line —
    /// `ReflectionClass::getFileName/getStartLine` for a class with no methods
    /// to derive them from (`b"prelude"` marks an engine class → false/internal).
    pub file: Box<[u8]>,
    pub line: u32,
    /// The line of the class body's closing `}` (`ReflectionClass::getEndLine` and
    /// the `@@ start-end` span of the export `__toString`). 0 when unknown.
    pub end_line: u32,
    /// The `/** ... */` doc comment (carried from `ClassDecl::doc`) —
    /// `ReflectionClass::getDocComment`.
    pub doc: Option<Box<[u8]>>,
    /// The name as a shared [`PhpStr`], stamped into each instance's
    /// [`php_types::Object::class_name`] without re-allocating.
    pub class_name: Rc<PhpStr>,
    /// Superclass, resolved to its [`ClassId`] at lowering; `None` for a root.
    pub parent: Option<ClassId>,
    /// Implemented interfaces (resolved ids); `instanceof` walks them transitively.
    pub interfaces: Vec<ClassId>,
    /// Whether `new` on this class is allowed, and the fatal reason if not.
    pub instantiable: Instantiable,
    /// `final class` / enum (for `ReflectionClass::isFinal`).
    pub is_final: bool,
    /// `abstract class` / interface (for `ReflectionClass::isAbstract`).
    pub is_abstract: bool,
    /// Effective instance properties, parent-first and flattened (a redeclared
    /// property keeps its inherited position with the most-derived default), each
    /// with its constant default materialised by [`Const::to_zval`].
    pub prop_defaults: Vec<(Box<[u8]>, Const)>,
    /// Declared-property visibility shape (for `var_dump`), shared by all instances.
    pub info: Rc<ObjectInfo>,
    /// Slot layout for instance property tables (`prop_defaults`' storage keys
    /// in declaration order), shared by every instance's [`Props`].
    pub props_layout: Rc<php_types::PropsLayout>,
    /// Methods declared *on this class* (resolution walks `parent` at run time).
    pub methods: Vec<CompiledMethod>,
    /// Flattened case-insensitive method table (WP-29 B1): `(ci_hash(name),
    /// defining ClassId, index into that class's `methods`)`, sorted by hash,
    /// parent chain baked in leaf-wins. `resolve_method_runtime` binary-
    /// searches it instead of scanning the chain with eq_ignore_ascii_case on
    /// every call. Empty on seed stubs (the walking scan remains as
    /// fallback). ⚠️ carries ClassIds — relocated at unit link like
    /// `prop_info.declaring_class` (the remap keeps the hash order intact).
    pub methods_ci: Box<[(u64, u32, u32)]>,
    /// Names of abstract methods this class carries unimplemented (own,
    /// interface- or trait-required) — `get_class_methods` reports them.
    pub abstract_methods: Vec<Box<[u8]>>,
    /// Full signatures (empty bodies) of the abstract/interface methods
    /// *declared here*: the Reflection surface (`hasMethod` / `getMethod` /
    /// `method_exists` — PHPUnit's mock-generator view of abstract classes)
    /// resolves against these; method dispatch never consults them.
    pub abstract_sigs: Vec<CompiledMethod>,
    /// Instance properties *declared directly on this class* with their
    /// visibility, in declaration order (OOP-2b). Used only for the *ordered*
    /// per-class enumeration in `get_object_vars` / `get_class_vars`; the
    /// visibility / readonly / type *lookups* go through [`CompiledClass::prop_info`].
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
    // (typed-property declared types now live in `prop_info`.)
    pub consts: Vec<CompiledConst>,
    /// Enum cases the VM can materialise as singletons (Session A); empty for a
    /// non-enum. Indexed by [`Op::EnumCase`]'s `case`.
    pub enum_cases: Vec<CompiledEnumCase>,
    /// Attributes declared on this class (`#[Foo(args)]`), in source order — read
    /// by `ReflectionClass::getAttributes()`. Empty for the common case.
    pub attributes: Vec<CompiledAttribute>,
    /// `#[Attr]` attributes declared on each *own* property, keyed by property
    /// name — read by `ReflectionProperty::getAttributes()`. A `ReflectionProperty`
    /// resolves its declaring class, so this need not be flattened. Empty (and the
    /// map absent of a key) for the common unattributed case.
    pub prop_attributes: std::collections::HashMap<Box<[u8]>, Vec<CompiledAttribute>>,
    /// Names of the traits this class uses directly (resolved, original case) —
    /// read by `class_uses()` / `ReflectionClass::getTraitNames()`. Empty when none.
    pub uses_traits: Vec<Box<[u8]>>,
    /// Names of the (flattened) typed instance properties that have no default, so
    /// start *uninitialized*: `new` stores `Zval::Undef` for these instead of NULL.
    /// Empty when the class has no such property.
    pub uninit_props: Vec<Box<[u8]>>,
    /// `false` if the class could not be fully compiled (e.g. a non-constant
    /// property default): [`Op::Alloc`] on it fatals instead of producing a
    /// wrong instance, mirroring the function-stub discipline.
    pub ok: bool,
    /// Unified, parent-resolved per-property metadata (the property-layout
    /// consolidation): name → [`PropInfo`], flattened parent-first at compile time.
    /// The single source for the visibility / readonly / type / hook *lookups* at
    /// run time (`prop_info(class, name)`), replacing the former scattered
    /// `readonly_props` / `prop_types` / `prop_hooks` fields and the
    /// `resolve_prop_decl` / `resolve_readonly_decl` / `resolve_prop_type` parent
    /// walks. (`own_prop_vis` is kept solely for ordered per-class enumeration.)
    pub prop_info: HashMap<Box<[u8]>, PropInfo>,
    /// Whether ANY entry of `prop_info` carries hooks — the VM's property
    /// fast paths skip the per-access hook lookups entirely when false
    /// (the overwhelmingly common case).
    pub has_prop_hooks: bool,
    /// Whether every flattened declared property is `public`. Combined with
    /// `!has_prop_hooks`, a *present, non-`Undef`* slot on a non-lazy instance
    /// can then be read straight off the property table — no visibility walk,
    /// no magic/hook probes (`__get` only ever applies to a miss) — the
    /// `Op::PropGet` fast path (WP-25). Trivially true for a class with no
    /// declared properties (stdClass and friends).
    pub all_props_public: bool,
    /// Whether every flattened declared property is *plain for writing*:
    /// public, symmetric (`set_visibility` none), non-readonly, untyped and
    /// hook-free. An overwrite of a *present* slot on such a class needs no
    /// visibility walk, no magic probe and no coercion — the `Op::PropSet`
    /// fast path (WP-25). Trivially true with no declared properties.
    pub plain_set_props: bool,
    /// Whether ANY flattened declared property carries asymmetric set
    /// visibility (`private(set)`/`protected(set)`, PHP 8.4). `false` for
    /// virtually every class, letting `asym_write_error` bail before its
    /// per-write prop_info lookup (WP-26 quick win: the WP-25 deny cost
    /// showed up as an unconditional hash lookup per declared write).
    pub has_asym_set: bool,
}

/// The compiled `get`/`set` hooks of one property (step 50). Each hook is a
/// method-like [`Func`]: `get` takes no parameter and returns the value; `set`
/// takes one (`$value`) and its return is discarded.
#[derive(Debug, Clone, PartialEq)]
pub struct PropHooks {
    pub get: Option<Func>,
    pub set: Option<Func>,
    /// Whether the property has backing storage (false = virtual: no slot, omitted
    /// from `var_dump`; a `$this->name` inside its own hook reaches the backing).
    pub backed: bool,
}

/// Unified, compile-time-resolved metadata for one declared instance property
/// (the property-layout consolidation). Built once per class in `compile_class`
/// by flattening the parent chain parent-first (most-derived declaration wins),
/// so every shadowing rule that the runtime `resolve_*` walks used to re-derive
/// on each access is baked in here: a more-derived untyped redeclaration clears
/// `type_hint`, a more-derived non-`readonly` redeclaration clears `readonly`.
/// Subsumes `own_prop_vis` + `readonly_props` + `prop_types` + `prop_hooks`; the
/// runtime reads it with a single `prop_info(class, name)` lookup. Storage is
/// unchanged (`Object.props` stays name-keyed and insertion-ordered).
#[derive(Debug, Clone, PartialEq)]
pub struct PropInfo {
    /// Declared visibility (subsumes `own_prop_vis`).
    pub visibility: Visibility,
    /// Asymmetric *write* visibility (`public private(set)`, PHP 8.4); `None`
    /// when the set side matches `visibility`. Read where reference semantics
    /// change (`$r = &$o->prop` from a non-writing scope binds a copy).
    pub set_visibility: Option<Visibility>,
    /// The most-derived class that (re)declares this property — the class named in
    /// access / readonly / type error messages and by `ReflectionProperty::$class`.
    pub declaring_class: ClassId,
    /// `true` iff the most-derived declaration is `readonly` (subsumes
    /// `readonly_props`, with the non-readonly-redeclaration shadow already applied).
    pub readonly: bool,
    /// Declared type of the most-derived declaration (`None` = untyped; an untyped
    /// redeclaration shadows an inherited type). Subsumes `prop_types`.
    pub type_hint: Option<TypeHint>,
    /// Composite (union/intersection) declared type, for `ReflectionProperty::
    /// getType()`. `None` for a single type (reflected through `type_hint`).
    pub reflect_type: Option<crate::hir::ReflectType>,
    /// Property hooks (`None` = not hooked). Subsumes `prop_hooks`; a virtual
    /// (`backed == false`) hooked property has an entry here but no `prop_defaults`
    /// slot.
    pub hooks: Option<PropHooks>,
    /// The key under which the value lives in `Object.props`. Equal to the property
    /// name today; reserved as the future target for private-property name mangling
    /// (so a subclass's `private $x` can shadow a parent's without colliding).
    pub storage_key: Box<[u8]>,
    /// The slot index of `storage_key` in the class's `props_layout` (WP-29):
    /// stamped once at class compile so the runtime reaches the storage slot
    /// without re-hashing the name (`Props::get_slot`). `None` for a VIRTUAL
    /// hooked property (no backing slot). Class-local index — needs no
    /// relocation across unit linking.
    pub slot: Option<u32>,
    /// The `/** … */` doc comment of the most-derived declaration, for
    /// `ReflectionProperty::getDocComment()` and the class export. `None` if absent.
    pub doc: Option<Box<[u8]>>,
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
    /// [`crate::hir::Program::functions`]. `Rc`-shared so an include/eval unit
    /// reuses the main module's compiled prelude functions (indices `0..P`)
    /// instead of recompiling and leaking ~1000 prelude bodies per included
    /// file (WP-20). A shared entry (`Rc::strong_count > 1`) is already
    /// relocated into the global id space; `relocate_module_class_ids` skips
    /// it via `Rc::get_mut`.
    pub functions: Vec<std::rc::Rc<Func>>,
    /// Indices into `functions` that are **conditional** declarations (a `function`
    /// statement inside a branch/block, possibly nested in another body): not
    /// resolvable by name until their [`Op::DeclareFn`] runs (which registers them
    /// in the VM's runtime function table), so name resolution skips these indices.
    pub conditional_fns: std::collections::HashSet<usize>,
    /// Case-insensitive function index (WP-29 B2): `(ci_hash(name), index
    /// into functions)`, sorted by hash. `invoke_named`/`is_name_callable`
    /// binary-search it instead of scanning every function (prelude
    /// included) with a per-entry case-compare. ALL functions are listed —
    /// conditional ones are filtered after the lookup, exactly like the
    /// legacy scan; same-name duplicates keep index order (the tuple sort).
    pub fn_ci: Box<[(u64, u32)]>,
    /// Indices into `classes` that are **conditional** declarations (a class /
    /// interface / enum statement inside a branch/block or a function/method body):
    /// not resolvable by name until their [`Op::DeclareClass`] runs (which registers
    /// the name in the VM's runtime class index), so the eager `class_index` and its
    /// runtime clone skip these indices.
    pub conditional_classes: std::collections::HashSet<usize>,
    /// Traits declared inside a branch (`(key, trait)`), registered into the
    /// VM's seed-trait image when their [`Op::DeclareTrait`] runs.
    pub conditional_traits: Vec<(Vec<u8>, crate::hir::LoweredTrait)>,
    /// Late-bound class-like declarations (unresolvable supertype at lowering
    /// time — Zend late binding), re-lowered by [`Op::DeclareDeferred`] /
    /// [`Op::NewAnonDeferred`] at their execution point.
    pub deferred: Vec<crate::hir::DeferredDecl>,
    /// Anonymous / arrow-function bodies — same index space as
    /// [`crate::hir::Program::closures`].
    pub closures: Vec<Func>,
    /// Compiled class metadata — same index space as
    /// [`crate::hir::Program::classes`] / [`ClassId`]. `Rc`-shared so the
    /// seed-stub entries of an include/eval unit (classes the VM already
    /// links, compiled as inert stubs) are interned per name instead of
    /// re-allocated per module — the stub prefix grows with the accumulated
    /// image, i.e. quadratically across a require storm (WP-20). A shared
    /// entry is skipped by relocation via `Rc::get_mut` (stubs carry no
    /// relocatable ids anyway).
    pub classes: Vec<std::rc::Rc<CompiledClass>>,
    /// Source file name, reproduced verbatim in diagnostics (`… in <file> on
    /// line N`), carried over from [`crate::hir::Program::file`].
    pub file: Box<[u8]>,
    /// Case-insensitive class-name → [`ClassId`] index, cloned from the
    /// compiler's `ProgramCtx`. The VM needs it at runtime to resolve an engine
    /// error's prelude class (`TypeError`, `DivisionByZeroError`, …) so the
    /// matching Throwable can be synthesized and offered to a `catch` (EXC-3a).
    pub class_index: HashMap<Vec<u8>, ClassId>,
    // (fn_ci lookup: see `Module::find_fn_ci` below.)
    /// Number of `static $x` bindings in the whole program (`id` space), used to
    /// size the VM's persistent `statics` storage. Carried from
    /// [`crate::hir::Program::static_count`].
    pub static_count: usize,
    /// `declare(strict_types=1)` is in effect for this file — scalar type hints
    /// are checked exactly (no coercion, `int`→`float` widening aside) at every
    /// call and return. Carried from [`crate::hir::Program::strict`] (step 16).
    pub strict: bool,
    /// `#[Attr]` attributes on top-level `const` declarations, keyed by FQN —
    /// read by `ReflectionConstant::getAttributes()`. Empty for the common case.
    pub const_attributes: std::collections::HashMap<Box<[u8]>, Vec<CompiledAttribute>>,
}

impl Module {
    /// Resolve `name` (ASCII-case-insensitive) to an unconditionally-callable
    /// function index (WP-29 B2): binary search on the sorted `fn_ci` table +
    /// name verify, skipping conditional declarations exactly like the legacy
    /// whole-table scan (which remains as the fallback for a table-less
    /// module). Same-name duplicates resolve to the lowest eligible index —
    /// the tuple sort keeps index order within one hash.
    pub fn find_fn_ci(&self, name: &[u8]) -> Option<usize> {
        if self.fn_ci.is_empty() {
            return self.functions.iter().enumerate().find_map(|(i, f)| {
                (!self.conditional_fns.contains(&i) && f.name.eq_ignore_ascii_case(name))
                    .then_some(i)
            });
        }
        let h = ci_hash(name);
        let mut i = self.fn_ci.partition_point(|e| e.0 < h);
        while let Some(&(eh, idx)) = self.fn_ci.get(i) {
            if eh != h {
                break;
            }
            let idx = idx as usize;
            if !self.conditional_fns.contains(&idx)
                && self.functions.get(idx).is_some_and(|f| f.name.eq_ignore_ascii_case(name))
            {
                return Some(idx);
            }
            i += 1;
        }
        None
    }
}
