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
            v.error_type_name()
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
                args[0].error_type_name()
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
