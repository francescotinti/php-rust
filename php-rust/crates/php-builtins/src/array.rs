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
