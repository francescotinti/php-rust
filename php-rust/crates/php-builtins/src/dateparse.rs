//! `date_parse` / `date_parse_from_format` support — a hand port of the timelib
//! date-string grammar (`ext/date/lib/parse_date.re`), producing the same
//! component/relative/timezone breakdown and warning/error arrays PHP returns.
//!
//! The grammar is matched format-family by format-family (absolute dates, times,
//! textual months, day names, relative expressions, keywords, `@timestamp`,
//! timezones). Fields left unset stay `None` → PHP `false`; a time anchor sets
//! h/m/s to 0. Coverage is broad but not yet the full timelib surface; formats
//! that do not match yield an error entry (see `parse`).

use php_runtime::Ctx;
use php_types::{convert, Key, PhpArray, PhpStr, Zval};

/// Relative offset accumulated from relative expressions (`+1 day`, `next month`,
/// `tomorrow`, weekday names).
#[derive(Default, Clone)]
struct Relative {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    weekday: Option<i64>,
    weekday_behavior: Option<i64>,
    /// True once any relative component (including a weekday) has been set.
    present: bool,
}

/// The timezone attached to a parsed string.
#[derive(Clone)]
enum Zone {
    /// A numeric UTC offset in seconds (`zone_type` 1).
    Offset { seconds: i64 },
    /// A named abbreviation such as `UTC`/`GMT`/`CET` (`zone_type` 2/3). A
    /// `zone_type` 2 abbreviation carries its UTC `offset`; a `zone_type` 3
    /// abbreviation (`UTC`) carries a `tz_id` instead.
    Abbr { abbr: String, tz_id: Option<String>, zone_type: i64, offset: i64 },
    /// A geographic timezone id such as `Europe/Rome` (`zone_type` 3).
    Id { id: String },
}

/// The full parse result mirroring `date_parse`'s array.
#[derive(Default)]
struct Parsed {
    year: Option<i64>,
    month: Option<i64>,
    day: Option<i64>,
    hour: Option<i64>,
    minute: Option<i64>,
    second: Option<i64>,
    fraction: Option<f64>,
    relative: Option<Relative>,
    zone: Option<Zone>,
    warnings: Vec<(usize, String)>,
    errors: Vec<(usize, String)>,
}

const MONTHS: &[(&str, i64)] = &[
    ("january", 1), ("february", 2), ("march", 3), ("april", 4), ("may", 5), ("june", 6),
    ("july", 7), ("august", 8), ("september", 9), ("october", 10), ("november", 11),
    ("december", 12), ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4), ("jun", 6), ("jul", 7),
    ("aug", 8), ("sep", 9), ("sept", 9), ("oct", 10), ("nov", 11), ("dec", 12), ("i", 1),
    ("ii", 2), ("iii", 3), ("iv", 4), ("v", 5), ("vi", 6), ("vii", 7), ("viii", 8), ("ix", 9),
    ("x", 10), ("xi", 11), ("xii", 12),
];

const WEEKDAYS: &[(&str, i64)] = &[
    ("sunday", 0), ("monday", 1), ("tuesday", 2), ("wednesday", 3), ("thursday", 4),
    ("friday", 5), ("saturday", 6), ("sun", 0), ("mon", 1), ("tue", 2), ("tues", 2), ("wed", 3),
    ("thu", 4), ("thur", 4), ("thurs", 4), ("fri", 5), ("sat", 6),
];

const REL_UNITS: &[(&str, u8)] = &[
    // 0=sec 1=min 2=hour 3=day 4=week 5=month 6=year 7=fortnight
    ("sec", 0), ("secs", 0), ("second", 0), ("seconds", 0), ("min", 1), ("mins", 1),
    ("minute", 1), ("minutes", 1), ("hour", 2), ("hours", 2), ("day", 3), ("days", 3),
    ("week", 4), ("weeks", 4), ("fortnight", 7), ("fortnights", 7), ("month", 5), ("months", 5),
    ("year", 6), ("years", 6),
];

fn month_num(w: &str) -> Option<i64> {
    let lw = w.to_ascii_lowercase();
    MONTHS.iter().find(|(n, _)| *n == lw).map(|(_, v)| *v)
}
fn weekday_num(w: &str) -> Option<i64> {
    let lw = w.to_ascii_lowercase();
    WEEKDAYS.iter().find(|(n, _)| *n == lw).map(|(_, v)| *v)
}

impl Relative {
    fn add_unit(&mut self, n: i64, unit: u8) {
        match unit {
            0 => self.second += n,
            1 => self.minute += n,
            2 => self.hour += n,
            3 => self.day += n,
            4 => self.day += n * 7,
            7 => self.day += n * 14,
            5 => self.month += n,
            6 => self.year += n,
            _ => {}
        }
        self.present = true;
    }
}

/// Parse `input` into the timelib-style breakdown. Always returns a `Parsed`;
/// unmatched trailing content becomes an error entry.
fn parse(input: &str) -> Parsed {
    let mut p = Parsed::default();
    if input.is_empty() {
        p.errors.push((0, "Empty string".to_string()));
        return p;
    }
    let trimmed = input.trim();
    if trimmed.is_empty() {
        // Whitespace only → all false, no errors.
        return p;
    }

    // `@<int>` unix timestamp.
    if let Some(rest) = trimmed.strip_prefix('@') {
        if let Ok(ts) = rest.trim().parse::<i64>() {
            p.year = Some(1970);
            p.month = Some(1);
            p.day = Some(1);
            p.hour = Some(0);
            p.minute = Some(0);
            p.second = Some(0);
            p.fraction = Some(0.0);
            let mut r = Relative::default();
            r.second = ts;
            r.present = true;
            p.relative = Some(r);
            p.zone = Some(Zone::Offset { seconds: 0 });
            return p;
        }
    }

    // Replace an ISO-8601 `T` that sits between digits with a space so the date
    // and time become separate tokens (a bare "UTC" must survive intact).
    let mut b: Vec<u8> = trimmed.bytes().collect();
    for i in 1..b.len().saturating_sub(1) {
        if (b[i] == b'T' || b[i] == b't') && b[i - 1].is_ascii_digit() && b[i + 1].is_ascii_digit() {
            b[i] = b' ';
        }
    }
    let normalized = String::from_utf8_lossy(&b).into_owned();
    // Tokenize on whitespace and commas (a comma is a separator in "June 15, 2023").
    let tokens: Vec<String> = tokenize(&normalized);
    let mut rel = Relative::default();
    let mut i = 0;
    let mut matched_any = false;
    while i < tokens.len() {
        let tok = &tokens[i];
        let low = tok.to_ascii_lowercase();

        // Keywords.
        match low.as_str() {
            "now" => {
                i += 1;
                matched_any = true;
                continue;
            }
            "today" | "midnight" => {
                set_time_anchor(&mut p, 0, 0, 0);
                i += 1;
                matched_any = true;
                continue;
            }
            "noon" => {
                set_time_anchor(&mut p, 12, 0, 0);
                i += 1;
                matched_any = true;
                continue;
            }
            "tomorrow" => {
                rel.day += 1;
                rel.present = true;
                set_time_anchor(&mut p, 0, 0, 0);
                i += 1;
                matched_any = true;
                continue;
            }
            "yesterday" => {
                rel.day -= 1;
                rel.present = true;
                set_time_anchor(&mut p, 0, 0, 0);
                i += 1;
                matched_any = true;
                continue;
            }
            _ => {}
        }

        // "[+-]N unit" relative — checked before the timezone so "+1 day" is a
        // relative offset rather than a "+1" hour zone.
        if let Some((n, consumed)) = parse_signed_int(&tokens, i) {
            if i + consumed < tokens.len() {
                let unit_tok = tokens[i + consumed].to_ascii_lowercase();
                if let Some((_, unit)) = REL_UNITS.iter().find(|(u, _)| *u == unit_tok) {
                    rel.add_unit(n, *unit);
                    i += consumed + 1;
                    matched_any = true;
                    continue;
                }
            }
        }

        // Timezone token (UTC/GMT/Z, offset, or Area/City id).
        if let Some(z) = parse_zone(tok) {
            p.zone = Some(z);
            i += 1;
            matched_any = true;
            continue;
        }

        // Weekday name → relative weekday.
        if let Some(wd) = weekday_num(tok) {
            rel.weekday = Some(wd);
            rel.present = true;
            set_time_anchor(&mut p, 0, 0, 0);
            i += 1;
            matched_any = true;
            continue;
        }

        // "next"/"last"/"this" <unit|weekday>.
        if matches!(low.as_str(), "next" | "last" | "this") && i + 1 < tokens.len() {
            let n = match low.as_str() {
                "next" => 1,
                "last" => -1,
                _ => 0,
            };
            let nxt = tokens[i + 1].to_ascii_lowercase();
            if let Some((_, unit)) = REL_UNITS.iter().find(|(u, _)| *u == nxt) {
                rel.add_unit(n, *unit);
                // timelib: a modifier + "week" also pins the weekday to 1 (the
                // ISO week start), unlike a bare "+1 week".
                if *unit == 4 {
                    rel.weekday = Some(1);
                }
                i += 2;
                matched_any = true;
                continue;
            }
            if let Some(wd) = weekday_num(&tokens[i + 1]) {
                rel.weekday = Some(wd);
                // timelib: "last <weekday>" carries a -7 day offset; "next"/"this"
                // carry none.
                if n < 0 {
                    rel.day -= 7;
                }
                rel.present = true;
                set_time_anchor(&mut p, 0, 0, 0);
                i += 2;
                matched_any = true;
                continue;
            }
        }

        // Time token (H:i[:s[.f]] with optional am/pm, or "2pm").
        if let Some(consumed) = try_time(&tokens, i, &mut p) {
            i += consumed;
            matched_any = true;
            continue;
        }

        // Absolute date (numeric or textual).
        if let Some(consumed) = try_date(&tokens, i, &mut p) {
            i += consumed;
            matched_any = true;
            continue;
        }

        // Unrecognized token.
        p.errors.push((0, "Unexpected character".to_string()));
        i += 1;
    }

    if rel.present {
        p.relative = Some(rel);
    }
    if !matched_any && p.errors.is_empty() {
        p.errors.push((0, "Unexpected character".to_string()));
    }
    p
}

/// Set the hour/minute/second anchor (used by day-only/keyword forms that imply
/// midnight); leaves an already-set time untouched.
fn set_time_anchor(p: &mut Parsed, h: i64, m: i64, s: i64) {
    if p.hour.is_none() {
        p.hour = Some(h);
        p.minute = Some(m);
        p.second = Some(s);
        p.fraction = Some(0.0);
    }
}

/// Split on whitespace, keeping a trailing/leading comma as its own break.
fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| c.is_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Parse an optional-signed integer starting at token `i`. A leading `+`/`-`
/// may be its own token or attached. Returns `(value, tokens_consumed)`.
fn parse_signed_int(tokens: &[String], i: usize) -> Option<(i64, usize)> {
    let t = &tokens[i];
    if let Ok(n) = t.parse::<i64>() {
        return Some((n, 1));
    }
    if (t == "+" || t == "-") && i + 1 < tokens.len() {
        if let Ok(n) = tokens[i + 1].parse::<i64>() {
            return Some((if t == "-" { -n } else { n }, 2));
        }
    }
    None
}

/// Recognise a timezone token: `UTC`/`GMT`/`Z` (abbr), a numeric offset
/// (`+02:00`, `-0500`, `+02`), or an `Area/City` id.
fn parse_zone(tok: &str) -> Option<Zone> {
    match tok.to_ascii_uppercase().as_str() {
        // `UTC` is a geographic id (zone_type 3); `GMT`/`Z` are abbreviations
        // with a zero offset (zone_type 2).
        "UTC" => return Some(Zone::Abbr { abbr: "UTC".into(), tz_id: Some("UTC".into()), zone_type: 3, offset: 0 }),
        "GMT" => return Some(Zone::Abbr { abbr: "GMT".into(), tz_id: None, zone_type: 2, offset: 0 }),
        "Z" => return Some(Zone::Abbr { abbr: "Z".into(), tz_id: None, zone_type: 2, offset: 0 }),
        _ => {}
    }
    // Numeric offset.
    if let Some(z) = parse_offset(tok) {
        return Some(Zone::Offset { seconds: z });
    }
    // Area/City id (letters and one or more '/'), e.g. Europe/Rome.
    if tok.contains('/')
        && tok.split('/').all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphabetic() || c == '_' || c == '-' || c == '+'))
    {
        return Some(Zone::Id { id: tok.to_string() });
    }
    None
}

/// Parse a `[+-]HH[:MM]` / `[+-]HHMM` offset into seconds.
fn parse_offset(tok: &str) -> Option<i64> {
    let (sign, body) = match tok.strip_prefix('+') {
        Some(b) => (1i64, b),
        None => match tok.strip_prefix('-') {
            Some(b) => (-1i64, b),
            None => return None,
        },
    };
    let digits: String = body.chars().filter(|c| *c != ':').collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let (h, m) = match digits.len() {
        1 | 2 => (digits.parse::<i64>().ok()?, 0),
        3 => (digits[..1].parse().ok()?, digits[1..].parse().ok()?),
        4 => (digits[..2].parse().ok()?, digits[2..].parse().ok()?),
        _ => return None,
    };
    Some(sign * (h * 3600 + m * 60))
}

/// Try to parse a time starting at token `i`. Handles `H:i[:s[.f]]` with an
/// optional following/attached am/pm, and the bare `2pm` form.
fn try_time(tokens: &[String], i: usize, p: &mut Parsed) -> Option<usize> {
    let tok = &tokens[i];
    let mut low = tok.to_ascii_lowercase();
    // A leading ISO `T` designator before a time (`T14:30`).
    if low.starts_with('t') && low[1..].contains(':') {
        low = low[1..].to_string();
    }
    // Strip a timezone attached to the time (`14:30:45+02:00`, `14:30:45Z`) —
    // only for an actual time (contains ':'), so a bare offset stays a zone.
    let mut attached_zone: Option<Zone> = None;
    if low.contains(':') {
        if let Some(stripped) = low.strip_suffix('z') {
            attached_zone = Some(Zone::Abbr { abbr: "Z".into(), tz_id: None, zone_type: 2, offset: 0 });
            low = stripped.to_string();
        } else if let Some(pos) = low[1..].find(['+', '-']).map(|x| x + 1) {
            if let Some(secs) = parse_offset(&low[pos..]) {
                attached_zone = Some(Zone::Offset { seconds: secs });
                low = low[..pos].to_string();
            }
        }
    }
    // Split an attached am/pm suffix (e.g. "2pm", "2:30pm").
    let (core, mut ampm) = if let Some(c) = low.strip_suffix("am") {
        (c.to_string(), Some(false))
    } else if let Some(c) = low.strip_suffix("pm") {
        (c.to_string(), Some(true))
    } else {
        (low.clone(), None)
    };

    let mut consumed = 1;
    // A following standalone am/pm token.
    if ampm.is_none() && i + 1 < tokens.len() {
        match tokens[i + 1].to_ascii_lowercase().as_str() {
            "am" | "a.m." => {
                ampm = Some(false);
                consumed = 2;
            }
            "pm" | "p.m." => {
                ampm = Some(true);
                consumed = 2;
            }
            _ => {}
        }
    }

    if core.contains(':') {
        let parts: Vec<&str> = core.split(':').collect();
        if !(2..=3).contains(&parts.len()) {
            return None;
        }
        let mut h: i64 = parts[0].parse().ok()?;
        let mi: i64 = parts[1].parse().ok()?;
        let (s, frac) = if parts.len() == 3 {
            let sp: Vec<&str> = parts[2].splitn(2, '.').collect();
            let s: i64 = sp[0].parse().ok()?;
            let frac = if sp.len() == 2 {
                format!("0.{}", sp[1]).parse::<f64>().ok()?
            } else {
                0.0
            };
            (s, frac)
        } else {
            (0, 0.0)
        };
        if let Some(pm) = ampm {
            h = apply_ampm(h, pm)?;
        }
        if !(0..=24).contains(&h) || !(0..=59).contains(&mi) || !(0..=59).contains(&s) {
            return None;
        }
        p.hour = Some(h);
        p.minute = Some(mi);
        p.second = Some(s);
        p.fraction = Some(frac);
        if let Some(z) = attached_zone {
            p.zone = Some(z);
        }
        return Some(consumed);
    }

    // Bare hour with am/pm ("2pm").
    if ampm.is_some() && !core.is_empty() && core.chars().all(|c| c.is_ascii_digit()) {
        let mut h: i64 = core.parse().ok()?;
        h = apply_ampm(h, ampm.unwrap())?;
        p.hour = Some(h);
        p.minute = Some(0);
        p.second = Some(0);
        p.fraction = Some(0.0);
        return Some(consumed);
    }
    None
}

/// 12-hour → 24-hour conversion (`12am`→0, `12pm`→12, `1pm`→13).
fn apply_ampm(h: i64, pm: bool) -> Option<i64> {
    if !(1..=12).contains(&h) {
        return None;
    }
    Some(match (h, pm) {
        (12, false) => 0,
        (12, true) => 12,
        (h, false) => h,
        (h, true) => h + 12,
    })
}

/// Try to parse an absolute date starting at token `i`. Handles `Y-m-d`,
/// `Y/m/d`, and textual forms `d M Y` / `M d Y`.
fn try_date(tokens: &[String], i: usize, p: &mut Parsed) -> Option<usize> {
    let tok = &tokens[i];

    // Numeric three-part date. `-`/`/` disambiguation follows timelib: a 4-digit
    // first group is `Y-m-d`; otherwise a 4-digit last group is `d-m-Y` (dashes)
    // or `m/d/Y` (slashes, American); with no 4-digit group, `Y-m-d`.
    for sep in ['-', '/'] {
        if tok.matches(sep).count() == 2 {
            let parts: Vec<&str> = tok.split(sep).collect();
            if parts.iter().any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit())) {
                return None;
            }
            let (y, m, d): (i64, i64, i64) = if parts[0].len() == 4 {
                (parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?)
            } else if parts[2].len() == 4 {
                let y = parts[2].parse().ok()?;
                if sep == '/' {
                    (y, parts[0].parse().ok()?, parts[1].parse().ok()?)
                } else {
                    (y, parts[1].parse().ok()?, parts[0].parse().ok()?)
                }
            } else {
                (parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?)
            };
            if valid_ymd(y, m, d) {
                p.year = Some(y);
                p.month = Some(m);
                p.day = Some(d);
                return Some(1);
            }
            return None;
        }
    }

    // Textual: "d Month Y" or "Month d Y" (3 tokens).
    if i + 2 < tokens.len() + 1 {
        // d Month Y
        if let (Ok(d), Some(m), Some(y)) = (
            tokens[i].parse::<i64>(),
            tokens.get(i + 1).and_then(|t| month_num(t)),
            tokens.get(i + 2).and_then(|t| t.parse::<i64>().ok()),
        ) {
            if valid_ymd(y, m, d) {
                p.year = Some(y);
                p.month = Some(m);
                p.day = Some(d);
                return Some(3);
            }
        }
        // Month d Y
        if let (Some(m), Some(d), Some(y)) = (
            month_num(&tokens[i]),
            tokens.get(i + 1).and_then(|t| t.parse::<i64>().ok()),
            tokens.get(i + 2).and_then(|t| t.parse::<i64>().ok()),
        ) {
            if valid_ymd(y, m, d) {
                p.year = Some(y);
                p.month = Some(m);
                p.day = Some(d);
                return Some(3);
            }
        }
    }
    None
}

fn valid_ymd(_y: i64, m: i64, d: i64) -> bool {
    (1..=12).contains(&m) && (1..=31).contains(&d)
}

/// Build the `date_parse` array from a [`Parsed`].
fn build_array(p: &Parsed) -> Zval {
    let mut a = PhpArray::new();
    let put = |a: &mut PhpArray, k: &[u8], v: Zval| a.insert(Key::from_bytes(k), v);
    let opt = |o: Option<i64>| o.map_or(Zval::Bool(false), Zval::Long);

    put(&mut a, b"year", opt(p.year));
    put(&mut a, b"month", opt(p.month));
    put(&mut a, b"day", opt(p.day));
    put(&mut a, b"hour", opt(p.hour));
    put(&mut a, b"minute", opt(p.minute));
    put(&mut a, b"second", opt(p.second));
    put(
        &mut a,
        b"fraction",
        p.fraction.map_or(Zval::Bool(false), Zval::Double),
    );
    put(&mut a, b"warning_count", Zval::Long(p.warnings.len() as i64));
    put(&mut a, b"warnings", messages_array(&p.warnings));
    put(&mut a, b"error_count", Zval::Long(p.errors.len() as i64));
    put(&mut a, b"errors", messages_array(&p.errors));
    put(&mut a, b"is_localtime", Zval::Bool(p.zone.is_some()));

    if let Some(z) = &p.zone {
        match z {
            Zone::Offset { seconds } => {
                put(&mut a, b"zone_type", Zval::Long(1));
                put(&mut a, b"zone", Zval::Long(*seconds));
                put(&mut a, b"is_dst", Zval::Bool(false));
            }
            Zone::Abbr { abbr, tz_id, zone_type, offset } => {
                put(&mut a, b"zone_type", Zval::Long(*zone_type));
                if *zone_type == 2 {
                    // Abbreviation: offset + dst flag, then the abbreviation.
                    put(&mut a, b"zone", Zval::Long(*offset));
                    put(&mut a, b"is_dst", Zval::Bool(false));
                    put(&mut a, b"tz_abbr", str_zv(abbr));
                } else {
                    put(&mut a, b"tz_abbr", str_zv(abbr));
                    if let Some(id) = tz_id {
                        put(&mut a, b"tz_id", str_zv(id));
                    }
                }
            }
            Zone::Id { id } => {
                put(&mut a, b"zone_type", Zval::Long(3));
                put(&mut a, b"tz_id", str_zv(id));
            }
        }
    }

    if let Some(r) = &p.relative {
        let mut ra = PhpArray::new();
        ra.insert(Key::from_bytes(b"year"), Zval::Long(r.year));
        ra.insert(Key::from_bytes(b"month"), Zval::Long(r.month));
        ra.insert(Key::from_bytes(b"day"), Zval::Long(r.day));
        ra.insert(Key::from_bytes(b"hour"), Zval::Long(r.hour));
        ra.insert(Key::from_bytes(b"minute"), Zval::Long(r.minute));
        ra.insert(Key::from_bytes(b"second"), Zval::Long(r.second));
        if let Some(wd) = r.weekday {
            ra.insert(Key::from_bytes(b"weekday"), Zval::Long(wd));
        }
        if let Some(wb) = r.weekday_behavior {
            ra.insert(Key::from_bytes(b"weekday_behavior"), Zval::Long(wb));
        }
        put(&mut a, b"relative", Zval::Array(std::rc::Rc::new(ra)));
    }

    Zval::Array(std::rc::Rc::new(a))
}

fn str_zv(s: &str) -> Zval {
    Zval::Str(PhpStr::new(s.as_bytes().to_vec()))
}

/// Build the `warnings`/`errors` associative array (position → message).
fn messages_array(msgs: &[(usize, String)]) -> Zval {
    let mut a = PhpArray::new();
    for (pos, m) in msgs {
        a.insert(Key::Int(*pos as i64), str_zv(m));
    }
    Zval::Array(std::rc::Rc::new(a))
}

/// `date_parse(string $datetime): array` — parse `$datetime` into its
/// components, relative offsets, timezone, and any warnings/errors.
pub fn date_parse(args: &[Zval], ctx: &mut Ctx) -> Result<Zval, php_types::PhpError> {
    let s = convert::to_zstr(
        args.first().ok_or_else(|| {
            php_types::PhpError::Error("date_parse() expects exactly 1 argument, 0 given".to_string())
        })?,
        ctx.diags,
    );
    let text = String::from_utf8_lossy(s.as_bytes()).into_owned();
    Ok(build_array(&parse(&text)))
}
