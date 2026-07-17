<?php
// ext/fileinfo: the finfo class and the procedural API, delegating to the
// __finfo_detect host builtin (php-builtins/src/fileinfo.rs — a libmagic
// 5.46 work-alike pinned on the WP test-suite corpus).
// finfo is engine-opaque (vm: is_opaque_handle_class): var_dump shows no
// props, clone/serialize throw, the hidden $__flags stays invisible.
// I/O deliberately happens PHP-side (file_get_contents), so userland stream
// wrappers and open_basedir behave exactly as in ext/fileinfo's php_stream
// path. Reads are capped at FILE_BYTES_MAX (7MB) like libmagic's window.

final class finfo
{
    public $__flags = 0;

    public function __construct(int $flags = FILEINFO_NONE, ?string $magic_database = null)
    {
        if (func_num_args() > 2) {
            throw new ArgumentCountError('finfo::__construct() expects at most 2 arguments, ' . func_num_args() . ' given');
        }
        $this->__flags = $flags;
    }

    public function file(string $filename, ?int $flags = null, $context = null)
    {
        return __finfo_path('finfo::file', 1, $filename, $flags === null ? $this->__flags : $flags);
    }

    public function buffer(string $string, ?int $flags = null, $context = null)
    {
        if ($context !== null) {
            __deprecated_from_caller('finfo::buffer(): The $context parameter has no effect for finfo_buffer()');
        }
        return __finfo_detect($string, $flags === null ? $this->__flags : $flags);
    }

    public function set_flags(int $flags): bool
    {
        $this->__flags = $flags;
        return true;
    }
}

// Shared path flow: directories short-circuit (libmagic's fsmagic), all other
// paths go through the stream layer; the Warning carries the caller's name
// and the caller's argument position, like ext/fileinfo's docref output.
function __finfo_path(string $fn, int $argpos, string $filename, int $flags)
{
    if ($filename === '') {
        throw new ValueError($fn . '(): Argument #' . $argpos . ' ($filename) must not be empty');
    }
    if (@is_dir($filename)) {
        return 'directory';
    }
    $d = @file_get_contents($filename, false, null, 0, 7340032);
    if ($d === false) {
        $e = error_get_last();
        $reason = 'No such file or directory';
        if ($e !== null && ($p = strpos($e['message'], 'Failed to open stream: ')) !== false) {
            $reason = substr($e['message'], $p + strlen('Failed to open stream: '));
        }
        __warning_from_caller($fn . '(' . $filename . '): Failed to open stream: ' . $reason);
        return false;
    }
    return __finfo_detect($d, $flags);
}

function finfo_open(int $flags = FILEINFO_NONE, ?string $magic_database = null)
{
    return new finfo($flags, $magic_database);
}

function finfo_close(finfo $finfo): bool
{
    __deprecated_from_caller('Function finfo_close() is deprecated since 8.5, as finfo objects are freed automatically');
    return true;
}

function finfo_file(finfo $finfo, string $filename, ?int $flags = null, $context = null)
{
    return __finfo_path('finfo_file', 2, $filename, $flags === null ? $finfo->__flags : $flags);
}

function finfo_buffer(finfo $finfo, string $string, ?int $flags = null, $context = null)
{
    if ($context !== null) {
        __deprecated_from_caller('finfo_buffer(): The $context parameter has no effect for finfo_buffer()');
    }
    return __finfo_detect($string, $flags === null ? $finfo->__flags : $flags);
}

function finfo_set_flags(finfo $finfo, int $flags): bool
{
    return $finfo->set_flags($flags);
}

function mime_content_type($filename)
{
    if (is_resource($filename)) {
        $d = @stream_get_contents($filename, 7340032, 0);
        if ($d === false) {
            return false;
        }
        return __finfo_detect($d, FILEINFO_MIME_TYPE);
    }
    return __finfo_path('mime_content_type', 1, (string)$filename, FILEINFO_MIME_TYPE);
}
