// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
// ext/sqlite3 sul medesimo backing rusqlite dei __pdo_* (stessa registry di
// connessioni): SQLite3Stmt tiene SQL+parametri e ri-prepara a ogni execute,
// SQLite3Result e' il rowset materializzato. Quirk oracle-verificati: exec
// ritorna true (non changes); fetchArray oltre la fine ritorna false E
// RESETTA il cursore (sqlite3_step dopo DONE auto-resetta); BOTH e' in ordine
// NUM-poi-nome per colonna (inverso di PDO); columnType riflette la riga
// APPENA fetchata (false prima del primo fetch e dopo il false di fine);
// bindValue senza tipo inferisce dal tipo PHP (bool->int); il ctor lancia
// \Exception anche con exceptions OFF, gli errori runtime SQLite3Exception
// solo con enableExceptions(true), altrimenti warning + false.
class SQLite3Exception extends Exception {}
class SQLite3 {
    private $__h = null;
    private $__throw = false;
    private $__err = array(0, '');
    public function __construct($filename = '', $flags = 6, $encryptionKey = '') {
        $this->open($filename, $flags, $encryptionKey);
    }
    public function open($filename, $flags = 6, $encryptionKey = '') {
        $r = __pdo_open('sqlite:' . $filename);
        if (is_array($r)) {
            throw new Exception('Unable to open database: ' . ($r[3] !== null ? $r[3] : $r[0]));
        }
        $this->__h = $r;
    }
    public function enableExceptions($enable = false) {
        $old = $this->__throw;
        $this->__throw = (bool)$enable;
        return $old;
    }
    // Error sink: record lastErrorCode/Msg, throw or warn per enableExceptions.
    public function __fail($code, $msg, $full, $fn) {
        $this->__err = array($code, $msg);
        if ($this->__throw) { throw new SQLite3Exception($full, $code); }
        trigger_error($fn . '(): ' . $full, E_USER_WARNING);
        return false;
    }
    public function exec($query) {
        $r = __pdo_exec($this->__h, (string)$query);
        if (isset($r['err'])) { $e = $r['err']; return $this->__fail($e[2], $e[3], $e[3], 'SQLite3::exec'); }
        return true;
    }
    public function query($query) {
        $r = __pdo_run($this->__h, (string)$query, array(), false);
        if (isset($r['err'])) { $e = $r['err']; return $this->__fail($e[2], $e[3], $e[3], 'SQLite3::query'); }
        $res = new SQLite3Result();
        $res->__init(isset($r['cols']) ? $r['cols'] : array(), isset($r['rows']) ? $r['rows'] : array());
        return $res;
    }
    public function querySingle($query, $entireRow = false) {
        $res = $this->query($query);
        if ($res === false) { return false; }
        $row = $res->fetchArray($entireRow ? SQLITE3_ASSOC : SQLITE3_NUM);
        if ($row === false) { return $entireRow ? array() : null; }
        return $entireRow ? $row : $row[0];
    }
    public function prepare($query) {
        $r = __pdo_prepare($this->__h, (string)$query);
        if (is_array($r) && isset($r['err'])) {
            $e = $r['err'];
            return $this->__fail($e[2], $e[3], 'Unable to prepare statement: ' . $e[3], 'SQLite3::prepare');
        }
        $st = new SQLite3Stmt();
        $st->__init($this, $this->__h, (string)$query);
        return $st;
    }
    public function changes() { return __pdo_changes($this->__h); }
    public function lastInsertRowID() { return __pdo_last_id($this->__h); }
    public function lastErrorCode() { return $this->__err[0]; }
    public function lastErrorMsg() { return $this->__err[1]; }
    public function busyTimeout($milliseconds) { return true; }
    public function close() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
        return true;
    }
    public function __destruct() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
    }
    public static function escapeString($string) { return str_replace("'", "''", (string)$string); }
    public static function version() {
        $s = __pdo_sqlite_version();
        $p = explode('.', $s);
        $n = (int)$p[0] * 1000000 + (isset($p[1]) ? (int)$p[1] * 1000 : 0) + (isset($p[2]) ? (int)$p[2] : 0);
        return array('versionString' => $s, 'versionNumber' => $n);
    }
}
class SQLite3Stmt {
    private $__db = null;
    private $__c = null;
    private $__sql = '';
    private $__bound = array();
    public function __init($db, $h, $sql) {
        $this->__db = $db;
        $this->__c = $h;
        $this->__sql = $sql;
    }
    private function __coerce($v, $t) {
        if ($t === null) {
            // Senza tipo dichiarato binda per tipo PHP (bool -> int).
            if (is_bool($v)) { return (int)$v; }
            return $v;
        }
        if ($v === null || $t === SQLITE3_NULL) { return null; }
        if ($t === SQLITE3_INTEGER) { return (int)$v; }
        if ($t === SQLITE3_FLOAT) { return (float)$v; }
        return (string)$v; // TEXT/BLOB
    }
    public function bindValue($param, $value, $type = null) {
        $this->__bound[$param] = array($this->__coerce($value, $type));
        return true;
    }
    public function bindParam($param, &$var, $type = null) {
        $this->__bound[$param] = array(null, $type, true, null);
        $this->__bound[$param][3] =& $var;
        return true;
    }
    public function execute() {
        $send = array();
        foreach ($this->__bound as $k => $b) {
            $send[$k] = isset($b[2]) && $b[2] ? $this->__coerce($b[3], $b[1]) : $b[0];
        }
        $r = __pdo_run($this->__c, $this->__sql, $send, false);
        if (isset($r['err'])) {
            $e = $r['err'];
            return $this->__db->__fail($e[2], $e[3], 'Unable to execute statement: ' . $e[3], 'SQLite3Stmt::execute');
        }
        $res = new SQLite3Result();
        $res->__init(isset($r['cols']) ? $r['cols'] : array(), isset($r['rows']) ? $r['rows'] : array());
        return $res;
    }
    public function paramCount() { return __pdo_param_count($this->__c, $this->__sql); }
    public function readOnly() { return __pdo_stmt_readonly($this->__c, $this->__sql); }
    public function getSQL($expand = false) { return $this->__sql; }
    public function reset() { return true; }
    public function clear() { $this->__bound = array(); return true; }
    public function close() { return true; }
}
class SQLite3Result {
    private $__cols = array();
    private $__rows = array();
    private $__pos = 0;
    public function __init($cols, $rows) {
        $this->__cols = $cols;
        $this->__rows = $rows;
        $this->__pos = 0;
    }
    public function fetchArray($mode = SQLITE3_BOTH) {
        if ($this->__pos >= count($this->__rows)) {
            // sqlite3_step dopo DONE auto-resetta: il fetch successivo riparte.
            $this->__pos = 0;
            return false;
        }
        $row = $this->__rows[$this->__pos];
        $this->__pos = $this->__pos + 1;
        $out = array();
        foreach ($row as $i => $v) {
            if (($mode & SQLITE3_NUM) === SQLITE3_NUM) { $out[$i] = $v; }
            if (($mode & SQLITE3_ASSOC) === SQLITE3_ASSOC) { $out[$this->__cols[$i]] = $v; }
        }
        return $out;
    }
    public function numColumns() { return count($this->__cols); }
    public function columnName($column) {
        return isset($this->__cols[$column]) ? $this->__cols[$column] : false;
    }
    public function columnType($column) {
        // Tipo del valore nella riga APPENA fetchata; false prima del primo
        // fetch e dopo il false di fine (pos resettata a 0).
        if ($this->__pos < 1 || !isset($this->__cols[$column])) { return false; }
        $v = $this->__rows[$this->__pos - 1][$column];
        if (is_int($v)) { return SQLITE3_INTEGER; }
        if (is_float($v)) { return SQLITE3_FLOAT; }
        if ($v === null) { return SQLITE3_NULL; }
        return SQLITE3_TEXT;
    }
    public function finalize() { return true; }
    public function reset() { $this->__pos = 0; return true; }
}
