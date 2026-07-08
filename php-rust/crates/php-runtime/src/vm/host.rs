//! Host builtins (the ~200 ho_* methods backing PHP standard-library
//! functions that need VM state). Split from vm/mod.rs; the host_builtins!
//! dispatch macro stays in mod.rs. Structural move only.

use super::*;

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
        // `$depth` must be a positive nesting limit; a non-positive one is a
        // ValueError before any parsing (json_decode_error). phpr does not enforce
        // the limit itself, but it must reject an invalid argument.
        if let Some(v) = args.get(2) {
            let depth = convert::to_long_cast(v, &mut self.diags);
            if depth <= 0 {
                return Err(PhpError::ValueError(
                    "json_decode(): Argument #3 ($depth) must be greater than 0".to_string(),
                ));
            }
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
        match crate::json::parse(&json) {
            Some(j) => {
                self.json_last_error = 0; // JSON_ERROR_NONE
                self.vm_json_to_zval(&j, assoc)
            }
            None => {
                self.json_last_error = 4; // JSON_ERROR_SYNTAX
                if flags & 4_194_304 != 0 {
                    // JSON_THROW_ON_ERROR
                    if let Some(cid) = self.class_index.get(&b"jsonexception"[..]).copied() {
                        let obj = self.synthesize_throwable(cid, "Syntax error")?;
                        return Err(PhpError::Thrown(obj));
                    }
                }
                Ok(Zval::Null)
            }
        }
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
    /// `__reflect_class_constants($class, $filter = null)`: every class constant
    /// visible on `$class` as a `name => value` array — backs
    /// `ReflectionClass::getConstants`. `$filter` is a visibility bitmask
    /// (IS_PUBLIC=1 / IS_PROTECTED=2 / IS_PRIVATE=4); `null` returns all. Values
    /// come from running each declaring class's value thunk. (Enum cases are not
    /// yet modelled.)
    pub(super) fn ho_reflect_class_constants(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let mut arr = php_types::PhpArray::new();
        let Some(&start) = self.class_index.get(&key) else {
            return Ok(Zval::Array(Rc::new(arr)));
        };
        let filter: Option<i64> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => Some(n),
            _ => None,
        };
        for (name, decl, idx) in self.collect_class_consts(start) {
            if let Some(bits) = filter {
                let vbit = match self.classes[decl].consts[idx].visibility {
                    crate::hir::Visibility::Public => 1,
                    crate::hir::Visibility::Protected => 2,
                    crate::hir::Visibility::Private => 4,
                };
                if bits & vbit == 0 {
                    continue;
                }
            }
            let thunk: &'m Func = &self.classes[decl].consts[idx].func;
            let v = self.run_value_thunk(thunk, Some(decl))?;
            arr.insert(Key::Str(PhpStr::new(name)), v);
        }
        // Enum cases are reported as (public) constants, value = the case singleton.
        if filter.map_or(true, |bits| bits & 1 != 0) {
            let n = self.classes[start].enum_cases.len();
            for i in 0..n {
                let name = self.classes[start].enum_cases[i].name.to_vec();
                let inst = Zval::Object(self.enum_case(start, i as u32));
                arr.insert(Key::Str(PhpStr::new(name)), inst);
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_class_const_names($class)`: the names of every class constant
    /// visible on `$class`, most-derived first — backs
    /// `ReflectionClass::getReflectionConstants` (which wraps each in a
    /// `ReflectionClassConstant`).
    pub(super) fn ho_reflect_class_const_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let mut arr = php_types::PhpArray::new();
        if let Some(&start) = self.class_index.get(&key) {
            for (name, _, _) in self.collect_class_consts(start) {
                let _ = arr.append(Zval::Str(PhpStr::new(name)));
            }
            // Enum cases are reported as constants too, after the real ones.
            for c in &self.classes[start].enum_cases {
                let _ = arr.append(Zval::Str(PhpStr::new(c.name.to_vec())));
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_class_const_info($class, $name)`: descriptor array for one class
    /// constant (`value`, `declaringClass`, `visibility`, `final`, `enumCase`), or
    /// `false` if undeclared — backs the `ReflectionClassConstant` accessors. The
    /// value is produced by the declaring class's thunk.
    pub(super) fn ho_reflect_class_const_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let Some((decl, idx)) = find_const_runtime(&self.classes, cid, &name) else {
            // An enum case is reachable as a class constant: its value is the case
            // singleton, it is implicitly public and is flagged `enumCase`.
            if let Some(ci) = self.enum_case_idx(cid, &name) {
                let value = Zval::Object(self.enum_case(cid, ci as u32));
                let decl_name = self.classes[cid].name.to_vec();
                let mut a = php_types::PhpArray::new();
                a.insert(Key::Str(PhpStr::new(b"value".to_vec())), value);
                a.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
                a.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(b"public".to_vec())));
                a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(false));
                a.insert(Key::Str(PhpStr::new(b"enumCase".to_vec())), Zval::Bool(true));
                return Ok(Zval::Array(Rc::new(a)));
            }
            return Ok(Zval::Bool(false));
        };
        let vis: &[u8] = match self.classes[decl].consts[idx].visibility {
            crate::hir::Visibility::Public => b"public",
            crate::hir::Visibility::Protected => b"protected",
            crate::hir::Visibility::Private => b"private",
        };
        let is_final = self.classes[decl].consts[idx].is_final;
        let decl_name = self.classes[decl].name.to_vec();
        let thunk: &'m Func = &self.classes[decl].consts[idx].func;
        let value = self.run_value_thunk(thunk, Some(decl))?;
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"value".to_vec())), value);
        a.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
        a.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(vis.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        a.insert(Key::Str(PhpStr::new(b"enumCase".to_vec())), Zval::Bool(false));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_enum_backing($class)`: the backing scalar type of a backed enum
    /// as a `ReflectionNamedType` descriptor (`int`/`string`), or `false` for a
    /// pure enum. Derived from the (folded) case values.
    pub(super) fn ho_reflect_enum_backing(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let name: Option<&[u8]> = self.classes[cid].enum_cases.iter().find_map(|c| match &c.value {
            Some(crate::bytecode::Const::Int(_)) => Some(&b"int"[..]),
            Some(crate::bytecode::Const::Str(_)) => Some(&b"string"[..]),
            _ => None,
        });
        let Some(name) = name else { return Ok(Zval::Bool(false)) };
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"name".to_vec())), Zval::Str(PhpStr::new(name.to_vec())));
        a.insert(Key::Str(PhpStr::new(b"builtin".to_vec())), Zval::Bool(true));
        a.insert(Key::Str(PhpStr::new(b"nullable".to_vec())), Zval::Bool(false));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_classconst_attributes($declClass, $name, $filter = null)`: the
    /// `ReflectionAttribute`s declared on `$declClass::$name`, each carrying the
    /// lazy handle (`__class`, `__classconst`, `__index`).
    pub(super) fn ho_reflect_classconst_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let Some(ci) = self.classes[cid].consts.iter().position(|k| k.name.as_ref() == name.as_slice()) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = self.classes[cid].consts[ci].attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, aname) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(aname)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__classconst", Zval::Str(PhpStr::new(name.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_classconst_attr_new($declClass, $name, $index)` — run the class
    /// constant attribute's `new Attr(args)` thunk (validates TARGET_CLASS_CONSTANT
    /// / repeatability first).
    pub(super) fn ho_reflect_classconst_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.classconst_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        let name = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some(ci) = self.classes[cid].consts.iter().position(|k| k.name.as_ref() == name.as_slice()) {
            let list = &self.classes[cid].consts[ci].attributes;
            if let Some(attr) = list.get(idx) {
                let attr_name = attr.name.to_vec();
                let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                self.validate_attr(&attr_name, &siblings, 16, "class constant")?;
            }
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_classconst_attr_args($declClass, $name, $index)` — run the class
    /// constant attribute's argument-array thunk.
    pub(super) fn ho_reflect_classconst_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.classconst_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_param_attributes($class, $func, $pos, $filter = null)`: the
    /// `ReflectionAttribute`s on parameter `$pos` of the callable, each carrying
    /// the lazy handle (`__paramclass`, `__paramfunc`, `__parampos`, `__index`).
    pub(super) fn ho_reflect_param_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let func = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let pos = match args.get(2).map(|v| v.deref_clone()) { Some(Zval::Long(n)) => n as usize, _ => return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(3).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let Some((f, _)) = self.resolve_param_owner(&class, &func) else { return empty() };
        let Some(list) = f.param_attributes.get(pos) else { return empty() };
        let matches: Vec<(usize, Vec<u8>)> = list
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(fl) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(fl),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, aname) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(aname)));
                b.props.set(b"__paramclass", Zval::Str(PhpStr::new(class.clone())));
                b.props.set(b"__paramfunc", Zval::Str(PhpStr::new(func.clone())));
                b.props.set(b"__parampos", Zval::Long(pos as i64));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_param_attr_new($class, $func, $pos, $index)` — run the parameter
    /// attribute's `new Attr(args)` thunk (validates TARGET_PARAMETER /
    /// repeatability first).
    pub(super) fn ho_reflect_param_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some((thunk, ctx)) = self.param_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let class = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let func = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let pos = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        let idx = match args.get(3) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some((f, _)) = self.resolve_param_owner(&class, &func) {
            if let Some(list) = f.param_attributes.get(pos) {
                if let Some(attr) = list.get(idx) {
                    let attr_name = attr.name.to_vec();
                    let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                    self.validate_attr(&attr_name, &siblings, 32, "parameter")?;
                }
            }
        }
        let module = ctx.map(|c| self.class_mod(c)).unwrap_or(self.module);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, module);
        frame.class = ctx;
        frame.static_class = ctx;
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_param_attr_args($class, $func, $pos, $index)` — run the parameter
    /// attribute's argument-array thunk.
    pub(super) fn ho_reflect_param_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some((thunk, ctx)) = self.param_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let module = ctx.map(|c| self.class_mod(c)).unwrap_or(self.module);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, module);
        frame.class = ctx;
        frame.static_class = ctx;
        self.frames.push(frame);
        self.drive_to_return(baseline)
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
    /// `__reflect_closure_info($closure)`: the signature descriptor of a closure
    /// value, or `false`. A first-class callable (`strlen(...)`) reflects the
    /// named function it wraps; an ordinary closure reflects its own body via the
    /// module that compiled it.
    pub(super) fn ho_reflect_closure_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Closure(cl)) = args.first().map(|v| v.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        if let Some(name) = &cl.named {
            let nm = name.as_bytes().to_vec();
            return match self.find_user_function(&nm) {
                Some(func) => Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?))),
                None => Ok(Zval::Bool(false)),
            };
        }
        let m = self.modules[cl.module_id];
        let Some(func) = m.closures.get(cl.fn_idx) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?)))
    }
    /// `__reflect_closure_bind($closure)`: the closure's binding info as
    /// `[bound_this, scopeClassName|null, is_static]` — backs
    /// `ReflectionFunction::getClosureThis`/`getClosureScopeClass`/`isStatic`.
    pub(super) fn ho_reflect_closure_bind(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Closure(cl)) = args.first().map(|v| v.deref_clone()) else {
            return Ok(Zval::Bool(false));
        };
        let mut out = php_types::PhpArray::new();
        let _ = out.append(cl.bound_this.clone().unwrap_or(Zval::Null));
        let scope = match cl.scope {
            Some(cid) => Zval::Str(PhpStr::new(self.classes[cid].name.to_vec())),
            None => Zval::Null,
        };
        let _ = out.append(scope);
        let _ = out.append(Zval::Bool(cl.is_static));
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_closure_uses($closure)`: the closure's captured variables
    /// (`use (...)`, plus an arrow function's auto-captures) as a `name => value`
    /// map — backs `ReflectionFunction::getClosureUsedVariables()`. A by-reference
    /// capture keeps its `Zval::Ref` (so var_dump shows `&`); names come from the
    /// closure body's slot table.
    pub(super) fn ho_reflect_closure_uses(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(Zval::Closure(cl)) = args.into_iter().next().map(|v| v.deref_clone()) else {
            return empty();
        };
        let Some((func, _)) = self.closure_func_mod(&cl) else { return empty() };
        let mut arr = php_types::PhpArray::new();
        for (slot, val) in &cl.captures {
            if let Some(name) = func.slot_names.get(*slot as usize) {
                arr.insert(Key::from_bytes(name), val.clone());
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_closure_attributes($closure, $filter = null)` — backs
    /// `ReflectionFunction::getAttributes()` for a closure. The handle carries the
    /// closure value itself (`__closure_val`).
    pub(super) fn ho_reflect_closure_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(clos) = args.first().map(|v| v.deref_clone()) else { return empty() };
        let Zval::Closure(cl) = &clos else { return empty() };
        let Some((func, _)) = self.closure_func_mod(cl) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__closure_val", clos.clone());
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_closure_attr_new($closure, $index)`.
    pub(super) fn ho_reflect_closure_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_closure_attr(&args, false)
    }
    /// `__reflect_closure_attr_args($closure, $index)`.
    pub(super) fn ho_reflect_closure_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_closure_attr(&args, true)
    }
    /// `__reflect_func_info($name)`: the signature descriptor of a user function, or
    /// `false` if it is unknown (or a builtin, whose signature is not retained).
    pub(super) fn ho_reflect_func_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else { return Ok(Zval::Bool(false)) };
        let name = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let Some(func) = self.find_user_function(&name) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Array(Rc::new(self.build_func_descriptor(func, None)?)))
    }
    /// `__reflect_method_info($class, $method)`: the signature descriptor of a method
    /// plus `static`/`visibility`/`abstract`/`declaringClass`, or `false` if unknown.
    pub(super) fn ho_reflect_method_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cname = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let mname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let Some((m, decl, is_abstract)) = self.find_method_reflect(cid, &mname) else {
            return Ok(Zval::Bool(false));
        };
        let is_static = m.is_static;
        let is_final = m.is_final;
        let vis: &[u8] = match m.visibility {
            Visibility::Public => b"public",
            Visibility::Protected => b"protected",
            Visibility::Private => b"private",
        };
        let decl_name = self.classes[decl].name.to_vec();
        let mut d = self.build_func_descriptor(&m.func, Some(decl))?;
        d.insert(Key::Str(PhpStr::new(b"static".to_vec())), Zval::Bool(is_static));
        d.insert(Key::Str(PhpStr::new(b"byRef".to_vec())), Zval::Bool(m.func.by_ref));
        d.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        d.insert(Key::Str(PhpStr::new(b"visibility".to_vec())), Zval::Str(PhpStr::new(vis.to_vec())));
        d.insert(Key::Str(PhpStr::new(b"abstract".to_vec())), Zval::Bool(is_abstract));
        d.insert(Key::Str(PhpStr::new(b"declaringClass".to_vec())), Zval::Str(PhpStr::new(decl_name)));
        // file / startLine / endLine are added by build_func_descriptor (shared with
        // ReflectionFunction), with a declaration-line fallback for body-less methods.
        Ok(Zval::Array(Rc::new(d)))
    }
    /// `__reflect_invoke($object, $class, $method, $args)` — the engine behind
    /// `ReflectionMethod::invoke`/`invokeArgs`. Resolves `$method` on the *reflected*
    /// class `$class` (non-virtual, as PHP's reflection does — a subclass override is
    /// not selected) and runs it **without** a visibility check: since PHP 8.1
    /// `ReflectionMethod::invoke` calls private/protected methods without
    /// `setAccessible(true)`. `$object` binds `$this` for an instance method (ignored
    /// for a static one). Returns the method's value (or its `Generator` handle).
    pub(super) fn ho_reflect_invoke(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = args
            .first()
            .map(|v| v.deref_clone())
            .filter(|v| matches!(v, Zval::Object(_)));
        let cname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Err(PhpError::Error("ReflectionMethod::invoke(): invalid class".into())),
        };
        let mname = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => {
                return Err(PhpError::Error(
                    "ReflectionMethod::invoke(): invalid method".into(),
                ))
            }
        };
        // Array elements pass AS-IS (no deref): a `[&$x]` element keeps its Ref
        // so a by-ref parameter aliases it (UtilsTest drives the private
        // detectAndCleanUtf8(&$data) through invokeArgs); the binder derefs
        // for by-value parameters as in any call.
        let argv: Vec<Zval> = match args.get(3).map(|v| v.deref_clone()) {
            Some(Zval::Array(a)) => a.iter().map(|(_, v)| v.clone()).collect(),
            _ => Vec::new(),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let cid = *self.class_index.get(&key[..]).ok_or_else(|| {
            PhpError::Error(format!(
                "Class \"{}\" does not exist",
                String::from_utf8_lossy(&cname)
            ))
        })?;
        let (defc, midx) = resolve_method_runtime(&self.classes, cid, &mname)
            .ok_or_else(|| undefined_method(&self.classes, cid, &mname))?;
        // A static method ignores the supplied object; an instance method binds it.
        let this = if self.classes[defc].methods[midx].is_static {
            None
        } else {
            obj
        };
        let baseline = self.frames.len();
        self.enter_authorized_method(cid, this, &mname, argv)?;
        // A generator-body method pushes no frame — its `Generator` handle is left on
        // the caller's stack (mirrors `call_callable`).
        if self.frames.len() == baseline {
            return Ok(self.frames[baseline - 1]
                .stack
                .pop()
                .expect("reflect-invoke result on caller stack"));
        }
        self.drive_to_return(baseline)
    }
    /// `__reflect_class_modifiers($class)`: `['final' => bool, 'abstract' => bool]`
    /// for `ReflectionClass::isFinal()`/`isAbstract()`. Empty (both false) if the
    /// class is unknown.
    pub(super) fn ho_reflect_class_modifiers(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (mut is_final, mut is_abstract) = (false, false);
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            if let Some(&cid) = self.class_index.get(&key) {
                let cc = self.classes[cid];
                is_final = cc.is_final;
                // An interface is not reported as abstract by Reflection (only an
                // abstract *class* is), though it carries `is_abstract` internally.
                is_abstract = cc.is_abstract && !matches!(cc.instantiable, Instantiable::Interface);
            }
        }
        let mut a = php_types::PhpArray::new();
        a.insert(Key::Str(PhpStr::new(b"final".to_vec())), Zval::Bool(is_final));
        a.insert(Key::Str(PhpStr::new(b"abstract".to_vec())), Zval::Bool(is_abstract));
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_new_no_ctor($class)`: allocate an instance of `$class` with its
    /// declared property defaults (typed properties left uninitialized) but
    /// *without* invoking the constructor — `ReflectionClass::newInstanceWithoutConstructor`.
    pub(super) fn ho_reflect_new_no_ctor(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error(
                "newInstanceWithoutConstructor() expects a class name".to_string(),
            )),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let v = self.alloc_object(cid)?;
        // Non-constant declared defaults (`= []`, …) live in the `prop_init`
        // thunk, run by `Op::InitProps` at a `new` site — mirror it here.
        if let Zval::Object(rc) = &v {
            self.run_prop_init_thunk(cid, rc);
        }
        Ok(v)
    }
    /// `__reflect_new_lazy_ghost($class, $init)`: allocate an uninitialized lazy
    /// ghost of `$class` whose `$init` closure runs on first access — PHP 8.4
    /// `ReflectionClass::newLazyGhost`.
    pub(super) fn ho_reflect_new_lazy_ghost(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error("newLazyGhost() expects a class name".to_string())),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let init = args.get(1).cloned().unwrap_or(Zval::Null);
        let options = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        self.alloc_lazy(cid, init, LazyKind::Ghost, options)
    }
    /// `__reflect_new_lazy_proxy($class, $factory)`: allocate an uninitialized
    /// lazy proxy of `$class` whose `$factory` runs on first access and returns
    /// the real instance the proxy forwards to — PHP 8.4
    /// `ReflectionClass::newLazyProxy`.
    pub(super) fn ho_reflect_new_lazy_proxy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = match args.first() {
            Some(v) => convert::to_zstr_cast(v, &mut self.diags).as_bytes().to_vec(),
            None => return Err(PhpError::Error("newLazyProxy() expects a class name".to_string())),
        };
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let factory = args.get(1).cloned().unwrap_or(Zval::Null);
        let options = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        self.alloc_lazy(cid, factory, LazyKind::Proxy, options)
    }
    /// `__reflect_reset_lazy($class, $obj, $is_proxy, $init)`: reset an existing
    /// instance back to an uninitialized lazy object (PHP 8.4
    /// `ReflectionClass::resetAsLazyGhost` / `resetAsLazyProxy`). `$obj` must be
    /// an instance of the reflected `$class` (or a subclass) — else a `TypeError`
    /// naming both classes. Returns the (now lazy) object.
    pub(super) fn ho_reflect_reset_lazy(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let raw = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let cid = *self.class_index.get(&key).ok_or_else(|| {
            PhpError::Error(format!("Class \"{}\" does not exist", String::from_utf8_lossy(&raw)))
        })?;
        let obj = args.get(1).cloned().unwrap_or(Zval::Null);
        let is_proxy = convert::to_bool(args.get(2).unwrap_or(&Zval::Null), &mut self.diags);
        let op = if is_proxy { "resetAsLazyProxy" } else { "resetAsLazyGhost" };
        let Some(rc) = deref_object(&obj) else {
            return Err(PhpError::TypeError(format!(
                "ReflectionClass::{op}(): Argument #1 ($object) must be of type {}, {} given",
                String::from_utf8_lossy(&self.classes[cid].name),
                args.get(1).unwrap_or(&Zval::Null).type_name_for_error(),
            )));
        };
        let ocid = rc.borrow().class_id as usize;
        if !is_instance_of(&self.classes, self.stringable_id, ocid, cid) {
            return Err(PhpError::TypeError(format!(
                "ReflectionClass::{op}(): Argument #1 ($object) must be of type {}, {} given",
                String::from_utf8_lossy(&self.classes[cid].name),
                String::from_utf8_lossy(&self.classes[ocid].name),
            )));
        }
        // Resetting destroys the object's current incarnation: a fully
        // constructed (non-lazy) object runs its `__destruct` before being reborn
        // lazy (PHP 8.4). An uninitialized lazy wrapper was never constructed, so
        // it does not. Mirrors zend_lazy_objects.c (zend_object_make_lazy): the
        // DESTRUCTOR_CALLED flag is set *before* the call and stays set if the
        // destructor throws (the reset aborts, and the destructor must not run a
        // second time); a completed reset clears it unconditionally — the reborn
        // incarnation destructs again when later realized and dropped.
        let options = match args.get(4).map(|v| v.deref_clone()) {
            Some(Zval::Long(n)) => n as u32,
            _ => 0,
        };
        let (oid, is_real) = { let b = rc.borrow(); (b.id, b.lazy.is_none()) };
        // SKIP_DESTRUCTOR (16): the displaced incarnation's own destructor is
        // suppressed (reset_as_lazy_may_skip_destructor).
        if is_real
            && options & 16 == 0
            && !self.destructed.contains(&oid)
            && resolve_method_runtime(&self.classes, ocid, b"__destruct").is_some()
        {
            self.destructed.insert(oid);
            self.call_method_sync(obj.clone(), b"__destruct", Vec::new())?;
        }
        self.destructed.remove(&oid);
        let kind = if is_proxy { LazyKind::Proxy } else { LazyKind::Ghost };
        let init = args.get(3).cloned().unwrap_or(Zval::Null);
        self.reject_internal_lazy(ocid)?;
        // Reset through the *reflected* class's layout: a subclass's additional
        // properties are preserved (install_lazy's reflected-scope rules).
        self.install_lazy(&rc, cid, kind, init, options)?;
        Ok(obj)
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
    /// `__reflect_prop_names($class)`: the declared instance properties of
    /// `$class` (flattened, declaration order) as a list — backs
    /// `ReflectionClass::getProperties`. Virtual/static properties are omitted
    /// (only the `prop_defaults` slots).
    pub(super) fn ho_reflect_prop_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut a = php_types::PhpArray::new();
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            if let Some(&cid) = self.class_index.get(&key) {
                for (n, _) in &self.classes[cid].prop_defaults {
                    // Slots are storage-keyed (mangled for privates); reflection
                    // speaks source-level names.
                    let _ = a.append(Zval::Str(PhpStr::new(php_types::prop_display_name(n).to_vec())));
                }
            }
        }
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_method_names($class)`: every method name visible on `$class`
    /// regardless of visibility — declaration order, child-most override first,
    /// walking the parent chain (a parent's `private` methods excluded, like
    /// `ReflectionClass::getMethods`). `get_class_methods` can't back this: it
    /// filters to public when called from outside the class (PHPUnit's hook
    /// discovery needs the protected `#[Before]` methods).
    pub(super) fn ho_reflect_method_names(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut a = php_types::PhpArray::new();
        if let Some(first) = args.first() {
            let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
            let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
            if let Some(&cid) = self.class_index.get(&key) {
                let mut seen: std::collections::HashSet<Vec<u8>> = std::collections::HashSet::new();
                let mut out: Vec<Vec<u8>> = Vec::new();
                // Parent chain first (child-most override wins), then the
                // interface closure: an interface's signatures live in
                // `abstract_sigs`, which mock generation must see too.
                let mut queue: Vec<(usize, usize)> = vec![(cid, 0)];
                while let Some((c, depth)) = queue.pop() {
                    let cls = &self.classes[c];
                    for m in cls.methods.iter().chain(cls.abstract_sigs.iter()) {
                        if depth > 0 && matches!(m.visibility, crate::hir::Visibility::Private) {
                            continue;
                        }
                        if seen.insert(m.name.to_ascii_lowercase()) {
                            out.push(m.name.to_vec());
                        }
                    }
                    if let Some(p) = cls.parent {
                        queue.push((p as usize, depth + 1));
                    }
                    for &i in &cls.interfaces {
                        queue.push((i as usize, depth + 1));
                    }
                }
                for n in out {
                    let _ = a.append(Zval::Str(PhpStr::new(n)));
                }
            }
        }
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_prop_defaults($class)`: `ReflectionClass::getDefaultProperties`
    /// — statics first (their *current* value, like Zend), then the instance
    /// defaults in declaration order; a typed property without a default
    /// (`Undef`) is omitted. Names are source-level (unmangled).
    pub(super) fn ho_reflect_prop_defaults(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut out = php_types::PhpArray::new();
        let Some(first) = args.first() else { return Ok(Zval::Array(Rc::new(out))) };
        let raw = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let key = raw.strip_prefix(b"\\").unwrap_or(&raw).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Array(Rc::new(out))) };
        // Statics along the parent chain (child-most first, like the function
        // table walks elsewhere).
        let mut cur = Some(cid);
        while let Some(c) = cur {
            for (i, sp) in self.classes[c].static_props.iter().enumerate() {
                let name = sp.name.to_vec();
                if out.get(&Key::from_bytes(&name)).is_some() {
                    continue;
                }
                let cell_key = (c, name.clone());
                let v = if let Some(cell) = self.static_props.get(&cell_key) {
                    cell.borrow().deref_clone()
                } else {
                    match &self.classes[c].static_props[i].init {
                        StaticInit::Const(k) => k.to_zval(),
                        StaticInit::Thunk(_) => Zval::Null,
                    }
                };
                out.insert(Key::from_bytes(&name), v);
            }
            cur = self.classes[c].parent;
        }
        let cc = self.classes[cid];
        for (n, d) in &cc.prop_defaults {
            if cc.uninit_props.iter().any(|u| u == n) {
                continue; // typed, no default: absent from the map
            }
            let disp = php_types::prop_display_name(n).to_vec();
            if out.get(&Key::from_bytes(&disp)).is_none() {
                out.insert(Key::from_bytes(&disp), d.to_zval());
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_prop_is_static($class, $prop)`: whether `$prop` is a static
    /// property of `$class` — backs `ReflectionProperty::isStatic`.
    pub(super) fn ho_reflect_prop_is_static(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let is = self
            .class_index
            .get(&key)
            .is_some_and(|&cid| self.classes[cid].static_props.iter().any(|sp| sp.name.as_ref() == prop.as_slice()));
        Ok(Zval::Bool(is))
    }
    /// `__reflect_prop_type($class, $prop)`: the declared type of `$prop` as the
    /// descriptor `ReflectionNamedType` is built from (`false` for an untyped
    /// property) — backs `ReflectionProperty::getType` / `hasType`. `$class` is the
    /// property's declaring class, so its (flattened) `prop_info` holds the
    /// most-derived declaration's type.
    pub(super) fn ho_reflect_prop_type(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let pi = self
            .class_index
            .get(&key)
            .and_then(|&cid| self.classes[cid].prop_info.get(prop.as_slice()));
        // A composite (union/intersection) type reflects through the dedicated
        // descriptor; a single type falls back to the enforced `type_hint`.
        if let Some(z) = pi.and_then(|pi| reflect_type_descriptor(&pi.reflect_type)) {
            return Ok(z);
        }
        Ok(typehint_descriptor(&pi.and_then(|pi| pi.type_hint.clone())))
    }
    /// `__reflect_prop_details($class, $prop)`: a descriptor array backing the
    /// non-type `ReflectionProperty` accessors — `visibility`
    /// (`public`/`protected`/`private`), `readonly`, `static`, `declaringClass`,
    /// `hasDefault` and the constant `default` value. `$class` is the declaring
    /// class, whose flattened `prop_info` / `prop_defaults` carry the resolved
    /// shape; static properties fall back to their `static_props` entry.
    pub(super) fn ho_reflect_prop_details(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cls = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cls.strip_prefix(b"\\").unwrap_or(&cls).to_ascii_lowercase();
        let vis_str = |v: crate::hir::Visibility| -> &'static [u8] {
            match v {
                crate::hir::Visibility::Public => b"public",
                crate::hir::Visibility::Protected => b"protected",
                crate::hir::Visibility::Private => b"private",
            }
        };
        let mut a = php_types::PhpArray::new();
        let put = |a: &mut php_types::PhpArray, k: &[u8], v: Zval| {
            a.insert(Key::Str(PhpStr::new(k.to_vec())), v);
        };
        if let Some(&cid) = self.class_index.get(&key) {
            let c = &self.classes[cid];
            if let Some(pi) = c.prop_info.get(prop.as_slice()) {
                let vis = vis_str(pi.visibility).to_vec();
                let readonly = pi.readonly;
                let decl = self.classes[pi.declaring_class].name.to_vec();
                let uninit = c.uninit_props.iter().any(|n| n.as_ref() == prop.as_slice());
                let default = c.prop_defaults.iter().find(|(n, _)| n.as_ref() == prop.as_slice()).map(|(_, k)| k.to_zval());
                put(&mut a, b"visibility", Zval::Str(PhpStr::new(vis)));
                put(&mut a, b"readonly", Zval::Bool(readonly));
                put(&mut a, b"static", Zval::Bool(false));
                put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(decl)));
                put(&mut a, b"hasDefault", Zval::Bool(!uninit));
                put(&mut a, b"default", default.unwrap_or(Zval::Null));
                let doc = pi.doc.as_ref().map_or(Zval::Bool(false), |d| Zval::Str(PhpStr::new(d.to_vec())));
                put(&mut a, b"doc", doc);
                return Ok(Zval::Array(Rc::new(a)));
            }
            if let Some(sp) = c.static_props.iter().find(|sp| sp.name.as_ref() == prop.as_slice()) {
                let vis = vis_str(sp.visibility).to_vec();
                let decl = c.name.to_vec();
                put(&mut a, b"visibility", Zval::Str(PhpStr::new(vis)));
                put(&mut a, b"readonly", Zval::Bool(false));
                put(&mut a, b"static", Zval::Bool(true));
                put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(decl)));
                put(&mut a, b"hasDefault", Zval::Bool(true));
                put(&mut a, b"default", Zval::Null);
                return Ok(Zval::Array(Rc::new(a)));
            }
        }
        put(&mut a, b"visibility", Zval::Str(PhpStr::new(b"public".to_vec())));
        put(&mut a, b"readonly", Zval::Bool(false));
        put(&mut a, b"static", Zval::Bool(false));
        put(&mut a, b"declaringClass", Zval::Str(PhpStr::new(cls.clone())));
        put(&mut a, b"hasDefault", Zval::Bool(true));
        put(&mut a, b"default", Zval::Null);
        Ok(Zval::Array(Rc::new(a)))
    }
    /// `__reflect_prop_initialized($class, $prop, $obj)`: whether `$prop` holds a
    /// value on `$obj` (vs an uninitialized typed property) — backs
    /// `ReflectionProperty::isInitialized`. Reads the raw slot without triggering
    /// lazy initialization; a non-object (or absent slot) reads as initialized.
    pub(super) fn ho_reflect_prop_initialized(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj = args.get(2).cloned().unwrap_or(Zval::Null);
        if !matches!(obj, Zval::Object(_)) {
            return Ok(Zval::Bool(true));
        }
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let key = match self.class_index.get(&lc).copied() {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        let v = read_property(&obj, &key, &mut self.diags);
        Ok(Zval::Bool(!matches!(v, Zval::Undef)))
    }
    /// `__reflect_prop_get($class, $prop, $obj)`: read property `$prop` (declared
    /// in `$class`) of `$obj` ignoring visibility — backs
    /// `ReflectionProperty::getValue`. A lazy object initializes first; a proxy
    /// forwards to its real instance.
    pub(super) fn ho_reflect_prop_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj = self.realize_full(&args.get(2).cloned().unwrap_or(Zval::Null))?;
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let cid = self.class_index.get(&lc).copied();
        let key = match cid {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        Ok(read_property(&obj, &key, &mut self.diags))
    }
    /// `__reflect_prop_set($class, $prop, $obj, $value)`: write `$value` into
    /// property `$prop` (declared in `$class`) of `$obj` ignoring visibility —
    /// backs `ReflectionProperty::setValue`. A lazy object initializes first; a
    /// proxy forwards to its real instance. EXCEPT: a property already made
    /// non-lazy by `skipLazyInitialization`/`setRawValueWithoutLazyInitialization`
    /// is written straight into the still-lazy wrapper without running the
    /// initializer (rfc_example_004/005).
    pub(super) fn ho_reflect_prop_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let obj_arg = args.get(2).cloned().unwrap_or(Zval::Null);
        let value = args.get(3).cloned().unwrap_or(Zval::Null);
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let cid = self.class_index.get(&lc).copied();
        let key = match cid {
            Some(c) => self.prop_decl_storage_key(c, &prop),
            None => prop.clone(),
        };
        // A setValue on a still-uninitialized lazy wrapper whose target property
        // has already been dropped from the lazy set (skipLazyInitialization) must
        // NOT trigger the initializer — materialize the single slot in place.
        let skip_materialized = {
            let mut target = obj_arg.clone();
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
            deref_object(&target).is_some_and(|o| {
                let b = o.borrow();
                b.lazy.is_some()
                    && b.proxy_instance.is_none()
                    && !self
                        .lazy_props
                        .get(&b.id)
                        .is_some_and(|set| set.iter().any(|n| n.as_ref() == key.as_slice()))
            })
        };
        if skip_materialized {
            self.lazy_materialize(&obj_arg, &key, value)?;
            return Ok(Zval::Null);
        }
        let obj = self.realize_full(&obj_arg)?;
        if let Some(old) = write_property(&obj, &key, value)? {
            self.gc_note(&old);
        }
        Ok(Zval::Null)
    }
    /// `__reflect_static_prop_get($class, $prop)`: read a static property
    /// ignoring visibility — backs `ReflectionProperty::getValue()` on statics.
    /// A constant default initializes lazily; a not-yet-run thunk default reads
    /// NULL (declared residue).
    pub(super) fn ho_reflect_static_prop_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else { return Ok(Zval::Null) };
        let Some((decl, idx)) = find_static_prop(&self.classes, cid, &prop) else {
            return Ok(Zval::Null);
        };
        let key = (decl, prop);
        if let Some(cell) = self.static_props.get(&key) {
            return Ok(cell.borrow().deref_clone());
        }
        match &self.classes[decl].static_props[idx].init {
            StaticInit::Const(c) => {
                let v = c.to_zval();
                self.static_props.insert(key, Rc::new(RefCell::new(v.clone())));
                Ok(v)
            }
            StaticInit::Thunk(_) => Ok(Zval::Null),
        }
    }
    /// `__reflect_static_prop_set($class, $prop, $value)`: write a static
    /// property ignoring visibility — backs `ReflectionProperty::setValue` with
    /// a NULL object (Composer pokes `InstalledVersions::$selfDir`). Returns
    /// whether the property exists.
    pub(super) fn ho_reflect_static_prop_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let value = args.get(2).cloned().unwrap_or(Zval::Null).deref_clone();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else { return Ok(Zval::Bool(false)) };
        let Some((decl, _)) = find_static_prop(&self.classes, cid, &prop) else {
            return Ok(Zval::Bool(false));
        };
        let key = (decl, prop);
        match self.static_props.get(&key) {
            Some(cell) => *cell.borrow_mut() = value,
            None => {
                self.static_props.insert(key, Rc::new(RefCell::new(value)));
            }
        }
        Ok(Zval::Bool(true))
    }
    /// `__reflect_static_vars($class|null, $function)`: a function's local
    /// `static $v` variables as a `name => value` map — backs
    /// `ReflectionFunctionAbstract::getStaticVariables()`. The current persistent
    /// cell value wins; before the function has run the declared initial value is
    /// used. `$class` selects a method, else a free function.
    pub(super) fn ho_reflect_static_vars(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let fname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return empty(),
        };
        // (name, cell id, folded initial value). The Func ref lives for `'m`, so it
        // coexists with the `self.statics` reads below.
        let entries: Vec<(Vec<u8>, u32, Zval)> = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(c)) => {
                let key = c.as_bytes().strip_prefix(b"\\").unwrap_or(c.as_bytes()).to_ascii_lowercase();
                let Some(&cid) = self.class_index.get(&key) else { return empty() };
                let Some((m, _, _)) = self.find_method_reflect(cid, &fname) else { return empty() };
                m.func.static_vars.iter().map(|s| (s.name.to_vec(), s.id, static_var_init(&s.init))).collect()
            }
            _ => {
                let Some(func) = self.find_user_function(&fname) else { return empty() };
                func.static_vars.iter().map(|s| (s.name.to_vec(), s.id, static_var_init(&s.init))).collect()
            }
        };
        let mut arr = php_types::PhpArray::new();
        for (name, id, init) in entries {
            let val = self
                .statics
                .get(id as usize)
                .and_then(|c| c.as_ref())
                .map(|c| c.borrow().deref_clone())
                .unwrap_or(init);
            arr.insert(Key::from_bytes(&name), val);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_static_props($class)`: all static properties of `$class` (its own
    /// and inherited) as a `name => value` map — backs
    /// `ReflectionClass::getStaticProperties()`. Derived class first; a name already
    /// seen (child redeclaration) keeps the derived value. A const default is
    /// realized lazily, a not-yet-run thunk reads NULL (as the single-prop getter).
    pub(super) fn ho_reflect_static_props(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let class = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let lc = class.strip_prefix(b"\\").unwrap_or(&class).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&lc) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let mut chain: Vec<usize> = Vec::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            chain.push(ci);
            c = self.classes[ci].parent;
        }
        let mut seen: HashSet<Vec<u8>> = HashSet::new();
        let mut out = php_types::PhpArray::new();
        for ci in chain {
            let props: Vec<(usize, Vec<u8>)> = self.classes[ci]
                .static_props
                .iter()
                .enumerate()
                .map(|(idx, sp)| (idx, sp.name.as_ref().to_vec()))
                .collect();
            for (idx, name) in props {
                if !seen.insert(name.clone()) {
                    continue;
                }
                let key = (ci, name.clone());
                let val = if let Some(cell) = self.static_props.get(&key) {
                    cell.borrow().deref_clone()
                } else {
                    match &self.classes[ci].static_props[idx].init {
                        StaticInit::Const(cst) => cst.to_zval(),
                        StaticInit::Thunk(_) => Zval::Null,
                    }
                };
                out.insert(Key::from_bytes(&name), val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_prop_attributes($class, $prop, $filter = null)`: the host backing
    /// of `ReflectionProperty::getAttributes()`. Returns `ReflectionAttribute`s for
    /// the `#[…]` declared on `$class::$prop`, each carrying the lazy handle
    /// (`__class`, `__prop`, `__index`) the materializers below use.
    pub(super) fn ho_reflect_prop_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let prop = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = match self.classes[cid].prop_attributes.get(prop.as_slice()) {
            Some(list) => list
                .iter()
                .enumerate()
                .filter(|(_, a)| match &filter {
                    None => true,
                    Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
                })
                .map(|(i, a)| (i, a.name.to_vec()))
                .collect(),
            None => return empty(),
        };
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__prop", Zval::Str(PhpStr::new(prop.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_prop_attr_new($class, $prop, $index)` — run the property
    /// attribute's `new Attr(args)` thunk (mirrors `__reflect_attr_newinstance`).
    pub(super) fn ho_reflect_prop_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.prop_attr_thunk(&args, false) else { return Ok(Zval::Null) };
        let cname = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let cid = self.class_index.get(&cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase()).copied().unwrap_or(0);
        // Validate the property attribute's target/repeatability first.
        let prop = match args.get(1) { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
        let idx = match args.get(2) { Some(Zval::Long(i)) => *i as usize, _ => 0 };
        if let Some(list) = self.classes[cid].prop_attributes.get(prop.as_slice()) {
            if let Some(attr) = list.get(idx) {
                let attr_name = attr.name.to_vec();
                let siblings: Vec<Vec<u8>> = list.iter().map(|a| a.name.to_vec()).collect();
                self.validate_attr(&attr_name, &siblings, 8, "property")?;
            }
        }
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_prop_attr_args($class, $prop, $index)` — run the property
    /// attribute's argument-array thunk (mirrors `__reflect_attr_arguments`).
    pub(super) fn ho_reflect_prop_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(thunk) = self.prop_attr_thunk(&args, true) else {
            return Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        };
        let cid = self.class_index.get(&{
            let c = match args.first() { Some(Zval::Str(s)) => s.as_bytes().to_vec(), _ => Vec::new() };
            c.strip_prefix(b"\\").unwrap_or(&c).to_ascii_lowercase()
        }).copied().unwrap_or(0);
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_func_attributes($func, $filter = null)` — backs
    /// `ReflectionFunction::getAttributes()`. Each `ReflectionAttribute` carries
    /// the `__func` handle the materializers below resolve.
    pub(super) fn ho_reflect_func_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let fname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let Some(func) = self.find_user_function(&fname) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__func", Zval::Str(PhpStr::new(fname.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_func_attr_new($func, $index)`.
    pub(super) fn ho_reflect_func_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_func_attr(&args, false)
    }
    /// `__reflect_func_attr_args($func, $index)`.
    pub(super) fn ho_reflect_func_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_func_attr(&args, true)
    }
    /// `__reflect_method_attributes($class, $method, $filter = null)` — backs
    /// `ReflectionMethod::getAttributes()`. Handle: `__class` + `__method`.
    pub(super) fn ho_reflect_method_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let method = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        // `find_method_reflect` also searches abstract signatures and the interface
        // graph, so an interface/abstract method's `#[…]` attributes are visible
        // (resolve_method_runtime only sees concrete `.methods`).
        let Some((m, _defc, _)) = self.find_method_reflect(cid, &method) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(2).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = m.func.attributes.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__method", Zval::Str(PhpStr::new(method.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_method_attr_new($class, $method, $index)`.
    pub(super) fn ho_reflect_method_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_method_attr(&args, false)
    }
    /// `__reflect_method_attr_args($class, $method, $index)`.
    pub(super) fn ho_reflect_method_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_method_attr(&args, true)
    }
    /// `__reflect_const_attributes($const, $filter = null)` — backs
    /// `ReflectionConstant::getAttributes()`. Top-level constants are
    /// case-sensitive; the handle is `__const`.
    pub(super) fn ho_reflect_const_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let cname = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_vec();
        let Some(attrs) = self.module.const_attributes.get(key.as_slice()) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else { return empty() };
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => { let raw = s.as_bytes(); Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec()) }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = attrs.iter().enumerate()
            .filter(|(_, a)| match &filter { None => true, Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f) })
            .map(|(i, a)| (i, a.name.to_vec())).collect();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__const", Zval::Str(PhpStr::new(key.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_const_attr_new($const, $index)`.
    pub(super) fn ho_reflect_const_attr_new(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_const_attr(&args, false)
    }
    /// `__reflect_const_attr_args($const, $index)`.
    pub(super) fn ho_reflect_const_attr_args(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        self.run_const_attr(&args, true)
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
    /// `__reflect_class_attributes($class, $filter = null)`: the host backing of
    /// `ReflectionClass::getAttributes()`. Returns an array of `ReflectionAttribute`
    /// objects, one per `#[…]` declared on `$class` (optionally filtered by
    /// attribute name). Each carries `name` plus the private handle (`__class`,
    /// `__index`) the other reflection builtins use to materialise it lazily.
    pub(super) fn ho_reflect_class_attributes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let empty = || Ok(Zval::Array(Rc::new(php_types::PhpArray::new())));
        let Some(first) = args.first() else { return empty() };
        let cname = convert::to_zstr_cast(first, &mut self.diags).as_bytes().to_vec();
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return empty() };
        let Some(&ra_cid) = self.class_index.get(&b"reflectionattribute"[..]) else {
            return empty();
        };
        // A non-empty string second argument restricts the result to that attribute
        // class (case-insensitively, leading `\` stripped) — `getAttributes($name)`.
        let filter: Option<Vec<u8>> = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => {
                let raw = s.as_bytes();
                Some(raw.strip_prefix(b"\\").unwrap_or(raw).to_vec())
            }
            _ => None,
        };
        let matches: Vec<(usize, Vec<u8>)> = self.classes[cid]
            .attributes
            .iter()
            .enumerate()
            .filter(|(_, a)| match &filter {
                None => true,
                Some(f) => a.name.strip_prefix(b"\\").unwrap_or(&a.name).eq_ignore_ascii_case(f),
            })
            .map(|(i, a)| (i, a.name.to_vec()))
            .collect();
        let target = self.classes[cid].name.to_vec();
        let mut arr = php_types::PhpArray::new();
        for (idx, name) in matches {
            let obj = self.alloc_object(ra_cid)?;
            if let Zval::Object(o) = &obj {
                let mut b = o.borrow_mut();
                b.props.set(b"name", Zval::Str(PhpStr::new(name)));
                b.props.set(b"__class", Zval::Str(PhpStr::new(target.clone())));
                b.props.set(b"__index", Zval::Long(idx as i64));
            }
            let _ = arr.append(obj);
        }
        Ok(Zval::Array(Rc::new(arr)))
    }
    /// `__reflect_attr_newinstance($class, $index)`: build the attribute object by
    /// running its retained `new Attr(args)` thunk in the attributed class's context
    /// (so `self::`/constants in the argument list resolve as written).
    pub(super) fn ho_reflect_attr_newinstance(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (cid, idx) = self.reflect_attr_handle(&args)?;
        let cc = self.classes[cid];
        // Validate the class attribute's target/repeatability first.
        let attr_name = cc.attributes[idx].name.to_vec();
        let siblings: Vec<Vec<u8>> = cc.attributes.iter().map(|a| a.name.to_vec()).collect();
        self.validate_attr(&attr_name, &siblings, 1, "class")?;
        let thunk = &cc.attributes[idx].new_thunk;
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_attr_arguments($class, $index)`: run the attribute's argument-array
    /// thunk (positional args int-keyed, named args string-keyed) — `getArguments()`.
    pub(super) fn ho_reflect_attr_arguments(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let (cid, idx) = self.reflect_attr_handle(&args)?;
        let cc = self.classes[cid];
        let thunk = &cc.attributes[idx].args_thunk;
        let baseline = self.frames.len();
        let mut frame = Frame::new(thunk, self.class_mod(cid));
        frame.class = Some(cid);
        frame.static_class = Some(cid);
        self.frames.push(frame);
        self.drive_to_return(baseline)
    }
    /// `__reflect_prop_declaring_class($class, $prop)`: the class that *declares*
    /// `$prop` — the most-derived class in `$class`'s ancestry whose own (instance
    /// or static) property list contains it. A child that redeclares an inherited
    /// property shadows the parent, so this returns the child, matching
    /// `ReflectionProperty::$class`. `false` if no class declares it.
    pub(super) fn ho_reflect_prop_declaring_class(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cname = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let pname = match args.get(1).map(|v| v.deref_clone()) {
            Some(Zval::Str(s)) => s.as_bytes().to_vec(),
            _ => return Ok(Zval::Bool(false)),
        };
        let key = cname.strip_prefix(b"\\").unwrap_or(&cname).to_ascii_lowercase();
        let Some(&cid) = self.class_index.get(&key) else { return Ok(Zval::Bool(false)) };
        let mut cur = Some(cid);
        while let Some(c) = cur {
            let cc = self.classes[c];
            let declares = cc.own_prop_vis.iter().any(|(n, _)| n.as_ref() == pname.as_slice())
                || cc.static_props.iter().any(|sp| sp.name.as_ref() == pname.as_slice());
            if declares {
                return Ok(Zval::Str(PhpStr::new(cc.name.to_vec())));
            }
            cur = cc.parent;
        }
        Ok(Zval::Bool(false))
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
    /// `__reflect_object_bind($reflectionObject, $instance)`: records the instance a
    /// `ReflectionObject` was built for (keyed by the ReflectionObject's id), so the
    /// prelude need not hold it as a var_dump-visible property.
    pub(super) fn ho_reflect_object_bind(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ro = args.first().map(|v| v.deref_clone());
        let inst = args.get(1).map(|v| v.deref_clone());
        if let (Some(Zval::Object(r)), Some(inst)) = (ro, inst) {
            let id = r.borrow().id;
            self.reflect_object_bound.insert(id, inst);
        }
        Ok(Zval::Null)
    }
    /// `__reflect_object_dynprops($reflectionObject)`: the names of the bound
    /// instance's *dynamic* (undeclared, unmangled) properties, in instance order.
    /// Reads the property table directly — it does **not** realise a lazy object
    /// (PHP's `ReflectionObject::__toString` enumerates dynamic props without
    /// triggering init: Zend/tests/lazy_objects/init_trigger_reflection_object_toString).
    pub(super) fn ho_reflect_object_dynprops(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Object(ro)) = args.into_iter().next().map(|v| v.deref_clone()) else {
            return Ok(Zval::Array(Rc::new(PhpArray::new())));
        };
        let ro_id = ro.borrow().id;
        let Some(Zval::Object(o)) = self.reflect_object_bound.get(&ro_id).cloned() else {
            return Ok(Zval::Array(Rc::new(PhpArray::new())));
        };
        let cid = o.borrow().class_id as usize;
        // Declared property names across the whole parent chain.
        let mut declared: HashSet<Box<[u8]>> = HashSet::new();
        let mut c = Some(cid);
        while let Some(ci) = c {
            for (name, _) in &self.classes[ci].own_prop_vis {
                declared.insert(name.clone());
            }
            c = self.classes[ci].parent;
        }
        let mut arr = PhpArray::new();
        let b = o.borrow();
        for (name, _) in b.props.iter() {
            if !declared.contains(name) && !name.starts_with(b"\0") {
                let _ = arr.append(Zval::Str(PhpStr::new(name.to_vec())));
            }
        }
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
    /// `__reflect_class_loc(name) -> [file|false, startLine, endLine]`: the file
    /// that declared the class (from its first method's compiled unit; the
    /// property-init thunk as fallback) and the line span covered by its method
    /// bodies — `false`/0 for a prelude ("internal") class or one with no
    /// compiled body. Serves ReflectionClass::getFileName/getStartLine/getEndLine
    /// (the span is an approximation from the op line tables, not the `class`
    /// keyword's line).
    pub(super) fn ho_reflect_class_loc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mut out = PhpArray::new();
        let Some(cid) = self.resolve_named_class_with_autoload(&args)? else {
            let _ = out.append(Zval::Bool(false));
            let _ = out.append(Zval::Long(0));
            let _ = out.append(Zval::Long(0));
            return Ok(Zval::Array(Rc::new(out)));
        };
        let c = &self.classes[cid];
        // Prefer the declaring unit recorded on the class itself; older paths
        // derived it from the first method, kept as fallback for classes whose
        // `file` is empty (e.g. synthesized ones).
        let file: Option<&[u8]> = Some(&c.file[..])
            .filter(|f| !f.is_empty())
            .or_else(|| c.methods.first().map(|m| &m.func.file[..]))
            .or_else(|| c.prop_init.as_ref().map(|f| &f.file[..]));
        match file {
            Some(f) if f != b"prelude" => {
                let _ = out.append(Zval::Str(PhpStr::new(f.to_vec())));
                // getStartLine is the `class` keyword's line; getEndLine the closing
                // `}` line, both recorded from the source span. Fall back to the
                // method op-line span for a class compiled before this was tracked.
                let start = c.line;
                let end = if c.end_line > 0 {
                    c.end_line
                } else {
                    let mut e = 0u32;
                    for m in c.methods.iter().filter(|m| m.func.file[..] == f[..]) {
                        for &l in m.func.lines.iter() {
                            e = e.max(l);
                        }
                    }
                    e.max(start)
                };
                let _ = out.append(Zval::Long(i64::from(start)));
                let _ = out.append(Zval::Long(i64::from(end)));
            }
            _ => {
                let _ = out.append(Zval::Bool(false));
                let _ = out.append(Zval::Long(0));
                let _ = out.append(Zval::Long(0));
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_class_real_name(name) -> string|false`: the CANONICAL declared
    /// name of a class (class names resolve case-insensitively, but the reflected
    /// `ReflectionClass::$name` must carry the real casing, not the argument's).
    /// `false` when the class does not exist. Lets the prelude normalize a
    /// `new ReflectionClass('MY\CLASS')` name back to `My\Class`
    /// (Doctrine ClassMetadata::initializeReflection: ClassMetadataTest::testClassCaseSensitivity).
    pub(super) fn ho_reflect_class_real_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match self.resolve_named_class_with_autoload(&args)? {
            Some(cid) => Ok(Zval::Str(PhpStr::new(self.classes[cid].name.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }
    /// `__reflect_ref_id(array, key) -> string|false`: the identity of the
    /// reference an array element holds, or `false` when the element is not a
    /// reference (or the key/argument is invalid). Two elements that alias the same
    /// reference report the same id — the contract `ReflectionReference::getId()`
    /// relies on (Symfony var-exporter / deepclone reference tracking). The id is
    /// the reference cell's address rendered as text; only equality is meaningful.
    pub(super) fn ho_reflect_ref_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(Zval::Array(a)) = args.first() else {
            return Ok(Zval::Bool(false));
        };
        let Some(key) = args.get(1).and_then(arrays::coerce_key_silent) else {
            return Ok(Zval::Bool(false));
        };
        match a.get(&key) {
            Some(Zval::Ref(cell)) => {
                let id = format!("{:p}", Rc::as_ptr(cell));
                Ok(Zval::Str(PhpStr::new(id.into_bytes())))
            }
            _ => Ok(Zval::Bool(false)),
        }
    }
    /// `__reflect_gen_info(gen) -> [line, file|false, this, funcName]`: the state of
    /// a `Generator`'s suspended frame, backing `ReflectionGenerator`. A running or
    /// finished generator (no parked frame) reports line 0 / file false / this null.
    pub(super) fn ho_reflect_gen_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let g = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Generator(g)) => g,
            _ => return Ok(Zval::Bool(false)),
        };
        let (id, func_name, is_done) = {
            let b = g.borrow();
            (b.id, b.func_name.clone(), matches!(b.status, GenStatus::Done))
        };
        let mut out = PhpArray::new();
        match self.generators.get(&id) {
            Some(frame) => {
                let line = frame.func.lines.get(frame.ip).copied().unwrap_or(0);
                out.insert(Key::Int(0), Zval::Long(i64::from(line)));
                out.insert(Key::Int(1), Zval::Str(PhpStr::new(frame.func.file.to_vec())));
                out.insert(Key::Int(2), frame.this.clone().unwrap_or(Zval::Null));
            }
            None => {
                out.insert(Key::Int(0), Zval::Long(0));
                out.insert(Key::Int(1), Zval::Bool(false));
                out.insert(Key::Int(2), Zval::Null);
            }
        }
        out.insert(Key::Int(3), Zval::Str(PhpStr::new(func_name.to_vec())));
        out.insert(Key::Int(4), Zval::Bool(is_done));
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_fiber_info(fiber) -> [line, file|false, callable]`: the suspended
    /// frame of a `Fiber` and the callable it was constructed with, backing
    /// `ReflectionFiber`. A not-started / finished fiber reports line 0 / file false.
    pub(super) fn ho_reflect_fiber_info(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let obj = match args.first().map(|v| v.deref_clone()) {
            Some(Zval::Object(o)) => o,
            _ => return Ok(Zval::Bool(false)),
        };
        let (id, cid) = {
            let b = obj.borrow();
            (b.id, b.class_id as usize)
        };
        let key = self.host_prop_key(cid, b"callable");
        let callable = obj.borrow().props.get(key.as_slice()).cloned().unwrap_or(Zval::Null);
        let mut out = PhpArray::new();
        match self.fibers.get(&id).and_then(|s| s.parked.last()) {
            Some(frame) => {
                let line = frame.func.lines.get(frame.ip).copied().unwrap_or(0);
                out.insert(Key::Int(0), Zval::Long(i64::from(line)));
                out.insert(Key::Int(1), Zval::Str(PhpStr::new(frame.func.file.to_vec())));
            }
            None => {
                out.insert(Key::Int(0), Zval::Long(0));
                out.insert(Key::Int(1), Zval::Bool(false));
            }
        }
        out.insert(Key::Int(2), callable);
        Ok(Zval::Array(Rc::new(out)))
    }
    /// `__reflect_class_doc(name) -> string|false`: the class declaration's
    /// retained `/** ... */` doc comment (ReflectionClass::getDocComment), false
    /// for none / an unknown class / a prelude ("internal") class.
    pub(super) fn ho_reflect_class_doc(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let Some(cid) = self.resolve_named_class_with_autoload(&args)? else {
            return Ok(Zval::Bool(false));
        };
        let c = &self.classes[cid];
        Ok(match (&c.doc, &c.file[..]) {
            (Some(d), f) if f != b"prelude" => Zval::Str(PhpStr::new(d.to_vec())),
            _ => Zval::Bool(false),
        })
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
        let (stream_type, seekable) = match &s.backend {
            StreamBackend::File(_) => ("STDIO", true),
            StreamBackend::Memory(_) => ("MEMORY", true),
            StreamBackend::Stdin | StreamBackend::Stdout | StreamBackend::Stderr => {
                ("STDIO", false)
            }
            StreamBackend::ChildStdin(_)
            | StreamBackend::ChildStdout(_)
            | StreamBackend::ChildStderr(_) => ("STDIO", false),
            StreamBackend::Tcp(_) | StreamBackend::Udp(_) => ("tcp_socket/unknown", false),
        };
        let wrapper = if s.uri.starts_with(b"php://") { "PHP" } else { "plainfile" };
        let mut arr = PhpArray::new();
        arr.insert(Key::from_bytes(b"timed_out"), Zval::Bool(false));
        arr.insert(Key::from_bytes(b"blocked"), Zval::Bool(true));
        arr.insert(Key::from_bytes(b"eof"), Zval::Bool(s.eof));
        arr.insert(
            Key::from_bytes(b"wrapper_type"),
            Zval::Str(PhpStr::new(wrapper.as_bytes().to_vec())),
        );
        arr.insert(
            Key::from_bytes(b"stream_type"),
            Zval::Str(PhpStr::new(stream_type.as_bytes().to_vec())),
        );
        arr.insert(Key::from_bytes(b"mode"), Zval::Str(PhpStr::new(s.mode.clone())));
        arr.insert(Key::from_bytes(b"unread_bytes"), Zval::Long(0));
        arr.insert(Key::from_bytes(b"seekable"), Zval::Bool(seekable));
        arr.insert(Key::from_bytes(b"uri"), Zval::Str(PhpStr::new(s.uri.clone())));
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
                    };
                    return Ok(self.alloc_resource(stream));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Ok(Zval::Bool(false)),
            }
        }
        Ok(Zval::Bool(false))
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
        let fail = |vm: &mut Self, errno: i64, errstr: String| -> Result<Zval, PhpError> {
            vm.diags.push(Diag::Warning(format!(
                "fsockopen(): Unable to connect to {text}:{port_arg} ({errstr})"
            )));
            let mut out = PhpArray::new();
            let _ = out.append(Zval::Bool(false));
            let _ = out.append(Zval::Long(errno));
            let _ = out.append(Zval::Str(PhpStr::new(errstr.into_bytes())));
            Ok(Zval::Array(Rc::new(out)))
        };
        let (scheme, rest) = match text.split_once("://") {
            Some((s, r)) => (s.to_ascii_lowercase(), r.to_string()),
            None => ("tcp".to_string(), text.clone()),
        };
        if scheme != "tcp" && scheme != "udp" {
            return fail(
                self,
                0,
                format!(
                    "Unable to find the socket transport \"{scheme}\" - did you forget to enable it when you configured PHP?"
                ),
            );
        }
        // host[:port] — an explicit :port in the target wins over the argument.
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) if p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty() => {
                (h.to_string(), p.parse::<i64>().unwrap_or(-1))
            }
            _ => (rest.clone(), port_arg),
        };
        if !(0..=65535).contains(&port) {
            return fail(self, 0, format!("Failed to parse address \"{rest}\""));
        }
        use std::net::ToSocketAddrs;
        let addr = match (host.as_str(), port as u16).to_socket_addrs() {
            Ok(mut it) => match it.next() {
                Some(a) => a,
                None => {
                    return fail(
                        self,
                        0,
                        format!("php_network_getaddresses: getaddrinfo for {host} failed: nodename nor servname provided, or not known"),
                    )
                }
            },
            Err(_) => {
                return fail(
                    self,
                    0,
                    format!("php_network_getaddresses: getaddrinfo for {host} failed: nodename nor servname provided, or not known"),
                )
            }
        };
        let backend = if scheme == "tcp" {
            match std::net::TcpStream::connect_timeout(
                &addr,
                std::time::Duration::from_secs_f64(timeout),
            ) {
                Ok(t) => php_types::stream::StreamBackend::Tcp(t),
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(0) as i64;
                    let msg = e.to_string();
                    let msg = msg.split(" (os error").next().unwrap_or(&msg).to_string();
                    return fail(self, errno, msg);
                }
            }
        } else {
            let sock = std::net::UdpSocket::bind("0.0.0.0:0")
                .and_then(|s| s.connect(addr).map(|()| s));
            match sock {
                Ok(s) => php_types::stream::StreamBackend::Udp(s),
                Err(e) => {
                    let errno = e.raw_os_error().unwrap_or(0) as i64;
                    let msg = e.to_string();
                    let msg = msg.split(" (os error").next().unwrap_or(&msg).to_string();
                    return fail(self, errno, msg);
                }
            }
        };
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let stream = php_types::Stream {
            backend,
            readable: true,
            writable: true,
            eof: false,
            uri: target,
            mode: b"r+".to_vec(),
        };
        let res = Zval::Resource(Rc::new(RefCell::new(Resource::new(id, stream))));
        let mut out = PhpArray::new();
        let _ = out.append(res);
        let _ = out.append(Zval::Long(0));
        let _ = out.append(Zval::Str(PhpStr::new(Vec::new())));
        Ok(Zval::Array(Rc::new(out)))
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
                        (convert::to_zstr(sep, &mut self.diags).as_bytes().to_vec(), Rc::clone(a))
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
