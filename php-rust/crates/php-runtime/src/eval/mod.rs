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

use corosensei::{Coroutine, CoroutineResult};

use php_types::{
    convert, dtoa, numstr, ClosureInfo, ClosureParam, ClosureRender, Diag, Diags, GenDriver, GenKey, GenStep, Key, Object, ObjectInfo, PhpArray,
    PhpError, PhpStr, Zval,
};

use crate::builtin::Registry;
use crate::hir::{
    ClassDecl, ClassId, Expr, ExprKind, FnDecl, Line, Place, PlaceBase, PlaceStep, Program, ScalarType, Slot, TypeHint,
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
mod class;

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

/// Whether the execution trace (per-statement logging) is enabled — read once
/// from `PHP_RUST_TRACE` (`exec`, `stmt` or `all`).
fn trace_exec_enabled() -> bool {
    std::env::var("PHP_RUST_TRACE")
        .map(|m| matches!(m.as_str(), "exec" | "stmt" | "all"))
        .unwrap_or(false)
}

/// Diagnostic hook (DevEx): when `PHP_RUST_TRACE` selects an HIR mode, dump the
/// lowered HIR to stderr before execution, so a failing test can be triaged as a
/// *lowering* vs *evaluation* problem. `PHP_RUST_TRACE=hir` (or `1`/`full`/`all`)
/// prints the whole `Program`; `body` prints just the top-level statement list;
/// the execution-only modes (`exec`/`stmt`) print nothing here. Stderr keeps it
/// out of the compared stdout/`rendered` stream.
fn trace_hir(name: &[u8], program: &Program) {
    let Ok(mode) = std::env::var("PHP_RUST_TRACE") else {
        return;
    };
    let full = matches!(mode.as_str(), "hir" | "1" | "full" | "all");
    if !full && mode != "body" {
        return; // exec/stmt: no static HIR dump
    }
    let file = String::from_utf8_lossy(name);
    eprintln!("=== PHP_RUST_TRACE: HIR for {file} ===");
    if full {
        eprintln!("{program:#?}");
    } else {
        eprintln!("{:#?}", program.body);
    }
    eprintln!("=== end HIR ===");
}

/// Lower `source` and run it with no builtins. Convenience wrapper over [`run`].
pub fn run_source(name: &[u8], source: &[u8]) -> Result<Outcome, crate::LowerError> {
    match crate::lower_source(name, source) {
        Ok(program) => {
            trace_hir(name, &program);
            Ok(run(&program))
        }
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
        Ok(program) => {
            trace_hir(name, &program);
            Ok(run_with(&program, registry))
        }
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
        trace_exec: trace_exec_enabled(),
        gen_yielder: None,
        mb_regex: crate::mbregex::MbRegexState::default(),
        strtok_state: None,
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
    /// DevEx execution trace (step 61b): when set, [`Self::exec_stmt`] logs each
    /// statement (line + variant, indented by call depth) to stderr, so a failing
    /// `.phpt` can be followed to the exact point execution diverges. Read once
    /// from `PHP_RUST_TRACE` (`exec`/`all`) at construction — never per-statement.
    trace_exec: bool,
    /// While a generator body runs, the type-erased `*const Yielder<ResumeIn,
    /// YieldOut>` of the active generator (step 39). The `yield` arm reborrows it
    /// to suspend. `None` outside any generator; saved/restored per resume by
    /// [`GenDriverImpl::resume`] (part of the swapped generator context).
    gen_yielder: Option<*const ()>,
    /// Persistent mbregex state (step 43): the global `mb_regex_encoding` /
    /// `mb_regex_set_options` and the `mb_ereg_search` cursor. Survives across
    /// `mb_ereg*` calls for the whole run, since the search family is stateful.
    mb_regex: crate::mbregex::MbRegexState,
    /// Persistent `strtok` cursor (step 65): the string being tokenized plus the
    /// offset where the next token search starts. `strtok($str, $tok)` sets it;
    /// `strtok($tok)` resumes from it; it is cleared (`None`) when the string is
    /// exhausted, matching PHP's `BG(strtok_string)`/`BG(strtok_last)`.
    strtok_state: Option<(Vec<u8>, usize)>,
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

// PREG flag constants and `$matches`-array builders moved to `crate::preg`
// (Session F1-pre: the VM shares them, so they no longer live in the evaluator).
pub(crate) use crate::preg::{
    capture_value, captures_array, offset_pair, PREG_OFFSET_CAPTURE, PREG_SET_ORDER,
    PREG_UNMATCHED_AS_NULL,
};

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
