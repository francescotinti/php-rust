//! S-194 B2: in-line copies and ends of life for the dispatcher's most executed arms.
//! Criterion: wp194-harness/s194-criterio-B.md.
use std::rc::Rc;

use super::Zval;

/// S-194 B2: `Zval::clone` for the dispatcher's pushes (`LoadVar`, `This`, `Dup`): the derived
/// clone is an out-of-line call with a jump table; the four species that make almost every
/// operand are copied (or reference-counted) in line. The census builds keep the counted paths.
#[inline(always)]
pub(super) fn clone_hot(v: &Zval) -> Zval {
    #[cfg(not(any(feature = "zval-census", feature = "mem-census")))]
    match v {
        Zval::Long(n) => return Zval::Long(*n),
        Zval::Object(o) => return Zval::Object(Rc::clone(o)),
        Zval::Str(s) => return Zval::Str(s.clone()),
        Zval::Array(a) => return Zval::Array(Rc::clone(a)),
        _ => {}
    }
    v.clone()
}

/// S-194 B2: a scalar owns nothing — its end of life needs neither the GC note nor the
/// out-of-line drop glue. True = `v` was such a value and is gone.
#[inline(always)]
pub(super) fn drop_scalar(v: Zval) -> Result<(), Zval> {
    match v {
        Zval::Long(_) | Zval::Bool(_) | Zval::Null | Zval::Undef | Zval::Double(_) => { std::mem::forget(v); Ok(()) }
        v => Err(v),
    }
}
