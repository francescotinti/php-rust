//! Adapter over oniguruma (the `onig` crate) for PHP's `mb_ereg*` family
//! (step 43). PHP backs mbregex with oniguruma; this module is a thin Strategy-A
//! adapter that compiles patterns with mbstring's default dialect (Ruby syntax +
//! the `"pr"` options) and exposes match / replace / split as owned results so no
//! `onig` borrow escapes into the evaluator.
//!
//! Scope: UTF-8 only (the default `mb_regex_encoding`); non-UTF-8 mbregex
//! encodings are out of scope (D-MB-ereg-enc, consistent with D-MB1).

use onig::{Regex, RegexOptions, SearchOptions, Syntax};
use php_types::{Key, PhpArray, PhpStr, Zval};
use std::rc::Rc;

/// Persistent per-run mbregex state, held by the evaluator. Carries the global
/// regex encoding + options and (step 43b) the `mb_ereg_search` cursor.
pub struct MbRegexState {
    pub encoding: Vec<u8>,
    pub options: Vec<u8>,
    /// The string handed to `mb_ereg_search_init`, searched by the cursor.
    pub search_str: Vec<u8>,
    /// Current byte offset of the search cursor.
    pub search_pos: usize,
    /// Pattern compiled by `mb_ereg_search_init` for the running search session.
    pub search_re: Option<Regex>,
    /// Group ranges (byte offsets) of the last successful search, for
    /// `mb_ereg_search_getregs`/`getpos`.
    pub last_regs: Option<Zval>,
}

impl Default for MbRegexState {
    fn default() -> Self {
        MbRegexState {
            encoding: b"UTF-8".to_vec(),
            options: b"pr".to_vec(),
            search_str: Vec::new(),
            search_pos: 0,
            search_re: None,
            last_regs: None,
        }
    }
}

/// Translate a PHP mbregex option string into oniguruma compile options + syntax.
/// PHP's default is `"pr"`: `p` = MULTILINE|SINGLELINE (`.` matches newline, `^`/`$`
/// anchor the whole string), `r` = Ruby syntax. Unknown chars are ignored.
fn parse_options(opts: &[u8]) -> (RegexOptions, &'static Syntax) {
    let mut o = RegexOptions::REGEX_OPTION_NONE;
    let mut syntax = Syntax::ruby();
    for &c in opts {
        match c {
            b'i' => o |= RegexOptions::REGEX_OPTION_IGNORECASE,
            b'x' => o |= RegexOptions::REGEX_OPTION_EXTEND,
            b'm' => o |= RegexOptions::REGEX_OPTION_MULTILINE,
            b's' => o |= RegexOptions::REGEX_OPTION_SINGLELINE,
            b'p' => {
                o |= RegexOptions::REGEX_OPTION_MULTILINE | RegexOptions::REGEX_OPTION_SINGLELINE
            }
            b'l' => o |= RegexOptions::REGEX_OPTION_FIND_LONGEST,
            b'n' => o |= RegexOptions::REGEX_OPTION_FIND_NOT_EMPTY,
            b'j' => syntax = Syntax::java(),
            b'u' => syntax = Syntax::gnu_regex(),
            b'g' => syntax = Syntax::grep(),
            b'c' => syntax = Syntax::emacs(),
            b'r' => syntax = Syntax::ruby(),
            b'z' => syntax = Syntax::perl(),
            b'b' => syntax = Syntax::posix_basic(),
            b'd' => syntax = Syntax::posix_extended(),
            _ => {}
        }
    }
    (o, syntax)
}

/// Compile `pattern` under `opts` (mbstring default when empty); `extra_ic`
/// forces case-insensitivity for `mb_eregi*`. On failure returns the oniguruma
/// error message (the text PHP shows after `mbregex compile err:`).
pub fn compile(pattern: &[u8], opts: &[u8], extra_ic: bool) -> Result<Regex, String> {
    let opts = if opts.is_empty() { b"pr" } else { opts };
    let (mut o, syntax) = parse_options(opts);
    if extra_ic {
        o |= RegexOptions::REGEX_OPTION_IGNORECASE;
    }
    let pat = String::from_utf8_lossy(pattern);
    Regex::with_options(&pat, o, syntax).map_err(|e| e.description().to_string())
}

/// Build the `$regs` array of a match: numbered groups 0..n (a non-participating
/// group is `false`, oracle-exact), then named groups appended with string keys.
fn build_regs(re: &Regex, caps: &onig::Captures) -> Zval {
    let mut arr = PhpArray::new();
    for i in 0..caps.len() {
        let v = match caps.at(i) {
            Some(s) => Zval::Str(PhpStr::new(s.as_bytes().to_vec())),
            None => Zval::Bool(false),
        };
        arr.insert(Key::Int(i as i64), v);
    }
    re.foreach_name(|name, nums| {
        if let Some(&g) = nums.first() {
            let v = match caps.at(g as usize) {
                Some(s) => Zval::Str(PhpStr::new(s.as_bytes().to_vec())),
                None => Zval::Bool(false),
            };
            arr.insert(Key::from_bytes(name.as_bytes()), v);
        }
        true
    });
    Zval::Array(Rc::new(arr))
}

/// Search `subject` for the first match. Returns the `$regs` array, or `None`.
pub fn exec(re: &Regex, subject: &[u8]) -> Option<Zval> {
    let subj = String::from_utf8_lossy(subject).into_owned();
    re.captures(&subj).map(|caps| build_regs(re, &caps))
}

/// Every match in `subject` as `(start, end, regs)` byte offsets plus its
/// `$regs` array, for `mb_ereg_replace_callback` (the ranges are collected up
/// front so the `onig` borrow ends before the evaluator calls back).
pub fn find_all(re: &Regex, subject: &[u8]) -> Vec<(usize, usize, Zval)> {
    let subj = String::from_utf8_lossy(subject).into_owned();
    re.captures_iter(&subj)
        .map(|caps| {
            let (start, end) = caps.pos(0).unwrap();
            (start, end, build_regs(re, &caps))
        })
        .collect()
}

/// `mb_ereg_match`: does `re` match anchored at the start of `subject`?
pub fn matches_at_start(re: &Regex, subject: &[u8]) -> bool {
    let subj = String::from_utf8_lossy(subject).into_owned();
    re.match_with_options(&subj, 0, SearchOptions::SEARCH_OPTION_NONE, None)
        .is_some()
}

/// Apply a replacement template across every match. Backreferences `\0`..`\9`
/// (single backslash) expand to the matched group (`\0` = whole match; a
/// non-participating group expands to nothing); `\\` is a literal backslash.
pub fn replace(re: &Regex, template: &[u8], subject: &[u8]) -> Vec<u8> {
    let subj = String::from_utf8_lossy(subject).into_owned();
    let bytes = subj.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut last = 0usize;
    for caps in re.captures_iter(&subj) {
        let (start, end) = caps.pos(0).unwrap();
        out.extend_from_slice(&bytes[last..start]);
        expand_template(template, &caps, &mut out);
        last = end;
        // Guard against zero-width matches looping forever (advance one byte).
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
    out
}

/// Expand one replacement template against `caps` into `out`.
fn expand_template(template: &[u8], caps: &onig::Captures, out: &mut Vec<u8>) {
    let mut i = 0;
    while i < template.len() {
        let b = template[i];
        if b == b'\\' && i + 1 < template.len() {
            let n = template[i + 1];
            if n.is_ascii_digit() {
                let g = (n - b'0') as usize;
                if let Some(s) = caps.at(g) {
                    out.extend_from_slice(s.as_bytes());
                }
                i += 2;
                continue;
            } else if n == b'\\' {
                out.push(b'\\');
                i += 2;
                continue;
            }
        }
        out.push(b);
        i += 1;
    }
}

/// `mb_split`: split `subject` on matches of `re`, up to `limit` pieces
/// (`limit <= 0` means unlimited). Empty fields are preserved (oracle-exact).
pub fn split(re: &Regex, subject: &[u8], limit: i64) -> Vec<Vec<u8>> {
    let subj = String::from_utf8_lossy(subject).into_owned();
    let bytes = subj.as_bytes();
    let mut out: Vec<Vec<u8>> = Vec::new();
    let mut last = 0usize;
    for (start, end) in re.find_iter(&subj) {
        if limit > 0 && out.len() as i64 == limit - 1 {
            break;
        }
        if end == start {
            // Skip zero-width separators (mirror PHP: no empty-on-empty split).
            continue;
        }
        out.push(bytes[last..start].to_vec());
        last = end;
    }
    out.push(bytes[last..].to_vec());
    out
}

/// Advance the search cursor: find the next match at/after byte offset `from`.
/// Returns `(start, end, regs)` byte offsets and the `$regs` array (built from
/// the match region so group offsets are absolute), or `None` at the end.
pub fn search_from(re: &Regex, subject: &[u8], from: usize) -> Option<(usize, usize, Zval)> {
    let subj = String::from_utf8_lossy(subject).into_owned();
    if from > subj.len() {
        return None;
    }
    let mut region = onig::Region::new();
    re.search_with_options(
        &subj,
        from,
        subj.len(),
        SearchOptions::SEARCH_OPTION_NONE,
        Some(&mut region),
    )?;
    let (start, end) = region.pos(0)?;
    Some((start, end, regs_from_region(re, &region, subj.as_bytes())))
}

/// Build a `$regs` array from a match region (numbered groups 0..n with `false`
/// for a non-participating group, then named groups by string key).
fn regs_from_region(re: &Regex, region: &onig::Region, bytes: &[u8]) -> Zval {
    let group = |i: usize| match region.pos(i) {
        Some((s, e)) => Zval::Str(PhpStr::new(bytes[s..e].to_vec())),
        None => Zval::Bool(false),
    };
    let mut arr = PhpArray::new();
    for i in 0..region.len() {
        arr.insert(Key::Int(i as i64), group(i));
    }
    re.foreach_name(|name, nums| {
        if let Some(&g) = nums.first() {
            arr.insert(Key::from_bytes(name.as_bytes()), group(g as usize));
        }
        true
    });
    Zval::Array(Rc::new(arr))
}
