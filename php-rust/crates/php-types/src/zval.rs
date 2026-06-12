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
}

impl Zval {
    pub fn str_from(s: &str) -> Zval {
        Zval::Str(PhpStr::from_str(s))
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
