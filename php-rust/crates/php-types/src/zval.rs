use std::cell::RefCell;
use std::rc::Rc;

use crate::{GenState, Object, PhpArray, PhpStr};

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
    /// A user-defined class instance (`Zval::Object`, step 19, D-19.1). Handle
    /// semantics: the shared `Rc<RefCell<Object>>` means `$q = $p` aliases the
    /// same instance and writes through any handle are visible to all (unlike the
    /// copy-on-write `Array`). `gettype` reports `"object"`.
    Object(Rc<RefCell<Object>>),
    /// A `Generator` object (step 39, D-GEN-3): the value a generator function
    /// returns. Like `Closure` it is modelled as a dedicated variant rather than
    /// a user-class instance; `gettype` reports `"object"` and `instanceof`
    /// special-cases `Generator`/`Iterator`/`Traversable`. Handle semantics via
    /// the shared `Rc<RefCell<GenState>>` (assigning the variable aliases the
    /// same running generator). See [`crate::GenState`].
    Generator(Rc<RefCell<GenState>>),
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
    /// `Some(name)` for a first-class callable such as `strlen(...)` (step 18-6,
    /// D-18.10): the value wraps a function *name* and `fn_idx`/`captures` are
    /// unused. `None` for an ordinary anonymous/arrow closure.
    pub named: Option<Rc<PhpStr>>,
    /// The bound `$this` (step 19-6, D-19.19): captured at creation for a
    /// non-static closure defined in a method, or set by `bindTo`/`Closure::bind`.
    /// `None` for a static closure, a top-level closure, or a first-class callable.
    pub bound_this: Option<Zval>,
    /// Per-instance object handle, shown as `#N` by `var_dump` (step 18-7).
    pub id: u32,
    /// Shared render metadata for `var_dump` / `print_r` (step 18-7, D-18.9).
    pub info: Rc<ClosureInfo>,
}

/// What `var_dump` / `print_r` print for a closure value (step 18-7, D-18.9).
#[derive(Clone, Debug)]
pub struct ClosureInfo {
    pub kind: ClosureRender,
    /// Formal parameters, in order, for the trailing `parameter` pseudo-property
    /// (omitted entirely when empty, matching PHP).
    pub params: Vec<ClosureParam>,
}

/// The header form of a dumped closure: an ordinary closure shows
/// `name`/`file`/`line`; a first-class callable shows a single `function`.
#[derive(Clone, Debug)]
pub enum ClosureRender {
    Closure {
        name: Rc<PhpStr>,
        file: Rc<PhpStr>,
        line: u32,
    },
    Function(Rc<PhpStr>),
}

/// One formal parameter as rendered by `var_dump` (`["$x"] => "<required>"`).
#[derive(Clone, Debug)]
pub struct ClosureParam {
    /// Parameter name *without* the leading `$`.
    pub name: Box<[u8]>,
    pub optional: bool,
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
            Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) => "object",
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
            Zval::Generator(_) => "Generator",
            // PHP uses the actual class name here; this funnel returns a
            // `&'static str`, so we render the generic name and let the evaluator
            // (which has the class table) build class-specific messages where it
            // matters (step 19-1 simplification; refine if the corpus needs it).
            Zval::Object(_) => "object",
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
