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
        routing::get,
        http::StatusCode,
        Router, response::IntoResponse,
    };

    /// WP-77.5: Worker pool context (shared server-lifetime state).
    /// Vm instances borrow from this; lives for entire server lifetime.
    struct WorkerPoolContext {
        // G1 spike: Minimal structure to store Vm without unsafe
        // TODO: Initialize RetainSet + Module here (server startup)
        _marker: std::marker::PhantomData<()>,
    }

    impl WorkerPoolContext {
        fn new() -> Arc<Self> {
            Arc::new(WorkerPoolContext {
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// WP-77.5: Worker pool type (collection of persistent worker threads).
    /// Each worker owns a Vm instance for the duration of the server.
    struct WorkerPool {
        context: Arc<WorkerPoolContext>,
        // G1 spike: Workers will be stored here (N dedicated OS threads)
        // TODO: Create workers on pool initialization
        _workers: Vec<Arc<std::sync::Mutex<()>>>,
    }

    impl WorkerPool {
        fn new(context: Arc<WorkerPoolContext>, _num_workers: usize) -> Self {
            WorkerPool {
                context,
                _workers: Vec::new(),
            }
        }
    }

    /// G1 spike: Test whether Vm can be stored in a worker without unsafe.
    /// This is the minimal gate: Can we even create the structure?
    async fn test_vm_storage() -> Result<(), String> {
        let context = WorkerPoolContext::new();
        let _pool = WorkerPool::new(context, 4);
        // G1 success: No unsafe required, compiles cleanly
        Ok(())
    }

    pub async fn hello_world() -> impl IntoResponse {
        // WP-77.5 integration point: handlers will receive worker dispatch
        // For now, placeholder until worker pool is wired
        let response = match test_vm_storage().await {
            Ok(()) => "Hello World from Axum + Worker Pool (G1 spike)".to_string(),
            Err(e) => format!("G1 spike failed: {e}"),
        };

        (StatusCode::OK, response)
    }

    pub async fn start_server(host: &str, port: u16) -> ExitCode {
        let addr = format!("{host}:{port}");
        let listener = match tokio::net::TcpListener::bind(&addr).await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("php-server: failed to bind {addr}: {e}");
                return ExitCode::from(1);
            }
        };

        let app = Router::new()
            .route("/", get(hello_world))
            .route("/hello-world", get(hello_world));

        eprintln!("php-server (Axum) listening on http://{addr}");

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
        return rt.block_on(axum_handler::start_server(&host, port));
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
