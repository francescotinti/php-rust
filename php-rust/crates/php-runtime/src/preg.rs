//! PCRE pattern support for the `preg_*` builtins (steps 27, 31, 36).
//!
//! Patterns are PHP's delimited form (`/body/flags`). Compilation uses an
//! auto-fallback engine ([`Engine`], step 36): the fast `regex` crate (RE2-style
//! DFA) is tried first, and on a compile error — backreferences (`\1`),
//! lookaround (`(?<=)`, `(?=)`), atomic groups, possessive quantifiers — the
//! pattern falls back to `fancy-regex` (NFA). The fallback is transparent: no
//! user flag, no configuration. Features neither engine supports (recursion
//! `(?R)`, conditionals `(?(1)..)`, `\K`, `\G`, callouts) fail both compiles and
//! make the `preg_*` function return `false`/`null` — a documented scope-out.

use regex::RegexBuilder;
use std::rc::Rc;

use php_types::{Key, PhpArray, PhpStr, Zval};

/// A single capture group's byte span and text, engine-neutral so the two
/// backends never leak their distinct `Match`/`Captures` lifetimes into the
/// evaluator.
pub struct CapMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Engine-neutral capture set: index 0 is the whole match, 1.. the groups.
/// A `None` entry is a group that did not participate in the match.
pub struct Caps {
    groups: Vec<Option<CapMatch>>,
}

impl Caps {
    /// The `i`-th group, or `None` if absent / non-participating.
    pub fn get(&self, i: usize) -> Option<&CapMatch> {
        self.groups.get(i).and_then(|g| g.as_ref())
    }

    /// Number of capture slots (whole match + groups).
    pub fn len(&self) -> usize {
        self.groups.len()
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
}

fn caps_from_regex(caps: &regex::Captures) -> Caps {
    let groups = (0..caps.len())
        .map(|i| {
            caps.get(i).map(|m| CapMatch {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
        })
        .collect();
    Caps { groups }
}

fn caps_from_fancy(caps: &fancy_regex::Captures) -> Caps {
    let groups = (0..caps.len())
        .map(|i| {
            caps.get(i).map(|m| CapMatch {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
        })
        .collect();
    Caps { groups }
}

fn caps_from_onig(caps: &onig::Captures) -> Caps {
    // `caps.len()` counts all slots (0 = whole match). A non-participating group
    // has no `pos`, so it collapses to `None` — the same convention as the other
    // backends.
    let groups = (0..caps.len())
        .map(|i| {
            caps.pos(i).map(|(start, end)| CapMatch {
                start,
                end,
                text: caps.at(i).unwrap_or("").to_string(),
            })
        })
        .collect();
    Caps { groups }
}

/// Build a `Caps` from an oniguruma `Region` (the offset-search path, where a
/// borrowed `onig::Captures` is not available). Group text is sliced from the
/// original subject by the region's absolute byte spans.
fn caps_from_onig_region(text: &str, region: &onig::Region) -> Caps {
    let groups = (0..region.len())
        .map(|i| {
            region.pos(i).map(|(start, end)| CapMatch {
                start,
                end,
                text: text.get(start..end).unwrap_or("").to_string(),
            })
        })
        .collect();
    Caps { groups }
}

/// A compiled PHP pattern. Tries the fast `regex` engine first, falling back to
/// `fancy-regex` for patterns using PCRE features `regex` cannot build (step 36),
/// then to oniguruma (the `onig` crate) for the deepest PCRE features neither
/// Rust engine supports — subroutine calls (`(?&name)`, `(?R)`), `(?(DEFINE))`
/// blocks, recursion. Oniguruma is PHP's own mbregex backend and reads PCRE
/// syntax under its `perl_ng` dialect. It is the LAST resort: only patterns both
/// Rust engines reject reach it, so no existing behaviour changes.
pub enum Engine {
    Regex(regex::Regex),
    Fancy(fancy_regex::Regex),
    Onig(onig::Regex),
}

impl Engine {
    /// Number of capture slots (whole match + named/numbered groups).
    pub fn captures_len(&self) -> usize {
        match self {
            Engine::Regex(r) => r.captures_len(),
            Engine::Fancy(r) => r.captures_len(),
            // oniguruma counts groups WITHOUT the implicit whole-match slot; the
            // other backends include it, so add one for a common convention.
            Engine::Onig(r) => r.captures_len() + 1,
        }
    }

    /// Capture-group names by index (`None` for unnamed / the whole match),
    /// collected to owned strings so both backends share one return type.
    pub fn capture_names(&self) -> Vec<Option<String>> {
        match self {
            Engine::Regex(r) => r
                .capture_names()
                .map(|n| n.map(|s| s.to_string()))
                .collect(),
            Engine::Fancy(r) => r
                .capture_names()
                .map(|n| n.map(|s| s.to_string()))
                .collect(),
            // oniguruma has no positional name iterator; build the index->name
            // table from `foreach_name` (a name may bind several group numbers,
            // e.g. duplicate `(?<x>..)(?<x>..)`).
            Engine::Onig(r) => {
                let mut names = vec![None; r.captures_len() + 1];
                r.foreach_name(|name, nums| {
                    for &g in nums {
                        if let Some(slot) = names.get_mut(g as usize) {
                            *slot = Some(name.to_string());
                        }
                    }
                    true
                });
                names
            }
        }
    }

    /// First match in `text`, engine-neutral. A `fancy-regex` runtime error
    /// (e.g. backtrack limit) collapses to "no match" (D-36.3).
    pub fn captures(&self, text: &str) -> Option<Caps> {
        match self {
            Engine::Regex(r) => r.captures(text).map(|c| caps_from_regex(&c)),
            Engine::Fancy(r) => r.captures(text).ok().flatten().map(|c| caps_from_fancy(&c)),
            Engine::Onig(r) => r.captures(text).map(|c| caps_from_onig(&c)),
        }
    }

    /// First match at or after byte offset `start` (PHP's `preg_match` 5th
    /// `$offset` argument). The whole `text` is still visible to the engine, so
    /// `^`/`\A`/lookbehind anchor relative to the true start and reported offsets
    /// stay absolute — matching PCRE. A `start` past the end yields no match.
    pub fn captures_at(&self, text: &str, start: usize) -> Option<Caps> {
        if start > text.len() {
            return None;
        }
        match self {
            Engine::Regex(r) => r.captures_at(text, start).map(|c| caps_from_regex(&c)),
            Engine::Fancy(r) => r.captures_from_pos(text, start).ok().flatten().map(|c| caps_from_fancy(&c)),
            // `start == 0` is a plain `captures`; a positive offset searches from
            // `start` over the still-fully-visible text via `search_with_options`
            // (its own `Captures` cannot be built outside the crate, so the match
            // is read out of a `Region`). Starting the search AT `start` is what
            // makes `\G` anchor there — PCRE's `preg_match($subject, …, $offset)`
            // semantics — while `\A`/`^` keep anchoring to the true start.
            Engine::Onig(r) => {
                if start == 0 {
                    r.captures(text).map(|c| caps_from_onig(&c))
                } else {
                    let mut region = onig::Region::new();
                    r.search_with_options(
                        text,
                        start,
                        text.len(),
                        onig::SearchOptions::SEARCH_OPTION_NONE,
                        Some(&mut region),
                    )
                    .map(|_| caps_from_onig_region(text, &region))
                }
            }
        }
    }

    /// All non-overlapping matches in `text`, eagerly materialised.
    ///
    /// On a `fancy-regex` runtime error (e.g. backtrack-limit exceeded on a
    /// pathological pattern) iteration STOPS (D-36.3 — error means "no further
    /// matches"). Stopping is mandatory, not cosmetic: fancy-regex's iterator
    /// does not advance its cursor past a position whose match attempt errored,
    /// so it yields the same `Err` forever — a `filter_map(Result::ok)` would
    /// loop without end (step 36-3, bug41638).
    pub fn captures_iter(&self, text: &str) -> Vec<Caps> {
        match self {
            Engine::Regex(r) => r.captures_iter(text).map(|c| caps_from_regex(&c)).collect(),
            Engine::Fancy(r) => {
                let mut out = Vec::new();
                for c in r.captures_iter(text) {
                    match c {
                        Ok(caps) => out.push(caps_from_fancy(&caps)),
                        Err(_) => break,
                    }
                }
                out
            }
            Engine::Onig(r) => r.captures_iter(text).map(|c| caps_from_onig(&c)).collect(),
        }
    }

    /// Replace every match in `text` using `repl` (already normalised to the
    /// `${n}` backreference form by [`translate_replacement`]).
    ///
    /// `fancy_regex::Regex::replace_all` is `try_replacen(..).unwrap()`, which
    /// PANICS on a runtime error; the fallible form is used so a backtrack-limit
    /// error on a pathological pattern leaves the text unchanged (D-36.3 — no
    /// match means no replacement) instead of crashing (step 36-3).
    pub fn replace_all(&self, text: &str, repl: &str) -> String {
        match self {
            Engine::Regex(r) => r.replace_all(text, repl).into_owned(),
            Engine::Fancy(r) => r
                .try_replacen(text, 0, repl)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| text.to_string()),
            // oniguruma's own replace uses a different backreference dialect, so
            // expand `repl` (already normalised to `${n}` / `$$` by
            // `translate_replacement`) by hand against every match.
            Engine::Onig(r) => onig_replace_all(r, text, repl),
        }
    }
}

/// Replace every non-overlapping match of `re` in `text`, expanding the
/// `translate_replacement`-normalised template `repl` (`${n}` backreferences,
/// `$$` → literal `$`) for each match. Zero-width matches advance one byte so
/// iteration terminates (mirrors the mbregex replacer).
fn onig_replace_all(re: &onig::Regex, text: &str, repl: &str) -> String {
    let bytes = text.as_bytes();
    let rb = repl.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut last = 0usize;
    for caps in re.captures_iter(text) {
        let (start, end) = caps.pos(0).unwrap();
        out.extend_from_slice(&bytes[last..start]);
        let mut i = 0;
        while i < rb.len() {
            if rb[i] == b'$' && rb.get(i + 1) == Some(&b'$') {
                out.push(b'$');
                i += 2;
            } else if rb[i] == b'$' && rb.get(i + 1) == Some(&b'{') {
                let mut j = i + 2;
                let ds = j;
                while j < rb.len() && rb[j].is_ascii_digit() {
                    j += 1;
                }
                if j > ds && rb.get(j) == Some(&b'}') {
                    let n: usize = repl[ds..j].parse().unwrap_or(usize::MAX);
                    if let Some(s) = caps.at(n) {
                        out.extend_from_slice(s.as_bytes());
                    }
                    i = j + 1;
                } else {
                    out.push(rb[i]);
                    i += 1;
                }
            } else {
                out.push(rb[i]);
                i += 1;
            }
        }
        last = end;
        // Guard zero-width matches: emit the next byte and step past it so the
        // cursor always advances.
        if end == start {
            if end < bytes.len() {
                out.push(bytes[end]);
                last = end + 1;
            } else {
                break;
            }
        }
    }
    out.extend_from_slice(&bytes[last..]);
    String::from_utf8_lossy(&out).into_owned()
}

/// Parse and compile a PHP PCRE pattern. Returns `None` when the delimiters are
/// malformed or neither engine can build the body (step 36 auto-fallback).
pub fn compile(pattern: &[u8]) -> Option<Engine> {
    let open = *pattern.first()?;
    let close = match open {
        b'(' => b')',
        b'{' => b'}',
        b'[' => b']',
        b'<' => b'>',
        d if d.is_ascii_alphanumeric() || d == b'\\' || d == b' ' => return None,
        d => d,
    };
    // The closing delimiter is the last occurrence; everything after it is flags.
    let end = pattern.iter().rposition(|&c| c == close)?;
    if end == 0 {
        return None;
    }
    let body = std::str::from_utf8(&pattern[1..end]).ok()?;
    let flags = &pattern[end + 1..];

    // PCRE's default `$` (no `m`, no `D`) is zero-width and matches at the end of
    // the subject OR just before a single trailing newline. The `regex` crate's
    // `$` is `\z`-only (absolute end), so a bare `$` is rewritten to the
    // lookahead `(?=\n?\z)`. That has no DFA equivalent, so the auto-fallback
    // routes such patterns to fancy-regex (D-37.1). With `D` the `$` keeps the
    // `\z` semantics we already have; with `m` it is per-line (and PHP ignores
    // `D` under `m`) — in both cases the rewrite is skipped.
    let mut body: std::borrow::Cow<str> =
        if !flags.contains(&b'm') && !flags.contains(&b'D') {
            match rewrite_dollar_anchor(body) {
                Some(rw) => std::borrow::Cow::Owned(rw),
                None => std::borrow::Cow::Borrowed(body),
            }
        } else {
            std::borrow::Cow::Borrowed(body)
        };

    // PCRE_ANCHORED (`A`): force the match to start at offset 0. Neither engine
    // has a portable builder switch, so the body is wrapped as `\A(?:…)`. The
    // non-capturing group keeps group numbering intact and anchors a top-level
    // alternation as a whole.
    if flags.contains(&b'A') {
        body = std::borrow::Cow::Owned(format!(r"\A(?:{body})"));
    }

    // First attempt: the fast `regex` engine, with flags applied via the builder.
    // The same i/m/s/x flags are accumulated as an inline `(?..)` prefix for the
    // fancy-regex fallback, which has no equivalent builder API.
    let mut b = RegexBuilder::new(&body);
    let mut inline = String::new();
    for &f in flags {
        match f {
            b'i' => {
                b.case_insensitive(true);
                inline.push('i');
            }
            b'm' => {
                b.multi_line(true);
                inline.push('m');
            }
            b's' => {
                b.dot_matches_new_line(true);
                inline.push('s');
            }
            b'x' => {
                b.ignore_whitespace(true);
                inline.push('x');
            }
            b'U' => {
                // PCRE_UNGREEDY: swap the greediness of every quantifier (an
                // explicit `?` flips it back). `regex` exposes this on the
                // builder and as the inline `(?U)` flag the fancy fallback uses.
                b.swap_greed(true);
                inline.push('U');
            }
            b'X' => {
                // PCRE_EXTRA: deprecated in PCRE2 (PHP's engine) and a no-op.
                // Listed explicitly so it is a documented no-op rather than an
                // accidental fall-through; it must NOT strip whitespace (that is
                // lowercase `x`).
            }
            // u (unicode, already on), D and others: ignored here (A is handled
            // above by wrapping the body; $ leniency / D arrive in 37-4).
            _ => {}
        }
    }
    if let Ok(r) = b.build() {
        return Some(Engine::Regex(r));
    }

    // Fallback: fancy-regex (backreferences + lookaround). Inline flags are
    // prepended as a leading group so they apply to the whole pattern.
    let mut pat = String::new();
    if !inline.is_empty() {
        pat.push_str("(?");
        pat.push_str(&inline);
        pat.push(')');
    }
    pat.push_str(&body);
    // Bound backtracking explicitly at PHP's `pcre.backtrack_limit` default so a
    // pathological pattern errors (→ no-match, D-36.3) rather than running away.
    // This equals fancy-regex 0.14's own default, but pinning it documents the
    // guarantee and survives a future default change (step 36-3).
    if let Ok(r) = fancy_regex::RegexBuilder::new(&pat)
        .backtrack_limit(1_000_000)
        .build()
    {
        return Some(Engine::Fancy(r));
    }

    // Last resort: oniguruma (PHP's own mbregex backend) under the `perl_ng`
    // dialect, which reads PCRE syntax including subroutine calls
    // (`(?&name)`, `(?R)`), `(?(DEFINE))` blocks and recursion — features
    // neither Rust engine can build. Flags map to oniguruma compile options;
    // `A` was already folded into `body` as `\A(?:…)`, and a bare `$` was
    // rewritten upstream, so here we mainly translate i/x/s and pin `^`/`$` to
    // whole-string anchors (PCRE default) unless `m` asked for per-line.
    let mut oo = onig::RegexOptions::REGEX_OPTION_NONE;
    for &f in flags {
        match f {
            b'i' => oo |= onig::RegexOptions::REGEX_OPTION_IGNORECASE,
            b'x' => oo |= onig::RegexOptions::REGEX_OPTION_EXTEND,
            // PCRE `s` (DOTALL): `.` matches newline — oniguruma's MULTILINE.
            b's' => oo |= onig::RegexOptions::REGEX_OPTION_MULTILINE,
            _ => {}
        }
    }
    if !flags.contains(&b'm') {
        // PCRE default: `^`/`$` anchor the whole subject (oniguruma treats them
        // as per-line by default, so opt into SINGLELINE to match PCRE).
        oo |= onig::RegexOptions::REGEX_OPTION_SINGLELINE;
    }
    onig::Regex::with_options(&body, oo, onig::Syntax::perl_ng())
        .ok()
        .map(Engine::Onig)
}

/// Translate a PHP replacement string into the `regex` crate's syntax: PHP
/// accepts `$1`, `${1}` and `\1` for the n-th group; the engine accepts `$1` /
/// `${1}`. Every backreference is normalised to the unambiguous `${n}` form so
/// that e.g. `$1abc` stays "group 1 followed by abc" rather than group "1abc".
pub fn translate_replacement(repl: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < repl.len() {
        let c = repl[i];
        if c == b'\\' || c == b'$' {
            // Optional brace form: \{1} / ${1}.
            let (mut j, braced) = if repl.get(i + 1) == Some(&b'{') {
                (i + 2, true)
            } else {
                (i + 1, false)
            };
            let start = j;
            while j < repl.len() && repl[j].is_ascii_digit() {
                j += 1;
            }
            let digit_end = j;
            if digit_end > start {
                if braced && repl.get(j) == Some(&b'}') {
                    j += 1;
                }
                out.extend_from_slice(b"${");
                out.extend_from_slice(&repl[start..digit_end]);
                out.push(b'}');
                i = j;
                continue;
            }
            // Not a backreference: emit `$` literally as `$$` (engine escape),
            // `\` literally.
            if c == b'$' {
                out.extend_from_slice(b"$$");
            } else {
                out.push(b'\\');
            }
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Rewrite every bare `$` anchor in `body` to the zero-width lookahead
/// `(?=\n?\z)`, reproducing PCRE's default `$` (end of subject, or just before a
/// single trailing newline) which the `regex` crate's `\z`-only `$` cannot
/// express. A `$` written as `\$` or appearing inside a `[...]` character class
/// is a literal dollar and left untouched. Returns `None` when nothing was
/// rewritten, so the caller keeps the original body (and its DFA fast path).
fn rewrite_dollar_anchor(body: &str) -> Option<String> {
    let b = body.as_bytes();
    if !b.contains(&b'$') {
        return None; // common case: no `$` at all, keep the fast path.
    }
    let mut out: Vec<u8> = Vec::with_capacity(b.len() + 8);
    let mut i = 0;
    let mut in_class = false;
    let mut rewrote = false;
    while i < b.len() {
        match b[i] {
            // Copy an escape pair verbatim (covers `\$`, `\]`, `\\`, …).
            b'\\' => {
                out.push(b'\\');
                if i + 1 < b.len() {
                    out.push(b[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            b'[' if !in_class => {
                in_class = true;
                out.push(b'[');
                i += 1;
                // A leading `^` and/or `]` immediately after `[` are literal.
                if i < b.len() && b[i] == b'^' {
                    out.push(b'^');
                    i += 1;
                }
                if i < b.len() && b[i] == b']' {
                    out.push(b']');
                    i += 1;
                }
            }
            b']' if in_class => {
                in_class = false;
                out.push(b']');
                i += 1;
            }
            b'$' if !in_class => {
                out.extend_from_slice(b"(?=\\n?\\z)");
                rewrote = true;
                i += 1;
            }
            other => {
                out.push(other);
                i += 1;
            }
        }
    }
    if rewrote {
        String::from_utf8(out).ok()
    } else {
        None
    }
}

/// `preg_quote`: backslash-escape every PCRE metacharacter (plus an optional
/// delimiter).
pub fn quote(s: &[u8], delim: Option<u8>) -> Vec<u8> {
    const SPECIAL: &[u8] = b".\\+*?[^]$(){}=!<>|:-#";
    let mut out = Vec::with_capacity(s.len());
    for &b in s {
        if SPECIAL.contains(&b) || Some(b) == delim {
            out.push(b'\\');
            out.push(b);
        } else if b == 0 {
            // NUL is escaped as \000 by PHP.
            out.extend_from_slice(b"\\000");
        } else {
            out.push(b);
        }
    }
    out
}

/// `PREG_OFFSET_CAPTURE`.
pub const PREG_OFFSET_CAPTURE: i64 = 256;
/// `PREG_UNMATCHED_AS_NULL`.
pub const PREG_UNMATCHED_AS_NULL: i64 = 512;
/// `PREG_SET_ORDER`.
pub const PREG_SET_ORDER: i64 = 2;

/// Build one match's `$matches` array. Named groups are emitted as the name key
/// immediately followed by their numeric index (PHP order). With
/// `PREG_OFFSET_CAPTURE` each value becomes a `[string, byte-offset]` pair; with
/// `PREG_UNMATCHED_AS_NULL` unmatched groups are `null` and every group is kept,
/// otherwise trailing unmatched groups are dropped.
pub fn captures_array(re: &Engine, caps: &Caps, flags: i64) -> Zval {
    let offset = flags & PREG_OFFSET_CAPTURE != 0;
    let as_null = flags & PREG_UNMATCHED_AS_NULL != 0;
    let names = re.capture_names();
    let limit = if as_null {
        caps.len().saturating_sub(1)
    } else {
        (0..caps.len())
            .rev()
            .find(|&i| caps.get(i).is_some())
            .unwrap_or(0)
    };
    let mut arr = PhpArray::new();
    for i in 0..=limit {
        let val = capture_value(caps.get(i), offset, as_null);
        if let Some(Some(name)) = names.get(i) {
            arr.insert(Key::from_bytes(name.as_bytes()), val.clone());
        }
        arr.insert(Key::Int(i as i64), val);
    }
    Zval::Array(Rc::new(arr))
}

/// A single capture group's value, honouring `PREG_OFFSET_CAPTURE` /
/// `PREG_UNMATCHED_AS_NULL`.
pub fn capture_value(m: Option<&CapMatch>, offset: bool, as_null: bool) -> Zval {
    match m {
        Some(mm) => {
            let s = Zval::Str(PhpStr::new(mm.text.as_bytes().to_vec()));
            if offset {
                offset_pair(s, mm.start as i64)
            } else {
                s
            }
        }
        None => {
            let base = if as_null {
                Zval::Null
            } else {
                Zval::Str(PhpStr::new(Vec::new()))
            };
            if offset {
                offset_pair(base, -1)
            } else {
                base
            }
        }
    }
}

/// `[value, offset]` pair for `PREG_OFFSET_CAPTURE`.
pub(crate) fn offset_pair(value: Zval, off: i64) -> Zval {
    let mut a = PhpArray::new();
    let _ = a.append(value);
    let _ = a.append(Zval::Long(off));
    Zval::Array(Rc::new(a))
}
