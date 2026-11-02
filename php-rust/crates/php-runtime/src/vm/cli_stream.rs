//! CLI write-through of the program output (GitHub issue #1, S-187 2026-11-02).
//!
//! The VM accumulates program output in `Vm::rendered` (with diagnostics and the
//! fatal banner inline) and the consumers read that buffer when the run is over:
//! the server takes it as the HTTP body, phpt-runner and the in-process tests
//! compare it. For a one-shot `phpr script.php` that meant nothing reached the
//! terminal before the script ended, `flush()` could not help and a killed
//! process lost everything. PHP's CLI SAPI does the opposite: `sapi_cli_ub_write`
//! hands every chunk to `write(2)` at once (no output buffering, implicit flush).
//!
//! `phpr` sets [`STREAM_STDOUT`] for the script run (next to `FAST_SHUTDOWN`);
//! every top-level write of the VM then also goes straight to the process stdout
//! through [`drain_if_on`], tracked by a sent-cursor over the append-only
//! `rendered`. Everyone else leaves the flag false and sees no change. The
//! `rendered` bytes appended outside `write_output` (late diagnostics, fatal
//! banners at shutdown) are written by the CLI after the run: `rendered[sent()..]`.
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Set ONLY by `phpr` for a one-shot script process. Never set by the server
/// (`phpr -S`, php-server), phpt-runner or the tests.
pub static STREAM_STDOUT: AtomicBool = AtomicBool::new(false);
/// Bytes of `Vm::rendered` already written to stdout (one run per process).
static SENT: AtomicUsize = AtomicUsize::new(0);

#[inline]
pub fn enabled() -> bool {
    STREAM_STDOUT.load(Ordering::Relaxed)
}

/// Write `rendered[sent..]` to stdout and flush it — one `write(2)` per chunk,
/// like php-cli. `rendered` is append-only during a run, so the cursor only grows.
pub fn drain(rendered: &[u8]) {
    let sent = SENT.load(Ordering::Relaxed);
    if rendered.len() <= sent {
        return;
    }
    let out = std::io::stdout();
    let mut lock = out.lock();
    let _ = lock.write_all(&rendered[sent..]);
    let _ = lock.flush();
    SENT.store(rendered.len(), Ordering::Relaxed);
}

/// The hook used by the VM write sites: a relaxed load when the flag is off.
#[inline]
pub fn drain_if_on(rendered: &[u8]) {
    if enabled() {
        drain(rendered);
    }
}

/// How much of `rendered` has already been written: the CLI writes the rest at exit.
pub fn sent() -> usize {
    SENT.load(Ordering::Relaxed)
}
