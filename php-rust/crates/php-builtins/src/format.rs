//! sprintf/printf engine (plan step 10).
//!
//! Supported conversions: d/i, u, f/F, e/E, g/G, h/H, s, x/X, o, b, c, %%.
//! Flags: `-` (left-justify), `+` (force sign), `0` (zero pad), `'<c>` (custom
//! pad char). Width and `.precision` are supported, including the PHP 8.4 `*`
//! (argument-driven) forms and positional `%N$` / `%*N$`. The `g`/`G`/`h`/`H`
//! conversions reproduce PHP's `php_gcvt` (fixed-or-scientific shortest form;
//! `h`/`H` are the locale-independent twins of `g`/`G`, identical under C locale).

use php_runtime::Ctx;
use php_types::{convert, Diags, PhpError, PhpStr, Zval};

/// sprintf($format, ...$args): the formatted string.
pub fn sprintf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = first_format(args, "sprintf", ctx.diags)?;
    let bytes = format_impl(&fmt, args, ctx.diags)?;
    Ok(Zval::Str(PhpStr::new(bytes)))
}

/// printf($format, ...$args): writes the result and returns its byte length.
pub fn printf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = first_format(args, "printf", ctx.diags)?;
    let bytes = format_impl(&fmt, args, ctx.diags)?;
    let n = bytes.len();
    ctx.out.extend_from_slice(&bytes);
    Ok(Zval::Long(n as i64))
}

/// Format `$format` against an array of values (step 56c). Slot 0 of the values
/// slice is an ignored placeholder for the format itself (the engine numbers
/// conversion args from index 1), so the array elements follow it.
fn vformat(args: &[Zval], fname: &str, diags: &mut Diags) -> Result<Vec<u8>, PhpError> {
    if args.len() != 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "{fname}() expects exactly 2 arguments, {} given",
            args.len()
        )));
    }
    let fmt = to_bytes(&args[0], diags);
    let Zval::Array(arr) = &args[1] else {
        return Err(PhpError::TypeError(format!(
            "{fname}(): Argument #2 ($values) must be of type array, {} given",
            args[1].error_type_name()
        )));
    };
    let mut vals: Vec<Zval> = vec![Zval::Null];
    for (_k, v) in arr.iter() {
        vals.push(v.clone());
    }
    format_impl(&fmt, &vals, diags)
}

/// vsprintf($format, $args): like sprintf with the conversion args in an array.
pub fn vsprintf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(vformat(args, "vsprintf", ctx.diags)?)))
}

/// vprintf($format, $args): like printf with the args in an array; returns length.
pub fn vprintf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let bytes = vformat(args, "vprintf", ctx.diags)?;
    let n = bytes.len();
    ctx.out.extend_from_slice(&bytes);
    Ok(Zval::Long(n as i64))
}

pub(crate) fn first_format(
    args: &[Zval],
    fname: &str,
    diags: &mut Diags,
) -> Result<Vec<u8>, PhpError> {
    match args.first() {
        Some(v) => Ok(to_bytes(v, diags)),
        None => Err(PhpError::ArgumentCountError(format!(
            "{fname}() expects at least 1 argument, 0 given"
        ))),
    }
}

fn to_bytes(v: &Zval, diags: &mut Diags) -> Vec<u8> {
    convert::to_zstr(v, diags).as_bytes().to_vec()
}

/// PHP caps width and precision at `INT_MAX`; beyond it a `ValueError` is thrown
/// (and, crucially, we never reach `Vec::with_capacity` with a pathological size).
const INT_MAX: u64 = 2147483647;

#[derive(Default)]
struct Spec {
    left: bool,
    plus: bool,
    pad: u8,
    width: usize,
    /// `None` = default; `Some(-1)` = shortest (only `g/G/h/H`); `Some(n>=0)` = n digits.
    precision: Option<i64>,
}

const INT_MAX_I64: i64 = 2147483647;

/// Resolve a `*` width/precision: an optional positional `N$` binds the star to
/// a specific argument, otherwise it consumes the next sequential one. The value
/// must be a real int (`field` names it for the "must be an integer" error).
fn read_star_arg(
    fmt: &[u8],
    i: &mut usize,
    args: &[Zval],
    next_arg: &mut usize,
    required: usize,
    field: &str,
) -> Result<i64, PhpError> {
    let (num, j) = read_uint(fmt, *i);
    let idx = match num {
        Some(n) if j < fmt.len() && fmt[j] == b'$' => {
            *i = j + 1;
            n as usize
        }
        _ => {
            let v = *next_arg;
            *next_arg += 1;
            v
        }
    };
    let arg = args.get(idx).ok_or_else(|| {
        PhpError::ArgumentCountError(format!(
            "{} arguments are required, {} given",
            required.max(idx + 1),
            args.len()
        ))
    })?;
    match arg {
        Zval::Long(n) => Ok(*n),
        _ => Err(PhpError::ValueError(format!("{field} must be an integer"))),
    }
}

/// Index a `*` width/precision argument while pre-scanning: an optional
/// positional `N$` binds it, otherwise it takes the next sequential slot.
fn star_index(fmt: &[u8], i: &mut usize, next_arg: &mut usize) -> usize {
    let (num, j) = read_uint(fmt, *i);
    match num {
        Some(n) if j < fmt.len() && fmt[j] == b'$' => {
            *i = j + 1;
            n as usize
        }
        _ => {
            let v = *next_arg;
            *next_arg += 1;
            v
        }
    }
}

/// Highest argument index a format references, mirroring the consumption order
/// of [`format_impl`] (positional `N$`, sequential fallback, and `*` width /
/// precision slots). PHP reports `required = this + 1` in the ArgumentCountError
/// for a short argument list — the *total* the format needs, not the index of
/// the first specifier that ran out.
fn max_arg_index(fmt: &[u8]) -> usize {
    let mut i = 0;
    let mut next_arg = 1usize;
    let mut max_idx = 0usize;
    while i < fmt.len() {
        if fmt[i] != b'%' {
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            break;
        }
        if fmt[i] == b'%' {
            i += 1;
            continue;
        }
        // Value positional `N$`.
        let mut val_pos = None;
        let save = i;
        let (num, j) = read_uint(fmt, i);
        if let Some(n) = num {
            if j < fmt.len() && fmt[j] == b'$' {
                val_pos = Some(n as usize);
                i = j + 1;
            } else {
                i = save;
            }
        }
        // Flags.
        loop {
            match fmt.get(i) {
                Some(b'-') | Some(b'+') | Some(b'0') | Some(b' ') => {}
                Some(b'\'') if i + 1 < fmt.len() => i += 1,
                _ => break,
            }
            i += 1;
        }
        // Width (`*` consumes an argument).
        if fmt.get(i) == Some(&b'*') {
            i += 1;
            max_idx = max_idx.max(star_index(fmt, &mut i, &mut next_arg));
        } else {
            let (_, j) = read_uint(fmt, i);
            i = j;
        }
        // Precision (`.*` consumes an argument).
        if fmt.get(i) == Some(&b'.') {
            i += 1;
            if fmt.get(i) == Some(&b'*') {
                i += 1;
                max_idx = max_idx.max(star_index(fmt, &mut i, &mut next_arg));
            } else {
                let (_, j) = read_uint(fmt, i);
                i = j;
            }
        }
        if fmt.get(i) == Some(&b'l') {
            i += 1;
        }
        if i < fmt.len() {
            i += 1; // conversion char
        }
        let vidx = val_pos.unwrap_or_else(|| {
            let v = next_arg;
            next_arg += 1;
            v
        });
        max_idx = max_idx.max(vidx);
    }
    max_idx
}

/// Core formatter shared by sprintf/printf.
pub(crate) fn format_impl(
    fmt: &[u8],
    args: &[Zval],
    diags: &mut Diags,
) -> Result<Vec<u8>, PhpError> {
    let required = max_arg_index(fmt) + 1;
    let mut out = Vec::with_capacity(fmt.len());
    let mut i = 0;
    let mut next_arg = 1usize; // args[0] is the format itself.

    while i < fmt.len() {
        if fmt[i] != b'%' {
            out.push(fmt[i]);
            i += 1;
            continue;
        }
        i += 1;
        if i >= fmt.len() {
            // A trailing `%` with nothing after it (PHP: catchable ValueError).
            return Err(PhpError::ValueError(
                "Missing format specifier at end of string".to_string(),
            ));
        }
        if fmt[i] == b'%' {
            out.push(b'%');
            i += 1;
            continue;
        }

        // Optional positional argnum: digits followed by '$'.
        let mut arg_idx = None;
        let save = i;
        let (num, j) = read_uint(fmt, i);
        if let Some(n) = num {
            if j < fmt.len() && fmt[j] == b'$' {
                arg_idx = Some(n);
                i = j + 1;
            } else {
                i = save;
            }
        }

        // Flags.
        let mut spec = Spec {
            pad: b' ',
            ..Spec::default()
        };
        loop {
            match fmt.get(i) {
                Some(b'-') => spec.left = true,
                Some(b'+') => spec.plus = true,
                Some(b'0') => spec.pad = b'0',
                Some(b' ') => {} // PHP ignores the space flag
                Some(b'\'') => {
                    if let Some(&c) = fmt.get(i + 1) {
                        spec.pad = c;
                        i += 1;
                    }
                }
                _ => break,
            }
            i += 1;
        }

        // Width: `*` (from an argument) or literal digits.
        if fmt.get(i) == Some(&b'*') {
            i += 1;
            let wv = read_star_arg(fmt, &mut i, args, &mut next_arg, required, "Width")?;
            if !(0..=INT_MAX_I64).contains(&wv) {
                return Err(PhpError::ValueError(
                    "Width must be between 0 and 2147483647".to_string(),
                ));
            }
            spec.width = wv as usize;
        } else {
            let (w, j) = read_uint(fmt, i);
            if let Some(w) = w {
                if w > INT_MAX {
                    return Err(PhpError::ValueError(
                        "Width must be between 0 and 2147483647".to_string(),
                    ));
                }
                spec.width = w as usize;
                i = j;
            }
        }

        // Precision: `*` (from an argument, may be -1 = shortest) or literal digits.
        if fmt.get(i) == Some(&b'.') {
            i += 1;
            if fmt.get(i) == Some(&b'*') {
                i += 1;
                let pv = read_star_arg(fmt, &mut i, args, &mut next_arg, required, "Precision")?;
                if !(-1..=INT_MAX_I64).contains(&pv) {
                    return Err(PhpError::ValueError(
                        "Precision must be between -1 and 2147483647".to_string(),
                    ));
                }
                spec.precision = Some(pv);
            } else {
                let (p, j) = read_uint(fmt, i);
                if p.unwrap_or(0) > INT_MAX {
                    return Err(PhpError::ValueError(
                        "Precision must be between 0 and 2147483647".to_string(),
                    ));
                }
                spec.precision = Some(p.unwrap_or(0) as i64);
                i = j;
            }
        }

        // A single `l` length modifier is accepted and ignored (PHP: `%ld` == `%d`).
        if fmt.get(i) == Some(&b'l') {
            i += 1;
        }

        let Some(&conv) = fmt.get(i) else {
            return Err(PhpError::ValueError(
                "Missing format specifier at end of string".to_string(),
            ));
        };
        i += 1;

        // Reject anything that is not a known conversion (PHP throws a catchable
        // ValueError, rather than silently emitting the character).
        if !matches!(
            conv,
            b'd' | b'i'
                | b'u'
                | b'f'
                | b'F'
                | b'e'
                | b'E'
                | b'g'
                | b'G'
                | b'h'
                | b'H'
                | b's'
                | b'x'
                | b'X'
                | b'o'
                | b'b'
                | b'c'
        ) {
            return Err(PhpError::ValueError(format!(
                "Unknown format specifier \"{}\"",
                conv as char
            )));
        }

        // A `-1` precision (shortest) is only meaningful for the g/G/h/H family.
        if spec.precision == Some(-1) && !matches!(conv, b'g' | b'G' | b'h' | b'H') {
            return Err(PhpError::ValueError(
                "Precision -1 is only supported for %g, %G, %h and %H".to_string(),
            ));
        }

        // Resolve the argument for this directive.
        let idx = arg_idx.map(|n| n as usize).unwrap_or_else(|| {
            let v = next_arg;
            next_arg += 1;
            v
        });
        let arg = match args.get(idx) {
            Some(v) => v,
            None => {
                return Err(PhpError::ArgumentCountError(format!(
                    "{} arguments are required, {} given",
                    required.max(idx + 1),
                    args.len()
                )))
            }
        };

        let formatted = format_one(conv, arg, &spec, diags);
        out.extend_from_slice(&formatted);
    }
    Ok(out)
}

/// Read a run of ASCII digits at `pos`; returns (value?, next_index).
fn read_uint(fmt: &[u8], pos: usize) -> (Option<u64>, usize) {
    let mut j = pos;
    let mut n: u64 = 0;
    while j < fmt.len() && fmt[j].is_ascii_digit() {
        n = n.saturating_mul(10).saturating_add((fmt[j] - b'0') as u64);
        j += 1;
    }
    if j == pos {
        (None, pos)
    } else {
        (Some(n), j)
    }
}

/// Format one resolved argument for conversion char `conv`.
fn format_one(conv: u8, arg: &Zval, spec: &Spec, diags: &mut Diags) -> Vec<u8> {
    match conv {
        b'd' | b'i' => {
            let n = convert::to_long_cast(arg, diags);
            let neg = n < 0;
            let mag = (n as i128).unsigned_abs().to_string().into_bytes();
            pad_numeric(neg, mag, spec)
        }
        b'u' => {
            let n = convert::to_long_cast(arg, diags) as u64;
            pad_numeric(false, n.to_string().into_bytes(), spec)
        }
        b'x' | b'X' | b'o' | b'b' => {
            let n = convert::to_long_cast(arg, diags) as u64;
            let body = match conv {
                b'x' => format!("{n:x}"),
                b'X' => format!("{n:X}"),
                b'o' => format!("{n:o}"),
                _ => format!("{n:b}"),
            };
            pad_numeric(false, body.into_bytes(), spec)
        }
        b'c' => {
            let n = convert::to_long_cast(arg, diags);
            vec![n as u8]
        }
        b'f' | b'F' => {
            let v = convert::to_double(arg);
            let prec = spec.precision.unwrap_or(6).max(0) as usize;
            let neg = v.is_sign_negative() && v != 0.0;
            let mag = format!("{:.*}", prec, v.abs()).into_bytes();
            pad_numeric(neg, mag, spec)
        }
        b'e' | b'E' => {
            let v = convert::to_double(arg);
            let prec = spec.precision.unwrap_or(6).max(0) as usize;
            let neg = v.is_sign_negative() && v != 0.0;
            let mag = format_exp(v.abs(), prec, conv == b'E');
            pad_numeric(neg, mag, spec)
        }
        b'g' | b'G' | b'h' | b'H' => {
            let v = convert::to_double(arg);
            if v.is_nan() {
                return pad_plain(b"NaN".to_vec(), spec);
            }
            if v.is_infinite() {
                let body = if v < 0.0 { &b"-INF"[..] } else { &b"INF"[..] };
                return pad_plain(body.to_vec(), spec);
            }
            let upper = matches!(conv, b'G' | b'H');
            let prec = spec.precision.unwrap_or(6);
            let neg = v.is_sign_negative();
            pad_numeric(neg, php_gcvt(v.abs(), prec, upper), spec)
        }
        b's' => {
            let mut body = to_bytes(arg, diags);
            if let Some(p) = spec.precision {
                body.truncate(p.max(0) as usize);
            }
            pad_plain(body, spec)
        }
        // Unknown conversion: emit nothing (the directive is consumed).
        _ => Vec::new(),
    }
}

/// PHP exponential form: `1.234568e+4` — always a sign, no leading zeros.
fn format_exp(mag: f64, prec: usize, upper: bool) -> Vec<u8> {
    let raw = format!("{mag:.prec$e}"); // e.g. "1.234568e4" / "1.2e-3"
    let (mantissa, exp) = match raw.split_once('e') {
        Some((m, e)) => (m, e),
        None => (raw.as_str(), "0"),
    };
    let exp_num: i64 = exp.parse().unwrap_or(0);
    let e = if upper { 'E' } else { 'e' };
    let sign = if exp_num < 0 { '-' } else { '+' };
    format!("{mantissa}{e}{sign}{}", exp_num.abs()).into_bytes()
}

/// Significant digits + decimal-point position of `mag > 0` (finite). `digits`
/// carries no point and has trailing zeros stripped; `decpt` is the number of
/// digits before the point (may be `<= 0`). `precision == -1` → shortest
/// round-trip, else exactly `max(precision, 1)` significant digits (rounded
/// half-to-even by Rust's float formatting, matching PHP's dtoa).
fn sig_digits(mag: f64, precision: i64) -> (Vec<u8>, i64) {
    let raw = if precision == -1 {
        format!("{mag:e}")
    } else {
        format!("{:.*e}", precision.max(1) as usize - 1, mag)
    };
    let (mantissa, exp) = raw.split_once('e').unwrap_or((raw.as_str(), "0"));
    let exp: i64 = exp.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa.bytes().filter(|&b| b != b'.').collect();
    while digits.len() > 1 && digits.last() == Some(&b'0') {
        digits.pop();
    }
    (digits, exp + 1)
}

/// PHP's `%g`/`%G`/`%h`/`%H` magnitude formatting (`php_gcvt`): pick fixed or
/// scientific by `decpt < -3 || decpt > P`, strip trailing zeros, and in
/// scientific form keep a single leading digit plus at least one fractional
/// digit (`1.0e+6`), with a signed exponent free of leading zeros.
fn php_gcvt(mag: f64, precision: i64, upper: bool) -> Vec<u8> {
    if mag == 0.0 {
        return vec![b'0'];
    }
    let (digits, decpt) = sig_digits(mag, precision);
    let p_thresh = if precision == -1 { 17 } else { precision.max(1) };
    if decpt < -3 || decpt > p_thresh {
        let mut out = vec![digits[0], b'.'];
        if digits.len() > 1 {
            out.extend_from_slice(&digits[1..]);
        } else {
            out.push(b'0');
        }
        out.push(if upper { b'E' } else { b'e' });
        let exp_out = decpt - 1;
        out.push(if exp_out < 0 { b'-' } else { b'+' });
        out.extend_from_slice(exp_out.abs().to_string().as_bytes());
        out
    } else if decpt <= 0 {
        let mut out = b"0.".to_vec();
        out.resize(out.len() + (-decpt) as usize, b'0');
        out.extend_from_slice(&digits);
        out
    } else if decpt as usize >= digits.len() {
        let mut out = digits.clone();
        out.resize(out.len() + (decpt as usize - digits.len()), b'0');
        out
    } else {
        let d = decpt as usize;
        let mut out = digits[..d].to_vec();
        out.push(b'.');
        out.extend_from_slice(&digits[d..]);
        out
    }
}

/// Pad a signed numeric body honoring sign/zero/left/width flags.
fn pad_numeric(neg: bool, mag: Vec<u8>, spec: &Spec) -> Vec<u8> {
    let sign: &[u8] = if neg {
        b"-"
    } else if spec.plus {
        b"+"
    } else {
        b""
    };
    let content_len = sign.len() + mag.len();
    if content_len >= spec.width {
        let mut out = sign.to_vec();
        out.extend_from_slice(&mag);
        return out;
    }
    let pad = spec.width - content_len;
    let mut out = Vec::with_capacity(spec.width);
    if spec.left {
        out.extend_from_slice(sign);
        out.extend_from_slice(&mag);
        out.resize(out.len() + pad, b' ');
    } else if spec.pad == b'0' {
        // Zeros go between the sign and the digits.
        out.extend_from_slice(sign);
        out.resize(out.len() + pad, b'0');
        out.extend_from_slice(&mag);
    } else {
        out.resize(pad, spec.pad);
        out.extend_from_slice(sign);
        out.extend_from_slice(&mag);
    }
    out
}

/// Pad a plain (signless) body — used for %s.
fn pad_plain(body: Vec<u8>, spec: &Spec) -> Vec<u8> {
    if body.len() >= spec.width {
        return body;
    }
    let pad = spec.width - body.len();
    let mut out = Vec::with_capacity(spec.width);
    if spec.left {
        out.extend_from_slice(&body);
        out.resize(out.len() + pad, spec.pad);
    } else {
        out.resize(pad, spec.pad);
        out.extend_from_slice(&body);
    }
    out
}
