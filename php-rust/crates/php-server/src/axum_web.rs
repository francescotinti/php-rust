//! S-188 (Tappa 2): the web-SAPI bridge of the axum front end — what turns an
//! HTTP request into the same `WebRequest` the `phpr -S` cli-server installs
//! (superglobals, cookies, body, PATH_INFO) and turns the Vm's response head
//! (`header()`, `http_response_code()`, `Set-Cookie`, `Location`) back into an
//! HTTP response. Routing is the shared docroot walk of `php_types::docroot`
//! (root `index.php` fallback for WordPress' virtual routes, static files with
//! the cli-server mime map, the byte-identical 404 page).
//!
//! Thread discipline: the `WebRequest` is a THREAD-LOCAL of php-types. It is
//! installed and cleared ON THE WORKER THREAD, around the request, through
//! [`RequestScope`] (RAII: cleared on every exit path, upload tmp files
//! removed like the cli-server does at request end). The tokio dispatcher
//! never touches it, so W>1 workers cannot bleed requests into each other
//! (tooth: `two_workers_no_request_bleed`).
//!
//! Status contract under the web SAPI = the cli-server's: `response_code`
//! or 200 — a PHP fatal is 200 with the html_errors banner (oracle `php -S`
//! probe: wp188-harness/oracle-fatal-status.out), `exit()` keeps the code the
//! script set, 500 is reserved for engine-level failures. Without an installed
//! web request (in-module tests, `__census_self`) the pre-S-188 CLI-anchored
//! form holds (fatal → 500, plain banners).
//!
//! Declared residues (`TODO(port)`, PHPR_DIVERGENCES_FROM_PHP.md §3.9): header
//! names reach PHP lowercased (the http crate's `HeaderName`), hyper owns the
//! wire order and adds `content-length`/`date`, a custom reason phrase from
//! `header("HTTP/1.1 404 Custom")` is not emitted, header values the http crate
//! rejects (control bytes) are dropped with a stderr note, the router script
//! argument is not honoured in `--axum`.

use std::net::SocketAddr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::rc::Rc;

use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode, Version};
use axum::response::{IntoResponse, Response};
use php_types::docroot::{
    asctime_local, mime_for_ext, normalize_path, not_found_page, percent_decode_path,
    split_target, status_reason, translate, Resolved,
};
use php_types::sapi::WebRequest;

use crate::worker_pool::WorkerHandlerMeta;

/// Request-body cap, the cli-server's (`server.rs` read_request).
const BODY_CAP: usize = 512 * 1024 * 1024;

/// What the worker hands back for one request: the full response head.
#[derive(Debug, Default)]
pub struct WorkerResponse {
    pub status: u16,
    /// Custom reason phrase (`header("HTTP/1.1 404 Custom")`) — carried,
    /// not emitted (declared residue).
    pub reason: Option<Vec<u8>>,
    /// `Name: value` lines in send order (replace semantics already applied).
    pub headers: Vec<Vec<u8>>,
    pub body: Vec<u8>,
}

impl WorkerResponse {
    /// A head-less response (pre-S-188 shape: body + status only).
    pub fn plain(body: Vec<u8>, status: StatusCode) -> Self {
        WorkerResponse { status: status.as_u16(), reason: None, headers: Vec::new(), body }
    }

    /// The pre-S-188 `(body, status)` pair — the shape the in-module gate
    /// tests assert on.
    #[cfg(test)]
    pub fn pair(self) -> (Vec<u8>, StatusCode) {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (self.body, status)
    }

    /// From a finished [`php_runtime::Outcome`] (compile-time fatal path):
    /// under the web SAPI the status is the script's or 200 (the cli-server
    /// rule); without one a fatal keeps the CLI-anchored 500. The `log_errors`
    /// lines go to stderr stamped like the cli-server log.
    pub fn from_outcome(o: php_runtime::Outcome) -> Self {
        log_error_lines(&o.error_log);
        let web = php_types::sapi::web_request().is_some();
        let status = o.response_code.unwrap_or(if !web && o.fatal.is_some() { 500 } else { 200 });
        WorkerResponse {
            status: status.clamp(100, 999) as u16,
            reason: o.response_reason,
            headers: o.headers,
            body: o.rendered,
        }
    }
}

/// A hard parse error of the main script: the html envelope under the web
/// SAPI (200, like `php -S`), the pre-S-188 plain envelope + 500 otherwise.
/// The message text stays phpr's own (registered divergence §3.9).
pub fn parse_error(message: &str) -> WorkerResponse {
    if php_types::sapi::web_request().is_some() {
        eprintln!("[{}] PHP Parse error:  {message}", asctime_local());
        let body = format!("<br />\n<b>Parse error</b>:  {message}<br />\n").into_bytes();
        return WorkerResponse {
            status: 200,
            reason: None,
            headers: vec![b"X-Powered-By: PHP/8.5.7".to_vec()],
            body,
        };
    }
    WorkerResponse::plain(format!("\nParse error: {message}\n").into_bytes(), StatusCode::INTERNAL_SERVER_ERROR)
}

fn log_error_lines(lines: &[Vec<u8>]) {
    for entry in lines {
        eprintln!("[{}] {}", asctime_local(), String::from_utf8_lossy(entry));
    }
}

/// The cli-server request log line (`[asctime] ip:port [code]: METHOD URI`) —
/// same shape on both SAPI hosts, so the measurement harness' request-count
/// gate (wp188) reads either server log.
pub fn log_request(peer: SocketAddr, code: u16, method: &str, target: &str) {
    eprintln!("[{}] {}:{} [{}]: {} {}", asctime_local(), peer.ip(), peer.port(), code, method, target);
}

/// RAII window of the thread-local web request on the WORKER thread.
pub struct RequestScope {
    installed: bool,
}

impl RequestScope {
    /// Install `web` (if any) for the current thread. `None` leaves the CLI
    /// semantics in place (no superglobal seeding, CLI-anchored statuses).
    pub fn install(web: Option<WebRequest>) -> Self {
        match web {
            Some(w) => {
                php_types::sapi::set_web_request(Rc::new(w));
                RequestScope { installed: true }
            }
            None => RequestScope { installed: false },
        }
    }
}

impl Drop for RequestScope {
    fn drop(&mut self) {
        if !self.installed {
            return;
        }
        php_types::sapi::clear_web_request();
        // Unclaimed upload tmp files die with the request (PHP request
        // shutdown; same as the cli-server, server.rs run_php).
        for tmp in php_types::sapi::take_uploaded_files() {
            // already gone = fine (moved by move_uploaded_file or never created)
            std::fs::remove_file(std::ffi::OsStr::from_bytes(&tmp)).unwrap_or(());
        }
    }
}

/// Collect the response from a Vm that has run the script — BEFORE
/// `request_end()` (capture-before-reset, KS-PP-2). `fatal` is the worker's
/// fatal flag for the CLI-anchored status when no web request is installed.
pub fn finish(vm: &mut php_runtime::Vm<'_>, fatal: bool) -> WorkerResponse {
    let head = vm.take_web_response();
    let body = std::mem::take(&mut vm.rendered);
    log_error_lines(&head.error_log);
    let status = if head.web { head.code.unwrap_or(200) } else if fatal { 500 } else { 200 };
    WorkerResponse { status: status.clamp(100, 999) as u16, reason: head.reason, headers: head.headers, body }
}

/// The server-lifetime facts the bridge needs to fill `$_SERVER`.
pub struct ServeCfg {
    pub host: String,
    pub port: u16,
    /// Canonical docroot (realpath, the cli-server chdirs there too).
    pub docroot: PathBuf,
}

/// Where a request goes after the docroot walk.
pub enum Routed {
    /// Answered by the dispatcher: a static file or the 404 page.
    Ready(Response),
    /// A PHP script for the worker pool, with its web request.
    Script { meta: WorkerHandlerMeta, web: WebRequest },
}

fn version_pair(v: Version) -> (u8, u8) {
    match v {
        Version::HTTP_09 => (0, 9),
        Version::HTTP_10 => (1, 0),
        Version::HTTP_2 => (2, 0),
        Version::HTTP_3 => (3, 0),
        _ => (1, 1),
    }
}

/// Resolve one request against the docroot (cli-server semantics).
pub async fn route(cfg: &ServeCfg, peer: SocketAddr, req: axum::extract::Request) -> Routed {
    let (parts, body) = req.into_parts();
    let target: Vec<u8> = parts
        .uri
        .path_and_query()
        .map(|pq| pq.as_str().as_bytes().to_vec())
        .unwrap_or_else(|| b"/".to_vec());
    let (path_part, query) = split_target(&target);
    let decoded = normalize_path(&percent_decode_path(path_part));
    match translate(&cfg.docroot, &decoded) {
        Resolved::Static(file) => {
            let bytes = std::fs::read(&file).unwrap_or_default();
            let mut hm = HeaderMap::new();
            let ext = file.extension().map(|e| e.as_bytes().to_vec()).unwrap_or_default();
            if let Some(mime) = mime_for_ext(&ext) {
                let ct = if mime.starts_with("text/") { format!("{mime}; charset=UTF-8") } else { mime.to_string() };
                if let Ok(v) = HeaderValue::from_str(&ct) {
                    hm.insert(header::CONTENT_TYPE, v);
                }
            }
            Routed::Ready((StatusCode::OK, hm, bytes).into_response())
        }
        Resolved::NotFound => {
            let mut hm = HeaderMap::new();
            hm.insert(HeaderName::from_static("x-powered-by"), HeaderValue::from_static("PHP/8.5.7"));
            hm.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=UTF-8"));
            Routed::Ready((StatusCode::NOT_FOUND, hm, not_found_page(&decoded)).into_response())
        }
        Resolved::Script(file, vpath, path_info) => {
            let source = match std::fs::read(&file) {
                Ok(s) => s,
                Err(e) => {
                    // The cli-server answers an unreadable script with a bare 500
                    // (server.rs write_bare_error); the reason goes to the log.
                    eprintln!("[{}] php-server: cannot read {}: {e}", asctime_local(), file.display());
                    let mut hm = HeaderMap::new();
                    hm.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=UTF-8"));
                    return Routed::Ready((StatusCode::INTERNAL_SERVER_ERROR, hm, Vec::new()).into_response());
                }
            };
            let body = match axum::body::to_bytes(body, BODY_CAP).await {
                Ok(b) => b.to_vec(),
                Err(_) => Vec::new(),
            };
            let headers: Vec<(Vec<u8>, Vec<u8>)> = parts
                .headers
                .iter()
                .map(|(n, v)| (n.as_str().as_bytes().to_vec(), v.as_bytes().to_vec()))
                .collect();
            let request_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            let web = WebRequest {
                method: parts.method.as_str().as_bytes().to_vec(),
                protocol: version_pair(parts.version),
                request_uri: target,
                vpath,
                path_info,
                query_string: query,
                headers,
                body,
                remote_addr: peer.ip().to_string(),
                remote_port: peer.port(),
                server_host: cfg.host.clone(),
                server_port: cfg.port,
                doc_root: cfg.docroot.as_os_str().as_bytes().to_vec(),
                script_filename: file.as_os_str().as_bytes().to_vec(),
                request_time,
            };
            // A-TH8: the script name is the resolved FILESYSTEM path (error
            // banners render it; the unit cache keys on it).
            let meta = WorkerHandlerMeta { path: file.to_string_lossy().into_owned(), source };
            Routed::Script { meta, web }
        }
    }
}

/// The worker's response as an HTTP response: `Name: value` lines become
/// headers (`append`, so repeated `Set-Cookie` lines survive), the default
/// `Content-type: text/html; charset=UTF-8` is added when the script set none
/// (cli-server `write_php_response`).
pub fn into_response(r: WorkerResponse) -> Response {
    let status = StatusCode::from_u16(r.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    if let Some(reason) = r.reason.as_deref().filter(|p| *p != status_reason(i64::from(r.status)).as_bytes()) {
        // Declared residue (§3.9): hyper emits the canonical reason phrase
        // (WordPress' status_header() sets the canonical one: no note).
        eprintln!(
            "[{}] php-server: custom reason phrase not emitted on the wire (TODO(port) §3.9): {}",
            asctime_local(),
            String::from_utf8_lossy(reason)
        );
    }
    let mut hm = HeaderMap::new();
    let mut have_ctype = false;
    for line in &r.headers {
        let Some(p) = line.iter().position(|&b| b == b':') else { continue };
        let name = line[..p].trim_ascii();
        let value = line[p + 1..].trim_ascii_start();
        match (HeaderName::from_bytes(name), HeaderValue::from_bytes(value)) {
            (Ok(n), Ok(v)) => {
                if n == header::CONTENT_TYPE {
                    have_ctype = true;
                }
                hm.append(n, v);
            }
            _ => eprintln!(
                "[{}] php-server: header line not representable on the wire, dropped (TODO(port) §3.9): {}",
                asctime_local(),
                String::from_utf8_lossy(line)
            ),
        }
    }
    if !have_ctype {
        hm.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=UTF-8"));
    }
    (status, hm, Body::from(r.body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker_pool::{execute_request, WorkerPool, WorkerPoolContext, WorkerTask};
    use php_builtins::registry;
    use std::sync::Arc;
    use tokio::sync::oneshot;

    fn tree() -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("phpr-axumweb-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("wp-includes/css")).unwrap();
        std::fs::write(root.join("index.php"), "<?php echo 'root:', $_SERVER['PATH_INFO'] ?? '-';").unwrap();
        std::fs::write(root.join("wp-includes/css/buttons.css"), "body{}").unwrap();
        std::fs::canonicalize(root).unwrap()
    }

    fn cfg(root: &PathBuf) -> ServeCfg {
        ServeCfg { host: "127.0.0.1".into(), port: 8080, docroot: root.clone() }
    }

    fn peer() -> SocketAddr {
        "127.0.0.1:50000".parse().unwrap()
    }

    fn get(uri: &str) -> axum::extract::Request {
        axum::http::Request::builder().uri(uri).header("Host", "127.0.0.1:8080").body(Body::empty()).unwrap()
    }

    /// A web request for `/gate.php` as the bridge would build it.
    fn web(method: &str, uri: &str, path_info: Option<&str>, extra: &[(&str, &str)], body: &[u8]) -> WebRequest {
        let (_p, q) = split_target(uri.as_bytes());
        let mut headers = vec![(b"host".to_vec(), b"127.0.0.1:8080".to_vec())];
        for (k, v) in extra {
            headers.push((k.as_bytes().to_vec(), v.as_bytes().to_vec()));
        }
        WebRequest {
            method: method.as_bytes().to_vec(),
            protocol: (1, 1),
            request_uri: uri.as_bytes().to_vec(),
            vpath: b"/gate.php".to_vec(),
            path_info: path_info.map(|s| s.as_bytes().to_vec()),
            query_string: q,
            headers,
            body: body.to_vec(),
            remote_addr: "127.0.0.1".into(),
            remote_port: 50000,
            server_host: "127.0.0.1".into(),
            server_port: 8080,
            doc_root: b"/tmp".to_vec(),
            script_filename: b"/tmp/gate.php".to_vec(),
            request_time: 0.0,
        }
    }

    fn meta(src: &str) -> WorkerHandlerMeta {
        WorkerHandlerMeta { path: "/gate.php".to_string(), source: src.as_bytes().to_vec() }
    }

    /// Run one script on this thread with `web` installed, as the worker does.
    fn run(src: &str, web: Option<WebRequest>) -> WorkerResponse {
        let reg = registry();
        let scope = RequestScope::install(web);
        let r = execute_request(&reg, &meta(src));
        drop(scope);
        r
    }

    #[tokio::test]
    async fn route_root_permalink_static_and_404() {
        let root = tree();
        let c = cfg(&root);
        // root → index.php, no PATH_INFO
        match route(&c, peer(), get("/")).await {
            Routed::Script { meta, web } => {
                assert!(meta.path.ends_with("/index.php"));
                assert_eq!(web.vpath, b"/index.php");
                assert!(web.path_info.is_none());
                assert_eq!(web.method, b"GET");
                assert_eq!(web.request_uri, b"/");
            }
            Routed::Ready(_) => panic!("root must be a script"),
        }
        // pretty permalink → root index.php with PATH_INFO + query split
        match route(&c, peer(), get("/2026/07/hello-world/?x=1")).await {
            Routed::Script { web, .. } => {
                assert_eq!(web.path_info.as_deref(), Some(&b"/2026/07/hello-world/"[..]));
                assert_eq!(web.query_string.as_deref(), Some(&b"x=1"[..]));
                assert_eq!(web.request_uri, b"/2026/07/hello-world/?x=1");
                assert_eq!(web.doc_root, root.as_os_str().as_bytes());
            }
            Routed::Ready(_) => panic!("permalink must fall back to index.php"),
        }
        // static css with charset
        match route(&c, peer(), get("/wp-includes/css/buttons.css")).await {
            Routed::Ready(resp) => {
                assert_eq!(resp.status(), StatusCode::OK);
                assert_eq!(resp.headers()[header::CONTENT_TYPE], "text/css; charset=UTF-8");
                let b = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
                assert_eq!(&b[..], b"body{}");
            }
            Routed::Script { .. } => panic!("css must be static"),
        }
        // 404 page (no root index.php in a bare docroot)
        std::fs::remove_file(root.join("index.php")).unwrap();
        match route(&c, peer(), get("/missing/x")).await {
            Routed::Ready(resp) => {
                assert_eq!(resp.status(), StatusCode::NOT_FOUND);
                assert_eq!(resp.headers()["x-powered-by"], "PHP/8.5.7");
                let b = axum::body::to_bytes(resp.into_body(), 1 << 20).await.unwrap();
                assert_eq!(&b[..], &not_found_page(b"/missing/x")[..]);
            }
            Routed::Script { .. } => panic!("must be 404"),
        }
    }

    #[test]
    fn superglobals_seeded_and_scope_cleared() {
        // (php_sapi_name() is process-global, set once in main() for --axum —
        // asserted by the WordPress battery on the real binary, not here.)
        let src = "<?php echo $_SERVER['REQUEST_METHOD'], '|', $_SERVER['PATH_INFO'], '|', $_GET['x'], '|', \
                   $_COOKIE['c'], '|', $_SERVER['HTTP_HOST'], '|', $_POST['p'], '|', $_SERVER['REMOTE_PORT'];";
        let w = web("POST", "/gate.php/extra?x=1", Some("/extra"), &[("cookie", "c=v"), ("content-type", "application/x-www-form-urlencoded")], b"p=42");
        let r = run(src, Some(w));
        assert_eq!(r.status, 200);
        assert_eq!(String::from_utf8_lossy(&r.body), "POST|/extra|1|v|127.0.0.1:8080|42|50000");
        assert!(php_types::sapi::web_request().is_none(), "scope must clear the thread-local");
        assert!(r.headers.iter().any(|h| h.starts_with(b"X-Powered-By: PHP/8.5.7")), "{:?}", r.headers);
        // without a web request: CLI semantics (no seeding, no X-Powered-By)
        let r = run("<?php echo isset($_SERVER['REQUEST_METHOD']) ? 'web' : 'cli';", None);
        assert_eq!(r.body, b"cli");
        assert!(r.headers.is_empty());
    }

    #[test]
    fn headers_cookies_location_and_exit_status() {
        let w = web("GET", "/gate.php", None, &[], b"");
        let r = run("<?php header('X-A: 1'); setcookie('a', '1'); setcookie('b', '2'); header('Location: /x'); echo 'moved';", Some(w));
        assert_eq!(r.status, 302, "Location implies 302");
        let resp = into_response(r);
        assert_eq!(resp.status(), StatusCode::FOUND);
        assert_eq!(resp.headers().get_all(header::SET_COOKIE).iter().count(), 2, "both cookies survive");
        assert_eq!(resp.headers()["x-a"], "1");
        assert_eq!(resp.headers()[header::LOCATION], "/x");
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "text/html; charset=UTF-8", "default content-type");
        // exit() keeps the code the script set; a script content-type is kept
        let w = web("GET", "/gate.php", None, &[], b"");
        let r = run("<?php header('Content-Type: text/plain'); http_response_code(404); echo 'x'; exit;", Some(w));
        assert_eq!((r.status, &r.body[..]), (404, &b"x"[..]));
        let resp = into_response(r);
        assert_eq!(resp.headers()[header::CONTENT_TYPE], "text/plain");
    }

    #[test]
    fn fatals_under_web_sapi_are_200_html_banners() {
        // runtime fatal (oracle-fatal-status.out: php -S answers 200 + html)
        let r = run("<?php echo 'pre'; nope_missing_fn();", Some(web("GET", "/gate.php", None, &[], b"")));
        assert_eq!(r.status, 200);
        let body = String::from_utf8_lossy(&r.body);
        assert!(body.starts_with("pre<br />\n<b>Fatal error</b>:  Uncaught Error: Call to undefined function nope_missing_fn()"), "{body}");
        // compile fatal
        let r = run("<?php echo 'pre'; goto nope;", Some(web("GET", "/gate.php", None, &[], b"")));
        assert_eq!(r.status, 200);
        assert!(String::from_utf8_lossy(&r.body).starts_with("<br />\n<b>Fatal error</b>:  'goto' to undefined label 'nope'"), "{:?}", String::from_utf8_lossy(&r.body));
        assert!(r.headers.iter().any(|h| h.starts_with(b"X-Powered-By")));
        // parse error: html envelope, 200
        let r = run("<?php echo 'pre'; function f( {", Some(web("GET", "/gate.php", None, &[], b"")));
        assert_eq!(r.status, 200);
        assert!(String::from_utf8_lossy(&r.body).starts_with("<br />\n<b>Parse error</b>:  "));
        // link-time fatal (enum implementing Serializable): html banner under the
        // web SAPI (oracle php -S probe 03/11: no stack trace in the body), plain otherwise
        let r = run("<?php\nenum LF implements Serializable { case A; }\necho 'never';", Some(web("GET", "/gate.php", None, &[], b"")));
        assert_eq!(r.status, 200);
        assert!(String::from_utf8_lossy(&r.body).contains("<br />\n<b>Fatal error</b>:  Enum LF"), "{:?}", String::from_utf8_lossy(&r.body));
        let r = run("<?php\nenum LF2 implements Serializable { case A; }", None);
        assert_eq!(r.status, 500);
        assert!(String::from_utf8_lossy(&r.body).contains("\nFatal error: Enum LF2"), "{:?}", String::from_utf8_lossy(&r.body));
        // and the pre-S-188 CLI-anchored contract without a web request: 500 + plain
        let r = run("<?php goto nope;", None);
        assert_eq!(r.status, 500);
        assert!(r.body.starts_with(b"\nFatal error: 'goto' to undefined label 'nope'"));
        let r = run("<?php function f( {", None);
        assert_eq!((r.status, &r.body[..14]), (500, &b"\nParse error: "[..]));
    }

    #[test]
    fn two_workers_no_request_bleed() {
        let ctx = WorkerPoolContext::new();
        let pool = WorkerPool::new(ctx, 2);
        let src = "<?php echo $_SERVER['PATH_INFO'] ?? 'none';";
        let mut rxs = Vec::new();
        for pi in ["/one", "/two", "/three", "/four"] {
            let (tx, rx) = oneshot::channel();
            pool.dispatch(WorkerTask { meta: meta(src), web: Some(web("GET", "/gate.php", Some(pi), &[], b"")), response_tx: tx }).unwrap();
            rxs.push((pi, rx));
        }
        for (pi, rx) in rxs {
            let r = rx.blocking_recv().unwrap();
            assert_eq!(String::from_utf8_lossy(&r.body), pi, "worker answered another request's PATH_INFO");
        }
        // a request without a web request on the same workers sees no leftovers
        for _ in 0..2 {
            let (tx, rx) = oneshot::channel();
            pool.dispatch(WorkerTask { meta: meta(src), web: None, response_tx: tx }).unwrap();
            assert_eq!(rx.blocking_recv().unwrap().body, b"none");
        }
        let Ok(pool) = Arc::try_unwrap(pool) else { panic!("pool still shared") };
        pool.shutdown();
    }


    /// S-189 (Council, signed 2026-11-03; Klabnik R5, Pedersen R9): the in-CI
    /// falsifier of the shared queue. With per-worker queues and blind
    /// round-robin a fresh W=2 pool sends A→w0, B→w1, C→w0, so C waits behind
    /// the long A (RED on the C2 tree: wp189-harness/s189-tooth-red-c2.out);
    /// with the shared FIFO w1 takes C as soon as B is done (GREEN). Bounds
    /// live in the assert messages — never a sleep as synchronisation (A-TH25).
    #[test]
    fn no_idle_worker_while_queue_nonempty() {
        // the census counters are process-global: serialize with the pool tests (A-SK8)
        let _guard = crate::worker_pool::DEPTH_TEST_LOCK.lock().unwrap();
        let ctx = WorkerPoolContext::new();
        let pool = WorkerPool::new(ctx, 2);
        let mut rxs = Vec::new();
        for src in ["<?php usleep(1500000); echo \"A\";", "<?php echo \"B\";", "<?php echo \"C\";"] {
            let (tx, rx) = oneshot::channel();
            pool.dispatch(WorkerTask { meta: meta(src), web: None, response_tx: tx }).unwrap();
            rxs.push(rx);
        }
        let t0 = std::time::Instant::now();
        let c = rxs.pop().unwrap();
        let b = rxs.pop().unwrap();
        let a = rxs.pop().unwrap();
        assert_eq!(b.blocking_recv().unwrap().body, b"B");
        assert_eq!(c.blocking_recv().unwrap().body, b"C");
        let t_c = t0.elapsed();
        assert!(
            t_c < std::time::Duration::from_millis(1000),
            "C answered after {t_c:?}: it waited behind the long A while a worker was idle (round-robin stall, S-189)"
        );
        assert_eq!(a.blocking_recv().unwrap().body, b"A");
        assert!(t0.elapsed() < std::time::Duration::from_millis(5000), "A did not finish within 5 s");
        let Ok(pool) = Arc::try_unwrap(pool) else { panic!("pool still shared") };
        pool.shutdown();
    }
}
