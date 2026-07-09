//! String builtins (plan step 10): implode, explode, substr, ...

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Diag, Key, PhpArray, PhpError, PhpStr, Zval};

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
                only.type_name_for_error()
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
                        other.type_name_for_error()
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


/// Case-insensitive (ASCII) counterpart of `replace_all`, backing `str_ireplace`.
fn replace_all_ci(s: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if i + from.len() <= s.len() && s[i..i + from.len()].eq_ignore_ascii_case(from) {
            out.extend_from_slice(to);
            i += from.len();
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    out
}

/// str_ireplace($search, $replace, $subject): like `str_replace` but matching is
/// ASCII-case-insensitive. The optional by-reference `$count` is unsupported.
pub fn str_ireplace(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let search = args.first().ok_or_else(|| {
        PhpError::Error("str_ireplace() expects at least 3 arguments, 0 given".to_string())
    })?;
    let replace = args.get(1).ok_or_else(|| {
        PhpError::Error("str_ireplace() expects at least 3 arguments, 1 given".to_string())
    })?;
    let subject = args.get(2).ok_or_else(|| {
        PhpError::Error("str_ireplace() expects at least 3 arguments, 2 given".to_string())
    })?;

    let pairs = replacement_pairs(search, replace, ctx);

    let apply = |subj: &Zval, ctx: &mut Ctx| -> Vec<u8> {
        let mut cur = convert::to_zstr(subj, ctx.diags).as_bytes().to_vec();
        for (s, r) in &pairs {
            if !s.is_empty() {
                cur = replace_all_ci(&cur, s, r);
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

/// `ZEND_THREEWAY_COMPARE`: clamp an ordering to -1 / 0 / 1.
fn normalize_bool(n: i64) -> i64 {
    (n > 0) as i64 - (n < 0) as i64
}

/// Core of the `strcmp` family (`zend_binary_strcmp`/`strncmp` and case
/// variants): compare the first `cap` bytes (all when `None`), case-folded as
/// ASCII when `ci`. Returns the raw byte difference at the first mismatch — PHP
/// surfaces `memcmp`'s / `c1 - c2` value, not a clamped sign — but when every
/// compared byte is equal, the *effective length* difference is normalized to
/// -1/0/1 (`ZEND_THREEWAY_COMPARE`).
fn zend_strcmp(a: &[u8], b: &[u8], cap: Option<usize>, ci: bool) -> i64 {
    let eff_a = cap.map_or(a.len(), |c| a.len().min(c));
    let eff_b = cap.map_or(b.len(), |c| b.len().min(c));
    let n = eff_a.min(eff_b);
    for i in 0..n {
        let (mut c1, mut c2) = (a[i], b[i]);
        if ci {
            c1 = c1.to_ascii_lowercase();
            c2 = c2.to_ascii_lowercase();
        }
        if c1 != c2 {
            return c1 as i64 - c2 as i64;
        }
    }
    normalize_bool(eff_a as i64 - eff_b as i64)
}

fn cmp_arg_str(args: &[Zval], i: usize, name: &str, ctx: &mut Ctx) -> Result<Rc<PhpStr>, PhpError> {
    let v = args.get(i).ok_or_else(|| {
        PhpError::Error(format!("{name}() expects at least {} arguments, {} given", i + 1, args.len()))
    })?;
    Ok(convert::to_zstr(v, ctx.diags))
}

/// `strcmp($s1, $s2)`: binary-safe byte comparison.
pub fn strcmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strcmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strcmp", ctx)?;
    Ok(Zval::Long(zend_strcmp(a.as_bytes(), b.as_bytes(), None, false)))
}

/// `strcasecmp($s1, $s2)`: case-insensitive (ASCII) byte comparison.
pub fn strcasecmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strcasecmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strcasecmp", ctx)?;
    Ok(Zval::Long(zend_strcmp(a.as_bytes(), b.as_bytes(), None, true)))
}

/// `strncmp($s1, $s2, $length)`: compare at most `$length` bytes; a negative
/// length is a `ValueError`.
pub fn strncmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strncmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strncmp", ctx)?;
    let length = convert::to_long_cast(
        args.get(2).ok_or_else(|| {
            PhpError::Error("strncmp() expects exactly 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    if length < 0 {
        return Err(PhpError::ValueError(
            "strncmp(): Argument #3 ($length) must be greater than or equal to 0".to_string(),
        ));
    }
    Ok(Zval::Long(zend_strcmp(a.as_bytes(), b.as_bytes(), Some(length as usize), false)))
}

/// `strncasecmp($s1, $s2, $length)`: case-insensitive `strncmp`.
pub fn strncasecmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strncasecmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strncasecmp", ctx)?;
    let length = convert::to_long_cast(
        args.get(2).ok_or_else(|| {
            PhpError::Error("strncasecmp() expects exactly 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    if length < 0 {
        return Err(PhpError::ValueError(
            "strncasecmp(): Argument #3 ($length) must be greater than or equal to 0".to_string(),
        ));
    }
    Ok(Zval::Long(zend_strcmp(a.as_bytes(), b.as_bytes(), Some(length as usize), true)))
}

/// C `isspace`: space, `\t`, `\n`, `\v`, `\f`, `\r` (`\v` = 0x0B is included,
/// unlike Rust's `is_ascii_whitespace`).
fn nat_isspace(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r')
}

/// A byte at `i`, or 0 (the C null terminator) when `i` is at/past the end —
/// lets the port read "one past the end" as PHP's C code does on its NUL-
/// terminated strings.
fn nat_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `compare_right`: two right-aligned integer runs — the longest run of digits
/// wins; on equal length the greatest value wins (remembered as `bias`).
fn nat_compare_right(a: &[u8], ai: &mut usize, b: &[u8], bi: &mut usize) -> i64 {
    let mut bias = 0i64;
    loop {
        let ad = *ai < a.len() && a[*ai].is_ascii_digit();
        let bd = *bi < b.len() && b[*bi].is_ascii_digit();
        if !ad && !bd {
            return bias;
        } else if !ad {
            return -1;
        } else if !bd {
            return 1;
        } else if a[*ai] < b[*bi] {
            if bias == 0 {
                bias = -1;
            }
        } else if a[*ai] > b[*bi] {
            if bias == 0 {
                bias = 1;
            }
        }
        *ai += 1;
        *bi += 1;
    }
}

/// `compare_left`: two left-aligned digit runs — the first differing digit wins.
fn nat_compare_left(a: &[u8], ai: &mut usize, b: &[u8], bi: &mut usize) -> i64 {
    loop {
        let ad = *ai < a.len() && a[*ai].is_ascii_digit();
        let bd = *bi < b.len() && b[*bi].is_ascii_digit();
        if !ad && !bd {
            return 0;
        } else if !ad {
            return -1;
        } else if !bd {
            return 1;
        } else if a[*ai] < b[*bi] {
            return -1;
        } else if a[*ai] > b[*bi] {
            return 1;
        }
        *ai += 1;
        *bi += 1;
    }
}

/// Natural-order comparison — a faithful port of PHP's `strnatcmp_ex`
/// (`ext/standard/strnatcmp.c`, Martin Pool's algorithm): leading zeros and
/// whitespace are skipped, digit runs compare by numeric magnitude, everything
/// else byte-by-byte (upper-cased when `ci`). Returns -1/0/1.
pub(crate) fn strnatcmp_ex(a: &[u8], b: &[u8], ci: bool) -> i64 {
    if a.is_empty() || b.is_empty() {
        return normalize_bool(a.len() as i64 - b.len() as i64);
    }
    let (mut ap, mut bp) = (0usize, 0usize);
    let mut ca = a[ap];
    let mut cb = b[bp];
    // Skip leading zeros (only when a digit follows, so a bare "0" is kept).
    while ca == b'0' && ap + 1 < a.len() && a[ap + 1].is_ascii_digit() {
        ap += 1;
        ca = a[ap];
    }
    while cb == b'0' && bp + 1 < b.len() && b[bp + 1].is_ascii_digit() {
        bp += 1;
        cb = b[bp];
    }
    loop {
        while nat_isspace(ca) {
            ap += 1;
            ca = nat_at(a, ap);
        }
        while nat_isspace(cb) {
            bp += 1;
            cb = nat_at(b, bp);
        }

        if ca.is_ascii_digit() && cb.is_ascii_digit() {
            let fractional = ca == b'0' || cb == b'0';
            let result = if fractional {
                nat_compare_left(a, &mut ap, b, &mut bp)
            } else {
                nat_compare_right(a, &mut ap, b, &mut bp)
            };
            if result != 0 {
                return result;
            } else if ap == a.len() && bp == b.len() {
                return 0;
            } else if ap == a.len() {
                return -1;
            } else if bp == b.len() {
                return 1;
            } else {
                ca = a[ap];
                cb = b[bp];
            }
        }

        let (mut xa, mut xb) = (ca, cb);
        if ci {
            xa = xa.to_ascii_uppercase();
            xb = xb.to_ascii_uppercase();
        }
        if xa < xb {
            return -1;
        } else if xa > xb {
            return 1;
        }

        ap += 1;
        bp += 1;
        if ap >= a.len() && bp >= b.len() {
            return 0;
        } else if ap >= a.len() {
            return -1;
        } else if bp >= b.len() {
            return 1;
        }
        ca = a[ap];
        cb = b[bp];
    }
}

/// `strnatcmp($s1, $s2)`: natural-order string comparison.
pub fn strnatcmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strnatcmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strnatcmp", ctx)?;
    Ok(Zval::Long(strnatcmp_ex(a.as_bytes(), b.as_bytes(), false)))
}

/// `strnatcasecmp($s1, $s2)`: case-insensitive natural-order comparison.
pub fn strnatcasecmp(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = cmp_arg_str(args, 0, "strnatcasecmp", ctx)?;
    let b = cmp_arg_str(args, 1, "strnatcasecmp", ctx)?;
    Ok(Zval::Long(strnatcmp_ex(a.as_bytes(), b.as_bytes(), true)))
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

/// str_rot13($string): ROT13 each ASCII letter, other bytes untouched.
pub fn str_rot13(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut b = str_arg(args, ctx, "str_rot13")?;
    for c in b.iter_mut() {
        match *c {
            b'a'..=b'z' => *c = b'a' + (*c - b'a' + 13) % 26,
            b'A'..=b'Z' => *c = b'A' + (*c - b'A' + 13) % 26,
            _ => {}
        }
    }
    Ok(Zval::Str(PhpStr::new(b)))
}

/// Coerce positional arg `idx` (named `pname`) to bytes for a 2-string builtin.
pub(crate) fn str_at(args: &[Zval], ctx: &mut Ctx, idx: usize, fname: &str, expected: usize) -> Result<Vec<u8>, PhpError> {
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
    // `$offset` / `$length` window the haystack (negatives count from the end;
    // an out-of-range offset is a ValueError; SQLServerPlatform's paren
    // balancing counts from the ORDER BY offset).
    let hlen = haystack.len() as i64;
    let mut start = match args.get(2) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 0,
    };
    if start < 0 {
        start += hlen;
    }
    if start < 0 || start > hlen {
        return Err(PhpError::ValueError(
            "substr_count(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
                .to_string(),
        ));
    }
    let mut end = match args.get(3) {
        Some(Zval::Null) | None => hlen,
        Some(v) => {
            let mut l = convert::to_long_cast(v, ctx.diags);
            if l < 0 {
                l += hlen - start;
            }
            if l < 0 || start + l > hlen {
                return Err(PhpError::ValueError(
                    "substr_count(): Argument #4 ($length) must be contained in argument #1 ($haystack)".to_string(),
                ));
            }
            start + l
        }
    };
    if end < start {
        end = start;
    }
    let window = &haystack[start as usize..end as usize];
    let mut count = 0i64;
    let mut from = 0usize;
    while from < window.len() {
        match find_sub(&window[from..], &needle) {
            Some(pos) => {
                count += 1;
                from += pos + needle.len();
            }
            None => break,
        }
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

/// `random_bytes($length)`: `$length` cryptographically-secure random bytes from
/// the OS CSPRNG (`/dev/urandom`). `$length < 1` is a `ValueError`; a source
/// failure is an `Error` (PHP raises `Random\RandomException`, which we model as a
/// plain `Error` until that class exists).
pub fn random_bytes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::io::Read;
    let len = convert::to_long_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("random_bytes() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    if len < 1 {
        return Err(PhpError::ValueError(
            "random_bytes(): Argument #1 ($length) must be greater than 0".to_string(),
        ));
    }
    let mut buf = vec![0u8; len as usize];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf))
        .map_err(|_| PhpError::Error("Cannot open source device".to_string()))?;
    Ok(Zval::Str(PhpStr::new(buf)))
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

/// `addcslashes($string, $characters)`: backslash-escape every byte of
/// `$string` that falls in the `$characters` set, C-style. The set is expanded
/// by `php_charmask` rules (ext/standard/string.c): `a..z` is an incrementing
/// range; a malformed range (decrementing, or `..` at either end) is taken
/// literally with the same warnings PHP raises. An escaped byte outside
/// 32..=126 becomes the C mnemonic (`\n`, `\t`, `\r`, `\a`, `\v`, `\b`, `\f`)
/// or a 3-digit octal `\ooo`; a printable one is just backslash-prefixed.
pub fn addcslashes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "addcslashes", 2)?;
    let charlist = str_at(args, ctx, 1, "addcslashes", 2)?;
    // php_charmask: expand the charlist (with `a..z` ranges) into a byte set.
    let mut mask = [false; 256];
    let cl = &charlist[..];
    let mut i = 0;
    while i < cl.len() {
        let c = cl[i];
        // A valid range needs `x..y` with y >= x and both ends present.
        if i + 3 < cl.len() && cl[i + 1] == b'.' && cl[i + 2] == b'.' {
            let end = cl[i + 3];
            if end >= c {
                for b in c..=end {
                    mask[b as usize] = true;
                }
                i += 4;
                continue;
            }
            ctx.diags.push(Diag::Warning(
                "addcslashes(): Invalid '..'-range, '..'-range needs to be incrementing".into(),
            ));
            // fall through: the bytes are taken literally
        } else if i + 1 < cl.len() && cl[i + 1] == b'.' && cl[i + 2..].first() == Some(&b'.') {
            // `x..` at the very end: literal, with PHP's warning
            ctx.diags.push(Diag::Warning("addcslashes(): Invalid '..'-range".into()));
        }
        mask[c as usize] = true;
        i += 1;
    }
    let mut out = Vec::with_capacity(s.len());
    for &b in &s {
        if mask[b as usize] {
            if !(32..=126).contains(&b) {
                out.push(b'\\');
                match b {
                    b'\n' => out.push(b'n'),
                    b'\t' => out.push(b't'),
                    b'\r' => out.push(b'r'),
                    7 => out.push(b'a'),
                    11 => out.push(b'v'),
                    8 => out.push(b'b'),
                    12 => out.push(b'f'),
                    _ => out.extend_from_slice(format!("{:03o}", b).as_bytes()),
                }
            } else {
                out.push(b'\\');
                out.push(b);
            }
        } else {
            out.push(b);
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

/// `substr_compare($haystack, $needle, $offset, ?$length = null, $case_insensitive = false): int`
/// — compare `$needle` against `$haystack` starting at `$offset` (negative counts
/// from the end), over at most `$length` bytes (default: the longer of the two
/// remaining lengths). Mirrors `zend_binary_strncmp_l`: a byte `memcmp` of the
/// common span, else the length difference. An offset past the end, or a negative
/// `$length`, is a `ValueError` (PHP 8).
pub fn substr_compare(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let main = str_at(args, ctx, 0, "substr_compare", 3)?;
    let needle = str_at(args, ctx, 1, "substr_compare", 3)?;
    let offset_v = args.get(2).ok_or_else(|| {
        PhpError::ArgumentCountError(format!(
            "substr_compare() expects at least 3 arguments, {} given",
            args.len()
        ))
    })?;
    let mut offset = convert::to_long_cast(offset_v, ctx.diags);
    let (length, length_is_default) = match args.get(3) {
        None | Some(Zval::Null) => (0i64, true),
        Some(v) => (convert::to_long_cast(v, ctx.diags), false),
    };
    let case_insensitive = args.get(4).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);

    if !length_is_default && length < 0 {
        return Err(PhpError::ValueError(
            "substr_compare(): Argument #4 ($length) must be greater than or equal to 0".into(),
        ));
    }
    let main_len = main.len() as i64;
    if offset < 0 {
        offset = (main_len + offset).max(0);
    }
    if offset > main_len {
        return Err(PhpError::ValueError(
            "substr_compare(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
                .into(),
        ));
    }
    let s1 = &main[offset as usize..];
    let len1 = s1.len();
    let len2 = needle.len();
    let cmp_len = if length_is_default { len1.max(len2) } else { length as usize };
    let n = cmp_len.min(len1).min(len2);
    let mut diff = 0i64;
    for i in 0..n {
        let (a, b) = if case_insensitive {
            (s1[i].to_ascii_lowercase(), needle[i].to_ascii_lowercase())
        } else {
            (s1[i], needle[i])
        };
        if a != b {
            diff = a as i64 - b as i64;
            break;
        }
    }
    // A byte mismatch yields the raw byte difference (like `memcmp`); an
    // all-equal common span yields the *normalized* length difference
    // (`ZEND_NORMALIZE_BOOL`: -1 / 0 / 1).
    let result = if diff == 0 {
        (cmp_len.min(len1) as i64 - cmp_len.min(len2) as i64).signum()
    } else {
        diff
    };
    Ok(Zval::Long(result))
}

/// One hex digit's value (caller guarantees it is `[0-9a-fA-F]`).
fn hexval(b: u8) -> u32 {
    match b {
        b'0'..=b'9' => (b - b'0') as u32,
        b'a'..=b'f' => (b - b'a' + 10) as u32,
        b'A'..=b'F' => (b - b'A' + 10) as u32,
        _ => 0,
    }
}

/// `stripcslashes($string)`: inverse of `addcslashes` — decode C-style escapes
/// (`\n \t \r \a \v \b \f \\`), octal `\ooo` (1-3 digits) and hex `\xHH` (1-2
/// digits). Any other `\c` drops the backslash and keeps `c`; a trailing lone
/// backslash is preserved. Mirrors `php_stripcslashes`.
pub fn stripcslashes(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "stripcslashes", 1)?;
    let end = s.len();
    let mut out = Vec::with_capacity(end);
    let mut i = 0usize;
    while i < end {
        if s[i] == b'\\' && i + 1 < end {
            i += 1; // consume backslash; now at the escape char
            match s[i] {
                b'n' => { out.push(b'\n'); i += 1; }
                b't' => { out.push(b'\t'); i += 1; }
                b'r' => { out.push(b'\r'); i += 1; }
                b'a' => { out.push(0x07); i += 1; }
                b'v' => { out.push(0x0B); i += 1; }
                b'b' => { out.push(0x08); i += 1; }
                b'f' => { out.push(0x0C); i += 1; }
                b'\\' => { out.push(b'\\'); i += 1; }
                b'x' if i + 1 < end && s[i + 1].is_ascii_hexdigit() => {
                    let mut val = hexval(s[i + 1]);
                    i += 2; // past 'x' and first hex digit
                    if i < end && s[i].is_ascii_hexdigit() {
                        val = val * 16 + hexval(s[i]);
                        i += 1;
                    }
                    out.push(val as u8);
                }
                _ => {
                    let mut digits = 0;
                    let mut val: u32 = 0;
                    while digits < 3 && i < end && (b'0'..=b'7').contains(&s[i]) {
                        val = val * 8 + (s[i] - b'0') as u32;
                        i += 1;
                        digits += 1;
                    }
                    if digits > 0 {
                        out.push(val as u8);
                    } else {
                        out.push(s[i]); // unrecognized escape: keep the char verbatim
                        i += 1;
                    }
                }
            }
        } else {
            out.push(s[i]);
            i += 1;
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `str_shuffle(string $string): string` — a random permutation of the bytes via
/// the Fisher-Yates shuffle in `php_binary_string_shuffle`, drawing from the same
/// Mt19937 state as `mt_rand`/`mt_srand` (so a seeded run is byte-reproducible).
pub fn str_shuffle(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut bytes = str_at(args, ctx, 0, "str_shuffle", 1)?;
    let mut n_left = bytes.len();
    while n_left > 1 {
        n_left -= 1;
        let rnd = crate::math::mt_range(n_left as u32) as usize;
        if rnd != n_left {
            bytes.swap(n_left, rnd);
        }
    }
    Ok(Zval::Str(PhpStr::new(bytes)))
}

/// `str_increment(string $string): string` — the next value in the alphanumeric
/// "Perl string increment" sequence (`"Az"`→`"Ba"`, `"Zz"`→`"AAa"`, `"a9"`→`"b0"`).
/// Empty or non-alphanumeric-ASCII input is a `ValueError`. Mirrors the 8.3 C.
pub fn str_increment(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "str_increment", 1)?;
    if s.is_empty() {
        return Err(PhpError::ValueError(
            "str_increment(): Argument #1 ($string) must not be empty".into(),
        ));
    }
    if !s.iter().all(u8::is_ascii_alphanumeric) {
        return Err(PhpError::ValueError(
            "str_increment(): Argument #1 ($string) must be composed only of alphanumeric ASCII characters".into(),
        ));
    }
    let mut buf = s.clone();
    let mut pos = buf.len() - 1;
    let mut carry;
    loop {
        let c = buf[pos];
        if c != b'z' && c != b'Z' && c != b'9' {
            carry = false;
            buf[pos] += 1;
        } else {
            carry = true;
            buf[pos] = if c == b'9' { b'0' } else { c - 25 };
        }
        if !carry || pos == 0 {
            break;
        }
        pos -= 1;
    }
    if carry {
        let lead = if buf[0] == b'0' { b'1' } else { buf[0] };
        let mut out = Vec::with_capacity(buf.len() + 1);
        out.push(lead);
        out.extend_from_slice(&buf);
        return Ok(Zval::Str(PhpStr::new(out)));
    }
    Ok(Zval::Str(PhpStr::new(buf)))
}

/// `str_decrement(string $string): string` — the inverse of `str_increment`
/// (`"B0"`→`"A9"`, `"aaa"`→`"zz"`). Empty/non-alphanumeric input, a leading `0`,
/// or decrementing below the range is a `ValueError`. Mirrors the 8.3 C.
pub fn str_decrement(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "str_decrement", 1)?;
    if s.is_empty() {
        return Err(PhpError::ValueError(
            "str_decrement(): Argument #1 ($string) must not be empty".into(),
        ));
    }
    if !s.iter().all(u8::is_ascii_alphanumeric) {
        return Err(PhpError::ValueError(
            "str_decrement(): Argument #1 ($string) must be composed only of alphanumeric ASCII characters".into(),
        ));
    }
    let out_of_range = || {
        PhpError::ValueError(format!(
            "str_decrement(): Argument #1 ($string) \"{}\" is out of decrement range",
            String::from_utf8_lossy(&s)
        ))
    };
    if s[0] == b'0' {
        return Err(out_of_range());
    }
    let mut buf = s.clone();
    let mut pos = buf.len() - 1;
    let mut carry;
    loop {
        let c = buf[pos];
        if c != b'a' && c != b'A' && c != b'0' {
            carry = false;
            buf[pos] -= 1;
        } else {
            carry = true;
            buf[pos] = if c == b'0' { b'9' } else { c + 25 };
        }
        if !carry || pos == 0 {
            break;
        }
        pos -= 1;
    }
    if carry || (buf[0] == b'0' && buf.len() > 1) {
        if buf.len() == 1 {
            return Err(out_of_range());
        }
        return Ok(Zval::Str(PhpStr::new(buf[1..].to_vec())));
    }
    Ok(Zval::Str(PhpStr::new(buf)))
}

/// `count_chars(string $string, int $mode = 0)` — per-byte frequency: mode 0 all
/// 256 bytes, 1 only present, 2 only absent, 3 present bytes as a string, 4 absent
/// bytes as a string.
pub fn count_chars(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "count_chars", 1)?;
    let mode = args.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !(0..=4).contains(&mode) {
        return Err(PhpError::ValueError(
            "count_chars(): Argument #2 ($mode) must be between 0 and 4 (inclusive)".into(),
        ));
    }
    let mut counts = [0i64; 256];
    for &b in &s {
        counts[b as usize] += 1;
    }
    if mode >= 3 {
        let want = mode == 3;
        let mut out = Vec::new();
        for (i, &n) in counts.iter().enumerate() {
            if (n != 0) == want {
                out.push(i as u8);
            }
        }
        return Ok(Zval::Str(PhpStr::new(out)));
    }
    let mut arr = PhpArray::new();
    for (i, &n) in counts.iter().enumerate() {
        let keep = match mode {
            0 => true,
            1 => n != 0,
            _ => n == 0,
        };
        if keep {
            arr.insert(Key::Int(i as i64), Zval::Long(n));
        }
    }
    Ok(Zval::Array(Rc::new(arr)))
}

/// Expand a `str_word_count` char-list into a 256-entry membership mask, honouring
/// `a..z` ranges (php_charmask).
fn word_charmask(list: &[u8]) -> [bool; 256] {
    let mut mask = [false; 256];
    let mut i = 0;
    while i < list.len() {
        let c = list[i];
        if i + 3 < list.len() && list[i + 1] == b'.' && list[i + 2] == b'.' && list[i + 3] >= c {
            for b in c..=list[i + 3] {
                mask[b as usize] = true;
            }
            i += 4;
            continue;
        }
        mask[c as usize] = true;
        i += 1;
    }
    mask
}

/// `str_word_count(string $string, int $format = 0, ?string $characters = null)`
/// — a word is a run of ASCII letters (plus `'`/`-`, never leading, `-` never
/// trailing) and any extra `$characters`. Format 0 counts, 1 lists the words, 2
/// maps byte-offset → word.
pub fn str_word_count(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "str_word_count", 1)?;
    let format = args.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !matches!(format, 0 | 1 | 2) {
        return Err(PhpError::ValueError(
            "str_word_count(): Argument #2 ($format) must be a valid format value".into(),
        ));
    }
    let (mask, has_list) = match args.get(2) {
        None | Some(Zval::Null) => ([false; 256], false),
        Some(v) => (word_charmask(convert::to_zstr(v, ctx.diags).as_bytes()), true),
    };
    if s.is_empty() {
        return Ok(if format == 0 { Zval::Long(0) } else { Zval::Array(Rc::new(PhpArray::new())) });
    }
    let is_word = |c: u8| {
        c.is_ascii_alphabetic() || (has_list && mask[c as usize]) || c == b'\'' || c == b'-'
    };
    let mut p = 0usize;
    let mut e = s.len();
    // A leading '/- (unless allowed) and a trailing - (unless allowed) are excluded.
    if (s[p] == b'\'' && !(has_list && mask[b'\'' as usize]))
        || (s[p] == b'-' && !(has_list && mask[b'-' as usize]))
    {
        p += 1;
    }
    if s[e - 1] == b'-' && !(has_list && mask[b'-' as usize]) {
        e -= 1;
    }
    let mut arr = PhpArray::new();
    let mut count = 0i64;
    while p < e {
        let start = p;
        while p < e && is_word(s[p]) {
            p += 1;
        }
        if p > start {
            match format {
                1 => {
                    let _ = arr.append(Zval::Str(PhpStr::new(s[start..p].to_vec())));
                }
                2 => {
                    arr.insert(Key::Int(start as i64), Zval::Str(PhpStr::new(s[start..p].to_vec())));
                }
                _ => count += 1,
            }
        }
        p += 1;
    }
    Ok(if format == 0 { Zval::Long(count) } else { Zval::Array(Rc::new(arr)) })
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

// ---- step 57a: search family (rpos / case-insensitive) + span -------------

/// Coerce the first two positional args of a `($haystack, $needle, …)` builtin
/// to bytes, mirroring PHP's "expects at least 2 arguments, N given" fatal.
fn haystack_needle(
    args: &[Zval],
    ctx: &mut Ctx,
    fname: &str,
) -> Result<(Vec<u8>, Vec<u8>), PhpError> {
    let hay = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error(format!("{fname}() expects at least 2 arguments, 0 given"))
        })?,
        ctx.diags,
    )
    .as_bytes()
    .to_vec();
    let needle = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error(format!("{fname}() expects at least 2 arguments, 1 given"))
        })?,
        ctx.diags,
    )
    .as_bytes()
    .to_vec();
    Ok((hay, needle))
}

fn offset_value_error(fname: &str) -> PhpError {
    PhpError::ValueError(format!(
        "{fname}(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
    ))
}

/// Index of the **last** occurrence of `needle` whose start lies in `[lo, hi]`
/// (inclusive); `hi` is clamped to the last position where `needle` still fits.
/// An empty needle matches at the clamped `hi`.
fn rfind_window(hay: &[u8], needle: &[u8], lo: usize, hi: usize) -> Option<usize> {
    let max_start = hay.len().checked_sub(needle.len())?;
    let hi = hi.min(max_start);
    if hi < lo {
        return None;
    }
    (lo..=hi).rev().find(|&i| &hay[i..i + needle.len()] == needle)
}

/// Resolve a `strrpos`-style `$offset` into the inclusive start-position window
/// `[lo, hi]`, or `Ok(None)` when the window is empty. Out-of-range is a fatal.
///
/// `offset >= 0` searches starts at or after `offset`; `offset < 0` searches
/// starts whose end is at or before `len + offset` (unless the needle is longer
/// than `-offset`, in which case the whole string is scanned).
fn rpos_window(
    len: i64,
    nlen: i64,
    offset: i64,
    fname: &str,
) -> Result<Option<(usize, usize)>, PhpError> {
    let (lo, hi) = if offset >= 0 {
        if offset > len {
            return Err(offset_value_error(fname));
        }
        (offset, len - nlen)
    } else {
        if offset < -len {
            return Err(offset_value_error(fname));
        }
        let hi = if nlen > -offset { len - nlen } else { len + offset };
        (0, hi)
    };
    if hi < lo || hi < 0 {
        return Ok(None);
    }
    Ok(Some((lo as usize, hi as usize)))
}

/// strrpos($haystack, $needle[, $offset]): byte index of the last occurrence, or
/// `false`. See [`rpos_window`] for the `$offset` semantics.
pub fn strrpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (hay, needle) = haystack_needle(args, ctx, "strrpos")?;
    let offset = args.get(2).map_or(0, |v| convert::to_long_cast(v, ctx.diags));
    match rpos_window(hay.len() as i64, needle.len() as i64, offset, "strrpos")? {
        Some((lo, hi)) => Ok(match rfind_window(&hay, &needle, lo, hi) {
            Some(p) => Zval::Long(p as i64),
            None => Zval::Bool(false),
        }),
        None => Ok(Zval::Bool(false)),
    }
}

/// stripos($haystack, $needle[, $offset]): case-insensitive [`strpos`].
pub fn stripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (hay, needle) = haystack_needle(args, ctx, "stripos")?;
    let len = hay.len() as i64;
    let offset = args.get(2).map_or(0, |v| convert::to_long_cast(v, ctx.diags));
    let start = if offset < 0 { len + offset } else { offset };
    if start < 0 || start > len {
        return Err(offset_value_error("stripos"));
    }
    let hay_lc = hay.to_ascii_lowercase();
    let needle_lc = needle.to_ascii_lowercase();
    match find_sub(&hay_lc[start as usize..], &needle_lc) {
        Some(pos) => Ok(Zval::Long(start + pos as i64)),
        None => Ok(Zval::Bool(false)),
    }
}

/// strripos($haystack, $needle[, $offset]): case-insensitive [`strrpos`].
pub fn strripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (hay, needle) = haystack_needle(args, ctx, "strripos")?;
    let offset = args.get(2).map_or(0, |v| convert::to_long_cast(v, ctx.diags));
    match rpos_window(hay.len() as i64, needle.len() as i64, offset, "strripos")? {
        Some((lo, hi)) => {
            let hay_lc = hay.to_ascii_lowercase();
            let needle_lc = needle.to_ascii_lowercase();
            Ok(match rfind_window(&hay_lc, &needle_lc, lo, hi) {
                Some(p) => Zval::Long(p as i64),
                None => Zval::Bool(false),
            })
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// Resolve the `$start` / `$length` arguments of `strspn`/`strcspn` into a byte
/// slice of `$s`, applying PHP's negative-counts-from-end + clamping rules.
fn span_slice<'a>(s: &'a [u8], args: &[Zval], ctx: &mut Ctx) -> &'a [u8] {
    let slen = s.len() as i64;
    let mut start = args.get(2).map_or(0, |v| convert::to_long_cast(v, ctx.diags));
    if start < 0 {
        start += slen;
        if start < 0 {
            start = 0;
        }
    } else if start > slen {
        start = slen;
    }
    let avail = slen - start;
    let len = match args.get(3) {
        None | Some(Zval::Null) => avail,
        Some(v) => {
            let mut l = convert::to_long_cast(v, ctx.diags);
            if l < 0 {
                l += avail;
                if l < 0 {
                    l = 0;
                }
            } else if l > avail {
                l = avail;
            }
            l
        }
    };
    let start = start as usize;
    &s[start..start + len as usize]
}

/// Set of bytes present in `mask`, for the span scanners.
fn byte_set(mask: &[u8]) -> [bool; 256] {
    let mut set = [false; 256];
    for &b in mask {
        set[b as usize] = true;
    }
    set
}

/// strspn($s, $mask[, $start[, $length]]): length of the initial segment of the
/// `[start, length)` window of `$s` made up solely of bytes present in `$mask`.
pub fn strspn(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (s, mask) = haystack_needle(args, ctx, "strspn")?;
    let set = byte_set(&mask);
    let window = span_slice(&s, args, ctx);
    let n = window.iter().take_while(|&&b| set[b as usize]).count();
    Ok(Zval::Long(n as i64))
}

/// strcspn($s, $mask[, $start[, $length]]): length of the initial segment of the
/// window of `$s` made up of bytes **not** present in `$mask`.
pub fn strcspn(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (s, mask) = haystack_needle(args, ctx, "strcspn")?;
    let set = byte_set(&mask);
    let window = span_slice(&s, args, ctx);
    let n = window.iter().take_while(|&&b| !set[b as usize]).count();
    Ok(Zval::Long(n as i64))
}

/// escapeshellarg($arg): wrap in single quotes with embedded `'` escaped as
/// `'\''` (php_escape_shell_arg, unix branch). A NUL byte is a ValueError.
pub fn escapeshellarg(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("escapeshellarg() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let bytes = s.as_bytes();
    if bytes.contains(&0) {
        return Err(PhpError::ValueError(
            "escapeshellarg(): Argument #1 ($arg) must not contain any null bytes".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(bytes.len() + 2);
    out.push(b'\'');
    for &b in bytes {
        if b == b'\'' {
            out.extend_from_slice(b"'\\''");
        } else {
            out.push(b);
        }
    }
    out.push(b'\'');
    Ok(Zval::Str(PhpStr::new(out)))
}

/// escapeshellcmd($command): backslash-escape the shell metacharacters
/// (php_escape_shell_cmd, unix branch); unmatched quotes are escaped too,
/// paired `'`/`"` pass through. A NUL byte is a ValueError.
pub fn escapeshellcmd(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("escapeshellcmd() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let bytes = s.as_bytes();
    if bytes.contains(&0) {
        return Err(PhpError::ValueError(
            "escapeshellcmd(): Argument #1 ($command) must not contain any null bytes".to_string(),
        ));
    }
    let squotes = bytes.iter().filter(|&&b| b == b'\'').count();
    let dquotes = bytes.iter().filter(|&&b| b == b'"').count();
    let mut out = Vec::with_capacity(bytes.len());
    for &b in bytes {
        match b {
            b'\'' if squotes % 2 == 0 => out.push(b),
            b'"' if dquotes % 2 == 0 => out.push(b),
            b'#' | b'&' | b';' | b'`' | b'|' | b'*' | b'?' | b'~' | b'<' | b'>' | b'^'
            | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'$' | b'\\' | b'\x0A' | b'\xFF'
            | b'\'' | b'"' => {
                out.push(b'\\');
                out.push(b);
            }
            _ => out.push(b),
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// strpbrk($string, $characters): the substring starting at the first byte of
/// `$string` present in `$characters`, or `false` when none is. An empty
/// `$characters` is a ValueError (PHP 8).
pub fn strpbrk(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (s, chars) = haystack_needle(args, ctx, "strpbrk")?;
    if chars.is_empty() {
        return Err(PhpError::ValueError(
            "strpbrk(): Argument #2 ($characters) must be a non-empty string".to_string(),
        ));
    }
    let set = byte_set(&chars);
    match s.iter().position(|&b| set[b as usize]) {
        Some(i) => Ok(Zval::Str(PhpStr::new(s[i..].to_vec()))),
        None => Ok(Zval::Bool(false)),
    }
}

// ---- step 57b: strtr + chunk_split ----------------------------------------

/// strtr($string, $from, $to): per-byte translation table, mapping `from[i]` to
/// `to[i]` for `i` in `0..min(len(from), len(to))`. Bytes absent from the table
/// pass through unchanged; a duplicate source byte takes its last mapping.
fn strtr_pairs(s: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let n = from.len().min(to.len());
    let mut table: [u8; 256] = core::array::from_fn(|i| i as u8);
    for i in 0..n {
        table[from[i] as usize] = to[i];
    }
    s.iter().map(|&b| table[b as usize]).collect()
}

/// strtr($string, $map): substring replacement, longest key first, scanning
/// left to right without re-scanning emitted output. Integer keys take their
/// decimal-string form; an empty key is ignored (with a Warning).
fn strtr_array(s: &[u8], map: &PhpArray, ctx: &mut Ctx) -> Vec<u8> {
    // PHP short-circuits an empty subject before touching the map, so the
    // empty-key Warning never fires for `strtr("", [...])`.
    if s.is_empty() {
        return Vec::new();
    }
    let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for (k, v) in map.iter() {
        let key = match k {
            Key::Int(i) => i.to_string().into_bytes(),
            Key::Str(s) => s.as_bytes().to_vec(),
        };
        if key.is_empty() {
            ctx.diags.push(Diag::Warning(
                "strtr(): Ignoring replacement of empty string".to_string(),
            ));
            continue;
        }
        let val = convert::to_zstr(v, ctx.diags).as_bytes().to_vec();
        pairs.push((key, val));
    }
    // Longest key first; the stable sort keeps insertion order among equal lengths.
    pairs.sort_by_key(|p| core::cmp::Reverse(p.0.len()));

    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let hit = pairs.iter().find(|(k, _)| s[i..].starts_with(k));
        match hit {
            Some((k, val)) => {
                out.extend_from_slice(val);
                i += k.len();
            }
            None => {
                out.push(s[i]);
                i += 1;
            }
        }
    }
    out
}

/// strtr($string, $from, $to) | strtr($string, $map): translate characters or
/// substrings. The two-argument form requires an array map.
pub fn strtr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let subject = str_at(args, ctx, 0, "strtr", 2)?;
    if args.len() == 2 {
        return match args.get(1) {
            Some(Zval::Array(map)) => Ok(Zval::Str(PhpStr::new(strtr_array(&subject, map, ctx)))),
            _ => Err(PhpError::TypeError(
                "strtr(): Argument #2 ($from) must be of type array, string given".to_string(),
            )),
        };
    }
    let from = str_at(args, ctx, 1, "strtr", 3)?;
    let to = str_at(args, ctx, 2, "strtr", 3)?;
    Ok(Zval::Str(PhpStr::new(strtr_pairs(&subject, &from, &to))))
}

/// chunk_split($string, $length = 76, $separator = "\r\n"): insert `$separator`
/// after every `$length`-byte chunk, including a trailing one after the final
/// (possibly short) chunk. An empty `$string` still yields one separator.
pub fn chunk_split(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "chunk_split", 1)?;
    let len = args.get(1).map_or(76, |v| convert::to_long_cast(v, ctx.diags));
    if len < 1 {
        return Err(PhpError::ValueError(
            "chunk_split(): Argument #2 ($length) must be greater than 0".to_string(),
        ));
    }
    let sep = match args.get(2) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => b"\r\n".to_vec(),
    };
    let len = len as usize;
    let mut out = Vec::with_capacity(s.len() + sep.len());
    let mut i = 0;
    while i + len <= s.len() {
        out.extend_from_slice(&s[i..i + len]);
        out.extend_from_slice(&sep);
        i += len;
    }
    if i < s.len() {
        out.extend_from_slice(&s[i..]);
        out.extend_from_slice(&sep);
    } else if s.is_empty() {
        out.extend_from_slice(&sep);
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

// ---- step 57c: strip_tags + quotemeta + levenshtein ----------------------

/// The C `isspace` set, for the "`<` not followed by a tag char" rule (note it
/// includes the vertical tab `\x0b`, which Rust's `is_ascii_whitespace` omits).
fn is_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// strip_tags($string): remove `<...>` tag spans (HTML / PHP / comments),
/// keeping the surrounding text. Faithful to PHP's scanner:
/// - a `<` followed by whitespace (or EOF-as-non-tag) stays literal;
/// - inside a normal tag, nested `<` raise a depth that `>` must balance, and a
///   quote (`"`/`'`) suppresses `<`/`>` until closed;
/// - `<!-- ... -->` is a comment whose closing `-->` may reuse the opener
///   dashes (so `<!-->` is an empty comment); `<! ...>` runs to the next `>`;
/// - `<? ... ?>` runs to `?>`.
///
/// The `$allowed_tags` argument is **not** honoured (scope-out D-57.1): every
/// tag is stripped regardless.
pub fn strip_tags(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "strip_tags", 1)?;
    let n = s.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        if s[i] != b'<' {
            out.push(s[i]);
            i += 1;
            continue;
        }
        match s.get(i + 1) {
            // `<` followed by whitespace is a literal `<`.
            Some(&b) if is_c_space(b) => {
                out.push(b'<');
                i += 1;
            }
            // `<? ... ?>` (PHP / processing instruction).
            Some(b'?') => {
                i = find_sub(&s[i + 2..], b"?>").map_or(n, |p| i + 2 + p + 2);
            }
            // `<!-- ... -->` comment, or `<! ...>` declaration.
            Some(b'!') => {
                let rest = &s[i + 2..];
                i = if rest.starts_with(b"--") {
                    find_sub(rest, b"-->").map_or(n, |p| i + 2 + p + 3)
                } else {
                    find_sub(rest, b">").map_or(n, |p| i + 2 + p + 1)
                };
            }
            // A normal tag (or a `<` at EOF): balance `<`-depth, track quotes.
            _ => {
                let mut depth = 1usize;
                let mut quote = 0u8;
                i += 1;
                while i < n && depth > 0 {
                    let c = s[i];
                    if quote != 0 {
                        if c == quote {
                            quote = 0;
                        }
                    } else {
                        match c {
                            b'"' | b'\'' => quote = c,
                            b'<' => depth += 1,
                            b'>' => depth -= 1,
                            _ => {}
                        }
                    }
                    i += 1;
                }
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// quotemeta($string): backslash-escape the regex metacharacters
/// `. \ + * ? [ ^ ] $ ( )`.
pub fn quotemeta(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = str_at(args, ctx, 0, "quotemeta", 1)?;
    let mut out = Vec::with_capacity(s.len());
    for &b in &s {
        if matches!(
            b,
            b'.' | b'\\' | b'+' | b'*' | b'?' | b'[' | b'^' | b']' | b'$' | b'(' | b')'
        ) {
            out.push(b'\\');
        }
        out.push(b);
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// levenshtein($string1, $string2): byte-oriented edit distance with unit
/// insert/replace/delete costs (the default two-argument form). The weighted
/// five-argument form is not implemented (scope-out D-57.2).
pub fn levenshtein(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let a = str_at(args, ctx, 0, "levenshtein", 2)?;
    let b = str_at(args, ctx, 1, "levenshtein", 2)?;
    let lb = b.len();
    if a.is_empty() {
        return Ok(Zval::Long(lb as i64));
    }
    if lb == 0 {
        return Ok(Zval::Long(a.len() as i64));
    }
    let mut prev: Vec<usize> = (0..=lb).collect();
    let mut cur = vec![0usize; lb + 1];
    for (i, &ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    Ok(Zval::Long(prev[lb] as i64))
}


// ---------------------------------------------------------------------------
// version_compare() — faithful port of ext/standard/versioning.c.
// ---------------------------------------------------------------------------

#[inline]
fn vc_normalize(x: i64) -> i64 {
    match x.cmp(&0) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => 0,
    }
}

/// `php_canonicalize_version()`: collapse separators to `.` and insert a `.` at
/// every digit<->non-digit boundary, so each `.`-delimited segment is
/// homogeneous (all digits or all non-digits).
fn canonicalize_version(version: &[u8]) -> Vec<u8> {
    if version.is_empty() {
        return Vec::new();
    }
    let isdigit = |x: u8| x.is_ascii_digit();
    let isdig = |x: u8| isdigit(x) && x != b'.';
    let isndig = |x: u8| !isdigit(x) && x != b'.';
    let isspecial = |x: u8| x == b'-' || x == b'_' || x == b'+';
    let mut q: Vec<u8> = Vec::with_capacity(version.len() * 2 + 1);
    let mut lp = version[0];
    q.push(lp);
    for &p in &version[1..] {
        let lq = *q.last().unwrap();
        if isspecial(p) {
            if lq != b'.' {
                q.push(b'.');
            }
        } else if (isndig(lp) && isdig(p)) || (isdig(lp) && isndig(p)) {
            if lq != b'.' {
                q.push(b'.');
            }
            q.push(p);
        } else if !p.is_ascii_alphanumeric() {
            if lq != b'.' {
                q.push(b'.');
            }
        } else {
            q.push(p);
        }
        lp = p;
    }
    // Strip a single trailing dot (an empty last component).
    if q.last() == Some(&b'.') {
        q.pop();
    }
    q
}

/// `compare_special_version_forms()`: prefix-match each form against the ordered
/// special-form table (first match wins); an unknown form ranks below `dev`.
fn special_form_order(form: &[u8]) -> i64 {
    const FORMS: &[(&[u8], i64)] = &[
        (b"dev", 0),
        (b"alpha", 1),
        (b"a", 1),
        (b"beta", 2),
        (b"b", 2),
        (b"RC", 3),
        (b"rc", 3),
        (b"#", 4),
        (b"pl", 5),
        (b"p", 5),
    ];
    for &(name, order) in FORMS {
        if form.starts_with(name) {
            return order;
        }
    }
    -1
}

fn compare_special(f1: &[u8], f2: &[u8]) -> i64 {
    vc_normalize(special_form_order(f1) - special_form_order(f2))
}

/// Split off the first `.`-delimited segment: `(segment, remainder, had_dot)`.
fn vc_split(s: &[u8]) -> (&[u8], &[u8], bool) {
    match s.iter().position(|&c| c == b'.') {
        Some(i) => (&s[..i], &s[i + 1..], true),
        None => (s, s, false),
    }
}

fn vc_parse_long(seg: &[u8]) -> i64 {
    // `strtol`: read the leading run of digits, saturating on overflow.
    let mut acc: i64 = 0;
    for &c in seg {
        if !c.is_ascii_digit() {
            break;
        }
        acc = acc
            .saturating_mul(10)
            .saturating_add((c - b'0') as i64);
    }
    acc
}

fn vc_compare_segments(p1: &[u8], p2: &[u8]) -> i64 {
    let d1 = p1.first().is_some_and(|c| c.is_ascii_digit());
    let d2 = p2.first().is_some_and(|c| c.is_ascii_digit());
    if d1 && d2 {
        vc_normalize(vc_parse_long(p1).cmp(&vc_parse_long(p2)) as i64)
    } else if !d1 && !d2 {
        compare_special(p1, p2)
    } else if d1 {
        compare_special(b"#N#", p2)
    } else {
        compare_special(p1, b"#N#")
    }
}

/// Walk two canonical (or `#`-sentinel) version strings segment by segment.
fn vc_walk(v1: &[u8], v2: &[u8]) -> i64 {
    if v1.is_empty() || v2.is_empty() {
        return if v1.is_empty() && v2.is_empty() {
            0
        } else if !v1.is_empty() {
            1
        } else {
            -1
        };
    }
    let mut p1 = v1;
    let mut p2 = v2;
    let mut compare = 0i64;
    let mut n1_some = true;
    let mut n2_some = true;
    while !p1.is_empty() && !p2.is_empty() && n1_some && n2_some {
        let (seg1, rest1, has1) = vc_split(p1);
        let (seg2, rest2, has2) = vc_split(p2);
        n1_some = has1;
        n2_some = has2;
        compare = vc_compare_segments(seg1, seg2);
        if compare != 0 {
            break;
        }
        if has1 {
            p1 = rest1;
        }
        if has2 {
            p2 = rest2;
        }
    }
    if compare == 0 {
        if n1_some {
            if p1.first().is_some_and(|c| c.is_ascii_digit()) {
                compare = 1;
            } else {
                compare = vc_walk(p1, b"#N#");
            }
        } else if n2_some {
            if p2.first().is_some_and(|c| c.is_ascii_digit()) {
                compare = -1;
            } else {
                compare = vc_walk(b"#N#", p2);
            }
        }
    }
    compare
}

/// `php_version_compare()`: canonicalize both (unless `#`-prefixed) then walk.
fn php_version_compare(o1: &[u8], o2: &[u8]) -> i64 {
    if o1.is_empty() || o2.is_empty() {
        return if o1.is_empty() && o2.is_empty() {
            0
        } else if !o1.is_empty() {
            1
        } else {
            -1
        };
    }
    let v1 = if o1[0] == b'#' {
        o1.to_vec()
    } else {
        canonicalize_version(o1)
    };
    let v2 = if o2[0] == b'#' {
        o2.to_vec()
    } else {
        canonicalize_version(o2)
    };
    vc_walk(&v1, &v2)
}

/// `version_compare($v1, $v2, $operator = null)`.
pub fn version_compare(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let v1 = convert::to_zstr(&args[0], ctx.diags);
    let v2 = convert::to_zstr(&args[1], ctx.diags);
    let cmp = php_version_compare(v1.as_bytes(), v2.as_bytes());
    let op = match args.get(2) {
        None | Some(Zval::Null) => return Ok(Zval::Long(cmp)),
        Some(o) => convert::to_zstr(o, ctx.diags),
    };
    let result = match op.as_bytes() {
        b"<" | b"lt" => cmp == -1,
        b"<=" | b"le" => cmp != 1,
        b">" | b"gt" => cmp == 1,
        b">=" | b"ge" => cmp != -1,
        b"==" | b"=" | b"eq" => cmp == 0,
        b"!=" | b"<>" | b"ne" => cmp != 0,
        _ => {
            return Err(PhpError::ValueError(
                "version_compare(): Argument #3 ($operator) must be a valid comparison operator"
                    .to_string(),
            ))
        }
    };
    Ok(Zval::Bool(result))
}
