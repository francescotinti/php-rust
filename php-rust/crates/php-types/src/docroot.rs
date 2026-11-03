//! Docroot resolution shared by every web SAPI host — the `phpr -S` cli-server
//! work-alike (php-cli) and the php-server axum front end — so both serve the
//! same paths the same way. Oracle-pinned against `php -S` 8.5.7 (WP-4/5/6
//! sapi-probe battery, WP-61 WordPress acceptance): longest existing file
//! prefix wins (the remainder is PATH_INFO for scripts), a directory tries
//! `index.php` then `index.html`, an unresolved path falls back to the DOCROOT
//! `index.php` with the whole decoded path as PATH_INFO (WordPress' virtual
//! routes: `/2026/07/hello-world/`, `/wp-json/`, `/robots.txt`), plus the
//! cli-server mime map, reason phrases, 404 template and asctime log stamp.
//!
//! Moved here from php-cli in S-188 (Tappa 2): php-server's `axum-server`
//! feature deliberately does NOT depend on php-cli (matrix identity A-AH16),
//! and php-types is already a dependency of every configuration.

use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use crate::mime::MIME_TYPE_MAP;

/// The 404 page of the cli-server, byte-identical (URI interpolated).
pub const ERROR_PAGE_HEAD: &str = "<!doctype html><html><head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>404 Not Found</title><style>\nbody { background-color: #fcfcfc; color: #333333; margin: 0; padding:0; }\nh1 { font-size: 1.5em; font-weight: normal; background-color: #9999cc; min-height:2em; line-height:2em; border-bottom: 1px inset black; margin: 0; }\nh1, p { padding-left: 10px; }\ncode.url { background-color: #eeeeee; font-family:monospace; padding:0 2px;}\n</style>\n</head><body><h1>Not Found</h1><p>The requested resource <code class=\"url\">";
pub const ERROR_PAGE_TAIL: &str = "</code> was not found on this server.</p></body></html>";

/// The cli-server 404 body for a decoded request path.
pub fn not_found_page(decoded: &[u8]) -> Vec<u8> {
    let mut page = ERROR_PAGE_HEAD.as_bytes().to_vec();
    page.extend_from_slice(decoded);
    page.extend_from_slice(ERROR_PAGE_TAIL.as_bytes());
    page
}

/// PHP's reason-phrase table (main/http_status_codes.h).
pub fn status_reason(code: i64) -> &'static str {
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
pub fn mime_for_ext(ext: &[u8]) -> Option<&'static str> {
    let ext = String::from_utf8_lossy(&ext.to_ascii_lowercase()).into_owned();
    MIME_TYPE_MAP
        .binary_search_by(|(e, _)| (*e).cmp(ext.as_str()))
        .ok()
        .map(|i| MIME_TYPE_MAP[i].1)
}

/// Local time as asctime ("Tue Jul 14 17:24:55 2026" — day space-padded),
/// the cli-server log timestamp.
pub fn asctime_local() -> String {
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

/// Percent-decode a URL *path* (no `+` → space — that is query-only).
pub fn percent_decode_path(s: &[u8]) -> Vec<u8> {
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
pub fn normalize_path(path: &[u8]) -> Vec<u8> {
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

/// Split a request target into (path, query): a bare trailing `?` registers
/// no QUERY_STRING (oracle-pinned).
pub fn split_target(target: &[u8]) -> (&[u8], Option<Vec<u8>>) {
    match target.iter().position(|&b| b == b'?') {
        Some(p) if p + 1 < target.len() => (&target[..p], Some(target[p + 1..].to_vec())),
        Some(p) => (&target[..p], None),
        None => (target, None),
    }
}

/// What the docroot walk resolved the request path to.
#[derive(Debug)]
pub enum Resolved {
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
pub fn translate(docroot: &Path, path: &[u8]) -> Resolved {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway docroot shaped like a WordPress install.
    fn tree() -> PathBuf {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("phpr-docroot-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("wp-admin")).unwrap();
        std::fs::create_dir_all(root.join("wp-includes/css")).unwrap();
        std::fs::create_dir_all(root.join("empty")).unwrap();
        std::fs::write(root.join("index.php"), "<?php echo 'root';").unwrap();
        std::fs::write(root.join("wp-admin/index.php"), "<?php echo 'admin';").unwrap();
        std::fs::write(root.join("wp-includes/css/buttons.css"), "body{}").unwrap();
        std::fs::write(root.join("a.php"), "<?php echo 'a';").unwrap();
        std::fs::write(root.join("UP.PHP"), "<?php echo 'up';").unwrap();
        root
    }

    fn script(r: Resolved) -> (PathBuf, Vec<u8>, Option<Vec<u8>>) {
        match r {
            Resolved::Script(f, v, p) => (f, v, p),
            other => panic!("expected Script, got {other:?}"),
        }
    }

    #[test]
    fn permalink_falls_back_to_root_index_with_path_info() {
        let root = tree();
        let (file, vpath, pi) = script(translate(&root, b"/2026/07/hello-world/"));
        assert_eq!(file, root.join("index.php"));
        assert_eq!(vpath, b"/index.php");
        assert_eq!(pi.as_deref(), Some(&b"/2026/07/hello-world/"[..]));
    }

    #[test]
    fn directory_serves_its_index_php() {
        let root = tree();
        let (file, vpath, pi) = script(translate(&root, b"/wp-admin/"));
        assert_eq!(file, root.join("wp-admin/index.php"));
        assert_eq!(vpath, b"/wp-admin/index.php");
        assert!(pi.is_none());
        // root itself
        let (file, vpath, _) = script(translate(&root, b"/"));
        assert_eq!(file, root.join("index.php"));
        assert_eq!(vpath, b"/index.php");
    }

    #[test]
    fn static_file_and_static_with_trailing_segments() {
        let root = tree();
        match translate(&root, b"/wp-includes/css/buttons.css") {
            Resolved::Static(f) => assert_eq!(f, root.join("wp-includes/css/buttons.css")),
            other => panic!("expected Static, got {other:?}"),
        }
        // a static file with extra segments is not a script: the walk says
        // NotFound and the root index.php takes it as PATH_INFO (oracle).
        let (file, _, pi) = script(translate(&root, b"/wp-includes/css/buttons.css/x"));
        assert_eq!(file, root.join("index.php"));
        assert_eq!(pi.as_deref(), Some(&b"/wp-includes/css/buttons.css/x"[..]));
    }

    #[test]
    fn script_with_path_info_and_uppercase_extension() {
        let root = tree();
        let (file, vpath, pi) = script(translate(&root, b"/a.php/extra/seg"));
        assert_eq!(file, root.join("a.php"));
        assert_eq!(vpath, b"/a.php");
        assert_eq!(pi.as_deref(), Some(&b"/extra/seg"[..]));
        let (file, vpath, _) = script(translate(&root, b"/UP.PHP"));
        assert_eq!(file, root.join("UP.PHP"));
        assert_eq!(vpath, b"/UP.PHP");
    }

    #[test]
    fn empty_dir_without_index_falls_back_to_root() {
        let root = tree();
        let (file, _, pi) = script(translate(&root, b"/empty/"));
        assert_eq!(file, root.join("index.php"));
        assert_eq!(pi.as_deref(), Some(&b"/empty/"[..]));
        // and a docroot WITHOUT a root index.php reports NotFound
        std::fs::remove_file(root.join("index.php")).unwrap();
        assert!(matches!(translate(&root, b"/nope/"), Resolved::NotFound));
    }

    #[test]
    fn decode_and_normalize_clamp_at_root() {
        assert_eq!(percent_decode_path(b"/%2e%2e/x%20y"), b"/../x y");
        assert_eq!(normalize_path(&percent_decode_path(b"/%2e%2e/x")), b"/x");
        assert_eq!(normalize_path(b"/../x"), b"/x");
        assert_eq!(normalize_path(b"/a/./b/../c/"), b"/a/c/");
        assert_eq!(normalize_path(b""), b"/");
        assert_eq!(percent_decode_path(b"/a%zz"), b"/a%zz");
    }

    #[test]
    fn split_target_bare_question_mark_has_no_query() {
        assert_eq!(split_target(b"/x?"), (&b"/x"[..], None));
        assert_eq!(split_target(b"/x?a=1&b"), (&b"/x"[..], Some(b"a=1&b".to_vec())));
        assert_eq!(split_target(b"/x"), (&b"/x"[..], None));
        assert_eq!(split_target(b"/?feed=rss2"), (&b"/"[..], Some(b"feed=rss2".to_vec())));
    }

    #[test]
    fn mime_status_and_404_page() {
        assert_eq!(mime_for_ext(b"CSS"), Some("text/css"));
        assert_eq!(mime_for_ext(b"js"), Some("application/javascript")); // PHP's map, not the IANA update
        assert_eq!(mime_for_ext(b"nope-ext"), None);
        assert_eq!(status_reason(404), "Not Found");
        assert_eq!(status_reason(999), "Unknown Status Code");
        let page = not_found_page(b"/missing");
        assert!(page.starts_with(ERROR_PAGE_HEAD.as_bytes()));
        assert!(page.ends_with(ERROR_PAGE_TAIL.as_bytes()));
        assert!(page.windows(8).any(|w| w == b"/missing"));
        let stamp = asctime_local();
        assert_eq!(stamp.len(), 24, "{stamp}");
    }
}
