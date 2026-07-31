# WP_SESSION_77_6_5_2_4.md — Council Amendments Implementation & Merge to Main

**Date**: 2026-07-31 02:00 UTC+2 (Session Start: 01:33 AM)
**Scope**: Implement 6 binding amendments from COUNCIL_WP77_6_5_2_3_REVIEWS; full test suite; merge to main
**Baseline Commit**: 7763a68 (S-77.6.5.2.3, unanimous CONCORDO from 9-chair council)
**Closure Commit**: fed30ce (S-77.6.5.2.4 amendments implemented + pushed)
**Binary Baseline**: 02aa2ad8... (unchanged; amendments documentation-only)

---

## Verdicts

### Amendments Implemented (6/6)

| Amendment | Chair | Scope | Status |
|---|---|---|---|
| A-MS1 | Matsakis | Send/Sync invariant in code | ✅ IMPLEMENTED |
| A-SK1 | Klabnik | G-APERTURA-2 gate contract in crate docs | ✅ IMPLEMENTED |
| A-AH1 | Hejlsberg | Feature-gating verification (CI guidance) | ✅ IMPLEMENTED |
| A-PP1 | Pedersen | Output-capture-before-request_end() ordering | ✅ IMPLEMENTED |
| A-DS1 | Stogov | Zend semantics isolation contract | ✅ IMPLEMENTED |
| A-BG1 | Gregg | G-APERTURA-2 measurement scope documentation | ✅ IMPLEMENTED |

### Test Verification

- **Command**: `cargo test --release`
- **Results**: 1651 passed, 1 ignored, 0 failed = 1652 total
- **Status**: ✅ PASS (baseline expectation: 1652/1652)
- **G-APERTURA-2 Gate**: ✅ PASS (byte-parity gate verified, all tests pass)

### Code Changes

- **main.rs**: +27 lines (G-APERTURA-2 gate contract, measurement scope, feature-gating verification)
- **worker_pool.rs**: +21 lines (Send/Sync safety invariant, output capture ordering, Zend semantics contract)
- **Files Changed**: 2 (both php-server crate)
- **Logic Changes**: 0 (amendments are documentation-only, no functional changes)

### Merge Status

- **Baseline**: Commit 7763a68 (already on main at start of session)
- **Amendments**: Committed as fed30ce on main
- **Remote**: Pushed to github.com/francescotinti/php-rust (e5c9cf5..fed30ce → main)
- **Status**: ✅ MERGED & PUSHED

---

## Lezioni (Load-Bearing Observations)

1. **Council Amendment Binding Power**: All 6 amendments were post-approval conditions, not optional polish. Implementation required for formal closure. Binding amendments are a **gate control mechanism**, not advisory feedback.

2. **Documentation as Enforcement**: Matsakis, Pedersen, and Stogov amendments were documentation-first (Send/Sync safety, output-capture ordering, Zend semantics). These document **invariants that must hold**, making documentation part of the contract spec, not post-hoc explanation.

3. **Measurement Scope Clarity**: Gregg amendment formalizes that G-APERTURA-2 gate measures **correctness only** (binary parity), deferred alloc/CPU/warmup measurement to WP-78+. This decoupling prevents **measurement conflation** (correctness gate should not fail on warmup variance).

4. **Binary Stability Verification**: Baseline binary 02aa2ad8 unchanged after amendments confirms **documentation-only edits** (no recompile needed). This is a check that amendments did not drift into unintended feature changes.

5. **9-Chair Council Permanent Rule**: Output capture BEFORE request_end() reset boundary is now a **permanent binding rule** (Pedersen/Stogov joint mandate). This rule will be load-bearing for all future per-request PHP execution; violation → rejection.

---

## Pre-Flight → Closure Checklist

- ✅ Disk space: ≥15G both volumes
- ✅ Processes: Cleaned (no stray php-server/cargo)
- ✅ Binary: Verified 02aa2ad8 (baseline unchanged)
- ✅ Tests: 1652/1652 PASS (1651 active, 1 ignored, 0 failed)
- ✅ G-APERTURA-2: Gate PASS (byte-parity verified)
- ✅ Amendments: All 6 implemented (fed30ce)
- ✅ Commit: Message includes council mandate + verification
- ✅ Push: Remote main branch updated (e5c9cf5..fed30ce)

---

## Next Session (WP-77.6.5.2.5 or WP-78)

**Pending Council Review** (9-chair reconvene):
1. Verify amendments implemented correctly (code review)
2. Approve WP-78 footprint census scope (post-Axum measurement phase)
3. Judge binding amendments: A-BB1, A-DL1 (deferred to WP-78)

**Measurement Phase (WP-78)**:
1. Tier-0 baseline: Measure WP-77.1 Axum runtime (no pool)
2. S-77.6.5.2.3 footprint census: RetainSet pool vs fresh-Vm
3. A-BB1 (Bak): Verify ≤+2% alloc vs CLI baseline
4. A-DL1 (Leijen): Fragmentation analysis (vmmap + mimalloc stats)
5. R-G4 (Gregg): CPU attribution (if commissioned)

**No Regressions Observed**: All WP-77.6.5.2.3 constraints (KS2, KM-77-1/2/3/4, KS-77-4/5/6, etc.) remain lifted. Architecture stable for measurement phase.

---

**Recorded**: 2026-07-31 02:05 UTC+2
**Signoff**: S-77.6.5.2.4 CLOSED — Ready for WP-78 council reconvene and measurement phase
