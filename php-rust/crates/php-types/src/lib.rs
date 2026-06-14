//! Core PHP value types and operator semantics.
//!
//! Semantics reference: PHP 8.5.7 C source (see diary/01-semantic-model.md).

mod array;
pub mod convert;
mod diag;
pub mod dtoa;
pub mod numstr;
mod object;
pub mod ops;
mod zstr;
mod zval;

pub use array::{ArrayAppendError, Key, PhpArray};
pub use diag::{Diag, Diags, PhpError};
pub use object::{Object, ObjectInfo, PropVis, Props};
pub use zstr::{PhpStr, ZStr};
pub use zval::{Closure, ClosureInfo, ClosureParam, ClosureRender, Zval};
