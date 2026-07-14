//! `phpr` — a minimal PHP CLI SAPI over the experiment's evaluator.
//!
//! Reads a script, lowers + runs it against the builtin registry, and writes the
//! CLI-faithful stream (`Outcome::rendered`: program output with diagnostics and
//! any uncaught fatal rendered inline, as PHP's CLI emits them under
//! `display_errors=1, html_errors=0`). The process exit status mirrors PHP: the
//! `exit`/`die` code when present, `255` for an uncaught fatal, otherwise `0`.

use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::ExitCode;

use php_builtins::registry;
use php_runtime::run_source_with_argv;

mod mime;
mod server;

/// The evaluator's per-request workload is allocation-bound (Zval/PhpArray
/// churn); mimalloc's sharded free lists stand in for Zend's bin/chunk ZMM.
#[global_allocator]
static GLOBAL_ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Best-effort human text from a caught panic payload (the common `&str` /
/// `String` cases; anything else is reported opaquely).
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

fn main() -> ExitCode {
    php_runtime::logging::init();
    // Leading `php`-style options before the script path. `-d key[=value]`
    // (separate or attached form) collects ini overrides — PHPUnit's
    // process-isolation runner spawns `PHP_BINARY -d k=v … <file>`.
    let mut ini_overrides: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut raw = std::env::args_os().skip(1).peekable();
    let mut path = None;
    while let Some(arg) = raw.next() {
        let bytes = arg.as_os_str().as_bytes();
        if bytes == b"-d" {
            if let Some(kv) = raw.next() {
                let kv = kv.as_os_str().as_bytes();
                let (k, v) = match kv.iter().position(|b| *b == b'=') {
                    Some(p) => (&kv[..p], &kv[p + 1..]),
                    None => (kv, &b"1"[..]),
                };
                ini_overrides.push((k.to_vec(), v.to_vec()));
            }
            continue;
        }
        if let Some(kv) = bytes.strip_prefix(b"-d") {
            if !kv.is_empty() {
                let (k, v) = match kv.iter().position(|b| *b == b'=') {
                    Some(p) => (&kv[..p], &kv[p + 1..]),
                    None => (kv, &b"1"[..]),
                };
                ini_overrides.push((k.to_vec(), v.to_vec()));
            }
            continue;
        }
        // `-n` (skip php.ini): phpr never loads one — accepted and ignored.
        if bytes == b"-n" {
            continue;
        }
        // `-S host:port [-t docroot] [router.php]`: the built-in web server.
        if bytes == b"-S" {
            let Some(addr) = raw.next() else {
                eprintln!("usage: phpr -S <addr>:<port> [-t docroot] [router.php]");
                return ExitCode::from(1);
            };
            return ExitCode::from(server::serve(&addr.to_string_lossy(), raw));
        }
        // `-f script`: explicit script-file form.
        if bytes == b"-f" {
            path = raw.next();
            break;
        }
        // `-r code`: run the code string (implicit `<?php ` prefix).
        if bytes == b"-r" {
            let Some(code) = raw.next() else {
                eprintln!("usage: phpr -r <code>");
                return ExitCode::from(1);
            };
            let mut source = b"<?php ".to_vec();
            source.extend_from_slice(code.as_os_str().as_bytes());
            let mut argv_owned: Vec<Vec<u8>> = vec![b"Standard input code".to_vec()];
            argv_owned.extend(raw.map(|a| a.as_os_str().as_bytes().to_vec()));
            return run(b"Standard input code", &source, argv_owned, ini_overrides);
        }
        path = Some(arg);
        break;
    }
    let Some(path) = path else {
        // No script argument: PHP's CLI reads the program from stdin —
        // PHPUnit's process-isolation runner pipes each test job this way.
        let mut source = Vec::new();
        use std::io::Read as _;
        if std::io::stdin().read_to_end(&mut source).is_err() || source.is_empty() {
            eprintln!("usage: phpr [-d key=value]... <script.php>");
            return ExitCode::from(1);
        }
        if source.starts_with(b"#!") {
            let end = source.iter().position(|&b| b == b'\n').map_or(source.len(), |p| p + 1);
            source.drain(..end);
        }
        let argv_owned: Vec<Vec<u8>> = vec![b"-".to_vec()];
        return run(b"Standard input code", &source, argv_owned, ini_overrides);
    };
    // `$argv` starts at the script path — the consumed options do not appear.
    let mut argv_owned: Vec<Vec<u8>> = vec![path.as_os_str().as_bytes().to_vec()];
    argv_owned.extend(raw.map(|a| a.as_os_str().as_bytes().to_vec()));

    let mut source = match std::fs::read(&path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Could not open input file: {}", path.to_string_lossy());
            return ExitCode::from(1);
        }
    };
    // PHP's CLI SAPI skips a `#!` shebang line of the *entry* script only
    // (cli_seek_file_begin) — it is neither output nor a statement, so
    // `namespace` right after it stays "the very first statement"
    // (Composer's vendor/bin proxies rely on this). Known skew: PHP starts
    // the lexer at line 2, so diagnostics in shebang scripts report one line
    // lower here.
    if source.starts_with(b"#!") {
        let end = source.iter().position(|&b| b == b'\n').map_or(source.len(), |p| p + 1);
        source.drain(..end);
    }

    // PHP resolves the script's `__FILE__` (and getFile()/trace paths) to its
    // realpath — an absolute, symlink-resolved path — so canonicalize the invoked
    // path, falling back to it verbatim if that fails.
    let canonical = std::fs::canonicalize(&path).unwrap_or_else(|_| std::path::PathBuf::from(&path));
    let name = canonical.to_string_lossy().into_owned();
    run(name.as_bytes(), &source, argv_owned, ini_overrides)
}

/// Lower + run the program and translate the outcome into PHP's CLI exit
/// status, behind a panic boundary: a bug in the runtime (a reachable
/// `expect`, a broken VM invariant) must not abort the host with a raw Rust
/// panic — it exits 255 with a labelled stderr line, like the isolated phpt
/// worker does per test. AssertUnwindSafe is sound here because on panic we
/// discard the runtime state and exit rather than reuse it.
fn run(
    name: &[u8],
    source: &[u8],
    argv_owned: Vec<Vec<u8>>,
    ini_overrides: Vec<(Vec<u8>, Vec<u8>)>,
) -> ExitCode {
    let registry = registry();
    // PHP CLI `$argv` / `$_SERVER['argv']`: element 0 is the script path, the rest
    // are the arguments after it (`phpr script.php a b` → ['script.php','a','b']).
    let argv_refs: Vec<&[u8]> = argv_owned.iter().map(|v| v.as_slice()).collect();
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_source_with_argv(name, source, &registry, &argv_refs, &ini_overrides)
    }));
    match result {
        Ok(Ok(outcome)) => {
            let mut stdout = std::io::stdout().lock();
            let _ = stdout.write_all(&outcome.rendered);
            let _ = stdout.flush();
            match outcome.exit_code {
                Some(code) => ExitCode::from(code),
                None if outcome.fatal.is_some() => ExitCode::from(255),
                None => ExitCode::SUCCESS,
            }
        }
        Ok(Err(e)) => {
            // A non-`Fatal` lowering error (e.g. a hard parse failure) — PHP would
            // print a Parse error and exit 255. (Unresolved supertypes no longer
            // land here: they defer to run time — Zend late binding — and render
            // as the faithful uncaught `Error`.)
            eprintln!("PHP Parse error: {e}");
            ExitCode::from(255)
        }
        Err(panic) => {
            // The default panic hook has already printed the payload + location to
            // stderr (honouring RUST_BACKTRACE); add a labelled line, mirror it to
            // the log when enabled, and exit 255 instead of aborting.
            let msg = panic_message(panic.as_ref());
            log::error!(target: "phpr::vm", "runtime panicked: {msg}");
            eprintln!("[phpr] internal error: the runtime panicked: {msg}");
            ExitCode::from(255)
        }
    }
}
