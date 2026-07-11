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
    /// A `proc_open` process handle. gettype "resource"; var_dump "process".
    Process(ProcHandle),
    /// A directory handle from `opendir` (step 53c). PHP models these as php_stream
    /// too, so they report the same "resource"/"stream" labels as a byte stream.
    Dir(DirHandle),
    /// A `stream_context_create` context: carries its options array
    /// (`['http'=>[...], 'ssl'=>[...]]`) for the stream functions to read.
    /// Reports gettype "resource" and var_dump type "stream-context".
    Context(crate::Zval),
    /// A stream opened through a userland wrapper (`stream_wrapper_register`):
    /// the file ops dispatch to the wrapper object's `stream_*` methods. Reports
    /// gettype "resource" and var_dump type "stream" like any byte stream.
    UserStream(UserStream),
    /// A `stream_filter_append` handle: identifies one attached filter on its
    /// stream, for `stream_filter_remove`.
    Filter {
        stream: std::rc::Rc<std::cell::RefCell<Resource>>,
        filter_id: u32,
    },
    /// After `fclose`: the handle keeps its id but the backend is gone. PHP
    /// reports `gettype` "resource (closed)" and var_dumps "of type (Unknown)".
    Closed,
}

/// One attached stream filter (`stream_filter_append`): a transform applied to
/// bytes flowing through the stream. The zlib pair wraps an incremental
/// [`crate::zlibio::ZCtx`]; the base64 pair buffers partial groups between
/// calls. PHP flush semantics: per-write steps run `Z_NO_FLUSH` (zlib buffers
/// internally), an explicit flush/seek runs `Z_SYNC_FLUSH`, and close/removal
/// runs `Z_FINISH` (the base64 filters only act on grouping).
pub enum StreamFilter {
    ZlibDeflate(crate::zlibio::ZCtx),
    ZlibInflate(crate::zlibio::ZCtx),
    Base64Enc(Vec<u8>),
    Base64Dec(Vec<u8>),
}

impl std::fmt::Debug for StreamFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StreamFilter::ZlibDeflate(_) => "zlib.deflate",
            StreamFilter::ZlibInflate(_) => "zlib.inflate",
            StreamFilter::Base64Enc(_) => "convert.base64-encode",
            StreamFilter::Base64Dec(_) => "convert.base64-decode",
        })
    }
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_val(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Encode every full 3-byte group of `pending`, leaving the remainder in place.
fn b64_encode_step(pending: &mut Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    let full = pending.len() / 3 * 3;
    for c in pending[..full].chunks_exact(3) {
        let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | c[2] as u32;
        out.extend_from_slice(&[
            B64[(n >> 18) as usize & 63],
            B64[(n >> 12) as usize & 63],
            B64[(n >> 6) as usize & 63],
            B64[n as usize & 63],
        ]);
    }
    pending.drain(..full);
    out
}

/// Encode the final partial group (with `=` padding), emptying `pending`.
fn b64_encode_tail(pending: &mut Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    match pending.len() {
        1 => {
            let n = (pending[0] as u32) << 16;
            out.extend_from_slice(&[B64[(n >> 18) as usize & 63], B64[(n >> 12) as usize & 63], b'=', b'=']);
        }
        2 => {
            let n = ((pending[0] as u32) << 16) | ((pending[1] as u32) << 8);
            out.extend_from_slice(&[
                B64[(n >> 18) as usize & 63],
                B64[(n >> 12) as usize & 63],
                B64[(n >> 6) as usize & 63],
                b'=',
            ]);
        }
        _ => {}
    }
    pending.clear();
    out
}

/// Decode every full 4-char group of `pending`, leaving the remainder in place.
fn b64_decode_step(pending: &mut Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    let full = pending.len() / 4 * 4;
    for g in pending[..full].chunks_exact(4) {
        let n = ((g[0] as u32) << 18) | ((g[1] as u32) << 12) | ((g[2] as u32) << 6) | g[3] as u32;
        out.extend_from_slice(&[(n >> 16) as u8, (n >> 8) as u8, n as u8]);
    }
    pending.drain(..full);
    out
}

/// Decode the final partial group (2 chars → 1 byte, 3 → 2), emptying `pending`.
fn b64_decode_tail(pending: &mut Vec<u8>) -> Vec<u8> {
    let mut out = Vec::new();
    match pending.len() {
        2 => {
            let n = ((pending[0] as u32) << 18) | ((pending[1] as u32) << 12);
            out.push((n >> 16) as u8);
        }
        3 => {
            let n = ((pending[0] as u32) << 18) | ((pending[1] as u32) << 12) | ((pending[2] as u32) << 6);
            out.extend_from_slice(&[(n >> 16) as u8, (n >> 8) as u8]);
        }
        _ => {}
    }
    pending.clear();
    out
}

impl StreamFilter {
    /// Build a filter by its PHP name. `level`/`window` apply to the zlib pair
    /// (the filter's `window` is raw `windowBits`, default −15 = raw deflate).
    pub fn from_name(name: &[u8], level: i32, window: i32) -> Option<StreamFilter> {
        match name {
            b"zlib.deflate" => {
                crate::zlibio::ZCtx::new_deflate(level, window, 8, 0, None).map(StreamFilter::ZlibDeflate)
            }
            b"zlib.inflate" => crate::zlibio::ZCtx::new_inflate(window, None).map(StreamFilter::ZlibInflate),
            b"convert.base64-encode" => Some(StreamFilter::Base64Enc(Vec::new())),
            b"convert.base64-decode" => Some(StreamFilter::Base64Dec(Vec::new())),
            _ => None,
        }
    }

    /// Transform one chunk. `zflush` is the zlib flush mode for the zlib pair
    /// (`0` = Z_NO_FLUSH per ordinary write, `2` = Z_SYNC_FLUSH on flush/seek).
    fn apply(&mut self, data: &[u8], zflush: i32) -> Result<Vec<u8>, ()> {
        match self {
            StreamFilter::ZlibDeflate(z) | StreamFilter::ZlibInflate(z) => {
                z.add(data, zflush).map_err(|_| ())
            }
            StreamFilter::Base64Enc(p) => {
                p.extend_from_slice(data);
                Ok(b64_encode_step(p))
            }
            StreamFilter::Base64Dec(p) => {
                p.extend(data.iter().copied().filter(|&c| b64_val(c).is_some()).map(|c| b64_val(c).unwrap()));
                Ok(b64_decode_step(p))
            }
        }
    }

    /// Final tail when the filter is closed/removed (`Z_FINISH` / padding).
    fn finish(&mut self) -> Result<Vec<u8>, ()> {
        match self {
            StreamFilter::ZlibDeflate(z) | StreamFilter::ZlibInflate(z) => {
                z.add(&[], 4 /* Z_FINISH */).map_err(|_| ())
            }
            StreamFilter::Base64Enc(p) => Ok(b64_encode_tail(p)),
            StreamFilter::Base64Dec(p) => Ok(b64_decode_tail(p)),
        }
    }
}

/// The filters attached to a stream, per direction, plus the read-side buffer
/// of filtered-but-unconsumed bytes. Attached ids identify a filter for
/// `stream_filter_remove`.
/// One attached filter: PHP only invokes a filter when data actually flowed
/// through it, so a flush on a still-clean filter is a no-op (`dirty` tracks
/// data seen since the last drain — a fresh gzip deflate must not emit its
/// header at a pre-write rewind).
#[derive(Debug)]
pub struct AttachedFilter {
    id: u32,
    dirty: bool,
    f: StreamFilter,
}

#[derive(Default, Debug)]
pub struct FilterState {
    pub read: Vec<AttachedFilter>,
    pub write: Vec<AttachedFilter>,
    read_buf: Vec<u8>,
    read_done: bool,
    next_id: u32,
}

/// A userland-wrapper stream (`stream_wrapper_register`): the wrapper object plus
/// PHP's read-buffer state. The VM drives the `stream_*` method calls (it can
/// re-enter the interpreter, which this crate cannot), so this is pure data: the
/// `obj` handle, the `chunk` size PHP fills the buffer in (default 8192), the
/// pending `buffer`, and a sticky `eof` flag `stream_eof()` sets.
#[derive(Debug)]
pub struct UserStream {
    pub obj: crate::Zval,
    pub uri: Vec<u8>,
    pub mode: Vec<u8>,
    pub buffer: Vec<u8>,
    pub chunk: usize,
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
    /// The spec the stream was opened with (`/path/to/file`, `php://stdout`) —
    /// `stream_get_meta_data()['uri']`.
    pub uri: Vec<u8>,
    /// The `fopen` mode as given (`"r"`, `"w+b"`) — `stream_get_meta_data()['mode']`.
    pub mode: Vec<u8>,
    /// gz read streams report EOF as soon as the decoded data is exhausted (the
    /// zlib layer saw `Z_STREAM_END`), unlike ordinary streams whose EOF flag is
    /// set only by a read attempt that comes up empty. Set by `gzopen` read mode.
    pub eof_on_exhaust: bool,
    /// Attached stream filters (`stream_filter_append`); `None` for the common
    /// unfiltered stream.
    pub filters: Option<Box<FilterState>>,
}

#[derive(Debug)]
pub enum StreamBackend {
    File(std::fs::File),
    /// `php://memory` / `php://temp` (step 51b). Backed by an in-process buffer.
    Memory(Cursor<Vec<u8>>),
    /// The process's standard input — backs the predefined `STDIN` constant and
    /// `php://stdin` (step 57). Reads pull from `std::io::stdin`.
    Stdin,
    Stdout,
    Stderr,
    /// `php://output`: writes land in the program's NORMAL output stream —
    /// through the `ob_*` stack, exactly like `echo` (unlike `php://stdout`,
    /// which bypasses it). The write itself is routed by the fwrite builtin
    /// (`ctx.out`); this backend only tags the handle. Write-only.
    Output,
    /// Pipes to a `proc_open` child: `$pipes[0]` writes to the child's stdin,
    /// `$pipes[1]`/`$pipes[2]` read its stdout/stderr. Unseekable.
    ChildStdin(std::process::ChildStdin),
    ChildStdout(std::process::ChildStdout),
    ChildStderr(std::process::ChildStderr),
    /// A connected TCP client socket (`fsockopen("tcp://...")`). Unseekable.
    Tcp(std::net::TcpStream),
    /// A connected UDP socket (`fsockopen("udp://...")`): writes send one
    /// datagram each, reads receive one. Unseekable, never EOF-terminated.
    Udp(std::net::UdpSocket),
    /// A gz *write* stream (`gzopen($path, "w"/"a")` / `compress.zlib://`):
    /// writes accumulate uncompressed in `buf`; `fclose` compresses the buffer
    /// (gzip, `level`) and writes/appends it to `path`. Read mode never uses
    /// this — a gz read stream decodes up front into a `Memory` backend.
    GzFile {
        path: Vec<u8>,
        buf: Cursor<Vec<u8>>,
        level: i32,
        append: bool,
    },
}

/// A `proc_open` child process: the handle, the command line it was launched
/// with (`proc_get_status`'s `command`), and the exit code once collected —
/// PHP reports `exitcode` from cache after the first wait.
#[derive(Debug)]
pub struct ProcHandle {
    pub child: std::process::Child,
    pub command: Vec<u8>,
    pub exit_code: Option<i32>,
}

impl Resource {
    pub fn new(id: u32, stream: Stream) -> Resource {
        Resource {
            id,
            kind: ResKind::Stream(stream),
        }
    }

    /// A `stream_context_create` resource carrying its options array.
    pub fn new_context(id: u32, options: crate::Zval) -> Resource {
        Resource {
            id,
            kind: ResKind::Context(options),
        }
    }

    /// A `proc_open` process resource.
    pub fn new_process(id: u32, proc: ProcHandle) -> Resource {
        Resource {
            id,
            kind: ResKind::Process(proc),
        }
    }

    /// The process handle, if this is a `proc_open` resource.
    pub fn as_process_mut(&mut self) -> Option<&mut ProcHandle> {
        match &mut self.kind {
            ResKind::Process(p) => Some(p),
            _ => None,
        }
    }

    /// A userland-wrapper stream resource wrapping `us`.
    pub fn new_user_stream(id: u32, us: UserStream) -> Resource {
        Resource { id, kind: ResKind::UserStream(us) }
    }

    /// The userland-wrapper stream, if this resource is one.
    pub fn as_user_stream(&self) -> Option<&UserStream> {
        match &self.kind {
            ResKind::UserStream(u) => Some(u),
            _ => None,
        }
    }

    /// Mutable access to the userland-wrapper stream.
    pub fn as_user_stream_mut(&mut self) -> Option<&mut UserStream> {
        match &mut self.kind {
            ResKind::UserStream(u) => Some(u),
            _ => None,
        }
    }

    /// The context options array, if this is a stream-context resource.
    pub fn context_options(&self) -> Option<&crate::Zval> {
        match &self.kind {
            ResKind::Context(opts) => Some(opts),
            _ => None,
        }
    }

    /// Mutable access to the context options array (`stream_context_set_option`).
    pub fn context_options_mut(&mut self) -> Option<&mut crate::Zval> {
        match &mut self.kind {
            ResKind::Context(opts) => Some(opts),
            _ => None,
        }
    }

    /// `gettype` text: open resources are "resource", closed ones the special
    /// "resource (closed)" (oracle-verified, D-51.1/D-51.5).
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            ResKind::Stream(_)
            | ResKind::Dir(_)
            | ResKind::Context(_)
            | ResKind::UserStream(_)
            | ResKind::Filter { .. }
            | ResKind::Process(_) => "resource",
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
            ResKind::Stream(_) | ResKind::Dir(_) | ResKind::UserStream(_) => "stream",
            ResKind::Filter { .. } => "stream filter",
            ResKind::Context(_) => "stream-context",
            ResKind::Process(_) => "process",
            ResKind::Closed => "Unknown",
        }
    }

    pub fn as_stream_mut(&mut self) -> Option<&mut Stream> {
        match &mut self.kind {
            ResKind::Stream(s) => Some(s),
            ResKind::Dir(_)
            | ResKind::Context(_)
            | ResKind::UserStream(_)
            | ResKind::Filter { .. }
            | ResKind::Process(_)
            | ResKind::Closed => None,
        }
    }

    /// Shared access to the byte stream, if this is a stream resource.
    pub fn as_stream_ref(&self) -> Option<&Stream> {
        match &self.kind {
            ResKind::Stream(s) => Some(s),
            _ => None,
        }
    }

    /// The directory handle, if this is an `opendir` resource (step 53c).
    pub fn as_dir_mut(&mut self) -> Option<&mut DirHandle> {
        match &mut self.kind {
            ResKind::Dir(d) => Some(d),
            ResKind::Stream(_)
            | ResKind::Context(_)
            | ResKind::UserStream(_)
            | ResKind::Filter { .. }
            | ResKind::Process(_)
            | ResKind::Closed => None,
        }
    }
}

impl Stream {
    /// One raw read from the backend into `buf` (no filters, no EOF bookkeeping).
    fn backend_read_raw(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.backend {
            StreamBackend::File(f) => f.read(buf),
            StreamBackend::Memory(c) => c.read(buf),
            StreamBackend::Stdin => std::io::stdin().read(buf),
            StreamBackend::ChildStdout(p) => p.read(buf),
            StreamBackend::ChildStderr(p) => p.read(buf),
            StreamBackend::Tcp(t) => t.read(buf),
            StreamBackend::Udp(u) => u.recv(buf),
            StreamBackend::Stdout
            | StreamBackend::Stderr
            | StreamBackend::Output
            | StreamBackend::ChildStdin(_)
            | StreamBackend::GzFile { .. } => Ok(0),
        }
    }

    // ---- stream filters (stream_filter_append) ----

    /// Attach `f` to the read or write chain — at the end (`stream_filter_append`)
    /// or the front (`stream_filter_prepend`) — returning its removal id.
    pub fn attach_filter(&mut self, write: bool, front: bool, f: StreamFilter) -> u32 {
        let fs = self.filters.get_or_insert_with(Default::default);
        let id = fs.next_id;
        fs.next_id += 1;
        let entry = AttachedFilter { id, dirty: false, f };
        let chain = if write { &mut fs.write } else { &mut fs.read };
        if front {
            chain.insert(0, entry);
        } else {
            chain.push(entry);
        }
        id
    }

    pub fn has_write_filters(&self) -> bool {
        self.filters.as_ref().is_some_and(|f| !f.write.is_empty())
    }

    pub fn has_read_filters(&self) -> bool {
        self.filters.as_ref().is_some_and(|f| !f.read.is_empty())
    }

    /// Detach the filter with `id`, finishing it; its tail (already passed
    /// through any downstream write filters) should be written to the stream by
    /// the caller. `None` if no such filter is attached.
    pub fn remove_filter(&mut self, id: u32) -> Option<Vec<u8>> {
        let fs = self.filters.as_mut()?;
        if let Some(pos) = fs.write.iter().position(|a| a.id == id) {
            let mut entry = fs.write.remove(pos);
            let mut cur = if entry.dirty { entry.f.finish().unwrap_or_default() } else { Vec::new() };
            for g in fs.write.iter_mut().skip(pos) {
                if cur.is_empty() {
                    break;
                }
                g.dirty = true;
                cur = g.f.apply(&cur, 0).unwrap_or_default();
            }
            return Some(cur);
        }
        if let Some(pos) = fs.read.iter().position(|a| a.id == id) {
            fs.read.remove(pos);
            return Some(Vec::new()); // read-side removal discards pending state
        }
        None
    }

    /// Pass `data` through the write chain (ordinary write: `Z_NO_FLUSH`).
    pub fn apply_write_filters(&mut self, data: &[u8]) -> Result<Vec<u8>, ()> {
        let Some(fs) = self.filters.as_mut() else { return Ok(data.to_vec()) };
        let mut cur = data.to_vec();
        for a in fs.write.iter_mut() {
            if !cur.is_empty() {
                a.dirty = true;
            }
            cur = a.f.apply(&cur, 0)?;
        }
        Ok(cur)
    }

    /// Drain the write chain: each filter's flush tail (`Z_SYNC_FLUSH` when
    /// `finish` is false — fflush/seek; `Z_FINISH` when true — close) is pushed
    /// through the filters after it. A still-clean filter is skipped (PHP only
    /// invokes a filter data flowed through — a fresh deflate must not emit its
    /// header at a pre-write rewind). A finishing drain detaches the chain.
    pub fn drain_write_filters(&mut self, finish: bool) -> Result<Vec<u8>, ()> {
        let Some(fs) = self.filters.as_mut() else { return Ok(Vec::new()) };
        let mut out = Vec::new();
        let n = fs.write.len();
        for i in 0..n {
            if !fs.write[i].dirty {
                continue;
            }
            let tail = if finish {
                fs.write[i].f.finish()?
            } else {
                fs.write[i].f.apply(&[], 2 /* Z_SYNC_FLUSH */)?
            };
            fs.write[i].dirty = false;
            let mut cur = tail;
            for g in fs.write.iter_mut().skip(i + 1) {
                if cur.is_empty() {
                    break;
                }
                g.dirty = true;
                cur = g.f.apply(&cur, 0)?;
            }
            out.extend_from_slice(&cur);
        }
        if finish {
            fs.write.clear();
        }
        Ok(out)
    }

    /// Serve a filtered read: pull raw chunks from the backend through the read
    /// chain into the buffer until `n` bytes (or end-of-data, which finishes the
    /// chain once). A filter data error surfaces as `InvalidData`.
    fn filtered_read(&mut self, n: usize) -> std::io::Result<Vec<u8>> {
        let data_err = || std::io::Error::new(std::io::ErrorKind::InvalidData, "zlib: data error");
        loop {
            let (buf_len, done) = {
                let fs = self.filters.as_ref().expect("filtered_read with filters");
                (fs.read_buf.len(), fs.read_done)
            };
            if buf_len >= n || done {
                break;
            }
            let mut raw = vec![0u8; 8192];
            let got = match self.backend_read_raw(&mut raw) {
                Ok(g) => g,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 0,
                Err(e) => return Err(e),
            };
            let fs = self.filters.as_mut().expect("filtered_read with filters");
            if got == 0 {
                // Source exhausted: finish each read filter once, chaining tails.
                let m = fs.read.len();
                for i in 0..m {
                    let tail = fs.read[i].f.finish().map_err(|_| data_err())?;
                    let mut cur = tail;
                    for g in fs.read.iter_mut().skip(i + 1) {
                        cur = g.f.apply(&cur, 0).map_err(|_| data_err())?;
                    }
                    fs.read_buf.extend_from_slice(&cur);
                }
                fs.read_done = true;
                break;
            }
            let mut cur = raw[..got].to_vec();
            for a in fs.read.iter_mut() {
                a.dirty = true;
                cur = a.f.apply(&cur, 0).map_err(|_| data_err())?;
            }
            fs.read_buf.extend_from_slice(&cur);
        }
        let fs = self.filters.as_mut().expect("filtered_read with filters");
        let take = n.min(fs.read_buf.len());
        let out: Vec<u8> = fs.read_buf.drain(..take).collect();
        if out.is_empty() && fs.read_done {
            self.eof = true;
        }
        Ok(out)
    }

    /// Read up to `n` bytes from the current position. Returns the bytes read
    /// (possibly fewer than `n`); sets the EOF flag when the read came up short.
    pub fn read(&mut self, n: usize) -> std::io::Result<Vec<u8>> {
        if self.has_read_filters() {
            return self.filtered_read(n);
        }
        let mut buf = vec![0u8; n];
        let mut filled = 0;
        while filled < n {
            let r = {
                let dst = &mut buf[filled..];
                self.backend_read_raw(dst)
            };
            let got = match r {
                Ok(g) => g,
                // A non-blocking descriptor with nothing buffered: return the
                // bytes so far (PHP's fread returns "" here) without EOF.
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
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
        // A filtered stream reads lines through the filter chain, byte by byte.
        if self.has_read_filters() {
            let mut out = Vec::new();
            loop {
                if matches!(max, Some(m) if out.len() >= m) {
                    break;
                }
                let b = self.filtered_read(1)?;
                if b.is_empty() {
                    break;
                }
                out.push(b[0]);
                if b[0] == b'\n' {
                    break;
                }
            }
            return Ok(if out.is_empty() { None } else { Some(out) });
        }
        let mut out = Vec::new();
        let mut one = [0u8; 1];
        loop {
            if matches!(max, Some(m) if out.len() >= m) {
                break;
            }
            let r = match &mut self.backend {
                StreamBackend::File(f) => f.read(&mut one),
                StreamBackend::Memory(c) => c.read(&mut one),
                StreamBackend::Stdin => std::io::stdin().read(&mut one),
                StreamBackend::ChildStdout(p) => p.read(&mut one),
                StreamBackend::ChildStderr(p) => p.read(&mut one),
                StreamBackend::Tcp(t) => t.read(&mut one),
                StreamBackend::Udp(u) => u.recv(&mut one),
                StreamBackend::Stdout
                | StreamBackend::Stderr
                | StreamBackend::Output
                | StreamBackend::ChildStdin(_)
                | StreamBackend::GzFile { .. } => Ok(0),
            };
            let got = match r {
                Ok(g) => g,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
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
            // Writing to STDIN is not permitted; report zero bytes written.
            StreamBackend::Stdin => Ok(0),
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
            // `php://output` writes are routed to the program's output stream
            // by the fwrite builtin (they must pass through the ob_* stack);
            // a raw backend write has nowhere ob-aware to go, so report the
            // bytes as accepted without emitting them here.
            StreamBackend::Output => Ok(data.len()),
            StreamBackend::ChildStdin(p) => {
                p.write_all(data)?;
                Ok(data.len())
            }
            StreamBackend::Tcp(t) => t.write(data),
            StreamBackend::Udp(u) => u.send(data),
            // A gz write stream accumulates plaintext; fclose compresses it.
            StreamBackend::GzFile { buf, .. } => buf.write(data),
            // The child's output ends are read-only; report zero bytes written.
            StreamBackend::ChildStdout(_) | StreamBackend::ChildStderr(_) => Ok(0),
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.backend {
            StreamBackend::File(f) => f.flush(),
            StreamBackend::Memory(c) => c.flush(),
            StreamBackend::Stdin => Ok(()),
            StreamBackend::Stdout => std::io::stdout().flush(),
            StreamBackend::Stderr => std::io::stderr().flush(),
            StreamBackend::ChildStdin(p) => p.flush(),
            StreamBackend::Tcp(t) => t.flush(),
            StreamBackend::Udp(_) => Ok(()),
            StreamBackend::Output => Ok(()),
            StreamBackend::GzFile { .. } => Ok(()), // compressed only at fclose
            StreamBackend::ChildStdout(_) | StreamBackend::ChildStderr(_) => Ok(()),
        }
    }

    /// Finalise a gz *write* stream: gzip-compress the buffered plaintext and
    /// write/append it to the target file, draining the buffer (so a second
    /// call is a no-op). Runs at `fclose` — and at request shutdown for streams
    /// a script never closed, exactly like PHP's stream destructor flush.
    pub fn finalize_gz_file(&mut self) {
        use std::os::unix::ffi::OsStrExt;
        if let StreamBackend::GzFile { path, buf, level, append } = &mut self.backend {
            if buf.get_ref().is_empty() && *append {
                return; // nothing buffered on an append handle: leave the file be
            }
            let compressed = crate::zlibio::compress(buf.get_ref(), *level, crate::zlibio::ENC_GZIP);
            buf.get_mut().clear();
            buf.set_position(0);
            let target = std::ffi::OsStr::from_bytes(path);
            let file = if *append {
                std::fs::OpenOptions::new().create(true).append(true).open(target)
            } else {
                std::fs::File::create(target)
            };
            if let Ok(mut f) = file {
                let _ = f.write_all(&compressed);
            }
            // Subsequent finalizes (shutdown after an explicit fclose ran, or a
            // duplicate close) must append-not-truncate an already-written file.
            *append = true;
        }
    }

    /// Whether the in-memory data is fully consumed — the [`Self::eof_on_exhaust`]
    /// (gz stream) EOF condition. Only a memory-backed stream can tell.
    pub fn at_end(&self) -> bool {
        match &self.backend {
            StreamBackend::Memory(c) => c.position() >= c.get_ref().len() as u64,
            _ => false,
        }
    }

    /// `fseek`: returns 0 on success, -1 on a non-seekable stream. Clears EOF.
    pub fn seek(&mut self, pos: SeekFrom) -> i64 {
        let r = match &mut self.backend {
            StreamBackend::File(f) => f.seek(pos).is_ok(),
            StreamBackend::Memory(c) => c.seek(pos).is_ok(),
            StreamBackend::GzFile { buf, .. } => buf.seek(pos).is_ok(),
            // std streams and child pipes are not seekable.
            _ => false,
        };
        if r {
            self.eof = false;
            0
        } else {
            -1
        }
    }

    /// The OS file descriptor behind this stream, when there is one — used by
    /// `stream_select` (poll) and `stream_set_blocking` (fcntl). In-process
    /// memory buffers have none.
    pub fn raw_fd(&self) -> Option<i32> {
        use std::os::unix::io::AsRawFd;
        Some(match &self.backend {
            StreamBackend::File(f) => f.as_raw_fd(),
            StreamBackend::Memory(_) | StreamBackend::GzFile { .. } | StreamBackend::Output => {
                return None
            }
            StreamBackend::Stdin => 0,
            StreamBackend::Stdout => 1,
            StreamBackend::Stderr => 2,
            StreamBackend::ChildStdin(p) => p.as_raw_fd(),
            StreamBackend::ChildStdout(p) => p.as_raw_fd(),
            StreamBackend::ChildStderr(p) => p.as_raw_fd(),
            StreamBackend::Tcp(t) => t.as_raw_fd(),
            StreamBackend::Udp(u) => u.as_raw_fd(),
        })
    }

    /// `stream_set_blocking`: toggle `O_NONBLOCK` on the underlying descriptor.
    /// A memory buffer is always "blocking-complete"; report success.
    pub fn set_blocking(&mut self, enable: bool) -> bool {
        let Some(fd) = self.raw_fd() else { return true };
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            if flags < 0 {
                return false;
            }
            let flags =
                if enable { flags & !libc::O_NONBLOCK } else { flags | libc::O_NONBLOCK };
            libc::fcntl(fd, libc::F_SETFL, flags) == 0
        }
    }

    /// `ftell`: current byte offset, or `None` if not tellable.
    pub fn tell(&mut self) -> Option<u64> {
        match &mut self.backend {
            StreamBackend::File(f) => f.stream_position().ok(),
            StreamBackend::Memory(c) => c.stream_position().ok(),
            // A gz write stream's offset is its position in the *uncompressed*
            // buffer (PHP's gztell on a write stream reports plaintext bytes).
            StreamBackend::GzFile { buf, .. } => buf.stream_position().ok(),
            // std streams and child pipes have no byte offset.
            _ => None,
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
/// `data:` / `data://` (RFC 2397) read-only streams: `data://MIME[;base64],payload`.
/// Non-base64 payloads are urldecoded (`+` → space, like PHP's wrapper); an
/// invalid base64 payload fails the open (PHP: "rfc2397: unable to decode").
/// `path` is the full URI including the `data:` prefix (kept as the stream uri).
pub fn open_data_stream(path: &[u8]) -> Option<Stream> {
    let spec = path.strip_prefix(b"data:")?;
    let spec = spec.strip_prefix(b"//").unwrap_or(spec);
    let comma = spec.iter().position(|&b| b == b',')?;
    let (meta, payload) = (&spec[..comma], &spec[comma + 1..]);
    let data = if meta.ends_with(b";base64") {
        base64_decode_strict(payload)?
    } else {
        // urldecode: %XX plus `+` → space.
        let mut out = Vec::with_capacity(payload.len());
        let mut i = 0;
        while i < payload.len() {
            match payload[i] {
                b'+' => out.push(b' '),
                b'%' if i + 2 < payload.len() => {
                    match (
                        (payload[i + 1] as char).to_digit(16),
                        (payload[i + 2] as char).to_digit(16),
                    ) {
                        (Some(h), Some(l)) => {
                            out.push((h * 16 + l) as u8);
                            i += 2;
                        }
                        _ => out.push(b'%'),
                    }
                }
                c => out.push(c),
            }
            i += 1;
        }
        out
    };
    Some(Stream {
        backend: StreamBackend::Memory(Cursor::new(data)),
        readable: true,
        writable: false,
        eof: false,
        uri: path.to_vec(),
        mode: b"rb".to_vec(),
        eof_on_exhaust: false,
        filters: None,
    })
}

/// Strict base64 (PHP's `base64_decode($s, true)`): whitespace tolerated,
/// any other out-of-alphabet byte (or data after `=` padding) fails.
fn base64_decode_strict(s: &[u8]) -> Option<Vec<u8>> {
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut out = Vec::new();
    let mut padded = false;
    for &c in s {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            b'=' => {
                padded = true;
                continue;
            }
            _ => return None,
        };
        if padded {
            return None;
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

pub fn open_php_stream(spec: &[u8], mode: &[u8]) -> Option<Stream> {
    let backend = if spec == b"memory" || spec == b"temp" || spec.starts_with(b"temp/") {
        StreamBackend::Memory(Cursor::new(Vec::new()))
    } else if spec == b"input" {
        // `php://input` is the raw request body. The CLI SAPI has none:
        // oracle-pinned, it reads as EMPTY (immediate EOF) even with data
        // piped on stdin — it is NOT an alias of `php://stdin` (mapping it to
        // the real stdin made a `Request::getContent()` test suite block
        // forever on a terminal that never closes).
        StreamBackend::Memory(Cursor::new(Vec::new()))
    } else if spec == b"stdin" {
        StreamBackend::Stdin
    } else if spec == b"stdout" {
        StreamBackend::Stdout
    } else if spec == b"stderr" {
        StreamBackend::Stderr
    } else if spec == b"output" {
        StreamBackend::Output
    } else {
        return None;
    };
    let (readable, writable) = match backend {
        StreamBackend::Stdin => (true, false),
        StreamBackend::Stdout | StreamBackend::Stderr | StreamBackend::Output => (false, true),
        // `php://memory` / `php://temp`: always readable (oracle: mode "a"
        // reads back; "r" reads the empty buffer); writable unless the mode is
        // read-only-ish — Zend's memory stream only knows a READONLY flag, set
        // for "r"/"x"/"c" (oracle-pinned matrix: r+ w w+ a a+ all write).
        StreamBackend::Memory(_) => {
            let writable =
                matches!(mode.first(), Some(b'w') | Some(b'a')) || mode.contains(&b'+');
            (true, writable)
        }
        _ => mode_caps(mode).unwrap_or((true, true)),
    };
    let mut uri = b"php://".to_vec();
    uri.extend_from_slice(spec);
    Some(Stream {
        backend,
        readable,
        writable,
        eof: false,
        uri,
        mode: mode.to_vec(),
        eof_on_exhaust: false,
        filters: None,
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
            uri: path.to_vec(),
            mode: mode.to_vec(),
            eof_on_exhaust: false,
            filters: None,
        }),
        Err(e) => {
            let m = e.to_string();
            Err(m.split(" (os error").next().unwrap_or(&m).to_string())
        }
    }
}
