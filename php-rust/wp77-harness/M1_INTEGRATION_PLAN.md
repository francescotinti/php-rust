# M1 Integration Plan: Vm::request_end() into Axum Handler

**Date**: 2026-07-30  
**Phase**: WP-77.4.1 — M1 Axum Integration  
**Binding Decision**: Option B (Handler Wrapper) recommended per council mandate

---

## Context

After WP-77.3 implementation of `Vm::request_end()`, WP-77.4 applied binding amendments AMEND-1/2/3. Now WP-77.4.1 must integrate the reset mechanism into the Axum HTTP handler.

**Current State**:
- `Vm::request_end()` exists and compiles (phpr-wp77.4-amend 7bf53854)
- Axum handler (main.rs axum_handler module) has a placeholder tokio::task_local! with String
- No real Vm instance exists in the handler yet

**Challenge**: Vm is !Send/!Sync due to Rc<Cell<T>> fields. M9 tokio::task_local! binds to async task (not thread), but we still need a Vm instance.

---

## Three Integration Options

### Option A: Lazy Static with Once

**Approach**: Initialize Vm once per task using `tokio::sync::Mutex` + `std::sync::OnceLock`

```rust
tokio::task_local! {
    static VM_CELL: std::cell::RefCell<Option<Vm>>;
}

async fn hello_world() {
    VM_CELL.scope(/* ... */).await
}
```

**Pros**: Minimal changes, scales to multiple handlers  
**Cons**: Mutation across requests needs interior mutability; complex initialization ordering

### Option B: Handler Wrapper (Recommended ✓)

**Approach**: Create a wrapper handler that manages Vm lifecycle per-request

```rust
tokio::task_local! {
    static VM_LAZY: std::cell::RefCell<Option<Vm>>;
}

async fn with_vm_lifecycle<F>(handler: F) -> impl IntoResponse
where F: FnOnce(&mut Vm) -> String
{
    let vm = VM_LAZY.get_or_insert(/* create Vm */);
    let result = handler(&mut vm);
    vm.request_end();  // Reset after handler
    result
}

async fn hello_world() -> impl IntoResponse {
    with_vm_lifecycle(|vm| {
        // PHP execution here
        format!("Hello from Vm (object_id={})", vm.next_object_id)
    }).await
}
```

**Pros**: Clean request lifecycle, explicit reset call, testable  
**Cons**: Wrapper overhead per request

### Option C: Middleware (Future)

**Approach**: Use Tower middleware to intercept requests

```rust
#[derive(Clone)]
struct VmResetMiddleware {
    // task-local Vm access
}

impl Middleware for VmResetMiddleware { /* ... */ }
```

**Pros**: Reusable, decoupled from handlers  
**Cons**: More complex, defer to WP-77.5 post-M1

---

## Recommended: Option B+C Hybrid (WP-77.4.1)

**WP-77.4.1 (now)**: Implement Option B wrapper for hello_world handler
- Direct request_end() call proves M1 resets work
- Gate test verifies object_id==1 after reset
- Baseline for concurrent isolation (KM-77-2)

**WP-77.5 (future)**: Refactor to Tower middleware (Option C)
- Cleanly applies request_end() to all routes
- Enables M3 exception bridge at middleware layer

---

## Implementation Steps (WP-77.4.1)

1. **Vm initialization**: Create Vm instance from Module + RetainSet (lazy, per-task)
2. **Handler wrapper**: `with_vm_lifecycle()` creates/borrows Vm, calls handler, calls request_end()
3. **hello_world integration**: Call wrapper, execute PHP snippet via run_module_with_hir
4. **Test gate KS-M1-Complete**: Verify next_object_id == 1 after reset
5. **Concurrent test KM-77-2**: Two parallel requests on same task, verify isolation via $GLOBALS

---

## Vm Initialization Strategy

### Challenge
- `Vm::new(retain, module)` requires 'static or explicit lifetime management
- Module is `&'m Module`, tied to handler scope
- Cannot store Module in tokio::task_local! (it's borrowed, not owned)

### Solution: Lazy Initialization via Closure

```rust
tokio::task_local! {
    static VM_INSTANCE: std::cell::RefCell<Option<Vm<'static>>>;
}

// On each request:
let module_bytes = include_bytes!("hello.php");  // compiled to bytecode
let module = create_module_from_bytes(module_bytes);
let retain = RetainSet::new();
let vm = Vm::new(&retain, &module);
// Use vm...
vm.request_end();
```

OR (simpler): Store Module in a thread-local or once_cell

```rust
use once_cell::sync::Lazy;

static MODULE: Lazy<Module> = Lazy::new(|| {
    let bytes = include_bytes!("hello.php");
    create_module_from_bytes(bytes)
});

tokio::task_local! {
    static RETAIN: std::cell::RefCell<RetainSet>;
    static VM: std::cell::RefCell<Option<Vm>>;
}

async fn with_vm_lifecycle<F>(handler: F) -> impl IntoResponse {
    RETAIN.scope({
        let retain = RetainSet::new();
        VM.scope({
            let vm = Vm::new(&retain, &*MODULE);
            let result = handler(&mut vm);
            vm.request_end();
            result
        }).await
    }).await
}
```

---

## Kill-Switches (Binding Mandates)

| Switch | Condition | Success Metric |
|--------|-----------|-----------------|
| **KS-M1-Complete** | Vm.next_object_id == 1 after reset | Fixture gate test passes |
| **KM-77-2-Concurrent** | Two simultaneous requests on same task | $GLOBALS isolation verified |
| **KS-M1-Gate-Upgraded** | Byte-snapshot on 3 workloads | hello/WordPress/ORM all PASS |
| **KS-M4-Steady** | Alloc variance ≤±2% | Deferred to M4 phase |

---

## Testing (wp77-harness/fixtures/)

### Fixture: gate_ks_m1_complete.php
```php
<?php
// KS-M1-Complete: Verify next_object_id reset
$id1_before = /* first request */;
$id1_after = /* capture after reset */;
assert($id1_after == 1, "Object ID should reset to 1");
?>
```

### Fixture: gate_km77_2_concurrent.php
```php
<?php
// KM-77-2: Concurrent isolation via $GLOBALS
$_GLOBALS_before = $GLOBALS;
// Request 1: modify $GLOBALS
$GLOBALS['test_key'] = 'request_1';
// Simulate concurrent request 2
$GLOBALS_after_r1 = $GLOBALS;
// After isolation, $GLOBALS should reset
$GLOBALS['test_key'] = 'request_2';
assert($GLOBALS['test_key'] == 'request_2', "Isolation failed");
?>
```

---

## References

- **M1 spec**: WP77_OPTION_B_AMENDMENTS.md
- **Binding verdicts**: COUNCIL_WP77_REVIEWS.md
- **Handler code**: crates/php-server/src/main.rs (axum_handler module)
- **Vm::request_end()**: crates/php-runtime/src/vm/mod.rs (lines ~3320–3450)
