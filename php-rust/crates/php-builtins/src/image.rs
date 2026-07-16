//! Image introspection: `getimagesize`, `getimagesizefromstring`,
//! `image_type_to_mime_type`, `image_type_to_extension` (ext/standard/image.c).
//!
//! The header parsers are ported from `php_handle_*`, reading the whole image
//! into a byte buffer and using absolute offsets (verified byte-identical to the
//! oracle). Covered: GIF, JPEG, PNG, BMP, WebP, AVIF, TIFF (II/MM), JP2/JPC,
//! PSD and ICO — every format WP's `file_is_valid_image` corpus exercises;
//! the exotic rest (SWF, IFF, WBMP, XBM…) reports unrecognised
//! (`getimagesize` returns `false`). The optional `&$image_info` out-parameter
//! carries the JPEG APPn segments (IPTC lives in APP13).

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};
use std::rc::Rc;

// IMAGETYPE_* constants (also mirrored in lower/mod.rs::resolve_constant).
const T_GIF: i64 = 1;
const T_JPEG: i64 = 2;
const T_PNG: i64 = 3;
const T_PSD: i64 = 5;
const T_BMP: i64 = 6;
const T_TIFF_II: i64 = 7;
const T_TIFF_MM: i64 = 8;
const T_JPC: i64 = 9;
const T_JP2: i64 = 10;
const T_ICO: i64 = 17;
const T_WEBP: i64 = 18;

/// The extracted geometry of a recognised image.
struct Gfx {
    width: u32,
    height: u32,
    itype: i64,
    bits: u32,
    channels: u32,
}

fn be32(b: &[u8], i: usize) -> u32 {
    ((b[i] as u32) << 24) | ((b[i + 1] as u32) << 16) | ((b[i + 2] as u32) << 8) | b[i + 3] as u32
}
fn le32(b: &[u8], i: usize) -> u32 {
    (b[i] as u32) | ((b[i + 1] as u32) << 8) | ((b[i + 2] as u32) << 16) | ((b[i + 3] as u32) << 24)
}
fn le16(b: &[u8], i: usize) -> u32 {
    (b[i] as u32) | ((b[i + 1] as u32) << 8)
}
fn be16(b: &[u8], i: usize) -> u32 {
    ((b[i] as u32) << 8) | b[i + 1] as u32
}

fn parse_gif(d: &[u8]) -> Option<Gfx> {
    if d.len() < 11 {
        return None;
    }
    let packed = d[10];
    Some(Gfx {
        width: le16(d, 6),
        height: le16(d, 8),
        itype: T_GIF,
        bits: if packed & 0x80 != 0 { (packed & 0x07) as u32 + 1 } else { 0 },
        channels: 3,
    })
}

fn parse_png(d: &[u8]) -> Option<Gfx> {
    if d.len() < 25 {
        return None;
    }
    Some(Gfx {
        width: be32(d, 16),
        height: be32(d, 20),
        itype: T_PNG,
        bits: d[24] as u32,
        channels: 0,
    })
}

fn parse_bmp(d: &[u8]) -> Option<Gfx> {
    if d.len() < 30 {
        return None;
    }
    let size = le32(d, 14);
    let (width, height, bits) = if size == 12 {
        (le16(d, 18), le16(d, 20), le16(d, 24))
    } else if size > 12 && (size <= 64 || size == 108 || size == 124) {
        let h = le32(d, 22) as i32;
        (le32(d, 18), h.unsigned_abs(), le16(d, 28))
    } else {
        return None;
    };
    Some(Gfx { width, height, itype: T_BMP, bits, channels: 0 })
}

/// WebP: after the 12-byte `RIFF????WEBP` prefix comes an 18-byte chunk header
/// (`VP8 ` / `VP8L` / `VP8X`) carrying the dimensions.
fn parse_webp(d: &[u8]) -> Option<Gfx> {
    if d.len() < 30 {
        return None;
    }
    let b = &d[12..30]; // the handler's 18-byte buffer
    if &b[0..3] != b"VP8" {
        return None;
    }
    let (width, height) = match b[3] {
        b' ' => (
            b[14] as u32 + (((b[15] & 0x3F) as u32) << 8),
            b[16] as u32 + (((b[17] & 0x3F) as u32) << 8),
        ),
        b'L' => (
            b[9] as u32 + (((b[10] & 0x3F) as u32) << 8) + 1,
            (b[10] >> 6) as u32 + ((b[11] as u32) << 2) + (((b[12] & 0x0F) as u32) << 10) + 1,
        ),
        b'X' => (
            b[12] as u32 + ((b[13] as u32) << 8) + ((b[14] as u32) << 16) + 1,
            b[15] as u32 + ((b[16] as u32) << 8) + ((b[17] as u32) << 16) + 1,
        ),
        _ => return None,
    };
    Some(Gfx { width, height, itype: T_WEBP, bits: 8, channels: 0 })
}

/// JPEG: scan segment markers from just past the SOI for a SOFn frame header
/// (`precision`, `height`, `width`, `component count`).
fn parse_jpeg(d: &[u8]) -> Option<Gfx> {
    let mut i = 2; // past FF D8
    let is_sof = |m: u8| {
        matches!(
            m,
            0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE | 0xCF
        )
    };
    while i < d.len() {
        // Advance to a marker: skip to the next 0xFF, then past any run of 0xFF.
        while i < d.len() && d[i] != 0xFF {
            i += 1;
        }
        while i < d.len() && d[i] == 0xFF {
            i += 1;
        }
        if i >= d.len() {
            break;
        }
        let marker = d[i];
        i += 1;
        if is_sof(marker) {
            if i + 8 > d.len() {
                break;
            }
            return Some(Gfx {
                width: be16(d, i + 5),
                height: be16(d, i + 3),
                itype: T_JPEG,
                bits: d[i + 2] as u32,
                channels: d[i + 7] as u32,
            });
        }
        match marker {
            0xD9 | 0xDA => break,            // EOI / SOS — no frame header before data
            0x01 | 0xD0..=0xD7 => continue,  // standalone markers (no length)
            _ => {
                if i + 2 > d.len() {
                    break;
                }
                let length = be16(d, i) as usize; // includes the 2 length bytes
                if length < 2 {
                    break;
                }
                i += length;
            }
        }
    }
    None
}

/// AVIF (IMAGETYPE_AVIF = 19): dimensions/bits/channels via the minimal
/// libavifinfo work-alike in exif.rs.
fn parse_avif(d: &[u8]) -> Option<Gfx> {
    let info = crate::exif::parse_avif_info(d)?;
    Some(Gfx {
        width: info.width,
        height: info.height,
        itype: 19,
        bits: info.bits,
        channels: info.channels,
    })
}

/// `php_handle_tiff`: walk IFD0's inline-valued entries for ImageWidth /
/// ImageLength (plain 0x0100/0x0101 or "compressed" 0xA002/0xA003). Zend
/// reports no bits/channels for TIFF.
fn parse_tiff_size(d: &[u8], motorola: bool) -> Option<Gfx> {
    let g16 = if motorola { be16 } else { le16 };
    let g32 = if motorola { be32 } else { le32 };
    if d.len() < 8 {
        return None;
    }
    let ifd = g32(d, 4) as usize;
    if ifd + 2 > d.len() {
        return None;
    }
    let n = g16(d, ifd) as usize;
    if ifd + 2 + n * 12 + 4 > d.len() {
        return None; // C reads the whole directory at once; short read → NULL
    }
    let (mut w, mut h) = (0u32, 0u32);
    for i in 0..n {
        let e = ifd + 2 + i * 12;
        let tag = g16(d, e);
        // Inline value only, by TAG_FMT: BYTE/SBYTE, USHORT/SSHORT, ULONG/SLONG.
        let val = match g16(d, e + 2) {
            1 | 6 => d[e + 8] as u32,
            3 | 8 => g16(d, e + 8),
            4 | 9 => g32(d, e + 8),
            _ => continue,
        };
        match tag {
            0x0100 | 0xA002 => w = val,
            0x0101 | 0xA003 => h = val,
            _ => {}
        }
    }
    (w != 0 && h != 0).then_some(Gfx {
        width: w,
        height: h,
        itype: if motorola { T_TIFF_MM } else { T_TIFF_II },
        bits: 0,
        channels: 0,
    })
}

/// `php_handle_psd`: height/width are the big-endian longs at header offsets
/// 14/18 (after the `8BPS` signature + version + reserved + channel count).
fn parse_psd(d: &[u8]) -> Option<Gfx> {
    if d.len() < 22 {
        return None;
    }
    Some(Gfx { width: be32(d, 18), height: be32(d, 14), itype: T_PSD, bits: 0, channels: 0 })
}

/// `php_handle_jpc`: the SIZ segment right after SOC. `pos` is the offset of
/// the SIZ marker's low byte (0x51). Width/height from Xsiz/Ysiz, channels
/// from Csiz, bits = the highest per-component depth (Ssiz[i] + 1).
fn parse_jpc(d: &[u8], pos: usize, itype: i64) -> Option<Gfx> {
    if d.get(pos) != Some(&0x51) {
        return None; // corrupt codestream: SIZ must follow SOC
    }
    let p = pos + 1 + 4; // skip Lsiz + Rsiz
    if p + 8 > d.len() {
        return None;
    }
    let (w, h) = (be32(d, p), be32(d, p + 4));
    let p = p + 8 + 24; // skip XOsiz..YTOsiz
    if p + 2 > d.len() {
        return None;
    }
    let ch = be16(d, p) as usize;
    if ch == 0 || ch > 256 {
        return None;
    }
    let p = p + 2;
    let mut bits = 0u32;
    for i in 0..ch {
        bits = bits.max(*d.get(p + i * 3)? as u32 + 1);
    }
    Some(Gfx { width: w, height: h, itype, bits, channels: ch as u32 })
}

/// `php_handle_jp2`: scan the root-level boxes for the first `jp2c`
/// codestream and hand its payload (skipping the 3 SOC bytes, as the C does
/// to "emulate the file type examination") to the JPC parser.
fn parse_jp2(d: &[u8]) -> Option<Gfx> {
    let mut p = 12usize; // after the 12-byte JP2 signature box
    loop {
        if p + 8 > d.len() {
            return None;
        }
        let lbox = be32(d, p);
        if lbox == 1 {
            return None; // XLBoxes unhandled, like the C
        }
        if &d[p + 4..p + 8] == b"jp2c" {
            return parse_jpc(d, p + 8 + 3, T_JP2);
        }
        if lbox as i32 <= 0 {
            return None; // last box, no codestream found
        }
        p += lbox as usize;
    }
}

/// `php_handle_ico`: scan the ICONDIR entries, keeping the last one whose bit
/// count is >= the best so far; a stored 0 dimension means 256.
fn parse_ico(d: &[u8]) -> Option<Gfx> {
    if d.len() < 6 {
        return None;
    }
    let n = le16(d, 4);
    if !(1..=255).contains(&n) {
        return None;
    }
    let (mut w, mut h, mut bits) = (0u32, 0u32, 0u32);
    for i in 0..n as usize {
        let e = 6 + i * 16;
        if e + 16 > d.len() {
            break;
        }
        let bc = le16(d, e + 6);
        if bc >= bits {
            w = d[e] as u32;
            h = d[e + 1] as u32;
            bits = bc;
        }
    }
    Some(Gfx {
        width: if w == 0 { 256 } else { w },
        height: if h == 0 { 256 } else { h },
        itype: T_ICO,
        bits,
        channels: 0,
    })
}

/// Recognise an image from its leading bytes and extract its geometry. Only the
/// common formats are parsed; everything else is `None`.
fn parse_image(d: &[u8]) -> Option<Gfx> {
    if d.len() >= 3 && &d[0..3] == b"GIF" {
        parse_gif(d)
    } else if d.len() >= 3 && d[0] == 0xFF && d[1] == 0xD8 && d[2] == 0xFF {
        parse_jpeg(d)
    } else if d.len() >= 8 && &d[0..8] == b"\x89PNG\r\n\x1a\n" {
        parse_png(d)
    } else if d.len() >= 2 && &d[0..2] == b"BM" {
        parse_bmp(d)
    } else if d.len() >= 12 && &d[0..4] == b"RIFF" && &d[8..12] == b"WEBP" {
        parse_webp(d)
    } else if d.len() >= 12 && &d[4..8] == b"ftyp" {
        parse_avif(d)
    } else if d.len() >= 4 && (&d[0..4] == b"II\x2A\x00" || &d[0..4] == b"MM\x00\x2A") {
        parse_tiff_size(d, d[0] == b'M')
    } else if d.len() >= 4 && &d[0..4] == b"8BPS" {
        parse_psd(d)
    } else if d.len() >= 12 && &d[0..12] == b"\x00\x00\x00\x0CjP  \r\n\x87\n" {
        parse_jp2(d)
    } else if d.len() >= 4 && &d[0..3] == b"\xFF\x4F\xFF" {
        parse_jpc(d, 3, T_JPC)
    } else if d.len() >= 4 && &d[0..4] == b"\x00\x00\x01\x00" {
        parse_ico(d)
    } else {
        None
    }
}

/// Collect the JPEG APPn segments into the `&$image_info` out-array, keyed
/// `"APP0"`…`"APP15"`, first tag of each kind only (php_read_APP).
fn collect_app_info(d: &[u8]) -> PhpArray {
    let mut info = PhpArray::new();
    if !(d.len() >= 3 && d[0] == 0xFF && d[1] == 0xD8 && d[2] == 0xFF) {
        return info;
    }
    let mut i = 2usize;
    while i < d.len() {
        while i < d.len() && d[i] != 0xFF {
            i += 1;
        }
        while i < d.len() && d[i] == 0xFF {
            i += 1;
        }
        if i >= d.len() {
            break;
        }
        let marker = d[i];
        i += 1;
        match marker {
            0xD9 | 0xDA => break,
            0x01 | 0xD0..=0xD7 => continue,
            _ => {}
        }
        if i + 2 > d.len() {
            break;
        }
        let length = be16(d, i) as usize;
        if length < 2 || i + length > d.len() {
            break;
        }
        if (0xE0..=0xEF).contains(&marker) {
            let key = format!("APP{}", marker - 0xE0);
            let k = Key::from_bytes(key.as_bytes());
            if info.get(&k).is_none() {
                info.insert(k, Zval::Str(PhpStr::new(d[i + 2..i + length].to_vec())));
            }
        }
        i += length;
    }
    info
}

/// The `image/…` MIME type for an IMAGETYPE_* value (shared with exif.rs).
pub(crate) fn mime_for_type(itype: i64) -> &'static [u8] {
    mime_for(itype)
}

/// The `image/…` MIME type for an IMAGETYPE_* value; unknown → the generic
/// `application/octet-stream` (`php_image_type_to_mime_type`).
fn mime_for(itype: i64) -> &'static [u8] {
    match itype {
        1 => b"image/gif",
        2 => b"image/jpeg",
        3 => b"image/png",
        4 | 13 => b"application/x-shockwave-flash",
        5 => b"image/psd",
        6 => b"image/bmp",
        7 | 8 => b"image/tiff",
        14 => b"image/iff",
        15 => b"image/vnd.wap.wbmp",
        9 => b"application/octet-stream",
        10 => b"image/jp2",
        16 => b"image/xbm",
        17 => b"image/vnd.microsoft.icon",
        18 => b"image/webp",
        19 => b"image/avif",
        20 => b"image/heif",
        _ => b"application/octet-stream",
    }
}

/// The canonical filename extension (with leading dot) for an IMAGETYPE_*, or
/// `None` if unknown (`image_type_to_extension`).
fn ext_for(itype: i64) -> Option<&'static [u8]> {
    Some(match itype {
        1 => b".gif".as_slice(),
        2 => b".jpeg",
        3 => b".png",
        4 | 13 => b".swf",
        5 => b".psd",
        6 | 15 => b".bmp",
        7 | 8 => b".tiff",
        14 => b".iff",
        9 => b".jpc",
        10 => b".jp2",
        11 => b".jpx",
        12 => b".jb2",
        16 => b".xbm",
        17 => b".ico",
        18 => b".webp",
        19 => b".avif",
        20 => b".heif",
        _ => return None,
    })
}

/// Assemble the `getimagesize` return array from parsed geometry (PHP 8.5 adds
/// `width_unit`/`height_unit`; `bits`/`channels` appear only when non-zero).
fn size_array(g: &Gfx) -> Zval {
    let mut a = PhpArray::new();
    let _ = a.append(Zval::Long(g.width as i64));
    let _ = a.append(Zval::Long(g.height as i64));
    let _ = a.append(Zval::Long(g.itype));
    a.insert(
        Key::Int(3),
        Zval::Str(PhpStr::new(
            format!("width=\"{}\" height=\"{}\"", g.width, g.height).into_bytes(),
        )),
    );
    if g.bits != 0 {
        a.insert(Key::from_bytes(b"bits"), Zval::Long(g.bits as i64));
    }
    if g.channels != 0 {
        a.insert(Key::from_bytes(b"channels"), Zval::Long(g.channels as i64));
    }
    a.insert(Key::from_bytes(b"mime"), Zval::Str(PhpStr::new(mime_for(g.itype).to_vec())));
    a.insert(Key::from_bytes(b"width_unit"), Zval::Str(PhpStr::new(b"px".to_vec())));
    a.insert(Key::from_bytes(b"height_unit"), Zval::Str(PhpStr::new(b"px".to_vec())));
    Zval::Array(Rc::new(a))
}

/// `getimagesize(string $filename, &$image_info = null): array|false`. Reads the
/// file (missing → the "Failed to open stream" Warning + `false`) and parses its
/// header; an unrecognised image is `false`.
pub fn getimagesize(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let name = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::ArgumentCountError(
                "getimagesize() expects at least 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    let Some(data) = crate::file::read_for_builtin(name.as_bytes(), "getimagesize", ctx) else {
        return Ok(Zval::Bool(false));
    };
    if data.len() < 12 {
        // php_getimagetype could not read its 12 signature bytes (E_NOTICE).
        ctx.diags.push(php_types::Diag::Notice(format!(
            "getimagesize(): Error reading from {}!",
            String::from_utf8_lossy(name.as_bytes())
        )));
        return Ok(Zval::Bool(false));
    }
    Ok(match parse_image(&data) {
        Some(g) => size_array(&g),
        None => Zval::Bool(false),
    })
}

/// `getimagesizefromstring(string $string, &$image_info = null): array|false`.
pub fn getimagesizefromstring(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::ArgumentCountError(
                "getimagesizefromstring() expects at least 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    if s.as_bytes().len() < 12 {
        ctx.diags.push(php_types::Diag::Notice(format!(
            "getimagesizefromstring(): Error reading from {}!",
            String::from_utf8_lossy(s.as_bytes())
        )));
        return Ok(Zval::Bool(false));
    }
    Ok(match parse_image(s.as_bytes()) {
        Some(g) => size_array(&g),
        None => Zval::Bool(false),
    })
}

/// `__getimagesize_info($filename)` → `[result, info]`: the registry-visible
/// pair form the VM's CallHostBuiltinOut dispatch splits for
/// `getimagesize($f, &$image_info)`. The info array is always (re)built —
/// PHP re-initialises the out zval even on failure.
pub fn getimagesize_info(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let name = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let mut pair = PhpArray::new();
    match crate::file::read_for_builtin(name.as_bytes(), "getimagesize", ctx) {
        Some(data) => {
            if data.len() < 12 {
                ctx.diags.push(php_types::Diag::Notice(format!(
                    "getimagesize(): Error reading from {}!",
                    String::from_utf8_lossy(name.as_bytes())
                )));
                let _ = pair.append(Zval::Bool(false));
                let _ = pair.append(Zval::Array(Rc::new(PhpArray::new())));
                return Ok(Zval::Array(Rc::new(pair)));
            }
            let _ = pair.append(match parse_image(&data) {
                Some(g) => size_array(&g),
                None => Zval::Bool(false),
            });
            let _ = pair.append(Zval::Array(Rc::new(collect_app_info(&data))));
        }
        None => {
            let _ = pair.append(Zval::Bool(false));
            let _ = pair.append(Zval::Array(Rc::new(PhpArray::new())));
        }
    }
    Ok(Zval::Array(Rc::new(pair)))
}

/// `__getimagesizefromstring_info($data)` → `[result, info]`.
pub fn getimagesizefromstring_info(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let mut pair = PhpArray::new();
    if s.as_bytes().len() < 12 {
        ctx.diags.push(php_types::Diag::Notice(format!(
            "getimagesizefromstring(): Error reading from {}!",
            String::from_utf8_lossy(s.as_bytes())
        )));
        let _ = pair.append(Zval::Bool(false));
        let _ = pair.append(Zval::Array(Rc::new(PhpArray::new())));
        return Ok(Zval::Array(Rc::new(pair)));
    }
    let _ = pair.append(match parse_image(s.as_bytes()) {
        Some(g) => size_array(&g),
        None => Zval::Bool(false),
    });
    let _ = pair.append(Zval::Array(Rc::new(collect_app_info(s.as_bytes()))));
    Ok(Zval::Array(Rc::new(pair)))
}

/// `image_type_to_mime_type(int $image_type): string`.
pub fn image_type_to_mime_type(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let t = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    Ok(Zval::Str(PhpStr::new(mime_for(t).to_vec())))
}

/// `image_type_to_extension(int $image_type, bool $include_dot = true): string|false`.
pub fn image_type_to_extension(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let t = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let include_dot = args.get(1).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(true);
    match ext_for(t) {
        Some(ext) => {
            let bytes = if include_dot { ext } else { &ext[1..] };
            Ok(Zval::Str(PhpStr::new(bytes.to_vec())))
        }
        None => Ok(Zval::Bool(false)),
    }
}
