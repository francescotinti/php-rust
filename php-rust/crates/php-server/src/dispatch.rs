//! S-189 (Council `wp189-harness/concilio/COUNCIL_WP189_REVIEWS.md`, signed
//! 2026-11-03): the SHARED FIFO work queue of the axum worker pool.
//!
//! Why: per-worker queues + blind round-robin let a blocking loopback request
//! (WordPress Site Health, theme/plugin editor, auto-updater) land in the queue
//! of the very worker that is waiting for it — with L sequential loopbacks
//! from one request, floor(L/W) deterministic stalls on an idle server
//! (PHPR_DIVERGENCES §3.9 (h), tooth `wp189-harness/gate-loopback`). `php -S`
//! with `PHP_CLI_SERVER_WORKERS` (N children + the master) and php-fpm accept
//! on ONE shared socket: an idle process takes the next connection, a busy one
//! never does. This queue is the userland transcript of that backlog: FIFO of
//! DISPATCH (not of accept — two handlers may race to `push`, declared), any
//! parked worker takes the next task, a busy worker is never assigned work.
//! W=1 still stalls, exactly like `php -S` at one process (kept by name).
//!
//! Form (team-meccanismo E1, Hoare R1/R4/R9, Matsakis R2/R5, Hejlsberg R1-R3,
//! Bak R11): std only — the workers are plain OS threads without a tokio
//! handle; no new crate. Lost and spurious wakeups are excluded by waiting on
//! the PREDICATE under the lock (`Condvar::wait_while`), which also makes the
//! "woken consumer finds the queue already emptied by a non-parked worker"
//! case a harmless re-wait. The critical section holds nothing but VecDeque
//! operations and the `closed` flag (never a send, a tripwire, a census line
//! or php_runtime), so a poisoned lock — unreachable in production: a worker
//! panic aborts the process, A-PP9 — is recovered with `into_inner` without
//! exposing torn state. API by value only: no peek, no guard escapes, no
//! `len()` (QUEUE_DEPTH stays the ONLY census source, A-TH1). `push` after
//! `close` hands the task back (the caller undoes its census increments and
//! answers 500); `pop_blocking` drains what is queued, then returns None
//! (drain-then-exit, A-MS7). "Exactly one worker per task" is a property of
//! OWNERSHIP now (`pop_front` moves T out under mutual exclusion and
//! `WorkerTask` is not `Clone` — sealed in worker_pool.rs) instead of one
//! receiver per worker. Wakeup order is whatever the OS picks (`notify_one`):
//! the per-thread dispatch map at W>1 is not verdict-grade (A-PP37/A-PP42 as
//! amended by S-189); only the exact row set {0..W-1} and Σcount == nreq +
//! orphan are.

use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, PoisonError};

struct State<T> {
    q: VecDeque<T>,
    closed: bool,
}

/// One FIFO shared by every worker of a pool. `T: Send` is the whole proof:
/// `Mutex<State<T>>: Sync ⇔ T: Send`, so `Arc<SharedQueue<T>>` travels to the
/// worker threads iff the payload may.
pub(crate) struct SharedQueue<T> {
    inner: Mutex<State<T>>,
    cv: Condvar,
}

// Negative control (WP-72 lesson, Matsakis R2): a non-Send payload must make
// the queue unusable across threads — an all-positive assertion set proves
// nothing. The positive twin lives next to `WorkerTask` in worker_pool.rs.
static_assertions::assert_not_impl_any!(SharedQueue<std::rc::Rc<()>>: Send, Sync);

impl<T: Send> SharedQueue<T> {
    /// `capacity` = expected peak queue length (W is the sane default); the
    /// deque never shrinks — a stable capacity is a stable `resid` at W=1
    /// (A-BB11, Leijen R2).
    pub(crate) fn new(capacity: usize) -> Arc<Self> {
        Arc::new(SharedQueue {
            inner: Mutex::new(State { q: VecDeque::with_capacity(capacity), closed: false }),
            cv: Condvar::new(),
        })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State<T>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Enqueue. `Err(t)` iff the queue is closed — no slot is consumed, no
    /// ghost (the A-PP37 round-robin ghost died with the index). The notify
    /// happens AFTER the lock is released so the woken consumer does not
    /// immediately park again on the mutex we still hold (Bak R11).
    pub(crate) fn push(&self, t: T) -> Result<(), T> {
        {
            let mut g = self.lock();
            if g.closed {
                return Err(t);
            }
            g.q.push_back(t);
        }
        self.cv.notify_one();
        Ok(())
    }

    /// Block until a task is available or the queue is closed AND empty.
    /// Worker threads only — never from a tokio task (it would park the
    /// runtime). Returns the task by value with the lock already released.
    pub(crate) fn pop_blocking(&self) -> Option<T> {
        let g = self.lock();
        let mut g = self
            .cv
            .wait_while(g, |s| s.q.is_empty() && !s.closed)
            .unwrap_or_else(PoisonError::into_inner);
        g.q.pop_front()
    }

    /// Close: every parked consumer wakes; tasks already queued are still
    /// drained before the consumers see `None`.
    pub(crate) fn close(&self) {
        self.lock().closed = true;
        self.cv.notify_all();
    }
}

/// S-189 (signed Q2): rendezvous of the `__census_self` gang — the W tasks of
/// ONE census call must each be taken by a DISTINCT worker (pigeonhole: a
/// worker parked here cannot pop a second task). `std::sync::Barrier` is
/// banned (no timeout: a worker busy on a loopback would wedge the pool in
/// silence, KS-PP-6); this latch gives up after `timeout` and the caller
/// aborts with attribution to the INSTRUMENT (A-MS31). Per-pool (never a
/// process static: the test binary builds several pools), sized from the
/// EFFECTIVE worker count. The generation resets on release so later census
/// calls start clean; a timed-out generation is left as is (the process is
/// aborting).
#[cfg(feature = "mem-census")]
pub(crate) struct CensusLatch {
    n: usize,
    inner: Mutex<(usize, u64)>, // (arrived in this generation, generation)
    cv: Condvar,
}

#[cfg(feature = "mem-census")]
impl CensusLatch {
    pub(crate) fn new(n: usize) -> Self {
        CensusLatch { n, inner: Mutex::new((0, 0)), cv: Condvar::new() }
    }

    pub(crate) fn size(&self) -> usize {
        self.n
    }

    /// `Ok(())` when all `n` participants arrived within `timeout`;
    /// `Err(arrived)` when the window closed first.
    pub(crate) fn wait(&self, timeout: std::time::Duration) -> Result<(), usize> {
        let mut g = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        g.0 += 1;
        let generation = g.1;
        if g.0 >= self.n {
            g.0 = 0;
            g.1 += 1;
            drop(g);
            self.cv.notify_all();
            return Ok(());
        }
        let (g, _) = self
            .cv
            .wait_timeout_while(g, timeout, |s| s.1 == generation)
            .unwrap_or_else(PoisonError::into_inner);
        if g.1 == generation { Err(g.0) } else { Ok(()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    const BOUND: Duration = Duration::from_secs(10);

    /// N producers × M consumers: every id consumed exactly once AND the total
    /// equals N·k (KS-TM-4: both halves, never one). close() after the
    /// producers is what lets the consumers exit — the join proves wake-all.
    #[test]
    fn n_producers_m_consumers_each_once() {
        const P: u64 = 4;
        const K: u64 = 250;
        let q = SharedQueue::<u64>::new(8);
        let got = Arc::new(Mutex::new(Vec::new()));
        let consumers: Vec<_> = (0..3)
            .map(|_| {
                let q = Arc::clone(&q);
                let got = Arc::clone(&got);
                std::thread::spawn(move || {
                    while let Some(v) = q.pop_blocking() {
                        got.lock().unwrap().push(v);
                    }
                })
            })
            .collect();
        let producers: Vec<_> = (0..P)
            .map(|p| {
                let q = Arc::clone(&q);
                std::thread::spawn(move || {
                    for i in 0..K {
                        q.push(p * K + i).unwrap();
                    }
                })
            })
            .collect();
        for h in producers {
            h.join().unwrap();
        }
        q.close();
        for h in consumers {
            h.join().unwrap();
        }
        let mut v = got.lock().unwrap().clone();
        v.sort_unstable();
        assert_eq!(v.len() as u64, P * K, "total consumed != produced");
        assert_eq!(v, (0..P * K).collect::<Vec<_>>(), "some id missing or duplicated");
    }

    /// Drain-then-exit: K queued at close() ⇒ K Some (FIFO order) then None,
    /// within a time bound (a hang here is the CI failure, like 2029).
    #[test]
    fn close_with_queued_tasks_drains_then_none() {
        let q = SharedQueue::<u32>::new(4);
        for i in 0..3 {
            q.push(i).unwrap();
        }
        q.close();
        let (tx, rx) = mpsc::channel();
        let qq = Arc::clone(&q);
        std::thread::spawn(move || {
            let seq: Vec<Option<u32>> = (0..4).map(|_| qq.pop_blocking()).collect();
            tx.send(seq).unwrap();
        });
        let seq = rx.recv_timeout(BOUND).expect("drain did not finish within the bound");
        assert_eq!(seq, vec![Some(0), Some(1), Some(2), None]);
    }

    #[test]
    fn push_after_close_returns_the_task() {
        let q = SharedQueue::<u32>::new(1);
        q.close();
        assert_eq!(q.push(7), Err(7));
        assert_eq!(q.pop_blocking(), None);
    }

    /// close() must wake a consumer that is PARKED on the condvar (the
    /// `drop(senders)` equivalence, Hoare R3). No sleep: the consumer
    /// announces "about to park" first; whichever interleaving follows,
    /// pop_blocking must return None within the bound.
    #[test]
    fn close_wakes_parked_consumer() {
        let q = SharedQueue::<u32>::new(1);
        let (announce_tx, announce_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let qq = Arc::clone(&q);
        std::thread::spawn(move || {
            announce_tx.send(()).unwrap();
            done_tx.send(qq.pop_blocking()).unwrap();
        });
        announce_rx.recv_timeout(BOUND).unwrap();
        q.close();
        assert_eq!(done_rx.recv_timeout(BOUND).expect("parked consumer never woke"), None);
    }

    /// Lost-wakeup control (Leijen R9a): a push with NO consumer waiting must
    /// still be found by a consumer that arrives later — the predicate is
    /// re-checked under the lock before every wait.
    #[test]
    fn push_before_consumer_is_not_lost() {
        let q = SharedQueue::<u32>::new(1);
        q.push(42).unwrap();
        let (tx, rx) = mpsc::channel();
        let qq = Arc::clone(&q);
        std::thread::spawn(move || tx.send(qq.pop_blocking()).unwrap());
        assert_eq!(rx.recv_timeout(BOUND).unwrap(), Some(42));
    }

    /// Exclusivity (Leijen R9b, the property the census latch relies on): W
    /// consumers each holding one task ⇒ the (W+1)-th task stays queued until
    /// one of them is free; then exactly one takes it.
    #[test]
    fn busy_consumers_leave_the_extra_task_queued() {
        const W: usize = 3;
        let q = SharedQueue::<u32>::new(W);
        let (got_tx, got_rx) = mpsc::channel::<u32>();
        let mut holds = Vec::new();
        let mut handles = Vec::new();
        for _ in 0..W {
            let (hold_tx, hold_rx) = mpsc::channel::<()>();
            holds.push(hold_tx);
            let q = Arc::clone(&q);
            let got_tx = got_tx.clone();
            handles.push(std::thread::spawn(move || {
                // first task: take it and HOLD (busy worker)
                let v = q.pop_blocking().unwrap();
                got_tx.send(v).unwrap();
                hold_rx.recv().unwrap();
                // free again: keep draining until close
                while let Some(v) = q.pop_blocking() {
                    got_tx.send(v).unwrap();
                }
            }));
        }
        drop(got_tx);
        for i in 0..W as u32 {
            q.push(i).unwrap();
        }
        let mut first: Vec<u32> = (0..W).map(|_| got_rx.recv_timeout(BOUND).unwrap()).collect();
        first.sort_unstable();
        assert_eq!(first, vec![0, 1, 2]);
        q.push(99).unwrap();
        // every consumer is busy: nobody may take the 4th task (bounded negative wait)
        assert!(
            got_rx.recv_timeout(Duration::from_millis(300)).is_err(),
            "a busy consumer took a second task"
        );
        for h in holds {
            h.send(()).unwrap();
        }
        assert_eq!(got_rx.recv_timeout(BOUND).unwrap(), 99);
        q.close();
        for h in handles {
            h.join().unwrap();
        }
    }

    /// W participants ⇒ all released; Err(arrived) when the window closes
    /// first (the probe aborts, never a silent wedge).
    #[cfg(feature = "mem-census")]
    #[test]
    fn census_latch_releases_all_and_times_out_when_short() {
        let latch = Arc::new(CensusLatch::new(3));
        let (tx, rx) = mpsc::channel();
        for _ in 0..3 {
            let l = Arc::clone(&latch);
            let tx = tx.clone();
            std::thread::spawn(move || tx.send(l.wait(BOUND)).unwrap());
        }
        for _ in 0..3 {
            assert_eq!(rx.recv_timeout(BOUND).unwrap(), Ok(()));
        }
        // generation reset: the latch is reusable
        let short = CensusLatch::new(2);
        assert_eq!(short.wait(Duration::from_millis(100)), Err(1));
        assert_eq!(latch.size(), 3);
    }
}
