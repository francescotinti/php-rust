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
// A-DS51 fase 1 (Council WP-92): PURE model of the Zend inheritance /
// Liskov signature checks over hand-built signature models. NOT WIRED —
// nothing calls it yet; fase 2 does the wiring.
pub mod lsp_check;
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

/// A-PE-100-2 (Concilio WP-100): sigillo EAGER del modo register-lowering a
/// un confine NOMINATO — il primo atto dei due main (CLI e server), mai lazy
/// alla prima compile. PHP `putenv()` chiama `std::env::set_var` in-process:
/// senza sigillo, il modo di un processo server long-lived sarebbe deciso
/// dalla prima richiesta che compila (stato di richiesta promosso a
/// configurazione di motore). Col sigillo l'autore del modo è SOLO
/// l'ambiente allo spawn del processo.
pub fn seal_reg_lower_mode() {
    let _ = compile::reg_lower::enabled();
}

/// Il default nominato del contratto di modo (S-100 punto 1): ciò che vale
/// quando `PHPR_REG_LOWER` è ASSENTE. Esportato perché i denti fuori-crate
/// (anti-putenv, funnel, launcher) derivino i bracci dal CONTRATTO invece
/// che da una premessa cablata che il flip del default renderebbe falsa.
pub use compile::reg_lower::DEFAULT_ON as REG_LOWER_DEFAULT_ON;
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
pub use vm::{uc_entry_count, uc_main_entry_count, uc_main_key_in_cache, uc_main_stats};
// A-PP34 (Council WP-87): displacement counters — the compensated
// insert+evict window of the a_pp20 tooth (membership, not cardinality).
pub use vm::uc_displacement_stats;
// A-DL24 (Council WP-85): per-thread unit-cache rows for the worker arm's
// teardown dump — mem-census instrumentation builds only.
#[cfg(feature = "mem-census")]
pub use vm::memcensus_unitcache_main_rows;
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
pub use vm::{RetainSet, Vm, VmGate, vm_new};
// A-MS26/A-TH28 (Council WP-85): the probe mint is re-exported ONLY in
// test/feature builds — a campaign/parity build has no reachable pub mint
// (KH85-1 closed by cfg, belt pins the alias surface ==0).
#[cfg(any(test, feature = "vm-gate-probe"))]
pub use vm::gate::vm_gate_probe;
pub use php_types::{Zval, PhpError};
