//! S-195 S2 — the release note of an OBJECT, moved out of `gc_note_slow` verbatim
//! (criterion wp195-harness/s195-criterio-sweep.md). The object is the common
//! species of a note; in its own function it pays a small frame instead of the
//! ten saved registers the recursive array/closure arms need. Behaviour is the
//! old arm's, line for line.
use super::Vm;
use php_types::Object;
use std::cell::RefCell;
use std::rc::Rc;

impl<'m> Vm<'m> {
    #[inline(never)]
    pub(super) fn gc_note_obj(&mut self, rc: &Rc<RefCell<Object>>) {
        // One borrow serves the whole note: the DESTRUCTED mirror
        // flag replaces the id-set probe (the set stays authoritative
        // everywhere else) and the buffer insert is inlined
        // (gc_buf_push would re-borrow).
        let b = rc.borrow();
        if b.gc.destructed() {
            return;
        }
        // Always buffer (no note-time classification): the sweep at
        // the END of this statement re-checks the live count, which is
        // the safety net for *unhooked* drops that happen later in the
        // same statement (a temp consumed off the operand stack is not
        // gc_note'd — see `gc_light_demoted`). A note-time `count > 2`
        // shortcut straight to the cycle-roots buffer loses exactly
        // that window: a destructor-driven cleanup (e.g. a file
        // unlink) can slip past the tests that depend on it (WP-21:
        // REST sideload unique-filename flake).
        if !b.gc.buffered() {
            b.gc.set_pos(self.gc_buf.len());
            b.gc.set_birth(false);
            drop(b);
            self.gc_buf.push(Some(Rc::clone(rc)));
            self.gc_idle_set([false; 2]);
            #[cfg(feature = "gc-census")]
            super::gc_census::note_inserted();
        } else {
            // Already buffered (birth seed, or an earlier note): this
            // is a LATER release, so the entry moves to the tail — the
            // sweep pops FIFO and Zend frees in release order. §3.35
            // form 3: `f(new A, new B)` dies A then B at leave, but
            // B's constructor-sweep demotion and `$this` re-note had
            // queued B ahead of A. The BIRTH mark is kept as it is
            // (a release note never flips it — `gc_buf_push`).
            // S-190: skipping this move for a live object (count > 3) was tried and RETIRED — the FIFO
            // position is order, not just liveness (sentinel iters_this_dynvars: a1/a2 and x2/x3 swapped).
            // (iv-b): already the TAIL slot ⇒ the move is the identity (no hole, no push); only the probe
            // window re-opens on it so the next `Ret` still sees this release (cursor invariant).
            if self.gc_buf.last().is_some_and(|s| s.as_ref().is_some_and(|l| Rc::ptr_eq(l, rc))) {
                drop(b);
                let p = self.gc_buf.len() - 1;
                if p >= self.gc_buf_head && self.gc_probe_mark > p {
                    self.gc_probe_mark = p;
                }
                return;
            }
            let pos = b.gc.pos();
            drop(b);
            if let Some(p) = pos.filter(|&p| p >= self.gc_buf_head) {
                if let Some(entry) = self.gc_buf[p].take() {
                    entry.borrow().gc.set_pos(self.gc_buf.len());
                    self.gc_buf.push(Some(entry));
                }
            }
        }
    }
}
