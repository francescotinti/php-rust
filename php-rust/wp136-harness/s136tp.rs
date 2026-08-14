//! S-136 modello del tempo FieldAssign (criterio s136-criterio-tempo.md) —
//! build emendata nel WORKTREE s135, MAI nel pin. UN segmento attivo per run
//! (`PHPR_TP=<seg>`); inattivo = un branch + load. Dump atexit su
//! `PHPR_TP_OUT` (ns accumulati + conteggio span).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::OnceLock;
use std::time::Instant;

pub const NSEG: usize = 10;
static ACC_NS: AtomicU64 = AtomicU64::new(0);
static SPANS: AtomicU64 = AtomicU64::new(0);
static REG: AtomicBool = AtomicBool::new(false);

fn active() -> usize {
    static A: OnceLock<usize> = OnceLock::new();
    *A.get_or_init(|| {
        std::env::var("PHPR_TP").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX)
    })
}

fn epoch() -> &'static Instant {
    static E: OnceLock<Instant> = OnceLock::new();
    E.get_or_init(Instant::now)
}

/// Inizio span del segmento `seg`: ns dall'epoca se attivo, altrimenti 0.
#[inline(always)]
pub fn t0(seg: usize) -> u64 {
    if seg != active() {
        return 0;
    }
    epoch().elapsed().as_nanos() as u64
}

/// Fine span: accumula solo se attivo (t0 del segmento attivo in mano).
#[inline(always)]
pub fn add(seg: usize, t0: u64) {
    if seg != active() {
        return;
    }
    let now = epoch().elapsed().as_nanos() as u64;
    ACC_NS.fetch_add(now.saturating_sub(t0), Relaxed);
    SPANS.fetch_add(1, Relaxed);
    if !REG.swap(true, Relaxed) {
        unsafe {
            atexit(dump);
        }
    }
}

unsafe extern "C" {
    fn atexit(f: extern "C" fn()) -> i32;
}

extern "C" fn dump() {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_TP_OUT") else { return };
    let Ok(mut f) = std::fs::File::create(path) else { return };
    let _ = writeln!(
        f,
        "s136tp seg={} ns={} spans={}",
        active(),
        ACC_NS.load(Relaxed),
        SPANS.load(Relaxed)
    );
}
