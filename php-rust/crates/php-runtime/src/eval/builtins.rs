//! Evaluator-dispatched builtins: higher-order functions (array_map/filter/walk,
//! usort, call_user_func*), the preg_*/mb_ereg* regex families, json_decode,
//! serialize/unserialize, fopen/dir/resource openers and class-introspection
//! (get_class_methods/get_object_vars). Split out of `eval.rs` (step 60) as one
//! cohesive `impl Evaluator` block; behaviour is unchanged.
use std::cell::RefCell;
use std::rc::Rc;

use php_types::{
    convert, Diag,
    DirHandle, Key, Object, PhpArray,
    PhpError, PhpStr, Props, ResKind, Resource, Stream, StreamBackend, Zval,
};

use crate::hir::{
    ClassDecl, ClassId, Expr, ExprKind,
};

use super::*;

impl<'p> Evaluator<'p> {
    /// Dispatch a higher-order builtin that the evaluator implements directly
    /// (step 18, D-18.6). Returns `None` for a name we do not intercept, so the
    /// caller falls through to the ordinary registry lookup.
    pub(super) fn dispatch_higher_order(
        &mut self,
        name: &[u8],
        args: &[Expr],
    ) -> Option<Result<Zval, PhpError>> {
        match name {
            b"is_callable" => Some(self.ho_is_callable(args)),
            b"call_user_func" => Some(self.ho_call_user_func(args)),
            b"call_user_func_array" => Some(self.ho_call_user_func_array(args)),
            b"array_map" => Some(self.ho_array_map(args)),
            b"array_filter" => Some(self.ho_array_filter(args)),
            b"array_walk" => Some(self.ho_array_walk(args)),
            b"usort" => Some(self.ho_usort(args)),
            b"json_decode" => Some(self.ho_json_decode(args)),
            b"strtok" => Some(self.ho_strtok(args)),
            b"unserialize" => Some(self.ho_unserialize(args)),
            b"fopen" => Some(self.ho_fopen(args)),
            b"tmpfile" => Some(self.ho_tmpfile(args)),
            b"opendir" => Some(self.ho_opendir(args)),
            b"sscanf" => Some(self.ho_sscanf(args)),
            b"fscanf" => Some(self.ho_fscanf(args)),
            b"preg_match" => Some(self.ho_preg_match(args)),
            b"preg_match_all" => Some(self.ho_preg_match_all(args)),
            b"preg_replace" => Some(self.ho_preg_replace(args)),
            b"preg_replace_callback" => Some(self.ho_preg_replace_callback(args)),
            b"preg_split" => Some(self.ho_preg_split(args)),
            b"preg_quote" => Some(self.ho_preg_quote(args)),
            b"mb_ereg" => Some(self.ho_mb_ereg(args, false)),
            b"mb_eregi" => Some(self.ho_mb_ereg(args, true)),
            b"mb_ereg_replace" => Some(self.ho_mb_ereg_replace(args, false)),
            b"mb_eregi_replace" => Some(self.ho_mb_ereg_replace(args, true)),
            b"mb_ereg_replace_callback" => Some(self.ho_mb_ereg_replace_callback(args)),
            b"mb_split" => Some(self.ho_mb_split(args)),
            b"mb_ereg_match" => Some(self.ho_mb_ereg_match(args)),
            b"mb_regex_encoding" => Some(self.ho_mb_regex_encoding(args)),
            b"mb_regex_set_options" => Some(self.ho_mb_regex_set_options(args)),
            b"mb_ereg_search_init" => Some(self.ho_mb_ereg_search_init(args)),
            b"mb_ereg_search" => Some(self.ho_mb_ereg_search(args)),
            b"mb_ereg_search_pos" => Some(self.ho_mb_ereg_search_pos(args)),
            b"mb_ereg_search_regs" => Some(self.ho_mb_ereg_search_regs(args)),
            b"mb_ereg_search_getregs" => Some(self.ho_mb_ereg_search_getregs()),
            b"mb_ereg_search_getpos" => Some(Ok(Zval::Long(self.mb_regex.search_pos as i64))),
            b"mb_ereg_search_setpos" => Some(self.ho_mb_ereg_search_setpos(args)),
            _ => None,
        }
    }

    /// `array_walk(&$array, $callback, $arg = null)` (step 32): apply `$callback`
    /// to each element. When the callback's first parameter is by-reference the
    /// element is passed through a shared cell and the mutation is written back;
    /// otherwise it is passed by value (read-only). Returns true. The keys are
    /// never modified.
    fn ho_array_walk(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let (Some(arr_expr), Some(cb_expr)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(format!(
                "array_walk() expects at least 2 arguments, {} given",
                args.len()
            )));
        };
        let ExprKind::Var(slot) = arr_expr.kind else {
            return Err(PhpError::Error(
                "array_walk(): Argument #1 ($array) could not be passed by reference".to_string(),
            ));
        };
        let callback = self.eval(cb_expr)?.deref_clone();
        let extra = match args.get(2) {
            Some(e) => Some(self.eval(e)?.deref_clone()),
            None => None,
        };
        let by_ref = self.callable_first_by_ref(&callback);
        let cell = self.slot_cell(slot as usize);
        let entries: Vec<(Key, Zval)> = match &*cell.borrow() {
            Zval::Array(a) => a.iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_walk(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };

        let mut out = PhpArray::new();
        for (k, v) in entries {
            let key_z = match &k {
                Key::Int(i) => Zval::Long(*i),
                Key::Str(s) => Zval::Str(Rc::clone(s)),
            };
            let new_v = if by_ref {
                let vcell = Rc::new(RefCell::new(v));
                let mut argv = vec![Zval::Ref(Rc::clone(&vcell)), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_value(callback.clone(), argv)?;
                let updated = vcell.borrow().clone();
                updated
            } else {
                let mut argv = vec![v.clone(), key_z];
                if let Some(e) = &extra {
                    argv.push(e.clone());
                }
                self.call_value(callback.clone(), argv)?;
                v
            };
            out.insert(k, new_v);
        }
        *cell.borrow_mut() = Zval::Array(Rc::new(out));
        Ok(Zval::Bool(true))
    }

    /// Whether a callable's first parameter is declared by-reference (`&$x`).
    /// Used by `array_walk` to decide if element mutations propagate. Only user
    /// closures and named user functions are inspected; anything else is false.
    fn callable_first_by_ref(&self, callee: &Zval) -> bool {
        match callee {
            Zval::Closure(cl) => match &cl.named {
                Some(name) => self.named_first_by_ref(name.as_bytes()),
                None => self
                    .closures
                    .get(cl.fn_idx)
                    .and_then(|f| f.params.first())
                    .is_some_and(|p| p.by_ref),
            },
            Zval::Str(s) => self.named_first_by_ref(s.as_bytes()),
            Zval::Ref(c) => {
                let inner = c.borrow().clone();
                self.callable_first_by_ref(&inner)
            }
            _ => false,
        }
    }

    /// First-parameter by-reference flag of a named user function.
    fn named_first_by_ref(&self, name: &[u8]) -> bool {
        self.fn_index
            .get(&name.to_ascii_lowercase())
            .and_then(|&i| self.funcs.get(i))
            .and_then(|f| f.params.first())
            .is_some_and(|p| p.by_ref)
    }

    /// Evaluate an argument and coerce it to a byte string (used by `preg_*` for
    /// the pattern and subject).
    fn preg_str(&mut self, e: &Expr) -> Result<Vec<u8>, PhpError> {
        let v = self.eval(e)?.deref_clone();
        Ok(self.stringify(&v)?.as_bytes().to_vec())
    }

    /// Write `value` to a plain-variable out-parameter (e.g. the `$matches` of
    /// `preg_match`). Only bare variables are supported as out-params; any other
    /// expression is silently ignored (step 27 scope-out).
    fn write_out_param(&mut self, target: &Expr, value: Zval) {
        if let ExprKind::Var(slot) = &target.kind {
            self.slot_set(*slot as usize, value);
        }
    }

    /// Evaluate an optional `preg_*` flags argument to an int (0 when absent).
    fn preg_flags(&mut self, arg: Option<&Expr>) -> Result<i64, PhpError> {
        match arg {
            Some(e) => Ok(convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags)),
            None => Ok(0),
        }
    }

    /// `sscanf($string, $format, ...&$vars)` (step 54a). Without output vars it
    /// returns the array of parsed values (NULL for unmatched conversions); with
    /// `&$var` args it assigns each conversion to its variable and returns the
    /// count of successful conversions (by-ref → higher-order, like preg_match;
    /// non-`$var` targets are silently skipped, D-54.1).
    fn ho_sscanf(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "sscanf() expects at least 2 arguments".to_string(),
            ));
        }
        let input = convert::to_zstr(&self.eval(&args[0])?.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        let fmt = convert::to_zstr(&self.eval(&args[1])?.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        let results = crate::scanf::run_scanf(&input, &fmt);
        self.scanf_finish(results, &args[2..])
    }

    /// Shared tail for `sscanf`/`fscanf`: turn the engine's per-conversion slots
    /// into either a return array (no out vars) or by-reference assignments
    /// returning the successful-conversion count.
    fn scanf_finish(&mut self, results: Vec<Option<Zval>>, out: &[Expr]) -> Result<Zval, PhpError> {
        if out.is_empty() {
            let mut arr = PhpArray::new();
            for v in results {
                let _ = arr.append(v.unwrap_or(Zval::Null));
            }
            return Ok(Zval::Array(Rc::new(arr)));
        }
        let mut count = 0i64;
        for (i, slot) in results.iter().enumerate() {
            let Some(target) = out.get(i) else { break };
            match slot {
                Some(v) => {
                    count += 1;
                    self.write_out_param(target, v.clone());
                }
                None => self.write_out_param(target, Zval::Null),
            }
        }
        Ok(Zval::Long(count))
    }

    /// `fscanf($stream, $format, ...&$vars)` (step 54b): read one line from the
    /// stream and scan it like `sscanf`. Returns `false` at end-of-file (so
    /// `while ($r = fscanf(...))` terminates); otherwise an array or the by-ref
    /// conversion count.
    fn ho_fscanf(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "fscanf() expects at least 2 arguments".to_string(),
            ));
        }
        let stream_v = self.eval(&args[0])?.deref_clone();
        let line = match &stream_v {
            Zval::Resource(r) => {
                let mut res = r.borrow_mut();
                match res.as_stream_mut() {
                    Some(s) => match s.read_line(None) {
                        Ok(Some(l)) => l,
                        _ => return Ok(Zval::Bool(false)), // EOF or read error
                    },
                    None => {
                        return Err(PhpError::TypeError(
                            "fscanf(): Argument #1 ($stream) must be an open stream resource"
                                .to_string(),
                        ))
                    }
                }
            }
            other => {
                return Err(PhpError::TypeError(format!(
                    "fscanf(): Argument #1 ($stream) must be of type resource, {} given",
                    other.error_type_name()
                )))
            }
        };
        let fmt = convert::to_zstr(&self.eval(&args[1])?.deref_clone(), &mut self.diags)
            .as_bytes()
            .to_vec();
        let results = crate::scanf::run_scanf(&line, &fmt);
        self.scanf_finish(results, &args[2..])
    }

    /// `preg_match($pattern, $subject, &$matches = null)` (step 27): returns 1 on
    /// a match, 0 on none, `false` on a bad pattern. `$matches[0]` is the whole
    /// match, `$matches[n]` the n-th group (numeric groups only).
    fn ho_preg_match(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let flags = self.preg_flags(args.get(3))?;
        let subj = String::from_utf8_lossy(&subject);
        let (ret, matches) = match re.captures(&subj) {
            Some(caps) => (1, captures_array(&re, &caps, flags)),
            None => (0, Zval::Array(Rc::new(PhpArray::new()))),
        };
        if let Some(out) = args.get(2) {
            self.write_out_param(out, matches);
        }
        Ok(Zval::Long(ret))
    }

    /// `preg_match_all($pattern, $subject, &$matches = null)` (step 27): default
    /// PREG_PATTERN_ORDER — `$matches[g]` is the array of group `g`'s text across
    /// all matches. Returns the match count, or `false` on a bad pattern.
    fn ho_preg_match_all(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_match_all() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let flags = self.preg_flags(args.get(3))?;
        let subj = String::from_utf8_lossy(&subject);
        let offset = flags & PREG_OFFSET_CAPTURE != 0;
        let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
        let mut count: i64 = 0;

        let outer = if flags & PREG_SET_ORDER != 0 {
            // One entry per match, each a full $matches array.
            let mut outer = PhpArray::new();
            for caps in re.captures_iter(&subj) {
                count += 1;
                let _ = outer.append(captures_array(&re, &caps, flags));
            }
            outer
        } else {
            // PREG_PATTERN_ORDER: one column per group (with named keys), each
            // the array of that group's value across all matches.
            let ngroups = re.captures_len();
            let names = re.capture_names();
            let mut cols: Vec<PhpArray> = (0..ngroups).map(|_| PhpArray::new()).collect();
            for caps in re.captures_iter(&subj) {
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
        if let Some(out) = args.get(2) {
            self.write_out_param(out, Zval::Array(Rc::new(outer)));
        }
        Ok(Zval::Long(count))
    }

    /// `preg_replace($pattern, $replacement, $subject)` (step 27): backreferences
    /// `$1` / `${1}` / `\1` in the replacement are honoured. Returns `null` on a
    /// bad pattern. Array patterns/subjects are a scope-out.
    fn ho_preg_replace(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let repl = self.preg_str(&args[1])?;
        let subject = self.preg_str(&args[2])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let repl = String::from_utf8_lossy(&crate::preg::translate_replacement(&repl)).into_owned();
        let subj = String::from_utf8_lossy(&subject);
        let result = re.replace_all(&subj, repl.as_str());
        Ok(Zval::Str(PhpStr::new(result.as_bytes().to_vec())))
    }

    /// `preg_replace_callback($pattern, $callback, $subject)` (step 27): the
    /// callback receives the match array and returns each replacement.
    fn ho_preg_replace_callback(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(
                "preg_replace_callback() expects at least 3 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let callback = self.eval(&args[1])?.deref_clone();
        let subject = self.preg_str(&args[2])?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Null);
        };
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let bytes = subj.as_bytes();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        // Collect (range, match-array) first so the regex borrow of `subj` ends
        // before we call back into the evaluator.
        let hits: Vec<(usize, usize, Zval)> = re
            .captures_iter(&subj)
            .into_iter()
            .map(|caps| {
                let m0 = caps.get(0).unwrap();
                (m0.start, m0.end, captures_array(&re, &caps, 0))
            })
            .collect();
        for (start, end, match_arr) in hits {
            out.extend_from_slice(&bytes[last..start]);
            let ret = self.call_value(callback.clone(), vec![match_arr])?;
            let rs = self.stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&bytes[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `preg_split($pattern, $subject)` (step 27): split on matches. Returns
    /// `false` on a bad pattern.
    fn ho_preg_split(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(
                "preg_split() expects at least 2 arguments".to_string(),
            ));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let limit = match args.get(2) {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
            None => -1,
        };
        let flags = self.preg_flags(args.get(3))?;
        let Some(re) = crate::preg::compile(&pat) else {
            return Ok(Zval::Bool(false));
        };
        let no_empty = flags & 1 != 0;
        let delim_capture = flags & 2 != 0;
        let offset_capture = flags & 4 != 0;
        let subj = String::from_utf8_lossy(&subject).into_owned();
        let mut arr = PhpArray::new();
        let mut last = 0usize;
        // A positive limit caps the piece count; the last piece keeps the rest.
        let push = |arr: &mut PhpArray, text: &str, off: usize| {
            if no_empty && text.is_empty() {
                return;
            }
            if offset_capture {
                let _ = arr.append(offset_pair(
                    Zval::Str(PhpStr::new(text.as_bytes().to_vec())),
                    off as i64,
                ));
            } else {
                let _ = arr.append(Zval::Str(PhpStr::new(text.as_bytes().to_vec())));
            }
        };
        for (idx, caps) in re.captures_iter(&subj).into_iter().enumerate() {
            let m0 = caps.get(0).unwrap();
            if limit > 0 && idx as i64 + 1 >= limit {
                break;
            }
            push(&mut arr, &subj[last..m0.start], last);
            if delim_capture {
                for g in 1..caps.len() {
                    if let Some(mm) = caps.get(g) {
                        push(&mut arr, mm.text.as_str(), mm.start);
                    }
                }
            }
            last = m0.end;
        }
        push(&mut arr, &subj[last..], last);
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `preg_quote($str, $delimiter = null)` (step 27).
    /// `strtok(string $string, string $token)` / `strtok(string $token)` (step 65).
    ///
    /// Stateful tokenizer: the two-arg form (re)sets the persistent cursor, the
    /// one-arg form resumes it. Faithful port of `PHP_FUNCTION(strtok)`
    /// (ext/standard/string.c): leading delimiters are skipped, the token runs up
    /// to the next delimiter, and the cursor is cleared once the string is spent.
    fn ho_strtok(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.is_empty() {
            return Err(PhpError::ArgumentCountError(
                "strtok() expects at least 1 argument, 0 given".to_string(),
            ));
        }
        if args.len() > 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "strtok() expects at most 2 arguments, {} given",
                args.len()
            )));
        }

        let tok: Vec<u8> = if args.len() == 2 {
            let s = convert::to_zstr(&self.eval(&args[0])?.deref_clone(), &mut self.diags)
                .as_bytes()
                .to_vec();
            let t = convert::to_zstr(&self.eval(&args[1])?.deref_clone(), &mut self.diags)
                .as_bytes()
                .to_vec();
            self.strtok_state = Some((s, 0));
            t
        } else {
            convert::to_zstr(&self.eval(&args[0])?.deref_clone(), &mut self.diags)
                .as_bytes()
                .to_vec()
        };

        // The string to tokenize must have been set by an earlier two-arg call.
        let mut state = match self.strtok_state.take() {
            Some(st) => st,
            None => {
                self.diags.push(Diag::Warning(
                    "strtok(): Both arguments must be provided when starting tokenization"
                        .to_string(),
                ));
                return Ok(Zval::Bool(false));
            }
        };

        let pe = state.0.len();
        let last = state.1;
        if last >= pe {
            // Reached the end; PHP returns false without clearing the string.
            self.strtok_state = Some(state);
            return Ok(Zval::Bool(false));
        }

        let mut is_delim = [false; 256];
        for &b in &tok {
            is_delim[b as usize] = true;
        }

        let s = &state.0;
        let mut p = last;
        let mut skipped = 0usize;
        // Skip leading delimiters; exhausting the string here clears the cursor.
        while is_delim[s[p] as usize] {
            p += 1;
            if p >= pe {
                return Ok(Zval::Bool(false)); // state already taken (cleared)
            }
            skipped += 1;
        }
        // Advance to the next delimiter (or the end of the string).
        loop {
            p += 1;
            if p >= pe || is_delim[s[p] as usize] {
                break;
            }
        }
        let token = s[last + skipped..p].to_vec();
        state.1 = p + 1;
        self.strtok_state = Some(state);
        Ok(Zval::Str(PhpStr::new(token)))
    }

    fn ho_preg_quote(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "preg_quote() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let s = self.preg_str(first)?;
        let delim = match args.get(1) {
            Some(e) => self.preg_str(e)?.first().copied(),
            None => None,
        };
        Ok(Zval::Str(PhpStr::new(crate::preg::quote(&s, delim))))
    }

    // --- mbstring regex family (step 43), backed by oniguruma via `mbregex`. ---

    /// Compile a pattern under `opts` against the current mbregex dialect,
    /// emitting PHP's `mbregex compile err:` warning and returning `None` on a
    /// compile error.
    fn mb_compile(&mut self, pat: &[u8], opts: &[u8], func: &str, ic: bool) -> Option<onig::Regex> {
        match crate::mbregex::compile(pat, opts, ic) {
            Ok(re) => Some(re),
            Err(msg) => {
                self.diags
                    .push(Diag::Warning(format!("{func}(): mbregex compile err: {msg}")));
                None
            }
        }
    }

    /// Resolve an optional `$options` argument (index `idx`) to an option string:
    /// the argument when present and non-null, else the global mbregex options.
    fn mb_opts_arg(&mut self, args: &[Expr], idx: usize) -> Result<Vec<u8>, PhpError> {
        match args.get(idx) {
            None => Ok(self.mb_regex.options.clone()),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if matches!(v, Zval::Null) {
                    Ok(self.mb_regex.options.clone())
                } else {
                    Ok(self.stringify(&v)?.as_bytes().to_vec())
                }
            }
        }
    }

    /// `mb_ereg($pattern, $string, &$regs = null)` / `mb_eregi` (case-insensitive):
    /// returns a bool (PHP 8). `$regs[0]` is the whole match, `$regs[n]` the n-th
    /// group (a non-participating group is `false`), with named groups appended
    /// by string key. On no match `$regs` is set to an empty array.
    fn ho_mb_ereg(&mut self, args: &[Expr], ic: bool) -> Result<Zval, PhpError> {
        let func = if ic { "mb_eregi" } else { "mb_ereg" };
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let opts = self.mb_regex.options.clone();
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            return Ok(Zval::Bool(false));
        };
        let regs = crate::mbregex::exec(&re, &subject);
        let matched = regs.is_some();
        if let Some(out) = args.get(2) {
            self.write_out_param(out, regs.unwrap_or_else(|| Zval::Array(Rc::new(PhpArray::new()))));
        }
        Ok(Zval::Bool(matched))
    }

    /// `mb_ereg_replace($pattern, $replacement, $string[, $options])` / the `i`
    /// variant. Backreferences `\0`..`\9` in the replacement are honoured.
    /// Returns `false` on a bad pattern.
    fn ho_mb_ereg_replace(&mut self, args: &[Expr], ic: bool) -> Result<Zval, PhpError> {
        let func = if ic { "mb_eregi_replace" } else { "mb_ereg_replace" };
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "{func}() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let repl = self.preg_str(&args[1])?;
        let subject = self.preg_str(&args[2])?;
        let opts = self.mb_opts_arg(args, 3)?;
        let Some(re) = self.mb_compile(&pat, &opts, func, ic) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Str(PhpStr::new(crate::mbregex::replace(&re, &repl, &subject))))
    }

    /// `mb_ereg_replace_callback($pattern, $callback, $string[, $options])`: the
    /// callback receives each match's `$regs` array and returns its replacement.
    fn ho_mb_ereg_replace_callback(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 3 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_replace_callback() expects at least 3 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let callback = self.eval(&args[1])?.deref_clone();
        let subject = self.preg_str(&args[2])?;
        let opts = self.mb_opts_arg(args, 3)?;
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_replace_callback", false) else {
            return Ok(Zval::Bool(false));
        };
        let bytes = subject.clone();
        let mut out: Vec<u8> = Vec::new();
        let mut last = 0usize;
        for (start, end, regs) in crate::mbregex::find_all(&re, &subject) {
            out.extend_from_slice(&bytes[last..start]);
            let ret = self.call_value(callback.clone(), vec![regs])?;
            let rs = self.stringify(&ret.deref_clone())?;
            out.extend_from_slice(rs.as_bytes());
            last = end;
        }
        out.extend_from_slice(&bytes[last..]);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `mb_split($pattern, $string[, $limit])`: split on matches, keeping empty
    /// fields. `$limit > 0` caps the piece count. Returns `false` on a bad pattern.
    fn ho_mb_split(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_split() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let limit = match args.get(2) {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
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

    /// `mb_ereg_match($pattern, $string[, $options])`: whether the pattern matches
    /// anchored at the start of `$string` (a prefix match, not a full match).
    fn ho_mb_ereg_match(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "mb_ereg_match() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let pat = self.preg_str(&args[0])?;
        let subject = self.preg_str(&args[1])?;
        let opts = self.mb_opts_arg(args, 2)?;
        let Some(re) = self.mb_compile(&pat, &opts, "mb_ereg_match", false) else {
            return Ok(Zval::Bool(false));
        };
        Ok(Zval::Bool(crate::mbregex::matches_at_start(&re, &subject)))
    }

    /// `mb_regex_encoding([$encoding])`: getter returns the current name ("UTF-8"
    /// default); setter stores it and returns true. Only UTF-8 is effectively
    /// supported (D-MB-ereg-enc).
    fn ho_mb_regex_encoding(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone()))),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if matches!(v, Zval::Null) {
                    return Ok(Zval::Str(PhpStr::new(self.mb_regex.encoding.clone())));
                }
                self.mb_regex.encoding = self.stringify(&v)?.as_bytes().to_vec();
                Ok(Zval::Bool(true))
            }
        }
    }

    /// `mb_regex_set_options([$options])`: getter returns the current options
    /// ("pr" default); setter stores them and returns the previous options.
    fn ho_mb_regex_set_options(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let prev = self.mb_regex.options.clone();
        match args.first() {
            None => Ok(Zval::Str(PhpStr::new(prev))),
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                if !matches!(v, Zval::Null) {
                    self.mb_regex.options = self.stringify(&v)?.as_bytes().to_vec();
                }
                Ok(Zval::Str(PhpStr::new(prev)))
            }
        }
    }

    // --- mbregex stateful search cursor (step 43b) ---

    /// Compile and store the search pattern from an optional `$pattern` argument
    /// at `idx` (its `$options` follow at `idx + 1`); keeps the existing compiled
    /// pattern when absent/null. Returns false on a compile error.
    fn mb_search_set_pattern(&mut self, args: &[Expr], idx: usize) -> Result<bool, PhpError> {
        if let Some(p) = args.get(idx) {
            let pv = self.eval(p)?.deref_clone();
            if !matches!(pv, Zval::Null) {
                let pat = self.stringify(&pv)?.as_bytes().to_vec();
                let opts = self.mb_opts_arg(args, idx + 1)?;
                match self.mb_compile(&pat, &opts, "mb_ereg_search", false) {
                    Some(re) => self.mb_regex.search_re = Some(re),
                    None => return Ok(false),
                }
            }
        }
        Ok(true)
    }

    /// Run the next search from the cursor, advancing it past the match (by one
    /// byte for a zero-width match) and recording the result for `getregs`.
    fn mb_search_step(&mut self) -> Option<(usize, usize, Zval)> {
        let re = self.mb_regex.search_re.take()?;
        let subject = std::mem::take(&mut self.mb_regex.search_str);
        let res = crate::mbregex::search_from(&re, &subject, self.mb_regex.search_pos);
        self.mb_regex.search_re = Some(re);
        self.mb_regex.search_str = subject;
        if let Some((start, end, regs)) = &res {
            self.mb_regex.search_pos = if end > start { *end } else { *end + 1 };
            self.mb_regex.last_regs = Some(regs.clone());
        }
        res
    }

    /// `mb_ereg_search_init($string[, $pattern[, $options]])`: start a stateful
    /// search over `$string`, resetting the cursor. Returns false on a bad pattern.
    fn ho_mb_ereg_search_init(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "mb_ereg_search_init() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        self.mb_regex.search_str = self.preg_str(first)?;
        self.mb_regex.search_pos = 0;
        self.mb_regex.last_regs = None;
        if !self.mb_search_set_pattern(args, 1)? {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(true))
    }

    /// `mb_ereg_search([$pattern[, $options]])`: advance the cursor to the next
    /// match; returns whether one was found.
    fn ho_mb_ereg_search(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
            return Ok(Zval::Bool(false));
        }
        Ok(Zval::Bool(self.mb_search_step().is_some()))
    }

    /// `mb_ereg_search_pos([$pattern[, $options]])`: next match as `[pos, len]`
    /// byte offsets, or false at the end.
    fn ho_mb_ereg_search_pos(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
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

    /// `mb_ereg_search_regs([$pattern[, $options]])`: next match's `$regs` array,
    /// or false at the end.
    fn ho_mb_ereg_search_regs(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if !self.mb_search_set_pattern(args, 0)? {
            return Ok(Zval::Bool(false));
        }
        match self.mb_search_step() {
            Some((_, _, regs)) => Ok(regs),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `mb_ereg_search_getregs()`: the `$regs` of the last successful search, or
    /// false if none has succeeded.
    fn ho_mb_ereg_search_getregs(&mut self) -> Result<Zval, PhpError> {
        Ok(self.mb_regex.last_regs.clone().unwrap_or(Zval::Bool(false)))
    }

    /// `mb_ereg_search_setpos($position)`: move the byte cursor; false if out of
    /// range.
    fn ho_mb_ereg_search_setpos(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let pos = match args.first() {
            Some(e) => convert::to_long_cast(&self.eval(e)?.deref_clone(), &mut self.diags),
            None => 0,
        };
        if pos < 0 || pos as usize > self.mb_regex.search_str.len() {
            return Ok(Zval::Bool(false));
        }
        self.mb_regex.search_pos = pos as usize;
        Ok(Zval::Bool(true))
    }

    /// `json_decode($json, $assoc = false, ...)` (step 26). Intercepted here
    /// because the default mode builds `stdClass` objects, which needs the class
    /// table. Returns `null` on a parse error (JSON_THROW_ON_ERROR is a
    /// scope-out). Objects become arrays when `$assoc` is true, `stdClass`
    /// otherwise; the `depth`/`flags` arguments are ignored.
    fn ho_json_decode(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "json_decode() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arg0 = self.eval(first)?.deref_clone();
        let json = self.stringify(&arg0)?;
        let assoc = match args.get(1) {
            Some(e) => {
                let v = self.eval(e)?.deref_clone();
                convert::to_bool(&v, &mut self.diags)
            }
            None => false,
        };
        match crate::json::parse(json.as_bytes()) {
            Some(j) => Ok(self.json_to_zval(&j, assoc)),
            None => Ok(Zval::Null),
        }
    }

    /// Convert a parsed [`crate::json::Json`] tree into a `Zval` (step 26).
    fn json_to_zval(&mut self, j: &crate::json::Json, assoc: bool) -> Zval {
        use crate::json::Json;
        match j {
            Json::Null => Zval::Null,
            Json::Bool(b) => Zval::Bool(*b),
            Json::Long(n) => Zval::Long(*n),
            Json::Double(d) => Zval::Double(*d),
            Json::Str(s) => Zval::Str(PhpStr::new(s.clone())),
            Json::Array(items) => {
                let mut arr = PhpArray::new();
                for item in items {
                    let v = self.json_to_zval(item, assoc);
                    let _ = arr.append(v);
                }
                Zval::Array(Rc::new(arr))
            }
            Json::Object(entries) => {
                let fields: Vec<(Vec<u8>, Zval)> = entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.json_to_zval(v, assoc)))
                    .collect();
                if assoc {
                    let mut arr = PhpArray::new();
                    for (k, v) in fields {
                        arr.insert(Key::from_bytes(&k), v);
                    }
                    Zval::Array(Rc::new(arr))
                } else {
                    self.make_stdclass(fields)
                }
            }
        }
    }

    /// Build a fresh `stdClass` with the given dynamic properties (step 26).
    /// `(object)$v`: arrays map each entry to a property (key stringified),
    /// objects pass through unchanged, null yields an empty stdClass, and any
    /// scalar becomes a single `scalar` property.
    pub(super) fn object_cast(&mut self, v: Zval) -> Zval {
        match v {
            Zval::Object(_) => v,
            Zval::Array(a) => {
                let fields = a
                    .iter()
                    .map(|(k, val)| {
                        let name = match k {
                            Key::Int(i) => i.to_string().into_bytes(),
                            Key::Str(s) => s.as_bytes().to_vec(),
                        };
                        (name, val.clone())
                    })
                    .collect();
                self.make_stdclass(fields)
            }
            Zval::Null => self.make_stdclass(Vec::new()),
            scalar => self.make_stdclass(vec![(b"scalar".to_vec(), scalar)]),
        }
    }

    fn make_stdclass(&mut self, fields: Vec<(Vec<u8>, Zval)>) -> Zval {
        let cid = self.class_index[b"stdclass".as_slice()];
        let class_name = PhpStr::new(self.classes[cid].name.to_vec());
        let mut props = Props::new();
        for (k, v) in fields {
            props.set(&k, v);
        }
        let info = self.class_shape(cid);
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info };
        let value = Zval::Object(Rc::new(RefCell::new(obj)));
        if let Zval::Object(o) = &value {
            self.created.push(o.clone());
        }
        value
    }

    /// Build an object of a named class with the given properties (step 50b,
    /// `unserialize`). A known class is instantiated with its real class id and
    /// shape, the properties set directly (the constructor is **not** run, as in
    /// PHP). An unknown class falls back to `stdClass` (D-50 scope-out: PHP makes
    /// a `__PHP_Incomplete_Class`); `__wakeup` is not called (D-50).
    fn make_object(&mut self, class: &[u8], fields: Vec<(Vec<u8>, Zval)>) -> Zval {
        let cid = match self.class_index.get(&class.to_ascii_lowercase()) {
            Some(&c) => c,
            None => return self.make_stdclass(fields),
        };
        let class_name = PhpStr::new(self.classes[cid].name.to_vec());
        let mut props = Props::new();
        for (k, v) in fields {
            props.set(&k, v);
        }
        let info = self.class_shape(cid);
        let id = self.next_id();
        let obj = Object { class_id: cid as u32, class_name, props, id, info };
        let value = Zval::Object(Rc::new(RefCell::new(obj)));
        if let Zval::Object(o) = &value {
            self.created.push(o.clone());
        }
        value
    }

    /// `unserialize($str)` (step 50b). Intercepted here because rebuilding an
    /// object needs the class table / id allocator. A malformed string yields
    /// `false` with PHP's notice, like the engine.
    fn ho_unserialize(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "unserialize() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arg0 = self.eval(first)?.deref_clone();
        let bytes = self.stringify(&arg0)?;
        match crate::unserialize::parse(bytes.as_bytes()) {
            Some(s) => Ok(self.ser_to_zval(s)),
            None => {
                // PHP reports a Warning with the byte length; we do not track the
                // precise failing offset, so report 0 (D-50).
                self.diags.push(Diag::Warning(format!(
                    "unserialize(): Error at offset 0 of {} bytes",
                    bytes.as_bytes().len()
                )));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `fopen($filename, $mode, …)` (step 51, evaluator-dispatched because it
    /// mints a resource id, D-51.3). 51a opens **real files** only; the
    /// `php://` wrappers land in 51b. On failure: Warning + `false`.
    fn ho_fopen(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(path_e) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "fopen() expects at least 2 arguments, 0 given".to_string(),
            ));
        };
        let Some(mode_e) = args.get(1) else {
            return Err(PhpError::ArgumentCountError(
                "fopen() expects at least 2 arguments, 1 given".to_string(),
            ));
        };
        let path_v = self.eval(path_e)?.deref_clone();
        let path = self.stringify(&path_v)?.as_bytes().to_vec();
        let mode_v = self.eval(mode_e)?.deref_clone();
        let mode = self.stringify(&mode_v)?.as_bytes().to_vec();
        // Args 3/4 (use_include_path, context) are a documented scope-out.
        if let Some(spec) = path.strip_prefix(b"php://".as_slice()) {
            return match open_php_stream(spec, &mode) {
                Some(stream) => Ok(self.alloc_resource(stream)),
                None => {
                    self.diags.push(Diag::Warning(format!(
                        "fopen({}): Failed to open stream: no suitable wrapper could be found",
                        String::from_utf8_lossy(&path)
                    )));
                    Ok(Zval::Bool(false))
                }
            };
        }
        match open_file_stream(&path, &mode) {
            Ok(stream) => Ok(self.alloc_resource(stream)),
            Err(msg) => {
                self.diags.push(Diag::Warning(format!(
                    "fopen({}): Failed to open stream: {msg}",
                    String::from_utf8_lossy(&path)
                )));
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `tmpfile()` (step 52e, evaluator-dispatched — mints a resource). Creates a
    /// fresh file under the system temp dir opened read+write, then immediately
    /// unlinks it so the OS reclaims it on close / process exit (PHP's
    /// auto-removal semantics; the path is not observable). `false` on failure.
    fn ho_tmpfile(&mut self, _args: &[Expr]) -> Result<Zval, PhpError> {
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
                    let _ = std::fs::remove_file(&path);
                    let stream = Stream {
                        backend: StreamBackend::File(f),
                        readable: true,
                        writable: true,
                        eof: false,
                    };
                    return Ok(self.alloc_resource(stream));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Ok(Zval::Bool(false)),
            }
        }
        Ok(Zval::Bool(false))
    }

    /// `opendir($directory)` (step 53c, evaluator-dispatched — mints a resource).
    /// Snapshots the directory entries (`.`/`..` first, then OS order) into a
    /// `DirHandle`; `false` + Warning if the directory can't be opened.
    fn ho_opendir(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        use std::os::unix::ffi::OsStrExt;
        let Some(path_e) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "opendir() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let path_v = self.eval(path_e)?.deref_clone();
        let path = self.stringify(&path_v)?.as_bytes().to_vec();
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

    /// Wrap a freshly opened stream in a `Zval::Resource` with the next id.
    fn alloc_resource(&mut self, stream: Stream) -> Zval {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        Zval::Resource(Rc::new(RefCell::new(Resource::new(id, stream))))
    }

    /// Turn a parsed [`crate::unserialize::Ser`] tree into a `Zval`.
    fn ser_to_zval(&mut self, s: crate::unserialize::Ser) -> Zval {
        use crate::unserialize::Ser;
        match s {
            Ser::Null => Zval::Null,
            Ser::Bool(b) => Zval::Bool(b),
            Ser::Long(n) => Zval::Long(n),
            Ser::Double(d) => Zval::Double(d),
            Ser::Str(bytes) => Zval::Str(PhpStr::new(bytes)),
            Ser::Array(items) => {
                let mut arr = PhpArray::new();
                for (k, v) in items {
                    let key = match k {
                        Ser::Long(i) => Key::Int(i),
                        // String keys coerce to int when canonically numeric,
                        // matching PHP array semantics.
                        Ser::Str(b) => Key::from_bytes(&b),
                        _ => continue,
                    };
                    let val = self.ser_to_zval(v);
                    arr.insert(key, val);
                }
                Zval::Array(Rc::new(arr))
            }
            Ser::Object(class, props) => {
                let fields: Vec<(Vec<u8>, Zval)> = props
                    .into_iter()
                    .map(|(name, v)| (name, self.ser_to_zval(v)))
                    .collect();
                self.make_object(&class, fields)
            }
        }
    }

    /// Class-introspection builtins the evaluator answers directly because they
    /// read the current object / class table rather than a pure value (step 20
    /// coda). Returns `None` for a name we do not intercept.
    pub(super) fn dispatch_class_introspection(
        &mut self,
        name: &[u8],
        args: &[Expr],
    ) -> Option<Result<Zval, PhpError>> {
        match name {
            b"get_class" => Some(self.ci_get_class(args)),
            b"get_parent_class" => Some(self.ci_get_parent_class(args)),
            b"get_class_methods" => Some(self.ci_get_class_methods(args)),
            b"get_object_vars" => Some(self.ci_get_object_vars(args)),
            _ => None,
        }
    }

    /// `get_class($object)` — the object's class name; with no argument, the
    /// class of the current `$this` (a fatal Error outside object context, PHP 8).
    fn ci_get_class(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let v = match args.first() {
            Some(e) => self.eval(e)?.deref_clone(),
            None => match &self.cur_this {
                Some(t) => t.clone(),
                None => {
                    return Err(PhpError::Error(
                        "get_class() without arguments must be called from within a class"
                            .to_string(),
                    ))
                }
            },
        };
        match &v {
            Zval::Object(o) => Ok(Zval::Str(PhpStr::new(o.borrow().class_name.as_bytes().to_vec()))),
            Zval::Closure(_) => Ok(Zval::Str(PhpStr::new(b"Closure".to_vec()))),
            other => Err(PhpError::TypeError(format!(
                "get_class(): Argument #1 ($object) must be of type object, {} given",
                other.error_type_name()
            ))),
        }
    }

    /// `get_parent_class([$object|$class])` — the parent class name, or `false`
    /// when there is none (or the target cannot be resolved). With no argument it
    /// uses the current class context.
    fn ci_get_parent_class(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let cid: Option<ClassId> = match args.first() {
            Some(e) => match self.eval(e)?.deref_clone() {
                Zval::Object(o) => Some(o.borrow().class_id as usize),
                Zval::Str(s) => self
                    .class_index
                    .get(&s.as_bytes().to_ascii_lowercase())
                    .copied(),
                _ => None,
            },
            None => self.cur_class,
        };
        match cid.and_then(|c| self.classes[c].parent) {
            Some(p) => Ok(Zval::Str(PhpStr::new(self.classes[p].name.to_vec()))),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `get_class_methods($objectOrClassName)` (step 47): the method names of the
    /// class, walking the inheritance chain child→parent (each method once;
    /// child overrides win), filtered by visibility from the calling scope
    /// (`visible_from`) — so from outside only `public` methods are returned, and
    /// from within the class the `protected`/`private` ones too.
    fn ci_get_class_methods(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let cid: Option<ClassId> = match args.first() {
            Some(e) => match self.eval(e)?.deref_clone() {
                Zval::Object(o) => Some(o.borrow().class_id as usize),
                Zval::Str(s) => self
                    .class_index
                    .get(&s.as_bytes().to_ascii_lowercase())
                    .copied(),
                _ => None,
            },
            None => {
                return Err(PhpError::ArgumentCountError(
                    "get_class_methods() expects exactly 1 argument, 0 given".to_string(),
                ))
            }
        };
        // An unresolved class name yields `null` (PHP raises a TypeError only for
        // a non-string/non-object argument, which we mapped to `None` above).
        let Some(start) = cid else {
            return Ok(Zval::Null);
        };
        let classes: &'p [ClassDecl] = self.classes;
        let mut arr = PhpArray::new();
        let mut seen: Vec<Vec<u8>> = Vec::new();
        let mut cur = Some(start);
        while let Some(c) = cur {
            for m in &classes[c].methods {
                let lname = m.decl.name.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue; // a more-derived class already defined this name
                }
                // Mark the name as resolved by this (most-derived) class even
                // when it is not visible, so a parent's same-named method (or an
                // overridden abstract signature) does not leak into the result.
                seen.push(lname);
                if self.visible_from(m.visibility, c) {
                    let _ = arr.append(Zval::Str(PhpStr::new(m.decl.name.to_vec())));
                }
            }
            // Abstract signatures (interface / `abstract` methods). Interface
            // methods are public; a protected `abstract` method that is never
            // overridden and queried from outside is a minor scope-out (D-47.1).
            for am in &classes[c].abstract_methods {
                let lname = am.to_ascii_lowercase();
                if seen.contains(&lname) {
                    continue;
                }
                seen.push(lname);
                let _ = arr.append(Zval::Str(PhpStr::new(am.to_vec())));
            }
            cur = classes[c].parent;
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// `get_object_vars($object)` (step 47): the object's properties as a
    /// `name => value` array, filtered by visibility from the calling scope —
    /// from outside only `public` properties, from within the class the
    /// `protected`/`private` ones too. Dynamic (undeclared) properties are
    /// always public. Declaration / insertion order is preserved.
    fn ci_get_object_vars(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let v = match args.first() {
            Some(e) => self.eval(e)?.deref_clone(),
            None => {
                return Err(PhpError::ArgumentCountError(
                    "get_object_vars() expects exactly 1 argument, 0 given".to_string(),
                ))
            }
        };
        let Zval::Object(o) = v else {
            return Err(PhpError::TypeError(format!(
                "get_object_vars(): Argument #1 ($object) must be of type object, {} given",
                v.error_type_name()
            )));
        };
        let obj = o.borrow();
        let cid = obj.class_id as usize;
        let mut arr = PhpArray::new();
        for (name, val) in obj.props.iter() {
            let visible = match self.resolve_prop_decl(cid, name) {
                Some((vis, decl_class)) => self.visible_from(vis, decl_class),
                None => true, // dynamic / undeclared property is public
            };
            if visible {
                arr.insert(Key::from_bytes(name), val.clone());
            }
        }
        Ok(Zval::Array(Rc::new(arr)))
    }

    /// Whether a function *name* resolves to something callable: a user function,
    /// a registered builtin, or a higher-order builtin the evaluator intercepts.
    fn is_name_callable(&self, name: &[u8]) -> bool {
        self.fn_index.contains_key(&name.to_ascii_lowercase())
            || self.reg.contains_key(name)
            || HIGHER_ORDER_BUILTINS.contains(&name)
    }

    /// `is_callable($value)` (step 18). Closures are callable; a string is
    /// callable iff it names a function; everything else (arrays/OOP callables
    /// are a scope-out) is not.
    fn ho_is_callable(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "is_callable() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let v = self.eval(first)?.deref_clone();
        let callable = match &v {
            Zval::Closure(_) => true,
            Zval::Str(s) => self.is_name_callable(s.as_bytes()),
            // An object is callable iff it defines `__invoke` (step 22, D-22.7).
            Zval::Object(o) => {
                let cid = o.borrow().class_id as usize;
                self.resolve_method(cid, b"__invoke").is_some()
            }
            _ => false,
        };
        Ok(Zval::Bool(callable))
    }

    /// `call_user_func($callable, ...$args)` (step 18): the remaining arguments
    /// are forwarded by value to the callable.
    fn ho_call_user_func(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some((callee_expr, rest)) = args.split_first() else {
            return Err(PhpError::ArgumentCountError(
                "call_user_func() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let callee = self.eval(callee_expr)?.deref_clone();
        let mut argv = Vec::with_capacity(rest.len());
        for a in rest {
            argv.push(self.eval(a)?);
        }
        self.call_value(callee, argv)
    }

    /// `call_user_func_array($callable, $args)` (step 18): the second argument is
    /// an array whose *values* become the positional arguments (string-keyed
    /// named arguments are a scope-out).
    fn ho_call_user_func_array(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "call_user_func_array() expects exactly 2 arguments, {} given",
                args.len()
            )));
        }
        let callee = self.eval(&args[0])?.deref_clone();
        let arr = self.eval(&args[1])?.deref_clone();
        let argv: Vec<Zval> = match arr {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "call_user_func_array(): Argument #2 ($args) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        self.call_value(callee, argv)
    }

    /// `array_map($callback, ...$arrays)` (step 18, D-18.6). With a single array
    /// the keys are preserved; with several arrays the result is re-indexed and
    /// the callback receives one element from each (missing tails are NULL). A
    /// NULL callback zips the arrays (single array → identity).
    fn ho_array_map(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        if args.len() < 2 {
            return Err(PhpError::ArgumentCountError(format!(
                "array_map() expects at least 2 arguments, {} given",
                args.len()
            )));
        }
        let cb = self.eval(&args[0])?.deref_clone();
        let null_cb = matches!(cb, Zval::Null);
        let mut arrays = Vec::with_capacity(args.len() - 1);
        for (i, a) in args[1..].iter().enumerate() {
            match self.eval(a)?.deref_clone() {
                Zval::Array(arr) => arrays.push(arr),
                other => {
                    return Err(PhpError::TypeError(format!(
                        "array_map(): Argument #{} must be of type array, {} given",
                        i + 2,
                        other.error_type_name()
                    )))
                }
            }
        }

        let mut out = PhpArray::new();
        if arrays.len() == 1 {
            // Single array: preserve keys.
            let entries: Vec<(Key, Zval)> =
                arrays[0].iter().map(|(k, v)| (k.clone(), v.deref_clone())).collect();
            for (k, v) in entries {
                let mapped = if null_cb {
                    v
                } else {
                    self.call_value(cb.clone(), vec![v])?
                };
                out.insert(k, mapped);
            }
        } else {
            // Several arrays: re-index 0..max, one element from each per row.
            let cols: Vec<Vec<Zval>> = arrays
                .iter()
                .map(|a| a.iter().map(|(_, v)| v.deref_clone()).collect())
                .collect();
            let max = cols.iter().map(|c| c.len()).max().unwrap_or(0);
            for i in 0..max {
                let row: Vec<Zval> = cols
                    .iter()
                    .map(|c| c.get(i).cloned().unwrap_or(Zval::Null))
                    .collect();
                let val = if null_cb {
                    let mut tuple = PhpArray::new();
                    for v in row {
                        let _ = tuple.append(v);
                    }
                    Zval::Array(Rc::new(tuple))
                } else {
                    self.call_value(cb.clone(), row)?
                };
                let _ = out.append(val);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `array_filter($array, $callback?, $mode = 0)` (step 18, D-18.6). Keys are
    /// always preserved. With no callback, truthy values are kept; otherwise the
    /// callback receives the value (mode 0), the key (`ARRAY_FILTER_USE_KEY`), or
    /// `(value, key)` (`ARRAY_FILTER_USE_BOTH`).
    fn ho_array_filter(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let Some(first) = args.first() else {
            return Err(PhpError::ArgumentCountError(
                "array_filter() expects at least 1 argument, 0 given".to_string(),
            ));
        };
        let arr = match self.eval(first)?.deref_clone() {
            Zval::Array(a) => a,
            other => {
                return Err(PhpError::TypeError(format!(
                    "array_filter(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };
        let cb = match args.get(1) {
            Some(a) => match self.eval(a)?.deref_clone() {
                Zval::Null => None,
                v => Some(v),
            },
            None => None,
        };
        let mode = match args.get(2) {
            Some(a) => {
                let v = self.eval(a)?.deref_clone();
                convert::to_long_cast(&v, &mut self.diags)
            }
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
                    let r = self.call_value(c.clone(), call_args)?;
                    convert::to_bool(&r, &mut self.diags)
                }
            };
            if keep {
                out.insert(k, v);
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// `usort(&$array, $callback)` (step 18, D-18.6): sort the array's values in
    /// place by the comparator, re-index 0..n, and return `true`. The first
    /// argument is taken by reference (like `sort`); the comparator returns an
    /// int (`$a <=> $b`-style).
    fn ho_usort(&mut self, args: &[Expr]) -> Result<Zval, PhpError> {
        let (Some(arr_expr), Some(cmp_expr)) = (args.first(), args.get(1)) else {
            return Err(PhpError::ArgumentCountError(format!(
                "usort() expects exactly 2 arguments, {} given",
                args.len()
            )));
        };
        let ExprKind::Var(slot) = arr_expr.kind else {
            return Err(PhpError::Error(
                "usort(): Argument #1 ($array) could not be passed by reference".to_string(),
            ));
        };
        let cmp = self.eval(cmp_expr)?.deref_clone();
        let cell = self.slot_cell(slot as usize);
        let values: Vec<Zval> = match &*cell.borrow() {
            Zval::Array(a) => a.iter().map(|(_, v)| v.deref_clone()).collect(),
            other => {
                return Err(PhpError::TypeError(format!(
                    "usort(): Argument #1 ($array) must be of type array, {} given",
                    other.error_type_name()
                )))
            }
        };

        let sorted = self.merge_sort_with(&cmp, values)?;
        let mut out = PhpArray::new();
        for v in sorted {
            let _ = out.append(v);
        }
        *cell.borrow_mut() = Zval::Array(Rc::new(out));
        Ok(Zval::Bool(true))
    }

    /// Stable merge sort driven by a PHP comparator callback (used by `usort`).
    /// Stability matches PHP 8's sort guarantee; the comparator's return value is
    /// cast to an int (`<= 0` keeps the left element first).
    fn merge_sort_with(&mut self, cmp: &Zval, mut vals: Vec<Zval>) -> Result<Vec<Zval>, PhpError> {
        let n = vals.len();
        if n <= 1 {
            return Ok(vals);
        }
        let right = vals.split_off(n / 2);
        let left = self.merge_sort_with(cmp, vals)?;
        let right = self.merge_sort_with(cmp, right)?;
        let mut merged = Vec::with_capacity(n);
        let (mut i, mut j) = (0, 0);
        while i < left.len() && j < right.len() {
            if self.compare_with_callback(cmp, &left[i], &right[j])? <= 0 {
                merged.push(left[i].clone());
                i += 1;
            } else {
                merged.push(right[j].clone());
                j += 1;
            }
        }
        merged.extend_from_slice(&left[i..]);
        merged.extend_from_slice(&right[j..]);
        Ok(merged)
    }

    /// Invoke a sort comparator and reduce its result to an int (step 18).
    fn compare_with_callback(&mut self, cmp: &Zval, a: &Zval, b: &Zval) -> Result<i64, PhpError> {
        let r = self.call_value(cmp.clone(), vec![a.clone(), b.clone()])?;
        Ok(convert::to_long_cast(&r, &mut self.diags))
    }
}
