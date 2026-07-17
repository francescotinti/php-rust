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

/// True when the `$encoding` argument at `idx` names the byte-transparent
/// `8bit`/`BINARY` encoding: every byte is one unit (symfony's
/// BinaryFileResponse measures fallback filenames with `mb_strlen(…, '8bit')`).
fn is_8bit_arg(args: &[Zval], idx: usize, ctx: &mut Ctx) -> bool {
    match args.get(idx) {
        None | Some(Zval::Null) => false,
        Some(v) => {
            let enc = convert::to_zstr(v, ctx.diags);
            enc.as_bytes().eq_ignore_ascii_case(b"8bit")
                || enc.as_bytes().eq_ignore_ascii_case(b"BINARY")
        }
    }
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
    if is_8bit_arg(args, 1, ctx) {
        return Ok(Zval::Long(s.as_bytes().len() as i64));
    }
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
    // `8bit`/`BINARY`: every byte is one unit (plain substr semantics).
    let bytes = s.as_bytes();
    let u = if is_8bit_arg(args, 3, ctx) {
        (0..bytes.len()).map(|i| (i, 1usize)).collect()
    } else {
        require_utf8(args, 3, ctx, "mb_substr", 4, "encoding")?;
        units(bytes)
    };
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
    // An offset outside the haystack is a ValueError (oracle-exact), for both the
    // forward and reverse variants.
    if offset > total || offset < -total {
        return Err(PhpError::ValueError(format!(
            "{func}(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
        )));
    }
    let found = if reverse {
        // The (in-range) offset on the reverse search is ignored (D-MB-rpos): the
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
    let codec = match args.get(1) {
        None | Some(Zval::Null) => Codec::Utf8,
        Some(v) => {
            let name = convert::to_zstr(v, ctx.diags);
            resolve_encoding_tracked(name.as_bytes(), "mb_check_encoding", ctx)
                .ok_or_else(|| {
                    PhpError::ValueError(format!(
                        "mb_check_encoding(): Argument #2 ($encoding) must be a valid encoding, \"{}\" given",
                        String::from_utf8_lossy(name.as_bytes())
                    ))
                })?
                .codec
        }
    };
    let ok = match args.first() {
        None | Some(Zval::Null) => true,
        Some(v) => validates(&codec, convert::to_zstr(v, ctx.diags).as_bytes()),
    };
    Ok(Zval::Bool(ok))
}

/// `mb_list_encodings(): array` — the mbstring encoding names phpr actually
/// resolves, in the oracle's ordering (the full mbfl list minus what is out of
/// scope here: never claim an encoding [`resolve_encoding`] cannot back).
pub fn mb_list_encodings(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const ORACLE_ORDER: &[&[u8]] = &[
        b"BASE64", b"UUENCODE", b"HTML-ENTITIES", b"Quoted-Printable", b"7bit", b"8bit",
        b"UCS-4", b"UCS-4BE", b"UCS-4LE", b"UCS-2", b"UCS-2BE", b"UCS-2LE", b"UTF-32",
        b"UTF-32BE", b"UTF-32LE", b"UTF-16", b"UTF-16BE", b"UTF-16LE", b"UTF-8", b"UTF-7",
        b"UTF7-IMAP", b"ASCII", b"EUC-JP", b"SJIS", b"eucJP-win", b"EUC-JP-2004",
        b"SJIS-Mobile#DOCOMO", b"SJIS-Mobile#KDDI", b"SJIS-Mobile#SOFTBANK", b"SJIS-mac",
        b"SJIS-2004", b"UTF-8-Mobile#DOCOMO", b"UTF-8-Mobile#KDDI-A", b"UTF-8-Mobile#KDDI-B",
        b"UTF-8-Mobile#SOFTBANK", b"CP932", b"SJIS-win", b"CP51932", b"JIS", b"ISO-2022-JP",
        b"ISO-2022-JP-MS", b"GB18030", b"GB18030-2022", b"Windows-1252", b"Windows-1254",
        b"ISO-8859-1", b"ISO-8859-2", b"ISO-8859-3", b"ISO-8859-4", b"ISO-8859-5",
        b"ISO-8859-6", b"ISO-8859-7", b"ISO-8859-8", b"ISO-8859-9", b"ISO-8859-10",
        b"ISO-8859-13", b"ISO-8859-14", b"ISO-8859-15", b"ISO-8859-16", b"EUC-CN", b"CP936",
        b"HZ", b"EUC-TW", b"BIG-5", b"CP950", b"EUC-KR", b"UHC", b"ISO-2022-KR",
        b"Windows-1251", b"CP866", b"KOI8-R", b"KOI8-U", b"ArmSCII-8", b"CP850", b"JIS-ms",
        b"ISO-2022-JP-2004", b"CP50220", b"CP50221", b"CP50222",
    ];
    let mut arr = PhpArray::new();
    for name in ORACLE_ORDER {
        if resolve_encoding(name).is_some() {
            let _ = arr.append(Zval::Str(PhpStr::new(name.to_vec())));
        }
    }
    Ok(Zval::Array(std::rc::Rc::new(arr)))
}

// --- step 42b: display width (mb_strwidth / mb_strimwidth / mb_strcut) ---

/// East-Asian-Width table ported verbatim from PHP's
/// `ext/mbstring/libmbfl/mbfl/eaw_table.h`: inclusive code-point ranges that
/// `mb_*` width functions render as double-width.
const FIRST_DOUBLEWIDTH: u32 = 0x1100;
#[rustfmt::skip]
static EAW_TABLE: &[(u32, u32)] = &[
    (0x1100, 0x115f), (0x231a, 0x231b), (0x2329, 0x232a), (0x23e9, 0x23ec),
    (0x23f0, 0x23f0), (0x23f3, 0x23f3), (0x25fd, 0x25fe), (0x2614, 0x2615),
    (0x2630, 0x2637), (0x2648, 0x2653), (0x267f, 0x267f), (0x268a, 0x268f),
    (0x2693, 0x2693), (0x26a1, 0x26a1), (0x26aa, 0x26ab), (0x26bd, 0x26be),
    (0x26c4, 0x26c5), (0x26ce, 0x26ce), (0x26d4, 0x26d4), (0x26ea, 0x26ea),
    (0x26f2, 0x26f3), (0x26f5, 0x26f5), (0x26fa, 0x26fa), (0x26fd, 0x26fd),
    (0x2705, 0x2705), (0x270a, 0x270b), (0x2728, 0x2728), (0x274c, 0x274c),
    (0x274e, 0x274e), (0x2753, 0x2755), (0x2757, 0x2757), (0x2795, 0x2797),
    (0x27b0, 0x27b0), (0x27bf, 0x27bf), (0x2b1b, 0x2b1c), (0x2b50, 0x2b50),
    (0x2b55, 0x2b55), (0x2e80, 0x2e99), (0x2e9b, 0x2ef3), (0x2f00, 0x2fd5),
    (0x2ff0, 0x303e), (0x3041, 0x3096), (0x3099, 0x30ff), (0x3105, 0x312f),
    (0x3131, 0x318e), (0x3190, 0x31e5), (0x31ef, 0x321e), (0x3220, 0x3247),
    (0x3250, 0xa48c), (0xa490, 0xa4c6), (0xa960, 0xa97c), (0xac00, 0xd7a3),
    (0xf900, 0xfaff), (0xfe10, 0xfe19), (0xfe30, 0xfe52), (0xfe54, 0xfe66),
    (0xfe68, 0xfe6b), (0xff01, 0xff60), (0xffe0, 0xffe6), (0x16fe0, 0x16fe4),
    (0x16ff0, 0x16ff6), (0x17000, 0x18cd5), (0x18cff, 0x18d1e), (0x18d80, 0x18df2),
    (0x1aff0, 0x1aff3), (0x1aff5, 0x1affb), (0x1affd, 0x1affe), (0x1b000, 0x1b122),
    (0x1b132, 0x1b132), (0x1b150, 0x1b152), (0x1b155, 0x1b155), (0x1b164, 0x1b167),
    (0x1b170, 0x1b2fb), (0x1d300, 0x1d356), (0x1d360, 0x1d376), (0x1f004, 0x1f004),
    (0x1f0cf, 0x1f0cf), (0x1f18e, 0x1f18e), (0x1f191, 0x1f19a), (0x1f200, 0x1f202),
    (0x1f210, 0x1f23b), (0x1f240, 0x1f248), (0x1f250, 0x1f251), (0x1f260, 0x1f265),
    (0x1f300, 0x1f320), (0x1f32d, 0x1f335), (0x1f337, 0x1f37c), (0x1f37e, 0x1f393),
    (0x1f3a0, 0x1f3ca), (0x1f3cf, 0x1f3d3), (0x1f3e0, 0x1f3f0), (0x1f3f4, 0x1f3f4),
    (0x1f3f8, 0x1f43e), (0x1f440, 0x1f440), (0x1f442, 0x1f4fc), (0x1f4ff, 0x1f53d),
    (0x1f54b, 0x1f54e), (0x1f550, 0x1f567), (0x1f57a, 0x1f57a), (0x1f595, 0x1f596),
    (0x1f5a4, 0x1f5a4), (0x1f5fb, 0x1f64f), (0x1f680, 0x1f6c5), (0x1f6cc, 0x1f6cc),
    (0x1f6d0, 0x1f6d2), (0x1f6d5, 0x1f6d8), (0x1f6dc, 0x1f6df), (0x1f6eb, 0x1f6ec),
    (0x1f6f4, 0x1f6fc), (0x1f7e0, 0x1f7eb), (0x1f7f0, 0x1f7f0), (0x1f90c, 0x1f93a),
    (0x1f93c, 0x1f945), (0x1f947, 0x1f9ff), (0x1fa70, 0x1fa7c), (0x1fa80, 0x1fa8a),
    (0x1fa8e, 0x1fac6), (0x1fac8, 0x1fac8), (0x1facd, 0x1fadc), (0x1fadf, 0x1faea),
    (0x1faef, 0x1faf8), (0x20000, 0x2fffd), (0x30000, 0x3fffd),
];

/// Display width of one code point: 2 for East-Asian Wide/Fullwidth (per
/// [`EAW_TABLE`]), 1 for everything else — combining marks, zero-width and
/// control characters included (mbfl assigns them width 1, not 0; see D-MB-width).
fn character_width(c: u32) -> usize {
    if c < FIRST_DOUBLEWIDTH {
        return 1;
    }
    match EAW_TABLE.binary_search_by(|&(b, e)| {
        use std::cmp::Ordering::*;
        if c < b {
            Greater
        } else if c > e {
            Less
        } else {
            Equal
        }
    }) {
        Ok(_) => 2,
        Err(_) => 1,
    }
}

/// mb_strwidth($string[, $encoding]): sum of code-point display widths.
pub fn mb_strwidth(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_strwidth", ctx)?;
    require_utf8(args, 1, ctx, "mb_strwidth", 2, "encoding")?;
    let w: usize = cps(s.as_bytes())
        .iter()
        .map(|&(c, _, _)| character_width(c as u32))
        .sum();
    Ok(Zval::Long(w as i64))
}

/// mb_strimwidth($string, $start, $width[, $trim_marker=""[, $encoding]]):
/// truncate to `$width` display columns starting at code-point `$start`,
/// appending `$trim_marker` when truncation happens (the marker's own width
/// counts toward the limit). An out-of-range `$start` is a `ValueError`.
pub fn mb_strimwidth(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_strimwidth", ctx)?;
    let start = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_strimwidth() expects at least 3 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let width = convert::to_long_cast(
        args.get(2).ok_or_else(|| {
            PhpError::Error("mb_strimwidth() expects at least 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    let marker_bytes: Vec<u8> = match args.get(3) {
        None | Some(Zval::Null) => Vec::new(),
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
    };
    require_utf8(args, 4, ctx, "mb_strimwidth", 5, "encoding")?;

    let bytes = s.as_bytes();
    let chars = cps(bytes);
    let n = chars.len() as i64;
    let from = if start < 0 { start + n } else { start };
    if from < 0 || from > n {
        return Err(PhpError::ValueError(
            "mb_strimwidth(): Argument #2 ($start) is out of range".to_string(),
        ));
    }
    let from = from as usize;

    // Byte slice for the half-open code-point range [a, b).
    let slice = |a: usize, b: usize| -> &[u8] {
        if b <= a {
            &[]
        } else {
            &bytes[chars[a].1..chars[b - 1].1 + chars[b - 1].2]
        }
    };
    let cw = |i: usize| character_width(chars[i].0 as u32) as i64;

    // The whole tail fits → return it untouched.
    let tail_width: i64 = (from..chars.len()).map(cw).sum();
    if tail_width <= width {
        return Ok(Zval::Str(PhpStr::new(slice(from, chars.len()).to_vec())));
    }

    // Truncate: reserve room for the marker, then take whole characters.
    let marker_width: i64 = cps(&marker_bytes)
        .iter()
        .map(|&(c, _, _)| character_width(c as u32) as i64)
        .sum();
    let avail = width - marker_width;
    let mut used = 0i64;
    let mut end = from;
    while end < chars.len() && used + cw(end) <= avail {
        used += cw(end);
        end += 1;
    }
    let mut out = slice(from, end).to_vec();
    out.extend_from_slice(&marker_bytes);
    Ok(Zval::Str(PhpStr::new(out)))
}

/// mb_strcut($string, $start[, $length[, $encoding]]): a byte-oriented cut that
/// never splits a multibyte character. `$start` rounds down to the character
/// boundary that contains it; `$length` is measured in bytes from that rounded
/// start, and only whole characters that fully fit are included.
pub fn mb_strcut(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_strcut", ctx)?;
    let start = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_strcut() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    require_utf8(args, 3, ctx, "mb_strcut", 4, "encoding")?;

    let bytes = s.as_bytes();
    let total = bytes.len() as i64;
    let u = units(bytes);
    // Resolve the start byte (negative counts from the end), clamped to [0, len].
    let from = if start < 0 {
        (total + start).max(0)
    } else {
        start.min(total)
    } as usize;
    // Round down to the start of the unit containing `from` (or len if past end).
    let rounded = u
        .iter()
        .find(|&&(b, l)| b <= from && from < b + l)
        .map(|&(b, _)| b)
        .unwrap_or(bytes.len());
    let limit = match args.get(2) {
        None | Some(Zval::Null) => bytes.len(),
        Some(v) => rounded.saturating_add(convert::to_long_cast(v, ctx.diags).max(0) as usize),
    };
    // Include whole units from `rounded` whose end byte stays within `limit`.
    let mut end = rounded;
    for &(b, l) in u.iter().filter(|&&(b, _)| b >= rounded) {
        if b + l <= limit {
            end = b + l;
        } else {
            break;
        }
    }
    Ok(Zval::Str(PhpStr::new(bytes[rounded..end].to_vec())))
}

// --- step 42a: encoding (mb_convert_encoding / mb_detect_encoding) ---

use encoding_rs::Encoding as RsEncoding;

/// Character codec backing the encoding-aware functions.
enum Codec {
    Ascii,
    Utf8,
    Latin1,
    Utf16Be,
    Utf16Le,
    /// mbstring's "HTML-ENTITIES" pseudo-encoding (mbfilter_htmlent.c): raw
    /// bytes are Latin-1 code points, `&…;` references decode to their
    /// character; encoding entitifies everything ≥ U+0080.
    HtmlEnt,
    Rs(&'static RsEncoding),
}

/// A resolved encoding: its codec plus the canonical PHP name that
/// `mb_detect_encoding` reports back.
struct Enc {
    codec: Codec,
    canonical: &'static str,
}

/// Resolve a PHP encoding name to a codec, or `None` if unsupported. ISO-8859-1
/// and the UTF-16 family are handled directly rather than through `encoding_rs`,
/// whose WHATWG label mapping diverges from PHP (true Latin-1 vs windows-1252;
/// no UTF-16 *encoder*) — see D-MB-enc-latin1 / D-MB-enc-utf16. Everything else
/// goes through `encoding_rs`'s label lookup.
fn resolve_encoding(name: &[u8]) -> Option<Enc> {
    let n = name.trim_ascii();
    let eq = |s: &[u8]| n.eq_ignore_ascii_case(s);
    if eq(b"UTF-8") || eq(b"UTF8") {
        Some(Enc { codec: Codec::Utf8, canonical: "UTF-8" })
    } else if eq(b"ASCII") || eq(b"US-ASCII") {
        Some(Enc { codec: Codec::Ascii, canonical: "ASCII" })
    } else if eq(b"ISO-8859-1") || eq(b"latin1") || eq(b"ISO8859-1") {
        Some(Enc { codec: Codec::Latin1, canonical: "ISO-8859-1" })
    } else if eq(b"UTF-16") || eq(b"UTF-16BE") {
        Some(Enc { codec: Codec::Utf16Be, canonical: "UTF-16BE" })
    } else if eq(b"UTF-16LE") {
        Some(Enc { codec: Codec::Utf16Le, canonical: "UTF-16LE" })
    } else if eq(b"SJIS") || eq(b"Shift_JIS") || eq(b"SHIFT-JIS") {
        Some(Enc { codec: Codec::Rs(encoding_rs::SHIFT_JIS), canonical: "SJIS" })
    } else if eq(b"EUC-JP") || eq(b"EUCJP") {
        Some(Enc { codec: Codec::Rs(encoding_rs::EUC_JP), canonical: "EUC-JP" })
    } else if eq(b"Windows-1252") || eq(b"CP1252") {
        Some(Enc { codec: Codec::Rs(encoding_rs::WINDOWS_1252), canonical: "Windows-1252" })
    } else if eq(b"HTML-ENTITIES") || eq(b"HTML") {
        Some(Enc { codec: Codec::HtmlEnt, canonical: "HTML-ENTITIES" })
    } else {
        RsEncoding::for_label_no_replacement(n).map(|e| Enc { codec: Codec::Rs(e), canonical: e.name() })
    }
}

thread_local! {
    /// The process-global mbstring internal encoding (canonical name). Batch-1
    /// mb functions operate on UTF-8 regardless; this state is what
    /// `mb_internal_encoding()` reports back (frameworks set it to "UTF-8" at
    /// bootstrap — the common, effect-free case).
    static MB_INTERNAL_ENCODING: std::cell::Cell<&'static str> = const { std::cell::Cell::new("UTF-8") };
}

/// `mb_internal_encoding(?string $encoding = null): string|bool` — get the current
/// internal encoding (canonical name) or set it (returns `true`); an unsupported
/// name is a `ValueError`.
pub fn mb_internal_encoding(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match args.first() {
        None | Some(Zval::Null) => {
            let cur = MB_INTERNAL_ENCODING.with(std::cell::Cell::get);
            Ok(Zval::Str(PhpStr::from_str(cur)))
        }
        Some(v) => {
            let name = convert::to_zstr(v, ctx.diags);
            match resolve_encoding(name.as_bytes()) {
                Some(enc) => {
                    MB_INTERNAL_ENCODING.with(|c| c.set(enc.canonical));
                    Ok(Zval::Bool(true))
                }
                None => Err(PhpError::ValueError(format!(
                    "mb_internal_encoding(): Argument #1 ($encoding) must be a valid encoding, \"{}\" given",
                    String::from_utf8_lossy(name.as_bytes())
                ))),
            }
        }
    }
}

thread_local! {
    /// `MBSTRG(last_used_encoding_name)`: the last explicit `$encoding` name
    /// resolved through the `php_mb_get_encoding` path. A cache hit (same name,
    /// case-insensitive) skips the deprecated-encoding check — which is why
    /// repeated identical calls warn only once on the oracle.
    static MB_LAST_USED: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
}

/// `php_mb_get_encoding` for an explicit name: resolve + deprecation on cache
/// miss (HTML-ENTITIES is the only deprecated pseudo-encoding we support).
fn resolve_encoding_tracked(name: &[u8], func: &str, ctx: &mut Ctx) -> Option<Enc> {
    let hit = MB_LAST_USED
        .with(|c| c.borrow().as_deref().is_some_and(|last| last.eq_ignore_ascii_case(name)));
    let enc = resolve_encoding(name)?;
    if !hit {
        if matches!(enc.codec, Codec::HtmlEnt) {
            ctx.diags.push(php_types::Diag::Deprecated(format!(
                "{func}(): Handling HTML entities via mbstring is deprecated; use \
                 htmlspecialchars, htmlentities, or \
                 mb_encode_numericentity/mb_decode_numericentity instead"
            )));
        }
        MB_LAST_USED.with(|c| *c.borrow_mut() = Some(name.to_vec()));
    }
    Some(enc)
}

/// Resolve an optional `$encoding` argument at `idx` (reported as argument
/// `arg_num` on error) to a codec, defaulting to the internal encoding when the
/// argument is absent or null. Mirrors C `php_mb_get_encoding`.
fn enc_arg(
    args: &[Zval],
    idx: usize,
    ctx: &mut Ctx,
    func: &str,
    arg_num: usize,
) -> Result<Enc, PhpError> {
    match args.get(idx) {
        None | Some(Zval::Null) => {
            let cur = MB_INTERNAL_ENCODING.with(std::cell::Cell::get);
            Ok(resolve_encoding(cur.as_bytes()).expect("internal encoding is always valid"))
        }
        Some(v) => {
            let name = convert::to_zstr(v, ctx.diags);
            resolve_encoding_tracked(name.as_bytes(), func, ctx).ok_or_else(|| {
                PhpError::ValueError(format!(
                    "{func}(): Argument #{arg_num} ($encoding) must be a valid encoding, \"{}\" given",
                    String::from_utf8_lossy(name.as_bytes())
                ))
            })
        }
    }
}

/// Coerce one `$map` element to a code value with `zval_try_get_long`
/// semantics: int/float/bool/null and *fully* numeric strings only; a
/// leading-numeric string ("5abc"), array, object or resource fails.
fn map_elem_long(v: &Zval) -> Option<i64> {
    match v {
        Zval::Undef | Zval::Null => Some(0),
        Zval::Bool(b) => Some(*b as i64),
        Zval::Long(l) => Some(*l),
        Zval::Double(d) => Some(convert::dval_to_lval(*d)),
        Zval::Str(s) => {
            php_types::numstr::parse_numeric_ex(s.as_bytes(), false).map(|i| match i.num {
                php_types::numstr::Num::Long(l) => l,
                php_types::numstr::Num::Double(d) => convert::dval_to_lval(d),
            })
        }
        Zval::Ref(c) => map_elem_long(&c.borrow()),
        _ => None,
    }
}

/// Build the flat `[lo, hi, offset, mask]*` conversion map from the `$map`
/// argument (C `make_conversion_map`): it must be an array whose element count
/// is a multiple of 4 and whose values all coerce to int.
fn build_convmap(args: &[Zval], func: &str) -> Result<Vec<u32>, PhpError> {
    let arr = match args.get(1) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "{func}(): Argument #2 ($map) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(format!(
                "{func}() expects at least 2 arguments, {} given",
                args.len()
            )))
        }
    };
    if arr.len() % 4 != 0 {
        return Err(PhpError::ValueError(format!(
            "{func}(): Argument #2 ($map) must have a multiple of 4 elements"
        )));
    }
    let mut map = Vec::with_capacity(arr.len());
    for (_, v) in arr.iter() {
        match map_elem_long(v) {
            Some(n) => map.push(n as u32),
            None => {
                return Err(PhpError::ValueError(format!(
                    "{func}(): Argument #2 ($map) must only be composed of values of type int"
                )))
            }
        }
    }
    Ok(map)
}

/// Encode side of the convmap: the first `[lo, hi, offset, mask]` group whose
/// range contains `w` yields `(w + offset) & mask` (all u32, wrapping).
fn convmap_encode(w: u32, map: &[u32]) -> Option<u32> {
    for e in map.chunks_exact(4) {
        if w >= e[0] && w <= e[1] {
            return Some(w.wrapping_add(e[2]) & e[3]);
        }
    }
    None
}

/// Decode side of the convmap: `codepoint = number - offset`; the first group
/// whose range contains that codepoint yields it.
fn convmap_decode(number: u32, map: &[u32]) -> Option<u32> {
    for e in map.chunks_exact(4) {
        let codepoint = number.wrapping_sub(e[2]);
        if codepoint >= e[0] && codepoint <= e[1] {
            return Some(codepoint);
        }
    }
    None
}

/// mb_encode_numericentity($string, $map, ?$encoding = null, $hex = false):
/// convert every code point falling in one of the `$map` ranges to a decimal
/// (or hex, when `$hex`) HTML numeric entity `&#NNN;` / `&#xHH;`.
pub fn mb_encode_numericentity(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const FUNC: &str = "mb_encode_numericentity";
    let s = arg_str(args, FUNC, ctx)?;
    let map = build_convmap(args, FUNC)?;
    let enc = enc_arg(args, 2, ctx, FUNC, 3)?;
    let hex = args.get(3).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);

    let text = decode_bytes(&enc.codec, s.as_bytes());
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match convmap_encode(ch as u32, &map) {
            Some(v) => {
                out.push('&');
                out.push('#');
                if hex {
                    out.push('x');
                    out.push_str(&format!("{v:X}"));
                } else {
                    out.push_str(&v.to_string());
                }
                out.push(';');
            }
            None => out.push(ch),
        }
    }
    Ok(Zval::Str(PhpStr::new(encode_str(&enc.codec, &out))))
}

/// mb_decode_numericentity($string, $map, ?$encoding = null): the inverse of
/// [`mb_encode_numericentity`] — replace `&#NNN;` / `&#xHH;` entities whose
/// (offset-adjusted) value falls in a `$map` range with the code point. A
/// non-matching or malformed entity is left verbatim; the terminating `;` is
/// optional. Ports C `html_numeric_entity_decode` (whole-buffer form).
pub fn mb_decode_numericentity(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const FUNC: &str = "mb_decode_numericentity";
    let s = arg_str(args, FUNC, ctx)?;
    let map = build_convmap(args, FUNC)?;
    let enc = enc_arg(args, 2, ctx, FUNC, 3)?;

    let text = decode_bytes(&enc.codec, s.as_bytes());
    let w: Vec<char> = text.chars().collect();
    let n = w.len();
    // "&#" + 1..=10 decimal digits; "&#x" + 1..=8 hex digits (C DEC/HEX
    // ENTITY_MIN/MAXLEN, measured from the leading '&').
    const DEC_MIN: usize = 3;
    const DEC_MAX: usize = 12;
    const HEX_MIN: usize = 4;
    const HEX_MAX: usize = 11;

    let is_dec = |c: char| c.is_ascii_digit();
    let is_hex = |c: char| c.is_ascii_hexdigit();
    let hexval = |c: char| -> u32 {
        if c <= '9' {
            c as u32 - '0' as u32
        } else if c >= 'a' {
            10 + (c as u32 - 'a' as u32)
        } else {
            10 + (c as u32 - 'A' as u32)
        }
    };

    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < n {
        if w[i] != '&' || i + 1 >= n || w[i + 1] != '#' {
            out.push(w[i]);
            i += 1;
            continue;
        }
        // At a "&#..." candidate entity.
        let hex = i + 2 < n && w[i + 2] == 'x';
        let digits_start = if hex { i + 3 } else { i + 2 };
        let mut j = digits_start;
        if hex {
            while j < n && is_hex(w[j]) {
                j += 1;
            }
        } else {
            while j < n && is_dec(w[j]) {
                j += 1;
            }
        }
        let len = j - i;
        let (min, max) = if hex { (HEX_MIN, HEX_MAX) } else { (DEC_MIN, DEC_MAX) };
        let mut ok = len >= min && len <= max;
        let mut value: u32 = 0;
        if ok {
            if hex {
                for k in digits_start..j {
                    value = value.wrapping_mul(16).wrapping_add(hexval(w[k]));
                }
            } else {
                for k in digits_start..j {
                    // Reject on the same u32 multiplication-overflow boundary as C.
                    if value > 0x1999_9999 {
                        ok = false;
                        break;
                    }
                    value = value * 10 + (w[k] as u32 - '0' as u32);
                }
            }
        }
        if ok {
            if let Some(cp) = convmap_decode(value, &map) {
                out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                // The terminating ';' is optional; consume it when present.
                i = if j < n && w[j] == ';' { j + 1 } else { j };
                continue;
            }
        }
        // Invalid or non-matching entity: emit the literal "&#..." run verbatim.
        for &c in &w[i..j] {
            out.push(c);
        }
        i = j;
    }

    Ok(Zval::Str(PhpStr::new(encode_str(&enc.codec, &out))))
}

fn decode_utf16(bytes: &[u8], be: bool) -> String {
    let words = bytes.chunks_exact(2).map(|c| {
        if be {
            u16::from_be_bytes([c[0], c[1]])
        } else {
            u16::from_le_bytes([c[0], c[1]])
        }
    });
    char::decode_utf16(words)
        .map(|r| r.unwrap_or('\u{FFFD}'))
        .collect()
}

/// mbstring's HTML entity table (html_entities.c, `mbfl_html_entity_list`) —
/// the HTML 4 set, NOT the HTML5 one `htmlspecialchars`/`html.rs` use; shared
/// with the loadHTML parser in [`php_types::html4`]. Order matters for
/// encoding ties only (first match wins, as in the C linear scan).
use php_types::html4::HTML4_ENTITIES as MBFL_HTML_ENTITIES;

/// HTML-ENTITIES → chars, a faithful port of `mb_htmlent_to_wchar`: bytes pass
/// through as Latin-1 code points; `&…;` whose body is `[0-9A-Za-z#]` decodes
/// numerically (wrapping u32 arithmetic, cap U+10FFFF) or via the HTML 4 table
/// (case-sensitive); anything else flushes verbatim. A decoded value that is
/// not a Unicode scalar becomes `?` (the wchar→UTF-8 substitute).
fn html_ent_decode(bytes: &[u8]) -> String {
    fn is_ent_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'#'
    }
    let mut out = String::new();
    let push_cp = |out: &mut String, v: u32| out.push(char::from_u32(v).unwrap_or('?'));
    let e = bytes.len();
    let mut p = 0;
    while p < e {
        let c = bytes[p];
        p += 1;
        if c != b'&' {
            out.push(c as char);
            continue;
        }
        let mut term = p;
        while term < e && is_ent_char(bytes[term]) {
            term += 1;
        }
        if term < e && bytes[term] == b';' {
            if bytes.get(p) == Some(&b'#') && e - p >= 2 {
                // Numeric reference.
                let mut digits = p + 1;
                let hex = matches!(bytes.get(digits), Some(b'x') | Some(b'X'));
                if hex {
                    digits += 1;
                }
                let mut value: u32 = 0;
                let mut ok = digits < term;
                for &d in &bytes[digits.min(term)..term] {
                    let v = match (d, hex) {
                        (b'0'..=b'9', _) => (d - b'0') as u32,
                        (b'A'..=b'F', true) => (d - b'A' + 10) as u32,
                        (b'a'..=b'f', true) => (d - b'a' + 10) as u32,
                        _ => {
                            ok = false;
                            break;
                        }
                    };
                    value = value.wrapping_mul(if hex { 16 } else { 10 }).wrapping_add(v);
                }
                if ok && value <= 0x10FFFF {
                    push_cp(&mut out, value);
                    p = term + 1;
                    continue;
                }
            } else if term > p {
                // Named reference.
                if let Some(&(_, code)) =
                    MBFL_HTML_ENTITIES.iter().find(|(n, _)| *n == &bytes[p..term])
                {
                    push_cp(&mut out, code);
                    p = term + 1;
                    continue;
                }
            }
        }
        // Unterminated or unrecognized: `&` and the scanned run pass through
        // (the `;` too when present — that is the not-an-entity case).
        out.push('&');
        for &b in &bytes[p..term] {
            out.push(b as char);
        }
        p = term;
        if term < e && bytes[term] == b';' {
            out.push(';');
            p = term + 1;
        }
    }
    out
}

/// chars → HTML-ENTITIES (`mb_wchar_to_htmlent`): everything below U+0080 is
/// literal (including `&<>`); the rest becomes a named entity from the HTML 4
/// table or a decimal `&#N;`.
fn html_ent_encode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for c in s.chars() {
        let w = c as u32;
        if w < 0x80 {
            out.push(w as u8);
            continue;
        }
        out.push(b'&');
        match MBFL_HTML_ENTITIES.iter().find(|(_, code)| *code == w) {
            Some((name, _)) => out.extend_from_slice(name),
            None => {
                out.push(b'#');
                out.extend_from_slice(w.to_string().as_bytes());
            }
        }
        out.push(b';');
    }
    out
}

/// Decode bytes in `codec` to a UTF-8 `String` (the internal representation),
/// substituting U+FFFD for malformed input.
fn decode_bytes(codec: &Codec, bytes: &[u8]) -> String {
    match codec {
        Codec::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
        Codec::Ascii => bytes
            .iter()
            .map(|&b| if b < 0x80 { b as char } else { '\u{FFFD}' })
            .collect(),
        Codec::Latin1 => bytes.iter().map(|&b| b as char).collect(),
        Codec::Utf16Be => decode_utf16(bytes, true),
        Codec::Utf16Le => decode_utf16(bytes, false),
        Codec::HtmlEnt => html_ent_decode(bytes),
        Codec::Rs(e) => e.decode_without_bom_handling(bytes).0.into_owned(),
    }
}

/// Encode a UTF-8 `String` to `codec`, substituting `?` (0x3F) for any character
/// the target cannot represent — PHP's default substitute, not the HTML numeric
/// entity `encoding_rs::encode` would emit (D-MB-enc-subst).
fn encode_str(codec: &Codec, s: &str) -> Vec<u8> {
    match codec {
        Codec::Utf8 => s.as_bytes().to_vec(),
        Codec::Ascii => s
            .chars()
            .map(|c| if (c as u32) < 0x80 { c as u8 } else { b'?' })
            .collect(),
        Codec::Latin1 => s
            .chars()
            .map(|c| if (c as u32) <= 0xFF { c as u8 } else { b'?' })
            .collect(),
        Codec::Utf16Be => s.encode_utf16().flat_map(|u| u.to_be_bytes()).collect(),
        Codec::Utf16Le => s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect(),
        Codec::HtmlEnt => html_ent_encode(s),
        Codec::Rs(e) => {
            let mut out = Vec::new();
            let mut buf = [0u8; 4];
            for c in s.chars() {
                let (bytes, _, unmappable) = e.encode(c.encode_utf8(&mut buf));
                if unmappable {
                    out.push(b'?');
                } else {
                    out.extend_from_slice(&bytes);
                }
            }
            out
        }
    }
}

/// Whether `bytes` decode in `codec` without any malformed sequence (used by
/// `mb_detect_encoding`).
fn validates(codec: &Codec, bytes: &[u8]) -> bool {
    match codec {
        Codec::Utf8 => std::str::from_utf8(bytes).is_ok(),
        Codec::Ascii => bytes.iter().all(|&b| b < 0x80),
        Codec::Latin1 => true,
        Codec::Utf16Be | Codec::Utf16Le => bytes.len().is_multiple_of(2),
        // The htmlent decoder never rejects input (bad references flush verbatim).
        Codec::HtmlEnt => true,
        Codec::Rs(e) => !e.decode_without_bom_handling(bytes).1,
    }
}

/// Parse an encoding-list argument: an array of strings or a comma-separated
/// string. Returns the raw candidate names (trimming is done by `resolve_encoding`).
fn parse_enc_list(v: &Zval, ctx: &mut Ctx) -> Vec<Vec<u8>> {
    let names: Vec<Vec<u8>> = match v {
        Zval::Array(a) => a
            .iter()
            .map(|(_, val)| convert::to_zstr(val, ctx.diags).as_bytes().to_vec())
            .collect(),
        _ => convert::to_zstr(v, ctx.diags)
            .as_bytes()
            .split(|&b| b == b',')
            .map(|p| p.to_vec())
            .collect(),
    };
    // Drop empty/whitespace-only entries so `''` parses to zero encodings (PHP
    // then raises "must specify at least one encoding").
    names
        .into_iter()
        .filter(|n| !n.trim_ascii().is_empty())
        .collect()
}

/// mb_convert_encoding($string, $to_encoding[, $from_encoding=null]): transcode
/// `$string` from `$from_encoding` (UTF-8 by default, or detected from a
/// list/comma string) to `$to_encoding`.
pub fn mb_convert_encoding(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_convert_encoding", ctx)?;
    let to_raw = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("mb_convert_encoding() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let to = resolve_encoding_tracked(to_raw.as_bytes(), "mb_convert_encoding", ctx).ok_or_else(|| {
        PhpError::ValueError(format!(
            "mb_convert_encoding(): Argument #2 ($to_encoding) must be a valid encoding, \"{}\" given",
            String::from_utf8_lossy(to_raw.as_bytes())
        ))
    })?;

    let from = match args.get(2) {
        None | Some(Zval::Null) => Codec::Utf8,
        Some(v) => {
            let names = parse_enc_list(v, ctx);
            if names.is_empty() {
                return Err(PhpError::ValueError(
                    "mb_convert_encoding(): Argument #3 ($from_encoding) must specify at least one encoding"
                        .to_string(),
                ));
            }
            let mut encs = Vec::new();
            for name in names {
                let e = resolve_encoding(&name).ok_or_else(|| {
                    PhpError::ValueError(format!(
                        "mb_convert_encoding(): Argument #3 ($from_encoding) contains invalid encoding \"{}\"",
                        String::from_utf8_lossy(&name)
                    ))
                })?;
                encs.push(e);
            }
            if encs.len() == 1 {
                encs.pop().unwrap().codec
            } else {
                // Detect among the candidates (non-strict: first valid, else first).
                let idx = encs
                    .iter()
                    .position(|e| validates(&e.codec, s.as_bytes()))
                    .unwrap_or(0);
                encs.into_iter().nth(idx).unwrap().codec
            }
        }
    };

    let decoded = decode_bytes(&from, s.as_bytes());
    Ok(Zval::Str(PhpStr::new(encode_str(&to.codec, &decoded))))
}

/// Encode `s` to `codec` for `iconv`: `//IGNORE` drops a char the target cannot
/// represent, `//TRANSLIT` substitutes a placeholder (`?`); with neither, an
/// unmappable char fails the whole conversion (returns `None`, → `false`).
fn iconv_encode(codec: &Codec, s: &str, ignore: bool, translit: bool) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut buf = [0u8; 4];
    for c in s.chars() {
        let mapped: Option<Vec<u8>> = match codec {
            Codec::Utf8 => Some(c.encode_utf8(&mut buf).as_bytes().to_vec()),
            Codec::Ascii => ((c as u32) < 0x80).then(|| vec![c as u8]),
            Codec::Latin1 => ((c as u32) <= 0xFF).then(|| vec![c as u8]),
            Codec::Utf16Be => {
                let mut u = [0u16; 2];
                Some(c.encode_utf16(&mut u).iter().flat_map(|x| x.to_be_bytes()).collect())
            }
            Codec::Utf16Le => {
                let mut u = [0u16; 2];
                Some(c.encode_utf16(&mut u).iter().flat_map(|x| x.to_le_bytes()).collect())
            }
            // Unreachable from iconv() (rejected as a charset there); total anyway.
            Codec::HtmlEnt => Some(html_ent_encode(c.encode_utf8(&mut buf))),
            Codec::Rs(e) => {
                let (bytes, _, unmappable) = e.encode(c.encode_utf8(&mut buf));
                (!unmappable).then(|| bytes.into_owned())
            }
        };
        match mapped {
            Some(b) => out.extend_from_slice(&b),
            None if ignore => continue,
            None if translit => out.push(b'?'),
            None => return None,
        }
    }
    Some(out)
}

/// `iconv($from_encoding, $to_encoding, $string)`: convert `$string` between
/// charsets. `$to_encoding` may carry `//TRANSLIT` and/or `//IGNORE` suffixes. An
/// unknown charset warns and returns false; an unmappable char (without a suffix)
/// warns and returns false too. Reuses the mbstring [`Codec`] tables.
pub fn iconv(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if args.len() < 3 {
        return Err(PhpError::ArgumentCountError(format!(
            "iconv() expects exactly 3 arguments, {} given",
            args.len()
        )));
    }
    let from = convert::to_zstr(&args[0], ctx.diags);
    let to_raw = convert::to_zstr(&args[1], ctx.diags);
    let s = convert::to_zstr(&args[2], ctx.diags);
    let tb = to_raw.as_bytes();
    let (base, flags) = match tb.windows(2).position(|w| w == b"//") {
        Some(p) => (&tb[..p], tb[p..].to_ascii_uppercase()),
        None => (tb, Vec::new()),
    };
    let ignore = flags.windows(6).any(|w| w == b"IGNORE");
    let translit = flags.windows(8).any(|w| w == b"TRANSLIT");
    // HTML-ENTITIES is an mbstring-only pseudo-encoding; iconv rejects it.
    let not_iconv = |e: Option<Enc>| e.filter(|e| !matches!(e.codec, Codec::HtmlEnt));
    let from_enc = not_iconv(resolve_encoding(from.as_bytes()));
    let to_enc = not_iconv(resolve_encoding(base));
    let (Some(from_enc), Some(to_enc)) = (from_enc, to_enc) else {
        ctx.diags.push(php_types::Diag::Warning(format!(
            "iconv(): Wrong charset, conversion from \"{}\" to \"{}\" is not allowed",
            String::from_utf8_lossy(from.as_bytes()),
            String::from_utf8_lossy(tb)
        )));
        return Ok(Zval::Bool(false));
    };
    let decoded = decode_bytes(&from_enc.codec, s.as_bytes());
    match iconv_encode(&to_enc.codec, &decoded, ignore, translit) {
        Some(out) => Ok(Zval::Str(PhpStr::new(out))),
        None => {
            ctx.diags.push(php_types::Diag::Warning(
                "iconv(): Detected an illegal character in input string".to_string(),
            ));
            Ok(Zval::Bool(false))
        }
    }
}

/// mb_detect_encoding($string[, $encodings=null[, $strict=false]]): return the
/// canonical name of the first candidate under which `$string` is valid. The
/// default candidate order is ASCII, UTF-8. Non-strict mode falls back to the
/// first candidate (never `false`); strict mode returns `false` when none match.
pub fn mb_detect_encoding(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, "mb_detect_encoding", ctx)?;
    let names: Vec<Vec<u8>> = match args.get(1) {
        None | Some(Zval::Null) => vec![b"ASCII".to_vec(), b"UTF-8".to_vec()],
        Some(v) => parse_enc_list(v, ctx),
    };
    if names.is_empty() {
        return Err(PhpError::ValueError(
            "mb_detect_encoding(): Argument #2 ($encodings) must specify at least one encoding"
                .to_string(),
        ));
    }
    let mut encs = Vec::new();
    for name in &names {
        let e = resolve_encoding(name).ok_or_else(|| {
            PhpError::ValueError(format!(
                "mb_detect_encoding(): Argument #2 ($encodings) contains invalid encoding \"{}\"",
                String::from_utf8_lossy(name)
            ))
        })?;
        encs.push(e);
    }
    let strict = match args.get(2) {
        None | Some(Zval::Null) => false,
        Some(v) => convert::to_bool(v, ctx.diags),
    };
    let bytes = s.as_bytes();
    let chosen = encs.iter().find(|e| validates(&e.codec, bytes));
    Ok(match chosen {
        Some(e) => Zval::Str(PhpStr::new(e.canonical.as_bytes().to_vec())),
        None if strict => Zval::Bool(false),
        None => Zval::Str(PhpStr::new(encs[0].canonical.as_bytes().to_vec())),
    })
}
