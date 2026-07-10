//! Low-level zlib (de)compression over the **system zlib** via `libz-sys`, with
//! PHP's exact `deflateInit2` parameters (`MAX_MEM_LEVEL`, `windowBits` = the
//! `ZLIB_ENCODING_*` value) so compressed output is byte-identical to PHP's.
//! Shared by the ext/zlib value builtins (php-builtins) and the VM's gz stream
//! handling (gzopen / `compress.zlib://`), so the FFI lives once, here in the
//! bottom crate. A higher-level crate (flate2/miniz_oxide/zlib-rs) hard-codes
//! `DEF_MEM_LEVEL` and diverges from PHP on larger inputs.

use std::mem::{size_of, MaybeUninit};
use std::os::raw::c_int;

use libz_sys::{
    deflate, deflateBound, deflateEnd, deflateInit2_, inflate, inflateEnd, inflateInit2_,
    z_stream, zlibVersion, Bytef, Z_DEFAULT_STRATEGY, Z_DEFLATED, Z_FINISH, Z_NO_FLUSH, Z_OK,
    Z_STREAM_END,
};

/// PHP's `ZLIB_ENCODING_*` == the zlib windowBits: raw = -15, zlib = 15, gzip = 31.
pub const ENC_RAW: i32 = -15;
pub const ENC_DEFLATE: i32 = 15;
pub const ENC_GZIP: i32 = 31;
/// `inflateInit2` windowBits for automatic zlib/gzip header detection.
pub const AUTODETECT: i32 = 15 + 32;
const MAX_MEM_LEVEL: c_int = 9;

/// Compress `data`, matching PHP's `deflateInit2(level, Z_DEFLATED, window_bits,
/// MAX_MEM_LEVEL, Z_DEFAULT_STRATEGY)`. `window_bits` selects the framing
/// (raw/zlib/gzip). `level` accepts `-1` (zlib's default).
pub fn compress(data: &[u8], level: i32, window_bits: i32) -> Vec<u8> {
    unsafe {
        // `z_stream` has non-nullable fn-pointer fields, so it can't be a zeroed
        // *value* — keep it in `MaybeUninit` and touch it only through the
        // pointer; `deflateInit2_` reads the zeroed zalloc/zfree as NULL (zlib's
        // default allocator) and initialises the rest.
        let mut zbox = MaybeUninit::<z_stream>::zeroed();
        let z = zbox.as_mut_ptr();
        if deflateInit2_(
            z,
            level,
            Z_DEFLATED,
            window_bits,
            MAX_MEM_LEVEL,
            Z_DEFAULT_STRATEGY,
            zlibVersion(),
            size_of::<z_stream>() as c_int,
        ) != Z_OK
        {
            return Vec::new();
        }
        let mut out = vec![0u8; deflateBound(z, data.len() as _) as usize];
        (*z).next_in = data.as_ptr() as *mut Bytef;
        (*z).avail_in = data.len() as _;
        (*z).next_out = out.as_mut_ptr();
        (*z).avail_out = out.len() as _;
        deflate(z, Z_FINISH);
        out.truncate(out.len() - (*z).avail_out as usize);
        deflateEnd(z);
        out
    }
}

/// Decompress one stream from `data`. `window_bits` selects the expected framing
/// (raw/zlib/gzip, or [`AUTODETECT`]). Returns the decoded bytes plus how many
/// input bytes the stream consumed; `None` on any zlib data error (including a
/// truncated stream, matching PHP's "data error").
pub fn uncompress_one(data: &[u8], window_bits: i32) -> Option<(Vec<u8>, usize)> {
    unsafe {
        let mut zbox = MaybeUninit::<z_stream>::zeroed();
        let z = zbox.as_mut_ptr();
        if inflateInit2_(z, window_bits, zlibVersion(), size_of::<z_stream>() as c_int) != Z_OK {
            return None;
        }
        (*z).next_in = data.as_ptr() as *mut Bytef;
        (*z).avail_in = data.len() as _;
        let mut out = Vec::new();
        let mut buf = vec![0u8; 32768];
        let result = loop {
            (*z).next_out = buf.as_mut_ptr();
            (*z).avail_out = buf.len() as _;
            let ret = inflate(z, Z_NO_FLUSH);
            out.extend_from_slice(&buf[..buf.len() - (*z).avail_out as usize]);
            match ret {
                Z_STREAM_END => break Some((out, (*z).total_in as usize)),
                Z_OK => continue,   // made progress, more to do
                _ => break None,    // Z_BUF_ERROR (truncated) / Z_DATA_ERROR / …
            }
        };
        inflateEnd(z);
        result
    }
}

/// Decompress one stream, discarding the consumed count (the common case).
pub fn uncompress(data: &[u8], window_bits: i32) -> Option<Vec<u8>> {
    uncompress_one(data, window_bits).map(|(v, _)| v)
}

/// Decode a gz *file's* payload the way PHP's gz stream layer does: every
/// concatenated gzip member in sequence (an appended gz file has several).
/// Trailing garbage after a valid member ends the decode (like gzip tools).
pub fn gzip_decode_members(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut rest = data;
    loop {
        let (chunk, used) = uncompress_one(rest, ENC_GZIP)?;
        out.extend_from_slice(&chunk);
        rest = &rest[used.min(rest.len())..];
        if rest.len() < 2 || rest[0] != 0x1f || rest[1] != 0x8b {
            return Some(out);
        }
    }
}
