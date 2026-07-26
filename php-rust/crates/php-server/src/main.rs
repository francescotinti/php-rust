//! `php-server` — the standalone HTTP front end of the phpr engine.
//!
//! A thin configurable launcher over the same oracle-pinned cli-server SAPI
//! `phpr -S` uses (`php_cli::server`): sequential request handling, POST
//! bodies and cookies, multipart uploads, `$_SERVER`/`$_GET`/`$_POST`, static
//! files with the cli-server mime map, and the docroot `index.php` PATH_INFO
//! fallback that serves WordPress virtual routes (pretty permalinks,
//! `/wp-json/`) without a router script.
//!
//!     php-server [--host HOST] [--port PORT] [--docroot DIR] [ROUTER.php]
//!
//! Defaults: 127.0.0.1:8080, docroot = current directory.

use std::ffi::OsString;
use std::process::ExitCode;

/// Same allocator choice as `phpr` (main.rs): the per-request workload is
/// allocation-bound, mimalloc stands in for Zend's bin/chunk ZMM.
#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

const USAGE: &str = "usage: php-server [--host HOST] [--port PORT] [--docroot|-t DIR] [ROUTER.php]\n\
       defaults: --host 127.0.0.1 --port 8080 --docroot .";

fn missing(flag: &str) -> ExitCode {
    eprintln!("php-server: {flag} requires a value\n{USAGE}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    php_runtime::logging::init();
    let mut host = String::from("127.0.0.1");
    let mut port: u16 = 8080;
    let mut docroot: Option<OsString> = None;
    let mut router: Option<OsString> = None;
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
    // Hand the residual options to the SAPI in `phpr -S` form.
    let mut rest: Vec<OsString> = Vec::new();
    if let Some(d) = docroot {
        rest.push("-t".into());
        rest.push(d);
    }
    if let Some(r) = router {
        rest.push(r);
    }
    let addr = format!("{host}:{port}");
    ExitCode::from(php_cli::server::serve(&addr, rest.into_iter().peekable()))
}
