use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

/// A PHP string: an arbitrary byte sequence (never assumed UTF-8).
///
/// Mirrors `zend_string` (Zend/zend_types.h:393-398): lazy hash with 0 meaning
/// "not yet computed", same convention as ZSTR_H (Zend/zend_string.h:114).
///
/// WP-38: un SSO (enum Inline/Heap dentro questa struct) è stato provato e
/// BOCCIATO dai dati — media reale +1,5% (cap 7, 24B totali via niche) /
/// +2,5% (cap 15, 32B); i malloc small-bin di mimalloc costano meno delle
/// copie inline + branch su ogni lettura. Restano i costruttori slice-fed
/// (`new` accetta `&[u8]`), `concat2` e `from_i64`, che evitano round-trip
/// inutili senza cambiare la rappresentazione. Da non riproporre senza
/// nuovi dati (cfr. NaN-boxing WP-32).
pub struct PhpStr {
    hash: Cell<u64>,
    bytes: Box<[u8]>,
}

const _: () = {
    assert!(std::mem::size_of::<PhpStr>() == 24);
};

pub type ZStr = Rc<PhpStr>;

impl PhpStr {
    /// The single construction funnel: every PhpStr goes through here.
    /// Accepts both owned buffers and plain slices (`&[u8]`/`&str` callers
    /// need no `to_vec` round-trip).
    pub fn new(bytes: impl AsRef<[u8]> + Into<Box<[u8]>>) -> ZStr {
        #[cfg(feature = "str-census")]
        census::record(bytes.as_ref().len());
        #[cfg(feature = "mem-census")]
        crate::memcensus::alloc(
            crate::memcensus::CH_STR,
            bytes.as_ref().len() + crate::memcensus::STR_OVERHEAD,
        );
        Rc::new(PhpStr {
            hash: Cell::new(0),
            bytes: bytes.into(),
        })
    }

    /// Binary concatenation in one exact-size allocation (WP-38): a small
    /// result builds straight into the inline buffer, a large one into an
    /// exactly-sized heap buffer. Byte-wise identical to concatenating into
    /// a Vec and calling `new`.
    pub fn concat2(a: &[u8], b: &[u8]) -> ZStr {
        let mut out = Vec::with_capacity(a.len() + b.len());
        out.extend_from_slice(a);
        out.extend_from_slice(b);
        Self::new(out)
    }

    /// Integer stringification without the `String`/fmt round-trip (WP-38):
    /// digits are rendered into a stack buffer and funneled through `new`.
    /// Byte-wise identical to `l.to_string()`.
    pub fn from_i64(v: i64) -> ZStr {
        let mut buf = [0u8; 20];
        let mut i = buf.len();
        let mut u = v.unsigned_abs();
        loop {
            i -= 1;
            buf[i] = b'0' + (u % 10) as u8;
            u /= 10;
            if u == 0 {
                break;
            }
        }
        if v < 0 {
            i -= 1;
            buf[i] = b'-';
        }
        Self::new(&buf[i..])
    }

    #[allow(clippy::should_implement_trait)] // infallible byte view, not FromStr
    pub fn from_str(s: &str) -> ZStr {
        Self::new(s.as_bytes())
    }

    pub fn empty() -> ZStr {
        Self::new(&[][..])
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// DJBX33A, same algorithm as zend_inline_hash_func. Lazily cached.
    /// Not observable in program output; kept identical to ease cross-debugging.
    pub fn zhash(&self) -> u64 {
        let h = self.hash.get();
        if h != 0 {
            return h;
        }
        let mut hash: u64 = 5381;
        for &b in self.as_bytes().iter() {
            hash = hash.wrapping_mul(33).wrapping_add(b as u64);
        }
        // Mirror Zend: force the "computed" bit so a result of 0 is impossible.
        let hash = hash | 0x8000_0000_0000_0000;
        self.hash.set(hash);
        hash
    }
}

/// Fase 0 byte-census: exact live tracking for the STR channel (the `new`
/// funnel is the single construction site). Census builds only.
#[cfg(feature = "mem-census")]
impl Drop for PhpStr {
    fn drop(&mut self) {
        crate::memcensus::free(
            crate::memcensus::CH_STR,
            self.bytes.len() + crate::memcensus::STR_OVERHEAD,
        );
    }
}

impl PartialEq for PhpStr {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
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
        write!(f, "PhpStr({:?})", String::from_utf8_lossy(self.as_bytes()))
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

    #[test]
    fn slice_and_vec_sources_agree() {
        // `new` accepts both owned buffers and slices: same bytes, len, zhash.
        for n in [0usize, 1, 14, 15, 16, 64] {
            let src: Vec<u8> = (0..n as u8).collect();
            let from_vec = PhpStr::new(src.clone());
            let from_slice = PhpStr::new(&src[..]);
            assert_eq!(from_vec.as_bytes(), &src[..], "n={n}");
            assert_eq!(from_vec.len(), n, "n={n}");
            assert_eq!(*from_vec, *from_slice, "n={n}");
            assert_eq!(from_vec.zhash(), from_slice.zhash(), "n={n}");
        }
    }

    #[test]
    fn concat2_matches_vec_path() {
        for (a, b) in [
            (&b""[..], &b""[..]),
            (b"abc", b""),
            (b"1234567", b"89012345"),  // 15: inline
            (b"12345678", b"89012345"), // 16: heap
            (b"x", &[0u8, 255][..]),
        ] {
            let fused = PhpStr::concat2(a, b);
            let mut v = a.to_vec();
            v.extend_from_slice(b);
            let plain = PhpStr::new(v);
            assert_eq!(*fused, *plain);
            assert_eq!(fused.zhash(), plain.zhash());
            assert_eq!(fused.len(), a.len() + b.len());
        }
    }

    #[test]
    fn from_i64_matches_to_string() {
        for v in [0i64, 7, -1, 42, -308641975, i64::MAX, i64::MIN] {
            assert_eq!(PhpStr::from_i64(v).as_bytes(), v.to_string().as_bytes());
        }
    }

    #[test]
    fn short_binary_safe() {
        let s = PhpStr::new(vec![0u8, 255, 0, 7]);
        assert_eq!(s.as_bytes(), &[0, 255, 0, 7]);
        assert_eq!(s.len(), 4);
        assert!(!s.is_empty());
        assert!(PhpStr::empty().is_empty());
    }
}
