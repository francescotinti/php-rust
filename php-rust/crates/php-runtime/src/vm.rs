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
    convert, ops, Closure, ClosureInfo, ClosureRender, Diag, Diags, GenKey, GenState, GenStatus,
    Key, Object, PhpArray, PhpError, PhpStr, Props, Zval,
};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::bytecode::{
    ClassTarget, DimBase, FieldBase, FieldStep, Func, Instantiable, Module, Op, StaticInit,
};
use crate::hir::{BinOp, CastKind, ClassId, Line, Slot, UnOp, Visibility};

/// The result of running a [`Module`]: the bytes written to stdout, the
/// diagnostics raised, and the fatal that stopped execution (if any).
#[derive(Debug, Default)]
pub struct VmOutcome {
    pub stdout: Vec<u8>,
    pub diags: Diags,
    pub fatal: Option<PhpError>,
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

/// Compile-and-run is the caller's job ([`crate::compile`]); this takes the
/// already-compiled module and executes its `main`.
pub fn run_module(module: &Module, registry: &Registry) -> VmOutcome {
    let mut vm = Vm {
        module,
        registry,
        stdout: Vec::new(),
        diags: Diags::new(),
        frames: Vec::new(),
        next_object_id: 1,
        static_props: HashMap::new(),
        magic_guard: HashSet::new(),
        created: Vec::new(),
        destructed: HashSet::new(),
        generators: HashMap::new(),
        fibers: HashMap::new(),
        fiber_stack: Vec::new(),
        fiber_class_id: module.class_index.get(&b"fiber"[..]).copied(),
        throwable_id: module.class_index.get(&b"throwable"[..]).copied(),
    };
    vm.frames.push(Frame::new(&module.main));
    let fatal = vm.run().err();
    // End-of-script destructors (LIFO over the objects still tracked), run after
    // `main` returns — or after a fatal, on a cleared stack (OOP-3d).
    vm.run_shutdown_destructors();
    VmOutcome {
        stdout: vm.stdout,
        diags: vm.diags,
        fatal,
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
            gen_id: None,
            yield_from: None,
            argc: 0,
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
    diags: Diags,
    frames: Vec<Frame<'m>>,
    /// Monotonic object-handle counter (`#N` in `var_dump`), starting at 1 like
    /// the tree-walker's `next_object_id`.
    next_object_id: u32,
    /// Persistent storage for `static` properties, keyed by (declaring class id,
    /// property name); lazily created on first access and shared for the run
    /// (OOP-2b), mirroring the tree-walker's `static_props`.
    static_props: HashMap<(ClassId, Vec<u8>), Rc<RefCell<Zval>>>,
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
}

impl<'m> Vm<'m> {
    /// Allocate a fresh object handle id.
    fn next_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
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
                Err(e) => match self.unwind(e, 0) {
                    None => {} // routed to a `catch`; resume there
                    Some(e) => return Err(e),
                },
            }
        }
    }

    /// Find the innermost `try` whose protected range covers the in-flight
    /// exception and route control to its catch-dispatch block, popping frames
    /// with no handler. Returns `None` once control is parked at a `catch`, or
    /// `Some(e)` if the exception is uncatchable (a non-object throw, an engine
    /// error — EXC-3 — or `Exit`) or escapes uncaught down to `floor`.
    ///
    /// `floor` is the lowest frame index this unwind may inspect/route into: the
    /// search stops once the frame at `floor` has been checked, leaving that frame
    /// on the stack. Top-level passes `0` (so `main` is the floor, retained as
    /// today); a generator resume passes the parked frame's depth, so an
    /// exception uncaught inside the generator surfaces at the resume site (the
    /// resumer then pops the dead generator frame).
    fn unwind(&mut self, e: PhpError, floor: usize) -> Option<PhpError> {
        // The in-flight Throwable object. A user `throw` of an object is itself
        // (EXC-1). An engine error (EXC-3a) is resolved to its prelude class by
        // name and a Throwable is synthesized carrying its message; if the class
        // is absent or can't be instantiated, the original error propagates.
        // `Exit` and a thrown non-object stay uncatchable.
        let obj = match &e {
            PhpError::Thrown(v) if matches!(v, Zval::Object(_)) => v.clone(),
            PhpError::Exit(_) | PhpError::Thrown(_) => return Some(e),
            engine => {
                let name = engine.class_name().to_ascii_lowercase();
                let msg = engine.message().to_owned();
                match self.module.class_index.get(name.as_bytes()).copied() {
                    Some(cid) => match self.synthesize_throwable(cid, &msg) {
                        Ok(v) => v,
                        Err(_) => return Some(e),
                    },
                    None => return Some(e),
                }
            }
        };
        loop {
            let top = self.frames.len() - 1;
            let faulting = self.frames[top].ip.saturating_sub(1);
            let region = self.frames[top]
                .func
                .exc_table
                .iter()
                .find(|r| faulting >= r.start as usize && faulting < r.end as usize)
                .copied();
            if let Some(r) = region {
                // Statement boundaries leave the operand stack at its baseline, so
                // clearing any partial-expression values restores it for the
                // handler. A catch region parks the exception on the stack for
                // `CatchMatch`; a finally region parks it in `pending_throw`, to be
                // re-raised at `EndFinally` (EXC-2).
                self.frames[top].stack.clear();
                if r.is_finally {
                    self.frames[top].pending_throw = Some(obj);
                } else {
                    // A new in-flight exception supersedes any exception parked by
                    // an earlier finally in this frame (e.g. a finally that threw).
                    self.frames[top].pending_throw = None;
                    self.frames[top].stack.push(obj);
                }
                self.frames[top].ip = r.target as usize;
                return None;
            }
            if top == floor {
                // The floor frame had no matching handler: propagate, leaving the
                // floor frame on the stack for the caller to dispose of.
                return Some(e);
            }
            self.frames.pop();
        }
    }

    /// The bounded dispatch loop: runs until the frame at `baseline` returns
    /// ([`RunExit::Returned`]) or a generator at `baseline` suspends at a `yield`
    /// ([`RunExit::Yielded`]), or an op raises a `PhpError` (which the caller
    /// routes through [`Self::unwind`]). Frames above `baseline` (ordinary
    /// callees) return normally to their callers within this same loop.
    fn run_loop(&mut self, baseline: usize) -> Result<RunExit, PhpError> {
        loop {
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
                Op::Pop => {
                    self.frames[top].stack.pop();
                }
                Op::Dup => {
                    let v = self.frames[top].stack.last().expect("Dup on empty stack").clone();
                    self.frames[top].stack.push(v);
                }
                Op::LoadSlot(s) => {
                    // An unset local reads as NULL (the curated corpus is
                    // warning-free; the "Undefined variable" notice rides the
                    // diagnostics-ordering work). A reference slot is followed.
                    let v = read_slot(&self.frames[top].slots[s as usize]);
                    self.frames[top].stack.push(v);
                }
                Op::StoreSlot(s) => {
                    let v = self.frames[top].stack.pop().expect("StoreSlot on empty stack");
                    store_slot(&mut self.frames[top].slots[s as usize], v);
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
                    self.stdout.extend_from_slice(s.as_bytes());
                }
                Op::Print => {
                    let v = self.frames[top].stack.pop().expect("Print operand");
                    let s = convert::to_zstr(&v, &mut self.diags);
                    self.stdout.extend_from_slice(s.as_bytes());
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
                Op::FetchDim => {
                    let key = self.frames[top].stack.pop().expect("FetchDim key");
                    let base = self.frames[top].stack.pop().expect("FetchDim base");
                    self.frames[top].stack.push(read_dim(&base, &key));
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
                Op::EndFinally => {
                    // EXC-2: if an exception was propagating through this finally,
                    // re-raise it now; otherwise fall through past the `try`.
                    if let Some(v) = self.frames[top].pending_throw.take() {
                        return Err(PhpError::Thrown(v));
                    }
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
                    self.enter_callee(frame);
                }
                Op::CallBuiltin { name, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::Value(f)) => *f,
                        // The compiler only emits CallBuiltin for value builtins.
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let args = self.pop_keys(top, argc); // pops argc, source order
                    let result = {
                        let mut ctx = Ctx { out: &mut self.stdout, diags: &mut self.diags };
                        f(&args, &mut ctx)?
                    };
                    self.frames[top].stack.push(result);
                }
                Op::CallBuiltinRef { name, slot, argc } => {
                    let f = match self.registry.get(&name[..]) {
                        Some(Builtin::RefFirst(f)) => *f,
                        _ => return Err(undefined_builtin(&name)),
                    };
                    let rest = self.pop_keys(top, argc);
                    let result = builtin_ref_call(f, &mut self.frames[top].slots[slot as usize], &rest, &mut self.stdout, &mut self.diags)?;
                    self.frames[top].stack.push(result);
                }
                Op::Ret => {
                    let ret = self.frames[top].stack.pop().unwrap_or(Zval::Null);
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
                Op::PropSet { name } => {
                    let value = self.frames[top].stack.pop().expect("PropSet value");
                    let obj = self.frames[top].stack.pop().expect("PropSet object");
                    let cur = self.frames[top].class;
                    let target = obj.deref_clone();
                    if let Zval::Object(o) = &target {
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
                    // A `Generator` is not a user object: dispatch its built-in
                    // methods (current/key/next/valid/rewind/…) directly (GEN).
                    if let Zval::Generator(gs) = &this {
                        let gs = Rc::clone(gs);
                        let result = self.generator_method(gs, &method, args)?;
                        self.frames[top].stack.push(result);
                        continue;
                    }
                    // A `Fiber` instance's methods (start/resume/getReturn/is*) are
                    // dispatched natively, except `__construct` which runs the
                    // prelude body via `InvokeMethod` (GEN-4).
                    if let (Zval::Object(o), Some(fcid)) = (&this, self.fiber_class_id) {
                        let cid = o.borrow().class_id as usize;
                        if is_instance_of(self.module, cid, fcid) {
                            let result = self.fiber_method(&this, &method, args)?;
                            self.frames[top].stack.push(result);
                            continue;
                        }
                    }
                    let cid = match &this {
                        Zval::Object(o) => o.borrow().class_id as usize,
                        other => {
                            return Err(PhpError::Error(format!(
                                "Call to a member function {}() on {}",
                                String::from_utf8_lossy(&method),
                                other.error_type_name()
                            )))
                        }
                    };
                    self.dispatch_instance_call(top, cid, this, &method, args)?;
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
                    self.enter_callee(frame);
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
                Op::StaticCallDynamic { method, argc } => {
                    // `$cls::m()` (PAR): args are on top, the class reference beneath.
                    let args = self.pop_keys(top, argc);
                    let classval =
                        self.frames[top].stack.pop().expect("StaticCallDynamic class");
                    let start = self.resolve_dynamic_class(&classval)?;
                    // A dynamic class is non-forwarding, like a named class.
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

    /// Run a path write/compound/incdec rooted at `base`, drilling through the
    /// intermediate `keys` and applying `last` at the leaf. Returns the value the
    /// expression evaluates to (assigned value / compound result / inc-dec value).
    fn path_op(
        &mut self,
        base: DimBase,
        top: usize,
        keys: Vec<Zval>,
        last: Last,
    ) -> Result<Zval, PhpError> {
        let cell = match base {
            DimBase::Local(s) => &mut self.frames[top].slots[s as usize],
            DimBase::Global(s) => &mut self.frames[0].slots[s as usize],
        };
        path_apply(cell, &keys, last, &mut self.diags)
    }

    /// Dispatch a dynamic call on a runtime callee value (CLO): an anonymous
    /// closure runs its body (binding captures then args); a named closure or a
    /// string names a user function / builtin; a reference is followed. Anything
    /// else is an uncatchable "not callable" error. A pushed frame runs via the
    /// main loop; a builtin result is pushed on the current frame's stack.
    fn invoke_value(&mut self, callee: Zval, args: Vec<Zval>) -> Result<(), PhpError> {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                None => {
                    self.push_closure_frame(&cl, args);
                    Ok(())
                }
                Some(name) => {
                    let name = name.as_bytes().to_vec();
                    self.invoke_named(&name, args)
                }
            },
            Zval::Str(s) => {
                let name = s.as_bytes().to_vec();
                self.invoke_named(&name, args)
            }
            Zval::Ref(rc) => {
                let inner = rc.borrow().clone();
                self.invoke_value(inner, args)
            }
            other => Err(PhpError::Error(format!(
                "Value of type {} is not callable",
                other.error_type_name()
            ))),
        }
    }

    /// Install a frame for an anonymous closure: bind its captured variables into
    /// their slots, then the call arguments into the leading parameter slots, and
    /// the bound `$this`. Mirrors `eval::call_closure` (captures before params).
    fn push_closure_frame(&mut self, cl: &Closure, args: Vec<Zval>) {
        let callee = &self.module.closures[cl.fn_idx];
        let mut frame = Frame::new(callee);
        for (slot, val) in &cl.captures {
            frame.slots[*slot as usize] = val.clone();
        }
        bind_params(&mut frame, args);
        frame.this = cl.bound_this.clone();
        self.enter_callee(frame);
    }

    /// Dispatch a call to a function *name* (a string callable / first-class
    /// callable / named closure): a user function (case-insensitive, shadows
    /// builtins) installs a frame; a value builtin runs and pushes its result.
    fn invoke_named(&mut self, name: &[u8], args: Vec<Zval>) -> Result<(), PhpError> {
        if let Some(idx) =
            self.module.functions.iter().position(|f| name_eq_ignore_case(&f.name, name))
        {
            let callee = &self.module.functions[idx];
            let mut frame = Frame::new(callee);
            bind_params(&mut frame, args);
            self.enter_callee(frame);
            return Ok(());
        }
        match self.registry.get(name) {
            Some(Builtin::Value(f)) => {
                let f = *f;
                let result = {
                    let mut ctx = Ctx { out: &mut self.stdout, diags: &mut self.diags };
                    f(&args, &mut ctx)?
                };
                let top = self.frames.len() - 1;
                self.frames[top].stack.push(result);
                Ok(())
            }
            Some(Builtin::RefFirst(_)) => Err(PhpError::Error(format!(
                "VM: dynamic call to by-reference builtin {}() is out of slice",
                String::from_utf8_lossy(name)
            ))),
            None => Err(PhpError::Error(format!(
                "Call to undefined function {}()",
                String::from_utf8_lossy(name)
            ))),
        }
    }

    /// Build a `Generator` handle for a freshly-bound generator-body `frame`
    /// (GEN): park the frame under a fresh id and return the `Zval::Generator` the
    /// call expression evaluates to. The body does not run until the generator is
    /// first advanced (`NotStarted`). The parked frame lives in `self.generators`
    /// — no coroutine, no `unsafe`, unlike the tree-walker's `corosensei` driver.
    fn make_generator(&mut self, mut frame: Frame<'m>) -> Zval {
        let id = self.next_id();
        frame.gen_id = Some(id);
        let func_name = frame.func.name.to_vec().into_boxed_slice();
        self.generators.insert(id, frame);
        Zval::Generator(Rc::new(RefCell::new(GenState {
            id,
            func_name,
            status: GenStatus::NotStarted,
            advanced: false,
            cur_key: Zval::Null,
            cur_val: Zval::Null,
            ret: Zval::Null,
            auto_key: 0,
            driver: None,
        })))
    }

    /// Enter a freshly-built callee `frame`: if its body is a generator,
    /// materialise a `Generator` handle on the caller's operand stack instead of
    /// running it (GEN); otherwise push it to run. The caller is the current top
    /// frame, so this is called *before* `frame` is pushed.
    fn enter_callee(&mut self, frame: Frame<'m>) {
        if frame.func.is_generator {
            let gen = self.make_generator(frame);
            let top = self.frames.len() - 1;
            self.frames[top].stack.push(gen);
        } else {
            self.frames.push(frame);
        }
    }

    /// Advance a generator one step (GEN): move its parked frame onto the call
    /// stack and run until it yields again, returns, or throws an uncaught
    /// exception. Mirrors `eval::resume_generator` for the status guards and
    /// auto-key resolution. `sent` is the value the suspended `yield` expression
    /// evaluates to (NULL for `next()`/`foreach`).
    fn resume_generator(
        &mut self,
        gs_rc: &Rc<RefCell<GenState>>,
        sent: Zval,
    ) -> Result<(), PhpError> {
        let was_suspended = {
            let mut gs = gs_rc.borrow_mut();
            match gs.status {
                GenStatus::Running => {
                    return Err(PhpError::Error(
                        "Cannot resume an already running generator".to_string(),
                    ))
                }
                GenStatus::Done => return Ok(()),
                GenStatus::Suspended => {
                    gs.advanced = true;
                    gs.status = GenStatus::Running;
                    true
                }
                GenStatus::NotStarted => {
                    gs.status = GenStatus::Running;
                    false
                }
            }
        };
        let id = gs_rc.borrow().id;
        let frame = self.generators.remove(&id).expect("parked generator frame");
        let baseline = self.frames.len();
        self.frames.push(frame);
        if was_suspended {
            // The suspended `yield` expression evaluates to the sent value.
            self.frames[baseline].stack.push(sent);
        }
        // Run the body until it yields/returns; route its *own* exceptions through
        // `unwind` with the generator frame as the floor, so a `try` inside the
        // generator is honoured and an uncaught throw surfaces at the resume site.
        let outcome = loop {
            match self.run_loop(baseline) {
                Ok(exit) => break Ok(exit),
                Err(e) => match self.unwind(e, baseline) {
                    None => continue,
                    Some(e) => break Err(e),
                },
            }
        };
        match outcome {
            Ok(RunExit::Yielded { key, value }) => {
                // The frame was already parked by `Op::Yield` / `Op::YieldFrom`.
                let mut gs = gs_rc.borrow_mut();
                let resolved = match key {
                    GenKey::Auto => {
                        let k = Zval::Long(gs.auto_key);
                        gs.auto_key += 1;
                        k
                    }
                    GenKey::Keyed(Zval::Long(n)) => {
                        if n >= gs.auto_key {
                            gs.auto_key = n.wrapping_add(1);
                        }
                        Zval::Long(n)
                    }
                    GenKey::Keyed(z) | GenKey::Verbatim(z) => z,
                };
                gs.cur_key = resolved;
                gs.cur_val = value;
                gs.status = GenStatus::Suspended;
                Ok(())
            }
            Ok(RunExit::Returned(v)) => {
                // The body returned; `Op::Ret` already popped the generator frame.
                let mut gs = gs_rc.borrow_mut();
                gs.ret = v;
                gs.cur_key = Zval::Null;
                gs.cur_val = Zval::Null;
                gs.status = GenStatus::Done;
                Ok(())
            }
            Ok(RunExit::Suspended { .. }) => {
                // `Fiber::suspend` reached across a generator resume (a fiber
                // suspended from within a generator that is itself inside the
                // fiber). This pathological nesting is out of scope; fail cleanly.
                let mut gs = gs_rc.borrow_mut();
                gs.status = GenStatus::Done;
                Err(PhpError::Error(
                    "VM: cannot suspend a Fiber from within a Generator (unsupported nesting)"
                        .to_string(),
                ))
            }
            Err(e) => {
                // Uncaught inside the generator: `unwind` left the dead frame at
                // the baseline; drop it and surface the exception at the resumer.
                self.frames.pop();
                let mut gs = gs_rc.borrow_mut();
                gs.cur_key = Zval::Null;
                gs.cur_val = Zval::Null;
                gs.status = GenStatus::Done;
                Err(e)
            }
        }
    }

    /// Prime a `NotStarted` generator to its first `yield` (GEN); a no-op
    /// otherwise. Mirrors `eval::ensure_started`.
    fn ensure_started(&mut self, gs_rc: &Rc<RefCell<GenState>>) -> Result<(), PhpError> {
        if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
            self.resume_generator(gs_rc, Zval::Null)?;
        }
        Ok(())
    }

    /// Synthesize a plain `Exception` carrying `msg`, for the generator misuse
    /// errors PHP raises as `Exception` (rewind-after-run, getReturn-before-return)
    /// — the tree-walker raises these as `Error`; the VM matches real PHP. Falls
    /// back to an engine `Error` if the prelude has no `Exception`.
    fn gen_exception(&mut self, msg: &str) -> PhpError {
        match self.module.class_index.get(&b"exception"[..]).copied() {
            Some(cid) => match self.synthesize_throwable(cid, msg) {
                Ok(obj) => PhpError::Thrown(obj),
                Err(e) => e,
            },
            None => PhpError::Error(msg.to_string()),
        }
    }

    /// Dispatch a built-in `Generator` method (GEN), returning the value to leave
    /// on the caller's stack. Mirrors `eval::generator_method`.
    fn generator_method(
        &mut self,
        gs_rc: Rc<RefCell<GenState>>,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        match method.to_ascii_lowercase().as_slice() {
            b"current" => {
                self.ensure_started(&gs_rc)?;
                Ok(gs_rc.borrow().cur_val.clone())
            }
            b"key" => {
                self.ensure_started(&gs_rc)?;
                Ok(gs_rc.borrow().cur_key.clone())
            }
            b"next" => {
                self.ensure_started(&gs_rc)?;
                self.resume_generator(&gs_rc, Zval::Null)?;
                Ok(Zval::Null)
            }
            b"valid" => {
                self.ensure_started(&gs_rc)?;
                let valid = !matches!(gs_rc.borrow().status, GenStatus::Done);
                Ok(Zval::Bool(valid))
            }
            b"rewind" => {
                self.ensure_started(&gs_rc)?;
                if gs_rc.borrow().advanced {
                    return Err(self.gen_exception("Cannot rewind a generator that was already run"));
                }
                Ok(Zval::Null)
            }
            b"send" => {
                // Deliver `$value` as the suspended `yield`'s result and advance;
                // an unstarted generator is primed first (GEN-2). Returns the next
                // yielded value (NULL once the generator is done).
                let value = args.into_iter().next().unwrap_or(Zval::Null);
                if matches!(gs_rc.borrow().status, GenStatus::NotStarted) {
                    self.resume_generator(&gs_rc, Zval::Null)?;
                }
                self.resume_generator(&gs_rc, value)?;
                Ok(gs_rc.borrow().cur_val.clone())
            }
            b"getreturn" => {
                // PHP auto-primes: getReturn() on a fresh generator starts it (so a
                // body that returns before any yield exposes its value); before the
                // body has returned, it is an error (GEN-2).
                self.ensure_started(&gs_rc)?;
                if !matches!(gs_rc.borrow().status, GenStatus::Done) {
                    return Err(self
                        .gen_exception("Cannot get return value of a generator that hasn't returned"));
                }
                Ok(gs_rc.borrow().ret.clone())
            }
            other => Err(PhpError::Error(format!(
                "Call to undefined method Generator::{}()",
                String::from_utf8_lossy(other)
            ))),
        }
    }

    /// The current status of fiber `id` (GEN-4); a missing entry means the fiber
    /// has not been started yet.
    fn fiber_status(&self, id: u32) -> Option<FiberStatus> {
        self.fibers.get(&id).map(|s| s.status)
    }

    /// Run a fiber's frame segment at `baseline` until it suspends, its callable
    /// returns, or it throws (GEN-4). Shared by `start`/`resume`. Returns the
    /// value to hand back to the caller (the `Fiber::suspend` value, or NULL on
    /// termination); an exception that escapes the fiber propagates to the caller.
    fn drive_fiber(&mut self, id: u32, obj: &Zval, baseline: usize) -> Result<Zval, PhpError> {
        self.fiber_stack.push(FiberContext { id, baseline, obj: obj.clone() });
        let outcome = loop {
            match self.run_loop(baseline) {
                Ok(exit) => break Ok(exit),
                Err(e) => match self.unwind(e, baseline) {
                    None => continue,
                    Some(e) => break Err(e),
                },
            }
        };
        self.fiber_stack.pop();
        match outcome {
            Ok(RunExit::Suspended { value }) => {
                // `Fiber::suspend` already parked frames[baseline..] into the entry.
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Suspended;
                }
                Ok(value)
            }
            Ok(RunExit::Returned(v)) => {
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Terminated;
                    st.ret = v;
                }
                Ok(Zval::Null)
            }
            Ok(RunExit::Yielded { .. }) => {
                unreachable!("a fiber callable does not `yield` at its own baseline")
            }
            Err(e) => {
                // The exception escaped the fiber: it terminates and the error
                // propagates out of start()/resume(). `unwind` left the dead
                // baseline frame; drop the whole segment.
                self.frames.truncate(baseline);
                if let Some(st) = self.fibers.get_mut(&id) {
                    st.status = FiberStatus::Terminated;
                }
                Err(e)
            }
        }
    }

    /// `$fiber->start(...$args)` (GEN-4): invoke the fiber's callable as a fresh
    /// frame and run it to the first suspend or to completion.
    fn fiber_start(&mut self, obj: &Zval, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_start on a non-object"),
        };
        if self.fiber_status(id).is_some() {
            return Err(PhpError::Error(
                "Cannot start a fiber that has already been started".to_string(),
            ));
        }
        let callable = match obj {
            Zval::Object(o) => o.borrow().props.get(b"callable").cloned().unwrap_or(Zval::Null),
            _ => Zval::Null,
        };
        self.fibers.insert(
            id,
            FiberState { status: FiberStatus::Running, parked: Vec::new(), ret: Zval::Null },
        );
        let baseline = self.frames.len();
        self.invoke_value(callable, args)?;
        if self.frames.len() != baseline + 1 {
            // A non-closure callable (builtin / generator function) did not push a
            // plain fiber frame; out of scope.
            self.frames.truncate(baseline);
            self.fibers.remove(&id);
            return Err(PhpError::Error(
                "VM: fiber callable must be a closure or function (other callables unsupported)"
                    .to_string(),
            ));
        }
        self.drive_fiber(id, obj, baseline)
    }

    /// `$fiber->resume($value)` (GEN-4): restore the parked segment, deliver
    /// `$value` as the suspended `Fiber::suspend`'s result, and run on.
    fn fiber_resume(&mut self, obj: &Zval, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_resume on a non-object"),
        };
        if self.fiber_status(id) != Some(FiberStatus::Suspended) {
            return Err(PhpError::Error(
                "Cannot resume a fiber that is not suspended".to_string(),
            ));
        }
        let value = args.into_iter().next().unwrap_or(Zval::Null);
        let parked = std::mem::take(&mut self.fibers.get_mut(&id).expect("fiber state").parked);
        let baseline = self.frames.len();
        self.frames.extend(parked);
        // The suspended `Fiber::suspend(...)` call evaluates to the resume value.
        self.frames.last_mut().expect("restored fiber frame").stack.push(value);
        self.fibers.get_mut(&id).expect("fiber state").status = FiberStatus::Running;
        self.drive_fiber(id, obj, baseline)
    }

    /// Dispatch a `Fiber` instance method (GEN-4), returning the value to leave on
    /// the caller's stack. `Fiber::suspend`/`getCurrent` are static and handled at
    /// the `StaticCall` site instead.
    fn fiber_method(
        &mut self,
        obj: &Zval,
        method: &[u8],
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        let id = match obj {
            Zval::Object(o) => o.borrow().id,
            _ => unreachable!("fiber_method on a non-object"),
        };
        match method.to_ascii_lowercase().as_slice() {
            b"start" => self.fiber_start(obj, args),
            b"resume" => self.fiber_resume(obj, args),
            b"getreturn" => match self.fibers.get(&id) {
                Some(st) if st.status == FiberStatus::Terminated => Ok(st.ret.clone()),
                _ => Err(PhpError::Error(
                    "Cannot get fiber return value: The fiber has not returned".to_string(),
                )),
            },
            b"isstarted" => Ok(Zval::Bool(self.fiber_status(id).is_some())),
            b"issuspended" => {
                Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Suspended)))
            }
            b"isrunning" => Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Running))),
            b"isterminated" => {
                Ok(Zval::Bool(self.fiber_status(id) == Some(FiberStatus::Terminated)))
            }
            b"throw" => Err(PhpError::Error(
                "VM: Fiber::throw() is not yet supported".to_string(),
            )),
            other => Err(PhpError::Error(format!(
                "Call to undefined method Fiber::{}()",
                String::from_utf8_lossy(other)
            ))),
        }
    }

    /// Dispatch an instance method call `obj->method(args)` where the receiver's
    /// class `cid` and bound `$this` are already resolved (OOP). A missing or
    /// inaccessible target routes to `__call`, otherwise raises the visibility /
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
                self.enter_callee(frame);
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
                self.enter_callee(frame);
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

    /// `(object)` cast (PAR): an object passes through; an array becomes a
    /// stdClass with one property per element (int keys stringified); null/unset
    /// is an empty stdClass; a scalar becomes `stdClass { scalar: v }`. Mirrors
    /// `eval::object_cast`.
    fn object_cast(&mut self, v: Zval) -> Result<Zval, PhpError> {
        match v.deref_clone() {
            obj @ Zval::Object(_) => Ok(obj),
            Zval::Array(a) => {
                let obj = self.alloc_stdclass()?;
                if let Zval::Object(o) = &obj {
                    let mut b = o.borrow_mut();
                    for (k, val) in a.iter() {
                        let name = match k {
                            Key::Int(i) => i.to_string().into_bytes(),
                            Key::Str(s) => s.as_bytes().to_vec(),
                        };
                        b.props.set(&name, val.deref_clone());
                    }
                }
                Ok(obj)
            }
            Zval::Null | Zval::Undef => self.alloc_stdclass(),
            scalar => {
                let obj = self.alloc_stdclass()?;
                if let Zval::Object(o) = &obj {
                    o.borrow_mut().props.set(b"scalar", scalar);
                }
                Ok(obj)
            }
        }
    }

    /// Allocate a fresh empty `stdClass` instance (PAR), for `(object)` casts.
    fn alloc_stdclass(&mut self) -> Result<Zval, PhpError> {
        let cid = self
            .module
            .class_index
            .get(&b"stdclass"[..])
            .copied()
            .ok_or_else(|| PhpError::Error("VM: stdClass is not available".to_string()))?;
        self.alloc_object(cid)
    }

    /// Resolve a runtime class-reference value to its class id (PAR, dynamic
    /// class): an object reuses its class; a string is looked up
    /// case-insensitively with a leading `\` stripped; anything else (or an
    /// unknown name) yields `None`. Used by `instanceof $cls` (where `None` means
    /// `false`); `new $cls` resolves inline so it can distinguish the error kinds.
    fn class_id_from_value(&self, v: &Zval) -> Option<ClassId> {
        match v {
            Zval::Object(o) => Some(o.borrow().class_id as usize),
            Zval::Str(s) => {
                let raw = s.as_bytes();
                let name = raw.strip_prefix(b"\\").unwrap_or(raw);
                self.module.class_index.get(&name.to_ascii_lowercase()).copied()
            }
            Zval::Ref(r) => self.class_id_from_value(&r.borrow()),
            _ => None,
        }
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

    /// Build a Throwable object of `cid` carrying `message`, used to materialise
    /// an engine error (`TypeError`, `DivisionByZeroError`, …) so it can be
    /// offered to a `catch` (EXC-3a). Mirrors `eval::synthesize_throwable`:
    /// allocates the instance (prop defaults + `info` + id, via `alloc_object`)
    /// **without** running a constructor, then overwrites `message` directly.
    /// `line`/`file`/`trace` stay at their prelude defaults here — they are
    /// filled by the line-tracking (EXC-3b) and stack-trace (EXC-3c) steps.
    fn synthesize_throwable(&mut self, cid: ClassId, message: &str) -> Result<Zval, PhpError> {
        let value = self.alloc_object(cid)?;
        // The line of the op that faulted (`ip-1` in the faulting frame), the
        // module's file, and the current stack trace — mirroring
        // `eval::synthesize_throwable` (EXC-3b/3c).
        let line = self.cur_line(self.frames.len() - 1);
        let (trace, trace_string) = self.capture_trace();
        if let Zval::Object(o) = &value {
            let mut b = o.borrow_mut();
            b.props
                .set(b"message", Zval::Str(PhpStr::new(message.as_bytes().to_vec())));
            b.props.set(b"line", Zval::Long(line as i64));
            b.props
                .set(b"file", Zval::Str(PhpStr::new(self.module.file.to_vec())));
            b.props.set(b"trace", trace);
            b.props
                .set(b"traceString", Zval::Str(PhpStr::new(trace_string)));
        }
        Ok(value)
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

    /// Write `value` through a mixed field path. The base cell borrows
    /// `self.frames` and `&mut self.diags` a disjoint field, so the two coexist
    /// (the same split the array `path_op` relies on).
    fn field_set(
        &mut self,
        base: FieldBase,
        top: usize,
        steps: &[FieldStep],
        keys: Vec<Zval>,
        value: Zval,
    ) -> Result<(), PhpError> {
        let cell = match base {
            FieldBase::Local(s) => &mut self.frames[top].slots[s as usize],
            FieldBase::Global(s) => &mut self.frames[0].slots[s as usize],
            FieldBase::This => self.frames[top].this.as_mut().ok_or_else(|| {
                PhpError::Error("Using $this when not in object context".to_string())
            })?,
        };
        field_write(cell, steps, &mut keys.into_iter(), value, &mut self.diags)
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

    /// Decide whether a magic property accessor of `kind` should run for `name` on
    /// `o` instead of direct access (OOP-3b), mirroring the tree-walker's
    /// `magic_prop_method`: it applies when the property is missing *or* not
    /// visible from `cur_class`, the class defines the accessor, and no same-key
    /// guard is active. Returns `(defining class, method index, object id)`.
    fn magic_applies(
        &self,
        o: &Rc<RefCell<Object>>,
        name: &[u8],
        cur_class: Option<ClassId>,
        kind: MagicKind,
        magic_name: &[u8],
    ) -> Option<(ClassId, usize, u32)> {
        let (cid, oid, present, accessible) = {
            let obj = o.borrow();
            let cid = obj.class_id as usize;
            let accessible = match resolve_prop_decl(self.module, cid, name) {
                Some((vis, dc)) => visible_from(self.module, cur_class, vis, dc),
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
        let (defc, midx) = resolve_method_runtime(self.module, cid, magic_name)?;
        Some((defc, midx, oid))
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

    /// Run `__destruct` on every object still tracked at the end of the script,
    /// in reverse creation order (PHP shutdown is LIFO), step OOP-3d. The frame
    /// stack is cleared first so this works even after a fatal unwound `main`.
    fn run_shutdown_destructors(&mut self) {
        self.frames.clear();
        let survivors = std::mem::take(&mut self.created);
        for o in survivors.into_iter().rev() {
            let (cid, id) = {
                let b = o.borrow();
                (b.class_id as usize, b.id)
            };
            if self.destructed.contains(&id) {
                continue;
            }
            let module = self.module;
            if let Some((defc, midx)) = resolve_method_runtime(module, cid, b"__destruct") {
                self.destructed.insert(id);
                let callee = &module.classes[defc].methods[midx].func;
                let mut frame = Frame::new(callee);
                frame.this = Some(Zval::Object(Rc::clone(&o)));
                frame.class = Some(defc);
                frame.static_class = Some(cid);
                self.frames.push(frame);
                // Drive the destructor to completion; swallow any fatal it raises
                // (PHP turns a shutdown-time throw into a separate fatal).
                let _ = self.run();
            }
        }
    }
}

/// The leaf operation of a path write, carried to the bottom of the drill-down.
enum Last {
    Set { key: Zval, value: Zval },
    Append { value: Zval },
    OpSet { key: Zval, op: BinOp, rhs: Zval },
    IncDec { key: Zval, inc: bool, pre: bool },
}

/// Silently follow `keys` from `cell` without auto-vivifying anything, returning
/// the leaf value if the whole path exists (an unset variable or a missing key
/// at any level yields `None`). Backs `isset` / `empty`. A reference is followed;
/// a string base supports a final byte-offset step.
fn silent_get_path(cell: &Zval, keys: &[Zval]) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
        return silent_get_path(&rc.borrow(), keys);
    }
    match keys.split_first() {
        None => match cell {
            Zval::Undef => None,
            other => Some(other.clone()),
        },
        Some((k, rest)) => match cell {
            Zval::Array(a) => {
                let key = coerce_key_silent(k)?;
                a.get(&key).and_then(|child| silent_get_path(child, rest))
            }
            Zval::Str(s) if rest.is_empty() => {
                string_offset(s, k).map(|byte| Zval::Str(PhpStr::new(vec![byte])))
            }
            _ => None,
        },
    }
}

/// The in-bounds byte at a string offset (negatives count from the end), or
/// `None` if out of range — the existence test behind `isset($s[i])`.
fn string_offset(s: &PhpStr, key: &Zval) -> Option<u8> {
    if matches!(key, Zval::Array(_) | Zval::Object(_) | Zval::Closure(_) | Zval::Generator(_)) {
        return None;
    }
    let i = convert::to_long_cast(key, &mut Diags::new());
    let len = s.len() as i64;
    let idx = if i < 0 { len + i } else { i };
    if idx < 0 || idx >= len {
        None
    } else {
        Some(s.as_bytes()[idx as usize])
    }
}

/// Remove the leaf of `keys` from `cell` (or, when `keys` is empty, unset the
/// variable itself by resetting it to `Undef`). A missing intermediate level is
/// a silent no-op; copy-on-write applies to each array touched.
fn unset_into(cell: &mut Zval, keys: &[Zval]) {
    match keys.split_first() {
        None => *cell = Zval::Undef,
        Some((k, rest)) => {
            if let Zval::Ref(rc) = cell {
                let mut inner = rc.borrow_mut();
                unset_into(&mut inner, keys);
                return;
            }
            if let Zval::Array(rc) = cell {
                if let Some(key) = coerce_key_silent(k) {
                    let arr = Rc::make_mut(rc);
                    if rest.is_empty() {
                        arr.remove(&key);
                    } else if let Some(child) = arr.get_mut(&key) {
                        unset_into(child, rest);
                    }
                }
            }
        }
    }
}

/// Write `value` through a mixed field path (OOP-2c), the VM analogue of the
/// tree-walker's `write_into`: a reference is written through; an object property
/// is navigated *in place* (no copy-on-write, shared `Rc<RefCell>`); an array
/// element auto-vivifies and copy-on-writes. `Index` steps consume `keys` in
/// source order.
fn field_write(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
    value: Zval,
    diags: &mut Diags,
) -> Result<(), PhpError> {
    if let Zval::Ref(cell) = target {
        let inner = &mut *cell.borrow_mut();
        return field_write(inner, steps, keys, value, diags);
    }
    let Some((first, rest)) = steps.split_first() else {
        *target = value;
        return Ok(());
    };
    match first {
        FieldStep::Prop(name) => {
            match target {
                Zval::Object(o) => {
                    let mut obj = o.borrow_mut();
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
                        field_write(child, rest, keys, value, diags)?;
                    }
                }
                other => {
                    diags.push(Diag::Warning(format!(
                        "Attempt to assign property \"{}\" on {}",
                        String::from_utf8_lossy(name),
                        other.error_type_name()
                    )));
                }
            }
            Ok(())
        }
        FieldStep::Index => {
            let key = keys.next().expect("field index key");
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if rest.is_empty() {
                // Overwrite a plain element, but write *through* an existing
                // reference element (the recursive call derefs at its top).
                match arr.get_mut(&k) {
                    Some(child) => field_write(child, rest, keys, value, diags)?,
                    None => arr.insert(k, value),
                }
            } else {
                if !arr.contains_key(&k) {
                    arr.insert(k.clone(), Zval::Array(Rc::new(PhpArray::new())));
                }
                let child = arr.get_mut(&k).expect("key just inserted");
                field_write(child, rest, keys, value, diags)?;
            }
            Ok(())
        }
        FieldStep::Append => {
            ensure_array(target)?;
            let Zval::Array(rc) = target else { unreachable!("ensured array") };
            let arr = Rc::make_mut(rc);
            let occupied =
                || PhpError::Error("Cannot add element to the array as the next element is already occupied".to_string());
            if rest.is_empty() {
                arr.append(value).map_err(|_| occupied())?;
            } else {
                let mut child = Zval::Array(Rc::new(PhpArray::new()));
                field_write(&mut child, rest, keys, value, diags)?;
                arr.append(child).map_err(|_| occupied())?;
            }
            Ok(())
        }
    }
}

/// Silently read a mixed field path's value (OOP-2c), `None` if any level is
/// absent — backs compound/inc-dec (missing → NULL) and `isset`/field tests.
fn field_get(cell: &Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>) -> Option<Zval> {
    if let Zval::Ref(rc) = cell {
        return field_get(&rc.borrow(), steps, keys);
    }
    match steps.split_first() {
        None => match cell {
            Zval::Undef => None,
            other => Some(other.deref_clone()),
        },
        Some((first, rest)) => match first {
            FieldStep::Prop(name) => match cell {
                Zval::Object(o) => {
                    let obj = o.borrow();
                    match obj.props.get(name) {
                        Some(v) => field_get(v, rest, keys),
                        None => None,
                    }
                }
                _ => None,
            },
            FieldStep::Index => {
                let key = keys.next()?;
                match cell {
                    Zval::Array(a) => {
                        let k = coerce_key_silent(&key)?;
                        a.get(&k).and_then(|c| field_get(c, rest, keys))
                    }
                    Zval::Str(s) if rest.is_empty() => {
                        string_offset(s, &key).map(|byte| Zval::Str(PhpStr::new(vec![byte])))
                    }
                    _ => None,
                }
            }
            FieldStep::Append => None,
        },
    }
}

/// Remove a mixed field path's leaf (OOP-2c). A missing intermediate level is a
/// silent no-op; arrays copy-on-write, objects mutate in place.
fn field_unset(target: &mut Zval, steps: &[FieldStep], keys: &mut std::vec::IntoIter<Zval>) {
    if let Zval::Ref(rc) = target {
        field_unset(&mut rc.borrow_mut(), steps, keys);
        return;
    }
    let Some((first, rest)) = steps.split_first() else {
        return;
    };
    match first {
        FieldStep::Prop(name) => {
            if let Zval::Object(o) = target {
                if rest.is_empty() {
                    o.borrow_mut().props.remove(name);
                } else if let Some(child) = o.borrow_mut().props.get_mut(name) {
                    field_unset(child, rest, keys);
                }
            }
        }
        FieldStep::Index => {
            let Some(key) = keys.next() else { return };
            if let Zval::Array(rc) = target {
                if let Some(k) = coerce_key_silent(&key) {
                    let arr = Rc::make_mut(rc);
                    if rest.is_empty() {
                        arr.remove(&k);
                    } else if let Some(child) = arr.get_mut(&k) {
                        field_unset(child, rest, keys);
                    }
                }
            }
        }
        FieldStep::Append => {}
    }
}

/// Invoke a by-reference-first builtin, handing it `&mut Zval` for the slot cell
/// (following a `Zval::Ref` so the write lands in the shared target).
fn builtin_ref_call(
    f: BuiltinRefFn,
    cell: &mut Zval,
    rest: &[Zval],
    out: &mut Vec<u8>,
    diags: &mut Diags,
) -> Result<Zval, PhpError> {
    let mut ctx = Ctx { out, diags };
    if let Zval::Ref(rc) = cell {
        let mut guard = rc.borrow_mut();
        f(&mut guard, rest, &mut ctx)
    } else {
        f(cell, rest, &mut ctx)
    }
}

/// The fatal a call raises when a name isn't a callable VM builtin (defensive:
/// the compiler already filters these, so this is a safety net).
fn undefined_builtin(name: &[u8]) -> PhpError {
    PhpError::Error(format!(
        "Call to undefined function {}()",
        String::from_utf8_lossy(name)
    ))
}

/// Read object property `name` by value (deref-clone), following a reference
/// receiver. A missing property — or a non-object receiver — warns and yields
/// NULL, mirroring the tree-walker's `read_property` (OOP-1 has no `__get` /
/// visibility enforcement).
fn read_property(recv: &Zval, name: &[u8], diags: &mut Diags) -> Zval {
    match recv {
        Zval::Object(o) => {
            let obj = o.borrow();
            if let Some(v) = obj.props.get(name) {
                return v.deref_clone();
            }
            let cls = String::from_utf8_lossy(obj.class_name.as_bytes()).into_owned();
            drop(obj);
            let prop = String::from_utf8_lossy(name).into_owned();
            diags.push(Diag::Warning(format!("Undefined property: {cls}::${prop}")));
            Zval::Null
        }
        Zval::Ref(rc) => read_property(&rc.borrow(), name, diags),
        Zval::Null | Zval::Undef => {
            let prop = String::from_utf8_lossy(name).into_owned();
            diags.push(Diag::Warning(format!("Attempt to read property \"{prop}\" on null")));
            Zval::Null
        }
        other => {
            let prop = String::from_utf8_lossy(name).into_owned();
            diags.push(Diag::Warning(format!(
                "Attempt to read property \"{prop}\" on {}",
                other.error_type_name()
            )));
            Zval::Null
        }
    }
}

/// Write `value` into object property `name` (created if absent), in place through
/// the shared object cell. A non-object receiver is a fatal, matching PHP 8.
fn write_property(recv: &Zval, name: &[u8], value: Zval) -> Result<(), PhpError> {
    match recv {
        Zval::Object(o) => {
            o.borrow_mut().props.set(name, value);
            Ok(())
        }
        Zval::Ref(rc) => write_property(&rc.borrow(), name, value),
        other => Err(PhpError::Error(format!(
            "Attempt to assign property \"{}\" on {}",
            String::from_utf8_lossy(name),
            other.error_type_name()
        ))),
    }
}

/// `isset($o->name)`: true iff the property exists and is not null/undefined
/// (silent), following a reference receiver.
fn prop_isset(recv: &Zval, name: &[u8]) -> bool {
    match recv {
        Zval::Object(o) => match o.borrow().props.get(name) {
            Some(v) => !matches!(v.deref_clone(), Zval::Null | Zval::Undef),
            None => false,
        },
        Zval::Ref(rc) => prop_isset(&rc.borrow(), name),
        _ => false,
    }
}

/// `unset($o->name)`: remove the property (no-op if absent or non-object).
fn prop_unset(recv: &Zval, name: &[u8]) {
    match recv {
        Zval::Object(o) => {
            o.borrow_mut().props.remove(name);
        }
        Zval::Ref(rc) => prop_unset(&rc.borrow(), name),
        _ => {}
    }
}

/// Resolve a method by name at run time, walking the receiver class's `parent`
/// chain child→ancestor (case-insensitive). Returns the *defining* class id and
/// the method's index in [`crate::bytecode::CompiledClass::methods`].
fn resolve_method_runtime(module: &Module, start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut cid = Some(start);
    while let Some(c) = cid {
        if let Some(i) = module.classes[c]
            .methods
            .iter()
            .position(|m| m.name.eq_ignore_ascii_case(name))
        {
            return Some((c, i));
        }
        cid = module.classes[c].parent;
    }
    None
}

/// The class id of an object value (following a reference), or `None` for a
/// non-object.
fn object_class_id(v: &Zval) -> Option<ClassId> {
    match v {
        Zval::Object(o) => Some(o.borrow().class_id as usize),
        Zval::Ref(rc) => object_class_id(&rc.borrow()),
        _ => None,
    }
}

/// Whether class `a` is `b` or descends from it (parent chain only) — the test
/// behind forwarding `$this` propagation for `Parent::m()`-style calls.
fn class_is_a(module: &Module, a: ClassId, b: ClassId) -> bool {
    let mut cur = Some(a);
    while let Some(c) = cur {
        if c == b {
            return true;
        }
        cur = module.classes[c].parent;
    }
    false
}

/// Resolve a class constant at run time (for `static::CONST`): own constants and
/// parent chain first, then interfaces transitively. Returns the declaring class
/// id and the constant's index. Case-sensitive, like PHP and the compiler's
/// `find_class_const`.
fn find_const_runtime(module: &Module, start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut c = Some(start);
    while let Some(x) = c {
        if let Some(i) = module.classes[x].consts.iter().position(|k| k.name.as_ref() == name) {
            return Some((x, i));
        }
        c = module.classes[x].parent;
    }
    let mut c = Some(start);
    while let Some(x) = c {
        for &i in &module.classes[x].interfaces {
            if let Some(r) = find_const_runtime(module, i, name) {
                return Some(r);
            }
        }
        c = module.classes[x].parent;
    }
    None
}

/// Pack call arguments into a 0-indexed list array — the second argument handed
/// to `__call` / `__callStatic` (OOP-3a), mirroring the tree-walker's `pack_args`.
fn pack_args(args: Vec<Zval>) -> Zval {
    let mut arr = PhpArray::new();
    for a in args {
        let _ = arr.append(a);
    }
    Zval::Array(Rc::new(arr))
}

/// The "call to undefined method" fatal, shared by instance and static dispatch.
fn undefined_method(module: &Module, cid: ClassId, method: &[u8]) -> PhpError {
    PhpError::Error(format!(
        "Call to undefined method {}::{}()",
        String::from_utf8_lossy(&module.classes[cid].name),
        String::from_utf8_lossy(method)
    ))
}

/// Whether a member of visibility `vis` declared on `decl` is accessible from the
/// running frame's class `cur` (OOP-2b), mirroring the tree-walker's
/// `visible_from`: public always; private only from the declaring class;
/// protected from anywhere in the same hierarchy.
fn visible_from(module: &Module, cur: Option<ClassId>, vis: Visibility, decl: ClassId) -> bool {
    match vis {
        Visibility::Public => true,
        Visibility::Private => cur == Some(decl),
        Visibility::Protected => matches!(
            cur,
            Some(cc) if class_is_a(module, cc, decl) || class_is_a(module, decl, cc)
        ),
    }
}

/// Resolve a declared instance property's visibility and declaring class by
/// walking `class`'s parent chain child→ancestor. `None` for a dynamic /
/// undeclared property (effectively public).
fn resolve_prop_decl(module: &Module, class: ClassId, name: &[u8]) -> Option<(Visibility, ClassId)> {
    let mut cid = Some(class);
    while let Some(c) = cid {
        if let Some((_, vis)) = module.classes[c].own_prop_vis.iter().find(|(n, _)| n.as_ref() == name) {
            return Some((*vis, c));
        }
        cid = module.classes[c].parent;
    }
    None
}

/// Resolve a static property to its declaring class and index, walking the parent
/// chain (OOP-2b).
fn find_static_prop(module: &Module, start: ClassId, name: &[u8]) -> Option<(ClassId, usize)> {
    let mut cid = Some(start);
    while let Some(c) = cid {
        if let Some(i) = module.classes[c].static_props.iter().position(|p| p.name.as_ref() == name) {
            return Some((c, i));
        }
        cid = module.classes[c].parent;
    }
    None
}

/// Enforce instance-property visibility for an access from frame class `cur` on an
/// object of `obj_class`. A dynamic / undeclared property is always accessible.
fn check_prop_access(
    module: &Module,
    cur: Option<ClassId>,
    obj_class: ClassId,
    name: &[u8],
) -> Result<(), PhpError> {
    if let Some((vis, decl)) = resolve_prop_decl(module, obj_class, name) {
        if !visible_from(module, cur, vis, decl) {
            return Err(prop_access_error(module, decl, name, vis));
        }
    }
    Ok(())
}

/// The "Cannot access {private,protected} property C::$p" fatal.
fn prop_access_error(module: &Module, decl: ClassId, name: &[u8], vis: Visibility) -> PhpError {
    let kind = if matches!(vis, Visibility::Private) { "private" } else { "protected" };
    PhpError::Error(format!(
        "Cannot access {kind} property {}::${}",
        String::from_utf8_lossy(&module.classes[decl].name),
        String::from_utf8_lossy(name)
    ))
}

/// The "Call to {private,protected} method C::m() from <scope>" fatal.
fn method_access_error(
    module: &Module,
    decl: ClassId,
    method: &[u8],
    cur: Option<ClassId>,
    vis: Visibility,
) -> PhpError {
    let kind = if matches!(vis, Visibility::Private) { "private" } else { "protected" };
    let scope = match cur {
        Some(c) => format!("scope {}", String::from_utf8_lossy(&module.classes[c].name)),
        None => "global scope".to_string(),
    };
    PhpError::Error(format!(
        "Call to {kind} method {}::{}() from {scope}",
        String::from_utf8_lossy(&module.classes[decl].name),
        String::from_utf8_lossy(method)
    ))
}

/// Whether an object of `class_id` is an instance of `target`: the class itself,
/// any ancestor, or any implemented interface (transitively), mirroring the
/// tree-walker's `is_instance_of` (OOP-1 omits the `Stringable` auto-impl).
fn is_instance_of(module: &Module, class_id: ClassId, target: ClassId) -> bool {
    let mut cur = Some(class_id);
    while let Some(c) = cur {
        if c == target {
            return true;
        }
        if module.classes[c].interfaces.iter().any(|&i| iface_is_a(module, i, target)) {
            return true;
        }
        cur = module.classes[c].parent;
    }
    false
}

/// Whether interface `i` is, or transitively extends, `target`.
fn iface_is_a(module: &Module, i: ClassId, target: ClassId) -> bool {
    if i == target {
        return true;
    }
    module.classes[i].interfaces.iter().any(|&p| iface_is_a(module, p, target))
}

/// ASCII-case-insensitive byte-string equality — PHP resolves function names
/// case-insensitively in ASCII (mirrors the compiler's resolution).
fn name_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

/// Bind positional `args` to a callee frame's leading parameter slots (PAR).
/// Omitted parameters are left `Undef` for the body's default prologue
/// ([`Op::FillDefault`]) to fill. For a **variadic** function the leading fixed
/// params are bound and every remaining argument is collected into an array in
/// the variadic slot (empty when there are none); otherwise surplus arguments
/// are dropped (PHP silently ignores them for a non-variadic function).
fn bind_params(frame: &mut Frame, args: Vec<Zval>) {
    frame.argc = args.len() as u32;
    match frame.func.variadic_slot {
        None => {
            let n = frame.func.n_params as usize;
            for (i, a) in args.into_iter().enumerate() {
                if i < n {
                    frame.slots[i] = a;
                }
            }
        }
        Some(vslot) => {
            let v = vslot as usize;
            let mut it = args.into_iter();
            for slot in frame.slots.iter_mut().take(v) {
                match it.next() {
                    Some(a) => *slot = a,
                    None => break, // omitted fixed params stay Undef (default prologue)
                }
            }
            let mut rest = PhpArray::new();
            for a in it {
                let _ = rest.append(a);
            }
            frame.slots[v] = Zval::Array(Rc::new(rest));
        }
    }
}

/// Read a local cell's value, following a reference and mapping an unset slot to
/// NULL.
fn read_slot(cell: &Zval) -> Zval {
    match cell {
        Zval::Undef => Zval::Null,
        Zval::Ref(r) => r.borrow().clone(),
        other => other.clone(),
    }
}

/// Coerce an index value to an array [`Key`] without raising diagnostics — the
/// proof slice reads and writes silently. Mirrors `eval::coerce_key` minus the
/// deprecation/warning pushes; `None` marks an illegal offset type
/// (array/object/closure/generator/resource).
fn coerce_key_silent(v: &Zval) -> Option<Key> {
    match v {
        Zval::Long(i) => Some(Key::Int(*i)),
        Zval::Bool(b) => Some(Key::Int(*b as i64)),
        Zval::Double(d) => Some(Key::Int(convert::dval_to_lval(*d))),
        Zval::Str(s) => Some(Key::from_zstr(s)),
        Zval::Null | Zval::Undef => Some(Key::from_bytes(b"")),
        Zval::Ref(c) => coerce_key_silent(&c.borrow()),
        _ => None,
    }
}

/// Snapshot an iterable into `(key, value)` pairs for `foreach`. An array (or a
/// reference to one) is copied element-wise — by-value `foreach` iterates this
/// snapshot, so the body mutating the source can't disturb the loop. Any other
/// value iterates zero times for now (object / Traversable support is OOP work).
///
/// Element values are cloned *shallowly* (`v.clone()`), so a reference element
/// keeps sharing its cell and is read live at bind time (see `IterNext`). This
/// is what reproduces PHP's lingering-reference gotcha — a `foreach (… as &$v)`
/// followed by `foreach (… as $v)` mutates the last element (D-R13) — and
/// mirrors the tree-walker (`eval::exec_foreach`).
fn snapshot_entries(iterable: &Zval) -> Vec<(Zval, Zval)> {
    match iterable {
        Zval::Array(a) => a.iter().map(|(k, v)| (key_to_zval(k), v.clone())).collect(),
        Zval::Ref(rc) => snapshot_entries(&rc.borrow()),
        _ => Vec::new(),
    }
}

/// Materialise an array [`Key`] as the [`Zval`] `foreach` binds to its key slot.
fn key_to_zval(k: &Key) -> Zval {
    match k {
        Key::Int(i) => Zval::Long(*i),
        Key::Str(s) => Zval::Str(Rc::clone(s)),
    }
}

/// Read `base[key]` by value (silent). Array elements deref-clone; a string base
/// reads a byte offset; anything else (or a missing key) yields NULL.
fn read_dim(base: &Zval, key: &Zval) -> Zval {
    match base {
        Zval::Array(a) => match coerce_key_silent(key) {
            Some(k) => a.get(&k).map(|v| v.deref_clone()).unwrap_or(Zval::Null),
            None => Zval::Null,
        },
        Zval::Str(s) => read_string_offset(s, key),
        Zval::Ref(rc) => read_dim(&rc.borrow(), key),
        _ => Zval::Null,
    }
}

/// String byte-offset read `$s[i]` (silent): integer index, negatives count from
/// the end, out-of-range yields `""`.
fn read_string_offset(s: &PhpStr, key: &Zval) -> Zval {
    match string_offset(s, key) {
        Some(byte) => Zval::Str(PhpStr::new(vec![byte])),
        None => Zval::Str(PhpStr::new(Vec::new())),
    }
}

/// Ensure `cell` is an array, auto-vivifying from null/undefined/false; a
/// non-empty scalar cannot become an array.
fn ensure_array(cell: &mut Zval) -> Result<(), PhpError> {
    match cell {
        Zval::Undef | Zval::Null | Zval::Bool(false) => {
            *cell = Zval::Array(Rc::new(PhpArray::new()));
            Ok(())
        }
        Zval::Array(_) => Ok(()),
        _ => Err(PhpError::Error(
            "Cannot use a scalar value as an array".to_string(),
        )),
    }
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

/// Navigate `steps` (only `Index` steps reach references — the compiler rejects
/// `Prop`/`Append`) from `target`, auto-vivifying missing elements as NULL, and
/// promote the addressed leaf to a shared `Zval::Ref`, returning its cell
/// (REF-4). Mirrors `eval::place_cell`: a reference is followed into its cell; a
/// scalar that cannot be indexed yields a detached cell so the caller does not
/// crash. `Index` steps consume `keys` in source order.
fn field_cell(
    target: &mut Zval,
    steps: &[FieldStep],
    keys: &mut std::vec::IntoIter<Zval>,
) -> Rc<RefCell<Zval>> {
    let Some((_first, rest)) = steps.split_first() else {
        return make_cell(target);
    };
    if let Zval::Ref(rc) = target {
        let inner = &mut *rc.borrow_mut();
        return field_cell(inner, steps, keys);
    }
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

#[cfg(test)]
mod tests {
    use crate::builtin::{Builtin, Ctx, Registry};
    use crate::compile::compile_program;
    use crate::lower::lower_source;
    use php_types::{Diag, PhpError, Zval};

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

    fn fake_registry() -> Registry {
        let mut r = Registry::new();
        r.insert(b"t_double".to_vec(), Builtin::Value(t_double));
        r.insert(b"t_emit".to_vec(), Builtin::Value(t_emit));
        r.insert(b"t_warn".to_vec(), Builtin::Value(t_warn));
        r.insert(b"t_set42".to_vec(), Builtin::RefFirst(t_set42));
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

    // ----- REF-2: by-reference parameters (user functions) -----

    #[test]
    fn by_ref_param_mutates_caller() {
        assert_eq!(
            vm_stdout(b"<?php function inc(&$x) { $x = $x + 1; } $n = 5; inc($n); echo $n;"),
            b"6"
        );
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
}

