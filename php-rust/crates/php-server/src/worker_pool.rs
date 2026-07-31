//! WP-77.6: Worker-Actor PHP-FPM-style pool
//!
//! Architecture:
//! - N OS threads, each owning 1 persistent RetainSet; a FRESH Vm is created
//!   per request and borrows the worker's RetainSet (no Vm reuse — the N=1
//!   reused-Vm architecture was REJECTED in WP-77.2)
//! - Axum → mpsc → Worker → execute_with_retain() → Response → Axum
//! - Vm and RetainSet never leave the owning worker thread (!Send)
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
//!
//! Panic policy (A-PP9, Council WP-79 — KS-PP-6): FAIL-FAST.
//! - A Rust panic in a worker thread ABORTS the whole process. Without this a
//!   panicked worker died silently and round-robin kept dispatching to its
//!   queue: 1/N of all requests blackholed. No respawn — respawning would mask
//!   state corruption. A PHP fatal is NOT a panic: workers survive fatals
//!   (see fatal_maps_to_http_500_compile_and_runtime).
//! - Executable gate: wp78-harness/gate-worker-panic.sh — the deliberate
//!   panic trigger is armed ONLY by the PHPR_TEST_WORKER_PANIC env var.
//! - Teardown (A-MS7/A-DL6): shutdown() drops the senders and JOINS every
//!   worker; exit-time figures without that join are ADVISORY (KS-MS-5,
//!   KL-78-4).

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
    /// A-BB5 (Council WP-78): deliberately minimal — HTTP method/body have no
    /// channel into the Vm yet (superglobal seeding is SAPI-layer work, WP-79+);
    /// dead fields here would be counted as pool overhead by the alloc census.
    pub struct WorkerHandlerMeta {
        pub path: String,
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

    /// Worker pool: N OS threads, each owning 1 thread-persistent RetainSet
    /// (arena of compiled unit modules); a FRESH Vm is created per request
    /// and dies at request end (A-MS6 — isolation by Vm death, A-DS2).
    pub struct WorkerPool {
        senders: Vec<mpsc::UnboundedSender<WorkerTask>>,
        next_worker: std::sync::atomic::AtomicUsize,
        // A-MS7/A-DL6 (Council WP-79): join handles retained so shutdown()
        // can prove worker teardown before process-exit stats are read
        // (KS-MS-5/KL-78-4: exit figures without a join are ADVISORY).
        handles: Vec<std::thread::JoinHandle<()>>,
    }

    impl WorkerPool {
        /// Create pool with N worker threads (default: CPU count).
        ///
        /// Each worker thread:
        /// - Declares RetainSet (server-lifetime arena of unit modules, P-67.5)
        /// - Creates a FRESH Vm per request that borrows the RetainSet
        /// - Receives WorkerTask via mpsc channel
        /// - Runs the request lifecycle to completion, then drops the Vm
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
            let mut handles = Vec::new();
            for _i in 0..num {
                let (tx, rx) = mpsc::unbounded_channel::<WorkerTask>();
                let ctx = Arc::clone(&context);

                // Spawn worker thread (P-67.5: RetainSet before Vm).
                // A-PP9 FAIL-FAST (KS-PP-6): any worker panic aborts the
                // process — the default panic hook has already printed the
                // payload and backtrace by the time catch_unwind sees it.
                let handle = std::thread::spawn(move || {
                    let unwind =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            Self::worker_loop(ctx, rx);
                        }));
                    if unwind.is_err() {
                        eprintln!(
                            "php-server: worker panicked — aborting (A-PP9 fail-fast, KS-PP-6)"
                        );
                        std::process::abort();
                    }
                });

                senders.push(tx);
                handles.push(handle);
            }

            Arc::new(WorkerPool {
                senders,
                next_worker: std::sync::atomic::AtomicUsize::new(0),
                handles,
            })
        }

        /// Worker thread main loop (P-67.5 drop order enforced).
        ///
        /// S-77.6.5.2.2: Persistent RetainSet per worker thread (php-fpm-style actor).
        /// - RetainSet is declared ONCE (server-lifetime arena of compiled unit
        ///   modules — bytecode only, NEVER userland state; A-DS6)
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
            // Build the PHP builtins registry ONCE per worker (A-DL5/A-PP5:
            // registry() is NOT cached — building it per request was pure churn
            // that would have polluted the WP-78 alloc census).
            let reg = registry();

            // P-67.5: RetainSet for entire thread (MUST be declared first so Vm dies first)
            // This persists: parked unit modules (compiled bytecode; a module
            // survives only while the Rc-owned unit cache still holds it)
            // SAFETY (A-MS2): RetainSet is !Send + !Sync — it is constructed HERE,
            // on this worker's stack, and never crosses a thread boundary; the
            // compiler enforces the affinity (assert_not_impl_any, php-runtime).
            let retain = php_runtime::RetainSet::new();

            // A-PP9 gate hook (KS-PP-6): deliberate panic trigger, armed ONLY
            // when PHPR_TEST_WORKER_PANIC is set in the environment — lets
            // gate-worker-panic.sh prove the fail-fast policy on the real
            // binary instead of asserting it in prose. Unarmed, the path is
            // an ordinary docroot lookup.
            let test_panic = std::env::var_os("PHPR_TEST_WORKER_PANIC").is_some();

            // Main request loop: each request gets fresh Vm that borrows persistent RetainSet
            while let Some(task) = rx.blocking_recv() {
                if test_panic && task.meta.path == "/__phpr_panic" {
                    panic!("PHPR_TEST_WORKER_PANIC armed: deliberate worker panic (A-PP9 gate)");
                }
                // Execute handler: creates Vm per-request, reuses RetainSet per-thread
                let (response, status) = execute_with_retain(&retain, &reg, &task.meta);
                let _ = task.response_tx.send((response, status));
            }
        }

        /// A-MS7/A-DL6 (Council WP-79): deterministic teardown — drop every
        /// sender (each worker's blocking_recv() then yields None and its
        /// loop ends) and JOIN every worker before returning. Only after this
        /// may process-exit figures (/usr/bin/time -l, MIMALLOC_SHOW_STATS)
        /// be read as verdict-grade (KS-MS-5/KL-78-4). Consumes the pool:
        /// call via Arc::try_unwrap once the router (the only other owner)
        /// has been dropped.
        pub fn shutdown(self) {
            let WorkerPool { senders, handles, .. } = self;
            drop(senders);
            let n = handles.len();
            for h in handles {
                // A panicked worker has already aborted the process (A-PP9),
                // so a join Err is unreachable — kept loud, not swallowed.
                if h.join().is_err() {
                    eprintln!("php-server: worker join failed after drain");
                }
            }
            eprintln!(
                "php-server: all {n} workers joined — exit stats are authoritative (KL-78-4)"
            );
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

    /// S-77.6.5.2.2: Execute PHP with persistent RetainSet (per-thread arena
    /// of compiled unit modules — bytecode only; A-DS6).
    ///
    /// Called from worker_loop with &RetainSet that persists across all requests in this thread.
    /// Vm is created per-request but borrows the thread-persistent RetainSet.
    /// Module is recompiled per-request from handler.meta.source.
    ///
    /// Lifecycle (Stogov order, per Council WP-77.5):
    /// 1. Compile new Module from request source
    /// 2. Create Vm from persistent RetainSet (parked unit modules persist —
    ///    bytecode only, never the object store or userland state)
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
    ///   touch them. They MUST remain per-request even if a Vm were ever
    ///   reused across requests (KS-DS-78-3: any Vm-reuse proposal requires the
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
        reg: &php_runtime::Registry,
        meta: &WorkerHandlerMeta,
    ) -> (Vec<u8>, StatusCode) {
        let name = meta.path.clone().into_bytes();
        let source = &meta.source;

        // Phase 1: Compile PHP source to bytecode (per-request Module).
        // A-PP4/A-TH5/A-DS4 (Council WP-78): the HTTP boundary mirrors the PHP
        // boundary — EVERY fatal (compile or runtime) is 500 with the same
        // byte-parity body the CLI renders (FPM: 500 when headers not sent).
        let file_s = String::from_utf8_lossy(&name).into_owned();
        let program = match php_runtime::lower_source(&name, source) {
            Ok(p) => p,
            Err(php_runtime::LowerError::Fatal { message, line }) => {
                let msg = format!(
                    "\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n"
                )
                .into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(e) => {
                let msg = format!("PHP Parse error: {e}\n").into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        // Phase 2: Compile to bytecode module
        let module = match php_runtime::compile_program(&program, reg) {
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
        let mut vm = php_runtime::vm_new(retain, &module, reg, Some(&program));

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
        // Violation results in truncated or lost output on subsequent requests.
        // Body = `rendered` ONLY: it is the CLI-faithful stream (stdout with
        // diagnostics inline + fatal tail); appending `stdout` too DOUBLED every
        // body — caught by the absolute-content gate tests (S-78.0, the head -1
        // check in the old HTTP gate was blind to it).
        let response = std::mem::take(&mut vm.rendered);

        // 4. request_end: reset ephemeral state for next request
        // (G-APERTURA-2 gate: two requests on same RetainSet must produce byte-identical output)
        vm.request_end();

        // A-PP4/A-TH5/A-DS4: runtime fatal → 500 (was an accidental 200; the
        // KS-M3 verbale claimed 500 — now true). Body stays byte-parity.
        let status = if fatal.is_some() {
            StatusCode::INTERNAL_SERVER_ERROR
        } else {
            StatusCode::OK
        };
        (response, status)
    }

    /// Gate tests (A-SK3/A-PP3, Council WP-78). They only compile under the
    /// axum-server feature — CI runs them via
    /// `cargo test --release -p php-server --features axum-server` (A-SK4);
    /// the default workspace suite covers none of this file.
    #[cfg(test)]
    mod tests {
        use super::*;

        fn meta(src: &str) -> WorkerHandlerMeta {
            WorkerHandlerMeta {
                path: "/gate.php".to_string(),
                source: src.as_bytes().to_vec(),
            }
        }

        /// A-SK3: in-cargo form of G-APERTURA-2 — two requests on the SAME
        /// RetainSet must produce byte-identical bodies AND match the absolute
        /// expected content (A-SK7/KS-SK-79.2: a parity assert may never stand
        /// alone — self-equality blessed the doubled-body bug).
        #[test]
        fn gate_apertura2_two_requests_same_retainset_byte_parity() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let (b1, s1) = execute_with_retain(&retain, &reg, &meta("<?php echo \"hello axum\\n\";"));
            let (b2, s2) = execute_with_retain(&retain, &reg, &meta("<?php echo \"hello axum\\n\";"));
            assert_eq!(s1, StatusCode::OK);
            assert_eq!(s2, StatusCode::OK);
            assert_eq!(
                b1, b"hello axum\n",
                "body != absolute expected content (A-SK7, KS-SK-79.2)"
            );
            assert_eq!(b1, b2, "bodies diverge (KS-SK-78.2 → REJECT)");
        }

        /// KS-DS-78-1 (A-DS3): function statics restart per request — request
        /// N≥2 printing ≠1 is FATAL (FPM semantics: isolation by Vm death).
        #[test]
        fn stateful_static_counter_restarts_each_request() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let src = "<?php function c() { static $n = 0; return ++$n; } echo c();";
            for i in 1..=3 {
                let (body, status) = execute_with_retain(&retain, &reg, &meta(src));
                assert_eq!(status, StatusCode::OK);
                assert_eq!(
                    body, b"1",
                    "request {i}: static counter leaked across requests (KS-DS-78-1)"
                );
            }
        }

        /// A-PP3 with POSITIVE control (KS-PP-1): buffered output (ob_start
        /// without explicit flush) survives two sequential requests — and the
        /// reset boundary really zeroes the buffers, proving a post-reset
        /// capture would lose the bytes.
        #[test]
        fn capture_before_reset_with_positive_control() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let src = "<?php ob_start(); echo \"buffered-bytes\";";
            let (b1, _) = execute_with_retain(&retain, &reg, &meta(src));
            let (b2, _) = execute_with_retain(&retain, &reg, &meta(src));
            assert_eq!(b1, b"buffered-bytes", "OB content lost on request 1");
            assert_eq!(b1, b2, "second request diverges (KS-PP-1)");

            // Positive control: replicate the lifecycle by hand and assert the
            // buffers are EMPTY after request_end() — i.e. the permanent rule
            // (capture BEFORE reset) is load-bearing, not decorative.
            let name = b"/gate.php".to_vec();
            let program = php_runtime::lower_source(&name, b"<?php echo \"payload\";").unwrap();
            let module = php_runtime::compile_program(&program, &reg).unwrap();
            let mut vm = php_runtime::vm_new(&retain, &module, &reg, Some(&program));
            vm.request_start(None, &[]);
            vm.final_flush = false;
            let _ = vm.run();
            vm.final_flush = true;
            vm.request_shutdown();
            assert!(
                !(vm.rendered.is_empty() && vm.stdout.is_empty()),
                "positive control broken: no output before request_end"
            );
            vm.request_end();
            // This is the one sanctioned post-reset read: it asserts EMPTINESS
            // (proving a capture here would lose bytes), it is not a capture.
            assert!(
                vm.rendered.is_empty() && vm.stdout.is_empty(), // grep-gate-allow: post-reset-emptiness-check
                "request_end() no longer clears the output buffers — \
                 the capture-before-reset rule lost its mechanism"
            );
        }

        /// A-PP4/A-TH5/A-DS4: every fatal is HTTP 500 — runtime fatal and
        /// compile fatal alike — with the CLI's byte-parity `Fatal error:` body.
        #[test]
        fn fatal_maps_to_http_500_compile_and_runtime() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            // Runtime fatal: undefined function.
            let (body, status) =
                execute_with_retain(&retain, &reg, &meta("<?php nope_missing_fn();"));
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert!(
                String::from_utf8_lossy(&body).contains("Fatal error:"),
                "runtime fatal body not byte-parity: {:?}",
                String::from_utf8_lossy(&body)
            );
            // The worker survives a fatal: next request on the same RetainSet
            // is clean (boundary discipline).
            let (ok_body, ok_status) =
                execute_with_retain(&retain, &reg, &meta("<?php echo \"alive\";"));
            assert_eq!(ok_status, StatusCode::OK);
            assert_eq!(ok_body, b"alive");
        }

        /// A-MS7/A-DL6 (KS-MS-5): shutdown() drops the senders and JOINS
        /// every worker — it returns only after real thread teardown (a hang
        /// here fails CI), with a real request drained through the pool
        /// first (positive control: the pool actually worked before dying).
        #[test]
        fn shutdown_joins_all_workers_after_drain() {
            let ctx = WorkerPoolContext::new();
            let pool = WorkerPool::new(ctx, 2);
            let (tx, rx) = oneshot::channel();
            let task = WorkerTask {
                meta: meta("<?php echo \"drain\";"),
                response_tx: tx,
            };
            pool.dispatch(task).unwrap();
            let (body, status) = rx.blocking_recv().unwrap();
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body, b"drain");
            let Ok(pool) = Arc::try_unwrap(pool) else {
                panic!("pool still shared — cannot exercise shutdown()");
            };
            pool.shutdown();
        }
    }
}

#[cfg(feature = "axum-server")]
pub use implementation::{WorkerPool, WorkerPoolContext, WorkerTask, WorkerHandlerMeta};
