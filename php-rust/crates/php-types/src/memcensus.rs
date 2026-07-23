//! Byte-attribution census (Fase 0, FOOTPRINT_CPU_ROADMAP.md — WP-45).
//!
//! Feature `mem-census`, off by default (hooks compile out). Per CHANNEL it
//! tracks `live_bytes` (alloc/free/adjust), `peak_bytes` (watermark of live),
//! `cumulative_bytes` and counts; the OBJECT channel is death-accounted
//! (exact bytes at drop + a live COUNT via the id choke) because objects are
//! cloned/built at too many sites to funnel. A proxy total (Σ live) drives
//! watermark snapshots so the dump captures composition AT the peak, not at
//! exit. Output: appended to `$PHPR_MEM_CENSUS` (atexit, per-pid lines —
//! same plumbing as the str/op/gc censuses).
//!
//! Measurement-only: never enabled in parity or A/B builds. Counters are
//! atomics (phpt-runner threads); the VM itself is single-threaded.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering::Relaxed};

pub const CH_STR: usize = 0;
pub const CH_ARR: usize = 1;
pub const CH_OBJ: usize = 2;
pub const CH_UNIT: usize = 3;
pub const N_CH: usize = 4;
pub const CHANNEL_NAMES: [&str; N_CH] = ["str", "arr", "obj", "unit"];

/// Fixed per-value overhead added on top of payload bytes, per channel:
/// Rc header (strong+weak = 16) + struct size. Documented in the dump.
pub const STR_OVERHEAD: usize = 16 + 24; // Rc hdr + PhpStr{hash,Box<[u8]>}
pub const ARR_OVERHEAD: usize = 16 + 40; // Rc hdr + PhpArray header
pub const OBJ_OVERHEAD: usize = 16 + 8; // Rc hdr + RefCell borrow flag

static LIVE: [AtomicI64; N_CH] = [const { AtomicI64::new(0) }; N_CH];
static PEAK: [AtomicI64; N_CH] = [const { AtomicI64::new(0) }; N_CH];
static CUM: [AtomicU64; N_CH] = [const { AtomicU64::new(0) }; N_CH];
static CUM_N: [AtomicU64; N_CH] = [const { AtomicU64::new(0) }; N_CH];
static LIVE_N: [AtomicI64; N_CH] = [const { AtomicI64::new(0) }; N_CH];

/// Extra gauges sampled by the runtime (not byte channels).
pub const G_CREATED: usize = 0; // Vm::created registry len
pub const G_UNITS: usize = 1; // leaked modules count
pub const N_GAUGES: usize = 2;
static GAUGES: [AtomicI64; N_GAUGES] = [const { AtomicI64::new(0) }; N_GAUGES];

static PROXY_PEAK: AtomicI64 = AtomicI64::new(0);
static REGISTERED: AtomicBool = AtomicBool::new(false);
/// Next proxy-total watermark (bytes) that triggers a snapshot line.
static NEXT_MARK: AtomicI64 = AtomicI64::new(256 << 20);

fn ensure_registered() {
    if !REGISTERED.swap(true, Relaxed) {
        unsafe { libc::atexit(dump_exit) };
    }
}

#[inline]
pub fn alloc(ch: usize, bytes: usize) {
    ensure_registered();
    let b = bytes as i64;
    let live = LIVE[ch].fetch_add(b, Relaxed) + b;
    PEAK[ch].fetch_max(live, Relaxed);
    CUM[ch].fetch_add(bytes as u64, Relaxed);
    CUM_N[ch].fetch_add(1, Relaxed);
    LIVE_N[ch].fetch_add(1, Relaxed);
    watermark();
}

#[inline]
pub fn free(ch: usize, bytes: usize) {
    LIVE[ch].fetch_sub(bytes as i64, Relaxed);
    LIVE_N[ch].fetch_sub(1, Relaxed);
}

/// Capacity delta from a mutating operation (may be negative).
#[inline]
pub fn adjust(ch: usize, delta: i64) {
    if delta == 0 {
        return;
    }
    let live = LIVE[ch].fetch_add(delta, Relaxed) + delta;
    if delta > 0 {
        PEAK[ch].fetch_max(live, Relaxed);
        CUM[ch].fetch_add(delta as u64, Relaxed);
        watermark();
    }
}

/// Death accounting (OBJ channel): exact bytes measured at drop; the live
/// byte figure for this channel is estimated at dump time as
/// `live_count × (cum/cum_n)`.
#[inline]
pub fn death(ch: usize, bytes: usize) {
    ensure_registered();
    CUM[ch].fetch_add(bytes as u64, Relaxed);
    CUM_N[ch].fetch_add(1, Relaxed);
}

#[inline]
pub fn count_alloc(ch: usize) {
    LIVE_N[ch].fetch_add(1, Relaxed);
}

#[inline]
pub fn count_free(ch: usize) {
    LIVE_N[ch].fetch_sub(1, Relaxed);
}

#[inline]
pub fn gauge(g: usize, v: i64) {
    GAUGES[g].store(v, Relaxed);
}

/// Proxy total = Σ live byte channels + OBJ estimate; snapshot on each
/// +128MB watermark so the dump shows composition at (near) the peak.
fn watermark() {
    let total = proxy_total();
    PROXY_PEAK.fetch_max(total, Relaxed);
    let mark = NEXT_MARK.load(Relaxed);
    if total >= mark
        && NEXT_MARK
            .compare_exchange(mark, total + (128 << 20), Relaxed, Relaxed)
            .is_ok()
    {
        dump_line("mark");
    }
}

fn proxy_total() -> i64 {
    let mut t = 0i64;
    for ch in 0..N_CH {
        t += live_estimate(ch);
    }
    t
}

fn live_estimate(ch: usize) -> i64 {
    if ch == CH_OBJ || ch == CH_ARR {
        // death-accounted channels: live bytes ≈ live count × average death
        // size (exact live requires mutator hooks — Fase 0 accepts the bias
        // and cross-checks against the external peak residual).
        let n = CUM_N[ch].load(Relaxed);
        if n == 0 {
            return 0;
        }
        let avg = CUM[ch].load(Relaxed) / n;
        LIVE_N[ch].load(Relaxed).max(0) * avg as i64
    } else {
        LIVE[ch].load(Relaxed)
    }
}

fn dump_line(tag: &str) {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_MEM_CENSUS") else { return };
    let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let mut line = format!("pid={} tag={}", std::process::id(), tag);
    for ch in 0..N_CH {
        line.push_str(&format!(
            " {}.live={} {}.peak={} {}.cum={} {}.cum_n={} {}.live_n={}",
            CHANNEL_NAMES[ch],
            live_estimate(ch),
            CHANNEL_NAMES[ch],
            PEAK[ch].load(Relaxed),
            CHANNEL_NAMES[ch],
            CUM[ch].load(Relaxed),
            CHANNEL_NAMES[ch],
            CUM_N[ch].load(Relaxed),
            CHANNEL_NAMES[ch],
            LIVE_N[ch].load(Relaxed),
        ));
    }
    line.push_str(&format!(
        " created={} units={} proxy_peak={}\n",
        GAUGES[G_CREATED].load(Relaxed),
        GAUGES[G_UNITS].load(Relaxed),
        PROXY_PEAK.load(Relaxed),
    ));
    let _ = f.write_all(line.as_bytes());
}

extern "C" fn dump_exit() {
    dump_line("exit");
}

/// Deep retained-size walk from a root value (Fase 0 root attribution):
/// counts each shared allocation ONCE (dedup by Rc pointer), so walking the
/// roots in priority order attributes shared structure to the FIRST root
/// that reaches it. Read-only borrows; recursion capped by `depth`
/// (a hit on the cap under-counts and bumps [`truncated`]).
pub fn deep_size(
    v: &crate::Zval,
    seen: &mut rustc_hash::FxHashSet<usize>,
    depth: u32,
) -> u64 {
    use crate::Zval;
    if depth > 2000 {
        TRUNCATED.fetch_add(1, Relaxed);
        return 0;
    }
    match v {
        Zval::Str(s) => {
            if seen.insert(std::rc::Rc::as_ptr(s) as usize) {
                (s.as_bytes().len() + STR_OVERHEAD) as u64
            } else {
                0
            }
        }
        Zval::Array(a) => {
            if !seen.insert(std::rc::Rc::as_ptr(a) as usize) {
                return 0;
            }
            let mut b = a.census_bytes() as u64;
            for (k, ev) in a.iter() {
                if let crate::Key::Str(ks) = &k {
                    if seen.insert(std::rc::Rc::as_ptr(ks) as usize) {
                        b += (ks.as_bytes().len() + STR_OVERHEAD) as u64;
                    }
                }
                b += deep_size(ev, seen, depth + 1);
            }
            b
        }
        Zval::Object(o) => {
            if !seen.insert(std::rc::Rc::as_ptr(o) as usize) {
                return 0;
            }
            let Ok(bo) = o.try_borrow() else { return 0 };
            let mut b = bo.census_bytes() as u64;
            let kids = bo.census_children_cloned();
            drop(bo);
            for k in &kids {
                b += deep_size(k, seen, depth + 1);
            }
            b
        }
        Zval::Ref(r) => {
            if !seen.insert(std::rc::Rc::as_ptr(r) as usize) {
                return 0;
            }
            let Ok(inner) = r.try_borrow() else { return 0 };
            32 + deep_size(&inner, seen, depth + 1)
        }
        Zval::Closure(c) => {
            if !seen.insert(std::rc::Rc::as_ptr(c) as usize) {
                return 0;
            }
            let mut b = 128u64;
            for (_, cv) in &c.captures {
                b += deep_size(cv, seen, depth + 1);
            }
            if let Some(bt) = &c.bound_this {
                b += deep_size(bt, seen, depth + 1);
            }
            b
        }
        Zval::Generator(g) => {
            if !seen.insert(std::rc::Rc::as_ptr(g) as usize) {
                return 0;
            }
            let Ok(gs) = g.try_borrow() else { return 0 };
            512 + deep_size(&gs.cur_key, seen, depth + 1)
                + deep_size(&gs.cur_val, seen, depth + 1)
        }
        Zval::Resource(_) | Zval::ArgPlace(_) => 256,
        _ => 0,
    }
}

static TRUNCATED: AtomicU64 = AtomicU64::new(0);

/// Append the per-root attribution lines (tag=root) plus a closing summary.
pub fn report_roots(entries: &[(String, u64)]) {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_MEM_CENSUS") else { return };
    let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    let pid = std::process::id();
    let total: u64 = entries.iter().map(|(_, b)| b).sum();
    for (name, b) in entries {
        if *b >= 1 << 20 {
            let _ = writeln!(f, "pid={pid} tag=root name={name} bytes={b}");
        }
    }
    let _ = writeln!(
        f,
        "pid={pid} tag=roots_total bytes={total} truncated={}",
        TRUNCATED.load(Relaxed)
    );
}
