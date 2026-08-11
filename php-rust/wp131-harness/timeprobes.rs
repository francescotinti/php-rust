//! S-131 sonda prop_step — partizione di E−E2 in `field_write_prop_step`.
//! SOLO build emendata (patch `time-probes-propstep.patch`, target separato):
//! questo file vive in wp131-harness/ ed entra nel crate via `#[path]` dalla
//! patch — nessuna strumentazione nei sorgenti del pin. Derivato da
//! wp130-harness/timeprobes.rs (NSEG 8→12, mappa segmenti nuova).
//!
//! Segmenti S-131 (criterio s131-criterio-propstep.md): 0=TOT arm FieldAssign ·
//! 4=E field_set · 5=E2 walk interno · 7=CAL coppia di letture back-to-back
//! (numerazione S-130 per confronto) · 1=prop_step INTERO (call-site driver) ·
//! 2=borrow (Rc::clone+try_borrow_mut+header) · 3=guardie (container-guard su
//! intermedio) · 6=defer-check intermedio · 8=key+container-op · 9=resolve
//! DENTRO FieldScope::prop_key · 10=resolve DENTRO prop_key_read · 11=resolve
//! TUTTE (= E1a S-130). Contatore `cntvct_el0` (mrs senza isb — dichiarato);
//! dump atexit su `PHPR_TIME_PROBES` (ticks + cntfrq per la conversione).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

pub const NSEG: usize = 12;
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
