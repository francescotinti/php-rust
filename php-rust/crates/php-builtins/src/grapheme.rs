//! `grapheme_*` builtins (ext/intl) over Unicode extended grapheme clusters
//! (UAX #29), matching ICU — the segmentation PHP uses. The `unicode-segmentation`
//! crate provides the same cluster boundaries (`graphemes(extended = true)`).
//!
//! Input is interpreted as UTF-8. Invalid UTF-8 that is not pure ASCII fails the
//! UTF-16 conversion PHP performs, so those functions return `null`/`false` there
//! (mirroring `intl_convert_utf8_to_utf16` failure). Case-insensitive variants
//! use Unicode default case folding (`to_lowercase`), matching ICU for the common
//! (ASCII / Latin) cases.

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};
use unicode_segmentation::UnicodeSegmentation;

/// Fetch positional string arg `idx`, coerced to bytes.
fn bytes_at(args: &[Zval], ctx: &mut Ctx, idx: usize) -> Vec<u8> {
    args.get(idx)
        .map(|v| convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
        .unwrap_or_default()
}

/// The `(byte_offset, grapheme)` pairs of `s`, plus the total grapheme count.
fn grapheme_offsets(s: &str) -> Vec<(usize, &str)> {
    s.grapheme_indices(true).collect()
}

/// `grapheme_strlen(string $string): int|false|null` — number of grapheme
/// clusters. Pure ASCII is the byte length; invalid UTF-8 yields `null`.
pub fn grapheme_strlen(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let bytes = bytes_at(args, ctx, 0);
    if bytes.is_ascii() {
        return Ok(Zval::Long(bytes.len() as i64));
    }
    match std::str::from_utf8(&bytes) {
        Ok(s) => Ok(Zval::Long(s.graphemes(true).count() as i64)),
        Err(_) => Ok(Zval::Null),
    }
}

/// `grapheme_substr(string $string, int $start, ?int $length = null): string|false`
/// — substring by grapheme cluster. Negative `$start` counts from the end
/// (clamped to the start); negative `$length` stops that many graphemes before
/// the end; an out-of-range `$start` yields `""`.
pub fn grapheme_substr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let bytes = bytes_at(args, ctx, 0);
    let start = args.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    let length = match args.get(2) {
        None | Some(Zval::Null) => None,
        Some(v) => Some(convert::to_long_cast(v, ctx.diags)),
    };
    let Ok(s) = std::str::from_utf8(&bytes) else {
        return Ok(Zval::Bool(false));
    };
    let g: Vec<&str> = s.graphemes(true).collect();
    let n = g.len() as i64;
    let s_idx = if start >= 0 { start.min(n) } else { (n + start).max(0) };
    let e_idx = match length {
        None => n,
        Some(l) if l >= 0 => (s_idx + l).min(n),
        Some(l) => (n + l).max(s_idx),
    };
    let out: String = g[s_idx as usize..e_idx as usize].concat();
    Ok(Zval::Str(PhpStr::new(out.into_bytes())))
}

/// `grapheme_str_split(string $string, int $length = 1): array|false` — split
/// into chunks of `$length` grapheme clusters.
pub fn grapheme_str_split(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let bytes = bytes_at(args, ctx, 0);
    let length = args.get(1).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(1);
    // PHP bounds $length to 1..=0x3FFFFFFF.
    if length < 1 || length > 0x3FFF_FFFF {
        return Err(PhpError::ValueError(
            "grapheme_str_split(): Argument #2 ($length) must be greater than 0 and less than or equal to 1073741823"
                .to_string(),
        ));
    }
    let Ok(s) = std::str::from_utf8(&bytes) else {
        return Ok(Zval::Bool(false));
    };
    let g: Vec<&str> = s.graphemes(true).collect();
    let mut out = PhpArray::new();
    for chunk in g.chunks(length as usize) {
        let _ = out.append(Zval::Str(PhpStr::new(chunk.concat().into_bytes())));
    }
    Ok(Zval::Array(std::rc::Rc::new(out)))
}

/// Validate the `$offset` argument shared by the position functions: it must fit
/// within `±byte_len` (C `OUTSIDE_STRING`), else a `ValueError`.
fn check_offset(offset: i64, byte_len: usize, func: &str) -> Result<(), PhpError> {
    let bl = byte_len as i64;
    if offset > bl || offset < -bl {
        return Err(PhpError::ValueError(format!(
            "{func}(): Argument #3 ($offset) must be contained in argument #1 ($haystack)"
        )));
    }
    Ok(())
}

/// Core of `grapheme_strpos`/`grapheme_stripos`: the grapheme index of the first
/// occurrence of `needle` at or after grapheme offset (from `$offset`), or `None`.
fn strpos_core(hay: &str, needle: &[u8], offset: i64, ci: bool) -> Option<i64> {
    let offs = grapheme_offsets(hay);
    let gcount = offs.len() as i64;
    let start_g = if offset >= 0 { offset } else { gcount + offset };
    if start_g > gcount {
        return None;
    }
    let start_g = start_g.max(0) as usize;
    // Empty needle matches at the start position (PHP returns the offset).
    if needle.is_empty() {
        return Some(start_g as i64);
    }
    let hay_bytes = hay.as_bytes();
    let needle_lc;
    let needle_cmp: &[u8] = if ci {
        needle_lc = lower_bytes(needle);
        &needle_lc
    } else {
        needle
    };
    for (i, &(byte_off, _)) in offs.iter().enumerate().skip(start_g) {
        let tail = &hay_bytes[byte_off..];
        let matched = if ci {
            lower_prefix_eq(tail, needle_cmp)
        } else {
            tail.starts_with(needle_cmp)
        };
        if matched {
            return Some(i as i64);
        }
    }
    None
}

/// Lowercase a byte slice as UTF-8 (Unicode default folding); invalid UTF-8 is
/// lowercased ASCII-only.
fn lower_bytes(b: &[u8]) -> Vec<u8> {
    match std::str::from_utf8(b) {
        Ok(s) => s.to_lowercase().into_bytes(),
        Err(_) => b.to_ascii_lowercase(),
    }
}

/// Whether `tail`, once lowercased, begins with the already-lowercased `needle_lc`.
fn lower_prefix_eq(tail: &[u8], needle_lc: &[u8]) -> bool {
    match std::str::from_utf8(tail) {
        Ok(s) => s.to_lowercase().as_bytes().starts_with(needle_lc),
        Err(_) => tail.to_ascii_lowercase().starts_with(needle_lc),
    }
}

fn strpos_impl(args: &[Zval], ctx: &mut Ctx, ci: bool, func: &str) -> Result<Zval, PhpError> {
    let hay = bytes_at(args, ctx, 0);
    let needle = bytes_at(args, ctx, 1);
    let offset = args.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    check_offset(offset, hay.len(), func)?;
    let Ok(s) = std::str::from_utf8(&hay) else {
        return Ok(Zval::Bool(false));
    };
    match strpos_core(s, &needle, offset, ci) {
        Some(i) => Ok(Zval::Long(i)),
        None => Ok(Zval::Bool(false)),
    }
}

/// `grapheme_strpos(string $haystack, string $needle, int $offset = 0): int|false`.
pub fn grapheme_strpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, false, "grapheme_strpos")
}
/// `grapheme_stripos(...)` — case-insensitive `grapheme_strpos`.
pub fn grapheme_stripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strpos_impl(args, ctx, true, "grapheme_stripos")
}

fn strrpos_impl(args: &[Zval], ctx: &mut Ctx, ci: bool, func: &str) -> Result<Zval, PhpError> {
    let hay = bytes_at(args, ctx, 0);
    let needle = bytes_at(args, ctx, 1);
    let offset = args.get(2).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(0);
    check_offset(offset, hay.len(), func)?;
    let Ok(s) = std::str::from_utf8(&hay) else {
        return Ok(Zval::Bool(false));
    };
    let offs = grapheme_offsets(s);
    let gcount = offs.len() as i64;
    let start_g = if offset >= 0 { offset } else { gcount + offset }.max(0) as usize;
    let hay_bytes = s.as_bytes();
    let needle_lc = if ci { lower_bytes(&needle) } else { needle.clone() };
    if needle.is_empty() {
        return Ok(Zval::Long(gcount));
    }
    let mut found: Option<i64> = None;
    for (i, &(byte_off, _)) in offs.iter().enumerate().skip(start_g) {
        let tail = &hay_bytes[byte_off..];
        let matched = if ci { lower_prefix_eq(tail, &needle_lc) } else { tail.starts_with(&needle_lc) };
        if matched {
            found = Some(i as i64);
        }
    }
    Ok(found.map(Zval::Long).unwrap_or(Zval::Bool(false)))
}

/// `grapheme_strrpos(...)` — grapheme index of the *last* occurrence.
pub fn grapheme_strrpos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strrpos_impl(args, ctx, false, "grapheme_strrpos")
}
/// `grapheme_strripos(...)` — case-insensitive `grapheme_strrpos`.
pub fn grapheme_strripos(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strrpos_impl(args, ctx, true, "grapheme_strripos")
}

fn strstr_impl(args: &[Zval], ctx: &mut Ctx, ci: bool) -> Result<Zval, PhpError> {
    let hay = bytes_at(args, ctx, 0);
    let needle = bytes_at(args, ctx, 1);
    let before = args.get(2).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    let Ok(s) = std::str::from_utf8(&hay) else {
        return Ok(Zval::Bool(false));
    };
    match strpos_core(s, &needle, 0, ci) {
        Some(i) => {
            let g: Vec<&str> = s.graphemes(true).collect();
            let out: String = if before {
                g[..i as usize].concat()
            } else {
                g[i as usize..].concat()
            };
            Ok(Zval::Str(PhpStr::new(out.into_bytes())))
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// `grapheme_strstr(string $haystack, string $needle, bool $beforeNeedle = false): string|false`.
pub fn grapheme_strstr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, false)
}
/// `grapheme_stristr(...)` — case-insensitive `grapheme_strstr`.
pub fn grapheme_stristr(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strstr_impl(args, ctx, true)
}

/// Validate a `grapheme_levenshtein` cost argument: it must be in `1..=0x3FFFFFFF`.
fn levenshtein_cost(
    args: &[Zval],
    ctx: &mut Ctx,
    idx: usize,
    arg_num: usize,
    name: &str,
) -> Result<i64, PhpError> {
    let c = args.get(idx).map(|v| convert::to_long_cast(v, ctx.diags)).unwrap_or(1);
    if c < 1 || c > 0x3FFF_FFFF {
        return Err(PhpError::ValueError(format!(
            "grapheme_levenshtein(): Argument #{arg_num} (${name}) must be greater than 0 and less than or equal to 1073741823"
        )));
    }
    Ok(c)
}

/// `grapheme_levenshtein(string $string1, string $string2, int $insertion_cost = 1,
/// int $replacement_cost = 1, int $deletion_cost = 1): int` (PHP 8.5) — Levenshtein
/// edit distance measured in **grapheme clusters** (a ZWJ emoji sequence is one
/// unit), with per-operation costs. Standard DP, so a cheap delete+insert can
/// beat an expensive replacement.
pub fn grapheme_levenshtein(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let b1 = bytes_at(args, ctx, 0);
    let b2 = bytes_at(args, ctx, 1);
    let cost_ins = levenshtein_cost(args, ctx, 2, 3, "insertion_cost")?;
    let cost_rep = levenshtein_cost(args, ctx, 3, 4, "replacement_cost")?;
    let cost_del = levenshtein_cost(args, ctx, 4, 5, "deletion_cost")?;
    let (Ok(s1), Ok(s2)) = (std::str::from_utf8(&b1), std::str::from_utf8(&b2)) else {
        return Ok(Zval::Bool(false));
    };
    let g1: Vec<&str> = s1.graphemes(true).collect();
    let g2: Vec<&str> = s2.graphemes(true).collect();
    let m = g2.len();
    // Row j of the DP is the distance transforming an empty prefix of g1 into
    // g2[..j] (all insertions); each row down deletes one grapheme of g1.
    let mut prev: Vec<i64> = (0..=m).map(|j| j as i64 * cost_ins).collect();
    for gi in &g1 {
        let mut cur = vec![0i64; m + 1];
        cur[0] = prev[0] + cost_del;
        for j in 1..=m {
            let sub = if gi == &g2[j - 1] { prev[j - 1] } else { prev[j - 1] + cost_rep };
            cur[j] = sub.min(prev[j] + cost_del).min(cur[j - 1] + cost_ins);
        }
        prev = cur;
    }
    Ok(Zval::Long(prev[m]))
}
