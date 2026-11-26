//! S-194 N1: first occurrence of a byte string in another, for the string builtins.
//!
//! `memchr::memmem::find` builds a searcher (prefilter ranking) on every call: ~1000 instructions
//! before the first byte is compared, more than the search itself for the common strpos /
//! str_contains call. A one-byte needle and a short haystack are answered by memchr on the
//! needle's first byte plus a slice comparison; longer haystacks keep the searcher.
//! Criterion: wp194-harness/s194-criterio-N.md.

/// First byte index of `needle` in `hay`. Empty needle matches at 0.
pub(crate) fn find_sub(hay: &[u8], needle: &[u8]) -> Option<usize> {
    let n = needle.len();
    if n == 1 {
        return memchr::memchr(needle[0], hay);
    }
    if n > hay.len() {
        return None;
    }
    if n == 0 || hay.len() > 256 {
        return memchr::memmem::find(hay, needle);
    }
    let last = hay.len() - n;
    let mut i = 0;
    while i <= last {
        let at = i + memchr::memchr(needle[0], &hay[i..=last])?;
        if &hay[at..at + n] == needle {
            return Some(at);
        }
        i = at + 1;
    }
    None
}
