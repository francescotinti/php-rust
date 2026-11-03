//! §3.39 (S-189): the `http://` stream wrapper's read/connect timeout.
//!
//! PHP applies the stream-context option `['http' => ['timeout' => <float s>]]`
//! or, in its absence, the ini `default_socket_timeout` (60 s) to connect AND
//! read (`ext/standard/http_fopen_wrapper.c`, `PHP_STREAM_OPTION_READ_TIMEOUT`):
//! a server that accepts and stays silent fails `file_get_contents` after the
//! timeout with the Warning "Failed to open stream: HTTP request failed!".
//! phpr used to wait forever (oracle probe 2026-11-03: context 1.0 → false at
//! 1004 ms; ini 2 → 2000 ms; phpr hung), which under `php-server --axum`
//! parks a worker for good. This module is the pure half of the cure; the ureq
//! agent in `file::http_get` applies it as `timeout_global`.
//!
//! Declared residual (`TODO(port)`, same posture as `fsockopen`/
//! `stream_socket_client` in php-runtime `vm/host`): the INI table lives in the
//! Vm and is not reachable from the builtins crate, so `ini_set(
//! 'default_socket_timeout', n)` is not honoured yet — the default is the
//! compiled-in 60 s of `vm/ini`.

use std::time::Duration;

use php_types::Zval;

/// PHP's compiled-in `default_socket_timeout` (seconds).
pub(crate) const DEFAULT_SOCKET_TIMEOUT: f64 = 60.0;

/// A context option as a float: Long, Double, numeric string (PHP's double
/// cast of the option), through references. Anything else → None (option
/// ignored, as PHP's `zend_parse_arg_double` path on junk).
pub(crate) fn ctx_f64(v: &Zval) -> Option<f64> {
    match v {
        Zval::Long(n) => Some(*n as f64),
        Zval::Double(d) => Some(*d),
        Zval::Str(s) => std::str::from_utf8(s.as_bytes()).ok()?.trim().parse::<f64>().ok(),
        Zval::Ref(r) => ctx_f64(&r.borrow()),
        _ => None,
    }
}

/// Effective wrapper timeout: the context `timeout` when given, else the
/// default; a non-positive value means "no timeout" (PHP: `default_socket_timeout`
/// = -1 disables it). NaN counts as absent.
pub(crate) fn http_wrapper_timeout(ctx_timeout: Option<f64>) -> Option<Duration> {
    let t = match ctx_timeout {
        Some(t) if !t.is_nan() => t,
        _ => DEFAULT_SOCKET_TIMEOUT,
    };
    if t > 0.0 { Some(Duration::from_secs_f64(t)) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::PhpStr;

    #[test]
    fn context_timeout_forms() {
        assert_eq!(ctx_f64(&Zval::Long(3)), Some(3.0));
        assert_eq!(ctx_f64(&Zval::Double(1.5)), Some(1.5));
        assert_eq!(ctx_f64(&Zval::Str(PhpStr::new(b" 2.5 ".to_vec()))), Some(2.5));
        assert_eq!(ctx_f64(&Zval::Str(PhpStr::new(b"abc".to_vec()))), None);
        assert_eq!(ctx_f64(&Zval::Bool(true)), None);
    }

    #[test]
    fn default_is_sixty_seconds_and_nonpositive_disables() {
        assert_eq!(http_wrapper_timeout(None), Some(Duration::from_secs(60)));
        assert_eq!(http_wrapper_timeout(Some(f64::NAN)), Some(Duration::from_secs(60)));
        assert_eq!(http_wrapper_timeout(Some(1.0)), Some(Duration::from_secs(1)));
        assert_eq!(http_wrapper_timeout(Some(0.25)), Some(Duration::from_millis(250)));
        assert_eq!(http_wrapper_timeout(Some(0.0)), None);
        assert_eq!(http_wrapper_timeout(Some(-1.0)), None);
    }
}
