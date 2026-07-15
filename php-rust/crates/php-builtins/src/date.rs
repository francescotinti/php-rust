//! Date/time builtins (plan step 34-1): `date()` / `gmdate()` formatting.
//!
//! Calendar arithmetic (leap years, day-of-week, ISO week, ordinal day) is
//! delegated to the pure-Rust `time` crate (D-DT1, Strategy A adapter). The
//! mapping from PHP `date()` format characters to output bytes is written by
//! hand here — PHP's format chars are not the same as `time`'s.
//!
//! Scope (D-DT3): UTC only. Timezone-dependent chars (`e`/`T`/`O`/`P`/`Z`/`I`)
//! always render the UTC values. `now` (omitted timestamp) reads the real
//! clock and is therefore not exercised by the differential tests (D-DT5).

use std::rc::Rc;

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};
use time::{Date, Month, OffsetDateTime};

/// timelib's request-global "last errors" container (`DateTime::getLastErrors`).
/// Subset: refreshed by `createFromFormat` parses only — the textual ctor path
/// does not update it, and the failure messages are the generic pair rather
/// than timelib's per-specifier wording (documented in PHPR_DIVERGENCES).
/// One phpr process models one PHP request, so `thread_local` is the right
/// lifetime (phpt/PHPUnit isolation is per-process).
#[derive(Default, Clone)]
struct LastDateErrors {
    warnings: Vec<(i64, &'static str)>,
    errors: Vec<(i64, &'static str)>,
}
thread_local! {
    static LAST_DATE_ERRORS: std::cell::RefCell<Option<LastDateErrors>> =
        const { std::cell::RefCell::new(None) };
}

// --- Timezone plumbing (D-DT3): every local-time builtin renders in the
// process default zone (php_types::tz), and DateTime carries a zone label
// that resolves here. ---

/// A DateTime-style zone label resolved for math/formatting: UTC, a fixed
/// offset ("+05:00", "-1000", "Z"), or an IANA identifier.
enum ZoneRef {
    Utc,
    Fixed { off: i64, label: String },
    Named(String),
}

/// One instant viewed in one zone; `name` is what `date('e')` prints and
/// `abbrev` what `date('T')` prints.
struct ZoneView {
    off: i64,
    abbrev: String,
    name: String,
    isdst: bool,
}

/// `±HH:MM` / `±HHMM` / `±HH` → seconds east of UTC.
fn parse_offset_label(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 3 || (b[0] != b'+' && b[0] != b'-') {
        return None;
    }
    let digits: String = s[1..].chars().filter(|c| *c != ':').collect();
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (h, m) = match digits.len() {
        2 => (digits.parse::<i64>().ok()?, 0),
        4 => (digits[..2].parse::<i64>().ok()?, digits[2..].parse::<i64>().ok()?),
        _ => return None,
    };
    let sign = if b[0] == b'-' { -1 } else { 1 };
    Some(sign * (h * 3600 + m * 60))
}

fn resolve_zone(label: &str) -> Option<ZoneRef> {
    if label == "UTC" {
        return Some(ZoneRef::Utc);
    }
    if label == "Z" {
        // `getName()` of a `...Z` literal stays "Z" (oracle-pinned).
        return Some(ZoneRef::Fixed { off: 0, label: "Z".to_string() });
    }
    if let Some(off) = parse_offset_label(label) {
        return Some(ZoneRef::Fixed { off, label: label.to_string() });
    }
    if php_types::tz::is_valid_zone(label) {
        return Some(ZoneRef::Named(label.to_string()));
    }
    None
}

fn default_zone() -> ZoneRef {
    resolve_zone(&php_types::tz::default_timezone()).unwrap_or(ZoneRef::Utc)
}

fn zone_view(zr: &ZoneRef, epoch: i64) -> ZoneView {
    match zr {
        ZoneRef::Utc => ZoneView {
            off: 0,
            abbrev: "UTC".to_string(),
            name: "UTC".to_string(),
            isdst: false,
        },
        ZoneRef::Fixed { off, label } => ZoneView {
            off: *off,
            // PHP's `T` for a fixed-offset zone is the "GMT±HHMM" pseudo
            // abbreviation (oracle-pinned in the prelude's format()).
            abbrev: format!(
                "GMT{}{:02}{:02}",
                if *off < 0 { '-' } else { '+' },
                off.abs() / 3600,
                off.abs() % 3600 / 60
            ),
            name: label.clone(),
            isdst: false,
        },
        ZoneRef::Named(n) => match php_types::tz::offset_at(n, epoch) {
            Some(i) => ZoneView { off: i.off, abbrev: i.abbrev, name: n.clone(), isdst: i.isdst },
            None => ZoneView { off: 0, abbrev: "UTC".to_string(), name: n.clone(), isdst: false },
        },
    }
}

/// Wall-clock time (civil fields packed as if UTC) → real epoch in `zr`,
/// with timelib's DST gap/fold resolution for named zones.
fn zone_wall_to_epoch(zr: &ZoneRef, wall: i64) -> i64 {
    match zr {
        ZoneRef::Utc => wall,
        ZoneRef::Fixed { off, .. } => wall - off,
        ZoneRef::Named(n) => php_types::tz::wall_to_epoch(n, wall).unwrap_or(wall),
    }
}

/// A timezone found inside a datetime STRING: its offset at that instant and
/// the display label `getTimezone()->getName()` keeps ("+05:00" normalized
/// from "+0500"; "UTC"/"GMT"/"Z" verbatim).
struct StrZone {
    off: i64,
    label: String,
}

fn str_zone_from_offset(sign: i64, h: i64, m: i64) -> StrZone {
    let off = sign * (h * 3600 + m * 60);
    StrZone { off, label: format!("{}{h:02}:{m:02}", if sign < 0 { '-' } else { '+' }) }
}

/// `__date_get_last_errors()`: the shared state above in PHP shape — `false`
/// when clean (PHP 8.2 changed the no-diagnostics return to `false`).
pub fn __date_get_last_errors(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let state = LAST_DATE_ERRORS.with(|s| s.borrow().clone());
    let Some(st) = state else { return Ok(Zval::Bool(false)) };
    if st.warnings.is_empty() && st.errors.is_empty() {
        return Ok(Zval::Bool(false));
    }
    let fill = |list: &[(i64, &'static str)]| {
        let mut m = PhpArray::new();
        for (pos, msg) in list {
            m.insert(Key::Int(*pos), Zval::Str(PhpStr::from_str(msg)));
        }
        Zval::Array(Rc::new(m))
    };
    let mut out = PhpArray::new();
    out.insert(Key::from_bytes(b"warning_count"), Zval::Long(st.warnings.len() as i64));
    out.insert(Key::from_bytes(b"warnings"), fill(&st.warnings));
    out.insert(Key::from_bytes(b"error_count"), Zval::Long(st.errors.len() as i64));
    out.insert(Key::from_bytes(b"errors"), fill(&st.errors));
    Ok(Zval::Array(Rc::new(out)))
}

const MONTHS_FULL: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
// PHP `l` (Sunday..Saturday).
const DAYS_FULL: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];
const DAYS_SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// English ordinal suffix for `S` (1st, 2nd, 3rd, 4th..). 11/12/13 are always `th`.
fn ordinal_suffix(day: u8) -> &'static str {
    if (11..=13).contains(&(day % 100)) {
        "th"
    } else {
        match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    }
}

/// Map a single PHP `date()` format char to its output, appending to `out`.
/// `dt` is the instant already shifted into the zone (`epoch + z.off`);
/// `epoch` stays the real instant (`U`, `B`); `z` supplies the zone fields.
fn append_char(out: &mut Vec<u8>, c: u8, dt: &OffsetDateTime, epoch: i64, z: &ZoneView) {
    let year = dt.year();
    let month = u8::from(dt.month());
    let day = dt.day();
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    // PHP w: 0=Sunday..6=Saturday. PHP N: 1=Monday..7=Sunday.
    let dow_sun0 = dt.weekday().number_days_from_sunday();
    let push = |out: &mut Vec<u8>, s: &str| out.extend_from_slice(s.as_bytes());
    let off_hhmm = |sep: &str| -> String {
        format!(
            "{}{:02}{sep}{:02}",
            if z.off < 0 { '-' } else { '+' },
            z.off.abs() / 3600,
            z.off.abs() % 3600 / 60
        )
    };
    match c {
        // --- Day ---
        b'd' => push(out, &format!("{day:02}")),
        b'j' => push(out, &format!("{day}")),
        b'D' => push(out, DAYS_SHORT[dow_sun0 as usize]),
        b'l' => push(out, DAYS_FULL[dow_sun0 as usize]),
        b'N' => push(out, &format!("{}", dt.weekday().number_from_monday())),
        b'w' => push(out, &format!("{dow_sun0}")),
        b'S' => push(out, ordinal_suffix(day)),
        b'z' => push(out, &format!("{}", dt.ordinal() - 1)),
        // --- Week ---
        b'W' => push(out, &format!("{:02}", dt.iso_week())),
        // --- Month ---
        b'F' => push(out, MONTHS_FULL[(month - 1) as usize]),
        b'M' => push(out, MONTHS_SHORT[(month - 1) as usize]),
        b'm' => push(out, &format!("{month:02}")),
        b'n' => push(out, &format!("{month}")),
        b't' => push(out, &format!("{}", days_in_month(year, month))),
        // --- Year ---
        b'L' => push(out, if is_leap_year(year) { "1" } else { "0" }),
        b'o' => push(out, &format!("{}", dt.to_iso_week_date().0)),
        b'Y' => push(out, &format!("{year:04}")),
        b'y' => push(out, &format!("{:02}", year.rem_euclid(100))),
        // --- Time ---
        b'a' => push(out, if hour < 12 { "am" } else { "pm" }),
        b'A' => push(out, if hour < 12 { "AM" } else { "PM" }),
        b'g' => push(out, &format!("{}", hour12(hour))),
        b'G' => push(out, &format!("{hour}")),
        b'h' => push(out, &format!("{:02}", hour12(hour))),
        b'H' => push(out, &format!("{hour:02}")),
        b'i' => push(out, &format!("{minute:02}")),
        b's' => push(out, &format!("{second:02}")),
        b'u' => push(out, "000000"),
        b'v' => push(out, "000"),
        b'B' => push(out, &format!("{:03}", swatch_beats(epoch))),
        // --- Timezone ---
        b'e' => push(out, &z.name),
        b'T' => push(out, &z.abbrev),
        b'I' => push(out, if z.isdst { "1" } else { "0" }),
        b'O' => push(out, &off_hhmm("")),
        b'P' => push(out, &off_hhmm(":")),
        b'Z' => push(out, &format!("{}", z.off)),
        // --- Full date/time ---
        b'c' => {
            // ISO 8601: Y-m-d\TH:i:sP
            push(
                out,
                &format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}{}",
                    off_hhmm(":")
                ),
            )
        }
        b'r' => {
            // RFC 2822: D, d M Y H:i:s O
            push(
                out,
                &format!(
                    "{}, {day:02} {} {year:04} {hour:02}:{minute:02}:{second:02} {}",
                    DAYS_SHORT[dow_sun0 as usize],
                    MONTHS_SHORT[(month - 1) as usize],
                    off_hhmm(""),
                ),
            )
        }
        b'U' => push(out, &format!("{epoch}")),
        // Any other byte is emitted literally.
        other => out.push(other),
    }
}

fn hour12(hour: u8) -> u8 {
    let h = hour % 12;
    if h == 0 {
        12
    } else {
        h
    }
}

/// Swatch Internet Time: thousandths of the day in Biel Mean Time (UTC+1),
/// zone-independent by definition.
fn swatch_beats(epoch: i64) -> i64 {
    (epoch + 3600).rem_euclid(86_400) * 1000 / 86_400
}

/// Format `epoch` (Unix seconds) per the PHP `fmt` string. Backslash escapes
/// the next byte (emitted literally). Unknown bytes pass through. `gmt`
/// callers (gmdate) render in UTC with the "GMT" abbreviation; everything
/// else renders in the process default timezone.
pub fn format_php(epoch: i64, fmt: &[u8], gmt: bool) -> Vec<u8> {
    let z = if gmt {
        ZoneView { off: 0, abbrev: "GMT".to_string(), name: "UTC".to_string(), isdst: false }
    } else {
        zone_view(&default_zone(), epoch)
    };
    let dt = match OffsetDateTime::from_unix_timestamp(epoch.saturating_add(z.off)) {
        Ok(dt) => dt,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::with_capacity(fmt.len() * 2);
    let mut i = 0;
    while i < fmt.len() {
        let c = fmt[i];
        if c == b'\\' {
            // Escaped: emit the next byte literally (or a trailing backslash).
            if i + 1 < fmt.len() {
                out.push(fmt[i + 1]);
                i += 2;
            } else {
                out.push(b'\\');
                i += 1;
            }
            continue;
        }
        append_char(&mut out, c, &dt, epoch, &z);
        i += 1;
    }
    out
}

/// Current Unix timestamp, or 0 if the clock is unavailable. Used only when the
/// `$timestamp` argument is omitted (non-deterministic; not differential-tested).
fn now_epoch() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `date(string $format, ?int $timestamp = null)`.
pub fn date(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("date() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let epoch = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    Ok(Zval::Str(PhpStr::new(format_php(epoch, fmt.as_bytes(), false))))
}

/// `gmdate(string $format, ?int $timestamp = null)`. With UTC scope this is
/// identical to `date()`.
pub fn gmdate(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("gmdate() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let epoch = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    Ok(Zval::Str(PhpStr::new(format_php(epoch, fmt.as_bytes(), true))))
}

/// Map one `strftime()` conversion char (the byte after `%`) to its output,
/// appending to `out`. C/POSIX locale, UTC scope (phpr is UTC-only, so the
/// timezone chars render the GMT values). Recursive specifiers (`%c %D %F %r %R
/// %T %x %X`) expand into their component chars. An unknown char is emitted
/// verbatim (without the `%`), matching glibc. Returns false only for `%%`-less
/// bookkeeping — always Some output here.
fn strftime_char(out: &mut Vec<u8>, c: u8, dt: &OffsetDateTime, epoch: i64) {
    let year = dt.year();
    let month = u8::from(dt.month());
    let day = dt.day();
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    let ordinal = dt.ordinal(); // 1..=366
    let dow_sun0 = dt.weekday().number_days_from_sunday(); // 0=Sun..6=Sat
    let push = |out: &mut Vec<u8>, s: &str| out.extend_from_slice(s.as_bytes());
    let expand = |out: &mut Vec<u8>, pat: &[u8]| {
        for &pc in pat {
            strftime_char(out, pc, dt, epoch);
        }
    };
    match c {
        b'a' => push(out, DAYS_SHORT[dow_sun0 as usize]),
        b'A' => push(out, DAYS_FULL[dow_sun0 as usize]),
        b'b' | b'h' => push(out, MONTHS_SHORT[(month - 1) as usize]),
        b'B' => push(out, MONTHS_FULL[(month - 1) as usize]),
        // %c: "%a %b %e %H:%M:%S %Y" — the C-locale preferred date+time.
        b'c' => expand(out, b"a b e H:M:S Y"),
        b'C' => push(out, &format!("{:02}", year.div_euclid(100))),
        b'd' => push(out, &format!("{day:02}")),
        b'D' => expand(out, b"m/d/y"),
        b'e' => push(out, &format!("{day:2}")), // space-padded day
        b'F' => expand(out, b"Y-m-d"),
        b'g' => push(out, &format!("{:02}", dt.to_iso_week_date().0.rem_euclid(100))),
        b'G' => push(out, &format!("{}", dt.to_iso_week_date().0)),
        b'H' => push(out, &format!("{hour:02}")),
        b'I' => push(out, &format!("{:02}", hour12(hour))),
        b'j' => push(out, &format!("{ordinal:03}")),
        b'm' => push(out, &format!("{month:02}")),
        b'M' => push(out, &format!("{minute:02}")),
        b'n' => out.push(b'\n'),
        b'p' => push(out, if hour < 12 { "AM" } else { "PM" }),
        b'P' => push(out, if hour < 12 { "am" } else { "pm" }),
        b'r' => expand(out, b"I:M:S p"),
        b'R' => expand(out, b"H:M"),
        b'S' => push(out, &format!("{second:02}")),
        b't' => out.push(b'\t'),
        b'T' => expand(out, b"H:M:S"),
        b'u' => push(out, &format!("{}", dt.weekday().number_from_monday())), // 1=Mon..7=Sun
        // %U: week of year, Sunday as the first day; days before the first Sunday
        // are week 00. %W is the same with Monday as the first day.
        b'U' => push(out, &format!("{:02}", (ordinal as i64 - 1 - dow_sun0 as i64 + 7) / 7)),
        b'V' => push(out, &format!("{:02}", dt.iso_week())),
        b'w' => push(out, &format!("{dow_sun0}")),
        b'W' => {
            let dow_mon0 = (dow_sun0 + 6) % 7; // 0=Mon..6=Sun
            push(out, &format!("{:02}", (ordinal as i64 - 1 - dow_mon0 as i64 + 7) / 7));
        }
        b'x' => expand(out, b"m/d/y"),
        b'X' => expand(out, b"H:M:S"),
        b'y' => push(out, &format!("{:02}", year.rem_euclid(100))),
        b'Y' => push(out, &format!("{year}")),
        b'z' => push(out, "+0000"), // UTC scope
        b'Z' => push(out, "GMT"),
        b'%' => out.push(b'%'),
        other => out.push(other),
    }
}

/// Format `epoch` (Unix seconds) per a `strftime()` format string, shifted
/// by `off` seconds into the rendering zone (`%s` stays the real epoch). A
/// `%` starts a conversion (the following byte selects it); a trailing `%`
/// is emitted literally. Non-`%` bytes pass through.
fn format_strftime(epoch: i64, fmt: &[u8], off: i64) -> Vec<u8> {
    let dt = match OffsetDateTime::from_unix_timestamp(epoch.saturating_add(off)) {
        Ok(dt) => dt,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::with_capacity(fmt.len() * 2);
    let mut i = 0;
    while i < fmt.len() {
        if fmt[i] == b'%' {
            if i + 1 < fmt.len() {
                strftime_char(&mut out, fmt[i + 1], &dt, epoch);
                i += 2;
            } else {
                out.push(b'%');
                i += 1;
            }
        } else {
            out.push(fmt[i]);
            i += 1;
        }
    }
    out
}

/// `strftime(string $format, ?int $timestamp = null): string|false` — legacy
/// C-library date formatting. Deprecated since 8.1. phpr is UTC-only, so this is
/// identical to [`gmstrftime`]. An empty result (PHP returns `false` when the C
/// library produces nothing) is reported as `false`.
pub fn strftime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strftime_impl(args, ctx, "strftime")
}

/// `gmstrftime(string $format, ?int $timestamp = null): string|false` — the UTC
/// variant of [`strftime`]; identical here (UTC scope). Deprecated since 8.1.
pub fn gmstrftime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    strftime_impl(args, ctx, "gmstrftime")
}

fn strftime_impl(args: &[Zval], ctx: &mut Ctx, fname: &str) -> Result<Zval, PhpError> {
    ctx.diags.push(php_types::Diag::Deprecated(format!(
        "Function {fname}() is deprecated since 8.1, use IntlDateFormatter::format() instead"
    )));
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error(format!("{fname}() expects at least 1 argument, 0 given"))
        })?,
        ctx.diags,
    );
    // PHP: an empty format string returns false.
    if fmt.as_bytes().is_empty() {
        return Ok(Zval::Bool(false));
    }
    let epoch = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    // gmstrftime renders UTC; strftime renders the default timezone.
    let off = if fname == "strftime" { zone_view(&default_zone(), epoch).off } else { 0 };
    let out = format_strftime(epoch, fmt.as_bytes(), off);
    // The C library yields nothing (e.g. the buffer stays empty) → PHP false.
    if out.is_empty() {
        return Ok(Zval::Bool(false));
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// `idate(string $format, ?int $timestamp = null): int|false` — a single numeric
/// date token as an integer (`php_idate`). A format that is not exactly one
/// character warns "idate format is one char" and returns false; a token outside
/// the numeric set warns "Unrecognized date format token" and returns false.
/// Every idate token renders as a numeric `date()` field, so this reuses
/// [`format_php`] and parses the digits.
pub fn idate(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("idate() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    if fmt.len() != 1 {
        ctx.diags
            .push(php_types::Diag::Warning("idate(): idate format is one char".to_string()));
        return Ok(Zval::Bool(false));
    }
    let c = fmt.as_bytes()[0];
    // The numeric tokens `php_idate` recognises; anything else is "unrecognized".
    const VALID: &[u8] = b"djNwzWmntLyYoBghHGisIZU";
    if !VALID.contains(&c) {
        ctx.diags.push(php_types::Diag::Warning(
            "idate(): Unrecognized date format token".to_string(),
        ));
        return Ok(Zval::Bool(false));
    }
    let epoch = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let s = format_php(epoch, &[c], false);
    let n = std::str::from_utf8(&s)
        .ok()
        .and_then(|t| t.trim().parse::<i64>().ok())
        .unwrap_or(0);
    Ok(Zval::Long(n))
}

/// PHP's legacy two-digit-year fixup for `mktime`: 0..69 → 2000..2069,
/// 70..100 → 1970..2000. Other values pass through unchanged.
fn fixup_two_digit_year(year: i64) -> i64 {
    if (0..=69).contains(&year) {
        year + 2000
    } else if (70..=100).contains(&year) {
        year + 1900
    } else {
        year
    }
}

/// Build a UTC Unix timestamp from civil components, normalizing every overflow
/// the PHP way: out-of-range months carry into the year, then any day/hour/
/// minute/second offset is added as a plain duration (so day 0 → previous
/// month's last day, hour 25 → next day +1h, etc.). `None` if the resulting
/// year is out of the representable range.
fn civil_to_epoch(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
) -> Option<i64> {
    // Normalize month into 1..=12 with a year carry.
    let total = year.checked_mul(12)?.checked_add(month - 1)?;
    let y = i32::try_from(total.div_euclid(12)).ok()?;
    let m = u8::try_from(total.rem_euclid(12) + 1).ok()?;
    let base = Date::from_calendar_date(y, Month::try_from(m).ok()?, 1)
        .ok()?
        .midnight()
        .assume_utc()
        .unix_timestamp();
    Some(base + (day - 1) * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// Nth int argument, defaulting to `default` when omitted or null.
fn int_arg_or(args: &[Zval], i: usize, default: i64, ctx: &mut Ctx) -> i64 {
    match args.get(i) {
        None | Some(Zval::Null) => default,
        Some(v) => convert::to_long_cast(v, ctx.diags),
    }
}

/// `mktime(?int $hour, ?int $minute, ?int $second, ?int $month, ?int $day,
/// ?int $year)`. Components are wall-clock values in the default timezone;
/// omitted trailing ones default to the current local time — those paths
/// read the real clock and are not differential-tested (D-DT5).
pub fn mktime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let zr = default_zone();
    let now_ts = now_epoch();
    let now = match OffsetDateTime::from_unix_timestamp(
        now_ts.saturating_add(zone_view(&zr, now_ts).off),
    ) {
        Ok(dt) => dt,
        Err(_) => return Ok(Zval::Bool(false)),
    };
    let hour = int_arg_or(args, 0, now.hour() as i64, ctx);
    let minute = int_arg_or(args, 1, now.minute() as i64, ctx);
    let second = int_arg_or(args, 2, now.second() as i64, ctx);
    let month = int_arg_or(args, 3, u8::from(now.month()) as i64, ctx);
    let day = int_arg_or(args, 4, now.day() as i64, ctx);
    let year = fixup_two_digit_year(int_arg_or(args, 5, now.year() as i64, ctx));
    match civil_to_epoch(year, month, day, hour, minute, second) {
        Some(wall) => Ok(Zval::Long(zone_wall_to_epoch(&zr, wall))),
        None => Ok(Zval::Bool(false)),
    }
}

/// `gmmktime(...)`: like [`mktime`] but the components are UTC, whatever the
/// default timezone.
pub fn gmmktime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let now = OffsetDateTime::now_utc();
    let hour = int_arg_or(args, 0, now.hour() as i64, ctx);
    let minute = int_arg_or(args, 1, now.minute() as i64, ctx);
    let second = int_arg_or(args, 2, now.second() as i64, ctx);
    let month = int_arg_or(args, 3, u8::from(now.month()) as i64, ctx);
    let day = int_arg_or(args, 4, now.day() as i64, ctx);
    let year = fixup_two_digit_year(int_arg_or(args, 5, now.year() as i64, ctx));
    match civil_to_epoch(year, month, day, hour, minute, second) {
        Some(ts) => Ok(Zval::Long(ts)),
        None => Ok(Zval::Bool(false)),
    }
}

/// `checkdate(int $month, int $day, int $year)`: true for a valid Gregorian
/// date with `1 <= month <= 12`, `1 <= year <= 32767`, and a day within the
/// month's length.
pub fn checkdate(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let month = convert::to_long_cast(
        args.first().ok_or_else(|| {
            PhpError::Error("checkdate() expects exactly 3 arguments, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let day = convert::to_long_cast(
        args.get(1).ok_or_else(|| {
            PhpError::Error("checkdate() expects exactly 3 arguments, 1 given".to_string())
        })?,
        ctx.diags,
    );
    let year = convert::to_long_cast(
        args.get(2).ok_or_else(|| {
            PhpError::Error("checkdate() expects exactly 3 arguments, 2 given".to_string())
        })?,
        ctx.diags,
    );
    let ok = (1..=12).contains(&month)
        && (1..=32767).contains(&year)
        && day >= 1
        && day <= days_in_month(year as i32, month as u8) as i64;
    Ok(Zval::Bool(ok))
}

/// Textual date formats (`strtotime`): the HTTP/cookie family —
/// `[DayName[,]] dd[- ]MonthName[- ](YYYY|YY) [HH:MM[:SS]] [GMT|UTC|Z|±HH[:]MM]`
/// (case-insensitive names; `Fri, 20-May-2011 15:25:52 GMT`,
/// `Tuesday, 08-Feb-94 14:15:29 GMT`, `20 May 2011`). Oracle-pinned rules: a
/// missing time is midnight; a two-digit year is 20YY below 70, 19YY from 70;
/// a day-name that disagrees with the date pushes it FORWARD to the next such
/// weekday. Named zones beyond the zero-offset ones (CEST, EST, …) would need
/// timelib's table and stay a parse failure. Returns the WALL time plus the
/// zone the string itself carried, if any (same contract as
/// [`parse_absolute`]).
fn parse_textual(s: &str) -> Option<(i64, Option<StrZone>)> {
    const DAYS: [&str; 7] =
        ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];
    const MONTHS: [&str; 12] = [
        "january", "february", "march", "april", "may", "june", "july", "august", "september",
        "october", "november", "december",
    ];
    // Normalise the date core's dashes to spaces (`20-May-2011` → `20 May
    // 2011`): only a dash TOUCHING a letter splits, so numeric offsets
    // (`-0500`) survive.
    let bytes = s.as_bytes();
    let mut norm = String::with_capacity(s.len());
    for (i, &c) in bytes.iter().enumerate() {
        let prev_alpha = i > 0 && bytes[i - 1].is_ascii_alphabetic();
        let next_alpha = bytes.get(i + 1).is_some_and(|n| n.is_ascii_alphabetic());
        if c == b'-' && (prev_alpha || next_alpha) {
            norm.push(' ');
        } else {
            norm.push(c as char);
        }
    }
    let mut toks: Vec<&str> = norm.split_whitespace().collect();
    if toks.is_empty() {
        return None;
    }
    // Optional leading day name (with or without a trailing comma).
    let mut want_dow: Option<i64> = None;
    {
        let head = toks[0].trim_end_matches(',').to_ascii_lowercase();
        if let Some(d) = DAYS.iter().position(|n| head == n[..3] || head == **n) {
            want_dow = Some(d as i64);
            toks.remove(0);
        }
    }
    // `dd MonthName YYYY` (the month name is what routes here at all).
    if toks.len() < 3 {
        return None;
    }
    if !toks[0].bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let day: i64 = toks[0].parse().ok()?;
    let mon_l = toks[1].to_ascii_lowercase();
    let month = 1 + MONTHS.iter().position(|m| mon_l == m[..3] || mon_l == **m)? as i64;
    let yraw = toks[2];
    if !yraw.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut year: i64 = yraw.parse().ok()?;
    if yraw.len() <= 2 {
        year = if year < 70 { 2000 + year } else { 1900 + year };
    }
    let mut i = 3;
    // Optional `HH:MM[:SS]` (absent → midnight, unlike date-from-format).
    let (mut h, mut mi, mut sec) = (0i64, 0i64, 0i64);
    if let Some(t) = toks.get(i) {
        if t.contains(':') {
            let mut p = t.split(':');
            h = p.next()?.parse().ok()?;
            mi = p.next()?.parse().ok()?;
            sec = match p.next() {
                Some(x) => x.parse().ok()?,
                None => 0,
            };
            if p.next().is_some() {
                return None;
            }
            i += 1;
        }
    }
    // Optional zone: zero-offset names or a numeric correction.
    let mut zone: Option<StrZone> = None;
    if let Some(t) = toks.get(i) {
        match t.to_ascii_uppercase().as_str() {
            up @ ("UTC" | "GMT" | "Z") => {
                zone = Some(StrZone { off: 0, label: up.to_string() })
            }
            z if z.starts_with('+') || z.starts_with('-') => {
                let sign = if z.starts_with('-') { -1 } else { 1 };
                let digits: String = z[1..].chars().filter(|c| *c != ':').collect();
                let (oh, om) = match digits.len() {
                    2 => (digits.parse::<i64>().ok()?, 0),
                    4 => (
                        digits[..2].parse::<i64>().ok()?,
                        digits[2..].parse::<i64>().ok()?,
                    ),
                    _ => return None,
                };
                zone = Some(str_zone_from_offset(sign, oh, om));
            }
            _ => return None,
        }
        i += 1;
    }
    if toks.get(i).is_some() {
        return None;
    }
    let mut base = civil_to_epoch(year, month, day, h, mi, sec)?;
    // A disagreeing day-name pushes the date forward (same rule as the
    // date-from-format `D`/`l` specifiers, oracle-pinned).
    if let Some(want) = want_dow {
        let days = base.div_euclid(86400);
        let dow = (days + 4).rem_euclid(7);
        base += (want - dow).rem_euclid(7) * 86400;
    }
    Some((base, zone))
}

/// Parse an absolute date string in the supported subset: `Y-m-d` or `Y/m/d`,
/// optionally followed by ` `/`T` and `H:i[:s]`. Returns the WALL-clock time
/// (civil fields packed as if UTC) plus the timezone carried by the string
/// itself, if any — the caller anchors wall times without one in the
/// prevailing zone.
fn parse_absolute(s: &str) -> Option<(i64, Option<StrZone>)> {
    // The ISO-8601 `T` separator only counts between digits — a blanket
    // replace would shred a trailing timezone NAME ("UTC" → "U C").
    let mut s = s.to_string();
    let b = s.clone().into_bytes();
    if let Some(p) = b
        .windows(3)
        .position(|w| w[0].is_ascii_digit() && w[1] == b'T' && w[2].is_ascii_digit())
    {
        s.replace_range(p + 1..p + 2, " ");
    }
    let mut parts = s.split_whitespace();
    let date = parts.next()?;
    let time = parts.next();
    // Optional third token: a timezone name (`… 15:58:59.123456 UTC`) or an
    // explicit offset. Only the zero-offset names are modelled; an unknown
    // name stays a parse failure.
    let mut name_zone: Option<StrZone> = None;
    if let Some(tzn) = parts.next() {
        match tzn.to_ascii_uppercase().as_str() {
            up @ ("UTC" | "GMT" | "Z") => {
                name_zone = Some(StrZone { off: 0, label: up.to_string() })
            }
            t if t.starts_with('+') || t.starts_with('-') => {
                let sign = if t.starts_with('-') { -1 } else { 1 };
                let digits: String = t[1..].chars().filter(|c| *c != ':').collect();
                let (h, m) = match digits.len() {
                    2 => (digits.parse::<i64>().ok()?, 0),
                    4 => (digits[..2].parse::<i64>().ok()?, digits[2..].parse::<i64>().ok()?),
                    _ => return None,
                };
                name_zone = Some(str_zone_from_offset(sign, h, m));
            }
            _ => return None,
        }
    }
    if parts.next().is_some() {
        return None;
    }
    // timelib "datenocolon": a bare 8-digit YYYYMMDD date token (what IPTC
    // 2#055 carries into WordPress' wp_read_image_metadata).
    let (year, month, day): (i64, i64, i64) =
        if date.len() == 8 && date.bytes().all(|b| b.is_ascii_digit()) {
            let y: i64 = date[..4].parse().ok()?;
            let m: i64 = date[4..6].parse().ok()?;
            let d: i64 = date[6..8].parse().ok()?;
            if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
                return None;
            }
            (y, m, d)
        } else {
            let sep = if date.contains('-') {
                '-'
            } else if date.contains('/') {
                '/'
            } else {
                return None;
            };
            let mut d = date.split(sep);
            let year: i64 = d.next()?.parse().ok()?;
            let month: i64 = d.next()?.parse().ok()?;
            let day: i64 = d.next()?.parse().ok()?;
            if d.next().is_some() {
                return None;
            }
            (year, month, day)
        };
    let (mut hour, mut min, mut sec) = (0i64, 0i64, 0i64);
    let mut time_zone: Option<StrZone> = None;
    if let Some(t) = time {
        // Split off a trailing ISO-8601 timezone (`Z`, `+HH:MM`, `-HHMM`,
        // `+HH`): the epoch is the civil time minus the offset.
        let mut t = t;
        if let Some(stripped) = t.strip_suffix('Z').or_else(|| t.strip_suffix('z')) {
            t = stripped;
            time_zone = Some(StrZone { off: 0, label: "Z".to_string() });
        } else if let Some(pos) = t.rfind(['+', '-']) {
            if pos > 0 {
                let (body, tzs) = t.split_at(pos);
                let sign = if tzs.starts_with('-') { -1 } else { 1 };
                let digits: String = tzs[1..].chars().filter(|c| *c != ':').collect();
                let (h, m) = match digits.len() {
                    2 => (digits.parse::<i64>().ok()?, 0),
                    4 => (digits[..2].parse::<i64>().ok()?, digits[2..].parse::<i64>().ok()?),
                    _ => return None,
                };
                time_zone = Some(str_zone_from_offset(sign, h, m));
                t = body;
            }
        }
        if t.len() == 6 && t.bytes().all(|b| b.is_ascii_digit()) {
            // timelib "timenocolon": HHMMSS (IPTC 2#060, e.g. "101112+0000" —
            // the offset suffix was already split off above).
            hour = t[..2].parse().ok()?;
            min = t[2..4].parse().ok()?;
            sec = t[4..6].parse().ok()?;
        } else {
            let mut tp = t.split(':');
            hour = tp.next()?.parse().ok()?;
            min = tp.next()?.parse().ok()?;
            sec = match tp.next() {
                // Fractional seconds are accepted and truncated (PHP returns an
                // integer epoch).
                Some(x) => x.split('.').next()?.parse().ok()?,
                None => 0,
            };
            if tp.next().is_some() {
                return None;
            }
        }
    }
    // A standalone zone token wins over a time-suffix one (parity with the
    // pre-refactor override order).
    let zone = name_zone.or(time_zone);
    Some((civil_to_epoch(year, month, day, hour, min, sec)?, zone))
}

/// Tokenize a relative expression (`[+-]N unit ...`, possibly repeated) into
/// accumulated component deltas `(years, months, days, hours, minutes,
/// seconds)`. Units: second(s)/sec, minute(s)/min, hour(s), day(s), week(s),
/// month(s), year(s); weeks fold into days, months/years stay separate (their
/// calendar-aware application happens in the caller). `None` if any token is
/// unrecognized or nothing applied.
fn accumulate_relative(s: &str) -> Option<(i64, i64, i64, i64, i64, i64)> {
    let (mut dy, mut dmo, mut dd, mut dh, mut dmi, mut ds) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
    let mut applied = false;
    let mut tokens = s.split_whitespace();
    while let Some(num_tok) = tokens.next() {
        let n: i64 = num_tok.parse().ok()?;
        let unit = tokens.next()?;
        match unit {
            "sec" | "secs" | "second" | "seconds" => ds += n,
            "min" | "mins" | "minute" | "minutes" => dmi += n,
            "hour" | "hours" => dh += n,
            "day" | "days" => dd += n,
            "week" | "weeks" => dd += n * 7,
            "month" | "months" => dmo += n,
            "year" | "years" => dy += n,
            _ => return None,
        }
        applied = true;
    }
    applied.then_some((dy, dmo, dd, dh, dmi, ds))
}

/// How a weekday name snaps the date: bare "monday" lands on the target
/// weekday going forward, today included; "next monday" strictly forward
/// (+7 when already there); "last friday" strictly backward (-7 when there).
#[derive(Clone, Copy)]
enum WeekdayMode {
    Bare,
    /// Bare name inverted by "ago": backward, today included.
    BareBack,
    Next,
    Last,
}

/// A parsed relative expression: the six component deltas plus timelib's
/// side effects — time-of-day reset (keywords/date words), month/year
/// overrides (month names), weekday snap, `first/last day of` modifiers.
#[derive(Default)]
struct RelExpr {
    dy: i64,
    dmo: i64,
    dd: i64,
    dh: i64,
    dmi: i64,
    ds: i64,
    set_year: Option<i64>,
    set_month: Option<i64>,
    set_time: Option<(i64, i64, i64)>,
    weekday: Option<(i64, WeekdayMode)>,
    first_day: bool,
    last_day: bool,
}

/// Fold `n × unit` into the deltas; `false` for an unknown unit.
fn apply_unit(r: &mut RelExpr, unit: &str, n: i64) -> bool {
    match unit {
        "sec" | "secs" | "second" | "seconds" => r.ds += n,
        "min" | "mins" | "minute" | "minutes" => r.dmi += n,
        "hour" | "hours" => r.dh += n,
        "day" | "days" => r.dd += n,
        "week" | "weeks" => r.dd += n * 7,
        "fortnight" | "fortnights" => r.dd += n * 14,
        "month" | "months" => r.dmo += n,
        "year" | "years" => r.dy += n,
        _ => return false,
    }
    true
}

/// Weekday name (full or 3-letter) → 0=Sunday..6=Saturday.
fn weekday_index(t: &str) -> Option<i64> {
    Some(match t {
        "sunday" | "sun" => 0,
        "monday" | "mon" => 1,
        "tuesday" | "tue" | "tues" => 2,
        "wednesday" | "wed" => 3,
        "thursday" | "thu" | "thur" | "thurs" => 4,
        "friday" | "fri" => 5,
        "saturday" | "sat" => 6,
        _ => return None,
    })
}

/// Month name (full or 3-letter) → 1..=12.
fn month_name_index(t: &str) -> Option<i64> {
    Some(match t {
        "january" | "jan" => 1,
        "february" | "feb" => 2,
        "march" | "mar" => 3,
        "april" | "apr" => 4,
        "may" => 5,
        "june" | "jun" => 6,
        "july" | "jul" => 7,
        "august" | "aug" => 8,
        "september" | "sep" | "sept" => 9,
        "october" | "oct" => 10,
        "november" | "nov" => 11,
        "december" | "dec" => 12,
        _ => return None,
    })
}

/// Split a token at digit/letter boundaries so timelib's fused forms
/// (`-1day`, `62seconds`) tokenize like their spaced spellings; a leading
/// sign stays glued to its number.
fn split_fused(tok: &str, out: &mut Vec<String>) {
    let mut cur = String::new();
    let mut cur_alpha = false;
    for c in tok.chars() {
        let alpha = c.is_ascii_alphabetic();
        if !cur.is_empty() && alpha != cur_alpha {
            out.push(std::mem::take(&mut cur));
        }
        cur_alpha = alpha;
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
}

/// Parse the relative-expression grammar subset (timelib): unit deltas
/// (spaced or fused), `ago`, day keywords (today/midnight/noon/tomorrow/
/// yesterday), `next|last|this|previous <unit|weekday>`, bare weekday and
/// month names (with optional year), `first|last day of`. `None` when any
/// token falls outside the modelled subset (→ strtotime false, like the
/// oracle on a typo).
fn parse_rel_expr(s: &str) -> Option<RelExpr> {
    let mut toks: Vec<String> = Vec::new();
    for t in s.split_whitespace() {
        split_fused(t.trim_matches(','), &mut toks);
    }
    let mut r = RelExpr::default();
    let mut applied = false;
    let mut i = 0;
    while i < toks.len() {
        let t = toks[i].as_str();
        let next_is = |off: usize, w: &str| toks.get(i + off).map(|x| x == w).unwrap_or(false);
        match t {
            "now" => {}
            "today" | "midnight" => r.set_time = Some((0, 0, 0)),
            "noon" => r.set_time = Some((12, 0, 0)),
            "tomorrow" => {
                r.dd += 1;
                r.set_time = Some((0, 0, 0));
            }
            "yesterday" => {
                r.dd -= 1;
                r.set_time = Some((0, 0, 0));
            }
            // "ago" negates everything accumulated so far ("2 days ago"),
            // weekday direction included ("saturday ago" = backward search).
            "ago" => {
                (r.dy, r.dmo, r.dd, r.dh, r.dmi, r.ds) =
                    (-r.dy, -r.dmo, -r.dd, -r.dh, -r.dmi, -r.ds);
                r.weekday = r.weekday.map(|(w, mode)| {
                    let flipped = match mode {
                        WeekdayMode::Bare => WeekdayMode::BareBack,
                        WeekdayMode::BareBack => WeekdayMode::Bare,
                        WeekdayMode::Next => WeekdayMode::Last,
                        WeekdayMode::Last => WeekdayMode::Next,
                    };
                    (w, flipped)
                });
            }
            "first" | "last" if next_is(1, "day") && next_is(2, "of") => {
                if t == "first" {
                    r.first_day = true;
                } else {
                    r.last_day = true;
                }
                i += 2;
            }
            "next" | "last" | "this" | "previous" => {
                let unit = toks.get(i + 1)?.as_str();
                if let Some(w) = weekday_index(unit) {
                    let mode = match t {
                        "next" => WeekdayMode::Next,
                        "this" => WeekdayMode::Bare,
                        _ => WeekdayMode::Last,
                    };
                    r.weekday = Some((w, mode));
                    r.set_time = Some((0, 0, 0));
                } else {
                    let n = match t {
                        "next" => 1,
                        "this" => 0,
                        _ => -1,
                    };
                    if !apply_unit(&mut r, unit, n) {
                        return None;
                    }
                }
                i += 1;
            }
            _ if weekday_index(t).is_some() => {
                r.weekday = Some((weekday_index(t)?, WeekdayMode::Bare));
                r.set_time = Some((0, 0, 0));
            }
            _ if month_name_index(t).is_some() => {
                r.set_month = month_name_index(t);
                // A month name is an absolute date element: time resets to
                // midnight (timelib) unless a later token sets it again.
                r.set_time.get_or_insert((0, 0, 0));
                // Optional plain year right after ("January 2027").
                if let Some(y) = toks
                    .get(i + 1)
                    .filter(|y| y.len() == 4 && y.bytes().all(|b| b.is_ascii_digit()))
                    .and_then(|y| y.parse::<i64>().ok())
                {
                    r.set_year = Some(y);
                    i += 1;
                }
            }
            _ => {
                // timelib relnumber is `[+-]* [ \t]* [0-9]+`: a run of signs
                // (each `-` flips) may stand apart from its digits — `+ 1
                // hour` and `--2 hours` both parse; the digits themselves
                // must then be unsigned (`+ -2 hours` is false).
                let signs = t.bytes().take_while(|b| matches!(b, b'+' | b'-')).count();
                let neg = t.bytes().take(signs).filter(|&b| b == b'-').count() % 2 == 1;
                let digits = &t[signs..];
                let (num_tok, extra) = if digits.is_empty() && signs > 0 {
                    (toks.get(i + 1)?.as_str(), 1)
                } else {
                    (digits, 0)
                };
                if !num_tok.bytes().all(|b| b.is_ascii_digit()) {
                    return None;
                }
                let n: i64 = num_tok.parse().ok()?;
                let n = if neg { -n } else { n };
                let unit = toks.get(i + 1 + extra)?.as_str();
                if !apply_unit(&mut r, unit, n) {
                    return None;
                }
                i += 1 + extra;
            }
        }
        applied = true;
        i += 1;
    }
    applied.then_some(r)
}

/// Days in the (possibly overflowed) civil month, after the same
/// month-into-year normalization `civil_to_epoch` does.
fn days_in_civil_month(year: i64, month: i64) -> Option<i64> {
    let total = year.checked_mul(12)?.checked_add(month - 1)?;
    let y = i32::try_from(total.div_euclid(12)).ok()?;
    let m = Month::try_from(u8::try_from(total.rem_euclid(12) + 1).ok()?).ok()?;
    Some(m.length(y) as i64)
}

/// Apply a relative expression to a base epoch, normalizing calendar overflow
/// the PHP way. Application order mirrors timelib: month/year overrides and
/// deltas → `first/last day of` → day delta → weekday snap → time-of-day
/// (reset or preserved) → hour/minute/second deltas.
fn parse_relative(s: &str, base: i64) -> Option<i64> {
    let r = parse_rel_expr(s)?;
    let dt = OffsetDateTime::from_unix_timestamp(base).ok()?;
    let year = r.set_year.unwrap_or(dt.year() as i64) + r.dy;
    let month = r.set_month.unwrap_or(u8::from(dt.month()) as i64) + r.dmo;
    let mut day = if r.first_day {
        1
    } else if r.last_day {
        days_in_civil_month(year, month)?
    } else {
        dt.day() as i64
    };
    day += r.dd;
    if let Some((target, mode)) = r.weekday {
        let midnight = civil_to_epoch(year, month, day, 0, 0, 0)?;
        // 1970-01-01 was a Thursday; 0=Sunday.
        let dow = (midnight.div_euclid(86_400) + 4).rem_euclid(7);
        day += match mode {
            WeekdayMode::Bare => (target - dow).rem_euclid(7),
            // timelib's negated-weekday branch (`d -= 7 - (|weekday| - dow)`,
            // do_adjust_for_weekday): "saturday ago" from Saturday is -7, not
            // 0. Sunday negates to 0 and empirically walks back inclusively.
            WeekdayMode::BareBack if target == 0 => -dow,
            WeekdayMode::BareBack => -(7 - (target - dow)),
            WeekdayMode::Next => (target - dow - 1).rem_euclid(7) + 1,
            WeekdayMode::Last => -((dow - target - 1).rem_euclid(7) + 1),
        };
    }
    let (hour, minute, second) = r.set_time.unwrap_or((
        dt.hour() as i64,
        dt.minute() as i64,
        dt.second() as i64,
    ));
    civil_to_epoch(
        year,
        month,
        day,
        hour + r.dh,
        minute + r.dmi,
        second + r.ds,
    )
}

// --- DateInterval / diff internals (step 34-6) --------------------------------
// These `__`-prefixed builtins back the DateInterval / DateTime::diff prelude
// classes; user code normally uses the OOP API.

/// Civil components (year, month, day, hour, minute, second) of a UTC epoch.
fn decompose(epoch: i64) -> Option<(i64, i64, i64, i64, i64, i64)> {
    let dt = OffsetDateTime::from_unix_timestamp(epoch).ok()?;
    Some((
        dt.year() as i64,
        u8::from(dt.month()) as i64,
        dt.day() as i64,
        dt.hour() as i64,
        dt.minute() as i64,
        dt.second() as i64,
    ))
}

/// Days in the month immediately preceding (`by`, `bm`), normalizing `bm`
/// outside 1..=12 into the adjacent year.
fn days_in_prev_month(mut by: i64, mut bm: i64) -> i64 {
    bm -= 1;
    while bm < 1 {
        bm += 12;
        by -= 1;
    }
    while bm > 12 {
        bm -= 12;
        by += 1;
    }
    days_in_month(by as i32, bm as u8) as i64
}

/// `__interval_parse(string $spec)`: parse an ISO 8601 duration
/// `P[nY][nM][nW][nD][T[nH][nM][nS]]` into an array of components (weeks fold
/// into days). Returns `false` on a malformed spec.
pub fn __interval_parse(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let spec = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("__interval_parse() expects 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let b = spec.as_bytes();
    let parsed = (|| {
        let rest = b.strip_prefix(b"P")?;
        let (mut y, mut mo, mut d, mut h, mut i, mut s) = (0i64, 0i64, 0i64, 0i64, 0i64, 0i64);
        let mut in_time = false;
        let mut num: Option<i64> = None;
        let mut saw_any = false;
        for &c in rest {
            if c == b'T' {
                if num.is_some() {
                    return None;
                }
                in_time = true;
                continue;
            }
            if c.is_ascii_digit() {
                num = Some(num.unwrap_or(0) * 10 + (c - b'0') as i64);
                continue;
            }
            let n = num.take()?;
            match (in_time, c) {
                (false, b'Y') => y = n,
                (false, b'M') => mo = n,
                (false, b'W') => d += n * 7,
                (false, b'D') => d += n,
                (true, b'H') => h = n,
                (true, b'M') => i = n,
                (true, b'S') => s = n,
                _ => return None,
            }
            saw_any = true;
        }
        if num.is_some() || !saw_any {
            return None;
        }
        Some((y, mo, d, h, i, s))
    })();
    match parsed {
        Some((y, mo, d, h, i, s)) => {
            let mut arr = PhpArray::new();
            arr.insert(Key::from_bytes(b"y"), Zval::Long(y));
            arr.insert(Key::from_bytes(b"m"), Zval::Long(mo));
            arr.insert(Key::from_bytes(b"d"), Zval::Long(d));
            arr.insert(Key::from_bytes(b"h"), Zval::Long(h));
            arr.insert(Key::from_bytes(b"i"), Zval::Long(i));
            arr.insert(Key::from_bytes(b"s"), Zval::Long(s));
            Ok(Zval::Array(Rc::new(arr)))
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// `__interval_from_date_string(string $rel)`: parse a relative expression
/// (`[+-]N unit ...`, the same subset `strtotime` accepts) into an
/// interval-component array `{y,m,d,h,i,s}` — weeks fold into days, months and
/// years stay separate. Backs `date_interval_create_from_date_string` (step 35,
/// D-PD3). Returns `false` when nothing parses.
pub fn __interval_from_date_string(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error(
                "__interval_from_date_string() expects 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    let lower = String::from_utf8_lossy(raw.as_bytes()).trim().to_ascii_lowercase();
    match accumulate_relative(&lower) {
        Some((y, mo, d, h, i, s)) => {
            let mut arr = PhpArray::new();
            arr.insert(Key::from_bytes(b"y"), Zval::Long(y));
            arr.insert(Key::from_bytes(b"m"), Zval::Long(mo));
            arr.insert(Key::from_bytes(b"d"), Zval::Long(d));
            arr.insert(Key::from_bytes(b"h"), Zval::Long(h));
            arr.insert(Key::from_bytes(b"i"), Zval::Long(i));
            arr.insert(Key::from_bytes(b"s"), Zval::Long(s));
            Ok(Zval::Array(Rc::new(arr)))
        }
        None => Ok(Zval::Bool(false)),
    }
}

/// `__date_diff(int $ts1, int $ts2)`: the calendar difference from `$ts1` to
/// `$ts2` as an array with `y/m/d/h/i/s/invert/days`. `invert` is 1 when
/// `$ts2 < $ts1`; `days` is the absolute total day count. The y/m/d breakdown
/// uses PHP's borrow algorithm (borrowing the preceding month's length, walking
/// the later date backward).
pub fn __date_diff(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let ts1 = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let ts2 = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let invert = if ts2 < ts1 { 1 } else { 0 };
    let (start, end) = if ts2 < ts1 { (ts2, ts1) } else { (ts1, ts2) };
    let days = (end - start) / 86_400;
    let (sy, smo, sd, sh, si, ss) = decompose(start).unwrap_or((0, 0, 0, 0, 0, 0));
    let (ey, emo, ed, eh, ei, es) = decompose(end).unwrap_or((0, 0, 0, 0, 0, 0));
    let (mut y, mut mo, mut d, mut h, mut i, mut s) =
        (ey - sy, emo - smo, ed - sd, eh - sh, ei - si, es - ss);
    if s < 0 {
        s += 60;
        i -= 1;
    }
    if i < 0 {
        i += 60;
        h -= 1;
    }
    if h < 0 {
        h += 24;
        d -= 1;
    }
    // Borrow whole months (the later date's preceding months) until days >= 0.
    let (mut base_y, mut base_m) = (ey, emo);
    while d < 0 {
        d += days_in_prev_month(base_y, base_m);
        mo -= 1;
        base_m -= 1;
        if base_m < 1 {
            base_m += 12;
            base_y -= 1;
        }
    }
    while mo < 0 {
        mo += 12;
        y -= 1;
    }
    let mut arr = PhpArray::new();
    arr.insert(Key::from_bytes(b"y"), Zval::Long(y));
    arr.insert(Key::from_bytes(b"m"), Zval::Long(mo));
    arr.insert(Key::from_bytes(b"d"), Zval::Long(d));
    arr.insert(Key::from_bytes(b"h"), Zval::Long(h));
    arr.insert(Key::from_bytes(b"i"), Zval::Long(i));
    arr.insert(Key::from_bytes(b"s"), Zval::Long(s));
    arr.insert(Key::from_bytes(b"invert"), Zval::Long(invert));
    arr.insert(Key::from_bytes(b"days"), Zval::Long(days));
    Ok(Zval::Array(Rc::new(arr)))
}

/// `__interval_format(DateInterval $iv, string $format)`: render a DateInterval
/// per its `%`-specifier mini-language, reading the object's public properties.
pub fn __interval_format(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let obj = match args.first() {
        Some(Zval::Object(o)) => o,
        _ => {
            return Err(PhpError::Error(
                "__interval_format(): argument 1 must be a DateInterval".to_string(),
            ))
        }
    };
    let fmt = convert::to_zstr(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let b = obj.borrow();
    let geti = |name: &[u8]| -> i64 {
        match b.props.get(name) {
            Some(Zval::Long(n)) => *n,
            _ => 0,
        }
    };
    let (y, m, d, h, i, s, invert) = (
        geti(b"y"),
        geti(b"m"),
        geti(b"d"),
        geti(b"h"),
        geti(b"i"),
        geti(b"s"),
        geti(b"invert"),
    );
    // `days` is either an int total or `false` (built from a spec).
    let days = match b.props.get(b"days") {
        Some(Zval::Long(n)) => Some(*n),
        _ => None,
    };
    let bytes = fmt.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut idx = 0;
    while idx < bytes.len() {
        if bytes[idx] != b'%' || idx + 1 >= bytes.len() {
            out.push(bytes[idx]);
            idx += 1;
            continue;
        }
        let spec = bytes[idx + 1];
        idx += 2;
        let push = |out: &mut Vec<u8>, s: String| out.extend_from_slice(s.as_bytes());
        match spec {
            b'Y' => push(&mut out, format!("{y:02}")),
            b'y' => push(&mut out, format!("{y}")),
            b'M' => push(&mut out, format!("{m:02}")),
            b'm' => push(&mut out, format!("{m}")),
            b'D' => push(&mut out, format!("{d:02}")),
            b'd' => push(&mut out, format!("{d}")),
            b'H' => push(&mut out, format!("{h:02}")),
            b'h' => push(&mut out, format!("{h}")),
            b'I' => push(&mut out, format!("{i:02}")),
            b'i' => push(&mut out, format!("{i}")),
            b'S' => push(&mut out, format!("{s:02}")),
            b's' => push(&mut out, format!("{s}")),
            b'a' => match days {
                Some(n) => push(&mut out, format!("{n}")),
                None => out.extend_from_slice(b"(unknown)"),
            },
            b'R' => out.push(if invert != 0 { b'-' } else { b'+' }),
            b'r' => {
                if invert != 0 {
                    out.push(b'-');
                }
            }
            b'%' => out.push(b'%'),
            other => {
                out.push(b'%');
                out.push(other);
            }
        }
    }
    Ok(Zval::Str(PhpStr::new(out)))
}

/// Read 1..=`max` ASCII digits at `*vi`, advancing it. `None` if no digit.
fn read_digits(val: &[u8], vi: &mut usize, max: usize) -> Option<i64> {
    let start = *vi;
    while *vi < val.len() && *vi - start < max && val[*vi].is_ascii_digit() {
        *vi += 1;
    }
    if *vi == start {
        return None;
    }
    std::str::from_utf8(&val[start..*vi]).ok()?.parse().ok()
}

/// `__date_from_format(string $format, string $value)`: parse `$value` per the
/// `date()`-style `$format` and return the UTC epoch, or `false` on mismatch.
/// `!` (leading) resets all fields to the Unix epoch; `|` resets the fields not
/// yet parsed. Unparsed fields without a reset default to the current date/time
/// (non-deterministic; D-DT5). Supported chars: `Y y m n d j H G h g i s` plus
/// literals (with `\` escape). This is the explicit-format subset (D-DT4).
pub fn __date_from_format(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let fmt = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let val = convert::to_zstr(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let f = fmt.as_bytes();
    let v = val.as_bytes();

    let now = decompose(now_epoch()).unwrap_or((1970, 1, 1, 0, 0, 0));
    let mut fi = 0;
    // A leading `!` starts from the Unix epoch instead of "now".
    let (mut yr, mut mo, mut d, mut h, mut mi, mut s) = if f.first() == Some(&b'!') {
        fi = 1;
        (1970i64, 1i64, 1i64, 0i64, 0i64, 0i64)
    } else {
        now
    };
    // Track which fields were explicitly parsed (for `|`).
    let mut seen = [false; 6]; // y, mo, d, h, i, s
    let mut vi = 0;
    // `O`/`P` timezone offset (seconds east) + its `+HH:MM` display form, and
    // `u`/`v` fractional seconds (as microseconds).
    let mut tz_off: Option<(i64, Vec<u8>)> = None;
    let mut micros: i64 = 0;
    // `D`/`l` textual weekday (0 = Sunday): oracle-pinned, a name that does
    // not match the parsed date jumps it FORWARD to the next such weekday
    // (`Sat, 12 Jul 2026` → the 18th), applied after all fields parse.
    let mut want_dow: Option<i64> = None;
    // getLastErrors bookkeeping: an out-of-range date/time that the epoch
    // conversion then normalizes is timelib's "parsed date/time was invalid"
    // *warning*; `fmt_consumed` distinguishes a leftover-input failure from a
    // mid-format mismatch for the failure message.
    let mut warn_invalid: Option<&'static str> = None;
    let mut fmt_consumed = false;

    let result = (|| {
        while fi < f.len() {
            let fc = f[fi];
            fi += 1;
            match fc {
                b'\\' => {
                    let lit = *f.get(fi)?;
                    fi += 1;
                    if v.get(vi) == Some(&lit) {
                        vi += 1;
                    } else {
                        return None;
                    }
                }
                b'!' => {
                    yr = 1970;
                    mo = 1;
                    d = 1;
                    h = 0;
                    mi = 0;
                    s = 0;
                    seen = [true; 6];
                }
                b'|' => {
                    let epoch = [1970, 1, 1, 0, 0, 0];
                    for (k, slot) in [&mut yr, &mut mo, &mut d, &mut h, &mut mi, &mut s]
                        .into_iter()
                        .enumerate()
                    {
                        if !seen[k] {
                            *slot = epoch[k];
                        }
                    }
                }
                b'Y' => {
                    yr = read_digits(v, &mut vi, 4)?;
                    seen[0] = true;
                }
                b'y' => {
                    let n = read_digits(v, &mut vi, 2)?;
                    yr = if n < 70 { 2000 + n } else { 1900 + n };
                    seen[0] = true;
                }
                b'm' | b'n' => {
                    mo = read_digits(v, &mut vi, 2)?;
                    seen[1] = true;
                }
                b'M' | b'F' => {
                    // Textual month, case-insensitive; PHP's parser accepts the
                    // abbreviated and full names for either specifier.
                    let start = vi;
                    while vi < v.len() && v[vi].is_ascii_alphabetic() {
                        vi += 1;
                    }
                    let w = v[start..vi].to_ascii_lowercase();
                    const MONTHS: [&[u8]; 12] = [
                        b"january", b"february", b"march", b"april", b"may", b"june", b"july",
                        b"august", b"september", b"october", b"november", b"december",
                    ];
                    mo = 1 + MONTHS
                        .iter()
                        .position(|m| w.as_slice() == &m[..3.min(m.len())] || w.as_slice() == *m)?
                        as i64;
                    seen[1] = true;
                }
                b'D' | b'l' => {
                    // Textual weekday, case-insensitive (see `want_dow`).
                    let start = vi;
                    while vi < v.len() && v[vi].is_ascii_alphabetic() {
                        vi += 1;
                    }
                    let w = v[start..vi].to_ascii_lowercase();
                    const DAYS: [&[u8]; 7] = [
                        b"sunday", b"monday", b"tuesday", b"wednesday", b"thursday", b"friday",
                        b"saturday",
                    ];
                    want_dow = Some(
                        DAYS.iter()
                            .position(|n| w.as_slice() == &n[..3] || w.as_slice() == *n)?
                            as i64,
                    );
                }
                b'd' | b'j' => {
                    d = read_digits(v, &mut vi, 2)?;
                    seen[2] = true;
                }
                b'H' | b'G' | b'h' | b'g' => {
                    h = read_digits(v, &mut vi, 2)?;
                    seen[3] = true;
                }
                b'i' => {
                    mi = read_digits(v, &mut vi, 2)?;
                    seen[4] = true;
                }
                b's' => {
                    s = read_digits(v, &mut vi, 2)?;
                    seen[5] = true;
                }
                b'O' | b'P' | b'p' => {
                    // `+0300` / `+03:00`, a literal `Z`, or — oracle-pinned —
                    // a zero-offset NAME (`GMT`/`UTC`, optionally `GMT+0200`):
                    // DATE_RFC2822 parses real HTTP dates ending in ` GMT`.
                    // Other abbreviations (EST, CET, …) need timelib's table
                    // and stay unsupported (parse fails, as before).
                    if v.get(vi).is_some_and(|c| c.is_ascii_alphabetic()) && v.get(vi) != Some(&b'Z')
                    {
                        let start = vi;
                        while vi < v.len() && v[vi].is_ascii_alphabetic() {
                            vi += 1;
                        }
                        match v[start..vi].to_ascii_uppercase().as_slice() {
                            b"GMT" | b"UTC" => {
                                let name = v[start..vi].to_ascii_uppercase();
                                if v.get(vi) == Some(&b'+') || v.get(vi) == Some(&b'-') {
                                    // `GMT+0200`: fall through to the numeric
                                    // correction below (name display dropped).
                                    let sign = if v[vi] == b'-' { -1i64 } else { 1 };
                                    vi += 1;
                                    let hh = read_digits(v, &mut vi, 2)?;
                                    if v.get(vi) == Some(&b':') {
                                        vi += 1;
                                    }
                                    let mm = read_digits(v, &mut vi, 2)?;
                                    let disp = format!(
                                        "{}{:02}:{:02}",
                                        if sign < 0 { '-' } else { '+' },
                                        hh,
                                        mm
                                    );
                                    tz_off = Some((sign * (hh * 3600 + mm * 60), disp.into_bytes()));
                                } else {
                                    tz_off = Some((0, name));
                                }
                            }
                            _ => return None,
                        }
                    } else if v.get(vi) == Some(&b'Z') {
                        vi += 1;
                        tz_off = Some((0, b"+00:00".to_vec()));
                    } else {
                        let sign = match v.get(vi)? {
                            b'+' => 1i64,
                            b'-' => -1i64,
                            _ => return None,
                        };
                        vi += 1;
                        let hh = read_digits(v, &mut vi, 2)?;
                        if v.get(vi) == Some(&b':') {
                            vi += 1;
                        }
                        let mm = read_digits(v, &mut vi, 2)?;
                        let disp = format!(
                            "{}{:02}:{:02}",
                            if sign < 0 { '-' } else { '+' },
                            hh,
                            mm
                        );
                        tz_off = Some((sign * (hh * 3600 + mm * 60), disp.into_bytes()));
                    }
                }
                b'T' | b'e' => {
                    // Timezone name/abbreviation. Only the zero-offset names
                    // are modelled (phpr keeps wall time in UTC); the display
                    // form feeds the object's tz so `format('T')` round-trips.
                    let start = vi;
                    while vi < v.len()
                        && (v[vi].is_ascii_alphabetic() || v[vi] == b'/' || v[vi] == b'_')
                    {
                        vi += 1;
                    }
                    if vi == start {
                        return None;
                    }
                    match v[start..vi].to_ascii_uppercase().as_slice() {
                        // Zero-offset names; the display keeps the name as
                        // written uppercased (`format('T')` on a GMT-parsed
                        // date shows GMT, oracle-pinned).
                        up @ (b"UTC" | b"GMT" | b"Z") => {
                            tz_off = Some((0, if up == b"Z" { b"UTC".to_vec() } else { up.to_vec() }))
                        }
                        _ => return None,
                    }
                }
                b'u' | b'v' => {
                    // Up to 6 (u) / 3 (v) fraction digits, scaled to micros.
                    let max = if fc == b'u' { 6 } else { 3 };
                    let start = vi;
                    let mut n: i64 = 0;
                    while vi < v.len() && vi - start < max && v[vi].is_ascii_digit() {
                        n = n * 10 + i64::from(v[vi] - b'0');
                        vi += 1;
                    }
                    if vi == start {
                        return None;
                    }
                    let mut digits = vi - start;
                    while digits < 6 {
                        n *= 10;
                        digits += 1;
                    }
                    micros = if fc == b'v' { n } else { n };
                }
                b'U' => {
                    // Epoch seconds, straight through.
                    let start = vi;
                    let mut n: i64 = 0;
                    let neg = if v.get(vi) == Some(&b'-') {
                        vi += 1;
                        true
                    } else {
                        false
                    };
                    while vi < v.len() && v[vi].is_ascii_digit() {
                        n = n * 10 + i64::from(v[vi] - b'0');
                        vi += 1;
                    }
                    if vi == start {
                        return None;
                    }
                    let ts = if neg { -n } else { n };
                    let (y2, mo2, d2, h2, mi2, s2) = decompose(ts)?;
                    (yr, mo, d, h, mi, s) = (y2, mo2, d2, h2, mi2, s2);
                    seen = [true; 6];
                    // An epoch is an absolute instant: PHP gives the object
                    // the zero-offset zone, so the caller must NOT re-anchor
                    // the result in a local zone (bug66836).
                    tz_off = Some((0, b"+00:00".to_vec()));
                }
                other => {
                    if v.get(vi) == Some(&other) {
                        vi += 1;
                    } else {
                        return None;
                    }
                }
            }
        }
        // The whole value must be consumed.
        fmt_consumed = true;
        if vi != v.len() {
            return None;
        }
        // timelib: once ANY time component is parsed, the unparsed time
        // components are 0, not "now" (`H:i` gives :00 seconds; the date
        // components still default to today).
        if seen[3] || seen[4] || seen[5] {
            if !seen[3] {
                h = 0;
            }
            if !seen[4] {
                mi = 0;
            }
            if !seen[5] {
                s = 0;
            }
        }
        // Out-of-range fields normalize below but leave a getLastErrors
        // warning, exactly what rejects `2012-21-07` under `Y-m-d`.
        if !(1..=12).contains(&mo) || d < 1 || d > days_in_civil_month(yr, mo).unwrap_or(31) {
            warn_invalid = Some("The parsed date was invalid");
        } else if !(0..=23).contains(&h) || !(0..=59).contains(&mi) || !(0..=59).contains(&s) {
            warn_invalid = Some("The parsed time was invalid");
        }
        let mut base = civil_to_epoch(yr, mo, d, h, mi, s)?;
        // A textual weekday that disagrees with the parsed date pushes the
        // wall date forward to the next such weekday (oracle-pinned).
        if let Some(want) = want_dow {
            let days = base.div_euclid(86400);
            let dow = (days + 4).rem_euclid(7); // 1970-01-01 was a Thursday
            base += (want - dow).rem_euclid(7) * 86400;
        }
        // A parsed offset means the wall time above was *in* that offset.
        Some(base - tz_off.as_ref().map(|(o, _)| *o).unwrap_or(0))
    })();

    // Refresh the getLastErrors container for this parse. Messages are the
    // honest generic subset: exhausted input keeps timelib's wording; other
    // failures use its catch-all (per-specifier wording is a scope-out).
    LAST_DATE_ERRORS.with(|st| {
        let mut diag = LastDateErrors::default();
        match &result {
            Some(_) => {
                if let Some(w) = warn_invalid {
                    diag.warnings.push((v.len() as i64, w));
                }
            }
            None => {
                let (pos, msg) = if fmt_consumed {
                    (vi as i64, "Trailing data")
                } else if vi >= v.len() {
                    (v.len() as i64, "Not enough data available to satisfy format")
                } else {
                    (vi as i64, "Unexpected data found.")
                };
                diag.errors.push((pos, msg));
            }
        }
        *st.borrow_mut() = Some(diag);
    });

    Ok(match result {
        Some(ts) => {
            // `[ts, tz-display|null, microseconds]` for the prelude wrapper.
            let mut out = php_types::PhpArray::new();
            let _ = out.append(Zval::Long(ts));
            let _ = out.append(match tz_off {
                Some((_, disp)) => Zval::Str(php_types::PhpStr::new(disp)),
                None => Zval::Null,
            });
            let _ = out.append(Zval::Long(micros));
            Zval::Array(std::rc::Rc::new(out))
        }
        None => Zval::Bool(false),
    })
}

/// `time()`: the current Unix timestamp. Non-deterministic (reads the real
/// clock); not differential-tested (D-DT5).
pub fn time(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(now_epoch()))
}

/// `microtime($as_float = false)`: the current epoch with microseconds — as the
/// PHP-classic `"0.NNNNNNNN SSSSSSSSSS"` string by default, as a float with
/// `true`. Non-deterministic (reads the real clock); not differential-tested.
pub fn microtime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let as_float = args.first().map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    if as_float {
        Ok(Zval::Double(d.as_secs() as f64 + f64::from(d.subsec_micros()) / 1e6))
    } else {
        Ok(Zval::Str(PhpStr::new(
            format!("0.{:08} {}", d.subsec_micros() * 100, d.as_secs()).into_bytes(),
        )))
    }
}

/// `getrusage($mode = 0)`: process resource usage. phpr reads no OS rusage
/// (no libc); consumers (PHPUnit's telemetry) only *delta* the CPU-time fields,
/// so `ru_utime` advances with the process's monotonic elapsed time and every
/// other field is 0, under the full standard key set.
pub fn getrusage(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::sync::OnceLock;
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    let start = *START.get_or_init(std::time::Instant::now);
    let d = start.elapsed();
    let mut out = PhpArray::new();
    let mut put = |k: &[u8], v: i64| {
        out.insert(php_types::Key::Str(PhpStr::new(k.to_vec())), Zval::Long(v));
    };
    for k in [
        &b"ru_oublock"[..], b"ru_inblock", b"ru_msgsnd", b"ru_msgrcv", b"ru_maxrss",
        b"ru_ixrss", b"ru_idrss", b"ru_isrss", b"ru_minflt", b"ru_majflt", b"ru_nsignals",
        b"ru_nvcsw", b"ru_nivcsw", b"ru_nswap",
    ] {
        put(k, 0);
    }
    put(b"ru_utime.tv_usec", i64::from(d.subsec_micros()));
    put(b"ru_utime.tv_sec", d.as_secs() as i64);
    put(b"ru_stime.tv_usec", 0);
    put(b"ru_stime.tv_sec", 0);
    Ok(Zval::Array(Rc::new(out)))
}

/// `hrtime($as_number = false)`: a monotonic high-resolution timestamp —
/// `[seconds, nanoseconds]` by default, total nanoseconds as an int with
/// `true`. Anchored to an arbitrary epoch (process start), like PHP's.
pub fn hrtime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    use std::sync::OnceLock;
    static START: OnceLock<std::time::Instant> = OnceLock::new();
    let start = *START.get_or_init(std::time::Instant::now);
    let d = start.elapsed();
    let as_number = args.first().map(|v| convert::to_bool(v, ctx.diags)).unwrap_or(false);
    if as_number {
        Ok(Zval::Long(d.as_nanos() as i64))
    } else {
        let mut out = PhpArray::new();
        let _ = out.append(Zval::Long(d.as_secs() as i64));
        let _ = out.append(Zval::Long(i64::from(d.subsec_nanos())));
        Ok(Zval::Array(Rc::new(out)))
    }
}

/// `date_default_timezone_set(string $timezoneId)`: install the request-wide
/// default zone. An unknown ID leaves the state untouched and notices
/// "Timezone ID '%s' is invalid" (returning `false`), like timelib.
pub fn date_default_timezone_set(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error(
                "date_default_timezone_set() expects at least 1 argument, 0 given".to_string(),
            )
        })?,
        ctx.diags,
    );
    let id = String::from_utf8_lossy(raw.as_bytes()).into_owned();
    if php_types::tz::set_default_timezone(&id) {
        Ok(Zval::Bool(true))
    } else {
        ctx.diags.push(php_types::Diag::Notice(format!(
            "date_default_timezone_set(): Timezone ID '{id}' is invalid"
        )));
        Ok(Zval::Bool(false))
    }
}

/// `date_default_timezone_get()`: the request-wide default zone ("UTC" until
/// something sets it).
pub fn date_default_timezone_get(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(php_types::tz::default_timezone().into_bytes())))
}

/// `strtotime(string $datetime, ?int $baseTimestamp = now)`. Supported subset
/// (D-DT4): `@N` epoch, `now`, ISO/`Y/m/d` absolute dates with optional time,
/// and `[+-]N unit` relative expressions. Everything else → `false`.
pub fn strtotime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(
        args.first().ok_or_else(|| {
            PhpError::Error("strtotime() expects at least 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let base = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let s = String::from_utf8_lossy(raw.as_bytes());
    Ok(match strtotime_in(s.trim(), base, &default_zone()) {
        Some((ts, _)) => Zval::Long(ts),
        None => Zval::Bool(false),
    })
}

/// The shared strtotime engine: parse `trimmed` against `base` with naked
/// wall-clock times anchored in `zr`. Returns the epoch plus the display
/// label of a zone the STRING itself carried (`None` when the prevailing
/// zone applied) — the DateTime constructor keeps that label.
fn strtotime_in(trimmed: &str, base: i64, zr: &ZoneRef) -> Option<(i64, Option<String>)> {
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = trimmed.strip_prefix('@') {
        // `@N` is an absolute instant; PHP labels it with the zero offset.
        return rest.parse::<i64>().ok().map(|ts| (ts, Some("+00:00".to_string())));
    }
    if lower == "now" {
        return Some((base, None));
    }
    if let Some((wall, zone)) = parse_absolute(trimmed).or_else(|| parse_textual(trimmed)) {
        return Some(match zone {
            Some(z) => (wall - z.off, Some(z.label)),
            None => (zone_wall_to_epoch(zr, wall), None),
        });
    }
    // Relative expressions do their calendar math on the WALL clock of the
    // zone (a "+1 day" across a DST jump keeps the wall time, spending 23 or
    // 25 real hours — oracle-pinned).
    let wall_base = base.saturating_add(zone_view(zr, base).off);
    parse_relative(&lower, wall_base).map(|w| (zone_wall_to_epoch(zr, w), None))
}

/// `__tz_offset(string $zone, int $ts)` (prelude-internal): `[offset_secs,
/// abbrev, isdst]` of the instant in the zone — labels are IANA names,
/// "±HH:MM"/"±HHMM" offsets, "UTC", "GMT" or "Z". `false` when the label
/// does not resolve.
pub fn __tz_offset(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let ts = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let label = String::from_utf8_lossy(raw.as_bytes());
    let Some(zr) = resolve_zone(&label) else {
        return Ok(Zval::Bool(false));
    };
    let v = zone_view(&zr, ts);
    let mut out = PhpArray::new();
    let _ = out.append(Zval::Long(v.off));
    let _ = out.append(Zval::Str(PhpStr::new(v.abbrev.into_bytes())));
    let _ = out.append(Zval::Bool(v.isdst));
    Ok(Zval::Array(Rc::new(out)))
}

/// `__tz_wall_ts(string $zone, int $wall)` (prelude-internal): re-anchor a
/// wall-clock time (civil fields packed as if UTC) in the zone, resolving
/// DST gaps/folds the timelib way. An unresolvable label is the identity.
pub fn __tz_wall_ts(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let wall = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let label = String::from_utf8_lossy(raw.as_bytes());
    let ts = match resolve_zone(&label) {
        Some(zr) => zone_wall_to_epoch(&zr, wall),
        None => wall,
    };
    Ok(Zval::Long(ts))
}

/// `__tz_transition(string $zone, int $ts)` (prelude-internal):
/// `[trans_offset, trans_time]` of the zone interval containing `$ts` —
/// timelib's `timelib_get_time_zone_offset_info` pair, feeding the DST
/// corrections of `timelib_diff_with_tzid`. Fixed-offset labels have no
/// transitions: their interval starts at PHP_INT_MIN.
pub fn __tz_transition(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let ts = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let label = String::from_utf8_lossy(raw.as_bytes());
    let (off, start) = match resolve_zone(&label) {
        Some(ZoneRef::Named(n)) => match php_types::tz::offset_at_ex(&n, ts) {
            Some((i, start)) => (i.off, start),
            None => (0, i64::MIN),
        },
        Some(ZoneRef::Fixed { off, .. }) => (off, i64::MIN),
        Some(ZoneRef::Utc) | None => (0, i64::MIN),
    };
    let mut out = PhpArray::new();
    let _ = out.append(Zval::Long(off));
    let _ = out.append(Zval::Long(start));
    Ok(Zval::Array(Rc::new(out)))
}

/// `__tz_transitions(string $zone, int $begin, int $end)` (prelude-internal,
/// feeds DateTimeZone::getTransitions): list of `[ts, offset, isdst, abbr]`
/// rows — the state at `$begin` first, then every transition in range. The
/// prelude rejects non-identifier zones before calling (PHP returns false for
/// offset/abbreviation zones); UTC yields its single steady row.
pub fn __tz_transitions(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let begin = convert::to_long_cast(args.get(1).unwrap_or(&Zval::Null), ctx.diags);
    let end = convert::to_long_cast(args.get(2).unwrap_or(&Zval::Null), ctx.diags);
    let label = String::from_utf8_lossy(raw.as_bytes());
    let rows: Vec<(i64, php_types::tz::TzInfo)> = match resolve_zone(&label) {
        Some(ZoneRef::Named(n)) => match php_types::tz::transitions_between(&n, begin, end) {
            Some(r) => r,
            None => return Ok(Zval::Bool(false)),
        },
        Some(ZoneRef::Utc) => vec![(
            begin,
            php_types::tz::TzInfo { off: 0, abbrev: "UTC".to_string(), isdst: false },
        )],
        _ => return Ok(Zval::Bool(false)),
    };
    let mut out = PhpArray::new();
    for (ts, info) in rows {
        let mut row = PhpArray::new();
        let _ = row.append(Zval::Long(ts));
        let _ = row.append(Zval::Long(info.off));
        let _ = row.append(Zval::Bool(info.isdst));
        let _ = row.append(Zval::Str(PhpStr::new(info.abbrev.into_bytes())));
        let _ = out.append(Zval::Array(Rc::new(row)));
    }
    Ok(Zval::Array(Rc::new(out)))
}

/// `__strtotime_tz(string $datetime, ?int $base, string $zone)`
/// (prelude-internal): [`strtotime_in`] against an explicit zone — the
/// DateTime constructor's parse. Returns `[epoch, zone-label-in-string|null]`
/// or `false` on parse failure.
pub fn __strtotime_tz(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let raw = convert::to_zstr(args.first().unwrap_or(&Zval::Null), ctx.diags);
    let base = match args.get(1) {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let zraw = convert::to_zstr(args.get(2).unwrap_or(&Zval::Null), ctx.diags);
    let zlabel = String::from_utf8_lossy(zraw.as_bytes());
    let zr = resolve_zone(&zlabel).unwrap_or(ZoneRef::Utc);
    let s = String::from_utf8_lossy(raw.as_bytes());
    Ok(match strtotime_in(s.trim(), base, &zr) {
        Some((ts, label)) => {
            let mut out = PhpArray::new();
            let _ = out.append(Zval::Long(ts));
            let _ = out.append(match label {
                Some(l) => Zval::Str(PhpStr::new(l.into_bytes())),
                None => Zval::Null,
            });
            Zval::Array(Rc::new(out))
        }
        None => Zval::Bool(false),
    })
}

// --- getdate / localtime (step 35-4, D-PD2) -----------------------------------
// Pure builtins returning arrays of broken-down time components; they touch no
// objects, so they live here rather than in the prelude.

fn str_zval(s: &str) -> Zval {
    Zval::Str(PhpStr::new(s.as_bytes().to_vec()))
}

/// `getdate(?int $timestamp = null)`: the components of `$timestamp` (default
/// now) as an associative array, with a trailing numeric `0` => the timestamp.
/// Key order mirrors PHP exactly.
pub fn getdate(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let ts = match args.first() {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    // Broken-down components are local (default-timezone) wall time.
    let local = ts.saturating_add(zone_view(&default_zone(), ts).off);
    let dt = OffsetDateTime::from_unix_timestamp(local)
        .map_err(|_| PhpError::ValueError("getdate(): timestamp out of range".to_string()))?;
    let wday = dt.weekday().number_days_from_sunday() as usize;
    let mon = u8::from(dt.month()) as usize;
    let mut arr = PhpArray::new();
    arr.insert(Key::from_bytes(b"seconds"), Zval::Long(dt.second() as i64));
    arr.insert(Key::from_bytes(b"minutes"), Zval::Long(dt.minute() as i64));
    arr.insert(Key::from_bytes(b"hours"), Zval::Long(dt.hour() as i64));
    arr.insert(Key::from_bytes(b"mday"), Zval::Long(dt.day() as i64));
    arr.insert(Key::from_bytes(b"wday"), Zval::Long(wday as i64));
    arr.insert(Key::from_bytes(b"mon"), Zval::Long(mon as i64));
    arr.insert(Key::from_bytes(b"year"), Zval::Long(dt.year() as i64));
    arr.insert(Key::from_bytes(b"yday"), Zval::Long((dt.ordinal() - 1) as i64));
    arr.insert(Key::from_bytes(b"weekday"), str_zval(DAYS_FULL[wday]));
    arr.insert(Key::from_bytes(b"month"), str_zval(MONTHS_FULL[mon - 1]));
    arr.insert(Key::Int(0), Zval::Long(ts));
    Ok(Zval::Array(Rc::new(arr)))
}

/// `localtime(?int $timestamp = null, bool $associative = false)`: the C
/// `struct tm` fields of `$timestamp`. Default is a numeric array
/// `[sec,min,hour,mday,mon(0-based),year-1900,wday,yday,isdst]`; with
/// `$associative=true` the same values keyed `tm_*`.
pub fn localtime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let ts = match args.first() {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let assoc = args.get(1).is_some_and(|v| convert::to_bool(v, ctx.diags));
    // Broken-down components are local (default-timezone) wall time.
    let view = zone_view(&default_zone(), ts);
    let dt = OffsetDateTime::from_unix_timestamp(ts.saturating_add(view.off))
        .map_err(|_| PhpError::ValueError("localtime(): timestamp out of range".to_string()))?;
    let fields: [(&[u8], i64); 9] = [
        (b"tm_sec", dt.second() as i64),
        (b"tm_min", dt.minute() as i64),
        (b"tm_hour", dt.hour() as i64),
        (b"tm_mday", dt.day() as i64),
        (b"tm_mon", u8::from(dt.month()) as i64 - 1),
        (b"tm_year", dt.year() as i64 - 1900),
        (b"tm_wday", dt.weekday().number_days_from_sunday() as i64),
        (b"tm_yday", (dt.ordinal() - 1) as i64),
        (b"tm_isdst", i64::from(view.isdst)),
    ];
    let mut arr = PhpArray::new();
    for (i, (name, val)) in fields.iter().enumerate() {
        let key = if assoc {
            Key::from_bytes(name)
        } else {
            Key::Int(i as i64)
        };
        arr.insert(key, Zval::Long(*val));
    }
    Ok(Zval::Array(Rc::new(arr)))
}
