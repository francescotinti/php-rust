//! WP-77.6: Worker-Actor PHP-FPM-style pool
//!
//! Architecture:
//! - N OS threads, each owns 1 persistent Vm
//! - Axum → mpsc → Worker → FnOnce(Vm) → (Vm, Response) → Axum
//! - Vm never leaves owning worker thread (!Send pinned ownership)
//!
//! Binding constraints (Council WP-77.5):
//! - request_start() → handler → request_shutdown() → request_end()
//! - request_shutdown() MUST precede request_end() (Stogov order)
//! - No task_local (Vm is !Send, ephemeral ownership rejected)

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use axum::http::StatusCode;

/// Server-lifetime context: shared state for entire server duration.
///
/// Contains RetainSet + Module + Registry that Vm instances borrow from.
/// Lives as long as the server runs; passed to each worker thread at startup.
pub struct WorkerPoolContext {
    // TODO: RetainSet (arena for Vm allocations)
    // TODO: Module (compiled PHP code)
    // TODO: Registry (shared built-in functions/classes)
}

impl WorkerPoolContext {
    /// Create new context for server startup.
    pub fn new() -> Arc<Self> {
        Arc::new(WorkerPoolContext {
            // TODO: Initialize RetainSet + Module from PHP file
        })
    }
}

/// Type-erased handler closure sent to worker.
///
/// Worker executes: (handler)(vm) → (vm, response)
/// Ownership round-trip: handler receives Vm, returns (Vm, Response)
pub type WorkerHandlerFn = Box<dyn FnOnce() -> (Vec<u8>, StatusCode) + Send>;

/// Task sent from Axum handler to worker thread.
pub struct WorkerTask {
    pub handler: WorkerHandlerFn,
    pub response_tx: oneshot::Sender<(Vec<u8>, StatusCode)>,
}

/// Worker pool: N OS threads, each managing 1 Vm.
pub struct WorkerPool {
    context: Arc<WorkerPoolContext>,
    // Senders to each worker thread (one per worker)
    // TODO: Create N workers on pool init, store mpsc senders
}

impl WorkerPool {
    /// Create pool with N worker threads.
    ///
    /// Each worker thread:
    /// - Owns 1 Vm for server lifetime
    /// - Receives WorkerTask via mpsc
    /// - Executes handler closure in FnOnce context
    /// - Returns (Vm, Response) via oneshot
    pub fn new(context: Arc<WorkerPoolContext>, num_workers: usize) -> Arc<Self> {
        let pool = Arc::new(WorkerPool {
            context,
            // TODO: Spawn N worker threads
            // TODO: Store mpsc senders for dispatch
        });

        pool
    }

    /// Dispatch task to next available worker.
    ///
    /// TODO: Implement round-robin or queue-based dispatch
    pub async fn dispatch(&self, task: WorkerTask) -> Result<(Vec<u8>, StatusCode), String> {
        // TODO: Send task to worker via mpsc
        // TODO: Await response on oneshot
        Err("dispatch not implemented".into())
    }
}

/// Per-request handler context (created by Axum handler for each request).
///
/// Captures request data + HTTP context, executed as FnOnce in worker thread.
pub struct RequestHandler {
    pub path: String,
    pub method: String,
    pub body: Vec<u8>,
}

impl RequestHandler {
    /// Execute handler in worker context (with Vm passed via FnOnce).
    ///
    /// This closure is sent to worker thread and called with:
    ///   closure(vm) → (vm, response_bytes)
    ///
    /// Lifecycle (Stogov order):
    /// 1. request_start(&vm) — initialize registry
    /// 2. Execute user handler (PHP code)
    /// 3. request_shutdown(&vm) — Zend teardown
    /// 4. request_end(&vm) — finalization
    pub fn execute(&self) -> WorkerHandlerFn {
        let path = self.path.clone();

        Box::new(move || {
            // TODO: Receive Vm from worker thread
            // let mut vm = ...;
            // request_start(&vm);
            // let response = handle_php_request(&vm, &path);
            // request_shutdown(&vm);
            // request_end(&vm);
            // return (response_bytes, status_code)

            // Placeholder for G1 spike
            let response = format!("WP-77.6: path={}", path).into_bytes();
            (response, StatusCode::OK)
        })
    }
}
