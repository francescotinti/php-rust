//! String builtins (plan step 10): implode, explode, substr, ...

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

/// implode($separator, $array) or implode($array).
///
/// PHP 8 removed the legacy reversed `implode($array, $glue)` order: passing an
/// array as the separator is now a `TypeError`. Each element is string-coerced.
pub fn implode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let (glue, arr) = match args {
        // Single argument: it must be the array; glue is empty.
        [Zval::Array(a)] => (Vec::new(), a),
        [only] => {
            return Err(PhpError::TypeError(format!(
                "implode(): Argument #1 ($array) must be of type array, {} given",
                only.error_type_name()
            )))
        }
        [sep, rest @ ..] => {
            if let Zval::Array(_) = sep {
                return Err(PhpError::TypeError(
                    "implode(): Argument #1 ($separator) must be of type string, array given"
                        .to_string(),
                ));
            }
            match rest.first() {
                Some(Zval::Array(a)) => (convert::to_zstr(sep, ctx.diags).as_bytes().to_vec(), a),
                Some(other) => {
                    return Err(PhpError::TypeError(format!(
                        "implode(): Argument #2 ($array) must be of type array, {} given",
                        other.error_type_name()
                    )))
                }
                None => unreachable!("rest has at least one element"),
            }
        }
        [] => {
            return Err(PhpError::Error(
                "implode() expects at least 1 argument, 0 given".to_string(),
            ))
        }
    };

    let mut out = Vec::new();
    for (i, (_, v)) in arr.iter().enumerate() {
        if i > 0 {
            out.extend_from_slice(&glue);
        }
        out.extend_from_slice(convert::to_zstr(v, ctx.diags).as_bytes());
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// explode($separator, $string, $limit = PHP_INT_MAX).
///
/// Empty separator is a `ValueError`. Positive limit caps the element count
/// (last element keeps the unsplit remainder); 0 behaves like 1; negative
/// drops that many trailing pieces.
pub fn explode(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let sep = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("explode() expects at least 2 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let sep = sep.as_bytes();
    if sep.is_empty() {
        return Err(PhpError::ValueError(
            "explode(): Argument #1 ($separator) must not be empty".to_string(),
        ));
    }
    let string = convert::to_zstr(
        args.get(1).ok_or_else(|| {
            PhpError::Error("explode() expects at least 2 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let limit = match args.get(2) {
        Some(v) => convert::to_long_cast(v, ctx.diags),
        None => i64::MAX,
    };

    let parts = split_all(string.as_bytes(), sep);
    let mut out = PhpArray::new();

    if limit > 0 && (limit as usize) < parts.len() {
        // Keep the first limit-1 pieces verbatim, then the rest as one element
        // reconstructed by re-joining with the separator.
        let keep = limit as usize - 1;
        for p in &parts[..keep] {
            let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
        }
        let remainder = parts[keep..].join(sep);
        let _ = out.append(Zval::Str(PhpStr::new(remainder)));
    } else if limit < 0 {
        let drop = (-limit) as usize;
        let keep = parts.len().saturating_sub(drop);
        for p in &parts[..keep] {
            let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
        }
    } else {
        // limit == 0 behaves like 1; limit >= parts.len() keeps everything.
        if limit == 0 {
            let _ = out.append(Zval::Str(Rc::clone(&string)));
        } else {
            for p in &parts {
                let _ = out.append(Zval::Str(PhpStr::new(p.to_vec())));
            }
        }
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// Split `s` on every (non-overlapping) occurrence of `sep` (sep non-empty).
fn split_all<'a>(s: &'a [u8], sep: &[u8]) -> Vec<&'a [u8]> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i + sep.len() <= s.len() {
        if &s[i..i + sep.len()] == sep {
            parts.push(&s[start..i]);
            i += sep.len();
            start = i;
        } else {
            i += 1;
        }
    }
    parts.push(&s[start..]);
    parts
}
