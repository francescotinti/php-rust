//! S-190 — the Ret-boundary GC probe with a CURSOR (cura O(1) v2; criterion
//! `wp190-harness/s190-criterio-o1v2.md`).
//!
//! §3.35: a holder that dies at a callee's `Ret` releases its object at
//! `leave`, not at the end of the statement. The 22/10 cure scanned the WHOLE
//! candidate buffer at every user Ret (O(|buf|) per Ret: quadratic in
//! `array_map(fn => new A, range(1, N))`, Hoare E3). The S-186 v1 scanned only
//! this Ret's own notes (`[before..)`) and lost every death noted OUTSIDE a
//! Ret — exception unwind (`exceptions::unwind`: the landing's partial stack,
//! each popped frame, `pending_throw`), the variable displaced by `CatchMatch`,
//! operand frees — which the whole scan used to catch by ACCIDENT at the next
//! unrelated Ret (bug49893 «2 1», bug47771, bug53511, assign_coalesce_002,
//! temporary_cleaning_013 are exactly those). The cursor keeps the O(1)
//! amortised cost AND the 22/10 bytes: `gc_probe_mark` is the buffer length at
//! the last probe that actually scanned, and a Ret examines everything buffered
//! since — `gc_buf[mark..len)`.
//!
//! Invariant (completeness for HOOKED releases): let E sit at position
//! p < mark when Ret R probes. p < mark means E's last push/move happened
//! before the previous scan P, where E had count ≠ 2. Between P and R, E can
//! reach count 2 only because a holder released it, and every hooked release
//! goes through `gc_note_slow`, which MOVES a buffered entry to the tail
//! (p ≥ len(P) = mark): contradiction. What the window misses is exactly the
//! UNHOOKED drops (a temporary consumed off the operand stack: §3.35 form (2)),
//! which the end-of-statement `Op::Sweep` already owns. Buffer positions never
//! compact outside the drains (a spent slot is `None`), and every drain resets
//! the cursor (`gc_buf_reset`, plus the two in-place drains in mod.rs).
//!
//! Why no probe inside `unwind()`: it returns `Option<PhpError>` — a
//! destructor throwing inside a synchronous sweep would have no `?` channel,
//! re-entering the unwind at `ip = r.target` would land in the same `try`
//! (PHP propagates: bug53511's expected trace), and the in-flight throwable
//! parked on the stack / in `pending_throw` would be visible to the destructor
//! half-way through the landing. With the cursor, the Ret that probes already
//! has its `?` towards the caller's unwind — the same site the 22/10 used.
//!
//! Cost: the hot Ret pays one compare (`mark < len`) inline; the body below is
//! out of line (`#[inline(never)]`) so `run_loop` keeps its `bl` budget
//! (disasm gate |Δbl| ≤ 2 vs the pin, Δsp_refs ≤ +97).
use super::*;

impl<'m> Vm<'m> {
    /// Probe the candidates buffered since the last scan; when one is
    /// collectable NOW (`created` + buffer clone = 2) drain the buffer
    /// synchronously and FIFO as a LIGHT sweep. Stays out of the way of a
    /// scheduled sweep in flight (`gc_buf_head != 0`: its Sweep op re-runs
    /// after the destructor it pushed), of the cycle collector, and of a
    /// caller that is a destructor body (its sweeps are silent). The cursor
    /// advances ONLY when a scan really happened.
    #[inline(never)]
    pub(super) fn gc_ret_probe(&mut self, caller: usize) -> Result<(), PhpError> {
        if self.gc_buf_head != 0
            || self.gc_collecting
            || self.frames[caller].flags.get(FrameFlags::IN_DESTRUCTOR)
        {
            return Ok(());
        }
        let len = self.gc_buf.len();
        let mark = self.gc_probe_mark.min(len);
        let hit = self.gc_buf[mark..]
            .iter()
            .any(|s| s.as_ref().is_some_and(|o| Rc::strong_count(o) == 2));
        self.gc_probe_mark = len;
        if hit {
            self.gc_sweep_impl(None, false)?;
        }
        Ok(())
    }

    /// Drop every buffered candidate and rewind both cursors (request reset,
    /// shutdown). The sweep tail and `collect_cycles` drain in place and reset
    /// `gc_probe_mark` themselves.
    pub(super) fn gc_buf_reset(&mut self) {
        self.gc_buf.clear();
        self.gc_buf_head = 0;
        self.gc_probe_mark = 0;
    }
}
