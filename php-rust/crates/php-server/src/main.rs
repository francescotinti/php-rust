//! `php-server` — HTTP front end for phpr engine.
//!
//! Two modes:
//! 1. `--cli-server` (default): Reuses the CLI SAPI from `phpr -S`
//! 2. `--axum`: Async Axum handler + thread-local Vm (N=1 reuse pattern, WP-77)
//!
//! Thread-local Vm (N=1):
//! - One Vm instance per handler thread (created at thread startup)
//! - Each request borrows from thread-local, calls handler, releases
//! - Vm::reset() clears state between requests (M1 alloc parity target)
//! - Avoids Arc<Mutex> contention (domain-web: stateless HTTP)

use std::ffi::OsString;
use std::process::ExitCode;

#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "axum-server")]
mod worker_pool;

const USAGE: &str = "usage: php-server [--host HOST] [--port PORT] [--docroot|-t DIR] [--axum] [ROUTER.php]\n\
       defaults: --host 127.0.0.1 --port 8080 --docroot . --cli-server";

fn missing(flag: &str) -> ExitCode {
    eprintln!("php-server: {flag} requires a value\n{USAGE}");
    ExitCode::from(2)
}

#[cfg(feature = "axum-server")]
mod axum_handler {
    use std::process::ExitCode;
    use std::sync::Arc;
    use axum::{
        routing::{get, post},
        http::{StatusCode, Method, Uri},
        extract::State,
        Router, response::IntoResponse,
        body::Body,
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
        method: Method,
        uri: Uri,
        body: Body,
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

        // Read request body
        let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
            Ok(b) => b.to_vec(),
            Err(e) => {
                let msg = format!("Failed to read body: {}\n", e);
                return (StatusCode::BAD_REQUEST, msg.into_bytes());
            }
        };

        // Create oneshot channel for response
        let (tx, rx) = oneshot::channel();

        // Build handler metadata
        let meta = crate::worker_pool::WorkerHandlerMeta {
            path: request_path.clone(),
            method: method.to_string(),
            body: body_bytes,
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

    pub async fn start_server(host: &str, port: u16, docroot: Option<&std::ffi::OsStr>) -> ExitCode {
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

        // Initialize worker pool
        let pool_context = crate::worker_pool::WorkerPoolContext::new();
        let pool = crate::worker_pool::WorkerPool::new(pool_context, 0); // 0 = CPU count

        let app_state = Arc::new(AppState {
            pool,
            docroot: docroot_str.clone(),
        });

        // Router: use fallback to catch-all paths
        let app = Router::new()
            .fallback(php_handler)
            .with_state(app_state);

        eprintln!("php-server (Axum) listening on http://{addr} (docroot: {docroot_str})");

        if let Err(e) = axum::serve(listener, app).await {
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
            "--axum" => {
                #[cfg(feature = "axum-server")]
                {
                    use_axum = true;
                }
                #[cfg(not(feature = "axum-server"))]
                {
                    eprintln!("php-server: --axum requires 'axum-server' feature");
                    return ExitCode::from(1);
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
        return rt.block_on(axum_handler::start_server(&host, port, docroot.as_deref()));
    }

    #[cfg(not(feature = "axum-server"))]
    if use_axum {
        eprintln!("php-server: --axum requires 'axum-server' feature");
        return ExitCode::from(1);
    }

    // Default: CLI server mode (M7: no CLI breakage)
    #[cfg(feature = "cli-server")]
    {
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
