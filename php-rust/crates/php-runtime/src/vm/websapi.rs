//! Web-SAPI request plumbing (phpr -S, the cli-server work-alike): the
//! request superglobals ($_SERVER/$_GET/$_POST/$_COOKIE/$_FILES/$_REQUEST),
//! the rfc1867 multipart parser, and the stateful header-family helpers.
//! Every format here is oracle-pinned against `php -S` 8.5.7 (see the
//! sapi-probe battery in the WP-4 session notes).

use super::*;
use php_types::sapi::WebRequest;

// ---------------------------------------------------------------------------
// Variable assembly (php_register_variable_ex work-alike)
// ---------------------------------------------------------------------------

/// Percent-decode with `+` → space (same table as `ho_parse_str`).
fn urldecode(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < s.len() {
        match s[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < s.len() => {
                let hi = (s[i + 1] as char).to_digit(16);
                let lo = (s[i + 2] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push((h * 16 + l) as u8);
                    i += 2;
                } else {
                    out.push(b'%');
                }
            }
            c => out.push(c),
        }
        i += 1;
    }
    out
}

/// Split `name[a][b]…` into the mangled top-level base and the bracket path
/// (`None` = the `[]` append form) — the exact `ho_parse_str` rules, including
/// the unterminated-`[` fold into the base name.
fn split_key(key: &[u8]) -> (Vec<u8>, Vec<Option<Vec<u8>>>) {
    let (base_end, path): (usize, Vec<Option<Vec<u8>>>) =
        match key.iter().position(|&b| b == b'[') {
            None => (key.len(), Vec::new()),
            Some(p) => {
                let mut segs = Vec::new();
                let mut i = p;
                while i < key.len() && key[i] == b'[' {
                    match key[i + 1..].iter().position(|&b| b == b']') {
                        Some(off) => {
                            let seg = &key[i + 1..i + 1 + off];
                            segs.push(if seg.is_empty() { None } else { Some(seg.to_vec()) });
                            i += off + 2;
                        }
                        None => break,
                    }
                }
                (p, segs)
            }
        };
    // PHP mangles `.` and ` ` to `_` in the top-level variable name.
    let mut base: Vec<u8> = key[..base_end]
        .iter()
        .map(|&b| if b == b'.' || b == b' ' { b'_' } else { b })
        .collect();
    if path.is_empty() && base_end < key.len() {
        // Unterminated `[`: the first bracket becomes `_`, the rest joins verbatim.
        base.push(b'_');
        base.extend_from_slice(&key[base_end + 1..]);
    }
    (base, path)
}

/// Walk/create the nested arrays for a bracket path and store `val` at the
/// leaf. `overwrite=false` keeps an existing leaf (cookie first-wins).
fn descend(arr: &mut PhpArray, path: &[Option<Vec<u8>>], val: Zval, overwrite: bool) {
    let (head, rest) = path.split_first().expect("non-empty path");
    if rest.is_empty() {
        match head {
            Some(k) => {
                let key = Key::from_bytes(k);
                if overwrite || arr.get(&key).is_none() {
                    arr.insert(key, val);
                }
            }
            None => {
                let _ = arr.append(val);
            }
        }
        return;
    }
    let key = match head {
        Some(k) => Key::from_bytes(k),
        None => {
            // `k[]` mid-path: the next free int index (append form).
            let mut next = 0i64;
            for (k, _) in arr.iter() {
                if let Key::Int(i) = k {
                    if i >= next {
                        next = i + 1;
                    }
                }
            }
            Key::Int(next)
        }
    };
    let mut entry = match arr.get(&key) {
        Some(Zval::Array(a)) => (**a).clone(),
        _ => PhpArray::new(),
    };
    descend(&mut entry, rest, val, overwrite);
    arr.insert(key, Zval::Array(Rc::new(entry)));
}

/// Register one decoded `key=value` pair into `result` (bracket nesting,
/// top-level mangling). Shared by the query, cookie and multipart paths.
fn assign_parsed_var(result: &mut PhpArray, key: &[u8], val: Zval, overwrite: bool) {
    let (base, mut path) = split_key(key);
    if base.is_empty() {
        return;
    }
    if path.is_empty() {
        let k = Key::from_bytes(&base);
        if overwrite || result.get(&k).is_none() {
            result.insert(k, val);
        }
        return;
    }
    path.insert(0, Some(base));
    descend(result, &path, val, overwrite);
}

/// Parse a URL query string (`a=1&b[]=2`) into an array — GET/POST form data.
pub(super) fn parse_query(src: &[u8]) -> PhpArray {
    let mut result = PhpArray::new();
    for pair in src.split(|&b| b == b'&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_val) = match pair.iter().position(|&b| b == b'=') {
            Some(eq) => (&pair[..eq], &pair[eq + 1..]),
            None => (pair, &pair[pair.len()..]),
        };
        let key = urldecode(raw_key);
        let val = Zval::Str(PhpStr::new(urldecode(raw_val)));
        assign_parsed_var(&mut result, &key, val, true);
    }
    result
}

/// Parse a `Cookie:` header value. Oracle-pinned asymmetries vs the query
/// form: pairs split on `;` with left-trimmed spaces, the NAME is *not*
/// percent-decoded (only mangled/nested), the value is; the first occurrence
/// of a name wins.
pub(super) fn parse_cookies(src: &[u8]) -> PhpArray {
    let mut result = PhpArray::new();
    for pair in src.split(|&b| b == b';') {
        let pair = {
            let start = pair.iter().position(|&b| b != b' ' && b != b'\t').unwrap_or(pair.len());
            &pair[start..]
        };
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_val) = match pair.iter().position(|&b| b == b'=') {
            Some(eq) => (&pair[..eq], &pair[eq + 1..]),
            None => (pair, &pair[pair.len()..]),
        };
        let val = Zval::Str(PhpStr::new(urldecode(raw_val)));
        assign_parsed_var(&mut result, raw_key, val, false);
    }
    result
}

// ---------------------------------------------------------------------------
// rfc1867 multipart/form-data
// ---------------------------------------------------------------------------

/// The boundary parameter of a `multipart/form-data` content type.
fn multipart_boundary(ctype: &[u8]) -> Option<Vec<u8>> {
    let lower = ctype.to_ascii_lowercase();
    let pos = memmem(&lower, b"boundary=")?;
    let rest = &ctype[pos + 9..];
    let rest = if rest.first() == Some(&b'"') {
        let end = rest[1..].iter().position(|&b| b == b'"')?;
        &rest[1..1 + end]
    } else {
        let end = rest.iter().position(|&b| b == b';' || b == b' ').unwrap_or(rest.len());
        &rest[..end]
    };
    (!rest.is_empty()).then(|| rest.to_vec())
}

fn memmem(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// A `name="…"` / `filename="…"` parameter of a Content-Disposition line.
fn cd_param(cd: &[u8], param: &[u8]) -> Option<Vec<u8>> {
    let lower = cd.to_ascii_lowercase();
    let mut pat = param.to_vec();
    pat.extend_from_slice(b"=\"");
    let pos = memmem(&lower, &pat)?;
    let rest = &cd[pos + pat.len()..];
    let end = rest.iter().position(|&b| b == b'"')?;
    Some(rest[..end].to_vec())
}

/// One unique upload tmp path (`phpXXXX…` in the system tmp dir), plus the
/// registry entry so `is_uploaded_file`/`move_uploaded_file` see it.
fn create_upload_tmp(data: &[u8]) -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    // PHP reports the realpath'd tmp dir in tmp_name (`/private/var/…` on
    // macOS, not the `/var/…` symlink).
    let dir = std::env::temp_dir();
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    let base = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for attempt in 0u32..100 {
        let mut suffix = format!("{:x}{:x}{:x}", std::process::id(), base, attempt);
        suffix.truncate(21);
        let path = dir.join(format!("php{suffix}"));
        if std::fs::OpenOptions::new().write(true).create_new(true).open(&path).is_ok() {
            if std::fs::write(&path, data).is_err() {
                return None;
            }
            let bytes = path.as_os_str().as_bytes().to_vec();
            php_types::sapi::add_uploaded_file(bytes.clone());
            return Some(bytes);
        }
    }
    None
}

/// Strip client path components (PHP keeps the basename for `name`).
fn upload_basename(filename: &[u8]) -> Vec<u8> {
    let cut = filename
        .iter()
        .rposition(|&b| b == b'/' || b == b'\\')
        .map(|p| p + 1)
        .unwrap_or(0);
    filename[cut..].to_vec()
}

/// Parse a multipart/form-data body into (`$_POST` fields, `$_FILES`).
/// The `$_FILES` shape is oracle-pinned: per file `name`/`full_path`/`type`/
/// `tmp_name`/`error`/`size` in that order; a bracketed field name nests each
/// attribute (`nested[a]` → `$_FILES['nested']['name']['a']`).
pub(super) fn parse_multipart(body: &[u8], ctype: &[u8]) -> Option<(PhpArray, PhpArray)> {
    let boundary = multipart_boundary(ctype)?;
    let mut delim = b"--".to_vec();
    delim.extend_from_slice(&boundary);
    let mut post = PhpArray::new();
    let mut files = PhpArray::new();

    let mut pos = memmem(body, &delim)? + delim.len();
    loop {
        // After the delimiter: `--` closes the stream, CRLF opens a part.
        if body[pos..].starts_with(b"--") {
            break;
        }
        let part_start = match memmem(&body[pos..], b"\r\n") {
            Some(p) => pos + p + 2,
            None => break,
        };
        // The part runs until CRLF + the next delimiter.
        let mut sep = b"\r\n".to_vec();
        sep.extend_from_slice(&delim);
        let part_end = match memmem(&body[part_start..], &sep) {
            Some(p) => part_start + p,
            None => break,
        };
        let part = &body[part_start..part_end];
        pos = part_end + sep.len();

        // Headers / payload split.
        let (head, payload) = match memmem(part, b"\r\n\r\n") {
            Some(p) => (&part[..p], &part[p + 4..]),
            None => (part, &b""[..]),
        };
        let mut cd: &[u8] = b"";
        let mut ptype: Option<Vec<u8>> = None;
        for line in head.split(|&b| b == b'\n') {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            let Some(colon) = line.iter().position(|&b| b == b':') else { continue };
            let name = line[..colon].to_ascii_lowercase();
            let mut value = &line[colon + 1..];
            while value.first() == Some(&b' ') {
                value = &value[1..];
            }
            if name == b"content-disposition" {
                cd = &line[colon + 1..];
            } else if name == b"content-type" {
                ptype = Some(value.to_vec());
            }
        }
        let Some(field) = cd_param(cd, b"name") else { continue };
        match cd_param(cd, b"filename") {
            None => {
                // Plain field → $_POST (raw name: multipart names are not
                // percent-encoded).
                assign_parsed_var(
                    &mut post,
                    &field,
                    Zval::Str(PhpStr::new(payload.to_vec())),
                    true,
                );
            }
            Some(filename) => {
                // File. An empty filename is the no-file case (UPLOAD_ERR_NO_FILE).
                let (name_v, full_v, type_v, tmp_v, err_v, size_v) = if filename.is_empty() {
                    (Vec::new(), Vec::new(), Vec::new(), Vec::new(), 4i64, 0i64)
                } else {
                    let tmp = create_upload_tmp(payload);
                    match tmp {
                        Some(tmp) => (
                            upload_basename(&filename),
                            filename.clone(),
                            ptype.clone().unwrap_or_default(),
                            tmp,
                            0,
                            payload.len() as i64,
                        ),
                        // Could not create the tmp file: UPLOAD_ERR_CANT_WRITE.
                        None => (
                            upload_basename(&filename),
                            filename.clone(),
                            ptype.clone().unwrap_or_default(),
                            Vec::new(),
                            7,
                            0,
                        ),
                    }
                };
                let attrs: [(&[u8], Zval); 6] = [
                    (b"name", Zval::Str(PhpStr::new(name_v))),
                    (b"full_path", Zval::Str(PhpStr::new(full_v))),
                    (b"type", Zval::Str(PhpStr::new(type_v))),
                    (b"tmp_name", Zval::Str(PhpStr::new(tmp_v))),
                    (b"error", Zval::Long(err_v)),
                    (b"size", Zval::Long(size_v)),
                ];
                let (base, path) = split_key(&field);
                if base.is_empty() {
                    continue;
                }
                let base_key = Key::from_bytes(&base);
                let mut entry = match files.get(&base_key) {
                    Some(Zval::Array(a)) => (**a).clone(),
                    _ => PhpArray::new(),
                };
                for (attr, val) in attrs {
                    if path.is_empty() {
                        entry.insert(Key::from_bytes(attr), val);
                    } else {
                        let mut sub = match entry.get(&Key::from_bytes(attr)) {
                            Some(Zval::Array(a)) => (**a).clone(),
                            _ => PhpArray::new(),
                        };
                        descend(&mut sub, &path, val, true);
                        entry.insert(Key::from_bytes(attr), Zval::Array(Rc::new(sub)));
                    }
                }
                files.insert(base_key, Zval::Array(Rc::new(entry)));
            }
        }
    }
    Some((post, files))
}

// ---------------------------------------------------------------------------
// Superglobal seeding
// ---------------------------------------------------------------------------

/// A request header's value, first occurrence (case-insensitive).
fn req_header<'a>(req: &'a WebRequest, name: &[u8]) -> Option<&'a [u8]> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_slice())
}

/// Seed the web-SAPI superglobals from the installed request — key set and
/// order oracle-pinned against `php -S` (sapi_cli_server_register_variables).
pub(super) fn seed_web_superglobals(superglobals: &mut [Zval; 8], req: &WebRequest) {
    use std::os::unix::ffi::OsStrExt;
    let str_zval = |b: &[u8]| Zval::Str(PhpStr::new(b.to_vec()));

    // --- $_SERVER ---
    let mut s = PhpArray::new();
    s.insert(Key::from_bytes(b"DOCUMENT_ROOT"), str_zval(&req.doc_root));
    s.insert(Key::from_bytes(b"REMOTE_ADDR"), str_zval(req.remote_addr.as_bytes()));
    s.insert(
        Key::from_bytes(b"REMOTE_PORT"),
        str_zval(req.remote_port.to_string().as_bytes()),
    );
    s.insert(
        Key::from_bytes(b"SERVER_SOFTWARE"),
        str_zval(b"PHP/8.5.7 (Development Server)"),
    );
    s.insert(
        Key::from_bytes(b"SERVER_PROTOCOL"),
        str_zval(format!("HTTP/{}.{}", req.protocol.0, req.protocol.1).as_bytes()),
    );
    s.insert(Key::from_bytes(b"SERVER_NAME"), str_zval(req.server_host.as_bytes()));
    s.insert(
        Key::from_bytes(b"SERVER_PORT"),
        str_zval(req.server_port.to_string().as_bytes()),
    );
    s.insert(Key::from_bytes(b"REQUEST_URI"), str_zval(&req.request_uri));
    s.insert(Key::from_bytes(b"REQUEST_METHOD"), str_zval(&req.method));
    s.insert(Key::from_bytes(b"SCRIPT_NAME"), str_zval(&req.vpath));
    s.insert(Key::from_bytes(b"SCRIPT_FILENAME"), str_zval(&req.script_filename));
    if let Some(pi) = &req.path_info {
        s.insert(Key::from_bytes(b"PATH_INFO"), str_zval(pi));
        let mut self_path = req.vpath.clone();
        self_path.extend_from_slice(pi);
        s.insert(Key::from_bytes(b"PHP_SELF"), str_zval(&self_path));
    } else {
        s.insert(Key::from_bytes(b"PHP_SELF"), str_zval(&req.vpath));
    }
    if let Some(q) = &req.query_string {
        s.insert(Key::from_bytes(b"QUERY_STRING"), str_zval(q));
    }
    // Request headers → HTTP_* (wire order). Content-Type/Content-Length also
    // register their CGI names, before the HTTP_ spelling (oracle-pinned).
    for (name, value) in &req.headers {
        let mut mangled: Vec<u8> = name
            .iter()
            .map(|&b| if b == b'-' { b'_' } else { b.to_ascii_uppercase() })
            .collect();
        if mangled == b"CONTENT_TYPE" || mangled == b"CONTENT_LENGTH" {
            s.insert(Key::from_bytes(&mangled), str_zval(value));
        }
        let mut http = b"HTTP_".to_vec();
        http.append(&mut mangled);
        s.insert(Key::from_bytes(&http), str_zval(value));
    }
    s.insert(Key::from_bytes(b"REQUEST_TIME_FLOAT"), Zval::Double(req.request_time));
    s.insert(Key::from_bytes(b"REQUEST_TIME"), Zval::Long(req.request_time as i64));

    // --- $_GET / $_POST / $_FILES / $_COOKIE ---
    let get = req
        .query_string
        .as_deref()
        .map(parse_query)
        .unwrap_or_default();
    let ctype = req_header(req, b"content-type").unwrap_or(b"");
    let ctype_lower = ctype.to_ascii_lowercase();
    let (post, files) = if req.method == b"POST" {
        if ctype_lower.starts_with(b"application/x-www-form-urlencoded") {
            (parse_query(&req.body), PhpArray::new())
        } else if ctype_lower.starts_with(b"multipart/form-data") {
            parse_multipart(&req.body, ctype).unwrap_or_default()
        } else {
            (PhpArray::new(), PhpArray::new())
        }
    } else {
        (PhpArray::new(), PhpArray::new())
    };
    let cookies = req_header(req, b"cookie").map(parse_cookies).unwrap_or_default();

    // --- $_REQUEST (request_order=GP: GET then POST overrides) ---
    let mut request = PhpArray::new();
    for (k, v) in get.iter() {
        request.insert(k.clone(), v.clone());
    }
    for (k, v) in post.iter() {
        request.insert(k.clone(), v.clone());
    }

    // --- $_ENV (variables_order EGPCS keeps E) ---
    let mut env = PhpArray::new();
    for (k, v) in std::env::vars_os() {
        env.insert(
            Key::from_bytes(k.as_os_str().as_bytes()),
            Zval::Str(PhpStr::new(v.as_os_str().as_bytes().to_vec())),
        );
    }

    for (i, name) in crate::bytecode::SUPERGLOBAL_NAMES.iter().enumerate() {
        superglobals[i] = match *name {
            b"_SERVER" => Zval::Array(Rc::new(s.clone())),
            b"_GET" => Zval::Array(Rc::new(get.clone())),
            b"_POST" => Zval::Array(Rc::new(post.clone())),
            b"_ENV" => Zval::Array(Rc::new(env.clone())),
            b"_FILES" => Zval::Array(Rc::new(files.clone())),
            b"_COOKIE" => Zval::Array(Rc::new(cookies.clone())),
            b"_REQUEST" => Zval::Array(Rc::new(request.clone())),
            // `$_SESSION` stays unset until session_start.
            _ => Zval::Undef,
        };
    }
}

// ---------------------------------------------------------------------------
// Response header machinery (web branches of the header family)
// ---------------------------------------------------------------------------

/// `epoch` (UTC) as PHP's cookie/HTTP date (shared with the server host).
use php_types::sapi::http_date;

impl<'m> Vm<'m> {
    /// The header name of a full `Name: value` line (empty when malformed).
    fn header_line_name(line: &[u8]) -> &[u8] {
        match line.iter().position(|&b| b == b':') {
            Some(p) => &line[..p],
            None => b"",
        }
    }

    /// Store one script header line with PHP's replace semantics: `replace`
    /// REMOVES every same-name line and appends the new one at the END
    /// (sapi_header_op SAPI_HEADER_REPLACE = llist delete + add; oracle-pinned
    /// — the WP feed's late Content-Type re-set lands after Last-Modified).
    pub(super) fn web_set_header(&mut self, line: Vec<u8>, replace: bool) {
        let name = Self::header_line_name(&line).to_ascii_lowercase();
        if replace && !name.is_empty() {
            self.response_headers
                .retain(|h| !Self::header_line_name(h).eq_ignore_ascii_case(&name));
        }
        self.response_headers.push(line);
    }

    /// `header()` under the web SAPI. The cli-server buffers the whole
    /// response, so there is no "headers already sent" state (oracle-pinned).
    pub(super) fn web_header(&mut self, args: &[Zval]) -> Result<Zval, PhpError> {
        let line = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        if line.iter().any(|&b| b == b'\r' || b == b'\n' || b == 0) {
            return Err(PhpError::ValueError(
                "header(): Header content must not contain any header injection characters \
                 like \\r, \\n or \\0"
                    .to_string(),
            ));
        }
        // Once the sink crossed the cli-server write buffer the headers are on
        // the wire: warn and drop, like PHP (WP-10 hs probe).
        if self.output_started {
            let msg = self.headers_sent_warning();
            self.diags.push(php_types::Diag::Warning(msg));
            return Ok(Zval::Null);
        }
        let replace = match args.get(1) {
            Some(v) => convert::to_bool(v, &mut self.diags),
            None => true,
        };
        // A raw status line: `header("HTTP/1.1 404 Not Found")`.
        if line.len() >= 5 && line[..5].eq_ignore_ascii_case(b"HTTP/") {
            let mut it = line.splitn(3, |&b| b == b' ');
            let _proto = it.next();
            if let Some(code) = it.next() {
                let code: i64 = String::from_utf8_lossy(code).trim().parse().unwrap_or(0);
                if code > 0 {
                    self.response_code = Some(code);
                    self.response_reason = it.next().map(|r| r.to_vec());
                }
            }
            return Ok(Zval::Null);
        }
        let name = Self::header_line_name(&line);
        if name.is_empty() {
            // No colon: PHP rejects the header (SAPI_HEADER_ADD without colon).
            return Ok(Zval::Null);
        }
        // `Location:` implies 302 unless a non-200 code was already chosen.
        if name.eq_ignore_ascii_case(b"Location")
            && matches!(self.response_code, None | Some(200))
        {
            self.response_code = Some(302);
            self.response_reason = None;
        }
        self.web_set_header(line, replace);
        // The explicit third argument overrides any implied code.
        if let Some(v) = args.get(2) {
            let code = convert::to_long_cast(v, &mut self.diags);
            if code != 0 {
                self.response_code = Some(code);
                self.response_reason = None;
            }
        }
        Ok(Zval::Null)
    }

    /// `setcookie` / `setrawcookie` under the web SAPI: build the
    /// `Set-Cookie:` line (oracle-pinned attribute order) and append it.
    pub(super) fn web_setcookie(&mut self, args: &[Zval], raw: bool) -> Result<Zval, PhpError> {
        let func = if raw { "setrawcookie" } else { "setcookie" };
        if self.output_started {
            let msg = self.headers_sent_warning();
            self.diags.push(php_types::Diag::Warning(msg));
            return Ok(Zval::Bool(false));
        }
        let name = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        if name.is_empty() {
            return Err(PhpError::ValueError(format!(
                "{func}(): Argument #1 ($name) cannot be empty"
            )));
        }
        if name.iter().any(|b| b"=,; \t\r\n\x0b\x0c".contains(b)) {
            return Err(PhpError::ValueError(format!(
                "{func}(): Argument #1 ($name) cannot contain \"=\", \",\", \";\", \" \", \
                 \"\\t\", \"\\r\", \"\\n\", \"\\013\", or \"\\014\""
            )));
        }
        let value = args
            .get(1)
            .map(|v| convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        if raw && value.iter().any(|b| b",; \t\r\n\x0b\x0c".contains(b)) {
            self.diags.push(Diag::Warning(format!(
                "{func}(): Argument #2 ($value) cannot contain \",\", \";\", \" \", \"\\t\", \
                 \"\\r\", \"\\n\", \"\\013\", or \"\\014\""
            )));
            return Ok(Zval::Bool(false));
        }
        // Third arg: expires int, or the options array.
        let mut expires: i64 = 0;
        let mut path = Vec::new();
        let mut domain = Vec::new();
        let mut secure = false;
        let mut httponly = false;
        let mut samesite = Vec::new();
        match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Array(opts)) => {
                for (k, v) in opts.iter() {
                    let kname = match k {
                        Key::Str(s) => s.as_bytes().to_vec(),
                        Key::Int(_) => continue,
                    };
                    match kname.as_slice() {
                        b"expires" => expires = convert::to_long_cast(v, &mut self.diags),
                        b"path" => {
                            path = convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec()
                        }
                        b"domain" => {
                            domain = convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec()
                        }
                        b"secure" => secure = convert::to_bool(v, &mut self.diags),
                        b"httponly" => httponly = convert::to_bool(v, &mut self.diags),
                        b"samesite" => {
                            samesite =
                                convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec()
                        }
                        other => {
                            self.diags.push(Diag::Warning(format!(
                                "{func}(): Argument #3 ($expires_or_options) contains an \
                                 unrecognized key \"{}\"",
                                String::from_utf8_lossy(other)
                            )));
                        }
                    }
                }
            }
            Some(v) => {
                expires = convert::to_long_cast(&v, &mut self.diags);
                path = args
                    .get(3)
                    .map(|v| convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec())
                    .unwrap_or_default();
                domain = args
                    .get(4)
                    .map(|v| convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec())
                    .unwrap_or_default();
                secure = args.get(5).map(|v| convert::to_bool(v, &mut self.diags)).unwrap_or(false);
                httponly =
                    args.get(6).map(|v| convert::to_bool(v, &mut self.diags)).unwrap_or(false);
            }
            None => {}
        }
        let line = build_set_cookie(&name, &value, raw, expires, &path, &domain, secure, httponly, &samesite);
        self.response_headers.push(line);
        Ok(Zval::Bool(true))
    }

    /// The web cache-limiter + session cookie headers `session_start` sends
    /// (default `nocache` limiter; cookie per session.cookie_* ini) —
    /// oracle-pinned bytes and order.
    pub(super) fn web_session_headers(&mut self, send_cookie: bool) {
        if send_cookie && self.ini.get_bool(b"session.use_cookies") {
            let name = self.ini.get(b"session.name").unwrap_or(b"PHPSESSID").to_vec();
            let path = self.ini.get(b"session.cookie_path").unwrap_or(b"/").to_vec();
            let domain = self.ini.get(b"session.cookie_domain").unwrap_or(b"").to_vec();
            let lifetime = self.ini.get_long(b"session.cookie_lifetime");
            let secure = self.ini.get_bool(b"session.cookie_secure");
            let httponly = self.ini.get_bool(b"session.cookie_httponly");
            let samesite = self.ini.get(b"session.cookie_samesite").unwrap_or(b"").to_vec();
            let expires = if lifetime > 0 {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0)
                    + lifetime
            } else {
                0
            };
            let id = self.session.id.clone();
            let line = build_set_cookie(
                &name, &id, false, expires, &path, &domain, secure, httponly, &samesite,
            );
            self.response_headers.push(line);
        }
        match self.ini.get(b"session.cache_limiter").unwrap_or(b"nocache") {
            b"nocache" => {
                self.response_headers
                    .push(b"Expires: Thu, 19 Nov 1981 08:52:00 GMT".to_vec());
                self.response_headers
                    .push(b"Cache-Control: no-store, no-cache, must-revalidate".to_vec());
                self.response_headers.push(b"Pragma: no-cache".to_vec());
            }
            // public/private/private_no_expire need real Expires math; sent
            // rarely — modelled as their Cache-Control line only.
            b"public" => self
                .response_headers
                .push(b"Cache-Control: public".to_vec()),
            b"private" | b"private_no_expire" => self
                .response_headers
                .push(b"Cache-Control: private, must-revalidate".to_vec()),
            _ => {}
        }
    }
}

/// The value encoding of `setcookie` (php_raw_url_encode: RFC 3986 unreserved
/// kept, everything else `%XX`, space included).
fn raw_url_encode(v: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(v.len());
    for &b in v {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b);
        } else {
            out.extend_from_slice(format!("%{b:02X}").as_bytes());
        }
    }
    out
}

/// Assemble one `Set-Cookie:` line (oracle-pinned: value rawurlencoded unless
/// raw; an empty value sends the `deleted` tombstone; attribute order
/// expires/Max-Age/path/domain/secure/HttpOnly/SameSite).
#[allow(clippy::too_many_arguments)]
fn build_set_cookie(
    name: &[u8],
    value: &[u8],
    raw: bool,
    expires: i64,
    path: &[u8],
    domain: &[u8],
    secure: bool,
    httponly: bool,
    samesite: &[u8],
) -> Vec<u8> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut line = b"Set-Cookie: ".to_vec();
    line.extend_from_slice(name);
    line.push(b'=');
    let (value, expires) = if value.is_empty() && !raw {
        (b"deleted".to_vec(), 1)
    } else {
        (
            if raw { value.to_vec() } else { raw_url_encode(value) },
            expires,
        )
    };
    line.extend_from_slice(&value);
    if expires > 0 {
        line.extend_from_slice(b"; expires=");
        line.extend_from_slice(http_date(expires).as_bytes());
        line.extend_from_slice(b"; Max-Age=");
        line.extend_from_slice((expires - now).max(0).to_string().as_bytes());
    }
    if !path.is_empty() {
        line.extend_from_slice(b"; path=");
        line.extend_from_slice(path);
    }
    if !domain.is_empty() {
        line.extend_from_slice(b"; domain=");
        line.extend_from_slice(domain);
    }
    if secure {
        line.extend_from_slice(b"; secure");
    }
    if httponly {
        line.extend_from_slice(b"; HttpOnly");
    }
    if !samesite.is_empty() {
        line.extend_from_slice(b"; SameSite=");
        line.extend_from_slice(samesite);
    }
    line
}
