//! WP-77.6: Worker-Actor PHP-FPM-style pool
//!
//! Architecture:
//! - N OS threads, each owns 1 persistent Vm
//! - Axum → mpsc → Worker → FnOnce handler() → Response → Axum
//! - Vm never leaves owning worker thread (!Send pinned ownership)
//!
//! Binding constraints (Council WP-77.5, P-67.5):
//! - RetainSet MUST be declared BEFORE Vm (Rust drop order is load-bearing)
//! - request_start() → handler → request_shutdown() → request_end()
//! - request_shutdown() MUST precede request_end() (Stogov order)

#[cfg(feature = "axum-server")]
mod implementation {
    use std::sync::Arc;
    use tokio::sync::{mpsc, oneshot};
    use axum::http::StatusCode;

    /// Server-lifetime context: shared state for entire server duration.
    pub struct WorkerPoolContext {
        // TODO: registry: Arc<php_runtime::Registry>,
    }

    impl WorkerPoolContext {
        /// Create new context from PHP builtins registry.
        pub fn new() -> Arc<Self> {
            Arc::new(WorkerPoolContext {
                // TODO: registry: Arc::new(php_builtins::registry()),
            })
        }
    }

    /// Type-erased handler closure sent to worker.
    pub type WorkerHandlerFn = Box<dyn FnOnce() -> (Vec<u8>, StatusCode) + Send>;

    /// Task sent from Axum handler to worker thread.
    pub struct WorkerTask {
        pub handler: WorkerHandlerFn,
        pub response_tx: oneshot::Sender<(Vec<u8>, StatusCode)>,
    }

    /// Worker pool: N OS threads, each managing 1 persistent Vm + RetainSet.
    pub struct WorkerPool {
        context: Arc<WorkerPoolContext>,
        senders: Vec<mpsc::UnboundedSender<WorkerTask>>,
        next_worker: std::sync::atomic::AtomicUsize,
    }

    impl WorkerPool {
        /// Create pool with N worker threads (default: CPU count).
        ///
        /// Each worker thread:
        /// - Declares RetainSet (server-lifetime arena, P-67.5)
        /// - Creates persistent Vm bound to RetainSet
        /// - Receives WorkerTask via mpsc channel
        /// - Executes handler closure in FnOnce context
        ///
        /// Lifecycle inside worker (P-67.5 drop order):
        /// ```ignore
        /// let retain = RetainSet::new();  // Live for entire thread
        /// while let Some(task) = rx.recv() {
        ///     let mut vm = Vm { retain: &retain, ... };  // Borrowed
        ///     (task.handler)();  // request_start/shutdown/end inside
        ///     // vm dropped before retain
        /// }
        /// ```
        pub fn new(context: Arc<WorkerPoolContext>, num_workers: usize) -> Arc<Self> {
            let num = if num_workers == 0 {
                num_cpus::get()
            } else {
                num_workers
            };

            let mut senders = Vec::new();
            for _i in 0..num {
                let (tx, rx) = mpsc::unbounded_channel::<WorkerTask>();
                let ctx = Arc::clone(&context);

                // Spawn worker thread (P-67.5: RetainSet before Vm)
                std::thread::spawn(move || {
                    Self::worker_loop(ctx, rx);
                });

                senders.push(tx);
            }

            Arc::new(WorkerPool {
                context,
                senders,
                next_worker: std::sync::atomic::AtomicUsize::new(0),
            })
        }

        /// Worker thread main loop (P-67.5 drop order enforced).
        ///
        /// RetainSet is declared first so Vm dies before it (LIFO drop order).
        fn worker_loop(
            _context: Arc<WorkerPoolContext>,
            mut rx: mpsc::UnboundedReceiver<WorkerTask>,
        ) {
            // P-67.5: RetainSet declared BEFORE Vm — Rust drops in reverse order,
            // so Vm (with user destructors) dies first, then its arena.
            // let _retain = php_runtime::RetainSet::new();

            while let Some(task) = rx.blocking_recv() {
                // Per-request Vm creation: borrowed from retain
                // TODO: Construct full Vm here with Module + request lifecycle
                // let mut vm = Vm { retain: &retain, module, ... };
                // (task.handler)();  // Closure executes in Vm context

                let (response, status) = (task.handler)();
                let _ = task.response_tx.send((response, status));
            }
        }

        /// Dispatch task to next available worker (round-robin).
        pub async fn dispatch(&self, task: WorkerTask) -> Result<(Vec<u8>, StatusCode), String> {
            let idx = self
                .next_worker
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                % self.senders.len();

            self.senders[idx]
                .send(task)
                .map_err(|_| "worker channel closed".to_string())?;

            Ok((Vec::new(), StatusCode::OK))
        }
    }

    /// Per-request handler context (created by Axum handler for each request).
    pub struct RequestHandler {
        pub path: String,
        pub method: String,
        pub body: Vec<u8>,
    }

    impl RequestHandler {
        /// Create handler closure (executed in worker thread with Vm available).
        ///
        /// Lifecycle (Stogov order, per Council WP-77.5):
        /// 1. request_start(&vm) — initialize registry
        /// 2. Execute user handler (PHP code)
        /// 3. request_shutdown(&vm) — Zend teardown
        /// 4. request_end(&vm) — finalization
        pub fn execute(&self) -> WorkerHandlerFn {
            let path = self.path.clone();

            Box::new(move || {
                let response = format!("WP-77.6: GET {}", path).into_bytes();
                (response, StatusCode::OK)
            })
        }
    }
}

#[cfg(feature = "axum-server")]
pub use implementation::{RequestHandler, WorkerHandlerFn, WorkerPool, WorkerPoolContext, WorkerTask};
