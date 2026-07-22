//! ext/session core (C1 of the port): the CLI `files` save handler, the
//! session lifecycle (`session_start` → `$_SESSION` → commit), the `php` /
//! `php_binary` / `php_serialize` serializers and the id generator. The user
//! save-handler plumbing (`session_set_save_handler`, `SessionHandler`) is C2.
//!
//! Every contract below is oracle-pinned (PHP 8.5.7 probes, 2026-07-11/12):
//! - Files land in `session.save_path` (or the system temp dir when empty) as
//!   `sess_<id>`, mode 0600; the `files` module CREATES the file at read.
//! - `php` format is `key|<serialized>` concatenated; a numeric key warns
//!   "Skipping numeric key N" and is dropped; a key containing `|` fails the
//!   whole encode ("Failed to write session data. Data contains invalid key").
//! - Undecodable data at start/`session_decode` destroys the session (file
//!   deleted, id cleared, `$_SESSION` emptied): "Failed to decode session
//!   object. Session has been destroyed".
//! - `session_destroy` deletes the file and clears the id but does NOT touch
//!   `$_SESSION`; `session_unset` empties it; `session_reset` re-reads.
//! - `session_regenerate_id(false)` first WRITES the current data under the
//!   old id (both files end up with the current payload), `true` deletes the
//!   old file; either way the session continues under a fresh id.
//! - lazy_write: an unchanged payload at commit touches the file mtime
//!   (updateTimestamp) instead of rewriting.
//! - GC runs at start with probability gc_probability/gc_divisor and collects
//!   `sess_*` files whose mtime is older than gc_maxlifetime; `session_gc()`
//!   returns the collected count.
//! - Headers-sent and session-active guards carry the "(sent from F on line
//!   N)" / "(started from F on line N)" suffixes; the shutdown auto-flush runs
//!   AFTER shutdown functions and destructors (both still see an ACTIVE
//!   session and their writes are persisted).

use std::io::Read as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;

/// The session module's runtime state (one session per request, like PHP).
#[derive(Default)]
pub(super) struct SessionState {
    pub active: bool,
    /// True only while `sess_commit` runs the save handler: the session
    /// already counts as closed (bug60634), but the prelude `\SessionHandler`
    /// delegate must still operate — PHP's guard is "no save handler open",
    /// not `status == active` (Symfony's SessionHandlerProxy write path).
    pub committing: bool,
    /// Current session id ("" when none); survives a commit, cleared by
    /// destroy and by a failed decode.
    pub id: Vec<u8>,
    /// Where the active session was started, for the "(started from F on line
    /// N)" suffix of the active-session guards.
    pub started_at: Option<(Vec<u8>, u32)>,
    /// The raw payload read at start: commit compares against the re-encoded
    /// data to pick write vs updateTimestamp (lazy_write). `None` forces a
    /// write (fresh id after regenerate).
    pub snapshot: Option<Vec<u8>>,
    /// The save path captured when the session opened (an ini change while
    /// active must not redirect the close-time write; a SessionHandler
    /// subclass may also pass a custom path to `parent::open`).
    pub open_path: Vec<u8>,
    /// The save handler in use: the built-in files module, or the user
    /// handler registered by `session_set_save_handler` (C2).
    pub handler: SaveHandler,
    /// Whether the prelude `SessionHandler`'s parent (files) module has been
    /// opened via `__session_files_op('open')` — its other ops warn "Parent
    /// session handler is not open" until then (class_005).
    pub files_open: bool,
}

/// The registered save handler.
#[derive(Default)]
pub(super) enum SaveHandler {
    #[default]
    Files,
    User(Box<UserHandler>),
}

/// The user save-handler callables (`session_set_save_handler`): either the
/// object form's `[obj, method]` pairs or the deprecated positional closures.
pub(super) struct UserHandler {
    open: Zval,
    close: Zval,
    read: Zval,
    write: Zval,
    destroy: Zval,
    gc: Zval,
    /// `SessionIdInterface::create_sid` when implemented/passed.
    create_sid: Option<Zval>,
    /// `SessionUpdateTimestampHandlerInterface` when implemented/passed.
    validate_id: Option<Zval>,
    update_timestamp: Option<Zval>,
    /// The handler's class (object form) for the write-failure warning.
    class_name: Option<Vec<u8>>,
}

/// The three serialize handlers `session.serialize_handler` can select.
#[derive(Clone, Copy, PartialEq)]
enum SerHandler {
    Php,
    PhpBinary,
    PhpSerialize,
}

/// `bin_to_readable` (ext/session/session.c): pack random bytes into the
/// session-id alphabet, `nbits` (4/5/6) per output character.
fn bin_to_readable(bytes: &[u8], out_len: usize, nbits: u32) -> Vec<u8> {
    const CHARS: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-,";
    let mask = (1u32 << nbits) - 1;
    let (mut w, mut have, mut p) = (0u32, 0u32, 0usize);
    let mut out = Vec::with_capacity(out_len);
    for _ in 0..out_len {
        if have < nbits {
            if p < bytes.len() {
                w |= (bytes[p] as u32) << have;
                p += 1;
                have += 8;
            } else {
                // Out of entropy (never happens: the caller sizes the buffer).
                break;
            }
        }
        out.push(CHARS[(w & mask) as usize]);
        w >>= nbits;
        have -= nbits;
    }
    out
}

/// Whether `id`/`prefix` sticks to the session-id alphabet `[-,a-zA-Z0-9]`.
fn sid_chars_ok(s: &[u8]) -> bool {
    s.iter().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b','))
}

impl<'m> Vm<'m> {
    // ---- configuration reads -------------------------------------------------

    fn sess_file(&self, dir: &std::path::Path, id: &[u8]) -> PathBuf {
        dir.join(format!("sess_{}", String::from_utf8_lossy(id)))
    }

    fn sess_ser_handler(&self) -> SerHandler {
        match self.ini.get(b"session.serialize_handler") {
            Some(b"php_binary") => SerHandler::PhpBinary,
            Some(b"php_serialize") => SerHandler::PhpSerialize,
            _ => SerHandler::Php,
        }
    }

    // ---- guards ---------------------------------------------------------------

    /// The "(sent from F on line N)" suffix once output has started.
    fn sess_sent_from(&self) -> String {
        match &self.output_start {
            Some((f, l)) => format!(" (sent from {} on line {})", String::from_utf8_lossy(f), l),
            None => String::new(),
        }
    }

    /// The "(started from F on line N)" suffix of the active-session guards.
    pub(super) fn sess_started_from(&self) -> String {
        match &self.session.started_at {
            Some((f, l)) => {
                format!(" (started from {} on line {})", String::from_utf8_lossy(f), l)
            }
            None => String::new(),
        }
    }

    /// Refuse a setter while a session is active: `func(): <what> cannot be
    /// changed when a session is active (started from F on line N)`.
    fn sess_refuse_active(&mut self, func: &str, what: &str) -> bool {
        if !self.session.active {
            return false;
        }
        let suffix = self.sess_started_from();
        self.diags.push(Diag::Warning(format!(
            "{func}(): {what} cannot be changed when a session is active{suffix}"
        )));
        true
    }

    /// Refuse a setter once output has started: `func(): <what> cannot be
    /// changed after headers have already been sent (sent from F on line N)`.
    /// Like session.c, the check only applies while the module would send a
    /// cookie: `session.use_cookies=0` disarms it (bug74941).
    fn sess_refuse_sent(&mut self, func: &str, what: &str) -> bool {
        if !self.output_started || !self.ini.get_bool(b"session.use_cookies") {
            return false;
        }
        let suffix = self.sess_sent_from();
        self.diags.push(Diag::Warning(format!(
            "{func}(): {what} cannot be changed after headers have already been sent{suffix}"
        )));
        true
    }

    // ---- $_SESSION binding ----------------------------------------------------

    /// The `$_SESSION` superglobal as an array; `None` when the user replaced
    /// it with a non-array (or unset it) — the encode then FAILS silently
    /// (session_encode → false, encode_variation2) and a commit leaves the
    /// created-empty file as-is (oracle probe: scalar $_SESSION → "" file).
    fn sess_read_superglobal(&self) -> Option<PhpArray> {
        let idx = crate::bytecode::superglobal_index(b"_SESSION")
            .expect("_SESSION is a superglobal") as usize;
        match read_slot(&self.superglobals[idx]) {
            Zval::Array(a) => Some((*a).clone()),
            _ => None,
        }
    }

    fn sess_write_superglobal(&mut self, arr: PhpArray) {
        let idx = crate::bytecode::superglobal_index(b"_SESSION")
            .expect("_SESSION is a superglobal") as usize;
        let old = store_slot(&mut self.superglobals[idx], Zval::Array(Rc::new(arr)));
        self.gc_note(&old);
    }

    // ---- files module ---------------------------------------------------------

    /// mod_files' save-path grammar `[depth;[mode;]]basedir` parsed against
    /// the path in effect (open-time capture). `None` = malformed (too many
    /// components / non-numeric depth or mode), the "Failed to create session
    /// data file path" failure.
    fn sess_files_conf(&self) -> Option<(u32, PathBuf)> {
        let raw = self.session_raw_path();
        if raw.is_empty() {
            return Some((0o600, std::env::temp_dir()));
        }
        let parts: Vec<&[u8]> = raw.split(|b| *b == b';').collect();
        let int_ok = |s: &[u8]| !s.is_empty() && s.iter().all(|b| b.is_ascii_digit());
        let (mode, base): (u32, &[u8]) = match parts.len() {
            1 => (0o600, parts[0]),
            2 if int_ok(parts[0]) => (0o600, parts[1]),
            3 if int_ok(parts[0]) && !parts[1].is_empty() => {
                let mode = u32::from_str_radix(&String::from_utf8_lossy(parts[1]), 8).ok()?;
                (mode, parts[2])
            }
            _ => return None,
        };
        if base.is_empty() {
            return None;
        }
        Some((mode, PathBuf::from(String::from_utf8_lossy(base).into_owned())))
    }

    /// The raw `session.save_path` in effect: the open-time capture while a
    /// session is open, the ini otherwise (also what the "(path: %s)"
    /// warnings display).
    fn session_raw_path(&self) -> Vec<u8> {
        if !self.session.open_path.is_empty() {
            return self.session.open_path.clone();
        }
        self.ini.get(b"session.save_path").unwrap_or(b"").to_vec()
    }

    /// mod_files' key check: 1..=128 chars of the sid alphabet.
    fn sess_files_id_ok(id: &[u8]) -> bool {
        !id.is_empty() && id.len() <= 128 && sid_chars_ok(id)
    }

    /// Read (creating, like mod_files' O_CREAT) the session file. `Err(())` is
    /// a real read failure, already warned (`func` names the caller).
    fn sess_files_read(&mut self, func: &str, id: &[u8]) -> Result<Vec<u8>, ()> {
        use std::os::unix::fs::OpenOptionsExt;
        let Some((mode, dir)) = self.sess_files_conf().filter(|_| Self::sess_files_id_ok(id))
        else {
            self.diags.push(Diag::Warning(format!(
                "{func}(): Failed to create session data file path. Too short session ID, \
                 invalid save_path or path length exceeds 4096 characters"
            )));
            return Err(());
        };
        let path = self.sess_file(&dir, id);
        let mut f = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(mode)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => {
                // mod_files reports the raw open(2) failure.
                let errno = e.raw_os_error().unwrap_or(0);
                let msg = e.to_string();
                let msg = msg.split(" (os error").next().unwrap_or(&msg);
                self.diags.push(Diag::Warning(format!(
                    "{func}(): open({}, O_RDWR) failed: {msg} ({errno})",
                    path.display()
                )));
                return Err(());
            }
        };
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).map_err(|_| ())?;
        Ok(buf)
    }

    fn sess_files_write(&mut self, id: &[u8], data: &[u8]) {
        use std::os::unix::fs::OpenOptionsExt;
        let Some((mode, dir)) = self.sess_files_conf() else { return };
        let path = self.sess_file(&dir, id);
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(&path)
        {
            let _ = f.write_all(data);
        }
    }

    /// The directory for close-time operations: the one captured at open.
    fn sess_dir_open(&self) -> PathBuf {
        self.sess_files_conf().map_or_else(std::env::temp_dir, |(_, d)| d)
    }

    fn sess_files_destroy(&mut self, id: &[u8]) {
        let dir = self.sess_dir_open();
        let _ = std::fs::remove_file(self.sess_file(&dir, id));
    }

    fn sess_files_touch(&mut self, id: &[u8]) {
        let dir = self.sess_dir_open();
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(self.sess_file(&dir, id)) {
            let _ = f.set_modified(SystemTime::now());
        }
    }

    /// Collect stale `sess_*` files (mtime older than gc_maxlifetime).
    fn sess_files_gc(&mut self) -> i64 {
        let maxlife = self.ini.get_long(b"session.gc_maxlifetime").max(0) as u64;
        let cutoff = SystemTime::now() - Duration::from_secs(maxlife);
        let dir = self.sess_dir_open();
        let Ok(entries) = std::fs::read_dir(&dir) else { return 0 };
        let mut n = 0;
        for e in entries.flatten() {
            if !e.file_name().as_encoded_bytes().starts_with(b"sess_") {
                continue;
            }
            let stale = e
                .metadata()
                .and_then(|m| m.modified())
                .is_ok_and(|m| m < cutoff);
            if stale && std::fs::remove_file(e.path()).is_ok() {
                n += 1;
            }
        }
        n
    }

    // ---- save-handler dispatch --------------------------------------------------

    /// The user handler's callable for `which`, if one is registered.
    fn sess_user_cb(&self, which: fn(&UserHandler) -> &Zval) -> Option<Zval> {
        match &self.session.handler {
            SaveHandler::Files => None,
            SaveHandler::User(h) => Some(which(h).clone()),
        }
    }

    fn sess_user_cb_opt(&self, which: fn(&UserHandler) -> &Option<Zval>) -> Option<Zval> {
        match &self.session.handler {
            SaveHandler::Files => None,
            SaveHandler::User(h) => which(h).clone(),
        }
    }

    /// The module name for the start-failure warnings ("files" / "user").
    fn sess_mod_name(&self) -> String {
        String::from_utf8_lossy(self.ini.get(b"session.save_handler").unwrap_or(b"files"))
            .into_owned()
    }

    /// A user callback declared `: bool` semantics: anything else warns
    /// "Session callback must have a return value of type bool, X returned"
    /// and counts as failure (session_set_save_handler_type_error*).
    fn sess_bool_ret(&mut self, r: Zval) -> bool {
        match r {
            Zval::Bool(b) => b,
            other => {
                self.diags.push(Diag::Warning(format!(
                    "Session callback must have a return value of type bool, {} returned",
                    other.type_name_for_error()
                )));
                false
            }
        }
    }

    /// open(save_path, session_name) → whether the module initialized.
    fn sess_mod_open(&mut self) -> Result<bool, PhpError> {
        let path = self.session.open_path.clone();
        if let Some(cb) = self.sess_user_cb(|h| &h.open) {
            let name = self.ini.get(b"session.name").unwrap_or(b"PHPSESSID").to_vec();
            let r = self.call_callable(
                cb,
                vec![
                    Zval::Str(PhpStr::new(path)),
                    Zval::Str(PhpStr::new(name)),
                ],
            )?;
            return Ok(self.sess_bool_ret(r));
        }
        // files: open never fails — a bogus save_path surfaces at READ time
        // ("Failed to create session data file path" / raw open(2) error).
        Ok(true)
    }

    /// read(id) → the payload; `None` = the handler failed the read (the
    /// files module has already warned with `func` attribution).
    fn sess_mod_read(&mut self, func: &str, id: &[u8]) -> Result<Option<Vec<u8>>, PhpError> {
        if let Some(cb) = self.sess_user_cb(|h| &h.read) {
            let r = self.call_callable(cb, vec![Zval::Str(PhpStr::new(id.to_vec()))])?;
            return Ok(match r {
                Zval::Bool(false) => None,
                Zval::Str(s) => Some(s.as_bytes().to_vec()),
                other => Some(convert::to_zstr(&other, &mut self.diags).as_bytes().to_vec()),
            });
        }
        Ok(self.sess_files_read(func, id).ok())
    }

    /// write(id, data) → whether the handler persisted the payload.
    fn sess_mod_write(&mut self, id: &[u8], data: &[u8]) -> Result<bool, PhpError> {
        if let Some(cb) = self.sess_user_cb(|h| &h.write) {
            let r = self.call_callable(
                cb,
                vec![
                    Zval::Str(PhpStr::new(id.to_vec())),
                    Zval::Str(PhpStr::new(data.to_vec())),
                ],
            )?;
            return Ok(self.sess_bool_ret(r));
        }
        self.sess_files_write(id, data);
        Ok(true)
    }

    fn sess_mod_close(&mut self) -> Result<(), PhpError> {
        if let Some(cb) = self.sess_user_cb(|h| &h.close) {
            let r = self.call_callable(cb, Vec::new())?;
            self.sess_bool_ret(r);
        }
        Ok(())
    }

    fn sess_mod_destroy(&mut self, id: &[u8]) -> Result<(), PhpError> {
        if let Some(cb) = self.sess_user_cb(|h| &h.destroy) {
            self.call_callable(cb, vec![Zval::Str(PhpStr::new(id.to_vec()))])?;
            return Ok(());
        }
        self.sess_files_destroy(id);
        Ok(())
    }

    /// gc(maxlifetime) → collected count, `None` when the handler failed.
    fn sess_mod_gc(&mut self) -> Result<Option<i64>, PhpError> {
        let maxlife = self.ini.get_long(b"session.gc_maxlifetime");
        if let Some(cb) = self.sess_user_cb(|h| &h.gc) {
            let r = self.call_callable(cb, vec![Zval::Long(maxlife)])?;
            return Ok(match r {
                Zval::Bool(false) | Zval::Null => None,
                other => Some(convert::to_long_cast(&other, &mut self.diags)),
            });
        }
        Ok(Some(self.sess_files_gc()))
    }

    /// The unchanged-payload commit path: updateTimestamp when the handler
    /// supports it (files: touch the mtime), a plain write otherwise.
    fn sess_mod_update_or_write(&mut self, id: &[u8], data: &[u8]) -> Result<bool, PhpError> {
        match &self.session.handler {
            SaveHandler::Files => {
                self.sess_files_touch(id);
                Ok(true)
            }
            SaveHandler::User(_) => {
                if let Some(cb) = self.sess_user_cb_opt(|h| &h.update_timestamp) {
                    let r = self.call_callable(
                        cb,
                        vec![
                            Zval::Str(PhpStr::new(id.to_vec())),
                            Zval::Str(PhpStr::new(data.to_vec())),
                        ],
                    )?;
                    Ok(self.sess_bool_ret(r))
                } else {
                    // No SessionUpdateTimestampHandlerInterface: always write.
                    self.sess_mod_write(id, data)
                }
            }
        }
    }

    /// create_sid via the handler when it implements SessionIdInterface.
    fn sess_mod_create_sid(&mut self) -> Result<Vec<u8>, PhpError> {
        if let Some(cb) = self.sess_user_cb_opt(|h| &h.create_sid) {
            let r = self.call_callable(cb, Vec::new())?;
            return Ok(convert::to_zstr(&r, &mut self.diags).as_bytes().to_vec());
        }
        Ok(self.sess_generate_id())
    }

    /// Strict-mode id validation: the handler's validateId when implemented
    /// (missing → every id validates, like mod_user), file existence for files.
    fn sess_mod_validate(&mut self, id: &[u8]) -> Result<bool, PhpError> {
        match &self.session.handler {
            SaveHandler::Files => {
                let dir = self.sess_dir_open();
                Ok(self.sess_file(&dir, id).exists())
            }
            SaveHandler::User(_) => match self.sess_user_cb_opt(|h| &h.validate_id) {
                Some(cb) => {
                    let r = self.call_callable(cb, vec![Zval::Str(PhpStr::new(id.to_vec()))])?;
                    Ok(convert::to_bool(&r, &mut self.diags))
                }
                None => Ok(true),
            },
        }
    }

    // ---- id generation ---------------------------------------------------------

    /// A fresh session id per `session.sid_length`/`sid_bits_per_character`.
    fn sess_generate_id(&mut self) -> Vec<u8> {
        let len = self.ini.get_long(b"session.sid_length").clamp(22, 256) as usize;
        let bits = self.ini.get_long(b"session.sid_bits_per_character").clamp(4, 6) as u32;
        let nbytes = (len * bits as usize).div_ceil(8);
        let mut buf = vec![0u8; nbytes];
        if os_random_fill(&mut buf).is_err() {
            // Entropy failure is not observable in practice; degrade to zeros.
        }
        bin_to_readable(&buf, len, bits)
    }

    // ---- serializers ------------------------------------------------------------

    /// Encode `$_SESSION` with the configured handler. `Ok(None)` = encode
    /// refused (a `|` key under `php`). `prefix` attributes the warnings:
    /// `"session_encode()"` / `"session_write_close()"` — or
    /// `"PHP Request Shutdown"`, which renders raw ("in Unknown on line 0",
    /// gh18634) because the shutdown flush is past the diag queue.
    fn sess_encode_data(&mut self, prefix: &str) -> Result<Option<Vec<u8>>, PhpError> {
        let raw = prefix == "PHP Request Shutdown";
        let warn = |vm: &mut Self, msg: String| {
            if raw {
                vm.render_startup_diag("Warning", &msg);
            } else {
                vm.diags.push(Diag::Warning(msg));
            }
        };
        let Some(arr) = self.sess_read_superglobal() else {
            return Ok(None);
        };
        match self.sess_ser_handler() {
            SerHandler::PhpSerialize => {
                let ser = self.ho_serialize(vec![Zval::Array(Rc::new(arr))])?;
                Ok(Some(match ser {
                    Zval::Str(s) => s.as_bytes().to_vec(),
                    _ => Vec::new(),
                }))
            }
            SerHandler::Php => {
                let mut out = Vec::new();
                for (k, v) in arr.iter() {
                    let key = match k {
                        Key::Int(n) => {
                            warn(self, format!("{prefix}: Skipping numeric key {n}"));
                            continue;
                        }
                        Key::Str(s) => s.as_bytes().to_vec(),
                    };
                    if key.contains(&b'|') || key.contains(&b'!') {
                        warn(
                            self,
                            format!(
                                "{prefix}: Failed to write session data. Data contains invalid key \"{}\"",
                                String::from_utf8_lossy(&key)
                            ),
                        );
                        return Ok(None);
                    }
                    let ser = self.ho_serialize(vec![v.clone()])?;
                    out.extend_from_slice(&key);
                    out.push(b'|');
                    if let Zval::Str(s) = ser {
                        out.extend_from_slice(s.as_bytes());
                    }
                }
                Ok(Some(out))
            }
            SerHandler::PhpBinary => {
                let mut out = Vec::new();
                for (k, v) in arr.iter() {
                    let key = match k {
                        Key::Int(n) => {
                            warn(self, format!("{prefix}: Skipping numeric key {n}"));
                            continue;
                        }
                        Key::Str(s) => s.as_bytes().to_vec(),
                    };
                    if key.len() > 127 {
                        // PS_BIN_MAX_KEYLEN: the binary format's length byte.
                        continue;
                    }
                    let ser = self.ho_serialize(vec![v.clone()])?;
                    out.push(key.len() as u8);
                    out.extend_from_slice(&key);
                    if let Zval::Str(s) = ser {
                        out.extend_from_slice(s.as_bytes());
                    }
                }
                Ok(Some(out))
            }
        }
    }

    /// Decode a payload into an array. `Ok(None)` = malformed (the caller
    /// applies the destroy-on-failure contract).
    fn sess_decode_data(&mut self, data: &[u8]) -> Result<Option<PhpArray>, PhpError> {
        match self.sess_ser_handler() {
            SerHandler::PhpSerialize => {
                if data.is_empty() {
                    return Ok(Some(PhpArray::new()));
                }
                match crate::unserialize::parse(data) {
                    Some(s) => match self.vm_ser_to_zval(s)? {
                        Zval::Array(a) => Ok(Some((*a).clone())),
                        // A non-array top level still "decodes"; PHP leaves
                        // $_SESSION empty.
                        _ => Ok(Some(PhpArray::new())),
                    },
                    None => Ok(None),
                }
            }
            SerHandler::Php => {
                let mut arr = PhpArray::new();
                let mut i = 0usize;
                while i < data.len() {
                    let Some(bar) = data[i..].iter().position(|b| *b == b'|') else {
                        return Ok(None);
                    };
                    let key = &data[i..i + bar];
                    i += bar + 1;
                    let Some((ser, used)) = crate::unserialize::parse_prefix(&data[i..]) else {
                        return Ok(None);
                    };
                    i += used;
                    let v = self.vm_ser_to_zval(ser)?;
                    arr.insert(Key::from_bytes(key), v);
                }
                Ok(Some(arr))
            }
            SerHandler::PhpBinary => {
                let mut arr = PhpArray::new();
                let mut i = 0usize;
                while i < data.len() {
                    let lenbyte = data[i];
                    i += 1;
                    let klen = (lenbyte & 0x7f) as usize;
                    if i + klen > data.len() {
                        return Ok(None);
                    }
                    let key = data[i..i + klen].to_vec();
                    i += klen;
                    if lenbyte & 0x80 != 0 {
                        // Undef marker: a name without a value.
                        continue;
                    }
                    let Some((ser, used)) = crate::unserialize::parse_prefix(&data[i..]) else {
                        return Ok(None);
                    };
                    i += used;
                    let v = self.vm_ser_to_zval(ser)?;
                    arr.insert(Key::from_bytes(&key), v);
                }
                Ok(Some(arr))
            }
        }
    }

    // ---- lifecycle core ---------------------------------------------------------

    /// The destroy-on-decode-failure path shared by `session_start` and
    /// `session_decode`: warn, destroy the stored session, clear id and
    /// `$_SESSION`, close the module.
    fn sess_destroy_on_decode_failure(&mut self, func: &str) -> Result<(), PhpError> {
        self.diags.push(Diag::Warning(format!(
            "{func}(): Failed to decode session object. Session has been destroyed"
        )));
        let id = self.session.id.clone();
        self.sess_mod_destroy(&id)?;
        self.sess_mod_close()?;
        self.session.active = false;
        self.session.id = Vec::new();
        self.session.started_at = None;
        self.session.snapshot = None;
        self.session.open_path = Vec::new();
        self.sess_write_superglobal(PhpArray::new());
        Ok(())
    }

    /// Commit the active session: encode, write (or updateTimestamp when the
    /// payload is unchanged under lazy_write), close.
    fn sess_commit(&mut self, func: &str) -> Result<(), PhpError> {
        let encoded = self.sess_encode_data(func)?;
        let snapshot = std::mem::take(&mut self.session.snapshot);
        let id = self.session.id.clone();
        // The session counts as closed BEFORE the handler runs: a write
        // callback that die()s must not trigger a second flush at shutdown
        // (bug60634), and the handler itself observes a closing session.
        self.session.active = false;
        self.session.started_at = None;
        self.session.committing = true;
        let write_result = match encoded {
            Some(data) => {
                let unchanged = snapshot.as_deref() == Some(data.as_slice())
                    && self.ini.get_bool(b"session.lazy_write");
                if unchanged {
                    self.sess_mod_update_or_write(&id, &data)
                } else {
                    self.sess_mod_write(&id, &data)
                }
            }
            None => Ok(true),
        };
        let ok = match write_result {
            Ok(ok) => ok,
            Err(e) => {
                self.session.committing = false;
                self.session.open_path = Vec::new();
                return Err(e);
            }
        };
        if !ok {
            // A user handler that failed the write (false / wrong type).
            let path = self.session_raw_path();
            let handler = self.sess_user_handler_display(b"write");
            let msg = format!(
                "{func}: Failed to write session data using user defined save handler. \
                 (session.save_path: {}, handler: {handler})",
                String::from_utf8_lossy(&path)
            );
            if func == "PHP Request Shutdown" {
                self.render_startup_diag("Warning", &msg);
            } else {
                self.diags.push(Diag::Warning(msg));
            }
        }
        let close_result = self.sess_mod_close();
        self.session.committing = false;
        close_result?;
        self.session.open_path = Vec::new();
        Ok(())
    }

    /// The user handler's display name for the write-failure warning
    /// (gh7787: "MySessionHandler::write").
    fn sess_user_handler_display(&self, method: &[u8]) -> String {
        match &self.session.handler {
            SaveHandler::User(h) => {
                let m = String::from_utf8_lossy(method).into_owned();
                match &h.class_name {
                    Some(c) => format!("{}::{m}", String::from_utf8_lossy(c)),
                    None => m,
                }
            }
            SaveHandler::Files => "files".to_string(),
        }
    }

    /// The automatic flush at request shutdown (after shutdown functions and
    /// destructors — both still observe an active session). Errors swallowed:
    /// shutdown is past the point of reporting.
    pub(super) fn session_shutdown_flush(&mut self) {
        if !self.session.active {
            return;
        }
        // The frame stack is empty this late in shutdown; the serializer needs
        // a caller frame (`cur_line`) — same synthetic-main trick as
        // `run_shutdown_functions` (and the zlib ob-callback underflow lesson).
        let synthetic = self.frames.is_empty();
        if synthetic {
            self.frames.push(Frame::new(&self.module.main, self.module));
        }
        let _ = self.sess_commit("PHP Request Shutdown");
        if synthetic {
            self.frames.clear();
        }
    }

    // ---- host builtins ------------------------------------------------------------

    /// `session_status(): int` — DISABLED (0) when the configured module does
    /// not resolve (php.ini `session.save_handler=non-existent`), else NONE
    /// (1) / ACTIVE (2).
    pub(super) fn ho_session_status(&mut self) -> Result<Zval, PhpError> {
        if matches!(self.session.handler, SaveHandler::Files)
            && self.ini.get(b"session.save_handler") != Some(b"files")
        {
            return Ok(Zval::Long(0));
        }
        Ok(Zval::Long(if self.session.active { 2 } else { 1 }))
    }

    /// `session_id(?string $id = null): string|false` — get/set the id. The
    /// setter does NOT validate the value (oracle: "bad id!!" is accepted).
    pub(super) fn ho_session_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => {
                Ok(Zval::Str(PhpStr::new(self.session.id.clone())))
            }
            Some(v) => {
                if self.sess_refuse_active("session_id", "Session ID")
                    || self.sess_refuse_sent("session_id", "Session ID")
                {
                    return Ok(Zval::Bool(false));
                }
                let new = convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec();
                let old = std::mem::replace(&mut self.session.id, new);
                Ok(Zval::Str(PhpStr::new(old)))
            }
        }
    }

    /// `session_name(?string $name = null): string|false`.
    pub(super) fn ho_session_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cur = self.ini.get(b"session.name").unwrap_or(b"PHPSESSID").to_vec();
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Ok(Zval::Str(PhpStr::new(cur))),
            Some(v) => {
                let new = convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec();
                if new.contains(&0) {
                    return Err(PhpError::ValueError(
                        "session_name(): Argument #1 ($name) must not contain any null bytes"
                            .to_string(),
                    ));
                }
                if self.sess_refuse_active("session_name", "Session name")
                    || self.sess_refuse_sent("session_name", "Session name")
                {
                    return Ok(Zval::Bool(false));
                }
                // A refused VALUE still returns the previous name (gh17541);
                // only the guards above return false.
                if self.ini_session_value_ok("session_name()", b"session.name", &new) {
                    self.ini_set_local(b"session.name", new);
                }
                Ok(Zval::Str(PhpStr::new(cur)))
            }
        }
    }

    /// `session_save_path(?string $path = null): string|false`.
    pub(super) fn ho_session_save_path(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cur = self.ini.get(b"session.save_path").unwrap_or(b"").to_vec();
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Ok(Zval::Str(PhpStr::new(cur))),
            Some(v) => {
                if self.sess_refuse_active("session_save_path", "Session save path")
                    || self.sess_refuse_sent("session_save_path", "Session save path")
                {
                    return Ok(Zval::Bool(false));
                }
                let new = convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec();
                self.ini_set_local(b"session.save_path", new);
                Ok(Zval::Str(PhpStr::new(cur)))
            }
        }
    }

    /// `session_module_name(?string $module = null): string|false` — only the
    /// `files` module exists ('user' is selected via session_set_save_handler).
    pub(super) fn ho_session_module_name(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cur = self.ini.get(b"session.save_handler").unwrap_or(b"files").to_vec();
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Ok(Zval::Str(PhpStr::new(cur))),
            Some(v) => {
                if self.sess_refuse_active("session_module_name", "Session save handler module")
                {
                    return Ok(Zval::Bool(false));
                }
                let new = convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec();
                if new == b"user" {
                    // Oracle-pinned (bug73100): selecting "user" directly is a
                    // ValueError, not the module-not-found warning.
                    return Err(PhpError::ValueError(
                        "session_module_name(): Argument #1 ($module) cannot be \"user\""
                            .to_string(),
                    ));
                }
                if new != b"files" {
                    self.diags.push(Diag::Warning(format!(
                        "session_module_name(): Session handler module \"{}\" cannot be found",
                        String::from_utf8_lossy(&new)
                    )));
                    return Ok(Zval::Bool(false));
                }
                self.ini_set_local(b"session.save_handler", new);
                Ok(Zval::Str(PhpStr::new(cur)))
            }
        }
    }

    /// `session_cache_limiter(?string $value = null): string|false`.
    pub(super) fn ho_session_cache_limiter(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cur = self.ini.get(b"session.cache_limiter").unwrap_or(b"nocache").to_vec();
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Ok(Zval::Str(PhpStr::new(cur))),
            Some(v) => {
                if self.sess_refuse_active("session_cache_limiter", "Session cache limiter")
                    || self.sess_refuse_sent("session_cache_limiter", "Session cache limiter")
                {
                    return Ok(Zval::Bool(false));
                }
                let new = convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec();
                self.ini_set_local(b"session.cache_limiter", new);
                Ok(Zval::Str(PhpStr::new(cur)))
            }
        }
    }

    /// `session_cache_expire(?int $value = null): int|false` — oracle oddity:
    /// the active-session refusal still returns the CURRENT value, not false.
    pub(super) fn ho_session_cache_expire(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cur = self.ini.get_long(b"session.cache_expire");
        match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Ok(Zval::Long(cur)),
            Some(v) => {
                if self.sess_refuse_active("session_cache_expire", "Session cache expiration") {
                    return Ok(Zval::Long(cur));
                }
                let new = convert::to_long_cast(&v, &mut self.diags);
                self.ini_set_local(b"session.cache_expire", new.to_string().into_bytes());
                Ok(Zval::Long(cur))
            }
        }
    }

    /// `session_get_cookie_params(): array` — the seven keys in PHP's order.
    pub(super) fn ho_session_get_cookie_params(&mut self) -> Result<Zval, PhpError> {
        let mut a = PhpArray::new();
        a.insert(Key::from_bytes(b"lifetime"), Zval::Long(self.ini.get_long(b"session.cookie_lifetime")));
        let s = |v: Option<&[u8]>| Zval::Str(PhpStr::new(v.unwrap_or(b"").to_vec()));
        a.insert(Key::from_bytes(b"path"), s(self.ini.get(b"session.cookie_path")));
        a.insert(Key::from_bytes(b"domain"), s(self.ini.get(b"session.cookie_domain")));
        a.insert(Key::from_bytes(b"secure"), Zval::Bool(self.ini.get_bool(b"session.cookie_secure")));
        a.insert(Key::from_bytes(b"partitioned"), Zval::Bool(self.ini.get_bool(b"session.cookie_partitioned")));
        a.insert(Key::from_bytes(b"httponly"), Zval::Bool(self.ini.get_bool(b"session.cookie_httponly")));
        a.insert(Key::from_bytes(b"samesite"), s(self.ini.get(b"session.cookie_samesite")));
        Ok(Zval::Array(Rc::new(a)))
    }

    /// `session_set_cookie_params(int|array $lifetime_or_options, ...): bool`.
    pub(super) fn ho_session_set_cookie_params(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if !self.ini.get_bool(b"session.use_cookies") {
            self.diags.push(Diag::Warning(
                "session_set_cookie_params(): Session cookies cannot be used when \
                 session.use_cookies is disabled"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        if self.sess_refuse_active("session_set_cookie_params", "Session cookie parameters")
            || self.sess_refuse_sent("session_set_cookie_params", "Session cookie parameters")
        {
            return Ok(Zval::Bool(false));
        }
        let first = args.first().map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let set = |vm: &mut Self, key: &str, val: Vec<u8>| {
            vm.ini_set_local(format!("session.cookie_{key}").as_bytes(), val);
        };
        if let Zval::Array(opts) = first {
            let mut valid = 0usize;
            for (k, v) in opts.iter() {
                let name = match k {
                    Key::Str(s) => s.as_bytes().to_vec(),
                    Key::Int(_) => continue,
                };
                let sval = match v.deref_clone() {
                    Zval::Bool(b) => (if b { "1" } else { "0" }).into(),
                    other => {
                        String::from_utf8_lossy(convert::to_zstr(&other, &mut self.diags).as_bytes())
                            .into_owned()
                    }
                };
                match &name[..] {
                    b"lifetime" | b"path" | b"domain" | b"secure" | b"httponly"
                    | b"samesite" | b"partitioned" => {
                        set(self, &String::from_utf8_lossy(&name), sval.into_bytes());
                        valid += 1;
                    }
                    _ => {
                        // Each unrecognized key warns; only a set with ZERO
                        // valid keys is a ValueError (variation7).
                        self.diags.push(Diag::Warning(format!(
                            "session_set_cookie_params(): Argument #1 ($lifetime_or_options) \
                             contains an unrecognized key \"{}\"",
                            String::from_utf8_lossy(&name)
                        )));
                    }
                }
            }
            if valid == 0 {
                return Err(PhpError::ValueError(
                    "session_set_cookie_params(): Argument #1 ($lifetime_or_options) must \
                     contain at least 1 valid key"
                        .to_string(),
                ));
            }
            return Ok(Zval::Bool(true));
        }
        // Positional form: lifetime, path, domain, secure, httponly.
        let lifetime = convert::to_long_cast(&first, &mut self.diags);
        set(self, "lifetime", lifetime.to_string().into_bytes());
        let keys = ["path", "domain", "secure", "httponly"];
        for (i, key) in keys.iter().enumerate() {
            if let Some(v) = args.get(i + 1) {
                let v = v.deref_clone();
                if matches!(v, Zval::Null) {
                    continue;
                }
                let sval = match v {
                    Zval::Bool(b) => if b { b"1".to_vec() } else { b"0".to_vec() },
                    other => convert::to_zstr(&other, &mut self.diags).as_bytes().to_vec(),
                };
                set(self, key, sval);
            }
        }
        Ok(Zval::Bool(true))
    }

    /// `session_start(array $options = []): bool`.
    pub(super) fn ho_session_start(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if self.session.active {
            let suffix = self.sess_started_from();
            self.diags.push(Diag::Notice(format!(
                "session_start(): Ignoring session_start() because a session is already active{suffix}"
            )));
            return Ok(Zval::Bool(true));
        }
        if self.output_started && self.ini.get_bool(b"session.use_cookies") {
            let suffix = self.sess_sent_from();
            self.diags.push(Diag::Warning(format!(
                "session_start(): Session cannot be started after headers have already been sent{suffix}"
            )));
            return Ok(Zval::Bool(false));
        }
        // Apply the per-start option overrides (they persist in the ini table,
        // like PHP's zend_alter_ini_entry with PHP_INI_STAGE_RUNTIME).
        let mut read_and_close = false;
        if let Some(Zval::Array(opts)) = args.first().map(|v| v.deref_clone()) {
            for (k, v) in opts.iter() {
                let name = match k {
                    Key::Str(s) => s.as_bytes().to_vec(),
                    Key::Int(_) => {
                        return Err(PhpError::ValueError(
                            "session_start(): Argument #1 ($options) must be of type array \
                             with keys as string"
                                .to_string(),
                        ))
                    }
                };
                let v = v.deref_clone();
                if name == b"read_and_close" {
                    read_and_close = convert::to_bool(&v, &mut self.diags);
                    continue;
                }
                let ini_name = [b"session.", &name[..]].concat();
                let sval = match v {
                    Zval::Bool(b) => if b { b"1".to_vec() } else { b"0".to_vec() },
                    other @ (Zval::Str(_) | Zval::Long(_)) => {
                        convert::to_zstr(&other, &mut self.diags).as_bytes().to_vec()
                    }
                    other => {
                        return Err(PhpError::TypeError(format!(
                            "session_start(): Option \"{}\" must be of type string|int|bool, {} given",
                            String::from_utf8_lossy(&name),
                            other.type_name_for_error()
                        )))
                    }
                };
                let known = self.ini.0.get(&ini_name).is_some_and(|e| e.access_user_settable());
                let valid = known && self.ini_session_value_ok("session_start()", &ini_name, &sval);
                if valid {
                    if let Some(msg) = super::ini::session_state_deprecation(&ini_name, &sval) {
                        self.diags.push(Diag::Deprecated(format!("session_start(): {msg}")));
                    }
                    self.ini_set_local(&ini_name, sval);
                } else {
                    self.diags.push(Diag::Warning(format!(
                        "session_start(): Setting option \"{}\" failed",
                        String::from_utf8_lossy(&name)
                    )));
                }
            }
        }
        // A module misconfigured in php.ini (stored raw at startup) surfaces
        // here: "session startup failed".
        if matches!(self.session.handler, SaveHandler::Files)
            && self.ini.get(b"session.save_handler") != Some(b"files")
        {
            let h = self.ini.get(b"session.save_handler").unwrap_or(b"").to_vec();
            self.diags.push(Diag::Warning(format!(
                "session_start(): Cannot find session save handler \"{}\" - session startup failed",
                String::from_utf8_lossy(&h)
            )));
            return Ok(Zval::Bool(false));
        }
        if !matches!(
            self.ini.get(b"session.serialize_handler"),
            Some(b"php" | b"php_binary" | b"php_serialize")
        ) {
            let h = self.ini.get(b"session.serialize_handler").unwrap_or(b"").to_vec();
            self.diags.push(Diag::Warning(format!(
                "session_start(): Cannot find session serialization handler \"{}\" - session startup failed",
                String::from_utf8_lossy(&h)
            )));
            return Ok(Zval::Bool(false));
        }
        // A partitioned session cookie requires the secure attribute.
        if self.ini.get_bool(b"session.use_cookies")
            && self.ini.get_bool(b"session.cookie_partitioned")
            && !self.ini.get_bool(b"session.cookie_secure")
        {
            self.diags.push(Diag::Warning(
                "session_start(): Partitioned session cookie cannot be used without also \
                 configuring it as secure"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        // The module observes an active session during its own open/read
        // calls (PHP flips the status before initializing); rolled back on
        // any failure below.
        self.session.active = true;
        self.session.open_path = self.ini.get(b"session.save_path").unwrap_or(b"").to_vec();
        // open(save_path, name); failure aborts the start (close still runs,
        // oracle-verified).
        if !self.sess_mod_open()? {
            self.sess_mod_close()?;
            self.sess_start_rollback();
            let (path, module) = (self.sess_dir_open(), self.sess_mod_name());
            self.diags.push(Diag::Warning(format!(
                "session_start(): Failed to initialize storage module: {module} (path: {})",
                path.display()
            )));
            return Ok(Zval::Bool(false));
        }
        // Session id: reuse the one set via session_id(), else (web SAPI) the
        // request cookie, else create one; under strict mode an id the module
        // does not recognize is discarded. An id arriving via the cookie
        // suppresses the response Set-Cookie (PHP sends it only for a new id).
        let mut id = std::mem::take(&mut self.session.id);
        let mut from_cookie = false;
        if self.web && id.is_empty() && self.ini.get_bool(b"session.use_cookies") {
            let name = self.ini.get(b"session.name").unwrap_or(b"PHPSESSID").to_vec();
            let idx = crate::bytecode::superglobal_index(b"_COOKIE").expect("_COOKIE") as usize;
            if let Zval::Array(c) = &self.superglobals[idx] {
                if let Some(Zval::Str(v)) = c.get(&Key::from_bytes(&name)) {
                    if !v.as_bytes().is_empty() {
                        id = v.as_bytes().to_vec();
                        from_cookie = true;
                    }
                }
            }
        }
        let strict = self.ini.get_bool(b"session.use_strict_mode");
        if !id.is_empty() && strict && !self.sess_mod_validate(&id)? {
            id = Vec::new();
            from_cookie = false;
        }
        if id.is_empty() {
            id = self.sess_mod_create_sid()?;
        }
        self.session.id = id.clone();
        // GC roll (gc_probability / gc_divisor).
        let prob = self.ini.get_long(b"session.gc_probability");
        let div = self.ini.get_long(b"session.gc_divisor").max(1);
        if prob > 0 {
            let mut r = [0u8; 8];
            let _ = os_random_fill(&mut r);
            let roll = u64::from_le_bytes(r) as f64 / u64::MAX as f64;
            if roll < prob as f64 / div as f64 {
                self.sess_mod_gc()?;
            }
        }
        // Read (the files module creates the file) and decode.
        let Some(data) = self.sess_mod_read("session_start", &id)? else {
            self.sess_mod_close()?;
            self.sess_start_rollback();
            self.session.id = id; // a failed read keeps the id
            let (path, module) = (self.session_raw_path(), self.sess_mod_name());
            self.diags.push(Diag::Warning(format!(
                "session_start(): Failed to read session data: {module} (path: {})",
                String::from_utf8_lossy(&path)
            )));
            return Ok(Zval::Bool(false));
        };
        match self.sess_decode_data(&data)? {
            Some(arr) => {
                self.sess_write_superglobal(arr);
                self.session.snapshot = Some(data);
            }
            None => {
                self.sess_destroy_on_decode_failure("session_start")?;
                return Ok(Zval::Bool(false));
            }
        }
        // Web SAPI: the session cookie (new ids only) + cache-limiter headers.
        if self.web {
            self.web_session_headers(!from_cookie);
        }
        let top = self.frames.len().saturating_sub(1);
        let file = self.frames.get(top).map(|f| f.module.file.to_vec()).unwrap_or_default();
        let line = self.cur_line(top);
        self.session.started_at = Some((file, line));
        if read_and_close {
            self.sess_mod_close()?;
            self.sess_start_rollback();
        }
        Ok(Zval::Bool(true))
    }

    /// Undo the active-state flip of a failing (or read_and_close) start.
    fn sess_start_rollback(&mut self) {
        self.session.active = false;
        self.session.started_at = None;
        self.session.snapshot = None;
        self.session.open_path = Vec::new();
    }

    /// `session_write_close(): bool` / `session_commit()`.
    pub(super) fn ho_session_write_close(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            return Ok(Zval::Bool(false));
        }
        self.sess_commit("session_write_close()")?;
        Ok(Zval::Bool(true))
    }

    /// `session_abort(): bool` — close without writing; `$_SESSION` keeps its
    /// contents, the id survives.
    pub(super) fn ho_session_abort(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            return Ok(Zval::Bool(false));
        }
        self.session.active = false;
        self.session.started_at = None;
        self.session.snapshot = None;
        self.session.open_path = Vec::new();
        Ok(Zval::Bool(true))
    }

    /// `session_reset(): bool` — re-read the stored data into `$_SESSION`.
    pub(super) fn ho_session_reset(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            return Ok(Zval::Bool(false));
        }
        let id = self.session.id.clone();
        let Some(data) = self.sess_mod_read("session_reset", &id)? else {
            return Ok(Zval::Bool(false));
        };
        match self.sess_decode_data(&data)? {
            Some(arr) => {
                self.sess_write_superglobal(arr);
                self.session.snapshot = Some(data);
                Ok(Zval::Bool(true))
            }
            None => {
                self.sess_destroy_on_decode_failure("session_reset")?;
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `session_unset(): bool` — empty `$_SESSION` (active sessions only).
    pub(super) fn ho_session_unset(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            return Ok(Zval::Bool(false));
        }
        self.sess_write_superglobal(PhpArray::new());
        Ok(Zval::Bool(true))
    }

    /// `session_destroy(): bool` — delete the stored session; `$_SESSION` is
    /// left as-is, the id is cleared.
    pub(super) fn ho_session_destroy(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            self.diags.push(Diag::Warning(
                "session_destroy(): Trying to destroy uninitialized session".to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        let id = std::mem::take(&mut self.session.id);
        self.sess_mod_destroy(&id)?;
        self.sess_mod_close()?;
        self.session.active = false;
        self.session.started_at = None;
        self.session.snapshot = None;
        self.session.open_path = Vec::new();
        Ok(Zval::Bool(true))
    }

    /// `session_gc(): int|false` — run the collector explicitly.
    pub(super) fn ho_session_gc(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            self.diags.push(Diag::Warning(
                "session_gc(): Session cannot be garbage collected when there is no active session"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        Ok(match self.sess_mod_gc()? {
            Some(n) => Zval::Long(n),
            None => Zval::Bool(false),
        })
    }

    /// `session_regenerate_id(bool $delete_old_session = false): bool` — the
    /// current data is written under the OLD id first (or the old session is
    /// destroyed), then the session continues under a fresh id.
    pub(super) fn ho_session_regenerate_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.session.active {
            self.diags.push(Diag::Warning(
                "session_regenerate_id(): Session ID cannot be regenerated when there is no active session"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        if self.output_started {
            let suffix = self.sess_sent_from();
            self.diags.push(Diag::Warning(format!(
                "session_regenerate_id(): Session ID cannot be regenerated after headers have already been sent{suffix}"
            )));
            return Ok(Zval::Bool(false));
        }
        let delete_old = args
            .first()
            .map(|v| convert::to_bool(&v.deref_clone(), &mut self.diags))
            .unwrap_or(false);
        let old_id = self.session.id.clone();
        if delete_old {
            self.sess_mod_destroy(&old_id)?;
        } else if let Some(data) = self.sess_encode_data("session_regenerate_id()")? {
            self.sess_mod_write(&old_id, &data)?;
        }
        // Oracle sequence: the old session closes, then the module reopens and
        // READS the fresh id — which is what creates the new file immediately
        // (bug61470) — while `$_SESSION` keeps the in-memory data.
        self.sess_mod_close()?;
        self.session.id = self.sess_mod_create_sid()?;
        self.sess_mod_open()?;
        let new_id = self.session.id.clone();
        let _ = self.sess_mod_read("session_regenerate_id", &new_id)?;
        // The fresh id has no backing payload yet: force a write at commit.
        self.session.snapshot = None;
        Ok(Zval::Bool(true))
    }

    /// `session_create_id(string $prefix = ""): string|false`.
    pub(super) fn ho_session_create_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let prefix = match args.first().map(|v| v.deref_clone()) {
            None | Some(Zval::Null) => Vec::new(),
            Some(v) => convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec(),
        };
        if prefix.contains(&0) {
            return Err(PhpError::ValueError(
                "session_create_id(): Argument #1 ($prefix) must not contain any null bytes"
                    .to_string(),
            ));
        }
        if prefix.len() > 256 {
            return Err(PhpError::ValueError(
                "session_create_id(): Argument #1 ($prefix) cannot be longer than 256 characters"
                    .to_string(),
            ));
        }
        if !sid_chars_ok(&prefix) {
            self.diags.push(Diag::Warning(
                "session_create_id(): Prefix cannot contain special characters. \
                 Only the A-Z, a-z, 0-9, \"-\", and \",\" characters are allowed"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        // With an active files session, avoid colliding with an existing file.
        let mut id = self.sess_generate_id();
        if self.session.active {
            let dir = self.sess_dir_open();
            for _ in 0..3 {
                if !self.sess_file(&dir, &id).exists() {
                    break;
                }
                id = self.sess_generate_id();
            }
        }
        let mut out = prefix;
        out.extend_from_slice(&id);
        Ok(Zval::Str(PhpStr::new(out)))
    }

    /// `session_encode(): string|false`.
    pub(super) fn ho_session_encode(&mut self) -> Result<Zval, PhpError> {
        if !self.session.active {
            self.diags.push(Diag::Warning(
                "session_encode(): Cannot encode non-existent session".to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        match self.sess_encode_data("session_encode()")? {
            Some(data) => Ok(Zval::Str(PhpStr::new(data))),
            None => Ok(Zval::Bool(false)),
        }
    }

    /// `session_decode(string $data): bool` — malformed input destroys the
    /// session (oracle contract).
    pub(super) fn ho_session_decode(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.session.active {
            self.diags.push(Diag::Warning(
                "session_decode(): Session data cannot be decoded when there is no active session"
                    .to_string(),
            ));
            return Ok(Zval::Bool(false));
        }
        let data = match args.first().map(|v| v.deref_clone()) {
            Some(v) => convert::to_zstr(&v, &mut self.diags).as_bytes().to_vec(),
            None => Vec::new(),
        };
        match self.sess_decode_data(&data)? {
            Some(arr) => {
                self.sess_write_superglobal(arr);
                Ok(Zval::Bool(true))
            }
            None => {
                self.sess_destroy_on_decode_failure("session_decode")?;
                Ok(Zval::Bool(false))
            }
        }
    }

    /// `session_register_shutdown(): void` — queue `session_write_close` at
    /// the current position of the shutdown-function list.
    pub(super) fn ho_session_register_shutdown(&mut self) -> Result<Zval, PhpError> {
        self.shutdown_fns
            .push((Zval::Str(PhpStr::from_str("session_write_close")), Vec::new()));
        Ok(Zval::Null)
    }

    /// `session_set_save_handler(SessionHandlerInterface $handler, bool
    /// $register_shutdown = true): bool` — or the deprecated positional form
    /// `(open, close, read, write, destroy, gc, ?create_sid, ?validate_sid,
    /// ?update_timestamp)`. Selects the 'user' module.
    pub(super) fn ho_session_set_save_handler(
        &mut self,
        args: Vec<Zval>,
    ) -> Result<Zval, PhpError> {
        if self.sess_refuse_active("session_set_save_handler", "Session save handler")
            || self.sess_refuse_sent("session_set_save_handler", "Session save handler")
        {
            return Ok(Zval::Bool(false));
        }
        let first = args.first().map(|v| v.deref_clone()).unwrap_or(Zval::Null);
        let handler = if args.len() <= 2 {
            // Object form.
            let Zval::Object(obj) = &first else {
                return Err(PhpError::TypeError(format!(
                    "session_set_save_handler(): Argument #1 ($handler) must be of type \
                     SessionHandlerInterface, {} given",
                    first.type_name_for_error()
                )));
            };
            let cid = obj.borrow().class_id as usize;
            let implements = |vm: &Self, iface: &[u8]| {
                vm.class_index
                    .get(iface)
                    .is_some_and(|&t| vm.instance_of(cid, t as usize))
            };
            if !implements(self, b"sessionhandlerinterface") {
                return Err(PhpError::TypeError(format!(
                    "session_set_save_handler(): Argument #1 ($handler) must be of type \
                     SessionHandlerInterface, {} given",
                    first.type_name_for_error()
                )));
            }
            let method = |name: &str| {
                let mut a = PhpArray::new();
                let _ = a.append(first.clone());
                let _ = a.append(Zval::Str(PhpStr::from_str(name)));
                Zval::Array(Rc::new(a))
            };
            let has_create_sid = implements(self, b"sessionidinterface");
            let has_ts = implements(self, b"sessionupdatetimestamphandlerinterface");
            let register_shutdown = match args.get(1) {
                Some(v) => convert::to_bool(&v.deref_clone(), &mut self.diags),
                None => true,
            };
            if register_shutdown {
                self.ho_session_register_shutdown()?;
            }
            let class_name = self.classes[cid].name.to_vec();
            UserHandler {
                open: method("open"),
                close: method("close"),
                read: method("read"),
                write: method("write"),
                destroy: method("destroy"),
                gc: method("gc"),
                create_sid: has_create_sid.then(|| method("create_sid")),
                validate_id: has_ts.then(|| method("validateId")),
                update_timestamp: has_ts.then(|| method("updateTimestamp")),
                class_name: Some(class_name),
            }
        } else {
            // Positional-callables form (deprecated since 8.4).
            self.diags.push(Diag::Deprecated(
                "session_set_save_handler(): Providing individual callbacks instead of an \
                 object implementing SessionHandlerInterface is deprecated"
                    .to_string(),
            ));
            if args.len() < 6 {
                return Err(PhpError::ArgumentCountError(format!(
                    "session_set_save_handler() expects at least 6 arguments, {} given",
                    args.len()
                )));
            }
            let cb = |i: usize| args[i].deref_clone();
            let opt = |i: usize| {
                args.get(i)
                    .map(|v| v.deref_clone())
                    .filter(|v| !matches!(v, Zval::Null))
            };
            // Each positional argument must be callable (bug31454).
            let names = [
                "open", "close", "read", "write", "destroy", "gc", "create_sid",
                "validate_sid", "update_timestamp",
            ];
            for (i, pname) in names.iter().enumerate().take(args.len().min(9)) {
                let v = args[i].deref_clone();
                if matches!(v, Zval::Null) && i >= 6 {
                    continue;
                }
                if !self.is_value_callable(&v) {
                    let reason = match &v {
                        Zval::Array(_) => "first array member is not a valid class name or object"
                            .to_string(),
                        Zval::Str(s) => format!(
                            "function \"{}\" not found or invalid function name",
                            String::from_utf8_lossy(s.as_bytes())
                        ),
                        other => format!("no array or string given, {} given", other.type_name_for_error()),
                    };
                    return Err(PhpError::TypeError(format!(
                        "session_set_save_handler(): Argument #{} (${pname}) must be a valid \
                         callback, {reason}",
                        i + 1
                    )));
                }
            }
            UserHandler {
                open: cb(0),
                close: cb(1),
                read: cb(2),
                write: cb(3),
                destroy: cb(4),
                gc: cb(5),
                create_sid: opt(6),
                validate_id: opt(7),
                update_timestamp: opt(8),
                class_name: None,
            }
        };
        self.session.handler = SaveHandler::User(Box::new(handler));
        self.ini_set_local(b"session.save_handler", b"user".to_vec());
        Ok(Zval::Bool(true))
    }

    /// `__session_files_op(op, ...)` — the prelude `SessionHandler` class
    /// delegates to the built-in files module through this hook. Outside an
    /// open session PHP throws: "Session is not active".
    pub(super) fn ho_session_files_op(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        if !self.session.active && !self.session.committing {
            return Err(PhpError::Error("Session is not active".to_string()));
        }
        let op = args
            .first()
            .map(|v| convert::to_zstr(&v.deref_clone(), &mut self.diags).as_bytes().to_vec())
            .unwrap_or_default();
        let str_arg = |vm: &mut Self, i: usize| {
            args.get(i)
                .map(|v| convert::to_zstr(&v.deref_clone(), &mut vm.diags).as_bytes().to_vec())
                .unwrap_or_default()
        };
        // Ops other than open require the parent module to have been opened
        // by THIS handler (a subclass overriding open() without calling
        // parent::open leaves it closed — class_005).
        if !self.session.files_open && !matches!(&op[..], b"open" | b"create_sid") {
            self.diags.push(Diag::Warning(format!(
                "SessionHandler::{}(): Parent session handler is not open",
                String::from_utf8_lossy(&op)
            )));
            return Ok(Zval::Bool(false));
        }
        match &op[..] {
            b"open" => {
                // A SessionHandler subclass may hand a custom path to
                // parent::open — it becomes the files module's directory.
                let path = str_arg(self, 1);
                self.session.open_path = path;
                self.session.files_open = true;
                Ok(Zval::Bool(self.sess_dir_open().is_dir()))
            }
            b"close" => {
                self.session.files_open = false;
                Ok(Zval::Bool(true))
            }
            b"read" => {
                let id = str_arg(self, 1);
                match self.sess_files_read("SessionHandler::read", &id) {
                    Ok(data) => Ok(Zval::Str(PhpStr::new(data))),
                    Err(()) => Ok(Zval::Bool(false)),
                }
            }
            b"write" => {
                let id = str_arg(self, 1);
                let data = str_arg(self, 2);
                self.sess_files_write(&id, &data);
                Ok(Zval::Bool(true))
            }
            b"destroy" => {
                let id = str_arg(self, 1);
                self.sess_files_destroy(&id);
                Ok(Zval::Bool(true))
            }
            b"gc" => Ok(Zval::Long(self.sess_files_gc())),
            b"create_sid" => Ok(Zval::Str(PhpStr::new(self.sess_generate_id()))),
            other => Err(PhpError::Error(format!(
                "__session_files_op: unknown op {}",
                String::from_utf8_lossy(other)
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_to_readable_charsets() {
        let bytes = [0xff; 32];
        let hex = bin_to_readable(&bytes, 32, 4);
        assert_eq!(hex.len(), 32);
        assert!(hex.iter().all(|b| b"0123456789abcdef".contains(b)));
        let b5 = bin_to_readable(&bytes, 32, 5);
        assert!(b5.iter().all(|b| b.is_ascii_digit() || (b'a'..=b'v').contains(b)));
        let b6 = bin_to_readable(&bytes, 32, 6);
        assert!(b6
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b',')));
        // 0xff with 4 bits per char is all 'f'.
        assert!(hex.iter().all(|b| *b == b'f'));
    }

    #[test]
    fn sid_charset_check() {
        assert!(sid_chars_ok(b"abc-DEF,123"));
        assert!(sid_chars_ok(b""));
        assert!(!sid_chars_ok(b"bad id"));
        assert!(!sid_chars_ok(b"x!"));
    }
}
