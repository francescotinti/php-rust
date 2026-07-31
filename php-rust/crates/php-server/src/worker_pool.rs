//! WP-77.6: Worker-Actor PHP-FPM-style pool
//!
//! Architecture (S-78.1.5: RetainSet is PER-REQUEST — the A-DS8 gate caught
//! the per-worker arena leaking one parked unit per include per request,
//! KS-DS-78-4; the CLI main's machinery is the faithful form, WP-68):
//! - N OS threads; each REQUEST gets a fresh RetainSet (the P-2 pin for unit
//!   modules, dropped after the Vm) and a FRESH Vm (no Vm reuse — the N=1
//!   reused-Vm architecture was REJECTED in WP-77.2)
//! - Cross-request reuse of compiled units is the THREAD-LOCAL unit cache
//!   (WP-63/66, Rc-owned, fingerprinted, ways-evicted — the opcache analogy);
//!   the RetainSet never was a lookup structure (write-only pin)
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
        /// Resolved FILESYSTEM path of the script (FPM's SCRIPT_FILENAME) —
        /// error banners render it, and the full-body oracle cmp in run-gate
        /// requires it to be the same path the oracle CLI sees (A-TH8).
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

    /// S-78.1.6 census counters (A-BB8/A-BG7, Council WP-79). Queue depth is
    /// the KH78-2 OBSERVABLE: a closed-sequential measurement run where any
    /// sample shows depth >1 is VOID (KB-78-2). Alloc phase counters live in
    /// crate::census_alloc; verdict-grade footprint figures come ONLY from
    /// the non-instrumented twin (KB-78-5/KL-78-5).
    #[cfg(feature = "census-instrumentation")]
    pub mod census {
        use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

        /// In-flight tasks across the pool channels (inc on dispatch, dec on
        /// worker pickup).
        pub static QUEUE_DEPTH: AtomicUsize = AtomicUsize::new(0);
        /// High-watermark of QUEUE_DEPTH since process start.
        pub static QUEUE_DEPTH_MAX: AtomicUsize = AtomicUsize::new(0);
        /// Requests completed pool-wide.
        pub static REQUESTS: AtomicU64 = AtomicU64::new(0);

        pub fn note_depth(d: usize) {
            QUEUE_DEPTH_MAX.fetch_max(d, Ordering::Relaxed);
        }
    }

    /// Worker pool: N OS threads; each request gets a fresh RetainSet (P-2
    /// pin) + a FRESH Vm, both dead at request end (A-MS6 — isolation by Vm
    /// death, A-DS2; leak-freedom by RetainSet death, KS-DS-78-4).
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
        /// - Receives WorkerTask via mpsc channel
        /// - Per request: declares a fresh RetainSet (P-67.5 drop order),
        ///   creates a FRESH Vm that borrows it, runs the lifecycle to
        ///   completion, then drops Vm and RetainSet (in that order)
        ///
        /// Lifecycle inside worker (P-67.5 drop order, per REQUEST):
        /// ```ignore
        /// while let Some(task) = rx.recv() {
        ///     let retain = RetainSet::new();  // request-lifetime pin
        ///     let mut vm = Vm { retain: &retain, ... };  // Borrowed
        ///     (task.handler)();  // request_start/shutdown/end inside
        ///     // vm dropped before retain, retain dropped before next request
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

        /// Worker thread main loop (P-67.5 drop order enforced per request).
        ///
        /// S-78.1.5: PER-REQUEST RetainSet (the A-DS8 gate proved the
        /// per-worker arena leaked: FrozenVec is append-only and park_module
        /// pins one clone per include per request — KS-DS-78-4 fired, len
        /// 1→4 over 4 requests; with WP-scale includes that is an unbounded
        /// per-request leak).
        /// - RetainSet is declared per REQUEST: the P-2 pin keeping unit
        ///   modules alive for the request ('m lifetime), dropped after Vm
        /// - Cross-request compiled-unit reuse belongs to the THREAD-LOCAL
        ///   unit cache (WP-63/66) which persists on this worker thread —
        ///   Rc-owned, fingerprinted, ways-evicted (opcache analogy)
        /// - Per-request: request_start → run → request_shutdown → request_end
        /// - request_end() resets per-request state (statics are NOT in its
        ///   scope: they die with the Vm — A-DS2)
        /// - Two sequential requests → byte-identical output (G-APERTURA-2 gate)
        fn worker_loop(
            _context: Arc<WorkerPoolContext>,
            mut rx: mpsc::UnboundedReceiver<WorkerTask>,
        ) {
            // Build the PHP builtins registry ONCE per worker (A-DL5/A-PP5:
            // registry() is NOT cached — building it per request was pure churn
            // that would have polluted the WP-78 alloc census).
            let reg = registry();

            // A-PP9 gate hook (KS-PP-6): deliberate panic trigger, armed ONLY
            // when PHPR_TEST_WORKER_PANIC is set in the environment — lets
            // gate-worker-panic.sh prove the fail-fast policy on the real
            // binary instead of asserting it in prose. Unarmed, the path is
            // an ordinary docroot lookup.
            let test_panic = std::env::var_os("PHPR_TEST_WORKER_PANIC").is_some();

            // Main request loop: each request gets a fresh RetainSet + fresh Vm
            while let Some(task) = rx.blocking_recv() {
                #[cfg(feature = "census-instrumentation")]
                census::QUEUE_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                if test_panic && task.meta.path.ends_with("/__phpr_panic") {
                    panic!("PHPR_TEST_WORKER_PANIC armed: deliberate worker panic (A-PP9 gate)");
                }
                // S-78.1.5: request-lifetime RetainSet (P-2 pin, the CLI
                // main's machinery). SAFETY (A-MS2): !Send + !Sync —
                // constructed HERE on this worker's stack, never crosses a
                // thread boundary (assert_not_impl_any, php-runtime); dropped
                // after the Vm at the end of this iteration (P-67.5), which
                // is what makes include parking leak-free (KS-DS-78-4).
                let retain = php_runtime::RetainSet::new();
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

            #[cfg(feature = "census-instrumentation")]
            {
                use std::sync::atomic::Ordering;
                let d = census::QUEUE_DEPTH.fetch_add(1, Ordering::Relaxed) + 1;
                census::note_depth(d);
            }

            Ok(())
        }
    }

    /// S-77.6.5.2.2 (retimed S-78.1.5): Execute PHP with a REQUEST-lifetime
    /// RetainSet — the P-2 pin for unit modules (bytecode only; A-DS6).
    ///
    /// Called from worker_loop with the &RetainSet created for THIS request;
    /// both die at the end of the loop iteration (Vm first, P-67.5).
    /// Module is recompiled per-request from handler.meta.source; compiled
    /// units are reused across requests by the thread-local unit cache.
    ///
    /// Lifecycle (Stogov order, per Council WP-77.5):
    /// 1. Compile new Module from request source
    /// 2. Create Vm borrowing the request's RetainSet (unit pins — bytecode
    ///    only, never the object store or userland state)
    /// 3. request_start: setup superglobals, INI, session
    /// 4. Execute user handler (PHP code via vm.run())
    /// 5. request_shutdown: shutdown functions, flush buffers, destructors
    /// 6. request_end: reset ephemeral state (byte-parity via G2)
    /// 7. Vm dropped, then the RetainSet (and its pinned units) with it
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
    /// - RetainSet (FrozenVec<Rc<Module>>) → request-lifetime PIN of unit
    ///   modules (S-78.1.5: it is append-only and write-only, so a per-worker
    ///   instance leaked one parked unit per include per request — KS-DS-78-4
    ///   fired). The opcache analogy belongs to the THREAD-LOCAL unit cache
    ///   (WP-63/66): bytecode/immutable data only — NEVER userland state.
    /// Rule: output capture before request_end() (Pedersen/Stogov permanent rule).
    pub fn execute_with_retain(
        retain: &php_runtime::RetainSet,
        reg: &php_runtime::Registry,
        meta: &WorkerHandlerMeta,
    ) -> (Vec<u8>, StatusCode) {
        // A-BB9 (S-78.1.6): NO per-request allocs in the fast path — the path
        // bytes are borrowed (no clone) and the lossy String is built only in
        // the error branches (it was unconditional: the census would have
        // billed it to "Axum overhead").
        let name: &[u8] = meta.path.as_bytes();
        let source = &meta.source;

        // Census phase boundaries (design78 §Contatori): a = lower+compile,
        // b = vm_new+request_start+run+render, c = shutdown+capture+end.
        // The census eprintln of the PREVIOUS request lands between its s3
        // and this request's s0 — self-traffic uncounted BY CONSTRUCTION
        // (WP-64: never meter a window that contains its own logging).
        #[cfg(feature = "census-instrumentation")]
        let census_s0 = crate::census_alloc::snapshot();

        // Phase 1: Compile PHP source to bytecode (per-request Module).
        // A-PP4/A-TH5/A-DS4 (Council WP-78): the HTTP boundary mirrors the PHP
        // boundary — EVERY fatal (compile or runtime) is 500 with the same
        // byte-parity body the CLI renders (FPM: 500 when headers not sent).
        let program = match php_runtime::lower_source(name, source) {
            Ok(p) => p,
            Err(php_runtime::LowerError::Fatal { message, line }) => {
                let file_s = String::from_utf8_lossy(name);
                let msg = format!(
                    "\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n"
                )
                .into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(e) => {
                // Parse error → 500. The ENVELOPE is the oracle's stdout form
                // ("\nParse error: … on line N\n"); the message text inside is
                // phpr's own parser diagnostic, which DIVERGES from Zend's
                // (registered in PHPR_DIVERGENCES_FROM_PHP.md §php-server,
                // A-DS10). NO parity claim on this body (KS-DS-78-5).
                let msg = format!("\nParse error: {e}\n").into_bytes();
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

        #[cfg(feature = "census-instrumentation")]
        let census_s1 = crate::census_alloc::snapshot();

        // Phase 3: Create Vm borrowing the request's RetainSet (P-2 pin;
        // S-78.1.5: pin and Vm die together at request end — cross-request
        // unit reuse is the thread-local unit cache's job)
        // - request_end() resets per-request state; statics die with the Vm (A-DS2)
        let mut vm = php_runtime::vm_new(retain, &module, reg, Some(&program));

        // Phase 3.5: link-time fatal check — the SAME sweep as the CLI main
        // (S-78.1.4/A-TH8: Zend's compile/link stage, e.g. an enum
        // implementing Serializable renders the plain banner).
        let link_fatal = vm.link_fatal_check(&module);
        let is_link_fatal = link_fatal.is_some();

        // Phase 4: Per-request lifecycle (Stogov order, Council WP-77.5)
        // 1. request_start: setup superglobals, INI, session
        vm.request_start(None, &[]);

        // 2. Execute PHP code
        vm.final_flush = false;
        let run_result = match link_fatal {
            Some(e) => Err(e),
            None => vm.run(),
        };
        // Routing off for everything past the main run (flush, fatal render,
        // shutdown destructors) — mirrors run_module_with_hir exactly.
        vm.final_flush = true;

        // Epilogue mirrors run_module_with_hir (A-TH8 fatal contract):
        // - exit()/die() is a CLEAN termination → HTTP 200, body = captured
        //   output (FPM semantics; the CLI surfaces the code, HTTP cannot)
        // - an uncaught throwable routed to set_exception_handler is NOT a
        //   fatal (no banner, 200 — same as the CLI main)
        // - a link fatal renders the plain banner (no throwable wrapping)
        // - any other fatal renders via render_fatal: the CLI-faithful form
        //   the corpus verifies byte-identical to the oracle
        //   ("Uncaught …\nStack trace:\n…  thrown in …")
        let (fatal, _return_value) = match run_result {
            Ok(v) => (None, v),
            Err(php_runtime::PhpError::Exit(_code)) => (None, php_runtime::Zval::Null),
            Err(e) if !is_link_fatal && vm.handle_uncaught_exception(&e) => {
                (None, php_runtime::Zval::Null)
            }
            Err(e) => (Some(e), php_runtime::Zval::Null),
        };

        // Flush diagnostics before shutdown
        let line = vm.fatal_line;
        let _ = vm.flush_diags(line);

        // Render fatal if present (PHP flushes active output buffers BEFORE
        // the fatal banner, so buffered script output precedes it)
        if let Some(err) = &fatal {
            vm.flush_all_output_buffers();
            if is_link_fatal {
                let file = String::from_utf8_lossy(&module.file);
                let block =
                    format!("\nFatal error: {} in {} on line {}\n", err.message(), file, line);
                vm.rendered.extend_from_slice(block.as_bytes());
            } else {
                vm.render_fatal(err, line);
            }
        }

        #[cfg(feature = "census-instrumentation")]
        let census_s2 = crate::census_alloc::snapshot();

        // 3. request_shutdown: shutdown functions, flush buffers, destructors
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
        // (G-APERTURA-2 gate: sequential requests must produce byte-identical output)
        vm.request_end();

        // Census line (stderr): phase diffs + per-worker observables. s3 is
        // taken BEFORE the eprintln so the logging's own allocs fall outside
        // every phase window (see the s0 comment).
        #[cfg(feature = "census-instrumentation")]
        {
            use std::sync::atomic::Ordering;
            let census_s3 = crate::census_alloc::snapshot();
            let req = census::REQUESTS.fetch_add(1, Ordering::Relaxed) + 1;
            eprintln!(
                "census: req={req} a_calls={} a_bytes={} b_calls={} b_bytes={} \
                 c_calls={} c_bytes={} retain_len={} live_objs={} depth_max={}",
                census_s1.0 - census_s0.0,
                census_s1.1 - census_s0.1,
                census_s2.0 - census_s1.0,
                census_s2.1 - census_s1.1,
                census_s3.0 - census_s2.0,
                census_s3.1 - census_s2.1,
                retain.len(),
                vm.live_objects(),
                census::QUEUE_DEPTH_MAX.load(Ordering::Relaxed),
            );
        }

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

        /// A-PP4/A-TH5/A-DS4 + A-TH8 (S-78.1.4): every fatal is HTTP 500 with
        /// the FULL CLI-faithful body — asserted as absolute content, never a
        /// `contains()` probe (KH79-2: parity claims on error bodies require
        /// the full body; the old `contains("Fatal error:")` was the same
        /// class of hole as `head -1`).
        #[test]
        fn fatal_maps_to_http_500_compile_and_runtime() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            // Runtime fatal: undefined function — render_fatal's uncaught-Error
            // form (the one the corpus pins byte-identical to the oracle).
            let (body, status) =
                execute_with_retain(&retain, &reg, &meta("<?php nope_missing_fn();"));
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(
                body,
                b"\nFatal error: Uncaught Error: Call to undefined function nope_missing_fn() \
                  in /gate.php:1\nStack trace:\n#0 {main}\n  thrown in /gate.php on line 1\n"
                    .to_vec(),
                "runtime fatal body != CLI-faithful full body (A-TH8): {:?}",
                String::from_utf8_lossy(&body)
            );
            // The worker survives a fatal: next request on the same RetainSet
            // is clean (boundary discipline).
            let (ok_body, ok_status) =
                execute_with_retain(&retain, &reg, &meta("<?php echo \"alive\";"));
            assert_eq!(ok_status, StatusCode::OK);
            assert_eq!(ok_body, b"alive");
        }

        /// A-TH8 contract: exit()/die() is a CLEAN termination — HTTP 200 with
        /// the captured output (FPM semantics: the exit code has no HTTP
        /// channel; it is NOT a fatal).
        #[test]
        fn exit_is_clean_termination_http_200() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let (body, status) = execute_with_retain(
                &retain,
                &reg,
                &meta("<?php echo \"out\"; exit(3); echo \"never\";"),
            );
            assert_eq!(status, StatusCode::OK, "exit() must not map to an error status");
            assert_eq!(body, b"out");
        }

        /// A-DS8 (S-78.1.5, KS-DS-78-4): the RetainSet is APPEND-ONLY and
        /// park_module pins one clone per include per request — this test, in
        /// its first (per-worker RetainSet) form, CAUGHT the leak: len went
        /// 1→4 over 4 requests. The fix is the CLI main's machinery: a fresh
        /// RetainSet per request (worker_loop shape, asserted here). Each
        /// request must pin EXACTLY the distinct units it includes (positive
        /// control: ≥1 — an all-zero counter is a failed check, KG-79.A) and
        /// the pin dies with the request — leak-freedom by destruction.
        #[test]
        fn include_units_pinned_per_request_not_leaked() {
            let reg = registry();
            let dir = std::env::temp_dir().join("phpr_gate_a_ds8");
            std::fs::create_dir_all(&dir).unwrap();
            let lib = dir.join("inc_lib.php");
            std::fs::write(
                &lib,
                "<?php function lib_tick() { static $n = 0; return ++$n; }",
            )
            .unwrap();
            let src = format!(
                "<?php require '{}'; echo lib_tick() . lib_tick();",
                lib.display()
            );
            for req in 1..=4 {
                // Worker_loop shape: fresh RetainSet per request (S-78.1.5).
                let retain = php_runtime::RetainSet::new();
                let (b, s) = execute_with_retain(&retain, &reg, &meta(&src));
                assert_eq!(s, StatusCode::OK);
                assert_eq!(
                    b, b"12",
                    "request {req}: static in included unit must restart per request"
                );
                assert_eq!(
                    retain.len(),
                    1,
                    "request {req}: pin must hold EXACTLY the distinct included units \
                     (0 = counter blind KG-79.A; >1 = duplicate parking KS-DS-78-4)"
                );
            }
        }

        /// A-DS9 (S-78.1.5): statics in methods and in INHERITED methods (PHP
        /// 8.1+: a child inheriting the method SHARES the parent's static).
        /// Absolute body pinned from the oracle (`php static_methods.php` →
        /// "SM:base=12;child=3;again=4"); every request reproduces it fresh.
        #[test]
        fn method_and_inherited_statics_restart_each_request() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let src = "<?php \
                class Base { public static function tick() { static $n = 0; return ++$n; } } \
                class Child extends Base {} \
                echo \"SM:base=\" . Base::tick() . Base::tick() \
                   . \";child=\" . Child::tick() . \";again=\" . Base::tick();";
            for i in 1..=3 {
                let (body, status) = execute_with_retain(&retain, &reg, &meta(src));
                assert_eq!(status, StatusCode::OK);
                assert_eq!(
                    body,
                    b"SM:base=12;child=3;again=4".to_vec(),
                    "request {i}: method/inherited statics diverged from oracle pin (A-DS9): {:?}",
                    String::from_utf8_lossy(&body)
                );
            }
        }

        /// A-TH8 contract: an uncaught throwable routed to a
        /// set_exception_handler is NOT a fatal — no banner, HTTP 200, same as
        /// the CLI main's epilogue (the worker mirrors run_module_with_hir).
        #[test]
        fn uncaught_routed_to_exception_handler_no_banner() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let src = "<?php set_exception_handler(function ($e) { \
                       echo \"handled:\" . $e->getMessage(); }); \
                       throw new Exception(\"boom\");";
            let (body, status) = execute_with_retain(&retain, &reg, &meta(src));
            assert_eq!(status, StatusCode::OK);
            assert_eq!(
                body,
                b"handled:boom".to_vec(),
                "exception-handler output wrong or banner leaked: {:?}",
                String::from_utf8_lossy(&body)
            );
        }

        /// KG-79.A positive control (KB-78-6): the census alloc counters MUST
        /// move across a request — a census whose counters were never seen
        /// moving is VOID.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_alloc_counters_move() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let before = crate::census_alloc::snapshot();
            let (b, s) = execute_with_retain(&retain, &reg, &meta("<?php echo \"x\";"));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"x");
            let after = crate::census_alloc::snapshot();
            assert!(
                after.0 > before.0 && after.1 > before.1,
                "alloc counters did not move across a request (KG-79.A): {before:?} -> {after:?}"
            );
        }

        /// A-BB8/A-TH9 burst positive control: the queue-depth watermark (the
        /// KH78-2 observable) MUST see >1 when tasks are dispatched faster
        /// than one worker drains them — a depth counter that can never fire
        /// would bless any load shape.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_queue_depth_burst_positive() {
            let ctx = WorkerPoolContext::new();
            let pool = WorkerPool::new(ctx, 1);
            let mut rxs = Vec::new();
            for _ in 0..4 {
                let (tx, rx) = oneshot::channel();
                pool.dispatch(WorkerTask {
                    meta: meta("<?php for ($i = 0; $i < 50000; $i++);"),
                    response_tx: tx,
                })
                .unwrap();
                rxs.push(rx);
            }
            for rx in rxs {
                let (_, s) = rx.blocking_recv().unwrap();
                assert_eq!(s, StatusCode::OK);
            }
            let peak = census::QUEUE_DEPTH_MAX.load(std::sync::atomic::Ordering::Relaxed);
            assert!(
                peak >= 2,
                "burst did not raise the depth watermark (A-BB8 positive control): peak={peak}"
            );
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
