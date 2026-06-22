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

use std::rc::Rc;

use php_types::{convert, ops, Diags, Key, PhpArray, PhpError, PhpStr, Zval};

use crate::bytecode::{DimBase, Func, Module, Op};
use crate::hir::{BinOp, CastKind, UnOp};

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
pub fn run_module(module: &Module) -> VmOutcome {
    let mut vm = Vm {
        module,
        stdout: Vec::new(),
        diags: Diags::new(),
        frames: Vec::new(),
    };
    vm.frames.push(Frame::new(&module.main));
    let fatal = vm.run().err();
    VmOutcome {
        stdout: vm.stdout,
        diags: vm.diags,
        fatal,
    }
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
    stdout: Vec<u8>,
    diags: Diags,
    frames: Vec<Frame<'m>>,
}

impl Vm<'_> {
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
                Op::Ret => {
                    let ret = self.frames[top].stack.pop().unwrap_or(Zval::Null);
                    self.frames.pop();
                    match self.frames.last_mut() {
                        Some(caller) => caller.stack.push(ret),
                        None => return Ok(ret),
                    }
                }
                Op::Fatal(i) => {
                    let msg = match &self.frames[top].func.consts[i as usize] {
                        crate::bytecode::Const::Str(b) => String::from_utf8_lossy(b).into_owned(),
                        _ => "VM: unsupported construct".to_string(),
                    };
                    return Err(PhpError::Error(msg));
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
    use crate::compile::compile_program;
    use crate::lower::lower_source;

    use super::run_module;

    /// Compile and run a PHP snippet through the bytecode VM, returning stdout.
    fn vm_stdout(src: &[u8]) -> Vec<u8> {
        let program = lower_source(b"test.php", src).expect("lower");
        let module = compile_program(&program).expect("compile");
        let out = run_module(&module);
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        out.stdout
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
}
