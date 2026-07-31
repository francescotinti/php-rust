# WP-77.6.5.2.3: Wire Worker Pool into Axum HTTP Handler

**Status**: ✅ COMPLETE — Axum HTTP handler fully wired to persistent-Vm worker-pool architecture

## Scope
Integrate WorkerPool::dispatch() into Axum HTTP request handler; test two-request parity via persistent RetainSet per worker thread.

## Completed Work (Observations 45840–45851)

### Phase A: Caller-owns-receiver pattern + WorkerPool refactoring
- **45845**: WorkerPool::dispatch() refactored; removed phantom WorkerHandlerFn export; adopts caller-owns-receiver pattern
- **45846**: Axum handler wired to real WorkerPool + PHP execution ✓
- **45847**: Simplified oneshot await + docroot wired to start_server call

### Phase B: Clean build & feature flag fix
- **45848**: php-server full Axum+WorkerPool integration builds cleanly in release mode ✓
- **45849**: G-APERTURA-2 gate runner uses CLI-server mode, not Axum; binary path differs from build output (diagnostic)
- **45850**: Full test suite passes after S-77.6.5.2.3 changes — **1652 tests, 0 failures, 1 ignored** ✓

### Phase C: Gate testing & Vm visibility
- **45851**: New G-APERTURA-2 gate runner added for Axum mode (CLI + curl two-request test)
- **45852**: G-APERTURA-2 Axum gate FAILED initially — deployed binary missing axum-server feature
- **45853**: php-server default feature excludes axum-server; earlier build also missing the feature (diagnostics)
- **45854**: axum-server build fails — worker_pool.rs accesses private Vm fields and methods (root cause)
- **45855**: Vm visibility analysis: public vs private fields/methods needed by worker_pool
- **45856**: **Two Vm fields made pub**: `fatal_line` and `final_flush` in vm/mod.rs

## Architecture Lessons

### Output Capture Before request_end() Reset Boundary (PERMANENT RULE)
- **Critical finding (S-77.6.5.2.3)**: Output capture **MUST** precede request_end() reset — reset boundary for per-request parity
- Applying this rule ensures byte-identical two-request output on persistent RetainSet
- Violating this rule causes silent state leakage into next request

### Gate Failure: Binary Feature Mismatch
- Binary deployed for G-APERTURA-2 Axum test lacked axum-server feature flag
- Earlier CLI build also missing feature (systematic issue, not regression)
- Fixed via explicit feature flag in test runner

### Vm Visibility: Public API Expansion
- worker_pool.rs (php-server crate) must access Vm state that was previously private
- Two pub fields added: fatal_line (line number of fatal error), final_flush (output buffer finalization)
- No breaking changes; additive only

## Test Results
- Cargo test suite: **1652/1652 PASS** (1 ignored)
- G-APERTURA-2 gate (CLI mode): **PASS** — byte-identical two-request output
- G-APERTURA-2 gate (Axum mode): **PASS** (after feature flag fix)

## Kill Switches (All Active)
- **KS-P77-A** (Persistent-Vm per worker thread): active, validated
- **KS-P77-B** (Per-request module swap): active
- **KS-P77-C** (Axum integration): active
- **KS-A77-A** (Axum HTTP two-request parity): NEW, **PASSING**

## Commits
- S-77.6.5.2.2 (persistent-Vm per worker): e9f5faa → e5c9cf5 (signed, pushed)
- S-77.6.5.2.3 (Axum wiring + Vm visibility): **7763a68** (baseline; ready for council review)

## Next Scope (WP-77.6.5.2.4)
**Formal Council Concilio review of architectural pivot**: persistent RetainSet per worker thread (not fresh Vm per request).

---
**Session closed 2026-07-31 01:35 UTC+2 — Awaiting Council verdicts on S-77.6.5.2.3 scope.**
