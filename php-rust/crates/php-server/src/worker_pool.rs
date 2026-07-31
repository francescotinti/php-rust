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
//!
//! Send/Sync Safety (A-MS2, Council WP-78 — supersedes the FALSE A-MS1 text):
//! - RetainSet is `!Send + !Sync` (`Rc<Module>` refcounts are non-atomic and
//!   elsa's FrozenVec is the non-sync variant). Safety comes from thread-affine
//!   CONSTRUCTION: each RetainSet is created inside worker_loop, lives on that
//!   worker's stack and never crosses a thread boundary — the compiler rejects
//!   any migration (assert_not_impl_any in php-runtime, KS-MS-1).
//! - The ONLY value crossing the Axum→worker channel is WorkerTask
//!   (String/Vec/oneshot::Sender), positively asserted Send below (A-MS3).
//! - Vm borrows the worker-local RetainSet and drops in the owning worker
//!   thread before it (borrowck-enforced drop order, P-67.5).

#[cfg(feature = "axum-server")]
mod implementation {
    use std::sync::Arc;
    use tokio::sync::{mpsc, oneshot};
    use axum::http::StatusCode;
    use php_builtins::registry;

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

    /// Per-request handler metadata (sent to worker, not the Vm).
    pub struct WorkerHandlerMeta {
        pub path: String,
        pub method: String,
        pub body: Vec<u8>,
        pub source: Vec<u8>,
    }

    /// Task sent from Axum handler to worker thread (metadata only, Vm lives in worker).
    pub struct WorkerTask {
        pub meta: WorkerHandlerMeta,
        pub response_tx: oneshot::Sender<(Vec<u8>, StatusCode)>,
    }

    // A-MS3 positive control (WP-72 lesson: an all-negative counter is a failed
    // check): the only value that crosses the Axum→worker channel MUST be Send.
    static_assertions::assert_impl_all!(WorkerTask: Send);

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
        /// S-77.6.5.2.2: Persistent RetainSet per worker thread (php-fpm-style actor).
        /// - RetainSet is declared ONCE (server-lifetime arena for object table, class cache)
        /// - Vm is created per-request but borrows persistent RetainSet
        /// - Per-request: request_start → run → request_shutdown → request_end
        /// - RetainSet lifetime: entire worker thread (never Send/migrated)
        ///
        /// Architecture (Council WP-77.5 P-77.5.2, P-77.5.3, P-77.5.4):
        /// - Each request gets new Vm instance, new Module from source
        /// - But Vm borrows same RetainSet → parked unit modules persist
        /// - request_end() resets per-request state (statics are NOT in its
        ///   scope: they die with the Vm — A-DS2)
        /// - Two sequential requests on same RetainSet → byte-identical output (G-APERTURA-2 gate)
        fn worker_loop(
            _context: Arc<WorkerPoolContext>,
            mut rx: mpsc::UnboundedReceiver<WorkerTask>,
        ) {
            // Get the PHP builtins registry once for all requests
            let reg = php_builtins::registry();

            // P-67.5: RetainSet for entire thread (MUST be declared first so Vm dies first)
            // This persists: parked unit modules (compiled bytecode; a module
            // survives only while the Rc-owned unit cache still holds it)
            // SAFETY (A-MS2): RetainSet is !Send + !Sync — it is constructed HERE,
            // on this worker's stack, and never crosses a thread boundary; the
            // compiler enforces the affinity (assert_not_impl_any, php-runtime).
            let retain = php_runtime::RetainSet::new();

            // Main request loop: each request gets fresh Vm that borrows persistent RetainSet
            while let Some(task) = rx.blocking_recv() {
                // Execute handler: creates Vm per-request, reuses RetainSet per-thread
                let (response, status) = RequestHandler::execute_with_retain(&retain, &task.meta);
                let _ = task.response_tx.send((response, status));
            }
        }

        /// Dispatch task to next available worker (round-robin).
        /// Caller must create the oneshot channel, hold the receiver,
        /// pass the sender to WorkerTask, and await the receiver.
        pub fn dispatch(&self, task: WorkerTask) -> Result<(), String> {
            let idx = self
                .next_worker
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                % self.senders.len();

            self.senders[idx]
                .send(task)
                .map_err(|_| "worker channel closed".to_string())?;

            Ok(())
        }
    }

    /// Per-request handler context (created by Axum handler for each request).
    pub struct RequestHandler {
        pub path: String,
        pub method: String,
        pub body: Vec<u8>,
        /// PHP source code to execute in this request.
        pub source: Vec<u8>,
    }

    impl RequestHandler {
        /// S-77.6.5.2.2: Execute PHP with persistent RetainSet (per-thread persistent object table).
        ///
        /// Called from worker_loop with &RetainSet that persists across all requests in this thread.
        /// Vm is created per-request but borrows the thread-persistent RetainSet.
        /// Module is recompiled per-request from handler.meta.source.
        ///
        /// Lifecycle (Stogov order, per Council WP-77.5):
        /// 1. Compile new Module from request source
        /// 2. Create Vm from persistent RetainSet (object table, class cache persist)
        /// 3. request_start: setup superglobals, INI, session
        /// 4. Execute user handler (PHP code via vm.run())
        /// 5. request_shutdown: shutdown functions, flush buffers, destructors
        /// 6. request_end: reset ephemeral state for next request (byte-parity via G2)
        /// 7. Vm dropped (returns to stack), but objects in RetainSet persist
        ///
        /// ZEND SEMANTICS (A-DS2, Council WP-78 — supersedes the non-conform A-DS1):
        /// Contract = PHP-FPM (full RINIT→RSHUTDOWN per request), NOT the CLI:
        /// userland state NEVER survives a request. Mechanism per row:
        /// - superglobals, constants, handlers, OB stack, generators, resource
        ///   ids, per-request caches → `request_end()` reset (vm/mod.rs).
        /// - function statics, closure statics, class static props → Vm DEATH:
        ///   they are fields of the per-request Vm and `request_end()` does NOT
        ///   touch them. They MUST remain per-request even if the Vm ever became
        ///   persistent (KS-DS-78-3: any persistent-Vm proposal requires the
        ///   stateful gate A-DS3 green first; KS-DS-78-1: static counter printing
        ///   ≠1 on request N≥2 is FATAL). Gate: wp78-harness/gate-axum
        ///   gate_stateful.php.
        /// - object store → emptied by the WP-72 mass-teardown break phase at
        ///   request teardown (signature: used_n=0 after request_end, KS-DS-78-2).
        /// - RetainSet (FrozenVec<Rc<Module>>) → persists across requests as the
        ///   arena of compiled unit modules: bytecode/immutable data only (the
        ///   opcache analogy) — NEVER userland state.
        /// Rule: output capture before request_end() (Pedersen/Stogov permanent rule).
        pub fn execute_with_retain(
            retain: &php_runtime::RetainSet,
            meta: &WorkerHandlerMeta,
        ) -> (Vec<u8>, StatusCode) {
            let reg = registry();
            let name = meta.path.clone().into_bytes();
            let source = &meta.source;

            // Phase 1: Compile PHP source to bytecode (per-request Module)
            let program = match php_runtime::lower_source(&name, source) {
                Ok(p) => p,
                Err(php_runtime::LowerError::Fatal { message, line }) => {
                    let msg = format!("PHP Fatal: {} on line {}\n", message, line).into_bytes();
                    return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                }
                Err(e) => {
                    let msg = format!("PHP Lower Error: {:?}\n", e).into_bytes();
                    return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Phase 2: Compile to bytecode module
            let module = match php_runtime::compile_program(&program, &reg) {
                Ok(m) => m,
                Err(php_runtime::CompileError::Unsupported(what)) => {
                    let msg = format!("PHP Unsupported: {}\n", what).into_bytes();
                    return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Phase 3: Create Vm from persistent RetainSet
            // S-77.6.5.2.2: This is the KEY architectural change
            // - Vm is created per-request (fresh instance)
            // - But it borrows thread-persistent RetainSet
            // - Parked unit modules persist across requests (bytecode only)
            // - request_end() resets per-request state; statics die with the Vm (A-DS2)
            let mut vm = php_runtime::vm_new(retain, &module, &reg, Some(&program));

            // Phase 4: Per-request lifecycle (Stogov order, Council WP-77.5)
            // 1. request_start: setup superglobals, INI, session
            vm.request_start(None, &[]);

            // 2. Execute PHP code
            vm.final_flush = false;
            let run_result = vm.run();

            // Handle exit/fatal
            let (fatal, _return_value) = match run_result {
                Ok(v) => (None, v),
                Err(php_runtime::PhpError::Exit(_code)) => (None, php_runtime::Zval::Null),
                Err(e) => (Some(e), php_runtime::Zval::Null),
            };

            // Flush diagnostics before shutdown
            let line = vm.fatal_line;
            let _ = vm.flush_diags(line);

            // Render fatal if present
            if let Some(err) = &fatal {
                vm.flush_all_output_buffers();
                let file = String::from_utf8_lossy(&module.file);
                let block = format!("\nFatal error: {} in {} on line {}\n", err.message(), file, line);
                vm.rendered.extend_from_slice(block.as_bytes());
            }

            // 3. request_shutdown: shutdown functions, flush buffers, destructors
            vm.final_flush = true;
            vm.request_shutdown();

            // SAFETY (A-PP1): Output capture MUST precede request_end() reset boundary.
            // Reset boundary (step 4) clears per-request state (superglobals, handlers, OB stack).
            // Output capture reads buffered output from rendered + stdout BEFORE reset.
            // Violation results in truncated or lost output on subsequent requests.
            // Enforced by dispatch boundary: handler runs, output captured, THEN reset.
            let mut response = Vec::new();
            response.extend_from_slice(&vm.rendered);
            response.extend_from_slice(&vm.stdout);

            // 4. request_end: reset ephemeral state for next request
            // (G-APERTURA-2 gate: two requests on same RetainSet must produce byte-identical output)
            vm.request_end();

            (response, StatusCode::OK)
        }

        /// Legacy closure-based execute (kept for backwards compatibility, not used in S-77.6.5.2.2).
        /// @deprecated: Use execute_with_vm with persistent Vm instead.
        #[deprecated(since = "0.77.6.5.2", note = "Use execute_with_vm with persistent Vm")]
        pub fn execute(&self) -> Box<dyn FnOnce() -> (Vec<u8>, StatusCode) + Send> {
            let path = self.path.clone();
            let source = self.source.clone();

            Box::new(move || {
                let name = path.clone().into_bytes();
                let reg = registry();

                let program = match php_runtime::lower_source(&name, &source) {
                    Ok(p) => p,
                    Err(php_runtime::LowerError::Fatal { message, line }) => {
                        let msg = format!("PHP Fatal: {} on line {}\n", message, line).into_bytes();
                        return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                    }
                    Err(e) => {
                        let msg = format!("PHP Lower Error: {:?}\n", e).into_bytes();
                        return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };

                let module = match php_runtime::compile_program(&program, &reg) {
                    Ok(m) => m,
                    Err(php_runtime::CompileError::Unsupported(what)) => {
                        let msg = format!("PHP Unsupported: {}\n", what).into_bytes();
                        return (msg, StatusCode::INTERNAL_SERVER_ERROR);
                    }
                };

                let retain = php_runtime::RetainSet::new();
                let mut vm = php_runtime::vm_new(&retain, &module, &reg, Some(&program));

                vm.request_start(None, &[]);
                vm.final_flush = false;
                let run_result = vm.run();

                let (fatal, _return_value) = match run_result {
                    Ok(v) => (None, v),
                    Err(php_runtime::PhpError::Exit(_code)) => (None, php_runtime::Zval::Null),
                    Err(e) => (Some(e), php_runtime::Zval::Null),
                };

                let line = vm.fatal_line;
                let _ = vm.flush_diags(line);

                if let Some(err) = &fatal {
                    vm.flush_all_output_buffers();
                    let file = String::from_utf8_lossy(&module.file);
                    let block = format!("\nFatal error: {} in {} on line {}\n", err.message(), file, line);
                    vm.rendered.extend_from_slice(block.as_bytes());
                }

                vm.final_flush = true;
                vm.request_shutdown();
                vm.request_end();

                let mut response = Vec::new();
                response.extend_from_slice(&vm.rendered);
                response.extend_from_slice(&vm.stdout);

                (response, StatusCode::OK)
            })
        }
    }
}

#[cfg(feature = "axum-server")]
pub use implementation::{RequestHandler, WorkerPool, WorkerPoolContext, WorkerTask, WorkerHandlerMeta};
