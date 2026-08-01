//! `php-server` — HTTP front end for phpr engine.
//!
//! Two modes:
//! 1. `--cli-server` (default): Reuses the CLI SAPI from `phpr -S`
//! 2. `--axum`: Axum HTTP front end + worker-actor pool (WP-77.6, retimed
//!    S-78.1.5): N OS worker threads; each REQUEST gets a fresh RetainSet
//!    (P-2 unit-module pin, dies with the request — the per-worker arena
//!    leaked, KS-DS-78-4) and a FRESH Vm. Cross-request unit reuse is the
//!    thread-local unit cache (WP-63/66). There is NO `Vm::reset()` and NO
//!    N=1 thread-local Vm reuse — REJECTED in WP-77.2 (A-TH3/A-BG2 purge).
//!    Request isolation: `request_end()` reset + Vm death (A-DS2 matrix in
//!    worker_pool.rs).
//!
//! Gate: G-APERTURA-2 (A-SK2, Council WP-78)
//! =========================================
//! Executable gate: `wp78-harness/gate-axum/run-gate.sh` — builds this crate
//! with the axum-server feature, records the php-server binary hash (A-SK5:
//! the Axum baseline binary is php-server, NEVER phpr), starts `--axum`, runs
//! sequential POSTs (hello x2, stateful x3) and byte-compares the bodies
//! (KS-SK-78.2; stateful drift = KS-DS-78-1). A PASS may only be declared by
//! citing that command and the hash it logs.
//! Scope (A-BG1): correctness only — alloc/CPU/footprint are measured
//! separately under the WP-78 protocol (wp78-harness/design78.md).
//!
//! Feature identity (A-AH2, Council WP-78)
//! =======================================
//! Cargo features UNION with `default = ["cli-server"]`, so the real build
//! matrix is a TRIPLE, verified by `wp78-harness/gate-feature-matrix.sh`
//! (all with -D warnings; KS-AH-78-2):
//!   1. default                                       → cli-server only
//!      (absence of axum symbols in the binary is asserted)
//!   2. --no-default-features --features axum-server  → axum only
//!   3. --features axum-server                        → UNION (both modes;
//!      the deployed dual-mode binary)
//! `cargo build --features axum-server` does NOT produce an axum-only binary —
//! it is the union build (the old A-AH1 comment claiming a pair was wrong).

use std::ffi::OsString;
use std::process::ExitCode;

#[cfg(not(any(feature = "census-instrumentation", feature = "mem-census")))]
#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

// A-DL15 (Council WP-83): the mem-census instrument gained a NET bracket
// (net-at-lower → `main_program_net`/`rw_main_net`). With plain mimalloc the
// mem-census build left `php_types::memcensus::alloc_counters()` frozen at
// (0,0) — the bracket (and every CensusNetWindow in this binary) was
// silently DEAD. This wrapper feeds the (alloc, free) pair; it produces
// counters only — footprint/peak verdicts still come EXCLUSIVELY from the
// non-instrumented twin (KB-78-5/KL-78-5 unchanged).
#[cfg(all(feature = "mem-census", not(feature = "census-instrumentation")))]
#[global_allocator]
static GLOBAL_ALLOC: mem_alloc::MemCountingMi = mem_alloc::MemCountingMi;

#[cfg(all(feature = "mem-census", not(feature = "census-instrumentation")))]
pub mod mem_alloc {
    use std::alloc::{GlobalAlloc, Layout};

    pub struct MemCountingMi;

    unsafe impl GlobalAlloc for MemCountingMi {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            php_types::memcensus::galloc_note(l.size());
            unsafe { mimalloc::MiMalloc.alloc(l) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            php_types::memcensus::gfree_note(l.size());
            unsafe { mimalloc::MiMalloc.dealloc(p, l) }
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            php_types::memcensus::galloc_note(l.size());
            unsafe { mimalloc::MiMalloc.alloc_zeroed(l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            php_types::memcensus::galloc_note(n);
            php_types::memcensus::gfree_note(l.size());
            unsafe { mimalloc::MiMalloc.realloc(p, l, n) }
        }
    }
}

// S-78.1.6 (A-BB7): the census build swaps in a COUNTING wrapper around
// mimalloc. KB-78-5/KL-78-5: any footprint/peak figure from this build is
// NULL — verdicts come from the non-instrumented twin; this build only
// produces counters.
#[cfg(feature = "census-instrumentation")]
#[global_allocator]
static GLOBAL_ALLOC: census_alloc::CountingAlloc = census_alloc::CountingAlloc;

#[cfg(feature = "census-instrumentation")]
pub mod census_alloc {
    use std::alloc::{GlobalAlloc, Layout};
    use std::sync::atomic::{AtomicU64, Ordering};

    pub static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
    pub static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

    /// (calls, bytes) since process start — phase attribution diffs these at
    /// the phase boundaries declared in design78 §Contatori.
    pub fn snapshot() -> (u64, u64) {
        (ALLOC_CALLS.load(Ordering::Relaxed), ALLOC_BYTES.load(Ordering::Relaxed))
    }

    pub struct CountingAlloc;

    unsafe impl GlobalAlloc for CountingAlloc {
        // A-DL15 (Council WP-83): on the UNION build (census + mem-census)
        // the net pair feeds the memcensus counters too — the census gross
        // semantics (ALLOC_* untouched by dealloc) stay EXACTLY as before.
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(l.size() as u64, Ordering::Relaxed);
            #[cfg(feature = "mem-census")]
            php_types::memcensus::galloc_note(l.size());
            unsafe { mimalloc::MiMalloc.alloc(l) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            #[cfg(feature = "mem-census")]
            php_types::memcensus::gfree_note(l.size());
            unsafe { mimalloc::MiMalloc.dealloc(p, l) }
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(l.size() as u64, Ordering::Relaxed);
            #[cfg(feature = "mem-census")]
            php_types::memcensus::galloc_note(l.size());
            unsafe { mimalloc::MiMalloc.alloc_zeroed(l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOC_BYTES.fetch_add(n as u64, Ordering::Relaxed);
            #[cfg(feature = "mem-census")]
            {
                php_types::memcensus::galloc_note(n);
                php_types::memcensus::gfree_note(l.size());
            }
            unsafe { mimalloc::MiMalloc.realloc(p, l, n) }
        }
    }
}

#[cfg(feature = "axum-server")]
mod worker_pool;

const USAGE: &str = "usage: php-server [--host HOST] [--port PORT] [--docroot|-t DIR] [--axum] [--workers N] [--tier0] [ROUTER.php]\n\
       defaults: --host 127.0.0.1 --port 8080 --docroot . --cli-server --workers 0 (= CPU count, axum mode only)\n\
       --tier0: axum floor baseline (KG-79.D) — static 200 \"tier0\", no pool, no Vm";

fn missing(flag: &str) -> ExitCode {
    eprintln!("php-server: {flag} requires a value\n{USAGE}");
    ExitCode::from(2)
}

#[cfg(feature = "axum-server")]
mod axum_handler {
    use std::process::ExitCode;
    use std::sync::Arc;
    use axum::{
        http::{StatusCode, Uri},
        extract::State,
        Router, response::IntoResponse,
    };
    use tokio::sync::oneshot;

    /// S-77.6.5.2.3: Application state (shared server-lifetime).
    pub struct AppState {
        pool: Arc<crate::worker_pool::WorkerPool>,
        docroot: String,
        /// A-PP4 gate hook (Council WP-80): DISPATCHER-side panic trigger,
        /// armed only by PHPR_TEST_WORKER_PANIC (same env as the worker
        /// trigger, distinct magic path). Unarmed, the path is an ordinary
        /// docroot lookup. Read once at startup, never per request.
        test_panic: bool,
    }

    /// PHP handler accepting path.php requests
    async fn php_handler(
        State(state): State<Arc<AppState>>,
        uri: Uri,
    ) -> impl IntoResponse {
        // Extract path from request URI
        let path = uri.path().to_string();
        let request_path = if path.is_empty() || path == "/" {
            "/index.php".to_string()
        } else {
            path
        };

        // S-79.0.3 (A-AH13/A-DL5): idle-window probe — answered from the
        // ROUTER task, no pool, no worker, no Vm. The driver brackets an
        // idle period with two probes; the counter drift between them is
        // the cross-thread background noise the per-phase diffs absorb.
        // Anchored ends_with (S-78.1 lesson: magic paths anchored at the
        // END, never by equality — the docroot prefix varies).
        #[cfg(feature = "census-instrumentation")]
        if request_path.ends_with("/__census_global") {
            let (calls, bytes) = crate::census_alloc::snapshot();
            // A-DL14 (Council WP-82): same gross-churn bytes as the census
            // line — same in-band tag (A-DL10). Appended LAST: the idle
            // parser reads fields $3/$5 positionally.
            eprintln!("census-global: calls={calls} bytes={bytes} gross=1");
            return (StatusCode::OK, b"census-global\n".to_vec());
        }

        // A-BG32 (Council WP-84): drain the per-request ns samples to
        // stderr — called by the slope campaign AFTER the timed window
        // (WP-64: no timed window contains its own logging I/O). Lives in
        // the UNION arm (env-gated, one branch per request when unarmed,
        // declared): the slope is measured on the union binary, never a
        // census build (KL-84-3). ends_with-anchored (S-78.1 lesson).
        if request_path.ends_with("/__reqns") {
            if !crate::worker_pool::req_ns_armed() {
                return (StatusCode::OK, b"reqns: unarmed\n".to_vec());
            }
            let mut v = crate::worker_pool::REQ_NS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let n = v.len();
            // A-PP28 (Council WP-85): `w=` is IN-BAND — at W>1 the samples
            // of different workers interleave in the global Mutex and the
            // slope verdict refuses them fail-closed (KS-PP-85-1).
            let w = state.pool.worker_count();
            let mut line = format!("reqns: n={n} w={w} arm=axum-worker unit=ns ns=");
            for (i, x) in v.iter().enumerate() {
                if i > 0 {
                    line.push(',');
                }
                line.push_str(&x.to_string());
            }
            v.clear();
            drop(v);
            eprintln!("{line}");
            return (StatusCode::OK, format!("reqns-drained n={n}\n").into_bytes());
        }

        // A-PP4 (Council WP-80): a panic in the AXUM task used to be caught
        // by tokio — dead task, hung request, LIVE process: the "silently
        // dead" state KS-PP-6 bans, on the dispatcher side. The abort
        // panic-hook (main) turns it into the same fail-fast abort as a
        // worker panic; this trigger lets gate-worker-panic Phase C PROVE
        // it on the real binary. ends_with-anchored (S-78.1 lesson).
        if state.test_panic && request_path.ends_with("/__phpr_panic_dispatcher") {
            panic!("PHPR_TEST_WORKER_PANIC armed: deliberate dispatcher panic (A-PP4 gate)");
        }

        // Read PHP source from disk (docroot + request_path)
        let file_path = format!("{}{}", state.docroot, request_path);
        let source = match std::fs::read(&file_path) {
            Ok(s) => s,
            Err(e) => {
                let msg = format!("404: {} ({})\n", file_path, e);
                return (StatusCode::NOT_FOUND, msg.into_bytes());
            }
        };

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();

        // Build handler metadata. A-BB5 (Council WP-78): HTTP method/body are
        // intentionally NOT carried — the Vm has no channel for them yet
        // (request_start seeds web superglobals from the SAPI layer, not from
        // here); carrying dead allocs across the channel would be counted as
        // "Axum overhead" by the census. Wiring real request metadata into the
        // superglobals is WP-79+ scope.
        // S-78.1.4 (A-TH8): the script name is the resolved FILESYSTEM path
        // (FPM's SCRIPT_FILENAME), not the request URI — error banners must
        // carry the same path the oracle renders for full-body cmp.
        let meta = crate::worker_pool::WorkerHandlerMeta {
            path: file_path.clone(),
            source,
        };

        // Dispatch to worker pool
        let task = crate::worker_pool::WorkerTask {
            meta,
            response_tx: tx,
        };

        if let Err(e) = state.pool.dispatch(task) {
            let msg = format!("Pool dispatch failed: {}\n", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, msg.into_bytes());
        }

        // Await response from worker
        match rx.await {
            Ok((body, status)) => (status, body),
            Err(_) => {
                let msg = b"Worker dropped response channel\n".to_vec();
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        }
    }

    pub async fn start_server(
        host: &str,
        port: u16,
        docroot: Option<&std::ffi::OsStr>,
        workers: usize,
        tier0: bool,
    ) -> ExitCode {
        let addr = format!("{host}:{port}");
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("php-server: failed to bind {addr}: {e}");
                return ExitCode::from(1);
            }
        };

        // Tier-0 (KG-79.D, S-78.1.6): the Axum FLOOR — tokio runtime +
        // listener + router only. NO pool, NO worker, NO Vm: the handler
        // answers from the router task. This is the traced executable
        // configuration design78 §Tier-0 requires for the baseline.
        if tier0 {
            let app = Router::new().fallback(|| async {
                (axum::http::StatusCode::OK, b"tier0\n".to_vec())
            });
            eprintln!("php-server (Axum TIER-0) listening on http://{addr} (no pool, no Vm)");
            let shutdown = async {
                let mut term =
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("install SIGTERM handler");
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
                eprintln!("php-server: shutdown signal received, draining");
            };
            if let Err(e) = axum::serve(listener, app)
                .with_graceful_shutdown(shutdown)
                .await
            {
                eprintln!("php-server: server error: {e}");
                return ExitCode::from(1);
            }
            return ExitCode::SUCCESS;
        }

        // Resolve docroot
        let docroot_str = docroot
            .and_then(|d| d.to_str())
            .unwrap_or(".")
            .to_string();

        // Initialize worker pool (A-BB3: size from --workers; 0 = CPU count.
        // Each worker retains its own RetainSet: footprint scales ~linearly
        // with N — measurements run --workers 1)
        let pool_context = crate::worker_pool::WorkerPoolContext::new();
        let pool = crate::worker_pool::WorkerPool::new(pool_context, workers);

        let app_state = Arc::new(AppState {
            pool: Arc::clone(&pool),
            docroot: docroot_str.clone(),
            test_panic: std::env::var_os("PHPR_TEST_WORKER_PANIC").is_some(),
        });

        // Router: use fallback to catch-all paths
        let app = Router::new()
            .fallback(php_handler)
            .with_state(app_state);

        eprintln!("php-server (Axum) listening on http://{addr} (docroot: {docroot_str})");

        // A-DL3 (Council WP-78): clean exit path — /usr/bin/time -l and
        // MIMALLOC_SHOW_STATS only report at process exit; kill -9 = zero
        // stats. SIGTERM/SIGINT drain the server and return from main().
        let shutdown = async {
            let mut term =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("install SIGTERM handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
            eprintln!("php-server: shutdown signal received, draining");
        };
        if let Err(e) = axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
        {
            eprintln!("php-server: server error: {e}");
            return ExitCode::from(1);
        }
        // A-MS7/A-DL6 (Council WP-79): serve() returning means the router —
        // the only other pool owner — is gone; unwrap and JOIN the workers
        // BEFORE process exit so /usr/bin/time -l and MIMALLOC_SHOW_STATS
        // report a fully torn-down process (KS-MS-5). If a straggler still
        // holds the Arc the figures are declared ADVISORY (KL-78-4), never
        // silently blessed.
        match Arc::try_unwrap(pool) {
            Ok(p) => p.shutdown(),
            Err(_) => eprintln!(
                "php-server: worker pool still referenced at exit — exit stats ADVISORY (KL-78-4)"
            ),
        }
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    php_runtime::logging::init();

    // S-79.0.3: hand the runtime-side census brackets (a1/a3 split) the
    // counting allocator's snapshot. Install-once; a duplicate set is a no-op.
    #[cfg(feature = "census-instrumentation")]
    let _ = php_runtime::alloc_census::SNAPSHOT_FN.set(census_alloc::snapshot);

    let mut host = String::from("127.0.0.1");
    let mut port: u16 = 8080;
    let mut docroot: Option<OsString> = None;
    let mut router: Option<OsString> = None;
    let mut use_axum = false;
    // 0 = num_cpus (axum mode only; ignored by the cli-server path)
    let mut workers: usize = 0;
    // KG-79.D: Tier-0 axum-floor baseline (axum mode only)
    let mut tier0 = false;

    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--host" => match args.next() {
                Some(v) => host = v.to_string_lossy().into_owned(),
                None => return missing("--host"),
            },
            "--port" => {
                let Some(v) = args.next() else { return missing("--port") };
                match v.to_string_lossy().parse() {
                    Ok(p) => port = p,
                    Err(_) => {
                        eprintln!("php-server: invalid port {:?}\n{USAGE}", v);
                        return ExitCode::from(2);
                    }
                }
            }
            "--docroot" | "-t" => match args.next() {
                Some(v) => docroot = Some(v),
                None => return missing("--docroot"),
            },
            // Uniform in both feature configs (the cfg(not) block below rejects
            // it when the feature is off) — keeps use_axum read+mutated in every
            // build, so -D warnings holds on the whole triple (A-AH2).
            "--axum" => use_axum = true,
            // Uniform in both feature configs, same rationale as --axum.
            "--tier0" => tier0 = true,
            "--workers" => {
                // A-BB3/A-DL2 (Council WP-78): pool size must be controllable —
                // every alloc/footprint measurement runs --workers 1 (steady
                // state per RetainSet); the num_cpus default multiplies the
                // retained footprint by N.
                let Some(v) = args.next() else { return missing("--workers") };
                match v.to_string_lossy().parse() {
                    Ok(n) => workers = n,
                    Err(_) => {
                        eprintln!("php-server: invalid workers {:?}\n{USAGE}", v);
                        return ExitCode::from(2);
                    }
                }
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            _ if router.is_none() && !arg.to_string_lossy().starts_with('-') => {
                router = Some(arg);
            }
            other => {
                eprintln!("php-server: unknown argument {other:?}\n{USAGE}");
                return ExitCode::from(2);
            }
        }
    }

    #[cfg(feature = "axum-server")]
    if use_axum {
        // A-MS4 (strong form) + A-PP4 (Council WP-80): fail-fast lives in
        // the panic HOOK — no unwind survives anywhere in the server. A
        // panic inside an axum task was otherwise swallowed by tokio (dead
        // task, hung request, live process — KS-PP-6's "silently dead" on
        // the dispatcher side). The default hook prints report + backtrace
        // first, then abort. The worker catch_unwind stays as belt and
        // braces (a dependency could swap the hook). Axum mode only: the
        // CLI server keeps stock panic behavior.
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            default_hook(info);
            eprintln!("php-server: panic — aborting (A-PP9/A-PP4 fail-fast, KS-PP-6)");
            std::process::abort();
        }));
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("php-server: failed to create tokio runtime: {e}");
                return ExitCode::from(1);
            }
        };
        return rt.block_on(axum_handler::start_server(
            &host,
            port,
            docroot.as_deref(),
            workers,
            tier0,
        ));
    }

    #[cfg(not(feature = "axum-server"))]
    if use_axum {
        eprintln!("php-server: --axum requires 'axum-server' feature");
        return ExitCode::from(1);
    }

    // Default: CLI server mode (M7: no CLI breakage)
    #[cfg(feature = "cli-server")]
    {
        // Pool size and Tier-0 are axum-only concepts; consumed here so the
        // default (cli-only) build stays clean under -D warnings (A-AH2 triple).
        let _ = (workers, tier0);
        let mut rest: Vec<OsString> = Vec::new();
        if let Some(d) = docroot {
            rest.push("-t".into());
            rest.push(d);
        }
        if let Some(r) = router {
            rest.push(r);
        }
        let addr = format!("{host}:{port}");
        return ExitCode::from(php_cli::server::serve(&addr, rest.into_iter().peekable()));
    }

    #[cfg(not(feature = "cli-server"))]
    {
        eprintln!("php-server: no server mode enabled (build with --features cli-server or axum-server)");
        ExitCode::from(1)
    }
}
