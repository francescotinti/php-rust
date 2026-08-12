//! S-133 sonda ctor — SOLO CONTEGGI (criterio s133-criterio-ctor.md p.1).
//! Build emendata via patch `count-probes-ctor.patch` + `#[path]`; nessuna
//! strumentazione nei sorgenti del pin. Niente timer: i conteggi sono
//! deterministici e insensibili al rumore.
//!
//! Segmenti: 0 = resolve dentro `magic_applies` (sito A) · 1 = resolve nel
//! fallback di `prop_set_entry` (sito B) · 2 = TUTTE le `resolve_prop_access`
//! (entry della fn) · 3 = entrate in `prop_set_entry` (slow-path o no).
//! Dump atexit su `PHPR_CTOR_PROBES`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

pub const NSEG: usize = 4;
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
    let Ok(path) = std::env::var("PHPR_CTOR_PROBES") else { return };
    let Ok(mut f) = std::fs::File::create(path) else { return };
    let _ = write!(f, "ctorprobes pid={}", std::process::id());
    for i in 0..NSEG {
        let _ = write!(f, " c{i}={}", CNT[i].load(Relaxed));
    }
    let _ = writeln!(f);
}
