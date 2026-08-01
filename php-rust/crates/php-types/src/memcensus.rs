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
        " created={} units={} unit_slack={} proxy_peak={}\n",
        GAUGES[G_CREATED].load(Relaxed),
        GAUGES[G_UNITS].load(Relaxed),
        UNIT_SLACK.load(Relaxed),
        PROXY_PEAK.load(Relaxed),
    ));
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
            if seen.insert(std::rc::Rc::as_ptr(s) as usize) {
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
                    if seen.insert(std::rc::Rc::as_ptr(ks) as usize) {
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

const TASK_VM_INFO: u32 = 22;

extern "C" {
    static mach_task_self_: u32;
    fn task_info(task: u32, flavor: u32, info: *mut i32, count: *mut u32) -> i32;
}

/// The ledger the judges' metric reports (`/usr/bin/time -l` "peak memory
/// footprint" is the peak of this): internal + compressed, in bytes.
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

// mimalloc v3 public API (statically linked via the `mimalloc` crate; no
// bindings in libmimalloc-sys for these, so they are declared here).
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
            let _ = writeln!(
                f,
                "pid={pid} tag=mi_proc win={win} mono_ms={} phys={phys} phys_peak={} rss={rss} peak_rss={prss} commit={cm} peak_commit={pcm}",
                mono_ms(),
                PHYS_PEAK.load(Relaxed),
            );
            // WP-60 P2(a): in-process window context (Gregg R1/V4) — the VM's
            // registered renderer names the frame stack; never child stdout.
            let _ = writeln!(
                f,
                "pid={pid} tag=ctx win={win} mono_ms={} top={}",
                mono_ms(),
                ctx_render().unwrap_or_else(|| "none".into()),
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
                let _ = writeln!(f, "pid={pid} tag=mi_bin win={win} src=defer visit=NULLHEAP");
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
                    let _ = writeln!(f, "pid={pid} tag=mi_bin win={win} src={src} visit=FAILED");
                    continue;
                }
                for i in 0..N_BINS {
                    if t.size[i] == 0 {
                        continue;
                    }
                    let _ = writeln!(
                        f,
                        "pid={pid} tag=mi_bin win={win} src={src} size={} reserved={} committed={} used_b={} used_n={} areas={}",
                        t.size[i], t.reserved[i], t.committed[i], t.used_b[i], t.used_n[i],
                        t.areas[i],
                    );
                }
                if t.overflow > 0 {
                    let _ = writeln!(
                        f,
                        "pid={pid} tag=mi_bin win={win} src={src} size=OVERFLOW areas={}",
                        t.overflow
                    );
                }
            }
        }
    }
    dump_line(tag);
    if let Ok(path) = std::env::var("PHPR_MI_STATS") {
        use std::os::unix::io::AsRawFd;
        if let Ok(mut f) =
            std::fs::OpenOptions::new().create(true).append(true).open(path)
        {
            let _ = writeln!(f, "== pid={} win={} phys={} ==", std::process::id(), win, phys);
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

#[inline]
pub fn galloc_note(bytes: usize) {
    GA_ALLOC.fetch_add(bytes as u64, Relaxed);
}

#[inline]
pub fn gfree_note(bytes: usize) {
    GA_FREE.fetch_add(bytes as u64, Relaxed);
}

/// Snapshot of the cumulative (allocated, freed) counters — the clone-delta
/// meter behind the seed deep-size split (design60 P3b): read before and
/// after a `clone()`, the alloc difference is the clone's owned deep size in
/// layout bytes (mimalloc bin rounding excluded; Rc-shared subtrees excluded
/// — a clone bumps their refcount without allocating).
pub fn alloc_counters() -> (u64, u64) {
    (GA_ALLOC.load(Relaxed), GA_FREE.load(Relaxed))
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
