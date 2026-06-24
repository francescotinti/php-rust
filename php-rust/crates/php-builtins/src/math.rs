//! Math / comparison builtins (plan step 10): abs, max, min.

use php_runtime::Ctx;
use php_types::numstr::{parse_numeric_ex, Num};
use php_types::{ops, PhpError, Zval};

/// Coerce an int|float parameter: ints/floats/bools pass through, fully numeric
/// strings are parsed, everything else is `None` (caller raises a `TypeError`).
fn as_number(v: &Zval) -> Option<Num> {
    match v {
        Zval::Long(n) => Some(Num::Long(*n)),
        Zval::Double(d) => Some(Num::Double(*d)),
        Zval::Bool(b) => Some(Num::Long(*b as i64)),
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), false) {
            Some(info) if !info.trailing => Some(info.num),
            _ => None,
        },
        _ => None,
    }
}

/// Coerce an int|float parameter to `f64` for the float-returning math
/// builtins (sqrt/floor/ceil/round). `None` → caller raises a `TypeError`.
fn as_double(v: &Zval) -> Option<f64> {
    match as_number(v) {
        Some(Num::Long(n)) => Some(n as f64),
        Some(Num::Double(d)) => Some(d),
        None => None,
    }
}

/// Nth positional int|float argument, or a `TypeError` naming `$pname`.
fn num_arg(args: &[Zval], idx: usize, fname: &str, n: usize, pname: &str) -> Result<Num, PhpError> {
    let v = args.get(idx).ok_or_else(|| {
        PhpError::ArgumentCountError(format!(
            "{fname}() expects at least {} arguments, {} given",
            idx + 1,
            args.len()
        ))
    })?;
    as_number(v).ok_or_else(|| {
        PhpError::TypeError(format!(
            "{fname}(): Argument #{n} (${pname}) must be of type int|float, {} given",
            v.type_name_for_error()
        ))
    })
}

/// intdiv($num1, $num2): integer division truncated toward zero.
pub fn intdiv(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = to_int_arg(args, 0, "intdiv", 1, "num1")?;
    let b = to_int_arg(args, 1, "intdiv", 2, "num2")?;
    if b == 0 {
        return Err(PhpError::DivisionByZeroError("Division by zero"));
    }
    if a == i64::MIN && b == -1 {
        return Err(PhpError::ArithmeticError(
            "Division of PHP_INT_MIN by -1 is not an integer",
        ));
    }
    Ok(Zval::Long(a / b))
}

/// intdiv coerces its arguments to int (it has `int` parameter types).
fn to_int_arg(
    args: &[Zval],
    idx: usize,
    fname: &str,
    n: usize,
    pname: &str,
) -> Result<i64, PhpError> {
    let v = args.get(idx).ok_or_else(|| {
        PhpError::ArgumentCountError(format!(
            "{fname}() expects exactly 2 arguments, {} given",
            args.len()
        ))
    })?;
    match v {
        Zval::Long(n) => Ok(*n),
        Zval::Double(d) => Ok(*d as i64),
        Zval::Bool(b) => Ok(*b as i64),
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), false) {
            Some(info) if !info.trailing => Ok(match info.num {
                Num::Long(n) => n,
                Num::Double(d) => d as i64,
            }),
            _ => Err(PhpError::TypeError(format!(
                "{fname}(): Argument #{n} (${pname}) must be of type int, string given"
            ))),
        },
        _ => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #{n} (${pname}) must be of type int, {} given",
            v.type_name_for_error()
        ))),
    }
}

/// pow($base, $exp): an int when both operands are ints and `$exp >= 0` (with
/// overflow promoting to float), otherwise a float.
pub fn pow(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = num_arg(args, 0, "pow", 1, "num")?;
    let exp = num_arg(args, 1, "pow", 2, "exponent")?;
    if let (Num::Long(b), Num::Long(e)) = (base, exp) {
        if e >= 0 {
            // Integer exponentiation, promoting to float on overflow.
            let mut acc: i64 = 1;
            let mut overflowed = false;
            for _ in 0..e {
                match acc.checked_mul(b) {
                    Some(v) => acc = v,
                    None => {
                        overflowed = true;
                        break;
                    }
                }
            }
            if !overflowed {
                return Ok(Zval::Long(acc));
            }
        }
    }
    let b = match base {
        Num::Long(n) => n as f64,
        Num::Double(d) => d,
    };
    let e = match exp {
        Num::Long(n) => n as f64,
        Num::Double(d) => d,
    };
    Ok(Zval::Double(b.powf(e)))
}

/// sqrt($num): square root as a float (NAN for negatives).
pub fn sqrt(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(double_arg(args, "sqrt")?.sqrt()))
}

/// floor($num): round down to the nearest integer, returned as a float.
pub fn floor(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(double_arg(args, "floor")?.floor()))
}

/// ceil($num): round up to the nearest integer, returned as a float.
pub fn ceil(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(double_arg(args, "ceil")?.ceil()))
}

/// round($num[, $precision]): round half away from zero to `$precision` decimal
/// places (which may be negative), returned as a float.
pub fn round(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = double_arg(args, "round")?;
    let precision = match args.get(1) {
        Some(p) => to_int_or_zero(p),
        None => 0,
    };
    let factor = 10f64.powi(precision as i32);
    let scaled = v * factor;
    // Round half away from zero (PHP_ROUND_HALF_UP), matching PHP's default.
    let rounded = if scaled >= 0.0 {
        (scaled + 0.5).floor()
    } else {
        (scaled - 0.5).ceil()
    };
    Ok(Zval::Double(rounded / factor))
}

/// First int|float positional argument coerced to `f64`, or a `TypeError`.
fn double_arg(args: &[Zval], fname: &str) -> Result<f64, PhpError> {
    let v = args.first().ok_or_else(|| {
        PhpError::ArgumentCountError(format!("{fname}() expects exactly 1 argument, 0 given"))
    })?;
    as_double(v).ok_or_else(|| {
        PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($num) must be of type int|float, {} given",
            v.type_name_for_error()
        ))
    })
}

/// Lenient int coercion for the optional `round($precision)` argument.
fn to_int_or_zero(v: &Zval) -> i64 {
    match as_number(v) {
        Some(Num::Long(n)) => n,
        Some(Num::Double(d)) => d as i64,
        None => 0,
    }
}

/// abs($num): absolute value. abs(PHP_INT_MIN) overflows to a float.
pub fn abs(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = args.first().ok_or_else(|| {
        PhpError::Error("abs() expects exactly 1 argument, 0 given".to_string())
    })?;
    match as_number(v) {
        Some(Num::Long(n)) => {
            if n == i64::MIN {
                Ok(Zval::Double((n as f64).abs()))
            } else {
                Ok(Zval::Long(n.abs()))
            }
        }
        Some(Num::Double(d)) => Ok(Zval::Double(d.abs())),
        None => Err(PhpError::TypeError(format!(
            "abs(): Argument #1 ($num) must be of type int|float, {} given",
            v.type_name_for_error()
        ))),
    }
}

/// max(...): see [`extreme`]. Keeps the greater value, first wins on a tie.
pub fn max(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    extreme(args, "max", true)
}

/// min(...): keeps the lesser value, first wins on a tie.
pub fn min(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    extreme(args, "min", false)
}

/// Shared max/min driver. With a single array argument the extreme is taken
/// over its elements; with 2+ arguments, over the arguments themselves.
fn extreme(args: &[Zval], fname: &str, want_max: bool) -> Result<Zval, PhpError> {
    if args.is_empty() {
        return Err(PhpError::ArgumentCountError(format!(
            "{fname}() expects at least 1 argument, 0 given"
        )));
    }
    if args.len() == 1 {
        let Zval::Array(a) = &args[0] else {
            return Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($value) must be of type array, {} given",
                args[0].type_name_for_error()
            )));
        };
        if a.is_empty() {
            return Err(PhpError::ValueError(format!(
                "{fname}(): Argument #1 ($value) must contain at least one element"
            )));
        }
        return Ok(reduce(a.iter().map(|(_, v)| v), want_max));
    }
    Ok(reduce(args.iter(), want_max))
}

/// Reduce a non-empty value sequence to its extreme. `replace only if strictly
/// beyond` keeps the first element on ties (matching PHP).
fn reduce<'a>(mut it: impl Iterator<Item = &'a Zval>, want_max: bool) -> Zval {
    let mut best = it.next().expect("non-empty").clone();
    for v in it {
        let ord = ops::compare(v, &best);
        if (want_max && ord > 0) || (!want_max && ord < 0) {
            best = v.clone();
        }
    }
    best
}
