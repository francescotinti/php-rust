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
    DirHandle, GenDriver, GenKey, GenState, GenStatus, GenStep, Key, Object, ObjectInfo, PhpArray,
    PhpError, PhpStr, PropVis, Props, ResKind, Resource, Stream, StreamBackend, Zval,
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
    b"unserialize",
    b"fopen",
    b"tmpfile",
    b"opendir",
    b"sscanf",
    b"fscanf",
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

/// Convert an engine constant's lowered literal to a runtime value, for
/// `constant()` (step 49c). [`resolve_constant`] only ever yields literal kinds.
fn const_literal_to_zval(kind: crate::hir::ExprKind) -> Option<Zval> {
    use crate::hir::ExprKind;
    Some(match kind {
        ExprKind::Null => Zval::Null,
        ExprKind::Bool(b) => Zval::Bool(b),
        ExprKind::Int(i) => Zval::Long(i),
        ExprKind::Float(f) => Zval::Double(f),
        ExprKind::Str(b) => Zval::Str(PhpStr::new(b)),
        _ => return None,
    })
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

mod builtins;
mod expr;
mod stmt;
mod calls;

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
        next_resource_id: 5,
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
        suppress_depth: 0,
        cur_line: 1,
        gen_yielder: None,
        mb_regex: crate::mbregex::MbRegexState::default(),
        constants: HashMap::new(),
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
    /// Next id minted for an `fopen` stream resource (the `#N` in "Resource id
    /// #N" / `resource(N)`). Monotonic; starts at 5 to match the CLI oracle,
    /// where STDIN/STDOUT/STDERR + one internal stream take ids 1–4 (D-51.4).
    next_resource_id: u32,
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
    /// Nesting depth of the `@` error-control operator (step 48). While > 0,
    /// `flush_diags` does not render diagnostics (the `@` handler drops them
    /// afterwards), so warnings/notices raised under `@` are suppressed.
    suppress_depth: usize,
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
    /// User-defined constants from `define()` (step 49c). Case-sensitive (the
    /// case-insensitive third arg was removed in PHP 8). A bare `NAME` the
    /// lowerer could not fold to an engine constant becomes [`ExprKind::Const`]
    /// and reads from here at runtime.
    constants: HashMap<Vec<u8>, Zval>,
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
        // Under `@` (step 48) diagnostics are not rendered; the operator drops
        // them once the suppressed expression finishes.
        if self.suppress_depth > 0 {
            return;
        }
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
    fn resolve_class_ref(&mut self, class: &ClassRef) -> Result<ClassId, PhpError> {
        match class {
            ClassRef::Named(name) => self.resolve_class_name(name),
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
            // `new $cls`, `$cls::m()`, `$obj::m()` (step 48): evaluate to a class
            // name (string) or an object, then resolve to a class id.
            ClassRef::Dynamic(expr) => match self.eval(expr)?.deref_clone() {
                Zval::Str(s) => {
                    // A leading namespace separator is stripped (`\Foo` == `Foo`).
                    let name = s.as_bytes();
                    let name = name.strip_prefix(b"\\").unwrap_or(name);
                    self.resolve_class_name(name)
                }
                Zval::Object(o) => Ok(o.borrow().class_id as usize),
                other => Err(PhpError::TypeError(format!(
                    "Class name must be a valid object or a string, {} given",
                    other.error_type_name()
                ))),
            },
        }
    }

    /// Resolve a class *name* (case-insensitive) to its id, or PHP's "not found"
    /// error (step 48; shared by `Named` and `Dynamic` class refs).
    fn resolve_class_name(&self, name: &[u8]) -> Result<ClassId, PhpError> {
        self.class_index
            .get(&name.to_ascii_lowercase())
            .copied()
            .ok_or_else(|| {
                PhpError::Error(format!("Class \"{}\" not found", String::from_utf8_lossy(name)))
            })
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
                let forwarding = !matches!(class, ClassRef::Named(_) | ClassRef::Dynamic(_));
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
                let forwarding = !matches!(class, ClassRef::Named(_) | ClassRef::Dynamic(_));
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
            // A resource offset casts to its id with a Warning (oracle:
            // "Resource ID#5 used as offset, casting to integer (5)", step 51).
            Zval::Resource(r) => {
                let id = r.borrow().id;
                self.diags.push(Diag::Warning(format!(
                    "Resource ID#{id} used as offset, casting to integer ({id})"
                )));
                Key::Int(id as i64)
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
        // A resource offset is its id (silent here; the noisy path warns).
        Zval::Resource(r) => Some(Key::Int(r.borrow().id as i64)),
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

/// Read/write capabilities implied by a PHP fopen mode: `r`→read, `w`/`a`/`x`/`c`
/// →write, and `+` adds the other direction. `None` for an unrecognised mode.
fn mode_caps(mode: &[u8]) -> Option<(bool, bool)> {
    let plus = mode.contains(&b'+');
    match mode.first() {
        Some(b'r') => Some((true, plus)),
        Some(b'w') | Some(b'a') | Some(b'x') | Some(b'c') => Some((plus, true)),
        _ => None,
    }
}

/// Open a `php://` stream (step 51b). `memory`/`temp` get an in-process buffer
/// (the `temp` spill-to-disk threshold is a scope-out); `stdout`/`stderr` map to
/// the process streams (write-only). Other wrappers (http/ftp/data/filter/…) are
/// unsupported → `None`, so `fopen` reports "no suitable wrapper".
fn open_php_stream(spec: &[u8], mode: &[u8]) -> Option<Stream> {
    let backend = if spec == b"memory" || spec == b"temp" || spec.starts_with(b"temp/") {
        StreamBackend::Memory(std::io::Cursor::new(Vec::new()))
    } else if spec == b"stdout" {
        StreamBackend::Stdout
    } else if spec == b"stderr" {
        StreamBackend::Stderr
    } else {
        return None;
    };
    // stdout/stderr are write-only regardless of mode; memory/temp honour it
    // (oracle: php://memory opened "r" is not writable), defaulting to read+write
    // for an unrecognised mode (php is lenient about the mode string here).
    let (readable, writable) = match backend {
        StreamBackend::Stdout | StreamBackend::Stderr => (false, true),
        _ => mode_caps(mode).unwrap_or((true, true)),
    };
    Some(Stream {
        backend,
        readable,
        writable,
        eof: false,
    })
}

/// Open a real file as a [`Stream`] per PHP `fopen` mode rules (step 51a).
/// Returns the OS error text (with Rust's " (os error N)" suffix stripped, so it
/// reads like PHP's strerror) on failure. Modes: `r`/`w`/`a`/`x`/`c` with an
/// optional `+` (adds the other direction); `b`/`t` are accepted and ignored.
fn open_file_stream(path: &[u8], mode: &[u8]) -> Result<Stream, String> {
    use std::os::unix::ffi::OsStrExt;
    let plus = mode.contains(&b'+');
    let Some((readable, writable)) = mode_caps(mode) else {
        return Err("`mode` is not a valid mode".to_string());
    };
    let mut opts = std::fs::OpenOptions::new();
    match mode.first() {
        Some(b'r') => {
            opts.read(true).write(plus);
        }
        Some(b'w') => {
            opts.write(true).create(true).truncate(true).read(plus);
        }
        Some(b'a') => {
            opts.append(true).create(true).read(plus);
        }
        Some(b'x') => {
            opts.write(true).create_new(true).read(plus);
        }
        Some(b'c') => {
            // create + write, NO truncate, position 0 (oracle: `c` keeps content).
            opts.write(true).create(true).read(plus);
        }
        _ => unreachable!("mode_caps already rejected unrecognised modes"),
    }
    let os_path = std::ffi::OsStr::from_bytes(path);
    match opts.open(os_path) {
        Ok(f) => Ok(Stream {
            backend: StreamBackend::File(f),
            readable,
            writable,
            eof: false,
        }),
        Err(e) => {
            // Strip Rust's trailing " (os error N)" so the message reads like
            // PHP's bare strerror text (e.g. "No such file or directory").
            let m = e.to_string();
            Err(m.split(" (os error").next().unwrap_or(&m).to_string())
        }
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
        Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) => "object",
        Zval::Resource(_) => "resource",
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
        Zval::Resource(_) => "of type resource".to_string(),
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
