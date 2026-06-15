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

/// A compiled PHP pattern. Tries the fast `regex` engine first, falling back to
/// `fancy-regex` for patterns using PCRE features `regex` cannot build (step 36).
pub enum Engine {
    Regex(regex::Regex),
    Fancy(fancy_regex::Regex),
}

impl Engine {
    /// Number of capture slots (whole match + named/numbered groups).
    pub fn captures_len(&self) -> usize {
        match self {
            Engine::Regex(r) => r.captures_len(),
            Engine::Fancy(r) => r.captures_len(),
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
        }
    }

    /// First match in `text`, engine-neutral. A `fancy-regex` runtime error
    /// (e.g. backtrack limit) collapses to "no match" (D-36.3).
    pub fn captures(&self, text: &str) -> Option<Caps> {
        match self {
            Engine::Regex(r) => r.captures(text).map(|c| caps_from_regex(&c)),
            Engine::Fancy(r) => r.captures(text).ok().flatten().map(|c| caps_from_fancy(&c)),
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
        }
    }
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

    // First attempt: the fast `regex` engine, with flags applied via the builder.
    // The same i/m/s/x flags are accumulated as an inline `(?..)` prefix for the
    // fancy-regex fallback, which has no equivalent builder API.
    let mut b = RegexBuilder::new(body);
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
            // u (unicode, already on), A/D/X and others: ignored.
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
    pat.push_str(body);
    // Bound backtracking explicitly at PHP's `pcre.backtrack_limit` default so a
    // pathological pattern errors (→ no-match, D-36.3) rather than running away.
    // This equals fancy-regex 0.14's own default, but pinning it documents the
    // guarantee and survives a future default change (step 36-3).
    fancy_regex::RegexBuilder::new(&pat)
        .backtrack_limit(1_000_000)
        .build()
        .ok()
        .map(Engine::Fancy)
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
