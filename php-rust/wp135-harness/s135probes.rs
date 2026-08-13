//! S-135 sonda conteggi (criterio s135-criterio-sonda.md p.1) — SOLO CONTEGGI.
//! Build emendata via edit nei WORKTREE (gambe s133/s134) + `#[path]`; nessuna
//! strumentazione nei sorgenti del pin. Niente timer: conteggi deterministici.
//!
//! Segmenti: 0 entrate prop_set_entry · 1 entrate resolve_prop_access ·
//! 2 IC-hit NP · 3 IC-fill NP · 4 magic-probe · 5 hook-lookup eseguito ·
//! 6 enum-check borrow · 7 asym check · 8 readonly decl-lookup ·
//! 9 deprecation props.contains · 10 coerce_typed_prop_write · 11 IC-hit
//! plain. Dump atexit su `PHPR_S135_PROBES`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

pub const NSEG: usize = 12;
static CNT: [AtomicU64; NSEG] = [const { AtomicU64::new(0) }; NSEG];
static REG: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    fn atexit(f: extern "C" fn()) -> i32;
}

#[inline(always)]
pub fn count(seg: usize) {
    CNT[seg].fetch_add(1, Relaxed);
    if !REG.swap(true, Relaxed) {
        unsafe {
            atexit(dump);
        }
    }
}

extern "C" fn dump() {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_S135_PROBES") else { return };
    let Ok(mut f) = std::fs::File::create(path) else { return };
    let _ = write!(f, "s135probes pid={}", std::process::id());
    for i in 0..NSEG {
        let _ = write!(f, " c{i}={}", CNT[i].load(Relaxed));
    }
    let _ = writeln!(f);
}
