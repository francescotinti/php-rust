use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

/// A PHP string: an arbitrary byte sequence (never assumed UTF-8).
///
/// Mirrors `zend_string` (Zend/zend_types.h:393-398): lazy hash with 0 meaning
/// "not yet computed", same convention as ZSTR_H (Zend/zend_string.h:114).
pub struct PhpStr {
    hash: Cell<u64>,
    bytes: Box<[u8]>,
}

pub type ZStr = Rc<PhpStr>;

impl PhpStr {
    pub fn new(bytes: impl Into<Box<[u8]>>) -> ZStr {
        Rc::new(PhpStr {
            hash: Cell::new(0),
            bytes: bytes.into(),
        })
    }

    pub fn from_str(s: &str) -> ZStr {
        Self::new(s.as_bytes().to_vec())
    }

    pub fn empty() -> ZStr {
        Self::new(Vec::new())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// DJBX33A, same algorithm as zend_inline_hash_func. Lazily cached.
    /// Not observable in program output; kept identical to ease cross-debugging.
    pub fn zhash(&self) -> u64 {
        let h = self.hash.get();
        if h != 0 {
            return h;
        }
        let mut hash: u64 = 5381;
        for &b in self.bytes.iter() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        // Mirror Zend: force the "computed" bit so a result of 0 is impossible.
        let hash = hash | 0x8000_0000_0000_0000;
        self.hash.set(hash);
        hash
    }
}

impl PartialEq for PhpStr {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for PhpStr {}

impl std::hash::Hash for PhpStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl fmt::Debug for PhpStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhpStr({:?})", String::from_utf8_lossy(&self.bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_safe() {
        let s = PhpStr::new(vec![0u8, 255, 1]);
        assert_eq!(s.len(), 3);
        assert_eq!(s.as_bytes(), &[0, 255, 1]);
    }

    #[test]
    fn eq_by_content() {
        assert_eq!(*PhpStr::from_str("abc"), *PhpStr::from_str("abc"));
        assert_ne!(*PhpStr::from_str("abc"), *PhpStr::from_str("abd"));
    }

    #[test]
    fn hash_cached_and_nonzero() {
        let s = PhpStr::empty();
        let h = s.zhash();
        assert_ne!(h, 0);
        assert_eq!(s.zhash(), h);
    }
}
