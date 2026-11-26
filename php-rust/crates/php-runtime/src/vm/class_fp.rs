//! S-194 N2: the digest of the registered class names is kept incrementally.
//!
//! [`Vm::unit_fp`] — computed at every `include` — XOR-combines one SipHash per registered
//! `(name, id)` pair: O(classes) hashes per included file (the named aperture "unit_fp O(classi)
//! per include"). The XOR of the terms is order-independent, so it can be maintained where the
//! index changes instead of being rebuilt: every write to `Vm::class_index` goes through
//! [`Vm::class_index_put`], which folds the term in (and the replaced one out). The value is the
//! same number the loop produced: fingerprints, and therefore unit-cache behaviour, do not move.
//! Criterion: wp194-harness/s194-criterio-N.md p.4.
use super::*;

/// The term of one registered `(name, id)` pair in the digest.
pub(super) fn class_fp(name: &Vec<u8>, id: &ClassId) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut eh = std::collections::hash_map::DefaultHasher::new();
    name.hash(&mut eh);
    id.hash(&mut eh);
    eh.finish()
}

impl<'m> Vm<'m> {
    /// The ONLY writer of `class_index` after construction.
    pub(super) fn class_index_put(&mut self, key: Vec<u8>, cid: ClassId) {
        let new = class_fp(&key, &cid);
        if let Some(old) = self.class_index.get(&key) {
            self.class_index_fp ^= class_fp(&key, old);
        }
        self.class_index.insert(key, cid);
        self.class_index_fp ^= new;
    }

    /// The digest by its definition (the loop `unit_fp` used to run): debug assertion and proof arm.
    pub(super) fn class_index_fp_full(&self) -> u64 {
        self.class_index.iter().fold(0, |a, (n, i)| a ^ class_fp(n, i))
    }
}
