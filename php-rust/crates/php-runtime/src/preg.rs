//! PCRE pattern support for the `preg_*` builtins (step 27).
//!
//! Patterns are PHP's delimited form (`/body/flags`); they are translated to the
//! Rust `regex` crate. Backreferences and lookaround are not supported by that
//! engine and make compilation fail (the `preg_*` function then returns
//! `false`/`null`) — a documented scope-out vs PCRE.

use regex::{Regex, RegexBuilder};

/// Parse and compile a PHP PCRE pattern. Returns `None` when the delimiters are
/// malformed or the body is not a regex the engine can build.
pub fn compile(pattern: &[u8]) -> Option<Regex> {
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

    let mut b = RegexBuilder::new(body);
    for &f in flags {
        match f {
            b'i' => {
                b.case_insensitive(true);
            }
            b'm' => {
                b.multi_line(true);
            }
            b's' => {
                b.dot_matches_new_line(true);
            }
            b'x' => {
                b.ignore_whitespace(true);
            }
            // u (unicode, already on), U/A/D/X and others: ignored.
            _ => {}
        }
    }
    b.build().ok()
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
