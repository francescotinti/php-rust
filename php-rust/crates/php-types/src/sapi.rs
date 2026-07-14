//! SAPI request context — the bridge between a web SAPI host (phpr -S, the
//! cli-server work-alike) and the engine/builtins.
//!
//! The host parses one HTTP request, fills a [`WebRequest`], and installs it
//! with [`set_web_request`] before running the script on this thread; the VM
//! seeds the web superglobals from it and the stream layer serves
//! `php://input` from its body. `clear_web_request` removes it afterwards
//! (the CLI SAPI never installs one, so everything keeps the CLI behaviour).
//!
//! The SAPI *name* (`php_sapi_name()` / `PHP_SAPI`) is process-global: the
//! server sets it to `cli-server` once at startup, before anything is lowered
//! (the engine folds `PHP_SAPI` at compile time).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

/// One parsed HTTP request, as the cli-server SAPI sees it.
#[derive(Debug, Default)]
pub struct WebRequest {
    /// Request method verbatim (`GET`, `POST`, …).
    pub method: Vec<u8>,
    /// Protocol version as `(major, minor)` — echoed in the response.
    pub protocol: (u8, u8),
    /// The request target verbatim (undecoded path + `?query`).
    pub request_uri: Vec<u8>,
    /// Decoded vpath of the script being run (`/index.php`); for a router run
    /// this is the decoded request path instead (oracle: SCRIPT_NAME/PHP_SELF).
    pub vpath: Vec<u8>,
    /// Decoded PATH_INFO trailing the script, if any (`/extra/path seg`).
    pub path_info: Option<Vec<u8>>,
    /// Query string verbatim (no `?`), `None` when the target has none.
    pub query_string: Option<Vec<u8>>,
    /// Request headers in wire order, original case, values verbatim.
    pub headers: Vec<(Vec<u8>, Vec<u8>)>,
    /// Raw request body (`php://input`).
    pub body: Vec<u8>,
    /// Peer address/port of the connection.
    pub remote_addr: String,
    pub remote_port: u16,
    /// The host/port the server was bound to (`-S host:port`).
    pub server_host: String,
    pub server_port: u16,
    /// Absolute document root (realpath, no trailing slash).
    pub doc_root: Vec<u8>,
    /// Absolute path of the script file being executed.
    pub script_filename: Vec<u8>,
    /// Request start (seconds since epoch, sub-second precision).
    pub request_time: f64,
}

thread_local! {
    static WEB_REQUEST: RefCell<Option<Rc<WebRequest>>> = const { RefCell::new(None) };
    /// tmp files created for this request's uploads (rfc1867): the set behind
    /// `is_uploaded_file`/`move_uploaded_file`; the host deletes the leftovers
    /// at request end.
    static UPLOADED_FILES: RefCell<Vec<Vec<u8>>> = const { RefCell::new(Vec::new()) };
}

pub fn set_web_request(req: Rc<WebRequest>) {
    WEB_REQUEST.with(|r| *r.borrow_mut() = Some(req));
}

pub fn clear_web_request() {
    WEB_REQUEST.with(|r| *r.borrow_mut() = None);
}

/// The active request on this thread (cheap clone of the `Rc`), if any.
pub fn web_request() -> Option<Rc<WebRequest>> {
    WEB_REQUEST.with(|r| r.borrow().clone())
}

/// The raw request body as `php://input` sees it: empty when no request is
/// active (CLI), and empty for a multipart POST (rfc1867 consumes the body —
/// oracle-pinned).
pub fn request_body() -> Vec<u8> {
    WEB_REQUEST.with(|r| {
        let borrow = r.borrow();
        let Some(q) = borrow.as_ref() else { return Vec::new() };
        if q.method == b"POST" {
            let multipart = q.headers.iter().any(|(k, v)| {
                k.eq_ignore_ascii_case(b"content-type")
                    && v.to_ascii_lowercase().starts_with(b"multipart/form-data")
            });
            if multipart {
                return Vec::new();
            }
        }
        q.body.clone()
    })
}

/// Register an upload tmp file for this request.
pub fn add_uploaded_file(path: Vec<u8>) {
    UPLOADED_FILES.with(|u| u.borrow_mut().push(path));
}

/// Whether `path` is one of this request's upload tmp files.
pub fn is_uploaded_file(path: &[u8]) -> bool {
    UPLOADED_FILES.with(|u| u.borrow().iter().any(|p| p == path))
}

/// Unregister (after a successful `move_uploaded_file`).
pub fn remove_uploaded_file(path: &[u8]) {
    UPLOADED_FILES.with(|u| u.borrow_mut().retain(|p| p != path));
}

/// Drain the registry, returning the tmp files still owned by the request
/// (the host deletes them — PHP removes unclaimed uploads at request end).
pub fn take_uploaded_files() -> Vec<Vec<u8>> {
    UPLOADED_FILES.with(|u| std::mem::take(&mut *u.borrow_mut()))
}

/// `epoch` (UTC) as the HTTP/cookie date PHP emits:
/// `Wdy, DD Mon YYYY HH:MM:SS GMT`.
pub fn http_date(epoch: i64) -> String {
    // Civil-from-days (Howard Hinnant's algorithm), all UTC.
    let days = epoch.div_euclid(86_400);
    let secs = epoch.rem_euclid(86_400);
    let (h, mi, se) = (secs / 3600, (secs % 3600) / 60, secs % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    let wd = (days + 4).rem_euclid(7); // 1970-01-01 = Thursday
    const WDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const MONS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
        WDAYS[wd as usize],
        d,
        MONS[(m - 1) as usize],
        y,
        h,
        mi,
        se
    )
}

static SAPI_NAME: OnceLock<&'static str> = OnceLock::new();

/// Set the process SAPI name once, at startup and before any lowering
/// (`PHP_SAPI` is folded at compile time). Later calls are ignored.
pub fn set_sapi_name(name: &'static str) {
    let _ = SAPI_NAME.set(name);
}

/// The SAPI name (`cli` unless the host installed another one).
pub fn sapi_name() -> &'static str {
    SAPI_NAME.get().copied().unwrap_or("cli")
}
