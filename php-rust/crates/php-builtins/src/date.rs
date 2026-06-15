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

use php_runtime::Ctx;
use php_types::{convert, PhpError, PhpStr, Zval};
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
