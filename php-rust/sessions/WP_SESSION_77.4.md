# WP-77.4: M1 Tier-0 Amendments Application (2026-07-31)

**Status**: ✅ COMPLETE — All AMEND-1/2/3 applied, compiled, committed, pushed

**Baseline**: phpr-wp77.3 (879fb2ed) + council 7-amendment binding

**Deliverable**: phpr-wp77.4-amend (7bf53854) with Tier-0 defects closed

---

## Summary

Executed mandatory Tier-0 amendment application per COUNCIL_WP77_REVIEWS.md binding verdicts. All three amendments (AMEND-1/2/3) successfully applied to `Vm::request_end()` in `crates/php-runtime/src/vm/mod.rs` (lines ~3320–3450).

**Amendments Applied**:
1. ✅ **AMEND-1** (Pedersen E-P1, Bak E-B1): Reset 7 resource ID counters to prevent wraparound collisions
2. ✅ **AMEND-2** (Pedersen E-P2): PRESERVE uncaught_throwable for M3 exception bridge
3. ✅ **AMEND-3** (Hejlsberg, Stogov, Matsakis, Pedersen): Reset 8 diagnostic fields + clear caches

**Compilation**: ✅ SUCCESS (phpr-wp77.4-amend 7bf53854, no errors, standard warnings only)

**Commits**: 1 (c0d056e "M1: Apply WP-77.3 council binding amendments AMEND-1/2/3")

**Push**: ✅ 8a58f4c..c0d056e main → main (GitHub remote)

---

## Technical Details

### AMEND-1: Resource ID Counter Resets (Bak E-B1, Pedersen E-P1)

**Problem**: Seven resource ID counters were not reset per-request, causing u32 wraparound collisions across requests.

**Fix**: Added resets in `request_end()`:
```rust
self.next_zip = 1;
self.next_pdo = 1;
self.next_gd = 1;
self.next_xslt = 1;
self.next_tidy = 1;
self.next_mysqli = 1;           // Note: field is next_mysqli, not next_mysql
self.next_dom = 1;
```

**Rationale**: Zend resource ID lifecycle requires counters to reset per-request, preventing stale resource handles from prior requests.

### AMEND-2: Preserve uncaught_throwable (Pedersen E-P2)

**Problem**: Original code reset `uncaught_throwable = None`, violating WP-76 binding R1.

**Fix**: REMOVED line `self.uncaught_throwable = None;` — exception state persists for M3 handler bridge.

**Rationale**: M3 exception bridge requires exception state available when handler reads/logs it. Resetting prematurely creates race condition.

### AMEND-3: Diagnostic Field Resets (8 fields)

**Problem**: Diagnostic and state fields leaked context across requests.

**Fix**: Added 8 field resets (web, fatal_line, error_level, magic_guard, iof_cache, census_on, user_abort_ignored, autoload_cursors).

**NOT reset**: `mb_regex` — persistent per Matsakis E-M1

---

## Compilation & Deployment

- **Build**: `cargo build --release` → SUCCESS
- **Binary hash**: 7bf53854 (phpr-wp77.4-amend)
- **Stashed**: `/Volumes/Extreme Pro/Claude/phpr-old-target/release/phpr-wp77.4`
- **Commit**: c0d056e (GitHub remote pushed 8a58f4c..c0d056e)

---

## Session Metrics

| Metric | Value |
|--------|-------|
| Amendments applied | 3/3 (100%) |
| Code lines changed | ~26 |
| Errors | 0 |
| Commits | 1 (c0d056e) |
| Gate tests passing | 1/3 (amendment verify; full gates pending M1 integration) |

---

## Kill-Switches (WP-77.4 close)

| Gate | Status |
|------|--------|
| **KS-M1-Complete** | 🟡 Pending M1 Axum integration (WP-77.4.1) |
| **KM-77-2-Concurrent** | 🟡 Pending concurrent fixture |
| **KS-M1-Gate-Upgraded** | 🟡 Pending byte-snapshot fixture |
| **KS-M4-Steady** | 🟡 Deferred to M4 phase |

---

## Next Session (WP-77.4.1)

**Focus**: M1 Axum integration + gate execution

**Baseline**: phpr-wp77.4-amend (7bf53854), commit c0d056e

**Deliverables**:
1. Vm lifecycle setup (Module + task_local!)
2. Handler integration (Option B wrapper)
3. KS-M1-Complete gate verification
4. KM-77-2 concurrent isolation test
5. KS-M1-Gate-Upgraded (byte-snapshot)

**Estimated Effort**: 2–3 hours
