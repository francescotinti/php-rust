//! File / stream builtins (steps 51–52). The stream functions operate on the
//! shared `Rc<RefCell<Resource>>` carried by a `Zval::Resource` argument; the
//! filesystem predicates/operations (step 52) are pure `std::fs` wrappers. The
//! resource-minting entry points (`fopen`/`opendir`/`tmpfile`) are
//! evaluator-dispatched (they own the resource-id counter, D-51.3).

use std::cell::RefCell;
use std::io::SeekFrom;
use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Diag, Key, PhpArray, PhpError, PhpStr, ResKind, Resource, StreamBackend, Zval};

/// Resolve the `$stream` first argument to its live resource cell, or raise the
/// PHP TypeError: a non-resource is "must be of type resource, T given", a
/// closed resource is "must be an open stream resource" (oracle-verified).
fn stream_arg<'a>(
    argv: &'a [Zval],
    fname: &str,
) -> Result<&'a Rc<RefCell<Resource>>, PhpError> {
    match argv.first() {
        Some(Zval::Resource(r)) => {
            if matches!(r.borrow().kind, ResKind::Closed) {
                Err(PhpError::TypeError(format!(
                    "{fname}(): Argument #1 ($stream) must be an open stream resource"
                )))
            } else {
                Ok(r)
            }
        }
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($stream) must be of type resource, {} given",
            other.error_type_name()
        ))),
        None => Err(PhpError::ArgumentCountError(format!(
            "{fname}() expects at least 1 argument, 0 given"
        ))),
    }
}

/// `fread($stream, $length)`: read up to `$length` bytes from the current
/// position. Short reads at EOF return fewer bytes; a read on a non-readable
/// stream returns `false` with a Notice.
pub fn fread(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fread")?;
    let Some(len_arg) = argv.get(1) else {
        return Err(PhpError::ArgumentCountError(
            "fread() expects exactly 2 arguments, 1 given".to_string(),
        ));
    };
    let len = convert::to_long_cast(len_arg, ctx.diags);
    if len < 1 {
        return Err(PhpError::ValueError(
            "fread(): Argument #2 ($length) must be greater than 0".to_string(),
        ));
    }
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    if !stream.readable {
        ctx.diags.push(Diag::Notice(format!(
            "fread(): Read of {len} bytes failed with errno=9 Bad file descriptor"
        )));
        return Ok(Zval::Bool(false));
    }
    match stream.read(len as usize) {
        Ok(bytes) => Ok(Zval::Str(PhpStr::new(bytes))),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

/// `fwrite($stream, $data, $length = null)` (alias `fputs`): write `$data` (at
/// most `$length` bytes if given) and return the byte count, or `false` with a
/// Notice on a non-writable stream.
pub fn fwrite(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fwrite")?;
    let Some(data_arg) = argv.get(1) else {
        return Err(PhpError::ArgumentCountError(
            "fwrite() expects at least 2 arguments, 1 given".to_string(),
        ));
    };
    let data = convert::to_zstr(data_arg, ctx.diags);
    let mut bytes: &[u8] = data.as_bytes();
    if let Some(len_arg) = argv.get(2) {
        // `$length` caps the write, clamped to [0, len]: a negative length
        // writes 0 bytes, an over-large one writes everything (oracle: fwrite.phpt
        // `fwrite($f,"data",-1)` → 0, `fwrite($f,"data",100000)` → 4).
        let len = convert::to_long_cast(len_arg, ctx.diags);
        let n = len.clamp(0, bytes.len() as i64) as usize;
        bytes = &bytes[..n];
    }
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    if !stream.writable {
        ctx.diags.push(Diag::Notice(format!(
            "fwrite(): Write of {} bytes failed with errno=9 Bad file descriptor",
            bytes.len()
        )));
        return Ok(Zval::Bool(false));
    }
    // `php://stdout` must land in the evaluator's output buffer (so it
    // interleaves with `echo` and is captured), not the real process stdout.
    if matches!(stream.backend, StreamBackend::Stdout) {
        ctx.out.extend_from_slice(bytes);
        return Ok(Zval::Long(bytes.len() as i64));
    }
    match stream.write(bytes) {
        Ok(n) => Ok(Zval::Long(n as i64)),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

/// `fclose($stream)`: drop the backend and mark the handle closed; the same
/// `Rc` is shared, so every alias of the resource now reads as closed.
pub fn fclose(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fclose")?;
    r.borrow_mut().kind = ResKind::Closed;
    Ok(Zval::Bool(true))
}

/// `fgets($stream, $length = null)`: read one line (up to and including `\n`),
/// to EOF, or at most `$length - 1` bytes. `false` at end-of-data.
pub fn fgets(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fgets")?;
    // PHP `fgets($f, $len)` reads at most `$len - 1` bytes (the C convention).
    let max = argv.get(1).map(|v| {
        let n = convert::to_long_cast(v, ctx.diags);
        if n > 1 {
            (n - 1) as usize
        } else {
            0
        }
    });
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    if !stream.readable {
        return Ok(Zval::Bool(false));
    }
    match stream.read_line(max) {
        Ok(Some(bytes)) => Ok(Zval::Str(PhpStr::new(bytes))),
        _ => Ok(Zval::Bool(false)),
    }
}

/// `fgetc($stream)`: read a single byte, or `false` at EOF.
pub fn fgetc(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fgetc")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    if !stream.readable {
        return Ok(Zval::Bool(false));
    }
    match stream.read(1) {
        Ok(b) if !b.is_empty() => Ok(Zval::Str(PhpStr::new(b))),
        _ => Ok(Zval::Bool(false)),
    }
}

/// `feof($stream)`: the stream's sticky EOF flag (set only by a read that hit
/// end-of-data, cleared by a seek).
pub fn feof(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "feof")?;
    let mut res = r.borrow_mut();
    let eof = res.as_stream_mut().map(|s| s.eof).unwrap_or(true);
    Ok(Zval::Bool(eof))
}

/// `fseek($stream, $offset, $whence = SEEK_SET)`: 0 on success, -1 on failure.
pub fn fseek(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fseek")?;
    let Some(off_arg) = argv.get(1) else {
        return Err(PhpError::ArgumentCountError(
            "fseek() expects at least 2 arguments, 1 given".to_string(),
        ));
    };
    let off = convert::to_long_cast(off_arg, ctx.diags);
    let whence = argv
        .get(2)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let pos = match whence {
        1 => SeekFrom::Current(off),
        2 => SeekFrom::End(off),
        // SEEK_SET (0) and any unknown whence: absolute. A negative absolute
        // offset is invalid → report failure without touching the stream.
        _ => {
            if off < 0 {
                return Ok(Zval::Long(-1));
            }
            SeekFrom::Start(off as u64)
        }
    };
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    Ok(Zval::Long(stream.seek(pos)))
}

/// `ftell($stream)`: current byte offset, or `false` if not tellable.
pub fn ftell(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "ftell")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    Ok(match stream.tell() {
        Some(p) => Zval::Long(p as i64),
        None => Zval::Bool(false),
    })
}

/// `rewind($stream)`: seek to offset 0 (also clears EOF). Returns `true`.
pub fn rewind(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "rewind")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    stream.seek(SeekFrom::Start(0));
    Ok(Zval::Bool(true))
}

/// `fflush($stream)`: flush buffered writes. Returns `true`.
pub fn fflush(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fflush")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    let _ = stream.flush();
    Ok(Zval::Bool(true))
}

/// Strip Rust's " (os error N)" suffix so the text reads like PHP's strerror.
fn strerror(e: &std::io::Error) -> String {
    let m = e.to_string();
    m.split(" (os error").next().unwrap_or(&m).to_string()
}

/// `file_get_contents($filename, $use_include_path = false, $context = null,
/// $offset = 0, $length = null)` (step 51c, pure builtin — no resource). Reads
/// the whole file, then applies `$offset`/`$length`. Missing → Warning + false.
pub fn file_get_contents(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let Some(name_arg) = argv.first() else {
        return Err(PhpError::ArgumentCountError(
            "file_get_contents() expects at least 1 argument, 0 given".to_string(),
        ));
    };
    let name = convert::to_zstr(name_arg, ctx.diags);
    let path = std::ffi::OsStr::from_bytes(name.as_bytes());
    let mut data = match std::fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "file_get_contents({}): Failed to open stream: {}",
                String::from_utf8_lossy(name.as_bytes()),
                strerror(&e)
            )));
            return Ok(Zval::Bool(false));
        }
    };
    // $offset (arg #4): positive from the start, negative from the end.
    let offset = argv
        .get(3)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let start = if offset >= 0 {
        (offset as usize).min(data.len())
    } else {
        data.len().saturating_sub((-offset) as usize)
    };
    data.drain(..start);
    // $length (arg #5): cap, when given and not null.
    if let Some(len_arg) = argv.get(4) {
        if !matches!(len_arg, Zval::Null | Zval::Undef) {
            let len = convert::to_long_cast(len_arg, ctx.diags);
            if len >= 0 && (len as usize) < data.len() {
                data.truncate(len as usize);
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(data)))
}

/// `file_put_contents($filename, $data, $flags = 0, $context = null)` (step
/// 51c). `$data` may be a string, an array (elements concatenated), or a
/// readable stream resource (drained). `FILE_APPEND` (8) appends; `LOCK_EX` is
/// accepted and ignored. Returns the byte count, or `false` + Warning.
pub fn file_put_contents(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;
    let Some(name_arg) = argv.first() else {
        return Err(PhpError::ArgumentCountError(
            "file_put_contents() expects at least 2 arguments, 0 given".to_string(),
        ));
    };
    let Some(data_arg) = argv.get(1) else {
        return Err(PhpError::ArgumentCountError(
            "file_put_contents() expects at least 2 arguments, 1 given".to_string(),
        ));
    };
    let name = convert::to_zstr(name_arg, ctx.diags);
    let bytes: Vec<u8> = match data_arg {
        Zval::Array(a) => {
            let mut v = Vec::new();
            for (_k, el) in a.iter() {
                v.extend_from_slice(convert::to_zstr(el, ctx.diags).as_bytes());
            }
            v
        }
        Zval::Resource(r) => {
            // Drain the remaining bytes of a readable stream resource.
            let mut v = Vec::new();
            if let Some(stream) = r.borrow_mut().as_stream_mut() {
                while let Ok(chunk) = stream.read(8192) {
                    if chunk.is_empty() {
                        break;
                    }
                    v.extend_from_slice(&chunk);
                }
            }
            v
        }
        other => convert::to_zstr(other, ctx.diags).as_bytes().to_vec(),
    };
    let flags = argv
        .get(2)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let append = flags & 8 != 0; // FILE_APPEND
    let path = std::ffi::OsStr::from_bytes(name.as_bytes());
    let mut opts = std::fs::OpenOptions::new();
    if append {
        opts.append(true).create(true);
    } else {
        opts.write(true).create(true).truncate(true);
    }
    let mut f = match opts.open(path) {
        Ok(f) => f,
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "file_put_contents({}): Failed to open stream: {}",
                String::from_utf8_lossy(name.as_bytes()),
                strerror(&e)
            )));
            return Ok(Zval::Bool(false));
        }
    };
    match f.write_all(&bytes) {
        Ok(()) => Ok(Zval::Long(bytes.len() as i64)),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

// ---- step 52a: path-string functions (pure, no filesystem access) ----

/// The trailing path component, after stripping trailing `/` (PHP `php_basename`
/// on Unix). With `$suffix`, the suffix is removed only when the result is
/// strictly longer than it (so `basename(".php", ".php")` stays `.php`).
fn php_basename(path: &[u8], suffix: Option<&[u8]>) -> Vec<u8> {
    let mut end = path.len();
    while end > 0 && path[end - 1] == b'/' {
        end -= 1;
    }
    let trimmed = &path[..end];
    let base = match trimmed.iter().rposition(|&c| c == b'/') {
        Some(i) => &trimmed[i + 1..],
        None => trimmed,
    };
    let mut base = base.to_vec();
    if let Some(suf) = suffix {
        if !suf.is_empty() && base.len() > suf.len() && base.ends_with(suf) {
            base.truncate(base.len() - suf.len());
        }
    }
    base
}

/// The parent directory (PHP `zend_dirname`), applied `levels` times. Empty in,
/// empty out; a path with no `/` → `.`; a single leading `/` → `/`.
fn php_dirname_once(path: &[u8]) -> Vec<u8> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut end = path.len();
    while end > 0 && path[end - 1] == b'/' {
        end -= 1;
    }
    if end == 0 {
        // The path was all slashes (e.g. "/").
        return b"/".to_vec();
    }
    let trimmed = &path[..end];
    match trimmed.iter().rposition(|&c| c == b'/') {
        None => b".".to_vec(),
        Some(0) => b"/".to_vec(),
        Some(i) => {
            // Drop the last component, then any slashes joining it ("a//b" → "a").
            let mut j = i;
            while j > 0 && trimmed[j - 1] == b'/' {
                j -= 1;
            }
            if j == 0 {
                b"/".to_vec()
            } else {
                trimmed[..j].to_vec()
            }
        }
    }
}

fn php_dirname(path: &[u8], levels: i64) -> Vec<u8> {
    let mut cur = path.to_vec();
    for _ in 0..levels.max(1) {
        cur = php_dirname_once(&cur);
    }
    cur
}

/// Split a basename into `(filename, extension)` at the last `.` — a leading
/// dot still counts (`.hidden` → filename `""`, extension `hidden`), and no dot
/// means no extension (PHP `pathinfo` semantics).
fn split_ext(base: &[u8]) -> (&[u8], Option<&[u8]>) {
    match base.iter().rposition(|&c| c == b'.') {
        Some(i) => (&base[..i], Some(&base[i + 1..])),
        None => (base, None),
    }
}

pub fn basename(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let path = convert::to_zstr(&argv[0], ctx.diags);
    let suffix = argv.get(1).map(|v| convert::to_zstr(v, ctx.diags));
    let base = php_basename(path.as_bytes(), suffix.as_ref().map(|s| s.as_bytes()));
    Ok(Zval::Str(PhpStr::new(base)))
}

pub fn dirname(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let path = convert::to_zstr(&argv[0], ctx.diags);
    let levels = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(1);
    Ok(Zval::Str(PhpStr::new(php_dirname(path.as_bytes(), levels))))
}

pub fn pathinfo(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let path = convert::to_zstr(&argv[0], ctx.diags);
    let p = path.as_bytes();
    let dir = php_dirname_once(p);
    let base = php_basename(p, None);
    let (filename, extension) = split_ext(&base);
    let flags = argv.get(1).map(|v| convert::to_long_cast(v, ctx.diags));
    // A single component flag returns that string (empty when absent).
    if let Some(f) = flags {
        let single = match f {
            1 => Some(dir.clone()),
            2 => Some(base.clone()),
            4 => Some(extension.map(|e| e.to_vec()).unwrap_or_default()),
            8 => Some(filename.to_vec()),
            _ => None, // a combination (or 0): fall through to the array form
        };
        if let Some(s) = single {
            return Ok(Zval::Str(PhpStr::new(s)));
        }
    }
    let mut arr = PhpArray::new();
    arr.insert(Key::from_bytes(b"dirname"), Zval::Str(PhpStr::new(dir)));
    arr.insert(Key::from_bytes(b"basename"), Zval::Str(PhpStr::new(base.clone())));
    if let Some(ext) = extension {
        arr.insert(Key::from_bytes(b"extension"), Zval::Str(PhpStr::new(ext.to_vec())));
    }
    arr.insert(Key::from_bytes(b"filename"), Zval::Str(PhpStr::new(filename.to_vec())));
    Ok(Zval::Array(Rc::new(arr)))
}

// ---- step 52b: existence / type predicates + realpath + cwd ----

/// The OS path for a builtin's first argument (raw bytes → `OsString`).
fn arg_os_path(argv: &[Zval], ctx: &mut Ctx) -> std::ffi::OsString {
    use std::os::unix::ffi::OsStrExt;
    let s = convert::to_zstr(&argv[0], ctx.diags);
    std::ffi::OsStr::from_bytes(s.as_bytes()).to_os_string()
}

/// `file_exists`: true if the path exists (following symlinks → a broken
/// symlink is `false`, oracle-verified).
pub fn file_exists(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(std::fs::metadata(&p).is_ok()))
}

pub fn is_file(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(
        std::fs::metadata(&p).map(|m| m.is_file()).unwrap_or(false),
    ))
}

pub fn is_dir(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(
        std::fs::metadata(&p).map(|m| m.is_dir()).unwrap_or(false),
    ))
}

/// `is_link`: true if the path itself is a symlink (no-follow), so a broken
/// symlink is still `true` (oracle-verified).
pub fn is_link(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(
        std::fs::symlink_metadata(&p)
            .map(|m| m.file_type().is_symlink())
            .unwrap_or(false),
    ))
}

/// `is_readable`: approximated as "statable" — we have no euid-aware `access(2)`
/// in std, so an existing file the process can stat reads as readable (D-52).
pub fn is_readable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(std::fs::metadata(&p).is_ok()))
}

/// `is_writable`: statable and the owner write bit set (std `readonly()`).
pub fn is_writable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(
        std::fs::metadata(&p)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false),
    ))
}

/// `is_executable`: statable and any execute bit set.
pub fn is_executable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::fs::MetadataExt;
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(
        std::fs::metadata(&p)
            .map(|m| m.mode() & 0o111 != 0)
            .unwrap_or(false),
    ))
}

/// `filetype`: the lstat-based type name (a symlink reports "link"), or `false`
/// + Warning when the path cannot be stat'd.
pub fn filetype(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::fs::FileTypeExt;
    let p = arg_os_path(argv, ctx);
    match std::fs::symlink_metadata(&p) {
        Ok(m) => {
            let ft = m.file_type();
            let name = if ft.is_symlink() {
                "link"
            } else if ft.is_dir() {
                "dir"
            } else if ft.is_file() {
                "file"
            } else if ft.is_fifo() {
                "fifo"
            } else if ft.is_char_device() {
                "char"
            } else if ft.is_block_device() {
                "block"
            } else if ft.is_socket() {
                "socket"
            } else {
                "unknown"
            };
            Ok(Zval::Str(PhpStr::from_str(name)))
        }
        Err(_) => {
            let shown = String::from_utf8_lossy(p.as_os_str().as_encoded_bytes()).into_owned();
            ctx.diags
                .push(Diag::Warning(format!("filetype(): Lstat failed for {shown}")));
            Ok(Zval::Bool(false))
        }
    }
}

/// `realpath`: the canonical absolute path (symlinks + `..` resolved), or
/// `false` if any component is missing.
pub fn realpath(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let p = arg_os_path(argv, ctx);
    match std::fs::canonicalize(&p) {
        Ok(abs) => Ok(Zval::Str(PhpStr::new(abs.as_os_str().as_bytes().to_vec()))),
        Err(_) => Ok(Zval::Bool(false)),
    }
}

/// `getcwd`: the current working directory, or `false`.
pub fn getcwd(_argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    Ok(match std::env::current_dir() {
        Ok(d) => Zval::Str(PhpStr::new(d.as_os_str().as_bytes().to_vec())),
        Err(_) => Zval::Bool(false),
    })
}

/// `chdir`: change the working directory. Process-global — safe under
/// `phpt-runner --isolate` (one process per test); cargo tests use absolute
/// paths to avoid interference (D-52).
pub fn chdir(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    Ok(Zval::Bool(std::env::set_current_dir(&p).is_ok()))
}

/// `sys_get_temp_dir`: the system temp directory, without a trailing slash.
pub fn sys_get_temp_dir(_argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let d = std::env::temp_dir();
    let mut bytes = d.as_os_str().as_bytes().to_vec();
    while bytes.len() > 1 && bytes.last() == Some(&b'/') {
        bytes.pop();
    }
    Ok(Zval::Str(PhpStr::new(bytes)))
}
