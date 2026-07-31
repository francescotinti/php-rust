//! PHP runtime: HIR, the mago→HIR lowering bridge, the bytecode compiler, and the
//! bytecode VM (the production execution engine).
//!
//! Architecture (see plan / diary 02-mapping-table, D-G8/D-G9):
//!
//! ```text
//! PHP source ──mago──► AST ──lower──► HIR ──compile──► bytecode ──► VM
//! ```
//!
//! This crate owns the AST→HIR boundary so the rest of the runtime never sees
//! mago's arena-bound types.

// S-79.0.3 (Council WP-80): compile-phase sub-channel attribution (a1
// prelude / a3 include-compile) for the census build — see the module doc.
#[cfg(feature = "census-instrumentation")]
pub mod alloc_census;
pub mod builtin;
pub mod bytecode;
pub mod coerce;
pub mod compile;
pub mod hir;
pub mod json;
pub mod logging;
pub mod lower;
pub mod vm;

// WP-77.6 Phase B: Expose Vm construction for worker-pool integration.
// run_module_with_hir handles: Vm init → request_start semantics →
// main run → shutdown_fns → ob_flush → destructors → session_shutdown.
pub use vm::run_module_with_hir;
pub mod mbregex;
pub mod preg;
pub mod scanf;
pub mod unserialize;

pub use builtin::{Builtin, BuiltinFn, BuiltinRefFn, Ctx, Registry};
// Session F: the bytecode VM is the sole production engine. `run_source` /
// `run_source_with` / `Outcome` resolve to the VM; the tree-walking `eval` module
// was deleted once every construct it handled became VM-native (F2).
pub use lower::{lower_source, lower_source_seeded, DeferPolicy, LowerError, MissingSym};
pub use vm::{
    run_source, run_source_with, run_source_with_argv, run_source_with_ini,
    VmOutcome as Outcome, VmRunError,
};
// A-BB6 (design79): main-unit cache API — acquire/publish for the worker
// SAPI, run_source_probed for the cli-server (probe = ONE parameter at the
// SAPI boundary, A-TH14; call-sites grep-gated by gate-lever-pins.sh).
pub use vm::{main_unit_acquire, run_source_probed, MainAcquireError, MainLever, MainUnit};
// A-PP20 (Council WP-83): entry-probe test surface for the worker's
// link-fatal-never-published tooth (counters + direct key probe).
pub use vm::{uc_main_key_in_cache, uc_main_stats};
// KL-82-2: the retained-walk positive control — run by a php-server test
// (php-runtime lib tests cannot link the mem-census dump).
#[cfg(feature = "mem-census")]
pub use vm::retained_walk_selftest;
// A-AH29 (Council WP-83): force-link mimalloc into the LIB-test binary —
// php_types::memcensus's mi_* externs resolve only against it. Dev-only
// (dev-dependency): production binaries own their allocator, unchanged.
#[cfg(test)]
use mimalloc as _;

// WP-77.6.5.2: Expose Vm lifecycle methods and construction for worker-pool integration
pub use compile::{compile_program, CompileError};
pub use vm::{RetainSet, Vm, vm_new};
pub use php_types::{Zval, PhpError};
