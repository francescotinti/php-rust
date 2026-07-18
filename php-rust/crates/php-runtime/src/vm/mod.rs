//! Bytecode VM: the dispatch loop that executes a [`crate::bytecode::Module`]
//! (VM-migration Fase 4, vertical proof slice).
//!
//! This replaced the original tree-walk. Where the tree-walker recursed over the
//! HIR, the VM advances an explicit instruction pointer over a flat
//! [`crate::bytecode::Op`] stream — the property that makes generators (park the
//! `ip`) and non-structured control flow (`Jump`) ordinary instead of requiring a
//! coroutine + `unsafe` reborrow.
//!
//! # Status: proof slice
//!
//! Runs a single frame (the script body); calls, references, arrays, OOP and
//! generators are out of slice (the compiler refuses them, so the VM never sees
//! their opcodes). Value semantics are delegated to `php_types::ops` /
//! `php_types::convert`, exactly as the tree-walker does — the VM only moves
//! data and steers control flow.

use std::cell::RefCell;
use std::collections::{BTreeMap, BinaryHeap};
use std::rc::Rc;

/// Fx-hashed engine maps (class/function/constant tables, caches, GC sets):
/// stands in for Zend's precomputed zend_string hashes; iteration order of
/// these maps is never observable (std's was already random per instance).
/// `Vm::unit_fp` keeps its own std `DefaultHasher` — do not conflate.
type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;
type HashSet<T> = rustc_hash::FxHashSet<T>;

use php_types::{
    convert, open_file_stream, open_php_stream, ops, ArgPlace, ArgPlaceBase, ArgPlaceStep, Closure,
    ClosureInfo, ClosureParam, ClosureRender, Diag,
    Diags, DirHandle, GenKey, GenState, GenStatus, Key, LazyKind, Object, ObjectInfo, PhpArray, PhpError,
    PhpStr, PropVis, Props, ResKind, Resource, Stream, StreamBackend, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::bytecode::{
    Addr, ClassTarget, CompiledClass, CompiledMethod, DimBase, FieldBase, FieldStep, Func,
    Instantiable, Module, Op, PropInfo, StaticInit,
};
use crate::coerce::coerce_to_hint;
use crate::hir::{
    BinOp, CastKind, ClassId, HintKind, IncludeMode, Line, Program, ScalarType, Slot, TypeHint, UnOp,
    Visibility,
};

mod arrays;
mod calls;
mod coroutines;
mod dom;
mod exceptions;
mod xmlparser;
mod host;
mod ini;
mod run;
mod session;
mod host_reflect;
mod oop;
mod pdo;
mod mysqli;
mod gd;
mod xslt;
mod tokenizer;
mod websapi;
use arrays::*;
use calls::*;
use oop::*;

/// The result of running a [`Module`]: the program's output streams plus an
/// optional uncaught fatal. Re-exported as `php_runtime::Outcome`.
#[derive(Debug)]
pub struct VmOutcome {
    /// Pure program output (`echo` / `print` / builtins), diagnostics *not*
    /// interleaved.
    pub stdout: Vec<u8>,
    /// CLI-faithful stream: `stdout` with diagnostics rendered inline at their
    /// point of occurrence and an uncaught fatal rendered at the tail, exactly as
    /// PHP's CLI SAPI emits them. This is what a `.phpt` `--EXPECT(F)--` section is
    /// compared against.
    pub rendered: Vec<u8>,
    /// Non-fatal diagnostics raised during execution, in order.
    pub diags: Diags,
    /// The fatal that stopped execution (if any); also rendered into `rendered`.
    pub fatal: Option<PhpError>,
    /// Top-level `return` value (NULL if the script ran to completion).
    pub return_value: Zval,
    /// Process exit code from `exit`/`die`, `0..=255`; `None` for a clean run.
    pub exit_code: Option<u8>,
    /// Web SAPI only: the response headers the script accumulated (full
    /// `Name: value` lines, in order, `X-Powered-By` first). Empty under CLI.
    pub headers: Vec<Vec<u8>>,
    /// Web SAPI only: the response code set via `http_response_code()` /
    /// `header()` (with the optional custom reason phrase from a raw
    /// `header("HTTP/1.1 …")`); `None` = 200.
    pub response_code: Option<i64>,
    pub response_reason: Option<Vec<u8>>,
    /// Web SAPI only: `log_errors` lines ("PHP Warning:  … in F on line N",
    /// multiline for a fatal) for the host to timestamp onto stderr.
    pub error_log: Vec<Vec<u8>>,
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
            headers: Vec::new(),
            response_code: None,
            response_reason: None,
            error_log: Vec::new(),
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
    /// The fiber's own `@` suppression state (depth, diag marks, saved
    /// error_reporting levels), parked at suspend: error suppression is
    /// per-execution-context in PHP, so the caller's `@$fiber->start()` must
    /// not silence diagnostics raised inside the fiber body (and vice versa).
    suppress: (u32, Vec<usize>, Vec<i64>),
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

impl std::fmt::Display for VmRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmRunError::Lower(e) => write!(f, "{e}"),
            VmRunError::Unsupported(what) => write!(f, "unsupported construct: {what}"),
        }
    }
}

impl std::error::Error for VmRunError {}

/// Defensive ceiling on PHP call-stack depth, mirroring the evaluator's
/// `eval::MAX_CALL_DEPTH`. Pure PHP recursion runs *iteratively* in [`run_loop`]
/// (each call grows the heap-allocated `frames` vector, not the native Rust
/// stack), so the guard's real job is to surface a catchable PHP `Error`
/// ("Maximum call stack depth …") before runaway recursion exhausts memory —
/// instead of letting the host process die. Callback-nested calls (a host
/// builtin re-entering `run_loop`) *do* recurse natively, so as with the
/// evaluator deep-recursion safety presumes a large worker stack.
const MAX_CALL_DEPTH: usize = 25_000;

/// Lower `source`, compile it to bytecode, and run it on the VM (E2): the crate's
/// production entry point, re-exported as `php_runtime::run_source_with`. A
/// compile-time PHP `Fatal error:` (link-time, e.g. an abstract-method collision)
/// becomes a rendered [`VmOutcome`]; a genuine lowering failure or a
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
    // Retain the lowered HIR so an `eval()` in the script can be compiled against
    // the image (step 57, Phase 1c-2c): both borrows outlive the run.
    Ok(run_module_with_hir(&module, registry, Some(&program), None, &[]))
}

/// Like [`run_source_with`], with `php -d`-style INI overrides applied before
/// the script runs (phpt `--INI--` sections). Each pair sets a REGISTERED
/// directive's startup (global) and current value — an unknown name is
/// silently ignored, exactly like `php -d unknown=x` (invisible to ini_get).
pub fn run_source_with_ini(
    name: &[u8],
    source: &[u8],
    registry: &Registry,
    ini_overrides: &[(Vec<u8>, Vec<u8>)],
) -> Result<VmOutcome, VmRunError> {
    let program = match crate::lower_source(name, source) {
        Ok(p) => p,
        Err(crate::LowerError::Fatal { message, line }) => {
            return Ok(compile_fatal_outcome(name, &message, line))
        }
        Err(e) => return Err(VmRunError::Lower(e)),
    };
    let module = crate::compile::compile_program(&program, registry)
        .map_err(|crate::compile::CompileError::Unsupported(what)| VmRunError::Unsupported(what))?;
    Ok(run_module_with_hir(&module, registry, Some(&program), None, ini_overrides))
}

/// Like [`run_source_with`] but for a real CLI invocation: seed the CLI
/// superglobals (`$_SERVER`/`$argv`/`$argc`/`$_ENV`) from `argv` (element 0 is the
/// script path). The plain [`run_source_with`] passes `None`, so the test harness
/// and library callers keep the previous behaviour (those superglobals undefined).
pub fn run_source_with_argv(
    name: &[u8],
    source: &[u8],
    registry: &Registry,
    argv: &[&[u8]],
    ini_overrides: &[(Vec<u8>, Vec<u8>)],
) -> Result<VmOutcome, VmRunError> {
    log::info!(target: "phpr::run", "run {} ({} bytes)", String::from_utf8_lossy(name), source.len());
    let program = match crate::lower_source(name, source) {
        Ok(p) => p,
        Err(crate::LowerError::Fatal { message, line }) => {
            return Ok(compile_fatal_outcome(name, &message, line))
        }
        Err(e) => return Err(VmRunError::Lower(e)),
    };
    let module = crate::compile::compile_program(&program, registry)
        .map_err(|crate::compile::CompileError::Unsupported(what)| VmRunError::Unsupported(what))?;
    log::debug!(target: "phpr::compile", "compiled {}: {} functions, {} classes", String::from_utf8_lossy(name), module.functions.len(), module.classes.len());
    Ok(run_module_with_hir(&module, registry, Some(&program), Some(argv), ini_overrides))
}

/// Lower `source`, compile it, and run it on the VM with no builtins registered.
/// Convenience wrapper over [`run_source_with`] with an empty [`Registry`].
pub fn run_source(name: &[u8], source: &[u8]) -> Result<VmOutcome, VmRunError> {
    run_source_with(name, source, &Registry::new())
}

/// Build the [`VmOutcome`] for a compile-time PHP `Fatal error:` (E2; mirrors
/// `eval::compile_fatal_outcome`): rendered like a runtime fatal but without the
/// "Uncaught" prefix or "thrown in" tail.
fn compile_fatal_outcome(file: &[u8], message: &str, line: Line) -> VmOutcome {
    let file_s = String::from_utf8_lossy(file);
    // Under the web SAPI the fatal renders in html_errors form and feeds the
    // host's stderr log (the response still carries the X-Powered-By header).
    if php_types::sapi::web_request().is_some() {
        let rendered = format!(
            "<br />\n<b>Fatal error</b>:  {message} in <b>{file_s}</b> on line <b>{line}</b><br />\n"
        );
        return VmOutcome {
            rendered: rendered.into_bytes(),
            fatal: Some(PhpError::Error(message.to_string())),
            headers: vec![b"X-Powered-By: PHP/8.5.7".to_vec()],
            error_log: vec![
                format!("PHP Fatal error:  {message} in {file_s} on line {line}").into_bytes(),
            ],
            ..VmOutcome::default()
        };
    }
    let rendered =
        format!("\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n");
    VmOutcome {
        rendered: rendered.into_bytes(),
        fatal: Some(PhpError::Error(message.to_string())),
        ..VmOutcome::default()
    }
}

/// Compile-and-run is the caller's job ([`crate::compile`]); this takes the
/// already-compiled module and executes its `main`. Started without a retained
/// HIR, so an `eval()` here lowers standalone (no compile-against-image).
pub fn run_module(module: &Module, registry: &Registry) -> VmOutcome {
    run_module_with_hir(module, registry, None, None, &[])
}

/// Seed the CLI superglobals into the VM superglobal store (`$_SERVER` with the
/// environment plus `argv`/`argc`/`SCRIPT_NAME`/`SCRIPT_FILENAME`/`PHP_SELF`,
/// `$_ENV` with the environment, and `$_GET`/`$_POST`/`$_FILES`/`$_COOKIE`/
/// `$_REQUEST` as empty arrays — matching PHP CLI's default `variables_order`;
/// `$_SESSION` stays unset until `session_start`). The plain `$argv`/`$argc`
/// globals still live in the script frame's slots (they are ordinary variables,
/// not data superglobals), seeded by name only where the script references them.
/// `argv[0]` is the script path.
fn seed_cli_superglobals(
    superglobals: &mut [Zval; 8],
    slots: &mut [Zval],
    names: &[Box<[u8]>],
    argv: &[&[u8]],
) {
    use std::os::unix::ffi::OsStrExt;
    let env_array = || {
        let mut a = PhpArray::new();
        for (k, v) in std::env::vars_os() {
            a.insert(
                Key::from_bytes(k.as_os_str().as_bytes()),
                Zval::Str(PhpStr::new(v.as_os_str().as_bytes().to_vec())),
            );
        }
        a
    };
    let argv_array = || {
        let mut a = PhpArray::new();
        for arg in argv {
            let _ = a.append(Zval::Str(PhpStr::new(arg.to_vec())));
        }
        a
    };
    let script = argv.first().copied().unwrap_or(b"");
    let str_zval = |b: &[u8]| Zval::Str(PhpStr::new(b.to_vec()));
    let server_array = || {
        let mut s = env_array();
        s.insert(Key::from_bytes(b"argv"), Zval::Array(Rc::new(argv_array())));
        s.insert(Key::from_bytes(b"argc"), Zval::Long(argv.len() as i64));
        s.insert(Key::from_bytes(b"SCRIPT_NAME"), str_zval(script));
        s.insert(Key::from_bytes(b"SCRIPT_FILENAME"), str_zval(script));
        s.insert(Key::from_bytes(b"PHP_SELF"), str_zval(script));
        s
    };
    // Seed the data superglobals by their fixed store index (see SUPERGLOBAL_NAMES).
    for (i, name) in crate::bytecode::SUPERGLOBAL_NAMES.iter().enumerate() {
        superglobals[i] = match *name {
            b"_SERVER" => Zval::Array(Rc::new(server_array())),
            b"_ENV" => Zval::Array(Rc::new(env_array())),
            // `$_SESSION` is not populated by the CLI SAPI (only after session_start).
            b"_SESSION" => Zval::Undef,
            // `$_GET`/`$_POST`/`$_FILES`/`$_COOKIE`/`$_REQUEST`: empty arrays in CLI.
            _ => Zval::Array(Rc::new(PhpArray::new())),
        };
    }
    // Plain `$argv` / `$argc` globals (ordinary variables) in the script frame.
    for (i, name) in names.iter().enumerate() {
        match &name[..] {
            b"argv" => slots[i] = Zval::Array(Rc::new(argv_array())),
            b"argc" => slots[i] = Zval::Long(argv.len() as i64),
            _ => {}
        }
    }
}

/// [`run_module`] with the caller's lowered HIR retained (`main_hir`), so an
/// `eval()` in the script compiles against the image (step 57, Phase 1c-2c).
pub(crate) fn run_module_with_hir<'m>(
    module: &'m Module,
    registry: &'m Registry,
    main_hir: Option<&'m Program>,
    argv: Option<&[&[u8]]>,
    ini_overrides: &[(Vec<u8>, Vec<u8>)],
) -> VmOutcome {
    // A fresh program starts a fresh handle space: ids freed by a PREVIOUS
    // run's teardown (same thread — in-process phpt batches, unit tests)
    // must not leak into this run's `#N` numbering.
    php_types::reset_freed_object_ids();
    let mut vm = Vm {
        module,
        classes: module.classes.iter().collect(),
        class_index: module.class_index.clone(),
        class_module: vec![module; module.classes.len()],
        modules: vec![module],
        main_hir,
        seed_classes: main_hir.map(|p| p.classes.clone()).unwrap_or_default(),
        seed_traits: main_hir.map(|p| p.traits.clone()).unwrap_or_default(),
        seed_static: main_hir.map_or(0, |p| p.static_count),
        seed_globals: main_hir.map(|p| p.slots.clone()).unwrap_or_default(),
        linked_functions: HashMap::default(),
        included_files: HashSet::default(),
        unit_chain_fp: {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            // Seed the chain with the main unit's identity: two requests whose
            // entry scripts differ must never share downstream cache entries.
            module.file.hash(&mut h);
            module.classes.len().hash(&mut h);
            main_hir.is_some().hash(&mut h);
            h.finish()
        },
        autoloaders: Vec::new(),
        autoloading: HashSet::default(),
        registry,
        stdout: Vec::new(),
        rendered: Vec::new(),
        ob_stack: Vec::new(),
        diags: Diags::new(),
        diags_rendered: 0,
        fatal_line: 1,
        error_level: 30719, // PHP 8.5 E_ALL
        last_error: None,
        exception_handlers: Vec::new(),
        error_handlers: Vec::new(),
        in_error_handler: false,
        final_flush: false,
        suppress_depth: 0,
        suppress_marks: Vec::new(),
        silence_saved: Vec::new(),
        superglobals: std::array::from_fn(|_| Zval::Undef),
        preg_cache: HashMap::default(),
        frames: Vec::new(),
        next_object_id: 1,
        next_resource_id: 5,
        static_props: HashMap::default(),
        statics: vec![None; module.static_count],
        closure_statics: HashMap::default(),
        magic_guard: HashSet::default(),
        typed_refs: Vec::new(),
        created: BTreeMap::new(),
        destructed: HashSet::default(),
        gc_roots: HashMap::default(),
        gc_queue: BinaryHeap::new(),
        gc_cycle_roots: HashSet::default(),
        gc_light_demoted: Vec::new(),
        shutdown_fns: Vec::new(),
        generators: HashMap::default(),
        fibers: HashMap::default(),
        fiber_stack: Vec::new(),
        fiber_class_id: module.class_index.get(&b"fiber"[..]).copied(),
        throwable_id: module.class_index.get(&b"throwable"[..]).copied(),
        arrayaccess_id: module.class_index.get(&b"arrayaccess"[..]).copied(),
        iterator_id: module.class_index.get(&b"iterator"[..]).copied(),
        iteratoraggregate_id: module.class_index.get(&b"iteratoraggregate"[..]).copied(),
        countable_id: module.class_index.get(&b"countable"[..]).copied(),
        stringable_id: module.class_index.get(&b"stringable"[..]).copied(),
        jsonserializable_id: module.class_index.get(&b"jsonserializable"[..]).copied(),
        json_last_error: 0,
        output_started: false,
        output_start: None,
        ini: ini::IniTable::new(),
        session: session::SessionState::default(),
        response_code: None,
        web: false,
        response_headers: Vec::new(),
        response_reason: None,
        error_log: Vec::new(),
        user_abort_ignored: false,
        stream_wrappers: std::collections::HashMap::new(),
        filtered_streams: Vec::new(),
        json_active: Vec::new(),
        enum_cache: HashMap::default(),
        constants: HashMap::default(),
        mb_regex: crate::mbregex::MbRegexState::default(),
        strtok: None,
        signal_handlers: HashMap::default(),
        async_signals: false,
        uncaught_throwable: None,
        retired_main: None,
        lazy_init: HashMap::default(),
        lazy_props: HashMap::default(),
        var_dump_debug: std::collections::HashMap::new(),
        stringify_args: std::collections::HashMap::new(),
        reflect_method_info_cache: std::collections::HashMap::new(),
        lazy_options: HashMap::default(),
        reflect_object_bound: HashMap::default(),
        lazy_initializing: HashSet::default(),
        zips: HashMap::default(),
        zip_writers: HashMap::default(),
        next_zip: 1,
        pdo_conns: HashMap::default(),
        next_pdo: 1,
        mysqli_conns: HashMap::default(),
        mysqli_stmts: HashMap::default(),
        next_mysqli: 1,
        gd_images: HashMap::default(),
        next_gd: 1,
        xslt_sheets: HashMap::default(),
        next_xslt: 1,
        stream_chunk_sizes: HashMap::default(),
        seed_aliases: Vec::new(),
        umask: 0o22,
        dom_docs: HashMap::default(),
        next_dom: 1,
        libxml_internal: false,
        libxml_errors: Vec::new(),
    };
    // Register the caller module's own functions in the global link table so a
    // call made from inside an `eval()` (where `self.module` is the eval unit,
    // not `main`) still resolves them — and runs each in its defining module, so
    // `__FILE__`/backtrace stay attributed to the caller's file (step 57, Phase
    // 1c-2c). Normal (non-eval) calls hit `self.module.functions` first, so these
    // entries only matter across the eval boundary.
    // Only the unconditionally-hoisted functions are pre-registered; a conditional
    // declaration registers itself via its `Op::DeclareFn` when reached.
    for (idx, f) in module.functions.iter().enumerate() {
        if !module.conditional_fns.contains(&idx) {
            vm.linked_functions.insert(f.name.to_ascii_lowercase(), (module, idx));
        }
    }
    // Predefined CLI stream constants `STDIN`/`STDOUT`/`STDERR` (resource ids
    // #1/#2/#3, as the PHP CLI SAPI), so a script can `fwrite(STDERR, …)` (step 57).
    let std_stream = |id: u32, backend: StreamBackend, readable: bool, writable: bool, uri: &[u8], mode: &[u8]| {
        Zval::Resource(Rc::new(RefCell::new(Resource::new(
            id,
            Stream {
                backend,
                readable,
                writable,
                eof: false,
                uri: uri.to_vec(),
                mode: mode.to_vec(),
                eof_on_exhaust: false,
                filters: None,
            },
        ))))
    };
    vm.constants.insert(
        b"STDIN".to_vec(),
        std_stream(1, StreamBackend::Stdin, true, false, b"php://stdin", b"rb"),
    );
    vm.constants.insert(
        b"STDOUT".to_vec(),
        std_stream(2, StreamBackend::Stdout, false, true, b"php://stdout", b"wb"),
    );
    vm.constants.insert(
        b"STDERR".to_vec(),
        std_stream(3, StreamBackend::Stderr, false, true, b"php://stderr", b"wb"),
    );
    // `PHP_BINARY`: absolute path to the running interpreter (PHP predefines it).
    // Composer and symfony reference it (e.g. xdebug-handler restart); a missing
    // constant otherwise fatals during `bin/composer`.
    {
        use std::os::unix::ffi::OsStrExt;
        let path = std::env::current_exe()
            .map(|p| p.as_os_str().as_bytes().to_vec())
            .unwrap_or_else(|_| b"php".to_vec());
        vm.constants.insert(b"PHP_BINARY".to_vec(), Zval::Str(PhpStr::new(path)));
    }
    // ext/dom "new DOM" namespaced constant (PHP 8.4+): parse HTML without the
    // implicit HTML namespace. Registered here because it is namespace-
    // qualified (the lowering's engine-constant fold is for global names).
    vm.constants.insert(b"Dom\\HTML_NO_DEFAULT_NS".to_vec(), Zval::Long(2_147_483_648));
    // Link-time `Serializable` policy for the hoisted (unconditional) classes:
    // the deprecation is staged in `diags` (flushed with the first statement,
    // so it prints before any script output, like Zend's compile-time emission)
    // and an enum implementing it aborts before `main` runs. Conditional
    // classes are checked by their `Op::DeclareClass` instead. (Classes linked
    // later by an include unit are not checked — deliberate slice.)
    let mut link_fatal = None;
    for cid in 0..vm.classes.len() {
        if module.conditional_classes.contains(&cid) {
            continue;
        }
        if let Err(e) = vm.serializable_link_check(cid) {
            link_fatal = Some(e);
            break;
        }
    }
    vm.frames.push(Frame::new(&module.main, module));
    // A web request installed on this thread (phpr -S) switches the run to
    // web-SAPI behaviour and seeds the request superglobals; the `argv` CLI
    // seeding below is skipped (the cli-server registers no argv/argc).
    if let Some(req) = php_types::sapi::web_request() {
        vm.web = true;
        vm.response_headers.push(b"X-Powered-By: PHP/8.5.7".to_vec());
        websapi::seed_web_superglobals(&mut vm.superglobals, &req);
        // The cli-server SAPI's own startup INI values (oracle-pinned).
        for (name, value) in [
            (&b"html_errors"[..], &b"1"[..]),
            (b"output_buffering", b"4096"),
            (b"implicit_flush", b""),
            // Zend's CLI SAPI hardwires these two; the cli-server keeps the
            // php.ini values (site-health debug tab reads them, WP-11).
            (b"max_execution_time", b"30"),
            (b"max_input_time", b"60"),
        ] {
            if let Some(e) = vm.ini.0.get_mut(name) {
                e.global = value.to_vec();
                e.local = value.to_vec();
            }
        }
    }
    // Seed the CLI superglobals (`$_SERVER`/`$argv`/`$argc`/`$_ENV`) into the
    // script frame's global slots for a real CLI run; the test harness passes
    // `None`, leaving them undefined as before. Only slots the script references
    // exist, so a script that never mentions `$_SERVER` is unaffected.
    if let (Some(argv), Some(prog)) = (argv, main_hir) {
        seed_cli_superglobals(&mut vm.superglobals, &mut vm.frames[0].slots, &prog.slots, argv);
        // Zend's CLI SAPI registers `$argv`/`$argc` in the global symbol table
        // unconditionally (register_argc_argv=On), so `$GLOBALS['argv']` works
        // from ANY unit — wp-cli's Runner reads it from a required file while
        // the entry script never mentions it. Register the names in the
        // cross-unit global registry; the named-slot path above already filled
        // them when the main script declares the variables.
        let mut arr = PhpArray::new();
        for a in argv {
            let _ = arr.append(Zval::Str(PhpStr::new(a.to_vec())));
        }
        let slot = vm.global_slot_by_name(b"argv");
        if matches!(vm.frames[0].slots[slot], Zval::Undef) {
            vm.frames[0].slots[slot] = Zval::Array(Rc::new(arr));
        }
        let slot = vm.global_slot_by_name(b"argc");
        if matches!(vm.frames[0].slots[slot], Zval::Undef) {
            vm.frames[0].slots[slot] = Zval::Long(argv.len() as i64);
        }
    }
    // `php -d`-style INI overrides (phpt --INI-- sections): a registered
    // directive gets the value as its startup default too (ini_restore under
    // run-tests.php reverts to the -d value); an unknown name is ignored,
    // invisible to ini_get exactly like `php -d unknown=x`. Validation and
    // ext/session's module-startup deprecations render "in Unknown on line 0".
    vm.apply_ini_overrides(ini_overrides);
    // `session.auto_start=1` opens the session at request start (RINIT).
    if vm.ini.get_bool(b"session.auto_start") {
        let _ = vm.ho_session_start(Vec::new());
    }
    // `exit`/`die` is a clean termination (the exit code is surfaced, not a fatal);
    // any other `Err` is an uncaught fatal. A `Ok` carries the top-level return.
    let mut exit_code = None;
    // Disable error-handler routing for everything past the main run: the final
    // flush, the uncaught-fatal render, and shutdown destructors must render raw
    // and never call user code (Session 2 `final_flush` guard).
    let is_link_fatal = link_fatal.is_some();
    let run_result = match link_fatal {
        Some(e) => Err(e),
        None => vm.run(),
    };
    vm.final_flush = true;
    let (fatal, return_value) = match run_result {
        Ok(v) => (None, v),
        Err(PhpError::Exit(code)) => {
            exit_code = Some(code);
            (None, Zval::Null)
        }
        // An uncaught throwable routed to a `set_exception_handler` is handled
        // there (no fatal banner; PHP exits cleanly); otherwise it is the fatal.
        // A link-time fatal is Zend's compile-time kind: never a throwable, so
        // it bypasses the exception handler and renders a plain banner below.
        Err(e) if !is_link_fatal && vm.handle_uncaught_exception(&e) => (None, Zval::Null),
        Err(e) => (Some(e), Zval::Null),
    };
    // Flush any diagnostics still staged, then render the uncaught fatal at the
    // tail of `rendered` (mirrors `eval::run_with`).
    let line = vm.fatal_line;
    // `final_flush` is set, so routing is skipped and this never errs.
    let _ = vm.flush_diags(line);
    if let Some(err) = &fatal {
        // PHP flushes any active output buffers *before* printing the fatal banner,
        // so the script's buffered output precedes the "Fatal error:" block.
        vm.flush_all_output_buffers();
        if is_link_fatal {
            // Compile/link-time fatal (e.g. an enum implementing Serializable):
            // a plain banner with the declaration site — no throwable wrapping,
            // no stack trace.
            let file = String::from_utf8_lossy(&module.file);
            let block = format!("\nFatal error: {} in {} on line {}\n", err.message(), file, line);
            vm.rendered.extend_from_slice(block.as_bytes());
        } else {
            vm.render_fatal(err, line);
        }
    }
    // `register_shutdown_function` callbacks run after the main script (and any
    // uncaught-fatal banner), before object destructors (PHP shutdown sequence).
    vm.run_shutdown_functions();
    // End-of-script destructors (LIFO over the objects still tracked), run after
    // `main` returns — or after a fatal, on a cleared stack (OOP-3d). Their output
    // flows through `emit_str`, so it lands in `rendered` after the fatal block.
    vm.run_shutdown_destructors();
    // An active session auto-commits at request shutdown — AFTER shutdown
    // functions and destructors (both still see an active session and their
    // $_SESSION writes are persisted; oracle-verified order).
    vm.session_shutdown_flush();
    // Streams with attached write filters flush their final tail when the stream
    // is destroyed at request end (PHP filter close) — a script need not fclose.
    vm.finalize_filtered_streams();
    // Flush any output buffers the script left open (PHP flushes the buffer stack
    // at request shutdown). Done last, so shutdown-function and destructor output
    // produced while a buffer was active is captured then emitted in order.
    vm.flush_all_output_buffers();
    VmOutcome {
        stdout: vm.stdout,
        rendered: vm.rendered,
        diags: vm.diags,
        fatal,
        return_value,
        exit_code,
        headers: vm.response_headers,
        response_code: vm.response_code,
        response_reason: vm.response_reason,
        error_log: vm.error_log,
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
    /// A property-hook activation (step 50). While a `get`/`set` hook for
    /// `(objectId, prop)` runs, this guard is active so a `$this->prop` inside the
    /// hook falls through to the backing store instead of re-entering the hook.
    Hook,
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
    /// The module (translation unit) this frame executes in (step 57). Module-
    /// relative bytecode indices — `Op::Call`'s function index, `__FILE__`, the
    /// strict-types flag — resolve against *this* module, not a single global one,
    /// so a frame from an `include`d / `eval`'d unit reads its own metadata. With
    /// a single compiled unit every frame's module is the same `main` module.
    module: &'m Module,
    ip: usize,
    slots: Vec<Zval>,
    stack: Vec<Zval>,
    /// Variables created by NAME at run time (`$$x = v` with a name outside
    /// the function's static slot set). Read/written only by the variable
    /// variable ops; empty (no allocation) for ordinary frames.
    dyn_vars: HashMap<Vec<u8>, Zval>,
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
    /// When true, this frame's `Ret` value is replaced by `result !== null` —
    /// `isset($obj->hookedProp)` runs the `get` hook and tests its result for
    /// being set (step 50), distinct from `ret_bool`'s truthiness coercion.
    ret_isset: bool,
    /// When true, this frame's `Ret` value is converted to a string before being
    /// pushed — for a `__toString` call scheduled by [`Op::Stringify`].
    ret_stringify: bool,
    /// When true, this frame's `Ret` value is dereferenced before being pushed
    /// — a by-reference protocol getter (`&offsetGet`, monolog's LogRecord)
    /// consumed in value context (`count($o['k'])`).
    ret_deref: bool,
    /// Magic-accessor recursion-guard keys to remove from [`Vm::magic_guard`]
    /// when this frame returns (OOP-3b). Usually zero or one; a guard
    /// transferred across proxy forwarding (gh18038) appends here too.
    guard_release: Vec<(u32, MagicKind, Vec<u8>)>,
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
    /// Set on an `eval()` unit's top (`main`) frame to its *call site* — the file
    /// and line where `eval()` was invoked (step 57, Phase 1c-2c). A backtrace
    /// renders this frame's function as `eval` and presents code called from
    /// within it under the composite `<file>(<line>) : eval()'d code` file name,
    /// matching PHP. `None` for an ordinary frame.
    eval_origin: Option<(Box<[u8]>, Line)>,
    /// Set on the `__clone` frame that the `clone` operator pushes (PHP 8.3
    /// readonly-clone amendment): while it runs, `$this`'s readonly properties may
    /// each be re-initialised once. On this frame's `Ret` the per-object
    /// permission (`Object::readonly_clone_writable`) is revoked, so a *manual*
    /// `$o->__clone()` call (no permission) still fatals on a readonly write.
    clone_init: bool,
    /// True for a frame running `__destruct` via the GC sweep. Its body's
    /// per-statement `Op::Sweep`s no-op: objects the destructor displaces are
    /// collected by the OUTER sweep that resumes after this frame returns, so
    /// the dying receiver releases its handle id FIRST — Zend's LIFO reuse
    /// order (gh10168: the receiver's id is reused before its displacees').
    in_destructor: bool,
    /// The instance id of the closure this frame is running, if any. `static $x`
    /// inside a closure persists **per closure instance** (PHP binds the static to
    /// the Closure object, not the op-array): a fresh closure from the same literal
    /// gets fresh statics. Non-closure frames (`None`) use the program-global
    /// `Vm::statics`; closure frames key `Vm::closure_statics` by `(id, static_id)`.
    closure_id: Option<u32>,
    /// Set on an include/eval unit frame that shares the includer's variable
    /// scope (`drive_unit`'s `scope_bridge`): the caller frame index. In Zend
    /// both run on ONE symbol table, so `global $x` inside the included file
    /// rebinds the *shared* symbol — `bind_global_dyn` walks this chain and
    /// installs the alias in every bridged ancestor too (wp-settings.php /
    /// plugin.php require'd from inside WP_CLI's Runner::load_wordpress()).
    bridge_caller: Option<usize>,
}

impl<'m> Frame<'m> {
    fn new(func: &'m Func, module: &'m Module) -> Self {
        Frame {
            func,
            module,
            ip: 0,
            slots: vec![Zval::Undef; func.n_slots as usize],
            stack: Vec::new(),
            dyn_vars: HashMap::default(),
            this: None,
            class: None,
            static_class: None,
            ret_cell: None,
            ret_bool: false,
            ret_isset: false,
            ret_deref: false,
            ret_stringify: false,
            guard_release: Vec::new(),
            iters: Vec::new(),
            pending_throw: None,
            pending_transfer: None,
            gen_id: None,
            yield_from: None,
            argc: 0,
            extra_args: Vec::new(),
            init_props: false,
            eval_origin: None,
            clone_init: false,
            in_destructor: false,
            closure_id: None,
            bridge_caller: None,
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
    /// `foreach` over a plain (non-Traversable) object: fully *live* hash-cursor
    /// semantics (PHP iterates the property table by position) — each step
    /// recomputes the visible entries and yields the first not yet visited, so a
    /// property added in the body is reached, a removed one is skipped, and a
    /// hooked property's `get` runs at its step. `yielded` holds the display
    /// names already produced.
    ObjVals { obj: Zval, scope: Option<ClassId>, yielded: Vec<Box<[u8]>> },
    /// `foreach ($obj as &$v)` over a plain object: like [`IterState::ObjVals`]
    /// but each step binds `$v` *by reference* — to the property's storage cell,
    /// or to the cell a `&get` hook returns (a by-value hook is a fatal
    /// "Cannot create reference to property C::$p").
    ObjRefs { obj: Zval, scope: Option<ClassId>, yielded: Vec<Box<[u8]>> },
    /// `foreach` over a generator (GEN): no snapshot — each step reads the
    /// generator's current `(key, value)` and resumes it for the next. `primed`
    /// is false until the first `IterNext` (which starts the generator rather than
    /// advancing it), matching the tree-walker's read-then-resume order.
    Gen { rc: Rc<RefCell<GenState>>, primed: bool },
    /// `foreach` over an object implementing `Iterator` / `IteratorAggregate`
    /// (step 51): the protocol methods are driven by a re-entrant state machine in
    /// `IterNext` (see [`ObjStage`]). `pending` is the cell the last protocol call
    /// wrote into; `cur_val` holds `current()` while `key()` is fetched.
    Object { it: Zval, stage: ObjStage, pending: Option<Rc<RefCell<Zval>>>, cur_val: Option<Zval> },
}

/// How one step of a live plain-object `foreach` ([`IterState::ObjVals`] /
/// [`IterState::ObjRefs`]) reads or binds the next visible property (built by
/// [`Vm::object_iter_entries`]).
enum PropIterEntry {
    /// Read/bind the storage slot under this (possibly mangled) key.
    Slot { key: Box<[u8]> },
    /// Dispatch the property's get hook as resolved from class `view`'s
    /// flattened table (`view` differs from the object's class only when the
    /// running scope's own private declaration shadows a subclass one).
    Hook { name: Box<[u8]>, view: ClassId },
}

/// One live typed-reference source (see `Vm::typed_refs`): the weak handle
/// keys the entry, the rest formats the TypeError and drives the check.
#[derive(Clone)]
struct TypedRefSource {
    cell: std::rc::Weak<RefCell<Zval>>,
    /// The owning object: Zend deletes a property's type source when the
    /// object is freed (`typed_properties_094`), so enforcement stops once
    /// only the VM's `created` tracking handle (strong count ≤ 1) remains.
    obj: std::rc::Weak<RefCell<Object>>,
    /// Declaring class name, for the error wording (`C::$p`).
    class_name: Box<[u8]>,
    prop: Box<[u8]>,
    hint: TypeHint,
}

/// The step the object-iterator state machine is about to perform (step 51). A
/// `NeedX` stage *issues* the protocol call; the paired `AfterX` *consumes* its
/// captured result when `IterNext` re-runs.
#[derive(Clone, Copy, PartialEq)]
enum ObjStage {
    Start,
    AfterAggregate,
    NeedRewind,
    NeedValid,
    AfterValid,
    NeedCurrent,
    AfterCurrent,
    NeedKey,
    AfterKey,
    NeedNext,
}

/// Where a protocol method's return value is routed (see [`Vm::enter_object_method`]).
enum RetMode {
    /// Flow to the caller's operand stack (a value getter like `offsetGet`).
    Stack,
    /// Drop it (a `void` method like `offsetSet`/`rewind`/`next`).
    Discard,
    /// Write it into this cell — the re-entrant iterator reads it back.
    Capture(Rc<RefCell<Zval>>),
}

/// In-progress `yield from` delegation (GEN-3), parked on the generator frame so
/// it survives suspension. `Op::YieldFrom` re-enters itself across resumes,
/// advancing this cursor one step each time until the delegate is exhausted.
enum YieldFromState {
    /// Delegating to an array's elements (re-yielded verbatim).
    Array { entries: Vec<(Zval, Zval)>, pos: usize },
    /// Delegating to a sub-generator (its `send()`s forwarded, its return value
    /// becoming the `yield from` expression's value). `opaque` marks a
    /// generator reached through an IteratorAggregate: PHP still streams it,
    /// but the `yield from` expression reads NULL, not its return value.
    Gen { rc: Rc<RefCell<GenState>>, opaque: bool },
}

/// What a Traversable `yield from` delegate resolves to (see `Op::YieldFrom`):
/// an inner Generator (delegated live) or the drained (key, value) entries of
/// an Iterator protocol run.
enum TraversableSource {
    Gen(Rc<RefCell<GenState>>),
    Entries(Vec<(Zval, Zval)>),
}

/// `unserialize()` slot registry: pre-order value numbering mirroring
/// `serialize()` (`R:` consumes no number; everything else, `r:` included,
/// does). `targets` holds the slots some `R:` aliases (pre-collected), `cells`
/// the shared reference cells built for them, `objs` each object slot's handle
/// for `r:` repeats.
#[derive(Default)]
struct UnserCtx {
    count: i64,
    targets: std::collections::HashSet<i64>,
    objs: std::collections::HashMap<i64, Zval>,
    cells: std::collections::HashMap<i64, Rc<RefCell<Zval>>>,
}

/// The virtual machine: the module under execution plus the explicit call stack.
/// PHP function calls grow `frames` rather than the Rust stack, so deep PHP
/// recursion cannot overflow the host stack, and a frame is suspendable.
struct Vm<'m> {
    module: &'m Module,
    /// The **global class table** (step 57, Phase 1c): every loaded module's
    /// classes flattened into one id space, so `ClassId` is global and an object's
    /// `class_id` resolves the same regardless of which module is executing. Built
    /// from `main`'s classes (ids identical) and grown as `eval`/`include` units
    /// link. All class lookups (`is_instance_of`, method/const/prop resolution) go
    /// through this, not `self.classes`.
    classes: Vec<&'m CompiledClass>,
    /// Global case-insensitive class-name → [`ClassId`] index over `classes`.
    class_index: HashMap<Vec<u8>, ClassId>,
    /// Defining [`Module`] of each class in `classes`, parallel to it (step 57,
    /// Phase 1c-2b). For a class linked by an `eval`/`include` unit this is that
    /// unit's leaked module, so a method frame entered for the class resolves its
    /// own bytecode indices (`Op::Call` funcidx, `MakeClosure`, `__FILE__`) in the
    /// module that compiled it, not the currently-executing one. For `main`'s own
    /// classes it is `main` (≡ `self.module` while main runs).
    class_module: Vec<&'m Module>,
    /// Registry of every leaked [`Module`] the VM has seen, indexed by a stable
    /// `module_id`. A [`Closure`] value records the id of its defining module here
    /// so it stays callable after control leaves that module — a closure created
    /// in an `eval`/`include` unit and invoked later (e.g. Composer's
    /// `autoload_static` initializer run via `call_user_func`) resolves its
    /// bytecode against the module that compiled it, not the running one. Id 0 is
    /// `main`; populated lazily by [`Self::module_id`].
    modules: Vec<&'m Module>,
    /// The caller's lowered HIR (`main`), retained so an `eval()` unit can be
    /// compiled *against the image* (step 57, Phase 1c-2c): its class/function
    /// tables seed the eval's lowering, letting an eval class `extend`/`implement`
    /// a caller's user class and flatten its inherited layout correctly. `None`
    /// when the VM was started from an already-compiled module (e.g. unit tests
    /// via [`run_module`]), in which case eval falls back to standalone lowering.
    main_hir: Option<&'m Program>,
    /// The accumulating HIR class image used to seed every `eval`/`include` unit
    /// (step 57, Phase 3). Starts as `main`'s classes and grows by each loaded
    /// unit's *new* classes, so a later unit can `extend`/`implement` a class an
    /// earlier one (or an autoloaded one) declared. Ids stay aligned with the
    /// global compiled table. Empty (and unused) when no HIR is retained.
    seed_classes: Vec<std::rc::Rc<crate::hir::ClassDecl>>,
    /// The accumulating trait image (step 21, trait analogue of `seed_classes`):
    /// every loaded unit's declared traits, keyed by bare lowercase name, so a
    /// later (e.g. autoloaded) unit's `use T` resolves a trait an earlier unit
    /// declared. Traits never enter the class table, hence a separate image.
    seed_traits: Vec<(Vec<u8>, crate::hir::LoweredTrait)>,
    /// The static-cell id high-water mark carried into a seeded unit's lowering, so
    /// its `static $x` cells get ids past every already-loaded unit's (Phase 3).
    seed_static: usize,
    /// The accumulating global-variable name registry, canonical slot-numbering for
    /// every `$GLOBALS['x']` / `global $x` access across units (step 57). Starts as
    /// `main`'s top-level variable names (`Program.slots`) and grows by each loaded
    /// unit's *new* global names, so a seeded unit numbers its global slots to agree
    /// with `main`'s bottom (global) frame — which all `DimBase::Global` ops index.
    /// `frames[0].slots` is grown in step with this so a new global slot addresses a
    /// real cell. Empty (and unused) when no HIR is retained.
    seed_globals: Vec<Box<[u8]>>,
    /// User functions declared by a linked `eval`/`include` unit (step 57, Phase
    /// 1c-2): lowercased name → (defining module, index into its `functions`), so
    /// they are callable by name after the unit returns. The defining module is
    /// kept so the function's frame resolves its own bytecode indices.
    linked_functions: HashMap<Vec<u8>, (&'m Module, usize)>,
    /// Resolved (canonical) paths already loaded by `include_once`/`require_once`
    /// (step 57, Phase 2), so a repeat `_once` of the same file no-ops and returns
    /// `true` without re-running it.
    included_files: HashSet<Vec<u8>>,
    /// Hash chain of every unit-load event so far (main identity, then each
    /// include's path+mtime+size and each eval's source): part of the unit-cache
    /// fingerprint ([`Vm::unit_fp`]) — two VMs with equal chains loaded the same
    /// code in the same order, so seeded lowering of the next unit is replayable.
    unit_chain_fp: u64,
    /// Autoload callbacks registered by `spl_autoload_register` (step 57, Phase 3),
    /// in registration order. Consulted when a class name fails to resolve (a `new`
    /// / static reference / `class_exists` of an undeclared class), each given the
    /// requested name; one that loads the class (typically via `require`) ends the
    /// search.
    autoloaders: Vec<Zval>,
    /// Class names (lowercased) currently mid-autoload, to break recursion if an
    /// autoloader (transitively) references the same name (step 57, Phase 3).
    autoloading: HashSet<Vec<u8>>,
    /// Builtin registry, injected by the caller (php-runtime can't build a
    /// populated one — that lives in php-builtins, which depends on php-runtime).
    registry: &'m Registry,
    stdout: Vec<u8>,
    /// CLI-faithful output stream built alongside `stdout` (E1): diagnostics are
    /// flushed into it (stamped with the current line) at each output point, and an
    /// uncaught fatal is rendered at the tail. Mirrors `eval::Evaluator::rendered`.
    rendered: Vec<u8>,
    /// Output-buffering stack (`ob_start` family). When non-empty, `echo`/`print`
    /// and output-producing builtins append to the topmost buffer instead of
    /// `stdout`/`rendered`; the buffer surfaces via `ob_get_contents`/`ob_get_clean`
    /// and is written to the underlying sink (the next buffer down, or the real
    /// streams) on flush. Empty in the common case, so a script that never calls
    /// `ob_start` is unaffected.
    ob_stack: Vec<OutputBuffer>,
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
    /// `@` error-suppression nesting depth (step 48). The silencing itself
    /// happens through `error_level` (Zend's BEGIN_SILENCE masks
    /// EG(error_reporting) with E_FATAL_ERRORS = 4437); the depth only tracks
    /// nesting for the fiber save/restore.
    suppress_depth: u32,
    /// Saved `diags` lengths, one per active `@` (innermost last): `Op::SuppressEnd`
    /// truncates back to its mark, dropping the diagnostics raised under it. An
    /// unwind past an active `@` truncates to the outermost mark and resets both.
    suppress_marks: Vec<usize>,
    /// Saved `error_level` values, one per active `@` (Zend's BEGIN_SILENCE
    /// result temp). `Op::SuppressEnd` — and the unwind past an abandoned `@` —
    /// restores conditionally, END_SILENCE style: only when the current level
    /// is still fatal-only and the saved one was not (an `error_reporting($x)`
    /// call inside the region survives it — bug27731/bug33771).
    silence_saved: Vec<i64>,
    /// The data superglobals (`$_SERVER`, …), indexed by
    /// [`crate::bytecode::SUPERGLOBAL_NAMES`]. Stored VM-wide (not per-frame) so a
    /// superglobal resolves by name from any unit/frame — an included file reads
    /// the same `$_SERVER` as the main script. Unseeded entries are `Undef`.
    superglobals: [Zval; 8],
    /// Compiled-regex cache, keyed by the raw PHP pattern (delimiters + flags),
    /// mirroring PCRE's per-request pattern cache. Composer/symfony call e.g.
    /// `preg_match('/.{1,10000}/u', …)` in a loop; without this each call would
    /// rebuild the (large, Unicode) NFA from scratch. `None` caches a pattern that
    /// failed to compile so it isn't retried.
    preg_cache: HashMap<Vec<u8>, Option<Rc<crate::preg::Engine>>>,
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
    /// Per-closure-instance storage for `static $x` declared inside a closure body,
    /// keyed by `(closure instance id, program-global static id)`. PHP binds a
    /// closure's static to the Closure *object*, so each fresh closure from the same
    /// literal starts its statics over (a shared `Vm::statics` slot would wrongly
    /// persist across instances). Grows only for closures that declare statics.
    closure_statics: HashMap<(u32, u32), Rc<RefCell<Zval>>>,
    /// Active magic-accessor guards (object id, kind, property) — a magic method
    /// is not re-entered for the same access while it is running (OOP-3b).
    magic_guard: HashSet<(u32, MagicKind, Vec<u8>)>,
    /// Live reference cells that alias a *typed* property's storage (PHP's
    /// typed-reference sources, narrowed to the cells phpr hands out via
    /// `&$o->typedProp` / by-ref foreach / a `&get` hook returning a typed
    /// backing): a write through such a cell keeps enforcing the property's
    /// declared type. Dead cells are pruned on registration; empty in the
    /// common program, so the write-path check is a cheap `is_empty`.
    typed_refs: Vec<TypedRefSource>,
    /// A strong handle to every object created via `new`, keyed by object id
    /// (OOP-3d). Ids are monotonic, so key order IS creation order — iteration
    /// replaces the old Vec's positional order, and the sweep's per-object
    /// lookup/removal is O(log n) instead of a linear scan of every tracked
    /// object (which went quadratic once cyclic garbage accumulated). The extra
    /// ref lets the destruction sweep detect unreachability
    /// (`Rc::strong_count == 1` ⇒ only this tracking ref remains); entries are
    /// removed as they are destructed or at shutdown.
    created: BTreeMap<u32, Rc<RefCell<Object>>>,
    /// Object handles whose `__destruct` has already run, guarding double calls.
    destructed: HashSet<u32>,
    /// Possible-roots buffer for the destruction sweep: objects that have just
    /// lost a reference and so *might* now be unreachable (mirrors Zend's
    /// `gc_possible_root` buffer). [`Op::Sweep`] re-examines only these instead
    /// of all of `created`, turning the per-statement O(created) scan into
    /// O(candidates) — the fix for the O(n²) blow-up on large cyclic graphs.
    /// Holds one strong clone per object id (the map key dedupes), so a
    /// buffered candidate's `Rc::strong_count` is inflated by exactly 1.
    /// Drained at each sweep (objects still referenced get re-noted the next
    /// time they lose a reference), keeping the buffer small.
    gc_roots: HashMap<u32, Rc<RefCell<Object>>>,
    /// Max-heap over the ids in `gc_roots`, pushed as each id enters the map.
    /// The sweep pops it to visit candidates newest-first instead of
    /// re-filtering the whole buffer after every freed object (which went
    /// quadratic once thousands of never-collectable cyclic candidates piled
    /// up). Entries whose id is no longer in the map are stale and skipped.
    gc_queue: BinaryHeap<u32>,
    /// Possible roots of *cyclic* garbage (mirrors Zend's root buffer feeding
    /// `gc_collect_cycles`): a candidate the sweep popped that was not
    /// collectable lost a reference but still has holders — if those holders
    /// form a dead cycle its refcount never falls to the collectable point, so
    /// it is demoted here instead of forgotten. When the set reaches
    /// [`GC_CYCLE_THRESHOLD`] (or `gc_collect_cycles()` forces it) the cycle
    /// collector runs a trial-deletion pass over these ids. Ids only —
    /// `created` keeps the objects alive; stale ids are filtered at collection.
    gc_cycle_roots: HashSet<u32>,
    /// Objects a LIGHT (in-body) sweep demoted to `gc_cycle_roots` since the
    /// last MAIN sweep. A temp consumed off the operand stack mid-statement is
    /// not gc_note'd, so its death is only observable by re-checking the
    /// refcount; the enclosing global statement's main sweep re-seeds exactly
    /// these ids once — the same set the pre-eager-sweep buffer would have
    /// held for that statement (no asymptotic cost change).
    gc_light_demoted: Vec<u32>,
    /// Callbacks registered with `register_shutdown_function`, each with its bound
    /// arguments, run in registration order at script end — after the main run (and
    /// any uncaught-fatal banner), before object destructors.
    shutdown_fns: Vec<(Zval, Vec<Zval>)>,
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
    /// The `ArrayAccess` interface id, so `$o[$k]` read/write/isset/unset on an
    /// implementing object dispatches `offsetGet`/`offsetSet`/`offsetExists`/
    /// `offsetUnset` instead of the array path (step 51).
    arrayaccess_id: Option<ClassId>,
    /// The `Iterator` / `IteratorAggregate` interface ids, so `foreach` over an
    /// implementing object drives the iterator protocol (step 51).
    iterator_id: Option<ClassId>,
    iteratoraggregate_id: Option<ClassId>,
    /// The `Countable` interface id, so `count($obj)`/`sizeof($obj)` dispatches
    /// the user `count()` method instead of erroring (step 56).
    countable_id: Option<ClassId>,
    /// The `Stringable` interface id, so `is_instance_of` can apply the
    /// auto-implementation (a class with `__toString` is `Stringable`) without a
    /// `class_index` lookup (step 57).
    stringable_id: Option<ClassId>,
    /// The `JsonSerializable` interface id, so `json_encode()` calls a value's
    /// `jsonSerialize()` method instead of encoding its properties (step 56c).
    jsonserializable_id: Option<ClassId>,
    /// The error code of the most recent `json_encode()`/`json_decode()` call,
    /// reported by `json_last_error()`/`json_last_error_msg()` (0 = JSON_ERROR_NONE,
    /// 4 = SYNTAX, 11 = NON_BACKED_ENUM).
    json_last_error: i64,
    /// Whether any output has reached the real sink (CLI stdout) — the point PHP
    /// considers HTTP headers "sent". Set on the first non-empty *unbuffered*
    /// write; `output_start` records the `(file, line)` for the
    /// "headers already sent by (output started at …)" warning.
    output_started: bool,
    output_start: Option<(Vec<u8>, Line)>,
    /// The mutable INI table (`ini_get`/`ini_set`/…); the session module reads
    /// its `session.*` configuration from here. See [`ini::IniTable`].
    ini: ini::IniTable,
    /// ext/session runtime state (`session_start` → `$_SESSION` → commit).
    session: session::SessionState,
    /// `http_response_code()` — `None` until explicitly set (CLI reports `false`
    /// for an unset code).
    response_code: Option<i64>,
    /// Whether this run serves a web request (a [`php_types::sapi::WebRequest`]
    /// is installed on the thread): the header family becomes stateful, the
    /// diagnostics render in `html_errors` form, and the web superglobals are
    /// seeded. The cli-server SAPI buffers the whole response, so headers are
    /// never "sent" mid-script (oracle-pinned).
    web: bool,
    /// Web response headers as full `Name: value` lines in emission order
    /// (`X-Powered-By: PHP/8.5.7` pre-seeded, like PHP with expose_php on).
    response_headers: Vec<Vec<u8>>,
    /// Custom reason phrase from a raw `header("HTTP/1.1 404 Custom")`.
    response_reason: Option<Vec<u8>>,
    /// Web `log_errors` sink: one entry per rendered diagnostic (host adds
    /// timestamps and writes stderr).
    error_log: Vec<Vec<u8>>,
    /// `ignore_user_abort()` flag; CLI has no client connection so it is purely a
    /// stored value (default `0`, returned as the "previous" on each set).
    user_abort_ignored: bool,
    /// Userland stream wrappers (`stream_wrapper_register`): lower-cased `scheme`
    /// → the handler class name (resolved to a class at `fopen` time, so a
    /// late-defined class still works). `fopen("scheme://…")` instantiates it and
    /// drives its `stream_*` methods.
    stream_wrappers: std::collections::HashMap<Vec<u8>, Vec<u8>>,
    /// Streams that had a filter attached (`stream_filter_append`), so shutdown
    /// can finish their write chains (PHP flushes filters when the stream is
    /// destroyed at request end — a script need not fclose).
    filtered_streams: Vec<Rc<RefCell<Resource>>>,
    /// Object addresses whose `jsonSerialize()` is currently on the encode stack —
    /// tracked ACROSS nested `json_encode()` calls (unlike the per-call `visiting`
    /// path). A nested `json_encode()` of such an object is JSON_ERROR_RECURSION;
    /// the return value of a `jsonSerialize()` that is the same object encodes by
    /// plain properties (json_encode_recursion_01/02).
    json_active: Vec<usize>,
    /// Interned enum case singletons, keyed by (enum class id, case index), so
    /// `E::Case === E::Case` (Session A). Materialised lazily on first `E::Case`.
    enum_cache: HashMap<(ClassId, u32), Rc<RefCell<Object>>>,
    /// User-defined constants from `define()` (B3), mirror of
    /// `eval::Evaluator::constants`. Read by [`Op::ConstFetch`] and `constant()`;
    /// engine constants (`PHP_INT_MAX`, …) are folded at lowering and not stored here.
    constants: HashMap<Vec<u8>, Zval>,
    /// Persistent mbregex state (step 43), mirror of `eval::Evaluator::mb_regex`:
    /// the global `mb_regex_encoding`/`mb_regex_set_options` and the `mb_ereg_search`
    /// cursor. Survives across `mb_*` calls for the whole run.
    mb_regex: crate::mbregex::MbRegexState,
    /// `strtok` tokenizer state: the string handed to `strtok($str, $tok)` and the
    /// current byte cursor. A one-argument `strtok($tok)` resumes from here.
    /// `None` before any `strtok` call (or after exhaustion resets it).
    strtok: Option<(Vec<u8>, usize)>,
    /// ext/pcntl handler table: signo → the PHP callable installed by
    /// `pcntl_signal` (or `Zval::Long(0/1)` for SIG_DFL/SIG_IGN). Delivery is
    /// two-phase like the engine's: the C-level catcher only marks the signal
    /// pending ([`PENDING_SIGNALS`]); `pcntl_signal_dispatch` (or any host
    /// builtin return, when `pcntl_async_signals(true)`) runs the PHP handler.
    signal_handlers: HashMap<i32, Zval>,
    /// `pcntl_async_signals` flag: dispatch pending signals at host-builtin
    /// boundaries instead of waiting for an explicit `pcntl_signal_dispatch`.
    async_signals: bool,
    /// The throwable synthesized for an *uncaught* error, stashed by `unwind` while
    /// the faulting frames are still live so `render_fatal` can show the real stack
    /// trace (an engine error reaches the renderer after its frames are popped).
    /// Cleared when an exception is caught; the deepest capture is kept.
    uncaught_throwable: Option<Zval>,
    /// The script `main` frame, parked when its `Ret` pops it: Zend tears the
    /// global variables down only at request shutdown, AFTER the
    /// `register_shutdown_function` callbacks ran — those callbacks re-install
    /// this frame as their bottom frame so `global $x` / `$GLOBALS` still see
    /// the script's values (`run_shutdown_functions`).
    retired_main: Option<Frame<'m>>,
    /// Pending initializer closures of *uninitialized* lazy objects (PHP 8.4),
    /// keyed by object id. The `Object.lazy` marker says an object is lazy; this
    /// holds the ghost initializer / proxy factory until the object is realized
    /// (the entry is removed on initialization). Kept off `Object` so it can carry
    /// a `Zval` (which is not `PartialEq`).
    lazy_init: HashMap<u32, Zval>,
    /// Per-property laziness of *uninitialized* lazy objects (PHP 8.4
    /// `skipLazyInitialization` / `setRawValueWithoutLazyInitialization`), keyed
    /// by object id: the declared instance properties (in declaration order) that
    /// are *still lazy*, i.e. an access to one of them triggers initialization.
    /// Seeded with every eligible property at alloc; a skip/raw-set removes one,
    /// and when the set empties the object realizes *without* running its
    /// initializer (every property is already materialized). Absent for a
    /// non-lazy object.
    lazy_props: HashMap<u32, Vec<Box<[u8]>>>,
    /// Scratch handoff from the `var_dump` arg-walk to `run_value_builtin`: each
    /// debuggable object's `__debugInfo()` result, keyed by object id. Populated
    /// only while dispatching a `var_dump` (and `mem::take`n right after), empty
    /// otherwise — never observed across ops.
    // (std map: the type crosses into php-builtins' `Ctx`, and the path is cold.)
    var_dump_debug: std::collections::HashMap<u32, Zval>,
    /// Scratch handoff from a string-coercing builtin's arg-walk to the builtin
    /// dispatch: each Stringable object argument's `__toString()` result, keyed
    /// by object id. Populated only while dispatching an unconditionally
    /// string-coercing builtin (`natsort`/`natcasesort`/…) and `mem::take`n right
    /// after; empty otherwise. Lets a pure builtin honor `__toString` without
    /// re-entering the VM (see `Ctx::stringify` / `compute_stringify`).
    // (std map: crosses into php-builtins like `var_dump_debug`; cold path.)
    stringify_args: std::collections::HashMap<u32, php_types::ZStr>,
    /// Memoized `__reflect_method_info` descriptors, keyed by (resolved class
    /// id, lowercased method name). Sound because a declared class's method
    /// table and ancestry never change afterwards (no runkit); later class
    /// declarations cannot alter an existing cid's resolution. PHPUnit builds
    /// thousands of `ReflectionMethod`s over the same (class, method) pairs
    /// during suite construction — this turns each repeat into an Rc clone.
    reflect_method_info_cache: std::collections::HashMap<(ClassId, Vec<u8>), Zval>,
    /// Per-lazy-object option flags (PHP 8.4 `ReflectionClass::newLazy*` /
    /// `resetAsLazy*` `$options`): SKIP_INITIALIZATION_ON_SERIALIZE (8) and
    /// SKIP_DESTRUCTOR (16, consumed at reset time). Keyed by object id.
    lazy_options: HashMap<u32, u32>,
    /// The instance a `ReflectionObject` was constructed for, keyed by the
    /// ReflectionObject's own id. Zend keeps this pointer internally (invisible to
    /// var_dump); the prelude cannot store it as a visible property, so it lives
    /// here. Read by `ReflectionObject::__toString` to list dynamic properties.
    reflect_object_bound: HashMap<u32, Zval>,
    /// Object ids whose initializer/factory is *currently running* (PHP 8.4):
    /// re-entering `resetAsLazy*` on such an object is an error ("Can not reset an
    /// object while it is being initialized"). Populated by `realize_lazy` around
    /// the initializer call and cleared when it returns (even on throw).
    lazy_initializing: HashSet<u32>,
    /// Open zip archives (`ZipArchive`, ext/zip subset): handle id → parsed
    /// archive, backing the `__zip_*` host builtins the prelude class delegates
    /// to. An id is allocated by `__zip_open` and released by `__zip_close`.
    zips: HashMap<u32, ::zip::ZipArchive<std::fs::File>>,
    /// Write-mode zip handles (`ZipArchive::open` with CREATE on a missing
    /// file → prelude `__zip_writer_*`): the archive is streamed out through
    /// the zip crate's writer and finalized on close (WP-17, privacy export).
    zip_writers: HashMap<u32, ::zip::ZipWriter<std::fs::File>>,
    /// Next zip handle id (monotonic; ids are never reused within a run).
    next_zip: u32,
    /// Open PDO connections (ext/pdo_sqlite subset): handle id → rusqlite
    /// connection, backing the `__pdo_*` host builtins the prelude `PDO` class
    /// delegates to. An id is allocated by `__pdo_open` and released by
    /// `__pdo_close` (the prelude `PDO::__destruct`).
    pdo_conns: HashMap<u32, rusqlite::Connection>,
    /// Next PDO handle id (monotonic; ids are never reused within a run).
    next_pdo: u32,
    /// Open mysqli connections (ext/mysqli): handle id → wire connection +
    /// client-side state, backing the `__mysqli_*` host builtins the prelude
    /// `mysqli` class delegates to (see vm/mysqli.rs).
    mysqli_conns: HashMap<u32, mysqli::MysqliConn>,
    /// Server-side prepared statements (`mysqli_stmt`): handle id → statement.
    mysqli_stmts: HashMap<u32, mysqli::MysqliStmt>,
    /// Open gd images (ext/gd): handle id → system-libgd image, backing the
    /// `__gd_*` host builtins the prelude `GdImage` class delegates to
    /// (see vm/gd.rs); freed by GdImage::__destruct via `__gd_destroy`.
    gd_images: HashMap<u32, php_types::gdio::GdImg>,
    /// Next gd handle id.
    next_gd: u32,
    /// `__xslt_*` host builtins the prelude `XSLTProcessor` class delegates to
    /// (see vm/xslt.rs); freed by XSLTProcessor::__destruct via `__xslt_free`.
    xslt_sheets: HashMap<u32, php_types::xsltio::XsltSheet>,
    /// Next xslt stylesheet handle id.
    next_xslt: u32,
    /// Next mysqli handle id (shared by connections and statements).
    next_mysqli: u32,
    /// Per-stream chunk size set by `stream_set_chunk_size` (resource id → size).
    /// phpr's I/O is unbuffered so the value has no read-path effect; it is kept
    /// only so the builtin can return the previous size (default 8192), as PHP does.
    stream_chunk_sizes: HashMap<u32, i64>,
    /// `class_alias` entries as `(alias, original)` names, threaded into every
    /// later unit's lowering so `extends AliasName` resolves to the original
    /// class decl (index-only: no clone, mangling/identity preserved).
    seed_aliases: Vec<(Vec<u8>, Vec<u8>)>,
    /// The process umask as `umask()` reports it. phpr never changes the real
    /// process umask (no unsafe/libc); the shadow value starts at the
    /// conventional 022 and get/set semantics match PHP (set returns previous).
    /// Consumers (Composer's BinaryInstaller) only use it to compute chmod
    /// masks like `0777 & ~umask()`, which the real `chmod` then applies.
    umask: i64,
    /// ext/dom documents (`DOMDocument` handles): doc id → arena tree, backing
    /// the `__dom_*` host builtins the prelude DOM classes delegate to.
    dom_docs: HashMap<u32, dom::DomDoc>,
    /// Next DOM document id (monotonic).
    next_dom: u32,
    /// ext/libxml error surface: `libxml_use_internal_errors` flag and the
    /// recorded parse errors `libxml_get_errors` reports.
    libxml_internal: bool,
    libxml_errors: Vec<dom::LxErr>,
}

impl<'m> Vm<'m> {
    /// Allocate an object handle id: REUSE the most recently freed handle
    /// first (Zend's objects-store free list, LIFO), else mint a fresh one.
    /// Per-id VM bookkeeping from the id's previous life is cleared here —
    /// a reused id must not inherit a "destructor already ran" mark or a
    /// dead lazy-object state.
    fn next_id(&mut self) -> u32 {
        if let Some(id) = php_types::take_freed_object_id() {
            self.destructed.remove(&id);
            self.lazy_init.remove(&id);
            self.lazy_props.remove(&id);
            self.lazy_options.remove(&id);
            self.lazy_initializing.remove(&id);
            self.gc_roots.remove(&id);
            return id;
        }
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Record a value that is about to be dropped as a possible GC root: if it
    /// (transitively) still holds tracked objects whose refcount is about to
    /// fall, push them onto `gc_roots` so the next [`Op::Sweep`] re-examines
    /// them (mirrors Zend's `gc_possible_root`). Over-noting is safe — the sweep
    /// re-checks `Rc::strong_count` — whereas under-noting delays a destructor,
    /// so this errs generous. A `Ref`/`Array`/`Closure` is descended only when
    /// this is its last holder (`strong_count == 1`), i.e. dropping it really
    /// releases its contents. An `Object`'s own properties are *not* descended
    /// here: they only lose their references when the object is actually freed,
    /// which the sweep handles via its cascade.
    fn gc_note(&mut self, v: &Zval) {
        match v {
            Zval::Object(rc) => {
                let id = rc.borrow().id;
                if self.destructed.contains(&id) {
                    return;
                }
                if let std::collections::hash_map::Entry::Vacant(e) = self.gc_roots.entry(id) {
                    e.insert(Rc::clone(rc));
                    self.gc_queue.push(id);
                }
            }
            Zval::Ref(r) => {
                if Rc::strong_count(r) == 1 {
                    let inner = r.borrow();
                    self.gc_note(&inner);
                }
            }
            Zval::Array(a) => {
                if Rc::strong_count(a) == 1 {
                    for (_, ev) in a.iter() {
                        self.gc_note(ev);
                    }
                }
            }
            Zval::Closure(cl) => {
                // Dropping the last handle to a closure frees its captured values
                // and bound `$this` — which may hold the last reference to an
                // object (e.g. a closure created in a method outliving its owner).
                if Rc::strong_count(cl) == 1 {
                    for (_, cv) in &cl.captures {
                        self.gc_note(cv);
                    }
                    if let Some(bt) = &cl.bound_this {
                        self.gc_note(bt);
                    }
                }
            }
            _ => {}
        }
    }

    fn gc_sweep(&mut self, top: usize, ip: usize, main: bool) -> Result<(), PhpError> {
        self.gc_sweep_impl(Some((top, ip)), main)
    }

    /// The sweep body. `resume = Some((top, ip))` is the statement-level mode:
    /// a found destructor is *scheduled* (frame pushed, `ip` rewound so the
    /// `Sweep` op re-runs after it returns). `None` drives each destructor to
    /// completion synchronously — the reset-as-lazy path, where PHP runs the
    /// displaced contents' destructors inside the reset itself. `main` (global
    /// statement boundaries, and every synchronous caller) first re-seeds what
    /// LIGHT sweeps demoted, so unhooked mid-statement temp deaths are caught.
    fn gc_sweep_impl(&mut self, resume: Option<(usize, usize)>, main: bool) -> Result<(), PhpError> {
        if main && !self.gc_light_demoted.is_empty() {
            for id in std::mem::take(&mut self.gc_light_demoted) {
                if let Some(o) = self.created.get(&id) {
                    if let std::collections::hash_map::Entry::Vacant(e) = self.gc_roots.entry(id) {
                        e.insert(Rc::clone(o));
                        self.gc_queue.push(id);
                        self.gc_cycle_roots.remove(&id);
                    }
                }
            }
        }
        log::trace!(
            target: "phpr::gc",
            "sweep: {} tracked / {} candidate objects",
            self.created.len(),
            self.gc_roots.len()
        );
        let verify = gc_verify_enabled();
        loop {
            // Candidate selection: a possible root is collectable when its only
            // strong references are `created` (1) and its own buffer clone (1).
            // Pop the max-heap so candidates are visited newest-first (highest
            // id), matching the legacy full-filter max-id order. A popped id no
            // longer in the buffer is stale — skip it. A popped candidate whose
            // strong_count is still > 2 is not collectable *now*: demote it to
            // the cycle-roots buffer instead of rescanning it after every freed
            // object — if it later loses another reference it is re-noted (and
            // re-queued) like any candidate, and if its remaining holders are a
            // dead cycle the cycle collector picks it up from there.
            let cand_id = loop {
                let Some(id) = self.gc_queue.pop() else { break None };
                match self.gc_roots.get(&id) {
                    None => continue,
                    Some(o) if Rc::strong_count(o) == 2 => break Some(id),
                    Some(_) => {
                        self.gc_roots.remove(&id);
                        self.gc_cycle_roots.insert(id);
                        // A light sweep's demotion must be re-checked by the
                        // enclosing main sweep (unhooked temp deaths).
                        if !main {
                            self.gc_light_demoted.push(id);
                        }
                    }
                }
            };

            let chosen_id = if verify {
                // Authoritative full scan, cross-checking buffer completeness.
                let full_id = {
                    let roots = &self.gc_roots;
                    self.created
                        .iter()
                        .filter(|(id, o)| {
                            let extra = roots.contains_key(id) as usize;
                            Rc::strong_count(o) - extra == 1
                        })
                        .map(|(id, _)| *id)
                        .max()
                };
                if full_id != cand_id {
                    // `full_id` is collectable but the candidate buffer did not
                    // surface it: a reference-drop site is missing a `gc_note`.
                    if let Some(fid) = full_id {
                        let name = self
                            .created
                            .get(&fid)
                            .map(|o| self.classes[o.borrow().class_id as usize].name.clone())
                            .unwrap_or_default();
                        log::error!(
                            target: "phpr::gc",
                            "VERIFY under-note: collectable {}#{} absent from candidates (cand={:?})",
                            String::from_utf8_lossy(&name), fid, cand_id
                        );
                        eprintln!(
                            "GC_VERIFY under-note: {}#{}",
                            String::from_utf8_lossy(&name), fid
                        );
                    }
                }
                full_id
            } else {
                cand_id
            };

            let Some(id) = chosen_id else {
                // Nothing more collectable by refcount. If enough possible
                // cycle roots piled up, run the cycle collector (mirrors Zend's
                // automatic gc_collect_cycles at root-buffer pressure): freed
                // cycle members re-enter the buffers, so take another pass.
                if self.gc_cycle_roots.len() >= Self::GC_CYCLE_THRESHOLD
                    && self.collect_cycles()? > 0
                {
                    continue;
                }
                // Drop the candidate buffer.
                self.gc_roots.clear();
                self.gc_queue.clear();
                break;
            };

            // Take ownership of the chosen object out of `created` and the buffer.
            // A candidate not in `created` is an object the GC never tracked (e.g.
            // an interned enum-case singleton that was noted on a value drop): its
            // `strong_count == 2` is a false positive, so just drop it from the
            // buffer and move on.
            let Some(o) = self.created.remove(&id) else {
                self.gc_unnote(id);
                continue;
            };
            self.gc_unnote(id);
            let cid = o.borrow().class_id as usize;

            if self.destructed.contains(&id) {
                // Already destructed: it just drops here. Note what it held so
                // those objects' falling refcounts are reconsidered next pass.
                self.gc_cascade(&o);
                continue;
            }
            // A lazy *wrapper* (an uninitialized ghost, or a proxy whether or not
            // it is initialized) does not run its own `__destruct` (PHP 8.4): an
            // uninitialized object was never constructed, and a proxy forwards to
            // its real instance, which is a separate tracked object that runs its
            // own destructor. Treat the wrapper as destructor-less; the cascade
            // releases the real instance so its `__destruct` fires next pass.
            if o.borrow().lazy.is_some() {
                // The initializer/factory closure dies with the wrapper — note
                // what it captured (it may hold the last ref to an object).
                if let Some(init) = self.lazy_init.remove(&id) {
                    self.gc_note(&init);
                }
                self.lazy_props.remove(&id);
                self.gc_cascade(&o);
                continue;
            }
            if let Some((defc, midx)) = resolve_method_runtime(&self.classes, cid, b"__destruct") {
                log::debug!(target: "phpr::gc", "destruct: {}#{}", String::from_utf8_lossy(&self.classes[cid].name), id);
                self.destructed.insert(id);
                let callee = &self.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee, self.class_mod(defc));
                frame.this = Some(Zval::Object(Rc::clone(&o)));
                frame.class = Some(defc);
                frame.static_class = Some(cid);
                frame.in_destructor = true;
                // Discard the destructor's return (don't disturb the caller's
                // operand stack).
                frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
                match resume {
                    Some((top, ip)) => {
                        self.frames[top].ip = ip; // re-run Sweep after it returns
                        self.frames.push(frame);
                        // `o` stays alive via `frame.this`; it is freed (and its
                        // contents cascaded by the `Ret` hook) when that frame
                        // returns.
                        break;
                    }
                    None => {
                        // Synchronous mode: run the destructor to completion and
                        // keep sweeping (the `Ret` hook cascades as usual). The
                        // return must surface to the bounded drive — a `ret_cell`
                        // would swallow the baseline return and the drive would
                        // run the caller's ops (the drive_to_return lesson).
                        frame.ret_cell = None;
                        let baseline = self.frames.len();
                        self.frames.push(frame);
                        let _ = self.drive_to_return(baseline)?;
                        continue;
                    }
                }
            }
            // Destructor-less object: note what it held, then drop it.
            self.gc_cascade(&o);
        }
        Ok(())
    }

    /// Note every object held one level down in `o`'s properties as a possible
    /// GC root: when `o` is freed those property references vanish, so the
    /// objects behind them may become collectable on the next pass (this replays
    /// the refcount cascade Zend gets for free from per-field DTORs).
    fn gc_cascade(&mut self, o: &Rc<RefCell<Object>>) {
        let b = o.borrow();
        for (_, v) in b.props.iter() {
            self.gc_note(v);
        }
        // A lazy proxy also holds its real instance off to the side (not in
        // `props`); releasing the proxy releases that, so it may become
        // collectable too — e.g. its own `__destruct` runs once the proxy goes.
        if let Some(inst) = &b.proxy_instance {
            self.gc_note(inst);
        }
    }

    /// Remove object `id`'s candidate clone from the possible-roots buffer (if
    /// present), so the buffer no longer keeps it alive. Called when the sweep
    /// takes ownership of an object it is about to free.
    fn gc_unnote(&mut self, id: u32) {
        self.gc_roots.remove(&id);
    }

    /// Track a freshly created object as a possible root. A temporary that is
    /// created and then dropped within a single statement (e.g.
    /// `Foo::make()->bar`) is consumed off the operand stack by ops we do not
    /// individually hook; seeding it here means it is already in the buffer and
    /// gets collected at that statement's sweep, just like the legacy full scan
    /// caught it. Objects that survive (stored in a variable/property) are simply
    /// drained and re-noted when they later lose a reference.
    fn gc_track(&mut self, rc: &Rc<RefCell<Object>>) {
        let id = rc.borrow().id;
        if let std::collections::hash_map::Entry::Vacant(e) = self.gc_roots.entry(id) {
            e.insert(Rc::clone(rc));
            self.gc_queue.push(id);
        }
    }

    /// Note every object held by a frame that is being dropped (a returning or
    /// unwinding frame): its locals, leftover operand stack, bound `$this`, live
    /// `foreach` iterators, surplus args and any parked exception all release
    /// their references, so a tracked object reached only through them may now be
    /// collectable. Not for *parked* frames (a suspended generator keeps these).
    fn gc_note_frame(&mut self, frame: &Frame<'m>) {
        for v in &frame.slots {
            self.gc_note(v);
        }
        for v in &frame.stack {
            self.gc_note(v);
        }
        for v in &frame.extra_args {
            self.gc_note(v);
        }
        for it in &frame.iters {
            self.gc_note_iter(it);
        }
        if let Some(exc) = &frame.pending_throw {
            self.gc_note(exc);
        }
        if let Some(this) = &frame.this {
            // A finished `__destruct`'s receiver has already been removed from
            // `created`, so the sweep cannot cascade it: if this frame holds its
            // last reference, do that cascade here before it is freed (otherwise
            // objects it owned would never be re-examined). Do NOT gc_note it —
            // re-buffering a destructed object only delays its handle-id
            // release past objects its destructor displaced, flipping Zend's
            // LIFO reuse order (gh10168: the second `new Test` must reuse the
            // NEWEST freed id).
            if let Zval::Object(o) = this {
                let id = o.borrow().id;
                if self.destructed.contains(&id) && !self.created.contains_key(&id) {
                    if Rc::strong_count(o) == 1 {
                        self.gc_cascade(o);
                    }
                } else {
                    self.gc_note(this);
                }
            } else {
                self.gc_note(this);
            }
        }
    }

    /// Note the objects a dropped `foreach` iterator releases. By-reference
    /// iterators alias a frame slot (noted with the frame), so only the value-
    /// carrying variants need walking.
    fn gc_note_iter(&mut self, it: &IterState) {
        match it {
            IterState::ByVal { entries, .. } => {
                for (k, v) in entries {
                    self.gc_note(k);
                    self.gc_note(v);
                }
            }
            IterState::ObjVals { obj, .. } | IterState::ObjRefs { obj, .. } => {
                self.gc_note(obj);
            }
            IterState::Object { it, cur_val, .. } => {
                self.gc_note(it);
                if let Some(cv) = cur_val {
                    self.gc_note(cv);
                }
            }
            // `ByRef` aliases a frame slot; `Gen` holds a generator whose frame is
            // tracked separately — neither owns a value to release here.
            IterState::ByRef { .. } | IterState::Gen { .. } => {}
        }
    }

    /// Cycle-roots pressure at which a sweep triggers an automatic cycle
    /// collection. Zend's default is 10001 buffered roots, but its buffer
    /// fills at *intra-statement* refcount decrements while phpr demotes at
    /// statement-boundary sweeps: corpus tests tuned to overflow Zend's buffer
    /// at an exact micro-instant (gh20657-002 crafts 10000 roots so the
    /// overflow lands inside a lazy-object realize) would fire here at a
    /// visibly different statement. A higher threshold keeps automatic
    /// collection for real workloads (doctrine/inflector piles up >100k dead
    /// cycle members) without crossing inside synthetic 10k-tuned tests.
    const GC_CYCLE_THRESHOLD: usize = 50_000;

    /// Trial-deletion cycle detection (the mark phase of Zend's
    /// `gc_collect_cycles`, adapted to `Rc`): starting from `roots`, walk the
    /// object graph through props, arrays, references, closure captures and
    /// lazy-proxy instances, counting for every reachable node how many of its
    /// strong references come from *inside* the walked graph. A node whose
    /// `Rc::strong_count` exceeds its in-graph edges (plus the handles we know
    /// about: `created` for objects, plus the clone this walk itself holds) has
    /// an external holder — a VM slot, a global, a static, a live container —
    /// and is alive; anything reachable from a live node is alive too. What
    /// remains is garbage kept only by its own cycles. Returns those object
    /// ids (ascending) plus the count of dead *containers* (arrays and
    /// closures — what Zend reports alongside objects in the
    /// `gc_collect_cycles` total; references are traversed transparently and
    /// never counted). Containers the walk cannot see through (generator /
    /// fiber state, host-side handles) simply leave their referents with
    /// unexplained strong counts ⇒ alive: unknown edges err on keeping.
    fn gc_classify(&self, roots: &[u32]) -> (Vec<u32>, usize) {
        use std::collections::VecDeque;
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        enum Node {
            Obj(u32),
            Arr(usize),
            Ref(usize),
            Clo(usize),
        }
        enum Handle {
            Obj(Rc<RefCell<Object>>),
            Arr(Rc<PhpArray>),
            Ref(Rc<RefCell<Zval>>),
            Clo(Rc<Closure>),
        }
        fn node_of(v: &Zval) -> Option<(Node, Handle)> {
            match v {
                Zval::Object(o) => Some((Node::Obj(o.borrow().id), Handle::Obj(Rc::clone(o)))),
                Zval::Array(a) => {
                    Some((Node::Arr(Rc::as_ptr(a) as usize), Handle::Arr(Rc::clone(a))))
                }
                Zval::Ref(r) => Some((Node::Ref(Rc::as_ptr(r) as usize), Handle::Ref(Rc::clone(r)))),
                Zval::Closure(c) => {
                    Some((Node::Clo(Rc::as_ptr(c) as usize), Handle::Clo(Rc::clone(c))))
                }
                _ => None,
            }
        }

        let mut handles: HashMap<Node, Handle> = HashMap::default();
        let mut in_edges: HashMap<Node, usize> = HashMap::default();
        let mut children: HashMap<Node, Vec<Node>> = HashMap::default();
        let mut work: VecDeque<Node> = VecDeque::new();
        for &id in roots {
            let Some(rc) = self.created.get(&id) else { continue };
            let node = Node::Obj(id);
            if handles.insert(node, Handle::Obj(Rc::clone(rc))).is_none() {
                work.push_back(node);
            }
        }
        while let Some(node) = work.pop_front() {
            let child_vals: Vec<Zval> = match handles.get(&node).expect("worklist node has handle") {
                Handle::Obj(o) => {
                    let b = o.borrow();
                    let mut vs: Vec<Zval> = b.props.iter().map(|(_, v)| v.clone()).collect();
                    if let Some(pi) = &b.proxy_instance {
                        vs.push((**pi).clone());
                    }
                    vs
                }
                Handle::Arr(a) => a.iter().map(|(_, v)| v.clone()).collect(),
                Handle::Ref(r) => vec![r.borrow().clone()],
                Handle::Clo(c) => {
                    let mut vs: Vec<Zval> = c.captures.iter().map(|(_, v)| v.clone()).collect();
                    if let Some(bt) = &c.bound_this {
                        vs.push(bt.clone());
                    }
                    vs
                }
            };
            let mut kids = Vec::new();
            for v in &child_vals {
                if let Some((cn, ch)) = node_of(v) {
                    *in_edges.entry(cn).or_insert(0) += 1;
                    kids.push(cn);
                    if let std::collections::hash_map::Entry::Vacant(e) = handles.entry(cn) {
                        e.insert(ch);
                        work.push_back(cn);
                    }
                }
            }
            children.insert(node, kids);
        }
        // External check. `known` = references this walk can account for
        // without an outside holder: our own handle clone, and `created` for a
        // tracked object. An untracked object (interned enum case) is immortal.
        let mut live: VecDeque<Node> = VecDeque::new();
        let mut is_live: HashSet<Node> = HashSet::default();
        for (node, h) in &handles {
            let external = match (node, h) {
                (Node::Obj(id), Handle::Obj(o)) => {
                    !self.created.contains_key(id)
                        || Rc::strong_count(o) - 2 > in_edges.get(node).copied().unwrap_or(0)
                }
                (_, Handle::Arr(a)) => {
                    Rc::strong_count(a) - 1 > in_edges.get(node).copied().unwrap_or(0)
                }
                (_, Handle::Ref(r)) => {
                    Rc::strong_count(r) - 1 > in_edges.get(node).copied().unwrap_or(0)
                }
                (_, Handle::Clo(c)) => {
                    Rc::strong_count(c) - 1 > in_edges.get(node).copied().unwrap_or(0)
                }
                _ => unreachable!("node/handle kinds always match"),
            };
            if external && is_live.insert(*node) {
                live.push_back(*node);
            }
        }
        while let Some(node) = live.pop_front() {
            if let Some(kids) = children.get(&node) {
                for k in kids {
                    if is_live.insert(*k) {
                        live.push_back(*k);
                    }
                }
            }
        }
        let mut whites: Vec<u32> = Vec::new();
        let mut dead_containers = 0usize;
        for n in handles.keys() {
            if is_live.contains(n) {
                continue;
            }
            match n {
                Node::Obj(id) if self.created.contains_key(id) => whites.push(*id),
                Node::Arr(_) | Node::Clo(_) => dead_containers += 1,
                _ => {}
            }
        }
        whites.sort_unstable();
        (whites, dead_containers)
    }

    /// Collect reference cycles (Zend `gc_collect_cycles`): classify the
    /// buffered possible roots via [`Self::gc_classify`], run `__destruct` on
    /// the doomed objects (oldest-first, synchronously — an exception
    /// propagates to the triggering statement, as in PHP), re-classify (a
    /// destructor may resurrect part of the graph), then break the surviving
    /// cycles by dropping every white object's properties and freeing it.
    /// Freeing may liberate further garbage (its contents are re-noted), so
    /// the whole pass repeats until nothing more dies — in PHP that follow-up
    /// garbage would have died by refcount the instant the cycle broke, so it
    /// must not outlive this call. Returns the number of destroyed objects,
    /// arrays and closures (what Zend's counter reports).
    fn collect_cycles(&mut self) -> Result<i64, PhpError> {
        let mut total = 0i64;
        loop {
            // Leftover acyclic candidates join the root set; their buffer
            // clones are dropped so the classifier sees only real holders.
            let leftover: Vec<u32> = self.gc_roots.keys().copied().collect();
            self.gc_roots.clear();
            self.gc_queue.clear();
            self.gc_cycle_roots.extend(leftover);
            let roots: Vec<u32> = self
                .gc_cycle_roots
                .drain()
                .collect::<Vec<_>>()
                .into_iter()
                .filter(|id| self.created.contains_key(id))
                .collect();
            if roots.is_empty() {
                break;
            }
            let (whites, _) = self.gc_classify(&roots);
            log::debug!(
                target: "phpr::gc",
                "cycle collect: {} roots, {} garbage objects",
                roots.len(),
                whites.len()
            );
            if whites.is_empty() {
                break;
            }
            // Destructor phase, oldest-first (creation order — matches Zend's
            // buffer order). The destructor runs at most once per object
            // (`destructed`), and never for an uninitialized lazy wrapper.
            for &id in whites.iter() {
                let Some(rc) = self.created.get(&id) else { continue };
                let rc = Rc::clone(rc);
                let (cid, lazy_wrapper) = {
                    let b = rc.borrow();
                    (b.class_id as usize, b.lazy.is_some())
                };
                if lazy_wrapper || self.destructed.contains(&id) {
                    continue;
                }
                if resolve_method_runtime(&self.classes, cid, b"__destruct").is_some() {
                    log::debug!(target: "phpr::gc", "destruct (cycle): {}#{}", String::from_utf8_lossy(&self.classes[cid].name), id);
                    self.destructed.insert(id);
                    self.call_method_sync(Zval::Object(rc), b"__destruct", Vec::new())?;
                }
            }
            // A destructor may have stored parts of the graph somewhere
            // reachable: only what is *still* unreferenced dies. Dead
            // arrays/closures inside the garbage count toward the total.
            let (whites, dead_containers) = self.gc_classify(&roots);
            // Detach every white's contents first, then free: a white's
            // properties may hold the last references to other whites.
            let mut taken: Vec<(u32, Props, Option<Box<Zval>>)> = Vec::new();
            for &id in whites.iter().rev() {
                let Some(rc) = self.created.get(&id) else { continue };
                let mut b = rc.borrow_mut();
                let props = std::mem::replace(&mut b.props, Props::new());
                let proxy = b.proxy_instance.take();
                drop(b);
                taken.push((id, props, proxy));
            }
            if taken.is_empty() && dead_containers == 0 {
                break;
            }
            total += (taken.len() + dead_containers) as i64;
            for (id, props, proxy) in taken {
                self.created.remove(&id);
                self.gc_unnote(id);
                // The dropped contents — properties, a proxy's real instance,
                // a lazy wrapper's initializer closure — may hold the last
                // reference to garbage outside the white set: note everything
                // so the next fixpoint round (or the normal sweep) reaps it.
                if let Some(init) = self.lazy_init.remove(&id) {
                    self.gc_note(&init);
                }
                self.lazy_props.remove(&id);
                for (_, v) in props.iter() {
                    self.gc_note(v);
                }
                if let Some(p) = &proxy {
                    self.gc_note(p);
                }
            }
        }
        if total > 0 {
            log::debug!(target: "phpr::gc", "cycle collect: {} values freed", total);
        }
        Ok(total)
    }


    /// PHP 8.1 link-time policy for the legacy `Serializable` interface
    /// (zend_inheritance.c): a class implementing it — directly or through an
    /// interface — gets an E_DEPRECATED at declaration unless it also provides
    /// both `__serialize()` and `__unserialize()`; interfaces and abstract
    /// classes are exempt (the concrete implementor reports). An enum may not
    /// implement it at all: the deprecation still fires first (Zend checks the
    /// interface before the enum rule), then the declaration is a fatal.
    fn serializable_link_check(&mut self, cid: usize) -> Result<(), PhpError> {
        let Some(&ser) = self.class_index.get(&b"serializable"[..]) else {
            return Ok(());
        };
        if cid == ser {
            return Ok(());
        }
        let cc = self.classes[cid];
        if matches!(cc.instantiable, Instantiable::Interface | Instantiable::Abstract) {
            return Ok(());
        }
        if !is_instance_of(&self.classes, self.stringable_id, cid, ser) {
            return Ok(());
        }
        if resolve_method_runtime(&self.classes, cid, b"__serialize").is_none()
            || resolve_method_runtime(&self.classes, cid, b"__unserialize").is_none()
        {
            self.diags.push(Diag::Deprecated(format!(
                "{} implements the Serializable interface, which is deprecated. Implement __serialize() and __unserialize() instead (or in addition, if support for old PHP versions is necessary)",
                String::from_utf8_lossy(cc.class_name.as_bytes())
            )));
        }
        if matches!(cc.instantiable, Instantiable::Enum) {
            self.fatal_line = cc.line;
            return Err(PhpError::Error(format!(
                "Enum {} cannot implement the Serializable interface",
                String::from_utf8_lossy(cc.class_name.as_bytes())
            )));
        }
        Ok(())
    }

    /// Render every diagnostic raised since the last flush into `rendered`,
    /// stamped with `line` and the module file (E1; mirrors `eval::flush_diags`):
    /// `\n{Severity}: {message} in {file} on line {line}\n`.
    /// Compile a PHP regex, memoising the result per raw pattern (PCRE keeps a
    /// per-request pattern cache). Returns a shared handle; `None` (also cached)
    /// means the pattern is invalid.
    fn preg_compile(&mut self, pat: &[u8]) -> Option<Rc<crate::preg::Engine>> {
        // preg_last_error: ogni operazione preg che compila resetta a
        // NO_ERROR; un pattern invalido (anche da cache) segna
        // PREG_INTERNAL_ERROR — il BAD_UTF8 lo segna subject_text (WP-16).
        let engine = if let Some(hit) = self.preg_cache.get(pat) {
            hit.clone()
        } else {
            let engine = crate::preg::compile(pat).map(Rc::new);
            self.preg_cache.insert(pat.to_vec(), engine.clone());
            engine
        };
        crate::preg::set_last_error(if engine.is_some() { 0 } else { 1 });
        engine
    }

    fn flush_diags(&mut self, line: Line) -> Result<(), PhpError> {
        // Under `@` the flush still runs: Zend delivers a suppressed diagnostic
        // to the user error handler (which sees error_reporting() == 4437 and
        // usually declines it) — only the default render is swallowed, which
        // `raise_diagnostic` gates on `suppress_depth` itself. `Op::SuppressEnd`
        // flushes any leftovers before dropping them.
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

    /// Apply increment/decrement to a *copy* of the operand, returning the new
    /// value and any diagnostics it produced (rather than rendering them). This
    /// lets the caller raise the diagnostics — and run a `set_error_handler` — at
    /// the right moment, before committing the result. For scalars the copy is by
    /// value (pure); a reference operand still mutates its shared cell, matching the
    /// pre-existing behaviour for that rare case.
    fn compute_incdec(&mut self, mut v: Zval, inc: bool) -> Result<(Zval, Vec<Diag>), PhpError> {
        // `BcMath\Number` overloads ++/-- as `+ 1` / `- 1` (do_operation), like PHP.
        if let Some(recv) = number_receiver(&v) {
            let code = if inc { 0 } else { 1 }; // 0=add, 1=sub
            let r = self.call_method_sync(
                recv,
                b"__op",
                vec![Zval::Long(code), v.clone(), Zval::Long(1)],
            )?;
            return Ok((r, Vec::new()));
        }
        let mut diags = Vec::new();
        if inc {
            ops::increment(&mut v, &mut diags)?;
        } else {
            ops::decrement(&mut v, &mut diags)?;
        }
        Ok((v, diags))
    }

    /// Raise a batch of just-produced diagnostics synchronously through the shared
    /// chokepoint (so a `set_error_handler` runs *now*, and can throw or mutate
    /// state before the caller commits its result). Used by inc/dec, where PHP
    /// raises the diagnostic before writing the new value back to the variable.
    fn raise_diags(&mut self, diags: Vec<Diag>, line: Line) -> Result<(), PhpError> {
        for d in diags {
            let (errno, message) = match d {
                Diag::Warning(m) => (2, m),
                Diag::Notice(m) => (8, m),
                Diag::Deprecated(m) => (8192, m),
            };
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
        // An enclosing `@` swallows the default render through the masked
        // `error_level` (BEGIN_SILENCE `&= 4437`) — the gate below — while the
        // `last_error` record above still happened, like PHP's error_get_last
        // under suppression (a nested call's trigger_error is in the region).
        // Default render (Session 1 behaviour), gated on `error_reporting`.
        // Attributed to the nearest NON-prelude frame: a prelude-authored
        // class raising a user-level warning (PDO/SQLite3 ERRMODE_WARNING)
        // reports the USER call site, like the C implementations do.
        if self.error_level & errno != 0 {
            let (file, rline) = self.diagnostic_site(line);
            // log_errors sink: an explicit `error_log` file receives the
            // php_log_err-stamped line under every SAPI (WP's template tests
            // read the file back); with no file, the web SAPI collects the
            // line for the host to timestamp onto stderr, and the CLI SAPI
            // log stays untouched (phpt baselines pin stdout+stderr).
            if self.ini.get_bool(b"log_errors") {
                let logline = format!(
                    "PHP {}:  {} in {} on line {}",
                    errno_label(errno),
                    message,
                    String::from_utf8_lossy(&file),
                    rline
                );
                let dest = self.ini.get(b"error_log").unwrap_or(b"").to_vec();
                if !dest.is_empty() {
                    use std::io::Write;
                    use std::os::unix::ffi::OsStrExt;
                    let mut lb = host::error_log_stamp();
                    lb.extend_from_slice(logline.as_bytes());
                    lb.push(b'\n');
                    let _ = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(std::ffi::OsStr::from_bytes(&dest))
                        .and_then(|mut f| f.write_all(&lb));
                } else if self.web {
                    self.error_log.push(logline.into_bytes());
                }
            }
            // `display_errors=0` suppresses the visible render only (the
            // last_error record above and the log line still happened).
            if !self.ini.get_bool(b"display_errors") {
                return Ok(());
            }
            // error_prepend_string/error_append_string wrap every displayed
            // diagnostic (main.c php_error_cb "%s<br />\n<b>%s</b>…%s").
            let prepend = self.ini.get(b"error_prepend_string").unwrap_or(b"").to_vec();
            let append = self.ini.get(b"error_append_string").unwrap_or(b"").to_vec();
            let mut block = if self.ini.get_bool(b"html_errors") {
                // html_errors display form (the web SAPI default; oracle-pinned,
                // note the double space after the colon).
                format!(
                    "<br />\n<b>{}</b>:  {} in <b>{}</b> on line <b>{}</b><br />\n",
                    errno_label(errno),
                    message,
                    String::from_utf8_lossy(&file),
                    rline
                )
                .into_bytes()
            } else {
                let mut block = format!("\n{}: {} in ", errno_label(errno), message).into_bytes();
                block.extend_from_slice(&file);
                block.extend_from_slice(format!(" on line {rline}\n").as_bytes());
                block
            };
            if !prepend.is_empty() {
                let mut b = prepend;
                b.extend_from_slice(&block);
                block = b;
            }
            block.extend_from_slice(&append);
            // Diagnostic display is ordinary output in PHP: it flows THROUGH
            // the ob stack (captured, compressed, reordered with echoes) and
            // marks headers-sent only when it reaches the real sink —
            // oracle-verified (a buffered warning leaves session_id() free to
            // change the id; an unbuffered one blocks it). Shutdown-time
            // renders stay raw: the buffers are already flushed and a handler
            // must not run this late.
            if self.final_flush {
                self.rendered.extend_from_slice(&block);
            } else {
                self.write_output(&block)?;
            }
        }
        Ok(())
    }

    /// The `(file, line)` a diagnostic is attributed to: the nearest NON-prelude
    /// frame — a prelude-authored class raising a user-level warning
    /// (PDO/SQLite3 ERRMODE_WARNING) reports the USER call site, like the C
    /// implementations do. Shared by the default render and the file/line
    /// arguments of a `set_error_handler` callback (WP captures them to
    /// re-render errors raised inside an output-buffer handler).
    fn diagnostic_site(&self, line: Line) -> (Vec<u8>, Line) {
        if self.frames.is_empty() {
            return (self.module.file.to_vec(), line);
        }
        // A prelude frame is detected via its *defining class* — or, for
        // a free prelude function (fsockopen, ob_gzhandler…), via the
        // function's own defining file (the prelude lowers into the main
        // unit, so module.file can't tell).
        let is_prelude = |f: &Frame| {
            f.class
                .map(|c| self.classes[c].file.as_ref() == b"prelude")
                .unwrap_or(false)
                || f.func.file.as_ref() == b"prelude"
        };
        let mut fi = self.frames.len() - 1;
        while fi > 0 && is_prelude(&self.frames[fi]) {
            fi -= 1;
        }
        let rline = if fi + 1 == self.frames.len() { line } else { self.cur_line(fi) };
        // An eval() unit renders as PHP's composite file name,
        // "<file>(<line>) : eval()'d code" (same as backtraces).
        let file = match &self.frames[fi].eval_origin {
            Some((ofile, oline)) => {
                let mut s = ofile.to_vec();
                s.extend_from_slice(format!("({oline}) : eval()'d code").as_bytes());
                s
            }
            None => self.frames[fi].module.file.to_vec(),
        };
        (file, rline)
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
        // file/line: same attribution as the default render (nearest
        // non-prelude frame) — NOT the main module, which is just the entry
        // script (vendor/bin/phpunit under a test run).
        let (hfile, hline) = self.diagnostic_site(line);
        let args = vec![
            Zval::Long(errno),
            Zval::Str(PhpStr::new(message.as_bytes().to_vec())),
            Zval::Str(PhpStr::new(hfile)),
            Zval::Long(hline as i64),
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
        self.write_output(bytes)?;
        Ok(())
    }

    /// Route program output to the active sink: the topmost output buffer
    /// (`ob_start`) when one is active, otherwise the real `stdout`/`rendered`
    /// streams. Diagnostics are *not* captured by output buffering, so they keep
    /// flowing through `flush_diags` into `rendered` at their point of occurrence.
    fn write_output(&mut self, bytes: &[u8]) -> Result<(), PhpError> {
        if let Some(buf) = self.ob_stack.last_mut() {
            buf.content.extend_from_slice(bytes);
            // A `chunk_size` buffer auto-flushes to its parent (handler phase
            // `PHP_OUTPUT_HANDLER_WRITE` = 0) the moment a write makes it reach the
            // threshold — PHP flushes the whole accumulated content at once, not in
            // chunk-sized pieces.
            let cs = buf.chunk_size;
            if cs > 0 && buf.content.len() >= cs {
                self.emit_buffer_op(0)?;
            }
        } else {
            // Output reaching the real sink is when PHP considers headers "sent";
            // remember where it first happened for the header() warnings. Under
            // the web SAPI the cli-server buffers 4096 bytes before the first
            // socket write, so "sent" flips only when the sink CROSSES that
            // threshold — and PHP's recorded first-output site is the statement
            // whose write crossed it (WP-10 hs probe), not the first echo.
            let crossed = if self.web {
                let before = self.rendered.len();
                before < WEB_SEND_THRESHOLD && before + bytes.len() >= WEB_SEND_THRESHOLD
            } else {
                !self.output_started && !bytes.is_empty()
            };
            if crossed {
                self.output_started = true;
                if let Some(top) = self.frames.len().checked_sub(1) {
                    let file = self.frames[top].module.file.to_vec();
                    let line = self.cur_line(top);
                    self.output_start = Some((file, line));
                }
            }
            self.stdout.extend_from_slice(bytes);
            self.rendered.extend_from_slice(bytes);
        }
        Ok(())
    }

    /// Flush the topmost buffer's content to its *parent* sink while keeping the
    /// buffer active (emptied) — the shared engine for a `chunk_size` auto-flush
    /// (`op` = 0) and a manual `ob_flush` (`op` = `PHP_OUTPUT_HANDLER_FLUSH` = 4).
    /// The handler phase gains `PHP_OUTPUT_HANDLER_START` (1) on the first flush of
    /// this buffer. The buffer is detached during the callback + parent write so
    /// that any output the handler itself produces goes to the parent, then pushed
    /// back emptied.
    fn emit_buffer_op(&mut self, op: i64) -> Result<(), PhpError> {
        let Some(mut buf) = self.ob_stack.pop() else { return Ok(()) };
        let content = std::mem::take(&mut buf.content);
        let mut phase = op;
        if !buf.started {
            phase |= 1;
            buf.started = true;
        }
        let out = self.apply_ob_callback(&buf.callback, content, phase)?;
        let r = self.write_output(&out);
        self.ob_stack.push(buf);
        r
    }

    /// Run an output buffer's handler (if any) over `content` with the given phase
    /// bitmask, returning the bytes to forward to the parent sink: the handler's
    /// non-`false` return cast to a string, or the original content when there is
    /// no handler or it returns `false`.
    fn apply_ob_callback(
        &mut self,
        callback: &Option<Zval>,
        content: Vec<u8>,
        phase: i64,
    ) -> Result<Vec<u8>, PhpError> {
        match callback {
            Some(cb) => {
                let r = self.call_callable(
                    cb.clone(),
                    vec![Zval::Str(PhpStr::new(content.clone())), Zval::Long(phase)],
                )?;
                Ok(match r.deref_clone() {
                    Zval::Bool(false) => content,
                    other => convert::to_zstr_cast(&other, &mut self.diags).as_bytes().to_vec(),
                })
            }
            None => Ok(content),
        }
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
        // PHP's ZPP dereferences a reference argument for every by-value
        // parameter, and the Value family is by-value by construction — but a
        // dynamic call site (unknown callee ⇒ prefer-ref sends) can deliver
        // `Zval::Ref` cells here (`$f = 'is_numeric'; $f($x)`, WordPress's
        // rest_get_best_type_for_value), which a variant-matching predicate
        // would misread. Unwrap them at this single choke point.
        let derefed: Vec<Zval>;
        let args: &[Zval] = if args.iter().any(|a| matches!(a, Zval::Ref(_))) {
            derefed = args.iter().map(Zval::deref_clone).collect();
            &derefed
        } else {
            args
        };
        let mut produced = Vec::new();
        let mut direct = Vec::new();
        // A `var_dump` arg-walk left the objects' `__debugInfo()` results here;
        // take them so the pure builtin can render them (empty for every other).
        let debug_info = std::mem::take(&mut self.var_dump_debug);
        // Likewise, a string-coercing builtin's arg-walk may have precomputed
        // `__toString()` results (empty for every builtin that does not coerce).
        let stringify = std::mem::take(&mut self.stringify_args);
        let res = {
            let mut ctx = Ctx {
                out: &mut produced,
                diags: &mut self.diags,
                direct_out: &mut direct,
                debug_info: &debug_info,
                stringify: &stringify,
            };
            f(args, &mut ctx)
        };
        self.flush_diags(line)?;
        self.write_output(&produced)?;
        // Stream writes to stdout bypass the ob stack (see Ctx::direct_out).
        if !direct.is_empty() {
            self.stdout.extend_from_slice(&direct);
            self.rendered.extend_from_slice(&direct);
        }
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


    /// The cell a [`DimBase`] is rooted at, for read-only path inspection.
    fn base_cell(&self, base: DimBase, top: usize) -> &Zval {
        match base {
            DimBase::Local(s) => &self.frames[top].slots[s as usize],
            DimBase::Global(s) => &self.frames[0].slots[s as usize],
            DimBase::Superglobal(i) => &self.superglobals[i as usize],
        }
    }

    /// Whether class `cid` implements `Iterator` or `IteratorAggregate` (i.e. a
    /// `foreach` over it drives the iterator protocol), step 51.
    fn is_traversable(&self, cid: usize) -> bool {
        self.iterator_id.is_some_and(|i| is_instance_of(&self.classes, self.stringable_id, cid, i))
            || self.iteratoraggregate_id.is_some_and(|i| is_instance_of(&self.classes, self.stringable_id, cid, i))
    }

    /// Whether class `cid` implements `IteratorAggregate` (foreach calls
    /// `getIterator()` first), step 51.
    fn is_aggregate(&self, cid: usize) -> bool {
        self.iteratoraggregate_id.is_some_and(|i| is_instance_of(&self.classes, self.stringable_id, cid, i))
    }

    /// If `v` (deref'd) is an object implementing `ArrayAccess`, return it as a
    /// receiver value for an `offset*` dispatch; otherwise `None` (step 51).
    fn as_arrayaccess(&self, v: &Zval) -> Option<Zval> {
        let aa = self.arrayaccess_id?;
        match v.deref_clone() {
            Zval::Object(o)
                if is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, aa) =>
            {
                Some(Zval::Object(o))
            }
            _ => None,
        }
    }

    /// If `v` (deref'd) is an object implementing `Countable`, return it as a
    /// receiver for a `count()` dispatch; otherwise `None` (step 56).
    fn as_countable(&self, v: &Zval) -> Option<Zval> {
        let c = self.countable_id?;
        match v.deref_clone() {
            Zval::Object(o)
                if is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, c) =>
            {
                Some(Zval::Object(o))
            }
            _ => None,
        }
    }

    /// Enter a protocol method frame (`offset*` for ArrayAccess, `rewind`/`valid`/
    /// `current`/`key`/`next`/`getIterator` for the iterator protocol) on `recv`
    /// with `args` bound (step 51). [`RetMode`] selects where the return goes:
    /// `Stack` flows it to the caller's operand stack (a value getter), `Discard`
    /// drops it (a `void` method), `Capture` writes it into the caller's cell (the
    /// re-entrant iterator state machine reads it back).
    fn enter_object_method(
        &mut self,
        recv: Zval,
        method: &[u8],
        args: Vec<Zval>,
        ret: RetMode,
    ) -> Result<(), PhpError> {
        let cid = object_class_id(&recv).expect("protocol receiver is an object");
        let (defc, midx) = resolve_method_runtime(&self.classes, cid, method).ok_or_else(|| {
            PhpError::Error(format!(
                "Call to undefined method {}::{}()",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(method)
            ))
        })?;
        let callee = &self.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee, self.class_mod(defc));
        // A by-reference getter (`&offsetGet`) returns its raw `Ref`; a value
        // consumer (`count($o['k'])`) needs the dereferenced value.
        frame.ret_deref = callee.by_ref && matches!(ret, RetMode::Stack);
        bind_params(&mut frame, args);
        frame.this = Some(recv);
        frame.class = Some(defc);
        frame.static_class = Some(cid);
        frame.ret_cell = match ret {
            RetMode::Stack => None,
            RetMode::Discard => Some(Rc::new(RefCell::new(Zval::Null))),
            RetMode::Capture(cell) => Some(cell),
        };
        self.enter_callee(frame)
    }

    /// The [`Module`] that *defines* class `defc`, used as a method/thunk frame's
    /// module so its own bytecode indices resolve in the right unit (step 57, Phase
    /// 1c-2b). Falls back to `self.module` defensively (a class id should always be
    /// in range, and for `main`'s classes the answer equals `self.module`).
    #[inline]
    fn class_mod(&self, defc: ClassId) -> &'m Module {
        self.class_module.get(defc).copied().unwrap_or(self.module)
    }

    /// Find or assign the stable `module_id` of a leaked module (by pointer
    /// identity), registering it in `self.modules` on first sight. Stored in
    /// [`Closure`] values so they stay callable across module boundaries.
    fn module_id(&mut self, m: &'m Module) -> usize {
        if let Some(i) = self.modules.iter().position(|x| std::ptr::eq(*x, m)) {
            i
        } else {
            self.modules.push(m);
            self.modules.len() - 1
        }
    }

    /// `eval($code)` (step 57, Phase 1): compile `src` (already `<?php`-prefixed)
    /// as its own translation unit at run time and run it via [`Self::drive_unit`].
    /// When the caller's HIR is retained (`main_hir`, the normal `run_source`
    /// path), the eval is lowered *against the image* (Phase 1c-2c): it sees the
    /// caller's user classes, so `eval("class Bar extends Foo {}")` flattens `Bar`'s
    /// inherited layout from `Foo`. Returns the unit's `return` value (or `null`);
    /// a parse/compile error yields `false` (MVP — PHP throws `ParseError`).
    /// The unit shares the caller's variable scope like `include` (Zend runs
    /// the op_array on the calling symbol table — symfony's ContainerBuilder
    /// evals `new class($initializer) …` reading a local). Limitation (MVP):
    /// with no retained HIR (unit tests via `run_module`) it compiles
    /// standalone and cannot extend a caller's user class.
    fn run_eval(&mut self, src: &[u8]) -> Result<Zval, PhpError> {
        // Compile against the accumulating image when an HIR is retained, so an
        // eval class can extend/implement an already-loaded (or autoloaded) class;
        // otherwise stand alone.
        // An eval can declare classes/functions: fold its source into the
        // unit-load chain so later includes key their cache on it.
        self.unit_chain_fp = fp_mix(self.unit_chain_fp, b"eval", src);
        let mut eval_pure = true;
        let program = match self.lower_unit(b"eval()'d code", src, &mut eval_pure)? {
            Ok(p) => p,
            Err(_) => return Ok(Zval::Bool(false)),
        };
        self.accumulate_seed(&program);
        let stubs = self.seed_stub_mask(&program);
        let module = match crate::compile::compile_program_stubbed(&program, self.registry, &stubs) {
            Ok(m) => m,
            Err(_) => return Ok(Zval::Bool(false)),
        };
        // Record the eval()'s call site (the file/line of the frame invoking it) so
        // a backtrace can render this unit as `eval()` and attribute code it calls
        // to "<file>(<line>) : eval()'d code" (Phase 1c-2c).
        let caller = self.frames.len() - 1;
        let origin = (self.frames[caller].module.file.clone(), self.cur_line(caller));
        self.drive_unit(module, Some(origin), Some(caller))
    }

    /// Link a freshly-compiled `eval`/`include` unit into the running VM and drive
    /// its `main` to completion (step 57). Shared by [`Self::run_eval`] and
    /// [`Self::run_include`]. Relocates the unit's compile-time class ids into the
    /// global table (Phase 1c-2b), offsets its static-cell ids past the live range,
    /// leaks it so its `&'m` bytecode outlives the call, appends its genuinely-new
    /// classes (so they stay visible afterwards) and links its new user functions
    /// (Phase 1c-2a/2). `eval_origin` marks the pushed frame as an `eval()` unit for
    /// backtraces; `None` for an `include` (its frame is the real file). Returns the
    /// unit's top-level `return` value (or `null` on fall-through).
    fn drive_unit(
        &mut self,
        mut module: Module,
        eval_origin: Option<(Box<[u8]>, Line)>,
        scope_bridge: Option<usize>,
    ) -> Result<Zval, PhpError> {
        let (class_remap, new_locals) = self.unit_class_remap(&module);
        // Rewrite every op / class-metadata id in place (the module is still owned),
        // and offset the unit's static-cell ids past the live `self.statics` range.
        relocate_module_class_ids(&mut module, &class_remap, self.statics.len());
        let leaked: &'static Module = Box::leak(Box::new(module));
        self.run_linked(leaked, &new_locals, eval_origin, scope_bridge)
    }

    /// Build the unit-local -> global class-id remap: a name already in the
    /// global table dedups to the existing id, a genuinely new user class is
    /// appended. Against the caller image the dedup is an identity for every
    /// shared class, leaving only this unit's own declarations to relocate.
    fn unit_class_remap(&self, module: &Module) -> (Vec<ClassId>, Vec<usize>) {
        let mut class_remap: Vec<ClassId> = Vec::with_capacity(module.classes.len());
        let mut new_locals: Vec<usize> = Vec::new();
        for (i, cc) in module.classes.iter().enumerate() {
            let lower = cc.name.to_ascii_lowercase();
            if let Some(&existing) = self.class_index.get(&lower) {
                class_remap.push(existing);
            } else if i < self.classes.len() && self.classes[i].name == cc.name {
                // A seed-image entry whose name is NOT registered: a
                // still-undeclared CONDITIONAL class of an earlier unit. The
                // seed prefix mirrors the runtime table 1:1 (accumulate_seed
                // keeps ids aligned), so remap by identity — re-appending it
                // as "new" would eagerly register the name and flip the outer
                // `if ( ! class_exists( … ) )` guard to skip its whole block
                // (WordPress pomo/translations.php via pomo/mo.php).
                class_remap.push(i);
            } else {
                let new_id = self.classes.len() + new_locals.len();
                class_remap.push(new_id);
                new_locals.push(i);
            }
        }
        (class_remap, new_locals)
    }

    /// Register a linked (relocated, leaked) unit module into the VM and drive
    /// its body — the back half of [`Self::drive_unit`], shared with the unit
    /// cache: `run_include` re-drives a cached, already-relocated module through
    /// here after verifying the remap/static baseline still matches.
    fn run_linked(
        &mut self,
        leaked: &'m Module,
        new_locals: &[usize],
        eval_origin: Option<(Box<[u8]>, Line)>,
        scope_bridge: Option<usize>,
    ) -> Result<Zval, PhpError> {
        self.statics.resize(self.statics.len() + leaked.static_count, None);
        // Append the new user classes to the global table (dedup'd prelude /
        // caller-image classes were already mapped to existing ids). A conditional
        // declaration still gets its global slot/id (so its ops relocate), but its
        // name is *not* registered until the unit body reaches its `Op::DeclareClass`
        // — so a guarded polyfill (`if (!class_exists(X)) { class X {} }`) respects
        // its condition, exactly as conditional functions do above.
        for &i in new_locals {
            self.classes.push(&leaked.classes[i]);
            self.class_module.push(leaked);
            if !leaked.conditional_classes.contains(&i) {
                self.class_index
                    .insert(leaked.classes[i].name.to_ascii_lowercase(), self.classes.len() - 1);
            }
        }

        let saved = self.module;
        self.module = leaked;
        // Register the unit's user functions (those not already provided by the
        // caller's module) BEFORE driving the body: Zend hoists top-level
        // functions at compile time of the include/eval, so a nested include
        // (or a hook fired from one) must already see them — wp-admin/menu.php
        // registers '_add_themes_utility_last' on 'admin_menu' and the hook
        // fires from wp-admin/includes/menu.php, included before menu.php ends.
        // Only the unconditionally-hoisted functions are registered here; a
        // conditional declaration registers itself via its `Op::DeclareFn` when
        // the unit body reaches it (so a guarded polyfill respects its condition).
        for (idx, f) in leaked.functions.iter().enumerate() {
            if leaked.conditional_fns.contains(&idx) {
                continue;
            }
            let already = saved.functions.iter().any(|cf| name_eq_ignore_case(&cf.name, &f.name));
            if !already {
                self.linked_functions.entry(f.name.to_ascii_lowercase()).or_insert((leaked, idx));
            }
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(&leaked.main, leaked);
        frame.eval_origin = eval_origin;
        // PHP's include shares the *including* scope's variable table. Alias the
        // unit frame's named slots to the includer's cells (promoting the
        // includer slot to a shared `Zval::Ref` via `make_cell`): reads see the
        // surrounding variables live, and assignments — including names the file
        // introduces — land back in the includer, with no copy-back needed even
        // when the unit throws. Global scope aligns by index (the unit was
        // lowered seeded on `seed_globals`, so its leading slots mirror the
        // global frame); a function scope matches by name.
        // Names the unit mentions that the includer does not have yet: PHP
        // defines them in the SHARED scope when the unit assigns them (wp-cli
        // collects wp-config.php's `$table_prefix` from an eval through
        // get_defined_vars()). Bridge each through a fresh cell and publish it
        // into the includer's dyn_vars after the run — but only if the unit
        // actually DEFINED it (an eval that merely reads `$x` must not create
        // `$x` in the caller).
        let mut fresh_bridged: Vec<(Vec<u8>, Rc<RefCell<Zval>>)> = Vec::new();
        if let Some(caller) = scope_bridge {
            // Is the includer itself running at GLOBAL scope? Walk the bridge
            // chain: a top-level include-of-include bottoms out at frame 0
            // through unit frames only; an include inside a function stops at
            // the function frame. At global scope Zend has ONE symbol table,
            // so a name this unit introduces must alias the GLOBAL cell (the
            // unit's lowering already registered it in `seed_globals`) — a
            // detached fresh cell would leave `global $x` / `$GLOBALS` in a
            // deeper include reading NULL (wp-admin/menu.php builds `$menu`,
            // wp-admin/includes/menu.php does `global $menu` and uksort()s it).
            let global_scope = {
                let mut root = caller;
                while root != 0 {
                    match self.frames[root].bridge_caller {
                        Some(up) => root = up,
                        None => break,
                    }
                }
                root == 0
            };
            let names = &leaked.main.slot_names;
            if caller == 0 {
                let n = names.len().min(self.frames[0].slots.len());
                for i in 0..n {
                    frame.slots[i] = Zval::Ref(make_cell(&mut self.frames[0].slots[i]));
                }
            } else {
                for (i, name) in names.iter().enumerate() {
                    if let Some(cs) =
                        self.frames[caller].func.slot_names.iter().position(|n| n == name)
                    {
                        if let Some(slot) = self.frames[caller].slots.get_mut(cs) {
                            frame.slots[i] = Zval::Ref(make_cell(slot));
                        }
                    } else if let Some(dyn_slot) =
                        self.frames[caller].dyn_vars.get_mut(&name[..])
                    {
                        // A variable the includer created by NAME at run time
                        // (extract() in HtmlErrorRenderer::include feeds its
                        // template context this way) shares the same way.
                        frame.slots[i] = Zval::Ref(make_cell(dyn_slot));
                    } else if global_scope {
                        // Global-scope include chain: the fresh name lives in
                        // the global symbol table, immediately (Zend has one
                        // table for the whole chain — see note above).
                        let slot = self.global_slot_by_name(name);
                        frame.slots[i] = Zval::Ref(make_cell(&mut self.frames[0].slots[slot]));
                    } else {
                        let cell = Rc::new(RefCell::new(Zval::Undef));
                        frame.slots[i] = Zval::Ref(Rc::clone(&cell));
                        fresh_bridged.push((name.to_vec(), cell));
                    }
                }
            }
            // An include/eval inside a method also inherits `$this` and the
            // class scope: an included template may read `$this->prop` or call
            // a private method of the including class (Zend runs the op_array
            // with the caller's scope/This — symfony's HtmlErrorRenderer
            // renders its .html.php views exactly this way).
            frame.this = self.frames[caller].this.clone();
            frame.class = self.frames[caller].class;
            frame.static_class = self.frames[caller].static_class;
            // A `global` statement in this unit rebinds the SHARED symbol —
            // record the includer so bind_global_dyn can walk the chain.
            frame.bridge_caller = Some(caller);
        }
        self.frames.push(frame);
        let outcome = self.drive_to_return(baseline);
        self.module = saved;
        // Publish unit-introduced variables into the includer scope (see
        // fresh_bridged above): only names the unit left DEFINED.
        if let Some(caller) = scope_bridge {
            if caller != 0 && caller < self.frames.len() {
                for (name, cell) in fresh_bridged {
                    if !matches!(&*cell.borrow(), Zval::Undef) {
                        self.frames[caller].dyn_vars.insert(name, Zval::Ref(cell));
                    }
                }
            }
        }
        outcome
    }

    /// Lower an `eval`/`include` unit against the accumulating class image,
    /// autoloading any `extends`/`implements` target not yet loaded and retrying
    /// (step 57, Phase 3) — so a lazily-autoloaded `class Dog extends Animal`
    /// loads `Animal` first. With no retained image it lowers standalone. The outer
    /// `Result` carries a throwing autoloader's exception; the inner one a genuine
    /// lower failure (parse / unsupported / still-undefined parent).
    /// `pure` is cleared when the lowering needed an autoload retry or a defer
    /// re-lower — such a result depends on side effects (files loaded mid-lower)
    /// the unit cache cannot replay, so only a first-shot success is cacheable.
    fn lower_unit(
        &mut self,
        name: &[u8],
        src: &[u8],
        pure: &mut bool,
    ) -> Result<Result<Program, crate::LowerError>, PhpError> {
        if self.main_hir.is_none() {
            return Ok(crate::lower_source(name, src));
        }
        // Names whose autoload failed: re-lower with them deferrable, so the
        // affected declarations bind at their execution point (Zend late
        // binding) instead of failing the whole unit — PHP compiles a file
        // whose class extends a missing parent just fine.
        let mut defer: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
        loop {
            match crate::lower_source_seeded(
                name,
                src,
                &self.seed_classes,
                self.seed_static,
                &self.seed_traits,
                &self.seed_globals,
                &self.seed_aliases,
                crate::DeferPolicy::Set(&defer),
            ) {
                Err(crate::LowerError::UndefinedClass { name: pname, kind, line }) => {
                    // An undefined name may be a class/interface *or* a trait used
                    // by a class being lowered; autoload it (which may load a trait
                    // file into `seed_traits`) and retry.
                    *pure = false;
                    if self.resolve_name_autoload(&pname)? {
                        continue;
                    }
                    // Unloadable: mark it deferrable and re-lower. A name that
                    // *stays* unresolved while already deferrable comes from a
                    // non-deferrable site (e.g. a hoisted trait's own `use`) —
                    // that one is the genuine failure.
                    if defer.insert(pname.to_ascii_lowercase()) {
                        continue;
                    }
                    return Ok(Err(crate::LowerError::UndefinedClass {
                        name: pname,
                        kind,
                        line,
                    }));
                }
                other => return Ok(other),
            }
        }
    }

    /// Execute a late-bound declaration ([`crate::hir::DeferredDecl`]) at its
    /// statement/expression site: re-lower its snippet against the *current*
    /// class image (autoloading supertypes that appeared since the unit was
    /// loaded), link and drive it exactly like a mini-include of the original
    /// file. A supertype still missing is PHP's faithful, catchable
    /// `Error: Class|Interface|Trait "X" not found`. For an anonymous-class
    /// expression (`expr = true`) the snippet `return`s the instance and the
    /// caller's scope is bridged so constructor arguments see its variables.
    fn run_deferred(&mut self, idx: usize, expr: bool) -> Result<Zval, PhpError> {
        let caller = self.frames.len() - 1;
        let unit = self.frames[caller].module;
        let dd = &unit.deferred[idx];
        if !expr {
            if self.class_index.contains_key(&dd.name.to_ascii_lowercase()) {
                return Err(PhpError::Error(format!(
                    "Cannot declare {} {}, because the name is already in use",
                    dd.kind_word,
                    String::from_utf8_lossy(&dd.name)
                )));
            }
        }
        let file = unit.file.clone();
        let snippet = dd.snippet.clone();
        let line = dd.line;
        let program = loop {
            match crate::lower_source_seeded(
                &file,
                &snippet,
                &self.seed_classes,
                self.seed_static,
                &self.seed_traits,
                &self.seed_globals,
                &self.seed_aliases,
                crate::DeferPolicy::No,
            ) {
                Ok(p) => break p,
                Err(crate::LowerError::UndefinedClass { name: missing, kind, line: eline }) => {
                    // The supertype may have been loaded (or become autoloadable)
                    // since the unit was lowered — that's the whole point of
                    // binding late. Still missing → the PHP error, at the
                    // declaration's (padded, so original) line.
                    if self.resolve_name_autoload(&missing)? {
                        continue;
                    }
                    self.fatal_line = if eline > 0 { eline } else { line };
                    return Err(PhpError::Error(format!(
                        "{} \"{}\" not found",
                        kind.word(),
                        String::from_utf8_lossy(&missing)
                    )));
                }
                Err(e) => {
                    log::warn!(
                        target: "phpr::include",
                        "deferred decl re-lower failed for {}: {:?}",
                        String::from_utf8_lossy(&file),
                        e
                    );
                    self.fatal_line = line;
                    return Err(PhpError::Error(format!(
                        "require(): Failed to compile '{}'",
                        String::from_utf8_lossy(&file)
                    )));
                }
            }
        };
        self.accumulate_seed(&program);
        let stubs = self.seed_stub_mask(&program);
        let module = match crate::compile::compile_program_stubbed(&program, self.registry, &stubs)
        {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    target: "phpr::include",
                    "deferred decl compile failed for {}: {:?}",
                    String::from_utf8_lossy(&file),
                    e
                );
                self.fatal_line = line;
                return Err(PhpError::Error(format!(
                    "require(): Failed to compile '{}'",
                    String::from_utf8_lossy(&file)
                )));
            }
        };
        // Bridge the calling frame's scope only for the expression form: its
        // constructor arguments are re-evaluated inside the snippet and must
        // see the caller's variables live.
        self.drive_unit(module, None, if expr { Some(caller) } else { None })
    }

    /// Fold a freshly-lowered unit's *new* classes into the accumulating seed image
    /// so later units can reference them (step 57, Phase 3). The unit was lowered
    /// from `seed_classes`, so its `program.classes` is `[seed…, new…]`; the tail
    /// past the current seed length is appended (ids already aligned).
    fn accumulate_seed(&mut self, program: &Program) {
        if self.main_hir.is_none() {
            return;
        }
        let l = self.seed_classes.len();
        if program.classes.len() > l {
            self.seed_classes.extend_from_slice(&program.classes[l..]);
        }
        self.seed_static = program.static_count;
        // Fold the unit's new global variable names into the shared registry and
        // grow the bottom (`main`) frame to cover them, so a `$GLOBALS['new']` /
        // `global $new` this unit introduced addresses a real cell rather than
        // overflowing `main`'s original slot count (step 57). The unit was lowered
        // seeded with `seed_globals`, so `program.slots` is `[seed…, new…]`.
        let g = self.seed_globals.len();
        if program.slots.len() > g {
            self.seed_globals.extend_from_slice(&program.slots[g..]);
            if let Some(main_frame) = self.frames.first_mut() {
                main_frame.slots.resize_with(self.seed_globals.len(), || Zval::Undef);
            }
        }
        // Fold the unit's new traits into the cross-unit trait image (a trait file
        // loaded via autoload makes its trait available to the unit that needed it).
        for (k, t) in &program.traits {
            if !self.seed_traits.iter().any(|(ek, _)| ek == k) {
                self.seed_traits.push((k.clone(), t.clone()));
            }
        }
    }

    /// Mask of `program.classes` indices the running VM already links by name:
    /// `drive_unit` dedups those to their existing global ids, so their (seed)
    /// compilation is pure waste and can be stubbed (`compile_program_stubbed`).
    /// A class declared *conditionally by this unit* always compiles in full
    /// (its `Op::DeclareClass` may need the real thing); a seed conditional
    /// never registered (name absent from `class_index`) also compiles in full,
    /// preserving today's behaviour.
    fn seed_stub_mask(&self, program: &Program) -> Vec<bool> {
        program
            .classes
            .iter()
            .enumerate()
            .map(|(i, cd)| {
                !program.conditional_classes.contains(&i)
                    && self.class_index.contains_key(&cd.name.to_ascii_lowercase())
            })
            .collect()
    }

    /// Fingerprint of the VM state a unit's lowering/compilation/relocation can
    /// observe: the chain of unit-load events so far ([`Vm::unit_chain_fp`])
    /// plus the seed-image / global-table sizes and an order-independent digest
    /// of the registered class ids and aliases. Equal fingerprints (same
    /// binary) ⇒ seeded lowering of the same file bytes replays byte-identically,
    /// so a cached unit ([`CachedUnit`]) is reusable as-is.
    fn unit_fp(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.unit_chain_fp.hash(&mut h);
        self.seed_classes.len().hash(&mut h);
        self.seed_static.hash(&mut h);
        // Trait keys and global names hash IN ORDER: a unit's lowering bakes
        // global SLOT INDICES by position (and trait bodies by key), and
        // runtime code can mint fresh global slots (`global $x`, `$GLOBALS`)
        // in request-dependent order — same count, different layout.
        for (k, _) in &self.seed_traits {
            k.hash(&mut h);
        }
        for g in &self.seed_globals {
            g.hash(&mut h);
        }
        self.classes.len().hash(&mut h);
        self.statics.len().hash(&mut h);
        self.linked_functions.len().hash(&mut h);
        // Registered classes: (name, id) digest, XOR-combined so the HashMap's
        // iteration order cannot leak into the fingerprint.
        let mut acc: u64 = 0;
        for (name, id) in &self.class_index {
            let mut eh = std::collections::hash_map::DefaultHasher::new();
            name.hash(&mut eh);
            id.hash(&mut eh);
            acc ^= eh.finish();
        }
        acc.hash(&mut h);
        let mut aacc: u64 = 0;
        for (alias, target) in &self.seed_aliases {
            let mut eh = std::collections::hash_map::DefaultHasher::new();
            alias.hash(&mut eh);
            target.hash(&mut eh);
            aacc ^= eh.finish();
        }
        aacc.hash(&mut h);
        h.finish()
    }

    /// Resolve an `include`/`require` path to a real file (step 57, Phase 2): an
    /// absolute path is used directly; a relative one is tried against the
    /// including file's directory first, then the process CWD (a minimal
    /// `include_path` — the full search list is a scope-out). Returns the canonical
    /// path, or `None` if no readable file matches.
    fn resolve_include_path(&self, path: &[u8]) -> Option<std::path::PathBuf> {
        use std::os::unix::ffi::OsStrExt;
        let p = std::path::Path::new(std::ffi::OsStr::from_bytes(path));
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if p.is_absolute() {
            candidates.push(p.to_path_buf());
        } else {
            let cur = self.frames.last().map(|f| f.module.file.to_vec()).unwrap_or_default();
            if let Some(dir) = std::path::Path::new(std::ffi::OsStr::from_bytes(&cur)).parent() {
                candidates.push(dir.join(p));
            }
            candidates.push(p.to_path_buf());
        }
        candidates
            .into_iter()
            .find(|c| c.is_file())
            .map(|c| std::fs::canonicalize(&c).unwrap_or(c))
    }

    /// `include`/`require`(`_once`) the file named by `path_val` (step 57, Phase 2).
    /// Loads and runs the file as its own unit via [`Self::drive_unit`], returning
    /// its top-level `return` value (or `int(1)` on fall-through). A `_once` re-load
    /// returns `true` without re-running. A missing/unreadable/unparsable file is a
    /// fatal for `require*` and a warning + `false` for `include*`.
    fn run_include(&mut self, path_val: Zval, mode: IncludeMode) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        let pstr = convert::to_zstr(&path_val, &mut self.diags);
        let path = pstr.as_bytes().to_vec();
        let Some(real) = self.resolve_include_path(&path) else {
            return self.include_open_failed(&path, mode);
        };
        let key = real.as_os_str().as_bytes().to_vec();
        if mode.is_once() && self.included_files.contains(&key) {
            return Ok(Zval::Bool(true));
        }
        // Unit-cache probe: the same file bytes (path+mtime+size), lowered /
        // compiled / relocated under the same VM fingerprint ([`Vm::unit_fp`]),
        // reproduce a byte-identical module — a web server replaying the same
        // include chain over a fresh VM skips lower+compile entirely. The reuse
        // is double-checked structurally (static baseline, recomputed remap);
        // a mismatch simply falls through to the fresh path.
        let unit_key = std::fs::metadata(&real).ok().and_then(|m| unit_key_for(&key, &m));
        let fp = self.unit_fp();
        if let Some(uk) = &unit_key {
            if let Some(cu) = unit_cache_get(uk, fp) {
                let (remap, locals) = self.unit_class_remap(cu.module);
                if cu.static_off == self.statics.len()
                    && remap == cu.class_remap
                    && locals == cu.new_locals
                {
                    self.included_files.insert(key.clone());
                    self.unit_chain_fp = fp_mix_key(self.unit_chain_fp, uk);
                    log::debug!(
                        target: "phpr::include",
                        "{:?} {} (unit cache)",
                        mode,
                        String::from_utf8_lossy(&key)
                    );
                    self.accumulate_seed(&cu.program);
                    let caller = self.frames.len() - 1;
                    let ret = self.run_linked(cu.module, &locals, None, Some(caller))?;
                    return Ok(if matches!(ret, Zval::Null) { Zval::Long(1) } else { ret });
                }
            }
        }
        let mut content = match std::fs::read(&real) {
            Ok(c) => c,
            Err(_) => return self.include_open_failed(&path, mode),
        };
        // The Zend lexer skips a leading `#!` line in *included* files too
        // (oracle-verified on 8.5: `include` of a shebang script outputs no
        // shebang) — Composer's vendor/bin proxies include the real tool
        // binary, shebang and all.
        if content.starts_with(b"#!") {
            let end = content.iter().position(|&b| b == b'\n').map_or(content.len(), |p| p + 1);
            content.drain(..end);
        }
        // Mark loaded before running, so a `_once` re-entry during the file's own
        // execution (mutual includes) sees it and short-circuits.
        self.included_files.insert(key.clone());
        // Fold this load event into the chain exactly as the cache-hit path
        // does, so hit and miss replays keep downstream fingerprints aligned.
        self.unit_chain_fp = match &unit_key {
            Some(uk) => fp_mix_key(self.unit_chain_fp, uk),
            None => fp_mix(self.unit_chain_fp, b"include-nostat", &key),
        };
        log::debug!(target: "phpr::include", "{:?} {}", mode, String::from_utf8_lossy(&key));
        let mut pure = true;
        let program = match self.lower_unit(&key, &content, &mut pure)? {
            Ok(p) => p,
            Err(e) => {
                log::warn!(target: "phpr::include", "lower failed for {}: {:?}", String::from_utf8_lossy(&key), e);
                return self.include_compile_failed(&key, mode);
            }
        };
        let program = Rc::new(program);
        self.accumulate_seed(&program);
        // Classes the VM already links compile as inert stubs — drive_unit dedups
        // them by name anyway; fully recompiling the whole accumulated seed image
        // per included file is quadratic (PHPUnit's preload() = ~1200 requires).
        let stubs = self.seed_stub_mask(&program);
        let mut module = match crate::compile::compile_program_stubbed(&program, self.registry, &stubs) {
            Ok(m) => m,
            Err(e) => {
                log::warn!(target: "phpr::include", "compile failed for {}: {:?}", String::from_utf8_lossy(&key), e);
                return self.include_compile_failed(&key, mode);
            }
        };
        // Link inline (rather than via drive_unit) so the relocated module can
        // be published to the unit cache before it runs.
        let static_off = self.statics.len();
        let (class_remap, new_locals) = self.unit_class_remap(&module);
        relocate_module_class_ids(&mut module, &class_remap, static_off);
        let leaked: &'static Module = Box::leak(Box::new(module));
        if pure && self.main_hir.is_some() {
            if let Some(uk) = unit_key {
                unit_cache_put(
                    uk,
                    CachedUnit {
                        fp,
                        static_off,
                        class_remap,
                        new_locals: new_locals.clone(),
                        program: Rc::clone(&program),
                        module: leaked,
                    },
                );
            }
        }
        // PHP scope rule: an included file shares the *including* scope's variable
        // table — it reads the surrounding variables and the ones it assigns land
        // back in the includer (global scope and function scope alike). Bridged in
        // drive_unit by aliasing the unit frame's named slots to the includer's
        // cells (see `scope_bridge` there).
        let caller = self.frames.len() - 1;
        let ret = self.run_linked(leaked, &new_locals, None, Some(caller))?;
        // A file with no top-level `return` yields int(1); an explicit return passes
        // through (a literal `return null;` is the accepted edge that also yields 1).
        Ok(if matches!(ret, Zval::Null) { Zval::Long(1) } else { ret })
    }

    /// The shared failure path for a file that could not be opened: two warnings
    /// then `false` for `include*`, a stream warning then a fatal for `require*`
    /// (step 57, Phase 2), mirroring PHP's diagnostics. Warnings go through the
    /// deferred `diags` buffer so `@` suppression (and the stamped line) apply.
    fn include_open_failed(&mut self, path: &[u8], mode: IncludeMode) -> Result<Zval, PhpError> {
        let line = self.cur_line(self.frames.len() - 1);
        let pstr = String::from_utf8_lossy(path).into_owned();
        let kw = mode.keyword();
        // The message embeds the CURRENT include_path directive (PHP prints
        // whatever set_include_path() left there; the resolver itself stays
        // cwd-based — see the ini table note).
        let ipath = String::from_utf8_lossy(self.ini.get(b"include_path").unwrap_or(b".:"))
            .into_owned();
        self.diags.push(Diag::Warning(format!(
            "{kw}({pstr}): Failed to open stream: No such file or directory"
        )));
        if !mode.is_require() {
            self.diags.push(Diag::Warning(format!(
                "{kw}(): Failed opening '{pstr}' for inclusion (include_path='{ipath}')"
            )));
        }
        self.flush_diags(line)?;
        if mode.is_require() {
            self.fatal_line = line;
            Err(PhpError::Error(format!(
                "Failed opening required '{pstr}' (include_path='{ipath}')"
            )))
        } else {
            Ok(Zval::Bool(false))
        }
    }

    /// Failure path for a file that opened but failed to parse/compile (step 57,
    /// Phase 2): a fatal regardless of `require`/`include`, as PHP raises a
    /// `ParseError`/compile fatal in the loaded unit.
    fn include_compile_failed(&mut self, name: &[u8], mode: IncludeMode) -> Result<Zval, PhpError> {
        let line = self.cur_line(self.frames.len() - 1);
        self.fatal_line = line;
        Err(PhpError::Error(format!(
            "{}(): Failed to compile '{}'",
            mode.keyword(),
            String::from_utf8_lossy(name)
        )))
    }

    /// Call `recv->method(args)` *synchronously* and return its value, driving a
    /// nested bounded dispatch loop to completion (mirrors `call_callable`). Used
    /// by spread (`[...$traversable]`) where the iterator protocol must be driven
    /// inline inside a single op rather than via the re-entrant `IterNext` state
    /// machine (step 56).
    fn call_method_sync(
        &mut self,
        recv: Zval,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let baseline = self.frames.len();
        self.enter_object_method(recv, method, args, RetMode::Stack)?;
        if self.frames.len() == baseline {
            // A builtin/non-frame method left its result on the caller stack.
            return Ok(self.frames[baseline - 1]
                .stack
                .pop()
                .expect("sync method result on caller stack"));
        }
        self.drive_to_return(baseline)
    }

    /// Operator overloading for `BcMath\Number` (the engine's `do_operation` /
    /// `compare_object` for that class). Returns `Some(result)` when a Number is
    /// involved and the operation applies, else `None` (the caller runs the
    /// standard `apply_binop`, whose "Unsupported operand types" error already
    /// covers array/null/other-object operands). Arithmetic dispatches to
    /// `Number::__op`, comparison to `Number::__cmp`.
    fn try_number_binop(
        &mut self,
        op: BinOp,
        a: &Zval,
        b: &Zval,
    ) -> Result<Option<Zval>, PhpError> {
        use BinOp::*;
        let (recv, is_gmp) = match overload_receiver(a).or_else(|| overload_receiver(b)) {
            Some(r) => r,
            None => return Ok(None),
        };
        if let Some(code) = overload_binop_opcode(op, is_gmp) {
            // GMP always dispatches (its `__op` brands invalid-operand errors);
            // Number dispatches only for numeric-ish operands, else the engine's
            // standard "Unsupported operand types" applies.
            if !is_gmp && !(operand_arith_ok(a) && operand_arith_ok(b)) {
                return Ok(None);
            }
            let r = self.call_method_sync(
                recv,
                b"__op",
                vec![Zval::Long(code), a.clone(), b.clone()],
            )?;
            return Ok(Some(r));
        }
        if matches!(op, Eq | NotEq | Lt | Le | Gt | Ge | Spaceship) {
            if is_gmp || (operand_cmp_ok(a) && operand_cmp_ok(b)) {
                let c = match self.call_method_sync(recv, b"__cmp", vec![a.clone(), b.clone()])? {
                    Zval::Long(n) => n,
                    _ => 0,
                };
                let res = match op {
                    Eq => Zval::Bool(c == 0),
                    NotEq => Zval::Bool(c != 0),
                    Lt => Zval::Bool(c < 0),
                    Le => Zval::Bool(c <= 0),
                    Gt => Zval::Bool(c > 0),
                    Ge => Zval::Bool(c >= 0),
                    Spaceship => Zval::Long(c),
                    _ => unreachable!(),
                };
                return Ok(Some(res));
            }
            // A Number compared with a non-numeric string / array / other-object /
            // resource is UNCOMPARABLE (PHP's compare handler → all relational and
            // `==` false, `!=` true, `<=>` 1). null/bool keep the engine's default
            // type ordering (fall through).
            let other = if number_receiver(a).is_some() { b } else { a };
            if operand_uncomparable(other) {
                let res = match op {
                    NotEq => Zval::Bool(true),
                    Spaceship => Zval::Long(1),
                    _ => Zval::Bool(false),
                };
                return Ok(Some(res));
            }
            return Ok(None);
        }
        Ok(None)
    }

    /// [`apply_binop`] with `BcMath\Number` / `GMP` operator overloading first.
    fn apply_binop_ovl(&mut self, op: BinOp, a: &Zval, b: &Zval) -> Result<Zval, PhpError> {
        if let Some(r) = self.try_number_binop(op, a, b)? {
            return Ok(r);
        }
        apply_binop(op, a, b, &mut self.diags)
    }

    /// [`apply_unop`] with overloading for `BcMath\Number` / `GMP`: unary `-` is
    /// `x * -1` and (GMP only) `~` is `x ^ -1`, routed through `__op`.
    fn apply_unop_ovl(&mut self, op: UnOp, a: &Zval) -> Result<Zval, PhpError> {
        if let Some((recv, is_gmp)) = overload_receiver(a) {
            match op {
                UnOp::Neg => {
                    return self.call_method_sync(
                        recv,
                        b"__op",
                        vec![Zval::Long(2), a.clone(), Zval::Long(-1)],
                    );
                }
                UnOp::BitNot if is_gmp => {
                    return self.call_method_sync(
                        recv,
                        b"__op",
                        vec![Zval::Long(8), a.clone(), Zval::Long(-1)],
                    );
                }
                _ => {}
            }
        }
        apply_unop(op, a, &mut self.diags)
    }

    /// Walk `var_dump`'s argument tree and invoke `__debugInfo()` on every object
    /// that declares it (PHP 8.4), returning object-id → the returned array. The
    /// call runs on the raw receiver — a lazy object is *not* pre-initialized, so
    /// it stays lazy unless the method body touches its own state
    /// (init_trigger_var_dump_debug_info_001/002). Recurses through arrays,
    /// references, the (non-debuggable) object's own slots, and each debug
    /// result, so nested debuggable objects are rendered too. Cycle-guarded by id.
    fn compute_debug_info(
        &mut self,
        args: &[Zval],
    ) -> Result<std::collections::HashMap<u32, Zval>, PhpError> {
        let mut map: std::collections::HashMap<u32, Zval> = std::collections::HashMap::new();
        let mut stack: Vec<Zval> = args.to_vec();
        let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
        // Reference cells and arrays can form cycles without any object in the
        // loop (`$a[] =& $a`; gc_041 dumps a destructor-resurrected graph):
        // walk each shared cell/table once, by pointer identity.
        let mut seen_cells: std::collections::HashSet<usize> = std::collections::HashSet::new();
        while let Some(v) = stack.pop() {
            match &v {
                Zval::Ref(cell) => {
                    if seen_cells.insert(Rc::as_ptr(cell) as usize) {
                        stack.push(cell.borrow().clone());
                    }
                }
                Zval::Array(a) => {
                    if seen_cells.insert(Rc::as_ptr(a) as usize) {
                        for (_, val) in a.iter() {
                            stack.push(val.clone());
                        }
                    }
                }
                Zval::Object(o) => {
                    let (id, cid) = {
                        let b = o.borrow();
                        (b.id, b.class_id as usize)
                    };
                    if !visited.insert(id) {
                        continue;
                    }
                    // Opaque handle classes (GdImage) dump with no properties,
                    // like their internal counterparts: a synthetic empty
                    // debug-info entry hides the prelude's `$__h`.
                    if is_opaque_handle_class(o.borrow().class_name.as_bytes()) {
                        map.insert(id, Zval::Array(Rc::new(PhpArray::new())));
                        continue;
                    }
                    if resolve_method_runtime(&self.classes, cid, b"__debugInfo").is_some() {
                        let res = self.call_method_sync(v.clone(), b"__debugInfo", Vec::new())?;
                        if let Zval::Array(a) = &res {
                            for (_, val) in a.iter() {
                                stack.push(val.clone());
                            }
                        }
                        map.insert(id, res);
                    } else {
                        for (_, val) in o.borrow().props.iter() {
                            stack.push(val.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(map)
    }

    /// Precompute `__toString()` for the Stringable object arguments of a
    /// builtin that *unconditionally* string-coerces them (`natsort`, `implode`,
    /// …), returning object-id → the coerced string. `roots` are the values the
    /// builtin will coerce. With `recurse_arrays`, descends into array arguments
    /// (for `implode`/`str_replace`, whose array elements are each coerced);
    /// otherwise only direct objects (through references) are considered, so an
    /// object a builtin renders as "Array" is not spuriously invoked.
    ///
    /// The walk is **in insertion order** (FIFO), so a user `__toString` with
    /// side effects (e.g. `echo`) runs in the same order PHP coerces the
    /// arguments. Cycle-guarded by id.
    fn compute_stringify(
        &mut self,
        roots: &[Zval],
        recurse_arrays: bool,
    ) -> Result<std::collections::HashMap<u32, php_types::ZStr>, PhpError> {
        // Fast path: string builtins (trim/substr/str_replace/…) are among the
        // hottest calls WP makes and virtually never receive objects — an
        // alloc-free pre-scan skips the queue/map/set machinery entirely
        // (std::HashMap::new() does not allocate until first insert).
        fn any_object(v: &Zval, recurse_arrays: bool, budget: &mut u32) -> bool {
            if *budget == 0 {
                return true; // pathological nesting: fall to the guarded walk
            }
            *budget -= 1;
            match v {
                Zval::Object(_) => true,
                Zval::Ref(c) => any_object(&c.borrow(), recurse_arrays, budget),
                Zval::Array(a) if recurse_arrays => {
                    a.iter().any(|(_, val)| any_object(val, recurse_arrays, budget))
                }
                _ => false,
            }
        }
        let mut budget = 4096u32;
        if !roots.iter().any(|r| any_object(r, recurse_arrays, &mut budget)) {
            return Ok(std::collections::HashMap::new());
        }
        let mut map: std::collections::HashMap<u32, php_types::ZStr> = std::collections::HashMap::new();
        let mut visited: std::collections::HashSet<u32> = std::collections::HashSet::new();
        // Index-driven FIFO queue: process in insertion order, appending
        // referents / array elements to the back as they are discovered.
        let mut queue: Vec<Zval> = roots.to_vec();
        let mut i = 0;
        while i < queue.len() {
            let v = queue[i].clone();
            i += 1;
            match &v {
                Zval::Ref(cell) => queue.push(cell.borrow().clone()),
                Zval::Array(a) if recurse_arrays => {
                    for (_, val) in a.iter() {
                        queue.push(val.clone());
                    }
                }
                Zval::Object(o) => {
                    let (id, cid) = {
                        let b = o.borrow();
                        (b.id, b.class_id as usize)
                    };
                    if !visited.insert(id) {
                        continue;
                    }
                    if resolve_method_runtime(&self.classes, cid, b"__toString").is_some() {
                        let res = self.call_method_sync(v.clone(), b"__toString", Vec::new())?;
                        // PHP requires `__toString` to return a string; the funnel
                        // here string-coerces the (VM-validated) return value.
                        let s = php_types::convert::to_zstr(&res, &mut self.diags);
                        map.insert(id, s);
                    }
                }
                _ => {}
            }
        }
        Ok(map)
    }

    /// Perform the full `Op::PropSet` object-write semantics for a leaf property
    /// write deferred by the field-path walker (`$this->e->foo = v` where `foo`
    /// is not a declared, accessible slot on the target object). Dispatches
    /// `__set` when a magic setter applies, registering the recursion guard
    /// around the synchronous call so a re-entrant write to the *same* property
    /// inside `__set` materialises instead of re-entering (exactly as
    /// `push_magic_prop` arranges for `Op::PropSet`; without it, `$this->e->foo`
    /// assignments inside `e::__set` recurse to a stack overflow). Absent a magic
    /// setter, enforces visibility (`Denied` → private-access error) and
    /// materialises a dynamic property.
    fn prop_set_magic_or_dynamic(
        &mut self,
        target: Zval,
        name: &[u8],
        value: Zval,
        top: usize,
    ) -> Result<(), PhpError> {
        let cur = self.frames[top].class;
        // A write to a lazy object initializes it first, then forwards to the
        // real instance (mirrors Op::PropSet; transitive — a reset instance
        // re-triggers). Magic dispatch and the recursion guard must be
        // evaluated against the *real* object, not the proxy, so an `__set`
        // already in progress on the real instance is not double-invoked
        // (gh21478: `$proxy->x` inside the real instance's `__set`). The walker
        // never runs during `init_props`, so no `init_props` guard is needed.
        let target = self.lazy_prop_access(target, name, cur, Some(true), (MagicKind::Set, b"__set"))?;
        let Some(o) = deref_object(&target) else {
            // The walker only defers object leaves, so a non-object here is
            // unreachable in practice; drop the write rather than panic.
            return Ok(());
        };
        // A `set` hook takes precedence over `__set` and direct write (step 50,
        // mirrors `Op::PropSet`); skipped while the property's own hook is
        // active (a backing write inside the hook). The hook runs to completion
        // synchronously — its return is discarded (surfaced to the drive, not
        // parked in a `ret_cell`, so the bounded run terminates).
        {
            let (oid, ocid) = {
                let b = o.borrow();
                (b.id, b.class_id as usize)
            };
            if !self.hook_guarded(oid, name) {
                if let Some(func) = self.prop_hook(ocid, name, true) {
                    let baseline = self.frames.len();
                    self.push_hook(func, target.clone(), oid, name, Some(value));
                    self.frames.last_mut().expect("hook frame just pushed").ret_cell = None;
                    let _ = self.drive_to_return(baseline)?;
                    return Ok(());
                }
                // A virtual hooked property with no set hook is read-only; a
                // *backed* one without a set hook writes its backing directly
                // (falls through to the plain write below).
                if self.is_virtual_hooked(ocid, name) {
                    return Err(PhpError::Error(format!(
                        "Property {}::${} is read-only",
                        String::from_utf8_lossy(&self.classes[ocid].name),
                        String::from_utf8_lossy(name),
                    )));
                }
            }
        }
        if let Some((_defc, _midx, oid)) =
            self.magic_applies(&o, name, cur, MagicKind::Set, b"__set")
        {
            let gkey = (oid, MagicKind::Set, name.to_vec());
            let inserted = self.magic_guard.insert(gkey.clone());
            let recv = Zval::Object(o);
            let r = self.call_method_sync(
                recv,
                b"__set",
                vec![Zval::Str(PhpStr::new(name.to_vec())), value],
            );
            if inserted {
                self.magic_guard.remove(&gkey);
            }
            r?;
            return Ok(());
        }
        // No magic setter: enforce visibility, then materialise. A `Denied`
        // (private reached from the wrong scope) errors here; otherwise this is a
        // dynamic-property creation (the walker defers only non-declared-slot
        // leaves, so no readonly / typed slot enforcement applies).
        let ocid = o.borrow().class_id as usize;
        check_prop_access(&self.classes, cur, ocid, name)?;
        let key = match resolve_prop_access(&self.classes, ocid, name, cur) {
            PropAccess::Slot(k) => k,
            _ => name.to_vec(),
        };
        if let Zval::Object(o2) = &target {
            self.lazy_ordered_insert(o2, &key);
        }
        // Typed enforcement + through-ref sources, mirroring Op::PropSet (the
        // dynamic-name write path lands here; typed_properties_002).
        let mut value = self.coerce_typed_prop_write(ocid, name, value)?;
        if !self.typed_refs.is_empty() {
            let cell = match o.borrow().props.get(&key) {
                Some(Zval::Ref(c)) => Some(Rc::clone(c)),
                _ => None,
            };
            if let Some(cell) = cell {
                let strict = self.frames.last().map(|f| f.module.strict).unwrap_or(self.module.strict);
                value = self.typed_ref_assign(&cell, value, strict)?;
            }
        }
        if let Some(old) = write_property(&target, &key, value)? {
            self.gc_note(&old);
        }
        Ok(())
    }

    /// Synchronously collect the `(key, value)` pairs of a `Traversable` object
    /// (step 56), used by both spread paths. An `IteratorAggregate` is resolved
    /// via `getIterator()` first (its result may be an array, a Generator, or an
    /// `Iterator` object); an `Iterator` object is driven through
    /// `rewind`/`valid`/`current`/`key`/`next`.
    fn collect_traversable(&mut self, obj: Zval) -> Result<Vec<(Key, Zval)>, PhpError> {
        let cid = object_class_id(&obj).expect("collect_traversable receiver is an object");
        if self.is_aggregate(cid) {
            let inner = self.call_method_sync(obj, b"getIterator", Vec::new())?;
            return self.spread_pairs(inner);
        }
        let mut out = Vec::new();
        self.call_method_sync(obj.clone(), b"rewind", Vec::new())?;
        loop {
            let valid = self.call_method_sync(obj.clone(), b"valid", Vec::new())?;
            if !convert::to_bool(&valid, &mut self.diags) {
                break;
            }
            let val = self.call_method_sync(obj.clone(), b"current", Vec::new())?;
            let key = self.call_method_sync(obj.clone(), b"key", Vec::new())?;
            let k = coerce_key_silent(&key).unwrap_or(Key::Int(0));
            out.push((k, val));
            self.call_method_sync(obj.clone(), b"next", Vec::new())?;
        }
        Ok(out)
    }



    /// Shared collector for `iterator_*`: an array yields its own pairs, a
    /// Generator/Traversable object is driven through the iterator protocol
    /// (delegates to `spread_pairs`, which raises a TypeError for non-iterables).
    fn iter_pairs(&mut self, src: Zval) -> Result<Vec<(Key, Zval)>, PhpError> {
        self.spread_pairs(src)
    }






    /// Build and throw an `AssertionError` carrying `message`.
    fn throw_assertion_error(&mut self, message: &[u8]) -> Result<Zval, PhpError> {
        let msg = String::from_utf8_lossy(message).into_owned();
        match self.class_index.get(&b"assertionerror"[..]).copied() {
            Some(cid) => {
                let obj = self.synthesize_throwable(cid, &msg)?;
                Err(PhpError::Thrown(obj))
            }
            None => Err(PhpError::Error(msg)),
        }
    }

    /// Recursively replace `JsonSerializable` objects with their `jsonSerialize()`
    /// return (itself normalised, so a returned JsonSerializable resolves too) and
    /// normalise array elements. A plain object is left untouched for the pure
    /// encoder to serialise by properties (step 56c).
    /// `visiting` carries the Rc addresses of the arrays/objects on the current
    /// descent path: revisiting one is a cycle — PHP's JSON_ERROR_RECURSION (6),
    /// not a stack overflow.
    fn json_normalize(&mut self, v: Zval, partial: bool, visiting: &mut Vec<usize>) -> Result<Zval, PhpError> {
        match v {
            Zval::Ref(r) => {
                let inner = r.borrow().deref_clone();
                self.json_normalize(inner, partial, visiting)
            }
            Zval::Object(_) => {
                // A lazy object initializes before being encoded (PHP 8.4); a
                // proxy forwards to its real instance. After realize the result is
                // non-lazy, so the re-normalize does not recurse here again.
                if deref_object(&v).is_some_and(|o| o.borrow().lazy.is_some()) {
                    let real = self.realize_full(&v)?;
                    return self.json_normalize(real, partial, visiting);
                }
                let addr = deref_object(&v).map(|o| Rc::as_ptr(&o) as usize).unwrap_or(0);
                if visiting.contains(&addr) {
                    self.json_last_error = 6; // JSON_ERROR_RECURSION
                    if partial {
                        // JSON_PARTIAL_OUTPUT_ON_ERROR: the revisited node
                        // reads as null, encoding continues.
                        return Ok(Zval::Null);
                    }
                    return Err(PhpError::Error("Recursion detected".to_string()));
                }
                let cid = object_class_id(&v).expect("object has a class id");
                // An opaque handle class (GdImage) encodes as `{}` — its hidden
                // handle prop stays invisible.
                if deref_object(&v)
                    .is_some_and(|o| is_opaque_handle_class(o.borrow().class_name.as_bytes()))
                {
                    if let Some(&std_cid) = self.class_index.get(&b"stdclass"[..]) {
                        return self.alloc_object(std_cid);
                    }
                }
                if self.jsonserializable_id.is_some_and(|j| is_instance_of(&self.classes, self.stringable_id, cid, j)) {
                    // A jsonSerialize() that returns the SAME object (directly or
                    // deeper) is encoded by its plain public properties — NOT by
                    // calling jsonSerialize() again, which would never terminate
                    // (json_encode_recursion_02). `json_active` marks the objects
                    // whose jsonSerialize() is currently running.
                    if self.json_active.contains(&addr) {
                        return Ok(v);
                    }
                    self.json_active.push(addr);
                    visiting.push(addr);
                    let r = self.call_method_sync(v, b"jsonSerialize", Vec::new());
                    visiting.pop();
                    let r = match r {
                        Ok(r) => r,
                        Err(e) => {
                            self.json_active.pop();
                            return Err(e);
                        }
                    };
                    let n = self.json_normalize(r, partial, visiting);
                    self.json_active.pop();
                    n
                } else if self
                    .class_index
                    .get(&b"arrayobject"[..])
                    .is_some_and(|&ao| is_instance_of(&self.classes, self.stringable_id, cid, ao))
                {
                    // json_encode(ArrayObject) reads the get_properties handler,
                    // i.e. the backing storage — always as a JSON *object* (a
                    // synthesized stdClass here). Mangled (NUL-prefixed) keys —
                    // private slots the ctor's `(array)` cast copied from an
                    // object backing — are dropped, like PHP's handler never
                    // exposing them (monolog wraps __PHP_Incomplete_Class this
                    // way and expects only the public marker to survive).
                    let storage = deref_object(&v)
                        .and_then(|o| o.borrow().props.get(b"\0ArrayObject\0__storage").cloned())
                        .map(|s| s.deref_clone());
                    visiting.push(addr);
                    let mut entries: Vec<(Vec<u8>, Zval)> = Vec::new();
                    if let Some(Zval::Array(sa)) = storage {
                        for (k, val) in sa.iter() {
                            let kb = match k {
                                Key::Int(i) => i.to_string().into_bytes(),
                                Key::Str(s) => {
                                    if s.as_bytes().first() == Some(&0) {
                                        continue;
                                    }
                                    s.as_bytes().to_vec()
                                }
                            };
                            let nv = self.json_normalize(val.deref_clone(), partial, visiting);
                            match nv {
                                Ok(nv) => entries.push((kb, nv)),
                                Err(e) => {
                                    visiting.pop();
                                    return Err(e);
                                }
                            }
                        }
                    }
                    visiting.pop();
                    let Some(&std_cid) = self.class_index.get(&b"stdclass"[..]) else {
                        return Ok(v);
                    };
                    let obj = self.alloc_object(std_cid)?;
                    if let Zval::Object(o) = &obj {
                        let mut b = o.borrow_mut();
                        for (k, nv) in entries {
                            b.props.set(&k, nv);
                        }
                    }
                    Ok(obj)
                } else if deref_object(&v).is_some_and(|o| o.borrow().info.is_enum_case) {
                    // An enum case serialises as its backing value; a non-backed
                    // enum has no JSON representation (JSON_ERROR_NON_BACKED_ENUM).
                    let backing = deref_object(&v)
                        .and_then(|o| o.borrow().props.get(b"value").cloned());
                    match backing {
                        Some(val) => self.json_normalize(val, partial, visiting),
                        None => {
                            self.json_last_error = 11; // JSON_ERROR_NON_BACKED_ENUM
                            // Unwinds to ho_json_encode, which detects the code 11
                            // and turns it into `false` / a JsonException; the error
                            // value itself is never surfaced.
                            Err(PhpError::Error(
                                "Non-backed enums have no default serialization".to_string(),
                            ))
                        }
                    }
                } else if self.classes[cid].prop_info.values().any(|pi| pi.hooks.is_some()) {
                    // A class with property hooks encodes its PUBLIC properties
                    // through their `get` hooks — including a virtual (get-only,
                    // unbacked) property the raw slot store never holds
                    // (init_trigger_json_encode_hooks). Enumerate as an external
                    // scope, normalize each value, and hand back a stdClass so the
                    // pure formatter emits a JSON object.
                    let (orc, oid) = match deref_object(&v) {
                        Some(o) => {
                            let id = o.borrow().id;
                            (o, id)
                        }
                        None => return Ok(v),
                    };
                    visiting.push(addr);
                    let vars = match self.object_vars_array(&orc, cid, oid, None) {
                        Ok(a) => a,
                        Err(e) => {
                            visiting.pop();
                            return Err(e);
                        }
                    };
                    let entries: Vec<(Vec<u8>, Zval)> = vars
                        .iter()
                        .map(|(k, val)| {
                            let kb = match k {
                                Key::Int(i) => i.to_string().into_bytes(),
                                Key::Str(s) => s.as_bytes().to_vec(),
                            };
                            (kb, val.deref_clone())
                        })
                        .collect();
                    let mut norm: Vec<(Vec<u8>, Zval)> = Vec::new();
                    for (k, val) in entries {
                        match self.json_normalize(val, partial, visiting) {
                            Ok(nv) => norm.push((k, nv)),
                            Err(e) => {
                                visiting.pop();
                                return Err(e);
                            }
                        }
                    }
                    visiting.pop();
                    let Some(&std_cid) = self.class_index.get(&b"stdclass"[..]) else {
                        return Ok(v);
                    };
                    let obj = self.alloc_object(std_cid)?;
                    if let Zval::Object(so) = &obj {
                        let mut b = so.borrow_mut();
                        for (k, nv) in norm {
                            b.props.set(&k, nv);
                        }
                    }
                    Ok(obj)
                } else {
                    Ok(v)
                }
            }
            Zval::Array(a) => {
                let addr = Rc::as_ptr(&a) as usize;
                if visiting.contains(&addr) {
                    self.json_last_error = 6; // JSON_ERROR_RECURSION
                    if partial {
                        return Ok(Zval::Null);
                    }
                    return Err(PhpError::Error("Recursion detected".to_string()));
                }
                visiting.push(addr);
                let entries: Vec<(Key, Zval)> =
                    a.iter().map(|(k, val)| (k.clone(), val.deref_clone())).collect();
                let mut out = PhpArray::new();
                for (k, val) in entries {
                    let nv = self.json_normalize(val, partial, visiting)?;
                    out.insert(k, nv);
                }
                visiting.pop();
                Ok(Zval::Array(Rc::new(out)))
            }
            other => Ok(other),
        }
    }

    /// Issue one object-iterator protocol call from `IterNext`: advance the active
    /// `IterState::Object` to `next` stage, rewind the loop frame's `ip` so
    /// `IterNext` re-runs once the call returns, and enter the method (step 51).
    /// `capture` keeps the return (read back via `pending`); otherwise it is a
    /// `void` call (`rewind`/`next`).
    fn issue_iter_call(
        &mut self,
        top: usize,
        ip: usize,
        method: &[u8],
        args: Vec<Zval>,
        capture: bool,
        next: ObjStage,
    ) -> Result<(), PhpError> {
        let cell = if capture { Some(Rc::new(RefCell::new(Zval::Null))) } else { None };
        let it = {
            let Some(IterState::Object { it, stage, pending, .. }) =
                self.frames[top].iters.last_mut()
            else {
                unreachable!("issue_iter_call without an object iterator");
            };
            *stage = next;
            *pending = cell.clone();
            it.clone()
        };
        self.frames[top].ip = ip; // re-run IterNext after the protocol call returns
        let ret = match cell {
            Some(c) => RetMode::Capture(c),
            None => RetMode::Discard,
        };
        self.enter_object_method(it, method, args, ret)
    }

    /// The receiver of the active object iterator (the `Iterator`/aggregate value).
    fn obj_iter_value(&self, top: usize) -> Zval {
        match self.frames[top].iters.last() {
            Some(IterState::Object { it, .. }) => it.clone(),
            _ => Zval::Null,
        }
    }

    /// Set the active object iterator's stage.
    fn set_obj_stage(&mut self, top: usize, stage: ObjStage) {
        if let Some(IterState::Object { stage: s, .. }) = self.frames[top].iters.last_mut() {
            *s = stage;
        }
    }

    /// Take the value the last protocol call captured into the iterator's `pending`
    /// cell (NULL if absent).
    fn take_obj_pending(&mut self, top: usize) -> Zval {
        let cell = match self.frames[top].iters.last_mut() {
            Some(IterState::Object { pending, .. }) => pending.take(),
            _ => None,
        };
        cell.map(|c| c.borrow().clone()).unwrap_or(Zval::Null)
    }

    /// Pop `n` index values off the current frame, restoring source order.
    fn pop_keys(&mut self, top: usize, n: u32) -> Vec<Zval> {
        let mut keys: Vec<Zval> = (0..n)
            .map(|_| self.frames[top].stack.pop().expect("path index key"))
            .collect();
        keys.reverse();
        keys
    }

    // `dispatch_host_builtin` is generated, together with `host_builtin_canonical`,
    // from the single `host_builtins!` table defined near `host_builtin_canonical`
    // below — adding a host builtin is one edit there, not a two-list sync.









    /// Emit the E_NOTICE PHP raises when an `ob_*` flush/clean operation finds no
    /// active buffer, stamped with the calling line.
    fn ob_no_buffer_notice(&mut self, msg: &str) -> Result<(), PhpError> {
        let line = self.cur_line(self.frames.len() - 1);
        self.raise_diagnostic(8, msg, line)
    }

    /// Write a popped buffer's content to the underlying sink (the next buffer
    /// down, or the real `stdout`/`rendered` streams) as the buffer's *final*
    /// flush. The handler (if any) runs with `PHP_OUTPUT_HANDLER_FINAL` (8), plus
    /// `PHP_OUTPUT_HANDLER_START` (1) when this buffer was never flushed before (so
    /// a plain `ob_start` + `ob_end_flush` yields phase 9, while a buffer already
    /// chunk-flushed yields phase 8). The buffer is already popped, so the parent
    /// write lands in the next sink down.
    fn flush_buffer(&mut self, mut buf: OutputBuffer) -> Result<(), PhpError> {
        let content = std::mem::take(&mut buf.content);
        let phase = if buf.started { 8 } else { 8 | 1 };
        let out = self.apply_ob_callback(&buf.callback, content, phase)?;
        self.write_output(&out)?;
        Ok(())
    }

    /// Discarded-teardown handler pass for a popped buffer: PHP still invokes
    /// the handler when a buffer is destroyed WITHOUT flushing
    /// (`ob_end_clean`/`ob_get_clean`), with `CLEAN|FINAL` (`|START` on first
    /// use) and the handler's return discarded. PHPUnit 13's OutputBuffer
    /// captures the test's output exactly there — skipping the pass made every
    /// `expectOutputString()` assertion compare against "".
    fn clean_buffer(&mut self, mut buf: OutputBuffer) -> Result<(), PhpError> {
        if buf.callback.is_none() {
            return Ok(());
        }
        let content = std::mem::take(&mut buf.content);
        let phase = if buf.started { 2 | 8 } else { 2 | 8 | 1 };
        let _ = self.apply_ob_callback(&buf.callback, content, phase)?;
        Ok(())
    }

    /// Flush every still-active output buffer at request shutdown (PHP implicitly
    /// flushes the buffer stack top-down, each into the next, finally to the SAPI).
    /// A flushing callback that throws is swallowed here (shutdown is past the point
    /// where a fatal can be reported).
    fn flush_all_output_buffers(&mut self) {
        while let Some(buf) = self.ob_stack.pop() {
            let _ = self.flush_buffer(buf);
        }
    }



    /// Convert a parsed [`crate::json::Json`] tree into a `Zval`. Objects build a
    /// `stdClass` (one property per entry) unless `assoc`, in which case they build
    /// a PHP array. Mirrors `eval::json_to_zval`.
    fn vm_json_to_zval(&mut self, j: &crate::json::Json, assoc: bool) -> Result<Zval, PhpError> {
        use crate::json::Json;
        match j {
            Json::Null => Ok(Zval::Null),
            Json::Bool(b) => Ok(Zval::Bool(*b)),
            Json::Long(n) => Ok(Zval::Long(*n)),
            Json::Double(d) => Ok(Zval::Double(*d)),
            Json::Str(s) => Ok(Zval::Str(PhpStr::new(s.clone()))),
            Json::Array(items) => {
                let mut arr = PhpArray::new();
                for item in items {
                    let v = self.vm_json_to_zval(item, assoc)?;
                    let _ = arr.append(v);
                }
                Ok(Zval::Array(Rc::new(arr)))
            }
            Json::Object(entries) => {
                if assoc {
                    let mut arr = PhpArray::new();
                    for (k, v) in entries {
                        let val = self.vm_json_to_zval(v, assoc)?;
                        arr.insert(Key::from_bytes(k), val);
                    }
                    Ok(Zval::Array(Rc::new(arr)))
                } else {
                    let obj = self.alloc_stdclass()?;
                    if let Zval::Object(o) = &obj {
                        for (k, v) in entries {
                            let val = self.vm_json_to_zval(v, assoc)?;
                            o.borrow_mut().props.set(k, val);
                        }
                    }
                    Ok(obj)
                }
            }
        }
    }

    /// Compile an mbregex pattern, pushing a warning diagnostic on failure
    /// (returning `None`). Mirrors `eval::mb_compile`.
    fn mb_compile(&mut self, pat: &[u8], opts: &[u8], func: &str, ic: bool) -> Option<onig::Regex> {
        match crate::mbregex::compile(pat, opts, ic) {
            Ok(re) => Some(re),
            Err(msg) => {
                self.diags
                    .push(php_types::Diag::Warning(format!("{func}(): mbregex compile err: {msg}")));
                None
            }
        }
    }




    // --- mb_ereg family (F2): stateful mbregex matching/search/replace ---

    /// Resolve an optional `$options` value argument at `idx`: the argument when
    /// present and non-null, else the global mbregex options. Mirrors
    /// `eval::mb_opts_arg` (over already-evaluated values).
    fn mb_opts_val(&mut self, args: &[Zval], idx: usize) -> Vec<u8> {
        match args.get(idx) {
            None => self.mb_regex.options.clone(),
            Some(v) => {
                let v = v.deref_clone();
                if matches!(v, Zval::Null) {
                    self.mb_regex.options.clone()
                } else {
                    convert::to_zstr_cast(&v, &mut self.diags).as_bytes().to_vec()
                }
            }
        }
    }





    /// Compile and store the search pattern from an optional `$pattern` value at
    /// `idx` (`$options` follow at `idx + 1`); keeps the existing compiled pattern
    /// when absent/null. Returns false on a compile error. Mirrors
    /// `eval::mb_search_set_pattern`.
    fn mb_search_set_pattern(&mut self, args: &[Zval], idx: usize) -> bool {
        if let Some(p) = args.get(idx) {
            let pv = p.deref_clone();
            if !matches!(pv, Zval::Null) {
                let pat = convert::to_zstr_cast(&pv, &mut self.diags).as_bytes().to_vec();
                let opts = self.mb_opts_val(args, idx + 1);
                match self.mb_compile(&pat, &opts, "mb_ereg_search", false) {
                    Some(re) => self.mb_regex.search_re = Some(re),
                    None => return false,
                }
            }
        }
        true
    }

    /// Run the next search from the cursor, advancing it past the match (by one byte
    /// for a zero-width match) and recording the result for `getregs`. Mirrors
    /// `eval::mb_search_step`.
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







    /// Run `sscanf`/`fscanf` (F2): parse the two fixed value args and return the
    /// per-conversion slots, or `None` for `fscanf` at end-of-file. The variadic
    /// out-param assignment / array-vs-count return is done by the op handler.
    /// Mirrors `eval::ho_sscanf` / `eval::ho_fscanf` (sans `scanf_finish`).
    fn dispatch_host_builtin_scanf(
        &mut self,
        name: &[u8],
        args: Vec<Zval>,
    ) -> Result<Option<Vec<Option<Zval>>>, PhpError> {
        match name {
            b"sscanf" => {
                if args.len() < 2 {
                    return Err(PhpError::ArgumentCountError(
                        "sscanf() expects at least 2 arguments".to_string(),
                    ));
                }
                let input =
                    convert::to_zstr(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
                let fmt =
                    convert::to_zstr(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
                Ok(Some(crate::scanf::run_scanf(&input, &fmt)))
            }
            b"fscanf" => {
                if args.len() < 2 {
                    return Err(PhpError::ArgumentCountError(
                        "fscanf() expects at least 2 arguments".to_string(),
                    ));
                }
                let stream_v = args[0].deref_clone();
                let line = match &stream_v {
                    Zval::Resource(r) => {
                        let mut res = r.borrow_mut();
                        match res.as_stream_mut() {
                            Some(s) => match s.read_line(None) {
                                Ok(Some(l)) => l,
                                _ => return Ok(None), // EOF or read error → false
                            },
                            None => {
                                return Err(PhpError::TypeError(
                                    "fscanf(): Argument #1 ($stream) must be an open stream resource"
                                        .to_string(),
                                ))
                            }
                        }
                    }
                    other => {
                        return Err(PhpError::TypeError(format!(
                            "fscanf(): Argument #1 ($stream) must be of type resource, {} given",
                            other.type_name_for_error()
                        )))
                    }
                };
                let fmt =
                    convert::to_zstr(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
                Ok(Some(crate::scanf::run_scanf(&line, &fmt)))
            }
            _ => Err(undefined_builtin(name)),
        }
    }




    /// Whether `name` is a known constant — a user `define()`, an engine constant,
    /// or a `Class::CONST` class constant (resolved through parents/interfaces).
    fn constant_known(&self, name: &[u8]) -> bool {
        if name.windows(2).any(|w| w == b"::") {
            return self.class_const_ref(name).is_some() || self.enum_case_ref(name).is_some();
        }
        self.constants.contains_key(name) || crate::lower::resolve_constant(name).is_some()
    }

    /// Resolve a `Class::CONST` name (as passed to `constant()` / `defined()`) to
    /// its declaring class id and constant index, walking the parent chain then
    /// interfaces (via `find_const_runtime`). `None` if the name is not
    /// `Class::CONST`, the class is unknown, or the constant is not declared.
    fn class_const_ref(&self, name: &[u8]) -> Option<(ClassId, usize)> {
        let pos = name.windows(2).position(|w| w == b"::")?;
        let cls = &name[..pos];
        let rest = &name[pos + 2..];
        let cls = cls.strip_prefix(b"\\").unwrap_or(cls);
        let cid = *self.class_index.get(&cls.to_ascii_lowercase())?;
        find_const_runtime(&self.classes, cid, rest)
    }

    /// Like [`Self::class_const_ref`] but running the autoloaders on a class
    /// miss first — `defined()`/`constant()` autoload their `Class::CONST`
    /// class, like Zend (http-discovery probes Guzzle via
    /// `defined('GuzzleHttp\ClientInterface::MAJOR_VERSION')` before anything
    /// has loaded the class). A throwing autoloader degrades to not-found.
    fn class_const_class_autoload(&mut self, name: &[u8]) {
        if let Some(pos) = name.windows(2).position(|w| w == b"::") {
            let cls = &name[..pos];
            let cls = cls.strip_prefix(b"\\").unwrap_or(cls);
            let _ = self.resolve_class_autoload(&cls.to_vec());
        }
    }

    /// The index of enum case `name` declared on class `cid`, if any.
    fn enum_case_idx(&self, cid: ClassId, name: &[u8]) -> Option<usize> {
        self.classes[cid].enum_cases.iter().position(|c| c.name.as_ref() == name)
    }

    /// Resolve a `Enum::Case` name to its class id and case index — the enum-case
    /// counterpart of [`class_const_ref`] (enum cases live in `enum_cases`, not
    /// `consts`, but are reachable as `Enum::Case` constants).
    fn enum_case_ref(&self, name: &[u8]) -> Option<(ClassId, usize)> {
        let pos = name.windows(2).position(|w| w == b"::")?;
        let cls = name[..pos].strip_prefix(b"\\").unwrap_or(&name[..pos]);
        let rest = &name[pos + 2..];
        let cid = *self.class_index.get(&cls.to_ascii_lowercase())?;
        self.enum_case_idx(cid, rest).map(|i| (cid, i))
    }

    /// Resolve a callable value to the data the `call_user_func[_array]`
    /// by-reference warning needs: the callee's display name, its per-parameter
    /// by-ref mask, and parameter names. Only user functions/methods/closures
    /// reachable here resolve; builtins and unresolvable shapes return `None`
    /// (they simply get no warning). Mirrors the resolution in [`Self::invoke_value`].
    fn callee_byref_sig(&self, callee: &Zval) -> Option<(String, Vec<bool>, Vec<Vec<u8>>)> {
        let pack = |display: String, f: &Func| {
            (
                display,
                f.param_by_ref.to_vec(),
                f.param_names.iter().map(|n| n.to_vec()).collect(),
            )
        };
        let method_sig = |cid: usize, method: &[u8], display_class: &[u8]| {
            let (rcid, idx) = resolve_method_runtime(&self.classes, cid, method)?;
            let m = &self.classes[rcid].methods[idx];
            let display = format!(
                "{}::{}",
                String::from_utf8_lossy(display_class),
                String::from_utf8_lossy(&m.name)
            );
            Some(pack(display, &m.func))
        };
        match callee {
            Zval::Ref(c) => self.callee_byref_sig(&c.borrow()),
            Zval::Closure(cl) => match &cl.named {
                Some(name) => self.named_byref_sig(name.as_bytes()),
                None => {
                    let f = self.module.closures.get(cl.fn_idx)?;
                    Some(pack(String::from_utf8_lossy(&f.name).into_owned(), f))
                }
            },
            Zval::Str(s) => {
                let b = s.as_bytes();
                if let Some(pos) = b.windows(2).position(|w| w == b"::") {
                    let cls = &b[..pos];
                    let cid = self.class_index.get(&cls.to_ascii_lowercase()).copied()?;
                    method_sig(cid, &b[pos + 2..], &self.classes[cid].name)
                } else {
                    self.named_byref_sig(b)
                }
            }
            Zval::Array(a) => {
                let elems: Vec<Zval> = a.iter().map(|(_, v)| v.deref_clone()).collect();
                if elems.len() != 2 {
                    return None;
                }
                let method = match &elems[1] {
                    Zval::Str(s) => s.as_bytes().to_vec(),
                    _ => return None,
                };
                match &elems[0] {
                    // A closure (or any object) under `[$obj, '__invoke']` resolves to
                    // the invoked body; PHP names it `Closure::__invoke` for a closure.
                    Zval::Closure(cl) if method == b"__invoke" => {
                        let f = match &cl.named {
                            Some(_) => return self.named_byref_sig(b"__invoke").or(None),
                            None => self.module.closures.get(cl.fn_idx)?,
                        };
                        Some(pack("Closure::__invoke".to_string(), f))
                    }
                    Zval::Object(o) => {
                        let cid = o.borrow().class_id as usize;
                        let cname = self.classes[cid].name.clone();
                        method_sig(cid, &method, &cname)
                    }
                    Zval::Str(s) => {
                        let cid = self.class_index.get(&s.as_bytes().to_ascii_lowercase()).copied()?;
                        method_sig(cid, &method, &self.classes[cid].name.clone())
                    }
                    _ => None,
                }
            }
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                let cname = self.classes[cid].name.clone();
                method_sig(cid, b"__invoke", &cname)
            }
            _ => None,
        }
    }

    /// By-ref signature of a named user function (case-insensitive), including
    /// functions declared by a linked eval/include unit. See [`Self::callee_byref_sig`].
    fn named_byref_sig(&self, name: &[u8]) -> Option<(String, Vec<bool>, Vec<Vec<u8>>)> {
        let pack = |f: &Func| {
            (
                String::from_utf8_lossy(&f.name).into_owned(),
                f.param_by_ref.to_vec(),
                f.param_names.iter().map(|n| n.to_vec()).collect(),
            )
        };
        if let Some(f) = self
            .module
            .functions
            .iter()
            .enumerate()
            .find(|(i, f)| !self.module.conditional_fns.contains(i) && name_eq_ignore_case(&f.name, name))
            .map(|(_, f)| f)
        {
            return Some(pack(f));
        }
        let &(fmod, idx) = self.linked_functions.get(&name.to_ascii_lowercase())?;
        Some(pack(&fmod.functions[idx]))
    }

    /// Emit the E_WARNING `call_user_func[_array]` raises for each by-reference
    /// parameter that receives a value: the trampoline forwards arguments by value,
    /// so a by-ref parameter can only bind when the supplied argument was itself a
    /// reference (`arg_is_ref[i]` — always false for `call_user_func`, true for an
    /// `_array` element that is a reference). The argument is then passed by value.
    fn warn_trampoline_byref(
        &mut self,
        callee: &Zval,
        argc: usize,
        arg_is_ref: &[bool],
    ) -> Result<(), PhpError> {
        let Some((display, by_ref, names)) = self.callee_byref_sig(callee) else {
            return Ok(());
        };
        let line = self.cur_line(self.frames.len() - 1);
        for i in 0..argc.min(by_ref.len()) {
            if by_ref[i] && !arg_is_ref.get(i).copied().unwrap_or(false) {
                let msg = match names.get(i) {
                    Some(n) if !n.is_empty() => format!(
                        "{}(): Argument #{} (${}) must be passed by reference, value given",
                        display,
                        i + 1,
                        String::from_utf8_lossy(n)
                    ),
                    _ => format!(
                        "{}(): Argument #{} must be passed by reference, value given",
                        display,
                        i + 1
                    ),
                };
                self.raise_diagnostic(2, &msg, line)?;
            }
        }
        Ok(())
    }





    /// Shared walker for the PHP 8.4 `array_all` / `array_any` / `array_find` /
    /// `array_find_key` quartet: run `$callback(value, key)` over `$fname`'s
    /// array argument until it returns truthy, yielding the first hit (or `None`).
    fn array_search_callback(
        &mut self,
        fname: &str,
        args: &[Zval],
    ) -> Result<Option<(Key, Zval)>, PhpError> {
        let arr = match args.first().map(|a| a.deref_clone()) {
            Some(Zval::Array(a)) => a,
            Some(other) => {
                return Err(PhpError::TypeError(format!(
                    "{fname}(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )));
            }
            None => {
                return Err(PhpError::ArgumentCountError(format!(
                    "{fname}() expects exactly 2 arguments, 0 given"
                )));
            }
        };
        let Some(cb) = args.get(1).map(|c| c.deref_clone()) else {
            return Err(PhpError::ArgumentCountError(format!(
                "{fname}() expects exactly 2 arguments, 1 given"
            )));
        };
        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        for (k, v) in entries {
            let r = self.call_callable(cb.clone(), vec![v.clone(), key_to_zval(&k)])?;
            if convert::to_bool(&r, &mut self.diags) {
                return Ok(Some((k, v)));
            }
        }
        Ok(None)
    }








    /// The unique per-object handle (`#N`) of an object value, shared by
    /// `spl_object_id` and `spl_object_hash`. Objects, closures and generators all
    /// carry a handle; anything else is a `TypeError` (matching PHP's `object`
    /// parameter type).
    fn object_handle_id(v: &Zval, fname: &str) -> Result<u32, PhpError> {
        match v {
            Zval::Object(o) => Ok(o.borrow().id),
            Zval::Closure(c) => Ok(c.id),
            Zval::Generator(g) => Ok(g.borrow().id),
            Zval::Ref(r) => Self::object_handle_id(&r.borrow(), fname),
            other => Err(PhpError::TypeError(format!(
                "{}(): Argument #1 ($object) must be of type object, {} given",
                fname,
                other.type_name_for_error()
            ))),
        }
    }













    /// "object or class-name string" -> [`ClassId`] for `class_parents` /
    /// `class_implements`: an object yields its class; a string is resolved against
    /// the class table, and an unresolved one warns ("Class X does not exist and
    /// could not be loaded") and yields `None` (the caller returns `false`). A
    /// non-object/non-string also yields `None`.
    fn class_arg_or_warn(&mut self, v: Zval, fname: &str) -> Option<ClassId> {
        match v.deref_clone() {
            Zval::Object(o) => Some(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                // These builtins autoload (their `$autoload` param defaults to
                // true) — a swallowed autoloader throw degrades to not-found.
                match self.resolve_class_autoload(name).ok().flatten() {
                    Some(c) => Some(c),
                    None => {
                        self.diags.push(Diag::Warning(format!(
                            "{fname}(): Class {} does not exist and could not be loaded",
                            String::from_utf8_lossy(name)
                        )));
                        None
                    }
                }
            }
            _ => None,
        }
    }

    /// The name of the special engine class an argument denotes, if any:
    /// `Generator` / `Closure` are modelled as bare zvals with no `ClassId`, so
    /// the class-registry lookups (`class_parents` / `class_implements`) miss them
    /// — yet `class_exists` reports them, so these must answer consistently. The
    /// argument is either the class-name string or a live value of that kind.
    fn engine_special_class_name(v: &Zval) -> Option<&'static str> {
        match v {
            Zval::Generator(_) => Some("Generator"),
            Zval::Closure(_) => Some("Closure"),
            Zval::Str(s) => {
                let n = s.as_bytes();
                let n = n.strip_prefix(b"\\").unwrap_or(n);
                if n.eq_ignore_ascii_case(b"Generator") {
                    Some("Generator")
                } else if n.eq_ignore_ascii_case(b"Closure") {
                    Some("Closure")
                } else {
                    None
                }
            }
            _ => None,
        }
    }



    /// Shared body of `is_a` / `is_subclass_of`. Resolves the subject to a
    /// `ClassId` (an object's class always; a class-name string only when
    /// `allow_string`) and the target class name to a `ClassId`, then reuses
    /// `is_instance_of` (which honours the `Stringable` auto-impl and transitive
    /// interfaces). For `is_subclass_of`, `allow_self` is false so the subject's
    /// own class does not count. Unresolved subject/target, or a disallowed
    /// string subject, yields `false` without warning.
    fn is_a_impl(&mut self, args: Vec<Zval>, default_allow_string: bool, allow_self: bool) -> Result<Zval, PhpError> {
        let allow_string = match args.get(2) {
            Some(v) => convert::to_bool(v, &mut self.diags),
            None => default_allow_string,
        };
        let subj_cid = match args.first().cloned().unwrap_or(Zval::Null).deref_clone() {
            Zval::Object(o) => Some(o.borrow().class_id as usize),
            Zval::Str(s) if allow_string => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                // A string subject is looked up WITH autoload (zend_lookup_class),
                // like Zend's is_a_impl — http-discovery probes candidates via
                // `is_subclass_of('GuzzleHttp\Client', ClientInterface::class)`.
                self.resolve_class_autoload(name).ok().flatten()
            }
            _ => None,
        };
        let Some(subj_cid) = subj_cid else { return Ok(Zval::Bool(false)) };
        let target = args.get(1).cloned().unwrap_or(Zval::Null);
        let tbytes = convert::to_zstr_cast(&target, &mut self.diags).as_bytes().to_vec();
        let tname = tbytes.strip_prefix(b"\\").unwrap_or(&tbytes);
        let Some(&tgt_cid) = self.class_index.get(&tname.to_ascii_lowercase()) else {
            return Ok(Zval::Bool(false));
        };
        if !allow_self && subj_cid == tgt_cid {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(is_instance_of(&self.classes, self.stringable_id, subj_cid, tgt_cid)))
    }



    /// Recursively collect an interface's constants (and those of the interfaces
    /// it extends) into `order`, most-derived first — helper for
    /// `__reflect_class_constants`. A name already in `seen` is shadowed.
    fn collect_iface_consts(&self, iface: ClassId, seen: &mut HashSet<Vec<u8>>, order: &mut Vec<(Vec<u8>, ClassId, usize)>) {
        for (i, k) in self.classes[iface].consts.iter().enumerate() {
            let n = k.name.to_vec();
            if seen.insert(n.clone()) {
                order.push((n, iface, i));
            }
        }
        for e in self.classes[iface].interfaces.clone() {
            self.collect_iface_consts(e, seen, order);
        }
    }


    /// Collect every class constant visible on `start`, most-derived first (own +
    /// parent chain, then interfaces transitively; a child redeclaration shadows
    /// the inherited one): `(name, declaring class, index in that class's consts)`.
    fn collect_class_consts(&self, start: ClassId) -> Vec<(Vec<u8>, ClassId, usize)> {
        let mut seen: HashSet<Vec<u8>> = HashSet::default();
        let mut order: Vec<(Vec<u8>, ClassId, usize)> = Vec::new();
        let mut c = Some(start);
        while let Some(x) = c {
            for (i, k) in self.classes[x].consts.iter().enumerate() {
                let n = k.name.to_vec();
                if seen.insert(n.clone()) {
                    order.push((n, x, i));
                }
            }
            c = self.classes[x].parent;
        }
        let mut c = Some(start);
        while let Some(x) = c {
            for iface in self.classes[x].interfaces.clone() {
                self.collect_iface_consts(iface, &mut seen, &mut order);
            }
            c = self.classes[x].parent;
        }
        order
    }





    /// Resolve `($declClass, $name, $index)` from a class-constant attribute handle
    /// to the `&'m Func` thunk at `field` (`new`/`args`), or `None` if absent.
    fn classconst_attr_thunk(&self, args: &[Zval], get_args: bool) -> Option<&'m Func> {
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let name = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => return None };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let &cid = self.class_index.get(&key)?;
        let ci = self.classes[cid].consts.iter().position(|k| k.name.as_ref() == name.as_slice())?;
        let attr = self.classes[cid].consts[ci].attributes.get(idx)?;
        Some(if get_args { &attr.args_thunk } else { &attr.new_thunk })
    }




    /// Resolve a parameter-attribute handle `($class, $func)` to the owning
    /// `&'m Func` and the class context its thunks run in (`None` for a free
    /// function). `$class` empty ⇒ a free function resolved by name; otherwise a
    /// method resolved on that class.
    fn resolve_param_owner(&self, class: &[u8], func: &[u8]) -> Option<(&'m Func, Option<ClassId>)> {
        if class.is_empty() {
            Some((self.find_user_function(func)?, None))
        } else {
            let key = class.strip_prefix(b"\\").unwrap_or(class).to_ascii_lowercase();
            let &cid = self.class_index.get(&key)?;
            let (m, decl, _) = self.find_method_reflect(cid, func)?;
            Some((&m.func, Some(decl)))
        }
    }


    /// Resolve `($class, $func, $pos, $index)` from a parameter-attribute handle to
    /// the `&'m Func` thunk at `field` (`new`/`args`) plus the class context.
    fn param_attr_thunk(&self, args: &[Zval], get_args: bool) -> Option<(&'m Func, Option<ClassId>)> {
        let class = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let func = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let pos = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => return None };
        let idx = match args.get(3) { Some(Zval::Long(i)) => *i as usize, _ => return None };
        let (f, ctx) = self.resolve_param_owner(&class, &func)?;
        let attr = f.param_attributes.get(pos)?.get(idx)?;
        Some((if get_args { &attr.args_thunk } else { &attr.new_thunk }, ctx))
    }






    /// Resolve a user function by name (case-insensitive, leading `\` stripped) to
    /// its [`Func`], searching the running module then any linked (eval/include)
    /// unit. `None` for an unknown function or a builtin (no retained signature).
    fn find_user_function(&self, name: &[u8]) -> Option<&'m Func> {
        let lc = name.strip_prefix(b"\\").unwrap_or(name).to_ascii_lowercase();
        if let Some(f) = self.module.functions.iter().find(|f| f.name.to_ascii_lowercase() == lc) {
            return Some(f);
        }
        self.linked_functions.get(&lc).map(|&(m, idx)| &m.functions[idx])
    }

    /// Resolve a method by walking `cid`'s ancestry; returns the compiled method,
    /// the class that declares it (for `ReflectionMethod::getDeclaringClass`) and
    /// whether it is abstract. Zend copies interface methods into the implementing
    /// class's function table as abstract entries, so after the parent chain
    /// (concrete methods, then own abstract signatures) the interface graph is
    /// searched — an abstract class satisfies `hasMethod`/`getMethod` for a method
    /// it merely inherits from an interface, declared by that interface.
    fn find_method_reflect(&self, cid: ClassId, method: &[u8]) -> Option<(&'m CompiledMethod, ClassId, bool)> {
        // Allocation-free candidate compare: PHPUnit's suite build walks these
        // tables tens of thousands of times (one per ReflectionMethod).
        let lc = method.to_ascii_lowercase();
        let mut ifaces: Vec<ClassId> = Vec::new();
        let mut cur = Some(cid);
        while let Some(c) = cur {
            if let Some(m) = self.classes[c].methods.iter().find(|m| m.name.eq_ignore_ascii_case(&lc)) {
                return Some((m, c, false));
            }
            if let Some(m) =
                self.classes[c].abstract_sigs.iter().find(|m| m.name.eq_ignore_ascii_case(&lc))
            {
                return Some((m, c, true));
            }
            ifaces.extend(self.classes[c].interfaces.iter().copied());
            cur = self.classes[c].parent;
        }
        let mut i = 0;
        while i < ifaces.len() {
            let c = ifaces[i];
            i += 1;
            if let Some(m) =
                self.classes[c].abstract_sigs.iter().find(|m| m.name.eq_ignore_ascii_case(&lc))
            {
                return Some((m, c, true));
            }
            for &n in &self.classes[c].interfaces {
                if !ifaces.contains(&n) {
                    ifaces.push(n);
                }
            }
        }
        None
    }

    /// Run a value thunk (a constant/default `<expr>; Ret`) in an optional class
    /// context and return its value. Mirrors the attribute-thunk path.
    fn run_value_thunk(&mut self, thunk: &'m Func, cur_class: Option<ClassId>) -> Result<Zval, PhpError> {
        let baseline = self.frames.len();
        let module = cur_class.map(|c| self.class_mod(c)).unwrap_or(self.module);
        let mut frame = Frame::new(thunk, module);
        frame.class = cur_class;
        frame.static_class = cur_class;
        self.frames.push(frame);
        // The thunk is speculative: its caller reports a failure through the
        // descriptor (defaultError) instead of a fatal, so a throwable stashed
        // for `render_fatal` while unwinding THIS drive must not stay armed —
        // it would mask a later, unrelated fatal with this stale trace (wp-cli
        // reflects get_wp_details($abspath = ABSPATH) at bootstrap; the stash
        // detonated on `wp option get` long after the Err was consumed).
        let saved = self.uncaught_throwable.take();
        let out = self.drive_to_return(baseline);
        if let Some(cur) = self.uncaught_throwable.take() {
            self.gc_note(&cur);
        }
        self.uncaught_throwable = saved;
        out
    }

    /// Build the signature descriptor array a `ReflectionFunction`/`ReflectionMethod`
    /// is constructed from: `name`, `returnType` and a `params` list (each a
    /// descriptor of name/position/optional/variadic/byref/type and any evaluated
    /// default). `cur_class` is the method's class (for `self::`-typed defaults).
    fn build_func_descriptor(&mut self, func: &'m Func, cur_class: Option<ClassId>) -> Result<php_types::PhpArray, PhpError> {
        let mut params = php_types::PhpArray::new();
        // Owner identity stamped into every parameter descriptor so a
        // `ReflectionParameter` can resolve its attributes back to this callable
        // (empty class name for a free function).
        let owner_class = cur_class.map(|c| self.classes[c].name.to_vec()).unwrap_or_default();
        let owner_func = func.name.to_vec();
        for i in 0..func.n_params as usize {
            let name = func.param_names.get(i).map(|b| b.to_vec()).unwrap_or_default();
            let is_variadic = func.variadic_slot == Some(i as u32);
            let required = func.param_required.get(i).copied().unwrap_or(true);
            let by_ref = func.param_by_ref.get(i).copied().unwrap_or(false);
            let ty = func.param_reflect_types.get(i).and_then(reflect_type_descriptor)
                .unwrap_or_else(|| func.param_hints.get(i).map(typehint_descriptor).unwrap_or(Zval::Bool(false)));
            // A parameter default is evaluated LAZILY in PHP — only when
            // getDefaultValue() is called. Evaluating it here is a convenience, but
            // a default that can't be evaluated (an undefined/namespaced constant,
            // say) must NOT abort building the descriptor and fatal the whole
            // ReflectionParameter construction. On failure, defer the error to
            // getDefaultValue(). `drive_to_return` already unwound the thunk's
            // frames on error, so the VM stack is back at baseline here.
            let (has_default, default_val, default_err) = match func.param_defaults.get(i).and_then(|o| o.as_ref()) {
                Some(thunk) => match self.run_value_thunk(thunk, cur_class) {
                    Ok(v) => (true, v, Zval::Bool(false)),
                    Err(e) => {
                        let m = e.message();
                        let msg = if m.is_empty() { "the parameter default value could not be evaluated" } else { m };
                        (true, Zval::Null, Zval::Str(PhpStr::new(msg.as_bytes().to_vec())))
                    }
                },
                None => (false, Zval::Null, Zval::Bool(false)),
            };
            let mut p = php_types::PhpArray::new();
            let put = |a: &mut php_types::PhpArray, k: &[u8], v: Zval| {
                a.insert(Key::Str(PhpStr::new(k.to_vec())), v);
            };
            put(&mut p, b"name", Zval::Str(PhpStr::new(name)));
            put(&mut p, b"position", Zval::Long(i as i64));
            put(&mut p, b"optional", Zval::Bool(!required || is_variadic));
            put(&mut p, b"variadic", Zval::Bool(is_variadic));
            put(&mut p, b"byref", Zval::Bool(by_ref));
            put(&mut p, b"type", ty);
            put(&mut p, b"hasDefault", Zval::Bool(has_default));
            put(&mut p, b"default", default_val);
            put(&mut p, b"defaultError", default_err);
            let default_const = match func.param_default_const.get(i).and_then(|o| o.as_ref()) {
                Some(name) => Zval::Str(PhpStr::new(name.to_vec())),
                None => Zval::Bool(false),
            };
            put(&mut p, b"defaultConstant", default_const);
            put(&mut p, b"promoted", Zval::Bool(func.param_promoted.get(i).copied().unwrap_or(false)));
            put(&mut p, b"declClass", Zval::Str(PhpStr::new(owner_class.clone())));
            put(&mut p, b"declFunc", Zval::Str(PhpStr::new(owner_func.clone())));
            let _ = params.append(Zval::Array(Rc::new(p)));
        }
        let mut d = php_types::PhpArray::new();
        d.insert(Key::Str(PhpStr::new(b"name".to_vec())), Zval::Str(PhpStr::new(func.name.to_vec())));
        let ret_ty = reflect_type_descriptor(&func.ret_reflect_type).unwrap_or_else(|| typehint_descriptor(&func.ret_hint));
        d.insert(Key::Str(PhpStr::new(b"returnType".to_vec())), ret_ty);
        d.insert(Key::Str(PhpStr::new(b"params".to_vec())), Zval::Array(Rc::new(params)));
        // getDocComment: the retained `/** ... */`, or false like PHP's.
        let doc = match &func.doc {
            Some(d) => Zval::Str(PhpStr::new(d.to_vec())),
            None => Zval::Bool(false),
        };
        d.insert(Key::Str(PhpStr::new(b"doc".to_vec())), doc);
        d.insert(Key::Str(PhpStr::new(b"isGenerator".to_vec())), Zval::Bool(func.is_generator));
        // returnsReference(): the callable's `function &` marker.
        d.insert(Key::Str(PhpStr::new(b"byRef".to_vec())), Zval::Bool(func.by_ref));
        // Source location (getFileName/getStartLine/getEndLine, and the `@@` line of
        // the __toString export). The op line table spans the body; a body-less
        // method (abstract / empty `{}`) has no op lines, so fall back to the
        // declaration line. A prelude ("internal") callable reports false.
        if func.file.as_ref() == b"prelude" || func.file.is_empty() {
            d.insert(Key::Str(PhpStr::new(b"file".to_vec())), Zval::Bool(false));
            d.insert(Key::Str(PhpStr::new(b"startLine".to_vec())), Zval::Bool(false));
            d.insert(Key::Str(PhpStr::new(b"endLine".to_vec())), Zval::Bool(false));
        } else {
            let start = func.lines.iter().copied().filter(|&l| l > 0).min().unwrap_or(func.line);
            // The true closing-brace line when tracked (set from the FnDecl span);
            // otherwise the op-line span's max (imprecise for a body whose last op
            // is not on the closing-brace line).
            let end = if func.end_line > 0 {
                func.end_line
            } else {
                func.lines.iter().copied().filter(|&l| l > 0).max().unwrap_or(func.line)
            };
            d.insert(Key::Str(PhpStr::new(b"file".to_vec())), Zval::Str(PhpStr::new(func.file.to_vec())));
            d.insert(Key::Str(PhpStr::new(b"startLine".to_vec())), Zval::Long(i64::from(start)));
            d.insert(Key::Str(PhpStr::new(b"endLine".to_vec())), Zval::Long(i64::from(end)));
        }
        Ok(d)
    }






    /// The [`Func`] backing a closure value (its body, or the named function /
    /// method a first-class callable wraps) plus the module that owns it (for
    /// running an attribute thunk).
    fn closure_func_mod(&self, cl: &php_types::Closure) -> Option<(&'m Func, &'m Module)> {
        if let Some(name) = &cl.named {
            let nb = name.as_bytes();
            // A method callable carries `Class::method` (see `make_method_closure`):
            // resolve the method's Func so getAttributes()/by-ref param info see
            // the real signature. A magic trampoline stays unresolved (None).
            if let Some(pos) = nb.windows(2).position(|w| w == b"::") {
                let key = nb[..pos].strip_prefix(b"\\").unwrap_or(&nb[..pos]).to_ascii_lowercase();
                let cid = *self.class_index.get(&key[..])?;
                let (m, decl, _) = self.find_method_reflect(cid, &nb[pos + 2..])?;
                return Some((&m.func, self.class_mod(decl)));
            }
            return self.user_function_with_mod(nb);
        }
        let m = self.modules[cl.module_id];
        m.closures.get(cl.fn_idx).map(|f| (f, m))
    }


    /// Run a closure-attribute thunk (`new`/`args`) by `($closure, $index)`.
    fn run_closure_attr(&mut self, args: &[Zval], get_args: bool) -> Result<Zval, PhpError> {
        let Some(clos) = args.first().map(|v| v.deref_clone()) else { return Ok(Zval::Null) };
        let Zval::Closure(cl) = &clos else { return Ok(Zval::Null) };
        let idx = match args.get(1) { Some(Zval::Long(i)) => *i as usize, _ => return Ok(Zval::Null) };
        let Some((func, fmod)) = self.closure_func_mod(cl) else { return Ok(Zval::Null) };
        let Some(attr) = func.attributes.get(idx) else { return Ok(Zval::Null) };
        let thunk = if get_args { &attr.args_thunk } else { &attr.new_thunk };
        if !get_args {
            let attr_name = attr.name.to_vec();
            let siblings: Vec<Vec<u8>> = func.attributes.iter().map(|a| a.name.to_vec()).collect();
            self.validate_attr(&attr_name, &siblings, 2, "function")?;
        }
        let baseline = self.frames.len();
        self.frames.push(Frame::new(thunk, fmod));
        self.drive_to_return(baseline)
    }





















    /// Resolve (or create) the bottom-frame slot for global `$name` at run
    /// time — the `$GLOBALS[$name] = v` dynamic write path. A created slot
    /// joins the cross-unit registry, so later units' `$GLOBALS['name']` and
    /// `global $name` lower onto the same cell.
    fn global_slot_by_name(&mut self, name: &[u8]) -> usize {
        if let Some(i) = self.seed_globals.iter().position(|n| n.as_ref() == name) {
            return i;
        }
        // A main-only run has an empty registry: seed it from the main func's
        // own slot names first so indices line up.
        if self.seed_globals.is_empty() {
            self.seed_globals =
                self.frames[0].func.slot_names.iter().map(|n| n.clone()).collect();
            if let Some(i) = self.seed_globals.iter().position(|n| n.as_ref() == name) {
                return i;
            }
        }
        let i = self.seed_globals.len();
        self.seed_globals.push(name.to_vec().into_boxed_slice());
        if let Some(f) = self.frames.first_mut() {
            f.slots.resize_with(self.seed_globals.len(), || Zval::Undef);
        }
        i
    }















    /// Resolve `($class, $prop, $index)` from a property-attribute handle to the
    /// `&'m Func` thunk at `field` (`new`/`args`), or `None` if absent.
    fn prop_attr_thunk(&self, args: &[Zval], get_args: bool) -> Option<&'m Func> {
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let prop = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return None };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => return None };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let &cid = self.class_index.get(&key)?;
        let attr = self.classes[cid].prop_attributes.get(prop.as_slice())?.get(idx)?;
        Some(if get_args { &attr.args_thunk } else { &attr.new_thunk })
    }




    /// Resolve a user function by name to its [`Func`] *and* the module it lives
    /// in (needed to run an attribute thunk it owns).
    fn user_function_with_mod(&self, name: &[u8]) -> Option<(&'m Func, &'m Module)> {
        let lc = name.strip_prefix(b"\\").unwrap_or(name).to_ascii_lowercase();
        if let Some(f) = self.module.functions.iter().find(|f| f.name.to_ascii_lowercase() == lc) {
            return Some((f, self.module));
        }
        self.linked_functions.get(&lc).map(|&(m, idx)| (&m.functions[idx], m))
    }


    /// Run a function-attribute thunk (`new`/`args`) by `($func, $index)`.
    fn run_func_attr(&mut self, args: &[Zval], get_args: bool) -> Result<Zval, PhpError> {
        let fname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return Ok(Zval::Null) };
        let idx = match args.get(1) { Some(Zval::Long(i)) => *i as usize, _ => return Ok(Zval::Null) };
        let Some((func, fmod)) = self.user_function_with_mod(&fname) else { return Ok(Zval::Null) };
        let Some(attr) = func.attributes.get(idx) else { return Ok(Zval::Null) };
        let thunk = if get_args { &attr.args_thunk } else { &attr.new_thunk };
        if !get_args {
            let attr_name = attr.name.to_vec();
            let siblings: Vec<Vec<u8>> = func.attributes.iter().map(|a| a.name.to_vec()).collect();
            self.validate_attr(&attr_name, &siblings, 2, "function")?;
        }
        let baseline = self.frames.len();
        self.frames.push(Frame::new(thunk, fmod));
        self.drive_to_return(baseline)
    }




    /// Run a method-attribute thunk (`new`/`args`) by `($class, $method, $index)`.
    fn run_method_attr(&mut self, args: &[Zval], get_args: bool) -> Result<Zval, PhpError> {
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return Ok(Zval::Null) };
        let method = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return Ok(Zval::Null) };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => return Ok(Zval::Null) };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Null) };
        // `find_method_reflect` also resolves interface/abstract signatures, so a
        // `#[…]` on a bodyless method instantiates like any other.
        let Some((m, defc, _)) = self.find_method_reflect(cid, &method) else { return Ok(Zval::Null) };
        let Some(attr) = m.func.attributes.get(idx) else { return Ok(Zval::Null) };
        let thunk = if get_args { &attr.args_thunk } else { &attr.new_thunk };
        log::debug!(
            target: "phpr::attr",
            "run_method_attr {}::{} idx {} -> defc {} attr {} (of {}) ops={:?} consts={:?}",
            String::from_utf8_lossy(&cname),
            String::from_utf8_lossy(&method),
            idx,
            defc,
            String::from_utf8_lossy(&attr.name),
            m.func.attributes.len(),
            thunk.ops,
            thunk.consts
        );
        if !get_args {
            let attr_name = attr.name.to_vec();
            let siblings: Vec<Vec<u8>> = m.func.attributes.iter().map(|a| a.name.to_vec()).collect();
            self.validate_attr(&attr_name, &siblings, 4, "method")?;
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(defc));
        frame.class = Some(defc);
        frame.static_class = Some(defc);
        self.frames.push(frame);
        let r = self.drive_to_return(baseline);
        if let Ok(v) = &r {
            log::debug!(
                target: "phpr::attr",
                "run_method_attr result: {}",
                match object_class_id(v) {
                    Some(c) => String::from_utf8_lossy(&self.classes[c].name).into_owned(),
                    None => "non-object".to_string(),
                }
            );
        }
        r
    }





    /// Run a const-attribute thunk (`new`/`args`) by `($const, $index)`.
    fn run_const_attr(&mut self, args: &[Zval], get_args: bool) -> Result<Zval, PhpError> {
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => return Ok(Zval::Null) };
        let idx = match args.get(1) { Some(Zval::Long(i)) => *i as usize, _ => return Ok(Zval::Null) };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_vec();
        let Some(list) = self.module.const_attributes.get(key.as_slice()) else { return Ok(Zval::Null) };
        let Some(attr) = list.get(idx) else { return Ok(Zval::Null) };
        // `newInstance()` validates the attribute's target/repeatability first.
        if !get_args {
            let attr_name = attr.name.to_vec();
            let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
            self.validate_attr(&attr_name, &siblings, 64, "constant")?;
        }
        let attr = self.module.const_attributes.get(key.as_slice()).and_then(|v| v.get(idx)).unwrap();
        let thunk = if get_args { &attr.args_thunk } else { &attr.new_thunk };
        let baseline = self.frames.len();
        self.frames.push(Frame::new(thunk, self.module));
        self.drive_to_return(baseline)
    }




    /// The `#[Attribute(flags)]` flag bitmask declared on attribute class
    /// `attr_class`, by running its retained args thunk (`#[Attribute]` with no
    /// args defaults to `TARGET_ALL = 127`). `None` when the class is *known* but
    /// carries no `#[Attribute]` — i.e. it is being used as an attribute but is
    /// not one (a "non-attribute class" error). An unknown class is permissive
    /// (`Some(127)`): the `new X()` thunk fails on its own with class-not-found.
    fn attr_flags(&mut self, attr_class: &[u8]) -> Option<i64> {
        let lc = attr_class.strip_prefix(b"\\").unwrap_or(attr_class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else { return Some(127) };
        let cc = self.classes[cid];
        let pos = cc.attributes.iter().position(|a| {
            a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(b"Attribute")
        })?;
        let thunk = &cc.attributes[pos].args_thunk;
        let baseline = self.frames.len();
        self.frames.push(Frame::new(thunk, self.class_mod(cid)));
        Some(match self.drive_to_return(baseline) {
            Ok(Zval::Array(a)) => match a.iter().next() {
                Some((_, Zval::Long(n))) => *n,
                _ => 127,
            },
            _ => 127,
        })
    }

    /// Validate that attribute `attr_class` may be instantiated on its target
    /// (PHP 8.4, at `newInstance()` time): the class must exist and be an
    /// attribute class, its `#[Attribute]` flags must allow `target_bit`, and —
    /// without `IS_REPEATABLE` (128) — it must not appear more than once among
    /// `siblings` (the attribute names on the same target).
    fn validate_attr(&mut self, attr_class: &[u8], siblings: &[Vec<u8>], target_bit: i64, target_label: &str) -> Result<(), PhpError> {
        let bare = attr_class.strip_prefix(b"\\").unwrap_or(attr_class);
        let name = String::from_utf8_lossy(bare).into_owned();
        // A miss runs the autoloaders first: PHP's newInstance() resolves the
        // attribute class through zend_lookup_class (PHPUnit's DataProvider &
        // co. are only ever loaded this way).
        if self.resolve_class_autoload(&bare.to_vec())?.is_none() {
            return Err(PhpError::Error(format!("Attribute class \"{}\" not found", name)));
        }
        let Some(flags) = self.attr_flags(attr_class) else {
            return Err(PhpError::Error(format!(
                "Attempting to use non-attribute class \"{}\" as attribute", name
            )));
        };
        let count = siblings.iter().filter(|s| s.strip_prefix(b"\\").unwrap_or(s).eq_ignore_ascii_case(bare)).count();
        if count > 1 && (flags & 128) == 0 {
            return Err(PhpError::Error(format!("Attribute \"{}\" must not be repeated", name)));
        }
        if (flags & target_bit) == 0 {
            return Err(PhpError::Error(format!(
                "Attribute \"{}\" cannot target {} (allowed targets: {})",
                name, target_label, allowed_targets_str(flags)
            )));
        }
        Ok(())
    }

    /// Resolve the class for a per-property lazy op (skip / raw-set) and check
    /// `prop` is eligible: a declared, non-static, non-virtual instance property.
    /// `Ok(cid)` when usable; `Err(msg)` is the `ReflectionException` message the
    /// PHP wrapper throws (naming `op`, the public method name).
    fn lazy_prop_target(&mut self, class: &Zval, prop: &[u8], op: &str) -> Result<ClassId, String> {
        let raw = convert::to_zstr_cast(class, &mut self.diags).as_bytes().to_vec();
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = match self.class_index.get(&key) {
            Some(&c) => c,
            None => return Err(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw))),
        };
        if self.is_lazy_eligible_prop(cid, prop) {
            return Ok(cid);
        }
        let cc = self.classes[cid];
        let cname = String::from_utf8_lossy(&cc.name).into_owned();
        let pname = String::from_utf8_lossy(prop).into_owned();
        if cc.static_props.iter().any(|sp| sp.name.as_ref() == prop) {
            Err(format!("Can not use {op} on static property {cname}::${pname}"))
        } else if cc.prop_info.get(prop).and_then(|pi| pi.hooks.as_ref()).is_some_and(|h| !h.backed) {
            Err(format!("Can not use {op} on virtual property {cname}::${pname}"))
        } else {
            Err(format!("Can not use {op} on dynamic property {cname}::${pname}"))
        }
    }




    /// Resolve an attribute handle (`$class` name + `$index`) to its
    /// `(ClassId, index)`, bounds-checked. Shared by `newInstance`/`getArguments`.
    fn reflect_attr_handle(&self, args: &[Zval]) -> Result<(ClassId, usize), PhpError> {
        let cname = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Err(PhpError::Error("ReflectionAttribute: invalid handle".to_string())),
        };
        let idx = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) if n >= 0 => n as usize,
            _ => return Err(PhpError::Error("ReflectionAttribute: invalid handle".to_string())),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let cid = self
            .class_index
            .get(&key)
            .copied()
            .ok_or_else(|| PhpError::Error("ReflectionAttribute: class missing".to_string()))?;
        if idx >= self.classes[cid].attributes.len() {
            return Err(PhpError::Error("ReflectionAttribute: index out of range".to_string()));
        }
        Ok((cid, idx))
    }




    /// Add interface `i` and the interfaces it extends to `arr` (name => name),
    /// transitively, skipping duplicates (helper for `class_implements`).
    fn collect_iface(&self, i: ClassId, arr: &mut php_types::PhpArray, seen: &mut HashSet<ClassId>) {
        if !seen.insert(i) {
            return;
        }
        let name = self.classes[i].name.to_vec();
        arr.insert(Key::Str(PhpStr::new(name.clone())), Zval::Str(PhpStr::new(name)));
        let parents = self.classes[i].interfaces.clone();
        for p in parents {
            self.collect_iface(p, arr, seen);
        }
    }

    /// Resolve an "object or class-name string" argument to a [`ClassId`], matching
    /// PHP 8.5's `TypeError` for an unresolvable string or a non-object/non-string
    /// value (shared by `get_parent_class` / `get_class_methods`, Session B2).
    fn class_arg_to_id(&mut self, v: Zval, fname: &str) -> Result<ClassId, PhpError> {
        match v {
            Zval::Object(o) => Ok(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw).to_vec();
                // A class-name string triggers autoload, like PHP's zend_lookup_class.
                self.resolve_class_autoload(&name)?.ok_or_else(|| {
                    PhpError::TypeError(format!(
                        "{fname}(): Argument #1 ($object_or_class) must be an object or a valid class name, string given"
                    ))
                })
            }
            Zval::Ref(r) => self.class_arg_to_id(r.borrow().clone(), fname),
            other => Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($object_or_class) must be an object or a valid class name, {} given",
                other.type_name_for_error()
            ))),
        }
    }

    /// `get_object_vars($object)` (Session B2): the object's properties as a
    /// `name => value` array, filtered by visibility from the calling class scope —
    /// from outside only `public`, from within the class the `protected`/`private`
    /// ones too. Dynamic properties are public. Insertion order is preserved.
    /// Mirrors `eval::ci_get_object_vars`.
    /// The entries a `foreach` over a plain (non-Traversable) object can yield,
    /// in iteration order, as `(display name, how-to-read)` pairs: declared
    /// properties first (parent-first declaration order, a redeclaration keeping
    /// its position), then dynamic ones in insertion order. Visibility follows
    /// [`resolve_prop_access`] from `scope`. A hooked property with a get hook
    /// yields a [`PropIterEntry::Hook`] (backed or virtual — the hook runs at
    /// the step, PHP 8.4); a set-only *virtual* property is unreadable and
    /// skipped; everything else is a [`PropIterEntry::Slot`] read, skipped when
    /// absent or uninitialized. Recomputed at each `IterNext`, so the loop
    /// observes live additions/removals (PHP's property-hash cursor).
    fn object_iter_entries(
        &self,
        o: &Rc<RefCell<Object>>,
        scope: Option<ClassId>,
    ) -> Vec<(Box<[u8]>, PropIterEntry)> {
        let cid = o.borrow().class_id as usize;
        let mut chain: Vec<usize> = Vec::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            chain.push(ci);
            c = self.classes[ci].parent;
        }
        let mut order: Vec<Box<[u8]>> = Vec::new();
        let mut declared: HashSet<Box<[u8]>> = HashSet::default();
        for ci in chain.iter().rev() {
            for (name, _) in &self.classes[*ci].own_prop_vis {
                if declared.insert(name.clone()) {
                    order.push(name.clone());
                }
            }
        }
        let b = o.borrow();
        let mut out: Vec<(Box<[u8]>, PropIterEntry)> = Vec::new();
        for name in &order {
            // The scope's view of the declared name (see `resolve_prop_access`):
            // the resolved storage key, a plain dynamic slot, or skipped.
            let key: Box<[u8]> = match resolve_prop_access(&self.classes, cid, name, scope) {
                PropAccess::Slot(k) => k.into(),
                PropAccess::Dynamic => name.clone(),
                PropAccess::Denied { .. } => continue,
            };
            // The hook view: the scope's own *private* declaration shadows a
            // subclass redeclaration (mirrors `resolve_prop_access` case 1);
            // otherwise the object's flattened (most-derived) table decides.
            let view = match scope {
                Some(s)
                    if prop_info(&self.classes, s, name).is_some_and(|pi| {
                        pi.visibility == Visibility::Private
                            && pi.declaring_class == s
                            && class_is_a(&self.classes, cid, s)
                    }) =>
                {
                    s
                }
                _ => cid,
            };
            if let Some(h) =
                self.classes[view].prop_info.get(name.as_ref()).and_then(|pi| pi.hooks.as_ref())
            {
                if h.get.is_some() {
                    out.push((name.clone(), PropIterEntry::Hook { name: name.clone(), view }));
                    continue;
                }
                if !h.backed {
                    // A set-only virtual property is not readable: skipped.
                    continue;
                }
                // A set-only backed property reads its backing store directly.
            }
            // Skip a slot that is absent or uninitialised (typed-no-default
            // `Undef`); both are absent from a foreach view.
            if !matches!(b.props.get(&key), None | Some(Zval::Undef)) {
                out.push((name.clone(), PropIterEntry::Slot { key }));
            }
        }
        // Dynamic (undeclared) properties, always public, in insertion order.
        // Mangled (private) slots are never dynamic.
        for (name, _) in b.props.iter() {
            if !declared.contains(name) && !name.starts_with(b"\0") {
                let n: Box<[u8]> = name.to_vec().into_boxed_slice();
                out.push((n.clone(), PropIterEntry::Slot { key: n }));
            }
        }
        out
    }

    /// The compiled get hook of property `name` as resolved from class `view`'s
    /// flattened `prop_info` — like [`Self::prop_hook`], with an explicit start
    /// class (the scope-private shadow case of `object_iter_entries`).
    fn prop_hook_in(&self, view: ClassId, name: &[u8]) -> Option<&'m Func> {
        self.classes[view].prop_info.get(name)?.hooks.as_ref()?.get.as_ref()
    }

    /// Run property `name`'s get hook (resolved from `view`) on `obj`, driving
    /// the hook frame to its return synchronously. `deref = true` keeps the
    /// by-value contract of a foreach value step; by-ref binding passes `false`
    /// to receive the cell a `&get` hook returns.
    fn run_iter_get_hook(
        &mut self,
        obj: &Zval,
        name: &[u8],
        view: ClassId,
        deref: bool,
    ) -> Result<Zval, PhpError> {
        let Some(func) = self.prop_hook_in(view, name) else { return Ok(Zval::Null) };
        let oid = deref_object(obj).map(|o| o.borrow().id).unwrap_or(0);
        let baseline = self.frames.len();
        self.push_hook(func, obj.clone(), oid, name, None);
        if !deref {
            self.frames.last_mut().expect("hook frame just pushed").ret_deref = false;
        }
        self.drive_to_return(baseline)
    }


    /// Enumerate an object's properties as `name => value`, in PHP's declared-then-
    /// dynamic order, filtered by the visibility visible from `cur` (`None` = an
    /// external scope, i.e. public only). Hooked/virtual properties surface through
    /// their `get` hook (an uninitialized typed property is omitted). Shared by
    /// `get_object_vars` and json-encoding a hooked object (json passes `None`).
    fn object_vars_array(
        &mut self,
        o: &Rc<RefCell<Object>>,
        cid: usize,
        oid: u32,
        cur: Option<ClassId>,
    ) -> Result<PhpArray, PhpError> {
        // Declared-property order, parent-first (root → derived), de-duplicated so
        // a redeclared property keeps its inherited position. `own_prop_vis`
        // includes *virtual* hooked properties (which have no instance storage),
        // unlike the instance prop store — so iterating it lets a `get`-only
        // hooked property surface (step 56c, GH-16725).
        let mut chain: Vec<usize> = Vec::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            chain.push(ci);
            c = self.classes[ci].parent;
        }
        let mut order: Vec<Box<[u8]>> = Vec::new();
        let mut declared: HashSet<Box<[u8]>> = HashSet::default();
        for ci in chain.iter().rev() {
            for (name, _) in &self.classes[*ci].own_prop_vis {
                if declared.insert(name.clone()) {
                    order.push(name.clone());
                }
            }
        }
        let mut arr = PhpArray::new();
        for name in &order {
            // The scope's view of the declared name: its resolved slot, a plain
            // dynamic slot (a parent's private is invisible here), or denied.
            let slot = match resolve_prop_access(&self.classes, cid, name, cur) {
                PropAccess::Slot(k) => Some(k),
                PropAccess::Dynamic => None,
                PropAccess::Denied { .. } => continue,
            };
            // A hooked property (backed or virtual) surfaces through its `get`
            // hook; a plain property reads from the instance store. An
            // uninitialised typed property (neither hooked nor stored) is omitted.
            if let Some(func) = slot.is_some().then(|| self.prop_hook(cid, name, false)).flatten() {
                let baseline = self.frames.len();
                self.push_hook(func, Zval::Object(o.clone()), oid, name, None);
                let val = self.drive_to_return(baseline)?;
                arr.insert(Key::from_bytes(name), val);
            } else if let Some(val) = o.borrow().props.get(slot.as_deref().unwrap_or(name)).cloned() {
                // An uninitialized typed property (`Undef`) is omitted; the entry
                // surfaces under its source-level name, whatever the storage key.
                if !matches!(val, Zval::Undef) {
                    arr.insert(Key::from_bytes(name), val);
                }
            }
        }
        // Dynamic (undeclared) properties keep instance order, after the declared
        // set; they are always public. Mangled (private) slots are not dynamic —
        // they were either surfaced above or are invisible to this scope.
        let dynamic: Vec<(Box<[u8]>, Zval)> = {
            let b = o.borrow();
            b.props
                .iter()
                .filter(|(name, _)| !declared.contains(*name) && !name.starts_with(b"\0"))
                .map(|(name, val)| (name.to_vec().into_boxed_slice(), val.clone()))
                .collect()
        };
        for (name, val) in dynamic {
            arr.insert(Key::from_bytes(&name), val);
        }
        Ok(arr)
    }









    /// Resolve `args[0]` as a class name for the `*_exists` family (step 57, Phase
    /// 3): when `args[1]` (the `$autoload` flag, default `true`) is set, a miss runs
    /// the registered autoloaders before the final lookup, so `class_exists("X")`
    /// triggers loading exactly as PHP does.
    fn resolve_named_class_with_autoload(
        &mut self,
        args: &[Zval],
    ) -> Result<Option<ClassId>, PhpError> {
        let Some(a) = args.first() else { return Ok(None) };
        let autoload = args.get(1).is_none_or(|v| convert::to_bool(v, &mut self.diags));
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let name = b.strip_prefix(b"\\").unwrap_or(b).to_vec();
        if autoload {
            self.resolve_class_autoload(&name)
        } else {
            Ok(self.class_index.get(&name.to_ascii_lowercase()).copied())
        }
    }













    /// Dispatch a host builtin with a by-reference output parameter (Session:
    /// out-param). Returns `(result, out_value, out_value2)`; the VM writes
    /// `out_value` into the caller's first out-param slot and `out_value2` (when
    /// `Some`) into the second (`exec`'s `&$result_code`). `_out_index` is the
    /// argument position of the primary out-param (kept for symmetry).
    fn dispatch_host_builtin_out(
        &mut self,
        name: &[u8],
        args: Vec<Zval>,
        _out_index: usize,
    ) -> Result<(Zval, Zval, Option<Zval>), PhpError> {
        let (result, out) = match name {
            b"preg_match" => self.ho_preg_match(args)?,
            b"preg_match_all" => self.ho_preg_match_all(args)?,
            b"mb_ereg" => self.ho_mb_ereg(false, args)?,
            b"mb_eregi" => self.ho_mb_ereg(true, args)?,
            b"preg_replace" => self.ho_preg_replace(args)?,
            b"preg_replace_callback" => self.ho_preg_replace_callback_out(args)?,
            b"flock" => self.ho_flock_out(args)?,
            b"proc_open" => self.ho_proc_open(args)?,
            b"pcntl_sigprocmask" => self.ho_pcntl_sigprocmask(args)?,
            b"parse_str" => self.ho_parse_str(args)?,
            b"system" => self.ho_system(args)?,
            b"passthru" => self.ho_passthru(args)?,
            b"grapheme_extract" => self.ho_grapheme_extract(args)?,
            b"similar_text" => self.ho_similar_text(args)?,
            b"str_replace" => self.ho_str_replace_out(args, false)?,
            b"str_ireplace" => self.ho_str_replace_out(args, true)?,
            b"getimagesize" => self.ho_getimagesize_out(args, false)?,
            b"getimagesizefromstring" => self.ho_getimagesize_out(args, true)?,
            b"getopt" => self.ho_getopt_out(args)?,
            b"is_callable" => self.ho_is_callable_out(args)?,
            // Two out-params: (result, $output array, Some($result_code)).
            b"exec" => return self.ho_exec(args),
            // Two out-params: (stream|false, $error_code, Some($error_message)).
            b"stream_socket_client" => return self.ho_stream_socket_client(args),
            // Two out-params: (bool, $filename, Some($line)).
            b"headers_sent" => return self.ho_headers_sent_out(args),
            _ => return Err(undefined_builtin(name)),
        };
        Ok((result, out, None))
    }











    /// Whether an object implements the named (lowercased) interface, via its
    /// full class/interface chain.
    pub(super) fn object_implements(&self, v: &Zval, iface_lc: &[u8]) -> bool {
        let Some(cid) = object_class_id(v) else { return false };
        self.class_index
            .get(iface_lc)
            .is_some_and(|&t| is_instance_of(&self.classes, self.stringable_id, cid, t))
    }

    /// Whether an object implements Traversable (directly or via its
    /// class/interface chain) — the `yield from` delegate check.
    fn object_is_traversable(&self, v: &Zval) -> bool {
        self.object_implements(v, b"traversable")
    }

    /// Resolve a Traversable object for `yield from`: IteratorAggregate chains
    /// unwrap through `getIterator()` (a Generator result is delegated live);
    /// a plain Iterator is drained through the protocol into (key, value)
    /// entries.
    fn traversable_entries(&mut self, mut it: Zval) -> Result<TraversableSource, PhpError> {
        // Unwrap IteratorAggregate (bounded: a cycle of getIterator()s errors).
        for _ in 0..16 {
            let Some(cid) = object_class_id(&it) else { break };
            let is_agg = self
                .class_index
                .get(&b"iteratoraggregate"[..])
                .is_some_and(|&a| is_instance_of(&self.classes, self.stringable_id, cid, a));
            if !is_agg {
                break;
            }
            let inner = self.call_method_sync(it.clone(), b"getIterator", Vec::new())?;
            match inner.deref_clone() {
                Zval::Generator(rc) => return Ok(TraversableSource::Gen(rc)),
                other @ Zval::Object(_) => it = other,
                Zval::Array(a) => {
                    let entries = a.iter().map(|(k, v)| (key_to_zval(k), v.deref_clone())).collect();
                    return Ok(TraversableSource::Entries(entries));
                }
                other => {
                    return Err(PhpError::Error(format!(
                        "getIterator() must return an object implementing Traversable, {} returned",
                        other.type_name_for_error()
                    )))
                }
            }
        }
        // Drive the Iterator protocol to exhaustion.
        let mut entries = Vec::new();
        self.call_method_sync(it.clone(), b"rewind", Vec::new())?;
        loop {
            let valid = self.call_method_sync(it.clone(), b"valid", Vec::new())?;
            if !convert::is_true_silent(&valid.deref_clone()) {
                break;
            }
            let key = self.call_method_sync(it.clone(), b"key", Vec::new())?.deref_clone();
            let val = self.call_method_sync(it.clone(), b"current", Vec::new())?.deref_clone();
            entries.push((key, val));
            self.call_method_sync(it.clone(), b"next", Vec::new())?;
        }
        Ok(TraversableSource::Entries(entries))
    }

    /// Whether the value graph contains an object with a `__serialize`/`__sleep`
    /// hook — the fast-path gate for [`Self::ho_serialize`]: a hook-free graph
    /// (the overwhelmingly common case) passes to the pure serializer untouched,
    /// keeping shared-object identity intact. `seen` breaks cycles.
    fn has_serialize_hooks(&self, v: &Zval, seen: &mut Vec<usize>) -> bool {
        match v {
            Zval::Ref(r) => {
                let addr = Rc::as_ptr(r) as usize;
                if seen.contains(&addr) {
                    return false;
                }
                seen.push(addr);
                self.has_serialize_hooks(&r.borrow(), seen)
            }
            Zval::Array(a) => a.iter().any(|(_, val)| self.has_serialize_hooks(val, seen)),
            Zval::Object(orc) => {
                let addr = Rc::as_ptr(orc) as usize;
                if seen.contains(&addr) {
                    return false;
                }
                seen.push(addr);
                // A lazy object must go through the VM-side prepare pass — it
                // initializes before serialization (PHP 8.4) — even when no
                // serialize hook exists anywhere in the graph.
                if orc.borrow().lazy.is_some() {
                    return true;
                }
                if let Some(cid) = object_class_id(v) {
                    if resolve_method_runtime(&self.classes, cid, b"__serialize").is_some()
                        || resolve_method_runtime(&self.classes, cid, b"__sleep").is_some()
                        || self.class_index.get(&b"serializable"[..]).is_some_and(|&ser| {
                            is_instance_of(&self.classes, self.stringable_id, cid, ser)
                        })
                    {
                        return true;
                    }
                }
                let vals: Vec<Zval> =
                    orc.borrow().props.iter().map(|(_, val)| val.clone()).collect();
                vals.iter().any(|val| self.has_serialize_hooks(val, seen))
            }
            _ => false,
        }
    }

    /// Rewrite an object graph for serialization, running `__serialize` (data
    /// array becomes the payload) / `__sleep` (side effects + prop filter) and
    /// substituting synthetic same-class objects the pure serializer then
    /// formats. `memo` keeps one synthetic per source object (shared subtrees /
    /// cycles map to the same rewrite).
    fn prepare_serialize(
        &mut self,
        v: Zval,
        memo: &mut HashMap<usize, Zval>,
    ) -> Result<Zval, PhpError> {
        match v {
            Zval::Ref(r) => {
                let inner = r.borrow().clone();
                self.prepare_serialize(inner, memo)
            }
            Zval::Array(a) => {
                let mut out = PhpArray::new();
                for (k, val) in a.iter() {
                    out.insert(k.clone(), self.prepare_serialize(val.clone(), memo)?);
                }
                Ok(Zval::Array(Rc::new(out)))
            }
            Zval::Object(ref orc) => {
                // A lazy object's serialization (PHP 8.4). `__serialize` (the
                // modern hook) always runs on the *raw* wrapper — no pre-init —
                // and initializes only if its body observes object state
                // (serialize___serialize_may_not_initialize). `__sleep` (legacy)
                // pre-initializes by DEFAULT (serialize___sleep_initializes) —
                // UNLESS SKIP_INITIALIZATION_ON_SERIALIZE (8) is set, which runs
                // it raw so it initializes only on state access
                // (serialize___sleep_skip_flag vs …_may_initialize). A hook-free
                // lazy object needs its real props, so it initializes now (a
                // proxy forwards to its instance) unless the skip flag serializes
                // the raw (empty) view.
                if orc.borrow().lazy.is_some() {
                    let (oid, ocid, uninit) = {
                        let b = orc.borrow();
                        (b.id, b.class_id as usize, b.proxy_instance.is_none())
                    };
                    let has_serialize =
                        resolve_method_runtime(&self.classes, ocid, b"__serialize").is_some();
                    let skip_flag =
                        uninit && self.lazy_options.get(&oid).is_some_and(|f| f & 8 != 0);
                    if !has_serialize && !skip_flag {
                        let real = self.realize_full(&v)?;
                        return self.prepare_serialize(real, memo);
                    }
                }
                let addr = Rc::as_ptr(orc) as usize;
                if let Some(done) = memo.get(&addr) {
                    return Ok(done.clone());
                }
                let cid = object_class_id(&v);
                // Legacy `Serializable` (priority: below `__serialize`, above
                // `__sleep`): `->serialize()` yields the raw payload, staged in
                // a marker record the pure formatter emits as
                // `C:<len>:"<Class>":<len>:{<payload>}` (or `N;` on null).
                if let Some(c) = cid.filter(|&c| {
                    resolve_method_runtime(&self.classes, c, b"__serialize").is_none()
                        && self.class_index.get(&b"serializable"[..]).is_some_and(|&ser| {
                            is_instance_of(&self.classes, self.stringable_id, c, ser)
                        })
                }) {
                    let ret = self
                        .call_method_sync(v.clone(), b"serialize", Vec::new())?
                        .deref_clone();
                    let payload = match ret {
                        Zval::Str(s) => Some(Zval::Str(s)),
                        Zval::Null | Zval::Undef => None,
                        _ => {
                            return Err(PhpError::TypeError(format!(
                                "{}::serialize() must return a string or NULL",
                                String::from_utf8_lossy(self.classes[c].name.as_ref())
                            )))
                        }
                    };
                    let mut props = Props::new();
                    props.set(
                        b"class",
                        Zval::Str(PhpStr::new(orc.borrow().class_name.as_bytes().to_vec())),
                    );
                    if let Some(p) = payload {
                        props.set(b"payload", p);
                    }
                    let synth = Object {
                        class_id: orc.borrow().class_id,
                        class_name: PhpStr::new(b"\0__phpr_cformat".to_vec()),
                        props,
                        id: self.next_id(),
                        info: Rc::new(php_types::ObjectInfo::default()),
                        readonly_init: Vec::new(),
                        readonly_clone_writable: Vec::new(), typed_unset: Vec::new(),
                        lazy: None,
                        proxy_instance: None,
                    };
                    let out = Zval::Object(Rc::new(RefCell::new(synth)));
                    memo.insert(addr, out.clone());
                    return Ok(out);
                }
                let mut opaque_keys = false;
                let entries: Vec<(Vec<u8>, Zval)> = if let Some(cid) = cid.filter(|&c| {
                    resolve_method_runtime(&self.classes, c, b"__serialize").is_some()
                }) {
                    let _ = cid;
                    // __serialize's array keys are OPAQUE payload names, not
                    // properties: they serialize verbatim (no visibility
                    // re-mangling from the class's declarations).
                    opaque_keys = true;
                    let data = self
                        .call_method_sync(v.clone(), b"__serialize", Vec::new())?
                        .deref_clone();
                    let Zval::Array(a) = data else {
                        return Err(PhpError::TypeError(format!(
                            "{}::__serialize() must return an array",
                            String::from_utf8_lossy(orc.borrow().class_name.as_bytes())
                        )));
                    };
                    a.iter()
                        .map(|(k, val)| {
                            let kb = match k {
                                Key::Int(i) => i.to_string().into_bytes(),
                                Key::Str(s) => s.as_bytes().to_vec(),
                            };
                            (kb, val.clone())
                        })
                        .collect()
                } else if cid.is_some_and(|c| {
                    resolve_method_runtime(&self.classes, c, b"__sleep").is_some()
                }) {
                    let names = self
                        .call_method_sync(v.clone(), b"__sleep", Vec::new())?
                        .deref_clone();
                    let Zval::Array(names) = names else {
                        self.diags.push(Diag::Notice(
                            "serialize(): __sleep() should return an array only containing the names of instance-variables to serialize"
                                .to_string(),
                        ));
                        return Ok(Zval::Null);
                    };
                    // `__sleep` may have initialized a lazy *proxy* (it observed
                    // state); read the named props from the real instance it now
                    // forwards to, not the placeholder wrapper.
                    let read_rc = orc
                        .borrow()
                        .proxy_instance
                        .as_ref()
                        .and_then(|inst| deref_object(inst))
                        .unwrap_or_else(|| orc.clone());
                    let ob = read_rc.borrow();
                    let mut picked = Vec::new();
                    for (_, n) in names.iter() {
                        let want = convert::to_zstr_cast(&n.deref_clone(), &mut self.diags)
                            .as_bytes()
                            .to_vec();
                        // Match by *declared* name: the stored key may carry a
                        // private/protected mangle prefix.
                        let hit = ob.props.iter().find(|(k, _)| {
                            let (disp, _) = php_types::unmangle_prop_key(k, &ob.info);
                            disp == &want[..]
                        });
                        match hit {
                            Some((k, val)) => picked.push((k.to_vec(), val.clone())),
                            None => {
                                // PHP 8 skips the member: silently when it is a
                                // DECLARED (typed, uninitialized) property, with
                                // the "does not exist" Warning when undeclared.
                                let declared = cid.is_some_and(|mut c| loop {
                                    if self.classes[c].prop_info.contains_key(&want[..]) {
                                        break true;
                                    }
                                    match self.classes[c].parent {
                                        Some(p) => c = p,
                                        None => break false,
                                    }
                                });
                                if !declared {
                                    self.diags.push(Diag::Warning(format!(
                                        "serialize(): \"{}\" returned as member variable from __sleep() but does not exist",
                                        String::from_utf8_lossy(&want)
                                    )));
                                }
                            }
                        }
                    }
                    drop(ob);
                    picked
                } else {
                    // No hook here: keep the props, but rewrite the values (a
                    // hooked object may sit deeper).
                    orc.borrow().props.iter().map(|(k, val)| (k.to_vec(), val.clone())).collect()
                };
                // Synthetic same-class carrier, registered in `memo` BEFORE the
                // recursion so a cyclic graph terminates.
                // id 0 = synthetic carrier: never printed, never releases a handle.
                let synth = Rc::new(RefCell::new(orc.borrow().copy_with_id(0)));
                {
                    let mut sb = synth.borrow_mut();
                    // The carrier is a concrete snapshot: drop any lazy marker so
                    // the pure formatter treats it as an ordinary object (a
                    // hook-serialized lazy wrapper must not carry proxy state).
                    sb.lazy = None;
                    sb.proxy_instance = None;
                    let keys: Vec<Vec<u8>> = sb.props.iter().map(|(k, _)| k.to_vec()).collect();
                    for k in keys {
                        sb.props.remove(&k);
                    }
                    if opaque_keys {
                        sb.info = Rc::new(php_types::ObjectInfo::opaque());
                    }
                }
                let out = Zval::Object(synth.clone());
                memo.insert(addr, out.clone());
                for (k, val) in entries {
                    let pv = self.prepare_serialize(val, memo)?;
                    synth.borrow_mut().props.set(&k, pv);
                }
                Ok(out)
            }
            other => Ok(other),
        }
    }



    /// Turn a decoded [`Ser`](crate::unserialize::Ser) tree into a `Zval`, recursing
    /// into arrays/objects. Mirrors `eval::ser_to_zval`; objects go through
    /// [`Self::vm_make_unserialized_object`] (the VM's class table / id allocator).
    // (see UnserCtx below `vm_ser_build` siblings)
    fn vm_ser_to_zval(&mut self, s: crate::unserialize::Ser) -> Result<Zval, PhpError> {
        // Slot numbering mirrors serialize(): every value slot consumes a
        // pre-order number except `R:` emissions (`r:` does consume one).
        // Alias targets are pre-collected so their slots build into a shared
        // `Zval::Ref` cell the `R:` occurrences then alias.
        let mut ctx = UnserCtx::default();
        crate::unserialize::collect_alias_targets(&s, &mut ctx.targets);
        self.vm_ser_to_zval_slot(s, &mut ctx)
    }

    /// Number one wire slot, resolve `R:`, and cell-wrap alias targets.
    fn vm_ser_to_zval_slot(
        &mut self,
        s: crate::unserialize::Ser,
        ctx: &mut UnserCtx,
    ) -> Result<Zval, PhpError> {
        use crate::unserialize::Ser;
        if let Ser::AliasRef(t) = s {
            if let Some(cell) = ctx.cells.get(&t) {
                return Ok(Zval::Ref(Rc::clone(cell)));
            }
            // An alias of an un-wrapped slot (malformed / forward reference)
            // degrades to a value copy — never a fatal.
            return Ok(ctx.objs.get(&t).cloned().unwrap_or(Zval::Null));
        }
        ctx.count += 1;
        let n = ctx.count;
        let built = self.vm_ser_build(s, n, ctx)?;
        if ctx.targets.contains(&n) {
            let cell = Rc::new(RefCell::new(built));
            ctx.cells.insert(n, Rc::clone(&cell));
            return Ok(Zval::Ref(cell));
        }
        Ok(built)
    }

    fn vm_ser_build(
        &mut self,
        s: crate::unserialize::Ser,
        slot: i64,
        ctx: &mut UnserCtx,
    ) -> Result<Zval, PhpError> {
        use crate::unserialize::Ser;
        Ok(match s {
            Ser::AliasRef(_) => Zval::Null, // handled by the slot layer
            Ser::ObjRef(t) => match ctx.objs.get(&t) {
                Some(v) => v.clone(),
                // `r:` into an alias-wrapped slot copies the object handle out
                // of the shared cell.
                None => match ctx.cells.get(&t) {
                    Some(cell) => cell.borrow().clone(),
                    None => Zval::Null,
                },
            },
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
                    let val = self.vm_ser_to_zval_slot(v, ctx)?;
                    arr.insert(key, val);
                }
                Zval::Array(Rc::new(arr))
            }
            Ser::Object(class, props) => {
                let lower = class.to_ascii_lowercase();
                // The PDO classes are marked not-serializable in ext/pdo:
                // unserializing one throws (doctrine/instantiator's fallback
                // path converts this into its UnexpectedValueException).
                if matches!(lower.as_slice(), b"pdo" | b"pdostatement" | b"pdorow") {
                    let msg = format!(
                        "Unserialization of '{}' is not allowed",
                        String::from_utf8_lossy(&class)
                    );
                    if let Some(cid) = self.class_index.get(&b"exception"[..]).copied() {
                        let obj = self.synthesize_throwable(cid, &msg)?;
                        return Err(PhpError::Thrown(obj));
                    }
                }
                // An unknown class goes through the autoloader first, exactly
                // like PHP's unserialize (PHPUnit's process-isolation child
                // unserializes its Configuration before anything loaded it).
                if !self.class_index.contains_key(lower.as_slice()) {
                    // The autoloader receives the name as serialized (no
                    // leading backslash in the wire format); a throw inside
                    // it aborts the unserialize like PHP.
                    self.try_autoload(&class, &lower)?;
                }
                let cid = self.class_index.get(lower.as_slice()).copied();
                // `__unserialize` receives the raw data array INSTEAD of prop
                // materialisation (PHP 7.4 protocol; wins over __wakeup).
                if let Some(cid) = cid {
                    if resolve_method_runtime(&self.classes, cid, b"__unserialize").is_some() {
                        // Create + register the instance BEFORE building the
                        // data array, so a cyclic `r:<this>` resolves (Zend
                        // registers on seeing `O:`).
                        let obj = self.vm_make_unserialized_object(&class, Vec::new());
                        ctx.objs.insert(slot, obj.clone());
                        let mut data = PhpArray::new();
                        for (name, v) in props {
                            let val = self.vm_ser_to_zval_slot(v, ctx)?;
                            data.insert(Key::from_bytes(&name), val);
                        }
                        self.call_method_sync(
                            obj.clone(),
                            b"__unserialize",
                            vec![Zval::Array(Rc::new(data))],
                        )?;
                        return Ok(obj);
                    }
                }
                // Shell first (registered for cycles), fields second.
                let obj = self.vm_unserialized_shell(&class);
                ctx.objs.insert(slot, obj.clone());
                let fields: Vec<(Vec<u8>, Zval)> = props
                    .into_iter()
                    .map(|(name, v)| Ok((name, self.vm_ser_to_zval_slot(v, ctx)?)))
                    .collect::<Result<_, PhpError>>()?;
                self.vm_apply_unserialized_fields(&obj, fields);
                // The legacy `__wakeup` runs after the props are materialised.
                if let Some(cid) = cid {
                    if resolve_method_runtime(&self.classes, cid, b"__wakeup").is_some() {
                        self.call_method_sync(obj.clone(), b"__wakeup", Vec::new())?;
                    }
                }
                obj
            }
            Ser::CObject(class, payload) => {
                // Legacy `Serializable` record: instantiate without the
                // constructor and hand the raw payload to `->unserialize()`.
                // A class without an unserializer degrades to `false` with a
                // Warning (mirrors Zend's "class has no unserializer" path).
                let lower = class.to_ascii_lowercase();
                let cid = self.class_index.get(lower.as_slice()).copied().filter(|&c| {
                    resolve_method_runtime(&self.classes, c, b"unserialize").is_some()
                });
                let Some(cid) = cid else {
                    self.diags.push(Diag::Warning(format!(
                        "unserialize(): Class {} has no unserializer",
                        String::from_utf8_lossy(&class)
                    )));
                    return Ok(Zval::Bool(false));
                };
                let obj = self.alloc_object(cid)?;
                ctx.objs.insert(slot, obj.clone());
                self.call_method_sync(
                    obj.clone(),
                    b"unserialize",
                    vec![Zval::Str(PhpStr::new(payload))],
                )?;
                obj
            }
        })
    }

    /// Build an object of named `class` with the given properties, the constructor
    /// **not** run (as PHP's `unserialize` does). Mirrors `eval::make_object` on the
    /// VM's machinery (`Self::alloc_object`'s construction, but with the serialized
    /// props instead of declared defaults). An unknown class falls back to
    /// `stdClass` (D-50).
    fn vm_make_unserialized_object(&mut self, class: &[u8], fields: Vec<(Vec<u8>, Zval)>) -> Zval {
        let obj = self.vm_unserialized_shell(class);
        self.vm_apply_unserialized_fields(&obj, fields);
        obj
    }

    /// The instance-creation half of [`Self::vm_make_unserialized_object`]:
    /// class resolution (unknown → `__PHP_Incomplete_Class` carrying the name),
    /// declared defaults + prop-init thunk, no constructor. Split out so a
    /// cyclic graph can register the object in the slot registry *before* its
    /// serialized fields (which may `r:`-reference it) are built.
    fn vm_unserialized_shell(&mut self, class: &[u8]) -> Zval {
        let lower = class.to_ascii_lowercase();
        let mut fields: Vec<(Vec<u8>, Zval)> = Vec::new();
        // An unknown class becomes `__PHP_Incomplete_Class`, keeping the data
        // plus the original class name in `__PHP_Incomplete_Class_Name` (Zend).
        let mut cid = self.class_index.get(lower.as_slice()).copied();
        if cid.is_none() {
            cid = self.class_index.get(&b"__php_incomplete_class"[..]).copied();
            fields.insert(
                0,
                (
                    b"__PHP_Incomplete_Class_Name".to_vec(),
                    Zval::Str(PhpStr::new(class.to_vec())),
                ),
            );
        }
        let Some(cid) = cid else {
            // No prelude fallback class (should never happen) — degrade gracefully.
            return Zval::Null;
        };
        let cc = self.classes[cid];
        let class_name = Rc::clone(&cc.class_name);
        let info = Rc::clone(&cc.info);
        // Start from the declared defaults, exactly like `alloc_object`: Zend's
        // unserialize builds the object from the class's default-properties
        // table and only then applies the serialized fields (doctrine's
        // Instantiator relies on `O:N:"C":0:{}` yielding a defaulted instance).
        let mut props = Props::new();
        for (name, c) in &cc.prop_defaults {
            props.set(name, c.to_zval());
        }
        for name in &cc.uninit_props {
            props.set(name, Zval::Undef);
        }
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info, readonly_init: Vec::new(), readonly_clone_writable: Vec::new(), typed_unset: Vec::new(), lazy: None, proxy_instance: None };
        let rc = Rc::new(RefCell::new(obj));
        // Track for `__destruct` (OOP-3d), like every other freshly minted object.
        self.created.insert(id, Rc::clone(&rc));
        self.gc_track(&rc);
        // Non-constant declared defaults (`= []`, `= SOME_CONST`, …) live in the
        // class's `prop_init` thunk — run it on the fresh instance, like
        // `Op::InitProps` after `Op::Alloc`. A failing thunk degrades to the
        // constant-only defaults (no fatal from inside unserialize).
        self.run_prop_init_thunk(cid, &rc);
        let out = Zval::Object(rc);
        self.vm_apply_unserialized_fields(&out, fields);
        out
    }

    /// The field-application half: the serialized fields overwrite the shell's
    /// defaults. A restored readonly property counts as already initialised (so
    /// a later write fatals, and a read does not raise the
    /// before-initialization error).
    fn vm_apply_unserialized_fields(&mut self, obj: &Zval, fields: Vec<(Vec<u8>, Zval)>) {
        let Zval::Object(rc) = obj else { return };
        let cid = rc.borrow().class_id as usize;
        for (k, v) in fields {
            // PHP's wire format mangles protected fields as `\0*\0name`; the
            // VM stores protected properties under the plain name. (A private
            // `\0Class\0name` matches the VM storage key as-is.)
            let k: Vec<u8> = match k.strip_prefix(b"\0*\0") {
                Some(rest) => rest.to_vec(),
                None => k,
            };
            if prop_readonly_decl(&self.classes, cid, &k).is_some() {
                rc.borrow_mut().readonly_init.push(k.as_slice().into());
            }
            rc.borrow_mut().props.set(&k, v);
        }
    }

    /// Run a class's non-constant property-default thunk (`prop_init`) on a
    /// fresh instance, synchronously, from host-builtin context — the same work
    /// `Op::InitProps` does after `Op::Alloc` at a `new` site. Shared by
    /// `unserialize` and `ReflectionClass::newInstanceWithoutConstructor`. A
    /// failing thunk degrades to the constant-only defaults.
    fn run_prop_init_thunk(&mut self, cid: ClassId, rc: &Rc<RefCell<Object>>) {
        if let Some(func) = &self.classes[cid].prop_init {
            let baseline = self.frames.len();
            let mut frame = Frame::new(func, self.class_mod(cid));
            frame.this = Some(Zval::Object(Rc::clone(rc)));
            frame.class = Some(cid);
            frame.static_class = Some(cid);
            frame.init_props = true; // privileged default writes
            self.frames.push(frame);
            let _ = self.drive_to_return(baseline);
        }
    }

    fn collect_backtrace(&self) -> Vec<BtFrame> {
        let top = self.frames.len() - 1;
        let mut out = Vec::new();
        for i in (1..=top).rev() {
            let f = &self.frames[i];
            // An `eval()` unit's frame renders as `eval`; an anonymous frame as
            // `{closure}`; otherwise its own name.
            let function = if f.eval_origin.is_some() {
                b"eval".to_vec()
            } else if f.func.name.is_empty() {
                b"{closure}".to_vec()
            } else {
                f.func.name.to_vec()
            };
            // The call was made from the *caller* frame (i-1): its file, unless that
            // caller is itself an eval unit, in which case PHP names it
            // `<file>(<line>) : eval()'d code`.
            let caller = &self.frames[i - 1];
            let file = match &caller.eval_origin {
                Some((ofile, oline)) => {
                    let mut s = ofile.to_vec();
                    s.extend_from_slice(format!("({oline}) : eval()'d code").as_bytes());
                    s
                }
                None => caller.module.file.to_vec(),
            };
            let (class, object) = match f.class {
                // Resolve the class id in the frame's own module (an eval'd /
                // included frame may differ from `self.module`).
                Some(cid) => (Some(self.classes[cid].name.to_vec()), f.this.clone()),
                None => (None, None),
            };
            out.push(BtFrame {
                function,
                file,
                line: self.cur_line(i - 1),
                class,
                // A method with no bound `$this` is a static call ("::"); otherwise "->".
                is_static: f.class.is_some() && f.this.is_none(),
                object,
                args: self.current_frame_args(i),
                is_eval: f.eval_origin.is_some(),
            });
        }
        out
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

    /// Mint a `stream-context` resource carrying its options array (a host
    /// builtin because it allocates a resource id, like `fopen`).
    fn alloc_resource_context(&mut self, options: Zval) -> Zval {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        Zval::Resource(Rc::new(RefCell::new(Resource::new_context(
            id,
            options,
            Zval::Array(Rc::new(PhpArray::new())),
        ))))
    }

















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
        if path.starts_with(b"data:") {
            return match php_types::open_data_stream(&path) {
                Some(stream) => Ok(self.alloc_resource(stream)),
                None => {
                    self.diags.push(Diag::Warning(format!(
                        "fopen({}): Failed to open stream: rfc2397: unable to decode",
                        String::from_utf8_lossy(&path)
                    )));
                    let line = self.cur_line(self.frames.len() - 1);
                    self.flush_diags(line)?;
                    Ok(Zval::Bool(false))
                }
            };
        }
        if let Some(spec) = path.strip_prefix(b"php://".as_slice()) {
            return match open_php_stream(spec, &mode) {
                Some(stream) => Ok(self.alloc_resource(stream)),
                None => {
                    self.diags.push(Diag::Warning(format!(
                        "fopen({}): Failed to open stream: no suitable wrapper could be found",
                        String::from_utf8_lossy(&path)
                    )));
                    // Synchronous flush: PHP raises the fopen warning AT the
                    // call, so a set_error_handler installed just around it
                    // (monolog's StreamHandler) sees it even when the deferred
                    // per-statement flush would land after restore_error_handler.
                    let line = self.cur_line(self.frames.len() - 1);
                    self.flush_diags(line)?;
                    Ok(Zval::Bool(false))
                }
            };
        }
        // compress.zlib://path — the zlib wrapper routes through the gz stream
        // machinery (transparent read of plain files, GzFile buffer on write).
        // The stream keeps the FULL wrapper spec as its uri, which is what makes
        // stream_get_meta_data report wrapper_type ZLIB (gzopen streams don't).
        if let Some(rest) = path.strip_prefix(b"compress.zlib://".as_slice()) {
            let rest = rest.to_vec();
            let r = self.gz_open_stream(&rest, &mode, "fopen")?;
            if let Zval::Resource(rc) = &r {
                if let Some(s) = rc.borrow_mut().as_stream_mut() {
                    s.uri = path.clone();
                }
            }
            let line = self.cur_line(self.frames.len() - 1);
            self.flush_diags(line)?;
            return Ok(r);
        }
        // A registered userland stream wrapper (stream_wrapper_register) claims
        // its scheme before the filesystem is consulted.
        if let Some(pos) = path.windows(3).position(|w| w == b"://") {
            let scheme = path[..pos].to_ascii_lowercase();
            if self.stream_wrappers.contains_key(&scheme) {
                let r = self.fopen_user_wrapper(&path, &mode);
                let line = self.cur_line(self.frames.len() - 1);
                self.flush_diags(line)?;
                return r;
            }
        }
        match open_file_stream(&path, &mode) {
            Ok(stream) => Ok(self.alloc_resource(stream)),
            Err(msg) => {
                self.diags.push(Diag::Warning(format!(
                    "fopen({}): Failed to open stream: {msg}",
                    String::from_utf8_lossy(&path)
                )));
                let line = self.cur_line(self.frames.len() - 1);
                self.flush_diags(line)?;
                Ok(Zval::Bool(false))
            }
        }
    }



    /// Resolve a `proc_*` argument to its process resource, or a `TypeError`.
    fn proc_arg(
        args: &[Zval],
        fname: &str,
    ) -> Result<Rc<RefCell<Resource>>, PhpError> {
        match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Resource(r)) => Ok(r),
            other => Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($process) must be of type resource, {} given",
                other.map(|v| v.type_name_for_error()).unwrap_or_else(|| "null".to_string())
            ))),
        }
    }







    /// Run the PHP handlers of every pending signal (two-phase delivery). The
    /// pending set is drained *before* calling out, so a handler re-raising the
    /// same signal is delivered on the next dispatch, not recursively.
    fn dispatch_pending_signals(&mut self) -> Result<(), PhpError> {
        loop {
            let pending = PENDING_SIGNALS.swap(0, std::sync::atomic::Ordering::SeqCst);
            if pending == 0 {
                return Ok(());
            }
            for signo in 1..32 {
                if pending & (1u64 << signo) == 0 {
                    continue;
                }
                let Some(handler) = self.signal_handlers.get(&(signo as i32)).cloned() else {
                    continue;
                };
                if matches!(handler, Zval::Long(_)) {
                    continue;
                }
                // PHP passes ($signo, $siginfo); phpr models no siginfo → null.
                self.call_callable(handler, vec![Zval::Long(signo), Zval::Null])?;
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

    /// Compute the exit status of `exit`/`die`'s argument (step 46), mirroring
    /// `eval::exit_status`: a string (or stringable object) is printed and the code
    /// is 0; an int / other scalar becomes the exit code (`% 256`); array / a
    /// non-stringable object is a `TypeError`.
    fn exit_status(&mut self, v: Zval, top: usize) -> Result<u8, PhpError> {
        let v = match v {
            Zval::Ref(cell) => cell.borrow().clone(),
            other => other,
        };
        match &v {
            Zval::Str(s) => {
                let bytes = s.as_bytes().to_vec();
                self.emit_str(top, &bytes)?;
                Ok(0)
            }
            Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) | Zval::Null | Zval::Undef => {
                Ok(convert::to_long_cast(&v, &mut self.diags).rem_euclid(256) as u8)
            }
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                if resolve_method_runtime(&self.classes, cid, b"__toString").is_some() {
                    let s = self.vm_stringify(&v)?;
                    let bytes = s.as_bytes().to_vec();
                    self.emit_str(top, &bytes)?;
                    Ok(0)
                } else {
                    Err(self.exit_type_error(&v))
                }
            }
            _ => Err(self.exit_type_error(&v)),
        }
    }

    /// The `TypeError` for `exit`/`die` given a value outside `string|int` (step
    /// 46): objects are named by their class, other values by their PHP type name.
    fn exit_type_error(&self, v: &Zval) -> PhpError {
        let given = match v {
            Zval::Object(o) => String::from_utf8_lossy(
                &self.classes[o.borrow().class_id as usize].name,
            )
            .into_owned(),
            other => crate::coerce::php_type_name(other).to_string(),
        };
        PhpError::TypeError(format!(
            "exit(): Argument #1 ($status) must be of type string|int, {given} given"
        ))
    }

    /// Convert a value to a string, running `__toString` for an object via a nested
    /// bounded run (the synchronous analogue of [`Op::Stringify`]). A non-object is
    /// coerced directly; an object without `__toString` is the usual fatal `Error`.
    fn vm_stringify(&mut self, v: &Zval) -> Result<Rc<PhpStr>, PhpError> {
        match v {
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                match resolve_method_runtime(&self.classes, cid, b"__toString") {
                    Some((defc, midx)) => {
                        let callee = &self.classes[defc].methods[midx].func;
                        let baseline = self.frames.len();
                        let mut frame = Frame::new(callee, self.class_mod(defc));
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


    /// `$$x` read (Op::LoadVarDyn): superglobals by name, then the frame's
    /// named slots, then the dynamic side-table. An undefined name raises the
    /// PHP 8 "Undefined variable" warning and reads NULL, like `LoadVar`.
    fn var_dyn_read(&mut self, top: usize, name: &[u8]) -> Zval {
        if let Some(idx) = crate::bytecode::superglobal_index(name) {
            return read_slot(&self.superglobals[idx as usize]);
        }
        let named = self.frames[top].func.slot_names.iter().position(|n| n.as_ref() == name);
        if let Some(s) = named {
            if matches!(self.frames[top].slots[s], Zval::Undef) {
                self.diags.push(Diag::Warning(format!(
                    "Undefined variable ${}",
                    String::from_utf8_lossy(name)
                )));
                return Zval::Null;
            }
            return read_slot(&self.frames[top].slots[s]);
        }
        if let Some(v) = self.frames[top].dyn_vars.get(name) {
            return read_slot(v);
        }
        self.diags.push(Diag::Warning(format!(
            "Undefined variable ${}",
            String::from_utf8_lossy(name)
        )));
        Zval::Null
    }

    /// `$$x = v` (Op::StoreVarDyn): write through a reference like
    /// `store_slot`; a name outside the frame's static slots lives in the
    /// dynamic side-table (`Frame::dyn_vars`).
    fn var_dyn_write(&mut self, top: usize, name: &[u8], v: Zval) -> Result<(), PhpError> {
        if name == b"this" {
            return Err(PhpError::Error("Cannot re-assign $this".to_string()));
        }
        if let Some(idx) = crate::bytecode::superglobal_index(name) {
            let old = store_slot(&mut self.superglobals[idx as usize], v);
            self.gc_note(&old);
            return Ok(());
        }
        let named = self.frames[top].func.slot_names.iter().position(|n| n.as_ref() == name);
        if let Some(s) = named {
            let old = store_slot(&mut self.frames[top].slots[s], v);
            self.gc_note(&old);
            return Ok(());
        }
        if let Some(cell) = self.frames[top].dyn_vars.get_mut(name) {
            let old = store_slot(cell, v);
            self.gc_note(&old);
            return Ok(());
        }
        self.frames[top].dyn_vars.insert(name.to_vec(), v);
        Ok(())
    }

    /// `global $$x` (Op::BindGlobalDyn): the dynamic-name counterpart of the
    /// static `global $x` BindRef pair. Resolves-or-creates the global cell by
    /// name (a fresh cell is created as NULL, matching Zend's global fetch —
    /// the entry appears in `$GLOBALS` even before any assignment), promotes
    /// it to a shared cell, and installs the alias into the same-named local:
    /// a named frame slot when the function declares one, otherwise the
    /// dynamic side-table (`Frame::dyn_vars`).
    fn bind_global_dyn(&mut self, top: usize, name: &[u8]) -> Result<(), PhpError> {
        if name == b"this" {
            return Err(PhpError::Error("Cannot re-assign $this".to_string()));
        }
        let cell = if let Some(idx) = crate::bytecode::superglobal_index(name) {
            make_cell(&mut self.superglobals[idx as usize])
        } else {
            let slot = self.global_slot_by_name(name);
            make_cell(&mut self.frames[0].slots[slot])
        };
        // An include/eval unit shares the includer's variable scope (ONE Zend
        // symbol table): `global` there rebinds the shared symbol, so install
        // the alias in every bridged ancestor as well — up to, but not into,
        // the global frame itself (frame 0 holds the cell already).
        let mut f = top;
        loop {
            let named =
                self.frames[f].func.slot_names.iter().position(|n| n.as_ref() == name);
            if let Some(s) = named {
                self.frames[f].slots[s] = Zval::Ref(cell.clone());
            } else {
                self.frames[f].dyn_vars.insert(name.to_vec(), Zval::Ref(cell.clone()));
            }
            match self.frames[f].bridge_caller {
                Some(caller) if caller != 0 => f = caller,
                _ => break,
            }
        }
        Ok(())
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
            b"uasort" => self.ho_uasort(slot, rest),
            b"uksort" => self.ho_uksort(slot, rest),
            b"array_walk" => self.ho_array_walk(slot, rest),
            b"array_walk_recursive" => self.ho_array_walk_recursive(slot, rest),
            b"reset" => self.ho_array_pointer(slot, PtrOp::Reset),
            b"end" => self.ho_array_pointer(slot, PtrOp::End),
            b"next" => self.ho_array_pointer(slot, PtrOp::Next),
            b"prev" => self.ho_array_pointer(slot, PtrOp::Prev),
            b"current" => self.ho_array_pointer(slot, PtrOp::Current),
            b"key" => self.ho_array_pointer(slot, PtrOp::Key),
            _ => Err(undefined_builtin(name)),
        }
    }




    /// Recursive worker for `array_walk_recursive`: rebuild `arr`, recursing into
    /// array elements (the callback never sees an array node) and applying the
    /// callback to each leaf (by value or, if `by_ref`, through a shared cell).
    fn walk_recursive(
        &mut self,
        arr: &PhpArray,
        callback: &Zval,
        extra: &Option<Zval>,
        by_ref: bool,
    ) -> Result<PhpArray, PhpError> {
        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        let mut out = PhpArray::new();
        for (k, v) in entries {
            let new_v = match v {
                Zval::Array(inner) => {
                    Zval::Array(Rc::new(self.walk_recursive(&inner, callback, extra, by_ref)?))
                }
                leaf => {
                    let key_z = key_to_zval(&k);
                    if by_ref {
                        let vcell = Rc::new(RefCell::new(leaf));
                        let mut argv = vec![Zval::Ref(Rc::clone(&vcell)), key_z];
                        if let Some(e) = extra {
                            argv.push(e.clone());
                        }
                        self.call_callable(callback.clone(), argv)?;
                        let updated = vcell.borrow().clone();
                        updated
                    } else {
                        let mut argv = vec![leaf.clone(), key_z];
                        if let Some(e) = extra {
                            argv.push(e.clone());
                        }
                        self.call_callable(callback.clone(), argv)?;
                        leaf
                    }
                }
            };
            out.insert(k, new_v);
        }
        Ok(out)
    }

    /// Whether a callable's first parameter is declared by-reference (`&$x`).
    /// Used by `array_walk` to decide if element mutations propagate. Only user
    /// closures and named user functions are inspected; anything else is false.
    fn callable_first_by_ref(&self, callee: &Zval) -> bool {
        match callee {
            // Resolve against the closure's OWN module (`closure_func_mod`):
            // reading `self.module.closures[fn_idx]` looked the flag up in the
            // *current* unit, so a `&$item` walk callback defined in an
            // included file lost its by-ref binding (symfony's
            // StreamedJsonResponse placeholder walk silently wrote nothing).
            Zval::Closure(cl) => self
                .closure_func_mod(cl)
                .and_then(|(f, _)| f.param_by_ref.first().copied())
                .unwrap_or(false),
            Zval::Str(s) => self.named_first_by_ref(s.as_bytes()),
            Zval::Ref(c) => self.callable_first_by_ref(&c.borrow()),
            _ => false,
        }
    }

    /// First-parameter by-reference flag of a named user function
    /// (case-insensitive), searching the current module first and the
    /// cross-unit `linked_functions` registry after — a string callable may
    /// name a function another unit defined.
    fn named_first_by_ref(&self, name: &[u8]) -> bool {
        self.user_function_with_mod(name)
            .and_then(|(f, _)| f.param_by_ref.first().copied())
            .unwrap_or(false)
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



    /// Stable merge sort of `(compare-value, key, value)` items by the PHP
    /// comparator applied to the compare-value — the key-preserving engine behind
    /// [`Self::ho_uasort`] / [`Self::ho_uksort`], mirroring [`Self::vm_merge_sort_with`].
    fn vm_merge_sort_pairs(
        &mut self,
        cmp: &Zval,
        mut items: Vec<(Zval, Key, Zval)>,
    ) -> Result<Vec<(Zval, Key, Zval)>, PhpError> {
        let n = items.len();
        if n <= 1 {
            return Ok(items);
        }
        let right = items.split_off(n / 2);
        let left = self.vm_merge_sort_pairs(cmp, items)?;
        let right = self.vm_merge_sort_pairs(cmp, right)?;
        let mut merged = Vec::with_capacity(n);
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            if self.compare_with_callback(cmp, &left[i].0, &right[j].0)? <= 0 {
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



    /// Whether `v` is callable (the predicate behind `is_callable`), without
    /// invoking it.
    fn is_value_callable(&self, v: &Zval) -> bool {
        // zend_is_callable checks accessibility from the CALLING scope: a
        // private/protected method is callable only from where a direct call
        // would be; an inaccessible or missing method still counts when the
        // magic trampoline exists (`__call` / `__callStatic`).
        let scope = self.frames.last().and_then(|f| f.class);
        // Zend's one legacy exception for static-style instance methods: the
        // calling frame's `$this`, when an instance of the NAMED class, makes
        // "C::m" callable as an instance call (bug48899 — a method calling
        // is_callable(['OwnClass','ownMethod']) sees true).
        let this_instance_of = |cid: ClassId| -> bool {
            self.frames
                .last()
                .and_then(|f| f.this.as_ref())
                .and_then(object_class_id)
                .is_some_and(|tc| is_instance_of(&self.classes, self.stringable_id, tc, cid))
        };
        let method_callable = |cid: ClassId, m: &[u8], magic: &[u8], static_style: bool| -> bool {
            match resolve_method_runtime(&self.classes, cid, m) {
                Some((decl, idx)) => {
                    let mm = &self.classes[decl].methods[idx];
                    if method_visible_from(&self.classes, scope, mm.visibility, decl, m) {
                        // A VISIBLE instance method referenced static-style
                        // ("C::m" / ['C','m']) is not callable — PHP 8 removed
                        // static-style instance calls — and does NOT fall back
                        // to __callStatic (the method exists). An inaccessible
                        // one does (Zend zend_is_callable_check_func).
                        !static_style || mm.is_static || this_instance_of(cid)
                    } else {
                        resolve_method_runtime(&self.classes, cid, magic).is_some()
                    }
                }
                None => resolve_method_runtime(&self.classes, cid, magic).is_some(),
            }
        };
        match v {
            Zval::Closure(_) => true,
            Zval::Str(s) => {
                let b = s.as_bytes();
                if let Some(pos) = b.windows(2).position(|w| w == b"::") {
                    self.class_id_from_value(&Zval::Str(PhpStr::new(b[..pos].to_vec())))
                        .map(|cid| method_callable(cid, &b[pos + 2..], b"__callStatic", true))
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
                    Some(cid) => {
                        let is_obj = matches!(elems[0], Zval::Object(_));
                        let magic = if is_obj { &b"__call"[..] } else { &b"__callStatic"[..] };
                        method_callable(cid, m.as_bytes(), magic, !is_obj)
                    }
                    None => false,
                }
            }
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                resolve_method_runtime(&self.classes, cid, b"__invoke").is_some()
            }
            Zval::Ref(r) => self.is_value_callable(&r.borrow()),
            _ => false,
        }
    }

    /// Coerce / check a by-value argument or return value against a single declared
    /// type hint (step 14, non-scalar extension). A scalar hint coerces under weak
    /// typing (or checks under `strict_types`); `array` / `callable` / `iterable` /
    /// `object` / a class name are *checked* (no coercion). `null` satisfies a
    /// nullable hint. On a mismatch, returns the class-named type of the value for
    /// the TypeError message.
    pub(super) fn coerce_or_check_hint(
        &mut self,
        value: Zval,
        hint: &TypeHint,
        strict: bool,
    ) -> Result<Zval, String> {
        let v = value.deref_clone();
        if matches!(v, Zval::Null | Zval::Undef) {
            return if hint.nullable { Ok(Zval::Null) } else { Err("null".to_string()) };
        }
        match &hint.kind {
            HintKind::Scalar(st) => {
                // Weak mode converts a __toString object for a `string` hint
                // (Zend's zend_verify_weak_scalar_type_checks; strict mode
                // rejects it) — symfony passes Stringable objects to ?string
                // parameters (Response::setContent) and string returns.
                // (Residue: a THROWING __toString here degrades to the
                // TypeError instead of propagating the exception.)
                if !strict && *st == crate::hir::ScalarType::String {
                    if let Some(s) = self.object_to_string_weak(&v) {
                        return Ok(Zval::Str(s));
                    }
                }
                coerce_to_hint(value, hint, &mut self.diags, strict).map_err(str::to_string)
            }
            HintKind::Array => match v {
                Zval::Array(_) => Ok(value),
                other => Err(other.type_name_for_error()),
            },
            HintKind::Object => match v {
                Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_) => Ok(value),
                other => Err(other.type_name_for_error()),
            },
            HintKind::Callable => {
                if self.is_value_callable(&v) {
                    Ok(value)
                } else {
                    Err(v.type_name_for_error())
                }
            }
            HintKind::Iterable => {
                if matches!(v, Zval::Array(_)) || self.value_satisfies_class(&v, b"Traversable") {
                    Ok(value)
                } else {
                    Err(v.type_name_for_error())
                }
            }
            HintKind::Class(name) => {
                if self.value_satisfies_class(&v, name) {
                    Ok(value)
                } else {
                    Err(v.type_name_for_error())
                }
            }
            HintKind::Union(members) => {
                // zend_verify_arg_type on a union: an exact (check-only or
                // same-scalar) member accepts as-is; only then weak mode tries
                // scalar coercion member by member in declared order.
                let exact = |k: &HintKind| -> bool {
                    match k {
                        HintKind::Scalar(ScalarType::Int) => matches!(v, Zval::Long(_)),
                        // An int is NOT an exact float in a union: when `int`
                        // is absent, Zend WIDENS it to float (`bool|float|string`
                        // given 7 yields 7.0), which the preference loop below
                        // performs — exact acceptance would keep it an int.
                        HintKind::Scalar(ScalarType::Float) => matches!(v, Zval::Double(_)),
                        HintKind::Scalar(ScalarType::String) => matches!(v, Zval::Str(_)),
                        HintKind::Scalar(ScalarType::Bool) => matches!(v, Zval::Bool(_)),
                        HintKind::Array => matches!(v, Zval::Array(_)),
                        HintKind::Object => {
                            matches!(v, Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_))
                        }
                        HintKind::Callable => self.is_value_callable(&v),
                        HintKind::Iterable => {
                            matches!(v, Zval::Array(_))
                                || self.value_satisfies_class(&v, b"Traversable")
                        }
                        HintKind::Class(name) => self.value_satisfies_class(&v, name),
                        HintKind::Union(_) => false, // unions do not nest
                    }
                };
                if members.iter().any(exact) {
                    return Ok(value);
                }
                if !strict {
                    // Weak union coercion follows Zend's PREFERENCE order —
                    // int, float, string, bool — over the members present,
                    // NOT the declared order (`bool|string` given 10 must
                    // produce "10", not true: symfony's
                    // addCacheControlDirective(bool|string) relies on it).
                    use crate::hir::ScalarType as St;
                    for want in [St::Int, St::Float, St::String, St::Bool] {
                        if !members.iter().any(|k| matches!(k, HintKind::Scalar(s) if *s == want))
                        {
                            continue;
                        }
                        // An object converts only toward `string`, via __toString.
                        if want == St::String {
                            if let Some(s) = self.object_to_string_weak(&v) {
                                return Ok(Zval::Str(s));
                            }
                        }
                        let single = TypeHint {
                            kind: HintKind::Scalar(want),
                            nullable: hint.nullable,
                        };
                        if let Ok(c) =
                            coerce_to_hint(value.clone(), &single, &mut self.diags, strict)
                        {
                            return Ok(c);
                        }
                    }
                }
                Err(v.type_name_for_error())
            }
        }
    }

    /// Weak-mode `object → string` conversion for a `string`(-containing) type:
    /// `Some(str)` when `v` is an object with a `__toString` method (invoked
    /// synchronously), `None` otherwise. Strict mode must not call this.
    fn object_to_string_weak(&mut self, v: &Zval) -> Option<php_types::ZStr> {
        let Zval::Object(o) = v else { return None };
        let cid = o.borrow().class_id as usize;
        resolve_method_runtime(&self.classes, cid, b"__toString")?;
        let res = self.call_method_sync(v.clone(), b"__toString", Vec::new()).ok()?;
        Some(php_types::convert::to_zstr(&res, &mut self.diags))
    }

    /// Whether class `cid` permits dynamic (undeclared) properties without the
    /// PHP 8.2 deprecation: `stdClass` (and internal classes modelled on it) and
    /// any class carrying `#[AllowDynamicProperties]`, which is inherited — so the
    /// check walks the parent chain.
    fn allows_dynamic_props(&self, cid: ClassId) -> bool {
        let mut c = Some(cid);
        while let Some(x) = c {
            let cc = self.classes[x];
            if cc.name.eq_ignore_ascii_case(b"stdClass") {
                return true;
            }
            if cc.attributes.iter().any(|a| {
                a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(b"AllowDynamicProperties")
            }) {
                return true;
            }
            c = cc.parent;
        }
        false
    }

    /// Enforce a typed instance property's declared type on a write (step: typed
    /// properties). Returns the value coerced to the declared type (weak typing) or
    /// a `TypeError` ("Cannot assign … to property C::$p of type T"). An untyped /
    /// dynamic property passes the value through unchanged. The strict-types mode of
    /// the *writing* frame governs coercion, mirroring the parameter binder.
    fn coerce_typed_prop_write(&mut self, ocid: ClassId, name: &[u8], value: Zval) -> Result<Zval, PhpError> {
        let Some((decl, hint)) = prop_type_decl(&self.classes, ocid, name) else {
            return Ok(value);
        };
        // Zend names the CLASS of an object value in the mismatch message
        // ("Cannot assign stdClass to …"); the scalar coercion layer only
        // knows "object".
        let obj_name = deref_object(&value)
            .map(|o| String::from_utf8_lossy(&self.classes[o.borrow().class_id as usize].name).into_owned());
        let strict = self.frames.last().map(|f| f.module.strict).unwrap_or(self.module.strict);
        match self.coerce_or_check_hint(value.clone(), &hint, strict) {
            Ok(v) => Ok(v),
            Err(given) => {
                // Weak typing: a Stringable object assigned to a string-typed slot
                // coerces via __toString — which may run arbitrary code and even
                // initialize a lazy object
                // (setRawValueWithoutLazyInitialization_side_effect_toString).
                if !strict {
                    if let Some(s) = self.stringify_for_string_hint(&value, &hint)? {
                        if let Ok(v) = self.coerce_or_check_hint(s, &hint, strict) {
                            return Ok(v);
                        }
                    }
                }
                let given = match (given.as_str(), obj_name) {
                    ("object", Some(n)) => n,
                    _ => given,
                };
                Err(PhpError::TypeError(format!(
                    "Cannot assign {given} to property {}::${} of type {}",
                    String::from_utf8_lossy(&self.classes[decl].name),
                    String::from_utf8_lossy(name),
                    hint.display_name(),
                )))
            }
        }
    }

    /// If `hint` admits a string and `value` is a `Stringable` object, return its
    /// `__toString()` result (which may run arbitrary code / throw) for weak-typing
    /// object→string coercion; `None` when the coercion does not apply.
    fn stringify_for_string_hint(
        &mut self,
        value: &Zval,
        hint: &TypeHint,
    ) -> Result<Option<Zval>, PhpError> {
        let accepts_string = match &hint.kind {
            HintKind::Scalar(ScalarType::String) => true,
            HintKind::Union(ms) => {
                ms.iter().any(|k| matches!(k, HintKind::Scalar(ScalarType::String)))
            }
            _ => false,
        };
        if !accepts_string {
            return Ok(None);
        }
        let Some(cid) = deref_object(value).map(|o| o.borrow().class_id as usize) else {
            return Ok(None);
        };
        if resolve_method_runtime(&self.classes, cid, b"__toString").is_none() {
            return Ok(None);
        }
        let s = self.call_method_sync(value.clone(), b"__toString", Vec::new())?.deref_clone();
        Ok(Some(s))
    }

    /// Whether `v` is an instance of the class/interface `name` (the `instanceof`
    /// check behind a class type hint). A `Closure`/`Generator` value satisfies its
    /// implicit class (`Closure`; `Generator`/`Iterator`/`Traversable`); a real
    /// object walks its ancestry. A non-object or an unknown name is `false`.
    fn value_satisfies_class(&self, v: &Zval, name: &[u8]) -> bool {
        let lc = name.strip_prefix(b"\\").unwrap_or(name).to_ascii_lowercase();
        match v {
            Zval::Closure(_) => lc == b"closure",
            Zval::Generator(_) => {
                matches!(&lc[..], b"generator" | b"iterator" | b"traversable")
            }
            Zval::Object(o) => {
                // A value satisfies a class/interface type hint if it is-a that
                // type — including implemented interfaces (transitively), not just
                // the parent chain. Mirrors `instanceof` (`is_instance_of`).
                matches!(self.class_index.get(&lc[..]),
                    Some(&target) if is_instance_of(&self.classes, self.stringable_id, o.borrow().class_id as usize, target))
            }
            Zval::Ref(r) => self.value_satisfies_class(&r.borrow(), name),
            _ => false,
        }
    }

    /// Whether a bare name is callable: a user function, any registry builtin, or a
    /// host builtin (mirrors `eval::is_name_callable`).
    fn is_name_callable(&self, name: &[u8]) -> bool {
        // A fully-qualified callable (`'\trim'`, `\trim(...)` reaching the
        // dynamic path) resolves like the bare name.
        let name = name.strip_prefix(b"\\").unwrap_or(name);
        // Conditional declarations are callable only once registered in
        // `linked_functions` by their `Op::DeclareFn`.
        self.module
            .functions
            .iter()
            .enumerate()
            .any(|(i, f)| !self.module.conditional_fns.contains(&i) && name_eq_ignore_case(&f.name, name))
            || self.linked_functions.contains_key(&name.to_ascii_lowercase())
            || self.registry.get(name).is_some()
            || host_builtin_canonical(name).is_some()
            || host_builtin_ref_first(name).is_some()
            // Builtins with by-ref *output* parameters (preg_match's &$matches,
            // sscanf's trailing vars) live in their own dispatch tables — they
            // are real functions to `function_exists`/`is_callable`.
            || host_builtin_out_param(name).is_some()
            || host_builtin_scanf(name).is_some()
            // `array_multisort` is compiled to its own all-by-ref op, so it is
            // not in any registry/host table but is a real callable function.
            || name.eq_ignore_ascii_case(b"array_multisort")
    }

    /// Install a frame for an anonymous closure: bind its captured variables into
    /// their slots, then the call arguments into the leading parameter slots, and
    /// the bound `$this`. Mirrors `eval::call_closure` (captures before params).
    fn push_closure_frame(&mut self, cl: &Closure, mut args: Vec<Zval>) -> Result<(), PhpError> {
        // A closure carries a module-local `fn_idx` plus the `module_id` of the
        // unit that defined it, so it stays callable after control leaves that
        // module (e.g. a closure made in an `include`/`eval` unit and invoked
        // later). Resolve the body against its *defining* module, not the running
        // one.
        let m = self.modules.get(cl.module_id).copied().unwrap_or(self.module);
        let Some(callee) = m.closures.get(cl.fn_idx) else {
            return Err(PhpError::Error(
                "closure is not callable in this context".to_string(),
            ));
        };
        // Deferred place arguments (SEND_VAR_EX, from a dynamic `$f(...)`)
        // resolve against the closure's by-ref mask now that the body is known.
        if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
            let top = self.frames.len() - 1;
            self.materialize_arg_places(top, &mut args, Some(callee))?;
        }
        let mut frame = Frame::new(callee, m);
        // `static $x` in the body persists per this closure *instance* (id).
        frame.closure_id = Some(cl.id);
        for (slot, val) in &cl.captures {
            frame.slots[*slot as usize] = val.clone();
        }
        bind_params(&mut frame, args);
        frame.this = cl.bound_this.clone();
        // The closure's class scope governs private/protected access and
        // `self::`/`static::` inside the body (set at creation or by
        // `Closure::bind`'s `$newScope`).
        frame.class = cl.scope;
        // `static::` resolves to the LSB captured at creation (Child::getCb()'s
        // closures keep Child), then the bound object's class, then the scope.
        frame.static_class = cl
            .lsb
            .or_else(|| cl.bound_this.as_ref().and_then(object_class_id))
            .or(cl.scope);
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
        let callee = &self.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee, self.class_mod(defc));
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
        let mut resolved = resolve_method_runtime(&self.classes, cid, method);
        // A caller-scope private with this name wins over the subclass method
        // (Zend non-virtual private dispatch, see `parent_private_rebind`).
        if let Some(rb) = parent_private_rebind(
            &self.classes,
            self.frames[top].class,
            cid,
            method,
            resolved,
        ) {
            resolved = Some(rb);
        }
        // Usable only if found *and* visible from the caller's scope.
        let usable = resolved.filter(|&(defc, midx)| {
            method_visible_from(&self.classes, self.frames[top].class, self.classes[defc].methods[midx].visibility, defc, method)
        });
        match usable {
            Some((defc, midx)) => {
                let callee = &self.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee, self.class_mod(defc));
                bind_params(&mut frame, args);
                frame.this = Some(this);
                frame.class = Some(defc);
                frame.static_class = Some(cid); // LSB = receiver's actual class
                self.enter_callee(frame)?;
            }
            // Missing or inaccessible: route to `__call` if defined, else the
            // original fatal (visibility / undefined method).
            None => match resolve_method_runtime(&self.classes, cid, b"__call") {
                Some((cdefc, cmidx)) => {
                    // `__call($name, $args)` gets a value array — decay references
                    // pushed by a dynamic call (SEND_VAR_EX).
                    self.push_magic_call(cdefc, cmidx, Some(this), cid, method, decay_args(args));
                }
                None => {
                    return Err(match resolved {
                        Some((defc, midx)) => method_access_error(
                            &self.classes,
                            defc,
                            method,
                            self.frames[top].class,
                            self.classes[defc].methods[midx].visibility,
                        ),
                        None => undefined_method(&self.classes, cid, method),
                    })
                }
            },
        }
        Ok(())
    }

    /// Dispatch a static method call `start::method(args)` whose starting class
    /// The resolved user callee whose signature decides SEND_VAR_EX for a
    /// dynamic *instance* call, or `None` when every argument is by-value
    /// anyway (non-object/native receiver, `__call` routing, unresolved or
    /// invisible method).
    fn instance_arg_ref_target(&self, top: usize, this: &Zval, method: &[u8]) -> Option<&'m Func> {
        let Zval::Object(o) = this else { return None };
        let cid = o.borrow().class_id as usize;
        let resolved = resolve_method_runtime(&self.classes, cid, method);
        // Mirror the dispatch's private-rebind so the by-ref mask matches the
        // method that will actually run.
        let (defc, midx) = parent_private_rebind(
            &self.classes,
            self.frames[top].class,
            cid,
            method,
            resolved,
        )
        .or(resolved)?;
        if !method_visible_from(
            &self.classes,
            self.frames[top].class,
            self.classes[defc].methods[midx].visibility,
            defc,
            method,
        ) {
            return None; // routes to `__call`: arguments are by-value
        }
        let cls = self.classes[defc];
        Some(&cls.methods[midx].func)
    }

    /// The static-call analogue of [`Self::instance_arg_ref_target`]:
    /// `None` (all by-value) for a missing/invisible method (`__callStatic`).
    fn static_arg_ref_target(&self, top: usize, start: ClassId, method: &[u8]) -> Option<&'m Func> {
        let (defc, midx) = resolve_method_runtime(&self.classes, start, method)?;
        if !method_visible_from(
            &self.classes,
            self.frames[top].class,
            self.classes[defc].methods[midx].visibility,
            defc,
            method,
        ) {
            return None;
        }
        let cls = self.classes[defc];
        Some(&cls.methods[midx].func)
    }

    /// Zend's SEND_VAR_EX decision point for deferred place arguments
    /// ([`Zval::ArgPlace`], pushed by [`Op::PushArgPlace`]): resolve each
    /// against the callee's by-reference mask. A by-ref parameter W-fetches
    /// the place via [`Self::make_ref_cell`] (aliases it, silently creating a
    /// missing key); a by-value one — or any argument of a `None` callee
    /// (native / `__call` / unresolved) — R-fetches it (plain read semantics:
    /// "Undefined variable"/"Undefined array key" warnings, nothing created).
    /// Runs in the *caller's* frame, before the callee frame is built.
    fn materialize_arg_places(
        &mut self,
        top: usize,
        args: &mut [Zval],
        callee: Option<&Func>,
    ) -> Result<(), PhpError> {
        for (i, a) in args.iter_mut().enumerate() {
            let Zval::ArgPlace(p) = a else { continue };
            let p = Rc::clone(p);
            let by_ref = callee.is_some_and(|f| match f.variadic_slot {
                Some(v) if i >= v as usize => f.param_by_ref.get(v as usize).copied().unwrap_or(false),
                _ => f.param_by_ref.get(i).copied().unwrap_or(false),
            });
            let base = match p.base {
                ArgPlaceBase::Local(s) => FieldBase::Local(s),
                ArgPlaceBase::Global(s) => FieldBase::Global(s),
                ArgPlaceBase::Superglobal(x) => FieldBase::Superglobal(x),
                ArgPlaceBase::This => FieldBase::This,
            };
            *a = if by_ref {
                let steps: Vec<FieldStep> = p
                    .steps
                    .iter()
                    .map(|s| match s {
                        ArgPlaceStep::Index => FieldStep::Index,
                        ArgPlaceStep::Prop(n) => FieldStep::Prop(n.clone()),
                        // `f($a[])` to a by-ref param: field_cell appends a
                        // fresh element and aliases it (PclZip, WP-17).
                        ArgPlaceStep::Append => FieldStep::Append,
                    })
                    .collect();
                let cell = self.make_ref_cell(top, base, &steps, p.keys.clone())?;
                Zval::Ref(cell)
            } else {
                self.arg_place_read(top, base, &p)?
            };
        }
        Ok(())
    }

    /// R-fetch of a deferred place argument (the by-value branch of
    /// SEND_VAR_EX): read the root like `Op::LoadVar` (an `Undef` slot warns
    /// "Undefined variable"), then walk each step — `Index` like
    /// `Op::FetchDim` (`offsetGet` for an ArrayAccess object, a warning read
    /// otherwise), `Prop` like `Op::PropGet` driven synchronously.
    fn arg_place_read(&mut self, top: usize, base: FieldBase, p: &ArgPlace) -> Result<Zval, PhpError> {
        let mut v = match base {
            FieldBase::Local(s) => {
                if matches!(self.frames[top].slots[s as usize], Zval::Undef) && !p.name.is_empty() {
                    self.diags.push(Diag::Warning(format!(
                        "Undefined variable ${}",
                        String::from_utf8_lossy(&p.name)
                    )));
                }
                arrays::read_slot(&self.frames[top].slots[s as usize])
            }
            FieldBase::Global(s) => {
                if matches!(self.frames[0].slots[s as usize], Zval::Undef) && !p.name.is_empty() {
                    self.diags.push(Diag::Warning(format!(
                        "Undefined variable ${}",
                        String::from_utf8_lossy(&p.name)
                    )));
                }
                arrays::read_slot(&self.frames[0].slots[s as usize])
            }
            FieldBase::Superglobal(i) => self.superglobals[i as usize].deref_clone(),
            FieldBase::This => match &self.frames[top].this {
                Some(t) => t.deref_clone(),
                None => {
                    return Err(PhpError::Error(
                        "Using $this when not in object context".to_string(),
                    ))
                }
            },
        };
        let mut keys = p.keys.iter();
        for step in p.steps.iter() {
            match step {
                ArgPlaceStep::Index => {
                    let k = keys.next().expect("arg place key per Index step");
                    if let Some(recv) = self.as_arrayaccess(&v) {
                        v = self.call_method_sync(recv, b"offsetGet", vec![k.clone()])?;
                    } else {
                        v = arrays::read_dim_warn(&v, k, &mut self.diags)?;
                    }
                }
                ArgPlaceStep::Prop(n) => {
                    v = self.prop_read_sync(top, v, n)?;
                }
                // `f($a[])` resolved against a by-VALUE parameter (or an
                // unknown/builtin callee): PHP's runtime Error, raised at the
                // call site (oracle: method call with by-value param).
                ArgPlaceStep::Append => {
                    return Err(PhpError::Error("Cannot use [] for reading".to_string()));
                }
            }
        }
        Ok(v)
    }

    /// Synchronous property read with `Op::PropGet` semantics, for the
    /// R-branch of a deferred place argument: lazy initialization, `get`
    /// hooks and `__get` (driven to completion inline), visibility check,
    /// private-storage key, uninitialized-typed-property error, and the
    /// "Undefined property"/"Attempt to read property" warnings.
    fn prop_read_sync(&mut self, top: usize, obj: Zval, name: &[u8]) -> Result<Zval, PhpError> {
        let cur = self.frames[top].class;
        let target = self.lazy_prop_access(obj, name, cur, Some(false), (MagicKind::Get, b"__get"))?;
        let mut key = name.to_vec();
        if let Zval::Object(o) = &target {
            let (oid, cid) = {
                let b = o.borrow();
                (b.id, b.class_id as usize)
            };
            if !self.hook_guarded(oid, name) {
                if let Some(func) = self.prop_hook(cid, name, false) {
                    let baseline = self.frames.len();
                    self.push_hook(func, target.clone(), oid, name, None);
                    return self.drive_to_return(baseline);
                }
                // A virtual hooked property with no get hook is write-only.
                if self.is_virtual_hooked(cid, name) {
                    return Err(PhpError::Error(format!(
                        "Property {}::${} is write-only",
                        String::from_utf8_lossy(&self.classes[cid].name),
                        String::from_utf8_lossy(name),
                    )));
                }
            }
            if let Some((defc, midx, oid)) = self.magic_applies(o, name, cur, MagicKind::Get, b"__get") {
                let baseline = self.frames.len();
                self.push_magic_prop(defc, midx, oid, MagicKind::Get, target.clone(), name, None, None, false);
                return self.drive_to_return(baseline);
            }
            check_prop_access(&self.classes, cur, o.borrow().class_id as usize, name)?;
            key = self.prop_storage_key(o.borrow().class_id as usize, name, cur);
            if let Some(err) = self.uninit_typed_read(o, &key, name) {
                return Err(err);
            }
        }
        Ok(read_property(&target, &key, &mut self.diags))
    }

    /// `start` is already resolved (OOP-2a). `forwarding` is true for
    /// `self`/`parent`/`static` (keep the caller's LSB class and `$this`), false
    /// for a named/dynamic class (rebind LSB; forward `$this` only when the
    /// receiver is in `start`'s hierarchy). A missing or inaccessible target
    /// routes to `__call` on `$this` (in object context) or `__callStatic`,
    /// otherwise raises the visibility / undefined-method error. Shared by
    /// `Op::StaticCall` (and, later, the dynamic `$cls::method()` path).
    /// `named` arguments (from a runtime argument array's string keys) bind via
    /// [`build_named_frame`], or ride string-keyed in a magic `$args` array.
    fn dispatch_static_call(
        &mut self,
        top: usize,
        start: ClassId,
        method: &[u8],
        forwarding: bool,
        mut args: Vec<Zval>,
        named: Vec<(Box<[u8]>, Zval)>,
    ) -> Result<(), PhpError> {
        // Deferred place arguments (SEND_VAR_EX) resolve against the callee's
        // by-ref mask now that it is known; an unresolved/invisible target
        // (`__callStatic`, enum builtin) takes every argument by value.
        if args.iter().any(|a| matches!(a, Zval::ArgPlace(_))) {
            let callee = self.static_arg_ref_target(top, start, method);
            self.materialize_arg_places(top, &mut args, callee)?;
            // R-fetch warnings report the CALL's line, not the callee's next
            // emit point.
            let line = self.cur_line(top);
            self.flush_diags(line)?;
        }
        // Enum built-in statics (`cases` / `from` / `tryFrom`) are reserved names
        // that shadow user resolution and produce a value directly rather than
        // entering a frame (step 23). `cases` is on every enum; `from`/`tryFrom`
        // only on a backed one.
        if !self.classes[start].enum_cases.is_empty() {
            if method.eq_ignore_ascii_case(b"cases") {
                let v = self.vm_enum_cases(start);
                self.frames[top].stack.push(v);
                return Ok(());
            }
            let backed = self.classes[start].enum_cases.iter().any(|c| c.value.is_some());
            if backed {
                let try_from = method.eq_ignore_ascii_case(b"tryFrom");
                if try_from || method.eq_ignore_ascii_case(b"from") {
                    // Decay a reference pushed by a dynamic call (SEND_VAR_EX).
                    // The single parameter is `$value`, so a named argument (or a
                    // spread's string key) binds by that name.
                    let arg = args
                        .into_iter()
                        .next()
                        .or_else(|| {
                            named.into_iter().find(|(n, _)| &n[..] == b"value").map(|(_, v)| v)
                        })
                        .map(decay_arg);
                    let v = self.vm_enum_from(start, arg, try_from)?;
                    self.frames[top].stack.push(v);
                    return Ok(());
                }
            }
        }
        let resolved = resolve_method_runtime(&self.classes, start, method);
        let usable = resolved.filter(|&(defc, midx)| {
            method_visible_from(&self.classes, self.frames[top].class, self.classes[defc].methods[midx].visibility, defc, method)
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
                    || matches!(object_class_id(t), Some(ocid) if class_is_a(&self.classes, ocid, start));
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
                let callee = &self.classes[defc].methods[midx].func;
                let mut frame = if named.is_empty() {
                    let mut frame = Frame::new(callee, self.class_mod(defc));
                    bind_params(&mut frame, args);
                    frame
                } else {
                    let qn = format!(
                        "{}::{}",
                        String::from_utf8_lossy(&self.classes[defc].name),
                        String::from_utf8_lossy(method)
                    );
                    build_named_frame(callee, self.class_mod(defc), &qn, args, named)?
                };
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
                        resolve_method_runtime(&self.classes, oc, b"__call").map(|(d, m)| (tv, oc, d, m))
                    });
                // `__call`/`__callStatic($name, $args)` gets a value array — decay
                // references pushed by a dynamic call (SEND_VAR_EX); named values
                // ride string-keyed alongside.
                if let Some((tv, oc, cdefc, cmidx)) = via_call {
                    self.push_magic_call_named(cdefc, cmidx, Some(tv), oc, method, decay_args(args), named);
                } else if let Some((cdefc, cmidx)) =
                    resolve_method_runtime(&self.classes, start, b"__callStatic")
                {
                    self.push_magic_call_named(cdefc, cmidx, None, start, method, decay_args(args), named);
                } else {
                    return Err(match resolved {
                        Some((defc, midx)) => method_access_error(
                            &self.classes,
                            defc,
                            method,
                            self.frames[top].class,
                            self.classes[defc].methods[midx].visibility,
                        ),
                        None => undefined_method(&self.classes, start, method),
                    });
                }
            }
        }
        Ok(())
    }

    fn resolve_dynamic_class(&mut self, v: &Zval) -> Result<ClassId, PhpError> {
        match v.deref_clone() {
            Zval::Object(o) => Ok(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                // Try the live table, then autoload (Phase 3) and retry, before the
                // catchable "Class not found".
                match self.resolve_class_autoload(name)? {
                    Some(id) => {
                        let got = &self.classes[id].name;
                        if !got.eq_ignore_ascii_case(name) {
                            log::debug!(
                                target: "phpr::attr",
                                "resolve_dynamic_class MISMATCH: asked {:?} got cid {} = {:?}",
                                String::from_utf8_lossy(name),
                                id,
                                String::from_utf8_lossy(got)
                            );
                        }
                        Ok(id)
                    }
                    None => Err(PhpError::Error(format!(
                        "Class \"{}\" not found",
                        String::from_utf8_lossy(name)
                    ))),
                }
            }
            _ => Err(PhpError::Error(
                "Class name must be a valid object or a string".to_string(),
            )),
        }
    }

    /// Resolve a class `name` to its id, running the registered autoloaders on a
    /// miss and retrying (step 57, Phase 3). Returns `None` if still undefined
    /// after autoloading; a throwing autoloader propagates.
    fn resolve_class_autoload(&mut self, name: &[u8]) -> Result<Option<ClassId>, PhpError> {
        let bare = name.strip_prefix(b"\\").unwrap_or(name);
        let key = bare.to_ascii_lowercase();
        if let Some(&id) = self.class_index.get(&key) {
            return Ok(Some(id));
        }
        // Zend keeps traits in the same class table as classes/interfaces: a
        // name already declared as a trait is "found" (so class_exists() is
        // false WITHOUT re-running the autoloader — the re-include would
        // collide with the file's other declarations: ReflectionClass on
        // PriorityTaggedServiceTrait, whose file also declares a class).
        if self.trait_declared(&key) {
            return Ok(None);
        }
        self.try_autoload(bare, &key)?;
        Ok(self.class_index.get(&key).copied())
    }

    /// Whether `key` (a fully-qualified name) matches a declared trait's real
    /// name, case-insensitively. The autoload paths treat a known trait as
    /// "already declared", mirroring Zend's single class table.
    pub(super) fn trait_declared(&self, key: &[u8]) -> bool {
        self.seed_traits.iter().any(|(_, t)| t.name.eq_ignore_ascii_case(key))
    }

    /// Whether `name` resolves (after autoload) to a class/interface *or* a trait.
    /// Used by the include-time lowering retry: an undefined name surfaced during
    /// lowering may be a parent/interface class or a `use`d trait, and a trait
    /// lands in `seed_traits` (keyed by bare lowercase name), not `class_index`.
    fn resolve_name_autoload(&mut self, name: &[u8]) -> Result<bool, PhpError> {
        let bare = name.strip_prefix(b"\\").unwrap_or(name);
        let key = bare.to_ascii_lowercase();
        let trait_key = bare.rsplit(|&b| b == b'\\').next().unwrap_or(bare).to_ascii_lowercase();
        let known = |s: &Self| {
            s.class_index.contains_key(&key) || s.seed_traits.iter().any(|(k, _)| *k == trait_key)
        };
        if known(self) {
            return Ok(true);
        }
        self.try_autoload(bare, &key)?;
        Ok(known(self))
    }

    /// Run the registered `spl_autoload_register` callbacks for `name` (already
    /// stripped of a leading `\`, with `key` its lowercased form), each given the
    /// class name, until one defines it or all are tried (step 57, Phase 3). A
    /// recursion guard prevents an autoloader that references the same name from
    /// looping. A throwing autoloader propagates.
    fn try_autoload(&mut self, name: &[u8], key: &[u8]) -> Result<(), PhpError> {
        if self.autoloaders.is_empty() || self.autoloading.contains(key) {
            return Ok(());
        }
        self.autoloading.insert(key.to_vec());
        log::debug!(target: "phpr::autoload", "autoload {}", String::from_utf8_lossy(name));
        let arg = Zval::Str(PhpStr::new(name.to_vec()));
        let loaders = self.autoloaders.clone();
        let mut outcome = Ok(());
        for loader in loaders {
            if let Err(e) = self.call_callable(loader, vec![arg.clone()]) {
                outcome = Err(e);
                break;
            }
            if self.class_index.contains_key(key) || self.trait_declared(key) {
                break;
            }
        }
        self.autoloading.remove(key);
        outcome
    }





    /// The "must not be accessed before initialization" fatal for reading a typed
    /// property that is still uninitialized (its slot holds `Zval::Undef`). Covers
    /// both plain and readonly typed properties (a readonly one is always typed and
    /// default-less, so it too starts `Undef`). `None` if the property is absent or
    /// holds a real value. The message names the *declaring* class.
    /// The "must not be accessed before initialization" fatal for an uninitialised
    /// typed property, or `None` if it is set. `key` is the storage slot (mangled
    /// for a private); `name` is the plain name used for the type lookup and message.
    fn uninit_typed_read(&self, o: &Rc<RefCell<Object>>, key: &[u8], name: &[u8]) -> Option<PhpError> {
        let b = o.borrow();
        match b.props.get(key) {
            // Never initialized: the slot still holds `Undef`.
            Some(Zval::Undef) => {}
            Some(_) => return None,
            // Explicitly `unset()`: the entry was removed. Every caller sits
            // after the `__get` dispatch attempt, so reaching here means no
            // magic handled it — for a *declared typed* property that is the
            // same before-init fatal (Zend); an untyped/undeclared name falls
            // through to the ordinary undefined-property read.
            None => {
                let ocid = b.class_id as usize;
                drop(b);
                let (decl, _) = prop_type_decl(&self.classes, ocid, name)?;
                return Some(PhpError::Error(format!(
                    "Typed property {}::${} must not be accessed before initialization",
                    String::from_utf8_lossy(&self.classes[decl].name),
                    String::from_utf8_lossy(name),
                )));
            }
        }
        let ocid = b.class_id as usize;
        drop(b);
        let decl = prop_type_decl(&self.classes, ocid, name).map(|(c, _)| c).unwrap_or(ocid);
        Some(PhpError::Error(format!(
            "Typed property {}::${} must not be accessed before initialization",
            String::from_utf8_lossy(&self.classes[decl].name),
            String::from_utf8_lossy(name),
        )))
    }

    /// The storage key for an instance-property access `obj(ocid)->name` from
    /// `scope`: the plain name today, a mangled `\0Class\0name` for an accessible
    /// private once mangling is on. Per-instance state (props, readonly tracking)
    /// is keyed by this; visibility is enforced separately via
    /// [`resolve_prop_access`] / `check_prop_access`.
    fn prop_storage_key(&self, ocid: ClassId, name: &[u8], scope: Option<ClassId>) -> Vec<u8> {
        match resolve_prop_access(&self.classes, ocid, name, scope) {
            PropAccess::Slot(k) => k,
            PropAccess::Dynamic | PropAccess::Denied { .. } => name.to_vec(),
        }
    }

    /// Unconditional storage key for the most-derived declaration of `name` on
    /// `ocid`, ignoring accessing scope/visibility — for a write that must target
    /// the declared slot regardless of scope (the property-init thunk). Falls back
    /// to the plain name for an undeclared property.
    fn prop_decl_storage_key(&self, ocid: ClassId, name: &[u8]) -> Vec<u8> {
        prop_info(&self.classes, ocid, name).map(|pi| pi.storage_key.to_vec()).unwrap_or_else(|| name.to_vec())
    }

    /// Storage key host (Rust) code uses for a *base-declared* internal property
    /// (`Exception::$trace` / `$traceString`, `Fiber::$callable`): the slot of the
    /// root-most ancestor of `ocid` that declares `name` — what Zend's
    /// `zend_read_property(scope = base_ce)` addresses — unaffected by a same-name
    /// private redeclaration in a user subclass. Plain name if nowhere declared.
    fn host_prop_key(&self, ocid: ClassId, name: &[u8]) -> Vec<u8> {
        let mut found: Option<&PropInfo> = None;
        let mut cur = Some(ocid);
        while let Some(c) = cur {
            let Some(cc) = self.classes.get(c) else { break };
            if let Some(pi) = cc.prop_info.get(name) {
                if pi.declaring_class == c {
                    found = Some(pi);
                }
            }
            cur = cc.parent;
        }
        found.map(|pi| pi.storage_key.to_vec()).unwrap_or_else(|| name.to_vec())
    }

    /// The fatal raised by a compound write (`+=`, `++`, …) to a `readonly`
    /// property: always an error — "Cannot modify readonly property" once
    /// initialised, or the before-initialization fatal if the implicit read of an
    /// uninitialised one comes first. `None` for a non-readonly property.
    fn readonly_rmw_error(&self, obj: &Zval, key: &[u8], name: &[u8]) -> Option<PhpError> {
        let target = obj.deref_clone();
        let Zval::Object(o) = &target else { return None };
        let ocid = o.borrow().class_id as usize;
        let decl = prop_readonly_decl(&self.classes, ocid, name)?;
        // A compound write during `__clone` is the permitted one re-init (8.3).
        if o.borrow().readonly_clone_writable(key) {
            let mut ob = o.borrow_mut();
            ob.consume_clone_writable(key);
            ob.mark_readonly_init(key);
            return None;
        }
        let cls = String::from_utf8_lossy(&self.classes[decl].name).into_owned();
        let prop = String::from_utf8_lossy(name).into_owned();
        Some(PhpError::Error(if o.borrow().is_readonly_init(key) {
            format!("Cannot modify readonly property {cls}::${prop}")
        } else {
            format!("Typed property {cls}::${prop} must not be accessed before initialization")
        }))
    }

    /// The `new`-site constructor visibility check (Zend's
    /// zend_std_get_constructor): a private/protected `__construct` — possibly
    /// inherited — must be visible from the calling scope `cur`, else
    /// "Call to {private,protected} C::__construct() from {scope}" naming the
    /// DECLARING class (no "method" in the wording). Non-instantiable classes
    /// return Ok so [`Self::alloc_object`]'s abstract/interface/enum fatal wins.
    /// Only `new` runs this — internal allocations (unserialize, reflection,
    /// host code) bypass constructor visibility like Zend's object_init does.
    fn check_new_ctor_access(&self, cur: Option<ClassId>, cid: ClassId) -> Result<(), PhpError> {
        let Some(cc) = self.classes.get(cid) else { return Ok(()) };
        if !matches!(cc.instantiable, Instantiable::Yes) {
            return Ok(());
        }
        let Some((defc, midx)) = resolve_method_runtime(&self.classes, cid, b"__construct") else {
            return Ok(());
        };
        let vis = self.classes[defc].methods[midx].visibility;
        if visible_from(&self.classes, cur, vis, defc) {
            return Ok(());
        }
        let kind = if matches!(vis, Visibility::Private) { "private" } else { "protected" };
        let scope = match cur {
            Some(c) => format!("scope {}", String::from_utf8_lossy(&self.classes[c].name)),
            None => "global scope".to_string(),
        };
        Err(PhpError::Error(format!(
            "Call to {kind} {}::__construct() from {scope}",
            String::from_utf8_lossy(&self.classes[defc].name)
        )))
    }

    /// Build a fresh instance of class `cid`: its declared property defaults
    /// materialised, a fresh handle id, shared class-name / visibility metadata.
    /// Fatal if the class is non-instantiable (abstract / interface / enum) or
    /// could not be compiled. Shared by [`Op::Alloc`] and [`Op::AllocStatic`].
    fn alloc_object(&mut self, cid: ClassId) -> Result<Zval, PhpError> {
        let cc = self.classes[cid]; // &'m CompiledClass: detach from `self` borrow

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
        // A typed property with no default starts uninitialized: overwrite its NULL
        // placeholder with `Undef` (kept in declaration order). Reading it errors;
        // `var_dump` renders `uninitialized(T)`; `isset` is false.
        for name in &cc.uninit_props {
            props.set(name, Zval::Undef);
        }
        let class_name = Rc::clone(&cc.class_name);
        let info = Rc::clone(&cc.info);
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info, readonly_init: Vec::new(), readonly_clone_writable: Vec::new(), typed_unset: Vec::new(), lazy: None, proxy_instance: None };
        let rc = Rc::new(RefCell::new(obj));
        // Track for `__destruct` (OOP-3d): the extra strong ref drives the sweep.
        self.created.insert(id, Rc::clone(&rc));
        self.gc_track(&rc);
        Ok(Zval::Object(rc))
    }

    /// Allocate an *uninitialized* lazy object of `cid` (PHP 8.4
    /// `ReflectionClass::newLazyGhost` / `newLazyProxy`): a real instance whose
    /// declared properties are all left uninitialized — typed ones as
    /// `uninitialized(T)`, untyped ones absent — with no default applied yet. The
    /// instantiability check and `#id` assignment are a normal alloc; the
    /// initializer/factory is stashed in `lazy_init` and runs on first access
    /// (see [`Self::realize_lazy`]). `kind` selects ghost vs proxy semantics.
    fn alloc_lazy(&mut self, cid: ClassId, init: Zval, kind: LazyKind, options: u32) -> Result<Zval, PhpError> {
        self.reject_internal_lazy(cid)?;
        let v = self.alloc_object(cid)?;
        if let Zval::Object(rc) = &v {
            self.install_lazy(rc, cid, kind, init, options)?;
        }
        Ok(v)
    }

    /// PHP 8.4 refuses to make instances of *internal* classes lazy (their
    /// native state cannot be left uninitialized) — `stdClass` is the one
    /// documented exception, and a userland subclass of an internal class is
    /// rejected naming the internal ancestor. phpr's "internal" classes are
    /// the prelude-defined ones.
    fn reject_internal_lazy(&self, cid: ClassId) -> Result<(), PhpError> {
        let mut c = Some(cid);
        while let Some(ci) = c {
            let cc = self.classes[ci];
            if cc.file.as_ref() == b"prelude" && cc.name.as_ref() != b"stdClass" {
                // The wording distinguishes the internal class itself from a
                // userland subclass inheriting it.
                return Err(PhpError::Error(if ci == cid {
                    format!(
                        "Cannot make instance of internal class lazy: {} is internal",
                        String::from_utf8_lossy(&cc.name),
                    )
                } else {
                    format!(
                        "Cannot make instance of internal class lazy: {} inherits internal class {}",
                        String::from_utf8_lossy(&self.classes[cid].name),
                        String::from_utf8_lossy(&cc.name),
                    )
                }));
            }
            c = cc.parent;
        }
        Ok(())
    }

    /// Turn an existing object handle `rc` (of class `cid`) into an
    /// *uninitialized* lazy object: rebuild its properties to the lazy layout
    /// (typed → `uninitialized(T)`, untyped absent), clear any proxy instance,
    /// set the lazy marker, and stash the initializer/factory + per-property
    /// still-lazy set. Shared by [`Self::alloc_lazy`] (fresh alloc) and the
    /// `resetAsLazy*` reflection path (an already-live instance).
    fn install_lazy(&mut self, rc: &Rc<RefCell<Object>>, cid: ClassId, kind: LazyKind, init: Zval, options: u32) -> Result<(), PhpError> {
        let ocid = rc.borrow().class_id as usize;
        // The properties that become lazy are those of the *reflected* class's
        // layout (`cid`): a `resetAsLazy*` through a parent reflector preserves
        // the subclass's additional properties and the dynamic ones
        // (`reset_as_lazy_ignores_additional_props`). A fresh `newLazy*`
        // allocation passes the object's own class, so everything resets.
        let reflected: HashSet<&[u8]> =
            self.classes[cid].prop_defaults.iter().map(|(n, _)| n.as_ref()).collect();
        // An *initialized* readonly property declared by a class other than
        // the reflected one also keeps its value and its initialized mark —
        // only the reflected class's own readonly slots are unlocked for the
        // initializer (`reset_as_lazy_readonly`, zend_object_make_lazy).
        let preserved_ro: HashSet<Box<[u8]>> = {
            let b = rc.borrow();
            b.readonly_init
                .iter()
                .filter(|k| {
                    reflected.contains(k.as_ref())
                        && prop_readonly_decl(&self.classes, ocid, php_types::prop_display_name(k))
                            .is_some_and(|decl| decl != cid)
                })
                .cloned()
                .collect()
        };
        // Every reset slot starts lazy; a skip / raw-set later removes one.
        let still_lazy: Vec<Box<[u8]>> = self.classes[cid]
            .prop_defaults
            .iter()
            .map(|(n, _)| n.clone())
            .filter(|n| !preserved_ro.contains(n))
            .collect();
        let (oid, dropped) = {
            let mut b = rc.borrow_mut();
            // Rebuild the layout in the object's full declaration order: a
            // reset slot goes typed-`Undef` / untyped-absent (its displaced
            // value is captured so its references can be released as possible
            // GC roots — resetting a live object runs the destructors of what
            // it held, matching PHP); a preserved slot keeps its value.
            let mut props = Props::new();
            let mut dropped: Vec<Zval> = Vec::new();
            let mut declared: HashSet<&[u8]> = HashSet::default();
            for (key, _) in &self.classes[ocid].prop_defaults {
                declared.insert(key.as_ref());
                if reflected.contains(key.as_ref()) && !preserved_ro.contains(key) {
                    // `prop_defaults` is storage-keyed; metadata speaks names.
                    if prop_type_decl(&self.classes, ocid, php_types::prop_display_name(key)).is_some() {
                        props.set(key, Zval::Undef);
                    }
                    if let Some(old) = b.props.get(key) {
                        dropped.push(old.clone());
                    }
                } else if let Some(cur) = b.props.get(key) {
                    props.set(key, cur.clone());
                }
            }
            // Dynamic properties are always reset — only *declared* properties
            // outside the reflected layout are preserved
            // (`reset_as_lazy_resets_dynamic_props`).
            for (key, val) in b.props.iter() {
                if !declared.contains(key) {
                    dropped.push(val.clone());
                }
            }
            if let Some(inst) = &b.proxy_instance {
                dropped.push((**inst).clone());
            }
            b.props = props;
            b.proxy_instance = None;
            // The lazy marker is set only *after* the displaced contents'
            // destructors below: they observe the target already emptied but
            // not yet lazy — `zend_object_make_lazy` sets the flag last
            // (`reset_as_lazy_can_reset_initialized_proxies`' dump).
            b.lazy = None;
            b.readonly_init.retain(|k| preserved_ro.contains(k));
            (b.id, dropped)
        };
        // The reset discards the property table, detaching any reference that
        // aliased a typed slot: Zend deletes the reference's source type, so a
        // later write through the alias is unchecked
        // (`reset_as_lazy_deletes_reference_source_type`).
        if !self.typed_refs.is_empty() {
            let owner = Rc::as_ptr(rc);
            self.typed_refs.retain(|t| !std::ptr::eq(t.obj.as_ptr(), owner));
        }
        // Release the displaced contents and run any destructor they held the
        // last reference to *synchronously* — PHP frees them inside the reset
        // itself, not at the next statement boundary. A fresh alloc has
        // nothing to release and skips the sweep.
        let had_content = !dropped.is_empty();
        for v in &dropped {
            self.gc_note(v);
        }
        drop(dropped);
        if had_content {
            self.gc_sweep_impl(None, true)?;
        }
        // A class with no eligible (reset) properties is never uninitialized
        // (PHP 8.4, support_stdClass): the object stays a plain instance and
        // the initializer is dropped without running.
        if still_lazy.is_empty() {
            return Ok(());
        }
        rc.borrow_mut().lazy = Some(kind);
        self.lazy_init.insert(oid, init);
        // Realization applies class defaults to these (still-lazy) slots only;
        // everything else keeps its preserved value.
        self.lazy_props.insert(oid, still_lazy);
        self.lazy_options.insert(oid, options);
        Ok(())
    }

    /// Initialize a lazy object (PHP 8.4). A **ghost** gets the class's real
    /// property defaults applied, its lazy marker cleared, then runs its pending
    /// initializer with the object (becoming an ordinary instance). A **proxy**
    /// calls its factory (which returns the real instance), stashes that in
    /// `proxy_instance`, and keeps its `Some(Proxy)` marker for life — later
    /// property access forwards to the instance. A no-op for a non-lazy or
    /// already-initialized object.
    fn realize_lazy(&mut self, v: &Zval) -> Result<(), PhpError> {
        let Some(rc) = deref_object(v) else { return Ok(()) };
        let (oid, cid, kind) = {
            let b = rc.borrow();
            match b.lazy {
                // Uninitialized: a ghost (lazy set), or a proxy with no instance yet.
                Some(k) if b.proxy_instance.is_none() => (b.id, b.class_id as usize, k),
                // Non-lazy, or an already-initialized proxy.
                _ => return Ok(()),
            }
        };
        match kind {
            LazyKind::Ghost => {
                // Snapshot the wrapper for rollback: an initializer exception
                // leaves the object LAZY, reverting any change the initializer
                // made to it (PHP 8.4, init_exception_reverts_initializer_changes).
                let saved_props = rc.borrow().props.clone();
                let saved_ro = rc.borrow().readonly_init.clone();
                let saved_lazy_props = self.lazy_props.get(&oid).cloned();
                // The initializer may `unset()` properties, deleting their
                // typed-reference sources — a rollback resurrects them
                // (init_handles_ref_source_types_exception).
                let saved_typed: Vec<TypedRefSource> = {
                    let op_ptr = Rc::as_ptr(&rc);
                    self.typed_refs
                        .iter()
                        .filter(|t| std::ptr::eq(t.obj.as_ptr(), op_ptr))
                        .cloned()
                        .collect()
                };
                {
                    let cc = self.classes[cid];
                    // Rebuild the property layout in declaration order (the ghost held
                    // only the typed placeholders, so an incremental `set` would
                    // mis-order the untyped ones appended on top). Defaults apply to
                    // the *still-lazy* slots only: a skipped/raw-set property, a
                    // subclass property outside a parent-reflector reset, an
                    // initialized foreign readonly, and dynamics all keep their
                    // current values.
                    let still: HashSet<Box<[u8]>> = self
                        .lazy_props
                        .get(&oid)
                        .map(|v| v.iter().cloned().collect())
                        .unwrap_or_default();
                    let mut b = rc.borrow_mut();
                    let mut props = Props::new();
                    let mut declared: HashSet<&[u8]> = HashSet::default();
                    for (name, c) in &cc.prop_defaults {
                        declared.insert(name.as_ref());
                        if still.contains(name.as_ref()) {
                            props.set(name, c.to_zval());
                        } else if let Some(cur) = b.props.get(name) {
                            props.set(name, cur.clone());
                        }
                    }
                    for name in &cc.uninit_props {
                        if still.contains(name.as_ref()) {
                            props.set(name, Zval::Undef);
                        }
                    }
                    for (name, val) in b.props.iter() {
                        if !declared.contains(name) {
                            props.set(name, val.clone());
                        }
                    }
                    b.lazy = None;
                    b.props = props;
                }
                self.lazy_props.remove(&oid);
                if let Some(init) = self.lazy_init.remove(&oid) {
                    // The ghost initializer populates the object; its return is
                    // ignored. The object is marked "initializing" so a re-entrant
                    // `resetAsLazy*` on it errors (cleared even if it throws).
                    self.lazy_initializing.insert(oid);
                    let r = self.call_callable(init.clone(), vec![v.clone()]);
                    self.lazy_initializing.remove(&oid);
                    // A ghost initializer must return NULL/no value (PHP 8.4);
                    // any failure rolls the wrapper back to its lazy state with
                    // the initializer reinstalled (a later access retries).
                    let r = match r {
                        Ok(ret) if !matches!(ret.deref_clone(), Zval::Null | Zval::Undef) => {
                            Err(PhpError::TypeError(
                                "Lazy object initializer must return NULL or no value".to_string(),
                            ))
                        }
                        other => other.map(|_| ()),
                    };
                    if let Err(e) = r {
                        {
                            let mut b = rc.borrow_mut();
                            b.props = saved_props;
                            b.readonly_init = saved_ro;
                            b.lazy = Some(LazyKind::Ghost);
                        }
                        if let Some(sp) = saved_lazy_props {
                            self.lazy_props.insert(oid, sp);
                        }
                        {
                            let op_ptr = Rc::as_ptr(&rc);
                            self.typed_refs.retain(|t| !std::ptr::eq(t.obj.as_ptr(), op_ptr));
                            self.typed_refs.extend(saved_typed);
                        }
                        self.lazy_init.insert(oid, init);
                        return Err(e);
                    }
                }
            }
            LazyKind::Proxy => {
                let saved_lazy_props = self.lazy_props.remove(&oid);
                // Remove the factory *before* invoking it so a re-entrant access on
                // the proxy during the factory does not recurse (it sees no factory
                // and bails). The factory's return is the real instance to forward to.
                if let Some(factory) = self.lazy_init.remove(&oid) {
                    self.lazy_initializing.insert(oid);
                    let r = self.call_callable(factory.clone(), vec![v.clone()]);
                    self.lazy_initializing.remove(&oid);
                    // Validate the factory's return: an object of the proxy's
                    // own class, or of an ancestor the proxy subclasses without
                    // additional properties or __destruct/__clone overrides
                    // (initializer_must_return_the_right_type).
                    let r = r.and_then(|real| {
                        let rd = real.deref_clone();
                        let Some(io) = deref_object(&rd) else {
                            return Err(PhpError::TypeError(format!(
                                "Lazy proxy factory must return an instance of a class compatible with {}, {} returned",
                                String::from_utf8_lossy(&self.classes[cid].name),
                                rd.type_name_for_error(),
                            )));
                        };
                        {
                            let b = io.borrow();
                            if b.lazy.is_some() && b.proxy_instance.is_none() {
                                return Err(PhpError::Error(
                                    "Lazy proxy factory must return a non-lazy object".to_string(),
                                ));
                            }
                        }
                        let icid = io.borrow().class_id as usize;
                        let decl_of = |c: ClassId, m: &[u8]| {
                            resolve_method_runtime(&self.classes, c, m).map(|(d, _)| d)
                        };
                        let ok = icid == cid
                            || (is_instance_of(&self.classes, self.stringable_id, cid, icid)
                                && self.classes[cid].prop_defaults.len()
                                    == self.classes[icid].prop_defaults.len()
                                && decl_of(cid, b"__destruct") == decl_of(icid, b"__destruct")
                                && decl_of(cid, b"__clone") == decl_of(icid, b"__clone"));
                        if !ok {
                            return Err(PhpError::TypeError(format!(
                                "The real instance class {} is not compatible with the proxy class {}. The proxy must be a instance of the same class as the real instance, or a sub-class with no additional properties, and no overrides of the __destructor or __clone methods.",
                                String::from_utf8_lossy(&self.classes[icid].name),
                                String::from_utf8_lossy(&self.classes[cid].name),
                            )));
                        }
                        Ok(rd)
                    });
                    match r {
                        Ok(real) => {
                            // The wrapper's own property table is superseded by
                            // the instance: its typed-reference sources die with
                            // it (init_handles_ref_source_types' proxy half —
                            // propagating pre-initialized props is the
                            // factory's job).
                            if !self.typed_refs.is_empty() {
                                let wp = Rc::as_ptr(&rc);
                                self.typed_refs.retain(|t| !std::ptr::eq(t.obj.as_ptr(), wp));
                            }
                            rc.borrow_mut().proxy_instance = Some(Box::new(real));
                        }
                        Err(e) => {
                            // A factory exception (or invalid return) leaves the
                            // proxy uninitialized and retryable (PHP 8.4).
                            if let Some(sp) = saved_lazy_props {
                                self.lazy_props.insert(oid, sp);
                            }
                            self.lazy_init.insert(oid, factory);
                            return Err(e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Materialize a single property of an *uninitialized* lazy object without
    /// running its initializer (PHP 8.4 `skipLazyInitialization` /
    /// `setRawValueWithoutLazyInitialization`): write `value`, drop the property
    /// from the still-lazy set, and — once that set empties — clear the lazy
    /// marker so the object becomes ordinary (no initializer/factory runs; every
    /// property is already set). The caller has validated `prop` is eligible.
    fn lazy_materialize(&mut self, v: &Zval, prop: &[u8], value: Zval) -> Result<(), PhpError> {
        // On an *initialized* proxy the raw write lands on the real instance
        // (clone_creates_object_with_independent_state_003); only an
        // uninitialized wrapper takes the materialize path proper.
        let mut v = v.clone();
        for _ in 0..64 {
            let next = self.proxy_redirect(v.clone());
            let same = match (deref_object(&v), deref_object(&next)) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => true,
            };
            v = next;
            if same {
                break;
            }
        }
        let v = &v;
        let Some(rc) = deref_object(v) else { return Ok(()) };
        let (oid, cid) = {
            let b = rc.borrow();
            (b.id, b.class_id as usize)
        };
        // A raw write still honours readonly write-once: the first raw-set of a
        // readonly slot marks it initialized, a second one errors
        // (setRawValueWithoutLazyInitialization_readonly). A skip materializing
        // `Undef` leaves the slot uninitialized — not an initialization
        // (skipLazyInitialization_readonly).
        if !matches!(value, Zval::Undef) {
        if let Some(decl) =
            prop_readonly_decl(&self.classes, cid, php_types::prop_display_name(prop))
        {
            if rc.borrow().is_readonly_init(prop) {
                return Err(PhpError::Error(format!(
                    "Cannot modify readonly property {}::${}",
                    String::from_utf8_lossy(&self.classes[decl].name),
                    String::from_utf8_lossy(php_types::prop_display_name(prop)),
                )));
            }
            rc.borrow_mut().mark_readonly_init(prop);
        }
        }
        // A raw write still enforces the declared type
        // (lazy_objects/realize: "Cannot assign stdClass to property C::$a of
        // type string"); a skip's `Undef` placeholder is not a write.
        let value = if matches!(value, Zval::Undef) {
            value
        } else {
            self.coerce_typed_prop_write(cid, php_types::prop_display_name(prop), value)?
        };
        // Materialize IN DECLARATION ORDER: the lazy wrapper's table holds only
        // the typed placeholders, so a plain `set` of an untyped property would
        // append it after them (skipLazyInitialization on `$a` must not move it
        // behind `$b`/`$c` in the dump). Rebuild against the class layout.
        let old = {
            let mut b = rc.borrow_mut();
            let mut props = Props::new();
            let mut old = None;
            let mut value = Some(value);
            for (key, _) in &self.classes[cid].prop_defaults {
                if key.as_ref() == prop {
                    old = b.props.get(key).cloned();
                    props.set(key, value.take().expect("materialized value used once"));
                } else if let Some(cur) = b.props.get(key) {
                    props.set(key, cur.clone());
                }
            }
            if let Some(v) = value.take() {
                // Not a declared slot (defensive): plain set, old semantics.
                old = b.props.get(prop).cloned();
                props.set(prop, v);
            }
            b.props = props;
            old
        };
        if let Some(set) = self.lazy_props.get_mut(&oid) {
            set.retain(|n| n.as_ref() != prop);
            if set.is_empty() {
                self.lazy_props.remove(&oid);
                self.lazy_init.remove(&oid);
                rc.borrow_mut().lazy = None;
            }
        }
        // A displaced previous raw value is released *now* (PHP refcount): its
        // destructor may observe — and thereby initialize — the lazy object
        // (`setRawValueWithoutLazyInitialization_side_effect_destruct`).
        if let Some(old) = old {
            self.gc_note(&old);
            drop(old);
            self.gc_sweep_impl(None, true)?;
        }
        Ok(())
    }

    /// If `v` is an *initialized* lazy proxy, the real instance it forwards to;
    /// otherwise `v` itself. Property-access ops call this (after `trigger_lazy`)
    /// so reads/writes land on the real object's slots, while method dispatch and
    /// `get_class` keep operating on the proxy.
    fn proxy_redirect(&self, v: Zval) -> Zval {
        if let Some(rc) = deref_object(&v) {
            let b = rc.borrow();
            if matches!(b.lazy, Some(LazyKind::Proxy)) {
                if let Some(inst) = &b.proxy_instance {
                    return (**inst).clone();
                }
            }
        }
        v
    }

    /// The object a property access on `target` actually lands on: trigger
    /// lazy initialization and follow proxy forwarding **transitively** — a
    /// proxy's real instance may itself have been reset lazy
    /// (`resetAsLazyProxy` on an already-linked instance,
    /// `reset_as_lazy_real_instance`), and the access must re-trigger it.
    /// The bound is a cycle guard only.
    fn lazy_prop_forward(&mut self, target: Zval, name: &[u8]) -> Result<Zval, PhpError> {
        let mut cur = target;
        for _ in 0..64 {
            self.trigger_lazy(&cur, name)?;
            let next = self.proxy_redirect(cur.clone());
            let same = match (deref_object(&cur), deref_object(&next)) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => true,
            };
            cur = next;
            if same {
                break;
            }
        }
        Ok(cur)
    }


    /// Fully initialize a lazy object — regardless of any per-property skip
    /// state — and return the object a *whole-object* operation should read: the
    /// real instance for an initialized proxy, else the object itself. Used by
    /// operations that consume the entire object at once (`(array)` cast,
    /// `get_object_vars`, `json_encode`, `serialize`, `var_export`, `foreach`,
    /// comparison), all of which trigger initialization in PHP 8.4.
    fn realize_full(&mut self, v: &Zval) -> Result<Zval, PhpError> {
        // A proxy's instance may itself have been reset lazy (a proxy chain,
        // `resetAsLazyProxy` on an already-linked instance): keep realizing
        // until the object a whole-object operation reads is a real one. The
        // bound is a cycle guard only.
        let mut cur = v.clone();
        for _ in 0..64 {
            self.realize_lazy(&cur)?;
            let next = self.proxy_redirect(cur.clone());
            let same = match (deref_object(&cur), deref_object(&next)) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => true,
            };
            cur = next;
            if same {
                break;
            }
        }
        Ok(cur)
    }

    /// Whether `v` is (a reference to) a lazy object — uninitialized, or a
    /// forwarding proxy — i.e. a value whole-object consumers must
    /// [`Self::realize_full`] before reading its property table.
    fn is_lazy_value(&self, v: &Zval) -> bool {
        deref_object(v).is_some_and(|o| o.borrow().lazy.is_some())
    }

    /// Writing a *declared* property into a lazy wrapper's sparse table would
    /// append it after the typed placeholders; rebuild in declaration order
    /// first so later dumps match Zend
    /// (init_exception_reverts_initializer_changes). The caller performs the
    /// actual write into the `Null` placeholder this leaves behind.
    fn lazy_ordered_insert(&self, o: &Rc<RefCell<Object>>, key: &[u8]) {
        let cid = {
            let b = o.borrow();
            if b.lazy.is_none() || b.proxy_instance.is_some() || b.props.contains(key) {
                return;
            }
            b.class_id as usize
        };
        if !self.classes[cid].prop_defaults.iter().any(|(k, _)| k.as_ref() == key) {
            return; // dynamic properties append as usual
        }
        let mut b = o.borrow_mut();
        let mut props = Props::new();
        let mut declared: HashSet<&[u8]> = HashSet::default();
        for (k, _) in &self.classes[cid].prop_defaults {
            declared.insert(k.as_ref());
            if k.as_ref() == key {
                props.set(k, Zval::Null);
            } else if let Some(cur) = b.props.get(k) {
                props.set(k, cur.clone());
            }
        }
        for (k, v) in b.props.iter() {
            if !declared.contains(k) {
                props.set(k, v.clone());
            }
        }
        b.props = props;
    }

    /// The property name a dynamic step's key denotes: an object converts via
    /// its `__toString` (typed_properties_001's stringable name); everything
    /// else through the usual string cast.
    fn dyn_prop_name_value(&mut self, k: &Zval) -> Result<Box<[u8]>, PhpError> {
        let kd = k.deref_clone();
        if let Some(o) = deref_object(&kd) {
            let cid = o.borrow().class_id as usize;
            if resolve_method_runtime(&self.classes, cid, b"__toString").is_some() {
                let s = self.call_method_sync(kd.clone(), b"__toString", Vec::new())?;
                return Ok(convert::to_zstr_cast(&s, &mut self.diags).as_bytes().to_vec().into());
            }
        }
        Ok(convert::to_zstr_cast(&kd, &mut self.diags).as_bytes().to_vec().into())
    }

    /// Bind-into-property housekeeping for `BindRefTo(Checked)` on a
    /// single-property path: stringifies a dynamic (possibly object) name in
    /// `keys[0]`, validates the reference's current value against a typed
    /// property (Zend errors at bind time) writing the coerced value through,
    /// and registers the cell as a typed-reference source.
    fn bind_ref_typed_check(
        &mut self,
        target: Option<&Zval>,
        steps: &[FieldStep],
        keys: &mut [Zval],
        cell: &Rc<RefCell<Zval>>,
    ) -> Result<(), PhpError> {
        if steps.len() != 1 {
            return Ok(());
        }
        let name: Box<[u8]> = match &steps[0] {
            FieldStep::Prop(n) => n.clone(),
            FieldStep::PropDyn => {
                let Some(k) = keys.first().cloned() else { return Ok(()) };
                let n = self.dyn_prop_name_value(&k)?;
                keys[0] = Zval::Str(PhpStr::new(n.to_vec()));
                n
            }
            _ => return Ok(()),
        };
        let Some(o) = target.and_then(deref_object) else { return Ok(()) };
        let cid = o.borrow().class_id as usize;
        if let Some((decl, hint)) = prop_type_decl(&self.classes, cid, &name) {
            let cur = cell.borrow().clone();
            let coerced = self.coerce_typed_prop_write(cid, &name, cur)?;
            *cell.borrow_mut() = coerced;
            self.register_typed_ref(cell, &o, decl, &name, hint);
        }
        Ok(())
    }

    /// Follow an *initialized* proxy's forwarding chain WITHOUT initializing:
    /// the view a by-value consumer (a builtin argument, an `(array)` cast)
    /// reads. An uninitialized wrapper passes through as-is.
    fn proxy_view(&self, v: Zval) -> Zval {
        let mut cur = v;
        for _ in 0..64 {
            let next = self.proxy_redirect(cur.clone());
            let same = match (deref_object(&cur), deref_object(&next)) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => true,
            };
            cur = next;
            if same {
                break;
            }
        }
        cur
    }

    /// If a field path's base holds a *lazy* object and the walk starts with a
    /// property step, initialize/forward it (PHP 8.4) and return the object
    /// the walk should root at instead of the raw base slot — the walkers
    /// themselves only see storage (`$proxy->obj->p = v` must reach the real
    /// instance, not scribble on the wrapper's placeholder).
    fn field_lazy_root(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: &[Zval],
        write: bool,
    ) -> Result<Option<Zval>, PhpError> {
        let n: Box<[u8]> = match steps.first() {
            Some(FieldStep::Prop(n)) => n.clone(),
            // A dynamic first step's name is the first popped key (an object
            // name converts via __toString, warning-free).
            Some(FieldStep::PropDyn) => match keys.first() {
                Some(k) => {
                    let k = k.clone();
                    self.dyn_prop_name_value(&k)?
                }
                None => return Ok(None),
            },
            _ => return Ok(None),
        };
        let n = &n;
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
            FieldBase::This => self.frames[top].this.as_ref(),
        };
        let Some(v) = base_val.map(|v| v.deref_clone()) else { return Ok(None) };
        if !self.is_lazy_value(&v) {
            return Ok(None);
        }
        let n = n.clone();
        let cur = self.frames[top].class;
        let fwd = if write {
            self.lazy_prop_access(v, &n, cur, Some(true), (MagicKind::Set, b"__set"))?
        } else {
            self.lazy_prop_access(v, &n, cur, Some(false), (MagicKind::Get, b"__get"))?
        };
        Ok(Some(fwd))
    }

    /// [`Self::lazy_prop_forward`] gated on *how* the access will be served:
    /// when the property dispatches a hook or a magic accessor, the wrapper is
    /// NOT initialized (PHP's `fetch_*_may_not_initialize` family — the hook /
    /// magic body's own accesses trigger instead). `hook_write` selects the
    /// hook side that intercepts this access kind (`None` checks both sides —
    /// compound ops and unset). `magic` is the access kind's magic interceptor
    /// (`__get`/`__set`/`__isset`/`__unset`).
    fn lazy_prop_access(
        &mut self,
        target: Zval,
        name: &[u8],
        scope: Option<ClassId>,
        hook_write: Option<bool>,
        magic: (MagicKind, &'static [u8]),
    ) -> Result<Zval, PhpError> {
        let Some(o) = deref_object(&target) else { return Ok(target) };
        let (uninit, init_proxy, oid, cid) = {
            let b = o.borrow();
            (
                b.lazy.is_some() && b.proxy_instance.is_none(),
                b.lazy.is_some() && b.proxy_instance.is_some(),
                b.id,
                b.class_id as usize,
            )
        };
        if uninit {
            let hooked = !self.hook_guarded(oid, name)
                && match hook_write {
                    Some(w) => self.prop_hook(cid, name, w).is_some(),
                    None => {
                        self.prop_hook(cid, name, false).is_some()
                            || self.prop_hook(cid, name, true).is_some()
                    }
                };
            if hooked || self.magic_applies(&o, name, scope, magic.0, magic.1).is_some() {
                return Ok(target);
            }
        }
        let fwd = self.lazy_prop_forward(target.clone(), name)?;
        // An *initialized* proxy keeps its OWN magic accessors: the handler
        // resolves on the wrapper's class (a subclass override wins,
        // gh21478-proxy-get-override), while property PRESENCE is judged on
        // the real instance the access forwards to. When magic applies, the
        // dispatch happens on the wrapper — return it unforwarded.
        if init_proxy {
            if let Some(io) = deref_object(&fwd) {
                let (present, accessible, inst_oid) = {
                    let b = io.borrow();
                    let inst_cid = b.class_id as usize;
                    let (p, a) = match resolve_prop_access(&self.classes, inst_cid, name, scope) {
                        PropAccess::Slot(k) => (b.props.contains(k.as_slice()), true),
                        PropAccess::Dynamic => (b.props.contains(name), true),
                        PropAccess::Denied { .. } => (b.props.contains(name), false),
                    };
                    (p, a, b.id)
                };
                let guarded = self.magic_guard.contains(&(oid, magic.0, name.to_vec()))
                    || self.magic_guard.contains(&(inst_oid, magic.0, name.to_vec()));
                if !(present && accessible)
                    && !guarded
                    && resolve_method_runtime(&self.classes, cid, magic.1).is_some()
                {
                    return Ok(target);
                }
            }
        }
        // A magic guard held on the wrapper carries over to the object the
        // access forwards to: `$this->$name = v` inside the wrapper's `__set`
        // writes the (possibly freshly initialized) instance raw instead of
        // re-dispatching (gh18038). The transferred key is released by the
        // same frame that releases the wrapper's.
        if let Some(fo) = deref_object(&fwd) {
            if !Rc::ptr_eq(&o, &fo) {
                let old_key = (oid, magic.0, name.to_vec());
                if self.magic_guard.contains(&old_key) {
                    let new_key = (fo.borrow().id, magic.0, name.to_vec());
                    if self.magic_guard.insert(new_key.clone()) {
                        match self
                            .frames
                            .iter_mut()
                            .rev()
                            .find(|f| f.guard_release.iter().any(|k| *k == old_key))
                        {
                            Some(f) => f.guard_release.push(new_key),
                            None => {
                                if let Some(f) = self.frames.last_mut() {
                                    f.guard_release.push(new_key);
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(fwd)
    }

    /// Trigger lazy initialization before an access to property `name` when `v`
    /// is an uninitialized lazy object. Cheap for the common case: a plain
    /// `Option` check, no work for non-lazy objects. A property that has been
    /// individually materialized (skip / raw-set, so no longer in the still-lazy
    /// set) does *not* trigger; an access to any still-lazy or dynamic property
    /// does.
    fn trigger_lazy(&mut self, v: &Zval, name: &[u8]) -> Result<(), PhpError> {
        if let Some(rc) = deref_object(v) {
            let (lazy, oid, cid) = {
                let b = rc.borrow();
                (b.lazy.is_some(), b.id, b.class_id as usize)
            };
            if !lazy {
                return Ok(());
            }
            // A declared eligible property that is no longer in the still-lazy set
            // has been materialized: reading/writing it must not initialize. The
            // set holds *storage* keys (mangled privates); `name` is source-level.
            if let Some(set) = self.lazy_props.get(&oid) {
                let key = self.prop_decl_storage_key(cid, name);
                let still_lazy = set.iter().any(|n| n.as_ref() == key.as_slice());
                if !still_lazy && self.is_lazy_eligible_prop(cid, name) {
                    return Ok(());
                }
            }
            self.realize_lazy(v)?;
        }
        Ok(())
    }

    /// Whether `name` is a property eligible for per-property lazy control on
    /// class `cid`: a declared, non-static, non-virtual instance property (those
    /// are exactly the entries of `prop_defaults`, keyed by *storage* key).
    fn is_lazy_eligible_prop(&self, cid: ClassId, name: &[u8]) -> bool {
        let key = self.prop_decl_storage_key(cid, name);
        self.classes[cid].prop_defaults.iter().any(|(n, _)| n.as_ref() == key.as_slice())
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
        let cc = self.classes[class];
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
            // Enum-case immutability has its own dedicated check (is_enum_case).
            readonly_init: Vec::new(),
            readonly_clone_writable: Vec::new(), typed_unset: Vec::new(),
            lazy: None,
            proxy_instance: None,
        };
        let rc = Rc::new(RefCell::new(obj));
        self.enum_cache.insert((class, case), Rc::clone(&rc));
        rc
    }

    /// `E::cases()` (step 23): a sequential array of every case singleton in
    /// declaration order. Works on pure and backed enums alike. Mirrors
    /// `eval::enum_cases`.
    fn vm_enum_cases(&mut self, cid: ClassId) -> Zval {
        let n = self.classes[cid].enum_cases.len();
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
        let method = if try_from { "tryFrom" } else { "from" };
        let cls = String::from_utf8_lossy(&self.classes[cid].name).into_owned();
        // Port of zend_enum_from_base's ZPP: an int-backed enum declares
        // Z_PARAM_LONG (the full weak long coercion — numeric strings, bools,
        // lossy floats with the deprecation, out-of-range → TypeError "int");
        // a string-backed one Z_PARAM_STR_OR_LONG in weak mode (scalars
        // stringify) and Z_PARAM_STR under a strict_types caller. Diagnostics
        // name the STUB type "string|int" except the int-backed TypeError,
        // which Zend words as "int". Backing type inferred from the first
        // case's value constant.
        let int_backed = matches!(
            self.classes[cid].enum_cases.first().and_then(|c| c.value.as_ref()),
            Some(crate::bytecode::Const::Int(_))
        );
        let strict = self.frames.last().map(|f| f.module.strict).unwrap_or(self.module.strict);
        let type_err = |types: &str, given: &str| {
            PhpError::TypeError(format!(
                "{cls}::{method}(): Argument #1 ($value) must be of type {types}, {given} given"
            ))
        };
        let arg = match arg {
            // A weak-mode null gets the internal-function deprecation, then
            // ZPP's zero value (0 / ""); a strict-mode null falls through to
            // the TypeError below.
            Zval::Null | Zval::Undef if !strict => {
                self.diags.push(Diag::Deprecated(format!(
                    "{cls}::{method}(): Passing null to parameter #1 ($value) of type string|int is deprecated"
                )));
                if int_backed { Zval::Long(0) } else { Zval::Str(PhpStr::new(Vec::new())) }
            }
            other => other,
        };
        let arg = if int_backed {
            let hint = TypeHint { kind: HintKind::Scalar(ScalarType::Int), nullable: false };
            match coerce_to_hint(arg, &hint, &mut self.diags, strict) {
                Ok(v) => v,
                Err(given) => return Err(type_err("int", given)),
            }
        } else if strict {
            match arg {
                Zval::Str(_) => arg,
                other => return Err(type_err("string", &other.type_name_for_error())),
            }
        } else {
            match arg {
                Zval::Str(_) => arg,
                Zval::Long(l) => Zval::Str(PhpStr::new(l.to_string().into_bytes())),
                Zval::Double(_) | Zval::Bool(_) => {
                    Zval::Str(convert::to_zstr(&arg, &mut self.diags))
                }
                // Z_PARAM_STR weak accepts a __toString object.
                Zval::Object(_) => match self.object_to_string_weak(&arg) {
                    Some(s) => Zval::Str(s),
                    None => return Err(type_err("string|int", &arg.type_name_for_error())),
                },
                other => return Err(type_err("string|int", &other.type_name_for_error())),
            }
        };
        let n = self.classes[cid].enum_cases.len();
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
            String::from_utf8_lossy(&self.classes[cid].name)
        )))
    }

    /// Expand a spread source into its `(key, value)` pairs for a call's argument
    /// unpacking (PAR): an array yields its entries verbatim; a generator is driven
    /// to completion, honouring its yielded keys (so string keys become named
    /// arguments). Any non-iterable is the PHP `TypeError`. Mirrors the
    /// array-merge logic of `Op::ArrayAppendSpread`.
    fn spread_pairs(&mut self, src: Zval) -> Result<Vec<(Key, Zval)>, PhpError> {
        match src.deref_clone() {
            // A reference element stays a live reference (Zend passes it by-ref
            // when the parameter is by-ref; frame binding decays it at a
            // by-value position) — `f(...[&$out, 'x'])` writes through.
            Zval::Array(s) => Ok(s
                .iter()
                .map(|(k, v)| {
                    let v = match v {
                        Zval::Ref(rc) => Zval::Ref(Rc::clone(rc)),
                        other => other.clone(),
                    };
                    (k.clone(), v)
                })
                .collect()),
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
            obj @ Zval::Object(_)
                if object_class_id(&obj).is_some_and(|c| self.is_traversable(c)) =>
            {
                self.collect_traversable(obj)
            }
            other => Err(PhpError::TypeError(format!(
                "Only arrays and Traversables can be unpacked, {} given",
                other.type_name_for_error()
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
        if !is_instance_of(&self.classes, self.stringable_id, cid, throwable_id) {
            return;
        }
        let top = self.frames.len() - 1;
        let line = self.cur_line(top);
        // The throwable's file is the defining file of the function where it was
        // constructed (`new` site), not the entry module — so `getFile()` and the
        // uncaught "in F:L" report the right file across includes.
        let file = self.frame_file(top).to_vec();
        let (trace, trace_string) = self.capture_trace();
        let mut b = o.borrow_mut();
        b.props.set(b"line", Zval::Long(line as i64));
        b.props.set(b"file", Zval::Str(PhpStr::new(file)));
        b.props.set(self.host_prop_key(cid, b"trace").as_slice(), trace);
        b.props
            .set(self.host_prop_key(cid, b"traceString").as_slice(), Zval::Str(PhpStr::new(trace_string)));
    }

    /// The source file attributed to frame `i`: its function's defining file
    /// (carried from the `FnDecl`), falling back to the entry module's file for a
    /// synthetic stub / the script body that recorded no file.
    fn frame_file(&self, i: usize) -> &[u8] {
        let f = &self.frames[i].func.file;
        if f.is_empty() {
            &self.module.file
        } else {
            f
        }
    }

    /// Snapshot the running frame stack as a Throwable's `(trace array, trace
    /// string)` (EXC-3c), mirroring `eval::capture_trace`. Frames are
    /// innermost-first, excluding the script body (`main`), and the string ends
    /// with `#N {main}`. Each entry's `line` is the call-site line in the
    /// *caller* (frame `k` was entered from frame `k-1`), recovered from the
    /// per-op line table (EXC-3b); `args` is empty, as the tree-walker leaves it.
    fn capture_trace(&self) -> (Zval, Vec<u8>) {
        let mut arr = PhpArray::new();
        let mut s: Vec<u8> = Vec::new();
        let n = self.frames.len();
        for (i, k) in (1..n).rev().enumerate() {
            let frame = &self.frames[k];
            let line = self.cur_line(k - 1) as i64;
            // The file/line are the *call site* in the caller (frame `k-1`): the
            // file is the caller function's defining file, so a callee invoked
            // across an include/autoload boundary still attributes to the right
            // file (the frame's module is the caller's, not the callee's). An empty
            // file (synthetic stub) falls back to the entry module.
            let caller_file = &self.frames[k - 1].func.file;
            let file: &[u8] = if caller_file.is_empty() {
                &self.module.file
            } else {
                caller_file
            };
            // The class shown is the late-static-binding (called) class, like the
            // tree-walker; absent for a free-function frame. The id is resolved in
            // the frame's own module (an eval'd/included frame may differ from the
            // currently executing `self.module`).
            let class: Option<&[u8]> = frame
                .static_class
                .map(|cid| self.classes[cid].name.as_ref());
            let is_static = frame.this.is_none();

            s.extend_from_slice(format!("#{i} ").as_bytes());
            s.extend_from_slice(file);
            s.extend_from_slice(format!("({line}): ").as_bytes());
            if let Some(c) = class {
                s.extend_from_slice(c);
                s.extend_from_slice(if is_static { b"::" } else { b"->" });
            }
            // PHP renders each frame's call arguments inline (`f(42, 'x')`),
            // scalars literal and strings quoted/truncated (`format_bt_arg`).
            let frame_args = self.current_frame_args(k);
            s.extend_from_slice(&frame.func.name);
            s.push(b'(');
            let joined =
                frame_args.iter().map(format_bt_arg).collect::<Vec<_>>().join(", ");
            s.extend_from_slice(joined.as_bytes());
            s.extend_from_slice(b")\n");

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
            let mut argsarr = PhpArray::new();
            for a in frame_args {
                let _ = argsarr.append(a);
            }
            fr.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(argsarr)));
            let _ = arr.append(Zval::Array(Rc::new(fr)));
        }
        s.extend_from_slice(format!("#{} {{main}}", n - 1).as_bytes());
        (Zval::Array(Rc::new(arr)), s)
    }

    /// Resolve a [`ClassTarget`] to the concrete class id in frame `top`'s
    /// context: `Static` reads the LSB class, `SelfScope`/`ParentScope` the
    /// closure frame's (rebindable) scope class, each with PHP's faithful
    /// error when the context is missing.
    pub(super) fn target_class_id(
        &self,
        target: ClassTarget,
        top: usize,
    ) -> Result<ClassId, PhpError> {
        match target {
            ClassTarget::Class(cid) => Ok(cid),
            ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                PhpError::Error("Cannot use \"static\" in the global scope".to_string())
            }),
            ClassTarget::SelfScope => self.frames[top].class.ok_or_else(|| {
                PhpError::Error("Cannot access \"self\" when no class scope is active".to_string())
            }),
            ClassTarget::ParentScope => {
                let c = self.frames[top].class.ok_or_else(|| {
                    PhpError::Error(
                        "Cannot access \"parent\" when no class scope is active".to_string(),
                    )
                })?;
                self.classes[c].parent.ok_or_else(|| {
                    PhpError::Error(
                        "Cannot access \"parent\" when current class scope has no parent"
                            .to_string(),
                    )
                })
            }
        }
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
        let start = self.target_class_id(target, top)?;
        let Some((decl, idx)) = find_static_prop(&self.classes, start, name) else {
            return Err(PhpError::Error(format!(
                "Access to undeclared static property {}::${}",
                String::from_utf8_lossy(&self.classes[start].name),
                String::from_utf8_lossy(name)
            )));
        };
        // Detach the declaring class as `&'m` so `entry`/the thunk `func` don't
        // borrow `self` across the later `&mut self` (`static_props`/`frames`) uses.
        let decl_cc = self.classes[decl];
        let entry = &decl_cc.static_props[idx];
        if !visible_from(&self.classes, self.frames[top].class, entry.visibility, decl) {
            return Err(prop_access_error(&self.classes, decl, name, entry.visibility));
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
                let mut frame = Frame::new(func, self.class_mod(decl));
                frame.class = Some(decl);
                frame.static_class = Some(decl);
                frame.ret_cell = Some(Rc::clone(&cell));
                self.frames[top].ip = ip;
                self.frames.push(frame);
                Ok(None)
            }
        }
    }

    /// Pop the operand-stack keys for a field path's `Index` / `PropDyn` steps
    /// (one value per such step), restoring source order.
    fn pop_field_keys(&mut self, top: usize, steps: &[FieldStep]) -> Vec<Zval> {
        let n = steps
            .iter()
            .filter(|s| matches!(s, FieldStep::Index | FieldStep::PropDyn))
            .count();
        let mut keys: Vec<Zval> =
            (0..n).map(|_| self.frames[top].stack.pop().expect("field index key")).collect();
        keys.reverse();
        keys
    }

    fn field_value(&self, base: FieldBase, top: usize, steps: &[FieldStep], keys: Vec<Zval>) -> Option<Zval> {
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        let cell = match base {
            FieldBase::Local(s) => &self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &self.frames[0].slots[s as usize],
            FieldBase::Superglobal(i) => &self.superglobals[i as usize],
            FieldBase::This => self.frames[top].this.as_ref()?,
        };
        field_get(cell, steps, &mut keys.into_iter(), fs)
    }

    /// The BP_VAR_IS walk behind `Op::FieldIsset`/`Op::FieldEmpty` with the
    /// `__isset`/`__get` protocol dispatched at ANY property step — not just
    /// the path start (`isset($block->block_type->uses_context)`: the magic
    /// leaf sits one hop in; WP_Block_Type keys its private props on it).
    /// Walks the plain prefix silently, runs the protocol at the first magic
    /// boundary, finishes the rest through the plain walker. `Ok(None)` = no
    /// magic boundary met — the caller keeps its plain machinery (ArrayAccess
    /// leaves, lazy roots, the raw field walk). Oracle-pinned protocol:
    /// - intermediate step: `__isset` gates (false → unset); the value then
    ///   comes from `__get` (also without `__isset` — the dim-fetch rule);
    /// - terminal `isset()`: `__isset` result; only-`__get` answers false
    ///   WITHOUT calling it;
    /// - terminal `empty()`: `__isset` false → true; true → `!truthy(__get)`;
    ///   only-`__get` answers true WITHOUT calling it.
    fn field_magic_probe(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: &[Zval],
        empty_mode: bool,
    ) -> Result<Option<bool>, PhpError> {
        let cur = self.frames[top].class;
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
            FieldBase::This => self.frames[top].this.as_ref(),
        };
        let Some(mut v) = base_val.map(|x| x.deref_clone()) else { return Ok(None) };
        let mut i = 0usize;
        let mut kpos = 0usize;
        while i < steps.len() {
            // Follow *initialized* proxy forwarding (no initialization).
            for _ in 0..64 {
                let next = self.proxy_redirect(v.clone());
                let same = match (deref_object(&v), deref_object(&next)) {
                    (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                    _ => true,
                };
                v = next;
                if same {
                    break;
                }
            }
            let step = &steps[i];
            let (name, step_keys): (Option<Vec<u8>>, usize) = match step {
                FieldStep::Prop(n) => (Some(n.to_vec()), 0),
                FieldStep::PropDyn => {
                    let Some(k) = keys.get(kpos) else { return Ok(None) };
                    (
                        Some(convert::to_zstr_cast(k, &mut self.diags).as_bytes().to_vec()),
                        1,
                    )
                }
                FieldStep::Index => (None, 1),
                FieldStep::Append => (None, 0),
            };
            if let (Some(name), Some(o)) = (&name, deref_object(&v)) {
                let has_isset =
                    self.magic_applies(&o, name, cur, MagicKind::Isset, b"__isset").is_some();
                let has_get =
                    self.magic_applies(&o, name, cur, MagicKind::Get, b"__get").is_some();
                if has_isset || has_get {
                    let rest = &steps[i + 1..];
                    let rest_keys: Vec<Zval> = keys[kpos + step_keys..].to_vec();
                    let oid = o.borrow().id;
                    let name_z = Zval::Str(PhpStr::new(name.clone()));
                    let isset_res = if has_isset {
                        let gkey = (oid, MagicKind::Isset, name.clone());
                        let ins = self.magic_guard.insert(gkey.clone());
                        let r = self.call_method_sync(v.clone(), b"__isset", vec![name_z.clone()]);
                        if ins {
                            self.magic_guard.remove(&gkey);
                        }
                        Some(convert::to_bool(&r?.deref_clone(), &mut self.diags))
                    } else {
                        None
                    };
                    fn fetch_get(
                        vm: &mut Vm,
                        oid: u32,
                        v: &Zval,
                        name: &[u8],
                        name_z: &Zval,
                    ) -> Result<Zval, PhpError> {
                        let gkey = (oid, MagicKind::Get, name.to_vec());
                        let ins = vm.magic_guard.insert(gkey.clone());
                        let r = vm.call_method_sync(v.clone(), b"__get", vec![name_z.clone()]);
                        if ins {
                            vm.magic_guard.remove(&gkey);
                        }
                        Ok(r?.deref_clone())
                    }
                    let result = if rest.is_empty() {
                        match (isset_res, empty_mode) {
                            (Some(false), false) => false,
                            (Some(false), true) => true,
                            (Some(true), false) => true,
                            (Some(true), true) => {
                                if has_get {
                                    !convert::is_true_silent(&fetch_get(
                                        self, oid, &v, name, &name_z,
                                    )?)
                                } else {
                                    false
                                }
                            }
                            (None, false) => false,
                            (None, true) => true,
                        }
                    } else if !isset_res.unwrap_or(true) || !has_get {
                        // Unset (or unfetchable) intermediate: the whole path
                        // reads as missing.
                        empty_mode
                    } else {
                        let gv = fetch_get(self, oid, &v, name, &name_z)?;
                        let fs = FieldScope { classes: &self.classes, scope: cur };
                        let leaf = field_get(&gv, rest, &mut rest_keys.into_iter(), fs);
                        match (leaf, empty_mode) {
                            (Some(x), false) => !matches!(x, Zval::Null | Zval::Undef),
                            (Some(x), true) => !convert::is_true_silent(&x),
                            (None, false) => false,
                            (None, true) => true,
                        }
                    };
                    return Ok(Some(result));
                }
            }
            // Plain step: advance one hop through the silent walker.
            let fs = FieldScope { classes: &self.classes, scope: cur };
            let step_key_vec: Vec<Zval> = keys[kpos..kpos + step_keys].to_vec();
            let next = field_get(&v, std::slice::from_ref(step), &mut step_key_vec.into_iter(), fs);
            match next {
                Some(nv) => v = nv,
                None => return Ok(None),
            }
            kpos += step_keys;
            i += 1;
        }
        Ok(None)
    }

    /// For the fused field ops (isset/empty/unset): when the path's LAST step
    /// is an Index whose prefix resolves to an ArrayAccess OBJECT, the op must
    /// dispatch the protocol instead of array-walking. Returns the receiver
    /// (shared Rc — protocol mutations hit the real instance) and the final
    /// key; `None` keeps the plain walker path.
    fn field_aa_leaf(
        &self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: &[Zval],
    ) -> Option<(Zval, Zval)> {
        let (last, prefix) = steps.split_last()?;
        if !matches!(last, FieldStep::Index) {
            return None;
        }
        // Conservative: only a leaf `Index` whose immediately-enclosing step is
        // a *declared, accessible* property (or `$this`/local base) dispatches
        // the protocol. A magic/undeclared property in the prefix keeps the
        // plain walker so `__get`/`__set` semantics are not skipped (bug40833:
        // `unset($obj->magicProp[0])`). `$this->collection[k]` — doctrine's
        // AbstractLazyCollection — has a declared `collection`, so it qualifies.
        let (last_prop, prefix_head) = match prefix.split_last() {
            Some((FieldStep::Prop(n), head)) => (n.as_ref(), head),
            // No property in the prefix (`$local[k]`): the base is a plain
            // variable, safe to dispatch.
            None => {
                let recv = self.base_field_cell(base, top)?.deref_clone();
                let key = keys.first()?.clone();
                return (matches!(recv, Zval::Object(_))
                    && self.object_implements(&recv, b"arrayaccess"))
                .then_some((recv, key));
            }
            _ => return None,
        };
        // The container the last property lives on (the prefix minus that prop).
        let head_consumed = prefix_head
            .iter()
            .filter(|s| matches!(s, FieldStep::Index | FieldStep::PropDyn))
            .count();
        let container = self.field_value(base, top, prefix_head, keys[..head_consumed].to_vec())?;
        let ccid = object_class_id(&container)?;
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        if !fs.prop_is_declared_slot(ccid, last_prop) {
            return None;
        }
        let consumed = prefix
            .iter()
            .filter(|s| matches!(s, FieldStep::Index | FieldStep::PropDyn))
            .count();
        let key = keys.get(consumed)?.clone();
        let recv = self.field_value(base, top, prefix, keys[..consumed].to_vec())?;
        (matches!(recv, Zval::Object(_)) && self.object_implements(&recv, b"arrayaccess"))
            .then_some((recv, key))
    }

    /// The base cell of a field path as a value (no navigation), for the
    /// prefix-less `field_aa_leaf` case.
    fn base_field_cell(&self, base: FieldBase, top: usize) -> Option<&Zval> {
        Some(match base {
            FieldBase::Local(s) => &self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &self.frames[0].slots[s as usize],
            FieldBase::Superglobal(i) => &self.superglobals[i as usize],
            FieldBase::This => self.frames[top].this.as_ref()?,
        })
    }

    /// Resolve a MULTI-key isset/empty dim path with Zend's BP_VAR_IS quiet
    /// fetch: raw containers walk silently, but an intermediate ArrayAccess
    /// object dispatches `offsetExists` (false short-circuits to `Missing`)
    /// then `offsetGet` — symfony VarDumper's `Data` nests exactly this way
    /// (`isset($data['a']['b'])`). The leaf is handed back for protocol
    /// dispatch when it lands on an ArrayAccess object, else pre-read raw.
    fn dim_is_walk(
        &mut self,
        base: DimBase,
        top: usize,
        keys: &[Zval],
    ) -> Result<DimIsLeaf, PhpError> {
        let mut cur = self.base_cell(base, top).deref_clone();
        // A bare `isset($x)`/`empty($x)` compiles with no keys: the base value
        // itself is the leaf.
        let Some((last, prefix)) = keys.split_last() else {
            return Ok(DimIsLeaf::Raw(Some(cur)));
        };
        for k in prefix {
            if matches!(cur, Zval::Object(_)) && self.object_implements(&cur, b"arrayaccess") {
                let ex = self.call_method_sync(cur.clone(), b"offsetExists", vec![k.clone()])?;
                if !convert::is_true_silent(&ex.deref_clone()) {
                    return Ok(DimIsLeaf::Missing);
                }
                cur = self.call_method_sync(cur, b"offsetGet", vec![k.clone()])?.deref_clone();
            } else {
                match silent_get_path(&cur, std::slice::from_ref(k)) {
                    Some(v) => cur = v,
                    None => return Ok(DimIsLeaf::Missing),
                }
            }
        }
        if matches!(cur, Zval::Object(_)) && self.object_implements(&cur, b"arrayaccess") {
            return Ok(DimIsLeaf::Aa(cur, last.clone()));
        }
        Ok(DimIsLeaf::Raw(silent_get_path(&cur, std::slice::from_ref(last))))
    }

    /// The fused-field analogue of [`Self::dim_is_walk`] for isset/empty:
    /// when a *declared* property (or the bare local base) holds an
    /// ArrayAccess object with TWO OR MORE trailing `Index` steps
    /// (`isset($this->data['a']['b'])`, symfony VarDumper `Data`), the
    /// intermediate indexes dispatch `offsetExists`/`offsetGet` (Zend's
    /// BP_VAR_IS quiet fetch) and the leaf is handed back for protocol
    /// dispatch. `None` keeps the existing raw/lazy paths (single-Index
    /// leaves stay with [`Self::field_aa_leaf`]).
    fn field_aa_walk(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: &[Zval],
    ) -> Result<Option<DimIsLeaf>, PhpError> {
        let m = steps.iter().rev().take_while(|s| matches!(s, FieldStep::Index)).count();
        if m < 2 || keys.len() < m {
            return Ok(None);
        }
        let prefix = &steps[..steps.len() - m];
        let split = keys.len() - m;
        // Same conservative gate as `field_aa_leaf`: the step owning the
        // trailing run must be a declared, accessible property (or the bare
        // local base) so magic `__get` prefixes keep the plain walker.
        let container = match prefix.split_last() {
            Some((FieldStep::Prop(n), head)) => {
                let head_consumed = head
                    .iter()
                    .filter(|s| matches!(s, FieldStep::Index | FieldStep::PropDyn))
                    .count();
                let Some(cont) = self.field_value(base, top, head, keys[..head_consumed].to_vec())
                else {
                    return Ok(None);
                };
                let Some(ccid) = object_class_id(&cont) else { return Ok(None) };
                let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
                if !fs.prop_is_declared_slot(ccid, n) {
                    return Ok(None);
                }
                match self.field_value(base, top, prefix, keys[..split].to_vec()) {
                    Some(v) => v,
                    None => return Ok(None),
                }
            }
            None => match self.base_field_cell(base, top) {
                Some(c) => c.deref_clone(),
                None => return Ok(None),
            },
            _ => return Ok(None),
        };
        if !(matches!(container, Zval::Object(_))
            && self.object_implements(&container, b"arrayaccess"))
        {
            return Ok(None);
        }
        let mut cur = container;
        let (last, mids) = keys[split..].split_last().expect("m >= 2");
        for k in mids {
            if matches!(cur, Zval::Object(_)) && self.object_implements(&cur, b"arrayaccess") {
                let ex = self.call_method_sync(cur.clone(), b"offsetExists", vec![k.clone()])?;
                if !convert::is_true_silent(&ex.deref_clone()) {
                    return Ok(Some(DimIsLeaf::Missing));
                }
                cur = self.call_method_sync(cur, b"offsetGet", vec![k.clone()])?.deref_clone();
            } else {
                match silent_get_path(&cur, std::slice::from_ref(k)) {
                    Some(v) => cur = v,
                    None => return Ok(Some(DimIsLeaf::Missing)),
                }
            }
        }
        if matches!(cur, Zval::Object(_)) && self.object_implements(&cur, b"arrayaccess") {
            return Ok(Some(DimIsLeaf::Aa(cur, last.clone())));
        }
        Ok(Some(DimIsLeaf::Raw(silent_get_path(&cur, std::slice::from_ref(last)))))
    }

    /// The dim-path analogue of [`Self::field_aa_leaf`]: for a MULTI-key
    /// unset whose prefix resolves RAW to an ArrayAccess object, return
    /// (receiver, final key) for protocol dispatch.
    fn dim_aa_leaf(&self, base: DimBase, top: usize, keys: &[Zval]) -> Option<(Zval, Zval)> {
        if keys.len() < 2 {
            return None;
        }
        let (last, prefix) = keys.split_last()?;
        let recv = silent_get_path(self.base_cell(base, top), prefix)?;
        if matches!(recv, Zval::Object(_)) && self.object_implements(&recv, b"arrayaccess") {
            Some((recv, last.clone()))
        } else {
            None
        }
    }

    fn field_remove(&mut self, base: FieldBase, top: usize, steps: &[FieldStep], keys: Vec<Zval>) {
        let fs = FieldScope { classes: &self.classes, scope: self.frames[top].class };
        let cell = match base {
            FieldBase::Local(s) => &mut self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &mut self.frames[0].slots[s as usize],
            FieldBase::Superglobal(i) => &mut self.superglobals[i as usize],
            FieldBase::This => match self.frames[top].this.as_mut() {
                Some(c) => c,
                None => return,
            },
        };
        field_unset(cell, steps, &mut keys.into_iter(), fs);
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
        let callee = &self.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee, self.class_mod(defc));
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
        let callee = &self.classes[defc].methods[midx].func;
        let mut frame = Frame::new(callee, self.class_mod(defc));
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
        frame.guard_release.push(key);
        self.frames.push(frame);
    }

    /// The compiled `get`/`set` hook of property `name` on class `cid`, if any
    /// (step 50). Read from the unified, parent-flattened `prop_info` table, so the
    /// most-derived hook is already in `cid`'s entry. The returned ref lives as long
    /// as the module.
    fn prop_hook(&self, cid: usize, name: &[u8], set: bool) -> Option<&'m Func> {
        let h = self.classes[cid].prop_info.get(name)?.hooks.as_ref()?;
        if set {
            h.set.as_ref()
        } else {
            h.get.as_ref()
        }
    }

    /// Whether a property hook for `(oid, name)` is currently on the stack, so an
    /// access to that property must reach the backing store directly (step 50).
    fn hook_guarded(&self, oid: u32, name: &[u8]) -> bool {
        self.magic_guard.contains(&(oid, MagicKind::Hook, name.to_vec()))
    }

    /// Whether `name` on `cid` is a *virtual* hooked property — it has hooks but
    /// no backing slot. Such a property is read-only when it has no `set` hook
    /// (a direct write throws "is read-only") and write-only when it has no `get`
    /// hook (a direct read throws "is write-only"), PHP 8.4. A *backed* hooked
    /// property instead reads/writes its backing store when a hook is absent.
    fn is_virtual_hooked(&self, cid: usize, name: &[u8]) -> bool {
        self.classes[cid]
            .prop_info
            .get(name)
            .and_then(|pi| pi.hooks.as_ref())
            .map_or(false, |h| !h.backed)
    }

    /// Dispatch a property `get`/`set` hook as a frame, mirroring
    /// [`Self::push_magic_prop`]. `set_value` is `Some` for a `set` hook (bound to
    /// slot 0; its return discarded into a throwaway cell) and `None` for a `get`
    /// hook (its return flows to the caller as the read result). The hook guard
    /// `(oid, Hook, name)` is released on `Ret`, so `$this->name` inside the hook
    /// reaches the backing store.
    fn push_hook(&mut self, func: &'m Func, recv: Zval, oid: u32, name: &[u8], set_value: Option<Zval>) {
        let lsb = object_class_id(&recv).unwrap_or(0);
        // A hook runs in the scope of the class that *declared* it (so it can reach
        // that class's private/protected members even when invoked on a subclass
        // instance), while late static binding stays the object's runtime class —
        // mirroring `push_magic_prop`. The hook body's bytecode is relative to the
        // declaring class's module.
        let decl = prop_info(&self.classes, lsb, name).map(|pi| pi.declaring_class).unwrap_or(lsb);
        let is_set = set_value.is_some();
        let mut frame = Frame::new(func, self.class_mod(decl));
        frame.argc = func.n_params;
        if let Some(v) = set_value {
            if !frame.slots.is_empty() {
                frame.slots[0] = v;
            }
        }
        frame.this = Some(recv);
        frame.class = Some(decl);
        frame.static_class = Some(lsb);
        if is_set {
            // A `set` hook's own return value is discarded (like `__set`).
            frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
        }
        // A `&get` hook returns a `Zval::Ref`; this implicit dispatch serves a
        // *value* context (a plain property read), so the caller needs the
        // dereferenced value, not the cell. Place contexts (`&$o->prop`,
        // `$o->prop[] = v`) run the hook via `byref_hook_root`, which clears
        // the flag to keep the cell.
        frame.ret_deref = func.by_ref && !is_set;
        let key = (oid, MagicKind::Hook, name.to_vec());
        // Only the frame that first guards `(oid, name)` releases it on `Ret`. A
        // nested explicit hook call re-entering the same property (a parent hook
        // calling its own parent) must not release the outer hook's guard early.
        if self.magic_guard.insert(key.clone()) {
            frame.guard_release.push(key);
        }
        self.frames.push(frame);
    }

    /// Dispatch an *explicit* parent/self property-hook call
    /// (`parent::$name::get()` / `self::$name::set($v)`, PHP 8.4). Unlike
    /// [`Self::push_hook`], the hook's defining class is taken from `start` — the
    /// statically-resolved class the call names — rather than the receiver's
    /// runtime class, so an overridden hook still runs in its own scope. Late
    /// static binding stays the receiver's class. A `set` hook's body return is
    /// discarded (the caller pushes the call result itself).
    fn push_parent_hook(
        &mut self,
        func: &'m Func,
        recv: Zval,
        oid: u32,
        name: &[u8],
        start: usize,
        set_value: Option<Zval>,
    ) {
        let lsb = object_class_id(&recv).unwrap_or(start);
        let decl = prop_info(&self.classes, start, name)
            .map(|pi| pi.declaring_class)
            .unwrap_or(start);
        let is_set = set_value.is_some();
        let mut frame = Frame::new(func, self.class_mod(decl));
        frame.argc = func.n_params;
        if let Some(v) = set_value {
            if !frame.slots.is_empty() {
                frame.slots[0] = v;
            }
        }
        frame.this = Some(recv);
        frame.class = Some(decl);
        frame.static_class = Some(lsb);
        if is_set {
            frame.ret_cell = Some(Rc::new(RefCell::new(Zval::Null)));
        }
        // Explicit `parent::$name::get()` is a value context: deref a `&get`
        // hook's returned cell (mirrors `push_hook`).
        frame.ret_deref = func.by_ref && !is_set;
        let key = (oid, MagicKind::Hook, name.to_vec());
        if self.magic_guard.insert(key.clone()) {
            frame.guard_release.push(key);
        }
        self.frames.push(frame);
    }

    /// If a mixed write/ref path starts by navigating *into* a property that has
    /// a get/set hook, PHP rejects the indirect modification — a hooked
    /// property's storage is not directly addressable (`zend_std_read_property`
    /// with a write fetch). Returns the `(class name, prop name)` to name in the
    /// error, or `None` when the path is allowed: the property is not hooked, or
    /// its current value is an object (object handles stay mutable). A `&get`
    /// by-reference hook also allows it — those paths are intercepted upstream
    /// by [`Self::byref_hook_root`] before this check runs.
    fn indirect_hook_target(
        &self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
    ) -> Option<(Vec<u8>, Vec<u8>)> {
        let FieldStep::Prop(name) = steps.first()? else { return None };
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize)?,
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize)?,
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize)?,
            FieldBase::This => self.frames[top].this.as_ref()?,
        };
        let o = deref_object(base_val)?;
        let cid = o.borrow().class_id as usize;
        if self.prop_hook(cid, name, false).is_none() && self.prop_hook(cid, name, true).is_none() {
            return None;
        }
        // Inside the property's own hook the access reaches the backing store
        // directly (the hook guard is active), so it is not an indirect access.
        if self.hook_guarded(o.borrow().id, name) {
            return None;
        }
        // An object value remains modifiable through its handle (Zend allows it).
        let key = self.prop_storage_key(cid, name, self.frames[top].class);
        if matches!(o.borrow().props.get(&key).map(|v| v.deref_clone()), Some(Zval::Object(_))) {
            return None;
        }
        Some((self.classes[cid].name.to_vec(), name.to_vec()))
    }

    /// Reject a write/ref path that indirectly modifies a hooked property
    /// (`$o->hookedProp[...] = ...`, `$ref =& $o->hookedProp`). No-op when the
    /// path is allowed (see [`Self::indirect_hook_target`]).
    fn reject_indirect_hook(
        &self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
    ) -> Result<(), PhpError> {
        if let Some((cls, prop)) = self.indirect_hook_target(base, top, steps) {
            return Err(PhpError::Error(format!(
                "Indirect modification of {}::${} is not allowed",
                String::from_utf8_lossy(&cls),
                String::from_utf8_lossy(&prop),
            )));
        }
        Ok(())
    }

    /// Whether a field path's first step writes *into* a hooked property —
    /// binding a reference to it (`$o->hookedProp =& $r`) is rejected with
    /// "Cannot assign by reference to overloaded object" regardless of value.
    fn field_starts_at_hook(&self, base: FieldBase, top: usize, steps: &[FieldStep]) -> bool {
        let Some(FieldStep::Prop(name)) = steps.first() else { return false };
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
            FieldBase::This => self.frames[top].this.as_ref(),
        };
        let Some(o) = base_val.and_then(deref_object) else { return false };
        // Inside the property's own hook the backing store is addressed directly.
        if self.hook_guarded(o.borrow().id, name) {
            return false;
        }
        let cid = o.borrow().class_id as usize;
        self.prop_hook(cid, name, false).is_some() || self.prop_hook(cid, name, true).is_some()
    }

    /// Record `cell` as aliasing typed property `decl::$prop`, so writes
    /// through the reference keep enforcing `hint` (Zend's typed-reference
    /// sources, narrowed to the cells phpr hands out). Dead entries are pruned
    /// here, keeping the table proportional to the *live* typed aliases.
    fn register_typed_ref(
        &mut self,
        cell: &Rc<RefCell<Zval>>,
        owner: &Rc<RefCell<Object>>,
        decl: ClassId,
        prop: &[u8],
        hint: TypeHint,
    ) {
        self.typed_refs.retain(|t| t.cell.strong_count() > 0);
        let ptr = Rc::as_ptr(cell);
        if self.typed_refs.iter().any(|t| std::ptr::eq(t.cell.as_ptr(), ptr)) {
            return;
        }
        self.typed_refs.push(TypedRefSource {
            cell: Rc::downgrade(cell),
            obj: Rc::downgrade(owner),
            class_name: self.classes[decl].name.clone(),
            prop: prop.into(),
            hint,
        });
    }

    /// If `base->name` is a *typed* declared property, register `cell` (just
    /// handed out as a reference to it) as a typed-reference source.
    fn register_prop_typed_ref(&mut self, base: FieldBase, top: usize, name: &[u8], cell: &Rc<RefCell<Zval>>) {
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize).map(|v| v.deref_clone()),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize).map(|v| v.deref_clone()),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize).map(|v| v.deref_clone()),
            FieldBase::This => self.frames[top].this.as_ref().map(|v| v.deref_clone()),
        };
        // The owner is the object the access actually lands on: an
        // initialized proxy's INSTANCE (unset purges by that identity).
        let Some(o) = base_val.map(|v| self.proxy_view(v)).as_ref().and_then(deref_object) else { return };
        let cid = o.borrow().class_id as usize;
        if let Some((decl, hint)) = prop_type_decl(&self.classes, cid, name) {
            self.register_typed_ref(cell, &o, decl, name, hint);
        }
    }

    /// Coerce (weak mode) / check (strict) `v` for a write through `cell` when
    /// the cell is a registered typed-property reference; pass-through
    /// otherwise. A mismatch is Zend's typed-ref TypeError ("Cannot assign
    /// string to reference held by property C::$p of type int").
    fn typed_ref_assign(&mut self, cell: &Rc<RefCell<Zval>>, v: Zval, strict: bool) -> Result<Zval, PhpError> {
        let ptr = Rc::as_ptr(cell);
        // The owning object must still be alive beyond the VM's own `created`
        // tracking handle: Zend deletes the type source at object free, and
        // an object whose only remaining handle is the tracking one is
        // dead-pending-sweep (`typed_properties_094`). A cell aliased by
        // SEVERAL typed properties (`$o->b =& $o->a`) must satisfy every
        // source (typed_properties_002).
        let sources: Vec<(TypeHint, Box<[u8]>, Box<[u8]>)> = self
            .typed_refs
            .iter()
            .filter(|t| {
                t.cell.strong_count() > 0
                    && std::ptr::eq(t.cell.as_ptr(), ptr)
                    && t.obj.strong_count() > 1
            })
            .map(|t| (t.hint.clone(), t.class_name.clone(), t.prop.clone()))
            .collect();
        let mut v = v;
        for (hint, cls, prop) in sources {
            v = self.coerce_or_check_hint(v, &hint, strict).map_err(|given| {
                PhpError::TypeError(format!(
                    "Cannot assign {given} to reference held by property {}::${} of type {}",
                    String::from_utf8_lossy(&cls),
                    String::from_utf8_lossy(&prop),
                    hint.display_name(),
                ))
            })?;
        }
        Ok(v)
    }

    /// `$r = &$o->prop` on a property whose asymmetric *set* visibility
    /// (`public private(set)`, PHP 8.4) excludes the current scope binds a
    /// reference to a **copy** — the storage must not become writable through
    /// the alias (`Zend/tests/asymmetric_visibility/object_reference.phpt`).
    /// Returns the detached cell, or `None` when the path is not a single
    /// property step or the scope may write the property.
    fn asym_set_ref_copy(
        &self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
    ) -> Option<Rc<RefCell<Zval>>> {
        let [FieldStep::Prop(name)] = steps else { return None };
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
            FieldBase::This => self.frames[top].this.as_ref(),
        };
        let o = base_val.and_then(deref_object)?;
        let cid = o.borrow().class_id as usize;
        let pi = prop_info(&self.classes, cid, name)?;
        let sv = pi.set_visibility?;
        if visible_from(&self.classes, self.frames[top].class, sv, pi.declaring_class) {
            return None;
        }
        let key = self.prop_storage_key(cid, name, self.frames[top].class);
        let val = o.borrow().props.get(&key).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        Some(Rc::new(RefCell::new(val)))
    }

    /// If a write/ref path starts at a property whose get hook returns **by
    /// reference** (`&get`, PHP 8.4), run the hook and hand back the cell it
    /// returned: the property becomes addressable and the cell is the root the
    /// rest of the path drills into (`$r = &$o->prop`, `$o->prop[] = v`,
    /// `sort($o->prop)`). `None` when the path does not start at such a
    /// property (or the hook guard is active — the hook's own backing access).
    /// The hook frame is driven to its return synchronously; its `ret_deref`
    /// is cleared because this dispatch serves a *place* context. A hook that
    /// returns a non-reference yields a detached cell (writes are lost, like
    /// PHP's temporary).
    fn byref_hook_root(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
    ) -> Result<Option<Rc<RefCell<Zval>>>, PhpError> {
        let name: Box<[u8]> = match steps.first() {
            Some(FieldStep::Prop(n)) => n.clone(),
            _ => return Ok(None),
        };
        let base_val = match base {
            FieldBase::Local(s) => self.frames[top].slots.get(s as usize),
            FieldBase::Global(s) => self.frames[0].slots.get(s as usize),
            FieldBase::Superglobal(i) => self.superglobals.get(i as usize),
            FieldBase::This => self.frames[top].this.as_ref(),
        };
        let Some(target) = base_val.map(|v| v.deref_clone()) else { return Ok(None) };
        if !matches!(target, Zval::Object(_)) {
            return Ok(None);
        }
        // A write fetch of a lazy object's property initializes it first (PHP
        // 8.4) — unless a hook serves it; an initialized proxy forwards to its
        // real instance (transitively — the instance may have been reset lazy).
        let target = self.lazy_prop_access(
            target,
            &name,
            self.frames[top].class,
            Some(false),
            (MagicKind::Get, b"__get"),
        )?;
        let Some(o) = deref_object(&target) else { return Ok(None) };
        let (oid, cid) = {
            let b = o.borrow();
            (b.id, b.class_id as usize)
        };
        if self.hook_guarded(oid, &name) {
            return Ok(None);
        }
        let Some(func) = self.prop_hook(cid, &name, false).filter(|f| f.by_ref) else {
            return Ok(None);
        };
        let baseline = self.frames.len();
        self.push_hook(func, target, oid, &name, None);
        self.frames.last_mut().expect("hook frame just pushed").ret_deref = false;
        let v = self.drive_to_return(baseline)?;
        Ok(Some(match v {
            Zval::Ref(rc) => rc,
            other => Rc::new(RefCell::new(other)),
        }))
    }

}

/// The leaf operation of a path write, carried to the bottom of the drill-down.
enum Last {
    Set { key: Zval, value: Zval },
    Append { value: Zval },
    OpSet { key: Zval, op: BinOp, rhs: Zval },
    IncDec { key: Zval, inc: bool, pre: bool },
}

/// A dim-write step of a *variable-rooted* path (`$a[0][1] = v`) that landed on
/// an OBJECT: [`Vm::path_op`] drains these through the ArrayAccess protocol —
/// `offsetSet` at the leaf, `offsetGet` + `offsetSet` for the compound leaves,
/// `offsetGet` + resumed drill mid-path (sodium_compat's BLAKE2b builds
/// `SplFixedArray` trees and writes `$ctx[0][$i]`).
enum PathAa {
    Write(crate::vm::arrays::AaWrite),
    Op { obj: Zval, key: Zval, op: BinOp, rhs: Zval },
    IncDec { obj: Zval, key: Zval, inc: bool, pre: bool },
    Descend { obj: Zval, key: Zval, rest: Vec<Zval>, last: Box<Last> },
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

/// Encode a [`TypeHint`] as the descriptor `ReflectionNamedType` is built from:
/// `false` for no type, else `['name' => str, 'builtin' => bool, 'nullable' =>
/// bool]`. Union/intersection/`self`/`void`/… hints lower to `None` upstream, so
/// they reflect as no type (a documented limitation).
fn typehint_descriptor(hint: &Option<TypeHint>) -> Zval {
    let Some(h) = hint else { return Zval::Bool(false) };
    let (name, builtin): (&[u8], bool) = match &h.kind {
        HintKind::Scalar(ScalarType::Int) => (b"int", true),
        HintKind::Scalar(ScalarType::Float) => (b"float", true),
        HintKind::Scalar(ScalarType::String) => (b"string", true),
        HintKind::Scalar(ScalarType::Bool) => (b"bool", true),
        HintKind::Array => (b"array", true),
        HintKind::Callable => (b"callable", true),
        HintKind::Iterable => (b"iterable", true),
        HintKind::Object => (b"object", true),
        HintKind::Class(c) => (c, false),
        // Composite hints reflect through `reflect_type_descriptor` (the
        // side-carried `ReflectType`), not this single-type descriptor.
        HintKind::Union(_) => return Zval::Bool(false),
    };
    let mut a = php_types::PhpArray::new();
    a.insert(Key::Str(PhpStr::new(b"name".to_vec())), Zval::Str(PhpStr::new(name.to_vec())));
    a.insert(Key::Str(PhpStr::new(b"builtin".to_vec())), Zval::Bool(builtin));
    a.insert(Key::Str(PhpStr::new(b"nullable".to_vec())), Zval::Bool(h.nullable));
    Zval::Array(Rc::new(a))
}


/// Encode a composite (union/intersection) [`ReflectType`] as the descriptor the
/// prelude builds a `ReflectionUnionType` / `ReflectionIntersectionType` from:
/// `{kind: "union"|"intersection", types: [{name,builtin,nullable}, …],
/// nullable}`. `T|null` normalises to a single nullable `ReflectionNamedType`
/// descriptor (matching PHP). Returns `None` when there is no composite type (the
/// caller then falls back to the single-type `typehint_descriptor`).
fn reflect_type_descriptor(rt: &Option<crate::hir::ReflectType>) -> Option<Zval> {
    use crate::hir::{ReflectNamed, ReflectType};
    let named = |n: &ReflectNamed, nullable: bool| -> Zval {
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"name".to_vec())), Zval::Str(PhpStr::new(n.name.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"builtin".to_vec())), Zval::Bool(n.builtin));
        a.insert(Key::Str(PhpStr::new(b"nullable".to_vec())), Zval::Bool(nullable));
        Zval::Array(Rc::new(a))
    };
    let composite = |kind: &[u8], members: &[ReflectNamed], nullable: bool| -> Zval {
        let mut types = php_types::PhpArray::new();
        for m in members {
            let _ = types.append(named(m, false));
        }
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"kind".to_vec())), Zval::Str(PhpStr::new(kind.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"types".to_vec())), Zval::Array(Rc::new(types)));
        a.insert(Key::Str(PhpStr::new(b"nullable".to_vec())), Zval::Bool(nullable));
        Zval::Array(Rc::new(a))
    };
    match rt.as_ref()? {
        ReflectType::Single(n, nullable) => Some(named(n, *nullable)),
        ReflectType::Union(members) => {
            let has_null = members.iter().any(|m| m.name.eq_ignore_ascii_case(b"null"));
            let non_null: Vec<&ReflectNamed> = members.iter().filter(|m| !m.name.eq_ignore_ascii_case(b"null")).collect();
            // `T|null` (one non-null member + null) is a nullable single type.
            if has_null && non_null.len() == 1 {
                return Some(named(non_null[0], true));
            }
            // PHP canonicalises union member order: class types first (in source
            // order), then built-ins in a fixed order. A stable sort by rank keeps
            // classes (rank 0) in source order.
            let mut ordered = members.clone();
            ordered.sort_by_key(|m| if m.builtin { union_builtin_rank(&m.name) } else { 0 });
            Some(composite(b"union", &ordered, has_null))
        }
        // Intersection members keep their source order (PHP does not reorder them).
        ReflectType::Intersection(members) => Some(composite(b"intersection", members, false)),
    }
}

/// PHP's canonical ordering rank for a built-in type within a union (class types
/// rank 0 and keep source order); lower sorts earlier.
fn union_builtin_rank(name: &[u8]) -> i32 {
    match name {
        b"static" => 1,
        b"callable" => 2,
        b"object" => 3,
        b"array" | b"iterable" => 4,
        b"string" => 5,
        b"int" => 6,
        b"float" => 7,
        b"bool" | b"false" | b"true" => 8,
        b"null" => 9,
        _ => 10,
    }
}

/// One frame of the output-buffering stack (`ob_start`). `content` accumulates
/// the captured output; `callback` is the optional user handler passed to
/// `ob_start(callable $callback)`, invoked with the buffer content when the
/// buffer is flushed (its non-false string return replaces the output).
/// `chunk_size` (0 = none) auto-flushes the buffer to its parent as soon as a
/// write makes `content` reach it. `started` records whether the handler has been
/// invoked at least once, so the phase bitmask passed to it carries
/// `PHP_OUTPUT_HANDLER_START` (1) on the first call only.
/// `flags` are the capability bits passed as `ob_start`'s third argument
/// (CLEANABLE 16 | FLUSHABLE 32 | REMOVABLE 64; default STDFLAGS 112): a
/// missing bit makes the corresponding `ob_*` operation fail with a notice
/// (main/output.c php_output_flush/clean/stack_pop). The script-end implicit
/// flush ignores them (POP_FORCE).
struct OutputBuffer {
    content: Vec<u8>,
    callback: Option<Zval>,
    chunk_size: usize,
    flags: i64,
    started: bool,
}

/// Single source of truth for the *call-a-callable / introspection* host-builtin
/// family (Sessions B–D): one table generates both [`host_builtin_canonical`] (the
/// compiler's recognition predicate — decides whether to emit
/// [`crate::bytecode::Op::CallHostBuiltin`]) and [`Vm::dispatch_host_builtin`] (the
/// runtime match). Adding a host builtin is therefore one edit in the table below,
/// not the former two-list sync. `name`/`args` name the bindings each arm body sees:
/// the dispatched (already canonical, lowercased) name bytes and the by-value
/// argument vector. An arm may list several names (`a | b => …`) sharing one body.
/// `vm` names the receiver the arm bodies use (the `self` keyword can't cross the
/// macro hygiene boundary, so it is rebound as a plain identifier).
macro_rules! host_builtins {
    (
        vm: $vm:ident, name: $name:ident, args: $args:ident;
        $( $( $lit:literal )|+ => $body:expr , )+
    ) => {
        pub(crate) fn host_builtin_canonical(name: &[u8]) -> Option<&'static [u8]> {
            HOST_BUILTIN_NAMES.iter().copied().find(|h| name.eq_ignore_ascii_case(h))
        }

        /// Every host-builtin name, for `get_defined_functions()['internal']`.
        pub(crate) const HOST_BUILTIN_NAMES: &[&[u8]] = &[ $( $( $lit , )+ )+ ];

        impl<'m> Vm<'m> {
            /// Dispatch an evaluator-only *host* builtin emitted as
            /// [`crate::bytecode::Op::CallHostBuiltin`]: the call-a-callable /
            /// introspection family. `name` is the canonical lowercased name from
            /// [`host_builtin_canonical`]. Generated by [`host_builtins!`].
            fn dispatch_host_builtin(&mut self, $name: &[u8], $args: Vec<Zval>) -> Result<Zval, PhpError> {
                let $vm = self;
                match $name {
                    $( $( $lit )|+ => $body , )+
                    _ => Err(undefined_builtin($name)),
                }
            }
        }
    };
}

host_builtins! {
    vm: vm, name: name, args: args;
    b"gc_collect_cycles" => vm.ho_gc_collect_cycles(args),
    b"spl_autoload_register" => vm.ho_spl_autoload_register(args),
    b"spl_autoload_unregister" => vm.ho_spl_autoload_unregister(args),
    b"spl_autoload_functions" => vm.ho_spl_autoload_functions(),
    b"spl_autoload_call" => vm.ho_spl_autoload_call(args),
    b"call_user_func" => vm.ho_call_user_func(args),
    b"call_user_func_array" => vm.ho_call_user_func_array(args),
    b"iterator_to_array" => vm.ho_iterator_to_array(args),
    b"iterator_count" => vm.ho_iterator_count(args),
    b"json_encode" => vm.ho_json_encode(args),
    b"is_callable" => vm.ho_is_callable(args),
    b"is_iterable" => vm.ho_is_iterable(args),
    b"filter_var" => vm.ho_filter_var(args),
    b"define" => vm.ho_define(args),
    b"defined" => vm.ho_defined(args),
    b"constant" => vm.ho_constant(args),
    b"token_get_all" => vm.ho_token_get_all(args),
    b"token_name" => vm.ho_token_name(args),
    b"array_map" => vm.ho_array_map(args),
    b"array_filter" => vm.ho_array_filter(args),
    b"array_reduce" => vm.ho_array_reduce(args),
    b"array_all" => vm.ho_array_all(args),
    b"array_any" => vm.ho_array_any(args),
    b"array_find" => vm.ho_array_find(args),
    b"array_find_key" => vm.ho_array_find_key(args),
    b"get_class" => vm.ho_get_class(args),
    b"get_debug_type" => vm.ho_get_debug_type(args),
    b"spl_object_id" => vm.ho_spl_object_id(args),
    b"spl_object_hash" => vm.ho_spl_object_hash(args),
    b"__weak_create" => vm.ho_weak_create(args),
    b"__weak_get" => vm.ho_weak_get(args),
    b"__reflect_static_prop_get" => vm.ho_reflect_static_prop_get(args),
    b"__reflect_static_props" => vm.ho_reflect_static_props(args),
    b"__reflect_static_vars" => vm.ho_reflect_static_vars(args),
    b"__reflect_closure_bind" => vm.ho_reflect_closure_bind(args),
    b"__reflect_closure_uses" => vm.ho_reflect_closure_uses(args),
    b"__reflect_static_prop_set" => vm.ho_reflect_static_prop_set(args),
    b"__zip_open" => vm.ho_zip_open(args),
    b"__zip_writer_open" => vm.ho_zip_writer_open(args),
    b"__zip_writer_add" => vm.ho_zip_writer_add(args),
    b"__zip_writer_close" => vm.ho_zip_writer_close(args),
    b"__zip_close" => vm.ho_zip_close(args),
    b"__zip_stat_index" => vm.ho_zip_stat_index(args),
    b"__zip_get_name_index" => vm.ho_zip_get_name_index(args),
    b"__zip_locate_name" => vm.ho_zip_locate_name(args),
    b"__zip_get_from_index" => vm.ho_zip_get_from_index(args),
    b"__zip_extract_to" => vm.ho_zip_extract_to(args),
    b"__pdo_open" => vm.ho_pdo_open(args),
    b"__pdo_close" => vm.ho_pdo_close(args),
    b"__pdo_sqlite_version" => vm.ho_pdo_sqlite_version(),
    b"__pdo_exec" => vm.ho_pdo_exec(args),
    b"__pdo_create_function" => vm.ho_pdo_create_function(args),
    b"__pdo_run" => vm.ho_pdo_run(args),
    b"__pdo_prepare" => vm.ho_pdo_prepare(args),
    b"__pdo_last_id" => vm.ho_pdo_last_id(args),
    b"__pdo_in_txn" => vm.ho_pdo_in_txn(args),
    b"__pdo_stmt_readonly" => vm.ho_pdo_stmt_readonly(args),
    b"__pdo_changes" => vm.ho_pdo_changes(args),
    b"__pdo_param_count" => vm.ho_pdo_param_count(args),
    b"__mysqli_connect" => vm.ho_mysqli_connect(args),
    b"__mysqli_close" => vm.ho_mysqli_close(args),
    b"__mysqli_query" => vm.ho_mysqli_query(args),
    b"__mysqli_more_results" => vm.ho_mysqli_more_results(args),
    b"__mysqli_next_result" => vm.ho_mysqli_next_result(args),
    b"__mysqli_select_db" => vm.ho_mysqli_select_db(args),
    b"__mysqli_set_charset" => vm.ho_mysqli_set_charset(args),
    b"__mysqli_charset" => vm.ho_mysqli_charset(args),
    b"__mysqli_escape" => vm.ho_mysqli_escape(args),
    b"__mysqli_ping" => vm.ho_mysqli_ping(args),
    b"__mysqli_stat" => vm.ho_mysqli_stat(args),
    b"__mysqli_prepare" => vm.ho_mysqli_prepare(args),
    b"__mysqli_multi_query" => vm.ho_mysqli_multi_query(args),
    b"__mysqli_stmt_execute" => vm.ho_mysqli_stmt_execute(args),
    b"__mysqli_stmt_close" => vm.ho_mysqli_stmt_close(args),
    b"getopt" => vm.ho_getopt(args),
    b"__xslt_import" => vm.ho_xslt_import(args),
    b"__xslt_free" => vm.ho_xslt_free(args),
    b"__xslt_transform" => vm.ho_xslt_transform(args),
    b"__gd_create" => vm.ho_gd_create(args),
    b"__gd_destroy" => vm.ho_gd_destroy(args),
    b"__gd_decode" => vm.ho_gd_decode(args),
    b"__gd_decode_auto" => vm.ho_gd_decode_auto(args),
    b"__gd_encode" => vm.ho_gd_encode(args),
    b"__gd_stat" => vm.ho_gd_stat(args),
    b"__gd_flag" => vm.ho_gd_flag(args),
    b"__gd_color" => vm.ho_gd_color(args),
    b"__gd_colortransparent" => vm.ho_gd_colortransparent(args),
    b"__gd_colorsforindex" => vm.ho_gd_colorsforindex(args),
    b"__gd_colorat" => vm.ho_gd_colorat(args),
    b"__gd_setpixel" => vm.ho_gd_setpixel(args),
    b"__gd_draw" => vm.ho_gd_draw(args),
    b"__gd_copy" => vm.ho_gd_copy(args),
    b"__gd_rotate" => vm.ho_gd_rotate(args),
    b"__gd_flip" => vm.ho_gd_flip(args),
    b"__gd_crop" => vm.ho_gd_crop(args),
    b"__gd_scale" => vm.ho_gd_scale(args),
    b"__gd_setinterpolation" => vm.ho_gd_setinterpolation(args),
    b"__gd_t2p" => vm.ho_gd_t2p(args),
    b"__gd_p2t" => vm.ho_gd_p2t(args),
    b"__gd_string" => vm.ho_gd_string(args),
    b"__gd_char" => vm.ho_gd_char(args),
    b"__gd_fontsize" => vm.ho_gd_fontsize(args),
    b"__gd_version" => vm.ho_gd_version(),
    b"get_parent_class" => vm.ho_get_parent_class(args),
    b"class_parents" => vm.ho_class_parents(args),
    b"class_implements" => vm.ho_class_implements(args),
    b"is_a" => vm.ho_is_a(args),
    b"is_subclass_of" => vm.ho_is_subclass_of(args),
    b"__reflect_class_constants" => vm.ho_reflect_class_constants(args),
    b"__reflect_class_const_names" => vm.ho_reflect_class_const_names(args),
    b"__reflect_class_const_info" => vm.ho_reflect_class_const_info(args),
    b"__reflect_enum_backing" => vm.ho_reflect_enum_backing(args),
    b"__reflect_classconst_attributes" => vm.ho_reflect_classconst_attributes(args),
    b"__reflect_classconst_attr_new" => vm.ho_reflect_classconst_attr_new(args),
    b"__reflect_classconst_attr_args" => vm.ho_reflect_classconst_attr_args(args),
    b"__reflect_param_attributes" => vm.ho_reflect_param_attributes(args),
    b"__reflect_param_attr_new" => vm.ho_reflect_param_attr_new(args),
    b"__reflect_param_attr_args" => vm.ho_reflect_param_attr_args(args),
    b"class_uses" => vm.ho_class_uses(args),
    b"trait_exists" => vm.ho_trait_exists(args),
    b"get_declared_traits" => vm.ho_get_declared_traits(),
    b"__reflect_class_attributes" => vm.ho_reflect_class_attributes(args),
    b"__reflect_attr_newinstance" => vm.ho_reflect_attr_newinstance(args),
    b"__reflect_attr_arguments" => vm.ho_reflect_attr_arguments(args),
    b"__reflect_prop_declaring_class" => vm.ho_reflect_prop_declaring_class(args),
    b"__reflect_method_names" => vm.ho_reflect_method_names(args),
    b"__reflect_prop_defaults" => vm.ho_reflect_prop_defaults(args),
    b"implode" | b"join" => vm.ho_implode(args),
    b"__reflect_func_info" => vm.ho_reflect_func_info(args),
    b"__reflect_closure_info" => vm.ho_reflect_closure_info(args),
    b"__reflect_closure_attributes" => vm.ho_reflect_closure_attributes(args),
    b"__reflect_closure_attr_new" => vm.ho_reflect_closure_attr_new(args),
    b"__reflect_closure_attr_args" => vm.ho_reflect_closure_attr_args(args),
    b"__reflect_method_info" => vm.ho_reflect_method_info(args),
    b"__reflect_object_bind" => vm.ho_reflect_object_bind(args),
    b"__reflect_object_dynprops" => vm.ho_reflect_object_dynprops(args),
    b"__reflect_object_instance" => vm.ho_reflect_object_instance(args),
    b"__reflect_invoke" => vm.ho_reflect_invoke(args),
    b"__reflect_class_modifiers" => vm.ho_reflect_class_modifiers(args),
    b"__reflect_new_no_ctor" => vm.ho_reflect_new_no_ctor(args),
    b"__reflect_new_lazy_ghost" => vm.ho_reflect_new_lazy_ghost(args),
    b"__reflect_new_lazy_proxy" => vm.ho_reflect_new_lazy_proxy(args),
    b"__reflect_reset_lazy" => vm.ho_reflect_reset_lazy(args),
    b"__lazy_is_uninitialized" => vm.ho_lazy_is_uninitialized(args),
    b"__lazy_is_initializing" => vm.ho_lazy_is_initializing(args),
    b"__lazy_initialize" => vm.ho_lazy_initialize(args),
    b"__lazy_mark_initialized" => vm.ho_lazy_mark_initialized(args),
    b"__lazy_skip_init" => vm.ho_lazy_skip_init(args),
    b"__lazy_set_raw" => vm.ho_lazy_set_raw(args),
    b"__lazy_get_initializer" => vm.ho_lazy_get_initializer(args),
    b"__lazy_prop_is_lazy" => vm.ho_lazy_prop_is_lazy(args),
    b"__reflect_prop_names" => vm.ho_reflect_prop_names(args),
    b"__reflect_prop_is_static" => vm.ho_reflect_prop_is_static(args),
    b"__reflect_prop_type" => vm.ho_reflect_prop_type(args),
    b"__reflect_prop_details" => vm.ho_reflect_prop_details(args),
    b"__reflect_prop_initialized" => vm.ho_reflect_prop_initialized(args),
    b"__reflect_prop_get" => vm.ho_reflect_prop_get(args),
    b"__reflect_prop_set" => vm.ho_reflect_prop_set(args),
    b"__reflect_prop_attributes" => vm.ho_reflect_prop_attributes(args),
    b"__reflect_prop_attr_new" => vm.ho_reflect_prop_attr_new(args),
    b"__reflect_prop_attr_args" => vm.ho_reflect_prop_attr_args(args),
    b"__reflect_func_attributes" => vm.ho_reflect_func_attributes(args),
    b"__reflect_func_attr_new" => vm.ho_reflect_func_attr_new(args),
    b"__reflect_func_attr_args" => vm.ho_reflect_func_attr_args(args),
    b"__reflect_method_attributes" => vm.ho_reflect_method_attributes(args),
    b"__reflect_method_attr_new" => vm.ho_reflect_method_attr_new(args),
    b"__reflect_method_attr_args" => vm.ho_reflect_method_attr_args(args),
    b"__reflect_const_attributes" => vm.ho_reflect_const_attributes(args),
    b"__reflect_const_attr_new" => vm.ho_reflect_const_attr_new(args),
    b"__reflect_const_attr_args" => vm.ho_reflect_const_attr_args(args),
    b"get_object_vars" => vm.ho_get_object_vars(args),
    b"get_class_vars" => vm.ho_get_class_vars(args),
    b"register_shutdown_function" => vm.ho_register_shutdown_function(args),
    b"get_class_methods" => vm.ho_get_class_methods(args),
    b"func_num_args" => vm.ho_func_num_args(),
    b"func_get_args" => vm.ho_func_get_args(),
    b"func_get_arg" => vm.ho_func_get_arg(args),
    b"sprintf" | b"printf" | b"vsprintf" | b"vprintf" | b"fprintf" | b"vfprintf" => vm.ho_format(name, args),
    b"function_exists" => vm.ho_function_exists(args),
    b"get_defined_functions" => vm.ho_get_defined_functions(),
    b"class_exists" => vm.ho_class_exists(args),
    b"class_alias" => vm.ho_class_alias(args),
    b"interface_exists" => vm.ho_interface_exists(args),
    b"method_exists" => vm.ho_method_exists(args),
    b"property_exists" => vm.ho_property_exists(args),
    b"get_called_class" => vm.ho_get_called_class(),
    b"error_reporting" => vm.ho_error_reporting(args),
    // CLI runtime-environment stubs: no request/client connection or wall-clock
    // limit, so these report the fixed CLI state (byte-identical to php-cli).
    b"set_time_limit" => vm.ho_set_time_limit(args),
    b"ignore_user_abort" => vm.ho_ignore_user_abort(args),
    b"connection_aborted" => Ok(Zval::Long(0)), // never aborted under CLI
    b"connection_status" => Ok(Zval::Long(0)),  // CONNECTION_NORMAL
    b"trigger_error" | b"user_error" => vm.ho_trigger_error(args),
    // Prelude-internal: raise an E_DEPRECATED attributed to the *caller* of the
    // prelude method (PHP reports an internal method's deprecation at the call
    // site, not inside the implementation — SplObjectStorage::attach & co.).
    b"__notice_from_caller" => {
        let msg = convert::to_zstr_cast(
            &args.first().map(|a| a.deref_clone()).unwrap_or(Zval::Null),
            &mut vm.diags,
        )
        .as_bytes()
        .to_vec();
        let caller = vm.frames.len().saturating_sub(2);
        let line = vm.cur_line(caller);
        vm.flush_diags(line)?;
        vm.raise_diagnostic(8, &String::from_utf8_lossy(&msg), line)?;
        Ok(Zval::Null)
    },
    b"__deprecated_from_caller" => {
        let msg = convert::to_zstr_cast(
            &args.first().map(|a| a.deref_clone()).unwrap_or(Zval::Null),
            &mut vm.diags,
        )
        .as_bytes()
        .to_vec();
        let caller = vm.frames.len().saturating_sub(2);
        let line = vm.cur_line(caller);
        vm.flush_diags(line)?;
        vm.raise_diagnostic(8192, &String::from_utf8_lossy(&msg), line)?;
        Ok(Zval::Null)
    },
    // Prelude-internal: raise an E_WARNING attributed to the *caller* of the
    // prelude function/method (same shape as __deprecated_from_caller): the
    // mysqli REPORT_ERROR path reports at the mysqli_query() call site.
    b"__warning_from_caller" => {
        let msg = convert::to_zstr_cast(
            &args.first().map(|a| a.deref_clone()).unwrap_or(Zval::Null),
            &mut vm.diags,
        )
        .as_bytes()
        .to_vec();
        let caller = vm.frames.len().saturating_sub(2);
        let line = vm.cur_line(caller);
        vm.flush_diags(line)?;
        vm.raise_diagnostic(2, &String::from_utf8_lossy(&msg), line)?;
        Ok(Zval::Null)
    },
    b"error_get_last" => vm.ho_error_get_last(),
    // `get_defined_vars()`: the calling scope's named locals (in declaration
    // order, undefined slots omitted), values by snapshot — host builtins run
    // frameless, so the top frame IS the caller.
    b"get_defined_vars" => {
        let top = vm.frames.len() - 1;
        let names: Vec<Box<[u8]>> = vm.frames[top].func.slot_names.to_vec();
        let mut arr = PhpArray::new();
        for (i, name) in names.iter().enumerate() {
            if let Some(v) = vm.frames[top].slots.get(i) {
                let v = v.deref_clone();
                if !matches!(v, Zval::Undef) {
                    arr.insert(Key::from_bytes(name), v);
                }
            }
        }
        // Dynamically-created names too: an eval'd unit publishing a NEW
        // variable into this scope lands in dyn_vars (wp-cli reads
        // wp-config.php's $table_prefix this way), as do $$name writes.
        let dyn_pairs: Vec<(Vec<u8>, Zval)> = vm.frames[top]
            .dyn_vars
            .iter()
            .map(|(k, v)| (k.clone(), v.deref_clone()))
            .collect();
        for (k, v) in dyn_pairs {
            if !matches!(v, Zval::Undef) && !arr.contains_key(&Key::from_bytes(&k)) {
                arr.insert(Key::from_bytes(&k), v);
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    },
    b"error_clear_last" => {
        // Realize pending diags first (same move as error_get_last), then forget.
        let line = vm.cur_line(vm.frames.len() - 1);
        vm.flush_diags(line)?;
        vm.last_error = None;
        Ok(Zval::Null)
    },
    b"set_exception_handler" => vm.ho_set_exception_handler(args),
    b"restore_exception_handler" => vm.ho_restore_exception_handler(),
    b"set_error_handler" => vm.ho_set_error_handler(args),
    b"restore_error_handler" => vm.ho_restore_error_handler(),
    b"get_error_handler" => vm.ho_get_error_handler(),
    b"get_exception_handler" => vm.ho_get_exception_handler(),
    b"unserialize" => vm.ho_unserialize(args),
    b"fopen" => vm.ho_fopen(args),
    b"tmpfile" => vm.ho_tmpfile(),
    b"opendir" => vm.ho_opendir(args),
    b"stream_context_create" => vm.ho_stream_context_create(args),
    b"stream_context_get_options" => vm.ho_stream_context_get_options(args),
    b"stream_context_get_params" => vm.ho_stream_context_get_params(args),
    b"stream_context_set_params" => vm.ho_stream_context_set_params(args),
    b"stream_context_set_option" => vm.ho_stream_context_set_option(args),
    b"stream_context_set_options" => vm.ho_stream_context_set_options(args),
    b"stream_set_chunk_size" => vm.ho_stream_set_chunk_size(args),
    b"stream_get_meta_data" => vm.ho_stream_get_meta_data(args),
    b"shell_exec" => vm.ho_shell_exec(args),
    b"filter_input" => vm.ho_filter_input(args),
    b"proc_close" => vm.ho_proc_close(args),
    b"__stream_select" => vm.ho_stream_select(args),
    b"proc_get_status" => vm.ho_proc_get_status(args),
    b"proc_terminate" => vm.ho_proc_terminate(args),
    b"__reflect_class_doc" => vm.ho_reflect_class_doc(args),
    b"pcntl_signal" => vm.ho_pcntl_signal(args),
    b"pcntl_signal_get_handler" => vm.ho_pcntl_signal_get_handler(args),
    b"pcntl_signal_dispatch" => vm.ho_pcntl_signal_dispatch(args),
    b"pcntl_async_signals" => vm.ho_pcntl_async_signals(args),
    b"posix_kill" => vm.ho_posix_kill(args),
    b"__fsockopen" => vm.ho_fsockopen(args),
    b"stream_set_timeout" => vm.ho_stream_set_timeout(args),
    b"stream_is_local" => vm.ho_stream_is_local(args),
    b"stream_wrapper_register" | b"stream_register_wrapper" => vm.ho_stream_wrapper_register(args),
    b"stream_wrapper_unregister" => vm.ho_stream_wrapper_unregister(args),
    b"stream_resolve_include_path" => vm.ho_stream_resolve_include_path(args),
    b"stream_get_line" => vm.ho_stream_get_line(args),
    b"stream_filter_append" => vm.ho_stream_filter_append(args, false),
    b"stream_filter_prepend" => vm.ho_stream_filter_append(args, true),
    b"gzopen" => vm.ho_gzopen(args),
    b"serialize" => vm.ho_serialize(args),
    b"umask" => vm.ho_umask(args),
    b"__dom_new_doc" => vm.ho_dom_new_doc(args),
    b"__dom_load" => vm.ho_dom_load(args),
    b"__dom_load_html" => vm.ho_dom_load_html(args),
    b"__dom_save_xml" => vm.ho_dom_save_xml(args),
    b"__dom_c14n" => vm.ho_dom_c14n(args),
    b"__dom_normalize" => vm.ho_dom_normalize(args),
    b"__dom_save_html" => vm.ho_dom_save_html(args),
    b"__xml_tokenize" => vm.ho_xml_tokenize(args),
    b"__dom_info" => vm.ho_dom_info(args),
    b"__dom_nav" => vm.ho_dom_nav(args),
    b"__dom_children" => vm.ho_dom_children(args),
    b"__dom_text" => vm.ho_dom_text(args),
    b"__dom_ns" => vm.ho_dom_ns(args),
    b"__dom_set_value" => vm.ho_dom_set_value(args),
    b"__dom_attr" => vm.ho_dom_attr(args),
    b"__dom_create" => vm.ho_dom_create(args),
    b"__dom_mutate" => vm.ho_dom_mutate(args),
    b"__dom_copy" => vm.ho_dom_copy(args),
    b"__dom_doc_element" => vm.ho_dom_doc_element(args),
    b"__dom_by_tag" => vm.ho_dom_by_tag(args),
    b"__dom_xpath" => vm.ho_dom_xpath(args),
    b"__dom_doc_meta" => vm.ho_dom_doc_meta(args),
    b"__reflect_class_loc" => vm.ho_reflect_class_loc(args),
    b"__reflect_class_real_name" => vm.ho_reflect_class_real_name(args),
    b"__reflect_ref_id" => vm.ho_reflect_ref_id(args),
    b"__reflect_gen_info" => vm.ho_reflect_gen_info(args),
    b"__reflect_fiber_info" => vm.ho_reflect_fiber_info(args),
    b"array_diff" => vm.ho_array_diff(args),
    b"get_declared_classes" => vm.ho_get_declared(0),
    b"get_declared_interfaces" => vm.ho_get_declared(1),
    b"libxml_use_internal_errors" => vm.ho_libxml_use_internal_errors(args),
    b"__libxml_get_errors" => vm.ho_libxml_get_errors(),
    b"libxml_clear_errors" => vm.ho_libxml_clear_errors(),
    b"preg_replace" => vm.ho_preg_replace(args).map(|(r, _)| r),
    b"preg_quote" => vm.ho_preg_quote(args),
    b"preg_split" => vm.ho_preg_split(args),
    b"preg_grep" => vm.ho_preg_grep(args),
    b"strtok" => vm.ho_strtok(args),
    b"random_int" => vm.ho_random_int(args),
    b"random_bytes" => vm.ho_random_bytes(args),
    b"debug_backtrace" => vm.ho_debug_backtrace(args),
    b"debug_print_backtrace" => vm.ho_debug_print_backtrace(),
    b"preg_replace_callback" => vm.ho_preg_replace_callback(args),
    b"json_decode" => vm.ho_json_decode(args),
    b"json_validate" => vm.ho_json_validate(args),
    b"compact" => vm.ho_compact(args),
    b"extract" => vm.ho_extract(args),
    b"array_udiff" => vm.ho_array_udiff(args),
    b"array_uintersect" => vm.ho_array_uintersect(args),
    b"array_diff_ukey" => vm.ho_array_diff_ukey(args),
    b"array_intersect_ukey" => vm.ho_array_intersect_ukey(args),
    b"array_udiff_assoc" => vm.ho_array_udiff_assoc(args),
    b"array_uintersect_assoc" => vm.ho_array_uintersect_assoc(args),
    b"array_diff_uassoc" => vm.ho_array_diff_uassoc(args),
    b"array_intersect_uassoc" => vm.ho_array_intersect_uassoc(args),
    b"array_udiff_uassoc" => vm.ho_array_udiff_uassoc(args),
    b"array_uintersect_uassoc" => vm.ho_array_uintersect_uassoc(args),
    b"header" => vm.ho_header(args),
    b"headers_sent" => vm.ho_headers_sent(args),
    b"preg_last_error" => vm.ho_preg_last_error(),
    b"preg_last_error_msg" => vm.ho_preg_last_error_msg(),
    b"ini_get" => vm.ho_ini_get(args),
    b"error_log" => vm.ho_error_log(args),
    b"get_cfg_var" => vm.ho_get_cfg_var(args),
    b"ini_set" => vm.ho_ini_set(args),
    b"ini_restore" => vm.ho_ini_restore(args),
    b"ini_get_all" => vm.ho_ini_get_all(args),
    b"get_include_path" => vm.ho_get_include_path(),
    b"set_include_path" => vm.ho_set_include_path(args),
    b"restore_include_path" => vm.ho_restore_include_path(),
    b"session_status" => vm.ho_session_status(),
    b"session_id" => vm.ho_session_id(args),
    b"session_name" => vm.ho_session_name(args),
    b"session_save_path" => vm.ho_session_save_path(args),
    b"session_module_name" => vm.ho_session_module_name(args),
    b"session_cache_limiter" => vm.ho_session_cache_limiter(args),
    b"session_cache_expire" => vm.ho_session_cache_expire(args),
    b"session_get_cookie_params" => vm.ho_session_get_cookie_params(),
    b"session_set_cookie_params" => vm.ho_session_set_cookie_params(args),
    b"session_start" => vm.ho_session_start(args),
    b"session_write_close" | b"session_commit" => vm.ho_session_write_close(),
    b"session_abort" => vm.ho_session_abort(),
    b"session_reset" => vm.ho_session_reset(),
    b"session_unset" => vm.ho_session_unset(),
    b"session_destroy" => vm.ho_session_destroy(),
    b"session_gc" => vm.ho_session_gc(),
    b"session_regenerate_id" => vm.ho_session_regenerate_id(args),
    b"session_create_id" => vm.ho_session_create_id(args),
    b"session_encode" => vm.ho_session_encode(),
    b"session_decode" => vm.ho_session_decode(args),
    b"session_register_shutdown" => vm.ho_session_register_shutdown(),
    b"session_set_save_handler" => vm.ho_session_set_save_handler(args),
    b"__session_files_op" => vm.ho_session_files_op(args),
    b"headers_list" => vm.ho_headers_list(),
    b"setcookie" => vm.ho_setcookie(args, false),
    b"setrawcookie" => vm.ho_setcookie(args, true),
    b"header_remove" => vm.ho_header_remove(args),
    b"http_response_code" => vm.ho_http_response_code(args),
    b"json_last_error" => vm.ho_json_last_error(args),
    b"json_last_error_msg" => vm.ho_json_last_error_msg(args),
    b"assert" => vm.ho_assert(args),
    b"mb_split" => vm.ho_mb_split(args),
    b"mb_regex_encoding" => vm.ho_mb_regex_encoding(args),
    b"mb_regex_set_options" => vm.ho_mb_regex_set_options(args),
    b"mb_ereg_replace" => vm.ho_mb_ereg_replace(false, args),
    b"mb_eregi_replace" => vm.ho_mb_ereg_replace(true, args),
    b"mb_ereg_replace_callback" => vm.ho_mb_ereg_replace_callback(args),
    b"mb_ereg_match" => vm.ho_mb_ereg_match(args),
    b"mb_ereg_search_init" => vm.ho_mb_ereg_search_init(args),
    b"mb_ereg_search" => vm.ho_mb_ereg_search(args),
    b"mb_ereg_search_pos" => vm.ho_mb_ereg_search_pos(args),
    b"mb_ereg_search_regs" => vm.ho_mb_ereg_search_regs(args),
    b"mb_ereg_search_getregs" => vm.ho_mb_ereg_search_getregs(),
    b"mb_ereg_search_getpos" => Ok(Zval::Long(vm.mb_regex.search_pos as i64)),
    b"mb_ereg_search_setpos" => vm.ho_mb_ereg_search_setpos(args),
    b"ob_start" => vm.ho_ob_start(args),
    b"ob_get_contents" => vm.ho_ob_get_contents(),
    b"ob_get_clean" => vm.ho_ob_get_clean(),
    b"ob_get_flush" => vm.ho_ob_get_flush(),
    b"ob_end_clean" => vm.ho_ob_end_clean(),
    b"ob_end_flush" => vm.ho_ob_end_flush(),
    b"ob_flush" => vm.ho_ob_flush(),
    b"ob_clean" => vm.ho_ob_clean(),
    b"ob_get_level" => Ok(Zval::Long(vm.ob_stack.len() as i64)),
    b"ob_get_status" => vm.ho_ob_get_status(args),
    b"ob_get_length" => Ok(match vm.ob_stack.last() {
        Some(b) => Zval::Long(b.content.len() as i64),
        None => Zval::Bool(false),
    }),
    b"flush" => Ok(Zval::Null),
}

/// Like [`host_builtin_canonical`] but for the *by-reference-first* host builtins
/// (Session C): their first argument is an array variable taken by reference. The
/// compiler emits [`crate::bytecode::Op::CallHostBuiltinRef`] (with the variable's
/// slot) for these; [`Vm::dispatch_host_builtin_ref`] matches the same canonical
/// name. The two lists are disjoint.
/// One reconstructed call-stack entry (see [`Vm::collect_backtrace`]).
struct BtFrame {
    function: Vec<u8>,
    /// The file the call was made *from* (the caller frame's unit), or — when the
    /// caller is an `eval()` unit — the composite `<file>(<line>) : eval()'d code`.
    file: Vec<u8>,
    line: Line,
    class: Option<Vec<u8>>,
    is_static: bool,
    object: Option<Zval>,
    args: Vec<Zval>,
    /// True for an `eval()` unit's frame: rendered with no `args` in the array form.
    is_eval: bool,
}

/// The human-readable allowed-target list for an `#[Attribute(flags)]` bitmask,
/// in PHP's order — used in the "cannot target X (allowed targets: …)" error.
fn allowed_targets_str(flags: i64) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if flags & 1 != 0 { parts.push("class"); }
    if flags & 2 != 0 { parts.push("function"); }
    if flags & 4 != 0 { parts.push("method"); }
    if flags & 8 != 0 { parts.push("property"); }
    if flags & 16 != 0 { parts.push("class constant"); }
    if flags & 32 != 0 { parts.push("parameter"); }
    if flags & 64 != 0 { parts.push("constant"); }
    parts.join(", ")
}

/// Format one argument the way `debug_print_backtrace` does: scalars literal,
/// a string single-quoted and truncated to 15 bytes + `...`, arrays as `Array`,
/// objects/closures/generators as `Object(Class)`, resources as `Resource id #N`.
fn format_bt_arg(v: &Zval) -> String {
    match v {
        Zval::Undef | Zval::Null | Zval::ArgPlace(_) => "NULL".to_string(),
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
        Zval::WeakHandle(_) => "Object(WeakReference)".to_string(),
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
            target.type_name_for_error()
        )));
    };
    // The returned element is DEREF'd: PHP hands back the value, never the
    // reference wrapper (a `=&`-bound element returned raw would alias the
    // receiving variable — `$first = reset($this->resultPointers)` in ORM's
    // ArrayHydrator turned `$first` into an alias, and the next iteration's
    // plain assignment wrote *through* it, building a self-referential cell).
    Ok(match op {
        PtrOp::Current => rc.ptr_current().map(|v| v.deref_clone()).unwrap_or(Zval::Bool(false)),
        PtrOp::Key => rc.ptr_key().map(|k| key_to_zval(&k)).unwrap_or(Zval::Null),
        PtrOp::Reset => Rc::make_mut(rc).ptr_reset().map(|v| v.deref_clone()).unwrap_or(Zval::Bool(false)),
        PtrOp::End => Rc::make_mut(rc).ptr_end().map(|v| v.deref_clone()).unwrap_or(Zval::Bool(false)),
        PtrOp::Next => Rc::make_mut(rc).ptr_next().map(|v| v.deref_clone()).unwrap_or(Zval::Bool(false)),
        PtrOp::Prev => Rc::make_mut(rc).ptr_prev().map(|v| v.deref_clone()).unwrap_or(Zval::Bool(false)),
    })
}

/// Host builtins with a by-reference **output** parameter, mapping the canonical
/// name to the argument index of that out-param. `preg_match`/`preg_match_all`
/// write their captures into `&$matches` at index 2. The compiler emits
/// [`crate::bytecode::Op::CallHostBuiltinOut`] for these; [`Vm::dispatch_host_builtin_out`]
/// produces `(result, out_value)` and the VM writes the out-value into the slot.
/// ext/pcntl pending-signal bitmask (bit N = signal N), set by the async-safe
/// C-level catcher and drained by `Vm::dispatch_pending_signals`. Process-wide:
/// signals are delivered to the process, not to a VM instance.
static PENDING_SIGNALS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The C signal handler installed by `pcntl_signal`: async-signal-safe by
/// construction (a single atomic OR), everything else happens at dispatch time.
extern "C" fn pcntl_mark_pending(signo: libc::c_int) {
    if (0..64).contains(&signo) {
        PENDING_SIGNALS.fetch_or(1u64 << signo, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Outcome of [`Vm::dim_is_walk`] (nested isset/empty over ArrayAccess).
enum DimIsLeaf {
    /// A step of the path is definitely absent: isset=false / empty=true.
    Missing,
    /// The leaf container is an ArrayAccess object: dispatch the protocol.
    Aa(Zval, Zval),
    /// A raw leaf, pre-read (`None` = absent).
    Raw(Option<Zval>),
}

/// Opaque internal handle classes (PHP 8 resource-object wrappers): the
/// canonical predicate lives in php-types (shared with the pure builtins).
pub(crate) use php_types::is_opaque_handle_class;

/// The cli-server SAPI write-buffer size: sink output beyond this many bytes
/// has hit the socket, so headers count as "sent" under the web SAPI
/// (oracle-pinned by the WP-10 hs probe: 100 bytes → false, 5000 → true).
pub(crate) const WEB_SEND_THRESHOLD: usize = 4096;

pub(crate) fn host_builtin_out_param(name: &[u8]) -> Option<(&'static [u8], usize)> {
    HOST_OUT.iter().copied().find(|(h, _)| name.eq_ignore_ascii_case(h))
}

pub(crate) const HOST_OUT: &[(&[u8], usize)] = &[
        (b"preg_match", 2),
        (b"preg_match_all", 2),
        // `&$result` receives the parsed query-string array.
        (b"parse_str", 1),
        (b"mb_ereg", 2),
        (b"mb_eregi", 2),
        // `&$count` (number of replacements). Also in the plain host table for
        // dynamic string-callable dispatch, so the compiler must consult this
        // table FIRST (see `FnCompiler::call`).
        (b"preg_replace", 4),
        // `&$count` (number of replacements), like `preg_replace`.
        (b"preg_replace_callback", 4),
        // `&$would_block`: phpr is single-process, the lock never blocks (0).
        (b"flock", 2),
        // `&$pipes` receives the pipe resources of the spawned child.
        (b"proc_open", 2),
        // `&$old_signals` receives the previous signal mask (optional arg).
        (b"pcntl_sigprocmask", 2),
        // `&$result_code` receives the child's exit status (optional arg).
        (b"system", 1),
        (b"passthru", 1),
        // `exec`'s primary out-param is `&$output` (the array of lines); its
        // secondary `&$result_code` is in `host_builtin_out_param_second`.
        (b"exec", 1),
        // `&$next` receives the byte offset after the extracted grapheme run.
        (b"grapheme_extract", 4),
        // `&$percent` receives the similarity percentage.
        (b"similar_text", 2),
        // `&$count` (number of replacements) — WordPress's _deep_replace()
        // loops until it reads 0. Compiled calls WITHOUT the fourth argument
        // keep the registry fast path (see `FnCompiler::call`).
        (b"str_replace", 3),
        (b"str_ireplace", 3),
        // `&$callable_name` receives the display name (zend_get_callable_name).
        (b"is_callable", 2),
        // `&$error_code` (#2); `&$error_message` (#3) is the second out-param.
        (b"stream_socket_client", 1),
        // `&$filename` (#1); `&$line` (#2) is the second out-param. PHP fills
        // both whenever supplied — ""/0 before any output.
        (b"headers_sent", 0),
        // `&$image_info` receives the JPEG APPn segments (IPTC path). Calls
        // WITHOUT the second argument keep the registry fast path (see
        // `FnCompiler::call`, same filter as str_replace's `&$count`).
        (b"getimagesize", 1),
        (b"getimagesizefromstring", 1),
        // `&$rest_index` (#2, optional): index of the first non-option argv
        // entry. Calls without it take the plain host arm.
        (b"getopt", 2),
    ];

/// The *second* by-reference out-param index of a host builtin, when it has two
/// (only `exec`'s `&$result_code` at index 2). `None` for every single-out
/// builtin in [`host_builtin_out_param`].
pub(crate) fn host_builtin_out_param_second(name: &[u8]) -> Option<usize> {
    if name.eq_ignore_ascii_case(b"exec") || name.eq_ignore_ascii_case(b"stream_socket_client") {
        Some(2)
    } else if name.eq_ignore_ascii_case(b"headers_sent") {
        Some(1)
    } else {
        None
    }
}

/// Host builtins with **variadic** by-reference output parameters from a fixed
/// index onward (`sscanf`/`fscanf`'s `...&$vars` at index 2). The compiler emits
/// [`crate::bytecode::Op::CallHostBuiltinScanf`] for these; [`Vm::dispatch_host_builtin_scanf`]
/// produces the per-conversion slots and the VM assigns them.
pub(crate) fn host_builtin_scanf(name: &[u8]) -> Option<&'static [u8]> {
    HOST_SCANF.iter().copied().find(|h| name.eq_ignore_ascii_case(h))
}

pub(crate) const HOST_SCANF: &[&[u8]] = &[b"sscanf", b"fscanf"];

pub(crate) fn host_builtin_ref_first(name: &[u8]) -> Option<&'static [u8]> {
    HOST_REF.iter().copied().find(|h| name.eq_ignore_ascii_case(h))
}

pub(crate) const HOST_REF: &[&[u8]] = &[
        b"usort",
        b"uasort",
        b"uksort",
        b"array_walk",
        b"array_walk_recursive",
        // Array internal-pointer family (Session: array-pointer). Each takes the
        // array by reference (mutating/reading its internal cursor).
        b"reset",
        b"end",
        b"next",
        b"prev",
        b"current",
        b"key",
    ];

/// ASCII-case-insensitive byte-string equality — PHP resolves function names
/// case-insensitively in ASCII (mirrors the compiler's resolution).
fn name_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}


/// The method name for a dynamic call (`$obj->$m()`, `$cls::$m()`): PHP requires a
/// *string* — a non-string (object, array, int, …) raises the catchable
/// "Method name must be a string" Error (it never coerces or calls `__toString`).
fn dyn_method_name(v: &Zval) -> Result<Vec<u8>, PhpError> {
    match v.deref_clone() {
        Zval::Str(s) => Ok(s.as_bytes().to_vec()),
        _ => Err(PhpError::Error("Method name must be a string".to_string())),
    }
}


/// Fill `buf` with cryptographically secure random bytes from the OS entropy
/// source (`getrandom`). Backs `random_int`/`random_bytes`; a source failure
/// (never expected on a normal host) surfaces as a plain `Error`.
fn os_random_fill(buf: &mut [u8]) -> Result<(), PhpError> {
    getrandom::getrandom(buf)
        .map_err(|e| PhpError::Error(format!("Failed to generate random bytes: {e}")))
}

/// A fresh random `u64` from the OS CSPRNG (byte order is irrelevant for
/// uniform randomness).
fn os_random_u64() -> Result<u64, PhpError> {
    let mut buf = [0u8; 8];
    os_random_fill(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

/// Uniform unsigned integer in `[0, umax]` via OS randomness and rejection
/// sampling — a direct port of PHP's `php_random_range64`, which discards the
/// biased tail so every value in range is equally likely.
fn os_random_range64(umax: u64) -> Result<u64, PhpError> {
    let mut result = os_random_u64()?;
    // Full 64-bit range: no reduction needed.
    if umax == u64::MAX {
        return Ok(result);
    }
    let umax = umax + 1; // make the range inclusive of `max`
    // Powers of two are unbiased — just mask off the low bits.
    if umax & (umax - 1) == 0 {
        return Ok(result & (umax - 1));
    }
    // Ceiling below which `result % umax` is unbiased; reject anything above.
    let limit = u64::MAX - (u64::MAX % umax) - 1;
    while result > limit {
        result = os_random_u64()?;
    }
    Ok(result % umax)
}

/// Best-effort equality of two callable [`Zval`]s for `spl_autoload_unregister`
/// (step 57, Phase 3): a function-name string (case-insensitive) or the same
/// closure handle. Other callable shapes (`[$obj, 'm']`) compare unequal.
fn callable_eq(a: &Zval, b: &Zval) -> bool {
    match (a, b) {
        (Zval::Str(x), Zval::Str(y)) => x.as_bytes().eq_ignore_ascii_case(y.as_bytes()),
        (Zval::Closure(x), Zval::Closure(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// Whether the value graph reachable from `v` contains a cycle (an array or
/// object revisited on the current descent path) — `json_encode`'s
/// JSON_ERROR_RECURSION test. Distinct from a *shared* (DAG) node, which PHP
/// encodes fine: `visiting` is push/popped, not a global seen-set.
fn json_has_cycle(v: &Zval, visiting: &mut Vec<usize>) -> bool {
    match v {
        Zval::Ref(r) => {
            let inner = r.borrow();
            json_has_cycle(&inner, visiting)
        }
        Zval::Array(a) => {
            let addr = Rc::as_ptr(a) as usize;
            if visiting.contains(&addr) {
                return true;
            }
            visiting.push(addr);
            let hit = a.iter().any(|(_, val)| json_has_cycle(val, visiting));
            visiting.pop();
            hit
        }
        Zval::Object(o) => {
            let addr = Rc::as_ptr(o) as usize;
            if visiting.contains(&addr) {
                return true;
            }
            visiting.push(addr);
            let vals: Vec<Zval> = o.borrow().props.iter().map(|(_, val)| val.clone()).collect();
            let hit = vals.iter().any(|val| json_has_cycle(val, visiting));
            visiting.pop();
            hit
        }
        _ => false,
    }
}

/// Rewrite every compile-time class id in a freshly-compiled `eval`/`include`
/// module into the VM's GLOBAL id space (step 57, Phase 1c-2b), and offset its
/// `static $x` cell ids past the live `self.statics` range. `remap[local]` is the
/// global id for each eval-local class id; `static_base` is added to every
/// static-cell id. The module must still be owned (pre-leak) so its bytecode and
/// class metadata are mutable.
/// One cached include unit: the lowered `Program` (for `accumulate_seed`) and
/// the compiled, RELOCATED, leaked `Module`, valid for the exact VM state
/// captured by `fp` ([`Vm::unit_fp`]). `static_off` and `class_remap` are the
/// relocation baseline baked into the module's ops; a hit re-verifies both
/// against the live VM before reuse (mismatch ⇒ treated as a miss).
#[derive(Clone)]
struct CachedUnit {
    fp: u64,
    static_off: usize,
    class_remap: Vec<ClassId>,
    new_locals: Vec<usize>,
    program: Rc<Program>,
    module: &'static Module,
}

/// File identity for the unit cache: canonical path + mtime + size. An edited
/// file changes its key (and, through the load-event chain, the fingerprint of
/// every unit loaded after it).
#[derive(PartialEq, Eq, Hash, Clone)]
struct UnitKey {
    path: Vec<u8>,
    mtime: (u64, u32),
    size: u64,
}

/// Distinct fingerprints kept per file — a template re-included at shifting
/// static offsets would otherwise grow its slot unboundedly.
const UNIT_CACHE_WAYS: usize = 4;

thread_local! {
    /// The per-process (per-thread) unit cache. Entries hold leaked modules —
    /// memory the include path leaks TODAY on every load; caching makes that
    /// leak bounded by reusing the same module across requests.
    static UNIT_CACHE: RefCell<HashMap<UnitKey, Vec<CachedUnit>>> =
        RefCell::new(HashMap::default());
}

fn unit_key_for(path: &[u8], meta: &std::fs::Metadata) -> Option<UnitKey> {
    let d = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(UnitKey {
        path: path.to_vec(),
        mtime: (d.as_secs(), d.subsec_nanos()),
        size: meta.len(),
    })
}

fn fp_mix(chain: u64, tag: &[u8], data: &[u8]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    chain.hash(&mut h);
    tag.hash(&mut h);
    data.hash(&mut h);
    h.finish()
}

fn fp_mix_key(chain: u64, uk: &UnitKey) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    chain.hash(&mut h);
    uk.hash(&mut h);
    h.finish()
}

fn unit_cache_get(key: &UnitKey, fp: u64) -> Option<CachedUnit> {
    UNIT_CACHE.with(|c| c.borrow().get(key)?.iter().find(|cu| cu.fp == fp).cloned())
}

fn unit_cache_put(key: UnitKey, cu: CachedUnit) {
    UNIT_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        let slot = cache.entry(key).or_default();
        if let Some(pos) = slot.iter().position(|e| e.fp == cu.fp) {
            slot[pos] = cu;
        } else {
            if slot.len() >= UNIT_CACHE_WAYS {
                slot.remove(0);
            }
            slot.push(cu);
        }
    })
}

fn relocate_module_class_ids(module: &mut Module, remap: &[ClassId], static_base: usize) {
    // 1. Every function body (main, free functions, closures, and each class's
    //    methods / property-init thunk / const thunks / non-const static-prop
    //    thunks / property-hook get & set bodies).
    relocate_func(&mut module.main, remap, static_base);
    for f in &mut module.functions {
        relocate_func(f, remap, static_base);
    }
    for f in &mut module.closures {
        relocate_func(f, remap, static_base);
    }
    // Attributes on the unit's top-level constants (`#[Foo] const X = …`).
    for attrs in module.const_attributes.values_mut() {
        relocate_attrs(attrs, remap, static_base);
    }
    for cc in &mut module.classes {
        // 2. Class metadata: superclass and implemented interfaces.
        if let Some(p) = cc.parent.as_mut() {
            *p = remap[*p];
        }
        for i in cc.interfaces.iter_mut() {
            *i = remap[*i];
        }
        for m in &mut cc.methods {
            relocate_func(&mut m.func, remap, static_base);
        }
        // Abstract signatures have empty bodies but their param-default /
        // attribute thunks carry bytecode — the relocation rule applies to
        // every nested `Func`.
        for m in &mut cc.abstract_sigs {
            relocate_func(&mut m.func, remap, static_base);
        }
        if let Some(pi) = cc.prop_init.as_mut() {
            relocate_func(pi, remap, static_base);
        }
        for c in &mut cc.consts {
            relocate_func(&mut c.func, remap, static_base);
            relocate_attrs(&mut c.attributes, remap, static_base);
        }
        // Attribute thunks on the class itself and on its properties (method and
        // parameter attributes ride inside `relocate_func` above).
        relocate_attrs(&mut cc.attributes, remap, static_base);
        for attrs in cc.prop_attributes.values_mut() {
            relocate_attrs(attrs, remap, static_base);
        }
        for sp in &mut cc.static_props {
            if let StaticInit::Thunk(f) = &mut sp.init {
                relocate_func(f, remap, static_base);
            }
        }
        // The unified per-property metadata carries the declaring class id and the
        // property's hook funcs — both must be relocated into the global id space,
        // exactly like `parent` / `interfaces`.
        for pi in cc.prop_info.values_mut() {
            pi.declaring_class = remap[pi.declaring_class];
            if let Some(hooks) = pi.hooks.as_mut() {
                if let Some(g) = hooks.get.as_mut() {
                    relocate_func(g, remap, static_base);
                }
                if let Some(s) = hooks.set.as_mut() {
                    relocate_func(s, remap, static_base);
                }
            }
        }
    }
}

/// Apply [`relocate_module_class_ids`] to one function body: remap every
/// class-id-carrying op and offset every static-cell op.
/// A static variable's declared initial value (its folded constant; a
/// non-constant initialiser reads as NULL before the function has run).
fn static_var_init(init: &StaticInit) -> Zval {
    match init {
        StaticInit::Const(c) => c.to_zval(),
        StaticInit::Thunk(_) => Zval::Null,
    }
}

fn relocate_func(func: &mut Func, remap: &[ClassId], static_base: usize) {
    let base = static_base as u32;
    for op in func.ops.iter_mut() {
        match op {
            Op::Alloc { class }
            | Op::InstanceOf { class }
            | Op::DeclareClass { class }
            | Op::InvokeMethod { class, .. }
            | Op::ClassConst { class, .. }
            | Op::EnumCase { class, .. } => *class = remap[*class],
            Op::CatchMatch { types, .. } => {
                for t in types.iter_mut() {
                    *t = remap[*t];
                }
            }
            Op::StaticCall { target, .. }
            | Op::HookCall { target, .. }
            | Op::StaticCallArgs { target, .. }
            // `self::$m()` / `Class::$m()` (dynamic METHOD, compile-time class):
            // missed here, an include-unit's `self::` cid indexed whatever class
            // sat at the stale global slot (symfony's IpUtils::checkIp resolved
            // its `self::$method(...)` to a test fixture enum).
            | Op::StaticCallTargetDynamicMethod { target, .. }
            | Op::StaticCallTargetDynamicMethodArgs { target, .. }
            | Op::StaticPropGet { target, .. }
            | Op::StaticPropSet { target, .. }
            | Op::StaticPropOpSet { target, .. }
            | Op::StaticPropIncDec { target, .. } => {
                if let ClassTarget::Class(c) = target {
                    *c = remap[*c];
                }
            }
            Op::StaticGuard { id, .. } | Op::StaticStore { id } | Op::StaticAlias { id, .. } => {
                *id += base;
            }
            _ => {}
        }
    }
    // The function's `#[Attr]` thunks (declaration + per-parameter) are Funcs of
    // their own: a `new Attr(...)` resolved at compile time bakes a unit-local
    // `Op::Alloc { class }` that must relocate exactly like a method body —
    // missed, `newInstance()` builds whatever class sits at the stale global id
    // (the PHPUnit DataProvider→CoversTrait bug). Attribute thunks carry no
    // attributes themselves, so the recursion is one level deep.
    relocate_attrs(&mut func.attributes, remap, static_base);
    for pa in func.param_attributes.iter_mut() {
        relocate_attrs(pa, remap, static_base);
    }
    // Parameter-default thunks are Funcs too (`$level = Level::Debug` bakes an
    // Op::EnumCase/ClassConst with a unit-local class id) — same relocation
    // rule as attribute thunks.
    for pd in func.param_defaults.iter_mut().flatten() {
        relocate_func(pd, remap, static_base);
    }
    // `getStaticVariables()` metadata carries the same cell ids as the Op::Static*
    // stream, so relocate them identically (init values are folded constants,
    // needing no relocation).
    for sv in func.static_vars.iter_mut() {
        sv.id += base;
    }
}

/// [`relocate_func`] over both thunks of each attribute in a list.
fn relocate_attrs(attrs: &mut [crate::bytecode::CompiledAttribute], remap: &[ClassId], static_base: usize) {
    for a in attrs.iter_mut() {
        relocate_func(&mut a.new_thunk, remap, static_base);
        relocate_func(&mut a.args_thunk, remap, static_base);
    }
}

/// The `[parameter]` pseudo-property descriptors for a function/closure body
/// (CLO, D-18.9): each parameter name (without `$`) and whether it is optional
/// (no default ⇒ required). Mirrors `eval::closure_params_for`.
pub(super) fn closure_params(func: &Func) -> Vec<ClosureParam> {
    func.param_names
        .iter()
        .zip(func.param_required.iter())
        .map(|(name, &req)| ClosureParam { name: name.clone(), optional: !req })
        .collect()
}

/// Drill through `keys` from `cell` (following references, auto-vivifying and
/// copy-on-writing each level), then apply `last` at the leaf. Recursion (not a
/// reassigned `&mut` in a loop) keeps the nested borrows well-formed.
fn path_apply(cell: &mut Zval, keys: &[Zval], last: Last, diags: &mut Diags, dropped: &mut Vec<Zval>, aa: &mut Option<PathAa>) -> Result<Zval, PhpError> {
    if let Zval::Ref(rc) = cell {
        let mut inner = rc.borrow_mut();
        return path_apply(&mut inner, keys, last, diags, dropped, aa);
    }
    match keys.split_first() {
        Some((k, rest)) => {
            // Writing *through* a string offset (`$s[0][1] = …`) is the PHP error.
            if matches!(cell, Zval::Str(_)) {
                return Err(PhpError::Error("Cannot use string offset as an array".to_string()));
            }
            if matches!(cell, Zval::Object(_)) {
                // Mid-path object: park an ArrayAccess descend for the caller.
                // The expression's provisional value is the assigned one (the
                // deeper drain overrides it for the compound leaves).
                let provisional = match &last {
                    Last::Set { value, .. } | Last::Append { value } => value.clone(),
                    _ => Zval::Null,
                };
                *aa = Some(PathAa::Descend {
                    obj: cell.clone(),
                    key: k.clone(),
                    rest: rest.to_vec(),
                    last: Box::new(last),
                });
                return Ok(provisional);
            }
            ensure_array(cell)?;
            let Zval::Array(rc) = cell else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let key = coerce_key_diag(k, diags)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if !arr.contains_key(&key) {
                arr.insert(key.clone(), Zval::Null);
            }
            let child = arr.get_mut(&key).expect("just inserted");
            path_apply(child, rest, last, diags, dropped, aa)
        }
        None => apply_last(cell, last, diags, dropped, aa),
    }
}

/// Apply the leaf step to the parent cell (which must hold the target array).
fn apply_last(parent: &mut Zval, last: Last, diags: &mut Diags, dropped: &mut Vec<Zval>, aa: &mut Option<PathAa>) -> Result<Zval, PhpError> {
    // A string parent takes the byte-offset write path (`$s[0] = 'X'`), whose
    // expression value is the written byte; the other leaf ops are the PHP
    // errors for string offsets.
    if matches!(parent, Zval::Str(_)) {
        return match last {
            Last::Set { key, value } => string_offset_write(parent, &key, &value, diags),
            Last::Append { .. } => {
                Err(PhpError::Error("[] operator not supported for strings".to_string()))
            }
            Last::OpSet { .. } => Err(PhpError::Error(
                "Cannot use assign-op operators with string offsets".to_string(),
            )),
            Last::IncDec { .. } => Err(PhpError::Error(
                "Cannot increment/decrement string offsets".to_string(),
            )),
        };
    }
    // A leaf on an OBJECT defers to the caller's ArrayAccess dispatch:
    // offsetSet for Set/Append, offsetGet→op→offsetSet for the compound and
    // inc/dec leaves (zend runs them without a by-ref offsetGet too).
    if matches!(parent, Zval::Object(_)) {
        match last {
            Last::Set { key, value } => {
                *aa = Some(PathAa::Write(crate::vm::arrays::AaWrite { obj: parent.clone(), key: Some(key), value: value.clone() }));
                return Ok(value);
            }
            Last::Append { value } => {
                *aa = Some(PathAa::Write(crate::vm::arrays::AaWrite { obj: parent.clone(), key: None, value: value.clone() }));
                return Ok(value);
            }
            Last::OpSet { key, op, rhs } => {
                *aa = Some(PathAa::Op { obj: parent.clone(), key, op, rhs });
                return Ok(Zval::Null);
            }
            Last::IncDec { key, inc, pre } => {
                *aa = Some(PathAa::IncDec { obj: parent.clone(), key, inc, pre });
                return Ok(Zval::Null);
            }
        }
    }
    ensure_array(parent)?;
    let Zval::Array(rc) = parent else { unreachable!("ensured array") };
    let arr = Rc::make_mut(rc);
    match last {
        Last::Set { key, value } => {
            let k = coerce_key_diag(&key, diags)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            // Write *through* an existing reference element (REF-4) so an alias
            // sees the update; otherwise overwrite / insert.
            match arr.get_mut(&k) {
                Some(slot) => {
                    // The displaced element is handed back via `dropped` so the
                    // caller can note it (an object it held may now be unreachable).
                    dropped.push(store_slot(slot, value.clone()));
                }
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
            let k = coerce_key_diag(&key, diags)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            let old = arr.get(&k).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
            let result = apply_binop(op, &old, &rhs, diags)?;
            // Write through an existing reference element (REF-4).
            match arr.get_mut(&k) {
                Some(slot) => {
                    dropped.push(store_slot(slot, result.clone()));
                }
                None => arr.insert(k, result.clone()),
            }
            Ok(result)
        }
        Last::IncDec { key, inc, pre } => {
            let k = coerce_key_diag(&key, diags)
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
/// (`Local`), in the global/script frame (`Global`), or a data superglobal —
/// Symfony's NativeSessionStorage aliases its bags into `$_SESSION` by
/// reference, so a superglobal IS a legitimate `BindRef` base/target.
fn ref_base_mut<'f>(
    frames: &'f mut [Frame<'_>],
    superglobals: &'f mut [Zval; 8],
    top: usize,
    base: DimBase,
) -> &'f mut Zval {
    match base {
        DimBase::Local(s) => &mut frames[top].slots[s as usize],
        DimBase::Global(s) => {
            // The bottom frame can be smaller than the cross-unit global
            // registry (a synthetic shutdown-phase frame, or a main whose
            // unit declared fewer top-level names than later includes added):
            // grow it — the registry indices are stable, missing cells are
            // simply still-undefined globals.
            if s as usize >= frames[0].slots.len() {
                frames[0].slots.resize_with(s as usize + 1, || Zval::Undef);
            }
            &mut frames[0].slots[s as usize]
        }
        DimBase::Superglobal(i) => &mut superglobals[i as usize],
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
    superglobals: &'f mut [Zval],
    top: usize,
    base: FieldBase,
) -> Result<&'f mut Zval, PhpError> {
    Ok(match base {
        FieldBase::Local(s) => &mut frames[top].slots[s as usize],
        FieldBase::Global(s) => &mut frames[0].slots[s as usize],
        FieldBase::Superglobal(i) => &mut superglobals[i as usize],
        FieldBase::This => frames[top].this.as_mut().ok_or_else(|| {
            PhpError::Error("Using $this when not in object context".to_string())
        })?,
    })
}

fn field_cell(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
    fs: FieldScope,
) -> Rc<RefCell<Zval>> {
    let Some((first, rest)) = steps.split_first() else {
        return make_cell(target);
    };
    if let Zval::Ref(rc) = target {
        let inner = &mut *rc.borrow_mut();
        return field_cell(inner, steps, keys, fs);
    }
    match first {
        FieldStep::Prop(_) | FieldStep::PropDyn => {
            // `&$o->prop` / `&$o->$n` (Session A / step 51): promote the property to
            // a shared cell. A non-object yields a detached cell (PHP warns).
            let owned;
            let name: &[u8] = match first {
                FieldStep::Prop(n) => n,
                _ => {
                    owned = prop_dyn_name(keys, &mut Diags::new());
                    &owned
                }
            };
            let Zval::Object(o) = target else {
                return Rc::new(RefCell::new(Zval::Null));
            };
            let mut obj = o.borrow_mut();
            let key = fs.prop_key(obj.class_id as usize, name);
            let key = key.as_ref();
            if !obj.props.contains(key) {
                obj.props.set(key, Zval::Null);
            }
            let child = obj.props.get_mut(key).expect("property present after insert");
            field_cell(child, rest, keys, fs)
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
                Some(child) => field_cell(child, rest, keys, fs),
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
            field_cell(child, rest, keys, fs)
        }
    }
}

/// Whether `PHPR_GC_VERIFY` is set: a diagnostic mode in which [`Vm::gc_sweep`]
/// also runs the authoritative full scan and reports any collectable object the
/// candidate buffer failed to surface (an un-hooked reference-drop site). Off in
/// normal runs (the candidate buffer alone drives the sweep). Read once.
fn gc_verify_enabled() -> bool {
    use std::sync::OnceLock;
    static V: OnceLock<bool> = OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PHPR_GC_VERIFY").is_some())
}

/// Write `v` into a local cell. A reference slot writes *through* its shared
/// cell (so aliases see the update); a plain slot is overwritten. This mirrors
/// the tree-walker's write-through discipline (`Zval::Ref`, D-R3).
/// Write `v` into `cell` (following a `Ref` to its shared cell), returning the
/// value it displaced. Callers that care about destruction timing pass the
/// returned old value to [`Vm::gc_note`] so a now-unreachable object enters the
/// possible-roots buffer; callers that don't simply drop it (unchanged
/// behaviour). The return value is deliberately not `#[must_use]`.
fn store_slot(cell: &mut Zval, v: Zval) -> Zval {
    if let Zval::Ref(r) = cell {
        std::mem::replace(&mut *r.borrow_mut(), v)
    } else {
        std::mem::replace(cell, v)
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

/// If `v` is (or references) an operator-overloaded arbitrary-precision object
/// (`BcMath\Number` or `GMP`), return a clonable object Zval plus whether it is a
/// `GMP` (which overloads bitwise ops and brands operand errors, unlike Number).
fn overload_receiver(v: &Zval) -> Option<(Zval, bool)> {
    let o = deref_object(v)?;
    let is_gmp = {
        let b = o.borrow();
        let cn = b.class_name.as_bytes();
        if cn == b"BcMath\\Number" {
            false
        } else if cn == b"GMP" {
            true
        } else {
            return None;
        }
    };
    Some((Zval::Object(o), is_gmp))
}

/// Just the receiver Zval (for ++/-- which don't need the class kind).
fn number_receiver(v: &Zval) -> Option<Zval> {
    overload_receiver(v).map(|(z, _)| z)
}

/// The `__op` opcode for a `BinOp`: 0-5 arithmetic (both classes), 6-8 bitwise
/// (GMP only). `None` for everything else.
fn overload_binop_opcode(op: BinOp, is_gmp: bool) -> Option<i64> {
    use BinOp::*;
    match op {
        Add => Some(0),
        Sub => Some(1),
        Mul => Some(2),
        Div => Some(3),
        Mod => Some(4),
        Pow => Some(5),
        BitAnd if is_gmp => Some(6),
        BitOr if is_gmp => Some(7),
        BitXor if is_gmp => Some(8),
        Shl if is_gmp => Some(9),
        Shr if is_gmp => Some(10),
        _ => None,
    }
}

/// True if `v` is an overloaded object or a scalar that `do_operation` coerces
/// (int/float/any string — a malformed string surfaces `__op`'s error).
fn operand_arith_ok(v: &Zval) -> bool {
    match v {
        Zval::Object(_) => overload_receiver(v).is_some(),
        Zval::Long(_) | Zval::Double(_) | Zval::Str(_) => true,
        Zval::Ref(c) => operand_arith_ok(&c.borrow()),
        _ => false,
    }
}

/// Like [`operand_arith_ok`] but for comparison, where a malformed string is
/// *uncomparable* (falls back to the engine) rather than an error.
fn operand_cmp_ok(v: &Zval) -> bool {
    match v {
        Zval::Object(_) => overload_receiver(v).is_some(),
        Zval::Long(_) | Zval::Double(_) => true,
        Zval::Str(s) => bc_str_wellformed(s.as_bytes()),
        Zval::Ref(c) => operand_cmp_ok(&c.borrow()),
        _ => false,
    }
}

/// Operands that are UNCOMPARABLE with a `BcMath\Number` (the compare handler
/// yields no ordering): a non-numeric string, an array, a resource, or an object
/// of another class. null/bool keep the engine's default ordering.
fn operand_uncomparable(v: &Zval) -> bool {
    match v {
        Zval::Str(s) => !bc_str_wellformed(s.as_bytes()),
        Zval::Array(_) | Zval::Resource(_) => true,
        Zval::Object(_) => overload_receiver(v).is_none(),
        Zval::Ref(c) => operand_uncomparable(&c.borrow()),
        _ => false,
    }
}

/// `[+-]?[0-9]*(\.[0-9]*)?` — libbcmath's number grammar.
fn bc_str_wellformed(s: &[u8]) -> bool {
    let mut i = 0;
    if matches!(s.first(), Some(b'+' | b'-')) {
        i += 1;
    }
    let mut seen_dot = false;
    while i < s.len() {
        match s[i] {
            b'0'..=b'9' => {}
            b'.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
        i += 1;
    }
    true
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
        // `(array)`: an array passes through, null/unset → empty, an object's
        // stored properties copy over verbatim — *raw* storage keys, so a private
        // surfaces as its mangled `\0Class\0name`, exactly PHP — and a scalar
        // wraps into a single `[0 => v]` element (mirrors `eval::array_cast`).
        CastKind::Array => match a.deref_clone() {
            arr @ Zval::Array(_) => arr,
            Zval::Null | Zval::Undef => Zval::Array(Rc::new(PhpArray::new())),
            Zval::Object(o) => {
                let ob = o.borrow();
                let mut arr = PhpArray::new();
                for (key, val) in ob.props.iter() {
                    // An uninitialized typed property is absent from the cast.
                    if matches!(val, Zval::Undef) {
                        continue;
                    }
                    // Zend keys: public plain, private `\0Class\0name` (the raw
                    // storage key already), protected `\0*\0name` — phpr stores
                    // protected props unmangled, so the marker is added here.
                    let (disp, vis) = php_types::unmangle_prop_key(key, &ob.info);
                    if matches!(vis, php_types::PropVis::Protected) {
                        let mut k = b"\0*\0".to_vec();
                        k.extend_from_slice(disp);
                        arr.insert(Key::from_bytes(&k), val.deref_clone());
                    } else {
                        arr.insert(Key::from_bytes(key), val.deref_clone());
                    }
                }
                drop(ob);
                Zval::Array(Rc::new(arr))
            }
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
        Zval::Null | Zval::Undef | Zval::ArgPlace(_) => "NULL".to_string(),
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
        Zval::WeakHandle(_) => "of type WeakReference".to_string(),
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

    /// Regression guard for the property-metadata consolidation: the unified
    /// compile-time `prop_info` table resolves visibility / declaring-class /
    /// readonly / type / hooks with the correct parent-flattening and shadowing.
    /// Exercises inheritance, a redeclaration that shadows both the inherited type
    /// and visibility, readonly, and backed/virtual hooks.
    #[test]
    fn prop_info_flattening_and_shadowing() {
        use crate::hir::Visibility;
        let src = br#"<?php
        class Base {
            public int $a = 1;
            protected readonly int $r;
            private string $p = "x";
            public int $hooked { get => $this->a; }
            public int $virt { get => 1; }
        }
        class Child extends Base {
            public float $c = 2.0;
            private int $p = 5;
        }
        abstract class AbsBase {
            public readonly string $name;
        }
        class Concrete extends AbsBase {
            public int $extra = 0;
        }
        "#;
        let program = lower_source(b"test.php", src).expect("lower");
        let reg = Registry::new();
        let module = compile_program(&program, &reg).expect("compile");
        let classes: Vec<&super::CompiledClass> = module.classes.iter().collect();
        let cid = |n: &[u8]| classes.iter().position(|c| c.name.as_ref() == n).expect("class");
        let pi = |class: usize, name: &[u8]| super::prop_info(&classes, class, name).expect("prop");

        let base = cid(b"Base");
        let child = cid(b"Child");
        let concrete = cid(b"Concrete");
        let absbase = cid(b"AbsBase");

        // Inherited property keeps the ancestor as declaring class.
        let a = pi(child, b"a");
        assert_eq!(a.declaring_class, base);
        assert_eq!(a.visibility, Visibility::Public);
        assert_eq!(a.type_hint.as_ref().map(|h| h.display_name()), Some("int".into()));

        // Redeclaration in Child shadows BOTH the inherited type (string->int) and
        // visibility, and moves the declaring class to Child.
        let p_base = pi(base, b"p");
        assert_eq!(p_base.declaring_class, base);
        assert_eq!(p_base.type_hint.as_ref().map(|h| h.display_name()), Some("string".into()));
        let p_child = pi(child, b"p");
        assert_eq!(p_child.declaring_class, child);
        assert_eq!(p_child.visibility, Visibility::Private);
        assert_eq!(p_child.type_hint.as_ref().map(|h| h.display_name()), Some("int".into()));

        // readonly flag + protected visibility carried in one entry.
        let r = pi(base, b"r");
        assert!(r.readonly);
        assert_eq!(r.visibility, Visibility::Protected);

        // readonly inherited through an abstract class.
        let name = pi(concrete, b"name");
        assert!(name.readonly);
        assert_eq!(name.declaring_class, absbase);

        // Hooks present (backed) and virtual; a non-hooked prop has none.
        assert!(pi(base, b"hooked").hooks.is_some());
        assert!(pi(base, b"virt").hooks.is_some());
        assert!(pi(base, b"a").hooks.is_none());

        // A dynamic / undeclared name resolves to nothing.
        assert!(super::prop_info(&classes, child, b"not_a_prop").is_none());
    }

    /// A property hook inherited by a subclass runs in its *declaring* class's
    /// scope, so it can reach that class's private members even when invoked on a
    /// subclass instance (regression: previously the hook ran in the object's class
    /// scope and a `$this->private` access from the inherited hook fatally failed
    /// with "Cannot access private property").
    #[test]
    fn inherited_hook_reaches_declaring_class_private() {
        let src = br#"<?php
        class Temp {
            private int $c = 0;
            public int $f { get => $this->c + 100; set => $this->c = $value; }
        }
        class Sub extends Temp {}
        $s = new Sub();
        $s->f = 5;
        echo $s->f;
        "#;
        assert_eq!(vm_stdout(src), b"105");
    }

    /// `foreach` over a plain object iterates the properties visible from the loop's
    /// scope (public from outside, all from inside the class); a `&$v` loop binds by
    /// reference so writes mutate the property.
    #[test]
    fn foreach_over_plain_object() {
        let src = br#"<?php
        class C { public $a=1; public $b=2; private $p=3;
          function inside(){ $o=''; foreach($this as $k=>$v) $o.="$k=$v;"; return $o; } }
        $c=new C();
        $s=''; foreach($c as $k=>$v) $s.="$k=$v;"; echo $s;
        echo "|", $c->inside();
        foreach($c as &$v){ $v+=10; } unset($v);
        echo "|", $c->a, ",", $c->b;
        "#;
        assert_eq!(vm_stdout(src), b"a=1;b=2;|a=1;b=2;p=3;|11,12");
    }

    /// `ob_start($cb, $chunk_size)`: a chunk_size buffer auto-flushes through its
    /// handler the moment a write reaches the threshold (phase START=1 on the first
    /// flush, FINAL=8 on the closing one); the handler's return replaces the output.
    /// The wrapping callback records the phase so both the chunking and the phase
    /// bitmask are exercised with builtin-free PHP.
    #[test]
    fn ob_start_chunk_size_and_phase() {
        let src = br#"<?php
        $cb = function($s, $p) { return "[$p:$s]"; };
        ob_start($cb, 4);
        echo "ab";   // 2 < 4: buffered
        echo "cde";  // 5 >= 4: flush, phase START(1)
        echo "f";    // 1 < 4: buffered
        ob_end_flush(); // flush, phase FINAL(8)
        "#;
        // First flush START=1 over "abcde", final FINAL=8 over "f".
        assert_eq!(vm_stdout(src), b"[1:abcde][8:f]");
    }

    /// A plain `ob_start` + `ob_end_flush` (no chunking, never pre-flushed) invokes
    /// the handler once with phase START|FINAL = 9.
    #[test]
    fn ob_start_single_call_phase_is_start_final() {
        let src = br#"<?php
        ob_start(function($s, $p) { return "<$p>$s"; });
        echo "hi";
        ob_end_flush();
        "#;
        assert_eq!(vm_stdout(src), b"<9>hi");
    }

    /// `call_user_func` forwards arguments by value, so a callee by-reference
    /// parameter that receives a value raises the E_WARNING "must be passed by
    /// reference, value given" (and is then passed by value). The signature is
    /// resolved at the trampoline against the named callee.
    #[test]
    fn call_user_func_by_ref_param_warns() {
        let out = vm_outcome(b"<?php function f(&$x){} call_user_func('f', 1); echo 'done';");
        assert_eq!(
            out.rendered,
            b"\nWarning: f(): Argument #1 ($x) must be passed by reference, value given in test.php on line 1\ndone".to_vec()
        );
        // stdout carries rendered diagnostics too (diag-through-ob fix).
        assert_eq!(out.stdout, out.rendered);
    }

    /// `call_user_func_array` with a plain (non-reference) element at a by-ref
    /// position warns just like `call_user_func`.
    #[test]
    fn call_user_func_array_value_element_warns() {
        let out = vm_outcome(b"<?php function f(&$x){} call_user_func_array('f', [1]); echo 'done';");
        assert_eq!(
            out.rendered,
            b"\nWarning: f(): Argument #1 ($x) must be passed by reference, value given in test.php on line 1\ndone".to_vec()
        );
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
    fn logical_xor_word_operator() {
        // `xor` evaluates both operands and yields a bool: exactly one truthy.
        assert_eq!(vm_stdout(b"<?php echo (1 xor 0) ? 't' : 'f';"), b"t");
        assert_eq!(vm_stdout(b"<?php echo (1 xor 1) ? 't' : 'f';"), b"f");
        assert_eq!(vm_stdout(b"<?php echo (0 xor 0) ? 't' : 'f';"), b"f");
        // truthiness coercion, not raw inequality (2 and 1 are both truthy → false).
        assert_eq!(vm_stdout(b"<?php echo (2 xor 1) ? 't' : 'f';"), b"f");
        // the result is a real bool (=== true), not a truthy int.
        assert_eq!(vm_stdout(b"<?php echo (true xor false) === true ? 'Y' : 'N';"), b"Y");
    }

    #[test]
    fn json_decode_assoc_and_stdclass() {
        // assoc=true: objects → arrays, nested arrays preserved.
        assert_eq!(
            vm_stdout(b"<?php $v=json_decode('{\"a\":1,\"b\":[2,3]}', true); echo $v['a'], '|', $v['b'][1];"),
            b"1|3"
        );
        assert_eq!(
            vm_stdout(b"<?php $v=json_decode('[1,\"x\",true,null]', true); echo $v[0], $v[1], ($v[2]?'T':'F'), ($v[3]===null?'N':'?');"),
            b"1xTN"
        );
        // default (assoc=false): objects → stdClass (get_class is a host builtin).
        assert_eq!(
            vm_stdout(b"<?php $o=json_decode('{\"a\":1,\"b\":\"z\"}'); echo get_class($o), '|', $o->a, $o->b;"),
            b"stdClass|1z"
        );
        // scalars keep their JSON type (=== checks discriminate string vs double).
        assert_eq!(vm_stdout(b"<?php $v=json_decode('\"hi\"'); echo $v, '|', ($v==='hi'?'S':'?');"), b"hi|S");
        assert_eq!(vm_stdout(b"<?php $v=json_decode('3.14'); echo $v, '|', ($v===3.14?'D':'?');"), b"3.14|D");
        // parse error and literal null both yield null.
        assert_eq!(vm_stdout(b"<?php echo json_decode('null')===null?'N':'?';"), b"N");
        assert_eq!(vm_stdout(b"<?php echo json_decode('not json')===null?'N':'?';"), b"N");
    }

    #[test]
    fn mb_split_and_regex_state() {
        // mb_split keeps empty fields; positive limit caps piece count.
        assert_eq!(
            vm_stdout(b"<?php $p=mb_split('\\\\s+', 'a  b   c'); echo $p[0],$p[1],$p[2];"),
            b"abc"
        );
        assert_eq!(
            vm_stdout(b"<?php $p=mb_split(',', 'a,b,c', 2); echo $p[0],'/',$p[1];"),
            b"a/b,c"
        );
        // mb_regex_encoding getter default + setter returns true; set_options getter
        // default + setter returns previous, and the change is observed by the getter.
        assert_eq!(vm_stdout(b"<?php echo mb_regex_encoding();"), b"UTF-8");
        assert_eq!(vm_stdout(b"<?php echo mb_regex_encoding('UTF-8')?'Y':'N';"), b"Y");
        assert_eq!(vm_stdout(b"<?php echo mb_regex_set_options();"), b"pr");
        assert_eq!(
            vm_stdout(b"<?php echo mb_regex_set_options('i'), '|', mb_regex_set_options();"),
            b"pr|i"
        );
    }

    #[test]
    fn goto_out_of_try_runs_finally() {
        // Jumping out of a try body runs its finally first, then lands on the
        // target (skipping the code between the try and the label). Corpus
        // finally_goto: "tfD".
        assert_eq!(
            vm_stdout(b"<?php try { echo 't'; goto done; } finally { echo 'f'; } echo 'X'; done: echo 'D';"),
            b"tfD"
        );
    }

    #[test]
    fn goto_back_to_label_before_try_runs_finally() {
        // Corpus finally_goto_005: the label is before the try; the goto inside the
        // body jumps out, so finally runs and its `return` ends the function.
        assert_eq!(
            vm_stdout(b"<?php function f(){ label: try { goto label; } finally { echo 'success'; return; } } f();"),
            b"success"
        );
    }

    #[test]
    fn goto_into_transparent_block_jumps() {
        // WP-18: jumping *into* an if/try/plain block is valid PHP and, on the
        // flat bytecode, just a patched `Jump` — the label's code runs.
        // (Entering a loop/switch body stays the LOWERING's compile fatal,
        // "'goto' into loop or switch statement is disallowed".)
        assert_eq!(vm_stdout(b"<?php goto a; if (true) { a: echo 'x'; }"), b"x");
    }

    #[test]
    fn undefined_function_is_runtime_fatal_after_output() {
        // PHP defers "Call to undefined function" to run time: the preceding output
        // is flushed, then a catchable Error is raised at the call site.
        let o = vm_outcome(b"<?php echo 'a'; nope();");
        assert_eq!(o.stdout, b"a");
        match o.fatal {
            Some(PhpError::Error(m)) => {
                assert!(m.contains("Call to undefined function nope"), "message was: {m}")
            }
            other => panic!("expected undefined-function Error, got {other:?}"),
        }
    }

    #[test]
    fn mb_ereg_match_replace_and_search() {
        // mb_ereg with a by-ref $regs out-param: returns bool, fills $m.
        assert_eq!(
            vm_stdout(b"<?php $ok=mb_ereg('(\\\\d+)', 'a42b', $m); echo ($ok?'Y':'N'),'/',$m[1];"),
            b"Y/42"
        );
        // mb_ereg_replace honours backrefs; mb_eregi_replace is case-insensitive.
        assert_eq!(vm_stdout(b"<?php echo mb_ereg_replace('(\\\\w)(\\\\w)', '\\\\2\\\\1', 'abcd');"), b"badc");
        assert_eq!(vm_stdout(b"<?php echo mb_eregi_replace('a', 'X', 'AaA');"), b"XXX");
        // mb_ereg_replace_callback: the closure transforms each match's $regs.
        assert_eq!(
            vm_stdout(b"<?php echo mb_ereg_replace_callback('\\\\d+', fn($m) => $m[0] * 2, 'a5b10');"),
            b"a10b20"
        );
        // mb_ereg_match is anchored at the start.
        assert_eq!(vm_stdout(b"<?php echo mb_ereg_match('\\\\d+', '123abc')?'Y':'N';"), b"Y");
        assert_eq!(vm_stdout(b"<?php echo mb_ereg_match('a', 'xa')?'Y':'N';"), b"N");
        // stateful cursor: search_pos walks each match as [pos, len], then false.
        assert_eq!(
            vm_stdout(b"<?php mb_ereg_search_init('a1b2c3', '\\\\d'); $s=''; while(($r=mb_ereg_search_pos())!==false){ $s.=$r[0].':'.$r[1].','; } echo $s;"),
            b"1:1,3:1,5:1,"
        );
        // search() advances; getpos returns the byte offset after the match.
        assert_eq!(
            vm_stdout(b"<?php mb_ereg_search_init('a1b2c3', '\\\\d'); echo mb_ereg_search()?'Y':'N', mb_ereg_search_getpos();"),
            b"Y2"
        );
    }

    #[test]
    fn sscanf_byref_and_array_mode() {
        // by-ref mode: assigns each conversion and returns the success count.
        assert_eq!(
            vm_stdout(b"<?php $n=sscanf('12 f0 0x1A 777','%d %x %x %o',$a,$b,$c,$d); echo \"$n|$a|$b|$c|$d\";"),
            b"4|12|240|26|511"
        );
        // failed conversion: count reflects successes, the out vars become null.
        assert_eq!(
            vm_stdout(b"<?php $n=sscanf('only','%d %s',$a,$b); echo $n,'/',($a===null?'N':'?'),($b===null?'N':'?');"),
            b"0/NN"
        );
        // array mode (no out vars): returns the parsed array.
        assert_eq!(
            vm_stdout(b"<?php $r=sscanf('age:42 pi:3.14 name:bob','age:%d pi:%f name:%s'); echo $r[0],'/',$r[2];"),
            b"42/bob"
        );
    }

    #[test]
    fn goto_forward_backward_and_out_of_loop() {
        // Forward jump skips intervening statements.
        assert_eq!(vm_stdout(b"<?php goto a; echo 'skip'; a: echo 'A';"), b"A");
        // Backward jump is a hand-rolled loop.
        assert_eq!(vm_stdout(b"<?php $i=0; loop: echo $i; $i++; if ($i<3) goto loop;"), b"012");
        // Jump out of a (nested) loop lands on the label past it.
        assert_eq!(
            vm_stdout(b"<?php for ($i=0;$i<5;$i++) { if ($i==2) goto done; echo $i; } done: echo '|';"),
            b"01|"
        );
    }

    #[test]
    fn goto_within_finally_is_allowed() {
        // A goto whose label is in the same finally body does not cross it.
        assert_eq!(
            vm_stdout(b"<?php function f(){ try {} finally { goto t; t: } } f(); echo 'ok';"),
            b"ok"
        );
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
    fn unknown_function_is_runtime_fatal_not_compile_error() {
        // An unknown name compiles (PHP defers "Call to undefined function" to run
        // time, where it is a catchable Error — the function may be declared later).
        let program = lower_source(b"test.php", b"<?php echo no_such_fn();").expect("lower");
        let reg = fake_registry();
        let module = compile_program(&program, &reg).expect("compiles");
        let out = run_module(&module, &reg);
        match out.fatal {
            Some(PhpError::Error(m)) => {
                assert!(m.contains("Call to undefined function no_such_fn"), "message was: {m}")
            }
            other => panic!("expected undefined-function Error, got {other:?}"),
        }
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
        // stdout carries rendered diagnostics too (diag-through-ob fix).
        assert_eq!(out.stdout, out.rendered);
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
        assert_eq!(out.stdout, out.rendered);
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
        assert_eq!(out.stdout, out.rendered);
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
        assert_eq!(out.stdout, out.rendered);
    }

    #[test]
    fn error_suppression_silences_warnings_only() {
        // `@$x` yields NULL with the undefined-variable warning dropped.
        let out = vm_outcome(b"<?php echo @$x === null ? 'N' : '?';");
        assert_eq!(out.stdout, b"N");
        assert!(!String::from_utf8_lossy(&out.rendered).contains("Undefined"));
        // Control: without `@`, the warning renders.
        let ctl = vm_outcome(b"<?php echo $y === null ? 'N' : '?';");
        assert!(String::from_utf8_lossy(&ctl.rendered).contains("Undefined variable $y"));
    }

    #[test]
    fn error_suppression_does_not_swallow_throwable() {
        // `@` silences warnings, not engine errors: a DivisionByZeroError from
        // `@(1 % 0)` still propagates to the catch, and suppression is cleared.
        assert_eq!(
            vm_stdout(
                b"<?php try { echo @(1 % 0); } catch (\\DivisionByZeroError $e) { echo 'caught'; } \
                  echo $z;"
            ),
            b"caught\nWarning: Undefined variable $z in test.php on line 1\n"
        );
        // The trailing `echo $z` warns normally — suppression did not leak past
        // the abandoned `@`.
        let out = vm_outcome(
            b"<?php try { echo @(1 % 0); } catch (\\DivisionByZeroError $e) {} echo $z;",
        );
        assert!(String::from_utf8_lossy(&out.rendered).contains("Undefined variable $z"));
    }

    #[test]
    fn exit_sets_code_and_diverges() {
        // int → exit code, no extra output; bare exit / die() → 0.
        let o = vm_outcome(b"<?php echo 'a'; exit(5); echo 'b';");
        assert_eq!(o.stdout, b"a");
        assert_eq!(o.exit_code, Some(5));
        assert_eq!(vm_outcome(b"<?php echo 'a'; exit;").exit_code, Some(0));
        // status wraps to a byte (256 → 0, -1 → 255).
        assert_eq!(vm_outcome(b"<?php exit(256);").exit_code, Some(0));
        assert_eq!(vm_outcome(b"<?php exit(-1);").exit_code, Some(255));
    }

    #[test]
    fn exit_string_prints_and_is_expression() {
        // A string status is printed (code 0); `exit`/`die` are expressions.
        let o = vm_outcome(b"<?php false or die('DEAD'); echo 'after';");
        assert_eq!(o.stdout, b"DEAD");
        assert_eq!(o.exit_code, Some(0));
    }

    #[test]
    fn exit_does_not_run_finally() {
        // Unlike return/throw, `exit` propagates uncatchably — finally never runs.
        let o = vm_outcome(b"<?php try { echo 't'; exit('X'); } finally { echo 'f'; }");
        assert_eq!(o.stdout, b"tX");
        assert_eq!(o.exit_code, Some(0));
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
        // Named arguments on a (non-user) builtin call are still a compile-time gap,
        // surfaced as `VmRunError::Unsupported`, not a fatal. (An *unknown* function
        // name is no longer rejected — it defers to a run-time Error.)
        let reg = Registry::new();
        let err = super::run_source_with(b"test.php", b"<?php array_map(callback: 'x', array: []);", &reg)
            .expect_err("vm should reject named-arg builtin call");
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
            b"\nWarning: Undefined property: C::$missing in test.php on line 1\nnull"
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
        // getMessage() names the call site only; the " and defined in F:L" suffix is
        // appended at render time from the throwable's (definition) file/line.
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeErrorAt { msg, line, .. })
                if msg == "f(): Argument #1 ($x) must be of type int, string given, \
                         called in test.php on line 1"
                    && *line == 1
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
        assert!(matches!(&out.fatal, Some(PhpError::TypeErrorAt { msg, .. }) if msg.contains("must be of type int, string given")));
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
    fn non_scalar_param_hints_are_checked() {
        // array / object / class hints accept the right value and run the body.
        assert_eq!(vm_stdout(b"<?php function f(array $a){ echo $a[0], $a[1]; } f([7, 8]);"), b"78");
        assert_eq!(
            vm_stdout(b"<?php class A {} class B extends A { function n(){ return 'B'; } } function f(A $a){ echo $a->n(); } f(new B());"),
            b"B"
        );
        // a class hint rejects an unrelated object, naming both classes.
        let out = vm_outcome(b"<?php class A {} function f(A $a){} f(new stdClass());");
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeErrorAt { msg, .. }) if msg.contains("must be of type A, stdClass given")
        ), "got {:?}", out.fatal);
        // an array hint rejects an int.
        let out = vm_outcome(b"<?php function f(array $a){} f(123);");
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeErrorAt { msg, .. }) if msg.contains("must be of type array, int given")
        ), "got {:?}", out.fatal);
        // `?Foo` accepts null.
        assert_eq!(
            vm_stdout(b"<?php class A {} function f(?A $a){ echo $a===null?'N':'O'; } f(null);"),
            b"N"
        );
        // callable hint accepts a closure, rejects a plain string non-callable.
        assert_eq!(vm_stdout(b"<?php function f(callable $c){ echo $c(); } f(fn()=>'ok');"), b"ok");
    }

    #[test]
    fn return_type_hint_coerces_and_errors() {
        assert_eq!(vm_stdout(b"<?php function f(): int { return '5'; } echo f() === 5 ? 'Y':'N';"), b"Y");
        let out = vm_outcome(b"<?php function f(): int { return 'x'; } f();");
        // getMessage() carries no location; the throwable's line defaults to the
        // `return` statement site (a plain TypeError, not TypeErrorAt).
        assert!(matches!(
            &out.fatal,
            Some(PhpError::TypeError(m))
                if m == "f(): Return value must be of type int, string returned"
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
    fn new_of_forward_declared_class_resolves_at_runtime() {
        // `new B()` before B's declaration is valid: the compiler defers the class
        // resolution to run time (PHP does not require a class to precede its use).
        assert_eq!(
            vm_stdout(b"<?php $o = new B(); echo $o->hi(); class B { function hi(){ return 'B'; } }"),
            b"B"
        );
    }

    #[test]
    fn new_of_undefined_class_is_runtime_fatal() {
        // A genuinely undefined class is a catchable run-time Error, not a compile
        // error (the name may be declared conditionally / later).
        let o = vm_outcome(b"<?php echo 'x'; new Nope();");
        assert_eq!(o.stdout, b"x");
        match o.fatal {
            Some(PhpError::Error(m)) => {
                assert!(m.contains("Class \"Nope\" not found"), "message was: {m}")
            }
            other => panic!("expected Class-not-found Error, got {other:?}"),
        }
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
            // zend_handle_undef_args: a named call reports the unbound parameter,
            // not the positional-only "Too few arguments" form (oracle-verified).
            b"C::m(): Argument #2 ($b) not passed"
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

    // ----- Session 8: constructor visibility, is_callable ZPP, enum from,
    // date comparison, eager destructor sweep (vs PHP 8.5.7 CLI) -----

    #[test]
    fn new_private_ctor_from_outside_is_error() {
        let out = vm_stdout(
            b"<?php class P { private function __construct() {} public static function make() { return new self(); } }
              try { new P(); } catch (Error $e) { echo $e->getMessage(); }
              echo '|', get_class(P::make());",
        );
        assert_eq!(out, b"Call to private P::__construct() from global scope|P");
    }

    #[test]
    fn new_protected_ctor_subclass_ok_abstract_wins() {
        let out = vm_stdout(
            b"<?php abstract class A { private function __construct() {} }
              class B { protected function __construct() {} }
              class C extends B { public static function make() { return new C(); } }
              try { new A(); } catch (Error $e) { echo $e->getMessage(); }
              try { new B(); } catch (Error $e) { echo '|', $e->getMessage(); }
              echo '|', get_class(C::make());",
        );
        assert_eq!(
            out,
            b"Cannot instantiate abstract class A|Call to protected B::__construct() from global scope|C".to_vec()
        );
    }

    #[test]
    fn is_callable_static_style_instance_method_false() {
        // A VISIBLE instance method referenced static-style is not callable
        // (and does not fall back to __callStatic); an inaccessible or
        // missing one is callable iff __callStatic exists.
        let out = vm_stdout(
            b"<?php class C { public function i() {} public static function s() {} protected function p() {} public static function __callStatic($n, $a) {} }
              class D { public function i() {} }
              echo is_callable(['C','i'])?1:0, is_callable('C::i')?1:0, is_callable(['C','s'])?1:0,
                   is_callable(['C','p'])?1:0, is_callable(['C','nope'])?1:0, is_callable(['D','i'])?1:0, is_callable(['D','nope'])?1:0;",
        );
        assert_eq!(out, b"0011100");
    }

    #[test]
    fn is_callable_syntax_only_and_callable_name() {
        let out = vm_stdout(
            b"<?php class C { public function i() {} }
              echo is_callable(['NoSuch','m'], true)?1:0, is_callable(['C','i'], true)?1:0,
                   is_callable('anything at all', true)?1:0, is_callable([2=>'C',3=>'m'], true)?1:0,
                   is_callable(new stdClass, true)?1:0, '|';
              is_callable(['C','i'], true, $n1); is_callable([1,'m'], true, $n2); is_callable(new C, false, $n3);
              echo $n1, '|', $n2, '|', $n3;",
        );
        assert_eq!(out, b"11100|C::i|Array|C::__invoke");
    }

    #[test]
    fn weak_int_coercion_rejects_out_of_range() {
        // Zend's zend_parse_arg_long_weak: an overflowing numeric string (or
        // float) for an `int` parameter is a TypeError, not a saturating cast.
        let out = vm_stdout(
            b"<?php $f = static fn (int $v): int => $v;
              foreach (['9223372036854775808', '09223372036854775808', 9223372036854775808.0] as $v) {
                  try { $f($v); echo 'ok'; } catch (TypeError $e) { echo 'T'; }
              }
              echo $f('06'), $f('9223372036854775807') === PHP_INT_MAX ? 'M' : 'x';",
        );
        assert_eq!(out, b"TTT6M");
    }

    #[test]
    fn enum_from_applies_zpp_coercion() {
        let out = vm_stdout(
            b"<?php enum I: int { case A = 1; } enum S: string { case A = 'a'; }
              try { I::from('value'); } catch (TypeError $e) { echo 'T'; }
              echo I::from('1')->name, I::from(true)->name, I::tryFrom('1')->name;
              try { S::from(5); } catch (ValueError $e) { echo '|', $e->getMessage(); }",
        );
        assert_eq!(out, b"TAAA|\"5\" is not a valid backing value for enum S");
    }

    #[test]
    fn exception_subclass_keeps_redeclared_defaults() {
        // The ctor writes message/code/previous only when supplied (Zend's
        // zend_exceptions.c): a subclass redeclaring a default keeps it.
        let out = vm_stdout(
            b"<?php class E extends Exception { protected $code = 'strcode'; protected $message = 'preset'; }
              $e = new E(); echo $e->getCode(), '|', $e->getMessage(), '|', (new E('x'))->getMessage(), '|', (new Exception('m', 7))->getCode();",
        );
        assert_eq!(out, b"strcode|preset|x|7");
    }

    #[test]
    fn destructor_runs_eagerly_inside_functions() {
        // Zend destructs on refcount-zero anywhere: a temporary dying inside a
        // function runs __destruct before the next statement (Symfony's DI
        // configurators register definitions in __destruct).
        let out = vm_stdout(
            b"<?php class D { public function __construct(public $cb) {} public function __destruct() { ($this->cb)(); } }
              function f(&$hit) { new D(function () use (&$hit) { $hit = true; }); echo $hit ? 'eager' : 'late'; }
              $h = false; f($h);",
        );
        assert_eq!(out, b"eager");
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
        assert_eq!(
            out.stdout,
            b"\nWarning: Constant FOO already defined, this will be an error in PHP 9 in test.php on line 1\nf|1"
        );
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
        // Unknown class → __PHP_Incomplete_Class carrying the original class
        // name in __PHP_Incomplete_Class_Name plus the serialized fields (Zend).
        assert_eq!(
            vm_stdout(b"<?php $o=unserialize('O:3:\"Zzz\":1:{s:1:\"a\";i:9;}'); echo get_class($o),':',$o->a,':',$o->__PHP_Incomplete_Class_Name;"),
            b"__PHP_Incomplete_Class:9:Zzz"
        );
    }

    #[test]
    fn serialize_and_unserialize_backrefs() {
        // (The serialize side lives in php-builtins, absent from this harness;
        // it is covered by the corpus and the oracle probes.) A self-cycle
        // `r:` restores shared identity; `R:` restores a live alias.
        assert_eq!(
            vm_stdout(b"<?php $o=unserialize('O:8:\"stdClass\":1:{s:4:\"self\";r:1;}'); echo $o===$o->self?'C':'?';"),
            b"C"
        );
        // `R:` aliases a slot: writes through one element reach the other.
        assert_eq!(
            vm_stdout(b"<?php $u=unserialize('a:2:{i:0;a:1:{i:0;i:1;}i:1;R:2;}'); $u[0][0]=9; echo $u[1][0];"),
            b"9"
        );
    }

    #[test]
    fn unserialize_malformed_returns_false_with_warning() {
        let out = vm_outcome(b"<?php echo unserialize('garbage')===false?'F':'?';");
        assert_eq!(out.stdout, out.rendered);
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
        assert_eq!(out.stdout, out.rendered);
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
            b"\nDeprecated: Creation of dynamic property C::$dyn is deprecated in test.php on line 1\n19nn"
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
            b"\nDeprecated: Creation of dynamic property C::$dyn is deprecated in test.php on line 1\n11110"
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
        assert_eq!(out.stdout, out.rendered);
        assert!(
            out.rendered.windows(b"Notice: hi in ".len()).any(|w| w == b"Notice: hi in "),
            "rendered: {}",
            String::from_utf8_lossy(&out.rendered)
        );
    }

    #[test]
    fn trigger_error_user_warning_level() {
        let out = vm_outcome(b"<?php trigger_error('warn', E_USER_WARNING); echo 'B';");
        assert_eq!(out.stdout, out.rendered);
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
            b"\nWarning: oops in test.php on line 1\n512|oops|1"
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
        assert_eq!(out.stdout, b"\nWarning: from builtin in test.php on line 1\n2|from builtin");
    }

    #[test]
    fn error_get_last_is_most_recent_across_kinds() {
        // Most-recent-wins: a built-in warning after a trigger_error overwrites it.
        let out = vm_run(
            b"<?php trigger_error('u', E_USER_NOTICE); t_warn(); $e=error_get_last(); echo $e['type'],'|',$e['message'];",
            &fake_registry(),
        );
        assert_eq!(
            out.stdout,
            b"\nNotice: u in test.php on line 1\n\nWarning: from builtin in test.php on line 1\n2|from builtin"
        );
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
        assert_eq!(out.stdout, b"\nWarning: from builtin in test.php on line 1\n2|from builtin");
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

