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
    deflate, deflateBound, deflateEnd, deflateInit2_, deflateReset, deflateSetDictionary, inflate,
    inflateEnd, inflateInit2_, inflateReset, inflateSetDictionary, z_stream, zlibVersion, Bytef,
    Z_DEFAULT_STRATEGY, Z_DEFLATED, Z_FINISH, Z_NEED_DICT, Z_NO_FLUSH, Z_OK, Z_STREAM_END,
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

/// How a [`ZCtx::add`] step failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZErr {
    /// A zlib data error (corrupt/invalid stream).
    Data,
    /// `Z_NEED_DICT` could not be satisfied: no preset dictionary, or its
    /// adler32 does not match the stream's expectation.
    DictMismatch,
}

/// A stateful (incremental) zlib context backing PHP's `deflate_init`/`inflate_init`
/// family: the `z_stream` lives at a fixed heap address for the context's whole
/// life (zlib's internal state back-references it), fed chunk by chunk via
/// [`ZCtx::add`]. Dropped → `deflateEnd`/`inflateEnd` + dealloc.
pub struct ZCtx {
    z: *mut z_stream,
    deflate: bool,
    dict: Option<Vec<u8>>,
    last_status: i32,
}

impl ZCtx {
    fn alloc() -> *mut z_stream {
        Box::into_raw(Box::new(MaybeUninit::<z_stream>::zeroed())) as *mut z_stream
    }

    /// A deflate context with PHP's `deflate_init` parameters (already mapped to
    /// real windowBits). An optional preset dictionary is installed immediately.
    pub fn new_deflate(level: i32, window_bits: i32, mem_level: i32, strategy: i32, dict: Option<Vec<u8>>) -> Option<ZCtx> {
        unsafe {
            let z = Self::alloc();
            if deflateInit2_(
                z,
                level,
                Z_DEFLATED,
                window_bits,
                mem_level,
                strategy,
                zlibVersion(),
                size_of::<z_stream>() as c_int,
            ) != Z_OK
            {
                drop(Box::from_raw(z as *mut MaybeUninit<z_stream>));
                return None;
            }
            if let Some(d) = &dict {
                deflateSetDictionary(z, d.as_ptr(), d.len() as _);
            }
            Some(ZCtx { z, deflate: true, dict, last_status: Z_OK })
        }
    }

    /// An inflate context; a preset dictionary is installed on `Z_NEED_DICT` — or
    /// immediately for a raw stream (negative windowBits), which never signals it.
    pub fn new_inflate(window_bits: i32, dict: Option<Vec<u8>>) -> Option<ZCtx> {
        unsafe {
            let z = Self::alloc();
            if inflateInit2_(z, window_bits, zlibVersion(), size_of::<z_stream>() as c_int) != Z_OK {
                drop(Box::from_raw(z as *mut MaybeUninit<z_stream>));
                return None;
            }
            if window_bits < 0 {
                if let Some(d) = &dict {
                    inflateSetDictionary(z, d.as_ptr(), d.len() as _);
                }
            }
            Some(ZCtx { z, deflate: false, dict, last_status: Z_OK })
        }
    }

    /// Feed `data` with the given zlib `flush` mode, returning whatever output the
    /// stream produces. A finished context (previous step hit `Z_STREAM_END`) is
    /// reset first, so it can be reused for a fresh stream (PHP's
    /// deflate_init/inflate_init reuse semantics).
    pub fn add(&mut self, data: &[u8], flush: i32) -> Result<Vec<u8>, ZErr> {
        unsafe {
            let z = self.z;
            if self.last_status == Z_STREAM_END {
                if self.deflate {
                    deflateReset(z);
                } else {
                    inflateReset(z);
                    // A raw stream's preset dictionary must be re-installed.
                    if let Some(d) = &self.dict {
                        inflateSetDictionary(z, d.as_ptr(), d.len() as _);
                    }
                }
                self.last_status = Z_OK;
            }
            (*z).next_in = data.as_ptr() as *mut Bytef;
            (*z).avail_in = data.len() as _;
            let mut out = Vec::new();
            let mut buf = vec![0u8; 32768];
            loop {
                (*z).next_out = buf.as_mut_ptr();
                (*z).avail_out = buf.len() as _;
                let ret = if self.deflate { deflate(z, flush) } else { inflate(z, flush) };
                out.extend_from_slice(&buf[..buf.len() - (*z).avail_out as usize]);
                self.last_status = ret;
                match ret {
                    Z_STREAM_END => return Ok(out),
                    Z_NEED_DICT if !self.deflate => {
                        // A preset dictionary satisfies the demand; a missing or
                        // adler32-mismatched one is PHP's dictionary error.
                        let Some(d) = &self.dict else { return Err(ZErr::DictMismatch) };
                        if inflateSetDictionary(z, d.as_ptr(), d.len() as _) != Z_OK {
                            return Err(ZErr::DictMismatch);
                        }
                    }
                    Z_OK => {
                        if (*z).avail_in == 0 && (*z).avail_out != 0 {
                            return Ok(out); // consumed everything, output complete
                        }
                    }
                    // Z_BUF_ERROR just means "no progress possible now" — with all
                    // input consumed that is a normal end of this add() step.
                    libz_sys::Z_BUF_ERROR if (*z).avail_in == 0 => return Ok(out),
                    _ => return Err(ZErr::Data),
                }
            }
        }
    }

    /// Total input bytes consumed so far (`inflate_get_read_len`).
    pub fn total_in(&self) -> i64 {
        unsafe { (*self.z).total_in as i64 }
    }

    /// The last zlib status code this context produced (`inflate_get_status`).
    pub fn last_status(&self) -> i32 {
        self.last_status
    }
}

impl Drop for ZCtx {
    fn drop(&mut self) {
        unsafe {
            if self.deflate {
                deflateEnd(self.z);
            } else {
                inflateEnd(self.z);
            }
            drop(Box::from_raw(self.z as *mut MaybeUninit<z_stream>));
        }
    }
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
