//! PHP runtime: HIR, the mago→HIR lowering bridge, and (later) the evaluator.
//!
//! Architecture (see plan / diary 02-mapping-table, D-G8/D-G9):
//!
//! ```text
//! PHP source ──mago──► AST ──lower──► HIR ──► evaluator (tree-walk)
//! ```
//!
//! This crate owns the AST→HIR boundary so the rest of the runtime never sees
//! mago's arena-bound types.

pub mod builtin;
pub mod bytecode;
pub mod compile;
pub mod eval;
pub mod hir;
pub mod json;
pub mod lower;
pub mod vm;
pub mod mbregex;
pub mod preg;
pub mod scanf;
pub mod unserialize;

pub use builtin::{Builtin, BuiltinFn, BuiltinRefFn, Ctx, Registry};
pub use eval::{run, run_source, run_source_with, run_with, Outcome};
pub use lower::{lower_source, LowerError};
