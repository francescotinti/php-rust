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
use sha2::{Sha256, Sha384, Sha512};
use sha1::Digest; // the `Digest` trait is re-exported by every RustCrypto crate

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};

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

    let raw: Vec<u8> = match algo.to_ascii_lowercase().as_slice() {
        b"md5" => Md5::digest(data).to_vec(),
        b"sha1" => Sha1::digest(data).to_vec(),
        b"sha256" => Sha256::digest(data).to_vec(),
        b"sha384" => Sha384::digest(data).to_vec(),
        b"sha512" => Sha512::digest(data).to_vec(),
        // hash('crc32b') == the zlib/crc32() function output.
        b"crc32b" => crc_be(crc32fast::hash(data)).to_vec(),
        _ => {
            return Err(PhpError::ValueError(format!(
                "hash(): Argument #1 ($algo) must be a valid hashing algorithm, \"{}\" given",
                String::from_utf8_lossy(algo)
            )))
        }
    };

    Ok(render_digest(&raw, binary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use php_types::Diags;

    fn call(f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>, args: &[Zval]) -> Zval {
        let mut out = Vec::new();
        let mut diags: Diags = Vec::new();
        let mut ctx = Ctx {
            out: &mut out,
            diags: &mut diags,
        };
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
