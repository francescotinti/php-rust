//! Generator object state (step 39).
//!
//! A PHP `Generator` is the object a generator function returns. Its observable
//! state — current `(key, value)`, run status, the auto-key counter, and the
//! `getReturn()` value — lives here in [`GenState`], a plain data type owned by
//! `php-types` so it can sit inside a [`crate::Zval`] variant.
//!
//! The actual *suspendable execution* lives in `php-runtime`: the bytecode VM
//! parks the generator's frame in its own table (keyed by the generator id) and
//! resumes it on the explicit interpreter loop — no stackful coroutine and no
//! `unsafe` reborrow. `php-types` therefore holds only the plain observable state.

use crate::Zval;

/// Run status of a generator (step 39).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenStatus {
    /// Created but never advanced; the body has not started.
    NotStarted,
    /// Suspended at a `yield`, with a current `(key, value)` available.
    Suspended,
    /// The body is currently executing (guards re-entrant resume, which PHP
    /// rejects with "Cannot resume an already running generator").
    Running,
    /// The body ran to completion (or returned); `getReturn()` is available.
    Done,
}

/// The key a `yield` reports, before the driver resolves it against the
/// auto-key counter (step 39).
impl Drop for GenState {
    fn drop(&mut self) {
        // Handle freed first (the pre-existing observable order); the current
        // key/value and return value can hold arbitrarily deep object graphs,
        // so they unwind via the bounded-drop trampoline.
        crate::object::free_object_id(self.id);
        for v in [&mut self.cur_key, &mut self.cur_val, &mut self.ret] {
            let v = std::mem::replace(v, Zval::Null);
            if !matches!(v, Zval::Undef | Zval::Null | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_)) {
                crate::object::drop_bounded(crate::object::DeepDrop::Val(v));
            }
        }
    }
}

#[derive(Debug)]
pub enum GenKey {
    /// `yield $v` — take the next auto-key and bump the counter.
    Auto,
    /// `yield $k => $v` — use `$k`; an integer key `>=` the counter bumps it
    /// (mirroring array append semantics).
    Keyed(Zval),
    /// A key forwarded verbatim from a `yield from` delegate (step 39-6): used
    /// as-is and the outer auto-key counter is **not** advanced.
    Verbatim(Zval),
}

/// The observable state of a `Generator` value (step 39). Shared via
/// `Rc<RefCell<GenState>>` so a generator has object/handle semantics: assigning
/// the variable aliases the same running generator.
pub struct GenState {
    /// Per-instance handle, shown as `#N` by `var_dump` (like a closure/object).
    pub id: u32,
    /// The generator function's name (a `{closure:file:line}` synthetic name for
    /// a closure generator). Rendered by `var_dump`/`print_r` as the `function`
    /// pseudo-property (step 39-7).
    pub func_name: Box<[u8]>,
    pub status: GenStatus,
    /// `true` once the generator has been advanced past its first yielded value
    /// (any resume while already `Suspended`). `rewind()` then becomes a fatal
    /// ("Cannot rewind a generator that was already run"), step 39-7.
    pub advanced: bool,
    /// Current key at the active suspension point (NULL before start / once done).
    pub cur_key: Zval,
    /// Current value at the active suspension point.
    pub cur_val: Zval,
    /// The `return` value of the body, available via `getReturn()` once `Done`.
    pub ret: Zval,
    /// Next auto-key handed to a keyless `yield` (starts at 0).
    pub auto_key: i64,
}

impl std::fmt::Debug for GenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Show only observable state (the suspended frame lives in the VM's table).
        f.debug_struct("GenState")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("cur_key", &self.cur_key)
            .field("cur_val", &self.cur_val)
            .finish_non_exhaustive()
    }
}
