// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
// ext/pdo + pdo_sqlite (sqlite driver only, backing doctrine/dbal): the prelude
// classes delegate to the __pdo_* host builtins (rusqlite VM-side, vm/pdo.rs);
// the handle is an int id in Vm.pdo_conns. A failing host op returns the array
// [message, code, sqlstate|null, native-msg|null] and the PHP side raises the
// PDOException. Constants are the oracle's full PDO constant table (8.5.7).
// No custom ctor, like the C class: internal raises pass SQLSTATE-string codes
// through the (prelude-untyped) Exception ctor and set the public errorInfo
// from the raising side.
class PDOException extends RuntimeException {
    public $errorInfo = null;
}
class PDO {
    const PARAM_NULL = 0;
    const PARAM_BOOL = 5;
    const PARAM_INT = 1;
    const PARAM_STR = 2;
    const PARAM_LOB = 3;
    const PARAM_STMT = 4;
    const PARAM_INPUT_OUTPUT = 2147483648;
    const PARAM_STR_NATL = 1073741824;
    const PARAM_STR_CHAR = 536870912;
    const PARAM_EVT_ALLOC = 0;
    const PARAM_EVT_FREE = 1;
    const PARAM_EVT_EXEC_PRE = 2;
    const PARAM_EVT_EXEC_POST = 3;
    const PARAM_EVT_FETCH_PRE = 4;
    const PARAM_EVT_FETCH_POST = 5;
    const PARAM_EVT_NORMALIZE = 6;
    const FETCH_DEFAULT = 0;
    const FETCH_LAZY = 1;
    const FETCH_ASSOC = 2;
    const FETCH_NUM = 3;
    const FETCH_BOTH = 4;
    const FETCH_OBJ = 5;
    const FETCH_BOUND = 6;
    const FETCH_COLUMN = 7;
    const FETCH_CLASS = 8;
    const FETCH_INTO = 9;
    const FETCH_FUNC = 10;
    const FETCH_GROUP = 32;
    const FETCH_UNIQUE = 64;
    const FETCH_KEY_PAIR = 12;
    const FETCH_CLASSTYPE = 128;
    const FETCH_SERIALIZE = 512;
    const FETCH_PROPS_LATE = 256;
    const FETCH_NAMED = 11;
    const ATTR_AUTOCOMMIT = 0;
    const ATTR_PREFETCH = 1;
    const ATTR_TIMEOUT = 2;
    const ATTR_ERRMODE = 3;
    const ATTR_SERVER_VERSION = 4;
    const ATTR_CLIENT_VERSION = 5;
    const ATTR_SERVER_INFO = 6;
    const ATTR_CONNECTION_STATUS = 7;
    const ATTR_CASE = 8;
    const ATTR_CURSOR_NAME = 9;
    const ATTR_CURSOR = 10;
    const ATTR_ORACLE_NULLS = 11;
    const ATTR_PERSISTENT = 12;
    const ATTR_STATEMENT_CLASS = 13;
    const ATTR_FETCH_TABLE_NAMES = 14;
    const ATTR_FETCH_CATALOG_NAMES = 15;
    const ATTR_DRIVER_NAME = 16;
    const ATTR_STRINGIFY_FETCHES = 17;
    const ATTR_MAX_COLUMN_LEN = 18;
    const ATTR_EMULATE_PREPARES = 20;
    const ATTR_DEFAULT_FETCH_MODE = 19;
    const ATTR_DEFAULT_STR_PARAM = 21;
    const ERRMODE_SILENT = 0;
    const ERRMODE_WARNING = 1;
    const ERRMODE_EXCEPTION = 2;
    const CASE_NATURAL = 0;
    const CASE_LOWER = 2;
    const CASE_UPPER = 1;
    const NULL_NATURAL = 0;
    const NULL_EMPTY_STRING = 1;
    const NULL_TO_STRING = 2;
    const ERR_NONE = '00000';
    const FETCH_ORI_NEXT = 0;
    const FETCH_ORI_PRIOR = 1;
    const FETCH_ORI_FIRST = 2;
    const FETCH_ORI_LAST = 3;
    const FETCH_ORI_ABS = 4;
    const FETCH_ORI_REL = 5;
    const CURSOR_FWDONLY = 0;
    const CURSOR_SCROLL = 1;
    const DBLIB_ATTR_CONNECTION_TIMEOUT = 1000;
    const DBLIB_ATTR_QUERY_TIMEOUT = 1001;
    const DBLIB_ATTR_STRINGIFY_UNIQUEIDENTIFIER = 1002;
    const DBLIB_ATTR_VERSION = 1003;
    const DBLIB_ATTR_TDS_VERSION = 1004;
    const DBLIB_ATTR_SKIP_EMPTY_ROWSETS = 1005;
    const DBLIB_ATTR_DATETIME_CONVERT = 1006;
    const MYSQL_ATTR_USE_BUFFERED_QUERY = 1000;
    const MYSQL_ATTR_LOCAL_INFILE = 1001;
    const MYSQL_ATTR_INIT_COMMAND = 1002;
    const MYSQL_ATTR_COMPRESS = 1003;
    const MYSQL_ATTR_DIRECT_QUERY = 20;
    const MYSQL_ATTR_FOUND_ROWS = 1004;
    const MYSQL_ATTR_IGNORE_SPACE = 1005;
    const MYSQL_ATTR_SSL_KEY = 1006;
    const MYSQL_ATTR_SSL_CERT = 1007;
    const MYSQL_ATTR_SSL_CA = 1008;
    const MYSQL_ATTR_SSL_CAPATH = 1009;
    const MYSQL_ATTR_SSL_CIPHER = 1010;
    const MYSQL_ATTR_SERVER_PUBLIC_KEY = 1011;
    const MYSQL_ATTR_MULTI_STATEMENTS = 1012;
    const MYSQL_ATTR_SSL_VERIFY_SERVER_CERT = 1013;
    const MYSQL_ATTR_LOCAL_INFILE_DIRECTORY = 1014;
    const ODBC_ATTR_USE_CURSOR_LIBRARY = 1000;
    const ODBC_ATTR_ASSUME_UTF8 = 1001;
    const ODBC_SQL_USE_IF_NEEDED = 0;
    const ODBC_SQL_USE_DRIVER = 2;
    const ODBC_SQL_USE_ODBC = 1;
    const PGSQL_ATTR_DISABLE_PREPARES = 1000;
    const PGSQL_TRANSACTION_IDLE = 0;
    const PGSQL_TRANSACTION_ACTIVE = 1;
    const PGSQL_TRANSACTION_INTRANS = 2;
    const PGSQL_TRANSACTION_INERROR = 3;
    const PGSQL_TRANSACTION_UNKNOWN = 4;
    const SQLITE_DETERMINISTIC = 2048;
    const SQLITE_ATTR_OPEN_FLAGS = 1000;
    const SQLITE_OPEN_READONLY = 1;
    const SQLITE_OPEN_READWRITE = 2;
    const SQLITE_OPEN_CREATE = 4;
    const SQLITE_ATTR_READONLY_STATEMENT = 1001;
    const SQLITE_ATTR_EXTENDED_RESULT_CODES = 1002;
    private $__h = null;
    private $__attrs = array();
    private $__err = array('00000', null, null);
    // The driver-level einfo (native code, native msg): unlike __err it is NOT
    // reset by a later success -- a fresh statement's errorInfo leaks it
    // (oracle: ['', 1, 'near "BROKEN": syntax error'] after a past failure).
    private $__driverErr = array(null, null);
    public function __construct($dsn, $username = null, $password = null, $options = null) {
        $r = __pdo_open((string)$dsn);
        if (is_array($r)) {
            $e = new PDOException($r[0], $r[1]);
            if ($r[2] !== null) { $e->errorInfo = array($r[2], $r[1], $r[3]); }
            throw $e;
        }
        $this->__h = $r;
        $this->__attrs = array(PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION);
        if (is_array($options)) {
            foreach ($options as $k => $v) { $this->setAttribute($k, $v); }
        }
    }
    public function __destruct() {
        if ($this->__h !== null) { __pdo_close($this->__h); $this->__h = null; }
    }
    // Register a PHP callable as a sqlite scalar function (the private $__h
    // lives on PDO, so both public surfaces delegate here).
    protected function __sqliteUdf(string $function_name, $callback, int $num_args, int $flags): bool {
        return __pdo_create_function($this->__h, $function_name, $callback, $num_args, $flags) === true;
    }
    // pdo_sqlite's BC method: deprecated in 8.5 in favour of the driver
    // subclass method Pdo\Sqlite::createFunction().
    public function sqliteCreateFunction(string $function_name, $callback, int $num_args = -1, int $flags = 0): bool {
        __deprecated_from_caller('Method PDO::sqliteCreateFunction() is deprecated since 8.5, use Pdo\Sqlite::createFunction() instead');
        return $this->__sqliteUdf($function_name, $callback, $num_args, $flags);
    }
    // PHP 8.4 static factory (doctrine/dbal's PDOConnect prefers it): returns
    // the driver subclass (Pdo\Sqlite) for a sqlite DSN, like the real one.
    public static function connect($dsn, $username = null, $password = null, $options = null) {
        if (strncmp((string)$dsn, 'sqlite:', 7) === 0 && !is_a(static::class, 'Pdo\Sqlite', true)) {
            return new \Pdo\Sqlite($dsn, $username, $password, $options);
        }
        return new static($dsn, $username, $password, $options);
    }
    // get/setAttribute follow pdo_dbh.c: the error state clears at method
    // ENTRY (a later errorCode() reads 00000 even after a failed call here);
    // an attribute outside the supported set raises SQLSTATE[IM001] on get
    // and plainly returns false on set.
    // TypeError text helper (PHP's zend_zval_value_name flavour).
    public static function __tname($v) {
        if ($v === null) { return 'null'; }
        if (is_bool($v)) { return 'bool'; }
        if (is_int($v)) { return 'int'; }
        if (is_float($v)) { return 'float'; }
        if (is_string($v)) { return 'string'; }
        if (is_array($v)) { return 'array'; }
        if (is_object($v)) { return get_class($v); }
        return gettype($v);
    }
    public function setAttribute($attribute, $value) {
        $this->__err = array('00000', null, null);
        // ATTR_STATEMENT_CLASS validates eagerly (pdo_dbh.c): array shape,
        // real class, derived from PDOStatement, non-public constructor.
        // Abstractness is NOT checked here -- it errors at instantiation.
        if ($attribute === PDO::ATTR_STATEMENT_CLASS) {
            if (!is_array($value)) {
                throw new TypeError('PDO::setAttribute(): Argument #2 ($value) PDO::ATTR_STATEMENT_CLASS value must be of type array, ' . PDO::__tname($value) . ' given');
            }
            if (!isset($value[0]) || !is_string($value[0]) || !class_exists($value[0])) {
                throw new TypeError('PDO::setAttribute(): Argument #2 ($value) PDO::ATTR_STATEMENT_CLASS class must be a valid class');
            }
            $cls = $value[0];
            if (strcasecmp($cls, 'PDOStatement') !== 0 && !is_subclass_of($cls, 'PDOStatement')) {
                throw new TypeError('PDO::setAttribute(): Argument #2 ($value) PDO::ATTR_STATEMENT_CLASS class must be derived from PDOStatement');
            }
            if (strcasecmp($cls, 'PDOStatement') !== 0 && method_exists($cls, '__construct')) {
                // Host hook directly -- intermediate Reflection objects would
                // burn handle ids the real pdo_dbh.c never allocates (#N).
                $mi = __reflect_method_info($cls, '__construct');
                if ($mi['visibility'] === 'public') {
                    throw new TypeError('PDO::setAttribute(): Argument #2 ($value) User-supplied statement class cannot have a public constructor');
                }
            }
            if (array_key_exists(1, $value) && $value[1] !== null && !is_array($value[1])) {
                throw new TypeError('PDO::setAttribute(): Argument #2 ($value) PDO::ATTR_STATEMENT_CLASS ctor_args must be of type ?array, ' . PDO::__tname($value[1]) . ' given');
            }
            $this->__attrs[$attribute] = $value;
            return true;
        }
        if ($attribute === PDO::ATTR_STRINGIFY_FETCHES) {
            if (!is_bool($value) && !is_int($value) && !is_float($value)) {
                throw new TypeError('Attribute value must be of type bool for selected attribute, ' . PDO::__tname($value) . ' given');
            }
            $this->__attrs[$attribute] = (bool)$value;
            return true;
        }
        if ($attribute === PDO::ATTR_ERRMODE || $attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS
            || $attribute === PDO::ATTR_DEFAULT_FETCH_MODE
            || $attribute === PDO::ATTR_DEFAULT_STR_PARAM
            || $attribute === PDO::ATTR_TIMEOUT || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) {
            $this->__attrs[$attribute] = $value;
            return true;
        }
        return false;
    }
    public function getAttribute($attribute) {
        $this->__err = array('00000', null, null);
        if ($attribute === PDO::ATTR_DRIVER_NAME) { return 'sqlite'; }
        if ($attribute === PDO::ATTR_SERVER_VERSION || $attribute === PDO::ATTR_CLIENT_VERSION) { return __pdo_sqlite_version(); }
        if ($attribute === PDO::ATTR_ERRMODE || $attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS
            || $attribute === PDO::ATTR_PERSISTENT || $attribute === PDO::ATTR_STRINGIFY_FETCHES
            || $attribute === PDO::ATTR_DEFAULT_FETCH_MODE || $attribute === PDO::ATTR_DEFAULT_STR_PARAM
            || $attribute === PDO::ATTR_STATEMENT_CLASS || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) {
            if (array_key_exists($attribute, $this->__attrs)) { return $this->__attrs[$attribute]; }
            if ($attribute === PDO::ATTR_CASE || $attribute === PDO::ATTR_ORACLE_NULLS || $attribute === PDO::SQLITE_ATTR_EXTENDED_RESULT_CODES) { return 0; }
            if ($attribute === PDO::ATTR_PERSISTENT || $attribute === PDO::ATTR_STRINGIFY_FETCHES) { return false; }
            if ($attribute === PDO::ATTR_DEFAULT_FETCH_MODE) { return PDO::FETCH_BOTH; }
            if ($attribute === PDO::ATTR_ERRMODE) { return PDO::ERRMODE_EXCEPTION; }
            if ($attribute === PDO::ATTR_STATEMENT_CLASS) { return array('PDOStatement'); }
            return null;
        }
        return $this->__raise(array('SQLSTATE[IM001]: Driver does not support this function: driver does not support that attribute', 'IM001', null, null), 'PDO::getAttribute');
    }
    public static function getAvailableDrivers() { return array('sqlite'); }
    public function exec($statement) {
        $r = __pdo_exec($this->__h, (string)$statement);
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::exec'); }
        $this->__err = array('00000', null, null);
        return $r['changes'];
    }
    // pdo_sqlite implements in_transaction via sqlite3_get_autocommit (so a
    // manual exec('BEGIN') IS visible here); the state errors below throw
    // *unconditionally*, whatever ATTR_ERRMODE says (pdo_dbh.c).
    public function beginTransaction() {
        $this->__err = array('00000', null, null);
        if ($this->inTransaction()) { throw new PDOException('There is already an active transaction'); }
        $r = __pdo_exec($this->__h, 'BEGIN');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::beginTransaction'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function commit() {
        $this->__err = array('00000', null, null);
        if (!$this->inTransaction()) { throw new PDOException('There is no active transaction'); }
        $r = __pdo_exec($this->__h, 'COMMIT');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::commit'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function rollBack() {
        $this->__err = array('00000', null, null);
        if (!$this->inTransaction()) { throw new PDOException('There is no active transaction'); }
        $r = __pdo_exec($this->__h, 'ROLLBACK');
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDO::rollBack'); }
        $this->__err = array('00000', null, null);
        return true;
    }
    public function inTransaction() { return __pdo_in_txn($this->__h); }
    public function errorCode() { return $this->__err[0]; }
    public function errorInfo() { return array($this->__err[0], $this->__err[1], $this->__err[2]); }
    public function __driverError() { return $this->__driverErr; }
    public function quote($string, $type = PDO::PARAM_STR) {
        $s = (string)$string;
        if (strpos($s, "\0") !== false) { return false; }
        return "'" . str_replace("'", "''", $s) . "'";
    }
    public function prepare($query, $options = null) {
        // pdo_sqlite prepares eagerly: broken SQL fails here, not at execute.
        $r = __pdo_prepare($this->__h, (string)$query);
        if (is_array($r) && isset($r['err'])) { return $this->__raise($r['err'], 'PDO::prepare'); }
        $this->__err = array('00000', null, null);
        return $this->__newStatement((string)$query);
    }
    // ATTR_STATEMENT_CLASS instantiation: the class was validated at set time;
    // abstractness surfaces HERE (an Error, like `new` would raise), and the
    // (usually non-public) constructor runs AFTER the statement is wired,
    // with the declared ctor_args -- visibility bypassed like pdo_stmt.c.
    public function __newStatement($sql) {
        $sc = isset($this->__attrs[PDO::ATTR_STATEMENT_CLASS]) ? $this->__attrs[PDO::ATTR_STATEMENT_CLASS] : null;
        $cls = $sc !== null ? $sc[0] : 'PDOStatement';
        if (strcasecmp($cls, 'PDOStatement') === 0) {
            $st = new PDOStatement();
            $st->__pdoInit($this, $this->__h, $sql);
            return $st;
        }
        // Host hooks directly (no intermediate Reflection objects: they would
        // burn handle ids the real pdo_stmt.c never allocates, shifting #N).
        if (__reflect_class_modifiers($cls)['abstract']) { throw new Error('Cannot instantiate abstract class ' . $cls); }
        $st = __reflect_new_no_ctor($cls);
        $st->__pdoInit($this, $this->__h, $sql);
        if (method_exists($cls, '__construct')) {
            $args = isset($sc[1]) && is_array($sc[1]) ? $sc[1] : array();
            __reflect_invoke($st, $cls, '__construct', $args);
        }
        return $st;
    }
    public function query($query, $mode = null, ...$args) {
        $st = $this->prepare($query);
        if ($st === false) { return false; }
        if ($mode !== null) { $st->setFetchMode($mode, ...$args); }
        if ($st->execute() === false) { return false; }
        return $st;
    }
    public function lastInsertId($name = null) {
        return (string)__pdo_last_id($this->__h);
    }
    // The shared error sink: record errorInfo and act per ATTR_ERRMODE.
    // Payload = [full message, sqlstate, native code|null, native msg|null];
    // runtime PDOException codes are the SQLSTATE *string* (connection-time
    // ctor failures use the native int instead, see __construct).
    public function __raise($e, $fn) {
        $this->__err = array($e[1], $e[2], $e[3]);
        if ($e[2] !== null) { $this->__driverErr = array($e[2], $e[3]); }
        $mode = isset($this->__attrs[PDO::ATTR_ERRMODE]) ? $this->__attrs[PDO::ATTR_ERRMODE] : PDO::ERRMODE_EXCEPTION;
        if ($mode === PDO::ERRMODE_EXCEPTION) {
            $ex = new PDOException($e[0], $e[1]);
            $ex->errorInfo = array($e[1], $e[2], $e[3]);
            throw $ex;
        }
        if ($mode === PDO::ERRMODE_WARNING) { trigger_error($fn . '(): ' . $e[0], E_USER_WARNING); }
        return false;
    }
}
// The statement: prepared-SQL + bound-params holder; execute() ships both to
// __pdo_run (the host re-prepares each time: sqlite has no server state to
// lose) and materializes the whole rowset VM-side, fetch* then walk it.
class PDOStatement implements IteratorAggregate {
    public $queryString = '';
    private $__pdo = null;
    private $__c = null;
    private $__cols = array();
    private $__rows = null;
    private $__pos = 0;
    private $__changes = 0;
    private $__bound = array();
    // null until the first execute-ish op: a fresh statement's errorCode() is
    // NULL while its errorInfo() shows '' plus the *connection's* last driver
    // einfo (oracle-verified pdo_stmt.c behaviour).
    private $__err = null;
    private $__mode = null;
    private $__modeArgs = array();
    private $__meta = array();
    private $__freed = false;
    private $__boundCols = array();
    public function __pdoInit($pdo, $h, $sql) {
        $this->__pdo = $pdo;
        $this->__c = $h;
        $this->queryString = $sql;
    }
    // bindValue/execute(array) coercions: execute(array) values are all
    // PARAM_STR (oracle: execute([1]) fetches back "1"); bindValue applies the
    // declared PARAM_* type. null always stays null; bools bind as sqlite ints.
    private function __coerce($v, $t) {
        if ($v === null) { return null; }
        $t = $t & ~PDO::PARAM_INPUT_OUTPUT;
        // A stream bound as PARAM_LOB sends its remaining contents.
        if ($t === PDO::PARAM_LOB && is_resource($v)) { return stream_get_contents($v); }
        if ($t === PDO::PARAM_INT) { return (int)$v; }
        if ($t === PDO::PARAM_BOOL) { return (bool)$v; }
        if ($t === PDO::PARAM_NULL) { return null; }
        if ($t === PDO::PARAM_STR || $t === PDO::PARAM_LOB) { return (string)$v; }
        return $v;
    }
    public function bindValue($param, $value, $type = PDO::PARAM_STR) {
        $this->__bound[$param] = array($this->__coerce($value, $type), $type, false, null);
        return true;
    }
    public function bindParam($param, &$var, $type = PDO::PARAM_STR, $maxLength = 0, $driverOptions = null) {
        $this->__bound[$param] = array(null, $type, true, null);
        $this->__bound[$param][3] =& $var;
        return true;
    }
    public function execute($params = null) {
        $send = array();
        $strict = false;
        if (is_array($params)) {
            $strict = true;
            foreach ($params as $k => $v) {
                // A 0-based execute(array) list feeds the 1-based placeholders.
                $key = is_int($k) ? $k + 1 : $k;
                $send[$key] = $v === null ? null : (string)$v;
            }
        } else {
            foreach ($this->__bound as $k => $b) {
                $send[$k] = $b[2] ? $this->__coerce($b[3], $b[1]) : $b[0];
            }
        }
        $r = __pdo_run($this->__c, $this->queryString, $send, $strict);
        if (isset($r['err'])) { return $this->__raise($r['err'], 'PDOStatement::execute'); }
        $this->__err = array('00000', null, null);
        $this->__cols = isset($r['cols']) ? $r['cols'] : array();
        $this->__rows = isset($r['rows']) ? $r['rows'] : array();
        $this->__meta = isset($r['meta']) ? $r['meta'] : array();
        $this->__pos = 0;
        $this->__freed = false;
        $this->__changes = isset($r['changes']) ? $r['changes'] : 0;
        return true;
    }
    // FETCH_CLASS instantiation: props are written BEFORE the constructor
    // runs, unless FETCH_PROPS_LATE flips the order (bug46139 semantics).
    private function __fetchClass($assoc, $class, $ctorArgs, $propsLate) {
        $rc = new ReflectionClass($class);
        $o = $rc->newInstanceWithoutConstructor();
        $ctor = $rc->getConstructor();
        if ($propsLate && $ctor !== null) { $o->__construct(...$ctorArgs); }
        foreach ($assoc as $k => $v) { $o->$k = $v; }
        if (!$propsLate && $ctor !== null) { $o->__construct(...$ctorArgs); }
        return $o;
    }
    private function __buildRow($row, $mode, $margs = null, $colOff = 0) {
        $pdo = $this->__pdo;
        if ($mode === null || $mode === PDO::FETCH_DEFAULT) { $mode = $this->__mode; }
        if ($mode === null || $mode === PDO::FETCH_DEFAULT) {
            $mode = $pdo !== null ? $pdo->getAttribute(PDO::ATTR_DEFAULT_FETCH_MODE) : PDO::FETCH_BOTH;
        }
        if ($margs === null) { $margs = $this->__modeArgs; }
        $flags = $mode & ~15;
        $mode = $mode & 15;
        $stringify = $pdo !== null && $pdo->getAttribute(PDO::ATTR_STRINGIFY_FETCHES);
        $case = $pdo !== null ? $pdo->getAttribute(PDO::ATTR_CASE) : PDO::CASE_NATURAL;
        // $colOff > 0 = FETCH_GROUP/UNIQUE: the first column became the group
        // key, the row is built from the REMAINING columns (0-rebased).
        $vals = array();
        foreach ($row as $i => $v) {
            if ($i < $colOff) { continue; }
            if ($stringify && (is_int($v) || is_float($v))) { $v = (string)$v; }
            $vals[$i - $colOff] = $v;
        }
        if ($mode === PDO::FETCH_NUM) { return $vals; }
        if ($mode === PDO::FETCH_COLUMN) {
            $col = isset($margs[0]) ? $margs[0] : 0;
            return array_key_exists($col, $vals) ? $vals[$col] : null;
        }
        $names = array();
        foreach ($this->__cols as $i => $n) {
            if ($i < $colOff) { continue; }
            if ($case === PDO::CASE_LOWER) { $n = strtolower($n); }
            elseif ($case === PDO::CASE_UPPER) { $n = strtoupper($n); }
            $names[$i - $colOff] = $n;
        }
        if ($mode === PDO::FETCH_CLASS) {
            $assoc = array();
            foreach ($vals as $i => $v) { $assoc[$names[$i]] = $v; }
            $class = isset($margs[0]) ? $margs[0] : 'stdClass';
            $ctorArgs = isset($margs[1]) && is_array($margs[1]) ? $margs[1] : array();
            return $this->__fetchClass($assoc, $class, $ctorArgs, ($flags & PDO::FETCH_PROPS_LATE) !== 0);
        }
        if ($mode === PDO::FETCH_INTO) {
            $obj = isset($margs[0]) ? $margs[0] : new stdClass();
            foreach ($vals as $i => $v) { $n = $names[$i]; $obj->$n = $v; }
            return $obj;
        }
        if ($mode === PDO::FETCH_KEY_PAIR) { return array($vals[0] => $vals[1]); }
        if ($mode === PDO::FETCH_NAMED) {
            $out = array();
            $dup = array();
            foreach ($vals as $i => $v) {
                $n = $names[$i];
                if (array_key_exists($n, $out)) {
                    if (!isset($dup[$n])) { $out[$n] = array($out[$n]); $dup[$n] = true; }
                    $out[$n][] = $v;
                } else { $out[$n] = $v; }
            }
            return $out;
        }
        $out = array();
        foreach ($vals as $i => $v) {
            if ($mode === PDO::FETCH_ASSOC || $mode === PDO::FETCH_OBJ || $mode === PDO::FETCH_BOTH) { $out[$names[$i]] = $v; }
            if ($mode === PDO::FETCH_BOTH || $mode === PDO::FETCH_NUM) { $out[$i] = $v; }
        }
        if ($mode === PDO::FETCH_OBJ) { return (object)$out; }
        return $out;
    }
    public function fetch($mode = null, $cursorOrientation = 0, $cursorOffset = 0) {
        if ($this->__rows === null || $this->__pos >= count($this->__rows)) { return false; }
        $row = $this->__rows[$this->__pos];
        $this->__pos = $this->__pos + 1;
        // Bound columns (bindColumn) refresh on EVERY fetch, whatever the mode;
        // FETCH_BOUND then just reports success without building a row.
        if (count($this->__boundCols)) {
            foreach ($this->__boundCols as $col => $b) {
                $i = is_int($col) ? $col - 1 : array_search($col, $this->__cols, true);
                $v = ($i !== false && $i !== null && array_key_exists($i, $row)) ? $row[$i] : null;
                $this->__boundCols[$col][0] = $this->__coerce($v, $b[1]);
            }
        }
        if ($mode === PDO::FETCH_BOUND) { return true; }
        return $this->__buildRow($row, $mode);
    }
    public function bindColumn($column, &$var, $type = PDO::PARAM_STR, $maxLength = 0, $driverOptions = null) {
        $this->__boundCols[$column] = array(null, $type === null ? PDO::PARAM_STR : $type);
        $this->__boundCols[$column][0] =& $var;
        return true;
    }
    public function fetchAll($mode = null, ...$args) {
        if ($this->__rows === null) { return array(); }
        $out = array();
        $flags = is_int($mode) ? $mode : 0;
        // FETCH_GROUP/FETCH_UNIQUE: key on the first column, build the rest.
        if (($flags & (PDO::FETCH_GROUP | PDO::FETCH_UNIQUE)) !== 0) {
            $unique = ($flags & PDO::FETCH_UNIQUE) !== 0;
            $inner = $mode & ~(PDO::FETCH_GROUP | PDO::FETCH_UNIQUE);
            if (($inner & 15) === 0) { $inner = $inner | PDO::FETCH_BOTH; }
            while ($this->__pos < count($this->__rows)) {
                $row = $this->__rows[$this->__pos];
                $this->__pos = $this->__pos + 1;
                $key = $row[0];
                $built = $this->__buildRow($row, $inner, count($args) ? $args : null, 1);
                if ($unique) { $out[$key] = $built; }
                else { $out[$key][] = $built; }
            }
            return $out;
        }
        // FETCH_FUNC: one callable invocation per row, columns as arguments.
        if (($flags & 15) === PDO::FETCH_FUNC) {
            $fn = isset($args[0]) ? $args[0] : (isset($this->__modeArgs[0]) ? $this->__modeArgs[0] : null);
            while (($row = $this->fetch(PDO::FETCH_NUM)) !== false) { $out[] = $fn(...$row); }
            return $out;
        }
        // FETCH_CLASSTYPE: the class name comes from the FIRST column, the
        // remaining columns are the properties (stdClass when unknown).
        if (($flags & PDO::FETCH_CLASSTYPE) !== 0) {
            $inner = $mode & ~PDO::FETCH_CLASSTYPE;
            while ($this->__pos < count($this->__rows)) {
                $row = $this->__rows[$this->__pos];
                $this->__pos = $this->__pos + 1;
                $cls = (string)$row[0];
                if (!class_exists($cls)) { $cls = 'stdClass'; }
                $out[] = $this->__buildRow($row, PDO::FETCH_CLASS | ($inner & ~15), array($cls, array()), 1);
            }
            return $out;
        }
        if ($mode === PDO::FETCH_COLUMN) {
            $col = isset($args[0]) ? $args[0] : (isset($this->__modeArgs[0]) ? $this->__modeArgs[0] : 0);
            while (($row = $this->fetch(PDO::FETCH_NUM)) !== false) { $out[] = array_key_exists($col, $row) ? $row[$col] : null; }
            return $out;
        }
        if ($mode === PDO::FETCH_KEY_PAIR) {
            while (($row = $this->fetch(PDO::FETCH_NUM)) !== false) { $out[$row[0]] = $row[1]; }
            return $out;
        }
        while ($this->__pos < count($this->__rows)) {
            $out[] = $this->__buildRow($this->__rows[$this->__pos], $mode, count($args) ? $args : null);
            $this->__pos = $this->__pos + 1;
        }
        return $out;
    }
    public function fetchColumn($column = 0) {
        $row = $this->fetch(PDO::FETCH_NUM);
        if ($row === false) { return false; }
        return array_key_exists($column, $row) ? $row[$column] : null;
    }
    public function fetchObject($class = 'stdClass', $constructorArgs = array()) {
        if (!is_string($class) || !class_exists($class)) {
            throw new TypeError('PDOStatement::fetchObject(): Argument #1 ($class) must be a valid class name, ' . $class . ' given');
        }
        $row = $this->fetch(PDO::FETCH_ASSOC);
        if ($row === false) { return false; }
        if ($class === 'stdClass') { return (object)$row; }
        $rc = new ReflectionClass($class);
        $o = $rc->newInstanceWithoutConstructor();
        foreach ($row as $k => $v) { $o->$k = $v; }
        if ($rc->getConstructor() !== null) { $o->__construct(...$constructorArgs); }
        return $o;
    }
    public function setFetchMode($mode, ...$args) {
        $this->__mode = $mode;
        $this->__modeArgs = $args;
        return true;
    }
    public function rowCount() { return $this->__changes; }
    public function columnCount() { return count($this->__cols); }
    // getColumnMeta statics come from the host ('meta': decl type + table,
    // absent on expression columns); native_type/pdo_type reflect the *value*
    // in the materialized first row, like sqlite3_column_type at execute.
    public function getColumnMeta($column) {
        if ($column < 0) { throw new ValueError('PDOStatement::getColumnMeta(): Argument #1 ($column) must be greater than or equal to 0'); }
        if ($this->__rows === null || $this->__freed) { return false; }
        if ($column >= count($this->__cols)) {
            // With rows still pending, pdo_sqlite surfaces the driver state
            // (SQLITE_ROW = 100); with the set exhausted it reports false
            // (PHP >= 8.3.18 behaviour, what DBAL's InvalidColumnIndex needs).
            if ($this->__pos < count($this->__rows)) {
                return $this->__raise(array('SQLSTATE[HY000]: General error: 100 another row available', 'HY000', 100, 'another row available'), 'PDOStatement::getColumnMeta');
            }
            return false;
        }
        $v = count($this->__rows) > 0 ? $this->__rows[0][$column] : null;
        if (is_int($v)) { $nt = 'integer'; $pt = PDO::PARAM_INT; }
        elseif (is_float($v)) { $nt = 'double'; $pt = PDO::PARAM_STR; }
        elseif ($v === null) { $nt = 'null'; $pt = PDO::PARAM_NULL; }
        else { $nt = 'string'; $pt = PDO::PARAM_STR; }
        $out = array('native_type' => $nt, 'pdo_type' => $pt);
        $m = isset($this->__meta[$column]) ? $this->__meta[$column] : array(null, null);
        if ($m[0] !== null) { $out['sqlite:decl_type'] = $m[0]; }
        if ($m[1] !== null) { $out['table'] = $m[1]; }
        $out['flags'] = array();
        $out['name'] = $this->__cols[$column];
        $out['len'] = -1;
        $out['precision'] = 0;
        return $out;
    }
    public function getAttribute($attribute) {
        if ($attribute === PDO::ATTR_EMULATE_PREPARES) { return false; }
        if ($attribute === PDO::SQLITE_ATTR_READONLY_STATEMENT) { return __pdo_stmt_readonly($this->__c, $this->queryString); }
        return $this->__raise(array('SQLSTATE[IM001]: Driver does not support this function: driver does not support that attribute', 'IM001', null, null), 'PDOStatement::getAttribute');
    }
    public function setAttribute($attribute, $value) { return false; }
    public function errorCode() { return $this->__err === null ? null : $this->__err[0]; }
    public function errorInfo() {
        if ($this->__err === null) {
            $d = $this->__pdo !== null ? $this->__pdo->__driverError() : array(null, null);
            return array('', $d[0], $d[1]);
        }
        return array($this->__err[0], $this->__err[1], $this->__err[2]);
    }
    public function closeCursor() {
        $this->__rows = array();
        $this->__pos = 0;
        $this->__freed = true;
        return true;
    }
    public function getIterator(): Iterator {
        $rows = array();
        while (($row = $this->fetch()) !== false) { $rows[] = $row; }
        return new ArrayIterator($rows);
    }
    // Mirrors PDO::__raise, reading the owner's ERRMODE.
    public function __raise($e, $fn) {
        $this->__err = array($e[1], $e[2], $e[3]);
        $mode = $this->__pdo !== null ? $this->__pdo->getAttribute(PDO::ATTR_ERRMODE) : PDO::ERRMODE_EXCEPTION;
        if ($mode === PDO::ERRMODE_EXCEPTION) {
            $ex = new PDOException($e[0], $e[1]);
            $ex->errorInfo = array($e[1], $e[2], $e[3]);
            throw $ex;
        }
        if ($mode === PDO::ERRMODE_WARNING) { trigger_error($fn . '(): ' . $e[0], E_USER_WARNING); }
        return false;
    }
}
final class PDORow {
    public $queryString = '';
    public function __construct() { throw new PDOException('You may not create a PDORow manually'); }
}
