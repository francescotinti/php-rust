//! Environment / runtime-introspection stubs.
//!
//! These functions exist so that scripts which incidentally call them compile
//! and run rather than fatalling with "undefined function". They model no real
//! engine state — the garbage collector, the memory allocator and the INI table
//! are not simulated — but their return *shapes* match PHP 8.5 (`int` for the GC
//! / memory counters, `"cli"` for the SAPI name, `false` for an absent INI
//! entry), which is all the vast majority of callers observe.

use php_runtime::Ctx;
use php_types::{PhpError, PhpStr, Zval};

/// `gc_collect_cycles()` — number of collected cycles. No GC is modelled, so 0.
pub fn gc_collect_cycles(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(0))
}

/// `gc_enable()` / `gc_disable()` — no-ops returning null.
pub fn gc_enable(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Null)
}

/// `gc_enabled()` — the cycle collector is reported as on (PHP's default).
pub fn gc_enabled(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(true))
}

/// `gc_mem_caches()` — bytes freed from the GC's caches. None modelled, so 0.
pub fn gc_mem_caches(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(0))
}

/// `memory_get_usage()` / `memory_get_peak_usage()` — a plausible constant byte
/// count (no allocator is tracked).
pub fn memory_get_usage(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(2_000_000))
}

/// `php_sapi_name()` — this engine runs as the command-line SAPI.
pub fn php_sapi_name(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::from_str("cli")))
}

/// `ini_get($name)` — no INI table is modelled, so every entry reads as absent
/// (`false`), matching PHP for an unknown directive.
pub fn ini_get(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(false))
}

/// `ini_set($name, $value)` — setting always "fails" (`false`); no INI table is
/// modelled. (PHP returns the previous value on success.)
pub fn ini_set(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(false))
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>, args: &[Zval]) -> Zval {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut ctx = Ctx { out: &mut out, diags: &mut diags };
        f(args, &mut ctx).unwrap()
    }

    #[test]
    fn stub_return_shapes_match_php() {
        assert!(matches!(call(gc_collect_cycles, &[]), Zval::Long(0)));
        assert!(matches!(call(gc_enable, &[]), Zval::Null));
        assert!(matches!(call(gc_enabled, &[]), Zval::Bool(true)));
        assert!(matches!(call(gc_mem_caches, &[]), Zval::Long(0)));
        assert!(matches!(call(memory_get_usage, &[]), Zval::Long(_)));
        match call(php_sapi_name, &[]) {
            Zval::Str(s) => assert_eq!(s.as_bytes(), b"cli"),
            other => panic!("expected string, got {other:?}"),
        }
        assert!(matches!(call(ini_get, &[Zval::Str(PhpStr::from_str("x"))]), Zval::Bool(false)));
        assert!(matches!(
            call(ini_set, &[Zval::Str(PhpStr::from_str("x")), Zval::Long(1)]),
            Zval::Bool(false)
        ));
    }
}
