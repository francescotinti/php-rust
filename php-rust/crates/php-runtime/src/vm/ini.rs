//! The mutable INI table behind `ini_get` / `ini_set` / `ini_restore` /
//! `ini_get_all` (C0 of the ext/session port). Session behaviour is driven
//! entirely by `session.*` directives — Symfony's NativeSessionStorage
//! configures the module through `ini_set()` — so the previous stateless
//! php-builtins stubs (ini_set always `false`) moved host-side onto a real
//! table the session module reads.
//!
//! Correct-or-absent: only registered directives exist — an unregistered name
//! reads and sets as `false`, exactly like an unknown directive in PHP. The
//! engine-hardwired entries phpr reports but does not honour at runtime
//! (`memory_limit`, `precision`, …) refuse `ini_set` and keep their startup
//! value; that divergence is documented in PHPR_DIVERGENCES_FROM_PHP.md.
//! `ini_get_all(null)` therefore lists ~40 directives, not PHP's ~291.
//!
//! Oracle-pinned contracts (PHP 8.5.7 probes, session kickoff 2026-07-11):
//! - `ini_set` returns the OLD local value as a string; unknown directive or
//!   one without `INI_USER` access → `false`, silently.
//! - The `$value` argument accepts `string|int|float|bool|null` only
//!   (TypeError without a "given" suffix otherwise); `false`/`null` store "".
//! - `session.*` sets/restores are refused once output has started:
//!   `Session ini settings cannot be changed after headers have already been
//!   sent (sent from FILE on line N)`.
//! - `session.sid_length` / `session.sid_bits_per_character`: setting any
//!   NON-DEFAULT value raises `Deprecated: ini_set(): session.X INI setting
//!   is deprecated` (even re-setting the same non-default value; restoring
//!   the default is silent).
//! - Integer-typed directives accept any string verbatim but warn through the
//!   zend quantity parser: `Invalid "NAME" setting. Invalid quantity "V":
//!   unknown multiplier "C", interpreting as "P" for backwards compatibility`
//!   (or `no valid leading digits, interpreting as "0"`).
//! - `session.name` refuses numeric/empty/reserved-char values;
//!   `session.save_handler` accepts only "files" ("user" has a dedicated
//!   refusal); `session.serialize_handler` accepts php/php_binary/php_serialize.
//! - `ini_get_all('session')` → 31 sorted entries of
//!   `{global_value, local_value, access}`; `details: false` → name→local map;
//!   unknown extension → Warning `Extension "X" cannot be found` + `false`.

use std::collections::BTreeMap;

use super::*;

/// `ini_get_all()` access bits: PHP_INI_USER — the level `ini_set` needs.
const INI_USER: i64 = 1;
/// PHP_INI_PERDIR (php.ini / .htaccess only): `session.auto_start` and the
/// `session.upload_progress.*` family — `ini_set` refuses them silently.
const INI_PERDIR: i64 = 2;
/// PHP_INI_SYSTEM (php.ini only): the hardwired allow_url_fopen family.
const INI_SYSTEM: i64 = 4;
/// PHP_INI_ALL.
const INI_ALL: i64 = 7;

/// One registered directive: its startup default (`global` — what
/// `ini_restore` reverts to), current value (`local`), and set-behaviour.
pub(super) struct IniEntry {
    pub global: Vec<u8>,
    pub local: Vec<u8>,
    pub access: i64,
    /// Whether phpr honours a runtime change. Engine-hardwired reads
    /// (memory_limit, precision, …) report their access level faithfully but
    /// refuse `ini_set` — better absent than a set that silently lies.
    settable: bool,
    /// zend "quantity" (integer) directive: a non-integer value still stores
    /// verbatim, but goes through the quantity-parser warning.
    int_typed: bool,
    /// `session.sid_length`/`session.sid_bits_per_character`: `ini_set` to any
    /// value other than the default raises a Deprecated.
    deprecated_off_default: bool,
}

/// The table, ordered by directive name (`ini_get_all` lists alphabetically).
pub(super) struct IniTable(pub BTreeMap<Vec<u8>, IniEntry>);

impl IniTable {
    pub(super) fn new() -> Self {
        let mut t = BTreeMap::new();
        let mut add = |name: &str, default: &str, access: i64, settable: bool, int_typed: bool| {
            t.insert(
                name.as_bytes().to_vec(),
                IniEntry {
                    global: default.as_bytes().to_vec(),
                    local: default.as_bytes().to_vec(),
                    access,
                    settable,
                    int_typed,
                    deprecated_off_default: name.starts_with("session.sid_"),
                },
            );
        };
        // The engine-hardwired directives the retired php-builtins ini_get
        // reported, same values (real code branches on them; Composer needs
        // allow_url_fopen "1" and memory_limit "-1", see the old env.rs notes).
        add("allow_url_fopen", "1", INI_SYSTEM, false, false);
        add("allow_url_include", "", INI_SYSTEM, false, false);
        add("disable_functions", "", INI_SYSTEM, false, false);
        add("enable_dl", "", INI_SYSTEM, false, false);
        add("memory_limit", "-1", INI_ALL, false, false);
        add("max_execution_time", "0", INI_ALL, false, false);
        add("default_socket_timeout", "60", INI_ALL, false, false);
        add("precision", "14", INI_ALL, false, false);
        add("serialize_precision", "-1", INI_ALL, false, false);
        // ext/session (31 directives, defaults from the 8.5.7 CLI oracle).
        add("session.auto_start", "0", INI_PERDIR, false, false);
        add("session.cache_expire", "180", INI_ALL, true, true);
        add("session.cache_limiter", "nocache", INI_ALL, true, false);
        add("session.cookie_domain", "", INI_ALL, true, false);
        add("session.cookie_httponly", "", INI_ALL, true, false);
        add("session.cookie_lifetime", "0", INI_ALL, true, true);
        add("session.cookie_partitioned", "0", INI_ALL, true, false);
        add("session.cookie_path", "/", INI_ALL, true, false);
        add("session.cookie_samesite", "", INI_ALL, true, false);
        add("session.cookie_secure", "0", INI_ALL, true, false);
        add("session.gc_divisor", "1000", INI_ALL, true, true);
        add("session.gc_maxlifetime", "1440", INI_ALL, true, true);
        add("session.gc_probability", "1", INI_ALL, true, true);
        add("session.lazy_write", "1", INI_ALL, true, false);
        add("session.name", "PHPSESSID", INI_ALL, true, false);
        add("session.referer_check", "", INI_ALL, true, false);
        add("session.save_handler", "files", INI_ALL, true, false);
        add("session.save_path", "", INI_ALL, true, false);
        add("session.serialize_handler", "php", INI_ALL, true, false);
        add("session.sid_bits_per_character", "4", INI_ALL, true, true);
        add("session.sid_length", "32", INI_ALL, true, true);
        add("session.upload_progress.cleanup", "1", INI_PERDIR, false, false);
        add("session.upload_progress.enabled", "1", INI_PERDIR, false, false);
        add("session.upload_progress.freq", "1%", INI_PERDIR, false, false);
        add("session.upload_progress.min_freq", "1", INI_PERDIR, false, false);
        add(
            "session.upload_progress.name",
            "PHP_SESSION_UPLOAD_PROGRESS",
            INI_PERDIR,
            false,
            false,
        );
        add("session.upload_progress.prefix", "upload_progress_", INI_PERDIR, false, false);
        add("session.use_cookies", "1", INI_ALL, true, false);
        add("session.use_only_cookies", "1", INI_ALL, true, false);
        add("session.use_strict_mode", "0", INI_ALL, true, false);
        add("session.use_trans_sid", "0", INI_ALL, true, false);
        // Oracle oddity: these two ARE settable session directives, but PHP
        // exempts them from the headers-already-sent freeze and omits them
        // from ini_get_all('session') — see TRANS_SID_EXEMPT.
        add("session.trans_sid_tags", "a=href,area=href,frame=src,form=", INI_ALL, true, false);
        add("session.trans_sid_hosts", "", INI_ALL, true, false);
        IniTable(t)
    }

    /// The current (local) value of a directive, or `None` when unregistered.
    /// The session module reads its configuration through this.
    pub(super) fn get(&self, name: &[u8]) -> Option<&[u8]> {
        self.0.get(name).map(|e| e.local.as_slice())
    }

    /// A directive parsed as an integer the way the session module consumes
    /// its int-typed settings: leading integer digits of the stored string
    /// (the zend quantity parser's backwards-compatible reading), 0 when none.
    #[allow(dead_code)] // consumed by the session module (C1 of the port)
    pub(super) fn get_long(&self, name: &[u8]) -> i64 {
        self.get(name).map_or(0, |v| leading_long(v))
    }

    /// A directive read as PHP's INI boolean (On/True/Yes/1 → true).
    #[allow(dead_code)] // consumed by the session module (C1 of the port)
    pub(super) fn get_bool(&self, name: &[u8]) -> bool {
        self.get(name).is_some_and(ini_bool)
    }
}

/// PHP's `zend_ini_parse_bool`: "1"/"true"/"yes"/"on" (case-insensitive) are
/// true, any nonzero leading number too; everything else is false.
fn ini_bool(v: &[u8]) -> bool {
    v.eq_ignore_ascii_case(b"true")
        || v.eq_ignore_ascii_case(b"yes")
        || v.eq_ignore_ascii_case(b"on")
        || leading_long(v) != 0
}

/// The leading `[+-]?digits` prefix of `v` as an i64 (0 when there is none) —
/// the "interpreting as" fallback of the zend quantity parser.
fn leading_long(v: &[u8]) -> i64 {
    let s = trim_ascii(v);
    let (sign, digits) = match s.first() {
        Some(b'-') => (-1i64, &s[1..]),
        Some(b'+') => (1, &s[1..]),
        _ => (1, s),
    };
    let end = digits.iter().position(|b| !b.is_ascii_digit()).unwrap_or(digits.len());
    std::str::from_utf8(&digits[..end])
        .ok()
        .and_then(|d| d.parse::<i64>().ok())
        .map_or(0, |n| sign * n)
}

fn trim_ascii(v: &[u8]) -> &[u8] {
    let start = v.iter().position(|b| !b.is_ascii_whitespace()).unwrap_or(v.len());
    let end = v.iter().rposition(|b| !b.is_ascii_whitespace()).map_or(start, |p| p + 1);
    &v[start..end]
}

/// Whether `v` is a clean quantity the zend parser accepts silently: an
/// optional sign, digits, and at most one K/M/G multiplier suffix.
fn quantity_is_clean(v: &[u8]) -> bool {
    let s = trim_ascii(v);
    if s.is_empty() {
        // The parser treats an empty string as 0 without a warning.
        return true;
    }
    let digits = match s.first() {
        Some(b'-' | b'+') => &s[1..],
        _ => s,
    };
    let end = digits.iter().position(|b| !b.is_ascii_digit()).unwrap_or(digits.len());
    let rest = &digits[end..];
    end > 0 && (rest.is_empty() || (rest.len() == 1 && matches!(rest[0], b'k' | b'K' | b'm' | b'M' | b'g' | b'G')))
}

/// The characters `session.name` refuses, in the exact spelling of the warning.
const SESSION_NAME_FORBIDDEN: &[u8] = b"=,;.[ \t\r\n\x0b\x0c";

/// The two `session.*` directives PHP exempts from the session-ini freeze and
/// from the `ini_get_all('session')` listing (oracle-verified oddity).
fn trans_sid_exempt(name: &[u8]) -> bool {
    name == b"session.trans_sid_tags" || name == b"session.trans_sid_hosts"
}

impl<'m> Vm<'m> {
    /// `ini_get(string $option): string|false` — the current local value of a
    /// registered directive; `false` (no diagnostic) otherwise. Case-sensitive.
    pub(super) fn ho_ini_get(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.ini_string_arg(&args, 0, b"ini_get", b"$option")?;
        Ok(match self.ini.get(&name) {
            Some(v) => Zval::Str(PhpStr::new(v.to_vec())),
            None => Zval::Bool(false),
        })
    }

    /// `ini_set(string $option, string|int|float|bool|null $value): string|false`
    /// — stores the stringified value and returns the previous one.
    pub(super) fn ho_ini_set(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.ini_string_arg(&args, 0, b"ini_set", b"$option")?;
        let value = match args.get(1).map(deref_zval) {
            Some(Zval::Str(_) | Zval::Long(_) | Zval::Double(_) | Zval::Bool(_) | Zval::Null) | None => {
                let v = args.get(1).cloned().unwrap_or(Zval::Null);
                convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec()
            }
            // Oracle-pinned: no ", array given" suffix on this union message.
            Some(_) => {
                return Err(PhpError::TypeError(
                    "ini_set(): Argument #2 ($value) must be of type string|int|float|bool|null"
                        .to_string(),
                ))
            }
        };
        if !self.ini_session_change_allowed(&name, b"ini_set") {
            return Ok(Zval::Bool(false));
        }
        let Some(entry) = self.ini.0.get(&name) else {
            return Ok(Zval::Bool(false));
        };
        if !entry.settable || entry.access & INI_USER == 0 {
            return Ok(Zval::Bool(false));
        }
        // Per-directive validation, oracle-pinned warnings (a refused value
        // leaves the entry untouched and returns false).
        if name == b"session.name" && !session_name_ok(&value) {
            self.diags.push(Diag::Warning(format!(
                "ini_set(): session.name \"{}\" must not be numeric, empty, contain null bytes \
                 or any of the following characters \"=,;.[ \\t\\r\\n\\013\\014\"",
                String::from_utf8_lossy(&value)
            )));
            return Ok(Zval::Bool(false));
        }
        if name == b"session.save_handler" && value != b"files" {
            self.diags.push(Diag::Warning(if value == b"user" {
                "ini_set(): Session save handler \"user\" cannot be set by ini_set()".to_string()
            } else {
                format!(
                    "ini_set(): Session save handler \"{}\" cannot be found",
                    String::from_utf8_lossy(&value)
                )
            }));
            return Ok(Zval::Bool(false));
        }
        if name == b"session.serialize_handler"
            && !matches!(&value[..], b"php" | b"php_binary" | b"php_serialize")
        {
            self.diags.push(Diag::Warning(format!(
                "ini_set(): Serialization handler \"{}\" cannot be found",
                String::from_utf8_lossy(&value)
            )));
            return Ok(Zval::Bool(false));
        }
        if entry.deprecated_off_default && value != entry.global {
            self.diags.push(Diag::Deprecated(format!(
                "ini_set(): {} INI setting is deprecated",
                String::from_utf8_lossy(&name)
            )));
        }
        if entry.int_typed && !quantity_is_clean(&value) {
            let s = trim_ascii(&value);
            let detail = if leading_long(s) == 0 && !s.first().is_some_and(|b| b.is_ascii_digit()) {
                "no valid leading digits, interpreting as \"0\"".to_string()
            } else {
                format!(
                    "unknown multiplier \"{}\", interpreting as \"{}\"",
                    String::from_utf8_lossy(&s[s.len() - 1..]),
                    leading_long(s)
                )
            };
            self.diags.push(Diag::Warning(format!(
                "Invalid \"{}\" setting. Invalid quantity \"{}\": {} for backwards compatibility",
                String::from_utf8_lossy(&name),
                String::from_utf8_lossy(&value),
                detail
            )));
        }
        let entry = self.ini.0.get_mut(&name).expect("checked above");
        let old = std::mem::replace(&mut entry.local, value);
        Ok(Zval::Str(PhpStr::new(old)))
    }

    /// `ini_restore(string $option): void` — reverts a directive to its startup
    /// value; silent for unknown names.
    pub(super) fn ho_ini_restore(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.ini_string_arg(&args, 0, b"ini_restore", b"$option")?;
        if self.ini_session_change_allowed(&name, b"ini_restore") {
            if let Some(entry) = self.ini.0.get_mut(&name) {
                entry.local = entry.global.clone();
            }
        }
        Ok(Zval::Null)
    }

    /// `ini_get_all(?string $extension = null, bool $details = true)` — the
    /// registered directives (all, or the `session` extension's), sorted.
    pub(super) fn ho_ini_get_all(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let ext = match args.first().map(deref_zval) {
            None | Some(Zval::Null) => None,
            Some(v) => Some(convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec()),
        };
        if let Some(ext) = &ext {
            // Only ext/session's directives are registered per-extension; any
            // other name is reported unknown (documented divergence for the
            // extensions PHP would recognise).
            if ext != b"session" {
                self.diags.push(Diag::Warning(format!(
                    "ini_get_all(): Extension \"{}\" cannot be found",
                    String::from_utf8_lossy(ext)
                )));
                return Ok(Zval::Bool(false));
            }
        }
        let details = match args.get(1).map(deref_zval) {
            None => true,
            Some(v) => convert::to_bool(&v, &mut self.diags),
        };
        let mut out = PhpArray::new();
        for (name, e) in &self.ini.0 {
            if ext.is_some() && (!name.starts_with(b"session.") || trans_sid_exempt(name)) {
                continue;
            }
            let key = Key::Str(PhpStr::new(name.clone()));
            if details {
                let mut row = PhpArray::new();
                row.insert(
                    Key::Str(PhpStr::from_str("global_value")),
                    Zval::Str(PhpStr::new(e.global.clone())),
                );
                row.insert(
                    Key::Str(PhpStr::from_str("local_value")),
                    Zval::Str(PhpStr::new(e.local.clone())),
                );
                row.insert(Key::Str(PhpStr::from_str("access")), Zval::Long(e.access));
                out.insert(key, Zval::Array(Rc::new(row)));
            } else {
                out.insert(key, Zval::Str(PhpStr::new(e.local.clone())));
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// The `session.*` change guard shared by `ini_set`/`ini_restore` (and by
    /// the session module's own setters): once output has started, session ini
    /// settings are frozen. Returns `false` (after warning) when refused.
    fn ini_session_change_allowed(&mut self, name: &[u8], func: &[u8]) -> bool {
        if !name.starts_with(b"session.") || trans_sid_exempt(name) {
            return true;
        }
        if self.output_started {
            let sent = match &self.output_start {
                Some((f, l)) => {
                    format!(" (sent from {} on line {})", String::from_utf8_lossy(f), l)
                }
                None => String::new(),
            };
            self.diags.push(Diag::Warning(format!(
                "{}(): Session ini settings cannot be changed after headers have already been sent{}",
                String::from_utf8_lossy(func),
                sent
            )));
            return false;
        }
        true
    }

    /// The string first-or-nth argument of an ini function, with the exact
    /// `func(): Argument #N ($name) must be of type string, X given` TypeError.
    fn ini_string_arg(
        &mut self,
        args: &[Zval],
        idx: usize,
        func: &[u8],
        param: &[u8],
    ) -> Result<Vec<u8>, PhpError> {
        match args.get(idx).map(deref_zval) {
            Some(Zval::Str(s)) => Ok(s.as_bytes().to_vec()),
            Some(Zval::Long(_) | Zval::Double(_) | Zval::Bool(_)) => {
                let v = args[idx].clone();
                Ok(convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec())
            }
            Some(other) => Err(PhpError::TypeError(format!(
                "{}(): Argument #{} ({}) must be of type string, {} given",
                String::from_utf8_lossy(func),
                idx + 1,
                String::from_utf8_lossy(param),
                other.type_name_for_error()
            ))),
            None => Err(PhpError::TypeError(format!(
                "{}() expects at least {} argument, 0 given",
                String::from_utf8_lossy(func),
                idx + 1
            ))),
        }
    }
}

/// `session.name` validation (session.c OnUpdateName): not empty, not a
/// numeric string, no NUL and none of the reserved cookie/URL characters.
fn session_name_ok(v: &[u8]) -> bool {
    !v.is_empty()
        && php_types::numstr::parse_numeric(v).is_none()
        && !v.contains(&0)
        && !v.iter().any(|b| SESSION_NAME_FORBIDDEN.contains(b))
}

/// A `Zval` with any reference layer peeled, cloned shallowly for matching.
fn deref_zval(v: &Zval) -> Zval {
    match v {
        Zval::Ref(c) => c.borrow().clone(),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_defaults_match_oracle() {
        let t = IniTable::new();
        assert_eq!(t.get(b"session.name"), Some(&b"PHPSESSID"[..]));
        assert_eq!(t.get(b"session.save_handler"), Some(&b"files"[..]));
        assert_eq!(t.get(b"memory_limit"), Some(&b"-1"[..]));
        assert_eq!(t.get(b"session.unknown"), None);
        assert_eq!(t.get_long(b"session.sid_length"), 32);
        assert_eq!(t.get_long(b"session.gc_divisor"), 1000);
        assert!(t.get_bool(b"session.use_cookies"));
        assert!(!t.get_bool(b"session.use_strict_mode"));
        // ini_get_all('session') lists exactly 31 directives: the trans_sid
        // pair is registered but exempt from the session listing.
        let listed = t
            .0
            .keys()
            .filter(|k| k.starts_with(b"session.") && !trans_sid_exempt(k))
            .count();
        assert_eq!(listed, 31);
    }

    #[test]
    fn quantity_parser_matches_zend() {
        assert!(quantity_is_clean(b"42"));
        assert!(quantity_is_clean(b"-1"));
        assert!(quantity_is_clean(b"1K"));
        assert!(quantity_is_clean(b""));
        assert!(!quantity_is_clean(b"1.5"));
        assert!(!quantity_is_clean(b"abc"));
        assert_eq!(leading_long(b"1.5"), 1);
        assert_eq!(leading_long(b"abc"), 0);
        assert_eq!(leading_long(b" 48 "), 48);
        assert_eq!(leading_long(b"-7x"), -7);
    }

    #[test]
    fn session_name_validation_matches_oracle() {
        assert!(session_name_ok(b"PHPSESSID"));
        assert!(session_name_ok(b"OK_name"));
        assert!(!session_name_ok(b""));
        assert!(!session_name_ok(b"123"));
        assert!(!session_name_ok(b"a=b"));
        assert!(!session_name_ok(b"a b"));
        assert!(!session_name_ok(b"a\0b"));
        // Not fully numeric → allowed (PHP refuses only numeric strings).
        assert!(session_name_ok(b"12a"));
    }

    #[test]
    fn ini_bool_matches_zend() {
        for v in [&b"1"[..], b"true", b"On", b"YES", b"2"] {
            assert!(ini_bool(v), "{}", String::from_utf8_lossy(v));
        }
        for v in [&b""[..], b"0", b"off", b"false", b"no"] {
            assert!(!ini_bool(v), "{}", String::from_utf8_lossy(v));
        }
    }
}
