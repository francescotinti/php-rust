//! ext/pdo + pdo_sqlite subset (host side of the prelude `PDO*` classes),
//! backed by rusqlite (bundled sqlite3). Only the sqlite driver exists: any
//! other DSN prefix reports "could not find driver", exactly like a PHP build
//! without that driver compiled in.
//!
//! Model: each `PDO` object owns an open [`rusqlite::Connection`] in
//! `Vm.pdo_conns`, addressed by an int handle the prelude class keeps in a
//! private prop; the class methods delegate to the `__pdo_*` host builtins
//! registered in `vm/mod.rs` (the ext/zip `__zip_*` pattern). rusqlite
//! `Statement`s borrow their `Connection`, so no prepared statement is stored
//! across calls: `PDOStatement` keeps the SQL text and the host re-prepares at
//! each execute() — observably identical for sqlite (no server-side state).
//!
//! Error protocol: a failing host op returns the PHP array
//! `[message, code, sqlstate|null, native-msg|null]`; the prelude side builds
//! and throws/reports the `PDOException` (message, int/string code and
//! errorInfo triple faithful to ext/pdo's formatting).

use std::rc::Rc;

use php_types::{convert, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

/// The `[message, code, sqlstate, native-msg]` error payload (see module doc).
fn pdo_err(message: &str, code: i64, state: Option<(&str, &str)>) -> Zval {
    let mut out = PhpArray::new();
    let _ = out.append(Zval::Str(PhpStr::new(message.as_bytes().to_vec())));
    let _ = out.append(Zval::Long(code));
    match state {
        Some((sqlstate, native)) => {
            let _ = out.append(Zval::Str(PhpStr::new(sqlstate.as_bytes().to_vec())));
            let _ = out.append(Zval::Str(PhpStr::new(native.as_bytes().to_vec())));
        }
        None => {
            let _ = out.append(Zval::Null);
            let _ = out.append(Zval::Null);
        }
    }
    Zval::Array(Rc::new(out))
}

/// A rusqlite error as PDO reports it: the sqlite *primary* result code (PHP
/// calls `sqlite3_errcode`; the extended code's low byte) and the bare native
/// message.
fn native_err(e: &rusqlite::Error) -> (i64, String) {
    match e {
        rusqlite::Error::SqliteFailure(f, msg) => (
            i64::from(f.extended_code & 0xff),
            msg.clone().unwrap_or_else(|| f.to_string()),
        ),
        other => (1, other.to_string()),
    }
}

/// The payload for a general driver error: `SQLSTATE[HY000] [N] msg` with the
/// native code, as ext/pdo formats a *connection-time* failure. rusqlite
/// decorates the sqlite message with `: <path>`; PHP reports the bare
/// `sqlite3_errmsg` text, so the known decoration is stripped.
fn pdo_conn_err(e: &rusqlite::Error, path: &str) -> Zval {
    let (code, mut msg) = native_err(e);
    if let Some(bare) = msg.strip_suffix(&format!(": {path}")) {
        msg = bare.to_string();
    }
    pdo_err(&format!("SQLSTATE[HY000] [{code}] {msg}"), code, Some(("HY000", &msg)))
}

impl<'m> Vm<'m> {
    /// `__pdo_open($dsn)` (the prelude `PDO::__construct` backing): parse the
    /// DSN and open the database. Success: the int handle. Failure: the error
    /// payload — "could not find driver" (code 0, no SQLSTATE) for a
    /// non-sqlite prefix, "invalid data source name" for a colonless DSN, or
    /// the connection-error format for an open failure (oracle: code 14,
    /// errorInfo `['HY000', 14, 'unable to open database file']`).
    pub(super) fn ho_pdo_open(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let dsn = convert::to_zstr_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags);
        let dsn = dsn.as_bytes();
        let Some(colon) = dsn.iter().position(|&b| b == b':') else {
            return Ok(pdo_err("invalid data source name", 0, None));
        };
        if &dsn[..colon] != b"sqlite" {
            return Ok(pdo_err("could not find driver", 0, None));
        }
        let path = String::from_utf8_lossy(&dsn[colon + 1..]).into_owned();
        let conn = if path == ":memory:" {
            rusqlite::Connection::open_in_memory()
        } else {
            // sqlite itself treats "" as a private temporary database, matching
            // PHP's `new PDO('sqlite:')`; a plain path is created read-write.
            rusqlite::Connection::open(&path)
        };
        match conn {
            Ok(c) => {
                let id = self.next_pdo;
                self.next_pdo += 1;
                self.pdo_conns.insert(id, c);
                Ok(Zval::Long(i64::from(id)))
            }
            Err(e) => Ok(pdo_conn_err(&e, &path)),
        }
    }

    /// `__pdo_close($id)`: release the handle. `false` on an unknown/closed one.
    pub(super) fn ho_pdo_close(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        Ok(Zval::Bool(self.pdo_conns.remove(&id).is_some()))
    }

    /// `__pdo_sqlite_version()`: the bundled sqlite3 library version, what
    /// `PDO::getAttribute(ATTR_SERVER_VERSION)` reports for the sqlite driver.
    pub(super) fn ho_pdo_sqlite_version(&mut self) -> Result<Zval, PhpError> {
        Ok(Zval::Str(PhpStr::new(rusqlite::version().as_bytes().to_vec())))
    }
}
