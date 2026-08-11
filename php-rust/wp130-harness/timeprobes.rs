//! S-130 sonda E1a — resolve_prop_access nel cammino `Op::FieldAssign`.
//! SOLO build emendata (patch `time-probes-e1a.patch`, target separato): questo
//! file vive in wp130-harness/ ed entra nel crate via `#[path]` dalla patch —
//! nessuna strumentazione nei sorgenti del pin. Modulo identico a
//! wp129-harness/timeprobes.rs salvo questa intestazione (mappa segmenti).
//!
//! Segmenti S-130: 0=TOT arm FieldAssign · 1=E1a resolve_prop_access (tutte le
//! chiamate del processo; conteggi a verbale) · 4=E field_set · 5=E2 walk
//! interno (prop_step) · 7=CAL coppia di letture back-to-back · 2,3,6 inutilizzati.
//! Contatore `cntvct_el0` (mrs senza isb — dichiarato nel criterio);
//! dump atexit su `PHPR_TIME_PROBES` (ticks + cntfrq per la conversione).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

pub const NSEG: usize = 8;
static SUM: [AtomicU64; NSEG] = [const { AtomicU64::new(0) }; NSEG];
static CNT: [AtomicU64; NSEG] = [const { AtomicU64::new(0) }; NSEG];
static REG: AtomicBool = AtomicBool::new(false);

unsafe extern "C" {
    fn atexit(f: extern "C" fn()) -> i32;
}

#[inline(always)]
pub fn now() -> u64 {
    let t: u64;
    unsafe { core::arch::asm!("mrs {t}, cntvct_el0", t = out(reg) t, options(nomem, nostack)) };
    t
}

#[inline(always)]
pub fn note(seg: usize, t0: u64) {
    SUM[seg].fetch_add(now().wrapping_sub(t0), Relaxed);
    CNT[seg].fetch_add(1, Relaxed);
    if !REG.swap(true, Relaxed) {
        unsafe {
            atexit(dump);
        }
    }
}

extern "C" fn dump() {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_TIME_PROBES") else { return };
    let Ok(mut f) = std::fs::File::create(path) else { return };
    let freq: u64;
    unsafe { core::arch::asm!("mrs {f}, cntfrq_el0", f = out(reg) freq, options(nomem, nostack)) };
    let _ = write!(f, "tprobes pid={} freq={freq}", std::process::id());
    for i in 0..NSEG {
        let _ = write!(f, " s{i}={} c{i}={}", SUM[i].load(Relaxed), CNT[i].load(Relaxed));
    }
    let _ = writeln!(f);
}
