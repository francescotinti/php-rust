//! Hashing and encoding builtins (step 62): base64_encode/decode, md5, sha1,
//! crc32, hash.
//!
//! PHP strings are bytes: every input and binary output here is `[u8]`, never
//! UTF-8. The digest crates are the RustCrypto family (`md-5`, `sha1`, `sha2`),
//! `crc32fast` for the zlib/IEEE CRC; base64 is hand-rolled to match PHP's
//! `php_base64_encode`/`decode` byte-for-byte (lenient decode that skips
//! non-alphabet bytes unless `$strict`).

use md5::Md5;
use sha1::Sha1;
use sha2::{Sha224, Sha256, Sha384, Sha512, Sha512_224, Sha512_256};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512};
use sha1::Digest; // the `Digest` trait is re-exported by every RustCrypto crate

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

// ---------------------------------------------------------------------------
// hex helper
// ---------------------------------------------------------------------------

fn to_hex(bytes: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
    out
}

/// Render a raw digest either as a lowercase hex string or as raw bytes,
/// matching PHP's `$binary` flag.
fn render_digest(raw: &[u8], binary: bool) -> Zval {
    if binary {
        Zval::Str(PhpStr::new(raw.to_vec()))
    } else {
        Zval::Str(PhpStr::new(to_hex(raw)))
    }
}

fn binary_flag(args: &[Zval], idx: usize) -> bool {
    args.get(idx).map(convert::is_true_silent).unwrap_or(false)
}

// ---------------------------------------------------------------------------
// base64
// ---------------------------------------------------------------------------

const B64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `base64_encode(string $string): string` — standard alphabet, `=` padding.
pub fn base64_encode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("base64_encode() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let data = s.as_bytes();
    let mut out = Vec::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        out.push(B64_ALPHABET[(b0 >> 2) as usize]);
        match chunk.len() {
            1 => {
                out.push(B64_ALPHABET[((b0 & 0x03) << 4) as usize]);
                out.push(b'=');
                out.push(b'=');
            }
            2 => {
                let b1 = chunk[1];
                out.push(B64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);
                out.push(B64_ALPHABET[((b1 & 0x0f) << 2) as usize]);
                out.push(b'=');
            }
            _ => {
                let b1 = chunk[1];
                let b2 = chunk[2];
                out.push(B64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize]);
                out.push(B64_ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize]);
                out.push(B64_ALPHABET[(b2 & 0x3f) as usize]);
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Reverse-map a base64 byte exactly like PHP's `base64_reverse_table`:
/// `-1` = whitespace (`\t \n \r` and space only — NOT `\v`/`\f`), `-2` = any
/// other non-alphabet byte, `0..=63` = the 6-bit value.
fn b64_reverse(c: u8) -> i16 {
    match c {
        b'A'..=b'Z' => (c - b'A') as i16,
        b'a'..=b'z' => (c - b'a') as i16 + 26,
        b'0'..=b'9' => (c - b'0') as i16 + 52,
        b'+' => 62,
        b'/' => 63,
        b'\t' | b'\n' | b'\r' | b' ' => -1,
        _ => -2,
    }
}

/// `base64_decode(string $string, bool $strict = false): string|false`.
///
/// Faithful port of `php_base64_decode_impl` (ext/standard/base64.c): lenient
/// mode skips every non-alphabet byte (including invalid ones); strict mode skips
/// only whitespace, fails on an invalid byte, on any data after `=`, on a single
/// leftover char in the final group, and on malformed padding length.
pub fn base64_decode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("base64_decode() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let strict = binary_flag(args, 1);
    let data = s.as_bytes();

    let mut out: Vec<u8> = vec![0; data.len() + 1]; // PHP allocates input length
    let mut j = 0usize; // committed output bytes
    let mut i = 0usize; // count of consumed alphabet chars
    let mut padding = 0usize;

    for &c in data {
        if c == b'=' {
            padding += 1;
            continue;
        }
        let ch = b64_reverse(c);
        if strict {
            if ch == -1 {
                continue; // whitespace
            }
            if ch == -2 || padding > 0 {
                return Ok(Zval::Bool(false));
            }
        } else if ch < 0 {
            continue; // skip whitespace and invalid bytes
        }
        let ch = ch as u8;
        match i % 4 {
            0 => out[j] = ch << 2,
            1 => {
                out[j] |= ch >> 4;
                j += 1;
                out[j] = (ch & 0x0f) << 4;
            }
            2 => {
                out[j] |= ch >> 2;
                j += 1;
                out[j] = (ch & 0x03) << 6;
            }
            _ => {
                out[j] |= ch;
                j += 1;
            }
        }
        i += 1;
    }

    if strict {
        // Truncated final group (single leftover char) is invalid.
        if i % 4 == 1 {
            return Ok(Zval::Bool(false));
        }
        // Padding must be 0, or bring the total to a 4-char boundary (max 2 pads).
        if padding > 0 && (padding > 2 || !(i + padding).is_multiple_of(4)) {
            return Ok(Zval::Bool(false));
        }
    }

    out.truncate(j);
    Ok(Zval::Str(PhpStr::new(out)))
}

// ---------------------------------------------------------------------------
// md5 / sha1
// ---------------------------------------------------------------------------

fn arg_str(args: &[Zval], ctx: &mut Ctx, name: &str) -> Result<php_types::ZStr, PhpError> {
    let v = args
        .first()
        .ok_or_else(|| PhpError::Error(format!("{name}() expects at least 1 argument, 0 given")))?;
    Ok(convert::to_zstr(v, ctx.diags))
}

/// `md5(string $string, bool $binary = false): string`.
pub fn md5(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, ctx, "md5")?;
    let digest = Md5::digest(s.as_bytes());
    Ok(render_digest(&digest, binary_flag(args, 1)))
}

/// `sha1(string $string, bool $binary = false): string`.
pub fn sha1(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, ctx, "sha1")?;
    let digest = Sha1::digest(s.as_bytes());
    Ok(render_digest(&digest, binary_flag(args, 1)))
}

/// Read a file's raw bytes for the `*_file` hashers, mirroring
/// `file_get_contents`' path handling and "Failed to open stream" Warning. An
/// empty `$filename` is a `ValueError` (`Z_PARAM_PATH`); an open failure yields
/// `Ok(None)` (Warning emitted) so the caller returns `false`.
fn hash_file_bytes(
    fname: &str,
    args: &[Zval],
    ctx: &mut Ctx,
) -> Result<Option<Vec<u8>>, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let name = arg_str(args, ctx, fname)?;
    if name.as_bytes().is_empty() {
        return Err(PhpError::ValueError("Path must not be empty".to_string()));
    }
    let path = std::ffi::OsStr::from_bytes(crate::file::strip_file_wrapper(name.as_bytes()));
    match std::fs::read(path) {
        Ok(data) => Ok(Some(data)),
        Err(e) => {
            ctx.diags.push(php_types::Diag::Warning(format!(
                "{}({}): Failed to open stream: {}",
                fname,
                String::from_utf8_lossy(name.as_bytes()),
                crate::file::strerror(&e)
            )));
            Ok(None)
        }
    }
}

/// One hex nibble's value (caller guarantees `[0-9a-fA-F]`).
fn qp_hexnib(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// `quoted_printable_encode(string $string): string` — RFC 2045 Quoted-Printable
/// encoding with 75-column soft-wrapping. Mirrors `php_quot_print_encode`.
pub fn quoted_printable_encode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let s = arg_str(args, ctx, "quoted_printable_encode")?;
    let b = s.as_bytes();
    let n = b.len();
    let mut out = Vec::new();
    let mut lp: u64 = 0;
    let mut i = 0;
    while i < n {
        let c = b[i];
        let next = b.get(i + 1).copied();
        if c == 0x0d && next == Some(0x0a) && (n - i - 1) > 0 {
            out.push(0x0d);
            out.push(0x0a);
            i += 2;
            lp = 0;
            continue;
        }
        let encode = c.is_ascii_control()
            || c == 0x7f
            || (c & 0x80) != 0
            || c == b'='
            || (c == b' ' && next == Some(0x0d));
        if encode {
            lp += 3;
            let wrap = (lp > 75 && c <= 0x7f)
                || (c > 0x7f && c <= 0xdf && lp + 3 > 75)
                || (c > 0xdf && c <= 0xef && lp + 6 > 75)
                || (c > 0xef && c <= 0xf4 && lp + 9 > 75);
            if wrap {
                out.extend_from_slice(b"=\r\n");
                lp = 3;
            }
            out.push(b'=');
            out.push(HEX[(c >> 4) as usize]);
            out.push(HEX[(c & 0x0f) as usize]);
        } else {
            lp += 1;
            if lp > 75 {
                out.extend_from_slice(b"=\r\n");
                lp = 1;
            }
            out.push(c);
        }
        i += 1;
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `quoted_printable_decode(string $string): string` — decode `=HH` escapes and
/// drop RFC 2045 soft line breaks; a `=` not starting a valid escape/break is kept
/// literally. Mirrors `PHP_FUNCTION(quoted_printable_decode)`.
pub fn quoted_printable_decode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, ctx, "quoted_printable_decode")?;
    let b = s.as_bytes();
    let n = b.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        if b[i] == b'=' {
            if i + 2 < n && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit() {
                out.push((qp_hexnib(b[i + 1]) << 4) | qp_hexnib(b[i + 2]));
                i += 3;
            } else {
                // Soft line break: skip trailing spaces/tabs, then a CR/LF/CRLF.
                let mut k = 1;
                while i + k < n && (b[i + k] == 32 || b[i + k] == 9) {
                    k += 1;
                }
                if i + k >= n {
                    i += k;
                } else if b[i + k] == 13 && i + k + 1 < n && b[i + k + 1] == 10 {
                    i += k + 2;
                } else if b[i + k] == 13 || b[i + k] == 10 {
                    i += k + 1;
                } else {
                    out.push(b[i]);
                    i += 1;
                }
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `md5_file(string $filename, bool $binary = false): string|false`.
pub fn md5_file(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match hash_file_bytes("md5_file", args, ctx)? {
        Some(data) => Ok(render_digest(&Md5::digest(&data), binary_flag(args, 1))),
        None => Ok(Zval::Bool(false)),
    }
}

/// `sha1_file(string $filename, bool $binary = false): string|false`.
pub fn sha1_file(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match hash_file_bytes("sha1_file", args, ctx)? {
        Some(data) => Ok(render_digest(&Sha1::digest(&data), binary_flag(args, 1))),
        None => Ok(Zval::Bool(false)),
    }
}

// ---------------------------------------------------------------------------
// crc32 (the standalone function: zlib/IEEE, reflected)
// ---------------------------------------------------------------------------

/// `crc32(string $string): int` — zlib CRC-32 (IEEE 802.3, poly 0xEDB88320,
/// reflected). On 64-bit PHP the result is the full unsigned value as a positive
/// int.
pub fn crc32(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let s = arg_str(args, ctx, "crc32")?;
    let crc = crc32fast::hash(s.as_bytes());
    Ok(Zval::Long(crc as i64))
}

// ---------------------------------------------------------------------------
// hash() — algorithm dispatch
// ---------------------------------------------------------------------------

/// big-endian 4-byte digest of a u32 crc value.
fn crc_be(crc: u32) -> [u8; 4] {
    crc.to_be_bytes()
}

/// `hash(string $algo, string $data, bool $binary = false): string`.
///
/// Supports the common algorithms used across the test corpus. An unknown
/// algorithm is a `ValueError` (PHP 8 behaviour).
pub fn hash(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let algo = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("hash() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let data = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("hash() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let binary = binary_flag(args, 2);
    let algo = algo.as_bytes();
    let data = data.as_bytes();

    let raw = hash_raw(algo, data).ok_or_else(|| {
        PhpError::ValueError(format!(
            "hash(): Argument #1 ($algo) must be a valid hashing algorithm, \"{}\" given",
            String::from_utf8_lossy(algo)
        ))
    })?;

    Ok(render_digest(&raw, binary))
}

/// Raw digest of `data` under `algo` (case-insensitive ASCII name), or `None`
/// for an unknown algorithm. Covers the RustCrypto digests we link plus the
/// zlib/IEEE `crc32b`; shared by `hash`/`hash_file`.
pub(crate) fn hash_raw(algo: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    let raw = match algo.to_ascii_lowercase().as_slice() {
        b"md5" => Md5::digest(data).to_vec(),
        b"sha1" => Sha1::digest(data).to_vec(),
        b"sha224" => Sha224::digest(data).to_vec(),
        b"sha256" => Sha256::digest(data).to_vec(),
        b"sha384" => Sha384::digest(data).to_vec(),
        b"sha512" => Sha512::digest(data).to_vec(),
        b"sha512/224" => Sha512_224::digest(data).to_vec(),
        b"sha512/256" => Sha512_256::digest(data).to_vec(),
        b"sha3-224" => Sha3_224::digest(data).to_vec(),
        b"sha3-256" => Sha3_256::digest(data).to_vec(),
        b"sha3-384" => Sha3_384::digest(data).to_vec(),
        b"sha3-512" => Sha3_512::digest(data).to_vec(),
        // hash('crc32b') == the zlib/crc32() function output.
        b"crc32b" => crc_be(crc32fast::hash(data)).to_vec(),
        // xxHash family (PHP 8.1): big-endian digests, default seed 0. `xxh3`
        // is the 64-bit XXH3 (Composer keys its solver rules with it).
        b"xxh32" => xxhash_rust::xxh32::xxh32(data, 0).to_be_bytes().to_vec(),
        b"xxh64" => xxhash_rust::xxh64::xxh64(data, 0).to_be_bytes().to_vec(),
        b"xxh3" => xxhash_rust::xxh3::xxh3_64(data).to_be_bytes().to_vec(),
        b"xxh128" => xxhash_rust::xxh3::xxh3_128(data).to_be_bytes().to_vec(),
        _ => return None,
    };
    Some(raw)
}

/// `hash_algos(): array` — the hashing algorithms [`hash_raw`] actually
/// supports, in PHP's canonical order for the subset (userland feature-detects
/// via `in_array($algo, hash_algos())`).
pub fn hash_algos(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut out = PhpArray::new();
    for algo in [
        "md5", "sha1", "sha224", "sha256", "sha384", "sha512/224", "sha512/256", "sha512",
        "sha3-224", "sha3-256", "sha3-384", "sha3-512", "crc32b", "xxh32", "xxh64", "xxh3",
        "xxh128",
    ] {
        let _ = out.append(Zval::Str(PhpStr::from_str(algo)));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// `stream_get_wrappers(): array` — the stream wrappers this runtime actually
/// opens (`open_file_stream` / `open_php_stream` / the ureq-backed http(s)
/// layer). Userland feature-detects with `in_array($scheme, ...)`; notably
/// `phar` is absent, matching phpr's unsupported-phar reality.
pub fn stream_get_wrappers(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut out = PhpArray::new();
    for w in ["php", "file", "data", "http", "https"] {
        let _ = out.append(Zval::Str(PhpStr::from_str(w)));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Raw HMAC of `data` under `algo` keyed by `key`, or `None` for an algorithm
/// that has no HMAC construction (a non-cryptographic hash like crc32, or an
/// unknown name). `SimpleHmac` works for any block hash and hashes over-long
/// keys per the HMAC spec, so it never rejects a key length.
pub(crate) fn hmac_raw(algo: &[u8], key: &[u8], data: &[u8]) -> Option<Vec<u8>> {
    use hmac::{Mac, SimpleHmac};
    macro_rules! mac {
        ($H:ty) => {{
            let mut m = SimpleHmac::<$H>::new_from_slice(key).expect("HMAC accepts any key length");
            m.update(data);
            Some(m.finalize().into_bytes().to_vec())
        }};
    }
    match algo.to_ascii_lowercase().as_slice() {
        b"md5" => mac!(Md5),
        b"sha1" => mac!(Sha1),
        b"sha224" => mac!(Sha224),
        b"sha256" => mac!(Sha256),
        b"sha384" => mac!(Sha384),
        b"sha512" => mac!(Sha512),
        b"sha512/224" => mac!(Sha512_224),
        b"sha512/256" => mac!(Sha512_256),
        b"sha3-224" => mac!(Sha3_224),
        b"sha3-256" => mac!(Sha3_256),
        b"sha3-384" => mac!(Sha3_384),
        b"sha3-512" => mac!(Sha3_512),
        _ => None,
    }
}

/// `hash_hmac($algo, $data, $key, $binary = false)`: keyed-hash message
/// authentication code. A non-cryptographic / unknown algorithm is a
/// `ValueError` (PHP's "valid cryptographic hashing algorithm").
pub fn hash_hmac(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let algo = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("hash_hmac() expects at least 3 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let data = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("hash_hmac() expects at least 3 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let key = convert::to_zstr(
        args.get(2).ok_or_else(|| {
            PhpError::Error("hash_hmac() expects at least 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    let binary = binary_flag(args, 3);
    let raw = hmac_raw(algo.as_bytes(), key.as_bytes(), data.as_bytes()).ok_or_else(|| {
        PhpError::ValueError(
            "hash_hmac(): Argument #1 ($algo) must be a valid cryptographic hashing algorithm"
                .to_string(),
        )
    })?;
    Ok(render_digest(&raw, binary))
}

/// `hash_equals($known_string, $user_string)`: timing-attack-safe string
/// comparison. Both arguments must be strings (`TypeError` otherwise, argument
/// #1 checked first); strings of different length are unequal, and equal-length
/// strings are compared in constant time (no early exit on the first mismatch).
pub fn hash_equals(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let known = match args.first() {
        Some(Zval::Str(s)) => s,
        Some(o) => {
            return Err(PhpError::TypeError(format!(
                "hash_equals(): Argument #1 ($known_string) must be of type string, {} given",
                o.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "hash_equals() expects exactly 2 arguments, 0 given".to_string(),
            ))
        }
    };
    let user = match args.get(1) {
        Some(Zval::Str(s)) => s,
        Some(o) => {
            return Err(PhpError::TypeError(format!(
                "hash_equals(): Argument #2 ($user_string) must be of type string, {} given",
                o.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "hash_equals() expects exactly 2 arguments, 1 given".to_string(),
            ))
        }
    };
    let (a, b) = (known.as_bytes(), user.as_bytes());
    let equal = a.len() == b.len() && {
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b) {
            diff |= x ^ y;
        }
        diff == 0
    };
    Ok(Zval::Bool(equal))
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>, args: &[Zval]) -> Zval {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut direct = Vec::new();
        let dbg = std::collections::HashMap::new();
        let mut ctx = Ctx { out: &mut out, diags: &mut diags, direct_out: &mut direct, debug_info: &dbg };
        f(args, &mut ctx).unwrap()
    }

    fn s(x: &str) -> Zval {
        Zval::Str(PhpStr::new(x.as_bytes().to_vec()))
    }

    fn as_str(z: &Zval) -> String {
        match z {
            Zval::Str(p) => String::from_utf8_lossy(p.as_bytes()).into_owned(),
            _ => panic!("expected string, got {z:?}"),
        }
    }

    #[test]
    fn md5_known_vectors() {
        assert_eq!(as_str(&call(md5, &[s("")])), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            as_str(&call(md5, &[s("abc")])),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn sha1_known_vectors() {
        assert_eq!(
            as_str(&call(sha1, &[s("")])),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            as_str(&call(sha1, &[s("abc")])),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn md5_binary_is_16_bytes() {
        let z = call(md5, &[s(""), Zval::Bool(true)]);
        match z {
            Zval::Str(p) => assert_eq!(p.as_bytes().len(), 16),
            _ => panic!(),
        }
    }

    #[test]
    fn crc32_known_value() {
        // crc32("") == 0; crc32("The quick brown fox jumped over the lazy dog.")
        match call(crc32, &[s("")]) {
            Zval::Long(v) => assert_eq!(v, 0),
            _ => panic!(),
        }
        match call(crc32, &[s("123456789")]) {
            Zval::Long(v) => assert_eq!(v, 0xCBF4_3926),
            _ => panic!(),
        }
    }

    #[test]
    fn base64_roundtrip() {
        let enc = call(base64_encode, &[s("Hello, World!")]);
        assert_eq!(as_str(&enc), "SGVsbG8sIFdvcmxkIQ==");
        let dec = call(base64_decode, &[enc]);
        assert_eq!(as_str(&dec), "Hello, World!");
    }

    fn is_false(z: &Zval) -> bool {
        matches!(z, Zval::Bool(false))
    }

    #[test]
    fn base64_padding_variants() {
        assert_eq!(as_str(&call(base64_encode, &[s("a")])), "YQ==");
        assert_eq!(as_str(&call(base64_encode, &[s("ab")])), "YWI=");
        assert_eq!(as_str(&call(base64_encode, &[s("abc")])), "YWJj");
    }

    #[test]
    fn base64_decode_lenient_skips_garbage() {
        // Spaces/newlines are ignored in lenient mode.
        let dec = call(base64_decode, &[s("SGVs bG8s\nIFdv cmxk IQ==")]);
        assert_eq!(as_str(&dec), "Hello, World!");
    }

    #[test]
    fn base64_decode_strict_rejects_garbage() {
        let z = call(base64_decode, &[s("SGVsbG8h*"), Zval::Bool(true)]);
        assert!(is_false(&z));
    }

    #[test]
    fn hash_dispatch() {
        assert_eq!(
            as_str(&call(hash, &[s("sha256"), s("abc")])),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            as_str(&call(hash, &[s("crc32b"), s("abc")])),
            "352441c2"
        );
    }
}
