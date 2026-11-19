//! S-192 — `IdMap<T>`: the live-object registry as an id-indexed table
//! (criterion `wp192-harness/s192-criterio-gc.md`).
//!
//! The VM keeps every live object under its handle id. Ids are dense and
//! reused, so an ordered tree pays `O(log n)` on every `new`, every free and
//! every GC probe for nothing. This is the same map — same key→value
//! relation, same ASCENDING-id iteration and drop order — with `O(1)`
//! access, exposing only the `BTreeMap` surface the VM uses. The table never
//! keeps an empty tail, so the last slot is always the greatest key.

/// Ids are dense handles; one this large means a corrupted id, not a workload.
const MAX_ID: u32 = 1 << 28;

pub struct IdMap<T> {
    slots: Vec<Option<(u32, T)>>,
    len: usize,
}

impl<T> Default for IdMap<T> {
    fn default() -> Self {
        IdMap { slots: Vec::new(), len: 0 }
    }
}

impl<T> IdMap<T> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn get(&self, id: &u32) -> Option<&T> {
        self.slots.get(*id as usize)?.as_ref().map(|(_, v)| v)
    }

    #[inline]
    pub fn contains_key(&self, id: &u32) -> bool {
        self.get(id).is_some()
    }

    pub fn insert(&mut self, id: u32, value: T) -> Option<T> {
        assert!(id < MAX_ID, "IdMap: object id {id} out of range");
        let i = id as usize;
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, || None);
        }
        let old = self.slots[i].replace((id, value));
        if old.is_none() {
            self.len += 1;
        }
        old.map(|(_, v)| v)
    }

    pub fn remove(&mut self, id: &u32) -> Option<T> {
        let old = self.slots.get_mut(*id as usize)?.take()?;
        self.len -= 1;
        self.trim();
        Some(old.1)
    }

    /// Keep the invariant "the last slot is occupied".
    fn trim(&mut self) {
        while matches!(self.slots.last(), Some(None)) {
            self.slots.pop();
        }
    }

    pub fn clear(&mut self) {
        self.slots.clear();
        self.len = 0;
    }

    /// Entries in ascending id order.
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &T)> {
        self.slots.iter().filter_map(|s| s.as_ref().map(|(k, v)| (k, v)))
    }

    pub fn keys(&self) -> impl Iterator<Item = &u32> {
        self.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, v)| v)
    }

    pub fn last_key_value(&self) -> Option<(&u32, &T)> {
        self.slots.last()?.as_ref().map(|(k, v)| (k, v))
    }

    /// Split off every entry whose id is `>= at` (as `BTreeMap::split_off`).
    pub fn split_off(&mut self, at: &u32) -> IdMap<T> {
        let i = (*at as usize).min(self.slots.len());
        let tail = self.slots.split_off(i);
        let n = tail.iter().filter(|s| s.is_some()).count();
        self.len -= n;
        self.trim();
        // Re-base the tail on its own ids: pad the front so index == id still holds.
        let mut slots: Vec<Option<(u32, T)>> = Vec::with_capacity(i + tail.len());
        slots.resize_with(i, || None);
        slots.extend(tail);
        let mut out = IdMap { slots, len: n };
        out.trim();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn same(a: &IdMap<u64>, b: &BTreeMap<u32, u64>) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.is_empty(), b.is_empty());
        assert!(a.iter().eq(b.iter()));
        assert!(a.keys().eq(b.keys()));
        assert!(a.values().eq(b.values()));
        assert_eq!(a.last_key_value(), b.last_key_value());
    }

    /// The same deterministic pseudo-random sequence on both maps: every
    /// answer and the whole iteration must agree at every step.
    #[test]
    fn behaves_like_btreemap_on_the_surface_the_vm_uses() {
        let (mut a, mut b) = (IdMap::<u64>::new(), BTreeMap::<u32, u64>::new());
        let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
        for step in 0..20_000u64 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let id = (x % 300) as u32;
            match (x >> 32) % 10 {
                0..=4 => assert_eq!(a.insert(id, step), b.insert(id, step)),
                5..=7 => assert_eq!(a.remove(&id), b.remove(&id)),
                8 => {
                    assert_eq!(a.get(&id), b.get(&id));
                    assert_eq!(a.contains_key(&id), b.contains_key(&id));
                }
                _ => {
                    let (ta, tb) = (a.split_off(&id), b.split_off(&id));
                    same(&ta, &tb);
                }
            }
            if step % 64 == 0 {
                same(&a, &b);
            }
        }
        same(&a, &b);
        a.clear();
        b.clear();
        same(&a, &b);
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn a_corrupted_id_panics_instead_of_allocating() {
        IdMap::<u8>::new().insert(u32::MAX, 0);
    }
}
