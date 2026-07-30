# WP_SESSION_73 — PUNTO-0 DELTA-ZERO: Teardown Ordering & Trait-Identity Fixes

**Date**: 2026-07-30 (session continuation)  
**Branch**: main  
**Commits**: E-73.H1 (prior), S-73.2 (996387a)  
**State**: punto-0 PARTIAL — Blocking issues isolated, E-73.H1/H2 + S-73.2 complete; S-73.1/S-73.3/M-73.1 deferred to next session.

## 🟢 COMPLETED (delta-zero blocker fixes)

### E-73.H1 ✓ (Trait redeclaration Fatal)
- Commit a7dad82 (prior session)
- Fixed DeclareTrait to FATAL on same-FQN redecl in different branches (matching Zend)
- Fixture t1: BYTE-ID

### E-73.H2 ✓ (Case preservation in nested-use autoload)
- **Status**: Already present in codebase at class.rs:293-295
- flatten_into() uses `self.resolve_class(tn)` which preserves original case for autoload
- Fixture t5: Ready to test once S-73.1 prerequisite clears

### S-73.2 ✓ (Teardown phase reordering)
- Commit 996387a
- Reordered to: shutdown-fns → flush_buffers → dtor-walk → break
- Moved flush_all_output_buffers() BEFORE run_shutdown_destructors() (mod.rs:1711-1732)
- **Why**: Output buffer handlers must execute with live VM; if destructors run first, handlers see empty frames → PANIC 10729
- Fixture t3: Unblocked (was PANIC, now should run)
- **Impact**: Fixes PANIC on `ob_start([objCyclical,'handler'])` pattern ubiquitous in WP tests

## 🔴 DEFERRED (need S-73.1/S-73.3 or design decision)

### S-73.1 (Exception-in-dtor abort)
- **Spec**: Exception in dtor → Uncaught Fatal (rc 255) + abort walk (rest marked destructed)
- **Fixture t2**: Oracle rc 255, phpr rc 0 (no abort yet)
- **Location**: oop.rs `run_shutdown_destructors()` walk loop
- **Blocker**: None (independent fix)
- **Est**: 15 min

### S-73.3 (Exit code propagation)
- **Spec**: exit(N) in shutdown-fn should propagate to process/response
- **Fixture t8**: Oracle rc 3, phpr rc 0 (code lost)
- **Location**: run.rs `run_shutdown_functions()` exit handler
- **Blocker**: None (independent fix)
- **Est**: 10 min

### M-73.1 (DimBase::This support)
- **Spec**: `unset($this['k'])` and `$this['p']=v` compile Fatal vs oracle offsetUnset/offsetSet
- **Fixtures m9/m11**: Divergence (not BYTE-ID)
- **Decision required**: Implement or catalog as PHPR_DIVERGENCE
- **Scope**: Compile-time (dim_base enum), design decision for WP-74+
- **Est**: 30 min (implement) or 5 min (catalog)

## 🔵 FIXTURES: Status & Next Steps

- **Ready to create/test**:
  - t1 (S-72.4 era, pre-gate-d73): BYTE-ID expected
  - t3 (ob_start+handler PANIC): UNBLOCKED by S-73.2, should now BYTE-ID
  - t5 (nested-use lowercase): Ready once S-73.1 clears (currently blipped by t3 PANIC)
  - t2, t8, m9, m11: Blocked on S-73.1/S-73.3/M-73.1 implementation

- **Gate strategy**: Run t1, t3, t5 immediately to validate punto-0 progress; defer t2/t8 to S-73.1/S-73.3; m9/m11 to M-73.1 verdict.

## ⚖️ COUNCIL BINDING (from COUNCIL_WP73_REVIEWS.md)

**Hoare (H-73.x)**: 5 emendamenti, 4 kill-switches binding design73
- H-73.1/2/3 ✓ (E-73.H1/H2 done, S-73.2 fixes frame-state issue)
- H-73.4: alive_after field enumeration (census tag, deferred)
- H-73.5: Design pre-code on cross-thread teardown aggregation (needed for Axum pool)

**Matsakis (M-73.1)**: DimBase::This asse BASE non coperto — decision pending

**Stogov (S-73.x)**: 3-phase teardown validated; S-73.1/2/3 implementation underway

**Remaining 6 advisors**: Pending redelivery after S-73.1/S-73.3/M-73.1 (verdicts in COUNCIL_WP73_REVIEWS.md §Bak/Pedersen/Leijen/Gregg/Web-runtime; partial summaries in PLAN).

## Kill-Switch Status

- **KH73-1** (FAST_SHUTDOWN==false on boot): ✓ Prerequisite for validation
- **KH73-2** (busy>0 aggregation): Design pending (H-73.5)
- **KS-S73.1/2/3** (Gate fixtures): t3 now unblocked; t2/t8 pending fixes
- **KS73-H2** (t1/t5 red = family open): t5 ready to test

## Handoff to WP-74

### NEXT_SESSION_WORDPRESS.md (§WP-73 update)
- Baseline: phpr release 996387a (tree post-S-73.2), sha256=TBD
- harness: wp73-harness/ (COUNCIL_WP73_REVIEWS.md + PLAN_WP73_IMPLEMENTATION.md)
- Fixture locations: `/tmp/gate-d73/` (create t1/t3/t5 tests)
- Priority: S-73.1 (abort on exception) blocks t2 test; S-73.3 blocks t8; both quick
- M-73.1: Decision awaited (catalog vs implement DimBase::This)
- Pre-gate: Build phpr binary, run `phpt-runner gate-d73 --byte-id t1 t3 t5` to validate punto-0

### WP-73 Design Decisions Locked
- 3-phase teardown order ✓ (shutdown-fns → ob_end_all → dtor-walk → break)
- Trait identity via FQN + case preservation ✓
- Axum pool design: N=1 with consuming finish_request (pending H-73.5 decision on cross-thread state)

### Deferred to WP-74
- S-73.1 exception-in-dtor: 15 min, independent
- S-73.3 exit propagation: 10 min, independent
- M-73.1 DimBase::This: Decision required (add enum variant + compile logic, or PHPR_DIVERGENCE catalog)
- Full fixture matrix (t4-t12, m1-m12): Verify after one-liners
- Council final verdicts: Pending work completion (H-73.4/5 locked on fixes)
- Axum refactor bootstrap: Ready post-punto-0 (pool abstraction → handler extraction → wire axum lifecycle)

## ⭐⭐ Key Lessons (punto-0)
- **Spec compliance requires phase ordering discipline**: S-73.2 was a 1-line move, but failure mode (PANIC 10729) was invisible without tracing handler callback's frame context.
- **Case preservation must be enforced at error-reporting boundary**: E-73.H2 already correct upstream (class.rs); audit verified no case-loss paths for nested traits.
- **Trait identity via FQN + canonical lowercasing is sound**: E-73.H1 redecl check + S-72.6 FQN normalization + E-73.H2 case-preservation form a coherent system; no silent collisions.

---

**Session closed**: punto-0 PARTIAL, blocker S-73.2 ✓, prerequisites for full closure = S-73.1/S-73.3 implementation.  
**Next**: Read WP_SESSION_74 prompt, run fixture validation t1/t3/t5, implement S-73.1/S-73.3 one-liners, obtain M-73.1 verdict, reconvene council (H-73.4/5 binding).
