//! GC-census instrumentation (WP-39, attribuzione leva E): counters over the
//! note → sweep → cycle-collect pipeline. Mirrors the op-census conventions
//! (WP-33): thread-local storage, armed by `PHPR_GC_CENSUS` (=1 → stderr dump;
//! an absolute path → file append, needed when the workload spawns phpr
//! subprocesses that inherit the env). Behind the `gc-census` feature so the
//! default binary carries zero extra work in the drop-site paths.

use std::cell::RefCell;

#[derive(Default)]
struct GcCensus {
    /// `gc_note` invocations (any Zval, scalars included).
    notes: u64,
    /// Notes that newly buffered an object id (Vacant insert).
    notes_inserted: u64,
    sweeps_main: u64,
    sweeps_light: u64,
    /// WP-50: statement sweeps skipped by the noop fast path ONLY because the
    /// purge-floor raised the effective bound (pressure ≥ threshold but <
    /// max(threshold, floor)) — the band entries the WP-49 lever created.
    /// Conservation identity vs the WP-49 census: light + band_skipped here
    /// must equal the old light count on the same (deterministic) workload.
    sweeps_band_skipped: u64,
    /// Candidates freed (or destructor-scheduled) by refcount at a sweep.
    cand_freed: u64,
    /// Candidates demoted to the cycle-roots buffer (strong_count > 2).
    cand_demoted: u64,
    /// `collect_cycles` invocations / roots examined / garbage found.
    collects: u64,
    collect_roots: u64,
    collect_freed: u64,
    /// Adaptive threshold observed at the last adjustment.
    threshold_last: u64,
    /// `__destruct` frames scheduled by sweeps.
    destructors: u64,
    /// Cumulative wall-time inside `gc_classify` (WP-48 attribution: the
    /// collect-walk share of the full-suite CPU residual).
    classify_ns: u64,
    /// `ho_reflect_method_info` memo traffic (WP-48: is the WP-47 epoch
    /// eviction cap 8192 thrashing — rebuilds = misses after evictions?).
    reflect_hits: u64,
    reflect_misses: u64,
    reflect_evictions: u64,
    /// WP-49 per-collect pattern: `collect_cycles` entries (calls, not
    /// classify rounds — `collects` above counts rounds), split by trigger.
    collect_calls: u64,
    explicit_calls: u64,
    /// Set by `mark_explicit` (host `gc_collect_cycles()`), consumed by the
    /// next `collect_begin`.
    explicit_pending: bool,
    /// Round ordinal within the current `collect_cycles` call.
    round_in_call: u64,
    current_call_explicit: bool,
    /// Object-root recurrence across the whole run: a root id classified in
    /// an earlier round that shows up again was re-noted while still alive —
    /// the "recurrent live root" share the selective-rooting lever targets.
    seen_roots: std::collections::HashSet<u32>,
    reroot_roots: u64,
    /// Object roots that came out of their classify round ALIVE (not white).
    live_roots: u64,
    t0: Option<std::time::Instant>,
    /// Per-round log file (`PHPR_GC_COLLECT_LOG`, absolute path, append).
    collect_log: Option<std::fs::File>,
}

thread_local! {
    static CENSUS: RefCell<Option<Box<GcCensus>>> = const { RefCell::new(None) };
}

pub fn arm() {
    if std::env::var_os("PHPR_GC_CENSUS").is_none() {
        return;
    }
    CENSUS.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            let mut census = Box::new(GcCensus::default());
            census.t0 = Some(std::time::Instant::now());
            if let Ok(path) = std::env::var("PHPR_GC_COLLECT_LOG") {
                if path.starts_with('/') {
                    census.collect_log = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&path)
                        .ok();
                }
            }
            *c = Some(census);
        }
    });
}

fn bump(f: impl FnOnce(&mut GcCensus)) {
    CENSUS.with(|c| {
        if let Some(census) = c.borrow_mut().as_deref_mut() {
            f(census);
        }
    });
}

pub fn note() { bump(|c| c.notes += 1); }
pub fn note_inserted() { bump(|c| c.notes_inserted += 1); }
pub fn sweep(main: bool) {
    bump(|c| if main { c.sweeps_main += 1 } else { c.sweeps_light += 1 });
}
pub fn sweep_band_skipped() { bump(|c| c.sweeps_band_skipped += 1); }
pub fn cand_freed() { bump(|c| c.cand_freed += 1); }
pub fn cand_demoted() { bump(|c| c.cand_demoted += 1); }
pub fn collect(roots: usize, freed: usize) {
    bump(|c| {
        c.collects += 1;
        c.collect_roots += roots as u64;
        c.collect_freed += freed as u64;
    });
}
pub fn threshold(t: usize) { bump(|c| c.threshold_last = t as u64); }
/// Host `gc_collect_cycles()` about to run: tag the next `collect_begin`.
pub fn mark_explicit() { bump(|c| c.explicit_pending = true); }
/// A `collect_cycles` call passed the re-entrancy latch.
pub fn collect_begin() {
    bump(|c| {
        c.collect_calls += 1;
        c.current_call_explicit = c.explicit_pending;
        if c.explicit_pending {
            c.explicit_calls += 1;
            c.explicit_pending = false;
        }
        c.round_in_call = 0;
    });
}
/// One classify round: obj roots (by id, for recurrence), container roots,
/// whites found, threshold in force, classify wall-time for THIS round.
/// One classify round. `white_objs` (sorted, from `GcWhites`) splits the
/// object roots into dead vs LIVE — the live share is what the
/// selective-rooting lever would avoid re-walking. Recurrence bookkeeping:
/// ids are recycled after death, so freed roots leave `seen_roots` — a
/// `reroot` is an id last seen ALIVE at a previous classify (proxy; an id
/// that died by refcount in between and got recycled still counts).
pub fn collect_round(
    obj_roots: &[u32],
    ctr_roots: usize,
    whites: usize,
    white_objs: &[u32],
    threshold: usize,
    classify_ns: u64,
) {
    bump(|c| {
        c.round_in_call += 1;
        let mut reroot = 0usize;
        let mut live = 0usize;
        for &id in obj_roots {
            if !c.seen_roots.insert(id) {
                reroot += 1;
            }
            if white_objs.binary_search(&id).is_err() {
                live += 1;
            }
        }
        for id in white_objs {
            c.seen_roots.remove(id);
        }
        c.reroot_roots += reroot as u64;
        c.live_roots += live as u64;
        if let Some(f) = c.collect_log.as_mut() {
            use std::io::Write as _;
            let t_ms = c.t0.map(|t| t.elapsed().as_millis()).unwrap_or(0);
            let _ = writeln!(
                f,
                "t={} call={} round={} src={} roots={} ctr={} live={} reroot={} whites={} thr={} classify_ms={}",
                t_ms,
                c.collect_calls,
                c.round_in_call,
                if c.current_call_explicit { "x" } else { "a" },
                obj_roots.len() + ctr_roots,
                ctr_roots,
                live,
                reroot,
                whites,
                threshold,
                classify_ns / 1_000_000,
            );
        }
    });
}
pub fn destructor() { bump(|c| c.destructors += 1); }
pub fn classify_ns(ns: u64) { bump(|c| c.classify_ns += ns); }
pub fn reflect_hit() { bump(|c| c.reflect_hits += 1); }
pub fn reflect_miss() { bump(|c| c.reflect_misses += 1); }
pub fn reflect_evict() { bump(|c| c.reflect_evictions += 1); }

/// Dump and clear at end of run (called next to `census_dump`).
pub fn dump() {
    let Some(c) = CENSUS.with(|c| c.borrow_mut().take()) else { return };
    let report = format!(
        "== PHPR_GC_CENSUS ==\n\
         notes {} (inserted {})\n\
         sweeps main {} / light {} / band_skipped {}\n\
         candidates freed {} / demoted {}\n\
         collects {} (roots {} freed {}) threshold_last {}\n\
         collect calls {} (explicit {}) distinct_obj_roots {} reroot {} live_roots {}\n\
         destructors {}\n\
         classify_ms {}\n\
         reflect cache hits {} / misses {} / evictions {}\n",
        c.notes, c.notes_inserted, c.sweeps_main, c.sweeps_light,
        c.sweeps_band_skipped,
        c.cand_freed, c.cand_demoted,
        c.collects, c.collect_roots, c.collect_freed, c.threshold_last,
        c.collect_calls, c.explicit_calls, c.seen_roots.len(), c.reroot_roots,
        c.live_roots,
        c.destructors,
        c.classify_ns / 1_000_000,
        c.reflect_hits, c.reflect_misses, c.reflect_evictions,
    );
    match std::env::var("PHPR_GC_CENSUS") {
        Ok(path) if path.starts_with('/') => {
            use std::io::Write as _;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                let _ = f.write_all(report.as_bytes());
            }
        }
        _ => eprint!("{report}"),
    }
}
