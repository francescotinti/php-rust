//! Host builtins (the ~200 ho_* methods backing PHP standard-library
//! functions that need VM state). Split from vm/mod.rs; the host_builtins!
//! dispatch macro stays in mod.rs. Structural move only.

use super::*;

/// How the `array_(u)diff/(u)intersect` family compares one dimension (value/key):
/// ignored, by `(string)` equality (INTERNAL), or by a user comparator (USER).
#[derive(Clone, Copy)]
enum DiffCmp {
    Ignore,
    Standard,
    Callback,
}

/// A socket connection error as `(errno, errstr)`, stripping Rust's " (os error
/// N)" suffix so the message matches PHP's.
fn sock_err(e: &std::io::Error) -> (i64, String) {
    let errno = e.raw_os_error().unwrap_or(0) as i64;
    let msg = e.to_string();
    let msg = msg.split(" (os error").next().unwrap_or(&msg).to_string();
    (errno, msg)
}

/// The `[result, errno, errstr]` triple the socket host builtins return for their
/// prelude wrappers to spread into by-ref `&$errno`/`&$errstr`.
fn socket_result(result: Zval, errno: i64, errstr: Vec<u8>) -> Zval {
    let mut out = PhpArray::new();
    let _ = out.append(result);
    let _ = out.append(Zval::Long(errno));
    let _ = out.append(Zval::Str(PhpStr::new(errstr)));
    Zval::Array(Rc::new(out))
}

/// The lower-cased URL scheme of `spec` (bytes before `://`), or "" when there is
/// none (a plain path).
fn url_scheme(spec: &[u8]) -> String {
    match spec.windows(3).position(|w| w == b"://") {
        Some(i) => String::from_utf8_lossy(&spec[..i]).to_ascii_lowercase(),
        None => String::new(),
    }
}

/// Whether `scheme` (lower-cased) is a built-in wrapper phpr recognises, so
/// `stream_wrapper_register` refuses to redefine it (matching PHP).
fn is_builtin_scheme(scheme: &[u8]) -> bool {
    matches!(
        scheme,
        b"file" | b"php" | b"http" | b"https" | b"ftp" | b"ftps" | b"data" | b"glob"
            | b"phar" | b"zip" | b"compress.zlib" | b"compress.bzip2"
    )
}

/// An array key as the bytes PHP would use for it (int keys become their decimal
/// text), for indexing stream-context wrapper/option sub-arrays.
fn key_bytes(k: &php_types::Key) -> Vec<u8> {
    match k {
        php_types::Key::Str(s) => s.as_bytes().to_vec(),
        php_types::Key::Int(i) => i.to_string().into_bytes(),
    }
}

/// Set `options[wrapper][option] = value` on a stream-context resource, creating
/// the wrapper sub-array if absent and preserving other options.
fn ctx_set_one(rc: &Rc<RefCell<php_types::Resource>>, wrapper: &[u8], option: &[u8], value: Zval) {
    let mut b = rc.borrow_mut();
    let Some(Zval::Array(opts_rc)) = b.context_options_mut() else { return };
    let opts = Rc::make_mut(opts_rc);
    let wkey = php_types::Key::from_bytes(wrapper);
    match opts.get_mut(&wkey) {
        Some(Zval::Array(sub_rc)) => {
            Rc::make_mut(sub_rc).insert(php_types::Key::from_bytes(option), value);
        }
        _ => {
            let mut sub = PhpArray::new();
            sub.insert(php_types::Key::from_bytes(option), value);
            opts.insert(wkey, Zval::Array(Rc::new(sub)));
        }
    }
}

impl<'m> super::Vm<'m> {
    /// `gc_collect_cycles()` — force a cycle collection now, regardless of the
    /// root-buffer threshold. Returns the number of destroyed objects.
    pub(super) fn ho_gc_collect_cycles(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        let n = self.collect_cycles()?;
        Ok(Zval::Long(n))
    }
    /// `iterator_to_array(iterable $it, bool $preserve_keys = true): array`
    /// (step 56b): collect an array / Generator / Traversable object into an
    /// array, reusing the same protocol-driver as spread. With `$preserve_keys`
    /// false the values are reindexed 0..n.
    pub(super) fn ho_iterator_to_array(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let src = args.first().cloned().unwrap_or(Zval::Null);
        let preserve = args.get(1).is_none_or(|v| convert::to_bool(v, &mut self.diags));
        let pairs = self.iter_pairs(src)?;
        let mut out = PhpArray::new();
        for (k, v) in pairs {
            if preserve {
                out.insert(k, v);
            } else {
                let _ = out.append(v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `iterator_count(iterable $it): int` (step 56b).
    pub(super) fn ho_iterator_count(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let src = args.first().cloned().unwrap_or(Zval::Null);
        Ok(Zval::Long(self.iter_pairs(src)?.len() as i64))
    }
    /// `json_encode($value, $flags = 0)` (step 56c): first normalise the value so
    /// every `JsonSerializable` object is replaced by its `jsonSerialize()` result
    /// and every enum case by its backing value (recursively), then hand off to
    /// the pure registry encoder which formats the now-method-free value. A
    /// non-backed enum has no representation: `json_last_error()` becomes
    /// `JSON_ERROR_NON_BACKED_ENUM` and the call returns `false`, or throws a
    /// `JsonException` under `JSON_THROW_ON_ERROR`.
    pub(super) fn ho_json_encode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.json_last_error = 0; // JSON_ERROR_NONE
        let value = args.first().cloned().unwrap_or(Zval::Null);
        let flags = args
            .get(1)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(0);
        let throw = flags & 4_194_304 != 0; // JSON_THROW_ON_ERROR
        // JSON_PARTIAL_OUTPUT_ON_ERROR: unencodable nodes become null (0 for
        // INF/NAN) instead of failing the whole encode — the cycle handling
        // moves from the up-front check to per-node substitution (normalize
        // for JsonSerializable/array revisits, the pure encoder for plain
        // object graphs).
        let partial = flags & 512 != 0;
        // A json_encode() re-entered from within a jsonSerialize() still running
        // for the SAME object is JSON_ERROR_RECURSION (json_encode_recursion_01),
        // not an infinite descent — the nested call fails fast with `false`.
        if let Some(addr) = deref_object(&value).map(|o| Rc::as_ptr(&o) as usize) {
            if self.json_active.contains(&addr) {
                self.json_last_error = 6;
                if throw {
                    if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                        let obj = self.synthesize_throwable(cid, "Recursion detected")?;
                        return Err(PhpError::Thrown(obj));
                    }
                }
                return Ok(Zval::Bool(false));
            }
        }
        // A cyclic value graph is PHP's JSON_ERROR_RECURSION — detected up
        // front over the whole graph (arrays *and* plain-object properties,
        // which json_normalize deliberately does not descend into).
        if !partial && json_has_cycle(&value, &mut Vec::new()) {
            self.json_last_error = 6;
            if throw {
                if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                    let obj = self.synthesize_throwable(cid, "Recursion detected")?;
                    return Err(PhpError::Thrown(obj));
                }
            }
            return Ok(Zval::Bool(false));
        }
        let mut visiting = Vec::new();
        let normalized = match self.json_normalize(value, partial, &mut visiting) {
            Ok(n) => n,
            Err(e) => {
                if self.json_last_error == 11 || self.json_last_error == 6 {
                    // Non-backed enum / recursion: false, or a JsonException.
                    if throw {
                        if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                            let msg = if self.json_last_error == 6 {
                                "Recursion detected"
                            } else {
                                "Non-backed enums have no default serialization"
                            };
                            let obj = self.synthesize_throwable(cid, msg)?;
                            return Err(PhpError::Thrown(obj));
                        }
                    }
                    return Ok(Zval::Bool(false));
                }
                return Err(e);
            }
        };
        let f = match self.registry.get(&b"json_encode"[..]) {
            Some(Builtin::Value(f)) => *f,
            _ => return Err(PhpError::Error("json_encode builtin unavailable".to_string())),
        };
        let mut call_args = vec![normalized];
        if let Some(flags) = args.get(1) {
            call_args.push(flags.clone());
        }
        let line = self.cur_line(self.frames.len() - 1);
        let result = self.run_value_builtin(f, &call_args, line)?;
        // The pure encoder reports failure as `false` without a cause; by far
        // the most common one (the cycle/enum cases are pre-checked above) is
        // malformed UTF-8 — JSON_ERROR_UTF8, or a JsonException under
        // JSON_THROW_ON_ERROR.
        fn has_nonfinite(v: &Zval) -> bool {
            match v {
                Zval::Double(d) => !d.is_finite(),
                Zval::Array(a) => a.iter().any(|(_, e)| has_nonfinite(e)),
                Zval::Object(o) => o.borrow().props.iter().any(|(_, e)| has_nonfinite(e)),
                Zval::Ref(r) => has_nonfinite(&r.borrow()),
                _ => false,
            }
        }
        if matches!(result, Zval::Bool(false)) && self.json_last_error == 0 {
            // Distinguish the two silent-encoder failures: a non-finite float
            // is JSON_ERROR_INF_OR_NAN (7), anything else is UTF-8 (5).
            let inf = call_args.first().is_some_and(has_nonfinite);
            self.json_last_error = if inf { 7 } else { 5 };
            let msg = if inf {
                "Inf and NaN cannot be JSON encoded"
            } else {
                "Malformed UTF-8 characters, possibly incorrectly encoded"
            };
            if throw {
                if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                    let obj = self.synthesize_throwable(cid, msg)?;
                    return Err(PhpError::Thrown(obj));
                }
            }
        }
        // JSON_PARTIAL_OUTPUT_ON_ERROR substitutes the offending node (INF/NAN → 0)
        // and still returns a string, but json_last_error() must report that the
        // error occurred (inf_nan_error).
        if partial && self.json_last_error == 0 && call_args.first().is_some_and(has_nonfinite) {
            self.json_last_error = 7;
        }
        Ok(result)
    }
    /// `json_last_error()`: the error code of the most recent JSON operation.
    pub(super) fn ho_json_last_error(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !args.is_empty() {
            return Err(PhpError::TypeError(format!(
                "json_last_error() expects exactly 0 arguments, {} given",
                args.len()
            )));
        }
        Ok(Zval::Long(self.json_last_error))
    }
    /// `json_last_error_msg()`: the human-readable message for that error code.
    pub(super) fn ho_json_last_error_msg(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !args.is_empty() {
            return Err(PhpError::TypeError(format!(
                "json_last_error_msg() expects exactly 0 arguments, {} given",
                args.len()
            )));
        }
        let msg: &[u8] = match self.json_last_error {
            0 => b"No error",
            1 => b"Maximum stack depth exceeded",
            2 => b"State mismatch (invalid or malformed JSON)",
            3 => b"Control character error, possibly incorrectly encoded",
            4 => b"Syntax error",
            5 => b"Malformed UTF-8 characters, possibly incorrectly encoded",
            6 => b"Recursion detected",
            7 => b"Inf and NaN cannot be JSON encoded",
            8 => b"Type is not supported",
            9 => b"The decoded property name is invalid",
            10 => b"Single unpaired UTF-16 surrogate in unicode escape",
            11 => b"Non-backed enums have no default serialization",
            _ => b"Unknown error",
        };
        Ok(Zval::Str(PhpStr::new(msg.to_vec())))
    }
    /// `assert($assertion, $description = null)`: when `$assertion` is falsy,
    /// throw (the engine default `assert.exception=1`). A `Throwable` description
    /// is rethrown as-is; any other description (including the source text the
    /// lowerer injects for `assert($expr)`) becomes the `AssertionError` message.
    /// A truthy assertion returns `true`.
    pub(super) fn ho_assert(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let assertion = args.first().cloned().unwrap_or(Zval::Null);
        if convert::to_bool(&assertion, &mut self.diags) {
            return Ok(Zval::Bool(true));
        }
        match args.get(1) {
            // A Throwable description is thrown unchanged.
            Some(obj @ Zval::Object(_)) => Err(PhpError::Thrown(obj.clone())),
            Some(desc) => {
                let msg = convert::to_zstr_cast(desc, &mut self.diags).as_bytes().to_vec();
                self.throw_assertion_error(&msg)
            }
            None => self.throw_assertion_error(b"assert()"),
        }
    }
    /// `ob_start($callback = null, $chunk_size = 0, $flags = ...)`: push a new
    /// output buffer. Subsequent output is captured into it until a matching
    /// `ob_get_clean`/`ob_end_*`. The optional callback is invoked with the
    /// buffered content when the buffer is flushed. Returns `true`.
    pub(super) fn ho_ob_start(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let callback = match args.first() {
            Some(Zval::Null) | None => None,
            Some(cb) => Some(cb.clone()),
        };
        // `chunk_size`: a negative or absent value means "no chunking" (0).
        let chunk_size = match args.get(1) {
            Some(v) => convert::to_long_cast(v, &mut self.diags).max(0) as usize,
            None => 0,
        };
        self.ob_stack
            .push(OutputBuffer { content: Vec::new(), callback, chunk_size, started: false });
        Ok(Zval::Bool(true))
    }
    /// `ob_get_contents()`: the current buffer's content as a string, or `false`
    /// if output buffering is not active.
    pub(super) fn ho_ob_get_contents(&mut self) -> Result<Zval, PhpError> {
        Ok(match self.ob_stack.last() {
            Some(b) => Zval::Str(PhpStr::new(b.content.clone())),
            None => Zval::Bool(false),
        })
    }
    /// `ob_get_clean()`: return the current buffer's content and discard the
    /// buffer (no flush to the parent). `false` if no buffer is active.
    pub(super) fn ho_ob_get_clean(&mut self) -> Result<Zval, PhpError> {
        Ok(match self.ob_stack.pop() {
            Some(b) => Zval::Str(PhpStr::new(b.content)),
            None => Zval::Bool(false),
        })
    }
    /// `ob_get_flush()`: return the current buffer's content *and* flush it to the
    /// underlying sink, then remove the buffer. `false` (with a notice) if none is
    /// active.
    pub(super) fn ho_ob_get_flush(&mut self) -> Result<Zval, PhpError> {
        let Some(buf) = self.ob_stack.pop() else {
            self.ob_no_buffer_notice("ob_get_flush(): Failed to delete and flush buffer. No buffer to delete or flush")?;
            return Ok(Zval::Bool(false));
        };
        let content = buf.content.clone();
        self.flush_buffer(buf)?;
        Ok(Zval::Str(PhpStr::new(content)))
    }
    /// `ob_end_clean()`: discard the current buffer (no flush). `true` on success,
    /// `false` (with a notice) if no buffer is active.
    pub(super) fn ho_ob_end_clean(&mut self) -> Result<Zval, PhpError> {
        if self.ob_stack.pop().is_some() {
            Ok(Zval::Bool(true))
        } else {
            self.ob_no_buffer_notice("ob_end_clean(): Failed to delete buffer. No buffer to delete")?;
            Ok(Zval::Bool(false))
        }
    }
    /// `ob_end_flush()`: flush the current buffer to the underlying sink and
    /// remove it. `true` on success, `false` (with a notice) if no buffer is active.
    pub(super) fn ho_ob_end_flush(&mut self) -> Result<Zval, PhpError> {
        match self.ob_stack.pop() {
            Some(buf) => {
                self.flush_buffer(buf)?;
                Ok(Zval::Bool(true))
            }
            None => {
                self.ob_no_buffer_notice("ob_end_flush(): Failed to delete and flush buffer. No buffer to delete or flush")?;
                Ok(Zval::Bool(false))
            }
        }
    }
    /// `ob_flush()`: send the current buffer's content to the underlying sink but
    /// keep the buffer active (cleared). `true` on success, `false` (with a notice)
    /// if no buffer is active.
    pub(super) fn ho_ob_flush(&mut self) -> Result<Zval, PhpError> {
        // A manual flush runs the handler with `PHP_OUTPUT_HANDLER_FLUSH` (4) (plus
        // START on the first flush) and keeps the buffer active, emptied.
        if self.ob_stack.last().is_some() {
            self.emit_buffer_op(4)?;
            Ok(Zval::Bool(true))
        } else {
            self.ob_no_buffer_notice("ob_flush(): Failed to flush buffer. No buffer to flush")?;
            Ok(Zval::Bool(false))
        }
    }
    /// `ob_clean()`: discard the current buffer's content but keep the buffer
    /// active. `true` on success, `false` (with a notice) if no buffer is active.
    pub(super) fn ho_ob_clean(&mut self) -> Result<Zval, PhpError> {
        if let Some(buf) = self.ob_stack.last_mut() {
            buf.content.clear();
            Ok(Zval::Bool(true))
        } else {
            self.ob_no_buffer_notice("ob_clean(): Failed to delete buffer. No buffer to delete")?;
            Ok(Zval::Bool(false))
        }
    }
    /// `json_decode($json, $assoc = false)` (F2): parse JSON via the shared
    /// [`crate::json`] parser, returning `null` on a parse error (JSON_THROW_ON_ERROR
    /// is a scope-out). Objects become arrays when `$assoc` is true, `stdClass`
    /// otherwise; the `depth`/`flags` arguments are ignored. Mirrors
    /// `eval::ho_json_decode`. Records the JSON error state for `json_last_error()`.
    pub(super) fn ho_json_decode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "json_decode() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let json = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let assoc = match args.get(1) {
            Some(v) => convert::to_bool(v, &mut self.diags),
            None => false,
        };
        // `$depth` is the nesting limit (default 512); a non-positive one is a
        // ValueError before any parsing (json_decode_error).
        let depth = match args.get(2) {
            Some(v) => convert::to_long_cast(v, &mut self.diags),
            None => 512,
        };
        if depth <= 0 {
            return Err(PhpError::ValueError(
                "json_decode(): Argument #3 ($depth) must be greater than 0".to_string(),
            ));
        }
        let flags = args
            .get(3)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(0);
        // JSON must be valid UTF-8; malformed input is JSON_ERROR_UTF8 (not a
        // syntax error) and decodes to null — UNLESS JSON_INVALID_UTF8_IGNORE
        // (0x100000) / JSON_INVALID_UTF8_SUBSTITUTE (0x200000) ask to scrub it
        // first (that PHP-exact per-byte substitution is not modelled yet, so
        // those flagged inputs fall through to the parser).
        let utf8_scrub = flags & (0x10_0000 | 0x20_0000) != 0;
        if !utf8_scrub && std::str::from_utf8(&json).is_err() {
            self.json_last_error = 5; // JSON_ERROR_UTF8
            if flags & 4_194_304 != 0 {
                if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                    let obj = self.synthesize_throwable(
                        cid,
                        "Malformed UTF-8 characters, possibly incorrectly encoded",
                    )?;
                    return Err(PhpError::Thrown(obj));
                }
            }
            return Ok(Zval::Null);
        }
        let (err_code, err_msg) = match crate::json::parse_depth(&json, depth.min(u32::MAX as i64) as u32) {
            Ok(j) => {
                self.json_last_error = 0; // JSON_ERROR_NONE
                return self.vm_json_to_zval(&j, assoc);
            }
            Err(crate::json::JsonError::Depth) => (1, "Maximum stack depth exceeded"),
            Err(crate::json::JsonError::Syntax) => (4, "Syntax error"),
        };
        self.json_last_error = err_code;
        if flags & 4_194_304 != 0 {
            // JSON_THROW_ON_ERROR
            if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                let obj = self.synthesize_throwable(cid, err_msg)?;
                return Err(PhpError::Thrown(obj));
            }
        }
        Ok(Zval::Null)
    }
    /// `json_validate(string $json, int $depth = 512, int $flags = 0): bool` —
    /// whether `$json` is a syntactically valid JSON document, using the same
    /// parser as `json_decode` (and recording `json_last_error` identically)
    /// without materializing the value. `$depth` must be positive; the only
    /// accepted flag is `JSON_INVALID_UTF8_IGNORE` (0x100000).
    pub(super) fn ho_json_validate(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "json_validate() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let json = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        // Argument order mirrors PHP's observable contract for `json_last_error`:
        // the `$flags` ValueError is raised BEFORE the error state is reset (so a
        // prior error survives it), whereas the `$depth` ValueError is raised
        // AFTER the reset (so it leaves JSON_ERROR_NONE behind).
        let flags = args
            .get(2)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(0);
        // json_validate accepts only JSON_INVALID_UTF8_IGNORE.
        if flags & !0x10_0000 != 0 {
            return Err(PhpError::ValueError(
                "json_validate(): Argument #3 ($flags) must be a valid flag \
                 (allowed flags: JSON_INVALID_UTF8_IGNORE)"
                    .to_string(),
            ));
        }
        self.json_last_error = 0; // JSON_ERROR_NONE
        // An empty string is a syntax error decided BEFORE the `$depth` validation
        // (PHP short-circuits zero-length input, so `json_validate("", -1)` is
        // `false`/JSON_ERROR_SYNTAX, not a depth ValueError).
        if json.is_empty() {
            self.json_last_error = 4; // JSON_ERROR_SYNTAX
            return Ok(Zval::Bool(false));
        }
        let depth = match args.get(1) {
            Some(v) => convert::to_long_cast(v, &mut self.diags),
            None => 512,
        };
        if depth <= 0 {
            return Err(PhpError::ValueError(
                "json_validate(): Argument #2 ($depth) must be greater than 0".to_string(),
            ));
        }
        if depth > i32::MAX as i64 {
            return Err(PhpError::ValueError(
                "json_validate(): Argument #2 ($depth) must be less than 2147483647".to_string(),
            ));
        }
        let utf8_ignore = flags & 0x10_0000 != 0;
        if !utf8_ignore && std::str::from_utf8(&json).is_err() {
            self.json_last_error = 5; // JSON_ERROR_UTF8
            return Ok(Zval::Bool(false));
        }
        match crate::json::parse_depth(&json, depth.min(u32::MAX as i64) as u32) {
            Ok(_) => {
                self.json_last_error = 0; // JSON_ERROR_NONE
                Ok(Zval::Bool(true))
            }
            Err(crate::json::JsonError::Depth) => {
                self.json_last_error = 1; // JSON_ERROR_DEPTH
                Ok(Zval::Bool(false))
            }
            Err(crate::json::JsonError::Syntax) => {
                self.json_last_error = 4; // JSON_ERROR_SYNTAX
                Ok(Zval::Bool(false))
            }
        }
    }
    /// Whether `name` is a syntactically valid PHP variable label: a leading
    /// letter/underscore or high byte, then letters/digits/underscore/high bytes.
    fn is_valid_php_label(name: &[u8]) -> bool {
        match name.first() {
            Some(&c) if c == b'_' || c.is_ascii_alphabetic() || c >= 0x80 => {}
            _ => return false,
        }
        name[1..].iter().all(|&c| c == b'_' || c.is_ascii_alphanumeric() || c >= 0x80)
    }

    /// Read a named local of the caller's frame (named slot then dynamic
    /// side-table), following references; `None` if unset. No warning — the
    /// callers (`compact`/`extract`) decide their own diagnostics.
    fn frame_local(&self, top: usize, name: &[u8]) -> Option<Zval> {
        // `$this` lives in the frame header, not the named slots.
        if name == b"this" {
            return self.frames[top].this.clone();
        }
        if let Some(s) = self.frames[top].func.slot_names.iter().position(|n| n.as_ref() == name) {
            let v = self.frames[top].slots[s].deref_clone();
            if !matches!(v, Zval::Undef) {
                return Some(v);
            }
        }
        self.frames[top].dyn_vars.get(name).map(|v| v.deref_clone())
    }

    /// One `compact()` argument: a variable name (added to `out` if set, else a
    /// Warning) or, recursively, an array of names. A non-string/array argument
    /// warns with its top-level position `arg_num`. `seen` holds the identities of
    /// the arrays currently being walked so a self-referential array raises
    /// `Error: Recursion detected` instead of overflowing the stack.
    fn compact_add(
        &mut self,
        top: usize,
        item: &Zval,
        arg_num: usize,
        out: &mut PhpArray,
        seen: &mut Vec<usize>,
    ) -> Result<(), PhpError> {
        match item {
            Zval::Str(s) => {
                let name = s.as_bytes().to_vec();
                match self.frame_local(top, &name) {
                    Some(v) => {
                        out.insert(Key::from_bytes(&name), v);
                    }
                    // `compact("this")` outside an object scope is silently skipped
                    // (no "undefined variable" Warning), unlike any other name.
                    None if name == b"this" => {}
                    None => self.diags.push(Diag::Warning(format!(
                        "compact(): Undefined variable ${}",
                        String::from_utf8_lossy(&name)
                    ))),
                }
            }
            Zval::Array(a) => {
                let id = Rc::as_ptr(a) as usize;
                if seen.contains(&id) {
                    return Err(PhpError::Error("Recursion detected".to_string()));
                }
                seen.push(id);
                let items: Vec<Zval> = a.iter().map(|(_, v)| v.deref_clone()).collect();
                for it in &items {
                    self.compact_add(top, it, arg_num, out, seen)?;
                }
                seen.pop();
            }
            other => {
                // PHP 8 names bool values "true"/"false" (not "bool") in this message.
                let tname = match other {
                    Zval::Bool(true) => "true".to_string(),
                    Zval::Bool(false) => "false".to_string(),
                    _ => other.type_name_for_error().to_string(),
                };
                self.diags.push(Diag::Warning(format!(
                    "compact(): Argument #{arg_num} must be string or array of strings, {tname} given"
                )));
            }
        }
        Ok(())
    }

    /// `compact(mixed ...$var_names): array` — build an associative array of the
    /// named locals that are set in the caller's scope; each name is a string or
    /// (recursively) an array of strings. An undefined name warns and is skipped.
    pub(super) fn ho_compact(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        let mut out = PhpArray::new();
        let mut seen = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            self.compact_add(top, arg, i + 1, &mut out, &mut seen)?;
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `extract(array $array, int $flags = EXTR_OVERWRITE, string $prefix = ""): int`
    /// — import the array's entries as locals of the caller's scope, honouring the
    /// EXTR_* strategy, and return how many were imported. Integer keys and invalid
    /// labels are skipped unless a prefixing flag rescues them; `$this`/`$GLOBALS`
    /// are never overwritten. EXTR_REFS degrades to a value copy (phpr has no
    /// by-ref local aliasing here).
    pub(super) fn ho_extract(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        const EXTR_OVERWRITE: i64 = 0;
        const EXTR_SKIP: i64 = 1;
        const EXTR_PREFIX_SAME: i64 = 2;
        const EXTR_PREFIX_ALL: i64 = 3;
        const EXTR_PREFIX_INVALID: i64 = 4;
        const EXTR_PREFIX_IF_EXISTS: i64 = 5;
        const EXTR_IF_EXISTS: i64 = 6;
        const EXTR_REFS: i64 = 256;

        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "extract() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let Zval::Array(arr) = first else {
            return Err(PhpError::TypeError(format!(
                "extract(): Argument #1 ($array) must be of type array, {} given",
                first.type_name_for_error()
            )));
        };
        let arr = arr.clone();
        let flags = args
            .get(1)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(EXTR_OVERWRITE);
        let extr_type = flags & !EXTR_REFS;
        if !(EXTR_OVERWRITE..=EXTR_IF_EXISTS).contains(&extr_type) {
            return Err(PhpError::ValueError(
                "extract(): Argument #2 ($flags) must be a valid extract type".to_string(),
            ));
        }
        // The prefixing strategies require an explicit prefix argument.
        let is_prefix_type = (EXTR_PREFIX_SAME..=EXTR_PREFIX_IF_EXISTS).contains(&extr_type);
        if is_prefix_type && args.get(2).is_none() {
            return Err(PhpError::ValueError(
                "extract(): Argument #3 ($prefix) is required when using this extract type"
                    .to_string(),
            ));
        }
        let prefix = args
            .get(2)
            .map(|v| convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        // A non-empty prefix must itself be a valid identifier.
        if is_prefix_type && !prefix.is_empty() && !Self::is_valid_php_label(&prefix) {
            return Err(PhpError::ValueError(
                "extract(): Argument #3 ($prefix) must be a valid identifier".to_string(),
            ));
        }

        let top = self.frames.len() - 1;
        // Own the pairs before mutating the frame.
        let pairs: Vec<(Key, Zval)> = arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        let mut count: i64 = 0;
        for (key, value) in pairs {
            let (key_bytes, key_is_valid) = match &key {
                Key::Str(s) => (s.as_bytes().to_vec(), Self::is_valid_php_label(s.as_bytes())),
                Key::Int(i) => (i.to_string().into_bytes(), false),
            };
            let exists = key_is_valid && self.frame_local(top, &key_bytes).is_some();
            let prefixed = || {
                let mut n = prefix.clone();
                n.push(b'_');
                n.extend_from_slice(&key_bytes);
                n
            };
            let target: Option<Vec<u8>> = match extr_type {
                EXTR_OVERWRITE => key_is_valid.then(|| key_bytes.clone()),
                EXTR_SKIP => (key_is_valid && !exists).then(|| key_bytes.clone()),
                EXTR_PREFIX_SAME => {
                    // Only valid labels are eligible; an invalid/integer key is
                    // skipped (unlike PREFIX_ALL/PREFIX_INVALID). A collision with
                    // an existing local is resolved by prefixing.
                    if !key_is_valid {
                        None
                    } else if exists {
                        Some(prefixed())
                    } else {
                        Some(key_bytes.clone())
                    }
                }
                // PREFIX_ALL prefixes every key EXCEPT an empty string key, which
                // it skips outright (an integer key is still prefixed).
                EXTR_PREFIX_ALL => (!key_bytes.is_empty()).then(prefixed),
                EXTR_PREFIX_INVALID => {
                    if key_is_valid {
                        Some(key_bytes.clone())
                    } else {
                        Some(prefixed())
                    }
                }
                EXTR_IF_EXISTS => (key_is_valid && exists).then(|| key_bytes.clone()),
                EXTR_PREFIX_IF_EXISTS => (key_is_valid && exists).then(prefixed),
                _ => None,
            };
            if let Some(name) = target {
                if Self::is_valid_php_label(&name) && name != b"this" && name != b"GLOBALS" {
                    self.var_dyn_write(top, &name, value)?;
                    count += 1;
                }
            }
        }
        Ok(Zval::Long(count))
    }
    /// The "headers already sent by (output started at FILE:LINE)" message text
    /// for the header-family warnings, using the recorded first-output location.
    fn headers_sent_warning(&self) -> String {
        match &self.output_start {
            Some((f, l)) => format!(
                "Cannot modify header information - headers already sent by \
                 (output started at {}:{})",
                String::from_utf8_lossy(f),
                l
            ),
            None => "Cannot modify header information - headers already sent".to_string(),
        }
    }

    /// `headers_sent(&$filename = null, &$line = null): bool` — whether output has
    /// reached the sink (CLI). The by-reference out-parameters are not populated
    /// (the bare no-arg form is what real code uses).
    pub(super) fn ho_headers_sent(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        Ok(Zval::Bool(self.output_started))
    }

    /// `header(string $header, bool $replace = true, int $response_code = 0): void`
    /// — a CLI no-op; only warns if output has already been sent.
    pub(super) fn ho_header(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        if self.output_started {
            let msg = self.headers_sent_warning();
            self.diags.push(Diag::Warning(msg));
        }
        Ok(Zval::Null)
    }

    /// `headers_list(): array` — always empty under the CLI SAPI.
    pub(super) fn ho_headers_list(&mut self) -> Result<Zval, PhpError> {
        Ok(Zval::Array(Rc::new(PhpArray::new())))
    }

    /// `setcookie(...) / setrawcookie(...): bool` — a CLI no-op returning `true`,
    /// or `false` with the "headers already sent" Warning once output has started.
    pub(super) fn ho_setcookie(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        if self.output_started {
            let msg = self.headers_sent_warning();
            self.diags.push(Diag::Warning(msg));
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }

    /// `header_remove(?string $name = null): void` — a CLI no-op (warns if sent).
    pub(super) fn ho_header_remove(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        if self.output_started {
            let msg = self.headers_sent_warning();
            self.diags.push(Diag::Warning(msg));
        }
        Ok(Zval::Null)
    }

    /// `http_response_code(int $response_code = 0): int|bool` — get returns the
    /// stored code (or `false` if never set, under CLI); set stores it and returns
    /// the previous code (or `true` on the first set).
    pub(super) fn ho_http_response_code(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match args.first() {
            None | Some(Zval::Null) => Ok(match self.response_code {
                Some(c) => Zval::Long(c),
                None => Zval::Bool(false),
            }),
            Some(v) => {
                // Setting the code once output has started is refused (its own
                // message, distinct from header()'s).
                if self.output_started {
                    let loc = match &self.output_start {
                        Some((f, l)) => format!(" (output started at {}:{})", String::from_utf8_lossy(f), l),
                        None => String::new(),
                    };
                    self.diags.push(Diag::Warning(format!(
                        "http_response_code(): Cannot set response code - headers already sent{loc}"
                    )));
                    return Ok(Zval::Bool(false));
                }
                let code = convert::to_long_cast(v, &mut self.diags);
                let old = self.response_code.replace(code);
                Ok(match old {
                    Some(c) => Zval::Long(c),
                    None => Zval::Bool(true),
                })
            }
        }
    }

    /// `set_time_limit(int $seconds): bool` — the CLI SAPI has no execution-time
    /// limit, so this validates the argument and always succeeds (like php-cli).
    pub(super) fn ho_set_time_limit(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let _ = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        Ok(Zval::Bool(true))
    }

    /// `ignore_user_abort(?bool $enable = null): int` — CLI has no client
    /// connection, so this is a stored flag: return the previous value, and when
    /// `$enable` is passed (non-null) update it.
    pub(super) fn ho_ignore_user_abort(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let previous = self.user_abort_ignored as i64;
        if let Some(v) = args.first() {
            let v = v.deref_clone();
            if !matches!(v, Zval::Null) {
                self.user_abort_ignored = convert::to_bool(&v, &mut self.diags);
            }
        }
        Ok(Zval::Long(previous))
    }
    /// The type name PHP 8 uses in `TypeError` messages, which names the singleton
    /// values `true`/`false`/`null` rather than their type.
    fn err_type_name(v: &Zval) -> String {
        match v {
            Zval::Bool(true) => "true".to_string(),
            Zval::Bool(false) => "false".to_string(),
            Zval::Null => "null".to_string(),
            _ => v.type_name_for_error().to_string(),
        }
    }

    /// An array key as a `Zval` (for the key-comparator callbacks).
    fn key_as_zval(k: &Key) -> Zval {
        match k {
            Key::Int(i) => Zval::Long(*i),
            Key::Str(s) => Zval::Str(PhpStr::new(s.as_bytes().to_vec())),
        }
    }

    /// Shared engine for the whole callback-driven `array_(u)diff/(u)intersect`
    /// family. Each of value/key is compared as: ignored, by `(string)` equality
    /// (INTERNAL), or by a user comparator (USER, 0 == equal). Two entries "match"
    /// when both compared dimensions match. Keep each first-array entry that
    /// matches NO entry in any other array (diff) or SOME entry in every other
    /// array (intersect); order and keys of the first array are preserved.
    /// `n_callbacks` trailing args are the comparators (for `_uassoc`: value then
    /// key).
    #[allow(clippy::too_many_arguments)]
    fn array_diff_intersect(
        &mut self,
        mut args: Vec<Zval>,
        name: &str,
        intersect: bool,
        value_mode: DiffCmp,
        key_mode: DiffCmp,
        n_callbacks: usize,
    ) -> Result<Zval, PhpError> {
        let min_args = 1 + n_callbacks;
        if args.len() < min_args {
            return Err(PhpError::ArgumentCountError(format!(
                "{name}() expects at least {min_args} arguments, {} given",
                args.len()
            )));
        }
        // Peel off the comparators from the end: the last is the key comparator
        // when the key uses USER, otherwise the value comparator; a second one
        // (the `_uassoc` forms) is always the value comparator.
        let mut value_cb = None;
        let mut key_cb = None;
        if n_callbacks >= 1 {
            let cb = args.pop().unwrap().deref_clone();
            if matches!(key_mode, DiffCmp::Callback) {
                key_cb = Some(cb);
            } else {
                value_cb = Some(cb);
            }
        }
        if n_callbacks >= 2 {
            value_cb = Some(args.pop().unwrap().deref_clone());
        }
        // The remaining args must all be arrays; collect their entries by value.
        let mut arrays: Vec<Vec<(Key, Zval)>> = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            match a.deref_clone() {
                Zval::Array(arr) => {
                    arrays.push(arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect())
                }
                other => {
                    let arg = if i == 0 {
                        "Argument #1 ($array)".to_string()
                    } else {
                        format!("Argument #{}", i + 1)
                    };
                    return Err(PhpError::TypeError(format!(
                        "{name}(): {arg} must be of type array, {} given",
                        Self::err_type_name(&other)
                    )));
                }
            }
        }
        let first = arrays.remove(0);
        let others = arrays;
        let mut out = PhpArray::new();
        for (k, v) in first {
            let mut present_in_all = true;
            let mut present_in_any = false;
            for other in &others {
                let mut found = false;
                for (k2, v2) in other {
                    let value_ok = match value_mode {
                        DiffCmp::Ignore => true,
                        DiffCmp::Standard => {
                            convert::to_zstr_cast(&v, &mut self.diags).as_bytes()
                                == convert::to_zstr_cast(v2, &mut self.diags).as_bytes()
                        }
                        DiffCmp::Callback => {
                            self.compare_with_callback(value_cb.as_ref().unwrap(), &v, v2)? == 0
                        }
                    };
                    if !value_ok {
                        continue;
                    }
                    let key_ok = match key_mode {
                        DiffCmp::Ignore => true,
                        DiffCmp::Standard => k == *k2,
                        DiffCmp::Callback => {
                            self.compare_with_callback(
                                key_cb.as_ref().unwrap(),
                                &Self::key_as_zval(&k),
                                &Self::key_as_zval(k2),
                            )? == 0
                        }
                    };
                    if key_ok {
                        found = true;
                        break;
                    }
                }
                if found {
                    present_in_any = true;
                } else {
                    present_in_all = false;
                }
            }
            let keep = if intersect { present_in_all } else { !present_in_any };
            if keep {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    pub(super) fn ho_array_udiff(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_udiff", false, DiffCmp::Callback, DiffCmp::Ignore, 1)
    }
    pub(super) fn ho_array_uintersect(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_uintersect", true, DiffCmp::Callback, DiffCmp::Ignore, 1)
    }
    pub(super) fn ho_array_diff_ukey(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_diff_ukey", false, DiffCmp::Ignore, DiffCmp::Callback, 1)
    }
    pub(super) fn ho_array_intersect_ukey(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_intersect_ukey", true, DiffCmp::Ignore, DiffCmp::Callback, 1)
    }
    pub(super) fn ho_array_udiff_assoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_udiff_assoc", false, DiffCmp::Callback, DiffCmp::Standard, 1)
    }
    pub(super) fn ho_array_uintersect_assoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_uintersect_assoc", true, DiffCmp::Callback, DiffCmp::Standard, 1)
    }
    pub(super) fn ho_array_diff_uassoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_diff_uassoc", false, DiffCmp::Standard, DiffCmp::Callback, 1)
    }
    pub(super) fn ho_array_intersect_uassoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_intersect_uassoc", true, DiffCmp::Standard, DiffCmp::Callback, 1)
    }
    pub(super) fn ho_array_udiff_uassoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_udiff_uassoc", false, DiffCmp::Callback, DiffCmp::Callback, 2)
    }
    pub(super) fn ho_array_uintersect_uassoc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.array_diff_intersect(args, "array_uintersect_uassoc", true, DiffCmp::Callback, DiffCmp::Callback, 2)
    }
    /// `mb_split($pattern, $string[, $limit])` (F2): split on matches, keeping
    /// empty fields. `$limit > 0` caps the piece count. Returns `false` on a bad
    /// pattern. Mirrors `eval::ho_mb_split`.
    pub(super) fn ho_mb_split(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_split() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let limit = match args.get(2) {
            Some(v) => convert::to_long_cast(&v.deref_clone(), &mut self.diags),
            None => -1,
        };
        let opts = self.mb_regex.options.clone();
        let Some(re) = self.mb_compile(&pat, &opts, "mb_split", false) else {
            return Ok(Zval::Bool(false));
        };
        let mut arr = PhpArray::new();
        for p in crate::mbregex::split(&re, &subject, limit) {
            let _ = arr.append(Zval::Str(PhpStr::new(p)));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `mb_regex_encoding([$encoding])` (F2): getter returns the current name
    /// ("UTF-8" default); setter stores it and returns true. Only UTF-8 is
    /// effectively supported (D-MB-ereg-enc). Mirrors `eval::ho_mb_regex_encoding`.
    pub(super) fn ho_mb_regex_encoding(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone()))),
            Some(v) => {
                let v = v.deref_clone();
                if matches!(v, Zval::Null) {
                    return Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone())));
                }
                self.mb_regex.encoding =
                    convert::to_zstr_cast(&v, &mut self.diags).as_bytes().to_vec();
                Ok(Zval::Bool(true))
            }
        }
    }
    /// `mb_regex_set_options([$options])` (F2): getter returns the current options
    /// ("pr" default); setter stores them and returns the previous options. Mirrors
    /// `eval::ho_mb_regex_set_options`.
    pub(super) fn ho_mb_regex_set_options(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.mb_regex.options.clone();
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(prev))),
            Some(v) => {
                let v = v.deref_clone();
                if !matches!(v, Zval::Null) {
                    self.mb_regex.options =
                        convert::to_zstr_cast(&v, &mut self.diags).as_bytes().to_vec();
                }
                Ok(Zval::Str(PhpStr::new(prev)))
            }
        }
    }
    /// `mb_ereg`/`mb_eregi` (F2): match `$pattern` against `$string`, writing the
    /// `$regs` array into the out-param (index 2) and returning whether it matched.
    /// Mirrors `eval::ho_mb_ereg`. The out value is written by the VM out-param path.
    pub(super) fn ho_mb_ereg(&mut self, ic: bool, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        let func = if ic { "mb_eregi" } else { "mb_ereg" };
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let opts = self.mb_regex.options.clone();
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            // Bad pattern: false, and no out-param write (empty array is harmless).
            return Ok((Zval::Bool(false), Zval::Array(Rc::new(PhpArray::new()))));
        };
        let regs = crate::mbregex::exec(&re, &subject);
        let matched = regs.is_some();
        let out = regs.unwrap_or_else(|| Zval::Array(Rc::new(PhpArray::new())));
        Ok((Zval::Bool(matched), out))
    }
    /// `mb_ereg_replace`/`mb_eregi_replace` (F2): replace matches of `$pattern` in
    /// `$string` with `$replacement` (backrefs `\0`..`\9` honoured). Returns `false`
    /// on a bad pattern. Mirrors `eval::ho_mb_ereg_replace`.
    pub(super) fn ho_mb_ereg_replace(&mut self, ic: bool, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let func = if ic { "mb_eregi_replace" } else { "mb_ereg_replace" };
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let repl =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let opts = self.mb_opts_val(&args, 3);
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Str(PhpStr::new(crate::mbregex::replace(&re, &repl, &subject))))
    }
    /// `mb_ereg_replace_callback($pattern, $callback, $string[, $options])` (F2):
    /// the callback receives each match's `$regs` array and returns its replacement.
    /// Mirrors `eval::ho_mb_ereg_replace_callback` (callback via `call_callable`).
    pub(super) fn ho_mb_ereg_replace_callback(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_replace_callback() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let callback = args[1].deref_clone();
        let subject =
            convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let opts = self.mb_opts_val(&args, 3);
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_replace_callback", false) else {
            return Ok(Zval::Bool(false));
        };
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        for (start, end, regs) in crate::mbregex::find_all(&re, &subject) {
            out.extend_from_slice(&subject[last..start]);
            let ret = self.call_callable(callback.clone(), vec![regs])?;
            let rs = convert::to_zstr_cast(&ret.deref_clone(), &mut self.diags);
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&subject[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }
    /// `mb_ereg_match($pattern, $string[, $options])` (F2): whether the pattern
    /// matches anchored at the start of `$string` (a prefix match). Mirrors
    /// `eval::ho_mb_ereg_match`.
    pub(super) fn ho_mb_ereg_match(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_match() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let opts = self.mb_opts_val(&args, 2);
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_match", false) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Bool(crate::mbregex::matches_at_start(&re, &subject)))
    }
    /// `mb_ereg_search_init($string[, $pattern[, $options]])` (F2): start a stateful
    /// search over `$string`, resetting the cursor. Mirrors
    /// `eval::ho_mb_ereg_search_init`.
    pub(super) fn ho_mb_ereg_search_init(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "mb_ereg_search_init() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        self.mb_regex.search_str =
            convert::to_zstr_cast(&first.deref_clone(), &mut self.diags).as_bytes().to_vec();
        self.mb_regex.search_pos = 0;
        self.mb_regex.last_regs = None;
        if !self.mb_search_set_pattern(&args, 1) {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }
    /// `mb_ereg_search([$pattern[, $options]])` (F2): advance the cursor to the next
    /// match; returns whether one was found. Mirrors `eval::ho_mb_ereg_search`.
    pub(super) fn ho_mb_ereg_search(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(&args, 0) {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(self.mb_search_step().is_some()))
    }
    /// `mb_ereg_search_pos([$pattern[, $options]])` (F2): next match as `[pos, len]`
    /// byte offsets, or false at the end. Mirrors `eval::ho_mb_ereg_search_pos`.
    pub(super) fn ho_mb_ereg_search_pos(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(&args, 0) {
            return Ok(Zval::Bool(false));
        }
        match self.mb_search_step() {
            Some((start, end, _)) => {
                let mut arr = PhpArray::new();
                let _ = arr.append(Zval::Long(start as i64));
                let _ = arr.append(Zval::Long((end - start) as i64));
                Ok(Zval::Array(Rc::new(arr)))
            }
            None => Ok(Zval::Bool(false)),
        }
    }
    /// `mb_ereg_search_regs([$pattern[, $options]])` (F2): next match's `$regs`
    /// array, or false at the end. Mirrors `eval::ho_mb_ereg_search_regs`.
    pub(super) fn ho_mb_ereg_search_regs(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(&args, 0) {
            return Ok(Zval::Bool(false));
        }
        match self.mb_search_step() {
            Some((_, _, regs)) => Ok(regs),
            None => Ok(Zval::Bool(false)),
        }
    }
    /// `mb_ereg_search_getregs()` (F2): the `$regs` of the last successful search,
    /// or false if none. Mirrors `eval::ho_mb_ereg_search_getregs`.
    pub(super) fn ho_mb_ereg_search_getregs(&mut self) -> Result<Zval, PhpError> {
        Ok(self.mb_regex.last_regs.clone().unwrap_or(Zval::Bool(false)))
    }
    /// `mb_ereg_search_setpos($position)` (F2): move the byte cursor; false if out
    /// of range. Mirrors `eval::ho_mb_ereg_search_setpos`.
    pub(super) fn ho_mb_ereg_search_setpos(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let pos = match args.first() {
            Some(v) => convert::to_long_cast(&v.deref_clone(), &mut self.diags),
            None => 0,
        };
        if pos < 0 || pos as usize > self.mb_regex.search_str.len() {
            return Ok(Zval::Bool(false));
        }
        self.mb_regex.search_pos = pos as usize;
        Ok(Zval::Bool(true))
    }
    /// `define($name, $value)` (B3): register a user constant. The name is coerced
    /// to a string; redefining an existing user *or* engine constant warns and
    /// returns `false` (PHP 8.5 message), otherwise stores it and returns `true`.
    /// (The legacy case-insensitive third argument was removed in PHP 8.)
    pub(super) fn ho_define(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Err(PhpError::Error(
                "define() expects at least 2 arguments, 0 given".to_string(),
            ));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        let value = args.get(1).cloned().unwrap_or(Zval::Null);
        if self.constant_known(&cname) {
            self.diags.push(Diag::Warning(format!(
                "Constant {} already defined, this will be an error in PHP 9",
                String::from_utf8_lossy(&cname)
            )));
            return Ok(Zval::Bool(false));
        }
        self.constants.insert(cname, value);
        Ok(Zval::Bool(true))
    }
    /// `defined($name)` (B3): whether `name` is a known user or engine constant.
    pub(super) fn ho_defined(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Ok(Zval::Bool(false));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        self.class_const_class_autoload(&cname);
        Ok(Zval::Bool(self.constant_known(&cname)))
    }
    /// `constant($name)` (B3): the value of user constant `name`, else the engine
    /// constant, else the catchable "Undefined constant" `Error`.
    pub(super) fn ho_constant(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(name_arg) = args.first() else {
            return Err(PhpError::Error(
                "constant() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let cname = convert::to_zstr_cast(name_arg, &mut self.diags).as_bytes().to_vec();
        if let Some(v) = self.constants.get(&cname) {
            return Ok(v.clone());
        }
        if let Some(z) = crate::lower::resolve_constant(&cname).and_then(const_literal_to_zval) {
            return Ok(z);
        }
        // `constant("Class::CONST")`: resolve the class constant through the
        // parent/interface chain and evaluate its value thunk in the declaring
        // class's context (the thunk ref is `'m`, so it survives `&mut self`).
        self.class_const_class_autoload(&cname);
        if let Some((decl, idx)) = self.class_const_ref(&cname) {
            let thunk: &'m Func = &self.classes[decl].consts[idx].func;
            return self.run_value_thunk(thunk, Some(decl));
        }
        // `constant("Enum::Case")` materialises the case singleton.
        if let Some((cid, ci)) = self.enum_case_ref(&cname) {
            return Ok(Zval::Object(self.enum_case(cid, ci as u32)));
        }
        Err(PhpError::Error(format!(
            "Undefined constant \"{}\"",
            String::from_utf8_lossy(&cname)
        )))
    }
    /// `call_user_func($callable, ...$args)`: forward the remaining arguments by
    /// value to the callable (mirrors `eval::ho_call_user_func`).
    pub(super) fn ho_call_user_func(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = args.into_iter();
        let Some(callee) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "call_user_func() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let argv: Vec<Zval> = it.map(|v| v.deref_clone()).collect();
        let callee = callee.deref_clone();
        // `call_user_func` always passes by value, so every by-ref parameter warns.
        self.warn_trampoline_byref(&callee, argv.len(), &[])?;
        self.call_callable(callee, argv)
    }
    /// `call_user_func_array($callable, $args)`: the second argument is an array
    /// whose *values* become the positional arguments (string-keyed named
    /// arguments are a scope-out, mirroring the evaluator).
    pub(super) fn ho_call_user_func_array(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "call_user_func_array() expects exactly 2 arguments, {} given",
                args.len()
            )));
        }
        let callee = args[0].deref_clone();
        // Track which elements were references: `_array` can pass those by reference,
        // so only the *non*-reference elements at by-ref positions warn.
        let (argv, arg_is_ref): (Vec<Zval>, Vec<bool>) = match args[1].deref_clone() {
            Zval::Array(a) => a
                .iter()
                .map(|(_, v)| (v.deref_clone(), matches!(v, Zval::Ref(_))))
                .unzip(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "call_user_func_array(): Argument #2 ($args) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        self.warn_trampoline_byref(&callee, argv.len(), &arg_is_ref)?;
        self.call_callable(callee, argv)
    }
    /// `array_map($callback, ...$arrays)` (Session C): a single array preserves
    /// keys; several arrays re-index 0..max and pass one element from each per row
    /// (missing tails NULL). A NULL callback zips the arrays (single array →
    /// identity). Mirrors `eval::ho_array_map`, calling via `call_callable`.
    pub(super) fn ho_array_map(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_map() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let cb = args[0].deref_clone();
        let null_cb = matches!(cb, Zval::Null);
        let mut arrays = Vec::with_capacity(args.len() - 1);
        for (i, a) in args[1..].iter().enumerate() {
            match a.deref_clone() {
                Zval::Array(arr) => arrays.push(arr),
                other => {
                    return Err(PhpError::TypeError(format!(
                        "array_map(): Argument #{} must be of type array, {} given",
                        i + 2,
                        other.type_name_for_error()
                    )))
                }
            }
        }

        let mut out = PhpArray::new();
        if arrays.len() == 1 {
            let entries: Vec<(Key, Zval)> =
                arrays[0].iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
            for (k, v) in entries {
                let mapped = if null_cb { v } else { self.call_callable(cb.clone(), vec![v])? };
                out.insert(k, mapped);
            }
        } else {
            let cols: Vec<Vec<Zval>> = arrays
                .iter()
                .map(|a| a.iter().map(|(_, v)| v.deref_clone()).collect())
                .collect();
            let max = cols.iter().map(|c| c.len()).max().unwrap_or(0);
            for i in 0..max {
                let row: Vec<Zval> =
                    cols.iter().map(|c| c.get(i).cloned().unwrap_or(Zval::Null)).collect();
                let val = if null_cb {
                    let mut tuple = PhpArray::new();
                    for v in row {
                        let _ = tuple.append(v);
                    }
                    Zval::Array(Rc::new(tuple))
                } else {
                    self.call_callable(cb.clone(), row)?
                };
                let _ = out.append(val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `array_filter($array, $callback?, $mode = 0)` (Session C): keys are always
    /// preserved. No callback keeps truthy values; otherwise the callback receives
    /// the value (mode 0), the key (`ARRAY_FILTER_USE_KEY` = 2), or `(value, key)`
    /// (`ARRAY_FILTER_USE_BOTH` = 1). Mirrors `eval::ho_array_filter`.
    pub(super) fn ho_array_filter(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "array_filter() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arr = match first.deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_filter(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        let cb = match args.get(1) {
            Some(a) => match a.deref_clone() {
                Zval::Null => None,
                v => Some(v),
            },
            None => None,
        };
        let mode = match args.get(2) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };

        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        let mut out = PhpArray::new();
        for (k, v) in entries {
            let keep = match &cb {
                None => convert::to_bool(&v, &mut self.diags),
                Some(c) => {
                    let call_args = match mode {
                        2 => vec![key_to_zval(&k)],
                        1 => vec![v.clone(), key_to_zval(&k)],
                        _ => vec![v.clone()],
                    };
                    let r = self.call_callable(c.clone(), call_args)?;
                    convert::to_bool(&r, &mut self.diags)
                }
            };
            if keep {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `array_all($array, $callback)` (PHP 8.4): whether the callback is truthy
    /// for *every* element (vacuously true when empty) — the first falsy stops.
    pub(super) fn ho_array_all(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        // "all satisfy P" == "none satisfies !P": walk until the callback is
        // falsy. The walker searches for truthy, so wrap per-element below.
        let arr = match args.first().map(|a| a.deref_clone()) {
            Some(Zval::Array(a)) => a,
            Some(other) => {
                return Err(PhpError::TypeError(format!(
                    "array_all(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )));
            }
            None => {
                return Err(PhpError::ArgumentCountError(
                    "array_all() expects exactly 2 arguments, 0 given".to_string(),
                ));
            }
        };
        let Some(cb) = args.get(1).map(|c| c.deref_clone()) else {
            return Err(PhpError::ArgumentCountError(
                "array_all() expects exactly 2 arguments, 1 given".to_string(),
            ));
        };
        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        for (k, v) in entries {
            let r = self.call_callable(cb.clone(), vec![v, key_to_zval(&k)])?;
            if !convert::to_bool(&r, &mut self.diags) {
                return Ok(Zval::Bool(false));
            }
        }
        Ok(Zval::Bool(true))
    }
    /// `array_any($array, $callback)` (PHP 8.4): whether the callback is truthy
    /// for *some* element (false when empty).
    pub(super) fn ho_array_any(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        Ok(Zval::Bool(self.array_search_callback("array_any", &args)?.is_some()))
    }
    /// `array_find($array, $callback)` (PHP 8.4): the first *value* the callback
    /// accepts, `null` if none.
    pub(super) fn ho_array_find(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        Ok(self.array_search_callback("array_find", &args)?.map(|(_, v)| v).unwrap_or(Zval::Null))
    }
    /// `array_find_key($array, $callback)` (PHP 8.4): the first *key* the
    /// callback accepts, `null` if none.
    pub(super) fn ho_array_find_key(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        Ok(self
            .array_search_callback("array_find_key", &args)?
            .map(|(k, _)| key_to_zval(&k))
            .unwrap_or(Zval::Null))
    }
    /// `array_reduce($array, $callback, $initial = null)` (Session C): fold the
    /// values left-to-right through `$callback($carry, $item)`, returning the final
    /// carry. (The evaluator has no `array_reduce`, so this is pure VM gain.)
    pub(super) fn ho_array_reduce(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_reduce() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let arr = match args[0].deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_reduce(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        let cb = args[1].deref_clone();
        let mut carry = args.get(2).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let values: Vec<Zval> = arr.iter().map(|(_, v)| v.deref_clone()).collect();
        for v in values {
            carry = self.call_callable(cb.clone(), vec![carry, v])?;
        }
        Ok(carry)
    }
    /// `get_class($object = null)` (Session B2): the object's class name. A
    /// `Closure` is `"Closure"`. With no argument PHP 8.5 uses the calling `$this`
    /// (now deprecated) and fatals outside object context. Mirrors `eval::ci_get_class`.
    pub(super) fn ho_get_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = match args.into_iter().next() {
            Some(a) => a.deref_clone(),
            None => {
                let top = self.frames.len() - 1;
                match self.frames[top].this.clone() {
                    Some(t) => {
                        self.diags.push(Diag::Deprecated(
                            "Calling get_class() without arguments is deprecated".to_string(),
                        ));
                        t
                    }
                    None => {
                        return Err(PhpError::Error(
                            "get_class() without arguments must be called from within a class"
                                .to_string(),
                        ))
                    }
                }
            }
        };
        match &v {
            Zval::Object(o) => {
                Ok(Zval::Str(PhpStr::new(o.borrow().class_name.as_bytes().to_vec())))
            }
            Zval::Closure(_) => Ok(Zval::Str(PhpStr::new(b"Closure".to_vec()))),
            // A generator is an object in PHP (PHPUnit's exporter reflects it).
            Zval::Generator(_) => Ok(Zval::Str(PhpStr::new(b"Generator".to_vec()))),
            other => Err(PhpError::TypeError(format!(
                "get_class(): Argument #1 ($object) must be of type object, {} given",
                other.type_name_for_error()
            ))),
        }
    }
    /// `get_debug_type($value)` (PHP 8.0): the modern type name — `null`/`bool`/
    /// `int`/`float`/`string`/`array`, the class name for an object, `Closure`/
    /// `Generator`, or `resource (TYPE)` / `resource (closed)`. Unlike `gettype`
    /// it returns the canonical scalar names and the concrete class.
    pub(super) fn ho_get_debug_type(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args
            .into_iter()
            .next()
            .ok_or_else(|| {
                PhpError::ArgumentCountError(
                    "get_debug_type() expects exactly 1 argument, 0 given".to_string(),
                )
            })?
            .deref_clone();
        let name: String = match &v {
            Zval::Undef | Zval::Null => "null".to_string(),
            Zval::Bool(_) => "bool".to_string(),
            Zval::Long(_) => "int".to_string(),
            Zval::Double(_) => "float".to_string(),
            Zval::Str(_) => "string".to_string(),
            Zval::Array(_) => "array".to_string(),
            Zval::Closure(_) => "Closure".to_string(),
            Zval::Generator(_) => "Generator".to_string(),
            Zval::Object(o) => String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned(),
            // An internal weak handle never surfaces to user code as a value.
            Zval::WeakHandle(_) => "object".to_string(),
            Zval::Resource(r) => {
                let rb = r.borrow();
                if rb.is_open() {
                    format!("resource ({})", rb.dump_type())
                } else {
                    "resource (closed)".to_string()
                }
            }
            Zval::Ref(_) => unreachable!("deref_clone strips Ref"),
        };
        Ok(Zval::Str(PhpStr::new(name.into_bytes())))
    }
    /// `spl_object_id($object)` — the object's integer handle (its `#N`). Unique
    /// among live objects; reusable after an object is freed (as in PHP).
    pub(super) fn ho_spl_object_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.into_iter().next().unwrap_or(Zval::Null).deref_clone();
        Ok(Zval::Long(Self::object_handle_id(&v, "spl_object_id")? as i64))
    }
    /// `spl_object_hash($object)` — a 32-hex-digit string unique to the object for
    /// its lifetime. PHP derives it from the handle; we render the handle as the
    /// low bytes of the 32-char hash, which preserves per-object uniqueness.
    pub(super) fn ho_spl_object_hash(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.into_iter().next().unwrap_or(Zval::Null).deref_clone();
        let id = Self::object_handle_id(&v, "spl_object_hash")?;
        Ok(Zval::Str(PhpStr::new(format!("{:032x}", id).into_bytes())))
    }
    /// `__weak_create($object)` (internal): build a weak handle to an object so
    /// it can be held without keeping it alive — the backing of `WeakReference` /
    /// `WeakMap`. A non-object passes through unchanged (a `Closure`/`Generator`
    /// referent degrades to a strong hold, a rare edge), so `__weak_get` round-
    /// trips it. Not a public PHP function.
    pub(super) fn ho_weak_create(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.into_iter().next().unwrap_or(Zval::Null).deref_clone();
        Ok(match v {
            Zval::Object(rc) => Zval::WeakHandle(Rc::downgrade(&rc)),
            other => other,
        })
    }
    /// `__weak_get($handle)` (internal): upgrade a weak handle to its object, or
    /// `null` once the last strong reference is gone (true weakness). A non-handle
    /// value (the strong-fallback case) passes through.
    pub(super) fn ho_weak_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.into_iter().next().unwrap_or(Zval::Null).deref_clone();
        Ok(match v {
            Zval::WeakHandle(w) => w.upgrade().map(Zval::Object).unwrap_or(Zval::Null),
            other => other,
        })
    }
    /// `__zip_open($path)` (internal, the prelude `ZipArchive::open` backing):
    /// parse the archive and allocate a handle. Returns `[id, numFiles]` on
    /// success or the PHP ZipArchive error constant as an int (ER_NOENT = 9 /
    /// ER_OPEN = 11 / ER_NOZIP = 19 / ER_READ = 5).
    pub(super) fn ho_zip_open(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let path = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        let path = String::from_utf8_lossy(path.as_bytes()).into_owned();
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                let code = if e.kind() == std::io::ErrorKind::NotFound { 9 } else { 11 };
                return Ok(Zval::Long(code));
            }
        };
        match ::zip::ZipArchive::new(file) {
            Ok(z) => {
                let id = self.next_zip;
                self.next_zip += 1;
                let n = z.len() as i64;
                self.zips.insert(id, z);
                let mut out = PhpArray::new();
                let _ = out.append(Zval::Long(i64::from(id)));
                let _ = out.append(Zval::Long(n));
                Ok(Zval::Array(Rc::new(out)))
            }
            Err(::zip::result::ZipError::InvalidArchive(_)) => Ok(Zval::Long(19)),
            Err(_) => Ok(Zval::Long(5)),
        }
    }
    /// `__zip_close($id)`: release the handle. `false` on an unknown/closed one.
    pub(super) fn ho_zip_close(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        Ok(Zval::Bool(self.zips.remove(&id).is_some()))
    }
    /// `__zip_stat_index($id, $index)`: the entry's metadata as PHP's statIndex
    /// array (`name`/`index`/`crc`/`size`/`mtime`/`comp_size`/`comp_method`/
    /// `encryption_method`), or `false` out of range. `mtime` is reported as 0
    /// (declared residue: the dosdate conversion is not modelled).
    pub(super) fn ho_zip_stat_index(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let idx = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(z) = self.zips.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        if idx < 0 {
            return Ok(Zval::Bool(false));
        }
        let Ok(f) = z.by_index_raw(idx as usize) else { return Ok(Zval::Bool(false)) };
        let mut out = PhpArray::new();
        out.insert(Key::from_bytes(b"name"), Zval::Str(PhpStr::new(f.name_raw().to_vec())));
        out.insert(Key::from_bytes(b"index"), Zval::Long(idx));
        out.insert(Key::from_bytes(b"crc"), Zval::Long(i64::from(f.crc32())));
        out.insert(Key::from_bytes(b"size"), Zval::Long(f.size() as i64));
        out.insert(Key::from_bytes(b"mtime"), Zval::Long(0));
        out.insert(Key::from_bytes(b"comp_size"), Zval::Long(f.compressed_size() as i64));
        out.insert(
            Key::from_bytes(b"comp_method"),
            Zval::Long(match f.compression() {
                ::zip::CompressionMethod::Stored => 0,
                ::zip::CompressionMethod::Deflated => 8,
                _ => -1,
            }),
        );
        out.insert(Key::from_bytes(b"encryption_method"), Zval::Long(if f.encrypted() { 257 } else { 0 }));
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__zip_get_name_index($id, $index)`: the entry's name, `false` out of range.
    pub(super) fn ho_zip_get_name_index(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let idx = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(z) = self.zips.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        if idx < 0 {
            return Ok(Zval::Bool(false));
        }
        match z.by_index_raw(idx as usize) {
            Ok(f) => Ok(Zval::Str(PhpStr::new(f.name_raw().to_vec()))),
            Err(_) => Ok(Zval::Bool(false)),
        }
    }
    /// `__zip_locate_name($id, $name)`: the index of the exact entry name,
    /// `false` when absent (PHP's default flag-less locateName).
    pub(super) fn ho_zip_locate_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(z) = self.zips.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        for i in 0..z.len() {
            if let Ok(f) = z.by_index_raw(i) {
                if f.name_raw() == name.as_bytes() {
                    return Ok(Zval::Long(i as i64));
                }
            }
        }
        Ok(Zval::Bool(false))
    }
    /// `__zip_get_from_index($id, $index)`: the entry's decompressed contents,
    /// `false` out of range / on a read error.
    pub(super) fn ho_zip_get_from_index(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let idx = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let Some(z) = self.zips.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        if idx < 0 {
            return Ok(Zval::Bool(false));
        }
        let Ok(mut f) = z.by_index(idx as usize) else { return Ok(Zval::Bool(false)) };
        let mut buf = Vec::new();
        match std::io::Read::read_to_end(&mut f, &mut buf) {
            Ok(_) => Ok(Zval::Str(PhpStr::new(buf))),
            Err(_) => Ok(Zval::Bool(false)),
        }
    }
    /// `__zip_extract_to($id, $dest)`: extract every entry under `$dest`
    /// (directories created as needed, unix permissions preserved when
    /// recorded, entries escaping the destination skipped — the zip-slip guard
    /// libzip applies). `false` on any I/O failure.
    pub(super) fn ho_zip_extract_to(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let dest = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let dest = std::path::PathBuf::from(String::from_utf8_lossy(dest.as_bytes()).into_owned());
        let Some(z) = self.zips.get_mut(&id) else { return Ok(Zval::Bool(false)) };
        if std::fs::create_dir_all(&dest).is_err() {
            return Ok(Zval::Bool(false));
        }
        for i in 0..z.len() {
            let Ok(mut f) = z.by_index(i) else { return Ok(Zval::Bool(false)) };
            // Zip-slip guard: entries with `..`/absolute paths are skipped.
            let Some(rel) = f.enclosed_name() else { continue };
            let path = dest.join(rel);
            if f.is_dir() {
                if std::fs::create_dir_all(&path).is_err() {
                    return Ok(Zval::Bool(false));
                }
                continue;
            }
            if let Some(parent) = path.parent() {
                if std::fs::create_dir_all(parent).is_err() {
                    return Ok(Zval::Bool(false));
                }
            }
            let Ok(mut out) = std::fs::File::create(&path) else { return Ok(Zval::Bool(false)) };
            if std::io::copy(&mut f, &mut out).is_err() {
                return Ok(Zval::Bool(false));
            }
            #[cfg(unix)]
            if let Some(mode) = f.unix_mode() {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode & 0o777));
            }
        }
        Ok(Zval::Bool(true))
    }
    /// `get_parent_class($object_or_class = null)` (Session B2): the parent class
    /// name, or `false` when there is none. An object or a *resolvable* class-name
    /// string selects the class; an unresolvable string (or other type) is a
    /// `TypeError`, matching PHP 8.5 (eval returns `false` here, so VM ≥ eval). No
    /// argument uses the current class context.
    pub(super) fn ho_get_parent_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        // Generator / Closure are valid engine classes with no parent (→ false),
        // not the "invalid class name" TypeError class_arg_to_id would raise.
        if args.first().and_then(Self::engine_special_class_name).is_some() {
            return Ok(Zval::Bool(false));
        }
        let top = self.frames.len() - 1;
        let cid: Option<ClassId> = match args.into_iter().next() {
            Some(a) => Some(self.class_arg_to_id(a.deref_clone(), "get_parent_class")?),
            None => self.frames[top].class,
        };
        match cid.and_then(|c| self.classes[c].parent) {
            Some(p) => Ok(Zval::Str(PhpStr::new(self.classes[p].name.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }
    /// `class_parents($object_or_class)` — the ancestor classes from the immediate
    /// parent upward, as a `name => name` array (insertion order = nearest first);
    /// `false` for a bad argument. Interfaces/traits are excluded (see
    /// `class_implements`).
    pub(super) fn ho_class_parents(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        // Generator / Closure have no parent class (class_parents → []).
        if args.first().and_then(Self::engine_special_class_name).is_some() {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        }
        let Some(cid) = args.into_iter().next().and_then(|v| self.class_arg_or_warn(v, "class_parents")) else {
            return Ok(Zval::Bool(false));
        };
        let mut arr = php_types::PhpArray::new();
        let mut cur = self.classes[cid].parent;
        while let Some(p) = cur {
            let name = self.classes[p].name.to_vec();
            arr.insert(Key::Str(PhpStr::new(name.clone())), Zval::Str(PhpStr::new(name)));
            cur = self.classes[p].parent;
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `class_implements($object_or_class)` — every interface the class implements,
    /// transitively (its own and its ancestors', plus interfaces those interfaces
    /// extend), as a `name => name` array; `false` for a bad argument.
    pub(super) fn ho_class_implements(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        // Generator implements Iterator (⊇ Traversable); Closure implements
        // nothing. Ordered as PHP returns them (Iterator, then Traversable).
        if let Some(kind) = args.first().and_then(Self::engine_special_class_name) {
            let mut arr = php_types::PhpArray::new();
            if kind == "Generator" {
                for name in [&b"Iterator"[..], &b"Traversable"[..]] {
                    arr.insert(
                        Key::Str(PhpStr::new(name.to_vec())),
                        Zval::Str(PhpStr::new(name.to_vec())),
                    );
                }
            }
            return Ok(Zval::Array(Rc::new(arr)));
        }
        let Some(cid) = args.into_iter().next().and_then(|v| self.class_arg_or_warn(v, "class_implements")) else {
            return Ok(Zval::Bool(false));
        };
        let mut arr = php_types::PhpArray::new();
        let mut seen: HashSet<ClassId> = HashSet::new();
        let mut klass = Some(cid);
        while let Some(c) = klass {
            let ifaces = self.classes[c].interfaces.clone();
            for i in ifaces {
                self.collect_iface(i, &mut arr, &mut seen);
            }
            klass = self.classes[c].parent;
        }
        // A class with a resolvable `__toString` auto-implements `Stringable`
        // (step 24-1), even without an explicit `implements` — mirroring
        // `is_instance_of`. Appended only if not already collected explicitly.
        if let Some(sid) = self.stringable_id {
            if !seen.contains(&sid)
                && resolve_method_runtime(&self.classes, cid, b"__toString").is_some()
            {
                let name = self.classes[sid].name.to_vec();
                arr.insert(Key::Str(PhpStr::new(name.clone())), Zval::Str(PhpStr::new(name)));
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `is_a($object_or_class, $class, $allow_string = false)`: whether the subject
    /// is an instance of `$class` (the class itself, an ancestor, or an
    /// implemented interface). A class-name string subject is only accepted when
    /// `$allow_string` is true.
    pub(super) fn ho_is_a(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.is_a_impl(args, false, true)
    }
    /// `is_subclass_of($object_or_class, $class, $allow_string = true)`: like
    /// `is_a` but the subject's own class does not count — only a *proper*
    /// subclass or an implemented interface returns true.
    pub(super) fn ho_is_subclass_of(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.is_a_impl(args, true, false)
    }
    /// `class_uses($class, $autoload = true)`: the traits used *directly* by the
    /// class (not inherited from parents), as a `name => name` array in source
    /// order. `false` if the class does not exist. Mirrors PHP, which reports only
    /// the directly-`use`d traits.
    pub(super) fn ho_class_uses(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cid) = args.into_iter().next().and_then(|v| self.class_arg_or_warn(v, "class_uses"))
        else {
            return Ok(Zval::Bool(false));
        };
        let mut arr = php_types::PhpArray::new();
        for t in &self.classes[cid].uses_traits {
            let name = t.to_vec();
            arr.insert(Key::Str(PhpStr::new(name.clone())), Zval::Str(PhpStr::new(name)));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `trait_exists($name, $autoload = true)`: whether `$name` names a declared
    /// trait. PHP does NOT namespace-resolve the argument: it is matched as a
    /// fully-qualified name (case-insensitively, a leading `\` stripped) against
    /// each trait's real name — so `trait_exists('IFoo')` inside namespace `foo`
    /// is `false` even when `foo\IFoo` exists. Autoload is attempted when allowed.
    pub(super) fn ho_trait_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else { return Ok(Zval::Bool(false)) };
        let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let want = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let autoload = !matches!(args.get(1).map(|v| v.deref_clone()), Some(Zval::Bool(false)));
        let present = |s: &Self| {
            s.seed_traits.iter().any(|(_, t)| t.name.to_ascii_lowercase() == want)
        };
        if present(self) {
            return Ok(Zval::Bool(true));
        }
        if autoload {
            self.try_autoload(&want, &want)?;
        }
        Ok(Zval::Bool(present(self)))
    }
    /// `get_declared_traits()`: the names (original case) of every declared trait,
    /// in declaration order.
    pub(super) fn ho_get_declared_traits(&mut self) -> Result<Zval, PhpError> {
        let mut arr = php_types::PhpArray::new();
        for (_, t) in &self.seed_traits {
            let _ = arr.append(Zval::Str(PhpStr::new(t.name.to_vec())));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__lazy_is_uninitialized($obj)`: whether `$obj` is an uninitialized lazy
    /// object — PHP 8.4 `ReflectionClass::isUninitializedLazyObject`. A ghost
    /// clears its marker on init; an initialized proxy keeps `Some(Proxy)` but
    /// carries a `proxy_instance`, so the "uninitialized" test excludes it.
    pub(super) fn ho_lazy_is_uninitialized(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let is = args
            .first()
            .and_then(deref_object)
            .map_or(false, |o| { let b = o.borrow(); b.lazy.is_some() && b.proxy_instance.is_none() });
        Ok(Zval::Bool(is))
    }
    /// `__lazy_is_initializing($obj)`: whether `$obj`'s initializer/factory is
    /// currently running — a re-entrant `resetAsLazy*` on it is forbidden (PHP
    /// 8.4).
    pub(super) fn ho_lazy_is_initializing(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let is = args
            .first()
            .and_then(deref_object)
            .is_some_and(|o| self.lazy_initializing.contains(&o.borrow().id));
        Ok(Zval::Bool(is))
    }
    /// `__lazy_initialize($obj)`: force initialization and return the object the
    /// caller should observe — the *real instance* for a proxy (transitively),
    /// the object itself for a ghost — PHP 8.4
    /// `ReflectionClass::initializeLazyObject`.
    pub(super) fn ho_lazy_initialize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.first().cloned().unwrap_or(Zval::Null);
        self.realize_full(&v)
    }
    /// `__lazy_get_initializer($obj)`: the pending initializer/factory of an
    /// *uninitialized* lazy object, or NULL — PHP 8.4
    /// `ReflectionClass::getLazyInitializer`.
    pub(super) fn ho_lazy_get_initializer(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let init = args
            .first()
            .and_then(deref_object)
            .filter(|o| {
                let b = o.borrow();
                b.lazy.is_some() && b.proxy_instance.is_none()
            })
            .and_then(|o| self.lazy_init.get(&o.borrow().id).cloned());
        Ok(init.unwrap_or(Zval::Null))
    }
    /// `__lazy_prop_is_lazy($obj, $class, $prop)`: whether the property is still
    /// lazy on `$obj` (an access would initialize) — PHP 8.4
    /// `ReflectionProperty::isLazy`.
    pub(super) fn ho_lazy_prop_is_lazy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut obj = args.first().cloned().unwrap_or(Zval::Null);
        let prop = convert::to_zstr_cast(args.get(2).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        // An initialized proxy reports its (possibly re-reset) instance's state.
        for _ in 0..64 {
            let next = self.proxy_redirect(obj.clone());
            let same = match (deref_object(&obj), deref_object(&next)) {
                (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                _ => true,
            };
            obj = next;
            if same {
                break;
            }
        }
        let is = deref_object(&obj).is_some_and(|o| {
            let (oid, cid, uninit) = {
                let b = o.borrow();
                (b.id, b.class_id as usize, b.lazy.is_some() && b.proxy_instance.is_none())
            };
            if !uninit {
                return false;
            }
            let key = self.prop_decl_storage_key(cid, &prop);
            self.lazy_props
                .get(&oid)
                .is_some_and(|set| set.iter().any(|n| n.as_ref() == key.as_slice()))
        });
        Ok(Zval::Bool(is))
    }
    /// `__lazy_mark_initialized($obj)`: PHP 8.4
    /// `ReflectionClass::markLazyObjectAsInitialized` — flip the object to its
    /// initialized shape (property defaults applied, lazy marker cleared)
    /// WITHOUT running the pending initializer/factory, which is discarded.
    /// A no-op on a non-lazy or already-initialized object.
    pub(super) fn ho_lazy_mark_initialized(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.first().cloned().unwrap_or(Zval::Null);
        if let Some(rc) = deref_object(&v) {
            let (oid, cid, uninit) = {
                let b = rc.borrow();
                (b.id, b.class_id as usize, b.lazy.is_some() && b.proxy_instance.is_none())
            };
            if uninit {
                let cc = self.classes[cid];
                let mut props = Props::new();
                for (name, c) in &cc.prop_defaults {
                    props.set(name, c.to_zval());
                }
                for name in &cc.uninit_props {
                    props.set(name, Zval::Undef);
                }
                {
                    let mut b = rc.borrow_mut();
                    b.lazy = None;
                    b.props = props;
                }
                self.lazy_props.remove(&oid);
                self.lazy_init.remove(&oid);
            }
        }
        Ok(v)
    }
    /// `__lazy_skip_init($obj, $class, $prop)`: mark a single declared property of
    /// an uninitialized lazy object as non-lazy, materializing its declared
    /// default without running the initializer (PHP 8.4
    /// `ReflectionProperty::skipLazyInitialization`). Returns `null` on success
    /// or a `string` ReflectionException message on an ineligible property.
    pub(super) fn ho_lazy_skip_init(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = args.first().cloned().unwrap_or(Zval::Null);
        let class = args.get(1).cloned().unwrap_or(Zval::Null);
        let prop = convert::to_zstr_cast(args.get(2).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let cid = match self.lazy_prop_target(&class, &prop, "skipLazyInitialization") {
            Ok(c) => c,
            Err(msg) => return Ok(Zval::Str(PhpStr::new(msg.into_bytes()))),
        };
        // Reflection speaks names; the slot (defaults, uninit set, lazy set) is
        // storage-keyed — mangled for a private.
        let key = self.prop_decl_storage_key(cid, &prop);
        // A skip is a NO-OP on an initialized object and on a property that is
        // no longer lazy (skipLazyInitialization_initialized_object /
        // _skips_non_lazy_prop) — unlike a raw-set, it writes nothing then.
        {
            let mut target = obj.clone();
            for _ in 0..64 {
                let next = self.proxy_redirect(target.clone());
                let same = match (deref_object(&target), deref_object(&next)) {
                    (Some(a), Some(b)) => Rc::ptr_eq(&a, &b),
                    _ => true,
                };
                target = next;
                if same {
                    break;
                }
            }
            let uninit_still_lazy = deref_object(&target).is_some_and(|o| {
                let b = o.borrow();
                b.lazy.is_some()
                    && b.proxy_instance.is_none()
                    && self
                        .lazy_props
                        .get(&b.id)
                        .is_some_and(|set| set.iter().any(|n| n.as_ref() == key.as_slice()))
            });
            if !uninit_still_lazy {
                return Ok(Zval::Null);
            }
        }
        // The property's declared default: its const, but a typed property with no
        // default stays uninitialized (`Undef`).
        let cc = self.classes[cid];
        let default = if cc.uninit_props.iter().any(|n| n.as_ref() == key.as_slice()) {
            Zval::Undef
        } else {
            cc.prop_defaults
                .iter()
                .find(|(n, _)| n.as_ref() == key.as_slice())
                .map(|(_, c)| c.to_zval())
                .unwrap_or(Zval::Null)
        };
        self.lazy_materialize(&obj, &key, default)?;
        Ok(Zval::Null)
    }
    /// `__lazy_set_raw($obj, $class, $prop, $value)`: set a single declared
    /// property of an uninitialized lazy object to `$value` without running the
    /// initializer, marking it non-lazy (PHP 8.4
    /// `ReflectionProperty::setRawValueWithoutLazyInitialization`). Returns `null`
    /// on success or a `string` ReflectionException message on an ineligible
    /// property.
    pub(super) fn ho_lazy_set_raw(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = args.first().cloned().unwrap_or(Zval::Null);
        let class = args.get(1).cloned().unwrap_or(Zval::Null);
        let prop = convert::to_zstr_cast(args.get(2).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let value = args.get(3).cloned().unwrap_or(Zval::Null);
        let cid = match self.lazy_prop_target(&class, &prop, "setRawValueWithoutLazyInitialization") {
            Ok(c) => c,
            Err(msg) => return Ok(Zval::Str(PhpStr::new(msg.into_bytes()))),
        };
        // Reflection speaks names; the slot is storage-keyed (mangled private).
        let key = self.prop_decl_storage_key(cid, &prop);
        self.lazy_materialize(&obj, &key, value)?;
        Ok(Zval::Null)
    }
    pub(super) fn ho_get_object_vars(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "get_object_vars() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let v = a.deref_clone();
        // A lazy object initializes before its properties are read (PHP 8.4); a
        // proxy then forwards to its real instance.
        let v = self.realize_full(&v)?;
        let Zval::Object(o) = v else {
            return Err(PhpError::TypeError(format!(
                "get_object_vars(): Argument #1 ($object) must be of type object, {} given",
                v.type_name_for_error()
            )));
        };
        let cur = self.frames[self.frames.len() - 1].class;
        let (cid, oid) = {
            let b = o.borrow();
            (b.class_id as usize, b.id)
        };
        let arr = self.object_vars_array(&o, cid, oid, cur)?;
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `get_class_vars(string $class)`: the class's default property values as an
    /// associative `name => default` array, filtered by visibility from the calling
    /// scope. Instance properties come first (most-derived class first, declaration
    /// order, a redeclaration keeping the derived position), then static properties.
    /// Values are the *declared* defaults — a since-modified static cell is ignored,
    /// matching PHP. An unknown class is the PHP 8 TypeError.
    pub(super) fn ho_get_class_vars(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "get_class_vars() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let name = raw.as_bytes();
        let key = name.strip_prefix(b"\\").unwrap_or(name).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(key.as_slice()) else {
            return Err(PhpError::TypeError(format!(
                "get_class_vars(): Argument #1 ($class) must be a valid class name, {} given",
                String::from_utf8_lossy(name)
            )));
        };
        let cur = self.frames[self.frames.len() - 1].class;
        // Materialise the *evaluated* instance defaults via a throwaway instance:
        // constants come from `alloc_object`, non-constant defaults (arrays, `1+2`,
        // …) from the prop-init thunk. The instance is untracked for `__destruct`.
        // Ids after this mark belong to the throwaway allocation below.
        let created_mark = self.created.last_key_value().map(|(id, _)| *id);
        let temp = self.alloc_object(cid)?;
        let cc = self.classes[cid];
        if let Some(func) = cc.prop_init.as_ref() {
            let baseline = self.frames.len();
            let mut frame = Frame::new(func, self.class_mod(cid));
            frame.this = Some(temp.clone());
            frame.class = Some(cid);
            frame.static_class = Some(cid);
            frame.init_props = true;
            // No `ret_cell`: the thunk's `Ret` must hit the `frames.len() == baseline`
            // check so `drive_to_return` stops here instead of resuming the caller.
            self.frames.push(frame);
            self.drive_to_return(baseline)?; // returns the thunk's NULL — ignored
        }
        let defaults: Vec<(Box<[u8]>, Zval)> = match &temp {
            Zval::Object(o) => o
                .borrow()
                .props
                .iter()
                .map(|(n, v)| (n.to_vec().into_boxed_slice(), v.deref_clone()))
                .collect(),
            _ => Vec::new(),
        };
        // Discard the throwaway (and anything its prop-init minted), no __destruct.
        match created_mark {
            Some(m) => drop(self.created.split_off(&(m + 1))),
            None => self.created.clear(),
        }
        // Inheritance chain, most-derived first.
        let mut chain: Vec<usize> = Vec::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            chain.push(ci);
            c = self.classes[ci].parent;
        }
        let mut arr = PhpArray::new();
        let mut seen: Vec<Box<[u8]>> = Vec::new();
        // Instance properties first, in most-derived-then-inherited order.
        for &ci in &chain {
            for (pname, vis) in &self.classes[ci].own_prop_vis {
                if seen.iter().any(|n| n == pname) {
                    continue;
                }
                seen.push(pname.clone());
                if !visible_from(&self.classes, cur, *vis, ci) {
                    continue;
                }
                // A *virtual* hooked property has no backing storage (absent from the
                // instance), so it is excluded — only backed properties are listed.
                // The default lives under the declaration's *storage* key (mangled
                // for a private); it surfaces under the source-level name.
                let skey: Vec<u8> = match vis {
                    Visibility::Private => php_types::mangle_prop_key(&self.classes[ci].name, pname),
                    _ => pname.to_vec(),
                };
                if let Some((_, val)) = defaults.iter().find(|(n, _)| n.as_ref() == skey.as_slice()) {
                    arr.insert(Key::from_bytes(pname), val.clone());
                }
            }
        }
        // Then static properties, in their declared-default value.
        for &ci in &chain {
            for sp in &self.classes[ci].static_props {
                if seen.iter().any(|n| *n == sp.name) {
                    continue;
                }
                seen.push(sp.name.clone());
                if !visible_from(&self.classes, cur, sp.visibility, ci) {
                    continue;
                }
                let val = match &sp.init {
                    StaticInit::Const(k) => k.to_zval(),
                    StaticInit::Thunk(_) => Zval::Null,
                };
                arr.insert(Key::from_bytes(&sp.name), val);
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `register_shutdown_function(callable $callback, mixed ...$args)`: queue
    /// `$callback` (with any bound `$args`) to run at script end, in registration
    /// order. Returns `null`.
    pub(super) fn ho_register_shutdown_function(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = args.into_iter();
        let Some(cb) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "register_shutdown_function() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let bound: Vec<Zval> = it.map(|a| a.deref_clone()).collect();
        self.shutdown_fns.push((cb.deref_clone(), bound));
        Ok(Zval::Null)
    }
    /// `get_class_methods($object_or_class)` (Session B2): the class's method names,
    /// walking the inheritance chain child→parent (each name once, child overrides
    /// win), filtered by visibility from the calling scope. An unresolvable
    /// class-name string is a `TypeError` (PHP 8.5; eval returns null → VM ≥ eval).
    /// Interface/abstract-only method names are a scope-out (not carried on the
    /// compiled class). Mirrors `eval::ci_get_class_methods` for concrete methods.
    pub(super) fn ho_get_class_methods(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "get_class_methods() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let start = self.class_arg_to_id(a.deref_clone(), "get_class_methods")?;
        let cur = self.frames[self.frames.len() - 1].class;
        let mut arr = PhpArray::new();
        let mut seen: Vec<Vec<u8>> = Vec::new();
        let mut ifaces: Vec<ClassId> = Vec::new();
        let mut c = Some(start);
        while let Some(cc) = c {
            // Concrete methods first, then the class's own abstract signatures
            // (both live in Zend's per-class function table).
            for m in self.classes[cc].methods.iter().chain(&self.classes[cc].abstract_sigs) {
                let lname = m.name.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue; // a more-derived class already defined this name
                }
                seen.push(lname);
                if visible_from(&self.classes, cur, m.visibility, cc) {
                    let _ = arr.append(Zval::Str(PhpStr::new(m.name.to_vec())));
                }
            }
            ifaces.extend(self.classes[cc].interfaces.iter().copied());
            c = self.classes[cc].parent;
        }
        // Interface-declared methods the chain never (re)declared: Zend copies
        // them into the implementing class's function table (always public).
        let mut i = 0;
        while i < ifaces.len() {
            let cc = ifaces[i];
            i += 1;
            for m in &self.classes[cc].abstract_sigs {
                let lname = m.name.to_ascii_lowercase();
                if !seen.contains(&lname) {
                    seen.push(lname);
                    let _ = arr.append(Zval::Str(PhpStr::new(m.name.to_vec())));
                }
            }
            for &n in &self.classes[cc].interfaces {
                if !ifaces.contains(&n) {
                    ifaces.push(n);
                }
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `function_exists($name)` (Session B4): whether `name` is a user function, a
    /// registry builtin, or a host builtin. A leading `\` is stripped.
    pub(super) fn ho_function_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(a) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "function_exists() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
        let b = raw.as_bytes();
        let name = b.strip_prefix(b"\\").unwrap_or(b);
        Ok(Zval::Bool(self.is_name_callable(name)))
    }
    pub(super) fn ho_class_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cid = self.resolve_named_class_with_autoload(&args)?;
        if matches!(cid, Some(c) if self.classes[c].instantiable != Instantiable::Interface) {
            return Ok(Zval::Bool(true));
        }
        // `Generator` and `Closure` are always-present engine classes phpr models
        // as special zvals (no ClassId), so the ClassId lookup misses them —
        // class_exists() must still report them present (PHPUnit's
        // assertInstanceOf(Generator::class, …): QueryTest::testIterateWithDistinct).
        if let Some(v) = args.first() {
            let n = convert::to_zstr_cast(v, &mut self.diags);
            let n = n.as_bytes().strip_prefix(b"\\").unwrap_or(n.as_bytes());
            if n.eq_ignore_ascii_case(b"Generator") || n.eq_ignore_ascii_case(b"Closure") {
                return Ok(Zval::Bool(true));
            }
        }
        Ok(Zval::Bool(false))
    }
    /// `class_alias(string $class, string $alias, bool $autoload = true): bool` —
    /// register `$alias` as another name for the existing class/interface `$class`
    /// by pointing it at the same class id, so `new $alias`, `instanceof`,
    /// `class_exists($alias)` and static access all resolve identically (instances
    /// still report the original class name). `false` (with a warning) if `$class`
    /// does not exist or `$alias` is already taken.
    pub(super) fn ho_class_alias(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(a0), Some(a1)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(
                "class_alias() expects at least 2 arguments".to_string(),
            ));
        };
        let autoload = args.get(2).is_none_or(|v| convert::to_bool(v, &mut self.diags));
        let orig_s = convert::to_zstr_cast(&a0.deref_clone(), &mut self.diags);
        let orig = orig_s.as_bytes().strip_prefix(b"\\").unwrap_or(orig_s.as_bytes()).to_vec();
        let alias_s = convert::to_zstr_cast(&a1.deref_clone(), &mut self.diags);
        let alias = alias_s.as_bytes().strip_prefix(b"\\").unwrap_or(alias_s.as_bytes()).to_vec();
        let cid = if autoload {
            self.resolve_class_autoload(&orig)?
        } else {
            self.class_index.get(&orig.to_ascii_lowercase()).copied()
        };
        let Some(cid) = cid else {
            self.diags.push(Diag::Warning(format!(
                "Class \"{}\" not found",
                String::from_utf8_lossy(&orig)
            )));
            return Ok(Zval::Bool(false));
        };
        let alias_key = alias.to_ascii_lowercase();
        if self.class_index.contains_key(&alias_key) {
            self.diags.push(Diag::Warning(format!(
                "Cannot declare class {}, because the name is already in use",
                String::from_utf8_lossy(&alias)
            )));
            return Ok(Zval::Bool(false));
        }
        self.class_index.insert(alias_key, cid);
        // Make the alias visible to the *lowering image* too, so a later unit
        // can `extends`/`implements` it (monolog's tests alias the legacy
        // PHPUnit_Framework_TestCase). An index-only entry: the alias resolves
        // to the ORIGINAL decl, so inherited private-property mangling keeps
        // the real declaring-class name and class identity is preserved.
        self.seed_aliases.push((alias, orig));
        Ok(Zval::Bool(true))
    }
    pub(super) fn ho_interface_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cid = self.resolve_named_class_with_autoload(&args)?;
        let is_iface = matches!(cid, Some(c) if self.classes[c].instantiable == Instantiable::Interface);
        Ok(Zval::Bool(is_iface))
    }
    /// `method_exists($object_or_class, $method)` (Session B4): whether the class of
    /// the object / named class defines `method` (walking the inheritance chain). An
    /// unresolvable target is `false` (no error, unlike `get_class_methods`).
    pub(super) fn ho_method_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(a0), Some(a1)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(
                "method_exists() expects exactly 2 arguments".to_string(),
            ));
        };
        let Some(cid) = self.class_id_from_value_autoload(&a0.deref_clone())? else {
            return Ok(Zval::Bool(false));
        };
        let m = convert::to_zstr_cast(&a1.deref_clone(), &mut self.diags);
        // Abstract/interface-declared methods exist in Zend's function table
        // even without a body — the reflect walk covers them.
        Ok(Zval::Bool(
            resolve_method_runtime(&self.classes, cid, m.as_bytes()).is_some()
                || self.find_method_reflect(cid, m.as_bytes()).is_some(),
        ))
    }
    /// `property_exists($object_or_class, $property)` (Session B4): whether the class
    /// declares an instance or static `property` (any visibility) — or, for an object
    /// argument, whether the instance carries it as a dynamic property. Mirrors PHP:
    /// visibility is ignored, an unresolvable target is `false`.
    pub(super) fn ho_property_exists(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(a0), Some(a1)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(
                "property_exists() expects exactly 2 arguments".to_string(),
            ));
        };
        let v = a0.deref_clone();
        let Some(cid) = self.class_id_from_value_autoload(&v)? else {
            return Ok(Zval::Bool(false));
        };
        let pname_z = convert::to_zstr_cast(&a1.deref_clone(), &mut self.diags);
        let pname = pname_z.as_bytes();
        if prop_vis_decl(&self.classes, cid, pname).is_some()
            || find_static_prop(&self.classes, cid, pname).is_some()
        {
            return Ok(Zval::Bool(true));
        }
        if let Zval::Object(o) = &v {
            if o.borrow().props.get(pname).is_some() {
                return Ok(Zval::Bool(true));
            }
        }
        Ok(Zval::Bool(false))
    }
    /// `get_called_class()` (Session B4): the late-static-binding class name (the
    /// receiver's actual class), a fatal `Error` outside class context.
    pub(super) fn ho_get_called_class(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        match self.frames[top].static_class {
            Some(cid) => Ok(Zval::Str(PhpStr::new(self.classes[cid].name.to_vec()))),
            None => Err(PhpError::Error(
                "get_called_class() must be called from within a class".to_string(),
            )),
        }
    }
    /// `preg_replace_callback($pattern, $callback, $subject)` (Session 3): replace
    /// each match of `pattern` in `subject` with the string returned by `callback`
    /// (called with the match array). A single pattern/subject, mirroring
    /// `eval::ho_preg_replace_callback`; the callback runs via `call_callable` and
    /// its result is stringified (honouring `__toString`). An invalid pattern yields
    /// null. The optional `limit`/`count` arguments are a scope-out.
    pub(super) fn ho_preg_replace_callback(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace_callback() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let callback = args[1].deref_clone();
        let subject =
            convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = self.preg_compile(&pat) else {
            return Ok(Zval::Null);
        };
        // Byte-true subject handling: an invalid-UTF-8 subject fails a /u
        // pattern (PHP: null) or round-trips through latin1 for a byte-mode
        // one; a latin1-fixed capture set carries original-byte offsets, so
        // the splice below always slices the ORIGINAL subject bytes.
        let unicode = crate::preg::pattern_is_unicode(&pat);
        let Some(txt) = crate::preg::subject_text(&subject, unicode) else {
            return Ok(Zval::Null);
        };
        let latin1 = txt.is_latin1();
        let subj = txt.as_str().to_owned();
        // Collect (range, match-array) up front so the regex borrow of `subj` ends
        // before we re-enter the VM via the callback.
        let hits: Vec<(usize, usize, Zval)> = re
            .captures_iter(&subj)
            .into_iter()
            .map(|mut caps| {
                if latin1 {
                    caps.latin1_fix(&subj);
                }
                let m0 = caps.get(0).expect("match has group 0");
                (m0.start, m0.end, crate::preg::captures_array(&re, &caps, 0))
            })
            .collect();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        for (start, end, match_arr) in hits {
            out.extend_from_slice(&subject[last..start]);
            let ret = self.call_callable(callback.clone(), vec![match_arr])?;
            let rs = self.vm_stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&subject[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }
    /// `preg_replace($pattern, $replacement, $subject, $limit, &$count)`:
    /// backreferences `$1`/`${1}`/`\1` in the replacement are honoured. Returns
    /// `(result, count)` — `&$count` (the number of replacements, symfony
    /// polyfill-intl-grapheme's `$len`) is written by the VM out-param path; the
    /// plain dispatch drops it. Returns `null` on a bad pattern. Single scalar
    /// pattern/subject and `$limit` stay a scope-out. Mirrors `eval::ho_preg_replace`.
    pub(super) fn ho_preg_replace(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace() expects at least 3 arguments".to_string(),
            ));
        }
        // `$pattern` may be an array; `$replacement` pairs with it in iteration
        // order (a shorter replacement array pads with "", Zend's rule), while a
        // string replacement broadcasts over every pattern. A replacement array
        // against a string pattern is the PHP 8 TypeError.
        let pattern_val = args[0].deref_clone();
        let repl_val = args[1].deref_clone();
        let pats: Vec<Vec<u8>> = match &pattern_val {
            Zval::Array(a) => a
                .iter()
                .map(|(_, v)| {
                    convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec()
                })
                .collect(),
            other => {
                vec![convert::to_zstr_cast(other, &mut self.diags).as_bytes().to_vec()]
            }
        };
        let repls: Vec<Vec<u8>> = match &repl_val {
            Zval::Array(a) => {
                if !matches!(pattern_val, Zval::Array(_)) {
                    return Err(PhpError::TypeError(
                        "preg_replace(): Argument #2 ($replacement) must be of type string when argument #1 ($pattern) is a string".to_string(),
                    ));
                }
                a.iter()
                    .map(|(_, v)| {
                        convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec()
                    })
                    .collect()
            }
            other => {
                let one = convert::to_zstr_cast(other, &mut self.diags).as_bytes().to_vec();
                vec![one; pats.len()]
            }
        };
        // Compile every pattern up front: any bad pattern nulls the whole call.
        // The translated replacement stays as bytes so it can enter whichever
        // text domain the subject picks (UTF-8, or latin1 for a binary one).
        let mut pairs = Vec::with_capacity(pats.len());
        let mut any_unicode = false;
        for (i, p) in pats.iter().enumerate() {
            let Some(re) = self.preg_compile(p) else {
                return Ok((Zval::Null, Zval::Long(0)));
            };
            any_unicode |= crate::preg::pattern_is_unicode(p);
            let r = repls.get(i).map(|r| r.as_slice()).unwrap_or(b"");
            pairs.push((re, crate::preg::translate_replacement(r)));
        }
        let mut total = 0i64;
        let run_one = |subject: &[u8], total: &mut i64| -> Zval {
            let Some(txt) = crate::preg::subject_text(subject, any_unicode) else {
                // Invalid UTF-8 under a /u pattern: PHP nulls this subject.
                return Zval::Null;
            };
            let latin1 = txt.is_latin1();
            let mut s = txt.as_str().to_owned();
            for (re, repl) in &pairs {
                let repl = if latin1 {
                    crate::preg::latin1_decode(repl)
                } else {
                    String::from_utf8_lossy(repl).into_owned()
                };
                *total += re.captures_iter(&s).len() as i64;
                s = re.replace_all(&s, repl.as_str()).to_string();
            }
            let out = if latin1 { crate::preg::latin1_encode(&s) } else { s.into_bytes() };
            Zval::Str(PhpStr::new(out))
        };
        // An array `$subject` maps per entry (keys preserved); a scalar maps to
        // a string result.
        match args[2].deref_clone() {
            Zval::Array(subjects) => {
                let entries: Vec<(Key, Vec<u8>)> = subjects
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            convert::to_zstr_cast(&v.deref_clone(), &mut self.diags)
                                .as_bytes()
                                .to_vec(),
                        )
                    })
                    .collect();
                let mut out = PhpArray::new();
                for (k, s) in entries {
                    let r = run_one(&s, &mut total);
                    out.insert(k, r);
                }
                Ok((Zval::Array(Rc::new(out)), Zval::Long(total)))
            }
            other => {
                let s = convert::to_zstr_cast(&other, &mut self.diags).as_bytes().to_vec();
                let r = run_one(&s, &mut total);
                Ok((r, Zval::Long(total)))
            }
        }
    }
    /// `preg_quote($str, $delimiter = null)`: escape regex metacharacters (and the
    /// optional delimiter). Mirrors `eval::ho_preg_quote` on `crate::preg::quote`.
    pub(super) fn ho_preg_quote(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "preg_quote() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let s = convert::to_zstr_cast(&first.deref_clone(), &mut self.diags).as_bytes().to_vec();
        let delim = match args.get(1) {
            Some(a) => convert::to_zstr_cast(&a.deref_clone(), &mut self.diags)
                .as_bytes()
                .first()
                .copied(),
            None => None,
        };
        Ok(Zval::Str(PhpStr::new(crate::preg::quote(&s, delim))))
    }
    /// `preg_grep($pattern, $array, $flags = 0)`: return the entries of `$array`
    /// whose (string-cast) value matches `$pattern`, preserving keys. With
    /// `PREG_GREP_INVERT` (1) the *non-matching* entries are returned instead.
    /// `false` on a bad pattern. Mirrors the shared `crate::preg` engine.
    pub(super) fn ho_preg_grep(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_grep() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let arr = match args[1].deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "preg_grep(): Argument #2 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        let flags = match args.get(2) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let invert = flags & 1 != 0; // PREG_GREP_INVERT
        let Some(re) = self.preg_compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let entries: Vec<(Key, Zval)> =
            arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
        let mut out = PhpArray::new();
        let unicode = crate::preg::pattern_is_unicode(&pat);
        for (k, v) in entries {
            let subj = convert::to_zstr_cast(&v, &mut self.diags).as_bytes().to_vec();
            // An invalid-UTF-8 entry under /u simply doesn't match (PHP flags
            // PREG_BAD_UTF8_ERROR but keeps filtering).
            let Some(txt) = crate::preg::subject_text(&subj, unicode) else {
                if invert {
                    out.insert(k, v);
                }
                continue;
            };
            let matched = re.captures_at(txt.as_str(), 0).is_some();
            if matched != invert {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `random_int(int $min, int $max): int` — a uniformly distributed random
    /// integer in `[$min, $max]`, drawn from the OS CSPRNG. Mirrors PHP's
    /// rejection-sampling algorithm (`php_random_range64`) to avoid modulo bias.
    pub(super) fn ho_random_int(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "random_int() expects exactly 2 arguments, {} given",
                args.len()
            )));
        }
        let min = convert::to_long_cast(&args[0].deref_clone(), &mut self.diags);
        let max = convert::to_long_cast(&args[1].deref_clone(), &mut self.diags);
        if min > max {
            return Err(PhpError::ValueError(
                "random_int(): Argument #1 ($min) must be less than or equal to argument #2 ($max)"
                    .to_string(),
            ));
        }
        // Unsigned span between the bounds (two's-complement subtraction keeps
        // it correct even when the range straddles zero or is the full i64).
        let umax = (max as u64).wrapping_sub(min as u64);
        let result = os_random_range64(umax)?;
        Ok(Zval::Long(min.wrapping_add(result as i64)))
    }
    /// `random_bytes(int $length): string` — `$length` cryptographically secure
    /// random bytes from the OS CSPRNG. `ValueError` when `$length < 1`.
    pub(super) fn ho_random_bytes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "random_bytes() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let len = convert::to_long_cast(&first.deref_clone(), &mut self.diags);
        if len < 1 {
            return Err(PhpError::ValueError(
                "random_bytes(): Argument #1 ($length) must be greater than 0".to_string(),
            ));
        }
        let mut buf = vec![0u8; len as usize];
        os_random_fill(&mut buf)?;
        Ok(Zval::Str(PhpStr::new(buf)))
    }
    /// `strtok([$string,] $token)`: split a string into tokens on any byte in
    /// `$token`. The two-argument form sets the internal string and cursor; the
    /// one-argument form resumes from the saved cursor. Leading delimiters are
    /// skipped; `false` is returned when the string is exhausted (or a
    /// one-argument call has no prior string). Mirrors PHP's stateful tokenizer.
    pub(super) fn ho_strtok(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let delims = match args.len() {
            0 => {
                return Err(PhpError::ArgumentCountError(
                    "strtok() expects at least 1 argument, 0 given".to_string(),
                ))
            }
            1 => convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags)
                .as_bytes()
                .to_vec(),
            _ => {
                let s = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags)
                    .as_bytes()
                    .to_vec();
                let tok = convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags)
                    .as_bytes()
                    .to_vec();
                self.strtok = Some((s, 0));
                tok
            }
        };
        let Some((buf, pos)) = self.strtok.as_mut() else {
            return Ok(Zval::Bool(false));
        };
        // Skip leading delimiters.
        while *pos < buf.len() && delims.contains(&buf[*pos]) {
            *pos += 1;
        }
        if *pos >= buf.len() {
            return Ok(Zval::Bool(false));
        }
        let start = *pos;
        while *pos < buf.len() && !delims.contains(&buf[*pos]) {
            *pos += 1;
        }
        Ok(Zval::Str(PhpStr::new(buf[start..*pos].to_vec())))
    }
    /// `preg_split($pattern, $subject, $limit = -1, $flags = 0)`: split `$subject`
    /// on matches of `$pattern`. Returns `false` on a bad pattern. Mirrors
    /// `eval::ho_preg_split` on the shared `crate::preg` engine (no-empty /
    /// delim-capture / offset-capture flags honoured; positive limit caps pieces).
    pub(super) fn ho_preg_split(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_split() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let limit = match args.get(2) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => -1,
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let Some(re) = self.preg_compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let no_empty = flags & 1 != 0;
        let delim_capture = flags & 2 != 0;
        let offset_capture = flags & 4 != 0;
        // Byte-true subject handling (see preg::subject_text): pieces always
        // slice the ORIGINAL bytes — a latin1-fixed capture set carries
        // original-byte offsets, and for a valid-UTF-8 subject the two byte
        // domains coincide.
        let unicode = crate::preg::pattern_is_unicode(&pat);
        let Some(txt) = crate::preg::subject_text(&subject, unicode) else {
            return Ok(Zval::Bool(false));
        };
        let latin1 = txt.is_latin1();
        let subj = txt.as_str().to_owned();
        let mut arr = PhpArray::new();
        let mut last = 0usize;
        let push = |arr: &mut PhpArray, text: &[u8], off: usize| {
            if no_empty && text.is_empty() {
                return;
            }
            if offset_capture {
                let _ = arr.append(crate::preg::offset_pair(
                    Zval::Str(PhpStr::new(text.to_vec())),
                    off as i64,
                ));
            } else {
                let _ = arr.append(Zval::Str(PhpStr::new(text.to_vec())));
            }
        };
        for (idx, mut caps) in re.captures_iter(&subj).into_iter().enumerate() {
            if latin1 {
                caps.latin1_fix(&subj);
            }
            let m0 = caps.get(0).unwrap();
            if limit > 0 && idx as i64 + 1 >= limit {
                break;
            }
            push(&mut arr, &subject[last..m0.start], last);
            if delim_capture {
                for g in 1..caps.len() {
                    if let Some(mm) = caps.get(g) {
                        push(&mut arr, &mm.text, mm.start);
                    }
                }
            }
            last = m0.end;
        }
        push(&mut arr, &subject[last..], last);
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `parse_str($string, &$result)`: parse a URL query string into an array
    /// (urldecoded keys/values, `k[]`/`k[sub]` nesting; PHP mangles `.`/` ` to
    /// `_` in *top-level* names). Returns `(null, result_array)` for the VM
    /// out-param path.
    pub(super) fn ho_parse_str(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
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
        let src = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        let mut result = PhpArray::new();
        for pair in src.as_bytes().split(|&b| b == b'&') {
            if pair.is_empty() {
                continue;
            }
            let (raw_key, raw_val) = match pair.iter().position(|&b| b == b'=') {
                Some(eq) => (&pair[..eq], &pair[eq + 1..]),
                None => (pair, &pair[pair.len()..]),
            };
            let key = urldecode(raw_key);
            let val = Zval::Str(PhpStr::new(urldecode(raw_val)));
            // Split `name[a][b]…` into the base name and bracket path.
            let (base_end, mut path): (usize, Vec<Option<Vec<u8>>>) =
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
            if base.is_empty() {
                continue;
            }
            if path.is_empty() {
                result.insert(Key::from_bytes(&base), val);
                continue;
            }
            // Walk/create the nested arrays for the bracket path.
            path.insert(0, Some(std::mem::take(&mut base)));
            fn descend(arr: &mut PhpArray, path: &[Option<Vec<u8>>], val: Zval) {
                let (head, rest) = path.split_first().expect("non-empty path");
                if rest.is_empty() {
                    match head {
                        Some(k) => arr.insert(Key::from_bytes(k), val),
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
                                if *i >= next {
                                    next = i + 1;
                                }
                            }
                        }
                        Key::Int(next)
                    }
                };
                let entry = match arr.get(&key) {
                    Some(Zval::Array(a)) => (**a).clone(),
                    _ => PhpArray::new(),
                };
                let mut entry = entry;
                descend(&mut entry, rest, val);
                arr.insert(key, Zval::Array(Rc::new(entry)));
            }
            descend(&mut result, &path, val);
        }
        Ok((Zval::Null, Zval::Array(Rc::new(result))))
    }
    /// `preg_match($pattern, $subject, &$matches = null, $flags = 0)`: returns 1 on
    /// a match, 0 on none, `false` on a bad pattern. Yields `(ret, matches_array)`;
    /// `$matches` is written by the VM out-param path. Mirrors `eval::ho_preg_match`.
    pub(super) fn ho_preg_match(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = self.preg_compile(&pat) else {
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        // 5th arg `$offset`: byte offset to start matching at (negative counts
        // from the end). The whole subject stays visible so `^`/lookbehind anchor
        // to the true start; an out-of-range offset just yields no match.
        let off = match args.get(4) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let start = if off < 0 {
            // A negative offset counts from the end and clamps to the start.
            (subject.len() as i64 + off).max(0) as usize
        } else {
            // A positive offset past the end is an error → `false` (PHP).
            if off as usize > subject.len() {
                return Ok((Zval::Bool(false), Zval::Null));
            }
            off as usize
        };
        let unicode = crate::preg::pattern_is_unicode(&pat);
        let Some(txt) = crate::preg::subject_text(&subject, unicode) else {
            // Invalid UTF-8 under /u: PREG_BAD_UTF8_ERROR → false, like PHP.
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let latin1 = txt.is_latin1();
        let subj = txt.as_str();
        let (ret, matches) = match re.captures_at(subj, start) {
            Some(mut caps) => {
                if latin1 {
                    caps.latin1_fix(subj);
                }
                (1, crate::preg::captures_array(&re, &caps, flags))
            }
            None => (0, Zval::Array(Rc::new(PhpArray::new()))),
        };
        Ok((Zval::Long(ret), matches))
    }
    /// `preg_match_all($pattern, $subject, &$matches = null, $flags = 0)`: default
    /// PREG_PATTERN_ORDER — `$matches[g]` is group `g`'s text across all matches;
    /// PREG_SET_ORDER gives one full match array per match. Returns the match count
    /// (or `false` on a bad pattern). Mirrors `eval::ho_preg_match_all`.
    pub(super) fn ho_preg_match_all(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        use crate::preg::{capture_value, PREG_OFFSET_CAPTURE, PREG_SET_ORDER, PREG_UNMATCHED_AS_NULL};
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match_all() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = convert::to_zstr_cast(&args[0].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let subject =
            convert::to_zstr_cast(&args[1].deref_clone(), &mut self.diags).as_bytes().to_vec();
        let Some(re) = self.preg_compile(&pat) else {
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let flags = match args.get(3) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 0,
        };
        let unicode = crate::preg::pattern_is_unicode(&pat);
        let Some(txt) = crate::preg::subject_text(&subject, unicode) else {
            return Ok((Zval::Bool(false), Zval::Null));
        };
        let latin1 = txt.is_latin1();
        let subj = txt.as_str().to_owned();
        let offset = flags & PREG_OFFSET_CAPTURE != 0;
        let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
        let mut count: i64 = 0;
        let outer = if flags & PREG_SET_ORDER != 0 {
            let mut outer = PhpArray::new();
            for mut caps in re.captures_iter(&subj) {
                if latin1 {
                    caps.latin1_fix(&subj);
                }
                count += 1;
                let _ = outer.append(crate::preg::captures_array(&re, &caps, flags));
            }
            outer
        } else {
            let ngroups = re.captures_len();
            let names = re.capture_names();
            let mut cols: Vec<PhpArray> = (0..ngroups).map(|_| PhpArray::new()).collect();
            for mut caps in re.captures_iter(&subj) {
                if latin1 {
                    caps.latin1_fix(&subj);
                }
                count += 1;
                for (g, col) in cols.iter_mut().enumerate() {
                    let _ = col.append(capture_value(caps.get(g), offset, as_null));
                }
            }
            let mut outer = PhpArray::new();
            for (g, col) in cols.into_iter().enumerate() {
                let col_z = Zval::Array(Rc::new(col));
                if let Some(Some(name)) = names.get(g) {
                    outer.insert(Key::from_bytes(name.as_bytes()), col_z.clone());
                }
                outer.insert(Key::Int(g as i64), col_z);
            }
            outer
        };
        Ok((Zval::Long(count), Zval::Array(Rc::new(outer))))
    }
    /// `error_reporting($level = null)` (Session 1): set the active reporting
    /// bitmask (consulted by [`Self::flush_diags`]) and return the previous one; a
    /// `null`/absent argument reads without changing it.
    pub(super) fn ho_error_reporting(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let old = self.error_level;
        if let Some(a) = args.first() {
            let v = a.deref_clone();
            if !matches!(v, Zval::Null) {
                self.error_level = convert::to_long_cast(&v, &mut self.diags);
            }
        }
        Ok(Zval::Long(old))
    }
    /// `trigger_error($message, $level = E_USER_NOTICE)` (Session 1): raise a user
    /// diagnostic. `E_USER_ERROR` becomes a fatal; the others render as
    /// Warning/Notice/Deprecated (gated by `error_reporting`). An invalid level is a
    /// `ValueError`. Records the error for [`Self::ho_error_get_last`].
    pub(super) fn ho_trigger_error(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(msg_arg) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "trigger_error() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let msg = convert::to_zstr_cast(&msg_arg.deref_clone(), &mut self.diags).as_bytes().to_vec();
        let level = match args.get(1) {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 1024, // E_USER_NOTICE
        };
        if !matches!(level, 256 | 512 | 1024 | 16384) {
            return Err(PhpError::ValueError(
                "trigger_error(): Argument #2 ($error_level) must be one of E_USER_ERROR, E_USER_WARNING, E_USER_NOTICE, or E_USER_DEPRECATED"
                    .to_string(),
            ));
        }
        let line = self.cur_line(self.frames.len() - 1);
        if level == 256 {
            self.flush_diags(line)?;
            // PHP 8.4+: passing E_USER_ERROR to trigger_error() is itself deprecated.
            // The oracle emits this E_DEPRECATED first (routed to any handler too),
            // *then* processes the E_USER_ERROR — so a handler sees both 8192 and 256.
            self.raise_diagnostic(
                8192,
                "Passing E_USER_ERROR to trigger_error() is deprecated since 8.4, throw an exception or call exit with a string message instead",
                line,
            )?;
            let message = String::from_utf8_lossy(&msg).into_owned();
            // If a handler is registered for E_USER_ERROR and handles it (truthy
            // return), the script CONTINUES (oracle-confirmed; error_get_last stays
            // unset, mirroring a handler-suppressed diagnostic). Otherwise — no
            // handler, masked out, or a `false` return — it is the fatal: record
            // `last_error` (the default/fatal handler ran) and propagate.
            if let Some(true) = self.route_to_handler(256, &message, line)? {
                return Ok(Zval::Bool(true));
            }
            self.last_error = Some((level, msg.clone(), line));
            return Err(PhpError::Error(message));
        }
        // Flush any pending built-in diagnostics, then route this user diagnostic
        // through the shared chokepoint so a `set_error_handler` callback sees it.
        // The default render is gated on the user level itself (E_USER_*), not the
        // label's built-in bit, since e.g. E_USER_DEPRECATED (16384) and
        // E_DEPRECATED (8192) are independent.
        self.flush_diags(line)?;
        let message = String::from_utf8_lossy(&msg).into_owned();
        self.raise_diagnostic(level, &message, line)?;
        Ok(Zval::Bool(true))
    }
    /// `error_get_last()`: the most recent diagnostic as `[type, message, file,
    /// line]`, or null. Captures both `trigger_error` and built-in warnings/notices
    /// (Session 2; recorded at the [`Self::raise_diagnostic`] chokepoint).
    pub(super) fn ho_error_get_last(&mut self) -> Result<Zval, PhpError> {
        // Realize any diagnostic still pending in `self.diags`: the VM flushes diags
        // lazily (at the next echo/builtin), so a warning raised mid-expression has
        // not yet updated `last_error` when `error_get_last()` is read right after it.
        // Flushing here — the same realize-state move `emit_str`/`run_value_builtin`
        // make — captures it (mirrors PHP's synchronous-at-emission `last_error`).
        let line = self.cur_line(self.frames.len() - 1);
        self.flush_diags(line)?;
        match &self.last_error {
            Some((level, msg, line)) => {
                let mut arr = PhpArray::new();
                arr.insert(Key::from_bytes(b"type"), Zval::Long(*level));
                arr.insert(Key::from_bytes(b"message"), Zval::Str(PhpStr::new(msg.clone())));
                arr.insert(
                    Key::from_bytes(b"file"),
                    Zval::Str(PhpStr::new(self.module.file.to_vec())),
                );
                arr.insert(Key::from_bytes(b"line"), Zval::Long(*line as i64));
                Ok(Zval::Array(Rc::new(arr)))
            }
            None => Ok(Zval::Null),
        }
    }
    /// `set_exception_handler($callable)` (Session 1b): install a top-level handler
    /// for uncaught throwables; returns the previously-active handler (or null).
    pub(super) fn ho_set_exception_handler(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.exception_handlers.last().cloned();
        let handler = args.into_iter().next().unwrap_or(Zval::Null);
        self.exception_handlers.push(handler);
        Ok(prev.unwrap_or(Zval::Null))
    }
    /// `restore_exception_handler()` (Session 1b): pop the current handler, making
    /// the previous one active again. Always returns true.
    pub(super) fn ho_restore_exception_handler(&mut self) -> Result<Zval, PhpError> {
        self.exception_handlers.pop();
        Ok(Zval::Bool(true))
    }
    /// `set_error_handler($callable, $levels = E_ALL)` (Session 2): install a
    /// user diagnostic handler routed by [`Self::raise_diagnostic`]; returns the
    /// previously-active handler (or null). The optional level mask gates which
    /// E_* numbers reach the handler (default `E_ALL` = 30719).
    pub(super) fn ho_set_error_handler(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.error_handlers.last().map(|(cb, _)| cb.clone());
        let mut it = args.into_iter();
        let handler = it.next().unwrap_or(Zval::Null);
        let level = match it.next() {
            Some(a) => convert::to_long_cast(&a.deref_clone(), &mut self.diags),
            None => 30719, // E_ALL (PHP 8.5)
        };
        self.error_handlers.push((handler, level));
        Ok(prev.unwrap_or(Zval::Null))
    }
    /// `restore_error_handler()` (Session 2): pop the current handler, re-exposing
    /// the previous one (or the engine default). Always returns true.
    pub(super) fn ho_restore_error_handler(&mut self) -> Result<Zval, PhpError> {
        self.error_handlers.pop();
        Ok(Zval::Bool(true))
    }
    /// `get_error_handler(): ?callable` (8.5): the current user error handler, or
    /// `null` when the engine default is in effect.
    pub(super) fn ho_get_error_handler(&mut self) -> Result<Zval, PhpError> {
        Ok(self.error_handlers.last().map(|(cb, _)| cb.clone()).unwrap_or(Zval::Null))
    }
    /// `get_exception_handler(): ?callable` (8.5): the current top-level exception
    /// handler, or `null`.
    pub(super) fn ho_get_exception_handler(&mut self) -> Result<Zval, PhpError> {
        Ok(self.exception_handlers.last().cloned().unwrap_or(Zval::Null))
    }
    /// `serialize($value)`: the pure formatter (php-builtins) does the encoding;
    /// this host wrapper first runs the object hooks (`__serialize`/`__sleep`)
    /// the pure side cannot call. Hook-free graphs pass through untouched.
    pub(super) fn ho_serialize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "serialize() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let v = first.deref_clone();
        let prepared = if self.has_serialize_hooks(&v, &mut Vec::new()) {
            self.prepare_serialize(v, &mut HashMap::new())?
        } else {
            v
        };
        let f = match self.registry.get(&b"serialize"[..]) {
            Some(Builtin::Value(f)) => *f,
            _ => return Err(PhpError::Error("serialize builtin unavailable".to_string())),
        };
        let line = self.cur_line(self.frames.len() - 1);
        self.run_value_builtin(f, &[prepared], line)
    }
    /// `unserialize($str)`: rebuild a value from PHP's serialization format. A
    /// host builtin because reconstructing an object needs the class table and id
    /// allocator. Mirrors `eval::ho_unserialize`: the shared
    /// [`crate::unserialize::parse`] decodes a pure [`Ser`](crate::unserialize::Ser)
    /// tree, then [`Self::vm_ser_to_zval`] materialises it. Malformed input yields
    /// `false` with PHP's Warning. `__wakeup` is not called (D-50 scope-out), and an
    /// unknown class falls back to `stdClass` (PHP makes a `__PHP_Incomplete_Class`).
    pub(super) fn ho_unserialize(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "unserialize() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arg0 = first.deref_clone();
        let bytes = convert::to_zstr_cast(&arg0, &mut self.diags);
        let nbytes = bytes.as_bytes().len();
        match crate::unserialize::parse(bytes.as_bytes()) {
            Some(s) => self.vm_ser_to_zval(s),
            None => {
                // PHP reports the failing offset; we do not track it, so report 0
                // (matches `eval`, D-50).
                self.diags.push(Diag::Warning(format!(
                    "unserialize(): Error at offset 0 of {nbytes} bytes"
                )));
                Ok(Zval::Bool(false))
            }
        }
    }
    pub(super) fn ho_debug_backtrace(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        let frames = self.collect_backtrace();
        let mut outer = PhpArray::new();
        for bt in frames {
            let mut e = PhpArray::new();
            e.insert(Key::from_bytes(b"file"), Zval::Str(PhpStr::new(bt.file.clone())));
            e.insert(Key::from_bytes(b"line"), Zval::Long(bt.line as i64));
            e.insert(Key::from_bytes(b"function"), Zval::Str(PhpStr::new(bt.function)));
            if let Some(cls) = bt.class {
                e.insert(Key::from_bytes(b"class"), Zval::Str(PhpStr::new(cls)));
                if let Some(obj) = bt.object {
                    e.insert(Key::from_bytes(b"object"), obj);
                }
                let ty: &[u8] = if bt.is_static { b"::" } else { b"->" };
                e.insert(Key::from_bytes(b"type"), Zval::Str(PhpStr::new(ty.to_vec())));
            }
            // PHP's `eval` frame carries no `args` entry.
            if !bt.is_eval {
                let mut argsarr = PhpArray::new();
                for a in bt.args {
                    let _ = argsarr.append(a);
                }
                e.insert(Key::from_bytes(b"args"), Zval::Array(Rc::new(argsarr)));
            }
            let _ = outer.append(Zval::Array(Rc::new(e)));
        }
        Ok(Zval::Array(Rc::new(outer)))
    }
    pub(super) fn ho_debug_print_backtrace(&mut self) -> Result<Zval, PhpError> {
        let frames = self.collect_backtrace();
        let mut s = String::new();
        for (n, bt) in frames.iter().enumerate() {
            let file = String::from_utf8_lossy(&bt.file);
            let callee = match &bt.class {
                Some(cls) => format!(
                    "{}{}{}",
                    String::from_utf8_lossy(cls),
                    if bt.is_static { "::" } else { "->" },
                    String::from_utf8_lossy(&bt.function)
                ),
                None => String::from_utf8_lossy(&bt.function).into_owned(),
            };
            let argstr = bt
                .args
                .iter()
                .map(format_bt_arg)
                .collect::<Vec<_>>()
                .join(", ");
            s.push_str(&format!("#{n} {file}({}): {callee}({argstr})\n", bt.line));
        }
        // Flush pending diagnostics first so the trace lands in output order, then
        // append to both streams (this is ordinary output, like an echo).
        let line = self.cur_line(self.frames.len() - 1);
        self.flush_diags(line)?;
        self.write_output(s.as_bytes())?;
        Ok(Zval::Null)
    }
    /// `stream_context_create($options = null, $params = null)`: build a context
    /// resource holding `$options` (a `wrapper => [option => value]` map, e.g.
    /// `['http' => ['method' => 'POST', 'header' => [...]], 'ssl' => [...]]`) for
    /// the stream functions to read. `$params` (stream notifications) is not
    /// modelled. Mirrors PHP's `?array` argument typing.
    pub(super) fn ho_stream_context_create(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let options = match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null | Zval::Undef) => Zval::Array(Rc::new(PhpArray::new())),
            Some(v @ Zval::Array(_)) => v,
            Some(other) => {
                return Err(PhpError::TypeError(format!(
                    "stream_context_create(): Argument #1 ($options) must be of type ?array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        Ok(self.alloc_resource_context(options))
    }

    /// `stream_context_get_options($stream_or_context): array` — the context's
    /// `wrapper => [option => value]` map. A plain stream carries no context in
    /// phpr, so it yields `[]` (as PHP does for a context-less stream).
    pub(super) fn ho_stream_context_get_options(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(arg) = args.first().map(|v| v.deref_clone()) else {
            return Err(PhpError::ArgumentCountError(
                "stream_context_get_options() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        match arg {
            Zval::Resource(rc) => Ok(rc
                .borrow()
                .context_options()
                .map(|o| o.deref_clone())
                .unwrap_or_else(|| Zval::Array(Rc::new(PhpArray::new())))),
            other => Err(PhpError::TypeError(format!(
                "stream_context_get_options(): Argument #1 ($stream_or_context) must be of type resource, {} given",
                other.type_name_for_error()
            ))),
        }
    }

    /// `stream_context_set_option($context, $wrapper, $option, $value)` (4-arg) or
    /// `stream_context_set_option($context, $options)` (2-arg, deprecated in 8.5 —
    /// `stream_context_set_options()` is the replacement). Merges into the context
    /// options and returns `true`.
    pub(super) fn ho_stream_context_set_option(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let deprecated = matches!(args.get(1).map(|v| v.deref_clone()), Some(Zval::Array(_)));
        let r = self.stream_context_set(args, "stream_context_set_option");
        if r.is_ok() && deprecated {
            self.diags.push(Diag::Deprecated(
                "Calling stream_context_set_option() with 2 arguments is deprecated, use stream_context_set_options() instead"
                    .to_string(),
            ));
        }
        r
    }

    /// `stream_context_set_options($context, array $options): bool` (8.5) — the
    /// non-deprecated 2-argument form of [`Self::ho_stream_context_set_option`].
    pub(super) fn ho_stream_context_set_options(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.stream_context_set(args, "stream_context_set_options")
    }

    /// Shared body for `stream_context_set_option(s)`: dispatch the 4-arg
    /// (wrapper/option/value) vs 2-arg (options map) forms and write the context.
    fn stream_context_set(&mut self, args: Vec<Zval>, fname: &str) -> Result<Zval, PhpError> {
        let Some(ctx) = args.first().map(|v| v.deref_clone()) else {
            return Err(PhpError::ArgumentCountError(format!(
                "{fname}() expects at least 2 arguments, 0 given"
            )));
        };
        let Zval::Resource(rc) = ctx else {
            return Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($context) must be of type resource, {} given",
                ctx.type_name_for_error()
            )));
        };
        match args.get(1).map(|v| v.deref_clone()) {
            None => Err(PhpError::ArgumentCountError(format!(
                "{fname}() expects at least 2 arguments, 1 given"
            ))),
            Some(Zval::Array(map)) => {
                // wrapper => [option => value]
                for (wk, wv) in map.iter() {
                    let Zval::Array(sub) = wv.deref_clone() else { continue };
                    let wrapper = key_bytes(wk);
                    for (ok, ov) in sub.iter() {
                        ctx_set_one(&rc, &wrapper, &key_bytes(ok), ov.deref_clone());
                    }
                }
                Ok(Zval::Bool(true))
            }
            Some(wrapper_val) => {
                // string wrapper form needs both $option (#3) and $value (#4)
                if args.len() < 4 {
                    return Err(PhpError::ValueError(format!(
                        "{fname}(): Argument #4 ($value) must be provided when argument #2 ($wrapper_or_options) is a string"
                    )));
                }
                let wrapper = convert::to_zstr_cast(&wrapper_val, &mut self.diags).as_bytes().to_vec();
                let option = convert::to_zstr_cast(&args[2].deref_clone(), &mut self.diags).as_bytes().to_vec();
                ctx_set_one(&rc, &wrapper, &option, args[3].deref_clone());
                Ok(Zval::Bool(true))
            }
        }
    }
    /// `fopen($filename, $mode, …)`: open a real file or a `php://` wrapper and mint
    /// a stream resource. A host builtin because it allocates a resource id. Args 3/4
    /// (use_include_path, context) are a scope-out. On failure: Warning + `false`.
    /// Mirrors `eval::ho_fopen`.
    /// `get_declared_classes()` / `get_declared_interfaces()` (`which` 0 / 1):
    /// names of every linked class table entry of that kind, in declaration
    /// order — prelude first, then user/included units, matching PHP's
    /// internal-then-user ordering. Residue: a compiled-but-not-executed
    /// conditional class is already listed. Consumers (PHPUnit's
    /// TestSuiteLoader) diff the list around a `require`, which this serves.
    pub(super) fn ho_get_declared(&mut self, which: i64) -> Result<Zval, PhpError> {
        let mut arr = PhpArray::new();
        for c in self.classes.iter() {
            let is_iface = matches!(c.instantiable, crate::bytecode::Instantiable::Interface);
            if (which == 1) == is_iface {
                let _ = arr.append(Zval::Str(PhpStr::new(c.name.to_vec())));
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `array_diff($array, ...$arrays)` as a HOST builtin: PHP compares by the
    /// values' *string* representation, which for objects runs `__toString` —
    /// something the stateless registry version cannot do (PHPUnit's
    /// TestSuiteSorter diffs ExecutionOrderDependency objects). Keys preserved,
    /// first-array order, N comparison arrays.
    pub(super) fn ho_array_diff(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_diff() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let Zval::Array(first) = args[0].deref_clone() else {
            return Err(PhpError::TypeError(
                "array_diff(): Argument #1 ($array) must be of type array".to_string(),
            ));
        };
        let mut seen: Vec<Vec<u8>> = Vec::new();
        for (i, a) in args.iter().enumerate().skip(1) {
            let Zval::Array(arr) = a.deref_clone() else {
                return Err(PhpError::TypeError(format!(
                    "array_diff(): Argument #{} must be of type array",
                    i + 1
                )));
            };
            for (_, v) in arr.iter() {
                seen.push(self.vm_stringify(&v.deref_clone())?.as_bytes().to_vec());
            }
        }
        let mut out = PhpArray::new();
        for (k, v) in first.iter() {
            let s = self.vm_stringify(&v.deref_clone())?.as_bytes().to_vec();
            if !seen.contains(&s) {
                out.insert(k.clone(), v.deref_clone());
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `umask(?int $mask = null)`: return the shadow umask; with an argument,
    /// set it and return the previous one (PHP semantics). See the `Vm.umask`
    /// field note — the real process umask is never touched.
    pub(super) fn ho_umask(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prev = self.umask;
        if let Some(a) = args.first() {
            self.umask = convert::to_long_cast(&a.deref_clone(), &mut self.diags) & 0o777;
        }
        Ok(Zval::Long(prev))
    }
    /// `stream_get_meta_data($stream)`: the stream's metadata array, mirroring
    /// `php_stream_populate_meta_data` (key order included). `timed_out`/`blocked`/
    /// `unread_bytes` are fixed (no socket timeouts or read buffering here);
    /// `stream_type`/`wrapper_type`/`seekable` derive from the backend; `uri`/
    /// `mode` were recorded at open time.
    pub(super) fn ho_stream_get_meta_data(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(arg) = args.first().map(|a| a.deref_clone()) else {
            return Err(PhpError::ArgumentCountError(
                "stream_get_meta_data() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let Zval::Resource(r) = &arg else {
            return Err(PhpError::TypeError(format!(
                "stream_get_meta_data(): Argument #1 ($stream) must be of type resource, {} given",
                arg.type_name_for_error()
            )));
        };
        let mut res = r.borrow_mut();
        let Some(s) = res.as_stream_mut() else {
            return Err(PhpError::TypeError(
                "stream_get_meta_data(): supplied resource is not a valid stream resource"
                    .to_string(),
            ));
        };
        use php_types::stream::StreamBackend;
        // A gz stream (either direction) reports stream_type ZLIB. Whether the
        // wrapper keys appear depends on HOW it was opened: fopen("compress.
        // zlib://…") keeps the wrapper spec as its uri → wrapper_type ZLIB + uri;
        // gzopen(path) has a plain uri → PHP omits wrapper_type and uri entirely.
        let is_gz = s.eof_on_exhaust || matches!(s.backend, StreamBackend::GzFile { .. });
        let (stream_type, seekable) = if is_gz {
            ("ZLIB", true)
        } else {
            match &s.backend {
                StreamBackend::File(_) => ("STDIO", true),
                StreamBackend::Memory(_) => ("MEMORY", true),
                StreamBackend::Stdin | StreamBackend::Stdout | StreamBackend::Stderr => {
                    ("STDIO", false)
                }
                StreamBackend::ChildStdin(_)
                | StreamBackend::ChildStdout(_)
                | StreamBackend::ChildStderr(_) => ("STDIO", false),
                StreamBackend::Tcp(_) | StreamBackend::Udp(_) => ("tcp_socket/unknown", false),
                StreamBackend::GzFile { .. } => ("ZLIB", true),
            }
        };
        let via_zlib_wrapper = s.uri.starts_with(b"compress.zlib://");
        let wrapper = if via_zlib_wrapper {
            "ZLIB"
        } else if s.uri.starts_with(b"php://") {
            "PHP"
        } else {
            "plainfile"
        };
        let mut arr = PhpArray::new();
        arr.insert(Key::from_bytes(b"timed_out"), Zval::Bool(false));
        arr.insert(Key::from_bytes(b"blocked"), Zval::Bool(true));
        arr.insert(Key::from_bytes(b"eof"), Zval::Bool(s.eof));
        if !is_gz || via_zlib_wrapper {
            arr.insert(
                Key::from_bytes(b"wrapper_type"),
                Zval::Str(PhpStr::new(wrapper.as_bytes().to_vec())),
            );
        }
        arr.insert(
            Key::from_bytes(b"stream_type"),
            Zval::Str(PhpStr::new(stream_type.as_bytes().to_vec())),
        );
        arr.insert(Key::from_bytes(b"mode"), Zval::Str(PhpStr::new(s.mode.clone())));
        arr.insert(Key::from_bytes(b"unread_bytes"), Zval::Long(0));
        arr.insert(Key::from_bytes(b"seekable"), Zval::Bool(seekable));
        if !is_gz || via_zlib_wrapper {
            arr.insert(Key::from_bytes(b"uri"), Zval::Str(PhpStr::new(s.uri.clone())));
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `stream_set_chunk_size($stream, $size)`: record the chunk size and return
    /// the previous one (default 8192). phpr's stream I/O is unbuffered, so the
    /// size has no read/write effect — the bookkeeping exists for the return
    /// value and the argument validation (oracle-verified messages).
    pub(super) fn ho_stream_set_chunk_size(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (Some(res_arg), Some(size_arg)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(format!(
                "stream_set_chunk_size() expects exactly 2 arguments, {} given",
                args.len()
            )));
        };
        let Zval::Resource(res) = res_arg.deref_clone() else {
            return Err(PhpError::TypeError(
                "stream_set_chunk_size(): Argument #1 ($stream) must be of type resource"
                    .to_string(),
            ));
        };
        let size = convert::to_long_cast(&size_arg.deref_clone(), &mut self.diags);
        if size <= 0 {
            return Err(PhpError::ValueError(
                "stream_set_chunk_size(): Argument #2 ($size) must be greater than 0".to_string(),
            ));
        }
        if size > i32::MAX as i64 {
            return Err(PhpError::ValueError(
                "stream_set_chunk_size(): Argument #2 ($size) is too large".to_string(),
            ));
        }
        let id = res.borrow().id;
        let prev = self.stream_chunk_sizes.insert(id, size).unwrap_or(8192);
        Ok(Zval::Long(prev))
    }
    /// `tmpfile()`: create a fresh temp file opened read+write, then immediately
    /// unlink it (PHP's auto-removal). `false` on failure. Mirrors `eval::ho_tmpfile`.
    pub(super) fn ho_tmpfile(&mut self) -> Result<Zval, PhpError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CTR: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir();
        for _ in 0..100 {
            let n = CTR.fetch_add(1, Ordering::Relaxed);
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);
            let mut path = dir.clone();
            path.push(format!("phpr_tmp_{:x}_{nanos:x}_{n:x}", std::process::id()));
            match std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(f) => {
                    use std::os::unix::ffi::OsStrExt;
                    let uri = path.as_os_str().as_bytes().to_vec();
                    let _ = std::fs::remove_file(&path);
                    let stream = Stream {
                        backend: StreamBackend::File(f),
                        readable: true,
                        writable: true,
                        eof: false,
                        uri,
                        mode: b"r+b".to_vec(),
                        eof_on_exhaust: false,
                        filters: None,
                    };
                    return Ok(self.alloc_resource(stream));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Ok(Zval::Bool(false)),
            }
        }
        Ok(Zval::Bool(false))
    }
    /// `shell_exec(string $command): string|false|null` — run `$command` through
    /// `/bin/sh -c` and return its **complete** standard output. stderr is
    /// inherited (goes to the terminal, as PHP does). Returns `null` when the
    /// command produced no stdout (PHP's contract, including a failed spawn or a
    /// non-zero exit with empty output). The backtick operator lowers to this.
    pub(super) fn ho_shell_exec(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};
        let Some(cmd_arg) = args.first() else {
            return Err(PhpError::Error(
                "shell_exec() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let cmd = convert::to_zstr_cast(&cmd_arg.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        let child = Command::new("/bin/sh")
            .arg("-c")
            .arg(std::ffi::OsStr::from_bytes(&cmd))
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();
        let out = match child.and_then(|c| c.wait_with_output()) {
            Ok(o) => o.stdout,
            Err(_) => return Ok(Zval::Null),
        };
        if out.is_empty() {
            Ok(Zval::Null)
        } else {
            Ok(Zval::Str(PhpStr::new(out)))
        }
    }

    /// `filter_input(int $type, string $var_name, int $filter = FILTER_DEFAULT, array|int $options = 0)`
    /// — read `$var_name` from the request superglobal named by `$type`
    /// (GET/POST/COOKIE/SERVER/ENV) and run it through `filter_var`. An absent
    /// variable yields `null`; an unknown source id yields `false`. In CLI the
    /// GET/POST/COOKIE superglobals are empty (so those return `null`, matching
    /// the oracle); SERVER/ENV read the seeded superglobals.
    pub(super) fn ho_filter_input(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let source = args
            .first()
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(0);
        let name = args
            .get(1)
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        let sg: &[u8] = match source {
            0 => b"_POST",
            1 => b"_GET",
            2 => b"_COOKIE",
            4 => b"_ENV",
            5 => b"_SERVER",
            _ => {
                return Err(PhpError::ValueError(
                    "filter_input(): Argument #1 ($type) must be an INPUT_* constant".to_string(),
                ))
            }
        };
        let Some(idx) = crate::bytecode::superglobal_index(sg) else {
            return Ok(Zval::Bool(false));
        };
        let val = match &self.superglobals[idx as usize] {
            Zval::Array(a) => a.get(&Key::from_bytes(&name)).map(|v| v.deref_clone()),
            _ => None,
        };
        // A variable that is not present returns null (no filtering happens).
        let Some(val) = val else {
            return Ok(Zval::Null);
        };
        // Delegate to the registry `filter_var($value, $filter, $options)`.
        let filter = args.get(2).map(|v| v.deref_clone()).unwrap_or(Zval::Long(516));
        let options = args.get(3).map(|v| v.deref_clone()).unwrap_or(Zval::Long(0));
        let f = match self.registry.get(&b"filter_var"[..]) {
            Some(crate::builtin::Builtin::Value(f)) => *f,
            _ => return Ok(Zval::Bool(false)),
        };
        let top = self.frames.len() - 1;
        let line = self.cur_line(top);
        self.run_value_builtin(f, &[val, filter, options], line)
    }

    /// `array_multisort(&$array1, $order1?, $flags1?, &$array2, ...)`: sort one or
    /// more arrays at once (SQL `ORDER BY` semantics), ported from C
    /// `PHP_FUNCTION(array_multisort)`. Arguments interleave arrays with by-value
    /// order (SORT_ASC/DESC) and type (SORT_REGULAR/NUMERIC/STRING/…) flags.
    /// Returns `(true, sorted)` where `sorted[i]` is the reordered array for the
    /// `i`-th argument (None for a flag argument); the VM writes each back into
    /// its by-ref slot. Numeric keys are renumbered from 0, string keys preserved.
    ///
    /// SORT_NATURAL is compared as SORT_STRING (a documented residual); the other
    /// flags are byte-identical.
    pub(super) fn ho_array_multisort(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<(Zval, Vec<Option<Zval>>), PhpError> {
        const SORT_REGULAR: i64 = 0;
        const SORT_NUMERIC: i64 = 1;
        const SORT_STRING: i64 = 2;
        const SORT_DESC: i64 = 3;
        const SORT_ASC: i64 = 4;
        const SORT_LOCALE_STRING: i64 = 5;
        const SORT_NATURAL: i64 = 6;
        const SORT_FLAG_CASE: i64 = 8;

        struct ArrCol {
            arg_idx: usize,
            arr: Rc<PhpArray>,
            sort_type: i64,
            sort_order: i64,
        }
        let argc = args.len();
        let mut out: Vec<Option<Zval>> = vec![None; argc];
        let mut arrays: Vec<ArrCol> = Vec::new();
        let mut sort_order = SORT_ASC;
        let mut sort_type = SORT_REGULAR;
        let mut order_allowed = false;
        let mut type_allowed = false;

        for (i, arg) in args.iter().enumerate() {
            let arg = arg.deref_clone();
            // Only the first (fixed `$array`) parameter is named in errors; the
            // variadic `$rest` arguments show no parameter name.
            let al = if i == 0 { " ($array)" } else { "" };
            match &arg {
                Zval::Array(a) => {
                    if let Some(last) = arrays.last_mut() {
                        last.sort_type = sort_type;
                        last.sort_order = sort_order;
                        sort_order = SORT_ASC;
                        sort_type = SORT_REGULAR;
                    }
                    arrays.push(ArrCol {
                        arg_idx: i,
                        arr: Rc::clone(a),
                        sort_type: SORT_REGULAR,
                        sort_order: SORT_ASC,
                    });
                    order_allowed = true;
                    type_allowed = true;
                }
                Zval::Long(l) => match l & !SORT_FLAG_CASE {
                    SORT_ASC | SORT_DESC => {
                        if order_allowed {
                            sort_order = if *l == SORT_DESC { SORT_DESC } else { SORT_ASC };
                            order_allowed = false;
                        } else {
                            return Err(PhpError::TypeError(format!(
                                "array_multisort(): Argument #{}{} must be an array or a sort flag that has not already been specified",
                                i + 1, al
                            )));
                        }
                    }
                    SORT_REGULAR | SORT_NUMERIC | SORT_STRING | SORT_NATURAL
                    | SORT_LOCALE_STRING => {
                        if type_allowed {
                            sort_type = *l;
                            type_allowed = false;
                        } else {
                            return Err(PhpError::TypeError(format!(
                                "array_multisort(): Argument #{}{} must be an array or a sort flag that has not already been specified",
                                i + 1, al
                            )));
                        }
                    }
                    _ => {
                        return Err(PhpError::ValueError(format!(
                            "array_multisort(): Argument #{}{} must be a valid sort flag",
                            i + 1, al
                        )))
                    }
                },
                _ => {
                    return Err(PhpError::TypeError(format!(
                        "array_multisort(): Argument #{}{} must be an array or a sort flag",
                        i + 1, al
                    )))
                }
            }
        }
        // Finalize the last array's accumulated flags.
        if let Some(last) = arrays.last_mut() {
            last.sort_type = sort_type;
            last.sort_order = sort_order;
        } else {
            return Err(PhpError::TypeError(
                "array_multisort(): Argument #1 must be an array or a sort flag".to_string(),
            ));
        }

        // All arrays must be the same size.
        let array_size = arrays[0].arr.len();
        for a in &arrays[1..] {
            if a.arr.len() != array_size {
                return Err(PhpError::ValueError("Array sizes are inconsistent".to_string()));
            }
        }
        if array_size < 1 {
            return Ok((Zval::Bool(true), out));
        }

        // Snapshot each column's (key, value) entries in order.
        let cols: Vec<Vec<(Key, Zval)>> = arrays
            .iter()
            .map(|a| a.arr.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect())
            .collect();

        // Precompute a comparison key per element per column (diags-aware string
        // coercion happens here, before the sort closure).
        enum SortKey {
            Num(f64),
            Bytes(Vec<u8>),
            // Raw bytes compared with `strnatcmp`; case-folding is done inside
            // `natcmp`, so these are not pre-lowercased.
            Natural(Vec<u8>),
            Regular(Zval),
        }
        let mut col_keys: Vec<Vec<SortKey>> = Vec::with_capacity(arrays.len());
        for (j, a) in arrays.iter().enumerate() {
            let mut keys = Vec::with_capacity(array_size);
            for (_, v) in &cols[j] {
                let key = match a.sort_type & !SORT_FLAG_CASE {
                    SORT_NUMERIC => SortKey::Num(convert::to_double(v)),
                    SORT_NATURAL => {
                        SortKey::Natural(convert::to_zstr(v, &mut self.diags).as_bytes().to_vec())
                    }
                    SORT_STRING | SORT_LOCALE_STRING => {
                        let mut b = convert::to_zstr(v, &mut self.diags).as_bytes().to_vec();
                        if a.sort_type & SORT_FLAG_CASE != 0 {
                            b.make_ascii_lowercase();
                        }
                        SortKey::Bytes(b)
                    }
                    _ => SortKey::Regular(v.clone()),
                };
                keys.push(key);
            }
            col_keys.push(keys);
        }

        // Sort row indices by the multi-column comparison, tie-broken by the
        // original position (PHP's stable_sort_fallback).
        let mut order: Vec<usize> = (0..array_size).collect();
        order.sort_by(|&ra, &rb| {
            for (j, a) in arrays.iter().enumerate() {
                let c = match (&col_keys[j][ra], &col_keys[j][rb]) {
                    (SortKey::Num(x), SortKey::Num(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (SortKey::Bytes(x), SortKey::Bytes(y)) => x.cmp(y),
                    (SortKey::Natural(x), SortKey::Natural(y)) => {
                        php_types::ops::natcmp(x, y, a.sort_type & SORT_FLAG_CASE != 0).cmp(&0)
                    }
                    (SortKey::Regular(x), SortKey::Regular(y)) => {
                        php_types::ops::compare(x, y).cmp(&0)
                    }
                    _ => std::cmp::Ordering::Equal,
                };
                let c = if a.sort_order == SORT_DESC { c.reverse() } else { c };
                if c != std::cmp::Ordering::Equal {
                    return c;
                }
            }
            ra.cmp(&rb)
        });

        // Rebuild each array in the sorted order: numeric keys renumber from 0,
        // string keys are preserved.
        for (j, a) in arrays.iter().enumerate() {
            let mut new_arr = PhpArray::new();
            let mut n: i64 = 0;
            for &row in &order {
                let (key, val) = &cols[j][row];
                match key {
                    Key::Int(_) => {
                        new_arr.insert(Key::Int(n), val.clone());
                        n += 1;
                    }
                    Key::Str(s) => {
                        new_arr.insert(Key::Str(s.clone()), val.clone());
                    }
                }
            }
            out[a.arg_idx] = Some(Zval::Array(Rc::new(new_arr)));
        }
        Ok((Zval::Bool(true), out))
    }

    /// `grapheme_extract(string $haystack, int $size, int $type = GRAPHEME_EXTR_COUNT,
    /// int $offset = 0, &$next = null): string|false` — extract a run of complete
    /// grapheme clusters starting at byte `$offset`, bounded by `$size` measured as
    /// clusters (COUNT=0), bytes (MAXBYTES=1) or code points (MAXCHARS=2). Returns
    /// `(result, next)`; the VM writes `next` (byte offset after the run) into the
    /// by-ref `&$next`. Ports C `PHP_FUNCTION(grapheme_extract)`.
    pub(super) fn ho_grapheme_extract(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<(Zval, Zval), PhpError> {
        use unicode_segmentation::UnicodeSegmentation;
        let str_bytes = args
            .first()
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        let size = args.get(1).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let extract_type = args.get(2).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let mut lstart = args.get(3).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let str_len = str_bytes.len() as i64;
        if lstart < 0 {
            lstart += str_len;
        }
        // `$next` defaults to the (possibly adjusted) start offset; it is
        // overwritten with the run end on success. Errors below throw, so the
        // out-value is irrelevant there.
        // Validation order mirrors the C: type, then offset, then size.
        if !(0..=2).contains(&extract_type) {
            return Err(PhpError::ValueError(
                "grapheme_extract(): Argument #3 ($type) must be one of GRAPHEME_EXTR_COUNT, GRAPHEME_EXTR_MAXBYTES, or GRAPHEME_EXTR_MAXCHARS".to_string(),
            ));
        }
        if lstart > i64::from(i32::MAX) || lstart < 0 || lstart >= str_len {
            return Ok((Zval::Bool(false), Zval::Long(lstart)));
        }
        if size < 0 {
            return Err(PhpError::ValueError(
                "grapheme_extract(): Argument #2 ($size) must be greater than or equal to 0".to_string(),
            ));
        }
        if size > i64::from(i32::MAX) {
            return Err(PhpError::ValueError(
                "grapheme_extract(): Argument #2 ($size) is too large".to_string(),
            ));
        }
        if size == 0 {
            return Ok((Zval::Str(PhpStr::new(Vec::new())), Zval::Long(lstart)));
        }
        let size = size as usize;
        let mut start = lstart as usize;
        // If `$offset` lands on a UTF-8 continuation byte, advance to the next
        // character boundary (C's "move forward to the start of the next char").
        while start < str_bytes.len() && (str_bytes[start] & 0xC0) == 0x80 {
            start += 1;
        }
        if start >= str_bytes.len() {
            return Ok((Zval::Bool(false), Zval::Long(lstart)));
        }
        let rest = &str_bytes[start..];
        // ASCII fast path: if the first min(size+1, len) bytes are ASCII, every
        // unit (grapheme / byte / char) is one byte, so return min(size, len).
        let check = (size + 1).min(rest.len());
        if rest[..check].iter().all(|&b| b < 0x80) {
            let nsize = size.min(rest.len());
            return Ok((
                Zval::Str(PhpStr::new(rest[..nsize].to_vec())),
                Zval::Long((start + nsize) as i64),
            ));
        }
        // General path: segment the valid-UTF-8 prefix into grapheme clusters.
        let valid = match std::str::from_utf8(rest) {
            Ok(s) => s,
            Err(e) => std::str::from_utf8(&rest[..e.valid_up_to()]).unwrap_or(""),
        };
        let ret_pos = match extract_type {
            // COUNT: the byte end of the `size`-th grapheme (or the string end).
            0 => {
                let mut ret = 0usize;
                for (n, (bo, g)) in valid.grapheme_indices(true).enumerate() {
                    if n >= size {
                        break;
                    }
                    ret = bo + g.len();
                }
                ret
            }
            // MAXBYTES: include a grapheme only if it ends at or before `size` bytes.
            1 => {
                let mut ret = 0usize;
                for (bo, g) in valid.grapheme_indices(true) {
                    let end = bo + g.len();
                    if end > size {
                        break;
                    }
                    ret = end;
                }
                ret
            }
            // MAXCHARS: include graphemes while the cumulative code-point count ≤ size.
            _ => {
                let mut ret = 0usize;
                let mut count = 0usize;
                for (bo, g) in valid.grapheme_indices(true) {
                    count += g.chars().count();
                    if count > size {
                        break;
                    }
                    ret = bo + g.len();
                }
                ret
            }
        };
        Ok((
            Zval::Str(PhpStr::new(rest[..ret_pos].to_vec())),
            Zval::Long((start + ret_pos) as i64),
        ))
    }

    /// `similar_text(string $string1, string $string2, float &$percent = null): int`
    /// — the number of matching characters (recursive longest-common-substring,
    /// C `php_similar_char`); `&$percent` receives `2*count/(len1+len2)*100`.
    /// Returns `(count, percent)`; the VM stores `percent` into the by-ref arg.
    pub(super) fn ho_similar_text(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        let t1 = convert::to_zstr_cast(
            &args.first().map(|v| v.deref_clone()).unwrap_or(Zval::Null),
            &mut self.diags,
        )
        .as_bytes()
        .to_vec();
        let t2 = convert::to_zstr_cast(
            &args.get(1).map(|v| v.deref_clone()).unwrap_or(Zval::Null),
            &mut self.diags,
        )
        .as_bytes()
        .to_vec();
        if t1.is_empty() && t2.is_empty() {
            return Ok((Zval::Long(0), Zval::Double(0.0)));
        }
        let sim = Self::similar_char(&t1, &t2);
        let percent = sim as f64 * 200.0 / (t1.len() + t2.len()) as f64;
        Ok((Zval::Long(sim as i64), Zval::Double(percent)))
    }

    /// The longest common substring of `t1`/`t2` (C `php_similar_str`): returns
    /// `(max_len, count, pos1, pos2)`.
    fn similar_str(t1: &[u8], t2: &[u8]) -> (usize, usize, usize, usize) {
        let (mut max, mut count, mut pos1, mut pos2) = (0usize, 0usize, 0usize, 0usize);
        for p in 0..t1.len() {
            for q in 0..t2.len() {
                let mut l = 0;
                while p + l < t1.len() && q + l < t2.len() && t1[p + l] == t2[q + l] {
                    l += 1;
                }
                if l > max {
                    max = l;
                    count += 1;
                    pos1 = p;
                    pos2 = q;
                }
            }
        }
        (max, count, pos1, pos2)
    }

    /// Recursive matching-character count (C `php_similar_char`): the LCS length
    /// plus the counts of the left/right remainders around it.
    fn similar_char(t1: &[u8], t2: &[u8]) -> usize {
        let (max, count, pos1, pos2) = Self::similar_str(t1, t2);
        let mut sum = max;
        if max > 0 {
            if pos1 > 0 && pos2 > 0 && count > 1 {
                sum += Self::similar_char(&t1[..pos1], &t2[..pos2]);
            }
            if pos1 + max < t1.len() && pos2 + max < t2.len() {
                sum += Self::similar_char(&t1[pos1 + max..], &t2[pos2 + max..]);
            }
        }
        sum
    }

    /// Spawn `$command` through `/bin/sh -c`, capture its **stdout** (stderr is
    /// inherited to the terminal, as PHP's `popen(cmd, "r")` does), and return
    /// `(stdout_bytes, exit_code)`. `Err(())` means the shell could not be
    /// spawned (PHP's "Unable to fork").
    fn spawn_shell_capture(&mut self, cmd: &[u8]) -> Result<(Vec<u8>, i64), ()> {
        use std::os::unix::ffi::OsStrExt;
        use std::process::{Command, Stdio};
        let child = Command::new("/bin/sh")
            .arg("-c")
            .arg(std::ffi::OsStr::from_bytes(cmd))
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn();
        match child.and_then(|c| c.wait_with_output()) {
            Ok(out) => Ok((out.stdout, out.status.code().map(|c| c as i64).unwrap_or(-1))),
            Err(_) => Err(()),
        }
    }

    /// The last line of shell output, right-trimmed of ASCII whitespace — the
    /// value `exec`/`system` return (C `php_exec` + `strip_trailing_whitespace`).
    /// A line is a `\n`-terminated (or final) chunk; a trailing `\n` does not
    /// create an empty final line. Empty output yields `""`.
    fn shell_last_line(output: &[u8]) -> Vec<u8> {
        if output.is_empty() {
            return Vec::new();
        }
        // Find the start of the final `\n`-delimited chunk (a trailing newline
        // terminates the previous chunk rather than opening an empty one).
        let mut last_start = 0;
        for i in 0..output.len() {
            if output[i] == b'\n' && i + 1 < output.len() {
                last_start = i + 1;
            }
        }
        let mut end = output.len();
        while end > last_start && output[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        output[last_start..end].to_vec()
    }

    /// Validate the `$command` argument shared by `system`/`passthru`/`exec`:
    /// coerce to bytes, rejecting an empty command with the same `ValueError` PHP
    /// raises (`Argument #1 ($command) must not be empty`).
    fn shell_command_arg(&mut self, args: &[Zval], func: &str) -> Result<Vec<u8>, PhpError> {
        let Some(v) = args.first() else {
            return Err(PhpError::Error(format!(
                "{func}() expects at least 1 argument, 0 given"
            )));
        };
        let cmd = self.vm_stringify(&v.deref_clone())?.as_bytes().to_vec();
        if cmd.is_empty() {
            return Err(PhpError::ValueError(format!(
                "{func}(): Argument #1 ($command) must not be empty"
            )));
        }
        Ok(cmd)
    }

    /// `system(string $command, int &$result_code = null): string|false` — run
    /// `$command`, write its stdout to the current output (ob-aware, like PHP),
    /// and return the last line (right-trimmed); `$result_code` receives the exit
    /// status. A spawn failure warns and returns `false` with code `-1`. The
    /// returned pair is `(result, $result_code)`; the VM stores the code in the
    /// by-ref arg.
    pub(super) fn ho_system(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        let cmd = self.shell_command_arg(&args, "system")?;
        match self.spawn_shell_capture(&cmd) {
            Ok((stdout, code)) => {
                self.write_output(&stdout)?;
                let last = Self::shell_last_line(&stdout);
                Ok((Zval::Str(PhpStr::new(last)), Zval::Long(code)))
            }
            Err(()) => {
                self.diags.push(Diag::Warning(format!(
                    "Unable to fork [{}]",
                    String::from_utf8_lossy(&cmd)
                )));
                Ok((Zval::Bool(false), Zval::Long(-1)))
            }
        }
    }

    /// `passthru(string $command, int &$result_code = null): ?false` — run
    /// `$command` and write its raw stdout to the current output (ob-aware),
    /// returning `null`; `$result_code` receives the exit status. A spawn failure
    /// warns and returns `false` with code `-1`.
    pub(super) fn ho_passthru(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        let cmd = self.shell_command_arg(&args, "passthru")?;
        match self.spawn_shell_capture(&cmd) {
            Ok((stdout, code)) => {
                self.write_output(&stdout)?;
                Ok((Zval::Null, Zval::Long(code)))
            }
            Err(()) => {
                self.diags.push(Diag::Warning(format!(
                    "Unable to fork [{}]",
                    String::from_utf8_lossy(&cmd)
                )));
                Ok((Zval::Bool(false), Zval::Long(-1)))
            }
        }
    }

    /// `exec(string $command, array &$output = null, int &$result_code = null): string|false`
    /// — run `$command` (no output emitted), append each output line (right-trimmed)
    /// to `&$output`, and return the last line (right-trimmed); `&$result_code`
    /// receives the exit status. Returns `(result, $output array, Some($result_code))`;
    /// the VM stores the two by-ref values. A spawn failure warns and returns
    /// `false` with an empty array and code `-1`. Ports C `php_exec` type 0/2.
    pub(super) fn ho_exec(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval, Option<Zval>), PhpError> {
        let cmd = self.shell_command_arg(&args, "exec")?;
        // PHP appends to a pre-existing `$output` array rather than replacing it
        // (php_exec_ex: SEPARATE_ARRAY then add_next_index). Seed from the current
        // value when it is already an array.
        let mut lines = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Array(a)) => (*a).clone(),
            _ => PhpArray::new(),
        };
        match self.spawn_shell_capture(&cmd) {
            Ok((stdout, code)) => {
                for line in Self::shell_lines(&stdout) {
                    let _ = lines.append(Zval::Str(PhpStr::new(line)));
                }
                let last = Self::shell_last_line(&stdout);
                Ok((
                    Zval::Str(PhpStr::new(last)),
                    Zval::Array(Rc::new(lines)),
                    Some(Zval::Long(code)),
                ))
            }
            Err(()) => {
                self.diags.push(Diag::Warning(format!(
                    "Unable to fork [{}]",
                    String::from_utf8_lossy(&cmd)
                )));
                Ok((Zval::Bool(false), Zval::Array(Rc::new(lines)), Some(Zval::Long(-1))))
            }
        }
    }

    /// Split shell output into lines the way C `php_exec` does for the
    /// `&$output` array: each `\n`-terminated (or final) chunk becomes one
    /// element, right-trimmed of ASCII whitespace; a trailing `\n` does not add a
    /// trailing empty line. Empty output yields no lines.
    fn shell_lines(output: &[u8]) -> Vec<Vec<u8>> {
        let mut out = Vec::new();
        let mut start = 0;
        let mut i = 0;
        while i < output.len() {
            if output[i] == b'\n' {
                let mut end = i; // exclude the '\n'
                while end > start && output[end - 1].is_ascii_whitespace() {
                    end -= 1;
                }
                out.push(output[start..end].to_vec());
                start = i + 1;
            }
            i += 1;
        }
        if start < output.len() {
            let mut end = output.len();
            while end > start && output[end - 1].is_ascii_whitespace() {
                end -= 1;
            }
            out.push(output[start..end].to_vec());
        }
        out
    }

    /// `proc_open($command, $descriptor_spec, &$pipes, $cwd?, $env?)`: spawn a
    /// child process. A string command runs through `/bin/sh -c` (PHP's POSIX
    /// behaviour); an array is a direct argv (PHP 7.4+). Descriptors 0/1/2
    /// support `['pipe', ...]` (a pipe resource in `$pipes`) and
    /// `['file', path, mode]`; anything else inherits. Returns
    /// `(process resource | false, pipes array)` — the VM writes the pipes
    /// array into the by-ref `$pipes` argument.
    pub(super) fn ho_proc_open(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        use php_types::stream::{ProcHandle, StreamBackend};
        use std::process::Stdio;
        let empty_pipes = Zval::Array(Rc::new(PhpArray::new()));
        let cmd_arg = args.first().map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let mut command = std::process::Command::new("/bin/sh");
        let cmd_repr: Vec<u8>;
        match &cmd_arg {
            Zval::Array(a) => {
                let argv: Vec<Vec<u8>> = a
                    .iter()
                    .map(|(_, v)| {
                        convert::to_zstr_cast(&v.deref_clone(), &mut self.diags)
                            .as_bytes()
                            .to_vec()
                    })
                    .collect();
                let Some(first) = argv.first() else {
                    self.diags.push(Diag::Warning(
                        "proc_open(): Command array must have at least one element".to_string(),
                    ));
                    return Ok((Zval::Bool(false), empty_pipes));
                };
                use std::os::unix::ffi::OsStrExt;
                command = std::process::Command::new(std::ffi::OsStr::from_bytes(first));
                for a in &argv[1..] {
                    command.arg(std::ffi::OsStr::from_bytes(a));
                }
                cmd_repr = argv.join(&b' ');
            }
            other => {
                let s = convert::to_zstr_cast(other, &mut self.diags).as_bytes().to_vec();
                use std::os::unix::ffi::OsStrExt;
                command.arg("-c").arg(std::ffi::OsStr::from_bytes(&s));
                cmd_repr = s;
            }
        }
        // Descriptor spec: which of fd 0/1/2 become pipes / files / inherited.
        let spec = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Array(a)) => a,
            _ => Rc::new(PhpArray::new()),
        };
        let mut want_pipe = [false; 3];
        for fd in 0..3i64 {
            let d = spec.get(&Key::Int(fd)).map(|v| v.deref_clone());
            let stdio = match d {
                Some(Zval::Array(desc)) => {
                    let kind = desc
                        .get(&Key::Int(0))
                        .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags))
                        .map(|s| s.as_bytes().to_vec())
                        .unwrap_or_default();
                    match kind.as_slice() {
                        b"pipe" => {
                            want_pipe[fd as usize] = true;
                            Stdio::piped()
                        }
                        b"file" => {
                            let path = desc
                                .get(&Key::Int(1))
                                .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags))
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_default();
                            let mode = desc
                                .get(&Key::Int(2))
                                .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags))
                                .map(|s| s.as_bytes().to_vec())
                                .unwrap_or_else(|| b"r".to_vec());
                            use std::os::unix::ffi::OsStrExt;
                            let p = std::path::Path::new(std::ffi::OsStr::from_bytes(&path));
                            let file = match mode.first() {
                                Some(b'r') => std::fs::File::open(p),
                                Some(b'a') => {
                                    std::fs::OpenOptions::new().create(true).append(true).open(p)
                                }
                                _ => std::fs::File::create(p),
                            };
                            match file {
                                Ok(f) => Stdio::from(f),
                                Err(_) => {
                                    self.diags.push(Diag::Warning(format!(
                                        "proc_open(): Unable to open descriptor {fd} file"
                                    )));
                                    return Ok((Zval::Bool(false), empty_pipes));
                                }
                            }
                        }
                        _ => Stdio::inherit(),
                    }
                }
                _ => Stdio::inherit(),
            };
            match fd {
                0 => command.stdin(stdio),
                1 => command.stdout(stdio),
                _ => command.stderr(stdio),
            };
        }
        if let Some(Zval::Str(cwd)) = args.get(3).map(|v| v.deref_clone()) {
            use std::os::unix::ffi::OsStrExt;
            command.current_dir(std::ffi::OsStr::from_bytes(cwd.as_bytes()));
        }
        if let Some(Zval::Array(env)) = args.get(4).map(|v| v.deref_clone()) {
            // PHP: a non-null $env_vars REPLACES the environment.
            command.env_clear();
            for (k, v) in env.iter() {
                let key = match k {
                    Key::Str(s) => s.as_bytes().to_vec(),
                    Key::Int(i) => i.to_string().into_bytes(),
                };
                let val = convert::to_zstr_cast(&v.deref_clone(), &mut self.diags);
                use std::os::unix::ffi::OsStrExt;
                command.env(
                    std::ffi::OsStr::from_bytes(&key),
                    std::ffi::OsStr::from_bytes(val.as_bytes()),
                );
            }
        }
        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.diags.push(Diag::Warning(format!("proc_open(): {e}")));
                return Ok((Zval::Bool(false), empty_pipes));
            }
        };
        let mut pipes = PhpArray::new();
        let mk_pipe = |vm: &mut Self, backend: StreamBackend, readable: bool| -> Zval {
            let stream = Stream {
                backend,
                readable,
                writable: !readable,
                eof: false,
                uri: b"pipe".to_vec(),
                mode: if readable { b"r".to_vec() } else { b"w".to_vec() },
                eof_on_exhaust: false,
                filters: None,
            };
            vm.alloc_resource(stream)
        };
        if want_pipe[0] {
            if let Some(sin) = child.stdin.take() {
                pipes.insert(Key::Int(0), mk_pipe(self, StreamBackend::ChildStdin(sin), false));
            }
        }
        if want_pipe[1] {
            if let Some(sout) = child.stdout.take() {
                pipes.insert(Key::Int(1), mk_pipe(self, StreamBackend::ChildStdout(sout), true));
            }
        }
        if want_pipe[2] {
            if let Some(serr) = child.stderr.take() {
                pipes.insert(Key::Int(2), mk_pipe(self, StreamBackend::ChildStderr(serr), true));
            }
        }
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let proc = ProcHandle { child, command: cmd_repr, exit_code: None };
        let res = Zval::Resource(Rc::new(RefCell::new(Resource::new_process(id, proc))));
        Ok((res, Zval::Array(Rc::new(pipes))))
    }
    /// `__stream_select($read, $write, $except, $seconds, $microseconds)`: the
    /// poll(2) core of `stream_select` (the prelude wrapper owns the by-ref
    /// array rewrite). Returns `[count, read, write, except]` with each array
    /// filtered to the ready streams (keys preserved), or `false` on error.
    /// A memory-backed stream is always ready; a `null` $seconds blocks.
    pub(super) fn ho_stream_select(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let arr_of = |v: Option<&Zval>| -> Rc<PhpArray> {
            match v.map(|x| x.deref_clone()) {
                Some(Zval::Array(a)) => a,
                _ => Rc::new(PhpArray::new()),
            }
        };
        let read = arr_of(args.first());
        let write = arr_of(args.get(1));
        let except = arr_of(args.get(2));
        let timeout_ms: i32 = match args.get(3).map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => -1,
            Some(sec) => {
                let s = convert::to_long_cast(&sec, &mut self.diags);
                let us = args
                    .get(4)
                    .map(|v| convert::to_long_cast(v, &mut self.diags))
                    .unwrap_or(0);
                (s * 1000 + us / 1000).min(i32::MAX as i64) as i32
            }
        };
        // Collect (set, key, fd) triples; memory streams count as ready now.
        let mut fds: Vec<libc::pollfd> = Vec::new();
        let mut plan: Vec<(usize, Key, Option<usize>)> = Vec::new(); // (set, key, pollfd idx)
        let mut ready_now = 0usize;
        for (set, arr, events) in
            [(0usize, &read, libc::POLLIN), (1, &write, libc::POLLOUT), (2, &except, libc::POLLPRI)]
        {
            for (k, v) in arr.iter() {
                let fd = match v.deref_clone() {
                    Zval::Resource(r) => r.borrow_mut().as_stream_mut().and_then(|s| s.raw_fd()),
                    _ => None,
                };
                match fd {
                    Some(fd) => {
                        plan.push((set, k.clone(), Some(fds.len())));
                        fds.push(libc::pollfd { fd, events, revents: 0 });
                    }
                    None => {
                        // No descriptor (php://memory &c.): always ready.
                        plan.push((set, k.clone(), None));
                        ready_now += 1;
                    }
                }
            }
        }
        let n = if fds.is_empty() {
            0
        } else {
            // With in-process streams already ready, don't block.
            let t = if ready_now > 0 { 0 } else { timeout_ms };
            let r = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, t) };
            if r < 0 {
                return Ok(Zval::Bool(false));
            }
            r as usize
        };
        let mut outs = [PhpArray::new(), PhpArray::new(), PhpArray::new()];
        let mut count = 0i64;
        let sources = [&read, &write, &except];
        for (set, key, idx) in plan {
            let ready = match idx {
                None => true,
                Some(i) => fds[i].revents != 0,
            };
            if ready {
                if let Some(v) = sources[set].get(&key) {
                    outs[set].insert(key, v.clone());
                    count += 1;
                }
            }
        }
        let _ = n;
        let [r_out, w_out, e_out] = outs;
        let mut result = PhpArray::new();
        let _ = result.append(Zval::Long(count));
        let _ = result.append(Zval::Array(Rc::new(r_out)));
        let _ = result.append(Zval::Array(Rc::new(w_out)));
        let _ = result.append(Zval::Array(Rc::new(e_out)));
        Ok(Zval::Array(Rc::new(result)))
    }
    /// `proc_close($process)`: wait for the child and return its exit code
    /// (cached if `proc_get_status` already collected it); the resource closes.
    pub(super) fn ho_proc_close(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let r = Self::proc_arg(&args, "proc_close")?;
        let mut res = r.borrow_mut();
        let code = match res.as_process_mut() {
            Some(p) => match p.exit_code {
                Some(c) => c,
                None => match p.child.wait() {
                    Ok(status) => status.code().unwrap_or(-1),
                    Err(_) => -1,
                },
            },
            None => -1,
        };
        res.kind = php_types::stream::ResKind::Closed;
        Ok(Zval::Long(code as i64))
    }
    /// `proc_get_status($process)`: command/pid/running/signaled/stopped/
    /// exitcode/termsig/stopsig, PHP's key order. The exit code stays readable
    /// on later calls (PHP 8 caches it).
    pub(super) fn ho_proc_get_status(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let r = Self::proc_arg(&args, "proc_get_status")?;
        let mut res = r.borrow_mut();
        let Some(p) = res.as_process_mut() else {
            return Err(PhpError::TypeError(
                "proc_get_status(): supplied resource is not a valid process resource".to_string(),
            ));
        };
        let pid = p.child.id() as i64;
        let mut running = false;
        let mut termsig = 0i64;
        if p.exit_code.is_none() {
            match p.child.try_wait() {
                Ok(Some(status)) => {
                    use std::os::unix::process::ExitStatusExt;
                    termsig = status.signal().unwrap_or(0) as i64;
                    p.exit_code = Some(status.code().unwrap_or(-1));
                }
                Ok(None) => running = true,
                Err(_) => {}
            }
        }
        let exitcode = if running { -1 } else { p.exit_code.unwrap_or(-1) as i64 };
        let mut a = PhpArray::new();
        a.insert(Key::from_bytes(b"command"), Zval::Str(PhpStr::new(p.command.clone())));
        a.insert(Key::from_bytes(b"pid"), Zval::Long(pid));
        a.insert(Key::from_bytes(b"running"), Zval::Bool(running));
        a.insert(Key::from_bytes(b"signaled"), Zval::Bool(termsig != 0));
        a.insert(Key::from_bytes(b"stopped"), Zval::Bool(false));
        a.insert(Key::from_bytes(b"exitcode"), Zval::Long(exitcode));
        a.insert(Key::from_bytes(b"termsig"), Zval::Long(termsig));
        a.insert(Key::from_bytes(b"stopsig"), Zval::Long(0));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `proc_terminate($process, $signal = 15)`: deliver a signal to the child.
    pub(super) fn ho_proc_terminate(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let r = Self::proc_arg(&args, "proc_terminate")?;
        let sig = match args.get(1) {
            Some(v) => convert::to_long_cast(v, &mut self.diags) as i32,
            None => 15, // SIGTERM
        };
        let mut res = r.borrow_mut();
        let Some(p) = res.as_process_mut() else {
            return Ok(Zval::Bool(false));
        };
        let ok = unsafe { libc::kill(p.child.id() as libc::pid_t, sig) } == 0;
        Ok(Zval::Bool(ok))
    }
    /// `pcntl_signal($signo, $handler, $restart_syscalls = true)`: install a PHP
    /// handler (stored VM-side; the C catcher only marks the signal pending) or
    /// the SIG_DFL/SIG_IGN disposition (0/1, forwarded to the OS directly).
    pub(super) fn ho_pcntl_signal(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let signo = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) if n > 0 && n < 32 => n as i32,
            _ => {
                return Err(PhpError::ValueError(
                    "pcntl_signal(): Argument #1 ($signal) must be greater than or equal to 1"
                        .to_string(),
                ))
            }
        };
        let handler = args.get(1).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let restart = args
            .get(2)
            .map(|v| convert::is_true_silent(&v.deref_clone()))
            .unwrap_or(true);
        match &handler {
            // SIG_DFL / SIG_IGN restore the OS disposition.
            Zval::Long(n @ (0 | 1)) => {
                let disp = if *n == 0 { libc::SIG_DFL } else { libc::SIG_IGN };
                unsafe { libc::signal(signo, disp) };
                self.signal_handlers.insert(signo, Zval::Long(*n));
            }
            _ => {
                let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
                sa.sa_sigaction = pcntl_mark_pending as *const () as usize;
                sa.sa_flags = if restart { libc::SA_RESTART } else { 0 };
                unsafe {
                    libc::sigemptyset(&mut sa.sa_mask);
                    libc::sigaction(signo, &sa, std::ptr::null_mut());
                }
                self.signal_handlers.insert(signo, handler);
            }
        }
        Ok(Zval::Bool(true))
    }
    /// `pcntl_signal_get_handler($signo)`: the installed PHP handler, or the
    /// SIG_DFL/SIG_IGN int; SIG_DFL for a never-touched signal.
    pub(super) fn ho_pcntl_signal_get_handler(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let signo = args
            .first()
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(0) as i32;
        Ok(self
            .signal_handlers
            .get(&signo)
            .cloned()
            .unwrap_or(Zval::Long(0)))
    }
    /// `pcntl_signal_dispatch()`: deliver pending signals now.
    pub(super) fn ho_pcntl_signal_dispatch(&mut self, _args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.dispatch_pending_signals()?;
        Ok(Zval::Bool(true))
    }
    /// `pcntl_async_signals(?bool $enable = null)`: read or flip asynchronous
    /// delivery. phpr's "async" is host-builtin-boundary granularity: pending
    /// signals are dispatched when any host builtin returns (in particular the
    /// `posix_kill` that raised them), which is where the engine-visible
    /// difference lies for single-process code.
    pub(super) fn ho_pcntl_async_signals(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let old = self.async_signals;
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => {}
            Some(v) => self.async_signals = convert::is_true_silent(&v),
        }
        Ok(Zval::Bool(old))
    }
    /// `pcntl_sigprocmask($how, $signals, &$old_signals = null)`: forward to the
    /// OS mask; the previous mask reads back as the third out-param array.
    pub(super) fn ho_pcntl_sigprocmask(&mut self, args: Vec<Zval>) -> Result<(Zval, Zval), PhpError> {
        let how = match args
            .first()
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
        {
            Some(1) => libc::SIG_BLOCK,
            Some(2) => libc::SIG_UNBLOCK,
            Some(3) => libc::SIG_SETMASK,
            _ => {
                return Err(PhpError::ValueError(
                    "pcntl_sigprocmask(): Argument #1 ($mode) must be one of SIG_BLOCK, SIG_UNBLOCK, or SIG_SETMASK".to_string(),
                ))
            }
        };
        let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
        unsafe { libc::sigemptyset(&mut set) };
        if let Some(Zval::Array(a)) = args.get(1).map(|v| v.deref_clone()) {
            for (_, v) in a.iter() {
                let signo = convert::to_long_cast(&v.deref_clone(), &mut self.diags) as i32;
                if signo > 0 {
                    unsafe { libc::sigaddset(&mut set, signo) };
                }
            }
        }
        let mut old: libc::sigset_t = unsafe { std::mem::zeroed() };
        let ok = unsafe { libc::sigprocmask(how, &set, &mut old) } == 0;
        let mut old_arr = PhpArray::new();
        for signo in 1..32 {
            if unsafe { libc::sigismember(&old, signo) } == 1 {
                let _ = old_arr.append(Zval::Long(signo as i64));
            }
        }
        Ok((Zval::Bool(ok), Zval::Array(Rc::new(old_arr))))
    }
    /// `posix_kill($pid, $signo)`: raise a signal. Lives VM-side (not in
    /// php-builtins) so a self-directed signal under `pcntl_async_signals(true)`
    /// is delivered by the async check right after this builtin returns.
    pub(super) fn ho_posix_kill(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let pid = args
            .first()
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(0) as libc::pid_t;
        let signo = args
            .get(1)
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(0) as i32;
        let ok = unsafe { libc::kill(pid, signo) } == 0;
        Ok(Zval::Bool(ok))
    }
    /// `__fsockopen(target, port, timeout_secs) -> [stream|false, errno, errstr]`:
    /// the host behind the prelude `fsockopen`/`pfsockopen` wrappers (whose two
    /// by-ref outputs live in the prelude). Transports: `tcp://` (or schemeless)
    /// over `TcpStream::connect_timeout`, `udp://` over a connected `UdpSocket`;
    /// anything else is PHP's "unable to find the socket transport". A failure
    /// also raises fsockopen's Warning (monolog `@`-suppresses it and reads the
    /// errno/errstr pair instead).
    pub(super) fn ho_fsockopen(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let target = args
            .first()
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        let port_arg = args
            .get(1)
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(-1);
        let timeout = match args.get(2).map(|v| convert::to_double(&v.deref_clone())) {
            Some(t) if t > 0.0 => t,
            _ => 60.0, // ini default_socket_timeout
        };
        let text = String::from_utf8_lossy(&target).into_owned();
        match self.socket_connect(&text, port_arg, timeout) {
            Ok(res) => Ok(socket_result(res, 0, Vec::new())),
            Err((errno, errstr)) => {
                self.diags.push(Diag::Warning(format!(
                    "fsockopen(): Unable to connect to {text}:{port_arg} ({errstr})"
                )));
                Ok(socket_result(Zval::Bool(false), errno, errstr.into_bytes()))
            }
        }
    }

    /// `stream_socket_client($address, &$error_code, &$error_message, ?float $timeout, …)`
    /// — a two-out-param host builtin (like `exec`): returns
    /// `(stream|false, $error_code, Some($error_message))` for the VM to write into
    /// the by-ref args. Shares [`Self::socket_connect`] with `fsockopen`; only the
    /// Warning wording (the full address, not host:port) differs. `$flags`
    /// (STREAM_CLIENT_*) and `$context` are a scope-out.
    pub(super) fn ho_stream_socket_client(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<(Zval, Zval, Option<Zval>), PhpError> {
        let address = args
            .first()
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        // `$timeout` is arg #4 (index 3); #2/#3 are the by-ref out-params.
        let timeout = match args.get(3).map(|v| convert::to_double(&v.deref_clone())) {
            Some(t) if t > 0.0 => t,
            _ => 60.0, // ini default_socket_timeout
        };
        let text = String::from_utf8_lossy(&address).into_owned();
        let (result, errno, errstr) = match self.socket_connect(&text, -1, timeout) {
            Ok(res) => (res, 0, Vec::new()),
            Err((errno, errstr)) => {
                self.diags.push(Diag::Warning(format!(
                    "stream_socket_client(): Unable to connect to {text} ({errstr})"
                )));
                (Zval::Bool(false), errno, errstr.into_bytes())
            }
        };
        Ok((result, Zval::Long(errno), Some(Zval::Str(PhpStr::new(errstr)))))
    }

    /// Connect a `tcp://`/`udp://` socket, returning the stream-resource Zval or
    /// `(errno, errstr)` on failure (raising no Warning — each caller words its
    /// own). `explicit_port` (fsockopen's `$port`) is used only when the address
    /// carries no `:port`. Shared by `fsockopen` and `stream_socket_client`.
    fn socket_connect(&mut self, address: &str, explicit_port: i64, timeout: f64) -> Result<Zval, (i64, String)> {
        let (scheme, rest) = match address.split_once("://") {
            Some((s, r)) => (s.to_ascii_lowercase(), r.to_string()),
            None => ("tcp".to_string(), address.to_string()),
        };
        if scheme != "tcp" && scheme != "udp" {
            return Err((0, format!(
                "Unable to find the socket transport \"{scheme}\" - did you forget to enable it when you configured PHP?"
            )));
        }
        // host[:port] — an explicit :port in the address wins over the argument.
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) if !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) => {
                (h.to_string(), p.parse::<i64>().unwrap_or(-1))
            }
            _ => (rest.clone(), explicit_port),
        };
        if !(0..=65535).contains(&port) {
            return Err((0, format!("Failed to parse address \"{rest}\"")));
        }
        use std::net::ToSocketAddrs;
        let addr = match (host.as_str(), port as u16).to_socket_addrs().ok().and_then(|mut it| it.next()) {
            Some(a) => a,
            None => return Err((0, format!(
                "php_network_getaddresses: getaddrinfo for {host} failed: nodename nor servname provided, or not known"
            ))),
        };
        let backend = if scheme == "tcp" {
            match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs_f64(timeout)) {
                Ok(t) => php_types::stream::StreamBackend::Tcp(t),
                Err(e) => return Err(sock_err(&e)),
            }
        } else {
            match std::net::UdpSocket::bind("0.0.0.0:0").and_then(|s| s.connect(addr).map(|()| s)) {
                Ok(s) => php_types::stream::StreamBackend::Udp(s),
                Err(e) => return Err(sock_err(&e)),
            }
        };
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let stream = php_types::Stream {
            backend,
            readable: true,
            writable: true,
            eof: false,
            uri: address.as_bytes().to_vec(),
            mode: b"r+".to_vec(),
            eof_on_exhaust: false,
            filters: None,
        };
        Ok(Zval::Resource(Rc::new(RefCell::new(Resource::new(id, stream)))))
    }

    /// `stream_set_timeout($stream, $seconds, $microseconds = 0): bool` — succeeds
    /// only for socket streams (PHP returns `false` for files / memory streams).
    /// The read timeout is applied to the underlying socket.
    pub(super) fn ho_stream_set_timeout(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Resource(rc)) = args.first().map(|v| v.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        let secs = args.get(1).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let usecs = args.get(2).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let dur = std::time::Duration::new(secs.max(0) as u64, (usecs.max(0) as u32).saturating_mul(1000));
        let dur = (dur > std::time::Duration::ZERO).then_some(dur);
        let mut b = rc.borrow_mut();
        match b.as_stream_mut().map(|s| &s.backend) {
            Some(php_types::stream::StreamBackend::Tcp(t)) => {
                let _ = t.set_read_timeout(dur);
                Ok(Zval::Bool(true))
            }
            Some(php_types::stream::StreamBackend::Udp(u)) => {
                let _ = u.set_read_timeout(dur);
                Ok(Zval::Bool(true))
            }
            _ => Ok(Zval::Bool(false)),
        }
    }

    /// `stream_is_local($stream_or_url): bool` — true unless the wrapper is a
    /// remote (URL) one. Classifies by scheme: no scheme / local wrappers → true;
    /// http/https/ftp/ftps/data → false; an unknown scheme warns and returns true
    /// (as PHP does for a wrapper it can't find).
    pub(super) fn ho_stream_is_local(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let scheme = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Resource(rc)) => rc
                .borrow()
                .as_stream_ref()
                .map(|s| url_scheme(&s.uri))
                .unwrap_or_default(),
            Some(other) => {
                let s = convert::to_zstr_cast(&other, &mut self.diags).as_bytes().to_vec();
                url_scheme(&s)
            }
            None => return Ok(Zval::Bool(true)),
        };
        Ok(Zval::Bool(match scheme.as_str() {
            "" | "file" | "php" | "phar" | "glob" | "zip" | "compress.zlib" | "compress.bzip2" => true,
            "http" | "https" | "ftp" | "ftps" | "data" => false,
            other => {
                self.diags.push(Diag::Warning(format!(
                    "stream_is_local(): Unable to find the wrapper \"{other}\" - did you forget to enable it when you configured PHP?"
                )));
                true
            }
        }))
    }

    /// `stream_wrapper_register($protocol, $class, $flags = 0): bool` — register a
    /// userland stream wrapper. Fails (Warning + false) if the scheme is already
    /// taken (by another userland wrapper or a built-in one).
    pub(super) fn ho_stream_wrapper_register(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let proto = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_ascii_lowercase();
        let class = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        if self.stream_wrappers.contains_key(&proto) || is_builtin_scheme(&proto) {
            self.diags.push(Diag::Warning(format!(
                "stream_wrapper_register(): Protocol {}:// is already defined.",
                String::from_utf8_lossy(&proto)
            )));
            return Ok(Zval::Bool(false));
        }
        self.stream_wrappers.insert(proto, class);
        Ok(Zval::Bool(true))
    }

    /// `stream_wrapper_unregister($protocol): bool` — remove a userland wrapper.
    pub(super) fn ho_stream_wrapper_unregister(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let proto = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_ascii_lowercase();
        if self.stream_wrappers.remove(&proto).is_some() {
            Ok(Zval::Bool(true))
        } else {
            self.diags.push(Diag::Warning(format!(
                "stream_wrapper_unregister(): Can't unregister URL stream wrapper {}://",
                String::from_utf8_lossy(&proto)
            )));
            Ok(Zval::Bool(false))
        }
    }

    /// `stream_filter_append($stream, $filtername, $mode = 0, $params = null)`:
    /// attach a transform filter to the stream's read and/or write chain and
    /// return a filter-handle resource (`stream_filter_remove`'s argument).
    /// Supported filters: `zlib.deflate`/`zlib.inflate` (options `level`,
    /// `window` — raw windowBits, default −15 like PHP's filter) and
    /// `convert.base64-encode`/`-decode`.
    pub(super) fn ho_stream_filter_append(&mut self, args: Vec<Zval>, front: bool) -> Result<Zval, PhpError> {
        let fname = if front { "stream_filter_prepend" } else { "stream_filter_append" };
        let Some(Zval::Resource(rc)) = args.first().map(|v| v.deref_clone()) else {
            return Err(PhpError::TypeError(format!(
                "{fname}(): Argument #1 ($stream) must be of type resource"
            )));
        };
        let name = args
            .get(1)
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        // Filter options: `level` / `window` from an array or an object's props.
        let (mut level, mut window) = (-1i64, -15i64);
        if let Some(opts) = args.get(3).map(|v| v.deref_clone()) {
            let get = |key: &[u8], table: &PhpArray| -> Option<i64> {
                table.get(&php_types::Key::from_bytes(key)).map(|v| {
                    let mut d = Vec::new();
                    convert::to_long_cast(&v.deref_clone(), &mut d)
                })
            };
            let table = match opts {
                Zval::Array(a) => Some((*a).clone()),
                Zval::Object(o) => {
                    let mut t = PhpArray::new();
                    for (n, v) in o.borrow().props.iter() {
                        t.insert(php_types::Key::from_bytes(n), v.deref_clone());
                    }
                    Some(t)
                }
                _ => None,
            };
            if let Some(t) = table {
                if let Some(l) = get(b"level", &t) {
                    level = l;
                }
                if let Some(w) = get(b"window", &t) {
                    window = w;
                }
            }
        }
        // Direction: STREAM_FILTER_READ (1) / WRITE (2) / ALL (3); with no mode
        // given, PHP attaches wherever the stream is open (read and/or write).
        let mode = args
            .get(2)
            .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
            .unwrap_or(0);
        let (readable, writable) = {
            let b = rc.borrow();
            match b.as_stream_ref() {
                Some(s) => (s.readable, s.writable),
                None => (false, false),
            }
        };
        let want_read = if mode == 0 { readable } else { mode & 1 != 0 };
        let want_write = if mode == 0 { writable } else { mode & 2 != 0 };
        let mut filter_id = None;
        for write in [false, true] {
            if (write && !want_write) || (!write && !want_read) {
                continue;
            }
            let Some(f) = php_types::stream::StreamFilter::from_name(&name, level as i32, window as i32)
            else {
                self.diags.push(Diag::Warning(format!(
                    "{fname}(): Unable to locate filter \"{}\"",
                    String::from_utf8_lossy(&name)
                )));
                return Ok(Zval::Bool(false));
            };
            let mut b = rc.borrow_mut();
            if let Some(s) = b.as_stream_mut() {
                filter_id = Some(s.attach_filter(write, front, f));
            }
        }
        let Some(filter_id) = filter_id else {
            return Ok(Zval::Bool(false));
        };
        // Track for the shutdown flush, and mint the filter-handle resource.
        self.filtered_streams.push(Rc::clone(&rc));
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        Ok(Zval::Resource(Rc::new(RefCell::new(Resource {
            id,
            kind: php_types::stream::ResKind::Filter { stream: rc, filter_id },
        }))))
    }

    /// Finish the write-filter chains of every stream that ever had a filter
    /// attached (request-shutdown flush): the final tail goes to the backend, or
    /// into the VM's output for a stdout-backed stream.
    pub(super) fn finalize_filtered_streams(&mut self) {
        let streams = std::mem::take(&mut self.filtered_streams);
        for rc in streams {
            let (tail, to_stdout) = {
                let mut b = rc.borrow_mut();
                let Some(s) = b.as_stream_mut() else { continue };
                if !s.has_write_filters() {
                    continue;
                }
                let tail = s.drain_write_filters(true).unwrap_or_default();
                let to_stdout =
                    matches!(s.backend, php_types::stream::StreamBackend::Stdout);
                if !to_stdout && !tail.is_empty() {
                    let _ = s.write(&tail);
                }
                (tail, to_stdout)
            };
            if to_stdout && !tail.is_empty() {
                self.stdout.extend_from_slice(&tail);
                self.rendered.extend_from_slice(&tail);
            }
        }
    }

    /// `stream_resolve_include_path($filename): string|false` — the real path of
    /// `$filename` resolved against the include path. phpr's include path is the
    /// working directory, so this is `realpath` of an existing file (absolute or
    /// cwd-relative), else `false`.
    pub(super) fn ho_stream_resolve_include_path(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        let path = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        match std::fs::canonicalize(std::ffi::OsStr::from_bytes(&path)) {
            Ok(p) => Ok(Zval::Str(PhpStr::new(p.as_os_str().as_bytes().to_vec()))),
            Err(_) => Ok(Zval::Bool(false)),
        }
    }

    /// `stream_get_line($stream, $length, $ending = ""): string|false` — read from
    /// the stream up to (but not including) the first `$ending`, or `$length` bytes
    /// (when `> 0`), or EOF. `false` only at EOF with nothing read. Works on both
    /// byte streams and userland-wrapper streams.
    pub(super) fn ho_stream_get_line(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Resource(rc)) = args.first().map(|v| v.deref_clone()) else {
            return Err(PhpError::TypeError(
                "stream_get_line(): Argument #1 ($stream) must be of type resource".to_string(),
            ));
        };
        let length = args.get(1).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
        let ending = args
            .get(2)
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        let max = if length > 0 { length as usize } else { usize::MAX };
        let is_user = rc.borrow().as_user_stream().is_some();
        let mut out: Vec<u8> = Vec::new();
        let mut read_any = false;
        while out.len() < max {
            let byte = if is_user {
                self.user_stream_fill(&rc, 1, false)?;
                let mut b = rc.borrow_mut();
                let us = b.as_user_stream_mut().unwrap();
                (!us.buffer.is_empty()).then(|| us.buffer.remove(0))
            } else {
                let mut b = rc.borrow_mut();
                match b.as_stream_mut() {
                    Some(s) => s.read(1).ok().and_then(|v| v.first().copied()),
                    None => None,
                }
            };
            match byte {
                None => break,
                Some(bb) => {
                    read_any = true;
                    out.push(bb);
                    if !ending.is_empty() && out.ends_with(&ending) {
                        out.truncate(out.len() - ending.len());
                        return Ok(Zval::Str(PhpStr::new(out)));
                    }
                }
            }
        }
        if !read_any {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `gzopen(string $filename, string $mode, int $use_include_path = 0)`:
    /// open a gz file stream. Read mode decodes the whole file up front (every
    /// concatenated gzip member; a plain file reads transparently) into a
    /// `Memory`-backed stream, so fread/fgets/fseek work unchanged; write/append
    /// mode mints a `GzFile` backend whose buffer `fclose` compresses. A level
    /// digit in the mode (`"w9"`) selects the compression level.
    pub(super) fn ho_gzopen(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let path = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let mode = args
            .get(1)
            .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        self.gz_open_stream(&path, &mode, "gzopen")
    }

    /// Shared gz stream open (gzopen and the `compress.zlib://` fopen wrapper).
    /// The stream's `uri` is the plain path here; the fopen wrapper hook rewrites
    /// it to the full `compress.zlib://…` spec afterwards, which is what makes
    /// `stream_get_meta_data` report the wrapper keys for wrapper-opened streams.
    pub(super) fn gz_open_stream(&mut self, path: &[u8], mode: &[u8], fname: &str) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        // A nested `file://` inside the wrapper chain resolves to the plain path.
        let path = path.strip_prefix(b"file://".as_slice()).unwrap_or(path);
        // A php:// meta-stream cannot back a gz stream (zlib needs seekability).
        if path.starts_with(b"php://") {
            let p = String::from_utf8_lossy(path);
            self.diags.push(Diag::Warning(format!(
                "{fname}({p}): could not make seekable - {p}"
            )));
            let line = self.cur_line(self.frames.len() - 1);
            self.flush_diags(line)?;
            return Ok(Zval::Bool(false));
        }
        // Mode `c` opens the underlying file but zlib's gzopen then rejects it:
        // the file is CREATED (as PHP does) and the generic failure is raised.
        if mode.first() == Some(&b'c') {
            let _ = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(std::ffi::OsStr::from_bytes(path));
            self.diags.push(Diag::Warning(format!("{fname}(): gzopen failed")));
            let line = self.cur_line(self.frames.len() - 1);
            self.flush_diags(line)?;
            return Ok(Zval::Bool(false));
        }
        // zlib streams are strictly one-directional: any '+' is rejected up
        // front (before touching the file — a `w+` must NOT truncate).
        if mode.contains(&b'+') {
            self.diags.push(Diag::Warning(format!(
                "{fname}(): Cannot open a zlib stream for reading and writing at the same time!"
            )));
            let line = self.cur_line(self.frames.len() - 1);
            self.flush_diags(line)?;
            return Ok(Zval::Bool(false));
        }
        // Only r/w/a lead a valid gz mode (later letters/digits are flags/level).
        if !matches!(mode.first(), Some(b'r' | b'w' | b'a')) {
            self.diags.push(Diag::Warning(format!(
                "{fname}({}): Failed to open stream: `{}' is not a valid mode for fopen",
                String::from_utf8_lossy(path),
                String::from_utf8_lossy(mode)
            )));
            let line = self.cur_line(self.frames.len() - 1);
            self.flush_diags(line)?;
            return Ok(Zval::Bool(false));
        }
        // A digit anywhere in the mode string is the compression level ("w9").
        let level = mode
            .iter()
            .find(|b| b.is_ascii_digit())
            .map(|&b| (b - b'0') as i32)
            .unwrap_or(-1);
        let append = mode.first() == Some(&b'a');
        let write = append || mode.first() == Some(&b'w');
        let stream = if write {
            php_types::Stream {
                backend: php_types::stream::StreamBackend::GzFile {
                    path: path.to_vec(),
                    buf: std::io::Cursor::new(Vec::new()),
                    level,
                    append,
                },
                readable: false,
                writable: true,
                eof: false,
                uri: path.to_vec(),
                mode: mode.to_vec(),
                eof_on_exhaust: false,
                filters: None,
            }
        } else {
            // Read mode: decode the whole file now (members concatenated; plain
            // files pass through) and serve it from a memory cursor.
            let raw = match std::fs::read(std::ffi::OsStr::from_bytes(path)) {
                Ok(d) => d,
                Err(e) => {
                    let msg = match e.kind() {
                        std::io::ErrorKind::NotFound => "No such file or directory".to_string(),
                        std::io::ErrorKind::PermissionDenied => "Permission denied".to_string(),
                        _ => e.to_string(),
                    };
                    self.diags.push(Diag::Warning(format!(
                        "{fname}({}): Failed to open stream: {msg}",
                        String::from_utf8_lossy(path)
                    )));
                    let line = self.cur_line(self.frames.len() - 1);
                    self.flush_diags(line)?;
                    return Ok(Zval::Bool(false));
                }
            };
            let decoded = if raw.starts_with(&[0x1f, 0x8b]) {
                match php_types::zlibio::gzip_decode_members(&raw) {
                    Some(d) => d,
                    None => {
                        self.diags.push(Diag::Warning(format!(
                            "{fname}({}): data error",
                            String::from_utf8_lossy(path)
                        )));
                        let line = self.cur_line(self.frames.len() - 1);
                        self.flush_diags(line)?;
                        return Ok(Zval::Bool(false));
                    }
                }
            } else {
                raw // transparent read of a plain file
            };
            php_types::Stream {
                backend: php_types::stream::StreamBackend::Memory(std::io::Cursor::new(decoded)),
                readable: true,
                writable: false,
                eof: false,
                uri: path.to_vec(),
                mode: mode.to_vec(),
                // gz semantics: EOF as soon as the decoded data is consumed.
                eof_on_exhaust: true,
                filters: None,
            }
        };
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        Ok(Zval::Resource(Rc::new(RefCell::new(Resource::new(id, stream)))))
    }

    /// Open a `scheme://…` URL whose scheme is a registered userland wrapper:
    /// instantiate the handler class, call `stream_open($path,$mode,$options,&$opened)`,
    /// and (on success) mint a `UserStream` resource. `false` + Warning otherwise.
    pub(super) fn fopen_user_wrapper(&mut self, path: &[u8], mode: &[u8]) -> Result<Zval, PhpError> {
        let scheme = url_scheme(path);
        let Some(class) = self.stream_wrappers.get(scheme.as_bytes()).cloned() else {
            return Ok(Zval::Bool(false));
        };
        let key = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let cname = String::from_utf8_lossy(&class).into_owned();
        let Some(&cid) = self.class_index.get(key.as_slice()) else {
            self.diags.push(Diag::Warning(format!(
                "fopen({}): Failed to open stream: wrapper class \"{cname}\" does not exist",
                String::from_utf8_lossy(path)
            )));
            return Ok(Zval::Bool(false));
        };
        if resolve_method_runtime(&self.classes, cid, b"stream_open").is_none() {
            self.diags.push(Diag::Warning(format!(
                "fopen({}): Failed to open stream: \"{cname}::stream_open\" is not implemented",
                String::from_utf8_lossy(path)
            )));
            return Ok(Zval::Bool(false));
        }
        let obj = self.instantiate_wrapper(cid)?;
        let opened = Rc::new(RefCell::new(Zval::Null));
        let ret = self.call_method_sync(
            obj.clone(),
            b"stream_open",
            vec![
                Zval::Str(PhpStr::new(path.to_vec())),
                Zval::Str(PhpStr::new(mode.to_vec())),
                Zval::Long(0),
                Zval::Ref(Rc::clone(&opened)),
            ],
        )?;
        if !convert::to_bool(&ret.deref_clone(), &mut self.diags) {
            self.diags.push(Diag::Warning(format!(
                "fopen({}): Failed to open stream: \"{cname}::stream_open\" call failed",
                String::from_utf8_lossy(path)
            )));
            return Ok(Zval::Bool(false));
        }
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let us = php_types::stream::UserStream {
            obj,
            uri: path.to_vec(),
            mode: mode.to_vec(),
            buffer: Vec::new(),
            chunk: 8192,
        };
        Ok(Zval::Resource(Rc::new(RefCell::new(Resource::new_user_stream(id, us)))))
    }

    /// Instantiate a wrapper class the way `new` does: allocate, run the evaluated
    /// property-default thunk, then the constructor (if any) with no arguments.
    fn instantiate_wrapper(&mut self, cid: ClassId) -> Result<Zval, PhpError> {
        let obj = self.alloc_object(cid)?;
        let cc = self.classes[cid];
        if let Some(func) = cc.prop_init.as_ref() {
            let baseline = self.frames.len();
            let mut frame = Frame::new(func, self.class_mod(cid));
            frame.this = Some(obj.clone());
            frame.class = Some(cid);
            frame.static_class = Some(cid);
            frame.init_props = true;
            self.frames.push(frame);
            self.drive_to_return(baseline)?;
        }
        if resolve_method_runtime(&self.classes, cid, b"__construct").is_some() {
            self.call_method_sync(obj.clone(), b"__construct", Vec::new())?;
        }
        Ok(obj)
    }

    /// `file_get_contents` over a registered userland wrapper: open, read to EOF,
    /// close. Returns the contents, or `false` if the open failed.
    pub(super) fn user_wrapper_get_contents(&mut self, path: &[u8]) -> Result<Zval, PhpError> {
        let opened = self.fopen_user_wrapper(path, b"rb")?;
        let Zval::Resource(rc) = opened else {
            return Ok(Zval::Bool(false));
        };
        self.user_stream_fill(&rc, usize::MAX, true)?;
        let data = rc
            .borrow_mut()
            .as_user_stream_mut()
            .map(|u| std::mem::take(&mut u.buffer))
            .unwrap_or_default();
        let obj = rc.borrow().as_user_stream().map(|u| u.obj.clone());
        if let Some(obj) = obj {
            let cid = object_class_id(&obj).unwrap_or(0);
            if resolve_method_runtime(&self.classes, cid, b"stream_close").is_some() {
                self.call_method_sync(obj, b"stream_close", Vec::new())?;
            }
        }
        Ok(Zval::Str(PhpStr::new(data)))
    }

    /// Dispatch a file op (`fread`/`fwrite`/…) on a `UserStream` to the wrapper
    /// object's `stream_*` methods. Called from the `CallBuiltin` fast-path only
    /// when arg #1 is such a resource, so normal file I/O is untouched.
    pub(super) fn user_stream_op(
        &mut self,
        name: &[u8],
        rc: Rc<RefCell<Resource>>,
        args: &[Zval],
    ) -> Result<Zval, PhpError> {
        let Some(obj) = rc.borrow().as_user_stream().map(|u| u.obj.clone()) else {
            return Ok(Zval::Bool(false));
        };
        let cid = object_class_id(&obj).unwrap_or(0);
        let has = |vm: &Self, m: &[u8]| resolve_method_runtime(&vm.classes, cid, m).is_some();
        match name {
            b"fread" => {
                let n = args
                    .get(1)
                    .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
                    .unwrap_or(0)
                    .max(0) as usize;
                self.user_stream_fill(&rc, n, false)?;
                let mut b = rc.borrow_mut();
                let us = b.as_user_stream_mut().unwrap();
                let take = n.min(us.buffer.len());
                Ok(Zval::Str(PhpStr::new(us.buffer.drain(..take).collect::<Vec<u8>>())))
            }
            b"fwrite" | b"fputs" => {
                let mut data = args
                    .get(1)
                    .map(|v| convert::to_zstr_cast(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
                    .unwrap_or_default();
                if let Some(l) = args.get(2).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)) {
                    if l >= 0 && (l as usize) < data.len() {
                        data.truncate(l as usize);
                    }
                }
                let r = self.call_method_sync(obj, b"stream_write", vec![Zval::Str(PhpStr::new(data))])?;
                Ok(Zval::Long(convert::to_long_cast(&r.deref_clone(), &mut self.diags)))
            }
            b"feof" => {
                if !rc.borrow().as_user_stream().map(|u| u.buffer.is_empty()).unwrap_or(true) {
                    return Ok(Zval::Bool(false));
                }
                if has(self, b"stream_eof") {
                    let r = self.call_method_sync(obj, b"stream_eof", Vec::new())?;
                    Ok(Zval::Bool(convert::to_bool(&r.deref_clone(), &mut self.diags)))
                } else {
                    Ok(Zval::Bool(true))
                }
            }
            b"fclose" => {
                if has(self, b"stream_close") {
                    self.call_method_sync(obj, b"stream_close", Vec::new())?;
                }
                rc.borrow_mut().kind = php_types::stream::ResKind::Closed;
                Ok(Zval::Bool(true))
            }
            b"stream_get_contents" => {
                self.user_stream_fill(&rc, usize::MAX, true)?;
                let mut b = rc.borrow_mut();
                let us = b.as_user_stream_mut().unwrap();
                Ok(Zval::Str(PhpStr::new(std::mem::take(&mut us.buffer))))
            }
            b"fgets" => {
                // Read up to a newline (inclusive), or the whole remaining stream.
                let cap = args
                    .get(1)
                    .map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags))
                    .filter(|&l| l > 0)
                    .map(|l| (l - 1) as usize);
                loop {
                    let (has_nl, len, want_more) = {
                        let b = rc.borrow();
                        let us = b.as_user_stream().unwrap();
                        let nl = us.buffer.iter().position(|&c| c == b'\n');
                        let reached = cap.map(|c| us.buffer.len() >= c).unwrap_or(false);
                        (nl.is_some(), us.buffer.len(), !nl.is_some() && !reached)
                    };
                    let _ = len;
                    if !want_more {
                        break;
                    }
                    let before = rc.borrow().as_user_stream().unwrap().buffer.len();
                    self.user_stream_fill(&rc, before + 1, false)?;
                    if rc.borrow().as_user_stream().unwrap().buffer.len() == before {
                        break; // no more data
                    }
                    let _ = has_nl;
                }
                let mut b = rc.borrow_mut();
                let us = b.as_user_stream_mut().unwrap();
                if us.buffer.is_empty() {
                    return Ok(Zval::Bool(false));
                }
                let mut end = us.buffer.iter().position(|&c| c == b'\n').map(|i| i + 1).unwrap_or(us.buffer.len());
                if let Some(c) = cap {
                    end = end.min(c);
                }
                Ok(Zval::Str(PhpStr::new(us.buffer.drain(..end).collect::<Vec<u8>>())))
            }
            b"rewind" => {
                let ok = if has(self, b"stream_seek") {
                    let r = self.call_method_sync(obj, b"stream_seek", vec![Zval::Long(0), Zval::Long(0)])?;
                    convert::to_bool(&r.deref_clone(), &mut self.diags)
                } else {
                    false
                };
                if ok {
                    if let Some(us) = rc.borrow_mut().as_user_stream_mut() {
                        us.buffer.clear();
                    }
                }
                Ok(Zval::Bool(ok))
            }
            b"fseek" => {
                let off = args.get(1).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
                let whence = args.get(2).map(|v| convert::to_long_cast(&v.deref_clone(), &mut self.diags)).unwrap_or(0);
                let ok = if has(self, b"stream_seek") {
                    let r = self.call_method_sync(obj, b"stream_seek", vec![Zval::Long(off), Zval::Long(whence)])?;
                    convert::to_bool(&r.deref_clone(), &mut self.diags)
                } else {
                    false
                };
                if ok {
                    if let Some(us) = rc.borrow_mut().as_user_stream_mut() {
                        us.buffer.clear();
                    }
                }
                Ok(Zval::Long(if ok { 0 } else { -1 }))
            }
            b"ftell" => {
                if has(self, b"stream_tell") {
                    let r = self.call_method_sync(obj, b"stream_tell", Vec::new())?;
                    Ok(Zval::Long(convert::to_long_cast(&r.deref_clone(), &mut self.diags)))
                } else {
                    Ok(Zval::Bool(false))
                }
            }
            _ => Ok(Zval::Bool(false)),
        }
    }

    /// Fill a `UserStream`'s read buffer by repeated `stream_read($chunk)` (each
    /// followed by the `stream_eof()` PHP consults). Two modes match PHP's stream
    /// layer: a bounded read (`to_eof == false`) stops once the buffer reaches
    /// `want` **or** a short read is seen; a read-to-EOF (`to_eof == true`, for
    /// `stream_get_contents`/`file_get_contents`) reads until an empty read.
    fn user_stream_fill(
        &mut self,
        rc: &Rc<RefCell<Resource>>,
        want: usize,
        to_eof: bool,
    ) -> Result<(), PhpError> {
        loop {
            let (have, chunk, obj) = {
                let b = rc.borrow();
                let Some(us) = b.as_user_stream() else { return Ok(()) };
                (us.buffer.len(), us.chunk, us.obj.clone())
            };
            if !to_eof && have >= want {
                break;
            }
            let r = self.call_method_sync(obj.clone(), b"stream_read", vec![Zval::Long(chunk as i64)])?;
            let bytes = convert::to_zstr_cast(&r.deref_clone(), &mut self.diags).as_bytes().to_vec();
            let got = bytes.len();
            let cid = object_class_id(&obj).unwrap_or(0);
            if resolve_method_runtime(&self.classes, cid, b"stream_eof").is_some() {
                self.call_method_sync(obj.clone(), b"stream_eof", Vec::new())?;
            }
            if let Some(us) = rc.borrow_mut().as_user_stream_mut() {
                us.buffer.extend_from_slice(&bytes);
            }
            if got == 0 || (!to_eof && got < chunk) {
                break; // empty read ends any fill; a short read ends a bounded one
            }
        }
        Ok(())
    }
    /// `opendir($directory)`: snapshot the directory entries (`.`/`..` first, then
    /// OS order) into a `DirHandle` resource; `false` + Warning on failure. Mirrors
    /// `eval::ho_opendir`.
    pub(super) fn ho_opendir(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        let Some(path_arg) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "opendir() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let path = convert::to_zstr_cast(&path_arg.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        // The ZLIB wrapper has no directory operations (PHP's exact warning).
        if path.starts_with(b"compress.zlib://") {
            self.diags.push(Diag::Warning(format!(
                "opendir({}): Failed to open directory: not implemented",
                String::from_utf8_lossy(&path)
            )));
            return Ok(Zval::Bool(false));
        }
        match std::fs::read_dir(std::ffi::OsStr::from_bytes(&path)) {
            Ok(rd) => {
                let mut entries = vec![b".".to_vec(), b"..".to_vec()];
                for e in rd.flatten() {
                    entries.push(e.file_name().as_os_str().as_bytes().to_vec());
                }
                let id = self.next_resource_id;
                self.next_resource_id += 1;
                Ok(Zval::Resource(Rc::new(RefCell::new(Resource {
                    id,
                    kind: ResKind::Dir(DirHandle { entries, pos: 0 }),
                }))))
            }
            Err(e) => {
                let msg = e.to_string();
                let msg = msg.split(" (os error").next().unwrap_or(&msg);
                self.diags.push(Diag::Warning(format!(
                    "opendir({}): Failed to open directory: {msg}",
                    String::from_utf8_lossy(&path)
                )));
                Ok(Zval::Bool(false))
            }
        }
    }
    /// `func_num_args()` (Session D1): the number of arguments passed to the current
    /// function. A fatal `Error` at global scope, matching PHP 8.5.
    pub(super) fn ho_func_num_args(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_num_args() must be called from a function context".to_string(),
            ));
        }
        Ok(Zval::Long(self.frames[top].argc as i64))
    }
    /// `func_get_args()` (Session D1): the current function's arguments as a 0-indexed
    /// array. A fatal `Error` at global scope.
    pub(super) fn ho_func_get_args(&mut self) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_get_args() must be called from a function context".to_string(),
            ));
        }
        let mut arr = PhpArray::new();
        for v in self.current_frame_args(top) {
            let _ = arr.append(v);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `func_get_arg($position)` (Session D1): the argument at `position`. A fatal
    /// `Error` at global scope; a `ValueError` if `position` is out of range.
    pub(super) fn ho_func_get_arg(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        if top == 0 {
            return Err(PhpError::Error(
                "func_get_arg() must be called from a function context".to_string(),
            ));
        }
        let Some(a0) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "func_get_arg() expects exactly 1 argument, 0 given".to_string(),
            ));
        };
        let pos = convert::to_long_cast(&a0.deref_clone(), &mut self.diags);
        let argc = self.frames[top].argc as i64;
        if pos < 0 || pos >= argc {
            return Err(PhpError::ValueError(
                "func_get_arg(): Argument #1 ($position) must be less than the number of the arguments passed to the currently executed function".to_string(),
            ));
        }
        let all = self.current_frame_args(top);
        Ok(all[pos as usize].clone())
    }
    /// The `sprintf`/`printf` family (Session D2): resolve object arguments to their
    /// `__toString` form (recursively through arrays) *before* handing them to the
    /// pure registry format engine, so `%s` on an object honours `__toString`.
    /// Mirrors `eval::ho_format`; the engine writes to stdout for the `printf`
    /// variants, so the call goes through [`Self::run_value_builtin`] for the
    /// faithful rendered-stream interleaving.
    pub(super) fn ho_format(&mut self, name: &[u8], args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut argv = Vec::with_capacity(args.len());
        for a in args {
            argv.push(self.format_resolve_objects(a)?);
        }
        let f = match self.registry.get(name) {
            Some(Builtin::Value(f)) => *f,
            _ => return Err(undefined_builtin(name)),
        };
        let top = self.frames.len() - 1;
        let line = self.cur_line(top);
        self.run_value_builtin(f, &argv, line)
    }
    /// `implode($separator, $array)` / `implode($array)` as a *host* builtin:
    /// elements convert via [`Self::vm_stringify`], so a `Stringable` object
    /// element runs its `__toString` (doctrine's nested CompositeExpression) —
    /// the registry implementation can only fatal on objects. Signature errors
    /// mirror `php-builtins`' implode exactly.
    pub(super) fn ho_implode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let args: Vec<Zval> = args.iter().map(|a| a.deref_clone()).collect();
        let (glue, arr) = match args.as_slice() {
            [Zval::Array(a)] => (Vec::new(), Rc::clone(a)),
            [only] => {
                return Err(PhpError::TypeError(format!(
                    "implode(): Argument #1 ($array) must be of type array, {} given",
                    only.type_name_for_error()
                )))
            }
            [sep, rest @ ..] => {
                if let Zval::Array(_) = sep {
                    return Err(PhpError::TypeError(
                        "implode(): Argument #1 ($separator) must be of type string, array given"
                            .to_string(),
                    ));
                }
                match rest.first() {
                    Some(Zval::Array(a)) => {
                        // The glue is string-coerced like the elements, so a
                        // Stringable separator runs its `__toString` (vm_stringify),
                        // not the object-fatal `convert::to_zstr` funnel.
                        (self.vm_stringify(sep)?.as_bytes().to_vec(), Rc::clone(a))
                    }
                    Some(other) => {
                        return Err(PhpError::TypeError(format!(
                            "implode(): Argument #2 ($array) must be of type array, {} given",
                            other.type_name_for_error()
                        )))
                    }
                    None => unreachable!("rest has at least one element"),
                }
            }
            [] => {
                return Err(PhpError::Error(
                    "implode() expects at least 1 argument, 0 given".to_string(),
                ))
            }
        };
        let mut out = Vec::new();
        for (i, (_, v)) in arr.iter().enumerate() {
            if i > 0 {
                out.extend_from_slice(&glue);
            }
            let s = self.vm_stringify(&v.deref_clone())?;
            out.extend_from_slice(s.as_bytes());
        }
        Ok(Zval::Str(PhpStr::new(out)))
    }
    /// The array internal-pointer family (`reset`/`end`/`next`/`prev`/`current`/
    /// `key`): operate on the array in `slot` (following a reference), mutating or
    /// reading its cursor. `current`/`prev`/`next`/`reset`/`end` return the value at
    /// the pointer (or `false`); `key` returns the key (or `null`). A non-array
    /// argument is a `TypeError`. Pure VM gain — the tree-walker has no equivalent.
    pub(super) fn ho_array_pointer(&mut self, slot: Slot, op: PtrOp) -> Result<Zval, PhpError> {
        let top = self.frames.len() - 1;
        match &mut self.frames[top].slots[slot as usize] {
            Zval::Ref(rc) => {
                let mut inner = rc.borrow_mut();
                array_pointer_apply(&mut inner, op)
            }
            other => array_pointer_apply(other, op),
        }
    }
    /// `array_walk(&$array, $callback, $arg = null)` (Session C): apply `$callback`
    /// to each element as `($value, $key[, $arg])`. When the callback's first
    /// parameter is by-reference the element is passed through a shared cell and
    /// the mutation is written back; otherwise it is read-only. Keys are never
    /// modified. Returns true. Mirrors `eval::ho_array_walk`.
    pub(super) fn ho_array_walk(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = rest.into_iter();
        let Some(callback) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "array_walk() expects at least 2 arguments, 1 given".to_string(),
            ));
        };
        let callback = callback.deref_clone();
        let extra = it.next().map(|e| e.deref_clone());
        let by_ref = self.callable_first_by_ref(&callback);
        let top = self.frames.len() - 1;
        let entries: Vec<(Key, Zval)> = match self.frames[top].slots[slot as usize].deref_clone() {
            Zval::Array(a) => a.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect(),
            other if deref_object(&other).is_some() => {
                // Walking an OBJECT visits its property table; a lazy one
                // initializes first (PHP 8.4). A by-ref callback binds each
                // property's storage cell — a typed property's cell keeps
                // enforcing its type (lazy_objects/array_walk).
                let v = if self.is_lazy_value(&other) { self.realize_full(&other)? } else { other };
                let o = deref_object(&v).expect("object checked above");
                let cid = o.borrow().class_id as usize;
                let keys: Vec<Box<[u8]>> = o
                    .borrow()
                    .props
                    .iter()
                    .filter(|(_, val)| !matches!(val, Zval::Undef))
                    .map(|(k, _)| k.to_vec().into_boxed_slice())
                    .collect();
                for key in keys {
                    let display = php_types::prop_display_name(&key).to_vec();
                    let key_z = Zval::Str(PhpStr::new(display.clone()));
                    if by_ref {
                        let cell = prop_ref_cell(&o, &key);
                        if let Some((decl, hint)) = prop_type_decl(&self.classes, cid, &display) {
                            self.register_typed_ref(&cell, &o, decl, &display, hint);
                        }
                        let mut argv = vec![Zval::Ref(cell), key_z];
                        if let Some(e) = &extra {
                            argv.push(e.clone());
                        }
                        self.call_callable(callback.clone(), argv)?;
                    } else {
                        let val = o.borrow().props.get(&key).map(|v| v.deref_clone()).unwrap_or(Zval::Null);
                        let mut argv = vec![val, key_z];
                        if let Some(e) = &extra {
                            argv.push(e.clone());
                        }
                        self.call_callable(callback.clone(), argv)?;
                    }
                }
                return Ok(Zval::Bool(true));
            }
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_walk(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };

        let mut out = PhpArray::new();
        for (k, v) in entries {
            let key_z = key_to_zval(&k);
            let new_v = if by_ref {
                let vcell = Rc::new(RefCell::new(v));
                let mut argv = vec![Zval::Ref(Rc::clone(&vcell)), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_callable(callback.clone(), argv)?;
                // Bind before the block ends so the `Ref` temporary is dropped
                // before `vcell`, satisfying the borrow checker.
                let updated = vcell.borrow().clone();
                updated
            } else {
                let mut argv = vec![v.clone(), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_callable(callback.clone(), argv)?;
                v
            };
            out.insert(k, new_v);
        }
        let top = self.frames.len() - 1;
        // Write through a reference slot (by-ref param, or a place ref from the
        // compiler's non-variable first-arg path) so the caller/place sees the
        // result; a plain slot is overwritten as before.
        store_slot(&mut self.frames[top].slots[slot as usize], Zval::Array(Rc::new(out)));
        Ok(Zval::Bool(true))
    }
    /// `array_walk_recursive(&$array, $callback, $arg = null)`: like `array_walk`
    /// but descends into nested arrays, invoking `$callback` only on the leaf
    /// (non-array) values. The structure is preserved; a by-ref first parameter
    /// lets the callback mutate leaves in place. Returns true.
    pub(super) fn ho_array_walk_recursive(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut it = rest.into_iter();
        let Some(callback) = it.next() else {
            return Err(PhpError::ArgumentCountError(
                "array_walk_recursive() expects at least 2 arguments, 1 given".to_string(),
            ));
        };
        let callback = callback.deref_clone();
        let extra = it.next().map(|e| e.deref_clone());
        let by_ref = self.callable_first_by_ref(&callback);
        let top = self.frames.len() - 1;
        let arr = match self.frames[top].slots[slot as usize].deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_walk_recursive(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        let walked = self.walk_recursive(&arr, &callback, &extra, by_ref)?;
        let top = self.frames.len() - 1;
        store_slot(&mut self.frames[top].slots[slot as usize], Zval::Array(Rc::new(walked)));
        Ok(Zval::Bool(true))
    }
    /// `usort(&$array, $callback)` (Session C): sort the array's values in place by
    /// the comparator, re-index `0..n`, and return `true`. The comparator returns
    /// an int (`$a <=> $b`-style). Mirrors `eval::ho_usort` — a stable merge sort,
    /// matching PHP 8's sort guarantee. Reads the array out of `slot` up front and
    /// writes the sorted result back, so no slot borrow is held across a callback.
    pub(super) fn ho_usort(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cmp) = rest.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "usort() expects exactly 2 arguments, 1 given".to_string(),
            ));
        };
        let cmp = cmp.deref_clone();
        let top = self.frames.len() - 1;
        let values: Vec<Zval> = match self.frames[top].slots[slot as usize].deref_clone() {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "usort(): Argument #1 ($array) must be of type array, {} given",
                    other.type_name_for_error()
                )))
            }
        };
        let sorted = self.vm_merge_sort_with(&cmp, values)?;
        let mut out = PhpArray::new();
        for v in sorted {
            let _ = out.append(v);
        }
        let top = self.frames.len() - 1;
        // Write through a reference slot (by-ref param, or a place ref from the
        // compiler's non-variable first-arg path) so the caller/place sees the
        // result; a plain slot is overwritten as before.
        store_slot(&mut self.frames[top].slots[slot as usize], Zval::Array(Rc::new(out)));
        Ok(Zval::Bool(true))
    }
    /// `uasort(&$array, $callback)`: sort by the value comparator like `usort`, but
    /// **preserve** each element's key/value association (no re-indexing). Returns
    /// `true`. Mirrors `usort`'s slot handling and stable merge sort.
    pub(super) fn ho_uasort(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cmp) = rest.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "uasort() expects exactly 2 arguments, 1 given".to_string(),
            ));
        };
        let cmp = cmp.deref_clone();
        let top = self.frames.len() - 1;
        // Each item carries (compare-value, key, value); `uasort` compares the value.
        let pairs: Vec<(Zval, Key, Zval)> =
            match self.frames[top].slots[slot as usize].deref_clone() {
                Zval::Array(a) => a
                    .iter()
                    .map(|(k, v)| {
                        let val = v.deref_clone();
                        (val.clone(), k.clone(), val)
                    })
                    .collect(),
                other => {
                    return Err(PhpError::TypeError(format!(
                        "uasort(): Argument #1 ($array) must be of type array, {} given",
                        other.type_name_for_error()
                    )))
                }
            };
        let sorted = self.vm_merge_sort_pairs(&cmp, pairs)?;
        let mut out = PhpArray::new();
        for (_, k, v) in sorted {
            out.insert(k, v);
        }
        let top = self.frames.len() - 1;
        // Write through a reference slot (by-ref param, or a place ref from the
        // compiler's non-variable first-arg path) so the caller/place sees the
        // result; a plain slot is overwritten as before.
        store_slot(&mut self.frames[top].slots[slot as usize], Zval::Array(Rc::new(out)));
        Ok(Zval::Bool(true))
    }
    /// `uksort(&$array, $callback)`: sort by the **key** comparator, preserving each
    /// key/value association. The comparator receives the keys (int keys as int,
    /// string keys as string). Returns `true`.
    pub(super) fn ho_uksort(&mut self, slot: Slot, rest: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cmp) = rest.into_iter().next() else {
            return Err(PhpError::ArgumentCountError(
                "uksort() expects exactly 2 arguments, 1 given".to_string(),
            ));
        };
        let cmp = cmp.deref_clone();
        let top = self.frames.len() - 1;
        // Compare the key (materialised as a Zval), keep key and value together.
        let pairs: Vec<(Zval, Key, Zval)> =
            match self.frames[top].slots[slot as usize].deref_clone() {
                Zval::Array(a) => a
                    .iter()
                    .map(|(k, v)| (key_to_zval(k), k.clone(), v.deref_clone()))
                    .collect(),
                other => {
                    return Err(PhpError::TypeError(format!(
                        "uksort(): Argument #1 ($array) must be of type array, {} given",
                        other.type_name_for_error()
                    )))
                }
            };
        let sorted = self.vm_merge_sort_pairs(&cmp, pairs)?;
        let mut out = PhpArray::new();
        for (_, k, v) in sorted {
            out.insert(k, v);
        }
        let top = self.frames.len() - 1;
        // Write through a reference slot (by-ref param, or a place ref from the
        // compiler's non-variable first-arg path) so the caller/place sees the
        // result; a plain slot is overwritten as before.
        store_slot(&mut self.frames[top].slots[slot as usize], Zval::Array(Rc::new(out)));
        Ok(Zval::Bool(true))
    }
    /// `is_callable($value)`: a closure / FCC, a string naming a function or
    /// `Class::method`, a `[target, method]` array, or an object with `__invoke`
    /// (mirrors `eval::ho_is_callable`; does not invoke the callable).
    pub(super) fn ho_is_callable(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(v) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "is_callable() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        Ok(Zval::Bool(self.is_value_callable(&v.deref_clone())))
    }
    /// `is_iterable($v)`: an array, a generator, or an object implementing
    /// `Traversable` (i.e. `Iterator` or `IteratorAggregate`).
    pub(super) fn ho_is_iterable(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let v = args.first().map_or(Zval::Null, |v| v.deref_clone());
        let result = match v {
            Zval::Array(_) | Zval::Generator(_) => true,
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                self.is_traversable(cid)
                    || self.iteratoraggregate_id.is_some_and(|i| {
                        is_instance_of(&self.classes, self.stringable_id, cid, i)
                    })
            }
            _ => false,
        };
        Ok(Zval::Bool(result))
    }
    /// `spl_autoload_register(?callable $callback = null, bool $throw = true,
    /// bool $prepend = false)` (step 57, Phase 3): register `$callback` as an
    /// autoloader (prepended when `$prepend`). With no callback PHP registers its
    /// default file-based loader, which we don't model — a no-op that still
    /// succeeds. The break-on-found in `try_autoload` makes a duplicate harmless,
    /// so registration is not deduplicated. Always returns `true`.
    pub(super) fn ho_spl_autoload_register(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match args.first() {
            None | Some(Zval::Null) => Ok(Zval::Bool(true)),
            Some(cb) => {
                let prepend = args.get(2).is_some_and(|v| convert::to_bool(v, &mut self.diags));
                if prepend {
                    self.autoloaders.insert(0, cb.clone());
                } else {
                    self.autoloaders.push(cb.clone());
                }
                Ok(Zval::Bool(true))
            }
        }
    }
    /// `spl_autoload_unregister(callable $callback)` (step 57, Phase 3): remove a
    /// previously registered autoloader. Matches string and closure callables;
    /// returns whether one was removed.
    pub(super) fn ho_spl_autoload_unregister(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cb) = args.first() else { return Ok(Zval::Bool(false)) };
        let before = self.autoloaders.len();
        self.autoloaders.retain(|a| !callable_eq(a, cb));
        Ok(Zval::Bool(self.autoloaders.len() != before))
    }
    /// `spl_autoload_functions()` (step 57, Phase 3): the registered autoloaders as
    /// an array, in call order.
    pub(super) fn ho_spl_autoload_functions(&mut self) -> Result<Zval, PhpError> {
        let mut arr = PhpArray::new();
        for a in &self.autoloaders {
            let _ = arr.append(a.clone());
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `spl_autoload_call(string $class)` (step 57, Phase 3): run the registered
    /// autoloaders for `$class` (no-op if it is already defined).
    pub(super) fn ho_spl_autoload_call(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if let Some(a) = args.first() {
            let raw = convert::to_zstr_cast(&a.deref_clone(), &mut self.diags);
            let b = raw.as_bytes();
            let name = b.strip_prefix(b"\\").unwrap_or(b).to_vec();
            let _ = self.resolve_class_autoload(&name)?;
        }
        Ok(Zval::Null)
    }
}
