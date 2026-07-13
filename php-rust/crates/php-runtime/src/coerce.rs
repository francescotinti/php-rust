//! Scalar type-hint coercion, shared by the tree-walker and the bytecode VM
//! (step 14 / 16). Relocated here from `eval/mod.rs` so the VM can enforce
//! parameter and return type hints without depending on the evaluator (the F1
//! engine-switch direction). The weak-mode rules coerce numeric strings, ints,
//! floats and bools across the four scalar types (emitting the lossy-float
//! deprecations); strict mode (`declare(strict_types=1)`) only accepts an exact
//! type, with `int` → `float` widening the single exception.

use php_types::{convert, numstr, Diag, Diags, Zval};

use crate::hir::{HintKind, ScalarType, TypeHint};

/// Coerce `value` to scalar type `hint` under PHP's *weak* typing rules
/// (step 14). On success returns the coerced value (emitting the lossy-float
/// deprecations along the way); on failure returns the PHP type name of `value`
/// for the TypeError message. `null` satisfies a nullable hint verbatim.
pub(crate) fn coerce_to_hint(
    value: Zval,
    hint: &TypeHint,
    diags: &mut Diags,
    strict: bool,
) -> Result<Zval, &'static str> {
    // Follow a reference to its value first (defensive; bound args are plain).
    if let Zval::Ref(c) = &value {
        let inner = c.borrow().clone();
        return coerce_to_hint(inner, hint, diags, strict);
    }
    // Only scalar hints coerce here; non-scalar hints (array/callable/object/class)
    // are *checked* by the VM binder (which has the class table), so leave the
    // value untouched if one reaches this funnel.
    let HintKind::Scalar(scalar) = &hint.kind else {
        return Ok(value);
    };
    if matches!(value, Zval::Null | Zval::Undef) {
        return if hint.nullable {
            Ok(Zval::Null)
        } else {
            Err("null")
        };
    }
    let given = php_type_name(&value);
    if strict {
        return coerce_strict(value, *scalar).ok_or(given);
    }
    match scalar {
        ScalarType::Int => coerce_to_int(value, diags),
        ScalarType::Float => coerce_to_float(value),
        ScalarType::String => coerce_to_string(value, diags),
        ScalarType::Bool => coerce_to_bool(value, diags),
    }
    .ok_or(given)
}

/// Strict-mode (`declare(strict_types=1)`) scalar check: the value's type must
/// match the hint exactly, with the single exception of `int` → `float`
/// widening. No coercion, no deprecations (step 16, D-16.3).
fn coerce_strict(value: Zval, scalar: ScalarType) -> Option<Zval> {
    match (scalar, &value) {
        (ScalarType::Int, Zval::Long(_))
        | (ScalarType::Float, Zval::Double(_))
        | (ScalarType::String, Zval::Str(_))
        | (ScalarType::Bool, Zval::Bool(_)) => Some(value),
        // The one widening allowed in strict mode.
        (ScalarType::Float, Zval::Long(l)) => Some(Zval::Double(*l as f64)),
        _ => None,
    }
}

/// Weak coercion to `int`: numeric strings must be *well formed* (stricter than
/// the `(int)` cast — `"12abc"` fails). A float / float-string that loses
/// precision emits a deprecation (D-14.6). `None` means a type error.
fn coerce_to_int(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Long(_) => Some(value),
        Zval::Bool(b) => Some(Zval::Long(b as i64)),
        Zval::Double(d) => Some(Zval::Long(convert::dval_to_lval_safe(d, diags))),
        Zval::Str(ref s) => {
            let info = numstr::parse_numeric_ex(s.as_bytes(), false)?;
            if info.trailing {
                return None;
            }
            match info.num {
                numstr::Num::Long(l) => Some(Zval::Long(l)),
                numstr::Num::Double(d) => {
                    let l = convert::dval_to_lval(d);
                    if !convert::is_long_compatible(d, l) {
                        diags.push(Diag::Deprecated(format!(
                            "Implicit conversion from float-string \"{}\" to int loses precision",
                            String::from_utf8_lossy(s.as_bytes())
                        )));
                    }
                    Some(Zval::Long(l))
                }
            }
        }
        _ => None,
    }
}

/// Weak coercion to `float`: numeric strings (incl. scientific) convert; others
/// are a type error.
fn coerce_to_float(value: Zval) -> Option<Zval> {
    match value {
        Zval::Double(_) => Some(value),
        Zval::Long(l) => Some(Zval::Double(l as f64)),
        Zval::Bool(b) => Some(Zval::Double(b as i64 as f64)),
        Zval::Str(ref s) => {
            let info = numstr::parse_numeric_ex(s.as_bytes(), false)?;
            if info.trailing {
                return None;
            }
            Some(Zval::Double(match info.num {
                numstr::Num::Long(l) => l as f64,
                numstr::Num::Double(d) => d,
            }))
        }
        _ => None,
    }
}

/// Weak coercion to `string`: any scalar stringifies; array / object are a type
/// error.
fn coerce_to_string(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Str(_) => Some(value),
        Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) => {
            Some(Zval::Str(convert::to_zstr(&value, diags)))
        }
        _ => None,
    }
}

/// Weak coercion to `bool`: any scalar converts; array / object are a type
/// error.
fn coerce_to_bool(value: Zval, diags: &mut Diags) -> Option<Zval> {
    match value {
        Zval::Bool(_) => Some(value),
        Zval::Long(_) | Zval::Double(_) | Zval::Str(_) => {
            Some(Zval::Bool(convert::to_bool(&value, diags)))
        }
        _ => None,
    }
}

/// Lowercase PHP type name used in `TypeError` messages (distinct from
/// `gettype`'s capitalised names): `null`/`bool`/`int`/`float`/`string`/`array`/
/// `object`/`resource`.
pub(crate) fn php_type_name(v: &Zval) -> &'static str {
    match v {
        Zval::Undef | Zval::Null | Zval::ArgPlace(_) => "null",
        Zval::Bool(_) => "bool",
        Zval::Long(_) => "int",
        Zval::Double(_) => "float",
        Zval::Str(_) => "string",
        Zval::Array(_) => "array",
        Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) | Zval::WeakHandle(_) => "object",
        Zval::Resource(_) => "resource",
        Zval::Ref(c) => php_type_name(&c.borrow()),
    }
}
