//! PHP stream resources (step 51). A `Zval::Resource` is a shared handle into a
//! `Resource`; the actual byte stream lives in [`StreamBackend`]. This module is
//! pure `std::io` data + mechanics — PHP-level policy (mode→capability mapping,
//! `false`+Warning on failure, the resource-id counter) lives in the evaluator
//! and the file builtins (D-51.2).

use std::io::{Cursor, Read, Seek, SeekFrom, Write};

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
    /// After `fclose`: the handle keeps its id but the backend is gone. PHP
    /// reports `gettype` "resource (closed)" and var_dumps "of type (Unknown)".
    Closed,
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
            ResKind::Stream(_) => "resource",
            ResKind::Closed => "resource (closed)",
        }
    }

    /// The `of type (...)` label inside `var_dump`: "stream" while open,
    /// "Unknown" once closed (oracle-verified, D-51.5).
    pub fn dump_type(&self) -> &'static str {
        match self.kind {
            ResKind::Stream(_) => "stream",
            ResKind::Closed => "Unknown",
        }
    }

    pub fn as_stream_mut(&mut self) -> Option<&mut Stream> {
        match &mut self.kind {
            ResKind::Stream(s) => Some(s),
            ResKind::Closed => None,
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
