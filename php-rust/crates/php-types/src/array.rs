use std::rc::Rc;

/// Fx-hashed key index: PHP array keys are hashed on every dim access, where
/// Zend reads a precomputed zend_string hash — SipHash here was ~10% of the
/// per-request profile. Insertion order lives in `entries`, so the hasher is
/// not observable.
type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

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

// The packed representation stores one `Option<Zval>` per slot; the enum
// niche must keep that at zval size (16B, like Zend's packed Bucket payload).
const _: () = assert!(
    std::mem::size_of::<Option<Zval>>() == std::mem::size_of::<Zval>()
);

/// Storage representation, mirroring Zend's packed/mixed hash split
/// (invisible to programs; the split only changes memory/CPU costs).
///
/// `Packed`: live keys are exactly the slot positions (`entries[i]` holds key
/// `i`); tombstones (`None`) only ever come from `unset` and are never
/// re-filled in place — the oracle appends re-inserted keys at the END of the
/// iteration order (packed_probe: `unset($a[1]); $a[1]=99` iterates 0,2,1),
/// so writing into a tombstone or past the end escalates to `Hashed` first.
/// One-way: an array never goes back to `Packed` (rebuilt arrays start fresh).
#[derive(Debug)]
enum Repr {
    Packed(Vec<Option<Zval>>),
    Hashed {
        entries: Vec<Option<(Key, Zval)>>,
        index: HashMap<Key, u32>,
    },
}

/// A PHP array: an insertion-ordered hash with int|string keys.
///
/// Mirrors the observable semantics of `zend_array` (Zend/zend_types.h:408-432):
/// iteration order is insertion order (survives unset), tombstones like Zend's
/// IS_UNDEF buckets, `next_free` never decreases on unset. Like Zend, a
/// dense 0..n int-keyed array is stored packed (values only, no key/index —
/// see [`Repr`]); the distinction is invisible to programs.
#[derive(Debug)]
pub struct PhpArray {
    repr: Repr,
    /// Mirrors nNextFreeElement: starts at i64::MIN (zend_hash.c:257),
    /// "MIN means empty-append uses 0" (zend_hash.c:1099), saturates at
    /// i64::MAX (zend_hash.c:1183), never decreases on unset.
    next_free: i64,
    count: u32,
    /// The internal pointer (`reset`/`next`/`prev`/`end`/`current`/`key`), as a
    /// slot position. `>= slots` (or pointing only at tombstones to its right)
    /// means "past the end" / invalid — `current` is then `false`. Reads skip
    /// forward over tombstones from this index (mirrors Zend advancing the
    /// pointer when the pointed bucket is deleted). `foreach` snapshots and does
    /// not touch it (PHP 8). Carried by `Clone`/COW like the rest of the array
    /// state. Escalation preserves slot positions, so the cursor survives it.
    cursor: usize,
    /// Conservative container-content marker: `false` only when every value ever
    /// stored is a scalar/string AND no `&mut` element handle was ever handed out
    /// (a caller could promote a scalar in place). Lets the GC's drop-descent and
    /// cycle classify skip scalar-only arrays without iterating them. Never
    /// cleared by `remove` — stays pessimistic once set.
    holds_containers: bool,
}

/// The element-duplication rule of `zend_array_dup` (Zend/zend_hash.c): an
/// element that is a REFERENCE this array is the only holder of (refcount 1 —
/// typically `foreach (… as &$v)` residue after the alias variable died)
/// is SPLIT into a plain value in the duplicate; a reference someone else
/// still aliases stays shared. Without the split, a by-ref foreach over a
/// COW copy writes through the surviving cells into every other holder
/// (WP_REST_Server::get_routes corrupted `$this->endpoints` this way).
/// …UNLESS the referent is this very array (a `$a[] =& $a` self-cycle):
/// zend_array_dup keeps that reference shared (bug69376).
fn dup_element(v: &Zval, owner: *const PhpArray) -> Zval {
    match v {
        Zval::Ref(cell) if Rc::strong_count(cell) == 1 => {
            let self_ref = matches!(
                &*cell.borrow(),
                Zval::Array(rc) if std::ptr::eq(Rc::as_ptr(rc), owner)
            );
            if self_ref {
                Zval::Ref(Rc::clone(cell))
            } else {
                cell.borrow().clone()
            }
        }
        other => other.clone(),
    }
}

impl Clone for PhpArray {
    fn clone(&self) -> Self {
        let owner = self as *const PhpArray;
        let repr = match &self.repr {
            Repr::Packed(slots) => Repr::Packed(
                slots
                    .iter()
                    .map(|e| e.as_ref().map(|v| dup_element(v, owner)))
                    .collect(),
            ),
            Repr::Hashed { entries, index } => Repr::Hashed {
                entries: entries
                    .iter()
                    .map(|e| {
                        e.as_ref()
                            .map(|(k, v)| (k.clone(), dup_element(v, owner)))
                    })
                    .collect(),
                index: index.clone(),
            },
        };
        #[cfg(feature = "mem-census")]
        crate::memcensus::count_alloc(crate::memcensus::CH_ARR);
        PhpArray {
            repr,
            next_free: self.next_free,
            count: self.count,
            cursor: self.cursor,
            holds_containers: self.holds_containers,
        }
    }
}

/// Fase 0 byte-census (census builds only): the ARR channel is
/// death-accounted — exact capacity bytes measured when the array drops,
/// live-count via new/default/clone. Live bytes are estimated at dump time
/// (live_n × average death size) and cross-checked against the residual of
/// the external peak footprint.
#[cfg(feature = "mem-census")]
impl Drop for PhpArray {
    fn drop(&mut self) {
        crate::memcensus::death(crate::memcensus::CH_ARR, self.census_bytes());
        crate::memcensus::count_free(crate::memcensus::CH_ARR);
    }
}

#[cfg(feature = "mem-census")]
impl PhpArray {
    /// Retained capacity bytes of this array right now: element storage plus
    /// (for hashed) the index map, approximated at hashbrown's ~1 ctrl byte
    /// per bucket, plus the fixed header+Rc overhead.
    pub(crate) fn census_bytes(&self) -> usize {
        let body = match &self.repr {
            Repr::Packed(slots) => slots.capacity() * std::mem::size_of::<Option<Zval>>(),
            Repr::Hashed { entries, index } => {
                entries.capacity() * std::mem::size_of::<Option<(Key, Zval)>>()
                    + index.capacity() * (std::mem::size_of::<(Key, u32)>() + 1)
            }
        };
        body + crate::memcensus::ARR_OVERHEAD
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        #[cfg(feature = "mem-census")]
        crate::memcensus::count_alloc(crate::memcensus::CH_ARR);
        PhpArray {
            repr: Repr::Packed(Vec::new()),
            next_free: i64::MIN,
            count: 0,
            cursor: 0,
            holds_containers: false,
        }
    }
}

impl PhpArray {
    pub fn new() -> PhpArray {
        PhpArray::default()
    }


    #[inline]
    pub fn len(&self) -> usize {
        self.count as usize
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Whether this array may (transitively) hold objects / references /
    /// other containers — see the `holds_containers` field. `false` is a
    /// guarantee; `true` only means "must be walked".
    #[inline]
    pub fn may_hold_containers(&self) -> bool {
        self.holds_containers
    }

    /// Convert a packed array to the hashed representation. Slot positions
    /// (and therefore the cursor) and tombstones are preserved exactly.
    fn to_hashed(&mut self) {
        let Repr::Packed(slots) = &mut self.repr else {
            return;
        };
        let entries: Vec<Option<(Key, Zval)>> = std::mem::take(slots)
            .into_iter()
            .enumerate()
            .map(|(i, e)| e.map(|v| (Key::Int(i as i64), v)))
            .collect();
        let mut index = HashMap::default();
        index.reserve(entries.len());
        for (pos, entry) in entries.iter().enumerate() {
            if let Some((key, _)) = entry {
                index.insert(key.clone(), pos as u32);
            }
        }
        self.repr = Repr::Hashed { entries, index };
    }

    /// Insert or update. Updating an existing key keeps its position.
    pub fn insert(&mut self, key: Key, val: Zval) {
        self.holds_containers |= !matches!(
            val,
            Zval::Undef | Zval::Null | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_) | Zval::Str(_)
        );
        if let Repr::Packed(slots) = &mut self.repr {
            match key {
                Key::Int(i) if (i as usize) < slots.len() && i >= 0 => {
                    if let Some(slot) = &mut slots[i as usize] {
                        // Update keeps its position (and next_free untouched:
                        // an existing key is always < next_free).
                        *slot = val;
                        return;
                    }
                    // Tombstone: a re-inserted key goes to the END of the
                    // iteration order (oracle-pinned) — escalate.
                }
                Key::Int(i) if i == slots.len() as i64 => {
                    if i >= self.next_free {
                        self.next_free = i.saturating_add(1);
                    }
                    slots.push(Some(val));
                    self.count += 1;
                    return;
                }
                _ => {}
            }
            self.to_hashed();
        }
        let Repr::Hashed { entries, index } = &mut self.repr else {
            unreachable!()
        };
        if let Some(&pos) = index.get(&key) {
            entries[pos as usize] = Some((key, val));
            return;
        }
        if let Key::Int(i) = key {
            if i >= self.next_free {
                self.next_free = i.saturating_add(1);
            }
        }
        let pos = entries.len() as u32;
        index.insert(key.clone(), pos);
        entries.push(Some((key, val)));
        self.count += 1;
    }

    /// Get-or-insert-`Null` in ONE lookup (WP-32): the exact semantics of the
    /// `contains_key` + `insert(key, Null)` + `get_mut` composite the nested
    /// array-write drill used to run (2-4 hash lookups + a key clone per
    /// level). Tombstone/hole/negative/string keys on a packed array escalate
    /// first, and a vivified key lands at the END of the iteration order —
    /// the WP-27 no-revive rule, oracle-pinned. `holds_containers` is set on
    /// return exactly like the composite's `get_mut` did (the caller may
    /// write any value through the handle).
    pub fn slot_or_vivify(&mut self, key: Key) -> &mut Zval {
        enum Plan {
            Hit(usize),
            Append(i64),
            Escalate,
        }
        if let Repr::Packed(slots) = &mut self.repr {
            let plan = match key {
                Key::Int(i) if (i as usize) < slots.len() && i >= 0 => {
                    if slots[i as usize].is_some() {
                        Plan::Hit(i as usize)
                    } else {
                        // Tombstone: re-inserted keys go to the END — escalate.
                        Plan::Escalate
                    }
                }
                Key::Int(i) if i == slots.len() as i64 => Plan::Append(i),
                _ => Plan::Escalate,
            };
            match plan {
                Plan::Hit(i) => {
                    self.holds_containers = true;
                    let Repr::Packed(slots) = &mut self.repr else { unreachable!() };
                    return slots[i].as_mut().unwrap();
                }
                Plan::Append(i) => {
                    if i >= self.next_free {
                        self.next_free = i.saturating_add(1);
                    }
                    self.count += 1;
                    self.holds_containers = true;
                    let Repr::Packed(slots) = &mut self.repr else { unreachable!() };
                    slots.push(Some(Zval::Null));
                    return slots.last_mut().unwrap().as_mut().unwrap();
                }
                Plan::Escalate => self.to_hashed(),
            }
        }
        self.holds_containers = true;
        let hit = {
            let Repr::Hashed { index, .. } = &self.repr else { unreachable!() };
            index.get(&key).copied()
        };
        if hit.is_none() {
            if let Key::Int(i) = key {
                if i >= self.next_free {
                    self.next_free = i.saturating_add(1);
                }
            }
            self.count += 1;
        }
        let Repr::Hashed { entries, index } = &mut self.repr else { unreachable!() };
        let pos = match hit {
            Some(pos) => pos,
            None => {
                let pos = entries.len() as u32;
                index.insert(key.clone(), pos);
                entries.push(Some((key, Zval::Null)));
                pos
            }
        };
        &mut entries[pos as usize].as_mut().unwrap().1
    }

    /// Single-lookup leaf write (WP-32): the exact semantics of the
    /// `get_mut` + write-through-Ref (REF-4) + fallback-`insert` composite of
    /// the array path-write leaf. A hit writes THROUGH an existing `Ref` slot
    /// (aliases observe the update) or overwrites in place, returning the
    /// displaced value for GC noting, and sets `holds_containers` exactly
    /// like `get_mut` did; a miss delegates to [`Self::insert`] (new-key
    /// logic byte-identical, `holds_containers` from the VALUE — never
    /// vivify-Null-then-overwrite, which would mis-flag scalar-only arrays
    /// and feed a spurious Null to gc_note) and returns `None`.
    pub fn set_returning_displaced(&mut self, key: Key, val: Zval) -> Option<Zval> {
        fn write_slot(slot: &mut Zval, val: Zval) -> Zval {
            match slot {
                Zval::Ref(cell) => std::mem::replace(&mut *cell.borrow_mut(), val),
                _ => std::mem::replace(slot, val),
            }
        }
        let hit = match &self.repr {
            Repr::Packed(slots) => match &key {
                Key::Int(i) if (*i as usize) < slots.len() && *i >= 0 && slots[*i as usize].is_some() => {
                    Some(*i as usize)
                }
                _ => None,
            },
            Repr::Hashed { index, .. } => index.get(&key).map(|&pos| pos as usize),
        };
        match hit {
            Some(pos) => {
                self.holds_containers = true;
                let displaced = match &mut self.repr {
                    Repr::Packed(slots) => write_slot(slots[pos].as_mut().unwrap(), val),
                    Repr::Hashed { entries, .. } => {
                        write_slot(&mut entries[pos].as_mut().unwrap().1, val)
                    }
                };
                Some(displaced)
            }
            None => {
                self.insert(key, val);
                None
            }
        }
    }

    /// Zend's `array_pop` adjustment (ext/standard/array.c:3579): popping the
    /// element whose int key was the latest auto-index (`next_free - 1`)
    /// frees that index again, so pop-then-append reuses the same key.
    pub fn pop_adjust_next_free(&mut self, popped: &Key) {
        if let Key::Int(i) = popped {
            if self.next_free != i64::MIN && *i == self.next_free - 1 {
                self.next_free = *i;
            }
        }
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

    /// `&$a[]`: append a fresh `Null` element at the next free int index and
    /// return a mutable reference to it, so a caller can promote it to a shared
    /// reference cell. `None` when that slot is occupied (saturation), matching
    /// [`Self::append`].
    pub fn append_default(&mut self) -> Option<&mut Zval> {
        let h = if self.next_free == i64::MIN { 0 } else { self.next_free };
        if self.contains_key(&Key::Int(h)) {
            return None;
        }
        self.insert(Key::Int(h), Zval::Null);
        self.get_mut(&Key::Int(h))
    }

    #[inline]
    pub fn get(&self, key: &Key) -> Option<&Zval> {
        match &self.repr {
            Repr::Packed(slots) => match key {
                Key::Int(i) if (*i as usize) < slots.len() && *i >= 0 => {
                    slots[*i as usize].as_ref()
                }
                _ => None,
            },
            Repr::Hashed { entries, index } => index
                .get(key)
                .map(|&pos| &entries[pos as usize].as_ref().unwrap().1),
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Zval> {
        match &mut self.repr {
            Repr::Packed(slots) => match key {
                Key::Int(i) if (*i as usize) < slots.len() && *i >= 0 => {
                    match &mut slots[*i as usize] {
                        Some(v) => {
                            // The caller may write any value through this handle.
                            self.holds_containers = true;
                            Some(v)
                        }
                        None => None,
                    }
                }
                _ => None,
            },
            Repr::Hashed { entries, index } => match index.get(key) {
                Some(&pos) => {
                    self.holds_containers = true;
                    Some(&mut entries[pos as usize].as_mut().unwrap().1)
                }
                None => None,
            },
        }
    }

    #[inline]
    pub fn contains_key(&self, key: &Key) -> bool {
        match &self.repr {
            Repr::Packed(slots) => matches!(
                key,
                Key::Int(i) if (*i as usize) < slots.len() && *i >= 0
                    && slots[*i as usize].is_some()
            ),
            Repr::Hashed { index, .. } => index.contains_key(key),
        }
    }

    /// `unset($a[k])`: leaves a tombstone so iteration order is preserved.
    /// `next_free` intentionally not touched (Zend semantics).
    pub fn remove(&mut self, key: &Key) -> Option<Zval> {
        match &mut self.repr {
            Repr::Packed(slots) => {
                let i = match key {
                    Key::Int(i) if (*i as usize) < slots.len() && *i >= 0 => *i as usize,
                    _ => return None,
                };
                let val = slots[i].take()?;
                self.count -= 1;
                // Trailing tombstones are dropped so that pop-then-append
                // (array_pop adjusts next_free back) re-pushes in place and
                // the array stays packed. Interior tombstones stay — they
                // cost one slot each, like Zend's IS_UNDEF buckets.
                if i + 1 == slots.len() {
                    while matches!(slots.last(), Some(None)) {
                        slots.pop();
                    }
                }
                Some(val)
            }
            Repr::Hashed { entries, index } => {
                let pos = index.remove(key)?;
                let (_, val) = entries[pos as usize].take().unwrap();
                self.count -= 1;
                if entries.len() >= 8 && (self.count as usize) < entries.len() / 2 {
                    self.compact();
                }
                Some(val)
            }
        }
    }

    fn compact(&mut self) {
        let Repr::Hashed { entries, index } = &mut self.repr else {
            return;
        };
        entries.retain(Option::is_some);
        index.clear();
        for (pos, entry) in entries.iter().enumerate() {
            let (key, _) = entry.as_ref().unwrap();
            index.insert(key.clone(), pos as u32);
        }
    }

    /// Iterate in insertion order, skipping tombstones. Keys are yielded by
    /// value: packed slots don't store them (`Int` is a copy, `Str` an Rc bump).
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        match &self.repr {
            Repr::Packed(slots) => Iter::Packed(slots.iter().enumerate()),
            Repr::Hashed { entries, .. } => Iter::Hashed(entries.iter()),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        // The caller may write any value through these handles.
        self.holds_containers = true;
        match &mut self.repr {
            Repr::Packed(slots) => IterMut::Packed(slots.iter_mut().enumerate()),
            Repr::Hashed { entries, .. } => IterMut::Hashed(entries.iter_mut()),
        }
    }

    // --- Internal pointer (`reset`/`next`/`prev`/`end`/`current`/`key`) ---

    /// Total slot count (live + tombstones) — the domain of cursor positions.
    fn slots_len(&self) -> usize {
        match &self.repr {
            Repr::Packed(slots) => slots.len(),
            Repr::Hashed { entries, .. } => entries.len(),
        }
    }

    /// Whether the slot at `i` is live (not a tombstone).
    fn live_at(&self, i: usize) -> bool {
        match &self.repr {
            Repr::Packed(slots) => slots[i].is_some(),
            Repr::Hashed { entries, .. } => entries[i].is_some(),
        }
    }

    /// The effective position of the internal pointer: the first live entry at or
    /// after `cursor` (skipping tombstones), or `None` when the pointer is past the
    /// end. A read never moves `cursor`; it skips forward lazily, so deleting the
    /// pointed bucket makes the next live one current (matches Zend).
    fn cursor_pos(&self) -> Option<usize> {
        (self.cursor..self.slots_len()).find(|&i| self.live_at(i))
    }

    /// `current($a)`: the value at the internal pointer, or `None` (PHP `false`).
    pub fn ptr_current(&self) -> Option<Zval> {
        self.cursor_pos().map(|i| match &self.repr {
            Repr::Packed(slots) => slots[i].as_ref().unwrap().clone(),
            Repr::Hashed { entries, .. } => entries[i].as_ref().unwrap().1.clone(),
        })
    }

    /// `key($a)`: the key at the internal pointer, or `None` (PHP `null`).
    pub fn ptr_key(&self) -> Option<Key> {
        self.cursor_pos().map(|i| match &self.repr {
            Repr::Packed(_) => Key::Int(i as i64),
            Repr::Hashed { entries, .. } => entries[i].as_ref().unwrap().0.clone(),
        })
    }

    /// `reset($a)`: move the pointer to the first live entry; return its value.
    pub fn ptr_reset(&mut self) -> Option<Zval> {
        self.cursor = (0..self.slots_len())
            .find(|&i| self.live_at(i))
            .unwrap_or(self.slots_len());
        self.ptr_current()
    }

    /// `end($a)`: move the pointer to the last live entry; return its value.
    pub fn ptr_end(&mut self) -> Option<Zval> {
        self.cursor = (0..self.slots_len())
            .rev()
            .find(|&i| self.live_at(i))
            .unwrap_or(self.slots_len());
        self.ptr_current()
    }

    /// `next($a)`: advance the pointer to the next live entry; return its value.
    /// Already past the end stays past the end (`false`).
    pub fn ptr_next(&mut self) -> Option<Zval> {
        let start = match self.cursor_pos() {
            Some(i) => i + 1,
            None => self.slots_len(),
        };
        self.cursor = (start..self.slots_len())
            .find(|&i| self.live_at(i))
            .unwrap_or(self.slots_len());
        self.ptr_current()
    }

    /// `prev($a)`: retreat the pointer to the previous live entry; return its value.
    /// Stepping before the first entry invalidates the pointer (`false`).
    pub fn ptr_prev(&mut self) -> Option<Zval> {
        let end = self.cursor_pos().unwrap_or(self.slots_len());
        self.cursor = (0..end)
            .rev()
            .find(|&i| self.live_at(i))
            .unwrap_or(self.slots_len());
        self.ptr_current()
    }
}

/// Borrowing iterator over live entries — see [`PhpArray::iter`].
pub enum Iter<'a> {
    Packed(std::iter::Enumerate<std::slice::Iter<'a, Option<Zval>>>),
    Hashed(std::slice::Iter<'a, Option<(Key, Zval)>>),
}

impl<'a> Iterator for Iter<'a> {
    type Item = (Key, &'a Zval);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Packed(it) => it.find_map(|(i, e)| {
                e.as_ref().map(|v| (Key::Int(i as i64), v))
            }),
            Iter::Hashed(it) => it.find_map(|e| {
                e.as_ref().map(|(k, v)| (k.clone(), v))
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let upper = match self {
            Iter::Packed(it) => it.len(),
            Iter::Hashed(it) => it.len(),
        };
        (0, Some(upper))
    }
}

impl DoubleEndedIterator for Iter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            Iter::Packed(it) => loop {
                let (i, e) = it.next_back()?;
                if let Some(v) = e.as_ref() {
                    return Some((Key::Int(i as i64), v));
                }
            },
            Iter::Hashed(it) => loop {
                let e = it.next_back()?;
                if let Some((k, v)) = e.as_ref() {
                    return Some((k.clone(), v));
                }
            },
        }
    }
}

/// Mutably borrowing iterator over live entries — see [`PhpArray::iter_mut`].
pub enum IterMut<'a> {
    Packed(std::iter::Enumerate<std::slice::IterMut<'a, Option<Zval>>>),
    Hashed(std::slice::IterMut<'a, Option<(Key, Zval)>>),
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (Key, &'a mut Zval);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterMut::Packed(it) => it.find_map(|(i, e)| {
                e.as_mut().map(|v| (Key::Int(i as i64), v))
            }),
            IterMut::Hashed(it) => it.find_map(|e| {
                e.as_mut().map(|(k, v)| (k.clone(), v))
            }),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let upper = match self {
            IterMut::Packed(it) => it.len(),
            IterMut::Hashed(it) => it.len(),
        };
        (0, Some(upper))
    }
}

impl DoubleEndedIterator for IterMut<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self {
            IterMut::Packed(it) => loop {
                let (i, e) = it.next_back()?;
                if let Some(v) = e.as_mut() {
                    return Some((Key::Int(i as i64), v));
                }
            },
            IterMut::Hashed(it) => loop {
                let e = it.next_back()?;
                if let Some((k, v)) = e.as_mut() {
                    return Some((k.clone(), v));
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// slot_or_vivify must be indistinguishable from the composite it
    /// replaces (`contains_key` + `insert(key, Null)` + `get_mut`) across
    /// every repr shape — including count/next_free/holds_containers and
    /// the resulting iteration order (WP-27 no-revive).
    #[test]
    fn slot_or_vivify_equals_composite() {
        let shapes: Vec<(&str, Box<dyn Fn() -> PhpArray>)> = vec![
            ("packed", Box::new(|| {
                let mut a = PhpArray::new();
                for i in 0..3 {
                    let _ = a.append(Zval::Long(i));
                }
                a
            })),
            ("packed-tombstone", Box::new(|| {
                let mut a = PhpArray::new();
                for i in 0..3 {
                    let _ = a.append(Zval::Long(i));
                }
                a.remove(&Key::Int(1));
                a
            })),
            ("hashed", Box::new(|| {
                let mut a = PhpArray::new();
                a.insert(Key::from_bytes(b"x"), Zval::Long(7));
                a.insert(Key::Int(4), Zval::Long(8));
                a
            })),
            ("empty", Box::new(PhpArray::new)),
        ];
        let keys = [
            Key::Int(0),          // packed in-range hit
            Key::Int(1),          // tombstone on the tombstone shape
            Key::Int(3),          // packed append position
            Key::Int(9),          // hole → escalate
            Key::Int(-2),         // negative → escalate
            Key::from_bytes(b"x"),// string hit on hashed
            Key::from_bytes(b"nu"),// string miss
        ];
        for (name, mk) in &shapes {
            for key in &keys {
                let mut a = mk();
                let mut b = mk();
                // composite (the old drill)
                if !a.contains_key(key) {
                    a.insert(key.clone(), Zval::Null);
                }
                *a.get_mut(key).expect("composite slot") = Zval::Long(99);
                // fused
                *b.slot_or_vivify(key.clone()) = Zval::Long(99);
                assert_eq!(a.len(), b.len(), "{name}/{key:?} count");
                assert_eq!(a.next_free, b.next_free, "{name}/{key:?} next_free");
                assert_eq!(a.may_hold_containers(), b.may_hold_containers(), "{name}/{key:?} holds");
                let av: Vec<_> = a.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                let bv: Vec<_> = b.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                assert_eq!(format!("{av:?}"), format!("{bv:?}"), "{name}/{key:?} order");
            }
        }
    }

    /// set_returning_displaced must match the leaf-write composite
    /// (`get_mut` hit → write-through-Ref, miss → `insert`) on every shape.
    #[test]
    fn set_returning_displaced_equals_composite() {
        use std::cell::RefCell;
        use std::rc::Rc;
        // Ref write-through: the alias cell observes the new value and the
        // displaced INNER value comes back.
        let cell = Rc::new(RefCell::new(Zval::Long(5)));
        let mut a = PhpArray::new();
        a.insert(Key::from_bytes(b"r"), Zval::Ref(Rc::clone(&cell)));
        let d = a.set_returning_displaced(Key::from_bytes(b"r"), Zval::Long(9));
        assert!(matches!(d, Some(Zval::Long(5))));
        assert!(matches!(&*cell.borrow(), Zval::Long(9)));
        assert!(matches!(a.get(&Key::from_bytes(b"r")), Some(Zval::Ref(_))));
        // Plain hit: displaced returned, holds_containers set like get_mut.
        let mut b = PhpArray::new();
        let _ = b.append(Zval::Long(1));
        let d = b.set_returning_displaced(Key::Int(0), Zval::Long(2));
        assert!(matches!(d, Some(Zval::Long(1))));
        assert!(b.may_hold_containers(), "hit mirrors get_mut's flag");
        // Miss with a scalar value: holds_containers stays FALSE (insert
        // semantics — no spurious Null vivify).
        let mut c = PhpArray::new();
        let d = c.set_returning_displaced(Key::from_bytes(b"x"), Zval::Long(3));
        assert!(d.is_none());
        assert!(!c.may_hold_containers(), "miss keeps scalar-only flag");
        assert_eq!(c.len(), 1);
        // Miss on a packed tombstone escalates and appends at the END.
        let mut e = PhpArray::new();
        for i in 0..3 {
            let _ = e.append(Zval::Long(i));
        }
        e.remove(&Key::Int(1));
        let d = e.set_returning_displaced(Key::Int(1), Zval::Long(7));
        assert!(d.is_none());
        let order: Vec<_> = e.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(format!("{order:?}"), format!("{:?}", [Key::Int(0), Key::Int(2), Key::Int(1)]));
    }

    fn k(s: &str) -> Key {
        Key::from_bytes(s.as_bytes())
    }

    fn is_packed(a: &PhpArray) -> bool {
        matches!(a.repr, Repr::Packed(_))
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
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
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
        a.insert(k("s"), Zval::Long(-1)); // force hashed repr
        a.remove(&k("s"));
        for i in 0..20 {
            a.insert(Key::Int(i), Zval::Long(i));
        }
        for i in 0..15 {
            a.remove(&Key::Int(i));
        }
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(
            keys,
            (15..20).map(Key::Int).collect::<Vec<_>>()
        );
        assert!(matches!(a.get(&Key::Int(17)), Some(Zval::Long(17))));
    }

    // --- Dual-representation (packed/hashed) behavior ---

    #[test]
    fn dense_int_arrays_stay_packed() {
        let mut a = PhpArray::new();
        for i in 0..100 {
            a.append(Zval::Long(i)).unwrap();
        }
        assert!(is_packed(&a));
        a.insert(Key::Int(50), Zval::Long(-50)); // in-place update
        assert!(is_packed(&a));
        assert!(matches!(a.get(&Key::Int(50)), Some(Zval::Long(-50))));
        // Explicit dense writes also stay packed:
        let mut b = PhpArray::new();
        for i in 0..10 {
            b.insert(Key::Int(i), Zval::Long(i));
        }
        assert!(is_packed(&b));
    }

    #[test]
    fn string_key_escalates_preserving_order() {
        let mut a = PhpArray::new();
        for i in 0..3 {
            a.append(Zval::Long(i)).unwrap();
        }
        a.insert(k("x"), Zval::Long(99));
        assert!(!is_packed(&a));
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(1), Key::Int(2), k("x")]);
        assert!(matches!(a.get(&Key::Int(1)), Some(Zval::Long(1))));
    }

    #[test]
    fn tombstone_reinsert_goes_to_end_like_oracle() {
        // unset($a[1]); $a[1] = 99  =>  iteration order 0, 2, 1 (oracle-pinned).
        let mut a = PhpArray::new();
        for i in 10..13 {
            a.append(Zval::Long(i)).unwrap();
        }
        a.remove(&Key::Int(1));
        assert!(is_packed(&a)); // interior tombstone keeps packed
        a.insert(Key::Int(1), Zval::Long(99));
        assert!(!is_packed(&a)); // re-insert into tombstone escalates
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(2), Key::Int(1)]);
    }

    #[test]
    fn hole_escalates() {
        let mut a = PhpArray::new();
        a.append(Zval::Long(0)).unwrap();
        a.insert(Key::Int(5), Zval::Long(5)); // hole 1..4
        assert!(!is_packed(&a));
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(5)]);
        a.append(Zval::Long(6)).unwrap();
        assert!(a.contains_key(&Key::Int(6)));
    }

    #[test]
    fn negative_key_escalates() {
        let mut a = PhpArray::new();
        a.append(Zval::Long(0)).unwrap();
        a.insert(Key::Int(-1), Zval::Long(-1));
        assert!(!is_packed(&a));
        assert!(matches!(a.get(&Key::Int(-1)), Some(Zval::Long(-1))));
    }

    #[test]
    fn pop_then_append_stays_packed_and_reuses_key() {
        // array_pop + [] reuses the key and the array stays packed.
        let mut a = PhpArray::new();
        for i in 0..3 {
            a.append(Zval::Long(i)).unwrap();
        }
        let popped = a.remove(&Key::Int(2)).unwrap();
        assert!(matches!(popped, Zval::Long(2)));
        a.pop_adjust_next_free(&Key::Int(2));
        a.append(Zval::Long(40)).unwrap();
        assert!(is_packed(&a));
        assert!(matches!(a.get(&Key::Int(2)), Some(Zval::Long(40))));
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(1), Key::Int(2)]);
    }

    #[test]
    fn unset_last_without_adjust_appends_next_key() {
        // unset($a[2]); $a[] = v  =>  key 3 (next_free never decreases).
        let mut a = PhpArray::new();
        for i in 0..3 {
            a.append(Zval::Long(i)).unwrap();
        }
        a.remove(&Key::Int(2));
        a.append(Zval::Long(4)).unwrap();
        assert!(a.contains_key(&Key::Int(3)));
        assert!(!a.contains_key(&Key::Int(2)));
        let keys: Vec<_> = a.iter().map(|(key, _)| key).collect();
        assert_eq!(keys, vec![Key::Int(0), Key::Int(1), Key::Int(3)]);
    }

    #[test]
    fn cursor_survives_escalation() {
        let mut a = PhpArray::new();
        for i in 0..4 {
            a.append(Zval::Long(i)).unwrap();
        }
        a.ptr_next(); // cursor at position 1
        assert_eq!(a.ptr_key(), Some(Key::Int(1)));
        a.insert(k("s"), Zval::Long(9)); // escalates
        assert!(!is_packed(&a));
        assert_eq!(a.ptr_key(), Some(Key::Int(1)));
        assert!(matches!(a.ptr_next(), Some(Zval::Long(2))));
    }

    #[test]
    fn packed_iter_rev_and_ptr_ops() {
        let mut a = PhpArray::new();
        for i in 0..5 {
            a.append(Zval::Long(i)).unwrap();
        }
        a.remove(&Key::Int(1));
        let back: Vec<_> = a.iter().rev().map(|(key, _)| key).collect();
        assert_eq!(back, vec![Key::Int(4), Key::Int(3), Key::Int(2), Key::Int(0)]);
        assert!(matches!(a.ptr_end(), Some(Zval::Long(4))));
        assert!(matches!(a.ptr_prev(), Some(Zval::Long(3))));
        assert_eq!(a.ptr_key(), Some(Key::Int(3)));
    }
}
