//! S-138 sonda attribuzione RMW (criterio s138-criterio-sonda-rmw.md) —
//! build emendata nel WORKTREE al sorgente della LEVA, MAI nel pin. COPIA
//! DICHIARATA di wp137-harness/s137tp.rs (tecnica arm-only VALIDATA in
//! s138-sonda-v2: inerzia 0,000, inline mantenuto) + UNA aggiunta: il
//! kill-switch `full_path()` (`PHPR_TP_FULL=1`) che forza il MISS del fast
//! path RMW per misurare l'arm PIENO dallo STESSO binario (contrasto
//! omogeneo per costruzione). UN segmento per run (`PHPR_TP=<seg>`).

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};
use std::sync::OnceLock;
use std::time::Instant;

static ACC_NS: AtomicU64 = AtomicU64::new(0);
static SPANS: AtomicU64 = AtomicU64::new(0);
static REG: AtomicBool = AtomicBool::new(false);

fn active() -> usize {
    static A: OnceLock<usize> = OnceLock::new();
    *A.get_or_init(|| {
        std::env::var("PHPR_TP").ok().and_then(|v| v.parse().ok()).unwrap_or(usize::MAX)
    })
}

/// Kill-switch della sonda: con `PHPR_TP_FULL=1` il fast path RMW fa MISS
/// immediato e l'arm misurato è il cammino PIENO del candidato (che include
/// prelude_skip+fill, DICHIARATI nel criterio come ε non prezzato).
pub fn full_path() -> bool {
    static F: OnceLock<bool> = OnceLock::new();
    *F.get_or_init(|| std::env::var("PHPR_TP_FULL").is_ok_and(|v| v == "1"))
}

fn epoch() -> &'static Instant {
    static E: OnceLock<Instant> = OnceLock::new();
    E.get_or_init(Instant::now)
}

fn register() {
    if !REG.swap(true, Relaxed) {
        unsafe {
            atexit(dump);
        }
    }
}

/// Inizio span del segmento `seg`: ns dall'epoca se attivo, altrimenti 0.
#[inline(always)]
pub fn t0(seg: usize) -> u64 {
    if seg != active() {
        return 0;
    }
    epoch().elapsed().as_nanos() as u64
}

/// Fine span: accumula solo se attivo.
#[inline(always)]
pub fn add(seg: usize, t0: u64) {
    if seg != active() {
        return;
    }
    let now = epoch().elapsed().as_nanos() as u64;
    ACC_NS.fetch_add(now.saturating_sub(t0), Relaxed);
    SPANS.fetch_add(1, Relaxed);
    register();
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
        "s138tp seg={} ns={} spans={} full={}",
        active(),
        ACC_NS.load(Relaxed),
        SPANS.load(Relaxed),
        full_path() as u8
    );
}
