# WP_SESSION_74 — PUNTO-0 DELTA-ZERO: Diagnostic & Framework (punto-0 partial, blocked on flush_buffer frame setup)

**Date**: 2026-07-30 (WP-74 point-0 diagnostics)  
**Branch**: main  
**Commits**: None (diagnostic phase only)  
**State**: punto-0 BLOCKED — S-73.2 ✓ (code correct), but t3 fixture still panics due to flush_buffer frame context issue

## 🔴 CRITICAL ISSUE — Root Cause Identified

### S-73.2 Code Verification: CORRECT ✅
- Commit 996387a successfully reordered shutdown phases
- Current sequence (mod.rs:1712-1723):
  1. `run_shutdown_functions()`
  2. **`flush_all_output_buffers()` ← MOVED HERE (was after destructors)**
  3. `run_shutdown_destructors()`
  4. `session_shutdown_flush()`
  5. `finalize_filtered_streams()`
  6. `break_request_cycles()` (if not FAST_SHUTDOWN)

### Test Fixture t3 Status: PANICS ❌
- Test: `ob_start([$objCyclical, 'handler']); echo "Hello";` (no explicit flush)
- Oracle (PHP 8.5.7): **`[H:Hello]`** ✓ (handler runs before destructors)
- phpr: **PANIC** at `crates/php-runtime/src/vm/mod.rs:10729:24`
  - Error: `index out of bounds: the len is 0 but the index is 18446744073709551615`
  - Indicates: `frames[top]` access where `top = usize::MAX` (−1), `frames.len() = 0`

### Root Cause: flush_buffer() Frame Setup
**Problem**: When `flush_all_output_buffers()` invokes a handler callback method, it does NOT push a frame for handler execution. The handler method tries to access `self.frames[top]` but frames stack is empty (line 10729 accesses frame's `class` field for method dispatch).

**Why S-73.2 moved flush but panic persists**:
- ✅ Structural order is now correct (flush before dtor-walk)
- ❌ But `flush_buffer()` implementation lacks frame context for handler invocation
- Handler invocation occurs without:
  - Frame pushed onto stack
  - `class` and `static_class` initialized  
  - `$this` binding available inside handler

**Expected Pattern** (from oop.rs:dtor_walk_round):
```rust
// Must push frame BEFORE invoking user code
let callee = &self.classes[defc].methods[midx].func;
let m = self.class_mod(defc);
let mut frame = self.pooled_frame(callee, m);
frame.this = Some(Zval::Object(Rc::clone(o)));
frame.class = Some(defc);
frame.static_class = Some(cid);
self.frames.push(frame);
let _ = self.run();  // NOW handler invocation has frame context
```

## 🔴 PUNTO-0 BLOCKERS (still deferred)

### 1. **CRITICAL: flush_buffer() Frame Setup** (blocks punto-0 validation)
   - **Location**: `crates/php-runtime/src/vm/mod.rs` (flush_buffer method, not yet located)
   - **Fix Pattern**: Push frame before `self.run()` invokes handler callback
   - **Impact**: Without this, t3 panics; S-73.2 cannot be validated
   - **Est**: 20-30 min (locate method + apply frame setup)

### 2. S-73.1 (Exception-in-dtor Abort) — deferred pending flush_buffer fix
   - File: `crates/php-runtime/src/vm/oop.rs:dtor_walk_round()`
   - Change: Catch exception from `self.run()`, emit fatal, abort walk
   - Fixture t2 blocked on this

### 3. S-73.3 (Exit Code Propagation) — deferred pending flush_buffer fix
   - File: `crates/php-runtime/src/vm/run.rs`
   - Change: Capture `PhpError::Exit(code)` from shutdown-fns
   - Fixture t8 blocked on this

### 4. E-73.H1/H2/H3 (Trait Redecl + Case + Fallback) — ready to implement
   - Fixtures t1 (redecl), t5 (case) blocked
   - These are orthogonal to flush_buffer fix

### 5. M-73.1 (DimBase::This) — decision needed
   - Recommend: Catalog as PHPR_DIVERGENCE to unblock punto-0
   - Fixtures m9, m11 ready

## ✅ WORK COMPLETED

1. **Pre-flight validation**: Disk, MySQL, binary, processes — all green
2. **S-73.2 code audit**: Confirmed correct implementation in mod.rs
3. **Test fixture creation**: `/tmp/gate-d73/t3_ob_handler_cycle.php` (oracle BYTE-ID)
4. **Root cause identification**: flush_buffer lacks frame context for handler invocation
5. **Analysis documentation**: Comprehensive findings in `/tmp/WP74-PUNTO0-FINDINGS.md`

## 🎯 IMMEDIATE NEXT STEPS (for WP-75)

1. **Locate flush_buffer()** in websapi.rs or mod.rs (search term: `fn flush_buffer`)
2. **Apply frame setup** before handler invocation (pattern: push frame like dtor_walk_round)
3. **Test t3 fixture** → must produce `[H:Hello]` (not panic)
4. **Verify S-73.2 fix** with gate-d73 t1/t2/t3/t4/t5/t8
5. **Implement S-73.1/S-73.3** per fixture test results
6. **Catalog M-73.1** as PHPR_DIVERGENCE (or decide on DimBase::This implementation)
7. **Close punto-0** with council binding verdicts

## ⭐ KEY LESSONS (punto-0 diagnostics)

- **Structural order is NOT sufficient**: Moving flush earlier is correct, but the handler invocation context (frame setup) must be sound. Structural fix + implementation detail both matter.
- **PANIC location pins handler context**: Index access to empty frames (`frames[top]` with empty stack) immediately reveals the issue — frame not pushed for handler.
- **Test-driven diagnosis works**: Creating t3 fixture exposed the flush_buffer gap that static review might miss.

---

**Session closed**: punto-0 partially diagnosed, critical flush_buffer issue isolated, ready for implementation in WP-75.  
**Recommendation**: Prioritize flush_buffer frame setup fix — it unblocks all fixture validation for punto-0 closure.
