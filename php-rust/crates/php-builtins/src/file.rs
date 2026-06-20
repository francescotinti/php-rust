//! File / stream builtins (step 51). These operate on the shared
//! `Rc<RefCell<Resource>>` carried by a `Zval::Resource` argument, so they need
//! no evaluator state and are plain by-value builtins. `fopen` itself is
//! evaluator-dispatched (it owns the resource-id counter, D-51.3).

use std::cell::RefCell;
use std::io::SeekFrom;
use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Diag, PhpError, PhpStr, ResKind, Resource, StreamBackend, Zval};

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
        let len = convert::to_long_cast(len_arg, ctx.diags);
        if len >= 0 && (len as usize) < bytes.len() {
            bytes = &bytes[..len as usize];
        }
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
