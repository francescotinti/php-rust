//! PHP stream resources (step 51). A `Zval::Resource` is a shared handle into a
//! `Resource`; the actual byte stream lives in [`StreamBackend`]. This module is
//! pure `std::io` data + mechanics — PHP-level policy (mode→capability mapping,
//! `false`+Warning on failure, the resource-id counter) lives in the evaluator
//! and the file builtins (D-51.2).

use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;

/// A PHP resource value. The numeric `id` is the `#N` shown by `var_dump` /
/// "Resource id #N"; `kind` is the live stream or `Closed` once `fclose` ran.
#[derive(Debug)]
pub struct Resource {
    pub id: u32,
    pub kind: ResKind,
}

#[derive(Debug)]
pub enum ResKind {
    Stream(Stream),
    /// A directory handle from `opendir` (step 53c). PHP models these as php_stream
    /// too, so they report the same "resource"/"stream" labels as a byte stream.
    Dir(DirHandle),
    /// After `fclose`: the handle keeps its id but the backend is gone. PHP
    /// reports `gettype` "resource (closed)" and var_dumps "of type (Unknown)".
    Closed,
}

/// An open directory: the entries snapshot (including `.`/`..`, in readdir order)
/// plus the cursor `readdir` advances and `rewinddir` resets (step 53c).
#[derive(Debug)]
pub struct DirHandle {
    pub entries: Vec<Vec<u8>>,
    pub pos: usize,
}

impl DirHandle {
    /// Return the next entry and advance, or `None` past the end (`readdir` → false).
    pub fn next_entry(&mut self) -> Option<&[u8]> {
        let e = self.entries.get(self.pos)?;
        self.pos += 1;
        Some(e)
    }

    /// Reset the cursor to the first entry (`rewinddir`).
    pub fn rewind(&mut self) {
        self.pos = 0;
    }
}

/// A byte stream with its capability flags and sticky EOF bit (`feof`).
#[derive(Debug)]
pub struct Stream {
    pub backend: StreamBackend,
    pub readable: bool,
    pub writable: bool,
    /// PHP's stream EOF flag: set only when a read *attempt* hits end-of-data,
    /// not merely when the position reaches the end. `feof` reads this; a seek
    /// clears it (D-51.5).
    pub eof: bool,
}

#[derive(Debug)]
pub enum StreamBackend {
    File(std::fs::File),
    /// `php://memory` / `php://temp` (step 51b). Backed by an in-process buffer.
    Memory(Cursor<Vec<u8>>),
    Stdout,
    Stderr,
}

impl Resource {
    pub fn new(id: u32, stream: Stream) -> Resource {
        Resource {
            id,
            kind: ResKind::Stream(stream),
        }
    }

    /// `gettype` text: open resources are "resource", closed ones the special
    /// "resource (closed)" (oracle-verified, D-51.1/D-51.5).
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ResKind::Stream(_) | ResKind::Dir(_) => "resource",
            ResKind::Closed => "resource (closed)",
        }
    }

    /// Whether the handle is still open. `is_resource()` is `false` once `fclose`
    /// has turned the backend into [`ResKind::Closed`] (oracle-verified).
    pub fn is_open(&self) -> bool {
        !matches!(self.kind, ResKind::Closed)
    }

    /// The `of type (...)` label inside `var_dump`: "stream" while open,
    /// "Unknown" once closed (oracle-verified, D-51.5).
    pub fn dump_type(&self) -> &'static str {
        match self.kind {
            ResKind::Stream(_) | ResKind::Dir(_) => "stream",
            ResKind::Closed => "Unknown",
        }
    }

    pub fn as_stream_mut(&mut self) -> Option<&mut Stream> {
        match &mut self.kind {
            ResKind::Stream(s) => Some(s),
            ResKind::Dir(_) | ResKind::Closed => None,
        }
    }

    /// The directory handle, if this is an `opendir` resource (step 53c).
    pub fn as_dir_mut(&mut self) -> Option<&mut DirHandle> {
        match &mut self.kind {
            ResKind::Dir(d) => Some(d),
            ResKind::Stream(_) | ResKind::Closed => None,
        }
    }
}

impl Stream {
    /// Read up to `n` bytes from the current position. Returns the bytes read
    /// (possibly fewer than `n`); sets the EOF flag when the read came up short.
    pub fn read(&mut self, n: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; n];
        let mut filled = 0;
        while filled < n {
            let got = match &mut self.backend {
                StreamBackend::File(f) => f.read(&mut buf[filled..])?,
                StreamBackend::Memory(c) => c.read(&mut buf[filled..])?,
                StreamBackend::Stdout | StreamBackend::Stderr => 0,
            };
            if got == 0 {
                self.eof = true;
                break;
            }
            filled += got;
        }
        buf.truncate(filled);
        Ok(buf)
    }

    /// Read a single line up to and including the next `\n`, to EOF, or until
    /// `max` bytes have been read (`fgets($f, $len)` caps at `$len - 1` bytes).
    /// Returns `None` at end-of-data (sets EOF), mirroring `fgets` → `false`.
    pub fn read_line(&mut self, max: Option<usize>) -> std::io::Result<Option<Vec<u8>>> {
        let mut out = Vec::new();
        let mut one = [0u8; 1];
        loop {
            if matches!(max, Some(m) if out.len() >= m) {
                break;
            }
            let got = match &mut self.backend {
                StreamBackend::File(f) => f.read(&mut one)?,
                StreamBackend::Memory(c) => c.read(&mut one)?,
                StreamBackend::Stdout | StreamBackend::Stderr => 0,
            };
            if got == 0 {
                self.eof = true;
                break;
            }
            out.push(one[0]);
            if one[0] == b'\n' {
                break;
            }
        }
        if out.is_empty() {
            Ok(None)
        } else {
            Ok(Some(out))
        }
    }

    pub fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        match &mut self.backend {
            StreamBackend::File(f) => f.write(data),
            StreamBackend::Memory(c) => c.write(data),
            StreamBackend::Stdout => {
                let mut o = std::io::stdout();
                o.write_all(data)?;
                Ok(data.len())
            }
            StreamBackend::Stderr => {
                let mut e = std::io::stderr();
                e.write_all(data)?;
                Ok(data.len())
            }
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.backend {
            StreamBackend::File(f) => f.flush(),
            StreamBackend::Memory(c) => c.flush(),
            StreamBackend::Stdout => std::io::stdout().flush(),
            StreamBackend::Stderr => std::io::stderr().flush(),
        }
    }

    /// `fseek`: returns 0 on success, -1 on a non-seekable stream. Clears EOF.
    pub fn seek(&mut self, pos: SeekFrom) -> i64 {
        let r = match &mut self.backend {
            StreamBackend::File(f) => f.seek(pos).is_ok(),
            StreamBackend::Memory(c) => c.seek(pos).is_ok(),
            StreamBackend::Stdout | StreamBackend::Stderr => false,
        };
        if r {
            self.eof = false;
            0
        } else {
            -1
        }
    }

    /// `ftell`: current byte offset, or `None` if not tellable.
    pub fn tell(&mut self) -> Option<u64> {
        match &mut self.backend {
            StreamBackend::File(f) => f.stream_position().ok(),
            StreamBackend::Memory(c) => c.stream_position().ok(),
            StreamBackend::Stdout | StreamBackend::Stderr => None,
        }
    }
}

/// Map a `fopen` mode string to `(readable, writable)`, or `None` if the leading
/// character is not a recognised mode. `+` adds the opposite direction. Shared by
/// the file builtins and stream openers (moved here from the evaluator so both
/// engines use one definition).
pub fn mode_caps(mode: &[u8]) -> Option<(bool, bool)> {
    let plus = mode.contains(&b'+');
    match mode.first() {
        Some(b'r') => Some((true, plus)),
        Some(b'w') | Some(b'a') | Some(b'x') | Some(b'c') => Some((plus, true)),
        _ => None,
    }
}

/// Open a `php://` wrapper stream (`memory`/`temp`/`stdout`/`stderr`), or `None`
/// for an unrecognised wrapper (step 51b). stdout/stderr are write-only; memory/
/// temp honour the mode (defaulting to read+write for a lenient/odd mode string).
pub fn open_php_stream(spec: &[u8], mode: &[u8]) -> Option<Stream> {
    let backend = if spec == b"memory" || spec == b"temp" || spec.starts_with(b"temp/") {
        StreamBackend::Memory(Cursor::new(Vec::new()))
    } else if spec == b"stdout" {
        StreamBackend::Stdout
    } else if spec == b"stderr" {
        StreamBackend::Stderr
    } else {
        return None;
    };
    let (readable, writable) = match backend {
        StreamBackend::Stdout | StreamBackend::Stderr => (false, true),
        _ => mode_caps(mode).unwrap_or((true, true)),
    };
    Some(Stream {
        backend,
        readable,
        writable,
        eof: false,
    })
}

/// Open a real file as a [`Stream`] per PHP `fopen` mode rules (step 51a).
/// Returns the OS error text (with Rust's " (os error N)" suffix stripped, so it
/// reads like PHP's strerror) on failure. Modes: `r`/`w`/`a`/`x`/`c` with an
/// optional `+` (adds the other direction); `b`/`t` are accepted and ignored.
pub fn open_file_stream(path: &[u8], mode: &[u8]) -> Result<Stream, String> {
    let plus = mode.contains(&b'+');
    let Some((readable, writable)) = mode_caps(mode) else {
        return Err("`mode` is not a valid mode".to_string());
    };
    let mut opts = std::fs::OpenOptions::new();
    match mode.first() {
        Some(b'r') => {
            opts.read(true).write(plus);
        }
        Some(b'w') => {
            opts.write(true).create(true).truncate(true).read(plus);
        }
        Some(b'a') => {
            opts.append(true).create(true).read(plus);
        }
        Some(b'x') => {
            opts.write(true).create_new(true).read(plus);
        }
        Some(b'c') => {
            // create + write, NO truncate, position 0 (oracle: `c` keeps content).
            opts.write(true).create(true).read(plus);
        }
        _ => unreachable!("mode_caps already rejected unrecognised modes"),
    }
    let os_path = std::ffi::OsStr::from_bytes(path);
    match opts.open(os_path) {
        Ok(f) => Ok(Stream {
            backend: StreamBackend::File(f),
            readable,
            writable,
            eof: false,
        }),
        Err(e) => {
            let m = e.to_string();
            Err(m.split(" (os error").next().unwrap_or(&m).to_string())
        }
    }
}
