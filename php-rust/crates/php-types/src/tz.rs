//! IANA timezone support (D-DT3): a minimal TZif v2/v3 reader over the
//! system's `/usr/share/zoneinfo`, plus the process-wide default timezone.
//!
//! PHP embeds timelib with its own tzdata copy; the system files carry the
//! same rules for the modern era, and phpr's scope is table-backed lookups
//! (epochs past the last stored transition fall back to the last matching
//! type — a documented divergence that starts in 2037 for DST zones).
//!
//! One phpr process models one PHP request, so `thread_local` state is the
//! right lifetime for both the default zone and the parse cache.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const ZONEINFO_DIR: &str = "/usr/share/zoneinfo";

/// Offset info of one instant in one zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TzInfo {
    /// Seconds east of UTC (Toronto summer = -14400).
    pub off: i64,
    /// Zone abbreviation as stored in the TZif designations ("EDT", "HST").
    pub abbrev: String,
    pub isdst: bool,
}

struct TzType {
    off: i64,
    isdst: bool,
    abbrev: String,
}

struct Zone {
    /// Transition instants (UTC epochs), ascending.
    trans: Vec<i64>,
    /// Type index in effect FROM `trans[i]` (same length as `trans`).
    trans_type: Vec<usize>,
    types: Vec<TzType>,
    /// Type in effect before the first transition (RFC 8536 suggests the
    /// first standard-time type; timelib agrees for the zones phpr reads).
    first_type: usize,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<Rc<Zone>>>> = RefCell::new(HashMap::new());
    static DEFAULT_TZ: RefCell<String> = RefCell::new(String::from("UTC"));
}

/// The request-wide default timezone (what `date_default_timezone_get`
/// returns and every local-time builtin renders in).
pub fn default_timezone() -> String {
    DEFAULT_TZ.with(|t| t.borrow().clone())
}

/// Install `name` as the default timezone. Returns `false` (state untouched)
/// for anything that is not a readable IANA zone — the caller emits PHP's
/// "Timezone ID '%s' is invalid" notice.
pub fn set_default_timezone(name: &str) -> bool {
    if zone(name).is_none() {
        return false;
    }
    DEFAULT_TZ.with(|t| *t.borrow_mut() = name.to_string());
    true
}

/// Whether `name` is a loadable IANA zone identifier.
pub fn is_valid_zone(name: &str) -> bool {
    zone(name).is_some()
}

/// Offset/abbreviation/DST of `epoch` in the named zone. `None` for an
/// unknown zone.
pub fn offset_at(name: &str, epoch: i64) -> Option<TzInfo> {
    offset_at_ex(name, epoch).map(|(i, _)| i)
}

/// Like [`offset_at`], plus the START instant of the interval containing
/// `epoch` (the last transition at or before it; `i64::MIN` before the first
/// transition) — timelib's `timelib_get_time_zone_offset_info`
/// `transition_time` out-param, needed by the DST corrections in
/// `timelib_diff_with_tzid`.
pub fn offset_at_ex(name: &str, epoch: i64) -> Option<(TzInfo, i64)> {
    let z = zone(name)?;
    let (ty, start) = match z.trans.partition_point(|&t| t <= epoch) {
        0 => (z.first_type, i64::MIN),
        n => (z.trans_type[n - 1], z.trans[n - 1]),
    };
    let t = &z.types[ty];
    Some((TzInfo { off: t.off, abbrev: t.abbrev.clone(), isdst: t.isdst }, start))
}

/// Transitions of `name` within `[begin, end]`, PHP `getTransitions`-shaped:
/// first the state AT `begin` (its `ts` is `begin` itself), then every
/// transition instant inside the range. `None` for an unknown zone.
pub fn transitions_between(name: &str, begin: i64, end: i64) -> Option<Vec<(i64, TzInfo)>> {
    let z = zone(name)?;
    let info = |ty: usize| {
        let t = &z.types[ty];
        TzInfo { off: t.off, abbrev: t.abbrev.clone(), isdst: t.isdst }
    };
    let idx = z.trans.partition_point(|&t| t <= begin);
    let cur = if idx == 0 { z.first_type } else { z.trans_type[idx - 1] };
    let mut out = vec![(begin, info(cur))];
    for i in idx..z.trans.len() {
        let t = z.trans[i];
        if t > end {
            break;
        }
        out.push((t, info(z.trans_type[i])));
    }
    Some(out)
}

/// Convert a wall-clock time (`wall` = the civil fields packed as if they
/// were UTC) in the named zone to the real UTC epoch, resolving DST gaps and
/// folds the way timelib does (oracle-pinned, America/Toronto 2026):
///   - fold (repeated hour): the FIRST occurrence wins (pre-transition offset);
///   - gap (skipped hour): the pre-transition offset applies, so the result
///     lands after the jump ("02:30" formats back as "03:30 EDT").
/// Both cases reduce to "use the offset of the last interval whose wall-clock
/// START is at or before `wall`, unless the PREVIOUS interval's wall-clock
/// range still covers `wall` (then the previous interval wins)".
pub fn wall_to_epoch(name: &str, wall: i64) -> Option<i64> {
    let z = zone(name)?;
    // Interval i covers epochs [start(i), start(i+1)) with offset off(i);
    // i == 0 is the pre-first-transition era.
    let n = z.trans.len();
    let off = |i: usize| -> i64 {
        if i == 0 { z.types[z.first_type].off } else { z.types[z.trans_type[i - 1]].off }
    };
    let start = |i: usize| -> i64 { if i == 0 { i64::MIN } else { z.trans[i - 1] } };
    // Last interval whose wall start <= wall.
    let mut i = n; // candidate interval index, scanned downward
    while i > 0 {
        let s = start(i);
        if s.saturating_add(off(i)) <= wall {
            break;
        }
        i -= 1;
    }
    // Fold: the previous interval's wall range ends at start(i) + off(i-1);
    // if that is still beyond `wall`, both cover it and the earlier one wins.
    if i > 0 && wall < start(i).saturating_add(off(i - 1)) {
        i -= 1;
    }
    Some(wall - off(i))
}

fn zone(name: &str) -> Option<Rc<Zone>> {
    if !valid_zone_name(name) {
        return None;
    }
    CACHE.with(|c| {
        if let Some(hit) = c.borrow().get(name) {
            return hit.clone();
        }
        // "UTC" is synthesized, not read: the engine's fallback zone must
        // exist even where the zoneinfo db is absent or trimmed.
        let parsed = if name == "UTC" {
            Some(Rc::new(Zone {
                trans: Vec::new(),
                trans_type: Vec::new(),
                types: vec![TzType { off: 0, isdst: false, abbrev: String::from("UTC") }],
                first_type: 0,
            }))
        } else {
            std::fs::read(format!("{ZONEINFO_DIR}/{name}"))
                .ok()
                .and_then(|data| parse_tzif(&data))
                .map(Rc::new)
        };
        c.borrow_mut().insert(name.to_string(), parsed.clone());
        parsed
    })
}

/// Zone identifiers are `Area/Location` words: reject anything that could
/// escape the zoneinfo directory or hit a non-zone file.
fn valid_zone_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 || name.starts_with('/') || name.contains("..") {
        return false;
    }
    name.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'_' | b'-' | b'+'))
}

fn be32(b: &[u8]) -> i64 {
    i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64
}

fn be64(b: &[u8]) -> i64 {
    i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

struct Header {
    isutcnt: usize,
    isstdcnt: usize,
    leapcnt: usize,
    timecnt: usize,
    typecnt: usize,
    charcnt: usize,
    version: u8,
}

fn parse_header(d: &[u8]) -> Option<Header> {
    if d.len() < 44 || &d[..4] != b"TZif" {
        return None;
    }
    let cnt = |i: usize| -> Option<usize> {
        let v = be32(&d[20 + i * 4..]);
        usize::try_from(v).ok()
    };
    Some(Header {
        version: d[4],
        isutcnt: cnt(0)?,
        isstdcnt: cnt(1)?,
        leapcnt: cnt(2)?,
        timecnt: cnt(3)?,
        typecnt: cnt(4)?,
        charcnt: cnt(5)?,
    })
}

impl Header {
    fn block_len(&self, time_size: usize) -> usize {
        self.timecnt * time_size
            + self.timecnt
            + self.typecnt * 6
            + self.charcnt
            + self.leapcnt * (time_size + 4)
            + self.isstdcnt
            + self.isutcnt
    }
}

fn parse_tzif(data: &[u8]) -> Option<Zone> {
    let h1 = parse_header(data)?;
    let (h, body, time_size) = if h1.version >= b'2' {
        // Skip the legacy 32-bit block; the real data is the second
        // (64-bit) header+block.
        let second = 44 + h1.block_len(4);
        let h2 = parse_header(data.get(second..)?)?;
        (h2, data.get(second + 44..)?, 8)
    } else {
        (h1, data.get(44..)?, 4)
    };
    if h.typecnt == 0 || body.len() < h.block_len(time_size) {
        return None;
    }
    let mut pos = 0;
    let mut trans = Vec::with_capacity(h.timecnt);
    for i in 0..h.timecnt {
        let at = pos + i * time_size;
        trans.push(if time_size == 8 { be64(&body[at..]) } else { be32(&body[at..]) });
    }
    pos += h.timecnt * time_size;
    let mut trans_type = Vec::with_capacity(h.timecnt);
    for i in 0..h.timecnt {
        let t = body[pos + i] as usize;
        if t >= h.typecnt {
            return None;
        }
        trans_type.push(t);
    }
    pos += h.timecnt;
    let chars_at = pos + h.typecnt * 6;
    let chars = &body[chars_at..chars_at + h.charcnt];
    let mut types = Vec::with_capacity(h.typecnt);
    for i in 0..h.typecnt {
        let rec = &body[pos + i * 6..pos + i * 6 + 6];
        let idx = rec[5] as usize;
        if idx > chars.len() {
            return None;
        }
        let end = chars[idx..].iter().position(|&b| b == 0).map_or(chars.len(), |p| idx + p);
        types.push(TzType {
            off: be32(rec),
            isdst: rec[4] != 0,
            abbrev: String::from_utf8_lossy(&chars[idx..end]).into_owned(),
        });
    }
    // Pre-first-transition era: the first standard-time type (RFC 8536 §3.2
    // fallback), or type 0.
    let first_type = types.iter().position(|t| !t.isdst).unwrap_or(0);
    Some(Zone { trans, trans_type, types, first_type })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Every pinned value below comes from the PHP 8.5.7 oracle running with
    // the named zone as its default (probe p7_tz1.php, sessione 7).

    #[test]
    fn toronto_summer_winter() {
        let s = offset_at("America/Toronto", 1718452845).unwrap(); // 2024-06-15
        assert_eq!((s.off, s.abbrev.as_str(), s.isdst), (-14400, "EDT", true));
        let w = offset_at("America/Toronto", 1700000000).unwrap(); // 2023-11-14
        assert_eq!((w.off, w.abbrev.as_str(), w.isdst), (-18000, "EST", false));
    }

    #[test]
    fn honolulu_fixed() {
        let h = offset_at("Pacific/Honolulu", 1342864800).unwrap();
        assert_eq!((h.off, h.abbrev.as_str(), h.isdst), (-36000, "HST", false));
    }

    #[test]
    fn utc_zone() {
        let u = offset_at("UTC", 0).unwrap();
        assert_eq!((u.off, u.abbrev.as_str(), u.isdst), (0, "UTC", false));
    }

    #[test]
    fn wall_plain() {
        // 2012-07-21 00:00:00 Honolulu = 1342864800 (oracle).
        assert_eq!(wall_to_epoch("Pacific/Honolulu", 1342828800), Some(1342864800));
        // UTC is the identity.
        assert_eq!(wall_to_epoch("UTC", 1342828800), Some(1342828800));
    }

    #[test]
    fn wall_gap_uses_pre_transition_offset() {
        // Toronto 2026-03-08 02:30:00 does not exist; strtotime → 1772955000
        // (renders as 03:30 EDT). Wall 02:30 packed as UTC = 1772937000.
        assert_eq!(wall_to_epoch("America/Toronto", 1772937000), Some(1772955000));
    }

    #[test]
    fn wall_fold_first_occurrence_wins() {
        // Toronto 2026-11-01 01:30:00 happens twice; strtotime → 1793511000
        // (the EDT one). Wall 01:30 packed as UTC = 1793496600.
        assert_eq!(wall_to_epoch("America/Toronto", 1793496600), Some(1793511000));
    }

    #[test]
    fn wall_unambiguous_near_transitions() {
        // 2026-03-08 03:30 EDT (just after the gap) = 1772955000; its wall
        // packed as UTC is 1772940600.
        assert_eq!(wall_to_epoch("America/Toronto", 1772940600), Some(1772955000));
        // 2026-11-01 00:30 EDT (before the fold) = 1793507400.
        assert_eq!(wall_to_epoch("America/Toronto", 1793493000), Some(1793507400));
        // 2026-11-01 02:30 EST (after the fold) = 1793514600 + 3600?  02:30
        // wall = 1793500200; EST offset -18000 → 1793518200.
        assert_eq!(wall_to_epoch("America/Toronto", 1793500200), Some(1793518200));
    }

    #[test]
    fn default_timezone_roundtrip() {
        assert_eq!(default_timezone(), "UTC");
        assert!(set_default_timezone("America/Toronto"));
        assert_eq!(default_timezone(), "America/Toronto");
        assert!(!set_default_timezone("Bogus/Zone"));
        assert_eq!(default_timezone(), "America/Toronto");
        assert!(set_default_timezone("UTC"));
    }

    #[test]
    fn invalid_names_rejected() {
        let long = "A".repeat(65);
        for n in ["", "../etc/passwd", "/etc/passwd", "posixrules\0", long.as_str()] {
            assert!(offset_at(n, 0).is_none(), "{n:?} accepted");
        }
        assert!(offset_at("Bogus/Zone", 0).is_none());
    }
}
