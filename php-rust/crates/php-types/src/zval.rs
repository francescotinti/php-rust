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
    /// A stream resource (`Zval::Resource`, step 51, D-51.1): the value `fopen`
    /// and friends return. Handle semantics like `Object` — `$g = $f` aliases
    /// the same stream via the shared `Rc<RefCell<Resource>>`, so `fclose($g)`
    /// also closes `$f`. `gettype` reports "resource"/"resource (closed)". The
    /// byte stream itself lives in [`crate::Resource`]; only `fopen` (which owns
    /// the id counter) mints these, in the evaluator (D-51.3).
    Resource(Rc<RefCell<crate::Resource>>),
    /// A weak reference to an object (the backing of `WeakReference`/`WeakMap`).
    /// Holds a `Weak` so it does *not* keep the object alive; `upgrade()` yields
    /// the object while at least one strong handle remains, else `None` (true
    /// weakness). Never surfaces to user code — `WeakReference::get()` /
    /// `WeakMap` offsets upgrade it to an object or `null` via the internal
    /// `__weak_get` builtin, and `var_dump` special-cases the two classes.
    WeakHandle(std::rc::Weak<RefCell<Object>>),
    /// A deferred function-argument *place* (Zend's FETCH_DIM_FUNC_ARG feeding
    /// SEND_VAR_EX): an all-`Index` path rooted at a variable, passed as an
    /// argument to a call whose callee — hence whether the parameter is
    /// by-reference — is only known at bind time. The dispatch funnels
    /// materialize it there: a by-reference parameter W-fetches (aliases the
    /// element, creating a missing key silently), a by-value one R-fetches
    /// (warns on a missing key, creates nothing). Never escapes the call
    /// window — no other consumer sees this variant.
    ArgPlace(Rc<ArgPlace>),
}

/// Payload of [`Zval::ArgPlace`]: which store roots the path, the steps to
/// walk, the `Index` keys (already evaluated, source order), and the root
/// variable's bare name for the R-branch's "Undefined variable" warning
/// (empty when the base cannot be an undefined variable).
#[derive(Debug)]
pub struct ArgPlace {
    pub base: ArgPlaceBase,
    pub steps: Box<[ArgPlaceStep]>,
    pub keys: Vec<Zval>,
    pub name: Box<[u8]>,
}

/// The store rooting an [`ArgPlace`] path.
#[derive(Clone, Copy, Debug)]
pub enum ArgPlaceBase {
    /// Caller-frame local slot.
    Local(u32),
    /// Slot in the global (bottom) frame.
    Global(u32),
    /// VM-level superglobal store index (`$_SESSION` &c.).
    Superglobal(u8),
    /// The caller frame's `$this` (a property path like `$this->data`).
    This,
}

/// One step of an [`ArgPlace`] path: an array/ArrayAccess dimension (its key
/// rides in `ArgPlace::keys`, one per `Index`, source order) or a literal
/// property name. Dynamic property names (`->$n`) are never deferred.
#[derive(Clone, Debug)]
pub enum ArgPlaceStep {
    Index,
    Prop(Box<[u8]>),
}

/// A lowered-and-captured closure value (step 18). `fn_idx` selects the body
/// (`FnDecl`) from the evaluator's flat closure table (D-18.2); `captures` are
/// the `(dst-slot, value)` bindings snapshotted at *creation* time — a `use($x)`
/// by-value capture holds a plain clone, a `use(&$x)` by-reference capture holds
/// a `Zval::Ref` to the shared cell (D-18.3).
impl Drop for Closure {
    fn drop(&mut self) {
        crate::object::free_object_id(self.id);
    }
}

#[derive(Debug)]
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
    /// Stable id of the module that defined this closure, into the VM's module
    /// registry. Keeps the closure callable after control leaves its defining
    /// module — e.g. a closure created in an `include`/`eval` unit and invoked
    /// later (Composer's `autoload_static` initializer). `0` is `main`.
    pub module_id: usize,
    /// The class *scope* the closure body runs in (a VM `ClassId`), governing
    /// `private`/`protected` member access and `self::`/`static::`. Set at
    /// creation to the defining class, or overridden by `Closure::bind`/`bindTo`'s
    /// `$newScope` argument. `None` for an unscoped (top-level) closure.
    pub scope: Option<usize>,
    /// `static function () {}` — a static closure never binds `$this`, so
    /// `bindTo`/`Closure::bind`/`call` with a non-null instance warns and yields
    /// `null` (step 19-6). `false` for an ordinary closure or first-class callable.
    pub is_static: bool,
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
            Zval::Undef | Zval::Null | Zval::ArgPlace(_) => "NULL",
            Zval::Bool(_) => "boolean",
            Zval::Long(_) => "integer",
            Zval::Double(_) => "double",
            Zval::Str(_) => "string",
            Zval::Array(_) => "array",
            Zval::Ref(cell) => cell.borrow().gettype(),
            Zval::Closure(_) | Zval::Object(_) | Zval::Generator(_) | Zval::WeakHandle(_) => "object",
            // "resource" while open, "resource (closed)" after fclose (D-51.1).
            Zval::Resource(r) => r.borrow().type_name(),
        }
    }

    /// Type name as used in error messages (TypeError etc.).
    pub fn error_type_name(&self) -> &'static str {
        match self {
            Zval::Undef | Zval::Null | Zval::ArgPlace(_) => "null",
            Zval::Bool(_) => "bool",
            Zval::Long(_) => "int",
            Zval::Double(_) => "float",
            Zval::Str(_) => "string",
            Zval::Array(_) => "array",
            Zval::Ref(cell) => cell.borrow().error_type_name(),
            Zval::Closure(_) => "Closure",
            Zval::Generator(_) => "Generator",
            // A user object: the generic name. Use [`Self::type_name_for_error`]
            // where PHP names it by class (operand / type errors).
            Zval::Object(_) | Zval::WeakHandle(_) => "object",
            Zval::Resource(r) => r.borrow().type_name(),
        }
    }

    /// Like [`Self::error_type_name`] but names an object by its **class** — as
    /// PHP's `zend_zval_type_name` does in operand and type errors ("stdClass",
    /// not "object"). Returns an owned `String` since a class name is dynamic.
    pub fn type_name_for_error(&self) -> String {
        match self {
            Zval::Object(o) => {
                String::from_utf8_lossy(o.borrow().class_name.as_bytes()).into_owned()
            }
            Zval::Ref(cell) => cell.borrow().type_name_for_error(),
            other => other.error_type_name().to_string(),
        }
    }

    /// Names a value like PHP's `zend_zval_value_name`: bools read as their
    /// *value* ("true"/"false"), everything else as [`Self::type_name_for_error`].
    /// PHP 8.3+ uses this flavour in the "Call to a member function … on …"
    /// fatal (while property-access warnings keep the plain type name).
    pub fn value_name_for_error(&self) -> String {
        match self {
            Zval::Bool(b) => if *b { "true" } else { "false" }.to_string(),
            Zval::Ref(cell) => cell.borrow().value_name_for_error(),
            other => other.type_name_for_error(),
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
