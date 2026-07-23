//! Register-lowering pass — Leva B stage 1 (REGISTER_BYTECODE_PLAN.md §5).
//!
//! The register model extends the frame's slot file: the compiler computes
//! per-function `max_temps` and the VM sizes the frame `n_slots + max_temps`,
//! so a "register" is an ordinary slot with a static index — no new storage,
//! no new GC invalidation (plan §4). This pass rewrites local produce→consume
//! sequences (LoadSlot/PushConst/linear temporaries) into direct operands on
//! the HOT ops ([`crate::bytecode::Operand`]) and elides dead Push/Pop/Dup.
//!
//! Stage 1 ships the pass EMPTY on purpose: the dual-mode plumbing (env flag,
//! unit-cache key, frame sizing, `Operand` type) must be proven zero-delta —
//! empty bytecode diff at flag off, gate22 green by name, A/B "infra present
//! but off" = zero noise — before any op is rewritten (stage 2: Binary/CmpJmp
//! direct operands). Non-negotiables for the real pass live in plan §3: never
//! reorder past an observable op (diagnostics flush AT the faulting op,
//! WP-33), RHS-first evaluation order (WP-14), statement-boundary Sweep, and
//! Ref-capable slots go through the generic form.

use crate::bytecode::Func;

/// Whether `PHPR_REG_LOWER` is set: per-process opt-in for the
/// register-lowering pass. Read once (mirrors `gc_verify_enabled`); the
/// unit-cache key carries this mode, so a unit compiled in one mode can never
/// serve a process running the other.
pub(crate) fn enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PHPR_REG_LOWER").is_some())
}

/// The pass. Stage 1: intentionally empty (see module doc) — called from
/// `compile_body` behind [`enabled`] so the call-site wiring is already the
/// one stage 2 will use.
pub(super) fn lower_func(_f: &mut Func) {}

/// Whether `PHPR_DUMP_OPS` is set: dump every compiled unit's bytecode to
/// stderr. Compile-time-only diagnostic for this arc: diff a flag-off dump
/// against a flag-on dump to prove a stage's rewrite is a no-op (stage 1) or
/// inspect exactly what it rewrote (stage 2+).
fn dump_enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PHPR_DUMP_OPS").is_some())
}

/// Dump a compiled unit's bytecode to stderr (gated on `PHPR_DUMP_OPS`).
/// Scope: main, functions, closures, class methods and prop-init thunks —
/// the bodies the lowering pass can touch. Reflection/const/attribute thunks
/// and property-hook bodies are NOT dumped (cold, or reachable only through
/// reflection); a stage that rewrites those needs to widen this first.
pub(super) fn dump_module_ops(m: &crate::bytecode::Module) {
    if !dump_enabled() {
        return;
    }
    use std::io::Write;
    let err = std::io::stderr();
    let mut w = err.lock();
    let _ = writeln!(w, "== unit {} ==", String::from_utf8_lossy(&m.file));
    dump_func(&mut w, "{main}", &m.main);
    for f in &m.functions {
        dump_func(&mut w, &format!("fn {}", String::from_utf8_lossy(&f.name)), f);
    }
    for (i, f) in m.closures.iter().enumerate() {
        dump_func(&mut w, &format!("closure#{i}"), f);
    }
    for c in &m.classes {
        let cname = String::from_utf8_lossy(&c.name);
        for meth in &c.methods {
            let label = format!("{cname}::{}", String::from_utf8_lossy(&meth.name));
            dump_func(&mut w, &label, &meth.func);
        }
        if let Some(pi) = &c.prop_init {
            dump_func(&mut w, &format!("{cname}::{{prop-init}}"), pi);
        }
    }
}

fn dump_func(w: &mut impl std::io::Write, label: &str, f: &Func) {
    let _ = writeln!(w, "-- {label} n_slots={} max_temps={} --", f.n_slots, f.max_temps);
    for (i, op) in f.ops.iter().enumerate() {
        let _ = writeln!(w, "{i:04} {op:?}");
    }
    for (i, c) in f.consts.iter().enumerate() {
        let _ = writeln!(w, "cst{i:03} {c:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Stage-1 guarantee: no compile path emits register temps yet, and the
    /// (empty) pass is an identity on compiled bodies — the bytecode diff at
    /// flag-off vs flag-on is empty by this very equality.
    #[test]
    fn stage1_pass_is_identity_and_no_temps() {
        let src = br#"<?php
        function f($a, $b) { $t = $a + $b; if ($t > 3) { echo "x"; } return $t . "s"; }
        class C { public $p = 1; public function m() { return $this->p + 1; } }
        $g = function ($x) { return $x * 2; };
        echo f(1, 2), (new C)->m(), $g(3);
        "#;
        let program = crate::lower_source(b"t.php", src).expect("lowers");
        let module = crate::compile::compile_program(&program, &crate::builtin::Registry::default())
            .expect("compiles");
        let mut all: Vec<&Func> = vec![&module.main];
        all.extend(module.functions.iter().map(|f| f.as_ref()));
        all.extend(module.closures.iter());
        for c in &module.classes {
            all.extend(c.methods.iter().map(|m| &m.func));
        }
        assert!(all.len() >= 4, "probe should cover main, fn, closure, method");
        for f in all {
            assert_eq!(f.max_temps, 0, "stage 1 emits no temps ({:?})", f.name);
            let mut lowered = f.clone();
            lower_func(&mut lowered);
            assert_eq!(&lowered, f, "stage-1 pass must be an identity ({:?})", f.name);
        }
    }
}
