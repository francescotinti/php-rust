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

use php_types::{convert, ops, Diag, Diags, Key, Object, PhpArray, PhpError, PhpStr, Props, Zval};

use crate::builtin::{Builtin, BuiltinRefFn, Ctx, Registry};
use crate::bytecode::{
    ClassTarget, DimBase, FieldBase, FieldStep, Func, Instantiable, Module, Op, StaticInit,
};
use crate::hir::{BinOp, CastKind, ClassId, UnOp, Visibility};

/// The result of running a [`Module`]: the bytes written to stdout, the
/// diagnostics raised, and the fatal that stopped execution (if any).
#[derive(Debug, Default)]
pub struct VmOutcome {
    pub stdout: Vec<u8>,
    pub diags: Diags,
    pub fatal: Option<PhpError>,
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
        }
    }
}

/// A `foreach` iteration snapshot: the (key, value) pairs captured at loop entry
/// and the cursor into them.
struct IterState {
    entries: Vec<(Zval, Zval)>,
    pos: usize,
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
}

impl Vm<'_> {
    /// Allocate a fresh object handle id.
    fn next_id(&mut self) -> u32 {
        let id = self.next_object_id;
        self.next_object_id += 1;
        id
    }

    /// Run until the bottom frame returns, yielding the script's result value.
    fn run(&mut self) -> Result<Zval, PhpError> {
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
                    let r = apply_cast(k, &a, &mut self.diags);
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
                Op::IterInit => {
                    let iterable = self.frames[top].stack.pop().expect("IterInit iterable");
                    self.frames[top].iters.push(IterState {
                        entries: snapshot_entries(&iterable),
                        pos: 0,
                    });
                }
                Op::IterNext { value, key, end } => {
                    let it = self.frames[top].iters.last().expect("IterNext without iterator");
                    if it.pos >= it.entries.len() {
                        self.frames[top].ip = end as usize;
                    } else {
                        let (k, v) = self.frames[top].iters.last().unwrap().entries[it.pos].clone();
                        self.frames[top].iters.last_mut().unwrap().pos += 1;
                        store_slot(&mut self.frames[top].slots[value as usize], v);
                        if let Some(ks) = key {
                            store_slot(&mut self.frames[top].slots[ks as usize], k);
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
                    for (i, a) in args.into_iter().enumerate() {
                        frame.slots[i] = a;
                    }
                    self.frames.push(frame);
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
                        match self.frames.last_mut() {
                            Some(caller) => caller.stack.push(v),
                            None => return Ok(v),
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
                    let module = self.module;
                    let mut args = self.pop_keys(top, argc); // source order
                    let recv = self.frames[top].stack.pop().expect("MethodCall receiver");
                    let this = recv.deref_clone();
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
                    let resolved = resolve_method_runtime(module, cid, &method);
                    // Usable only if found *and* visible from the caller's scope.
                    let usable = resolved.filter(|&(defc, midx)| {
                        visible_from(module, self.frames[top].class, module.classes[defc].methods[midx].visibility, defc)
                    });
                    match usable {
                        Some((defc, midx)) => {
                            let callee = &module.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee);
                            for (i, a) in args.drain(..).enumerate() {
                                frame.slots[i] = a;
                            }
                            frame.this = Some(this);
                            frame.class = Some(defc);
                            frame.static_class = Some(cid); // LSB = receiver's actual class
                            self.frames.push(frame);
                        }
                        // Missing or inaccessible: route to `__call` if defined,
                        // else the original fatal (visibility / undefined method).
                        None => match resolve_method_runtime(module, cid, b"__call") {
                            Some((cdefc, cmidx)) => {
                                self.push_magic_call(cdefc, cmidx, Some(this), cid, &method, args);
                            }
                            None => {
                                return Err(match resolved {
                                    Some((defc, midx)) => method_access_error(
                                        module,
                                        defc,
                                        &method,
                                        self.frames[top].class,
                                        module.classes[defc].methods[midx].visibility,
                                    ),
                                    None => undefined_method(module, cid, &method),
                                })
                            }
                        },
                    }
                }
                Op::InvokeMethod { class, method_idx, argc } => {
                    let module = self.module;
                    let mut args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeMethod receiver");
                    let this = recv.deref_clone();
                    let lsb = object_class_id(&this).unwrap_or(class);
                    let callee = &module.classes[class].methods[method_idx as usize].func;
                    let mut frame = Frame::new(callee);
                    for (i, a) in args.drain(..).enumerate() {
                        frame.slots[i] = a;
                    }
                    frame.this = Some(this);
                    frame.class = Some(class);
                    frame.static_class = Some(lsb);
                    self.frames.push(frame);
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
                Op::StaticCall { target, method, forwarding, argc } => {
                    let module = self.module;
                    let mut args = self.pop_keys(top, argc);
                    let start = match target {
                        ClassTarget::Class(cid) => cid,
                        ClassTarget::Static => self.frames[top].static_class.ok_or_else(|| {
                            PhpError::Error("Cannot use \"static\" outside class context".to_string())
                        })?,
                    };
                    let resolved = resolve_method_runtime(module, start, &method);
                    let usable = resolved.filter(|&(defc, midx)| {
                        visible_from(module, self.frames[top].class, module.classes[defc].methods[midx].visibility, defc)
                    });
                    // LSB: a forwarding call (self/parent/static) keeps the caller's;
                    // a named call rebinds it to the start class.
                    let static_class = if forwarding {
                        self.frames[top].static_class.unwrap_or(start)
                    } else {
                        start
                    };
                    // `$this` is forwarded for a forwarding call, or for a named
                    // call to a class in the current object's hierarchy.
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
                            for (i, a) in args.drain(..).enumerate() {
                                frame.slots[i] = a;
                            }
                            frame.this = this;
                            frame.class = Some(defc);
                            frame.static_class = Some(static_class);
                            self.frames.push(frame);
                        }
                        None => {
                            // In object context (a `$this` in the hierarchy) a
                            // missing/inaccessible static target routes to `__call`
                            // on `$this`; otherwise to `__callStatic` on the class.
                            let via_call = this
                                .as_ref()
                                .and_then(|t| object_class_id(t).map(|oc| (t.clone(), oc)))
                                .and_then(|(tv, oc)| {
                                    resolve_method_runtime(module, oc, b"__call").map(|(d, m)| (tv, oc, d, m))
                                });
                            if let Some((tv, oc, cdefc, cmidx)) = via_call {
                                self.push_magic_call(cdefc, cmidx, Some(tv), oc, &method, args);
                            } else if let Some((cdefc, cmidx)) =
                                resolve_method_runtime(module, start, b"__callStatic")
                            {
                                self.push_magic_call(cdefc, cmidx, None, start, &method, args);
                            } else {
                                return Err(match resolved {
                                    Some((defc, midx)) => method_access_error(
                                        module,
                                        defc,
                                        &method,
                                        self.frames[top].class,
                                        module.classes[defc].methods[midx].visibility,
                                    ),
                                    None => undefined_method(module, start, &method),
                                });
                            }
                        }
                    }
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
                Op::ClassNameStatic => {
                    let start = self.frames[top].static_class.ok_or_else(|| {
                        PhpError::Error("Cannot use \"static\" outside class context".to_string())
                    })?;
                    let name = self.module.classes[start].name.to_vec();
                    self.frames[top].stack.push(Zval::Str(PhpStr::new(name)));
                }
                Op::InvokeCtor { argc } => {
                    let module = self.module;
                    let mut args = self.pop_keys(top, argc);
                    let recv = self.frames[top].stack.pop().expect("InvokeCtor receiver");
                    let this = recv.deref_clone();
                    let cid = object_class_id(&this).expect("InvokeCtor on a non-object");
                    match resolve_method_runtime(module, cid, b"__construct") {
                        Some((defc, midx)) => {
                            let callee = &module.classes[defc].methods[midx].func;
                            let mut frame = Frame::new(callee);
                            for (i, a) in args.drain(..).enumerate() {
                                frame.slots[i] = a;
                            }
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
fn snapshot_entries(iterable: &Zval) -> Vec<(Zval, Zval)> {
    match iterable {
        Zval::Array(a) => a.iter().map(|(k, v)| (key_to_zval(k), v.deref_clone())).collect(),
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
            arr.insert(k, value.clone());
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
            arr.insert(k, result.clone());
            Ok(result)
        }
        Last::IncDec { key, inc, pre } => {
            let k = coerce_key_silent(&key)
                .ok_or_else(|| PhpError::TypeError("Illegal offset type".to_string()))?;
            if !arr.contains_key(&k) {
                arr.insert(k.clone(), Zval::Null);
            }
            let cell = arr.get_mut(&k).expect("just inserted");
            let old = cell.clone();
            if inc {
                ops::increment(cell, diags)?;
            } else {
                ops::decrement(cell, diags)?;
            }
            Ok(if pre { cell.clone() } else { old })
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
        CastKind::String => Zval::Str(convert::to_zstr_cast(a, d)),
        CastKind::Bool => Zval::Bool(convert::to_bool(a, d)),
        // The compiler only emits Int/String/Bool casts in the proof slice.
        other => unreachable!("VM saw an unported cast {other:?}"),
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
}
