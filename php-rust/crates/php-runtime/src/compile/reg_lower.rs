//! Register-lowering pass — Leva B stage 2 v3 "raw registers"
//! (REGISTER_BYTECODE_PLAN.md §5; post-mortem v1/v2 in
//! sessions/WP_SESSION_44.md).
//!
//! v3 design rule (the discriminating experiment after the enum-operand
//! hybrid measured +1.2% consistent on A/B): the run_loop must see ZERO
//! runtime operand dispatch. Every fused shape is its own MONOMORPHIC op
//! with bare u16 indices ([`Op::BinarySS`]/[`Op::BinarySSDst`]/
//! [`Op::BinarySC`]/[`Op::BinarySCDst`]/[`Op::BinaryDst`]/[`Op::CmpJmpSS`]/
//! [`Op::CmpJmpSC`]); the compiler does all the resolution here. Shapes
//! outside this set are NOT rewritten (no stack-lhs source folds, no 1:1
//! CmpJmpConst rename): the stack forms keep their existing monomorphic
//! handlers, so no polymorphism is added anywhere.
//!
//! Fold rules:
//! - sources: `LoadVar` (only when the name const is byte-identical to
//!   `slot_names[slot]`, so the fused handler re-synthesises the exact
//!   "Undefined variable" warning) and `PushConst`; `LoadSlot` (silent,
//!   cold) is never folded.
//! - const is ALWAYS rhs: a written const-lhs folds only for commutative
//!   ops (same op, operands swapped — legal only for a non-diagnosing
//!   scalar const, or the coercion-diag ORDER could flip) or mirrorable
//!   comparisons (Lt↔Gt, Le↔Ge; Eq-family unchanged — compares emit no
//!   coercion diags). Spaceship is not mirrorable → no fold.
//! - dst folds: `Binary, StoreSlot s` and `Binary, Dup, StoreSlot s, Pop`
//!   sink into the `*Dst` forms (net stack/slot/gc_note effect identical —
//!   the only elision is the transient duplicate, which no longer exists
//!   to note).
//!
//! Window guards (plan §3): one source line per window (diagnostic
//! parity), no jump target or exc-region boundary mid-window (the head MAY
//! be a target), folded indices fit u16. Compaction remaps every `Addr`
//! (op stream + exc table); addresses past the original length
//! (`Addr::MAX` jump-threading terminals) are preserved. `max_temps`
//! stays 0.

use crate::bytecode::{Addr, Const, Func, Op};
use crate::hir::BinOp;

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
/// must be added HERE or the pass corrupts it.
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
        Op::CmpJmp { addr, .. }
        | Op::CmpJmpConst { addr, .. }
        | Op::CmpJmpSS { addr, .. }
        | Op::CmpJmpSC { addr, .. } => f(addr),
        Op::IterNext { end, .. } | Op::IterNextRef { end, .. } => f(end),
        _ => {}
    }
}

/// A foldable `LoadVar` source: index fits u16 and the name const equals
/// `slot_names[slot]` (warning parity). `LoadSlot` is silent — never folded.
fn fold_slot(f: &Func, i: usize) -> Option<u16> {
    match &f.ops[i] {
        Op::LoadVar { slot, name } if *slot <= u16::MAX as u32 => {
            match &f.consts[*name as usize] {
                Const::Str(s)
                    if f.slot_names.get(*slot as usize).map(|n| &n[..]) == Some(s.as_bytes()) =>
                {
                    Some(*slot as u16)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// A foldable `PushConst` source (index fits u16).
fn fold_const(f: &Func, i: usize) -> Option<u16> {
    match &f.ops[i] {
        Op::PushConst(c) if *c <= u16::MAX as u32 => Some(*c as u16),
        _ => None,
    }
}

/// Whether the const can never emit a coercion diagnostic (scalar
/// non-string): swapping it to the rhs of a commutative op is then
/// diag-order-safe.
fn const_nondiag(f: &Func, c: u16) -> bool {
    matches!(
        f.consts[c as usize],
        Const::Null | Const::Bool(_) | Const::Int(_) | Const::Float(_)
    )
}

/// Mirror a comparison so its operands can swap sides (const to rhs).
/// Comparisons emit no coercion diagnostics, so the swap is unobservable.
/// `None` = not mirrorable (Spaceship, arithmetic, …).
fn mirror_cmp(b: BinOp) -> Option<BinOp> {
    Some(match b {
        BinOp::Eq | BinOp::NotEq | BinOp::Identical | BinOp::NotIdentical => b,
        BinOp::Lt => BinOp::Gt,
        BinOp::Le => BinOp::Ge,
        BinOp::Gt => BinOp::Lt,
        BinOp::Ge => BinOp::Le,
        _ => return None,
    })
}

/// Operand order is free for these — swapping a scalar const to the rhs
/// changes nothing observable (see [`const_nondiag`] for the diag guard).
fn is_commutative(b: BinOp) -> bool {
    matches!(b, BinOp::Add | BinOp::Mul | BinOp::BitAnd | BinOp::BitOr | BinOp::BitXor)
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

/// Source shape of a Binary window, pre-resolved by the scanner.
enum BinKind {
    SS(u16, u16),
    SC(u16, u16),
    Stack,
}

/// Recognise the longest fusable window starting at `i`; `(op, width)` —
/// width 1 with the original op when nothing fuses.
fn fuse_window(f: &Func, blocked: &[bool], i: usize) -> (Op, usize) {
    let n = f.ops.len();
    let line = f.lines[i];
    let free = |j: usize| j < n && !blocked[j] && f.lines[j] == line;

    if let Some(a) = fold_slot(f, i) {
        if free(i + 1) && free(i + 2) {
            // [LoadVar, LoadVar, Binary|CmpJmp]
            if let Some(b) = fold_slot(f, i + 1) {
                match &f.ops[i + 2] {
                    Op::Binary(op) => return bin_dst(f, &free, i, 3, BinKind::SS(a, b), *op),
                    Op::CmpJmp { op, addr, when } => {
                        return (
                            Op::CmpJmpSS { op: *op, l: a, r: b, addr: *addr, when: *when },
                            3,
                        )
                    }
                    _ => {}
                }
            }
            // [LoadVar, PushConst, Binary]
            if let Some(c) = fold_const(f, i + 1) {
                if let Op::Binary(op) = &f.ops[i + 2] {
                    return bin_dst(f, &free, i, 3, BinKind::SC(a, c), *op);
                }
            }
        }
        // [LoadVar, CmpJmpConst] → slot vs const compare (mirror const-lhs)
        if free(i + 1) {
            if let Op::CmpJmpConst { op, cidx, addr, when, const_lhs } = &f.ops[i + 1] {
                if *cidx <= u16::MAX as u32 {
                    let op2 = if *const_lhs { mirror_cmp(*op) } else { Some(*op) };
                    if let Some(op2) = op2 {
                        return (
                            Op::CmpJmpSC {
                                op: op2,
                                slot: a,
                                cidx: *cidx as u16,
                                addr: *addr,
                                when: *when,
                            },
                            2,
                        );
                    }
                }
            }
        }
    }
    // [PushConst, LoadVar, Binary] — const written first: fold only when the
    // swap is unobservable (commutative op with non-diagnosing scalar const,
    // or mirrorable comparison).
    if let Some(c) = fold_const(f, i) {
        if free(i + 1) && free(i + 2) && const_nondiag(f, c) {
            if let Some(a) = fold_slot(f, i + 1) {
                if let Op::Binary(op) = &f.ops[i + 2] {
                    if is_commutative(*op) {
                        return bin_dst(f, &free, i, 3, BinKind::SC(a, c), *op);
                    }
                    if let Some(m) = mirror_cmp(*op) {
                        return bin_dst(f, &free, i, 3, BinKind::SC(a, c), m);
                    }
                }
            }
        }
    }
    // Bare Binary: wins only with an assign-and-discard tail.
    if let Op::Binary(op) = &f.ops[i] {
        return bin_dst(f, &free, i, 1, BinKind::Stack, *op);
    }
    (f.ops[i].clone(), 1)
}

/// Extend a Binary window over an assign-and-discard tail (`StoreSlot s` or
/// `Dup, StoreSlot s, Pop`) and emit the matching monomorphic variant. With
/// no tail: `SS`/`SC` push, a bare stack Binary stays as it is (nothing to
/// win).
fn bin_dst(
    f: &Func,
    free: &dyn Fn(usize) -> bool,
    i: usize,
    w: usize,
    kind: BinKind,
    op: BinOp,
) -> (Op, usize) {
    let j = i + w;
    let tail: Option<(u16, usize)> = if free(j) {
        match &f.ops[j] {
            Op::StoreSlot(s) if *s <= u16::MAX as u32 => Some((*s as u16, 1)),
            Op::Dup if free(j + 1) && free(j + 2) => {
                match (&f.ops[j + 1], &f.ops[j + 2]) {
                    (Op::StoreSlot(s), Op::Pop) if *s <= u16::MAX as u32 => {
                        Some((*s as u16, 3))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    };
    match (kind, tail) {
        (BinKind::SS(l, r), Some((dst, e))) => (Op::BinarySSDst { op, l, r, dst }, w + e),
        (BinKind::SS(l, r), None) => (Op::BinarySS { op, l, r }, w),
        (BinKind::SC(slot, cidx), Some((dst, e))) => {
            (Op::BinarySCDst { op, slot, cidx, dst }, w + e)
        }
        (BinKind::SC(slot, cidx), None) => (Op::BinarySC { op, slot, cidx }, w),
        (BinKind::Stack, Some((dst, e))) => (Op::BinaryDst { op, dst }, w + e),
        (BinKind::Stack, None) => (f.ops[i].clone(), 1),
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
    use crate::builtin::Registry;
    use crate::bytecode::Module;

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

    fn is_reg_form(o: &Op) -> bool {
        matches!(
            o,
            Op::BinarySS { .. }
                | Op::BinarySSDst { .. }
                | Op::BinarySC { .. }
                | Op::BinarySCDst { .. }
                | Op::BinaryDst { .. }
                | Op::CmpJmpSS { .. }
                | Op::CmpJmpSC { .. }
        )
    }

    /// v3 shape: hot windows become the specialized monomorphic forms and
    /// no fused compare window survives un-rewritten; no register temps.
    #[test]
    fn stage2v3_rewrites_hot_windows() {
        let m = compile(
            br#"<?php
            function f($a, $b) {
                $c = $a + $b;
                if ($a > $b) { $c = $c * 2; }
                if ($a == 3) { return -1; }
                if (3 < $b) { $c = $c + 1; }
                return $c . "s";
            }
            echo f(1, 2), f(4, 2), f(3, 0), f(1, 7);
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
        assert!(has(&|o| matches!(o, Op::BinarySSDst { .. })), "$c=$a+$b: {:#?}", lf.ops);
        assert!(has(&|o| matches!(o, Op::CmpJmpSS { .. })), "$a>$b");
        assert!(has(&|o| matches!(o, Op::CmpJmpSC { .. })), "$a==3 / 3<$b (mirrored)");
        assert!(
            has(&|o| matches!(o, Op::BinarySCDst { .. })),
            "$c*2 / $c+1 (const fold with dst)"
        );
        // No compare window with a foldable LoadVar in front may survive.
        for (x, y) in lf.ops.iter().zip(lf.ops.iter().skip(1)) {
            assert!(
                !(matches!(x, Op::LoadVar { .. })
                    && matches!(y, Op::CmpJmpConst { .. } | Op::CmpJmp { .. })),
                "unfused compare window"
            );
        }
        for f in all_funcs(&lm) {
            assert_eq!(f.max_temps, 0, "v3 emits no temps");
        }
        assert_eq!(run(&m), run(&lm));
    }

    /// A compare whose lhs comes from the stack (no foldable producer) keeps
    /// the monomorphic WP-34 CmpJmpConst — no-elision rewrites are the
    /// measured v1 regression.
    #[test]
    fn stage2v3_stack_lhs_compare_keeps_cmpjmpconst() {
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
            !lh.ops.iter().any(|o| matches!(o, Op::CmpJmpSS { .. } | Op::CmpJmpSC { .. })),
            "no fold available in h"
        );
        assert_eq!(run(&m), run(&lm));
    }

    /// v3 parity battery: control flow that emits `Addr`s (loops, if/else,
    /// try/catch/finally, foreach by value and by ref, static guard, param
    /// defaults, ?? / ?:), plus the diagnostic paths the folds must preserve
    /// (undefined-variable warning through slot_names, DivisionByZeroError
    /// at the fused op, references, self-assign, const-first mirror and
    /// non-commutative const-first NON-fold, numeric-string coercions) —
    /// lowered output must equal stack output, and every remapped address
    /// must land inside the function.
    #[test]
    fn stage2v3_behavioral_parity_and_remap() {
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
            br#"<?php $a=5; $b = 3 + $a; $c = 3 - $a; echo $b, ",", $c; if (3 < $a) { echo "m"; } if (3 <= $a) { echo "e"; } if ("x" == $a) { echo "s"; } else { echo "n"; }"#,
            br#"<?php $w = "7"; echo 3 + $w, 3 * $w, 10 - $w, "3" . $w; if (10 > $w) echo "g";"#,
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
    /// wider element — D-cache). Pinned so a future field addition trips
    /// this consciously.
    #[test]
    fn stage2v3_op_size_unchanged() {
        assert_eq!(std::mem::size_of::<Op>(), 48, "Op must not widen");
    }

    /// Dual-mode guard: with the env flag unset, compilation never emits a
    /// register form and the frame contract is unchanged (flag-off bytecode
    /// stays byte-identical to WP-43 — proven end-to-end by the dump diff).
    #[test]
    fn stage2v3_flag_off_emits_no_register_forms() {
        if enabled() {
            return; // an exported PHPR_REG_LOWER would invert the premise
        }
        let m = compile(br#"<?php function f($a,$b){ $c=$a+$b; if($c>3){$c=$c*2;} return $c; } echo f(1,2);"#);
        for f in all_funcs(&m) {
            assert_eq!(f.max_temps, 0);
            assert!(
                !f.ops.iter().any(is_reg_form),
                "flag-off compile must stay stack-based"
            );
        }
    }
}
