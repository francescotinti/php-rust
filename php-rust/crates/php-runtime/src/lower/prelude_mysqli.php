<?php

/*
 * mysqli — MySQL improved (ext/mysqli), the wpdb surface and beyond.
 *
 * The classes and the mysqli_* procedural functions live here (global
 * namespace), delegating to the `__mysqli_*` host builtins (mysql crate) in
 * vm/mysqli.rs. The host returns payload arrays ('t' = ok|res|err); this file
 * turns them into object state, mysqli_result buffers, warnings and
 * mysqli_sql_exception throws according to the driver report mode.
 *
 * Result sets are fully buffered (MYSQLI_STORE_RESULT semantics; USE_RESULT
 * behaves as STORE — no observable difference for a single-threaded client
 * except memory). max_length in field metadata is the constant 0 of
 * PHP >= 8.1 mysqlnd. See PHPR_DIVERGENCES_FROM_PHP.md (WordPress-8).
 */

class mysqli_sql_exception extends RuntimeException
{
    protected string $sqlstate = '00000';

    public function getSqlState(): string
    {
        return $this->sqlstate;
    }

    public static function _make(string $message, int $code, string $sqlstate): mysqli_sql_exception
    {
        $e = new mysqli_sql_exception($message, $code);
        $e->sqlstate = $sqlstate;
        return $e;
    }
}

class mysqli_driver
{
    public static $__report_mode = 3; // MYSQLI_REPORT_ERROR | MYSQLI_REPORT_STRICT
    public $client_info = 'mysqlnd 8.5.7';
    public $client_version = 80507;
    public $driver_version = 101012;
    public $reconnect = false;

    public function __get($name)
    {
        if ($name === 'report_mode') {
            return self::$__report_mode;
        }
        return null;
    }

    public function __set($name, $value)
    {
        if ($name === 'report_mode') {
            self::$__report_mode = (int) $value;
        }
    }
}

class mysqli
{
    public $affected_rows = 0;
    public $client_info = 'mysqlnd 8.5.7';
    public $client_version = 80507;
    public $connect_errno = 0;
    public $connect_error = null;
    public $errno = 0;
    public $error = '';
    public $error_list = array();
    public $field_count = 0;
    public $host_info = null;
    public $info = null;
    public $insert_id = 0;
    public $protocol_version = 10;
    public $server_info = null;
    public $server_version = null;
    public $sqlstate = '00000';
    public $thread_id = 0;
    public $warning_count = 0;

    private $__h = null;
    private $__stash = null; // real_query()'s pending 'res' payload for store_result()

    public static $__last_connect_errno = 0;
    public static $__last_connect_error = null;

    public function __construct(
        ?string $hostname = null,
        ?string $username = null,
        ?string $password = null,
        ?string $database = null,
        ?int $port = null,
        ?string $socket = null
    ) {
        if (\func_num_args() === 0) {
            return; // mysqli_init() shape: initialized, not connected
        }
        $this->real_connect($hostname, $username, $password, $database, $port, $socket);
    }

    /** The open native handle, or an Error identical to ext/mysqli's. */
    public function _h()
    {
        if ($this->__h === null) {
            throw new Error('mysqli object is already closed');
        }
        return $this->__h;
    }

    /** Absorb an err payload into errno/error/sqlstate (+ report mode). */
    private function _fail($p, string $where)
    {
        $this->errno = $p['errno'];
        $this->error = $p['error'];
        $this->sqlstate = $p['sqlstate'];
        $this->error_list = array(array(
            'errno' => $p['errno'],
            'sqlstate' => $p['sqlstate'],
            'error' => $p['error'],
        ));
        $this->affected_rows = -1;
        $mode = mysqli_driver::$__report_mode;
        if ($mode & 2) { // MYSQLI_REPORT_STRICT
            throw mysqli_sql_exception::_make($p['error'], $p['errno'], $p['sqlstate']);
        }
        if ($mode & 1) { // MYSQLI_REPORT_ERROR
            __warning_from_caller($where . '(): (' . $p['sqlstate'] . '/' . $p['errno'] . '): ' . $p['error']);
        }
        return false;
    }

    private function _clearErr()
    {
        $this->errno = 0;
        $this->error = '';
        $this->sqlstate = '00000';
        $this->error_list = array();
    }

    public function real_connect(
        $hostname = null,
        $username = null,
        $password = null,
        $database = null,
        $port = null,
        $socket = null,
        $flags = 0
    ) {
        if ($username === null) { $username = 'root'; }
        if ($password === null) { $password = ''; }
        if ($port === '') { $port = null; }
        $r = __mysqli_connect($hostname, (string) $username, (string) $password, $database, $port === null ? null : (int) $port, $socket);
        if ($r['t'] === 'err') {
            $this->connect_errno = $r['errno'];
            $this->connect_error = $r['error'];
            self::$__last_connect_errno = $r['errno'];
            self::$__last_connect_error = $r['error'];
            if (mysqli_driver::$__report_mode & 2) {
                throw mysqli_sql_exception::_make($r['error'], $r['errno'], $r['sqlstate']);
            }
            __warning_from_caller('mysqli_real_connect(): (' . $r['sqlstate'] . '/' . $r['errno'] . '): ' . $r['error']);
            return false;
        }
        $this->__h = $r['h'];
        $this->connect_errno = 0;
        $this->connect_error = null;
        self::$__last_connect_errno = 0;
        self::$__last_connect_error = null;
        $this->server_info = $r['server_info'];
        $this->server_version = $r['server_version'];
        $this->host_info = $r['host_info'];
        $this->protocol_version = $r['protocol'];
        $this->thread_id = $r['thread_id'];
        $this->_clearErr();
        return true;
    }

    /** Absorb a query payload: true / mysqli_result / false-with-error. */
    public function _absorb($p, string $where)
    {
        if ($p['t'] === 'err') {
            return $this->_fail($p, $where);
        }
        $this->_clearErr();
        if ($p['t'] === 'ok') {
            $this->affected_rows = $p['affected'];
            $this->insert_id = $p['insert_id'];
            $this->warning_count = $p['warnings'];
            $this->info = $p['info'];
            $this->field_count = 0;
            return true;
        }
        // 't' === 'res'
        $res = mysqli_result::_make($p);
        $this->affected_rows = $res->num_rows;
        $this->insert_id = 0;
        $this->warning_count = $p['warnings'];
        $this->info = null;
        $this->field_count = $res->field_count;
        return $res;
    }

    public function query(string $query, int $result_mode = MYSQLI_STORE_RESULT)
    {
        return $this->_absorb(__mysqli_query($this->_h(), $query), 'mysqli_query');
    }

    public function real_query(string $query): bool
    {
        $p = __mysqli_query($this->_h(), $query);
        if ($p['t'] === 'err') {
            return $this->_fail($p, 'mysqli_real_query') !== false;
        }
        if ($p['t'] === 'res') {
            $this->__stash = $p;
            $this->_clearErr();
            $this->field_count = \count($p['fields']);
            $this->warning_count = $p['warnings'];
            return true;
        }
        return $this->_absorb($p, 'mysqli_real_query') === true;
    }

    public function store_result(int $mode = 0)
    {
        $this->_h();
        if ($this->__stash === null) {
            return false;
        }
        $p = $this->__stash;
        $this->__stash = null;
        $res = mysqli_result::_make($p);
        $this->affected_rows = $res->num_rows;
        return $res;
    }

    public function use_result()
    {
        return $this->store_result();
    }

    public function multi_query(string $query): bool
    {
        $p = __mysqli_multi_query($this->_h(), $query);
        if ($p['t'] === 'err') {
            return $this->_fail($p, 'mysqli_multi_query') !== false;
        }
        if ($p['t'] === 'res') {
            $this->__stash = $p;
            $this->_clearErr();
            $this->field_count = \count($p['fields']);
            $this->warning_count = $p['warnings'];
            return true;
        }
        return $this->_absorb($p, 'mysqli_multi_query') === true;
    }

    public function more_results(): bool
    {
        return __mysqli_more_results($this->_h());
    }

    public function next_result(): bool
    {
        $p = __mysqli_next_result($this->_h());
        if ($p === null) {
            return false;
        }
        if ($p['t'] === 'err') {
            return $this->_fail($p, 'mysqli_next_result') !== false;
        }
        if ($p['t'] === 'res') {
            $this->__stash = $p;
            $this->_clearErr();
            $this->field_count = \count($p['fields']);
            return true;
        }
        $this->_absorb($p, 'mysqli_next_result');
        return true;
    }

    public function prepare(string $query)
    {
        $p = __mysqli_prepare($this->_h(), $query);
        if ($p['t'] === 'err') {
            return $this->_fail($p, 'mysqli_prepare');
        }
        $this->_clearErr();
        return mysqli_stmt::_make($this, $p);
    }

    public function stmt_init(): mysqli_stmt
    {
        $this->_h();
        return mysqli_stmt::_make($this, null);
    }

    public function select_db(string $database): bool
    {
        $r = __mysqli_select_db($this->_h(), $database);
        if ($r === true) {
            $this->_clearErr();
            return true;
        }
        return $this->_fail($r, 'mysqli_select_db') !== false;
    }

    public function set_charset(string $charset): bool
    {
        $r = __mysqli_set_charset($this->_h(), $charset);
        if ($r === true) {
            $this->_clearErr();
            return true;
        }
        return $this->_fail($r, 'mysqli_set_charset') !== false;
    }

    public function character_set_name(): string
    {
        return __mysqli_charset($this->_h());
    }

    public function get_charset()
    {
        $name = $this->character_set_name();
        $o = new stdClass();
        $o->charset = $name;
        $o->collation = $name === 'utf8mb4' ? 'utf8mb4_general_ci' : $name . '_general_ci';
        $o->dir = '';
        $o->min_length = 1;
        $o->max_length = $name === 'utf8mb4' ? 4 : ($name === 'utf8' || $name === 'utf8mb3' ? 3 : 1);
        $o->number = $name === 'utf8mb4' ? 45 : 0;
        $o->state = 1;
        $o->comment = '';
        return $o;
    }

    public function real_escape_string(string $string): string
    {
        return __mysqli_escape($this->_h(), $string);
    }

    public function escape_string(string $string): string
    {
        return $this->real_escape_string($string);
    }

    public function close(): bool
    {
        __mysqli_close($this->_h());
        $this->__h = null;
        return true;
    }

    public function ping(): bool
    {
        return __mysqli_ping($this->_h());
    }

    public function stat()
    {
        return __mysqli_stat($this->_h());
    }

    public function get_server_info(): string
    {
        $this->_h();
        return $this->server_info;
    }

    public function get_client_info(): string
    {
        return $this->client_info;
    }

    public function autocommit(bool $enable): bool
    {
        return $this->query('SET autocommit=' . ($enable ? '1' : '0')) === true;
    }

    public function begin_transaction(int $flags = 0, $name = null): bool
    {
        return $this->query('START TRANSACTION') === true;
    }

    public function commit(int $flags = 0, $name = null): bool
    {
        return $this->query('COMMIT') === true;
    }

    public function rollback(int $flags = 0, $name = null): bool
    {
        return $this->query('ROLLBACK') === true;
    }

    public function options(int $option, $value): bool
    {
        return true; // accepted and ignored (connect-time tuning knobs)
    }

    public function set_opt(int $option, $value): bool
    {
        return $this->options($option, $value);
    }

    public function ssl_set($key, $certificate, $ca_certificate, $ca_path, $cipher_algos): bool
    {
        return true;
    }

    public function kill(int $process_id): bool
    {
        return $this->query('KILL ' . $process_id) === true;
    }

    public function refresh(int $flags): bool
    {
        return false;
    }

    public function dump_debug_info(): bool
    {
        return false;
    }
}

class mysqli_result implements IteratorAggregate
{
    public $current_field = 0;
    public $field_count = 0;
    public $lengths = null;
    public $num_rows = 0;
    public $type = 0; // MYSQLI_STORE_RESULT

    private $__rows = array();   // list of positional rows
    private $__fields = array(); // list of field-metadata assoc arrays
    private $__pos = 0;
    private $__freed = false;

    public static function _make($p): mysqli_result
    {
        $r = new mysqli_result();
        $r->__rows = $p['rows'];
        $r->__fields = $p['fields'];
        $r->field_count = \count($p['fields']);
        $r->num_rows = \count($p['rows']);
        return $r;
    }

    private function _open()
    {
        if ($this->__freed) {
            throw new Error('mysqli_result object is already closed');
        }
    }

    public function fetch_row()
    {
        $this->_open();
        if ($this->__pos >= $this->num_rows) {
            $this->lengths = null;
            return null;
        }
        $row = $this->__rows[$this->__pos++];
        $lengths = array();
        foreach ($row as $v) {
            $lengths[] = $v === null ? 0 : \strlen((string) $v);
        }
        $this->lengths = $lengths;
        return $row;
    }

    public function fetch_assoc()
    {
        $row = $this->fetch_row();
        if ($row === null) {
            return null;
        }
        $out = array();
        foreach ($this->__fields as $i => $f) {
            $out[$f['name']] = $row[$i];
        }
        return $out;
    }

    public function fetch_array(int $mode = MYSQLI_BOTH)
    {
        $row = $this->fetch_row();
        if ($row === null) {
            return null;
        }
        if ($mode === MYSQLI_NUM) {
            return $row;
        }
        $out = array();
        foreach ($this->__fields as $i => $f) {
            if ($mode === MYSQLI_BOTH) {
                $out[$i] = $row[$i];
            }
            $out[$f['name']] = $row[$i];
        }
        return $out;
    }

    public function fetch_object(string $class = 'stdClass', array $constructor_args = array())
    {
        $assoc = $this->fetch_assoc();
        if ($assoc === null) {
            return null;
        }
        if ($class === 'stdClass') {
            $o = new stdClass();
        } else {
            $o = new $class(...$constructor_args);
        }
        foreach ($assoc as $k => $v) {
            $o->$k = $v;
        }
        return $o;
    }

    public function fetch_all(int $mode = MYSQLI_NUM): array
    {
        $this->_open();
        $out = array();
        while (($row = $this->fetch_array($mode)) !== null) {
            $out[] = $row;
        }
        return $out;
    }

    public function fetch_column(int $column = 0)
    {
        $row = $this->fetch_row();
        if ($row === null) {
            return false;
        }
        return \array_key_exists($column, $row) ? $row[$column] : false;
    }

    public function data_seek(int $offset): bool
    {
        $this->_open();
        if ($offset < 0 || $offset >= $this->num_rows) {
            return false;
        }
        $this->__pos = $offset;
        return true;
    }

    private function _field_obj($f)
    {
        $o = new stdClass();
        $o->name = $f['name'];
        $o->orgname = $f['orgname'];
        $o->table = $f['table'];
        $o->orgtable = $f['orgtable'];
        $o->def = $f['def'];
        $o->db = $f['db'];
        $o->catalog = $f['catalog'];
        $o->max_length = $f['max_length'];
        $o->length = $f['length'];
        $o->charsetnr = $f['charsetnr'];
        $o->flags = $f['flags'];
        $o->type = $f['type'];
        $o->decimals = $f['decimals'];
        return $o;
    }

    public function fetch_field()
    {
        $this->_open();
        if ($this->current_field >= $this->field_count) {
            return false;
        }
        return $this->_field_obj($this->__fields[$this->current_field++]);
    }

    public function fetch_fields(): array
    {
        $this->_open();
        $out = array();
        foreach ($this->__fields as $f) {
            $out[] = $this->_field_obj($f);
        }
        return $out;
    }

    public function fetch_field_direct(int $index)
    {
        $this->_open();
        if ($index < 0 || $index >= $this->field_count) {
            return false;
        }
        return $this->_field_obj($this->__fields[$index]);
    }

    public function field_seek(int $index): bool
    {
        $this->_open();
        if ($index < 0 || $index >= $this->field_count) {
            return false;
        }
        $this->current_field = $index;
        return true;
    }

    public function free(): void
    {
        $this->_open();
        $this->__freed = true;
        $this->__rows = array();
    }

    public function close(): void
    {
        $this->free();
    }

    public function free_result(): void
    {
        $this->free();
    }

    public function getIterator(): Iterator
    {
        $this->data_seek(0);
        while (($row = $this->fetch_assoc()) !== null) {
            yield $row;
        }
    }
}

class mysqli_stmt
{
    public $affected_rows = 0;
    public $insert_id = 0;
    public $num_rows = 0;
    public $param_count = 0;
    public $field_count = 0;
    public $errno = 0;
    public $error = '';
    public $sqlstate = '00000';
    public $error_list = array();
    public $id = 0;

    private $__link = null;
    private $__sid = null;
    private $__types = '';
    private $__vars = array();   // bind_param references
    private $__out = array();    // bind_result references
    private $__res = null;       // last execute's 'res' payload
    private $__pos = 0;
    private $__closed = false;

    public static function _make($link, $p): mysqli_stmt
    {
        $s = new mysqli_stmt();
        $s->__link = $link;
        if ($p !== null) {
            $s->__sid = $p['h'];
            $s->param_count = $p['params'];
            $s->field_count = \count($p['fields']);
        }
        return $s;
    }

    private function _sid()
    {
        if ($this->__closed) {
            throw new Error('mysqli_stmt object is already closed');
        }
        if ($this->__sid === null) {
            throw new Error('mysqli_stmt object is not fully initialized');
        }
        return $this->__sid;
    }

    private function _fail($p)
    {
        $this->errno = $p['errno'];
        $this->error = $p['error'];
        $this->sqlstate = $p['sqlstate'];
        $this->error_list = array(array(
            'errno' => $p['errno'],
            'sqlstate' => $p['sqlstate'],
            'error' => $p['error'],
        ));
        $mode = mysqli_driver::$__report_mode;
        if ($mode & 2) {
            throw mysqli_sql_exception::_make($p['error'], $p['errno'], $p['sqlstate']);
        }
        return false;
    }

    public function prepare(string $query): bool
    {
        if ($this->__link === null) {
            throw new Error('mysqli_stmt object is not fully initialized');
        }
        $p = __mysqli_prepare($this->__link->_h(), $query);
        if ($p['t'] === 'err') {
            return $this->_fail($p) !== false;
        }
        $this->__sid = $p['h'];
        $this->param_count = $p['params'];
        $this->field_count = \count($p['fields']);
        return true;
    }

    public function bind_param(string $types, mixed &...$vars): bool
    {
        if (\strlen($types) !== \count($vars)) {
            __warning_from_caller("mysqli_stmt_bind_param(): The number of variables must match the number of parameters in the prepared statement");
            return false;
        }
        $this->__types = $types;
        $this->__vars = array();
        foreach ($vars as $i => &$v) {
            $this->__vars[$i] = &$v;
        }
        return true;
    }

    public function execute(?array $params = null): bool
    {
        $sid = $this->_sid();
        $values = array();
        if ($params !== null) {
            foreach ($params as $v) {
                $values[] = $v === null ? null : (string) $v;
            }
        } else {
            foreach ($this->__vars as $i => $v) {
                $t = $i < \strlen($this->__types) ? $this->__types[$i] : 's';
                if ($v === null) {
                    $values[] = null;
                } elseif ($t === 'i') {
                    $values[] = (int) $v;
                } elseif ($t === 'd') {
                    $values[] = (float) $v;
                } else {
                    $values[] = (string) $v;
                }
            }
        }
        $p = __mysqli_stmt_execute($sid, $values);
        if ($p['t'] === 'err') {
            return $this->_fail($p) !== false;
        }
        $this->errno = 0;
        $this->error = '';
        $this->sqlstate = '00000';
        $this->error_list = array();
        if ($p['t'] === 'ok') {
            $this->affected_rows = $p['affected'];
            $this->insert_id = $p['insert_id'];
            $this->__res = null;
            return true;
        }
        $this->__res = $p;
        $this->__pos = 0;
        $this->affected_rows = -1;
        $this->num_rows = 0;
        $this->field_count = \count($p['fields']);
        return true;
    }

    public function get_result()
    {
        $this->_sid();
        if ($this->__res === null) {
            return false;
        }
        $p = $this->__res;
        $this->__res = null;
        return mysqli_result::_make($p);
    }

    public function bind_result(mixed &...$vars): bool
    {
        $this->__out = array();
        foreach ($vars as $i => &$v) {
            $this->__out[$i] = &$v;
        }
        return true;
    }

    public function fetch(): ?bool
    {
        $this->_sid();
        if ($this->__res === null) {
            return null;
        }
        $rows = $this->__res['rows'];
        if ($this->__pos >= \count($rows)) {
            return null;
        }
        $row = $rows[$this->__pos++];
        foreach ($this->__out as $i => &$slot) {
            $slot = $row[$i] ?? null;
        }
        return true;
    }

    public function store_result(): bool
    {
        $this->_sid();
        if ($this->__res !== null) {
            $this->num_rows = \count($this->__res['rows']);
        }
        return true;
    }

    public function free_result(): void
    {
        // rows stay buffered for fetch(); a real free would drop them —
        // observably equal for the fetch-then-free pattern.
    }

    public function reset(): bool
    {
        $this->__pos = 0;
        return true;
    }

    public function close(): bool
    {
        if ($this->__sid !== null) {
            __mysqli_stmt_close($this->__sid);
        }
        $this->__closed = true;
        return true;
    }

    public function attr_set(int $attribute, int $value): bool
    {
        return true;
    }

    public function attr_get(int $attribute): int
    {
        return 0;
    }

    public function result_metadata()
    {
        $this->_sid();
        if ($this->__res === null) {
            return false;
        }
        $p = array('rows' => array(), 'fields' => $this->__res['fields']);
        return mysqli_result::_make($p);
    }
}

class mysqli_warning
{
    public $message = '';
    public $sqlstate = 'HY000';
    public $errno = 0;

    public function next(): bool
    {
        return false;
    }
}

function mysqli_init(): mysqli
{
    return new mysqli();
}

function mysqli_report(int $flags): bool
{
    mysqli_driver::$__report_mode = $flags;
    return true;
}

function mysqli_connect(
    ?string $hostname = null,
    ?string $username = null,
    ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null
) {
    $m = new mysqli();
    $ok = $m->real_connect($hostname, $username, $password, $database, $port, $socket);
    if (!$ok) {
        return false;
    }
    return $m;
}

function mysqli_connect_errno(): int
{
    return mysqli::$__last_connect_errno;
}

function mysqli_connect_error(): ?string
{
    return mysqli::$__last_connect_error;
}

function mysqli_real_connect(
    mysqli $mysql,
    ?string $hostname = null,
    ?string $username = null,
    ?string $password = null,
    ?string $database = null,
    ?int $port = null,
    ?string $socket = null,
    int $flags = 0
): bool {
    return $mysql->real_connect($hostname, $username, $password, $database, $port, $socket, $flags);
}

function mysqli_close(mysqli $mysql): bool
{
    return $mysql->close();
}

function mysqli_query(mysqli $mysql, string $query, int $result_mode = MYSQLI_STORE_RESULT)
{
    return $mysql->query($query, $result_mode);
}

function mysqli_real_query(mysqli $mysql, string $query): bool
{
    return $mysql->real_query($query);
}

function mysqli_multi_query(mysqli $mysql, string $query): bool
{
    return $mysql->multi_query($query);
}

function mysqli_store_result(mysqli $mysql, int $mode = 0)
{
    return $mysql->store_result($mode);
}

function mysqli_use_result(mysqli $mysql)
{
    return $mysql->use_result();
}

function mysqli_more_results(mysqli $mysql): bool
{
    return $mysql->more_results();
}

function mysqli_next_result(mysqli $mysql): bool
{
    return $mysql->next_result();
}

function mysqli_select_db(mysqli $mysql, string $database): bool
{
    return $mysql->select_db($database);
}

function mysqli_set_charset(mysqli $mysql, string $charset): bool
{
    return $mysql->set_charset($charset);
}

function mysqli_character_set_name(mysqli $mysql): string
{
    return $mysql->character_set_name();
}

function mysqli_get_charset(mysqli $mysql)
{
    return $mysql->get_charset();
}

function mysqli_real_escape_string(mysqli $mysql, string $string): string
{
    return $mysql->real_escape_string($string);
}

function mysqli_escape_string(mysqli $mysql, string $string): string
{
    return $mysql->real_escape_string($string);
}

function mysqli_error(mysqli $mysql): string
{
    return $mysql->error;
}

function mysqli_errno(mysqli $mysql): int
{
    return $mysql->errno;
}

function mysqli_sqlstate(mysqli $mysql): string
{
    return $mysql->sqlstate;
}

function mysqli_error_list(mysqli $mysql): array
{
    return $mysql->error_list;
}

function mysqli_insert_id(mysqli $mysql)
{
    return $mysql->insert_id;
}

function mysqli_affected_rows(mysqli $mysql)
{
    return $mysql->affected_rows;
}

function mysqli_field_count(mysqli $mysql): int
{
    return $mysql->field_count;
}

function mysqli_warning_count(mysqli $mysql): int
{
    return $mysql->warning_count;
}

function mysqli_info(mysqli $mysql): ?string
{
    return $mysql->info;
}

function mysqli_get_server_info(mysqli $mysql): string
{
    return $mysql->get_server_info();
}

function mysqli_get_server_version(mysqli $mysql): int
{
    return $mysql->server_version;
}

function mysqli_get_host_info(mysqli $mysql): string
{
    return $mysql->host_info;
}

function mysqli_get_proto_info(mysqli $mysql): int
{
    return $mysql->protocol_version;
}

function mysqli_get_client_info(?mysqli $mysql = null): string
{
    return 'mysqlnd 8.5.7';
}

function mysqli_get_client_version(): int
{
    return 80507;
}

function mysqli_thread_id(mysqli $mysql): int
{
    return $mysql->thread_id;
}

function mysqli_ping(mysqli $mysql): bool
{
    __deprecated_from_caller('Function mysqli_ping() is deprecated since 8.4, as the reconnect feature has been removed in PHP 8.2');
    return $mysql->ping();
}

function mysqli_stat(mysqli $mysql)
{
    return $mysql->stat();
}

function mysqli_autocommit(mysqli $mysql, bool $enable): bool
{
    return $mysql->autocommit($enable);
}

function mysqli_begin_transaction(mysqli $mysql, int $flags = 0, $name = null): bool
{
    return $mysql->begin_transaction($flags, $name);
}

function mysqli_commit(mysqli $mysql, int $flags = 0, $name = null): bool
{
    return $mysql->commit($flags, $name);
}

function mysqli_rollback(mysqli $mysql, int $flags = 0, $name = null): bool
{
    return $mysql->rollback($flags, $name);
}

function mysqli_options(mysqli $mysql, int $option, $value): bool
{
    return $mysql->options($option, $value);
}

function mysqli_set_opt(mysqli $mysql, int $option, $value): bool
{
    return $mysql->options($option, $value);
}

function mysqli_ssl_set(mysqli $mysql, $key, $certificate, $ca_certificate, $ca_path, $cipher_algos): bool
{
    return $mysql->ssl_set($key, $certificate, $ca_certificate, $ca_path, $cipher_algos);
}

function mysqli_kill(mysqli $mysql, int $process_id): bool
{
    return $mysql->kill($process_id);
}

function mysqli_prepare(mysqli $mysql, string $query)
{
    return $mysql->prepare($query);
}

function mysqli_stmt_init(mysqli $mysql): mysqli_stmt
{
    return $mysql->stmt_init();
}

function mysqli_num_rows(mysqli_result $result)
{
    return $result->num_rows;
}

function mysqli_num_fields(mysqli_result $result): int
{
    return $result->field_count;
}

function mysqli_fetch_assoc(mysqli_result $result): ?array
{
    return $result->fetch_assoc();
}

function mysqli_fetch_array(mysqli_result $result, int $mode = MYSQLI_BOTH): ?array
{
    return $result->fetch_array($mode);
}

function mysqli_fetch_row(mysqli_result $result): ?array
{
    return $result->fetch_row();
}

function mysqli_fetch_object(mysqli_result $result, string $class = 'stdClass', array $constructor_args = array()): ?object
{
    return $result->fetch_object($class, $constructor_args);
}

function mysqli_fetch_all(mysqli_result $result, int $mode = MYSQLI_NUM): array
{
    return $result->fetch_all($mode);
}

function mysqli_fetch_column(mysqli_result $result, int $column = 0)
{
    return $result->fetch_column($column);
}

function mysqli_fetch_field(mysqli_result $result)
{
    return $result->fetch_field();
}

function mysqli_fetch_fields(mysqli_result $result): array
{
    return $result->fetch_fields();
}

function mysqli_fetch_field_direct(mysqli_result $result, int $index)
{
    return $result->fetch_field_direct($index);
}

function mysqli_field_seek(mysqli_result $result, int $index): bool
{
    return $result->field_seek($index);
}

function mysqli_field_tell(mysqli_result $result): int
{
    return $result->current_field;
}

function mysqli_data_seek(mysqli_result $result, int $offset): bool
{
    return $result->data_seek($offset);
}

function mysqli_fetch_lengths(mysqli_result $result)
{
    return $result->lengths === null ? false : $result->lengths;
}

function mysqli_free_result(mysqli_result $result): void
{
    $result->free();
}

function mysqli_stmt_prepare(mysqli_stmt $statement, string $query): bool
{
    return $statement->prepare($query);
}

function mysqli_stmt_bind_param(mysqli_stmt $statement, string $types, mixed &...$vars): bool
{
    return $statement->bind_param($types, ...$vars);
}

function mysqli_stmt_execute(mysqli_stmt $statement, ?array $params = null): bool
{
    return $statement->execute($params);
}

function mysqli_stmt_get_result(mysqli_stmt $statement)
{
    return $statement->get_result();
}

function mysqli_stmt_bind_result(mysqli_stmt $statement, mixed &...$vars): bool
{
    return $statement->bind_result(...$vars);
}

function mysqli_stmt_fetch(mysqli_stmt $statement): ?bool
{
    return $statement->fetch();
}

function mysqli_stmt_store_result(mysqli_stmt $statement): bool
{
    return $statement->store_result();
}

function mysqli_stmt_free_result(mysqli_stmt $statement): void
{
    $statement->free_result();
}

function mysqli_stmt_reset(mysqli_stmt $statement): bool
{
    return $statement->reset();
}

function mysqli_stmt_close(mysqli_stmt $statement): bool
{
    return $statement->close();
}

function mysqli_stmt_affected_rows(mysqli_stmt $statement)
{
    return $statement->affected_rows;
}

function mysqli_stmt_insert_id(mysqli_stmt $statement)
{
    return $statement->insert_id;
}

function mysqli_stmt_num_rows(mysqli_stmt $statement)
{
    return $statement->num_rows;
}

function mysqli_stmt_param_count(mysqli_stmt $statement): int
{
    return $statement->param_count;
}

function mysqli_stmt_field_count(mysqli_stmt $statement): int
{
    return $statement->field_count;
}

function mysqli_stmt_errno(mysqli_stmt $statement): int
{
    return $statement->errno;
}

function mysqli_stmt_error(mysqli_stmt $statement): string
{
    return $statement->error;
}

function mysqli_stmt_sqlstate(mysqli_stmt $statement): string
{
    return $statement->sqlstate;
}

function mysqli_stmt_error_list(mysqli_stmt $statement): array
{
    return $statement->error_list;
}

function mysqli_stmt_result_metadata(mysqli_stmt $statement)
{
    return $statement->result_metadata();
}

function mysqli_stmt_attr_set(mysqli_stmt $statement, int $attribute, int $value): bool
{
    return $statement->attr_set($attribute, $value);
}

function mysqli_stmt_attr_get(mysqli_stmt $statement, int $attribute): int
{
    return $statement->attr_get($attribute);
}
