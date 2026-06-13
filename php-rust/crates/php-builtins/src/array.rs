//! Array builtins (plan step 10): count, array_keys, array_values, ...

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, Zval};

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
