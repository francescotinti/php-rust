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
/// evaluator. `text` is bytes (PHP strings are): for a latin1-round-tripped
/// subject (see [`subject_text`]) it holds the *original* subject bytes after
/// [`Caps::latin1_fix`].
pub struct CapMatch {
    pub start: usize,
    pub end: usize,
    pub text: Vec<u8>,
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

    /// Map a capture set produced in the latin1-decoded domain (see
    /// [`subject_text`]) back to the subject's original byte domain: every
    /// text latin1-encodes, and the UTF-8 offsets become original byte
    /// offsets (each original byte decoded to exactly one char, so the
    /// offset is the char count before the span).
    pub fn latin1_fix(&mut self, decoded: &str) {
        for g in self.groups.iter_mut().flatten() {
            g.text = latin1_encode(&String::from_utf8_lossy(&g.text));
            g.start = decoded[..g.start.min(decoded.len())].chars().count();
            g.end = g.start + g.text.len();
        }
    }
}

/// Whether a PHP pattern carries the `/u` (PCRE_UTF8) modifier — flags sit
/// after the last closing delimiter, mirroring [`compile`]'s extraction.
pub fn pattern_is_unicode(pattern: &[u8]) -> bool {
    let Some(&open) = pattern.first() else { return false };
    let close = match open {
        b'(' => b')',
        b'{' => b'}',
        b'[' => b']',
        b'<' => b'>',
        d => d,
    };
    match pattern.iter().rposition(|&c| c == close) {
        Some(end) => pattern[end + 1..].contains(&b'u'),
        None => false,
    }
}

/// Decode bytes as latin1: byte N → char U+00N (bijective).
pub fn latin1_decode(b: &[u8]) -> String {
    b.iter().map(|&c| c as char).collect()
}

/// Encode the latin1 round-trip back: chars ≤ U+00FF become their single
/// byte; anything above (text injected from a genuine-UTF-8 replacement)
/// keeps its UTF-8 bytes.
pub fn latin1_encode(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len());
    for c in s.chars() {
        if (c as u32) <= 0xFF {
            out.push(c as u32 as u8);
        } else {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
    }
    out
}

/// A regex subject as text for the str-based engines. PHP subjects are byte
/// strings: a valid-UTF-8 one runs directly; an invalid one under `/u` is a
/// `PREG_BAD_UTF8_ERROR` (`None` → the caller returns `false` like PHP), and
/// under a byte-mode pattern round-trips through latin1 (the caller must
/// [`Caps::latin1_fix`] every capture set and latin1-encode assembled
/// output). A valid-UTF-8 subject under a byte-mode pattern keeps the
/// existing char-domain behaviour (documented divergence: PHP matches bytes
/// there too, but real byte-mode patterns on valid UTF-8 are ASCII-safe).
pub enum SubjectText<'a> {
    Utf8(&'a str),
    Latin1(String),
}

impl SubjectText<'_> {
    pub fn as_str(&self) -> &str {
        match self {
            SubjectText::Utf8(s) => s,
            SubjectText::Latin1(s) => s,
        }
    }

    pub fn is_latin1(&self) -> bool {
        matches!(self, SubjectText::Latin1(_))
    }
}

/// See [`SubjectText`]. `None` = invalid UTF-8 under `/u` (PHP: `false`).
/// Without `/u`, PCRE is BYTE-oriented: a valid-UTF-8 subject with high bytes
/// still matches per byte (`[\x80-\xff]` hits each half of a UTF-8 pair —
/// WP's esc_url keeps its `\x80-\xff` allowlist working this way), so
/// anything non-ASCII goes through the 1-byte-per-char Latin1 view.
thread_local! {
    /// `preg_last_error`: 0 = PREG_NO_ERROR, 1 = PREG_INTERNAL_ERROR (pattern
    /// invalido), 4 = PREG_BAD_UTF8_ERROR (subject non-UTF-8 sotto `/u`).
    /// phpr non ha backtrack/recursion limit (divergenza documentata), quindi
    /// gli altri codici non occorrono mai.
    static PREG_LAST_ERROR: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

pub fn set_last_error(code: i64) {
    PREG_LAST_ERROR.with(|c| c.set(code));
}

pub fn last_error() -> i64 {
    PREG_LAST_ERROR.with(|c| c.get())
}

/// Il testo di `preg_last_error_msg()` per i codici che phpr produce.
pub fn last_error_msg() -> &'static str {
    match last_error() {
        0 => "No error",
        1 => "Internal error",
        4 => "Malformed UTF-8 characters, possibly incorrectly encoded",
        _ => "Internal error",
    }
}

pub fn subject_text(subject: &[u8], unicode: bool) -> Option<SubjectText<'_>> {
    if unicode {
        let txt = std::str::from_utf8(subject).ok();
        if txt.is_none() {
            set_last_error(4);
        }
        return txt.map(SubjectText::Utf8);
    }
    match std::str::from_utf8(subject) {
        Ok(s) if s.is_ascii() => Some(SubjectText::Utf8(s)),
        _ => Some(SubjectText::Latin1(latin1_decode(subject))),
    }
}

fn caps_from_regex(caps: &regex::Captures) -> Caps {
    let groups = (0..caps.len())
        .map(|i| {
            caps.get(i).map(|m| CapMatch {
                start: m.start(),
                end: m.end(),
                text: m.as_str().as_bytes().to_vec(),
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
                text: m.as_str().as_bytes().to_vec(),
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
                text: caps.at(i).unwrap_or("").as_bytes().to_vec(),
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
                text: text.get(start..end).unwrap_or("").as_bytes().to_vec(),
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
    /// PCRE_ANCHORED (`/A`): the inner engine compiled WITHOUT any `\A` wrap;
    /// every search accepts a match only if it starts exactly at the search
    /// offset. This is PCRE's semantics — `\A` would anchor to the *string*
    /// start and never match `preg_match($re, $s, $m, 0, $offset)` at a
    /// positive offset (symfony/expression-language's Lexer tokenises that
    /// way), while lookbehinds must still see the bytes before the offset.
    Anchored(Box<Engine>),
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
            Engine::Anchored(inner) => inner.captures_len(),
        }
    }

    /// Capture-group names by index (`None` for unnamed / the whole match),
    /// collected to owned strings so both backends share one return type.
    /// Synthetic names introduced by [`demix_numbered_backrefs`] are hidden
    /// (their groups read as unnamed, exactly like the original pattern).
    pub fn capture_names(&self) -> Vec<Option<String>> {
        let strip_synthetic = |v: Vec<Option<String>>| -> Vec<Option<String>> {
            v.into_iter()
                .map(|n| n.filter(|s| !s.starts_with(SYN_BACKREF_PREFIX)))
                .collect()
        };
        return strip_synthetic(match self {
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
            Engine::Anchored(inner) => inner.capture_names(),
        });
    }

    /// First match in `text`, engine-neutral. A `fancy-regex` runtime error
    /// (e.g. backtrack limit) collapses to "no match" (D-36.3).
    pub fn captures(&self, text: &str) -> Option<Caps> {
        match self {
            Engine::Regex(r) => r.captures(text).map(|c| caps_from_regex(&c)),
            Engine::Fancy(r) => r.captures(text).ok().flatten().map(|c| caps_from_fancy(&c)),
            Engine::Onig(r) => r.captures(text).map(|c| caps_from_onig(&c)),
            Engine::Anchored(_) => self.captures_at(text, 0),
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
            // Anchored: search from `start`, then require the match to begin
            // exactly there (a later match is what PCRE would never have tried).
            Engine::Anchored(inner) => inner
                .captures_at(text, start)
                .filter(|c| c.get(0).is_some_and(|m| m.start == start)),
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
            // Anchored global scan (PCRE2_ANCHORED under preg_match_all): each
            // match must start where the previous one ended; the first gap ends
            // the scan. A zero-width match advances one byte so it terminates.
            Engine::Anchored(_) => {
                let mut out = Vec::new();
                let mut pos = 0usize;
                while pos <= text.len() {
                    let Some(c) = self.captures_at(text, pos) else { break };
                    let (s, e) = c.get(0).map(|m| (m.start, m.end)).unwrap_or((pos, pos));
                    out.push(c);
                    pos = if e > s { e } else { e + 1 };
                }
                out
            }
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
            // Anchored replace: only the contiguous run of matches from each
            // scan position is replaced (same walk as `captures_iter`).
            Engine::Anchored(_) => {
                let mut out = String::new();
                let mut last = 0usize;
                for c in self.captures_iter(text) {
                    let Some(m) = c.get(0) else { continue };
                    out.push_str(&text[last..m.start]);
                    out.push_str(&expand_caps_template(&c, repl));
                    last = m.end;
                }
                out.push_str(&text[last..]);
                out
            }
        }
    }

    /// Like [`Self::replace_all`] but replaces at most `limit` matches
    /// (`limit == 0` means unlimited — the Rust `replacen` convention; the
    /// caller maps preg_replace's `$limit = -1` onto it). Serves preg_replace's
    /// 4th argument: PhpDumper prunes its container template with
    /// `preg_replace(..., limit: 1)` and a second removal corrupts the dump.
    pub fn replacen(&self, text: &str, limit: usize, repl: &str) -> String {
        if limit == 0 {
            return self.replace_all(text, repl);
        }
        match self {
            Engine::Regex(r) => r.replacen(text, limit, repl).into_owned(),
            Engine::Fancy(r) => r
                .try_replacen(text, limit, repl)
                .map(|c| c.into_owned())
                .unwrap_or_else(|_| text.to_string()),
            // Onig/Anchored: the same manual walk as `replace_all`, capped.
            Engine::Onig(_) | Engine::Anchored(_) => {
                let mut out = String::new();
                let mut last = 0usize;
                for (n, c) in self.captures_iter(text).into_iter().enumerate() {
                    if n >= limit {
                        break;
                    }
                    let Some(m) = c.get(0) else { continue };
                    out.push_str(&text[last..m.start]);
                    out.push_str(&expand_caps_template(&c, repl));
                    last = m.end;
                }
                out.push_str(&text[last..]);
                out
            }
        }
    }
}

/// Expand the `translate_replacement`-normalised template `repl` (`${n}`
/// backreferences, `$$` → literal `$`) against one engine-neutral match.
fn expand_caps_template(c: &Caps, repl: &str) -> String {
    let rb = repl.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < rb.len() {
        if rb[i] == b'$' && rb.get(i + 1) == Some(&b'$') {
            out.push(b'$');
            i += 2;
        } else if rb[i] == b'$' && rb.get(i + 1) == Some(&b'{') {
            let ds = i + 2;
            let mut j = ds;
            while j < rb.len() && rb[j].is_ascii_digit() {
                j += 1;
            }
            if j > ds && rb.get(j) == Some(&b'}') {
                let n: usize = repl[ds..j].parse().unwrap_or(usize::MAX);
                if let Some(m) = c.get(n) {
                    out.extend_from_slice(&m.text);
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
    String::from_utf8_lossy(&out).into_owned()
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
    // php_pcre.c skips leading whitespace (incl. newlines) before reading the
    // delimiter — an indented-heredoc pattern (doctrine/dbal's name parser)
    // starts with spaces and is valid PCRE.
    let ws = pattern.iter().take_while(|b| b.is_ascii_whitespace()).count();
    let pattern = &pattern[ws..];
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
    let flags = &pattern[end + 1..];
    // Without `/u` PCRE is byte-oriented: a body with high bytes gets the
    // same 1-byte-per-char Latin1 view the subject does (see subject_text),
    // so multibyte literals and `[\x80-\xff]` classes both match per byte.
    let body_owned: String;
    let body: &str = if flags.contains(&b'u') || pattern[1..end].is_ascii() {
        std::str::from_utf8(&pattern[1..end]).ok()?
    } else {
        body_owned = latin1_decode(&pattern[1..end]);
        &body_owned
    };

    // PCRE's default `$` (no `m`, no `D`) is zero-width and matches at the end of
    // the subject OR just before a single trailing newline. The `regex` crate's
    // `$` is `\z`-only (absolute end), so a bare `$` is rewritten to the
    // lookahead `(?=\n?\z)`. That has no DFA equivalent, so the auto-fallback
    // routes such patterns to fancy-regex (D-37.1). With `D` the `$` keeps the
    // `\z` semantics we already have; with `m` it is per-line (and PHP ignores
    // `D` under `m`) — in both cases the rewrite is skipped.
    let body: std::borrow::Cow<str> =
        if !flags.contains(&b'm') && !flags.contains(&b'D') {
            match rewrite_dollar_anchor(body) {
                Some(rw) => std::borrow::Cow::Owned(rw),
                None => std::borrow::Cow::Borrowed(body),
            }
        } else {
            std::borrow::Cow::Borrowed(body)
        };

    // PCRE reads `\<` and `\>` as redundant escapes of the LITERAL `<`/`>`;
    // the `regex` crate (1.10+) reads them as zero-width word-START/END
    // boundaries instead — an alternation like `…|\<|\>|…` (symfony
    // expression-language's operator lexer) then "matches" the empty string
    // at every word edge and a `$cursor += strlen($match)` tokeniser loops
    // forever. Unescape them outside AND inside classes (`<`/`>` are never
    // metacharacters), honouring `\\` pairs.
    let body: std::borrow::Cow<str> = if body.contains(r"\<") || body.contains(r"\>") {
        let src = body.as_bytes();
        let mut out = Vec::with_capacity(src.len());
        let mut i = 0;
        while i < src.len() {
            if src[i] == b'\\' && i + 1 < src.len() {
                match src[i + 1] {
                    b'<' | b'>' => out.push(src[i + 1]),
                    c => {
                        out.push(b'\\');
                        out.push(c);
                    }
                }
                i += 2;
            } else {
                out.push(src[i]);
                i += 1;
            }
        }
        std::borrow::Cow::Owned(String::from_utf8(out).ok()?)
    } else {
        body
    };

    // PCRE reads a bare `[` inside a character class as the LITERAL bracket
    // (`[([{"\-]`, wptexturize's openers class); the Rust engines read it as a
    // nested-class opener and fail to parse — which matters when the pattern
    // then needs fancy-regex (variable-length lookbehind: oniguruma, the last
    // fallback, rejects those). Escape it, leaving POSIX classes
    // (`[[:alpha:]]`) intact.
    let body: std::borrow::Cow<str> = match escape_class_brackets(&body) {
        Some(rw) => std::borrow::Cow::Owned(rw),
        None => body,
    };

    // A negative lookbehind over an ALTERNATION decomposes into a conjunction
    // of single-branch lookbehinds (De Morgan: not(A or B) = not A and not B)
    // — `(?<!A|B|C)` → `(?<!A)(?<!B)(?<!C)`. The variable-length alternation
    // form is what the backend engines reject in combination with other fancy
    // constructs (wptexturize's `(?<!<spaces>)'(?!\Z|…)` apostrophe rule); the
    // fixed-length conjuncts compile everywhere.
    let body: std::borrow::Cow<str> = match split_neg_lookbehind_alternation(&body) {
        Some(rw) => std::borrow::Cow::Owned(rw),
        None => body,
    };

    // PCRE conditional groups with a LOOKAHEAD condition — `(?(?=A)B|C)` — are
    // rejected by every backend engine (regex, fancy-regex, oniguruma
    // perl_ng). They rewrite EXACTLY to a guarded alternation
    // `(?:(?=A)B|(?!A)C)`: PCRE commits to one branch on the condition and
    // never falls back to the other, which is precisely what the mutually
    // exclusive guards encode. WordPress' wp_html_split/wptexturize regexes
    // live on this construct.
    let body: std::borrow::Cow<str> = if body.contains("(?(?=") || body.contains("(?(?!") {
        match rewrite_lookahead_conditionals(&body) {
            Some(rw) => std::borrow::Cow::Owned(rw),
            None => body,
        }
    } else {
        body
    };

    // PCRE_ANCHORED (`A`): the match must start exactly at the search offset.
    // A `\A(?:…)` wrap is WRONG for `preg_match(..., $offset)` at a positive
    // offset (it anchors to the string start and never matches — symfony's
    // expression-language Lexer broke on it), and slicing the subject would
    // blind lookbehinds. Instead the compiled engine is wrapped in
    // [`Engine::Anchored`], whose searches filter on `match.start == offset`.
    let anchored = flags.contains(&b'A');
    let wrap = |e: Engine| if anchored { Engine::Anchored(Box::new(e)) } else { e };

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
        return Some(wrap(Engine::Regex(r)));
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
        return Some(wrap(Engine::Fancy(r)));
    }
    // PCRE allows mixing NAMED groups with NUMBERED backreferences in one
    // pattern; both fallback engines reject that ("numbered backref/call is
    // not allowed") — wp-cli's FILE_DIR_PATTERN skips quoted strings via
    // `(?=(\\?))\1` next to `(?<file>__FILE__)`. Rewrite the numbered refs to
    // synthetic named form and retry; `capture_names()` hides the synthetic
    // names again so PHP's $matches shape is unchanged.
    let demixed = demix_numbered_backrefs(&body);
    if let Some(demixed_body) = &demixed {
        let mut pat = String::new();
        if !inline.is_empty() {
            pat.push_str("(?");
            pat.push_str(&inline);
            pat.push(')');
        }
        pat.push_str(demixed_body);
        if let Ok(r) = fancy_regex::RegexBuilder::new(&pat)
            .backtrack_limit(1_000_000)
            .build()
        {
            return Some(wrap(Engine::Fancy(r)));
        }
    }

    // Last resort: oniguruma (PHP's own mbregex backend) under the `perl_ng`
    // dialect, which reads PCRE syntax including subroutine calls
    // (`(?&name)`, `(?R)`), `(?(DEFINE))` blocks and recursion — features
    // neither Rust engine can build. Flags map to oniguruma compile options;
    // `A` is handled by the [`Engine::Anchored`] wrapper, and a bare `$` was
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
    let body = onigify_python_groups(demixed.as_deref().unwrap_or(&body));
    onig::Regex::with_options(&body, oo, onig::Syntax::perl_ng())
        .ok()
        .map(|r| wrap(Engine::Onig(r)))
}

/// Escape a bare `[` inside a character class (`[([{"\-]` → `[(\[{"\-]`):
/// PCRE reads it as the literal bracket, the Rust engines as a nested class.
/// A POSIX class (`[[:alpha:]]`) keeps its inner bracket. Returns `None`
/// when nothing needed escaping.
fn escape_class_brackets(body: &str) -> Option<String> {
    let s = body.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(s.len() + 4);
    let mut changed = false;
    let mut in_class = false;
    let mut class_start = 0usize; // position just after the class's `[` (and `^`)
    let mut i = 0;
    while i < s.len() {
        let b = s[i];
        if b == b'\\' && i + 1 < s.len() {
            out.extend_from_slice(&s[i..i + 2]);
            i += 2;
            continue;
        }
        if !in_class {
            if b == b'[' {
                in_class = true;
                out.push(b);
                i += 1;
                if s.get(i) == Some(&b'^') {
                    out.push(b'^');
                    i += 1;
                }
                // A leading `]` is the literal bracket in PCRE.
                if s.get(i) == Some(&b']') {
                    out.extend_from_slice(b"\\]");
                    changed = true;
                    i += 1;
                }
                class_start = i;
                continue;
            }
            out.push(b);
            i += 1;
            continue;
        }
        // Inside a class.
        match b {
            b']' => {
                in_class = false;
                out.push(b);
                i += 1;
            }
            b'[' => {
                // `[:alpha:]` / `[.x.]` / `[=x=]` POSIX forms stay verbatim.
                if matches!(s.get(i + 1), Some(b':') | Some(b'.') | Some(b'=')) {
                    let punct = s[i + 1];
                    if let Some(close) = s[i + 2..]
                        .windows(2)
                        .position(|w| w == [punct, b']'])
                    {
                        out.extend_from_slice(&s[i..i + 2 + close + 2]);
                        i += 2 + close + 2;
                        continue;
                    }
                }
                out.extend_from_slice(b"\\[");
                changed = true;
                i += 1;
            }
            _ => {
                out.push(b);
                i += 1;
            }
        }
    }
    let _ = class_start;
    if !changed {
        return None;
    }
    String::from_utf8(out).ok()
}

/// Split `(?<!A|B|C)` into `(?<!A)(?<!B)(?<!C)` (exact De Morgan equivalence
/// for negative lookbehind). Only applied when the body has a top-level `|`
/// and contains no groups at all (branches with groups keep their form — the
/// simple character/literal alternations are the ones the engines reject).
/// Returns `None` when nothing was rewritten.
fn split_neg_lookbehind_alternation(body: &str) -> Option<String> {
    let s = body.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut changed = false;
    let mut in_class = false;
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'\\' && i + 1 < s.len() {
            out.extend_from_slice(&s[i..i + 2]);
            i += 2;
            continue;
        }
        if in_class {
            if s[i] == b']' {
                in_class = false;
            }
            out.push(s[i]);
            i += 1;
            continue;
        }
        if s[i] == b'[' {
            in_class = true;
            out.push(s[i]);
            i += 1;
            continue;
        }
        if s[i..].starts_with(b"(?<!") {
            // Find the matching close, tracking classes/escapes; bail on
            // nested groups (those alternations stay as-is).
            let mut depth = 1i32;
            let mut cls = false;
            let mut has_group = false;
            let mut splits: Vec<usize> = Vec::new(); // top-level `|` offsets
            let mut j = i + 4;
            let mut end = None;
            while j < s.len() {
                match s[j] {
                    b'\\' => j += 1,
                    b'[' if !cls => cls = true,
                    b']' if cls => cls = false,
                    b'(' if !cls => {
                        depth += 1;
                        has_group = true;
                    }
                    b')' if !cls => {
                        depth -= 1;
                        if depth == 0 {
                            end = Some(j);
                            break;
                        }
                    }
                    b'|' if !cls && depth == 1 => splits.push(j),
                    _ => {}
                }
                j += 1;
            }
            let Some(end) = end else {
                out.extend_from_slice(&s[i..]);
                break;
            };
            if splits.is_empty() || has_group {
                out.extend_from_slice(&s[i..=end]);
                i = end + 1;
                continue;
            }
            let mut start = i + 4;
            let mut bounds = splits;
            bounds.push(end);
            for b in bounds {
                out.extend_from_slice(b"(?<!");
                out.extend_from_slice(&s[start..b]);
                out.push(b')');
                start = b + 1;
            }
            changed = true;
            i = end + 1;
            continue;
        }
        out.push(s[i]);
        i += 1;
    }
    if !changed {
        return None;
    }
    String::from_utf8(out).ok()
}

/// Rewrite PCRE conditionals whose condition is a pure lookahead —
/// `(?(?=A)B|C)` → `(?:(?=A)B|(?!A)C)` and `(?(?!A)B|C)` → `(?:(?!A)B|(?=A)C)`
/// — recursively (branches and conditions may nest further conditionals).
/// Returns `None` when nothing was rewritten, or when a condition contains a
/// capturing group (the guard duplicates the condition, which would renumber
/// `$matches` — such patterns stay with the engines as-is).
fn rewrite_lookahead_conditionals(body: &str) -> Option<String> {
    /// Whether `s` contains a capturing group outside character classes
    /// (plain `(`, `(?P<`, `(?<name>`, `(?'name'`).
    fn has_capturing_group(s: &[u8]) -> bool {
        let mut in_class = false;
        let mut i = 0;
        while i < s.len() {
            match s[i] {
                b'\\' => i += 1,
                b'[' if !in_class => in_class = true,
                b']' if in_class => in_class = false,
                b'(' if !in_class => {
                    if s.get(i + 1) != Some(&b'?') {
                        return true;
                    }
                    match s.get(i + 2) {
                        Some(b'P') | Some(b'\'') => return true,
                        Some(b'<') if !matches!(s.get(i + 3), Some(b'=') | Some(b'!')) => {
                            return true
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            i += 1;
        }
        false
    }
    /// Index of the `)` matching the `(` at `open` (class- and escape-aware).
    fn close_paren(s: &[u8], open: usize) -> Option<usize> {
        let mut depth = 0i32;
        let mut in_class = false;
        let mut i = open;
        while i < s.len() {
            match s[i] {
                b'\\' => i += 1,
                b'[' if !in_class => in_class = true,
                b']' if in_class => in_class = false,
                b'(' if !in_class => depth += 1,
                b')' if !in_class => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }
    fn rw(s: &[u8], changed: &mut bool) -> Option<Vec<u8>> {
        let mut out: Vec<u8> = Vec::with_capacity(s.len());
        let mut in_class = false;
        let mut i = 0;
        while i < s.len() {
            if s[i] == b'\\' && i + 1 < s.len() {
                out.extend_from_slice(&s[i..i + 2]);
                i += 2;
                continue;
            }
            if in_class {
                if s[i] == b']' {
                    in_class = false;
                }
                out.push(s[i]);
                i += 1;
                continue;
            }
            if s[i] == b'[' {
                in_class = true;
                out.push(s[i]);
                i += 1;
                continue;
            }
            if s[i..].starts_with(b"(?(?=") || s[i..].starts_with(b"(?(?!") {
                let whole_end = close_paren(s, i)?; // `)` of the conditional
                let cond_open = i + 2; // `(` of the lookahead condition
                let cond_end = close_paren(s, cond_open)?;
                let neg = s[i + 4] == b'!';
                let cond_inner = &s[cond_open + 3..cond_end];
                if has_capturing_group(cond_inner) {
                    return None;
                }
                // Split yes|no at depth 0 between the condition and the close.
                let mut depth = 0i32;
                let mut cls = false;
                let mut split = None;
                let mut j = cond_end + 1;
                while j < whole_end {
                    match s[j] {
                        b'\\' => j += 1,
                        b'[' if !cls => cls = true,
                        b']' if cls => cls = false,
                        b'(' if !cls => depth += 1,
                        b')' if !cls => depth -= 1,
                        b'|' if !cls && depth == 0 => {
                            split = Some(j);
                            break;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let (yes_raw, no_raw) = match split {
                    Some(p) => (&s[cond_end + 1..p], &s[p + 1..whole_end]),
                    None => (&s[cond_end + 1..whole_end], &b""[..]),
                };
                let cond = rw(cond_inner, changed)?;
                let yes = rw(yes_raw, changed)?;
                let no = rw(no_raw, changed)?;
                let (g, ng) = if neg { (b'!', b'=') } else { (b'=', b'!') };
                out.extend_from_slice(b"(?:(?");
                out.push(g);
                out.extend_from_slice(&cond);
                out.push(b')');
                out.extend_from_slice(&yes);
                out.extend_from_slice(b"|(?");
                out.push(ng);
                out.extend_from_slice(&cond);
                out.push(b')');
                out.extend_from_slice(&no);
                out.push(b')');
                *changed = true;
                i = whole_end + 1;
                continue;
            }
            out.push(s[i]);
            i += 1;
        }
        Some(out)
    }
    let mut changed = false;
    let out = rw(body.as_bytes(), &mut changed)?;
    if !changed {
        return None;
    }
    String::from_utf8(out).ok()
}

/// The name prefix for capturing groups synthesised by
/// [`demix_numbered_backrefs`]; [`Engine::capture_names`] hides them.
const SYN_BACKREF_PREFIX: &str = "__phprbg";

/// Rewrite a pattern that mixes NAMED capture groups with NUMBERED
/// backreferences (allowed by PCRE, rejected by fancy-regex and oniguruma):
/// every unnamed group a numbered backref targets gets a synthetic name and
/// the backref becomes `\k<name>`. Returns `None` when no rewrite is needed
/// (no named groups, no numbered backrefs) or when the pattern uses
/// constructs that shift group numbering (`(?|`) or conditionals (`(?(`),
/// which this rewriter does not model.
fn demix_numbered_backrefs(body: &str) -> Option<String> {
    let b = body.as_bytes();
    // Pass A: inventory the capturing groups (index → native name, if any).
    let mut group_names: Vec<Option<String>> = vec![None]; // slot 0 = pad
    let mut has_named = false;
    let mut in_class = false;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'[' => in_class = true,
            b'(' => {
                if b.get(i + 1) == Some(&b'?') {
                    if b.get(i + 2) == Some(&b'|') || b.get(i + 2) == Some(&b'(') {
                        return None; // branch-reset / conditional: numbering games
                    }
                    // Named forms: (?<name> (not lookbehind), (?P<name>, (?'name'
                    let (open, close, start) = match (b.get(i + 2), b.get(i + 3)) {
                        (Some(b'<'), Some(d)) if *d != b'=' && *d != b'!' => (b'<', b'>', i + 3),
                        (Some(b'P'), Some(b'<')) => (b'<', b'>', i + 4),
                        (Some(b'\''), _) => (b'\'', b'\'', i + 3),
                        _ => {
                            i += 1;
                            continue; // non-capturing / lookaround / atomic / flags
                        }
                    };
                    let _ = open;
                    let mut j = start;
                    while j < b.len() && b[j] != close {
                        j += 1;
                    }
                    group_names.push(Some(String::from_utf8_lossy(&b[start..j]).into_owned()));
                    has_named = true;
                } else {
                    group_names.push(None);
                }
            }
            _ => {}
        }
        i += 1;
    }
    let n_groups = group_names.len() - 1;
    if !has_named || n_groups == 0 {
        return None;
    }
    // Pass B+C: rewrite backrefs, naming the targeted unnamed groups.
    let mut targets: Vec<bool> = vec![false; n_groups + 1];
    let parse_ref = |i: usize| -> Option<(usize, usize)> {
        // `\N` / `\NN` at byte i+1; two digits win when that group exists (PCRE).
        let d1 = *b.get(i + 1)?;
        if !d1.is_ascii_digit() || d1 == b'0' {
            return None;
        }
        let n1 = (d1 - b'0') as usize;
        if let Some(&d2) = b.get(i + 2) {
            if d2.is_ascii_digit() {
                let n2 = n1 * 10 + (d2 - b'0') as usize;
                if n2 <= n_groups {
                    return Some((n2, 3));
                }
            }
        }
        (n1 <= n_groups).then_some((n1, 2))
    };
    // First sweep marks the targets so pass C can name their groups.
    let mut any_ref = false;
    {
        let mut in_class = false;
        let mut i = 0;
        while i < b.len() {
            let c = b[i];
            if c == b'\\' && i + 1 < b.len() {
                if !in_class {
                    if let Some((n, w)) = parse_ref(i) {
                        targets[n] = true;
                        any_ref = true;
                        i += w;
                        continue;
                    }
                }
                i += 2;
                continue;
            }
            if c == b'[' && !in_class {
                in_class = true;
            } else if c == b']' && in_class {
                in_class = false;
            }
            i += 1;
        }
    }
    if !any_ref {
        return None;
    }
    // Pass C: emit, renaming group openings and rewriting the refs. Built as
    // bytes (multibyte UTF-8 passes through untouched).
    let mut out: Vec<u8> = Vec::with_capacity(body.len() + 16);
    let mut in_class = false;
    let mut group_no = 0usize;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            if !in_class {
                if let Some((n, w)) = parse_ref(i) {
                    out.extend_from_slice(b"\\k<");
                    match &group_names[n] {
                        Some(name) => out.extend_from_slice(name.as_bytes()),
                        None => {
                            out.extend_from_slice(SYN_BACKREF_PREFIX.as_bytes());
                            out.extend_from_slice(n.to_string().as_bytes());
                        }
                    }
                    out.push(b'>');
                    i += w;
                    continue;
                }
            }
            out.push(c);
            out.push(b[i + 1]);
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            out.push(c);
            i += 1;
            continue;
        }
        match c {
            b'[' => {
                in_class = true;
                out.push(b'[');
            }
            b'(' => {
                if b.get(i + 1) == Some(&b'?') {
                    // Count named groups (pass A logic) but emit unchanged.
                    match (b.get(i + 2), b.get(i + 3)) {
                        (Some(b'<'), Some(d)) if *d != b'=' && *d != b'!' => group_no += 1,
                        (Some(b'P'), Some(b'<')) => group_no += 1,
                        (Some(b'\''), _) => group_no += 1,
                        _ => {}
                    }
                    out.push(b'(');
                } else {
                    group_no += 1;
                    if targets[group_no] && group_names[group_no].is_none() {
                        out.extend_from_slice(b"(?<");
                        out.extend_from_slice(SYN_BACKREF_PREFIX.as_bytes());
                        out.extend_from_slice(group_no.to_string().as_bytes());
                        out.push(b'>');
                    } else {
                        out.push(b'(');
                    }
                }
            }
            _ => out.push(c),
        }
        i += 1;
    }
    String::from_utf8(out).ok()
}

/// Rewrite Python-style named-group syntax — `(?P<name>…)`, `(?P=name)`,
/// `(?P>name)` — into the spellings oniguruma's `perl_ng` dialect accepts
/// (`(?<name>…)`, `\k<name>`, `\g<name>`). PCRE reads both; the two Rust
/// engines read `(?P<…>` natively, so only this last-resort path needs the
/// translation (Composer's JsonManipulator mixes `(?P<start>` with
/// `(?(DEFINE))`, which is exactly the combination that lands here). Escaped
/// characters and character classes are left untouched.
fn onigify_python_groups(body: &str) -> String {
    let b = body.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    let mut in_class = false;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' && i + 1 < b.len() {
            out.push(c);
            out.push(b[i + 1]);
            i += 2;
            continue;
        }
        if in_class {
            if c == b']' {
                in_class = false;
            }
            out.push(c);
            i += 1;
            continue;
        }
        if c == b'[' {
            in_class = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == b'(' && b[i..].starts_with(b"(?P") {
            match b.get(i + 3) {
                // `(?P<name>` → `(?<name>`
                Some(b'<') => {
                    out.extend_from_slice(b"(?<");
                    i += 4;
                    continue;
                }
                // `(?P=name)` → `\k<name>` (backreference by name)
                Some(b'=') => {
                    if let Some(end) = b[i + 4..].iter().position(|&x| x == b')') {
                        out.extend_from_slice(b"\\k<");
                        out.extend_from_slice(&b[i + 4..i + 4 + end]);
                        out.push(b'>');
                        i += 4 + end + 1;
                        continue;
                    }
                }
                // `(?P>name)` → `\g<name>` (subroutine call by name)
                Some(b'>') => {
                    if let Some(end) = b[i + 4..].iter().position(|&x| x == b')') {
                        out.extend_from_slice(b"\\g<");
                        out.extend_from_slice(&b[i + 4..i + 4 + end]);
                        out.push(b'>');
                        i += 4 + end + 1;
                        continue;
                    }
                }
                _ => {}
            }
        }
        out.push(c);
        i += 1;
    }
    // Only ASCII was spliced, so the bytes stay valid UTF-8.
    String::from_utf8(out).unwrap_or_else(|_| body.to_string())
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
        // PCRE replacement: `\\` is ONE literal backslash (doctrine's
        // escapeStringForLike relies on `'\\\\$1'` producing `\` + group).
        if c == b'\\' && repl.get(i + 1) == Some(&b'\\') {
            out.push(b'\\');
            i += 2;
            continue;
        }
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
            let s = Zval::Str(PhpStr::new(mm.text.clone()));
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
