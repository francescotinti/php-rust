//! Builtin function ABI (D-G16).
//!
//! The evaluator never references concrete builtins: it dispatches through a
//! [`Registry`] of function pointers injected at run time. The implementations
//! live in the `php-builtins` crate (which depends on this one) and expose a
//! `registry()` the caller passes to [`crate::run_with`] /
//! [`crate::run_source_with`]. This keeps the dependency edge one-way
//! (php-builtins → php-runtime) while still letting builtins write to stdout and
//! raise diagnostics.

use std::collections::HashMap;

use php_types::{Diags, PhpError, Zval};

/// The execution context handed to a builtin: the shared output sink and the
/// diagnostic accumulator owned by the evaluator.
pub struct Ctx<'a> {
    pub out: &'a mut Vec<u8>,
    pub diags: &'a mut Diags,
}

/// A builtin function: positional arguments in, a value (or fatal error) out.
pub type BuiltinFn = fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>;

/// Maps a function name (bytes, as written in source) to its implementation.
pub type Registry = HashMap<Vec<u8>, BuiltinFn>;
