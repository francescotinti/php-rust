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
                other.type_name_for_error()
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
            other.type_name_for_error()
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
                other.type_name_for_error()
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
                arg.type_name_for_error()
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

/// array_merge_recursive(...$arrays): like [`array_merge`], but a string key
/// present on both sides *merges* instead of overwriting: each side is coerced
/// to array form (a non-array value wraps as `[v]`) and the two are merged
/// recursively — so scalars accumulate (`['k'=>'x'] + ['k'=>'y']` →
/// `['k'=>['x','y']]`) and nested arrays deep-merge. Integer keys append with
/// renumbering, exactly like `array_merge`.
pub fn array_merge_recursive(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    fn to_arr(v: &Zval) -> PhpArray {
        match v.deref_clone() {
            Zval::Array(a) => (*a).clone(),
            other => {
                let mut w = PhpArray::new();
                let _ = w.append(other);
                w
            }
        }
    }
    fn merge_into(out: &mut PhpArray, src: &PhpArray) {
        for (k, v) in src.iter() {
            match k {
                Key::Int(_) => {
                    let _ = out.append(v.clone());
                }
                Key::Str(_) => match out.get(k) {
                    Some(existing) => {
                        let mut merged = to_arr(existing);
                        merge_into(&mut merged, &to_arr(v));
                        out.insert(k.clone(), Zval::Array(Rc::new(merged)));
                    }
                    None => out.insert(k.clone(), v.clone()),
                },
            }
        }
    }
    let mut out = PhpArray::new();
    for (i, arg) in args.iter().enumerate() {
        let Zval::Array(a) = arg.deref_clone() else {
            return Err(PhpError::TypeError(format!(
                "array_merge_recursive(): Argument #{} must be of type array, {} given",
                i + 1,
                arg.type_name_for_error()
            )));
        };
        merge_into(&mut out, &a);
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Build the "Argument #N must be of type array" TypeError shared by the
/// array_replace family (mirrors array_merge's wording).
fn replace_arg_err(func: &str, n: usize, arg: &Zval) -> PhpError {
    PhpError::TypeError(format!(
        "{func}(): Argument #{n} must be of type array, {} given",
        arg.type_name_for_error()
    ))
}

/// array_replace($array, ...$replacements): replace entries of the first array
/// by key with those of later arrays. Unlike array_merge, integer keys are NOT
/// renumbered — every key (int or string) is matched and overwritten in place,
/// preserving the first array's order; keys absent from it are appended. Later
/// arrays win over earlier ones.
pub fn array_replace(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut out = match args.first() {
        Some(Zval::Array(a)) => (**a).clone(),
        Some(other) => return Err(replace_arg_err("array_replace", 1, other)),
        None => {
            return Err(PhpError::Error(
                "array_replace() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };
    for (i, arg) in args.iter().enumerate().skip(1) {
        let Zval::Array(a) = arg else {
            return Err(replace_arg_err("array_replace", i + 1, arg));
        };
        for (k, v) in a.iter() {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_replace_recursive($array, ...$replacements): like array_replace, but
/// when a key holds an array on *both* the current result and a replacement,
/// the two are merged recursively rather than replaced wholesale. If either
/// side is a non-array, the replacement value wins outright (PHP semantics).
pub fn array_replace_recursive(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let mut out = match args.first() {
        Some(Zval::Array(a)) => (**a).clone(),
        Some(other) => return Err(replace_arg_err("array_replace_recursive", 1, other)),
        None => {
            return Err(PhpError::Error(
                "array_replace_recursive() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };
    for (i, arg) in args.iter().enumerate().skip(1) {
        let Zval::Array(a) = arg else {
            return Err(replace_arg_err("array_replace_recursive", i + 1, arg));
        };
        replace_recursive_into(&mut out, a);
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Merge `repl` into `base` following array_replace_recursive semantics: a key
/// that is an array on both sides recurses; anything else overwrites in place.
fn replace_recursive_into(base: &mut PhpArray, repl: &PhpArray) {
    for (k, v) in repl.iter() {
        let new_val = match (base.get(k), v) {
            (Some(Zval::Array(existing)), Zval::Array(incoming)) => {
                let mut merged = (**existing).clone();
                replace_recursive_into(&mut merged, incoming);
                Zval::Array(Rc::new(merged))
            }
            _ => v.clone(),
        };
        base.insert(k.clone(), new_val);
    }
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
/// settype(&$var, string $type): convert `$var` to `$type` in place, returning
/// `true`. The coercion rules mirror the corresponding cast; an unknown type name
/// is a `ValueError`. (`object` is not yet supported — it needs VM state to mint a
/// stdClass — and reports the invalid-type error.)
pub fn settype(var: &mut Zval, args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let ty = args.first().ok_or_else(|| {
        PhpError::ArgumentCountError("settype() expects exactly 2 arguments, 1 given".into())
    })?;
    let lc = convert::to_zstr(ty, ctx.diags).as_bytes().to_ascii_lowercase();
    let new = match lc.as_slice() {
        b"int" | b"integer" => Zval::Long(convert::to_long_cast(var, ctx.diags)),
        b"float" | b"double" => Zval::Double(convert::to_double(var)),
        b"string" => Zval::Str(convert::to_zstr_cast(var, ctx.diags)),
        b"bool" | b"boolean" => Zval::Bool(convert::to_bool(var, ctx.diags)),
        b"array" => match var.deref_clone() {
            arr @ Zval::Array(_) => arr,
            Zval::Null | Zval::Undef => Zval::Array(Rc::new(PhpArray::new())),
            scalar => {
                let mut arr = PhpArray::new();
                arr.insert(Key::Int(0), scalar);
                Zval::Array(Rc::new(arr))
            }
        },
        b"null" => Zval::Null,
        _ => {
            return Err(PhpError::ValueError(
                "settype(): Argument #2 ($type) must be a valid type".into(),
            ))
        }
    };
    *var = new;
    Ok(Zval::Bool(true))
}

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

/// `rsort(array &$array, int $flags = SORT_REGULAR): true` — like [`sort`] but
/// descending. Reindexes from 0, dropping the original keys.
pub fn rsort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "rsort")?;
    let mut vals: Vec<Zval> = rc.iter().map(|(_, v)| v.clone()).collect();
    vals.sort_by(|a, b| ops::compare(b, a).cmp(&0));
    let mut out = PhpArray::new();
    for v in vals {
        let _ = out.append(v);
    }
    *arr = Zval::Array(Rc::new(out));
    Ok(Zval::Bool(true))
}

/// Sort the `(key, value)` pairs in place, preserving the key→value association,
/// and rebuild the array in the sorted order. `cmp` compares two pairs. Shared by
/// `asort`/`arsort`/`ksort`/`krsort` (the association-preserving sorts).
fn sort_assoc<F>(arr: &mut Zval, fname: &str, mut cmp: F) -> Result<Zval, PhpError>
where
    F: FnMut(&(Key, Zval), &(Key, Zval)) -> std::cmp::Ordering,
{
    let rc = as_array_mut(arr, fname)?;
    let mut pairs: Vec<(Key, Zval)> = rc.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    pairs.sort_by(|a, b| cmp(a, b));
    let mut out = PhpArray::new();
    for (k, v) in pairs {
        out.insert(k, v);
    }
    *arr = Zval::Array(Rc::new(out));
    Ok(Zval::Bool(true))
}

/// `asort(array &$array, int $flags = SORT_REGULAR): true` — sort by value
/// ascending, preserving keys.
pub fn asort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    sort_assoc(arr, "asort", |a, b| ops::compare(&a.1, &b.1).cmp(&0))
}

/// `arsort(array &$array, int $flags = SORT_REGULAR): true` — sort by value
/// descending, preserving keys.
pub fn arsort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    sort_assoc(arr, "arsort", |a, b| ops::compare(&b.1, &a.1).cmp(&0))
}

/// `ksort(array &$array, int $flags = SORT_REGULAR): true` — sort by key
/// ascending, preserving the key→value association.
pub fn ksort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    sort_assoc(arr, "ksort", |a, b| {
        ops::compare(&key_to_zval(&a.0), &key_to_zval(&b.0)).cmp(&0)
    })
}

/// `krsort(array &$array, int $flags = SORT_REGULAR): true` — sort by key
/// descending, preserving the key→value association.
pub fn krsort(arr: &mut Zval, _args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    sort_assoc(arr, "krsort", |a, b| {
        ops::compare(&key_to_zval(&b.0), &key_to_zval(&a.0)).cmp(&0)
    })
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

/// `array_unshift(array &$array, mixed ...$values): int` — prepend `$values` to
/// the front of the referenced array, reindexing integer keys from 0 (string
/// keys are preserved, after the prepended values), returning the new count.
pub fn array_unshift(arr: &mut Zval, values: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "array_unshift")?;
    let mut out = PhpArray::new();
    for v in values {
        let _ = out.append(v.clone());
    }
    for (key, val) in rc.iter() {
        match key {
            Key::Int(_) => {
                let _ = out.append(val.clone());
            }
            Key::Str(_) => out.insert(key.clone(), val.clone()),
        }
    }
    let n = out.len() as i64;
    *arr = Zval::Array(Rc::new(out));
    Ok(Zval::Long(n))
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
            other.type_name_for_error()
        ))),
    }
}

/// array_splice(&$array, $offset, $length = null, $replacement = []): remove the
/// positional slice and splice in `$replacement` (by-ref, step 32). Returns the
/// removed elements (reindexed). Integer keys in the result are renumbered;
/// string keys of the kept elements are preserved.
pub fn array_splice(arr: &mut Zval, args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rc = as_array_mut(arr, "array_splice")?;
    let entries: Vec<(Key, Zval)> = rc.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let len = entries.len() as i64;

    let offset_raw = args
        .first()
        .map(|v| convert::to_long_cast(v, ctx.diags))
        .unwrap_or(0);
    let offset = if offset_raw < 0 {
        (len + offset_raw).max(0)
    } else {
        offset_raw.min(len)
    };

    let end = match args.get(1) {
        None | Some(Zval::Null) => len,
        Some(v) => {
            let l = convert::to_long_cast(v, ctx.diags);
            if l < 0 {
                (len + l).max(offset)
            } else {
                (offset + l).min(len)
            }
        }
    };

    let replacement: Vec<Zval> = match args.get(2) {
        Some(Zval::Array(r)) => r.iter().map(|(_, v)| v.clone()).collect(),
        Some(other) => vec![other.clone()],
        None => Vec::new(),
    };

    let (offset, end) = (offset as usize, end as usize);
    let mut removed = PhpArray::new();
    let mut out = PhpArray::new();
    // Elements before the cut keep their key kind (string preserved, int renumbered).
    for (k, v) in &entries[..offset] {
        push_kept(&mut out, k, v);
    }
    for v in replacement {
        let _ = out.append(v);
    }
    for (k, v) in &entries[end..] {
        push_kept(&mut out, k, v);
    }
    for (_, v) in &entries[offset..end] {
        let _ = removed.append(v.clone());
    }

    *arr = Zval::Array(Rc::new(out));
    Ok(Zval::Array(Rc::new(removed)))
}

/// Re-add a kept element: preserve a string key, renumber an integer key.
fn push_kept(out: &mut PhpArray, k: &Key, v: &Zval) {
    match k {
        Key::Str(_) => out.insert(k.clone(), v.clone()),
        Key::Int(_) => {
            let _ = out.append(v.clone());
        }
    }
}

// --- Step 29-2: pure array builtins ----------------------------------------

/// Coerce a value to an array key with PHP's rules (int|bool|float -> int key;
/// null -> "" key; numeric strings normalize).
fn zval_to_key(v: &Zval, ctx: &mut Ctx) -> Key {
    match v {
        Zval::Long(i) => Key::Int(*i),
        Zval::Bool(b) => Key::Int(*b as i64),
        Zval::Double(d) => Key::Int(*d as i64),
        Zval::Null => Key::Str(php_types::PhpStr::new(Vec::new())),
        Zval::Str(s) => Key::from_zstr(s),
        other => Key::from_bytes(convert::to_zstr(other, ctx.diags).as_bytes()),
    }
}

/// array_key_exists($key, $array): true even when the value is null (unlike
/// `isset`).
pub fn array_key_exists(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let key = args.first().ok_or_else(|| {
        PhpError::Error("array_key_exists() expects exactly 2 arguments, 0 given".to_string())
    })?;
    let arr = match args.get(1) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "array_key_exists(): Argument #2 ($array) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "array_key_exists() expects exactly 2 arguments, 1 given".to_string(),
            ))
        }
    };
    let k = zval_to_key(key, ctx);
    Ok(Zval::Bool(arr.contains_key(&k)))
}

/// array_search($needle, $haystack, $strict = false): the key of the first
/// match, or false. Loose comparison by default.
pub fn array_search(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let needle = args.first().ok_or_else(|| {
        PhpError::Error("array_search() expects at least 2 arguments, 0 given".to_string())
    })?;
    let arr = match args.get(1) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "array_search(): Argument #2 ($haystack) must be of type array, {} given",
                other.type_name_for_error()
            )))
        }
        None => {
            return Err(PhpError::Error(
                "array_search() expects at least 2 arguments, 1 given".to_string(),
            ))
        }
    };
    let strict = matches!(args.get(2), Some(v) if convert::is_true_silent(v));
    for (k, v) in arr.iter() {
        let hit = if strict {
            ops::identical(v, needle)
        } else {
            ops::loose_eq(v, needle)
        };
        if hit {
            return Ok(key_to_zval(k));
        }
    }
    Ok(Zval::Bool(false))
}

/// array_fill($start, $count, $value): `$count` copies keyed `$start`,
/// `$start+1`, ... (consecutive, PHP 8). A negative count is a `ValueError`.
pub fn array_fill(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let start = convert::to_long_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("array_fill() expects exactly 3 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let count = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("array_fill() expects exactly 3 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let value = args.get(2).ok_or_else(|| {
        PhpError::Error("array_fill() expects exactly 3 arguments, 2 given".to_string())
    })?;
    if count < 0 {
        return Err(PhpError::ValueError(
            "array_fill(): Argument #2 ($count) must be greater than or equal to 0".to_string(),
        ));
    }
    let mut out = PhpArray::new();
    for i in 0..count {
        out.insert(Key::Int(start + i), value.clone());
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_fill_keys($keys, $value): one entry per element of `$keys`, all set to
/// `$value`. Mirrors PHP's C loop exactly: an integer key passes through; every
/// other key is converted *as a string* (zval_get_string — so `5.7` keys as
/// "5.7", `false`/`null` as "", an array warns "Array to string conversion" and
/// keys as "Array", a non-stringable object is the conversion Error), then
/// canonicalized through the symtable rule ("5" → 5). Duplicate keys collapse,
/// last position wins nothing — first insertion order is kept, value identical.
pub fn array_fill_keys(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let keys = match args.first().map(|a| a.deref_clone()) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "array_fill_keys(): Argument #1 ($keys) must be of type array, {} given",
                other.type_name_for_error()
            )));
        }
        None => {
            return Err(PhpError::ArgumentCountError(
                "array_fill_keys() expects exactly 2 arguments, 0 given".to_string(),
            ));
        }
    };
    let Some(value) = args.get(1) else {
        return Err(PhpError::ArgumentCountError(
            "array_fill_keys() expects exactly 2 arguments, 1 given".to_string(),
        ));
    };
    let mut out = PhpArray::new();
    for (_, k) in keys.iter() {
        let key = match k.deref_clone() {
            Zval::Long(i) => Key::Int(i),
            other => Key::from_bytes(convert::to_zstr_cast(&other, ctx.diags).as_bytes()),
        };
        out.insert(key, value.clone());
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_chunk($array, $length, $preserve_keys = false): split into chunks of
/// `$length` (the last one possibly shorter). Without `$preserve_keys` every
/// chunk reindexes from 0; with it the original keys carry over. `$length < 1`
/// is the PHP 8 ValueError.
pub fn array_chunk(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = match args.first().map(|a| a.deref_clone()) {
        Some(Zval::Array(a)) => a,
        Some(other) => {
            return Err(PhpError::TypeError(format!(
                "array_chunk(): Argument #1 ($array) must be of type array, {} given",
                other.type_name_for_error()
            )));
        }
        None => {
            return Err(PhpError::ArgumentCountError(
                "array_chunk() expects at least 2 arguments, 0 given".to_string(),
            ));
        }
    };
    let length = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::ArgumentCountError(
                "array_chunk() expects at least 2 arguments, 1 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    if length < 1 {
        return Err(PhpError::ValueError(
            "array_chunk(): Argument #2 ($length) must be greater than 0".to_string(),
        ));
    }
    let preserve = args.get(2).map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    let mut out = PhpArray::new();
    let mut chunk = PhpArray::new();
    for (k, v) in arr.iter() {
        if preserve {
            chunk.insert(k.clone(), v.clone());
        } else {
            let _ = chunk.append(v.clone());
        }
        if chunk.len() as i64 == length {
            let _ = out.append(Zval::Array(Rc::new(std::mem::replace(&mut chunk, PhpArray::new()))));
        }
    }
    if chunk.len() > 0 {
        let _ = out.append(Zval::Array(Rc::new(chunk)));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_flip($array): swap keys and values. Only int|string values become
/// keys; others are skipped.
pub fn array_flip(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_flip")?;
    let mut out = PhpArray::new();
    for (k, v) in arr.iter() {
        let new_key = match v {
            Zval::Long(i) => Key::Int(*i),
            Zval::Str(s) => Key::from_zstr(s),
            _ => continue,
        };
        out.insert(new_key, key_to_zval(k));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_combine($keys, $values): zip into an array. The two arrays must have
/// the same length, else a `ValueError`.
pub fn array_combine(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let keys = arr_nth(args, 0, "array_combine", "keys")?;
    let values = arr_nth(args, 1, "array_combine", "values")?;
    if keys.len() != values.len() {
        return Err(PhpError::ValueError(
            "array_combine(): Argument #1 ($keys) and argument #2 ($values) must have the same number of elements".to_string(),
        ));
    }
    let mut out = PhpArray::new();
    for ((_, kv), (_, vv)) in keys.iter().zip(values.iter()) {
        let key = zval_to_key(kv, ctx);
        out.insert(key, vv.clone());
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_pad($array, $size, $value): pad to abs($size) elements with $value, on
/// the right for positive size, on the left for negative. Integer keys are
/// renumbered; string keys are preserved.
pub fn array_pad(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_pad")?;
    let size = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("array_pad() expects exactly 3 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let value = args.get(2).ok_or_else(|| {
        PhpError::Error("array_pad() expects exactly 3 arguments, 2 given".to_string())
    })?;
    let len = arr.len() as i64;
    let target = size.unsigned_abs() as usize;
    let pad_count = target.saturating_sub(arr.len());

    let mut out = PhpArray::new();
    let push_orig = |out: &mut PhpArray, arr: &PhpArray| {
        for (k, v) in arr.iter() {
            match k {
                Key::Str(_) => out.insert(k.clone(), v.clone()),
                Key::Int(_) => {
                    let _ = out.append(v.clone());
                }
            }
        }
    };
    if pad_count == 0 || target <= len as usize {
        push_orig(&mut out, arr);
    } else if size < 0 {
        for _ in 0..pad_count {
            let _ = out.append(value.clone());
        }
        push_orig(&mut out, arr);
    } else {
        push_orig(&mut out, arr);
        for _ in 0..pad_count {
            let _ = out.append(value.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_product($array): product of the elements (numeric-coerced). The empty
/// array yields 1.
pub fn array_product(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_product")?;
    let mut acc = Zval::Long(1);
    for (_, v) in arr.iter() {
        acc = ops::mul(&acc, v, ctx.diags)?;
    }
    Ok(acc)
}

/// array_key_first($array) / array_key_last($array): the first/last key, or
/// null for an empty array.
pub fn array_key_first(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_key_first")?;
    Ok(arr.iter().next().map(|(k, _)| key_to_zval(k)).unwrap_or(Zval::Null))
}

pub fn array_key_last(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let arr = arr_arg(args, "array_key_last")?;
    Ok(arr.iter().last().map(|(k, _)| key_to_zval(k)).unwrap_or(Zval::Null))
}

/// array_diff($array, ...$excludes): elements of $array (keys preserved) whose
/// string form is absent from every other array.
pub fn array_diff(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_diff")?;
    let others = string_sets(&args[1..], ctx, "array_diff")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        let s = convert::to_zstr(v, ctx.diags).as_bytes().to_vec();
        if !others.iter().any(|set| set.contains(&s)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_intersect($array, ...$others): elements of $array (keys preserved)
/// whose string form is present in every other array.
pub fn array_intersect(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_intersect")?;
    let others = string_sets(&args[1..], ctx, "array_intersect")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        let s = convert::to_zstr(v, ctx.diags).as_bytes().to_vec();
        if others.iter().all(|set| set.contains(&s)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Collect the trailing array arguments as sets of string-coerced values.
fn string_sets(
    args: &[Zval],
    ctx: &mut Ctx,
    fname: &str,
) -> Result<Vec<std::collections::HashSet<Vec<u8>>>, PhpError> {
    let mut sets = Vec::with_capacity(args.len());
    for (i, a) in args.iter().enumerate() {
        match a {
            Zval::Array(arr) => {
                let set = arr
                    .iter()
                    .map(|(_, v)| convert::to_zstr(v, ctx.diags).as_bytes().to_vec())
                    .collect();
                sets.push(set);
            }
            other => {
                return Err(PhpError::TypeError(format!(
                    "{fname}(): Argument #{} must be of type array, {} given",
                    i + 2,
                    other.type_name_for_error()
                )))
            }
        }
    }
    Ok(sets)
}

/// Positional array argument `idx` (named `pname`), else a `TypeError`.
fn arr_nth<'a>(
    args: &'a [Zval],
    idx: usize,
    fname: &str,
    pname: &str,
) -> Result<&'a PhpArray, PhpError> {
    match args.get(idx) {
        Some(Zval::Array(a)) => Ok(a),
        Some(other) => Err(PhpError::TypeError(format!(
            "{fname}(): Argument #{} (${pname}) must be of type array, {} given",
            idx + 1,
            other.type_name_for_error()
        ))),
        None => Err(PhpError::Error(format!(
            "{fname}() expects at least {} arguments, {} given",
            idx + 1,
            args.len()
        ))),
    }
}

// --- Step 33: key/assoc set-ops + array_column -----------------------------

/// Validate and collect the trailing array arguments as references (arrays are
/// small, so the `*_key`/`*_assoc` variants query them directly).
fn other_arrays<'a>(
    args: &'a [Zval],
    fname: &str,
) -> Result<Vec<&'a PhpArray>, PhpError> {
    let mut out = Vec::with_capacity(args.len());
    for (i, a) in args.iter().enumerate() {
        match a {
            Zval::Array(arr) => out.push(&**arr),
            other => {
                return Err(PhpError::TypeError(format!(
                    "{fname}(): Argument #{} must be of type array, {} given",
                    i + 2,
                    other.type_name_for_error()
                )))
            }
        }
    }
    Ok(out)
}

/// array_diff_key($array, ...$others): entries of $array whose key is absent
/// from every other array.
pub fn array_diff_key(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_diff_key")?;
    let others = other_arrays(&args[1..], "array_diff_key")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        if !others.iter().any(|o| o.contains_key(k)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_intersect_key($array, ...$others): entries whose key is present in
/// every other array.
pub fn array_intersect_key(args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_intersect_key")?;
    let others = other_arrays(&args[1..], "array_intersect_key")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        if others.iter().all(|o| o.contains_key(k)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Whether some/every other array maps `k` to the string `vs`.
fn assoc_match(other: &PhpArray, k: &Key, vs: &[u8], ctx: &mut Ctx) -> bool {
    other
        .get(k)
        .is_some_and(|ov| convert::to_zstr(ov, ctx.diags).as_bytes() == vs)
}

/// array_diff_assoc($array, ...$others): entries whose (key, value) pair (value
/// compared as a string) is matched by no other array.
pub fn array_diff_assoc(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_diff_assoc")?;
    let others = other_arrays(&args[1..], "array_diff_assoc")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        let vs = convert::to_zstr(v, ctx.diags).as_bytes().to_vec();
        if !others.iter().any(|o| assoc_match(o, k, &vs, ctx)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_intersect_assoc($array, ...$others): entries whose (key, value) pair
/// (value compared as a string) is present in every other array.
pub fn array_intersect_assoc(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let base = arr_arg(args, "array_intersect_assoc")?;
    let others = other_arrays(&args[1..], "array_intersect_assoc")?;
    let mut out = PhpArray::new();
    for (k, v) in base.iter() {
        let vs = convert::to_zstr(v, ctx.diags).as_bytes().to_vec();
        if others.iter().all(|o| assoc_match(o, k, &vs, ctx)) {
            out.insert(k.clone(), v.clone());
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// array_column($array, $column_key, $index_key = null): pluck `$column_key`
/// from each row (a row missing it is skipped); `null` column keeps the whole
/// row. With `$index_key` the result is keyed by that field, else sequential.
/// Rows may be arrays or objects (public properties).
pub fn array_column(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let rows = arr_arg(args, "array_column")?;
    let column = args.get(1).ok_or_else(|| {
        PhpError::Error("array_column() expects at least 2 arguments, 1 given".to_string())
    })?;
    let column_key = if matches!(column, Zval::Null) {
        None
    } else {
        Some(zval_to_key(column, ctx))
    };
    let index_key = args.get(2).filter(|v| !matches!(v, Zval::Null)).map(|v| zval_to_key(v, ctx));

    let mut out = PhpArray::new();
    for (_, row) in rows.iter() {
        let value = match &column_key {
            Some(ck) => match row_get(row, ck) {
                Some(v) => v,
                None => continue, // row lacks the column
            },
            None => row.clone(),
        };
        match index_key.as_ref().and_then(|ik| row_get(row, ik)) {
            Some(idx) => out.insert(zval_to_key(&idx, ctx), value),
            None => {
                let _ = out.append(value);
            }
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Look up a field of a row that is either an array (by key) or an object (by
/// public property name).
fn row_get(row: &Zval, key: &Key) -> Option<Zval> {
    match row {
        Zval::Array(a) => a.get(key).cloned(),
        Zval::Object(o) => {
            let name = match key {
                Key::Str(s) => s.as_bytes().to_vec(),
                Key::Int(i) => i.to_string().into_bytes(),
            };
            o.borrow().props.get(&name).cloned()
        }
        _ => None,
    }
}
