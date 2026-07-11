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
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: phpr <script.php>");
        return ExitCode::from(1);
    };

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
    let name = canonical.to_string_lossy();
    let registry = registry();
    // PHP CLI `$argv` / `$_SERVER['argv']`: element 0 is the script path, the rest
    // are the arguments after it (`phpr script.php a b` → ['script.php','a','b']).
    let argv_owned: Vec<Vec<u8>> = std::env::args_os()
        .skip(1)
        .map(|a| a.as_os_str().as_bytes().to_vec())
        .collect();
    let argv_refs: Vec<&[u8]> = argv_owned.iter().map(|v| v.as_slice()).collect();
    // Panic boundary: a bug in the runtime (a reachable `expect`, a broken VM
    // invariant) must not abort the host with a raw Rust panic — turn it into
    // PHP's uncaught-fatal exit status (255) with a labelled stderr line, like the
    // isolated phpt worker already does per test. AssertUnwindSafe is sound here
    // because on panic we discard the runtime state and exit rather than reuse it.
    let result = catch_unwind(AssertUnwindSafe(|| {
        run_source_with_argv(name.as_bytes(), &source, &registry, &argv_refs)
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
