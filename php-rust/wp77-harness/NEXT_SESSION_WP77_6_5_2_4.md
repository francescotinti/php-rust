# NEXT_SESSION — WP-77.6.5.2.4: Formal Council Concilio Review of S-77.6.5.2.3

**Baseline**: commit afa8911 (S-77.6.5.2.3 complete)  
**Scope**: Formal 9-seat council review of Axum HTTP handler wiring architecture  
**Mandate**: Council must refute or validate design decisions; identify constraints; approve S-77.6.5.2.4 scope

---

## Stato Gate (Release)

| Gate | Status | Evidence |
|---|---|---|
| **G-APERTURA-2 CLI** | ✅ PASS | Two sequential requests via `--cli-server` produce byte-identical output |
| **G-APERTURA-2 Axum** | ✅ PASS | Two sequential requests via `--axum` produce byte-identical output |
| **KS-P77-A,B,C** | ✅ ACTIVE | All persistent-Vm kill-switches passing |
| **Regression** | ✅ OK | 2748/2748 tests PASS (no breakage) |
| **Binary** | ✅ STABLE | 02aa2ad8 (matches S-77.6.5.2.2 baseline) |

**Harness**: `/Volumes/Extreme Pro/Claude/wp77-harness/` (all runs, gates, reviews here)

---

## Direttive Permanenti (Never Repropse)

1. ⭐⭐ **Per-request output lifecycle**: `request_end()` is the reset boundary for per-request parity. Capture all output streams (`stdout`, `rendered`) **before** calling `request_end()`. (Proven: S-77.6.5.2.3)

2. ⭐⭐ **Kill-switch wiring**: Gate against the **mechanism that calls** the code under test, not an identical twin gate. (Rule from Council WP-77.5)

3. ⭐⭐ **Worker pool thread safety**: Vm is !Send+!Static (Rust borrow checker enforces per-thread lifetime). Persistent RetainSet is the correct unit of cross-request persistence, not Vm. (Architecture proven S-77.6.5.2.2)

4. ⭐⭐ **API exposure for wiring**: When writing new execution contexts (HTTP handlers, WSGI, etc.), expose ONLY the Vm fields/methods needed for that context. Don't expose internals (diags, error_level, etc.) unless they're part of the handler lifecycle.

---

## Council Concilio Manifest (9 Sedie)

| Sedia | Reviewer | Perimetro | Prompt |
|---|---|---|---|
| **Hoare** | design/safety | Rust safety, ownership constraints | "Does the axum_handler correctly enforce Vm !Send + !Static lifetime? Is the oneshot channel bridge safe across threads?" |
| **Matsakis** | ownership/borrow | Lifetime validation | "Is the &RetainSet → &Vm borrow dance safe? Can the Vm outlive RetainSet?" |
| **Klabnik** | spec/testing | Testability, gate coverage | "Are G-APERTURA-2 gates sufficient to validate per-request parity? What edge cases might slip?" |
| **Hejlsberg** | compiler/dedup | Incremental compilation, interning | "Does the worker_pool spawning break incremental builds? Is there duplicate allocation on first request vs. persistent?" |
| **Bak** | perf/alloc-rate | Allocation, cache locality | "What's the allocation overhead for creating oneshot channels per request? Does dispatch() scale linearly?" |
| **Pedersen** | boundary discipline | Per-request reset, lifecycle | "Is request_end() the ONLY reset boundary? Can output leak past request_shutdown()? Is the comment explaining output capture timing clear enough?" |
| **Leijen** | allocator/footprint | Physical memory, fragmentation | "Does the persistent RetainSet grow unbounded? Are arena allocations cleaned per request?" |
| **Stogov** | Zend semantics | Engine lifecycle, shutdown order | "Does the Stogov order (request_start → run → request_shutdown → request_end) match Zend exactly? Are shutdown functions guaranteed to run before output reset?" |
| **Gregg** | measurement/attribution | Methodology, benchmarking | "Are the 2748 test results reproducible? Did we miss any workload where axum mode diverges from cli mode?" |

Each reviewer reads: NEXT_SESSION, WP_SESSION_77_6_5_2_3.md, S-77.6.5.2.3 code diff, and their domain-specific code (Vm::run for Stogov, dispatch for Bak, etc).

**Output format**: VERDETTO (CONCORDO / CON EMENDAMENTI / MI OPPONGO) + numbered fixes (if any) + kill-switch name (if blocking).

**Deadline**: Verbal in wp77-harness/COUNCIL_WP77_6_5_2_3_REVIEWS.md; verdicts are **BINDING** for S-77.6.5.2.4 scope.

---

## Work Scope: S-77.6.5.2.4

**Pending the Council review, S-77.6.5.2.4 will:**

1. **Apply Council verdicts** to code/docs (if any emendamenti or oppongo blocks require design changes).
2. **Wire persistent-Vm into production HTTP server** (if not blocked):
   - Route all .php requests through worker pool (not just fallback)
   - Test with real WordPress request corpus (not just gate)
   - Verify G-APERTURA-2 still passes under load
3. **Measure performance baseline**:
   - Request latency (two-request cycle time)
   - Per-request allocation (Axum vs CLI)
   - Memory footprint stability (1000 sequential requests)
4. **Formalize Concilio verdicts** (create S_77_6_5_2_4_COUNCIL_REVIEW.md with Council verdicts + synthesis).

---

## Pre-flight (Today, before any work)

1. Disk: ≥15G free on both `/Volumes/Extreme Pro` and system volume
2. MySQL: `mysql --socket=/private/tmp/mysql-wp8.sock -uroot -e "SHOW DATABASES"` lists wp, wp_o, wp_p, wptests
3. Binary: `shasum -a 256 ~/Claude/php-rust-output/release/phpr | cut -c1-8` = 02aa2ad8 (or new stash if rebuild)
4. No orphan processes: `pgrep -fl "phpr|php-server"` returns nothing
5. Baseline tests: `cargo test --release 2>&1 | grep "test result:" | head -5` all show "ok"

---

## Direttive Sessione (WP-77.6.5.2.4)

- **No revert**: Council verdicts are binding; don't second-guess. Implement emendamenti exactly as stated.
- **No micro-optimization before gate**: If Council says "measure latency", measure on the passing gate first; don't optimize internals until after.
- **Kill-switch coverage**: If any new code path is added, create a gate that exercises it before merge.
- **Push at every step**: Commit after each Council fix + each new gate. No batching.

---

## VCS State

- **Repo**: `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/`
- **Last commit**: afa8911 (session file + cleanup)
- **Branch**: main (no feature branches)
- **Origin**: github.com/francescotinti/php-rust (dual repo; parent for handoff context)

---

## Ordine di Lettura

1. This file (orientation)
2. `WP_SESSION_77_6_5_2_3.md` (what was done, lessons)
3. `COUNCIL_WP77.5_REVIEWS.md` (prior mandate, still binding)
4. Code: `crates/php-server/src/main.rs` (axum_handler + router) + `crates/php-server/src/worker_pool.rs` (output lifecycle fix)
5. Council prompt (await reviewer input)

---

## Chiusura

Invoke `/chiudi-sessione` once all Council reviews are collected and synthesis is ready for S-77.6.5.2.4 approval.
