//! ext/intl's IDNA pair `idn_to_ascii` / `idn_to_utf8`, backed by a native
//! RFC 3492 punycode codec (no ICU). Scope: the UTS46 mapping step is
//! approximated with plain lowercasing (ASCII fast path; `char::to_lowercase`
//! for the rest), which matches ICU for the WordPress corpus (WP-17,
//! WP_Email_Address round-trips `xn--…` labels through both directions). The
//! `$idna_info` by-ref out parameter is not populated (registry builtins are
//! by-value); WordPress never passes it.

use php_runtime::Ctx;
use php_types::{PhpError, PhpStr, Zval};

const BASE: u32 = 36;
const TMIN: u32 = 1;
const TMAX: u32 = 26;
const SKEW: u32 = 38;
const DAMP: u32 = 700;
const INITIAL_BIAS: u32 = 72;
const INITIAL_N: u32 = 128;

/// RFC 3492 §6.1 bias adaptation.
fn adapt(mut delta: u32, numpoints: u32, firsttime: bool) -> u32 {
    delta = if firsttime { delta / DAMP } else { delta / 2 };
    delta += delta / numpoints;
    let mut k = 0;
    while delta > ((BASE - TMIN) * TMAX) / 2 {
        delta /= BASE - TMIN;
        k += BASE;
    }
    k + ((BASE - TMIN + 1) * delta) / (delta + SKEW)
}

/// Decode one punycode label (WITHOUT the "xn--" prefix) to code points.
fn punycode_decode(input: &[u8]) -> Option<String> {
    let mut output: Vec<char> = Vec::new();
    let rest = match input.iter().rposition(|&b| b == b'-') {
        Some(pos) => {
            for &b in &input[..pos] {
                if b >= 0x80 {
                    return None;
                }
                output.push(b as char);
            }
            &input[pos + 1..]
        }
        None => input,
    };
    if rest.is_empty() && input.iter().any(|&b| b == b'-') && output.is_empty() {
        // "xn---" style degenerate; fall through (loop no-ops).
    }
    let digit_of = |b: u8| -> Option<u32> {
        match b {
            b'a'..=b'z' => Some((b - b'a') as u32),
            b'A'..=b'Z' => Some((b - b'A') as u32),
            b'0'..=b'9' => Some((b - b'0') as u32 + 26),
            _ => None,
        }
    };
    let mut n: u32 = INITIAL_N;
    let mut i: u32 = 0;
    let mut bias: u32 = INITIAL_BIAS;
    let mut pos = 0;
    while pos < rest.len() {
        let oldi = i;
        let mut w: u32 = 1;
        let mut k = BASE;
        loop {
            let b = *rest.get(pos)?;
            pos += 1;
            let digit = digit_of(b)?;
            i = i.checked_add(digit.checked_mul(w)?)?;
            let t = if k <= bias {
                TMIN
            } else if k >= bias + TMAX {
                TMAX
            } else {
                k - bias
            };
            if digit < t {
                break;
            }
            w = w.checked_mul(BASE - t)?;
            k += BASE;
        }
        let len1 = output.len() as u32 + 1;
        bias = adapt(i - oldi, len1, oldi == 0);
        n = n.checked_add(i / len1)?;
        i %= len1;
        let c = char::from_u32(n)?;
        if (n as usize) < 0x80 && rest.len() == input.len() {
            // Basic code point encoded in the extended part with no delimiter
            // present is fine per RFC; keep it.
        }
        output.insert(i as usize, c);
        i += 1;
    }
    Some(output.into_iter().collect())
}

/// Encode one unicode label to punycode (WITHOUT the "xn--" prefix).
fn punycode_encode(input: &str) -> Option<Vec<u8>> {
    let cps: Vec<u32> = input.chars().map(|c| c as u32).collect();
    let mut output: Vec<u8> = cps
        .iter()
        .filter(|&&c| c < 0x80)
        .map(|&c| c as u8)
        .collect();
    let b = output.len() as u32;
    let mut h = b;
    if b > 0 {
        output.push(b'-');
    }
    let digit_char = |d: u32| -> u8 {
        if d < 26 {
            b'a' + d as u8
        } else {
            b'0' + (d - 26) as u8
        }
    };
    let mut n: u32 = INITIAL_N;
    let mut delta: u32 = 0;
    let mut bias: u32 = INITIAL_BIAS;
    let total = cps.len() as u32;
    while h < total {
        let m = *cps.iter().filter(|&&c| c >= n).min()?;
        delta = delta.checked_add((m - n).checked_mul(h + 1)?)?;
        n = m;
        for &c in &cps {
            if c < n {
                delta = delta.checked_add(1)?;
            }
            if c == n {
                let mut q = delta;
                let mut k = BASE;
                loop {
                    let t = if k <= bias {
                        TMIN
                    } else if k >= bias + TMAX {
                        TMAX
                    } else {
                        k - bias
                    };
                    if q < t {
                        break;
                    }
                    output.push(digit_char(t + (q - t) % (BASE - t)));
                    q = (q - t) / (BASE - t);
                    k += BASE;
                }
                output.push(digit_char(q));
                bias = adapt(delta, h + 1, h == b);
                delta = 0;
                h += 1;
            }
        }
        delta = delta.checked_add(1)?;
        n = n.checked_add(1)?;
    }
    Some(output)
}

/// The shared per-domain walk: lowercase, split on `.`, transform each label.
fn map_labels(domain: &str, f: impl Fn(&str) -> Option<String>) -> Option<String> {
    if domain.is_empty() {
        return None;
    }
    let mut out: Vec<String> = Vec::new();
    for label in domain.split('.') {
        let lowered: String = if label.is_ascii() {
            label.to_ascii_lowercase()
        } else {
            label.chars().flat_map(|c| c.to_lowercase()).collect()
        };
        out.push(f(&lowered)?);
    }
    Some(out.join("."))
}

/// `idn_to_ascii($domain, $flags = IDNA_DEFAULT, $variant = INTL_IDNA_VARIANT_UTS46)`.
pub fn idn_to_ascii(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let domain = ctx.to_zstr(argv.first().unwrap_or(&Zval::Null));
    let Ok(domain) = std::str::from_utf8(domain.as_bytes()) else {
        return Ok(Zval::Bool(false));
    };
    let mapped = map_labels(domain, |label| {
        if label.is_ascii() {
            if label.len() > 63 {
                return None;
            }
            return Some(label.to_string());
        }
        let enc = punycode_encode(label)?;
        if enc.len() + 4 > 63 {
            return None;
        }
        let mut s = String::from("xn--");
        s.push_str(std::str::from_utf8(&enc).ok()?);
        Some(s)
    });
    Ok(match mapped {
        Some(s) => Zval::Str(PhpStr::new(s.into_bytes())),
        None => Zval::Bool(false),
    })
}

/// `idn_to_utf8($domain, $flags = IDNA_DEFAULT, $variant = INTL_IDNA_VARIANT_UTS46)`.
pub fn idn_to_utf8(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let domain = ctx.to_zstr(argv.first().unwrap_or(&Zval::Null));
    let Ok(domain) = std::str::from_utf8(domain.as_bytes()) else {
        return Ok(Zval::Bool(false));
    };
    let mapped = map_labels(domain, |label| {
        if label.len() > 63 {
            return None;
        }
        if let Some(rest) = label.strip_prefix("xn--") {
            let decoded = punycode_decode(rest.as_bytes())?;
            if decoded.is_empty() {
                return None;
            }
            return Some(decoded);
        }
        Some(label.to_string())
    });
    Ok(match mapped {
        Some(s) => Zval::Str(PhpStr::new(s.into_bytes())),
        None => Zval::Bool(false),
    })
}

/// ext/intl's `normalizer_normalize($string, $form = Normalizer::FORM_C)`,
/// on the pure-Rust `unicode-normalization` tables. Forms use ICU's values
/// (FORM_D=4, FORM_KD=8, FORM_C=16, FORM_KC=32); an invalid form is a
/// ValueError, a non-UTF-8 input yields `false` (as ext/intl). The prelude
/// `Normalizer` class delegates here (WP's `remove_accents` NFD path, WP-18).
pub fn normalizer_normalize(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use unicode_normalization::UnicodeNormalization;
    let s = ctx.to_zstr(argv.first().unwrap_or(&Zval::Null));
    let form = argv
        .get(1)
        .map(|v| crate::convert::to_long_cast(v, ctx.diags))
        .unwrap_or(16);
    let Ok(txt) = std::str::from_utf8(s.as_bytes()) else {
        return Ok(Zval::Bool(false));
    };
    let out: String = match form {
        4 => txt.nfd().collect(),
        8 => txt.nfkd().collect(),
        16 => txt.nfc().collect(),
        32 => txt.nfkc().collect(),
        _ => {
            return Err(PhpError::ValueError(
                "normalizer_normalize(): Argument #2 ($form) must be a a valid normalization form"
                    .to_string(),
            ))
        }
    };
    Ok(Zval::Str(PhpStr::new(out.into_bytes())))
}

/// ext/intl's `normalizer_is_normalized($string, $form = Normalizer::FORM_C)`.
pub fn normalizer_is_normalized(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match normalizer_normalize(argv, ctx)? {
        Zval::Str(out) => {
            let s = ctx.to_zstr(argv.first().unwrap_or(&Zval::Null));
            Ok(Zval::Bool(out.as_bytes() == s.as_bytes()))
        }
        other => Ok(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punycode_roundtrip_known_labels() {
        // grå ↔ gr-zia (WP's latin punycode domain).
        assert_eq!(punycode_decode(b"gr-zia").as_deref(), Some("gr\u{e5}"));
        assert_eq!(punycode_encode("gr\u{e5}").as_deref(), Some(&b"gr-zia"[..]));
        // 慕田峪长城 ↔ uist2j67d64zv30b (WP's han punycode domain).
        assert_eq!(
            punycode_decode(b"uist2j67d64zv30b").as_deref(),
            Some("\u{6155}\u{7530}\u{5cea}\u{957f}\u{57ce}")
        );
        assert_eq!(
            punycode_encode("\u{6155}\u{7530}\u{5cea}\u{957f}\u{57ce}").as_deref(),
            Some(&b"uist2j67d64zv30b"[..])
        );
        // 网址 ↔ ses554g.
        assert_eq!(punycode_decode(b"ses554g").as_deref(), Some("\u{7f51}\u{5740}"));
    }
}
