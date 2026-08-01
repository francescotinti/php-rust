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
//! - Axum → mpsc → Worker → execute_request() → Response → Axum (S-80.0.4:
//!   the RetainSet is constructed INSIDE the sealed entry-point — the
//!   &RetainSet form is module-private, KS-MS-81-3)
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

        /// QUEUED tasks across the pool channels. Protocol (A-TH1/KH80-4):
        /// increment in dispatch BEFORE send, decrement in the worker AFTER
        /// recv — the decrement can then never precede its increment, so the
        /// counter cannot underflow. SEMANTICS (A-TH9, Council WP-81): with
        /// the decrement at PICKUP this counts the QUEUE, not the requests in
        /// flight — one request EXECUTING plus one just picked up shows a
        /// watermark of 1, so pipelined overlap is invisible here. Any
        /// closed-sequential claim from this watermark alone is ADVISORY
        /// (KH81-1); the verdict-grade observable is OUTSTANDING below.
        pub static QUEUE_DEPTH: AtomicUsize = AtomicUsize::new(0);
        /// High-watermark of QUEUE_DEPTH since process start.
        pub static QUEUE_DEPTH_MAX: AtomicUsize = AtomicUsize::new(0);
        /// S-80.0.3 (A-TH9/KH81-1): requests inside the server — increment in
        /// dispatch BEFORE the channel send, decrement in the worker AFTER
        /// the response send. Queued AND executing both count, which is
        /// exactly what the queue-only watermark missed. A-TH15 re-scope
        /// (Council WP-82): the decrement is sequenced-after the response
        /// send but NOT atomic with it — a worker descheduled in the
        /// send→dec window lets a genuinely closed-sequential client show 2.
        /// A watermark >1 therefore means "run VOID: overlap OR the send→dec
        /// window" — fail-closed (it can invalidate a good run, never bless
        /// overlap), and it is never by itself a proof of overlap; the judge
        /// of the closed-sequential claim stays the driver's inflight_max<=1
        /// enforcement.
        pub static OUTSTANDING: AtomicUsize = AtomicUsize::new(0);
        /// High-watermark of OUTSTANDING since process start (census line
        /// field `inflight_max`; the measure driver enforces <=1).
        pub static OUTSTANDING_MAX: AtomicUsize = AtomicUsize::new(0);
        /// Requests completed pool-wide.
        pub static REQUESTS: AtomicU64 = AtomicU64::new(0);
        /// A-PP14 (Council WP-82): churn of requests whose census line was
        /// never emitted (fatal early-returns) — the SplitDrain guard books
        /// the s0→drop window here instead of discarding it silently, so
        /// the global identity reconciles WITH fatals too:
        /// Δglobal = Σ(a+b+c+resid over lines) + drained. Reported at pool
        /// teardown (`census-drained:` line); KS-PP-82-1 still VOIDs any
        /// figure from a run containing a fatal — these accumulators make
        /// the void VISIBLE in the ledger, they never bless such a run.
        pub static DRAINED_CALLS: AtomicU64 = AtomicU64::new(0);
        pub static DRAINED_BYTES: AtomicU64 = AtomicU64::new(0);

        /// S-80.0.3 tripwire (A-TH10/KH81-2): `d` arrives as fetch_add()+1,
        /// which is >=1 in true arithmetic — d==0 is reachable ONLY through a
        /// release-mode wrap (MAX+1 -> 0), i.e. the inc-after-send regression
        /// the anti-wrap test alone could NOT falsify (its inc/dec balance
        /// out and the drain==0 assert passes WITH the bug). The panic
        /// escalates to abort in production via the global hook (A-PP9/A-PP4).
        pub fn note_depth(d: usize) {
            if d == 0 {
                panic!(
                    "census depth tripwire: observed depth 0 after an increment — \
                     usize wrap, the inc-before-send protocol is broken (KH81-2)"
                );
            }
            QUEUE_DEPTH_MAX.fetch_max(d, Ordering::Relaxed);
        }

        /// Same tripwire contract for the OUTSTANDING watermark.
        pub fn note_outstanding(d: usize) {
            if d == 0 {
                panic!(
                    "census outstanding tripwire: observed 0 after an increment — \
                     usize wrap (KH81-2)"
                );
            }
            OUTSTANDING_MAX.fetch_max(d, Ordering::Relaxed);
        }

        thread_local! {
            /// s3 of the previous request on THIS worker: the s3→s0 gap is
            /// the residual channel (A-BB11 — the server-side traffic the
            /// phase windows deliberately exclude: fs read, WorkerTask,
            /// channels, the census line itself). Counters are global, so at
            /// --workers >1 the gap absorbs other workers' traffic — the
            /// measurement protocol pins --workers 1 (KB-78-3).
            pub static LAST_S3: std::cell::Cell<Option<(u64, u64)>> =
                const { std::cell::Cell::new(None) };
            /// Last request's ((a1_calls, a1_bytes), (a3_calls, a3_bytes)) —
            /// the emitter consumes take_split(), the in-cargo positive
            /// controls read the copy stashed here.
            pub static LAST_SPLIT: std::cell::Cell<((u64, u64), (u64, u64))> =
                const { std::cell::Cell::new(((0, 0), (0, 0))) };
        }

        /// A-SK8 (Council WP-80): tests observing the depth watermark must
        /// start from a clean one — without a reset, a PASS can be inherited
        /// from another test's traffic. Test-only: the production watermarks
        /// are process-lifetime by design and are never reset.
        #[cfg(test)]
        pub fn reset_depth_stats() {
            QUEUE_DEPTH_MAX.store(0, Ordering::Relaxed);
            OUTSTANDING_MAX.store(0, Ordering::Relaxed);
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
                    // SAFETY (A-MS4 Council WP-80, corrected A-TH11 Council
                    // WP-81): AssertUnwindSafe is sound here because the Err
                    // arm NEVER resumes — it aborts the process, so no code
                    // can observe state broken mid-panic. In axum mode the
                    // GLOBAL abort hook (main.rs) fires BEFORE any unwind:
                    // the process dies inside the hook and NO drops run —
                    // this catch_unwind is the belt for the non-axum/test
                    // contexts where the hook is not installed (there the
                    // unwind DOES run the in-flight request's Vm/RetainSet
                    // drops before abort()). Either way nothing resumes past
                    // a broken invariant.
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
                // Pickup: the task leaves the QUEUE (it stays OUTSTANDING
                // until its response is sent). S-80.0.3 tripwire (KH81-2): a
                // decrement finding 0 has no matching increment — wrap.
                #[cfg(feature = "census-instrumentation")]
                {
                    let old = census::QUEUE_DEPTH
                        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    if old == 0 {
                        panic!(
                            "census depth tripwire: pickup decrement on 0 — \
                             unmatched dec, inc-before-send protocol broken (KH81-2)"
                        );
                    }
                }
                if test_panic && task.meta.path.ends_with("/__phpr_panic") {
                    panic!("PHPR_TEST_WORKER_PANIC armed: deliberate worker panic (A-PP9 gate)");
                }
                // S-78.1.5: request-lifetime RetainSet (P-2 pin, the CLI
                // main's machinery). SAFETY (A-MS2): !Send + !Sync —
                // constructed HERE on this worker's stack, never crosses a
                // thread boundary (assert_not_impl_any, php-runtime); dropped
                // after the Vm at the end of this iteration (P-67.5), which
                // is what makes include parking leak-free (KS-DS-78-4).
                let (response, status) = execute_request(&reg, &task.meta);
                let _ = task.response_tx.send((response, status));
                // Response sent: the request leaves the server — only now
                // does OUTSTANDING drop (A-TH9: dec-after-send is what makes
                // the closed-sequential watermark verdict-grade).
                #[cfg(feature = "census-instrumentation")]
                {
                    let old = census::OUTSTANDING
                        .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    if old == 0 {
                        panic!(
                            "census outstanding tripwire: decrement on 0 — \
                             unmatched dec (KH81-2)"
                        );
                    }
                }
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
            // A-PP14: the drained ledger — nonzero means at least one
            // request's line is missing (rows==N catches it; the figure
            // makes the gap reconcilable instead of silent).
            #[cfg(feature = "census-instrumentation")]
            {
                use std::sync::atomic::Ordering;
                eprintln!(
                    "census-drained: calls={} bytes={} gross=1 (A-PP14: fatal-request churn; 0 on a clean run)",
                    census::DRAINED_CALLS.load(Ordering::Relaxed),
                    census::DRAINED_BYTES.load(Ordering::Relaxed),
                );
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

            // A-TH1/A-BB13/A-MS1 (Council WP-80, KH80-4): increment BEFORE
            // send. With inc-after-send the worker's pickup decrement could
            // precede the increment: usize underflow (wrap to MAX) and a
            // watermark that can UNDER-count — a fail-open observable for
            // KH78-2. Inc → send → recv → dec makes underflow impossible:
            // every decrement is preceded by its own increment (the channel
            // provides the happens-before edge). S-80.0.3: OUTSTANDING gets
            // the same inc-before-send edge; its dec is after the response
            // send in the worker (A-TH9).
            #[cfg(feature = "census-instrumentation")]
            {
                use std::sync::atomic::Ordering;
                let d = census::QUEUE_DEPTH.fetch_add(1, Ordering::Relaxed) + 1;
                census::note_depth(d);
                let o = census::OUTSTANDING.fetch_add(1, Ordering::Relaxed) + 1;
                census::note_outstanding(o);
            }

            if self.senders[idx].send(task).is_err() {
                // Failed send: no worker will ever pick this task up, so no
                // decrement will pair with the increments above — undo both.
                // A-MS16 (Council WP-82) DECLARED: the note_* watermarks ran
                // BEFORE the send and are NOT undone here — a failed dispatch
                // leaves a ghost in QUEUE_DEPTH_MAX/OUTSTANDING_MAX. A run
                // containing a dispatch Err is VOID by declaration; the
                // driver's rows==N check already catches it (the request
                // emits no census line), this comment makes the rule NAMED.
                #[cfg(feature = "census-instrumentation")]
                {
                    census::QUEUE_DEPTH.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                    census::OUTSTANDING.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                }
                return Err("worker channel closed".to_string());
            }

            Ok(())
        }
    }

    /// S-80.0.4 (A-MS8/KS-MS-81-3, Council WP-81): the SEALED entry-point —
    /// the RetainSet is constructed INSIDE, lives exactly one request, and no
    /// caller can hand in (or hold on to) one. With A-BB6 the main module
    /// parked in the RetainSet becomes the largest per-request object: a
    /// caller reusing a RetainSet across requests would accumulate one main
    /// per request — KS-DS-78-4 in aggravated form. The `&RetainSet` shape
    /// below is module-private (tests introspect the pin through it; nothing
    /// outside this module can), so the invariant lives in the type system,
    /// not in reviewer vigilance (the S-78.0 lesson).
    pub fn execute_request(
        reg: &php_runtime::Registry,
        meta: &WorkerHandlerMeta,
    ) -> (Vec<u8>, StatusCode) {
        // S-78.1.5: request-lifetime RetainSet (P-2 pin, the CLI main's
        // machinery). SAFETY (A-MS2): !Send + !Sync — constructed HERE on
        // the calling worker's stack, never crosses a thread boundary
        // (assert_not_impl_any, php-runtime); dropped after the Vm at the
        // end of this call (P-67.5), which is what makes include parking
        // leak-free (KS-DS-78-4).
        let retain = php_runtime::RetainSet::new();
        execute_with_retain(&retain, reg, meta)
    }

    /// S-77.6.5.2.2 (retimed S-78.1.5): Execute PHP with a REQUEST-lifetime
    /// RetainSet — the P-2 pin for unit modules (bytecode only; A-DS6).
    ///
    /// MODULE-PRIVATE since S-80.0.4 (KS-MS-81-3): production traffic enters
    /// through [`execute_request`]; this form exists so the in-module gate
    /// tests can inspect the pin (`retain.len()`) after the call.
    ///
    /// Called with a &RetainSet created for THIS request;
    /// both die at the end of the caller's scope (Vm first, P-67.5).
    /// A-BB6 (design79): the MAIN Module comes from `main_unit_acquire` —
    /// thread-local unit-cache HIT on steady state (a1+a2 skipped), the
    /// pre-lever lower+compile verbatim on a MISS; include units keep their
    /// own probe. Bytecode/immutable data only — NEVER userland state.
    ///
    /// Lifecycle (Stogov order, per Council WP-77.5):
    /// 1. Acquire MAIN Module (cache HIT or fresh compile of request source)
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
    fn execute_with_retain(
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

        // S-80.0.3 (A-PP11/KS-PP-81-1): drain guard — the fatal early-returns
        // below exit BEFORE the census emitter, leaving take_split residue
        // and a stale LAST_S3 that pollute the NEXT request's line (exactly
        // fixture F8's sequence: fatal → fix → 200). The CLI arm drains
        // unconditionally; this guard replicates that form on EVERY exit —
        // the emitter marks itself done, all other paths drain-and-discard
        // (rows==N still flags the missing line itself, A-PP5).
        #[cfg(feature = "census-instrumentation")]
        struct SplitDrain {
            emitted: bool,
            /// A-PP14: the request's s0 — the drop books the s0→drop window
            /// into the DRAINED_* accumulators (a fatal's churn used to
            /// vanish from the denominator entirely).
            s0: (u64, u64),
        }
        #[cfg(feature = "census-instrumentation")]
        impl Drop for SplitDrain {
            fn drop(&mut self) {
                if !self.emitted {
                    let _ = php_runtime::alloc_census::take_split();
                    let s = crate::census_alloc::snapshot();
                    census::DRAINED_CALLS.fetch_add(
                        s.0.saturating_sub(self.s0.0),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    census::DRAINED_BYTES.fetch_add(
                        s.1.saturating_sub(self.s0.1),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    census::LAST_S3.with(|c| c.set(Some((s.0, s.1))));
                }
            }
        }
        #[cfg(feature = "census-instrumentation")]
        let mut split_drain = SplitDrain { emitted: false, s0: census_s0 };

        // Phase 1+2 (A-BB6, design79 §2): acquire the MAIN unit — cache
        // probe on this long-lived SAPI (probe=true, A-TH14: the one
        // parameter; the shared acquire IS the pre-lever lower+compile on a
        // MISS). Error bodies unchanged (A-PP4/A-TH5/A-DS4 fatal contract):
        // EVERY fatal (compile or runtime) is 500 with the same byte-parity
        // body the CLI renders; the Parse-error ENVELOPE is the oracle's
        // stdout form with phpr's own diagnostic inside — NO parity claim on
        // that body (KS-DS-78-5, PHPR_DIVERGENCES_FROM_PHP.md §php-server).
        let mut unit = match php_runtime::main_unit_acquire(name, source, reg, true) {
            Ok(u) => u,
            Err(php_runtime::MainAcquireError::Lower(php_runtime::LowerError::Fatal {
                message,
                line,
            })) => {
                let file_s = String::from_utf8_lossy(name);
                let msg = format!(
                    "\nFatal error: {message} in {file_s} on line {line}\nStack trace:\n#0 {{main}}\n"
                )
                .into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(php_runtime::MainAcquireError::Lower(e)) => {
                let msg = format!("\nParse error: {e}\n").into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
            Err(php_runtime::MainAcquireError::Unsupported(what)) => {
                let msg = format!("PHP Unsupported: {}\n", what).into_bytes();
                return (msg, StatusCode::INTERNAL_SERVER_ERROR);
            }
        };

        #[cfg(feature = "census-instrumentation")]
        let census_s1 = crate::census_alloc::snapshot();

        // Phase 3 (design79 §4): the MAIN module's clone parks in the
        // request RetainSet BEFORE any borrow is lent (eviction/supersede
        // mid-request can never leave the cache as unique owner; A-DS8
        // pinned expectation moves 1→2 BY NAME: lib + main). The Program
        // half of the pair is `unit` itself living on this stack frame
        // (clone-on-stack, KS-PP-81-2). Then the Vm borrows the request's
        // RetainSet (P-2 pin; statics die with the Vm — A-DS2).
        let lever = unit.lever();
        // A-MS17 mint 2: the door key exists only while holding the acquired
        // main unit — a bypass of `main_unit_acquire` cannot compile.
        let gate = unit.vm_gate();
        retain.park_main(std::rc::Rc::clone(&unit.module), &gate);
        let mut vm = php_runtime::vm_new(retain, &unit.module, reg, Some(&unit.program), &gate);

        // Phase 3.5: link-time fatal check — the SAME sweep as the CLI main
        // (S-78.1.4/A-TH8: Zend's compile/link stage, e.g. an enum
        // implementing Serializable renders the plain banner).
        let link_fatal = vm.link_fatal_check(&unit.module);
        let is_link_fatal = link_fatal.is_some();
        // A-BB6 §6 (A-PP16/A-PP17): the put sits lexically AFTER
        // link_fatal_check and fires on a clean link only — a link-fatal
        // main is NEVER published (F8c: main_put==0, 500 byte-identical).
        if !is_link_fatal {
            lever.publish_if_armed();
        }

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
                let file = String::from_utf8_lossy(&unit.module.file);
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
            // S-79.0.3 (KH80-2/A-BB10/A-BB11): sub-channel split — a1 =
            // prelude, a2 = a − a1 (main proper), a3 = include-compile on
            // unit-cache miss (booked during the b window: includes compile
            // at run time; a3 is the compile share within b, what a cache
            // hit skips). resid = s3(prev)→s0(now) gap: the per-request
            // traffic OUTSIDE every phase window. A-PP14 (Council WP-82):
            // the identity Δglobal = Σ(a+b+c+resid) holds on FATAL-FREE
            // runs only — a fatal request emits no line and its s0→drop
            // churn goes to the DRAINED_* accumulators (census-drained:
            // teardown line): Δglobal = Σlines + drained, and the run is
            // VOID for census figures regardless (KS-PP-82-1).
            let ((a1_calls, a1_bytes), (a3_calls, a3_bytes)) =
                php_runtime::alloc_census::take_split();
            split_drain.emitted = true;
            census::LAST_SPLIT
                .with(|c| c.set(((a1_calls, a1_bytes), (a3_calls, a3_bytes))));
            let a_calls = census_s1.0 - census_s0.0;
            let a_bytes = census_s1.1 - census_s0.1;
            // S-80.0.3 (A-TH12): a1 is a sub-channel of a — a1>a means a
            // runtime compile satisfied the main-path classification and
            // accumulated into the WRONG request phase; the saturating_sub
            // below would silently floor a2 at 0 and mask it. FATAL in the
            // census build: a mis-attributed split is worse than no split.
            assert!(
                a1_calls <= a_calls && a1_bytes <= a_bytes,
                "census split violation: a1 ({a1_calls} calls/{a1_bytes} B) exceeds \
                 phase a ({a_calls} calls/{a_bytes} B) — a1 accumulated outside \
                 the s0..s1 window (A-TH12)"
            );
            let (resid_calls, resid_bytes) = census::LAST_S3.with(|c| {
                let gap = match c.get() {
                    Some((pc, pb)) => (census_s0.0 - pc, census_s0.1 - pb),
                    None => (0, 0),
                };
                c.set(Some((census_s3.0, census_s3.1)));
                gap
            });
            // gross=1 (A-DL10): every byte figure on this line is GROSS churn
            // (realloc at full size, deallocs never netted) — upper bound.
            eprintln!(
                "census: req={req} gross=1 a_calls={a_calls} a_bytes={a_bytes} \
                 a1_calls={a1_calls} a1_bytes={a1_bytes} a2_calls={} a2_bytes={} \
                 a3_calls={a3_calls} a3_bytes={a3_bytes} b_calls={} b_bytes={} \
                 c_calls={} c_bytes={} resid_calls={resid_calls} \
                 resid_bytes={resid_bytes} retain_len={} live_objs={} depth_max={} \
                 inflight_max={} a3_trip={}",
                a_calls.saturating_sub(a1_calls),
                a_bytes.saturating_sub(a1_bytes),
                census_s2.0 - census_s1.0,
                census_s2.1 - census_s1.1,
                census_s3.0 - census_s2.0,
                census_s3.1 - census_s2.1,
                retain.len(),
                vm.live_objects(),
                census::QUEUE_DEPTH_MAX.load(Ordering::Relaxed),
                census::OUTSTANDING_MAX.load(Ordering::Relaxed),
                php_runtime::alloc_census::trip_count(),
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

        // A-SK8: the depth counters are process-global — every test that
        // drives a WorkerPool (and thus dispatch's inc/dec) serializes here,
        // so watermark resets and end-of-drain assertions cannot race with
        // another test's pool traffic.
        static DEPTH_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

        // A-PP14/A-MS14 class: the DRAINED_* ledger is process-global — the
        // two fatal-generating tests serialize here so the positive control
        // (after > before) can never be satisfied by a CONCURRENT drain.
        // (census-gated like its two users: unused in the axum-only test
        // build, and A-AH6 makes that warning a red matrix.)
        #[cfg(feature = "census-instrumentation")]
        static DRAIN_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

        fn meta(src: &str) -> WorkerHandlerMeta {
            WorkerHandlerMeta {
                path: "/gate.php".to_string(),
                source: src.as_bytes().to_vec(),
            }
        }

        /// A-SK3: in-cargo form of G-APERTURA-2 — two requests must produce
        /// byte-identical bodies AND match the absolute expected content
        /// (A-SK7/KS-SK-79.2: a parity assert may never stand alone —
        /// self-equality blessed the doubled-body bug). S-79.0.6 (A-MS2):
        /// WORKER_LOOP FORM — fresh RetainSet per request, like production;
        /// the shared-RetainSet shape is the one KS-DS-78-4 banned and would
        /// silently change meaning once A-BB6 parks the main module.
        #[test]
        fn gate_apertura2_two_requests_worker_loop_form_byte_parity() {
            let reg = registry();
            let r1 = php_runtime::RetainSet::new();
            let (b1, s1) = execute_with_retain(&r1, &reg, &meta("<?php echo \"hello axum\\n\";"));
            let r2 = php_runtime::RetainSet::new();
            let (b2, s2) = execute_with_retain(&r2, &reg, &meta("<?php echo \"hello axum\\n\";"));
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
            let src = "<?php function c() { static $n = 0; return ++$n; } echo c();";
            for i in 1..=3 {
                // Worker_loop form (A-MS2): fresh RetainSet per request.
                let retain = php_runtime::RetainSet::new();
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
            let src = "<?php ob_start(); echo \"buffered-bytes\";";
            // Worker_loop form (A-MS2): fresh RetainSet per request.
            let ra = php_runtime::RetainSet::new();
            let (b1, _) = execute_with_retain(&ra, &reg, &meta(src));
            let rb = php_runtime::RetainSet::new();
            let (b2, _) = execute_with_retain(&rb, &reg, &meta(src));
            let retain = php_runtime::RetainSet::new();
            assert_eq!(b1, b"buffered-bytes", "OB content lost on request 1");
            assert_eq!(b1, b2, "second request diverges (KS-PP-1)");

            // Positive control: replicate the lifecycle by hand and assert the
            // buffers are EMPTY after request_end() — i.e. the permanent rule
            // (capture BEFORE reset) is load-bearing, not decorative.
            let name = b"/gate.php".to_vec();
            let program = php_runtime::lower_source(&name, b"<?php echo \"payload\";").unwrap();
            let module = php_runtime::compile_program(&program, &reg).unwrap();
            // A-MS17 mint 3: hand-replicated lifecycle — the probe mint, by name.
            let gate = php_runtime::vm_gate_probe();
            let mut vm = php_runtime::vm_new(&retain, &module, &reg, Some(&program), &gate);
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
            // Worker_loop form (A-MS2): fresh RetainSet per request.
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
            // The worker survives a fatal: the NEXT request (fresh RetainSet,
            // worker_loop form) is clean (boundary discipline).
            let retain2 = php_runtime::RetainSet::new();
            let (ok_body, ok_status) =
                execute_with_retain(&retain2, &reg, &meta("<?php echo \"alive\";"));
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
                // A-BB6 (design79 §4, DECLARED BY NAME — never a silent
                // bump): expected pins move 1 → 2 when the lever lands:
                // the included lib + the MAIN module (park_main, one
                // park-event per probed request).
                assert_eq!(
                    retain.len(),
                    2,
                    "request {req}: pin must hold EXACTLY the distinct included units \
                     plus the parked MAIN (A-BB6: lib + main = 2; \
                     0/1 = counter blind KG-79.A; >2 = duplicate parking KS-DS-78-4)"
                );
            }
        }

        /// A-PP20 (Council WP-83): the in-cargo tooth of "a link-fatal main
        /// is NEVER published". F8c's counter form (put==0) assumes the
        /// publish path is unique (A-PP16, grep-gated); this tooth ALSO
        /// probes the cache ENTRIES directly — a second insertion path
        /// would be caught even if the counters missed it. Real file path
        /// (the probe canonicalizes meta.path): probe delta == 2 across two
        /// requests, put == 0, hit == 0, and the (key, fp) is ABSENT.
        #[test]
        fn a_pp20_link_fatal_main_never_published_key_never_in_entries() {
            let reg = registry();
            let dir = std::env::temp_dir().join("phpr_gate_a_pp20");
            std::fs::create_dir_all(&dir).unwrap();
            let file = dir.join("lf.php");
            // Compile-OK, LINK-fatal (the F8c source): enum implementing
            // Serializable renders the plain fatal banner at link time.
            let src = "<?php\nenum LF implements Serializable { case A; }\necho \"never-reached\";";
            std::fs::write(&file, src).unwrap();
            let path = file.display().to_string();
            let (p0, h0, u0) = php_runtime::uc_main_stats();
            for req in 1..=2 {
                let retain = php_runtime::RetainSet::new();
                let (body, status) = execute_with_retain(
                    &retain,
                    &reg,
                    &WorkerHandlerMeta { path: path.clone(), source: src.as_bytes().to_vec() },
                );
                assert_eq!(
                    status,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "request {req}: link-fatal main must 500"
                );
                assert!(
                    String::from_utf8_lossy(&body).contains("Fatal error"),
                    "request {req}: body must carry the fatal banner"
                );
            }
            let (p1, h1, u1) = php_runtime::uc_main_stats();
            assert_eq!(p1 - p0, 2, "positive control: both requests must PROBE (vitality)");
            assert_eq!(u1 - u0, 0, "link-fatal main was PUT (A-PP16/A-PP20)");
            assert_eq!(h1 - h0, 0, "link-fatal main was served from cache (A-PP20)");
            assert!(
                !php_runtime::uc_main_key_in_cache(path.as_bytes(), src.as_bytes()),
                "link-fatal main's (key, fp) is IN the entries — a second \
                 insertion path exists (A-PP20)"
            );
        }

        /// A-DS9 (S-78.1.5): statics in methods and in INHERITED methods (PHP
        /// 8.1+: a child inheriting the method SHARES the parent's static).
        /// Absolute body pinned from the oracle (`php static_methods.php` →
        /// "SM:base=12;child=3;again=4"); every request reproduces it fresh.
        #[test]
        fn method_and_inherited_statics_restart_each_request() {
            let reg = registry();
            let src = "<?php \
                class Base { public static function tick() { static $n = 0; return ++$n; } } \
                class Child extends Base {} \
                echo \"SM:base=\" . Base::tick() . Base::tick() \
                   . \";child=\" . Child::tick() . \";again=\" . Base::tick();";
            for i in 1..=3 {
                // Worker_loop form (A-MS2): fresh RetainSet per request.
                let retain = php_runtime::RetainSet::new();
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

        /// S-79.0.3 positive controls (KH80-2/KB-80-1): the a1/a3 sub-channel
        /// counters must MOVE where they claim to and stay ZERO where they
        /// claim to. a1 (prelude) moves on every request; a3 moves on the
        /// FIRST include in a thread (unit-cache miss) and returns to 0 on
        /// the second (cache hit) — the exact discriminator the A-BB6 design
        /// is entitled to rely on (a counter never seen moving is blind,
        /// KG-79.A; one never seen at zero proves nothing either).
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_split_a1_prelude_and_a3_include_discriminate() {
            let _ = php_runtime::alloc_census::SNAPSHOT_FN.set(crate::census_alloc::snapshot);
            let reg = registry();

            // Request 1: hello — a1 > 0 (prelude lowered+compiled fresh),
            // a3 == 0 (no include anywhere in the request).
            let retain = php_runtime::RetainSet::new();
            let (b, s) = execute_with_retain(&retain, &reg, &meta("<?php echo \"h\";"));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"h");
            let ((a1, _), (a3, _)) = census::LAST_SPLIT.with(|c| c.get());
            assert!(a1 > 0, "a1 (prelude) never moved on a plain request (KG-79.A)");
            assert_eq!(a3, 0, "a3 moved without any include — window leak");

            // Request 2: first include on this thread — cache miss, a3 > 0.
            let dir = std::env::temp_dir().join("phpr_census_split_a3");
            std::fs::create_dir_all(&dir).unwrap();
            let lib = dir.join("split_lib.php");
            std::fs::write(&lib, "<?php function split_lib_f() { return 7; }").unwrap();
            let src = format!("<?php require '{}'; echo split_lib_f();", lib.display());
            let retain = php_runtime::RetainSet::new();
            let (b, s) = execute_with_retain(&retain, &reg, &meta(&src));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"7");
            let ((a1, _), (a3_miss, _)) = census::LAST_SPLIT.with(|c| c.get());
            assert!(a1 > 0, "a1 must move on the include request too");
            assert!(a3_miss > 0, "a3 (include-compile) never moved on a cache MISS");

            // Request 3: same include — thread-local unit cache HIT, a3 == 0.
            let retain = php_runtime::RetainSet::new();
            let (b, s) = execute_with_retain(&retain, &reg, &meta(&src));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"7");
            let (_, (a3_hit, _)) = census::LAST_SPLIT.with(|c| c.get());
            assert_eq!(
                a3_hit, 0,
                "a3 moved on a unit-cache HIT — either the cache missed \
                 (fingerprint drift) or the window brackets hit-path code"
            );
        }

        /// A-BB12/A-BG14 (KB-80-5): positive control for the c CHANNEL — a
        /// fixture whose teardown ALLOCATES must move the s2→s3 window.
        /// Until this exists, "c=0" is indistinguishable from a blind
        /// counter. The lifecycle is replicated by hand (same order as
        /// execute_with_retain) so the window can be read directly.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_c_channel_positive_control_teardown_allocates() {
            let reg = registry();
            let retain = php_runtime::RetainSet::new();
            let name = b"/gate.php".to_vec();
            let src = b"<?php register_shutdown_function(function () { \
                        $s = str_repeat('x', 65536); echo strlen($s); });";
            let program = php_runtime::lower_source(&name, src).unwrap();
            let module = php_runtime::compile_program(&program, &reg).unwrap();
            // A-MS17 mint 3: hand-replicated lifecycle — the probe mint, by name.
            let gate = php_runtime::vm_gate_probe();
            let mut vm = php_runtime::vm_new(&retain, &module, &reg, Some(&program), &gate);
            vm.request_start(None, &[]);
            vm.final_flush = false;
            let _ = vm.run();
            vm.final_flush = true;
            // c window opens here (mirror of census_s2 in execute_with_retain)
            let s2 = crate::census_alloc::snapshot();
            vm.request_shutdown();
            let response = std::mem::take(&mut vm.rendered);
            vm.request_end();
            let s3 = crate::census_alloc::snapshot();
            assert_eq!(response, b"65536", "shutdown function did not run in teardown");
            assert!(
                s3.0 > s2.0 && s3.1 > s2.1,
                "c window did not see the allocating teardown (KB-80-5): \
                 {s2:?} -> {s3:?} — the channel is blind"
            );
        }

        /// A-BB8/A-TH9 burst positive control: the queue-depth watermark (the
        /// KH78-2 observable) MUST see >1 when tasks are dispatched faster
        /// than one worker drains them — a depth counter that can never fire
        /// would bless any load shape.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_queue_depth_burst_positive() {
            let _guard = DEPTH_TEST_LOCK.lock().unwrap();
            census::reset_depth_stats();
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

        /// A-TH1/A-BB13/A-MS1 anti-wrap (KH80-4), re-scoped S-80.0.3 per
        /// A-TH10 (Council WP-81): this test alone is NOT the falsifier — in
        /// release, a wrap is modular arithmetic that self-cancels (balanced
        /// inc/dec drain to 0 and MAX+1 wraps back through the watermark
        /// unchanged), so these asserts pass WITH the inc-after-send
        /// regression installed. The falsifier is the d==0 TRIPWIRE in
        /// note_depth/the pickup dec (see census_depth_tripwire_bites_on_zero
        /// for the negative control): under the regression the wrapped 0/MAX
        /// values hit the tripwire and abort. This test remains as the
        /// exercise rig (200 dispatch→await cycles across the exact
        /// send/recv boundary) plus the drain==0 sanity floor.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn census_queue_depth_no_underflow_inc_before_send() {
            let _guard = DEPTH_TEST_LOCK.lock().unwrap();
            census::reset_depth_stats();
            let ctx = WorkerPoolContext::new();
            let pool = WorkerPool::new(ctx, 1);
            const N: usize = 200;
            for _ in 0..N {
                let (tx, rx) = oneshot::channel();
                pool.dispatch(WorkerTask {
                    meta: meta("<?php echo \"w\";"),
                    response_tx: tx,
                })
                .unwrap();
                let (b, s) = rx.blocking_recv().unwrap();
                assert_eq!(s, StatusCode::OK);
                assert_eq!(b, b"w");
                // A-TH22 (Council WP-83): the rescoped window doc ADMITS
                // that recv can complete while the worker sits between send
                // and the OUTSTANDING decrement — without this drain-sync
                // the opeak==1 assert below contradicts that doc and can
                // legitimately flake to 2. Bounded spin until drained.
                let mut spins: u64 = 0;
                while census::OUTSTANDING.load(std::sync::atomic::Ordering::Relaxed) != 0 {
                    std::thread::yield_now();
                    spins += 1;
                    assert!(spins < 10_000_000, "OUTSTANDING never drained after recv (A-TH22)");
                }
            }
            let depth = census::QUEUE_DEPTH.load(std::sync::atomic::Ordering::Relaxed);
            let peak = census::QUEUE_DEPTH_MAX.load(std::sync::atomic::Ordering::Relaxed);
            assert_eq!(
                depth, 0,
                "depth did not drain to 0: {depth} (a huge value here = usize wrap)"
            );
            assert!(
                (1..=N).contains(&peak),
                "watermark out of sane bounds: {peak} (usize wrap poisons it to ~MAX)"
            );
            // S-80.0.3: OUTSTANDING drains too, and its watermark must show
            // exactly 1 on a strictly awaited dispatch loop (A-TH9 — this is
            // the closed-sequential observable, not the queue watermark).
            let out = census::OUTSTANDING.load(std::sync::atomic::Ordering::Relaxed);
            let opeak = census::OUTSTANDING_MAX.load(std::sync::atomic::Ordering::Relaxed);
            assert_eq!(out, 0, "outstanding did not drain to 0: {out}");
            assert_eq!(
                opeak, 1,
                "closed-sequential loop must watermark OUTSTANDING at exactly 1: {opeak}"
            );
        }

        /// S-80.0.3 (KH81-2) NEGATIVE control: the depth tripwire must BITE
        /// on d==0 — reachable only through a release-mode wrap, which the
        /// anti-wrap drain test above provably cannot see (A-TH10). A
        /// tripwire never seen firing is a failed check (WP-72 lesson).
        #[cfg(feature = "census-instrumentation")]
        #[test]
        #[should_panic(expected = "census depth tripwire")]
        fn census_depth_tripwire_bites_on_zero() {
            census::note_depth(0);
        }

        /// KH81-2: same negative control for the OUTSTANDING watermark.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        #[should_panic(expected = "census outstanding tripwire")]
        fn census_outstanding_tripwire_bites_on_zero() {
            census::note_outstanding(0);
        }

        /// S-80.0.3 (A-PP11/KS-PP-81-1): a fatal early-return must DRAIN the
        /// split — its a1 residue polluted the NEXT request's line (exactly
        /// fixture F8's sequence: fatal → fix → 200). `goto x;` with no label
        /// is a LOWER-time fatal raised AFTER the prelude is lowered
        /// (validate_goto runs post-seeding), so the fatal request is
        /// GUARANTEED to have a1 residue pending when it early-returns.
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn fatal_early_return_drains_split_residue() {
            let _guard = DRAIN_TEST_LOCK.lock().unwrap();
            let _ = php_runtime::alloc_census::SNAPSHOT_FN.set(crate::census_alloc::snapshot);
            let reg = registry();
            // r1: clean hello pins the per-request a1 baseline.
            let r1 = php_runtime::RetainSet::new();
            let (_, s) = execute_with_retain(&r1, &reg, &meta("<?php echo \"a\";"));
            assert_eq!(s, StatusCode::OK);
            let ((a1_clean, _), _) = census::LAST_SPLIT.with(|c| c.get());
            assert!(a1_clean > 0, "clean hello did not move a1 (KG-79.A)");
            // r2: lower-time fatal — early-returns BEFORE the census emitter.
            let r2 = php_runtime::RetainSet::new();
            let (_, s) = execute_with_retain(&r2, &reg, &meta("<?php goto x;"));
            assert_eq!(
                s,
                StatusCode::INTERNAL_SERVER_ERROR,
                "goto-to-undefined-label must be a lower-time fatal"
            );
            // Direct falsifier: with the drain guard removed, the fatal
            // request's prelude a1 would still sit in the accumulator here.
            let ((undrained, _), _) = php_runtime::alloc_census::take_split();
            assert_eq!(
                undrained, 0,
                "fatal early-return left a1 residue undrained (A-PP11): {undrained}"
            );
            // r3: the next request's split must be its OWN, not clean+residue.
            let r3 = php_runtime::RetainSet::new();
            let (b, s) = execute_with_retain(&r3, &reg, &meta("<?php echo \"a\";"));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"a");
            let ((a1_after, _), _) = census::LAST_SPLIT.with(|c| c.get());
            assert!(
                a1_after <= a1_clean + a1_clean / 2,
                "a1 after a fatal request carries residue: {a1_after} vs clean {a1_clean} (A-PP11)"
            );
        }

        /// F6 (design79 §9, A-MS10/KS-MS-80-2 v2): park-EVENTS pinned — the
        /// same file included TWICE non-`_once` parks TWICE by construction,
        /// plus the MAIN's park (A-BB6): retain.len() == 2 + 1 = 3 EXACT.
        #[test]
        fn f6_double_include_parks_two_events_plus_main() {
            let reg = registry();
            let dir = std::env::temp_dir().join("phpr_gate_f6");
            std::fs::create_dir_all(&dir).unwrap();
            let lib = dir.join("f6_lib.php");
            std::fs::write(&lib, b"<?php echo \"x\";").unwrap();
            let src = format!(
                "<?php include '{p}'; include '{p}';",
                p = lib.display()
            );
            let retain = php_runtime::RetainSet::new();
            let (b, s) = execute_with_retain(&retain, &reg, &meta(&src));
            assert_eq!(s, StatusCode::OK);
            assert_eq!(b, b"xx");
            assert_eq!(
                retain.len(),
                3,
                "park-EVENTS pin: 2 include events (non-_once, same file) + 1 main = 3 (F6)"
            );
        }

        /// KL-82-2 (Council WP-82): the retained-walk shared-subgraph
        /// positive control — the walker's visited-set is prose without it.
        /// Runs HERE because php-runtime lib tests cannot link the
        /// mem-census dump (mimalloc symbols live in the bin crates).
        #[cfg(feature = "mem-census")]
        #[test]
        fn retained_walk_shared_subgraph_control() {
            php_runtime::retained_walk_selftest();
        }

        /// A-PP14 (Council WP-82) positive control: a fatal request's churn
        /// must LAND in the DRAINED_* ledger, not vanish from the
        /// denominator (Δglobal = Σlines + drained).
        #[cfg(feature = "census-instrumentation")]
        #[test]
        fn drained_ledger_books_fatal_request_churn() {
            let _guard = DRAIN_TEST_LOCK.lock().unwrap();
            let _ = php_runtime::alloc_census::SNAPSHOT_FN.set(crate::census_alloc::snapshot);
            let reg = registry();
            use std::sync::atomic::Ordering;
            let before = census::DRAINED_CALLS.load(Ordering::Relaxed);
            let r = php_runtime::RetainSet::new();
            let (_, s) = execute_with_retain(&r, &reg, &meta("<?php goto x;"));
            assert_eq!(s, StatusCode::INTERNAL_SERVER_ERROR);
            let after = census::DRAINED_CALLS.load(Ordering::Relaxed);
            assert!(
                after > before,
                "fatal request churn not booked in DRAINED_CALLS (A-PP14): {before} -> {after}"
            );
        }

        /// A-MS7/A-DL6 (KS-MS-5): shutdown() drops the senders and JOINS
        /// every worker — it returns only after real thread teardown (a hang
        /// here fails CI), with a real request drained through the pool
        /// first (positive control: the pool actually worked before dying).
        #[test]
        fn shutdown_joins_all_workers_after_drain() {
            let _guard = DEPTH_TEST_LOCK.lock().unwrap();
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
