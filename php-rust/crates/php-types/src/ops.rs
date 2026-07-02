//! Operators: faithful port of the observable semantics of
//! Zend/zend_operators.c (the one module ported line-for-line, D-G11).
//!
//! All "oracle:" comments refer to behavior verified against the reference
//! php 8.5.7 CLI built from this repo's source.

use std::rc::Rc;

use crate::convert::{dval_to_lval_safe, fits_long, is_long_compatible, is_true_silent, to_zstr};
use crate::diag::{Diag, Diags, PhpError};
use crate::dtoa::double_to_precision;
use crate::numstr::{parse_numeric, parse_numeric_ex, Num};
use crate::{Key, PhpArray, PhpStr, Zval};

pub type OpResult = Result<Zval, PhpError>;

/// ZEND_THREEWAY_COMPARE (zend_portability.h:516): NAN involvement yields 1.
fn threeway(a: f64, b: f64) -> i32 {
    if a == b {
        0
    } else if a < b {
        -1
    } else {
        1
    }
}

fn type_pair_error(a: &Zval, b: &Zval, sym: &str) -> PhpError {
    PhpError::TypeError(format!(
        "Unsupported operand types: {} {} {}",
        a.type_name_for_error(),
        sym,
        b.type_name_for_error()
    ))
}

/// Numeric operand for arithmetic binops, mirroring the
/// zendi_try_convert_scalar_to_number slow path: fully-numeric strings are
/// silent, leading-numeric warn, non-numeric/array fail (caller raises the
/// op-specific TypeError).
fn try_to_number(v: &Zval, diags: &mut Diags) -> Option<Num> {
    match v {
        Zval::Undef | Zval::Null => Some(Num::Long(0)),
        Zval::Bool(b) => Some(Num::Long(*b as i64)),
        Zval::Long(l) => Some(Num::Long(*l)),
        Zval::Double(d) => Some(Num::Double(*d)),
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), true) {
            Some(info) => {
                if info.trailing {
                    diags.push(Diag::Warning("A non-numeric value encountered".to_string()));
                }
                Some(info.num)
            }
            None => None,
        },
        Zval::Array(_) => None,
        // An object (closure or instance) has no numeric value: the caller raises
        // the op's TypeError ("Unsupported operand types: ...", step 18/19). A
        // resource is the same (oracle: `$fp + 1` → TypeError, step 51).
        Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) | Zval::WeakHandle(_) | Zval::Resource(_) => None,
        // A reference is converted as its target (deref-on-read should make this
        // unreachable, but recursing keeps the op correct if one slips through).
        Zval::Ref(c) => try_to_number(&c.borrow(), diags),
    }
}

/// Integer operand for %, <<, >>, &, |, ^: port of zendi_try_get_long
/// (zend_operators.c:378). Doubles go through dval_to_lval_safe; float
/// strings use strtol-style SATURATION (zend_dval_to_lval_cap) plus the
/// "float-string" deprecation only when the value doesn't round-trip
/// (oracle: "9223372036854775808"|0 is silent, "1e100"|0 deprecates).
fn try_to_long(v: &Zval, diags: &mut Diags) -> Option<i64> {
    match v {
        Zval::Undef | Zval::Null => Some(0),
        Zval::Bool(b) => Some(*b as i64),
        Zval::Long(l) => Some(*l),
        Zval::Double(d) => Some(dval_to_lval_safe(*d, diags)),
        Zval::Str(s) => {
            let info = parse_numeric_ex(s.as_bytes(), true)?;
            if info.trailing {
                diags.push(Diag::Warning("A non-numeric value encountered".to_string()));
            }
            match info.num {
                Num::Long(l) => Some(l),
                Num::Double(d) => {
                    let l = if !d.is_finite() {
                        0
                    } else if !fits_long(d) {
                        if d > 0.0 {
                            i64::MAX
                        } else {
                            i64::MIN
                        }
                    } else {
                        d as i64
                    };
                    if !is_long_compatible(d, l) {
                        diags.push(Diag::Deprecated(format!(
                            "Implicit conversion from float-string \"{}\" to int loses precision",
                            String::from_utf8_lossy(s.as_bytes())
                        )));
                    }
                    Some(l)
                }
            }
        }
        Zval::Array(_) => None,
        Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) | Zval::WeakHandle(_) | Zval::Resource(_) => None,
        Zval::Ref(c) => try_to_long(&c.borrow(), diags),
    }
}

/// Sequential operand conversion: op1 first; if it fails op2 is NOT
/// converted (no spurious warning) — oracle: "abc" + "5xyz" vs "5xyz" + "abc".
fn binop_nums(a: &Zval, b: &Zval, sym: &str, diags: &mut Diags) -> Result<(Num, Num), PhpError> {
    let x = try_to_number(a, diags).ok_or_else(|| type_pair_error(a, b, sym))?;
    let y = try_to_number(b, diags).ok_or_else(|| type_pair_error(a, b, sym))?;
    Ok((x, y))
}

fn binop_longs(a: &Zval, b: &Zval, sym: &str, diags: &mut Diags) -> Result<(i64, i64), PhpError> {
    let x = try_to_long(a, diags).ok_or_else(|| type_pair_error(a, b, sym))?;
    let y = try_to_long(b, diags).ok_or_else(|| type_pair_error(a, b, sym))?;
    Ok((x, y))
}

// ---------------------------------------------------------------------------
// Arithmetic
// ---------------------------------------------------------------------------

pub fn add(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    if let (Zval::Array(x), Zval::Array(y)) = (a, b) {
        // Array union: left keys win (add_function_slow).
        let mut out = (**x).clone();
        for (k, v) in y.iter() {
            if !out.contains_key(k) {
                out.insert(k.clone(), v.clone());
            }
        }
        return Ok(Zval::Array(Rc::new(out)));
    }
    let (x, y) = binop_nums(a, b, "+", diags)?;
    Ok(match (x, y) {
        (Num::Long(l), Num::Long(r)) => match l.checked_add(r) {
            Some(s) => Zval::Long(s),
            None => Zval::Double(l as f64 + r as f64),
        },
        (x, y) => Zval::Double(num_f64(x) + num_f64(y)),
    })
}

pub fn sub(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_nums(a, b, "-", diags)?;
    Ok(match (x, y) {
        (Num::Long(l), Num::Long(r)) => match l.checked_sub(r) {
            Some(s) => Zval::Long(s),
            None => Zval::Double(l as f64 - r as f64),
        },
        (x, y) => Zval::Double(num_f64(x) - num_f64(y)),
    })
}

pub fn mul(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_nums(a, b, "*", diags)?;
    Ok(match (x, y) {
        (Num::Long(l), Num::Long(r)) => match l.checked_mul(r) {
            Some(s) => Zval::Long(s),
            None => Zval::Double(l as f64 * r as f64),
        },
        (x, y) => Zval::Double(num_f64(x) * num_f64(y)),
    })
}

pub fn div(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_nums(a, b, "/", diags)?;
    // Division by zero throws for both int and float divisors (PHP 8).
    let divisor_is_zero = match y {
        Num::Long(n) => n == 0,
        Num::Double(d) => d == 0.0,
    };
    if divisor_is_zero {
        return Err(PhpError::DivisionByZeroError("Division by zero"));
    }
    Ok(match (x, y) {
        (Num::Long(l), Num::Long(r)) => {
            if l == i64::MIN && r == -1 {
                Zval::Double(-(i64::MIN as f64))
            } else if l % r == 0 {
                Zval::Long(l / r)
            } else {
                Zval::Double(l as f64 / r as f64)
            }
        }
        (x, y) => Zval::Double(num_f64(x) / num_f64(y)),
    })
}

pub fn modulo(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_longs(a, b, "%", diags)?;
    if y == 0 {
        return Err(PhpError::DivisionByZeroError("Modulo by zero"));
    }
    // C truncated semantics; i64::MIN % -1 would overflow in Rust's checked
    // world but is 0 mathematically (Zend special-cases it the same way).
    if y == -1 {
        return Ok(Zval::Long(0));
    }
    Ok(Zval::Long(x % y))
}

pub fn pow(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_nums(a, b, "**", diags)?;
    if let (Num::Long(base), Num::Long(exp)) = (x, y) {
        if exp >= 0 {
            return Ok(int_pow(base, exp));
        }
        return Ok(Zval::Double(safe_pow(base as f64, exp as f64, diags)));
    }
    let (bf, ef) = (num_f64(x), num_f64(y));
    Ok(Zval::Double(safe_pow(bf, ef, diags)))
}

fn safe_pow(base: f64, exponent: f64, diags: &mut Diags) -> f64 {
    if base == 0.0 && exponent < 0.0 {
        diags.push(Diag::Deprecated(
            "Power of base 0 and negative exponent is deprecated".to_string(),
        ));
    }
    base.powf(exponent)
}

/// pow_function_base long path: on overflow the square-and-multiply loop
/// CONTINUES in double from the overflow point (preserving the accumulated
/// rounding and the sign of odd exponents) — oracle: 5**100, PHP_INT_MIN**MAX.
fn int_pow(base: i64, exp: i64) -> Zval {
    if exp == 0 {
        return Zval::Long(1);
    }
    if base == 0 {
        return Zval::Long(0);
    }
    let mut l1: i64 = 1;
    let mut l2: i64 = base;
    let mut i: i64 = exp;
    while i >= 1 {
        if i % 2 != 0 {
            i -= 1;
            match l1.checked_mul(l2) {
                Some(r) => l1 = r,
                None => {
                    let dval = l1 as f64 * l2 as f64;
                    return Zval::Double(dval * (l2 as f64).powf(i as f64));
                }
            }
        } else {
            i /= 2;
            match l2.checked_mul(l2) {
                Some(r) => l2 = r,
                None => {
                    let dval = l2 as f64 * l2 as f64;
                    return Zval::Double(l1 as f64 * dval.powf(i as f64));
                }
            }
        }
    }
    Zval::Long(l1)
}

pub fn concat(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let sa = to_zstr(a, diags);
    let sb = to_zstr(b, diags);
    let mut out = Vec::with_capacity(sa.len() + sb.len());
    out.extend_from_slice(sa.as_bytes());
    out.extend_from_slice(sb.as_bytes());
    Ok(Zval::Str(PhpStr::new(out)))
}

pub fn neg(a: &Zval, diags: &mut Diags) -> OpResult {
    // Compiled as multiplication by -1; error message says "string * int"
    // (oracle: -"abc").
    mul(a, &Zval::Long(-1), diags)
}

fn num_f64(n: Num) -> f64 {
    match n {
        Num::Long(l) => l as f64,
        Num::Double(d) => d,
    }
}

// ---------------------------------------------------------------------------
// Bitwise
// ---------------------------------------------------------------------------

fn bitwise_str(a: &[u8], b: &[u8], op: u8) -> Vec<u8> {
    // String operands: bytewise. AND/XOR -> shorter length, OR -> longer
    // with the longer operand's tail copied (zend_operators.c bitwise_*).
    match op {
        b'&' => a.iter().zip(b).map(|(x, y)| x & y).collect(),
        b'^' => a.iter().zip(b).map(|(x, y)| x ^ y).collect(),
        _ => {
            let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
            let mut out = long.to_vec();
            for (i, y) in short.iter().enumerate() {
                out[i] |= y;
            }
            out
        }
    }
}

fn bitop(a: &Zval, b: &Zval, sym: &str, op: u8, diags: &mut Diags) -> OpResult {
    if let (Zval::Str(x), Zval::Str(y)) = (a, b) {
        return Ok(Zval::Str(PhpStr::new(bitwise_str(
            x.as_bytes(),
            y.as_bytes(),
            op,
        ))));
    }
    let (x, y) = binop_longs(a, b, sym, diags)?;
    Ok(Zval::Long(match op {
        b'&' => x & y,
        b'|' => x | y,
        _ => x ^ y,
    }))
}

pub fn bw_and(a: &Zval, b: &Zval, d: &mut Diags) -> OpResult {
    bitop(a, b, "&", b'&', d)
}
pub fn bw_or(a: &Zval, b: &Zval, d: &mut Diags) -> OpResult {
    bitop(a, b, "|", b'|', d)
}
pub fn bw_xor(a: &Zval, b: &Zval, d: &mut Diags) -> OpResult {
    bitop(a, b, "^", b'^', d)
}

pub fn bw_not(a: &Zval, diags: &mut Diags) -> OpResult {
    match a {
        Zval::Str(s) => Ok(Zval::Str(PhpStr::new(
            s.as_bytes().iter().map(|b| !b).collect::<Vec<u8>>(),
        ))),
        Zval::Long(l) => Ok(Zval::Long(!l)),
        Zval::Double(d) => Ok(Zval::Long(!dval_to_lval_safe(*d, diags))),
        Zval::Ref(c) => bw_not(&c.borrow(), diags),
        // zend_zval_value_name: bools spell out their value (oracle: ~true), an
        // object is named by its class.
        _ => Err(PhpError::TypeError(format!(
            "Cannot perform bitwise not on {}",
            match a {
                Zval::Bool(true) => "true".to_string(),
                Zval::Bool(false) => "false".to_string(),
                other => other.type_name_for_error(),
            }
        ))),
    }
}

pub fn shl(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_longs(a, b, "<<", diags)?;
    if y < 0 {
        return Err(PhpError::ArithmeticError("Bit shift by negative number"));
    }
    if y >= 64 {
        return Ok(Zval::Long(0));
    }
    Ok(Zval::Long(((x as u64) << y) as i64))
}

pub fn shr(a: &Zval, b: &Zval, diags: &mut Diags) -> OpResult {
    let (x, y) = binop_longs(a, b, ">>", diags)?;
    if y < 0 {
        return Err(PhpError::ArithmeticError("Bit shift by negative number"));
    }
    if y >= 64 {
        return Ok(Zval::Long(if x < 0 { -1 } else { 0 }));
    }
    Ok(Zval::Long(x >> y))
}

// ---------------------------------------------------------------------------
// Comparison
// ---------------------------------------------------------------------------

/// compare_long_to_string (zend_operators.c:2260): non-numeric strings make
/// the int compare AS A STRING (PHP 8).
fn compare_long_to_string(lval: i64, s: &PhpStr) -> i32 {
    match parse_numeric(s.as_bytes()) {
        Some(Num::Long(r)) => threeway_i(lval, r),
        Some(Num::Double(r)) => threeway(lval as f64, r),
        None => normalize(byte_cmp(lval.to_string().as_bytes(), s.as_bytes())),
    }
}

fn compare_double_to_string(dval: f64, s: &PhpStr) -> i32 {
    debug_assert!(!dval.is_nan());
    match parse_numeric(s.as_bytes()) {
        Some(Num::Long(r)) => threeway(dval, r as f64),
        Some(Num::Double(r)) => threeway(dval, r),
        None => normalize(byte_cmp(&double_to_precision(dval, 14), s.as_bytes())),
    }
}

fn byte_cmp(a: &[u8], b: &[u8]) -> i32 {
    match a.cmp(b) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn normalize(n: i32) -> i32 {
    n.signum()
}

fn threeway_i(a: i64, b: i64) -> i32 {
    if a > b {
        1
    } else if a < b {
        -1
    } else {
        0
    }
}

/// zendi_smart_strcmp (zend_operators.c:3421): numeric-aware string ordering
/// with overflow guards.
pub fn smart_strcmp(s1: &PhpStr, s2: &PhpStr) -> i32 {
    let i1 = parse_numeric_ex(s1.as_bytes(), false);
    let i2 = parse_numeric_ex(s2.as_bytes(), false);
    if let (Some(i1), Some(i2)) = (i1, i2) {
        if i1.oflow != 0 && i1.oflow == i2.oflow && num_f64(i1.num) - num_f64(i2.num) == 0.0 {
            return normalize(byte_cmp(s1.as_bytes(), s2.as_bytes()));
        }
        match (i1.num, i2.num) {
            (Num::Long(l1), Num::Long(l2)) => threeway_i(l1, l2),
            (n1, n2) => {
                if let (Num::Long(_), Num::Double(_)) = (n1, n2) {
                    if i2.oflow != 0 {
                        return -i2.oflow as i32;
                    }
                } else if let (Num::Double(_), Num::Long(_)) = (n1, n2) {
                    if i1.oflow != 0 {
                        return i1.oflow as i32;
                    }
                } else if num_f64(n1) == num_f64(n2) && !num_f64(n1).is_finite() {
                    return normalize(byte_cmp(s1.as_bytes(), s2.as_bytes()));
                }
                normalize(if num_f64(n1) - num_f64(n2) == 0.0 {
                    0
                } else if num_f64(n1) - num_f64(n2) > 0.0 {
                    1
                } else {
                    -1
                })
            }
        }
    } else {
        normalize(byte_cmp(s1.as_bytes(), s2.as_bytes()))
    }
}

/// zendi_smart_streq (zend_operators.c:3373).
pub fn smart_streq(s1: &PhpStr, s2: &PhpStr) -> bool {
    let i1 = parse_numeric_ex(s1.as_bytes(), false);
    let i2 = parse_numeric_ex(s2.as_bytes(), false);
    if let (Some(i1), Some(i2)) = (i1, i2) {
        if i1.oflow != 0 && i1.oflow == i2.oflow && num_f64(i1.num) - num_f64(i2.num) == 0.0 {
            return s1.as_bytes() == s2.as_bytes();
        }
        match (i1.num, i2.num) {
            (Num::Long(l1), Num::Long(l2)) => l1 == l2,
            (n1, n2) => {
                if matches!(n1, Num::Long(_)) && i2.oflow != 0 {
                    return false;
                }
                if matches!(n2, Num::Long(_)) && i1.oflow != 0 {
                    return false;
                }
                let (d1, d2) = (num_f64(n1), num_f64(n2));
                if matches!(n1, Num::Double(_))
                    && matches!(n2, Num::Double(_))
                    && d1 == d2
                    && !d1.is_finite()
                {
                    return s1.as_bytes() == s2.as_bytes();
                }
                d1 == d2
            }
        }
    } else {
        s1.as_bytes() == s2.as_bytes()
    }
}

fn type_lt_true(v: &Zval) -> bool {
    matches!(v, Zval::Undef | Zval::Null | Zval::Bool(false))
}

/// zend_compare (zend_operators.c:2306-2470), minus objects/resources/refs.
pub fn compare(a: &Zval, b: &Zval) -> i32 {
    // Follow references up front (D-R11): the rest of the routine, and the
    // recursive `compare_arrays` call, then only ever see plain values.
    let mut oa = a.deref_clone();
    let mut ob = b.deref_clone();
    let mut converted = false;
    loop {
        match (&oa, &ob) {
            (Zval::Long(l), Zval::Long(r)) => return threeway_i(*l, *r),
            (Zval::Double(l), Zval::Long(r)) => return threeway(*l, *r as f64),
            (Zval::Long(l), Zval::Double(r)) => return threeway(*l as f64, *r),
            (Zval::Double(l), Zval::Double(r)) => return threeway(*l, *r),
            (Zval::Array(l), Zval::Array(r)) => return compare_arrays(l, r),
            // Two resources compare by their numeric id (oracle: `$a < $b` is
            // `id_a < id_b`, step 51).
            (Zval::Resource(l), Zval::Resource(r)) => {
                return threeway_i(l.borrow().id as i64, r.borrow().id as i64)
            }
            (Zval::Undef | Zval::Null, Zval::Undef | Zval::Null) => return 0,
            (Zval::Undef | Zval::Null, Zval::Bool(rb)) => return if *rb { -1 } else { 0 },
            (Zval::Bool(lb), Zval::Undef | Zval::Null) => return if *lb { 1 } else { 0 },
            (Zval::Bool(lb), Zval::Bool(rb)) => return threeway_i(*lb as i64, *rb as i64),
            (Zval::Str(l), Zval::Str(r)) => {
                if Rc::ptr_eq(l, r) {
                    return 0;
                }
                return smart_strcmp(l, r);
            }
            (Zval::Undef | Zval::Null, Zval::Str(r)) => {
                return if r.is_empty() { 0 } else { -1 };
            }
            (Zval::Str(l), Zval::Undef | Zval::Null) => {
                return if l.is_empty() { 0 } else { 1 };
            }
            (Zval::Long(l), Zval::Str(r)) => return compare_long_to_string(*l, r),
            (Zval::Str(l), Zval::Long(r)) => return -compare_long_to_string(*r, l),
            (Zval::Double(l), Zval::Str(r)) => {
                if l.is_nan() {
                    return 1;
                }
                return compare_double_to_string(*l, r);
            }
            (Zval::Str(l), Zval::Double(r)) => {
                if r.is_nan() {
                    return 1;
                }
                return -compare_double_to_string(*r, l);
            }
            _ => {
                if !converted {
                    let nan_present = matches!(&oa, Zval::Double(d) if d.is_nan())
                        || matches!(&ob, Zval::Double(d) if d.is_nan());
                    if nan_present {
                        if type_lt_true(&oa) {
                            return -1;
                        } else if matches!(oa, Zval::Bool(true)) || matches!(ob, Zval::Bool(true))
                        {
                            return 0;
                        } else if type_lt_true(&ob) {
                            return 1;
                        } else if !matches!(oa, Zval::Double(_)) {
                            oa = scalar_to_number_silent(&oa);
                            converted = true;
                        } else if !matches!(ob, Zval::Double(_)) {
                            ob = scalar_to_number_silent(&ob);
                            converted = true;
                        }
                    } else if type_lt_true(&oa) {
                        return if is_true_silent(&ob) { -1 } else { 0 };
                    } else if matches!(oa, Zval::Bool(true)) {
                        return if is_true_silent(&ob) { 0 } else { 1 };
                    } else if type_lt_true(&ob) {
                        return if is_true_silent(&oa) { 1 } else { 0 };
                    } else if matches!(ob, Zval::Bool(true)) {
                        return if is_true_silent(&oa) { 0 } else { -1 };
                    } else {
                        oa = scalar_to_number_silent(&oa);
                        ob = scalar_to_number_silent(&ob);
                        converted = true;
                    }
                } else if matches!(oa, Zval::Array(_)) {
                    return 1;
                } else if matches!(ob, Zval::Array(_)) {
                    return -1;
                } else {
                    return 1;
                }
            }
        }
    }
}

/// _zendi_convert_scalar_to_number_silent: arrays pass through untouched.
fn scalar_to_number_silent(v: &Zval) -> Zval {
    match v {
        Zval::Undef | Zval::Null => Zval::Long(0),
        Zval::Bool(b) => Zval::Long(*b as i64),
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), true) {
            Some(i) => match i.num {
                Num::Long(l) => Zval::Long(l),
                Num::Double(d) => Zval::Double(d),
            },
            None => Zval::Long(0),
        },
        other => other.clone(),
    }
}

/// zend_compare_arrays -> zend_hash_compare unordered: count first, then
/// each key of the left side must exist on the right with == values.
fn compare_arrays(a: &PhpArray, b: &PhpArray) -> i32 {
    if a.len() != b.len() {
        return if a.len() < b.len() { -1 } else { 1 };
    }
    for (k, v1) in a.iter() {
        match b.get(k) {
            None => return 1,
            Some(v2) => {
                let c = compare(v1, v2);
                if c != 0 {
                    return c;
                }
            }
        }
    }
    0
}

/// Loose equality with the engine's fast paths (smart_streq for strings,
/// native IEEE for doubles).
pub fn loose_eq(a: &Zval, b: &Zval) -> bool {
    match (a, b) {
        (Zval::Long(l), Zval::Long(r)) => l == r,
        (Zval::Double(l), Zval::Double(r)) => l == r,
        (Zval::Long(l), Zval::Double(r)) => *l as f64 == *r,
        (Zval::Double(l), Zval::Long(r)) => *l == *r as f64,
        (Zval::Str(l), Zval::Str(r)) => smart_streq(l, r),
        // Two objects are loosely equal iff they are the same instance, or share
        // the same class and every property is loosely equal (PHP object `==`).
        // For enum case singletons this reduces to identity (step 23).
        (Zval::Object(l), Zval::Object(r)) => {
            if Rc::ptr_eq(l, r) {
                return true;
            }
            let (lb, rb) = (l.borrow(), r.borrow());
            lb.class_id == rb.class_id
                && lb.props.len() == rb.props.len()
                && lb
                    .props
                    .iter()
                    .zip(rb.props.iter())
                    .all(|((k1, v1), (k2, v2))| k1 == k2 && loose_eq(v1, v2))
        }
        (Zval::Ref(c), _) => loose_eq(&c.borrow(), b),
        (_, Zval::Ref(c)) => loose_eq(a, &c.borrow()),
        _ => compare(a, b) == 0,
    }
}

/// `<` with VM double fast path (NAN-correct), compare() fallback.
pub fn smaller(a: &Zval, b: &Zval) -> bool {
    match (a, b) {
        (Zval::Long(l), Zval::Long(r)) => l < r,
        (Zval::Double(l), Zval::Double(r)) => l < r,
        (Zval::Long(l), Zval::Double(r)) => (*l as f64) < *r,
        (Zval::Double(l), Zval::Long(r)) => *l < *r as f64,
        _ => compare(a, b) < 0,
    }
}

pub fn smaller_or_equal(a: &Zval, b: &Zval) -> bool {
    match (a, b) {
        (Zval::Long(l), Zval::Long(r)) => l <= r,
        (Zval::Double(l), Zval::Double(r)) => l <= r,
        (Zval::Long(l), Zval::Double(r)) => (*l as f64) <= *r,
        (Zval::Double(l), Zval::Long(r)) => *l <= *r as f64,
        _ => compare(a, b) <= 0,
    }
}

/// zend_is_identical (zend_operators.c:2474-2510).
pub fn identical(a: &Zval, b: &Zval) -> bool {
    match (a, b) {
        (Zval::Undef | Zval::Null, Zval::Undef | Zval::Null) => true,
        (Zval::Bool(l), Zval::Bool(r)) => l == r,
        (Zval::Long(l), Zval::Long(r)) => l == r,
        (Zval::Double(l), Zval::Double(r)) => l == r, // IEEE: -0.0===0.0, NAN!==NAN
        (Zval::Str(l), Zval::Str(r)) => l.as_bytes() == r.as_bytes(),
        (Zval::Array(l), Zval::Array(r)) => {
            if Rc::ptr_eq(l, r) {
                return true;
            }
            if l.len() != r.len() {
                return false;
            }
            // Ordered: same key sequence, identical values.
            l.iter()
                .zip(r.iter())
                .all(|((k1, v1), (k2, v2))| keys_identical(k1, k2) && identical(v1, v2))
        }
        // Objects have handle identity: `$a === $b` iff they are the same
        // instance (object assignment shares the `Rc`). Enum cases are interned
        // singletons, so this also gives `E::Case === E::Case` (step 23, D-23.3).
        (Zval::Object(l), Zval::Object(r)) => Rc::ptr_eq(l, r),
        // Two resources are identical iff the same handle (`$f === $f`); each
        // `fopen` mints a unique handle so this also matches `==` by id (step 51).
        (Zval::Resource(l), Zval::Resource(r)) => Rc::ptr_eq(l, r),
        // Closures and generators are objects: identity is handle identity
        // (`$c === $c`, and `$c(...)`/fromCallable pass the same instance through).
        (Zval::Closure(l), Zval::Closure(r)) => Rc::ptr_eq(l, r),
        (Zval::Generator(l), Zval::Generator(r)) => Rc::ptr_eq(l, r),
        // Identity follows references on either side (also covers reference
        // elements inside arrays via the recursive call above).
        (Zval::Ref(c), _) => identical(&c.borrow(), b),
        (_, Zval::Ref(c)) => identical(a, &c.borrow()),
        _ => false,
    }
}

fn keys_identical(a: &Key, b: &Key) -> bool {
    match (a, b) {
        (Key::Int(x), Key::Int(y)) => x == y,
        (Key::Str(x), Key::Str(y)) => x.as_bytes() == y.as_bytes(),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Increment / decrement
// ---------------------------------------------------------------------------

pub fn increment(v: &mut Zval, diags: &mut Diags) -> Result<(), PhpError> {
    match v {
        Zval::Long(l) => {
            *v = match l.checked_add(1) {
                Some(n) => Zval::Long(n),
                None => Zval::Double(-(i64::MIN as f64)), // (double)LONG_MAX + 1
            };
        }
        Zval::Double(d) => *v = Zval::Double(*d + 1.0),
        Zval::Undef | Zval::Null => *v = Zval::Long(1),
        Zval::Str(s) => match parse_numeric(s.as_bytes()) {
            Some(Num::Long(l)) => {
                *v = match l.checked_add(1) {
                    Some(n) => Zval::Long(n),
                    None => Zval::Double(-(i64::MIN as f64)),
                };
            }
            Some(Num::Double(d)) => *v = Zval::Double(d + 1.0),
            None => {
                diags.push(Diag::Deprecated(
                    "Increment on non-numeric string is deprecated, use str_increment() instead"
                        .to_string(),
                ));
                *v = Zval::Str(increment_string(s.as_bytes()));
            }
        },
        Zval::Bool(_) => {
            diags.push(Diag::Warning(
                "Increment on type bool has no effect, this will change in the next major version of PHP"
                    .to_string(),
            ));
        }
        Zval::Array(_) => {
            return Err(PhpError::TypeError("Cannot increment array".to_string()));
        }
        Zval::Closure(_) => {
            return Err(PhpError::TypeError("Cannot increment Closure".to_string()));
        }
        Zval::Generator(_) => {
            return Err(PhpError::TypeError("Cannot increment Generator".to_string()));
        }
        Zval::Object(o) => {
            let name = String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned();
            return Err(PhpError::TypeError(format!("Cannot increment {name}")));
        }
        Zval::Resource(_) => {
            return Err(PhpError::TypeError("Cannot increment resource".to_string()));
        }
        Zval::WeakHandle(_) => {
            return Err(PhpError::TypeError("Cannot increment WeakReference".to_string()));
        }
        Zval::Ref(cell) => {
            let inner = &mut *cell.borrow_mut();
            return increment(inner, diags);
        }
    }
    Ok(())
}

/// Perl-style alphanumeric carry (zend_operators.c:2613).
fn increment_string(s: &[u8]) -> Rc<PhpStr> {
    if s.is_empty() {
        return PhpStr::from_str("1");
    }
    let mut out = s.to_vec();
    let mut pos = out.len();
    let mut carry = false;
    let mut last = 0u8; // class of last carried position
    while pos > 0 {
        pos -= 1;
        let ch = out[pos];
        match ch {
            b'a'..=b'z' => {
                last = b'a';
                if ch == b'z' {
                    out[pos] = b'a';
                    carry = true;
                } else {
                    out[pos] = ch + 1;
                    carry = false;
                }
            }
            b'A'..=b'Z' => {
                last = b'A';
                if ch == b'Z' {
                    out[pos] = b'A';
                    carry = true;
                } else {
                    out[pos] = ch + 1;
                    carry = false;
                }
            }
            b'0'..=b'9' => {
                last = b'0';
                if ch == b'9' {
                    out[pos] = b'0';
                    carry = true;
                } else {
                    out[pos] = ch + 1;
                    carry = false;
                }
            }
            _ => {
                carry = false;
                break;
            }
        }
        if !carry {
            break;
        }
    }
    if carry {
        let head = match last {
            b'0' => b'1',
            b'A' => b'A',
            _ => b'a',
        };
        out.insert(0, head);
    }
    PhpStr::new(out)
}

pub fn decrement(v: &mut Zval, diags: &mut Diags) -> Result<(), PhpError> {
    match v {
        Zval::Long(l) => {
            *v = match l.checked_sub(1) {
                Some(n) => Zval::Long(n),
                None => Zval::Double(i64::MIN as f64), // (double)LONG_MIN - 1
            };
        }
        Zval::Double(d) => *v = Zval::Double(*d - 1.0),
        Zval::Undef | Zval::Null => {
            diags.push(Diag::Warning(
                "Decrement on type null has no effect, this will change in the next major version of PHP"
                    .to_string(),
            ));
        }
        Zval::Str(s) => {
            if s.is_empty() {
                diags.push(Diag::Deprecated(
                    "Decrement on empty string is deprecated as non-numeric".to_string(),
                ));
                *v = Zval::Long(-1);
            } else {
                match parse_numeric(s.as_bytes()) {
                    Some(Num::Long(l)) => {
                        *v = match l.checked_sub(1) {
                            Some(n) => Zval::Long(n),
                            None => Zval::Double(i64::MIN as f64),
                        };
                    }
                    Some(Num::Double(d)) => *v = Zval::Double(d - 1.0),
                    None => {
                        diags.push(Diag::Deprecated(
                            "Decrement on non-numeric string has no effect and is deprecated"
                                .to_string(),
                        ));
                    }
                }
            }
        }
        Zval::Bool(_) => {
            diags.push(Diag::Warning(
                "Decrement on type bool has no effect, this will change in the next major version of PHP"
                    .to_string(),
            ));
        }
        Zval::Array(_) => {
            return Err(PhpError::TypeError("Cannot decrement array".to_string()));
        }
        Zval::Closure(_) => {
            return Err(PhpError::TypeError("Cannot decrement Closure".to_string()));
        }
        Zval::Generator(_) => {
            return Err(PhpError::TypeError("Cannot decrement Generator".to_string()));
        }
        Zval::Object(o) => {
            let name = String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned();
            return Err(PhpError::TypeError(format!("Cannot decrement {name}")));
        }
        Zval::Resource(_) => {
            return Err(PhpError::TypeError("Cannot decrement resource".to_string()));
        }
        Zval::WeakHandle(_) => {
            return Err(PhpError::TypeError("Cannot decrement WeakReference".to_string()));
        }
        Zval::Ref(cell) => {
            let inner = &mut *cell.borrow_mut();
            return decrement(inner, diags);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! Fast, oracle-independent unit tests for the type-juggling core. These pin
    //! the trickiest PHP 8.5 semantics (comparison, coercion, overflow, Perl-style
    //! string increment) as named regression guards, so a single discrepancy is
    //! localised in seconds — without the `differential` test's PHP oracle binary
    //! (which self-skips when absent). Expectations were confirmed against PHP 8.5.7.

    use super::*;

    fn d() -> Diags {
        Vec::new()
    }
    fn s(x: &str) -> Zval {
        Zval::str_from(x)
    }
    fn as_long(z: &Zval) -> i64 {
        match z {
            Zval::Long(n) => *n,
            o => panic!("expected Long, got {o:?}"),
        }
    }
    fn as_double(z: &Zval) -> f64 {
        match z {
            Zval::Double(f) => *f,
            o => panic!("expected Double, got {o:?}"),
        }
    }
    fn as_str(z: &Zval) -> String {
        match z {
            Zval::Str(s) => String::from_utf8_lossy(s.as_bytes()).into_owned(),
            o => panic!("expected Str, got {o:?}"),
        }
    }

    #[test]
    fn loose_eq_php8_semantics() {
        // The PHP 8 change: a number vs a non-numeric string compares *as strings*.
        assert!(!loose_eq(&Zval::Long(0), &s("foo"))); // 0 == "foo" -> false
        assert!(!loose_eq(&Zval::Long(0), &s(""))); // 0 == ""   -> false
        assert!(loose_eq(&s("10"), &s("1e1"))); // numeric strings compare numerically
        assert!(loose_eq(&s("1"), &Zval::Long(1)));
        assert!(loose_eq(&Zval::Null, &Zval::Bool(false)));
        assert!(!loose_eq(&s("0x1A"), &Zval::Long(26))); // no hex strings since PHP 7
    }

    #[test]
    fn compare_threeway() {
        assert_eq!(compare(&s("abc"), &s("abd")), -1);
        assert_eq!(compare(&Zval::Long(2), &Zval::Long(2)), 0);
        assert_eq!(compare(&Zval::Long(5), &Zval::Long(3)), 1);
    }

    #[test]
    fn identical_is_type_strict() {
        assert!(!identical(&Zval::Long(1), &Zval::Double(1.0))); // 1 === 1.0 -> false
        assert!(identical(&Zval::Long(1), &Zval::Long(1)));
        assert!(!identical(&s("1"), &Zval::Long(1)));
    }

    #[test]
    fn add_coercion_and_overflow() {
        assert_eq!(as_long(&add(&s("5"), &Zval::Long(3), &mut d()).unwrap()), 8);
        assert_eq!(as_double(&add(&s("5.5"), &Zval::Long(1), &mut d()).unwrap()), 6.5);
        assert_eq!(as_long(&add(&Zval::Bool(true), &Zval::Bool(true), &mut d()).unwrap()), 2);
        assert_eq!(as_long(&add(&Zval::Null, &Zval::Long(5), &mut d()).unwrap()), 5);
        // int overflow promotes the result to float
        let r = add(&Zval::Long(i64::MAX), &Zval::Long(1), &mut d()).unwrap();
        assert_eq!(as_double(&r), 9_223_372_036_854_775_808.0);
    }

    #[test]
    fn div_modulo_int_float_and_by_zero() {
        assert_eq!(as_long(&div(&Zval::Long(6), &Zval::Long(3), &mut d()).unwrap()), 2);
        assert_eq!(as_double(&div(&Zval::Long(7), &Zval::Long(2), &mut d()).unwrap()), 3.5);
        assert_eq!(as_long(&modulo(&Zval::Long(7), &Zval::Long(3), &mut d()).unwrap()), 1);
        assert!(div(&Zval::Long(1), &Zval::Long(0), &mut d()).is_err());
        assert!(modulo(&Zval::Long(1), &Zval::Long(0), &mut d()).is_err());
    }

    #[test]
    fn pow_and_concat() {
        assert_eq!(as_long(&pow(&Zval::Long(2), &Zval::Long(10), &mut d()).unwrap()), 1024);
        assert_eq!(as_str(&concat(&Zval::Long(3), &Zval::Long(4), &mut d()).unwrap()), "34");
    }

    #[test]
    fn string_increment_perl_style() {
        for (input, expected) in [("az", "ba"), ("Zz", "AAa"), ("a9", "b0"), ("Az", "Ba")] {
            let mut z = s(input);
            increment(&mut z, &mut d()).unwrap();
            assert_eq!(as_str(&z), expected, "increment({input:?})");
        }
        // A numeric string increments numerically; an empty string becomes "1".
        let mut n = s("9");
        increment(&mut n, &mut d()).unwrap();
        assert_eq!(as_long(&n), 10);
        let mut e = s("");
        increment(&mut e, &mut d()).unwrap();
        assert_eq!(as_str(&e), "1");
    }
}
