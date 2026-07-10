//! ext/zlib — string (de)compression builtins + the pure gz-file readers.
//!
//! The zlib FFI itself lives in `php_types::zlibio` (system zlib, PHP's exact
//! `deflateInit2` parameters → byte-identical output; shared with the VM's
//! gzopen / `compress.zlib://` handling). This module is the ZPP layer:
//! argument coercion/validation and PHP's exact error strings. The stream
//! (`gzopen` & co.), incremental (`deflate_*`/`inflate_*`) and `ob_gzhandler`
//! functions are a separate batch.

use php_runtime::Ctx;
use php_types::zlibio;
use php_types::{convert, Diag, PhpArray, PhpError, PhpStr, Zval};

const ENC_RAW: i64 = zlibio::ENC_RAW as i64;
const ENC_DEFLATE: i64 = zlibio::ENC_DEFLATE as i64;
const ENC_GZIP: i64 = zlibio::ENC_GZIP as i64;

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
    Ok(Zval::Str(PhpStr::new(zlibio::compress(&data, level, enc))))
}

/// `gzcompress(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_DEFLATE): string|false`
pub fn gzcompress(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let level = level_arg(argv, 1, ctx, "gzcompress")?;
    let enc = encoding_arg(argv, 2, ENC_DEFLATE, ctx, "gzcompress")?;
    Ok(Zval::Str(PhpStr::new(zlibio::compress(&data, level, enc))))
}

/// `gzencode(string $data, int $level = -1, int $encoding = ZLIB_ENCODING_GZIP): string|false`
pub fn gzencode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let level = level_arg(argv, 1, ctx, "gzencode")?;
    let enc = encoding_arg(argv, 2, ENC_GZIP, ctx, "gzencode")?;
    Ok(Zval::Str(PhpStr::new(zlibio::compress(&data, level, enc))))
}

/// `gzinflate(string $data, int $max_length = 0): string|false`
pub fn gzinflate(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzinflate")?;
    finish_inflate(zlibio::uncompress(&data, zlibio::ENC_RAW), max, ctx, "gzinflate")
}

/// `gzuncompress(string $data, int $max_length = 0): string|false`
pub fn gzuncompress(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzuncompress")?;
    finish_inflate(zlibio::uncompress(&data, zlibio::ENC_DEFLATE), max, ctx, "gzuncompress")
}

/// `gzdecode(string $data, int $max_length = 0): string|false`
pub fn gzdecode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let max = max_length_arg(argv, 1, ctx, "gzdecode")?;
    finish_inflate(zlibio::uncompress(&data, zlibio::ENC_GZIP), max, ctx, "gzdecode")
}

/// `zlib_encode(string $data, int $encoding, int $level = -1): string|false`
pub fn zlib_encode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let encoding = argv.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !matches!(encoding, ENC_RAW | ENC_DEFLATE | ENC_GZIP) {
        return Ok(Zval::Bool(false));
    }
    let level = argv.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1) as i32;
    Ok(Zval::Str(PhpStr::new(zlibio::compress(&data, level, encoding as i32))))
}

/// `zlib_decode(string $data, int $max_length = 0): string|false` — auto-detect
/// zlib / gzip / (fallback) raw deflate. Single stream, like PHP.
pub fn zlib_decode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let data = bytes_arg(argv, 0, ctx);
    let out = zlibio::uncompress(&data, zlibio::AUTODETECT)
        .or_else(|| zlibio::uncompress(&data, zlibio::ENC_RAW));
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

/// The underlying path if `spec` targets the `compress.zlib://` wrapper (a
/// nested `file://` is also stripped, as PHP's wrapper chain does).
pub(crate) fn zlib_wrapped(spec: &[u8]) -> Option<&[u8]> {
    let rest = spec.strip_prefix(b"compress.zlib://".as_slice())?;
    Some(rest.strip_prefix(b"file://".as_slice()).unwrap_or(rest))
}

/// gzip-compress `data` (zlib default level, like the wrapper) and write `path`.
pub(crate) fn write_gz_file(path: &[u8], data: &[u8]) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    std::fs::write(
        std::ffi::OsStr::from_bytes(path),
        zlibio::compress(data, -1, zlibio::ENC_GZIP),
    )
}

/// Read the gz file at `path` and decode it the way PHP's gz stream layer does:
/// every concatenated gzip member; a file without the gzip magic reads
/// transparently as plain bytes. `None` = open/decode failure.
pub(crate) fn read_gz_file(path: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let raw = std::fs::read(std::ffi::OsStr::from_bytes(path)).ok()?;
    if raw.starts_with(&[0x1f, 0x8b]) {
        zlibio::gzip_decode_members(&raw)
    } else {
        Some(raw) // transparent read of a plain file
    }
}

/// `gzfile(string $filename, int $use_include_path = 0): array|false` — the
/// decoded contents split into lines, each keeping its trailing newline.
pub fn gzfile(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let path = bytes_arg(argv, 0, ctx);
    let Some(data) = read_gz_file(&path) else {
        ctx.diags.push(Diag::Warning(format!(
            "gzfile({}): Failed to open stream: No such file or directory",
            String::from_utf8_lossy(&path)
        )));
        return Ok(Zval::Bool(false));
    };
    let mut arr = PhpArray::new();
    let mut start = 0;
    for (i, &b) in data.iter().enumerate() {
        if b == b'\n' {
            let _ = arr.append(Zval::Str(PhpStr::new(data[start..=i].to_vec())));
            start = i + 1;
        }
    }
    if start < data.len() {
        let _ = arr.append(Zval::Str(PhpStr::new(data[start..].to_vec())));
    }
    Ok(Zval::Array(std::rc::Rc::new(arr)))
}

/// `readgzfile(string $filename, int $use_include_path = 0): int|false` — echo
/// the decoded contents; returns the number of (uncompressed) bytes.
pub fn readgzfile(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let path = bytes_arg(argv, 0, ctx);
    let Some(data) = read_gz_file(&path) else {
        ctx.diags.push(Diag::Warning(format!(
            "readgzfile({}): Failed to open stream: No such file or directory",
            String::from_utf8_lossy(&path)
        )));
        return Ok(Zval::Bool(false));
    };
    let n = data.len() as i64;
    ctx.out.extend_from_slice(&data);
    Ok(Zval::Long(n))
}

// ---- gz stream ops -------------------------------------------------------
// A gz stream is an ordinary stream resource (gzopen decodes up front / GzFile
// buffers writes), so each gz op IS the corresponding file op — but PHP brands
// its errors with the gz name ("gzread(): Argument #1 …"), so the aliases run
// the file op and rebrand any error/diagnostic it produced.

/// Replace the leading `from()` with `to()` in a message PHP brands by fname.
fn rebrand(msg: &str, from: &str, to: &str) -> String {
    let pfx = format!("{from}()");
    match msg.strip_prefix(&pfx) {
        Some(rest) => format!("{to}(){rest}"),
        None => msg.to_string(),
    }
}

/// Run a file op under a gz alias: any `PhpError` or diagnostic it raises has
/// its `from()` prefix rebranded to `to()`.
fn gz_alias(
    f: fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>,
    argv: &[Zval],
    ctx: &mut Ctx,
    from: &str,
    to: &str,
) -> Result<Zval, PhpError> {
    let mark = ctx.diags.len();
    let r = f(argv, ctx);
    for d in ctx.diags[mark..].iter_mut() {
        *d = match d {
            Diag::Warning(m) => Diag::Warning(rebrand(m, from, to)),
            Diag::Notice(m) => Diag::Notice(rebrand(m, from, to)),
            Diag::Deprecated(m) => Diag::Deprecated(rebrand(m, from, to)),
        };
    }
    r.map_err(|e| match e {
        PhpError::Error(m) => PhpError::Error(rebrand(&m, from, to)),
        PhpError::TypeError(m) => PhpError::TypeError(rebrand(&m, from, to)),
        PhpError::ValueError(m) => PhpError::ValueError(rebrand(&m, from, to)),
        PhpError::ArgumentCountError(m) => PhpError::ArgumentCountError(rebrand(&m, from, to)),
        other => other,
    })
}

/// Whether argument #1 is a gz stream, and which direction: `Some(true)` = a
/// write stream (`GzFile` buffer), `Some(false)` = a read stream (decoded
/// `Memory` with the gz `eof_on_exhaust` marker), `None` = not a gz stream.
fn gz_stream_dir(argv: &[Zval]) -> Option<bool> {
    let Some(Zval::Resource(rc)) = argv.first().map(|v| v.deref_clone()) else {
        return None;
    };
    let mut b = rc.borrow_mut();
    let s = b.as_stream_mut()?;
    if matches!(s.backend, php_types::StreamBackend::GzFile { .. }) {
        Some(true)
    } else if s.eof_on_exhaust {
        Some(false)
    } else {
        None
    }
}

pub fn gzread(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // PHP: reading a write-mode gz stream is a silent `false` (no notice).
    if gz_stream_dir(argv) == Some(true) {
        return Ok(Zval::Bool(false));
    }
    gz_alias(crate::file::fread, argv, ctx, "fread", "gzread")
}
pub fn gzwrite(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // PHP: writing a read-mode gz stream reports 0 bytes (no notice, not false).
    if gz_stream_dir(argv) == Some(false) {
        return Ok(Zval::Long(0));
    }
    gz_alias(crate::file::fwrite, argv, ctx, "fwrite", "gzwrite")
}
pub fn gzputs(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if gz_stream_dir(argv) == Some(false) {
        return Ok(Zval::Long(0));
    }
    gz_alias(crate::file::fwrite, argv, ctx, "fwrite", "gzputs")
}
pub fn gzclose(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::fclose, argv, ctx, "fclose", "gzclose")
}
pub fn gzgets(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::fgets, argv, ctx, "fgets", "gzgets")
}
pub fn gzgetc(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::fgetc, argv, ctx, "fgetc", "gzgetc")
}
pub fn gzeof(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::feof, argv, ctx, "feof", "gzeof")
}
pub fn gzrewind(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // PHP: a write-mode gz stream cannot rewind — `false`, buffer untouched.
    if gz_stream_dir(argv) == Some(true) {
        return Ok(Zval::Bool(false));
    }
    gz_alias(crate::file::rewind, argv, ctx, "rewind", "gzrewind")
}
pub fn gztell(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::ftell, argv, ctx, "ftell", "gztell")
}
pub fn gzseek(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    // PHP: SEEK_END is not supported on a gz stream in either direction — the
    // warning fires and the position is untouched.
    if gz_stream_dir(argv).is_some() {
        let whence = argv.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
        if whence == 2 {
            ctx.diags
                .push(Diag::Warning("gzseek(): SEEK_END is not supported".to_string()));
            return Ok(Zval::Long(-1));
        }
    }
    gz_alias(crate::file::fseek, argv, ctx, "fseek", "gzseek")
}
pub fn gzpassthru(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    gz_alias(crate::file::fpassthru, argv, ctx, "fpassthru", "gzpassthru")
}

// ---- incremental (de)compression contexts --------------------------------
// PHP's deflate_init/inflate_init family: a stateful z_stream fed chunk by
// chunk. The PHP-visible DeflateContext/InflateContext are prelude classes
// wrapping an id into this thread-local table (the curl-handle pattern); the
// prelude functions marshal `$ctx->__id` into these `__`-prefixed builtins,
// whose error strings are branded with the PUBLIC function names.

thread_local! {
    static ZCTXS: std::cell::RefCell<Vec<Option<php_types::zlibio::ZCtx>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Validate and extract the `dictionary` option: a string, or an array of
/// non-empty NUL-free words (concatenated). `fname` brands the ValueErrors.
fn dict_option(opts: &PhpArray, fname: &str) -> Result<Option<Vec<u8>>, PhpError> {
    let Some(v) = opts.get(&php_types::Key::from_bytes(b"dictionary")) else {
        return Ok(None);
    };
    match v.deref_clone() {
        Zval::Array(words) => {
            let mut dict = Vec::new();
            for (_, w) in words.iter() {
                let w = w.deref_clone();
                if let Zval::Object(o) = &w {
                    // No VM here to run __toString: PHP's (string) failure text.
                    return Err(PhpError::Error(format!(
                        "Object of class {} could not be converted to string",
                        String::from_utf8_lossy(o.borrow().class_name.as_bytes())
                    )));
                }
                let bytes = match &w {
                    Zval::Str(s) => s.as_bytes().to_vec(),
                    other => {
                        let mut d = Vec::new();
                        let b = php_types::convert::to_zstr(other, &mut d).as_bytes().to_vec();
                        b
                    }
                };
                if bytes.is_empty() {
                    return Err(PhpError::ValueError(format!(
                        "{fname}(): Argument #2 ($options) must not contain empty strings"
                    )));
                }
                if bytes.contains(&0) {
                    return Err(PhpError::ValueError(format!(
                        "{fname}(): Argument #2 ($options) must not contain strings with null bytes"
                    )));
                }
                // PHP joins the words NUL-separated with a trailing NUL (an array
                // dictionary equals `implode("\0", $words) . "\0"`).
                dict.extend_from_slice(&bytes);
                dict.push(0);
            }
            Ok(Some(dict))
        }
        other => {
            let mut d = Vec::new();
            Ok(Some(php_types::convert::to_zstr(&other, &mut d).as_bytes().to_vec()))
        }
    }
}

/// The `$options` argument as a table: an array as-is, an object via its
/// property table (PHP's ZPP `h` accepts both — GH-17745), absent → empty.
fn opts_table(argv: &[Zval], fname: &str) -> Result<PhpArray, PhpError> {
    match argv.get(1).map(|v| v.deref_clone()) {
        None | Some(Zval::Null) | Some(Zval::Undef) => Ok(PhpArray::new()),
        Some(Zval::Array(a)) => Ok((*a).clone()),
        Some(Zval::Object(o)) => {
            let mut t = PhpArray::new();
            for (n, v) in o.borrow().props.iter() {
                t.insert(php_types::Key::from_bytes(n), v.deref_clone());
            }
            Ok(t)
        }
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #2 ($options) must be of type array, {} given",
            other.type_name_for_error()
        ))),
    }
}

/// An integer option in `opts`, defaulting to `default`.
fn int_option(opts: &PhpArray, key: &[u8], default: i64, ctx: &mut Ctx) -> i64 {
    opts.get(&php_types::Key::from_bytes(key))
        .map(|v| convert::to_long_cast(&v.deref_clone(), ctx.diags))
        .unwrap_or(default)
}

/// Map PHP's (encoding, window) pair to the real `windowBits`: RAW → -window,
/// GZIP → window + 16, DEFLATE → window.
fn window_bits_for(encoding: i64, window: i64) -> i32 {
    match encoding {
        ENC_RAW => -(window as i32),
        ENC_GZIP => window as i32 + 16,
        _ => window as i32,
    }
}

/// `__deflate_init($encoding, $options)` → context id. Errors are PHP's exact
/// `deflate_init()` messages.
pub fn __deflate_init(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let encoding = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !matches!(encoding, ENC_RAW | ENC_DEFLATE | ENC_GZIP) {
        return Err(PhpError::ValueError(
            "deflate_init(): Argument #1 ($encoding) must be one of ZLIB_ENCODING_RAW, ZLIB_ENCODING_GZIP, or ZLIB_ENCODING_DEFLATE".to_string(),
        ));
    }
    let opts = opts_table(argv, "deflate_init")?;
    let opts = &opts;
    let level = int_option(opts, b"level", -1, ctx);
    if !(-1..=9).contains(&level) {
        return Err(PhpError::ValueError(
            "deflate_init(): \"level\" option must be between -1 and 9".to_string(),
        ));
    }
    let memory = int_option(opts, b"memory", 8, ctx);
    if !(1..=9).contains(&memory) {
        return Err(PhpError::ValueError(
            "deflate_init(): \"memory\" option must be between 1 and 9".to_string(),
        ));
    }
    let window = int_option(opts, b"window", 15, ctx);
    if !(8..=15).contains(&window) {
        return Err(PhpError::ValueError(
            "deflate_init(): \"window\" option must be between 8 and 15".to_string(),
        ));
    }
    let strategy = int_option(opts, b"strategy", 0, ctx);
    if !(0..=4).contains(&strategy) {
        return Err(PhpError::ValueError(
            "deflate_init(): \"strategy\" option must be one of ZLIB_FILTERED, ZLIB_HUFFMAN_ONLY, ZLIB_RLE, ZLIB_FIXED or ZLIB_DEFAULT_STRATEGY".to_string(),
        ));
    }
    let dict = dict_option(opts, "deflate_init")?;
    let Some(zc) = php_types::zlibio::ZCtx::new_deflate(
        level as i32,
        window_bits_for(encoding, window),
        memory as i32,
        strategy as i32,
        dict,
    ) else {
        return Ok(Zval::Bool(false));
    };
    let id = ZCTXS.with(|t| {
        let mut t = t.borrow_mut();
        t.push(Some(zc));
        t.len() - 1
    });
    Ok(Zval::Long(id as i64))
}

/// `__inflate_init($encoding, $options)` → context id. Note the legacy wording
/// of the encoding error (unlike deflate_init's ZPP-style message).
pub fn __inflate_init(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let encoding = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    if !matches!(encoding, ENC_RAW | ENC_DEFLATE | ENC_GZIP) {
        return Err(PhpError::ValueError(
            "Encoding mode must be ZLIB_ENCODING_RAW, ZLIB_ENCODING_GZIP or ZLIB_ENCODING_DEFLATE".to_string(),
        ));
    }
    let opts = opts_table(argv, "inflate_init")?;
    let opts = &opts;
    let window = int_option(opts, b"window", 15, ctx);
    if !(8..=15).contains(&window) {
        return Err(PhpError::ValueError(
            "inflate_init(): \"window\" option must be between 8 and 15".to_string(),
        ));
    }
    let dict = dict_option(opts, "inflate_init")?;
    let Some(zc) = php_types::zlibio::ZCtx::new_inflate(window_bits_for(encoding, window), dict)
    else {
        return Ok(Zval::Bool(false));
    };
    let id = ZCTXS.with(|t| {
        let mut t = t.borrow_mut();
        t.push(Some(zc));
        t.len() - 1
    });
    Ok(Zval::Long(id as i64))
}

/// The flush-mode validation shared by deflate_add/inflate_add.
fn flush_arg(argv: &[Zval], idx: usize, default: i64, ctx: &mut Ctx, fname: &str) -> Result<i32, PhpError> {
    let f = argv.get(idx).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(default);
    if !(0..=5).contains(&f) {
        return Err(PhpError::ValueError(format!(
            "{fname}(): Argument #3 ($flush_mode) must be one of ZLIB_NO_FLUSH, ZLIB_PARTIAL_FLUSH, ZLIB_SYNC_FLUSH, ZLIB_FULL_FLUSH, ZLIB_BLOCK, or ZLIB_FINISH"
        )));
    }
    Ok(f as i32)
}

/// Run `add` on the context with `id`, mapping a hard zlib failure to `false`
/// + a branded Warning (PHP distinguishes the dictionary-mismatch case).
fn ctx_add(id: i64, data: &[u8], flush: i32, ctx: &mut Ctx, fname: &str) -> Result<Zval, PhpError> {
    let out = ZCTXS.with(|t| {
        let mut t = t.borrow_mut();
        t.get_mut(id as usize).and_then(|s| s.as_mut()).map(|zc| zc.add(data, flush))
    });
    match out {
        Some(Ok(bytes)) => Ok(Zval::Str(PhpStr::new(bytes))),
        Some(Err(php_types::zlibio::ZErr::DictMismatch)) => {
            ctx.diags.push(Diag::Warning(format!(
                "{fname}(): Dictionary does not match expected dictionary (incorrect adler32 hash)"
            )));
            Ok(Zval::Bool(false))
        }
        Some(Err(php_types::zlibio::ZErr::Data)) => {
            ctx.diags.push(Diag::Warning(format!("{fname}(): data error")));
            Ok(Zval::Bool(false))
        }
        None => Ok(Zval::Bool(false)), // unknown/dropped context id
    }
}

/// `__deflate_add($id, $data, $flush_mode)`.
pub fn __deflate_add(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1);
    let data = bytes_arg(argv, 1, ctx);
    let flush = flush_arg(argv, 2, 2, ctx, "deflate_add")?; // default ZLIB_SYNC_FLUSH
    ctx_add(id, &data, flush, ctx, "deflate_add")
}

/// `__inflate_add($id, $data, $flush_mode)`.
pub fn __inflate_add(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1);
    let data = bytes_arg(argv, 1, ctx);
    let flush = flush_arg(argv, 2, 2, ctx, "inflate_add")?;
    ctx_add(id, &data, flush, ctx, "inflate_add")
}

/// `__inflate_get_status($id)` — the context's last zlib status code.
pub fn __inflate_get_status(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1);
    let st = ZCTXS.with(|t| {
        t.borrow().get(id as usize).and_then(|s| s.as_ref().map(|z| z.last_status()))
    });
    Ok(st.map(|s| Zval::Long(s as i64)).unwrap_or(Zval::Bool(false)))
}

/// `__inflate_get_read_len($id)` — total input bytes the context consumed.
pub fn __inflate_get_read_len(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(-1);
    let n = ZCTXS.with(|t| {
        t.borrow().get(id as usize).and_then(|s| s.as_ref().map(|z| z.total_in()))
    });
    Ok(n.map(Zval::Long).unwrap_or(Zval::Bool(false)))
}
