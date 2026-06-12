use std::collections::HashMap;
use std::rc::Rc;

use crate::{PhpStr, Zval};

/// An array key: PHP arrays have dual int|string keys.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Key {
    Int(i64),
    Str(Rc<PhpStr>),
}

impl Key {
    /// Canonicalize a string key, mirroring `_zend_handle_numeric_str_ex`
    /// (Zend/zend_hash.c:3300): optional '-', digits only, no leading zeros
    /// ("08" stays string, "-0" stays string), max 19 digits, no i64 overflow.
    pub fn from_bytes(bytes: &[u8]) -> Key {
        match canonical_int_key(bytes) {
            Some(i) => Key::Int(i),
            None => Key::Str(PhpStr::new(bytes.to_vec())),
        }
    }

    pub fn from_zstr(s: &Rc<PhpStr>) -> Key {
        match canonical_int_key(s.as_bytes()) {
            Some(i) => Key::Int(i),
            None => Key::Str(Rc::clone(s)),
        }
    }
}

/// Returns Some(i) when `bytes` is the canonical decimal form of an i64.
fn canonical_int_key(bytes: &[u8]) -> Option<i64> {
    let (neg, digits) = match bytes.split_first()? {
        (b'-', rest) => (true, rest),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return None;
    }
    // Leading zeros: "0" alone is valid, "0..." with more chars is not;
    // "-0" is not (sign consumed, '0' with the original length > 1).
    if digits[0] == b'0' && bytes.len() > 1 {
        return None;
    }
    // MAX_LENGTH_OF_LONG - 1 = 19 digits on 64-bit (Zend/zend_long.h:112).
    if digits.len() > 19 {
        return None;
    }
    let mut idx: u64 = 0;
    for &b in digits {
        if !b.is_ascii_digit() {
            return None;
        }
        idx = idx * 10 + (b - b'0') as u64;
    }
    if neg {
        // Allow down to i64::MIN: idx - 1 must not exceed i64::MAX.
        if idx.wrapping_sub(1) > i64::MAX as u64 {
            return None;
        }
        Some((idx as i64).wrapping_neg())
    } else {
        if idx > i64::MAX as u64 {
            return None;
        }
        Some(idx as i64)
    }
}

/// Appending past the max int key fails in PHP with
/// "Cannot add element to the array as the next element is already occupied".
#[derive(Debug, PartialEq, Eq)]
pub struct ArrayAppendError;

/// A PHP array: an insertion-ordered hash with int|string keys.
///
/// Mirrors the observable semantics of `zend_array` (Zend/zend_types.h:408-432):
/// iteration order is insertion order (survives unset), tombstones like Zend's
/// IS_UNDEF buckets, `next_free` never decreases on unset. The internal
/// packed/mixed distinction of Zend is invisible and not reproduced.
#[derive(Clone, Debug)]
pub struct PhpArray {
    entries: Vec<Option<(Key, Zval)>>,
    index: HashMap<Key, u32>,
    /// Mirrors nNextFreeElement: starts at i64::MIN (zend_hash.c:257),
    /// "MIN means empty-append uses 0" (zend_hash.c:1099), saturates at
    /// i64::MAX (zend_hash.c:1183), never decreases on unset.
    next_free: i64,
    count: u32,
}

impl Default for PhpArray {
    fn default() -> Self {
        PhpArray {
            entries: Vec::new(),
            index: HashMap::new(),
            next_free: i64::MIN,
            count: 0,
        }
    }
}

impl PhpArray {
    pub fn new() -> PhpArray {
        PhpArray::default()
    }

    pub fn len(&self) -> usize {
        self.count as usize
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Insert or update. Updating an existing key keeps its position.
    pub fn insert(&mut self, key: Key, val: Zval) {
        if let Some(&pos) = self.index.get(&key) {
            self.entries[pos as usize] = Some((key, val));
            return;
        }
        if let Key::Int(i) = key {
            if i >= self.next_free {
                self.next_free = i.checked_add(1).unwrap_or(i64::MAX);
            }
        }
        let pos = self.entries.len() as u32;
        self.index.insert(key.clone(), pos);
        self.entries.push(Some((key, val)));
        self.count += 1;
    }

    /// `$a[] = v`: uses the next free int index, which never decreases.
    /// Fails only when that slot is occupied (possible after saturation at
    /// i64::MAX), matching Zend's "next element is already occupied" error.
    pub fn append(&mut self, val: Zval) -> Result<(), ArrayAppendError> {
        let h = if self.next_free == i64::MIN { 0 } else { self.next_free };
        if self.contains_key(&Key::Int(h)) {
            return Err(ArrayAppendError);
        }
        self.insert(Key::Int(h), val);
        Ok(())
    }

    pub fn get(&self, key: &Key) -> Option<&Zval> {
        self.index
            .get(key)
            .map(|&pos| &self.entries[pos as usize].as_ref().unwrap().1)
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Zval> {
        match self.index.get(key) {
            Some(&pos) => Some(&mut self.entries[pos as usize].as_mut().unwrap().1),
            None => None,
        }
    }

    pub fn contains_key(&self, key: &Key) -> bool {
        self.index.contains_key(key)
    }

    /// `unset($a[k])`: leaves a tombstone so iteration order is preserved.
    /// `next_free` intentionally not touched (Zend semantics).
    pub fn remove(&mut self, key: &Key) -> Option<Zval> {
        let pos = self.index.remove(key)?;
        let (_, val) = self.entries[pos as usize].take().unwrap();
        self.count -= 1;
        if self.entries.len() >= 8 && (self.count as usize) < self.entries.len() / 2 {
            self.compact();
        }
        Some(val)
    }

    fn compact(&mut self) {
        self.entries.retain(Option::is_some);
        self.index.clear();
        for (pos, entry) in self.entries.iter().enumerate() {
            let (key, _) = entry.as_ref().unwrap();
            self.index.insert(key.clone(), pos as u32);
        }
    }

    /// Iterate in insertion order, skipping tombstones.
    pub fn iter(&self) -> impl Iterator<Item = (&Key, &Zval)> {
        self.entries
            .iter()
            .filter_map(|e| e.as_ref().map(|(k, v)| (k, v)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Key, &mut Zval)> {
        self.entries
            .iter_mut()
            .filter_map(|e| e.as_mut().map(|(k, v)| (&*k, v)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(s: &str) -> Key {
        Key::from_bytes(s.as_bytes())
    }

    #[test]
    fn key_canonicalization() {
        assert_eq!(k("8"), Key::Int(8));
        assert_eq!(k("0"), Key::Int(0));
        assert_eq!(k("-5"), Key::Int(-5));
        assert_eq!(k("9223372036854775807"), Key::Int(i64::MAX));
        assert_eq!(k("-9223372036854775808"), Key::Int(i64::MIN));
        // These all stay strings:
        for s in ["08", "-0", "1.5", "0x1A", "1e3", " 1", "9223372036854775808",
                  "-9223372036854775809", "12345678901234567890", ""] {
            assert!(matches!(k(s), Key::Str(_)), "{s:?} should stay a string key");
        }
    }

    #[test]
    fn string_and_int_keys_collide_when_canonical() {
        let mut a = PhpArray::new();
        a.insert(k("8"), Zval::Long(1));
        assert_eq!(a.len(), 1);
        assert!(a.contains_key(&Key::Int(8)));
        // "08" is a distinct (string) key.
        a.insert(k("08"), Zval::Long(2));
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn insertion_order_survives_update_and_unset() {
        let mut a = PhpArray::new();
        a.insert(Key::Int(0), Zval::Long(10));
        a.insert(k("x"), Zval::Long(20));
        a.insert(Key::Int(1), Zval::Long(30));
        a.insert(Key::Int(0), Zval::Long(99)); // update keeps position
        a.remove(&k("x"));
        let keys: Vec<_> = a.iter().map(|(key, _)| key.clone()).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(1)]);
        match a.get(&Key::Int(0)) {
            Some(Zval::Long(99)) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn next_free_never_decreases() {
        let mut a = PhpArray::new();
        a.append(Zval::Long(1)).unwrap(); // [0]
        a.append(Zval::Long(2)).unwrap(); // [1]
        a.remove(&Key::Int(1));
        a.append(Zval::Long(3)).unwrap(); // [2], NOT [1]
        assert!(a.contains_key(&Key::Int(2)));
        assert!(!a.contains_key(&Key::Int(1)));
    }

    #[test]
    fn next_free_follows_max_inserted() {
        let mut a = PhpArray::new();
        a.insert(Key::Int(100), Zval::Null);
        a.append(Zval::Null).unwrap();
        assert!(a.contains_key(&Key::Int(101)));
        // Negative keys don't raise next_free below 0 usage:
        let mut b = PhpArray::new();
        b.insert(Key::Int(-5), Zval::Null);
        b.append(Zval::Null).unwrap();
        // PHP: next free after key -5 is -4? No: nNextFreeElement = -5+1 = -4.
        assert!(b.contains_key(&Key::Int(-4)));
    }

    #[test]
    fn append_after_max_key_fails() {
        let mut a = PhpArray::new();
        a.insert(Key::Int(i64::MAX), Zval::Null);
        assert_eq!(a.append(Zval::Null), Err(ArrayAppendError));
        // ...but unsetting MAX frees the (saturated) slot again, like Zend.
        a.remove(&Key::Int(i64::MAX));
        assert!(a.append(Zval::Null).is_ok());
        assert!(a.contains_key(&Key::Int(i64::MAX)));
    }

    #[test]
    fn compaction_preserves_order_and_lookups() {
        let mut a = PhpArray::new();
        for i in 0..20 {
            a.insert(Key::Int(i), Zval::Long(i));
        }
        for i in 0..15 {
            a.remove(&Key::Int(i));
        }
        let keys: Vec<_> = a.iter().map(|(key, _)| key.clone()).collect();
        assert_eq!(
            keys,
            (15..20).map(Key::Int).collect::<Vec<_>>()
        );
        assert!(matches!(a.get(&Key::Int(17)), Some(Zval::Long(17))));
    }
}
