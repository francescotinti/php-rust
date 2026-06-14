//! String builtins (plan step 10): implode, explode, substr, ...

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

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
