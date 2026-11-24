//! S-194 C3b: the scope-keyed entry of a method call site's cache ([`MethodIc`]).
//!
//! A site caches `(receiver class) -> (defining class, method index)` only when that outcome is
//! the same for every calling scope (a public winner with no private homonym up the chain). Every
//! other resolution — a private or protected method, a private rebind — is a pure function of the
//! class tables, the receiver class, the method name AND the calling scope, so it can be cached
//! with the scope in the key. A scoped entry carries the high bit in its epoch: [`MethodIc::get`],
//! the hot hit of public methods, never matches it and stays as it was; only [`MethodIc::get_scoped`]
//! reads it, and the callers ask it after the public miss.
//! Criterion: wp194-harness/s194-criterio-C.md p.23.
use crate::bytecode::{ic_epoch, MethodIc};

const SCOPED: u64 = 1 << 63;

#[inline]
fn scope_key(scope: Option<usize>) -> u32 {
    scope.map_or(0, |s| s as u32 + 1)
}

impl MethodIc {
    /// The cached `(defining ClassId, method idx)` when filled in this run for exactly this
    /// receiver class called from exactly this scope.
    #[inline]
    pub(crate) fn get_scoped(&self, cid: usize, scope: Option<usize>) -> Option<(usize, usize)> {
        let (epoch, cid1, defc, midx, s1) = self.0.get();
        (cid1 as usize == cid + 1 && epoch == (ic_epoch() | SCOPED) && s1 == scope_key(scope))
            .then_some((defc as usize, midx as usize))
    }

    #[inline]
    pub(crate) fn fill_scoped(&self, cid: usize, scope: Option<usize>, defc: usize, midx: usize) {
        self.0.set((ic_epoch() | SCOPED, cid as u32 + 1, defc as u32, midx as u32, scope_key(scope)));
    }
}
