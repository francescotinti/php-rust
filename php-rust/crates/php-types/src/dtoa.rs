//! double -> string formatting: faithful port of `zend_gcvt`
//! (Zend/zend_strtod.c) on top of Rust's exact float formatting.
//!
//! Two observable modes (diary/01-semantic-model.md §8):
//! - echo / string conversion: precision = 14 significant digits
//! - var_dump / serialize: serialize_precision = -1 -> shortest roundtrip,
//!   with the E-notation threshold computed as if ndigit were 17.

/// gcvt mode 2: `ndigit` significant digits (echo uses 14).
pub fn double_to_precision(value: f64, ndigit: u32) -> Vec<u8> {
    match special(value) {
        Some(s) => s,
        None => {
            let (digits, decpt) = digits_fixed(value.abs(), ndigit as usize);
            assemble(value.is_sign_negative(), &digits, decpt, ndigit as i32)
        }
    }
}

/// gcvt mode 0: shortest roundtrip digits, threshold as ndigit=17.
pub fn double_to_shortest(value: f64) -> Vec<u8> {
    match special(value) {
        Some(s) => s,
        None => {
            let (digits, decpt) = digits_shortest(value.abs());
            assemble(value.is_sign_negative(), &digits, decpt, 17)
        }
    }
}

fn special(value: f64) -> Option<Vec<u8>> {
    if value.is_nan() {
        Some(b"NAN".to_vec())
    } else if value.is_infinite() {
        Some(if value < 0.0 { b"-INF".to_vec() } else { b"INF".to_vec() })
    } else {
        None
    }
}

/// Shortest-roundtrip significant digits + decimal point position.
/// `decpt` follows dtoa convention: value = 0.digits * 10^decpt.
fn digits_shortest(v: f64) -> (Vec<u8>, i32) {
    debug_assert!(v >= 0.0 && v.is_finite());
    split_exp(&format!("{:e}", v))
}

/// `n` correctly-rounded significant digits, trailing zeros stripped
/// (dtoa mode 2 strips them too).
fn digits_fixed(v: f64, n: usize) -> (Vec<u8>, i32) {
    debug_assert!(v >= 0.0 && v.is_finite() && n >= 1);
    split_exp(&format!("{:.*e}", n - 1, v))
}

fn split_exp(s: &str) -> (Vec<u8>, i32) {
    let (mant, exp) = s.split_once('e').expect("exp format");
    let exp: i32 = exp.parse().expect("exp int");
    let mut digits: Vec<u8> = mant.bytes().filter(|b| *b != b'.').collect();
    while digits.len() > 1 && *digits.last().unwrap() == b'0' {
        digits.pop();
    }
    (digits, exp + 1)
}

/// Assembly logic ported line-by-line from zend_gcvt (Zend/zend_strtod.c):
/// E-style iff `(decpt >= 0 && decpt > ndigit) || decpt < -3`.
fn assemble(neg: bool, digits: &[u8], decpt: i32, ndigit: i32) -> Vec<u8> {
    let mut out = Vec::with_capacity(24);
    if neg {
        out.push(b'-');
    }
    if (decpt >= 0 && decpt > ndigit) || decpt < -3 {
        // Exponential: first digit, '.', rest (or '0'), 'E', sign, exponent.
        let exp10 = decpt - 1;
        out.push(digits[0]);
        out.push(b'.');
        if digits.len() > 1 {
            out.extend_from_slice(&digits[1..]);
        } else {
            out.push(b'0');
        }
        out.push(b'E');
        out.push(if exp10 < 0 { b'-' } else { b'+' });
        out.extend_from_slice(exp10.abs().to_string().as_bytes());
    } else if decpt < 0 {
        // 0.00ddd
        out.push(b'0');
        out.push(b'.');
        for _ in 0..(-decpt) {
            out.push(b'0');
        }
        out.extend_from_slice(digits);
    } else {
        // ddd[.ddd] with zero padding up to the decimal point.
        let decpt = decpt as usize;
        let int_part = digits.len().min(decpt);
        if decpt == 0 {
            out.push(b'0');
        }
        out.extend_from_slice(&digits[..int_part]);
        for _ in int_part..decpt {
            out.push(b'0');
        }
        if digits.len() > decpt {
            out.push(b'.');
            out.extend_from_slice(&digits[decpt..]);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p14(v: f64) -> String {
        String::from_utf8(double_to_precision(v, 14)).unwrap()
    }
    fn sh(v: f64) -> String {
        String::from_utf8(double_to_shortest(v)).unwrap()
    }

    #[test]
    fn echo_mode_matches_oracle() {
        // Expected outputs verified against /tmp/php-src php 8.5.7:
        assert_eq!(p14(0.1 + 0.2), "0.3");
        assert_eq!(p14(1.0 / 3.0), "0.33333333333333");
        assert_eq!(p14(1e15), "1.0E+15");
        assert_eq!(p14(1e14), "1.0E+14");
        assert_eq!(p14(99999999999999.0), "99999999999999");
        assert_eq!(p14(0.0001), "0.0001");
        assert_eq!(p14(0.00001), "1.0E-5");
        assert_eq!(p14(-0.0), "-0");
        assert_eq!(p14(0.0), "0");
        assert_eq!(p14(100.0), "100");
        assert_eq!(p14(1.5), "1.5");
        assert_eq!(p14(123.456), "123.456");
        assert_eq!(p14(9.223372036854776e18), "9.2233720368548E+18");
        assert_eq!(p14(f64::INFINITY), "INF");
        assert_eq!(p14(f64::NEG_INFINITY), "-INF");
        assert_eq!(p14(f64::NAN), "NAN");
    }

    #[test]
    fn shortest_mode_matches_oracle() {
        assert_eq!(sh(1e15), "1000000000000000");
        assert_eq!(sh(1e16), "10000000000000000");
        assert_eq!(sh(1e17), "1.0E+17");
        assert_eq!(sh(1e21), "1.0E+21");
        assert_eq!(sh(-1e21), "-1.0E+21");
        assert_eq!(sh(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(sh(1.0 / 3.0), "0.3333333333333333");
        assert_eq!(sh(0.001), "0.001");
        assert_eq!(sh(0.0001), "0.0001");
        assert_eq!(sh(0.00001), "1.0E-5");
        assert_eq!(sh(1.5e-5), "1.5E-5");
        assert_eq!(sh(123456789012345678.0), "1.2345678901234568E+17");
        assert_eq!(sh(f64::MAX), "1.7976931348623157E+308");
        assert_eq!(sh(5e-324), "5.0E-324");
        assert_eq!(sh(-0.0), "-0");
        assert_eq!(sh(0.0), "0");
        assert_eq!(sh(9.223372036854776e18), "9.223372036854776E+18");
    }
}
