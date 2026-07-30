# WP-77.5 SESSION PROGRESS

**Session**: WP-77.5 (2026-07-30 19:00–ongoing)  
**Architecture**: Worker-actor php-fpm-style pool (Council binding mandate)  
**Baseline**: phpr 7bf53854, Commit 2be6813

---

## Pre-flight (✓ ALL GREEN)

- ✓ Disk /Volumes/Extreme Pro: 2929G free (threshold ≥15G)
- ✓ Disk /System/Volumes/Data: 25G free
- ✓ MySQL databases: wp, wp_o, wp_p, wptests online
- ✓ Orphaned processes: none
- ✓ WordPress uploads: normal state (2010/, 2026/)
- ✓ phpr binary: 16.9M at ~/Claude/php-rust-output/release/phpr
- ✓ phpr baseline: 7bf53854 matches council mandate

---

## Gate Status

| Gate | Status | Notes |
|------|--------|-------|
| **G1** | 🟢 SPIKE COMPILES | WorkerPool structure, no unsafe, clean build |
| **G2** | ⏳ Pending | Two-requests-same-Vm byte-parity |
| **G3** | ⏳ Pending | Tripla + K=10 amplification |
| **G4** | ⏳ Pending | Oracle pair fresh vs reuse |
| **G5** | ⏳ Pending | Request lifecycle order (G5 fixture) |

---

## G1: Vm Storage Spike — IMPLEMENTED ✓

**Goal**: Can we store Vm in worker pool without unsafe?

**Status**: ✓ COMPILES CLEANLY

**Changes**:
1. Removed M9/task_local (tokio::task_local! VM_EPOCH) — COUNCIL VETO
2. Created `WorkerPoolContext` struct (server-lifetime shared state)
3. Created `WorkerPool` struct (collection of workers)
4. Implemented `test_vm_storage()` — verifies compilation
5. Binary produced: 17.8M php-server with axum-server feature

**Commit**: 2be6813

**Next**: Expand to actual Vm creation (requires RetainSet + Module setup)

---

## Architecture Notes

### Design Pattern (php-fpm-style)

```
Server startup:
    ├─ Create WorkerPoolContext (server-lifetime)
    │   ├─ RetainSet (manages module parking)
    │   └─ Shared registry
    │
    └─ Spawn N worker threads
        └─ Each worker owns: Vm<'server>

Request (Axum handler):
    ├─ Send request to available worker via mpsc channel
    ├─ Worker executes FnOnce handler on Vm
    └─ Receive response, return Vm to pool

Worker thread loop:
    ├─ Receive (request, response_tx) from channel
    ├─ Call request_start()
    ├─ Execute handler FnOnce
    ├─ Call request_shutdown() then request_end()
    └─ Send response back, loop
```

### Key Constraints (Council Binding)

1. ❌ NO M9/task_local (REJECTED by Matsakis, Pedersen, Leijen, Stogov)
2. ✓ YES php-fpm-style worker threads (each owns Vm)
3. ✓ YES Vm reuse across requests (reset via request_end)
4. ✓ YES request_shutdown() BEFORE request_end (Stogov order)
5. ✓ YES Cache inventory + clear cost accounting (Leijen caveat)

---

## Known Issues / Blockers

### Vm Construction Complexity

Vm struct has 80+ fields including:
- Borrowed: `retain: &'m RetainSet`, `module: &'m Module`, `registry: &'m Registry`
- Owned: `Vec<Frame<'m>>`, `HashMap`, `BTreeMap<u32, Rc<RefCell<Object>>>`
- State: `created`, `statics`, `closure_statics`, `preg_cache` (all Rc-based)

**Issue**: Rc fields mean Vm is `!Send` (not sendable between threads).

**Solution**: Each worker thread owns its Vm (Vm never crosses thread boundary).
Rc usage is safe because Rc only contains intra-request state.

### G1 Next Step: Full Vm Construction

Need to:
1. Create `WorkerPoolContext::with_vm_lifecycle()` method
2. Spawn N worker threads with owned Vm instances
3. Test with Valgrind (zero leaks)
4. Run gate_two_reqs_same_vm.php (G2)

---

## Files Changed

- `/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust/crates/php-server/src/main.rs`
  - Removed: `tokio::task_local! { VM_EPOCH }`
  - Added: `WorkerPoolContext`, `WorkerPool`, `test_vm_storage()`
  - Replaced M9 handler with minimal spike test

---

## Next Session Goals

1. Implement full Vm creation in worker threads
2. Add channel dispatch for requests (mpsc + oneshot)
3. Run G2 gate: two-requests-same-Vm byte-parity
4. Create gate fixtures (gate_two_reqs_same_vm.php, etc.)

---

## Council Kill-Switches (Active)

| Switch | Condition | Status |
|--------|-----------|--------|
| KS-M-77.5-A | Spike requires unsafe/leak | ARMED (not triggered) |
| KS-P77-A | req-2 ≠ fresh | ARMED (gate G2 will test) |
| KS-W77-A | Reactor blocked | ARMED |
| KL-77.5-1 | ΔVm >25% | ARMED |
| KS-B77-1 | M4 without oracle | ARMED |
| KS-S77-A | Incomplete lifecycle | ARMED |

---

## Session Metrics

- **Compile time**: 3.3s
- **Binary size**: 17.8M (php-server)
- **Code changes**: 1 file, +40/-28 lines
- **Unsafe used**: 0 (target met for G1 spike)
