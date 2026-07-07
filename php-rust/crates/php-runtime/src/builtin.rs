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
    /// Output that must reach the real stdout *bypassing output buffering*:
    /// PHP routes stream writes (`fwrite(STDOUT)`, `php://stdout`) past the
    /// `ob_*` stack, unlike `echo`/`print`. The VM appends this straight to
    /// its stdout/rendered sinks after the builtin returns.
    pub direct_out: &'a mut Vec<u8>,
    /// Precomputed `__debugInfo()` results for `var_dump`, keyed by object id.
    /// The VM invokes each debuggable object's `__debugInfo` method *before* the
    /// dump (so a lazy object initializes only if that method touches its state,
    /// PHP 8.4) and `var_dump` renders the returned array under the object header
    /// instead of the raw property slots. Empty for every other builtin.
    pub debug_info: &'a std::collections::HashMap<u32, Zval>,
}

/// A by-value builtin: positional arguments in, a value (or fatal error) out.
pub type BuiltinFn = fn(&[Zval], &mut Ctx) -> Result<Zval, PhpError>;

/// A builtin whose *first* argument is taken by reference (D-R7, step 11c). The
/// evaluator binds the caller's first-argument variable and hands the builtin
/// `&mut Zval` access to it; the remaining positional arguments arrive by value.
/// Mutations to the first argument are visible to the caller (write-through).
///
/// The family modelled here (`array_push` / `sort` / `array_pop` /
/// `array_shift`) shares a single required first parameter named `$array`, which
/// is why the evaluator can raise the generic "Argument #1 ($array) could not be
/// passed by reference" / "expects at least 1 argument" errors on its behalf.
pub type BuiltinRefFn = fn(&mut Zval, &[Zval], &mut Ctx) -> Result<Zval, PhpError>;

/// A registered builtin: either fully by-value, or by-reference in its first
/// argument.
#[derive(Clone, Copy)]
pub enum Builtin {
    Value(BuiltinFn),
    RefFirst(BuiltinRefFn),
}

/// Maps a function name (bytes, as written in source) to its implementation.
pub type Registry = HashMap<Vec<u8>, Builtin>;
