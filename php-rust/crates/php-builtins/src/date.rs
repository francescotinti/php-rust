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
/// `dt` is the instant interpreted in UTC.
fn append_char(out: &mut Vec<u8>, c: u8, dt: &OffsetDateTime, epoch: i64) {
    let year = dt.year();
    let month = u8::from(dt.month());
    let day = dt.day();
    let hour = dt.hour();
    let minute = dt.minute();
    let second = dt.second();
    // PHP w: 0=Sunday..6=Saturday. PHP N: 1=Monday..7=Sunday.
    let dow_sun0 = dt.weekday().number_days_from_sunday();
    let push = |out: &mut Vec<u8>, s: &str| out.extend_from_slice(s.as_bytes());
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
        b'B' => push(out, &format!("{:03}", swatch_beats(hour, minute, second))),
        // --- Timezone (UTC only, D-DT3) ---
        b'e' => push(out, "UTC"),
        b'T' => push(out, "UTC"),
        b'I' => push(out, "0"),
        b'O' => push(out, "+0000"),
        b'P' => push(out, "+00:00"),
        b'Z' => push(out, "0"),
        // --- Full date/time ---
        b'c' => {
            // ISO 8601: Y-m-d\TH:i:sP
            push(
                out,
                &format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}+00:00"),
            )
        }
        b'r' => {
            // RFC 2822: D, d M Y H:i:s O
            push(
                out,
                &format!(
                    "{}, {day:02} {} {year:04} {hour:02}:{minute:02}:{second:02} +0000",
                    DAYS_SHORT[dow_sun0 as usize],
                    MONTHS_SHORT[(month - 1) as usize],
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

/// Swatch internet time (`B`): beats in Biel Mean Time (UTC+1), 0..999.
fn swatch_beats(hour: u8, minute: u8, second: u8) -> u32 {
    let secs = (hour as u32 + 1) % 24 * 3600 + minute as u32 * 60 + second as u32;
    ((secs as f64) / 86.4) as u32 % 1000
}

/// Format `epoch` (Unix seconds, UTC) per the PHP `fmt` string. Backslash
/// escapes the next byte (emitted literally). Unknown bytes pass through.
pub fn format_php(epoch: i64, fmt: &[u8]) -> Vec<u8> {
    let dt = match OffsetDateTime::from_unix_timestamp(epoch) {
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
        append_char(&mut out, c, &dt, epoch);
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
    Ok(Zval::Str(PhpStr::new(format_php(epoch, fmt.as_bytes()))))
}

/// `gmdate(string $format, ?int $timestamp = null)`. With UTC scope this is
/// identical to `date()`.
pub fn gmdate(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    date(args, ctx)
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
/// ?int $year)`. Omitted trailing components default to the current local
/// (UTC, D-DT3) time — those paths read the real clock and are not
/// differential-tested (D-DT5).
pub fn mktime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
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

/// `gmmktime(...)`. Identical to `mktime` under the UTC scope (D-DT3).
pub fn gmmktime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    mktime(args, ctx)
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

/// Parse an absolute date string in the supported subset: `Y-m-d` or `Y/m/d`,
/// optionally followed by ` `/`T` and `H:i[:s]`. Returns the UTC epoch.
fn parse_absolute(s: &str) -> Option<i64> {
    let s = s.replace('T', " ");
    let mut parts = s.split_whitespace();
    let date = parts.next()?;
    let time = parts.next();
    if parts.next().is_some() {
        return None;
    }
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
    let (mut hour, mut min, mut sec) = (0i64, 0i64, 0i64);
    if let Some(t) = time {
        let mut tp = t.split(':');
        hour = tp.next()?.parse().ok()?;
        min = tp.next()?.parse().ok()?;
        sec = match tp.next() {
            Some(x) => x.parse().ok()?,
            None => 0,
        };
        if tp.next().is_some() {
            return None;
        }
    }
    civil_to_epoch(year, month, day, hour, min, sec)
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

/// Apply a relative expression (`+N unit` / `-N unit`, possibly repeated) to a
/// base epoch, normalizing calendar overflow. `None` if it doesn't parse.
fn parse_relative(s: &str, base: i64) -> Option<i64> {
    let dt = OffsetDateTime::from_unix_timestamp(base).ok()?;
    let (dy, dmo, dd, dh, dmi, ds) = accumulate_relative(s)?;
    civil_to_epoch(
        dt.year() as i64 + dy,
        u8::from(dt.month()) as i64 + dmo,
        dt.day() as i64 + dd,
        dt.hour() as i64 + dh,
        dt.minute() as i64 + dmi,
        dt.second() as i64 + ds,
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
        if vi != v.len() {
            return None;
        }
        civil_to_epoch(yr, mo, d, h, mi, s)
    })();

    Ok(match result {
        Some(ts) => Zval::Long(ts),
        None => Zval::Bool(false),
    })
}

/// `time()`: the current Unix timestamp. Non-deterministic (reads the real
/// clock); not differential-tested (D-DT5).
pub fn time(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Long(now_epoch()))
}

/// `date_default_timezone_set(string $timezoneId)`: always returns `true`. With
/// the UTC-only scope (D-DT3) the timezone is not actually stored — setting a
/// non-UTC zone is a no-op, a documented divergence (formatting stays UTC).
pub fn date_default_timezone_set(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Bool(true))
}

/// `date_default_timezone_get()`: always `"UTC"` (D-DT3 scope).
pub fn date_default_timezone_get(_args: &[Zval], _ctx: &mut Ctx) -> Result<Zval, PhpError> {
    Ok(Zval::Str(PhpStr::new(b"UTC".to_vec())))
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
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();

    let result = if trimmed.is_empty() {
        None
    } else if let Some(rest) = trimmed.strip_prefix('@') {
        rest.parse::<i64>().ok()
    } else if lower == "now" {
        Some(base)
    } else if let Some(ts) = parse_absolute(trimmed) {
        Some(ts)
    } else {
        parse_relative(&lower, base)
    };

    Ok(match result {
        Some(ts) => Zval::Long(ts),
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
    let dt = OffsetDateTime::from_unix_timestamp(ts)
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
/// `$associative=true` the same values keyed `tm_*`. `isdst` is always 0 (UTC,
/// D-DT3).
pub fn localtime(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, PhpError> {
    let ts = match args.first() {
        None | Some(Zval::Null) => now_epoch(),
        Some(v) => convert::to_long_cast(v, ctx.diags),
    };
    let assoc = args.get(1).is_some_and(|v| convert::to_bool(v, ctx.diags));
    let dt = OffsetDateTime::from_unix_timestamp(ts)
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
        (b"tm_isdst", 0),
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
