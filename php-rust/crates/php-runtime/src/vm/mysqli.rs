//! ext/mysqli subset (host side of the prelude `mysqli*` classes), backed by
//! the pure-Rust `mysql` crate (sync client, caching_sha2 auth). WordPress'
//! wpdb is the reference consumer: the procedural API it uses
//! (mysqli_init/real_connect/query/fetch_* /error/errno/escape/charset) is
//! modelled faithfully against the PHP 8.5 oracle probes (see the WP-8 diary).
//!
//! Model: each `mysqli` object owns an open [`mysql::Conn`] in
//! `Vm.mysqli_conns`, addressed by an int handle kept in a private prop of the
//! prelude class; prepared statements live in `Vm.mysqli_stmts` the same way.
//! Result sets are fully materialized host-side and returned to PHP as one
//! payload array — the prelude `mysqli_result` keeps rows/fields/cursor as
//! plain PHP state (MySQL's text protocol returns every non-NULL cell as a
//! string, so the buffered copy is byte-faithful; the binary protocol of
//! prepared statements returns native ints/floats, preserved as such).
//!
//! Payload protocol (host → prelude): an assoc array with `t` ∈
//! `ok` (status query: affected/insert_id/warnings/info) ·
//! `res` (result set: fields/rows + the same counters) ·
//! `err` (errno/error/sqlstate). Connect returns `t=ok` with the handle and
//! the connection facts (server_info/host_info/thread_id/…) or `t=err` with
//! the client-format errno/message (2002 "Connection refused" & co.).

use std::collections::VecDeque;

use mysql::prelude::Queryable;
use mysql::{Column, Conn, OptsBuilder, Value};
use php_types::{convert, Key, PhpArray, PhpError, PhpStr, Zval};

use super::Vm;

/// One open connection: the wire handle plus the client-side state mysqli
/// exposes (current charset name, buffered extra result sets of a
/// `multi_query`, the last statement's info string).
pub(super) struct MysqliConn {
    conn: Conn,
    charset: String,
    /// Result sets after the current one (multi_query): payload arrays ready
    /// to hand to the prelude, popped by `__mysqli_next_result`.
    pending: VecDeque<Zval>,
}

/// One server-side prepared statement (`mysqli_stmt`), tied to its connection.
pub(super) struct MysqliStmt {
    conn_id: u32,
    stmt: mysql::Statement,
}

/// Charsets accepted by `mysqli_set_charset` (client-side check: an unknown
/// name fails with client errno 2019 before touching the server).
const KNOWN_CHARSETS: &[&str] = &[
    "big5", "dec8", "cp850", "hp8", "koi8r", "latin1", "latin2", "swe7", "ascii", "ujis", "sjis",
    "hebrew", "tis620", "euckr", "koi8u", "gb2312", "greek", "cp1250", "gbk", "latin5", "armscii8",
    "utf8", "utf8mb3", "ucs2", "cp866", "keybcs2", "macce", "macroman", "cp852", "latin7",
    "utf8mb4", "cp1251", "utf16", "utf16le", "cp1256", "cp1257", "utf32", "binary", "geostd8",
    "cp932", "eucjpms", "gb18030",
];

fn zstr(s: impl AsRef<[u8]>) -> Zval {
    Zval::Str(PhpStr::new(s.as_ref().to_vec()))
}

fn put(arr: &mut PhpArray, key: &str, v: Zval) {
    arr.insert(Key::from_bytes(key.as_bytes()), v);
}

/// The `t=err` payload: client- or server-reported errno/message/sqlstate.
fn err_payload(errno: i64, error: &str, sqlstate: &str) -> Zval {
    let mut a = PhpArray::new();
    put(&mut a, "t", zstr("err"));
    put(&mut a, "errno", Zval::Long(errno));
    put(&mut a, "error", zstr(error));
    put(&mut a, "sqlstate", zstr(sqlstate));
    Zval::Array(std::rc::Rc::new(a))
}

/// Map a `mysql::Error` to the (errno, message, sqlstate) triple mysqli
/// reports. Server errors carry all three verbatim; transport errors get the
/// classic client codes (2002/2006/2013).
fn error_triple(e: &mysql::Error) -> (i64, String, String) {
    match e {
        mysql::Error::MySqlError(me) => {
            (i64::from(me.code), me.message.clone(), me.state.clone())
        }
        mysql::Error::IoError(io) => match io.kind() {
            std::io::ErrorKind::ConnectionRefused => {
                (2002, "Connection refused".to_string(), "HY000".to_string())
            }
            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe => {
                (2006, "MySQL server has gone away".to_string(), "HY000".to_string())
            }
            std::io::ErrorKind::TimedOut => (
                2002,
                "Connection timed out".to_string(),
                "HY000".to_string(),
            ),
            _ => (2002, io.to_string(), "HY000".to_string()),
        },
        mysql::Error::DriverError(de) => {
            let msg = de.to_string();
            if msg.contains("Connection refused") {
                (2002, "Connection refused".to_string(), "HY000".to_string())
            } else if msg.contains("timed out") {
                (2002, "Connection timed out".to_string(), "HY000".to_string())
            } else {
                (2000, msg, "HY000".to_string())
            }
        }
        other => (2000, other.to_string(), "HY000".to_string()),
    }
}

fn err_from(e: &mysql::Error) -> Zval {
    let (errno, msg, state) = error_triple(e);
    err_payload(errno, &msg, &state)
}

/// A binary-protocol cell as mysqli/mysqlnd reports it: native ints/floats,
/// temporal values re-serialized to MySQL's canonical text form. (The text
/// protocol never reaches this: its cells arrive as `Value::Bytes` already.)
fn value_to_zval(v: Value) -> Zval {
    match v {
        Value::NULL => Zval::Null,
        Value::Bytes(b) => Zval::Str(PhpStr::new(b)),
        Value::Int(i) => Zval::Long(i),
        Value::UInt(u) => {
            if let Ok(i) = i64::try_from(u) {
                Zval::Long(i)
            } else {
                zstr(u.to_string())
            }
        }
        Value::Float(f) => Zval::Double(f64::from(f)),
        Value::Double(d) => Zval::Double(d),
        Value::Date(y, mo, d, h, mi, s, us) => {
            let mut out = format!("{y:04}-{mo:02}-{d:02}");
            if h != 0 || mi != 0 || s != 0 || us != 0 || true {
                // DATETIME/TIMESTAMP columns always carry the time part; a
                // plain DATE arrives with h=m=s=us=0 AND a DATE column type,
                // which the crate encodes as Date(y,m,d,0,0,0,0) too — the
                // prelude trims by column type where it matters (none of the
                // WP surface does).
                out.push_str(&format!(" {h:02}:{mi:02}:{s:02}"));
                if us != 0 {
                    out.push_str(&format!(".{us:06}"));
                }
            }
            zstr(out)
        }
        Value::Time(neg, days, h, mi, s, us) => {
            let hours = u32::from(h) + days * 24;
            let sign = if neg { "-" } else { "" };
            let mut out = format!("{sign}{hours:02}:{mi:02}:{s:02}");
            if us != 0 {
                out.push_str(&format!(".{us:06}"));
            }
            zstr(out)
        }
    }
}

/// One column's metadata as `mysqli_result::fetch_field` exposes it (the
/// stdClass the prelude builds from this assoc array). `max_length` is the
/// constant 0 of PHP ≥ 8.1 mysqlnd; `def`/`catalog` are the fixed ""/"def".
fn column_to_zval(c: &Column) -> Zval {
    let mut a = PhpArray::new();
    put(&mut a, "name", zstr(c.name_ref()));
    put(&mut a, "orgname", zstr(c.org_name_ref()));
    put(&mut a, "table", zstr(c.table_ref()));
    put(&mut a, "orgtable", zstr(c.org_table_ref()));
    put(&mut a, "def", zstr(""));
    put(&mut a, "db", zstr(c.schema_ref()));
    put(&mut a, "catalog", zstr("def"));
    put(&mut a, "max_length", Zval::Long(0));
    put(&mut a, "length", Zval::Long(i64::from(c.column_length())));
    put(&mut a, "charsetnr", Zval::Long(i64::from(c.character_set())));
    // mysqlnd sets NUM_FLAG (32768) client-side for the types it fetches
    // numerically (ints/floats/year/bit — NOT decimal/timestamp, per oracle);
    // the crate's ColumnFlags bitmask also drops the bit if the server sent it.
    let mut flags = i64::from(c.flags().bits());
    if matches!(c.column_type() as u8, 1..=5 | 8 | 9 | 13 | 16) {
        flags |= 32768;
    }
    put(&mut a, "flags", Zval::Long(flags));
    put(&mut a, "type", Zval::Long(i64::from(c.column_type() as u8)));
    put(&mut a, "decimals", Zval::Long(i64::from(c.decimals())));
    Zval::Array(std::rc::Rc::new(a))
}

/// Drain every result set of an executed query into payload arrays (first set
/// returned, the rest queued for more_results/next_result). Fully buffered:
/// mysqli's default MYSQLI_STORE_RESULT is exactly this.
fn drain_sets(
    mut qr: mysql::QueryResult<'_, '_, '_, mysql::Text>,
) -> Result<(Zval, VecDeque<Zval>), mysql::Error> {
    let mut sets: VecDeque<Zval> = VecDeque::new();
    while let Some(set) = qr.iter() {
        sets.push_back(set_to_payload(set)?);
    }
    let first = sets.pop_front().unwrap_or_else(|| {
        // No set at all (shouldn't happen: even a status query yields one).
        err_payload(2000, "no result", "HY000")
    });
    Ok((first, sets))
}

/// One result set → `t=res` (columns present) or `t=ok` (status) payload.
fn set_to_payload(
    mut set: mysql::ResultSet<'_, '_, '_, '_, mysql::Text>,
) -> Result<Zval, mysql::Error> {
    let cols = set.columns();
    let cols: &[Column] = cols.as_ref();
    let mut a = PhpArray::new();
    if cols.is_empty() {
        put(&mut a, "t", zstr("ok"));
        put(&mut a, "affected", Zval::Long(set.affected_rows() as i64));
        put(
            &mut a,
            "insert_id",
            Zval::Long(set.last_insert_id().unwrap_or(0) as i64),
        );
        put(&mut a, "warnings", Zval::Long(i64::from(set.warnings())));
        match set.info_str() {
            s if s.is_empty() => put(&mut a, "info", Zval::Null),
            s => {
                let owned = s.into_owned();
                put(&mut a, "info", zstr(owned));
            }
        }
        // Drive the (empty) row iterator to completion so the protocol state
        // advances past this set.
        for row in set.by_ref() {
            let _ = row?;
        }
        return Ok(Zval::Array(std::rc::Rc::new(a)));
    }
    let mut fields = PhpArray::new();
    for c in cols {
        let _ = fields.append(column_to_zval(c));
    }
    put(&mut a, "t", zstr("res"));
    put(&mut a, "fields", Zval::Array(std::rc::Rc::new(fields)));
    let mut rows = PhpArray::new();
    for row in set.by_ref() {
        let row = row?;
        let mut r = PhpArray::new();
        for v in row.unwrap() {
            let _ = r.append(value_to_zval(v));
        }
        let _ = rows.append(Zval::Array(std::rc::Rc::new(r)));
    }
    put(&mut a, "rows", Zval::Array(std::rc::Rc::new(rows)));
    put(&mut a, "warnings", Zval::Long(0));
    Ok(Zval::Array(std::rc::Rc::new(a)))
}

/// mysqli_real_escape_string's byte map (charset-safe for the ASCII-supersets
/// WP uses; identical to libmysql's escaping in the default sql mode).
fn escape_bytes(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() + 8);
    for &b in input {
        match b {
            0 => out.extend_from_slice(b"\\0"),
            b'\n' => out.extend_from_slice(b"\\n"),
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\\' => out.extend_from_slice(b"\\\\"),
            b'\'' => out.extend_from_slice(b"\\'"),
            b'"' => out.extend_from_slice(b"\\\""),
            0x1a => out.extend_from_slice(b"\\Z"),
            _ => out.push(b),
        }
    }
    out
}

impl<'m> Vm<'m> {
    fn mysqli_conn(&mut self, args: &[Zval]) -> Result<&mut MysqliConn, Zval> {
        let id = match args.first() {
            Some(v) => convert::to_long_cast(v, &mut self.diags) as u32,
            None => 0,
        };
        self.mysqli_conns
            .get_mut(&id)
            .ok_or_else(|| err_payload(2006, "MySQL server has gone away", "HY000"))
    }

    /// `__mysqli_connect($host, $user, $pass, $db|null, $port|null, $socket|null)`
    /// (the prelude `mysqli::real_connect` backing). Success: `t=ok` payload
    /// with the handle and connection facts; failure: `t=err` with the
    /// client-format triple (server message verbatim for auth/db errors).
    pub(super) fn ho_mysqli_connect(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let get = |i: usize| args.get(i).map(Zval::deref_clone).unwrap_or(Zval::Null);
        let host_z = get(0);
        let host = match &host_z {
            Zval::Null => "localhost".to_string(),
            v => String::from_utf8_lossy(convert::to_zstr_cast(v, &mut self.diags).as_bytes())
                .into_owned(),
        };
        let user = String::from_utf8_lossy(
            convert::to_zstr_cast(&get(1), &mut self.diags).as_bytes(),
        )
        .into_owned();
        let pass = String::from_utf8_lossy(
            convert::to_zstr_cast(&get(2), &mut self.diags).as_bytes(),
        )
        .into_owned();
        let db = match get(3) {
            Zval::Null => None,
            v => {
                let s = convert::to_zstr_cast(&v, &mut self.diags);
                let s = String::from_utf8_lossy(s.as_bytes()).into_owned();
                if s.is_empty() { None } else { Some(s) }
            }
        };
        let port = match get(4) {
            Zval::Null => 3306u16,
            v => convert::to_long_cast(&v, &mut self.diags) as u16,
        };
        let socket = match get(5) {
            Zval::Null => None,
            v => {
                let s = convert::to_zstr_cast(&v, &mut self.diags);
                let s = String::from_utf8_lossy(s.as_bytes()).into_owned();
                if s.is_empty() { None } else { Some(s) }
            }
        };
        // "localhost" without an explicit socket goes over TCP here (the
        // crate would try the default socket path): matches WP's docker-era
        // configs which always use TCP hosts; a socket path is honoured.
        let use_socket = socket.is_some();
        let mut builder = OptsBuilder::new()
            .user(Some(user))
            .pass(Some(pass))
            .db_name(db)
            .prefer_socket(false);
        if let Some(sock) = &socket {
            builder = builder.socket(Some(sock.clone()));
        } else {
            let tcp_host = if host == "localhost" { "127.0.0.1".to_string() } else { host.clone() };
            builder = builder.ip_or_hostname(Some(tcp_host)).tcp_port(port);
        }
        match Conn::new(builder) {
            Ok(mut conn) => {
                // mysqlnd's handshake charset is utf8mb4 with the *server's*
                // default collation (utf8mb4_0900_ai_ci on 8.0+); the crate
                // hands utf8mb4_general_ci to the server instead, which would
                // tag result metadata with charsetnr 45 instead of 255.
                let _ = conn.query_drop("SET NAMES 'utf8mb4'");
                let id = self.next_mysqli;
                self.next_mysqli += 1;
                let (maj, min, patch) = conn.server_version();
                let thread_id = conn.connection_id();
                let mut a = PhpArray::new();
                put(&mut a, "t", zstr("ok"));
                put(&mut a, "h", Zval::Long(i64::from(id)));
                put(&mut a, "server_info", zstr(format!("{maj}.{min}.{patch}")));
                put(
                    &mut a,
                    "server_version",
                    Zval::Long(i64::from(maj) * 10000 + i64::from(min) * 100 + i64::from(patch)),
                );
                put(
                    &mut a,
                    "host_info",
                    zstr(if use_socket {
                        "Localhost via UNIX socket".to_string()
                    } else {
                        format!("{host} via TCP/IP")
                    }),
                );
                put(&mut a, "protocol", Zval::Long(10));
                put(&mut a, "thread_id", Zval::Long(i64::from(thread_id)));
                self.mysqli_conns.insert(
                    id,
                    MysqliConn { conn, charset: "utf8mb4".to_string(), pending: VecDeque::new() },
                );
                Ok(Zval::Array(std::rc::Rc::new(a)))
            }
            Err(e) => {
                // Connect-time errors report the client-side HY000 sqlstate
                // (CR convention) whatever the server said — oracle: a 1045
                // exception's getSqlState() is HY000, not 28000.
                let (errno, msg, _) = error_triple(&e);
                Ok(err_payload(errno, &msg, "HY000"))
            }
        }
    }

    /// `__mysqli_close($h)`: drop the connection (and any statement on it).
    pub(super) fn ho_mysqli_close(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let id = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        self.mysqli_stmts.retain(|_, s| s.conn_id != id);
        Ok(Zval::Bool(self.mysqli_conns.remove(&id).is_some()))
    }

    /// `__mysqli_query($h, $sql)`: run one statement, fully buffer its result
    /// sets; the first set's payload is returned, the rest queue for
    /// more_results/next_result (CALL with result sets).
    pub(super) fn ho_mysqli_query(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        mc.pending.clear();
        match mc.conn.query_iter(String::from_utf8_lossy(&sql).into_owned()) {
            Ok(qr) => match drain_sets(qr) {
                Ok((first, rest)) => {
                    mc.pending = rest;
                    Ok(first)
                }
                Err(e) => Ok(err_from(&e)),
            },
            Err(e) => Ok(err_from(&e)),
        }
    }

    /// `__mysqli_more_results($h)` / `__mysqli_next_result($h)`: the buffered
    /// tail of the last (multi_)query. next pops the following set's payload
    /// (the prelude makes it the current result); false-shaped Null when done.
    pub(super) fn ho_mysqli_more_results(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(_) => return Ok(Zval::Bool(false)),
        };
        Ok(Zval::Bool(!mc.pending.is_empty()))
    }

    pub(super) fn ho_mysqli_next_result(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(_) => return Ok(Zval::Null),
        };
        Ok(mc.pending.pop_front().unwrap_or(Zval::Null))
    }

    /// `__mysqli_select_db($h, $db)`: true | err payload.
    pub(super) fn ho_mysqli_select_db(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let db = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        let db = String::from_utf8_lossy(&db).into_owned();
        match mc.conn.select_db(&db) {
            Ok(()) => Ok(Zval::Bool(true)),
            Err(e) => Ok(err_from(&e)),
        }
    }

    /// `__mysqli_set_charset($h, $cs)`: client-side name check (errno 2019),
    /// then `SET NAMES` and remember the name for character_set_name().
    pub(super) fn ho_mysqli_set_charset(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let cs = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let cs = String::from_utf8_lossy(&cs).into_owned();
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        if !KNOWN_CHARSETS.contains(&cs.as_str()) {
            return Ok(err_payload(
                2019,
                &format!("Can't initialize character set {cs} (path: compiled_in)"),
                "HY000",
            ));
        }
        match mc.conn.query_drop(format!("SET NAMES '{cs}'")) {
            Ok(()) => {
                mc.charset = cs;
                Ok(Zval::Bool(true))
            }
            Err(e) => Ok(err_from(&e)),
        }
    }

    /// `__mysqli_charset($h)`: the connection's current charset name.
    pub(super) fn ho_mysqli_charset(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        Ok(zstr(mc.charset.clone()))
    }

    /// `__mysqli_escape($h, $s)`: mysqli_real_escape_string.
    pub(super) fn ho_mysqli_escape(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let s = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        if self.mysqli_conn(&args).is_err() {
            return Ok(Zval::Null);
        }
        Ok(Zval::Str(PhpStr::new(escape_bytes(&s))))
    }

    /// `__mysqli_ping($h)`: a lightweight round-trip on the wire.
    pub(super) fn ho_mysqli_ping(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(_) => return Ok(Zval::Bool(false)),
        };
        Ok(Zval::Bool(mc.conn.ping().is_ok()))
    }

    /// `__mysqli_stat($h)`: the COM_STATISTICS-shaped one-liner. Assembled
    /// from status variables (the crate exposes no raw COM_STATISTICS).
    pub(super) fn ho_mysqli_stat(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(_) => return Ok(Zval::Bool(false)),
        };
        let one = |conn: &mut Conn, q: &str| -> Option<String> {
            conn.query_first::<(String, String), _>(q).ok().flatten().map(|(_, v)| v)
        };
        let uptime = one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Uptime'").unwrap_or_default();
        let threads =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Threads_connected'").unwrap_or_default();
        let questions =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Questions'").unwrap_or_default();
        let slow =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Slow_queries'").unwrap_or_default();
        let opens =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Opened_tables'").unwrap_or_default();
        let flush =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Flush_commands'").unwrap_or_default();
        let open =
            one(&mut mc.conn, "SHOW GLOBAL STATUS LIKE 'Open_tables'").unwrap_or_default();
        Ok(zstr(format!(
            "Uptime: {uptime}  Threads: {threads}  Questions: {questions}  Slow queries: {slow}  Opens: {opens}  Flush tables: {flush}  Open tables: {open}  Queries per second avg: 0.000"
        )))
    }

    /// `__mysqli_prepare($h, $sql)`: server-side prepare. Success:
    /// `['t'=>'ok','h'=>sid,'params'=>n,'fields'=>[…]]`; failure: err payload.
    pub(super) fn ho_mysqli_prepare(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let conn_id =
            convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        match mc.conn.prep(String::from_utf8_lossy(&sql).into_owned()) {
            Ok(stmt) => {
                let sid = self.next_mysqli;
                self.next_mysqli += 1;
                let mut a = PhpArray::new();
                put(&mut a, "t", zstr("ok"));
                put(&mut a, "h", Zval::Long(i64::from(sid)));
                put(&mut a, "params", Zval::Long(i64::from(stmt.num_params())));
                let mut fields = PhpArray::new();
                for c in stmt.columns().as_ref() {
                    let _ = fields.append(column_to_zval(c));
                }
                put(&mut a, "fields", Zval::Array(std::rc::Rc::new(fields)));
                self.mysqli_stmts.insert(sid, MysqliStmt { conn_id, stmt });
                Ok(Zval::Array(std::rc::Rc::new(a)))
            }
            Err(e) => Ok(err_from(&e)),
        }
    }

    /// `__mysqli_stmt_execute($sid, $values)`: run a prepared statement with
    /// the (already PHP-side-coerced) positional values. Result payload as
    /// `__mysqli_query`, with binary-protocol native types in the rows.
    pub(super) fn ho_mysqli_stmt_execute(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let sid = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        let vals = args.get(1).map(Zval::deref_clone).unwrap_or(Zval::Null);
        let mut params: Vec<Value> = Vec::new();
        if let Zval::Array(a) = &vals {
            for (_, v) in a.iter() {
                params.push(match v.deref_clone() {
                    Zval::Null => Value::NULL,
                    Zval::Bool(b) => Value::Int(i64::from(b)),
                    Zval::Long(i) => Value::Int(i),
                    Zval::Double(d) => Value::Double(d),
                    other => {
                        let s = convert::to_zstr_cast(&other, &mut self.diags);
                        Value::Bytes(s.as_bytes().to_vec())
                    }
                });
            }
        }
        let Some(ms) = self.mysqli_stmts.get(&sid) else {
            return Ok(err_payload(2006, "MySQL server has gone away", "HY000"));
        };
        let stmt = ms.stmt.clone();
        let conn_id = ms.conn_id;
        let Some(mc) = self.mysqli_conns.get_mut(&conn_id) else {
            return Ok(err_payload(2006, "MySQL server has gone away", "HY000"));
        };
        let p = if params.is_empty() {
            mysql::Params::Empty
        } else {
            mysql::Params::Positional(params)
        };
        match mc.conn.exec_iter(&stmt, p) {
            Ok(qr) => {
                // Binary-protocol drain: same shape as the text path.
                let mut sets: VecDeque<Zval> = VecDeque::new();
                let mut qr = qr;
                while let Some(set) = qr.iter() {
                    match set_to_payload_bin(set) {
                        Ok(z) => sets.push_back(z),
                        Err(e) => return Ok(err_from(&e)),
                    }
                }
                Ok(sets
                    .pop_front()
                    .unwrap_or_else(|| err_payload(2000, "no result", "HY000")))
            }
            Err(e) => Ok(err_from(&e)),
        }
    }

    /// `__mysqli_multi_query($h, $sql)`: mysqli_multi_query. The server-side
    /// multi-statement capability is emulated by splitting the SQL client-side
    /// (quotes/comments respected) and running the pieces sequentially —
    /// observably equivalent: the real server also stops at the first failing
    /// statement, and result sets surface one per next_result(). Returns the
    /// first statement's payload; the rest (or the terminating error) queue.
    pub(super) fn ho_mysqli_multi_query(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let sql = convert::to_zstr_cast(args.get(1).unwrap_or(&Zval::Null), &mut self.diags)
            .as_bytes()
            .to_vec();
        let mc = match self.mysqli_conn(&args) {
            Ok(mc) => mc,
            Err(e) => return Ok(e),
        };
        mc.pending.clear();
        let mut payloads: VecDeque<Zval> = VecDeque::new();
        for piece in split_statements(&sql) {
            let piece = String::from_utf8_lossy(&piece).into_owned();
            if piece.trim().is_empty() {
                continue;
            }
            match mc.conn.query_iter(piece) {
                Ok(qr) => match drain_sets(qr) {
                    Ok((first, rest)) => {
                        payloads.push_back(first);
                        payloads.extend(rest);
                    }
                    Err(e) => {
                        payloads.push_back(err_from(&e));
                        break;
                    }
                },
                Err(e) => {
                    payloads.push_back(err_from(&e));
                    break;
                }
            }
        }
        let first = payloads
            .pop_front()
            .unwrap_or_else(|| err_payload(1065, "Query was empty", "42000"));
        mc.pending = payloads;
        Ok(first)
    }

    /// `__mysqli_stmt_close($sid)`.
    pub(super) fn ho_mysqli_stmt_close(&mut self, args: Vec<Zval>) -> Result<Zval, PhpError> {
        let sid = convert::to_long_cast(args.first().unwrap_or(&Zval::Null), &mut self.diags) as u32;
        Ok(Zval::Bool(self.mysqli_stmts.remove(&sid).is_some()))
    }
}

/// Split a multi-statement SQL string on top-level `;` — string literals
/// (`'`/`"` with `\` escapes and doubled quotes), backtick identifiers and
/// comments (`--` + space, `#`, `/* */`) are respected.
fn split_statements(sql: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut i = 0;
    #[derive(PartialEq)]
    enum St {
        Plain,
        Squote,
        Dquote,
        Btick,
        LineComment,
        BlockComment,
    }
    let mut st = St::Plain;
    while i < sql.len() {
        let b = sql[i];
        match st {
            St::Plain => match b {
                b';' => {
                    out.push(std::mem::take(&mut cur));
                    i += 1;
                    continue;
                }
                b'\'' => st = St::Squote,
                b'"' => st = St::Dquote,
                b'`' => st = St::Btick,
                b'#' => st = St::LineComment,
                b'-' if sql.get(i + 1) == Some(&b'-')
                    && matches!(sql.get(i + 2), Some(b' ' | b'\t' | b'\n') | None) =>
                {
                    st = St::LineComment
                }
                b'/' if sql.get(i + 1) == Some(&b'*') => st = St::BlockComment,
                _ => {}
            },
            St::Squote => match b {
                b'\\' if i + 1 < sql.len() => {
                    cur.push(b);
                    i += 1;
                    cur.push(sql[i]);
                    i += 1;
                    continue;
                }
                b'\'' => st = St::Plain,
                _ => {}
            },
            St::Dquote => match b {
                b'\\' if i + 1 < sql.len() => {
                    cur.push(b);
                    i += 1;
                    cur.push(sql[i]);
                    i += 1;
                    continue;
                }
                b'"' => st = St::Plain,
                _ => {}
            },
            St::Btick => {
                if b == b'`' {
                    st = St::Plain;
                }
            }
            St::LineComment => {
                if b == b'\n' {
                    st = St::Plain;
                }
            }
            St::BlockComment => {
                if b == b'*' && sql.get(i + 1) == Some(&b'/') {
                    cur.push(b);
                    i += 1;
                    cur.push(sql[i]);
                    i += 1;
                    st = St::Plain;
                    continue;
                }
            }
        }
        cur.push(b);
        i += 1;
    }
    if !cur.iter().all(|b| b.is_ascii_whitespace()) {
        out.push(cur);
    }
    out
}

/// Binary-protocol result set → payload (prepared-statement path).
fn set_to_payload_bin(
    mut set: mysql::ResultSet<'_, '_, '_, '_, mysql::Binary>,
) -> Result<Zval, mysql::Error> {
    let cols = set.columns();
    let cols: &[Column] = cols.as_ref();
    let mut a = PhpArray::new();
    if cols.is_empty() {
        put(&mut a, "t", zstr("ok"));
        put(&mut a, "affected", Zval::Long(set.affected_rows() as i64));
        put(
            &mut a,
            "insert_id",
            Zval::Long(set.last_insert_id().unwrap_or(0) as i64),
        );
        put(&mut a, "warnings", Zval::Long(i64::from(set.warnings())));
        put(&mut a, "info", Zval::Null);
        for row in set.by_ref() {
            let _ = row?;
        }
        return Ok(Zval::Array(std::rc::Rc::new(a)));
    }
    let mut fields = PhpArray::new();
    for c in cols {
        let _ = fields.append(column_to_zval(c));
    }
    put(&mut a, "t", zstr("res"));
    put(&mut a, "fields", Zval::Array(std::rc::Rc::new(fields)));
    let mut rows = PhpArray::new();
    for row in set.by_ref() {
        let row = row?;
        let mut r = PhpArray::new();
        for v in row.unwrap() {
            let _ = r.append(value_to_zval(v));
        }
        let _ = rows.append(Zval::Array(std::rc::Rc::new(r)));
    }
    put(&mut a, "rows", Zval::Array(std::rc::Rc::new(rows)));
    put(&mut a, "warnings", Zval::Long(0));
    Ok(Zval::Array(std::rc::Rc::new(a)))
}
