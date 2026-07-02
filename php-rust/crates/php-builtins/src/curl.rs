//! ext/curl easy-handle facade over the rustls-backed `ureq` transport (the
//! same backend as the http(s):// stream wrapper in `file.rs`).
//!
//! Scope: only the *easy* API is modelled. `curl_multi_*` is deliberately
//! absent so dual-backend consumers that probe for it (Composer's
//! `HttpDownloader::isCurlEnabled`) keep taking the stream-wrapper path, while
//! `function_exists('curl_exec')` consumers (monolog's `Curl\Util`, Guzzle's
//! sync `CurlHandler`) get a working surface. Callback/stream sink options
//! (`CURLOPT_WRITEFUNCTION`, `CURLOPT_FILE`, …) need VM re-entry that a
//! registry builtin cannot do, so `curl_setopt` refuses them with a Warning
//! rather than silently dropping response data.
//!
//! A handle's state lives in a thread-local table keyed by an integer id; the
//! PHP-visible `CurlHandle` object is a prelude class wrapping that id (the
//! ext/dom pattern). `curl_close()` is a PHP 8 no-op, so entries live for the
//! run's lifetime — fine for the CLI workloads phpr targets.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use php_runtime::Ctx;
use php_types::{convert, Diag, Key, PhpArray, PhpError, PhpStr, Zval};

// CURLOPT_* / CURLINFO_* numbers used below (values from the brew PHP 8.5
// oracle dump; the full 689-constant table lives in php-runtime's
// lower/curl_consts.rs).
const CURLOPT_URL: i64 = 10002;
const CURLOPT_POST: i64 = 47;
const CURLOPT_RETURNTRANSFER: i64 = 19913;
const CURLOPT_POSTFIELDS: i64 = 10015;
const CURLOPT_HTTPHEADER: i64 = 10023;
const CURLOPT_CUSTOMREQUEST: i64 = 10036;
const CURLOPT_SSL_VERIFYPEER: i64 = 64;
const CURLOPT_SSL_VERIFYHOST: i64 = 81;
const CURLOPT_TIMEOUT: i64 = 13;
const CURLOPT_TIMEOUT_MS: i64 = 155;
const CURLOPT_CONNECTTIMEOUT: i64 = 78;
const CURLOPT_CONNECTTIMEOUT_MS: i64 = 156;
const CURLOPT_FOLLOWLOCATION: i64 = 52;
const CURLOPT_MAXREDIRS: i64 = 68;
const CURLOPT_USERAGENT: i64 = 10018;
const CURLOPT_USERPWD: i64 = 10005;
const CURLOPT_USERNAME: i64 = 10173;
const CURLOPT_PASSWORD: i64 = 10174;
const CURLOPT_HEADER: i64 = 42;
const CURLOPT_NOBODY: i64 = 44;
const CURLOPT_HTTPGET: i64 = 80;
const CURLOPT_UPLOAD: i64 = 46;
const CURLOPT_PUT: i64 = 54;
const CURLOPT_FAILONERROR: i64 = 45;
const CURLOPT_REFERER: i64 = 10016;
const CURLOPT_COOKIE: i64 = 10022;
// Options that require calling back into PHP (closures) or writing to a PHP
// stream mid-transfer — not expressible from a registry builtin.
const CALLBACK_OPTS: &[i64] = &[
    10001, // CURLOPT_FILE
    10009, // CURLOPT_INFILE
    10029, // CURLOPT_WRITEHEADER
    20011, // CURLOPT_WRITEFUNCTION
    20012, // CURLOPT_READFUNCTION
    20079, // CURLOPT_HEADERFUNCTION
];

const CURLINFO_EFFECTIVE_URL: i64 = 1048577;
const CURLINFO_HTTP_CODE: i64 = 2097154; // == CURLINFO_RESPONSE_CODE
const CURLINFO_CONTENT_TYPE: i64 = 1048594;
const CURLINFO_HEADER_SIZE: i64 = 2097163;
const CURLINFO_TOTAL_TIME: i64 = 3145731;
const CURLINFO_TOTAL_TIME_T: i64 = 6291506;
const CURLINFO_SIZE_DOWNLOAD: i64 = 3145736;
const CURLINFO_REDIRECT_COUNT: i64 = 2097172;
const CURLINFO_REDIRECT_URL: i64 = 1048607;
const CURLINFO_SCHEME: i64 = 1048625;
const CURLINFO_EFFECTIVE_METHOD: i64 = 1048634;
const CURLINFO_PRIVATE: i64 = 1048597;

/// `curl_strerror()` messages for CURLE 0..=99, from the oracle's libcurl.
static CURL_STRERROR: [&str; 100] = [
    "No error",
    "Unsupported protocol",
    "Failed initialization",
    "URL using bad/illegal format or missing URL",
    "A requested feature, protocol or option was not found built-in in this libcurl due to a build-time decision.",
    "Could not resolve proxy name",
    "Could not resolve hostname",
    "Could not connect to server",
    "Weird server reply",
    "Access denied to remote resource",
    "FTP: The server failed to connect to data port",
    "FTP: unknown PASS reply",
    "FTP: Accepting server connect has timed out",
    "FTP: unknown PASV reply",
    "FTP: unknown 227 response format",
    "FTP: cannot figure out the host in the PASV response",
    "Error in the HTTP2 framing layer",
    "FTP: could not set file type",
    "Transferred a partial file",
    "FTP: could not retrieve (RETR failed) the specified file",
    "Unknown error",
    "Quote command returned error",
    "HTTP response code said error",
    "Failed writing received data to disk/application",
    "Unknown error",
    "Upload failed (at start/before it took off)",
    "Failed to open/read local data from file/application",
    "Out of memory",
    "Timeout was reached",
    "Unknown error",
    "FTP: command PORT failed",
    "FTP: command REST failed",
    "Unknown error",
    "Requested range was not delivered by the server",
    "Unknown error",
    "SSL connect error",
    "Could not resume download",
    "Could not read a file:// file",
    "LDAP: cannot bind",
    "LDAP: search failed",
    "Unknown error",
    "Unknown error",
    "Operation was aborted by an application callback",
    "A libcurl function was given a bad argument",
    "Unknown error",
    "Failed binding local connection end",
    "Unknown error",
    "Number of redirects hit maximum amount",
    "An unknown option was passed in to libcurl",
    "Malformed option provided in a setopt",
    "Unknown error",
    "Unknown error",
    "Server returned nothing (no headers, no data)",
    "SSL crypto engine not found",
    "Can not set SSL crypto engine as default",
    "Failed sending data to the peer",
    "Failure when receiving data from the peer",
    "Unknown error",
    "Problem with the local SSL certificate",
    "Could not use specified SSL cipher",
    "SSL peer certificate or SSH remote key was not OK",
    "Unrecognized or bad HTTP Content or Transfer-Encoding",
    "Unknown error",
    "Maximum file size exceeded",
    "Requested SSL level failed",
    "Send failed since rewinding of the data stream failed",
    "Failed to initialize SSL crypto engine",
    "Login denied",
    "TFTP: File Not Found",
    "TFTP: Access Violation",
    "Disk full or allocation exceeded",
    "TFTP: Illegal operation",
    "TFTP: Unknown transfer ID",
    "Remote file already exists",
    "TFTP: No such user",
    "Unknown error",
    "Unknown error",
    "Problem with the SSL CA cert (path? access rights?)",
    "Remote file not found",
    "Error in the SSH layer",
    "Failed to shut down the SSL connection",
    "Socket not ready for send/recv",
    "Failed to load CRL file (path? access rights?, format?)",
    "Issuer check against peer certificate failed",
    "FTP: The server did not accept the PRET command.",
    "RTSP CSeq mismatch or invalid CSeq",
    "RTSP session error",
    "Unable to parse FTP file list",
    "Chunk callback failed",
    "The max connection limit is reached",
    "SSL public key does not match pinned public key",
    "SSL server certificate status verification FAILED",
    "Stream error in the HTTP/2 framing layer",
    "API function called from within callback",
    "An authentication function returned an error",
    "HTTP/3 error",
    "QUIC connection error",
    "proxy handshake error",
    "SSL Client Certificate required",
    "Unrecoverable error in select/poll",
];

/// POSTFIELDS payload: a raw string body, or an array → multipart/form-data.
#[derive(Clone)]
enum PostBody {
    Str(Vec<u8>),
    Form(Vec<(Vec<u8>, Vec<u8>)>),
}

/// Transfer results of the last `curl_exec`, backing `curl_getinfo`/`_errno`/`_error`.
#[derive(Clone, Default)]
struct LastTransfer {
    http_code: i64,
    content_type: Option<Vec<u8>>,
    /// Response status line + header lines + blank line, `\r\n`-joined — what
    /// `CURLOPT_HEADER` prepends and what `header_size` measures.
    header_block: Vec<u8>,
    request_size: i64,
    size_download: i64,
    total_time_us: i64,
    effective_method: Vec<u8>,
    scheme: Vec<u8>,
}

/// One easy handle's configuration + last-transfer state.
#[derive(Clone)]
struct CurlState {
    url: Vec<u8>,
    custom_method: Option<Vec<u8>>,
    post: bool,
    nobody: bool,
    upload: bool,
    postfields: Option<PostBody>,
    http_headers: Vec<Vec<u8>>,
    return_transfer: bool,
    include_header: bool,
    follow_location: bool,
    max_redirs: i64,
    timeout_ms: u64,
    connect_timeout_ms: u64,
    ssl_verify_peer: bool,
    useragent: Option<Vec<u8>>,
    username: Option<Vec<u8>>,
    password: Option<Vec<u8>>,
    userpwd: Option<Vec<u8>>,
    fail_on_error: bool,
    referer: Option<Vec<u8>>,
    cookie: Option<Vec<u8>>,
    private_data: Option<Zval>,
    errno: i64,
    error: String,
    last: LastTransfer,
}

impl Default for CurlState {
    fn default() -> Self {
        CurlState {
            url: Vec::new(),
            custom_method: None,
            post: false,
            nobody: false,
            upload: false,
            postfields: None,
            http_headers: Vec::new(),
            return_transfer: false,
            include_header: false,
            follow_location: false,
            // libcurl's CURLOPT_MAXREDIRS default: -1 = unlimited (curl caps at 30).
            max_redirs: -1,
            timeout_ms: 0,
            connect_timeout_ms: 0,
            ssl_verify_peer: true,
            useragent: None,
            username: None,
            password: None,
            userpwd: None,
            fail_on_error: false,
            referer: None,
            cookie: None,
            private_data: None,
            errno: 0,
            error: String::new(),
            last: LastTransfer::default(),
        }
    }
}

thread_local! {
    static HANDLES: RefCell<HashMap<i64, CurlState>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<i64> = const { RefCell::new(1) };
}

/// The `id` argument (arg #0 of every `__curl_*` builtin — the prelude passes
/// `$handle->__id`).
fn handle_id(argv: &[Zval], name: &str) -> Result<i64, PhpError> {
    match argv.first().map(|v| v.deref_clone()) {
        Some(Zval::Long(n)) => Ok(n),
        _ => Err(PhpError::Error(format!("{name}(): invalid curl handle id"))),
    }
}

/// Run `f` on the handle's state, erroring on an unknown id (a handle from a
/// foreign thread or a corrupted `__id`).
fn with_state<R>(
    id: i64,
    name: &str,
    f: impl FnOnce(&mut CurlState) -> R,
) -> Result<R, PhpError> {
    HANDLES.with(|h| {
        let mut map = h.borrow_mut();
        match map.get_mut(&id) {
            Some(st) => Ok(f(st)),
            None => Err(PhpError::Error(format!("{name}(): invalid curl handle id"))),
        }
    })
}

/// `__curl_init() -> id`: allocate a fresh easy-handle state.
pub fn __curl_init(_argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = NEXT_ID.with(|n| {
        let mut n = n.borrow_mut();
        let id = *n;
        *n += 1;
        id
    });
    HANDLES.with(|h| h.borrow_mut().insert(id, CurlState::default()));
    Ok(Zval::Long(id))
}

/// A setopt value as bytes, following references. Non-strings go through the
/// standard string cast (curl option values are `char*` in libcurl).
fn opt_bytes(v: &Zval, ctx: &mut Ctx) -> Vec<u8> {
    convert::to_zstr(v, ctx.diags).as_bytes().to_vec()
}

fn opt_bool(v: &Zval) -> bool {
    convert::is_true_silent(v)
}

fn opt_long(v: &Zval, ctx: &mut Ctx) -> i64 {
    convert::to_long_cast(v, ctx.diags)
}

/// `__curl_setopt(id, option, value) -> bool`.
pub fn __curl_setopt(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_setopt")?;
    let opt = match argv.get(1).map(|v| v.deref_clone()) {
        Some(Zval::Long(n)) => n,
        _ => {
            return Err(PhpError::TypeError(
                "curl_setopt(): Argument #2 ($option) must be of type int".to_string(),
            ))
        }
    };
    let value = argv.get(2).cloned().unwrap_or(Zval::Null).deref_clone();
    if CALLBACK_OPTS.contains(&opt) {
        // Refusing (false) is honest: silently accepting would drop response
        // data that the callback/stream was supposed to receive.
        ctx.diags.push(Diag::Warning(format!(
            "curl_setopt(): phpr does not support callback/stream option {opt}"
        )));
        return Ok(Zval::Bool(false));
    }
    let val_is_null = matches!(value, Zval::Null);
    with_state(id, "curl_setopt", |st| -> Result<(), PhpError> {
        match opt {
        CURLOPT_URL => st.url = opt_bytes(&value, ctx),
        CURLOPT_POST => st.post = opt_bool(&value),
        CURLOPT_RETURNTRANSFER => st.return_transfer = opt_bool(&value),
        CURLOPT_POSTFIELDS => match &value {
            Zval::Array(a) => {
                let mut form = Vec::new();
                for (k, v) in a.iter() {
                    let kb = match k {
                        Key::Int(i) => i.to_string().into_bytes(),
                        Key::Str(s) => s.as_bytes().to_vec(),
                    };
                    form.push((kb, opt_bytes(v, ctx)));
                }
                st.postfields = Some(PostBody::Form(form));
            }
            _ => st.postfields = Some(PostBody::Str(opt_bytes(&value, ctx))),
        },
        CURLOPT_HTTPHEADER => {
            let Zval::Array(a) = &value else {
                return Err(PhpError::TypeError(
                    "curl_setopt(): Argument #3 ($value) must be of type array for the CURLOPT_HTTPHEADER option"
                        .to_string(),
                ));
            };
            st.http_headers = a.iter().map(|(_, v)| opt_bytes(v, ctx)).collect();
        }
        CURLOPT_CUSTOMREQUEST => {
            st.custom_method = if val_is_null { None } else { Some(opt_bytes(&value, ctx)) }
        }
        CURLOPT_SSL_VERIFYPEER => st.ssl_verify_peer = opt_bool(&value),
        // Host verification is not separable from peer verification in rustls;
        // VERIFYHOST is accepted for compatibility (2 = default behaviour).
        CURLOPT_SSL_VERIFYHOST => {}
        CURLOPT_TIMEOUT => st.timeout_ms = (opt_long(&value, ctx).max(0) as u64) * 1000,
        CURLOPT_TIMEOUT_MS => st.timeout_ms = opt_long(&value, ctx).max(0) as u64,
        CURLOPT_CONNECTTIMEOUT => {
            st.connect_timeout_ms = (opt_long(&value, ctx).max(0) as u64) * 1000
        }
        CURLOPT_CONNECTTIMEOUT_MS => st.connect_timeout_ms = opt_long(&value, ctx).max(0) as u64,
        CURLOPT_FOLLOWLOCATION => st.follow_location = opt_bool(&value),
        CURLOPT_MAXREDIRS => st.max_redirs = opt_long(&value, ctx),
        CURLOPT_USERAGENT => st.useragent = Some(opt_bytes(&value, ctx)),
        CURLOPT_USERPWD => st.userpwd = Some(opt_bytes(&value, ctx)),
        CURLOPT_USERNAME => st.username = Some(opt_bytes(&value, ctx)),
        CURLOPT_PASSWORD => st.password = Some(opt_bytes(&value, ctx)),
        CURLOPT_HEADER => st.include_header = opt_bool(&value),
        CURLOPT_NOBODY => st.nobody = opt_bool(&value),
        CURLOPT_HTTPGET => {
            if opt_bool(&value) {
                st.post = false;
                st.nobody = false;
                st.upload = false;
                st.custom_method = None;
                st.postfields = None;
            }
        }
        CURLOPT_UPLOAD | CURLOPT_PUT => st.upload = opt_bool(&value),
        CURLOPT_FAILONERROR => st.fail_on_error = opt_bool(&value),
        CURLOPT_REFERER => st.referer = Some(opt_bytes(&value, ctx)),
        CURLOPT_COOKIE => st.cookie = Some(opt_bytes(&value, ctx)),
        CURLINFO_PRIVATE => st.private_data = Some(value.clone()),
        // Every other option (VERBOSE, NOSIGNAL, NOPROGRESS, buffer sizes,
        // protocol restrictions, …) is accepted and has no behavioural
        // counterpart in the ureq backend.
        _ => {}
        }
        Ok(())
    })??;
    Ok(Zval::Bool(true))
}

/// Map a `ureq` transport error to (CURLE errno, curl-style message).
fn map_transport_error(e: &ureq::Error, host: &str) -> (i64, String) {
    use ureq::Error as E;
    match e {
        E::HostNotFound => (6, format!("Could not resolve host: {host}")),
        E::Timeout(_) => (28, "Timeout was reached".to_string()),
        E::ConnectionFailed => (7, format!("Failed to connect to {host}")),
        E::TooManyRedirects | E::RedirectFailed => {
            (47, "Number of redirects hit maximum amount".to_string())
        }
        E::BadUri(_) => (3, "URL using bad/illegal format or missing URL".to_string()),
        E::Tls(m) => (35, (*m).to_string()),
        E::Rustls(err) => {
            let m = err.to_string();
            // Certificate validation failures are CURLE_PEER_FAILED_VERIFICATION.
            if m.to_ascii_lowercase().contains("certificate") {
                (60, m)
            } else {
                (35, m)
            }
        }
        // DNS failures surface as Io(getaddrinfo) rather than HostNotFound.
        E::Io(ioe) if ioe.to_string().contains("lookup address") => {
            (6, format!("Could not resolve host: {host}"))
        }
        E::Io(ioe) if ioe.kind() == std::io::ErrorKind::ConnectionRefused => {
            (7, format!("Failed to connect to {host}"))
        }
        other => (56, other.to_string()),
    }
}

/// The host portion of a URL, for error messages ("Could not resolve host: x").
fn url_host(url: &str) -> String {
    let rest = url.split("://").nth(1).unwrap_or(url);
    let auth = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = auth.rsplit('@').next().unwrap_or(auth);
    host.split(':').next().unwrap_or(host).to_string()
}

/// Build the multipart/form-data body for an array POSTFIELDS.
fn multipart_body(form: &[(Vec<u8>, Vec<u8>)], boundary: &str) -> Vec<u8> {
    let mut body = Vec::new();
    for (k, v) in form {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"");
        body.extend_from_slice(k);
        body.extend_from_slice(b"\"\r\n\r\n");
        body.extend_from_slice(v);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

/// `__curl_exec(id) -> string|bool`: perform the transfer. Body goes to the
/// return value under CURLOPT_RETURNTRANSFER, to stdout otherwise (curl's
/// default sink).
pub fn __curl_exec(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_exec")?;
    // Snapshot the config so the table borrow is not held across the transfer.
    let mut st = with_state(id, "curl_exec", |s| s.clone())?;
    st.errno = 0;
    st.error = String::new();
    st.last = LastTransfer::default();

    let fail = |st: &mut CurlState, errno: i64, msg: String| {
        st.errno = errno;
        st.error = msg;
    };

    let outcome: Option<Vec<u8>> = (|| {
        let Ok(url_str) = std::str::from_utf8(&st.url).map(str::to_owned) else {
            fail(&mut st, 3, "URL using bad/illegal format or missing URL".into());
            return None;
        };
        if url_str.is_empty() {
            fail(&mut st, 3, "No URL set".into());
            return None;
        }
        let lower = url_str.to_ascii_lowercase();
        // getinfo 'scheme' reads lowercase in the oracle ("http"/"https").
        let scheme = if lower.starts_with("https://") {
            "https"
        } else if lower.starts_with("http://") {
            "http"
        } else {
            let s = url_str.split(':').next().unwrap_or("").to_string();
            fail(&mut st, 1, format!("Protocol \"{s}\" not supported"));
            return None;
        };
        st.last.scheme = scheme.as_bytes().to_vec();

        // Method resolution mirrors libcurl: CUSTOMREQUEST overrides the verb
        // chosen by POST/UPLOAD/NOBODY, which override the GET default.
        let method_bytes: Vec<u8> = match &st.custom_method {
            Some(m) => m.clone(),
            None if st.post => b"POST".to_vec(),
            None if st.upload => b"PUT".to_vec(),
            None if st.nobody => b"HEAD".to_vec(),
            None => b"GET".to_vec(),
        };
        let Ok(method) = ureq::http::Method::from_bytes(&method_bytes) else {
            fail(&mut st, 3, "URL using bad/illegal format or missing URL".into());
            return None;
        };
        st.last.effective_method = method_bytes.clone();

        // Known divergence: with FOLLOWLOCATION off, the body of a 3xx response
        // reads as "" (ureq's protocol layer consumes it while positioning the
        // connection); status and headers stay faithful.
        let mut cfg = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .user_agent(ureq::config::AutoHeaderValue::None)
            .max_redirects_will_error(false)
            .max_redirects(if st.follow_location {
                if st.max_redirs < 0 {
                    30 // curl's cap for "unlimited"
                } else {
                    st.max_redirs.min(u32::MAX as i64) as u32
                }
            } else {
                0
            });
        if st.timeout_ms > 0 {
            cfg = cfg.timeout_global(Some(Duration::from_millis(st.timeout_ms)));
        }
        if st.connect_timeout_ms > 0 {
            cfg = cfg.timeout_connect(Some(Duration::from_millis(st.connect_timeout_ms)));
        }
        if !st.ssl_verify_peer {
            cfg = cfg.tls_config(
                ureq::tls::TlsConfig::builder().disable_verification(true).build(),
            );
        }
        let agent: ureq::Agent = cfg.build().into();

        let mut builder = ureq::http::Request::builder().method(method).uri(&url_str);
        let mut have_ct = false;
        let mut have_accept = false;
        let mut have_ua = false;
        let mut have_auth = false;
        let mut sent_headers_size = 0usize;
        for line in &st.http_headers {
            let Some(pos) = line.iter().position(|&b| b == b':') else {
                continue;
            };
            let name = String::from_utf8_lossy(&line[..pos]).trim().to_string();
            let raw = &line[pos + 1..];
            let value = String::from_utf8_lossy(raw).trim().to_string();
            // "Name:" with an empty value is curl's idiom for suppressing a
            // default header; ureq has no such defaults, so just skip it.
            if value.is_empty() {
                continue;
            }
            let lc = name.to_ascii_lowercase();
            have_ct |= lc == "content-type";
            have_accept |= lc == "accept";
            have_ua |= lc == "user-agent";
            have_auth |= lc == "authorization";
            sent_headers_size += name.len() + 2 + value.len() + 2;
            builder = builder.header(name, value);
        }
        if !have_accept {
            // curl always sends `Accept: */*`.
            builder = builder.header("Accept", "*/*");
            sent_headers_size += "Accept: */*\r\n".len();
        }
        if !have_ua {
            if let Some(ua) = &st.useragent {
                builder = builder.header("User-Agent", String::from_utf8_lossy(ua).to_string());
                sent_headers_size += "User-Agent: \r\n".len() + ua.len();
            }
        }
        if let Some(r) = &st.referer {
            builder = builder.header("Referer", String::from_utf8_lossy(r).to_string());
        }
        if let Some(c) = &st.cookie {
            builder = builder.header("Cookie", String::from_utf8_lossy(c).to_string());
        }
        if !have_auth {
            // USERPWD ("user:pass") or USERNAME+PASSWORD → Basic auth, curl's
            // default CURLAUTH scheme.
            let cred: Option<Vec<u8>> = match (&st.userpwd, &st.username) {
                (Some(up), _) => Some(up.clone()),
                (None, Some(u)) => {
                    let mut c = u.clone();
                    c.push(b':');
                    if let Some(p) = &st.password {
                        c.extend_from_slice(p);
                    }
                    Some(c)
                }
                _ => None,
            };
            if let Some(c) = cred {
                let Ok(Zval::Str(b64)) = crate::encoding::base64_encode(
                    &[Zval::Str(PhpStr::new(c))],
                    ctx,
                ) else {
                    unreachable!("base64_encode of a string cannot fail");
                };
                builder = builder.header(
                    "Authorization",
                    format!("Basic {}", String::from_utf8_lossy(b64.as_bytes())),
                );
            }
        }

        let body: Vec<u8> = match &st.postfields {
            None => Vec::new(),
            Some(PostBody::Str(s)) => {
                if !have_ct && (st.post || st.custom_method.is_some()) {
                    builder = builder.header("Content-Type", "application/x-www-form-urlencoded");
                }
                s.clone()
            }
            Some(PostBody::Form(form)) => {
                // Deterministic boundary: curl randomises it, but nothing may
                // depend on unpredictability here (no cross-request reuse).
                let boundary = format!("------------------------phpr{id:016x}");
                if !have_ct {
                    builder = builder.header(
                        "Content-Type",
                        format!("multipart/form-data; boundary={boundary}"),
                    );
                }
                multipart_body(form, &boundary)
            }
        };

        // Approximate request_size: request line + headers + blank line (we do
        // not see ureq's exact wire bytes; consumers read this only informationally).
        let path_len = url_str.split("://").nth(1).and_then(|r| r.find('/').map(|i| r.len() - i)).unwrap_or(1);
        st.last.request_size =
            (method_bytes.len() + 1 + path_len + " HTTP/1.1\r\n".len() + sent_headers_size + 2) as i64;

        let Ok(request) = builder.body(body) else {
            fail(&mut st, 3, "URL using bad/illegal format or missing URL".into());
            return None;
        };

        let started = Instant::now();
        match agent.run(request) {
            Ok(mut resp) => {
                let code = resp.status().as_u16();
                let reason = resp.status().canonical_reason().unwrap_or("");
                let mut block = format!("HTTP/1.1 {code} {reason}\r\n").into_bytes();
                for (name, value) in resp.headers().iter() {
                    block.extend_from_slice(name.as_str().as_bytes());
                    block.extend_from_slice(b": ");
                    block.extend_from_slice(value.as_bytes());
                    block.extend_from_slice(b"\r\n");
                }
                block.extend_from_slice(b"\r\n");
                st.last.http_code = code as i64;
                st.last.content_type = resp
                    .headers()
                    .get("content-type")
                    .map(|v| v.as_bytes().to_vec());
                st.last.header_block = block;
                // curl has no body-size cap; lift ureq's 10MB read default.
                let body = match resp.body_mut().with_config().limit(u64::MAX).read_to_vec() {
                    Ok(b) => b,
                    Err(e) => {
                        let (errno, msg) = map_transport_error(&e, &url_host(&url_str));
                        st.last.total_time_us = started.elapsed().as_micros() as i64;
                        fail(&mut st, errno, msg);
                        return None;
                    }
                };
                st.last.total_time_us = started.elapsed().as_micros() as i64;
                st.last.size_download = body.len() as i64;
                if st.fail_on_error && code >= 400 {
                    fail(
                        &mut st,
                        22,
                        format!("The requested URL returned error: {code}"),
                    );
                    return None;
                }
                Some(body)
            }
            Err(e) => {
                let (errno, msg) = map_transport_error(&e, &url_host(&url_str));
                st.last.total_time_us = started.elapsed().as_micros() as i64;
                fail(&mut st, errno, msg);
                None
            }
        }
    })();

    let result = match outcome {
        None => Zval::Bool(false),
        Some(body) => {
            let mut payload = Vec::new();
            if st.include_header {
                payload.extend_from_slice(&st.last.header_block);
            }
            payload.extend_from_slice(&body);
            if st.return_transfer {
                Zval::Str(PhpStr::new(payload))
            } else {
                ctx.out.extend_from_slice(&payload);
                Zval::Bool(true)
            }
        }
    };
    // Publish results back to the table (config fields are unchanged).
    with_state(id, "curl_exec", |s| {
        s.errno = st.errno;
        s.error = st.error.clone();
        s.last = st.last.clone();
    })?;
    Ok(result)
}

/// `__curl_errno(id) -> int`.
pub fn __curl_errno(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_errno")?;
    with_state(id, "curl_errno", |s| Zval::Long(s.errno))
}

/// `__curl_error(id) -> string`.
pub fn __curl_error(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_error")?;
    with_state(id, "curl_error", |s| {
        Zval::Str(PhpStr::new(s.error.clone().into_bytes()))
    })
}

/// `__curl_reset(id)`: restore every option (and the transfer state) to defaults.
pub fn __curl_reset(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_reset")?;
    with_state(id, "curl_reset", |s| *s = CurlState::default())?;
    Ok(Zval::Null)
}

/// `curl_close($handle)`: a no-op since PHP 8.0, deprecated in 8.5. A host
/// builtin (not a prelude function) so the Deprecated diag is attributed to
/// the caller's file/line, like the engine's.
pub fn curl_close(_argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    ctx.diags.push(Diag::Deprecated(
        "Function curl_close() is deprecated since 8.5, as it has no effect since PHP 8.0"
            .to_string(),
    ));
    Ok(Zval::Null)
}

/// `curl_strerror($errno) -> string`.
pub fn curl_strerror(argv: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let n = argv.first().map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    let msg = if (0..100).contains(&n) {
        CURL_STRERROR[n as usize]
    } else {
        "Unknown error"
    };
    Ok(Zval::Str(PhpStr::new(msg.as_bytes().to_vec())))
}

fn zstr(s: &[u8]) -> Zval {
    Zval::Str(PhpStr::new(s.to_vec()))
}

/// `__curl_getinfo(id, option|null)`: the full oracle-shaped array with no
/// option, a single value otherwise.
pub fn __curl_getinfo(argv: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let id = handle_id(argv, "curl_getinfo")?;
    let opt = match argv.get(1).map(|v| v.deref_clone()) {
        None | Some(Zval::Null) => None,
        Some(Zval::Long(n)) => Some(n),
        Some(_) => {
            return Err(PhpError::TypeError(
                "curl_getinfo(): Argument #2 ($option) must be of type ?int".to_string(),
            ))
        }
    };
    with_state(id, "curl_getinfo", |s| {
        let total_time = s.last.total_time_us as f64 / 1_000_000.0;
        let header_size = s.last.header_block.len() as i64;
        match opt {
            None => {
                // Key order and defaults mirror the brew PHP 8.5 oracle's
                // curl_getinfo() dump for a fresh handle.
                let mut a = PhpArray::new();
                let mut put = |k: &[u8], v: Zval| {
                    let _ = a.insert(Key::from_bytes(k), v);
                };
                put(b"url", zstr(&s.url));
                put(
                    b"content_type",
                    match &s.last.content_type {
                        Some(ct) => zstr(ct),
                        None => Zval::Null,
                    },
                );
                put(b"http_code", Zval::Long(s.last.http_code));
                put(b"header_size", Zval::Long(header_size));
                put(b"request_size", Zval::Long(s.last.request_size));
                put(b"filetime", Zval::Long(-1));
                put(b"ssl_verify_result", Zval::Long(0));
                put(b"redirect_count", Zval::Long(0));
                put(b"total_time", Zval::Double(total_time));
                put(b"namelookup_time", Zval::Double(0.0));
                put(b"connect_time", Zval::Double(0.0));
                put(b"pretransfer_time", Zval::Double(0.0));
                put(b"size_upload", Zval::Double(0.0));
                put(b"size_download", Zval::Double(s.last.size_download as f64));
                put(b"speed_download", Zval::Double(0.0));
                put(b"speed_upload", Zval::Double(0.0));
                put(b"download_content_length", Zval::Double(-1.0));
                put(b"upload_content_length", Zval::Double(-1.0));
                put(b"starttransfer_time", Zval::Double(0.0));
                put(b"redirect_time", Zval::Double(0.0));
                put(b"redirect_url", zstr(b""));
                put(b"primary_ip", zstr(b""));
                put(b"certinfo", Zval::Array(Rc::new(PhpArray::new())));
                put(b"primary_port", Zval::Long(-1));
                put(b"local_ip", zstr(b""));
                put(b"local_port", Zval::Long(-1));
                put(b"http_version", Zval::Long(if s.last.http_code > 0 { 2 } else { 0 }));
                put(b"protocol", Zval::Long(0));
                put(b"ssl_verifyresult", Zval::Long(0));
                put(b"scheme", zstr(&s.last.scheme));
                put(b"appconnect_time_us", Zval::Long(0));
                put(b"queue_time_us", Zval::Long(0));
                put(b"connect_time_us", Zval::Long(0));
                put(b"namelookup_time_us", Zval::Long(0));
                put(b"pretransfer_time_us", Zval::Long(0));
                put(b"redirect_time_us", Zval::Long(0));
                put(b"starttransfer_time_us", Zval::Long(0));
                put(b"posttransfer_time_us", Zval::Long(0));
                put(b"total_time_us", Zval::Long(s.last.total_time_us));
                put(
                    b"effective_method",
                    if s.last.effective_method.is_empty() {
                        zstr(b"GET")
                    } else {
                        zstr(&s.last.effective_method)
                    },
                );
                put(b"capath", zstr(b""));
                put(b"cainfo", zstr(b""));
                put(b"used_proxy", Zval::Long(0));
                put(b"httpauth_used", Zval::Long(0));
                put(b"proxyauth_used", Zval::Long(0));
                put(b"conn_id", Zval::Long(-1));
                Ok(Zval::Array(Rc::new(a)))
            }
            Some(CURLINFO_EFFECTIVE_URL) => Ok(zstr(&s.url)),
            Some(CURLINFO_HTTP_CODE) => Ok(Zval::Long(s.last.http_code)),
            Some(CURLINFO_CONTENT_TYPE) => Ok(match &s.last.content_type {
                Some(ct) => zstr(ct),
                None => Zval::Null,
            }),
            Some(CURLINFO_HEADER_SIZE) => Ok(Zval::Long(header_size)),
            Some(CURLINFO_TOTAL_TIME) => Ok(Zval::Double(total_time)),
            Some(CURLINFO_TOTAL_TIME_T) => Ok(Zval::Long(s.last.total_time_us)),
            Some(CURLINFO_SIZE_DOWNLOAD) => Ok(Zval::Double(s.last.size_download as f64)),
            Some(CURLINFO_REDIRECT_COUNT) => Ok(Zval::Long(0)),
            Some(CURLINFO_REDIRECT_URL) => Ok(zstr(b"")),
            Some(CURLINFO_SCHEME) => Ok(zstr(&s.last.scheme)),
            Some(CURLINFO_EFFECTIVE_METHOD) => Ok(if s.last.effective_method.is_empty() {
                zstr(b"GET")
            } else {
                zstr(&s.last.effective_method)
            }),
            Some(CURLINFO_PRIVATE) => Ok(match &s.private_data {
                Some(v) => v.clone(),
                None => Zval::Bool(false),
            }),
            // Valid-but-unmodelled selectors read as their fresh-handle default
            // rather than erroring (libcurl never fails a known getinfo).
            Some(_) => Ok(Zval::Bool(false)),
        }
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strerror_table_matches_oracle_spot_checks() {
        assert_eq!(CURL_STRERROR[0], "No error");
        assert_eq!(CURL_STRERROR[6], "Could not resolve hostname");
        assert_eq!(CURL_STRERROR[28], "Timeout was reached");
        assert_eq!(CURL_STRERROR[60], "SSL peer certificate or SSH remote key was not OK");
    }

    #[test]
    fn url_host_extraction() {
        assert_eq!(url_host("https://example.com/x?y"), "example.com");
        assert_eq!(url_host("http://user:pw@h.tld:8080/p"), "h.tld");
        assert_eq!(url_host("http://localhost:9111"), "localhost");
    }

    #[test]
    fn multipart_body_shape() {
        let b = multipart_body(
            &[(b"a".to_vec(), b"1".to_vec())],
            "XYZ",
        );
        let s = String::from_utf8(b).unwrap();
        assert!(s.starts_with("--XYZ\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\n1\r\n"));
        assert!(s.ends_with("--XYZ--\r\n"));
    }
}
