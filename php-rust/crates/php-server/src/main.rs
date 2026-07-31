//! `php-server` — HTTP front end for phpr engine.
//!
//! Two modes:
//! 1. `--cli-server` (default): Reuses the CLI SAPI from `phpr -S`
//! 2. `--axum`: Axum HTTP front end + worker-actor pool (WP-77.6):
//!    N OS worker threads, each owning a thread-persistent RetainSet (arena of
//!    compiled unit modules); a FRESH Vm is created per request and borrows the
//!    worker's RetainSet. There is NO `Vm::reset()` and NO N=1 thread-local Vm
//!    reuse — that architecture was REJECTED in WP-77.2 (A-TH3/A-BG2 purge).
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

#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "axum-server")]
mod worker_pool;

const USAGE: &str = "usage: php-server [--host HOST] [--port PORT] [--docroot|-t DIR] [--axum] [--workers N] [ROUTER.php]\n\
       defaults: --host 127.0.0.1 --port 8080 --docroot . --cli-server --workers 0 (= CPU count, axum mode only)";

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
        let meta = crate::worker_pool::WorkerHandlerMeta {
            path: request_path.clone(),
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
    ) -> ExitCode {
        let addr = format!("{host}:{port}");
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("php-server: failed to bind {addr}: {e}");
                return ExitCode::from(1);
            }
        };

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
            pool,
            docroot: docroot_str.clone(),
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
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    php_runtime::logging::init();

    let mut host = String::from("127.0.0.1");
    let mut port: u16 = 8080;
    let mut docroot: Option<OsString> = None;
    let mut router: Option<OsString> = None;
    let mut use_axum = false;
    // 0 = num_cpus (axum mode only; ignored by the cli-server path)
    let mut workers: usize = 0;

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
        // Pool size is an axum-only concept; consumed here so the default
        // (cli-only) build stays clean under -D warnings (A-AH2 triple).
        let _ = workers;
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
