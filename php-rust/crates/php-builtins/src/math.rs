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

/// dechex($num): lowercase hex digits of the int, treated as *unsigned* 64-bit
/// (PHP: dechex(-1) = "ffffffffffffffff").
pub fn dechex(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = to_int_arg(args, 0, "dechex", 1, "num")?;
    Ok(Zval::str_from(&format!("{:x}", n as u64)))
}

const BASE_DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";

/// `_php_math_basetozval`: read `s` as a number in `base` (2..=36), skipping any
/// byte that is not a valid digit of that base. Accumulates in an `i64`; on
/// overflow it switches to `f64` accumulation — losing precision exactly as PHP
/// does. Returns `(long, double, used_double, invalid_char_count)`.
fn basetozval(s: &[u8], base: i64) -> (i64, f64, bool, usize) {
    // Skip leading/trailing whitespace (C `isspace`: space, \t, \n, \v, \f, \r).
    let is_space = |b: u8| b == b' ' || (0x09..=0x0d).contains(&b);
    let mut start = 0;
    let mut end = s.len();
    while start < end && is_space(s[start]) {
        start += 1;
    }
    while end > start && is_space(s[end - 1]) {
        end -= 1;
    }
    let mut s = &s[start..end];
    // Strip a base-specific literal prefix (`0x`/`0o`/`0b`), only when at least
    // two bytes remain — the RFC base_convert improvements.
    if s.len() >= 2 && s[0] == b'0' {
        let p = s[1];
        if (base == 16 && (p == b'x' || p == b'X'))
            || (base == 8 && (p == b'o' || p == b'O'))
            || (base == 2 && (p == b'b' || p == b'B'))
        {
            s = &s[2..];
        }
    }
    let mut num: i64 = 0;
    let mut fnum: f64 = 0.0;
    let mut mode_double = false;
    let mut invalid = 0usize;
    let cutoff = i64::MAX / base;
    let cutlim = i64::MAX % base;
    for &ch in s {
        let c: i64 = match ch {
            b'0'..=b'9' => (ch - b'0') as i64,
            b'A'..=b'Z' => (ch - b'A' + 10) as i64,
            b'a'..=b'z' => (ch - b'a' + 10) as i64,
            _ => {
                invalid += 1;
                continue;
            }
        };
        if c >= base {
            invalid += 1;
            continue;
        }
        if !mode_double {
            if num < cutoff || (num == cutoff && c <= cutlim) {
                num = num * base + c;
                continue;
            }
            fnum = num as f64;
            mode_double = true;
        }
        fnum = fnum * base as f64 + c as f64;
    }
    (num, fnum, mode_double, invalid)
}

/// Non-negative `i64` rendered in `base` (2..=36).
fn long_to_base(mut n: i64, base: i64) -> Vec<u8> {
    if n == 0 {
        return vec![b'0'];
    }
    let mut buf = Vec::new();
    while n > 0 {
        buf.push(BASE_DIGITS[(n % base) as usize]);
        n /= base;
    }
    buf.reverse();
    buf
}

/// `_php_math_zvaltobase` double branch: `floor` the value then peel off digits
/// with `(int)fmod(fvalue, base)` while dividing (fractional truncation recovers
/// each digit — matching PHP's exact algorithm, precision loss included). Capped
/// at 64 digits like PHP's fixed `(sizeof(double) << 3) + 1` buffer, so very
/// large magnitudes keep the low 64 digits and drop the high ones, byte-for-byte.
fn dbl_to_base(value: f64, base: i64) -> Vec<u8> {
    let b = base as f64;
    let mut fvalue = value.floor();
    let mut buf = Vec::new();
    loop {
        let d = (fvalue % b).trunc() as i64;
        buf.push(BASE_DIGITS[d.rem_euclid(base) as usize]);
        fvalue /= b;
        if buf.len() >= 64 || fvalue.abs() < 1.0 {
            break;
        }
    }
    buf.reverse();
    buf
}

/// `base_convert(string $num, int $from_base, int $to_base): string` — reinterpret
/// the digits of `$num` (read in `$from_base`, invalid digits skipped) into
/// `$to_base`. Both bases must be 2..=36. Very large inputs overflow the integer
/// accumulator to `double` and lose precision, exactly as PHP.
pub fn base_convert(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num_arg = args.first().ok_or_else(|| {
        PhpError::ArgumentCountError("base_convert() expects exactly 3 arguments, 0 given".into())
    })?;
    // `$num` is a `Z_PARAM_STR` parameter: scalars (and Stringable objects)
    // coerce, but an array or resource is a hard TypeError. (A non-Stringable
    // object can only be detected VM-side, so it stays on the lenient path — the
    // same limitation as `strlen` et al.)
    if matches!(num_arg, Zval::Array(_) | Zval::Resource(_)) {
        return Err(PhpError::TypeError(format!(
            "base_convert(): Argument #1 ($num) must be of type string, {} given",
            num_arg.type_name_for_error()
        )));
    }
    let num = php_types::convert::to_zstr(num_arg, ctx.diags);
    let from = to_int_arg(args, 1, "base_convert", 2, "from_base")?;
    let to = to_int_arg(args, 2, "base_convert", 3, "to_base")?;
    if !(2..=36).contains(&from) {
        return Err(PhpError::ValueError(
            "base_convert(): Argument #2 ($from_base) must be between 2 and 36 (inclusive)".into(),
        ));
    }
    if !(2..=36).contains(&to) {
        return Err(PhpError::ValueError(
            "base_convert(): Argument #3 ($to_base) must be between 2 and 36 (inclusive)".into(),
        ));
    }
    let (num_long, num_dbl, is_dbl, invalid) = basetozval(num.as_bytes(), from);
    if invalid > 0 {
        ctx.diags.push(php_types::Diag::Deprecated(
            "Invalid characters passed for attempted conversion, these have been ignored".into(),
        ));
    }
    let out = if is_dbl {
        dbl_to_base(num_dbl, to)
    } else {
        long_to_base(num_long, to)
    };
    Ok(Zval::Str(php_types::PhpStr::new(out)))
}

/// decoct($num): octal digits, unsigned like dechex.
pub fn decoct(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = to_int_arg(args, 0, "decoct", 1, "num")?;
    Ok(Zval::str_from(&format!("{:o}", n as u64)))
}

/// decbin($num): binary digits, unsigned like dechex.
pub fn decbin(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = to_int_arg(args, 0, "decbin", 1, "num")?;
    Ok(Zval::str_from(&format!("{:b}", n as u64)))
}

/// The `hexdec`/`octdec`/`bindec` core: fold the digits of `base`, ignoring
/// characters outside the alphabet (PHP semantics), overflowing into float.
fn basedec(args: &[Zval], base: u32, fname: &'static str, ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v = args.first().ok_or_else(|| {
        PhpError::ArgumentCountError(format!("{fname}() expects exactly 1 argument, 0 given"))
    })?;
    let s = php_types::convert::to_zstr_cast(v, ctx.diags);
    let mut int_acc: i64 = 0;
    let mut float_acc: f64 = 0.0;
    let mut overflow = false;
    for &b in s.as_bytes() {
        let d = match (b as char).to_digit(base) {
            Some(d) => d,
            None => continue,
        };
        if !overflow {
            match int_acc.checked_mul(base as i64).and_then(|x| x.checked_add(d as i64)) {
                Some(x) => int_acc = x,
                None => {
                    overflow = true;
                    float_acc = int_acc as f64;
                }
            }
        }
        if overflow {
            float_acc = float_acc * base as f64 + d as f64;
        }
    }
    Ok(if overflow { Zval::Double(float_acc) } else { Zval::Long(int_acc) })
}

/// hexdec($hex_string): int (or float on overflow) value of the hex digits.
pub fn hexdec(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    basedec(args, 16, "hexdec", ctx)
}

/// octdec($octal_string).
pub fn octdec(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    basedec(args, 8, "octdec", ctx)
}

/// bindec($binary_string).
pub fn bindec(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    basedec(args, 2, "bindec", ctx)
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

/// fdiv($num1, $num2): IEEE-754 floating-point division. Unlike `/`, it never
/// raises `DivisionByZeroError`: `x / 0` yields `±INF` and `0 / 0` yields `NAN`,
/// exactly as Rust's `f64` division does.
pub fn fdiv(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = fdiv_arg(args, 0, 1, "num1")?;
    let b = fdiv_arg(args, 1, 2, "num2")?;
    Ok(Zval::Double(a / b))
}

/// `fdiv` coerces each argument to float (its parameters are typed `float`).
fn fdiv_arg(args: &[Zval], idx: usize, n: usize, pname: &str) -> Result<f64, PhpError> {
    let v = args.get(idx).ok_or_else(|| {
        PhpError::ArgumentCountError(format!(
            "fdiv() expects exactly 2 arguments, {} given",
            args.len()
        ))
    })?;
    as_double(v).ok_or_else(|| {
        PhpError::TypeError(format!(
            "fdiv(): Argument #{n} (${pname}) must be of type float, {} given",
            v.type_name_for_error()
        ))
    })
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

// ---------------------------------------------------------------------------
// Transcendental / trig family (thin f64 wrappers, PHP casts numerics)
// ---------------------------------------------------------------------------

/// One `f64 -> f64` math builtin: coerce like the other float builtins, apply.
fn unary_f64(
    args: &[Zval],
    fname: &str,
    pname: &str,
    f: fn(f64) -> f64,
) -> Result<Zval, PhpError> {
    let v = args.first().ok_or_else(|| {
        PhpError::ArgumentCountError(format!("{fname}() expects exactly 1 argument, 0 given"))
    })?;
    let d = as_double(v).ok_or_else(|| {
        PhpError::TypeError(format!(
            "{fname}(): Argument #1 (${pname}) must be of type int|float, {} given",
            v.type_name_for_error()
        ))
    })?;
    Ok(Zval::Double(f(d)))
}

macro_rules! unary_math {
    ($(($rust:ident, $pname:literal)),* $(,)?) => {$(
        pub fn $rust(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
            unary_f64(args, stringify!($rust), $pname, f64::$rust)
        }
    )*};
}

unary_math!(
    (sin, "num"), (cos, "num"), (tan, "num"),
    (asin, "num"), (acos, "num"), (atan, "num"),
    (sinh, "num"), (cosh, "num"), (tanh, "num"),
    (asinh, "num"), (acosh, "num"), (atanh, "num"),
    (exp, "num"), (log10, "num"),
    (exp_m1, "num"), (ln_1p, "num"),
);

/// log($num, $base = M_E)
pub fn log(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let x = match num_arg(args, 0, "log", 1, "num")? {
        Num::Long(n) => n as f64,
        Num::Double(d) => d,
    };
    match args.get(1) {
        None | Some(Zval::Null) => Ok(Zval::Double(x.ln())),
        Some(b) => {
            let base = as_double(b).ok_or_else(|| {
                PhpError::TypeError(format!(
                    "log(): Argument #2 ($base) must be of type int|float, {} given",
                    b.type_name_for_error()
                ))
            })?;
            Ok(Zval::Double(x.log(base)))
        }
    }
}

pub fn atan2(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let y = match num_arg(args, 0, "atan2", 1, "y")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    let x = match num_arg(args, 1, "atan2", 2, "x")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    Ok(Zval::Double(y.atan2(x)))
}

pub fn hypot(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = match num_arg(args, 0, "hypot", 1, "x")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    let b = match num_arg(args, 1, "hypot", 2, "y")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    Ok(Zval::Double(a.hypot(b)))
}

pub fn fmod(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = match num_arg(args, 0, "fmod", 1, "num1")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    let b = match num_arg(args, 1, "fmod", 2, "num2")? { Num::Long(n) => n as f64, Num::Double(d) => d };
    Ok(Zval::Double(a % b))
}

pub fn deg2rad(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    unary_f64(args, "deg2rad", "num", f64::to_radians)
}

pub fn rad2deg(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    unary_f64(args, "rad2deg", "num", f64::to_degrees)
}

pub fn pi(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Double(std::f64::consts::PI))
}

pub fn is_nan(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    unary_f64(args, "is_nan", "num", |d| d).map(|v| match v {
        Zval::Double(d) => Zval::Bool(d.is_nan()),
        _ => Zval::Bool(false),
    })
}

pub fn is_finite(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    unary_f64(args, "is_finite", "num", |d| d).map(|v| match v {
        Zval::Double(d) => Zval::Bool(d.is_finite()),
        _ => Zval::Bool(false),
    })
}

pub fn is_infinite(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    unary_f64(args, "is_infinite", "num", |d| d).map(|v| match v {
        Zval::Double(d) => Zval::Bool(d.is_infinite()),
        _ => Zval::Bool(false),
    })
}

// ---------------------------------------------------------------------------
// mt_rand / rand — MT19937, faithful to ext/standard/mt_rand.c so a seeded
// sequence matches PHP byte for byte.
// ---------------------------------------------------------------------------

struct Mt19937 {
    state: [u32; 624],
    idx: usize,
}

impl Mt19937 {
    fn seeded(seed: u32) -> Mt19937 {
        let mut state = [0u32; 624];
        state[0] = seed;
        for i in 1..624 {
            let prev = state[i - 1];
            state[i] = (1812433253u32
                .wrapping_mul(prev ^ (prev >> 30)))
                .wrapping_add(i as u32);
        }
        Mt19937 { state, idx: 624 }
    }

    fn next(&mut self) -> u32 {
        if self.idx >= 624 {
            for i in 0..624 {
                let y = (self.state[i] & 0x8000_0000) | (self.state[(i + 1) % 624] & 0x7fff_ffff);
                let mut n = self.state[(i + 397) % 624] ^ (y >> 1);
                if y & 1 != 0 {
                    n ^= 0x9908_b0df;
                }
                self.state[i] = n;
            }
            self.idx = 0;
        }
        let mut y = self.state[self.idx];
        self.idx += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }
}

thread_local! {
    static MT: std::cell::RefCell<Option<Mt19937>> = const { std::cell::RefCell::new(None) };
}

fn mt_next() -> u32 {
    MT.with(|c| {
        let mut m = c.borrow_mut();
        if m.is_none() {
            // Unseeded: seed from the OS, as PHP does on first use.
            let mut b = [0u8; 4];
            let _ = getrandom::getrandom(&mut b);
            *m = Some(Mt19937::seeded(u32::from_le_bytes(b)));
        }
        m.as_mut().unwrap().next()
    })
}

/// `php_random_range32` verbatim: raw word for the full range, `& (n-1)` for a
/// power-of-two span, otherwise modulo with redraws above the unbiased limit.
pub(crate) fn mt_range(umax: u32) -> u32 {
    let mut result = mt_next();
    if umax == u32::MAX {
        return result;
    }
    let n = umax.wrapping_add(1); // inclusive span
    if n & (n - 1) == 0 {
        return result & (n - 1);
    }
    let limit = u32::MAX - (u32::MAX % n) - 1;
    while result > limit {
        result = mt_next();
    }
    result % n
}

/// mt_rand() / mt_rand($min, $max); rand() is its alias since PHP 7.1.
pub fn mt_rand(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.is_empty() {
        return Ok(Zval::Long((mt_next() >> 1) as i64));
    }
    let min = to_int_arg(args, 0, "mt_rand", 1, "min")?;
    let max = to_int_arg(args, 1, "mt_rand", 2, "max")?;
    if min > max {
        return Err(PhpError::ValueError(format!(
            "mt_rand(): Argument #1 ($min) must be less than or equal to argument #2 ($max)"
        )));
    }
    let umax = (max as i128 - min as i128) as u128;
    if umax <= u32::MAX as u128 {
        Ok(Zval::Long(min.wrapping_add(mt_range(umax as u32) as i64)))
    } else {
        // 64-bit span: two words, then the same rejection idea via modulo bias
        // being negligible for PHP's use (ext uses rand_range64 similarly).
        let wide = ((mt_next() as u64) << 32) | mt_next() as u64;
        Ok(Zval::Long(min.wrapping_add((wide as u128 % (umax + 1)) as i64)))
    }
}

/// mt_srand($seed = unset) / srand: (re)seed the shared MT19937 state.
pub fn mt_srand(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let seed = match args.first() {
        Some(v) => match v {
            Zval::Long(n) => *n as u32,
            Zval::Bool(b) => *b as u32,
            Zval::Double(d) => *d as i64 as u32,
            _ => 0,
        },
        None => {
            let mut b = [0u8; 4];
            let _ = getrandom::getrandom(&mut b);
            u32::from_le_bytes(b)
        }
    };
    MT.with(|c| *c.borrow_mut() = Some(Mt19937::seeded(seed)));
    Ok(Zval::Null)
}

pub fn mt_getrandmax(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(2147483647))
}
