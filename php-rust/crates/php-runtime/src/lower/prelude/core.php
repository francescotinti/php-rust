<?php
class stdClass {}
// unserialize() of an unknown class: the instance keeps its data plus the
// original class name in `__PHP_Incomplete_Class_Name` (set VM-side).
class __PHP_Incomplete_Class {}
// Incremental hashing (hash_init/update/final): the context buffers the fed
// data and the digest is computed at final by the one-shot hash()/hash_hmac()
// builtins (which are oracle-faithful), so every algorithm they support works
// incrementally too. Output-identical to streaming; memory-proportional to
// the fed data (fine for the workloads phpr targets).
final class HashContext {
    public $__algo = '';
    public $__buf = '';
    public $__key = null;
}
function hash_init($algo, $flags = 0, $key = '') {
    try { hash($algo, ''); } catch (ValueError $e) {
        // hash_init()'s message has no trailing `, "x" given` (unlike hash()).
        throw new ValueError('hash_init(): Argument #1 ($algo) must be a valid hashing algorithm');
    }
    $c = new HashContext;
    $c->__algo = $algo;
    if (($flags & HASH_HMAC) !== 0) {
        if ($key === '' || $key === null) {
            throw new ValueError('hash_init(): Argument #3 ($key) cannot be empty when HASH_HMAC is specified');
        }
        $c->__key = $key;
    }
    return $c;
}
function hash_update($context, $data) { $context->__buf .= $data; return true; }
function hash_update_stream($context, $stream, $length = -1) {
    $data = $length >= 0 ? stream_get_contents($stream, $length) : stream_get_contents($stream);
    if ($data === false) { return 0; }
    $context->__buf .= $data;
    return strlen($data);
}
function hash_update_file($context, $filename) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    $context->__buf .= $d;
    return true;
}
function hash_final($context, $binary = false) {
    if ($context->__key !== null) {
        return hash_hmac($context->__algo, $context->__buf, $context->__key, $binary);
    }
    return hash($context->__algo, $context->__buf, $binary);
}
function hash_copy($context) { return clone $context; }
function hash_file($algo, $filename, $binary = false) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    return hash($algo, $d, $binary);
}
function fsockopen($hostname, $port = -1, &$error_code = null, &$error_string = null, $timeout = null) {
    $r = __fsockopen((string)$hostname, (int)$port, $timeout === null ? -1.0 : (float)$timeout);
    $error_code = $r[1];
    $error_string = $r[2];
    return $r[0];
}
// phpr has no persistent-connection pool: pfsockopen connects fresh.
function pfsockopen($hostname, $port = -1, &$error_code = null, &$error_string = null, $timeout = null) {
    $r = __fsockopen((string)$hostname, (int)$port, $timeout === null ? -1.0 : (float)$timeout);
    $error_code = $r[1];
    $error_string = $r[2];
    return $r[0];
}
// ext/zlib incremental contexts: opaque PHP 8 objects wrapping an id into the
// host-side z_stream table (__deflate_/__inflate_ builtins, the curl pattern).
final class DeflateContext {
    public $__id = 0;
}
final class InflateContext {
    public $__id = 0;
}
function deflate_init(int $encoding, $options = []) {
    $id = __deflate_init($encoding, $options);
    if ($id === false) { return false; }
    $c = new DeflateContext;
    $c->__id = $id;
    return $c;
}
function deflate_add(DeflateContext $context, string $data, int $flush_mode = ZLIB_SYNC_FLUSH) {
    return __deflate_add($context->__id, $data, $flush_mode);
}
function inflate_init(int $encoding, array $options = []) {
    $id = __inflate_init($encoding, $options);
    if ($id === false) { return false; }
    $c = new InflateContext;
    $c->__id = $id;
    return $c;
}
function inflate_add(InflateContext $context, string $data, int $flush_mode = ZLIB_SYNC_FLUSH) {
    return __inflate_add($context->__id, $data, $flush_mode);
}
function inflate_get_status(InflateContext $context): int {
    return __inflate_get_status($context->__id);
}
function inflate_get_read_len(InflateContext $context): int {
    return __inflate_get_read_len($context->__id);
}
// ob_start("ob_gzhandler"): compress the buffered output per the client's
// Accept-Encoding. Without one (the CLI case) PHP's handler declines with
// false and the original output is used — byte-faithful for phpr's CLI SAPI.
function ob_gzhandler($data, $flags) {
    // Output compression needs a real web SAPI to negotiate an encoding and send
    // the Content-Encoding/Vary response headers. Under the CLI SAPI there is no
    // such channel, so PHP's handler declines unconditionally: it returns false
    // (the buffer passes through unchanged) and sends NO headers — hence no
    // "headers already sent" warning even after output has begun (bug #61820).
    // `$_SERVER['HTTP_ACCEPT_ENCODING']` is irrelevant here: the oracle leaves
    // output uncompressed under CLI regardless of it.
    return false;
}
function dir($directory, $context = null) {
    $handle = $context === null ? opendir($directory) : opendir($directory, $context);
    if ($handle === false) {
        return false;
    }
    $d = new Directory();
    $d->path = $directory;
    $d->handle = $handle;
    return $d;
}
function stream_select(&$read, &$write, &$except, $seconds, $microseconds = null) {
    $r = __stream_select($read ?? [], $write ?? [], $except ?? [], $seconds, $microseconds);
    if ($r === false) { return false; }
    $read = $r[1]; $write = $r[2]; $except = $r[3];
    return $r[0];
}
function hash_hmac_file($algo, $filename, $key, $binary = false) {
    $d = @file_get_contents($filename);
    if ($d === false) { return false; }
    return hash_hmac($algo, $d, $key, $binary);
}
// ext/curl easy API: CurlHandle (a PHP 8 object, not a resource) wraps an id
// into the host-side handle table (__curl_* in php-builtins/curl.rs, over the
// same rustls/ureq transport as the http:// stream wrapper). curl_multi_* is
// deliberately NOT defined: consumers that probe for it (Composer) fall back
// to streams; function_exists('curl_exec') consumers (monolog, Guzzle sync)
// take this path.
final class CurlHandle {
    public $__id = 0;
    // Response-sink options live PHP-side (the host builtin cannot re-enter
    // the VM): curl_exec() below feeds headers/body to them after the raw
    // transfer. Null = option unset, exactly like a fresh libcurl handle.
    public $__writefn = null;     // CURLOPT_WRITEFUNCTION
    public $__headerfn = null;    // CURLOPT_HEADERFUNCTION
    public $__file = null;        // CURLOPT_FILE (stream)
    public $__writeheader = null; // CURLOPT_WRITEHEADER (stream)
}
function curl_init($url = null) {
    $h = new CurlHandle;
    $h->__id = __curl_init();
    if ($url !== null) { curl_setopt($h, CURLOPT_URL, $url); }
    return $h;
}
function curl_setopt($handle, $option, $value) {
    switch ($option) {
        case CURLOPT_WRITEFUNCTION:  $handle->__writefn = $value; return true;
        case CURLOPT_HEADERFUNCTION: $handle->__headerfn = $value; return true;
        case CURLOPT_FILE:           $handle->__file = $value; return true;
        case CURLOPT_WRITEHEADER:    $handle->__writeheader = $value; return true;
    }
    return __curl_setopt($handle->__id, $option, $value);
}
function curl_setopt_array($handle, $options) {
    foreach ($options as $k => $v) {
        if (!curl_setopt($handle, $k, $v)) { return false; }
    }
    return true;
}
function curl_exec($handle) {
    if ($handle->__writefn === null && $handle->__headerfn === null
        && $handle->__file === null && $handle->__writeheader === null) {
        return __curl_exec($handle->__id);
    }
    // Raw mode: [header_block, body, return_transfer, include_header].
    $r = __curl_exec($handle->__id, true);
    if ($r === false) { return false; }
    [$hdr, $body, $ret, $inc] = $r;
    if ($handle->__headerfn !== null) {
        // libcurl hands the block to the callback one line at a time, line
        // terminator included, closing with the blank "\r\n" line. A short
        // return aborts the transfer with CURLE_WRITE_ERROR.
        $fn = $handle->__headerfn;
        $off = 0; $len = strlen($hdr);
        while ($off < $len) {
            $eol = strpos($hdr, "\n", $off);
            $line = $eol === false ? substr($hdr, $off) : substr($hdr, $off, $eol - $off + 1);
            $off += strlen($line);
            if ($fn($handle, $line) !== strlen($line)) {
                __curl_set_cb_error($handle->__id, 23, 'Failed writing header');
                return false;
            }
        }
    } elseif ($handle->__writeheader !== null) {
        fwrite($handle->__writeheader, $hdr);
    }
    $payload = $inc ? $hdr . $body : $body;
    if ($handle->__writefn !== null) {
        // Body reaches the callback in chunks of at most CURL_MAX_WRITE_SIZE
        // (16384); the write callback overrides RETURNTRANSFER/FILE sinks.
        $fn = $handle->__writefn;
        $off = 0; $len = strlen($payload);
        while ($off < $len) {
            $chunk = substr($payload, $off, 16384);
            $off += strlen($chunk);
            if ($fn($handle, $chunk) !== strlen($chunk)) {
                __curl_set_cb_error($handle->__id, 23, 'Failed writing received data to disk/application');
                return false;
            }
        }
        return true;
    }
    if ($handle->__file !== null) { fwrite($handle->__file, $payload); return true; }
    if ($ret) { return $payload; }
    echo $payload;
    return true;
}
function curl_errno($handle) { return __curl_errno($handle->__id); }
function curl_error($handle) { return __curl_error($handle->__id); }
function curl_getinfo($handle, $option = null) { return __curl_getinfo($handle->__id, $option); }
// curl_close() is a host builtin (no-op + 8.5 deprecation with caller attribution).
function curl_reset($handle) {
    __curl_reset($handle->__id);
    $handle->__writefn = null;
    $handle->__headerfn = null;
    $handle->__file = null;
    $handle->__writeheader = null;
}
function curl_escape($handle, $string) { return rawurlencode($string); }
function curl_unescape($handle, $string) { return rawurldecode($string); }
function curl_version() {
    // Honest facade values: version mirrors a current libcurl line so version
    // gates pass, but ssl_version/host say what the backend really is. The
    // features bitmask claims only IPV6 (1) + SSL (4) - no HTTP2/libz/brotli.
    return [
        'version_number' => 526081,
        'age' => 11,
        'features' => 5,
        'feature_list' => [
            'AsynchDNS' => false,
            'IPv6' => true,
            'SSL' => true,
            'libz' => false,
            'HTTP2' => false,
            'brotli' => false,
            'zstd' => false,
        ],
        'ssl_version_number' => 0,
        'version' => '8.7.1',
        'host' => 'phpr-rustls',
        'ssl_version' => 'rustls',
        'libz_version' => '',
        'protocols' => ['http', 'https'],
        'ares' => '',
        'ares_num' => 0,
        'libidn' => '',
        'iconv_ver_num' => 0,
        'libssh_version' => '',
        'brotli_ver_num' => 0,
        'brotli_version' => '',
    ];
}
#[Attribute(Attribute::TARGET_CLASS)]
class Attribute {
    const TARGET_CLASS = 1;
    const TARGET_FUNCTION = 2;
    const TARGET_METHOD = 4;
    const TARGET_PROPERTY = 8;
    const TARGET_CLASS_CONSTANT = 16;
    const TARGET_PARAMETER = 32;
    const TARGET_CONSTANT = 64;
    const TARGET_ALL = 127;
    const IS_REPEATABLE = 128;
    public int $flags;
    public function __construct(int $flags = self::TARGET_ALL) { $this->flags = $flags; }
}
interface UnitEnum {}
interface BackedEnum extends UnitEnum {}
// PHP 8.4 rounding mode enum (used by round() and bcround()/BcMath\Number).
enum RoundingMode {
    case HalfAwayFromZero;
    case HalfTowardsZero;
    case HalfEven;
    case HalfOdd;
    case TowardsZero;
    case AwayFromZero;
    case NegativeInfinity;
    case PositiveInfinity;
}
// ext/tokenizer: the PHP 8.0 object-oriented token (delegates to token_get_all).
class PhpToken implements Stringable {
    public int $id;
    public string $text;
    public int $line;
    public int $pos;
    final public function __construct(int $id, string $text, int $line = -1, int $pos = -1) {
        $this->id = $id;
        $this->text = $text;
        $this->line = $line;
        $this->pos = $pos;
    }
    public static function tokenize(string $code, int $flags = 0): array {
        $out = [];
        $pos = 0;
        foreach (\token_get_all($code, $flags) as $t) {
            if (\is_array($t)) {
                $out[] = new static($t[0], $t[1], $t[2], $pos);
                $pos += \strlen($t[1]);
            } else {
                $line = \substr_count(\substr($code, 0, $pos), "\n") + 1;
                $out[] = new static(\ord($t), $t, $line, $pos);
                $pos += \strlen($t);
            }
        }
        return $out;
    }
    public function is(int|string|array $kind): bool {
        if (\is_array($kind)) {
            foreach ($kind as $k) {
                if ($this->is($k)) {
                    return true;
                }
            }
            return false;
        }
        return \is_int($kind) ? $this->id === $kind : $this->text === $kind;
    }
    public function isIgnorable(): bool {
        return $this->id === 397 || $this->id === 392 || $this->id === 393 || $this->id === 394;
    }
    public function getTokenName(): ?string {
        if ($this->id < 256) {
            return \chr($this->id);
        }
        $n = \token_name($this->id);
        return $n === 'UNKNOWN' ? null : $n;
    }
    public function __toString(): string {
        return $this->text;
    }
}
// Engine interfaces carry their real method signatures (compiled as
// abstract_sigs): hasMethod/getMethods and PHPUnit interface mocks read them.
interface Stringable {
    public function __toString(): string;
}
interface Throwable {}
interface Traversable {}
interface Iterator extends Traversable {
    public function current(): mixed;
    public function key(): mixed;
    public function next(): void;
    public function rewind(): void;
    public function valid(): bool;
}
interface IteratorAggregate extends Traversable {
    public function getIterator(): Traversable;
}
interface ArrayAccess {
    public function offsetExists(mixed $offset): bool;
    public function offsetGet(mixed $offset): mixed;
    public function offsetSet(mixed $offset, mixed $value): void;
    public function offsetUnset(mixed $offset): void;
}
interface Countable {
    public function count(): int;
}
interface JsonSerializable {
    public function jsonSerialize(): mixed;
}
interface Serializable {
    public function serialize();
    public function unserialize($data);
}
class Exception implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    private $trace = [];
    private $traceString = "#0 {main}";
    public function __construct($message = "", $code = 0, $previous = null) {
        // Zend (zend_exceptions.c ctor): each slot is written ONLY when the
        // argument was supplied (message) or is non-zero/non-null (code,
        // previous) — a subclass redeclaring a property default keeps it
        // otherwise (ExceptionDataCollector's 'non-integer-code').
        if (func_num_args() >= 1 && $message !== null) { $this->message = $message; }
        if ($code) { $this->code = $code; }
        if ($previous !== null) { $this->previous = $previous; }
    }
    public function getMessage() { return $this->message; }
    public function getCode() { return $this->code; }
    public function getPrevious() { return $this->previous; }
    public function getLine() { return $this->line; }
    public function getFile() { return $this->file; }
    public function getTrace() { return $this->trace; }
    public function getTraceAsString() { return $this->traceString; }
    public function __toString() {
        $r = "";
        if ($this->previous !== null) {
            $r = $this->previous->__toString() . "\n\nNext ";
        }
        $msg = $this->message === "" ? "" : ": " . $this->message;
        $sep = (strpos($this->message, ", called in ") !== false) ? " and defined in " : " in ";
        $r .= get_class($this) . $msg . $sep . $this->file . ":" . $this->line . "\nStack trace:\n" . $this->traceString;
        return $r;
    }
}
class Error implements Throwable {
    protected $message = "";
    protected $code = 0;
    protected $file = "";
    protected $line = 0;
    private $previous = null;
    private $trace = [];
    private $traceString = "#0 {main}";
    public function __construct($message = "", $code = 0, $previous = null) {
        // Zend (zend_exceptions.c ctor): each slot is written ONLY when the
        // argument was supplied (message) or is non-zero/non-null (code,
        // previous) — a subclass redeclaring a property default keeps it
        // otherwise (ExceptionDataCollector's 'non-integer-code').
        if (func_num_args() >= 1 && $message !== null) { $this->message = $message; }
        if ($code) { $this->code = $code; }
        if ($previous !== null) { $this->previous = $previous; }
    }
    public function getMessage() { return $this->message; }
    public function getCode() { return $this->code; }
    public function getPrevious() { return $this->previous; }
    public function getLine() { return $this->line; }
    public function getFile() { return $this->file; }
    public function getTrace() { return $this->trace; }
    public function getTraceAsString() { return $this->traceString; }
    public function __toString() {
        $r = "";
        if ($this->previous !== null) {
            $r = $this->previous->__toString() . "\n\nNext ";
        }
        $msg = $this->message === "" ? "" : ": " . $this->message;
        $sep = (strpos($this->message, ", called in ") !== false) ? " and defined in " : " in ";
        $r .= get_class($this) . $msg . $sep . $this->file . ":" . $this->line . "\nStack trace:\n" . $this->traceString;
        return $r;
    }
}
class ErrorException extends Exception {
    protected $severity = E_ERROR;
    public function __construct($message = "", $code = 0, $severity = E_ERROR, $filename = null, $line = null, $previous = null) {
        parent::__construct($message, $code, $previous);
        $this->severity = $severity;
        // Absent filename/line keep the engine-stamped creation site.
        if ($filename !== null) { $this->file = $filename; }
        if ($line !== null) { $this->line = $line; }
    }
    final public function getSeverity() { return $this->severity; }
}
class LogicException extends Exception {}
class BadFunctionCallException extends LogicException {}
class BadMethodCallException extends BadFunctionCallException {}
class DomainException extends LogicException {}
class InvalidArgumentException extends LogicException {}
class LengthException extends LogicException {}
class OutOfRangeException extends LogicException {}
class RuntimeException extends Exception {}
class OutOfBoundsException extends RuntimeException {}
class OverflowException extends RuntimeException {}
class RangeException extends RuntimeException {}
class UnderflowException extends RuntimeException {}
class UnexpectedValueException extends RuntimeException {}
class JsonException extends Exception {}
class PharException extends Exception {}
class TypeError extends Error {}
class ArgumentCountError extends TypeError {}
class ValueError extends Error {}
class ArithmeticError extends Error {}
class DivisionByZeroError extends ArithmeticError {}
class UnhandledMatchError extends Error {}
class AssertionError extends Error {}
class CompileError extends Error {}
class ParseError extends CompileError {}
class Fiber {
    private $callable;
    public function __construct($callable) { $this->callable = $callable; }
}
final class WeakReference {
    // `__h` is an internal weak handle (see __weak_create/__weak_get): it does
    // NOT keep the referent alive, so get() returns the object while a strong
    // reference exists elsewhere and null once it is collected (true weakness).
    private $__h;
    private function __construct() {}
    public static function create($object) {
        if (!is_object($object)) {
            $t = gettype($object);
            $t = ["integer" => "int", "double" => "float", "boolean" => "bool", "NULL" => "null"][$t] ?? $t;
            throw new TypeError("WeakReference::create(): Argument #1 (\$object) must be of type object, $t given");
        }
        $ref = new self();
        $ref->__h = __weak_create($object);
        return $ref;
    }
    public function get() {
        return __weak_get($this->__h);
    }
}
class WeakMap implements ArrayAccess, Countable, IteratorAggregate {
    // id => [weak-handle, value]. Keys are held *weakly* (via __weak_create): an
    // entry whose key has been collected is pruned lazily on access (__prune /
    // __live), giving true weakness without a tracing GC. Keyed by spl_object_id.
    private $__entries = [];
    private function __live($id) {
        // The live key object for $id, pruning the entry if it has been collected.
        if (!isset($this->__entries[$id])) {
            return null;
        }
        $o = __weak_get($this->__entries[$id][0]);
        if ($o === null) {
            unset($this->__entries[$id]);
        }
        return $o;
    }
    private function __prune() {
        foreach ($this->__entries as $id => $entry) {
            if (__weak_get($entry[0]) === null) {
                unset($this->__entries[$id]);
            }
        }
    }
    public function offsetExists($object) {
        // isset()/empty() on an ArrayAccess element use offsetExists as the
        // backend; PHP reports a null-valued (or collected) key as not set.
        $id = spl_object_id($object);
        return $this->__live($id) !== null && $this->__entries[$id][1] !== null;
    }
    public function offsetGet($object) {
        if (!is_object($object)) {
            throw new TypeError("WeakMap key must be an object");
        }
        $id = spl_object_id($object);
        if ($this->__live($id) === null) {
            throw new Error("Object " . get_class($object) . "#" . $id . " not contained in WeakMap");
        }
        return $this->__entries[$id][1];
    }
    public function offsetSet($object, $value) {
        if (!is_object($object)) {
            throw new TypeError("WeakMap key must be an object");
        }
        $this->__entries[spl_object_id($object)] = [__weak_create($object), $value];
    }
    public function offsetUnset($object) {
        unset($this->__entries[spl_object_id($object)]);
    }
    public function count() {
        $this->__prune();
        return count($this->__entries);
    }
    public function getIterator() {
        $this->__prune();
        foreach ($this->__entries as $entry) {
            $o = __weak_get($entry[0]);
            if ($o !== null) {
                yield $o => $entry[1];
            }
        }
    }
}

// --- SPL iterator classes (step 56): the two by-far most-demanded SPL types in
// the Zend/tests corpus (ArrayIterator 32 files, ArrayObject 28). Implemented
// entirely in PHP, backed by a plain `array $__storage`, reusing the working
// Iterator + ArrayAccess protocols + the array builtins. Zero VM changes.
// `__keys` is a key snapshot taken at rewind() so the integer `__pos` cursor is
// order-preserving and survives mutation, matching SPL semantics.
// ZipArchive (ext/zip subset backing Composer's dist downloads): the prelude
// class delegates to the __zip_* host builtins (zip crate VM-side); the handle
// is an int id in Vm.zips. Read-only surface: open/close/count/statIndex/
// getNameIndex/locateName/getFromIndex/getFromName/extractTo. No writing.
class ZipArchive implements Countable {
    const CREATE = 1; const EXCL = 2; const CHECKCONS = 4; const OVERWRITE = 8; const RDONLY = 16;
    const FL_NOCASE = 1; const FL_NODIR = 2;
    const CM_DEFAULT = -1; const CM_STORE = 0; const CM_DEFLATE = 8;
    const EM_NONE = 0;
    const ER_OK = 0; const ER_MULTIDISK = 1; const ER_RENAME = 2; const ER_CLOSE = 3;
    const ER_SEEK = 4; const ER_READ = 5; const ER_WRITE = 6; const ER_CRC = 7;
    const ER_ZIPCLOSED = 8; const ER_NOENT = 9; const ER_EXISTS = 10; const ER_OPEN = 11;
    const ER_TMPOPEN = 12; const ER_ZLIB = 13; const ER_MEMORY = 14; const ER_CHANGED = 15;
    const ER_COMPNOTSUPP = 16; const ER_EOF = 17; const ER_INVAL = 18; const ER_NOZIP = 19;
    const ER_INTERNAL = 20; const ER_INCONS = 21; const ER_REMOVE = 22; const ER_DELETED = 23;
    public $numFiles = 0;
    public $status = 0;
    public $statusSys = 0;
    public $filename = '';
    public $comment = '';
    private $__h = null;
    private $__w = false;
    public function open($filename, $flags = 0) {
        // CREATE/OVERWRITE on a missing (or OVERWRITE'd) file opens a fresh
        // WRITE-mode archive backed by __zip_writer_* (WP privacy export).
        if (($flags & self::OVERWRITE) || (($flags & self::CREATE) && !file_exists($filename))) {
            $r = __zip_writer_open($filename);
            if (is_int($r)) { $this->status = $r; return $r; }
            $this->__h = $r[0];
            $this->__w = true;
            $this->numFiles = 0;
            $this->filename = $filename;
            $this->status = 0;
            return true;
        }
        $r = __zip_open($filename);
        if (is_int($r)) { $this->status = $r; return $r; }
        $this->__h = $r[0];
        $this->__w = false;
        $this->numFiles = $r[1];
        $this->filename = $filename;
        $this->status = 0;
        return true;
    }
    public function addFile($filepath, $entryname = null, $start = 0, $length = 0, $flags = 0) {
        if ($this->__h === null || !$this->__w) { return false; }
        $data = @file_get_contents($filepath);
        if ($data === false) { return false; }
        $name = ($entryname === null || $entryname === '') ? basename($filepath) : $entryname;
        if (!__zip_writer_add($this->__h, $name, $data)) { return false; }
        $this->numFiles++;
        return true;
    }
    public function addFromString($name, $content, $flags = 0) {
        if ($this->__h === null || !$this->__w) { return false; }
        if (!__zip_writer_add($this->__h, $name, $content)) { return false; }
        $this->numFiles++;
        return true;
    }
    public function close() {
        if ($this->__h === null) { return false; }
        $r = $this->__w ? __zip_writer_close($this->__h) : __zip_close($this->__h);
        $this->__h = null; $this->__w = false; $this->numFiles = 0; $this->filename = '';
        return $r;
    }
    public function count(): int { return $this->numFiles; }
    public function statIndex($index, $flags = 0) { return $this->__h === null ? false : __zip_stat_index($this->__h, $index); }
    public function getNameIndex($index, $flags = 0) { return $this->__h === null ? false : __zip_get_name_index($this->__h, $index); }
    public function locateName($name, $flags = 0) { return $this->__h === null ? false : __zip_locate_name($this->__h, $name); }
    public function getFromIndex($index, $len = 0, $flags = 0) { return $this->__h === null ? false : __zip_get_from_index($this->__h, $index); }
    public function getFromName($name, $len = 0, $flags = 0) {
        if ($this->__h === null) { return false; }
        $i = __zip_locate_name($this->__h, $name);
        return $i === false ? false : __zip_get_from_index($this->__h, $i);
    }
    public function extractTo($pathto, $files = null) { return $this->__h === null ? false : __zip_extract_to($this->__h, $pathto); }
    public function getStatusString() { return $this->status === 0 ? 'No error' : 'Unknown error ' . $this->status; }
}
function pdo_drivers() { return PDO::getAvailableDrivers(); }
function enum_exists($enum, $autoload = true) {
    // Reuse class_exists for the (autoload-aware) existence check, then confirm
    // the class is an enum via its implicit UnitEnum interface.
    return class_exists($enum, $autoload) && in_array('UnitEnum', class_implements($enum));
}

// `preg_replace_callback_array(['/rx/' => cb, ...], $subject)`: sequential
// preg_replace_callback over the pattern map (the replacement-count out-param
// stays unreported, matching phpr's preg_replace_callback).
function preg_replace_callback_array($pattern, $subject, $limit = -1, &$count = null, $flags = 0) {
    $count = 0;
    foreach ($pattern as $rx => $cb) {
        $subject = preg_replace_callback($rx, $cb, $subject, $limit);
        if ($subject === null) { return null; }
    }
    return $subject;
}
function is_countable($value) { return is_array($value) || $value instanceof Countable; }
