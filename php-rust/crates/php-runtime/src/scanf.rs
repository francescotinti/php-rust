//! C-style `sscanf`/`fscanf` engine (step 54a). Parses an input byte string
//! against a scanf format and yields one result slot per (non-suppressed)
//! conversion specifier — `None` when that conversion failed or was never
//! reached. The evaluator (`ho_sscanf`/`ho_fscanf`) turns the slots into either
//! a return array or by-reference assignments. Semantics verified byte-exact
//! against PHP 8.5.7 (e.g. `%i` auto-detects 0x/0 base, `%d` is strict decimal,
//! `%*d` consumes without producing a slot, scanning stops at the first failed
//! conversion or literal mismatch).

use php_types::{PhpStr, Zval};

fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

fn digit_val(b: u8) -> Option<u32> {
    match b {
        b'0'..=b'9' => Some((b - b'0') as u32),
        b'a'..=b'f' => Some((b - b'a' + 10) as u32),
        b'A'..=b'F' => Some((b - b'A' + 10) as u32),
        _ => None,
    }
}

/// The conversion kind behind a `%` specifier.
enum Conv {
    /// Integer. `base` 0 means C "auto" (`%i`: 0x→16, 0→8, else 10); otherwise a
    /// fixed base (`%d`/`%u`=10, `%x`=16, `%o`=8, `%b`=2).
    Int { base: u32 },
    Float,
    Str,
    Char,
    /// `%[...]` / `%[^...]` character class as inclusive byte ranges.
    Class { negate: bool, ranges: Vec<(u8, u8)> },
}

/// A parsed format directive.
enum Tok {
    /// One or more whitespace bytes in the format → match 0+ input whitespace.
    Ws,
    /// A literal byte that must match the input exactly.
    Lit(u8),
    Conv {
        suppress: bool,
        width: Option<usize>,
        conv: Conv,
    },
}

/// Parse a `%[...]` class body starting just after the `[`. Returns the ranges,
/// whether it is negated, and the index just past the closing `]`.
fn parse_class(fmt: &[u8], mut i: usize) -> (bool, Vec<(u8, u8)>, usize) {
    let negate = fmt.get(i) == Some(&b'^');
    if negate {
        i += 1;
    }
    let mut ranges = Vec::new();
    // A `]` immediately after `[`/`[^` is a literal member.
    if fmt.get(i) == Some(&b']') {
        ranges.push((b']', b']'));
        i += 1;
    }
    while i < fmt.len() && fmt[i] != b']' {
        if i + 2 < fmt.len() && fmt[i + 1] == b'-' && fmt[i + 2] != b']' {
            ranges.push((fmt[i], fmt[i + 2]));
            i += 3;
        } else {
            ranges.push((fmt[i], fmt[i]));
            i += 1;
        }
    }
    if i < fmt.len() {
        i += 1; // consume closing ']'
    }
    (negate, ranges, i)
}

fn parse_format(fmt: &[u8]) -> Vec<Tok> {
    let mut toks = Vec::new();
    let mut i = 0;
    while i < fmt.len() {
        let b = fmt[i];
        if is_ws(b) {
            while i < fmt.len() && is_ws(fmt[i]) {
                i += 1;
            }
            toks.push(Tok::Ws);
            continue;
        }
        if b != b'%' {
            toks.push(Tok::Lit(b));
            i += 1;
            continue;
        }
        // A '%' directive.
        i += 1;
        if fmt.get(i) == Some(&b'%') {
            toks.push(Tok::Lit(b'%'));
            i += 1;
            continue;
        }
        let suppress = fmt.get(i) == Some(&b'*');
        if suppress {
            i += 1;
        }
        let mut width = 0usize;
        let mut has_width = false;
        while i < fmt.len() && fmt[i].is_ascii_digit() {
            has_width = true;
            width = width.saturating_mul(10).saturating_add((fmt[i] - b'0') as usize);
            i += 1;
        }
        // Skip C length modifiers (h/l/L/q/j/z/t), which we ignore.
        while matches!(fmt.get(i), Some(b'h' | b'l' | b'L' | b'q' | b'j' | b'z' | b't')) {
            i += 1;
        }
        let Some(&ty) = fmt.get(i) else { break };
        i += 1;
        let conv = match ty {
            b'd' | b'u' => Conv::Int { base: 10 },
            b'i' => Conv::Int { base: 0 },
            b'x' | b'X' => Conv::Int { base: 16 },
            b'o' => Conv::Int { base: 8 },
            b'b' => Conv::Int { base: 2 },
            b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => Conv::Float,
            b's' => Conv::Str,
            b'c' => Conv::Char,
            b'[' => {
                let (negate, ranges, ni) = parse_class(fmt, i);
                i = ni;
                Conv::Class { negate, ranges }
            }
            // Unknown specifier: treat the type char as a literal (defensive).
            other => {
                toks.push(Tok::Lit(b'%'));
                toks.push(Tok::Lit(other));
                continue;
            }
        };
        toks.push(Tok::Conv {
            suppress,
            width: has_width.then_some(width),
            conv,
        });
    }
    toks
}

fn skip_ws(input: &[u8], i: &mut usize) {
    while *i < input.len() && is_ws(input[*i]) {
        *i += 1;
    }
}

/// Scan an integer of the given base (0 = C auto). Advances `i`; returns the
/// value or `None` if no valid digit was read.
fn scan_int(input: &[u8], i: &mut usize, base: u32, width: Option<usize>) -> Option<i64> {
    skip_ws(input, i);
    let limit = width.map(|w| (*i + w).min(input.len())).unwrap_or(input.len());
    let mut j = *i;
    let neg = match input.get(j) {
        Some(b'+') => {
            j += 1;
            false
        }
        Some(b'-') => {
            j += 1;
            true
        }
        _ => false,
    };
    let mut base = base;
    if base == 0 {
        // C `%i`: 0x→16, leading 0→8, else 10.
        if input.get(j) == Some(&b'0') && matches!(input.get(j + 1), Some(b'x' | b'X')) && j + 1 < limit
        {
            base = 16;
            j += 2;
        } else if input.get(j) == Some(&b'0') {
            base = 8;
        } else {
            base = 10;
        }
    } else if base == 16
        && input.get(j) == Some(&b'0')
        && matches!(input.get(j + 1), Some(b'x' | b'X'))
        && j + 1 < limit
    {
        j += 2; // optional 0x prefix
    }
    let digits_start = j;
    let mut acc: i64 = 0;
    while j < limit {
        match digit_val(input[j]) {
            Some(d) if d < base => {
                acc = acc.saturating_mul(base as i64).saturating_add(d as i64);
                j += 1;
            }
            _ => break,
        }
    }
    if j == digits_start {
        return None;
    }
    *i = j;
    Some(if neg { -acc } else { acc })
}

/// Scan a C-`strtod`-style float, width-limited. Advances `i`.
fn scan_float(input: &[u8], i: &mut usize, width: Option<usize>) -> Option<f64> {
    skip_ws(input, i);
    let limit = width.map(|w| (*i + w).min(input.len())).unwrap_or(input.len());
    let start = *i;
    let mut j = *i;
    if matches!(input.get(j), Some(b'+' | b'-')) {
        j += 1;
    }
    let mut saw_digit = false;
    while j < limit && input[j].is_ascii_digit() {
        j += 1;
        saw_digit = true;
    }
    if input.get(j) == Some(&b'.') && j < limit {
        j += 1;
        while j < limit && input[j].is_ascii_digit() {
            j += 1;
            saw_digit = true;
        }
    }
    if !saw_digit {
        return None;
    }
    // Optional exponent.
    if matches!(input.get(j), Some(b'e' | b'E')) && j < limit {
        let mut k = j + 1;
        if matches!(input.get(k), Some(b'+' | b'-')) {
            k += 1;
        }
        let exp_digits = k;
        while k < limit && input[k].is_ascii_digit() {
            k += 1;
        }
        if k > exp_digits {
            j = k;
        }
    }
    let s = std::str::from_utf8(&input[start..j]).ok()?;
    let v = s.parse::<f64>().ok()?;
    *i = j;
    Some(v)
}

fn in_class(b: u8, negate: bool, ranges: &[(u8, u8)]) -> bool {
    let hit = ranges.iter().any(|&(lo, hi)| b >= lo && b <= hi);
    hit != negate
}

/// Attempt a single conversion. Advances `i`; returns the value or `None`.
fn scan_one(conv: &Conv, width: Option<usize>, input: &[u8], i: &mut usize) -> Option<Zval> {
    match conv {
        Conv::Int { base } => scan_int(input, i, *base, width).map(Zval::Long),
        Conv::Float => scan_float(input, i, width).map(Zval::Double),
        Conv::Str => {
            skip_ws(input, i);
            let limit = width.map(|w| (*i + w).min(input.len())).unwrap_or(input.len());
            let start = *i;
            while *i < limit && !is_ws(input[*i]) {
                *i += 1;
            }
            if *i == start {
                return None;
            }
            Some(Zval::Str(PhpStr::new(input[start..*i].to_vec())))
        }
        Conv::Char => {
            // No whitespace skip; reads exactly `width` bytes (default 1).
            let n = width.unwrap_or(1);
            let end = (*i + n).min(input.len());
            if end == *i {
                return None;
            }
            let out = input[*i..end].to_vec();
            *i = end;
            Some(Zval::Str(PhpStr::new(out)))
        }
        Conv::Class { negate, ranges } => {
            let limit = width.map(|w| (*i + w).min(input.len())).unwrap_or(input.len());
            let start = *i;
            while *i < limit && in_class(input[*i], *negate, ranges) {
                *i += 1;
            }
            if *i == start {
                return None;
            }
            Some(Zval::Str(PhpStr::new(input[start..*i].to_vec())))
        }
    }
}

/// Run the scanf engine. Returns one slot per non-suppressed conversion in
/// format order: `Some(value)` for a successful conversion, `None` once a
/// conversion fails or a literal directive mismatches (all later conversions
/// are `None` too). The result length always equals the number of
/// non-suppressed conversions in `fmt`.
pub fn run_scanf(input: &[u8], fmt: &[u8]) -> Vec<Option<Zval>> {
    let toks = parse_format(fmt);
    let mut results = Vec::new();
    let mut i = 0usize;
    let mut stopped = false;
    for tok in &toks {
        match tok {
            Tok::Ws => {
                if !stopped {
                    skip_ws(input, &mut i);
                }
            }
            Tok::Lit(b) => {
                if !stopped {
                    if input.get(i) == Some(b) {
                        i += 1;
                    } else {
                        stopped = true;
                    }
                }
            }
            Tok::Conv {
                suppress,
                width,
                conv,
            } => {
                if stopped {
                    if !suppress {
                        results.push(None);
                    }
                    continue;
                }
                let v = scan_one(conv, *width, input, &mut i);
                if v.is_none() {
                    stopped = true;
                }
                if !suppress {
                    results.push(v);
                }
            }
        }
    }
    results
}
