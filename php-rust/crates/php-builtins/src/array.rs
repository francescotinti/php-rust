//! Array builtins (plan step 10): count, array_keys, array_values, ...

use std::rc::Rc;

use php_runtime::Ctx;
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
