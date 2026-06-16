//! Multibyte string builtins (`mb_*`), batch 1 — UTF-8 code-point functions.
//!
//! The `PhpStr` payload is bytes; these functions interpret it as UTF-8 and
//! operate on *code points*. Batch 1 supports the UTF-8 encoding only (plus its
//! aliases); any other `$encoding` argument is reported as an invalid encoding
//! (see `diary/NEXT-mbstring.md`, D-MB1). See the design pass for scope.

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval, ZStr};

/// Fetch the first (`$string`) argument, string-coerced. A missing argument is
/// the shared arity `Error`.
fn arg_str(args: &[Zval], func: &str, ctx: &mut Ctx) -> Result<ZStr, PhpError> {
    let v = args
        .first()
        .ok_or_else(|| PhpError::Error(format!("{func}() expects at least 1 argument, 0 given")))?;
    Ok(convert::to_zstr(v, ctx.diags))
}

/// True for the encoding names batch 1 treats as UTF-8 (ASCII is a subset).
fn is_utf8_alias(b: &[u8]) -> bool {
    b.eq_ignore_ascii_case(b"UTF-8")
        || b.eq_ignore_ascii_case(b"UTF8")
        || b.eq_ignore_ascii_case(b"US-ASCII")
        || b.eq_ignore_ascii_case(b"ASCII")
}

/// Validate an optional trailing `$encoding` argument at `idx`. Absent or null
/// means the internal encoding (UTF-8). A non-UTF-8 name raises the same
/// `ValueError` PHP raises for an unknown encoding (D-MB1: valid-but-non-UTF-8
/// encodings are reported as invalid until a transcoding batch lands).
fn require_utf8(
    args: &[Zval],
    idx: usize,
    ctx: &mut Ctx,
    func: &str,
    arg_num: usize,
    arg_name: &str,
) -> Result<(), PhpError> {
    match args.get(idx) {
        None | Some(Zval::Null) => Ok(()),
        Some(v) => {
            let enc = convert::to_zstr(v, ctx.diags);
            if is_utf8_alias(enc.as_bytes()) {
                Ok(())
            } else {
                Err(PhpError::ValueError(format!(
                    "{func}(): Argument #{arg_num} (${arg_name}) must be a valid encoding, \"{}\" given",
                    String::from_utf8_lossy(enc.as_bytes())
                )))
            }
        }
    }
}

/// Walk `bytes` as UTF-8 code points: each maximal valid scalar is one unit, and
/// each invalid byte is one unit (matching `mb_*`'s code-point counting, e.g.
/// `mb_strlen("a\xFF\xFEb") == 4`). Returns `(byte_start, byte_len)` per unit.
fn units(bytes: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let len = scalar_len(bytes, i);
        out.push((i, len));
        i += len;
    }
    out
}

/// Byte length of the UTF-8 scalar starting at `i`, or 1 for an invalid lead /
/// truncated / bad-continuation byte (lenient decode, see [`units`]).
fn scalar_len(bytes: &[u8], i: usize) -> usize {
    let b = bytes[i];
    let n = if b < 0x80 {
        return 1;
    } else if b >> 5 == 0b110 {
        2
    } else if b >> 4 == 0b1110 {
        3
    } else if b >> 3 == 0b11110 {
        4
    } else {
        return 1;
    };
    if i + n > bytes.len() {
        return 1;
    }
    for k in 1..n {
        if bytes[i + k] & 0xC0 != 0x80 {
            return 1;
        }
    }
    n
}

/// mb_strlen($string[, $encoding]): number of code points.
pub fn mb_strlen(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("mb_strlen() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    require_utf8(args, 1, ctx, "mb_strlen", 2, "encoding")?;
    Ok(Zval::Long(units(s.as_bytes()).len() as i64))
}

/// mb_substr($string, $start[, $length[, $encoding]]): substring by code point.
/// Negative `$start`/`$length` count from the end; an omitted/null length runs
/// to the end. Mirrors `substr` but on code-point units.
pub fn mb_substr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("mb_substr() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let offset = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_substr() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    require_utf8(args, 3, ctx, "mb_substr", 4, "encoding")?;

    let bytes = s.as_bytes();
    let u = units(bytes);
    let total = u.len() as i64;
    let start = if offset < 0 {
        (total + offset).max(0)
    } else {
        offset.min(total)
    };
    let end = match args.get(2) {
        None | Some(Zval::Null) => total,
        Some(v) => {
            let length = convert::to_long_cast(v, ctx.diags);
            if length < 0 {
                (total + length).max(start)
            } else {
                (start + length).min(total)
            }
        }
    };
    let window: &[u8] = if end > start {
        let byte_start = u[start as usize].0;
        let last = u[(end - 1) as usize];
        &bytes[byte_start..last.0 + last.1]
    } else {
        &[]
    };
    Ok(Zval::Str(PhpStr::new(window.to_vec())))
}

/// Decode one code-point unit (`byte_start`, `byte_len` from [`units`]) to its
/// `char`, or `None` for an invalid byte that should be copied verbatim.
fn unit_char(bytes: &[u8], start: usize, len: usize) -> Option<char> {
    std::str::from_utf8(&bytes[start..start + len])
        .ok()
        .and_then(|s| s.chars().next())
}

/// Map each code point of `bytes` through `f` (returning the replacement
/// string), copying invalid bytes verbatim. Used for the case transforms.
fn map_chars(bytes: &[u8], f: impl Fn(char) -> String) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for (start, len) in units(bytes) {
        match unit_char(bytes, start, len) {
            Some(c) => out.extend_from_slice(f(c).as_bytes()),
            None => out.extend_from_slice(&bytes[start..start + len]),
        }
    }
    out
}

/// mb_strtoupper($string[, $encoding]): full Unicode upper-casing.
pub fn mb_strtoupper(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_strtoupper", ctx)?;
    require_utf8(args, 1, ctx, "mb_strtoupper", 2, "encoding")?;
    Ok(Zval::Str(PhpStr::new(map_chars(s.as_bytes(), |c| {
        c.to_uppercase().to_string()
    }))))
}

/// mb_strtolower($string[, $encoding]): full Unicode lower-casing.
pub fn mb_strtolower(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_strtolower", ctx)?;
    require_utf8(args, 1, ctx, "mb_strtolower", 2, "encoding")?;
    Ok(Zval::Str(PhpStr::new(map_chars(s.as_bytes(), |c| {
        c.to_lowercase().to_string()
    }))))
}

/// mb_convert_case($string, $mode[, $encoding]): UPPER(0)/LOWER(1)/TITLE(2)/
/// FOLD(3) plus the `*_SIMPLE` aliases (4-7, mapped to the full forms — D-MB3).
/// TITLE upper-cases the first letter of each word (boundary = a non-letter)
/// and lower-cases the rest. An out-of-range mode is a `ValueError`.
pub fn mb_convert_case(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_convert_case", ctx)?;
    let mode = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_convert_case() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    require_utf8(args, 2, ctx, "mb_convert_case", 3, "encoding")?;
    let bytes = s.as_bytes();
    let out = match mode {
        0 | 4 => map_chars(bytes, |c| c.to_uppercase().to_string()),
        1 | 5 => map_chars(bytes, |c| c.to_lowercase().to_string()),
        // FOLD: approximated by full lower-casing (D-MB3).
        3 | 7 => map_chars(bytes, |c| c.to_lowercase().to_string()),
        2 | 6 => {
            let mut out = Vec::with_capacity(bytes.len());
            let mut in_word = false;
            for (start, len) in units(bytes) {
                match unit_char(bytes, start, len) {
                    // A cased letter: title-case the word's first, lower the rest.
                    // (Divergence: Unicode Case_Ignorable chars such as `'` are
                    // treated as boundaries here — see diary D-MB3.)
                    Some(c) if c.is_alphabetic() => {
                        let mapped = if in_word {
                            c.to_lowercase().to_string()
                        } else {
                            c.to_uppercase().to_string()
                        };
                        out.extend_from_slice(mapped.as_bytes());
                        in_word = true;
                    }
                    Some(_) | None => {
                        out.extend_from_slice(&bytes[start..start + len]);
                        in_word = false;
                    }
                }
            }
            out
        }
        _ => {
            return Err(PhpError::ValueError(
                "mb_convert_case(): Argument #2 ($mode) must be one of the MB_CASE_* constants"
                    .to_string(),
            ))
        }
    };
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Transform only the first code point of `bytes` through `f`, copying the rest
/// verbatim (shared by mb_ucfirst / mb_lcfirst).
fn map_first(bytes: &[u8], f: impl Fn(char) -> String) -> Vec<u8> {
    let u = units(bytes);
    let Some(&(_, first_len)) = u.first() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(bytes.len());
    match unit_char(bytes, 0, first_len) {
        Some(c) => out.extend_from_slice(f(c).as_bytes()),
        None => out.extend_from_slice(&bytes[0..first_len]),
    }
    out.extend_from_slice(&bytes[first_len..]);
    out
}

/// mb_ucfirst($string[, $encoding]): upper-case the first code point (PHP 8.4).
pub fn mb_ucfirst(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_ucfirst", ctx)?;
    require_utf8(args, 1, ctx, "mb_ucfirst", 2, "encoding")?;
    Ok(Zval::Str(PhpStr::new(map_first(s.as_bytes(), |c| {
        c.to_uppercase().to_string()
    }))))
}

/// mb_lcfirst($string[, $encoding]): lower-case the first code point (PHP 8.4).
pub fn mb_lcfirst(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_lcfirst", ctx)?;
    require_utf8(args, 1, ctx, "mb_lcfirst", 2, "encoding")?;
    Ok(Zval::Str(PhpStr::new(map_first(s.as_bytes(), |c| {
        c.to_lowercase().to_string()
    }))))
}

/// mb_str_split($string[, $length[, $encoding]]): array of code-point chunks of
/// `$length` (default 1). A non-positive length is a `ValueError`.
pub fn mb_str_split(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("mb_str_split() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let chunk = match args.get(1) {
        None | Some(Zval::Null) => 1,
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    if chunk < 1 {
        return Err(PhpError::ValueError(
            "mb_str_split(): Argument #2 ($length) must be greater than 0".to_string(),
        ));
    }
    require_utf8(args, 2, ctx, "mb_str_split", 3, "encoding")?;

    let bytes = s.as_bytes();
    let u = units(bytes);
    let mut arr = PhpArray::new();
    for group in u.chunks(chunk as usize) {
        let start = group[0].0;
        let last = group[group.len() - 1];
        let _ = arr.append(Zval::Str(PhpStr::new(bytes[start..last.0 + last.1].to_vec())));
    }
    Ok(Zval::Array(std::rc::Rc::new(arr)))
}

// --- search helpers (code-point indexed) ---

/// Code points of `bytes` as `(char, byte_start, byte_len)`; an invalid byte
/// becomes one U+FFFD unit (so indices line up with `mb_strlen`).
fn cps(bytes: &[u8]) -> Vec<(char, usize, usize)> {
    units(bytes)
        .into_iter()
        .map(|(s, l)| (unit_char(bytes, s, l).unwrap_or('\u{FFFD}'), s, l))
        .collect()
}

/// Simple case-fold of one code point (first char of its lowercase). Full
/// length-changing folds (İ, final sigma) are approximated — see D-MB3.
fn fold(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

fn matches_at(hay: &[char], needle: &[char], i: usize, ci: bool) -> bool {
    needle.iter().enumerate().all(|(k, &nc)| {
        let hc = hay[i + k];
        if ci {
            fold(hc) == fold(nc)
        } else {
            hc == nc
        }
    })
}

/// First index `>= from` where `needle` occurs in `hay`. Empty needle matches
/// at `from` (clamped).
fn find(hay: &[char], needle: &[char], from: usize, ci: bool) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(from.min(hay.len()));
    }
    if n > hay.len() {
        return None;
    }
    (from..=hay.len() - n).find(|&i| matches_at(hay, needle, i, ci))
}

/// Last index where `needle` occurs in `hay`.
fn rfind(hay: &[char], needle: &[char], ci: bool) -> Option<usize> {
    let n = needle.len();
    if n == 0 {
        return Some(hay.len());
    }
    if n > hay.len() {
        return None;
    }
    (0..=hay.len() - n).rev().find(|&i| matches_at(hay, needle, i, ci))
}

/// Fetch the `$haystack` (#1) and `$needle` (#2) arguments, string-coerced.
fn two_strs(args: &[Zval], func: &str, ctx: &mut Ctx) -> Result<(ZStr, ZStr), PhpError> {
    let hay = arg_str(args, func, ctx)?;
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error(format!("{func}() expects at least 2 arguments, 1 given"))
        })?,
        ctx.diags,
    );
    Ok((hay, needle))
}

/// Shared body of the `mb_str(r)(i)pos` family: code-point index of an
/// occurrence, or `false`. `reverse` finds the last; `ci` folds case.
fn strpos_impl(
    args: &[Zval],
    ctx: &mut Ctx,
    func: &str,
    ci: bool,
    reverse: bool,
) -> Result<Zval, PhpError> {
    let (hay, needle) = two_strs(args, func, ctx)?;
    let offset = match args.get(2) {
        None | Some(Zval::Null) => 0,
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    require_utf8(args, 3, ctx, func, 4, "encoding")?;
    let hchars: Vec<char> = cps(hay.as_bytes()).into_iter().map(|x| x.0).collect();
    let ndl: Vec<char> = cps(needle.as_bytes()).into_iter().map(|x| x.0).collect();
    let total = hchars.len() as i64;
    let found = if reverse {
        // Offset on the reverse search is out of scope (D-MB, batch 1): the
        // default whole-string last-occurrence is what the corpus needs.
        rfind(&hchars, &ndl, ci)
    } else {
        let from = if offset < 0 {
            (total + offset).max(0)
        } else {
            offset.min(total)
        } as usize;
        find(&hchars, &ndl, from, ci)
    };
    Ok(match found {
        Some(i) => Zval::Long(i as i64),
        None => Zval::Bool(false),
    })
}

/// mb_strpos / mb_stripos / mb_strrpos / mb_strripos.
pub fn mb_strpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, "mb_strpos", false, false)
}
pub fn mb_stripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, "mb_stripos", true, false)
}
pub fn mb_strrpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, "mb_strrpos", false, true)
}
pub fn mb_strripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, "mb_strripos", true, true)
}

/// Shared body of the `mb_str(r)(i)str`/`chr` family: the slice of `$haystack`
/// from the first (or last, if `reverse`) occurrence of `$needle` to the end,
/// or before it when `$before_needle` is set; `false` if not found.
fn strstr_impl(
    args: &[Zval],
    ctx: &mut Ctx,
    func: &str,
    ci: bool,
    reverse: bool,
) -> Result<Zval, PhpError> {
    let (hay, needle) = two_strs(args, func, ctx)?;
    let before = match args.get(2) {
        None | Some(Zval::Null) => false,
        Some(v) => convert::to_bool(v, ctx.diags),
    };
    require_utf8(args, 3, ctx, func, 4, "encoding")?;
    let hay_cps = cps(hay.as_bytes());
    let hchars: Vec<char> = hay_cps.iter().map(|x| x.0).collect();
    let ndl: Vec<char> = cps(needle.as_bytes()).into_iter().map(|x| x.0).collect();
    let idx = if reverse {
        rfind(&hchars, &ndl, ci)
    } else {
        find(&hchars, &ndl, 0, ci)
    };
    let Some(i) = idx else {
        return Ok(Zval::Bool(false));
    };
    let bytes = hay.as_bytes();
    let byte_at = hay_cps.get(i).map(|x| x.1).unwrap_or(bytes.len());
    let slice = if before {
        &bytes[..byte_at]
    } else {
        &bytes[byte_at..]
    };
    Ok(Zval::Str(PhpStr::new(slice.to_vec())))
}

/// mb_strstr / mb_stristr / mb_strrchr / mb_strrichr.
pub fn mb_strstr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, "mb_strstr", false, false)
}
pub fn mb_stristr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, "mb_stristr", true, false)
}
pub fn mb_strrchr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, "mb_strrchr", false, true)
}
pub fn mb_strrichr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, "mb_strrichr", true, true)
}

/// mb_substr_count($haystack, $needle[, $encoding]): non-overlapping count. An
/// empty needle is a `ValueError`.
pub fn mb_substr_count(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (hay, needle) = two_strs(args, "mb_substr_count", ctx)?;
    require_utf8(args, 2, ctx, "mb_substr_count", 3, "encoding")?;
    let hchars: Vec<char> = cps(hay.as_bytes()).into_iter().map(|x| x.0).collect();
    let ndl: Vec<char> = cps(needle.as_bytes()).into_iter().map(|x| x.0).collect();
    if ndl.is_empty() {
        return Err(PhpError::ValueError(
            "mb_substr_count(): Argument #2 ($needle) must not be empty".to_string(),
        ));
    }
    let mut count = 0i64;
    let mut from = 0usize;
    while let Some(i) = find(&hchars, &ndl, from, false) {
        count += 1;
        from = i + ndl.len();
    }
    Ok(Zval::Long(count))
}

// --- mb-4: ord / chr / str_pad / trim / check_encoding ---

/// mb_ord($string[, $encoding]): code point of the first character; empty input
/// is a `ValueError`. An invalid leading byte yields `false`.
pub fn mb_ord(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_ord", ctx)?;
    require_utf8(args, 1, ctx, "mb_ord", 2, "encoding")?;
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return Err(PhpError::ValueError(
            "mb_ord(): Argument #1 ($string) must not be empty".to_string(),
        ));
    }
    let (start, len) = units(bytes)[0];
    Ok(match unit_char(bytes, start, len) {
        Some(c) => Zval::Long(c as i64),
        None => Zval::Bool(false),
    })
}

/// mb_chr($codepoint[, $encoding]): the character for `$codepoint`, or `false`
/// for a negative / out-of-range / surrogate code point.
pub fn mb_chr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let cp = convert::to_long_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("mb_chr() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    require_utf8(args, 1, ctx, "mb_chr", 2, "encoding")?;
    if !(0..=0x10FFFF).contains(&cp) {
        return Ok(Zval::Bool(false));
    }
    Ok(match char::from_u32(cp as u32) {
        Some(c) => Zval::Str(PhpStr::new(c.to_string().into_bytes())),
        None => Zval::Bool(false),
    })
}

/// Build `count` code points by cycling the pad string's units.
fn cycle_pad(pad: &[u8], pad_units: &[(usize, usize)], count: usize) -> Vec<u8> {
    let mut out = Vec::new();
    for k in 0..count {
        let (s, l) = pad_units[k % pad_units.len()];
        out.extend_from_slice(&pad[s..s + l]);
    }
    out
}

/// mb_str_pad($string, $length[, $pad=" "[, $type=STR_PAD_RIGHT[, $encoding]]]):
/// pad to `$length` code points (LEFT=0 / RIGHT=1 / BOTH=2). An empty pad string
/// is a `ValueError`. No-op when already long enough.
pub fn mb_str_pad(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_str_pad", ctx)?;
    let length = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_str_pad() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let pad: Vec<u8> = match args.get(2) {
        None | Some(Zval::Null) => b" ".to_vec(),
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
    };
    let pad_type = match args.get(3) {
        None | Some(Zval::Null) => 1,
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    require_utf8(args, 4, ctx, "mb_str_pad", 5, "encoding")?;

    let bytes = s.as_bytes();
    let cur_len = units(bytes).len() as i64;
    if length <= cur_len {
        return Ok(Zval::Str(PhpStr::new(bytes.to_vec())));
    }
    let pad_units = units(&pad);
    if pad_units.is_empty() {
        return Err(PhpError::ValueError(
            "mb_str_pad(): Argument #3 ($pad_string) must be a non-empty string".to_string(),
        ));
    }
    let need = (length - cur_len) as usize;
    let (left, right) = match pad_type {
        0 => (need, 0),                   // STR_PAD_LEFT
        2 => (need / 2, need - need / 2), // STR_PAD_BOTH (extra on the right)
        _ => (0, need),                   // STR_PAD_RIGHT
    };
    let mut out = cycle_pad(&pad, &pad_units, left);
    out.extend_from_slice(bytes);
    out.extend_from_slice(&cycle_pad(&pad, &pad_units, right));
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Default `mb_trim` character set: space, tab, LF, CR, VT, FF, NUL (wider than
/// `trim`'s default — it also strips form-feed, oracle-verified).
const MB_TRIM_DEFAULT: &[char] = &[' ', '\t', '\n', '\r', '\u{0B}', '\u{0C}', '\0'];

fn mb_trim_impl(
    args: &[Zval],
    ctx: &mut Ctx,
    func: &str,
    left: bool,
    right: bool,
) -> Result<Zval, PhpError> {
    let s = arg_str(args, func, ctx)?;
    let set: Vec<char> = match args.get(1) {
        None | Some(Zval::Null) => MB_TRIM_DEFAULT.to_vec(),
        Some(v) => cps(convert::to_zstr(v, ctx.diags).as_bytes())
            .into_iter()
            .map(|x| x.0)
            .collect(),
    };
    require_utf8(args, 2, ctx, func, 3, "encoding")?;
    let bytes = s.as_bytes();
    let cur = cps(bytes);
    let mut lo = 0usize;
    let mut hi = cur.len();
    if left {
        while lo < hi && set.contains(&cur[lo].0) {
            lo += 1;
        }
    }
    if right {
        while hi > lo && set.contains(&cur[hi - 1].0) {
            hi -= 1;
        }
    }
    let slice: &[u8] = if lo >= hi {
        &[]
    } else {
        &bytes[cur[lo].1..cur[hi - 1].1 + cur[hi - 1].2]
    };
    Ok(Zval::Str(PhpStr::new(slice.to_vec())))
}

/// mb_trim / mb_ltrim / mb_rtrim (PHP 8.4): strip the character set (default
/// whitespace) from both / left / right, by code point.
pub fn mb_trim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    mb_trim_impl(args, ctx, "mb_trim", true, true)
}
pub fn mb_ltrim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    mb_trim_impl(args, ctx, "mb_ltrim", true, false)
}
pub fn mb_rtrim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    mb_trim_impl(args, ctx, "mb_rtrim", false, true)
}

/// mb_check_encoding($value = null, $encoding = null): for UTF-8, true iff the
/// value is well-formed UTF-8 (no value → true).
pub fn mb_check_encoding(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    require_utf8(args, 1, ctx, "mb_check_encoding", 2, "encoding")?;
    let ok = match args.first() {
        None | Some(Zval::Null) => true,
        Some(v) => std::str::from_utf8(convert::to_zstr(v, ctx.diags).as_bytes()).is_ok(),
    };
    Ok(Zval::Bool(ok))
}
