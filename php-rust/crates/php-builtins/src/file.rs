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
            // Only a live byte stream qualifies — a closed handle or a directory
            // handle (ResKind::Dir, step 53c) is rejected, keeping the
            // `as_stream_mut().expect(...)` in the stream builtins sound.
            if matches!(r.borrow().kind, ResKind::Stream(_)) {
                Ok(r)
            } else {
                Err(PhpError::TypeError(format!(
                    "{fname}(): Argument #1 ($stream) must be an open stream resource"
                )))
            }
        }
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($stream) must be of type resource, {} given",
            other.type_name_for_error()
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

/// `stream_isatty($stream)`: whether the stream is connected to a terminal. The
/// three standard streams report the real process tty state (`false` when piped /
/// redirected, as in the test harness and Composer's non-interactive runs); any
/// other backend (file, memory) is never a tty.
pub fn stream_isatty(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::io::IsTerminal;
    let r = stream_arg(argv, "stream_isatty")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    let tty = match stream.backend {
        StreamBackend::Stdin => std::io::stdin().is_terminal(),
        StreamBackend::Stdout => std::io::stdout().is_terminal(),
        StreamBackend::Stderr => std::io::stderr().is_terminal(),
        _ => false,
    };
    Ok(Zval::Bool(tty))
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
    let path = std::ffi::OsStr::from_bytes(strip_file_wrapper(name.as_bytes()));
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
    let path = std::ffi::OsStr::from_bytes(strip_file_wrapper(name.as_bytes()));
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

// ---- step 55a: file() / readfile() / fpassthru() (whole-file read + output) ----

/// `file($filename, $flags = 0)`: read a file into an array of lines. Each line
/// keeps its trailing newline unless `FILE_IGNORE_NEW_LINES` (2) is set;
/// `FILE_SKIP_EMPTY_LINES` (4) drops lines that are empty (after the newline).
/// Missing file → `false` + the "Failed to open stream" Warning.
pub fn file(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    let flags = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let ignore_nl = flags & 2 != 0;
    let skip_empty = flags & 4 != 0;
    let data = match std::fs::read(&p) {
        Ok(d) => d,
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "file({}): Failed to open stream: {}",
                show_path(&p),
                strerror(&e)
            )));
            return Ok(Zval::Bool(false));
        }
    };
    let mut arr = PhpArray::new();
    let mut start = 0;
    let push_line = |arr: &mut PhpArray, raw: &[u8]| {
        // The line content with any trailing "\r\n"/"\n" removed.
        let mut end = raw.len();
        if end > 0 && raw[end - 1] == b'\n' {
            end -= 1;
            if end > 0 && raw[end - 1] == b'\r' {
                end -= 1;
            }
        }
        let stripped = &raw[..end];
        if skip_empty && stripped.is_empty() {
            return;
        }
        let stored = if ignore_nl { stripped } else { raw };
        let _ = arr.append(Zval::Str(PhpStr::new(stored.to_vec())));
    };
    for i in 0..data.len() {
        if data[i] == b'\n' {
            push_line(&mut arr, &data[start..=i]);
            start = i + 1;
        }
    }
    if start < data.len() {
        push_line(&mut arr, &data[start..]); // trailing line without a newline
    }
    Ok(Zval::Array(Rc::new(arr)))
}

/// `readfile($filename)`: write the whole file to program output and return the
/// byte count; `false` + Warning if it cannot be opened.
pub fn readfile(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::read(&p) {
        Ok(d) => {
            let n = d.len();
            ctx.out.extend_from_slice(&d);
            Ok(Zval::Long(n as i64))
        }
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "readfile({}): Failed to open stream: {}",
                show_path(&p),
                strerror(&e)
            )));
            Ok(Zval::Bool(false))
        }
    }
}

/// `fpassthru($stream)`: write the rest of the stream (from the current position)
/// to program output and return the number of bytes passed through.
pub fn fpassthru(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fpassthru")?;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    let mut total = 0usize;
    while let Ok(chunk) = stream.read(8192) {
        if chunk.is_empty() {
            break;
        }
        total += chunk.len();
        ctx.out.extend_from_slice(&chunk);
    }
    Ok(Zval::Long(total as i64))
}

// ---- step 55b: stream_get_contents / stream_copy_to_stream / ftruncate ----

/// Read the rest of a stream into a buffer (or up to `max` bytes when `max >= 0`).
fn read_remaining(stream: &mut php_types::Stream, max: i64) -> Vec<u8> {
    let mut buf = Vec::new();
    if max < 0 {
        while let Ok(chunk) = stream.read(8192) {
            if chunk.is_empty() {
                break;
            }
            buf.extend_from_slice(&chunk);
        }
    } else {
        let mut remaining = max as usize;
        while remaining > 0 {
            match stream.read(remaining.min(8192)) {
                Ok(chunk) if !chunk.is_empty() => {
                    remaining -= chunk.len();
                    buf.extend_from_slice(&chunk);
                }
                _ => break,
            }
        }
    }
    buf
}

/// `stream_get_contents($stream, $maxlength = -1, $offset = -1)`: read the rest
/// of the stream (or `$maxlength` bytes), optionally seeking to `$offset` first.
pub fn stream_get_contents(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "stream_get_contents")?;
    let maxlength = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(-1);
    let offset = argv
        .get(2)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(-1);
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    if offset >= 0 {
        stream.seek(SeekFrom::Start(offset as u64));
    }
    Ok(Zval::Str(PhpStr::new(read_remaining(stream, maxlength))))
}

/// `stream_copy_to_stream($from, $to, $length = null, $offset = 0)`: copy the
/// rest of `$from` (or `$length` bytes) into `$to`; returns the byte count.
pub fn stream_copy_to_stream(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let from = stream_arg(argv, "stream_copy_to_stream")?;
    let to = match argv.get(1) {
        Some(Zval::Resource(r)) if matches!(r.borrow().kind, ResKind::Stream(_)) => r,
        Some(_) => {
            return Err(PhpError::TypeError(
                "stream_copy_to_stream(): Argument #2 ($to) must be an open stream resource"
                    .to_string(),
            ))
        }
        None => {
            return Err(PhpError::ArgumentCountError(
                "stream_copy_to_stream() expects at least 2 arguments, 1 given".to_string(),
            ))
        }
    };
    let length = match argv.get(2) {
        Some(Zval::Null) | None => -1,
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let offset = argv
        .get(3)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    // Read fully first so `from` and `to` are never borrowed at the same time
    // (they may even be the same resource).
    let buf = {
        let mut res = from.borrow_mut();
        let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
        if offset > 0 {
            stream.seek(SeekFrom::Start(offset as u64));
        }
        read_remaining(stream, length)
    };
    let n = buf.len();
    if let Some(s) = to.borrow_mut().as_stream_mut() {
        let _ = s.write(&buf);
    }
    Ok(Zval::Long(n as i64))
}

/// `ftruncate($stream, $size)`: truncate (or zero-extend) the underlying file /
/// in-memory buffer to `$size` bytes. Returns `true` on success.
pub fn ftruncate(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "ftruncate")?;
    let size = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0)
        .max(0) as u64;
    let mut res = r.borrow_mut();
    let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
    let ok = match &mut stream.backend {
        StreamBackend::File(f) => f.set_len(size).is_ok(),
        StreamBackend::Memory(c) => {
            c.get_mut().resize(size as usize, 0);
            true
        }
        StreamBackend::Stdin | StreamBackend::Stdout | StreamBackend::Stderr => false,
    };
    Ok(Zval::Bool(ok))
}

// ---- step 55c: environment + disk space ----

/// `getenv($name = null, $local_only = false)`: the value of an environment
/// variable (string) or `false` if unset; with no argument, an array of all
/// environment variables.
pub fn getenv(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    match argv.first() {
        Some(v) => {
            let name = convert::to_zstr(v, ctx.diags);
            match std::env::var_os(std::ffi::OsStr::from_bytes(name.as_bytes())) {
                Some(val) => Ok(Zval::Str(PhpStr::new(val.as_os_str().as_bytes().to_vec()))),
                None => Ok(Zval::Bool(false)),
            }
        }
        None => {
            let mut arr = PhpArray::new();
            for (k, val) in std::env::vars_os() {
                arr.insert(
                    Key::from_bytes(k.as_os_str().as_bytes()),
                    Zval::Str(PhpStr::new(val.as_os_str().as_bytes().to_vec())),
                );
            }
            Ok(Zval::Array(Rc::new(arr)))
        }
    }
}

/// `putenv("NAME=VALUE")` sets an environment variable; `putenv("NAME")` unsets
/// it. Returns `true` (process-global; safe under per-process `--isolate`).
pub fn putenv(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let setting = convert::to_zstr(
        argv.first().ok_or_else(|| {
            PhpError::ArgumentCountError("putenv() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let bytes = setting.as_bytes();
    match bytes.iter().position(|&b| b == b'=') {
        Some(eq) => std::env::set_var(
            std::ffi::OsStr::from_bytes(&bytes[..eq]),
            std::ffi::OsStr::from_bytes(&bytes[eq + 1..]),
        ),
        None => std::env::remove_var(std::ffi::OsStr::from_bytes(bytes)),
    }
    Ok(Zval::Bool(true))
}

/// Shared body for `disk_free_space`/`disk_total_space` via `statvfs(2)`. Returns
/// the byte count as a float, or `false` if the path cannot be stat'd.
fn disk_space(argv: &[Zval], ctx: &mut Ctx, total: bool) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let p = arg_os_path(argv, ctx);
    let Ok(c) = std::ffi::CString::new(p.as_os_str().as_bytes()) else {
        return Ok(Zval::Bool(false));
    };
    let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
        return Ok(Zval::Bool(false));
    }
    let blocks = if total { st.f_blocks } else { st.f_bavail } as f64;
    Ok(Zval::Double(blocks * st.f_frsize as f64))
}

pub fn disk_free_space(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    disk_space(argv, ctx, false)
}
pub fn disk_total_space(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    disk_space(argv, ctx, true)
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

/// Strip a leading `file://` stream-wrapper, yielding the local filesystem
/// path. `file:///a/b` -> `/a/b`; `file://localhost/a/b` -> `/a/b` (the host
/// component is dropped). Inputs without the wrapper are returned unchanged.
/// Other schemes (http://, phar://, ...) are left intact for their own handling.
pub(crate) fn strip_file_wrapper(p: &[u8]) -> &[u8] {
    if let Some(rest) = p.strip_prefix(b"file://") {
        if rest.first() == Some(&b'/') {
            rest
        } else if let Some(i) = rest.iter().position(|&c| c == b'/') {
            &rest[i..]
        } else {
            rest
        }
    } else {
        p
    }
}

/// The OS path for a builtin's first argument (raw bytes → `OsString`).
fn arg_os_path(argv: &[Zval], ctx: &mut Ctx) -> std::ffi::OsString {
    use std::os::unix::ffi::OsStrExt;
    let s = convert::to_zstr(&argv[0], ctx.diags);
    std::ffi::OsStr::from_bytes(strip_file_wrapper(s.as_bytes())).to_os_string()
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

/// Run `access(2)` on the first argument with the given mode mask. PHP's
/// `is_readable`/`is_writable`/`is_executable` defer to the OS permission check
/// (euid-aware, symlink-following), so a `chmod 0` file reads as not readable
/// even to its stat-capable owner — matching this rather than the raw mode bits
/// (D-52.7, oracle-verified). A path with an interior NUL is not accessible.
fn access_ok(argv: &[Zval], ctx: &mut Ctx, mode: libc::c_int) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let p = arg_os_path(argv, ctx);
    match std::ffi::CString::new(p.as_bytes()) {
        Ok(c) => unsafe { libc::access(c.as_ptr(), mode) == 0 },
        Err(_) => false,
    }
}

/// `is_readable`: the OS grants read access (`access(2)` `R_OK`).
pub fn is_readable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(access_ok(argv, ctx, libc::R_OK)))
}

/// `is_writable`: the OS grants write access (`access(2)` `W_OK`).
pub fn is_writable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(access_ok(argv, ctx, libc::W_OK)))
}

/// `is_executable`: the OS grants execute access (`access(2)` `X_OK`).
pub fn is_executable(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(access_ok(argv, ctx, libc::X_OK)))
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

/// `clearstatcache`: a no-op returning null. Unlike PHP-C we hold no per-request
/// stat cache — every predicate / `stat` call hits the filesystem fresh — so
/// there is nothing to clear (D-52.8). The optional `$clear_realpath_cache` /
/// `$filename` arguments are accepted and ignored.
pub fn clearstatcache(_argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Null)
}

/// `get_resource_type($resource)`: the resource's type label ("stream" for our
/// file/dir streams, "Unknown" once closed) — this is exactly `dump_type`
/// (step 53b, oracle-verified).
pub fn get_resource_type(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    match argv.first() {
        Some(Zval::Resource(r)) => Ok(Zval::Str(PhpStr::from_str(r.borrow().dump_type()))),
        Some(other) => Err(PhpError::TypeError(format!(
            "get_resource_type(): Argument #1 ($resource) must be of type resource, {} given",
            other.type_name_for_error()
        ))),
        None => Err(PhpError::ArgumentCountError(
            "get_resource_type() expects exactly 1 argument, 0 given".to_string(),
        )),
    }
}

// ---- step 53d: fprintf / vfprintf (sprintf engine → stream) ----

/// Write `bytes` to a stream resource and return the byte count as a PHP int.
/// (`fprintf`/`vfprintf` report the number of bytes written, like `printf`.)
fn write_to_stream(r: &Rc<RefCell<Resource>>, bytes: &[u8]) -> Zval {
    let n = bytes.len();
    if let Some(stream) = r.borrow_mut().as_stream_mut() {
        let _ = stream.write(bytes);
    }
    Zval::Long(n as i64)
}

/// `fprintf($stream, $format, ...$args)`: format like `sprintf` and write the
/// result to the stream, returning the byte count (step 53d).
pub fn fprintf(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "fprintf() expects at least 2 arguments, {} given",
            argv.len()
        )));
    }
    let r = stream_arg(argv, "fprintf")?;
    // The sprintf engine treats slot 0 as the format; for fprintf that is argv[1].
    let rest = &argv[1..];
    let fmt = crate::format::first_format(rest, "fprintf", ctx.diags)?;
    let bytes = crate::format::format_impl(&fmt, rest, ctx.diags)?;
    Ok(write_to_stream(r, &bytes))
}

/// `vfprintf($stream, $format, $args)`: like `fprintf` but the conversion args
/// come from an array (step 53d).
pub fn vfprintf(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() != 3 {
        return Err(PhpError::ArgumentCountError(format!(
            "vfprintf() expects exactly 3 arguments, {} given",
            argv.len()
        )));
    }
    let r = stream_arg(argv, "vfprintf")?;
    let fmt = convert::to_zstr(&argv[1], ctx.diags).as_bytes().to_vec();
    let Zval::Array(a) = &argv[2] else {
        return Err(PhpError::TypeError(format!(
            "vfprintf(): Argument #3 ($values) must be of type array, {} given",
            argv[2].type_name_for_error()
        )));
    };
    // Slot 0 is the (ignored) format placeholder; the array values follow.
    let mut vals: Vec<Zval> = vec![Zval::Null];
    for (_k, v) in a.iter() {
        vals.push(v.clone());
    }
    let bytes = crate::format::format_impl(&fmt, &vals, ctx.diags)?;
    Ok(write_to_stream(r, &bytes))
}

// ---- step 54d: CSV stream I/O (fgetcsv / fputcsv) ----

/// `fputcsv($stream, $fields, $sep=",", $enclosure="\"", $escape="\\", $eol="\n")`:
/// write one CSV record and return the byte count. Emits the PHP 8.5 `$escape`
/// deprecation when that argument is omitted.
pub fn fputcsv(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fputcsv")?;
    crate::csv::maybe_escape_deprecation(ctx, "fputcsv", argv.len(), 4);
    let fields: Vec<Vec<u8>> = match argv.get(1) {
        Some(Zval::Array(a)) => a
            .iter()
            .map(|(_, v)| convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
            .collect(),
        _ => {
            return Err(PhpError::TypeError(
                "fputcsv(): Argument #2 ($fields) must be of type array".to_string(),
            ))
        }
    };
    let sep = crate::csv::first_byte(argv.get(2), ctx, b',');
    let enc = crate::csv::first_byte(argv.get(3), ctx, b'"');
    let esc = crate::csv::escape_byte(argv.get(4), ctx);
    let eol = match argv.get(5) {
        Some(v) => convert::to_zstr(v, ctx.diags).as_bytes().to_vec(),
        None => vec![b'\n'],
    };
    let mut line = crate::csv::format_csv_line(&fields, sep, enc, esc);
    line.extend_from_slice(&eol);
    let n = line.len();
    if let Some(stream) = r.borrow_mut().as_stream_mut() {
        let _ = stream.write(&line);
    }
    Ok(Zval::Long(n as i64))
}

/// `fgetcsv($stream, $length=null, $sep=",", $enclosure="\"", $escape="\\")`:
/// read one line and parse it into a fields array; `false` at end-of-file.
/// Emits the PHP 8.5 `$escape` deprecation when that argument is omitted.
pub fn fgetcsv(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = stream_arg(argv, "fgetcsv")?;
    crate::csv::maybe_escape_deprecation(ctx, "fgetcsv", argv.len(), 4);
    let sep = crate::csv::first_byte(argv.get(2), ctx, b',');
    let enc = crate::csv::first_byte(argv.get(3), ctx, b'"');
    let esc = crate::csv::escape_byte(argv.get(4), ctx);
    let line = {
        let mut res = r.borrow_mut();
        let stream = res.as_stream_mut().expect("open stream checked in stream_arg");
        match stream.read_line(None) {
            Ok(Some(l)) => l,
            _ => return Ok(Zval::Bool(false)), // EOF or read error
        }
    };
    let mut end = line.len();
    while end > 0 && matches!(line[end - 1], b'\n' | b'\r') {
        end -= 1;
    }
    Ok(crate::csv::fields_to_array(&line[..end], sep, enc, esc))
}

// ---- step 53c: directory iteration (opendir is evaluator-dispatched) ----

/// Resolve the `$dir_handle` argument to its live resource cell.
fn dir_arg<'a>(argv: &'a [Zval], fname: &str) -> Result<&'a Rc<RefCell<Resource>>, PhpError> {
    match argv.first() {
        Some(Zval::Resource(r)) => Ok(r),
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($dir_handle) must be of type resource, {} given",
            other.type_name_for_error()
        ))),
        None => Err(PhpError::ArgumentCountError(format!(
            "{fname}() expects exactly 1 argument, 0 given"
        ))),
    }
}

/// `readdir($dir_handle)`: the next entry (incl. `.`/`..`) as raw bytes, or
/// `false` past the end — so a directory entry literally named "0" still trips
/// the canonical `=== false` loop guard.
pub fn readdir(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = dir_arg(argv, "readdir")?;
    let mut res = r.borrow_mut();
    match res.as_dir_mut().and_then(|d| d.next_entry().map(|e| e.to_vec())) {
        Some(name) => Ok(Zval::Str(PhpStr::new(name))),
        None => Ok(Zval::Bool(false)),
    }
}

/// `closedir($dir_handle)`: close the handle (it becomes a closed resource).
pub fn closedir(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = dir_arg(argv, "closedir")?;
    r.borrow_mut().kind = ResKind::Closed;
    Ok(Zval::Null)
}

/// `rewinddir($dir_handle)`: reset the read cursor to the first entry.
pub fn rewinddir(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = dir_arg(argv, "rewinddir")?;
    if let Some(d) = r.borrow_mut().as_dir_mut() {
        d.rewind();
    }
    Ok(Zval::Null)
}

// ---- step 52c: stat / lstat / fstat + single-field accessors ----

/// The 13 stat fields in PHP's documented order; each value appears twice in the
/// result array — first under integer keys `0..=12`, then under these names.
const STAT_NAMES: [&[u8]; 13] = [
    b"dev", b"ino", b"mode", b"nlink", b"uid", b"gid", b"rdev", b"size", b"atime", b"mtime",
    b"ctime", b"blksize", b"blocks",
];

/// Extract the 13 stat fields from unix metadata as signed longs (PHP exposes
/// them as `int`; dev/ino/size fit i64 on the platforms we target).
fn stat_vals(m: &std::fs::Metadata) -> [i64; 13] {
    use std::os::unix::fs::MetadataExt;
    [
        m.dev() as i64,
        m.ino() as i64,
        m.mode() as i64,
        m.nlink() as i64,
        m.uid() as i64,
        m.gid() as i64,
        m.rdev() as i64,
        m.size() as i64,
        m.atime(),
        m.mtime(),
        m.ctime(),
        m.blksize() as i64,
        m.blocks() as i64,
    ]
}

/// Build the 26-element `stat` array: integer keys `0..=12` then the named keys,
/// in order (D-52.9). var_dump / array access depend on this exact ordering.
fn stat_array_from(vals: [i64; 13]) -> PhpArray {
    let mut arr = PhpArray::new();
    for (i, v) in vals.iter().enumerate() {
        arr.insert(Key::Int(i as i64), Zval::Long(*v));
    }
    for (name, v) in STAT_NAMES.iter().zip(vals.iter()) {
        arr.insert(Key::from_bytes(name), Zval::Long(*v));
    }
    arr
}

/// Push the PHP Warning a stat-family builtin raises when the path can't be
/// stat'd. `verb` is the exact phrase PHP uses ("stat failed" / "Lstat failed").
fn warn_stat_failed(ctx: &mut Ctx, p: &std::ffi::OsStr, fname: &str, verb: &str) {
    let shown = String::from_utf8_lossy(p.as_encoded_bytes()).into_owned();
    ctx.diags
        .push(Diag::Warning(format!("{fname}(): {verb} for {shown}")));
}

/// `stat`: the 26-element array, following symlinks; `false` + Warning on error.
pub fn stat(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::metadata(&p) {
        Ok(m) => Ok(Zval::Array(Rc::new(stat_array_from(stat_vals(&m))))),
        Err(_) => {
            warn_stat_failed(ctx, &p, "stat", "stat failed");
            Ok(Zval::Bool(false))
        }
    }
}

/// `lstat`: like `stat` but does not follow a final symlink (its own metadata).
pub fn lstat(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::symlink_metadata(&p) {
        Ok(m) => Ok(Zval::Array(Rc::new(stat_array_from(stat_vals(&m))))),
        Err(_) => {
            warn_stat_failed(ctx, &p, "lstat", "Lstat failed");
            Ok(Zval::Bool(false))
        }
    }
}

/// `fstat`: the stat array for the file behind a stream resource. In-memory and
/// std stream backends have no inode, so we synthesize a regular-file 0666 entry
/// carrying the buffer length as `size` and zeros elsewhere (D-52.10). A
/// directory handle (`opendir`) or a closed handle has no byte stream → `false`
/// (no panic; we cannot reconstruct the path, D-53.1).
pub fn fstat(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let r = match argv.first() {
        Some(Zval::Resource(r)) => r,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "fstat(): Argument #1 ($stream) must be of type resource, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::ArgumentCountError(
                "fstat() expects exactly 1 argument, 0 given".to_string(),
            ))
        }
    };
    let mut res = r.borrow_mut();
    let Some(stream) = res.as_stream_mut() else {
        return Ok(Zval::Bool(false)); // directory / closed handle
    };
    let vals = match &stream.backend {
        StreamBackend::File(f) => match f.metadata() {
            Ok(m) => stat_vals(&m),
            Err(_) => return Ok(Zval::Bool(false)),
        },
        StreamBackend::Memory(c) => {
            let mut v = [0i64; 13];
            v[2] = 0o100_666; // mode: regular file, rw-rw-rw-
            v[3] = 1; // nlink
            v[7] = c.get_ref().len() as i64; // size
            v
        }
        StreamBackend::Stdin | StreamBackend::Stdout | StreamBackend::Stderr => {
            let mut v = [0i64; 13];
            v[2] = 0o100_666;
            v[3] = 1;
            v
        }
    };
    Ok(Zval::Array(Rc::new(stat_array_from(vals))))
}

/// Shared body for the single-field accessors (`filesize`, `filemtime`, …):
/// follow symlinks, return the picked field as an `int`, or `false` + the
/// "%s(): stat failed for %s" Warning on error.
fn file_stat_long(
    argv: &[Zval],
    ctx: &mut Ctx,
    fname: &str,
    pick: impl Fn([i64; 13]) -> i64,
) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::metadata(&p) {
        Ok(m) => Ok(Zval::Long(pick(stat_vals(&m)))),
        Err(_) => {
            warn_stat_failed(ctx, &p, fname, "stat failed");
            Ok(Zval::Bool(false))
        }
    }
}

pub fn filesize(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "filesize", |v| v[7])
}
pub fn filemtime(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "filemtime", |v| v[9])
}
pub fn fileatime(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "fileatime", |v| v[8])
}
pub fn filectime(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "filectime", |v| v[10])
}
/// `fileperms`: the full `st_mode` (type bits included), e.g. 0100644 for a
/// regular 0644 file — matching PHP (callers mask with `& 0777` themselves).
pub fn fileperms(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "fileperms", |v| v[2])
}
pub fn fileinode(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "fileinode", |v| v[1])
}
pub fn fileowner(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "fileowner", |v| v[4])
}
pub fn filegroup(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    file_stat_long(argv, ctx, "filegroup", |v| v[5])
}

// ---- step 52d: filesystem mutators ----

/// Render a path argument's raw bytes for an error message (lossy UTF-8).
fn show_path(p: &std::ffi::OsStr) -> String {
    String::from_utf8_lossy(p.as_encoded_bytes()).into_owned()
}

/// The OS path for the `idx`-th argument (raw bytes → `OsString`).
fn os_path_at(argv: &[Zval], ctx: &mut Ctx, idx: usize) -> std::ffi::OsString {
    use std::os::unix::ffi::OsStrExt;
    let s = convert::to_zstr(&argv[idx], ctx.diags);
    std::ffi::OsStr::from_bytes(s.as_bytes()).to_os_string()
}

/// `unlink`: delete a file; `false` + "unlink(%s): %s" Warning on failure.
pub fn unlink(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::remove_file(&p) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "unlink({}): {}",
                show_path(&p),
                strerror(&e)
            )));
            Ok(Zval::Bool(false))
        }
    }
}

/// `mkdir($dir, $permissions = 0777, $recursive = false)`. The mode is applied
/// through `mkdir(2)` (kernel masks it with the umask, exactly like PHP); a
/// non-recursive create over an existing path fails with "mkdir(): File exists".
pub fn mkdir(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::fs::DirBuilderExt;
    let p = arg_os_path(argv, ctx);
    let perms = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0o777);
    let recursive = argv
        .get(2)
        .map(|v| convert::to_bool(v, ctx.diags))
        .unwrap_or(false);
    let mut b = std::fs::DirBuilder::new();
    b.recursive(recursive).mode(perms as u32);
    match b.create(&p) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags
                .push(Diag::Warning(format!("mkdir(): {}", strerror(&e))));
            Ok(Zval::Bool(false))
        }
    }
}

/// `rmdir`: remove an empty directory; "rmdir(%s): %s" Warning on failure.
pub fn rmdir(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    match std::fs::remove_dir(&p) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "rmdir({}): {}",
                show_path(&p),
                strerror(&e)
            )));
            Ok(Zval::Bool(false))
        }
    }
}

/// `rename($from, $to)`: atomic where the OS allows; overwrites an existing
/// destination (like PHP). "rename(%s,%s): %s" Warning on failure.
pub fn rename(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "rename() expects at least 2 arguments, {} given",
            argv.len()
        )));
    }
    let from = arg_os_path(argv, ctx);
    let to = os_path_at(argv, ctx, 1);
    match std::fs::rename(&from, &to) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "rename({},{}): {}",
                show_path(&from),
                show_path(&to),
                strerror(&e)
            )));
            Ok(Zval::Bool(false))
        }
    }
}

/// `copy($from, $to)`: byte copy, overwriting the destination. PHP frames the
/// failure around the source stream: "copy(%s): Failed to open stream: %s".
pub fn copy(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "copy() expects at least 2 arguments, {} given",
            argv.len()
        )));
    }
    let from = arg_os_path(argv, ctx);
    let to = os_path_at(argv, ctx, 1);
    match std::fs::copy(&from, &to) {
        Ok(_) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "copy({}): Failed to open stream: {}",
                show_path(&from),
                strerror(&e)
            )));
            Ok(Zval::Bool(false))
        }
    }
}

/// Set a path's access + modification times (seconds) via `utimes(2)`.
fn set_times(p: &std::ffi::OsStr, atime: i64, mtime: i64) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(p.as_bytes())
        .map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidInput))?;
    let tv = [
        libc::timeval {
            tv_sec: atime as libc::time_t,
            tv_usec: 0,
        },
        libc::timeval {
            tv_sec: mtime as libc::time_t,
            tv_usec: 0,
        },
    ];
    if unsafe { libc::utimes(c.as_ptr(), tv.as_ptr()) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// `touch($filename, $mtime = null, $atime = null)`: create the file if absent
/// (without truncating an existing one), then stamp its times. A null `$mtime`
/// uses now; a null `$atime` mirrors `$mtime` (PHP semantics).
pub fn touch(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let p = arg_os_path(argv, ctx);
    if let Err(e) = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(&p)
    {
        ctx.diags.push(Diag::Warning(format!(
            "touch(): Unable to create file {} because {}",
            show_path(&p),
            strerror(&e)
        )));
        return Ok(Zval::Bool(false));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mtime = match argv.get(1) {
        Some(v) if !matches!(v, Zval::Null) => convert::to_long_cast(v, ctx.diags),
        _ => now,
    };
    let atime = match argv.get(2) {
        Some(v) if !matches!(v, Zval::Null) => convert::to_long_cast(v, ctx.diags),
        _ => mtime,
    };
    let _ = set_times(p.as_os_str(), atime, mtime);
    Ok(Zval::Bool(true))
}

/// `symlink($target, $link)`: create `$link` pointing at `$target`.
pub fn symlink(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "symlink() expects exactly 2 arguments, {} given",
            argv.len()
        )));
    }
    let target = arg_os_path(argv, ctx);
    let link = os_path_at(argv, ctx, 1);
    match std::os::unix::fs::symlink(&target, &link) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags
                .push(Diag::Warning(format!("symlink(): {}", strerror(&e))));
            Ok(Zval::Bool(false))
        }
    }
}

/// `link($target, $link)`: create a hard link `$link` to `$target`.
pub fn link(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "link() expects exactly 2 arguments, {} given",
            argv.len()
        )));
    }
    let target = arg_os_path(argv, ctx);
    let link = os_path_at(argv, ctx, 1);
    match std::fs::hard_link(&target, &link) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags
                .push(Diag::Warning(format!("link(): {}", strerror(&e))));
            Ok(Zval::Bool(false))
        }
    }
}

/// `readlink`: the target a symlink points to, or `false` + Warning.
pub fn readlink(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let p = arg_os_path(argv, ctx);
    match std::fs::read_link(&p) {
        Ok(target) => Ok(Zval::Str(PhpStr::new(
            target.as_os_str().as_bytes().to_vec(),
        ))),
        Err(e) => {
            ctx.diags
                .push(Diag::Warning(format!("readlink(): {}", strerror(&e))));
            Ok(Zval::Bool(false))
        }
    }
}

/// `chmod($filename, $permissions)`: set the mode (follows symlinks, like PHP).
pub fn chmod(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::fs::PermissionsExt;
    if argv.len() < 2 {
        return Err(PhpError::ArgumentCountError(format!(
            "chmod() expects exactly 2 arguments, {} given",
            argv.len()
        )));
    }
    let p = arg_os_path(argv, ctx);
    let perms = convert::to_long_cast(&argv[1], ctx.diags) as u32;
    match std::fs::set_permissions(&p, std::fs::Permissions::from_mode(perms)) {
        Ok(()) => Ok(Zval::Bool(true)),
        Err(e) => {
            ctx.diags
                .push(Diag::Warning(format!("chmod(): {}", strerror(&e))));
            Ok(Zval::Bool(false))
        }
    }
}

// ---- step 52e: scandir / glob / tempnam ----

/// A `&OsStr` view over raw bytes (unix paths are arbitrary bytes).
fn os_from_bytes(b: &[u8]) -> &std::ffi::OsStr {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::OsStr::from_bytes(b)
}

/// `scandir($directory, $sorting_order = SCANDIR_SORT_ASCENDING)`: the entries
/// including `.`/`..`, byte-sorted ascending (0) / descending (1) / unsorted (2).
/// On failure PHP emits *two* Warnings then returns false (oracle-verified).
pub fn scandir(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    let p = arg_os_path(argv, ctx);
    let sort = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let rd = match std::fs::read_dir(&p) {
        Ok(rd) => rd,
        Err(e) => {
            ctx.diags.push(Diag::Warning(format!(
                "scandir({}): Failed to open directory: {}",
                show_path(&p),
                strerror(&e)
            )));
            ctx.diags.push(Diag::Warning(format!(
                "scandir(): (errno {}): {}",
                e.raw_os_error().unwrap_or(0),
                strerror(&e)
            )));
            return Ok(Zval::Bool(false));
        }
    };
    let mut names: Vec<Vec<u8>> = vec![b".".to_vec(), b"..".to_vec()];
    for ent in rd.flatten() {
        names.push(ent.file_name().as_os_str().as_bytes().to_vec());
    }
    match sort {
        1 => names.sort_by(|a, b| b.cmp(a)),
        2 => {} // SCANDIR_SORT_NONE: leave readdir order
        _ => names.sort(),
    }
    let mut arr = PhpArray::new();
    for (i, n) in names.into_iter().enumerate() {
        arr.insert(Key::Int(i as i64), Zval::Str(PhpStr::new(n)));
    }
    Ok(Zval::Array(Rc::new(arr)))
}

const GLOB_MARK: i64 = 8;
const GLOB_NOSORT: i64 = 32;
const GLOB_NOCHECK: i64 = 16;
const GLOB_BRACE: i64 = 128;
const GLOB_ONLYDIR: i64 = 1_073_741_824;

/// Does a pattern segment contain a glob metacharacter?
fn has_wildcard(s: &[u8]) -> bool {
    s.iter().any(|&c| c == b'*' || c == b'?' || c == b'[')
}

/// Match a `[...]` character class at the start of `p` against `ch`. Returns
/// `(matched, bytes_consumed)`, or `None` if the class has no closing `]`.
fn match_class(p: &[u8], ch: u8) -> Option<(bool, usize)> {
    let mut i = 1; // skip '['
    let negate = matches!(p.get(1), Some(b'!') | Some(b'^'));
    if negate {
        i += 1;
    }
    let start = i;
    let mut matched = false;
    while i < p.len() {
        if p[i] == b']' && i > start {
            return Some((matched ^ negate, i + 1));
        }
        if i + 2 < p.len() && p[i + 1] == b'-' && p[i + 2] != b']' {
            if ch >= p[i] && ch <= p[i + 2] {
                matched = true;
            }
            i += 3;
        } else {
            if p[i] == ch {
                matched = true;
            }
            i += 1;
        }
    }
    None
}

/// fnmatch over a single path segment: `*` (no `/`), `?`, `[...]`, literals.
fn fnmatch(p: &[u8], s: &[u8]) -> bool {
    match p.first() {
        None => s.is_empty(),
        Some(b'*') => {
            let rest = &p[1..];
            if rest.is_empty() {
                return true;
            }
            (0..=s.len()).any(|k| fnmatch(rest, &s[k..]))
        }
        Some(b'?') => !s.is_empty() && fnmatch(&p[1..], &s[1..]),
        Some(b'[') => match (s.first(), match_class(p, *s.first().unwrap_or(&0))) {
            (Some(_), Some((matched, consumed))) => matched && fnmatch(&p[consumed..], &s[1..]),
            // Malformed class → treat '[' literally.
            _ => s.first() == Some(&b'[') && fnmatch(&p[1..], &s[1..]),
        },
        Some(&c) => !s.is_empty() && s[0] == c && fnmatch(&p[1..], &s[1..]),
    }
}

/// Glob's leading-dot rule: a name beginning with `.` matches only when the
/// pattern segment also begins with a literal `.`.
fn glob_segment_match(pat: &[u8], name: &[u8]) -> bool {
    if name.first() == Some(&b'.') && pat.first() != Some(&b'.') {
        return false;
    }
    fnmatch(pat, name)
}

fn join_path(prefix: &[u8], name: &[u8]) -> Vec<u8> {
    if prefix.is_empty() {
        name.to_vec()
    } else if prefix == b"/" {
        let mut v = vec![b'/'];
        v.extend_from_slice(name);
        v
    } else {
        let mut v = prefix.to_vec();
        v.push(b'/');
        v.extend_from_slice(name);
        v
    }
}

/// Add a fully-matched path to the results, applying GLOB_ONLYDIR (keep only
/// directories) and GLOB_MARK (append `/` to a directory).
fn glob_emit(path: &[u8], flags: i64, out: &mut Vec<Vec<u8>>) {
    let is_dir = std::fs::metadata(os_from_bytes(path))
        .map(|m| m.is_dir())
        .unwrap_or(false);
    if flags & GLOB_ONLYDIR != 0 && !is_dir {
        return;
    }
    let mut p = path.to_vec();
    if flags & GLOB_MARK != 0 && is_dir && p.last() != Some(&b'/') {
        p.push(b'/');
    }
    out.push(p);
}

/// Walk the remaining `segments` from a directory `prefix`, matching each
/// wildcard segment against the live filesystem.
fn glob_rec(prefix: &[u8], segments: &[&[u8]], flags: i64, out: &mut Vec<Vec<u8>>) {
    let Some((seg, rest)) = segments.split_first() else {
        glob_emit(prefix, flags, out);
        return;
    };
    let last = rest.is_empty();
    if !has_wildcard(seg) {
        let cand = join_path(prefix, seg);
        if last {
            if std::fs::symlink_metadata(os_from_bytes(&cand)).is_ok() {
                glob_emit(&cand, flags, out);
            }
        } else {
            glob_rec(&cand, rest, flags, out);
        }
        return;
    }
    let read_path = if prefix.is_empty() {
        b".".to_vec()
    } else {
        prefix.to_vec()
    };
    if let Ok(rd) = std::fs::read_dir(os_from_bytes(&read_path)) {
        use std::os::unix::ffi::OsStrExt;
        for ent in rd.flatten() {
            let name = ent.file_name().as_os_str().as_bytes().to_vec();
            if glob_segment_match(seg, &name) {
                let cand = join_path(prefix, &name);
                if last {
                    glob_emit(&cand, flags, out);
                } else {
                    glob_rec(&cand, rest, flags, out);
                }
            }
        }
    }
}

/// Expand `{a,b,c}` alternations (GLOB_BRACE), innermost/leftmost first.
fn brace_expand(pat: &[u8]) -> Vec<Vec<u8>> {
    let Some(open) = pat.iter().position(|&c| c == b'{') else {
        return vec![pat.to_vec()];
    };
    // Matching close brace, honouring nesting.
    let mut depth = 0;
    let mut close = None;
    for (i, &c) in pat.iter().enumerate().skip(open) {
        match c {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(close) = close else {
        return vec![pat.to_vec()];
    };
    // Split the inner content on top-level commas.
    let inner = &pat[open + 1..close];
    let mut alts: Vec<&[u8]> = Vec::new();
    let (mut d, mut start) = (0, 0);
    for (i, &c) in inner.iter().enumerate() {
        match c {
            b'{' => d += 1,
            b'}' => d -= 1,
            b',' if d == 0 => {
                alts.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    alts.push(&inner[start..]);
    if alts.len() == 1 {
        // No top-level comma: not a real alternation, keep the braces literal.
        return vec![pat.to_vec()];
    }
    let pre = &pat[..open];
    let post = &pat[close + 1..];
    let mut result = Vec::new();
    for alt in alts {
        let mut combined = pre.to_vec();
        combined.extend_from_slice(alt);
        combined.extend_from_slice(post);
        result.extend(brace_expand(&combined));
    }
    result
}

/// `glob($pattern, $flags = 0)`: shell-style pattern expansion over the live
/// filesystem. Returns the matched paths (empty array on no match, unless
/// GLOB_NOCHECK). Supports `*`/`?`/`[...]` across segments plus GLOB_MARK /
/// GLOB_NOSORT / GLOB_NOCHECK / GLOB_BRACE / GLOB_ONLYDIR (D-52.11).
pub fn glob(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let pat = convert::to_zstr(&argv[0], ctx.diags).as_bytes().to_vec();
    let flags = argv
        .get(1)
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let patterns = if flags & GLOB_BRACE != 0 {
        brace_expand(&pat)
    } else {
        vec![pat.clone()]
    };
    let mut out: Vec<Vec<u8>> = Vec::new();
    for p in &patterns {
        let absolute = p.first() == Some(&b'/');
        let segments: Vec<&[u8]> = p.split(|&c| c == b'/').filter(|s| !s.is_empty()).collect();
        let start: Vec<u8> = if absolute { vec![b'/'] } else { Vec::new() };
        glob_rec(&start, &segments, flags, &mut out);
    }
    if out.is_empty() && flags & GLOB_NOCHECK != 0 {
        out = patterns;
    }
    if flags & GLOB_NOSORT == 0 {
        out.sort();
    }
    let mut arr = PhpArray::new();
    for (i, p) in out.into_iter().enumerate() {
        arr.insert(Key::Int(i as i64), Zval::Str(PhpStr::new(p)));
    }
    Ok(Zval::Array(Rc::new(arr)))
}

/// `tempnam($directory, $prefix)`: create a unique 0600 file in `$directory`
/// and return its path (canonicalized, matching PHP's realpath-resolved result
/// on macOS). `false` if no name could be created.
pub fn tempnam(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::OpenOptionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let dir = arg_os_path(argv, ctx);
    let prefix = convert::to_zstr(&argv[1], ctx.diags);
    for _ in 0..100 {
        let n = CTR.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let mut name = dir.as_os_str().as_bytes().to_vec();
        if name.last() != Some(&b'/') {
            name.push(b'/');
        }
        name.extend_from_slice(prefix.as_bytes());
        name.extend_from_slice(format!("{:x}{nanos:x}{n:x}", std::process::id()).as_bytes());
        let path = os_from_bytes(&name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(_) => {
                let created = std::path::Path::new(path);
                let resolved =
                    std::fs::canonicalize(created).unwrap_or_else(|_| created.to_path_buf());
                return Ok(Zval::Str(PhpStr::new(
                    resolved.as_os_str().as_bytes().to_vec(),
                )));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => break,
        }
    }
    Ok(Zval::Bool(false))
}
