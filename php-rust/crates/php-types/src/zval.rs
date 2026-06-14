use std::cell::RefCell;
use std::rc::Rc;

use crate::{PhpArray, PhpStr};

/// A PHP value. Mirrors the observable semantics of `zval`
/// (Zend/zend_types.h:355-380, type tags :620-631).
///
/// `Undef` is distinct from `Null`: reading an unset variable raises
/// "Warning: Undefined variable" and yields NULL; the engine needs the
/// distinction, programs never see it directly.
///
/// Heap types use `Rc` with copy-on-write via `Rc::make_mut`, which matches
/// Zend's refcount + SEPARATE_* separation exactly.
#[derive(Clone, Debug)]
pub enum Zval {
    Undef,
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    Str(Rc<PhpStr>),
    Array(Rc<PhpArray>),
    /// A PHP reference (`IS_REFERENCE`, step 11d): a shared, mutable cell that
    /// any number of variables / array elements can alias. Writing through any
    /// alias is visible to all. **Invariant:** the inner value is never itself a
    /// `Ref` (PHP collapses reference-to-reference). Consumers materialise the
    /// underlying value with [`Zval::deref_clone`]; only `var_dump` inspects the
    /// `Ref` wrapper directly (to print the `&` marker for container elements).
    Ref(Rc<RefCell<Zval>>),
    /// A closure / callable value (`Zval::Closure`, step 18, D-18.1). PHP
    /// closures are objects (`instanceof Closure`); with no OOP yet we model them
    /// as a dedicated variant. The body lives in the evaluator's closure table
    /// (selected by [`Closure::fn_idx`]); `gettype` reports `"object"`.
    Closure(Rc<Closure>),
}

/// A lowered-and-captured closure value (step 18). `fn_idx` selects the body
/// (`FnDecl`) from the evaluator's flat closure table (D-18.2); `captures` are
/// the `(dst-slot, value)` bindings snapshotted at *creation* time — a `use($x)`
/// by-value capture holds a plain clone, a `use(&$x)` by-reference capture holds
/// a `Zval::Ref` to the shared cell (D-18.3).
#[derive(Clone, Debug)]
pub struct Closure {
    pub fn_idx: usize,
    pub captures: Vec<(u32, Zval)>,
}

impl Zval {
    pub fn str_from(s: &str) -> Zval {
        Zval::Str(PhpStr::from_str(s))
    }

    /// The underlying value, following a reference (D-R11). A non-reference is
    /// cloned as-is; a `Ref` yields a clone of its current cell contents. By the
    /// no-ref-to-ref invariant this never returns a `Ref`.
    pub fn deref_clone(&self) -> Zval {
        match self {
            Zval::Ref(cell) => cell.borrow().clone(),
            v => v.clone(),
        }
    }

    /// Whether this value is a reference wrapper (used by `var_dump` to emit the
    /// `&` marker on container elements).
    pub fn is_ref(&self) -> bool {
        matches!(self, Zval::Ref(_))
    }

    /// Type name as reported by gettype().
    pub fn gettype(&self) -> &'static str {
        match self {
            Zval::Undef | Zval::Null => "NULL",
            Zval::Bool(_) => "boolean",
            Zval::Long(_) => "integer",
            Zval::Double(_) => "double",
            Zval::Str(_) => "string",
            Zval::Array(_) => "array",
            Zval::Ref(cell) => cell.borrow().gettype(),
            Zval::Closure(_) => "object",
        }
    }

    /// Type name as used in error messages (TypeError etc.).
    pub fn error_type_name(&self) -> &'static str {
        match self {
            Zval::Undef | Zval::Null => "null",
            Zval::Bool(_) => "bool",
            Zval::Long(_) => "int",
            Zval::Double(_) => "float",
            Zval::Str(_) => "string",
            Zval::Array(_) => "array",
            Zval::Ref(cell) => cell.borrow().error_type_name(),
            Zval::Closure(_) => "Closure",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clone_shares_heap_payload() {
        let a = Zval::str_from("hello");
        let b = a.clone();
        match (&a, &b) {
            (Zval::Str(x), Zval::Str(y)) => assert!(Rc::ptr_eq(x, y)),
            _ => unreachable!(),
        }
    }

    #[test]
    fn gettype_names() {
        assert_eq!(Zval::Null.gettype(), "NULL");
        assert_eq!(Zval::Bool(true).gettype(), "boolean");
        assert_eq!(Zval::Long(1).gettype(), "integer");
        assert_eq!(Zval::Double(1.0).gettype(), "double");
    }
}
