//! ext/fileinfo: mime/charset detector modelled on PHP's *bundled* libmagic
//! (file 5.46 — the brew oracle links it, not the system one). The encoding
//! tables are `encoding.c` ported verbatim; the JSON/CSV builtin parsers are
//! `is_json.c`/`is_csv.c` ported verbatim; the signature table is a curated
//! subset of the magic database pinned on the WordPress test-suite corpus
//! (849-file ground truth, `finfo-truth` diff — MIME_TYPE/MIME/ENCODING are
//! the parity target; FILEINFO_NONE descriptions are best-effort and any
//! divergence is documented in PHPR_DIVERGENCES_FROM_PHP.md).
//!
//! One host builtin, `__finfo_detect(string $data, int $flags)`; the `finfo`
//! class, the procedural API, the directory/missing-file handling and all
//! I/O live PHP-side in `prelude_fileinfo.php` (gd/zlib pattern: streams and
//! wrappers keep their engine semantics).

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

const F_MIME_TYPE: i64 = 16; // MAGIC_MIME_TYPE
const F_MIME_ENCODING: i64 = 1024; // MAGIC_MIME_ENCODING
const F_EXTENSION: i64 = 1 << 24; // MAGIC_EXTENSION

/// file.h FILE_ENCODING_MAX: how much of the head feeds encoding detection.
const ENCODING_MAX: usize = 64 * 1024;
/// ascmagic.c MAXLINELEN (file.h): the "very long lines" threshold.
const MAXLINELEN: i64 = 300;

// ---------------------------------------------------------------------------
// encoding.c — ported verbatim
// ---------------------------------------------------------------------------

const F: u8 = 0; // never text
const T: u8 = 1; // plain ASCII text
const I: u8 = 2; // ISO-8859 text
const X: u8 = 3; // non-ISO extended ASCII (Mac, IBM PC)

#[rustfmt::skip]
const TEXT_CHARS: [u8; 256] = [
    //                BEL BS HT LF VT FF CR
    F, F, F, F, F, F, F, T, T, T, T, T, T, T, F, F, // 0x0X
    //                           ESC
    F, F, F, F, F, F, F, F, F, F, F, T, F, F, F, F, // 0x1X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, // 0x2X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, // 0x3X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, // 0x4X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, // 0x5X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, // 0x6X
    T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, F, // 0x7X
    //         NEL
    X, X, X, X, X, T, X, X, X, X, X, X, X, X, X, X, // 0x8X
    X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, X, // 0x9X
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xaX
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xbX
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xcX
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xdX
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xeX
    I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, I, // 0xfX
];

#[rustfmt::skip]
const EBCDIC_TO_ASCII: [u8; 256] = [
      0,   1,   2,   3, 156,   9, 134, 127, 151, 141, 142,  11,  12,  13,  14,  15,
     16,  17,  18,  19, 157, 133,   8, 135,  24,  25, 146, 143,  28,  29,  30,  31,
    128, 129, 130, 131, 132,  10,  23,  27, 136, 137, 138, 139, 140,   5,   6,   7,
    144, 145,  22, 147, 148, 149, 150,   4, 152, 153, 154, 155,  20,  21, 158,  26,
    b' ', 160, 161, 162, 163, 164, 165, 166, 167, 168, 213, b'.', b'<', b'(', b'+', b'|',
    b'&', 169, 170, 171, 172, 173, 174, 175, 176, 177, b'!', b'$', b'*', b')', b';', b'~',
    b'-', b'/', 178, 179, 180, 181, 182, 183, 184, 185, 203, b',', b'%', b'_', b'>', b'?',
    186, 187, 188, 189, 190, 191, 192, 193, 194, b'`', b':', b'#', b'@', b'\'', b'=', b'"',
    195, b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', 196, 197, 198, 199, 200, 201,
    202, b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b'^', 204, 205, 206, 207, 208,
    209, 229, b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', 210, 211, 212, b'[', 214, 215,
    216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, b']', 230, 231,
    b'{', b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', 232, 233, 234, 235, 236, 237,
    b'}', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', 238, 239, 240, 241, 242, 243,
    b'\\', 159, b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', 244, 245, 246, 247, 248, 249,
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', 250, 251, 252, 253, 254, 255,
];

struct Enc {
    /// The human name file_encoding leaves in `code` ("ASCII", "ISO-8859", …).
    code: &'static str,
    /// The charset printed after "; charset=" ("us-ascii", "binary", …).
    mime: &'static str,
    /// file_encoding's return: does the buffer look like text at all?
    text: bool,
    /// The decoded per-character buffer ascmagic inspects (`ubuf`).
    ubuf: Vec<u32>,
}

fn looks_with(buf: &[u8], reject: impl Fn(u8) -> bool) -> Option<Vec<u32>> {
    let mut ubuf = Vec::with_capacity(buf.len());
    for &b in buf {
        if reject(TEXT_CHARS[b as usize]) {
            return None;
        }
        ubuf.push(b as u32);
    }
    Some(ubuf)
}

fn looks_ascii(buf: &[u8]) -> Option<Vec<u32>> {
    looks_with(buf, |t| t != T)
}
fn looks_latin1(buf: &[u8]) -> Option<Vec<u32>> {
    looks_with(buf, |t| t != T && t != I)
}
fn looks_extended(buf: &[u8]) -> Option<Vec<u32>> {
    looks_with(buf, |t| t != T && t != I && t != X)
}

fn looks_utf7(buf: &[u8]) -> bool {
    buf.len() > 4
        && buf[0] == b'+'
        && buf[1] == b'/'
        && buf[2] == b'v'
        && matches!(buf[3], b'8' | b'9' | b'+' | b'/')
}

/// encoding.c file_looks_utf8: -1 invalid, 0 odd controls, 1 pure 7-bit,
/// 2 definitely UTF-8 (with the Go-derived first-byte/accept-range tables).
fn looks_utf8(buf: &[u8]) -> (i32, Vec<u32>) {
    // (accept-range index, total size); XX = invalid, AS = ascii.
    const XX: u8 = 0xF1;
    const AS: u8 = 0xF0;
    const S1: u8 = 0x02;
    const S2: u8 = 0x13;
    const S3: u8 = 0x03;
    const S4: u8 = 0x23;
    const S5: u8 = 0x34;
    const S6: u8 = 0x04;
    const S7: u8 = 0x44;
    #[rustfmt::skip]
    const FIRST: [u8; 256] = [
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS, AS,
        XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
        XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
        XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
        XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
        XX, XX, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1,
        S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1, S1,
        S2, S3, S3, S3, S3, S3, S3, S3, S3, S3, S3, S3, S3, S4, S3, S3,
        S5, S6, S6, S6, S7, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX, XX,
    ];
    const ACCEPT: [(u8, u8); 5] = [(0x80, 0xBF), (0xA0, 0xBF), (0x80, 0x9F), (0x90, 0xBF), (0x80, 0x8F)];

    let mut ubuf = Vec::with_capacity(buf.len());
    let (mut gotone, mut ctrl) = (false, false);
    let mut i = 0usize;
    while i < buf.len() {
        let b = buf[i];
        if b & 0x80 == 0 {
            if TEXT_CHARS[b as usize] != T {
                ctrl = true;
            }
            ubuf.push(b as u32);
        } else if b & 0x40 == 0 {
            return (-1, ubuf);
        } else {
            let x = FIRST[b as usize];
            if x == XX {
                return (-1, ubuf);
            }
            let ar = ACCEPT[(x >> 4) as usize];
            let (mut c, following) = if b & 0x20 == 0 {
                ((b & 0x1f) as u32, 1)
            } else if b & 0x10 == 0 {
                ((b & 0x0f) as u32, 2)
            } else if b & 0x08 == 0 {
                ((b & 0x07) as u32, 3)
            } else if b & 0x04 == 0 {
                ((b & 0x03) as u32, 4)
            } else if b & 0x02 == 0 {
                ((b & 0x01) as u32, 5)
            } else {
                return (-1, ubuf);
            };
            for n in 0..following {
                i += 1;
                if i >= buf.len() {
                    // Truncated tail sequence: C's `goto done` keeps the verdict.
                    return (if ctrl { 0 } else if gotone { 2 } else { 1 }, ubuf);
                }
                let bb = buf[i];
                if n == 0 && (bb < ar.0 || bb > ar.1) {
                    return (-1, ubuf);
                }
                if bb & 0x80 == 0 || bb & 0x40 != 0 {
                    return (-1, ubuf);
                }
                c = (c << 6) + (bb & 0x3f) as u32;
            }
            ubuf.push(c);
            gotone = true;
        }
        i += 1;
    }
    (if ctrl { 0 } else if gotone { 2 } else { 1 }, ubuf)
}

fn looks_ucs16(buf: &[u8]) -> Option<(bool, Vec<u32>)> {
    if buf.len() < 2 {
        return None;
    }
    let bigend = if buf[0] == 0xff && buf[1] == 0xfe {
        false
    } else if buf[0] == 0xfe && buf[1] == 0xff {
        true
    } else {
        return None;
    };
    let mut ubuf = Vec::new();
    let mut hi: u32 = 0;
    let mut i = 2usize;
    while i + 1 < buf.len() {
        let mut uc = if bigend {
            (buf[i + 1] as u32) | ((buf[i] as u32) << 8)
        } else {
            (buf[i] as u32) | ((buf[i + 1] as u32) << 8)
        } & 0xffff;
        if uc == 0xfffe || uc == 0xffff || (0xfdd0..=0xfdef).contains(&uc) {
            return None;
        }
        if hi != 0 {
            if !(0xdc00..=0xdfff).contains(&uc) {
                return None;
            }
            uc = 0x10000 + 0x400 * (hi - 1) + (uc - 0xdc00);
            hi = 0;
        }
        if uc < 128 && TEXT_CHARS[uc as usize] != T {
            return None;
        }
        ubuf.push(uc);
        if (0xd800..=0xdbff).contains(&uc) {
            hi = uc - 0xd800 + 1;
        }
        if (0xdc00..=0xdfff).contains(&uc) {
            return None;
        }
        i += 2;
    }
    Some((bigend, ubuf))
}

fn looks_ucs32(buf: &[u8]) -> Option<(bool, Vec<u32>)> {
    if buf.len() < 4 {
        return None;
    }
    let bigend = if buf[0] == 0xff && buf[1] == 0xfe && buf[2] == 0 && buf[3] == 0 {
        false
    } else if buf[0] == 0 && buf[1] == 0 && buf[2] == 0xfe && buf[3] == 0xff {
        true
    } else {
        return None;
    };
    let mut ubuf = Vec::new();
    let mut i = 4usize;
    while i + 3 < buf.len() {
        let uc = if bigend {
            (buf[i + 3] as u32) | ((buf[i + 2] as u32) << 8) | ((buf[i + 1] as u32) << 16) | ((buf[i] as u32) << 24)
        } else {
            (buf[i] as u32) | ((buf[i + 1] as u32) << 8) | ((buf[i + 2] as u32) << 16) | ((buf[i + 3] as u32) << 24)
        };
        if uc == 0xfffe {
            return None;
        }
        if uc < 128 && TEXT_CHARS[uc as usize] != T {
            return None;
        }
        ubuf.push(uc);
        i += 4;
    }
    Some((bigend, ubuf))
}

/// encoding.c file_encoding — same decision cascade, on at most
/// [`ENCODING_MAX`] bytes.
fn file_encoding(full: &[u8]) -> Enc {
    let buf = &full[..full.len().min(ENCODING_MAX)];
    if let Some(ubuf) = looks_ascii(buf) {
        if looks_utf7(buf) {
            return Enc { code: "Unicode text, UTF-7", mime: "utf-7", text: true, ubuf };
        }
        return Enc { code: "ASCII", mime: "us-ascii", text: true, ubuf };
    }
    if buf.len() > 3 && buf[0] == 0xef && buf[1] == 0xbb && buf[2] == 0xbf {
        let (rv, ubuf) = looks_utf8(&buf[3..]);
        if rv > 0 {
            return Enc { code: "Unicode text, UTF-8 (with BOM)", mime: "utf-8", text: true, ubuf };
        }
    }
    {
        let (rv, ubuf) = looks_utf8(buf);
        if rv > 1 {
            return Enc { code: "Unicode text, UTF-8", mime: "utf-8", text: true, ubuf };
        }
    }
    if let Some((bigend, ubuf)) = looks_ucs32(buf) {
        return if bigend {
            Enc { code: "Unicode text, UTF-32, big-endian", mime: "utf-32be", text: true, ubuf }
        } else {
            Enc { code: "Unicode text, UTF-32, little-endian", mime: "utf-32le", text: true, ubuf }
        };
    }
    if let Some((bigend, ubuf)) = looks_ucs16(buf) {
        return if bigend {
            Enc { code: "Unicode text, UTF-16, big-endian", mime: "utf-16be", text: true, ubuf }
        } else {
            Enc { code: "Unicode text, UTF-16, little-endian", mime: "utf-16le", text: true, ubuf }
        };
    }
    if let Some(ubuf) = looks_latin1(buf) {
        return Enc { code: "ISO-8859", mime: "iso-8859-1", text: true, ubuf };
    }
    if let Some(ubuf) = looks_extended(buf) {
        return Enc { code: "Non-ISO extended-ASCII", mime: "unknown-8bit", text: true, ubuf };
    }
    let converted: Vec<u8> = buf.iter().map(|&b| EBCDIC_TO_ASCII[b as usize]).collect();
    if let Some(ubuf) = looks_ascii(&converted) {
        return Enc { code: "EBCDIC", mime: "ebcdic", text: true, ubuf };
    }
    if let Some(ubuf) = looks_latin1(&converted) {
        return Enc { code: "International EBCDIC", mime: "ebcdic", text: true, ubuf };
    }
    Enc { code: "unknown", mime: "binary", text: false, ubuf: Vec::new() }
}

// ---------------------------------------------------------------------------
// is_json.c — ported verbatim (accept iff an object or a closed array occurs;
// a second top-level value of the same kind makes it NDJSON)
// ---------------------------------------------------------------------------

#[derive(Default)]
struct JsonSt {
    object: usize,
    arrayn: usize,
}

fn json_skip_space(buf: &[u8], mut i: usize) -> usize {
    while i < buf.len() && matches!(buf[i], b' ' | b'\n' | b'\r' | b'\t') {
        i += 1;
    }
    i
}

fn json_parse_string(buf: &[u8], mut i: usize) -> (bool, usize) {
    while i < buf.len() {
        let c = buf[i];
        i += 1;
        match c {
            0 => return (false, i),
            b'\\' => {
                if i == buf.len() {
                    return (false, i);
                }
                let e = buf[i];
                i += 1;
                match e {
                    0 => return (false, i),
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                    b'u' => {
                        if buf.len() - i < 4 {
                            return (false, buf.len());
                        }
                        for _ in 0..4 {
                            if !buf[i].is_ascii_hexdigit() {
                                return (false, i + 1);
                            }
                            i += 1;
                        }
                    }
                    _ => return (false, i),
                }
            }
            b'"' => return (true, i),
            _ => {}
        }
    }
    (false, i)
}

fn json_parse_array(buf: &[u8], mut i: usize, st: &mut JsonSt, lvl: usize) -> (bool, usize) {
    while i < buf.len() {
        i = json_skip_space(buf, i);
        if i == buf.len() {
            return (false, i);
        }
        if buf[i] == b']' {
            st.arrayn += 1;
            return (true, i + 1);
        }
        let (ok, ni) = json_parse_value(buf, i, st, lvl + 1);
        i = ni;
        if !ok || i == buf.len() {
            return (false, i);
        }
        match buf[i] {
            b',' => i += 1,
            b']' => {
                st.arrayn += 1;
                return (true, i + 1);
            }
            _ => return (false, i),
        }
    }
    (false, i)
}

fn json_parse_object(buf: &[u8], mut i: usize, st: &mut JsonSt, lvl: usize) -> (bool, usize) {
    while i < buf.len() {
        i = json_skip_space(buf, i);
        if i == buf.len() {
            return (false, i);
        }
        if buf[i] == b'}' {
            return (true, i + 1);
        }
        if buf[i] != b'"' {
            return (false, i + 1);
        }
        let (ok, ni) = json_parse_string(buf, i + 1);
        i = ni;
        if !ok {
            return (false, i);
        }
        i = json_skip_space(buf, i);
        if i == buf.len() {
            return (false, i);
        }
        if buf[i] != b':' {
            return (false, i + 1);
        }
        let (ok, ni) = json_parse_value(buf, i + 1, st, lvl + 1);
        i = ni;
        if !ok || i == buf.len() {
            return (false, i);
        }
        let c = buf[i];
        i += 1;
        match c {
            b',' => {}
            b'}' => return (true, i),
            _ => return (false, i - 1),
        }
    }
    (false, i)
}

fn json_parse_number(buf: &[u8], mut i: usize) -> (bool, usize) {
    let mut got = false;
    if i == buf.len() {
        return (false, i);
    }
    if buf[i] == b'-' {
        i += 1;
    }
    while i < buf.len() && buf[i].is_ascii_digit() {
        got = true;
        i += 1;
    }
    if i == buf.len() {
        return (got, i);
    }
    if buf[i] == b'.' {
        i += 1;
    }
    while i < buf.len() && buf[i].is_ascii_digit() {
        got = true;
        i += 1;
    }
    if i == buf.len() {
        return (got, i);
    }
    if got && (buf[i] == b'e' || buf[i] == b'E') {
        i += 1;
        got = false;
        if i == buf.len() {
            return (false, i);
        }
        if buf[i] == b'+' || buf[i] == b'-' {
            i += 1;
        }
        while i < buf.len() && buf[i].is_ascii_digit() {
            got = true;
            i += 1;
        }
    }
    (got, i)
}

fn json_parse_const(buf: &[u8], i: usize, s: &[u8]) -> (bool, usize) {
    // C quirks kept: `len` there is sizeof() (NUL included), the cursor
    // advances len-2 past the dispatch byte, and a buffer that ends mid-word
    // still counts as a good constant.
    let mut ok = true;
    for k in 1..s.len() {
        if i + k - 1 >= buf.len() {
            break;
        }
        if buf[i + k - 1] != s[k] {
            ok = false;
            break;
        }
    }
    (ok, (i + s.len() - 1).min(buf.len()))
}

fn json_parse_value(buf: &[u8], i: usize, st: &mut JsonSt, lvl: usize) -> (bool, usize) {
    let start = json_skip_space(buf, i);
    if start == buf.len() || lvl > 500 {
        return (false, start);
    }
    let c = buf[start];
    let i = start + 1;
    let (rv, ni) = match c {
        b'"' => json_parse_string(buf, i),
        b'[' => json_parse_array(buf, i, st, lvl),
        b'{' => {
            let (rv, ni) = json_parse_object(buf, i, st, lvl);
            if rv {
                st.object += 1;
            }
            (rv, ni)
        }
        b't' => json_parse_const(buf, i, b"true"),
        b'f' => json_parse_const(buf, i, b"false"),
        b'n' => json_parse_const(buf, i, b"null"),
        _ => json_parse_number(buf, start),
    };
    (rv, json_skip_space(buf, ni))
}

/// 0 = not JSON, 1 = JSON, 2 = NDJSON.
fn is_json(buf: &[u8]) -> u8 {
    let mut st = JsonSt::default();
    let start = json_skip_space(buf, 0);
    if start == buf.len() {
        return 0;
    }
    let first = buf[start];
    let (rv, i) = json_parse_value(buf, 0, &mut st, 1);
    if !rv {
        return 0;
    }
    let hit = |st: &JsonSt| st.arrayn > 0 || st.object > 0;
    if i == buf.len() {
        return if hit(&st) { 1 } else { 0 };
    }
    if buf[i] == first {
        let mut ok = true;
        let mut j = i;
        while j < buf.len() {
            let (rv, nj) = json_parse_value(buf, j, &mut st, 2);
            if !rv {
                ok = false;
                break;
            }
            j = nj;
        }
        if ok && hit(&st) {
            return 2;
        }
    }
    0
}

// ---------------------------------------------------------------------------
// is_csv.c — ported verbatim (≥3 fields per line, constant, ≥2 lines;
// only the first CSV_LINES=10 lines are checked)
// ---------------------------------------------------------------------------

fn is_csv(buf: &[u8]) -> bool {
    let (mut nf, mut tf, mut nl) = (0usize, 0usize, 0usize);
    let mut i = 0usize;
    while i < buf.len() {
        let c = buf[i];
        i += 1;
        match c {
            b'"' => {
                // eatquote: skip to the char after the closing quote
                // (quote-quote escapes stay inside).
                let mut quote = false;
                while i < buf.len() {
                    let q = buf[i];
                    i += 1;
                    if q != b'"' {
                        if quote {
                            i -= 1;
                            break;
                        }
                    } else {
                        quote = !quote;
                    }
                }
            }
            b',' => nf += 1,
            b'\n' => {
                nl += 1;
                if nl == 10 {
                    return tf > 1 && tf == nf;
                }
                if tf == 0 {
                    if nf == 0 {
                        return false;
                    }
                    tf = nf;
                } else if tf != nf {
                    return false;
                }
                nf = 0;
            }
            _ => {}
        }
    }
    tf > 1 && nl >= 2
}

// ---------------------------------------------------------------------------
// signature table (BINTEST-phase): the magic entries the WP corpus exercises
// ---------------------------------------------------------------------------

struct Det {
    /// Empty string = the magic entry has no `!:mime` line: the description
    /// stands, but the mime/charset fall through to the text pipeline
    /// (e.g. "\x7fELF" is "ELF" in NONE mode and EBCDIC text in MIME mode).
    mime: String,
    /// Raw bytes: file_printable passes ISO-8859 bytes through untouched,
    /// so a description is not guaranteed to be UTF-8.
    desc: Vec<u8>,
    ext: &'static str,
}

fn det(mime: &str, desc: String, ext: &'static str) -> Det {
    Det { mime: mime.to_string(), desc: desc.into_bytes(), ext }
}

fn be16(d: &[u8], o: usize) -> u32 {
    ((d[o] as u32) << 8) | d[o + 1] as u32
}
fn le16(d: &[u8], o: usize) -> u32 {
    ((d[o + 1] as u32) << 8) | d[o] as u32
}
fn be32(d: &[u8], o: usize) -> u32 {
    ((d[o] as u32) << 24) | ((d[o + 1] as u32) << 16) | ((d[o + 2] as u32) << 8) | d[o + 3] as u32
}
fn le32(d: &[u8], o: usize) -> u32 {
    ((d[o + 3] as u32) << 24) | ((d[o + 2] as u32) << 16) | ((d[o + 1] as u32) << 8) | d[o] as u32
}

fn find(hay: &[u8], needle: &[u8], limit: usize) -> Option<usize> {
    let end = hay.len().min(limit);
    if needle.is_empty() || end < needle.len() {
        return None;
    }
    (0..=end - needle.len()).find(|&i| &hay[i..i + needle.len()] == needle)
}

/// ISO-BMFF `ftyp` major brands → (mime, description).
fn ftyp_brand(d: &[u8]) -> Option<Det> {
    if d.len() < 12 || &d[4..8] != b"ftyp" {
        return None;
    }
    let brand = &d[8..12];
    let (mime, desc): (&str, &str) = match brand {
        b"avif" => ("image/avif", "ISO Media, AVIF Image"),
        b"avis" => ("image/avif", "ISO Media, AVIF Image Sequence"),
        b"heic" | b"heix" => ("image/heic", "ISO Media, HEIF Image HEVC Main or Main Still Picture Profile"),
        b"hevc" | b"hevx" => ("image/heic-sequence", "ISO Media, HEIF Image Sequence HEVC Main or Main Still Picture Profile"),
        b"mif1" => ("image/heif", "ISO Media, HEIF Image"),
        b"msf1" => ("image/heif-sequence", "ISO Media, HEIF Image Sequence"),
        b"qt  " => ("video/quicktime", "ISO Media, Apple QuickTime movie, Apple QuickTime (.MOV/QT)"),
        b"M4A " => ("audio/x-m4a", "ISO Media, Apple iTunes ALAC/AAC-LC (.M4A) Audio"),
        b"M4V " => ("video/x-m4v", "ISO Media, Apple iTunes Video (.M4V) Video"),
        b"mp41" => ("video/mp4", "ISO Media, MP4 v1 [ISO 14496-1:ch13]"),
        b"mp42" => ("video/mp4", "ISO Media, MP4 v2 [ISO 14496-14]"),
        b"isom" => ("video/mp4", "ISO Media, MP4 Base Media v1 [ISO 14496-12:2003]"),
        b"iso2" => ("video/mp4", "ISO Media, MP4 Base Media v2 [ISO 14496-12:2005]"),
        b"avc1" => ("video/mp4", "ISO Media, MP4 Base w/ AVC ext [ISO 14496-12:2005]"),
        b"dash" => ("video/mp4", "ISO Media, MPEG v4 system, Dynamic Adaptive Streaming over HTTP"),
        _ if brand.starts_with(b"3gp") => ("video/3gpp", "ISO Media, MPEG v4 system, 3GPP"),
        _ if brand.starts_with(b"3g2") => ("video/3gpp2", "ISO Media, MPEG v4 system, 3GPP2"),
        _ => return None,
    };
    Some(det(mime, desc.to_string(), "???"))
}

fn zip_deep(d: &[u8]) -> Det {
    // Local file header: name length at 26, extra length at 28, name at 30.
    if d.len() >= 30 {
        let nlen = le16(d, 26) as usize;
        let xlen = le16(d, 28) as usize;
        if d.len() >= 30 + nlen {
            let name = &d[30..30 + nlen];
            if name == b"mimetype" && d.len() >= 30 + nlen + xlen + 20 {
                let body = &d[30 + nlen + xlen..];
                if body.starts_with(b"application/epub+zip") {
                    return det("application/epub+zip", "EPUB document".to_string(), "???");
                }
            }
            // Google-docs-style zips put a payload entry first (data
            // descriptors, no [Content_Types].xml up front): the msooxml
            // magic matches the member name directly.
            if name.starts_with(b"word/") {
                return det(
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                    "Microsoft Word 2007+".to_string(),
                    "???",
                );
            }
            if name.starts_with(b"xl/") {
                return det(
                    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                    "Microsoft Excel 2007+".to_string(),
                    "???",
                );
            }
            if name.starts_with(b"ppt/") {
                return det(
                    "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                    "Microsoft PowerPoint 2007+".to_string(),
                    "???",
                );
            }
            if name == b"[Content_Types].xml" || name.starts_with(b"_rels/") || name.starts_with(b"docProps") {
                if find(d, b"word/", 4096).is_some() {
                    return det(
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                        "Microsoft Word 2007+".to_string(),
                        "???",
                    );
                }
                if find(d, b"xl/", 4096).is_some() {
                    return det(
                        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                        "Microsoft Excel 2007+".to_string(),
                        "???",
                    );
                }
                if find(d, b"ppt/", 4096).is_some() {
                    return det(
                        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                        "Microsoft PowerPoint 2007+".to_string(),
                        "???",
                    );
                }
                return det("application/zip", "Microsoft OOXML".to_string(), "???");
            }
        }
    }
    // Version needed to extract, at offset 4 (v/10.v%10).
    let mut desc = "Zip archive data".to_string();
    if d.len() >= 6 {
        let v = le16(d, 4);
        desc.push_str(&format!(", at least v{}.{} to extract", v / 10, v % 10));
        if d.len() >= 10 {
            let method = le16(d, 8);
            let m = match method {
                0 => Some("store"),
                8 => Some("deflate"),
                9 => Some("deflate64"),
                12 => Some("bzip2"),
                14 => Some("lzma"),
                93 => Some("zstd"),
                95 => Some("xz"),
                99 => Some("AES Encrypted"),
                _ => None,
            };
            if let Some(m) = m {
                desc.push_str(&format!(", compression method={m}"));
            }
        }
    }
    det("application/zip", desc, "???")
}

fn flac_desc(d: &[u8]) -> String {
    let mut desc = "FLAC audio bitstream data".to_string();
    // STREAMINFO: 4 sig + 4 block header, sample rate 20 bits at byte 18,
    // channels 3 bits, bits-per-sample 5 bits, total samples 36 bits.
    if d.len() >= 8 + 34 && d[4] & 0x7f == 0 {
        let s = &d[8..];
        let rate = ((s[10] as u32) << 12) | ((s[11] as u32) << 4) | ((s[12] as u32) >> 4);
        let channels = ((s[12] >> 1) & 0x07) + 1;
        let bps = (((s[12] & 0x01) << 4) | (s[13] >> 4)) + 1;
        let total: u64 = (((s[13] & 0x0f) as u64) << 32)
            | ((s[14] as u64) << 24)
            | ((s[15] as u64) << 16)
            | ((s[16] as u64) << 8)
            | (s[17] as u64);
        desc.push_str(&format!(", {bps} bit"));
        match channels {
            1 => desc.push_str(", mono"),
            2 => desc.push_str(", stereo"),
            n => desc.push_str(&format!(", {n} channels")),
        }
        let khz = rate as f64 / 1000.0;
        if (khz - khz.round()).abs() < 1e-9 {
            desc.push_str(&format!(", {} kHz", khz.round() as u64));
        } else {
            desc.push_str(&format!(", {khz} kHz"));
        }
        if total > 0 {
            desc.push_str(&format!(", {total} samples"));
        }
    }
    desc
}

fn woff_desc(d: &[u8], v2: bool) -> String {
    let mut desc = if v2 {
        "Web Open Font Format (Version 2)".to_string()
    } else {
        "Web Open Font Format".to_string()
    };
    if d.len() >= 12 {
        match &d[4..8] {
            [0x00, 0x01, 0x00, 0x00] => desc.push_str(", TrueType"),
            b"OTTO" => desc.push_str(", CFF"),
            b"true" => desc.push_str(", TrueType"),
            _ => {}
        }
        desc.push_str(&format!(", length {}", be32(d, 8)));
    }
    let vo = if v2 { 24 } else { 20 };
    if d.len() >= vo + 4 {
        desc.push_str(&format!(", version {}.{}", be16(d, vo), be16(d, vo + 2)));
    }
    desc
}

/// MPEG audio frame-sync sanity: layer III only, like the raw-stream ADTS
/// entries the oracle actually fires (a wider net would eat UTF-16 BOMs —
/// 0xFF 0xFE parses as a "valid" layer-I header).
fn mpeg_audio_frame(d: &[u8]) -> bool {
    d.len() >= 4
        && d[0] == 0xff
        && d[1] & 0xe0 == 0xe0
        && (d[1] >> 3) & 0x03 != 0x01 // version reserved
        && (d[1] >> 1) & 0x03 == 0x01 // layer III
        && d[2] >> 4 != 0x0f // bitrate bad
        && d[2] >> 4 != 0x00 // free-form
        && (d[2] >> 2) & 0x03 != 0x03 // samplerate reserved
}

/// Magdir/gnu .mo detail: revision, message count, the PO-Revision-Date
/// header line from the metadata msgstr, and the first real msgid quoted.
fn mo_desc(d: &[u8], be: bool) -> Vec<u8> {
    let rd = |o: usize| -> Option<u32> {
        if d.len() < o + 4 {
            return None;
        }
        Some(if be { be32(d, o) } else { le32(d, o) })
    };
    let mut desc: Vec<u8> = if be {
        b"GNU message catalog (big endian)".to_vec()
    } else {
        b"GNU message catalog (little endian)".to_vec()
    };
    let Some(rev) = rd(4) else { return desc };
    desc.extend_from_slice(format!(", revision {}.{}", rev >> 16, rev & 0xffff).as_bytes());
    let Some(n) = rd(8) else { return desc };
    desc.extend_from_slice(format!(", {n} messages").as_bytes());
    let (Some(_orig_off), Some(trans_off)) = (rd(12), rd(16)) else { return desc };
    let string_at = |table: u32, idx: u32| -> Option<&[u8]> {
        let e = table as usize + 8 * idx as usize;
        let len = rd(e)? as usize;
        let off = rd(e + 4)? as usize;
        d.get(off..off + len)
    };
    // The oracle prints the first line of translation 0 (the metadata
    // msgstr, or the first real one when there is no header), then the
    // first line of translation 1 quoted; both left-trimmed, cut at the
    // first newline, capped at 127 bytes and run through file_printable.
    let first_line = |s: &[u8]| -> Vec<u8> {
        let mut s = s;
        while let Some((&c, rest)) = s.split_first() {
            if matches!(c, b' ' | b'\t' | b'\r' | b'\n') {
                s = rest;
            } else {
                break;
            }
        }
        let end = s.iter().position(|&b| b == b'\n' || b == 0).unwrap_or(s.len());
        printable(&s[..end.min(127)])
    };
    if let Some(meta) = string_at(trans_off, 0) {
        desc.extend_from_slice(b", ");
        desc.extend_from_slice(&first_line(meta));
    }
    if n > 1 {
        if let Some(t1) = string_at(trans_off, 1) {
            desc.extend_from_slice(b" '");
            desc.extend_from_slice(&first_line(t1));
            desc.push(b'\'');
        }
    }
    desc
}

/// funcs.c file_printable, as the oracle behaves: printable ASCII and
/// ISO-8859-printable high bytes (class I, 0xA0+) pass RAW — even mid-UTF-8 —
/// while control ASCII and the 0x80–0x9F range (class X) are octal-escaped.
fn printable(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for &c in s {
        let raw = (0x20..0x7f).contains(&c) || TEXT_CHARS[c as usize] == I;
        if raw {
            out.push(c);
        } else {
            out.extend_from_slice(format!("\\{c:03o}").as_bytes());
        }
    }
    out
}

fn softmagic_bin(d: &[u8]) -> Option<Det> {
    let n = d.len();
    // --- images ---
    if d.starts_with(b"\x89PNG\r\n\x1a\n") {
        let mut desc = "PNG image data".to_string();
        if n >= 33 && &d[12..16] == b"IHDR" {
            let (w, h) = (be32(d, 16), be32(d, 20));
            let bits = d[24];
            let color = match d[25] {
                0 => " grayscale",
                2 => "/color RGB",
                3 => " colormap",
                4 => " gray+alpha",
                6 => "/color RGBA",
                _ => "",
            };
            let inter = if d[28] != 0 { "interlaced" } else { "non-interlaced" };
            desc.push_str(&format!(", {w} x {h}, {bits}-bit{color}, {inter}"));
        }
        return Some(det("image/png", desc, "png"));
    }
    if d.starts_with(b"\xff\xd8\xff") {
        return Some(det("image/jpeg", jpeg_desc(d), "jpeg/jpg/jpe/jfif"));
    }
    if d.starts_with(b"GIF87a") || d.starts_with(b"GIF89a") {
        let mut desc = format!("GIF image data, version {}", String::from_utf8_lossy(&d[3..6]));
        if n >= 10 {
            desc.push_str(&format!(", {} x {}", le16(d, 6), le16(d, 8)));
        }
        return Some(det("image/gif", desc, "gif"));
    }
    if d.starts_with(b"RIFF") && n >= 12 {
        match &d[8..12] {
            b"WEBP" => {
                let mut desc = "RIFF (little-endian) data, Web/P image".to_string();
                if n >= 16 {
                    match &d[12..16] {
                        b"VP8 " if n >= 30 => {
                            let scale = ["[none]", "[5/4]", "[5/3]", "[2]"];
                            let (wv, hv) = (le16(d, 26), le16(d, 28));
                            desc.push_str(&format!(
                                ", VP8 encoding, {}x{}, Scaling: {}x{}, YUV color, decoders should clamp",
                                wv & 0x3fff,
                                hv & 0x3fff,
                                scale[(wv >> 14) as usize],
                                scale[(hv >> 14) as usize]
                            ));
                        }
                        b"VP8L" => desc.push_str(", lossless"),
                        b"VP8X" if n >= 30 => {
                            let flags = d[20];
                            if flags & 0x20 != 0 {
                                desc.push_str(", ICC profile");
                            }
                            if flags & 0x02 != 0 {
                                desc.push_str(", animated");
                            }
                            if flags & 0x04 != 0 {
                                desc.push_str(", XMP metadata");
                            }
                            if flags & 0x08 != 0 {
                                desc.push_str(", EXIF metadata");
                            }
                            if flags & 0x10 != 0 {
                                desc.push_str(", with alpha");
                            }
                            let w1 = (d[24] as u32) | ((d[25] as u32) << 8) | ((d[26] as u32) << 16);
                            let h1 = (d[27] as u32) | ((d[28] as u32) << 8) | ((d[29] as u32) << 16);
                            desc.push_str(&format!(", {w1}+1x{h1}+1"));
                        }
                        _ => {}
                    }
                }
                return Some(det("image/webp", desc, "webp"));
            }
            b"WAVE" => return Some(det("audio/x-wav", "RIFF (little-endian) data, WAVE audio".to_string(), "wav")),
            fourcc if fourcc.starts_with(b"AVI") => {
                return Some(det("video/x-msvideo", "RIFF (little-endian) data, AVI".to_string(), "avi"))
            }
            _ => return Some(det("", "RIFF (little-endian) data".to_string(), "???")),
        }
    }
    if d.starts_with(b"RIFF") {
        return Some(det("", "RIFF (little-endian) data".to_string(), "???"));
    }
    if d.starts_with(b"\x7fELF") {
        let mut desc = "ELF".to_string();
        if n > 4 {
            match d[4] {
                1 => desc.push_str(" 32-bit"),
                2 => desc.push_str(" 64-bit"),
                _ => {}
            }
        }
        if n > 5 {
            match d[5] {
                1 => desc.push_str(" LSB"),
                2 => desc.push_str(" MSB"),
                _ => {}
            }
        }
        return Some(det("", desc, "???"));
    }
    if d.starts_with(b"\x00\x00\x001\x00\x00\x000\x00\x00\x000\x00\x00\x002\x00\x00\x000\x00\x00\x000\x00\x00\x003") {
        let mut desc = "old ACE/gr binary file".to_string();
        if n > 39 && d[39] > 0 {
            desc.push_str(&format!(" - version {}", d[39] as char));
        }
        return Some(det("", desc, "???"));
    }
    if d.starts_with(b"\x55\x7a\x6e\x61") {
        let mut desc = "xo65 object,".to_string();
        if n >= 6 {
            desc.push_str(&format!(" version {},", le16(d, 4)));
        }
        if n >= 8 {
            desc.push_str(if le16(d, 6) & 1 == 1 { " with debug info" } else { " no debug info" });
        }
        return Some(det("", desc, "???"));
    }
    if d.starts_with(b"id=ImageMagick") {
        return Some(det("image/x-miff", "MIFF image data".to_string(), "mif/miff"));
    }
    if d.starts_with(b"II*\x00") {
        let desc = format!("TIFF image data, little-endian{}", tiff_ifd_desc(d, 0, false));
        return Some(det("image/tiff", desc, "tif/tiff"));
    }
    if d.starts_with(b"MM\x00*") {
        let desc = format!("TIFF image data, big-endian{}", tiff_ifd_desc(d, 0, true));
        return Some(det("image/tiff", desc, "tif/tiff"));
    }
    if n >= 6 && d[0] == 0 && d[1] == 0 && d[2] == 1 && d[3] == 0 && le16(d, 4) > 0 && le16(d, 4) < 256 {
        let count = le16(d, 4);
        let mut desc = format!("MS Windows icon resource - {count} icon{}", if count == 1 { "" } else { "s" });
        if n >= 8 {
            let (w, h) = (d[6] as u32, d[7] as u32);
            desc.push_str(&format!(
                ", {}x{}",
                if w == 0 { 256 } else { w },
                if h == 0 { 256 } else { h }
            ));
            if n >= 14 {
                let bits = le16(d, 12);
                if bits > 0 {
                    desc.push_str(&format!(", {bits} bits/pixel"));
                }
            }
        }
        return Some(det("image/vnd.microsoft.icon", desc, "ico"));
    }
    if d.starts_with(b"8BPS") {
        let mut desc = "Adobe Photoshop Image".to_string();
        if n >= 26 {
            let channels = be16(d, 12);
            let h = be32(d, 14);
            let w = be32(d, 18);
            let depth = be16(d, 22);
            let mode = be16(d, 24);
            desc.push_str(&format!(", {w} x {h}"));
            let mode_name = match mode {
                0 => "bitmap",
                1 => "grayscale",
                2 => "indexed",
                3 => "RGB",
                4 => "CMYK",
                7 => "multichannel",
                8 => "duotone",
                9 => "lab",
                _ => "",
            };
            if !mode_name.is_empty() {
                desc.push_str(&format!(", {mode_name}"));
                if mode == 3 && channels == 4 {
                    desc.push('A');
                }
            }
            desc.push_str(&format!(", {channels}x {depth}-bit channels"));
        }
        return Some(det("image/vnd.adobe.photoshop", desc, "psd"));
    }
    if d.starts_with(b"\x00\x00\x00\x0cjP  \r\n\x87\n") {
        // JPEG-2000 container: the brand at 20..24 picks the family member.
        let mime = if n >= 24 {
            match &d[20..24] {
                b"jp2 " => "image/jp2",
                b"jpx " => "image/jpx",
                b"jpm " => "image/jpm",
                b"mjp2" => "video/mj2",
                _ => "image/jp2",
            }
        } else {
            "image/jp2"
        };
        return Some(det(mime, "JPEG 2000 Part 1 (JP2)".to_string(), "jp2"));
    }
    if let Some(x) = ftyp_brand(d) {
        return Some(x);
    }
    if d.starts_with(b"\x01\xda") && n >= 12 && d[2] <= 1 && (d[3] == 1 || d[3] == 2) {
        let mut desc = "SGI image data".to_string();
        if d[2] == 1 {
            desc.push_str(", RLE");
        }
        let dim = be16(d, 4);
        if dim == 3 {
            desc.push_str(", 3-D");
        }
        desc.push_str(&format!(", {} x {}", be16(d, 6), be16(d, 8)));
        if dim == 3 {
            desc.push_str(&format!(", {} channels", be16(d, 10)));
        }
        return Some(det("image/x-sgi", desc, "sgi"));
    }
    if d.starts_with(b"BM") && n >= 18 {
        let dib = le32(d, 14);
        if matches!(dib, 12 | 40 | 52 | 56 | 64 | 108 | 124) {
            let mut desc = "PC bitmap".to_string();
            if dib == 40 && n >= 46 {
                desc.push_str(&format!(
                    ", Windows 3.x format, {} x {} x {}",
                    le32(d, 18) as i32,
                    le32(d, 22) as i32,
                    le16(d, 28)
                ));
                let (xr, yr) = (le32(d, 38), le32(d, 42));
                if xr > 0 || yr > 0 {
                    desc.push_str(&format!(", resolution {xr} x {yr} px/m"));
                }
                desc.push_str(&format!(", cbSize {}, bits offset {}", le32(d, 2), le32(d, 10)));
            }
            return Some(det("image/bmp", desc, "bmp"));
        }
    }
    // --- fonts ---
    if d.starts_with(b"wOFF") {
        return Some(det("font/woff", woff_desc(d, false), "woff"));
    }
    if d.starts_with(b"wOF2") {
        return Some(det("font/woff2", woff_desc(d, true), "woff2"));
    }
    if d.starts_with(b"\x00\x01\x00\x00") && n >= 6 {
        let mut desc = "TrueType Font data".to_string();
        let tables = be16(d, 4) as usize;
        if n >= 12 + 16 * tables && tables > 0 {
            desc.push_str(&format!(", {tables} tables"));
            desc.push_str(&format!(", 1st \"{}\"", String::from_utf8_lossy(&d[12..16])));
            // The name table: count, then the first record's platform/language.
            for t in 0..tables {
                let e = 12 + 16 * t;
                if &d[e..e + 4] == b"name" {
                    let off = be32(d, e + 8) as usize;
                    if n >= off + 18 {
                        let names = be16(d, off + 2);
                        desc.push_str(&format!(", {names} names"));
                        let platform = be16(d, off + 6);
                        match platform {
                            0 => desc.push_str(", Unicode"),
                            1 => desc.push_str(", Macintosh"),
                            3 => desc.push_str(", Microsoft"),
                            _ => {}
                        }
                        desc.push_str(&format!(", language {:#x}", be16(d, off + 10)));
                    }
                    break;
                }
            }
        }
        return Some(det("font/sfnt", desc, "ttf"));
    }
    if d.starts_with(b"OTTO") {
        return Some(det("application/vnd.ms-opentype", "OpenType font data".to_string(), "otf"));
    }
    if d.starts_with(b"ttcf") {
        return Some(det("font/collection", "TrueType font collection data".to_string(), "ttc"));
    }
    // --- documents / archives ---
    if find(d, b"%PDF-", 1024).is_some() {
        let off = find(d, b"%PDF-", 1024).unwrap();
        let mut desc = "PDF document".to_string();
        if n >= off + 8 {
            let maj = d[off + 5] - b'0';
            let min = d[off + 7] - b'0';
            if d[off + 5].is_ascii_digit() && d[off + 7].is_ascii_digit() {
                desc.push_str(&format!(", version {maj}.{min}"));
            }
        }
        if let Some(p) = find(d, b"/Count ", n) {
            let mut pages = String::new();
            for &c in &d[p + 7..n.min(p + 17)] {
                if c.is_ascii_digit() {
                    pages.push(c as char);
                } else {
                    break;
                }
            }
            if !pages.is_empty() {
                desc.push_str(&format!(", {pages} page(s)"));
            }
        }
        return Some(det("application/pdf", desc, "pdf"));
    }
    if d.starts_with(b"PK\x03\x04") {
        return Some(zip_deep(d));
    }
    if d.starts_with(b"PK\x05\x06") {
        return Some(det("application/zip", "Zip archive data (empty)".to_string(), "???"));
    }
    if d.starts_with(b"Cr24") {
        let mut desc = "Google Chrome extension".to_string();
        if n >= 8 {
            desc.push_str(&format!(", version {}", le32(d, 4)));
        }
        return Some(det("application/x-chrome-extension", desc, "crx"));
    }
    if d.starts_with(b"\xde\x12\x04\x95") {
        return Some(Det {
            mime: "application/x-gettext-translation".to_string(),
            desc: mo_desc(d, false),
            ext: "mo",
        });
    }
    if d.starts_with(b"\x95\x04\x12\xde") {
        return Some(Det {
            mime: "application/x-gettext-translation".to_string(),
            desc: mo_desc(d, true),
            ext: "mo",
        });
    }
    if d.starts_with(b"SQLite format 3\x00") {
        return Some(det("application/vnd.sqlite3", "SQLite 3.x database".to_string(), "sqlite/sqlite3/db/dbe"));
    }
    if d.starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1") {
        return Some(det("application/CDFV2", "Composite Document File V2 Document".to_string(), "???"));
    }
    if d.starts_with(b"\x1f\x8b") {
        return Some(det("application/gzip", "gzip compressed data".to_string(), "gz/tgz"));
    }
    if d.starts_with(b"BZh") && n >= 4 && d[3].is_ascii_digit() {
        return Some(det("application/x-bzip2", "bzip2 compressed data".to_string(), "bz2"));
    }
    if d.starts_with(b"\xfd7zXZ\x00") {
        return Some(det("application/x-xz", "XZ compressed data".to_string(), "xz"));
    }
    if d.starts_with(b"\x28\xb5\x2f\xfd") {
        return Some(det("application/zstd", "Zstandard compressed data".to_string(), "zst"));
    }
    if d.starts_with(b"7z\xbc\xaf\x27\x1c") {
        return Some(det("application/x-7z-compressed", "7-zip archive data".to_string(), "7z"));
    }
    if d.starts_with(b"Rar!\x1a\x07") {
        return Some(det("application/x-rar", "RAR archive data".to_string(), "rar"));
    }
    if n > 262 && (&d[257..262] == b"ustar" || &d[257..263] == b"ustar ") {
        return Some(det("application/x-tar", "POSIX tar archive".to_string(), "tar"));
    }
    if d.starts_with(b"\x00asm") {
        return Some(det("application/wasm", "WebAssembly (wasm) binary module".to_string(), "wasm"));
    }
    // --- audio / video ---
    if d.starts_with(b"fLaC") {
        return Some(det("audio/flac", flac_desc(d), "flac"));
    }
    if d.starts_with(b"ID3") {
        return Some(det("audio/mpeg", "Audio file with ID3".to_string(), "mp3"));
    }
    if mpeg_audio_frame(d) {
        let version = match (d[1] >> 3) & 0x03 {
            3 => ", v1",
            2 => ", v2",
            0 => ", v2.5",
            _ => "",
        };
        // Layer III bitrate/samplerate tables (v1 row; v2/v2.5 halved rows).
        let v1 = (d[1] >> 3) & 0x03 == 3;
        let br_v1 = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320, 0];
        let br_v2 = [0, 8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160, 0];
        let kbps = if v1 { br_v1[(d[2] >> 4) as usize] } else { br_v2[(d[2] >> 4) as usize] };
        let sr_v1 = ["44.1", "48", "32", ""];
        let sr_v2 = ["22.05", "24", "16", ""];
        let sr25 = ["11.025", "12", "8", ""];
        let khz = match (d[1] >> 3) & 0x03 {
            3 => sr_v1[((d[2] >> 2) & 3) as usize],
            2 => sr_v2[((d[2] >> 2) & 3) as usize],
            _ => sr25[((d[2] >> 2) & 3) as usize],
        };
        let mode = match d.get(3).copied().unwrap_or(0) >> 6 {
            0 => ", Stereo",
            1 => ", JntStereo",
            2 => ", 2 x Monaural",
            _ => ", Monaural",
        };
        return Some(det(
            "audio/mpeg",
            format!("MPEG ADTS, layer III{version}, {kbps} kbps, {khz} kHz{mode}"),
            "mp3",
        ));
    }
    if d.starts_with(b"OggS") {
        let (mime, desc) = if n >= 35 && &d[28..35] == b"\x01vorbis" {
            ("audio/ogg", "Ogg data, Vorbis audio")
        } else if n >= 36 && &d[28..36] == b"OpusHead" {
            ("audio/ogg", "Ogg data, Opus audio")
        } else if n >= 35 && &d[28..35] == b"\x80theora" {
            ("video/ogg", "Ogg data, Theora video")
        } else {
            ("application/ogg", "Ogg data")
        };
        return Some(det(mime, desc.to_string(), "ogg/oga/ogv/ogx"));
    }
    if d.starts_with(b"\x1aE\xdf\xa3") {
        // EBML: DocType (0x4282) names the flavor.
        if let Some(p) = find(d, b"\x42\x82", 4096) {
            let rest = &d[p + 2..];
            if rest.len() > 5 && rest[1..].starts_with(b"webm") {
                return Some(det("video/webm", "WebM".to_string(), "webm"));
            }
            if rest.len() > 9 && rest[1..].starts_with(b"matroska") {
                return Some(det("video/x-matroska", "Matroska data".to_string(), "mkv"));
            }
        }
        return Some(det("application/octet-stream", "EBML file".to_string(), "???"));
    }
    if d.starts_with(b"\x30\x26\xb2\x75\x8e\x66\xcf\x11") {
        return Some(det("video/x-ms-asf", "Microsoft ASF".to_string(), "asf/wmv/wma"));
    }
    if d.starts_with(b"MThd") {
        return Some(det("audio/midi", "Standard MIDI data".to_string(), "mid/midi"));
    }
    if d.starts_with(b"FORM") && n >= 12 && &d[8..12] == b"AIFF" {
        return Some(det("audio/x-aiff", "IFF data, AIFF audio".to_string(), "aif/aiff"));
    }
    if d.starts_with(b".snd") {
        return Some(det("audio/basic", "Sun/NeXT audio data".to_string(), "au/snd"));
    }
    if d.starts_with(b"#!AMR") {
        return Some(det("audio/amr", "Adaptive Multi-Rate Codec (GSM telephony)".to_string(), "amr"));
    }
    // --- weak magics last ---
    if n >= 528 && d[522] == 0x00 && d[523] == 0x11 && d[524] == 0x02 && d[525] == 0xff {
        // Magdir/pict v2 shape; the QuickTime-decompressor sub-print is not
        // modelled (documented desc-only divergence).
        let mut desc = "Macintosh QuickDraw PICT, version 2".to_string();
        desc.push_str(&format!(", {} x {}", be16(d, 518), be16(d, 520)));
        desc.push_str(&format!(", at 528 {:#06x}", be16(d, 528)));
        if n >= 554 {
            desc.push_str(&format!(", at 552 second opcode {:#06x}", be16(d, 552)));
        }
        return Some(det("image/x-pict", desc, "pct/pict"));
    }
    if n >= 18
        && d[1] <= 1
        && matches!(d[2], 1 | 2 | 3 | 9 | 10 | 11)
        && matches!(d[16], 1 | 8 | 15 | 16 | 24 | 32)
        && (d[1] == 1) == (d[2] == 1 || d[2] == 9)
    {
        let mut desc = "Targa image data".to_string();
        let kind = match d[2] {
            1 | 9 => " - Map",
            2 | 10 => " - RGB",
            _ => " - Mono",
        };
        desc.push_str(kind);
        if d[2] > 8 {
            desc.push_str(" - RLE");
        }
        desc.push_str(&format!(" {} x {}", le16(d, 12), le16(d, 14)));
        desc.push_str(&format!(" x {}", d[16]));
        if d[17] & 0x20 != 0 {
            desc.push_str(" - top");
        }
        if d[17] & 0x10 != 0 {
            desc.push_str(" - right");
        }
        return Some(det("image/x-tga", desc, "tga/tpic"));
    }
    None
}

/// The Magdir `tiff_entry` chain: walk the IFD printing the known tags in
/// entry order; the first tag outside the chain ends the walk. RATIONALs
/// print the raw 32-bit value slot (the offset), exactly like the magic.
fn tiff_ifd_desc(d: &[u8], tiff_base: usize, be: bool) -> String {
    let r16 = |o: usize| -> Option<u32> {
        d.get(o..o + 2).map(|s| if be { ((s[0] as u32) << 8) | s[1] as u32 } else { ((s[1] as u32) << 8) | s[0] as u32 })
    };
    let r32 = |o: usize| -> Option<u32> {
        d.get(o..o + 4).map(|s| {
            if be {
                ((s[0] as u32) << 24) | ((s[1] as u32) << 16) | ((s[2] as u32) << 8) | s[3] as u32
            } else {
                ((s[3] as u32) << 24) | ((s[2] as u32) << 16) | ((s[1] as u32) << 8) | s[0] as u32
            }
        })
    };
    let mut out = String::new();
    let Some(ifd_rel) = r32(tiff_base + 4) else { return out };
    let ifd = tiff_base + ifd_rel as usize;
    let Some(count) = r16(ifd) else { return out };
    out.push_str(&format!(", direntries={count}"));
    let str_at = |off: u32| -> String {
        let p = tiff_base + off as usize;
        let mut end = p;
        while end < d.len() && d[end] != 0 && end - p < 127 {
            end += 1;
        }
        String::from_utf8_lossy(&d[p.min(d.len())..end]).into_owned()
    };
    for i in 0..count.min(64) {
        let e = ifd + 2 + 12 * i as usize;
        let (Some(tag), Some(cnt), Some(val32), Some(val16)) = (r16(e), r32(e + 4), r32(e + 8), r16(e + 8)) else {
            break;
        };
        match tag {
            0x00fe | 0x0111 | 0x013e | 0x013f | 0x0211 | 0x0213 | 0x0214 | 0x8769 => {}
            0x8825 => out.push_str(", GPS-Data"),
            0x0100 => {
                if cnt != 1 {
                    break;
                }
                out.push_str(&format!(", width={val16}"));
            }
            0x0101 => {
                if cnt != 1 {
                    break;
                }
                out.push_str(&format!(", height={val16}"));
            }
            0x0102 => out.push_str(&format!(", bps={val16}")),
            0x0103 => {
                if cnt != 1 {
                    break;
                }
                let name = match val16 {
                    1 => "none".to_string(),
                    2 => "huffman".to_string(),
                    3 => "bi-level group 3".to_string(),
                    4 => "bi-level group 4".to_string(),
                    5 => "LZW".to_string(),
                    6 => "JPEG (old)".to_string(),
                    7 => "JPEG".to_string(),
                    8 => "deflate".to_string(),
                    9 => "JBIG, ITU-T T.85".to_string(),
                    0xa => "JBIG, ITU-T T.43".to_string(),
                    0x7ffe => "NeXT RLE 2-bit".to_string(),
                    0x8005 => "PackBits (Macintosh RLE)".to_string(),
                    0x8029 => "Thunderscan RLE".to_string(),
                    0x807f => "RasterPadding (CT or MP)".to_string(),
                    0x8080 => "RLE (Line Work)".to_string(),
                    0x8081 => "RLE (High-Res Cont-Tone)".to_string(),
                    0x8082 => "RLE (Binary Line Work)".to_string(),
                    0x80b2 => "Deflate (PKZIP)".to_string(),
                    0x80b3 => "Kodak DCS".to_string(),
                    0x8765 => "JBIG".to_string(),
                    0x8798 => "JPEG2000".to_string(),
                    0x8799 => "Nikon NEF Compressed".to_string(),
                    n => format!("(unknown {n:#x})"),
                };
                out.push_str(&format!(", compression={name}"));
            }
            0x0106 => {
                let name = match val16 {
                    0 => "WhiteIsZero".to_string(),
                    1 => "BlackIsZero".to_string(),
                    2 => "RGB".to_string(),
                    3 => "RGB Palette".to_string(),
                    4 => "Transparency Mask".to_string(),
                    5 => "CMYK".to_string(),
                    6 => "YCbCr".to_string(),
                    8 => "CIELab".to_string(),
                    n => format!("(unknown={n:#x})"),
                };
                out.push_str(&format!(", PhotometricInterpretation={name}"));
            }
            0x010a => {
                if cnt != 1 {
                    break;
                }
            }
            0x010d => out.push_str(&format!(", name={}", str_at(val32))),
            0x010e => out.push_str(&format!(", description={}", str_at(val32))),
            0x010f => out.push_str(&format!(", manufacturer={}", str_at(val32))),
            0x0110 => out.push_str(&format!(", model={}", str_at(val32))),
            0x0112 => {
                let name = match val16 {
                    1 => "upper-left".to_string(),
                    3 => "lower-right".to_string(),
                    6 => "upper-right".to_string(),
                    8 => "lower-left".to_string(),
                    9 => "undefined".to_string(),
                    n => format!("[*{n}*]"),
                };
                out.push_str(&format!(", orientation={name}"));
            }
            0x011a => out.push_str(&format!(", xresolution={val32}")),
            0x011b => out.push_str(&format!(", yresolution={val32}")),
            0x0128 => out.push_str(&format!(", resolutionunit={val16}")),
            0x0131 => out.push_str(&format!(", software={}", str_at(val32))),
            0x0132 => out.push_str(&format!(", datetime={}", str_at(val32))),
            0x013c => out.push_str(&format!(", hostcomputer={}", str_at(val32))),
            0x8298 => out.push_str(&format!(", copyright={}", str_at(val32))),
            _ => break,
        }
    }
    out
}

/// JPEG NONE-mode description: the segment walk in file order — JFIF header,
/// Exif APP1 (recursing into the TIFF printer), comments, then the SOF facts.
fn jpeg_desc(d: &[u8]) -> String {
    let mut desc = "JPEG image data".to_string();
    let mut exif_seen = false;
    let mut i = 2usize;
    while i + 4 <= d.len() {
        if d[i] != 0xff {
            break;
        }
        let marker = d[i + 1];
        if (0xd0..=0xd8).contains(&marker) {
            i += 2;
            continue;
        }
        if marker == 0xda {
            break;
        }
        let len = be16(d, i + 2) as usize;
        match marker {
            0xe0 if i == 2 && d.len() >= i + 18 && &d[i + 4..i + 8] == b"JFIF" => {
                desc.push_str(&format!(", JFIF standard {}.{:02}", d[i + 9], d[i + 10]));
                match d[i + 11] {
                    0 => desc.push_str(", aspect ratio"),
                    1 => desc.push_str(", resolution (DPI)"),
                    2 => desc.push_str(", resolution (DPCM)"),
                    _ => {}
                }
                desc.push_str(&format!(", density {}x{}", be16(d, i + 12), be16(d, i + 14)));
                desc.push_str(&format!(", segment length {len}"));
            }
            0xe1 if !exif_seen && d.len() >= i + 14 && &d[i + 4..i + 8] == b"Exif" => {
                exif_seen = true;
                // At offset 6 the entry is ", Exif standard:"; found later
                // (after JFIF or other APPn) it's the capitalised search hit.
                let label = if i == 2 { ", Exif standard: [" } else { ", Exif Standard: [" };
                desc.push_str(label);
                let base = i + 10;
                if d.len() >= base + 8 {
                    let be = &d[base..base + 2] == b"MM";
                    desc.push_str(&format!(
                        "TIFF image data, {}-endian{}",
                        if be { "big" } else { "little" },
                        tiff_ifd_desc(d, base, be)
                    ));
                }
                desc.push(']');
            }
            0xfe if len >= 2 && d.len() > i + 4 => {
                let body = &d[i + 4..d.len().min(i + 2 + len)];
                let end = body.iter().position(|&b| b == 0 || b == b'\n').unwrap_or(body.len());
                desc.push_str(&format!(
                    ", comment: \"{}\"",
                    String::from_utf8_lossy(&body[..end.min(127)])
                ));
            }
            0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
                if i + 9 < d.len() =>
            {
                let kind = match marker {
                    0xc0 => "baseline",
                    0xc2 => "progressive",
                    _ => "non-baseline",
                };
                desc.push_str(&format!(
                    ", {kind}, precision {}, {}x{}, components {}",
                    d[i + 4],
                    be16(d, i + 7),
                    be16(d, i + 5),
                    if i + 9 < d.len() { d[i + 9] } else { 0 }
                ));
                break;
            }
            _ => {}
        }
        if len < 2 {
            break;
        }
        i += 2 + len;
    }
    desc
}

// ---------------------------------------------------------------------------
// TEXTTEST-phase classifiers (run on the utf8 re-encoding of ubuf)
// ---------------------------------------------------------------------------

struct TextHit {
    mime: &'static str,
    /// softmagic stub as libmagic prints it, e.g. "HTML document text" —
    /// ascmagic replaces a " text"/" text executable" tail when composing.
    stub: &'static str,
}

fn ascii_starts_with_ci(hay: &[u8], needle: &[u8]) -> bool {
    hay.len() >= needle.len() && hay[..needle.len()].eq_ignore_ascii_case(needle)
}

fn find_ci(hay: &[u8], needle: &[u8], limit: usize) -> Option<usize> {
    let end = hay.len().min(limit);
    if needle.is_empty() || end < needle.len() {
        return None;
    }
    (0..=end - needle.len()).find(|&i| hay[i..i + needle.len()].eq_ignore_ascii_case(needle))
}

fn shebang_line(t: &[u8]) -> Option<&[u8]> {
    if !t.starts_with(b"#!") {
        return None;
    }
    let end = t.iter().position(|&b| b == b'\n').unwrap_or(t.len());
    Some(&t[..end])
}

fn lines_prefix(t: &[u8], limit: usize) -> impl Iterator<Item = &[u8]> {
    t[..t.len().min(limit)].split(|&b| b == b'\n')
}

fn text_softmagic(t: &[u8]) -> Option<TextHit> {
    // WebVTT: exact offset-0 signature.
    if t.starts_with(b"WEBVTT") {
        return Some(TextHit { mime: "text/vtt", stub: "WebVTT subtitles" });
    }
    // PHP: at offset 0 (search/1 entries), or a php shebang.
    if ascii_starts_with_ci(t, b"<?php") || t.starts_with(b"<?\n") || t.starts_with(b"<?\r") {
        return Some(TextHit { mime: "text/x-php", stub: "PHP script text" });
    }
    if let Some(sb) = shebang_line(t) {
        if find(sb, b"php", sb.len()).is_some() {
            return Some(TextHit { mime: "text/x-php", stub: "PHP script text executable" });
        }
        if find(sb, b"python", sb.len()).is_some() {
            return Some(TextHit { mime: "text/x-script.python", stub: "Python script text executable" });
        }
        if find(sb, b"node", sb.len()).is_some() {
            return Some(TextHit { mime: "application/javascript", stub: "Node.js script text executable" });
        }
        if sb.ends_with(b"/bash") || find(sb, b"env bash", sb.len()).is_some() {
            return Some(TextHit { mime: "text/x-shellscript", stub: "Bourne-Again shell script text executable" });
        }
        if sb.ends_with(b"/zsh") || find(sb, b"env zsh", sb.len()).is_some() {
            return Some(TextHit { mime: "text/x-shellscript", stub: "Paul Falstad's zsh script text executable" });
        }
        if sb.ends_with(b"/sh") || find(sb, b"env sh", sb.len()).is_some() {
            return Some(TextHit { mime: "text/x-shellscript", stub: "POSIX shell script text executable" });
        }
    }
    // XML / SVG: prolog at offset 0 (BOM already stripped from ubuf).
    let tw = {
        let mut s = t;
        while let Some((&c, rest)) = s.split_first() {
            if c == b' ' || c == b'\t' || c == b'\r' || c == b'\n' {
                s = rest;
            } else {
                break;
            }
        }
        s
    };
    if ascii_starts_with_ci(tw, b"<?xml") {
        if find_ci(t, b"<svg", 4096).is_some() {
            return Some(TextHit { mime: "image/svg+xml", stub: "SVG Scalable Vector Graphics image" });
        }
        return Some(TextHit { mime: "text/xml", stub: "XML 1.0 document text" });
    }
    if ascii_starts_with_ci(tw, b"<svg") {
        return Some(TextHit { mime: "image/svg+xml", stub: "SVG Scalable Vector Graphics image" });
    }
    // HTML: the sgml token list, searched in the first 4096 bytes. Each tag
    // needs `>` or a whitespace run right after (the Magdir cWt entries);
    // `<a href=` carries the w flag, so its blank matches any whitespace run.
    fn ws(c: u8) -> bool {
        matches!(c, b' ' | b'\t' | b'\r' | b'\n')
    }
    for tok in
        [b"<!doctype html".as_slice(), b"<html", b"<head", b"<title", b"<script", b"<style", b"<table"]
    {
        let mut from = 0usize;
        while let Some(rel) = find_ci(&t[from..], tok, 4096usize.saturating_sub(from)) {
            let p = from + rel;
            match t.get(p + tok.len()).copied() {
                Some(b'>') => return Some(TextHit { mime: "text/html", stub: "HTML document text" }),
                Some(c) if ws(c) => return Some(TextHit { mime: "text/html", stub: "HTML document text" }),
                _ => from = p + 1,
            }
        }
    }
    {
        let mut from = 0usize;
        while let Some(rel) = find_ci(&t[from..], b"<a", 4096usize.saturating_sub(from)) {
            let mut p = from + rel + 2;
            while t.get(p).copied().is_some_and(ws) {
                p += 1;
            }
            if t.len() >= p + 5 && t[p..p + 5].eq_ignore_ascii_case(b"href=") {
                return Some(TextHit { mime: "text/html", stub: "HTML document text" });
            }
            from = from + rel + 1;
        }
    }
    // gettext .po (Magdir/gnu): `\nmsgid` within the first 1024 bytes, then
    // `\nmsgstr` within 1024 after that match.
    if let Some(p) = find(t, b"\nmsgid", 1024) {
        let rest = &t[p + 6..];
        if find(rest, b"\nmsgstr", 1024 + 7).is_some() {
            return Some(TextHit { mime: "text/x-po", stub: "GNU gettext message catalogue text" });
        }
    }
    // Python heuristics: `from X import` / `def name(...):` at a line start.
    for l in lines_prefix(t, 4096) {
        if l.starts_with(b"from ") && find(l, b" import", l.len()).is_some() {
            return Some(TextHit { mime: "text/x-script.python", stub: "Python script text executable" });
        }
        if let Some(rest) = l.strip_prefix(b"def ") {
            if rest.first().is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_') {
                let l = l.strip_suffix(b"\r").unwrap_or(l);
                if find(l, b"(", l.len()).is_some() && l.ends_with(b"):") {
                    return Some(TextHit { mime: "text/x-script.python", stub: "Python script text executable" });
                }
            }
        }
    }
    // JavaScript: an IIFE or a 'use strict' prologue at a line start.
    for l in lines_prefix(t, 4096) {
        if l.starts_with(b"(function(") {
            return Some(TextHit { mime: "application/javascript", stub: "JavaScript source text" });
        }
        let l = l.strip_suffix(b"\r").unwrap_or(l);
        if l.starts_with(b"'use strict'") || l.starts_with(b"\"use strict\"") {
            return Some(TextHit { mime: "application/javascript", stub: "JavaScript source text" });
        }
    }
    // Any SGML comment or non-html doctype in the head — the weakest
    // entries: desc-only, the mime stays text/plain.
    if find(t, b"<!--", 4096).is_some() || find_ci(t, b"<!doctype", 4096).is_some() {
        return Some(TextHit { mime: "text/plain", stub: "exported SGML document text" });
    }
    None
}

/// RTF runs in the binary phase (its match never gets the encoding suffix).
fn rtf_det(d: &[u8]) -> Option<Det> {
    if !d.starts_with(b"{\\rtf") {
        return None;
    }
    let mut desc = "Rich Text Format data".to_string();
    if d.len() > 5 && d[5].is_ascii_digit() {
        desc.push_str(&format!(", version {}", d[5] as char));
    }
    if find(d, b"\\ansicpg", 1024).is_some() {
        let p = find(d, b"\\ansicpg", 1024).unwrap() + 8;
        let mut cp = String::new();
        for &c in &d[p..d.len().min(p + 8)] {
            if c.is_ascii_digit() {
                cp.push(c as char);
            } else {
                break;
            }
        }
        desc.push_str(", ANSI");
        if !cp.is_empty() {
            desc.push_str(&format!(", code page {cp}"));
        }
    } else if find(d, b"\\ansi", 1024).is_some() {
        desc.push_str(", ANSI");
    } else if find(d, b"\\mac", 1024).is_some() {
        desc.push_str(", Apple Macintosh");
    } else {
        desc.push_str(", unknown character set");
    }
    Some(det("text/rtf", desc, "rtf"))
}

// ---------------------------------------------------------------------------
// ascmagic — text description composition
// ---------------------------------------------------------------------------

fn encode_utf8(ubuf: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ubuf.len());
    for &c in ubuf {
        if c <= 0x7f {
            out.push(c as u8);
        } else if c <= 0x7ff {
            out.push(((c >> 6) + 0xc0) as u8);
            out.push(((c & 0x3f) + 0x80) as u8);
        } else if c <= 0xffff {
            out.push(((c >> 12) + 0xe0) as u8);
            out.push((((c >> 6) & 0x3f) + 0x80) as u8);
            out.push(((c & 0x3f) + 0x80) as u8);
        } else {
            out.push(((c >> 18) + 0xf0) as u8);
            out.push((((c >> 12) & 0x3f) + 0x80) as u8);
            out.push((((c >> 6) & 0x3f) + 0x80) as u8);
            out.push(((c & 0x3f) + 0x80) as u8);
        }
    }
    out
}

fn trim_nuls(d: &[u8]) -> &[u8] {
    let mut n = d.len();
    while n > 1 && d[n - 1] == 0 {
        n -= 1;
    }
    // Keep UTF-16 tails intact: don't trim to odd if the input was even.
    if n % 2 == 1 && d.len() % 2 == 0 {
        n += 1;
    }
    &d[..n]
}

/// The line-facts suffix ascmagic appends: very long lines, terminators,
/// escapes, overstriking.
fn text_details(ubuf: &[u32]) -> String {
    let mut out = String::new();
    let (mut n_crlf, mut n_lf, mut n_cr, mut n_nel) = (0usize, 0usize, 0usize, 0usize);
    let (mut has_escapes, mut has_backspace) = (false, false);
    let mut seen_cr = false;
    let mut last_line_end: i64 = -1;
    let mut has_long_lines: i64 = 0;
    for (i, &c) in ubuf.iter().enumerate() {
        let i = i as i64;
        if c == 0x0a {
            if seen_cr {
                n_crlf += 1;
            } else {
                n_lf += 1;
            }
            last_line_end = i;
        } else if seen_cr {
            n_cr += 1;
        }
        seen_cr = c == 0x0d;
        if seen_cr {
            last_line_end = i;
        }
        if c == 0x85 {
            n_nel += 1;
            last_line_end = i;
        }
        if i > last_line_end + MAXLINELEN {
            let ll = i - last_line_end;
            if ll > has_long_lines {
                has_long_lines = ll;
            }
        }
        if c == 0x1b {
            has_escapes = true;
        }
        if c == 0x08 {
            has_backspace = true;
        }
    }
    if has_long_lines > 0 {
        out.push_str(&format!(", with very long lines ({has_long_lines})"));
    }
    let none = n_crlf == 0 && n_cr == 0 && n_nel == 0 && n_lf == 0;
    if none || n_crlf != 0 || n_cr != 0 || n_nel != 0 {
        out.push_str(", with");
        if none {
            out.push_str(" no");
        } else {
            if n_crlf != 0 {
                out.push_str(" CRLF");
                if n_cr != 0 || n_lf != 0 || n_nel != 0 {
                    out.push(',');
                }
            }
            if n_cr != 0 {
                out.push_str(" CR");
                if n_lf != 0 || n_nel != 0 {
                    out.push(',');
                }
            }
            if n_lf != 0 {
                out.push_str(" LF");
                if n_nel != 0 {
                    out.push(',');
                }
            }
            if n_nel != 0 {
                out.push_str(" NEL");
            }
        }
        out.push_str(" line terminators");
    }
    if has_escapes {
        out.push_str(", with escape sequences");
    }
    if has_backspace {
        out.push_str(", with overstriking");
    }
    out
}

// ---------------------------------------------------------------------------
// master pipeline (funcs.c file_buffer order)
// ---------------------------------------------------------------------------

struct Full {
    mime: String,
    desc: Vec<u8>,
    charset: &'static str,
    ext: &'static str,
}

fn detect(data: &[u8]) -> Full {
    let n = data.len();
    if n == 0 {
        return Full {
            mime: "application/x-empty".into(),
            desc: b"empty".to_vec(),
            charset: "binary",
            ext: "???",
        };
    }
    if n == 1 {
        return Full {
            mime: "application/octet-stream".into(),
            desc: b"very short file (no magic)".to_vec(),
            charset: "binary",
            ext: "???",
        };
    }
    // Encoding on the untrimmed head — this is the charset that gets printed.
    let enc = file_encoding(data);
    let charset = enc.mime;

    // JSON, then CSV (both before any softmagic, as in file_buffer).
    match is_json(data) {
        1 => {
            return Full {
                mime: "application/json".into(),
                desc: b"JSON text data".to_vec(),
                charset,
                ext: "json",
            }
        }
        2 => {
            return Full {
                mime: "application/x-ndjson".into(),
                desc: b"New Line Delimited JSON text data".to_vec(),
                charset,
                ext: "ndjson/jsonl",
            }
        }
        _ => {}
    }
    if enc.text && is_csv(data) {
        return Full {
            mime: "text/csv".into(),
            desc: format!("CSV {} text", enc.code).into_bytes(),
            charset,
            ext: "csv",
        };
    }

    // Binary-phase softmagic (incl. RTF, which never gets the text suffix).
    // A mime-less entry keeps its description but lets the mime fall through
    // to the text pipeline, like a magic entry without a `!:mime` line.
    let mut desc_override: Option<Vec<u8>> = None;
    if let Some(x) = rtf_det(data) {
        return Full { mime: x.mime, desc: x.desc, charset, ext: x.ext };
    }
    if let Some(x) = softmagic_bin(data) {
        if !x.mime.is_empty() {
            return Full { mime: x.mime, desc: x.desc, charset, ext: x.ext };
        }
        desc_override = Some(x.desc);
    }

    // ascmagic: re-run encoding on the NUL-trimmed buffer, then the text
    // classifiers on the utf8 re-encoding of the decoded characters.
    let trimmed = trim_nuls(data);
    if trimmed.len() > 1 {
        let enc2 = file_encoding(trimmed);
        if enc2.text {
            let utf8 = encode_utf8(&enc2.ubuf);
            let hit = text_softmagic(&utf8);
            let mime = hit.as_ref().map_or("text/plain", |h| h.mime).to_string();
            if let Some(desc) = desc_override {
                return Full { mime, desc, charset, ext: "???" };
            }
            let ext = "???";
            // Compose the NONE description: stub (± " text"/" text executable"
            // tail folded into ", ") + encoding name + "text" + line facts.
            let mut desc = String::new();
            let mut executable = false;
            if let Some(h) = &hit {
                if let Some(pre) = h.stub.strip_suffix(" text executable") {
                    desc.push_str(pre);
                    desc.push_str(", ");
                    executable = true;
                } else if let Some(pre) = h.stub.strip_suffix(" text") {
                    desc.push_str(pre);
                    desc.push_str(", ");
                } else {
                    desc.push_str(h.stub);
                    desc.push_str(", ");
                }
            }
            desc.push_str(enc2.code);
            desc.push_str(" text");
            if executable {
                desc.push_str(" executable");
            }
            desc.push_str(&text_details(&enc2.ubuf));
            return Full { mime, desc: desc.into_bytes(), charset, ext };
        }
    }

    Full {
        mime: "application/octet-stream".into(),
        desc: desc_override.unwrap_or_else(|| b"data".to_vec()),
        charset,
        ext: "???",
    }
}

// ---------------------------------------------------------------------------
// builtin
// ---------------------------------------------------------------------------

/// `__finfo_detect(string $data, int $flags): string` — the prelude's finfo
/// core. Directories, missing files and all stream I/O are handled PHP-side.
pub fn finfo_detect(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = convert::to_zstr(
        args.first()
            .ok_or_else(|| PhpError::ArgumentCountError("__finfo_detect() expects 2 arguments".into()))?,
        ctx.diags,
    );
    let flags = args
        .get(1)
        .map(|v| match v.deref_clone() {
            Zval::Long(n) => n,
            Zval::Double(d) => d as i64,
            _ => 0,
        })
        .unwrap_or(0);
    let full = detect(data.as_bytes());
    let out: Vec<u8> = if flags & F_EXTENSION != 0 {
        full.ext.as_bytes().to_vec()
    } else if flags & F_MIME_TYPE != 0 {
        if flags & F_MIME_ENCODING != 0 {
            format!("{}; charset={}", full.mime, full.charset).into_bytes()
        } else {
            full.mime.into_bytes()
        }
    } else if flags & F_MIME_ENCODING != 0 {
        full.charset.as_bytes().to_vec()
    } else {
        full.desc
    };
    Ok(Zval::Str(PhpStr::new(out)))
}
