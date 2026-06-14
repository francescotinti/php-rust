//! JSON builtins (step 26).
//!
//! `json_encode` is a pure value function and lives here. `json_decode` must
//! build `stdClass` instances for the default (non-assoc) mode, which needs the
//! class table, so it is intercepted by the evaluator instead (see
//! `eval.rs::ho_json_decode`).

use php_runtime::Ctx;
use php_types::dtoa::double_to_shortest;
use php_types::{Key, Object, PhpArray, PhpError, PhpStr, PropVis, Zval};

const UNESCAPED_SLASHES: i64 = 64;
const PRETTY_PRINT: i64 = 128;
const UNESCAPED_UNICODE: i64 = 256;

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
    match encode(value, flags, 1, &mut out) {
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

fn encode(v: &Zval, flags: i64, depth: usize, out: &mut Vec<u8>) -> Result<(), ()> {
    match v {
        Zval::Null | Zval::Undef => out.extend_from_slice(b"null"),
        Zval::Bool(true) => out.extend_from_slice(b"true"),
        Zval::Bool(false) => out.extend_from_slice(b"false"),
        Zval::Long(n) => out.extend_from_slice(n.to_string().as_bytes()),
        Zval::Double(d) => {
            if !d.is_finite() {
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
        }
        Zval::Str(s) => encode_string(s.as_bytes(), flags, out)?,
        Zval::Array(a) => encode_array(a, flags, depth, out)?,
        Zval::Object(o) => encode_object(&o.borrow(), flags, depth, out)?,
        Zval::Ref(r) => encode(&r.borrow(), flags, depth, out)?,
        _ => return Err(()),
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

fn encode_array(a: &PhpArray, flags: i64, depth: usize, out: &mut Vec<u8>) -> Result<(), ()> {
    if a.len() == 0 {
        out.extend_from_slice(b"[]");
        return Ok(());
    }
    if is_list(a) {
        let items: Vec<&Zval> = a.iter().map(|(_, v)| v).collect();
        encode_seq(b'[', b']', flags, depth, items.len(), out, |i, flags, depth, out| {
            encode(items[i], flags, depth, out)
        })
    } else {
        let entries: Vec<(Vec<u8>, &Zval)> = a
            .iter()
            .map(|(k, v)| (key_bytes(k), v))
            .collect();
        encode_seq(b'{', b'}', flags, depth, entries.len(), out, |i, flags, depth, out| {
            encode_string(&entries[i].0, flags, out)?;
            out.extend_from_slice(if flags & PRETTY_PRINT != 0 { b": " } else { b":" });
            encode(entries[i].1, flags, depth, out)
        })
    }
}

fn encode_object(o: &Object, flags: i64, depth: usize, out: &mut Vec<u8>) -> Result<(), ()> {
    // Only public properties are serialised, in declaration / insertion order.
    let entries: Vec<(Vec<u8>, &Zval)> = o
        .props
        .iter()
        .filter(|(name, _)| matches!(o.info.vis_of(name), PropVis::Public))
        .map(|(name, v)| (name.to_vec(), v))
        .collect();
    if entries.is_empty() {
        out.extend_from_slice(b"{}");
        return Ok(());
    }
    encode_seq(b'{', b'}', flags, depth, entries.len(), out, |i, flags, depth, out| {
        encode_string(&entries[i].0, flags, out)?;
        out.extend_from_slice(if flags & PRETTY_PRINT != 0 { b": " } else { b":" });
        encode(entries[i].1, flags, depth, out)
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
    // PHP requires valid UTF-8; otherwise json_encode fails (JSON_ERROR_UTF8).
    let text = std::str::from_utf8(s).map_err(|_| ())?;
    out.push(b'"');
    for ch in text.chars() {
        match ch {
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
                if flags & UNESCAPED_UNICODE != 0 {
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
