# WP-77.4.2 Results — Binding Steps 4–5 (KM-77-2 + KS-M1-Gate-Upgraded)

**Date**: 2026-07-30 · **Binary**: phpr 7bf53854 (parity re-verified after
php-server axum build — phpr untouched) · **Baseline commit**: 45d3bc5

---

## Verdicts (both from COMMITTED runners, verdict files in
`/Volumes/Extreme Pro/Claude/wp77-harness/gate-out/`)

| Kill-switch | Verdict | Evidence |
|---|---|---|
| **KM-77-2** (concurrent isolation) | ✅ **PASS** | 15 rounds × 2 parallel curls vs php-server cli-mode: **30/30 PASS, 30 unique markers, 0 leaks**; positive control (stale `$GLOBALS` injected) **FAILs correctly** → the fixture bites |
| **KS-M1-Gate-Upgraded** (byte-snapshot ×3 workloads) | ✅ **PASS** | A: axum hello-world 7/7 byte-id (42 B) · B: WordPress front 7/7 (57 428 B) + post permalink 7/7 (71 093 B) + feed 7/7 (1 675 B) · C: ORM 3484 test **3E/13F**, fail-set **16 nomi IDENTICO** |

## Fixture honesty notes (bite-worthiness)

1. **`gate_km77_2_concurrent.php` (WP-77.4.1) false-PASSes on leaks**: it sets
   the marker to `'initial'` and checks for `'initial'` — a leaked marker would
   still read `'initial'`. Superseded by **`gate_km77_2_http.php`**: stamps each
   request with a UNIQUE value (`req-<pid>-<rand>`); any residue is a real FAIL.
   Proven by positive control (rc: FAIL when stale state injected).
2. **`/?p=1` is a 301 with empty body** — 7 identical hashes on 0-byte bodies
   was a false green (the all-zero-counter lesson). The runner now resolves the
   real permalink (`/2026/07/hello-world/`) and guards `size>0` on every snap.
3. **`wait` nudo nel runner aspetta anche il server backgrounded** (hang al
   primo run, morso e fixato: wait sui soli PID curl).

## ORM baseline provenance

Counts 3E/13F match the WP-72 gate state (NEXT_SESSION §Stato gate); the 16
names match the catalogued families (php-rust-orm-gate-recipe 2026-07-15):
XSD ×8 · rollback EM/UoW ×2 · HydrationCache · LifecycleCallback · NewOperator ·
PropertyHooks · AbstractHydrator toIterable · DebugTest. Name-level pin file
created this session: `gate-baseline/orm7742-fails.txt` (16 righe) — future
sessions diff BY NAME against it.

## M4 metrics (clean, measured AFTER the ORM run finished — first pass was
contaminated by concurrent phpunit CPU and discarded)

| Endpoint | Median latency (n=10) | Physical footprint (vmmap) |
|---|---|---|
| Axum hello-world (placeholder, no PHP) | 0.3 ms | 4.3 MB |
| cli-server, km77 fixture | 6.8 ms | 18.4 MB |
| WordPress front (phpr -S, DB wp) | 352 ms | 350–367 MB, **stable across ~46 req (no growth)** |

Abort-time metrics: N/A this session — M3 exception bridge not yet built;
nothing aborted. To collect in M3/M4.

## Scope caveat (honest)

The Axum handler is still a **placeholder** (no PHP execution; `with_vm_lifecycle`
returns a literal). Therefore:
- Workload A validates the Axum plumbing + response byte-stability only.
- The REAL per-request `$GLOBALS` isolation (KM-77-2) and WordPress/ORM parity
  run on the **cli-server path**, which is where PHP executes per-request today.
- KM-77-2 "simultaneous async tasks sharing one process" in its full sense can
  only be tested once the Axum handler executes PHP (WP-77.5+: Vm in handler).

## Files

- `fixtures/gate_km77_2_http.php` — biting concurrent fixture (committed)
- `run_gate_km77_2.sh` — KM-77-2 runner (committed)
- `run_gate_ks_m1_upgraded.sh` — 3-workload snapshot runner (committed)
- `gate-baseline/orm7742-fails.txt` — ORM name pin (committed)
- Verdicts + snaps + ORM log: `/Volumes/Extreme Pro/Claude/wp77-harness/gate-out/`
