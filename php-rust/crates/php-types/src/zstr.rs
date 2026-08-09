use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::borrow::Borrow;
use std::cell::Cell;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;

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
///
/// S-124 single-alloc: la coppia `Rc<PhpStr{hash, Vec<u8>}>` (2 malloc: RcBox
/// + buffer del Vec) diventa UN blocco stile `zend_string`: questo header di
/// 32 B `{rc, hash, len, cap}` con i byte in coda alla STESSA allocazione e
/// refcount custom non-atomico (la VM è single-thread; `NonNull` tiene ZStr
/// !Send/!Sync come lo era Rc). La lettura resta un deref — nessun branch SSO
/// — ma sparisce un hop (offset fisso invece del puntatore del Vec) e l'header
/// condivide la cache line coi primi byte. NON è l'SSO bocciato WP-38.
///
/// WP-55 (invariata nella sostanza): l'append-in-place di `.=` (mirror di
/// `zend_string_extend`) vive in [`ZStr::try_append`]: SOLO quando la stringa
/// è unica (rc == 1) il blocco cresce con `realloc` ammortizzato ×2 invece di
/// riallocare+copiare l'intera stringa (canale O(n²), probe WP-54: 244× vs
/// oracle). I costruttori restano exact-size (`cap == len`); solo il path
/// append lascia slack di crescita. Le stringhe condivise restano
/// copy-on-write per costruzione (fallback `concat2` al sito di chiamata).
#[repr(C)]
pub struct PhpStr {
    /// Non-atomic refcount, mirrors `Rc`'s strong count (no weak field).
    rc: Cell<usize>,
    hash: Cell<u64>,
    len: usize,
    cap: usize,
    // `len` payload bytes follow the header in the same allocation
    // (`cap` bytes reserved).
}

const _: () = {
    assert!(std::mem::size_of::<PhpStr>() == 32);
    assert!(std::mem::align_of::<PhpStr>() == 8);
};

const HDR: usize = std::mem::size_of::<PhpStr>();

/// Layout of a whole string block (header + `cap` payload bytes).
fn block_layout(cap: usize) -> Layout {
    let size = HDR.checked_add(cap).expect("PhpStr capacity overflow");
    Layout::from_size_align(size, std::mem::align_of::<PhpStr>())
        .expect("PhpStr layout overflow")
}

/// Owning handle to a refcounted single-allocation [`PhpStr`] block.
/// Replaces the old `pub type ZStr = Rc<PhpStr>`: same 8-byte niche-friendly
/// representation, same !Send/!Sync (raw pointer inside), same
/// Deref-to-`PhpStr` call sites.
pub struct ZStr {
    ptr: NonNull<PhpStr>,
    _own: PhantomData<PhpStr>,
}

impl ZStr {
    /// Allocate a block with `cap` reserved bytes and `len` already claimed.
    /// The caller must initialise `len` payload bytes before the ZStr is read.
    fn alloc_block(len: usize, cap: usize) -> NonNull<PhpStr> {
        let lay = block_layout(cap);
        let raw = unsafe { alloc(lay) };
        let Some(ptr) = NonNull::new(raw.cast::<PhpStr>()) else {
            handle_alloc_error(lay)
        };
        unsafe {
            ptr.as_ptr().write(PhpStr {
                rc: Cell::new(1),
                hash: Cell::new(0),
                len,
                cap,
            });
        }
        ptr
    }

    #[inline]
    fn from_raw(ptr: NonNull<PhpStr>) -> ZStr {
        ZStr {
            ptr,
            _own: PhantomData,
        }
    }

    #[inline]
    fn data_ptr(ptr: NonNull<PhpStr>) -> *mut u8 {
        unsafe { ptr.as_ptr().cast::<u8>().add(HDR) }
    }

    /// Pointer identity (the old `Rc::ptr_eq`).
    #[inline]
    pub fn ptr_eq(a: &ZStr, b: &ZStr) -> bool {
        a.ptr == b.ptr
    }

    /// Stable header address (the old `Rc::as_ptr`); census dedup key.
    #[inline]
    pub fn as_ptr(this: &ZStr) -> *const PhpStr {
        this.ptr.as_ptr()
    }

    /// WP-55 append-in-place, single-alloc edition: extends the block with
    /// amortised growth and INVALIDATES the cached hash — the one in-place
    /// mutation site; every constructor freezes the bytes. Returns `false`
    /// when the string is shared (`rc > 1`): the caller falls back to
    /// `concat2`, which is what keeps aliased/interned strings copy-on-write
    /// by construction (the old `Rc::get_mut` gate, run.rs fused `.=`).
    ///
    /// `more` can never alias this block: borrowing it from `self` is ruled
    /// out by `&mut self`, and a slice from another handle to the same block
    /// implies `rc > 1`, which bails out above.
    pub fn try_append(&mut self, more: &[u8]) -> bool {
        if self.rc.get() != 1 {
            return false;
        }
        #[cfg(feature = "mem-census")]
        crate::memcensus::adjust(crate::memcensus::CH_STR, more.len() as i64);
        unsafe {
            let (len, cap) = {
                let h = self.ptr.as_ref();
                (h.len, h.cap)
            };
            let need = len.checked_add(more.len()).expect("PhpStr length overflow");
            if need > cap {
                // Vec-mirror growth: at least double, never below `need`.
                let new_cap = need.max(cap.saturating_mul(2));
                let new_lay = block_layout(new_cap);
                let raw = realloc(
                    self.ptr.as_ptr().cast::<u8>(),
                    block_layout(cap),
                    new_lay.size(),
                );
                let Some(p) = NonNull::new(raw.cast::<PhpStr>()) else {
                    handle_alloc_error(new_lay)
                };
                self.ptr = p;
                (*self.ptr.as_ptr()).cap = new_cap;
            }
            std::ptr::copy_nonoverlapping(
                more.as_ptr(),
                Self::data_ptr(self.ptr).add(len),
                more.len(),
            );
            let h = self.ptr.as_ptr();
            (*h).len = need;
            (*h).hash.set(0);
        }
        true
    }
}

impl Clone for ZStr {
    #[inline]
    fn clone(&self) -> ZStr {
        // Mirror Rc: wrapping increment, hard abort on overflow.
        let n = self.rc.get().wrapping_add(1);
        self.rc.set(n);
        if n == 0 {
            std::process::abort();
        }
        ZStr::from_raw(self.ptr)
    }
}

impl Drop for ZStr {
    #[inline]
    fn drop(&mut self) {
        let n = self.rc.get() - 1;
        self.rc.set(n);
        if n == 0 {
            #[cfg(feature = "mem-census")]
            crate::memcensus::free(
                crate::memcensus::CH_STR,
                self.len() + crate::memcensus::STR_OVERHEAD,
            );
            unsafe {
                let cap = self.ptr.as_ref().cap;
                dealloc(self.ptr.as_ptr().cast::<u8>(), block_layout(cap));
            }
        }
    }
}

impl Deref for ZStr {
    type Target = PhpStr;
    #[inline]
    fn deref(&self) -> &PhpStr {
        unsafe { self.ptr.as_ref() }
    }
}

impl Borrow<PhpStr> for ZStr {
    #[inline]
    fn borrow(&self) -> &PhpStr {
        self
    }
}

impl PartialEq for ZStr {
    /// RcEqIdent semantics preserved BY HAND (istruttoria S-123, rischio 1):
    /// the libstd specialisation `ptr_eq || byte-eq` for `Rc<T: Eq>` does not
    /// carry over to a custom type — losing it silently regresses key lookups.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ZStr::ptr_eq(self, other) || self.as_bytes() == other.as_bytes()
    }
}

impl Eq for ZStr {}

impl std::hash::Hash for ZStr {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl fmt::Debug for ZStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

/// Exact-capacity builder: writes multi-part concatenations (ConcatN join)
/// straight into the single block — no transient `Vec`.
pub struct ZStrBuilder {
    ptr: NonNull<PhpStr>,
    written: usize,
}

impl ZStrBuilder {
    pub fn push(&mut self, bytes: &[u8]) {
        let cap = unsafe { self.ptr.as_ref().cap };
        assert!(
            bytes.len() <= cap - self.written,
            "ZStrBuilder capacity overflow"
        );
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                ZStr::data_ptr(self.ptr).add(self.written),
                bytes.len(),
            );
        }
        self.written += bytes.len();
    }

    pub fn finish(self) -> ZStr {
        unsafe {
            let cap = self.ptr.as_ref().cap;
            let mut ptr = self.ptr;
            if self.written < cap {
                // Exact-size discipline (WP-55 pin): shrink the rare
                // under-filled block, same contract as the old shrink_to_fit.
                let new_lay = block_layout(self.written);
                let raw = realloc(
                    ptr.as_ptr().cast::<u8>(),
                    block_layout(cap),
                    new_lay.size(),
                );
                let Some(p) = NonNull::new(raw.cast::<PhpStr>()) else {
                    handle_alloc_error(new_lay)
                };
                ptr = p;
                (*ptr.as_ptr()).cap = self.written;
            }
            (*ptr.as_ptr()).len = self.written;
            #[cfg(feature = "str-census")]
            census::record(self.written);
            #[cfg(feature = "mem-census")]
            crate::memcensus::alloc(
                crate::memcensus::CH_STR,
                self.written + crate::memcensus::STR_OVERHEAD,
            );
            std::mem::forget(self);
            ZStr::from_raw(ptr)
        }
    }
}

impl Drop for ZStrBuilder {
    fn drop(&mut self) {
        // Abandoned builder (no `finish`): release the block; no census
        // alloc was recorded yet, so nothing to balance.
        unsafe {
            let cap = self.ptr.as_ref().cap;
            dealloc(self.ptr.as_ptr().cast::<u8>(), block_layout(cap));
        }
    }
}

impl PhpStr {
    /// The single construction funnel: every PhpStr goes through here.
    /// Accepts any byte view (`&[u8]`/`&str`/`Vec<u8>`/`String`); the payload
    /// is copied once into the block (`cap == len`, exact-size by build).
    pub fn new(bytes: impl AsRef<[u8]>) -> ZStr {
        let b = bytes.as_ref();
        #[cfg(feature = "str-census")]
        census::record(b.len());
        #[cfg(feature = "mem-census")]
        crate::memcensus::alloc(
            crate::memcensus::CH_STR,
            b.len() + crate::memcensus::STR_OVERHEAD,
        );
        let ptr = ZStr::alloc_block(b.len(), b.len());
        unsafe {
            std::ptr::copy_nonoverlapping(b.as_ptr(), ZStr::data_ptr(ptr), b.len());
        }
        ZStr::from_raw(ptr)
    }

    /// Exact-capacity multi-part builder (see [`ZStrBuilder`]).
    pub fn builder(cap: usize) -> ZStrBuilder {
        ZStrBuilder {
            ptr: ZStr::alloc_block(0, cap),
            written: 0,
        }
    }

    /// Binary concatenation in ONE exact-size block (WP-38, S-124: written
    /// directly, no intermediate buffer). Byte-wise identical to
    /// concatenating into a Vec and calling `new`.
    pub fn concat2(a: &[u8], b: &[u8]) -> ZStr {
        let total = a.len().checked_add(b.len()).expect("PhpStr length overflow");
        #[cfg(feature = "str-census")]
        census::record(total);
        #[cfg(feature = "mem-census")]
        crate::memcensus::alloc(
            crate::memcensus::CH_STR,
            total + crate::memcensus::STR_OVERHEAD,
        );
        let ptr = ZStr::alloc_block(total, total);
        unsafe {
            let d = ZStr::data_ptr(ptr);
            std::ptr::copy_nonoverlapping(a.as_ptr(), d, a.len());
            std::ptr::copy_nonoverlapping(b.as_ptr(), d.add(a.len()), b.len());
        }
        ZStr::from_raw(ptr)
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
        unsafe {
            std::slice::from_raw_parts((self as *const PhpStr).add(1).cast::<u8>(), self.len)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
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

    #[test]
    fn append_unique_shared_and_hash_invalidation() {
        let mut s = PhpStr::from_str("ab");
        let h0 = s.zhash();
        assert!(s.try_append(b"cd")); // unique: in-place
        assert_eq!(s.as_bytes(), b"abcd");
        assert_ne!(s.zhash(), h0); // cached hash invalidated
        let alias = s.clone();
        assert!(!s.try_append(b"x")); // shared: COW fallback at the call site
        assert_eq!(s.as_bytes(), b"abcd");
        assert_eq!(alias.as_bytes(), b"abcd");
        drop(alias);
        assert!(s.try_append(b"ef")); // unique again after the alias drops
        assert_eq!(s.as_bytes(), b"abcdef");
    }

    #[test]
    fn append_grows_across_many_extends() {
        // Exercises the realloc path repeatedly (amortised growth).
        let mut s = PhpStr::from_str("");
        let mut expect = Vec::new();
        for i in 0..200u8 {
            assert!(s.try_append(&[i, i, i]));
            expect.extend_from_slice(&[i, i, i]);
        }
        assert_eq!(s.as_bytes(), &expect[..]);
    }

    #[test]
    fn eq_identity_fast_path_and_ptr_api() {
        let a = PhpStr::from_str("k1");
        let b = a.clone();
        assert!(ZStr::ptr_eq(&a, &b));
        assert_eq!(ZStr::as_ptr(&a), ZStr::as_ptr(&b));
        assert_eq!(a, b);
        assert_eq!(a, PhpStr::from_str("k1")); // byte path
        assert_ne!(a, PhpStr::from_str("k2"));
    }

    #[test]
    fn builder_exact_and_underfill() {
        let mut b = PhpStr::builder(5);
        b.push(b"he");
        b.push(b"llo");
        let s = b.finish();
        assert_eq!(s.as_bytes(), b"hello");
        assert_eq!(s.len(), 5);
        // Under-filled builder shrinks to written size.
        let mut u = PhpStr::builder(64);
        u.push(b"xy");
        let t = u.finish();
        assert_eq!(t.as_bytes(), b"xy");
        assert_eq!(t.len(), 2);
        // Abandoned builder: Drop releases the block (no leak, no crash).
        let mut dead = PhpStr::builder(8);
        dead.push(b"zz");
        drop(dead);
    }
}
