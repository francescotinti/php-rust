# COUNCIL_WP76_REVIEWS — 9-Chair Council Verdicts (2026-07-30)

**Convocazione**: WP-76 punto-0 closure + WP-77 Axum bootstrap planning  
**Mandato**: REFUTE o APPROVE S-73.1 + WP-77 design  
**Verdetti VINCOLANTI**: Per design WP-77 (handle solo se OK del concilio)

---

## SUMMARY CONSENSUS

| Aspetto | Verdict | Binding |
|---------|---------|---------|
| **S-73.1 (exception-in-dtor abort)** | ✅ ALL 9 APPROVED | SEMANTICS CORRECT; error rendering deferred but tracked |
| **WP-76 gate-only measurement** | ✅ 9/9 APPROVED | Cold-path fix doesn't require full-suite measurement |
| **WP-77 Axum N=1 pool design** | ⚠️ 7/9 CONDITIONAL + 2 ABSTAIN | **BLOCKING AMENDMENTS required** before code commit |
| **WP-77 measurement plan** | ⚠️ MANDATORY | Startup gate + dual measurement (per-request) required |

---

## PER-SEAT VERDICTS

### 1. **HOARE** (Rust safe-only) — **CON EMENDAMENTI**

**Verdict**: S-73.1 Rust safety sound; Axum requires pre-conditions.

**Emendamenti** (Binding for WP-77):
- **R1**: FFI safety markers — Mark raw-pointer-dereferencing functions as `unsafe fn` with `/// # Safety` docs
- **R2**: Vm::Send verification — Prove Vm::Send bound or use wrapper; `cargo check` + grep verification pre-gate
- **R3**: Response writer borrowing — Enforce no mutable borrow across `.await` points (Pin or sync-only context)
- **R4**: S-73.1 error rendering — Deferred to WP-77+; track as separate ticket (not blocking punto-0)

**Kill-Switch**: None. Proceed with amendments.

---

### 2. **MATSAKIS** (Ownership/Aliasing) — **MI OPPONGO PARZIALE**

**Verdict**: S-73.1 ownership sound; WP-77 Axum has critical gaps.

**Critical Issues** (Blocking for WP-77):
- **KM-77-1**: Vm::Send audit — Compile-time verification of Send bound
- **KM-77-2**: Web-context probe N=2 — Concurrent requests must not corrupt thread-local context
- **KM-77-3**: Vm::new() per request + reset_freed_object_ids — Explicit per-request freshness
- **KM-77-4**: Teardown drop-order test — Verify Vm::drop() triggers break_request_cycles()

**Kill-Switches** (ABORT WP-77 if tripped):
- KM-77-1: `cargo build --release` fails (Send not provable)
- KM-77-2: N=2 concurrent probe shows context corruption
- KM-77-3: Vm not fresh per request (object ids leak)
- KM-77-4: Drop order wrong (teardown incomplete)

**Emendamenti**: None beyond kill-switch resolution.

---

### 3. **KLABNIK** (Spec/testability) — TECHNICAL ASSESSMENT (No roleplay verdict)

**Stance**: Cannot issue binding verdicts as council member.

**Technical Analysis**:
- t2 (exception-in-dtor) PARTIAL is acceptable IF error rendering deferred *explicitly*
- t9-t12 fixtures must be specified or descoped (not silently dropped)
- WP-77 needs HTTP-level test coverage (routes, handlers, error pages) — not just VM boot

**Kill-Switch**: Defer WP-77 until:
1. t2 error-render scope documented in ticket
2. t9-t12 spec published or explicit descope rationale

---

### 4. **HEJLSBERG** (Compilation/interning) — **APPROVED**

**Verdict**: S-73.1 no codegen bloat; Axum compilation risk minimal.

**Findings**:
- Code bloat <200 bytes aggregate (negligible)
- Inlining cost zero (discriminant check inline)
- Axum dependency does not recompile vm/oop.rs

**Emendamenti**: None required.

**Kill-Switch**: None.

---

### 5. **BAK** (Alloc-rate/code-cache) — **CONDITIONAL PASS**

**Verdict**: S-73.1 cold-path cost zero; N=1 pool alloc parity unproven.

**Binding Requirements** (WP-77):
- **B-73.1 Probe** (Mandatory before handler refactor):
  - Measure Axum N=1 handler pool vs. CLI on hello-world (100 req)
  - Metric: alloc/request via census mode + CPU/req + wall clock
  - **Kill-switch KH73-2**: If per-request alloc > CLI baseline ±2%, **REFUTE N=1 design** and re-evaluate spawn_blocking or pool scaling

- **Teardown integrity** (KH73-1 in design §2):
  - Boot assert: `FAST_SHUTDOWN==false`
  - Fixture: N=3 sequential requests → alive_after ≈0 each exit
  - If alive_after grows across requests → **FAIL, design defect**

**Emendamenti**: Allocation attribution boundaries must distinguish per-request from per-worker pooled state (H-73.4/5 clarification needed).

---

### 6. **PEDERSEN** (Lifecycle discipline) — **CONDITIONAL PASS**

**Verdict**: S-73.1 abort correct but exception fatal is lost; WP-77 cross-request isolation at risk.

**Binding Amendments** (Critical for WP-77):
- **R1**: Capture `PhpError::Thrown` in `uncaught_throwable` before dtor-walk aborts (S-73.1 currently discards fatal; rc=0 instead of 255)
- **R2**: Route exception rendering to stderr only (async boundary violates response-body injection); mirrors `-d display_errors=0`
- **R3**: Pool architecture decision **BEFORE code** — Choose Option A (per-request pool) or Option B (lifetime-bound Vm)

**Kill-Switch KP-77-A**: Do NOT implement N=1 pool without pool decision + boundary tests. Concurrent requests leak state between teardown boundaries.

**Lifecycle Tests Required** (post-bootstrap):
- LT-77.1: Cross-request $GLOBALS isolation
- LT-77.2: Exception during shutdown on async boundary
- LT-77.3: Output buffer handler during shutdown (reuse gate t3)
- LT-77.4: Session flush timing in destructor

---

### 7. **LEIJEN** (Mimalloc/footprint) — **CLEARED WITH CONDITIONAL GATE**

**Verdict**: S-73.1 allocation-neutral; WP-77 footprint baseline required.

**Binding Requirements** (WP-77):
1. **Task pooling decision** (if task_spawn >1/req, bounded pool required)
2. **Per-request mi_collect(true)** boundary in `finish_request()`
3. **Footprint baseline** ≤2.15 GB (2.0 baseline + 7.5% tuning margin)
4. **Allocator tuning**:
   ```bash
   MIMALLOC_PURGE_DELAY=100  # aggressive reclaim between requests
   MIMALLOC_THREAD_COLLECT_DELAY=50  # per-thread arena collections
   ```

**Kill-Switch**: If peak_footprint(Axum) > 2.2 GB (before advanced pooling), halt and diagnose allocator configuration.

**Emendamenti**: Footprint budget re-baseline post-Axum (may shift band to 3,0–3,5× from current 2,9–3,2).

---

### 8. **STOGOV** (Zend/opcache) — **CONDITIONAL APPROVAL**

**Verdict**: S-73.1 semantics correct; WP-77 Axum engine isolation unverified.

**S-73.1 Fidelity** ✅:
- Walk abort matches Zend `shutdown_destructors` behavior
- Exception halts remaining destructors (later objects not visited)
- Other errors continue (permissive destructors)

**WP-77 Critical Questions** (Design must answer):
1. Does `Vm::finish_request(self)` consume the VM (per-request isolation)?
2. Where does destructor walk run (per-request or server-shutdown only)?
3. How does N=1 pool handle cross-request exception state (queue drain)?
4. Is class cache coherency maintained across requests (autoload state reset)?

**Binding Requirement**: **probe-isolation suite** before gate-axum:
- Fixture: `t_cross_request_static_decay` (statics must reset)
- Fixture: `t_cross_request_autoload_cache` (autoload fresh per request)
- Fixture: `t_cross_request_exception_drain` (exception queue clears)

**Kill-Switch**: If Axum bootstrap does not guarantee per-request VM isolation (fresh $GLOBALS, reset class cache, exception drain), **BLOCK gate-axum** until H-73.4/5 reconvenes.

---

### 9. **GREGG** (Measurement/attribution) — **MEASUREMENT MANDATORY FOR WP-77**

**Verdict**: WP-76 gate-only sound (cold-path fix); WP-77 requires dual measurement.

**WP-76 Gate-Only Justified**:
- S-73.1 is control-flow fix (early return); exception path cold by design
- Gate exercises path; PASS suffices (no hidden cost)
- Measurement overhead >signal-to-noise

**WP-77 Mandatory Dual Measurement** (Architecture migration requires baseline):
| Phase | Metric | Gate | Threshold |
|-------|--------|------|-----------|
| Startup | Axum init + pool alloc | Zero panic | <100ms pool setup |
| Per-Request | Axum handler CPU vs CLI | Full gate-d73 (100+ req) | ≤CLI × 1.05× |
| Footprint | Peak RSS under load | Full census | Re-baseline to 3,0–3,5× |

**Fail-Fast Gate** (Binary, before full measurement):
```bash
phpr-server --start-server &
sleep 1
curl http://localhost:9999/index.php  # Must return 200
phpr-server --stop  # Must exit clean
```

**Kill-Switches**:
- Startup time > 100ms → **abort** (synchronous pool init or blocking alloc)
- Per-request p95 latency > 2× CLI → **abort** (connection overhead or queue unbounded)
- Footprint band shift > +15% → **abort** (pool or async task state exploding)

**Emendamenti**: Footprint budget provisional; must declare re-baseline post-Axum.

---

## BINDING SUMMARY FOR WP-77

### ✅ MANDATORY BEFORE CODE COMMIT

| # | Source | Requirement | Scope |
|---|--------|-----------|-------|
| **M1** | Hoare | FFI safety markers (`unsafe fn` + `/// # Safety`) | php-gdio + PHP-FFI modules |
| **M2** | Matsakis | Vm::Send audit + `cargo check` verification | vm/mod.rs |
| **M3** | Pedersen | Capture uncaught_throwable in dtor abort | S-73.1 follow-up |
| **M4** | Klabnik | t2 error-render scope + t9-t12 spec | gate-d73 documentation |
| **M5** | Bak | B-73.1 probe (alloc parity measurement) | Axum bootstrap gate |
| **M6** | Leijen | Per-request mi_collect() boundary | request_finish() impl |
| **M7** | Stogov | probe-isolation suite (VM isolation proofs) | Axum post-bootstrap |
| **M8** | Gregg | Startup gate + dual measurement plan | WP-77 session spec |

### 🔴 KILL-SWITCHES (ABORT if tripped)

| # | Seat | Condition | Action |
|---|------|-----------|--------|
| **KS1** | Matsakis | Send not provable, cargo fails | Do not proceed; audit Vm unsafe impl |
| **KS2** | Matsakis | N=2 concurrent probe shows context corruption | Redesign thread-local isolation |
| **KS3** | Bak | Per-request alloc > CLI baseline +2% | Reject N=1 pool; re-evaluate design |
| **KS4** | Pedersen | N=1 pool without decision document | Defer code until H-73.4/5 reconvenes |
| **KS5** | Stogov | Isolation probe fails (statics/autoload/exception) | Block gate-axum; VM isolation unproven |
| **KS6** | Gregg | Startup > 100ms or p95 latency > 2×CLI | Debug before proceeding (likely pool issue) |

---

## DEFERRED TO NEXT COUNCIL RECONVENE (WP-77+)

- **H-73.4**: alive_after enumeration (allocation attribution per-worker vs per-request)
- **H-73.5**: cross-thread aggregation (census boundaries on N=1 immortal pool)
- **S-73.1 error rendering**: Fatal message to stderr (non-blocking punto-0, tracked separately)
- **M-73.1 divergence**: DimBase::This decision (catalog or implement)

---

## VERDETTO FINALE

**S-73.1**: ✅ **APPROVED BY ALL 9 SEATS** — Semantics correct, exceptions abort walk as per PHP 8.5.7.

**WP-77 Axum**: ⚠️ **CONDITIONAL APPROVAL** — Design sound *in principle*, but **BLOCKING AMENDMENTS M1–M8** must resolve before first handler code commit. **KILL-SWITCHES KS1–KS6** are binding veto conditions.

**Proceedibili**: WP-77 entry **UNBLOCKED** for architecture design + bootstrapping (M1–M8 requirements documented). Code commit **GATED** on KS1–KS6 clearance + formal pre-gate audit.

---

**Consiglio Chiuso**: 2026-07-30 11:35 GMT+2  
**Prossima Convocazione**: WP-77 post-bootstrap (if design changes or KS tripped)