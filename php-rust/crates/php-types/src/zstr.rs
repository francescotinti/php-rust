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
        let bytes = bytes.into();
        #[cfg(feature = "str-census")]
        census::record(bytes.len());
        Rc::new(PhpStr {
            hash: Cell::new(0),
            bytes,
        })
    }

    #[allow(clippy::should_implement_trait)] // infallible byte view, not FromStr
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
        // WP-29 B4: feed the CACHED per-string hash (zend_string->h
        // semantics) instead of re-hashing the bytes — an array-key string
        // hashes once in its lifetime, not on every PhpArray/HashMap
        // insert/lookup. Eq stays byte-based, and equal bytes yield equal
        // zhash, so the Hash/Eq contract holds.
        state.write_u64(self.zhash());
    }
}

impl fmt::Debug for PhpStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhpStr({:?})", String::from_utf8_lossy(&self.bytes))
    }
}

/// WP-37 attribution counters (SSO groundwork, WP-26 lesson: measure BEFORE
/// the refactor). Every `PhpStr::new` records its length into a bucket; the
/// histogram is APPENDED to `$PHPR_STR_CENSUS` at process exit (`libc::atexit`
/// — fires on `process::exit` too, and append-mode aggregates phpr
/// subprocesses like the op-census file dump). Buckets are chosen around the
/// candidate inline capacities of an SSO PhpStr (payload that fits alongside
/// `len` without growing the struct beyond the current 24B heap-repr).
#[cfg(feature = "str-census")]
mod census {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

    /// Upper bounds of each bucket, inclusive; the last is a catch-all.
    pub const BOUNDS: [usize; 8] = [0, 7, 15, 23, 31, 63, 255, usize::MAX];
    static COUNTS: [AtomicU64; 8] = [const { AtomicU64::new(0) }; 8];
    static BYTES: AtomicU64 = AtomicU64::new(0);
    static REGISTERED: AtomicBool = AtomicBool::new(false);

    pub fn record(len: usize) {
        if !REGISTERED.swap(true, Ordering::Relaxed) {
            unsafe { libc::atexit(dump) };
        }
        let i = BOUNDS.iter().position(|&b| len <= b).unwrap_or(7);
        COUNTS[i].fetch_add(1, Ordering::Relaxed);
        BYTES.fetch_add(len as u64, Ordering::Relaxed);
    }

    extern "C" fn dump() {
        use std::io::Write;
        let Ok(path) = std::env::var("PHPR_STR_CENSUS") else { return };
        let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) else {
            return;
        };
        let mut line = format!("pid={}", std::process::id());
        let mut prev = 0usize;
        for (i, &b) in BOUNDS.iter().enumerate() {
            let label = if b == usize::MAX {
                format!("{}+", prev)
            } else {
                format!("{}-{}", prev, b)
            };
            line.push_str(&format!(" {}={}", label, COUNTS[i].load(Ordering::Relaxed)));
            prev = b.saturating_add(1);
        }
        line.push_str(&format!(" bytes={}\n", BYTES.load(Ordering::Relaxed)));
        let _ = f.write_all(line.as_bytes());
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
