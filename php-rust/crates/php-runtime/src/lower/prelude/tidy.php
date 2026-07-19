// ============================== ext/tidy ==============================
// The tidy / tidyNode classes and the tidy_* procedural functions, over the
// system libtidy (vm/tidy.rs → php_types::tidyio — the same Homebrew keg the
// oracle links, so output and diagnostics bytes are identical). The native
// document handle lives in the tidy object; every tidyNode holds a reference
// to its owning tidy object (PHPTidyDoc::ref_count via plain refcounting), so
// the handle is freed by tidy::__destruct only when the doc AND all its
// nodes are gone. No extra keeper object: object ids line up with the
// oracle's (tidy #1, first node #2 — test 010/012 dump them).

class tidy {
    public ?string $errorBuffer = null;
    public ?string $value = null;
    private $__h = 0;

    public function __destruct() {
        if ($this->__h !== 0) { __tidy_free($this->__h); }
    }

    // The raw handle, for tidyNode's host calls (the property is private).
    public function __handle() { return $this->__h; }

    // php_tidy_apply_config: a non-array non-null $config stringifies to a
    // config FILE path (tidyLoadConfig — Warning/Notice, never a failure); an
    // array applies per-option (unknown/read-only → ValueError, refused
    // string value → TypeError, silent refusal → overall false). $where names
    // the calling function in the messages.
    private static function __apply_config($h, $config, $where, $argn) {
        if ($config === null) { return true; }
        if (!is_array($config)) {
            $config = (string)$config;
            $r = __tidy_conf_file($h, $config);
            if ($r < 0) {
                __warning_from_caller($where . '(): Could not load the Tidy configuration file "' . $config . '"');
            } elseif ($r > 0) {
                __notice_from_caller($where . '(): There were errors while parsing the Tidy configuration file "' . $config . '"');
            }
            return true;
        }
        $ok = true;
        foreach ($config as $name => $value) {
            if (!is_string($name)) {
                throw new TypeError($where . '(): Argument #' . $argn . ' ($config) must be of type array with keys as string');
            }
            if (is_object($value)) { $value = (string)$value; }
            elseif ($value === null) { $value = ''; }
            elseif (is_array($value)) { $value = 0; }
            elseif (is_float($value)) {
                // zval_get_long: the engine's lossy-cast Warning (NAN/INF)
                // fires here, exactly where php_tidy_set_tidy_opt triggers it.
                $value = (int)$value;
            }
            $r = __tidy_opt_set($h, $name, $value);
            if ($r === 1) {
                throw new ValueError($where . '(): Argument #' . $argn . ' ($config) Unknown Tidy configuration option "' . $name . '"');
            } elseif ($r === 2) {
                throw new ValueError($where . '(): Argument #' . $argn . ' ($config) Attempting to set read-only option "' . $name . '"');
            } elseif ($r === 3) {
                throw new TypeError($where . '(): Argument #' . $argn . ' ($config) option "' . $name . '" does not accept "' . $value . '" as a value');
            } elseif ($r !== 0) {
                $ok = false;
            }
        }
        return $ok;
    }

    // php_tidy_parse_string: optional encoding first (Warning + false on a
    // bad name), then the parse (Warning with the error-buffer text on
    // failure), then tidy_doc_update_properties.
    private function __parse_with($data, $encoding, $where) {
        $h = $this->__h;
        if ($encoding !== null && $encoding !== '') {
            if (!__tidy_set_enc($h, $encoding)) {
                __warning_from_caller($where . '(): Could not set encoding "' . $encoding . '"');
                return false;
            }
        }
        $ok = __tidy_parse($h, $data);
        if (!$ok) {
            $eb = __tidy_errbuf($h);
            __warning_from_caller($where . '(): ' . ($eb === false ? '' : $eb));
            return false;
        }
        $this->__update_props();
        return true;
    }

    // tidy_doc_update_properties: value only when output is non-empty,
    // errorBuffer only when the buffer was written.
    private function __update_props() {
        $out = __tidy_output($this->__h);
        if ($out !== '') { $this->value = $out; }
        $eb = __tidy_errbuf($this->__h);
        if ($eb !== false && $eb !== '') { $this->errorBuffer = $eb; }
    }

    // The internal entry tidy_parse_string()/tidy_parse_file() use: apply
    // config + parse under the FUNCTION's name; false = the caller's false.
    public function __config_and_parse($data, $config, $encoding, $where) {
        if (!self::__apply_config($this->__h, $config, $where, 2)) { return false; }
        return $this->__parse_with($data, $encoding, $where);
    }

    public function __construct(?string $filename = null, $config = null, ?string $encoding = null, bool $useIncludePath = false) {
        $this->__h = __tidy_new();
        if ($filename !== null) {
            if ($filename === '') { throw new ValueError('Path must not be empty'); }
            $contents = @file_get_contents($filename, $useIncludePath);
            if ($contents === false) {
                throw new Exception('Cannot load "' . $filename . '" into memory' . ($useIncludePath ? ' (using include path)' : ''));
            }
            self::__apply_config($this->__h, $config, 'tidy::__construct', 2);
            $this->__parse_with($contents, $encoding, 'tidy::__construct');
        }
    }

    public function parseString(string $string, $config = null, ?string $encoding = null): bool {
        return self::__apply_config($this->__h, $config, 'tidy::parseString', 2)
            && $this->__parse_with($string, $encoding, 'tidy::parseString');
    }

    public function parseFile(string $filename, $config = null, ?string $encoding = null, bool $useIncludePath = false): bool {
        if ($filename === '') { throw new ValueError('Path must not be empty'); }
        $contents = @file_get_contents($filename, $useIncludePath);
        if ($contents === false) {
            __warning_from_caller('tidy::parseFile(): Cannot load "' . $filename . '" into memory' . ($useIncludePath ? ' (using include path)' : ''));
            return false;
        }
        return self::__apply_config($this->__h, $config, 'tidy::parseFile', 2)
            && $this->__parse_with($contents, $encoding, 'tidy::parseFile');
    }

    public static function repairString(string $string, $config = null, ?string $encoding = null) {
        return __tidy_quick_repair($string, $config, $encoding, 'tidy::repairString');
    }

    public static function repairFile(string $filename, $config = null, ?string $encoding = null, bool $useIncludePath = false) {
        if ($filename === '') { throw new ValueError('Path must not be empty'); }
        $contents = @file_get_contents($filename, $useIncludePath);
        if ($contents === false) { return false; }
        return __tidy_quick_repair($contents, $config, $encoding, 'tidy::repairFile');
    }

    public function cleanRepair(): bool {
        if (__tidy_clean_repair($this->__h)) {
            $this->__update_props();
            return true;
        }
        return false;
    }

    public function diagnose(): bool {
        if (__tidy_diagnose($this->__h)) {
            $this->__update_props();
            return true;
        }
        return false;
    }

    public function getOpt(string $option) {
        $v = __tidy_getopt($this->__h, $option);
        if ($v === null) {
            throw new ValueError('tidy::getOpt(): Argument #1 ($option) is an invalid configuration option, "' . $option . '" given');
        }
        return $v;
    }

    public function getOptDoc(string $option) {
        $v = __tidy_opt_doc($this->__h, $option);
        if ($v === null) {
            throw new ValueError('tidy::getOptDoc(): Argument #1 ($option) is an invalid configuration option, "' . $option . '" given');
        }
        return $v;
    }

    public function getConfig(): array { return __tidy_get_config($this->__h); }
    public function getStatus(): int { return __tidy_stat($this->__h, 0); }

    // Host-stat window for the procedural count functions (the handle is
    // private to the class).
    public function __stat($what) { return __tidy_stat($this->__h, $what); }

    public function getHtmlVer(): int {
        $this->__require_init();
        return __tidy_stat($this->__h, 1);
    }
    public function isXhtml(): bool {
        $this->__require_init();
        return __tidy_stat($this->__h, 2);
    }
    public function isXml(): bool {
        $this->__require_init();
        return __tidy_stat($this->__h, 3);
    }

    private function __require_init() {
        if (!__tidy_stat($this->__h, 8)) {
            throw new Error('tidy object is not initialized');
        }
    }

    public function getRelease(): string { return __tidy_release(); }

    public function root(): ?tidyNode { return $this->__node(0); }
    public function html(): ?tidyNode { return $this->__node(1); }
    public function head(): ?tidyNode { return $this->__node(2); }
    public function body(): ?tidyNode { return $this->__node(3); }

    private function __node($which) {
        $p = __tidy_node($this->__h, $which);
        if ($p === null) { return null; }
        return tidyNode::__make($this, $p);
    }

    // tidy_doc_cast_handler(IS_STRING): the current pretty-printed output.
    public function __toString(): string { return __tidy_output($this->__h); }

    // The oracle's tidy object exposes exactly errorBuffer + value.
    public function __debugInfo() {
        return array('errorBuffer' => $this->errorBuffer, 'value' => $this->value);
    }
}

final class tidyNode {
    public readonly string $value;
    public readonly string $name;
    public readonly int $type;
    public readonly int $line;
    public readonly int $column;
    public readonly bool $proprietary;
    public readonly ?int $id;
    public readonly ?array $attribute;
    public readonly ?array $child;
    private $__doc = null;
    private $__p = 0;

    // The manual-construction guard (PHP_METHOD(tidyNode, __construct));
    // __make passes the internal pair. Outside callers hit the visibility
    // error first, exactly like the oracle's private constructor.
    private function __construct($doc = null, $ptr = null) {
        if ($doc === null) {
            throw new Error('You should not create a tidyNode manually');
        }
        $this->__doc = $doc;
        $this->__p = $ptr;
        $i = __tidy_node_info($doc->__handle(), $ptr);
        $this->value = $i['v'];
        $this->name = $i['n'];
        $this->type = $i['t'];
        $this->line = $i['l'];
        $this->column = $i['c'];
        $this->proprietary = $i['pr'];
        $this->id = $i['id'];
        $this->attribute = $i['at'];
        if (count($i['ch']) === 0) {
            $this->child = null;
        } else {
            $kids = array();
            foreach ($i['ch'] as $cp) { $kids[] = tidyNode::__make($doc, $cp); }
            $this->child = $kids;
        }
    }

    // tidy_create_node_object + tidy_add_node_default_properties: the node's
    // scalar props and its child subtree are materialised EAGERLY (as in
    // ext/tidy); navigation (parent/siblings) goes back to the live tree.
    public static function __make($doc, $ptr) {
        return new tidyNode($doc, $ptr);
    }

    public function hasChildren(): bool {
        return __tidy_node_rel($this->__doc->__handle(), $this->__p, 3) !== null;
    }
    public function hasSiblings(): bool {
        return __tidy_node_rel($this->__doc->__handle(), $this->__p, 2) !== null;
    }
    public function isComment(): bool { return $this->type === TIDY_NODETYPE_COMMENT; }
    public function isHtml(): bool {
        return $this->type === TIDY_NODETYPE_START || $this->type === TIDY_NODETYPE_END
            || $this->type === TIDY_NODETYPE_STARTEND;
    }
    public function isText(): bool { return $this->type === TIDY_NODETYPE_TEXT; }
    public function isJste(): bool { return $this->type === TIDY_NODETYPE_JSTE; }
    public function isAsp(): bool { return $this->type === TIDY_NODETYPE_ASP; }
    public function isPhp(): bool { return $this->type === TIDY_NODETYPE_PHP; }

    public function getParent(): ?tidyNode { return $this->__rel(0); }
    public function getPreviousSibling(): ?tidyNode { return $this->__rel(1); }
    public function getNextSibling(): ?tidyNode { return $this->__rel(2); }

    private function __rel($rel) {
        $p = __tidy_node_rel($this->__doc->__handle(), $this->__p, $rel);
        if ($p === null) { return null; }
        return tidyNode::__make($this->__doc, $p);
    }

    // tidy_node_cast_handler(IS_STRING): the node's own text.
    public function __toString(): string { return $this->value; }

    public function __debugInfo() {
        return array(
            'value' => $this->value, 'name' => $this->name, 'type' => $this->type,
            'line' => $this->line, 'column' => $this->column,
            'proprietary' => $this->proprietary, 'id' => $this->id,
            'attribute' => $this->attribute, 'child' => $this->child,
        );
    }
}

function tidy_parse_string(string $string, $config = null, ?string $encoding = null) {
    $t = new tidy();
    if (!$t->__config_and_parse($string, $config, $encoding, 'tidy_parse_string')) { return false; }
    return $t;
}

function tidy_parse_file(string $filename, $config = null, ?string $encoding = null, bool $useIncludePath = false) {
    if ($filename === '') { throw new ValueError('Path must not be empty'); }
    $contents = @file_get_contents($filename, $useIncludePath);
    if ($contents === false) {
        __warning_from_caller('tidy_parse_file(): Cannot load "' . $filename . '" into memory' . ($useIncludePath ? ' (using include path)' : ''));
        return false;
    }
    $t = new tidy();
    if (!$t->__config_and_parse($contents, $config, $encoding, 'tidy_parse_file')) { return false; }
    return $t;
}

// php_tidy_quick_repair: throwaway doc — config, optional encoding, parse,
// clean-and-repair, save. false on any failure (parse failure warns with the
// error-buffer text).
function __tidy_quick_repair($data, $config, $encoding, $where) {
    $t = new tidy();
    if (!$t->__config_and_parse($data, $config, $encoding, $where)) { return false; }
    if (!$t->cleanRepair()) { return false; }
    return (string)$t;
}

function tidy_repair_string(string $string, $config = null, ?string $encoding = null) {
    return __tidy_quick_repair($string, $config, $encoding, 'tidy_repair_string');
}

function tidy_repair_file(string $filename, $config = null, ?string $encoding = null, bool $useIncludePath = false) {
    if ($filename === '') { throw new ValueError('Path must not be empty'); }
    $contents = @file_get_contents($filename, $useIncludePath);
    if ($contents === false) { return false; }
    return __tidy_quick_repair($contents, $config, $encoding, 'tidy_repair_file');
}

// The output-buffer filter ext/tidy registers (ob_start("ob_tidyhandler")):
// a throwaway doc over the whole buffer — parse, clean-and-repair, save
// (php_tidy_output_handler; chunked use is unsupported there too).
function ob_tidyhandler(string $input, int $phase = 0) {
    $h = __tidy_new();
    __tidy_parse($h, $input);
    __tidy_clean_repair($h);
    $out = __tidy_output($h);
    __tidy_free($h);
    return $out;
}

function tidy_get_output(tidy $tidy): string { return (string)$tidy; }

function tidy_get_error_buffer(tidy $tidy) {
    $eb = $tidy->errorBuffer;
    return $eb === null ? false : $eb;
}

function tidy_clean_repair(tidy $tidy): bool { return $tidy->cleanRepair(); }
function tidy_diagnose(tidy $tidy): bool { return $tidy->diagnose(); }
function tidy_get_release(): string { return __tidy_release(); }

function tidy_getopt(tidy $tidy, string $option) {
    try {
        return $tidy->getOpt($option);
    } catch (ValueError $e) {
        throw new ValueError('tidy_getopt(): Argument #2 ($option) is an invalid configuration option, "' . $option . '" given');
    }
}

function tidy_get_opt_doc(tidy $tidy, string $option) {
    try {
        return $tidy->getOptDoc($option);
    } catch (ValueError $e) {
        throw new ValueError('tidy_get_opt_doc(): Argument #2 ($option) is an invalid configuration option, "' . $option . '" given');
    }
}

function tidy_get_config(tidy $tidy): array { return $tidy->getConfig(); }
function tidy_get_status(tidy $tidy): int { return $tidy->getStatus(); }
function tidy_get_html_ver(tidy $tidy): int { return $tidy->getHtmlVer(); }
function tidy_is_xhtml(tidy $tidy): bool { return $tidy->isXhtml(); }
function tidy_is_xml(tidy $tidy): bool { return $tidy->isXml(); }

function tidy_error_count(tidy $tidy): int { return $tidy->__stat(4); }
function tidy_warning_count(tidy $tidy): int { return $tidy->__stat(5); }
function tidy_access_count(tidy $tidy): int { return $tidy->__stat(6); }
function tidy_config_count(tidy $tidy): int { return $tidy->__stat(7); }

function tidy_get_root(tidy $tidy): ?tidyNode { return $tidy->root(); }
function tidy_get_html(tidy $tidy): ?tidyNode { return $tidy->html(); }
function tidy_get_head(tidy $tidy): ?tidyNode { return $tidy->head(); }
function tidy_get_body(tidy $tidy): ?tidyNode { return $tidy->body(); }
