//! Core PHP value types and operator semantics.
//!
//! Semantics reference: PHP 8.5.7 C source (see diary/01-semantic-model.md).

mod array;
pub mod convert;
mod diag;
pub mod dtoa;
mod generator;
pub mod numstr;
mod object;
pub mod ops;
pub mod stream;
mod zstr;
mod zval;

pub use array::{ArrayAppendError, Key, PhpArray};
pub use diag::{Diag, Diags, PhpError};
pub use generator::{GenKey, GenState, GenStatus};
pub use object::{unmangle_prop_key, LazyKind, Object, ObjectInfo, PropVis, Props};
pub use stream::{
    open_file_stream, open_php_stream, DirHandle, ResKind, Resource, Stream, StreamBackend,
};
pub use zstr::{PhpStr, ZStr};
pub use zval::{Closure, ClosureInfo, ClosureParam, ClosureRender, Zval};
