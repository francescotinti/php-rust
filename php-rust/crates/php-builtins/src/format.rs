//! sprintf/printf engine (plan step 10).
//!
//! Supported conversions: d/i, u, f/F, e/E, s, x/X, o, b, c, %%. Flags: `-`
//! (left-justify), `+` (force sign), `0` (zero pad), `'<c>` (custom pad char).
//! Width and `.precision` are supported, as is positional `%N$`. The `g`/`G`
//! conversions are intentionally out of scope (shortest-form formatting differs
//! subtly from PHP and is rarely used in the corpus).

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

/// sprintf($format, ...$args): the formatted string.
pub fn sprintf(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = first_format(args, "sprintf")?;
    let bytes = format_impl(&fmt, args)?;
    Ok(Zval::Str(PhpStr::new(bytes)))
}

/// printf($format, ...$args): writes the result and returns its byte length.
pub fn printf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = first_format(args, "printf")?;
    let bytes = format_impl(&fmt, args)?;
    let n = bytes.len();
    ctx.out.extend_from_slice(&bytes);
    Ok(Zval::Long(n as i64))
}

/// Format `$format` against an array of values (step 56c). Slot 0 of the values
/// slice is an ignored placeholder for the format itself (the engine numbers
/// conversion args from index 1), so the array elements follow it.
fn vformat(args: &[Zval], fname: &str) -> Result<Vec<u8>, PhpError> {
    let fmt = first_format(args, fname)?;
    let mut vals: Vec<Zval> = vec![Zval::Null];
    if let Some(Zval::Array(a)) = args.get(1) {
        for (_k, v) in a.iter() {
            vals.push(v.clone());
        }
    }
    format_impl(&fmt, &vals)
}

/// vsprintf($format, $args): like sprintf with the conversion args in an array.
pub fn vsprintf(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(vformat(args, "vsprintf")?)))
}

/// vprintf($format, $args): like printf with the args in an array; returns length.
pub fn vprintf(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let bytes = vformat(args, "vprintf")?;
    let n = bytes.len();
    ctx.out.extend_from_slice(&bytes);
    Ok(Zval::Long(n as i64))
}

pub(crate) fn first_format(args: &[Zval], fname: &str) -> Result<Vec<u8>, PhpError> {
    match args.first() {
        Some(v) => Ok(to_bytes(v)),
        None => Err(PhpError::Error(format!(
            "{fname}() expects at least 1 argument, 0 given"
        ))),
    }
}

fn to_bytes(v: &Zval) -> Vec<u8> {
    // Builtins never observe Array-to-string warnings here in practice; use a
    // throwaway diag sink for the rare coercion.
    let mut diags = Vec::new();
    convert::to_zstr(v, &mut diags).as_bytes().to_vec()
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
    precision: Option<usize>,
}

/// Core formatter shared by sprintf/printf.
pub(crate) fn format_impl(fmt: &[u8], args: &[Zval]) -> Result<Vec<u8>, PhpError> {
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
            break;
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

        // Width.
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

        // Precision.
        if fmt.get(i) == Some(&b'.') {
            i += 1;
            let (p, j) = read_uint(fmt, i);
            if p.unwrap_or(0) > INT_MAX {
                return Err(PhpError::ValueError(
                    "Precision must be between 0 and 2147483647".to_string(),
                ));
            }
            spec.precision = Some(p.unwrap_or(0) as usize);
            i = j;
        }

        let Some(&conv) = fmt.get(i) else { break };
        i += 1;

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
                    idx + 1,
                    args.len()
                )))
            }
        };

        let formatted = format_one(conv, arg, &spec);
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
fn format_one(conv: u8, arg: &Zval, spec: &Spec) -> Vec<u8> {
    match conv {
        b'd' | b'i' => {
            let n = convert::to_long_cast(arg, &mut Vec::new());
            let neg = n < 0;
            let mag = (n as i128).unsigned_abs().to_string().into_bytes();
            pad_numeric(neg, mag, spec)
        }
        b'u' => {
            let n = convert::to_long_cast(arg, &mut Vec::new()) as u64;
            pad_numeric(false, n.to_string().into_bytes(), spec)
        }
        b'x' | b'X' | b'o' | b'b' => {
            let n = convert::to_long_cast(arg, &mut Vec::new()) as u64;
            let body = match conv {
                b'x' => format!("{n:x}"),
                b'X' => format!("{n:X}"),
                b'o' => format!("{n:o}"),
                _ => format!("{n:b}"),
            };
            pad_numeric(false, body.into_bytes(), spec)
        }
        b'c' => {
            let n = convert::to_long_cast(arg, &mut Vec::new());
            vec![n as u8]
        }
        b'f' | b'F' => {
            let v = convert::to_double(arg);
            let prec = spec.precision.unwrap_or(6);
            let neg = v.is_sign_negative() && v != 0.0;
            let mag = format!("{:.*}", prec, v.abs()).into_bytes();
            pad_numeric(neg, mag, spec)
        }
        b'e' | b'E' => {
            let v = convert::to_double(arg);
            let prec = spec.precision.unwrap_or(6);
            let neg = v.is_sign_negative() && v != 0.0;
            let mag = format_exp(v.abs(), prec, conv == b'E');
            pad_numeric(neg, mag, spec)
        }
        b's' => {
            let mut body = to_bytes(arg);
            if let Some(p) = spec.precision {
                body.truncate(p);
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
