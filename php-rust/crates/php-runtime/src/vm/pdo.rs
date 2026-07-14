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

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

thread_local! {
    /// VM re-entry pointer for PHP-defined SQLite UDFs (`createFunction`):
    /// set around every statement/batch execution on a PDO connection so the
    /// rusqlite scalar-function closure (which must be `'static`) can call
    /// back into the PHP callable while sqlite is stepping. The VM is
    /// single-threaded and the outer `&mut self` is suspended inside sqlite
    /// while the callback runs — the same shape as php-src re-entering its
    /// executor from the UDF hook.
    static ACTIVE_VM: Cell<*mut ()> = const { Cell::new(std::ptr::null_mut()) };
    /// A PhpError raised by a UDF's PHP callback. sqlite only carries an
    /// error *string* out of the step loop, so the original error (a thrown
    /// exception object, say) is parked here and re-raised by the host op
    /// that ran the statement — PHP propagates the callback's exception.
    static UDF_ERROR: RefCell<Option<PhpError>> = const { RefCell::new(None) };
}

/// A PHP callable captured by a sqlite UDF closure. rusqlite requires the
/// closure to be `Send + UnwindSafe`, but a `Zval` is `Rc`-based: safe here
/// because the `Connection` and the VM live and die on this one thread, and
/// the process aborts on a Rust panic (no unwinding into sqlite).
struct UdfCallable(Zval);
unsafe impl Send for UdfCallable {}
impl std::panic::UnwindSafe for UdfCallable {}
impl UdfCallable {
    /// Accessor instead of direct `.0` field use inside the UDF closure:
    /// edition-2021 disjoint capture would otherwise capture the bare `Zval`
    /// field and bypass this wrapper's `Send` assertion.
    fn get(&self) -> Zval {
        self.0.clone()
    }
}

/// Install the VM re-entry pointer for the duration of a statement run;
/// restores the previous value on drop (UDF-triggered nested statements).
struct VmReentry(*mut ());
impl VmReentry {
    fn install(vm: &mut Vm<'_>) -> Self {
        let p = vm as *mut Vm<'_> as *mut ();
        VmReentry(ACTIVE_VM.replace(p))
    }
}
impl Drop for VmReentry {
    fn drop(&mut self) {
        ACTIVE_VM.set(self.0);
    }
}

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

/// Register the math functions the bundled sqlite lacks (it ships without
/// `SQLITE_ENABLE_MATH_FUNCTIONS`). Doctrine's DQL maps `SQRT(x)` straight to a
/// `SQRT(...)` SQL call (QueryDqlFunctionTest::testFunctionSqrt); real PHP links
/// a sqlite built with the math extension, so these names must exist. Names are
/// matched case-insensitively by sqlite. Best-effort: a registration error is
/// ignored (the query then fails exactly as before).
fn register_sqlite_math(c: &rusqlite::Connection) {
    use rusqlite::functions::FunctionFlags;
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    let _ = c.create_scalar_function("sqrt", 1, flags, |ctx| {
        let x: Option<f64> = ctx.get(0)?;
        Ok(x.map(f64::sqrt))
    });
    let _ = c.create_scalar_function("power", 2, flags, |ctx| {
        let base: Option<f64> = ctx.get(0)?;
        let exp: Option<f64> = ctx.get(1)?;
        Ok(match (base, exp) {
            (Some(b), Some(e)) => Some(b.powf(e)),
            _ => None,
        })
    });
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
        // Prepare-time syntax errors: rusqlite's Display decorates them with
        // ` in <sql> at offset N`; PHP reports the bare sqlite message.
        rusqlite::Error::SqlInputError { error, msg, .. } => {
            (i64::from(error.extended_code & 0xff), msg.clone())
        }
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

/// A runtime (statement-level) error as the dict `['err' => payload]` the
/// prelude query path expects. The SQLSTATE is derived from the sqlite primary
/// code exactly like pdo_sqlite's `_pdo_sqlite_error` mapper, and the message
/// is PDO's `SQLSTATE[state]: Description: code msg` runtime format (unlike
/// the connection-time `SQLSTATE[HY000] [code] msg` one). PDOException codes
/// for these are the SQLSTATE *string* (payload[1]).
fn stmt_err(code: i64, msg: &str) -> Zval {
    let state = match code {
        19 => "23000", // SQLITE_CONSTRAINT
        18 => "22001", // SQLITE_TOOBIG
        9 => "01002",  // SQLITE_INTERRUPT
        22 => "HYC00", // SQLITE_NOLFS
        _ => "HY000",
    };
    let desc = match state {
        "23000" => "Integrity constraint violation",
        "22001" => "String data, right truncated",
        "01002" => "Disconnect error",
        "HYC00" => "Optional feature not implemented",
        _ => "General error",
    };
    let mut payload = PhpArray::new();
    let full = format!("SQLSTATE[{state}]: {desc}: {code} {msg}");
    let _ = payload.append(Zval::Str(PhpStr::new(full.into_bytes())));
    let _ = payload.append(Zval::Str(PhpStr::new(state.as_bytes().to_vec())));
    let _ = payload.append(Zval::Long(code));
    let _ = payload.append(Zval::Str(PhpStr::new(msg.as_bytes().to_vec())));
    let mut out = PhpArray::new();
    out.insert(Key::from_bytes(b"err"), Zval::Array(Rc::new(payload)));
    Zval::Array(Rc::new(out))
}

fn stmt_err_of(e: &rusqlite::Error) -> Zval {
    let (code, msg) = native_err(e);
    stmt_err(code, &msg)
}

/// Every parameter-binding defect (count mismatch on an execute(array), a
/// named placeholder that does not exist, a position out of range) surfaces in
/// PHP as sqlite's SQLITE_RANGE — oracle: `SQLSTATE[HY000]: General error: 25
/// column index out of range` — so one payload covers them all.
fn range_err() -> Zval {
    stmt_err(25, "column index out of range")
}

/// A PHP value as the sqlite value pdo_sqlite would bind: the type coercions
/// (PARAM_INT & co.) already happened prelude-side, so binding follows the
/// zval type. Non-UTF-8 strings bind as blobs (rusqlite's Text is a String;
/// sqlite itself does not re-validate on read-back).
fn zval_to_sql(v: &Zval) -> rusqlite::types::Value {
    use rusqlite::types::Value;
    match v.deref_clone() {
        Zval::Null => Value::Null,
        Zval::Bool(b) => Value::Integer(i64::from(b)),
        Zval::Long(i) => Value::Integer(i),
        Zval::Double(f) => Value::Real(f),
        Zval::Str(s) => match std::str::from_utf8(s.as_bytes()) {
            Ok(t) => Value::Text(t.to_string()),
            Err(_) => Value::Blob(s.as_bytes().to_vec()),
        },
        _ => Value::Null,
    }
}

/// A result cell as PHP 8.1+ pdo_sqlite reports it natively (STRINGIFY off):
/// INTEGER→int, REAL→float, TEXT/BLOB→string, NULL→null.
fn sql_to_zval(v: rusqlite::types::ValueRef<'_>) -> Zval {
    use rusqlite::types::ValueRef;
    match v {
        ValueRef::Null => Zval::Null,
        ValueRef::Integer(i) => Zval::Long(i),
        ValueRef::Real(f) => Zval::Double(f),
        ValueRef::Text(t) => Zval::Str(PhpStr::new(t.to_vec())),
        ValueRef::Blob(b) => Zval::Str(PhpStr::new(b.to_vec())),
    }
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
                register_sqlite_math(&c);
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

    /// `__pdo_exec($id, $sql)` (the `PDO::exec` backing): run the whole string
    /// (sqlite3_exec semantics: multiple `;`-separated statements) and report
    /// `['changes' => N]` — N from the last modifying statement, 0 for a pure
    /// SELECT — or `['err' => payload]`.
    pub(super) fn ho_pdo_exec(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let sql = String::from_utf8_lossy(sql.as_bytes()).into_owned();
        // Same take-out/re-entry dance as ho_pdo_run: a UDF may run mid-batch.
        let Some(conn) = self.pdo_conns.remove(&id) else {
            return Ok(stmt_err(21, "library routine called out of sequence"));
        };
        let out = {
            let _reentry = VmReentry::install(self);
            match conn.execute_batch(&sql) {
                Ok(()) => {
                    let mut out = PhpArray::new();
                    out.insert(Key::from_bytes(b"changes"), Zval::Long(conn.changes() as i64));
                    Zval::Array(Rc::new(out))
                }
                Err(e) => stmt_err_of(&e),
            }
        };
        self.pdo_conns.insert(id, conn);
        if let Some(pe) = UDF_ERROR.with(|u| u.borrow_mut().take()) {
            return Err(pe);
        }
        Ok(out)
    }

    /// `__pdo_run($id, $sql, $params, $strict)` (the `PDOStatement::execute`
    /// backing): prepare, bind, and run one statement. `$params` carries
    /// 1-based int keys for positional placeholders and (with or without the
    /// `:`) names for named ones; values are already PARAM_*-coerced prelude
    /// side. `$strict` is the execute(array) path, where PDO additionally
    /// requires the array to cover the placeholders exactly (oracle: the
    /// SQLITE_RANGE error); the bindValue path leaves gaps as sqlite NULLs.
    /// Success: `['cols' =>…, 'rows' =>…]` for a row-returning statement
    /// (rows fully materialized: sqlite has no server cursor to preserve) or
    /// `['changes' => N]`; failure: `['err' => payload]`.
    pub(super) fn ho_pdo_run(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let sql = String::from_utf8_lossy(sql.as_bytes()).into_owned();
        let strict = matches!(args.get(3), Some(Zval::Bool(true)));
        let params = args.get(2).map(|a| a.deref_clone());
        // Take the connection OUT of the table for the run: a UDF callback
        // re-enters the VM through ACTIVE_VM, and must not find an aliasing
        // borrow of this connection (a nested statement on the same handle
        // reads as "out of sequence", like sqlite's own re-entrancy guard).
        let Some(conn) = self.pdo_conns.remove(&id) else {
            return Ok(stmt_err(21, "library routine called out of sequence"));
        };
        let out = {
            let _reentry = VmReentry::install(self);
            run_prepared(&conn, &sql, params, strict)
        };
        self.pdo_conns.insert(id, conn);
        // A UDF callback that errored parked the original PhpError (thrown
        // exception): re-raise it so it propagates as PHP does, instead of
        // the flattened sqlite error text.
        if let Some(pe) = UDF_ERROR.with(|u| u.borrow_mut().take()) {
            return Err(pe);
        }
        Ok(out)
    }
}

/// The prepare/bind/step body of [`Vm::ho_pdo_run`], self-borrow-free so the
/// UDF re-entry pointer is the only live route to the VM while sqlite steps.
fn run_prepared(
    conn: &rusqlite::Connection,
    sql: &str,
    params: Option<Zval>,
    strict: bool,
) -> Zval {
    {
        let mut stmt = match conn.prepare(sql) {
            Ok(s) => s,
            Err(e) => return stmt_err_of(&e),
        };
        let pc = stmt.parameter_count();
        let _ = strict;
        if let Some(Zval::Array(params)) = params {
            // Oracle (8.5) shape: placeholders left UNBOUND read as sqlite
            // NULL with no error — execute(array()) with named placeholders is
            // fine (the sqlite plugin's pragma_table_info(:table_name) probe).
            // Binding an UNKNOWN name or an out-of-range position is the
            // SQLITE_RANGE error, on both the execute(array) and bindValue
            // paths.
            for (k, v) in params.iter() {
                let pos = match k {
                    Key::Int(i) => *i,
                    Key::Str(name) => {
                        let mut n = Vec::with_capacity(name.as_bytes().len() + 1);
                        if !name.as_bytes().starts_with(b":") {
                            n.push(b':');
                        }
                        n.extend_from_slice(name.as_bytes());
                        match std::str::from_utf8(&n).ok().and_then(|n| stmt.parameter_index(n).ok().flatten()) {
                            Some(i) => i as i64,
                            None => return range_err(),
                        }
                    }
                };
                if pos < 1 || pos as usize > pc {
                    return range_err();
                }
                if stmt.raw_bind_parameter(pos as usize, zval_to_sql(v)).is_err() {
                    return range_err();
                }
            }
        }
        let cc = stmt.column_count();
        if cc > 0 {
            let mut cols = PhpArray::new();
            for name in stmt.column_names() {
                let _ = cols.append(Zval::Str(PhpStr::new(name.as_bytes().to_vec())));
            }
            // Per-column getColumnMeta statics: [decl type | null, table | null]
            // (both absent on an expression column); the value-derived
            // native_type/pdo_type come from the materialized first row,
            // prelude-side.
            let mut meta = PhpArray::new();
            {
                let decls = stmt.columns();
                let origins = stmt.columns_with_metadata();
                for i in 0..cc {
                    let mut m = PhpArray::new();
                    let _ = m.append(match decls.get(i).and_then(|c| c.decl_type()) {
                        Some(d) => Zval::Str(PhpStr::new(d.as_bytes().to_vec())),
                        None => Zval::Null,
                    });
                    let _ = m.append(match origins.get(i).and_then(|c| c.table_name()) {
                        Some(t) => Zval::Str(PhpStr::new(t.as_bytes().to_vec())),
                        None => Zval::Null,
                    });
                    let _ = meta.append(Zval::Array(Rc::new(m)));
                }
            }
            let mut rows_out = PhpArray::new();
            let mut rows = stmt.raw_query();
            loop {
                match rows.next() {
                    Ok(Some(row)) => {
                        let mut vals = PhpArray::new();
                        for i in 0..cc {
                            let cell = row.get_ref(i).map(sql_to_zval).unwrap_or(Zval::Null);
                            let _ = vals.append(cell);
                        }
                        let _ = rows_out.append(Zval::Array(Rc::new(vals)));
                    }
                    Ok(None) => break,
                    Err(e) => return stmt_err_of(&e),
                }
            }
            let mut out = PhpArray::new();
            out.insert(Key::from_bytes(b"cols"), Zval::Array(Rc::new(cols)));
            out.insert(Key::from_bytes(b"rows"), Zval::Array(Rc::new(rows_out)));
            out.insert(Key::from_bytes(b"meta"), Zval::Array(Rc::new(meta)));
            Zval::Array(Rc::new(out))
        } else {
            match stmt.raw_execute() {
                Ok(n) => {
                    let mut out = PhpArray::new();
                    out.insert(Key::from_bytes(b"changes"), Zval::Long(n as i64));
                    Zval::Array(Rc::new(out))
                }
                Err(e) => stmt_err_of(&e),
            }
        }
    }
}

impl<'m> Vm<'m> {
    /// `__pdo_create_function($id, $name, $callback, $argc, $flags)` — the
    /// host side of `Pdo\Sqlite::createFunction` / `PDO::sqliteCreateFunction`:
    /// register a PHP callable as a sqlite scalar function on the connection.
    /// The callable runs mid-query via the ACTIVE_VM re-entry pointer.
    /// `true` on success, the error payload on a registration failure.
    pub(super) fn ho_pdo_create_function(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        use rusqlite::functions::FunctionFlags;
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let name = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let name = String::from_utf8_lossy(name.as_bytes()).into_owned();
        let cb = UdfCallable(args.get(2).cloned().unwrap_or(Zval::Null).deref_clone());
        let argc = args
            .get(3)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(-1);
        let php_flags = args
            .get(4)
            .map(|v| convert::to_long_cast(v, &mut self.diags))
            .unwrap_or(0);
        let Some(conn) = self.pdo_conns.get(&id) else {
            return Ok(stmt_err(21, "library routine called out of sequence"));
        };
        let mut flags = FunctionFlags::SQLITE_UTF8;
        if php_flags & 2048 != 0 {
            // PDO::SQLITE_DETERMINISTIC
            flags |= FunctionFlags::SQLITE_DETERMINISTIC;
        }
        let r = conn.create_scalar_function(name.as_str(), argc as i32, flags, move |fctx| {
            let p = ACTIVE_VM.get();
            if p.is_null() {
                return Err(rusqlite::Error::UserFunctionError(
                    "phpr: no active VM for a SQLite UDF".into(),
                ));
            }
            // SAFETY: single-threaded VM; the outer &mut self that installed
            // the pointer is suspended inside sqlite's step loop while this
            // callback runs, and the connection was moved out of Vm.pdo_conns
            // for the duration (no aliasing through the VM).
            let vm: &mut Vm<'static> = unsafe { &mut *(p as *mut Vm<'static>) };
            let mut argv = Vec::with_capacity(fctx.len());
            for i in 0..fctx.len() {
                argv.push(sql_to_zval(fctx.get_raw(i)));
            }
            match vm.call_callable(cb.get(), argv) {
                Ok(v) => Ok(zval_to_sql(&v)),
                Err(e) => {
                    let msg = e.message().to_owned();
                    UDF_ERROR.with(|u| *u.borrow_mut() = Some(e));
                    Err(rusqlite::Error::UserFunctionError(msg.into()))
                }
            }
        });
        match r {
            Ok(()) => Ok(Zval::Bool(true)),
            Err(e) => Ok(stmt_err_of(&e)),
        }
    }

    /// `__pdo_prepare($id, $sql)`: compile-check only. pdo_sqlite prepares
    /// eagerly (no EMULATE_PREPARES), so `PDO::prepare` on broken SQL fails
    /// *immediately* (false/throw per ERRMODE); the checked statement is
    /// discarded here and re-prepared at execute (see module doc). `true` or
    /// `['err' => payload]`.
    pub(super) fn ho_pdo_prepare(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let sql = String::from_utf8_lossy(sql.as_bytes()).into_owned();
        let Some(conn) = self.pdo_conns.get(&id) else {
            return Ok(stmt_err(21, "library routine called out of sequence"));
        };
        match conn.prepare(&sql) {
            Ok(_) => Ok(Zval::Bool(true)),
            Err(e) => Ok(stmt_err_of(&e)),
        }
    }

    /// `__pdo_stmt_readonly($id, $sql)`: sqlite3_stmt_readonly on a fresh
    /// prepare, what `PDOStatement::getAttribute(SQLITE_ATTR_READONLY_STATEMENT)`
    /// reports. `false` on any error (the attribute read does not raise).
    pub(super) fn ho_pdo_stmt_readonly(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let sql = String::from_utf8_lossy(sql.as_bytes()).into_owned();
        let Some(conn) = self.pdo_conns.get(&id) else { return Ok(Zval::Bool(false)) };
        Ok(Zval::Bool(conn.prepare(&sql).map(|s| s.readonly()).unwrap_or(false)))
    }

    /// `__pdo_changes($id)`: sqlite3_changes — rows affected by the most
    /// recent statement on the connection (`SQLite3::changes`,
    /// `SQLite3Result::rowCount` via the DBAL driver).
    pub(super) fn ho_pdo_changes(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        Ok(Zval::Long(self.pdo_conns.get(&id).map(|c| c.changes() as i64).unwrap_or(0)))
    }

    /// `__pdo_param_count($id, $sql)`: sqlite3_bind_parameter_count on a fresh
    /// prepare (`SQLite3Stmt::paramCount`). 0 on any error (the count read
    /// does not raise; prepare already validated the SQL).
    pub(super) fn ho_pdo_param_count(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags);
        let sql = String::from_utf8_lossy(sql.as_bytes()).into_owned();
        let Some(conn) = self.pdo_conns.get(&id) else { return Ok(Zval::Long(0)) };
        Ok(Zval::Long(conn.prepare(&sql).map(|s| s.parameter_count() as i64).unwrap_or(0)))
    }

    /// `__pdo_in_txn($id)`: whether a transaction is open, as pdo_sqlite's
    /// in_transaction handler reports it (`!sqlite3_get_autocommit`, so a
    /// manual `exec('BEGIN')` counts too).
    pub(super) fn ho_pdo_in_txn(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        Ok(Zval::Bool(self.pdo_conns.get(&id).is_some_and(|c| !c.is_autocommit())))
    }

    /// `__pdo_last_id($id)`: sqlite's last-inserted rowid (0 before any
    /// insert); `PDO::lastInsertId` stringifies it prelude-side.
    pub(super) fn ho_pdo_last_id(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let Some(conn) = self.pdo_conns.get(&id) else {
            return Ok(Zval::Long(0));
        };
        Ok(Zval::Long(conn.last_insert_rowid()))
    }
}
