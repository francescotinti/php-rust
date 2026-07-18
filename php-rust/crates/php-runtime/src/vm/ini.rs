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
    /// Directive whose startup value is NULL (`upload_tmp_dir`): `ini_get`
    /// still reads "", but `ini_get_all` reports NULL for both values.
    null_value: bool,
}

impl IniEntry {
    /// Whether a runtime set (ini_set / session_start options) may change it.
    pub(super) fn access_user_settable(&self) -> bool {
        self.settable && self.access & INI_USER != 0
    }
}

/// The table, ordered by directive name (`ini_get_all` lists alphabetically).
pub(super) struct IniTable(pub BTreeMap<Vec<u8>, IniEntry>);

impl IniTable {
    pub(super) fn new() -> Self {
        // Request (module-init) reset: the engine default timezone lives in a
        // thread_local (php_types::tz) that outlives a Vm when several run on
        // one thread (unit tests); re-anchor it to the table default.
        let _ = php_types::tz::set_default_timezone("UTC");
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
                    null_value: false,
                },
            );
        };
        // The engine-hardwired directives the retired php-builtins ini_get
        // reported, same values (real code branches on them; Composer needs
        // allow_url_fopen "1", see the old env.rs notes).
        add("allow_url_fopen", "1", INI_SYSTEM, false, false);
        add("allow_url_include", "", INI_SYSTEM, false, false);
        add("disable_functions", "", INI_SYSTEM, false, false);
        add("enable_dl", "", INI_SYSTEM, false, false);
        // The brew oracle's php.ini pins memory_limit=128M under every SAPI;
        // WP site-health's debug tab reports it verbatim (WP-11). Settable:
        // wp_raise_memory_limit() ini_sets 256M in admin and site-health then
        // reports BOTH values (memory_limit + admin_memory_limit) — phpr never
        // enforces the limit either way, so accepting the write is the
        // faithful observable state.
        add("memory_limit", "128M", INI_ALL, true, false);
        // Zend's CLI SAPI hardwires 0 / -1; `phpr -S` swaps in the php.ini
        // web values (30 / 60) at request init, see the web_request block in
        // vm/mod.rs. Both report-only: phpr has no execution/input clock.
        add("max_execution_time", "0", INI_ALL, false, false);
        add("max_input_time", "-1", INI_PERDIR | INI_SYSTEM, false, false);
        // Superglobal-parsing cap, same value under every SAPI (site-health).
        add("max_input_vars", "1000", INI_PERDIR | INI_SYSTEM, false, false);
        add("default_socket_timeout", "60", INI_ALL, false, false);
        add("precision", "14", INI_ALL, false, false);
        add("serialize_precision", "-1", INI_ALL, false, false);
        // phpr resolves includes against the working directory only; ".:" is
        // the value its include-failure messages always embedded. Settable
        // because PHPUnit's process-isolation runner round-trips it
        // (get_include_path → child set_include_path) and the failure message
        // reflects it; entries do not extend the resolver (documented
        // divergence).
        add("include_path", ".:", INI_ALL, true, false);
        // Destination file of error_log()/engine diagnostics ("" = the SAPI
        // log, stderr under CLI). Settable: Symfony's HttpKernel Logger
        // round-trips it (ini_set to a temp file, log, restore).
        add("error_log", "", INI_ALL, true, false);
        // Content-type default for the Content-Type header ("text/html" under
        // every SAPI; `php -n -i` confirms). WP's template-enhancement output
        // buffer falls back to it when headers_list() shows no Content-Type
        // (always the case under CLI) to decide HTML-ness (WP-17).
        add("default_mimetype", "text/html", INI_ALL, true, false);
        add("default_charset", "UTF-8", INI_ALL, true, false);
        // Wrapped around every displayed diagnostic by the default render
        // (main.c php_error_cb); both empty under `php -n`. Settable: WP's
        // template tests round-trip them (error_prepend_string data sets).
        add("error_prepend_string", "", INI_ALL, true, false);
        add("error_append_string", "", INI_ALL, true, false);
        // Upload limits (php -n defaults). phpr's CLI never receives uploads,
        // but UploadedFile::getMaxFilesize() computes min(post_max_size,
        // upload_max_filesize) from these.
        add("upload_max_filesize", "2M", INI_PERDIR | INI_SYSTEM, false, false);
        add("post_max_size", "8M", INI_PERDIR | INI_SYSTEM, false, false);
        // WP site-health's debug tab reads both (WP-10).
        add("file_uploads", "1", INI_SYSTEM, false, false);
        add("max_file_uploads", "20", INI_PERDIR | INI_SYSTEM, false, false);
        // The default timezone (D-DT3). The CLI oracle reports "UTC" under
        // `-n`; writes propagate to php_types::tz so the date builtins and
        // date_default_timezone_get() see them.
        add("date.timezone", "UTC", INI_ALL, true, false);
        // Diagnostics-display directives (CLI defaults; the web SAPI swaps in
        // its own at request init — html_errors=1, output_buffering=4096).
        // html_errors/display_errors/log_errors are honoured by the render
        // chokepoints; output_buffering/implicit_flush are report-only.
        add("display_errors", "1", INI_ALL, true, false);
        // EG(error_reporting) as an INI directive: ini_set/ini_get see the
        // same mask as the error_reporting() builtin (ho_ini_set and
        // ho_error_reporting mirror writes into `error_level`). WP's template
        // tests drive it exclusively through ini_set (WP-17).
        add("error_reporting", "30719", INI_ALL, true, true);
        add("log_errors", "1", INI_ALL, true, false);
        add("html_errors", "0", INI_ALL, true, false);
        add("output_buffering", "0", INI_PERDIR | INI_SYSTEM, false, false);
        add("implicit_flush", "1", INI_ALL, false, false);
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
        drop(add);
        // upload_tmp_dir e open_basedir partono NULL (ini_get → "",
        // ini_get_all → NULL, WP-16 probe). open_basedir è settable —
        // WP_Automatic_Updater::is_allowed_dir legge ini_get e fa i suoi
        // check; phpr NON applica la restrizione alle operazioni su file
        // (divergenza documentata).
        t.insert(
            b"upload_tmp_dir".to_vec(),
            IniEntry {
                global: Vec::new(),
                local: Vec::new(),
                access: INI_SYSTEM,
                settable: false,
                int_typed: false,
                deprecated_off_default: false,
                null_value: true,
            },
        );
        t.insert(
            b"open_basedir".to_vec(),
            IniEntry {
                global: Vec::new(),
                local: Vec::new(),
                access: INI_ALL,
                settable: true,
                int_typed: false,
                deprecated_off_default: false,
                null_value: true,
            },
        );
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
    pub(super) fn get_long(&self, name: &[u8]) -> i64 {
        self.get(name).map_or(0, |v| leading_long(v))
    }

    /// A directive read as PHP's INI boolean (On/True/Yes/1 → true).
    pub(super) fn get_bool(&self, name: &[u8]) -> bool {
        self.get(name).is_some_and(ini_bool)
    }
}

/// Bool-typed (OnUpdateBool) directives store their value NORMALIZED to
/// "1"/"0" — `session_set_cookie_params_variation4`: setting "TRUE" reads
/// back as "1". Other directives store the raw string.
fn normalize_bool_ini(name: &[u8], value: Vec<u8>) -> Vec<u8> {
    let bool_typed = matches!(
        name,
        b"session.use_cookies"
            | b"session.use_only_cookies"
            | b"session.use_strict_mode"
            | b"session.use_trans_sid"
            | b"session.cookie_secure"
            | b"session.cookie_httponly"
            | b"session.cookie_partitioned"
            | b"session.lazy_write"
            | b"session.auto_start"
            | b"session.upload_progress.enabled"
            | b"session.upload_progress.cleanup"
    );
    if bool_typed {
        (if ini_bool(&value) { b"1" } else { b"0" }) .to_vec()
    } else {
        value
    }
}

/// PHP's `zend_ini_parse_bool`: "1"/"true"/"yes"/"on" (case-insensitive) are
/// true, any nonzero leading number too; everything else is false.
pub(super) fn ini_bool(v: &[u8]) -> bool {
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

/// The GET/POST-session deprecations (8.4): setting one of these directives to
/// its deprecated STATE warns — falsy `use_only_cookies`, truthy
/// `use_trans_sid`, any non-default `trans_sid_tags`/`trans_sid_hosts`/
/// `referer_check` (deprecations.phpt: value-based, restoring the default is
/// silent). Shared by ini_set, session_start options and the startup path.
pub(super) fn session_state_deprecation(name: &[u8], value: &[u8]) -> Option<String> {
    let deprecated = match name {
        b"session.use_only_cookies" if !ini_bool(value) => {
            return Some("Disabling session.use_only_cookies INI setting is deprecated".into())
        }
        b"session.use_trans_sid" if ini_bool(value) => {
            return Some("Enabling session.use_trans_sid INI setting is deprecated".into())
        }
        b"session.trans_sid_tags" => value != b"a=href,area=href,frame=src,form=",
        b"session.trans_sid_hosts" | b"session.referer_check" => !value.is_empty(),
        _ => false,
    };
    deprecated.then(|| {
        format!("Usage of {} INI setting is deprecated", String::from_utf8_lossy(name))
    })
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

    /// `get_cfg_var(string $option): string|array|false` — the *startup* value
    /// of a registered directive (`e.global`, which `php -d`/`--INI--` set),
    /// `false` for an unknown key. `cfg_file_path` stays unknown → `false`:
    /// phpr loads no php.ini, like `php -n`.
    pub(super) fn ho_get_cfg_var(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let name = self.ini_string_arg(&args, 0, b"get_cfg_var", b"$option")?;
        Ok(match self.ini.0.get(&name) {
            Some(e) => Zval::Str(PhpStr::new(e.global.clone())),
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
        // Capture the entry's flags up front: the validation below needs
        // `&mut self` for its diagnostics.
        let Some((user_settable, deprecated_off_default, int_typed, global)) = self
            .ini
            .0
            .get(&name)
            .map(|e| (e.access_user_settable(), e.deprecated_off_default, e.int_typed, e.global.clone()))
        else {
            return Ok(Zval::Bool(false));
        };
        if !user_settable {
            return Ok(Zval::Bool(false));
        }
        if !self.ini_session_value_ok("ini_set()", &name, &value) {
            return Ok(Zval::Bool(false));
        }
        if deprecated_off_default && value != global {
            self.diags.push(Diag::Deprecated(format!(
                "ini_set(): {} INI setting is deprecated",
                String::from_utf8_lossy(&name)
            )));
        }
        if let Some(msg) = session_state_deprecation(&name, &value) {
            self.diags.push(Diag::Deprecated(format!("ini_set(): {msg}")));
        }
        if int_typed && !quantity_is_clean(&value) {
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
        let value = normalize_bool_ini(&name, value);
        let entry = self.ini.0.get_mut(&name).expect("checked above");
        if name == b"date.timezone" {
            let _ = php_types::tz::set_default_timezone(
                &String::from_utf8_lossy(&value).into_owned(),
            );
        }
        // Oddity oracle-pinned (WP-16 probe): un ini_set di open_basedir
        // aggiorna ANCHE global_value (la restrizione non è ripristinabile,
        // OnUpdateBaseDir non conserva l'orig).
        if name == b"open_basedir" {
            entry.global = value.clone();
        }
        let old = std::mem::replace(&mut entry.local, value);
        // `error_reporting` is EG(error_reporting): the write must land in the
        // live mask consulted by raise_diagnostic/flush_diags too.
        if name == b"error_reporting" {
            self.error_level = leading_long(trim_ascii(&entry.local));
        }
        Ok(Zval::Str(PhpStr::new(old)))
    }

    /// `get_include_path(): string|false` — the include_path directive.
    pub(super) fn ho_get_include_path(&mut self) -> Result<Zval, PhpError> {
        Ok(match self.ini.get(b"include_path") {
            Some(v) => Zval::Str(PhpStr::new(v.to_vec())),
            None => Zval::Bool(false),
        })
    }

    /// `set_include_path(string $include_path): string|false` — returns the
    /// previous value. The resolver itself stays cwd-based (see the table).
    pub(super) fn ho_set_include_path(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let new = match args.first() {
            Some(v) => convert::to_zstr(&v.deref_clone(), &mut self.diags).as_bytes().to_vec(),
            None => {
                return Err(PhpError::ArgumentCountError(
                    "set_include_path() expects exactly 1 argument, 0 given".to_string(),
                ))
            }
        };
        let old = self.ini.get(b"include_path").unwrap_or(b".").to_vec();
        self.ini_set_local(b"include_path", new);
        Ok(Zval::Str(PhpStr::new(old)))
    }

    /// `restore_include_path(): void`.
    pub(super) fn ho_restore_include_path(&mut self) -> Result<Zval, PhpError> {
        if let Some(e) = self.ini.0.get_mut(&b"include_path"[..]) {
            e.local = e.global.clone();
        }
        Ok(Zval::Null)
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
            let val = |bytes: &Vec<u8>| {
                if e.null_value && bytes.is_empty() {
                    Zval::Null
                } else {
                    Zval::Str(PhpStr::new(bytes.clone()))
                }
            };
            if details {
                let mut row = PhpArray::new();
                row.insert(Key::Str(PhpStr::from_str("global_value")), val(&e.global));
                row.insert(Key::Str(PhpStr::from_str("local_value")), val(&e.local));
                row.insert(Key::Str(PhpStr::from_str("access")), Zval::Long(e.access));
                out.insert(key, Zval::Array(Rc::new(row)));
            } else {
                out.insert(key, val(&e.local));
            }
        }
        Ok(Zval::Array(Rc::new(out)))
    }

    /// Per-directive value validation shared by `ini_set`, the session
    /// module's setters/start-options and the startup override path. `prefix`
    /// is the attribution — `"ini_set()"` / `"session_name()"` /
    /// `"PHP Startup"` (oracle: `Warning: PHP Startup: session.name "" must
    /// not be numeric…`). A refused value leaves the entry untouched. When
    /// `startup` the warning renders raw ("in Unknown on line 0") instead of
    /// through the diag queue.
    pub(super) fn ini_session_value_ok(&mut self, prefix: &str, name: &[u8], value: &[u8]) -> bool {
        let startup = prefix == "PHP Startup";
        let msg = if name == b"session.name" && !session_name_ok(value) {
            Some(format!(
                "{prefix}: session.name \"{}\" must not be numeric, empty, contain null bytes \
                 or any of the following characters \"=,;.[ \\t\\r\\n\\013\\014\"",
                String::from_utf8_lossy(value)
            ))
        } else if !startup && name == b"session.save_handler" && value != b"files" {
            // At startup (php.ini / -d) an unknown handler is STORED raw and
            // only refused at session_start ("session startup failed") —
            // session_status() reports DISABLED meanwhile.
            Some(if value == b"user" {
                format!("{prefix}: Session save handler \"user\" cannot be set by ini_set()")
            } else {
                format!(
                    "{prefix}: Session save handler \"{}\" cannot be found",
                    String::from_utf8_lossy(value)
                )
            })
        } else if !startup
            && name == b"session.serialize_handler"
            && !matches!(value, b"php" | b"php_binary" | b"php_serialize")
        {
            Some(format!(
                "{prefix}: Serialization handler \"{}\" cannot be found",
                String::from_utf8_lossy(value)
            ))
        } else if name == b"session.gc_probability" && leading_long(value) < 0 {
            Some(format!("{prefix}: session.gc_probability must be greater than or equal to 0"))
        } else if name == b"session.gc_divisor" && leading_long(value) < 1 {
            Some(format!("{prefix}: session.gc_divisor must be greater than 0"))
        } else {
            None
        };
        let Some(msg) = msg else { return true };
        if startup {
            self.render_startup_diag("Warning", &msg);
        } else {
            self.diags.push(Diag::Warning(msg));
        }
        false
    }

    /// Render a pre-main (startup) or shutdown diagnostic the way PHP does:
    /// `Sev: PHP Startup: msg in Unknown on line 0`, straight onto the output
    /// (no frame exists to attribute it to).
    pub(super) fn render_startup_diag(&mut self, severity: &str, msg: &str) {
        let block = format!("\n{severity}: {msg} in Unknown on line 0\n");
        self.rendered.extend_from_slice(block.as_bytes());
        self.stdout.extend_from_slice(block.as_bytes());
    }

    /// Apply `php -d`-style startup overrides (phpt `--INI--`): validated
    /// values land as BOTH startup default and current value; ext/session's
    /// module-startup deprecations then fire in session.c's fixed order.
    pub(super) fn apply_ini_overrides(&mut self, overrides: &[(Vec<u8>, Vec<u8>)]) {
        for (k, v) in overrides {
            if !self.ini_session_value_ok("PHP Startup", k, v) {
                continue;
            }
            // session.upload_progress.freq bounds (rfc1867 startup checks).
            if k == b"session.upload_progress.freq" {
                let n = leading_long(v);
                if n < 0 {
                    self.render_startup_diag(
                        "Warning",
                        "PHP Startup: session.upload_progress.freq must be greater than or equal to 0",
                    );
                    continue;
                }
                if v.ends_with(b"%") && n > 100 {
                    self.render_startup_diag(
                        "Warning",
                        "PHP Startup: session.upload_progress.freq must be less than or equal to 100%",
                    );
                    continue;
                }
            }
            if let Some(e) = self.ini.0.get_mut(k) {
                let v = normalize_bool_ini(k, v.clone());
                e.global = v.clone();
                e.local = v;
                if k == b"date.timezone" {
                    // Invalid zones stay in the table but leave the engine
                    // default (UTC) untouched, like timelib's lazy fallback.
                    let _ = php_types::tz::set_default_timezone(
                        &String::from_utf8_lossy(&e.local).into_owned(),
                    );
                }
            }
        }
        let mut dep = Vec::new();
        if !self.ini.get_bool(b"session.use_only_cookies") {
            dep.push("PHP Startup: Disabling session.use_only_cookies INI setting is deprecated");
        }
        if self.ini.get_bool(b"session.use_trans_sid") {
            dep.push("PHP Startup: Enabling session.use_trans_sid INI setting is deprecated");
        }
        if self.ini.get(b"session.sid_length") != Some(b"32") {
            dep.push("PHP Startup: session.sid_length INI setting is deprecated");
        }
        if self.ini.get(b"session.sid_bits_per_character") != Some(b"4") {
            dep.push("PHP Startup: session.sid_bits_per_character INI setting is deprecated");
        }
        for msg in dep {
            self.render_startup_diag("Deprecated", msg);
        }
    }

    /// Set a directive's local value directly — the session module's setters
    /// (`session_name`, start options, cookie params) already ran their own
    /// guards and validation, so no freeze/access checks re-apply here.
    pub(super) fn ini_set_local(&mut self, name: &[u8], value: Vec<u8>) {
        let value = normalize_bool_ini(name, value);
        if let Some(e) = self.ini.0.get_mut(name) {
            e.local = value;
        }
    }

    /// The `session.*` change guard shared by `ini_set`/`ini_restore` (and by
    /// the session module's own setters): once output has started, session ini
    /// settings are frozen. Returns `false` (after warning) when refused.
    fn ini_session_change_allowed(&mut self, name: &[u8], func: &[u8]) -> bool {
        if !name.starts_with(b"session.") || trans_sid_exempt(name) {
            return true;
        }
        if self.session.active {
            let suffix = self.sess_started_from();
            self.diags.push(Diag::Warning(format!(
                "{}(): Session ini settings cannot be changed when a session is active{suffix}",
                String::from_utf8_lossy(func)
            )));
            return false;
        }
        if self.output_started && self.ini.get_bool(b"session.use_cookies") {
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
        assert_eq!(t.get(b"memory_limit"), Some(&b"128M"[..]));
        assert_eq!(t.get(b"max_input_vars"), Some(&b"1000"[..]));
        assert_eq!(t.get(b"max_input_time"), Some(&b"-1"[..]));
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
