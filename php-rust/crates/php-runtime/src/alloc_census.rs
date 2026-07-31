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
//! NOTE (A-DL10): the counting allocator books `realloc` at FULL size and
//! never nets deallocs — every figure in these channels is GROSS churn, an
//! upper bound (the census line carries `gross=1` to say so in-band).
//!
//! S-80.0.3 integrity hardening (Council WP-81 A-MS7/A-MS11/A-TH12):
//! - both window types are RAII and `!Send` (compile-time, KS-MS-81-1): a
//!   guard dropped on a foreign thread would pop the WRONG thread's stack;
//! - a3 windows carry a LIFO depth token: an out-of-order drop can no longer
//!   silently swap deltas between windows;
//! - a provider installed MID-window makes the closing snapshot whole-process
//!   history, not a delta — the booking is discarded;
//! - every such violation bumps the process-wide SPLIT_TRIP counter, emitted
//!   on the census line as `a3_trip=`; the measure driver VOIDs any run with
//!   a nonzero value (KS-MS-81-2) — corruption is loud, never absorbed.

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Cumulative (alloc_calls, alloc_bytes) provider — e.g. php-server's
/// `census_alloc::snapshot`. Installed once; `set` is idempotent-safe via
/// `OnceLock` semantics (later installs are ignored).
pub static SNAPSHOT_FN: std::sync::OnceLock<fn() -> (u64, u64)> = std::sync::OnceLock::new();

#[inline]
fn snap() -> (u64, u64) {
    SNAPSHOT_FN.get().map(|f| f()).unwrap_or((0, 0))
}

#[inline]
fn provider_installed() -> bool {
    SNAPSHOT_FN.get().is_some()
}

/// S-80.0.3 split-channel tripwire (KS-MS-81-2). Monotonic, process-wide:
/// the driver's per-run check is "ever nonzero ⇒ run VOID", so a cumulative
/// count is exactly the right shape (it can only stay 0 by never tripping).
static SPLIT_TRIP: AtomicU64 = AtomicU64::new(0);

pub fn trip_count() -> u64 {
    SPLIT_TRIP.load(Ordering::Relaxed)
}

#[cold]
fn trip() {
    SPLIT_TRIP.fetch_add(1, Ordering::Relaxed);
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
static PRELUDE_FN_COUNT: AtomicUsize = AtomicUsize::new(0);
static PRELUDE_CLASS_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn note_prelude_counts(fns: usize, classes: usize) {
    PRELUDE_FN_COUNT.store(fns, Ordering::Relaxed);
    PRELUDE_CLASS_COUNT.store(classes, Ordering::Relaxed);
}

pub fn prelude_fn_count() -> usize {
    PRELUDE_FN_COUNT.load(Ordering::Relaxed)
}

pub fn prelude_class_count() -> usize {
    PRELUDE_CLASS_COUNT.load(Ordering::Relaxed)
}

/// RAII a1 (prelude) window — S-80.0.3 (A-MS11): the manual open/close pair
/// lost the window on any early return between the two calls, silently
/// inflating a2 (derived as a − a1) on exactly the fatal paths fixture F8
/// exercises. `close()` books at the prefix boundary; Drop is the
/// tail/backstop on every other path. a1 windows never nest: the bracketed
/// segments (lower_prelude, the prelude prefix of the main compile loops)
/// are straight-line code with no include/eval re-entry.
pub struct A1Window {
    start: Option<(u64, u64)>,
    installed: bool,
    _not_send: PhantomData<*const ()>,
}

impl A1Window {
    pub fn open() -> Self {
        A1Window {
            start: Some(snap()),
            installed: provider_installed(),
            _not_send: PhantomData,
        }
    }

    /// Book the window now (prefix boundary reached). Idempotent — the later
    /// Drop becomes a no-op.
    pub fn close(&mut self) {
        let Some(s) = self.start.take() else { return };
        // Provider guard (A-MS11): installed mid-window ⇒ the end snapshot is
        // the whole process history, not a delta. Discard, count, stay loud.
        if !self.installed && provider_installed() {
            trip();
            return;
        }
        let e = snap();
        A1.with(|c| {
            let (ac, ab) = c.get();
            c.set((
                ac + e.0.saturating_sub(s.0),
                ab + e.1.saturating_sub(s.1),
            ));
        });
    }
}

impl Drop for A1Window {
    fn drop(&mut self) {
        self.close();
    }
}

/// RAII a3 (include-compile) window: the fresh path of `run_include` has
/// early failure returns (open failed, lower failed, compile failed) and a
/// stack-leaked window would corrupt every later delta — Drop closes it on
/// every path. S-80.0.3 (A-MS7): carries its LIFO depth token; a drop that
/// is not the current stack top trips instead of swapping deltas.
pub struct A3Window {
    /// Stack length right after our push — the LIFO token.
    depth: usize,
    installed: bool,
    _not_send: PhantomData<*const ()>,
}

impl A3Window {
    pub fn open() -> Self {
        let s = snap();
        let installed = provider_installed();
        let depth = A3_STACK.with(|st| {
            let mut st = st.borrow_mut();
            st.push([s.0, s.1, 0, 0]);
            st.len()
        });
        A3Window {
            depth,
            installed,
            _not_send: PhantomData,
        }
    }
}

impl Drop for A3Window {
    fn drop(&mut self) {
        let e = snap();
        A3_STACK.with(|st| {
            let mut st = st.borrow_mut();
            if st.len() != self.depth {
                // Out-of-order drop (or a foreign pop): booking would swap
                // deltas between windows. Discard down to below our frame so
                // later windows stay coherent, and count the violation.
                trip();
                st.truncate(self.depth.saturating_sub(1));
                return;
            }
            let Some([sc, sb, nc, nb]) = st.pop() else {
                trip();
                return;
            };
            if !self.installed && provider_installed() {
                // Provider guard (A-MS11) — same contract as A1Window.
                trip();
                return;
            }
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

// KS-MS-81-1 (Council WP-81): a thread-affine guard that auto-derives
// Send/Sync is a REJECT — these operate on thread-local stacks, and a
// cross-thread drop would corrupt the wrong thread's accounting. Same
// compile-time form as the RetainSet assert.
static_assertions::assert_not_impl_any!(A1Window: Send, Sync);
static_assertions::assert_not_impl_any!(A3Window: Send, Sync);

/// Read-and-reset the per-request split: ((a1_calls, a1_bytes),
/// (a3_calls, a3_bytes)). Called once per request by the census line
/// emitter — and by the S-80.0.3 drain guard on every non-emitting exit
/// (fatal early-returns), so a failed request's residue can never pollute
/// the next request's line (A-PP11/KS-PP-81-1). The driver's
/// expected-lines==N assert (A-PP5) still flags the missing line itself.
pub fn take_split() -> ((u64, u64), (u64, u64)) {
    let a1 = A1.with(|c| c.replace((0, 0)));
    let a3 = A3.with(|c| c.replace((0, 0)));
    (a1, a3)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The tripwire tests mutate the process-wide SPLIT_TRIP; they assert on
    // DELTAS so they compose with each other and with any future test.

    /// S-80.0.3 negative control: an out-of-order (non-LIFO) drop must TRIP,
    /// not silently swap deltas between windows (A-MS7).
    #[test]
    fn a3_window_non_lifo_drop_trips() {
        let before = trip_count();
        let w1 = A3Window::open();
        let w2 = A3Window::open();
        drop(w1); // not the stack top — must trip and discard
        let mid = trip_count();
        assert!(mid > before, "non-LIFO drop did not trip (A-MS7)");
        drop(w2); // its frame was discarded by the recovery — trips too
        assert!(trip_count() > mid, "orphaned window drop did not trip");
    }

    /// S-80.0.3: pop-on-empty (foreign interference) trips.
    #[test]
    fn a3_window_pop_on_empty_trips() {
        let w = A3Window::open();
        A3_STACK.with(|st| st.borrow_mut().clear());
        let before = trip_count();
        drop(w);
        assert!(trip_count() > before, "pop-on-empty did not trip");
    }

    /// A1Window closes exactly once: close() books, the later Drop no-ops —
    /// and a well-ordered open/close/drop sequence must NOT trip.
    #[test]
    fn a1_window_close_is_idempotent_and_clean() {
        let before_trip = trip_count();
        let mut w = A1Window::open();
        w.close();
        drop(w); // must not book (or trip) a second time
        assert_eq!(
            trip_count(),
            before_trip,
            "a clean close/drop sequence tripped the split tripwire"
        );
    }
}
