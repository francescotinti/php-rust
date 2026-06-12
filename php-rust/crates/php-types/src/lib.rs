//! Core PHP value types: byte strings, zval, ordered arrays.
//!
//! Semantics reference: PHP 8.5.7 C source (see diary/01-semantic-model.md).

mod array;
mod zstr;
mod zval;

pub use array::{ArrayAppendError, Key, PhpArray};
pub use zstr::{PhpStr, ZStr};
pub use zval::Zval;
