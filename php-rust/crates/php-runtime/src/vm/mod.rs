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

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use php_types::{
    convert, open_file_stream, open_php_stream, ops, Closure, ClosureInfo, ClosureRender, Diag,
    Diags, DirHandle, GenKey, GenState, GenStatus, Key, Object, ObjectInfo, PhpArray, PhpError,
    PhpStr, PropVis, Props, ResKind, Resource, Stream, StreamBackend, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::bytecode::{
    Addr, ClassTarget, DimBase, FieldBase, FieldStep, Func, Instantiable, Module, Op, StaticInit,
};
use crate::coerce::coerce_to_hint;
use crate::hir::{BinOp, CastKind, ClassId, Line, Slot, TypeHint, UnOp, Visibility};

mod arrays;
mod calls;
mod coroutines;
mod exceptions;
mod oop;
use arrays::*;
use calls::*;
use oop::*;

/// The result of running a [`Module`] — at parity with [`crate::eval::Outcome`]
/// (E1), so the corpus harness can compare the two engines.
#[derive(Debug)]
pub struct VmOutcome {
    /// Pure program output (`echo` / `print` / builtins), diagnostics *not*
    /// interleaved.
    pub stdout: Vec<u8>,
    /// CLI-faithful stream: `stdout` with diagnostics rendered inline at their
    /// point of occurrence and an uncaught fatal rendered at the tail, exactly as
    /// PHP's CLI SAPI emits them. This is what a `.phpt` `--EXPECT(F)--` section is
    /// compared against (mirrors [`crate::eval::Outcome::rendered`]).
    pub rendered: Vec<u8>,
    /// Non-fatal diagnostics raised during execution, in order.
    pub diags: Diags,
    /// The fatal that stopped execution (if any); also rendered into `rendered`.
    pub fatal: Option<PhpError>,
    /// Top-level `return` value (NULL if the script ran to completion).
    pub return_value: Zval,
    /// Process exit code from `exit`/`die`, `0..=255`; `None` for a clean run.
    pub exit_code: Option<u8>,
}

impl Default for VmOutcome {
    fn default() -> Self {
        VmOutcome {
            stdout: Vec::new(),
            rendered: Vec::new(),
            diags: Diags::new(),
            fatal: None,
            return_value: Zval::Null,
            exit_code: None,
        }
    }
}

/// How a bounded dispatch run ([`Vm::run_loop`]) terminated. The runner is
/// parametrised by a *baseline* frame depth: it runs until the frame at that
/// depth returns ([`RunExit::Returned`]) or a generator at that depth suspends
/// at a `yield` ([`RunExit::Yielded`]). The top-level run uses `baseline = 0`
/// (only `Returned` is possible); a generator resume uses the parked frame's
/// depth, which is what makes suspension a plain return up the Rust stack — the
/// payoff that retires `corosensei` (GEN).
enum RunExit {
    /// The baseline frame ran to its `Ret`; carries the returned value.
    Returned(Zval),
    /// The baseline generator frame hit `Op::Yield` / `Op::YieldFrom`; it has
    /// already been parked in [`Vm::generators`]. The key (`Auto`/`Keyed` from a
    /// plain yield, `Verbatim` from `yield from`) and value are handed back for
    /// the resumer to record — auto-key resolution lives in
    /// [`Vm::resume_generator`], mirroring the tree-walker.
    Yielded { key: GenKey, value: Zval },
    /// A `Fiber::suspend($value)` ran inside the fiber whose frames begin at this
    /// `baseline` (GEN-4). The whole frame segment `frames[baseline..]` has
    /// already been parked in [`Vm::fibers`]; `value` is what `start()`/`resume()`
    /// returns to its caller.
    Suspended { value: Zval },
}

/// Run status of a fiber (GEN-4). `NotStarted` is the absence of a
/// [`Vm::fibers`] entry, so it is not represented here.
#[derive(Clone, Copy, PartialEq)]
enum FiberStatus {
    Running,
    Suspended,
    Terminated,
}

/// A fiber's runtime state (GEN-4), keyed by its object handle id in
/// [`Vm::fibers`]. `parked` holds the suspended frame *segment* (everything the
/// fiber pushed, innermost last) while it is `Suspended` — unlike a generator
/// (one frame), a fiber suspends its whole call stack, since `Fiber::suspend`
/// can be called from any depth.
struct FiberState<'m> {
    status: FiberStatus,
    parked: Vec<Frame<'m>>,
    ret: Zval,
}

/// The currently-running fiber context (GEN-4), pushed while a fiber executes.
/// `baseline` is the frame depth its segment starts at (so `Fiber::suspend`
/// knows how much of the stack to park); `obj` backs `Fiber::getCurrent()`.
struct FiberContext {
    id: u32,
    baseline: usize,
    obj: Zval,
}

/// Why [`run_source_with`] could not produce a [`VmOutcome`] (E2). `Lower` is a
/// failure shared with the evaluator — a parse error or an unsupported *lowering*
/// — so both engines fail alike on it; `Unsupported` is the bytecode compiler
/// ([`crate::compile`]) rejecting a construct the evaluator still runs, which is
/// the VM-vs-eval gap the corpus harness (E4) measures.
#[derive(Debug)]
pub enum VmRunError {
    Lower(crate::LowerError),
    Unsupported(String),
}

/// Defensive ceiling on PHP call-stack depth, mirroring the evaluator's
/// `eval::MAX_CALL_DEPTH`. Pure PHP recursion runs *iteratively* in [`run_loop`]
/// (each call grows the heap-allocated `frames` vector, not the native Rust
/// stack), so the guard's real job is to surface a catchable PHP `Error`
/// ("Maximum call stack depth …") before runaway recursion exhausts memory —
/// instead of letting the host process die. Callback-nested calls (a host
/// builtin re-entering `run_loop`) *do* recurse natively, so as with the
/// evaluator deep-recursion safety presumes a large worker stack.
const MAX_CALL_DEPTH: usize = 25_000;

/// Lower `source`, compile it to bytecode, and run it on the VM (E2) — the VM
/// analogue of [`crate::eval::run_source_with`], so the corpus harness can drive
/// either engine behind an `--engine` flag. A compile-time PHP `Fatal error:`
/// (link-time, e.g. an abstract-method collision) becomes a rendered
/// [`VmOutcome`] just as the evaluator does; a genuine lowering failure or a
/// bytecode-compiler rejection is surfaced as [`VmRunError`].
pub fn run_source_with(
    name: &[u8],
    source: &[u8],
    registry: &Registry,
) -> Result<VmOutcome, VmRunError> {
    let program = match crate::lower_source(name, source) {
        Ok(p) => p,
        // A link-time PHP fatal renders like a runtime one (no "Uncaught" prefix),
        // mirroring `eval::compile_fatal_outcome`.
        Err(crate::LowerError::Fatal { message, line }) => {
            return Ok(compile_fatal_outcome(name, &message, line))
        }
        Err(e) => return Err(VmRunError::Lower(e)),
    };
    let module = crate::compile::compile_program(&program, registry)
        .map_err(|crate::compile::CompileError::Unsupported(what)| VmRunError::Unsupported(what))?;
    Ok(run_module(&module, registry))
}

/// Lower `source`, compile it, and run it on the VM with no builtins registered
/// — the VM analogue of [`crate::eval::run_source`]. Convenience wrapper over
/// [`run_source_with`] with an empty [`Registry`].
pub fn run_source(name: &[u8], source: &[u8]) -> Result<VmOutcome, VmRunError> {
    run_source_with(name, source, &Registry::new())
}

/// Build the [`VmOutcome`] for a compile-time PHP `Fatal error:` (E2; mirrors
/// `eval::compile_fatal_outcome`): rendered like a runtime fatal but without the
/// "Uncaught" prefix or "thrown in" tail.
fn compile_fatal_outcome(file: &[u8], message: &str, line: Line) -> VmOutcome {
    let file_s = String::from_utf8_lossy(file);
    let rendered =
        format!("\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n");
    VmOutcome {
        rendered: rendered.into_bytes(),
        fatal: Some(PhpError::Error(message.to_string())),
        ..VmOutcome::default()
    }
}

/// Compile-and-run is the caller's job ([`crate::compile`]); this takes the
/// already-compiled module and executes its `main`.
pub fn run_module(module: &Module, registry: &Registry) -> VmOutcome {
    let mut vm = Vm {
        module,
        registry,
        stdout: Vec::new(),
        rendered: Vec::new(),
        diags: Diags::new(),
        diags_rendered: 0,
        fatal_line: 1,
        error_level: 30719, // PHP 8.5 E_ALL
        last_error: None,
        exception_handlers: Vec::new(),
        error_handlers: Vec::new(),
        in_error_handler: false,
        final_flush: false,
        frames: Vec::new(),
        next_object_id: 1,
        next_resource_id: 5,
        static_props: HashMap::new(),
        statics: vec![None; module.static_count],
        magic_guard: HashSet::new(),
        created: Vec::new(),
        destructed: HashSet::new(),
        generators: HashMap::new(),
        fibers: HashMap::new(),
        fiber_stack: Vec::new(),
        fiber_class_id: module.class_index.get(&b"fiber"[..]).copied(),
        throwable_id: module.class_index.get(&b"throwable"[..]).copied(),
        enum_cache: HashMap::new(),
        constants: HashMap::new(),
    };
    vm.frames.push(Frame::new(&module.main));
    // `exit`/`die` is a clean termination (the exit code is surfaced, not a fatal);
    // any other `Err` is an uncaught fatal. A `Ok` carries the top-level return.
    let mut exit_code = None;
    // Disable error-handler routing for everything past the main run: the final
    // flush, the uncaught-fatal render, and shutdown destructors must render raw
    // and never call user code (Session 2 `final_flush` guard).
    let run_result = vm.run();
    vm.final_flush = true;
    let (fatal, return_value) = match run_result {
        Ok(v) => (None, v),
        Err(PhpError::Exit(code)) => {
            exit_code = Some(code);
            (None, Zval::Null)
        }
        // An uncaught throwable routed to a `set_exception_handler` is handled
        // there (no fatal banner; PHP exits cleanly); otherwise it is the fatal.
        Err(e) if vm.handle_uncaught_exception(&e) => (None, Zval::Null),
        Err(e) => (Some(e), Zval::Null),
    };
    // Flush any diagnostics still staged, then render the uncaught fatal at the
    // tail of `rendered` (mirrors `eval::run_with`).
    let line = vm.fatal_line;
    // `final_flush` is set, so routing is skipped and this never errs.
    let _ = vm.flush_diags(line);
    if let Some(err) = &fatal {
        vm.render_fatal(err, line);
    }
    // End-of-script destructors (LIFO over the objects still tracked), run after
    // `main` returns — or after a fatal, on a cleared stack (OOP-3d). Their output
    // flows through `emit_str`, so it lands in `rendered` after the fatal block.
    vm.run_shutdown_destructors();
    VmOutcome {
        stdout: vm.stdout,
        rendered: vm.rendered,
        diags: vm.diags,
        fatal,
        return_value,
        exit_code,
    }
}

/// Which magic property accessor a dispatch is for (OOP-3b). Doubles as part of
/// the recursion-guard key so e.g. a `__get` that reads the same property again
/// falls through to direct access instead of re-entering `__get`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MagicKind {
    Get,
    Set,
    Isset,
    Unset,
}

/// A control transfer (`return` / `break` / `continue`) parked while a `finally`
/// block runs, resumed at [`Op::EndFinally`] (EXC-2b). A `break`/`continue`
/// crossing a finally is pre-resolved to the loop's target address at compile
/// time, so resuming is a plain jump.
#[derive(Debug, Clone)]
enum Transfer {
    /// `return <value>`: the value is carried so [`Op::EndFinally`] can push it
    /// and fall through to the function's `Ret`.
    Return(Zval),
    /// `break` / `continue`: jump to the loop's (already-patched) break/continue
    /// target once the finally completes.
    Jump(Addr),
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
    /// The object bound to `$this` while running a method, or `None` for the
    /// script body and free functions. Read by [`Op::This`]; set when a
    /// [`Op::MethodCall`] / [`Op::InvokeMethod`] pushes a method frame.
    this: Option<Zval>,
    /// The class that *defines* the running method — the referent of `self::`, the
    /// base of `parent::`, and (OOP-2b) the access context for visibility. `None`
    /// outside a method.
    class: Option<ClassId>,
    /// The late-static-binding class — the referent of `static::` / `new static`.
    /// For an instance / constructor call it is the receiver's actual class;
    /// forwarding static calls preserve the caller's. `None` outside a method.
    static_class: Option<ClassId>,
    /// When set, this frame's `Ret` value is written into this shared cell instead
    /// of being pushed to the caller. Used by a static-property init thunk (the
    /// access opcode rewound its `ip` to re-read the cell) and by `__set`/`__unset`
    /// (whose return is discarded — a throwaway cell — while the expression's own
    /// result was pre-pushed).
    ret_cell: Option<Rc<RefCell<Zval>>>,
    /// When true, this frame's `Ret` value is cast to bool before being pushed to
    /// the caller — for `__isset`, whose return PHP coerces to bool.
    ret_bool: bool,
    /// When true, this frame's `Ret` value is converted to a string before being
    /// pushed — for a `__toString` call scheduled by [`Op::Stringify`].
    ret_stringify: bool,
    /// A magic-accessor recursion-guard key to remove from [`Vm::magic_guard`]
    /// when this frame returns (OOP-3b).
    guard_release: Option<(u32, MagicKind, Vec<u8>)>,
    /// Active `foreach` iterators, innermost last. Lives in the frame (not the
    /// operand stack) so it survives across the loop body; freed by `IterPop`,
    /// and discarded wholesale when the frame unwinds (a `return` out of a loop).
    iters: Vec<IterState>,
    /// An exception parked while a `finally` block runs (EXC-2): set when an
    /// exception propagates into a finally region, re-raised at [`Op::EndFinally`].
    pending_throw: Option<Zval>,
    /// A `return` / `break` / `continue` parked while a `finally` block runs
    /// (EXC-2b): a control transfer crossing a finally is delayed until the finally
    /// completes, then resumed at [`Op::EndFinally`] — unless the finally itself
    /// transfers control (which executes directly and discards this).
    pending_transfer: Option<Transfer>,
    /// The generator handle id this frame belongs to (GEN), or `None` for an
    /// ordinary frame. Set when a generator frame is created; read by
    /// [`Op::Yield`] to park the frame back into [`Vm::generators`] under its id.
    gen_id: Option<u32>,
    /// An in-progress `yield from` delegation (GEN-3), `None` outside one. Lives
    /// on the frame so it is preserved across the generator's suspensions.
    yield_from: Option<YieldFromState>,
    /// The number of arguments actually passed to this call (PAR), recorded by
    /// the binder. Read by `Op::CheckArity` for the `ArgumentCountError` message.
    argc: u32,
    /// Arguments passed *beyond* the declared (non-variadic) parameters, snapshotted
    /// at bind time for `func_get_args` / `func_get_arg` (Session D1). Empty for a
    /// variadic callee (the surplus lands in the variadic array) or a call with no
    /// extra arguments. Declared-parameter values are read live from the slots, so
    /// `func_get_args` reflects in-body reassignment, matching PHP.
    extra_args: Vec<Zval>,
    /// Set on the `prop_init` thunk frame (`Op::InitProps`): its `$this->prop =`
    /// writes are privileged initialization, so `Op::PropSet` skips the visibility
    /// check and `__set` — a subclass thunk initialises an inherited *private*
    /// default (e.g. `Exception::$trace = []`) without a "cannot access" fatal.
    init_props: bool,
}

impl<'m> Frame<'m> {
    fn new(func: &'m Func) -> Self {
        Frame {
            func,
            ip: 0,
            slots: vec![Zval::Undef; func.n_slots as usize],
            stack: Vec::new(),
            this: None,
            class: None,
            static_class: None,
            ret_cell: None,
            ret_bool: false,
            ret_stringify: false,
            guard_release: None,
            iters: Vec::new(),
            pending_throw: None,
            pending_transfer: None,
            gen_id: None,
            yield_from: None,
            argc: 0,
            extra_args: Vec::new(),
            init_props: false,
        }
    }
}

/// A `foreach` iteration cursor. **By-value** mode snapshots the `(key, value)`
/// pairs at loop entry (so the body mutating the source can't disturb the loop).
/// **By-reference** mode (REF-3) snapshots only the *keys* and rebinds each
/// element of the source variable live each step, so body writes land back in
/// the array.
enum IterState {
    ByVal { entries: Vec<(Zval, Zval)>, pos: usize },
    ByRef { source: Slot, keys: Vec<Key>, pos: usize },
    /// `foreach` over a generator (GEN): no snapshot — each step reads the
    /// generator's current `(key, value)` and resumes it for the next. `primed`
    /// is false until the first `IterNext` (which starts the generator rather than
    /// advancing it), matching the tree-walker's read-then-resume order.
    Gen { rc: Rc<RefCell<GenState>>, primed: bool },
}

/// In-progress `yield from` delegation (GEN-3), parked on the generator frame so
/// it survives suspension. `Op::YieldFrom` re-enters itself across resumes,
/// advancing this cursor one step each time until the delegate is exhausted.
enum YieldFromState {
    /// Delegating to an array's elements (re-yielded verbatim).
    Array { entries: Vec<(Zval, Zval)>, pos: usize },
    /// Delegating to a sub-generator (its `send()`s forwarded, its return value
    /// becoming the `yield from` expression's value).
    Gen { rc: Rc<RefCell<GenState>> },
}

/// The virtual machine: the module under execution plus the explicit call stack.
/// PHP function calls grow `frames` rather than the Rust stack, so deep PHP
/// recursion cannot overflow the host stack, and a frame is suspendable.
struct Vm<'m> {
    module: &'m Module,
    /// Builtin registry, injected by the caller (php-runtime can't build a
    /// populated one — that lives in php-builtins, which depends on php-runtime).
    registry: &'m Registry,
    stdout: Vec<u8>,
    /// CLI-faithful output stream built alongside `stdout` (E1): diagnostics are
    /// flushed into it (stamped with the current line) at each output point, and an
    /// uncaught fatal is rendered at the tail. Mirrors `eval::Evaluator::rendered`.
    rendered: Vec<u8>,
    diags: Diags,
    /// How many entries of `diags` have already been rendered into `rendered`
    /// (the flush cursor), mirroring `eval`'s `diags_rendered`.
    diags_rendered: usize,
    /// The source line where the uncaught fatal occurred, captured before unwinding
    /// pops the faulting frame — used by [`Vm::render_fatal`] for an engine error
    /// (a thrown object carries its own line).
    fatal_line: Line,
    /// The active `error_reporting` bitmask (Session 1). A diagnostic is rendered
    /// by [`Vm::flush_diags`] only when its severity bit is set here. Defaults to
    /// PHP 8.5's `E_ALL` (30719), so every diagnostic surfaces unless a script
    /// narrows it.
    error_level: i64,
    /// The most recent error as `(errno, message, line)` for `error_get_last`,
    /// set at the [`Vm::raise_diagnostic`] chokepoint (Session 2): every diagnostic
    /// — built-in warnings/notices *and* `trigger_error` — is captured (errno from
    /// the diag's E_* number), most-recent-wins. The E_USER_ERROR fatal path records
    /// it directly (it bypasses the chokepoint).
    last_error: Option<(i64, Vec<u8>, Line)>,
    /// The `set_exception_handler` stack (Session 1b); the last entry is active. An
    /// uncaught throwable is routed to it instead of the fatal banner.
    /// `restore_exception_handler` pops; `set_exception_handler` pushes and returns
    /// the previously-active handler.
    exception_handlers: Vec<Zval>,
    /// The `set_error_handler` stack as `(callable, level_mask)` (Session 2); the
    /// last entry is active. A diagnostic whose `errno & mask != 0` is routed to it
    /// by [`Vm::raise_diagnostic`] instead of the default render.
    /// `restore_error_handler` pops; `set_error_handler` pushes and returns the
    /// previously-active handler.
    error_handlers: Vec<(Zval, i64)>,
    /// Re-entrancy guard: while a user error handler is running, a diagnostic it
    /// raises is *not* routed back into the handler (it default-renders instead).
    in_error_handler: bool,
    /// Set true once `run()` returns (in [`run_module`]): the final flush,
    /// `render_fatal`, and shutdown destructors must render raw and never call a
    /// user error handler. Load-bearing guard in [`Vm::raise_diagnostic`].
    final_flush: bool,
    frames: Vec<Frame<'m>>,
    /// Monotonic object-handle counter (`#N` in `var_dump`), starting at 1 like
    /// the tree-walker's `next_object_id`.
    next_object_id: u32,
    /// Monotonic resource-id counter (`fopen`/`tmpfile`/`opendir` mint these),
    /// starting at 5 to match the tree-walker's `next_resource_id` (PHP's first few
    /// ids are taken by the default streams).
    next_resource_id: u32,
    /// Persistent storage for `static` properties, keyed by (declaring class id,
    /// property name); lazily created on first access and shared for the run
    /// (OOP-2b), mirroring the tree-walker's `static_props`.
    static_props: HashMap<(ClassId, Vec<u8>), Rc<RefCell<Zval>>>,
    /// Persistent storage for function `static $x` variables, indexed by the
    /// program-global binding id (`Module::static_count` entries). A cell is
    /// created on the first execution of its declaration and shared across all
    /// later calls — and across recursion — so the variable accumulates. Mirrors
    /// the tree-walker's `Evaluator::statics`.
    statics: Vec<Option<Rc<RefCell<Zval>>>>,
    /// Active magic-accessor guards (object id, kind, property) — a magic method
    /// is not re-entered for the same access while it is running (OOP-3b).
    magic_guard: HashSet<(u32, MagicKind, Vec<u8>)>,
    /// A strong handle to every object created via `new`, in creation order
    /// (OOP-3d). The extra ref lets the destruction sweep detect unreachability
    /// (`Rc::strong_count == 1` ⇒ only this tracking ref remains); entries are
    /// removed as they are destructed or at shutdown.
    created: Vec<Rc<RefCell<Object>>>,
    /// Object handles whose `__destruct` has already run, guarding double calls.
    destructed: HashSet<u32>,
    /// Suspended generator frames, keyed by generator handle id (GEN). A frame
    /// lives here while the generator is `NotStarted` or `Suspended`; it is moved
    /// onto the main `frames` stack while running (resumed), and parked back on
    /// `yield`. This side-table is what lets a generator be a frame *outside* the
    /// call stack without the tree-walker's `corosensei`/`unsafe`.
    generators: HashMap<u32, Frame<'m>>,
    /// Suspended fiber state (GEN-4), keyed by the fiber's object handle id. An
    /// entry exists once the fiber has been started; its `parked` segment holds
    /// the whole suspended call stack while the fiber is `Suspended`.
    fibers: HashMap<u32, FiberState<'m>>,
    /// The stack of currently-running fibers (GEN-4), innermost last. Backs
    /// `Fiber::suspend` (which parks `frames[ctx.baseline..]`) and
    /// `Fiber::getCurrent`.
    fiber_stack: Vec<FiberContext>,
    /// The prelude `Fiber` class id, resolved once at startup (GEN-4), for
    /// recognising fiber receivers and `Fiber::suspend`/`getCurrent`.
    fiber_class_id: Option<ClassId>,
    /// The prelude `Throwable` interface's class id, resolved once at startup
    /// (EXC-3b). Used to recognise a `new`-constructed exception so its
    /// `line`/`file` are stamped at allocation time, as PHP does.
    throwable_id: Option<ClassId>,
    /// Interned enum case singletons, keyed by (enum class id, case index), so
    /// `E::Case === E::Case` (Session A). Materialised lazily on first `E::Case`.
    enum_cache: HashMap<(ClassId, u32), Rc<RefCell<Object>>>,
    /// User-defined constants from `define()` (B3), mirror of
    /// `eval::Evaluator::constants`. Read by [`Op::ConstFetch`] and `constant()`;
    /// engine constants (`PHP_INT_MAX`, …) are folded at lowering and not stored here.
    constants: HashMap<Vec<u8>, Zval>,
}

impl<'m> Vm<'m> {
    /// Allocate a fresh object handle id.
    fn next_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Render every diagnostic raised since the last flush into `rendered`,
    /// stamped with `line` and the module file (E1; mirrors `eval::flush_diags`):
    /// `\n{Severity}: {message} in {file} on line {line}\n`.
    fn flush_diags(&mut self, line: Line) -> Result<(), PhpError> {
        while self.diags_rendered < self.diags.len() {
            // Map each built-in diagnostic to its E_* number, then route through the
            // shared chokepoint. The message is cloned and `diags_rendered` advanced
            // *before* dispatch: a user handler may itself echo (re-entering
            // `flush_diags`), so this diag must already be consumed.
            let (errno, message) = match &self.diags[self.diags_rendered] {
                Diag::Warning(m) => (2, m.clone()),     // E_WARNING
                Diag::Notice(m) => (8, m.clone()),      // E_NOTICE
                Diag::Deprecated(m) => (8192, m.clone()), // E_DEPRECATED
            };
            self.diags_rendered += 1;
            self.raise_diagnostic(errno, &message, line)?;
        }
        Ok(())
    }

    /// The single routing chokepoint for every diagnostic (Session 2). If a
    /// `set_error_handler` callback is active and its level mask matches `errno`,
    /// the handler is invoked with `(errno, message, file, line)`; its return value
    /// decides whether the default render is suppressed. Otherwise — or when the
    /// handler returns falsy — the diagnostic renders the default way, gated by
    /// `error_reporting`. A handler that throws propagates out (the caller `?`s it,
    /// so it surfaces from the statement that raised the diagnostic).
    fn raise_diagnostic(&mut self, errno: i64, message: &str, line: Line) -> Result<(), PhpError> {
        // 1. Offer the diagnostic to the active user handler.
        if let Some(true) = self.route_to_handler(errno, message, line)? {
            return Ok(()); // handled (truthy return) — suppressed entirely.
        }
        // 2. Default handler reached (no matching handler, or it returned `false`).
        //    Record `last_error` for `error_get_last` here — oracle-confirmed: a
        //    diagnostic suppressed by a user handler does NOT update last_error, and
        //    recording is independent of `error_reporting` (which gates only the
        //    visible render below).
        self.last_error = Some((errno, message.as_bytes().to_vec(), line));
        // Default render (Session 1 behaviour), gated on `error_reporting`.
        if self.error_level & errno != 0 {
            let header = format!("\n{}: {} in ", errno_label(errno), message);
            self.rendered.extend_from_slice(header.as_bytes());
            self.rendered.extend_from_slice(&self.module.file);
            let tail = format!(" on line {line}\n");
            self.rendered.extend_from_slice(tail.as_bytes());
        }
        Ok(())
    }

    /// Offer a diagnostic to the active `set_error_handler` callback. Returns
    /// `Ok(None)` when no handler is eligible (none registered, masked out, or
    /// routing is disabled during shutdown / inside another handler — so the
    /// default handler must run); `Ok(Some(true))` when the handler ran and
    /// returned truthy (diagnostic handled — suppress the default); `Ok(Some(false))`
    /// when it returned a literal `false` (default handler must still run). A handler
    /// that throws propagates as `Err`. `error_reporting` does NOT gate routing — the
    /// handler is invoked even under `error_reporting(0)`.
    fn route_to_handler(&mut self, errno: i64, message: &str, line: Line) -> Result<Option<bool>, PhpError> {
        let active = if !self.final_flush && !self.in_error_handler {
            self.error_handlers
                .last()
                .filter(|(_, mask)| errno & *mask != 0)
                .map(|(cb, _)| cb.clone())
        } else {
            None
        };
        let Some(handler) = active else { return Ok(None) };
        let args = vec![
            Zval::Long(errno),
            Zval::Str(PhpStr::new(message.as_bytes().to_vec())),
            Zval::Str(PhpStr::new(self.module.file.to_vec())),
            Zval::Long(line as i64),
        ];
        self.in_error_handler = true;
        let r = self.call_callable(handler, args);
        self.in_error_handler = false;
        let r = r?; // the handler threw — propagate to the faulting statement.
        // Oracle-confirmed: ONLY a literal boolean `false` lets the default handler
        // run; `null` (incl. no return), `0`, `''` — anything else — suppresses.
        Ok(Some(!matches!(r.deref_clone(), Zval::Bool(false))))
    }

    /// Emit `bytes` to both output streams (E1): flush pending diagnostics first
    /// (so they land ahead of the output they precede, stamped with the current
    /// line), then append to `stdout` and `rendered`. Mirrors `eval::emit`.
    fn emit_str(&mut self, top: usize, bytes: &[u8]) -> Result<(), PhpError> {
        let line = self.cur_line(top);
        self.flush_diags(line)?;
        self.stdout.extend_from_slice(bytes);
        self.rendered.extend_from_slice(bytes);
        Ok(())
    }

    /// Run a by-value builtin, mirroring its fresh stdout into `rendered` and
    /// flushing its diagnostics around it (E1; mirrors `eval::dispatch_value_builtin`):
    /// pre-existing diagnostics, then the builtin's own warnings, then its output
    /// — so e.g. a `printf` "Array to string conversion" prints ahead of the
    /// formatted result.
    fn run_value_builtin(
        &mut self,
        f: crate::builtin::BuiltinFn,
        args: &[Zval],
        line: Line,
    ) -> Result<Zval, PhpError> {
        self.flush_diags(line)?;
        let pre = self.stdout.len();
        let res = {
            let mut ctx = Ctx { out: &mut self.stdout, diags: &mut self.diags };
            f(args, &mut ctx)
        };
        let produced = self.stdout[pre..].to_vec();
        self.flush_diags(line)?;
        self.rendered.extend_from_slice(&produced);
        res
    }

    /// Run until the bottom frame returns, yielding the script's result value.
    /// Intercepts thrown exceptions (EXC): when [`Self::run_until_error`] surfaces
    /// an `Err`, [`Self::unwind`] either routes it to a matching `catch` (and we
    /// resume) or reports it unhandled (and we propagate it as the run's fatal).
    fn run(&mut self) -> Result<Zval, PhpError> {
        loop {
            // Top-level run: baseline 0 (only `main` and its callees), so the only
            // possible exit is the script body returning. `floor = 0` keeps `main`
            // on the stack when an exception escapes uncaught (reported as fatal).
            match self.run_loop(0) {
                Ok(RunExit::Returned(v)) => return Ok(v),
                Ok(RunExit::Yielded { .. }) => {
                    unreachable!("a `yield` can only run inside a resumed generator frame")
                }
                Ok(RunExit::Suspended { .. }) => {
                    unreachable!("`Fiber::suspend` outside a fiber is rejected at the call site")
                }
                Err(e) => {
                    // Capture the faulting line before `unwind` pops frames, for an
                    // uncaught engine error's `render_fatal` (E1).
                    self.fatal_line = self.cur_line(self.frames.len() - 1);
                    match self.unwind(e, 0) {
                        None => {} // routed to a `catch`; resume there
                        Some(e) => return Err(e),
                    }
                }
            }
        }
    }

    /// The bounded dispatch loop: runs until the frame at `baseline` returns
    /// ([`RunExit::Returned`]) or a generator at `baseline` suspends at a `yield`
    /// ([`RunExit::Yielded`]), or an op raises a `PhpError` (which the caller
    /// routes through [`Self::unwind`]). Frames above `baseline` (ordinary
    /// callees) return normally to their callers within this same loop.
    fn run_loop(&mut self, baseline: usize) -> Result<RunExit, PhpError> {
        loop {
            // Defensive call-stack depth guard (mirrors `eval::guard_call_depth`):
            // surface a catchable PHP `Error` before runaway recursion exhausts
            // memory (pure PHP recursion is iterative here, growing `frames`) or
            // overflows the native stack (callback-nested `run_loop`s).
            if self.frames.len() > MAX_CALL_DEPTH {
                return Err(PhpError::Error(format!(
                    "Maximum call stack depth of {MAX_CALL_DEPTH} exceeded"
                )));
            }
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
                Op::ConstFetch { name } => {
                    // A user constant (B3): engine constants were folded at lowering.
                    let v = self.constants.get(&name[..]).cloned().ok_or_else(|| {
                        PhpError::Error(format!(
                            "Undefined constant \"{}\"",
                            String::from_utf8_lossy(&name)
                        ))
                    })?;
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
                    // An unset local reads as NULL (silent — used for compiler
                    // temporaries and PHP's warning-free contexts). A reference
                    // slot is followed. Source-level `$x` reads use `LoadVar`.
                    let v = read_slot(&self.frames[top].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::LoadVar { slot, name } => {
                    // A source-level `$x` read: an `Undef` slot raises the PHP 8
                    // "Undefined variable" warning (queued; flushed at the next
                    // emit point with the reading op's line) and yields NULL.
                    if matches!(self.frames[top].slots[slot as usize], Zval::Undef) {
                        if let crate::bytecode::Const::Str(b) =
                            &self.frames[top].func.consts[name as usize]
                        {
                            let msg = format!("Undefined variable ${}", String::from_utf8_lossy(b));
                            self.diags.push(Diag::Warning(msg));
                        }
                    }
                    let v = read_slot(&self.frames[top].slots[slot as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSlot(s) => {
                    let v = self.frames[top].stack.pop().expect("StoreSlot on empty stack");
                    store_slot(&mut self.frames[top].slots[s as usize], v);
                }
                Op::StaticGuard { id, skip } => {
                    // First execution of this `static` declaration falls through to
                    // run the initialiser; every later one skips to the alias.
                    if self.statics[id as usize].is_some() {
                        self.frames[top].ip = skip as usize;
                    }
                }
                Op::StaticStore { id } => {
                    let v = self.frames[top].stack.pop().expect("StaticStore on empty stack");
                    self.statics[id as usize] = Some(Rc::new(RefCell::new(v)));
                }
                Op::StaticAlias { slot, id } => {
                    // Alias the local slot to the persistent cell: reads/writes of
                    // the variable now go through it (the slot holds a `Zval::Ref`,
                    // followed by `read_slot`/`store_slot` like any reference).
                    let cell = Rc::clone(
                        self.statics[id as usize]
                            .as_ref()
                            .expect("StaticAlias reached before its StaticStore"),
                    );
                    self.frames[top].slots[slot as usize] = Zval::Ref(cell);
                }
                Op::LoadGlobal(s) => {
                    // `$GLOBALS['x']` read: the global lives in the script frame.
                    let v = read_slot(&self.frames[0].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreGlobal(s) => {
                    // `$GLOBALS['x'] = …`: write/create the global in the script frame.
                    let v = self.frames[top].stack.pop().expect("StoreGlobal on empty stack");
                    store_slot(&mut self.frames[0].slots[s as usize], v);
                }
                Op::IncDecGlobal { slot, inc, pre } => {
                    let i = slot as usize;
                    if matches!(self.frames[0].slots[i], Zval::Undef) {
                        self.frames[0].slots[i] = Zval::Null;
                    }
                    let old = if pre { None } else { Some(self.frames[0].slots[i].clone()) };
                    {
                        let cell = &mut self.frames[0].slots[i];
                        if inc {
                            ops::increment(cell, &mut self.diags)?;
                        } else {
                            ops::decrement(cell, &mut self.diags)?;
                        }
                    }
                    let pushed = old.unwrap_or_else(|| self.frames[0].slots[i].clone());
                    self.frames[top].stack.push(pushed);
                }
                Op::PushUndef => {
                    self.frames[top].stack.push(Zval::Undef);
                }
                Op::FillDefault { slot, skip } => {
                    // Default-parameter prologue (PAR): skip the default if the
                    // argument was supplied (the slot is not `Undef`).
                    if !matches!(self.frames[top].slots[slot as usize], Zval::Undef) {
                        self.frames[top].ip = skip as usize;
                    }
                }
                Op::CoerceParam { slot, hint } => {
                    // Coerce a just-filled scalar-hinted default (step 14). A valid
                    // constant default always coerces; keep the value otherwise.
                    let v = self.frames[top].slots[slot as usize].clone();
                    if let Ok(c) = coerce_to_hint(v, &hint, &mut self.diags, self.module.strict) {
                        self.frames[top].slots[slot as usize] = c;
                    }
                }
                Op::CheckArity { required, exactly } => {
                    let argc = self.frames[top].argc;
                    if argc < required {
                        // `Class::method` for a method, bare name for a function.
                        let func_name = self.frames[top].func.name.clone();
                        let name = match self.frames[top].class {
                            Some(cid) => format!(
                                "{}::{}",
                                String::from_utf8_lossy(&self.module.classes[cid].name),
                                String::from_utf8_lossy(&func_name)
                            ),
                            None => String::from_utf8_lossy(&func_name).into_owned(),
                        };
                        // The message reports the *call site* line (the caller's
                        // current op), recovered from the EXC-3b line table.
                        let line = if self.frames.len() >= 2 {
                            self.cur_line(self.frames.len() - 2)
                        } else {
                            self.cur_line(top)
                        };
                        let qualifier = if exactly { "exactly" } else { "at least" };
                        let msg = format!(
                            "Too few arguments to function {name}(), {argc} passed in {} on line {line} and {qualifier} {required} expected",
                            String::from_utf8_lossy(&self.module.file)
                        );
                        return Err(PhpError::ArgumentCountError(msg));
                    }
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
                    // `(object)` needs the object table (stdClass alloc); the rest
                    // are pure value conversions.
                    let r = if matches!(k, CastKind::Object) {
                        self.object_cast(a)?
                    } else {
                        apply_cast(k, &a, &mut self.diags)
                    };
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
                    self.emit_str(top, s.as_bytes())?;
                }
                Op::Print => {
                    let v = self.frames[top].stack.pop().expect("Print operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.emit_str(top, s.as_bytes())?;
                    self.frames[top].stack.push(Zval::Long(1));
                }
                Op::Stringify => {
                    let v = self.frames[top].stack.pop().expect("Stringify operand");
                    let target = v.deref_clone();
                    match &target {
                        Zval::Object(o) => {
                            let cid = o.borrow().class_id as usize;
                            match resolve_method_runtime(self.module, cid, b"__toString") {
                                // __toString's (stringified) return flows back via Ret.
                                Some((defc, midx)) => {
                                    let callee = &self.module.classes[defc].methods[midx].func;
                                    let mut frame = Frame::new(callee);
                                    frame.this = Some(target.clone());
                                    frame.class = Some(defc);
                                    frame.static_class = Some(cid);
                                    frame.ret_stringify = true;
                                    self.frames.push(frame);
                                }
                                None => {
                                    let name = String::from_utf8_lossy(
                                        o.borrow().class_name.as_bytes(),
                                    )
                                    .into_owned();
                                    return Err(PhpError::Error(format!(
                                        "Object of class {name} could not be converted to string"
                                    )));
                                }
                            }
                        }
                        other => {
                            let s = convert::to_zstr(other, &mut self.diags);
                            self.frames[top].stack.push(Zval::Str(s));
                        }
                    }
                }
                Op::JumpIfNotNull(addr) => {
                    let keep = !matches!(
                        self.frames[top].stack.last(),
                        Some(Zval::Null | Zval::Undef)
                    );
                    if keep {
                        self.frames[top].ip = addr as usize;
                    } else {
                        self.frames[top].stack.pop();
                    }
                }
                Op::JumpIfNull(addr) => {
                    // Peek; the value is kept either way (nullsafe `?->`).
                    if matches!(self.frames[top].stack.last(), Some(Zval::Null | Zval::Undef)) {
                        self.frames[top].ip = addr as usize;
                    }
                }
                Op::ArrayInit => {
                    self.frames[top].stack.push(Zval::Array(Rc::new(PhpArray::new())));
                }
                Op::ArrayPush => {
                    let value = self.frames[top].stack.pop().expect("ArrayPush value");
                    let mut arr = self.frames[top].stack.pop().expect("ArrayPush array");
                    if let Zval::Array(rc) = &mut arr {
                        let _ = Rc::make_mut(rc).append(value);
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::ArrayInsert => {
                    let value = self.frames[top].stack.pop().expect("ArrayInsert value");
                    let key = self.frames[top].stack.pop().expect("ArrayInsert key");
                    let mut arr = self.frames[top].stack.pop().expect("ArrayInsert array");
                    if let Zval::Array(rc) = &mut arr {
                        let k = coerce_key_silent(&key)
                            .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
                        Rc::make_mut(rc).insert(k, value);
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::ArrayAppendSpread => {
                    let src = self.frames[top].stack.pop().expect("ArrayAppendSpread source");
                    // Collect the (int-key → append, string-key → insert) pairs to
                    // merge. A generator is driven to completion (its keys are
                    // re-yielded verbatim, so honour them like an array's).
                    let pairs: Vec<(Key, Zval)> = match src.deref_clone() {
                        Zval::Array(s) => {
                            s.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect()
                        }
                        Zval::Generator(rc) => {
                            let mut out = Vec::new();
                            self.ensure_started(&rc)?;
                            loop {
                                let (k, v, done) = {
                                    let g = rc.borrow();
                                    (g.cur_key.clone(), g.cur_val.clone(), matches!(g.status, GenStatus::Done))
                                };
                                if done {
                                    break;
                                }
                                let key = coerce_key_silent(&k).unwrap_or(Key::Int(0));
                                out.push((key, v));
                                self.resume_generator(&rc, Zval::Null)?;
                            }
                            out
                        }
                        _ => Vec::new(),
                    };
                    let mut arr = self.frames[top].stack.pop().expect("ArrayAppendSpread array");
                    if let Zval::Array(rc) = &mut arr {
                        let dest = Rc::make_mut(rc);
                        for (k, v) in pairs {
                            if matches!(k, Key::Int(_)) {
                                let _ = dest.append(v);
                            } else {
                                dest.insert(k, v);
                            }
                        }
                    }
                    self.frames[top].stack.push(arr);
                }
                Op::FetchDim => {
                    let key = self.frames[top].stack.pop().expect("FetchDim key");
                    let base = self.frames[top].stack.pop().expect("FetchDim base");
                    let v = read_dim_warn(&base, &key, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::CoalesceFetchDim => {
                    let key = self.frames[top].stack.pop().expect("CoalesceFetchDim key");
                    let base = self.frames[top].stack.pop().expect("CoalesceFetchDim base");
                    self.frames[top].stack.push(read_dim_nullable(&base, &key));
                }
                Op::AssignPath { base, nkeys, append } => {
                    let value = self.frames[top].stack.pop().expect("AssignPath value");
                    let mut keys = self.pop_keys(top, nkeys);
                    let last = if append {
                        Last::Append { value }
                    } else {
                        Last::Set { key: keys.pop().expect("AssignPath key"), value }
                    };
                    let result = self.path_op(base, top, keys, last)?;
                    self.frames[top].stack.push(result);
                }
                Op::AssignOpPath { base, nkeys, op } => {
                    let rhs = self.frames[top].stack.pop().expect("AssignOpPath rhs");
                    let mut keys = self.pop_keys(top, nkeys);
                    let key = keys.pop().expect("AssignOpPath key");
                    let result = self.path_op(base, top, keys, Last::OpSet { key, op, rhs })?;
                    self.frames[top].stack.push(result);
                }
                Op::IncDecPath { base, nkeys, inc, pre } => {
                    let mut keys = self.pop_keys(top, nkeys);
                    let key = keys.pop().expect("IncDecPath key");
                    let result = self.path_op(base, top, keys, Last::IncDec { key, inc, pre })?;
                    self.frames[top].stack.push(result);
                }
                Op::IssetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    let set = matches!(
                        silent_get_path(self.base_cell(base, top), &keys),
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                    );
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::EmptyPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    let empty = match silent_get_path(self.base_cell(base, top), &keys) {
                        Some(v) => !convert::is_true_silent(&v),
                        None => true,
                    };
                    self.frames[top].stack.push(Zval::Bool(empty));
                }
                Op::UnsetPath { base, nkeys } => {
                    let keys = self.pop_keys(top, nkeys);
                    let cell = match base {
                        DimBase::Local(s) => &mut self.frames[top].slots[s as usize],
                        DimBase::Global(s) => &mut self.frames[0].slots[s as usize],
                    };
                    unset_into(cell, &keys);
                }
                Op::BindRef { target, source } => {
                    // REF-1: promote `source` to a shared cell, alias `target` to
                    // the same `Rc`, and push the cell's value (the assignment
                    // expression yields the aliased value). The two slot reads are
                    // sequential, so the borrows never overlap.
                    let cell = make_cell(ref_base_mut(&mut self.frames, top, source));
                    let value = cell.borrow().clone();
                    *ref_base_mut(&mut self.frames, top, target) = Zval::Ref(cell);
                    self.frames[top].stack.push(value);
                }
                Op::PushRef(slot) => {
                    // REF-2: promote the local to a shared cell and push the ref;
                    // the next `Op::Call` binds it into the by-ref callee slot.
                    let cell = make_cell(&mut self.frames[top].slots[slot as usize]);
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::MakeClosure { fn_idx, captures, bind_this } => {
                    let mut bound = Vec::with_capacity(captures.len());
                    for cap in captures.iter() {
                        let val = if cap.by_ref {
                            Zval::Ref(make_cell(&mut self.frames[top].slots[cap.src as usize]))
                        } else {
                            read_slot(&self.frames[top].slots[cap.src as usize])
                        };
                        bound.push((cap.dst, val));
                    }
                    let bound_this = if bind_this { self.frames[top].this.clone() } else { None };
                    let func = &self.module.closures[fn_idx as usize];
                    // Minimal render metadata: the VM omits the parameter dump
                    // (`var_dump` of a closure is a declared cosmetic gap here).
                    let info = Rc::new(ClosureInfo {
                        kind: ClosureRender::Closure {
                            name: PhpStr::new(func.name.to_vec()),
                            file: PhpStr::new(self.module.file.to_vec()),
                            line: func.line,
                        },
                        params: Vec::new(),
                    });
                    let id = self.next_id();
                    let cl = Closure {
                        fn_idx: fn_idx as usize,
                        captures: bound,
                        named: None,
                        bound_this,
                        id,
                        info,
                    };
                    self.frames[top].stack.push(Zval::Closure(Rc::new(cl)));
                }
                Op::MakeFcc { name } => {
                    // CLO-2: a first-class callable wraps a function *name*.
                    let info = Rc::new(ClosureInfo {
                        kind: ClosureRender::Function(PhpStr::new(name.to_vec())),
                        params: Vec::new(),
                    });
                    let id = self.next_id();
                    let cl = Closure {
                        fn_idx: 0,
                        captures: Vec::new(),
                        named: Some(PhpStr::new(name.to_vec())),
                        bound_this: None,
                        id,
                        info,
                    };
                    self.frames[top].stack.push(Zval::Closure(Rc::new(cl)));
                }
                Op::CallValue { argc } => {
                    let n = argc as usize;
                    let mut args = Vec::with_capacity(n);
                    for _ in 0..n {
                        args.push(self.frames[top].stack.pop().expect("CallValue argument"));
                    }
                    args.reverse();
                    let callee = self.frames[top].stack.pop().expect("CallValue callee");
                    self.invoke_value(callee, args)?;
                }
                Op::Throw => {
                    let v = self.frames[top].stack.pop().expect("throw operand");
                    return Err(PhpError::Thrown(v.deref_clone()));
                }
                Op::Rethrow => {
                    let v = self.frames[top].stack.pop().expect("rethrow operand");
                    return Err(PhpError::Thrown(v));
                }
                Op::CatchMatch { types, var, body } => {
                    let exc = self.frames[top].stack.last().expect("in-flight exception").clone();
                    let caught = object_class_id(&exc)
                        .is_some_and(|ec| types.iter().any(|&t| is_instance_of(self.module, ec, t)));
                    if caught {
                        self.frames[top].stack.pop();
                        if let Some(slot) = var {
                            store_slot(&mut self.frames[top].slots[slot as usize], exc);
                        }
                        self.frames[top].ip = body as usize;
                    }
                    // else: fall through to the next CatchMatch / Rethrow.
                }
                Op::EndFinally { after } => {
                    // EXC-2/2b: resolve the finally's pending action. A propagating
                    // exception wins; then a parked return (push the value and fall
                    // through to the trailing `Ret`); then a parked break/continue
                    // (jump to its loop target); otherwise skip past the `try`.
                    if let Some(v) = self.frames[top].pending_throw.take() {
                        return Err(PhpError::Thrown(v));
                    }
                    match self.frames[top].pending_transfer.take() {
                        Some(Transfer::Return(val)) => {
                            self.frames[top].stack.push(val);
                            // fall through to the `Ret` emitted right after this op
                        }
                        Some(Transfer::Jump(addr)) => {
                            self.frames[top].ip = addr as usize;
                        }
                        None => {
                            self.frames[top].ip = after as usize;
                        }
                    }
                }
                Op::ParkReturn => {
                    let v = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    self.frames[top].pending_transfer = Some(Transfer::Return(v));
                }
                Op::ParkJump(addr) => {
                    self.frames[top].pending_transfer = Some(Transfer::Jump(addr));
                }
                Op::DerefTop => {
                    // REF-4b: copy a by-ref return used in value context.
                    if let Some(Zval::Ref(_)) = self.frames[top].stack.last() {
                        let v = self.frames[top].stack.pop().unwrap().deref_clone();
                        self.frames[top].stack.push(v);
                    }
                }
                Op::MakeRef { base, steps } => {
                    // REF-4: navigate to the place's leaf, promote it to a shared
                    // cell, and push a reference to it. Keys (for `Index` steps)
                    // were pushed in source order and sit on top of the stack.
                    let keys = self.pop_field_keys(top, &steps);
                    let cell = {
                        let base_cell = field_base_mut(&mut self.frames, top, base)?;
                        if steps.is_empty() {
                            make_cell(base_cell)
                        } else {
                            field_cell(base_cell, &steps, &mut keys.into_iter())
                        }
                    };
                    self.frames[top].stack.push(Zval::Ref(cell));
                }
                Op::BindRefTo { base, steps } => {
                    // REF-4: pop the reference, bind the target place to its cell,
                    // and push the aliased value (the assignment's result).
                    let top_val = self.frames[top].stack.pop().expect("BindRefTo value");
                    let cell = match top_val {
                        Zval::Ref(rc) => rc,
                        other => Rc::new(RefCell::new(other)),
                    };
                    let value = cell.borrow().clone();
                    let keys = self.pop_field_keys(top, &steps);
                    if steps.is_empty() {
                        // A step-less base is rebound directly (not written
                        // through), matching `eval::bind_ref_target`.
                        let base_cell = field_base_mut(&mut self.frames, top, base)?;
                        *base_cell = Zval::Ref(cell);
                    } else {
                        self.field_set(base, top, &steps, keys, Zval::Ref(cell))?;
                    }
                    self.frames[top].stack.push(value);
                }
                Op::IterInit => {
                    let iterable = self.frames[top].stack.pop().expect("IterInit iterable");
                    // A generator iterates live (no snapshot); an array/other is
                    // snapshotted by value as before (GEN).
                    if let Zval::Generator(gs) = iterable.deref_clone() {
                        self.frames[top]
                            .iters
                            .push(IterState::Gen { rc: gs, primed: false });
                    } else {
                        self.frames[top].iters.push(IterState::ByVal {
                            entries: snapshot_entries(&iterable),
                            pos: 0,
                        });
                    }
                }
                Op::IterNext { value, key, end } => {
                    // A generator step: prime on the first visit, otherwise resume
                    // to the next yield, then bind the current `(key, value)` or
                    // jump to `end` when the generator is done (GEN).
                    let gen = match self.frames[top].iters.last_mut() {
                        Some(IterState::Gen { rc, primed }) => {
                            let rc = Rc::clone(rc);
                            let was_primed = *primed;
                            *primed = true;
                            Some((rc, was_primed))
                        }
                        _ => None,
                    };
                    if let Some((rc, was_primed)) = gen {
                        if was_primed {
                            self.resume_generator(&rc, Zval::Null)?;
                        } else {
                            self.ensure_started(&rc)?;
                        }
                        let (k, v, done) = {
                            let gs = rc.borrow();
                            (gs.cur_key.clone(), gs.cur_val.clone(), matches!(gs.status, GenStatus::Done))
                        };
                        if done {
                            self.frames[top].ip = end as usize;
                        } else {
                            store_slot(&mut self.frames[top].slots[value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], k);
                            }
                        }
                        continue;
                    }
                    // Read the cursor and bump it in a scoped borrow, then touch
                    // the slots — keeping the `iters` and `slots` borrows disjoint.
                    let pair = {
                        let it = self.frames[top].iters.last_mut().expect("IterNext without iterator");
                        let IterState::ByVal { entries, pos } = it else {
                            unreachable!("IterNext on a by-reference iterator");
                        };
                        if *pos >= entries.len() {
                            None
                        } else {
                            let pair = entries[*pos].clone();
                            *pos += 1;
                            Some(pair)
                        }
                    };
                    match pair {
                        None => self.frames[top].ip = end as usize,
                        Some((k, v)) => {
                            // Deref at bind time: a reference element snapshots its
                            // cell and is read live here. `store_slot` writes
                            // *through* a value slot that is itself a reference (the
                            // lingering-ref gotcha), matching the tree-walker.
                            store_slot(&mut self.frames[top].slots[value as usize], v.deref_clone());
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], k);
                            }
                        }
                    }
                }
                Op::IterInitRef(source) => {
                    // REF-3: snapshot the source array's keys once; each step
                    // rebinds the live element by reference.
                    let keys = ref_array_keys(&self.frames[top].slots[source as usize]);
                    self.frames[top].iters.push(IterState::ByRef { source, keys, pos: 0 });
                }
                Op::IterNextRef { value, key, end } => {
                    let next = {
                        let it = self.frames[top].iters.last_mut().expect("IterNextRef without iterator");
                        let IterState::ByRef { source, keys, pos } = it else {
                            unreachable!("IterNextRef on a by-value iterator");
                        };
                        if *pos >= keys.len() {
                            None
                        } else {
                            let k = keys[*pos].clone();
                            let src = *source;
                            *pos += 1;
                            Some((src, k))
                        }
                    };
                    match next {
                        None => self.frames[top].ip = end as usize,
                        Some((src, k)) => {
                            let cell = elem_cell(&mut self.frames[top].slots[src as usize], &k);
                            if let Some(ks) = key {
                                store_slot(&mut self.frames[top].slots[ks as usize], key_to_zval(&k));
                            }
                            // Direct overwrite, *not* `store_slot`: on later
                            // iterations the value slot is itself a `Zval::Ref` to
                            // the previous element, and writing through it would
                            // corrupt that element (D-R13).
                            self.frames[top].slots[value as usize] = Zval::Ref(cell);
                        }
                    }
                }
                Op::IterPop => {
                    self.frames[top].iters.pop();
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
                    bind_params(&mut frame, args);
                    self.enter_callee(frame)?;
                }
                Op::CallArgs { func } => {
                    // Spread call `f(...$arr)` (PAR): the arguments are the values
                    // of a runtime array, bound positionally (variadic/defaults
                    // compose via the binder).
                    let argsval = self.frames[top].stack.pop().expect("CallArgs array");
                    let args = args_from_array_value(argsval);
                    let callee = &self.module.functions[func as usize];
                    let mut frame = Frame::new(callee);
                    bind_params(&mut frame, args);
                    self.enter_callee(frame)?;
                }
                Op::CallNamed { func, positional, names } => {
                    // Named function call bound at run time (unknown/overwrite/
                    // variadic/by-ref): pop named values (source order), then the
                    // positional values, and bind via `build_named_frame`.
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let named: Vec<(Box<[u8]>, Zval)> =
                        names.iter().cloned().zip(named_vals).collect();
                    let pos = self.pop_keys(top, positional);
                    let line = self.cur_line(top);
                    let callee = &self.module.functions[func as usize];
                    let qn = String::from_utf8_lossy(&callee.name).into_owned();
                    let frame =
                        build_named_frame(callee, &self.module.file, line, &qn, pos, named)?;
                    self.enter_callee(frame)?;
                }
                Op::CallSpread { func, spreads, names } => {
                    // Pop explicit named values (source order), then one value per
                    // leading component (a positional value or a spread source).
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let comp_vals = self.pop_keys(top, spreads.len() as u32);
                    let mut positional: Vec<Zval> = Vec::new();
                    let mut named: Vec<(Box<[u8]>, Zval)> = Vec::new();
                    let mut seen_named = false;
                    for (&is_spread, val) in spreads.iter().zip(comp_vals) {
                        if is_spread {
                            // Integer keys are positional, string keys named; a
                            // positional after a named (within the unpacking) is an
                            // error, a non-iterable a TypeError.
                            for (k, v) in self.spread_pairs(val)? {
                                match k {
                                    Key::Int(_) => {
                                        if seen_named {
                                            return Err(PhpError::Error("Cannot use positional argument after named argument during unpacking".to_string()));
                                        }
                                        positional.push(v);
                                    }
                                    Key::Str(s) => {
                                        named.push((s.as_bytes().to_vec().into_boxed_slice(), v));
                                        seen_named = true;
                                    }
                                }
                            }
                        } else {
                            if seen_named {
                                return Err(PhpError::Error("Cannot use positional argument after named argument".to_string()));
                            }
                            positional.push(val);
                        }
                    }
                    // Explicit named args always come last, so no positional can
                    // follow — no need to track `seen_named` past here.
                    for (label, v) in names.iter().cloned().zip(named_vals) {
                        named.push((label, v));
                    }
                    let line = self.cur_line(top);
                    let callee = &self.module.functions[func as usize];
                    let qn = String::from_utf8_lossy(&callee.name).into_owned();
                    let frame = build_named_frame(
                        callee,
                        &self.module.file,
                        line,
                        &qn,
                        positional,
                        named,
                    )?;
                    self.enter_callee(frame)?;
                }
                Op::CallBuiltin { name, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::Value(f)) => *f,
                        // The compiler only emits CallBuiltin for value builtins.
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let args = self.pop_keys(top, argc); // pops argc, source order
                    let line = self.cur_line(top);
                    let result = self.run_value_builtin(f, &args, line)?;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltin { name, argc } => {
                    // An evaluator-only host builtin (Session B): it may invoke a
                    // user callable via `call_callable` (a nested `run_loop`).
                    let args = self.pop_keys(top, argc);
                    let result = self.dispatch_host_builtin(&name, args)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinRef { name, slot, argc } => {
                    // A by-reference-first host builtin (`usort`, Session C): its
                    // array argument lives in `slot` of the caller frame and is
                    // written back in place; the callback may run a nested `run_loop`.
                    let rest = self.pop_keys(top, argc);
                    let result = self.dispatch_host_builtin_ref(&name, slot, rest)?;
                    let top = self.frames.len() - 1;
                    self.frames[top].stack.push(result);
                }
                Op::CallHostBuiltinOut { name, out_slot, out_index, argc } => {
                    // A host builtin with a by-reference output parameter
                    // (`preg_match`/`preg_match_all`'s `&$matches`): dispatch with all
                    // args by value, then write the produced out-value into `out_slot`.
                    let args = self.pop_keys(top, argc);
                    let (result, out_val) =
                        self.dispatch_host_builtin_out(&name, args, out_index as usize)?;
                    let top = self.frames.len() - 1;
                    if let Some(slot) = out_slot {
                        match &mut self.frames[top].slots[slot as usize] {
                            Zval::Ref(rc) => *rc.borrow_mut() = out_val,
                            cell => *cell = out_val,
                        }
                    }
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRef { name, slot, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let rest = self.pop_keys(top, argc);
                    // Mirror `eval`'s ref-builtin rendering (E1): flush, run, append
                    // the builtin's output, then flush its own warnings.
                    let line = self.cur_line(top);
                    self.flush_diags(line)?;
                    let pre = self.stdout.len();
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[slot as usize], &rest, &mut self.stdout, &mut self.diags);
                    let produced = self.stdout[pre..].to_vec();
                    self.rendered.extend_from_slice(&produced);
                    self.flush_diags(line)?;
                    let result = result?;
                    self.frames[top].stack.push(result);
                }
                Op::Ret => {
                    let mut ret = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    // Coerce the returned value to a scalar return hint (weak, or
                    // checked under strict_types) — step 14. A by-reference function
                    // returns an alias, so its return type stays unenforced; the
                    // init-thunk / magic path (`ret_cell`) carries no hint either.
                    let func = self.frames[top].func;
                    if let Some(hint) = func.ret_hint {
                        if !func.by_ref && self.frames[top].ret_cell.is_none() {
                            match coerce_to_hint(ret, &hint, &mut self.diags, self.module.strict) {
                                Ok(c) => ret = c,
                                Err(given) => {
                                    return Err(self.return_type_error(func, &hint, given))
                                }
                            }
                        }
                    }
                    let ret_cell = self.frames[top].ret_cell.take();
                    let ret_bool = self.frames[top].ret_bool;
                    let ret_stringify = self.frames[top].ret_stringify;
                    let guard = self.frames[top].guard_release.take();
                    self.frames.pop();
                    if let Some(key) = guard {
                        self.magic_guard.remove(&key);
                    }
                    if let Some(cell) = ret_cell {
                        // Init thunk / discarded magic return: store into the cell;
                        // the caller already has (or re-reads) its own value.
                        *cell.borrow_mut() = ret;
                    } else {
                        let v = if ret_bool {
                            Zval::Bool(convert::to_bool(&ret, &mut self.diags))
                        } else if ret_stringify {
                            Zval::Str(convert::to_zstr(&ret, &mut self.diags))
                        } else {
                            ret
                        };
                        // The frame that owned this bounded run has returned: hand
                        // the value back to whoever started it (the host, for the
                        // top-level run; `resume_generator`, for a generator body).
                        if self.frames.len() == baseline {
                            return Ok(RunExit::Returned(v));
                        }
                        self.frames
                            .last_mut()
                            .expect("a non-baseline Ret has a caller")
                            .stack
                            .push(v);
                    }
                }
                Op::Yield { has_key } => {
                    // Suspend the running generator frame (GEN). Pop the yielded
                    // value (and key), park the frame back under its handle id, and
                    // hand the key/value to `resume_generator`. `ip` is already
                    // past this op, so the resume continues after the `yield`.
                    let value = self.frames[top].stack.pop().expect("Yield value");
                    let key = if has_key {
                        GenKey::Keyed(self.frames[top].stack.pop().expect("Yield key"))
                    } else {
                        GenKey::Auto
                    };
                    let gid = self.frames[top]
                        .gen_id
                        .expect("Yield outside a generator frame");
                    debug_assert_eq!(top, baseline, "a generator yields at its own baseline");
                    let frame = self.frames.pop().expect("generator frame to park");
                    self.generators.insert(gid, frame);
                    return Ok(RunExit::Yielded { key, value });
                }
                Op::YieldFrom => {
                    // `yield from` (GEN-3): re-enters itself across resumes, driving
                    // one delegated step per visit. First visit sets up the cursor
                    // from the delegate on the stack; a re-visit pops the resume's
                    // sent value (forwarded into a sub-generator, ignored by arrays).
                    if self.frames[top].yield_from.is_none() {
                        let delegate = self.frames[top].stack.pop().expect("YieldFrom delegate");
                        match delegate.deref_clone() {
                            Zval::Array(_) => {
                                let entries = snapshot_entries(&delegate);
                                self.frames[top].yield_from =
                                    Some(YieldFromState::Array { entries, pos: 0 });
                            }
                            Zval::Generator(rc) => {
                                self.frames[top].yield_from =
                                    Some(YieldFromState::Gen { rc: Rc::clone(&rc) });
                                self.ensure_started(&rc)?; // prime to its first yield
                            }
                            other => {
                                return Err(PhpError::Error(format!(
                                    "Can use \"yield from\" only with arrays and Traversables, {} given",
                                    other.error_type_name()
                                )))
                            }
                        }
                    } else {
                        // Re-entry from a resume: the sent value is on the stack.
                        let sent = self.frames[top].stack.pop().expect("YieldFrom sent");
                        let sub = match &self.frames[top].yield_from {
                            Some(YieldFromState::Gen { rc }) => Some(Rc::clone(rc)),
                            _ => None,
                        };
                        if let Some(rc) = sub {
                            self.resume_generator(&rc, sent)?;
                        }
                    }
                    // Take the next delegated `(key, value)`, or finish.
                    let step = match self.frames[top].yield_from.as_mut().unwrap() {
                        YieldFromState::Array { entries, pos } => {
                            if *pos < entries.len() {
                                let pair = entries[*pos].clone();
                                *pos += 1;
                                Some(pair)
                            } else {
                                None
                            }
                        }
                        YieldFromState::Gen { rc } => {
                            let g = rc.borrow();
                            if matches!(g.status, GenStatus::Done) {
                                None
                            } else {
                                Some((g.cur_key.clone(), g.cur_val.clone()))
                            }
                        }
                    };
                    match step {
                        Some((k, v)) => {
                            // Re-enter this op on the next resume; park and re-yield
                            // verbatim (the outer auto-key counter is untouched).
                            self.frames[top].ip -= 1;
                            let gid =
                                self.frames[top].gen_id.expect("YieldFrom outside a generator");
                            let frame = self.frames.pop().expect("generator frame to park");
                            self.generators.insert(gid, frame);
                            return Ok(RunExit::Yielded { key: GenKey::Verbatim(k), value: v });
                        }
                        None => {
                            // Delegation done: leave the delegate's return value (NULL
                            // for an array, the sub-generator's getReturn()) on the
                            // stack as the `yield from` expression's value.
                            let value = match self.frames[top].yield_from.take().unwrap() {
                                YieldFromState::Array { .. } => Zval::Null,
                                YieldFromState::Gen { rc } => rc.borrow().ret.clone(),
                            };
                            self.frames[top].stack.push(value);
                        }
                    }
                }
                Op::Alloc { class } => {
                    let obj = self.alloc_object(class)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocStatic => {
                    let cid = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" outside class context".to_string())
                    })?;
                    let obj = self.alloc_object(cid)?;
                    self.frames[top].stack.push(obj);
                }
                Op::AllocDynamic => {
                    // `new $cls` (PAR): resolve the class reference at run time.
                    let classval = self.frames[top].stack.pop().expect("AllocDynamic class");
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let obj = self.alloc_object(cid)?;
                    self.frames[top].stack.push(obj);
                }
                Op::StampThrowable => {
                    // Stamp line/file/trace on a `new`-constructed Throwable, after
                    // its property-init thunk ran (which would otherwise clobber
                    // `trace`), leaving the object on the stack (EXC-3b/3c).
                    if let Some(obj) = self.frames[top].stack.last().cloned() {
                        self.stamp_throwable_location(&obj);
                    }
                }
                Op::This => match &self.frames[top].this {
                    Some(t) => {
                        let v = t.clone();
                        self.frames[top].stack.push(v);
                    }
                    None => {
                        return Err(PhpError::Error(
                            "Using $this when not in object context".to_string(),
                        ))
                    }
                },
                Op::PropGet { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropGet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    if let Zval::Object(o) = &target {
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            // __get's return *is* the read result (flows via Ret).
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                        check_prop_access(self.module, cur, o.borrow().class_id as usize, &name)?;
                    }
                    let v = read_property(&target, &name, &mut self.diags);
                    self.frames[top].stack.push(v);
                }
                Op::PropGetSilent { name } => {
                    // Like PropGet but with no "Undefined property" warning and no
                    // visibility error (the read context of `empty()` / `??`).
                    let obj = self.frames[top].stack.pop().expect("PropGetSilent object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    if let Zval::Object(o) = &target {
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Get, b"__get")
                        {
                            self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), &name, None, None, false);
                            continue;
                        }
                    }
                    let mut sink = Diags::new();
                    let v = read_property(&target, &name, &mut sink);
                    self.frames[top].stack.push(v);
                }
                Op::PropSet { name } => {
                    let value = self.frames[top].stack.pop().expect("PropSet value");
                    let obj = self.frames[top].stack.pop().expect("PropSet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    // A `prop_init` thunk writes defaults directly: no `__set`, no
                    // visibility check (so a subclass can set an inherited private).
                    if self.frames[top].init_props {
                        write_property(&target, &name, value.clone())?;
                        self.frames[top].stack.push(value);
                        continue;
                    }
                    if let Zval::Object(o) = &target {
                        // An enum case is immutable: every property is readonly and
                        // no dynamic property may be created (step 23).
                        {
                            let ob = o.borrow();
                            if ob.info.is_enum_case {
                                let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                                let prop = String::from_utf8_lossy(&name).into_owned();
                                return Err(PhpError::Error(if ob.props.contains(&name) {
                                    format!("Cannot modify readonly property {cls}::${prop}")
                                } else {
                                    format!("Cannot create dynamic property {cls}::${prop}")
                                }));
                            }
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Set, b"__set")
                        {
                            // The expression yields the assigned value; __set's own
                            // return is discarded into a throwaway cell.
                            self.frames[top].stack.push(value.clone());
                            let discard = Rc::new(RefCell::new(Zval::Null));
                            self.push_magic_prop(defc, midx, oid, MagicKind::Set, target.clone(), &name, Some(value), Some(discard), false);
                            continue;
                        }
                        check_prop_access(self.module, cur, o.borrow().class_id as usize, &name)?;
                    }
                    write_property(&target, &name, value.clone())?;
                    self.frames[top].stack.push(value);
                }
                Op::PropOpSet { name, op } => {
                    let rhs = self.frames[top].stack.pop().expect("PropOpSet rhs");
                    let obj = self.frames[top].stack.pop().expect("PropOpSet object");
                    if let Some(ocid) = object_class_id(&obj) {
                        check_prop_access(self.module, self.frames[top].class, ocid, &name)?;
                    }
                    let old = read_property(&obj, &name, &mut self.diags);
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    write_property(&obj, &name, result.clone())?;
                    self.frames[top].stack.push(result);
                }
                Op::PropIncDec { name, inc, pre } => {
                    let obj = self.frames[top].stack.pop().expect("PropIncDec object");
                    if let Some(ocid) = object_class_id(&obj) {
                        check_prop_access(self.module, self.frames[top].class, ocid, &name)?;
                    }
                    let old = read_property(&obj, &name, &mut self.diags);
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    write_property(&obj, &name, newv.clone())?;
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::PropIsset { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropIsset object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    let set = if let Zval::Object(o) = &target {
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Isset, b"__isset")
                        {
                            // __isset's return (coerced to bool via ret_bool) is the
                            // result.
                            self.push_magic_prop(defc, midx, oid, MagicKind::Isset, target.clone(), &name, None, None, true);
                            continue;
                        }
                        // No magic: an inaccessible declared property reads as not-set.
                        match resolve_prop_decl(self.module, o.borrow().class_id as usize, &name) {
                            Some((vis, decl)) if !visible_from(self.module, cur, vis, decl) => false,
                            _ => prop_isset(&target, &name),
                        }
                    } else {
                        prop_isset(&target, &name)
                    };
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::PropUnset { name } => {
                    let obj = self.frames[top].stack.pop().expect("PropUnset object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    if let Zval::Object(o) = &target {
                        // An enum case property is readonly — it cannot be unset.
                        if o.borrow().info.is_enum_case {
                            let ob = o.borrow();
                            let cls = String::from_utf8_lossy(ob.class_name.as_bytes()).into_owned();
                            let prop = String::from_utf8_lossy(&name).into_owned();
                            return Err(PhpError::Error(format!(
                                "Cannot unset readonly property {cls}::${prop}"
                            )));
                        }
                        if let Some((defc, midx, oid)) =
                            self.magic_applies(o, &name, cur, MagicKind::Unset, b"__unset")
                        {
                            let discard = Rc::new(RefCell::new(Zval::Null));
                            self.push_magic_prop(defc, midx, oid, MagicKind::Unset, target.clone(), &name, None, Some(discard), false);
                            continue;
                        }
                        check_prop_access(self.module, cur, o.borrow().class_id as usize, &name)?;
                    }
                    prop_unset(&target, &name);
                }
                Op::MethodCall { method, argc } => {
                    let args = self.pop_keys(top, argc); // source order
                    let recv = self.frames[top].stack.pop().expect("MethodCall receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args)?;
                }
                Op::MethodCallArgs { method } => {
                    // Spread `$obj->m(...$a)` (Session A): the arguments are the
                    // values of a runtime array (the receiver sits beneath it).
                    let argsval = self.frames[top].stack.pop().expect("MethodCallArgs array");
                    let args = args_from_array_value(argsval);
                    let recv = self.frames[top].stack.pop().expect("MethodCallArgs receiver");
                    let this = recv.deref_clone();
                    self.method_call(top, this, &method, args)?;
                }
                Op::MethodCallNamed { method, positional, names } => {
                    // Named `$obj->m(p…, n: v, …)` (Session A): pop the named values
                    // (source order), then the positional values, then the receiver.
                    let named_vals = self.pop_keys(top, names.len() as u32);
                    let named: Vec<(Box<[u8]>, Zval)> =
                        names.iter().cloned().zip(named_vals).collect();
                    let pos = self.pop_keys(top, positional);
                    let recv = self.frames[top].stack.pop().expect("MethodCallNamed receiver");
                    let this = recv.deref_clone();
                    self.dispatch_instance_call_named(top, this, &method, pos, named)?;
                }
                Op::InvokeMethod { class, method_idx, argc } => {
                    let module = self.module;
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeMethod receiver");
                    let this = recv.deref_clone();
                    let lsb = object_class_id(&this).unwrap_or(class);
                    let callee = &module.classes[class].methods[method_idx as usize].func;
                    let mut frame = Frame::new(callee);
                    bind_params(&mut frame, args);
                    frame.this = Some(this);
                    frame.class = Some(class);
                    frame.static_class = Some(lsb);
                    self.enter_callee(frame)?;
                }
                Op::InstanceOf { class } => {
                    let v = self.frames[top].stack.pop().expect("InstanceOf operand");
                    let result = match v.deref_clone() {
                        Zval::Object(o) => {
                            is_instance_of(self.module, o.borrow().class_id as usize, class)
                        }
                        _ => false,
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfStatic => {
                    let v = self.frames[top].stack.pop().expect("InstanceOfStatic operand");
                    let target = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" outside class context".to_string())
                    })?;
                    let result = match v.deref_clone() {
                        Zval::Object(o) => {
                            is_instance_of(self.module, o.borrow().class_id as usize, target)
                        }
                        _ => false,
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfDynamic => {
                    // `$x instanceof $cls` (PAR): an unknown class name (or a
                    // non-object operand) yields false — PHP does not error here.
                    let classval = self.frames[top].stack.pop().expect("InstanceOfDynamic class");
                    let operand = self.frames[top].stack.pop().expect("InstanceOfDynamic operand");
                    let result = match (object_class_id(&operand), self.class_id_from_value(&classval))
                    {
                        (Some(ocid), Some(tcid)) => is_instance_of(self.module, ocid, tcid),
                        _ => false,
                    };
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::InstanceOfBuiltin(_iface) => {
                    // Generator/Iterator/Traversable have no ClassId; a generator
                    // value satisfies all three, nothing else among the value
                    // types does (objects against these names already test false).
                    let v = self.frames[top].stack.pop().expect("InstanceOfBuiltin operand");
                    let result = matches!(v.deref_clone(), Zval::Generator(_));
                    self.frames[top].stack.push(Zval::Bool(result));
                }
                Op::StaticCall { target, method, forwarding, argc } => {
                    let args = self.pop_keys(top, argc);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" outside class context".to_string())
                        })?,
                    };
                    // `Fiber::suspend` / `Fiber::getCurrent` are native static
                    // dispatch (GEN-4), handled before normal method resolution.
                    if self.fiber_class_id == Some(start) {
                        if method.eq_ignore_ascii_case(b"suspend") {
                            let (id, baseline) = match self.fiber_stack.last() {
                                Some(c) => (c.id, c.baseline),
                                None => {
                                    return Err(PhpError::Error(
                                        "Cannot suspend outside of a fiber".to_string(),
                                    ))
                                }
                            };
                            let value = args.into_iter().next().unwrap_or(Zval::Null);
                            // Park the whole fiber segment; it is restored by resume.
                            let parked = self.frames.split_off(baseline);
                            self.fibers.get_mut(&id).expect("running fiber state").parked = parked;
                            return Ok(RunExit::Suspended { value });
                        }
                        if method.eq_ignore_ascii_case(b"getcurrent") {
                            let cur = self
                                .fiber_stack
                                .last()
                                .map(|c| c.obj.clone())
                                .unwrap_or(Zval::Null);
                            self.frames[top].stack.push(cur);
                            continue;
                        }
                    }
                    self.dispatch_static_call(top, start, &method, forwarding, args)?;
                }
                Op::ClosureStatic { method, argc } => {
                    // `Closure::bind(...)` / `Closure::fromCallable(...)` (step 19-6).
                    let args = self.pop_keys(top, argc); // source order
                    let result = self.closure_static_method(&method, args)?;
                    self.frames[top].stack.push(result);
                }
                Op::StaticCallArgs { target, method, forwarding } => {
                    // Spread `C::m(...$a)` (Session A): args from a runtime array.
                    let argsval = self.frames[top].stack.pop().expect("StaticCallArgs array");
                    let args = args_from_array_value(argsval);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" outside class context".to_string())
                        })?,
                    };
                    self.dispatch_static_call(top, start, &method, forwarding, args)?;
                }
                Op::StaticCallDynamic { method, argc } => {
                    // `$cls::m()` (PAR): args are on top, the class reference beneath.
                    let args = self.pop_keys(top, argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamic class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    // A dynamic class is non-forwarding, like a named class.
                    self.dispatch_static_call(top, start, &method, false, args)?;
                }
                Op::StaticCallDynamicArgs { method } => {
                    // Spread `$cls::m(...$a)` (Session A): args array on top, the
                    // class reference beneath.
                    let argsval = self.frames[top].stack.pop().expect("StaticCallDynamicArgs array");
                    let args = args_from_array_value(argsval);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamicArgs class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    self.dispatch_static_call(top, start, &method, false, args)?;
                }
                Op::ClassConst { class, idx } => {
                    // Run the constant's value thunk as a frame in its declaring
                    // class's context; its `Ret` leaves the value on the caller's
                    // stack.
                    let thunk = &self.module.classes[class].consts[idx as usize].func;
                    let mut frame = Frame::new(thunk);
                    frame.class = Some(class);
                    frame.static_class = Some(class);
                    self.frames.push(frame);
                }
                Op::ClassConstDyn { name } => {
                    let module = self.module;
                    let start = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" outside class context".to_string())
                    })?;
                    let Some((decl, idx)) = find_const_runtime(module, start, &name) else {
                        return Err(PhpError::Error(format!(
                            "Undefined constant {}::{}",
                            String::from_utf8_lossy(&module.classes[start].name),
                            String::from_utf8_lossy(&name)
                        )));
                    };
                    let thunk = &module.classes[decl].consts[idx].func;
                    let mut frame = Frame::new(thunk);
                    frame.class = Some(decl);
                    frame.static_class = Some(decl);
                    self.frames.push(frame);
                }
                Op::ClassConstFromValue { name } => {
                    let classval =
                        self.frames[top].stack.pop().expect("ClassConstFromValue class");
                    if name.eq_ignore_ascii_case(b"class") {
                        // `$x::class`: an object yields its class name; a string (or
                        // any non-object) is a TypeError in PHP 8.
                        match classval.deref_clone() {
                            Zval::Object(o) => {
                                let cls = self.module.classes[o.borrow().class_id as usize].name.to_vec();
                                self.frames[top].stack.push(Zval::Str(PhpStr::new(cls)));
                            }
                            other => {
                                return Err(PhpError::TypeError(format!(
                                    "Cannot use \"::class\" on {}",
                                    other.error_type_name()
                                )))
                            }
                        }
                    } else {
                        let cid = self.resolve_dynamic_class(&classval)?;
                        let module = self.module;
                        let Some((decl, idx)) = find_const_runtime(module, cid, &name) else {
                            return Err(PhpError::Error(format!(
                                "Undefined constant {}::{}",
                                String::from_utf8_lossy(&module.classes[cid].name),
                                String::from_utf8_lossy(&name)
                            )));
                        };
                        let thunk = &module.classes[decl].consts[idx].func;
                        let mut frame = Frame::new(thunk);
                        frame.class = Some(decl);
                        frame.static_class = Some(decl);
                        self.frames.push(frame);
                    }
                }
                Op::ClassNameStatic => {
                    let start = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" outside class context".to_string())
                    })?;
                    let name = self.module.classes[start].name.to_vec();
                    self.frames[top].stack.push(Zval::Str(PhpStr::new(name)));
                }
                Op::EnumCase { class, case } => {
                    let obj = self.enum_case(class, case);
                    self.frames[top].stack.push(Zval::Object(obj));
                }
                Op::InvokeCtor { argc } => {
                    let module = self.module;
                    let args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtor receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtor on a non-object");
                    match resolve_method_runtime(module, cid, b"__construct") {
                        Some((defc, midx)) => {
                            let callee = &module.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee);
                            bind_params(&mut frame, args);
                            frame.this = Some(this);
                            frame.class = Some(defc);
                            frame.static_class = Some(cid);
                            self.frames.push(frame);
                        }
                        // No constructor: leave NULL so the surrounding `Pop` keeps
                        // the operand stack balanced (the instance is kept by `Dup`).
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::InvokeCtorArgs => {
                    // Spread `new C(...$a)` / `new $cls(...)` / `new static(...)`
                    // (Session A): constructor arguments come from a runtime array.
                    let module = self.module;
                    let argsval = self.frames[top].stack.pop().expect("InvokeCtorArgs array");
                    let args = args_from_array_value(argsval);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtorArgs receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtorArgs on a non-object");
                    match resolve_method_runtime(module, cid, b"__construct") {
                        Some((defc, midx)) => {
                            let callee = &module.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee);
                            bind_params(&mut frame, args);
                            frame.this = Some(this);
                            frame.class = Some(defc);
                            frame.static_class = Some(cid);
                            self.frames.push(frame);
                        }
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::InitProps => {
                    let module = self.module;
                    let recv = self.frames[top].stack.pop().expect("InitProps receiver");
                    let cid = object_class_id(&recv).expect("InitProps on a non-object");
                    match &module.classes[cid].prop_init {
                        Some(func) => {
                            let mut frame = Frame::new(func);
                            frame.this = Some(recv.deref_clone());
                            frame.class = Some(cid);
                            frame.static_class = Some(cid);
                            frame.init_props = true; // privileged default writes
                            self.frames.push(frame);
                        }
                        // No non-constant defaults: nothing to do, balance the stack.
                        None => self.frames[top].stack.push(Zval::Null),
                    }
                }
                Op::StaticPropGet { target, name } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue, // init thunk scheduled; re-run after it
                    };
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSet { target, name } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let value = self.frames[top].stack.pop().expect("StaticPropSet value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropOpSet { target, name, op } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let rhs = self.frames[top].stack.pop().expect("StaticPropOpSet rhs");
                    let old = cell.borrow().deref_clone();
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    *cell.borrow_mut() = result.clone();
                    self.frames[top].stack.push(result);
                }
                Op::StaticPropIncDec { target, name, inc, pre } => {
                    let cell = match self.ensure_static(target, &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    let old = cell.borrow().deref_clone();
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::StaticPropGetDynamic { name } => {
                    // The class reference is on top; peek it so a scheduled init
                    // thunk can re-run this op without losing it (PAR).
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // remove the class reference
                    let v = cell.borrow().deref_clone();
                    self.frames[top].stack.push(v);
                }
                Op::StaticPropSetDynamic { name } => {
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let value = self.frames[top].stack.pop().expect("StaticPropSetDynamic value");
                    *cell.borrow_mut() = value.clone();
                    self.frames[top].stack.push(value);
                }
                Op::StaticPropOpSetDynamic { name, op } => {
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let rhs = self.frames[top].stack.pop().expect("StaticPropOpSetDynamic rhs");
                    let old = cell.borrow().deref_clone();
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    *cell.borrow_mut() = result.clone();
                    self.frames[top].stack.push(result);
                }
                Op::StaticPropIncDecDynamic { name, inc, pre } => {
                    // `$cls::$p++` (PAR): peek the class ref so a scheduled init
                    // thunk can re-run this op; pop it once the cell is ready.
                    let classval = self.frames[top].stack.last().expect("class ref").clone();
                    let cid = self.resolve_dynamic_class(&classval)?;
                    let cell = match self.ensure_static(ClassTarget::Class(cid), &name, top, ip)? {
                        Some(c) => c,
                        None => continue,
                    };
                    self.frames[top].stack.pop(); // class
                    let old = cell.borrow().deref_clone();
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    *cell.borrow_mut() = newv.clone();
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::FieldAssign { base, steps } => {
                    let value = self.frames[top].stack.pop().expect("FieldAssign value");
                    let keys = self.pop_field_keys(top, &steps);
                    self.field_set(base, top, &steps, keys, value.clone())?;
                    self.frames[top].stack.push(value);
                }
                Op::FieldAssignOp { base, steps, op } => {
                    let rhs = self.frames[top].stack.pop().expect("FieldAssignOp rhs");
                    let keys = self.pop_field_keys(top, &steps);
                    let old = self.field_value(base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let result = apply_binop(op, &old, &rhs, &mut self.diags)?;
                    self.field_set(base, top, &steps, keys, result.clone())?;
                    self.frames[top].stack.push(result);
                }
                Op::FieldIncDec { base, steps, inc, pre } => {
                    let keys = self.pop_field_keys(top, &steps);
                    let old = self.field_value(base, top, &steps, keys.clone()).unwrap_or(Zval::Null);
                    let mut newv = old.clone();
                    if inc {
                        ops::increment(&mut newv, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut newv, &mut self.diags)?;
                    }
                    self.field_set(base, top, &steps, keys, newv.clone())?;
                    self.frames[top].stack.push(if pre { newv } else { old });
                }
                Op::FieldIsset { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    let set = matches!(
                        self.field_value(base, top, &steps, keys),
                        Some(v) if !matches!(v, Zval::Null | Zval::Undef)
                    );
                    self.frames[top].stack.push(Zval::Bool(set));
                }
                Op::FieldUnset { base, steps } => {
                    let keys = self.pop_field_keys(top, &steps);
                    self.field_remove(base, top, &steps, keys);
                }
                Op::Fatal(i) => {
                    let msg = match &self.frames[top].func.consts[i as usize] {
                        crate::bytecode::Const::Str(b) => String::from_utf8_lossy(b).into_owned(),
                        _ => "VM: unsupported construct".to_string(),
                    };
                    return Err(PhpError::Error(msg));
                }
                Op::EmitNotice(i) => {
                    if let crate::bytecode::Const::Str(b) = &self.frames[top].func.consts[i as usize] {
                        let msg = String::from_utf8_lossy(b).into_owned();
                        self.diags.push(Diag::Notice(msg));
                    }
                }
                Op::MatchError(slot) => {
                    let subj = read_slot(&self.frames[top].slots[slot as usize]);
                    return Err(PhpError::Error(format!(
                        "Unhandled match case {}",
                        match_case_repr(&subj)
                    )));
                }
                Op::Sweep => {
                    let module = self.module;
                    // Release every now-unreachable tracked object, running one
                    // destructor per pass. A destructor is a frame: schedule it and
                    // rewind so this Sweep re-runs (to a fixpoint) once it returns.
                    while let Some(i) =
                        self.created.iter().rposition(|o| Rc::strong_count(o) == 1)
                    {
                        let o = self.created.remove(i);
                        let (cid, id) = {
                            let b = o.borrow();
                            (b.class_id as usize, b.id)
                        };
                        if self.destructed.contains(&id) {
                            continue; // `o` drops here, freeing what it held
                        }
                        // A destructor-less object just drops here; one with a
                        // `__destruct` runs it in a pushed frame (rewind so Sweep
                        // re-runs to a fixpoint after it returns).
                        if let Some((defc, midx)) = resolve_method_runtime(module, cid, b"__destruct") {
                            self.destructed.insert(id);
                            let callee = &module.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee);
                            frame.this = Some(Zval::Object(Rc::clone(&o)));
                            frame.class = Some(defc);
                            frame.static_class = Some(cid);
                            // Discard the destructor's return (don't disturb the
                            // caller's operand stack).
                            frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
                            self.frames[top].ip = ip; // re-run Sweep after it returns
                            self.frames.push(frame);
                            break;
                        }
                    }
                }
                Op::Nop => {}
            }
        }
    }

    /// The cell a [`DimBase`] is rooted at, for read-only path inspection.
    fn base_cell(&self, base: DimBase, top: usize) -> &Zval {
        match base {
            DimBase::Local(s) => &self.frames[top].slots[s as usize],
            DimBase::Global(s) => &self.frames[0].slots[s as usize],
        }
    }

    /// Pop `n` index values off the current frame, restoring source order.
    fn pop_keys(&mut self, top: usize, n: u32) -> Vec<Zval> {
        let mut keys: Vec<Zval> = (0..n)
            .map(|_| self.frames[top].stack.pop().expect("path index key"))
            .collect();
        keys.reverse();
        keys
    }

    /// Dispatch an evaluator-only *host* builtin (Session B1) emitted as
    /// [`Op::CallHostBuiltin`]: the call-a-callable family. `name` is the canonical
    /// lowercased name from [`host_builtin_canonical`].
    fn dispatch_host_builtin(&mut self, name: &[u8], args: Vec<Zval>) -> Result<Zval, PhpError> {
        match name {
            b"call_user_func" => self.ho_call_user_func(args),
            b"call_user_func_array" => self.ho_call_user_func_array(args),
            b"is_callable" => self.ho_is_callable(args),
            b"define" => self.ho_define(args),
            b"defined" => self.ho_defined(args),
            b"constant" => self.ho_constant(args),
            b"array_map" => self.ho_array_map(args),
            b"array_filter" => self.ho_array_filter(args),
            b"array_reduce" => self.ho_array_reduce(args),
            b"get_class" => self.ho_get_class(args),
            b"get_parent_class" => self.ho_get_parent_class(args),
            b"get_object_vars" => self.ho_get_object_vars(args),
            b"get_class_methods" => self.ho_get_class_methods(args),
            b"func_num_args" => self.ho_func_num_args(),
            b"func_get_args" => self.ho_func_get_args(),
            b"func_get_arg" => self.ho_func_get_arg(args),
            b"sprintf" | b"printf" | b"vsprintf" | b"vprintf" | b"fprintf" | b"vfprintf" => {
                self.ho_format(name, args)
            }
            b"function_exists" => self.ho_function_exists(args),
            b"class_exists" => self.ho_class_exists(args),
            b"interface_exists" => self.ho_interface_exists(args),
            b"method_exists" => self.ho_method_exists(args),
            b"property_exists" => self.ho_property_exists(args),
            b"get_called_class" => self.ho_get_called_class(),
            b"error_reporting" => self.ho_error_reporting(args),
            b"trigger_error" | b"user_error" => self.ho_trigger_error(args),
            b"error_get_last" => self.ho_error_get_last(),
            b"set_exception_handler" => self.ho_set_exception_handler(args),
            b"restore_exception_handler" => self.ho_restore_exception_handler(),
            b"set_error_handler" => self.ho_set_error_handler(args),
            b"restore_error_handler" => self.ho_restore_error_handler(),
            b"unserialize" => self.ho_unserialize(args),
            b"fopen" => self.ho_fopen(args),
            b"tmpfile" => self.ho_tmpfile(),
            b"opendir" => self.ho_opendir(args),
            b"preg_replace" => self.ho_preg_replace(args),
            b"preg_quote" => self.ho_preg_quote(args),
            b"preg_split" => self.ho_preg_split(args),
            b"debug_backtrace" => self.ho_debug_backtrace(args),
            b"debug_print_backtrace" => self.ho_debug_print_backtrace(),
            b"preg_replace_callback" => self.ho_preg_replace_callback(args),
            _ => Err(undefined_builtin(name)),
        }
    }

    /// `define($name, $value)` (B3): register a user constant. The name is coerced
    /// to a string; redefining an existing user *or* engine constant warns and
    /// returns `false` (PHP 8.5 message), otherwise stores it and returns `true`.
    /// (The legacy case-insensitive third argument was removed in PHP 8.)
    fn ho_define(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Err(PhpError::Error(
                "define() expects at least 2 arguments, 0 given".to_string(),
            ));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        let value = args.get(1).cloned().unwrap_or(Zval::Null);
        if self.constant_known(&cname) {
            self.diags.push(Diag::Warning(format!(
                "Constant {} already defined, this will be an error in PHP 9",
                String::from_utf8_lossy(&cname)
            )));
            return Ok(Zval::Bool(false));
        }
        self.constants.insert(cname, value);
        Ok(Zval::Bool(true))
    }

    /// `defined($name)` (B3): whether `name` is a known user or engine constant.
    fn ho_defined(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Ok(Zval::Bool(false));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        Ok(Zval::Bool(self.constant_known(&cname)))
    }

    /// `constant($name)` (B3): the value of user constant `name`, else the engine
    /// constant, else the catchable "Undefined constant" `Error`.
    fn ho_constant(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Err(PhpError::Error(
                "constant() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        if let Some(v) = self.constants.get(&cname) {
            return Ok(v.clone());
        }
        if let Some(z) = crate::lower::resolve_constant(&cname).and_then(const_literal_to_zval) {
            return Ok(z);
        }
        Err(PhpError::Error(format!(
            "Undefined constant \"{}\"",
            String::from_utf8_lossy(&cname)
        )))
    }

    /// Whether `name` is a known constant — a user `define()` or an engine constant.
    fn constant_known(&self, name: &[u8]) -> bool {
        self.constants.contains_key(name) || crate::lower::resolve_constant(name).is_some()
    }

    /// `call_user_func($callable, ...$args)`: forward the remaining arguments by
    /// value to the callable (mirrors `eval::ho_call_user_func`).
    fn ho_call_user_func(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = args.into_iter();
        let Some(callee) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "call_user_func() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let argv: Vec<Zval> = it.map(|v| v.deref_clone()).collect();
        self.call_callable(callee.deref_clone(), argv)
    }

    /// `call_user_func_array($callable, $args)`: the second argument is an array
    /// whose *values* become the positional arguments (string-keyed named
    /// arguments are a scope-out, mirroring the evaluator).
    fn ho_call_user_func_array(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "call_user_func_array() expects exactly 2 arguments, {} given",
                args.len()
            )));
        }
        let callee = args[0].deref_clone();
        let argv: Vec<Zval> = match args[1].deref_clone() {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "call_user_func_array(): Argument #2 ($args) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        self.call_callable(callee, argv)
    }

    /// `array_map($callback, ...$arrays)` (Session C): a single array preserves
    /// keys; several arrays re-index 0..max and pass one element from each per row
    /// (missing tails NULL). A NULL callback zips the arrays (single array →
    /// identity). Mirrors `eval::ho_array_map`, calling via `call_callable`.
    fn ho_array_map(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_map() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let cb = args[0].deref_clone();
        let null_cb = matches!(cb, Zval::Null);
        let mut arrays = Vec::with_capacity(args.len() - 1);
        for (i, a) in args[1..].iter().enumerate() {
            match a.deref_clone() {
                Zval::Array(arr) => arrays.push(arr),
                other => {
                    return Err(PhpError::TypeError(format!(
                        "array_map(): Argument #{} must be of type array, {} given",
                        i + 2,
                        other.error_type_name()
                    )))
                }
            }
        }

        let mut out = PhpArray::new();
        if arrays.len() == 1 {
            let entries: Vec<(Key, Zval)> =
                arrays[0].iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
            for (k, v) in entries {
                let mapped = if null_cb { v } else { self.call_callable(cb.clone(), vec![v])? };
                out.insert(k, mapped);
            }
        } else {
            let cols: Vec<Vec<Zval>> = arrays
                .iter()
                .map(|a| a.iter().map(|(_, v)| v.deref_clone()).collect())
                .collect();
            let max = cols.iter().map(|c| c.len()).max().unwrap_or(0);
            for i in 0..max {
                let row: Vec<Zval> =
                    cols.iter().map(|c| c.get(i).cloned().unwrap_or(Zval::Null)).collect();
                let val = if null_cb {
                    let mut tuple = PhpArray::new();
                    for v in row {
                        let _ = tuple.append(v);
                    }
                    Zval::Array(Rc::new(tuple))
                } else {
                    self.call_callable(cb.clone(), row)?
                };
                let _ = out.append(val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `array_filter($array, $callback?, $mode = 0)` (Session C): keys are always
    /// preserved. No callback keeps truthy values; otherwise the callback receives
    /// the value (mode 0), the key (`ARRAY_FILTER_USE_KEY` = 2), or `(value, key)`
    /// (`ARRAY_FILTER_USE_BOTH` = 1). Mirrors `eval::ho_array_filter`.
    fn ho_array_filter(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "array_filter() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arr = match first.deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_filter(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        let cb = match args.get(1) {
            Some(a) => match a.deref_clone() {
                Zval::Null => None,
                v => Some(v),
            },
            None => None,
        };
        let mode = match args.get(2) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };

        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        let mut out = PhpArray::new();
        for (k, v) in entries {
            let keep = match &cb {
                None => convert::to_bool(&v, &mut self.diags),
                Some(c) => {
                    let call_args = match mode {
                        2 => vec![key_to_zval(&k)],
                        1 => vec![v.clone(), key_to_zval(&k)],
                        _ => vec![v.clone()],
                    };
                    let r = self.call_callable(c.clone(), call_args)?;
                    convert::to_bool(&r, &mut self.diags)
                }
            };
            if keep {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `array_reduce($array, $callback, $initial = null)` (Session C): fold the
    /// values left-to-right through `$callback($carry, $item)`, returning the final
    /// carry. (The evaluator has no `array_reduce`, so this is pure VM gain.)
    fn ho_array_reduce(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_reduce() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let arr = match args[0].deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_reduce(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        let cb = args[1].deref_clone();
        let mut carry = args.get(2).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let values: Vec<Zval> = arr.iter().map(|(_, v)| v.deref_clone()).collect();
        for v in values {
            carry = self.call_callable(cb.clone(), vec![carry, v])?;
        }
        Ok(carry)
    }

    /// `get_class($object = null)` (Session B2): the object's class name. A
    /// `Closure` is `"Closure"`. With no argument PHP 8.5 uses the calling `$this`
    /// (now deprecated) and fatals outside object context. Mirrors `eval::ci_get_class`.
    fn ho_get_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = match args.into_iter().next() {
            Some(a) => a.deref_clone(),
            None => {
                let top = self.frames.len() - 1;
                match self.frames[top].this.clone() {
                    Some(t) => {
                        self.diags.push(Diag::Deprecated(
                            "Calling get_class() without arguments is deprecated".to_string(),
                        ));
                        t
                    }
                    None => {
                        return Err(PhpError::Error(
                            "get_class() without arguments must be called from within a class"
                                .to_string(),
                        ))
                    }
                }
            }
        };
        match &v {
            Zval::Object(o) => {
                Ok(Zval::Str(PhpStr::new(o.borrow().class_name.as_bytes().to_vec())))
            }
            Zval::Closure(_) => Ok(Zval::Str(PhpStr::new(b"Closure".to_vec()))),
            other => Err(PhpError::TypeError(format!(
                "get_class(): Argument #1 ($object) must be of type object, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// `get_parent_class($object_or_class = null)` (Session B2): the parent class
    /// name, or `false` when there is none. An object or a *resolvable* class-name
    /// string selects the class; an unresolvable string (or other type) is a
    /// `TypeError`, matching PHP 8.5 (eval returns `false` here, so VM ≥ eval). No
    /// argument uses the current class context.
    fn ho_get_parent_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        let cid: Option<ClassId> = match args.into_iter().next() {
            Some(a) => Some(self.class_arg_to_id(a.deref_clone(), "get_parent_class")?),
            None => self.frames[top].class,
        };
        match cid.and_then(|c| self.module.classes[c].parent) {
            Some(p) => Ok(Zval::Str(PhpStr::new(self.module.classes[p].name.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// Resolve an "object or class-name string" argument to a [`ClassId`], matching
    /// PHP 8.5's `TypeError` for an unresolvable string or a non-object/non-string
    /// value (shared by `get_parent_class` / `get_class_methods`, Session B2).
    fn class_arg_to_id(&self, v: Zval, fname: &str) -> Result<ClassId, PhpError> {
        match v {
            Zval::Object(o) => Ok(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                self.module.class_index.get(&name.to_ascii_lowercase()).copied().ok_or_else(|| {
                    PhpError::TypeError(format!(
                        "{fname}(): Argument #1 ($object_or_class) must be an object or a valid class name, string given"
                    ))
                })
            }
            Zval::Ref(r) => self.class_arg_to_id(r.borrow().clone(), fname),
            other => Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($object_or_class) must be an object or a valid class name, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// `get_object_vars($object)` (Session B2): the object's properties as a
    /// `name => value` array, filtered by visibility from the calling class scope —
    /// from outside only `public`, from within the class the `protected`/`private`
    /// ones too. Dynamic properties are public. Insertion order is preserved.
    /// Mirrors `eval::ci_get_object_vars`.
    fn ho_get_object_vars(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "get_object_vars() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let v = a.deref_clone();
        let Zval::Object(o) = v else {
            return Err(PhpError::TypeError(format!(
                "get_object_vars(): Argument #1 ($object) must be of type object, {} given",
                v.error_type_name()
            )));
        };
        let cur = self.frames[self.frames.len() - 1].class;
        let obj = o.borrow();
        let cid = obj.class_id as usize;
        let mut arr = PhpArray::new();
        for (name, val) in obj.props.iter() {
            let visible = match resolve_prop_decl(self.module, cid, name) {
                Some((vis, decl)) => visible_from(self.module, cur, vis, decl),
                None => true, // dynamic / undeclared property is public
            };
            if visible {
                arr.insert(Key::from_bytes(name), val.clone());
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `get_class_methods($object_or_class)` (Session B2): the class's method names,
    /// walking the inheritance chain child→parent (each name once, child overrides
    /// win), filtered by visibility from the calling scope. An unresolvable
    /// class-name string is a `TypeError` (PHP 8.5; eval returns null → VM ≥ eval).
    /// Interface/abstract-only method names are a scope-out (not carried on the
    /// compiled class). Mirrors `eval::ci_get_class_methods` for concrete methods.
    fn ho_get_class_methods(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "get_class_methods() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let start = self.class_arg_to_id(a.deref_clone(), "get_class_methods")?;
        let cur = self.frames[self.frames.len() - 1].class;
        let mut arr = PhpArray::new();
        let mut seen: Vec<Vec<u8>> = Vec::new();
        let mut c = Some(start);
        while let Some(cc) = c {
            for m in &self.module.classes[cc].methods {
                let lname = m.name.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue; // a more-derived class already defined this name
                }
                seen.push(lname);
                if visible_from(self.module, cur, m.visibility, cc) {
                    let _ = arr.append(Zval::Str(PhpStr::new(m.name.to_vec())));
                }
            }
            c = self.module.classes[cc].parent;
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `function_exists($name)` (Session B4): whether `name` is a user function, a
    /// registry builtin, or a host builtin. A leading `\` is stripped.
    fn ho_function_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "function_exists() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let name = b.strip_prefix(b"\\").unwrap_or(b);
        Ok(Zval::Bool(self.is_name_callable(name)))
    }

    /// Resolve a class-name *string* argument to a [`ClassId`] via the class index
    /// (leading `\` stripped, case-insensitive). `None` if absent or unknown.
    /// Shared by the `*_exists` predicates (Session B4).
    fn resolve_class_name(&mut self, arg: Option<&Zval>) -> Option<ClassId> {
        let a = arg?;
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let name = b.strip_prefix(b"\\").unwrap_or(b);
        self.module.class_index.get(&name.to_ascii_lowercase()).copied()
    }

    /// `class_exists($name, $autoload = true)` (Session B4): whether `name` names a
    /// declared class — including `abstract` and `enum`, but NOT an interface
    /// (matching PHP 8.5). The autoload flag is a no-op (no autoloading is modelled).
    fn ho_class_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let exists = match self.resolve_class_name(args.first()) {
            Some(cid) => self.module.classes[cid].instantiable != Instantiable::Interface,
            None => false,
        };
        Ok(Zval::Bool(exists))
    }

    /// `interface_exists($name, $autoload = true)` (Session B4): whether `name`
    /// names a declared interface.
    fn ho_interface_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let is_iface = matches!(
            self.resolve_class_name(args.first()),
            Some(cid) if self.module.classes[cid].instantiable == Instantiable::Interface
        );
        Ok(Zval::Bool(is_iface))
    }

    /// `method_exists($object_or_class, $method)` (Session B4): whether the class of
    /// the object / named class defines `method` (walking the inheritance chain). An
    /// unresolvable target is `false` (no error, unlike `get_class_methods`).
    fn ho_method_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(a0), Some(a1)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(
                "method_exists() expects exactly 2 arguments".to_string(),
            ));
        };
        let Some(cid) = self.class_id_from_value(&a0.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        let m = convert::to_zstr_cast(&a1.deref_clone(), &mut self.diags);
        Ok(Zval::Bool(resolve_method_runtime(self.module, cid, m.as_bytes()).is_some()))
    }

    /// `property_exists($object_or_class, $property)` (Session B4): whether the class
    /// declares an instance or static `property` (any visibility) — or, for an object
    /// argument, whether the instance carries it as a dynamic property. Mirrors PHP:
    /// visibility is ignored, an unresolvable target is `false`.
    fn ho_property_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(a0), Some(a1)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(
                "property_exists() expects exactly 2 arguments".to_string(),
            ));
        };
        let v = a0.deref_clone();
        let Some(cid) = self.class_id_from_value(&v) else {
            return Ok(Zval::Bool(false));
        };
        let pname_z = convert::to_zstr_cast(&a1.deref_clone(), &mut self.diags);
        let pname = pname_z.as_bytes();
        if resolve_prop_decl(self.module, cid, pname).is_some()
            || find_static_prop(self.module, cid, pname).is_some()
        {
            return Ok(Zval::Bool(true));
        }
        if let Zval::Object(o) = &v {
            if o.borrow().props.get(pname).is_some() {
                return Ok(Zval::Bool(true));
            }
        }
        Ok(Zval::Bool(false))
    }

    /// `get_called_class()` (Session B4): the late-static-binding class name (the
    /// receiver's actual class), a fatal `Error` outside class context.
    fn ho_get_called_class(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        match self.frames[top].static_class {
            Some(cid) => Ok(Zval::Str(PhpStr::new(self.module.classes[cid].name.to_vec()))),
            None => Err(PhpError::Error(
                "get_called_class() must be called from within a class".to_string(),
            )),
        }
    }

    /// `preg_replace_callback($pattern, $callback, $subject)` (Session 3): replace
    /// each match of `pattern` in `subject` with the string returned by `callback`
    /// (called with the match array). A single pattern/subject, mirroring
    /// `eval::ho_preg_replace_callback`; the callback runs via `call_callable` and
    /// its result is stringified (honouring `__toString`). An invalid pattern yields
    /// null. The optional `limit`/`count` arguments are a scope-out.
    fn ho_preg_replace_callback(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace_callback() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let callback = args[1].deref_clone();
        let subject =
            convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let bytes = subj.as_bytes().to_vec();
        // Collect (range, match-array) up front so the regex borrow of `subj` ends
        // before we re-enter the VM via the callback.
        let hits: Vec<(usize, usize, Zval)> = re
            .captures_iter(&subj)
            .into_iter()
            .map(|caps| {
                let m0 = caps.get(0).expect("match has group 0");
                (m0.start, m0.end, crate::preg::captures_array(&re, &caps, 0))
            })
            .collect();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        for (start, end, match_arr) in hits {
            out.extend_from_slice(&bytes[last..start]);
            let ret = self.call_callable(callback.clone(), vec![match_arr])?;
            let rs = self.vm_stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&bytes[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `preg_replace($pattern, $replacement, $subject)`: backreferences `$1`/`${1}`/
    /// `\1` in the replacement are honoured. Returns `null` on a bad pattern. Single
    /// scalar pattern/subject (array forms are a scope-out). Mirrors
    /// `eval::ho_preg_replace` on the shared `crate::preg` engine.
    fn ho_preg_replace(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let repl = convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let repl = String::from_utf8_lossy(&crate::preg::translate_replacement(&repl)).into_owned();
        let subj = String::from_utf8_lossy(&subject);
        let result = re.replace_all(&subj, repl.as_str());
        Ok(Zval::Str(PhpStr::new(result.as_bytes().to_vec())))
    }

    /// `preg_quote($str, $delimiter = null)`: escape regex metacharacters (and the
    /// optional delimiter). Mirrors `eval::ho_preg_quote` on `crate::preg::quote`.
    fn ho_preg_quote(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "preg_quote() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let s = convert::to_zstr_cast(&first.deref_clone(), &mut self.diags).as_bytes().to_vec();
        let delim = match args.get(1) {
            Some(a) => convert::to_zstr_cast(&a.deref_clone(), &mut self.diags)
                .as_bytes()
                .first()
                .copied(),
            None => None,
        };
        Ok(Zval::Str(PhpStr::new(crate::preg::quote(&s, delim))))
    }

    /// `preg_split($pattern, $subject, $limit = -1, $flags = 0)`: split `$subject`
    /// on matches of `$pattern`. Returns `false` on a bad pattern. Mirrors
    /// `eval::ho_preg_split` on the shared `crate::preg` engine (no-empty /
    /// delim-capture / offset-capture flags honoured; positive limit caps pieces).
    fn ho_preg_split(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_split() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let limit = match args.get(2) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => -1,
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let no_empty = flags & 1 != 0;
        let delim_capture = flags & 2 != 0;
        let offset_capture = flags & 4 != 0;
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let mut arr = PhpArray::new();
        let mut last = 0usize;
        let push = |arr: &mut PhpArray, text: &str, off: usize| {
            if no_empty && text.is_empty() {
                return;
            }
            if offset_capture {
                let _ = arr.append(crate::preg::offset_pair(
                    Zval::Str(PhpStr::new(text.as_bytes().to_vec())),
                    off as i64,
                ));
            } else {
                let _ = arr.append(Zval::Str(PhpStr::new(text.as_bytes().to_vec())));
            }
        };
        for (idx, caps) in re.captures_iter(&subj).into_iter().enumerate() {
            let m0 = caps.get(0).unwrap();
            if limit > 0 && idx as i64 + 1 >= limit {
                break;
            }
            push(&mut arr, &subj[last..m0.start], last);
            if delim_capture {
                for g in 1..caps.len() {
                    if let Some(mm) = caps.get(g) {
                        push(&mut arr, mm.text.as_str(), mm.start);
                    }
                }
            }
            last = m0.end;
        }
        push(&mut arr, &subj[last..], last);
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// Dispatch a host builtin with a by-reference output parameter (Session:
    /// out-param). Returns `(result, out_value)`; the VM writes `out_value` into the
    /// caller's out-param slot. `_out_index` is the argument position of the
    /// out-param (always the same per builtin; kept for symmetry / future use).
    fn dispatch_host_builtin_out(
        &mut self,
        name: &[u8],
        args: Vec<Zval>,
        _out_index: usize,
    ) -> Result<(Zval, Zval), PhpError> {
        match name {
            b"preg_match" => self.ho_preg_match(args),
            b"preg_match_all" => self.ho_preg_match_all(args),
            _ => Err(undefined_builtin(name)),
        }
    }

    /// `preg_match($pattern, $subject, &$matches = null, $flags = 0)`: returns 1 on
    /// a match, 0 on none, `false` on a bad pattern. Yields `(ret, matches_array)`;
    /// `$matches` is written by the VM out-param path. Mirrors `eval::ho_preg_match`.
    fn ho_preg_match(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let subj = String::from_utf8_lossy(&subject);
        let (ret, matches) = match re.captures(&subj) {
            Some(caps) => (1, crate::preg::captures_array(&re, &caps, flags)),
            None => (0, Zval::Array(Rc::new(PhpArray::new()))),
        };
        Ok((Zval::Long(ret), matches))
    }

    /// `preg_match_all($pattern, $subject, &$matches = null, $flags = 0)`: default
    /// PREG_PATTERN_ORDER — `$matches[g]` is group `g`'s text across all matches;
    /// PREG_SET_ORDER gives one full match array per match. Returns the match count
    /// (or `false` on a bad pattern). Mirrors `eval::ho_preg_match_all`.
    fn ho_preg_match_all(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        use crate::preg::{capture_value, PREG_OFFSET_CAPTURE, PREG_SET_ORDER, PREG_UNMATCHED_AS_NULL};
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match_all() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let offset = flags & PREG_OFFSET_CAPTURE != 0;
        let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
        let mut count: i64 = 0;
        let outer = if flags & PREG_SET_ORDER != 0 {
            let mut outer = PhpArray::new();
            for caps in re.captures_iter(&subj) {
                count += 1;
                let _ = outer.append(crate::preg::captures_array(&re, &caps, flags));
            }
            outer
        } else {
            let ngroups = re.captures_len();
            let names = re.capture_names();
            let mut cols: Vec<PhpArray> = (0..ngroups).map(|_| PhpArray::new()).collect();
            for caps in re.captures_iter(&subj) {
                count += 1;
                for (g, col) in cols.iter_mut().enumerate() {
                    let _ = col.append(capture_value(caps.get(g), offset, as_null));
                }
            }
            let mut outer = PhpArray::new();
            for (g, col) in cols.into_iter().enumerate() {
                let col_z = Zval::Array(Rc::new(col));
                if let Some(Some(name)) = names.get(g) {
                    outer.insert(Key::from_bytes(name.as_bytes()), col_z.clone());
                }
                outer.insert(Key::Int(g as i64), col_z);
            }
            outer
        };
        Ok((Zval::Long(count), Zval::Array(Rc::new(outer))))
    }

    /// `error_reporting($level = null)` (Session 1): set the active reporting
    /// bitmask (consulted by [`Self::flush_diags`]) and return the previous one; a
    /// `null`/absent argument reads without changing it.
    fn ho_error_reporting(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let old = self.error_level;
        if let Some(a) = args.first() {
            let v = a.deref_clone();
            if !matches!(v, Zval::Null) {
                self.error_level = convert::to_long_cast(&v, &mut self.diags);
            }
        }
        Ok(Zval::Long(old))
    }

    /// `trigger_error($message, $level = E_USER_NOTICE)` (Session 1): raise a user
    /// diagnostic. `E_USER_ERROR` becomes a fatal; the others render as
    /// Warning/Notice/Deprecated (gated by `error_reporting`). An invalid level is a
    /// `ValueError`. Records the error for [`Self::ho_error_get_last`].
    fn ho_trigger_error(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(msg_arg) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "trigger_error() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let msg = convert::to_zstr_cast(&msg_arg.deref_clone(), &mut self.diags).as_bytes().to_vec();
        let level = match args.get(1) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 1024, // E_USER_NOTICE
        };
        if !matches!(level, 256 | 512 | 1024 | 16384) {
            return Err(PhpError::ValueError(
                "trigger_error(): Argument #2 ($error_level) must be one of E_USER_ERROR, E_USER_WARNING, E_USER_NOTICE, or E_USER_DEPRECATED"
                    .to_string(),
            ));
        }
        let line = self.cur_line(self.frames.len() - 1);
        if level == 256 {
            self.flush_diags(line)?;
            // PHP 8.4+: passing E_USER_ERROR to trigger_error() is itself deprecated.
            // The oracle emits this E_DEPRECATED first (routed to any handler too),
            // *then* processes the E_USER_ERROR — so a handler sees both 8192 and 256.
            self.raise_diagnostic(
                8192,
                "Passing E_USER_ERROR to trigger_error() is deprecated since 8.4, throw an exception or call exit with a string message instead",
                line,
            )?;
            let message = String::from_utf8_lossy(&msg).into_owned();
            // If a handler is registered for E_USER_ERROR and handles it (truthy
            // return), the script CONTINUES (oracle-confirmed; error_get_last stays
            // unset, mirroring a handler-suppressed diagnostic). Otherwise — no
            // handler, masked out, or a `false` return — it is the fatal: record
            // `last_error` (the default/fatal handler ran) and propagate.
            if let Some(true) = self.route_to_handler(256, &message, line)? {
                return Ok(Zval::Bool(true));
            }
            self.last_error = Some((level, msg.clone(), line));
            return Err(PhpError::Error(message));
        }
        // Flush any pending built-in diagnostics, then route this user diagnostic
        // through the shared chokepoint so a `set_error_handler` callback sees it.
        // The default render is gated on the user level itself (E_USER_*), not the
        // label's built-in bit, since e.g. E_USER_DEPRECATED (16384) and
        // E_DEPRECATED (8192) are independent.
        self.flush_diags(line)?;
        let message = String::from_utf8_lossy(&msg).into_owned();
        self.raise_diagnostic(level, &message, line)?;
        Ok(Zval::Bool(true))
    }

    /// `error_get_last()`: the most recent diagnostic as `[type, message, file,
    /// line]`, or null. Captures both `trigger_error` and built-in warnings/notices
    /// (Session 2; recorded at the [`Self::raise_diagnostic`] chokepoint).
    fn ho_error_get_last(&mut self) -> Result<Zval, PhpError> {
        // Realize any diagnostic still pending in `self.diags`: the VM flushes diags
        // lazily (at the next echo/builtin), so a warning raised mid-expression has
        // not yet updated `last_error` when `error_get_last()` is read right after it.
        // Flushing here — the same realize-state move `emit_str`/`run_value_builtin`
        // make — captures it (mirrors PHP's synchronous-at-emission `last_error`).
        let line = self.cur_line(self.frames.len() - 1);
        self.flush_diags(line)?;
        match &self.last_error {
            Some((level, msg, line)) => {
                let mut arr = PhpArray::new();
                arr.insert(Key::from_bytes(b"type"), Zval::Long(*level));
                arr.insert(Key::from_bytes(b"message"), Zval::Str(PhpStr::new(msg.clone())));
                arr.insert(
                    Key::from_bytes(b"file"),
                    Zval::Str(PhpStr::new(self.module.file.to_vec())),
                );
                arr.insert(Key::from_bytes(b"line"), Zval::Long(*line as i64));
                Ok(Zval::Array(Rc::new(arr)))
            }
            None => Ok(Zval::Null),
        }
    }

    /// `set_exception_handler($callable)` (Session 1b): install a top-level handler
    /// for uncaught throwables; returns the previously-active handler (or null).
    fn ho_set_exception_handler(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.exception_handlers.last().cloned();
        let handler = args.into_iter().next().unwrap_or(Zval::Null);
        self.exception_handlers.push(handler);
        Ok(prev.unwrap_or(Zval::Null))
    }

    /// `restore_exception_handler()` (Session 1b): pop the current handler, making
    /// the previous one active again. Always returns true.
    fn ho_restore_exception_handler(&mut self) -> Result<Zval, PhpError> {
        self.exception_handlers.pop();
        Ok(Zval::Bool(true))
    }

    /// `set_error_handler($callable, $levels = E_ALL)` (Session 2): install a
    /// user diagnostic handler routed by [`Self::raise_diagnostic`]; returns the
    /// previously-active handler (or null). The optional level mask gates which
    /// E_* numbers reach the handler (default `E_ALL` = 30719).
    fn ho_set_error_handler(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.error_handlers.last().map(|(cb, _)| cb.clone());
        let mut it = args.into_iter();
        let handler = it.next().unwrap_or(Zval::Null);
        let level = match it.next() {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 30719, // E_ALL (PHP 8.5)
        };
        self.error_handlers.push((handler, level));
        Ok(prev.unwrap_or(Zval::Null))
    }

    /// `restore_error_handler()` (Session 2): pop the current handler, re-exposing
    /// the previous one (or the engine default). Always returns true.
    fn ho_restore_error_handler(&mut self) -> Result<Zval, PhpError> {
        self.error_handlers.pop();
        Ok(Zval::Bool(true))
    }

    /// `unserialize($str)`: rebuild a value from PHP's serialization format. A
    /// host builtin because reconstructing an object needs the class table and id
    /// allocator. Mirrors `eval::ho_unserialize`: the shared
    /// [`crate::unserialize::parse`] decodes a pure [`Ser`](crate::unserialize::Ser)
    /// tree, then [`Self::vm_ser_to_zval`] materialises it. Malformed input yields
    /// `false` with PHP's Warning. `__wakeup` is not called (D-50 scope-out), and an
    /// unknown class falls back to `stdClass` (PHP makes a `__PHP_Incomplete_Class`).
    fn ho_unserialize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "unserialize() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arg0 = first.deref_clone();
        let bytes = convert::to_zstr_cast(&arg0, &mut self.diags);
        let nbytes = bytes.as_bytes().len();
        match crate::unserialize::parse(bytes.as_bytes()) {
            Some(s) => Ok(self.vm_ser_to_zval(s)),
            None => {
                // PHP reports the failing offset; we do not track it, so report 0
                // (matches `eval`, D-50).
                self.diags.push(Diag::Warning(format!(
                    "unserialize(): Error at offset 0 of {nbytes} bytes"
                )));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// Turn a decoded [`Ser`](crate::unserialize::Ser) tree into a `Zval`, recursing
    /// into arrays/objects. Mirrors `eval::ser_to_zval`; objects go through
    /// [`Self::vm_make_unserialized_object`] (the VM's class table / id allocator).
    fn vm_ser_to_zval(&mut self, s: crate::unserialize::Ser) -> Zval {
        use crate::unserialize::Ser;
        match s {
            Ser::Null => Zval::Null,
            Ser::Bool(b) => Zval::Bool(b),
            Ser::Long(n) => Zval::Long(n),
            Ser::Double(d) => Zval::Double(d),
            Ser::Str(bytes) => Zval::Str(PhpStr::new(bytes)),
            Ser::Array(items) => {
                let mut arr = PhpArray::new();
                for (k, v) in items {
                    let key = match k {
                        Ser::Long(i) => Key::Int(i),
                        // A string key coerces to int when canonically numeric.
                        Ser::Str(b) => Key::from_bytes(&b),
                        _ => continue,
                    };
                    let val = self.vm_ser_to_zval(v);
                    arr.insert(key, val);
                }
                Zval::Array(Rc::new(arr))
            }
            Ser::Object(class, props) => {
                let fields: Vec<(Vec<u8>, Zval)> = props
                    .into_iter()
                    .map(|(name, v)| (name, self.vm_ser_to_zval(v)))
                    .collect();
                self.vm_make_unserialized_object(&class, fields)
            }
        }
    }

    /// Build an object of named `class` with the given properties, the constructor
    /// **not** run (as PHP's `unserialize` does). Mirrors `eval::make_object` on the
    /// VM's machinery (`Self::alloc_object`'s construction, but with the serialized
    /// props instead of declared defaults). An unknown class falls back to
    /// `stdClass` (D-50).
    fn vm_make_unserialized_object(&mut self, class: &[u8], fields: Vec<(Vec<u8>, Zval)>) -> Zval {
        let module = self.module; // &'m Module: detach from the `self` borrow.
        let lower = class.to_ascii_lowercase();
        let cid = module
            .class_index
            .get(lower.as_slice())
            .or_else(|| module.class_index.get(&b"stdclass"[..]))
            .copied();
        let Some(cid) = cid else {
            // No stdClass in the prelude (should never happen) — degrade gracefully.
            return Zval::Null;
        };
        let cc = &module.classes[cid];
        let class_name = Rc::clone(&cc.class_name);
        let info = Rc::clone(&cc.info);
        let mut props = Props::new();
        for (k, v) in fields {
            props.set(&k, v);
        }
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info };
        let rc = Rc::new(RefCell::new(obj));
        // Track for `__destruct` (OOP-3d), like every other freshly minted object.
        self.created.push(Rc::clone(&rc));
        Zval::Object(rc)
    }

    /// Walk the call stack into structured backtrace entries (shared by
    /// `debug_backtrace` / `debug_print_backtrace`). Reports the frames from the
    /// caller of the `debug_*` builtin (the top frame — the builtin pushes none)
    /// down to, but excluding, the top-level script body (frame 0). Each entry's
    /// `line` is the *call-site* line: the caller frame's current line, which —
    /// because `run_loop` advances `ip` past the `Call` op before dispatching it —
    /// `cur_line(i - 1)` resolves to the `Call` op's own line.
    fn collect_backtrace(&self) -> Vec<BtFrame> {
        let top = self.frames.len() - 1;
        let mut out = Vec::new();
        for i in (1..=top).rev() {
            let f = &self.frames[i];
            let function = if f.func.name.is_empty() {
                b"{closure}".to_vec()
            } else {
                f.func.name.to_vec()
            };
            let (class, object) = match f.class {
                Some(cid) => (Some(self.module.classes[cid].name.to_vec()), f.this.clone()),
                None => (None, None),
            };
            out.push(BtFrame {
                function,
                line: self.cur_line(i - 1),
                class,
                // A method with no bound `$this` is a static call ("::"); otherwise "->".
                is_static: f.class.is_some() && f.this.is_none(),
                object,
                args: self.current_frame_args(i),
            });
        }
        out
    }

    /// `debug_backtrace()`: the call stack as an array of per-frame arrays with
    /// `file`/`line`/`function`/`args` (plus `class`/`object`/`type` for a method).
    /// Pure VM gain — the tree-walker has no equivalent. Options args are a scope-out.
    fn ho_debug_backtrace(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        let frames = self.collect_backtrace();
        let file = self.module.file.to_vec();
        let mut outer = PhpArray::new();
        for bt in frames {
            let mut e = PhpArray::new();
            e.insert(Key::from_bytes(b"file"), Zval::Str(PhpStr::new(file.clone())));
            e.insert(Key::from_bytes(b"line"), Zval::Long(bt.line as i64));
            e.insert(Key::from_bytes(b"function"), Zval::Str(PhpStr::new(bt.function)));
            if let Some(cls) = bt.class {
                e.insert(Key::from_bytes(b"class"), Zval::Str(PhpStr::new(cls)));
                if let Some(obj) = bt.object {
                    e.insert(Key::from_bytes(b"object"), obj);
                }
                let ty: &[u8] = if bt.is_static { b"::" } else { b"->" };
                e.insert(Key::from_bytes(b"type"), Zval::Str(PhpStr::new(ty.to_vec())));
            }
            let mut argsarr = PhpArray::new();
            for a in bt.args {
                let _ = argsarr.append(a);
            }
            e.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(argsarr)));
            let _ = outer.append(Zval::Array(Rc::new(e)));
        }
        Ok(Zval::Array(Rc::new(outer)))
    }

    /// `debug_print_backtrace()`: print the call stack as
    /// `#N file(line): callee(args)` lines. Args render in PHP's compact form
    /// (scalars literal, strings single-quoted+truncated, arrays `Array`, objects
    /// `Object(Class)`). Pure VM gain.
    fn ho_debug_print_backtrace(&mut self) -> Result<Zval, PhpError> {
        let frames = self.collect_backtrace();
        let file = String::from_utf8_lossy(&self.module.file).into_owned();
        let mut s = String::new();
        for (n, bt) in frames.iter().enumerate() {
            let callee = match &bt.class {
                Some(cls) => format!(
                    "{}{}{}",
                    String::from_utf8_lossy(cls),
                    if bt.is_static { "::" } else { "->" },
                    String::from_utf8_lossy(&bt.function)
                ),
                None => String::from_utf8_lossy(&bt.function).into_owned(),
            };
            let argstr = bt
                .args
                .iter()
                .map(format_bt_arg)
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!("#{n} {file}({}): {callee}({argstr})\n", bt.line));
        }
        // Flush pending diagnostics first so the trace lands in output order, then
        // append to both streams (this is ordinary output, like an echo).
        let line = self.cur_line(self.frames.len() - 1);
        self.flush_diags(line)?;
        self.stdout.extend_from_slice(s.as_bytes());
        self.rendered.extend_from_slice(s.as_bytes());
        Ok(Zval::Null)
    }

    /// Wrap a freshly opened stream in a `Zval::Resource` with the next id (mirrors
    /// `eval::alloc_resource`). The whole `fread`/`fwrite`/`fclose`/… family is in
    /// the shared registry and operates on the `Rc<RefCell<Resource>>` by value, so
    /// minting the resource here is all the VM needs to unlock it.
    fn alloc_resource(&mut self, stream: Stream) -> Zval {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        Zval::Resource(Rc::new(RefCell::new(Resource::new(id, stream))))
    }

    /// `fopen($filename, $mode, …)`: open a real file or a `php://` wrapper and mint
    /// a stream resource. A host builtin because it allocates a resource id. Args 3/4
    /// (use_include_path, context) are a scope-out. On failure: Warning + `false`.
    /// Mirrors `eval::ho_fopen`.
    fn ho_fopen(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(path_arg) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "fopen() expects at least 2 arguments, 0 given".to_string(),
            ));
        };
        let Some(mode_arg) = args.get(1) else {
            return Err(PhpError::ArgumentCountError(
                "fopen() expects at least 2 arguments, 1 given".to_string(),
            ));
        };
        let path = convert::to_zstr_cast(&path_arg.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        let mode = convert::to_zstr_cast(&mode_arg.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        if let Some(spec) = path.strip_prefix(b"php://".as_slice()) {
            return match open_php_stream(spec, &mode) {
                Some(stream) => Ok(self.alloc_resource(stream)),
                None => {
                    self.diags.push(Diag::Warning(format!(
                        "fopen({}): Failed to open stream: no suitable wrapper could be found",
                        String::from_utf8_lossy(&path)
                    )));
                    Ok(Zval::Bool(false))
                }
            };
        }
        match open_file_stream(&path, &mode) {
            Ok(stream) => Ok(self.alloc_resource(stream)),
            Err(msg) => {
                self.diags.push(Diag::Warning(format!(
                    "fopen({}): Failed to open stream: {msg}",
                    String::from_utf8_lossy(&path)
                )));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `tmpfile()`: create a fresh temp file opened read+write, then immediately
    /// unlink it (PHP's auto-removal). `false` on failure. Mirrors `eval::ho_tmpfile`.
    fn ho_tmpfile(&mut self) -> Result<Zval, PhpError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir();
        for _ in 0..100 {
            let n = CTR.fetch_add(1, Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);
            let mut path = dir.clone();
            path.push(format!("phpr_tmp_{:x}_{nanos:x}_{n:x}", std::process::id()));
            match std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(f) => {
                    let _ = std::fs::remove_file(&path);
                    let stream = Stream {
                        backend: StreamBackend::File(f),
                        readable: true,
                        writable: true,
                        eof: false,
                    };
                    return Ok(self.alloc_resource(stream));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Ok(Zval::Bool(false)),
            }
        }
        Ok(Zval::Bool(false))
    }

    /// `opendir($directory)`: snapshot the directory entries (`.`/`..` first, then
    /// OS order) into a `DirHandle` resource; `false` + Warning on failure. Mirrors
    /// `eval::ho_opendir`.
    fn ho_opendir(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        let Some(path_arg) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "opendir() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let path = convert::to_zstr_cast(&path_arg.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        match std::fs::read_dir(std::ffi::OsStr::from_bytes(&path)) {
            Ok(rd) => {
                let mut entries = vec![b".".to_vec(), b"..".to_vec()];
                for e in rd.flatten() {
                    entries.push(e.file_name().as_os_str().as_bytes().to_vec());
                }
                let id = self.next_resource_id;
                self.next_resource_id += 1;
                Ok(Zval::Resource(Rc::new(RefCell::new(Resource {
                    id,
                    kind: ResKind::Dir(DirHandle { entries, pos: 0 }),
                }))))
            }
            Err(e) => {
                let msg = e.to_string();
                let msg = msg.split(" (os error").next().unwrap_or(&msg);
                self.diags.push(Diag::Warning(format!(
                    "opendir({}): Failed to open directory: {msg}",
                    String::from_utf8_lossy(&path)
                )));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// Reconstruct the flat argument list of the currently executing frame for the
    /// `func_get_args` family (Session D1): declared parameters are read live from
    /// their slots (so a parameter reassigned in the body is reflected, matching
    /// PHP), while surplus arguments come from the variadic array (variadic callee)
    /// or the `extra_args` snapshot taken at bind time (non-variadic callee).
    fn current_frame_args(&self, top: usize) -> Vec<Zval> {
        let frame = &self.frames[top];
        let a = frame.argc as usize;
        let p = frame.func.n_params as usize;
        let mut out = Vec::with_capacity(a);
        match frame.func.variadic_slot {
            None => {
                for i in 0..a {
                    if i < p {
                        out.push(frame.slots[i].deref_clone());
                    } else {
                        out.push(frame.extra_args[i - p].deref_clone());
                    }
                }
            }
            Some(vs) => {
                let v = vs as usize;
                for i in 0..a.min(v) {
                    out.push(frame.slots[i].deref_clone());
                }
                if a > v {
                    if let Zval::Array(arr) = &frame.slots[v] {
                        for (_, e) in arr.iter() {
                            out.push(e.deref_clone());
                        }
                    }
                }
            }
        }
        out
    }

    /// `func_num_args()` (Session D1): the number of arguments passed to the current
    /// function. A fatal `Error` at global scope, matching PHP 8.5.
    fn ho_func_num_args(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_num_args() must be called from a function context".to_string(),
            ));
        }
        Ok(Zval::Long(self.frames[top].argc as i64))
    }

    /// `func_get_args()` (Session D1): the current function's arguments as a 0-indexed
    /// array. A fatal `Error` at global scope.
    fn ho_func_get_args(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_get_args() must be called from a function context".to_string(),
            ));
        }
        let mut arr = PhpArray::new();
        for v in self.current_frame_args(top) {
            let _ = arr.append(v);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `func_get_arg($position)` (Session D1): the argument at `position`. A fatal
    /// `Error` at global scope; a `ValueError` if `position` is out of range.
    fn ho_func_get_arg(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_get_arg() must be called from a function context".to_string(),
            ));
        }
        let Some(a0) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "func_get_arg() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let pos = convert::to_long_cast(&a0.deref_clone(), &mut self.diags);
        let argc = self.frames[top].argc as i64;
        if pos < 0 || pos >= argc {
            return Err(PhpError::ValueError(
                "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function".to_string(),
            ));
        }
        let all = self.current_frame_args(top);
        Ok(all[pos as usize].clone())
    }

    /// The `sprintf`/`printf` family (Session D2): resolve object arguments to their
    /// `__toString` form (recursively through arrays) *before* handing them to the
    /// pure registry format engine, so `%s` on an object honours `__toString`.
    /// Mirrors `eval::ho_format`; the engine writes to stdout for the `printf`
    /// variants, so the call goes through [`Self::run_value_builtin`] for the
    /// faithful rendered-stream interleaving.
    fn ho_format(&mut self, name: &[u8], args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.format_resolve_objects(a)?);
        }
        let f = match self.registry.get(name) {
            Some(Builtin::Value(f)) => *f,
            _ => return Err(undefined_builtin(name)),
        };
        let top = self.frames.len() - 1;
        let line = self.cur_line(top);
        self.run_value_builtin(f, &argv, line)
    }

    /// Replace every object (recursively, inside arrays) with its `__toString`
    /// string so the pure format engine sees only scalars. Mirrors
    /// `eval::format_resolve_objects`.
    fn format_resolve_objects(&mut self, v: Zval) -> Result<Zval, PhpError> {
        let v = v.deref_clone();
        match v {
            Zval::Object(_) => Ok(Zval::Str(self.vm_stringify(&v)?)),
            Zval::Array(arr) => {
                let mut out = PhpArray::new();
                for (k, e) in arr.iter() {
                    out.insert(k.clone(), self.format_resolve_objects(e.deref_clone())?);
                }
                Ok(Zval::Array(Rc::new(out)))
            }
            other => Ok(other),
        }
    }

    /// Convert a value to a string, running `__toString` for an object via a nested
    /// bounded run (the synchronous analogue of [`Op::Stringify`]). A non-object is
    /// coerced directly; an object without `__toString` is the usual fatal `Error`.
    fn vm_stringify(&mut self, v: &Zval) -> Result<Rc<PhpStr>, PhpError> {
        match v {
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                match resolve_method_runtime(self.module, cid, b"__toString") {
                    Some((defc, midx)) => {
                        let callee = &self.module.classes[defc].methods[midx].func;
                        let baseline = self.frames.len();
                        let mut frame = Frame::new(callee);
                        frame.this = Some(v.clone());
                        frame.class = Some(defc);
                        frame.static_class = Some(cid);
                        frame.ret_stringify = true;
                        self.frames.push(frame);
                        let result = self.drive_to_return(baseline)?;
                        Ok(convert::to_zstr(&result, &mut self.diags))
                    }
                    None => {
                        let name =
                            String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned();
                        Err(PhpError::Error(format!(
                            "Object of class {name} could not be converted to string"
                        )))
                    }
                }
            }
            other => Ok(convert::to_zstr(other, &mut self.diags)),
        }
    }

    /// Dispatch a by-reference-first host builtin (`usort`, Session C): `slot` is
    /// the array variable in the current (caller) frame, taken by reference; `rest`
    /// are the remaining by-value arguments. The canonical name comes from
    /// [`host_builtin_ref_first`].
    fn dispatch_host_builtin_ref(
        &mut self,
        name: &[u8],
        slot: Slot,
        rest: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        match name {
            b"usort" => self.ho_usort(slot, rest),
            b"array_walk" => self.ho_array_walk(slot, rest),
            b"reset" => self.ho_array_pointer(slot, PtrOp::Reset),
            b"end" => self.ho_array_pointer(slot, PtrOp::End),
            b"next" => self.ho_array_pointer(slot, PtrOp::Next),
            b"prev" => self.ho_array_pointer(slot, PtrOp::Prev),
            b"current" => self.ho_array_pointer(slot, PtrOp::Current),
            b"key" => self.ho_array_pointer(slot, PtrOp::Key),
            _ => Err(undefined_builtin(name)),
        }
    }

    /// The array internal-pointer family (`reset`/`end`/`next`/`prev`/`current`/
    /// `key`): operate on the array in `slot` (following a reference), mutating or
    /// reading its cursor. `current`/`prev`/`next`/`reset`/`end` return the value at
    /// the pointer (or `false`); `key` returns the key (or `null`). A non-array
    /// argument is a `TypeError`. Pure VM gain — the tree-walker has no equivalent.
    fn ho_array_pointer(&mut self, slot: Slot, op: PtrOp) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        match &mut self.frames[top].slots[slot as usize] {
            Zval::Ref(rc) => {
                let mut inner = rc.borrow_mut();
                array_pointer_apply(&mut inner, op)
            }
            other => array_pointer_apply(other, op),
        }
    }

    /// `array_walk(&$array, $callback, $arg = null)` (Session C): apply `$callback`
    /// to each element as `($value, $key[, $arg])`. When the callback's first
    /// parameter is by-reference the element is passed through a shared cell and
    /// the mutation is written back; otherwise it is read-only. Keys are never
    /// modified. Returns true. Mirrors `eval::ho_array_walk`.
    fn ho_array_walk(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = rest.into_iter();
        let Some(callback) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "array_walk() expects at least 2 arguments, 1 given".to_string(),
            ));
        };
        let callback = callback.deref_clone();
        let extra = it.next().map(|e| e.deref_clone());
        let by_ref = self.callable_first_by_ref(&callback);
        let top = self.frames.len() - 1;
        let entries: Vec<(Key, Zval)> = match self.frames[top].slots[slot as usize].deref_clone() {
            Zval::Array(a) => a.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_walk(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };

        let mut out = PhpArray::new();
        for (k, v) in entries {
            let key_z = key_to_zval(&k);
            let new_v = if by_ref {
                let vcell = Rc::new(RefCell::new(v));
                let mut argv = vec![Zval::Ref(Rc::clone(&vcell)), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_callable(callback.clone(), argv)?;
                // Bind before the block ends so the `Ref` temporary is dropped
                // before `vcell`, satisfying the borrow checker.
                let updated = vcell.borrow().clone();
                updated
            } else {
                let mut argv = vec![v.clone(), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_callable(callback.clone(), argv)?;
                v
            };
            out.insert(k, new_v);
        }
        let top = self.frames.len() - 1;
        self.frames[top].slots[slot as usize] = Zval::Array(Rc::new(out));
        Ok(Zval::Bool(true))
    }

    /// Whether a callable's first parameter is declared by-reference (`&$x`).
    /// Used by `array_walk` to decide if element mutations propagate. Only user
    /// closures and named user functions are inspected; anything else is false.
    fn callable_first_by_ref(&self, callee: &Zval) -> bool {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                Some(name) => self.named_first_by_ref(name.as_bytes()),
                None => self
                    .module
                    .closures
                    .get(cl.fn_idx)
                    .and_then(|f| f.param_by_ref.first())
                    .copied()
                    .unwrap_or(false),
            },
            Zval::Str(s) => self.named_first_by_ref(s.as_bytes()),
            Zval::Ref(c) => self.callable_first_by_ref(&c.borrow()),
            _ => false,
        }
    }

    /// First-parameter by-reference flag of a named user function (case-insensitive).
    fn named_first_by_ref(&self, name: &[u8]) -> bool {
        self.module
            .functions
            .iter()
            .find(|f| name_eq_ignore_case(&f.name, name))
            .and_then(|f| f.param_by_ref.first())
            .copied()
            .unwrap_or(false)
    }

    /// `usort(&$array, $callback)` (Session C): sort the array's values in place by
    /// the comparator, re-index `0..n`, and return `true`. The comparator returns
    /// an int (`$a <=> $b`-style). Mirrors `eval::ho_usort` — a stable merge sort,
    /// matching PHP 8's sort guarantee. Reads the array out of `slot` up front and
    /// writes the sorted result back, so no slot borrow is held across a callback.
    fn ho_usort(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cmp) = rest.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "usort() expects exactly 2 arguments, 1 given".to_string(),
            ));
        };
        let cmp = cmp.deref_clone();
        let top = self.frames.len() - 1;
        let values: Vec<Zval> = match self.frames[top].slots[slot as usize].deref_clone() {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "usort(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        let sorted = self.vm_merge_sort_with(&cmp, values)?;
        let mut out = PhpArray::new();
        for v in sorted {
            let _ = out.append(v);
        }
        let top = self.frames.len() - 1;
        self.frames[top].slots[slot as usize] = Zval::Array(Rc::new(out));
        Ok(Zval::Bool(true))
    }

    /// Stable merge sort driven by a PHP comparator callback (used by `usort`).
    /// The comparator's return value is cast to an int (`<= 0` keeps the left
    /// element first). Mirrors `eval::merge_sort_with`.
    fn vm_merge_sort_with(&mut self, cmp: &Zval, mut vals: Vec<Zval>) -> Result<Vec<Zval>, PhpError> {
        let n = vals.len();
        if n <= 1 {
            return Ok(vals);
        }
        let right = vals.split_off(n / 2);
        let left = self.vm_merge_sort_with(cmp, vals)?;
        let right = self.vm_merge_sort_with(cmp, right)?;
        let mut merged = Vec::with_capacity(n);
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            if self.compare_with_callback(cmp, &left[i], &right[j])? <= 0 {
                merged.push(left[i].clone());
                i += 1;
            } else {
                merged.push(right[j].clone());
                j += 1;
            }
        }
        merged.extend_from_slice(&left[i..]);
        merged.extend_from_slice(&right[j..]);
        Ok(merged)
    }

    /// Invoke a sort comparator and reduce its result to an int (`usort`).
    fn compare_with_callback(&mut self, cmp: &Zval, a: &Zval, b: &Zval) -> Result<i64, PhpError> {
        let r = self.call_callable(cmp.clone(), vec![a.clone(), b.clone()])?;
        Ok(convert::to_long_cast(&r, &mut self.diags))
    }

    /// `is_callable($value)`: a closure / FCC, a string naming a function or
    /// `Class::method`, a `[target, method]` array, or an object with `__invoke`
    /// (mirrors `eval::ho_is_callable`; does not invoke the callable).
    fn ho_is_callable(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(v) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "is_callable() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        Ok(Zval::Bool(self.is_value_callable(&v.deref_clone())))
    }

    /// Whether `v` is callable (the predicate behind `is_callable`), without
    /// invoking it.
    fn is_value_callable(&self, v: &Zval) -> bool {
        match v {
            Zval::Closure(_) => true,
            Zval::Str(s) => {
                let b = s.as_bytes();
                if let Some(pos) = b.windows(2).position(|w| w == b"::") {
                    self.class_id_from_value(&Zval::Str(PhpStr::new(b[..pos].to_vec())))
                        .map(|cid| resolve_method_runtime(self.module, cid, &b[pos + 2..]).is_some())
                        .unwrap_or(false)
                } else {
                    self.is_name_callable(b)
                }
            }
            Zval::Array(a) => {
                let elems: Vec<Zval> = a.iter().map(|(_, v)| v.deref_clone()).collect();
                if elems.len() != 2 {
                    return false;
                }
                let Zval::Str(m) = &elems[1] else { return false };
                match self.class_id_from_value(&elems[0]) {
                    Some(cid) => resolve_method_runtime(self.module, cid, m.as_bytes()).is_some(),
                    None => false,
                }
            }
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                resolve_method_runtime(self.module, cid, b"__invoke").is_some()
            }
            Zval::Ref(r) => self.is_value_callable(&r.borrow()),
            _ => false,
        }
    }

    /// Whether a bare name is callable: a user function, any registry builtin, or a
    /// host builtin (mirrors `eval::is_name_callable`).
    fn is_name_callable(&self, name: &[u8]) -> bool {
        self.module.functions.iter().any(|f| name_eq_ignore_case(&f.name, name))
            || self.registry.get(name).is_some()
            || host_builtin_canonical(name).is_some()
            || host_builtin_ref_first(name).is_some()
    }

    /// Install a frame for an anonymous closure: bind its captured variables into
    /// their slots, then the call arguments into the leading parameter slots, and
    /// the bound `$this`. Mirrors `eval::call_closure` (captures before params).
    fn push_closure_frame(&mut self, cl: &Closure, args: Vec<Zval>) -> Result<(), PhpError> {
        let callee = &self.module.closures[cl.fn_idx];
        let mut frame = Frame::new(callee);
        for (slot, val) in &cl.captures {
            frame.slots[*slot as usize] = val.clone();
        }
        bind_params(&mut frame, args);
        frame.this = cl.bound_this.clone();
        self.enter_callee(frame)
    }

    /// Like [`Self::push_magic_call`] but the forwarded `$args` array also carries
    /// any **named arguments** keyed by name (string keys), matching PHP's `__call`
    /// behaviour for `$obj->missing(x: 1)` (Session A).
    #[allow(clippy::too_many_arguments)]
    fn push_magic_call_named(
        &mut self,
        defc: ClassId,
        midx: usize,
        this: Option<Zval>,
        static_class: ClassId,
        method: &[u8],
        positional: Vec<Zval>,
        named: Vec<(Box<[u8]>, Zval)>,
    ) {
        let callee = &self.module.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee);
        frame.argc = callee.n_params;
        if !frame.slots.is_empty() {
            frame.slots[0] = Zval::Str(PhpStr::new(method.to_vec()));
        }
        if frame.slots.len() > 1 {
            let mut arr = PhpArray::new();
            for a in positional {
                let _ = arr.append(a);
            }
            for (name, val) in named {
                arr.insert(Key::Str(PhpStr::new(name.to_vec())), val);
            }
            frame.slots[1] = Zval::Array(Rc::new(arr));
        }
        frame.this = this;
        frame.class = Some(defc);
        frame.static_class = Some(static_class);
        self.frames.push(frame);
    }

    /// undefined-method error. Shared by `Op::MethodCall`.
    fn dispatch_instance_call(
        &mut self,
        top: usize,
        cid: ClassId,
        this: Zval,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<(), PhpError> {
        let module = self.module;
        let resolved = resolve_method_runtime(module, cid, method);
        // Usable only if found *and* visible from the caller's scope.
        let usable = resolved.filter(|&(defc, midx)| {
            visible_from(module, self.frames[top].class, module.classes[defc].methods[midx].visibility, defc)
        });
        match usable {
            Some((defc, midx)) => {
                let callee = &module.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee);
                bind_params(&mut frame, args);
                frame.this = Some(this);
                frame.class = Some(defc);
                frame.static_class = Some(cid); // LSB = receiver's actual class
                self.enter_callee(frame)?;
            }
            // Missing or inaccessible: route to `__call` if defined, else the
            // original fatal (visibility / undefined method).
            None => match resolve_method_runtime(module, cid, b"__call") {
                Some((cdefc, cmidx)) => {
                    self.push_magic_call(cdefc, cmidx, Some(this), cid, method, args);
                }
                None => {
                    return Err(match resolved {
                        Some((defc, midx)) => method_access_error(
                            module,
                            defc,
                            method,
                            self.frames[top].class,
                            module.classes[defc].methods[midx].visibility,
                        ),
                        None => undefined_method(module, cid, method),
                    })
                }
            },
        }
        Ok(())
    }

    /// Dispatch a static method call `start::method(args)` whose starting class
    /// `start` is already resolved (OOP-2a). `forwarding` is true for
    /// `self`/`parent`/`static` (keep the caller's LSB class and `$this`), false
    /// for a named/dynamic class (rebind LSB; forward `$this` only when the
    /// receiver is in `start`'s hierarchy). A missing or inaccessible target
    /// routes to `__call` on `$this` (in object context) or `__callStatic`,
    /// otherwise raises the visibility / undefined-method error. Shared by
    /// `Op::StaticCall` (and, later, the dynamic `$cls::method()` path).
    fn dispatch_static_call(
        &mut self,
        top: usize,
        start: ClassId,
        method: &[u8],
        forwarding: bool,
        args: Vec<Zval>,
    ) -> Result<(), PhpError> {
        let module = self.module;
        // Enum built-in statics (`cases` / `from` / `tryFrom`) are reserved names
        // that shadow user resolution and produce a value directly rather than
        // entering a frame (step 23). `cases` is on every enum; `from`/`tryFrom`
        // only on a backed one.
        if !module.classes[start].enum_cases.is_empty() {
            if method.eq_ignore_ascii_case(b"cases") {
                let v = self.vm_enum_cases(start);
                self.frames[top].stack.push(v);
                return Ok(());
            }
            let backed = module.classes[start].enum_cases.iter().any(|c| c.value.is_some());
            if backed {
                let try_from = method.eq_ignore_ascii_case(b"tryFrom");
                if try_from || method.eq_ignore_ascii_case(b"from") {
                    let arg = args.into_iter().next();
                    let v = self.vm_enum_from(start, arg, try_from)?;
                    self.frames[top].stack.push(v);
                    return Ok(());
                }
            }
        }
        let resolved = resolve_method_runtime(module, start, method);
        let usable = resolved.filter(|&(defc, midx)| {
            visible_from(module, self.frames[top].class, module.classes[defc].methods[midx].visibility, defc)
        });
        // LSB: a forwarding call keeps the caller's; a named/dynamic call rebinds.
        let static_class = if forwarding {
            self.frames[top].static_class.unwrap_or(start)
        } else {
            start
        };
        // `$this` is forwarded for a forwarding call, or for a named/dynamic call
        // to a class in the current object's hierarchy.
        let this = match &self.frames[top].this {
            Some(t) => {
                let keep = forwarding
                    || matches!(object_class_id(t), Some(ocid) if class_is_a(module, ocid, start));
                if keep {
                    Some(t.clone())
                } else {
                    None
                }
            }
            None => None,
        };
        match usable {
            Some((defc, midx)) => {
                let callee = &module.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee);
                bind_params(&mut frame, args);
                frame.this = this;
                frame.class = Some(defc);
                frame.static_class = Some(static_class);
                self.enter_callee(frame)?;
            }
            None => {
                // In object context (a `$this` in the hierarchy) a missing /
                // inaccessible static target routes to `__call` on `$this`;
                // otherwise to `__callStatic` on the class.
                let via_call = this
                    .as_ref()
                    .and_then(|t| object_class_id(t).map(|oc| (t.clone(), oc)))
                    .and_then(|(tv, oc)| {
                        resolve_method_runtime(module, oc, b"__call").map(|(d, m)| (tv, oc, d, m))
                    });
                if let Some((tv, oc, cdefc, cmidx)) = via_call {
                    self.push_magic_call(cdefc, cmidx, Some(tv), oc, method, args);
                } else if let Some((cdefc, cmidx)) =
                    resolve_method_runtime(module, start, b"__callStatic")
                {
                    self.push_magic_call(cdefc, cmidx, None, start, method, args);
                } else {
                    return Err(match resolved {
                        Some((defc, midx)) => method_access_error(
                            module,
                            defc,
                            method,
                            self.frames[top].class,
                            module.classes[defc].methods[midx].visibility,
                        ),
                        None => undefined_method(module, start, method),
                    });
                }
            }
        }
        Ok(())
    }

    /// Like [`Self::class_id_from_value`] but for contexts that must error rather
    /// than yield `false` (`new $cls`, `$cls::m()`): an unknown name is a
    /// catchable `Error` ("Class \"X\" not found") and a non-string/object is an
    /// `Error` ("Class name must be a valid object or a string").
    fn resolve_dynamic_class(&self, v: &Zval) -> Result<ClassId, PhpError> {
        match v.deref_clone() {
            Zval::Object(o) => Ok(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                self.module.class_index.get(&name.to_ascii_lowercase()).copied().ok_or_else(|| {
                    PhpError::Error(format!(
                        "Class \"{}\" not found",
                        String::from_utf8_lossy(name)
                    ))
                })
            }
            _ => Err(PhpError::Error(
                "Class name must be a valid object or a string".to_string(),
            )),
        }
    }

    /// Build a fresh instance of class `cid`: its declared property defaults
    /// materialised, a fresh handle id, shared class-name / visibility metadata.
    /// Fatal if the class is non-instantiable (abstract / interface / enum) or
    /// could not be compiled. Shared by [`Op::Alloc`] and [`Op::AllocStatic`].
    fn alloc_object(&mut self, cid: ClassId) -> Result<Zval, PhpError> {
        let module = self.module; // &'m Module: detach from `self` borrow
        let cc = &module.classes[cid];
        match cc.instantiable {
            Instantiable::Yes => {}
            Instantiable::Abstract => {
                return Err(PhpError::Error(format!(
                    "Cannot instantiate abstract class {}",
                    String::from_utf8_lossy(&cc.name)
                )))
            }
            Instantiable::Interface => {
                return Err(PhpError::Error(format!(
                    "Cannot instantiate interface {}",
                    String::from_utf8_lossy(&cc.name)
                )))
            }
            Instantiable::Enum => {
                return Err(PhpError::Error(format!(
                    "Cannot instantiate enum {}",
                    String::from_utf8_lossy(&cc.name)
                )))
            }
        }
        if !cc.ok {
            return Err(PhpError::Error(format!(
                "VM: cannot instantiate {} (non-constant property default not yet ported)",
                String::from_utf8_lossy(&cc.name)
            )));
        }
        let mut props = Props::new();
        for (name, c) in &cc.prop_defaults {
            props.set(name, c.to_zval());
        }
        let class_name = Rc::clone(&cc.class_name);
        let info = Rc::clone(&cc.info);
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info };
        let rc = Rc::new(RefCell::new(obj));
        // Track for `__destruct` (OOP-3d): the extra strong ref drives the sweep.
        self.created.push(Rc::clone(&rc));
        Ok(Zval::Object(rc))
    }

    /// Return the interned singleton object for enum `class`'s `case`-th case,
    /// materialising it on first use (Session A; mirrors `eval::eval_enum_case`).
    /// It carries a read-only `name` property and, for a backed enum, a `value`
    /// property; the object holds the enum's class id so the whole OOP machinery
    /// (`instanceof`, method dispatch, `$this`) applies. Cached so `E::Case` is the
    /// same handle every time (identity `===`). Singletons are *not* tracked for
    /// `__destruct` — they live for the whole run.
    fn enum_case(&mut self, class: ClassId, case: u32) -> Rc<RefCell<Object>> {
        if let Some(o) = self.enum_cache.get(&(class, case)) {
            return Rc::clone(o);
        }
        let cc = &self.module.classes[class];
        let decl = &cc.enum_cases[case as usize];
        let mut props = Props::new();
        let mut entries: Vec<(Box<[u8]>, PropVis)> = vec![(Box::from(&b"name"[..]), PropVis::Public)];
        props.set(b"name", Zval::Str(PhpStr::new(decl.name.to_vec())));
        if let Some(v) = &decl.value {
            props.set(b"value", v.to_zval());
            entries.push((Box::from(&b"value"[..]), PropVis::Public));
        }
        let id = self.next_id();
        let obj = Object {
            class_id: class as u32,
            class_name: Rc::clone(&cc.class_name),
            props,
            id,
            info: Rc::new(ObjectInfo::enum_case(entries)),
        };
        let rc = Rc::new(RefCell::new(obj));
        self.enum_cache.insert((class, case), Rc::clone(&rc));
        rc
    }

    /// `E::cases()` (step 23): a sequential array of every case singleton in
    /// declaration order. Works on pure and backed enums alike. Mirrors
    /// `eval::enum_cases`.
    fn vm_enum_cases(&mut self, cid: ClassId) -> Zval {
        let n = self.module.classes[cid].enum_cases.len();
        let mut arr = PhpArray::new();
        for i in 0..n {
            let case = self.enum_case(cid, i as u32);
            let _ = arr.append(Zval::Object(case));
        }
        Zval::Array(Rc::new(arr))
    }

    /// `BackedEnum::from($v)` / `tryFrom($v)` (step 23): return the singleton whose
    /// backing `value` is identical (`===`) to `$v`. `from` raises a catchable
    /// `ValueError` on no match; `tryFrom` returns `null`. Mirrors `eval::enum_from`.
    fn vm_enum_from(
        &mut self,
        cid: ClassId,
        arg: Option<Zval>,
        try_from: bool,
    ) -> Result<Zval, PhpError> {
        let arg = arg.unwrap_or(Zval::Null).deref_clone();
        let n = self.module.classes[cid].enum_cases.len();
        for i in 0..n {
            let case = self.enum_case(cid, i as u32);
            let hit = case
                .borrow()
                .props
                .get(b"value")
                .is_some_and(|v| ops::identical(v, &arg));
            if hit {
                return Ok(Zval::Object(case));
            }
        }
        if try_from {
            return Ok(Zval::Null);
        }
        // PHP quotes a string backing value but not an integer one.
        let repr = match &arg {
            Zval::Str(s) => format!("\"{}\"", String::from_utf8_lossy(s.as_bytes())),
            Zval::Long(l) => l.to_string(),
            other => {
                let z = convert::to_zstr(other, &mut self.diags);
                String::from_utf8_lossy(z.as_bytes()).into_owned()
            }
        };
        Err(PhpError::ValueError(format!(
            "{repr} is not a valid backing value for enum {}",
            String::from_utf8_lossy(&self.module.classes[cid].name)
        )))
    }

    /// Expand a spread source into its `(key, value)` pairs for a call's argument
    /// unpacking (PAR): an array yields its entries verbatim; a generator is driven
    /// to completion, honouring its yielded keys (so string keys become named
    /// arguments). Any non-iterable is the PHP `TypeError`. Mirrors the
    /// array-merge logic of `Op::ArrayAppendSpread`.
    fn spread_pairs(&mut self, src: Zval) -> Result<Vec<(Key, Zval)>, PhpError> {
        match src.deref_clone() {
            Zval::Array(s) => Ok(s.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect()),
            Zval::Generator(rc) => {
                let mut out = Vec::new();
                self.ensure_started(&rc)?;
                loop {
                    let (k, v, done) = {
                        let g = rc.borrow();
                        (g.cur_key.clone(), g.cur_val.clone(), matches!(g.status, GenStatus::Done))
                    };
                    if done {
                        break;
                    }
                    let key = coerce_key_silent(&k).unwrap_or(Key::Int(0));
                    out.push((key, v));
                    self.resume_generator(&rc, Zval::Null)?;
                }
                Ok(out)
            }
            other => Err(PhpError::TypeError(format!(
                "Only arrays and Traversables can be unpacked, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// The source line of the instruction that just ran (or faulted) in frame
    /// `top`: `lines[ip-1]`, since the dispatch loop has already advanced `ip`
    /// past it. Defensive: returns 0 if the table is short or `ip` is 0 (EXC-3b).
    fn cur_line(&self, top: usize) -> Line {
        let f = &self.frames[top];
        f.ip.checked_sub(1).and_then(|i| f.func.lines.get(i).copied()).unwrap_or(0)
    }

    /// PHP fixes a Throwable's `line`/`file` at `new` time (not in the user
    /// constructor). After allocating an object whose class is-a `Throwable`,
    /// stamp the current source location onto it before `__construct` runs
    /// (EXC-3b). A no-op for non-Throwable classes.
    fn stamp_throwable_location(&self, obj: &Zval) {
        let Some(throwable_id) = self.throwable_id else { return };
        let Zval::Object(o) = obj else { return };
        let cid = o.borrow().class_id as ClassId;
        if !is_instance_of(self.module, cid, throwable_id) {
            return;
        }
        let line = self.cur_line(self.frames.len() - 1);
        let (trace, trace_string) = self.capture_trace();
        let mut b = o.borrow_mut();
        b.props.set(b"line", Zval::Long(line as i64));
        b.props
            .set(b"file", Zval::Str(PhpStr::new(self.module.file.to_vec())));
        b.props.set(b"trace", trace);
        b.props
            .set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
    }

    /// Snapshot the running frame stack as a Throwable's `(trace array, trace
    /// string)` (EXC-3c), mirroring `eval::capture_trace`. Frames are
    /// innermost-first, excluding the script body (`main`), and the string ends
    /// with `#N {main}`. Each entry's `line` is the call-site line in the
    /// *caller* (frame `k` was entered from frame `k-1`), recovered from the
    /// per-op line table (EXC-3b); `args` is empty, as the tree-walker leaves it.
    fn capture_trace(&self) -> (Zval, Vec<u8>) {
        let file = &self.module.file;
        let mut arr = PhpArray::new();
        let mut s: Vec<u8> = Vec::new();
        let n = self.frames.len();
        for (i, k) in (1..n).rev().enumerate() {
            let frame = &self.frames[k];
            let line = self.cur_line(k - 1) as i64;
            // The class shown is the late-static-binding (called) class, like the
            // tree-walker; absent for a free-function frame.
            let class: Option<&[u8]> = frame
                .static_class
                .map(|cid| self.module.classes[cid].name.as_ref());
            let is_static = frame.this.is_none();

            s.extend_from_slice(format!("#{i} ").as_bytes());
            s.extend_from_slice(file);
            s.extend_from_slice(format!("({line}): ").as_bytes());
            if let Some(c) = class {
                s.extend_from_slice(c);
                s.extend_from_slice(if is_static { b"::" } else { b"->" });
            }
            s.extend_from_slice(&frame.func.name);
            s.extend_from_slice(b"()\n");

            let mut fr = PhpArray::new();
            fr.insert(Key::from_bytes(b"file"), Zval::Str(PhpStr::new(file.to_vec())));
            fr.insert(Key::from_bytes(b"line"), Zval::Long(line));
            fr.insert(
                Key::from_bytes(b"function"),
                Zval::Str(PhpStr::new(frame.func.name.to_vec())),
            );
            if let Some(c) = class {
                fr.insert(Key::from_bytes(b"class"), Zval::Str(PhpStr::new(c.to_vec())));
                let ty: &[u8] = if is_static { b"::" } else { b"->" };
                fr.insert(Key::from_bytes(b"type"), Zval::Str(PhpStr::new(ty.to_vec())));
            }
            fr.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(PhpArray::new())));
            let _ = arr.append(Zval::Array(Rc::new(fr)));
        }
        s.extend_from_slice(format!("#{} {{main}}", n - 1).as_bytes());
        (Zval::Array(Rc::new(arr)), s)
    }

    /// Resolve and lazily initialise the persistent cell for static property
    /// `target::$name`, enforcing visibility against the running frame's class.
    /// Returns `Some(cell)` when ready, or `None` when a non-constant default's
    /// init thunk was just scheduled — in that case the caller has had its `ip`
    /// rewound and must `continue` so the access re-runs once the cell is filled.
    fn ensure_static(
        &mut self,
        target: ClassTarget,
        name: &[u8],
        top: usize,
        ip: usize,
    ) -> Result<Option<Rc<RefCell<Zval>>>, PhpError> {
        let module = self.module;
        let start = match target {
            ClassTarget::Class(cid) => cid,
            ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                PhpError::Error("Cannot use \"static\" outside class context".to_string())
            })?,
        };
        let Some((decl, idx)) = find_static_prop(module, start, name) else {
            return Err(PhpError::Error(format!(
                "Access to undeclared static property {}::${}",
                String::from_utf8_lossy(&module.classes[start].name),
                String::from_utf8_lossy(name)
            )));
        };
        let entry = &module.classes[decl].static_props[idx];
        if !visible_from(module, self.frames[top].class, entry.visibility, decl) {
            return Err(prop_access_error(module, decl, name, entry.visibility));
        }
        let key = (decl, name.to_vec());
        if let Some(cell) = self.static_props.get(&key) {
            return Ok(Some(Rc::clone(cell)));
        }
        match &entry.init {
            StaticInit::Const(c) => {
                let cell = Rc::new(RefCell::new(c.to_zval()));
                self.static_props.insert(key, Rc::clone(&cell));
                Ok(Some(cell))
            }
            StaticInit::Thunk(func) => {
                // Insert a placeholder cell now, run the thunk into it, and rewind
                // so the access re-reads the filled cell on the next iteration.
                let cell = Rc::new(RefCell::new(Zval::Null));
                self.static_props.insert(key, Rc::clone(&cell));
                let mut frame = Frame::new(func);
                frame.class = Some(decl);
                frame.static_class = Some(decl);
                frame.ret_cell = Some(Rc::clone(&cell));
                self.frames[top].ip = ip;
                self.frames.push(frame);
                Ok(None)
            }
        }
    }

    /// Pop the operand-stack keys for a field path's `Index` steps (one per
    /// `Index`), restoring source order.
    fn pop_field_keys(&mut self, top: usize, steps: &[FieldStep]) -> Vec<Zval> {
        let n = steps.iter().filter(|s| matches!(s, FieldStep::Index)).count();
        let mut keys: Vec<Zval> =
            (0..n).map(|_| self.frames[top].stack.pop().expect("field index key")).collect();
        keys.reverse();
        keys
    }

    /// Read a mixed field path's value (silent; `None` if any level is absent).
    fn field_value(&self, base: FieldBase, top: usize, steps: &[FieldStep], keys: Vec<Zval>) -> Option<Zval> {
        let cell = match base {
            FieldBase::Local(s) => &self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &self.frames[0].slots[s as usize],
            FieldBase::This => self.frames[top].this.as_ref()?,
        };
        field_get(cell, steps, &mut keys.into_iter())
    }

    /// Remove a mixed field path's leaf (silent no-op if absent).
    fn field_remove(&mut self, base: FieldBase, top: usize, steps: &[FieldStep], keys: Vec<Zval>) {
        let cell = match base {
            FieldBase::Local(s) => &mut self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &mut self.frames[0].slots[s as usize],
            FieldBase::This => match self.frames[top].this.as_mut() {
                Some(c) => c,
                None => return,
            },
        };
        field_unset(cell, steps, &mut keys.into_iter());
    }

    /// Push a `__call` / `__callStatic` magic-dispatch frame (OOP-3a): the magic
    /// method receives the original method name and a 0-indexed array of the
    /// arguments. Its `Ret` leaves the result on the caller's operand stack, like
    /// any method call.
    fn push_magic_call(
        &mut self,
        defc: ClassId,
        midx: usize,
        this: Option<Zval>,
        static_class: ClassId,
        method: &[u8],
        args: Vec<Zval>,
    ) {
        let callee = &self.module.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee);
        // A magic accessor is always invoked with exactly its declared
        // parameters, so the arity guard (PAR) sees a full argument count.
        frame.argc = callee.n_params;
        if !frame.slots.is_empty() {
            frame.slots[0] = Zval::Str(PhpStr::new(method.to_vec()));
        }
        if frame.slots.len() > 1 {
            frame.slots[1] = pack_args(args);
        }
        frame.this = this;
        frame.class = Some(defc);
        frame.static_class = Some(static_class);
        self.frames.push(frame);
    }

    /// Push a magic property-accessor frame (`__get`/`__set`/`__isset`/`__unset`),
    /// binding the property name (and, for `__set`, the value) and registering the
    /// recursion guard (released on the frame's `Ret`). `ret_cell` discards the
    /// return (`__set`/`__unset`); `ret_bool` coerces it to bool (`__isset`).
    #[allow(clippy::too_many_arguments)]
    fn push_magic_prop(
        &mut self,
        defc: ClassId,
        midx: usize,
        oid: u32,
        kind: MagicKind,
        recv: Zval,
        name: &[u8],
        extra: Option<Zval>,
        ret_cell: Option<Rc<RefCell<Zval>>>,
        ret_bool: bool,
    ) {
        let lsb = object_class_id(&recv).unwrap_or(defc);
        let callee = &self.module.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee);
        // A magic accessor is always invoked with exactly its declared
        // parameters, so the arity guard (PAR) sees a full argument count.
        frame.argc = callee.n_params;
        if !frame.slots.is_empty() {
            frame.slots[0] = Zval::Str(PhpStr::new(name.to_vec()));
        }
        if let Some(v) = extra {
            if frame.slots.len() > 1 {
                frame.slots[1] = v;
            }
        }
        frame.this = Some(recv);
        frame.class = Some(defc);
        frame.static_class = Some(lsb);
        frame.ret_cell = ret_cell;
        frame.ret_bool = ret_bool;
        let key = (oid, kind, name.to_vec());
        self.magic_guard.insert(key.clone());
        frame.guard_release = Some(key);
        self.frames.push(frame);
    }

}

/// The leaf operation of a path write, carried to the bottom of the drill-down.
enum Last {
    Set { key: Zval, value: Zval },
    Append { value: Zval },
    OpSet { key: Zval, op: BinOp, rhs: Zval },
    IncDec { key: Zval, inc: bool, pre: bool },
}

/// If `name` (ASCII-case-insensitive, PHP function names) is an *evaluator-only
/// host builtin* the VM dispatches itself — a higher-order builtin that invokes a
/// user callable, class introspection, or the `define` family (Sessions B–D) —
/// return its canonical lowercased name; otherwise `None`. The compiler calls this
/// to decide whether to emit [`Op::CallHostBuiltin`]; the VM's
/// [`Vm::dispatch_host_builtin`] matches on the same canonical name. The two MUST
/// agree — a name emitted here but unmatched there is a clean runtime error.
/// The severity label PHP prints for an E_* number in the default render
/// (`main/main.c`, `error_type_to_string`). Covers the built-in diagnostics
/// (`E_WARNING`/`E_NOTICE`/`E_DEPRECATED`) and the `trigger_error` user levels
/// (`E_USER_*`), which share the same three labels.
fn errno_label(errno: i64) -> &'static str {
    match errno {
        2 | 512 => "Warning",        // E_WARNING / E_USER_WARNING
        8192 | 16384 => "Deprecated", // E_DEPRECATED / E_USER_DEPRECATED
        _ => "Notice",                // E_NOTICE (8) / E_USER_NOTICE (1024)
    }
}

pub(crate) fn host_builtin_canonical(name: &[u8]) -> Option<&'static [u8]> {
    // B1: the call-a-callable family. B3: the define family. Sessions C/D grow
    // this list (array_map, usort, sprintf, get_class, …).
    const HOST: &[&[u8]] = &[
        b"call_user_func",
        b"call_user_func_array",
        b"is_callable",
        b"define",
        b"defined",
        b"constant",
        b"array_map",
        b"array_filter",
        b"array_reduce",
        b"get_class",
        b"get_parent_class",
        b"get_object_vars",
        b"get_class_methods",
        b"func_num_args",
        b"func_get_args",
        b"func_get_arg",
        b"sprintf",
        b"printf",
        b"vsprintf",
        b"vprintf",
        b"fprintf",
        b"vfprintf",
        b"function_exists",
        b"class_exists",
        b"interface_exists",
        b"method_exists",
        b"property_exists",
        b"get_called_class",
        b"error_reporting",
        b"trigger_error",
        b"user_error",
        b"error_get_last",
        b"set_exception_handler",
        b"restore_exception_handler",
        b"set_error_handler",
        b"restore_error_handler",
        b"unserialize",
        b"fopen",
        b"tmpfile",
        b"opendir",
        b"preg_replace",
        b"preg_quote",
        b"preg_split",
        b"debug_backtrace",
        b"debug_print_backtrace",
        b"preg_replace_callback",
    ];
    HOST.iter().copied().find(|h| name.eq_ignore_ascii_case(h))
}

/// Like [`host_builtin_canonical`] but for the *by-reference-first* host builtins
/// (Session C): their first argument is an array variable taken by reference. The
/// compiler emits [`crate::bytecode::Op::CallHostBuiltinRef`] (with the variable's
/// slot) for these; [`Vm::dispatch_host_builtin_ref`] matches the same canonical
/// name. The two lists are disjoint.
/// One reconstructed call-stack entry (see [`Vm::collect_backtrace`]).
struct BtFrame {
    function: Vec<u8>,
    line: Line,
    class: Option<Vec<u8>>,
    is_static: bool,
    object: Option<Zval>,
    args: Vec<Zval>,
}

/// Format one argument the way `debug_print_backtrace` does: scalars literal,
/// a string single-quoted and truncated to 15 bytes + `...`, arrays as `Array`,
/// objects/closures/generators as `Object(Class)`, resources as `Resource id #N`.
fn format_bt_arg(v: &Zval) -> String {
    match v {
        Zval::Undef | Zval::Null => "NULL".to_string(),
        Zval::Bool(true) => "true".to_string(),
        Zval::Bool(false) => "false".to_string(),
        Zval::Long(n) => n.to_string(),
        Zval::Double(d) => String::from_utf8_lossy(&php_types::dtoa::double_to_precision(*d, 14)).into_owned(),
        Zval::Str(s) => {
            let b = s.as_bytes();
            let shown = if b.len() > 15 {
                format!("{}...", String::from_utf8_lossy(&b[..15]))
            } else {
                String::from_utf8_lossy(b).into_owned()
            };
            format!("'{shown}'")
        }
        Zval::Array(_) => "Array".to_string(),
        Zval::Object(o) => format!("Object({})", String::from_utf8_lossy(o.borrow().class_name.as_bytes())),
        Zval::Closure(_) => "Object(Closure)".to_string(),
        Zval::Generator(_) => "Object(Generator)".to_string(),
        Zval::Resource(r) => format!("Resource id #{}", r.borrow().id),
        Zval::Ref(rc) => format_bt_arg(&rc.borrow()),
    }
}

/// One array internal-pointer operation (see [`Vm::ho_array_pointer`]).
#[derive(Clone, Copy)]
enum PtrOp {
    Current,
    Key,
    Reset,
    End,
    Next,
    Prev,
}

impl PtrOp {
    fn name(self) -> &'static str {
        match self {
            PtrOp::Current => "current",
            PtrOp::Key => "key",
            PtrOp::Reset => "reset",
            PtrOp::End => "end",
            PtrOp::Next => "next",
            PtrOp::Prev => "prev",
        }
    }
}

/// Apply an internal-pointer op to `target` (the dereferenced array slot). Reads
/// (`current`/`key`) take `&PhpArray`; the movers COW via `Rc::make_mut` since they
/// mutate the cursor. Non-array → the PHP `TypeError`.
fn array_pointer_apply(target: &mut Zval, op: PtrOp) -> Result<Zval, PhpError> {
    let Zval::Array(rc) = target else {
        return Err(PhpError::TypeError(format!(
            "{}(): Argument #1 ($array) must be of type array, {} given",
            op.name(),
            target.error_type_name()
        )));
    };
    Ok(match op {
        PtrOp::Current => rc.ptr_current().unwrap_or(Zval::Bool(false)),
        PtrOp::Key => rc.ptr_key().map(|k| key_to_zval(&k)).unwrap_or(Zval::Null),
        PtrOp::Reset => Rc::make_mut(rc).ptr_reset().unwrap_or(Zval::Bool(false)),
        PtrOp::End => Rc::make_mut(rc).ptr_end().unwrap_or(Zval::Bool(false)),
        PtrOp::Next => Rc::make_mut(rc).ptr_next().unwrap_or(Zval::Bool(false)),
        PtrOp::Prev => Rc::make_mut(rc).ptr_prev().unwrap_or(Zval::Bool(false)),
    })
}

/// Host builtins with a by-reference **output** parameter, mapping the canonical
/// name to the argument index of that out-param. `preg_match`/`preg_match_all`
/// write their captures into `&$matches` at index 2. The compiler emits
/// [`crate::bytecode::Op::CallHostBuiltinOut`] for these; [`Vm::dispatch_host_builtin_out`]
/// produces `(result, out_value)` and the VM writes the out-value into the slot.
pub(crate) fn host_builtin_out_param(name: &[u8]) -> Option<(&'static [u8], usize)> {
    const HOST_OUT: &[(&[u8], usize)] = &[(b"preg_match", 2), (b"preg_match_all", 2)];
    HOST_OUT
        .iter()
        .find(|(h, _)| name.eq_ignore_ascii_case(h))
        .map(|&(h, i)| (h, i))
}

pub(crate) fn host_builtin_ref_first(name: &[u8]) -> Option<&'static [u8]> {
    const HOST_REF: &[&[u8]] = &[
        b"usort",
        b"array_walk",
        // Array internal-pointer family (Session: array-pointer). Each takes the
        // array by reference (mutating/reading its internal cursor).
        b"reset",
        b"end",
        b"next",
        b"prev",
        b"current",
        b"key",
    ];
    HOST_REF.iter().copied().find(|h| name.eq_ignore_ascii_case(h))
}

/// ASCII-case-insensitive byte-string equality — PHP resolves function names
/// case-insensitively in ASCII (mirrors the compiler's resolution).
fn name_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Drill through `keys` from `cell` (following references, auto-vivifying and
/// copy-on-writing each level), then apply `last` at the leaf. Recursion (not a
/// reassigned `&mut` in a loop) keeps the nested borrows well-formed.
fn path_apply(cell: &mut Zval, keys: &[Zval], last: Last, diags: &mut Diags) -> Result<Zval, PhpError> {
    if let Zval::Ref(rc) = cell {
        let mut inner = rc.borrow_mut();
        return path_apply(&mut inner, keys, last, diags);
    }
    match keys.split_first() {
        Some((k, rest)) => {
            ensure_array(cell)?;
            let Zval::Array(rc) = cell else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let key = coerce_key_silent(k)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if !arr.contains_key(&key) {
                arr.insert(key.clone(), Zval::Null);
            }
            let child = arr.get_mut(&key).expect("just inserted");
            path_apply(child, rest, last, diags)
        }
        None => apply_last(cell, last, diags),
    }
}

/// Apply the leaf step to the parent cell (which must hold the target array).
fn apply_last(parent: &mut Zval, last: Last, diags: &mut Diags) -> Result<Zval, PhpError> {
    ensure_array(parent)?;
    let Zval::Array(rc) = parent else { unreachable!("ensured array") };
    let arr = Rc::make_mut(rc);
    match last {
        Last::Set { key, value } => {
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            // Write *through* an existing reference element (REF-4) so an alias
            // sees the update; otherwise overwrite / insert.
            match arr.get_mut(&k) {
                Some(slot) => store_slot(slot, value.clone()),
                None => arr.insert(k, value.clone()),
            }
            Ok(value)
        }
        Last::Append { value } => {
            arr.append(value.clone()).map_err(|_| {
                PhpError::Error(
                    "Cannot add element to the array as the next element is already occupied"
                        .to_string(),
                )
            })?;
            Ok(value)
        }
        Last::OpSet { key, op, rhs } => {
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            let old = arr.get(&k).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
            let result = apply_binop(op, &old, &rhs, diags)?;
            // Write through an existing reference element (REF-4).
            match arr.get_mut(&k) {
                Some(slot) => store_slot(slot, result.clone()),
                None => arr.insert(k, result.clone()),
            }
            Ok(result)
        }
        Last::IncDec { key, inc, pre } => {
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if !arr.contains_key(&k) {
                arr.insert(k.clone(), Zval::Null);
            }
            // Operate through a reference element (REF-4) so an alias updates too.
            let slot = arr.get_mut(&k).expect("just inserted");
            let cell = if let Zval::Ref(rc) = slot {
                Rc::clone(rc)
            } else {
                let old = slot.clone();
                if inc {
                    ops::increment(slot, diags)?;
                } else {
                    ops::decrement(slot, diags)?;
                }
                return Ok(if pre { slot.clone() } else { old });
            };
            let mut inner = cell.borrow_mut();
            let old = inner.clone();
            if inc {
                ops::increment(&mut inner, diags)?;
            } else {
                ops::decrement(&mut inner, diags)?;
            }
            Ok(if pre { inner.clone() } else { old })
        }
    }
}

/// The mutable cell a [`DimBase`] addresses: a slot in the current frame
/// (`Local`) or in the global/script frame (`Global`). Mirrors the inline match
/// `Op::UnsetPath` uses; factored out for the REF-1 `BindRef` arm.
fn ref_base_mut<'f>(frames: &'f mut [Frame<'_>], top: usize, base: DimBase) -> &'f mut Zval {
    match base {
        DimBase::Local(s) => &mut frames[top].slots[s as usize],
        DimBase::Global(s) => &mut frames[0].slots[s as usize],
    }
}

/// Promote a cell to a shared [`Zval::Ref`], returning the shared cell. An
/// already-shared cell is returned as-is; an `Undef` is promoted to a defined
/// `Null` (a later write through the alias then has a real cell to land in).
/// Mirrors `eval::make_cell` (the step-11d reference machinery, D-R3 / D-12.4).
fn make_cell(cell: &mut Zval) -> Rc<RefCell<Zval>> {
    if let Zval::Ref(rc) = cell {
        return Rc::clone(rc);
    }
    let init = match &*cell {
        Zval::Undef => Zval::Null,
        other => other.clone(),
    };
    let rc = Rc::new(RefCell::new(init));
    *cell = Zval::Ref(Rc::clone(&rc));
    rc
}

/// The keys of the array a `foreach … as &$v` iterates, snapshotted once at loop
/// entry (REF-3). A reference is followed; a non-array yields no keys (the loop
/// runs zero times), matching the by-value path's tolerance of non-iterables.
fn ref_array_keys(cell: &Zval) -> Vec<Key> {
    match cell {
        Zval::Array(a) => a.iter().map(|(k, _)| k.clone()).collect(),
        Zval::Ref(rc) => ref_array_keys(&rc.borrow()),
        _ => Vec::new(),
    }
}

/// Promote `array[key]` to a shared cell and return it, de-COW-ing the array in
/// place (REF-3 / future REF-4). Mirrors `eval::place_cell` for a single key
/// step: a missing element auto-vivifies as `Null`, the element is promoted to a
/// `Zval::Ref`, and that shared cell is returned. A reference is followed; a
/// non-array yields a detached cell so the caller has something to bind.
fn elem_cell(cell: &mut Zval, key: &Key) -> Rc<RefCell<Zval>> {
    if let Zval::Ref(rc) = cell {
        let inner = &mut *rc.borrow_mut();
        return elem_cell(inner, key);
    }
    if let Zval::Array(rc) = cell {
        let arr = Rc::make_mut(rc);
        if !arr.contains_key(key) {
            arr.insert(key.clone(), Zval::Null);
        }
        let child = arr.get_mut(key).expect("key present after insert");
        return make_cell(child);
    }
    Rc::new(RefCell::new(Zval::Null))
}

/// The mutable cell a [`FieldBase`] addresses — the root of a [`Op::MakeRef`] /
/// [`Op::BindRefTo`] path. Mirrors the base match in `Vm::field_set`, adding the
/// `$this`-not-in-object error.
fn field_base_mut<'f>(
    frames: &'f mut [Frame<'_>],
    top: usize,
    base: FieldBase,
) -> Result<&'f mut Zval, PhpError> {
    Ok(match base {
        FieldBase::Local(s) => &mut frames[top].slots[s as usize],
        FieldBase::Global(s) => &mut frames[0].slots[s as usize],
        FieldBase::This => frames[top].this.as_mut().ok_or_else(|| {
            PhpError::Error("Using $this when not in object context".to_string())
        })?,
    })
}

/// Navigate `steps` from `target`, auto-vivifying missing levels as NULL, and
/// promote the addressed leaf to a shared `Zval::Ref`, returning its cell
/// (REF-4 + Session A `&$o->p` / `&$a[]`). A reference is followed into its cell;
/// an `Index` step drills into an array element (consuming `keys` in source
/// order), a `Prop` step into an object property, and a final `Append` creates a
/// fresh array element. A scalar/non-object where a container was expected yields
/// a detached cell so the caller does not crash.
fn field_cell(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
) -> Rc<RefCell<Zval>> {
    let Some((first, rest)) = steps.split_first() else {
        return make_cell(target);
    };
    if let Zval::Ref(rc) = target {
        let inner = &mut *rc.borrow_mut();
        return field_cell(inner, steps, keys);
    }
    match first {
        FieldStep::Prop(name) => {
            // `&$o->prop` (Session A): promote the property to a shared cell. A
            // non-object yields a detached cell (PHP warns / does nothing).
            let Zval::Object(o) = target else {
                return Rc::new(RefCell::new(Zval::Null));
            };
            let mut obj = o.borrow_mut();
            if !obj.props.contains(name) {
                obj.props.set(name, Zval::Null);
            }
            let child = obj.props.get_mut(name).expect("property present after insert");
            field_cell(child, rest, keys)
        }
        FieldStep::Append => {
            // `&$a[]` (Session A): append a fresh element and reference it. Append
            // is always the final step (the compiler enforces it).
            if ensure_array(target).is_err() {
                return Rc::new(RefCell::new(Zval::Null));
            }
            let Zval::Array(rc) = target else {
                return Rc::new(RefCell::new(Zval::Null));
            };
            let arr = Rc::make_mut(rc);
            match arr.append_default() {
                Some(child) => field_cell(child, rest, keys),
                None => Rc::new(RefCell::new(Zval::Null)),
            }
        }
        FieldStep::Index => {
            let key = keys.next().expect("ref index key");
            let Some(k) = coerce_key_silent(&key) else {
                return Rc::new(RefCell::new(Zval::Null));
            };
            if ensure_array(target).is_err() {
                return Rc::new(RefCell::new(Zval::Null));
            }
            let Zval::Array(rc) = target else {
                return Rc::new(RefCell::new(Zval::Null));
            };
            let arr = Rc::make_mut(rc);
            if !arr.contains_key(&k) {
                arr.insert(k.clone(), Zval::Null);
            }
            let child = arr.get_mut(&k).expect("key present after insert");
            field_cell(child, rest, keys)
        }
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
        CastKind::Float => Zval::Double(convert::to_double(a)),
        CastKind::String => Zval::Str(convert::to_zstr_cast(a, d)),
        CastKind::Bool => Zval::Bool(convert::to_bool(a, d)),
        // `(array)`: an array passes through, null/unset → empty, a scalar wraps
        // into a single `[0 => v]` element (mirrors `eval::array_cast`).
        CastKind::Array => match a.deref_clone() {
            arr @ Zval::Array(_) => arr,
            Zval::Null | Zval::Undef => Zval::Array(Rc::new(PhpArray::new())),
            scalar => {
                let mut arr = PhpArray::new();
                arr.insert(Key::Int(0), scalar);
                Zval::Array(Rc::new(arr))
            }
        },
        // `(object)` is lowered to a stub by the compiler (it needs VM state).
        CastKind::Object => unreachable!("VM saw an unported (object) cast"),
    }
}

/// Render a `match` subject for the `UnhandledMatchError` message (mirrors
/// `eval::match_case_repr`): scalars print their value (strings quoted), and
/// composite/object values print `of type <name>`.
fn match_case_repr(v: &Zval) -> String {
    match v {
        Zval::Long(i) => i.to_string(),
        Zval::Bool(true) => "true".to_string(),
        Zval::Bool(false) => "false".to_string(),
        Zval::Null | Zval::Undef => "NULL".to_string(),
        Zval::Double(d) => {
            String::from_utf8_lossy(&php_types::dtoa::double_to_shortest(*d)).into_owned()
        }
        Zval::Str(s) => format!("'{}'", String::from_utf8_lossy(s.as_bytes())),
        Zval::Array(_) => "of type array".to_string(),
        Zval::Closure(_) => "of type Closure".to_string(),
        Zval::Object(o) => format!(
            "of type {}",
            String::from_utf8_lossy(o.borrow().class_name.as_bytes())
        ),
        Zval::Generator(_) => "of type Generator".to_string(),
        Zval::Resource(_) => "of type resource".to_string(),
        Zval::Ref(c) => match_case_repr(&c.borrow()),
    }
}

#[cfg(test)]
mod tests {
    use crate::builtin::{Builtin, Ctx, Registry};
    use crate::compile::compile_program;
    use crate::lower::lower_source;
    use php_types::{convert, Diag, PhpError, PhpStr, Zval};

    use super::run_module;

    /// Compile and run a PHP snippet through the bytecode VM (no builtins),
    /// returning stdout. The bulk of the suite is builtin-free control flow.
    fn vm_stdout(src: &[u8]) -> Vec<u8> {
        vm_run(src, &Registry::new()).stdout
    }

    /// Compile and run with a given registry, asserting no fatal; full outcome.
    fn vm_run(src: &[u8], reg: &Registry) -> super::VmOutcome {
        let program = lower_source(b"test.php", src).expect("lower");
        let module = compile_program(&program, reg).expect("compile");
        let out = run_module(&module, reg);
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        out
    }

    // --- fake builtins, to exercise the VM's dispatch mechanism without the
    // real php-builtins crate (which would be a dependency cycle here) ---

    /// `t_double($n)` -> int(n*2). A pure value builtin.
    fn t_double(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
        let n = match args.first() {
            Some(Zval::Long(n)) => *n,
            _ => 0,
        };
        Ok(Zval::Long(n * 2))
    }

    /// `t_emit($s)` -> writes its string arg to stdout, returns null.
    fn t_emit(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
        if let Some(Zval::Str(s)) = args.first() {
            ctx.out.extend_from_slice(s.as_bytes());
        }
        Ok(Zval::Null)
    }

    /// `t_warn()` -> pushes a warning diagnostic, returns null.
    fn t_warn(_args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
        ctx.diags.push(Diag::Warning("from builtin".to_string()));
        Ok(Zval::Null)
    }

    /// `t_set42(&$x)` -> writes int(42) through the by-ref first arg, returns true.
    fn t_set42(target: &mut Zval, _rest: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
        *target = Zval::Long(42);
        Ok(Zval::Bool(true))
    }

    /// A stand-in for the real format engine: return the concatenation of the
    /// arguments after the format string. By the time it runs, `ho_format` has
    /// already resolved any object argument to its `__toString` form (D2), so this
    /// lets the unit tests observe that resolution without the php-builtins crate.
    fn t_sprintf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
        let mut s = Vec::new();
        for a in args.iter().skip(1) {
            s.extend_from_slice(convert::to_zstr(a, ctx.diags).as_bytes());
        }
        Ok(Zval::Str(PhpStr::new(s)))
    }

    fn fake_registry() -> Registry {
        let mut r = Registry::new();
        r.insert(b"t_double".to_vec(), Builtin::Value(t_double));
        r.insert(b"t_emit".to_vec(), Builtin::Value(t_emit));
        r.insert(b"t_warn".to_vec(), Builtin::Value(t_warn));
        r.insert(b"t_set42".to_vec(), Builtin::RefFirst(t_set42));
        r.insert(b"sprintf".to_vec(), Builtin::Value(t_sprintf));
        r
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

    #[test]
    fn array_literal_and_index_read() {
        assert_eq!(vm_stdout(b"<?php $a = [10, 20, 30]; echo $a[1];"), b"20");
    }

    #[test]
    fn keyed_array_literal() {
        assert_eq!(
            vm_stdout(b"<?php $a = ['x' => 5, 'y' => 7]; echo $a['x'] + $a['y'];"),
            b"12"
        );
    }

    #[test]
    fn element_assign_and_append() {
        assert_eq!(
            vm_stdout(b"<?php $a = []; $a[0] = 1; $a[1] = 2; echo $a[0] + $a[1];"),
            b"3"
        );
        assert_eq!(
            vm_stdout(b"<?php $a = []; $a[] = 'p'; $a[] = 'q'; echo $a[0] . $a[1];"),
            b"pq"
        );
    }

    #[test]
    fn autovivification_from_undefined() {
        assert_eq!(vm_stdout(b"<?php $a[] = 1; $a[] = 2; echo $a[0] + $a[1];"), b"3");
        assert_eq!(vm_stdout(b"<?php $a['k'] = 9; echo $a['k'];"), b"9");
    }

    #[test]
    fn nested_array_read() {
        assert_eq!(vm_stdout(b"<?php $a = [[1, 2], [3, 4]]; echo $a[1][0];"), b"3");
    }

    #[test]
    fn string_offset_read() {
        assert_eq!(vm_stdout(b"<?php $s = 'hello'; echo $s[1];"), b"e");
        assert_eq!(vm_stdout(b"<?php $s = 'hello'; echo $s[-1];"), b"o");
    }

    #[test]
    fn coalesce_on_variable() {
        assert_eq!(vm_stdout(b"<?php echo $x ?? 'def';"), b"def");
        assert_eq!(vm_stdout(b"<?php $x = 'v'; echo $x ?? 'def';"), b"v");
    }

    #[test]
    fn coalesce_on_array_element() {
        assert_eq!(
            vm_stdout(b"<?php $a = ['k' => 9]; echo $a['k'] ?? 0; echo $a['m'] ?? 7;"),
            b"97"
        );
    }

    #[test]
    fn coalesce_assign() {
        assert_eq!(vm_stdout(b"<?php $x ??= 5; echo $x;"), b"5");
        assert_eq!(vm_stdout(b"<?php $x = 1; $x ??= 5; echo $x;"), b"1");
    }

    #[test]
    fn nested_array_write() {
        assert_eq!(
            vm_stdout(b"<?php $a = []; $a[0][1] = 'x'; echo $a[0][1];"),
            b"x"
        );
    }

    #[test]
    fn nested_append_autovivifies() {
        // An intermediate `[]` autovivifies a fresh array and appends into it.
        assert_eq!(vm_stdout(b"<?php $a[][] = 'z'; echo $a[0][0];"), b"z");
        assert_eq!(vm_stdout(b"<?php $b[][][] = 'd'; echo $b[0][0][0];"), b"d");
        // Mixed: index then append then index.
        assert_eq!(vm_stdout(b"<?php $c['k'][]['x'] = 1; echo $c['k'][0]['x'];"), b"1");
    }

    #[test]
    fn nested_write_autovivifies_each_level() {
        assert_eq!(vm_stdout(b"<?php $a[1][2][3] = 5; echo $a[1][2][3];"), b"5");
    }

    #[test]
    fn nested_append() {
        assert_eq!(
            vm_stdout(b"<?php $a[0][] = 'p'; $a[0][] = 'q'; echo $a[0][0] . $a[0][1];"),
            b"pq"
        );
    }

    #[test]
    fn compound_assign_on_element() {
        assert_eq!(
            vm_stdout(b"<?php $a = ['n' => 10]; $a['n'] += 5; echo $a['n'];"),
            b"15"
        );
        // Compound on a missing element starts from NULL.
        assert_eq!(vm_stdout(b"<?php $a['c'] += 3; echo $a['c'];"), b"3");
        // Nested compound.
        assert_eq!(
            vm_stdout(b"<?php $a[0][0] = 1; $a[0][0] += 9; echo $a[0][0];"),
            b"10"
        );
    }

    #[test]
    fn incdec_on_element() {
        assert_eq!(vm_stdout(b"<?php $a = [5]; $a[0]++; echo $a[0];"), b"6");
        assert_eq!(vm_stdout(b"<?php $a = [5]; echo ++$a[0];"), b"6");
        // Postfix yields the old value.
        assert_eq!(vm_stdout(b"<?php $a = [5]; echo $a[0]++, '/', $a[0];"), b"5/6");
        // Nested.
        assert_eq!(vm_stdout(b"<?php $a[2][3] = 9; $a[2][3]--; echo $a[2][3];"), b"8");
    }

    #[test]
    fn isset_variable_and_element() {
        assert_eq!(vm_stdout(b"<?php echo isset($x) ? 'y' : 'n';"), b"n");
        assert_eq!(vm_stdout(b"<?php $x = 1; echo isset($x) ? 'y' : 'n';"), b"y");
        // A null value is not "set".
        assert_eq!(vm_stdout(b"<?php $x = null; echo isset($x) ? 'y' : 'n';"), b"n");
        assert_eq!(
            vm_stdout(b"<?php $a = ['k' => 1]; echo isset($a['k']) ? 'y' : 'n', isset($a['m']) ? 'y' : 'n';"),
            b"yn"
        );
        // Nested + missing intermediate level.
        assert_eq!(vm_stdout(b"<?php $a[1][2] = 3; echo isset($a[1][2]) ? 'y' : 'n', isset($a[9][9]) ? 'y' : 'n';"), b"yn");
    }

    #[test]
    fn isset_multiple_is_and() {
        assert_eq!(vm_stdout(b"<?php $a = 1; $b = 2; echo isset($a, $b) ? 'y' : 'n';"), b"y");
        assert_eq!(vm_stdout(b"<?php $a = 1; echo isset($a, $b) ? 'y' : 'n';"), b"n");
    }

    #[test]
    fn isset_on_string_offset() {
        assert_eq!(vm_stdout(b"<?php $s = 'hi'; echo isset($s[1]) ? 'y' : 'n', isset($s[5]) ? 'y' : 'n';"), b"yn");
    }

    #[test]
    fn empty_semantics() {
        assert_eq!(vm_stdout(b"<?php echo empty($x) ? 'y' : 'n';"), b"y");
        assert_eq!(vm_stdout(b"<?php $x = 0; echo empty($x) ? 'y' : 'n';"), b"y");
        assert_eq!(vm_stdout(b"<?php $x = 'a'; echo empty($x) ? 'y' : 'n';"), b"n");
        assert_eq!(
            vm_stdout(b"<?php $a = ['k' => 0, 'm' => 5]; echo empty($a['k']) ? 'y' : 'n', empty($a['m']) ? 'y' : 'n';"),
            b"yn"
        );
    }

    #[test]
    fn foreach_value_only() {
        assert_eq!(vm_stdout(b"<?php foreach ([1, 2, 3] as $v) echo $v;"), b"123");
    }

    #[test]
    fn foreach_key_and_value() {
        assert_eq!(
            vm_stdout(b"<?php foreach (['a' => 1, 'b' => 2] as $k => $v) echo $k, $v;"),
            b"a1b2"
        );
    }

    #[test]
    fn foreach_iterates_a_snapshot() {
        // Mutating the source inside the body does not change the iteration.
        assert_eq!(
            vm_stdout(b"<?php $a = [1, 2, 3]; foreach ($a as $v) { $a[] = 99; echo $v; }"),
            b"123"
        );
    }

    #[test]
    fn foreach_with_break_and_continue() {
        assert_eq!(
            vm_stdout(b"<?php foreach ([1, 2, 3, 4] as $v) { if ($v == 3) break; echo $v; }"),
            b"12"
        );
        assert_eq!(
            vm_stdout(b"<?php foreach ([1, 2, 3, 4] as $v) { if ($v == 2) continue; echo $v; }"),
            b"134"
        );
    }

    #[test]
    fn nested_foreach_with_break_levels() {
        // break 2 must free both iterators and leave the outer loop cleanly.
        assert_eq!(
            vm_stdout(
                b"<?php foreach ([1, 2] as $i) { foreach ([3, 4] as $j) { echo $i, $j; if ($j == 3 && $i == 1) break 2; } } echo 'X';"
            ),
            b"13X"
        );
    }

    #[test]
    fn foreach_over_non_array_is_empty() {
        assert_eq!(vm_stdout(b"<?php foreach (null as $v) echo $v; echo 'done';"), b"done");
    }

    #[test]
    fn unset_variable_and_element() {
        assert_eq!(vm_stdout(b"<?php $x = 1; unset($x); echo isset($x) ? 'y' : 'n';"), b"n");
        assert_eq!(
            vm_stdout(b"<?php $a = ['k' => 1, 'm' => 2]; unset($a['k']); echo isset($a['k']) ? 'y' : 'n', $a['m'];"),
            b"n2"
        );
        // Nested unset leaves siblings intact.
        assert_eq!(
            vm_stdout(b"<?php $a[1][2] = 'x'; $a[1][3] = 'y'; unset($a[1][2]); echo isset($a[1][2]) ? 'y' : 'n', $a[1][3];"),
            b"ny"
        );
        // unset of multiple targets.
        assert_eq!(
            vm_stdout(b"<?php $a = 1; $b = 2; unset($a, $b); echo isset($a) ? '1' : '0', isset($b) ? '1' : '0';"),
            b"00"
        );
    }

    // --- builtin dispatch mechanism (with fake builtins) ---

    #[test]
    fn value_builtin_returns_a_value() {
        let out = vm_run(b"<?php echo t_double(21);", &fake_registry());
        assert_eq!(out.stdout, b"42");
    }

    #[test]
    fn value_builtin_writes_to_stdout() {
        let out = vm_run(b"<?php t_emit('hi'); t_emit('!');", &fake_registry());
        assert_eq!(out.stdout, b"hi!");
    }

    #[test]
    fn value_builtin_in_expression() {
        let out = vm_run(b"<?php echo t_double(t_double(3)) + 1;", &fake_registry());
        assert_eq!(out.stdout, b"13");
    }

    #[test]
    fn builtin_diagnostics_propagate() {
        let out = vm_run(b"<?php t_warn();", &fake_registry());
        assert_eq!(out.diags.len(), 1);
    }

    #[test]
    fn ref_builtin_writes_through_to_the_variable() {
        let out = vm_run(b"<?php $x = 1; t_set42($x); echo $x;", &fake_registry());
        assert_eq!(out.stdout, b"42");
    }

    #[test]
    fn user_function_shadows_a_builtin() {
        // A user t_double wins over the registry's t_double.
        let out = vm_run(
            b"<?php function t_double($n) { return $n + 100; } echo t_double(5);",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"105");
    }

    #[test]
    fn unknown_function_is_unsupported_at_compile_time() {
        // Not a user function and not in the registry -> the module won't compile
        // for the VM (so the harness can fall back to the tree-walker).
        let program = lower_source(b"test.php", b"<?php echo no_such_fn();").expect("lower");
        let reg = fake_registry();
        assert!(compile_program(&program, &reg).is_err());
    }

    // --- switch ---

    #[test]
    fn switch_basic_with_break() {
        assert_eq!(
            vm_stdout(b"<?php $x = 2; switch ($x) { case 1: echo 'a'; break; case 2: echo 'b'; break; default: echo 'd'; }"),
            b"b"
        );
    }

    #[test]
    fn switch_fall_through() {
        assert_eq!(
            vm_stdout(b"<?php $x = 1; switch ($x) { case 1: echo 'a'; case 2: echo 'b'; break; case 3: echo 'c'; }"),
            b"ab"
        );
    }

    #[test]
    fn switch_default_in_the_middle_falls_through() {
        assert_eq!(
            vm_stdout(b"<?php $x = 9; switch ($x) { case 1: echo '1'; default: echo 'd'; case 2: echo '2'; }"),
            b"d2"
        );
    }

    #[test]
    fn switch_uses_loose_equality() {
        assert_eq!(
            vm_stdout(b"<?php switch ('1') { case 1: echo 'y'; break; default: echo 'n'; }"),
            b"y"
        );
    }

    #[test]
    fn switch_break_inside_loop_leaves_only_the_switch() {
        assert_eq!(
            vm_stdout(b"<?php for ($i = 0; $i < 3; $i++) { switch ($i) { case 1: break; default: echo $i; } }"),
            b"02"
        );
    }

    // --- match ---

    #[test]
    fn match_basic() {
        assert_eq!(vm_stdout(b"<?php echo match (2) { 1 => 'a', 2 => 'b', 3 => 'c' };"), b"b");
    }

    #[test]
    fn match_multiple_conditions() {
        assert_eq!(vm_stdout(b"<?php echo match (3) { 1, 2 => 'low', 3, 4 => 'high' };"), b"high");
    }

    #[test]
    fn match_default() {
        assert_eq!(vm_stdout(b"<?php echo match (9) { 1 => 'a', default => 'd' };"), b"d");
    }

    #[test]
    fn match_is_strict() {
        // '1' !== 1, so the string arm wins.
        assert_eq!(vm_stdout(b"<?php echo match ('1') { 1 => 'int', '1' => 'str' };"), b"str");
    }

    #[test]
    fn match_unhandled_is_fatal() {
        let program = lower_source(b"test.php", b"<?php echo match (9) { 1 => 'a' };").expect("lower");
        let reg = Registry::new();
        let module = compile_program(&program, &reg).expect("compile");
        let out = run_module(&module, &reg);
        assert!(out.fatal.is_some());
    }

    // --- OOP-1: classes, objects, $this, properties, methods, instanceof ---

    /// Compile and run a snippet (no builtins), returning the full outcome — used
    /// for the fatal-path OOP tests.
    fn vm_outcome(src: &[u8]) -> super::VmOutcome {
        let program = lower_source(b"test.php", src).expect("lower");
        let reg = Registry::new();
        let module = compile_program(&program, &reg).expect("compile");
        run_module(&module, &reg)
    }

    // ----- E1: VmOutcome parity (rendered / return_value / fatal) vs PHP 8.5.7 -----

    #[test]
    fn inline_html_around_php_tags() {
        // Text outside `<?php … ?>` is emitted verbatim (unblocks the corpus, E4).
        assert_eq!(vm_stdout(b"hello <?php echo 'X'; ?> world"), b"hello X world");
    }

    #[test]
    fn inline_html_after_close_tag() {
        // PHP swallows exactly one newline right after `?>`, so only "tail" leaks.
        let out = vm_outcome(b"<?php echo 'a'; ?>\ntail");
        assert_eq!(out.stdout, b"atail");
        assert_eq!(out.rendered, b"atail");
    }

    #[test]
    fn rendered_equals_stdout_when_no_diagnostics() {
        let out = vm_outcome(b"<?php echo 'hello';");
        assert_eq!(out.rendered, b"hello");
        assert_eq!(out.stdout, b"hello");
    }

    #[test]
    fn rendered_interleaves_array_to_string_warning() {
        let out = vm_outcome(b"<?php echo [1,2];");
        assert_eq!(
            out.rendered,
            b"\nWarning: Array to string conversion in test.php on line 1\nArray".to_vec()
        );
        // stdout stays the pure output (no diagnostic text).
        assert_eq!(out.stdout, b"Array");
    }

    #[test]
    fn rendered_assign_ref_to_non_ref_function_notice() {
        // `$y = &f()` where f is NOT by-reference: copy the value and raise a
        // notice (rendered, interleaved before the echo). stdout stays clean.
        let out = vm_outcome(b"<?php function f(){ return 5; } $y = &f(); echo $y;");
        assert_eq!(
            out.rendered,
            b"\nNotice: Only variables should be assigned by reference in test.php on line 1\n5".to_vec()
        );
        assert_eq!(out.stdout, b"5");
    }

    #[test]
    fn rendered_return_by_ref_non_lvalue_notice() {
        // A by-ref function returning a non-lvalue raises the return-side notice
        // and still yields the value.
        let out = vm_outcome(b"<?php function &f(){ return 1 + 2; } $y = &f(); echo $y;");
        assert_eq!(
            out.rendered,
            b"\nNotice: Only variable references should be returned by reference in test.php on line 1\n3".to_vec()
        );
        assert_eq!(out.stdout, b"3");
    }

    #[test]
    fn rendered_undefined_variable_warning() {
        // Reading an unset variable warns (PHP 8) and yields NULL; the warning is
        // flushed at the echo with that line, interleaved before the output.
        let out = vm_outcome(b"<?php\necho 'a';\necho $undef;\necho 'b';\n");
        assert_eq!(
            out.rendered,
            b"a\nWarning: Undefined variable $undef in test.php on line 3\nb".to_vec()
        );
        assert_eq!(out.stdout, b"ab");
    }

    #[test]
    fn rendered_undefined_array_key_warning() {
        // Reading a missing array key warns "Undefined array key 5" and yields NULL.
        let out = vm_outcome(b"<?php\n$a = [1];\necho $a[5];\n");
        assert_eq!(
            out.rendered,
            b"\nWarning: Undefined array key 5 in test.php on line 3\n".to_vec()
        );
    }

    #[test]
    fn coalesce_and_isset_suppress_undefined_variable_warning() {
        // `??`, isset, and `@` must NOT raise the undefined-variable warning.
        let out = vm_outcome(b"<?php $y = $x ?? 'd'; echo $y; echo isset($z) ? 'S' : 'U';");
        assert_eq!(out.rendered, b"dU".to_vec());
        assert_eq!(out.stdout, b"dU");
    }

    #[test]
    fn rendered_appends_uncaught_exception_fatal() {
        let out = vm_outcome(b"<?php\necho \"before\\n\";\nthrow new Exception(\"boom\");");
        assert_eq!(
            out.rendered,
            b"before\n\nFatal error: Uncaught Exception: boom in test.php:3\nStack trace:\n#0 {main}\n  thrown in test.php on line 3\n".to_vec()
        );
        assert!(out.fatal.is_some());
    }

    #[test]
    fn rendered_appends_engine_error_fatal() {
        let out = vm_outcome(b"<?php\necho 1 % 0;");
        assert_eq!(
            out.rendered,
            b"\nFatal error: Uncaught DivisionByZeroError: Modulo by zero in test.php:2\nStack trace:\n#0 {main}\n  thrown in test.php on line 2\n".to_vec()
        );
    }

    #[test]
    fn outcome_captures_top_level_return_value() {
        let out = vm_outcome(b"<?php return 6 * 7;");
        assert!(matches!(out.return_value, Zval::Long(42)));
        assert!(out.fatal.is_none());
    }

    #[test]
    fn caught_exception_does_not_render_a_fatal() {
        // A caught exception leaves no fatal block in `rendered`.
        let out = vm_outcome(b"<?php try { throw new Exception('x'); } catch (Exception $e) { echo 'caught'; }");
        assert_eq!(out.rendered, b"caught");
        assert!(out.fatal.is_none());
    }

    // ----- E2: vm::run_source_with (lower → compile → run) -----

    #[test]
    fn run_source_with_runs_plain_code() {
        let reg = Registry::new();
        let out = super::run_source_with(b"test.php", b"<?php echo 'ok';", &reg).expect("ok");
        assert_eq!(out.rendered, b"ok");
    }

    #[test]
    fn run_source_with_reports_vm_unsupported() {
        // `var_export` is still an evaluator-only host builtin the bytecode
        // compiler rejects — surfaced as `VmRunError::Unsupported`, not a fatal.
        // (`array_map`/`get_object_vars` are now VM-native, Sessions C/B2.)
        let reg = Registry::new();
        let err = super::run_source_with(b"test.php", b"<?php var_export($x);", &reg)
            .expect_err("vm should reject var_export");
        assert!(matches!(err, super::VmRunError::Unsupported(_)));
    }

    #[test]
    fn constructor_sets_property_read_back() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x; function __construct($v) { $this->x = $v; } } $o = new C(7); echo $o->x;"),
            b"7"
        );
    }

    #[test]
    fn constant_property_default() {
        assert_eq!(vm_stdout(b"<?php class C { public $x = 5; } $o = new C(); echo $o->x;"), b"5");
    }

    #[test]
    fn property_write_then_read() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x; } $o = new C(); $o->x = 'hi'; echo $o->x;"),
            b"hi"
        );
    }

    #[test]
    fn method_call_returns_value_from_this() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = 10; function get() { return $this->x; } } $o = new C(); echo $o->get();"),
            b"10"
        );
    }

    #[test]
    fn method_takes_arguments() {
        assert_eq!(
            vm_stdout(b"<?php class C { function add($a, $b) { return $a + $b; } } $o = new C(); echo $o->add(2, 3);"),
            b"5"
        );
    }

    #[test]
    fn compound_and_incdec_on_this_property() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $n = 0; function bump() { $this->n += 5; $this->n++; return $this->n; } } $o = new C(); echo $o->bump();"),
            b"6"
        );
    }

    #[test]
    fn isset_and_unset_property() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = 1; } $o = new C(); echo isset($o->x) ? 'y' : 'n'; unset($o->x); echo isset($o->x) ? 'y' : 'n';"),
            b"yn"
        );
        // A null-valued property is not "set".
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = null; } $o = new C(); echo isset($o->x) ? 'y' : 'n';"),
            b"n"
        );
    }

    #[test]
    fn inherited_method() {
        assert_eq!(
            vm_stdout(b"<?php class A { function hi() { return 'A'; } } class B extends A {} $o = new B(); echo $o->hi();"),
            b"A"
        );
    }

    #[test]
    fn overridden_method() {
        assert_eq!(
            vm_stdout(b"<?php class A { function hi() { return 'A'; } } class B extends A { function hi() { return 'B'; } } $o = new B(); echo $o->hi();"),
            b"B"
        );
    }

    #[test]
    fn inherited_constructor() {
        assert_eq!(
            vm_stdout(b"<?php class A { public $x; function __construct($v) { $this->x = $v; } } class B extends A {} $o = new B(9); echo $o->x;"),
            b"9"
        );
    }

    #[test]
    fn inherited_and_overridden_property_defaults() {
        // Parent-first layout; B redeclares $x with a new default, keeps $y.
        assert_eq!(
            vm_stdout(b"<?php class A { public $x = 1; public $y = 2; } class B extends A { public $x = 10; } $o = new B(); echo $o->x, $o->y;"),
            b"102"
        );
    }

    #[test]
    fn instanceof_self_parent_interface_and_false() {
        assert_eq!(
            vm_stdout(
                b"<?php interface I {} class A implements I {} class B extends A {} class C {} $o = new B(); echo ($o instanceof B) ? '1' : '0', ($o instanceof A) ? '1' : '0', ($o instanceof I) ? '1' : '0', ($o instanceof C) ? '1' : '0';"
            ),
            b"1110"
        );
    }

    #[test]
    fn instanceof_non_object_is_false() {
        assert_eq!(
            vm_stdout(b"<?php class C {} $x = 5; echo ($x instanceof C) ? '1' : '0';"),
            b"0"
        );
    }

    #[test]
    fn instanceof_generator_builtin_interfaces() {
        // A generator is a Generator/Iterator/Traversable, but not Countable.
        assert_eq!(
            vm_stdout(
                b"<?php function g(){yield 1;} $g=g(); \
                  echo ($g instanceof Generator)?'1':'0', ($g instanceof Iterator)?'1':'0', \
                  ($g instanceof Traversable)?'1':'0', ($g instanceof Countable)?'1':'0';"
            ),
            b"1110"
        );
    }

    #[test]
    fn object_handle_semantics_are_shared() {
        // Two handles to the same instance see each other's mutations (no COW).
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = 1; } $a = new C(); $b = $a; $b->x = 99; echo $a->x;"),
            b"99"
        );
    }

    #[test]
    fn instantiating_abstract_class_is_fatal() {
        assert!(vm_outcome(b"<?php abstract class A {} $o = new A();").fatal.is_some());
    }

    #[test]
    fn instantiating_interface_is_fatal() {
        assert!(vm_outcome(b"<?php interface I {} $o = new I();").fatal.is_some());
    }

    #[test]
    fn non_constant_property_default_is_initialised() {
        // An array default is materialised by the prop-init thunk at `new` time
        // (OOP-2b), not stubbed.
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = [1, 2, 3]; } $o = new C(); echo $o->x[0], $o->x[2];"),
            b"13"
        );
    }

    #[test]
    fn non_constant_default_set_before_constructor() {
        // The prop-init thunk runs before __construct, which can then read it.
        assert_eq!(
            vm_stdout(b"<?php class C { public $arr = [5, 6]; public $first; function __construct() { $this->first = $this->arr[0]; } } $o = new C(); echo $o->first, $o->arr[1];"),
            b"56"
        );
    }

    // --- OOP-2a: self/parent/static, class constants, static calls ---

    #[test]
    fn class_constant_and_class_name() {
        assert_eq!(vm_stdout(b"<?php class C { const N = 42; } echo C::N, '/', C::class;"), b"42/C");
    }

    #[test]
    fn self_and_parent_constants_resolve_by_defining_class() {
        // f() lives in A (self::V = 'a'); g()/h() in B (parent::V = 'a', self::V = 'b').
        assert_eq!(
            vm_stdout(b"<?php class A { const V = 'a'; function f() { return self::V; } } class B extends A { const V = 'b'; function g() { return parent::V; } function h() { return self::V; } } $b = new B(); echo $b->f(), $b->g(), $b->h();"),
            b"aab"
        );
    }

    #[test]
    fn constant_referencing_another_constant() {
        assert_eq!(vm_stdout(b"<?php class C { const A = 10; const B = self::A + 5; } echo C::B;"), b"15");
    }

    #[test]
    fn late_static_binding_via_named_static_call() {
        // who() is declared in A; `static::class` reflects the *called* class.
        assert_eq!(
            vm_stdout(b"<?php class A { static function who() { return static::class; } } class B extends A {} echo A::who(), '/', B::who();"),
            b"A/B"
        );
    }

    #[test]
    fn forwarding_static_call_preserves_lsb() {
        // b() calls self::a(); self:: is forwarding, so a() sees LSB = B.
        assert_eq!(
            vm_stdout(b"<?php class A { static function a() { return static::class; } static function b() { return self::a(); } } class B extends A {} echo B::b();"),
            b"B"
        );
    }

    #[test]
    fn new_static_instantiates_the_called_class() {
        assert_eq!(
            vm_stdout(b"<?php class A { static function make() { return new static(); } } class B extends A {} $x = A::make(); $y = B::make(); echo ($x instanceof A) ? '1' : '0', ($x instanceof B) ? '1' : '0', ($y instanceof B) ? '1' : '0';"),
            b"101"
        );
    }

    #[test]
    fn new_self_instantiates_the_defining_class() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = 5; static function make() { return new self(); } } $o = C::make(); echo $o->x;"),
            b"5"
        );
    }

    #[test]
    fn parent_static_call_forwards_this() {
        // B::greet() forwards to A::greet() keeping $this, which reads B's property.
        assert_eq!(
            vm_stdout(b"<?php class A { function greet() { return 'hi from ' . $this->name; } } class B extends A { public $name = 'B'; function greet() { return parent::greet(); } } $b = new B(); echo $b->greet();"),
            b"hi from B"
        );
    }

    #[test]
    fn instanceof_self_in_method() {
        assert_eq!(
            vm_stdout(b"<?php class A {} class B extends A { function test($o) { return ($o instanceof self) ? '1' : '0'; } } $b = new B(); $a = new A(); echo $b->test($b), $b->test($a);"),
            b"10"
        );
    }

    #[test]
    fn instanceof_static_uses_lsb() {
        assert_eq!(
            vm_stdout(b"<?php class A { function check($o) { return ($o instanceof static) ? '1' : '0'; } } class B extends A {} $b = new B(); $a = new A(); echo $b->check($b), $b->check($a);"),
            b"10"
        );
    }

    // --- OOP-2b: static properties + visibility enforcement ---

    #[test]
    fn static_property_shared_across_instances() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $count = 0; function inc() { self::$count++; } } $a = new C(); $b = new C(); $a->inc(); $b->inc(); $a->inc(); echo C::$count;"),
            b"3"
        );
    }

    #[test]
    fn static_property_write_and_op_assign() {
        assert_eq!(vm_stdout(b"<?php class C { public static $x = 1; } C::$x = 42; echo C::$x;"), b"42");
        assert_eq!(vm_stdout(b"<?php class C { public static $n = 10; } C::$n += 5; echo C::$n;"), b"15");
    }

    #[test]
    fn static_property_non_constant_default_lazy_init() {
        // An array default initialises via its thunk on first access.
        assert_eq!(vm_stdout(b"<?php class C { public static $list = [1, 2, 3]; } echo C::$list[1], C::$list[2];"), b"23");
    }

    #[test]
    fn inherited_static_property_shares_one_cell() {
        // B::$v resolves to A's declaration; a write through B is seen through A.
        assert_eq!(
            vm_stdout(b"<?php class A { public static $v = 'p'; } class B extends A {} echo B::$v; B::$v = 'q'; echo A::$v;"),
            b"pq"
        );
    }

    #[test]
    fn static_property_coalesce_assign() {
        assert_eq!(vm_stdout(b"<?php class C { public static $x = null; } C::$x ??= 7; echo C::$x;"), b"7");
    }

    #[test]
    fn private_property_accessible_from_inside_only() {
        assert_eq!(
            vm_stdout(b"<?php class C { private $secret = 42; function reveal() { return $this->secret; } } $o = new C(); echo $o->reveal();"),
            b"42"
        );
        assert!(vm_outcome(b"<?php class C { private $secret = 1; } $o = new C(); echo $o->secret;").fatal.is_some());
    }

    #[test]
    fn protected_property_visible_in_subclass_but_not_outside() {
        assert_eq!(
            vm_stdout(b"<?php class A { protected $x = 7; } class B extends A { function get() { return $this->x; } } $o = new B(); echo $o->get();"),
            b"7"
        );
        assert!(vm_outcome(b"<?php class A { protected $x = 7; } $o = new A(); echo $o->x;").fatal.is_some());
    }

    #[test]
    fn private_method_accessible_from_inside_only() {
        assert_eq!(
            vm_stdout(b"<?php class C { private function secret() { return 9; } function call_it() { return $this->secret(); } } $o = new C(); echo $o->call_it();"),
            b"9"
        );
        assert!(vm_outcome(b"<?php class C { private function secret() { return 1; } } $o = new C(); echo $o->secret();").fatal.is_some());
    }

    #[test]
    fn isset_on_inaccessible_property_is_false() {
        assert_eq!(
            vm_stdout(b"<?php class C { private $x = 1; } $o = new C(); echo isset($o->x) ? 'y' : 'n';"),
            b"n"
        );
    }

    #[test]
    fn private_static_property_from_outside_is_fatal() {
        assert!(vm_outcome(b"<?php class C { private static $s = 1; } echo C::$s;").fatal.is_some());
    }

    // --- OOP-2c (1/2): nullsafe ?-> ---

    #[test]
    fn nullsafe_property_on_object_and_null() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $x = 7; } $o = new C(); echo $o?->x; $n = null; echo $n?->x; echo 'end';"),
            b"7end"
        );
    }

    #[test]
    fn nullsafe_method_on_object_and_null() {
        assert_eq!(
            vm_stdout(b"<?php class C { function f() { return 'hi'; } } $o = new C(); echo $o?->f(); $n = null; echo $n?->f(); echo 'end';"),
            b"hiend"
        );
    }

    #[test]
    fn nullsafe_chain_short_circuits() {
        // $n?->a?->b: an all-nullsafe chain yields null without erroring.
        assert_eq!(
            vm_stdout(b"<?php $n = null; echo ($n?->a?->b) === null ? 'null' : 'set';"),
            b"null"
        );
    }

    // --- OOP-2c (2/2): mixed property + index paths ---

    #[test]
    fn property_array_element_write_and_read() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $arr = []; } $o = new C(); $o->arr[0] = 'x'; $o->arr['k'] = 'y'; echo $o->arr[0], $o->arr['k'];"),
            b"xy"
        );
    }

    #[test]
    fn this_property_append() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $items = []; function add($v) { $this->items[] = $v; } function get($i) { return $this->items[$i]; } } $o = new C(); $o->add('a'); $o->add('b'); echo $o->get(0), $o->get(1);"),
            b"ab"
        );
    }

    #[test]
    fn nested_object_property_write() {
        assert_eq!(
            vm_stdout(b"<?php class P { public $inner; } class Q { public $val = 1; } $p = new P(); $p->inner = new Q(); $p->inner->val = 99; echo $p->inner->val;"),
            b"99"
        );
    }

    #[test]
    fn compound_assign_on_property_element() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $counts = []; function bump($k) { $this->counts[$k] += 1; } } $o = new C(); $o->bump('a'); $o->bump('a'); $o->bump('b'); echo $o->counts['a'], $o->counts['b'];"),
            b"21"
        );
    }

    #[test]
    fn incdec_on_property_element() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $arr = [5]; } $o = new C(); $o->arr[0]++; echo $o->arr[0];"),
            b"6"
        );
    }

    #[test]
    fn isset_and_unset_on_property_element() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $arr = ['k' => 1]; } $o = new C(); echo isset($o->arr['k']) ? 'y' : 'n', isset($o->arr['z']) ? 'y' : 'n'; unset($o->arr['k']); echo isset($o->arr['k']) ? 'y' : 'n';"),
            b"ynn"
        );
    }

    #[test]
    fn nested_autovivification_through_property() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $data; } $o = new C(); $o->data['x']['y'] = 7; echo $o->data['x']['y'];"),
            b"7"
        );
    }

    // --- OOP-3a: __call / __callStatic ---

    #[test]
    fn magic_call_for_missing_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __call($name, $args) { return $name . '/' . $args[0]; } } $o = new C(); echo $o->foo('x');"),
            b"foo/x"
        );
    }

    #[test]
    fn magic_call_receives_argument_array() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __call($n, $a) { return $a[0] + $a[1]; } } $o = new C(); echo $o->sum(2, 3);"),
            b"5"
        );
    }

    #[test]
    fn magic_call_for_inaccessible_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { private function secret() { return 'no'; } function __call($n, $a) { return 'magic:' . $n; } } $o = new C(); echo $o->secret();"),
            b"magic:secret"
        );
    }

    #[test]
    fn real_method_not_routed_to_magic_call() {
        assert_eq!(
            vm_stdout(b"<?php class C { function real() { return 'real'; } function __call($n, $a) { return 'magic'; } } $o = new C(); echo $o->real();"),
            b"real"
        );
    }

    #[test]
    fn magic_callstatic_for_missing_static_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function __callStatic($name, $args) { return 'static:' . $name; } } echo C::foo();"),
            b"static:foo"
        );
    }

    #[test]
    fn undefined_method_without_magic_is_fatal() {
        assert!(vm_outcome(b"<?php class C {} $o = new C(); echo $o->nope();").fatal.is_some());
        assert!(vm_outcome(b"<?php class C {} echo C::nope();").fatal.is_some());
    }

    // --- OOP-3b: __get / __set / __isset / __unset ---

    #[test]
    fn magic_get_for_missing_and_inaccessible() {
        assert_eq!(
            vm_stdout(b"<?php class C { private $data = 1; function __get($n) { return 'got:' . $n; } } $o = new C(); echo $o->missing, '/', $o->data;"),
            b"got:missing/got:data"
        );
    }

    #[test]
    fn magic_set_then_get_roundtrip() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $store = []; function __set($n, $v) { $this->store[$n] = $v; } function __get($n) { return $this->store[$n]; } } $o = new C(); $o->x = 5; echo $o->x;"),
            b"5"
        );
    }

    #[test]
    fn magic_set_expression_yields_assigned_value() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __set($n, $v) {} } $o = new C(); $r = ($o->x = 42); echo $r;"),
            b"42"
        );
    }

    #[test]
    fn magic_isset_coerces_to_bool() {
        assert_eq!(
            vm_stdout(b"<?php class C { private $h = ['a' => 1]; function __isset($n) { return isset($this->h[$n]); } } $o = new C(); echo isset($o->a) ? 'y' : 'n', isset($o->b) ? 'y' : 'n';"),
            b"yn"
        );
    }

    #[test]
    fn magic_unset_is_invoked() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $log = ''; function __unset($n) { $this->log = 'unset:' . $n; } } $o = new C(); unset($o->ghost); echo $o->log;"),
            b"unset:ghost"
        );
    }

    #[test]
    fn magic_get_recursion_is_guarded() {
        // __get reading the same missing property must not recurse forever: the
        // guard makes the inner access fall through to a direct (null) read.
        assert_eq!(
            vm_stdout(b"<?php class C { function __get($n) { return $this->missing; } } $o = new C(); $x = $o->missing; echo ($x === null) ? 'null' : 'other';"),
            b"null"
        );
    }

    // --- OOP-3c: __toString ---

    #[test]
    fn tostring_on_echo() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __toString() { return 'hello'; } } $o = new C(); echo $o;"),
            b"hello"
        );
    }

    #[test]
    fn tostring_in_concatenation() {
        assert_eq!(
            vm_stdout(b"<?php class Money { public $amt; function __construct($a) { $this->amt = $a; } function __toString() { return '$' . $this->amt; } } $m = new Money(5); echo 'price: ' . $m;"),
            b"price: $5"
        );
    }

    #[test]
    fn tostring_in_interpolation() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __toString() { return 'V'; } } $o = new C(); echo \"val=$o!\";"),
            b"val=V!"
        );
    }

    #[test]
    fn tostring_on_string_cast() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __toString() { return 'X'; } } $o = new C(); $s = (string)$o; echo $s;"),
            b"X"
        );
    }

    #[test]
    fn tostring_via_print() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __toString() { return 'P'; } } $o = new C(); print $o;"),
            b"P"
        );
    }

    #[test]
    fn object_without_tostring_is_fatal_on_echo() {
        assert!(vm_outcome(b"<?php class C {} $o = new C(); echo $o;").fatal.is_some());
    }

    // --- OOP-3d: __destruct + destruction sweep ---

    #[test]
    fn destructor_runs_at_shutdown() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __destruct() { echo 'bye'; } } $o = new C(); echo 'mid';"),
            b"midbye"
        );
    }

    #[test]
    fn destructor_runs_mid_script_on_unset() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __destruct() { echo 'D'; } } $o = new C(); unset($o); echo 'after';"),
            b"Dafter"
        );
    }

    #[test]
    fn destructor_runs_on_reassignment() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $n; function __construct($n) { $this->n = $n; } function __destruct() { echo 'd' . $this->n; } } $o = new C(1); $o = new C(2); echo 'x';"),
            b"d1xd2"
        );
    }

    #[test]
    fn shutdown_destructors_run_lifo() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $n; function __construct($n) { $this->n = $n; } function __destruct() { echo $this->n; } } $a = new C(1); $b = new C(2); echo '|';"),
            b"|21"
        );
    }

    #[test]
    fn destructorless_object_freed_silently() {
        assert_eq!(
            vm_stdout(b"<?php class C {} $o = new C(); unset($o); echo 'ok';"),
            b"ok"
        );
    }

    // ----- REF-1: `$a = &$b` (bare variables) + `global` -----

    #[test]
    fn ref_alias_writes_through_to_original() {
        // Writing through the alias updates the original.
        assert_eq!(vm_stdout(b"<?php $a = 1; $b = &$a; $b = 2; echo $a;"), b"2");
    }

    #[test]
    fn ref_original_writes_visible_through_alias() {
        // Writing through the original is visible through the alias.
        assert_eq!(vm_stdout(b"<?php $a = 1; $b = &$a; $a = 5; echo $b;"), b"5");
    }

    #[test]
    fn ref_assignment_is_an_expression() {
        // `$b = &$a` yields the aliased value, usable in a surrounding assignment.
        assert_eq!(vm_stdout(b"<?php $a = 7; $c = ($b = &$a); echo $c;"), b"7");
    }

    #[test]
    fn ref_to_undefined_var_promotes_to_null_cell() {
        // Aliasing an undefined variable defines a shared NULL cell; a later write
        // through the alias creates the original (D-12.4 semantics for bare vars).
        assert_eq!(vm_stdout(b"<?php $b = &$a; $b = 9; echo $a;"), b"9");
    }

    #[test]
    fn ref_chain_three_aliases() {
        // A three-way alias chain all shares one cell.
        assert_eq!(
            vm_stdout(b"<?php $a = 1; $b = &$a; $c = &$b; $c = 8; echo $a + $b + $c;"),
            b"24"
        );
    }

    #[test]
    fn global_reads_and_writes_through_into_global() {
        assert_eq!(
            vm_stdout(b"<?php $g = 10; function f() { global $g; $g = $g + 5; } f(); echo $g;"),
            b"15"
        );
    }

    #[test]
    fn global_creates_undefined_global() {
        // `global $g` on an undefined global promotes it to a cell; a write through
        // the alias creates the global, visible at script scope after the call.
        assert_eq!(
            vm_stdout(b"<?php function f() { global $g; $g = 42; } f(); echo $g;"),
            b"42"
        );
    }

    #[test]
    fn global_at_script_scope_is_noop() {
        // At script scope the named variable *is* the global, so `global` does
        // nothing and the variable keeps its value.
        assert_eq!(vm_stdout(b"<?php $x = 3; global $x; echo $x;"), b"3");
    }

    // ----- Session A: $GLOBALS['x']->prop writes (field path) vs PHP 8.5.7 -----

    #[test]
    fn globals_property_set() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=1; } $x=new C; $GLOBALS['x']->v = 5; echo $x->v;"),
            b"5"
        );
    }

    #[test]
    fn globals_property_op_set() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=10; } $x=new C; $GLOBALS['x']->v += 3; echo $x->v;"),
            b"13"
        );
    }

    #[test]
    fn globals_property_incdec() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=10; } $x=new C; $GLOBALS['x']->v++; echo $x->v;"),
            b"11"
        );
    }

    #[test]
    fn globals_property_nested_index() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $a=[]; } $x=new C; $GLOBALS['x']->a[0] = 7; echo $x->a[0];"),
            b"7"
        );
    }

    #[test]
    fn globals_property_isset_and_unset() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=10; } $x=new C; echo isset($GLOBALS['x']->v)?'y':'n'; unset($GLOBALS['x']->v); echo isset($GLOBALS['x']->v)?'y':'n';"),
            b"yn"
        );
    }

    // ----- REF-2: by-reference parameters (user functions) -----

    #[test]
    fn by_ref_param_mutates_caller() {
        assert_eq!(
            vm_stdout(b"<?php function inc(&$x) { $x = $x + 1; } $n = 5; inc($n); echo $n;"),
            b"6"
        );
    }

    #[test]
    fn by_ref_param_nonvariable_is_catchable_error() {
        // Passing a literal to a by-ref parameter is a catchable \Error at run
        // time (not a compile rejection), with PHP's exact message.
        assert_eq!(
            vm_stdout(
                b"<?php function inc(&$x){$x++;} \
                  try { inc(5); } catch (\\Error $e) { echo $e->getMessage(); }"
            ),
            b"inc(): Argument #1 ($x) could not be passed by reference"
        );
    }

    // ----- step 14 / 16: scalar parameter & return type hints (vs PHP 8.5.7) -----

    #[test]
    fn scalar_param_hint_coerces_weak() {
        // Weak mode coerces the argument to the declared scalar type; `===` proves
        // the coerced *type*.
        assert_eq!(vm_stdout(b"<?php function f(int $x){ echo $x === 123 ? 'Y':'N'; } f('123');"), b"Y");
        assert_eq!(vm_stdout(b"<?php function f(float $x){ echo $x === 7.0 ? 'Y':'N'; } f(7);"), b"Y");
        assert_eq!(vm_stdout(b"<?php function f(string $x){ echo $x === '42' ? 'Y':'N'; } f(42);"), b"Y");
    }

    #[test]
    fn scalar_param_hint_type_error_message() {
        let out = vm_outcome(b"<?php function f(int $x){ return $x; } f('abc');");
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeError(m))
                if m == "f(): Argument #1 ($x) must be of type int, string given, \
                         called in test.php on line 1 and defined in test.php:1"
        ), "got {:?}", out.fatal);
    }

    #[test]
    fn nullable_param_hint_accepts_null_else_coerces() {
        assert_eq!(vm_stdout(b"<?php function f(?int $x){ echo $x === null ? 'Y':'N'; } f(null);"), b"Y");
        assert_eq!(vm_stdout(b"<?php function f(?int $x){ echo $x === 5 ? 'Y':'N'; } f('5');"), b"Y");
    }

    #[test]
    fn strict_types_rejects_coercion_but_widens_int_to_float() {
        // Under strict_types a numeric string for `int` is a TypeError, but int→float
        // widening is allowed.
        let out = vm_outcome(b"<?php declare(strict_types=1); function f(int $x){} f('5');");
        assert!(matches!(&out.fatal, Some(PhpError::TypeError(m)) if m.contains("must be of type int, string given")));
        assert_eq!(
            vm_stdout(b"<?php declare(strict_types=1); function f(float $x){ echo $x === 5.0 ? 'Y':'N'; } f(5);"),
            b"Y"
        );
    }

    #[test]
    fn default_value_coerced_to_param_hint() {
        // `float $n = 0` stores 0.0 when the default is used (D-NEW-6).
        assert_eq!(vm_stdout(b"<?php function f(float $n = 0){ echo $n === 0.0 ? 'Y':'N'; } f();"), b"Y");
    }

    #[test]
    fn return_type_hint_coerces_and_errors() {
        assert_eq!(vm_stdout(b"<?php function f(): int { return '5'; } echo f() === 5 ? 'Y':'N';"), b"Y");
        let out = vm_outcome(b"<?php function f(): int { return 'x'; } f();");
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeError(m))
                if m == "f(): Return value must be of type int, string returned in test.php:1"
        ), "got {:?}", out.fatal);
    }

    #[test]
    fn engine_type_error_is_catchable() {
        assert_eq!(
            vm_stdout(b"<?php function f(int $x){ return $x; } try { f([]); } catch (TypeError $e) { echo 'T'; }"),
            b"T"
        );
    }

    #[test]
    fn lossy_float_to_int_param_deprecates() {
        // A lossy float→int coercion is a deprecation, not a fatal.
        let out = vm_outcome(b"<?php function f(int $x){} f(3.7);");
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        assert!(rendered_has(&out, b"Implicit conversion from float 3.7 to int loses precision"));
    }

    #[test]
    fn by_ref_param_swap() {
        assert_eq!(
            vm_stdout(b"<?php function swap(&$a, &$b) { $t = $a; $a = $b; $b = $t; } $x = 1; $y = 2; swap($x, $y); echo $x . $y;"),
            b"21"
        );
    }

    #[test]
    fn by_ref_and_by_value_mixed() {
        // The by-ref param writes through; the by-value param is a copy.
        assert_eq!(
            vm_stdout(b"<?php function f(&$r, $v) { $r = $v * 10; $v = 0; } $a = 0; $b = 4; f($a, $b); echo $a . '|' . $b;"),
            b"40|4"
        );
    }

    #[test]
    fn by_value_param_does_not_mutate_caller() {
        assert_eq!(
            vm_stdout(b"<?php function noref($v) { $v = 99; } $a = 5; noref($a); echo $a;"),
            b"5"
        );
    }

    #[test]
    fn by_ref_param_defines_undefined_var() {
        // Passing an undefined variable by reference defines it in the caller.
        assert_eq!(
            vm_stdout(b"<?php function set(&$x) { $x = 7; } set($u); echo $u;"),
            b"7"
        );
    }

    #[test]
    fn same_var_to_two_by_ref_params_shares_cell() {
        assert_eq!(
            vm_stdout(b"<?php function two(&$a, &$b) { $a = 1; $b = 2; } $x = 0; two($x, $x); echo $x;"),
            b"2"
        );
    }

    #[test]
    fn by_ref_propagates_through_nested_calls() {
        // `outer`'s by-ref param is itself passed by-ref to `inner`: the write in
        // `inner` reaches all the way back to the original caller's variable.
        assert_eq!(
            vm_stdout(b"<?php function outer(&$x) { inner($x); } function inner(&$y) { $y = 42; } $n = 0; outer($n); echo $n;"),
            b"42"
        );
    }

    // ----- REF-3: foreach by-reference (`foreach $a as &$v`) -----

    #[test]
    fn foreach_by_ref_mutates_source() {
        assert_eq!(
            vm_stdout(b"<?php $a = [1, 2, 3]; foreach ($a as &$v) { $v = $v * 2; } echo $a[0]; echo $a[1]; echo $a[2];"),
            b"246"
        );
    }

    #[test]
    fn foreach_by_ref_over_temporary_is_tolerated() {
        // PHP does not error on `foreach (<non-lvalue> as &$v)`: it degrades to
        // by-value iteration (the writes land nowhere observable). Must run clean.
        assert_eq!(
            vm_stdout(b"<?php foreach ([1, 2, 3] as &$v) { $v *= 2; } echo 'ok';"),
            b"ok"
        );
    }

    #[test]
    fn foreach_by_ref_with_key() {
        // The key is bound by value while the value aliases the element.
        assert_eq!(
            vm_stdout(b"<?php $a = [10, 20]; foreach ($a as $k => &$v) { $v = $v + $k; } echo $a[0] . ',' . $a[1];"),
            b"10,21"
        );
    }

    #[test]
    fn foreach_by_ref_then_unset_is_safe() {
        // Unsetting the alias after the loop detaches it; a later by-value loop is
        // then unaffected.
        assert_eq!(
            vm_stdout(b"<?php $a = [1, 2, 3]; foreach ($a as &$v) { $v = $v + 10; } unset($v); foreach ($a as $v) {} echo $a[0]; echo $a[1]; echo $a[2];"),
            b"111213"
        );
    }

    #[test]
    fn foreach_by_ref_lingering_reference_gotcha() {
        // The classic PHP gotcha: after a by-ref loop, `$v` still aliases the last
        // element; a following by-value loop overwrites it on each step, leaving
        // the last element equal to the second-to-last (D-R13).
        assert_eq!(
            vm_stdout(b"<?php $a = [1, 2, 3]; foreach ($a as &$v) {} foreach ($a as $v) {} echo $a[0]; echo $a[1]; echo $a[2];"),
            b"122"
        );
    }

    #[test]
    fn foreach_by_ref_empty_array() {
        assert_eq!(
            vm_stdout(b"<?php $a = []; foreach ($a as &$v) { $v = 1; } echo 'done';"),
            b"done"
        );
    }

    #[test]
    fn foreach_by_ref_string_keys() {
        assert_eq!(
            vm_stdout(b"<?php $a = ['x' => 1, 'y' => 2]; foreach ($a as &$v) { $v = $v * 100; } echo $a['x']; echo '-'; echo $a['y'];"),
            b"100-200"
        );
    }

    // ----- REF-4: references into array elements -----

    #[test]
    fn ref_to_array_element_writes_through() {
        // `$x = &$a[0]`: writing $x updates the array element.
        assert_eq!(
            vm_stdout(b"<?php $a = [10, 20]; $x = &$a[0]; $x = 99; echo $a[0]; echo '-'; echo $a[1];"),
            b"99-20"
        );
    }

    #[test]
    fn ref_to_array_element_visible_from_element() {
        // The reverse direction: writing the element updates the alias.
        assert_eq!(
            vm_stdout(b"<?php $a = [1, 2]; $r = &$a[1]; $a[1] = 50; echo $r;"),
            b"50"
        );
    }

    #[test]
    fn array_element_aliases_variable() {
        // `$a[0] = &$x`: the element aliases the (initially undefined) variable.
        assert_eq!(
            vm_stdout(b"<?php $a = [1]; $a[0] = &$x; $x = 7; echo $a[0];"),
            b"7"
        );
    }

    #[test]
    fn ref_between_two_array_elements() {
        // `$a[0] = &$b[1]`: both sides are stepped places.
        assert_eq!(
            vm_stdout(b"<?php $a = [0]; $b = [0, 0]; $a[0] = &$b[1]; $b[1] = 7; echo $a[0];"),
            b"7"
        );
    }

    #[test]
    fn ref_to_nested_array_element() {
        assert_eq!(
            vm_stdout(b"<?php $a = [[1, 2]]; $r = &$a[0][1]; $r = 88; echo $a[0][1];"),
            b"88"
        );
    }

    #[test]
    fn ref_to_array_element_string_key() {
        assert_eq!(
            vm_stdout(b"<?php $a = ['k' => 1]; $r = &$a['k']; $r = 8; echo $a['k'];"),
            b"8"
        );
    }

    #[test]
    fn ref_to_array_element_autovivifies() {
        // Referencing a missing element defines it (NULL) then a write creates it.
        assert_eq!(
            vm_stdout(b"<?php $a = []; $r = &$a[5]; $r = 'hi'; echo $a[5];"),
            b"hi"
        );
    }

    // ----- Session A: references into object properties / `[]` (vs PHP 8.5.7) -----

    #[test]
    fn ref_to_object_property_writes_through() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=1; } $o=new C; $r = &$o->v; $r = 99; echo $o->v;"),
            b"99"
        );
    }

    #[test]
    fn ref_to_appended_element() {
        // `&$a[]` appends a fresh element and aliases it.
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2]; $r = &$a[]; $r = 99; echo $a[0],$a[1],$a[2];"),
            b"1299"
        );
    }

    #[test]
    fn bind_ref_into_object_property() {
        // `$o->v = &$x`: the property aliases the variable's cell.
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=0; } $o=new C; $x=5; $o->v = &$x; $x=42; echo $o->v;"),
            b"42"
        );
    }

    #[test]
    fn ref_to_object_array_element() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $a=[10,20]; } $o=new C; $r=&$o->a[1]; $r=99; echo $o->a[1];"),
            b"99"
        );
    }

    #[test]
    fn append_a_reference_to_array() {
        // `$a[] = &$x`: the appended element aliases the variable.
        assert_eq!(
            vm_stdout(b"<?php $a=[]; $x=7; $a[] = &$x; $x=88; echo $a[0];"),
            b"88"
        );
    }

    // ----- REF-4b: by-reference return (`function &f()`) + `$y = &f()` -----

    #[test]
    fn return_ref_of_by_ref_param_aliases() {
        // `function &id(&$x) { return $x; }` returns a reference to its by-ref
        // param; `$r = &id($a)` aliases the caller's variable.
        assert_eq!(
            vm_stdout(b"<?php function &id(&$x) { return $x; } $a = 5; $r = &id($a); $r = 10; echo $a;"),
            b"10"
        );
    }

    #[test]
    fn return_ref_of_array_element_aliases() {
        // Returning a reference to a by-ref param's array element; writing the
        // alias updates the caller's array.
        assert_eq!(
            vm_stdout(b"<?php function &elem(&$arr, $k) { return $arr[$k]; } $a = [1, 2, 3]; $r = &elem($a, 1); $r = 99; echo $a[0]; echo $a[1]; echo $a[2];"),
            b"1993"
        );
    }

    #[test]
    fn ref_return_in_value_context_copies() {
        // `$y = f()` (no `&`) copies the by-ref return — `DerefTop` — so a later
        // write to $y does not touch the source.
        assert_eq!(
            vm_stdout(b"<?php function &f() { global $g; return $g; } $g = 5; $y = f(); $y = 100; echo $g;"),
            b"5"
        );
    }

    #[test]
    fn ref_return_via_global_aliases() {
        // `$y = &f()` over a by-ref return of a global aliases the global cell.
        assert_eq!(
            vm_stdout(b"<?php function &f() { global $g; return $g; } $g = 1; $y = &f(); $y = 42; echo $g;"),
            b"42"
        );
    }

    // ----- CLO: closures, arrow functions, first-class callables -----

    #[test]
    fn closure_basic_call() {
        assert_eq!(vm_stdout(b"<?php $f = function() { return 42; }; echo $f();"), b"42");
    }

    #[test]
    fn closure_with_params() {
        assert_eq!(
            vm_stdout(b"<?php $add = function($a, $b) { return $a + $b; }; echo $add(2, 3);"),
            b"5"
        );
    }

    #[test]
    fn closure_capture_by_value_snapshots() {
        // `use($x)` snapshots the value at creation; a later write does not change it.
        assert_eq!(
            vm_stdout(b"<?php $x = 10; $f = function() use ($x) { return $x; }; $x = 20; echo $f();"),
            b"10"
        );
    }

    #[test]
    fn closure_capture_by_ref() {
        // `use(&$x)` shares the cell; the closure's write is visible to the caller.
        assert_eq!(
            vm_stdout(b"<?php $x = 10; $f = function() use (&$x) { $x = $x + 5; }; $f(); echo $x;"),
            b"15"
        );
    }

    #[test]
    fn arrow_function_auto_captures() {
        assert_eq!(vm_stdout(b"<?php $y = 7; $f = fn($n) => $n + $y; echo $f(3);"), b"10");
    }

    #[test]
    fn closure_immediately_invoked() {
        assert_eq!(vm_stdout(b"<?php echo (function() { return 'hi'; })();"), b"hi");
    }

    #[test]
    fn closure_returning_closure() {
        assert_eq!(
            vm_stdout(b"<?php $mk = function($s) { return function() use ($s) { return $s; }; }; $c = $mk(9); echo $c();"),
            b"9"
        );
    }

    #[test]
    fn closure_captures_this_in_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v = 5; function mk() { return function() { return $this->v; }; } } $o = new C(); $f = $o->mk(); echo $f();"),
            b"5"
        );
    }

    #[test]
    fn dynamic_string_call_to_user_function() {
        assert_eq!(
            vm_stdout(b"<?php function greet() { return 'hi'; } $f = 'greet'; echo $f();"),
            b"hi"
        );
    }

    #[test]
    fn first_class_callable_of_user_function() {
        assert_eq!(
            vm_stdout(b"<?php function dbl($n) { return $n * 2; } $f = dbl(...); echo $f(21);"),
            b"42"
        );
    }

    #[test]
    fn dynamic_call_to_value_builtin() {
        // `$f = 't_double'; $f(21)` dispatches to the registered value builtin.
        assert_eq!(
            vm_run(b"<?php $f = 't_double'; echo $f(21);", &fake_registry()).stdout,
            b"42"
        );
    }

    // ----- EXC-1: throw + try/catch (user-thrown objects, no finally) -----

    #[test]
    fn try_catch_basic() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('boom'); } catch (Exception $e) { echo $e->getMessage(); }"),
            b"boom"
        );
    }

    #[test]
    fn try_catch_no_throw_skips_handler() {
        assert_eq!(
            vm_stdout(b"<?php try { echo 'body'; } catch (Exception $e) { echo 'no'; } echo '!';"),
            b"body!"
        );
    }

    #[test]
    fn try_catch_resumes_after() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Exception $e) {} echo 'after';"),
            b"after"
        );
    }

    #[test]
    fn try_catch_first_matching_clause_wins() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('a'); } catch (TypeError $e) { echo 'no'; } catch (Exception $e) { echo 'yes'; }"),
            b"yes"
        );
    }

    #[test]
    fn try_catch_variable_less() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Exception) { echo 'caught'; }"),
            b"caught"
        );
    }

    #[test]
    fn try_catch_by_throwable_interface() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Throwable $e) { echo 't'; }"),
            b"t"
        );
    }

    #[test]
    fn exception_propagates_from_called_function() {
        assert_eq!(
            vm_stdout(b"<?php function f() { throw new Exception('deep'); } try { f(); echo 'unreached'; } catch (Exception $e) { echo $e->getMessage(); }"),
            b"deep"
        );
    }

    #[test]
    fn throw_mid_expression_clears_stack() {
        // The partial expression value (`5 +`) is discarded before the catch runs.
        assert_eq!(
            vm_stdout(b"<?php function g() { throw new Exception('e'); } try { $r = 5 + g(); echo 'unreached'; } catch (Exception $e) { echo 'caught'; }"),
            b"caught"
        );
    }

    #[test]
    fn nested_try_inner_rethrows_to_outer() {
        // The inner clause (TypeError) does not match; its Rethrow reaches the
        // outer Exception handler.
        assert_eq!(
            vm_stdout(b"<?php try { try { throw new Exception('x'); } catch (TypeError $e) { echo 'inner'; } } catch (Exception $e) { echo 'outer'; }"),
            b"outer"
        );
    }

    #[test]
    fn uncaught_exception_is_fatal() {
        // No matching clause anywhere: the run reports a fatal (not a panic).
        let program = lower_source(b"test.php", b"<?php throw new Exception('nope');").expect("lower");
        let module = compile_program(&program, &Registry::new()).expect("compile");
        let out = run_module(&module, &Registry::new());
        assert!(out.fatal.is_some(), "expected an uncaught-exception fatal");
    }

    // ----- EXC-2: finally -----

    #[test]
    fn finally_runs_after_normal_completion() {
        assert_eq!(
            vm_stdout(b"<?php try { echo 'a'; } finally { echo 'b'; } echo 'c';"),
            b"abc"
        );
    }

    #[test]
    fn finally_runs_after_caught() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Exception $e) { echo 'caught'; } finally { echo 'fin'; } echo '!';"),
            b"caughtfin!"
        );
    }

    #[test]
    fn finally_runs_while_exception_propagates() {
        assert_eq!(
            vm_stdout(b"<?php function f() { try { throw new Exception('x'); } finally { echo 'fin'; } } try { f(); } catch (Exception $e) { echo 'outer'; }"),
            b"finouter"
        );
    }

    #[test]
    fn finally_runs_then_completes_return() {
        // `return` in try runs the finally, then the return completes (EXC-2b).
        assert_eq!(
            vm_stdout(b"<?php function f(){ try { return 't'; } finally { echo 'f'; } } echo f();"),
            b"ft"
        );
    }

    #[test]
    fn finally_return_overrides_try_return_and_exception() {
        // A `return` in finally wins over a try-side return…
        assert_eq!(
            vm_stdout(b"<?php function f(){ try { return 'try'; } finally { return 'fin'; } } echo f();"),
            b"fin"
        );
        // …and swallows an in-flight exception.
        assert_eq!(
            vm_stdout(b"<?php function f(){ try { throw new Exception('x'); } finally { return 'ok'; } } echo f();"),
            b"ok"
        );
    }

    #[test]
    fn finally_runs_then_completes_break_and_continue() {
        // break/continue crossing a finally run it first, then transfer (EXC-2b).
        assert_eq!(
            vm_stdout(b"<?php for($i=0;$i<3;$i++){ try { if($i==1) break; echo $i; } finally { echo 'f'; } }"),
            b"0ff"
        );
        assert_eq!(
            vm_stdout(b"<?php for($i=0;$i<3;$i++){ try { if($i==1) continue; echo $i; } finally { echo 'f'; } }"),
            b"0ff2f"
        );
    }

    #[test]
    fn finally_runs_when_clause_does_not_match() {
        assert_eq!(
            vm_stdout(b"<?php try { try { throw new Exception('x'); } catch (TypeError $e) { echo 'no'; } finally { echo 'fin'; } } catch (Exception $e) { echo 'out'; }"),
            b"finout"
        );
    }

    #[test]
    fn nested_finally_both_run() {
        assert_eq!(
            vm_stdout(b"<?php function f() { try { try { throw new Exception('x'); } finally { echo '1'; } } finally { echo '2'; } } try { f(); } catch (Exception $e) { echo '3'; }"),
            b"123"
        );
    }

    #[test]
    fn exception_in_finally_overrides_original() {
        assert_eq!(
            vm_stdout(b"<?php try { try { throw new Exception('a'); } finally { throw new Exception('b'); } } catch (Exception $e) { echo $e->getMessage(); }"),
            b"b"
        );
    }

    #[test]
    fn finally_pending_does_not_leak_to_next_try() {
        // After a finally re-throws and is caught, a *later* normally-completing
        // try/finally must not re-raise the stale parked exception.
        assert_eq!(
            vm_stdout(b"<?php try { try { throw new Exception('a'); } finally { throw new Exception('b'); } } catch (Exception $e) {} try { echo 'x'; } finally { echo 'y'; } echo 'z';"),
            b"xyz"
        );
    }

    // ----- EXC-3a: engine errors are catchable -----

    #[test]
    fn division_by_zero_error_catchable() {
        // `1 % 0` raises a DivisionByZeroError, synthesized into a Throwable and
        // routed to the matching `catch`; its message round-trips via getMessage.
        assert_eq!(
            vm_stdout(b"<?php try { $x = 1 % 0; } catch (DivisionByZeroError $e) { echo $e->getMessage(); }"),
            b"Modulo by zero"
        );
    }

    #[test]
    fn divide_by_zero_error_catchable() {
        assert_eq!(
            vm_stdout(b"<?php try { $x = 1 / 0; } catch (DivisionByZeroError $e) { echo $e->getMessage(); }"),
            b"Division by zero"
        );
    }

    #[test]
    fn engine_error_caught_by_supertype() {
        // DivisionByZeroError extends ArithmeticError: a clause for the supertype
        // catches it (instance-of is interface/parent-aware).
        assert_eq!(
            vm_stdout(b"<?php try { $x = 1 % 0; } catch (ArithmeticError $e) { echo 'arith'; }"),
            b"arith"
        );
    }

    #[test]
    fn type_error_catchable() {
        // `[] + 1` raises a TypeError (unsupported operand types).
        assert_eq!(
            vm_stdout(b"<?php try { $x = [] + 1; } catch (TypeError $e) { echo 'type'; }"),
            b"type"
        );
    }

    #[test]
    fn type_error_caught_as_error() {
        // TypeError extends Error: a `catch (Error)` clause catches it.
        assert_eq!(
            vm_stdout(b"<?php try { $x = [] + 1; } catch (Error $e) { echo 'err'; }"),
            b"err"
        );
    }

    #[test]
    fn engine_error_caught_by_throwable() {
        assert_eq!(
            vm_stdout(b"<?php try { $x = 1 % 0; } catch (Throwable $e) { echo 'caught'; }"),
            b"caught"
        );
    }

    #[test]
    fn error_base_catchable() {
        // Instantiating an abstract class raises a plain Error, caught here.
        assert_eq!(
            vm_stdout(b"<?php abstract class A {} try { $a = new A(); } catch (Error $e) { echo $e->getMessage(); }"),
            b"Cannot instantiate abstract class A"
        );
    }

    #[test]
    fn engine_error_non_matching_clause_propagates() {
        // A clause for an unrelated type does not catch the engine error: it
        // keeps unwinding and the run reports a fatal (not a panic).
        let program = lower_source(b"test.php", b"<?php try { $x = 1 % 0; } catch (ValueError $e) { echo 'no'; }").expect("lower");
        let module = compile_program(&program, &Registry::new()).expect("compile");
        let out = run_module(&module, &Registry::new());
        assert!(out.fatal.is_some(), "expected an uncaught engine-error fatal");
    }

    #[test]
    fn uncaught_engine_error_is_fatal() {
        let program = lower_source(b"test.php", b"<?php $x = 1 % 0;").expect("lower");
        let module = compile_program(&program, &Registry::new()).expect("compile");
        let out = run_module(&module, &Registry::new());
        assert!(out.fatal.is_some(), "expected an uncaught engine-error fatal");
    }

    // ----- EXC-3b: line / file tracking -----

    #[test]
    fn new_exception_carries_line() {
        // PHP fixes a Throwable's line at `new` time: the `throw new Exception`
        // sits on source line 3.
        assert_eq!(
            vm_stdout(b"<?php\ntry {\n    throw new Exception('boom');\n} catch (Exception $e) {\n    echo $e->getLine();\n}"),
            b"3"
        );
    }

    #[test]
    fn new_exception_carries_file() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Exception $e) { echo $e->getFile(); }"),
            b"test.php"
        );
    }

    #[test]
    fn engine_error_carries_line() {
        // The synthesized DivisionByZeroError reports the line of the faulting
        // `1 % 0` op (source line 3).
        assert_eq!(
            vm_stdout(b"<?php\ntry {\n    $x = 1 % 0;\n} catch (DivisionByZeroError $e) {\n    echo $e->getLine();\n}"),
            b"3"
        );
    }

    #[test]
    fn engine_error_carries_file() {
        assert_eq!(
            vm_stdout(b"<?php try { $x = 1 % 0; } catch (DivisionByZeroError $e) { echo $e->getFile(); }"),
            b"test.php"
        );
    }

    #[test]
    fn exception_line_is_new_site_not_construct() {
        // A Throwable's line is fixed at the `new` site (source line 3), not at
        // the later `make()` call site (line 5) that returns it.
        assert_eq!(
            vm_stdout(b"<?php\nfunction make() {\n    return new Exception('e');\n}\n$e = make();\necho $e->getLine();"),
            b"3"
        );
    }

    // ----- EXC-3c: stack trace -----

    #[test]
    fn trace_string_function_chain() {
        // `a()` is called from `b` (line 3); `b()` from main (line 5). The trace
        // is byte-identical to the tree-walker's (verified against `eval::run`).
        assert_eq!(
            vm_stdout(b"<?php\nfunction a() { throw new Exception('x'); }\nfunction b() { a(); }\ntry {\n    b();\n} catch (Exception $e) {\n    echo $e->getTraceAsString();\n}"),
            b"#0 test.php(3): a()\n#1 test.php(5): b()\n#2 {main}"
        );
    }

    #[test]
    fn trace_string_method_chain() {
        // Instance call renders `C->m`, static call `C::s`.
        assert_eq!(
            vm_stdout(b"<?php\nclass C {\n    function m() { throw new Exception('x'); }\n    static function s() { (new C)->m(); }\n}\ntry {\n    C::s();\n} catch (Exception $e) {\n    echo $e->getTraceAsString();\n}"),
            b"#0 test.php(4): C->m()\n#1 test.php(7): C::s()\n#2 {main}"
        );
    }

    #[test]
    fn trace_string_engine_error() {
        // A synthesized engine error captures the trace at the faulting site:
        // `d()` called from main (line 4) — matching real PHP. (The tree-walker
        // synthesizes lazily *after* unwinding and so reports an empty trace
        // here; the VM is intentionally more faithful to PHP on this point.)
        assert_eq!(
            vm_stdout(b"<?php\nfunction d() { $x = 1 % 0; }\ntry {\n    d();\n} catch (DivisionByZeroError $e) {\n    echo $e->getTraceAsString();\n}"),
            b"#0 test.php(4): d()\n#1 {main}"
        );
    }

    #[test]
    fn trace_string_top_level_throw_is_main_only() {
        assert_eq!(
            vm_stdout(b"<?php try { throw new Exception('x'); } catch (Exception $e) { echo $e->getTraceAsString(); }"),
            b"#0 {main}"
        );
    }

    #[test]
    fn trace_array_shape_function() {
        // getTrace()[0] carries function / line / file for a free-function frame
        // (no class/type keys). `count` is an evaluator-only builtin, so index
        // the array directly.
        assert_eq!(
            vm_stdout(b"<?php\nfunction a() { throw new Exception('x'); }\ntry {\n    a();\n} catch (Exception $e) {\n    $t = $e->getTrace();\n    echo $t[0]['function'], '|', $t[0]['line'], '|', $t[0]['file'];\n}"),
            b"a|4|test.php"
        );
    }

    #[test]
    fn trace_array_shape_method() {
        // A method frame additionally carries class and type (`->` / `::`).
        assert_eq!(
            vm_stdout(b"<?php\nclass C {\n    function m() { throw new Exception('x'); }\n}\ntry {\n    (new C)->m();\n} catch (Exception $e) {\n    $t = $e->getTrace();\n    echo $t[0]['class'], $t[0]['type'], $t[0]['function'];\n}"),
            b"C->m"
        );
    }

    // ----- GEN-1: generators (yield, foreach, current/key/next/valid/rewind) -----
    // Expected outputs verified byte-for-byte against PHP 8.5.7 CLI.

    #[test]
    fn generator_foreach_values() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; yield 2; yield 3; } foreach (g() as $v) echo $v;"),
            b"123"
        );
    }

    #[test]
    fn generator_foreach_keyed() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 'a'=>1; yield 'b'=>2; } foreach (g() as $k=>$v) echo \"$k=$v;\";"),
            b"a=1;b=2;"
        );
    }

    #[test]
    fn generator_auto_keys() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 10; yield 20; } foreach (g() as $k=>$v) echo \"$k=$v;\";"),
            b"0=10;1=20;"
        );
    }

    #[test]
    fn generator_mixed_keys_resume_counter() {
        // An explicit integer key bumps the auto-key counter (5 → next auto 6).
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 5=>'a'; yield 'b'; } foreach (g() as $k=>$v) echo \"$k=$v;\";"),
            b"5=a;6=b;"
        );
    }

    #[test]
    fn generator_code_between_yields_runs_in_order() {
        // Code between yields runs lazily, interleaved with the foreach body.
        assert_eq!(
            vm_stdout(b"<?php function g(){ echo 'a'; yield 1; echo 'b'; yield 2; echo 'c'; } foreach(g() as $v) echo $v;"),
            b"a1b2c"
        );
    }

    #[test]
    fn generator_methods_current_next_valid() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 7; yield 8; } $x=g(); echo $x->current(); $x->next(); echo $x->current(); echo $x->valid()?'Y':'N'; $x->next(); echo $x->valid()?'Y':'N';"),
            b"78YN"
        );
    }

    #[test]
    fn generator_key_method() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 'x'=>9; } $i=g(); echo $i->key(), $i->current();"),
            b"x9"
        );
    }

    #[test]
    fn closure_generator() {
        assert_eq!(
            vm_stdout(b"<?php $g = function(){ yield 1; yield 2; }; foreach ($g() as $v) echo $v;"),
            b"12"
        );
    }

    #[test]
    fn generator_send_value_via_yield_expression() {
        // The `yield` expression evaluates to NULL under `next()`/`foreach`
        // (send arrives in GEN-2); here it is discarded, exercising `$x = yield`.
        assert_eq!(
            vm_stdout(b"<?php function g(){ $a = yield 1; yield 2; } foreach (g() as $v) echo $v;"),
            b"12"
        );
    }

    // ----- GEN-2: send / return / getReturn (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn generator_send_ping_pong() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ $x = yield 1; echo \"got:$x;\"; $y = yield 2; echo \"got:$y;\"; } $gen=g(); echo $gen->current(); echo $gen->send('A'); echo $gen->send('B');"),
            b"1got:A;2got:B;"
        );
    }

    #[test]
    fn generator_send_on_fresh_primes_then_delivers() {
        // `send` on a NotStarted generator primes to the first yield, then
        // delivers the value as that yield's result.
        assert_eq!(
            vm_stdout(b"<?php function g(){ $x = yield 1; echo \"x=$x;\"; yield 2; } $g=g(); echo $g->send('Z');"),
            b"x=Z;2"
        );
    }

    #[test]
    fn generator_get_return() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; yield 2; return 42; } $g=g(); foreach($g as $v) echo $v; echo '|', $g->getReturn();"),
            b"12|42"
        );
    }

    #[test]
    fn generator_return_bare_is_null() {
        // A bare `return;` leaves getReturn() NULL, which echoes as empty.
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; return; } $g=g(); foreach($g as $v) echo $v; echo '[', $g->getReturn(), ']';"),
            b"1[]"
        );
    }

    #[test]
    fn generator_return_without_yield() {
        // A body that returns before any yield: getReturn auto-primes it.
        assert_eq!(
            vm_stdout(b"<?php function g(){ if(false) yield; return 99; } $g=g(); echo $g->getReturn();"),
            b"99"
        );
    }

    #[test]
    fn generator_get_return_too_early_throws_exception() {
        // PHP raises a plain `Exception` here (the tree-walker raises `Error`).
        // The `catch (Exception)` arm firing (not `catch (Error)`) proves the class.
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; return 5; } $g=g(); try { $g->getReturn(); } catch (Exception $e) { echo 'Exception:', $e->getMessage(); } catch (Error $e) { echo 'Error'; }"),
            b"Exception:Cannot get return value of a generator that hasn't returned"
        );
    }

    #[test]
    fn generator_rewind_after_run_throws_exception() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; yield 2; } $g=g(); $g->next(); try { $g->rewind(); } catch (Exception $e) { echo 'Exception:', $e->getMessage(); } catch (Error $e) { echo 'Error'; }"),
            b"Exception:Cannot rewind a generator that was already run"
        );
    }

    // ----- GEN-3: yield from (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn yield_from_array_keeps_keys_and_counter() {
        // Array keys re-yielded verbatim; the outer auto-key counter is NOT
        // advanced, so the trailing `yield 3` is key 0.
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield from [1,2]; yield 3; } foreach(g() as $k=>$v) echo \"$k:$v;\";"),
            b"0:1;1:2;0:3;"
        );
    }

    #[test]
    fn yield_from_assoc_array() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield from ['x'=>1, 'y'=>2]; } foreach(g() as $k=>$v) echo \"$k:$v;\";"),
            b"x:1;y:2;"
        );
    }

    #[test]
    fn yield_from_subgenerator() {
        assert_eq!(
            vm_stdout(b"<?php function inner(){ yield 'a'; yield 'b'; } function outer(){ yield from inner(); yield 'c'; } foreach(outer() as $k=>$v) echo \"$k:$v;\";"),
            b"0:a;1:b;0:c;"
        );
    }

    #[test]
    fn yield_from_return_value() {
        // The `yield from` expression evaluates to the sub-generator's return.
        assert_eq!(
            vm_stdout(b"<?php function inner(){ yield 1; return 99; } function outer(){ $r = yield from inner(); echo \"r=$r;\"; } foreach(outer() as $v) echo $v;"),
            b"1r=99;"
        );
    }

    #[test]
    fn yield_from_forwards_send() {
        // `send()` on the outer is delivered to the suspended `yield` in the inner.
        assert_eq!(
            vm_stdout(b"<?php function inner(){ $x = yield 1; echo \"inner:$x;\"; yield 2; } function outer(){ yield from inner(); } $g=outer(); echo $g->current(); echo $g->send('S');"),
            b"1inner:S;2"
        );
    }

    #[test]
    fn yield_from_nested() {
        assert_eq!(
            vm_stdout(b"<?php function a(){ yield 1; yield 2; } function b(){ yield from a(); yield 3; } function c(){ yield from b(); yield 4; } foreach(c() as $v) echo $v;"),
            b"1234"
        );
    }

    // ----- GEN-4: Fiber (net-new; verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn fiber_basic_start_resume() {
        // start() runs to the first suspend (returning its value); resume()
        // delivers a value as that suspend's result and runs on.
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function(){ echo 'A'; $x = Fiber::suspend('s1'); echo \"B$x\"; }); $v = $f->start(); echo \"[$v]\"; $f->resume('R'); echo 'end';"),
            b"A[s1]BRend"
        );
    }

    #[test]
    fn fiber_get_return() {
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function(){ Fiber::suspend(1); return 42; }); $f->start(); $f->resume(); echo $f->getReturn();"),
            b"42"
        );
    }

    #[test]
    fn fiber_nested_suspend() {
        // Fiber::suspend called from a nested function call parks the whole
        // frame segment (not just one frame).
        assert_eq!(
            vm_stdout(b"<?php function deep(){ Fiber::suspend('deep'); } $f = new Fiber(function(){ echo 'x'; deep(); echo 'y'; }); echo $f->start(); $f->resume(); echo 'z';"),
            b"xdeepyz"
        );
    }

    #[test]
    fn fiber_status_flags() {
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function(){ Fiber::suspend(); }); echo $f->isStarted()?1:0; $f->start(); echo $f->isSuspended()?1:0; echo $f->isTerminated()?1:0; $f->resume(); echo $f->isTerminated()?1:0;"),
            b"0101"
        );
    }

    #[test]
    fn fiber_get_current() {
        assert_eq!(
            vm_stdout(b"<?php echo Fiber::getCurrent()===null?'out-null;':'out-x;'; $f=new Fiber(function(){ echo Fiber::getCurrent() instanceof Fiber ? 'in-fiber;':'in-no;'; }); $f->start();"),
            b"out-null;in-fiber;"
        );
    }

    #[test]
    fn fiber_start_args() {
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function($a,$b){ echo $a+$b; }); $f->start(3,4);"),
            b"7"
        );
    }

    #[test]
    fn fiber_exception_escapes_to_caller() {
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function(){ throw new Exception('boom'); }); try { $f->start(); } catch (Exception $e) { echo 'caught:', $e->getMessage(); }"),
            b"caught:boom"
        );
    }

    #[test]
    fn fiber_multi_suspend_ping_pong() {
        assert_eq!(
            vm_stdout(b"<?php $f = new Fiber(function(){ $a = Fiber::suspend(1); $b = Fiber::suspend($a+1); return $b+1; }); echo $f->start(); echo $f->resume(10); echo $f->resume(20); echo $f->getReturn();"),
            b"11121"
        );
    }

    #[test]
    fn fiber_suspend_outside_is_error() {
        assert_eq!(
            vm_stdout(b"<?php try { Fiber::suspend(1); } catch (\\Throwable $e) { echo 'err'; }"),
            b"err"
        );
    }

    // ----- PAR: default parameters + arity (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn default_param_omitted_and_given() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b=5){ return $a+$b; } echo f(1), ',', f(1,2);"),
            b"6,3"
        );
    }

    #[test]
    fn default_param_expression() {
        assert_eq!(
            vm_stdout(b"<?php function greet($name, $greeting='Hello'){ return \"$greeting, $name\"; } echo greet('X');"),
            b"Hello, X"
        );
    }

    #[test]
    fn extra_args_dropped() {
        // A non-variadic function silently ignores surplus positional arguments.
        assert_eq!(
            vm_stdout(b"<?php function f($a){ return $a; } echo f(7, 8, 9);"),
            b"7"
        );
    }

    #[test]
    fn method_default_param() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($x, $y=10){ return $x*$y; } } $o=new C; echo $o->m(3), ',', $o->m(3,2);"),
            b"30,6"
        );
    }

    #[test]
    fn default_references_earlier_param() {
        // The default runs in the callee frame, so it can see earlier params.
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b=null){ $b = $b ?? $a*2; return $b; } echo f(4);"),
            b"8"
        );
    }

    #[test]
    fn constructor_default_param() {
        assert_eq!(
            vm_stdout(b"<?php class P { public $v; function __construct($v=99){ $this->v=$v; } } $p=new P; echo $p->v; $q=new P(7); echo $q->v;"),
            b"997"
        );
    }

    #[test]
    fn closure_default_param() {
        assert_eq!(
            vm_stdout(b"<?php $f = function($a, $b=3){ return $a+$b; }; echo $f(10);"),
            b"13"
        );
    }

    // ----- PAR: variadic parameters (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn variadic_collects_all_args() {
        assert_eq!(
            vm_stdout(b"<?php function sum(...$n){ $s=0; foreach($n as $x) $s+=$x; return $s; } echo sum(1,2,3,4);"),
            b"10"
        );
    }

    #[test]
    fn variadic_after_fixed_param() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, ...$rest){ $s=$a; foreach($rest as $x) $s.=$x; return $s; } echo f('x',1,2,3);"),
            b"x123"
        );
    }

    #[test]
    fn variadic_empty_when_no_extra_args() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, ...$rest){ $c=0; foreach($rest as $x) $c++; return \"$a:$c\"; } echo f('x');"),
            b"x:0"
        );
    }

    #[test]
    fn variadic_array_is_indexable_with_int_keys() {
        assert_eq!(
            vm_stdout(b"<?php function f(...$a){ return $a[0].'-'.$a[2]; } echo f(10,20,30);"),
            b"10-30"
        );
    }

    #[test]
    fn variadic_array_keys_are_sequential() {
        assert_eq!(
            vm_stdout(b"<?php function f(...$a){ $s=''; foreach($a as $k=>$v) $s.=\"$k:$v;\"; return $s; } echo f('a','b');"),
            b"0:a;1:b;"
        );
    }

    #[test]
    fn variadic_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($x, ...$ys){ $s=$x; foreach($ys as $y) $s+=$y; return $s; } } echo (new C)->m(10,1,2,3);"),
            b"16"
        );
    }

    #[test]
    fn variadic_with_default_in_between() {
        // `f($a, $b=1, ...$rest)`: defaults fill, then the rest collects.
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b=1, ...$rest){ $c=0; foreach($rest as $r) $c++; return \"$a-$b-$c\"; } echo f(5),'|',f(5,6),'|',f(5,6,7,8);"),
            b"5-1-0|5-6-0|5-6-2"
        );
    }

    // ----- PAR: dynamic class references (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn new_dynamic_class_from_string() {
        assert_eq!(
            vm_stdout(b"<?php class Foo { function __construct(){ echo 'made;'; } function n(){ return 'Foo'; } } $c='Foo'; $o=new $c; echo $o->n();"),
            b"made;Foo"
        );
    }

    #[test]
    fn new_dynamic_class_then_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { function hi(){ return 'hi'; } } $c='C'; echo (new $c)->hi();"),
            b"hi"
        );
    }

    #[test]
    fn new_dynamic_class_with_args_and_default() {
        assert_eq!(
            vm_stdout(b"<?php class P { public $v; function __construct($a,$b=2){ $this->v=$a+$b; } } $c='P'; echo (new $c(10))->v;"),
            b"12"
        );
    }

    #[test]
    fn new_dynamic_from_object_reuses_class() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $v=3; } $o=new C; $o2=new $o; echo $o2->v;"),
            b"3"
        );
    }

    #[test]
    fn new_dynamic_strips_leading_backslash() {
        assert_eq!(
            vm_stdout(b"<?php class C { function n(){ return 'C'; } } $c='\\C'; echo (new $c)->n();"),
            b"C"
        );
    }

    #[test]
    fn new_dynamic_unknown_class_errors() {
        assert_eq!(
            vm_stdout(b"<?php $c='Nope'; try { new $c; } catch(Error $e){ echo 'Error:', $e->getMessage(); }"),
            b"Error:Class \"Nope\" not found"
        );
    }

    #[test]
    fn new_dynamic_scalar_errors() {
        assert_eq!(
            vm_stdout(b"<?php $c=5; try { new $c; } catch(Error $e){ echo $e->getMessage(); }"),
            b"Class name must be a valid object or a string"
        );
    }

    #[test]
    fn instanceof_dynamic_class() {
        assert_eq!(
            vm_stdout(b"<?php class A{} class B extends A{} $cls='A'; $b=new B; echo ($b instanceof $cls)?'Y':'N'; $cls2='B'; echo (new A instanceof $cls2)?'Y':'N';"),
            b"YN"
        );
    }

    #[test]
    fn instanceof_dynamic_unknown_is_false() {
        assert_eq!(
            vm_stdout(b"<?php $cls='Nope'; echo (5 instanceof $cls)?'Y':'N';"),
            b"N"
        );
    }

    // ----- PAR: dynamic static calls $cls::method() (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn dynamic_static_call_basic() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s($x){ return \"S$x\"; } } $c='C'; echo $c::s(7);"),
            b"S7"
        );
    }

    #[test]
    fn dynamic_static_call_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class A { static function who(){ return 'A'; } } class B extends A {} $c='B'; echo $c::who();"),
            b"A"
        );
    }

    #[test]
    fn dynamic_static_call_with_default_arg() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s($x, $y=5){ return $x+$y; } } $c='C'; echo $c::s(10);"),
            b"15"
        );
    }

    #[test]
    fn dynamic_static_call_variadic() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s(...$a){ $t=0; foreach($a as $x) $t+=$x; return $t; } } $c='C'; echo $c::s(1,2,3);"),
            b"6"
        );
    }

    #[test]
    fn dynamic_static_call_callstatic() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function __callStatic($n,$a){ return $n.':'.$a[0].$a[1]; } } $c='C'; echo $c::ghost(1,2);"),
            b"ghost:12"
        );
    }

    #[test]
    fn dynamic_static_call_via_object() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s(){ return 'ok'; } } $o=new C; echo $o::s();"),
            b"ok"
        );
    }

    #[test]
    fn dynamic_static_call_unknown_class_errors() {
        assert_eq!(
            vm_stdout(b"<?php $c='Nope'; try { $c::s(); } catch(Error $e){ echo 'Error:', $e->getMessage(); }"),
            b"Error:Class \"Nope\" not found"
        );
    }

    // ----- PAR: dynamic class constants $cls::CONST / $cls::class -----

    #[test]
    fn dynamic_class_const() {
        assert_eq!(
            vm_stdout(b"<?php class C { const K = 42; } $c='C'; echo $c::K;"),
            b"42"
        );
    }

    #[test]
    fn dynamic_class_const_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class A { const K='ak'; } class B extends A {} $c='B'; echo $c::K;"),
            b"ak"
        );
    }

    #[test]
    fn dynamic_class_const_undefined_errors() {
        assert_eq!(
            vm_stdout(b"<?php class C{} $c='C'; try { echo $c::NOPE; } catch(Error $e){ echo 'Error:', $e->getMessage(); }"),
            b"Error:Undefined constant C::NOPE"
        );
    }

    #[test]
    fn dynamic_class_const_unknown_class_errors() {
        assert_eq!(
            vm_stdout(b"<?php $c='Nope'; try { echo $c::K; } catch(Error $e){ echo $e->getMessage(); }"),
            b"Class \"Nope\" not found"
        );
    }

    #[test]
    fn dynamic_class_name_on_object() {
        assert_eq!(
            vm_stdout(b"<?php class C{} $o=new C; echo $o::class;"),
            b"C"
        );
    }

    #[test]
    fn dynamic_class_name_on_string_is_type_error() {
        // PHP 8: `$str::class` is a TypeError (only objects work dynamically).
        assert_eq!(
            vm_stdout(b"<?php $c='C'; class C{} try { echo $c::class; } catch(TypeError $e){ echo 'TE:', $e->getMessage(); }"),
            b"TE:Cannot use \"::class\" on string"
        );
    }

    // ----- PAR: (float) and (array) casts (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn float_cast_string() {
        assert_eq!(vm_stdout(b"<?php echo (float)'3.14';"), b"3.14");
    }

    #[test]
    fn float_cast_in_arithmetic() {
        assert_eq!(vm_stdout(b"<?php echo (float)'2e3' + 1;"), b"2001");
    }

    #[test]
    fn float_cast_int_prints_without_point() {
        assert_eq!(vm_stdout(b"<?php echo (float)10;"), b"10");
    }

    #[test]
    fn array_cast_scalar_wraps() {
        assert_eq!(vm_stdout(b"<?php $a=(array)5; echo $a[0];"), b"5");
    }

    #[test]
    fn array_cast_string_wraps() {
        assert_eq!(vm_stdout(b"<?php $a=(array)'hi'; echo $a[0];"), b"hi");
    }

    #[test]
    fn array_cast_null_is_empty() {
        assert_eq!(
            vm_stdout(b"<?php $a=(array)null; $c=0; foreach($a as $x) $c++; echo $c;"),
            b"0"
        );
    }

    #[test]
    fn array_cast_array_passes_through() {
        assert_eq!(vm_stdout(b"<?php $a=(array)[1,2,3]; echo $a[0],$a[2];"), b"13");
    }

    #[test]
    fn object_cast_scalar() {
        assert_eq!(vm_stdout(b"<?php $o=(object)5; echo $o->scalar;"), b"5");
    }

    #[test]
    fn object_cast_assoc_array() {
        assert_eq!(
            vm_stdout(b"<?php $o=(object)['a'=>1,'b'=>2]; echo $o->a, $o->b;"),
            b"12"
        );
    }

    #[test]
    fn object_cast_object_passes_through() {
        assert_eq!(
            vm_stdout(b"<?php class C{public $v=7;} $c=new C; $o=(object)$c; echo $o->v;"),
            b"7"
        );
    }

    #[test]
    fn object_cast_null_is_empty_stdclass() {
        assert_eq!(
            vm_stdout(b"<?php $o=(object)null; echo ($o instanceof stdClass)?'Y':'N';"),
            b"Y"
        );
    }

    // ----- PAR: ArgumentCountError (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn too_few_args_function() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b){ return $a+$b; } try { f(1); } catch(ArgumentCountError $e){ echo $e->getMessage(); }"),
            b"Too few arguments to function f(), 1 passed in test.php on line 1 and exactly 2 expected"
        );
    }

    #[test]
    fn too_few_args_zero_passed() {
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(); } catch(ArgumentCountError $e){ echo $e->getMessage(); }"),
            b"Too few arguments to function f(), 0 passed in test.php on line 1 and exactly 1 expected"
        );
    }

    #[test]
    fn too_few_args_method() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b,$c){} } try { (new C)->m(1); } catch(ArgumentCountError $e){ echo $e->getMessage(); }"),
            b"Too few arguments to function C::m(), 1 passed in test.php on line 1 and exactly 3 expected"
        );
    }

    #[test]
    fn too_few_args_at_least_with_optional() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b,$c=3){} try { f(1); } catch(ArgumentCountError $e){ echo $e->getMessage(); }"),
            b"Too few arguments to function f(), 1 passed in test.php on line 1 and at least 2 expected"
        );
    }

    #[test]
    fn enough_args_no_error() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b){ return $a+$b; } echo f(1,2);"),
            b"3"
        );
    }

    #[test]
    fn argument_count_error_is_a_type_error() {
        // ArgumentCountError extends TypeError, so a TypeError clause catches it.
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(); } catch(TypeError $e){ echo 'caught'; }"),
            b"caught"
        );
    }

    // ----- PAR: named arguments (function calls; verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn named_args_reordered() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b){ return \"$a-$b\"; } echo f(b: 2, a: 1);"),
            b"1-2"
        );
    }

    #[test]
    fn named_args_skip_optional() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, $b=1, $c=2){ return \"$a-$b-$c\"; } echo f(1, c: 3);"),
            b"1-1-3"
        );
    }

    #[test]
    fn named_args_mixed_positional_and_named() {
        assert_eq!(
            vm_stdout(b"<?php function f($x, $y, $z=9){ return \"$x$y$z\"; } echo f(1, z: 7, y: 2);"),
            b"127"
        );
    }

    #[test]
    fn named_args_all_named() {
        assert_eq!(
            vm_stdout(b"<?php function greet($greeting, $name){ return \"$greeting, $name!\"; } echo greet(name: 'X', greeting: 'Hi');"),
            b"Hi, X!"
        );
    }

    // ----- PAR: dynamic static property $cls::$prop (verified vs PHP 8.5.7) -----

    #[test]
    fn dynamic_static_prop_get() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $x = 5; } $c='C'; echo $c::$x;"),
            b"5"
        );
    }

    #[test]
    fn dynamic_static_prop_set() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $x = 5; } $c='C'; $c::$x = 20; echo C::$x;"),
            b"20"
        );
    }

    #[test]
    fn dynamic_static_prop_opset() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $x = 5; } $c='C'; $c::$x += 3; echo $c::$x;"),
            b"8"
        );
    }

    #[test]
    fn dynamic_static_prop_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class A { public static $v = 'av'; } class B extends A {} $c='B'; echo $c::$v;"),
            b"av"
        );
    }

    #[test]
    fn dynamic_static_prop_via_object() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $x = 7; } $o=new C; echo $o::$x;"),
            b"7"
        );
    }

    // ----- Session A: ++/-- and ??= on a dynamic-class static property -----

    #[test]
    fn dynamic_static_prop_post_incr() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = 5; } $c='C'; echo $c::$p++; echo '|'; echo C::$p;"),
            b"5|6"
        );
    }

    #[test]
    fn dynamic_static_prop_pre_incr() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = 5; } $c='C'; echo ++$c::$p; echo '|'; echo C::$p;"),
            b"6|6"
        );
    }

    #[test]
    fn dynamic_static_prop_post_decr() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = 3; } $c='C'; echo $c::$p--; echo '|'; echo C::$p;"),
            b"3|2"
        );
    }

    #[test]
    fn dynamic_static_prop_coalesce_assigns_when_null() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = null; } $c='C'; $c::$p ??= 7; echo C::$p;"),
            b"7"
        );
    }

    #[test]
    fn dynamic_static_prop_coalesce_keeps_when_set() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = 4; } $c='C'; $c::$p ??= 7; echo C::$p;"),
            b"4"
        );
    }

    #[test]
    fn dynamic_static_prop_coalesce_skips_rhs_when_set() {
        // The rhs (which would mutate `C::$log`) must not run when `$p` is set.
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p=4; public static $log='no'; } $c='C'; $c::$p ??= (C::$log='ran'); echo C::$p, '|', C::$log;"),
            b"4|no"
        );
    }

    #[test]
    fn dynamic_static_prop_incr_via_object() {
        assert_eq!(
            vm_stdout(b"<?php class C { public static $p = 1; } $o=new C; echo $o::$p++; echo '|'; echo C::$p;"),
            b"1|2"
        );
    }

    // ----- PAR: named arguments on new / static (known class) vs PHP 8.5.7 -----

    #[test]
    fn named_args_new() {
        assert_eq!(
            vm_stdout(b"<?php class P { public $v; function __construct($a, $b){ $this->v=\"$a-$b\"; } } echo (new P(b: 2, a: 1))->v;"),
            b"1-2"
        );
    }

    #[test]
    fn named_args_new_skip_optional() {
        assert_eq!(
            vm_stdout(b"<?php class P { public $v; function __construct($a, $b=5){ $this->v=\"$a-$b\"; } } echo (new P(a: 7))->v;"),
            b"7-5"
        );
    }

    #[test]
    fn named_args_static_call() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s($x, $y){ return \"$x/$y\"; } } echo C::s(y: 2, x: 1);"),
            b"1/2"
        );
    }

    #[test]
    fn named_args_static_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class A { static function s($a, $b){ return \"$a$b\"; } } class B extends A {} echo B::s(b: 'Y', a: 'X');"),
            b"XY"
        );
    }

    #[test]
    fn named_args_self_static_call() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function make($a,$b){ return \"$a:$b\"; } static function go(){ return self::make(b: 2, a: 1); } } echo C::go();"),
            b"1:2"
        );
    }

    // ----- PAR: argument unpacking f(...$arr) (verified vs PHP 8.5.7 CLI) -----

    #[test]
    fn spread_call_fills_positional() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b,$c){ return \"$a$b$c\"; } echo f(...[1,2,3]);"),
            b"123"
        );
    }

    #[test]
    fn spread_call_mixed_with_positional() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b,$c){ return \"$a$b$c\"; } echo f(1, ...[2,3]);"),
            b"123"
        );
    }

    #[test]
    fn spread_call_of_variable() {
        assert_eq!(
            vm_stdout(b"<?php $arr=[5,6]; function f($a,$b){ return $a+$b; } echo f(...$arr);"),
            b"11"
        );
    }

    #[test]
    fn spread_call_into_variadic() {
        assert_eq!(
            vm_stdout(b"<?php function sum(...$n){ $s=0; foreach($n as $x) $s+=$x; return $s; } echo sum(...[1,2,3,4]);"),
            b"10"
        );
    }

    #[test]
    fn spread_call_with_default() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b=9){ return \"$a-$b\"; } echo f(...[1]);"),
            b"1-9"
        );
    }

    #[test]
    fn spread_call_of_generator() {
        assert_eq!(
            vm_stdout(b"<?php function g(){ yield 1; yield 2; } function f($a,$b){ return $a+$b; } echo f(...g());"),
            b"3"
        );
    }

    // ----- Session A: spread on method / new / static (vs PHP 8.5.7 CLI) -----

    #[test]
    fn spread_method_fills_positional() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b,$c){ return \"$a$b$c\"; } } echo (new C)->m(...[1,2,3]);"),
            b"123"
        );
    }

    #[test]
    fn spread_method_mixed_with_positional() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b,$c){ return \"$a$b$c\"; } } echo (new C)->m(1, ...[2,3]);"),
            b"123"
        );
    }

    #[test]
    fn spread_method_into_variadic() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m(...$n){ $s=0; foreach($n as $x) $s+=$x; return $s; } } echo (new C)->m(...[1,2,3,4]);"),
            b"10"
        );
    }

    #[test]
    fn spread_method_nullsafe_on_present_receiver() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b){ return $a+$b; } } $o=new C; echo $o?->m(...[10,20]);"),
            b"30"
        );
    }

    #[test]
    fn spread_static_call() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function m($a,$b){ return $a+$b; } } echo C::m(...[5,6]);"),
            b"11"
        );
    }

    #[test]
    fn spread_static_call_dynamic_class() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function m($a,$b){ return $a*$b; } } $c='C'; echo $c::m(...[3,4]);"),
            b"12"
        );
    }

    #[test]
    fn spread_new_with_constructor() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $s; function __construct($a,$b){ $this->s=\"$a-$b\"; } } echo (new C(...[1,2]))->s;"),
            b"1-2"
        );
    }

    #[test]
    fn spread_new_dynamic_class() {
        assert_eq!(
            vm_stdout(b"<?php class C { public $s; function __construct($a,$b){ $this->s=$a+$b; } } $c='C'; $o=new $c(...[4,5]); echo $o->s;"),
            b"9"
        );
    }

    #[test]
    fn spread_new_static_preserves_lsb() {
        // `new static(...$a)` allocates the late-static-bound class and spreads.
        assert_eq!(
            vm_stdout(b"<?php class C { public $s; function __construct($a){ $this->s=$a; } static function make(...$a){ return new static(...$a); } } echo C::make(7)->s;"),
            b"7"
        );
    }

    // ----- Session A: named arguments on instance methods (vs PHP 8.5.7 CLI) -----

    #[test]
    fn named_method_skips_optional() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b=7,$c=9){ return \"$a-$b-$c\"; } } echo (new C)->m(1, c:3);"),
            b"1-7-3"
        );
    }

    #[test]
    fn named_method_all_reordered() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b){ return \"$a:$b\"; } } echo (new C)->m(b:2, a:1);"),
            b"1:2"
        );
    }

    #[test]
    fn named_method_mixed_positional_and_named() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b,$c){ return \"$a$b$c\"; } } echo (new C)->m(1, c:3, b:2);"),
            b"123"
        );
    }

    #[test]
    fn named_method_into_variadic() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a, ...$r){ $s=$a; foreach($r as $k=>$v) $s.=\";$k=$v\"; return $s; } } echo (new C)->m(1, x:2, y:3);"),
            b"1;x=2;y=3"
        );
    }

    #[test]
    fn named_method_nullsafe() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b){ return $a+$b; } } $o=new C; echo $o?->m(b:20, a:10);"),
            b"30"
        );
    }

    #[test]
    fn named_method_inherited() {
        // The defining class (P) resolves the parameter names at run time.
        assert_eq!(
            vm_stdout(b"<?php class P { function m($a,$b){ return \"$a/$b\"; } } class C extends P {} echo (new C)->m(b:2, a:1);"),
            b"1/2"
        );
    }

    #[test]
    fn named_method_missing_required() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a,$b){} } try { (new C)->m(a:1); } catch(ArgumentCountError $e){ echo $e->getMessage(); }"),
            b"Too few arguments to function C::m(), 1 passed in test.php on line 1 and exactly 2 expected"
        );
    }

    #[test]
    fn named_method_unknown_parameter() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a){} } try { (new C)->m(z:1); } catch(\\Error $e){ echo $e->getMessage(); }"),
            b"Unknown named parameter $z"
        );
    }

    #[test]
    fn named_method_overwrites_positional() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($a){} } try { (new C)->m(1, a:2); } catch(\\Error $e){ echo $e->getMessage(); }"),
            b"Named parameter $a overwrites previous argument"
        );
    }

    #[test]
    fn named_method_routes_to_call_magic() {
        // `__call` collects named arguments into its `$args` array (string keys).
        assert_eq!(
            vm_stdout(b"<?php class C { function __call($n,$a){ $s=$n; foreach($a as $k=>$v) $s.=\";$k=$v\"; return $s; } } echo (new C)->missing(x:1, y:2);"),
            b"missing;x=1;y=2"
        );
    }

    // ----- Session A: enum cases via `::` (vs PHP 8.5.7 CLI) -----

    #[test]
    fn enum_pure_case_name() {
        assert_eq!(
            vm_stdout(b"<?php enum Suit { case Hearts; case Spades; } echo Suit::Hearts->name;"),
            b"Hearts"
        );
    }

    #[test]
    fn enum_backed_case_value() {
        assert_eq!(
            vm_stdout(b"<?php enum E: int { case A = 1; case B = 2; } echo E::B->value;"),
            b"2"
        );
    }

    #[test]
    fn enum_case_value_from_inherited_interface_const() {
        // A backed case value may reference an inherited interface constant
        // (I::A) or self:: (resolving through the implemented interface).
        assert_eq!(
            vm_stdout(
                b"<?php interface I { const A = 'A'; const B = 'B'; } \
                  enum E: string implements I { case C = I::A; case D = self::B; } \
                  echo E::A, E::B, E::C->value, E::D->value;"
            ),
            b"ABAB"
        );
    }

    #[test]
    fn enum_backed_string_case() {
        assert_eq!(
            vm_stdout(b"<?php enum E: string { case A = 'x'; } echo E::A->name,'/',E::A->value;"),
            b"A/x"
        );
    }

    #[test]
    fn enum_case_is_a_singleton() {
        assert_eq!(
            vm_stdout(b"<?php enum E { case A; } echo E::A === E::A ? 'y':'n';"),
            b"y"
        );
    }

    #[test]
    fn enum_case_instanceof() {
        assert_eq!(
            vm_stdout(b"<?php enum E { case A; } echo (E::A instanceof E) ? 'y':'n';"),
            b"y"
        );
    }

    #[test]
    fn enum_case_identity_comparison() {
        assert_eq!(
            vm_stdout(b"<?php enum E { case A; case B; } $x = E::A; echo $x === E::A ? 'same':'diff'; echo $x === E::B ? 'x':'-';"),
            b"same-"
        );
    }

    #[test]
    fn enum_case_method_uses_value() {
        assert_eq!(
            vm_stdout(b"<?php enum E: int { case A = 1; case B = 2; function label(): string { return 'case-'.$this->value; } } echo E::B->label();"),
            b"case-2"
        );
    }

    #[test]
    fn enum_case_in_match() {
        assert_eq!(
            vm_stdout(b"<?php enum E { case A; case B; } function f($e){ return match($e){ E::A=>'a', E::B=>'b' }; } echo f(E::B);"),
            b"b"
        );
    }

    // ----- Session B1: call_user_func* / is_callable (vs PHP 8.5.7 CLI) -----

    #[test]
    fn cuf_closure() {
        assert_eq!(vm_stdout(b"<?php echo call_user_func(function($x){ return $x*2; }, 5);"), b"10");
    }

    #[test]
    fn cuf_user_function() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b){ return $a+$b; } echo call_user_func('f', 3, 4);"),
            b"7"
        );
    }

    #[test]
    fn cuf_array_with_values() {
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b){ return $a*$b; } echo call_user_func_array('f', [6, 7]);"),
            b"42"
        );
    }

    #[test]
    fn cuf_instance_method_callable() {
        assert_eq!(
            vm_stdout(b"<?php class C { function m($x){ return $x+1; } } echo call_user_func([new C, 'm'], 10);"),
            b"11"
        );
    }

    #[test]
    fn cuf_static_array_callable() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s($x){ return $x*3; } } echo call_user_func(['C','s'], 4);"),
            b"12"
        );
    }

    #[test]
    fn cuf_static_string_callable() {
        assert_eq!(
            vm_stdout(b"<?php class C { static function s($x){ return $x*3; } } echo call_user_func('C::s', 4);"),
            b"12"
        );
    }

    #[test]
    fn cuf_invoke_object() {
        assert_eq!(
            vm_stdout(b"<?php class C { function __invoke($x){ return $x*5; } } $o=new C; echo call_user_func($o, 6);"),
            b"30"
        );
    }

    #[test]
    fn cuf_nested_recursion() {
        // The callback re-enters `call_callable` (nested `run_loop`).
        assert_eq!(
            vm_stdout(b"<?php function fact($n){ return $n<=1 ? 1 : $n * call_user_func('fact', $n-1); } echo call_user_func('fact', 5);"),
            b"120"
        );
    }

    #[test]
    fn cuf_exception_propagates_through_host() {
        // An exception thrown in the callback unwinds out through the host builtin.
        assert_eq!(
            vm_stdout(b"<?php try { call_user_func(function(){ throw new Exception('boom'); }); } catch(Exception $e){ echo 'caught:'.$e->getMessage(); }"),
            b"caught:boom"
        );
    }

    #[test]
    fn cuf_exception_caught_inside_callback() {
        // A `try/catch` *inside* the callback resumes there (unwind floor=baseline).
        assert_eq!(
            vm_stdout(b"<?php echo call_user_func(function(){ try { throw new Exception('x'); } catch(Exception $e){ return 'inner'; } });"),
            b"inner"
        );
    }

    // ----- Session B3: define / defined / constant (vs PHP 8.5.7 CLI) -----

    #[test]
    fn define_and_read_user_constant() {
        assert_eq!(vm_stdout(b"<?php define('FOO', 42); echo FOO;"), b"42");
    }

    #[test]
    fn constant_reads_user_and_engine() {
        assert_eq!(
            vm_stdout(b"<?php define('FOO', 'hi'); echo constant('FOO'),'|',constant('PHP_INT_SIZE');"),
            b"hi|8"
        );
    }

    #[test]
    fn defined_user_unknown_and_engine() {
        assert_eq!(
            vm_stdout(b"<?php define('FOO', 1); echo defined('FOO')?1:0, defined('BAR')?1:0, defined('PHP_INT_MAX')?1:0;"),
            b"101"
        );
    }

    #[test]
    fn redefine_constant_warns_and_keeps_first() {
        // The redefinition fails (false), the original value is kept, and the PHP
        // 8.5 deprecation warning is rendered inline.
        let out = vm_outcome(b"<?php define('FOO', 1); echo (define('FOO', 2))?'t':'f','|',FOO;");
        assert_eq!(out.stdout, b"f|1");
        assert!(
            out.rendered.windows(b"Constant FOO already defined, this will be an error in PHP 9".len())
                .any(|w| w == b"Constant FOO already defined, this will be an error in PHP 9"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn undefined_constant_read_is_a_fatal() {
        let out = vm_outcome(b"<?php echo NOPE;");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"Undefined constant \"NOPE\"".len())
                .any(|w| w == b"Undefined constant \"NOPE\""),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn constant_of_undefined_name_errors() {
        let out = vm_outcome(b"<?php echo constant('NOPE');");
        assert!(out.fatal.is_some());
    }

    #[test]
    fn is_callable_various() {
        // Closure / instance-method array / static array → true; a plain object
        // (no __invoke) and a missing method → false. (Registry builtins aren't
        // registered in this harness, so string-function names are tested via the
        // corpus, not here.)
        assert_eq!(
            vm_stdout(b"<?php class C { function m(){} static function s(){} } echo (is_callable(function(){})?1:0), (is_callable([new C,'m'])?1:0), (is_callable(['C','s'])?1:0), (is_callable(new C)?1:0), (is_callable([new C,'nope'])?1:0);"),
            b"11100"
        );
    }

    // ----- Session C: array_map / array_filter / array_reduce (vs PHP 8.5.7) -----

    #[test]
    fn array_map_single_preserves_keys() {
        assert_eq!(
            vm_stdout(b"<?php $r=array_map(fn($x)=>$x*$x,[1,2,3]); echo $r[0],$r[1],$r[2];"),
            b"149"
        );
    }

    #[test]
    fn array_map_string_callable() {
        assert_eq!(
            vm_stdout(b"<?php function dbl($x){ return $x*2; } $r=array_map('dbl',[1,2,3]); echo $r[0],$r[1],$r[2];"),
            b"246"
        );
    }

    #[test]
    fn array_map_multi_reindexes_and_pads() {
        // Several arrays: re-index 0.., one element from each per row, NULL tails.
        assert_eq!(
            vm_stdout(b"<?php $r=array_map(fn($a,$b)=>$a+$b,[1,2,3],[10,20,30,40]); echo $r[0],'-',$r[1],'-',$r[2],'-',$r[3];"),
            b"11-22-33-40"
        );
    }

    #[test]
    fn array_map_null_callback_zips() {
        assert_eq!(
            vm_stdout(b"<?php $r=array_map(null,[1,2],[3,4]); echo $r[0][0],$r[0][1],$r[1][0],$r[1][1];"),
            b"1324"
        );
    }

    #[test]
    fn array_filter_no_callback_keeps_truthy() {
        // Keys are preserved; the falsy 0 entries at keys 0 and 3 are dropped.
        assert_eq!(
            vm_stdout(b"<?php $r=array_filter([0,1,2,0,3]); echo $r[1],$r[2],$r[4],(isset($r[0])?'y':'n'),(isset($r[3])?'y':'n');"),
            b"123nn"
        );
    }

    #[test]
    fn array_filter_use_key() {
        assert_eq!(
            vm_stdout(b"<?php $r=array_filter(['a'=>1,'b'=>2,'c'=>3],fn($k)=>$k!=='b',2); echo $r['a'],$r['c'],(isset($r['b'])?'y':'n');"),
            b"13n"
        );
    }

    #[test]
    fn array_filter_use_both() {
        // mode 1 = ARRAY_FILTER_USE_BOTH: keep even values regardless of key.
        assert_eq!(
            vm_stdout(b"<?php $r=array_filter([10,11,12,13],fn($v,$k)=>$v%2===0,1); echo $r[0],$r[2],(isset($r[1])?'y':'n'),(isset($r[3])?'y':'n');"),
            b"1012nn"
        );
    }

    #[test]
    fn array_reduce_sum_and_concat() {
        assert_eq!(
            vm_stdout(b"<?php echo array_reduce([1,2,3,4],fn($c,$i)=>$c+$i,0),'|',array_reduce([1,2,3],fn($c,$i)=>$c.$i,'x');"),
            b"10|x123"
        );
    }

    #[test]
    fn array_reduce_empty_returns_initial_null() {
        assert_eq!(
            vm_stdout(b"<?php echo (array_reduce([],fn($c,$i)=>$c+$i)===null)?'N':'V';"),
            b"N"
        );
    }

    #[test]
    fn usort_sorts_and_reindexes() {
        // Out-of-order keys 5/2/9 collapse to 0/1/2; usort returns true.
        assert_eq!(
            vm_stdout(b"<?php $a=[5=>'x',2=>'y',9=>'z']; $r=usort($a,fn($p,$q)=>$p<=>$q); echo ($r?'T':'F'),$a[0],$a[1],$a[2];"),
            b"Txyz"
        );
    }

    #[test]
    fn usort_string_callback_descending() {
        assert_eq!(
            vm_stdout(b"<?php function cmp($x,$y){ return $y-$x; } $a=[3,1,2]; usort($a,'cmp'); echo $a[0],$a[1],$a[2];"),
            b"321"
        );
    }

    #[test]
    fn usort_is_stable() {
        // Equal weights keep their original order (b before a), like PHP 8's sort.
        assert_eq!(
            vm_stdout(b"<?php $a=[['n'=>'b','w'=>1],['n'=>'a','w'=>1],['n'=>'c','w'=>0]]; usort($a,fn($x,$y)=>$x['w']<=>$y['w']); echo $a[0]['n'],$a[1]['n'],$a[2]['n'];"),
            b"cba"
        );
    }

    #[test]
    fn usort_empty_returns_true() {
        assert_eq!(
            vm_stdout(b"<?php $a=[]; echo (usort($a,fn($x,$y)=>0)?'T':'F'),(isset($a[0])?'y':'n');"),
            b"Tn"
        );
    }

    #[test]
    fn array_walk_by_value_visits_key_and_value() {
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2,3]; array_walk($a, function($v,$k){ echo $k,'=',$v,' '; });"),
            b"0=1 1=2 2=3 "
        );
    }

    #[test]
    fn array_walk_by_ref_mutates_in_place() {
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2,3]; array_walk($a, function(&$v,$k){ $v=$v*10; }); echo $a[0],$a[1],$a[2];"),
            b"102030"
        );
    }

    #[test]
    fn array_walk_by_value_does_not_mutate_and_returns_true() {
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2]; $r=array_walk($a, function($v,$k){ $v=99; }); echo ($r?'T':'F'),$a[0],$a[1];"),
            b"T12"
        );
    }

    #[test]
    fn array_walk_extra_arg_by_ref() {
        assert_eq!(
            vm_stdout(b"<?php $a=['x'=>1,'y'=>2]; array_walk($a, function(&$v,$k,$p){ $v=$k.$v.$p; }, '!'); echo $a['x'],'|',$a['y'];"),
            b"x1!|y2!"
        );
    }

    #[test]
    fn array_walk_named_by_ref_function() {
        assert_eq!(
            vm_stdout(b"<?php function addk(&$v,$k){ $v=$v+$k; } $a=[10,20,30]; array_walk($a,'addk'); echo $a[0],$a[1],$a[2];"),
            b"102132"
        );
    }

    // ----- Array internal pointer: reset/end/next/prev/current/key (vs PHP 8.5.7) -----

    #[test]
    fn array_pointer_basic_movement() {
        // current=10, next=20, next=30, next past end=false, key=null, end=30,
        // prev=20, reset=10. Matches the oracle byte-for-byte.
        assert_eq!(
            vm_stdout(b"<?php $a=[10,20,30]; echo current($a),next($a),next($a),(next($a)===false?'F':'?'),(key($a)===null?'N':'?'),end($a),prev($a),reset($a);"),
            b"102030FN302010"
        );
    }

    #[test]
    fn array_pointer_string_keys() {
        assert_eq!(
            vm_stdout(b"<?php $a=['x'=>1,'y'=>2]; echo key($a),'=',current($a); next($a); echo key($a),'=',current($a);"),
            b"x=1y=2"
        );
    }

    #[test]
    fn array_pointer_empty_array() {
        assert_eq!(
            vm_stdout(b"<?php $a=[]; echo (current($a)===false?'F':'?'),(key($a)===null?'N':'?'),(reset($a)===false?'F':'?'),(end($a)===false?'F':'?'),(next($a)===false?'F':'?');"),
            b"FNFFF"
        );
    }

    #[test]
    fn array_pointer_skips_tombstones() {
        // unset leaves a tombstone; reset/next skip over it.
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2,3,4]; unset($a[1]); echo reset($a),next($a),next($a),(next($a)===false?'F':'?');"),
            b"134F"
        );
    }

    #[test]
    fn array_pointer_advances_when_pointed_entry_unset() {
        // The pointer is on value 2 (key 1); unsetting it makes the next live entry
        // current (Zend advances the pointer): current=3, key=2.
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2,3]; next($a); unset($a[1]); echo current($a),'|',key($a);"),
            b"3|2"
        );
    }

    #[test]
    fn array_pointer_untouched_by_foreach() {
        // foreach snapshots (PHP 8) — the internal pointer stays at the first entry.
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2,3]; foreach($a as $v){} echo current($a);"),
            b"1"
        );
    }

    #[test]
    fn array_pointer_prev_before_start_invalidates() {
        assert_eq!(
            vm_stdout(b"<?php $a=[1,2]; reset($a); echo (prev($a)===false?'F':'?'),(current($a)===false?'F':'?');"),
            b"FF"
        );
    }

    #[test]
    fn array_pointer_non_array_is_type_error() {
        let out = vm_outcome(b"<?php $x=5; next($x);");
        assert!(out.fatal.is_some(), "next() on a non-array must be a TypeError");
        assert!(
            out.rendered.windows(b"must be of type array, int given".len())
                .any(|w| w == b"must be of type array, int given"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- unserialize (vs PHP 8.5.7 CLI) -----

    #[test]
    fn unserialize_scalars() {
        assert_eq!(vm_stdout(b"<?php echo unserialize('i:42;');"), b"42");
        assert_eq!(vm_stdout(b"<?php echo unserialize('b:1;')?'T':'F';"), b"T");
        assert_eq!(vm_stdout(b"<?php echo unserialize('s:3:\"abc\";');"), b"abc");
        assert_eq!(vm_stdout(b"<?php echo (unserialize('N;')===null)?'N':'?';"), b"N");
    }

    #[test]
    fn unserialize_array_mixed_keys() {
        assert_eq!(
            vm_stdout(b"<?php $a=unserialize('a:2:{i:0;i:10;s:1:\"k\";i:20;}'); echo $a[0],'|',$a['k'];"),
            b"10|20"
        );
    }

    #[test]
    fn unserialize_object_known_class() {
        // Props are set directly; the constructor is not run. get_class round-trips.
        assert_eq!(
            vm_stdout(b"<?php class P { public $x=0; public $y=0; } $o=unserialize('O:1:\"P\":2:{s:1:\"x\";i:1;s:1:\"y\";i:2;}'); echo get_class($o),':',$o->x,$o->y;"),
            b"P:12"
        );
    }

    #[test]
    fn unserialize_unknown_class_falls_back_to_stdclass() {
        // D-50 scope-out: unknown class → stdClass (PHP makes __PHP_Incomplete_Class).
        assert_eq!(
            vm_stdout(b"<?php $o=unserialize('O:3:\"Zzz\":1:{s:1:\"a\";i:9;}'); echo get_class($o),':',$o->a;"),
            b"stdClass:9"
        );
    }

    #[test]
    fn unserialize_malformed_returns_false_with_warning() {
        let out = vm_outcome(b"<?php echo unserialize('garbage')===false?'F':'?';");
        assert_eq!(out.stdout, b"F");
        assert!(
            out.rendered.windows(b"unserialize(): Error at offset 0 of 7 bytes".len())
                .any(|w| w == b"unserialize(): Error at offset 0 of 7 bytes"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- preg_replace / preg_quote (vs PHP 8.5.7 CLI) -----

    #[test]
    fn preg_replace_basic_and_backref() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace('/\\d+/', 'N', 'a1b22c333');"),
            b"aNbNcN"
        );
        // Backreferences in the replacement ($2$1).
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace('/(\\w)(\\d)/', '$2$1', 'a1b2');"),
            b"1a2b"
        );
    }

    #[test]
    fn preg_replace_bad_pattern_is_null() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace('/[/', 'x', 'abc') === null ? 'NULL' : '?';"),
            b"NULL"
        );
    }

    #[test]
    fn preg_quote_escapes_metachars_and_delimiter() {
        assert_eq!(vm_stdout(b"<?php echo preg_quote('a.b*c+');"), b"a\\.b\\*c\\+");
        assert_eq!(vm_stdout(b"<?php echo preg_quote('a/b', '/');"), b"a\\/b");
    }

    #[test]
    fn preg_split_basic_delim_and_lookahead() {
        // Plain split, zero-width lookahead split, and PREG_SPLIT_NO_EMPTY (=1).
        assert_eq!(
            vm_stdout(b"<?php $p = preg_split('/,/', 'a,b,c'); echo $p[0], $p[1], $p[2];"),
            b"abc"
        );
        assert_eq!(
            vm_stdout(b"<?php $p = preg_split('/(?=,)/', 'a,b,c'); echo $p[0], '~', $p[1], '~', $p[2];"),
            b"a~,b~,c"
        );
        assert_eq!(
            vm_stdout(b"<?php $p = preg_split('/,/', 'a,,b', -1, 1); echo $p[0], $p[1], isset($p[2]) ? 'X' : '.';"),
            b"ab."
        );
    }

    #[test]
    fn preg_split_bad_pattern_is_false() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_split('/[/', 'abc') === false ? 'FALSE' : '?';"),
            b"FALSE"
        );
    }

    #[test]
    fn preg_match_writes_matches_out_param() {
        // The by-reference $matches out-param is written back: [0]=whole, [n]=group.
        assert_eq!(
            vm_stdout(b"<?php $n=preg_match('/(\\d)(\\d)/', 'a12b', $m); echo $n,'|',$m[0],'|',$m[1],'|',$m[2];"),
            b"1|12|1|2"
        );
    }

    #[test]
    fn preg_match_no_match_and_named_group() {
        // No match: returns 0, $matches emptied.
        assert_eq!(
            vm_stdout(b"<?php $n=preg_match('/x/', 'abc', $m); echo $n,'|',($m===[]?'E':'?');"),
            b"0|E"
        );
        // Named group is keyed by name and by index.
        assert_eq!(
            vm_stdout(b"<?php preg_match('/(?<y>\\d+)/', 'n42', $m); echo $m['y'],'|',$m[1];"),
            b"42|42"
        );
    }

    #[test]
    fn preg_match_two_arg_form_no_out_param() {
        // Omitting $matches is allowed (out_slot = None).
        assert_eq!(vm_stdout(b"<?php echo preg_match('/b/', 'abc');"), b"1");
    }

    #[test]
    fn preg_match_all_pattern_order() {
        // $m[0] = whole-match column, $m[1] = group-1 column, across all 3 matches.
        assert_eq!(
            vm_stdout(b"<?php $n=preg_match_all('/(\\d)/', '1a2b3', $m); echo $n,'|',$m[0][0],$m[0][1],$m[0][2],'|',$m[1][0],$m[1][1],$m[1][2];"),
            b"3|123|123"
        );
    }

    #[test]
    fn preg_match_bad_pattern_is_false() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_match('/[/', 'x', $m) === false ? 'F' : '?';"),
            b"F"
        );
    }

    // ----- debug_backtrace / debug_print_backtrace (vs PHP 8.5.7 CLI) -----

    #[test]
    fn debug_print_backtrace_functions() {
        // Call-site lines: b() is called on line 2 (inside a), a() on line 4.
        let src = b"<?php\nfunction a() { b(); }\nfunction b() { debug_print_backtrace(); }\na();\n";
        assert_eq!(
            vm_stdout(src),
            b"#0 test.php(2): b()\n#1 test.php(4): a()\n"
        );
    }

    #[test]
    fn debug_print_backtrace_args_and_method() {
        // Arg formatting: int literal, single-quoted string, Array, and a method
        // call rendered `Class->method`.
        let src = b"<?php\nclass C { function m($n, $s, $arr) { debug_print_backtrace(); } }\n(new C)->m(7, 'hi', [1,2]);\n";
        assert_eq!(
            vm_stdout(src),
            b"#0 test.php(3): C->m(7, 'hi', Array)\n"
        );
    }

    #[test]
    fn debug_backtrace_array_fields() {
        let src = b"<?php\nfunction a($x) { $bt = debug_backtrace(); echo $bt[0]['function'],'@',$bt[0]['line'],'|',$bt[0]['args'][0]; }\na(99);\n";
        assert_eq!(vm_stdout(src), b"a@3|99");
    }

    // ----- Session B2a: get_class / get_parent_class (vs PHP 8.5.7 CLI) -----

    #[test]
    fn get_class_of_object_and_closure() {
        assert_eq!(vm_stdout(b"<?php class C{} echo get_class(new C),'|',get_class(function(){});"), b"C|Closure");
    }

    #[test]
    fn get_class_no_arg_uses_this_and_deprecates() {
        let out = vm_outcome(b"<?php class C{ function w(){ return get_class(); } } echo (new C)->w();");
        assert_eq!(out.stdout, b"C");
        assert!(
            out.rendered.windows(b"Calling get_class() without arguments is deprecated".len())
                .any(|w| w == b"Calling get_class() without arguments is deprecated"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn get_class_non_object_is_type_error() {
        let out = vm_outcome(b"<?php get_class(5);");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be of type object, int given".len())
                .any(|w| w == b"must be of type object, int given"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn get_parent_class_object_string_and_none() {
        assert_eq!(
            vm_stdout(b"<?php class A{} class B extends A{} echo get_parent_class(new B),'|',get_parent_class('B'),'|',(get_parent_class(new A)===false?'F':'?');"),
            b"A|A|F"
        );
    }

    #[test]
    fn get_parent_class_no_arg_uses_current_class() {
        assert_eq!(
            vm_stdout(b"<?php class A{} class B extends A{ function p(){ return get_parent_class(); } } echo (new B)->p();"),
            b"A"
        );
    }

    #[test]
    fn get_parent_class_unresolved_string_is_type_error() {
        let out = vm_outcome(b"<?php get_parent_class('Nope');");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be an object or a valid class name".len())
                .any(|w| w == b"must be an object or a valid class name"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- Session B2b: get_object_vars / get_class_methods (vs PHP 8.5.7 CLI) -----

    #[test]
    fn get_object_vars_from_outside_only_public() {
        // public x + dynamic dyn visible; protected y / private z hidden.
        assert_eq!(
            vm_stdout(b"<?php class C{ public $x=1; protected $y=2; private $z=3; } $o=new C; $o->dyn=9; $v=get_object_vars($o); echo $v['x'],$v['dyn'],(isset($v['y'])?'y':'n'),(isset($v['z'])?'z':'n');"),
            b"19nn"
        );
    }

    #[test]
    fn get_object_vars_from_inside_sees_all() {
        assert_eq!(
            vm_stdout(b"<?php class C{ public $x=1; protected $y=2; private $z=3; function d(){ $v=get_object_vars($this); return $v['x'].$v['y'].$v['z']; } } echo (new C)->d();"),
            b"123"
        );
    }

    #[test]
    fn get_object_vars_non_object_is_type_error() {
        let out = vm_outcome(b"<?php get_object_vars(5);");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be of type object, int given".len())
                .any(|w| w == b"must be of type object, int given"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn get_class_methods_outside_child_then_parent_visible() {
        // B::d first, then A::a (public); A::b (protected) is hidden from outside.
        assert_eq!(
            vm_stdout(b"<?php class A{ public function a(){} protected function b(){} } class B extends A{ public function d(){} } $s=''; foreach(get_class_methods('B') as $n){ $s.=$n.' '; } echo $s;"),
            b"d a "
        );
    }

    #[test]
    fn get_class_methods_inside_sees_private_protected() {
        assert_eq!(
            vm_stdout(b"<?php class A{ public function a(){} protected function b(){} private function c(){} function ms(){ $s=''; foreach(get_class_methods($this) as $n){ $s.=$n; } return $s; } } echo (new A)->ms();"),
            b"abcms"
        );
    }

    #[test]
    fn get_class_methods_unresolved_string_is_type_error() {
        let out = vm_outcome(b"<?php get_class_methods('Nope');");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be an object or a valid class name".len())
                .any(|w| w == b"must be an object or a valid class name"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- Session D1: func_num_args / func_get_args / func_get_arg (vs 8.5.7) -----

    #[test]
    fn func_num_args_counts_passed_not_declared() {
        assert_eq!(vm_stdout(b"<?php function f($a,$b=0){ return func_num_args(); } echo f(1,2,3,4),'|',f(7);"), b"4|1");
    }

    #[test]
    fn func_get_args_reflects_current_param_and_extras() {
        // $a reassigned to 99 shows through; extra args 2,3 recovered from snapshot.
        assert_eq!(
            vm_stdout(b"<?php function f($a){ $a=99; $r=func_get_args(); return $r[0].'-'.$r[1].'-'.$r[2]; } echo f(1,2,3);"),
            b"99-2-3"
        );
    }

    #[test]
    fn func_get_args_variadic_is_flat() {
        assert_eq!(
            vm_stdout(b"<?php function f($a, ...$rest){ $r=func_get_args(); return $r[0].$r[1].$r[2].$r[3]; } echo f(10,20,30,40);"),
            b"10203040"
        );
    }

    #[test]
    fn func_get_arg_returns_position() {
        assert_eq!(vm_stdout(b"<?php function f(){ return func_get_arg(1); } echo f('x','y','z');"), b"y");
    }

    #[test]
    fn func_get_arg_out_of_range_is_value_error() {
        let out = vm_outcome(b"<?php function g(){ return func_get_arg(5); } g(1);");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be less than the number of the arguments".len())
                .any(|w| w == b"must be less than the number of the arguments"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn func_num_args_global_scope_is_fatal() {
        let out = vm_outcome(b"<?php func_num_args();");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be called from a function context".len())
                .any(|w| w == b"must be called from a function context"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- Session D2: sprintf/printf object __toString resolution -----
    // (The real format engine isn't linkable here; `t_sprintf` stands in and
    // observes that `ho_format` resolved object arguments before the engine ran.)

    #[test]
    fn format_resolves_object_via_tostring() {
        let reg = fake_registry();
        let out = vm_run(
            b"<?php class P { function __toString(){ return 'OBJ'; } } echo sprintf('%s', new P());",
            &reg,
        );
        assert_eq!(out.stdout, b"OBJ");
    }

    #[test]
    fn format_passes_scalars_through() {
        let reg = fake_registry();
        let out = vm_run(b"<?php echo sprintf('%s', 42, 'x');", &reg);
        assert_eq!(out.stdout, b"42x");
    }

    #[test]
    fn format_resolves_object_nested_in_array() {
        // An object inside an array argument is resolved too (recursive).
        let reg = fake_registry();
        let out = vm_run(
            b"<?php class P { function __toString(){ return 'Z'; } } $a=[new P()]; echo sprintf('%s', $a[0]);",
            &reg,
        );
        assert_eq!(out.stdout, b"Z");
    }

    #[test]
    fn format_object_without_tostring_is_fatal() {
        let reg = fake_registry();
        let program = lower_source(b"test.php", b"<?php class Q {} echo sprintf('%s', new Q());").expect("lower");
        let module = compile_program(&program, &reg).expect("compile");
        let out = run_module(&module, &reg);
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"could not be converted to string".len())
                .any(|w| w == b"could not be converted to string"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- Session B4: *_exists / get_called_class predicates (vs PHP 8.5.7) -----

    #[test]
    fn function_exists_user_host_and_missing() {
        // uf is a user function; array_map / usort are host builtins; nope is none.
        assert_eq!(
            vm_stdout(b"<?php function uf(){} echo (function_exists('uf')?1:0),(function_exists('array_map')?1:0),(function_exists('usort')?1:0),(function_exists('nope')?1:0);"),
            b"1110"
        );
    }

    #[test]
    fn class_exists_and_interface_exists() {
        // class_exists is true for class/abstract/enum, false for an interface.
        assert_eq!(
            vm_stdout(b"<?php abstract class AB{} interface IF1{} enum EN{ case A; } class C{} echo (class_exists('C')?1:0),(class_exists('AB')?1:0),(class_exists('EN')?1:0),(class_exists('IF1')?1:0),(class_exists('Nope')?1:0),'|',(interface_exists('IF1')?1:0),(interface_exists('C')?1:0);"),
            b"11100|10"
        );
    }

    #[test]
    fn method_exists_object_string_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class B{ function bm(){} } class C extends B{ function m(){} static function s(){} } echo (method_exists('C','m')?1:0),(method_exists(new C,'s')?1:0),(method_exists('C','bm')?1:0),(method_exists('C','nope')?1:0),(method_exists('Nope','m')?1:0);"),
            b"11100"
        );
    }

    #[test]
    fn property_exists_declared_static_dynamic_inherited() {
        assert_eq!(
            vm_stdout(b"<?php class B{ public $base=1; static $st=9; } class C extends B{ protected $own=2; } $o=new C; $o->dyn=5; echo (property_exists('C','own')?1:0),(property_exists('C','base')?1:0),(property_exists('C','st')?1:0),(property_exists($o,'dyn')?1:0),(property_exists('C','dyn')?1:0);"),
            b"11110"
        );
    }

    #[test]
    fn get_called_class_is_late_static_bound() {
        assert_eq!(
            vm_stdout(b"<?php class P{ static function who(){ return get_called_class(); } } class Q extends P{} echo Q::who(),'|',P::who();"),
            b"Q|P"
        );
    }

    #[test]
    fn get_called_class_global_scope_is_fatal() {
        let out = vm_outcome(b"<?php get_called_class();");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be called from within a class".len())
                .any(|w| w == b"must be called from within a class"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    // ----- Session 1: error_reporting / trigger_error / error_get_last -----

    #[test]
    fn e_all_constant_is_php85_value() {
        assert_eq!(vm_stdout(b"<?php echo E_ALL;"), b"30719");
    }

    #[test]
    fn error_reporting_get_and_set_returns_old() {
        assert_eq!(
            vm_stdout(b"<?php $a=error_reporting(); $old=error_reporting(0); $b=error_reporting(); echo $a,'|',$old,'|',$b;"),
            b"30719|30719|0"
        );
    }

    #[test]
    fn trigger_error_default_is_notice() {
        let out = vm_outcome(b"<?php trigger_error('hi'); echo 'A';");
        assert_eq!(out.stdout, b"A");
        assert!(
            out.rendered.windows(b"Notice: hi in ".len()).any(|w| w == b"Notice: hi in "),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn trigger_error_user_warning_level() {
        let out = vm_outcome(b"<?php trigger_error('warn', E_USER_WARNING); echo 'B';");
        assert_eq!(out.stdout, b"B");
        assert!(
            out.rendered.windows(b"Warning: warn in ".len()).any(|w| w == b"Warning: warn in "),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn error_reporting_zero_silences_trigger_error() {
        let out = vm_outcome(b"<?php error_reporting(0); trigger_error('silent'); echo 'C';");
        assert_eq!(out.stdout, b"C");
        assert!(
            !out.rendered.windows(b"silent".len()).any(|w| w == b"silent"),
            "diagnostic should be gated; rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn trigger_error_user_error_no_handler_is_fatal() {
        // No handler: E_USER_ERROR is the fatal (PHP 8.4 deprecation renders first).
        let out = vm_outcome(b"<?php trigger_error('boom', E_USER_ERROR); echo 'AFTER';");
        assert!(out.fatal.is_some(), "E_USER_ERROR without a handler must be fatal");
        assert!(!out.stdout.windows(5).any(|w| w == b"AFTER"), "script must not continue past the fatal");
        assert!(
            out.rendered.windows(b"deprecated since 8.4".len()).any(|w| w == b"deprecated since 8.4"),
            "the 8.4 deprecation renders before the fatal: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn trigger_error_user_error_handled_continues() {
        // Handler handles both the 8.4 deprecation (8192) and the E_USER_ERROR (256)
        // and returns truthy → the script CONTINUES past trigger_error.
        let out = vm_outcome(
            b"<?php set_error_handler(function($n,$s){ echo \"[H:$n]\"; return true; }); trigger_error('boom', E_USER_ERROR); echo 'AFTER';",
        );
        assert!(out.fatal.is_none(), "handled E_USER_ERROR must not be fatal: {:?}", out.fatal);
        assert_eq!(out.stdout, b"[H:8192][H:256]AFTER");
    }

    #[test]
    fn trigger_error_user_error_handler_false_is_fatal() {
        // Handler returns false for the E_USER_ERROR → it falls through to the fatal.
        let out = vm_outcome(
            b"<?php set_error_handler(function($n,$s){ return false; }); trigger_error('boom', E_USER_ERROR); echo 'AFTER';",
        );
        assert!(out.fatal.is_some(), "a `false` return on E_USER_ERROR is still fatal");
    }

    #[test]
    fn error_get_last_null_after_handled_user_error() {
        // A handled E_USER_ERROR leaves error_get_last unset (oracle-confirmed).
        let out = vm_outcome(
            b"<?php set_error_handler(function($n,$s){ return true; }); trigger_error('boom', E_USER_ERROR); echo (error_get_last()===null)?'NULL':'SET';",
        );
        assert!(out.fatal.is_none());
        assert_eq!(out.stdout, b"NULL");
    }

    #[test]
    fn trigger_error_invalid_level_is_value_error() {
        let out = vm_outcome(b"<?php trigger_error('x', E_WARNING);");
        assert!(out.fatal.is_some());
        assert!(
            out.rendered.windows(b"must be one of E_USER_ERROR".len())
                .any(|w| w == b"must be one of E_USER_ERROR"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn error_get_last_reports_trigger_error() {
        assert_eq!(
            vm_stdout(b"<?php trigger_error('oops', E_USER_WARNING); $e=error_get_last(); echo $e['type'],'|',$e['message'],'|',$e['line'];"),
            b"512|oops|1"
        );
    }

    #[test]
    fn error_get_last_null_when_none() {
        assert_eq!(vm_stdout(b"<?php echo (error_get_last()===null)?'N':'S';"), b"N");
    }

    #[test]
    fn error_get_last_captures_builtin_diagnostic() {
        // Session 2 refinement: a built-in warning (errno 2) is recorded too, not
        // just `trigger_error`. `t_warn()` emits `Diag::Warning("from builtin")`.
        let out = vm_run(
            b"<?php t_warn(); $e=error_get_last(); echo $e['type'],'|',$e['message'];",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"2|from builtin");
    }

    #[test]
    fn error_get_last_is_most_recent_across_kinds() {
        // Most-recent-wins: a built-in warning after a trigger_error overwrites it.
        let out = vm_run(
            b"<?php trigger_error('u', E_USER_NOTICE); t_warn(); $e=error_get_last(); echo $e['type'],'|',$e['message'];",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"2|from builtin");
    }

    #[test]
    fn error_get_last_not_set_when_handler_suppresses() {
        // Oracle-confirmed: a diagnostic *handled* by a user handler (truthy return)
        // does NOT update error_get_last — only the default handler records it.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ return true; }); t_warn(); echo (error_get_last()===null)?'NULL':'SET';",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"NULL");
    }

    #[test]
    fn error_get_last_set_when_handler_returns_false() {
        // Handler returns false → the default handler runs → last_error IS recorded.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ return false; }); t_warn(); $e=error_get_last(); echo $e===null?'NULL':($e['type'].'|'.$e['message']);",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"2|from builtin");
    }

    // ----- Session 3: preg_replace_callback (vs PHP 8.5.7 CLI) -----

    #[test]
    fn preg_replace_callback_wraps_matches() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace_callback('/\\d+/', function($m){ return '['.$m[0].']'; }, 'a1b22c');"),
            b"a[1]b[22]c"
        );
    }

    #[test]
    fn preg_replace_callback_uses_capture_groups() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace_callback('/(\\w)(\\d)/', function($m){ return $m[2].$m[1]; }, 'x5y6');"),
            b"5x6y"
        );
    }

    #[test]
    fn preg_replace_callback_no_match_is_unchanged() {
        assert_eq!(
            vm_stdout(b"<?php echo preg_replace_callback('/z/', fn($m)=>'!', 'abc');"),
            b"abc"
        );
    }

    // ----- Session 1b: set_exception_handler / restore_exception_handler -----

    #[test]
    fn exception_handler_catches_uncaught_throw() {
        let out = vm_outcome(b"<?php set_exception_handler(function($e){ echo 'caught:'.$e->getMessage(); }); throw new RuntimeException('boom');");
        assert!(out.fatal.is_none(), "handler should suppress the fatal: {:?}", out.fatal);
        assert_eq!(out.stdout, b"caught:boom");
    }

    #[test]
    fn exception_handler_receives_engine_error() {
        // `$x->foo()` on an undefined variable raises an Error, synthesized into the
        // Error throwable and handed to the handler.
        let out = vm_outcome(b"<?php set_exception_handler(function($e){ echo 'H:'.get_class($e); }); $x->foo();");
        assert!(out.fatal.is_none(), "fatal: {:?}", out.fatal);
        assert_eq!(out.stdout, b"H:Error");
    }

    #[test]
    fn restore_exception_handler_re_exposes_fatal() {
        let out = vm_outcome(b"<?php set_exception_handler(function($e){ echo 'X'; }); restore_exception_handler(); throw new Exception('y');");
        assert!(out.fatal.is_some(), "handler was restored, so the throw is fatal");
        assert_eq!(out.stdout, b"");
        assert!(
            out.rendered.windows(b"Uncaught Exception: y".len())
                .any(|w| w == b"Uncaught Exception: y"),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn exception_subclass_initialises_inherited_private_default() {
        // Regression: constructing a subclass of the prelude Exception ran the
        // prop_init thunk in the subclass scope, faulting on Exception's private
        // `$trace = []` default. Init writes are now privileged.
        assert_eq!(
            vm_stdout(b"<?php class MyEx extends InvalidArgumentException {} $e=new MyEx('bad', 7); echo get_class($e),':',$e->getMessage(),':',$e->getCode();"),
            b"MyEx:bad:7"
        );
    }

    #[test]
    fn set_exception_handler_returns_previous() {
        assert_eq!(
            vm_stdout(b"<?php $p1=set_exception_handler(function($e){}); $p2=set_exception_handler(function($e){}); echo ($p1===null?'N':'?'),($p2!==null?'S':'?');"),
            b"NS"
        );
    }

    // ----- Session 2: set_error_handler / restore_error_handler -----
    //
    // The VM reads an unset local as silent NULL (no "Undefined variable"), so
    // these exercise routing through the `t_warn()` test builtin (a built-in
    // `Diag::Warning("from builtin")`, errno 2) and `trigger_error` (E_USER_*).
    // The handler-return / mask / re-entrancy / throw semantics asserted here were
    // each confirmed byte-for-byte against the PHP 8.5.7 oracle.

    /// True when `rendered` contains `needle` (e.g. a default `Warning:` line that
    /// only appears when the engine handler ran, not a suppressing user callback).
    fn rendered_has(out: &super::VmOutcome, needle: &[u8]) -> bool {
        out.rendered.windows(needle.len()).any(|w| w == needle)
    }

    #[test]
    fn error_handler_routes_builtin_warning_and_suppresses_default() {
        // Handler returns null → the built-in warning is suppressed (no default render).
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ echo \"[H:$n:$s]\"; }); t_warn(); echo 'END';",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"[H:2:from builtin]END");
        assert!(!rendered_has(&out, b"Warning:"), "default render must be suppressed: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn error_handler_returning_false_runs_default_render() {
        // Returning literal `false` lets the default `Warning:` render run too.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ echo '[H]'; return false; }); t_warn(); echo '|END';",
            &fake_registry(),
        );
        assert!(rendered_has(&out, b"[H]"), "handler ran");
        assert!(rendered_has(&out, b"Warning: from builtin in test.php"), "default render ran: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn error_handler_called_under_error_reporting_zero() {
        // The handler is invoked even under `error_reporting(0)`; its `false` would
        // normally render, but `error_reporting(0)` gates the default render away.
        let out = vm_run(
            b"<?php error_reporting(0); set_error_handler(function($n,$s){ echo \"[H:$n]\"; return false; }); t_warn(); echo '|END';",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"[H:2]|END");
        assert!(!rendered_has(&out, b"Warning:"), "error_reporting(0) gates default render: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn error_handler_level_mask_excludes_builtin_warning() {
        // Handler registered for E_USER_WARNING only: a built-in E_WARNING falls to
        // the default render, but `trigger_error(.., E_USER_WARNING)` hits the handler.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ echo \"[H:$n]\"; }, E_USER_WARNING); t_warn(); trigger_error('ut', E_USER_WARNING); echo '|END';",
            &fake_registry(),
        );
        assert!(rendered_has(&out, b"Warning: from builtin in test.php"), "builtin warning uses default: {}", String::from_utf8_lossy(&out.rendered));
        assert!(rendered_has(&out, b"[H:512]"), "trigger_error routes to handler: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn error_handler_throw_propagates_to_surrounding_try() {
        // The extremely common `throw new ErrorException(...)` idiom: the handler's
        // throw surfaces from the faulting statement, caught by its try/catch.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ throw new RuntimeException('from handler'); }); try { t_warn(); } catch (Throwable $e) { echo '[C:'.$e->getMessage().']'; } echo '|END';",
            &fake_registry(),
        );
        assert!(out.fatal.is_none(), "throw was caught, not fatal: {:?}", out.fatal);
        assert_eq!(out.stdout, b"[C:from handler]|END");
    }

    #[test]
    fn error_handler_is_not_reentered() {
        // A warning raised *inside* the handler must not recurse: it default-renders,
        // and the handler body runs exactly once.
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ echo '[H]'; t_warn(); }); t_warn(); echo '|END';",
            &fake_registry(),
        );
        let hits = out.rendered.windows(3).filter(|w| *w == b"[H]").count();
        assert_eq!(hits, 1, "handler ran exactly once (no recursion): {}", String::from_utf8_lossy(&out.rendered));
        assert!(rendered_has(&out, b"Warning: from builtin in test.php"), "inner warning default-renders: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn trigger_error_routes_to_handler() {
        let out = vm_run(
            b"<?php set_error_handler(function($n,$s){ echo \"[H:$n:$s]\"; }); trigger_error('boom', E_USER_NOTICE); echo '|END';",
            &fake_registry(),
        );
        assert_eq!(out.stdout, b"[H:1024:boom]|END");
        assert!(!rendered_has(&out, b"Notice:"), "default render suppressed by handler: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn restore_error_handler_re_exposes_default() {
        // After restore, a built-in warning renders the default way again, and the
        // first `set_error_handler` returned null (no previous handler).
        let out = vm_run(
            b"<?php $p=set_error_handler(function($n,$s){ echo '[H]'; }); restore_error_handler(); t_warn(); echo '|'.($p===null?'N':'?').'END';",
            &fake_registry(),
        );
        assert!(rendered_has(&out, b"Warning: from builtin in test.php"), "default restored: {}", String::from_utf8_lossy(&out.rendered));
        assert!(rendered_has(&out, b"|NEND"), "first set returned null previous: {}", String::from_utf8_lossy(&out.rendered));
    }

    #[test]
    fn set_error_handler_returns_previous() {
        assert_eq!(
            vm_stdout(b"<?php $p1=set_error_handler(function($n,$s){}); $p2=set_error_handler(function($n,$s){}); echo ($p1===null?'N':'?'),($p2!==null?'S':'?');"),
            b"NS"
        );
    }

    #[test]
    fn deep_recursion_yields_clean_error_not_host_crash() {
        // Runaway recursion must surface a catchable PHP `Error` via the
        // call-stack depth guard, not exhaust memory / abort the host process.
        // The VM runs PHP recursion *iteratively* (frames grow on the heap, the
        // unwind pops them in a loop), so unlike the tree-walker this needs no
        // oversized worker stack.
        let program =
            lower_source(b"t.php", b"<?php function r($n){ return r($n + 1); } r(0);").expect("lower");
        let module = compile_program(&program, &Registry::new()).expect("compile");
        let out = run_module(&module, &Registry::new());
        match out.fatal {
            Some(PhpError::Error(m)) => {
                assert!(m.contains("call stack depth"), "unexpected message: {m}")
            }
            other => panic!("expected depth-guard error, got {other:?}"),
        }
    }

    #[test]
    fn static_var_persists_accumulates_and_is_per_function() {
        // Accumulates across calls.
        assert_eq!(vm_stdout(b"<?php function f(){ static $n = 0; echo ++$n; } f(); f(); f();"), b"123");
        // Per-function: f and g keep independent statics.
        assert_eq!(
            vm_stdout(b"<?php function f(){ static $n=0; echo ++$n; } function g(){ static $n=100; echo ++$n; } f(); g(); f();"),
            b"11012"
        );
        // Multiple bindings in one declaration.
        assert_eq!(vm_stdout(b"<?php function f(){ static $a=1, $b=2; echo $a+$b; $a++; $b++; } f(); f();"), b"35");
        // No initialiser: null on first call, then persists.
        assert_eq!(vm_stdout(b"<?php function f(){ static $a; echo $a===null?'Y':'N'; $a=1; } f(); f();"), b"YN");
        // Shared across recursion (same cell at every depth).
        assert_eq!(
            vm_stdout(b"<?php function f($d){ static $n=0; $n++; if ($d>0) f($d-1); return $n; } echo f(3);"),
            b"4"
        );
    }

    #[test]
    fn globals_superglobal_reads_writes_and_compounds_script_scope() {
        // Create/write a global from inside a function, read it back outside.
        assert_eq!(vm_stdout(b"<?php function f(){ $GLOBALS['n'] = 5; } f(); echo $n;"), b"5");
        // Read an outer global from inside a function.
        assert_eq!(vm_stdout(b"<?php $x=10; function f(){ echo $GLOBALS['x']; } f();"), b"10");
        // Overwrite an existing global from inside a function.
        assert_eq!(vm_stdout(b"<?php $x=3; function f(){ $GLOBALS['x'] = 8; } f(); echo $x;"), b"8");
        // Compound assign through $GLOBALS.
        assert_eq!(vm_stdout(b"<?php $x=1; function f(){ $GLOBALS['x'] += 4; } f(); echo $x;"), b"5");
        // Top-level $GLOBALS write aliases the plain variable.
        assert_eq!(vm_stdout(b"<?php $GLOBALS['x']=7; echo $x;"), b"7");
    }

    #[test]
    fn closure_bind_bindto_and_fromcallable() {
        // Closure::bind rebinds $this and returns a new closure.
        assert_eq!(
            vm_stdout(b"<?php class C { public $v = 3; } $f = function() { return $this->v; }; $g = Closure::bind($f, new C); echo $g();"),
            b"3"
        );
        // $closure->bindTo(...) is the instance-method form.
        assert_eq!(
            vm_stdout(b"<?php class C { public $v = 9; } $f = function() { return $this->v; }; $g = $f->bindTo(new C); echo $g();"),
            b"9"
        );
        // Closure::fromCallable wraps a function name into an invokable closure.
        assert_eq!(
            vm_stdout(b"<?php function dbl($x){ return $x*2; } $f = Closure::fromCallable('dbl'); echo $f(21);"),
            b"42"
        );
    }

    #[test]
    fn enum_static_methods_cases_from_tryfrom() {
        // cases() returns the singletons in declaration order.
        assert_eq!(
            vm_stdout(b"<?php enum Suit { case Hearts; case Spades; } $n=0; foreach (Suit::cases() as $c){ echo $c->name; $n++; } echo $n;"),
            b"HeartsSpades2"
        );
        // cases() yields the same singletons as direct case access.
        assert_eq!(vm_stdout(b"<?php enum Suit { case Hearts; case Spades; } echo Suit::cases()[0] === Suit::Hearts ? 'y':'n';"), b"y");
        // from() matches a backing value; tryFrom() returns null on a miss.
        assert_eq!(
            vm_stdout(b"<?php enum S:string { case A='x'; case B='y'; } echo S::from('y')===S::B?'y':'n'; echo S::tryFrom('z')===null?'y':'n';"),
            b"yy"
        );
    }

    #[test]
    fn enum_from_miss_is_valueerror_and_cases_are_readonly() {
        // from() on a miss raises a catchable ValueError with PHP's message.
        assert_eq!(
            vm_stdout(b"<?php enum Size:int { case S=1; case L=3; } try { Size::from(9); } catch (\\ValueError $e) { echo $e->getMessage(); }"),
            b"9 is not a valid backing value for enum Size"
        );
        // A backed case is immutable: modifying an existing property is an Error.
        assert_eq!(
            vm_stdout(b"<?php enum St:string { case A='A'; } $a=St::A; try { $a->value='Z'; } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Cannot modify readonly property St::$value"
        );
        // Creating a dynamic property on a case is an Error.
        assert_eq!(
            vm_stdout(b"<?php enum St:string { case A='A'; } $a=St::A; try { $a->nope=1; } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Cannot create dynamic property St::$nope"
        );
    }

    #[test]
    fn named_function_args_runtime_binding() {
        // Unknown name → catchable Error.
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(z:1); } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Unknown named parameter $z"
        );
        // A name colliding with a positional → catchable Error.
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(1, a:2); } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Named parameter $a overwrites previous argument"
        );
        // A name with no fixed home collects into the variadic, keyed by name.
        assert_eq!(
            vm_stdout(b"<?php function f($a, ...$rest){ $s=\"$a|\"; foreach($rest as $k=>$v) $s.=\"$k:$v \"; return $s; } echo f(1, k:2);"),
            b"1|k:2 "
        );
        // A by-reference named argument writes back through its variable.
        assert_eq!(
            vm_stdout(b"<?php function inc(&$x){ $x++; } $n=5; inc(x: $n); echo $n;"),
            b"6"
        );
    }

    #[test]
    fn spread_call_named_and_positional_semantics() {
        // String keys map to parameters by name; gaps use defaults.
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b='B',$c='C'){ return \"$a-$b-$c\"; } echo f(...['a'=>1,'c'=>3]);"),
            b"1-B-3"
        );
        // Spread positional then explicit named.
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b,$c){ return \"$a-$b-$c\"; } echo f(...[1], c:3, b:2);"),
            b"1-2-3"
        );
        // String keys collected into a variadic keep their key.
        assert_eq!(
            vm_stdout(b"<?php function f(...$args){ $s=''; foreach($args as $k=>$v) $s.=\"$k:$v \"; return $s; } echo f(...['x'=>1,'y'=>2]);"),
            b"x:1 y:2 "
        );
        // A generator's string keys become named too.
        assert_eq!(
            vm_stdout(b"<?php function gen(){ yield 'x'=>1; yield 'y'=>2; } function f(...$n){ $s=''; foreach($n as $k=>$v) $s.=\"$k:$v \"; return $s; } echo f(...gen());"),
            b"x:1 y:2 "
        );
    }

    #[test]
    fn spread_call_error_paths() {
        // A non-iterable spread is a TypeError.
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(...5); } catch (\\TypeError $e) { echo $e->getMessage(); }"),
            b"Only arrays and Traversables can be unpacked, int given"
        );
        // An unknown named key from a spread is a catchable Error.
        assert_eq!(
            vm_stdout(b"<?php function f($a){} try { f(...['z'=>1]); } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Unknown named parameter $z"
        );
        // A spread named key colliding with a positional overwrites it → Error.
        assert_eq!(
            vm_stdout(b"<?php function f($a,$b,$c){} try { f(1, ...['a'=>9,'b'=>2,'c'=>3]); } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Named parameter $a overwrites previous argument"
        );
        // A positional (int key) after a named one within the unpacking → Error.
        assert_eq!(
            vm_stdout(b"<?php function f($x, ...$r){} try { f(1, ...['k'=>2, 0=>3]); } catch (\\Error $e) { echo $e->getMessage(); }"),
            b"Cannot use positional argument after named argument during unpacking"
        );
    }

    #[test]
    fn coalesce_and_coalesce_assign_on_properties() {
        // `??=` on a declared null property assigns.
        assert_eq!(vm_stdout(b"<?php class C { public $x = null; } $c = new C; $c->x ??= 7; echo $c->x;"), b"7");
        // `??=` on a magic property: __isset decides, __set only when unset.
        assert_eq!(
            vm_stdout(b"<?php class C { private $d=[]; function __isset($n){return isset($this->d[$n]);} function __get($n){return $this->d[$n]??null;} function __set($n,$v){$this->d[$n]=$v;} } $c=new C(); $c->x ??= 'NEW'; echo $c->x;"),
            b"NEW"
        );
        // `??` on an unset magic property uses __isset and never calls __get.
        assert_eq!(
            vm_stdout(b"<?php class C { function __isset($n){return false;} function __get($n){echo 'G'; return 1;} } $c=new C(); echo ($c->x ?? 'D');"),
            b"D"
        );
    }

    #[test]
    fn empty_and_compound_assign_on_properties() {
        // empty() on a declared property: set+truthy vs null/unset.
        assert_eq!(
            vm_stdout(b"<?php class C { public $x=5; public $y=null; } $c=new C; echo empty($c->x)?'D':'d'; echo empty($c->y)?'E':'e'; echo empty($c->z)?'F':'f';"),
            b"dEF"
        );
        // empty() with __isset true but no __get: value is null → empty (silent).
        assert_eq!(
            vm_stdout(b"<?php class C { private $d=['foo'=>'']; function __isset($n){return isset($this->d[$n]);} } $c=new C(); echo empty($c->foo)?'E':'N';"),
            b"E"
        );
        // empty() with __isset then __get.
        assert_eq!(
            vm_stdout(b"<?php class C { private $d=['z'=>0,'v'=>5]; function __isset($n){return isset($this->d[$n]);} function __get($n){return $this->d[$n];} } $c=new C(); echo empty($c->z)?'1':'0'; echo empty($c->v)?'1':'0'; echo empty($c->missing)?'1':'0';"),
            b"101"
        );
        // Compound `+=` on a magic property routes through __get then __set.
        assert_eq!(
            vm_stdout(b"<?php class C { private $d=['n'=>10]; function __get($k){return $this->d[$k];} function __set($k,$v){$this->d[$k]=$v;} } $c=new C(); $c->n += 5; echo $c->n;"),
            b"15"
        );
    }

    #[test]
    fn match_unhandled_includes_subject_and_stringable_instanceof() {
        // UnhandledMatchError carries the subject value in its message.
        let program = lower_source(b"t.php", b"<?php echo match (5) { 1 => 'a' };").expect("lower");
        let module = compile_program(&program, &Registry::new()).expect("compile");
        let out = run_module(&module, &Registry::new());
        match out.fatal {
            Some(PhpError::Error(m)) => assert_eq!(m, "Unhandled match case 5"),
            other => panic!("expected UnhandledMatchError, got {other:?}"),
        }
        // A class with __toString auto-implements Stringable.
        assert_eq!(
            vm_stdout(b"<?php class A { function __toString():string { return 'x'; } } class B {} echo ((new A) instanceof Stringable)?'1':'0', ((new B) instanceof Stringable)?'1':'0';"),
            b"10"
        );
    }

    #[test]
    fn coalesce_on_string_offset_and_array_element_assign() {
        // `??` on a string offset: in-range yields the char, out-of-range or a
        // non-integer key is unset → default.
        assert_eq!(
            vm_stdout(b"<?php $s='test'; echo $s[0]??'d', $s[5]??'d', $s['str']??'d', $s[-1]??'d';"),
            b"tddt"
        );
        // `??=` on an array element assigns only when unset.
        assert_eq!(vm_stdout(b"<?php $a=[]; $a['x'] ??= 7; echo $a['x']; $a['x'] ??= 9; echo $a['x'];"), b"77");
    }
}

