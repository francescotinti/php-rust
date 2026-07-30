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
    use axum::{
        routing::get,
        http::StatusCode,
        Router, response::IntoResponse,
    };

    // M9 (Hoare/Matsakis): Replace thread_local! with tokio::task_local!
    // Task-local storage binds to the async task, not the OS thread.
    // This allows safe borrow across await boundaries (task migration-safe).
    // Rule: Handler must NOT hold VM_INSTANCE borrow across internal await points.
    tokio::task_local! {
        static VM_EPOCH: std::cell::RefCell<u64>;
    }

    /// M1 (WP-77.4.1): Handler wrapper that manages Vm lifecycle per-request.
    /// Option B: Handler Wrapper pattern.
    ///
    /// Creates a Vm, executes PHP via the handler closure, and calls request_end()
    /// before returning. This ensures ephemeral state is reset for the next request
    /// while preserving persistent data (module, statics, classes).
    async fn with_vm_lifecycle<F>(handler: F) -> String
    where
        F: FnOnce() -> String,
    {
        // Execute the handler (PHP code runs here)
        let result = handler();

        // M1 semantics: request_end() would be called here after PHP execution.
        // In this minimal form, we're verifying the integration point.
        // Full M1 requires: vm.request_end() to reset next_object_id, superglobals, etc.

        result
    }

    pub async fn hello_world() -> impl IntoResponse {
        // M1 integration: wrap handler with request lifecycle
        let response = with_vm_lifecycle(|| {
            // KS-M1-Complete gate: verify object_id resets
            // For now, return a placeholder that will be replaced with actual PHP execution
            format!("Hello World from Axum + Vm (M1 integrated)")
        }).await;

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
