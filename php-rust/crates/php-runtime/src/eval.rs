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
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use corosensei::{Coroutine, CoroutineResult, Yielder};

use php_types::{
    convert, dtoa, numstr, ops, Closure, ClosureInfo, ClosureParam, ClosureRender, Diag, Diags,
    GenDriver, GenKey, GenState, GenStatus, GenStep, Key, Object, ObjectInfo, PhpArray, PhpError,
    PhpStr, PropVis, Props, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::hir::{
    BinOp, Capture, CastKind, ClassDecl, ClassId, ClassRef, Expr, ExprKind, FnDecl, Line,
    MethodDecl, Param, Place, PlaceBase, PlaceStep, Program, ScalarType, Slot, StaticAssignOp,
    Stmt, StmtKind, TypeHint, UnOp, Visibility,
};

/// Builtins the evaluator implements directly because they invoke a callback
/// (step 18, D-18.6). They are dispatched ahead of the registry and are treated
/// as callable names by `is_callable`.
const HIGHER_ORDER_BUILTINS: &[&[u8]] = &[
    b"is_callable",
    b"call_user_func",
    b"call_user_func_array",
    b"array_map",
    b"array_filter",
    b"usort",
    b"json_decode",
    b"preg_match",
    b"preg_match_all",
    b"preg_replace",
    b"preg_replace_callback",
    b"preg_split",
    b"preg_quote",
    b"mb_ereg",
    b"mb_eregi",
    b"mb_ereg_replace",
    b"mb_eregi_replace",
    b"mb_ereg_replace_callback",
    b"mb_split",
    b"mb_ereg_match",
    b"mb_regex_encoding",
    b"mb_regex_set_options",
    b"mb_ereg_search_init",
    b"mb_ereg_search",
    b"mb_ereg_search_pos",
    b"mb_ereg_search_regs",
    b"mb_ereg_search_getregs",
    b"mb_ereg_search_getpos",
    b"mb_ereg_search_setpos",
];

/// One entry of the runtime call stack (step 28). `line` is the *call site*
/// (where this function was invoked from), `function` its name, and `class`/
/// `is_static` describe a method (`Class->m` / `Class::m`) versus a free
/// function (`class: None`).
struct CallFrame {
    class: Option<Vec<u8>>,
    function: Vec<u8>,
    is_static: bool,
    line: i64,
}

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
            // `$this` as a write base (`$this->x = …`): the current object handle
            // (step 19, D-19.9). Callers guard the no-`$this` case before reaching
            // here, so the unwrap is sound.
            PlaceBase::This => $self
                .cur_this
                .as_mut()
                .expect("$this in write context guarded by caller"),
        }
    };
}

/// A user-function argument as bound for a call: a plain value for a by-value
/// parameter, or a shared cell for a `&$x` by-reference parameter (D-R6).
/// `Default` is a gap left by named arguments (step 38) — the parameter at this
/// index was not supplied, so its declared default applies.
/// Named arguments produced by unpacking string keys (`...['k' => v]`, step 40):
/// already evaluated by value, awaiting placement by parameter name.
type SpreadNamed = Vec<(Box<[u8]>, Zval)>;

/// Defensive ceiling on PHP call-stack depth. The evaluator recurses on the
/// native (Rust) stack, which has no overflow protection — runaway recursion
/// would abort the host process with SIGABRT (e.g. taking down a whole
/// `phpt-runner` batch). This guard converts that into a clean catchable
/// `Error` instead. It is calibrated for the `phpt-runner`'s 1 GiB worker
/// thread (native overflow observed ~38k frames there); realistic programs
/// never approach it. NB: on a small caller stack the native overflow can
/// still happen first — deep-recursion safety presumes a large worker stack.
const MAX_CALL_DEPTH: usize = 25_000;

enum Arg {
    Val(Zval),
    Ref(Rc<RefCell<Zval>>),
    Default,
    /// A named argument with no matching parameter, collected into a trailing
    /// variadic `...$rest` keyed by its name (step 40-2). Only ever appended
    /// after all positional slots, so it is seen exclusively by the variadic
    /// branch of [`Evaluator::bind_params`].
    Named(Box<[u8]>, Zval),
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
    /// `goto label` searching for its target (step 45). Propagates outward
    /// through enclosing blocks/loops until an `exec_stmts` frame finds a
    /// matching `Label`; lowering guarantees the target exists in scope and is
    /// not inside a loop/switch, so it always resolves before escaping the
    /// function body.
    Goto(Box<[u8]>),
}

/// Error for a `goto` that escapes its function scope unresolved (step 45). After
/// lowering's compile-time check this can only be the scope-out case D-45.1:
/// jumping *into* a transparent block (`if`/`try` body/`catch`/plain block),
/// which the tree-walking evaluator cannot land in mid-block. Raised as a
/// catchable engine error rather than silently mis-running.
fn unsupported_goto(label: &[u8]) -> PhpError {
    PhpError::Error(format!(
        "'goto' into a block is not supported (label '{}', D-45.1)",
        String::from_utf8_lossy(label)
    ))
}

// --- generators (step 39) ---
//
// A generator body runs inside a stackful `corosensei::Coroutine` so a `yield`
// can suspend the evaluator's native recursion mid-`eval` and resume later. The
// coroutine never captures `&mut Evaluator` (the borrow checker cannot see that
// the driver and the body never touch it simultaneously); instead each `resume`
// hands in a lifetime-erased `*mut ()` pointer that the body reborrows. See
// [`GenDriverImpl::resume`] for the soundness argument.

/// Value passed *into* the coroutine on each resume: the value the suspended
/// `yield` expression should evaluate to (`send()` argument, NULL for `next()`),
/// plus the lifetime-erased evaluator pointer for this resume.
struct ResumeIn {
    sent: Zval,
    ev: *mut (),
}

/// Value the coroutine hands back when it suspends at a `yield`: the (still
/// unresolved) key and the yielded value. The driver resolves the key against
/// the generator's auto-key counter.
struct YieldOut {
    key: GenKey,
    value: Zval,
}

/// The evaluator-context a generator owns between suspensions. While the
/// coroutine is suspended these fields hold the generator's frame; they are
/// swapped into the live [`Evaluator`] for the duration of each resume (and the
/// driver's own frame swapped back out). Keeping them here — rather than on the
/// suspended native stack — is what lets the single `Evaluator` serve both the
/// driver and the generator body without losing either frame.
struct GenCtx {
    locals: Vec<Zval>,
    /// Raw pointer to the generator body's slot names (`FnDecl::slots`), kept
    /// alive by `_body`. Used only for undefined-variable diagnostics.
    local_names: *const [Box<[u8]>],
    cur_this: Option<Zval>,
    cur_class: Option<ClassId>,
    cur_static_class: Option<ClassId>,
    fn_returns_ref: bool,
    /// Type-erased `*const Yielder<ResumeIn, YieldOut>`, set by the body on its
    /// first run and reachable from the `yield` arm while this generator runs.
    gen_yielder: Option<*const ()>,
}

/// The runtime side of a generator: a stackful coroutine plus the owned body and
/// the generator's swappable evaluator context. Erased behind [`GenDriver`] so
/// `php-types` (where [`GenState`] lives) never names the coroutine crate or the
/// evaluator.
struct GenDriverImpl {
    co: Coroutine<ResumeIn, YieldOut, Result<Zval, PhpError>>,
    ctx: GenCtx,
    /// Owns the body the coroutine walks and the slot names `ctx.local_names`
    /// points into; must outlive the coroutine.
    _body: Rc<FnDecl>,
}

impl GenDriver for GenDriverImpl {
    fn resume(&mut self, sent: Zval, ev_erased: *mut ()) -> GenStep {
        // SAFETY: `ev_erased` is a `*mut Evaluator` the driver just produced from
        // a live `&mut self`; the evaluator outlives every resume (a generator
        // cannot outlive the run that owns the program). We reborrow it as
        // `Evaluator<'static>` — a lifetime extension that is sound because every
        // use is synchronous within this call and the borrowed program data is
        // never persisted past its real lifetime. The re-entrancy guard in
        // `resume_generator` guarantees this generator is not already on the
        // stack, so the reborrow does not alias another live `&mut Evaluator`
        // for *this* generator.
        let ev: &mut Evaluator<'static> = unsafe { &mut *(ev_erased as *mut Evaluator<'static>) };

        // Swap the generator's frame into the evaluator, saving the driver's.
        let saved_locals = ev.locals.replace(std::mem::take(&mut self.ctx.locals));
        // SAFETY: `ctx.local_names` points into `_body` (held here), alive for
        // the whole resume.
        let saved_names = ev.local_names.replace(unsafe { &*self.ctx.local_names });
        let saved_this = std::mem::replace(&mut ev.cur_this, self.ctx.cur_this.take());
        let saved_class = std::mem::replace(&mut ev.cur_class, self.ctx.cur_class);
        let saved_static = std::mem::replace(&mut ev.cur_static_class, self.ctx.cur_static_class);
        let saved_ret_ref = std::mem::replace(&mut ev.fn_returns_ref, self.ctx.fn_returns_ref);
        let saved_yielder = std::mem::replace(&mut ev.gen_yielder, self.ctx.gen_yielder);

        let step = self.co.resume(ResumeIn { sent, ev: ev_erased });

        // Pull the (possibly advanced) generator frame back out and restore the
        // driver's. `local_names` is unchanged (slot names are immutable).
        self.ctx.locals = ev.locals.take().unwrap_or_default();
        self.ctx.cur_this = ev.cur_this.take();
        self.ctx.cur_class = ev.cur_class;
        self.ctx.cur_static_class = ev.cur_static_class;
        self.ctx.fn_returns_ref = ev.fn_returns_ref;
        self.ctx.gen_yielder = ev.gen_yielder;
        ev.locals = saved_locals;
        ev.local_names = saved_names;
        ev.cur_this = saved_this;
        ev.cur_class = saved_class;
        ev.cur_static_class = saved_static;
        ev.fn_returns_ref = saved_ret_ref;
        ev.gen_yielder = saved_yielder;

        match step {
            CoroutineResult::Yield(out) => GenStep::Yielded {
                key: out.key,
                value: out.value,
            },
            CoroutineResult::Return(res) => GenStep::Returned(res),
        }
    }
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
    /// Process exit code from `exit`/`die` (step 46), normalised to `0..=255`.
    /// `None` when the script ran to completion without an explicit `exit`
    /// (PHP's implicit status 0). A real CLI SAPI would `process::exit` on this;
    /// here it is surfaced for tests (the `php-cli` binary stays a stub).
    pub exit_code: Option<u8>,
}

/// Lower `source` and run it with no builtins. Convenience wrapper over [`run`].
pub fn run_source(name: &[u8], source: &[u8]) -> Result<Outcome, crate::LowerError> {
    match crate::lower_source(name, source) {
        Ok(program) => Ok(run(&program)),
        Err(crate::LowerError::Fatal { message, line }) => {
            Ok(compile_fatal_outcome(name, &message, line))
        }
        Err(e) => Err(e),
    }
}

/// Lower `source` and run it with the given builtin registry.
pub fn run_source_with(
    name: &[u8],
    source: &[u8],
    registry: &Registry,
) -> Result<Outcome, crate::LowerError> {
    match crate::lower_source(name, source) {
        Ok(program) => Ok(run_with(&program, registry)),
        Err(crate::LowerError::Fatal { message, line }) => {
            Ok(compile_fatal_outcome(name, &message, line))
        }
        Err(e) => Err(e),
    }
}

/// Build the [`Outcome`] for a compile-time `Fatal error:` (step 21). PHP renders
/// these like a runtime fatal but without the "Uncaught" prefix or "thrown in"
/// tail: `\nFatal error: <msg> in <file> on line <line>\nStack trace:\n#0 {main}\n`.
fn compile_fatal_outcome(file: &[u8], message: &str, line: crate::hir::Line) -> Outcome {
    let file_s = String::from_utf8_lossy(file);
    let rendered = format!(
        "\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n"
    );
    Outcome {
        stdout: Vec::new(),
        rendered: rendered.into_bytes(),
        diags: Vec::new(),
        fatal: Some(PhpError::Error(message.to_string())),
        return_value: Zval::Null,
        exit_code: None,
    }
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

    // Precompute the render metadata (name/file/line + parameters) for each
    // closure body once; created values share it via `Rc` (step 18-7).
    let closure_info: Vec<Rc<ClosureInfo>> = program
        .closures
        .iter()
        .map(|f| Rc::new(closure_info_for(f, &program.file)))
        .collect();

    // Index classes by ASCII-lowercased name (PHP class names are
    // case-insensitive), step 19.
    let class_index: HashMap<Vec<u8>, usize> = program
        .classes
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name.to_ascii_lowercase(), i))
        .collect();

    let mut ev = Evaluator {
        global_names: &program.slots,
        local_names: None,
        reg: registry,
        funcs: &program.functions,
        closures: &program.closures,
        closure_info,
        next_object_id: 1,
        fn_index: &fn_index,
        classes: &program.classes,
        class_index: &class_index,
        cur_this: None,
        cur_class: None,
        cur_static_class: None,
        static_props: HashMap::new(),
        class_shapes: HashMap::new(),
        enum_cache: HashMap::new(),
        created: Vec::new(),
        destructed: HashSet::new(),
        call_stack: Vec::new(),
        magic_guard: HashSet::new(),
        file: &program.file,
        globals: fresh_slots(program.slots.len()),
        locals: None,
        fn_returns_ref: false,
        statics: vec![None; program.static_count],
        strict: program.strict,
        out: Vec::new(),
        rendered: Vec::new(),
        diags: Vec::new(),
        diags_rendered: 0,
        cur_line: 1,
        gen_yielder: None,
        mb_regex: crate::mbregex::MbRegexState::default(),
    };

    let mut exit_code = None;
    let (fatal, return_value) = match ev.exec_stmts(&program.body) {
        Ok(Flow::Return(v)) => (None, v),
        // An unresolved `goto` at script top level is the unsupported
        // into-transparent-block case (D-45.1); see `run_user_fn_body`.
        Ok(Flow::Goto(label)) => (Some(unsupported_goto(&label)), Zval::Null),
        Ok(_) => (None, Zval::Null),
        // `exit`/`die` is a clean termination, not a fatal (step 46): record the
        // process exit code and render nothing. Any message it printed is already
        // in the output streams.
        Err(PhpError::Exit(code)) => {
            exit_code = Some(code);
            (None, Zval::Null)
        }
        Err(e) => (Some(e), Zval::Null),
    };

    // Render any diagnostics still staged (defensive; statement/expression exits
    // already flush), then the uncaught fatal at the tail of the stream.
    ev.flush_diags();
    if let Some(err) = &fatal {
        ev.render_fatal(err);
    }

    // Shutdown sequence (step 24-2): run `__destruct` on every object still
    // reachable at the end of the script. PHP runs these after the body (and
    // after an uncaught fatal is printed), in reverse creation order.
    ev.run_destructors();

    Outcome {
        stdout: ev.out,
        rendered: ev.rendered,
        diags: ev.diags,
        fatal,
        return_value,
        exit_code,
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
    /// Anonymous/arrow function bodies, selected by a closure value's `fn_idx`
    /// (step 18, D-18.2).
    closures: &'p [FnDecl],
    /// Per-closure-body render metadata for `var_dump`/`print_r`, precomputed
    /// once and shared (cloned by `Rc`) into each created value (step 18-7).
    closure_info: Vec<Rc<ClosureInfo>>,
    /// Next object handle assigned to a created closure (the `#N` in `var_dump`).
    /// Monotonic — handles are not recycled when a closure is freed, so dumps of
    /// short-lived closures may number higher than PHP's (step 18-7 scope-out).
    next_object_id: u32,
    fn_index: &'p HashMap<Vec<u8>, usize>,
    /// User classes and their name→index map (step 19, D-19.3). A `new` / method
    /// dispatch resolves a class against `class_index`.
    classes: &'p [ClassDecl],
    class_index: &'p HashMap<Vec<u8>, usize>,
    /// The current object while a method body runs (`$this`, D-19.5). `None` at
    /// top level / inside a free function; reading `$this` then is a fatal Error.
    /// Saved and restored around each method call like `locals`.
    cur_this: Option<Zval>,
    /// The class that *defines* the running method (step 19-3, D-19.11): the
    /// referent of `self::` and the base for `parent::`, and the access context
    /// for private/protected visibility checks. Saved/restored per method call.
    cur_class: Option<ClassId>,
    /// The late-static-binding class (step 19-4, D-19.12): the referent of
    /// `static::` / `new static`. For an instance call it is the object's actual
    /// (most-derived) class; forwarding calls (`self`/`parent`/`static`) preserve
    /// it. Saved/restored per method call.
    cur_static_class: Option<ClassId>,
    /// Persistent storage for `static` properties, keyed by (declaring class id,
    /// property name); lazily initialised from the declared default on first
    /// access and shared for the whole run (step 19-4, D-19.14).
    static_props: HashMap<(ClassId, Vec<u8>), Rc<RefCell<Zval>>>,
    /// Cache of per-class property-visibility shapes for object dumping, built on
    /// first instantiation and shared by all instances (step 19-7, D-19.20).
    class_shapes: HashMap<ClassId, Rc<ObjectInfo>>,
    /// Interned enum case singletons, keyed by (enum class id, case name). The
    /// first `E::Case` access materialises the object; every later access returns
    /// the same `Rc`, giving `===`/`match` identity (step 23, D-23.2).
    enum_cache: HashMap<(ClassId, Vec<u8>), Rc<RefCell<Object>>>,
    /// Strong handles to every live object created via `new`, in creation order.
    /// The extra strong ref is what lets the destruction sweep detect
    /// unreachability: when an object's `Rc::strong_count` falls to 1, only this
    /// tracking ref remains, so the program can no longer reach it and its
    /// `__destruct` is due (step 24-2/24-3). Objects are removed as they are
    /// destructed (sweep) or at shutdown.
    created: Vec<Rc<RefCell<Object>>>,
    /// Object handles whose `__destruct` has already run, guarding against a
    /// double call (step 24-2).
    destructed: HashSet<u32>,
    /// Active call stack (step 28): one frame per user function / method
    /// currently executing, recording the call-site line and the callee's
    /// display name. Snapshotted into a Throwable's `trace`/`traceString` at
    /// construction so getTrace / getTraceAsString / the uncaught renderer show
    /// real frames.
    call_stack: Vec<CallFrame>,
    /// Active magic-accessor guards, keyed by (object handle, accessor kind,
    /// property name). While a guard is present, a nested access of the same
    /// kind to the same property bypasses the magic method (step 22, D-22.4).
    magic_guard: HashSet<(u32, MagicAccess, Vec<u8>)>,
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
    /// `declare(strict_types=1)` in effect: scalar hints are enforced without
    /// coercion (only `int`→`float` widening), step 16.
    strict: bool,
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
    /// While a generator body runs, the type-erased `*const Yielder<ResumeIn,
    /// YieldOut>` of the active generator (step 39). The `yield` arm reborrows it
    /// to suspend. `None` outside any generator; saved/restored per resume by
    /// [`GenDriverImpl::resume`] (part of the swapped generator context).
    gen_yielder: Option<*const ()>,
    /// Persistent mbregex state (step 43): the global `mb_regex_encoding` /
    /// `mb_regex_set_options` and the `mb_ereg_search` cursor. Survives across
    /// `mb_ereg*` calls for the whole run, since the search family is stateful.
    mb_regex: crate::mbregex::MbRegexState,
}

/// Which magic property accessor is currently running for an object/property,
/// used to suppress re-entry (step 22, D-22.4). Mirrors Zend's per-property
/// guard bits: `$this->p` inside `__get('p')` hits the real property, not a
/// nested `__get`, but a nested `__set('p')` is still allowed.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum MagicAccess {
    Get,
    Set,
    Isset,
    Unset,
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
        let file = String::from_utf8_lossy(self.file).into_owned();
        // A user-`throw`n object carries its own class, message and creation line
        // (step 20); an engine error uses its variant name and the current line.
        let (class, message, line, trace) = match err {
            PhpError::Thrown(Zval::Object(o)) => {
                let b = o.borrow();
                let class = String::from_utf8_lossy(b.class_name.as_bytes()).into_owned();
                let message = match b.props.get(b"message") {
                    Some(Zval::Str(s)) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                    _ => String::new(),
                };
                let line = match b.props.get(b"line") {
                    Some(Zval::Long(n)) => *n,
                    _ => self.cur_line as i64,
                };
                // Real frames captured at construction (step 28).
                let trace = match b.props.get(b"traceString") {
                    Some(Zval::Str(s)) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
                    _ => "#0 {main}".to_string(),
                };
                (class, message, line, trace)
            }
            other => (
                other.class_name().to_string(),
                other.message().to_string(),
                self.cur_line as i64,
                "#0 {main}".to_string(),
            ),
        };
        let block = format!(
            "\nFatal error: Uncaught {class}: {message} in {file}:{line}\nStack trace:\n{trace}\n  thrown in {file} on line {line}\n",
        );
        self.rendered.extend_from_slice(block.as_bytes());
    }

    // --- statements ---

    fn exec_stmts(&mut self, stmts: &[Stmt]) -> Result<Flow, PhpError> {
        // Index-based loop (step 45) so a `goto` can re-enter this block at a
        // different position. With no `goto` involved this walks the statements
        // exactly once, top to bottom, like the original `for`.
        let mut i = 0;
        while i < stmts.len() {
            match self.exec_stmt(&stmts[i])? {
                Flow::Normal => {}
                // A `goto` raised below: if its target label lives in *this*
                // block, jump there and keep executing; otherwise let it bubble
                // up to the enclosing block that owns the label.
                Flow::Goto(label) => {
                    match stmts
                        .iter()
                        .position(|s| matches!(&s.kind, StmtKind::Label(n) if **n == *label))
                    {
                        Some(j) => {
                            i = j;
                            continue;
                        }
                        None => return Ok(Flow::Goto(label)),
                    }
                }
                other => return Ok(other),
            }
            // Immediate destruction sweep at global-scope statement boundaries
            // (step 24-3): objects that just became unreachable (a discarded
            // temporary, an overwritten or unset variable, a returned-from local)
            // get their `__destruct` now. Destructor bodies run with a local
            // frame, so the `locals.is_none()` gate keeps this from re-entering.
            if self.locals.is_none() {
                self.sweep_destructors();
            }
            i += 1;
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

            // `label:` is a pure marker — `exec_stmts` uses it as a jump target
            // (step 45); reaching it during normal fall-through does nothing.
            StmtKind::Label(_) => {}

            // `goto label;` raises a `Goto` flow that `exec_stmts` resolves to
            // the matching label in this or an enclosing block (step 45).
            StmtKind::Goto(label) => return Ok(Flow::Goto(label.clone())),

            StmtKind::InlineHtml(bytes) => self.emit(bytes),

            StmtKind::Echo(values) => {
                for e in values {
                    let z = self.eval(e)?;
                    // `__toString` is honoured for objects (step 19-6); other
                    // values use the ordinary string funnel.
                    let s = self.stringify(&z)?;
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
                    self.check_first_prop_write(p.base, &steps, MagicAccess::Unset, b"__unset")?;
                    self.unset_place(p.base, &steps)?;
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

            StmtKind::Try {
                body,
                catches,
                finally,
            } => {
                // Run the protected body; a thrown exception (`Err(Thrown)`) is
                // offered to the catch clauses. Any other control flow (return /
                // break / continue) — or an uncaught throw — is carried in
                // `outcome` and resumes *after* `finally` runs.
                let outcome = match self.exec_stmts(body) {
                    Err(e) => self.handle_thrown(e, catches),
                    flow => flow,
                };
                // `exit`/`die` bypasses `finally` entirely (step 46): unlike a
                // thrown exception, a `return`, or a `break`, PHP does NOT run
                // `finally` on the way out of an `exit`. Propagate immediately.
                if matches!(&outcome, Err(PhpError::Exit(_))) {
                    return outcome;
                }
                if finally.is_empty() {
                    return outcome;
                }
                // `finally` always runs. Its own control flow (a return / throw /
                // break inside it) overrides the try/catch outcome; otherwise the
                // outcome (value, propagating signal, or re-thrown error) wins.
                match self.exec_stmts(finally)? {
                    Flow::Normal => return outcome,
                    other => return Ok(other),
                }
            }
        }
        Ok(Flow::Normal)
    }

    /// Offer a thrown exception to a `try`'s catch clauses (step 20). The first
    /// clause whose type matches by `instanceof` runs (binding `$e` if named);
    /// an unmatched throw propagates.
    ///
    /// Both user `throw`n objects and engine errors are catchable: an engine
    /// error (`PhpError::TypeError`, `DivisionByZeroError`, …) is resolved to its
    /// matching prelude class by name, and a Throwable object is *synthesized*
    /// (with its message) only if a clause actually binds it (step 20-3).
    fn handle_thrown(
        &mut self,
        e: PhpError,
        catches: &[crate::hir::CatchClause],
    ) -> Result<Flow, PhpError> {
        // The class id of the in-flight throwable: the object's own class for a
        // user throw, or the prelude class named by an engine error.
        let obj_cid = match &e {
            // `exit`/`die` is uncatchable (step 46): never offered to a `catch`,
            // it just keeps unwinding. The enclosing `try` still runs `finally`
            // on the way out (the generic `Err` path below `handle_thrown`).
            PhpError::Exit(_) => return Err(e),
            PhpError::Thrown(Zval::Object(o)) => o.borrow().class_id as usize,
            PhpError::Thrown(_) => return Err(e),
            engine => match self
                .class_index
                .get(engine.class_name().to_ascii_lowercase().as_bytes())
            {
                Some(&cid) => cid,
                None => return Err(e),
            },
        };
        for c in catches {
            for tname in &c.types {
                if let Some(&tid) = self.class_index.get(&tname.to_ascii_lowercase()) {
                    if self.is_instance_of(obj_cid, tid) {
                        if let Some(slot) = c.var {
                            let obj = match &e {
                                PhpError::Thrown(v) => v.clone(),
                                engine => self.synthesize_throwable(obj_cid, engine.message())?,
                            };
                            self.slot_set(slot as usize, obj);
                        }
                        return self.exec_stmts(&c.body);
                    }
                }
            }
        }
        Err(e)
    }

    /// Build a Throwable object of `class_id` carrying `message` (step 20-3), used
    /// to materialise an engine error (`TypeError`, `DivisionByZeroError`, …) when
    /// a `catch` binds it. Mirrors `eval_new`'s instance layout but sets the
    /// message/line/file directly instead of running a constructor.
    fn synthesize_throwable(&mut self, class_id: ClassId, message: &str) -> Result<Zval, PhpError> {
        let class_name = PhpStr::new(self.classes[class_id].name.to_vec());
        let props = self.collect_props(class_id)?;
        let info = self.class_shape(class_id);
        let id = self.next_id();
        let value = Zval::Object(Rc::new(RefCell::new(Object {
            class_id: class_id as u32,
            class_name,
            props,
            id,
            info,
        })));
        let (trace, trace_string) = self.capture_trace();
        if let Zval::Object(o) = &value {
            let mut b = o.borrow_mut();
            b.props
                .set(b"message", Zval::Str(PhpStr::new(message.as_bytes().to_vec())));
            b.props.set(b"line", Zval::Long(self.cur_line as i64));
            b.props
                .set(b"file", Zval::Str(PhpStr::new(self.file.to_vec())));
            b.props.set(b"trace", trace);
            b.props
                .set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
        }
        Ok(value)
    }

    /// Snapshot the current call stack as a Throwable's `(trace array, trace
    /// string)` (step 28). Frames are innermost-first; the final line is
    /// `#N {main}`. The array mirrors PHP's `getTrace()` shape (file / line /
    /// function / class / type / empty args).
    fn capture_trace(&self) -> (Zval, Vec<u8>) {
        let file = self.file;
        let mut arr = PhpArray::new();
        let mut s: Vec<u8> = Vec::new();
        for (i, frame) in self.call_stack.iter().rev().enumerate() {
            s.extend_from_slice(format!("#{i} ").as_bytes());
            s.extend_from_slice(file);
            s.extend_from_slice(format!("({}): ", frame.line).as_bytes());
            s.extend_from_slice(&frame_display(frame));
            s.extend_from_slice(b"()\n");

            let mut fr = PhpArray::new();
            fr.insert(Key::from_bytes(b"file"), Zval::Str(PhpStr::new(file.to_vec())));
            fr.insert(Key::from_bytes(b"line"), Zval::Long(frame.line));
            fr.insert(
                Key::from_bytes(b"function"),
                Zval::Str(PhpStr::new(frame.function.clone())),
            );
            if let Some(class) = &frame.class {
                fr.insert(Key::from_bytes(b"class"), Zval::Str(PhpStr::new(class.clone())));
                let ty: &[u8] = if frame.is_static { b"::" } else { b"->" };
                fr.insert(Key::from_bytes(b"type"), Zval::Str(PhpStr::new(ty.to_vec())));
            }
            fr.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(PhpArray::new())));
            let _ = arr.append(Zval::Array(Rc::new(fr)));
        }
        s.extend_from_slice(format!("#{} {{main}}", self.call_stack.len()).as_bytes());
        (Zval::Array(Rc::new(arr)), s)
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
            // A `goto` whose label was not found in this loop body (else
            // `exec_stmts` would have jumped) targets an enclosing scope: leave
            // the loop and keep searching outward (step 45). Jumping *into* a
            // loop is rejected at lowering, so this only ever exits a loop.
            Flow::Goto(l) => LoopStep::Propagate(Flow::Goto(l)),
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
            // A generator is driven live (step 39-2): unlike an array it is not
            // snapshotted — each iteration advances it.
            Zval::Generator(gs) => return self.foreach_generator(&gs, key, value, body),
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

    /// `foreach ($gen as [$k =>] $v)` over a generator (step 39-2). Drives it
    /// live — start, then on each iteration bind the current `(key, value)`, run
    /// the body, and advance — rather than snapshotting like an array. The
    /// generator's own key (already a `Zval`) is bound directly.
    fn foreach_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        key: Option<Slot>,
        value: Slot,
        body: &[Stmt],
    ) -> Result<Flow, PhpError> {
        self.ensure_started(gs_rc)?;
        loop {
            let (k, v, done) = {
                let gs = gs_rc.borrow();
                (
                    gs.cur_key.clone(),
                    gs.cur_val.clone(),
                    matches!(gs.status, GenStatus::Done),
                )
            };
            if done {
                break;
            }
            if let Some(ks) = key {
                self.slot_set(ks as usize, k);
            }
            self.slot_set(value as usize, v.deref_clone());
            match self.loop_step(body)? {
                LoopStep::Iterate => {}
                LoopStep::Stop => break,
                LoopStep::Propagate(f) => return Ok(f),
            }
            self.resume_generator(gs_rc, Zval::Null)?;
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
                // A `goto` whose label was not found inside this case body
                // targets an enclosing scope — leave the switch (jumping *into*
                // a switch is rejected at lowering). Step 45.
                Flow::Goto(l) => return Ok(Flow::Goto(l)),
            }
        }
        Ok(Flow::Normal)
    }

    // --- user functions ---

    /// Reject a call that would push the PHP call stack past [`MAX_CALL_DEPTH`],
    /// before recursing further on the native stack (see the constant's docs).
    /// `call_stack` already tracks every active function/method frame, so its
    /// length is the current call depth.
    fn guard_call_depth(&self) -> Result<(), PhpError> {
        if self.call_stack.len() >= MAX_CALL_DEPTH {
            return Err(PhpError::Error(format!(
                "Maximum call stack depth of {MAX_CALL_DEPTH} exceeded"
            )));
        }
        Ok(())
    }

    /// Invoke a hoisted user function: validate arity, set up a fresh local
    /// frame (its own slot table and slot names), bind parameters, run the body,
    /// then restore the caller's frame. Recursion uses the host (Rust) stack.
    fn call_user_fn(&mut self, idx: usize, argv: Vec<Arg>) -> Result<Zval, PhpError> {
        self.guard_call_depth()?;
        // `funcs` is `&'p [FnDecl]` (Copy): copying it out detaches the borrow
        // from `self`, so installing the local overlay below can mutate the
        // active frame freely.
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];

        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
        // A required parameter must have a real argument at its index; named
        // arguments (step 38) can leave `Arg::Default` gaps, so the supplied
        // count is not enough — check each required slot directly.
        let missing_required = f
            .params
            .iter()
            .enumerate()
            .any(|(i, p)| {
                p.default.is_none()
                    && !p.variadic
                    && !matches!(argv.get(i), Some(Arg::Val(_) | Arg::Ref(_)))
            });
        if missing_required {
            let passed = argv
                .iter()
                .filter(|a| matches!(a, Arg::Val(_) | Arg::Ref(_)))
                .count();
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}(), {} passed and {} expected",
                String::from_utf8_lossy(&f.name),
                passed,
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

        // Record a stack frame for the duration of the body (step 28); the
        // call-site line is the line currently executing in the caller.
        self.call_stack.push(CallFrame {
            class: None,
            function: f.name.to_vec(),
            is_static: false,
            line: self.cur_line as i64,
        });
        // A generator function does not run its body on call (step 39): bind the
        // arguments into the fresh frame, then hand that frame to a lazy
        // `Generator` value whose body runs on demand inside a coroutine.
        let result = if f.is_generator {
            match self.bind_params(f, argv) {
                Ok(()) => {
                    let frame = self.locals.take().expect("callee overlay installed");
                    Ok(self.make_generator(f, frame))
                }
                Err(e) => Err(e),
            }
        } else {
            self.run_user_fn_body(f, argv)
        };
        self.call_stack.pop();

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
        self.bind_params(f, argv)?;
        let ret = match self.exec_stmts(&f.body)? {
            Flow::Return(v) => v,
            // An unresolved `goto` escaping the function body can only be the
            // unsupported "jump *into* a transparent block" case (D-45.1):
            // lowering already proved the label exists in scope and is not a
            // forbidden into-loop/switch/finally jump. Surface it instead of
            // silently returning null.
            Flow::Goto(label) => return Err(unsupported_goto(&label)),
            _ => Zval::Null,
        };
        // Coerce the return value to a scalar return type (weak). A by-reference
        // function returns a `Zval::Ref` to alias, so its return type stays
        // unenforced here (scope-out, D-14.5/D-13.7).
        let strict = self.strict;
        match &f.ret_hint {
            Some(hint) if !f.by_ref => match coerce_to_hint(ret, hint, &mut self.diags, strict) {
                Ok(v) => Ok(v),
                Err(given) => Err(self.return_type_error(f, hint, given)),
            },
            _ => Ok(ret),
        }
    }

    /// Bind a call's arguments into the (already installed) callee frame's
    /// leading parameter slots. Shared by ordinary calls ([`run_user_fn_body`])
    /// and generator construction ([`make_generator`]), which binds the frame but
    /// does not run the body. By-value arguments are coerced to scalar hints
    /// (weak); by-reference arguments share the caller's cell; gaps fall back to
    /// defaults evaluated in the new frame.
    fn bind_params(&mut self, f: &'p FnDecl, argv: Vec<Arg>) -> Result<(), PhpError> {
        let strict = self.strict;
        for (i, p) in f.params.iter().enumerate() {
            // A variadic `...$rest` (always last) collects every remaining
            // argument into a 0-indexed array (step 38-5).
            if p.variadic {
                let mut arr = PhpArray::new();
                for a in argv.iter().skip(i) {
                    match a {
                        // Positional tail entries take the next free int key.
                        Arg::Val(v) => {
                            let _ = arr.append(v.clone());
                        }
                        Arg::Ref(cell) => {
                            let _ = arr.append(cell.borrow().clone());
                        }
                        // A named-into-variadic entry keeps its string key (step 40-2).
                        Arg::Named(name, v) => {
                            arr.insert(Key::Str(PhpStr::new(name.clone())), v.clone());
                        }
                        Arg::Default => continue,
                    }
                }
                frame_mut!(self)[p.slot as usize] = Zval::Array(Rc::new(arr));
                break;
            }
            let binding = match argv.get(i) {
                // A by-value argument is coerced to the parameter's scalar hint
                // under weak typing; a failure is an uncaught TypeError (D-14.4).
                // By-reference arguments and defaults are bound as-is.
                Some(Arg::Val(v)) => {
                    let val = v.clone();
                    match &p.hint {
                        Some(hint) => match coerce_to_hint(val, hint, &mut self.diags, strict) {
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
                // unreachable failure we keep the evaluated value. `Arg::Default`
                // is a gap left by named arguments (step 38) — same path as None.
                Some(Arg::Default) | None => {
                    let v = self.eval(p.default.as_ref().expect("required arg checked"))?;
                    match &p.hint {
                        Some(hint) => {
                            coerce_to_hint(v.clone(), hint, &mut self.diags, strict).unwrap_or(v)
                        }
                        None => v,
                    }
                }
                // `Arg::Named` is only ever appended past the variadic slot, so a
                // non-variadic parameter never sees one (step 40-2 invariant).
                Some(Arg::Named(..)) => {
                    unreachable!("named-into-variadic arg reached a fixed parameter slot")
                }
            };
            frame_mut!(self)[p.slot as usize] = binding;
        }
        Ok(())
    }

    // --- generators (step 39) ---

    /// Build the lazy `Generator` value a generator function returns. The
    /// argument-bound `frame` becomes the generator's initial locals; the body is
    /// cloned into an `Rc` so the `'static` coroutine can own it. The body does
    /// not run until the generator is first advanced.
    fn make_generator(&mut self, f: &FnDecl, frame: Vec<Zval>) -> Zval {
        let body_rc: Rc<FnDecl> = Rc::new(f.clone());
        let names_ptr: *const [Box<[u8]>] = body_rc.slots.as_slice();
        let ctx = GenCtx {
            locals: frame,
            local_names: names_ptr,
            // A generator defined in a method keeps its `$this` / class context
            // (captured here); a free-function generator has none.
            cur_this: self.cur_this.clone(),
            cur_class: self.cur_class,
            cur_static_class: self.cur_static_class,
            fn_returns_ref: false,
            gen_yielder: None,
        };
        let body_for_co = Rc::clone(&body_rc);
        let co = Coroutine::new(
            move |y: &Yielder<ResumeIn, YieldOut>, first: ResumeIn| -> Result<Zval, PhpError> {
                // SAFETY: see `GenDriverImpl::resume` — `first.ev` is the live
                // evaluator, valid for the whole body run; the reborrow as
                // `'static` is a lifetime extension that never escapes the call.
                let ev: &mut Evaluator<'static> =
                    unsafe { &mut *(first.ev as *mut Evaluator<'static>) };
                ev.gen_yielder = Some(y as *const Yielder<ResumeIn, YieldOut> as *const ());
                match ev.exec_stmts(&body_for_co.body) {
                    Ok(Flow::Return(v)) => Ok(v),
                    Ok(_) => Ok(Zval::Null),
                    Err(e) => Err(e),
                }
            },
        );
        let driver = GenDriverImpl {
            co,
            ctx,
            _body: body_rc,
        };
        let id = self.next_object_id;
        self.next_object_id += 1;
        Zval::Generator(Rc::new(RefCell::new(GenState {
            id,
            func_name: f.name.clone(),
            status: GenStatus::NotStarted,
            advanced: false,
            cur_key: Zval::Null,
            cur_val: Zval::Null,
            ret: Zval::Null,
            auto_key: 0,
            driver: Some(Box::new(driver)),
        })))
    }

    /// Drive a generator one step: resume its coroutine with `sent` (the value
    /// the suspended `yield` evaluates to), then record the outcome — a new
    /// `(key, value)` (resolving the auto-key) or completion (`getReturn` value /
    /// a propagated exception). The driver is taken out of [`GenState`] for the
    /// duration so a re-entrant resume of the *same* generator sees `Running` and
    /// errors cleanly (also upholding the reborrow's non-aliasing invariant).
    fn resume_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        sent: Zval,
    ) -> Result<(), PhpError> {
        let mut driver = {
            let mut gs = gs_rc.borrow_mut();
            match gs.status {
                GenStatus::Running => {
                    return Err(PhpError::Error(
                        "Cannot resume an already running generator".to_string(),
                    ))
                }
                GenStatus::Done => return Ok(()),
                // Resuming an already-suspended generator advances it past its
                // first element, which disallows a later `rewind()` (step 39-7).
                GenStatus::Suspended => gs.advanced = true,
                GenStatus::NotStarted => {}
            }
            gs.status = GenStatus::Running;
            gs.driver
                .take()
                .expect("driver present while generator not done")
        };
        // The borrow on `gs_rc` is released here, so the body may legally call
        // back into other generators (and a re-entrant call on *this* one hits
        // the `Running` guard above instead of a RefCell double-borrow).
        let step = driver.resume(sent, self as *mut Self as *mut ());
        let mut gs = gs_rc.borrow_mut();
        match step {
            GenStep::Yielded { key, value } => {
                let resolved = match key {
                    GenKey::Auto => {
                        let k = Zval::Long(gs.auto_key);
                        gs.auto_key += 1;
                        k
                    }
                    GenKey::Keyed(z) => {
                        // An integer key `>=` the counter advances it, mirroring
                        // array append semantics (D-GEN auto-key).
                        if let Zval::Long(n) = &z {
                            if *n >= gs.auto_key {
                                gs.auto_key = n.wrapping_add(1);
                            }
                        }
                        z
                    }
                    // `yield from` keys are forwarded as-is and do not advance the
                    // outer counter (step 39-6).
                    GenKey::Verbatim(z) => z,
                };
                gs.cur_key = resolved;
                gs.cur_val = value;
                gs.status = GenStatus::Suspended;
                gs.driver = Some(driver);
            }
            GenStep::Returned(res) => {
                gs.status = GenStatus::Done;
                gs.cur_key = Zval::Null;
                gs.cur_val = Zval::Null;
                // `driver` is dropped here (not stored back), unwinding/freeing the
                // coroutine stack.
                match res {
                    Ok(v) => gs.ret = v,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// Suspend the active generator at a `yield`, handing out `(key, value)` and
    /// returning the value the next resume delivers (step 39). The single point
    /// the `yield` / `yield from` arms reach the running coroutine's `Yielder`.
    fn gen_suspend(&mut self, key: GenKey, value: Zval) -> Result<Zval, PhpError> {
        let yptr = self.gen_yielder.ok_or_else(|| {
            PhpError::Error("Cannot use \"yield\" outside a generator".to_string())
        })?;
        // SAFETY: `gen_yielder` is set by the active generator body (in
        // `make_generator`'s coroutine) and read only while that body runs; the
        // `Yielder` lives for the coroutine's whole lifetime.
        let y = unsafe { &*(yptr as *const Yielder<ResumeIn, YieldOut>) };
        let resumed = y.suspend(YieldOut { key, value });
        Ok(resumed.sent)
    }

    /// `yield from <iterator>` (step 39-6): re-yield every element of the
    /// delegate *verbatim* (keys preserved, the outer auto-key counter
    /// untouched). For an array the expression evaluates to NULL; for a
    /// sub-generator it drives it (forwarding `send()` values in) and evaluates
    /// to its `return` value.
    fn eval_yield_from(&mut self, iter: &Expr) -> Result<Zval, PhpError> {
        let src = self.eval(iter)?.deref_clone();
        match src {
            Zval::Array(a) => {
                let pairs: Vec<(Key, Zval)> =
                    a.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
                for (k, v) in pairs {
                    // A sent value is discarded when delegating to an array.
                    self.gen_suspend(GenKey::Verbatim(key_to_zval(&k)), v)?;
                }
                Ok(Zval::Null)
            }
            Zval::Generator(sub) => {
                self.ensure_started(&sub)?;
                loop {
                    let (k, v, done) = {
                        let g = sub.borrow();
                        (
                            g.cur_key.clone(),
                            g.cur_val.clone(),
                            matches!(g.status, GenStatus::Done),
                        )
                    };
                    if done {
                        break;
                    }
                    // Re-yield the sub-generator's current pair; forward the value
                    // the consumer sends back into the sub-generator.
                    let sent = self.gen_suspend(GenKey::Verbatim(k), v)?;
                    self.resume_generator(&sub, sent)?;
                }
                // The `yield from` expression evaluates to the delegate's return.
                let ret = sub.borrow().ret.clone();
                Ok(ret)
            }
            // `yield from` over a user `Traversable`/`Iterator` is a companion of
            // the (still scoped-out) generic `foreach` over objects; catalogue if
            // the corpus needs it (step 39 scope-out). The message matches PHP's
            // exactly (Zend/zend_generators.c).
            _ => Err(PhpError::Error(
                "Can use \"yield from\" only with arrays and Traversables".to_string(),
            )),
        }
    }

    /// Start a generator if it has not run yet (PHP starts lazily on the first
    /// `current`/`key`/`valid`/`next`/`foreach`).
    fn ensure_started(&mut self, gs_rc: &Rc<RefCell<GenState>>) -> Result<(), PhpError> {
        if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
            self.resume_generator(gs_rc, Zval::Null)?;
        }
        Ok(())
    }

    /// Built-in methods on a `Generator` value (the `Iterator` interface plus
    /// `send`/`getReturn`), step 39. Dispatched like [`closure_method`], ahead of
    /// user-class method resolution.
    fn generator_method(
        &mut self,
        gs_rc: Rc<RefCell<GenState>>,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"current") {
            self.ensure_started(&gs_rc)?;
            Ok(gs_rc.borrow().cur_val.clone())
        } else if method.eq_ignore_ascii_case(b"key") {
            self.ensure_started(&gs_rc)?;
            Ok(gs_rc.borrow().cur_key.clone())
        } else if method.eq_ignore_ascii_case(b"next") {
            self.ensure_started(&gs_rc)?;
            self.resume_generator(&gs_rc, Zval::Null)?;
            Ok(Zval::Null)
        } else if method.eq_ignore_ascii_case(b"valid") {
            self.ensure_started(&gs_rc)?;
            let done = matches!(gs_rc.borrow().status, GenStatus::Done);
            Ok(Zval::Bool(!done))
        } else if method.eq_ignore_ascii_case(b"rewind") {
            // Starts the generator (lazily). Rewinding one already advanced past
            // its first element is a fatal (step 39-7).
            self.ensure_started(&gs_rc)?;
            if gs_rc.borrow().advanced {
                return Err(PhpError::Error(
                    "Cannot rewind a generator that was already run".to_string(),
                ));
            }
            Ok(Zval::Null)
        } else if method.eq_ignore_ascii_case(b"send") {
            // Resume delivering `$value` as the result of the suspended `yield`
            // (step 39-4). An unstarted generator is primed first.
            let value = argv.into_iter().next().unwrap_or(Zval::Null);
            if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
                self.resume_generator(&gs_rc, Zval::Null)?;
            }
            self.resume_generator(&gs_rc, value)?;
            Ok(gs_rc.borrow().cur_val.clone())
        } else if method.eq_ignore_ascii_case(b"getReturn") {
            // PHP auto-primes here: getReturn() on a fresh generator starts it
            // (so one whose body returns before any yield exposes its value); if
            // it has not yet returned, that is an Error.
            self.ensure_started(&gs_rc)?;
            if !matches!(gs_rc.borrow().status, GenStatus::Done) {
                return Err(PhpError::Error(
                    "Cannot get return value of a generator that hasn't returned".to_string(),
                ));
            }
            Ok(gs_rc.borrow().ret.clone())
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Generator::{}()",
                String::from_utf8_lossy(method)
            )))
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
    /// Evaluate a user function's call arguments into the positional `argv` plus
    /// the named arguments produced by unpacking string keys (step 40). Plain
    /// positional arguments (which lowering guarantees precede any spread) honour
    /// by-reference parameters; unpacked values are always by value.
    fn eval_call_args(
        &mut self,
        idx: usize,
        args: &[Expr],
    ) -> Result<(Vec<Arg>, SpreadNamed), PhpError> {
        let funcs: &'p [FnDecl] = self.funcs;
        let f: &'p FnDecl = &funcs[idx];
        let mut out: Vec<Arg> = Vec::with_capacity(args.len());
        let mut named: SpreadNamed = Vec::new();
        for a in args {
            if let ExprKind::Spread(inner) = &a.kind {
                let mut pos = Vec::new();
                self.expand_spread(inner, &mut pos, &mut named)?;
                out.extend(pos.into_iter().map(Arg::Val));
                continue;
            }
            // A plain positional binds at the next positional slot; only these
            // (never unpacked values) may target a by-reference parameter.
            let i = out.len();
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
        Ok((out, named))
    }

    /// The catchable `Error` PHP raises for an unresolvable named argument; used
    /// for the scope-out targets (closures, `__call`, enum statics) that have no
    /// declared parameter list to bind names against (step 38).
    /// Reject any named argument — explicit (step 38) or produced by unpacking
    /// string keys (step 40) — for a target that has no parameter list to bind
    /// against (closures, generators, `__call`). Returns the unknown-parameter
    /// `Error` for the first offending name, or `None` if there are none.
    fn reject_named(
        &self,
        named: &[(Box<[u8]>, Expr)],
        spread_named: &[(Box<[u8]>, Zval)],
    ) -> Option<PhpError> {
        let name = named
            .first()
            .map(|(n, _)| n.as_ref())
            .or_else(|| spread_named.first().map(|(n, _)| n.as_ref()))?;
        Some(PhpError::Error(format!(
            "Unknown named parameter ${}",
            String::from_utf8_lossy(name)
        )))
    }

    /// Place one already-built named argument into `argv` by parameter name
    /// (steps 38 / 40-2). A matching non-variadic parameter takes the value at
    /// its slot (filling earlier gaps with `Arg::Default`); an already-filled
    /// slot is an overwrite `Error`. With no matching parameter, a trailing
    /// variadic collects the value keyed by name ([`Arg::Named`]); otherwise the
    /// name is an unknown-parameter `Error`. Messages match PHP.
    fn place_named_arg(
        &self,
        argv: &mut Vec<Arg>,
        f: &FnDecl,
        name: &[u8],
        arg: Arg,
    ) -> Result<(), PhpError> {
        if let Some(j) = f
            .params
            .iter()
            .position(|p| !p.variadic && f.slots[p.slot as usize][..] == name[..])
        {
            // A positional argument already occupied this slot, or a duplicate
            // named argument targets it (an `Arg::Default` gap is not "previous").
            if matches!(argv.get(j), Some(Arg::Val(_) | Arg::Ref(_))) {
                return Err(PhpError::Error(format!(
                    "Named parameter ${} overwrites previous argument",
                    String::from_utf8_lossy(name)
                )));
            }
            if argv.len() <= j {
                argv.resize_with(j + 1, || Arg::Default);
            }
            argv[j] = arg;
            Ok(())
        } else if f.params.last().is_some_and(|p| p.variadic) {
            // No matching fixed parameter, but a trailing `...$rest` collects the
            // named argument keyed by its name (step 40-2). A by-reference value
            // is dereferenced — variadics collect by value.
            let val = match arg {
                Arg::Val(v) => v,
                Arg::Ref(cell) => cell.borrow().clone(),
                Arg::Default => return Ok(()),
                Arg::Named(_, v) => v,
            };
            argv.push(Arg::Named(name.into(), val));
            Ok(())
        } else {
            Err(PhpError::Error(format!(
                "Unknown named parameter ${}",
                String::from_utf8_lossy(name)
            )))
        }
    }

    /// Apply a call's named arguments to the positional `argv`: first the named
    /// arguments produced by string keys during unpacking (step 40, already
    /// evaluated, by value), then the explicit `name: value` arguments (step 38),
    /// each evaluated in the caller frame. A name targeting a by-reference
    /// parameter binds the caller's variable cell (step 38-4).
    fn apply_named_args(
        &mut self,
        f: &'p FnDecl,
        mut argv: Vec<Arg>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Vec<Arg>, PhpError> {
        for (name, val) in spread_named {
            self.place_named_arg(&mut argv, f, &name, Arg::Val(val))?;
        }
        for (name, expr) in named {
            // A by-reference parameter binds the caller's variable cell when the
            // named value is a plain variable (mirrors `eval_call_args`); a
            // non-variable to a by-ref param is the same fatal as positionally.
            let target = f
                .params
                .iter()
                .position(|p| !p.variadic && f.slots[p.slot as usize][..] == name[..]);
            let arg = match target {
                Some(j) if f.params[j].by_ref => match &expr.kind {
                    ExprKind::Var(slot) => Arg::Ref(self.slot_cell(*slot as usize)),
                    _ => {
                        return Err(PhpError::Error(format!(
                            "{}(): Argument #{} (${}) could not be passed by reference",
                            String::from_utf8_lossy(&f.name),
                            j + 1,
                            String::from_utf8_lossy(name),
                        )))
                    }
                },
                _ => Arg::Val(self.eval(expr)?),
            };
            self.place_named_arg(&mut argv, f, name, arg)?;
        }
        Ok(argv)
    }

    /// Expand one unpacked value (`...$e`, step 40) into the positional `pos`
    /// stream and the `named` stream. Array/Traversable int keys append to
    /// `pos` in iteration order (the key value is ignored); string keys append
    /// to `named`. An int key after any string key already emitted during this
    /// call's unpacking is a catchable `Error`; a non-iterable is a `TypeError`.
    fn expand_spread(
        &mut self,
        inner: &Expr,
        pos: &mut Vec<Zval>,
        named: &mut SpreadNamed,
    ) -> Result<(), PhpError> {
        // A positional value produced by unpacking is rejected once any named
        // (string-keyed) value has already been emitted, matching PHP.
        macro_rules! push_pos {
            ($v:expr) => {{
                if !named.is_empty() {
                    return Err(PhpError::Error(
                        "Cannot use positional argument after named argument during unpacking"
                            .to_string(),
                    ));
                }
                pos.push($v);
            }};
        }
        let value = self.eval(inner)?.deref_clone();
        match value {
            Zval::Array(arr) => {
                for (k, v) in arr.iter() {
                    match k {
                        Key::Int(_) => push_pos!(v.clone()),
                        Key::Str(s) => named.push((s.as_bytes().into(), v.clone())),
                    }
                }
                Ok(())
            }
            Zval::Generator(gs) => {
                self.ensure_started(&gs)?;
                loop {
                    let (k, v, done) = {
                        let g = gs.borrow();
                        (
                            g.cur_key.clone(),
                            g.cur_val.deref_clone(),
                            matches!(g.status, GenStatus::Done),
                        )
                    };
                    if done {
                        break;
                    }
                    match k {
                        Zval::Str(s) => named.push((s.as_bytes().into(), v)),
                        _ => push_pos!(v),
                    }
                    self.resume_generator(&gs, Zval::Null)?;
                }
                Ok(())
            }
            other => Err(PhpError::TypeError(format!(
                "Only arrays and Traversables can be unpacked, {} given",
                other.error_type_name()
            ))),
        }
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

    /// Run a by-value builtin, mirroring its fresh stdout into `rendered` and
    /// flushing its diagnostics, exactly like the `Call` dispatch path. Shared by
    /// direct calls and dynamic string-callable dispatch (step 18).
    fn dispatch_value_builtin(
        &mut self,
        f: crate::builtin::BuiltinFn,
        argv: &[Zval],
    ) -> Result<Zval, PhpError> {
        self.flush_diags();
        let pre = self.out.len();
        let mut ctx = Ctx {
            out: &mut self.out,
            diags: &mut self.diags,
        };
        let res = f(argv, &mut ctx);
        let produced = self.out[pre..].to_vec();
        self.rendered.extend_from_slice(&produced);
        self.flush_diags();
        res
    }

    /// Allocate the next monotonic object handle (the `#N` in `var_dump`).
    fn next_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Build the render metadata for a first-class callable `name(...)`: the
    /// parameters are known only when it wraps a user function; a builtin target
    /// has no signature available, so its parameter list is empty (step 18-7
    /// scope-out).
    fn first_class_info(&self, name: &[u8]) -> Rc<ClosureInfo> {
        let params = match self.fn_index.get(&name.to_ascii_lowercase()) {
            Some(&idx) => closure_params_for(&self.funcs[idx]),
            None => Vec::new(),
        };
        Rc::new(ClosureInfo {
            kind: ClosureRender::Function(PhpStr::new(name.to_vec())),
            params,
        })
    }

    /// Snapshot a closure's captured variables in the *active* frame (step 18,
    /// D-18.3): a by-value `use($x)` reads (and warns on undefined) the value; a
    /// by-reference `use(&$x)` shares the variable's cell as a `Zval::Ref`.
    fn bind_captures(&mut self, captures: &[Capture]) -> Vec<(u32, Zval)> {
        let mut out = Vec::with_capacity(captures.len());
        for cap in captures {
            let val = if cap.by_ref {
                Zval::Ref(self.slot_cell(cap.src as usize))
            } else {
                self.read_var(cap.src)
            };
            out.push((cap.dst, val));
        }
        out
    }

    /// Invoke a runtime callable value (step 18, D-18.5): a closure runs its
    /// lowered body; a string names a user function or builtin; anything else is
    /// an uncaught `Error` ("Value of type X is not callable").
    fn call_value(&mut self, callee: Zval, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                // A first-class callable dispatches like a string callable.
                Some(name) => self.call_named(name.as_bytes(), argv),
                None => self.call_closure(&cl, argv),
            },
            Zval::Str(s) => self.call_named(s.as_bytes(), argv),
            Zval::Ref(cell) => {
                let inner = cell.borrow().clone();
                self.call_value(inner, argv)
            }
            // An object is callable iff it defines `__invoke` (step 22, D-22.7).
            Zval::Object(ref o) => {
                let cid = o.borrow().class_id as usize;
                match self.resolve_method(cid, b"__invoke") {
                    Some((defc, m)) => {
                        self.invoke_method(Some(callee.clone()), defc, cid, m, b"__invoke", argv)
                    }
                    None => Err(PhpError::Error(format!(
                        "Object of type {} is not callable",
                        String::from_utf8_lossy(&self.classes[cid].name)
                    ))),
                }
            }
            other => Err(PhpError::Error(format!(
                "Value of type {} is not callable",
                other.error_type_name()
            ))),
        }
    }

    /// Dispatch a string callable to a user function (shadows builtins) or a
    /// by-value builtin. Arguments arrive by value (by-reference parameters in a
    /// dynamic string call are a scope-out, D-18.5).
    fn call_named(&mut self, name: &[u8], argv: Vec<Zval>) -> Result<Zval, PhpError> {
        if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
            let args: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
            let result = self.call_user_fn(idx, args)?;
            return Ok(match result {
                Zval::Ref(cell) => cell.borrow().clone(),
                other => other,
            });
        }
        match self.reg.get(name).copied() {
            Some(Builtin::Value(f)) => self.dispatch_value_builtin(f, &argv),
            // String-calling a by-reference builtin (sort/array_push/…) is a
            // documented scope-out (it needs a variable, not a value).
            Some(Builtin::RefFirst(_)) => Err(PhpError::Error(format!(
                "{}(): cannot be called dynamically with a by-value argument",
                String::from_utf8_lossy(name)
            ))),
            None => Err(PhpError::Error(format!(
                "Call to undefined function {}()",
                String::from_utf8_lossy(name)
            ))),
        }
    }

    /// Invoke a closure value: install its frame, bind the captured variables
    /// into their slots, then bind parameters and run the body via the shared
    /// [`Evaluator::run_user_fn_body`] (step 18, D-18.2).
    fn call_closure(&mut self, cl: &Closure, argv: Vec<Zval>) -> Result<Zval, PhpError> {
        let closures: &'p [FnDecl] = self.closures;
        let f: &'p FnDecl = &closures[cl.fn_idx];

        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
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

        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);
        // Install the closure's bound `$this` (step 19-6); a static / top-level
        // closure carries `None`, so `$this` inside it is the usual fatal.
        let saved_this = std::mem::replace(&mut self.cur_this, cl.bound_this.clone());

        // Bind captured variables into their closure-frame slots before params.
        for (slot, val) in &cl.captures {
            frame_mut!(self)[*slot as usize] = val.clone();
        }

        let args: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
        // A generator closure (its body contains `yield`) returns a lazy
        // Generator just like a generator function (step 39): bind params into
        // the frame (which already holds the captures), then hand it off.
        let result = if f.is_generator {
            match self.bind_params(f, args) {
                Ok(()) => {
                    let frame = self.locals.take().expect("closure overlay installed");
                    Ok(self.make_generator(f, frame))
                }
                Err(e) => Err(e),
            }
        } else {
            self.run_user_fn_body(f, args)
        };

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        self.cur_this = saved_this;
        result.map(|r| match r {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        })
    }

    /// Build a copy of a closure with a new bound `$this` (step 19-6, D-19.19),
    /// assigning it a fresh object handle.
    fn rebind_closure(&mut self, cl: &Closure, bound_this: Option<Zval>) -> Zval {
        Zval::Closure(Rc::new(Closure {
            fn_idx: cl.fn_idx,
            captures: cl.captures.clone(),
            named: cl.named.clone(),
            bound_this,
            id: self.next_id(),
            info: Rc::clone(&cl.info),
        }))
    }

    /// Built-in methods on a closure value (`$c->bindTo(...)`, `$c->call(...)`),
    /// step 19-6, D-19.19.
    fn closure_method(
        &mut self,
        cl: &Closure,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bindTo") {
            let new_this = closure_this_arg(argv.into_iter().next());
            Ok(self.rebind_closure(cl, new_this))
        } else if method.eq_ignore_ascii_case(b"call") {
            // `$c->call($newThis, ...args)`: bind then invoke immediately.
            let mut it = argv.into_iter();
            let new_this = closure_this_arg(it.next());
            let rest: Vec<Zval> = it.collect();
            let bound = self.rebind_closure(cl, new_this);
            self.call_value(bound, rest)
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Closure::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }

    /// Static methods on the `Closure` class (`Closure::bind`,
    /// `Closure::fromCallable`), step 19-6, D-19.19.
    fn closure_static(&mut self, method: &[u8], argv: Vec<Zval>) -> Result<Zval, PhpError> {
        if method.eq_ignore_ascii_case(b"bind") {
            let mut it = argv.into_iter();
            let target = it.next().map(|v| v.deref_clone());
            let new_this = closure_this_arg(it.next());
            match target {
                Some(Zval::Closure(cl)) => Ok(self.rebind_closure(&cl, new_this)),
                _ => Err(PhpError::Error(
                    "Closure::bind(): Argument #1 ($closure) must be of type Closure".to_string(),
                )),
            }
        } else if method.eq_ignore_ascii_case(b"fromCallable") {
            match argv.into_iter().next().map(|v| v.deref_clone()) {
                Some(Zval::Closure(cl)) => Ok(Zval::Closure(cl)),
                Some(Zval::Str(s)) => {
                    let info = self.first_class_info(s.as_bytes());
                    Ok(Zval::Closure(Rc::new(Closure {
                        fn_idx: 0,
                        captures: Vec::new(),
                        named: Some(s),
                        bound_this: None,
                        id: self.next_id(),
                        info,
                    })))
                }
                _ => Err(PhpError::Error(
                    "Closure::fromCallable(): Argument #1 ($callback) is not callable".to_string(),
                )),
            }
        } else {
            Err(PhpError::Error(format!(
                "Call to undefined method Closure::{}()",
                String::from_utf8_lossy(method)
            )))
        }
    }

    // --- objects (step 19) ---

    /// Evaluate a call's arguments by value into a flat positional vector plus
    /// the named arguments produced by unpacking string keys (step 40). Shared
    /// by method calls and constructor dispatch. Methods take all positional
    /// arguments by value, so unpacking has no by-reference subtlety here.
    fn eval_value_args(
        &mut self,
        args: &[Expr],
    ) -> Result<(Vec<Zval>, SpreadNamed), PhpError> {
        let mut out = Vec::with_capacity(args.len());
        let mut named: SpreadNamed = Vec::new();
        for a in args {
            if let ExprKind::Spread(inner) = &a.kind {
                self.expand_spread(inner, &mut out, &mut named)?;
            } else {
                out.push(self.eval(a)?);
            }
        }
        Ok((out, named))
    }

    /// Resolve a method by name, walking the inheritance chain child→ancestor
    /// (step 19-3, D-19.10). Returns the *defining* class id and the method.
    fn resolve_method(&self, start: ClassId, name: &[u8]) -> Option<(ClassId, &'p MethodDecl)> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cid = Some(start);
        while let Some(c) = cid {
            if let Some(m) = classes[c]
                .methods
                .iter()
                .find(|m| m.decl.name.eq_ignore_ascii_case(name))
            {
                return Some((c, m));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// Assemble an instance's property map: parent declarations first, then each
    /// subclass, so the layout order is root→leaf and a redeclared property keeps
    /// its inherited position with the subclass's default (step 19-3, D-19.10).
    fn collect_props(&mut self, cid: ClassId) -> Result<Props, PhpError> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut chain = Vec::new();
        let mut c = Some(cid);
        while let Some(x) = c {
            chain.push(x);
            c = classes[x].parent;
        }
        chain.reverse();
        let mut props = Props::new();
        for &x in &chain {
            for p in &classes[x].props {
                let v = match &p.default {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                props.set(&p.name, v);
            }
        }
        Ok(props)
    }

    /// Build (and cache) a class's property-visibility shape for object dumping
    /// (step 19-7, D-19.20): declared properties in root→leaf order, a redeclared
    /// property taking the most-derived visibility.
    fn class_shape(&mut self, cid: ClassId) -> Rc<ObjectInfo> {
        if let Some(s) = self.class_shapes.get(&cid) {
            return Rc::clone(s);
        }
        let classes: &'p [ClassDecl] = self.classes;
        let mut chain = Vec::new();
        let mut c = Some(cid);
        while let Some(x) = c {
            chain.push(x);
            c = classes[x].parent;
        }
        chain.reverse();
        let mut entries: Vec<(Box<[u8]>, PropVis)> = Vec::new();
        for &x in &chain {
            let cname = PhpStr::new(classes[x].name.to_vec());
            for p in &classes[x].props {
                let vis = match p.visibility {
                    Visibility::Public => PropVis::Public,
                    Visibility::Protected => PropVis::Protected,
                    Visibility::Private => PropVis::Private(Rc::clone(&cname)),
                };
                match entries.iter_mut().find(|(k, _)| k.as_ref() == p.name.as_ref()) {
                    Some(e) => e.1 = vis,
                    None => entries.push((p.name.clone(), vis)),
                }
            }
        }
        let info = Rc::new(ObjectInfo::from_entries(entries));
        self.class_shapes.insert(cid, Rc::clone(&info));
        info
    }

    /// The visibility and *declaring* class of a declared property, found by
    /// walking the chain child→ancestor. `None` for a dynamic/undeclared property
    /// (which is effectively public), step 19-3, D-19.13.
    fn resolve_prop_decl(&self, class_id: ClassId, name: &[u8]) -> Option<(Visibility, ClassId)> {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cid = Some(class_id);
        while let Some(c) = cid {
            if let Some(p) = classes[c].props.iter().find(|p| p.name.as_ref() == name) {
                return Some((p.visibility, c));
            }
            cid = classes[c].parent;
        }
        None
    }

    /// Whether `a` is `b` or descends from it (used for protected access checks).
    fn class_is_a(&self, a: ClassId, b: ClassId) -> bool {
        let classes: &'p [ClassDecl] = self.classes;
        let mut cur = Some(a);
        while let Some(c) = cur {
            if c == b {
                return true;
            }
            cur = classes[c].parent;
        }
        false
    }

    /// Whether the given visibility, declared on `decl_class`, is accessible from
    /// the current class context (`self.cur_class`), step 19-3, D-19.13.
    fn visible_from(&self, vis: Visibility, decl_class: ClassId) -> bool {
        match vis {
            Visibility::Public => true,
            Visibility::Private => self.cur_class == Some(decl_class),
            // Protected: accessible from anywhere in the same hierarchy.
            Visibility::Protected => matches!(
                self.cur_class,
                Some(cc) if self.class_is_a(cc, decl_class) || self.class_is_a(decl_class, cc)
            ),
        }
    }

    /// Enforce property visibility for an access on `class_id`. A dynamic /
    /// undeclared property is always accessible (public).
    fn check_prop_access(&self, class_id: ClassId, name: &[u8]) -> Result<(), PhpError> {
        let Some((vis, decl_class)) = self.resolve_prop_decl(class_id, name) else {
            return Ok(());
        };
        if self.visible_from(vis, decl_class) {
            return Ok(());
        }
        let kind = if matches!(vis, Visibility::Private) {
            "private"
        } else {
            "protected"
        };
        Err(PhpError::Error(format!(
            "Cannot access {kind} property {}::${}",
            String::from_utf8_lossy(&self.classes[decl_class].name),
            String::from_utf8_lossy(name)
        )))
    }

    /// If the first place step is a property, enforce its visibility against the
    /// object the base designates (write/unset contexts), step 19-3. Deeper
    /// properties in a chain are not checked (19-3 simplification). When a magic
    /// accessor (`__set`/`__unset`) will handle a missing-or-inaccessible
    /// property, the visibility error is suppressed so the magic call can run
    /// (step 22, D-22.2).
    fn check_first_prop_write(
        &self,
        base: PlaceBase,
        steps: &[Step],
        kind: MagicAccess,
        magic_name: &[u8],
    ) -> Result<(), PhpError> {
        if let Some(Step::Prop(name)) = steps.first() {
            if let Zval::Object(o) = self.base_clone(base) {
                if self.magic_prop_method(&o, name, kind, magic_name).is_some() {
                    return Ok(());
                }
                let cid = o.borrow().class_id as usize;
                return self.check_prop_access(cid, name);
            }
        }
        Ok(())
    }

    /// Resolve a [`ClassRef`] to a class id in the current context (step 19-4):
    /// a named class via the class table, `self`/`parent` via the defining class,
    /// `static` via the late-static-binding class.
    fn resolve_class_ref(&self, class: &ClassRef) -> Result<ClassId, PhpError> {
        match class {
            ClassRef::Named(name) => self
                .class_index
                .get(&name.to_ascii_lowercase())
                .copied()
                .ok_or_else(|| {
                    PhpError::Error(format!(
                        "Class \"{}\" not found",
                        String::from_utf8_lossy(name)
                    ))
                }),
            ClassRef::SelfClass => self
                .cur_class
                .ok_or_else(|| PhpError::Error("Cannot use \"self\" outside class context".into())),
            ClassRef::Parent => self
                .cur_class
                .and_then(|c| self.classes[c].parent)
                .ok_or_else(|| {
                    PhpError::Error(
                        "Cannot use \"parent\" when current class scope has no parent".into(),
                    )
                }),
            ClassRef::Static => self.cur_static_class.ok_or_else(|| {
                PhpError::Error("Cannot use \"static\" outside class context".into())
            }),
        }
    }

    /// Evaluate `new ClassRef(args)` (step 19, D-19.6/D-19.12): resolve the class
    /// (including `self`/`static` late binding), build an instance with the full
    /// inherited property set, then run `__construct` (resolved up the chain).
    fn eval_new(
        &mut self,
        class: &ClassRef,
        args: &[Expr],
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        // An enum has no constructor and cannot be instantiated (step 23, D-23.9).
        if self.classes[cid].is_enum {
            return Err(PhpError::Error(format!(
                "Cannot instantiate enum {}",
                String::from_utf8_lossy(&self.classes[cid].name)
            )));
        }
        // An abstract class or interface cannot be instantiated (step 19-5).
        if self.classes[cid].is_abstract {
            let what = if self.classes[cid].is_interface {
                "interface"
            } else {
                "abstract class"
            };
            return Err(PhpError::Error(format!(
                "Cannot instantiate {what} {}",
                String::from_utf8_lossy(&self.classes[cid].name)
            )));
        }
        let class_name = PhpStr::new(self.classes[cid].name.to_vec());
        let props = self.collect_props(cid)?;
        let info = self.class_shape(cid);
        let id = self.next_id();
        let obj = Object {
            class_id: cid as u32,
            class_name,
            props,
            id,
            info,
        };
        let value = Zval::Object(Rc::new(RefCell::new(obj)));
        // Track the new instance for `__destruct` (step 24-2/24-3): an extra
        // strong ref whose presence is later used to detect unreachability.
        if let Zval::Object(o) = &value {
            self.created.push(o.clone());
        }
        // A Throwable records its creation site (`getLine`/`getFile`) at `new`
        // time, before the constructor runs (step 20). PHP sets these from the
        // engine, not from `Exception::__construct`.
        if self.is_throwable(cid) {
            // Capture the trace at construction (step 28), before the constructor
            // runs — PHP snapshots the stack at `new`, not at `throw`.
            let (trace, trace_string) = self.capture_trace();
            if let Zval::Object(o) = &value {
                let create_line = self.cur_line as i64;
                let mut b = o.borrow_mut();
                b.props.set(b"line", Zval::Long(create_line));
                b.props.set(b"file", Zval::Str(PhpStr::new(self.file.to_vec())));
                b.props.set(b"trace", trace);
                b.props.set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
            }
        }
        // Run the constructor (inherited if not overridden); its mutations write
        // through the shared `Rc`, so they show in the returned value. The new
        // instance's class is its own LSB class.
        if let Some((defc, m)) = self.resolve_method(cid, b"__construct") {
            // Positional args (incl. unpacked, step 40), then named placed by
            // parameter name (step 38-2).
            let (vals, spread_named) = self.eval_value_args(args)?;
            let argv: Vec<Arg> = vals.into_iter().map(Arg::Val).collect();
            let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
            self.invoke_method_args(Some(value.clone()), defc, cid, m, b"__construct", argv)?;
        } else if !named.is_empty() || !args.is_empty() {
            // No constructor: named args would have nowhere to bind. PHP ignores
            // extra args to a default constructor, but a named arg is an Error.
            // Keep parity with the no-ctor positional path (args ignored); a named
            // arg to a constructor-less class is rare — treat as unknown param.
            if let Some((name, _)) = named.first() {
                return Err(PhpError::Error(format!(
                    "Unknown named parameter ${}",
                    String::from_utf8_lossy(name)
                )));
            }
        }
        Ok(value)
    }

    /// Run `__destruct` on object `o` exactly once, if it declares one. The
    /// caller is responsible for having removed `o` from `created` already.
    fn run_one_destructor(&mut self, o: &Rc<RefCell<Object>>) {
        let (cid, id) = {
            let b = o.borrow();
            (b.class_id as usize, b.id)
        };
        if self.destructed.contains(&id) {
            return;
        }
        if let Some((defc, m)) = self.resolve_method(cid, b"__destruct") {
            self.destructed.insert(id);
            let value = Zval::Object(o.clone());
            // A destructor that throws is swallowed: its unwinding would otherwise
            // abort the remaining destructors. PHP turns it into a shutdown fatal;
            // refining that is future work.
            let _ = self.invoke_method(Some(value), defc, cid, m, b"__destruct", Vec::new());
            self.flush_diags();
        }
    }

    /// Mid-script destruction sweep (step 24-3): release every tracked object the
    /// program can no longer reach (`Rc::strong_count == 1`, i.e. only the
    /// tracking ref remains), most-recently-created first. Running one destructor
    /// or dropping a destructor-less object may make another unreachable
    /// (transitively, e.g. an object held only by a now-freed array), so the scan
    /// repeats until a fixpoint. Called at global-scope statement boundaries;
    /// destructor bodies run with a local frame, so the `locals.is_none()` gate at
    /// the call site keeps this from re-entering.
    fn sweep_destructors(&mut self) {
        loop {
            let idx = self
                .created
                .iter()
                .rposition(|o| Rc::strong_count(o) == 1);
            let Some(i) = idx else { break };
            let o = self.created.remove(i);
            self.run_one_destructor(&o);
            // `o` drops here, possibly releasing another tracked object.
        }
    }

    /// End-of-script shutdown (step 24-2): invoke `__destruct` on every object
    /// still tracked at the end of the run, in reverse creation order (PHP
    /// shutdown is LIFO). These are the objects still reachable when the script
    /// ends (e.g. held by globals); mid-script releases were already handled by
    /// [`Evaluator::sweep_destructors`].
    fn run_destructors(&mut self) {
        let survivors: Vec<Rc<RefCell<Object>>> = std::mem::take(&mut self.created);
        for o in survivors.into_iter().rev() {
            self.run_one_destructor(&o);
        }
    }

    /// Resolve and evaluate a class constant `Class::NAME` (step 19-4, D-19.15),
    /// or the special `Class::class` (the class name string). The constant's
    /// value expression is evaluated in its *declaring* class's context.
    fn eval_class_const(&mut self, class: &ClassRef, name: &[u8]) -> Result<Zval, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        if name.eq_ignore_ascii_case(b"class") {
            return Ok(Zval::Str(PhpStr::new(self.classes[cid].name.to_vec())));
        }
        // An enum case (`E::Case`) is matched case-sensitively before user
        // constants, and resolves to the interned singleton (step 23, D-23.2).
        if self.classes[cid].is_enum
            && self.classes[cid].enum_cases.iter().any(|c| c.name.as_ref() == name)
        {
            return self.eval_enum_case(cid, name);
        }
        // Resolve the constant through the parent chain *and* implemented
        // interfaces (interface constants are inherited too — gh7821, also a
        // general class gap surfaced by enums in step 23).
        let (decl_class, expr) = self.find_class_const(cid, name).ok_or_else(|| {
            PhpError::Error(format!(
                "Undefined constant {}::{}",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(name)
            ))
        })?;
        // Evaluate in the declaring class's context so a `self::OTHER` inside the
        // constant resolves correctly.
        let saved_class = self.cur_class.replace(decl_class);
        let result = self.eval(expr);
        self.cur_class = saved_class;
        result
    }

    /// Find a class constant by name, searching the class's own constants, then
    /// its parent chain, then (transitively) its implemented interfaces. Returns
    /// the declaring class id and the value expression (step 23 / gh7821).
    fn find_class_const(&self, cid: ClassId, name: &[u8]) -> Option<(ClassId, &'p Expr)> {
        let classes: &'p [ClassDecl] = self.classes;
        // Own constants + parent chain take precedence.
        let mut c = Some(cid);
        while let Some(x) = c {
            if let Some(k) = classes[x].consts.iter().find(|k| k.name.as_ref() == name) {
                return Some((x, &k.value));
            }
            c = classes[x].parent;
        }
        // Then interfaces of the class and its ancestors (transitively).
        let mut c = Some(cid);
        while let Some(x) = c {
            for &i in &classes[x].interfaces {
                if let Some(r) = self.find_class_const(i, name) {
                    return Some(r);
                }
            }
            c = classes[x].parent;
        }
        None
    }

    /// Return the interned singleton object for enum case `E::name`, creating it
    /// on first access (step 23, D-23.2/D-23.4). The case is guaranteed to exist
    /// (the caller checked). Synthesises the read-only `name` (and, for a backed
    /// enum, `value`) properties; the object carries the enum's class id so the
    /// whole OOP machinery (`instanceof`, method dispatch, `$this`) applies.
    fn eval_enum_case(&mut self, cid: ClassId, name: &[u8]) -> Result<Zval, PhpError> {
        let key = (cid, name.to_vec());
        if let Some(o) = self.enum_cache.get(&key) {
            return Ok(Zval::Object(Rc::clone(o)));
        }
        let classes: &'p [ClassDecl] = self.classes;
        let case = classes[cid]
            .enum_cases
            .iter()
            .find(|c| c.name.as_ref() == name)
            .expect("caller verified the case exists");
        let mut props = Props::new();
        let mut entries: Vec<(Box<[u8]>, PropVis)> =
            vec![(Box::from(&b"name"[..]), PropVis::Public)];
        props.set(b"name", Zval::Str(PhpStr::new(name.to_vec())));
        if let Some(expr) = &case.value {
            // The backing value is a compile-time literal of the declared type
            // (PHP rejects a mismatch at link time), so it is stored as-is once,
            // when the singleton is first materialised (step 23, D-23.4/D-23.10).
            let saved = self.cur_class.replace(cid);
            let value = self.eval(expr);
            self.cur_class = saved;
            props.set(b"value", value?);
            entries.push((Box::from(&b"value"[..]), PropVis::Public));
        }
        let id = self.next_id();
        let obj = Object {
            class_id: cid as u32,
            class_name: PhpStr::new(classes[cid].name.to_vec()),
            props,
            id,
            info: Rc::new(ObjectInfo::enum_case(entries)),
        };
        let rc = Rc::new(RefCell::new(obj));
        self.enum_cache.insert(key, Rc::clone(&rc));
        Ok(Zval::Object(rc))
    }

    /// `E::cases()` (step 23, D-23.6): a sequential array of every case singleton
    /// in declaration order. Works on pure and backed enums alike.
    fn enum_cases(&mut self, cid: ClassId) -> Result<Zval, PhpError> {
        let names: Vec<Vec<u8>> = self.classes[cid]
            .enum_cases
            .iter()
            .map(|c| c.name.to_vec())
            .collect();
        let mut arr = PhpArray::new();
        for n in &names {
            let case = self.eval_enum_case(cid, n)?;
            let _ = arr.append(case);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `BackedEnum::from($v)` / `BackedEnum::tryFrom($v)` (step 23, D-23.6). Scans
    /// the cases for one whose backing `value` is identical (`===`) to `$v` and
    /// returns its singleton. `from` raises a catchable `ValueError` on no match;
    /// `tryFrom` returns `null`.
    fn enum_from(
        &mut self,
        cid: ClassId,
        arg: Option<&Zval>,
        try_from: bool,
    ) -> Result<Zval, PhpError> {
        let arg = arg.cloned().unwrap_or(Zval::Null);
        let names: Vec<Vec<u8>> = self.classes[cid]
            .enum_cases
            .iter()
            .map(|c| c.name.to_vec())
            .collect();
        for n in &names {
            let case = self.eval_enum_case(cid, n)?;
            let hit = matches!(&case, Zval::Object(o)
                if o.borrow().props.get(b"value").is_some_and(|v| ops::identical(v, &arg)));
            if hit {
                return Ok(case);
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
            String::from_utf8_lossy(&self.classes[cid].name)
        )))
    }

    /// The persistent cell backing a `static` property, resolving the declaring
    /// class up the chain and lazily initialising from the declared default on
    /// first access (step 19-4, D-19.14). Enforces visibility.
    fn static_prop_cell(
        &mut self,
        class: &ClassRef,
        name: &[u8],
    ) -> Result<Rc<RefCell<Zval>>, PhpError> {
        let cid = self.resolve_class_ref(class)?;
        let classes: &'p [ClassDecl] = self.classes;
        let mut c = Some(cid);
        let mut decl: Option<(ClassId, &'p crate::hir::StaticPropDecl)> = None;
        while let Some(x) = c {
            if let Some(p) = classes[x].static_props.iter().find(|p| p.name.as_ref() == name) {
                decl = Some((x, p));
                break;
            }
            c = classes[x].parent;
        }
        let (decl_class, pd) = decl.ok_or_else(|| {
            PhpError::Error(format!(
                "Access to undeclared static property {}::${}",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(name)
            ))
        })?;
        // Visibility against the current class context.
        if !self.visible_from(pd.visibility, decl_class) {
            let kind = if matches!(pd.visibility, Visibility::Private) {
                "private"
            } else {
                "protected"
            };
            return Err(PhpError::Error(format!(
                "Cannot access {kind} property {}::${}",
                String::from_utf8_lossy(&self.classes[decl_class].name),
                String::from_utf8_lossy(name)
            )));
        }
        let key = (decl_class, name.to_vec());
        if let Some(cell) = self.static_props.get(&key) {
            return Ok(Rc::clone(cell));
        }
        // First access: initialise from the default (evaluated in the declaring
        // class's context), then store.
        let init = match &pd.default {
            Some(e) => {
                let saved = self.cur_class.replace(decl_class);
                let v = self.eval(e);
                self.cur_class = saved;
                v?
            }
            None => Zval::Null,
        };
        let cell = Rc::new(RefCell::new(init));
        self.static_props.insert(key, Rc::clone(&cell));
        Ok(cell)
    }

    /// Resolve the argument of `exit`/`die` to a process exit code, following
    /// PHP's `exit(string|int $status = 0)` signature (step 46). A `string` (or
    /// a `__toString` object) takes the string branch: it is emitted as a
    /// message with exit code `0`. An `int`/`float`/`bool`/`null` takes the int
    /// branch: coerced to an integer exit code (normalised to `0..=255`, nothing
    /// printed). Anything else (`array`, a non-stringable object, …) is a
    /// `TypeError`, matching the oracle (`exit(): Argument #1 ($status) must be
    /// of type string|int, X given`). The float-precision / null deprecation
    /// notices PHP emits on coercion are a declared scope-out (D-46.1).
    fn exit_status(&mut self, v: Zval) -> Result<u8, PhpError> {
        // Collapse a reference to its referent (the invariant forbids ref-to-ref).
        let v = match v {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        };
        match &v {
            // A string is a message printed verbatim.
            Zval::Str(s) => {
                self.emit(s.as_bytes());
                Ok(0)
            }
            // Scalars with a defined integer coercion become the exit code.
            Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) | Zval::Null | Zval::Undef => {
                Ok(convert::to_long_cast(&v, &mut self.diags).rem_euclid(256) as u8)
            }
            // An object joins the `string` branch only if it is stringable.
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                if self.resolve_method(cid, b"__toString").is_some() {
                    let s = self.stringify(&v)?;
                    self.emit(s.as_bytes());
                    Ok(0)
                } else {
                    Err(self.exit_type_error(&v))
                }
            }
            // array / closure / generator: no string|int coercion → TypeError.
            _ => Err(self.exit_type_error(&v)),
        }
    }

    /// The `TypeError` for `exit`/`die` given a value outside `string|int`
    /// (step 46). Objects are named by their class (`stdClass given`), other
    /// values by their PHP type name.
    fn exit_type_error(&self, v: &Zval) -> PhpError {
        let given = match v {
            Zval::Object(o) => {
                String::from_utf8_lossy(&self.classes[o.borrow().class_id as usize].name)
                    .into_owned()
            }
            other => php_type_name(other).to_string(),
        };
        PhpError::TypeError(format!(
            "exit(): Argument #1 ($status) must be of type string|int, {given} given"
        ))
    }

    /// Convert a value to a string, honouring `__toString` on objects (step 19-6,
    /// D-19.18). A non-object goes through the ordinary `to_zstr` funnel; an
    /// object without `__toString` is the fatal PHP raises (the placeholder the
    /// infallible funnel emits is thereby avoided for the contexts that route
    /// through here: echo, concat, `(string)`).
    fn stringify(&mut self, v: &Zval) -> Result<Rc<PhpStr>, PhpError> {
        let v = v.deref_clone();
        match &v {
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                match self.resolve_method(cid, b"__toString") {
                    Some((defc, m)) => {
                        let r =
                            self.invoke_method(Some(v.clone()), defc, cid, m, b"__toString", vec![])?;
                        Ok(convert::to_zstr(&r, &mut self.diags))
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

    /// Decide whether a magic property accessor of `kind` (`__get`/`__set`/…)
    /// should run for `name` on object `o` instead of direct access (step 22,
    /// D-22.2/D-22.4). A magic call applies when the property is missing or not
    /// visible from the current scope, the class defines the accessor, and no
    /// same-kind guard is already active. Returns `(defining class, object class,
    /// object handle, method)` to invoke, or `None` for direct access.
    fn magic_prop_method(
        &self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
        kind: MagicAccess,
        magic_name: &[u8],
    ) -> Option<(ClassId, ClassId, u32, &'p MethodDecl)> {
        let (obj_cid, oid, present, accessible) = {
            let obj = o.borrow();
            let cid = obj.class_id as usize;
            let accessible = match self.resolve_prop_decl(cid, name) {
                Some((vis, dc)) => self.visible_from(vis, dc),
                None => true,
            };
            (cid, obj.id, obj.props.contains(name), accessible)
        };
        if present && accessible {
            return None;
        }
        if self.magic_guard.contains(&(oid, kind, name.to_vec())) {
            return None;
        }
        let (defc, m) = self.resolve_method(obj_cid, magic_name)?;
        Some((defc, obj_cid, oid, m))
    }

    /// If `__isset` applies to `name` on `o` (property missing/inaccessible,
    /// method present, not guarded), invoke it under a guard and return its
    /// boolean result; `None` means no magic (caller does the direct check),
    /// step 22, D-22.1.
    fn magic_isset_bool(
        &mut self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
    ) -> Option<Result<bool, PhpError>> {
        let (defc, obj_cid, oid, m) = self.magic_prop_method(o, name, MagicAccess::Isset, b"__isset")?;
        let key = (oid, MagicAccess::Isset, name.to_vec());
        self.magic_guard.insert(key.clone());
        let recv = Zval::Object(Rc::clone(o));
        let arg = Zval::Str(PhpStr::new(name.to_vec()));
        let r = self.invoke_method(Some(recv), defc, obj_cid, m, b"__isset", vec![arg]);
        self.magic_guard.remove(&key);
        Some(r.map(|v| convert::is_true_silent(&v)))
    }

    /// `isset()` truth for a resolved place (step 22): a single trailing property
    /// on an object routes to `__isset`; anything else uses the silent traversal.
    fn place_isset(&mut self, base: PlaceBase, steps: &[Step]) -> Result<bool, PhpError> {
        if let [Step::Prop(name)] = steps {
            if let Zval::Object(o) = self.base_clone(base) {
                if let Some(r) = self.magic_isset_bool(&o, name) {
                    return r;
                }
            }
        }
        Ok(matches!(
            self.silent_get(base, steps),
            Some(v) if !matches!(v, Zval::Null | Zval::Undef)
        ))
    }

    /// The value of `name` on `o` for a *silent* context — `empty()`, `??`,
    /// `??=` after `__isset` returned true (step 22): `__get` if defined, else
    /// the present value or NULL, raising no undefined-property warning
    /// (bug #44899).
    fn prop_value_silent(&mut self, o: &Rc<RefCell<Object>>, name: &[u8]) -> Result<Zval, PhpError> {
        if let Some((defc, obj_cid, oid, m)) =
            self.magic_prop_method(o, name, MagicAccess::Get, b"__get")
        {
            let key = (oid, MagicAccess::Get, name.to_vec());
            self.magic_guard.insert(key.clone());
            let recv = Zval::Object(Rc::clone(o));
            let arg = Zval::Str(PhpStr::new(name.to_vec()));
            let r = self.invoke_method(Some(recv), defc, obj_cid, m, b"__get", vec![arg]);
            self.magic_guard.remove(&key);
            return r;
        }
        Ok(o.borrow().props.get(name).map(Zval::deref_clone).unwrap_or(Zval::Null))
    }

    /// `empty()` truth for a resolved place (step 22): a magic property is
    /// `__isset` then (if set) `__get`, mirroring PHP.
    fn place_empty(&mut self, base: PlaceBase, steps: &[Step]) -> Result<bool, PhpError> {
        if let [Step::Prop(name)] = steps {
            if let Zval::Object(o) = self.base_clone(base) {
                if let Some(r) = self.magic_isset_bool(&o, name) {
                    if !r? {
                        return Ok(true);
                    }
                    let v = self.prop_value_silent(&o, name)?;
                    return Ok(!convert::is_true_silent(&v));
                }
            }
        }
        Ok(match self.silent_get(base, steps) {
            Some(v) => !convert::is_true_silent(&v),
            None => true,
        })
    }

    /// Read property `name` from a value (step 19, D-19.8; step 22 `__get`).
    /// Enforces visibility on a declared property; a missing or inaccessible
    /// property routes to `__get` if defined, else warns and yields NULL.
    fn read_property(&mut self, recv: &Zval, name: &[u8]) -> Result<Zval, PhpError> {
        match recv {
            Zval::Object(o) => {
                if let Some((defc, obj_cid, oid, m)) =
                    self.magic_prop_method(o, name, MagicAccess::Get, b"__get")
                {
                    let key = (oid, MagicAccess::Get, name.to_vec());
                    self.magic_guard.insert(key.clone());
                    let arg = Zval::Str(PhpStr::new(name.to_vec()));
                    let r = self.invoke_method(
                        Some(recv.clone()),
                        defc,
                        obj_cid,
                        m,
                        b"__get",
                        vec![arg],
                    );
                    self.magic_guard.remove(&key);
                    return r;
                }
                let cid = o.borrow().class_id as usize;
                self.check_prop_access(cid, name)?;
                let obj = o.borrow();
                if let Some(v) = obj.props.get(name) {
                    return Ok(v.deref_clone());
                }
                let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
                drop(obj);
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Undefined property: {cls}::${prop}"
                )));
                Ok(Zval::Null)
            }
            Zval::Null | Zval::Undef => {
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Attempt to read property \"{prop}\" on null"
                )));
                Ok(Zval::Null)
            }
            other => {
                let prop = String::from_utf8_lossy(name).into_owned();
                self.diags.push(Diag::Warning(format!(
                    "Attempt to read property \"{prop}\" on {}",
                    other.error_type_name()
                )));
                Ok(Zval::Null)
            }
        }
    }

    /// Pack call arguments into a 0-indexed list array, the second argument of
    /// `__call`/`__callStatic` (step 22, D-22.5).
    fn pack_args(&self, argv: Vec<Zval>) -> Zval {
        let mut arr = PhpArray::new();
        for v in argv {
            let _ = arr.append(v);
        }
        Zval::Array(Rc::new(arr))
    }

    /// Invoke `$obj->method(argv)` (step 19, D-19.7; step 22 `__call`): resolve
    /// the method up the chain, enforce visibility, then run it with `$this`
    /// bound to the receiver. A method missing or inaccessible from the current
    /// scope routes to `__call($method, $args)` if defined.
    fn call_method(
        &mut self,
        recv: Zval,
        method: &[u8],
        argv: Vec<Zval>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        let cid = match &recv {
            Zval::Object(o) => o.borrow().class_id as usize,
            other => {
                return Err(PhpError::Error(format!(
                    "Call to a member function {}() on {}",
                    String::from_utf8_lossy(method),
                    other.error_type_name()
                )))
            }
        };
        match self.resolve_method(cid, method) {
            Some((defc, m)) if self.visible_from(m.visibility, defc) => {
                // An instance call's LSB class is the object's actual class.
                // Named args (and named unpacking) are placed by name (step 38-3 / 40).
                let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
                let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
                self.invoke_method_args(Some(recv), defc, cid, m, method, argv)
            }
            found => {
                // `__call` collects args into an array; named-arg placement does
                // not apply to it (step 38 / 40 scope-out).
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                if let Some((cdefc, cm)) = self.resolve_method(cid, b"__call") {
                    let args = self.pack_args(argv);
                    let name = Zval::Str(PhpStr::new(method.to_vec()));
                    return self.invoke_method(Some(recv), cdefc, cid, cm, b"__call", vec![name, args]);
                }
                match found {
                    // Found but inaccessible and no __call: the visibility error.
                    Some((defc, m)) => {
                        self.check_method_access(defc, m, method)?;
                        unreachable!("check_method_access errors when not visible")
                    }
                    None => Err(PhpError::Error(format!(
                        "Call to undefined method {}::{}()",
                        String::from_utf8_lossy(&self.classes[cid].name),
                        String::from_utf8_lossy(method)
                    ))),
                }
            }
        }
    }

    /// Dispatch `Class::m()` / `self::m()` / `parent::m()` / `static::m()` (step
    /// 19-3/19-4). The starting class comes from the reference; `self`/`parent`/
    /// `static` are *forwarding* (keep `$this` and the caller's LSB class), while
    /// a named class sets the LSB class to itself.
    fn call_static(
        &mut self,
        class: &ClassRef,
        method: &[u8],
        argv: Vec<Zval>,
        spread_named: SpreadNamed,
        named: &[(Box<[u8]>, Expr)],
    ) -> Result<Zval, PhpError> {
        // `Closure::bind(...)` / `Closure::fromCallable(...)` are built-in (the
        // engine `Closure` class is not in the user class table), step 19-6.
        // Named args to these / enum built-in statics are out of scope (step 38 / 40).
        if let ClassRef::Named(n) = class {
            if n.eq_ignore_ascii_case(b"Closure") {
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                return self.closure_static(method, argv);
            }
        }
        let start = self.resolve_class_ref(class)?;
        // Enum built-in static methods (step 23, D-23.6). They are reserved names,
        // so they shadow user resolution. `cases` exists on every enum;
        // `from`/`tryFrom` only on backed ones (on a pure enum they fall through
        // to "undefined method").
        if self.classes[start].is_enum {
            if method.eq_ignore_ascii_case(b"cases") {
                return self.enum_cases(start);
            }
            if self.classes[start].enum_backing.is_some() {
                if method.eq_ignore_ascii_case(b"from") {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.enum_from(start, argv.first(), false);
                }
                if method.eq_ignore_ascii_case(b"tryFrom") {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.enum_from(start, argv.first(), true);
                }
            }
        }
        match self.resolve_method(start, method) {
            Some((defc, m)) if self.visible_from(m.visibility, defc) => {
                let forwarding = !matches!(class, ClassRef::Named(_));
                // LSB class: forwarding calls preserve the caller's, a named call
                // rebinds it to the named class.
                let static_class = if forwarding {
                    self.cur_static_class.unwrap_or(start)
                } else {
                    start
                };
                // `$this` is kept for a forwarding call, or for a named call to a
                // class in the current object's hierarchy (`ParentName::m()`).
                let this = match &self.cur_this {
                    Some(t @ Zval::Object(o))
                        if forwarding
                            || self.class_is_a(o.borrow().class_id as usize, start) =>
                    {
                        Some(t.clone())
                    }
                    _ => None,
                };
                // Named args (and named unpacking) placed by name (step 38-3 / 40).
                let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
                let argv = self.apply_named_args(&m.decl, argv, spread_named, named)?;
                self.invoke_method_args(this, defc, static_class, m, method, argv)
            }
            found => {
                if let Some(e) = self.reject_named(named, &spread_named) {
                    return Err(e);
                }
                // Missing or inaccessible. In object context (a usable `$this`,
                // e.g. `parent::priv()` from a method) it routes to `__call` on
                // `$this`; otherwise to `__callStatic` (step 22, D-22.3,
                // bug #53826).
                let forwarding = !matches!(class, ClassRef::Named(_));
                let this_obj = match &self.cur_this {
                    Some(t @ Zval::Object(o))
                        if forwarding
                            || self.class_is_a(o.borrow().class_id as usize, start) =>
                    {
                        Some(t.clone())
                    }
                    _ => None,
                };
                if let Some(this) = this_obj {
                    let ocid = match &this {
                        Zval::Object(o) => o.borrow().class_id as usize,
                        _ => unreachable!("matched Zval::Object above"),
                    };
                    if let Some((cdefc, cm)) = self.resolve_method(ocid, b"__call") {
                        let args = self.pack_args(argv);
                        let name = Zval::Str(PhpStr::new(method.to_vec()));
                        return self.invoke_method(Some(this), cdefc, ocid, cm, b"__call", vec![name, args]);
                    }
                }
                if let Some((cdefc, cm)) = self.resolve_method(start, b"__callStatic") {
                    let args = self.pack_args(argv);
                    let name = Zval::Str(PhpStr::new(method.to_vec()));
                    return self.invoke_method(None, cdefc, start, cm, b"__callStatic", vec![name, args]);
                }
                match found {
                    Some((defc, m)) => {
                        self.check_method_access(defc, m, method)?;
                        unreachable!("check_method_access errors when not visible")
                    }
                    None => Err(PhpError::Error(format!(
                        "Call to undefined method {}::{}()",
                        String::from_utf8_lossy(&self.classes[start].name),
                        String::from_utf8_lossy(method)
                    ))),
                }
            }
        }
    }

    /// Enforce method visibility against the current class context (step 19-3).
    fn check_method_access(
        &self,
        defining_class: ClassId,
        m: &MethodDecl,
        method: &[u8],
    ) -> Result<(), PhpError> {
        if self.visible_from(m.visibility, defining_class) {
            return Ok(());
        }
        let kind = if matches!(m.visibility, Visibility::Private) {
            "private"
        } else {
            "protected"
        };
        Err(PhpError::Error(format!(
            "Call to {kind} method {}::{}() from {}",
            String::from_utf8_lossy(&self.classes[defining_class].name),
            String::from_utf8_lossy(method),
            match self.cur_class {
                Some(c) => format!(
                    "scope {}",
                    String::from_utf8_lossy(&self.classes[c].name)
                ),
                None => "global scope".to_string(),
            }
        )))
    }

    /// Shared method-frame setup (step 19): bind `$this` and the defining class,
    /// check arity, bind parameters, run the body, then restore the saved context.
    fn invoke_method(
        &mut self,
        this: Option<Zval>,
        defining_class: ClassId,
        static_class: ClassId,
        m: &'p MethodDecl,
        method: &[u8],
        argv: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let argv: Vec<Arg> = argv.into_iter().map(Arg::Val).collect();
        self.invoke_method_args(this, defining_class, static_class, m, method, argv)
    }

    /// Like [`Self::invoke_method`] but taking already-bound [`Arg`]s, so a call
    /// site can supply named-argument placement (step 38, `Arg::Default` gaps).
    fn invoke_method_args(
        &mut self,
        this: Option<Zval>,
        defining_class: ClassId,
        static_class: ClassId,
        m: &'p MethodDecl,
        method: &[u8],
        argv: Vec<Arg>,
    ) -> Result<Zval, PhpError> {
        self.guard_call_depth()?;
        let f: &'p FnDecl = &m.decl;
        let required = f
            .params
            .iter()
            .filter(|p| p.default.is_none() && !p.variadic)
            .count();
        // A required parameter must have a real argument at its index (named args
        // can leave `Arg::Default` gaps — step 38).
        let missing_required = f
            .params
            .iter()
            .enumerate()
            .any(|(i, p)| {
                p.default.is_none()
                    && !p.variadic
                    && !matches!(argv.get(i), Some(Arg::Val(_) | Arg::Ref(_)))
            });
        if missing_required {
            let passed = argv
                .iter()
                .filter(|a| matches!(a, Arg::Val(_) | Arg::Ref(_)))
                .count();
            let expected = if required == f.params.len() {
                format!("exactly {required}")
            } else {
                format!("at least {required}")
            };
            return Err(PhpError::Error(format!(
                "Too few arguments to function {}::{}(), {} passed and {} expected",
                String::from_utf8_lossy(&self.classes[defining_class].name),
                String::from_utf8_lossy(method),
                passed,
                expected,
            )));
        }

        // Record a method stack frame for the body (step 28): `Class->m` for an
        // instance call, `Class::m` for a static one. Push before `this` moves.
        self.call_stack.push(CallFrame {
            class: Some(self.classes[static_class].name.to_vec()),
            function: method.to_vec(),
            is_static: this.is_none(),
            line: self.cur_line as i64,
        });

        let frame = fresh_slots(f.slots.len());
        let saved_locals = self.locals.replace(frame);
        let saved_names = self.local_names.replace(f.slots.as_slice());
        let saved_returns_ref = std::mem::replace(&mut self.fn_returns_ref, f.by_ref);
        let saved_this = std::mem::replace(&mut self.cur_this, this);
        let saved_class = self.cur_class.replace(defining_class);
        let saved_static = self.cur_static_class.replace(static_class);

        let result = self.run_user_fn_body(f, argv);

        self.locals = saved_locals;
        self.local_names = saved_names;
        self.fn_returns_ref = saved_returns_ref;
        self.cur_this = saved_this;
        self.cur_class = saved_class;
        self.cur_static_class = saved_static;
        self.call_stack.pop();
        result.map(|r| match r {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        })
    }

    /// Dispatch a higher-order builtin that the evaluator implements directly
    /// (step 18, D-18.6). Returns `None` for a name we do not intercept, so the
    /// caller falls through to the ordinary registry lookup.
    fn dispatch_higher_order(
        &mut self,
        name: &[u8],
        args: &[Expr],
    ) -> Option<Result<Zval, PhpError>> {
        match name {
            b"is_callable" => Some(self.ho_is_callable(args)),
            b"call_user_func" => Some(self.ho_call_user_func(args)),
            b"call_user_func_array" => Some(self.ho_call_user_func_array(args)),
            b"array_map" => Some(self.ho_array_map(args)),
            b"array_filter" => Some(self.ho_array_filter(args)),
            b"array_walk" => Some(self.ho_array_walk(args)),
            b"usort" => Some(self.ho_usort(args)),
            b"json_decode" => Some(self.ho_json_decode(args)),
            b"preg_match" => Some(self.ho_preg_match(args)),
            b"preg_match_all" => Some(self.ho_preg_match_all(args)),
            b"preg_replace" => Some(self.ho_preg_replace(args)),
            b"preg_replace_callback" => Some(self.ho_preg_replace_callback(args)),
            b"preg_split" => Some(self.ho_preg_split(args)),
            b"preg_quote" => Some(self.ho_preg_quote(args)),
            b"mb_ereg" => Some(self.ho_mb_ereg(args, false)),
            b"mb_eregi" => Some(self.ho_mb_ereg(args, true)),
            b"mb_ereg_replace" => Some(self.ho_mb_ereg_replace(args, false)),
            b"mb_eregi_replace" => Some(self.ho_mb_ereg_replace(args, true)),
            b"mb_ereg_replace_callback" => Some(self.ho_mb_ereg_replace_callback(args)),
            b"mb_split" => Some(self.ho_mb_split(args)),
            b"mb_ereg_match" => Some(self.ho_mb_ereg_match(args)),
            b"mb_regex_encoding" => Some(self.ho_mb_regex_encoding(args)),
            b"mb_regex_set_options" => Some(self.ho_mb_regex_set_options(args)),
            b"mb_ereg_search_init" => Some(self.ho_mb_ereg_search_init(args)),
            b"mb_ereg_search" => Some(self.ho_mb_ereg_search(args)),
            b"mb_ereg_search_pos" => Some(self.ho_mb_ereg_search_pos(args)),
            b"mb_ereg_search_regs" => Some(self.ho_mb_ereg_search_regs(args)),
            b"mb_ereg_search_getregs" => Some(self.ho_mb_ereg_search_getregs()),
            b"mb_ereg_search_getpos" => Some(Ok(Zval::Long(self.mb_regex.search_pos as i64))),
            b"mb_ereg_search_setpos" => Some(self.ho_mb_ereg_search_setpos(args)),
            _ => None,
        }
    }

    /// `array_walk(&$array, $callback, $arg = null)` (step 32): apply `$callback`
    /// to each element. When the callback's first parameter is by-reference the
    /// element is passed through a shared cell and the mutation is written back;
    /// otherwise it is passed by value (read-only). Returns true. The keys are
    /// never modified.
    fn ho_array_walk(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let (Some(arr_expr), Some(cb_expr)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(format!(
                "array_walk() expects at least 2 arguments, {} given",
                args.len()
            )));
        };
        let ExprKind::Var(slot) = arr_expr.kind else {
            return Err(PhpError::Error(
                "array_walk(): Argument #1 ($array) could not be passed by reference".to_string(),
            ));
        };
        let callback = self.eval(cb_expr)?.deref_clone();
        let extra = match args.get(2) {
            Some(e) => Some(self.eval(e)?.deref_clone()),
            None => None,
        };
        let by_ref = self.callable_first_by_ref(&callback);
        let cell = self.slot_cell(slot as usize);
        let entries: Vec<(Key, Zval)> = match &*cell.borrow() {
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
            let key_z = match &k {
                Key::Int(i) => Zval::Long(*i),
                Key::Str(s) => Zval::Str(Rc::clone(s)),
            };
            let new_v = if by_ref {
                let vcell = Rc::new(RefCell::new(v));
                let mut argv = vec![Zval::Ref(Rc::clone(&vcell)), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_value(callback.clone(), argv)?;
                let updated = vcell.borrow().clone();
                updated
            } else {
                let mut argv = vec![v.clone(), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_value(callback.clone(), argv)?;
                v
            };
            out.insert(k, new_v);
        }
        *cell.borrow_mut() = Zval::Array(Rc::new(out));
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
                    .closures
                    .get(cl.fn_idx)
                    .and_then(|f| f.params.first())
                    .is_some_and(|p| p.by_ref),
            },
            Zval::Str(s) => self.named_first_by_ref(s.as_bytes()),
            Zval::Ref(c) => {
                let inner = c.borrow().clone();
                self.callable_first_by_ref(&inner)
            }
            _ => false,
        }
    }

    /// First-parameter by-reference flag of a named user function.
    fn named_first_by_ref(&self, name: &[u8]) -> bool {
        self.fn_index
            .get(&name.to_ascii_lowercase())
            .and_then(|&i| self.funcs.get(i))
            .and_then(|f| f.params.first())
            .is_some_and(|p| p.by_ref)
    }

    /// Evaluate an argument and coerce it to a byte string (used by `preg_*` for
    /// the pattern and subject).
    fn preg_str(&mut self, e: &Expr) -> Result<Vec<u8>, PhpError> {
        let v = self.eval(e)?.deref_clone();
        Ok(self.stringify(&v)?.as_bytes().to_vec())
    }

    /// Write `value` to a plain-variable out-parameter (e.g. the `$matches` of
    /// `preg_match`). Only bare variables are supported as out-params; any other
    /// expression is silently ignored (step 27 scope-out).
    fn write_out_param(&mut self, target: &Expr, value: Zval) {
        if let ExprKind::Var(slot) = &target.kind {
            self.slot_set(*slot as usize, value);
        }
    }

    /// Evaluate an optional `preg_*` flags argument to an int (0 when absent).
    fn preg_flags(&mut self, arg: Option<&Expr>) -> Result<i64, PhpError> {
        match arg {
            Some(e) => Ok(convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags)),
            None => Ok(0),
        }
    }

    /// `preg_match($pattern, $subject, &$matches = null)` (step 27): returns 1 on
    /// a match, 0 on none, `false` on a bad pattern. `$matches[0]` is the whole
    /// match, `$matches[n]` the n-th group (numeric groups only).
    fn ho_preg_match(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let flags = self.preg_flags(args.get(3))?;
        let subj = String::from_utf8_lossy(&subject);
        let (ret, matches) = match re.captures(&subj) {
            Some(caps) => (1, captures_array(&re, &caps, flags)),
            None => (0, Zval::Array(Rc::new(PhpArray::new()))),
        };
        if let Some(out) = args.get(2) {
            self.write_out_param(out, matches);
        }
        Ok(Zval::Long(ret))
    }

    /// `preg_match_all($pattern, $subject, &$matches = null)` (step 27): default
    /// PREG_PATTERN_ORDER — `$matches[g]` is the array of group `g`'s text across
    /// all matches. Returns the match count, or `false` on a bad pattern.
    fn ho_preg_match_all(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match_all() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let flags = self.preg_flags(args.get(3))?;
        let subj = String::from_utf8_lossy(&subject);
        let offset = flags & PREG_OFFSET_CAPTURE != 0;
        let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
        let mut count: i64 = 0;

        let outer = if flags & PREG_SET_ORDER != 0 {
            // One entry per match, each a full $matches array.
            let mut outer = PhpArray::new();
            for caps in re.captures_iter(&subj) {
                count += 1;
                let _ = outer.append(captures_array(&re, &caps, flags));
            }
            outer
        } else {
            // PREG_PATTERN_ORDER: one column per group (with named keys), each
            // the array of that group's value across all matches.
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
        if let Some(out) = args.get(2) {
            self.write_out_param(out, Zval::Array(Rc::new(outer)));
        }
        Ok(Zval::Long(count))
    }

    /// `preg_replace($pattern, $replacement, $subject)` (step 27): backreferences
    /// `$1` / `${1}` / `\1` in the replacement are honoured. Returns `null` on a
    /// bad pattern. Array patterns/subjects are a scope-out.
    fn ho_preg_replace(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let repl = self.preg_str(&args[1])?;
        let subject = self.preg_str(&args[2])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let repl = String::from_utf8_lossy(&crate::preg::translate_replacement(&repl)).into_owned();
        let subj = String::from_utf8_lossy(&subject);
        let result = re.replace_all(&subj, repl.as_str());
        Ok(Zval::Str(PhpStr::new(result.as_bytes().to_vec())))
    }

    /// `preg_replace_callback($pattern, $callback, $subject)` (step 27): the
    /// callback receives the match array and returns each replacement.
    fn ho_preg_replace_callback(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace_callback() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let callback = self.eval(&args[1])?.deref_clone();
        let subject = self.preg_str(&args[2])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let bytes = subj.as_bytes();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        // Collect (range, match-array) first so the regex borrow of `subj` ends
        // before we call back into the evaluator.
        let hits: Vec<(usize, usize, Zval)> = re
            .captures_iter(&subj)
            .into_iter()
            .map(|caps| {
                let m0 = caps.get(0).unwrap();
                (m0.start, m0.end, captures_array(&re, &caps, 0))
            })
            .collect();
        for (start, end, match_arr) in hits {
            out.extend_from_slice(&bytes[last..start]);
            let ret = self.call_value(callback.clone(), vec![match_arr])?;
            let rs = self.stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&bytes[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `preg_split($pattern, $subject)` (step 27): split on matches. Returns
    /// `false` on a bad pattern.
    fn ho_preg_split(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_split() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let limit = match args.get(2) {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
            None => -1,
        };
        let flags = self.preg_flags(args.get(3))?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let no_empty = flags & 1 != 0;
        let delim_capture = flags & 2 != 0;
        let offset_capture = flags & 4 != 0;
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let mut arr = PhpArray::new();
        let mut last = 0usize;
        // A positive limit caps the piece count; the last piece keeps the rest.
        let push = |arr: &mut PhpArray, text: &str, off: usize| {
            if no_empty && text.is_empty() {
                return;
            }
            if offset_capture {
                let _ = arr.append(offset_pair(
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

    /// `preg_quote($str, $delimiter = null)` (step 27).
    fn ho_preg_quote(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "preg_quote() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let s = self.preg_str(first)?;
        let delim = match args.get(1) {
            Some(e) => self.preg_str(e)?.first().copied(),
            None => None,
        };
        Ok(Zval::Str(PhpStr::new(crate::preg::quote(&s, delim))))
    }

    // --- mbstring regex family (step 43), backed by oniguruma via `mbregex`. ---

    /// Compile a pattern under `opts` against the current mbregex dialect,
    /// emitting PHP's `mbregex compile err:` warning and returning `None` on a
    /// compile error.
    fn mb_compile(&mut self, pat: &[u8], opts: &[u8], func: &str, ic: bool) -> Option<onig::Regex> {
        match crate::mbregex::compile(pat, opts, ic) {
            Ok(re) => Some(re),
            Err(msg) => {
                self.diags
                    .push(Diag::Warning(format!("{func}(): mbregex compile err: {msg}")));
                None
            }
        }
    }

    /// Resolve an optional `$options` argument (index `idx`) to an option string:
    /// the argument when present and non-null, else the global mbregex options.
    fn mb_opts_arg(&mut self, args: &[Expr], idx: usize) -> Result<Vec<u8>, PhpError> {
        match args.get(idx) {
            None => Ok(self.mb_regex.options.clone()),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if matches!(v, Zval::Null) {
                    Ok(self.mb_regex.options.clone())
                } else {
                    Ok(self.stringify(&v)?.as_bytes().to_vec())
                }
            }
        }
    }

    /// `mb_ereg($pattern, $string, &$regs = null)` / `mb_eregi` (case-insensitive):
    /// returns a bool (PHP 8). `$regs[0]` is the whole match, `$regs[n]` the n-th
    /// group (a non-participating group is `false`), with named groups appended
    /// by string key. On no match `$regs` is set to an empty array.
    fn ho_mb_ereg(&mut self, args: &[Expr], ic: bool) -> Result<Zval, PhpError> {
        let func = if ic { "mb_eregi" } else { "mb_ereg" };
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let opts = self.mb_regex.options.clone();
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            return Ok(Zval::Bool(false));
        };
        let regs = crate::mbregex::exec(&re, &subject);
        let matched = regs.is_some();
        if let Some(out) = args.get(2) {
            self.write_out_param(out, regs.unwrap_or_else(|| Zval::Array(Rc::new(PhpArray::new()))));
        }
        Ok(Zval::Bool(matched))
    }

    /// `mb_ereg_replace($pattern, $replacement, $string[, $options])` / the `i`
    /// variant. Backreferences `\0`..`\9` in the replacement are honoured.
    /// Returns `false` on a bad pattern.
    fn ho_mb_ereg_replace(&mut self, args: &[Expr], ic: bool) -> Result<Zval, PhpError> {
        let func = if ic { "mb_eregi_replace" } else { "mb_ereg_replace" };
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let repl = self.preg_str(&args[1])?;
        let subject = self.preg_str(&args[2])?;
        let opts = self.mb_opts_arg(args, 3)?;
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Str(PhpStr::new(crate::mbregex::replace(&re, &repl, &subject))))
    }

    /// `mb_ereg_replace_callback($pattern, $callback, $string[, $options])`: the
    /// callback receives each match's `$regs` array and returns its replacement.
    fn ho_mb_ereg_replace_callback(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_replace_callback() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let callback = self.eval(&args[1])?.deref_clone();
        let subject = self.preg_str(&args[2])?;
        let opts = self.mb_opts_arg(args, 3)?;
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_replace_callback", false) else {
            return Ok(Zval::Bool(false));
        };
        let bytes = subject.clone();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        for (start, end, regs) in crate::mbregex::find_all(&re, &subject) {
            out.extend_from_slice(&bytes[last..start]);
            let ret = self.call_value(callback.clone(), vec![regs])?;
            let rs = self.stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&bytes[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `mb_split($pattern, $string[, $limit])`: split on matches, keeping empty
    /// fields. `$limit > 0` caps the piece count. Returns `false` on a bad pattern.
    fn ho_mb_split(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_split() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let limit = match args.get(2) {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
            None => -1,
        };
        let opts = self.mb_regex.options.clone();
        let Some(re) = self.mb_compile(&pat, &opts, "mb_split", false) else {
            return Ok(Zval::Bool(false));
        };
        let mut arr = PhpArray::new();
        for p in crate::mbregex::split(&re, &subject, limit) {
            let _ = arr.append(Zval::Str(PhpStr::new(p)));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `mb_ereg_match($pattern, $string[, $options])`: whether the pattern matches
    /// anchored at the start of `$string` (a prefix match, not a full match).
    fn ho_mb_ereg_match(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_match() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let opts = self.mb_opts_arg(args, 2)?;
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_match", false) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Bool(crate::mbregex::matches_at_start(&re, &subject)))
    }

    /// `mb_regex_encoding([$encoding])`: getter returns the current name ("UTF-8"
    /// default); setter stores it and returns true. Only UTF-8 is effectively
    /// supported (D-MB-ereg-enc).
    fn ho_mb_regex_encoding(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone()))),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if matches!(v, Zval::Null) {
                    return Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone())));
                }
                self.mb_regex.encoding = self.stringify(&v)?.as_bytes().to_vec();
                Ok(Zval::Bool(true))
            }
        }
    }

    /// `mb_regex_set_options([$options])`: getter returns the current options
    /// ("pr" default); setter stores them and returns the previous options.
    fn ho_mb_regex_set_options(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let prev = self.mb_regex.options.clone();
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(prev))),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if !matches!(v, Zval::Null) {
                    self.mb_regex.options = self.stringify(&v)?.as_bytes().to_vec();
                }
                Ok(Zval::Str(PhpStr::new(prev)))
            }
        }
    }

    // --- mbregex stateful search cursor (step 43b) ---

    /// Compile and store the search pattern from an optional `$pattern` argument
    /// at `idx` (its `$options` follow at `idx + 1`); keeps the existing compiled
    /// pattern when absent/null. Returns false on a compile error.
    fn mb_search_set_pattern(&mut self, args: &[Expr], idx: usize) -> Result<bool, PhpError> {
        if let Some(p) = args.get(idx) {
            let pv = self.eval(p)?.deref_clone();
            if !matches!(pv, Zval::Null) {
                let pat = self.stringify(&pv)?.as_bytes().to_vec();
                let opts = self.mb_opts_arg(args, idx + 1)?;
                match self.mb_compile(&pat, &opts, "mb_ereg_search", false) {
                    Some(re) => self.mb_regex.search_re = Some(re),
                    None => return Ok(false),
                }
            }
        }
        Ok(true)
    }

    /// Run the next search from the cursor, advancing it past the match (by one
    /// byte for a zero-width match) and recording the result for `getregs`.
    fn mb_search_step(&mut self) -> Option<(usize, usize, Zval)> {
        let re = self.mb_regex.search_re.take()?;
        let subject = std::mem::take(&mut self.mb_regex.search_str);
        let res = crate::mbregex::search_from(&re, &subject, self.mb_regex.search_pos);
        self.mb_regex.search_re = Some(re);
        self.mb_regex.search_str = subject;
        if let Some((start, end, regs)) = &res {
            self.mb_regex.search_pos = if end > start { *end } else { *end + 1 };
            self.mb_regex.last_regs = Some(regs.clone());
        }
        res
    }

    /// `mb_ereg_search_init($string[, $pattern[, $options]])`: start a stateful
    /// search over `$string`, resetting the cursor. Returns false on a bad pattern.
    fn ho_mb_ereg_search_init(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "mb_ereg_search_init() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        self.mb_regex.search_str = self.preg_str(first)?;
        self.mb_regex.search_pos = 0;
        self.mb_regex.last_regs = None;
        if !self.mb_search_set_pattern(args, 1)? {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }

    /// `mb_ereg_search([$pattern[, $options]])`: advance the cursor to the next
    /// match; returns whether one was found.
    fn ho_mb_ereg_search(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(self.mb_search_step().is_some()))
    }

    /// `mb_ereg_search_pos([$pattern[, $options]])`: next match as `[pos, len]`
    /// byte offsets, or false at the end.
    fn ho_mb_ereg_search_pos(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
            return Ok(Zval::Bool(false));
        }
        match self.mb_search_step() {
            Some((start, end, _)) => {
                let mut arr = PhpArray::new();
                let _ = arr.append(Zval::Long(start as i64));
                let _ = arr.append(Zval::Long((end - start) as i64));
                Ok(Zval::Array(Rc::new(arr)))
            }
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `mb_ereg_search_regs([$pattern[, $options]])`: next match's `$regs` array,
    /// or false at the end.
    fn ho_mb_ereg_search_regs(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
            return Ok(Zval::Bool(false));
        }
        match self.mb_search_step() {
            Some((_, _, regs)) => Ok(regs),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `mb_ereg_search_getregs()`: the `$regs` of the last successful search, or
    /// false if none has succeeded.
    fn ho_mb_ereg_search_getregs(&mut self) -> Result<Zval, PhpError> {
        Ok(self.mb_regex.last_regs.clone().unwrap_or(Zval::Bool(false)))
    }

    /// `mb_ereg_search_setpos($position)`: move the byte cursor; false if out of
    /// range.
    fn ho_mb_ereg_search_setpos(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let pos = match args.first() {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
            None => 0,
        };
        if pos < 0 || pos as usize > self.mb_regex.search_str.len() {
            return Ok(Zval::Bool(false));
        }
        self.mb_regex.search_pos = pos as usize;
        Ok(Zval::Bool(true))
    }

    /// `json_decode($json, $assoc = false, ...)` (step 26). Intercepted here
    /// because the default mode builds `stdClass` objects, which needs the class
    /// table. Returns `null` on a parse error (JSON_THROW_ON_ERROR is a
    /// scope-out). Objects become arrays when `$assoc` is true, `stdClass`
    /// otherwise; the `depth`/`flags` arguments are ignored.
    fn ho_json_decode(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "json_decode() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arg0 = self.eval(first)?.deref_clone();
        let json = self.stringify(&arg0)?;
        let assoc = match args.get(1) {
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                convert::to_bool(&v, &mut self.diags)
            }
            None => false,
        };
        match crate::json::parse(json.as_bytes()) {
            Some(j) => Ok(self.json_to_zval(&j, assoc)),
            None => Ok(Zval::Null),
        }
    }

    /// Convert a parsed [`crate::json::Json`] tree into a `Zval` (step 26).
    fn json_to_zval(&mut self, j: &crate::json::Json, assoc: bool) -> Zval {
        use crate::json::Json;
        match j {
            Json::Null => Zval::Null,
            Json::Bool(b) => Zval::Bool(*b),
            Json::Long(n) => Zval::Long(*n),
            Json::Double(d) => Zval::Double(*d),
            Json::Str(s) => Zval::Str(PhpStr::new(s.clone())),
            Json::Array(items) => {
                let mut arr = PhpArray::new();
                for item in items {
                    let v = self.json_to_zval(item, assoc);
                    let _ = arr.append(v);
                }
                Zval::Array(Rc::new(arr))
            }
            Json::Object(entries) => {
                let fields: Vec<(Vec<u8>, Zval)> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.json_to_zval(v, assoc)))
                    .collect();
                if assoc {
                    let mut arr = PhpArray::new();
                    for (k, v) in fields {
                        arr.insert(Key::from_bytes(&k), v);
                    }
                    Zval::Array(Rc::new(arr))
                } else {
                    self.make_stdclass(fields)
                }
            }
        }
    }

    /// Build a fresh `stdClass` with the given dynamic properties (step 26).
    /// `(object)$v`: arrays map each entry to a property (key stringified),
    /// objects pass through unchanged, null yields an empty stdClass, and any
    /// scalar becomes a single `scalar` property.
    fn object_cast(&mut self, v: Zval) -> Zval {
        match v {
            Zval::Object(_) => v,
            Zval::Array(a) => {
                let fields = a
                    .iter()
                    .map(|(k, val)| {
                        let name = match k {
                            Key::Int(i) => i.to_string().into_bytes(),
                            Key::Str(s) => s.as_bytes().to_vec(),
                        };
                        (name, val.clone())
                    })
                    .collect();
                self.make_stdclass(fields)
            }
            Zval::Null => self.make_stdclass(Vec::new()),
            scalar => self.make_stdclass(vec![(b"scalar".to_vec(), scalar)]),
        }
    }

    fn make_stdclass(&mut self, fields: Vec<(Vec<u8>, Zval)>) -> Zval {
        let cid = self.class_index[b"stdclass".as_slice()];
        let class_name = PhpStr::new(self.classes[cid].name.to_vec());
        let mut props = Props::new();
        for (k, v) in fields {
            props.set(&k, v);
        }
        let info = self.class_shape(cid);
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info };
        let value = Zval::Object(Rc::new(RefCell::new(obj)));
        if let Zval::Object(o) = &value {
            self.created.push(o.clone());
        }
        value
    }

    /// Class-introspection builtins the evaluator answers directly because they
    /// read the current object / class table rather than a pure value (step 20
    /// coda). Returns `None` for a name we do not intercept.
    fn dispatch_class_introspection(
        &mut self,
        name: &[u8],
        args: &[Expr],
    ) -> Option<Result<Zval, PhpError>> {
        match name {
            b"get_class" => Some(self.ci_get_class(args)),
            b"get_parent_class" => Some(self.ci_get_parent_class(args)),
            b"get_class_methods" => Some(self.ci_get_class_methods(args)),
            b"get_object_vars" => Some(self.ci_get_object_vars(args)),
            _ => None,
        }
    }

    /// `get_class($object)` — the object's class name; with no argument, the
    /// class of the current `$this` (a fatal Error outside object context, PHP 8).
    fn ci_get_class(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let v = match args.first() {
            Some(e) => self.eval(e)?.deref_clone(),
            None => match &self.cur_this {
                Some(t) => t.clone(),
                None => {
                    return Err(PhpError::Error(
                        "get_class() without arguments must be called from within a class"
                            .to_string(),
                    ))
                }
            },
        };
        match &v {
            Zval::Object(o) => Ok(Zval::Str(PhpStr::new(o.borrow().class_name.as_bytes().to_vec()))),
            Zval::Closure(_) => Ok(Zval::Str(PhpStr::new(b"Closure".to_vec()))),
            other => Err(PhpError::TypeError(format!(
                "get_class(): Argument #1 ($object) must be of type object, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// `get_parent_class([$object|$class])` — the parent class name, or `false`
    /// when there is none (or the target cannot be resolved). With no argument it
    /// uses the current class context.
    fn ci_get_parent_class(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let cid: Option<ClassId> = match args.first() {
            Some(e) => match self.eval(e)?.deref_clone() {
                Zval::Object(o) => Some(o.borrow().class_id as usize),
                Zval::Str(s) => self
                    .class_index
                    .get(&s.as_bytes().to_ascii_lowercase())
                    .copied(),
                _ => None,
            },
            None => self.cur_class,
        };
        match cid.and_then(|c| self.classes[c].parent) {
            Some(p) => Ok(Zval::Str(PhpStr::new(self.classes[p].name.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `get_class_methods($objectOrClassName)` (step 47): the method names of the
    /// class, walking the inheritance chain child→parent (each method once;
    /// child overrides win), filtered by visibility from the calling scope
    /// (`visible_from`) — so from outside only `public` methods are returned, and
    /// from within the class the `protected`/`private` ones too.
    fn ci_get_class_methods(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let cid: Option<ClassId> = match args.first() {
            Some(e) => match self.eval(e)?.deref_clone() {
                Zval::Object(o) => Some(o.borrow().class_id as usize),
                Zval::Str(s) => self
                    .class_index
                    .get(&s.as_bytes().to_ascii_lowercase())
                    .copied(),
                _ => None,
            },
            None => {
                return Err(PhpError::ArgumentCountError(
                    "get_class_methods() expects exactly 1 argument, 0 given".to_string(),
                ))
            }
        };
        // An unresolved class name yields `null` (PHP raises a TypeError only for
        // a non-string/non-object argument, which we mapped to `None` above).
        let Some(start) = cid else {
            return Ok(Zval::Null);
        };
        let classes: &'p [ClassDecl] = self.classes;
        let mut arr = PhpArray::new();
        let mut seen: Vec<Vec<u8>> = Vec::new();
        let mut cur = Some(start);
        while let Some(c) = cur {
            for m in &classes[c].methods {
                let lname = m.decl.name.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue; // a more-derived class already defined this name
                }
                // Mark the name as resolved by this (most-derived) class even
                // when it is not visible, so a parent's same-named method (or an
                // overridden abstract signature) does not leak into the result.
                seen.push(lname);
                if self.visible_from(m.visibility, c) {
                    let _ = arr.append(Zval::Str(PhpStr::new(m.decl.name.to_vec())));
                }
            }
            // Abstract signatures (interface / `abstract` methods). Interface
            // methods are public; a protected `abstract` method that is never
            // overridden and queried from outside is a minor scope-out (D-47.1).
            for am in &classes[c].abstract_methods {
                let lname = am.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue;
                }
                seen.push(lname);
                let _ = arr.append(Zval::Str(PhpStr::new(am.to_vec())));
            }
            cur = classes[c].parent;
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `get_object_vars($object)` (step 47): the object's properties as a
    /// `name => value` array, filtered by visibility from the calling scope —
    /// from outside only `public` properties, from within the class the
    /// `protected`/`private` ones too. Dynamic (undeclared) properties are
    /// always public. Declaration / insertion order is preserved.
    fn ci_get_object_vars(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let v = match args.first() {
            Some(e) => self.eval(e)?.deref_clone(),
            None => {
                return Err(PhpError::ArgumentCountError(
                    "get_object_vars() expects exactly 1 argument, 0 given".to_string(),
                ))
            }
        };
        let Zval::Object(o) = v else {
            return Err(PhpError::TypeError(format!(
                "get_object_vars(): Argument #1 ($object) must be of type object, {} given",
                v.error_type_name()
            )));
        };
        let obj = o.borrow();
        let cid = obj.class_id as usize;
        let mut arr = PhpArray::new();
        for (name, val) in obj.props.iter() {
            let visible = match self.resolve_prop_decl(cid, name) {
                Some((vis, decl_class)) => self.visible_from(vis, decl_class),
                None => true, // dynamic / undeclared property is public
            };
            if visible {
                arr.insert(Key::from_bytes(name), val.clone());
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// Whether a function *name* resolves to something callable: a user function,
    /// a registered builtin, or a higher-order builtin the evaluator intercepts.
    fn is_name_callable(&self, name: &[u8]) -> bool {
        self.fn_index.contains_key(&name.to_ascii_lowercase())
            || self.reg.contains_key(name)
            || HIGHER_ORDER_BUILTINS.contains(&name)
    }

    /// `is_callable($value)` (step 18). Closures are callable; a string is
    /// callable iff it names a function; everything else (arrays/OOP callables
    /// are a scope-out) is not.
    fn ho_is_callable(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "is_callable() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let v = self.eval(first)?.deref_clone();
        let callable = match &v {
            Zval::Closure(_) => true,
            Zval::Str(s) => self.is_name_callable(s.as_bytes()),
            // An object is callable iff it defines `__invoke` (step 22, D-22.7).
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                self.resolve_method(cid, b"__invoke").is_some()
            }
            _ => false,
        };
        Ok(Zval::Bool(callable))
    }

    /// `call_user_func($callable, ...$args)` (step 18): the remaining arguments
    /// are forwarded by value to the callable.
    fn ho_call_user_func(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some((callee_expr, rest)) = args.split_first() else {
            return Err(PhpError::ArgumentCountError(
                "call_user_func() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let callee = self.eval(callee_expr)?.deref_clone();
        let mut argv = Vec::with_capacity(rest.len());
        for a in rest {
            argv.push(self.eval(a)?);
        }
        self.call_value(callee, argv)
    }

    /// `call_user_func_array($callable, $args)` (step 18): the second argument is
    /// an array whose *values* become the positional arguments (string-keyed
    /// named arguments are a scope-out).
    fn ho_call_user_func_array(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "call_user_func_array() expects exactly 2 arguments, {} given",
                args.len()
            )));
        }
        let callee = self.eval(&args[0])?.deref_clone();
        let arr = self.eval(&args[1])?.deref_clone();
        let argv: Vec<Zval> = match arr {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "call_user_func_array(): Argument #2 ($args) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        self.call_value(callee, argv)
    }

    /// `array_map($callback, ...$arrays)` (step 18, D-18.6). With a single array
    /// the keys are preserved; with several arrays the result is re-indexed and
    /// the callback receives one element from each (missing tails are NULL). A
    /// NULL callback zips the arrays (single array → identity).
    fn ho_array_map(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_map() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let cb = self.eval(&args[0])?.deref_clone();
        let null_cb = matches!(cb, Zval::Null);
        let mut arrays = Vec::with_capacity(args.len() - 1);
        for (i, a) in args[1..].iter().enumerate() {
            match self.eval(a)?.deref_clone() {
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
            // Single array: preserve keys.
            let entries: Vec<(Key, Zval)> =
                arrays[0].iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
            for (k, v) in entries {
                let mapped = if null_cb {
                    v
                } else {
                    self.call_value(cb.clone(), vec![v])?
                };
                out.insert(k, mapped);
            }
        } else {
            // Several arrays: re-index 0..max, one element from each per row.
            let cols: Vec<Vec<Zval>> = arrays
                .iter()
                .map(|a| a.iter().map(|(_, v)| v.deref_clone()).collect())
                .collect();
            let max = cols.iter().map(|c| c.len()).max().unwrap_or(0);
            for i in 0..max {
                let row: Vec<Zval> = cols
                    .iter()
                    .map(|c| c.get(i).cloned().unwrap_or(Zval::Null))
                    .collect();
                let val = if null_cb {
                    let mut tuple = PhpArray::new();
                    for v in row {
                        let _ = tuple.append(v);
                    }
                    Zval::Array(Rc::new(tuple))
                } else {
                    self.call_value(cb.clone(), row)?
                };
                let _ = out.append(val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `array_filter($array, $callback?, $mode = 0)` (step 18, D-18.6). Keys are
    /// always preserved. With no callback, truthy values are kept; otherwise the
    /// callback receives the value (mode 0), the key (`ARRAY_FILTER_USE_KEY`), or
    /// `(value, key)` (`ARRAY_FILTER_USE_BOTH`).
    fn ho_array_filter(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "array_filter() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arr = match self.eval(first)?.deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_filter(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        let cb = match args.get(1) {
            Some(a) => match self.eval(a)?.deref_clone() {
                Zval::Null => None,
                v => Some(v),
            },
            None => None,
        };
        let mode = match args.get(2) {
            Some(a) => {
                let v = self.eval(a)?.deref_clone();
                convert::to_long_cast(&v, &mut self.diags)
            }
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
                    let r = self.call_value(c.clone(), call_args)?;
                    convert::to_bool(&r, &mut self.diags)
                }
            };
            if keep {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `usort(&$array, $callback)` (step 18, D-18.6): sort the array's values in
    /// place by the comparator, re-index 0..n, and return `true`. The first
    /// argument is taken by reference (like `sort`); the comparator returns an
    /// int (`$a <=> $b`-style).
    fn ho_usort(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let (Some(arr_expr), Some(cmp_expr)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(format!(
                "usort() expects exactly 2 arguments, {} given",
                args.len()
            )));
        };
        let ExprKind::Var(slot) = arr_expr.kind else {
            return Err(PhpError::Error(
                "usort(): Argument #1 ($array) could not be passed by reference".to_string(),
            ));
        };
        let cmp = self.eval(cmp_expr)?.deref_clone();
        let cell = self.slot_cell(slot as usize);
        let values: Vec<Zval> = match &*cell.borrow() {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "usort(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };

        let sorted = self.merge_sort_with(&cmp, values)?;
        let mut out = PhpArray::new();
        for v in sorted {
            let _ = out.append(v);
        }
        *cell.borrow_mut() = Zval::Array(Rc::new(out));
        Ok(Zval::Bool(true))
    }

    /// Stable merge sort driven by a PHP comparator callback (used by `usort`).
    /// Stability matches PHP 8's sort guarantee; the comparator's return value is
    /// cast to an int (`<= 0` keeps the left element first).
    fn merge_sort_with(&mut self, cmp: &Zval, mut vals: Vec<Zval>) -> Result<Vec<Zval>, PhpError> {
        let n = vals.len();
        if n <= 1 {
            return Ok(vals);
        }
        let right = vals.split_off(n / 2);
        let left = self.merge_sort_with(cmp, vals)?;
        let right = self.merge_sort_with(cmp, right)?;
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

    /// Invoke a sort comparator and reduce its result to an int (step 18).
    fn compare_with_callback(&mut self, cmp: &Zval, a: &Zval, b: &Zval) -> Result<i64, PhpError> {
        let r = self.call_value(cmp.clone(), vec![a.clone(), b.clone()])?;
        Ok(convert::to_long_cast(&r, &mut self.diags))
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
                    // `(string)$obj` honours `__toString` (step 19-6); other values
                    // use the cast funnel (which warns on NaN).
                    CastKind::String if matches!(v, Zval::Object(_)) => {
                        Zval::Str(self.stringify(&v)?)
                    }
                    CastKind::String => Zval::Str(convert::to_zstr_cast(&v, &mut self.diags)),
                    CastKind::Bool => Zval::Bool(convert::to_bool(&v, &mut self.diags)),
                    CastKind::Array => array_cast(v),
                    CastKind::Object => self.object_cast(v),
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

            ExprKind::IncDecPlace { place, inc, pre } => {
                let steps = self.resolve_steps(place)?;
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let mut val = match self.read_place_value(place.base, &steps)? {
                    Zval::Undef => Zval::Null,
                    v => v,
                };
                let old = val.clone();
                if *inc {
                    ops::increment(&mut val, &mut self.diags)?;
                } else {
                    ops::decrement(&mut val, &mut self.diags)?;
                }
                self.write_place(place.base, &steps, val.clone())?;
                Ok(if *pre { val } else { old })
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

            ExprKind::Call { name, args, named } => {
                // A user-defined function shadows the builtin namespace (PHP
                // resolves both from one function table; you cannot redefine a
                // builtin, but a user function wins when present). User functions
                // bind by-reference parameters (step 11b), so their arguments are
                // resolved against the declaration rather than blindly evaluated.
                if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
                    let (argv, spread_named) = self.eval_call_args(idx, args)?;
                    // Named arguments — explicit (step 38) and from unpacking string
                    // keys (step 40) — are placed by parameter name after the
                    // positional ones.
                    let f: &'p FnDecl = &self.funcs[idx];
                    let argv = self.apply_named_args(f, argv, spread_named, named)?;
                    let result = self.call_user_fn(idx, argv)?;
                    // A by-reference function returns a `Zval::Ref`; in this
                    // (value) context it must be copied, not aliased — only
                    // `$y = &f()` keeps the cell (D-13.6).
                    return Ok(match result {
                        Zval::Ref(cell) => cell.borrow().clone(),
                        other => other,
                    });
                }
                // Named arguments to a builtin are a scope-out (step 38, D-38.2):
                // the registry carries no parameter-name metadata. User functions
                // are handled above.
                if !named.is_empty() {
                    return Err(PhpError::Error(format!(
                        "named arguments to builtin {}() are not supported",
                        String::from_utf8_lossy(name)
                    )));
                }
                // Higher-order builtins need to invoke a callback, so they are
                // run by the evaluator itself rather than the (pure) registry
                // (step 18, D-18.6). They take precedence over the registry.
                if let Some(res) = self.dispatch_higher_order(name, args) {
                    return res;
                }
                // Class-introspection builtins read the current object / class
                // table, so the evaluator answers them directly (step 20 coda).
                if let Some(res) = self.dispatch_class_introspection(name, args) {
                    return res;
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
                self.dispatch_value_builtin(f, &argv)
            }

            // A closure / arrow expression: snapshot its captures in the active
            // frame and build the `Zval::Closure` value (step 18, D-18.2/D-18.3).
            ExprKind::Closure {
                fn_idx,
                captures,
                bind_this,
            } => {
                let bound = self.bind_captures(captures);
                // A non-static closure captures the current `$this` (step 19-6).
                let bound_this = if *bind_this { self.cur_this.clone() } else { None };
                Ok(Zval::Closure(Rc::new(Closure {
                    fn_idx: *fn_idx,
                    captures: bound,
                    named: None,
                    bound_this,
                    id: self.next_id(),
                    info: Rc::clone(&self.closure_info[*fn_idx]),
                })))
            }

            // A first-class callable `name(...)` — a closure wrapping a name.
            ExprKind::FirstClassCallable(name) => {
                let info = self.first_class_info(name);
                Ok(Zval::Closure(Rc::new(Closure {
                    fn_idx: 0,
                    captures: Vec::new(),
                    named: Some(PhpStr::new(name.to_vec())),
                    bound_this: None,
                    id: self.next_id(),
                    info,
                })))
            }

            // A dynamic call `$f(...)` dispatched on the callee value (step 18,
            // D-18.5). Arguments are evaluated by value (left to right).
            ExprKind::CallDynamic { callee, args } => {
                let c = self.eval(callee)?.deref_clone();
                let mut argv = Vec::with_capacity(args.len());
                for a in args {
                    argv.push(self.eval(a)?);
                }
                self.call_value(c, argv)
            }

            // A spread `...$e` is only meaningful as a call argument, where the
            // dedicated argument-evaluation paths intercept it (step 40). Reaching
            // the generic evaluator means it appeared elsewhere.
            ExprKind::Spread(_) => Err(PhpError::Error(
                "Cannot use spread operator outside of function call".to_string(),
            )),

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
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let value = self.eval(rhs)?;
                self.write_place(place.base, &steps, value.clone())?;
                Ok(value)
            }

            ExprKind::AssignOpPlace(op, place, rhs) => {
                let steps = self.resolve_steps(place)?;
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
                let cur = self.read_place_value(place.base, &steps)?;
                let rv = self.eval(rhs)?;
                let res = self.apply_binop(*op, cur, rv)?;
                self.write_place(place.base, &steps, res.clone())?;
                Ok(res)
            }

            ExprKind::AssignCoalescePlace(place, rhs) => {
                let steps = self.resolve_steps(place)?;
                // Magic property: `__isset` decides, then `__get` (existing) or
                // `__set` (new), step 22, D-22.6.
                if let [Step::Prop(name)] = steps.as_slice() {
                    if let Zval::Object(o) = self.base_clone(place.base) {
                        if let Some(r) = self.magic_isset_bool(&o, name) {
                            return if r? {
                                self.prop_value_silent(&o, name)
                            } else {
                                let value = self.eval(rhs)?;
                                self.write_place(place.base, &steps, value.clone())?;
                                Ok(value)
                            };
                        }
                    }
                }
                self.check_first_prop_write(place.base, &steps, MagicAccess::Set, b"__set")?;
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
                    if !self.place_isset(p.base, &steps)? {
                        return Ok(Zval::Bool(false));
                    }
                }
                Ok(Zval::Bool(true))
            }

            ExprKind::Empty(place) => {
                let steps = self.resolve_steps(place)?;
                let empty = self.place_empty(place.base, &steps)?;
                Ok(Zval::Bool(empty))
            }

            // `print expr` (step 46): emit the stringified value (honouring
            // `__toString`, like echo), then evaluate to int(1).
            ExprKind::Print(e) => {
                let z = self.eval(e)?;
                let s = self.stringify(&z)?;
                self.emit(s.as_bytes());
                Ok(Zval::Long(1))
            }

            // `exit`/`die [arg]` (step 46): the argument follows PHP's
            // `string|int $status` union (see `exit_status`). Raised on the `Err`
            // channel so it is uncatchable and bypasses `finally`.
            ExprKind::Exit(arg) => {
                let code = match arg {
                    Some(e) => {
                        let v = self.eval(e)?;
                        self.exit_status(v)?
                    }
                    None => 0,
                };
                Err(PhpError::Exit(code))
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

            ExprKind::New { class, args, named } => self.eval_new(class, args, named),

            ExprKind::MethodCall {
                object,
                method,
                args,
                named,
                nullsafe,
            } => {
                let recv = self.eval(object)?.deref_clone();
                if *nullsafe && matches!(recv, Zval::Null | Zval::Undef) {
                    return Ok(Zval::Null);
                }
                let (argv, spread_named) = self.eval_value_args(args)?;
                // Methods on a closure value (`$c->bindTo(...)`) are built-in
                // (step 19-6), not user-class dispatch. Named args (and named
                // unpacking) there are out of scope (step 38 / 40).
                if let Zval::Closure(cl) = &recv {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.closure_method(cl, method, argv);
                }
                // Methods on a `Generator` value (`->current()`, `->next()`, …)
                // are built-in (step 39), dispatched ahead of user-class lookup.
                if let Zval::Generator(gs) = &recv {
                    if let Some(e) = self.reject_named(named, &spread_named) {
                        return Err(e);
                    }
                    return self.generator_method(Rc::clone(gs), method, argv);
                }
                self.call_method(recv, method, argv, spread_named, named)
            }

            ExprKind::PropGet {
                object,
                name,
                nullsafe,
            } => {
                let recv = self.eval(object)?.deref_clone();
                if *nullsafe && matches!(recv, Zval::Null | Zval::Undef) {
                    return Ok(Zval::Null);
                }
                self.read_property(&recv, name)
            }

            ExprKind::This => match &self.cur_this {
                Some(obj) => Ok(obj.clone()),
                None => Err(PhpError::Error(
                    "Using $this when not in object context".to_string(),
                )),
            },

            ExprKind::StaticCall {
                class,
                method,
                args,
                named,
            } => {
                let (argv, spread_named) = self.eval_value_args(args)?;
                self.call_static(class, method, argv, spread_named, named)
            }

            ExprKind::ClassConst { class, name } => self.eval_class_const(class, name),

            ExprKind::StaticProp { class, name } => {
                let cell = self.static_prop_cell(class, name)?;
                let v = cell.borrow().deref_clone();
                Ok(v)
            }

            ExprKind::StaticPropAssign {
                class,
                name,
                op,
                rhs,
            } => {
                let cell = self.static_prop_cell(class, name)?;
                match op {
                    StaticAssignOp::Plain => {
                        let v = self.eval(rhs)?;
                        *cell.borrow_mut() = v.clone();
                        Ok(v)
                    }
                    StaticAssignOp::Coalesce => {
                        let cur = cell.borrow().deref_clone();
                        if matches!(cur, Zval::Null | Zval::Undef) {
                            let v = self.eval(rhs)?;
                            *cell.borrow_mut() = v.clone();
                            Ok(v)
                        } else {
                            Ok(cur)
                        }
                    }
                    StaticAssignOp::Op(b) => {
                        let cur = cell.borrow().deref_clone();
                        let rv = self.eval(rhs)?;
                        let res = self.apply_binop(*b, cur, rv)?;
                        *cell.borrow_mut() = res.clone();
                        Ok(res)
                    }
                }
            }

            ExprKind::StaticPropIncDec {
                class,
                name,
                inc,
                pre,
            } => {
                let cell = self.static_prop_cell(class, name)?;
                let old = cell.borrow().deref_clone();
                {
                    let mut guard = cell.borrow_mut();
                    if *inc {
                        ops::increment(&mut guard, &mut self.diags)?;
                    } else {
                        ops::decrement(&mut guard, &mut self.diags)?;
                    }
                }
                Ok(if *pre { cell.borrow().deref_clone() } else { old })
            }

            ExprKind::InstanceOf { expr, class } => {
                let v = self.eval(expr)?.deref_clone();
                let result = match &v {
                    Zval::Object(o) => match self.resolve_class_ref(class) {
                        Ok(target) => self.is_instance_of(o.borrow().class_id as usize, target),
                        // An unknown class on the RHS is simply not matched (PHP
                        // does not error here under the CLI without autoloading).
                        Err(_) => false,
                    },
                    // A `Generator` satisfies `Generator`, `Iterator`, and
                    // `Traversable` (the built-in interface chain), step 39-7.
                    Zval::Generator(_) => match class {
                        ClassRef::Named(name) => matches!(
                            name.to_ascii_lowercase().as_slice(),
                            b"generator" | b"iterator" | b"traversable"
                        ),
                        _ => false,
                    },
                    _ => false,
                };
                Ok(Zval::Bool(result))
            }

            // `throw <expr>` (step 20): evaluate the operand and unwind with
            // `PhpError::Thrown`, which propagates through every `?` until a
            // matching `catch` (or the top, where it renders as an uncaught fatal).
            ExprKind::Throw(e) => {
                let v = self.eval(e)?.deref_clone();
                Err(PhpError::Thrown(v))
            }

            // `yield [$k =>] [$v]` (step 39): suspend the running generator,
            // handing out the (key, value); the expression evaluates to the value
            // the next resume delivers (`send()` argument / NULL for `next()`).
            ExprKind::Yield { key, value } => {
                let value = match value {
                    Some(e) => self.eval(e)?,
                    None => Zval::Null,
                };
                let key = match key {
                    Some(e) => GenKey::Keyed(self.eval(e)?),
                    None => GenKey::Auto,
                };
                self.gen_suspend(key, value)
            }

            // `yield from <iterator>` — delegated iteration (step 39-6).
            ExprKind::YieldFrom(e) => self.eval_yield_from(e),
        }
    }

    /// Whether an object of `class_id` is an instance of `target` (step 19-5,
    /// D-19.16): the class itself, any ancestor, or any implemented interface,
    /// transitively through interface inheritance.
    fn is_instance_of(&self, class_id: ClassId, target: ClassId) -> bool {
        // `Stringable` is auto-implemented (step 24-1): any class with a
        // resolvable `__toString` satisfies it, even without an explicit
        // `implements Stringable`. PHP 8 adds this interface implicitly.
        if self.class_index.get(b"stringable".as_slice()) == Some(&target)
            && self.resolve_method(class_id, b"__toString").is_some()
        {
            return true;
        }
        let classes: &'p [ClassDecl] = self.classes;
        let mut cur = Some(class_id);
        while let Some(c) = cur {
            if c == target {
                return true;
            }
            if classes[c].interfaces.iter().any(|&i| self.iface_is_a(i, target)) {
                return true;
            }
            cur = classes[c].parent;
        }
        false
    }

    /// Whether `cid` is a `Throwable` (step 20): used to stamp `line`/`file` on
    /// exception instances at `new` time. The prelude guarantees `Throwable`
    /// exists, so the lookup is normally `Some`.
    fn is_throwable(&self, cid: ClassId) -> bool {
        match self.class_index.get(b"throwable".as_slice()) {
            Some(&tid) => self.is_instance_of(cid, tid),
            None => false,
        }
    }

    /// Whether interface `i` is, or transitively extends, `target` (step 19-5).
    fn iface_is_a(&self, i: ClassId, target: ClassId) -> bool {
        if i == target {
            return true;
        }
        let classes: &'p [ClassDecl] = self.classes;
        classes[i].interfaces.iter().any(|&p| self.iface_is_a(p, target))
    }

    fn apply_binop(&mut self, op: BinOp, a: Zval, b: Zval) -> Result<Zval, PhpError> {
        // String concatenation honours `__toString` on object operands (step
        // 19-6); `ops::concat` (in `php_types`) cannot reach the evaluator.
        if matches!(op, BinOp::Concat) && (matches!(a, Zval::Object(_)) || matches!(b, Zval::Object(_)))
        {
            let mut out = self.stringify(&a)?.as_bytes().to_vec();
            out.extend_from_slice(self.stringify(&b)?.as_bytes());
            return Ok(Zval::Str(PhpStr::new(out)));
        }
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
            // `$this`-rooted place: the current object, or NULL outside a method
            // (the write path's guard turns that into the proper fatal first).
            PlaceBase::This => self
                .cur_this
                .as_ref()
                .map(Zval::deref_clone)
                .unwrap_or(Zval::Null),
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
            // `$o->p ??` / `$o->p ?? d`: a magic property is `__isset` then (only
            // if set) `__get`; a plain property is read silently — no undefined
            // warning, unlike a normal read (step 22, D-22.6).
            ExprKind::PropGet {
                object,
                name,
                nullsafe,
            } => {
                let recv = self.eval_isset(object)?.deref_clone();
                if matches!(recv, Zval::Null | Zval::Undef) {
                    return Ok(Zval::Null);
                }
                if let Zval::Object(o) = &recv {
                    if let Some(r) = self.magic_isset_bool(o, name) {
                        return if r? {
                            self.prop_value_silent(o, name)
                        } else {
                            Ok(Zval::Null)
                        };
                    }
                    return Ok(o
                        .borrow()
                        .props
                        .get(name)
                        .map(Zval::deref_clone)
                        .unwrap_or(Zval::Null));
                }
                let _ = nullsafe;
                Ok(Zval::Null)
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
            Zval::Array(_) | Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) => {
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
                PlaceStep::Prop(name) => out.push(Step::Prop(name.clone())),
            }
        }
        Ok(out)
    }

    /// Write `value` to `slot` following the resolved steps, auto-vivifying
    /// intermediate arrays and copying-on-write shared ones.
    fn write_place(&mut self, base: PlaceBase, steps: &[Step], value: Zval) -> Result<(), PhpError> {
        // A `$this`-rooted write outside any method is the same fatal as reading
        // `$this` there (step 19, D-19.5).
        if base == PlaceBase::This && self.cur_this.is_none() {
            return Err(PhpError::Error(
                "Using $this when not in object context".to_string(),
            ));
        }
        // Magic `__set` for a single trailing property write on an object whose
        // property is missing or inaccessible (step 22, D-22.1/D-22.2).
        if let [Step::Prop(name)] = steps {
            let recv = self.base_clone(base);
            if let Zval::Object(o) = &recv {
                if let Some((defc, obj_cid, oid, m)) =
                    self.magic_prop_method(o, name, MagicAccess::Set, b"__set")
                {
                    let key = (oid, MagicAccess::Set, name.to_vec());
                    self.magic_guard.insert(key.clone());
                    let argv = vec![Zval::Str(PhpStr::new(name.to_vec())), value];
                    let r = self.invoke_method(Some(recv.clone()), defc, obj_cid, m, b"__set", argv);
                    self.magic_guard.remove(&key);
                    return r.map(|_| ());
                }
            }
        }
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
        // Named arguments in a by-ref-return binding (`$y =& f(x: 1)`) are an
        // unhandled edge here (step 38); the positional path covers the common case.
        let ExprKind::Call { name, args, .. } = &call.kind else {
            // Lowering only builds `AssignRefCall` around a call; be defensive.
            return Ok((self.eval(call)?, false));
        };
        if let Some(&idx) = self.fn_index.get(&name.to_ascii_lowercase()) {
            let by_ref = self.funcs[idx].by_ref;
            let (argv, spread_named) = self.eval_call_args(idx, args)?;
            let f: &'p FnDecl = &self.funcs[idx];
            let argv = self.apply_named_args(f, argv, spread_named, &[])?;
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
            cur = match step {
                Step::Key(k) => self.read_index(&cur, &key_to_zval(k), false)?,
                Step::Prop(name) => self.read_property(&cur, name)?,
                Step::Append => return Ok(Zval::Null),
            };
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
            match step {
                Step::Key(k) => match &cur {
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
                },
                // `isset($o->prop)` is true iff the property exists and is
                // non-null (the caller checks non-null), step 19-2.
                Step::Prop(name) => {
                    let next = match &cur {
                        Zval::Object(o) => o.borrow().props.get(name).map(Zval::deref_clone),
                        _ => return None,
                    };
                    match next {
                        Some(v) => cur = v,
                        None => return None,
                    }
                }
                Step::Append => return None,
            }
        }
        Some(cur)
    }

    /// `unset($slot)` / `unset($a[k]...)`: drop a variable or array element. A
    /// single trailing property on an object whose property is missing or
    /// inaccessible routes to `__unset` (step 22, D-22.1).
    fn unset_place(&mut self, base: PlaceBase, steps: &[Step]) -> Result<(), PhpError> {
        if steps.is_empty() {
            // Drop *this* binding only: replacing it with a fresh value slot
            // releases this alias's share of any reference cell, leaving other
            // aliases untouched (D-R5).
            *slot_mut!(self, base) = Zval::Undef;
            return Ok(());
        }
        if let [Step::Prop(name)] = steps {
            let recv = self.base_clone(base);
            if let Zval::Object(o) = &recv {
                if let Some((defc, obj_cid, oid, m)) =
                    self.magic_prop_method(o, name, MagicAccess::Unset, b"__unset")
                {
                    let key = (oid, MagicAccess::Unset, name.to_vec());
                    self.magic_guard.insert(key.clone());
                    let arg = Zval::Str(PhpStr::new(name.to_vec()));
                    let r = self.invoke_method(Some(recv.clone()), defc, obj_cid, m, b"__unset", vec![arg]);
                    self.magic_guard.remove(&key);
                    return r.map(|_| ());
                }
                // An enum case property is readonly — it cannot be unset
                // (step 23, D-23.4).
                if o.borrow().info.is_enum_case {
                    let b = o.borrow();
                    return Err(PhpError::Error(format!(
                        "Cannot unset readonly property {}::${}",
                        String::from_utf8_lossy(b.class_name.as_bytes()),
                        String::from_utf8_lossy(name)
                    )));
                }
            }
        }
        match slot_mut!(self, base) {
            Zval::Ref(cell) => {
                let z = &mut *cell.borrow_mut();
                unset_into(z, steps);
            }
            other => unset_into(other, steps),
        }
        Ok(())
    }
}

/// A resolved place step: an array key, the append marker `[]`, or an object
/// property name (`->prop`, step 19, D-19.9).
enum Step {
    Key(Key),
    Append,
    Prop(Box<[u8]>),
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
    // A property step navigates into a shared object in place (step 19, D-19.9):
    // unlike arrays there is no copy-on-write write-back, since all handles share
    // the same `Rc<RefCell<Object>>`.
    if let Step::Prop(name) = first {
        match target {
            Zval::Object(o) => {
                let mut obj = o.borrow_mut();
                // An enum case is immutable: every property is readonly and no
                // dynamic property may be created on it (step 23, D-23.4).
                if obj.info.is_enum_case {
                    let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
                    let prop = String::from_utf8_lossy(name).into_owned();
                    return Err(PhpError::Error(if obj.props.contains(name) {
                        format!("Cannot modify readonly property {cls}::${prop}")
                    } else {
                        format!("Cannot create dynamic property {cls}::${prop}")
                    }));
                }
                if rest.is_empty() {
                    obj.props.set(name, value);
                } else {
                    if !obj.props.contains(name) {
                        obj.props.set(name, Zval::Array(Rc::new(PhpArray::new())));
                    }
                    let child = obj.props.get_mut(name).expect("property just inserted");
                    write_into(child, rest, value, diags)?;
                }
            }
            // Assigning a property on a non-object: PHP warns and discards the
            // write (the null-vivification edge is deferred to 19-2).
            other => {
                diags.push(Diag::Warning(format!(
                    "Attempt to assign property \"{}\" on {}",
                    String::from_utf8_lossy(name),
                    other.error_type_name()
                )));
            }
        }
        return Ok(());
    }
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
        // A property step is handled before `ensure_array_mut` above (objects are
        // navigated in place, not as arrays), so it never reaches here.
        Step::Prop(_) => unreachable!("property step handled before array navigation"),
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

/// Normalise the `$newThis` argument of `bindTo`/`bind`/`call` (step 19-6): an
/// object binds, `null` (or anything else) clears the binding.
fn closure_this_arg(v: Option<Zval>) -> Option<Zval> {
    match v.map(|v| v.deref_clone()) {
        Some(o @ Zval::Object(_)) => Some(o),
        _ => None,
    }
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
/// Build a PHP `$matches`-style array from regex captures: index 0 is the whole
/// match, index n the n-th group, with unmatched groups as empty strings
/// (step 27, numeric groups only).
/// The `Class->method` / `Class::method` / `function` display name of a stack
/// frame (step 28).
fn frame_display(frame: &CallFrame) -> Vec<u8> {
    let mut d = Vec::new();
    if let Some(class) = &frame.class {
        d.extend_from_slice(class);
        d.extend_from_slice(if frame.is_static { b"::" } else { b"->" });
    }
    d.extend_from_slice(&frame.function);
    d
}

/// `PREG_OFFSET_CAPTURE`.
const PREG_OFFSET_CAPTURE: i64 = 256;
/// `PREG_UNMATCHED_AS_NULL`.
const PREG_UNMATCHED_AS_NULL: i64 = 512;
/// `PREG_SET_ORDER`.
const PREG_SET_ORDER: i64 = 2;

/// Build one match's `$matches` array. Named groups are emitted as the name key
/// immediately followed by their numeric index (PHP order). With
/// `PREG_OFFSET_CAPTURE` each value becomes a `[string, byte-offset]` pair; with
/// `PREG_UNMATCHED_AS_NULL` unmatched groups are `null` and every group is kept,
/// otherwise trailing unmatched groups are dropped.
fn captures_array(re: &crate::preg::Engine, caps: &crate::preg::Caps, flags: i64) -> Zval {
    let offset = flags & PREG_OFFSET_CAPTURE != 0;
    let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
    let names = re.capture_names();
    let limit = if as_null {
        caps.len().saturating_sub(1)
    } else {
        (0..caps.len())
            .rev()
            .find(|&i| caps.get(i).is_some())
            .unwrap_or(0)
    };
    let mut arr = PhpArray::new();
    for i in 0..=limit {
        let val = capture_value(caps.get(i), offset, as_null);
        if let Some(Some(name)) = names.get(i) {
            arr.insert(Key::from_bytes(name.as_bytes()), val.clone());
        }
        arr.insert(Key::Int(i as i64), val);
    }
    Zval::Array(Rc::new(arr))
}

/// A single capture group's value, honouring `PREG_OFFSET_CAPTURE` /
/// `PREG_UNMATCHED_AS_NULL`.
fn capture_value(m: Option<&crate::preg::CapMatch>, offset: bool, as_null: bool) -> Zval {
    match m {
        Some(mm) => {
            let s = Zval::Str(PhpStr::new(mm.text.as_bytes().to_vec()));
            if offset {
                offset_pair(s, mm.start as i64)
            } else {
                s
            }
        }
        None => {
            let base = if as_null {
                Zval::Null
            } else {
                Zval::Str(PhpStr::new(Vec::new()))
            };
            if offset {
                offset_pair(base, -1)
            } else {
                base
            }
        }
    }
}

/// `[value, offset]` pair for `PREG_OFFSET_CAPTURE`.
fn offset_pair(value: Zval, off: i64) -> Zval {
    let mut a = PhpArray::new();
    let _ = a.append(value);
    let _ = a.append(Zval::Long(off));
    Zval::Array(Rc::new(a))
}

fn unset_into(target: &mut Zval, steps: &[Step]) {
    let (first, rest) = match steps.split_first() {
        Some(p) => p,
        None => return,
    };
    match first {
        Step::Key(k) => {
            if let Zval::Array(rc) = target {
                if rest.is_empty() {
                    Rc::make_mut(rc).remove(k);
                } else if let Some(child) = Rc::make_mut(rc).get_mut(k) {
                    unset_into(child, rest);
                }
            }
        }
        // `unset($o->prop)` removes the property from the shared object in place
        // (step 19-2).
        Step::Prop(name) => {
            if let Zval::Object(o) = target {
                let mut obj = o.borrow_mut();
                if rest.is_empty() {
                    obj.props.remove(name);
                } else if let Some(child) = obj.props.get_mut(name) {
                    unset_into(child, rest);
                }
            }
        }
        Step::Append => {}
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
        Zval::Array(_) | Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) => None,
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
fn coerce_to_hint(
    value: Zval,
    hint: &TypeHint,
    diags: &mut Diags,
    strict: bool,
) -> Result<Zval, &'static str> {
    // Follow a reference to its value first (defensive; bound args are plain).
    if let Zval::Ref(c) = &value {
        let inner = c.borrow().clone();
        return coerce_to_hint(inner, hint, diags, strict);
    }
    if matches!(value, Zval::Null | Zval::Undef) {
        return if hint.nullable {
            Ok(Zval::Null)
        } else {
            Err("null")
        };
    }
    let given = php_type_name(&value);
    if strict {
        return coerce_strict(value, hint).ok_or(given);
    }
    match hint.kind {
        ScalarType::Int => coerce_to_int(value, diags),
        ScalarType::Float => coerce_to_float(value),
        ScalarType::String => coerce_to_string(value, diags),
        ScalarType::Bool => coerce_to_bool(value, diags),
    }
    .ok_or(given)
}

/// Strict-mode (`declare(strict_types=1)`) scalar check: the value's type must
/// match the hint exactly, with the single exception of `int` → `float`
/// widening. No coercion, no deprecations (step 16, D-16.3).
fn coerce_strict(value: Zval, hint: &TypeHint) -> Option<Zval> {
    match (hint.kind, &value) {
        (ScalarType::Int, Zval::Long(_))
        | (ScalarType::Float, Zval::Double(_))
        | (ScalarType::String, Zval::Str(_))
        | (ScalarType::Bool, Zval::Bool(_)) => Some(value),
        // The one widening allowed in strict mode.
        (ScalarType::Float, Zval::Long(l)) => Some(Zval::Double(*l as f64)),
        _ => None,
    }
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

/// Render metadata for an anonymous/arrow closure body (step 18-7): its
/// `{closure:file:line}` name, the program file, the definition line, and the
/// parameter descriptors.
fn closure_info_for(f: &FnDecl, file: &[u8]) -> ClosureInfo {
    ClosureInfo {
        kind: ClosureRender::Closure {
            name: PhpStr::new(f.name.to_vec()),
            file: PhpStr::new(file.to_vec()),
            line: f.line,
        },
        params: closure_params_for(f),
    }
}

/// The parameter descriptors of a function/closure body: each name (without the
/// leading `$`) and whether it is optional (has a default), in order (step 18-7).
fn closure_params_for(f: &FnDecl) -> Vec<ClosureParam> {
    f.params
        .iter()
        .map(|p| ClosureParam {
            name: f.slots[p.slot as usize].clone(),
            optional: p.default.is_some(),
        })
        .collect()
}

fn php_type_name(v: &Zval) -> &'static str {
    match v {
        Zval::Undef | Zval::Null => "null",
        Zval::Bool(_) => "bool",
        Zval::Long(_) => "int",
        Zval::Double(_) => "float",
        Zval::Str(_) => "string",
        Zval::Array(_) => "array",
        Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) => "object",
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
        Zval::Closure(_) => "of type Closure".to_string(),
        Zval::Object(o) => format!(
            "of type {}",
            String::from_utf8_lossy(o.borrow().class_name.as_bytes())
        ),
        Zval::Generator(_) => "of type Generator".to_string(),
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
