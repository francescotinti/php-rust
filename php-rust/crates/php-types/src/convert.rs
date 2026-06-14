//! Type conversions (convert_to_* family, Zend/zend_operators.c:687-850).

use std::rc::Rc;

use crate::diag::{Diag, Diags};
use crate::dtoa::double_to_precision;
use crate::numstr::{parse_numeric_ex, Num};
use crate::{PhpStr, ZStr, Zval};

/// convert_to_boolean (zend_operators.c:687-756). Falsy: null, false, 0,
/// ±0.0, "", "0", []. NAN is truthy but warns (verified on the 8.5.7 oracle:
/// "Warning: unexpected NAN value was coerced to bool").
pub fn to_bool(v: &Zval, diags: &mut Diags) -> bool {
    match v {
        Zval::Undef | Zval::Null => false,
        Zval::Bool(b) => *b,
        Zval::Long(l) => *l != 0,
        Zval::Double(d) => {
            if d.is_nan() {
                diags.push(Diag::Warning(
                    "unexpected NAN value was coerced to bool".to_string(),
                ));
            }
            *d != 0.0 || d.is_nan()
        }
        Zval::Str(s) => {
            let b = s.as_bytes();
            !(b.is_empty() || b == b"0")
        }
        Zval::Array(a) => !a.is_empty(),
        Zval::Ref(c) => to_bool(&c.borrow(), diags),
        // An object (closure) is always truthy (step 18).
        Zval::Closure(_) => true,
    }
}

/// Silent truthiness for engine-internal paths (zend_compare fallback uses
/// zval_is_true without the NAN warning observable difference is nil since
/// compare never reaches it with NAN — the NAN block runs first).
pub fn is_true_silent(v: &Zval) -> bool {
    match v {
        Zval::Double(d) => *d != 0.0 || d.is_nan(),
        Zval::Undef | Zval::Null => false,
        Zval::Bool(b) => *b,
        Zval::Long(l) => *l != 0,
        Zval::Str(s) => {
            let b = s.as_bytes();
            !(b.is_empty() || b == b"0")
        }
        Zval::Array(a) => !a.is_empty(),
        Zval::Ref(c) => is_true_silent(&c.borrow()),
        Zval::Closure(_) => true,
    }
}

/// zend_dval_to_lval (zend_operators.h:126-172): truncation toward zero;
/// non-finite -> 0; out-of-range -> modular reduction into i64 (the C
/// "slow" path does the same two's-complement wrap).
pub fn dval_to_lval(d: f64) -> i64 {
    if !d.is_finite() {
        return 0;
    }
    if d >= -(i64::MIN as f64) || d < i64::MIN as f64 {
        // fmod into [0, 2^64) then wrap, like zend_dval_to_lval_slow.
        const TWO_P64: f64 = 18446744073709551616.0;
        let mut m = d % TWO_P64;
        if m < 0.0 {
            m += TWO_P64;
        }
        return m as u64 as i64;
    }
    d as i64
}

/// ZEND_DOUBLE_FITS_LONG.
pub fn fits_long(d: f64) -> bool {
    !(d >= -(i64::MIN as f64) || d < i64::MIN as f64)
}

/// zend_is_long_compatible: the conversion round-trips exactly.
pub fn is_long_compatible(d: f64, l: i64) -> bool {
    d == l as f64
}

fn warn_not_representable(d: f64, diags: &mut Diags) {
    diags.push(Diag::Warning(format!(
        "The float {} is not representable as an int, cast occurred",
        String::from_utf8_lossy(&double_to_precision(d, 14))
    )));
}

/// zend_dval_to_lval (the erroring flavor in zend_operators.h:126-172):
/// warns for non-finite / out-of-range, then converts like the silent one.
pub fn dval_to_lval_noisy(d: f64, diags: &mut Diags) -> i64 {
    if !d.is_finite() {
        warn_not_representable(d, diags);
        0
    } else if !fits_long(d) {
        warn_not_representable(d, diags);
        dval_to_lval(d)
    } else {
        d as i64
    }
}

/// zend_dval_to_lval_safe: noisy conversion + the lossy deprecation when the
/// value fit but did not round-trip (oracle: NAN | 0 emits BOTH diagnostics).
pub fn dval_to_lval_safe(d: f64, diags: &mut Diags) -> i64 {
    let l = dval_to_lval_noisy(d, diags);
    if !is_long_compatible(d, l) && fits_long(d) {
        diags.push(Diag::Deprecated(format!(
            "Implicit conversion from float {} to int loses precision",
            String::from_utf8_lossy(&double_to_precision(d, 14))
        )));
    }
    l
}

/// Explicit (int) cast semantics: "abc" -> 0, "5abc" -> 5 silently, but
/// non-representable floats warn (oracle: "Warning: The float NAN is not
/// representable as an int, cast occurred").
pub fn to_long_cast(v: &Zval, diags: &mut Diags) -> i64 {
    match v {
        Zval::Undef | Zval::Null => 0,
        Zval::Bool(b) => *b as i64,
        Zval::Long(l) => *l,
        Zval::Double(d) => {
            if !d.is_finite() || *d >= -(i64::MIN as f64) || *d < i64::MIN as f64 {
                diags.push(Diag::Warning(format!(
                    "The float {} is not representable as an int, cast occurred",
                    String::from_utf8_lossy(&double_to_precision(*d, 14))
                )));
            }
            dval_to_lval(*d)
        }
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), true) {
            Some(i) => match i.num {
                Num::Long(l) => l,
                Num::Double(d) => dval_to_lval(d),
            },
            None => 0,
        },
        Zval::Array(a) => !a.is_empty() as i64,
        Zval::Ref(c) => to_long_cast(&c.borrow(), diags),
        // Object → int: objects are truthy, yielding 1 (step 18; PHP also warns,
        // an edge case not yet modelled).
        Zval::Closure(_) => 1,
    }
}

/// zval_get_double semantics (silent).
pub fn to_double(v: &Zval) -> f64 {
    match v {
        Zval::Undef | Zval::Null => 0.0,
        Zval::Bool(b) => *b as i64 as f64,
        Zval::Long(l) => *l as f64,
        Zval::Double(d) => *d,
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), true) {
            Some(i) => match i.num {
                Num::Long(l) => l as f64,
                Num::Double(d) => d,
            },
            None => 0.0,
        },
        Zval::Array(a) => !a.is_empty() as i64 as f64,
        Zval::Ref(c) => to_double(&c.borrow()),
        Zval::Closure(_) => 1.0,
    }
}

/// String conversion for echo/concat. Arrays convert to "Array" with a
/// warning (they only TypeError in stricter contexts handled elsewhere).
pub fn to_zstr(v: &Zval, diags: &mut Diags) -> ZStr {
    match v {
        Zval::Undef | Zval::Null => PhpStr::empty(),
        Zval::Bool(false) => PhpStr::empty(),
        Zval::Bool(true) => PhpStr::from_str("1"),
        Zval::Long(l) => PhpStr::new(l.to_string().into_bytes()),
        // NAN converts silently here (oracle: null . NAN); the explicit
        // (string) cast warns — see to_zstr_cast.
        Zval::Double(d) => PhpStr::new(double_to_precision(*d, 14)),
        Zval::Str(s) => Rc::clone(s),
        Zval::Array(_) => {
            diags.push(Diag::Warning("Array to string conversion".to_string()));
            PhpStr::from_str("Array")
        }
        Zval::Ref(c) => to_zstr(&c.borrow(), diags),
        // PHP actually raises a fatal `Error: Object of class Closure could not
        // be converted to string`; this infallible funnel cannot, so it warns
        // and yields a placeholder (step 18 scope-out, revisit with OOP).
        Zval::Closure(_) => {
            diags.push(Diag::Warning(
                "Object of class Closure could not be converted to string".to_string(),
            ));
            PhpStr::from_str("Closure")
        }
    }
}

/// Explicit (string) cast: like to_zstr but NAN warns
/// (oracle: "Warning: unexpected NAN value was coerced to string").
pub fn to_zstr_cast(v: &Zval, diags: &mut Diags) -> ZStr {
    if let Zval::Double(d) = v {
        if d.is_nan() {
            diags.push(Diag::Warning(
                "unexpected NAN value was coerced to string".to_string(),
            ));
        }
    }
    to_zstr(v, diags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PhpArray;

    #[test]
    fn bool_falsy_table() {
        let mut d = Diags::new();
        assert!(!to_bool(&Zval::Null, &mut d));
        assert!(!to_bool(&Zval::Long(0), &mut d));
        assert!(!to_bool(&Zval::Double(0.0), &mut d));
        assert!(!to_bool(&Zval::Double(-0.0), &mut d));
        assert!(!to_bool(&Zval::str_from(""), &mut d));
        assert!(!to_bool(&Zval::str_from("0"), &mut d));
        assert!(!to_bool(&Zval::Array(Rc::new(PhpArray::new())), &mut d));
        assert!(to_bool(&Zval::str_from("0.0"), &mut d));
        assert!(to_bool(&Zval::str_from("00"), &mut d));
        assert!(to_bool(&Zval::str_from(" "), &mut d));
        assert!(d.is_empty());
        // NAN: truthy + warning (oracle-verified)
        assert!(to_bool(&Zval::Double(f64::NAN), &mut d));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn long_conversions() {
        let mut d = Diags::new();
        assert_eq!(to_long_cast(&Zval::str_from("7.9"), &mut d), 7);
        assert_eq!(to_long_cast(&Zval::str_from("5abc"), &mut d), 5);
        assert_eq!(to_long_cast(&Zval::str_from("abc"), &mut d), 0);
        assert!(d.is_empty());
        assert_eq!(to_long_cast(&Zval::Double(f64::NAN), &mut d), 0);
        assert_eq!(d.len(), 1); // NAN cast warning
        assert_eq!(dval_to_lval(f64::NAN), 0);
        assert_eq!(dval_to_lval(f64::INFINITY), 0);
        assert_eq!(dval_to_lval(-7.9), -7);
    }

    #[test]
    fn string_conversions() {
        let mut d = Diags::new();
        assert_eq!(to_zstr(&Zval::Bool(true), &mut d).as_bytes(), b"1");
        assert_eq!(to_zstr(&Zval::Bool(false), &mut d).as_bytes(), b"");
        assert_eq!(to_zstr(&Zval::Null, &mut d).as_bytes(), b"");
        assert_eq!(to_zstr(&Zval::Double(0.1 + 0.2), &mut d).as_bytes(), b"0.3");
        assert!(d.is_empty());
        let a = Zval::Array(Rc::new(PhpArray::new()));
        assert_eq!(to_zstr(&a, &mut d).as_bytes(), b"Array");
        assert_eq!(d.len(), 1);
    }
}
