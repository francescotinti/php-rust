//! Register-lowering pass — Leva B stage 2 (REGISTER_BYTECODE_PLAN.md §5).
//!
//! The register model extends the frame's slot file: the compiler computes
//! per-function `max_temps` and the VM sizes the frame `n_slots + max_temps`,
//! so a "register" is an ordinary slot with a static index — no new storage,
//! no new GC invalidation (plan §4). This pass rewrites local produce→consume
//! sequences into direct operands on the HOT ops
//! ([`crate::bytecode::Operand`]) and elides the dead stack traffic.
//!
//! Stage 2 (this file): windows ending in `Binary`/`CmpJmp`/`CmpJmpConst`
//! become [`Op::BinaryReg`]/[`Op::CmpJmpReg`]:
//!
//! - source folds: `LoadVar` → `Operand::Slot` (only when the LoadVar's name
//!   const is byte-identical to `slot_names[slot]`, so the handler can
//!   re-synthesise the exact "Undefined variable" warning) and `PushConst` →
//!   `Operand::Const`. `LoadSlot` (silent temps, cold: 1.9M vs LoadVar's
//!   58.4M per census) is never folded — `Operand::Slot` always carries
//!   LoadVar semantics.
//! - dst folds: `Binary, StoreSlot s` and `Binary, Dup, StoreSlot s, Pop`
//!   sink into `dst: Slot(s)` (the census assign-and-discard bigrams).
//! - absorption by SHAPE (plan §3.7 — substitution, never coexistence):
//!   `CmpJmpReg` owns every *folded* shape (a `CmpJmpConst` preceded by a
//!   foldable LoadVar is absorbed into it), while the pure stack-lhs
//!   `CmpJmpConst` keeps its WP-34 monomorphic handler — the 1:1 rewrite
//!   with no elision was measured at +1.3% consistent (WP-44 first A/B):
//!   polymorphic operand dispatch on the hottest compare site is pure cost.
//!   A `Binary` with no folded source and no dst stays `Op::Binary`.
//!
//! Window guards (plan §3 non-negotiables): every op of a window shares one
//! source line (diagnostic parity — the fused op reports the same line), no
//! jump target or exc-region boundary lands mid-window (the window head MAY
//! be a target: jumping there runs the whole sequence, unchanged semantics),
//! and folded indices must fit `Operand`'s u16 (Slot/ConstIdx are u32).
//! Contiguity does the rest: nothing can observe the elided pushes, the
//! operand reads stay in evaluation order inside the handler, and
//! statement-boundary `Sweep` ops are never part of a window. Compaction
//! remaps every `Addr` in the op stream and the exc table; addresses past
//! the original length (`Addr::MAX` jump-threading terminals, WP-34) are
//! preserved verbatim. `max_temps` stays 0 — register temps arrive with
//! stage 3.

use crate::bytecode::{Addr, Const, Func, Op, Operand};

/// Whether `PHPR_REG_LOWER` is set: per-process opt-in for the
/// register-lowering pass. Read once (mirrors `gc_verify_enabled`); the
/// unit-cache key carries this mode, so a unit compiled in one mode can never
/// serve a process running the other.
pub(crate) fn enabled() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("PHPR_REG_LOWER").is_some())
}

/// Visit every jump address the op carries. The single authority both the
/// target-collection and the remap phase use — a new `Addr`-bearing variant
/// must be added HERE or the pass corrupts it (checked by the behavioral
/// tests below, which cover every control-flow construct that emits one).
fn visit_addrs(op: &mut Op, f: &mut impl FnMut(&mut Addr)) {
    match op {
        Op::FillDefault { skip, .. } | Op::StaticGuard { skip, .. } => f(skip),
        Op::CatchMatch { body, .. } => f(body),
        Op::EndFinally { after } => f(after),
        Op::ParkJump(a)
        | Op::Jump(a)
        | Op::JumpIfFalse(a)
        | Op::JumpIfTrue(a)
        | Op::JumpIfNotNull(a)
        | Op::JumpIfNull(a) => f(a),
        Op::CmpJmp { addr, .. } | Op::CmpJmpConst { addr, .. } | Op::CmpJmpReg { addr, .. } => {
            f(addr)
        }
        Op::IterNext { end, .. } | Op::IterNextRef { end, .. } => f(end),
        _ => {}
    }
}

/// A foldable *source* op: `LoadVar` (guarded: index fits u16, and the name
/// const equals `slot_names[slot]` so the fused handler's warning is
/// byte-identical) or `PushConst` (index fits u16). `LoadSlot` is silent —
/// never folded (see module doc).
fn src_operand(f: &Func, i: usize) -> Option<Operand> {
    match &f.ops[i] {
        Op::PushConst(c) if *c <= u16::MAX as u32 => Some(Operand::Const(*c as u16)),
        Op::LoadVar { slot, name } if *slot <= u16::MAX as u32 => {
            match &f.consts[*name as usize] {
                Const::Str(s)
                    if f.slot_names.get(*slot as usize).map(|n| &n[..]) == Some(s.as_bytes()) =>
                {
                    Some(Operand::Slot(*slot as u16))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// The pass (called from `compile_body` behind [`enabled`]): scan for
/// fusable windows, rebuild `ops`/`lines`, remap every address.
pub(super) fn lower_func(f: &mut Func) {
    let n = f.ops.len();
    if n == 0 {
        return;
    }
    debug_assert_eq!(f.lines.len(), n, "lines parallel to ops");
    // Positions a window may not absorb: jump targets and exc boundaries.
    let mut blocked = vec![false; n + 1];
    {
        let mut mark = |a: Addr| {
            if (a as usize) <= n {
                blocked[a as usize] = true;
            }
        };
        for r in &f.exc_table {
            mark(r.start);
            mark(r.end);
            mark(r.target);
        }
        for op in &mut f.ops {
            visit_addrs(op, &mut |a| mark(*a));
        }
    }
    let mut new_ops: Vec<Op> = Vec::with_capacity(n);
    let mut new_lines = Vec::with_capacity(n);
    let mut map = vec![0u32; n + 1];
    let mut i = 0usize;
    while i < n {
        let (op, w) = fuse_window(f, &blocked, i);
        for k in i..i + w {
            map[k] = new_ops.len() as u32;
        }
        new_lines.push(f.lines[i]);
        new_ops.push(op);
        i += w;
    }
    map[n] = new_ops.len() as u32;
    for op in &mut new_ops {
        visit_addrs(op, &mut |a| {
            if (*a as usize) <= n {
                *a = map[*a as usize];
            }
        });
    }
    for r in &mut f.exc_table {
        for a in [&mut r.start, &mut r.end, &mut r.target] {
            if (*a as usize) <= n {
                *a = map[*a as usize];
            }
        }
    }
    f.ops = new_ops;
    f.lines = new_lines;
}

/// Recognise the longest fusable window starting at `i`; `(op, width)` —
/// width 1 with the original op when nothing fuses.
fn fuse_window(f: &Func, blocked: &[bool], i: usize) -> (Op, usize) {
    let n = f.ops.len();
    let line = f.lines[i];
    let free = |j: usize| j < n && !blocked[j] && f.lines[j] == line;

    if let Some(l) = src_operand(f, i) {
        // [src, src, Binary|CmpJmp] — both operands folded.
        if free(i + 1) && free(i + 2) {
            if let Some(r) = src_operand(f, i + 1) {
                match &f.ops[i + 2] {
                    Op::Binary(b) => return with_dst(f, blocked, i, 3, *b, l, r),
                    Op::CmpJmp { op, addr, when } => {
                        return (
                            Op::CmpJmpReg { op: *op, l, r, addr: *addr, when: *when },
                            3,
                        )
                    }
                    _ => {}
                }
            }
        }
        // [src, Binary|CmpJmp|CmpJmpConst] — rhs folded, lhs from the stack
        // (or from the CmpJmpConst literal, per const_lhs).
        if free(i + 1) {
            match &f.ops[i + 1] {
                Op::Binary(b) => return with_dst(f, blocked, i, 2, *b, Operand::Stack, l),
                Op::CmpJmp { op, addr, when } => {
                    return (
                        Op::CmpJmpReg { op: *op, l: Operand::Stack, r: l, addr: *addr, when: *when },
                        2,
                    )
                }
                Op::CmpJmpConst { op, cidx, addr, when, const_lhs }
                    if *cidx <= u16::MAX as u32 =>
                {
                    let c = Operand::Const(*cidx as u16);
                    let (lo, ro) = if *const_lhs { (c, l) } else { (l, c) };
                    return (
                        Op::CmpJmpReg { op: *op, l: lo, r: ro, addr: *addr, when: *when },
                        2,
                    );
                }
                _ => {}
            }
        }
    }
    match &f.ops[i] {
        // A bare Binary still wins if its result sinks straight into a slot.
        Op::Binary(b) => with_dst(f, blocked, i, 1, *b, Operand::Stack, Operand::Stack),
        // A bare CmpJmpConst (stack lhs, no fold) STAYS: rewriting it 1:1
        // into CmpJmpReg elides nothing and turns the hottest monomorphic
        // handler (ThisPropGet→CmpJmpConst, 29.9M/run) into a polymorphic
        // one — measured +1.3% consistent on the WP-44 first-cut A/B.
        // "Substitution, never coexistence" (plan §3.7) applies per operand
        // SHAPE: CmpJmpReg owns the folded shapes, CmpJmpConst the stack one.
        _ => (f.ops[i].clone(), 1),
    }
}

/// Extend a would-be `BinaryReg` window over an assign-and-discard tail:
/// `StoreSlot s` or `Dup, StoreSlot s, Pop` right after the Binary sink into
/// `dst: Slot(s)` (net stack/slot/gc_note effect identical — the only elision
/// is the transient duplicate, which no longer exists to note). With no tail
/// and nothing folded, the original stack `Binary` is kept (substitution
/// only where there is a win).
fn with_dst(
    f: &Func,
    blocked: &[bool],
    i: usize,
    w: usize,
    b: crate::hir::BinOp,
    l: Operand,
    r: Operand,
) -> (Op, usize) {
    let n = f.ops.len();
    let line = f.lines[i];
    let free = |j: usize| j < n && !blocked[j] && f.lines[j] == line;
    let j = i + w;
    if free(j) {
        if let Op::StoreSlot(s) = &f.ops[j] {
            if *s <= u16::MAX as u32 {
                return (Op::BinaryReg { op: b, l, r, dst: Operand::Slot(*s as u16) }, w + 1);
            }
        }
        if matches!(&f.ops[j], Op::Dup) && free(j + 1) && free(j + 2) {
            if let (Op::StoreSlot(s), Op::Pop) = (&f.ops[j + 1], &f.ops[j + 2]) {
                if *s <= u16::MAX as u32 {
                    return (
                        Op::BinaryReg { op: b, l, r, dst: Operand::Slot(*s as u16) },
                        w + 3,
                    );
                }
            }
        }
    }
    if w == 1 {
        (f.ops[i].clone(), 1)
    } else {
        (Op::BinaryReg { op: b, l, r, dst: Operand::Stack }, w)
    }
}

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
    use crate::bytecode::Module;
    use crate::builtin::Registry;

    fn compile(src: &[u8]) -> Module {
        let program = crate::lower_source(b"t.php", src).expect("lowers");
        crate::compile::compile_program(&program, &Registry::default()).expect("compiles")
    }

    /// Apply the pass to every body of a compiled module (bypassing the env
    /// flag), mirroring the `compile_body` funnel.
    fn lowered(m: &Module) -> Module {
        let mut m2 = m.clone();
        lower_func(&mut m2.main);
        for f in &mut m2.functions {
            let mut nf = (**f).clone();
            lower_func(&mut nf);
            *f = std::rc::Rc::new(nf);
        }
        for f in &mut m2.closures {
            lower_func(f);
        }
        for c in &mut m2.classes {
            let mut nc = (**c).clone();
            for meth in &mut nc.methods {
                lower_func(&mut meth.func);
            }
            if let Some(pi) = &mut nc.prop_init {
                lower_func(pi);
            }
            *c = std::rc::Rc::new(nc);
        }
        m2
    }

    fn all_funcs(m: &Module) -> Vec<&Func> {
        let mut all: Vec<&Func> = vec![&m.main];
        all.extend(m.functions.iter().map(|f| f.as_ref()));
        all.extend(m.closures.iter());
        for c in &m.classes {
            all.extend(c.methods.iter().map(|meth| &meth.func));
        }
        all
    }

    /// Run and return the CLI-faithful stream (diagnostics inline) — the
    /// parity comparison must cover the warning text/order too.
    fn run(m: &Module) -> Vec<u8> {
        let reg = Registry::default();
        let out = crate::vm::run_module(m, &reg);
        assert!(out.fatal.is_none(), "unexpected fatal: {:?}", out.fatal);
        out.rendered
    }

    /// Stage-2 shape: hot windows become register forms, CmpJmpConst is
    /// fully absorbed, and no register temps are ever emitted.
    #[test]
    fn stage2_rewrites_hot_windows() {
        let m = compile(
            br#"<?php
            function f($a, $b) {
                $c = $a + $b;
                if ($a > $b) { $c = $c * 2; }
                if ($a == 3) { return -1; }
                return $c . "s";
            }
            echo f(1, 2), f(4, 2), f(3, 0);
            "#,
        );
        let lm = lowered(&m);
        let lf = lm
            .functions
            .iter()
            .find(|f| f.name.as_ref() == b"f")
            .expect("fn f present")
            .as_ref();
        let has = |p: &dyn Fn(&Op) -> bool| lf.ops.iter().any(|o| p(o));
        assert!(
            has(&|o| matches!(
                o,
                Op::BinaryReg { l: Operand::Slot(_), r: Operand::Slot(_), dst: Operand::Slot(_), .. }
            )),
            "expected slot+slot→slot BinaryReg: {:#?}",
            lf.ops
        );
        assert!(
            has(&|o| matches!(o, Op::CmpJmpReg { l: Operand::Slot(_), r: Operand::Slot(_), .. })),
            "expected slot,slot CmpJmpReg"
        );
        assert!(
            has(&|o| matches!(
                o,
                Op::CmpJmpReg { r: Operand::Const(_), .. }
                    | Op::CmpJmpReg { l: Operand::Const(_), .. }
            )),
            "expected a Const-side CmpJmpReg (absorbed CmpJmpConst)"
        );
        // Folded shapes only: a CmpJmpConst *preceded by a foldable LoadVar*
        // is absorbed; no fused window may survive un-rewritten in f.
        for (a, b) in lf.ops.iter().zip(lf.ops.iter().skip(1)) {
            assert!(
                !(matches!(a, Op::LoadVar { .. }) && matches!(b, Op::CmpJmpConst { .. })),
                "LoadVar→CmpJmpConst window left unfused"
            );
        }
        for f in all_funcs(&lm) {
            assert_eq!(f.max_temps, 0, "stage 2 emits no temps");
        }
        // Behavior identical to the stack forms.
        assert_eq!(run(&m), run(&lm));
    }

    /// A compare whose lhs comes from the stack (no foldable producer) keeps
    /// the monomorphic WP-34 CmpJmpConst — the 1:1 no-elision rewrite was a
    /// measured regression (see module doc).
    #[test]
    fn stage2_stack_lhs_compare_keeps_cmpjmpconst() {
        let m = compile(
            br#"<?php
            function g($a) { return $a + 1; }
            function h($a) { if (g($a) == 3) { return 1; } return 2; }
            echo h(2), h(5);
            "#,
        );
        let lm = lowered(&m);
        let lh = lm
            .functions
            .iter()
            .find(|f| f.name.as_ref() == b"h")
            .expect("fn h present");
        assert!(
            lh.ops.iter().any(|o| matches!(o, Op::CmpJmpConst { .. })),
            "stack-lhs compare must stay CmpJmpConst: {:#?}",
            lh.ops
        );
        assert!(
            !lh.ops.iter().any(|o| matches!(o, Op::CmpJmpReg { .. })),
            "no fold available in h — no CmpJmpReg expected"
        );
        assert_eq!(run(&m), run(&lm));
    }

    /// Stage-2 parity battery: every control-flow construct that emits an
    /// `Addr` (loops, if/else chains, try/catch/finally, foreach by value
    /// and by ref, static guard, param defaults, ?? / ?: null jumps),
    /// plus the diagnostic paths the fold must preserve (undefined-variable
    /// warning through slot_names, DivisionByZeroError at the fused op,
    /// references, self-assign) — lowered output must equal stack output,
    /// and every remapped address must land inside the function.
    #[test]
    fn stage2_behavioral_parity_and_remap() {
        let snippets: &[&[u8]] = &[
            br#"<?php $s=0; for ($i=0; $i<10; $i++) { $s = $s + $i; } echo $s;"#,
            br#"<?php $i=0; while ($i < 5) { $i = $i + 1; if ($i == 3) continue; echo $i; } echo "|", $i;"#,
            br#"<?php $a=2; $b=3; try { echo $a % ($b - 3); } catch (\DivisionByZeroError $e) { echo "dbz"; } finally { echo "-f"; }"#,
            br#"<?php function g($x = 5) { static $n = 0; $n = $n + 1; return $x + $n; } echo g(), g(1), g();"#,
            br#"<?php $t = ['a'=>1,'b'=>2]; $s=''; foreach ($t as $k=>$v) { $s = $s . $k . ($v + 1); } echo $s;"#,
            br#"<?php $arr=[1,2,3]; foreach ($arr as &$v) { $v = $v * 2; } unset($v); echo $arr[0], $arr[1], $arr[2];"#,
            br#"<?php echo $u + 1; $q = $u2 . "x"; echo $q;"#,
            br#"<?php $a=1; $b=&$a; $c = $b + 1; $b = $b + 10; echo $a, ",", $c;"#,
            br#"<?php $a=1; $a = $a + 1; $a = 41 + $a > 42 ? $a * 10 : $a - 1; echo $a;"#,
            br#"<?php $x=null; $y = $x ?? 7; $z = $x ?: 9; echo $y, $z, 5 <=> 3, "10" == "1e1" ? "t" : "f";"#,
            br#"<?php $s="ab"; $n=2; if ($s == "ab" && $n > 1) { echo "y"; } if (3 == $n + 1) { echo "z"; }"#,
        ];
        for src in snippets {
            let m = compile(src);
            let lm = lowered(&m);
            for (f, orig) in all_funcs(&lm).into_iter().zip(all_funcs(&m)) {
                let (new_n, old_n) = (f.ops.len(), orig.ops.len());
                let mut check = |a: Addr| {
                    assert!(
                        (a as usize) <= new_n || (a as usize) > old_n,
                        "addr {a} out of range (new {new_n}, old {old_n}) in {:?}",
                        f.name
                    );
                };
                let mut ops = f.ops.clone();
                for op in &mut ops {
                    visit_addrs(op, &mut |a| check(*a));
                }
                for r in &f.exc_table {
                    check(r.start);
                    check(r.end);
                    check(r.target);
                }
            }
            assert_eq!(
                run(&m),
                run(&lm),
                "lowered output diverges for {}",
                String::from_utf8_lossy(src)
            );
        }
    }

    /// The register forms must not widen the Op enum (every ops Vec pays a
    /// wider element — D-cache): both fit well under the largest existing
    /// variant. Pinned so a future field addition trips this consciously.
    #[test]
    fn stage2_op_size_unchanged() {
        assert_eq!(std::mem::size_of::<Op>(), 48, "Op must not widen");
    }

    /// Dual-mode guard: with the env flag unset, compilation never emits a
    /// register form and the frame contract is unchanged (flag-off bytecode
    /// stays byte-identical to WP-43 — proven end-to-end by the dump diff).
    #[test]
    fn stage2_flag_off_emits_no_register_forms() {
        if enabled() {
            return; // an exported PHPR_REG_LOWER would invert the premise
        }
        let m = compile(br#"<?php function f($a,$b){ $c=$a+$b; if($c>3){$c=$c*2;} return $c; } echo f(1,2);"#);
        for f in all_funcs(&m) {
            assert_eq!(f.max_temps, 0);
            assert!(
                !f.ops
                    .iter()
                    .any(|o| matches!(o, Op::BinaryReg { .. } | Op::CmpJmpReg { .. })),
                "flag-off compile must stay stack-based"
            );
        }
    }
}
