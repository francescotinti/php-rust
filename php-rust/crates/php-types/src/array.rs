use std::rc::Rc;

use crate::{PhpStr, Zval};

/// Outcome of [`PhpArray::set_returning_displaced`] (H-70.1, WP-70). `Done`
/// is the write as performed (through an existing `Ref` slot or in place),
/// carrying the displaced value for GC noting. `Busy` means the slot is a
/// `Ref` whose cell is currently borrowed — a reference cycle mid-statement:
/// nothing was written, and the CALLER owns the store (through the returned
/// cell) once its guards drop.
pub enum LeafWrite {
    Done(Option<Zval>),
    Busy(Rc<std::cell::RefCell<Zval>>, Zval),
}

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
        index: KeyIndex,
    },
}

/// Slot markers of [`KeyIndex`]: entry positions are `< SLOT_TOMB`.
const SLOT_EMPTY: u32 = u32::MAX;
const SLOT_TOMB: u32 = u32::MAX - 1;

/// Bit layout of `PhpArray::cur_holds` (WP-58) — see the field doc.
const HOLDS_BIT: u32 = 1 << 31;
const CURSOR_MASK: u32 = HOLDS_BIT - 1;

/// WP-58 (leva C): hashed arrays whose entry table (live + tombstones) fits
/// in this many slots carry NO index at all (`KeyIndex::slots` empty) — a
/// lookup linearly scans the entries, which is the same key comparison the
/// probe path performs on its candidate, minus the mix/probe/maintenance
/// and minus the 32B+ slots allocation. The dominant hashed population
/// (WP-57 histogram: 21,4k of 26,7k standing arrays hold 1-4 elements)
/// never builds an index. Not observable: iteration order lives in
/// `entries`, and a key is unique by invariant, so scan and probe find the
/// same slot. The index materializes once the table would exceed this
/// bound and only goes away again through `build` (compaction).
// WP-59 Ob.3 asse 2: binario diagnostico `scan4` per l'attribuzione della
// regressione full-only (il pool-off ne ha spiegato ~0,6% su +1..+2,5%).
const SCAN_MAX: usize = if cfg!(feature = "scan4") { 4 } else { 8 };

/// Murmur3 fmix64: spreads the (cached) key hash over all bits so the
/// power-of-two mask sees a uniform distribution.
#[inline]
fn mix(h: u64) -> u64 {
    let mut x = h;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    x ^ (x >> 33)
}

impl Key {
    /// Index hash without storing the key: ints hash by value, strings reuse
    /// the per-string cached DJBX33A (WP-29) — a key string hashes once in
    /// its lifetime, exactly like the former FxHashMap `Hash` impl.
    #[inline]
    fn khash(&self) -> u64 {
        match self {
            Key::Int(i) => mix(*i as u64),
            Key::Str(s) => mix(s.zhash()),
        }
    }
}

/// The abbreviated entry-slice type the index probes against.
type Entries = [Option<(Key, Zval)>];

/// Keyless position index of the hashed representation (WP-56, Fase 3):
/// an open-addressing table of `u32` positions into `entries` — the key
/// itself lives only in the entry it points at, mirroring Zend's
/// arData + uint32 hash slots (no duplicated key). Insertion order lives
/// in `entries`, so neither the hash function nor the probe order is
/// observable (same invariant the former FxHashMap index had).
///
/// Linear probing over a power-of-two table, occupancy (live + tombstones)
/// kept ≤ 1/2 by [`KeyIndex::rebuild`]; index tombstones only appear on
/// `remove` and are reclaimed by the next rebuild. Invariant: every
/// non-sentinel slot points at a live (`Some`) entry.
#[derive(Debug, Clone)]
struct KeyIndex {
    slots: Box<[u32]>,
    /// Occupied slots (== the array's live count).
    live: u32,
    /// Tombstoned slots — counted like occupied ones by the rebuild
    /// trigger so probe chains stay short under insert/remove churn.
    tomb: u32,
}

/// Table size for `live` keys: power of two with occupancy ≤ 1/2, min 8.
fn table_cap(live: usize) -> usize {
    (live * 2 + 1).next_power_of_two().max(8)
}

impl KeyIndex {
    /// Build a fresh index over the live entries (escalation, compaction).
    /// A table of at most [`SCAN_MAX`] slots stays in scan mode (no slots
    /// allocation — see the constant's doc).
    fn build(entries: &Entries, live: usize) -> KeyIndex {
        if entries.len() <= SCAN_MAX {
            return KeyIndex {
                slots: Box::new([]),
                live: entries.iter().filter(|e| e.is_some()).count() as u32,
                tomb: 0,
            };
        }
        let mut idx = KeyIndex {
            slots: vec![SLOT_EMPTY; table_cap(live)].into_boxed_slice(),
            live: 0,
            tomb: 0,
        };
        for (pos, e) in entries.iter().enumerate() {
            if let Some((k, _)) = e {
                idx.raw_insert(k.khash(), pos as u32);
                idx.live += 1;
            }
        }
        idx
    }

    /// Probe to the first EMPTY slot and write `pos` (rebuild/build path:
    /// the key is known absent and the table holds no tombstones).
    #[inline]
    fn raw_insert(&mut self, khash: u64, pos: u32) {
        let mask = self.slots.len() - 1;
        let mut i = (khash as usize) & mask;
        while self.slots[i] != SLOT_EMPTY {
            i = (i + 1) & mask;
        }
        self.slots[i] = pos;
    }

    /// Position of `key`, comparing candidate slots against the entry they
    /// point at (the single stored copy of the key).
    #[inline]
    fn lookup(&self, key: &Key, entries: &Entries) -> Option<u32> {
        if self.slots.is_empty() {
            // Scan mode (≤ SCAN_MAX slots): compare keys in entry order.
            return entries.iter().enumerate().find_map(|(i, e)| match e {
                Some((k, _)) if k == key => Some(i as u32),
                _ => None,
            });
        }
        let mask = self.slots.len() - 1;
        let mut i = (key.khash() as usize) & mask;
        loop {
            match self.slots[i] {
                SLOT_EMPTY => return None,
                SLOT_TOMB => {}
                pos => {
                    let (k, _) = entries[pos as usize].as_ref().unwrap();
                    if k == key {
                        return Some(pos);
                    }
                }
            }
            i = (i + 1) & mask;
        }
    }

    /// Insert a key known to be ABSENT (callers always probe first), to be
    /// pushed at `pos` — call BEFORE or AFTER the entry push: a triggered
    /// rebuild rehashes from the OLD slots, never rescans `entries`, so a
    /// pushed-but-unindexed entry cannot be double-added.
    fn insert_new(&mut self, key: &Key, pos: u32, entries: &Entries) {
        if self.slots.is_empty() {
            self.live += 1;
            if (pos as usize) < SCAN_MAX {
                return; // still within the scan bound after this insert
            }
            // Crossing SCAN_MAX: materialize the table over the live
            // entries plus the incoming (key, pos). The incoming entry may
            // or may not be pushed yet (both call orders are legal), so
            // position `pos` is skipped in the sweep and added explicitly.
            self.slots =
                vec![SLOT_EMPTY; table_cap(self.live as usize)].into_boxed_slice();
            self.tomb = 0;
            for (i, e) in entries.iter().enumerate() {
                if i as u32 != pos {
                    if let Some((k, _)) = e {
                        self.raw_insert(k.khash(), i as u32);
                    }
                }
            }
            self.raw_insert(key.khash(), pos);
            return;
        }
        if ((self.live + self.tomb + 1) as usize) * 2 > self.slots.len() {
            self.rebuild(entries);
        }
        let mask = self.slots.len() - 1;
        let mut i = (key.khash() as usize) & mask;
        loop {
            match self.slots[i] {
                SLOT_EMPTY => {
                    self.slots[i] = pos;
                    self.live += 1;
                    return;
                }
                SLOT_TOMB => {
                    self.slots[i] = pos;
                    self.live += 1;
                    self.tomb -= 1;
                    return;
                }
                _ => i = (i + 1) & mask,
            }
        }
    }

    /// Tombstone `key`'s slot; returns the entry position it held.
    fn remove(&mut self, key: &Key, entries: &Entries) -> Option<u32> {
        if self.slots.is_empty() {
            // Scan mode: no slot to tombstone, just the live count.
            let pos = entries.iter().enumerate().find_map(|(i, e)| match e {
                Some((k, _)) if k == key => Some(i as u32),
                _ => None,
            })?;
            self.live -= 1;
            return Some(pos);
        }
        let mask = self.slots.len() - 1;
        let mut i = (key.khash() as usize) & mask;
        loop {
            match self.slots[i] {
                SLOT_EMPTY => return None,
                SLOT_TOMB => {}
                pos => {
                    let (k, _) = entries[pos as usize].as_ref().unwrap();
                    if k == key {
                        self.slots[i] = SLOT_TOMB;
                        self.live -= 1;
                        self.tomb += 1;
                        return Some(pos);
                    }
                }
            }
            i = (i + 1) & mask;
        }
    }

    /// Re-size for the current live count (dropping tombstones), rehashing
    /// the positions held by the OLD table — `entries` is only read at those
    /// positions, which are live by invariant.
    #[cold]
    fn rebuild(&mut self, entries: &Entries) {
        let old = std::mem::replace(
            &mut self.slots,
            vec![SLOT_EMPTY; table_cap(self.live as usize + 1)].into_boxed_slice(),
        );
        self.tomb = 0;
        for &pos in old.iter() {
            if pos < SLOT_TOMB {
                let (k, _) = entries[pos as usize].as_ref().unwrap();
                self.raw_insert(k.khash(), pos);
            }
        }
    }
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
    ///
    /// Packed word (WP-58 header diet): bits 0..31 = the cursor position,
    /// bit 31 = the `holds_containers` flag (doc below). Positions are
    /// already bounded to u32 by [`KeyIndex`] (`SLOT_TOMB`), `count` is u32,
    /// and a table past 2G slots is physically unreachable (32GB of slots);
    /// saturating casts keep "past the end" meaning past the end even in
    /// that impossible world. The flag rides the COLD word (cursor is only
    /// touched by the ptr_* builtins), never `count`/`len()`. This drops the
    /// `Rc<RefCell<PhpArray>>` heap block from 104B (mimalloc bin 112) to
    /// 96B (exact bin 96): −16B on every array allocation.
    cur_holds: u32,
    /// WP-57 (census builds only): bytes currently credited to the LIVE
    /// arr counter for THIS array. Synced to `census_bytes()` at the top of
    /// every mutating method (lag ≤ the array's most recent capacity step);
    /// `Drop` frees exactly this figure, so a missed sync can never drift
    /// the channel negative.
    #[cfg(feature = "mem-census")]
    accounted: std::cell::Cell<usize>,
    // `holds_containers` (bit 31 of `cur_holds`): conservative container-
    // content marker — `false` only when every value ever stored is a
    // scalar/string AND no `&mut` element handle was ever handed out
    // (a caller could promote a scalar in place). Lets the GC's drop-descent
    // and cycle classify skip scalar-only arrays without iterating them.
    // Never cleared by `remove` — stays pessimistic once set.
    /// Cycle-classify in-node mark (WP-52) — see [`crate::object::WalkMark`].
    /// Epoch-guarded: stale between walks by construction, so no reset pass;
    /// `Clone` (COW) starts fresh — a copy is a new node.
    walk: crate::object::WalkMark,
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
        let new = PhpArray {
            repr,
            next_free: self.next_free,
            count: self.count,
            cur_holds: self.cur_holds,
            #[cfg(feature = "mem-census")]
            accounted: std::cell::Cell::new(0),
            walk: crate::object::WalkMark::new(),
        };
        #[cfg(feature = "mem-census")]
        {
            let cb = new.census_bytes();
            crate::memcensus::alloc(crate::memcensus::CH_ARR, cb);
            new.accounted.set(cb);
        }
        new
    }
}

/// WP-57: the ARR channel is now LIVE-accounted exactly (the WP-56 verdict
/// killed the death-avg estimator: churn-shaped deaths overstate standing
/// 5,7×). Every construction funnels through `Default`/`Clone` (private
/// fields), every capacity change through a `&mut self` method that opens
/// with [`PhpArray::census_sync`]; `Drop` frees the per-array `accounted`
/// figure, so the channel can never drift. CUM now accumulates
/// alloc + positive adjusts (same convention as the str channel) — the
/// old `death()` feed is gone with the estimator.
#[cfg(feature = "mem-census")]
impl Drop for PhpArray {
    fn drop(&mut self) {
        crate::memcensus::free(crate::memcensus::CH_ARR, self.accounted.get());
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
                    + index.slots.len() * std::mem::size_of::<u32>()
            }
        };
        body + crate::memcensus::ARR_OVERHEAD
    }

    /// Re-credit this array's current capacity to the LIVE arr counter
    /// (delta vs the last sync). Called at the TOP of every mutating
    /// method: the lag is at most the array's own most recent capacity
    /// step, and `Drop` reconciles whatever is left.
    #[inline]
    pub(crate) fn census_sync(&self) {
        let cb = self.census_bytes();
        let delta = cb as i64 - self.accounted.get() as i64;
        if delta != 0 {
            crate::memcensus::adjust(crate::memcensus::CH_ARR, delta);
            self.accounted.set(cb);
        }
    }

    /// Census shape row for the per-repr histogram: (is_packed, live count,
    /// capacity bytes) — read by the EOR reached-walk.
    pub fn census_shape(&self) -> (bool, u32, usize) {
        (matches!(self.repr, Repr::Packed(_)), self.count, self.census_bytes())
    }
}

impl Default for PhpArray {
    fn default() -> Self {
        let new = PhpArray {
            repr: Repr::Packed(Vec::new()),
            next_free: i64::MIN,
            count: 0,
            cur_holds: 0,
            #[cfg(feature = "mem-census")]
            accounted: std::cell::Cell::new(0),
            walk: crate::object::WalkMark::new(),
        };
        #[cfg(feature = "mem-census")]
        {
            let cb = new.census_bytes();
            crate::memcensus::alloc(crate::memcensus::CH_ARR, cb);
            new.accounted.set(cb);
        }
        new
    }
}

impl PhpArray {
    /// The classify-walk mark carried by this array (WP-52).
    #[inline]
    pub fn walk_mark(&self) -> &crate::object::WalkMark {
        &self.walk
    }

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
    /// other containers — see the `holds_containers` bit doc on `cur_holds`.
    /// `false` is a guarantee; `true` only means "must be walked".
    #[inline]
    pub fn may_hold_containers(&self) -> bool {
        self.cur_holds & HOLDS_BIT != 0
    }

    /// The cursor position carried in `cur_holds` — see the field doc.
    #[inline]
    fn cursor(&self) -> usize {
        (self.cur_holds & CURSOR_MASK) as usize
    }

    /// Store a cursor position, saturating into the 31-bit field (a real
    /// position never reaches the clamp — see the `cur_holds` doc).
    #[inline]
    fn set_cursor(&mut self, pos: usize) {
        self.cur_holds =
            (self.cur_holds & HOLDS_BIT) | pos.min(CURSOR_MASK as usize) as u32;
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
        let index = KeyIndex::build(&entries, self.count as usize);
        self.repr = Repr::Hashed { entries, index };
    }

    /// Insert or update. Updating an existing key keeps its position.
    pub fn insert(&mut self, key: Key, val: Zval) {
        #[cfg(feature = "mem-census")]
        self.census_sync();
        self.cur_holds |= (!matches!(
            val,
            Zval::Undef | Zval::Null | Zval::Bool(_) | Zval::Long(_) | Zval::Double(_) | Zval::Str(_)
        ) as u32)
            << 31;
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
        if let Some(pos) = index.lookup(&key, entries) {
            entries[pos as usize] = Some((key, val));
            return;
        }
        if let Key::Int(i) = key {
            if i >= self.next_free {
                self.next_free = i.saturating_add(1);
            }
        }
        let pos = entries.len() as u32;
        index.insert_new(&key, pos, entries);
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
        #[cfg(feature = "mem-census")]
        self.census_sync();
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
                    self.cur_holds |= HOLDS_BIT;
                    let Repr::Packed(slots) = &mut self.repr else { unreachable!() };
                    return slots[i].as_mut().unwrap();
                }
                Plan::Append(i) => {
                    if i >= self.next_free {
                        self.next_free = i.saturating_add(1);
                    }
                    self.count += 1;
                    self.cur_holds |= HOLDS_BIT;
                    let Repr::Packed(slots) = &mut self.repr else { unreachable!() };
                    slots.push(Some(Zval::Null));
                    return slots.last_mut().unwrap().as_mut().unwrap();
                }
                Plan::Escalate => self.to_hashed(),
            }
        }
        self.cur_holds |= HOLDS_BIT;
        let hit = {
            let Repr::Hashed { entries, index } = &self.repr else { unreachable!() };
            index.lookup(&key, entries)
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
                index.insert_new(&key, pos, entries);
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
    pub fn set_returning_displaced(&mut self, key: Key, val: Zval) -> LeafWrite {
        #[cfg(feature = "mem-census")]
        self.census_sync();
        fn write_slot(slot: &mut Zval, val: Zval) -> Result<Zval, LeafWrite> {
            match slot {
                // H-70.1 (WP-70): the slot's cell can be mid-write on this
                // very statement — a reference cycle closing at the leaf
                // (`$b[0] = "x"` with `$a[0] = &$a`, `$b` aliasing `$a`)
                // reaches here while the walker's guard on the same cell is
                // still alive. Never a naked `borrow_mut` on a walk path:
                // hand the write back to the caller, who owns it once its
                // guards drop.
                Zval::Ref(cell) => match cell.try_borrow_mut() {
                    Ok(mut inner) => Ok(std::mem::replace(&mut *inner, val)),
                    Err(_) => Err(LeafWrite::Busy(Rc::clone(cell), val)),
                },
                _ => Ok(std::mem::replace(slot, val)),
            }
        }
        let hit = match &self.repr {
            Repr::Packed(slots) => match &key {
                Key::Int(i) if (*i as usize) < slots.len() && *i >= 0 && slots[*i as usize].is_some() => {
                    Some(*i as usize)
                }
                _ => None,
            },
            Repr::Hashed { entries, index } => {
                index.lookup(&key, entries).map(|pos| pos as usize)
            }
        };
        match hit {
            Some(pos) => {
                self.cur_holds |= HOLDS_BIT;
                let displaced = match &mut self.repr {
                    Repr::Packed(slots) => write_slot(slots[pos].as_mut().unwrap(), val),
                    Repr::Hashed { entries, .. } => {
                        write_slot(&mut entries[pos].as_mut().unwrap().1, val)
                    }
                };
                match displaced {
                    Ok(d) => LeafWrite::Done(Some(d)),
                    Err(busy) => busy,
                }
            }
            None => {
                self.insert(key, val);
                LeafWrite::Done(None)
            }
        }
    }

    /// Zend's `array_pop` adjustment (ext/standard/array.c:3579): popping the
    /// element whose int key was the latest auto-index (`next_free - 1`)
    /// frees that index again, so pop-then-append reuses the same key.
    pub fn pop_adjust_next_free(&mut self, popped: &Key) {
        #[cfg(feature = "mem-census")]
        self.census_sync();
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
        #[cfg(feature = "mem-census")]
        self.census_sync();
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
        #[cfg(feature = "mem-census")]
        self.census_sync();
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
                .lookup(key, entries)
                .map(|pos| &entries[pos as usize].as_ref().unwrap().1),
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
                            self.cur_holds |= HOLDS_BIT;
                            Some(v)
                        }
                        None => None,
                    }
                }
                _ => None,
            },
            Repr::Hashed { entries, index } => match index.lookup(key, entries) {
                Some(pos) => {
                    self.cur_holds |= HOLDS_BIT;
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
            Repr::Hashed { entries, index } => index.lookup(key, entries).is_some(),
        }
    }

    /// `unset($a[k])`: leaves a tombstone so iteration order is preserved.
    /// `next_free` intentionally not touched (Zend semantics).
    pub fn remove(&mut self, key: &Key) -> Option<Zval> {
        #[cfg(feature = "mem-census")]
        self.census_sync();
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
                let pos = index.remove(key, entries)?;
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
        *index = KeyIndex::build(entries, entries.len());
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
        self.cur_holds |= HOLDS_BIT;
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
        (self.cursor()..self.slots_len()).find(|&i| self.live_at(i))
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
        self.set_cursor(
            (0..self.slots_len())
                .find(|&i| self.live_at(i))
                .unwrap_or(self.slots_len()),
        );
        self.ptr_current()
    }

    /// `end($a)`: move the pointer to the last live entry; return its value.
    pub fn ptr_end(&mut self) -> Option<Zval> {
        self.set_cursor(
            (0..self.slots_len())
                .rev()
                .find(|&i| self.live_at(i))
                .unwrap_or(self.slots_len()),
        );
        self.ptr_current()
    }

    /// `next($a)`: advance the pointer to the next live entry; return its value.
    /// Already past the end stays past the end (`false`).
    pub fn ptr_next(&mut self) -> Option<Zval> {
        let start = match self.cursor_pos() {
            Some(i) => i + 1,
            None => self.slots_len(),
        };
        self.set_cursor(
            (start..self.slots_len())
                .find(|&i| self.live_at(i))
                .unwrap_or(self.slots_len()),
        );
        self.ptr_current()
    }

    /// `prev($a)`: retreat the pointer to the previous live entry; return its value.
    /// Stepping before the first entry invalidates the pointer (`false`).
    pub fn ptr_prev(&mut self) -> Option<Zval> {
        let end = self.cursor_pos().unwrap_or(self.slots_len());
        self.set_cursor(
            (0..end)
                .rev()
                .find(|&i| self.live_at(i))
                .unwrap_or(self.slots_len()),
        );
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
        assert!(matches!(d, LeafWrite::Done(Some(Zval::Long(5)))));
        assert!(matches!(&*cell.borrow(), Zval::Long(9)));
        assert!(matches!(a.get(&Key::from_bytes(b"r")), Some(Zval::Ref(_))));
        // Plain hit: displaced returned, holds_containers set like get_mut.
        let mut b = PhpArray::new();
        let _ = b.append(Zval::Long(1));
        let d = b.set_returning_displaced(Key::Int(0), Zval::Long(2));
        assert!(matches!(d, LeafWrite::Done(Some(Zval::Long(1)))));
        assert!(b.may_hold_containers(), "hit mirrors get_mut's flag");
        // Miss with a scalar value: holds_containers stays FALSE (insert
        // semantics — no spurious Null vivify).
        let mut c = PhpArray::new();
        let d = c.set_returning_displaced(Key::from_bytes(b"x"), Zval::Long(3));
        assert!(matches!(d, LeafWrite::Done(None)));
        assert!(!c.may_hold_containers(), "miss keeps scalar-only flag");
        assert_eq!(c.len(), 1);
        // Miss on a packed tombstone escalates and appends at the END.
        let mut e = PhpArray::new();
        for i in 0..3 {
            let _ = e.append(Zval::Long(i));
        }
        e.remove(&Key::Int(1));
        let d = e.set_returning_displaced(Key::Int(1), Zval::Long(7));
        assert!(matches!(d, LeafWrite::Done(None)));
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

    /// WP-56 keyless index: heavy insert/remove/overwrite churn (mixed
    /// int/string keys, compaction and index rebuilds included) against a
    /// Vec model of Zend order semantics — updates keep position, re-inserts
    /// go to the end; lookups, count and iteration order match at every step.
    #[test]
    fn keyless_index_churn_matches_model() {
        let mut a = PhpArray::new();
        a.insert(k("seed"), Zval::Long(-1)); // hashed from the start
        a.remove(&k("seed"));
        let mut model: Vec<(Key, i64)> = Vec::new();
        let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
        for step in 0..4000i64 {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let r = (x >> 33) % 100;
            let key = if r % 2 == 0 {
                Key::Int((r as i64) % 37)
            } else {
                k(&format!("k{}", r % 37))
            };
            if r < 70 {
                match model.iter_mut().find(|(mk, _)| *mk == key) {
                    Some((_, v)) => *v = step,
                    None => model.push((key.clone(), step)),
                }
                a.insert(key, Zval::Long(step));
            } else {
                let removed = a.remove(&key);
                let had = model.iter().position(|(mk, _)| *mk == key);
                assert_eq!(removed.is_some(), had.is_some(), "step {step}");
                if let Some(i) = had {
                    model.remove(i);
                }
            }
            assert_eq!(a.len(), model.len(), "step {step}");
        }
        let got: Vec<(Key, i64)> = a
            .iter()
            .map(|(kk, v)| match v {
                Zval::Long(n) => (kk, *n),
                other => panic!("non-long value {other:?}"),
            })
            .collect();
        assert_eq!(got, model);
        for (mk, mv) in &model {
            match a.get(mk) {
                Some(Zval::Long(n)) => assert_eq!(n, mv),
                other => panic!("{mk:?}: {other:?}"),
            }
        }
    }

    /// WP-58 leva C: hashed arrays within SCAN_MAX slots carry no index
    /// allocation; the index materializes exactly when the 9th slot
    /// arrives, and correctness holds across the boundary, removals, and
    /// compaction back under the bound.
    #[test]
    fn small_hashed_scan_mode_elides_index() {
        fn idx_len(a: &PhpArray) -> usize {
            match &a.repr {
                Repr::Hashed { index, .. } => index.slots.len(),
                Repr::Packed(_) => panic!("expected hashed"),
            }
        }
        let mut a = PhpArray::new();
        a.insert(k("k0"), Zval::Long(0));
        assert_eq!(idx_len(&a), 0, "small hashed: no index");
        for i in 1..8 {
            a.insert(k(&format!("k{i}")), Zval::Long(i));
        }
        assert_eq!(a.len(), 8);
        assert_eq!(idx_len(&a), 0, "8 slots still scan mode");
        for i in 0..8 {
            assert!(matches!(a.get(&k(&format!("k{i}"))), Some(Zval::Long(_))));
        }
        assert!(a.get(&k("nope")).is_none());
        a.insert(k("k8"), Zval::Long(8)); // 9th slot: materializes
        assert!(idx_len(&a) >= 16, "index materialized past SCAN_MAX");
        for i in 0..9 {
            assert!(matches!(a.get(&k(&format!("k{i}"))), Some(Zval::Long(_))));
        }
        let keys: Vec<_> = a.iter().map(|(kk, _)| kk).collect();
        let want: Vec<_> = (0..9).map(|i| k(&format!("k{i}"))).collect();
        assert_eq!(keys, want, "insertion order across materialization");
        // Deep removal compacts back under the bound → scan mode again.
        for i in 0..7 {
            a.remove(&k(&format!("k{i}")));
        }
        assert_eq!(a.len(), 2);
        assert_eq!(idx_len(&a), 0, "compaction under SCAN_MAX drops the index");
        assert!(matches!(a.get(&k("k7")), Some(Zval::Long(7))));
        assert!(matches!(a.get(&k("k8")), Some(Zval::Long(8))));
        assert!(a.get(&k("k0")).is_none());
    }

    /// WP-58 pin (a): the allocation-side sizes the arena quota rests on.
    /// Printed (not asserted) so a layout drift shows up in --nocapture runs
    /// next to the census constants they must stay reconciled with.
    #[test]
    fn wp58_layout_size_pins() {
        use std::cell::RefCell;
        use std::mem::size_of;
        eprintln!(
            "wp58 sizes: PhpArray={} Repr={} KeyIndex={} RefCell<PhpArray>={} \
             Rc-block(RefCell+16)={} hashed_entry={} packed_slot={} Key={}",
            size_of::<PhpArray>(),
            size_of::<Repr>(),
            size_of::<KeyIndex>(),
            size_of::<RefCell<PhpArray>>(),
            size_of::<RefCell<PhpArray>>() + 16,
            size_of::<Option<(Key, Zval)>>(),
            size_of::<Option<Zval>>(),
            size_of::<Key>(),
        );
        // The entry payloads the whole Fase 3 quota is denominated in.
        assert_eq!(size_of::<Option<(Key, Zval)>>(), 32);
        assert_eq!(size_of::<Option<Zval>>(), 16);
        // WP-58 header diet: the per-array heap block must stay on the
        // exact 96B mimalloc bin (going back to 104 costs +16B/array).
        // Census builds carry the `accounted` Cell (+8B) — parity only.
        #[cfg(not(feature = "mem-census"))]
        assert_eq!(size_of::<RefCell<PhpArray>>() + 16, 96);
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
