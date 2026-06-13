//! Numeric-string recognition: faithful port of `_is_numeric_string_ex`
//! (Zend/zend_operators.c:3620-3750).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Num {
    Long(i64),
    Double(f64),
}

#[derive(Debug, Clone, Copy)]
pub struct NumStrInfo {
    pub num: Num,
    /// -1 / 0 / 1: integer literal overflowed below i64::MIN / no / above i64::MAX.
    pub oflow: i8,
    /// True when `allow_errors` and non-whitespace bytes follow the number
    /// ("leading-numeric string", e.g. "5abc").
    pub trailing: bool,
}

const MAX_LENGTH_OF_LONG: usize = 20; // Zend/zend_long.h:112 (64-bit)
const LONG_MIN_DIGITS: &[u8] = b"9223372036854775808"; // |i64::MIN|

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// Scan a C-strtod-style double starting at `from` (sign position) and parse
/// it. Returns (value, end-of-consumed-input). Caller guarantees a valid
/// number starts there.
fn scan_double(bytes: &[u8], from: usize) -> (f64, usize) {
    let mut j = from;
    if matches!(bytes.get(j), Some(b'+' | b'-')) {
        j += 1;
    }
    let int_start = j;
    while matches!(bytes.get(j), Some(d) if d.is_ascii_digit()) {
        j += 1;
    }
    let int_digits = j - int_start;
    let mut end = j;
    if matches!(bytes.get(j), Some(b'.')) {
        let mut m = j + 1;
        while matches!(bytes.get(m), Some(d) if d.is_ascii_digit()) {
            m += 1;
        }
        // strtod consumes "5." and ".5" but not "." alone.
        if m > j + 1 || int_digits > 0 {
            end = m;
        }
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) && end > from {
        let mut m = end + 1;
        if matches!(bytes.get(m), Some(b'+' | b'-')) {
            m += 1;
        }
        if matches!(bytes.get(m), Some(d) if d.is_ascii_digit()) {
            while matches!(bytes.get(m), Some(d) if d.is_ascii_digit()) {
                m += 1;
            }
            end = m;
        }
    }
    let s = std::str::from_utf8(&bytes[from..end]).expect("numeric span is ASCII");
    // Rust's f64 parser accepts the same prefix grammar (incl. "5.", ".5",
    // huge exponents -> inf) once we've bounded the span ourselves.
    (s.parse::<f64>().expect("scanned span must parse"), end)
}

pub fn parse_numeric_ex(bytes: &[u8], allow_errors: bool) -> Option<NumStrInfo> {
    if bytes.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < bytes.len() && is_ws(bytes[i]) {
        i += 1;
    }
    let num_start = i; // C: `str` after whitespace skip (strtod starts here)
    let mut neg = false;
    match bytes.get(i) {
        Some(b'-') => {
            neg = true;
            i += 1;
        }
        Some(b'+') => {
            i += 1;
        }
        _ => {}
    }

    let mut oflow: i8 = 0;
    let num;
    let mut end;
    let mut long_digits: Option<(usize, usize, u64)> = None; // (first, count, accum)

    match bytes.get(i) {
        Some(d) if d.is_ascii_digit() => {
            while matches!(bytes.get(i), Some(b'0')) {
                i += 1;
            }
            let first = i;
            let mut tmp: u64 = 0;
            let mut digits = 0usize;
            let mut to_double = false;
            loop {
                if digits >= MAX_LENGTH_OF_LONG {
                    // Integer overflow (zend_operators.c:3686-3692).
                    oflow = if neg { -1 } else { 1 };
                    to_double = true;
                    break;
                }
                match bytes.get(i) {
                    Some(d) if d.is_ascii_digit() => {
                        tmp = tmp.wrapping_mul(10).wrapping_add((d - b'0') as u64);
                        digits += 1;
                        i += 1;
                    }
                    Some(b'.') => {
                        to_double = true;
                        break;
                    }
                    Some(b'e' | b'E') => {
                        let mut e = i + 1;
                        if matches!(bytes.get(e), Some(b'-' | b'+')) {
                            e += 1;
                        }
                        if matches!(bytes.get(e), Some(d) if d.is_ascii_digit()) {
                            to_double = true;
                        }
                        break;
                    }
                    _ => break,
                }
            }
            if to_double {
                let (v, e) = scan_double(bytes, num_start);
                num = Num::Double(v);
                end = e;
            } else {
                long_digits = Some((first, digits, tmp));
                num = Num::Long(0); // provisional; finalized below
                end = i;
            }
        }
        Some(b'.') if matches!(bytes.get(i + 1), Some(d) if d.is_ascii_digit()) => {
            let (v, e) = scan_double(bytes, num_start);
            num = Num::Double(v);
            end = e;
        }
        _ => return None,
    }

    // Trailing whitespace is allowed in both modes (zend_operators.c:3709-3722);
    // other trailing bytes only with allow_errors ("leading-numeric").
    let mut trailing = false;
    while end < bytes.len() && is_ws(bytes[end]) {
        end += 1;
    }
    if end != bytes.len() {
        if !allow_errors {
            return None;
        }
        trailing = true;
    }

    // Finalize the integer path, incl. the 19-digit boundary against
    // |i64::MIN| (zend_operators.c:3725-3745).
    let num = match long_digits {
        None => num,
        Some((first, digits, tmp)) => {
            if digits == MAX_LENGTH_OF_LONG - 1 {
                let span = &bytes[first..first + digits];
                let cmp = span.cmp(LONG_MIN_DIGITS);
                if !(cmp == std::cmp::Ordering::Less
                    || (cmp == std::cmp::Ordering::Equal && neg))
                {
                    oflow = if neg { -1 } else { 1 };
                    let (v, _) = scan_double(bytes, num_start);
                    return Some(NumStrInfo {
                        num: Num::Double(v),
                        oflow,
                        trailing,
                    });
                }
            }
            let v = if neg {
                (tmp.wrapping_neg()) as i64
            } else {
                tmp as i64
            };
            Num::Long(v)
        }
    };

    Some(NumStrInfo { num, oflow, trailing })
}

/// `is_numeric_string` without error tolerance.
pub fn parse_numeric(bytes: &[u8]) -> Option<Num> {
    parse_numeric_ex(bytes, false).map(|i| i.num)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(s: &str) -> Option<Num> {
        parse_numeric(s.as_bytes())
    }

    #[test]
    fn longs() {
        assert_eq!(n("0"), Some(Num::Long(0)));
        assert_eq!(n("123"), Some(Num::Long(123)));
        assert_eq!(n("-123"), Some(Num::Long(-123)));
        assert_eq!(n("+5"), Some(Num::Long(5)));
        assert_eq!(n("0007"), Some(Num::Long(7)));
        assert_eq!(n(" 5 "), Some(Num::Long(5))); // ws both sides, PHP 8
        assert_eq!(n("9223372036854775807"), Some(Num::Long(i64::MAX)));
        assert_eq!(n("-9223372036854775808"), Some(Num::Long(i64::MIN)));
    }

    #[test]
    fn long_overflow_to_double() {
        assert_eq!(n("9223372036854775808"), Some(Num::Double(9.223372036854776e18)));
        assert_eq!(n("-9223372036854775809"), Some(Num::Double(-9.223372036854776e18)));
        let info = parse_numeric_ex(b"99999999999999999999999", false).unwrap();
        assert_eq!(info.oflow, 1);
        assert!(matches!(info.num, Num::Double(_)));
    }

    #[test]
    fn doubles() {
        assert_eq!(n("1.5"), Some(Num::Double(1.5)));
        assert_eq!(n(".5"), Some(Num::Double(0.5)));
        assert_eq!(n("5."), Some(Num::Double(5.0)));
        assert_eq!(n("1e3"), Some(Num::Double(1000.0)));
        assert_eq!(n("1E+3"), Some(Num::Double(1000.0)));
        assert_eq!(n("-1.5e-2"), Some(Num::Double(-0.015)));
        assert_eq!(n("1e999"), Some(Num::Double(f64::INFINITY)));
    }

    #[test]
    fn non_numeric() {
        for s in ["", " ", "abc", "0x1A", ".", "-", "+", "e5", "5e", "1.2.3", "5 5"] {
            assert_eq!(n(s), None, "{s:?}");
        }
    }

    #[test]
    fn leading_numeric() {
        let i = parse_numeric_ex(b"5abc", true).unwrap();
        assert_eq!(i.num, Num::Long(5));
        assert!(i.trailing);
        let i = parse_numeric_ex(b"1.5x", true).unwrap();
        assert_eq!(i.num, Num::Double(1.5));
        assert!(i.trailing);
        let i = parse_numeric_ex(b"5e", true).unwrap(); // bare 'e' not consumed
        assert_eq!(i.num, Num::Long(5));
        assert!(i.trailing);
        let i = parse_numeric_ex(b"0xFF", true).unwrap(); // leading zero, then 'x'
        assert_eq!(i.num, Num::Long(0));
        assert!(i.trailing);
        assert!(parse_numeric_ex(b"abc", true).is_none());
    }

    #[test]
    fn trailing_ws_is_not_trailing_data() {
        let i = parse_numeric_ex(b"5  \t\n", true).unwrap();
        assert_eq!(i.num, Num::Long(5));
        assert!(!i.trailing);
        assert_eq!(n("5  "), Some(Num::Long(5)));
    }
}
