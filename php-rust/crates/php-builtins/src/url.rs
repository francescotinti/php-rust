//! URL builtins (`parse_url`). Faithful port of `php_url_parse_ex2` and the
//! `parse_url()` handler from ext/standard/url.c, working on raw bytes.

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

#[derive(Default)]
pub(crate) struct ParsedUrl {
    pub(crate) scheme: Option<Vec<u8>>,
    pub(crate) host: Option<Vec<u8>>,
    port: u16,
    has_port: bool,
    user: Option<Vec<u8>>,
    pass: Option<Vec<u8>>,
    pub(crate) path: Option<Vec<u8>>,
    pub(crate) query: Option<Vec<u8>>,
    fragment: Option<Vec<u8>>,
}

#[inline]
fn iscntrl(c: u8) -> bool {
    c < 0x20 || c == 0x7f
}

/// `php_replace_controlchars`: any control byte becomes `_`.
fn replace_controlchars(mut v: Vec<u8>) -> Vec<u8> {
    for b in v.iter_mut() {
        if iscntrl(*b) {
            *b = b'_';
        }
    }
    v
}

/// First occurrence of `ch` in `str[lo..hi]`.
fn memchr(s: &[u8], lo: usize, hi: usize, ch: u8) -> Option<usize> {
    s[lo..hi].iter().position(|&c| c == ch).map(|i| lo + i)
}

/// Last occurrence of `ch` in `str[lo..hi]` (`zend_memrchr`).
fn memrchr(s: &[u8], lo: usize, hi: usize, ch: u8) -> Option<usize> {
    s[lo..hi].iter().rposition(|&c| c == ch).map(|i| lo + i)
}

/// `binary_strcspn`: earliest position (within `[lo, hi)`) of any char in
/// `chars`, else `hi`.
fn binary_strcspn(s: &[u8], lo: usize, hi: usize, chars: &[u8]) -> usize {
    let mut e = hi;
    for &ch in chars {
        if let Some(p) = memchr(s, lo, e, ch) {
            e = p;
        }
    }
    e
}

/// Parse the ASCII decimal in `str[lo..hi]` as a port, validating range.
/// Returns `Some(port)` when it is a valid 0..=65535 value with at least one
/// digit, else `None`.
fn parse_port(s: &[u8], lo: usize, hi: usize) -> Option<u16> {
    if lo >= hi {
        return None;
    }
    let mut val: u32 = 0;
    for &c in &s[lo..hi] {
        if !c.is_ascii_digit() {
            // `strtol` stops at the first non-digit; the caller only feeds digit
            // runs, but mirror the "end != port_buf" success requirement.
            break;
        }
        val = val.saturating_mul(10).saturating_add((c - b'0') as u32);
    }
    if val <= 65535 {
        Some(val as u16)
    } else {
        None
    }
}

enum Stage {
    Scheme,
    ParsePort(usize), // carries the colon index `e`
    ParseHost,
    JustPath,
}

pub(crate) fn php_url_parse(s: &[u8]) -> Option<ParsedUrl> {
    let ue = s.len();
    let mut r = ParsedUrl::default();
    let mut pos = 0usize; // `s` in the C source
    let mut stage = Stage::Scheme;

    let is_scheme_char =
        |c: u8| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == b'+' || c == b'.' || c == b'-';

    loop {
        match stage {
            Stage::Scheme => {
                let e = memchr(s, pos, ue, b':');
                match e {
                    Some(ec) if ec != pos => {
                        // Validate the scheme characters in `[pos, ec)`.
                        let mut p = pos;
                        let mut diverted = false;
                        while p < ec {
                            if !is_scheme_char(s[p]) {
                                if ec + 1 < ue && ec < binary_strcspn(s, pos, ue, b"?#") {
                                    stage = Stage::ParsePort(ec);
                                } else if pos + 1 < ue && s[pos] == b'/' && s[pos + 1] == b'/' {
                                    pos += 2;
                                    stage = Stage::ParseHost;
                                } else {
                                    stage = Stage::JustPath;
                                }
                                diverted = true;
                                break;
                            }
                            p += 1;
                        }
                        if diverted {
                            continue;
                        }
                        // Scheme is fully valid.
                        if ec + 1 == ue {
                            r.scheme = Some(replace_controlchars(s[pos..ec].to_vec()));
                            return Some(r);
                        }
                        if s[ec + 1] != b'/' {
                            // The data after the colon may be a port (e.g. a.com:80).
                            let mut pp = ec + 1;
                            while pp < ue && s[pp].is_ascii_digit() {
                                pp += 1;
                            }
                            if (pp == ue || s[pp] == b'/') && (pp - ec) < 7 {
                                stage = Stage::ParsePort(ec);
                                continue;
                            }
                            r.scheme = Some(replace_controlchars(s[pos..ec].to_vec()));
                            pos = ec + 1;
                            stage = Stage::JustPath;
                            continue;
                        }
                        // `*(e+1) == '/'`.
                        let scheme_bytes = s[pos..ec].to_vec();
                        let is_file = scheme_bytes.eq_ignore_ascii_case(b"file");
                        r.scheme = Some(replace_controlchars(scheme_bytes));
                        if ec + 2 < ue && s[ec + 2] == b'/' {
                            pos = ec + 3;
                            if is_file && ec + 3 < ue && s[ec + 3] == b'/' {
                                // file:///c:/... — keep the drive letter on the path.
                                if ec + 5 < ue && s[ec + 5] == b':' {
                                    pos = ec + 4;
                                }
                                stage = Stage::JustPath;
                                continue;
                            }
                            stage = Stage::ParseHost;
                            continue;
                        } else {
                            pos = ec + 1;
                            stage = Stage::JustPath;
                            continue;
                        }
                    }
                    Some(ec) => {
                        // Colon at the very start: treat as a port.
                        stage = Stage::ParsePort(ec);
                        continue;
                    }
                    None => {
                        if pos + 1 < ue && s[pos] == b'/' && s[pos + 1] == b'/' {
                            pos += 2;
                            stage = Stage::ParseHost;
                        } else {
                            stage = Stage::JustPath;
                        }
                        continue;
                    }
                }
            }
            Stage::ParsePort(e) => {
                let p = e + 1;
                let mut pp = p;
                while pp < ue && pp - p < 6 && s[pp].is_ascii_digit() {
                    pp += 1;
                }
                if pp - p > 0 && pp - p < 6 && (pp == ue || s[pp] == b'/') {
                    match parse_port(s, p, pp) {
                        Some(port) => {
                            r.has_port = true;
                            r.port = port;
                            if pos + 1 < ue && s[pos] == b'/' && s[pos + 1] == b'/' {
                                pos += 2;
                            }
                        }
                        None => return None,
                    }
                    stage = Stage::ParseHost;
                } else if p == pp && pp == ue {
                    return None;
                } else if pos + 1 < ue && s[pos] == b'/' && s[pos + 1] == b'/' {
                    pos += 2;
                    stage = Stage::ParseHost;
                } else {
                    stage = Stage::JustPath;
                }
                continue;
            }
            Stage::ParseHost => {
                let e = binary_strcspn(s, pos, ue, b"/?#");
                // Login / password.
                if let Some(at) = memrchr(s, pos, e, b'@') {
                    if let Some(colon) = memchr(s, pos, at, b':') {
                        r.user = Some(replace_controlchars(s[pos..colon].to_vec()));
                        r.pass = Some(replace_controlchars(s[colon + 1..at].to_vec()));
                    } else {
                        r.user = Some(replace_controlchars(s[pos..at].to_vec()));
                    }
                    pos = at + 1;
                }
                // Port.
                let colon = if pos < ue && s[pos] == b'[' && e > pos && s[e - 1] == b']' {
                    None // IPv6 literal — short-circuit the port scan.
                } else {
                    memrchr(s, pos, e, b':')
                };
                let host_end = if let Some(c) = colon {
                    if r.port == 0 {
                        let pstart = c + 1;
                        if e > pstart {
                            if e - pstart > 5 {
                                return None;
                            }
                            match parse_port(s, pstart, e) {
                                Some(port) => {
                                    r.has_port = true;
                                    r.port = port;
                                }
                                None => return None,
                            }
                        }
                    }
                    c
                } else {
                    e
                };
                if host_end <= pos {
                    return None;
                }
                r.host = Some(replace_controlchars(s[pos..host_end].to_vec()));
                if e == ue {
                    return Some(r);
                }
                pos = e;
                stage = Stage::JustPath;
                continue;
            }
            Stage::JustPath => {
                let mut e = ue;
                if let Some(hash) = memchr(s, pos, e, b'#') {
                    let p = hash + 1;
                    r.fragment = Some(if p < e {
                        replace_controlchars(s[p..e].to_vec())
                    } else {
                        Vec::new()
                    });
                    e = hash;
                }
                if let Some(q) = memchr(s, pos, e, b'?') {
                    let p = q + 1;
                    r.query = Some(if p < e {
                        replace_controlchars(s[p..e].to_vec())
                    } else {
                        Vec::new()
                    });
                    e = q;
                }
                if pos < e || pos == ue {
                    r.path = Some(replace_controlchars(s[pos..e].to_vec()));
                }
                return Some(r);
            }
        }
    }
}

/// `parse_url($url, $component = -1)`.
pub fn parse_url(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let url = convert::to_zstr(&args[0], ctx.diags);
    let key = match args.get(1) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => -1,
    };
    let r = match php_url_parse(url.as_bytes()) {
        Some(r) => r,
        None => return Ok(Zval::Bool(false)),
    };
    let str_zval = |v: &Option<Vec<u8>>| v.clone().map(|b| Zval::Str(PhpStr::new(b)));
    if key > -1 {
        let comp = match key {
            0 => str_zval(&r.scheme),
            1 => str_zval(&r.host),
            2 => {
                if r.has_port {
                    Some(Zval::Long(r.port as i64))
                } else {
                    None
                }
            }
            3 => str_zval(&r.user),
            4 => str_zval(&r.pass),
            5 => str_zval(&r.path),
            6 => str_zval(&r.query),
            7 => str_zval(&r.fragment),
            _ => {
                return Err(PhpError::ValueError(format!(
                    "parse_url(): Argument #2 ($component) must be a valid URL component identifier, {key} given"
                )))
            }
        };
        return Ok(comp.unwrap_or(Zval::Null));
    }
    let mut arr = PhpArray::new();
    fn put(arr: &mut PhpArray, k: &[u8], v: &Option<Vec<u8>>) {
        if let Some(b) = v {
            arr.insert(Key::from_bytes(k), Zval::Str(PhpStr::new(b.clone())));
        }
    }
    put(&mut arr, b"scheme", &r.scheme);
    put(&mut arr, b"host", &r.host);
    if r.has_port {
        arr.insert(Key::from_bytes(b"port"), Zval::Long(r.port as i64));
    }
    put(&mut arr, b"user", &r.user);
    put(&mut arr, b"pass", &r.pass);
    put(&mut arr, b"path", &r.path);
    put(&mut arr, b"query", &r.query);
    put(&mut arr, b"fragment", &r.fragment);
    Ok(Zval::Array(Rc::new(arr)))
}


// ---------------------------------------------------------------------------
// urlencode / urldecode / rawurlencode / rawurldecode — ports of url.c.
// ---------------------------------------------------------------------------

#[inline]
fn hex_digit(b: u8) -> u8 {
    // High nibble first; uppercase, as PHP emits.
    b"0123456789ABCDEF"[b as usize & 0xf]
}

#[inline]
fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// `urlencode`: application/x-www-form-urlencoded — space becomes `+`, and every
/// byte outside `[A-Za-z0-9_.-]` becomes `%XX`.
fn encode(bytes: &[u8], raw: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for &c in bytes {
        let unreserved = c.is_ascii_alphanumeric()
            || matches!(c, b'_' | b'-' | b'.')
            || (raw && c == b'~');
        if unreserved {
            out.push(c);
        } else if !raw && c == b' ' {
            out.push(b'+');
        } else {
            out.push(b'%');
            out.push(hex_digit(c >> 4));
            out.push(hex_digit(c));
        }
    }
    out
}

/// `urldecode`/`rawurldecode`: decode `%XX`; for the non-raw form a `+` also
/// becomes a space. A `%` not followed by two hex digits is left verbatim.
fn decode(bytes: &[u8], raw: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // `%XX` needs two following hex digits (PHP's `len >= 2` guard).
        if c == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        if !raw && c == b'+' {
            out.push(b' ');
        } else {
            out.push(c);
        }
        i += 1;
    }
    out
}

/// `urlencode($string)`.
pub fn urlencode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(&args[0], ctx.diags);
    Ok(Zval::Str(PhpStr::new(encode(s.as_bytes(), false))))
}

/// `rawurlencode($string)`.
pub fn rawurlencode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(&args[0], ctx.diags);
    Ok(Zval::Str(PhpStr::new(encode(s.as_bytes(), true))))
}

/// `urldecode($string)`.
pub fn urldecode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(&args[0], ctx.diags);
    Ok(Zval::Str(PhpStr::new(decode(s.as_bytes(), false))))
}

/// `rawurldecode($string)`.
pub fn rawurldecode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(&args[0], ctx.diags);
    Ok(Zval::Str(PhpStr::new(decode(s.as_bytes(), true))))
}

/// `http_build_query($data, $numeric_prefix = "", $arg_separator = "&",
/// $encoding_type = PHP_QUERY_RFC1738)` — port of `php_url_encode_hash_ex`
/// (ext/standard/http.c): null values are skipped, bools become `1`/`0`,
/// nested arrays/objects recurse as `parent%5Bkey%5D`, integer keys at the top
/// level take the numeric prefix, and RFC3986 (`encoding_type` 2) uses `%20`
/// for spaces instead of `+`.
pub fn http_build_query(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = match args.first().map(|v| v.deref_clone()) {
        Some(v @ (Zval::Array(_) | Zval::Object(_))) => v,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "http_build_query(): Argument #1 ($data) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "http_build_query() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };
    let numeric_prefix = match args.get(1).map(|v| v.deref_clone()) {
        Some(Zval::Null) | None => Vec::new(),
        Some(v) => convert::to_zstr(&v, ctx.diags).as_bytes().to_vec(),
    };
    let sep = match args.get(2).map(|v| v.deref_clone()) {
        Some(Zval::Null) | None => b"&".to_vec(),
        Some(v) => convert::to_zstr(&v, ctx.diags).as_bytes().to_vec(),
    };
    let raw = matches!(args.get(3).map(|v| v.deref_clone()), Some(Zval::Long(2)));
    let mut parts: Vec<Vec<u8>> = Vec::new();
    hbq_walk(&data, None, &numeric_prefix, raw, &mut parts, ctx);
    Ok(Zval::Str(PhpStr::new(parts.join(&sep[..]))))
}

/// One level of `php_url_encode_hash_ex`: emit `key=value` pairs into `parts`.
/// `prefix` is the raw (unencoded) composite key of the enclosing level —
/// per-byte encoding at the leaf produces the same bytes as PHP's per-level
/// encoding, since `encode(a) + "%5B" + encode(b) + "%5D" == encode(a[b])`.
fn hbq_walk(
    data: &Zval,
    prefix: Option<&[u8]>,
    numeric_prefix: &[u8],
    raw: bool,
    parts: &mut Vec<Vec<u8>>,
    ctx: &mut Ctx,
) {
    // (key bytes or None for an integer key, integer key value, member value)
    let entries: Vec<(Option<Vec<u8>>, i64, Zval)> = match data {
        Zval::Array(a) => a
            .iter()
            .map(|(k, v)| match k {
                Key::Int(i) => (None, *i, v.deref_clone()),
                Key::Str(s) => (Some(s.as_bytes().to_vec()), 0, v.deref_clone()),
            })
            .collect(),
        Zval::Object(o) => o
            .borrow()
            .props
            .iter()
            // Private properties are stored NUL-mangled (`\0Class\0p`) and are
            // not included, matching PHP's public-only hash of an object.
            .filter(|(k, _)| !k.starts_with(b"\0"))
            .map(|(k, v)| (Some(k.to_vec()), 0, v.deref_clone()))
            .collect(),
        _ => return,
    };
    for (skey, ikey, value) in entries {
        if matches!(value, Zval::Null | Zval::Undef) {
            continue;
        }
        let composite: Vec<u8> = match (&skey, prefix) {
            (Some(s), None) => s.clone(),
            (None, None) => {
                let mut k = numeric_prefix.to_vec();
                k.extend_from_slice(ikey.to_string().as_bytes());
                k
            }
            (Some(s), Some(p)) => {
                let mut k = p.to_vec();
                k.push(b'[');
                k.extend_from_slice(s);
                k.push(b']');
                k
            }
            (None, Some(p)) => {
                let mut k = p.to_vec();
                k.push(b'[');
                k.extend_from_slice(ikey.to_string().as_bytes());
                k.push(b']');
                k
            }
        };
        match value {
            Zval::Array(_) | Zval::Object(_) => {
                hbq_walk(&value, Some(&composite), numeric_prefix, raw, parts, ctx)
            }
            Zval::Bool(b) => {
                let mut pair = encode(&composite, raw);
                pair.push(b'=');
                pair.push(if b { b'1' } else { b'0' });
                parts.push(pair);
            }
            other => {
                let sv = convert::to_zstr(&other, ctx.diags);
                let mut pair = encode(&composite, raw);
                pair.push(b'=');
                pair.extend_from_slice(&encode(sv.as_bytes(), raw));
                parts.push(pair);
            }
        }
    }
}
