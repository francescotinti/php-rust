//! Generator object state (step 39).
//!
//! A PHP `Generator` is the object a generator function returns. Its observable
//! state — current `(key, value)`, run status, the auto-key counter, and the
//! `getReturn()` value — lives here in [`GenState`], a plain data type owned by
//! `php-types` so it can sit inside a [`crate::Zval`] variant.
//!
//! The actual *suspendable execution* (a stackful coroutine) lives behind the
//! type-erased [`GenDriver`] trait, implemented in `php-runtime` (which owns the
//! interpreter and the coroutine engine). `php-types` never names the evaluator
//! or the coroutine crate: the driver receives a type-erased `*mut ()` evaluator
//! pointer and hands back a [`GenStep`]. This keeps the crate layering intact
//! (`php-types` depends on nothing interpreter-specific).

use crate::{PhpError, Zval};

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
#[derive(Debug, Clone)]
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

/// One step of generator execution, returned by [`GenDriver::resume`].
pub enum GenStep {
    /// The body suspended at a `yield`, producing this key/value.
    Yielded { key: GenKey, value: Zval },
    /// The body finished: `Ok` carries the `return` value (for `getReturn()`),
    /// `Err` an exception that unwound out of the generator (surfaces at the
    /// advancing call site).
    Returned(Result<Zval, PhpError>),
}

/// Type-erased handle to a generator's suspendable execution. Implemented in
/// `php-runtime`; the `ev` pointer is a lifetime-and-type-erased `*mut Evaluator`
/// reborrowed inside. **Invariant:** never resume a generator that is already
/// `Running` (the [`GenState::status`] guard enforces this), which also upholds
/// the soundness of the reborrow (no aliasing of the evaluator).
pub trait GenDriver {
    fn resume(&mut self, sent: Zval, ev: *mut ()) -> GenStep;
}

/// The observable state of a `Generator` value (step 39). Shared via
/// `Rc<RefCell<GenState>>` so a generator has object/handle semantics: assigning
/// the variable aliases the same running generator.
pub struct GenState {
    /// Per-instance handle, shown as `#N` by `var_dump` (like a closure/object).
    pub id: u32,
    pub status: GenStatus,
    /// Current key at the active suspension point (NULL before start / once done).
    pub cur_key: Zval,
    /// Current value at the active suspension point.
    pub cur_val: Zval,
    /// The `return` value of the body, available via `getReturn()` once `Done`.
    pub ret: Zval,
    /// Next auto-key handed to a keyless `yield` (starts at 0).
    pub auto_key: i64,
    /// The suspendable body. `Some` until the generator finishes, then dropped
    /// (freeing the coroutine stack).
    pub driver: Option<Box<dyn GenDriver>>,
}

impl std::fmt::Debug for GenState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The driver holds a coroutine (not `Debug`); show only observable state.
        f.debug_struct("GenState")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("cur_key", &self.cur_key)
            .field("cur_val", &self.cur_val)
            .finish_non_exhaustive()
    }
}
