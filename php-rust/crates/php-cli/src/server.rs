//! `phpr -S host:port [-t docroot] [router.php]` — a work-alike of PHP's
//! built-in development web server (the cli-server SAPI).
//!
//! Faithful to `php -S` 8.5.7 (oracle-pinned by the WP-4 sapi-probe battery):
//! sequential request handling on one thread, `Connection: close` on every
//! response, the request Host echoed back, PHP script responses without
//! Content-Length, static files with the cli-server mime map and
//! `Content-Length`, the exact 404 template, and the asctime-stamped stderr
//! log (`Accepted` / `[code]: METHOD URI` / `Closing`, PHP diagnostics
//! interleaved).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::ffi::OsStrExt;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use php_types::sapi::WebRequest;

use crate::mime::MIME_TYPE_MAP;

/// The 404 page of the cli-server, byte-identical (URI interpolated).
const ERROR_PAGE_HEAD: &str = "<!doctype html><html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>404 Not Found</title><style>\nbody { background-color: #fcfcfc; color: #333333; margin: 0; padding:0; }\nh1 { font-size: 1.5em; font-weight: normal; background-color: #9999cc; min-height:2em; line-height:2em; border-bottom: 1px inset black; margin: 0; }\nh1, p { padding-left: 10px; }\ncode.url { background-color: #eeeeee; font-family:monospace; padding:0 2px;}\n</style>\n</head><body><h1>Not Found</h1><p>The requested resource <code class=\"url\">";
const ERROR_PAGE_TAIL: &str = "</code> was not found on this server.</p></body></html>";

/// PHP's reason-phrase table (main/http_status_codes.h).
fn status_reason(code: i64) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Request Entity Too Large",
        414 => "Request-URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Requested Range Not Satisfiable",
        417 => "Expectation Failed",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        511 => "Network Authentication Required",
        _ => "Unknown Status Code",
    }
}

/// The mime type for a file extension (lowercased lookup, sorted table).
fn mime_for_ext(ext: &[u8]) -> Option<&'static str> {
    let ext = String::from_utf8_lossy(&ext.to_ascii_lowercase()).into_owned();
    MIME_TYPE_MAP
        .binary_search_by(|(e, _)| (*e).cmp(ext.as_str()))
        .ok()
        .map(|i| MIME_TYPE_MAP[i].1)
}

/// Local time as asctime ("Tue Jul 14 17:24:55 2026" — day space-padded),
/// the cli-server log timestamp.
fn asctime_local() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        let t = now as libc::time_t;
        libc::localtime_r(&t, &mut tm);
    }
    const WDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{} {} {:2} {:02}:{:02}:{:02} {}",
        WDAYS[tm.tm_wday.clamp(0, 6) as usize],
        MONS[tm.tm_mon.clamp(0, 11) as usize],
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec,
        tm.tm_year + 1900
    )
}

fn log_line(msg: &str) {
    eprintln!("[{}] {}", asctime_local(), msg);
}

/// Percent-decode a URL *path* (no `+` → space — that is query-only).
fn percent_decode_path(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'%' && i + 2 < s.len() {
            let hi = (s[i + 1] as char).to_digit(16);
            let lo = (s[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

/// Normalize a decoded path: resolve `.`/`..` segments, clamping at the root
/// (the oracle serves `GET /../x` as `/x`). Preserves a trailing slash.
fn normalize_path(path: &[u8]) -> Vec<u8> {
    let trailing = path.ends_with(b"/");
    let mut segs: Vec<&[u8]> = Vec::new();
    for seg in path.split(|&b| b == b'/') {
        match seg {
            b"" | b"." => {}
            b".." => {
                segs.pop();
            }
            s => segs.push(s),
        }
    }
    let mut out = Vec::new();
    for s in &segs {
        out.push(b'/');
        out.extend_from_slice(s);
    }
    if out.is_empty() {
        out.push(b'/');
    } else if trailing {
        out.push(b'/');
    }
    out
}

/// One parsed HTTP request off the socket.
struct HttpRequest {
    method: Vec<u8>,
    target: Vec<u8>,
    protocol: (u8, u8),
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    body: Vec<u8>,
}

fn header_value<'a>(headers: &'a [(Vec<u8>, Vec<u8>)], name: &[u8]) -> Option<&'a [u8]> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_slice())
}

/// Read and parse one request (headers + Content-Length body).
fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    let mut tmp = [0u8; 8192];
    let head_end;
    loop {
        if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            head_end = p;
            break;
        }
        if buf.len() > 1024 * 1024 {
            return None;
        }
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
    let head = buf[..head_end].to_vec();
    let mut rest = buf[head_end + 4..].to_vec();
    let mut lines = head.split(|&b| b == b'\n').map(|l| l.strip_suffix(b"\r").unwrap_or(l));
    let request_line = lines.next()?;
    let mut parts = request_line.split(|&b| b == b' ');
    let method = parts.next()?.to_vec();
    let target = parts.next()?.to_vec();
    let proto = parts.next().unwrap_or(b"HTTP/1.1");
    let protocol = if proto.starts_with(b"HTTP/") && proto.len() >= 8 {
        (
            proto[5].wrapping_sub(b'0').min(9),
            proto[7].wrapping_sub(b'0').min(9),
        )
    } else {
        (1, 1)
    };
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let Some(colon) = line.iter().position(|&b| b == b':') else { continue };
        let name = line[..colon].to_vec();
        let mut value = &line[colon + 1..];
        while value.first() == Some(&b' ') || value.first() == Some(&b'\t') {
            value = &value[1..];
        }
        headers.push((name, value.to_vec()));
    }
    // Body: `Transfer-Encoding: chunked` is decoded like PHP's parser
    // (php://input carries the DE-chunked bytes; CONTENT_LENGTH stays unset
    // because the request has no Content-Length header); otherwise
    // Content-Length bytes.
    let te_chunked = header_value(&headers, b"transfer-encoding")
        .map(|v| v.to_ascii_lowercase())
        .is_some_and(|v| v.windows(7).any(|w| w == b"chunked"));
    if te_chunked {
        rest = read_chunked_body(stream, rest)?;
    } else {
        let clen: usize = header_value(&headers, b"content-length")
            .and_then(|v| String::from_utf8_lossy(v).trim().parse().ok())
            .unwrap_or(0);
        let clen = clen.min(512 * 1024 * 1024);
        while rest.len() < clen {
            let n = stream.read(&mut tmp).ok()?;
            if n == 0 {
                break;
            }
            rest.extend_from_slice(&tmp[..n]);
        }
        rest.truncate(clen);
    }
    Some(HttpRequest {
        method,
        target,
        protocol,
        headers,
        body: rest,
    })
}

/// Decode a `Transfer-Encoding: chunked` request body: `already` holds the
/// bytes read past the header block; more are pulled from the stream as
/// needed. Returns the de-chunked payload (trailers are consumed and dropped).
fn read_chunked_body(stream: &mut TcpStream, already: Vec<u8>) -> Option<Vec<u8>> {
    let mut raw = already;
    let mut pos = 0usize;
    let mut out = Vec::new();
    let mut tmp = [0u8; 8192];
    // Ensure at least `need` bytes exist past `pos`, reading as necessary.
    macro_rules! ensure {
        ($need:expr) => {
            while raw.len() - pos < $need {
                let n = stream.read(&mut tmp).ok()?;
                if n == 0 {
                    return None;
                }
                raw.extend_from_slice(&tmp[..n]);
            }
        };
    }
    loop {
        // Chunk-size line: hex[;extensions]\r\n.
        let line_end = loop {
            if let Some(p) = raw[pos..].windows(2).position(|w| w == b"\r\n") {
                break pos + p;
            }
            ensure!(raw.len() - pos + 1);
        };
        let line = &raw[pos..line_end];
        let hex_part = line.split(|&b| b == b';').next().unwrap_or(line);
        let size = usize::from_str_radix(String::from_utf8_lossy(hex_part).trim(), 16).ok()?;
        pos = line_end + 2;
        if size == 0 {
            // Trailer section ends at the first empty line; a bare CRLF right
            // here is the common no-trailer case.
            loop {
                ensure!(2);
                if let Some(p) = raw[pos..].windows(2).position(|w| w == b"\r\n") {
                    let blank = p == 0;
                    pos += p + 2;
                    if blank {
                        return Some(out);
                    }
                } else {
                    ensure!(raw.len() - pos + 1);
                }
            }
        }
        if size > 512 * 1024 * 1024 {
            return None;
        }
        ensure!(size + 2);
        out.extend_from_slice(&raw[pos..pos + size]);
        pos += size;
        if &raw[pos..pos + 2] == b"\r\n" {
            pos += 2;
        }
    }
}

/// What the docroot walk resolved the request path to.
enum Resolved {
    /// A PHP script: absolute file, its vpath, and any PATH_INFO.
    Script(PathBuf, Vec<u8>, Option<Vec<u8>>),
    /// A static file on disk.
    Static(PathBuf),
    NotFound,
}

/// Translate a decoded, normalized path against the docroot; an unresolved
/// path falls back to the DOCROOT index.php with the whole decoded path as
/// PATH_INFO (oracle-pinned: SCRIPT_NAME=/index.php, PHP_SELF=
/// /index.php/robots.txt) — this is what serves WordPress' virtual routes
/// (/robots.txt, /wp-json/) without a router script.
fn translate(docroot: &Path, path: &[u8]) -> Resolved {
    match translate_walk(docroot, path) {
        Resolved::NotFound => {
            let root_index = docroot.join("index.php");
            if root_index.is_file() {
                return Resolved::Script(root_index, b"/index.php".to_vec(), Some(path.to_vec()));
            }
            Resolved::NotFound
        }
        hit => hit,
    }
}

/// The docroot walk: longest existing file prefix wins (the remainder is
/// PATH_INFO for scripts), a directory tries `index.php` then `index.html`
/// (cli-server order).
fn translate_walk(docroot: &Path, path: &[u8]) -> Resolved {
    let rel = &path[1.min(path.len())..];
    let mut acc = docroot.to_path_buf();
    let mut vpath: Vec<u8> = Vec::new();
    let segs: Vec<&[u8]> = if rel.is_empty() {
        Vec::new()
    } else {
        rel.split(|&b| b == b'/').collect()
    };
    for (i, seg) in segs.iter().enumerate() {
        if seg.is_empty() {
            continue;
        }
        acc.push(std::ffi::OsStr::from_bytes(seg));
        vpath.push(b'/');
        vpath.extend_from_slice(seg);
        let Ok(meta) = std::fs::metadata(&acc) else { return Resolved::NotFound };
        if meta.is_file() {
            let rest: Vec<u8> = segs[i + 1..]
                .iter()
                .flat_map(|s| {
                    let mut v = vec![b'/'];
                    v.extend_from_slice(s);
                    v
                })
                .collect();
            let path_info = (!rest.is_empty()).then_some(rest);
            if vpath.to_ascii_lowercase().ends_with(b".php") {
                return Resolved::Script(acc, vpath, path_info);
            }
            return if path_info.is_none() {
                Resolved::Static(acc)
            } else {
                Resolved::NotFound
            };
        }
    }
    // Landed on a directory: try the index files.
    for idx in [&b"index.php"[..], &b"index.html"[..]] {
        let cand = acc.join(std::ffi::OsStr::from_bytes(idx));
        if cand.is_file() {
            vpath.push(b'/');
            vpath.extend_from_slice(idx);
            return if idx.ends_with(b".php") {
                Resolved::Script(cand, vpath, None)
            } else {
                Resolved::Static(cand)
            };
        }
    }
    Resolved::NotFound
}

/// The response head shared by every kind of response.
fn response_head(
    protocol: (u8, u8),
    code: i64,
    reason: &str,
    host: Option<&[u8]>,
) -> Vec<u8> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut out = Vec::with_capacity(256);
    out.extend_from_slice(
        format!("HTTP/{}.{} {} {}\r\n", protocol.0, protocol.1, code, reason).as_bytes(),
    );
    if let Some(h) = host {
        out.extend_from_slice(b"Host: ");
        out.extend_from_slice(h);
        out.extend_from_slice(b"\r\n");
    }
    out.extend_from_slice(
        format!("Date: {}\r\n", php_types::sapi::http_date(now)).as_bytes(),
    );
    out.extend_from_slice(b"Connection: close\r\n");
    out
}

struct ServerConfig {
    host: String,
    port: u16,
    docroot: PathBuf,
    router: Option<PathBuf>,
}

/// One request/response cycle on an accepted connection.
fn handle_client(
    cfg: &ServerConfig,
    registry: &php_runtime::Registry,
    stream: &mut TcpStream,
    peer: (String, u16),
) {
    let Some(req) = read_request(stream) else { return };
    let head = req.method == b"HEAD";
    let host = header_value(&req.headers, b"host").map(|v| v.to_vec());
    // A bare trailing `?` registers no QUERY_STRING (oracle-pinned).
    let (path_part, query) = match req.target.iter().position(|&b| b == b'?') {
        Some(p) if p + 1 < req.target.len() => {
            (&req.target[..p], Some(req.target[p + 1..].to_vec()))
        }
        Some(p) => (&req.target[..p], None),
        None => (&req.target[..], None),
    };
    let decoded = normalize_path(&percent_decode_path(path_part));
    let uri_label = format!(
        "{} {}",
        String::from_utf8_lossy(&req.method),
        String::from_utf8_lossy(&req.target)
    );

    // Router script first: any return value other than boolean false ends the
    // request; false falls through to the normal docroot resolution. Anything
    // the router ECHOED before returning false is NOT discarded (oracle-pinned
    // WP-10): for a PHP target it lands at the front of the script's body (the
    // oracle runs both in one request sharing the output stream); for a static
    // file / 404 page it is flushed RAW on the socket ahead of the response
    // head (yes: the oracle emits a malformed response there).
    let mut router_prefix: Vec<u8> = Vec::new();
    if let Some(router) = &cfg.router {
        let outcome = run_php(
            cfg, registry, &req, &peer, router, decoded.clone(), None, query.clone(),
        );
        match outcome {
            Some((outcome, routed)) if routed => {
                let detail = fatal_detail(&outcome);
                let code =
                    write_php_response(stream, &req, host.as_deref(), &outcome, head, b"");
                log_request(&peer, code, &uri_label, detail.as_deref());
                return;
            }
            None => {
                let code = write_bare_error(stream, &req, host.as_deref(), 500, head);
                log_request(&peer, code, &uri_label, None);
                return;
            }
            Some((outcome, _)) => {
                // returned false — fall through, keeping its output.
                router_prefix = outcome.rendered.clone();
            }
        }
    }

    match translate(&cfg.docroot, &decoded) {
        Resolved::Script(file, vpath, path_info) => {
            match run_php(cfg, registry, &req, &peer, &file, vpath, path_info, query) {
                Some((outcome, _)) => {
                    let detail = fatal_detail(&outcome);
                    let code = write_php_response(
                        stream,
                        &req,
                        host.as_deref(),
                        &outcome,
                        head,
                        &router_prefix,
                    );
                    log_request(&peer, code, &uri_label, detail.as_deref());
                }
                None => {
                    let code = write_bare_error(stream, &req, host.as_deref(), 500, head);
                    log_request(&peer, code, &uri_label, None);
                }
            }
        }
        Resolved::Static(file) => {
            let body = std::fs::read(&file).unwrap_or_default();
            if !router_prefix.is_empty() {
                let _ = stream.write_all(&router_prefix);
            }
            let mut out = response_head(req.protocol, 200, "OK", host.as_deref());
            let ext = file
                .extension()
                .map(|e| e.as_bytes().to_vec())
                .unwrap_or_default();
            if let Some(mime) = mime_for_ext(&ext) {
                out.extend_from_slice(b"Content-Type: ");
                out.extend_from_slice(mime.as_bytes());
                if mime.starts_with("text/") {
                    out.extend_from_slice(b"; charset=UTF-8");
                }
                out.extend_from_slice(b"\r\n");
            }
            out.extend_from_slice(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
            if !head {
                out.extend_from_slice(&body);
            }
            let _ = stream.write_all(&out);
            log_request(&peer, 200, &uri_label, None);
        }
        Resolved::NotFound => {
            let mut page = ERROR_PAGE_HEAD.as_bytes().to_vec();
            page.extend_from_slice(&decoded);
            page.extend_from_slice(ERROR_PAGE_TAIL.as_bytes());
            if !router_prefix.is_empty() {
                let _ = stream.write_all(&router_prefix);
            }
            let mut out = response_head(req.protocol, 404, "Not Found", host.as_deref());
            out.extend_from_slice(b"X-Powered-By: PHP/8.5.7\r\n");
            out.extend_from_slice(b"Content-Type: text/html; charset=UTF-8\r\n");
            out.extend_from_slice(format!("Content-Length: {}\r\n\r\n", page.len()).as_bytes());
            if !head {
                out.extend_from_slice(&page);
            }
            let _ = stream.write_all(&out);
            log_request(&peer, 404, &uri_label, Some("No such file or directory"));
        }
    }
}

/// The `- detail` of the request log line when the script died with a fatal:
/// the whole (multiline) fatal message, prefix stripped — oracle-pinned.
fn fatal_detail(outcome: &php_runtime::Outcome) -> Option<String> {
    if outcome.fatal.is_none() {
        return None;
    }
    let last = outcome.error_log.last()?;
    let s = String::from_utf8_lossy(last);
    Some(s.strip_prefix("PHP Fatal error:  ").unwrap_or(&s).to_string())
}

fn log_request(peer: &(String, u16), code: i64, uri: &str, detail: Option<&str>) {
    match detail {
        Some(d) => log_line(&format!("{}:{} [{}]: {} - {}", peer.0, peer.1, code, uri, d)),
        None => log_line(&format!("{}:{} [{}]: {}", peer.0, peer.1, code, uri)),
    }
}

/// Run one PHP script for this request. Returns the outcome plus whether the
/// run "handled" the request (always true for a non-router script; a router
/// returning boolean false does not). `None` = the engine failed hard
/// (lowering error or panic) — the caller sends a bare 500.
#[allow(clippy::too_many_arguments)]
fn run_php(
    cfg: &ServerConfig,
    registry: &php_runtime::Registry,
    req: &HttpRequest,
    peer: &(String, u16),
    file: &Path,
    vpath: Vec<u8>,
    path_info: Option<Vec<u8>>,
    query: Option<Vec<u8>>,
) -> Option<(php_runtime::Outcome, bool)> {
    let source = std::fs::read(file).ok()?;
    let request_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    let web = WebRequest {
        method: req.method.clone(),
        protocol: req.protocol,
        request_uri: req.target.clone(),
        vpath,
        path_info,
        query_string: query,
        headers: req.headers.clone(),
        body: req.body.clone(),
        remote_addr: peer.0.clone(),
        remote_port: peer.1,
        server_host: cfg.host.clone(),
        server_port: cfg.port,
        doc_root: cfg.docroot.as_os_str().as_bytes().to_vec(),
        script_filename: file.as_os_str().as_bytes().to_vec(),
        request_time,
    };
    php_types::sapi::set_web_request(Rc::new(web));
    let name = file.as_os_str().as_bytes().to_vec();
    let result = catch_unwind(AssertUnwindSafe(|| {
        php_runtime::run_source_with_ini(&name, &source, registry, &[])
    }));
    php_types::sapi::clear_web_request();
    // Unclaimed upload tmp files die with the request (PHP request shutdown).
    for tmp in php_types::sapi::take_uploaded_files() {
        let _ = std::fs::remove_file(std::ffi::OsStr::from_bytes(&tmp));
    }
    let outcome = match result {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            log_line(&format!("PHP Parse error: {e}"));
            return None;
        }
        Err(_) => {
            log_line("[phpr] internal error: the runtime panicked serving this request");
            return None;
        }
    };
    // Stderr log: the script's diagnostics ("PHP Warning:  …"), each stamped
    // like the oracle (continuation lines of a fatal stay unstamped).
    for entry in &outcome.error_log {
        log_line(&String::from_utf8_lossy(entry));
    }
    let routed = !matches!(outcome.return_value, php_types::Zval::Bool(false));
    Some((outcome, routed))
}

/// Write a PHP outcome as the HTTP response; returns the status code sent.
fn write_php_response(
    stream: &mut TcpStream,
    req: &HttpRequest,
    host: Option<&[u8]>,
    outcome: &php_runtime::Outcome,
    head: bool,
    body_prefix: &[u8],
) -> i64 {
    let code = outcome.response_code.unwrap_or(200);
    let reason_owned;
    let reason = match &outcome.response_reason {
        Some(r) => {
            reason_owned = String::from_utf8_lossy(r).into_owned();
            reason_owned.as_str()
        }
        None => status_reason(code),
    };
    let mut out = response_head(req.protocol, code, reason, host);
    let mut have_ctype = false;
    for line in &outcome.headers {
        if let Some(p) = line.iter().position(|&b| b == b':') {
            if line[..p].eq_ignore_ascii_case(b"content-type") {
                have_ctype = true;
            }
        }
        out.extend_from_slice(line);
        out.extend_from_slice(b"\r\n");
    }
    if !have_ctype {
        out.extend_from_slice(b"Content-type: text/html; charset=UTF-8\r\n");
    }
    out.extend_from_slice(b"\r\n");
    if !head {
        out.extend_from_slice(body_prefix);
        out.extend_from_slice(&outcome.rendered);
    }
    let _ = stream.write_all(&out);
    code
}

/// A headers-only error response for engine-level failures (bare 500).
fn write_bare_error(
    stream: &mut TcpStream,
    req: &HttpRequest,
    host: Option<&[u8]>,
    code: i64,
    head: bool,
) -> i64 {
    let mut out = response_head(req.protocol, code, status_reason(code), host);
    out.extend_from_slice(b"Content-Type: text/html; charset=UTF-8\r\nContent-Length: 0\r\n\r\n");
    let _ = stream.write_all(&out);
    let _ = head;
    code
}

/// Entry point for `phpr -S`. Parses the residual arguments (already past
/// `-S host:port`), binds, and serves forever. Only returns on a bind error.
pub fn serve(addr: &str, mut rest: std::iter::Peekable<impl Iterator<Item = std::ffi::OsString>>) -> u8 {
    // The SAPI name must be installed before ANYTHING is lowered (PHP_SAPI is
    // folded at compile time, prelude included).
    php_types::sapi::set_sapi_name("cli-server");

    let (host, port_s) = match addr.rsplit_once(':') {
        Some(hp) => hp,
        None => {
            eprintln!("Invalid address: {addr}");
            return 1;
        }
    };
    let Ok(port) = port_s.parse::<u16>() else {
        eprintln!("Invalid address: {addr}");
        return 1;
    };
    let mut docroot: Option<PathBuf> = None;
    let mut router: Option<PathBuf> = None;
    while let Some(arg) = rest.next() {
        let bytes = arg.as_os_str().as_bytes();
        if bytes == b"-t" {
            docroot = rest.next().map(PathBuf::from);
        } else if router.is_none() {
            router = Some(PathBuf::from(arg));
        }
    }
    let docroot = docroot.unwrap_or_else(|| PathBuf::from("."));
    let Ok(docroot) = std::fs::canonicalize(&docroot) else {
        eprintln!("Directory {} does not exist.", docroot.display());
        return 1;
    };
    let router = router.map(|r| std::fs::canonicalize(&r).unwrap_or(r));
    // The cli-server chdirs to the docroot (oracle-pinned: getcwd() there,
    // relative fopen resolves against it).
    let _ = std::env::set_current_dir(&docroot);

    let registry = php_builtins::registry();
    let cfg = ServerConfig {
        host: host.to_string(),
        port,
        docroot,
        router,
    };
    let listener = match TcpListener::bind((host, port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to listen on {host}:{port} (reason: {e})");
            return 1;
        }
    };
    log_line(&format!(
        "PHP 8.5.7 Development Server (http://{host}:{port}) started"
    ));
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let peer = stream
            .peer_addr()
            .map(|a| (a.ip().to_string(), a.port()))
            .unwrap_or_else(|_| ("127.0.0.1".to_string(), 0));
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(60)));
        log_line(&format!("{}:{} Accepted", peer.0, peer.1));
        handle_client(&cfg, &registry, &mut stream, peer.clone());
        let _ = stream.flush();
        log_line(&format!("{}:{} Closing", peer.0, peer.1));
    }
    0
}
