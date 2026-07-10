//! ext/zlib — string (de)compression builtins.
//!
//! These call the **system zlib** (the same `libz.1.2.12` brew-php links)
//! directly through `libz-sys`, with PHP's exact `deflateInit2` parameters —
//! `MAX_MEM_LEVEL` and `windowBits` = the `ZLIB_ENCODING_*` value — so the
//! compressed output is byte-identical to PHP's at every level and format. (A
//! higher-level crate hard-codes `DEF_MEM_LEVEL`, which diverges on larger
//! inputs.) zlib writes the zlib/gzip framing itself from `windowBits`, so
//! raw/zlib/gzip all go through one code path. The gz-file/stream, incremental
//! (`deflate_*`/`inflate_*`) and `ob_gzhandler` functions are a separate batch.

use std::mem::{size_of, MaybeUninit};
use std::os::raw::c_int;

use libz_sys::{
    deflate, deflateBound, deflateEnd, deflateInit2_, inflate, inflateEnd, inflateInit2_,
    z_stream, zlibVersion, Bytef, Z_BUF_ERROR, Z_DEFAULT_STRATEGY, Z_DEFLATED, Z_FINISH,
    Z_NO_FLUSH, Z_OK, Z_STREAM_END,
};
use php_runtime::Ctx;
use php_types::{convert, Diag, PhpError, PhpStr, Zval};

// PHP's ZLIB_ENCODING_* == the zlib windowBits: raw = -15, zlib = 15, gzip = 31.
const ENC_RAW: i64 = -15;
const ENC_DEFLATE: i64 = 15;
const ENC_GZIP: i64 = 31;
// `inflateInit2` windowBits for `zlib_decode`'s automatic header detection.
const AUTODETECT: i64 = 15 + 32;
const MAX_MEM_LEVEL: c_int = 9;

/// Compress `data` with zlib, matching PHP's `deflateInit2(level, Z_DEFLATED,
/// window_bits, MAX_MEM_LEVEL, Z_DEFAULT_STRATEGY)`. `window_bits` selects the
/// framing (raw/zlib/gzip). `level` accepts `-1` (zlib's default).
fn zlib_compress(data: &[u8], level: i32, window_bits: i32) -> Vec<u8> {
    unsafe {
        // `z_stream` has non-nullable fn-pointer fields, so it can't be a
        // zeroed *value* — keep it in `MaybeUninit` and touch it only through the
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

/// Decompress `data` with zlib. `window_bits` selects the expected framing
/// (raw/zlib/gzip, or [`AUTODETECT`]). `None` on any zlib data error (including a
/// truncated stream, matching PHP's "data error").
fn zlib_uncompress(data: &[u8], window_bits: i32) -> Option<Vec<u8>> {
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
                Z_STREAM_END => break Some(out),
                Z_OK => continue,       // made progress, more to do
                _ => break None,        // Z_BUF_ERROR (truncated) / Z_DATA_ERROR / …
            }
        };
        inflateEnd(z);
        result
    }
}

/// The `$data` bytes of argument `idx`, coerced like PHP's `string` typing.
fn bytes_arg(argv: &[Zval], idx: usize, ctx: &mut Ctx) -> Vec<u8> {
    argv.get(idx)
        .map(|v| convert::to_zstr_cast(v, ctx.diags).as_bytes().to_vec())
        .unwrap_or_default()
}

/// A compression level argument (default `-1`), validated to `-1..=9`.
fn level_arg(argv: &[Zval], idx: usize, ctx: &mut Ctx, fname: &str) -> Result<i32, PhpError> {
    let l = argv.get(idx).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1);
    if !(-1..=9).contains(&l) {
        return Err(PhpError::ValueError(format!(
            "{fname}(): Argument #2 ($level) must be between -1 and 9"
        )));
    }
    Ok(l as i32)
}

/// The `$encoding` (windowBits) argument at `idx`, defaulting to `default`,
/// validated to one of ZLIB_ENCODING_RAW / _DEFLATE / _GZIP.
fn encoding_arg(argv: &[Zval], idx: usize, default: i64, ctx: &mut Ctx, fname: &str) -> Result<i32, PhpError> {
    let e = argv.get(idx).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(default);
    if matches!(e, ENC_RAW | ENC_DEFLATE | ENC_GZIP) {
        Ok(e as i32)
    } else {
        Err(PhpError::ValueError(format!(
            "{fname}(): Argument #3 ($encoding) must be one of ZLIB_ENCODING_RAW, ZLIB_ENCODING_GZIP, or ZLIB_ENCODING_DEFLATE"
        )))
    }
}

/// The `$max_length` argument (0 = unlimited); negative is a ValueError.
fn max_length_arg(argv: &[Zval], idx: usize, ctx: &mut Ctx, fname: &str) -> Result<usize, PhpError> {
    let l = argv.get(idx).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if l < 0 {
        return Err(PhpError::ValueError(format!(
            "{fname}(): Argument #2 ($max_length) must be greater than or equal to 0"
        )));
    }
    Ok(l as usize)
}

/// Turn a decode result into the PHP return: `false` + "data error" on failure,
/// `false` + "insufficient memory" when a positive `$max_length` is exceeded.
fn finish_inflate(res: Option<Vec<u8>>, max: usize, ctx: &mut Ctx, fname: &str) -> Result<Zval, PhpError> {
    match res {
        Some(v) if max > 0 && v.len() > max => {
            ctx.diags.push(Diag::Warning(format!("{fname}(): insufficient memory")));
            Ok(Zval::Bool(false))
        }
        Some(v) => Ok(Zval::Str(PhpStr::new(v))),
        None => {
            ctx.diags.push(Diag::Warning(format!("{fname}(): data error")));
            Ok(Zval::Bool(false))
        }
    }
}

/// `gzdeflate(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_RAW): string|false`
pub fn gzdeflate(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let level = level_arg(argv, 1, ctx, "gzdeflate")?;
    let enc = encoding_arg(argv, 2, ENC_RAW, ctx, "gzdeflate")?;
    Ok(Zval::Str(PhpStr::new(zlib_compress(&data, level, enc))))
}

/// `gzcompress(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_DEFLATE): string|false`
pub fn gzcompress(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let level = level_arg(argv, 1, ctx, "gzcompress")?;
    let enc = encoding_arg(argv, 2, ENC_DEFLATE, ctx, "gzcompress")?;
    Ok(Zval::Str(PhpStr::new(zlib_compress(&data, level, enc))))
}

/// `gzencode(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_GZIP): string|false`
pub fn gzencode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let level = level_arg(argv, 1, ctx, "gzencode")?;
    let enc = encoding_arg(argv, 2, ENC_GZIP, ctx, "gzencode")?;
    Ok(Zval::Str(PhpStr::new(zlib_compress(&data, level, enc))))
}

/// `gzinflate(string $data, int $max_length = 0): string|false`
pub fn gzinflate(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzinflate")?;
    finish_inflate(zlib_uncompress(&data, ENC_RAW as i32), max, ctx, "gzinflate")
}

/// `gzuncompress(string $data, int $max_length = 0): string|false`
pub fn gzuncompress(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzuncompress")?;
    finish_inflate(zlib_uncompress(&data, ENC_DEFLATE as i32), max, ctx, "gzuncompress")
}

/// `gzdecode(string $data, int $max_length = 0): string|false`
pub fn gzdecode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzdecode")?;
    finish_inflate(zlib_uncompress(&data, ENC_GZIP as i32), max, ctx, "gzdecode")
}

/// `zlib_encode(string $data, int $encoding, int $level = -1): string|false`
pub fn zlib_encode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let encoding = argv.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !matches!(encoding, ENC_RAW | ENC_DEFLATE | ENC_GZIP) {
        return Ok(Zval::Bool(false));
    }
    let level = argv.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1) as i32;
    Ok(Zval::Str(PhpStr::new(zlib_compress(&data, level, encoding as i32))))
}

/// `zlib_decode(string $data, int $max_length = 0): string|false` — auto-detect
/// zlib / gzip / (fallback) raw deflate.
pub fn zlib_decode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let out = zlib_uncompress(&data, AUTODETECT as i32).or_else(|| zlib_uncompress(&data, ENC_RAW as i32));
    match out {
        Some(v) => Ok(Zval::Str(PhpStr::new(v))),
        None => Ok(Zval::Bool(false)),
    }
}

/// `zlib_get_coding_type(): string|false` — the transparent output-compression
/// coding. phpr never compresses its output, so this is always `false`.
pub fn zlib_get_coding_type(_argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(false))
}
