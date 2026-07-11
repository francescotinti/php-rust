//! Network-address builtins: `ip2long`, `long2ip`, `inet_pton`, `inet_ntop`.
//!
//! IPv4 is parsed by hand (PHP accepts leading zeros — `192.168.01.1` — which
//! `std::net::Ipv4Addr::from_str` rejects); IPv6 goes through `std::net::Ipv6Addr`,
//! whose RFC 5952 canonical form matches PHP's `inet_ntop`.

use std::net::Ipv6Addr;
use std::str::FromStr;

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

/// Parse a dotted-quad IPv4 address into its four octets. PHP-lenient: exactly
/// four decimal groups (1–3 digits each, ≤ 255); leading zeros are allowed.
fn parse_ipv4(s: &str) -> Option<[u8; 4]> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut out = [0u8; 4];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() || p.len() > 3 || !p.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let v: u32 = p.parse().ok()?;
        if v > 255 {
            return None;
        }
        out[i] = v as u8;
    }
    Some(out)
}

/// `ip2long(string $ip): int|false` — an IPv4 dotted-quad to its 32-bit value.
pub fn ip2long(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let text = match std::str::from_utf8(s.as_bytes()) {
        Ok(t) => t,
        Err(_) => return Ok(Zval::Bool(false)),
    };
    match parse_ipv4(text) {
        Some(o) => Ok(Zval::Long(u32::from_be_bytes(o) as i64)),
        None => Ok(Zval::Bool(false)),
    }
}

/// `long2ip(int $ip): string` — a 32-bit value (masked to 32 bits) to a dotted
/// quad. Always succeeds in PHP 8.
pub fn long2ip(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let n = (v & 0xFFFF_FFFF) as u32;
    let b = n.to_be_bytes();
    Ok(Zval::Str(PhpStr::new(
        format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]).into_bytes(),
    )))
}

/// `inet_pton(string $ip): string|false` — a printable IPv4/IPv6 address to its
/// packed 4-/16-byte form.
pub fn inet_pton(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    // A NUL byte in the input is PHP's ValueError, not a plain false
    // (symfony's IpUtils::anonymize probes rely on it to detect a PACKED
    // binary address handed to the textual API).
    if s.as_bytes().contains(&0) {
        return Err(PhpError::ValueError(
            "inet_pton(): Argument #1 ($ip) must not contain any null bytes".to_string(),
        ));
    }
    let text = match std::str::from_utf8(s.as_bytes()) {
        Ok(t) => t,
        Err(_) => return Ok(Zval::Bool(false)),
    };
    if text.contains(':') {
        match Ipv6Addr::from_str(text) {
            Ok(a) => Ok(Zval::Str(PhpStr::new(a.octets().to_vec()))),
            Err(_) => Ok(Zval::Bool(false)),
        }
    } else {
        match parse_ipv4(text) {
            Some(o) => Ok(Zval::Str(PhpStr::new(o.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }
}

/// `inet_ntop(string $in_addr): string|false` — a packed 4-/16-byte address to
/// its printable form (IPv6 in RFC 5952 canonical form).
pub fn inet_ntop(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let b = s.as_bytes();
    match b.len() {
        4 => Ok(Zval::Str(PhpStr::new(
            format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]).into_bytes(),
        ))),
        16 => {
            let mut o = [0u8; 16];
            o.copy_from_slice(b);
            // The system inet_ntop renders a v4-COMPATIBLE address (first six
            // 16-bit words zero, seventh non-zero: `::123.234.0.0`) in dotted
            // form — oracle-pinned; `::1`/`::2` (seventh word zero too) stay
            // hex. Rust's Display only does this for the v4-MAPPED form.
            let words0_5_zero = o[..12] == [0u8; 12];
            if words0_5_zero && (o[12] != 0 || o[13] != 0) {
                return Ok(Zval::Str(PhpStr::new(
                    format!("::{}.{}.{}.{}", o[12], o[13], o[14], o[15]).into_bytes(),
                )));
            }
            Ok(Zval::Str(PhpStr::new(Ipv6Addr::from(o).to_string().into_bytes())))
        }
        _ => Ok(Zval::Bool(false)),
    }
}
