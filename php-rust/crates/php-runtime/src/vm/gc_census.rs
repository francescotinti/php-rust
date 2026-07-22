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
            *c = Some(Box::new(GcCensus::default()));
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
pub fn destructor() { bump(|c| c.destructors += 1); }

/// Dump and clear at end of run (called next to `census_dump`).
pub fn dump() {
    let Some(c) = CENSUS.with(|c| c.borrow_mut().take()) else { return };
    let report = format!(
        "== PHPR_GC_CENSUS ==\n\
         notes {} (inserted {})\n\
         sweeps main {} / light {}\n\
         candidates freed {} / demoted {}\n\
         collects {} (roots {} freed {}) threshold_last {}\n\
         destructors {}\n",
        c.notes, c.notes_inserted, c.sweeps_main, c.sweeps_light,
        c.cand_freed, c.cand_demoted,
        c.collects, c.collect_roots, c.collect_freed, c.threshold_last,
        c.destructors,
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
