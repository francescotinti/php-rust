//! S-194 T1: a small memo of class type checks that already succeeded.
//!
//! `Vm::value_satisfies_class` — the `instanceof` behind every class-typed parameter, property
//! and return — compares the object's class name with the declared one case-insensitively and,
//! for a subclass or an interface, lowercases the name, looks it up in the class index and asks
//! the ancestry memo: ~1.2 % of an ORM run. A `(class id, declared name)` pair that was once
//! satisfied stays satisfied for the life of the run (class ids, their ancestry and a declared
//! name's binding never change), so the pair is remembered in a direct-mapped table and a repeat
//! costs one index, one integer compare and one byte compare. Only successes are remembered: a
//! name that is not declared yet may be declared later. `static` is never asked here.
//! Criterion: wp194-harness/s194-criterio-N.md p.5.

const SLOTS: usize = 1024;

pub(super) struct HintMemo(Box<[(u32, Box<[u8]>)]>);

impl Default for HintMemo {
    fn default() -> Self {
        HintMemo((0..SLOTS).map(|_| (0u32, Box::default())).collect())
    }
}

impl HintMemo {
    #[inline]
    fn idx(cid: u32, name: &[u8]) -> usize {
        (cid as usize).wrapping_mul(31).wrapping_add(name.len()).wrapping_add(name.last().copied().unwrap_or(0) as usize * 7) & (SLOTS - 1)
    }

    /// Whether `(cid, name)` is a remembered success. `cid + 1` is stored, 0 marks an empty slot.
    #[inline]
    pub(super) fn hit(&self, cid: u32, name: &[u8]) -> bool {
        let e = &self.0[Self::idx(cid, name)];
        e.0 == cid.wrapping_add(1) && *e.1 == *name
    }

    pub(super) fn put(&mut self, cid: u32, name: &[u8]) {
        self.0[Self::idx(cid, name)] = (cid.wrapping_add(1), name.into());
    }

    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
}
