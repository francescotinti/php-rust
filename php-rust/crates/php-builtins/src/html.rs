//! HTML entity builtins (step 56b): `htmlspecialchars`, `htmlspecialchars_decode`,
//! `htmlentities`, `html_entity_decode`. Verified byte-exact against PHP 8.5.7.
//! `htmlspecialchars` touches only the five ASCII specials (`& < > " '`);
//! `htmlentities` additionally maps the Latin-1 supplement (U+00A0–U+00FF) to its
//! named entities (the full HTML4 set — Greek/maths — is a scope-out, D-56.1).

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

/// Default `$flags` in PHP 8.1+: ENT_QUOTES | ENT_SUBSTITUTE | ENT_HTML401 = 11.
const DEFAULT_FLAGS: i64 = 11;

/// Named entities for the Latin-1 supplement, in code-point order U+00A0..=U+00FF.
const LATIN1: [&[u8]; 96] = [
    b"nbsp", b"iexcl", b"cent", b"pound", b"curren", b"yen", b"brvbar", b"sect", b"uml", b"copy",
    b"ordf", b"laquo", b"not", b"shy", b"reg", b"macr", b"deg", b"plusmn", b"sup2", b"sup3",
    b"acute", b"micro", b"para", b"middot", b"cedil", b"sup1", b"ordm", b"raquo", b"frac14",
    b"frac12", b"frac34", b"iquest", b"Agrave", b"Aacute", b"Acirc", b"Atilde", b"Auml", b"Aring",
    b"AElig", b"Ccedil", b"Egrave", b"Eacute", b"Ecirc", b"Euml", b"Igrave", b"Iacute", b"Icirc",
    b"Iuml", b"ETH", b"Ntilde", b"Ograve", b"Oacute", b"Ocirc", b"Otilde", b"Ouml", b"times",
    b"Oslash", b"Ugrave", b"Uacute", b"Ucirc", b"Uuml", b"Yacute", b"THORN", b"szlig", b"agrave",
    b"aacute", b"acirc", b"atilde", b"auml", b"aring", b"aelig", b"ccedil", b"egrave", b"eacute",
    b"ecirc", b"euml", b"igrave", b"iacute", b"icirc", b"iuml", b"eth", b"ntilde", b"ograve",
    b"oacute", b"ocirc", b"otilde", b"ouml", b"divide", b"oslash", b"ugrave", b"uacute", b"ucirc",
    b"uuml", b"yacute", b"thorn", b"yuml",
];

fn flags_of(args: &[Zval], idx: usize, ctx: &mut Ctx) -> (bool, bool) {
    let flags = args
        .get(idx)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(DEFAULT_FLAGS);
    (flags & 1 != 0, flags & 2 != 0) // (single, double)
}

/// Encode the five ASCII specials into `out`; returns true if `b` was special.
fn encode_special(out: &mut Vec<u8>, b: u8, single: bool, double: bool) -> bool {
    match b {
        b'&' => out.extend_from_slice(b"&amp;"),
        b'<' => out.extend_from_slice(b"&lt;"),
        b'>' => out.extend_from_slice(b"&gt;"),
        b'"' if double => out.extend_from_slice(b"&quot;"),
        b'\'' if single => out.extend_from_slice(b"&#039;"),
        _ => return false,
    }
    true
}

/// Decode one UTF-8 sequence at the start of `s`; returns `(codepoint, len)`.
/// Invalid bytes decode as the single raw byte.
fn decode_utf8(s: &[u8]) -> (u32, usize) {
    let b0 = s[0];
    let (len, init) = match b0 {
        0x00..=0x7f => return (b0 as u32, 1),
        0xc0..=0xdf => (2, (b0 & 0x1f) as u32),
        0xe0..=0xef => (3, (b0 & 0x0f) as u32),
        0xf0..=0xf7 => (4, (b0 & 0x07) as u32),
        _ => return (b0 as u32, 1),
    };
    if s.len() < len {
        return (b0 as u32, 1);
    }
    let mut cp = init;
    for &b in &s[1..len] {
        if b & 0xc0 != 0x80 {
            return (b0 as u32, 1);
        }
        cp = (cp << 6) | (b & 0x3f) as u32;
    }
    (cp, len)
}

/// Encode a code point as UTF-8 bytes.
fn encode_utf8(cp: u32) -> Vec<u8> {
    match char::from_u32(cp) {
        Some(c) => c.to_string().into_bytes(),
        None => Vec::new(),
    }
}

/// `htmlspecialchars($string, $flags = …)`: encode only the five ASCII specials.
pub fn htmlspecialchars(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "htmlspecialchars", 1)?;
    let (single, double) = flags_of(args, 1, ctx);
    let mut out = Vec::with_capacity(s.len());
    for &b in &s {
        if !encode_special(&mut out, b, single, double) {
            out.push(b);
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `htmlentities($string, $flags = …)`: encode the five specials plus the
/// Latin-1 supplement named entities.
pub fn htmlentities(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "htmlentities", 1)?;
    let (single, double) = flags_of(args, 1, ctx);
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] < 0x80 {
            if !encode_special(&mut out, s[i], single, double) {
                out.push(s[i]);
            }
            i += 1;
        } else {
            let (cp, len) = decode_utf8(&s[i..]);
            if (0xa0..=0xff).contains(&cp) {
                out.push(b'&');
                out.extend_from_slice(LATIN1[(cp - 0xa0) as usize]);
                out.push(b';');
            } else {
                out.extend_from_slice(&s[i..i + len]);
            }
            i += len;
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Resolve a single entity body (the chars between `&` and `;`) to its bytes, or
/// `None` to leave the `&` literal. `full` enables named Latin-1 + numeric forms.
fn decode_entity(ent: &[u8], full: bool, single: bool, double: bool) -> Option<Vec<u8>> {
    match ent {
        b"amp" => return Some(vec![b'&']),
        b"lt" => return Some(vec![b'<']),
        b"gt" => return Some(vec![b'>']),
        b"quot" if double => return Some(vec![b'"']),
        b"#039" | b"#39" if single => return Some(vec![b'\'']),
        _ => {}
    }
    if !full {
        return None;
    }
    if let Some(rest) = ent.strip_prefix(b"#x").or_else(|| ent.strip_prefix(b"#X")) {
        let cp = u32::from_str_radix(std::str::from_utf8(rest).ok()?, 16).ok()?;
        return Some(encode_utf8(cp));
    }
    if let Some(rest) = ent.strip_prefix(b"#") {
        let cp = std::str::from_utf8(rest).ok()?.parse::<u32>().ok()?;
        return Some(encode_utf8(cp));
    }
    // Named Latin-1 entity → its code point.
    LATIN1
        .iter()
        .position(|&name| name == ent)
        .map(|idx| encode_utf8(0xa0 + idx as u32))
}

fn decode_all(s: &[u8], full: bool, single: bool, double: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'&' {
            if let Some(rel) = s[i + 1..].iter().position(|&b| b == b';') {
                let semi = i + 1 + rel;
                if let Some(repl) = decode_entity(&s[i + 1..semi], full, single, double) {
                    out.extend_from_slice(&repl);
                    i = semi + 1;
                    continue;
                }
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

/// `htmlspecialchars_decode($string, $flags = …)`: reverse the five specials.
pub fn htmlspecialchars_decode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "htmlspecialchars_decode", 1)?;
    let (single, double) = flags_of(args, 1, ctx);
    Ok(Zval::Str(PhpStr::new(decode_all(&s, false, single, double))))
}

/// `html_entity_decode($string, $flags = …)`: reverse named (Latin-1) + numeric
/// entities as well as the five specials.
pub fn html_entity_decode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "html_entity_decode", 1)?;
    let (single, double) = flags_of(args, 1, ctx);
    Ok(Zval::Str(PhpStr::new(decode_all(&s, true, single, double))))
}
