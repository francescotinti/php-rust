//! census-time (S-186 fetta 0, braccio C — concilio S-186 Q2, team-sonda):
//! per-class wall-time ESTIMATE of the VM mechanisms, sampled one event in 64
//! with `Instant` (two clock reads per sampled event, ~50 ns, amortised to
//! <1 ns/event), plus plain counts for the fast/general admissions. Classes
//! are NOT disjoint where the mechanism nests (a Ret scope contains the sweep
//! `gc_sweep_after_ret` runs; a host-builtin scope contains any user callable
//! it invokes through a nested `run_loop`) — declared in the criterion, read
//! as upper bounds there. Mirrors the op/gc-census conventions: thread-local
//! storage, `census-time` feature only (never in the pin, never a verdict
//! figure), dump at end of run on STDERR, or appended to the file named by
//! `PHPR_CENSUS_TIME` when it is an absolute path. Line format (TSV):
//! `census-time\t<class>\t<count>\t<sampled>\t<est_ns>`; `est_ns` is the
//! sampled sum scaled by count/sampled (0 when the class only counts).

use std::cell::Cell;
use std::time::Instant;

#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Class {
    /// `Op::Call` arm (fast path + general path), frame push mechanism only.
    CallFast,
    /// General-path admissions of `Op::Call` (count only).
    CallGeneral,
    /// `Op::MethodCall` + `Op::ThisMethodCall` arms (fast + general).
    MCall,
    /// General-path admissions of the method-call ops (count only).
    MCallGeneral,
    /// `Op::Ret` arm (fused path + `ret_slow`), including `gc_sweep_after_ret`.
    RetFast,
    /// `ret_slow` admissions of `Op::Ret` (count only).
    RetSlow,
    /// Property ops (`PropGet*`, `ThisPropGet`, `PropSet*`, `PropDimGetConst`).
    Prop,
    /// String ops (`Concat*`, `Stringify*`).
    Str,
    /// Array ops (`FetchDim*`, `ArrayPush`, `CoalesceFetchDim`, dim stores).
    Arr,
    /// `Op::New` arm: allocation + `gc_track` + constructor frame push.
    New,
    /// Every sweep body (`gc_sweep_impl`: statement, light, after-ret, sync).
    Sweep,
    /// Registry lookup by name inside `Op::CallBuiltin`.
    BuiltinLookup,
    /// `Op::CallBuiltin` arm: lookup + argument pops + body.
    Builtin,
    /// `Op::CallHostBuiltin` arm (may contain nested user code).
    HostBuiltin,
}

const N: usize = 14;
const NAMES: [&str; N] = [
    "call_fast",
    "call_general",
    "mcall",
    "mcall_general",
    "ret_fast",
    "ret_slow",
    "prop",
    "str",
    "arr",
    "new",
    "sweep",
    "builtin_lookup",
    "builtin",
    "hostbuiltin",
];

struct Counters {
    count: [Cell<u64>; N],
    sampled: [Cell<u64>; N],
    nanos: [Cell<u64>; N],
    dumped: Cell<bool>,
}

thread_local! {
    static C: Counters = const {
        Counters {
            count: [const { Cell::new(0) }; N],
            sampled: [const { Cell::new(0) }; N],
            nanos: [const { Cell::new(0) }; N],
            dumped: Cell::new(false),
        }
    };
}

/// A timed scope: the clock is read only on every 64th event of its class.
pub struct Scope {
    class: usize,
    start: Option<Instant>,
}

impl Drop for Scope {
    #[inline]
    fn drop(&mut self) {
        if let Some(t) = self.start {
            let d = t.elapsed().as_nanos() as u64;
            C.with(|c| {
                c.nanos[self.class].set(c.nanos[self.class].get() + d);
                c.sampled[self.class].set(c.sampled[self.class].get() + 1);
            });
        }
    }
}

#[inline]
pub fn scope(class: Class) -> Scope {
    let i = class as usize;
    let n = C.with(|c| {
        let n = c.count[i].get();
        c.count[i].set(n + 1);
        n
    });
    Scope { class: i, start: (n & 63 == 0).then(Instant::now) }
}

/// Count an admission without timing it.
#[inline]
pub fn tick(class: Class) {
    C.with(|c| {
        let i = class as usize;
        c.count[i].set(c.count[i].get() + 1);
    });
}

thread_local! {
    /// S-192 h2 census: WHY an event left the fast path — `(kind, reason) -> count`.
    static H2: std::cell::RefCell<std::collections::BTreeMap<(&'static str, String), u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

fn h2(kind: &'static str, reason: String) {
    H2.with(|m| *m.borrow_mut().entry((kind, reason)).or_insert(0) += 1);
}

/// `ret_slow` admission: counts it and records which conditions of the fused
/// `Op::Ret` test failed (all of them, joined — the test is a conjunction).
pub fn ret_slow(f: &super::Frame<'_>, top: usize) {
    tick(Class::RetSlow);
    let mut r = Vec::new();
    if f.func.ret_shape != 0 {
        r.push(format!("shape{}", f.func.ret_shape));
    }
    if f.flags.bits() != 0 {
        r.push(format!("flags{:#x}", f.flags.bits()));
    }
    if f.ret_cell.is_some() {
        r.push("ret_cell".to_string());
    }
    if !f.iters.is_empty() {
        r.push("iters".to_string());
    }
    if f.ext.is_some() {
        r.push("ext".to_string());
    }
    if f.dyn_vars.is_some() {
        r.push("dyn_vars".to_string());
    }
    if top == 0 {
        r.push("main".to_string());
    }
    h2("ret_slow", r.join("+"));
}

/// General-path admission of `Op::MethodCall`: counts it and records the
/// first condition of `methodcall_fast` that refused the call (the operand
/// stack is still intact when the op falls through).
pub fn mcall_general(vm: &super::Vm<'_>, top: usize, n: usize, ic: &crate::bytecode::MethodIc) {
    tick(Class::MCallGeneral);
    let stack = &vm.frames[top].stack;
    let reason = match stack.len().checked_sub(n + 1).map(|i| &stack[i]) {
        None => "stack".to_string(),
        Some(php_types::Zval::Object(o)) => {
            let cid = o.borrow().class_id as usize;
            match ic.get(cid) {
                None => "ic_miss".to_string(),
                Some((defc, midx)) => {
                    let c = &vm.classes[defc].methods[midx].func;
                    if !c.simple_call {
                        let mut r = vec!["not_simple".to_string()];
                        if c.has_hints {
                            r.push("hint".to_string());
                        }
                        if c.param_by_ref.iter().any(|b| *b) {
                            r.push("byref".to_string());
                        }
                        if c.variadic_slot.is_some() {
                            r.push("variadic".to_string());
                        }
                        if c.is_generator {
                            r.push("generator".to_string());
                        }
                        r.join("+")
                    } else if n < c.n_params as usize {
                        "arity_fewer_args".to_string()
                    } else {
                        "arity_more_args".to_string()
                    }
                }
            }
        }
        Some(php_types::Zval::Ref(_)) => "recv_ref".to_string(),
        Some(_) => "recv_other".to_string(),
    };
    h2("mcall_general", reason);
}

/// General-path admission of `Op::ThisMethodCall` (zero-argument `$this->m()`).
pub fn this_mcall_general(vm: &super::Vm<'_>, top: usize) {
    tick(Class::MCallGeneral);
    let reason = match &vm.frames[top].this {
        Some(php_types::Zval::Object(_)) => "ic_miss",
        Some(_) => "this_not_object",
        None => "no_this",
    };
    h2("this_mcall_general", reason.to_string());
}

/// The `$this` note of a fused method `Ret`: did the S-192 tail short path
/// answer it (`hit`), or did it go down the general note chain?
pub fn note_this(hit: bool) {
    h2("note_this", if hit { "tail_short_path" } else { "general_chain" }.to_string());
}

/// Dump once at end of run (called next to `gc_census::dump`).
pub fn dump() {
    let report = C.with(|c| {
        if c.dumped.replace(true) {
            return None;
        }
        let mut s = String::new();
        for i in 0..N {
            let (count, sampled, nanos) = (c.count[i].get(), c.sampled[i].get(), c.nanos[i].get());
            let est = if sampled > 0 { (nanos as u128 * count as u128 / sampled as u128) as u64 } else { 0 };
            s.push_str(&format!("census-time\t{}\t{count}\t{sampled}\t{est}\n", NAMES[i]));
        }
        H2.with(|m| {
            for ((kind, reason), n) in m.borrow().iter() {
                s.push_str(&format!("census-h2\t{kind}\t{reason}\t{n}\n"));
            }
        });
        Some(s)
    });
    let Some(report) = report else { return };
    match std::env::var("PHPR_CENSUS_TIME") {
        Ok(path) if path.starts_with('/') => {
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                let _ = f.write_all(report.as_bytes());
            }
        }
        _ => eprint!("{report}"),
    }
}
