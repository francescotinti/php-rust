//! Environment / runtime-introspection stubs.
//!
//! These functions exist so that scripts which incidentally call them compile
//! and run rather than fatalling with "undefined function". They model no real
//! engine state — the garbage collector, the memory allocator and the INI table
//! are not simulated — but their return *shapes* match PHP 8.5 (`int` for the GC
//! / memory counters, `"cli"` for the SAPI name, `false` for an absent INI
//! entry), which is all the vast majority of callers observe.

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

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

/// `gc_status()` (PHP 8.3 shape): the full 12-key stats array with an idle
/// collector's values (threshold/buffer_size are PHP's compiled-in defaults).
/// Consumers (PHPUnit's telemetry) diff the counters, so constant zeros read
/// as "no collector activity", which is honest — phpr's cycle collector runs
/// its own accounting that these counters do not model.
pub fn gc_status(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::rc::Rc;
    let mut out = php_types::PhpArray::new();
    let mut put = |k: &str, v: Zval| {
        out.insert(php_types::Key::Str(PhpStr::from_str(k)), v);
    };
    put("running", Zval::Bool(false));
    put("protected", Zval::Bool(false));
    put("full", Zval::Bool(false));
    put("runs", Zval::Long(0));
    put("collected", Zval::Long(0));
    put("threshold", Zval::Long(10001));
    put("buffer_size", Zval::Long(16384));
    put("roots", Zval::Long(0));
    put("application_time", Zval::Double(0.0));
    put("collector_time", Zval::Double(0.0));
    put("destructor_time", Zval::Double(0.0));
    put("free_time", Zval::Double(0.0));
    Ok(Zval::Array(Rc::new(out)))
}

/// `memory_get_usage()` / `memory_get_peak_usage()` — a plausible constant byte
/// count (no allocator is tracked).
pub fn memory_get_usage(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(2_000_000))
}

/// `memory_reset_peak_usage()` (PHP 8.2) — no-op: no allocator peak is tracked
/// (see `memory_get_usage`). Returns null (void), as PHP.
pub fn memory_reset_peak_usage(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Null)
}

/// `getmypid()` — the real process id (PHPUnit's ShutdownHandler keys on it).
pub fn getmypid(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(std::process::id() as i64))
}

/// `posix_getpid()` — ext/posix spelling of the same pid (`posix_kill` targets
/// it in the pcntl signal tests). ext/pcntl's signal delivery lives VM-side.
pub fn posix_getpid(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(std::process::id() as i64))
}

/// `php_sapi_name()` — `cli`, or the name the web host installed at startup
/// (`cli-server` under `phpr -S`).
pub fn php_sapi_name(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::from_str(php_types::sapi::sapi_name())))
}

/// `getallheaders()` / `apache_request_headers()` — the request headers in
/// wire order with their original case. Registered only under the web SAPI
/// (the CLI oracle has neither function).
pub fn getallheaders(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut a = php_types::PhpArray::new();
    if let Some(req) = php_types::sapi::web_request() {
        for (k, v) in &req.headers {
            a.insert(php_types::Key::from_bytes(k), Zval::Str(PhpStr::new(v.clone())));
        }
    }
    Ok(Zval::Array(std::rc::Rc::new(a)))
}

/// `posix_geteuid()` — the process's effective uid (libc, real value: DBAL's
/// ExceptionTest skips its chmod-based test when running as root).
pub fn posix_geteuid(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(i64::from(unsafe { libc::geteuid() })))
}

/// `php_ini_loaded_file()` — path of the primary loaded `php.ini`, or `false`
/// when none was loaded. phpr reads no INI file, so `false` (as a PHP build
/// started with `-n`). Composer's XdebugHandler tolerates this.
pub fn php_ini_loaded_file(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(false))
}

/// `php_ini_scanned_files()` — comma-separated list of INI files parsed from the
/// scan dir, or `false` when there is no scan dir / none were parsed. phpr scans
/// none, so `false`.
pub fn php_ini_scanned_files(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(false))
}

/// `phpinfo($flags = INFO_ALL)` — writes a report and returns `true`. phpr cannot
/// reproduce the full multi-section report (which is HTML/text bound to the C
/// SAPI internals), so it emits a minimal, honest *general* block. Notably it has
/// no "Configure Command" line — phpr is not an autoconf build — which is exactly
/// what a consumer parsing that field (Composer's DiagnoseCommand) should see.
pub fn phpinfo(_args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    ctx.out.extend_from_slice(
        b"phpinfo()\n\
          PHP Version => 8.5.7\n\
          \n\
          System => Darwin\n\
          Build Date => Jan  1 2026 00:00:00\n\
          PHP Version => 8.5.7\n\
          PHP SAPI => cli\n\
          Zend Engine => 4.5.7\n\
          Thread Safety => disabled\n\
          Debug Build => no\n\
          \n\
          xsl\n\
          \n\
          XSL => enabled\n\
          libxslt Version => 1.1.35\n\
          libxslt compiled against libxml Version => 2.9.13\n\
          EXSLT => enabled\n\
          libexslt Version => 0.8.20\n",
    );
    Ok(Zval::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>, args: &[Zval]) -> Zval {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut direct = Vec::new();
        let dbg = std::collections::HashMap::new();
        let strf = std::collections::HashMap::new();
        let mut ctx = Ctx { out: &mut out, diags: &mut diags, direct_out: &mut direct, debug_info: &dbg, stringify: &strf };
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
    }
}

/// `gethostname()`: the host's standard name, or `false` on failure.
pub fn gethostname(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match hostname::get() {
        Ok(h) => Ok(Zval::Str(PhpStr::new(h.to_string_lossy().into_owned().into_bytes()))),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

/// `sys_getloadavg()`: the 1/5/15-minute load averages, or `false` on failure.
pub fn sys_getloadavg(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut loads = [0f64; 3];
    let n = unsafe { libc::getloadavg(loads.as_mut_ptr(), 3) };
    if n < 3 {
        return Ok(Zval::Bool(false));
    }
    let mut arr = php_types::PhpArray::new();
    for l in loads {
        let _ = arr.append(Zval::Double(l));
    }
    Ok(Zval::Array(std::rc::Rc::new(arr)))
}

/// `usleep($microseconds)`: pause execution.
pub fn usleep(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let us = args.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if us > 0 {
        std::thread::sleep(std::time::Duration::from_micros(us as u64));
    }
    Ok(Zval::Null)
}

/// `sleep($seconds)`: pause execution; returns 0 (the POSIX remainder).
pub fn sleep(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = args.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if s > 0 {
        std::thread::sleep(std::time::Duration::from_secs(s as u64));
    }
    Ok(Zval::Long(0))
}

/// `php_uname($mode = "a")`: system information via `uname(2)` — `s`ysname,
/// `n`odename, `r`elease, `v`ersion, `m`achine, or `a`ll (space-joined).
pub fn php_uname(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mode = match args.first() {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b"a".to_vec(),
    };
    let mut uts: libc::utsname = unsafe { std::mem::zeroed() };
    if unsafe { libc::uname(&mut uts) } != 0 {
        return Ok(Zval::Bool(false));
    }
    let field = |a: &[libc::c_char]| -> Vec<u8> {
        a.iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect()
    };
    let out = match mode.as_slice() {
        b"s" => field(&uts.sysname),
        b"n" => field(&uts.nodename),
        b"r" => field(&uts.release),
        b"v" => field(&uts.version),
        b"m" => field(&uts.machine),
        b"a" => {
            let mut v = field(&uts.sysname);
            for f in [&uts.nodename, &uts.release, &uts.version, &uts.machine] {
                v.push(b' ');
                v.extend_from_slice(&field(f));
            }
            v
        }
        _ => {
            return Err(PhpError::ValueError(
                "php_uname(): Argument #1 ($mode) must be one of \"a\", \"m\", \"n\", \"r\", \"s\", or \"v\""
                    .to_string(),
            ))
        }
    };
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `uniqid($prefix = "", $more_entropy = false)`: hex of the current epoch
/// seconds + microseconds (13 chars), PHP's exact format; `$more_entropy`
/// appends `.` + a 9-digit fractional, like PHP's combined LCG tail.
pub fn uniqid(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let prefix = args
        .first()
        .map(|v| convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
        .unwrap_or_default();
    let more = args.get(1).map(|v| matches!(v, Zval::Bool(true))).unwrap_or(false);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let mut out = prefix;
    out.extend_from_slice(format!("{:08x}{:05x}", now.as_secs(), now.subsec_micros()).as_bytes());
    if more {
        let mut b = [0u8; 4];
        let _ = getrandom::getrandom(&mut b);
        let frac = (u32::from_le_bytes(b) as f64 / u32::MAX as f64) * 10.0;
        out.extend_from_slice(format!("{:.8}", frac).as_bytes());
    }
    Ok(Zval::Str(PhpStr::new(out)))
}
