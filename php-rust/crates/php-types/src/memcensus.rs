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
pub const STR_OVERHEAD: usize = 16 + 32; // Rc hdr + PhpStr{hash,Vec<u8>} (+8B growable, WP-55)
pub const ARR_OVERHEAD: usize = 16 + 48; // Rc hdr + PhpArray header (+8B WalkMark, WP-52)
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

/// Fase 1.1 (WP-48): cumulative Vec capacity SLACK (capacity − len, in the
/// same terms [`CH_UNIT`] counts) of every leaked module, measured at the
/// leak site. On a pre-shrink binary this IS the predicted saving of the
/// shrink lever; on a post-shrink binary it must read ≈ 0 (mechanism-check).
static UNIT_SLACK: AtomicU64 = AtomicU64::new(0);

pub fn unit_slack(bytes: u64) {
    UNIT_SLACK.fetch_add(bytes, Relaxed);
}

static PROXY_PEAK: AtomicI64 = AtomicI64::new(0);
static REGISTERED: AtomicBool = AtomicBool::new(false);
/// Next proxy-total watermark (bytes) that triggers a snapshot line.
static NEXT_MARK: AtomicI64 = AtomicI64::new(256 << 20);

fn ensure_registered() {
    if !REGISTERED.swap(true, Relaxed) {
        mono_ms(); // anchor the monotonic clock at the first census event
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

/// WP-58 Ob.2: fixed live-bytes of one REAL object (the `Rc<RefCell<..>>`
/// header block), allocated at the id choke (`Vm::next_id`) and freed at
/// `Object::drop` (`id != 0`); the props part is live-accounted on `Props`
/// itself. `ObjRare` + `proxy_instance` are WALK-only (no mutation funnel:
/// pub-field writes) — a small documented positive residual of
/// `reached_b − live_b` on objects that carry them.
pub fn obj_fixed() -> usize {
    std::mem::size_of::<crate::Object>() + OBJ_OVERHEAD
}

/// Death accounting (formerly the OBJ channel's feed; every value channel
/// is live-accounted since WP-58 — kept for potential one-off censuses).
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
    // WP-59 Ob.1a: windows re-keyed on the TRUE physical footprint —
    // throttled (task_info ~1µs) to one check per 16384 census events.
    if PHYS_TICK.fetch_add(1, Relaxed) & 0x3FFF == 0 {
        phys_check();
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
    // Every channel is live-accounted since WP-58 (obj: fixed part at the
    // id choke + Props accounted/sync/Drop; the death-avg estimator died
    // with WP-56's 5,7× over-count verdict on arr).
    LIVE[ch].load(Relaxed)
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
        " created={} units={} unit_slack={} proxy_peak={}",
        GAUGES[G_CREATED].load(Relaxed),
        GAUGES[G_UNITS].load(Relaxed),
        UNIT_SLACK.load(Relaxed),
        PROXY_PEAK.load(Relaxed),
    ));
    // S-143 istruttoria: numeratori-evento e denominatore raw NELLA STESSA
    // run (mai denominatori a memoria); zval_size dichiarato (Hejlsberg R3).
    #[cfg(feature = "mem-census")]
    {
        let (arrbuf, propsbuf) = s143_counters();
        let (gan, gfn) = alloc_event_counters();
        line.push_str(&format!(
            " s143.arrbuf_n={} s143.propsbuf_n={} s143.galloc_n={} s143.gfree_n={} s143.zval_size={}",
            arrbuf,
            propsbuf,
            gan,
            gfn,
            std::mem::size_of::<crate::Zval>(),
        ));
        // S-144 tranche-2: rczval/vecargs + realloc-eventi DICHIARATI accanto
        // al denominatore (fuori da galloc_n per costruzione, A-LE-104-1).
        let (rcz, rczp, vargs) = s144_counters();
        let (grn, _, _) = realloc_counters();
        line.push_str(&format!(
            " s144.rczval_n={} s144.rczval_prop_n={} s144.vecargs_n={} s144.grealloc_n={} s144.objsynth_n={}",
            rcz, rczp, vargs, grn,
            s144_objsynth(),
        ));
    }
    line.push('\n');
    let _ = f.write_all(line.as_bytes());
}

extern "C" fn dump_exit() {
    dump_line("exit");
    // WP-59 Ob.1: exit snapshot (win=0) — the cleanest fragmentation read:
    // channel live is exact here (recon reached==live), so per-bin
    // committed−used at this instant IS retention + non-censused standing.
    phys_window_dump(0, phys_footprint(), "exit_mi");
    // WP-65 L-65.2 (L-64b made exigible): the STANDING checkpoint —
    // mi_collect(true) drains retention, then a second per-bin/phys read
    // says what actually stands. The PRIMA/DOPO pair for the lever is this
    // window across the two census binaries (KL-65.1/KL-65.3); the delta
    // vs `exit_mi` above is retention, free. Opt-in per run.
    if std::env::var_os("PHPR_MI_COLLECT_EXIT").is_some_and(|v| v == "1") {
        collect_mi_standing();
    }
}

/// The standing checkpoint body shared by the atexit path above and the
/// per-request opt-in below — the ONLY `mi_collect` call site (no new
/// unsafe was added for WP-67, KH67-1).
fn collect_mi_standing() {
    unsafe { mi_collect(true) };
    phys_window_dump(0, phys_footprint(), "exit_collect_mi");
}

/// S-93.0 B3 (LEVER-2, arm A/B del probe): force-collect ON-THREAD,
/// pubblico per il worker del server. I sei blocchi huge del preludio
/// (wp92-harness/huge-worker.out) sono LIBERATI dal drop dell'arena
/// bumpalo alla prima richiesta (provato col trace S-93.0), ma il theap
/// li tiene committed — questo collect li decommitta sul thread che li
/// possiede. Census builds only (il modulo è feature-gated), mai parity.
pub fn mi_collect_on_thread() {
    unsafe { mi_collect(true) }
}

/// A-DL49 (Council WP-91, Leijen): the IN-REQUEST census at the PEAK —
/// workers ALIVE, RetainSet retained. On macOS commit==peak_commit
/// (A-DL48): any post-workload pre-shutdown snapshot reads the peak
/// commit; the per-heap/per-bin distribution it captures is the LIVE one
/// the attribution needs (the win=0 exit census only ever saw the
/// post-teardown corpse). win=9 is RESERVED for this checkpoint
/// (ckpt=peak_inreq); the judge pins commit>=0.9*peak_commit (KL-91-3)
/// and heaps_total against W+1. QUIESCENZA (KS-MS-91-2) is the CALLER's
/// contract: the campaign asserts zero outstanding requests before
/// hitting the trigger — the serving thread allocates only outside the
/// visit (visitors are no-alloc, A-MS50).
pub fn peak_census_dump(phase: &str) {
    // `phase` names the campaign checkpoint by NAME (A-BG54/KG-91-2:
    // extractors select per ckpt, never positionally): ckpt=peak_inreq
    // for the peak proper, ckpt=peak_inreq_p<phase> for the Δcommitted
    // phase ladder (A-DL50 braccio 2 — the commit counter is monotone on
    // macOS, so per-phase deltas are exact, WP-88 lesson).
    if phase.is_empty() {
        phys_window_dump(9, phys_footprint(), "peak_inreq");
    } else {
        let tag = format!("peak_inreq{phase}");
        phys_window_dump(9, phys_footprint(), &tag);
    }
}

/// WP-67 (probe N={1,100,1000}, L-67.4): a killed `-S` server never reaches
/// atexit, so the per-request census dump invokes the SAME standing
/// checkpoint behind `PHPR_MI_COLLECT_REQ=1`. The metro reads the LAST
/// `exit_collect_mi` pair before the kill = the last completed request.
pub fn request_collect_mi() {
    if std::env::var_os("PHPR_MI_COLLECT_REQ").is_some_and(|v| v == "1") {
        collect_mi_standing();
        // KL71-2 (WP-71, escalation pre-registrata): block-CONTENT dump ai
        // checkpoint dichiarati — dopo il drain (standing only).
        blockdump_maybe();
    }
}

// ---------------------------------------------------------------------------
// KL71-2 (WP-71): ispezione CONTENUTO dei blocchi vivi nei bin della firma.
// Armata da PHPR_MI_BLOCKDUMP=<base> (+ PHPR_MI_BLOCKDUMP_AT="500,1000");
// al checkpoint per-richiesta n ∈ AT scrive <base>.r<n> con una riga per
// blocco vivo dei bin {32,64,96,112,128,160,192}: ptr, size e i primi 48
// byte (ascii-escaped). Il diff per puntatore+contenuto tra due checkpoint
// isola il set cresciuto-e-sopravvissuto; l'istogramma dei contenuti nomina
// il canale. Census-only, opt-in, mai su path di parità.
// ---------------------------------------------------------------------------
thread_local! {
    static BLOCKDUMP_REQ: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

// KL71-2 seconda passata: TUTTI i bin fino a 1024 — le foglie dei nodi
// della firma (stringhe condivise) vivono nei bin medi (256/512/640).
const BLOCKDUMP_MAX: usize = 1024;

unsafe extern "C" fn block_content_visitor(
    _heap: *const std::os::raw::c_void,
    area: *const MiHeapArea,
    block: *mut std::os::raw::c_void,
    block_size: usize,
    arg: *mut std::os::raw::c_void,
) -> bool {
    if block.is_null() {
        return true; // area entry (visit_blocks also reports areas)
    }
    if (*area).block_size > BLOCKDUMP_MAX {
        return true;
    }
    let f = &mut *(arg as *mut std::io::BufWriter<std::fs::File>);
    use std::io::Write;
    let take = block_size.min(48);
    let bytes = std::slice::from_raw_parts(block as *const u8, take);
    let mut esc = String::with_capacity(take * 2);
    for &b in bytes {
        if (0x20..0x7f).contains(&b) && b != b'\\' {
            esc.push(b as char);
        } else {
            esc.push_str(&format!("\\x{b:02x}"));
        }
    }
    let _ = writeln!(f, "ptr={:p} bin={} size={} data={}", block, (*area).block_size, block_size, esc);
    true
}

fn blockdump_maybe() {
    let Some(base) = std::env::var_os("PHPR_MI_BLOCKDUMP") else { return };
    let n = BLOCKDUMP_REQ.with(|c| {
        c.set(c.get() + 1);
        c.get()
    });
    let at = std::env::var("PHPR_MI_BLOCKDUMP_AT").unwrap_or_else(|_| "500,1000".into());
    if !at.split(',').any(|t| t.trim().parse::<u64>() == Ok(n)) {
        return;
    }
    let path = format!("{}.r{n}", base.to_string_lossy());
    let Ok(f) = std::fs::File::create(&path) else { return };
    let mut w = std::io::BufWriter::new(f);
    let ok = unsafe {
        mi_heap_visit_blocks(
            mi_heap_main(),
            true,
            block_content_visitor,
            &mut w as *mut std::io::BufWriter<std::fs::File> as *mut std::os::raw::c_void,
        )
    };
    use std::io::Write;
    let _ = writeln!(w, "blockdump req={n} complete={ok}");
    let _ = w.flush();
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
            if seen.insert(crate::ZStr::as_ptr(s) as usize) {
                let b = (s.as_bytes().len() + STR_OVERHEAD) as u64;
                reached(CH_STR, b);
                b
            } else {
                0
            }
        }
        Zval::Array(a) => {
            if !seen.insert(std::rc::Rc::as_ptr(a) as usize) {
                return 0;
            }
            let own = a.census_bytes() as u64;
            reached(CH_ARR, own);
            let (is_packed, n_live, cb) = a.census_shape();
            shape_note(is_packed, n_live, cb);
            let mut b = own;
            for (k, ev) in a.iter() {
                if let crate::Key::Str(ks) = &k {
                    if seen.insert(crate::ZStr::as_ptr(ks) as usize) {
                        let kb = (ks.as_bytes().len() + STR_OVERHEAD) as u64;
                        reached(CH_STR, kb);
                        b += kb;
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
            reached(CH_OBJ, bo.census_bytes() as u64);
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

/// Per-channel tally of what the ROOT WALK actually reached (WP-47): count
/// and bytes of each distinct allocation credited by [`deep_size`]. Reset
/// before a walk; the reconciliation lines in [`report_roots`] compare these
/// against the channel live counters — the residual IS the unattributed set.
static REACHED_N: [AtomicU64; N_CH] = [const { AtomicU64::new(0) }; N_CH];
static REACHED_B: [AtomicU64; N_CH] = [const { AtomicU64::new(0) }; N_CH];

/// WP-57: per-repr histogram of the REACHED arr population (EOR walk):
/// row 0 = packed, row 1 = hashed; bucket = log2(live element count)
/// (bucket 0 = empty array). `SHAPE_B` carries capacity bytes
/// (`census_bytes`), so Σ SHAPE_B ≈ reached_b of the arr channel.
pub const N_SHAPE_BUCKETS: usize = 24;
static SHAPE_N: [[AtomicU64; N_SHAPE_BUCKETS]; 2] =
    [const { [const { AtomicU64::new(0) }; N_SHAPE_BUCKETS] }; 2];
static SHAPE_B: [[AtomicU64; N_SHAPE_BUCKETS]; 2] =
    [const { [const { AtomicU64::new(0) }; N_SHAPE_BUCKETS] }; 2];

fn shape_note(is_packed: bool, count: u32, bytes: usize) {
    let row = if is_packed { 0 } else { 1 };
    let b = if count == 0 {
        0
    } else {
        (u32::BITS - count.leading_zeros()) as usize
    }
    .min(N_SHAPE_BUCKETS - 1);
    SHAPE_N[row][b].fetch_add(1, Relaxed);
    SHAPE_B[row][b].fetch_add(bytes as u64, Relaxed);
}

pub fn reached_reset() {
    for ch in 0..N_CH {
        REACHED_N[ch].store(0, Relaxed);
        REACHED_B[ch].store(0, Relaxed);
    }
    for row in 0..2 {
        for b in 0..N_SHAPE_BUCKETS {
            SHAPE_N[row][b].store(0, Relaxed);
            SHAPE_B[row][b].store(0, Relaxed);
        }
    }
}

#[inline]
fn reached(ch: usize, bytes: u64) {
    REACHED_N[ch].fetch_add(1, Relaxed);
    REACHED_B[ch].fetch_add(bytes, Relaxed);
}

/// Credit an allocation reached OUTSIDE [`deep_size`] (e.g. the unit const
/// pools, tallied by the caller with its own dedup against the same `seen`).
pub fn reached_note(ch: usize, bytes: u64) {
    reached(ch, bytes);
}

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
        // Bracketed names carry an entry count (n=...) whose value matters
        // even at 0 bytes (WP-49: created-dead-rc1 distinguishes "subset
        // empty" from "walk not run").
        if *b >= 1 << 20 || name.contains('[') {
            let _ = writeln!(f, "pid={pid} tag=root name={name} bytes={b}");
        }
    }
    let _ = writeln!(
        f,
        "pid={pid} tag=roots_total bytes={total} truncated={}",
        TRUNCATED.load(Relaxed)
    );
    // WP-47 reconciliation: walk-reached vs channel-live, per channel. The
    // residual (live − reached) is the set no Vm root reaches — the number
    // the second-generation attribution must drive to ~0 (±15%).
    for ch in [CH_STR, CH_ARR, CH_OBJ] {
        let _ = writeln!(
            f,
            "pid={pid} tag=walk_recon ch={} reached_n={} reached_b={} live_n={} live_b={}",
            CHANNEL_NAMES[ch],
            REACHED_N[ch].load(Relaxed),
            REACHED_B[ch].load(Relaxed),
            LIVE_N[ch].load(Relaxed),
            live_estimate(ch),
        );
    }
    // WP-57 per-repr histogram of the reached arr population (non-empty
    // buckets only): the input the Fase-3 tranche-2 ordering needs — where
    // the standing bytes live by shape and size.
    for (row, name) in [(0usize, "packed"), (1, "hashed")] {
        for b in 0..N_SHAPE_BUCKETS {
            let n = SHAPE_N[row][b].load(Relaxed);
            if n > 0 {
                let _ = writeln!(
                    f,
                    "pid={pid} tag=arr_shape repr={name} b={b} n={n} bytes={}",
                    SHAPE_B[row][b].load(Relaxed)
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// WP-59 Ob.1 — "Fase 0-bis" window instrumentation (census-only).
//
// The WP-45 watermark keys on the census PROXY total (peaks ~0,4GB) while the
// physical footprint peaks ~1,6GB: three quarters of the climb happened in
// windows the dump never delimited (Gregg R2). These hooks re-key snapshot
// windows on the TRUE physical footprint (`task_info` phys_footprint — the
// same ledger `footprint(1)`/vmmap report), and at each +128MB window emit:
//   tag=mi_proc   — mi_process_info (rss/commit, current+peak) + phys
//   tag=mi_bin    — per-size-class occupancy (reserved/committed/used) from
//                   mi_heap_visit_blocks(visit_blocks=false), main heap and
//                   abandoned pages separately (src=main|aband)
//   tag=mark_phys — the usual channel snapshot line
// plus the full mimalloc stats to $PHPR_MI_STATS (own file: multi-line), and
// a flag-file touch ($PHPR_PHYS_FLAG) for the external vmmap supervisor.
// Reconciliation identity (pre-registered, Leijen 3): Σ used_bins = census +
// non-censused live; Σ (committed − used) = fragmentation; + metadata +
// non-mimalloc regions ≈ physical footprint ±10-15%.
//
// Everything below is metrology: it runs only in `mem-census` builds and only
// every 16384 census events (task_info ~1µs) or at a window boundary.

/// Prefix of darwin's `task_vm_info` out to `phys_footprint` (rev1). The
/// kernel copies `min(requested, available)` words, so asking for exactly
/// this prefix is stable ABI on every macOS that has phys_footprint.
#[cfg(target_os = "macos")]
#[repr(C)]
struct TaskVmInfoPrefix {
    virtual_size: u64,
    region_count: i32,
    page_size: i32,
    resident_size: u64,
    resident_size_peak: u64,
    device: u64,
    device_peak: u64,
    internal: u64,
    internal_peak: u64,
    external: u64,
    external_peak: u64,
    reusable: u64,
    reusable_peak: u64,
    purgeable_volatile_pmap: u64,
    purgeable_volatile_resident: u64,
    purgeable_volatile_virtual: u64,
    compressed: u64,
    compressed_peak: u64,
    compressed_lifetime: u64,
    phys_footprint: u64,
}

#[cfg(target_os = "macos")]
const TASK_VM_INFO: u32 = 22;

#[cfg(target_os = "macos")]
extern "C" {
    static mach_task_self_: u32;
    fn task_info(task: u32, flavor: u32, info: *mut i32, count: *mut u32) -> i32;
}

/// The ledger the judges' metric reports (`/usr/bin/time -l` "peak memory
/// footprint" is the peak of this): internal + compressed, in bytes.
#[cfg(target_os = "macos")]
pub fn phys_footprint() -> u64 {
    let mut info = std::mem::MaybeUninit::<TaskVmInfoPrefix>::zeroed();
    let mut count = (std::mem::size_of::<TaskVmInfoPrefix>() / 4) as u32;
    let kr = unsafe {
        task_info(mach_task_self_, TASK_VM_INFO, info.as_mut_ptr() as *mut i32, &mut count)
    };
    if kr != 0 || (count as usize) * 4 < std::mem::size_of::<TaskVmInfoPrefix>() {
        return 0;
    }
    unsafe { info.assume_init() }.phys_footprint
}

/// Fuori macOS il ledger phys_footprint NON ESISTE (task_info è API Mach):
/// stub a 0 — lo stesso valore del ramo kr!=0, «nessuna lettura» — SOLO per
/// tenere compilabili le corsie mem-census sul runner Linux (S-139, fase 1
/// GH Actions). Da qui non esce mai una cifra: lo strumento è macOS-only.
#[cfg(not(target_os = "macos"))]
pub fn phys_footprint() -> u64 {
    0
}

// mimalloc v3 public API (statically linked via the `mimalloc` crate; no
// bindings in libmimalloc-sys for these, so they are declared here).
/// mi_subproc_id_t: `typedef struct { void* _mi_subproc_id; }` — passed
/// BY VALUE (A-DL46-census).
#[repr(C)]
struct MiSubprocId {
    _p: *mut std::os::raw::c_void,
}

#[repr(C)]
struct MiHeapArea {
    blocks: *mut std::os::raw::c_void,
    reserved: usize,
    committed: usize,
    used: usize, // number of ALLOCATED BLOCKS in the area
    block_size: usize,
    full_block_size: usize, // block size incl. padding/metadata
    reserved1: *mut std::os::raw::c_void,
}

type MiBlockVisitFun = unsafe extern "C" fn(
    heap: *const std::os::raw::c_void,
    area: *const MiHeapArea,
    block: *mut std::os::raw::c_void,
    block_size: usize,
    arg: *mut std::os::raw::c_void,
) -> bool;

extern "C" {
    fn mi_heap_main() -> *mut std::os::raw::c_void;
    // A-DL31 (Council WP-87): the heap OWNING a pointer — this tree's
    // mimalloc v3 does not export mi_heap_get_default (same pruning class
    // as the known mi_heap_set_default absence), so the calling thread's
    // default heap is recovered from a probe allocation it just made.
    fn mi_heap_of(p: *const std::os::raw::c_void) -> *mut std::os::raw::c_void;
    fn mi_heap_new() -> *mut std::os::raw::c_void;
    fn mi_heap_malloc_aligned(
        heap: *mut std::os::raw::c_void,
        size: usize,
        align: usize,
    ) -> *mut std::os::raw::c_void;
    fn mi_heap_zalloc_aligned(
        heap: *mut std::os::raw::c_void,
        size: usize,
        align: usize,
    ) -> *mut std::os::raw::c_void;
    fn mi_heap_realloc_aligned(
        heap: *mut std::os::raw::c_void,
        p: *mut std::os::raw::c_void,
        newsize: usize,
        align: usize,
    ) -> *mut std::os::raw::c_void;
    fn mi_heap_visit_blocks(
        heap: *mut std::os::raw::c_void,
        visit_blocks: bool,
        visitor: MiBlockVisitFun,
        arg: *mut std::os::raw::c_void,
    ) -> bool;
    fn mi_heap_visit_abandoned_blocks(
        heap: *mut std::os::raw::c_void,
        visit_blocks: bool,
        visitor: MiBlockVisitFun,
        arg: *mut std::os::raw::c_void,
    ) -> bool;
    fn mi_process_info(
        elapsed_msecs: *mut usize,
        user_msecs: *mut usize,
        system_msecs: *mut usize,
        current_rss: *mut usize,
        peak_rss: *mut usize,
        current_commit: *mut usize,
        peak_commit: *mut usize,
        page_faults: *mut usize,
    );
    fn mi_stats_print_out(
        out: Option<unsafe extern "C" fn(*const std::os::raw::c_char, *mut std::os::raw::c_void)>,
        arg: *mut std::os::raw::c_void,
    );
    /// WP-65 L-65.2: force-collect for the exit STANDING checkpoint —
    /// census builds only (the module is feature-gated), never parity.
    fn mi_collect(force: bool);
    // A-DL41 (Council WP-89, Leijen KL-89-1): environment-arm read-back —
    // an arm without an in-band `mi_option_get` echo is a MUTE arm and its
    // delta is VOID. Ordinals pinned against the VENDORED v3 header
    // (libmimalloc-sys 0.1.49 c_src/mimalloc/v3/include/mimalloc.h enum
    // mi_option_e): arena_eager_commit=4, purge_delay=15. The positive
    // control is in-band: campaigns run MIMALLOC_PURGE_DELAY=0, so
    // purge_delay MUST read back 0 (default 10) — a wrong ordinal or a
    // dead env path both surface as a wrong read-back, never silently.
    fn mi_option_get(option: i32) -> std::os::raw::c_long;
    // A-BB55≡A-DL42 (Council WP-89, Bak Q2 + Leijen Q2): the committed of
    // process at low W is a 64 KiB-granular STEP function — the arena/commit
    // term must be named in-band. mi_stats_get_json builds the full
    // mi_stats_t INSIDE mimalloc (no struct layout duplicated in Rust — a
    // hand-computed offset is a silent-lie channel) and writes into OUR
    // buffer; we then emit it through the atomic census channel. Per-arena
    // enumeration is not public in v3 (mi_arena_id_t is an opaque void*),
    // so the NAMED substitution is: process-level arena stats
    // (arena_count, committed, reserved, purged) + chunk_bins census —
    // declared in the campaign header. commit_calls BANNED (A-DL46).
    fn mi_stats_get_json(buf_size: usize, buf: *mut std::os::raw::c_char)
        -> *mut std::os::raw::c_char;
    // A-DL46-census (Council WP-90, Leijen — KL-90-4): per-theap page
    // census for the b-attribution. mi_subproc_visit_heaps walks every
    // live heap of the subprocess; combined with the area walk
    // (mi_heap_visit_blocks, visit_blocks=false: one callback per AREA,
    // block==NULL) it shows WHERE the committed lives — per heap, with
    // retained-free pages (used==0, still committed: the
    // page_full_retain candidates, ord 36) binned by block_size.
    fn mi_subproc_current() -> MiSubprocId;
    fn mi_subproc_visit_heaps(
        subproc: MiSubprocId,
        visitor: unsafe extern "C" fn(
            *mut std::os::raw::c_void,
            *mut std::os::raw::c_void,
        ) -> bool,
        arg: *mut std::os::raw::c_void,
    ) -> bool;
}

/// Per-size-class occupancy table filled by [`area_visitor`]. Fixed arrays:
/// the visitor MUST NOT allocate (it walks the very heap that would serve
/// the allocation). Sizes > 64KiB collapse into one "large" bucket — those
/// areas are 1 block each and carry no per-page fragmentation to speak of.
const N_BINS: usize = 192;
const LARGE_BUCKET: usize = 1 << 20;

struct BinTab {
    size: [usize; N_BINS],
    reserved: [u64; N_BINS],
    committed: [u64; N_BINS],
    used_b: [u64; N_BINS],
    used_n: [u64; N_BINS],
    areas: [u32; N_BINS],
    overflow: u32,
}

impl BinTab {
    fn boxed() -> Box<Self> {
        Box::new(BinTab {
            size: [0; N_BINS],
            reserved: [0; N_BINS],
            committed: [0; N_BINS],
            used_b: [0; N_BINS],
            used_n: [0; N_BINS],
            areas: [0; N_BINS],
            overflow: 0,
        })
    }
}

// L-71.1 (WP-71, Leijen): SCOPE-HEAP mimalloc attorno a `run_deferred` —
// il retained-form del canale defer PER COSTRUZIONE: ogni allocazione fatta
// mentre lo scope è attivo finisce nelle pagine del heap dedicato; ciò che
// sopravvive al pop (e ai teardown successivi) resta contato lì, il churn
// che muore non accumula. Il visitor emette le sue righe come
// `tag=mi_bin src=defer`.
//
// ADDENDUM STRUMENTO (pre-letto, forma P70-0-bis): il mimalloc v3 di questo
// tree non esporta più `mi_heap_set_default` — il push/pop del default-heap
// del lock è realizzato al livello del GLOBAL ALLOCATOR del binario census
// (php-cli `CountingMi`): a scope attivo `scoped_alloc`/`scoped_realloc`
// instradano su `mi_heap_malloc_aligned(defer_heap, …)`; `mi_free` è
// heap-agnostico (libera nella pagina proprietaria). La GRANDEZZA misurata
// è identica a quella lockata. Thread-safety: heap e flag thread_local
// (mai condivisi); il master wpdev è single-thread.
thread_local! {
    static DEFER_HEAP: std::cell::Cell<*mut std::os::raw::c_void> =
        const { std::cell::Cell::new(std::ptr::null_mut()) };
    static DEFER_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// RAII della finestra scope-heap: decrementa la profondità al drop
/// (rientranze annidate di run_deferred restano nello scope).
pub struct DeferHeapScope;

/// Entra nello scope-heap del canale defer (creandolo alla prima entrata).
/// Un fallimento di `mi_heap_new` degrada a no-op (heap null ⇒
/// `scoped_alloc` risponde None e l'allocazione resta sul default): il
/// letto per-src semplicemente non esiste, mai un crash del census.
pub fn defer_heap_enter() -> DeferHeapScope {
    DEFER_HEAP.with(|h| {
        if h.get().is_null() {
            h.set(unsafe { mi_heap_new() });
        }
    });
    DEFER_DEPTH.with(|d| d.set(d.get() + 1));
    DeferHeapScope
}

impl Drop for DeferHeapScope {
    fn drop(&mut self) {
        DEFER_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

#[inline]
fn defer_active_heap() -> *mut std::os::raw::c_void {
    if DEFER_DEPTH.with(|d| d.get()) == 0 {
        return std::ptr::null_mut();
    }
    DEFER_HEAP.with(|h| h.get())
}

/// Allocazione instradata sullo scope-heap quando attivo; `None` = usa il
/// default (scope inattivo o heap non creabile). Chiamata dal
/// `#[global_allocator]` del binario census.
#[inline]
pub fn scoped_alloc(size: usize, align: usize, zeroed: bool) -> Option<*mut u8> {
    let heap = defer_active_heap();
    if heap.is_null() {
        return None;
    }
    let p = unsafe {
        if zeroed {
            mi_heap_zalloc_aligned(heap, size, align)
        } else {
            mi_heap_malloc_aligned(heap, size, align)
        }
    };
    Some(p as *mut u8)
}

/// Realloc instradato sullo scope-heap quando attivo (un blocco nato su un
/// altro heap viene ricopiato nel defer-heap da mimalloc). `None` = default.
#[inline]
pub fn scoped_realloc(ptr: *mut u8, new_size: usize, align: usize) -> Option<*mut u8> {
    let heap = defer_active_heap();
    if heap.is_null() {
        return None;
    }
    let p = unsafe {
        mi_heap_realloc_aligned(heap, ptr as *mut std::os::raw::c_void, new_size, align)
    };
    Some(p as *mut u8)
}

unsafe extern "C" fn area_visitor(
    _heap: *const std::os::raw::c_void,
    area: *const MiHeapArea,
    _block: *mut std::os::raw::c_void,
    _block_size: usize,
    arg: *mut std::os::raw::c_void,
) -> bool {
    let t = &mut *(arg as *mut BinTab);
    let a = &*area;
    let key = if a.block_size > 65536 { LARGE_BUCKET } else { a.block_size };
    for i in 0..N_BINS {
        if t.size[i] == key || t.size[i] == 0 {
            t.size[i] = key;
            t.reserved[i] += a.reserved as u64;
            t.committed[i] += a.committed as u64;
            t.used_b[i] += (a.used as u64) * (a.full_block_size as u64);
            t.used_n[i] += a.used as u64;
            t.areas[i] += 1;
            return true;
        }
    }
    t.overflow += 1;
    true
}

unsafe extern "C" fn mi_out_write(
    msg: *const std::os::raw::c_char,
    arg: *mut std::os::raw::c_void,
) {
    let fd = arg as usize as i32;
    let len = libc::strlen(msg);
    let _ = libc::write(fd, msg as *const std::os::raw::c_void, len);
}

static PHYS_TICK: AtomicU64 = AtomicU64::new(0);
static PHYS_MARK: AtomicI64 = AtomicI64::new(256 << 20);
static PHYS_PEAK: AtomicI64 = AtomicI64::new(0);
static PHYS_WIN: AtomicU64 = AtomicU64::new(0);

/// Throttled physical-footprint window check, called from [`watermark`]
/// every 16384 census events (~tens of ms at media alloc rates).
fn phys_check() {
    let phys = phys_footprint();
    if phys == 0 {
        return;
    }
    PHYS_PEAK.fetch_max(phys as i64, Relaxed);
    let mark = PHYS_MARK.load(Relaxed);
    if phys as i64 >= mark
        && PHYS_MARK
            .compare_exchange(mark, phys as i64 + (128 << 20), Relaxed, Relaxed)
            .is_ok()
    {
        let win = PHYS_WIN.fetch_add(1, Relaxed) + 1;
        phys_window_dump(win as u32, phys, "mark_phys");
    }
}

/// One full window snapshot: mi_process_info + per-bin occupancy (main +
/// abandoned) into the census file, channel line, mimalloc stats into
/// `$PHPR_MI_STATS`, flag-file touch for the external vmmap supervisor.
/// `win == 0` is the exit snapshot.
fn phys_window_dump(win: u32, phys: u64, tag: &str) {
    use std::io::Write;
    // A-DL32 (Council WP-87, Leijen — A-DL30 closed): PHYS_PEAK is only
    // advanced by the throttled phys_check (every 16384 events), so a dump
    // printing a FRESH phys next to a STALE peak produced the impossible
    // phys_peak<phys rows (dl28s: 38.732.304 < 74.711.544 — a sampling
    // artifact, not an accounting error). Fold the fresh phys into the
    // peak BEFORE printing; mi_proc rows re-enter the corpus from here on
    // (KL-87-3 lifts post-fix).
    PHYS_PEAK.fetch_max(phys as i64, Relaxed);
    if let Ok(path) = std::env::var("PHPR_MEM_CENSUS") {
        if let Ok(mut f) =
            std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            let pid = std::process::id();
            let (mut el, mut us, mut sy, mut rss, mut prss, mut cm, mut pcm, mut flt) =
                (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
            unsafe {
                mi_process_info(
                    &mut el, &mut us, &mut sy, &mut rss, &mut prss, &mut cm, &mut pcm,
                    &mut flt,
                )
            };
            // A-DL37 (Council WP-88, Leijen): the census_line atomicity fix
            // covered census_line only — these rows still went out one write
            // PER FORMAT FRAGMENT and two concurrent teardowns garble them
            // MID-LINE exactly like the pid=pid= smoke. Same cure for every
            // row of this dump: format first, ONE write_all on O_APPEND.
            let _ = f.write_all(
                format!(
                    "pid={pid} tag=mi_proc win={win} ckpt={tag} mono_ms={} phys={phys} phys_peak={} rss={rss} peak_rss={prss} commit={cm} peak_commit={pcm}\n",
                    mono_ms(),
                    PHYS_PEAK.load(Relaxed),
                )
                .as_bytes(),
            );
            // WP-60 P2(a): in-process window context (Gregg R1/V4) — the VM's
            // registered renderer names the frame stack; never child stdout.
            let _ = f.write_all(
                format!(
                    "pid={pid} tag=ctx win={win} ckpt={tag} mono_ms={} top={}\n",
                    mono_ms(),
                    ctx_render().unwrap_or_else(|| "none".into()),
                )
                .as_bytes(),
            );
            let mut main_t = BinTab::boxed();
            let main_ok = unsafe {
                mi_heap_visit_blocks(
                    mi_heap_main(),
                    false,
                    area_visitor,
                    &mut *main_t as *mut BinTab as *mut std::os::raw::c_void,
                )
            };
            let mut ab_t = BinTab::boxed();
            let ab_ok = unsafe {
                mi_heap_visit_abandoned_blocks(
                    mi_heap_main(),
                    false,
                    area_visitor,
                    &mut *ab_t as *mut BinTab as *mut std::os::raw::c_void,
                )
            };
            // L-71.1: the defer scope-heap, when it exists (census builds
            // running the defer channel), reports as src=defer. A never-
            // entered heap emits a single NULLHEAP marker so the analyzer
            // can tell "absent" from "visit failed".
            let mut defer_t = BinTab::boxed();
            let defer_heap = DEFER_HEAP.with(|h| h.get());
            let defer_ok = if defer_heap.is_null() {
                let _ = f.write_all(
                    format!("pid={pid} tag=mi_bin win={win} ckpt={tag} src=defer visit=NULLHEAP\n").as_bytes(),
                );
                false
            } else {
                unsafe {
                    mi_heap_visit_blocks(
                        defer_heap,
                        false,
                        area_visitor,
                        &mut *defer_t as *mut BinTab as *mut std::os::raw::c_void,
                    )
                }
            };
            let defer_rows = !defer_heap.is_null();
            for (src, ok, t) in [
                ("main", main_ok, &main_t),
                ("aband", ab_ok, &ab_t),
                ("defer", defer_ok && defer_rows, &defer_t),
            ] {
                if src == "defer" && !defer_rows {
                    continue;
                }
                if !ok {
                    let _ = f.write_all(
                        format!("pid={pid} tag=mi_bin win={win} ckpt={tag} src={src} visit=FAILED\n")
                            .as_bytes(),
                    );
                    continue;
                }
                for i in 0..N_BINS {
                    if t.size[i] == 0 {
                        continue;
                    }
                    let _ = f.write_all(
                        format!(
                            "pid={pid} tag=mi_bin win={win} ckpt={tag} src={src} size={} reserved={} committed={} used_b={} used_n={} areas={}\n",
                            t.size[i], t.reserved[i], t.committed[i], t.used_b[i], t.used_n[i],
                            t.areas[i],
                        )
                        .as_bytes(),
                    );
                }
                if t.overflow > 0 {
                    let _ = f.write_all(
                        format!(
                            "pid={pid} tag=mi_bin win={win} ckpt={tag} src={src} size=OVERFLOW areas={}\n",
                            t.overflow
                        )
                        .as_bytes(),
                    );
                }
            }
            // --- A-DL41 (KL-89-1): environment-arm read-back, in-band ----
            // A mute arm (env exported but never consumed) is
            // indistinguishable from a no-effect arm without these rows.
            let eager = unsafe { mi_option_get(4) };
            let purge = unsafe { mi_option_get(15) };
            // A-DL46-census (Council WP-90): page_full_retain ord=36
            // (verified against the vendored enum with the SAME offset
            // that maps purge_delay to its live-bitten 15). The retain=0
            // discrimination arm reads back 0 against default 2 — a
            // positive that bites on its own (A-DL41 class).
            let retain = unsafe { mi_option_get(36) };
            let _ = f.write_all(
                format!(
                    "pid={pid} tag=mi_option win={win} ckpt={tag} name=arena_eager_commit ord=4 val={eager}\npid={pid} tag=mi_option win={win} ckpt={tag} name=purge_delay ord=15 val={purge}\npid={pid} tag=mi_option win={win} ckpt={tag} name=page_full_retain ord=36 val={retain}\n"
                )
                .as_bytes(),
            );
            for name in [
                "MIMALLOC_ARENA_EAGER_COMMIT",
                "MIMALLOC_PURGE_DELAY",
                "MIMALLOC_PAGE_FULL_RETAIN",
                "RUST_MIN_STACK",
            ] {
                let val = std::env::var(name).unwrap_or_else(|_| "unset".into());
                let _ = f.write_all(
                    format!("pid={pid} tag=env_readback win={win} ckpt={tag} name={name} val={val}\n")
                        .as_bytes(),
                );
            }
            // --- A-DL46-census (Council WP-90, Leijen — KL-90-4): per-theap
            // page census: one area-walk per live heap of the subprocess.
            // DECLARED (KL-90-3): heap identity is the VISIT INDEX (the
            // heap->thread mapping is not public API); at win=0 the census
            // runs post-teardown — abandoned/merged pages sit where the
            // visitor finds them. free_* fields = areas with used==0 that
            // are still committed: the page_full_retain (ord 36)
            // candidates, binned by block_size.
            {
                // A-MS50 (Council WP-91, Matsakis — KS-MS-91-3): the v89
                // visitors ALLOCATED (Vec::push growth, BTreeMap::entry)
                // via mimalloc while mi_subproc_visit_heaps enumerated the
                // heaps and while mi_heap_visit_blocks walked the visiting
                // thread's own default heap — mutation under C-side
                // iteration; "it did not crash" is not soundness. v90:
                // capacity pre-reserved BEFORE the visit (one upfront
                // reserve, pushes never reallocate), per-heap free-page
                // bins on the BinTab fixed-slot scheme, capacity overflows
                // DECLARED in-band, and the &mut re-derivation from `arg`
                // scoped so no Rust reference lives across the nested
                // visit (Stacked/Tree Borrows margin).
                const MAX_THEAPS: usize = 64;
                struct TheapAgg {
                    pages: usize,
                    committed: usize,
                    used_blocks: usize,
                    free_pages: usize,
                    free_committed: usize,
                    bin_size: [usize; N_BINS],
                    bin_fp: [u32; N_BINS],
                    bin_fc: [u64; N_BINS],
                    bin_overflow: u32,
                }
                impl TheapAgg {
                    fn zeroed() -> TheapAgg {
                        TheapAgg {
                            pages: 0,
                            committed: 0,
                            used_blocks: 0,
                            free_pages: 0,
                            free_committed: 0,
                            bin_size: [0; N_BINS],
                            bin_fp: [0; N_BINS],
                            bin_fc: [0; N_BINS],
                            bin_overflow: 0,
                        }
                    }
                }
                struct Acc {
                    heaps: Vec<TheapAgg>,   // reserved to MAX_THEAPS BEFORE the visit
                    heap_overflow: u32,
                }
                unsafe extern "C" fn theap_area_cb(
                    _h: *const std::os::raw::c_void,
                    area: *const MiHeapArea,
                    _block: *mut std::os::raw::c_void,
                    _bs: usize,
                    arg: *mut std::os::raw::c_void,
                ) -> bool {
                    let acc = unsafe { &mut *(arg as *mut Acc) };
                    let a = unsafe { &*area };
                    if let Some(h) = acc.heaps.last_mut() {
                        h.pages += 1;
                        h.committed += a.committed;
                        h.used_blocks += a.used;
                        if a.used == 0 {
                            h.free_pages += 1;
                            h.free_committed += a.committed;
                            let key = if a.block_size > 65536 { LARGE_BUCKET } else { a.block_size };
                            let mut placed = false;
                            for i in 0..N_BINS {
                                if h.bin_size[i] == key || h.bin_size[i] == 0 {
                                    h.bin_size[i] = key;
                                    h.bin_fp[i] += 1;
                                    h.bin_fc[i] += a.committed as u64;
                                    placed = true;
                                    break;
                                }
                            }
                            if !placed {
                                h.bin_overflow += 1;
                            }
                        }
                    }
                    true
                }
                unsafe extern "C" fn theap_heap_cb(
                    heap: *mut std::os::raw::c_void,
                    arg: *mut std::os::raw::c_void,
                ) -> bool {
                    {
                        let acc = unsafe { &mut *(arg as *mut Acc) };
                        if acc.heaps.len() == acc.heaps.capacity() {
                            acc.heap_overflow += 1;
                            return true;   // declared below, never silent
                        }
                        acc.heaps.push(TheapAgg::zeroed());
                    }   // &mut dropped BEFORE the nested visit (A-MS50)
                    unsafe { mi_heap_visit_blocks(heap, false, theap_area_cb, arg) };
                    true
                }
                let mut acc = Acc { heaps: Vec::new(), heap_overflow: 0 };
                acc.heaps.reserve_exact(MAX_THEAPS);   // the ONLY allocation, pre-visit
                let vok = unsafe {
                    mi_subproc_visit_heaps(
                        mi_subproc_current(),
                        theap_heap_cb,
                        &mut acc as *mut Acc as *mut std::os::raw::c_void,
                    )
                };
                if !vok {
                    let _ = f.write_all(
                        format!("pid={pid} tag=mi_theap_pages win={win} ckpt={tag} visit=FAILED\n")
                            .as_bytes(),
                    );
                }
                for (i, h) in acc.heaps.iter().enumerate() {
                    let _ = f.write_all(
                        format!(
                            "pid={pid} tag=mi_theap_pages win={win} ckpt={tag} heap={i} pages={} committed={} used_blocks={} free_pages={} free_committed={}\n",
                            h.pages, h.committed, h.used_blocks, h.free_pages, h.free_committed
                        )
                        .as_bytes(),
                    );
                    for b in 0..N_BINS {
                        if h.bin_size[b] == 0 {
                            continue;
                        }
                        let (bs, fp, fc) = (h.bin_size[b], h.bin_fp[b], h.bin_fc[b]);
                        let _ = f.write_all(
                            format!(
                                "pid={pid} tag=mi_theap_bin win={win} ckpt={tag} heap={i} bin={bs} free_pages={fp} free_committed={fc}\n"
                            )
                            .as_bytes(),
                        );
                    }
                    if h.bin_overflow > 0 {
                        let _ = f.write_all(
                            format!(
                                "pid={pid} tag=mi_theap_bin win={win} ckpt={tag} heap={i} bin=OVERFLOW areas={} (A-MS50 declared cap)\n",
                                h.bin_overflow
                            )
                            .as_bytes(),
                        );
                    }
                }
                let _ = f.write_all(
                    format!(
                        "pid={pid} tag=mi_theap_pages win={win} ckpt={tag} heaps_total={} heap_overflow={} declared=heap-by-visit-index,noalloc-visitors-A-MS50 (KL-90-3)\n",
                        acc.heaps.len(),
                        acc.heap_overflow
                    )
                    .as_bytes(),
                );
            }
            // --- A-BB55≡A-DL42: arena/commit term named in-band ----------
            // The 64 KiB probe buffer is allocated AFTER the mi_proc row of
            // this window was read and written — the commit= figure above
            // does not include it (same class as the BinTab boxes).
            let mut jbuf = vec![0u8; 64 * 1024];
            let jret = unsafe {
                mi_stats_get_json(jbuf.len(), jbuf.as_mut_ptr() as *mut std::os::raw::c_char)
            };
            if jret.is_null() {
                let _ = f.write_all(
                    format!("pid={pid} tag=mi_arena win={win} ckpt={tag} visit=FAILED\n").as_bytes(),
                );
            } else {
                let nul = jbuf.iter().position(|&b| b == 0).unwrap_or(jbuf.len());
                let json =
                    String::from_utf8_lossy(&jbuf[..nul]).replace(['\n', '\r'], " ");
                // The full-JSON row is the AUTHORITY (mimalloc's own field
                // names, no struct layout duplicated in Rust); the extracted
                // rows below are awk-friendly conveniences the verdict can
                // cross-check against it.
                let _ = f.write_all(
                    format!("pid={pid} tag=mi_arena_json win={win} ckpt={tag} json={json}\n").as_bytes(),
                );
                let sep = |c: char| c == ',' || c == ' ' || c == '}';
                let counter = |key: &str| -> Option<i64> {
                    let pat = format!("\"{key}\": ");
                    let i = json.find(&pat)? + pat.len();
                    let rest = &json[i..];
                    rest[..rest.find(sep)?].trim().parse().ok()
                };
                let count3 = |key: &str| -> Option<(i64, i64, i64)> {
                    let pat = format!("\"{key}\": {{");
                    let i = json.find(&pat)?;
                    let seg = &json[i..i + json[i..].find('}')? + 1];
                    let g = |k: &str| -> Option<i64> {
                        let p = format!("\"{k}\": ");
                        let j = seg.find(&p)? + p.len();
                        let r = &seg[j..];
                        r[..r.find(sep)?].trim().parse().ok()
                    };
                    Some((g("total")?, g("peak")?, g("current")?))
                };
                for key in ["committed", "reserved", "pages", "page_committed"] {
                    let row = match count3(key) {
                        Some((t, p, c)) => format!(
                            "pid={pid} tag=mi_arena win={win} ckpt={tag} key={key} total={t} peak={p} current={c}\n"
                        ),
                        None => format!(
                            "pid={pid} tag=mi_arena win={win} ckpt={tag} key={key} parse=FAILED\n"
                        ),
                    };
                    let _ = f.write_all(row.as_bytes());
                }
                // A-DL46 (Council WP-90, Leijen): `commit_calls` BANNED from
                // emission — the substring parse below is section-blind
                // (first occurrence GLOBAL) and the extracted val=0 against
                // a 150 MB committed was auto-incoherent: a row that can
                // lie silently is worse than no row. Re-admit only with a
                // section-aware parse (scoped to the arena JSON section).
                for key in
                    ["arena_count", "purged", "reset", "purge_calls", "mmap_calls"]
                {
                    let row = match counter(key) {
                        Some(v) => format!(
                            "pid={pid} tag=mi_arena win={win} ckpt={tag} key={key} val={v}\n"
                        ),
                        None => format!(
                            "pid={pid} tag=mi_arena win={win} ckpt={tag} key={key} parse=FAILED\n"
                        ),
                    };
                    let _ = f.write_all(row.as_bytes());
                }
                if let Some(i) = json.find("\"chunk_bins\": [") {
                    if let Some(e) = json[i..].find(']') {
                        let seg = &json[i..i + e + 1];
                        let _ = f.write_all(
                            format!("pid={pid} tag=mi_arena_chunk_bins win={win} ckpt={tag} {seg}\n")
                                .as_bytes(),
                        );
                    }
                }
            }
        }
    }
    dump_line(tag);
    // A-DL40 (Council WP-89, Leijen KL-89-4): this channel is NON-CORPUS.
    // mi_stats_print_out streams via the mimalloc fragment callback — the
    // writes are NOT atomic per row and concurrent teardowns can garble
    // them mid-line. No verdict may parse $PHPR_MI_STATS as evidence; the
    // atomic equivalents live in the census file (tag=mi_arena* rows).
    if let Ok(path) = std::env::var("PHPR_MI_STATS") {
        use std::os::unix::io::AsRawFd;
        if let Ok(mut f) =
            std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            // A-DL37 same class: single write for the stats header too.
            let _ = f.write_all(
                format!("== pid={} win={} phys={} ==\n", std::process::id(), win, phys).as_bytes(),
            );
            let _ = f.flush();
            unsafe {
                mi_stats_print_out(
                    Some(mi_out_write),
                    f.as_raw_fd() as usize as *mut std::os::raw::c_void,
                )
            };
        }
    }
    if let Ok(flag) = std::env::var("PHPR_PHYS_FLAG") {
        let _ = std::fs::write(flag, format!("{win} {phys}\n"));
    }
}

// ---------------------------------------------------------------------------
// WP-60 (P2/P3): counting-allocator meter, window context hook, abandoned
// positive-control, raw census lines. Census-build instrumentation only.
// ---------------------------------------------------------------------------

/// Cumulative layout bytes allocated/freed through the process-global
/// allocator. Fed by php-cli's census `#[global_allocator]` wrapper (the
/// binary owns the allocator choice; this crate only owns the counters).
static GA_ALLOC: AtomicU64 = AtomicU64::new(0);
static GA_FREE: AtomicU64 = AtomicU64::new(0);
// S-102 (A-LE-103-1): anche i CONTEGGI di eventi, non solo i byte — la gamba
// alloc si giudica a mem-census diretto (le stats a pagine sono cieche al
// churn malloc/free bilanciato, RC-LE-103-1).
static GA_ALLOC_N: AtomicU64 = AtomicU64::new(0);
static GA_FREE_N: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn galloc_note(bytes: usize) {
    GA_ALLOC.fetch_add(bytes as u64, Relaxed);
    GA_ALLOC_N.fetch_add(1, Relaxed);
    hist_note(bytes);
}

#[inline]
pub fn gfree_note(bytes: usize) {
    GA_FREE.fetch_add(bytes as u64, Relaxed);
    GA_FREE_N.fetch_add(1, Relaxed);
    // S-104 H-D (A-LE-105-1): il free era ASSUNTO simmetrico all'alloc
    // senza istogramma — ora la distribuzione lato free si MISURA.
    fhist_note(bytes);
}

// S-103 H-D (A-LE-104-1, RC-LE-104-7): il realloc DISAGGREGATO — contarlo
// come alloc+free pieni fa apparire un realloc in-place come churn doppio
// («2 alloc/35B» era esistenza, non cifra). Famiglia separata.
static GA_REALLOC_N: AtomicU64 = AtomicU64::new(0);
static GA_REALLOC_OLD: AtomicU64 = AtomicU64::new(0);
static GA_REALLOC_NEW: AtomicU64 = AtomicU64::new(0);
// Istogramma size-class degli ALLOC puri (bucket: ≤16 ≤32 ≤48 ≤64 ≤96
// ≤128 ≤256 ≤512 ≤1k ≤4k >4k): «~35 B medio» diventa una distribuzione.
pub const HIST_BUCKETS: [usize; 10] = [16, 32, 48, 64, 96, 128, 256, 512, 1024, 4096];
static GA_HIST: [AtomicU64; 11] = [const { AtomicU64::new(0) }; 11];

#[inline]
fn hist_note(bytes: usize) {
    let mut i = 0;
    while i < HIST_BUCKETS.len() && bytes > HIST_BUCKETS[i] {
        i += 1;
    }
    GA_HIST[i].fetch_add(1, Relaxed);
}

/// Un realloc: NON passa più da galloc/gfree (disaggregazione A-LE-104-1).
#[inline]
pub fn grealloc_note(old: usize, new: usize) {
    GA_REALLOC_N.fetch_add(1, Relaxed);
    GA_REALLOC_OLD.fetch_add(old as u64, Relaxed);
    GA_REALLOC_NEW.fetch_add(new as u64, Relaxed);
}

/// (realloc_n, old_bytes, new_bytes) cumulativi.
pub fn realloc_counters() -> (u64, u64, u64) {
    (
        GA_REALLOC_N.load(Relaxed),
        GA_REALLOC_OLD.load(Relaxed),
        GA_REALLOC_NEW.load(Relaxed),
    )
}

/// L'istogramma size-class degli alloc puri (11 bucket, vedi HIST_BUCKETS).
pub fn alloc_histogram() -> [u64; 11] {
    std::array::from_fn(|i| GA_HIST[i].load(Relaxed))
}

// S-104 H-D (A-LE-105-1): istogramma size-class dei FREE puri — stessa
// bucketizzazione degli alloc, famiglia separata (i realloc restano fuori
// da entrambe, disaggregazione A-LE-104-1).
static GA_FHIST: [AtomicU64; 11] = [const { AtomicU64::new(0) }; 11];

#[inline]
fn fhist_note(bytes: usize) {
    let mut i = 0;
    while i < HIST_BUCKETS.len() && bytes > HIST_BUCKETS[i] {
        i += 1;
    }
    GA_FHIST[i].fetch_add(1, Relaxed);
}

/// L'istogramma size-class dei free puri (stessi bucket degli alloc).
pub fn free_histogram() -> [u64; 11] {
    std::array::from_fn(|i| GA_FHIST[i].load(Relaxed))
}

// S-105 H-D gate G2 (censimento ARITÀ): istogramma dell'arità vista da
// bind_params — il choke-point esatto del perimetro della leva args.
// Bucket {0,1,2,3,4,≥5}; il chiamante è feature-gated (mem-census), la
// build di parità non contiene la chiamata.
static GA_ARITY: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];

#[inline]
pub fn arity_note(n: usize) {
    GA_ARITY[n.min(5)].fetch_add(1, Relaxed);
}

// S-106-D-12 (A-MA-107-2): contatore del backstop ArgPlace in decay_arg —
// ogni hit è un funnel di materializzazione mancato (in parità il chiamante
// è feature-gated; il degrade a NULL resta, ma non più silenzioso).
static GA_ARGPLACE_DECAY: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn argplace_decay_note() {
    GA_ARGPLACE_DECAY.fetch_add(1, Relaxed);
}

/// Hit del backstop ArgPlace (atteso 0: ogni valore ≠0 è un funnel mancato).
pub fn argplace_decay_hits() -> u64 {
    GA_ARGPLACE_DECAY.load(Relaxed)
}

/// L'istogramma dell'arità dei bind (bucket 0..4 esatti, ultimo = ≥5).
pub fn arity_histogram() -> [u64; 6] {
    std::array::from_fn(|i| GA_ARITY[i].load(Relaxed))
}

/// Snapshot of the cumulative (allocated, freed) counters — the clone-delta
/// meter behind the seed deep-size split (design60 P3b): read before and
/// after a `clone()`, the alloc difference is the clone's owned deep size in
/// layout bytes (mimalloc bin rounding excluded; Rc-shared subtrees excluded
/// — a clone bumps their refcount without allocating).
pub fn alloc_counters() -> (u64, u64) {
    (GA_ALLOC.load(Relaxed), GA_FREE.load(Relaxed))
}

/// S-102 (A-LE-103-1): event COUNTS (alloc_n, free_n) — zero in any build
/// whose global allocator does not route through the counting wrapper.
pub fn alloc_event_counters() -> (u64, u64) {
    (GA_ALLOC_N.load(Relaxed), GA_FREE_N.load(Relaxed))
}

// S-142 (criterio s142-criterio-census-rd1.md, REVISIONE dell'edit dopo STOP
// hash: la v1 con statiche sempre-compilate NON era byte-neutra): contatori
// del MECCANISMO L-RD1 — teardown inline di PhpArray. TUTTO sotto feature
// `mem-census`: nel pin non esiste nemmeno il simbolo.
#[cfg(feature = "mem-census")]
static RD1_ARRAYS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "mem-census")]
static RD1_ELEMS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "mem-census")]
static RD1_TOMBS: AtomicU64 = AtomicU64::new(0);

/// Pre-pass di conteggio all'ingresso di `PhpArray::drop` (repr non vuota,
/// elementi vivi, tombstoni) — SOLO build census; il drain del pin resta
/// testualmente intatto.
#[cfg(feature = "mem-census")]
#[inline]
pub fn rd1_note(len: usize, elems: u64, tombs: u64) {
    if len > 0 {
        RD1_ARRAYS.fetch_add(1, Relaxed);
    }
    RD1_ELEMS.fetch_add(elems, Relaxed);
    RD1_TOMBS.fetch_add(tombs, Relaxed);
}

/// Snapshot (arrays, elems, tombs) per la riga `zvalcensus_s142`.
#[cfg(feature = "mem-census")]
pub fn rd1_counters() -> (u64, u64, u64) {
    (
        RD1_ARRAYS.load(Relaxed),
        RD1_ELEMS.load(Relaxed),
        RD1_TOMBS.load(Relaxed),
    )
}

// S-143 istruttoria (criterio s143-criterio-istruttoria.md p.2, deliberato
// concilio): eventi di CREAZIONE BUFFER per arr/props — colmano il buco tra
// i `cum_n` di canale (box) e le coppie raw dell'allocatore. Un buffer
// Hashed conta UNA volta (entries+index insieme, banda dichiarata); il
// propsbuf conta la transizione accounted 0→>0 (slots+dyn insieme).
// TUTTO sotto feature `mem-census` (disciplina S-142 p.2).
#[cfg(feature = "mem-census")]
static S143_ARRBUF_N: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "mem-census")]
static S143_PROPSBUF_N: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "mem-census")]
#[inline]
pub fn s143_arrbuf_note() {
    S143_ARRBUF_N.fetch_add(1, Relaxed);
}

#[cfg(feature = "mem-census")]
#[inline]
pub fn s143_propsbuf_note() {
    S143_PROPSBUF_N.fetch_add(1, Relaxed);
}

/// Snapshot (arrbuf, propsbuf) per la riga `s143` del dump.
#[cfg(feature = "mem-census")]
pub fn s143_counters() -> (u64, u64) {
    (S143_ARRBUF_N.load(Relaxed), S143_PROPSBUF_N.load(Relaxed))
}

// S-144 tranche-2 (revisione S-143 az.1, criterio s144-criterio-tranche2.md):
// nascite dei box `Rc<RefCell<Zval>>` (classe `rczval` del criterio S-143
// p.2) al funnel unico `zval::zcell`/`zcell_prop` — il flag prop marca i
// SOLI siti inequivocabilmente di macchineria-proprietà (bracket
// [strict, loose]: la classificazione dei siti misti resta NO, dichiarata)
// — più il contatore `vecargs` (Vec argomenti con buffer allocato ai funnel
// bind_params/decay_args; path diretto no-alloc e nativi fuori perimetro,
// dichiarato). TUTTO sotto feature `mem-census` (disciplina S-142 p.2).
#[cfg(feature = "mem-census")]
static S144_RCZVAL_N: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "mem-census")]
static S144_RCZVAL_PROP_N: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "mem-census")]
static S144_VECARGS_N: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "mem-census")]
#[inline]
pub fn s144_rczval_note(prop: bool) {
    S144_RCZVAL_N.fetch_add(1, Relaxed);
    if prop {
        S144_RCZVAL_PROP_N.fetch_add(1, Relaxed);
    }
}

#[cfg(feature = "mem-census")]
#[inline]
pub fn s144_vecargs_note() {
    S144_VECARGS_N.fetch_add(1, Relaxed);
}

// Revisione S-143 az.5: il box sintetico di `copy_with_id(0)`
// (vm/mod.rs, serializzazione) NON passa dal mint oggetti — tick dedicato
// perché il suo peso smetta di essere «probabile piccolo» e diventi cifra.
#[cfg(feature = "mem-census")]
static S144_OBJSYNTH_N: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "mem-census")]
#[inline]
pub fn s144_objsynth_note() {
    S144_OBJSYNTH_N.fetch_add(1, Relaxed);
}

#[cfg(feature = "mem-census")]
pub fn s144_objsynth() -> u64 {
    S144_OBJSYNTH_N.load(Relaxed)
}

/// Snapshot (rczval, rczval_prop, vecargs) per la riga `s144` del dump.
#[cfg(feature = "mem-census")]
pub fn s144_counters() -> (u64, u64, u64) {
    (
        S144_RCZVAL_N.load(Relaxed),
        S144_RCZVAL_PROP_N.load(Relaxed),
        S144_VECARGS_N.load(Relaxed),
    )
}

/// Monotonic milliseconds since the first census event in the process —
/// stamped on window lines so the external supervisor's log joins on time,
/// never on child stdout offsets (Gregg V4).
pub fn mono_ms() -> u64 {
    static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    T0.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64
}

/// A-BB47 (Council WP-87): monotonic MICROseconds — the lowering spans of
/// the overlap canary are ~100 µs–3 ms, below ms resolution. Own epoch
/// (first call); only span-vs-span comparisons within one process are
/// meaningful — never join against mono_ms.
pub fn mono_us() -> u64 {
    static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    T0.get_or_init(std::time::Instant::now).elapsed().as_micros() as u64
}

/// Window context hook (Gregg R1): the VM registers a renderer that formats
/// its current PHP stack top; [`phys_window_dump`] calls it in-process on
/// every window so each window has a test/frame address.
static CTX_HOOK: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn set_ctx_hook(f: fn(&mut String)) {
    CTX_HOOK.store(f as usize, Relaxed);
}

fn ctx_render() -> Option<String> {
    let p = CTX_HOOK.load(Relaxed);
    if p == 0 {
        return None;
    }
    // SAFETY: the only writer is `set_ctx_hook`, which stores a valid
    // `fn(&mut String)` pointer; 0 is filtered above.
    let f: fn(&mut String) = unsafe { std::mem::transmute(p) };
    let mut s = String::new();
    f(&mut s);
    Some(s)
}

/// A-AH33 (Council WP-84, KS-AH-84-2): SEMANTIC identity of the counting
/// surface — a binary hash discriminates everything and therefore nothing;
/// retained/net figures are cross-campaign comparable ONLY between raws
/// carrying the SAME alloc_id. Bump = same-commit, named, at every change
/// of the counting surface (allocator wrapper, deep_size coverage,
/// net-window semantics). History: v1 = pre-S82 (net instrument DEAD in
/// mem-census builds, allocator bare); v2 = S-82.0 MemCountingMi.
/// Pinned by gate-census-twin.sh (source-const tooth).
pub const ALLOC_ID: &str = "memcount-v2-s82";

/// Raw appended census line (pid-prefixed) for census-build reporters that
/// live outside this crate (seed split, unit-per-path histogram).
pub fn census_line(line: &str) {
    use std::io::Write;
    let Ok(path) = std::env::var("PHPR_MEM_CENSUS") else { return };
    let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
        return;
    };
    // S-86.0 (found by the A-PP36 smoke): writeln! issues one write PER
    // FORMAT FRAGMENT — two threads tearing down concurrently interleave
    // MID-LINE ("pid=pid=90234..."), garbling both rows for every parser.
    // O_APPEND makes a SINGLE write atomic: format first, write once.
    let buf = format!("pid={} {}\n", std::process::id(), line);
    let _ = f.write_all(buf.as_bytes());
}

/// A-DL31 (Council WP-87, Leijen): per-THREAD bin occupancy — MUST be
/// called ON the thread whose heap is being read (mimalloc default heaps
/// are thread-local; a foreign thread cannot visit them). Emits
/// `tag=mi_bin thr=<n> src=tls` rows with the window-dump columns plus a
/// `tag=mi_bin_thr_sum` closing row. The physical half of the ×W budget
/// joins the committed sum against the time -l per-worker delta
/// (3.605.572 B ±5% predicted).
pub fn mi_bin_thread_rows(thr: usize) {
    // Probe allocation: freshly boxed by THIS thread => mi_heap_of returns
    // this thread's default heap (mi_heap_get_default is not exported by
    // the tree's mimalloc v3).
    let probe = Box::new(0u8);
    let heap = unsafe { mi_heap_of(&*probe as *const u8 as *const std::os::raw::c_void) };
    if heap.is_null() {
        census_line(&format!("tag=mi_bin thr={thr} src=tls visit=NULLHEAP"));
        return;
    }
    let mut t = BinTab::boxed();
    let ok = unsafe {
        mi_heap_visit_blocks(
            heap,
            false,
            area_visitor,
            &mut *t as *mut BinTab as *mut std::os::raw::c_void,
        )
    };
    drop(probe);
    if !ok {
        census_line(&format!("tag=mi_bin thr={thr} src=tls visit=FAILED"));
        return;
    }
    let (mut committed, mut reserved, mut used) = (0u64, 0u64, 0u64);
    for i in 0..N_BINS {
        if t.size[i] == 0 {
            continue;
        }
        reserved += t.reserved[i];
        committed += t.committed[i];
        used += t.used_b[i];
        census_line(&format!(
            "tag=mi_bin thr={thr} src=tls size={} reserved={} committed={} used_b={} used_n={} areas={}",
            t.size[i], t.reserved[i], t.committed[i], t.used_b[i], t.used_n[i], t.areas[i],
        ));
    }
    if t.overflow > 0 {
        census_line(&format!("tag=mi_bin thr={thr} src=tls size=OVERFLOW areas={}", t.overflow));
    }
    // heap=<ptr> in-band: if two threads ever report the SAME heap pointer
    // the per-thread reading is reading ONE heap twice — the verdict must
    // refuse the split (honesty tooth, S-86.0 smoke showed near-identical
    // sums across threads).
    census_line(&format!(
        "tag=mi_bin_thr_sum thr={thr} heap={heap:p} reserved={reserved} committed={committed} used_b={used} alloc_id={ALLOC_ID}"
    ));
}

/// WP-60 P2(b): positive control of the abandoned-blocks visitor (Gregg
/// R5a) — a thread leaks ~1.25MiB and dies; either the abandoned visitor
/// sees it (SEEN), or the main-heap visit absorbed the pages
/// (ADOPTED_MAIN: v3 reclaimed the dead thread's pages), or the column is
/// dead (DEAD) and the reports must label it so.
pub fn abandoned_positive_control() {
    fn visit(aband: bool) -> (bool, u64, u64) {
        let mut t = BinTab::boxed();
        let arg = &mut *t as *mut BinTab as *mut std::os::raw::c_void;
        let ok = unsafe {
            if aband {
                mi_heap_visit_abandoned_blocks(mi_heap_main(), false, area_visitor, arg)
            } else {
                mi_heap_visit_blocks(mi_heap_main(), false, area_visitor, arg)
            }
        };
        let (mut ub, mut un) = (0u64, 0u64);
        for i in 0..N_BINS {
            ub += t.used_b[i];
            un += t.used_n[i];
        }
        (ok, ub, un)
    }
    let (_, mb0, mn0) = visit(false);
    let joined = std::thread::spawn(|| {
        for _ in 0..32 {
            std::mem::forget(vec![0xA5u8; 40960].into_boxed_slice());
        }
    })
    .join();
    let (ok, ab, an) = visit(true);
    let (_, mb1, mn1) = visit(false);
    let planted = 32u64 * 40960;
    let main_delta = mb1.saturating_sub(mb0);
    census_line(&format!(
        "tag=aband_check join_ok={} visit_ok={ok} aband_b={ab} aband_n={an} \
         main_delta_b={main_delta} main_delta_n={} verdict={}",
        joined.is_ok(),
        mn1.saturating_sub(mn0),
        if ok && ab >= planted {
            "SEEN"
        } else if main_delta >= planted {
            "ADOPTED_MAIN"
        } else {
            "DEAD"
        },
    ));
}
