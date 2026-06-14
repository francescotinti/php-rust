//! Array builtins (plan step 10): count, array_keys, array_values, ...

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::numstr::{parse_numeric_ex, Num};
use php_types::{convert, ops, Key, PhpArray, PhpError, Zval};

/// A `Key` as the `Zval` array_keys/foreach expose it: ints stay ints,
/// string keys become strings.
fn key_to_zval(key: &Key) -> Zval {
    match key {
        Key::Int(i) => Zval::Long(*i),
        Key::Str(s) => Zval::Str(Rc::clone(s)),
    }
}

/// count($value, $mode = COUNT_NORMAL).
///
/// Only arrays (and Countable, unsupported here) are accepted; any scalar is a
/// `TypeError` (PHP 8 removed the old "count(scalar) == 1" leniency). `$mode`
/// COUNT_RECURSIVE (1) descends into nested arrays, counting each nested
/// container as well as its elements.
pub fn count(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let value = args.first().ok_or_else(|| {
        PhpError::Error("count() expects at least 1 argument, 0 given".to_string())
    })?;
    let arr = match value {
        Zval::Array(a) => a,
        other => {
            return Err(PhpError::TypeError(format!(
                "count(): Argument #1 ($value) must be of type Countable|array, {} given",
                other.error_type_name()
            )))
        }
    };
    let recursive = matches!(args.get(1), Some(v) if convert::to_long_cast(v, _ctx.diags) == 1);
    let n = if recursive {
        count_recursive(arr)
    } else {
        arr.len()
    };
    Ok(Zval::Long(n as i64))
}

fn count_recursive(arr: &PhpArray) -> usize {
    let mut n = arr.len();
    for (_, v) in arr.iter() {
        if let Zval::Array(inner) = v {
            n += count_recursive(inner);
        }
    }
    n
}

/// Wrap an iterator of values into a fresh 0-indexed array (list).
fn into_list(vals: impl IntoIterator<Item = Zval>) -> Zval {
    let mut out = PhpArray::new();
    for v in vals {
        // append on a fresh array only fails past i64::MAX entries.
        let _ = out.append(v);
    }
    Zval::Array(Rc::new(out))
}

/// First positional argument, required to be an array, else a `TypeError`
/// matching PHP's "Argument #1 ($array) must be of type array, X given".
fn arr_arg<'a>(args: &'a [Zval], fname: &str) -> Result<&'a PhpArray, PhpError> {
    match args.first() {
        Some(Zval::Array(a)) => Ok(a),
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($array) must be of type array, {} given",
            other.error_type_name()
        ))),
        None => Err(PhpError::Error(format!(
            "{fname}() expects at least 1 argument, 0 given"
        ))),
    }
}

/// array_keys($array, [$search, [$strict]]).
///
/// With no search value, returns every key. With a search value, returns only
/// the keys whose value matches — loosely by default, identically if `$strict`.
pub fn array_keys(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_keys")?;
    match args.get(1) {
        None => Ok(into_list(arr.iter().map(|(k, _)| key_to_zval(k)))),
        Some(search) => {
            let strict = matches!(args.get(2), Some(v) if convert::is_true_silent(v));
            let matches = arr.iter().filter(|(_, v)| {
                if strict {
                    ops::identical(v, search)
                } else {
                    ops::loose_eq(v, search)
                }
            });
            Ok(into_list(matches.map(|(k, _)| key_to_zval(k))))
        }
    }
}

/// array_values($array): the values reindexed 0..n as a list.
pub fn array_values(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_values")?;
    Ok(into_list(arr.iter().map(|(_, v)| v.clone())))
}

/// in_array($needle, $haystack[, $strict]): whether `$needle` is a value of
/// `$haystack`. Loose comparison by default, identical when `$strict` truthy.
pub fn in_array(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let needle = args.first().ok_or_else(|| {
        PhpError::Error("in_array() expects at least 2 arguments, 0 given".to_string())
    })?;
    let haystack = match args.get(1) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "in_array(): Argument #2 ($haystack) must be of type array, {} given",
                other.error_type_name()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "in_array() expects at least 2 arguments, 1 given".to_string(),
            ))
        }
    };
    let strict = matches!(args.get(2), Some(v) if convert::is_true_silent(v));
    let found = haystack.iter().any(|(_, v)| {
        if strict {
            ops::identical(v, needle)
        } else {
            ops::loose_eq(v, needle)
        }
    });
    Ok(Zval::Bool(found))
}

/// array_merge(...$arrays): concatenate arrays. Integer keys are renumbered
/// sequentially (append), string keys are kept with later values overwriting
/// earlier ones. Zero arguments yields an empty array.
pub fn array_merge(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut out = PhpArray::new();
    for (i, arg) in args.iter().enumerate() {
        let Zval::Array(a) = arg else {
            return Err(PhpError::TypeError(format!(
                "array_merge(): Argument #{} must be of type array, {} given",
                i + 1,
                arg.error_type_name()
            )));
        };
        for (k, v) in a.iter() {
            match k {
                Key::Int(_) => {
                    let _ = out.append(v.clone());
                }
                Key::Str(_) => out.insert(k.clone(), v.clone()),
            }
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

// --- by-reference builtins (step 11c) ---
//
// These receive the caller's first-argument variable as `&mut Zval` (D-R7); the
// evaluator handles the binding and the missing / non-variable first-argument
// errors, so each only needs to validate that the bound value is an array.

/// `array_push(array &$array, mixed ...$values): int` — append each value to the
/// referenced array and return its new element count.
pub fn array_push(arr: &mut Zval, values: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "array_push")?;
    let a = Rc::make_mut(rc);
    for v in values {
        a.append(v.clone()).map_err(|_| {
            PhpError::Error(
                "Cannot add element to the array as the next element is already occupied"
                    .to_string(),
            )
        })?;
    }
    Ok(Zval::Long(a.len() as i64))
}

/// `sort(array &$array, int $flags = SORT_REGULAR): true` — sort the values in
/// place with the default (SORT_REGULAR) comparison and reindex from 0, dropping
/// the original keys. `$flags` is accepted but not honoured (Tier 1 scope).
pub fn sort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "sort")?;
    let mut vals: Vec<Zval> = rc.iter().map(|(_, v)| v.clone()).collect();
    vals.sort_by(|a, b| ops::compare(a, b).cmp(&0));
    let mut out = PhpArray::new();
    for v in vals {
        let _ = out.append(v);
    }
    *arr = Zval::Array(Rc::new(out));
    Ok(Zval::Bool(true))
}

/// `array_pop(array &$array): mixed` — remove and return the last element
/// (keys of the remaining elements are left unchanged); NULL on an empty array.
pub fn array_pop(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "array_pop")?;
    let last_key = rc.iter().last().map(|(k, _)| k.clone());
    match last_key {
        Some(k) => Ok(Rc::make_mut(rc).remove(&k).unwrap_or(Zval::Null)),
        None => Ok(Zval::Null),
    }
}

/// `array_shift(array &$array): mixed` — remove and return the first element,
/// reindexing the remaining integer keys from 0 (string keys are preserved);
/// NULL on an empty array.
pub fn array_shift(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "array_shift")?;
    let mut iter = rc.iter();
    let Some((_, first_val)) = iter.next() else {
        return Ok(Zval::Null);
    };
    let shifted = first_val.clone();
    let mut out = PhpArray::new();
    for (key, val) in iter {
        match key {
            Key::Int(_) => {
                let _ = out.append(val.clone());
            }
            Key::Str(_) => out.insert(key.clone(), val.clone()),
        }
    }
    *arr = Zval::Array(Rc::new(out));
    Ok(shifted)
}

// --- step 17-5: range / slice / reverse / unique / sum ---

/// A numeric value coerced for `range`: `(value, is_float)`. `is_float` is set
/// for genuine floats (so a float start/end/step makes a float range).
fn range_num(v: &Zval) -> Option<(f64, bool)> {
    match v {
        Zval::Long(n) => Some((*n as f64, false)),
        Zval::Double(d) => Some((*d, true)),
        Zval::Bool(b) => Some((*b as i64 as f64, false)),
        Zval::Str(s) => match parse_numeric_ex(s.as_bytes(), false) {
            Some(info) if !info.trailing => Some(match info.num {
                Num::Long(n) => (n as f64, false),
                Num::Double(d) => (d, true),
            }),
            _ => None,
        },
        _ => None,
    }
}

/// range($start, $end, $step = 1): an array of values from `$start` to `$end`.
///
/// Numeric (int/float) or single-character mode is auto-detected; the result is
/// a float range if any of start/end/step is a float. Direction follows
/// start vs end; `$step` 0 is a `ValueError`, and a negative step on an
/// increasing range is a `ValueError`.
pub fn range(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let start = args.first().ok_or_else(|| {
        PhpError::Error("range() expects at least 2 arguments, 0 given".to_string())
    })?;
    let end = args.get(1).ok_or_else(|| {
        PhpError::Error("range() expects at least 2 arguments, 1 given".to_string())
    })?;

    // Signed step (default 1). `cannot be 0` and the increasing-range checks use
    // the sign; magnitude drives generation.
    let (step_val, step_float) = match args.get(2) {
        Some(v) => range_num(v).unwrap_or((1.0, false)),
        None => (1.0, false),
    };
    if step_val == 0.0 {
        return Err(PhpError::ValueError(
            "range(): Argument #3 ($step) cannot be 0".to_string(),
        ));
    }

    // Character mode: both bounds are non-numeric strings (use their first byte).
    let char_mode = matches!(start, Zval::Str(_))
        && matches!(end, Zval::Str(_))
        && range_num(start).is_none()
        && range_num(end).is_none();

    if char_mode {
        let lo = byte0(start);
        let hi = byte0(end);
        let step = step_val.abs().max(1.0) as i64;
        let mut out = PhpArray::new();
        emit_int_range(&mut out, lo as i64, hi as i64, step, |n| {
            Zval::Str(php_types::PhpStr::new(vec![n as u8]))
        });
        return Ok(Zval::Array(Rc::new(out)));
    }

    let (start_f, sf) = range_num(start).unwrap_or((0.0, false));
    let (end_f, ef) = range_num(end).unwrap_or((0.0, false));
    let is_float = sf || ef || step_float;

    let increasing = start_f <= end_f;
    if increasing && step_val < 0.0 {
        return Err(PhpError::ValueError(
            "range(): Argument #3 ($step) must be greater than 0 for increasing ranges".to_string(),
        ));
    }
    let step = step_val.abs();

    let mut out = PhpArray::new();
    if is_float {
        // Count-based generation avoids floating-point drift past the endpoint.
        let n = ((end_f - start_f).abs() / step + 1e-9).floor() as i64;
        for i in 0..=n {
            let val = if increasing {
                start_f + i as f64 * step
            } else {
                start_f - i as f64 * step
            };
            let _ = out.append(Zval::Double(val));
        }
    } else {
        emit_int_range(&mut out, start_f as i64, end_f as i64, step as i64, Zval::Long);
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// First byte of a string `Zval` (0 if empty / not a string).
fn byte0(v: &Zval) -> u8 {
    match v {
        Zval::Str(s) => s.as_bytes().first().copied().unwrap_or(0),
        _ => 0,
    }
}

/// Append `lo..=hi` (either direction) by `step` (>= 1), mapping each integer
/// through `make`.
fn emit_int_range(out: &mut PhpArray, lo: i64, hi: i64, step: i64, make: impl Fn(i64) -> Zval) {
    let step = step.max(1);
    if lo <= hi {
        let mut cur = lo;
        while cur <= hi {
            let _ = out.append(make(cur));
            cur += step;
        }
    } else {
        let mut cur = lo;
        while cur >= hi {
            let _ = out.append(make(cur));
            cur -= step;
        }
    }
}

/// array_slice($array, $offset, $length = null, $preserve_keys = false).
pub fn array_slice(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_slice")?;
    let entries: Vec<(Key, Zval)> = arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let len = entries.len() as i64;
    let offset = match args.get(1) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => 0,
    };
    let start = if offset < 0 {
        (len + offset).max(0)
    } else {
        offset.min(len)
    };
    let end = match args.get(2) {
        None | Some(Zval::Null) => len,
        Some(v) => {
            let length = convert::to_long_cast(v, ctx.diags);
            if length < 0 {
                (len + length).max(start)
            } else {
                (start + length).min(len)
            }
        }
    };
    let preserve = matches!(args.get(3), Some(v) if convert::is_true_silent(v));
    let mut out = PhpArray::new();
    for (k, v) in entries.into_iter().take(end as usize).skip(start as usize) {
        push_entry(&mut out, k, v, preserve);
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_reverse($array, $preserve_keys = false).
pub fn array_reverse(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_reverse")?;
    let preserve = matches!(args.get(1), Some(v) if convert::is_true_silent(v));
    let entries: Vec<(Key, Zval)> = arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let mut out = PhpArray::new();
    for (k, v) in entries.into_iter().rev() {
        push_entry(&mut out, k, v, preserve);
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Push `(k, v)` honouring `preserve`: when false, integer keys reindex
/// (append) while string keys are kept; when true, every key is kept verbatim.
fn push_entry(out: &mut PhpArray, k: Key, v: Zval, preserve: bool) {
    match (&k, preserve) {
        (Key::Int(_), false) => {
            let _ = out.append(v);
        }
        _ => out.insert(k, v),
    }
}

/// array_unique($array, $flags = SORT_STRING): keep the first occurrence of each
/// distinct value (compared as a string), preserving keys and order.
pub fn array_unique(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_unique")?;
    let entries: Vec<(Key, Zval)> = arr.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let mut seen: Vec<Vec<u8>> = Vec::new();
    let mut out = PhpArray::new();
    for (k, v) in entries {
        let repr = convert::to_zstr(&v, ctx.diags).as_bytes().to_vec();
        if !seen.contains(&repr) {
            seen.push(repr);
            out.insert(k, v);
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_sum($array): the sum of the values (int unless a float is involved;
/// the empty array sums to int(0)).
pub fn array_sum(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_sum")?;
    let mut acc = Zval::Long(0);
    for (_, v) in arr.iter() {
        acc = ops::add(&acc, v, ctx.diags)?;
    }
    Ok(acc)
}

/// Borrow a by-reference first argument as an array, or raise the shared
/// `Argument #1 ($array) must be of type array, {type} given` TypeError.
fn as_array_mut<'a>(arr: &'a mut Zval, fname: &str) -> Result<&'a mut Rc<PhpArray>, PhpError> {
    match arr {
        Zval::Array(rc) => Ok(rc),
        other => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #1 ($array) must be of type array, {} given",
            other.error_type_name()
        ))),
    }
}
