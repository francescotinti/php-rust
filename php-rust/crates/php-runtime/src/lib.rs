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
pub mod coerce;
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
// Session F switch: the bytecode VM is the production engine. `run_source` /
// `run_source_with` / `Outcome` now resolve to the VM (the tree-walker in
// `eval` is retained only for the corpus `--engine=eval` baseline until F2).
pub use lower::{lower_source, LowerError};
pub use vm::{run_source, run_source_with, VmOutcome as Outcome, VmRunError};
