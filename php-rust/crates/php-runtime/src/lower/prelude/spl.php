// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
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
interface SeekableIterator extends Iterator {
    public function seek(int $offset);
}
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
// La variante ricorsiva: stesso filtro, con la discesa delegata all'iteratore
// interno (WordPress la usa per lo scan dei template dei block theme — spesso
// solo per la costante GET_MATCH ereditata, passata a un RegexIterator).
class RecursiveRegexIterator extends RegexIterator implements RecursiveIterator {
    public function hasChildren(): bool { return $this->getInnerIterator()->hasChildren(); }
    public function getChildren(): RecursiveRegexIterator {
        return new static($this->getInnerIterator()->getChildren(), $this->getRegex(), $this->getMode(), $this->getFlags(), 0);
    }
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
// SplFileObject / SplTempFileObject: the SPL file-handle layer over the fopen
// family (symfony BinaryFileResponse streams responses through them). Modelled:
// construction, the f* passthroughs, eof/rewind, fstat. Residues: the
// line-iterator protocol (current/next/key/valid, READ_CSV and friends), the
// $useIncludePath/$context constructor args, flock's by-ref $wouldBlock.
class SplFileObject extends SplFileInfo {
    protected $__fh;
    public function __construct($filename, $mode = 'r') {
        parent::__construct($filename);
        $h = @fopen($filename, $mode);
        if ($h === false) {
            throw new RuntimeException("SplFileObject::__construct($filename): Failed to open stream: No such file or directory");
        }
        $this->__fh = $h;
    }
    // A stream URI keeps its full spelling (oracle: getFilename() on
    // "php://temp" is "php://temp", not basename's "temp").
    public function getFilename() { return strpos($this->__path, '://') !== false ? $this->__path : parent::getFilename(); }
    public function fread($length) { return fread($this->__fh, $length); }
    public function fwrite($data, $length = null) { return $length === null ? fwrite($this->__fh, $data) : fwrite($this->__fh, $data, $length); }
    public function fgets() { return fgets($this->__fh); }
    public function fgetc() { return fgetc($this->__fh); }
    public function fseek($offset, $whence = SEEK_SET) { return fseek($this->__fh, $offset, $whence); }
    public function ftell() { return ftell($this->__fh); }
    public function eof() { return feof($this->__fh); }
    public function rewind() { rewind($this->__fh); }
    public function fpassthru() { return fpassthru($this->__fh); }
    public function fstat() { return fstat($this->__fh); }
    public function ftruncate($size) { return ftruncate($this->__fh, $size); }
    public function fflush() { return fflush($this->__fh); }
    public function flock($operation) { return flock($this->__fh, $operation); }
}
class SplTempFileObject extends SplFileObject {
    public function __construct($maxMemory = 2097152) {
        parent::__construct('php://temp', 'w+b');
    }
}
// FilesystemIterator: DirectoryIterator with flag-shaped key()/current() and
// (by default) dot entries skipped. Oracle-pinned on 8.5: SKIP_DOTS is in the
// DEFAULT flags but an explicit flags argument without it DOES surface `.`/`..`
// (Symfony Filesystem::remove spreads `[...new FilesystemIterator($dir, ...)]`).
// Own private state: DirectoryIterator's fields are private (name-mangled).
class FilesystemIterator extends DirectoryIterator {
    const CURRENT_AS_FILEINFO = 0; const CURRENT_AS_PATHNAME = 32; const CURRENT_AS_SELF = 16;
    const KEY_AS_PATHNAME = 0; const KEY_AS_FILENAME = 256;
    const FOLLOW_SYMLINKS = 512; const NEW_CURRENT_AND_KEY = 256;
    const SKIP_DOTS = 4096; const UNIX_PATHS = 8192;
    private $__fsdir;
    private $__fsnames = [];
    private $__fspos = 0;
    private $__fsflags;
    // Default = KEY_AS_PATHNAME | CURRENT_AS_FILEINFO | SKIP_DOTS.
    public function __construct($directory, $flags = 4096) {
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("FilesystemIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        parent::__construct($directory);
        $this->__fsdir = rtrim($directory, '/');
        $this->__fsflags = $flags;
        // Zend iterates in readdir (OS) order, not scandir's sorted default.
        $names = scandir($directory, SCANDIR_SORT_NONE);
        if (($flags & self::SKIP_DOTS) === self::SKIP_DOTS) {
            $keep = [];
            foreach ($names as $n) { if ($n !== '.' && $n !== '..') { $keep[] = $n; } }
            $names = $keep;
        }
        $this->__fsnames = $names;
        $this->__fssync();
    }
    private function __fscur() { return $this->__fsdir . '/' . $this->__fsnames[$this->__fspos]; }
    private function __fssync() {
        if ($this->__fspos < count($this->__fsnames)) { $this->__path = $this->__fscur(); }
    }
    public function rewind(): void { $this->__fspos = 0; $this->__fssync(); }
    public function valid(): bool { return $this->__fspos < count($this->__fsnames); }
    public function next(): void { $this->__fspos++; $this->__fssync(); }
    public function seek($offset): void { $this->__fspos = $offset; $this->__fssync(); }
    public function key(): mixed {
        if (($this->__fsflags & self::KEY_AS_FILENAME) === self::KEY_AS_FILENAME) {
            return $this->__fsnames[$this->__fspos];
        }
        return $this->__fscur();
    }
    public function current(): mixed {
        if (($this->__fsflags & self::CURRENT_AS_PATHNAME) === self::CURRENT_AS_PATHNAME) {
            return $this->__fscur();
        }
        if (($this->__fsflags & self::CURRENT_AS_SELF) === self::CURRENT_AS_SELF) {
            return $this;
        }
        return new SplFileInfo($this->__fscur());
    }
    public function isDot(): bool {
        $n = $this->__fsnames[$this->__fspos] ?? '';
        return $n === '.' || $n === '..';
    }
    public function getFlags() { return $this->__fsflags; }
    public function setFlags($flags) { $this->__fsflags = $flags; }
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
        // Zend iterates in readdir (OS) order, not scandir's sorted default.
        $this->__names = scandir($directory, SCANDIR_SORT_NONE);
        $this->__disync();
    }
    // NB: helper names are per-class unique (__dicur/__disync) — phpr resolves
    // $this->privateMethod() on the RECEIVER's class, so a same-named private
    // in a subclass (RecursiveDirectoryIterator's __sync) would be picked from
    // this ctor's chain and rejected as inaccessible (engine gap: private
    // method shadowing; props already handle this via storage keys).
    private function __dicur() { return $this->__dir . '/' . $this->__names[$this->__pos]; }
    private function __disync() {
        if ($this->__pos < count($this->__names)) { $this->__path = $this->__dicur(); }
    }
    public function rewind(): void { $this->__pos = 0; $this->__disync(); }
    public function valid(): bool { return $this->__pos < count($this->__names); }
    public function next(): void { $this->__pos++; $this->__disync(); }
    public function seek($offset): void { $this->__pos = $offset; $this->__disync(); }
    public function key(): mixed { return $this->__pos; }
    public function current(): mixed { return $this; }
    public function isDot(): bool {
        $n = $this->__names[$this->__pos] ?? '';
        return $n === '.' || $n === '..';
    }
    // Zend: DirectoryIterator stringifies to the current entry's FILENAME
    // (SplFileInfo's inherited one would give the pathname) — wp-cli builds
    // "$dir/$filename" include paths from it.
    public function __toString(): string { return $this->getFilename(); }
}
class RecursiveDirectoryIterator extends FilesystemIterator implements RecursiveIterator {
    private $__dir;
    private $__flags;
    private $__names = [];
    private $__pos = 0;
    private $__sub = ''; // path of this level relative to the traversal root
    public function __construct($directory, $flags = 0) {
        // Validate BEFORE chaining up: the parent (FilesystemIterator) throws
        // its own-branded message, but a bad directory must report
        // "RecursiveDirectoryIterator::__construct(...)".
        if (!is_dir($directory)) {
            throw new UnexpectedValueException("RecursiveDirectoryIterator::__construct($directory): Failed to open directory: No such file or directory");
        }
        parent::__construct($directory, $flags);
        $this->__dir = rtrim($directory, '/');
        $this->__flags = $flags;
        // Zend iterates in readdir (OS) order, not scandir's sorted default.
        $names = scandir($directory, SCANDIR_SORT_NONE);
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
// `SplPriorityQueue`: a max-heap keyed by an arbitrary $priority (Symfony's
// DecoratorServicePass inserts [$priority, --$seq] array priorities). The
// sift order replicates spl_heap.c exactly so equal-priority extraction
// matches PHP byte-for-byte. Iteration is destructive, as in ext/spl.
class SplPriorityQueue implements Iterator, Countable {
    const EXTR_DATA = 1;
    const EXTR_PRIORITY = 2;
    const EXTR_BOTH = 3;
    private $__h = [];  // binary heap of [data, priority]
    private $__flags = self::EXTR_DATA;
    public function compare($priority1, $priority2) {
        return $priority1 <=> $priority2;
    }
    // spl_ptr_heap_insert: sift up while the parent compares strictly smaller.
    public function insert($value, $priority) {
        $node = [$value, $priority];
        $i = count($this->__h);
        $this->__h[] = $node;
        while ($i > 0) {
            $p = ($i - 1) >> 1;
            if ($this->compare($this->__h[$p][1], $priority) < 0) {
                $this->__h[$i] = $this->__h[$p];
                $i = $p;
            } else {
                break;
            }
        }
        $this->__h[$i] = $node;
        return true;
    }
    public function extract() {
        if (!$this->__h) { throw new RuntimeException("Can't extract from an empty heap"); }
        $top = $this->__shape($this->__h[0]);
        $this->__deleteTop();
        return $top;
    }
    public function top() {
        if (!$this->__h) { throw new RuntimeException("Can't peek at an empty heap"); }
        return $this->__shape($this->__h[0]);
    }
    // spl_ptr_heap_delete_top: move the last leaf to the root and sift down,
    // preferring the left child on compare ties.
    private function __deleteTop() {
        $last = array_pop($this->__h);
        $n = count($this->__h);
        if ($n === 0) { return; }
        $i = 0;
        while (true) {
            $j = 2 * $i + 1;
            if ($j >= $n) { break; }
            if ($j + 1 < $n && $this->compare($this->__h[$j + 1][1], $this->__h[$j][1]) > 0) { $j += 1; }
            if ($this->compare($last[1], $this->__h[$j][1]) >= 0) { break; }
            $this->__h[$i] = $this->__h[$j];
            $i = $j;
        }
        $this->__h[$i] = $last;
    }
    private function __shape($node) {
        if ($this->__flags === self::EXTR_BOTH) {
            return ['data' => $node[0], 'priority' => $node[1]];
        }
        return $this->__flags === self::EXTR_PRIORITY ? $node[1] : $node[0];
    }
    public function setExtractFlags($flags) {
        if (($flags & self::EXTR_BOTH) === 0) {
            throw new RuntimeException('Must specify at least one extract flag');
        }
        $this->__flags = $flags & self::EXTR_BOTH;
    }
    public function getExtractFlags() { return $this->__flags; }
    public function count(): int { return count($this->__h); }
    public function isEmpty() { return !$this->__h; }
    public function isCorrupted() { return false; }
    public function recoverFromCorruption() { return true; }
    // Destructive iteration: `next()` drops the current top; `key()` is the
    // remaining count - 1 (counts down to 0); a drained queue reports
    // current() NULL / key() -1 with no error (oracle-pinned).
    public function rewind(): void {}
    public function valid(): bool { return (bool) $this->__h; }
    public function current() { return $this->__h ? $this->__shape($this->__h[0]) : null; }
    public function key() { return count($this->__h) - 1; }
    public function next(): void { if ($this->__h) { $this->__deleteTop(); } }
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
// `RecursiveArrayIterator`: ArrayIterator whose array/object elements are
// recursable children (WpOrg\Requests walks header data through
// RecursiveIteratorIterator(new RecursiveArrayIterator($data))).
class RecursiveArrayIterator extends ArrayIterator implements RecursiveIterator {
    const CHILD_ARRAYS_ONLY = 4;
    public function hasChildren(): bool {
        $cur = $this->current();
        return is_array($cur) || is_object($cur);
    }
    public function getChildren() {
        $cur = $this->current();
        if ($cur instanceof self) { return $cur; }
        return new static($cur);
    }
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
        if ($i === null) { throw new Error("[] operator not supported for SplFixedArray"); }
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
