//! JSON builtins (step 26).
//!
//! `json_encode` is a pure value function and lives here. `json_decode` must
//! build `stdClass` instances for the default (non-assoc) mode, which needs the
//! class table, so it is intercepted by the evaluator instead (see
//! `eval.rs::ho_json_decode`).

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::dtoa::double_to_shortest;
use php_types::{Key, Object, PhpArray, PhpError, PhpStr, PropVis, Zval};

const UNESCAPED_SLASHES: i64 = 64;
const PRETTY_PRINT: i64 = 128;
const UNESCAPED_UNICODE: i64 = 256;
const UNESCAPED_LINE_TERMINATORS: i64 = 2048;

const INDENT: &[u8] = b"    ";

/// `json_encode($value, $flags = 0)`. Returns the JSON string, or `false` on a
/// value JSON cannot represent (non-finite float, invalid UTF-8, a resource /
/// closure). Supported flags: `JSON_PRETTY_PRINT`, `JSON_UNESCAPED_SLASHES`,
/// `JSON_UNESCAPED_UNICODE`.
pub fn json_encode(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let Some(value) = args.first() else {
        return Err(PhpError::ArgumentCountError(
            "json_encode() expects at least 1 argument, 0 given".to_string(),
        ));
    };
    let flags = args.get(1).map(as_i64).unwrap_or(0);
    let mut out = Vec::new();
    match encode(value, flags, 1, &mut seen_stack(), &mut out) {
        Ok(()) => Ok(Zval::Str(PhpStr::new(out))),
        Err(()) => Ok(Zval::Bool(false)),
    }
}

fn as_i64(v: &Zval) -> i64 {
    match v {
        Zval::Long(n) => *n,
        Zval::Bool(b) => *b as i64,
        Zval::Double(d) => *d as i64,
        _ => 0,
    }
}

/// JSON_PARTIAL_OUTPUT_ON_ERROR.
const PARTIAL: i64 = 512;

fn seen_stack() -> Vec<usize> {
    Vec::new()
}

/// Emit PHP's partial-output substitute for an unencodable node (`null`; a
/// non-finite double becomes `0`).
fn partial_null(out: &mut Vec<u8>) {
    out.extend_from_slice(b"null");
}

fn encode(
    v: &Zval,
    flags: i64,
    depth: usize,
    seen: &mut Vec<usize>,
    out: &mut Vec<u8>,
) -> Result<(), ()> {
    let partial = flags & PARTIAL != 0;
    // PHP's json_encode depth limit (default $depth = 512): a deeper nesting —
    // in particular a cyclic object graph, which json_normalize's cycle check
    // cannot see through *plain* object properties — is `false`, not a stack
    // overflow (JSON_ERROR_DEPTH/RECURSION territory).
    if depth > 512 {
        if partial {
            partial_null(out);
            return Ok(());
        }
        return Err(());
    }
    match v {
        Zval::Null | Zval::Undef => out.extend_from_slice(b"null"),
        Zval::Bool(true) => out.extend_from_slice(b"true"),
        Zval::Bool(false) => out.extend_from_slice(b"false"),
        Zval::Long(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Zval::Double(d) => {
            if !d.is_finite() {
                // Partial output substitutes 0 for INF/NAN (Zend's rule).
                if partial {
                    out.push(b'0');
                    return Ok(());
                }
                return Err(());
            }
            // Same shortest-roundtrip digits as var_dump (serialize_precision=-1),
            // but JSON uses a lowercase exponent marker.
            let mut s = double_to_shortest(*d);
            for b in s.iter_mut() {
                if *b == b'E' {
                    *b = b'e';
                }
            }
            out.extend_from_slice(&s);
            // JSON_PRESERVE_ZERO_FRACTION (1024): a whole-number float keeps a
            // ".0" so it decodes back as a float (Elastica's stringify sets it).
            if flags & 1024 != 0 && !s.contains(&b'.') && !s.contains(&b'e') {
                out.extend_from_slice(b".0");
            }
        }
        Zval::Str(s) => {
            // JSON_NUMERIC_CHECK (32): a numeric STRING value encodes as its
            // number (values only — array/object keys go through the key
            // path, which PHP does not numeric-check).
            const NUMERIC_CHECK: i64 = 32;
            if flags & NUMERIC_CHECK != 0 {
                if let Some(info) = php_types::numstr::parse_numeric_ex(s.as_bytes(), true) {
                    if !info.trailing {
                        match info.num {
                            php_types::numstr::Num::Long(n) => {
                                out.extend_from_slice(n.to_string().as_bytes());
                                return Ok(());
                            }
                            php_types::numstr::Num::Double(d) if d.is_finite() => {
                                let mut buf = double_to_shortest(d);
                                for b in buf.iter_mut() {
                                    if *b == b'E' {
                                        *b = b'e';
                                    }
                                }
                                out.extend_from_slice(&buf);
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                }
            }
            let before = out.len();
            if encode_string(s.as_bytes(), flags, out).is_err() {
                if partial {
                    out.truncate(before);
                    partial_null(out);
                    return Ok(());
                }
                return Err(());
            }
        }
        Zval::Array(a) => {
            // Recursion check by identity: a revisited container is PHP's
            // JSON_ERROR_RECURSION (→ null under partial output).
            let addr = Rc::as_ptr(a) as usize;
            if seen.contains(&addr) {
                if partial {
                    partial_null(out);
                    return Ok(());
                }
                return Err(());
            }
            seen.push(addr);
            let r = encode_array(a, flags, depth, seen, out);
            seen.pop();
            r?
        }
        Zval::Object(o) => {
            let addr = Rc::as_ptr(o) as usize;
            if seen.contains(&addr) {
                if partial {
                    partial_null(out);
                    return Ok(());
                }
                return Err(());
            }
            seen.push(addr);
            let r = encode_object(&o.borrow(), flags, depth, seen, out);
            seen.pop();
            r?
        }
        Zval::Ref(r) => encode(&r.borrow(), flags, depth, seen, out)?,
        _ => {
            // Resources and other unencodable values.
            if partial {
                partial_null(out);
                return Ok(());
            }
            return Err(());
        }
    }
    Ok(())
}

/// A PHP array encodes as a JSON array iff its keys are exactly `0..n-1` in
/// order; otherwise it is a JSON object with stringified keys.
fn is_list(a: &PhpArray) -> bool {
    a.iter()
        .enumerate()
        .all(|(i, (k, _))| matches!(k, Key::Int(n) if *n == i as i64))
}

fn encode_array(
    a: &PhpArray,
    flags: i64,
    depth: usize,
    seen: &mut Vec<usize>,
    out: &mut Vec<u8>,
) -> Result<(), ()> {
    // JSON_FORCE_OBJECT (16): every array — list or empty — encodes as an object.
    const FORCE_OBJECT: i64 = 16;
    if a.is_empty() {
        out.extend_from_slice(if flags & FORCE_OBJECT != 0 { b"{}" } else { b"[]" });
        return Ok(());
    }
    if flags & FORCE_OBJECT == 0 && is_list(a) {
        let items: Vec<&Zval> = a.iter().map(|(_, v)| v).collect();
        encode_seq(b'[', b']', flags, depth, items.len(), out, |i, flags, depth, out| {
            encode(items[i], flags, depth, seen, out)
        })
    } else {
        let entries: Vec<(Vec<u8>, &Zval)> = a
            .iter()
            .map(|(k, v)| (key_bytes(k), v))
            .collect();
        encode_seq(b'{', b'}', flags, depth, entries.len(), out, |i, flags, depth, out| {
            encode_string(&entries[i].0, flags, out)?;
            out.extend_from_slice(if flags & PRETTY_PRINT != 0 { b": " } else { b":" });
            encode(entries[i].1, flags, depth, seen, out)
        })
    }
}

fn encode_object(
    o: &Object,
    flags: i64,
    depth: usize,
    seen: &mut Vec<usize>,
    out: &mut Vec<u8>,
) -> Result<(), ()> {
    // Only public properties are serialised, in declaration / insertion order
    // (a mangled private key unmangles to Private and is filtered out).
    let entries: Vec<(Vec<u8>, &Zval)> = o
        .props
        .iter()
        .filter_map(|(key, v)| {
            let (disp, vis) = php_types::unmangle_prop_key(key, &o.info);
            matches!(vis, PropVis::Public).then(|| (disp.to_vec(), v))
        })
        .collect();
    if entries.is_empty() {
        out.extend_from_slice(b"{}");
        return Ok(());
    }
    encode_seq(b'{', b'}', flags, depth, entries.len(), out, |i, flags, depth, out| {
        encode_string(&entries[i].0, flags, out)?;
        out.extend_from_slice(if flags & PRETTY_PRINT != 0 { b": " } else { b":" });
        encode(entries[i].1, flags, depth, seen, out)
    })
}

/// Emit `open … close` with `len` comma-separated items, honouring
/// `JSON_PRETTY_PRINT` (4-space indent, newlines). `item(i, flags, depth, out)`
/// writes element `i`.
fn encode_seq(
    open: u8,
    close: u8,
    flags: i64,
    depth: usize,
    len: usize,
    out: &mut Vec<u8>,
    mut item: impl FnMut(usize, i64, usize, &mut Vec<u8>) -> Result<(), ()>,
) -> Result<(), ()> {
    let pretty = flags & PRETTY_PRINT != 0;
    out.push(open);
    for i in 0..len {
        if i > 0 {
            out.push(b',');
        }
        if pretty {
            out.push(b'\n');
            for _ in 0..depth {
                out.extend_from_slice(INDENT);
            }
        }
        item(i, flags, depth + 1, out)?;
    }
    if pretty {
        out.push(b'\n');
        for _ in 0..depth - 1 {
            out.extend_from_slice(INDENT);
        }
    }
    out.push(close);
    Ok(())
}

fn key_bytes(k: &Key) -> Vec<u8> {
    match k {
        Key::Int(n) => n.to_string().into_bytes(),
        Key::Str(s) => s.as_bytes().to_vec(),
    }
}

fn encode_string(s: &[u8], flags: i64, out: &mut Vec<u8>) -> Result<(), ()> {
    // PHP requires valid UTF-8; otherwise json_encode fails (JSON_ERROR_UTF8) —
    // unless JSON_INVALID_UTF8_SUBSTITUTE (each invalid sequence becomes
    // U+FFFD) or JSON_INVALID_UTF8_IGNORE (dropped) is set.
    const INVALID_UTF8_IGNORE: i64 = 1048576;
    const INVALID_UTF8_SUBSTITUTE: i64 = 2097152;
    let owned: String;
    let text = match std::str::from_utf8(s) {
        Ok(t) => t,
        Err(_) if flags & INVALID_UTF8_SUBSTITUTE != 0 => {
            owned = String::from_utf8_lossy(s).into_owned();
            &owned
        }
        Err(_) if flags & INVALID_UTF8_IGNORE != 0 => {
            owned = s.utf8_chunks().map(|c| c.valid()).collect();
            &owned
        }
        Err(_) => return Err(()),
    };
    // JSON_HEX_TAG (1) / _AMP (2) / _APOS (4) / _QUOT (8): HTML-safe escaping
    // of <, >, &, ', " as \u00XX (symfony JsonResponse's default options).
    const HEX_TAG: i64 = 1;
    const HEX_AMP: i64 = 2;
    const HEX_APOS: i64 = 4;
    const HEX_QUOT: i64 = 8;
    out.push(b'"');
    for ch in text.chars() {
        match ch {
            // php_json_encoder.c emits these as FIXED literals — uppercase hex
            // (`<`), unlike the lowercase generic escaper below.
            '<' if flags & HEX_TAG != 0 => out.extend_from_slice(b"\\u003C"),
            '>' if flags & HEX_TAG != 0 => out.extend_from_slice(b"\\u003E"),
            '&' if flags & HEX_AMP != 0 => out.extend_from_slice(b"\\u0026"),
            '\'' if flags & HEX_APOS != 0 => out.extend_from_slice(b"\\u0027"),
            '"' if flags & HEX_QUOT != 0 => out.extend_from_slice(b"\\u0022"),
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '/' => {
                if flags & UNESCAPED_SLASHES != 0 {
                    out.push(b'/');
                } else {
                    out.extend_from_slice(b"\\/");
                }
            }
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\u{08}' => out.extend_from_slice(b"\\b"),
            '\u{0C}' => out.extend_from_slice(b"\\f"),
            c if (c as u32) < 0x20 => unicode_escape(c as u32, out),
            c if (c as u32) < 0x80 => out.push(c as u8),
            c => {
                // U+2028/U+2029 restano escapati anche sotto UNESCAPED_UNICODE
                // finché non c'è JSON_UNESCAPED_LINE_TERMINATORS (json.c 7.1+).
                if flags & UNESCAPED_UNICODE != 0
                    && (!matches!(c, '\u{2028}' | '\u{2029}')
                        || flags & UNESCAPED_LINE_TERMINATORS != 0)
                {
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                } else {
                    let cp = c as u32;
                    if cp > 0xFFFF {
                        // Encode as a UTF-16 surrogate pair.
                        let v = cp - 0x10000;
                        unicode_escape(0xD800 + (v >> 10), out);
                        unicode_escape(0xDC00 + (v & 0x3FF), out);
                    } else {
                        unicode_escape(cp, out);
                    }
                }
            }
        }
    }
    out.push(b'"');
    Ok(())
}

fn unicode_escape(cp: u32, out: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.extend_from_slice(b"\\u");
    out.push(HEX[((cp >> 12) & 0xF) as usize]);
    out.push(HEX[((cp >> 8) & 0xF) as usize]);
    out.push(HEX[((cp >> 4) & 0xF) as usize]);
    out.push(HEX[(cp & 0xF) as usize]);
}
