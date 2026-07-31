//! S-79.0.3 (Council WP-80, KH80-2/KB-80-1/KS-SK-80-2): alloc sub-channel
//! attribution for the compile phase. The aggregate "phase a" of design78 is
//! a BED, not a channel (A-TH2): on a minimal fixture it is dominated by the
//! PRELUDE, and a lever aimed at the main script must be judged on the
//! sub-channel it actually attacks. Split:
//!   - a1 = prelude lower+compile on the MAIN path (fresh, per request)
//!   - a2 = main-script proper (derived: a_total − a1, computed by the census
//!     line emitter — never a counter here)
//!   - a3 = include/eval unit lower+compile on a unit-cache MISS (the exact
//!     work a cache hit skips)
//!
//! The SAPI that owns the counting allocator (php-server's census build)
//! installs the snapshot provider at process start; without it every window
//! reads (0,0) and the split degrades to zeros — absent, never wrong.
//! Accumulators are thread-local: each worker attributes its own traffic,
//! and the per-request reader (`take_split`) runs on the worker thread.

use std::cell::{Cell, RefCell};

/// Cumulative (alloc_calls, alloc_bytes) provider — e.g. php-server's
/// `census_alloc::snapshot`. Installed once; `set` is idempotent-safe via
/// `OnceLock` semantics (later installs are ignored).
pub static SNAPSHOT_FN: std::sync::OnceLock<fn() -> (u64, u64)> = std::sync::OnceLock::new();

#[inline]
fn snap() -> (u64, u64) {
    SNAPSHOT_FN.get().map(|f| f()).unwrap_or((0, 0))
}

thread_local! {
    /// a1 accumulator (calls, bytes) since the last `take_split`.
    static A1: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
    /// a3 accumulator (calls, bytes) since the last `take_split`.
    static A3: Cell<(u64, u64)> = const { Cell::new((0, 0)) };
    /// Open a3 windows: [start_calls, start_bytes, nested_calls, nested_bytes].
    /// Windows nest (an autoload fired mid-lower runs a whole nested include);
    /// each level books its delta NET of inner windows so Σa3 counts every
    /// byte exactly once.
    static A3_STACK: RefCell<Vec<[u64; 4]>> = const { RefCell::new(Vec::new()) };
}

/// Prelude table sizes, noted by the first main-path lowering. The compile
/// side classifies its leading fn/class indices below these counts as
/// prelude bodies (the lowering seeds the prelude at the FRONT of both
/// tables, so the prefix is contiguous by construction).
static PRELUDE_FN_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static PRELUDE_CLASS_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

pub fn note_prelude_counts(fns: usize, classes: usize) {
    PRELUDE_FN_COUNT.store(fns, std::sync::atomic::Ordering::Relaxed);
    PRELUDE_CLASS_COUNT.store(classes, std::sync::atomic::Ordering::Relaxed);
}

pub fn prelude_fn_count() -> usize {
    PRELUDE_FN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn prelude_class_count() -> usize {
    PRELUDE_CLASS_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}

/// Open an a1 (prelude) window. a1 windows never nest: the bracketed
/// segments (lower_prelude, the prelude prefix of the main compile loops)
/// are straight-line code with no include/eval re-entry.
pub fn a1_open() -> (u64, u64) {
    snap()
}

pub fn a1_close(s: (u64, u64)) {
    let e = snap();
    A1.with(|c| {
        let (ac, ab) = c.get();
        c.set((
            ac + e.0.saturating_sub(s.0),
            ab + e.1.saturating_sub(s.1),
        ));
    });
}

/// RAII a3 (include-compile) window: the fresh path of `run_include` has
/// early failure returns (open failed, lower failed, compile failed) and a
/// stack-leaked window would corrupt every later delta — Drop closes it on
/// every path.
pub struct A3Window {
    _priv: (),
}

impl A3Window {
    pub fn open() -> Self {
        let s = snap();
        A3_STACK.with(|st| st.borrow_mut().push([s.0, s.1, 0, 0]));
        A3Window { _priv: () }
    }
}

impl Drop for A3Window {
    fn drop(&mut self) {
        let e = snap();
        A3_STACK.with(|st| {
            let mut st = st.borrow_mut();
            let Some([sc, sb, nc, nb]) = st.pop() else { return };
            let gross = (e.0.saturating_sub(sc), e.1.saturating_sub(sb));
            let own = (gross.0.saturating_sub(nc), gross.1.saturating_sub(nb));
            A3.with(|c| {
                let (ac, ab) = c.get();
                c.set((ac + own.0, ab + own.1));
            });
            if let Some(parent) = st.last_mut() {
                parent[2] += gross.0;
                parent[3] += gross.1;
            }
        });
    }
}

/// Read-and-reset the per-request split: ((a1_calls, a1_bytes),
/// (a3_calls, a3_bytes)). Called once per request by the census line
/// emitter. An early-fatal request that skips the emitter leaves its
/// residue to the NEXT request's line — bounded and visible; the driver's
/// expected-lines==N assert (A-PP5) flags the missing line itself.
pub fn take_split() -> ((u64, u64), (u64, u64)) {
    let a1 = A1.with(|c| c.replace((0, 0)));
    let a3 = A3.with(|c| c.replace((0, 0)));
    (a1, a3)
}
