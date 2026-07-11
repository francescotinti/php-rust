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
// The directory OOP surface: dir()/getdir() return a Directory whose read()/
// rewind()/close() delegate to the procedural readdir/rewinddir/closedir over a
// stored opendir() handle. Property order (path, handle) and the exposed handle
// resource match the internal class var_dump byte-for-byte.
class Directory {
    public $path = '';
    public $handle;
    public function read($dir_handle = null) {
        return readdir($dir_handle ?? $this->handle);
    }
    public function rewind($dir_handle = null) {
        rewinddir($dir_handle ?? $this->handle);
    }
    public function close($dir_handle = null) {
        closedir($dir_handle ?? $this->handle);
    }
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
}
function curl_init($url = null) {
    $h = new CurlHandle;
    $h->__id = __curl_init();
    if ($url !== null) { curl_setopt($h, CURLOPT_URL, $url); }
    return $h;
}
function curl_setopt($handle, $option, $value) { return __curl_setopt($handle->__id, $option, $value); }
function curl_setopt_array($handle, $options) {
    foreach ($options as $k => $v) {
        if (!curl_setopt($handle, $k, $v)) { return false; }
    }
    return true;
}
function curl_exec($handle) { return __curl_exec($handle->__id); }
function curl_errno($handle) { return __curl_errno($handle->__id); }
function curl_error($handle) { return __curl_error($handle->__id); }
function curl_getinfo($handle, $option = null) { return __curl_getinfo($handle->__id, $option); }
// curl_close() is a host builtin (no-op + 8.5 deprecation with caller attribution).
function curl_reset($handle) { __curl_reset($handle->__id); }
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
interface SeekableIterator extends Iterator {
    public function seek(int $offset);
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
        $this->message = $message;
        $this->code = $code;
        $this->previous = $previous;
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
        $this->message = $message;
        $this->code = $code;
        $this->previous = $previous;
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
class ErrorException extends Exception {}
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
interface DateTimeInterface {
    const ATOM = 'Y-m-d\TH:i:sP';
    const COOKIE = 'l, d-M-Y H:i:s T';
    const ISO8601 = 'Y-m-d\TH:i:sO';
    const ISO8601_EXPANDED = 'X-m-d\TH:i:sP';
    const RFC822 = 'D, d M y H:i:s O';
    const RFC850 = 'l, d-M-y H:i:s T';
    const RFC1036 = 'D, d M y H:i:s O';
    const RFC1123 = 'D, d M Y H:i:s O';
    const RFC7231 = 'D, d M Y H:i:s \G\M\T';
    const RFC2822 = 'D, d M Y H:i:s O';
    const RFC3339 = 'Y-m-d\TH:i:sP';
    const RFC3339_EXTENDED = 'Y-m-d\TH:i:s.vP';
    const RSS = 'D, d M Y H:i:s O';
    const W3C = 'Y-m-d\TH:i:sP';
}
// phpr models instants as UTC unix timestamps, so a timezone is carried for
// `getName()`/`getTimezone()` but does not shift the stored timestamp (faithful
// for the UTC zone Composer uses; tz-aware display is deliberately out of scope).
class DateTimeZone {
    const AFRICA = 1;
    const AMERICA = 2;
    const ANTARCTICA = 4;
    const ARCTIC = 8;
    const ASIA = 16;
    const ATLANTIC = 32;
    const AUSTRALIA = 64;
    const EUROPE = 128;
    const INDIAN = 256;
    const PACIFIC = 512;
    const UTC = 1024;
    const ALL = 2047;
    const ALL_WITH_BC = 4095;
    const PER_COUNTRY = 4096;
    private $__name = "UTC";
    public function __construct($timezone = "UTC") { $this->__name = (string)$timezone; }
    public function getName() { return $this->__name; }
    public function __toString() { return $this->__name; }
    // The oracle's 419 identifiers (macOS tzdata), comma-packed to keep the
    // prelude compact. Group/country filtering is not modelled: real consumers
    // (monolog's setTimezoneProvider) call it bare.
    public static function listIdentifiers($timezoneGroup = DateTimeZone::ALL, $countryCode = null) {
        return explode(',', 'Africa/Abidjan,Africa/Accra,Africa/Addis_Ababa,Africa/Algiers,Africa/Asmara,Africa/Bamako,Africa/Bangui,Africa/Banjul,Africa/Bissau,Africa/Blantyre,Africa/Brazzaville,Africa/Bujumbura,Africa/Cairo,Africa/Casablanca,Africa/Ceuta,Africa/Conakry,Africa/Dakar,Africa/Dar_es_Salaam,Africa/Djibouti,Africa/Douala,Africa/El_Aaiun,Africa/Freetown,Africa/Gaborone,Africa/Harare,Africa/Johannesburg,Africa/Juba,Africa/Kampala,Africa/Khartoum,Africa/Kigali,Africa/Kinshasa,Africa/Lagos,Africa/Libreville,Africa/Lome,Africa/Luanda,Africa/Lubumbashi,Africa/Lusaka,Africa/Malabo,Africa/Maputo,Africa/Maseru,Africa/Mbabane,Africa/Mogadishu,Africa/Monrovia,Africa/Nairobi,Africa/Ndjamena,Africa/Niamey,Africa/Nouakchott,Africa/Ouagadougou,Africa/Porto-Novo,Africa/Sao_Tome,Africa/Tripoli,Africa/Tunis,Africa/Windhoek,America/Adak,America/Anchorage,America/Anguilla,America/Antigua,America/Araguaina,America/Argentina/Buenos_Aires,America/Argentina/Catamarca,America/Argentina/Cordoba,America/Argentina/Jujuy,America/Argentina/La_Rioja,America/Argentina/Mendoza,America/Argentina/Rio_Gallegos,America/Argentina/Salta,America/Argentina/San_Juan,America/Argentina/San_Luis,America/Argentina/Tucuman,America/Argentina/Ushuaia,America/Aruba,America/Asuncion,America/Atikokan,America/Bahia,America/Bahia_Banderas,America/Barbados,America/Belem,America/Belize,America/Blanc-Sablon,America/Boa_Vista,America/Bogota,America/Boise,America/Cambridge_Bay,America/Campo_Grande,America/Cancun,America/Caracas,America/Cayenne,America/Cayman,America/Chicago,America/Chihuahua,America/Ciudad_Juarez,America/Costa_Rica,America/Coyhaique,America/Creston,America/Cuiaba,America/Curacao,America/Danmarkshavn,America/Dawson,America/Dawson_Creek,America/Denver,America/Detroit,America/Dominica,America/Edmonton,America/Eirunepe,America/El_Salvador,America/Fort_Nelson,America/Fortaleza,America/Glace_Bay,America/Goose_Bay,America/Grand_Turk,America/Grenada,America/Guadeloupe,America/Guatemala,America/Guayaquil,America/Guyana,America/Halifax,America/Havana,America/Hermosillo,America/Indiana/Indianapolis,America/Indiana/Knox,America/Indiana/Marengo,America/Indiana/Petersburg,America/Indiana/Tell_City,America/Indiana/Vevay,America/Indiana/Vincennes,America/Indiana/Winamac,America/Inuvik,America/Iqaluit,America/Jamaica,America/Juneau,America/Kentucky/Louisville,America/Kentucky/Monticello,America/Kralendijk,America/La_Paz,America/Lima,America/Los_Angeles,America/Lower_Princes,America/Maceio,America/Managua,America/Manaus,America/Marigot,America/Martinique,America/Matamoros,America/Mazatlan,America/Menominee,America/Merida,America/Metlakatla,America/Mexico_City,America/Miquelon,America/Moncton,America/Monterrey,America/Montevideo,America/Montserrat,America/Nassau,America/New_York,America/Nome,America/Noronha,America/North_Dakota/Beulah,America/North_Dakota/Center,America/North_Dakota/New_Salem,America/Nuuk,America/Ojinaga,America/Panama,America/Paramaribo,America/Phoenix,America/Port-au-Prince,America/Port_of_Spain,America/Porto_Velho,America/Puerto_Rico,America/Punta_Arenas,America/Rankin_Inlet,America/Recife,America/Regina,America/Resolute,America/Rio_Branco,America/Santarem,America/Santiago,America/Santo_Domingo,America/Sao_Paulo,America/Scoresbysund,America/Sitka,America/St_Barthelemy,America/St_Johns,America/St_Kitts,America/St_Lucia,America/St_Thomas,America/St_Vincent,America/Swift_Current,America/Tegucigalpa,America/Thule,America/Tijuana,America/Toronto,America/Tortola,America/Vancouver,America/Whitehorse,America/Winnipeg,America/Yakutat,Antarctica/Casey,Antarctica/Davis,Antarctica/DumontDUrville,Antarctica/Macquarie,Antarctica/Mawson,Antarctica/McMurdo,Antarctica/Palmer,Antarctica/Rothera,Antarctica/Syowa,Antarctica/Troll,Antarctica/Vostok,Arctic/Longyearbyen,Asia/Aden,Asia/Almaty,Asia/Amman,Asia/Anadyr,Asia/Aqtau,Asia/Aqtobe,Asia/Ashgabat,Asia/Atyrau,Asia/Baghdad,Asia/Bahrain,Asia/Baku,Asia/Bangkok,Asia/Barnaul,Asia/Beirut,Asia/Bishkek,Asia/Brunei,Asia/Chita,Asia/Colombo,Asia/Damascus,Asia/Dhaka,Asia/Dili,Asia/Dubai,Asia/Dushanbe,Asia/Famagusta,Asia/Gaza,Asia/Hebron,Asia/Ho_Chi_Minh,Asia/Hong_Kong,Asia/Hovd,Asia/Irkutsk,Asia/Jakarta,Asia/Jayapura,Asia/Jerusalem,Asia/Kabul,Asia/Kamchatka,Asia/Karachi,Asia/Kathmandu,Asia/Khandyga,Asia/Kolkata,Asia/Krasnoyarsk,Asia/Kuala_Lumpur,Asia/Kuching,Asia/Kuwait,Asia/Macau,Asia/Magadan,Asia/Makassar,Asia/Manila,Asia/Muscat,Asia/Nicosia,Asia/Novokuznetsk,Asia/Novosibirsk,Asia/Omsk,Asia/Oral,Asia/Phnom_Penh,Asia/Pontianak,Asia/Pyongyang,Asia/Qatar,Asia/Qostanay,Asia/Qyzylorda,Asia/Riyadh,Asia/Sakhalin,Asia/Samarkand,Asia/Seoul,Asia/Shanghai,Asia/Singapore,Asia/Srednekolymsk,Asia/Taipei,Asia/Tashkent,Asia/Tbilisi,Asia/Tehran,Asia/Thimphu,Asia/Tokyo,Asia/Tomsk,Asia/Ulaanbaatar,Asia/Urumqi,Asia/Ust-Nera,Asia/Vientiane,Asia/Vladivostok,Asia/Yakutsk,Asia/Yangon,Asia/Yekaterinburg,Asia/Yerevan,Atlantic/Azores,Atlantic/Bermuda,Atlantic/Canary,Atlantic/Cape_Verde,Atlantic/Faroe,Atlantic/Madeira,Atlantic/Reykjavik,Atlantic/South_Georgia,Atlantic/St_Helena,Atlantic/Stanley,Australia/Adelaide,Australia/Brisbane,Australia/Broken_Hill,Australia/Darwin,Australia/Eucla,Australia/Hobart,Australia/Lindeman,Australia/Lord_Howe,Australia/Melbourne,Australia/Perth,Australia/Sydney,Europe/Amsterdam,Europe/Andorra,Europe/Astrakhan,Europe/Athens,Europe/Belgrade,Europe/Berlin,Europe/Bratislava,Europe/Brussels,Europe/Bucharest,Europe/Budapest,Europe/Busingen,Europe/Chisinau,Europe/Copenhagen,Europe/Dublin,Europe/Gibraltar,Europe/Guernsey,Europe/Helsinki,Europe/Isle_of_Man,Europe/Istanbul,Europe/Jersey,Europe/Kaliningrad,Europe/Kirov,Europe/Kyiv,Europe/Lisbon,Europe/Ljubljana,Europe/London,Europe/Luxembourg,Europe/Madrid,Europe/Malta,Europe/Mariehamn,Europe/Minsk,Europe/Monaco,Europe/Moscow,Europe/Oslo,Europe/Paris,Europe/Podgorica,Europe/Prague,Europe/Riga,Europe/Rome,Europe/Samara,Europe/San_Marino,Europe/Sarajevo,Europe/Saratov,Europe/Simferopol,Europe/Skopje,Europe/Sofia,Europe/Stockholm,Europe/Tallinn,Europe/Tirane,Europe/Ulyanovsk,Europe/Vaduz,Europe/Vatican,Europe/Vienna,Europe/Vilnius,Europe/Volgograd,Europe/Warsaw,Europe/Zagreb,Europe/Zurich,Indian/Antananarivo,Indian/Chagos,Indian/Christmas,Indian/Cocos,Indian/Comoro,Indian/Kerguelen,Indian/Mahe,Indian/Maldives,Indian/Mauritius,Indian/Mayotte,Indian/Reunion,Pacific/Apia,Pacific/Auckland,Pacific/Bougainville,Pacific/Chatham,Pacific/Chuuk,Pacific/Easter,Pacific/Efate,Pacific/Fakaofo,Pacific/Fiji,Pacific/Funafuti,Pacific/Galapagos,Pacific/Gambier,Pacific/Guadalcanal,Pacific/Guam,Pacific/Honolulu,Pacific/Kanton,Pacific/Kiritimati,Pacific/Kosrae,Pacific/Kwajalein,Pacific/Majuro,Pacific/Marquesas,Pacific/Midway,Pacific/Nauru,Pacific/Niue,Pacific/Norfolk,Pacific/Noumea,Pacific/Pago_Pago,Pacific/Palau,Pacific/Pitcairn,Pacific/Pohnpei,Pacific/Port_Moresby,Pacific/Rarotonga,Pacific/Saipan,Pacific/Tahiti,Pacific/Tarawa,Pacific/Tongatapu,Pacific/Wake,Pacific/Wallis,UTC');
    }
}
class DateTime implements DateTimeInterface {
    private $__ts = 0;
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        if ($timezone !== null) { $this->__tz = $timezone->getName(); }
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            // A leading '@' (unix timestamp) forces the UTC-offset zone "+00:00",
            // ignoring any passed timezone (a PHP quirk).
            if (is_string($datetime) && isset($datetime[0]) && $datetime[0] === "@") {
                $this->__tz = "+00:00";
            }
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = strtotime($parse);
            if ($r === false) {
                throw new Exception("DateTime::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function setTimezone($timezone) {
        $this->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $this;
    }
    public static function createFromInterface($object) {
        $d = new DateTime("@" . $object->getTimestamp());
        return $d->setTimezone($object->getTimezone());
    }
    public static function createFromImmutable($object) { return static::createFromInterface($object); }
    public function format($format) {
        // 'u'/'v' (micro/milliseconds) come from this instance, not date():
        // substitute the digits as backslash-escaped literals in the format.
        $out = ''; $esc = false;
        for ($i = 0, $len = strlen($format); $i < $len; $i++) {
            $c = $format[$i];
            if ($esc) { $out .= '\\' . $c; $esc = false; continue; }
            if ($c === '\\') { $esc = true; continue; }
            if ($c === 'u' || $c === 'v') {
                $n = $c === 'u' ? sprintf('%06d', $this->__us) : sprintf('%03d', intdiv($this->__us, 1000));
                foreach (str_split($n) as $d) { $out .= '\\' . $d; }
                continue;
            }
            $out .= $c;
        }
        return date($out, $this->__ts);
    }
    public function getTimestamp() { return $this->__ts; }
    public function setTimestamp($timestamp) { $this->__ts = $timestamp; return $this; }
    public function setDate($year, $month, $day) {
        $this->__ts = mktime((int)date('G', $this->__ts), (int)date('i', $this->__ts), (int)date('s', $this->__ts), $month, $day, $year);
        return $this;
    }
    public function setTime($hour, $minute, $second = 0) {
        $this->__ts = mktime($hour, $minute, $second, (int)date('n', $this->__ts), (int)date('j', $this->__ts), (int)date('Y', $this->__ts));
        return $this;
    }
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        $d = new DateTime("@" . $r[0]);
        // "@ts" leaves the "+00:00" offset tz; the real createFromFormat keeps
        // the parsed offset, the $timezone argument, or the default (UTC).
        $d->__tz = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : 'UTC');
        $d->__us = $r[2];
        return $d;
    }
    public function modify($modifier) { $this->__ts = strtotime($modifier, $this->__ts); return $this; }
    public function add($interval) { $this->__ts = $this->__apply($interval, 1); return $this; }
    public function sub($interval) { $this->__ts = $this->__apply($interval, -1); return $this; }
    private function __apply($iv, $dir) {
        $sign = $dir * ($iv->invert ? -1 : 1);
        return mktime(
            (int)date('G', $this->__ts) + $sign * $iv->h,
            (int)date('i', $this->__ts) + $sign * $iv->i,
            (int)date('s', $this->__ts) + $sign * $iv->s,
            (int)date('n', $this->__ts) + $sign * $iv->m,
            (int)date('j', $this->__ts) + $sign * $iv->d,
            (int)date('Y', $this->__ts) + $sign * $iv->y);
    }
    public function diff($other) {
        $info = __date_diff($this->__ts, $other->getTimestamp());
        $iv = new DateInterval("PT0S");
        $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $info['d'];
        $iv->h = $info['h']; $iv->i = $info['i']; $iv->s = $info['s'];
        $iv->invert = $info['invert']; $iv->days = $info['days'];
        return $iv;
    }
}
class DateInterval {
    public $y = 0;
    public $m = 0;
    public $d = 0;
    public $h = 0;
    public $i = 0;
    public $s = 0;
    public $f = 0;
    public $invert = 0;
    public $days = false;
    public function __construct($duration) {
        $p = __interval_parse($duration);
        if ($p === false) {
            throw new Exception("DateInterval::__construct(): Unknown or bad format ($duration)");
        }
        $this->y = $p['y']; $this->m = $p['m']; $this->d = $p['d'];
        $this->h = $p['h']; $this->i = $p['i']; $this->s = $p['s'];
    }
    public function format($format) { return __interval_format($this, $format); }
}
class DateTimeImmutable implements DateTimeInterface {
    private $__ts = 0;
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        if ($timezone !== null) { $this->__tz = $timezone->getName(); }
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            // A leading '@' (unix timestamp) forces the UTC-offset zone "+00:00".
            if (is_string($datetime) && isset($datetime[0]) && $datetime[0] === "@") {
                $this->__tz = "+00:00";
            }
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = strtotime($parse);
            if ($r === false) {
                throw new Exception("DateTimeImmutable::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r;
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function setTimezone($timezone) {
        // `clone` keeps the runtime class, so a userland subclass (monolog's
        // JsonSerializableDateTimeImmutable) survives, like PHP's `static`.
        $c = clone $this;
        $c->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $c;
    }
    public static function createFromInterface($object) {
        $d = new DateTimeImmutable("@" . $object->getTimestamp());
        return $d->setTimezone($object->getTimezone());
    }
    public static function createFromMutable($object) { return static::createFromInterface($object); }
    public function format($format) {
        // 'u'/'v' (micro/milliseconds) come from this instance, not date():
        // substitute the digits as backslash-escaped literals in the format.
        $out = ''; $esc = false;
        for ($i = 0, $len = strlen($format); $i < $len; $i++) {
            $c = $format[$i];
            if ($esc) { $out .= '\\' . $c; $esc = false; continue; }
            if ($c === '\\') { $esc = true; continue; }
            if ($c === 'u' || $c === 'v') {
                $n = $c === 'u' ? sprintf('%06d', $this->__us) : sprintf('%03d', intdiv($this->__us, 1000));
                foreach (str_split($n) as $d) { $out .= '\\' . $d; }
                continue;
            }
            $out .= $c;
        }
        return date($out, $this->__ts);
    }
    public function getTimestamp() { return $this->__ts; }
    // Every "wither" clones: the runtime class survives (PHP returns `static`,
    // monolog's JsonSerializableDateTimeImmutable relies on it), and so do
    // the timezone label and (where PHP keeps them) the microseconds.
    public function setTimestamp($timestamp) {
        $c = clone $this; $c->__ts = $timestamp; $c->__us = 0; return $c;
    }
    public function setDate($year, $month, $day) {
        $c = clone $this;
        $c->__ts = mktime((int)date('G', $this->__ts), (int)date('i', $this->__ts), (int)date('s', $this->__ts), $month, $day, $year);
        return $c;
    }
    public function setTime($hour, $minute, $second = 0, $microsecond = 0) {
        $c = clone $this;
        $c->__ts = mktime($hour, $minute, $second, (int)date('n', $this->__ts), (int)date('j', $this->__ts), (int)date('Y', $this->__ts));
        $c->__us = $microsecond;
        return $c;
    }
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        $d = new DateTimeImmutable("@" . $r[0]);
        $d->__tz = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : 'UTC');
        $d->__us = $r[2];
        return $d;
    }
    public function modify($modifier) {
        $r = strtotime($modifier, $this->__ts);
        if ($r === false) { return false; }
        $c = clone $this; $c->__ts = $r; return $c;
    }
    public function add($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, 1); return $c; }
    public function sub($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, -1); return $c; }
    private function __apply($iv, $dir) {
        $sign = $dir * ($iv->invert ? -1 : 1);
        return mktime(
            (int)date('G', $this->__ts) + $sign * $iv->h,
            (int)date('i', $this->__ts) + $sign * $iv->i,
            (int)date('s', $this->__ts) + $sign * $iv->s,
            (int)date('n', $this->__ts) + $sign * $iv->m,
            (int)date('j', $this->__ts) + $sign * $iv->d,
            (int)date('Y', $this->__ts) + $sign * $iv->y);
    }
    public function diff($other) {
        $info = __date_diff($this->__ts, $other->getTimestamp());
        $iv = new DateInterval("PT0S");
        $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $info['d'];
        $iv->h = $info['h']; $iv->i = $info['i']; $iv->s = $info['s'];
        $iv->invert = $info['invert']; $iv->days = $info['days'];
        return $iv;
    }
}

// --- Procedural date API (step 35): thin global-function wrappers over the OOP
// API above. PHP exposes both styles; these delegate so the two stay identical.
function date_create($datetime = "now") { return new DateTime($datetime); }
function date_create_immutable($datetime = "now") { return new DateTimeImmutable($datetime); }
function date_format($object, $format) { return $object->format($format); }
function date_timestamp_get($object) { return $object->getTimestamp(); }
function date_diff($base, $target, $absolute = false) {
    $r = $base->diff($target);
    if ($absolute) { $r->invert = 0; }
    return $r;
}
function date_add($object, $interval) { return $object->add($interval); }
function date_sub($object, $interval) { return $object->sub($interval); }
function date_modify($object, $modifier) { return $object->modify($modifier); }
function date_date_set($object, $year, $month, $day) { return $object->setDate($year, $month, $day); }
function date_time_set($object, $hour, $minute, $second = 0) { return $object->setTime($hour, $minute, $second); }
function date_timestamp_set($object, $timestamp) { return $object->setTimestamp($timestamp); }
function date_create_from_format($format, $datetime, $timezone = null) { return DateTime::createFromFormat($format, $datetime); }
function date_create_immutable_from_format($format, $datetime, $timezone = null) { return DateTimeImmutable::createFromFormat($format, $datetime); }
function date_interval_format($object, $format) { return $object->format($format); }
function date_interval_create_from_date_string($datetime) {
    $p = __interval_from_date_string($datetime);
    if ($p === false) { return false; }
    $iv = new DateInterval("PT0S");
    $iv->y = $p['y']; $iv->m = $p['m']; $iv->d = $p['d'];
    $iv->h = $p['h']; $iv->i = $p['i']; $iv->s = $p['s'];
    return $iv;
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
    public function open($filename, $flags = 0) {
        $r = __zip_open($filename);
        if (is_int($r)) { $this->status = $r; return $r; }
        $this->__h = $r[0];
        $this->numFiles = $r[1];
        $this->filename = $filename;
        $this->status = 0;
        return true;
    }
    public function close() {
        if ($this->__h === null) { return false; }
        $r = __zip_close($this->__h);
        $this->__h = null; $this->numFiles = 0; $this->filename = '';
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
function pdo_drivers() { return PDO::getAvailableDrivers(); }
// SPL: iteratore vuoto e filtro-regex (doctrine/persistence FileClassLocator
// filtra i file di mapping con RegexIterator::MATCH; GET_MATCH sostituisce
// current() con l'array dei match; le chiavi dell'inner sono preservate).
class EmptyIterator implements Iterator {
    public function current(): mixed { throw new BadMethodCallException('Accessing the value of an EmptyIterator'); }
    public function key(): mixed { throw new BadMethodCallException('Accessing the key of an EmptyIterator'); }
    public function next(): void {}
    public function rewind(): void {}
    public function valid(): bool { return false; }
}
class RegexIterator implements Iterator {
    const USE_KEY = 1;
    const INVERT_MATCH = 2;
    const MATCH = 0;
    const GET_MATCH = 1;
    const ALL_MATCHES = 2;
    const SPLIT = 3;
    const REPLACE = 4;
    public $replacement = null;
    private $__it;
    private $__regex;
    private $__mode;
    private $__flags;
    private $__pregFlags;
    private $__cur;
    public function __construct($iterator, $pattern, $mode = self::MATCH, $flags = 0, $pregFlags = 0) {
        $this->__it = $iterator;
        $this->__regex = $pattern;
        $this->__mode = $mode;
        $this->__flags = $flags;
        $this->__pregFlags = $pregFlags;
    }
    public function getInnerIterator() { return $this->__it; }
    public function getRegex() { return $this->__regex; }
    public function getMode() { return $this->__mode; }
    public function getFlags() { return $this->__flags; }
    private function __advance() {
        while ($this->__it->valid()) {
            $v = $this->__it->current();
            $subject = ($this->__flags & self::USE_KEY) === self::USE_KEY ? $this->__it->key() : $v;
            $m = array();
            if ($this->__mode === self::ALL_MATCHES) {
                $ok = preg_match_all($this->__regex, (string)$subject, $m, $this->__pregFlags) > 0;
            } else {
                $ok = preg_match($this->__regex, (string)$subject, $m) === 1;
            }
            if (($this->__flags & self::INVERT_MATCH) === self::INVERT_MATCH) { $ok = !$ok; }
            if ($ok) {
                if ($this->__mode === self::GET_MATCH || $this->__mode === self::ALL_MATCHES) { $this->__cur = $m; }
                elseif ($this->__mode === self::SPLIT) { $this->__cur = preg_split($this->__regex, (string)$subject, -1, $this->__pregFlags); }
                elseif ($this->__mode === self::REPLACE) { $this->__cur = preg_replace($this->__regex, (string)$this->replacement, (string)$subject); }
                else { $this->__cur = $v; }
                return;
            }
            $this->__it->next();
        }
    }
    public function rewind(): void { $this->__it->rewind(); $this->__advance(); }
    public function valid(): bool { return $this->__it->valid(); }
    public function current(): mixed { return $this->__cur; }
    public function key(): mixed { return $this->__it->key(); }
    public function next(): void { $this->__it->next(); $this->__advance(); }
}
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
// SplFileInfo + the directory-iterator family (Composer's Filesystem: rm -rf
// via CHILD_FIRST, copy-tree via SELF_FIRST + getSubPathname). The recursive
// iterator SNAPSHOTS the traversal at rewind() -- Composer's uses (delete after
// yield, copy) are order-compatible; live-mutation semantics, CATCH_GET_CHILD
// and the flag combinations beyond SKIP_DOTS/CURRENT_AS_PATHNAME are residues.
class SplFileInfo {
    protected $__path;
    public function __construct($path) { $this->__path = $path; }
    public function getPathname() { return $this->__path; }
    public function isDir() { return is_dir($this->__path); }
    public function isFile() { return is_file($this->__path); }
    public function isLink() { return is_link($this->__path); }
    public function getFilename() { return basename($this->__path); }
    public function getBasename($suffix = '') { return basename($this->__path, $suffix); }
    public function getPath() { return dirname($this->__path); }
    public function getRealPath() { return realpath($this->__path); }
    public function getSize() { return filesize($this->__path); }
    public function getPerms() { return fileperms($this->__path); }
    public function getMTime() { return filemtime($this->__path); }
    public function isReadable() { return is_readable($this->__path); }
    public function isWritable() { return is_writable($this->__path); }
    public function getExtension() {
        $f = basename($this->__path);
        $p = strrpos($f, '.');
        return $p === false || $p === 0 ? '' : substr($f, $p + 1);
    }
    public function __toString() { return $this->__path; }
}
class FilesystemIterator extends SplFileInfo {
    const CURRENT_AS_FILEINFO = 0; const CURRENT_AS_PATHNAME = 32; const CURRENT_AS_SELF = 16;
    const KEY_AS_PATHNAME = 0; const KEY_AS_FILENAME = 256;
    const FOLLOW_SYMLINKS = 512; const NEW_CURRENT_AND_KEY = 256;
    const SKIP_DOTS = 4096; const UNIX_PATHS = 8192;
}
// DirectoryIterator (flat, dots included): the native iterator is the current
// entry, like RecursiveDirectoryIterator below (Symfony Console's completion
// command scans /etc/bash_completion.d with it).
class DirectoryIterator extends SplFileInfo implements SeekableIterator {
    private $__dir;
    private $__names = [];
    private $__pos = 0;
    public function __construct($directory) {
        parent::__construct($directory);
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("DirectoryIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        $this->__dir = rtrim($directory, '/');
        $this->__names = scandir($directory);
        $this->__sync();
    }
    private function __cur() { return $this->__dir . '/' . $this->__names[$this->__pos]; }
    private function __sync() {
        if ($this->__pos < count($this->__names)) { $this->__path = $this->__cur(); }
    }
    public function rewind(): void { $this->__pos = 0; $this->__sync(); }
    public function valid(): bool { return $this->__pos < count($this->__names); }
    public function next(): void { $this->__pos++; $this->__sync(); }
    public function seek($offset): void { $this->__pos = $offset; $this->__sync(); }
    public function key(): mixed { return $this->__pos; }
    public function current(): mixed { return $this; }
    public function isDot(): bool {
        $n = $this->__names[$this->__pos] ?? '';
        return $n === '.' || $n === '..';
    }
}
class RecursiveDirectoryIterator extends FilesystemIterator implements RecursiveIterator {
    private $__dir;
    private $__flags;
    private $__names = [];
    private $__pos = 0;
    private $__sub = ''; // path of this level relative to the traversal root
    public function __construct($directory, $flags = 0) {
        parent::__construct($directory);
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("RecursiveDirectoryIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        $this->__dir = rtrim($directory, '/');
        $this->__flags = $flags;
        $names = scandir($directory);
        if (($flags & self::SKIP_DOTS) === self::SKIP_DOTS) {
            $keep = [];
            foreach ($names as $n) { if ($n !== '.' && $n !== '..') { $keep[] = $n; } }
            $names = $keep;
        }
        $this->__names = $names;
        $this->__sync();
    }
    private function __cur() { return $this->__dir . '/' . $this->__names[$this->__pos]; }
    // The native iterator *is* the current entry: every inherited SplFileInfo
    // accessor (getFilename/getPathname/isDir/...) reflects the position; Symfony
    // Finder's RecursiveDirectoryIterator::current() is built on exactly that.
    private function __sync() {
        if ($this->__pos < count($this->__names)) { $this->__path = $this->__cur(); }
    }
    public function rewind(): void { $this->__pos = 0; $this->__sync(); }
    public function valid(): bool { return $this->__pos < count($this->__names); }
    public function next(): void { $this->__pos++; $this->__sync(); }
    public function key(): mixed { return $this->__cur(); }
    public function current(): mixed {
        if (($this->__flags & self::CURRENT_AS_PATHNAME) === self::CURRENT_AS_PATHNAME) { return $this->__cur(); }
        return new SplFileInfo($this->__cur());
    }
    public function hasChildren($allowLinks = false) {
        // The native RDI never descends into the dot entries -- without this
        // guard a traversal WITHOUT SKIP_DOTS recurses into `.` forever
        // (doctrine/persistence's SymfonyFileLocator iterates dots included).
        $n = $this->__names[$this->__pos] ?? '';
        if ($n === '.' || $n === '..') { return false; }
        $p = $this->__cur();
        return is_dir($p) && ($allowLinks || ($this->__flags & self::FOLLOW_SYMLINKS) === self::FOLLOW_SYMLINKS || !is_link($p));
    }
    public function getChildren() {
        // `new static`: a subclass (Symfony Finder's iterator) gets subclass
        // children, exactly like the native late-static getChildren.
        $child = new static($this->__cur(), $this->__flags);
        $name = $this->__names[$this->__pos];
        $child->__sub = $this->__sub === '' ? $name : $this->__sub . '/' . $name;
        return $child;
    }
    public function getSubPath() { return $this->__sub; }
    public function getSubPathname() {
        $name = $this->__names[$this->__pos];
        return $this->__sub === '' ? $name : $this->__sub . '/' . $name;
    }
}
class RecursiveIteratorIterator implements OuterIterator {
    const LEAVES_ONLY = 0; const SELF_FIRST = 1; const CHILD_FIRST = 2; const CATCH_GET_CHILD = 16;
    private $__it;
    private $__mode;
    private $__list = [];  // [[key, subpathname, current, depth], ...] in emit order
    private $__pos = 0;
    private $__maxDepth = -1; // -1 = unlimited (setMaxDepth(-1), the default)
    public function __construct($iterator, $mode = 0, $flags = 0) {
        $this->__it = $iterator;
        $this->__mode = $mode;
    }
    public function setMaxDepth($maxDepth = -1) {
        if ($maxDepth < -1) { throw new InvalidArgumentException('Parameter maxDepth must be >= -1'); }
        $this->__maxDepth = $maxDepth;
    }
    public function getMaxDepth() { return $this->__maxDepth === -1 ? false : $this->__maxDepth; }
    private function __collect($it, $depth) {
        for ($it->rewind(); $it->valid(); $it->next()) {
            $descend = method_exists($it, 'hasChildren') && $it->hasChildren()
                && ($this->__maxDepth === -1 || $depth < $this->__maxDepth);
            $entry = [$it->key(), method_exists($it, 'getSubPathname') ? $it->getSubPathname() : $it->key(), $it->current(), $depth];
            if ($descend) {
                if ($this->__mode === self::SELF_FIRST) { $this->__list[] = $entry; }
                $this->__collect($it->getChildren(), $depth + 1);
                if ($this->__mode === self::CHILD_FIRST) { $this->__list[] = $entry; }
            } else {
                $this->__list[] = $entry;
            }
        }
    }
    public function rewind(): void {
        $this->__list = [];
        $this->__pos = 0;
        $this->__collect($this->__it, 0);
    }
    public function valid(): bool { return $this->__pos < count($this->__list); }
    public function next(): void { $this->__pos++; }
    public function key(): mixed { return $this->__list[$this->__pos][0]; }
    public function current(): mixed { return $this->__list[$this->__pos][2]; }
    public function getSubPathname() { return $this->__list[$this->__pos][1]; }
    public function getDepth() { return $this->__list[$this->__pos][3]; }
    public function getInnerIterator() { return $this->__it; }
}
// SplDoublyLinkedList family (Composer's dependency solver: SplQueue work
// queues, RuleWatchChain extends the list and removes mid-iteration). Backed by
// a plain array kept dense via array_splice; the iteration cursor is a plain
// index that runs bottom-up (FIFO) or top-down (LIFO). Offsets index from the
// iteration head: $stack[0] is the most recently pushed element (oracle).
class SplDoublyLinkedList implements Iterator, Countable, ArrayAccess {
    const IT_MODE_LIFO = 2;
    const IT_MODE_FIFO = 0;
    const IT_MODE_DELETE = 1;
    const IT_MODE_KEEP = 0;
    protected $__items = [];
    protected $__pos = 0;
    protected $__mode = 0; // FIFO|KEEP; SplStack flips to LIFO
    private function __lifo() { return ($this->__mode & self::IT_MODE_LIFO) === self::IT_MODE_LIFO; }
    private function __real($index, $method) {
        $n = count($this->__items);
        $i = (int)$index;
        $r = $this->__lifo() ? $n - 1 - $i : $i;
        if ($i < 0 || $r < 0 || $r >= $n) {
            throw new OutOfRangeException("SplDoublyLinkedList::$method(): Argument #1 (\$index) is out of range");
        }
        return $r;
    }
    public function push($value) { $this->__items[] = $value; }
    public function pop() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't pop from an empty datastructure"); }
        return array_pop($this->__items);
    }
    public function shift() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't shift from an empty datastructure"); }
        return array_shift($this->__items);
    }
    public function unshift($value) { array_unshift($this->__items, $value); }
    public function top() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't peek at an empty datastructure"); }
        return $this->__items[count($this->__items) - 1];
    }
    public function bottom() {
        if (count($this->__items) === 0) { throw new RuntimeException("Can't peek at an empty datastructure"); }
        return $this->__items[0];
    }
    public function isEmpty() { return count($this->__items) === 0; }
    public function count(): int { return count($this->__items); }
    public function setIteratorMode($mode) { $this->__mode = (int)$mode; }
    public function getIteratorMode() { return $this->__mode; }
    public function toArray() { return $this->__items; }
    public function rewind(): void { $this->__pos = $this->__lifo() ? count($this->__items) - 1 : 0; }
    public function valid(): bool { return $this->__pos >= 0 && $this->__pos < count($this->__items); }
    public function current(): mixed { return $this->valid() ? $this->__items[$this->__pos] : null; }
    public function key(): mixed { return $this->__pos; }
    public function next(): void {
        if ($this->__lifo()) {
            if (($this->__mode & self::IT_MODE_DELETE) === self::IT_MODE_DELETE && $this->valid()) { $this->pop(); }
            $this->__pos--;
        } else {
            if (($this->__mode & self::IT_MODE_DELETE) === self::IT_MODE_DELETE && $this->valid()) {
                $this->shift();
            } else {
                $this->__pos++;
            }
        }
    }
    public function prev(): void { if ($this->__lifo()) { $this->__pos++; } else { $this->__pos--; } }
    public function offsetExists($index): bool {
        $n = count($this->__items);
        $i = (int)$index;
        return $i >= 0 && $i < $n;
    }
    public function offsetGet($index): mixed { return $this->__items[$this->__real($index, "offsetGet")]; }
    public function offsetSet($index, $value): void {
        if ($index === null) { $this->__items[] = $value; return; }
        $this->__items[$this->__real($index, "offsetSet")] = $value;
    }
    public function offsetUnset($index): void {
        array_splice($this->__items, $this->__real($index, "offsetUnset"), 1);
    }
    public function add($index, $value) {
        $n = count($this->__items);
        $i = (int)$index;
        if ($i < 0 || $i > $n) {
            throw new OutOfRangeException("SplDoublyLinkedList::add(): Argument #1 (\$index) is out of range");
        }
        array_splice($this->__items, $i, 0, [$value]);
    }
}
class SplQueue extends SplDoublyLinkedList {
    public function enqueue($value) { $this->push($value); }
    public function dequeue() { return $this->shift(); }
}
class SplStack extends SplDoublyLinkedList {
    protected $__mode = 2; // IT_MODE_LIFO | IT_MODE_KEEP
}
class SplObjectStorage implements Countable, Iterator, ArrayAccess {
    private $__objs = [];  // spl_object_id => object (strong ref, as ext/spl)
    private $__data = [];  // spl_object_id => attached info
    private $__pos = 0;
    private $__ids = [];   // iteration snapshot (rewind)
    private function __attach($object, $info) {
        $id = spl_object_id($object);
        $this->__objs[$id] = $object;
        $this->__data[$id] = $info;
    }
    public function attach($object, $info = null) {
        __deprecated_from_caller('Method SplObjectStorage::attach() is deprecated since 8.5, use method SplObjectStorage::offsetSet() instead');
        $this->__attach($object, $info);
    }
    public function detach($object) {
        __deprecated_from_caller('Method SplObjectStorage::detach() is deprecated since 8.5, use method SplObjectStorage::offsetUnset() instead');
        $id = spl_object_id($object);
        unset($this->__objs[$id], $this->__data[$id]);
    }
    public function contains($object) {
        __deprecated_from_caller('Method SplObjectStorage::contains() is deprecated since 8.5, use method SplObjectStorage::offsetExists() instead');
        return isset($this->__objs[spl_object_id($object)]);
    }
    public function addAll($storage) {
        foreach ($storage->__objs as $id => $obj) {
            $this->__objs[$id] = $obj;
            $this->__data[$id] = $storage->__data[$id];
        }
        return $this->count();
    }
    public function removeAll($storage) {
        foreach ($storage->__objs as $id => $obj) {
            unset($this->__objs[$id], $this->__data[$id]);
        }
        return $this->count();
    }
    public function removeAllExcept($storage) {
        foreach ($this->__objs as $id => $obj) {
            if (!isset($storage->__objs[$id])) {
                unset($this->__objs[$id], $this->__data[$id]);
            }
        }
        return $this->count();
    }
    public function getHash($object) { return spl_object_hash($object); }
    public function count($mode = COUNT_NORMAL) { return count($this->__objs); }
    public function rewind() { $this->__ids = array_keys($this->__objs); $this->__pos = 0; }
    public function valid() { return isset($this->__ids[$this->__pos]) && isset($this->__objs[$this->__ids[$this->__pos]]); }
    public function key() { return $this->__pos; }
    public function current() { return $this->__objs[$this->__ids[$this->__pos]]; }
    public function next() { $this->__pos++; }
    public function getInfo() {
        if (!$this->valid()) { return null; }
        return $this->__data[$this->__ids[$this->__pos]];
    }
    public function setInfo($info) {
        if ($this->valid()) { $this->__data[$this->__ids[$this->__pos]] = $info; }
    }
    public function offsetExists($object) { return isset($this->__objs[spl_object_id($object)]); }
    public function offsetSet($object, $info = null) { $this->__attach($object, $info); }
    public function offsetUnset($object) {
        $id = spl_object_id($object);
        unset($this->__objs[$id], $this->__data[$id]);
    }
    public function offsetGet($object) {
        $id = spl_object_id($object);
        if (!isset($this->__objs[$id])) {
            throw new UnexpectedValueException('Object not found');
        }
        return $this->__data[$id];
    }
}
class ArrayIterator implements Iterator, ArrayAccess, Countable {
    private $__storage = [];
    private $__keys = [];
    private $__pos = 0;
    public function __construct($array = []) {
        $this->__storage = (array)$array;
        $this->__keys = array_keys($this->__storage);
    }
    public function rewind() { $this->__keys = array_keys($this->__storage); $this->__pos = 0; }
    public function valid() { return $this->__pos < count($this->__keys); }
    public function current() { return $this->__storage[$this->__keys[$this->__pos]]; }
    public function key() { return $this->__keys[$this->__pos]; }
    public function next() { $this->__pos++; }
    public function offsetExists($key) { return isset($this->__storage[$key]); }
    public function offsetGet($key) { return $this->__storage[$key] ?? null; }
    public function offsetSet($key, $value) {
        if ($key === null) { $this->__storage[] = $value; }
        else { $this->__storage[$key] = $value; }
    }
    public function offsetUnset($key) { unset($this->__storage[$key]); }
    public function count() { return count($this->__storage); }
    public function getArrayCopy() { return $this->__storage; }
    public function append($value) { $this->__storage[] = $value; }
}
class ArrayObject implements IteratorAggregate, ArrayAccess, Countable {
    private $__storage = [];
    public function __construct($array = []) { $this->__storage = (array)$array; }
    public function getIterator() { return new ArrayIterator($this->__storage); }
    public function offsetExists($key) { return isset($this->__storage[$key]); }
    public function offsetGet($key) { return $this->__storage[$key] ?? null; }
    public function offsetSet($key, $value) {
        if ($key === null) { $this->__storage[] = $value; }
        else { $this->__storage[$key] = $value; }
    }
    public function offsetUnset($key) { unset($this->__storage[$key]); }
    public function count() { return count($this->__storage); }
    public function getArrayCopy() { return $this->__storage; }
    public function append($value) { $this->__storage[] = $value; }
}
// `IteratorIterator` wraps any Traversable as a concrete `Iterator`, resolving an
// `IteratorAggregate` to its inner iterator once at construction; protocol calls
// delegate to the inner. `getInnerIterator()` returns the wrapped iterator.
class IteratorIterator implements Iterator {
    private $__it;
    public function __construct($iterator) {
        if ($iterator instanceof IteratorAggregate) { $iterator = $iterator->getIterator(); }
        $this->__it = $iterator;
    }
    public function getInnerIterator() { return $this->__it; }
    public function rewind() { return $this->__it->rewind(); }
    public function valid() { return $this->__it->valid(); }
    public function current() { return $this->__it->current(); }
    public function key() { return $this->__it->key(); }
    public function next() { return $this->__it->next(); }
    // SPL's dual-iterators forward unknown method calls to the inner iterator
    // (spl_dual_it_call_method): `$filterIt->getFilename()` reaches the
    // wrapped (Recursive)DirectoryIterator down the chain. Symfony Finder's
    // whole Iterator/ stack leans on this.
    public function __call($name, $args) { return $this->__it->$name(...$args); }
}
interface OuterIterator extends Iterator {
    public function getInnerIterator();
}
interface RecursiveIterator extends Iterator {
    public function hasChildren();
    public function getChildren();
}
// `FilterIterator`: skip inner entries the subclass's accept() rejects --
// rewind/next fast-forward to the next accepted position (Symfony Finder's
// whole Iterator/ directory is built on it).
abstract class FilterIterator extends IteratorIterator {
    abstract public function accept(): bool;
    public function rewind() {
        parent::rewind();
        while (parent::valid() && !$this->accept()) { parent::next(); }
    }
    public function next() {
        parent::next();
        while (parent::valid() && !$this->accept()) { parent::next(); }
    }
}
class CallbackFilterIterator extends FilterIterator {
    private $__cb;
    public function __construct($iterator, $callback) {
        parent::__construct($iterator);
        $this->__cb = $callback;
    }
    public function accept(): bool {
        $cb = $this->__cb;
        return (bool) $cb($this->current(), $this->key(), $this->getInnerIterator());
    }
}
// `RecursiveFilterIterator`: FilterIterator over a RecursiveIterator; children
// are wrapped in the SUBCLASS (new static) so the filter applies at every
// depth (PHPUnit's Runner/Filter iterators are built on it).
abstract class RecursiveFilterIterator extends FilterIterator implements RecursiveIterator {
    public function hasChildren(): bool {
        return $this->getInnerIterator()->hasChildren();
    }
    public function getChildren() {
        return new static($this->getInnerIterator()->getChildren());
    }
}
// `AppendIterator`: iterate several iterators in sequence. Each appended
// iterator is rewound when it becomes current (append-after-start supported).
class AppendIterator implements OuterIterator {
    private $__its = [];
    private $__idx = 0;
    public function __construct() {}
    public function append($iterator) { $this->__its[] = $iterator; }
    public function getInnerIterator() { return $this->__its[$this->__idx] ?? null; }
    private function __settle() {
        while ($this->__idx < count($this->__its) && !$this->__its[$this->__idx]->valid()) {
            $this->__idx++;
            if ($this->__idx < count($this->__its)) { $this->__its[$this->__idx]->rewind(); }
        }
    }
    public function rewind() {
        $this->__idx = 0;
        if (count($this->__its) > 0) { $this->__its[0]->rewind(); }
        $this->__settle();
    }
    public function valid() { return $this->__idx < count($this->__its) && $this->__its[$this->__idx]->valid(); }
    public function current() { return $this->valid() ? $this->__its[$this->__idx]->current() : null; }
    public function key() { return $this->valid() ? $this->__its[$this->__idx]->key() : null; }
    public function next() {
        if ($this->valid()) { $this->__its[$this->__idx]->next(); }
        $this->__settle();
    }
}
// `SplFixedArray`: a fixed-size, integer-indexed array. Backed by `$__storage`
// filled with nulls to `$__size`; out-of-range offsets throw RuntimeException.
class SplFixedArray implements ArrayAccess, Countable, Iterator {
    private $__storage = [];
    private $__size = 0;
    private $__pos = 0;
    public function __construct($size = 0) {
        $this->__size = $size;
        for ($i = 0; $i < $size; $i++) { $this->__storage[$i] = null; }
    }
    public function getSize() { return $this->__size; }
    public function setSize($size) {
        if ($size < $this->__size) {
            for ($i = $size; $i < $this->__size; $i++) { unset($this->__storage[$i]); }
        } else {
            for ($i = $this->__size; $i < $size; $i++) { $this->__storage[$i] = null; }
        }
        $this->__size = $size;
        return true;
    }
    public function count() { return $this->__size; }
    public function toArray() { return $this->__storage; }
    public function offsetExists($i) { return $i >= 0 && $i < $this->__size; }
    public function offsetGet($i) {
        if ($i < 0 || $i >= $this->__size) { throw new RuntimeException("Index invalid or out of range"); }
        return $this->__storage[$i];
    }
    public function offsetSet($i, $v) {
        if ($i < 0 || $i >= $this->__size) { throw new RuntimeException("Index invalid or out of range"); }
        $this->__storage[$i] = $v;
    }
    public function offsetUnset($i) {
        if ($i >= 0 && $i < $this->__size) { $this->__storage[$i] = null; }
    }
    public function rewind() { $this->__pos = 0; }
    public function valid() { return $this->__pos < $this->__size; }
    public function current() { return $this->__storage[$this->__pos]; }
    public function key() { return $this->__pos; }
    public function next() { $this->__pos++; }
    public static function fromArray($array) {
        $a = new SplFixedArray(count($array));
        $i = 0;
        foreach ($array as $v) { $a[$i] = $v; $i++; }
        return $a;
    }
}
// The Reflector interface every reflection object satisfies (it extends
// Stringable). phpr does not yet render the full PHP export format in the
// classes' __toString, but the interface membership itself is what code
// type-hints and tests via `instanceof Reflector`.
interface Reflector extends Stringable {}
// The Reflection base: a handful of static helpers, chiefly getModifierNames().
abstract class Reflection {
    public static function getModifierNames($modifiers) {
        $names = [];
        // Order fixed by ext/reflection: abstract, final, visibility, static,
        // readonly (NOT by bit value).
        if ($modifiers & 64)  { $names[] = 'abstract'; }   // IS_ABSTRACT
        if ($modifiers & 32)  { $names[] = 'final'; }      // IS_FINAL
        if ($modifiers & 1)   { $names[] = 'public'; }     // IS_PUBLIC
        if ($modifiers & 2)   { $names[] = 'protected'; }  // IS_PROTECTED
        if ($modifiers & 4)   { $names[] = 'private'; }    // IS_PRIVATE
        if ($modifiers & 16)  { $names[] = 'static'; }     // IS_STATIC
        if ($modifiers & 128) { $names[] = 'readonly'; }   // IS_READONLY
        return $names;
    }
}
class ReflectionException extends Exception {}
// A loaded Zend (C) extension. phpr has no C extensions, so this reflects the
// bare name it is constructed with: enough for `new ReflectionZendExtension($n)`
// and its accessors to exist.
class ReflectionZendExtension implements Reflector {
    public $name;
    public function __construct($name) { $this->name = $name; }
    public function __toString() { return ''; }
    public function getName() { return $this->name; }
    public function getVersion() { return ''; }
    public function getAuthor() { return ''; }
    public function getURL() { return ''; }
    public function getCopyright() { return ''; }
}
class ReflectionAttribute {
    const IS_INSTANCEOF = 2;
    // Validate the $flags argument and apply the IS_INSTANCEOF filter. `$label` is
    // the Reflection class reported in the invalid-flag error (PHP reports the
    // declaring scope, e.g. ReflectionFunctionAbstract for func/method). When the
    // flag is set, `$all` was fetched unfiltered (host name = null) and we keep
    // only attributes whose class is `$name` or a subclass/implementor of it; the
    // filter class must exist (else PHP throws "Class X not found"). Without the
    // flag the host already applied the exact-name filter, so `$all` is as-is.
    public static function __filter($all, $name, $flags, $label) {
        if ($flags !== 0 && $flags !== self::IS_INSTANCEOF) {
            throw new Error($label . '::getAttributes(): Argument #2 ($flags) must be a valid attribute filter flag');
        }
        if (($flags & self::IS_INSTANCEOF) === 0 || $name === null) { return $all; }
        if (!class_exists($name) && !interface_exists($name)) {
            throw new Error('Class "' . $name . '" not found');
        }
        $want = strtolower($name);
        $out = [];
        foreach ($all as $a) {
            $x = $a->getName();
            $isa = strcasecmp($x, $name) === 0;
            // Only walk the hierarchy for a class that actually exists; an
            // unresolved attribute class is simply not an instance of anything but
            // its own name (mirrors PHP, which emits no warning here).
            if (!$isa && (class_exists($x) || interface_exists($x))) {
                foreach (class_parents($x) as $p) { if (strtolower($p) === $want) { $isa = true; break; } }
                if (!$isa) {
                    foreach (class_implements($x) as $i) { if (strtolower($i) === $want) { $isa = true; break; } }
                }
            }
            if ($isa) { $out[] = $a; }
        }
        return $out;
    }
    public $name;
    // Private handle to the owning class + the attribute's position in it, used by
    // the host builtins to materialise the attribute lazily. `__prop` is set for an
    // attribute that decorates a property (vs the class itself).
    public $__class;
    public $__index;
    public $__prop;
    public $__func;
    public $__method;
    public $__const;
    public $__classconst;
    public $__paramfunc;
    public $__paramclass;
    public $__parampos;
    public $__closure_val;
    public function getName() { return $this->name; }
    public function getArguments() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_args($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_args($this->__class, $this->__classconst, $this->__index);
        }
        if (isset($this->__prop)) {
            return __reflect_prop_attr_args($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_args($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_args($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_args($this->__const, $this->__index);
        }
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_args($this->__closure_val, $this->__index);
        }
        return __reflect_attr_arguments($this->__class, $this->__index);
    }
    public function newInstance() {
        if (isset($this->__paramfunc)) {
            return __reflect_param_attr_new($this->__paramclass, $this->__paramfunc, $this->__parampos, $this->__index);
        }
        if (isset($this->__classconst)) {
            return __reflect_classconst_attr_new($this->__class, $this->__classconst, $this->__index);
        }
        if (isset($this->__prop)) {
            return __reflect_prop_attr_new($this->__class, $this->__prop, $this->__index);
        }
        if (isset($this->__func)) {
            return __reflect_func_attr_new($this->__func, $this->__index);
        }
        if (isset($this->__method)) {
            return __reflect_method_attr_new($this->__class, $this->__method, $this->__index);
        }
        if (isset($this->__const)) {
            return __reflect_const_attr_new($this->__const, $this->__index);
        }
        if ($this->__closure_val !== null) {
            return __reflect_closure_attr_new($this->__closure_val, $this->__index);
        }
        return __reflect_attr_newinstance($this->__class, $this->__index);
    }
}
class ReflectionClass implements Reflector {
    const SKIP_INITIALIZATION_ON_SERIALIZE = 8;
    const SKIP_DESTRUCTOR = 16;
    public $name;
    public function __construct($objectOrClass) {
        $this->name = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        // An *object* argument is always reflectable (engine values like a
        // Closure included); only a class-name string is checked for existence.
        if (!is_object($objectOrClass) && !class_exists($this->name) && !interface_exists($this->name) && !trait_exists($this->name)) {
            throw new ReflectionException(sprintf('Class "%s" does not exist', $this->name));
        }
        // Class names resolve case-insensitively but ReflectionClass::$name carries
        // the CANONICAL declared casing (a `new ReflectionClass('MY\CLASS')` reports
        // `My\Class`): normalize a string argument to it.
        if (!is_object($objectOrClass)) {
            $real = __reflect_class_real_name($this->name);
            if ($real !== false) { $this->name = $real; }
        }
    }
    public function getFileName() { $l = __reflect_class_loc($this->name); return $l[0]; }
    public function isInternal() { return $this->getFileName() === false; }
    public function isUserDefined() { return $this->getFileName() !== false; }
    public function getDocComment() { return __reflect_class_doc($this->name); }
    public function isReadOnly() { return __reflect_class_modifiers($this->name)['readonly'] ?? false; }
    public function getStartLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[1]; }
    public function getEndLine() { $l = __reflect_class_loc($this->name); return $l[0] === false ? false : $l[2]; }
    // phpr mangles anonymous classes exactly like PHP: `class@anonymous\0N`.
    public function isAnonymous() { return strpos($this->name, 'class@anonymous') === 0; }
    public function getName() { return $this->name; }
    // Indent every non-empty line of a stringable child (a nested Method block) by
    // $n spaces for the export format.
    private function __indent($obj, $n) {
        $pad = str_repeat(' ', $n);
        $out = '';
        foreach (explode("\n", rtrim((string) $obj, "\n")) as $l) {
            $out .= ($l === '' ? '' : $pad . $l) . "\n";
        }
        return $out;
    }
    // The PHP `Reflection::export` string for a class (ext/reflection golden output).
    public function __toString() {
        return $this->__exportString(false, []);
    }
    // Shared class/object export body. In object mode the header word is
    // "Object of class" and a "- Dynamic properties [N]" section (from $dynNames)
    // is inserted between the declared Properties and the Methods.
    protected function __exportString($objectMode, $dynNames) {
        $src = $this->isUserDefined() ? '<user>' : '<internal>';
        if ($objectMode) {
            $head = 'Object of class';
            $kind = ($this->isAbstract() ? 'abstract ' : '') . ($this->isFinal() ? 'final ' : '') . 'class';
        }
        elseif ($this->isInterface()) { $head = 'Interface'; $kind = 'interface'; }
        elseif ($this->isTrait()) { $head = 'Class'; $kind = 'trait'; }
        else {
            $head = 'Class';
            $kind = ($this->isAbstract() ? 'abstract ' : '') . ($this->isFinal() ? 'final ' : '') . 'class';
        }
        $hdr = "$head [ $src $kind {$this->name}";
        $parent = $this->getParentClass();
        if ($parent !== false) { $hdr .= " extends " . $parent->getName(); }
        $ifaces = $this->getInterfaceNames();
        if (!empty($ifaces)) { $hdr .= " implements " . implode(', ', $ifaces); }
        $s = $hdr . " ] {\n";
        $file = $this->getFileName();
        if ($file !== false) {
            $s .= "  @@ $file " . $this->getStartLine() . "-" . $this->getEndLine() . "\n";
        }
        $consts = $this->getReflectionConstants();
        $props = $this->getProperties();
        $sprops = []; $iprops = [];
        foreach ($props as $p) { if ($p->isStatic()) { $sprops[] = $p; } else { $iprops[] = $p; } }
        $methods = $this->getMethods();
        $smeth = []; $imeth = [];
        foreach ($methods as $m) { if ($m->isStatic()) { $smeth[] = $m; } else { $imeth[] = $m; } }
        $s .= "\n  - Constants [" . count($consts) . "] {\n";
        foreach ($consts as $c) { $s .= "    " . rtrim((string) $c, "\n") . "\n"; }
        $s .= "  }\n";
        $s .= "\n  - Static properties [" . count($sprops) . "] {\n";
        foreach ($sprops as $p) { $s .= "    " . rtrim((string) $p, "\n") . "\n"; }
        $s .= "  }\n";
        $s .= "\n  - Static methods [" . count($smeth) . "] {\n";
        foreach ($smeth as $m) { $s .= $this->__indent($m, 4); }
        $s .= "  }\n";
        $s .= "\n  - Properties [" . count($iprops) . "] {\n";
        foreach ($iprops as $p) {
            $doc = $p->getDocComment();
            if ($doc !== false) { $s .= "    " . $doc . "\n"; }
            $s .= "    " . rtrim((string) $p, "\n") . "\n";
        }
        $s .= "  }\n";
        if ($objectMode) {
            $s .= "\n  - Dynamic properties [" . count($dynNames) . "] {\n";
            foreach ($dynNames as $dn) { $s .= "    Property [ <dynamic> public \$$dn ]\n"; }
            $s .= "  }\n";
        }
        $s .= "\n  - Methods [" . count($imeth) . "] {\n";
        foreach ($imeth as $m) { $s .= $this->__indent($m, 4); }
        $s .= "  }\n";
        return $s . "}\n";
    }
    public function getShortName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? $this->name : substr($this->name, $p + 1);
    }
    // Attributes are retained at lowering; the host builds one ReflectionAttribute
    // per attribute declared on the class (optionally filtered by name).
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_class_attributes($this->name, $hostName), $name, $flags, 'ReflectionClass');
    }
    public function newInstance(...$args) { return new $this->name(...$args); }
    public function newInstanceArgs($args = []) { return new $this->name(...$args); }
    public function newInstanceWithoutConstructor() {
        // Internal final classes cannot skip their constructor (Zend rejects
        // it; doctrine/instantiator's PDORow probe relies on the refusal).
        if ($this->isInternal() && $this->isFinal()) {
            throw new ReflectionException('Class ' . $this->name . ' is an internal class marked as final that cannot be instantiated without invoking its constructor');
        }
        return __reflect_new_no_ctor($this->name);
    }
    const SKIP_INITIALIZATION_ON_SERIALIZE = 8;
    const SKIP_DESTRUCTOR = 16;
    private function __lazyOptionsCheck($method, $argno, $options, $valid) {
        if (($options & ~$valid) !== 0) {
            // SKIP_DESTRUCTOR alone gets the dedicated wording; any other
            // stray bit is "invalid flags" (Zend's order).
            if (($options & ~($valid | self::SKIP_DESTRUCTOR)) === 0 && ($options & self::SKIP_DESTRUCTOR)) {
                throw new ReflectionException(sprintf('ReflectionClass::%s(): Argument #%d ($options) does not accept ReflectionClass::SKIP_DESTRUCTOR', $method, $argno));
            }
            throw new ReflectionException(sprintf('ReflectionClass::%s(): Argument #%d ($options) contains invalid flags', $method, $argno));
        }
    }
    public function newLazyGhost(callable $initializer, int $options = 0) {
        $this->__lazyOptionsCheck('newLazyGhost', 2, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE);
        return __reflect_new_lazy_ghost($this->name, $initializer, $options);
    }
    public function newLazyProxy(callable $factory, int $options = 0) {
        $this->__lazyOptionsCheck('newLazyProxy', 2, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE);
        return __reflect_new_lazy_proxy($this->name, $factory, $options);
    }
    public function resetAsLazyGhost($object, callable $initializer, int $options = 0) {
        $this->__lazyOptionsCheck('resetAsLazyGhost', 3, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE | self::SKIP_DESTRUCTOR);
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, false, $initializer, $options);
    }
    public function resetAsLazyProxy($object, callable $factory, int $options = 0) {
        $this->__lazyOptionsCheck('resetAsLazyProxy', 3, $options, self::SKIP_INITIALIZATION_ON_SERIALIZE | self::SKIP_DESTRUCTOR);
        if (__lazy_is_initializing($object)) { throw new Error('Can not reset an object while it is being initialized'); }
        if (__lazy_is_uninitialized($object)) { throw new ReflectionException('Object is already lazy'); }
        return __reflect_reset_lazy($this->name, $object, true, $factory, $options);
    }
    public function isUninitializedLazyObject($object) { return __lazy_is_uninitialized($object); }
    public function getLazyInitializer($object) { return __lazy_get_initializer($object); }
    public function initializeLazyObject($object) { return __lazy_initialize($object); }
    public function markLazyObjectAsInitialized($object) { return __lazy_mark_initialized($object); }
    public function isInstantiable() { return class_exists($this->name); }
    public function isCloneable() {
        if ($this->isInterface() || $this->isAbstract() || $this->isEnum()) { return false; }
        if ($this->hasMethod('__clone')) {
            return $this->getMethod('__clone')->isPublic();
        }
        return true;
    }
    public function isInterface() { return interface_exists($this->name); }
    public function isTrait() { return trait_exists($this->name); }
    public function isEnum() { return in_array('UnitEnum', class_implements($this->name)); }
    public function isFinal() { return __reflect_class_modifiers($this->name)['final']; }
    public function isAbstract() { return __reflect_class_modifiers($this->name)['abstract']; }
    // Bitmask of class modifiers. Only an *explicit* `abstract class` sets 64;
    // interfaces/traits (whose methods are implicitly abstract) return 0, as does
    // a plain class. `final` is 32.
    public function getModifiers() {
        $m = 0;
        if (!$this->isInterface() && !$this->isTrait() && $this->isAbstract()) { $m |= 64; }
        if ($this->isFinal()) { $m |= 32; }
        return $m;
    }
    public function isInstance($object) { return $object instanceof $this->name; }
    // A class is iterable when it implements Traversable (via Iterator or
    // IteratorAggregate); class_implements includes inherited interfaces.
    public function isIterateable() {
        return in_array('Traversable', class_implements($this->name), true)
            || strcasecmp($this->name, 'Traversable') === 0;
    }
    public function isIterable() { return $this->isIterateable(); }
    // Static properties as a `name => value` map (own + inherited), values current.
    public function getStaticProperties() { return __reflect_static_props($this->name); }
    public function getStaticPropertyValue($name, $default = null) {
        $props = $this->getStaticProperties();
        if (array_key_exists($name, $props)) { return $props[$name]; }
        if (func_num_args() >= 2) { return $default; }
        throw new ReflectionException(sprintf('Property %s::$%s does not exist', $this->name, $name));
    }
    public function setStaticPropertyValue($name, $value) {
        if (!__reflect_static_prop_set($this->name, $name, $value)) {
            throw new ReflectionException(sprintf('Class %s does not have a property named %s', $this->name, $name));
        }
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
    public function hasProperty($name) {
        if (!property_exists($this->name, $name)) { return false; }
        // An ancestor's PRIVATE property is invisible to a subclass' reflection
        // surface: hasProperty() must agree with getProperty()/new
        // ReflectionProperty() (which throw for it), else a caller that guards a
        // getProperty() with hasProperty() still hits the exception
        // (Doctrine ORM ClassMetadata::isTypedProperty, GH11199).
        $decl = __reflect_prop_declaring_class($this->name, $name);
        if ($decl !== false && strcasecmp($decl, $this->name) !== 0) {
            $info = __reflect_prop_details($decl, $name);
            if (is_array($info) && ($info['visibility'] ?? null) === 'private') {
                return false;
            }
        }
        return true;
    }
    public function getProperty($name) { return new ReflectionProperty($this->name, $name); }
    public function getProperties($filter = null) {
        $out = [];
        foreach (__reflect_prop_names($this->name) as $n) {
            // Construct each with the DECLARING class as scope so the ancestor-
            // private guard in the ctor does not fire (it would otherwise abort
            // enumeration). Zend then OMITS an ancestor's private property from a
            // subclass' getProperties() (_addproperty: PRIVATE && prop_info->ce !=
            // ce), matching hasProperty()/getProperty() which reject it.
            $decl = __reflect_prop_declaring_class($this->name, $n);
            $rp = new ReflectionProperty($decl === false ? $this->name : $decl, $n);
            if ($decl !== false && strcasecmp($decl, $this->name) !== 0 && $rp->isPrivate()) {
                continue;
            }
            $out[] = $rp;
        }
        return $out;
    }
    public function hasConstant($name) { return defined($this->name . '::' . $name); }
    public function getConstant($name) { return constant($this->name . '::' . $name); }
    public function getConstants($filter = null) {
        return $filter === null
            ? __reflect_class_constants($this->name)
            : __reflect_class_constants($this->name, $filter);
    }
    public function getReflectionConstant($name) {
        try { return new ReflectionClassConstant($this->name, $name); }
        catch (ReflectionException $e) { return false; }
    }
    public function getReflectionConstants($filter = null) {
        $out = [];
        foreach (__reflect_class_const_names($this->name) as $n) {
            $rc = new ReflectionClassConstant($this->name, $n);
            if ($filter === null || ($rc->getModifiers() & $filter)) { $out[] = $rc; }
        }
        return $out;
    }
    public function implementsInterface($interface) {
        return in_array($interface, class_implements($this->name), true);
    }
    public function isSubclassOf($class) {
        return in_array($class, class_parents($this->name), true)
            || in_array($class, class_implements($this->name), true);
    }
    public function getParentClass() {
        $p = get_parent_class($this->name);
        return $p === false ? false : new ReflectionClass($p);
    }
    public function getInterfaceNames() { return array_values(class_implements($this->name)); }
    public function getInterfaces() {
        $out = [];
        foreach (class_implements($this->name) as $i) { $out[$i] = new ReflectionClass($i); }
        return $out;
    }
    public function getTraitNames() { return array_values(class_uses($this->name)); }
    public function getTraits() {
        $out = [];
        foreach (class_uses($this->name) as $t) { $out[$t] = new ReflectionClass($t); }
        return $out;
    }
    public function getTraitAliases() { return []; }
    public function getDefaultProperties() { return __reflect_prop_defaults($this->name); }
    public function getNamespaceName() {
        $p = strrpos($this->name, '\\');
        return $p === false ? '' : substr($this->name, 0, $p);
    }
    public function inNamespace() { return strpos($this->name, '\\') !== false; }
    public function getMethod($name) { return new ReflectionMethod($this->name, $name); }
    public function getConstructor() {
        return method_exists($this->name, '__construct')
            ? new ReflectionMethod($this->name, '__construct') : null;
    }
    public function getMethods($filter = null) {
        $out = [];
        // All visibilities, parent chain included (get_class_methods filters
        // to public outside the class -- PHPUnit's #[Before] hooks are protected).
        foreach (__reflect_method_names($this->name) as $m) {
            $rm = new ReflectionMethod($this->name, $m);
            if ($filter !== null && ($rm->getModifiers() & $filter) === 0) { continue; }
            $out[] = $rm;
        }
        return $out;
    }
    public function hasMethod($name) { return method_exists($this->name, $name); }
}
abstract class ReflectionType {
    abstract public function allowsNull(): bool;
    abstract public function __toString(): string;
}
class ReflectionUnionType extends ReflectionType {
    public $__types; public $__nullable;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('|', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionUnionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionIntersectionType extends ReflectionType {
    public $__types;
    public function getTypes() { return $this->__types; }
    public function allowsNull(): bool { return false; }
    public function __toString(): string {
        $parts = [];
        foreach ($this->__types as $t) { $parts[] = $t->getName(); }
        return implode('&', $parts);
    }
    public static function __fromInfo($t) {
        $r = new ReflectionIntersectionType();
        $types = [];
        foreach ($t['types'] as $m) { $types[] = ReflectionNamedType::__fromInfo($m); }
        $r->__types = $types;
        return $r;
    }
}
class ReflectionNamedType extends ReflectionType {
    public $__name; public $__builtin; public $__nullable;
    public function getName() { return $this->__name; }
    public function allowsNull(): bool { return $this->__nullable; }
    public function isBuiltin() { return $this->__builtin; }
    public function __toString(): string {
        $q = ($this->__nullable && $this->__name !== 'mixed' && $this->__name !== 'null') ? '?' : '';
        return $q . $this->__name;
    }
    public static function __fromInfo($t) {
        if ($t === false) { return null; }
        if (isset($t['kind'])) {
            return $t['kind'] === 'intersection'
                ? ReflectionIntersectionType::__fromInfo($t)
                : ReflectionUnionType::__fromInfo($t);
        }
        $r = new ReflectionNamedType();
        $r->__name = $t['name']; $r->__builtin = $t['builtin']; $r->__nullable = $t['nullable'];
        return $r;
    }
}
class ReflectionParameter implements Reflector {
    public $name;
    public $__pos; public $__optional; public $__variadic; public $__byref;
    public $__type; public $__hasDefault; public $__default;
    public $__declClass; public $__declFunc; public $__defaultConst; public $__defaultError; public $__promoted;
    public function __construct($function = null, $param = null) {
        if ($function === null) { return; } // internal factory path (__fromInfo)
        $info = is_array($function)
            ? __reflect_method_info($function[0], $function[1])
            : __reflect_func_info($function);
        if ($info === false) { throw new ReflectionException('The function does not exist'); }
        foreach ($info['params'] as $p) {
            if ((is_int($param) && $p['position'] === $param) || $p['name'] === $param) {
                $this->__init($p); return;
            }
        }
        throw new ReflectionException('The parameter specified does not exist');
    }
    public function __init($p) {
        $this->name = $p['name']; $this->__pos = $p['position'];
        $this->__optional = $p['optional']; $this->__variadic = $p['variadic'];
        $this->__byref = $p['byref']; $this->__type = $p['type'];
        $this->__hasDefault = $p['hasDefault']; $this->__default = $p['default'];
        $this->__declClass = $p['declClass'] ?? ''; $this->__declFunc = $p['declFunc'] ?? '';
        $this->__defaultConst = $p['defaultConstant'] ?? false;
        $this->__defaultError = $p['defaultError'] ?? false;
        $this->__promoted = $p['promoted'] ?? false;
    }
    public function isPromoted() { return $this->__promoted; }
    public function isDefaultValueConstant() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        return $this->__defaultConst !== false;
    }
    public function getDefaultValueConstantName() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        return $this->__defaultConst === false ? null : $this->__defaultConst;
    }
    public static function __fromInfo($p) { $r = new ReflectionParameter(); $r->__init($p); return $r; }
    public function getName() { return $this->name; }
    public function getPosition() { return $this->__pos; }
    public function isOptional() { return $this->__optional; }
    public function isVariadic() { return $this->__variadic; }
    public function isPassedByReference() { return $this->__byref; }
    public function canBePassedByValue() { return !$this->__byref; }
    public function hasType() { return $this->__type !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo($this->__type); }
    public function allowsNull() { return $this->__type === false ? true : $this->__type['nullable']; }
    public function isDefaultValueAvailable() { return $this->__hasDefault; }
    public function getDefaultValue() {
        if (!$this->__hasDefault) {
            throw new ReflectionException('Internal error: Failed to retrieve the default value');
        }
        // A default that could not be evaluated surfaces its error here (lazily),
        // never at ReflectionParameter construction (PHP evaluates on demand).
        if ($this->__defaultError !== false) {
            throw new Error($this->__defaultError);
        }
        return $this->__default;
    }
    // The declaring function/method of this parameter.
    public function getDeclaringFunction() {
        return $this->__declClass !== ''
            ? new ReflectionMethod($this->__declClass, $this->__declFunc)
            : new ReflectionFunction($this->__declFunc);
    }
    // The class the parameter is declared in, or null for a plain function.
    public function getDeclaringClass() {
        return $this->__declClass !== '' ? new ReflectionClass($this->__declClass) : null;
    }
    // Deprecated (pre-8.0): the ReflectionClass of a class-typed parameter, else
    // null. `self`/`parent`/`static` resolve against the declaring class.
    public function getClass() {
        $t = $this->__type;
        if ($t === false || isset($t['kind']) || $t['builtin']) { return null; }
        $n = $t['name'];
        if (strcasecmp($n, 'self') === 0 || strcasecmp($n, 'static') === 0) { $n = $this->__declClass; }
        elseif (strcasecmp($n, 'parent') === 0) { $n = get_parent_class($this->__declClass); }
        return $n === '' || $n === false ? null : new ReflectionClass($n);
    }
    // Deprecated type predicates (pre-8.0): true for a bare `array`/`callable` hint.
    public function isArray() {
        return $this->__type !== false && !isset($this->__type['kind'])
            && strcasecmp($this->__type['name'], 'array') === 0;
    }
    public function isCallable() {
        return $this->__type !== false && !isset($this->__type['kind'])
            && strcasecmp($this->__type['name'], 'callable') === 0;
    }
    // `Parameter #N [ <optional> Type $name = DEFAULT ]` (oracle format).
    // PHPUnit's mock generator parses the piece after ' = ' as *source code*
    // for an object default, so enum cases render `\FQCN::CASE`.
    public function __toString() {
        // A variadic is <optional> without a default; the ` = DEFAULT` suffix is
        // driven separately by whether a default value is available.
        $opt = $this->isOptional();
        $hasDefault = $this->isDefaultValueAvailable();
        $s = 'Parameter #' . $this->getPosition() . ' [ <' . ($opt ? 'optional' : 'required') . '> ';
        $t = $this->getType();
        $ts = '';
        if ($t !== null) {
            if ($t instanceof ReflectionUnionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('|', $parts);
            } elseif ($t instanceof ReflectionIntersectionType) {
                $parts = array();
                foreach ($t->getTypes() as $tt) { $parts[] = $tt->getName(); }
                $ts = implode('&', $parts);
            } else {
                $ts = $t->getName();
                if ($t->allowsNull() && $ts !== 'null' && $ts !== 'mixed') { $ts = '?' . $ts; }
            }
        }
        $s .= ($ts !== '' ? $ts . ' ' : '') . ($this->isPassedByReference() ? '&' : '') . ($this->isVariadic() ? '...' : '') . '$' . $this->getName();
        if ($hasDefault && !$this->isVariadic()) {
            $v = $this->__default;
            if ($v === null) { $d = 'NULL'; }
            elseif (is_object($v)) {
                $d = ($v instanceof UnitEnum) ? '\\' . get_class($v) . '::' . $v->name : 'new \\' . get_class($v) . '(...)';
            }
            elseif (is_array($v)) { $d = str_replace("\n", '', var_export($v, true)); }
            else { $d = var_export($v, true); }
            $s .= ' = ' . $d;
        }
        return $s . ' ]';
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_param_attributes($this->__declClass, $this->__declFunc, $this->__pos, $hostName), $name, $flags, 'ReflectionParameter');
    }
}
class ReflectionObject extends ReflectionClass {
    // The reflected instance is held host-side (keyed by this object's id) rather
    // than as a property, so a var_dump of a ReflectionObject shows only `name`
    // (matching Zend, which keeps the pointer internally).
    public function __construct($object) {
        parent::__construct($object);
        if (is_object($object)) { __reflect_object_bind($this, $object); }
    }
    // Full export format with the object's dynamic properties. The dynamic-prop
    // names come from a host that reads the property table directly, so a lazy
    // object is NOT initialized (init_trigger_reflection_object_toString).
    public function __toString() {
        return $this->__exportString(true, __reflect_object_dynprops($this));
    }
}
// A reference an array element holds (7.4). `fromArrayElement` returns null when
// the element is not a `&`-reference; `getId` yields the same string for two
// elements aliasing the same reference (Symfony var-exporter / deepclone rely on
// this to preserve shared references across a clone).
final class ReflectionReference {
    private $id;
    private function __construct() {}
    public static function fromArrayElement(array $array, int|string $key): ?ReflectionReference {
        if (!array_key_exists($key, $array)) {
            throw new ReflectionException(sprintf('Key "%s" does not exist', $key));
        }
        $id = __reflect_ref_id($array, $key);
        if ($id === false) { return null; }
        $ref = new ReflectionReference();
        $ref->id = $id;
        return $ref;
    }
    public function getId(): string { return $this->id; }
}
// Reflects a suspended Generator: its current execution point, bound $this and
// the underlying function. Backed by the host, which reads the parked frame.
final class ReflectionGenerator {
    private $__gen;
    public function __construct(Generator $generator) { $this->__gen = $generator; }
    public function getExecutingLine(): int { return __reflect_gen_info($this->__gen)[0]; }
    public function getExecutingFile() { return __reflect_gen_info($this->__gen)[1]; }
    public function getThis() { return __reflect_gen_info($this->__gen)[2]; }
    public function getFunction(): ReflectionFunctionAbstract {
        $name = __reflect_gen_info($this->__gen)[3];
        if (strpos($name, '::') !== false) {
            [$c, $m] = explode('::', $name, 2);
            return new ReflectionMethod($c, $m);
        }
        return new ReflectionFunction($name);
    }
    public function getExecutingGenerator(): Generator { return $this->__gen; }
    public function isClosed(): bool { return __reflect_gen_info($this->__gen)[4]; }
    // The suspended call stack is not modelled; return an empty trace rather than
    // erroring (best-effort; the other accessors are exact).
    public function getTrace($options = 1): array { return []; }
}
// Reflects a suspended Fiber: its current execution point and the callable it
// runs. Backed by the host, which reads the fiber's parked frame.
final class ReflectionFiber {
    private $__fiber;
    public function __construct(Fiber $fiber) { $this->__fiber = $fiber; }
    public function getFiber(): Fiber { return $this->__fiber; }
    public function getExecutingLine(): int { return __reflect_fiber_info($this->__fiber)[0]; }
    public function getExecutingFile() { return __reflect_fiber_info($this->__fiber)[1]; }
    public function getCallable(): callable { return __reflect_fiber_info($this->__fiber)[2]; }
    // The suspended call stack is not modelled (see ReflectionGenerator::getTrace).
    public function getTrace($options = 1): array { return []; }
}
class ReflectionConstant {
    public $name;
    public function __construct($name) {
        if (!defined($name)) {
            throw new ReflectionException(sprintf('Constant "%s" does not exist', $name));
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getValue() { return constant($this->name); }
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_const_attributes($this->name, $hostName), $name, $flags, 'ReflectionConstant');
    }
    public function __toString() { return sprintf("Constant [ %s ]\n", $this->name); }
}
class ReflectionClassConstant implements Reflector {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_FINAL = 32;
    public $name;
    public $class;
    public $__info;
    public function __construct($class, $constant) {
        $cls = is_object($class) ? get_class($class) : $class;
        $info = __reflect_class_const_info($cls, $constant);
        if ($info === false) {
            throw new ReflectionException(sprintf('Constant %s::%s does not exist', $cls, $constant));
        }
        $this->name = $constant;
        $this->class = $info['declaringClass'];
        $this->__info = $info;
    }
    public function getName() { return $this->name; }
    public function getValue() { return $this->__info['value']; }
    public function getDeclaringClass() { return new ReflectionClass($this->class); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isFinal() { return $this->__info['final']; }
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    // phpr does not retain a class-constant's doc comment; false (as PHP for one
    // without a `/** */` block).
    public function getDocComment() { return false; }
    public function isEnumCase() { return $this->__info['enumCase']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['final']) { $m |= self::IS_FINAL; }
        return $m;
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_classconst_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionClassConstant');
    }
    // `Constant [ VIS TYPE NAME ] { VALUE }` (ext/reflection golden output). The
    // type shown matches the declared type for a typed constant and the value's
    // type for an untyped one; for scalars they coincide, so derive from the value.
    public function __toString() {
        $vis = $this->__info['visibility'];
        $v = $this->getValue();
        $tmap = ['integer' => 'int', 'double' => 'float', 'boolean' => 'bool', 'NULL' => 'null'];
        $t = gettype($v);
        $type = $tmap[$t] ?? $t;
        if (is_bool($v)) { $val = $v ? 'true' : 'false'; }
        elseif ($v === null) { $val = ''; }
        elseif (is_string($v)) { $val = $v; }
        else { $val = (string) $v; }
        return "Constant [ $vis $type {$this->name} ] { $val }\n";
    }
}
class ReflectionEnumUnitCase extends ReflectionClassConstant {
    // getValue() is inherited: __reflect_class_const_info returns the case
    // singleton as the constant's value.
    public function getEnum() { return new ReflectionEnum($this->class); }
    public function getDocComment() { return false; }
}
class ReflectionEnumBackedCase extends ReflectionEnumUnitCase {
    public function getBackingValue() { return $this->getValue()->value; }
}
class ReflectionEnum extends ReflectionClass {
    public function isBacked() { return in_array('BackedEnum', class_implements($this->name)); }
    public function getBackingType() { return ReflectionNamedType::__fromInfo(__reflect_enum_backing($this->name)); }
    public function hasCase($name) {
        $cls = $this->name;
        foreach ($cls::cases() as $c) { if ($c->name === $name) { return true; } }
        return false;
    }
    public function getCase($name) {
        if (!$this->hasCase($name)) {
            throw new ReflectionException(sprintf('Case %s::%s does not exist', $this->name, $name));
        }
        return $this->isBacked()
            ? new ReflectionEnumBackedCase($this->name, $name)
            : new ReflectionEnumUnitCase($this->name, $name);
    }
    public function getCases() {
        $out = [];
        $cls = $this->name;
        $backed = $this->isBacked();
        foreach ($cls::cases() as $c) {
            $out[] = $backed
                ? new ReflectionEnumBackedCase($this->name, $c->name)
                : new ReflectionEnumUnitCase($this->name, $c->name);
        }
        return $out;
    }
}
abstract class ReflectionFunctionAbstract implements Reflector {}
class ReflectionFunction extends ReflectionFunctionAbstract {
    public $name;
    public $__info;
    public $__closure;
    public function __construct($name) {
        if ($name instanceof Closure) {
            $this->name = '{closure}';
            $this->__closure = $name;
            $this->__info = __reflect_closure_info($name);
        } else {
            $this->name = is_string($name) ? $name : '{closure}';
            $this->__info = __reflect_func_info($this->name);
        }
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Function %s() does not exist', $this->name));
        }
    }
    public function getName() { return $this->name; }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    public function getFileName() { return $this->__info['file'] ?? false; }
    public function getStartLine() { return $this->__info['startLine'] ?? false; }
    public function getEndLine() { return $this->__info['endLine'] ?? false; }
    public function isUserDefined() { return ($this->__info['file'] ?? false) !== false; }
    public function isInternal() { return ($this->__info['file'] ?? false) === false; }
    public function isClosure() { return $this->__closure !== null; }
    public function isAnonymous() { return $this->__closure !== null; }
    // Closure binding surface: null for a named-function reflection.
    public function getClosureThis() {
        return $this->__closure !== null ? __reflect_closure_bind($this->__closure)[0] : null;
    }
    public function getClosureScopeClass() {
        if ($this->__closure === null) { return null; }
        $s = __reflect_closure_bind($this->__closure)[1];
        return $s === null ? null : new ReflectionClass($s);
    }
    public function isStatic() {
        return $this->__closure !== null && __reflect_closure_bind($this->__closure)[2];
    }
    public function getClosureUsedVariables() {
        return $this->__closure !== null ? __reflect_closure_uses($this->__closure) : [];
    }
    public function getStaticVariables() { return __reflect_static_vars(null, $this->name); }
    public function isGenerator() { return $this->__info['isGenerator'] ?? false; }
    // Deprecated via the #[\Deprecated] attribute (8.4).
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function invoke(...$args) { return call_user_func_array($this->name, $args); }
    public function invokeArgs($args) { return call_user_func_array($this->name, $args); }
    // The underlying closure (for a Closure-reflection) or the named function
    // wrapped as one.
    public function getClosure() {
        return $this->__closure !== null ? $this->__closure : Closure::fromCallable($this->name);
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        $all = $this->__closure !== null
            ? __reflect_closure_attributes($this->__closure, $hostName)
            : __reflect_func_attributes($this->name, $hostName);
        return ReflectionAttribute::__filter($all, $name, $flags, 'ReflectionFunctionAbstract');
    }
    // The PHP `Reflection::export` string for a function (ext/reflection golden output).
    public function __toString() {
        $src = ($this->__info['file'] ?? false) !== false ? '<user>' : '<internal>';
        $s = "Function [ $src function {$this->name} ] {\n";
        if (($this->__info['file'] ?? false) !== false) {
            $s .= "  @@ " . $this->__info['file'] . " " . ($this->__info['startLine'] ?? 0) . " - " . ($this->__info['endLine'] ?? 0) . "\n";
        }
        $params = $this->getParameters();
        if (count($params) > 0 || $this->hasReturnType()) {
            $s .= "\n  - Parameters [" . count($params) . "] {\n";
            foreach ($params as $p) { $s .= "    " . $p . "\n"; }
            $s .= "  }\n";
            if ($this->hasReturnType()) { $s .= "  - Return [ " . $this->getReturnType() . " ]\n"; }
        }
        return $s . "}\n";
    }
}
class ReflectionMethod extends ReflectionFunctionAbstract {
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_STATIC = 16;
    const IS_FINAL = 32;
    const IS_ABSTRACT = 64;
    public $name;
    public $class;
    public $__info;
    public function getName() { return $this->name; }
    public function getModifiers() {
        $bits = 0;
        if ($this->isPublic()) { $bits |= 1; }
        if ($this->isProtected()) { $bits |= 2; }
        if ($this->isPrivate()) { $bits |= 4; }
        if ($this->isStatic()) { $bits |= 16; }
        if ($this->isFinal()) { $bits |= 32; }
        if ($this->isAbstract()) { $bits |= 64; }
        return $bits;
    }
    public function getParameters() {
        $out = [];
        foreach ($this->__info['params'] as $p) { $out[] = ReflectionParameter::__fromInfo($p); }
        return $out;
    }
    public function getNumberOfParameters() { return count($this->__info['params']); }
    public function getNumberOfRequiredParameters() {
        $n = 0;
        foreach ($this->__info['params'] as $p) { if (!$p['optional']) { $n++; } }
        return $n;
    }
    public function isVariadic() {
        foreach ($this->__info['params'] as $p) { if ($p['variadic']) { return true; } }
        return false;
    }
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function getReturnType() { return ReflectionNamedType::__fromInfo($this->__info['returnType']); }
    public function hasReturnType() { return $this->__info['returnType'] !== false; }
    // The hierarchy annotation Zend appends inside `<user...>` (see
    // ext/reflection _function_string): `inherits X` when the body lives in an
    // ancestor, `overwrites X` when this class redeclares a parent method, and
    // `prototype X` for the topmost interface/abstract origin. Order matches Zend:
    // inherits|overwrites, then prototype, then ctor.
    private function __hierAnnot() {
        $scope = $this->class;
        $declaring = $this->__info['declaringClass'];
        $out = '';
        if (strcasecmp($declaring, $scope) !== 0) {
            $out .= ", inherits $declaring";
        } else {
            $parent = get_parent_class($scope);
            if ($parent !== false && method_exists($parent, $this->name)) {
                $pm = new ReflectionMethod($parent, $this->name);
                $ps = $pm->getDeclaringClass()->getName();
                if (strcasecmp($ps, $scope) !== 0 && !$pm->isPrivate()) {
                    $out .= ", overwrites $ps";
                }
            }
        }
        $proto = $this->__protoScope();
        if ($proto !== null && strcasecmp($proto, $scope) !== 0) { $out .= ", prototype $proto"; }
        if ($this->isConstructor()) { $out .= ', ctor'; }
        return $out;
    }
    // Zend's `fptr->common.prototype->common.scope`: the topmost class or
    // interface that declares this method (independent of which subclass reflects
    // it, so computed from the declaring class). Interfaces are the ultimate
    // origin; among ancestor classes the furthest one wins. Returns null when the
    // method has no inherited counterpart (its prototype pointer is NULL in Zend).
    private function __protoScope() {
        $name = $this->name;
        $declaring = $this->__info['declaringClass'];
        // A root interface method that declares this name wins as the prototype.
        foreach (class_implements($declaring) as $iface) {
            if (method_exists($iface, $name)) {
                $rm = new ReflectionMethod($iface, $name);
                $root = $rm->getDeclaringClass()->getName();
                // topmost interface: one whose own super-interfaces do not declare it
                $topmost = true;
                foreach (class_implements($root) as $sup) {
                    if (strcasecmp($sup, $root) !== 0 && method_exists($sup, $name)) { $topmost = false; break; }
                }
                if ($topmost) { return $root; }
            }
        }
        // Otherwise the furthest ancestor class that declares it.
        $best = null;
        for ($p = get_parent_class($declaring); $p !== false; $p = get_parent_class($p)) {
            if (method_exists($p, $name)) {
                $rm = new ReflectionMethod($p, $name);
                if ($rm->isPrivate()) { break; }
                $best = $rm->getDeclaringClass()->getName();
            }
        }
        return $best;
    }
    // The PHP `Reflection::export` string for a method (ext/reflection golden output).
    public function __toString() {
        $src = $this->isUserDefined() ? ('<user' . $this->__hierAnnot() . '>') : ('<internal' . $this->__hierAnnot() . '>');
        $mods = '';
        if ($this->isAbstract()) { $mods .= 'abstract '; }
        if ($this->isFinal()) { $mods .= 'final '; }
        if ($this->isStatic()) { $mods .= 'static '; }
        $vis = $this->isPublic() ? 'public' : ($this->isProtected() ? 'protected' : 'private');
        $s = "Method [ $src {$mods}{$vis} method {$this->name} ] {\n";
        // An internal (prelude) method has no source location, so PHP omits the `@@`.
        if ($this->isUserDefined()) {
            $s .= "  @@ " . $this->getFileName() . " " . $this->getStartLine() . " - " . $this->getEndLine() . "\n";
        }
        if (!$this->isAbstract()) {
            $params = $this->getParameters();
            // The parameters/return block appears only when there is something in it.
            if (count($params) > 0 || $this->hasReturnType()) {
                $s .= "\n  - Parameters [" . count($params) . "] {\n";
                foreach ($params as $p) { $s .= "    " . $p . "\n"; }
                $s .= "  }\n";
                if ($this->hasReturnType()) { $s .= "  - Return [ " . $this->getReturnType() . " ]\n"; }
            }
        }
        return $s . "}\n";
    }
    public function __construct($objectOrClass, $method = null) {
        // A class-name string autoloads (Zend does; the info lookup below is
        // autoload-blind and would report "does not exist" for a not-yet-loaded
        // class).
        if (is_string($objectOrClass)) { class_exists(strpos($objectOrClass, '::') !== false ? explode('::', $objectOrClass, 2)[0] : $objectOrClass); }
        if ($method === null && is_string($objectOrClass) && strpos($objectOrClass, '::') !== false) {
            $parts = explode('::', $objectOrClass, 2);
            $objectOrClass = $parts[0]; $method = $parts[1];
        }
        $this->class = is_object($objectOrClass) ? get_class($objectOrClass) : $objectOrClass;
        $this->name = $method;
        $this->__info = __reflect_method_info($this->class, $method);
        if ($this->__info === false) {
            throw new ReflectionException(sprintf('Method %s::%s() does not exist', $this->class, $method));
        }
    }
    public function isConstructor() { return strcasecmp($this->name, '__construct') === 0; }
    // Named constructor (8.3): build from a "Class::method" string.
    public static function createFromMethodName(string $method): static {
        $parts = explode('::', $method, 2);
        if (count($parts) !== 2) {
            throw new ReflectionException(sprintf('%s is not a valid method name', $method));
        }
        return new static($parts[0], $parts[1]);
    }
    public function hasPrototype() { return $this->__protoScope() !== null; }
    public function getPrototype() {
        $proto = $this->__protoScope();
        if ($proto === null) {
            throw new ReflectionException(sprintf('Method %s::%s does not have a prototype', $this->class, $this->name));
        }
        return new ReflectionMethod($proto, $this->name);
    }
    public function returnsReference() { return $this->__info['byRef'] ?? false; }
    // A method is deprecated when it carries the #[\Deprecated] attribute (8.4);
    // not inherited (an override without the attribute is not deprecated).
    public function isDeprecated() { return count($this->getAttributes('Deprecated')) > 0; }
    public function getStaticVariables() { return __reflect_static_vars($this->class, $this->name); }
    public function isGenerator() { return $this->__info['isGenerator'] ?? false; }
    public function hasTentativeReturnType() { return false; }
    public function getTentativeReturnType() { return null; }
    public function isUserDefined() { return ($this->__info['file'] ?? false) !== false; }
    public function isInternal() { return ($this->__info['file'] ?? false) === false; }
    public function isDestructor() { return strcasecmp($this->name, '__destruct') === 0; }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function getFileName() { return $this->__info['file']; }
    public function getStartLine() { return $this->__info['startLine']; }
    public function getEndLine() { return $this->__info['endLine']; }
    public function isStatic() { return $this->__info['static']; }
    public function isFinal() { return $this->__info['final']; }
    public function isAbstract() { return $this->__info['abstract']; }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function setAccessible($accessible) {}
    public function invoke($object, ...$args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    public function invokeArgs($object, $args) {
        return __reflect_invoke($object, $this->class, $this->name, $args);
    }
    // A Closure bound to the method: a static method ignores $object, an instance
    // method binds it (8.0+ allows null for a static method).
    public function getClosure($object = null) {
        return $this->isStatic()
            ? Closure::fromCallable($this->class . '::' . $this->name)
            : Closure::fromCallable([$object, $this->name]);
    }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_method_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionFunctionAbstract');
    }
}
class ReflectionProperty implements Reflector {
    public function getDocComment() { return $this->__info['doc'] ?? false; }
    public function isFinal() { return false; }
    public function isAbstract() { return false; }
    // Asymmetric visibility (8.4): phpr does not model aviz declarations, so
    // every property reports symmetric get/set (ORM's RuntimeReflectionService
    // probes these on each mapped field).
    public function isProtectedSet() { return false; }
    public function isPrivateSet() { return false; }
    public function isVirtual() { return false; }
    public function hasHooks() { return false; }
    public function getHooks() { return []; }
    public function hasHook($type) { return false; }
    public function getHook($type) { return null; }
    public function isDynamic() { return $this->__info['dynamic'] ?? false; }
    // A declared property is a "default" property; a dynamic one is not.
    public function isDefault() { return !$this->isDynamic(); }
    // Constructor-promoted iff the declaring class's constructor has a promoted
    // parameter of the same name (derived from ReflectionParameter::isPromoted).
    public function isPromoted() {
        $ctor = (new ReflectionClass($this->class))->getConstructor();
        if ($ctor === null) { return false; }
        foreach ($ctor->getParameters() as $p) {
            if (strcasecmp($p->getName(), $this->name) === 0 && $p->isPromoted()) { return true; }
        }
        return false;
    }
    // The property's storage key in the object's property table: public keeps its
    // name, protected is `\0*\0name`, private is `\0DeclaringClass\0name` (matches
    // a `(array)` cast of the instance).
    public function getMangledName() {
        if ($this->isPublic()) { return $this->name; }
        if ($this->isProtected()) { return "\0*\0" . $this->name; }
        return "\0" . $this->__info['declaringClass'] . "\0" . $this->name;
    }
    // Without asymmetric visibility modelled, the settable type equals the type.
    public function getSettableType() { return $this->getType(); }
    public function setAccessible($accessible) {}
    public function isLazy($object = null) {
        return $object !== null && __lazy_prop_is_lazy($object, $this->class, $this->name);
    }
    const IS_STATIC = 16;
    const IS_PUBLIC = 1;
    const IS_PROTECTED = 2;
    const IS_PRIVATE = 4;
    const IS_READONLY = 128;
    const IS_PROTECTED_SET = 2048;
    const IS_PRIVATE_SET = 4096;
    public $name;
    public $class;
    public $__info;
    public function __construct($class, $property) {
        $cls = is_object($class) ? get_class($class) : $class;
        // A dynamic property is reflectable when the INSTANCE is given.
        if (!property_exists($cls, $property)
            && !(is_object($class) && property_exists($class, $property))) {
            throw new ReflectionException(sprintf('Property %s::$%s does not exist', $cls, $property));
        }
        // The declaring class is the most-derived class that declares the property
        // itself (a child redeclaration shadows the parent's); mirrors
        // ReflectionProperty::$class. The host resolves it from the per-class
        // declared-property lists, which `property_exists` (inherited too) can't.
        $this->name = $property;
        $decl = __reflect_prop_declaring_class($cls, $property);
        $this->class = $decl === false ? $cls : $decl;
        $this->__info = __reflect_prop_details($this->class, $this->name);
        // An ancestor's PRIVATE property is invisible to the subclass's
        // reflection surface (Zend: getProperty throws; realize_skipped).
        if (is_array($this->__info) && isset($this->__info['visibility'])
            && $this->__info['visibility'] === 'private'
            && strcasecmp($this->class, $cls) !== 0) {
            throw new ReflectionException(sprintf('Property %s::$%s does not exist', $cls, $property));
        }
        if (!is_array($this->__info)) {
            // A dynamic property: public, untyped, no default.
            $this->__info = ['visibility' => 'public', 'static' => false, 'readonly' => false,
                'hasDefault' => false, 'default' => null, 'declaringClass' => $this->class,
                'dynamic' => true];
        }
    }
    public function getName() { return $this->name; }
    public function getValue($object = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            return __reflect_static_prop_get($this->class, $this->name);
        }
        return __reflect_prop_get($this->class, $this->name, $object);
    }
    public function setValue($object, $value = null) {
        if (__reflect_prop_is_static($this->class, $this->name)) {
            // Static form: setValue($value) and setValue(null, $value) both write
            // the class-level slot (Composer pokes InstalledVersions::$selfDir).
            if (func_num_args() === 1) { $value = $object; }
            __reflect_static_prop_set($this->class, $this->name, $value);
            return;
        }
        __reflect_prop_set($this->class, $this->name, $object, $value);
    }
    // PHP 8.4 raw accessors bypass property hooks; phpr's reflection reads the
    // backing slots directly already, so they alias get/setValue.
    public function getRawValue($object) { return $this->getValue($object); }
    public function setRawValue($object, $value) { $this->setValue($object, $value); }
    public function getAttributes($name = null, $flags = 0) {
        $hostName = ($flags & ReflectionAttribute::IS_INSTANCEOF) ? null : $name;
        return ReflectionAttribute::__filter(__reflect_prop_attributes($this->class, $this->name, $hostName), $name, $flags, 'ReflectionProperty');
    }
    public function isStatic() { return __reflect_prop_is_static($this->class, $this->name); }
    public function hasType() { return __reflect_prop_type($this->class, $this->name) !== false; }
    public function getType() { return ReflectionNamedType::__fromInfo(__reflect_prop_type($this->class, $this->name)); }
    public function isPublic() { return $this->__info['visibility'] === 'public'; }
    public function isProtected() { return $this->__info['visibility'] === 'protected'; }
    public function isPrivate() { return $this->__info['visibility'] === 'private'; }
    public function isReadOnly() { return $this->__info['readonly']; }
    public function getModifiers() {
        $m = 0;
        if ($this->__info['visibility'] === 'public') { $m |= self::IS_PUBLIC; }
        elseif ($this->__info['visibility'] === 'protected') { $m |= self::IS_PROTECTED; }
        else { $m |= self::IS_PRIVATE; }
        if ($this->__info['static']) { $m |= self::IS_STATIC; }
        if ($this->__info['readonly']) {
            $m |= self::IS_READONLY;
            // PHP 8.4: a `public readonly` property's implicit set-visibility is
            // downgraded to protected (asymmetric visibility), adding IS_PROTECTED_SET.
            if ($this->__info['visibility'] === 'public') { $m |= self::IS_PROTECTED_SET; }
        }
        return $m;
    }
    public function getDeclaringClass() { return new ReflectionClass($this->__info['declaringClass']); }
    public function hasDefaultValue() { return $this->__info['hasDefault']; }
    public function getDefaultValue() { return $this->__info['default']; }
    // `Property [ VIS [static ][readonly ]TYPE $name[ = DEFAULT] ]` (export format).
    // Asymmetric set-visibility (`protected(set)`) is not modelled, so a readonly
    // property renders without it.
    public function __toString() {
        $mods = $this->isPublic() ? 'public' : ($this->isProtected() ? 'protected' : 'private');
        if ($this->isStatic()) { $mods .= ' static'; }
        if ($this->isReadOnly()) { $mods .= ' readonly'; }
        $type = '';
        if ($this->hasType()) {
            $t = $this->getType();
            if ($t instanceof ReflectionNamedType) {
                $tn = $t->getName();
                if ($t->allowsNull() && $tn !== 'null' && $tn !== 'mixed') { $tn = '?' . $tn; }
            } else {
                $tn = (string) $t;
            }
            $type = $tn . ' ';
        }
        $s = "Property [ $mods {$type}\$" . $this->name;
        if ($this->hasDefaultValue()) {
            $v = $this->getDefaultValue();
            if ($v === null) { $d = 'NULL'; }
            elseif (is_array($v)) { $d = str_replace("\n", '', var_export($v, true)); }
            else { $d = var_export($v, true); }
            $s .= ' = ' . $d;
        }
        return $s . " ]\n";
    }
    public function isInitialized($object = null) { return __reflect_prop_initialized($this->class, $this->name, $object); }
    public function skipLazyInitialization($object) {
        $msg = __lazy_skip_init($object, $this->class, $this->name);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
    public function setRawValueWithoutLazyInitialization($object, $value) {
        $msg = __lazy_set_raw($object, $this->class, $this->name, $value);
        if ($msg !== null) { throw new ReflectionException($msg); }
    }
}
class ReflectionExtension implements Reflector {
    public $name;
    public function __construct($name) {
        if (!extension_loaded($name)) {
            throw new ReflectionException(sprintf('Extension "%s" does not exist', $name));
        }
        // Canonical casing as get_loaded_extensions reports it, mirroring
        // ReflectionExtension::getName().
        foreach (get_loaded_extensions() as $ext) {
            if (strcasecmp($ext, $name) === 0) { $name = $ext; break; }
        }
        $this->name = $name;
    }
    public function getName() { return $this->name; }
    public function getVersion() { return phpversion($this->name); }
    // phpr's registered extensions are all statically built in, never runtime-loaded.
    public function isPersistent() { return true; }
    public function isTemporary() { return false; }
    public function getDependencies() { return []; }
    // info() prints the phpinfo() block for the extension. phpr models these
    // extensions with Rust crates, not the C internals whose text this reports,
    // so it emits nothing (the OpenSSL-text-rendering class of rabbit hole);
    // callers parse it defensively (e.g. Composer regexes for an optional
    // sub-library version and simply skips it when absent).
    public function info() {}
    public function __toString() { return ''; }
    // Constants per extension. pcntl's table is what real consumers read
    // (monolog's SignalHandler maps signo -> "SIG*" name through it); values
    // are the macOS oracle's. Other extensions read as empty for now.
    public function getConstants() {
        if (strcasecmp($this->name, 'pcntl') === 0) {
            return [
                'WNOHANG' => 1, 'WUNTRACED' => 2, 'WCONTINUED' => 16, 'WEXITED' => 4,
                'WSTOPPED' => 8, 'WNOWAIT' => 32, 'P_ALL' => 0, 'P_PID' => 1, 'P_PGID' => 2,
                'SIG_IGN' => 1, 'SIG_DFL' => 0, 'SIG_ERR' => -1, 'SIGHUP' => 1, 'SIGINT' => 2,
                'SIGQUIT' => 3, 'SIGILL' => 4, 'SIGTRAP' => 5, 'SIGABRT' => 6, 'SIGIOT' => 6,
                'SIGBUS' => 10, 'SIGFPE' => 8, 'SIGKILL' => 9, 'SIGUSR1' => 30, 'SIGSEGV' => 11,
                'SIGUSR2' => 31, 'SIGPIPE' => 13, 'SIGALRM' => 14, 'SIGTERM' => 15,
                'SIGCHLD' => 20, 'SIGCONT' => 19, 'SIGSTOP' => 17, 'SIGTSTP' => 18,
                'SIGTTIN' => 21, 'SIGTTOU' => 22, 'SIGURG' => 16, 'SIGXCPU' => 24,
                'SIGXFSZ' => 25, 'SIGVTALRM' => 26, 'SIGPROF' => 27, 'SIGWINCH' => 28,
                'SIGIO' => 23, 'SIGINFO' => 29, 'SIGSYS' => 12, 'SIGBABY' => 12,
                'PRIO_PGRP' => 1, 'PRIO_USER' => 2, 'PRIO_PROCESS' => 0,
                'PRIO_DARWIN_BG' => 4096, 'PRIO_DARWIN_THREAD' => 3, 'SIG_BLOCK' => 1,
                'SIG_UNBLOCK' => 2, 'SIG_SETMASK' => 3, 'PCNTL_EINTR' => 4,
                'PCNTL_ECHILD' => 10, 'PCNTL_EINVAL' => 22, 'PCNTL_EAGAIN' => 35,
                'PCNTL_ESRCH' => 3, 'PCNTL_EACCES' => 13, 'PCNTL_EPERM' => 1,
                'PCNTL_ENOMEM' => 12, 'PCNTL_E2BIG' => 7, 'PCNTL_EFAULT' => 14,
                'PCNTL_EIO' => 5, 'PCNTL_EISDIR' => 21, 'PCNTL_ELOOP' => 62,
                'PCNTL_EMFILE' => 24, 'PCNTL_ENAMETOOLONG' => 63, 'PCNTL_ENFILE' => 23,
                'PCNTL_ENOENT' => 2, 'PCNTL_ENOEXEC' => 8, 'PCNTL_ENOTDIR' => 20,
                'PCNTL_ETXTBSY' => 26, 'PCNTL_ENOSPC' => 28, 'PCNTL_EUSERS' => 68,
            ];
        }
        return [];
    }
}
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

// ----- ext/dom (host arena behind the __dom_* builtins; see vm/dom.rs) -----
// Node identity is the (docId, nodeId) handle pair; every accessor re-wraps a
// fresh PHP object around the handle (isSameNode compares handles, as ext/dom
// offers for the same reason).
class DOMException extends Exception {}

class LibXMLError {
    public $level = 0;
    public $code = 0;
    public $column = 0;
    public $message = '';
    public $file = '';
    public $line = 0;
}
function is_countable($value) { return is_array($value) || $value instanceof Countable; }
function libxml_get_errors() {
    $out = array();
    foreach (__libxml_get_errors() as $e) {
        $o = new LibXMLError();
        $o->level = $e['level']; $o->code = $e['code']; $o->column = $e['column'];
        $o->message = $e['message']; $o->file = $e['file']; $o->line = $e['line'];
        $out[] = $o;
    }
    return $out;
}
function libxml_get_last_error() {
    $all = libxml_get_errors();
    $n = count($all);
    return $n > 0 ? $all[$n - 1] : false;
}

class DOMNodeList implements IteratorAggregate, Countable {
    public $length = 0;
    public $__items = array();
    public static function __make($items) {
        $l = new DOMNodeList();
        $l->__items = $items;
        $l->length = count($items);
        return $l;
    }
    public function item($index) { return isset($this->__items[$index]) ? $this->__items[$index] : null; }
    public function count(): int { return $this->length; }
    public function getIterator(): Iterator { return new ArrayIterator($this->__items); }
}

class DOMNamedNodeMap implements IteratorAggregate, Countable {
    public $length = 0;
    public $__items = array(); // name => DOMAttr
    public static function __make($items) {
        $m = new DOMNamedNodeMap();
        $m->__items = $items;
        $m->length = count($items);
        return $m;
    }
    public function getNamedItem($name) { return isset($this->__items[$name]) ? $this->__items[$name] : null; }
    public function item($index) {
        $i = 0;
        foreach ($this->__items as $v) { if ($i === (int)$index) return $v; $i++; }
        return null;
    }
    public function count(): int { return $this->length; }
    public function getIterator(): Iterator { return new ArrayIterator($this->__items); }
}

class DOMNode {
    public $__d = -1;
    public $__n = -1;
    public static function __wrap($d, $n) {
        if ($d < 0 || $n < 0) { return null; }
        $i = __dom_info($d, $n);
        switch ($i[0]) {
            case 1: $c = 'DOMElement'; break;
            case 3: $c = 'DOMText'; break;
            case 4: $c = 'DOMCdataSection'; break;
            case 7: $c = 'DOMProcessingInstruction'; break;
            case 8: $c = 'DOMComment'; break;
            case 9: $c = 'DOMDocument'; break;
            case 10: $c = 'DOMDocumentType'; break;
            case 11: $c = 'DOMDocumentFragment'; break;
            default: $c = 'DOMNode';
        }
        $r = new ReflectionClass($c);
        $o = $r->newInstanceWithoutConstructor();
        $o->__d = $d;
        $o->__n = $n;
        return $o;
    }
    public function __get($name) {
        switch ($name) {
            case 'nodeType': $i = __dom_info($this->__d, $this->__n); return $i[0];
            case 'nodeName': $i = __dom_info($this->__d, $this->__n); return $i[1];
            case 'nodeValue':
                $i = __dom_info($this->__d, $this->__n);
                // ext/dom: an element/fragment reports its text content here
                // (xmlNodeGetContent); a document reports NULL.
                if ($i[0] === 1 || $i[0] === 11) { return __dom_text($this->__d, $this->__n); }
                return $i[2];
            case 'textContent': return __dom_text($this->__d, $this->__n);
            case 'parentNode': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 0));
            case 'firstChild': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 1));
            case 'lastChild': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 2));
            case 'nextSibling': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 3));
            case 'previousSibling': return DOMNode::__wrap($this->__d, __dom_nav($this->__d, $this->__n, 4));
            case 'ownerDocument':
                $i = __dom_info($this->__d, $this->__n);
                return $i[0] === 9 ? null : DOMNode::__wrap($this->__d, 0);
            case 'childNodes':
                $items = array();
                foreach (__dom_children($this->__d, $this->__n) as $c) {
                    $items[] = DOMNode::__wrap($this->__d, $c);
                }
                return DOMNodeList::__make($items);
            case 'attributes':
                $i = __dom_info($this->__d, $this->__n);
                if ($i[0] !== 1) { return null; }
                $items = array();
                foreach (__dom_attr($this->__d, $this->__n, 4, '', '') as $an) {
                    $items[$an] = DOMAttr::__wrapAttr($this->__d, $this->__n, $an);
                }
                return DOMNamedNodeMap::__make($items);
            case 'namespaceURI': case 'prefix': case 'localName':
                $r = __dom_ns($this->__d, $this->__n, '');
                if ($name === 'namespaceURI') { return $r[0]; }
                if ($name === 'prefix') { return $r[1]; }
                return $r[2];
            case 'baseURI':
                return null;
        }
        return null;
    }
    public function __set($name, $value) {
        if ($name === 'nodeValue' || $name === 'textContent') {
            __dom_set_value($this->__d, $this->__n, (string)$value);
        }
        // Other magic props are read-only in ext/dom; silently ignore like a no-op.
    }
    public function appendChild($node) {
        if ($node->__d !== $this->__d) { throw new DOMException('Wrong Document Error'); }
        if (!__dom_mutate($this->__d, 0, $this->__n, $node->__n, -1)) {
            throw new DOMException('Hierarchy Request Error');
        }
        return $node;
    }
    public function insertBefore($node, $refNode = null) {
        if ($node->__d !== $this->__d) { throw new DOMException('Wrong Document Error'); }
        $ref = $refNode === null ? -1 : $refNode->__n;
        if (!__dom_mutate($this->__d, 1, $this->__n, $node->__n, $ref)) {
            throw new DOMException('Hierarchy Request Error');
        }
        return $node;
    }
    public function removeChild($node) {
        if (!__dom_mutate($this->__d, 2, $this->__n, $node->__n, -1)) {
            throw new DOMException('Not Found Error');
        }
        return $node;
    }
    public function replaceChild($newNode, $oldNode) {
        $this->insertBefore($newNode, $oldNode);
        return $this->removeChild($oldNode);
    }
    public function cloneNode($deep = false) {
        $n = __dom_copy($this->__d, $this->__d, $this->__n, $deep ? 1 : 0);
        return DOMNode::__wrap($this->__d, $n);
    }
    public function hasChildNodes() { return __dom_nav($this->__d, $this->__n, 1) >= 0; }
    public function hasAttributes() {
        $i = __dom_info($this->__d, $this->__n);
        if ($i[0] !== 1) { return false; }
        return count(__dom_attr($this->__d, $this->__n, 4, '', '')) > 0;
    }
    public function isSameNode($other) {
        return $other instanceof DOMNode && $other->__d === $this->__d && $other->__n === $this->__n;
    }
    public function normalize() {}
    public function getLineNo() { return 0; }
    public function getNodePath() { return null; }
    public function lookupNamespaceURI($prefix) { return null; }
    public function lookupPrefix($namespace) { return null; }
    public function isDefaultNamespace($namespace) { return false; }
    public function isSupported($feature, $version) { return false; }
    public function contains($other) {
        if (!($other instanceof DOMNode) || $other->__d !== $this->__d) { return false; }
        $n = $other->__n;
        while ($n >= 0) {
            if ($n === $this->__n) { return true; }
            $n = __dom_nav($this->__d, $n, 0);
        }
        return false;
    }
}

// Class-shape stub of ext/xmlreader's pull parser: enough for code that
// subclasses or type-checks XMLReader (doctrine/instantiator's test assets).
// Actual pull-parsing is out of slice: there are deliberately no methods, so
// any real use fails loudly with "undefined method" instead of misparsing.
class XMLReader {
    const NONE = 0;
    const ELEMENT = 1;
    const ATTRIBUTE = 2;
    const TEXT = 3;
    const CDATA = 4;
    const ENTITY_REF = 5;
    const ENTITY = 6;
    const PI = 7;
    const COMMENT = 8;
    const DOC = 9;
    const DOC_TYPE = 10;
    const DOC_FRAGMENT = 11;
    const NOTATION = 12;
    const WHITESPACE = 13;
    const SIGNIFICANT_WHITESPACE = 14;
    const END_ELEMENT = 15;
    const END_ENTITY = 16;
    const XML_DECLARATION = 17;
    const LOADDTD = 1;
    const DEFAULTATTRS = 2;
    const VALIDATE = 3;
    const SUBST_ENTITIES = 4;
}
class DOMDocument extends DOMNode {
    public $preserveWhiteSpace = true;
    public $formatOutput = false;
    public $validateOnParse = false;
    public $recover = false;
    public $resolveExternals = false;
    public $substituteEntities = false;
    public $strictErrorChecking = true;
    public $documentURI = null;
    public function __construct($version = '1.0', $encoding = '') {
        $this->__d = __dom_new_doc($version, $encoding);
        $this->__n = 0;
    }
    public function loadXML($source, $options = 0) {
        return __dom_load($this->__d, (string)$source, 0);
    }
    public function load($filename, $options = 0) {
        $this->documentURI = (string)$filename;
        return __dom_load($this->__d, (string)$filename, 1);
    }
    public function saveXML($node = null, $options = 0) {
        return __dom_save_xml($this->__d, $node === null ? -1 : $node->__n);
    }
    public function save($filename) {
        return file_put_contents($filename, __dom_save_xml($this->__d, -1));
    }
    public function schemaValidate($filename, $flags = 0) {
        // XSD validation is out of slice: a well-formed document is accepted.
        // (PHPUnit only uses this to warn about an invalid phpunit.xml.)
        return true;
    }
    public function schemaValidateSource($source, $flags = 0) { return true; }
    public function relaxNGValidate($filename) { return true; }
    public function relaxNGValidateSource($source) { return true; }
    public function xinclude($options = 0) {
        // XInclude substitution (PHPUnit's config loader calls this on every
        // phpunit.xml). A document with no XInclude elements is untouched and
        // the count is 0; actual substitution is out of slice, so report -1
        // (libxml's processing-error result) instead of pretending it worked.
        foreach (__dom_by_tag($this->__d, -1, 'xi:include') as $n) { return -1; }
        foreach (__dom_by_tag($this->__d, -1, 'xinclude') as $n) { return -1; }
        return 0;
    }
    public function createElement($localName, $value = '') {
        $n = __dom_create($this->__d, 1, (string)$localName, '');
        if ($n < 0) { throw new DOMException('Invalid Character Error'); }
        if ($value !== '' && $value !== null) {
            $t = __dom_create($this->__d, 3, (string)$value, '');
            __dom_mutate($this->__d, 0, $n, $t, -1);
        }
        return DOMNode::__wrap($this->__d, $n);
    }
    public function createTextNode($data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 3, (string)$data, ''));
    }
    public function createComment($data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 8, (string)$data, ''));
    }
    public function createCDATASection($data) {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 4, (string)$data, ''));
    }
    public function createProcessingInstruction($target, $data = '') {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 7, (string)$target, (string)$data));
    }
    public function createDocumentFragment() {
        return DOMNode::__wrap($this->__d, __dom_create($this->__d, 11, '', ''));
    }
    public function createAttribute($localName) {
        return DOMAttr::__wrapDetached($this->__d, (string)$localName);
    }
    public function getElementsByTagName($qualifiedName) {
        $items = array();
        foreach (__dom_by_tag($this->__d, -1, (string)$qualifiedName) as $n) {
            $items[] = DOMNode::__wrap($this->__d, $n);
        }
        return DOMNodeList::__make($items);
    }
    public function getElementById($elementId) {
        // Without DTD machinery only xml:id qualifies, as in PHP with no DTD.
        foreach (__dom_by_tag($this->__d, -1, '*') as $n) {
            $v = __dom_attr($this->__d, $n, 0, 'xml:id', '');
            if ($v !== false && $v === (string)$elementId) { return DOMNode::__wrap($this->__d, $n); }
        }
        return null;
    }
    public function importNode($node, $deep = false) {
        $n = __dom_copy($this->__d, $node->__d, $node->__n, $deep ? 1 : 0);
        return DOMNode::__wrap($this->__d, $n);
    }
    public function adoptNode($node) { return $this->importNode($node, true); }
    public function __get($name) {
        switch ($name) {
            case 'documentElement': return DOMNode::__wrap($this->__d, __dom_doc_element($this->__d));
            case 'doctype':
                foreach (__dom_children($this->__d, 0) as $c) {
                    $i = __dom_info($this->__d, $c);
                    if ($i[0] === 10) { return DOMNode::__wrap($this->__d, $c); }
                }
                return null;
            case 'xmlVersion': case 'version':
                $m = __dom_doc_meta($this->__d); return $m[0];
            case 'xmlEncoding': case 'encoding': case 'actualEncoding':
                $m = __dom_doc_meta($this->__d); return $m[1];
            case 'xmlStandalone': case 'standalone': return true;
        }
        return parent::__get($name);
    }
}

class DOMElement extends DOMNode {
    public function __construct($qualifiedName, $value = null, $namespace = '') {
        // A standalone element lives in its own private document until adopted
        // (appendChild across documents raises Wrong Document, as in PHP before
        // importNode).
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 1, (string)$qualifiedName, '');
        if ($this->__n < 0) { throw new DOMException('Invalid Character Error'); }
        if ($value !== null && $value !== '') {
            $t = __dom_create($this->__d, 3, (string)$value, '');
            __dom_mutate($this->__d, 0, $this->__n, $t, -1);
        }
    }
    public function __get($name) {
        if ($name === 'tagName') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[1];
        }
        return parent::__get($name);
    }
    public function getAttribute($qualifiedName) {
        $v = __dom_attr($this->__d, $this->__n, 0, (string)$qualifiedName, '');
        return $v === false ? '' : $v;
    }
    public function hasAttribute($qualifiedName) {
        return __dom_attr($this->__d, $this->__n, 2, (string)$qualifiedName, '');
    }
    public function setAttribute($qualifiedName, $value) {
        __dom_attr($this->__d, $this->__n, 1, (string)$qualifiedName, (string)$value);
        return DOMAttr::__wrapAttr($this->__d, $this->__n, (string)$qualifiedName);
    }
    public function removeAttribute($qualifiedName) {
        return __dom_attr($this->__d, $this->__n, 3, (string)$qualifiedName, '');
    }
    public function getAttributeNames() {
        return __dom_attr($this->__d, $this->__n, 4, '', '');
    }
    public function getAttributeNode($qualifiedName) {
        if (!$this->hasAttribute($qualifiedName)) { return false; }
        return DOMAttr::__wrapAttr($this->__d, $this->__n, (string)$qualifiedName);
    }
    public function setAttributeNode($attr) {
        __dom_attr($this->__d, $this->__n, 1, $attr->name, $attr->value);
        $attr->__d = $this->__d;
        $attr->__e = $this->__n;
        return null;
    }
    public function removeAttributeNode($attr) {
        __dom_attr($this->__d, $this->__n, 3, $attr->name, '');
        return $attr;
    }
    public function toggleAttribute($qualifiedName, $force = null) {
        $has = $this->hasAttribute($qualifiedName);
        $want = $force === null ? !$has : (bool)$force;
        if ($want && !$has) { $this->setAttribute($qualifiedName, ''); }
        if (!$want && $has) { $this->removeAttribute($qualifiedName); }
        return $want;
    }
    public function getElementsByTagName($qualifiedName) {
        $items = array();
        foreach (__dom_by_tag($this->__d, $this->__n, (string)$qualifiedName) as $n) {
            $items[] = DOMNode::__wrap($this->__d, $n);
        }
        return DOMNodeList::__make($items);
    }
    public function setIdAttribute($qualifiedName, $isId) {}
    public function remove() {
        $p = __dom_nav($this->__d, $this->__n, 0);
        if ($p >= 0) { __dom_mutate($this->__d, 2, $p, $this->__n, -1); }
    }
}

class DOMAttr extends DOMNode {
    public $name = '';
    public $value = '';
    public $__e = -1; // owner element node id (-1 = detached)
    public function __construct($name, $value = '') {
        $this->name = (string)$name;
        $this->value = (string)$value;
    }
    public static function __wrapAttr($d, $elem, $name) {
        $a = new DOMAttr($name);
        $a->__d = $d;
        $a->__e = $elem;
        $v = __dom_attr($d, $elem, 0, $name, '');
        $a->value = $v === false ? '' : $v;
        return $a;
    }
    public static function __wrapDetached($d, $name) {
        $a = new DOMAttr($name);
        $a->__d = $d;
        return $a;
    }
    public function __get($prop) {
        switch ($prop) {
            case 'nodeType': return 2;
            case 'nodeName': return $this->name;
            case 'nodeValue': case 'textContent':
                if ($this->__e >= 0) {
                    $v = __dom_attr($this->__d, $this->__e, 0, $this->name, '');
                    if ($v !== false) { return $v; }
                }
                return $this->value;
            case 'ownerElement':
                return $this->__e >= 0 ? DOMNode::__wrap($this->__d, $this->__e) : null;
            case 'specified': return true;
            case 'namespaceURI': case 'prefix': case 'localName':
                if ($this->__e >= 0) {
                    $r = __dom_ns($this->__d, $this->__e, $this->name);
                    if ($prop === 'namespaceURI') { return $r[0]; }
                    if ($prop === 'prefix') { return $r[1]; }
                    return $r[2];
                }
                $p = strpos($this->name, ':');
                if ($prop === 'prefix') { return $p === false ? '' : substr($this->name, 0, $p); }
                if ($prop === 'localName') { return $p === false ? $this->name : substr($this->name, $p + 1); }
                return null;
        }
        return parent::__get($prop);
    }
    public function __set($prop, $v) {
        if ($prop === 'value' || $prop === 'nodeValue') {
            $this->value = (string)$v;
            if ($this->__e >= 0) {
                __dom_attr($this->__d, $this->__e, 1, $this->name, (string)$v);
            }
            return;
        }
        parent::__set($prop, $v);
    }
    public function isId() { return false; }
}

class DOMCharacterData extends DOMNode {
    public function __get($name) {
        if ($name === 'data') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[2];
        }
        if ($name === 'length') {
            $i = __dom_info($this->__d, $this->__n);
            return strlen($i[2]);
        }
        return parent::__get($name);
    }
    public function __set($name, $value) {
        if ($name === 'data') {
            __dom_set_value($this->__d, $this->__n, (string)$value);
            return;
        }
        parent::__set($name, $value);
    }
    public function appendData($data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, $i[2] . (string)$data);
        return true;
    }
    public function substringData($offset, $count) {
        $i = __dom_info($this->__d, $this->__n);
        return substr($i[2], $offset, $count);
    }
    public function insertData($offset, $data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . (string)$data . substr($i[2], $offset));
        return true;
    }
    public function deleteData($offset, $count) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . substr($i[2], $offset + $count));
        return true;
    }
    public function replaceData($offset, $count, $data) {
        $i = __dom_info($this->__d, $this->__n);
        __dom_set_value($this->__d, $this->__n, substr($i[2], 0, $offset) . (string)$data . substr($i[2], $offset + $count));
        return true;
    }
    public function remove() {
        $p = __dom_nav($this->__d, $this->__n, 0);
        if ($p >= 0) { __dom_mutate($this->__d, 2, $p, $this->__n, -1); }
    }
}

class DOMText extends DOMCharacterData {
    public function __construct($data = '') {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 3, (string)$data, '');
    }
    public function isElementContentWhitespace() {
        $i = __dom_info($this->__d, $this->__n);
        return trim($i[2]) === '';
    }
}

class DOMComment extends DOMCharacterData {
    public function __construct($data = '') {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 8, (string)$data, '');
    }
}

class DOMCdataSection extends DOMText {
    public function __construct($data) {
        $this->__d = __dom_new_doc('1.0', '');
        $this->__n = __dom_create($this->__d, 4, (string)$data, '');
    }
}

class DOMProcessingInstruction extends DOMNode {
    public function __get($name) {
        $i = __dom_info($this->__d, $this->__n);
        if ($name === 'target') { return $i[1]; }
        if ($name === 'data') { return $i[2]; }
        return parent::__get($name);
    }
}

class DOMDocumentFragment extends DOMNode {
    public function appendXML($data) {
        // Parse via a throwaway wrapper document, then copy the children in.
        $tmp = new DOMDocument();
        if (!__dom_load($tmp->__d, '<r>' . (string)$data . '</r>', 0)) { return false; }
        $root = __dom_doc_element($tmp->__d);
        foreach (__dom_children($tmp->__d, $root) as $c) {
            $copied = __dom_copy($this->__d, $tmp->__d, $c, 1);
            __dom_mutate($this->__d, 0, $this->__n, $copied, -1);
        }
        return true;
    }
}

class DOMDocumentType extends DOMNode {
    public function __get($name) {
        if ($name === 'name') {
            $i = __dom_info($this->__d, $this->__n);
            return $i[1];
        }
        if ($name === 'publicId' || $name === 'systemId') { return ''; }
        return parent::__get($name);
    }
}

class DOMImplementation {
    public function hasFeature($feature, $version) { return true; }
    public function createDocument($namespace = null, $qualifiedName = '', $doctype = null) {
        $doc = new DOMDocument();
        if ($qualifiedName !== '') {
            $doc->appendChild($doc->createElement($qualifiedName));
        }
        return $doc;
    }
}

class DOMXPath {
    public $document;
    public $__ns = array();
    public function __construct($document, $registerNodeNS = true) {
        $this->document = $document;
    }
    public function registerNamespace($prefix, $namespace) {
        $this->__ns[(string)$prefix] = (string)$namespace;
        return true;
    }
    public function query($expression, $contextNode = null, $registerNodeNS = true) {
        $r = __dom_xpath($this->document->__d, $contextNode === null ? -1 : $contextNode->__n, (string)$expression, $this->__ns);
        if (!is_array($r)) { return false; }
        return DOMNodeList::__make($this->__wrapAll($r));
    }
    public function evaluate($expression, $contextNode = null, $registerNodeNS = true) {
        $r = __dom_xpath($this->document->__d, $contextNode === null ? -1 : $contextNode->__n, (string)$expression, $this->__ns);
        if (is_array($r)) { return DOMNodeList::__make($this->__wrapAll($r)); }
        return $r;
    }
    public function __wrapAll($items) {
        $out = array();
        foreach ($items as $it) {
            if ($it[0] === 'a') {
                $out[] = DOMAttr::__wrapAttr($this->document->__d, $it[1], $it[2]);
            } else {
                $out[] = DOMNode::__wrap($this->document->__d, $it[1]);
            }
        }
        return $out;
    }
    public static function quote($str) {
        $s = (string)$str;
        if (strpos($s, "'") === false) { return "'" . $s . "'"; }
        if (strpos($s, '"') === false) { return '"' . $s . '"'; }
        // Mixed quotes: concat() form, exactly like PHP 8.4's implementation.
        $parts = explode("'", $s);
        $enc = array();
        foreach ($parts as $k => $p) {
            if ($k > 0) { $enc[] = '"\'"'; }
            if ($p !== '') { $enc[] = "'" . $p . "'"; }
        }
        return 'concat(' . implode(',', $enc) . ')';
    }
}
// --- SimpleXML on the __dom_* host hooks -----------------------------------
// One class models PHP's four faces of SimpleXMLElement via $__k:
//   'e' = a concrete element node;
//   's' = a named set ($el->field: children of $__par named $__nm; '' = all,
//         the children() result) -- reads apply to the FIRST match, foreach
//         walks all matches, count() counts them;
//   'a' = a single attribute ($el['name'] / $attrs->name);
//   'A' = the attributes() bag.
class SimpleXMLElement implements ArrayAccess, Countable, Iterator, Stringable {
    public $__d; public $__k = 'e'; public $__n = -1; public $__par = -1; public $__nm = '';
    private $__it = array(); private $__i = 0;
    public function __construct($data = null, $options = 0, $dataIsURL = false, $ns = '', $isPrefix = false) {
        if ($data === null) { return; }
        $doc = new DOMDocument();
        $ok = $dataIsURL ? @$doc->load((string)$data) : @$doc->loadXML((string)$data);
        if (!$ok) { throw new Exception('String could not be parsed as XML'); }
        $this->__d = $doc->__d;
        $this->__n = __dom_doc_element($doc->__d);
    }
    public static function __mk($d, $k, $n, $par = -1, $nm = '') {
        $e = new SimpleXMLElement();
        $e->__d = $d; $e->__k = $k; $e->__n = $n; $e->__par = $par; $e->__nm = (string)$nm;
        return $e;
    }
    public static function __local($n) { $p = strrpos($n, ':'); return $p === false ? $n : substr($n, $p + 1); }
    // The concrete element this face reads from: itself for an element, the
    // FIRST match for a named set, and the PARENT for the children() face
    // ('' set) -- in PHP children() is the same element seen through its
    // child list, so ->x / ['attr'] / getName() on it address the parent.
    public function __node() {
        if ($this->__k === 'e' || $this->__k === 'A') { return $this->__n; }
        if ($this->__k === 's') {
            if ($this->__nm === '') { return $this->__par; }
            $l = $this->__elems();
            return count($l) ? $l[0] : -1;
        }
        return -1;
    }
    // Matching element node ids (set kind; an element is its own singleton).
    public function __elems() {
        $out = array();
        if ($this->__k === 'e') { $out[] = $this->__n; return $out; }
        if ($this->__k !== 's' || $this->__par < 0) { return $out; }
        foreach (__dom_children($this->__d, $this->__par) as $c) {
            $i = __dom_info($this->__d, $c);
            if ($i[0] !== 1) { continue; }
            if ($this->__nm === '' || $i[1] === $this->__nm || SimpleXMLElement::__local($i[1]) === $this->__nm) { $out[] = $c; }
        }
        return $out;
    }
    public function __get($name) {
        $name = (string)$name;
        if ($this->__k === 'A') {
            if (__dom_attr($this->__d, $this->__n, 0, $name, '') === false) { return null; }
            return SimpleXMLElement::__mk($this->__d, 'a', $this->__n, -1, $name);
        }
        $n = $this->__node();
        if ($n < 0) { return null; }
        $s = SimpleXMLElement::__mk($this->__d, 's', -1, $n, $name);
        // No matching child -> null: PHP's empty named set casts to FALSE
        // (`if ($el->{'join-table'})`), and phpr objects are always truthy,
        // so absence must surface as null. Present elements (even empty
        // ones) cast to TRUE, which the always-truthy set object matches.
        if (count($s->__elems()) === 0) { return null; }
        return $s;
    }
    public function __isset($name) {
        $name = (string)$name;
        if ($this->__k === 'A') { return __dom_attr($this->__d, $this->__n, 0, $name, '') !== false; }
        $n = $this->__node();
        if ($n < 0) { return false; }
        $s = SimpleXMLElement::__mk($this->__d, 's', -1, $n, $name);
        return count($s->__elems()) > 0;
    }
    public function offsetExists($k) {
        if (is_int($k)) {
            if ($this->__k === 's') { return $k >= 0 && $k < count($this->__elems()); }
            return $k === 0 && $this->__node() >= 0;
        }
        $n = $this->__node(); if ($n < 0) { return false; }
        return __dom_attr($this->__d, $n, 0, (string)$k, '') !== false;
    }
    public function offsetGet($k) {
        if (is_int($k)) {
            $l = $this->__elems();
            if ($k < 0 || $k >= count($l)) { return null; }
            return SimpleXMLElement::__mk($this->__d, 'e', $l[$k]);
        }
        $n = $this->__node(); if ($n < 0) { return null; }
        // The attribute VALUE as a plain string (PHP hands back an attribute
        // SimpleXMLElement; the string keeps `empty($el['columns'])` faithful
        // for an empty attribute, which phpr's always-truthy objects cannot).
        $v = __dom_attr($this->__d, $n, 0, (string)$k, '');
        return $v === false ? null : (string)$v;
    }
    public function offsetSet($k, $v) {
        $n = $this->__node(); if ($n < 0) { return; }
        __dom_attr($this->__d, $n, 1, (string)$k, (string)$v);
    }
    public function offsetUnset($k) {
        $n = $this->__node(); if ($n < 0) { return; }
        __dom_attr($this->__d, $n, 3, (string)$k, '');
    }
    public function count() {
        if ($this->__k === 'a') { return 0; }
        if ($this->__k === 'A') { return count(__dom_attr($this->__d, $this->__n, 4, '', '')); }
        if ($this->__k === 's') { return count($this->__elems()); }
        $n = $this->__node(); if ($n < 0) { return 0; }
        $c = 0;
        foreach (__dom_children($this->__d, $n) as $ch) {
            $i = __dom_info($this->__d, $ch);
            if ($i[0] === 1) { $c++; }
        }
        return $c;
    }
    public function __toString() {
        if ($this->__k === 'a') {
            $v = __dom_attr($this->__d, $this->__n, 0, $this->__nm, '');
            return $v === false ? '' : (string)$v;
        }
        $n = $this->__node(); if ($n < 0) { return ''; }
        // PHP concatenates the DIRECT text/cdata children only (not descendants).
        $out = '';
        foreach (__dom_children($this->__d, $n) as $c) {
            $i = __dom_info($this->__d, $c);
            if ($i[0] === 3 || $i[0] === 4) { $out .= $i[2]; }
        }
        return $out;
    }
    public function getName() {
        if ($this->__k === 'a') { return SimpleXMLElement::__local($this->__nm); }
        $n = $this->__node(); if ($n < 0) { return ''; }
        $i = __dom_info($this->__d, $n);
        return SimpleXMLElement::__local($i[1]);
    }
    public function children($ns = null, $isPrefix = false) {
        $n = $this->__node();
        return SimpleXMLElement::__mk($this->__d, 's', -1, $n, '');
    }
    public function attributes($ns = null, $isPrefix = false) {
        $n = $this->__node(); if ($n < 0) { return null; }
        return SimpleXMLElement::__mk($this->__d, 'A', $n);
    }
    public function asXML($filename = null) {
        $n = $this->__node(); if ($n < 0) { return false; }
        $xml = __dom_save_xml($this->__d, $n);
        if ($filename !== null) { return file_put_contents((string)$filename, $xml) !== false; }
        return $xml;
    }
    public function saveXML($filename = null) { return $this->asXML($filename); }
    public function rewind() {
        $this->__it = array();
        if ($this->__k === 'A') {
            foreach (__dom_attr($this->__d, $this->__n, 4, '', '') as $an) {
                $this->__it[] = array(SimpleXMLElement::__local($an), SimpleXMLElement::__mk($this->__d, 'a', $this->__n, -1, $an));
            }
        } elseif ($this->__k === 's') {
            foreach ($this->__elems() as $e) {
                $i = __dom_info($this->__d, $e);
                $this->__it[] = array(SimpleXMLElement::__local($i[1]), SimpleXMLElement::__mk($this->__d, 'e', $e));
            }
        } elseif ($this->__k === 'e') {
            foreach (__dom_children($this->__d, $this->__n) as $c) {
                $i = __dom_info($this->__d, $c);
                if ($i[0] !== 1) { continue; }
                $this->__it[] = array(SimpleXMLElement::__local($i[1]), SimpleXMLElement::__mk($this->__d, 'e', $c));
            }
        }
        $this->__i = 0;
    }
    public function valid() { return $this->__i < count($this->__it); }
    public function current() { return $this->__it[$this->__i][1]; }
    public function key() { return $this->__it[$this->__i][0]; }
    public function next() { $this->__i++; }
}
function simplexml_load_string($data, $class_name = 'SimpleXMLElement', $options = 0, $namespace_or_prefix = '', $is_prefix = false) {
    $doc = new DOMDocument();
    if (!@$doc->loadXML((string)$data)) { return false; }
    $r = __dom_doc_element($doc->__d);
    if (!is_int($r) || $r < 0) { return false; }
    return SimpleXMLElement::__mk($doc->__d, 'e', $r);
}
function simplexml_load_file($filename, $class_name = 'SimpleXMLElement', $options = 0, $namespace_or_prefix = '', $is_prefix = false) {
    $c = @file_get_contents((string)$filename);
    if ($c === false) { return false; }
    return simplexml_load_string($c, $class_name, $options, $namespace_or_prefix, $is_prefix);
}
function simplexml_import_dom($node, $class_name = 'SimpleXMLElement') {
    if ($node instanceof DOMDocument) { return SimpleXMLElement::__mk($node->__d, 'e', __dom_doc_element($node->__d)); }
    return SimpleXMLElement::__mk($node->__d, 'e', $node->__n);
}
