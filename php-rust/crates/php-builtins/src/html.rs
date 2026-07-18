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

/// HTML 4.01 named entities beyond the Latin-1 supplement (the `symbols` +
/// `special` tables of Zend/ext/standard/html_tables.h), name → code point.
/// Completes D-56.1: WordPress' `WP_Scripts::localize` html_entity_decode()s
/// every translated string (`Crunching&hellip;` must round-trip to `…`).
const HTML401_EXT: [(&[u8], u32); 152] = [
    (b"fnof", 402),
    (b"Alpha", 913), (b"Beta", 914), (b"Gamma", 915), (b"Delta", 916),
    (b"Epsilon", 917), (b"Zeta", 918), (b"Eta", 919), (b"Theta", 920),
    (b"Iota", 921), (b"Kappa", 922), (b"Lambda", 923), (b"Mu", 924),
    (b"Nu", 925), (b"Xi", 926), (b"Omicron", 927), (b"Pi", 928),
    (b"Rho", 929), (b"Sigma", 931), (b"Tau", 932), (b"Upsilon", 933),
    (b"Phi", 934), (b"Chi", 935), (b"Psi", 936), (b"Omega", 937),
    (b"alpha", 945), (b"beta", 946), (b"gamma", 947), (b"delta", 948),
    (b"epsilon", 949), (b"zeta", 950), (b"eta", 951), (b"theta", 952),
    (b"iota", 953), (b"kappa", 954), (b"lambda", 955), (b"mu", 956),
    (b"nu", 957), (b"xi", 958), (b"omicron", 959), (b"pi", 960),
    (b"rho", 961), (b"sigmaf", 962), (b"sigma", 963), (b"tau", 964),
    (b"upsilon", 965), (b"phi", 966), (b"chi", 967), (b"psi", 968),
    (b"omega", 969), (b"thetasym", 977), (b"upsih", 978), (b"piv", 982),
    (b"bull", 8226), (b"hellip", 8230), (b"prime", 8242), (b"Prime", 8243),
    (b"oline", 8254), (b"frasl", 8260),
    (b"weierp", 8472), (b"image", 8465), (b"real", 8476), (b"trade", 8482),
    (b"alefsym", 8501),
    (b"larr", 8592), (b"uarr", 8593), (b"rarr", 8594), (b"darr", 8595),
    (b"harr", 8596), (b"crarr", 8629), (b"lArr", 8656), (b"uArr", 8657),
    (b"rArr", 8658), (b"dArr", 8659), (b"hArr", 8660),
    (b"forall", 8704), (b"part", 8706), (b"exist", 8707), (b"empty", 8709),
    (b"nabla", 8711), (b"isin", 8712), (b"notin", 8713), (b"ni", 8715),
    (b"prod", 8719), (b"sum", 8721), (b"minus", 8722), (b"lowast", 8727),
    (b"radic", 8730), (b"prop", 8733), (b"infin", 8734), (b"ang", 8736),
    (b"and", 8743), (b"or", 8744), (b"cap", 8745), (b"cup", 8746),
    (b"int", 8747), (b"there4", 8756), (b"sim", 8764), (b"cong", 8773),
    (b"asymp", 8776), (b"ne", 8800), (b"equiv", 8801), (b"le", 8804),
    (b"ge", 8805), (b"sub", 8834), (b"sup", 8835), (b"nsub", 8836),
    (b"sube", 8838), (b"supe", 8839), (b"oplus", 8853), (b"otimes", 8855),
    (b"perp", 8869), (b"sdot", 8901),
    (b"lceil", 8968), (b"rceil", 8969), (b"lfloor", 8970), (b"rfloor", 8971),
    (b"lang", 9001), (b"rang", 9002),
    (b"loz", 9674), (b"spades", 9824), (b"clubs", 9827), (b"hearts", 9829),
    (b"diams", 9830),
    (b"OElig", 338), (b"oelig", 339), (b"Scaron", 352), (b"scaron", 353),
    (b"Yuml", 376), (b"circ", 710), (b"tilde", 732),
    (b"ensp", 8194), (b"emsp", 8195), (b"thinsp", 8201), (b"zwnj", 8204),
    (b"zwj", 8205), (b"lrm", 8206), (b"rlm", 8207),
    (b"ndash", 8211), (b"mdash", 8212), (b"lsquo", 8216), (b"rsquo", 8217),
    (b"sbquo", 8218), (b"ldquo", 8220), (b"rdquo", 8221), (b"bdquo", 8222),
    (b"dagger", 8224), (b"Dagger", 8225), (b"permil", 8240),
    (b"lsaquo", 8249), (b"rsaquo", 8250), (b"euro", 8364),
];

fn flags_of(args: &[Zval], idx: usize, ctx: &mut Ctx) -> (bool, bool, bool) {
    let flags = args
        .get(idx)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(DEFAULT_FLAGS);
    // Doctype bits (mask 48): ENT_HTML401=0 renders the apostrophe as the
    // numeric `&#039;`; ENT_XML1/ENT_XHTML/ENT_HTML5 use the named `&apos;`
    // (WP's esc_xml, WP-18). (single, double, apos_named).
    (flags & 1 != 0, flags & 2 != 0, flags & 48 != 0)
}

/// Encode the five ASCII specials into `out`; returns true if `b` was special.
fn encode_special(out: &mut Vec<u8>, b: u8, single: bool, double: bool, apos_named: bool) -> bool {
    match b {
        b'&' => out.extend_from_slice(b"&amp;"),
        b'<' => out.extend_from_slice(b"&lt;"),
        b'>' => out.extend_from_slice(b"&gt;"),
        b'"' if double => out.extend_from_slice(b"&quot;"),
        b'\'' if single && apos_named => out.extend_from_slice(b"&apos;"),
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

/// With `$double_encode=false`, an `&` opening an existing entity is left
/// alone. Returns the entity length (`&` through `;`) or `None`. PHP checks
/// the doctype's entity table; this accepts any well-formed named/numeric
/// body — WordPress pre-normalizes unknown entities to `&amp;…` before the
/// double_encode=false call, so the shapes agree where it matters.
fn entity_len(s: &[u8]) -> Option<usize> {
    if s.first() != Some(&b'&') {
        return None;
    }
    let body_ok = |body: &[u8]| -> bool {
        if let Some(num) = body.strip_prefix(b"#") {
            if let Some(hex) = num.strip_prefix(b"x").or_else(|| num.strip_prefix(b"X")) {
                return !hex.is_empty() && hex.iter().all(|b| b.is_ascii_hexdigit());
            }
            return !num.is_empty() && num.iter().all(|b| b.is_ascii_digit());
        }
        // Named bodies: every real HTML entity name is >= 2 chars (lt, gt,
        // mu, …) — a 1-char body is never valid and PHP re-encodes its `&`.
        body.len() >= 2
            && body[0].is_ascii_alphabetic()
            && body.iter().all(|b| b.is_ascii_alphanumeric())
    };
    let end = s[1..].iter().take(32).position(|&b| b == b';')? + 1;
    body_ok(&s[1..end]).then_some(end + 1)
}

/// `htmlspecialchars($string, $flags = …, $encoding = null, $double_encode = true)`:
/// encode only the five ASCII specials.
pub fn htmlspecialchars(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "htmlspecialchars", 1)?;
    let (single, double, apos_named) = flags_of(args, 1, ctx);
    let double_encode = match args.get(3) {
        Some(v) => convert::to_bool(v, ctx.diags),
        None => true,
    };
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        let b = s[i];
        if b == b'&' && !double_encode {
            if let Some(len) = entity_len(&s[i..]) {
                out.extend_from_slice(&s[i..i + len]);
                i += len;
                continue;
            }
        }
        if !encode_special(&mut out, b, single, double, apos_named) {
            out.push(b);
        }
        i += 1;
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `htmlentities($string, $flags = …)`: encode the five specials plus the
/// Latin-1 supplement named entities.
pub fn htmlentities(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "htmlentities", 1)?;
    let (single, double, apos_named) = flags_of(args, 1, ctx);
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] < 0x80 {
            if !encode_special(&mut out, s[i], single, double, apos_named) {
                out.push(s[i]);
            }
            i += 1;
        } else {
            let (cp, len) = decode_utf8(&s[i..]);
            if (0xa0..=0xff).contains(&cp) {
                out.push(b'&');
                out.extend_from_slice(LATIN1[(cp - 0xa0) as usize]);
                out.push(b';');
            } else if let Some((name, _)) = HTML401_EXT.iter().find(|&&(_, c)| c == cp) {
                out.push(b'&');
                out.extend_from_slice(name);
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
    if let Some(idx) = LATIN1.iter().position(|&name| name == ent) {
        return Some(encode_utf8(0xa0 + idx as u32));
    }
    // HTML 4.01 symbols/special (Greek, dashes, quotes, arrows, maths, …).
    HTML401_EXT
        .iter()
        .find(|&&(name, _)| name == ent)
        .map(|&(_, cp)| encode_utf8(cp))
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
    let (single, double, _apos_named) = flags_of(args, 1, ctx);
    Ok(Zval::Str(PhpStr::new(decode_all(&s, false, single, double))))
}

/// `html_entity_decode($string, $flags = …)`: reverse named (Latin-1) + numeric
/// entities as well as the five specials.
pub fn html_entity_decode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = super::string::str_at(args, ctx, 0, "html_entity_decode", 1)?;
    let (single, double, _apos_named) = flags_of(args, 1, ctx);
    Ok(Zval::Str(PhpStr::new(decode_all(&s, true, single, double))))
}
