//! `phpr` — a minimal PHP CLI SAPI over the experiment's evaluator.
//!
//! Reads a script, lowers + runs it against the builtin registry, and writes the
//! CLI-faithful stream (`Outcome::rendered`: program output with diagnostics and
//! any uncaught fatal rendered inline, as PHP's CLI emits them under
//! `display_errors=1, html_errors=0`). The process exit status mirrors PHP: the
//! `exit`/`die` code when present, `255` for an uncaught fatal, otherwise `0`.

use std::io::Write;
use std::process::ExitCode;

use php_builtins::registry;
use php_runtime::run_source_with;

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: phpr <script.php>");
        return ExitCode::from(1);
    };

    let source = match std::fs::read(&path) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Could not open input file: {}", path.to_string_lossy());
            return ExitCode::from(1);
        }
    };

    let name = path.to_string_lossy();
    let registry = registry();
    match run_source_with(name.as_bytes(), &source, &registry) {
        Ok(outcome) => {
            let mut stdout = std::io::stdout().lock();
            let _ = stdout.write_all(&outcome.rendered);
            let _ = stdout.flush();
            match outcome.exit_code {
                Some(code) => ExitCode::from(code),
                None if outcome.fatal.is_some() => ExitCode::from(255),
                None => ExitCode::SUCCESS,
            }
        }
        Err(e) => {
            // A non-`Fatal` lowering error (e.g. a hard parse failure) — PHP would
            // print a Parse error and exit 255.
            eprintln!("PHP Parse error: {e:?}");
            ExitCode::from(255)
        }
    }
}
