//! String builtins (plan step 10): implode, explode, substr, ...

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Diag, PhpArray, PhpError, PhpStr, Zval};

/// implode($separator, $array) or implode($array).
///
/// PHP 8 removed the legacy reversed `implode($array, $glue)` order: passing an
/// array as the separator is now a `TypeError`. Each element is string-coerced.
pub fn implode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (glue, arr) = match args {
        // Single argument: it must be the array; glue is empty.
        [Zval::Array(a)] => (Vec::new(), a),
        [only] => {
            return Err(PhpError::TypeError(format!(
                "implode(): Argument #1 ($array) must be of type array, {} given",
                only.error_type_name()
            )))
        }
        [sep, rest @ ..] => {
            if let Zval::Array(_) = sep {
                return Err(PhpError::TypeError(
                    "implode(): Argument #1 ($separator) must be of type string, array given"
                        .to_string(),
                ));
            }
            match rest.first() {
                Some(Zval::Array(a)) => (convert::to_zstr(sep, ctx.diags).as_bytes().to_vec(), a),
                Some(other) => {
                    return Err(PhpError::TypeError(format!(
                        "implode(): Argument #2 ($array) must be of type array, {} given",
                        other.error_type_name()
                    )))
                }
                None => unreachable!("rest has at least one element"),
            }
        }
        [] => {
            return Err(PhpError::Error(
                "implode() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };

    let mut out = Vec::new();
    for (i, (_, v)) in arr.iter().enumerate() {
        if i > 0 {
            out.extend_from_slice(&glue);
        }
        out.extend_from_slice(convert::to_zstr(v, ctx.diags).as_bytes());
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// explode($separator, $string, $limit = PHP_INT_MAX).
///
/// Empty separator is a `ValueError`. Positive limit caps the element count
/// (last element keeps the unsplit remainder); 0 behaves like 1; negative
/// drops that many trailing pieces.
pub fn explode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let sep = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("explode() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let sep = sep.as_bytes();
    if sep.is_empty() {
        return Err(PhpError::ValueError(
            "explode(): Argument #1 ($separator) must not be empty".to_string(),
        ));
    }
    let string = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("explode() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let limit = match args.get(2) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => i64::MAX,
    };

    let parts = split_all(string.as_bytes(), sep);
    let mut out = PhpArray::new();

    if limit > 0 && (limit as usize) < parts.len() {
        // Keep the first limit-1 pieces verbatim, then the rest as one element
        // reconstructed by re-joining with the separator.
        let keep = limit as usize - 1;
        for p in &parts[..keep] {
            let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
        }
        let remainder = parts[keep..].join(sep);
        let _ = out.append(Zval::Str(PhpStr::new(remainder)));
    } else if limit < 0 {
        let drop = (-limit) as usize;
        let keep = parts.len().saturating_sub(drop);
        for p in &parts[..keep] {
            let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
        }
    } else {
        // limit == 0 behaves like 1; limit >= parts.len() keeps everything.
        if limit == 0 {
            let _ = out.append(Zval::Str(Rc::clone(&string)));
        } else {
            for p in &parts {
                let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
            }
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// substr($string, $offset[, $length]).
///
/// Offsets and lengths may be negative (counted from the end). The resulting
/// window is clamped into `[0, len]`; an empty window yields "".
pub fn substr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("substr() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let bytes = s.as_bytes();
    let len = bytes.len() as i64;
    let offset = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("substr() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );

    let start = if offset < 0 {
        (len + offset).max(0)
    } else {
        offset.min(len)
    };
    let end = match args.get(2) {
        None | Some(Zval::Null) => len,
        Some(v) => {
            let length = convert::to_long_cast(v, ctx.diags);
            if length < 0 {
                (len + length).max(start)
            } else {
                (start + length).min(len)
            }
        }
    };
    let window = if end > start {
        &bytes[start as usize..end as usize]
    } else {
        &[]
    };
    Ok(Zval::Str(PhpStr::new(window.to_vec())))
}

/// strpos($haystack, $needle[, $offset]): byte index of the first occurrence at
/// or after `$offset`, or `false`. Negative offset counts from the end; an
/// offset outside the string is a `ValueError`.
pub fn strpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("strpos() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("strpos() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let hay = haystack.as_bytes();
    let len = hay.len() as i64;
    let offset = match args.get(2) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 0,
    };
    let start = if offset < 0 { len + offset } else { offset };
    if start < 0 || start > len {
        return Err(PhpError::ValueError(
            "strpos(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
                .to_string(),
        ));
    }
    let start = start as usize;
    match find_sub(&hay[start..], needle.as_bytes()) {
        Some(pos) => Ok(Zval::Long((start + pos) as i64)),
        None => Ok(Zval::Bool(false)),
    }
}

/// str_replace($search, $replace, $subject).
///
/// `$search`/`$replace` may be scalars or arrays (element-wise, replacements
/// applied sequentially). When `$subject` is an array each element is processed
/// and an array is returned. The optional by-reference `$count` is unsupported.
pub fn str_replace(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let search = args.first().ok_or_else(|| {
        PhpError::Error("str_replace() expects at least 3 arguments, 0 given".to_string())
    })?;
    let replace = args.get(1).ok_or_else(|| {
        PhpError::Error("str_replace() expects at least 3 arguments, 1 given".to_string())
    })?;
    let subject = args.get(2).ok_or_else(|| {
        PhpError::Error("str_replace() expects at least 3 arguments, 2 given".to_string())
    })?;

    // Build the (search, replacement) pair list once.
    let pairs = replacement_pairs(search, replace, ctx);

    let apply = |subj: &Zval, ctx: &mut Ctx| -> Vec<u8> {
        let mut cur = convert::to_zstr(subj, ctx.diags).as_bytes().to_vec();
        for (s, r) in &pairs {
            if !s.is_empty() {
                cur = replace_all(&cur, s, r);
            }
        }
        cur
    };

    if let Zval::Array(a) = subject {
        let mut out = PhpArray::new();
        let entries: Vec<_> = a.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (k, v) in entries {
            out.insert(k, Zval::Str(PhpStr::new(apply(&v, ctx))));
        }
        Ok(Zval::Array(Rc::new(out)))
    } else {
        Ok(Zval::Str(PhpStr::new(apply(subject, ctx))))
    }
}

/// Pair each search term with its replacement. Scalars become a single pair;
/// an array search pairs element-wise with an array replace (missing entries
/// replace with "") or with the same scalar replace for every term.
fn replacement_pairs(search: &Zval, replace: &Zval, ctx: &mut Ctx) -> Vec<(Vec<u8>, Vec<u8>)> {
    match search {
        Zval::Array(searches) => {
            let repls: Option<Vec<Vec<u8>>> = match replace {
                Zval::Array(r) => Some(
                    r.iter()
                        .map(|(_, v)| convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
                        .collect(),
                ),
                _ => None,
            };
            let scalar_repl = repls
                .is_none()
                .then(|| convert::to_zstr(replace, ctx.diags).as_bytes().to_vec());
            searches
                .iter()
                .enumerate()
                .map(|(i, (_, s))| {
                    let s = convert::to_zstr(s, ctx.diags).as_bytes().to_vec();
                    let r = match &repls {
                        Some(list) => list.get(i).cloned().unwrap_or_default(),
                        None => scalar_repl.clone().unwrap(),
                    };
                    (s, r)
                })
                .collect()
        }
        _ => vec![(
            convert::to_zstr(search, ctx.diags).as_bytes().to_vec(),
            convert::to_zstr(replace, ctx.diags).as_bytes().to_vec(),
        )],
    }
}

/// Replace every non-overlapping occurrence of `from` (non-empty) in `s`.
fn replace_all(s: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if i + from.len() <= s.len() && &s[i..i + from.len()] == from {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    out
}

/// Sole `&str` argument coerced to bytes, or an `ArgumentCountError`-style fatal.
fn str_arg(args: &[Zval], ctx: &mut Ctx, fname: &str) -> Result<Vec<u8>, PhpError> {
    let v = args
        .first()
        .ok_or_else(|| PhpError::Error(format!("{fname}() expects exactly 1 argument, 0 given")))?;
    Ok(convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
}

/// strtoupper($string): ASCII-only uppercasing (C locale); bytes >= 0x80 intact.
pub fn strtoupper(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "strtoupper")?;
    b.make_ascii_uppercase();
    Ok(Zval::Str(PhpStr::new(b)))
}

/// strtolower($string): ASCII-only lowercasing (C locale); bytes >= 0x80 intact.
pub fn strtolower(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "strtolower")?;
    b.make_ascii_lowercase();
    Ok(Zval::Str(PhpStr::new(b)))
}

/// ucfirst($string): uppercase the first byte only.
pub fn ucfirst(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "ucfirst")?;
    if let Some(first) = b.first_mut() {
        first.make_ascii_uppercase();
    }
    Ok(Zval::Str(PhpStr::new(b)))
}

/// lcfirst($string): lowercase the first byte only.
pub fn lcfirst(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "lcfirst")?;
    if let Some(first) = b.first_mut() {
        first.make_ascii_lowercase();
    }
    Ok(Zval::Str(PhpStr::new(b)))
}

/// ucwords($string[, $delimiters]): uppercase the first byte and every byte that
/// follows a delimiter. Default delimiters are " \t\r\n\f\v".
pub fn ucwords(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "ucwords")?;
    let delims: Vec<u8> = match args.get(1) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b" \t\r\n\x0c\x0b".to_vec(),
    };
    let mut at_word_start = true;
    for byte in b.iter_mut() {
        if at_word_start {
            byte.make_ascii_uppercase();
        }
        at_word_start = delims.contains(byte);
    }
    Ok(Zval::Str(PhpStr::new(b)))
}

/// str_repeat($string, $times): `$times` copies. Negative is a `ValueError`.
pub fn str_repeat(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("str_repeat() expects exactly 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let times = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("str_repeat() expects exactly 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    if times < 0 {
        return Err(PhpError::ValueError(
            "str_repeat(): Argument #2 ($times) must be greater than or equal to 0".to_string(),
        ));
    }
    Ok(Zval::Str(PhpStr::new(s.as_bytes().repeat(times as usize))))
}

/// str_pad($string, $length, $pad_string = " ", $pad_type = STR_PAD_RIGHT).
///
/// `$pad_type`: 0 = left, 1 = right (default), 2 = both (extra char on the
/// right). A length <= the input length returns it unchanged; an empty pad
/// string is a `ValueError`.
pub fn str_pad(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("str_pad() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let s = s.as_bytes();
    let length = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("str_pad() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let pad = match args.get(2) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b" ".to_vec(),
    };
    let pad_type = match args.get(3) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 1,
    };

    if length <= s.len() as i64 {
        return Ok(Zval::Str(PhpStr::new(s.to_vec())));
    }
    if pad.is_empty() {
        return Err(PhpError::ValueError(
            "str_pad(): Argument #3 ($pad_string) must not be empty".to_string(),
        ));
    }
    let total = (length as usize) - s.len();
    let (left, right) = match pad_type {
        0 => (total, 0),         // STR_PAD_LEFT
        2 => (total / 2, total - total / 2), // STR_PAD_BOTH (extra on the right)
        _ => (0, total),         // STR_PAD_RIGHT (default)
    };
    let mut out = Vec::with_capacity(length as usize);
    out.extend(pad.iter().cycle().take(left));
    out.extend_from_slice(s);
    out.extend(pad.iter().cycle().take(right));
    Ok(Zval::Str(PhpStr::new(out)))
}

/// chr($codepoint): a single byte, `$codepoint` reduced modulo 256.
pub fn chr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = convert::to_long_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("chr() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let byte = n.rem_euclid(256) as u8;
    Ok(Zval::Str(PhpStr::new(vec![byte])))
}

/// ord($character): the value of the first byte (0 for an empty string).
pub fn ord(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("ord() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    Ok(Zval::Long(s.as_bytes().first().copied().unwrap_or(0) as i64))
}

/// Default trim character set: " \t\n\r\0\x0B".
const TRIM_DEFAULT: &[u8] = b" \t\n\r\0\x0b";

/// Build the 256-entry membership mask for a trim charlist. A `a..b` triple
/// expands to the inclusive byte range, matching PHP's `php_charmask`.
fn trim_mask(chars: &[u8]) -> [bool; 256] {
    let mut mask = [false; 256];
    let mut i = 0;
    while i < chars.len() {
        // `c1..c2` range form: needs a fourth byte at i+3 with ".." between.
        if i + 3 < chars.len() && chars[i + 1] == b'.' && chars[i + 2] == b'.' {
            let lo = chars[i];
            let hi = chars[i + 3];
            if lo <= hi {
                for b in lo..=hi {
                    mask[b as usize] = true;
                }
                i += 4;
                continue;
            }
        }
        mask[chars[i] as usize] = true;
        i += 1;
    }
    mask
}

/// Shared trim driver. `left`/`right` select which ends are stripped.
fn do_trim(args: &[Zval], ctx: &mut Ctx, fname: &str, left: bool, right: bool) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first()
            .ok_or_else(|| PhpError::Error(format!("{fname}() expects at least 1 argument, 0 given")))?,
        ctx.diags,
    );
    let bytes = s.as_bytes();
    let chars = match args.get(1) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => TRIM_DEFAULT.to_vec(),
    };
    let mask = trim_mask(&chars);
    let mut start = 0;
    let mut end = bytes.len();
    if left {
        while start < end && mask[bytes[start] as usize] {
            start += 1;
        }
    }
    if right {
        while end > start && mask[bytes[end - 1] as usize] {
            end -= 1;
        }
    }
    Ok(Zval::Str(PhpStr::new(bytes[start..end].to_vec())))
}

/// trim($string[, $characters]): strip the charlist from both ends.
pub fn trim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    do_trim(args, ctx, "trim", true, true)
}

/// ltrim($string[, $characters]): strip the charlist from the left.
pub fn ltrim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    do_trim(args, ctx, "ltrim", true, false)
}

/// rtrim($string[, $characters]): strip the charlist from the right.
pub fn rtrim(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    do_trim(args, ctx, "rtrim", false, true)
}

/// First byte index of `needle` in `hay`. Empty needle matches at 0.
fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    if needle.len() > hay.len() {
        return None;
    }
    (0..=hay.len() - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// Split `s` on every (non-overlapping) occurrence of `sep` (sep non-empty).
fn split_all<'a>(s: &'a [u8], sep: &[u8]) -> Vec<&'a [u8]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + sep.len() <= s.len() {
        if &s[i..i + sep.len()] == sep {
            parts.push(&s[start..i]);
            i += sep.len();
            start = i;
        } else {
            i += 1;
        }
    }
    parts.push(&s[start..]);
    parts
}

// --- Step 29-1: pure string builtins ---------------------------------------

/// strrev($string): reverse the bytes (byte-oriented, like PHP).
pub fn strrev(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "strrev")?;
    b.reverse();
    Ok(Zval::Str(PhpStr::new(b)))
}

/// Coerce positional arg `idx` (named `pname`) to bytes for a 2-string builtin.
fn str_at(args: &[Zval], ctx: &mut Ctx, idx: usize, fname: &str, expected: usize) -> Result<Vec<u8>, PhpError> {
    let v = args.get(idx).ok_or_else(|| {
        PhpError::Error(format!(
            "{fname}() expects exactly {expected} arguments, {} given",
            args.len()
        ))
    })?;
    Ok(convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
}

/// str_contains($haystack, $needle): an empty needle is always found.
pub fn str_contains(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = str_at(args, ctx, 0, "str_contains", 2)?;
    let needle = str_at(args, ctx, 1, "str_contains", 2)?;
    let found = needle.is_empty() || find_sub(&haystack, &needle).is_some();
    Ok(Zval::Bool(found))
}

/// str_starts_with($haystack, $needle).
pub fn str_starts_with(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = str_at(args, ctx, 0, "str_starts_with", 2)?;
    let needle = str_at(args, ctx, 1, "str_starts_with", 2)?;
    Ok(Zval::Bool(haystack.starts_with(&needle[..])))
}

/// str_ends_with($haystack, $needle).
pub fn str_ends_with(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = str_at(args, ctx, 0, "str_ends_with", 2)?;
    let needle = str_at(args, ctx, 1, "str_ends_with", 2)?;
    Ok(Zval::Bool(haystack.ends_with(&needle[..])))
}

/// str_split($string, $length = 1): split into `$length`-byte chunks. An empty
/// string yields an empty array (PHP 8.2+); a length < 1 is a `ValueError`.
pub fn str_split(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_arg(args, ctx, "str_split")?;
    let length = match args.get(1) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 1,
    };
    if length < 1 {
        return Err(PhpError::ValueError(
            "str_split(): Argument #2 ($length) must be greater than 0".to_string(),
        ));
    }
    let length = length as usize;
    let mut out = PhpArray::new();
    for chunk in s.chunks(length) {
        let _ = out.append(Zval::Str(PhpStr::new(chunk.to_vec())));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// substr_count($haystack, $needle): count non-overlapping occurrences. An
/// empty needle is a `ValueError`.
pub fn substr_count(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = str_at(args, ctx, 0, "substr_count", 2)?;
    let needle = str_at(args, ctx, 1, "substr_count", 2)?;
    if needle.is_empty() {
        return Err(PhpError::ValueError(
            "substr_count(): Argument #2 ($needle) must not be empty".to_string(),
        ));
    }
    let mut count = 0i64;
    let mut from = 0usize;
    while let Some(pos) = find_sub(&haystack[from..], &needle) {
        count += 1;
        from += pos + needle.len();
    }
    Ok(Zval::Long(count))
}

/// number_format($num, $decimals = 0, $dec_sep = ".", $thousands_sep = ",").
///
/// PHP rounds half away from zero on the *decimal* value the user wrote (so
/// 2.675 -> 2.68, not the binary-truncated 2.67), then groups the integer part
/// in threes. A result that rounds to zero never carries a minus sign.
pub fn number_format(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let num = convert::to_double(args.first().ok_or_else(|| {
        PhpError::Error("number_format() expects at least 1 argument, 0 given".to_string())
    })?);
    let decimals = match args.get(1) {
        Some(v) => convert::to_long_cast(v, ctx.diags).max(0) as usize,
        None => 0,
    };
    let dec_sep = match args.get(2) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b".".to_vec(),
    };
    let thousands_sep = match args.get(3) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b",".to_vec(),
    };

    let (mut negative, int_digits, frac_digits) = round_decimal(num, decimals);
    // -0 prints without a sign.
    if int_digits.iter().all(|&d| d == b'0') && frac_digits.iter().all(|&d| d == b'0') {
        negative = false;
    }

    let mut out = Vec::new();
    if negative {
        out.push(b'-');
    }
    // Group the integer part in threes from the right.
    let n = int_digits.len();
    for (i, d) in int_digits.iter().enumerate() {
        if i > 0 && (n - i) % 3 == 0 {
            out.extend_from_slice(&thousands_sep);
        }
        out.push(*d);
    }
    if decimals > 0 {
        out.extend_from_slice(&dec_sep);
        out.extend_from_slice(&frac_digits);
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Round `value` to `decimals` fractional places (half away from zero) working
/// on its shortest round-trip decimal expansion. Returns `(negative, integer
/// digits, fractional digits)` with the fractional part padded to `decimals`.
fn round_decimal(value: f64, decimals: usize) -> (bool, Vec<u8>, Vec<u8>) {
    if !value.is_finite() {
        return (false, b"0".to_vec(), vec![b'0'; decimals]);
    }
    let negative = value.is_sign_negative() && value != 0.0;
    // Shortest decimal expansion of |value| as integer + fractional digit runs.
    let (mut int_digits, mut frac_digits) = decimal_parts(value.abs());

    if frac_digits.len() > decimals {
        let round_up = frac_digits[decimals] >= b'5';
        frac_digits.truncate(decimals);
        if round_up {
            // Propagate the carry through the fractional then integer digits.
            let mut carry = true;
            for d in frac_digits.iter_mut().rev() {
                if !carry {
                    break;
                }
                if *d == b'9' {
                    *d = b'0';
                } else {
                    *d += 1;
                    carry = false;
                }
            }
            if carry {
                for d in int_digits.iter_mut().rev() {
                    if *d == b'9' {
                        *d = b'0';
                    } else {
                        *d += 1;
                        carry = false;
                        break;
                    }
                }
                if carry {
                    int_digits.insert(0, b'1');
                }
            }
        }
    } else {
        while frac_digits.len() < decimals {
            frac_digits.push(b'0');
        }
    }
    (negative, int_digits, frac_digits)
}

/// |v| as (integer digits, fractional digits) from its shortest round-trip
/// representation. Never uses scientific notation in the output.
fn decimal_parts(v: f64) -> (Vec<u8>, Vec<u8>) {
    debug_assert!(v >= 0.0 && v.is_finite());
    // `{:e}` gives `mantissa e exp`; reposition the point at exp+1.
    let s = format!("{v:e}");
    let (mant, exp) = s.split_once('e').expect("exp format");
    let exp: i32 = exp.parse().expect("exp int");
    let all: Vec<u8> = mant.bytes().filter(|b| *b != b'.').collect();
    let point = exp + 1; // number of integer digits
    let (int_digits, frac_digits) = if point <= 0 {
        let mut frac = vec![b'0'; (-point) as usize];
        frac.extend_from_slice(&all);
        (vec![b'0'], frac)
    } else if (point as usize) >= all.len() {
        let mut int = all.clone();
        int.extend(std::iter::repeat_n(b'0', point as usize - all.len()));
        (int, Vec::new())
    } else {
        let p = point as usize;
        (all[..p].to_vec(), all[p..].to_vec())
    };
    (int_digits, frac_digits)
}

/// `strstr($haystack, $needle, $before_needle = false)` (alias `strchr`): the
/// slice of `$haystack` from the first occurrence of `$needle` to the end, or
/// the part before it when `$before_needle` is true; `false` if not found.
pub fn strstr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("strstr() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("strstr() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let before = matches!(args.get(2), Some(v) if convert::to_bool(v, ctx.diags));
    let hay = haystack.as_bytes();
    match find_sub(hay, needle.as_bytes()) {
        Some(pos) => {
            let part = if before { &hay[..pos] } else { &hay[pos..] };
            Ok(Zval::Str(PhpStr::new(part.to_vec())))
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// `stristr`: case-insensitive `strstr`. The match is located case-insensitively
/// but the returned slice preserves the original casing of `$haystack`.
pub fn stristr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("stristr() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("stristr() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let before = matches!(args.get(2), Some(v) if convert::to_bool(v, ctx.diags));
    let hay = haystack.as_bytes();
    let hay_lc = hay.to_ascii_lowercase();
    let needle_lc = needle.as_bytes().to_ascii_lowercase();
    match find_sub(&hay_lc, &needle_lc) {
        Some(pos) => {
            let part = if before { &hay[..pos] } else { &hay[pos..] };
            Ok(Zval::Str(PhpStr::new(part.to_vec())))
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// `strrchr($haystack, $needle)`: the slice from the *last* occurrence of the
/// first byte of `$needle` to the end of `$haystack`; `false` if not present.
pub fn strrchr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let haystack = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("strrchr() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("strrchr() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let hay = haystack.as_bytes();
    let Some(&ch) = needle.as_bytes().first() else {
        return Ok(Zval::Bool(false));
    };
    match hay.iter().rposition(|&c| c == ch) {
        Some(pos) => Ok(Zval::Str(PhpStr::new(hay[pos..].to_vec()))),
        None => Ok(Zval::Bool(false)),
    }
}

// ---- step 56a: binary / escape / transform string functions ----

/// `bin2hex($string)`: each byte → two lowercase hex digits.
pub fn bin2hex(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let s = str_at(args, ctx, 0, "bin2hex", 1)?;
    let mut out = Vec::with_capacity(s.len() * 2);
    for &b in &s {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `hex2bin($string)`: inverse of `bin2hex`. Odd length or a non-hex byte → false
/// + "Input string must be hexadecimal string" Warning.
pub fn hex2bin(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "hex2bin", 1)?;
    let nibble = |b: u8| -> Option<u8> {
        match b {
            b'0'..=b'9' => Some(b - b'0'),
            b'a'..=b'f' => Some(b - b'a' + 10),
            b'A'..=b'F' => Some(b - b'A' + 10),
            _ => None,
        }
    };
    let bad = |ctx: &mut Ctx| {
        ctx.diags.push(Diag::Warning(
            "hex2bin(): Input string must be hexadecimal string".to_string(),
        ));
    };
    if s.len() % 2 != 0 {
        bad(ctx);
        return Ok(Zval::Bool(false));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for pair in s.chunks(2) {
        match (nibble(pair[0]), nibble(pair[1])) {
            (Some(h), Some(l)) => out.push((h << 4) | l),
            _ => {
                bad(ctx);
                return Ok(Zval::Bool(false));
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `addslashes($string)`: backslash-escape `'`, `"`, `\` and NUL.
pub fn addslashes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "addslashes", 1)?;
    let mut out = Vec::with_capacity(s.len());
    for &b in &s {
        match b {
            b'\'' | b'"' | b'\\' => {
                out.push(b'\\');
                out.push(b);
            }
            0 => {
                out.push(b'\\');
                out.push(b'0');
            }
            _ => out.push(b),
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `stripslashes($string)`: drop one backslash before any char (`\0` → NUL); a
/// trailing lone backslash is removed.
pub fn stripslashes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "stripslashes", 1)?;
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' {
            i += 1;
            if i < s.len() {
                out.push(if s[i] == b'0' { 0 } else { s[i] });
                i += 1;
            }
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `substr_replace($string, $replace, $start, $length = ∞)` (scalar form): splice
/// `$replace` into `$string` over the `[start, start+length)` window; negative
/// start/length count from the end, `length = 0` inserts.
pub fn substr_replace(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "substr_replace", 3)?;
    let repl = str_at(args, ctx, 1, "substr_replace", 3)?;
    let len_i = s.len() as i64;
    let offset = convert::to_long_cast(
        args.get(2).ok_or_else(|| {
            PhpError::Error("substr_replace() expects at least 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    let start = if offset < 0 {
        (len_i + offset).max(0)
    } else {
        offset.min(len_i)
    };
    let end = match args.get(3) {
        None | Some(Zval::Null) => len_i,
        Some(v) => {
            let l = convert::to_long_cast(v, ctx.diags);
            if l < 0 {
                (len_i + l).max(start)
            } else {
                (start + l).min(len_i)
            }
        }
    };
    let (start, end) = (start as usize, end.max(start) as usize);
    let mut out = Vec::with_capacity(s.len() + repl.len());
    out.extend_from_slice(&s[..start]);
    out.extend_from_slice(&repl);
    out.extend_from_slice(&s[end..]);
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `nl2br($string, $use_xhtml = true)`: insert `<br />` (or `<br>`) before each
/// `\n` / `\r\n` / `\r`, keeping the newline.
pub fn nl2br(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "nl2br", 1)?;
    let xhtml = match args.get(1) {
        Some(v) => convert::to_bool(v, ctx.diags),
        None => true,
    };
    let br: &[u8] = if xhtml { b"<br />" } else { b"<br>" };
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            b'\r' => {
                out.extend_from_slice(br);
                out.push(b'\r');
                if s.get(i + 1) == Some(&b'\n') {
                    out.push(b'\n');
                    i += 2;
                } else {
                    i += 1;
                }
            }
            b'\n' => {
                out.extend_from_slice(br);
                out.push(b'\n');
                i += 1;
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `wordwrap($string, $width = 75, $break = "\n", $cut = false)`: greedy
/// word-wrap, breaking long words only when `$cut` is set.
pub fn wordwrap(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "wordwrap", 1)?;
    let width = args
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(75)
        .max(1) as usize;
    let brk = match args.get(2) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b"\n".to_vec(),
    };
    let cut = match args.get(3) {
        Some(v) => convert::to_bool(v, ctx.diags),
        None => false,
    };
    if s.is_empty() || brk.is_empty() {
        return Ok(Zval::Str(PhpStr::new(s)));
    }
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut line_start = 0usize;
    let mut last_space: Option<usize> = None;
    let mut i = 0;
    while i < s.len() {
        if s[i..].starts_with(&brk) {
            out.extend_from_slice(&brk);
            i += brk.len();
            line_start = out.len();
            last_space = None;
            continue;
        }
        let c = s[i];
        out.push(c);
        i += 1;
        if c == b' ' {
            last_space = Some(out.len() - 1);
        }
        if out.len() - line_start > width {
            if let Some(sp) = last_space.filter(|&sp| sp >= line_start) {
                let after = out[sp + 1..].to_vec();
                out.truncate(sp);
                out.extend_from_slice(&brk);
                out.extend_from_slice(&after);
                line_start = sp + brk.len();
                last_space = None;
            } else if cut {
                let last = out.pop().unwrap();
                out.extend_from_slice(&brk);
                out.push(last);
                line_start = out.len() - 1;
                last_space = None;
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}
