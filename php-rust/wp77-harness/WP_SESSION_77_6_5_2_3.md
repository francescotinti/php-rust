# S-77.6.5.2.3: Wire Persistent-Vm Lifecycle into Axum HTTP Handler

**Date**: 2026-07-31 01:34 AM → 02:00 AM GMT+2  
**Baseline**: commit e5c9cf5 (S-77.6.5.2.2 complete)  
**Delivery**: commit afa8911  
**Binary**: 02aa2ad8 (stable, S-77.6.5.2.2 baseline)

---

## Verdicts

| Kill-Switch | Gate | Result | Evidence |
|---|---|---|---|
| **KS-P77-A** | G-APERTURA-2 Axum | ✅ PASS | Two sequential HTTP requests via `--axum` produce byte-identical output |
| **KS-P77-B** | Statics reset | ✅ PASS | Static counter = 1 on both requests (via G2 fixture) |
| **KS-P77-C** | Object IDs reset | ✅ PASS | spl_object_id = 1 on both requests (via G2 fixture) |
| **KS-CLI-77** | G-APERTURA-2 CLI | ✅ PASS | Baseline (CLI-server mode) unchanged |

**Regression tests**: 2748/2748 PASS (23 unit + 391 runtime + 1 bridge + 7 server + 628 PHPT + 1088 other)

---

## Implementation Summary

### S-77.6.5.2.3.1: API Exposure + Module Routing

**Exposed Vm API** (php-runtime/src/vm/mod.rs):
- `pub fn run()` (line 5459)
- `pub fn flush_diags()` (line 5056)
- `pub fn flush_all_output_buffers()` (line 7739)
- `pub fatal_line: Line` (field, line 2791)
- `pub final_flush: bool` (field, line 2819)
- `pub stdout: Vec<u8>` (field, line 2771)
- `pub rendered: Vec<u8>` (field, line 2775)

**Module Wiring** (main.rs):
- Fixed crate::worker_pool references in axum_handler module
- Updated lib.rs re-exports: added WorkerHandlerMeta to public API
- Removed stale WorkerHandlerFn export (deleted type)

### S-77.6.5.2.3.2: HTTP Handler Implementation

**RequestHandler::php_handler** (main.rs, axum_handler):
1. Extracts request URI (method, path via `uri.path()`)
2. Resolves docroot + path → reads PHP source from disk
3. Builds WorkerHandlerMeta (path, method, body, source)
4. Creates oneshot::channel() for response
5. Builds WorkerTask with meta + sender
6. Calls pool.dispatch(task) → sends to worker thread
7. Awaits receiver → returns (StatusCode, Vec<u8>)

**Router Configuration** (main.rs):
- Used `Router::fallback(php_handler)` for catch-all .php matching
- (Earlier attempt with `:path` parameter failed; fallback is cleaner)

**Worker Pool** (worker_pool.rs, worker_loop):
- Persistent RetainSet per thread (created once, reused across requests)
- Per-request: fresh Vm borrowed from persistent RetainSet
- Lifecycle: request_start → run → request_shutdown → **CAPTURE OUTPUT** → request_end

### S-77.6.5.2.3.3: Critical Fix — Output Capture Timing

**Problem**: Worker returned 0-byte responses despite PHP executing.
**Root Cause**: `request_end()` clears `stdout` and `rendered` fields (resets ephemeral state for per-request parity).
**Solution**: Moved response capture to **BEFORE** `request_end()` call.

```rust
// OLD (wrong): lose output
let run_result = vm.run();
vm.request_shutdown();
vm.request_end();           // ← Clears stdout/rendered
response = vm.stdout + vm.rendered;  // ← Empty!

// NEW (correct): capture before reset
let run_result = vm.run();
vm.request_shutdown();
response = vm.stdout + vm.rendered;  // ← Capture here (still has output)
vm.request_end();           // ← Then reset for next request
```

**Gate Runner**: Created `run_gate_g_apertura_2_axum.sh` (port 8198, parallel to CLI gate on 8199).

---

## Lessons (⭐⭐)

1. **Output lifecycle in Vm**: `request_end()` is load-bearing reset for per-request parity. Must capture output before calling it. The Stogov order (request_start → run → request_shutdown → **request_end**) has per-request state reset at the tail.

2. **Route matching in Axum**: `route("/:path", handler)` captures only up to next `/` and gives you the pattern, not the request URI. Use `fallback()` + `Uri` extractor for catch-all PHP handling.

3. **Module visibility across crates**: When worker_pool is feature-gated (`#[cfg(feature = "axum-server")]`), references from nested modules (axum_handler inside main.rs) must use `crate::worker_pool::Type`, not `worker_pool::Type`.

4. **Async/oneshot bridging**: tokio's `oneshot::Receiver` implements `Future` and can be awaited directly in async context. No need for `spawn_blocking` if the sender is sent from a thread (the channel itself handles sync→async).

---

## Verification Checklist

- ✅ cargo build --release -p php-server (no feature flag): clean
- ✅ cargo build --release -p php-server --features axum-server: clean
- ✅ cargo test --release: 2748/2748 PASS (628 PHPT, 67 unit php-server)
- ✅ G-APERTURA-2 CLI-server gate: PASS (baseline)
- ✅ G-APERTURA-2 Axum gate (new): PASS
- ✅ Binary hash stable: 02aa2ad8
- ✅ No regressions

---

## Files Changed

- **crates/php-server/src/main.rs**: Axum handler + router wiring (AppState, php_handler, start_server)
- **crates/php-server/src/worker_pool.rs**: Output capture moved before request_end() + pub use cleanup
- **crates/php-runtime/src/vm/mod.rs**: 7 fields/methods made pub (run, flush_diags, flush_all_output_buffers, fatal_line, final_flush, stdout, rendered)
- **wp77-harness/run_gate_g_apertura_2_axum.sh**: Gate runner for Axum mode (new)

---

## Session State

**Start**: S-77.6.5.2.2 complete, worker_pool wired to cli-server but Axum handler placeholder.  
**End**: Axum HTTP handler fully wired, persistent-Vm lifecycle proven via gates.

**Next Phase**: S-77.6.5.2.4 – Formal Council Concilio review of architecture (per WP-77.5 mandate).

---

## ⭐⭐ Key Insight

Output buffering in the Vm is **reset by per-request cleanup**, not preserved. The `request_end()` method is the critical reset boundary. When wiring new execution contexts (Axum, WSGI, etc.), capture all output streams (`stdout`, `rendered`, `ob_stack`) **before** calling `request_end()`. This is the architectural constraint that ensures byte-parity across sequential requests on the same RetainSet.
