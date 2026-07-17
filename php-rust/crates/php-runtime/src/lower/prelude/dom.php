// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)

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
    public function loadHTML($source, $options = 0) {
        return __dom_load($this->__d, (string)$source, 2);
    }
    public function loadHTMLFile($filename, $options = 0) {
        $this->documentURI = (string)$filename;
        return __dom_load($this->__d, (string)$filename, 3);
    }
    public function saveHTML($node = null) {
        return __dom_save_html($this->__d, $node === null ? -1 : $node->__n);
    }
    public function saveHTMLFile($filename) {
        return file_put_contents($filename, __dom_save_html($this->__d, -1));
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
        // On the DOCUMENT element PHP serializes the whole document (XML
        // declaration + trailing newline); a non-root element is bare.
        $xml = ($n === __dom_doc_element($this->__d))
            ? __dom_save_xml($this->__d, -1)
            : __dom_save_xml($this->__d, $n);
        if ($filename !== null) { return file_put_contents((string)$filename, $xml) !== false; }
        return $xml;
    }
    public function saveXML($filename = null) { return $this->asXML($filename); }
    public function addChild($qualifiedName, $value = null, $namespace = null) {
        $n = $this->__node(); if ($n < 0) { return null; }
        $c = __dom_create($this->__d, 1, (string)$qualifiedName, '');
        if ($c < 0) { throw new Exception('SimpleXMLElement::addChild(): Invalid element name'); }
        __dom_mutate($this->__d, 0, $n, $c, -1);
        if ($value !== null && (string)$value !== '') {
            // PHP treats the value as already-escaped XML text (the documented
            // addChild gotcha): entities decode on the way in, the serializer
            // re-escapes on the way out (WP passes esc_html()ed values).
            $t = __dom_create($this->__d, 3, html_entity_decode((string)$value, ENT_QUOTES), '');
            __dom_mutate($this->__d, 0, $c, $t, -1);
        }
        return SimpleXMLElement::__mk($this->__d, 'e', $c);
    }
    public function addAttribute($qualifiedName, $value = '', $namespace = null) {
        $n = $this->__node(); if ($n < 0) { return; }
        __dom_attr($this->__d, $n, 1, (string)$qualifiedName, html_entity_decode((string)$value, ENT_QUOTES));
    }
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

// ----- ext/xml (expat-style SAX API) -----
// State lives on the XMLParser object; the whole input is buffered until the
// final xml_parse() chunk, tokenized by the __xml_tokenize host builtin, and
// the events dispatched to the registered handlers from here.
final class XMLParser {
    public $__sep = null;
    public $__buf = '';
    public $__fold = true;
    public $__skipwhite = false;
    public $__tagstart = 0;
    public $__target_enc = 'UTF-8';
    public $__obj = null;
    public $__hstart = null;
    public $__hend = null;
    public $__hcdata = null;
    public $__hpi = null;
    public $__hdefault = null;
    public $__err = 0;
    public $__line = 1;
    public $__col = 0;
    public $__byte = 0;
    public $__done = false;
}
function xml_parser_create($encoding = null) {
    return new XMLParser();
}
function xml_parser_create_ns($encoding = null, $separator = ':') {
    $p = new XMLParser();
    $p->__sep = (string)$separator;
    return $p;
}
function xml_parser_free($parser) { return true; }
function xml_parser_set_option($parser, $option, $value) {
    switch ((int)$option) {
        case 1: $parser->__fold = (bool)$value; return true;
        case 2: $parser->__target_enc = (string)$value; return true;
        case 3: $parser->__tagstart = (int)$value; return true;
        case 4: $parser->__skipwhite = (bool)$value; return true;
    }
    return false;
}
function xml_parser_get_option($parser, $option) {
    switch ((int)$option) {
        case 1: return (int)$parser->__fold;
        case 2: return $parser->__target_enc;
        case 3: return $parser->__tagstart;
        case 4: return (int)$parser->__skipwhite;
    }
    return false;
}
function __xml_handler_norm($parser, $h) {
    if ($h === null || $h === '') { return null; }
    if (is_string($h) && $parser->__obj !== null) { return array($parser->__obj, $h); }
    return $h;
}
function xml_set_object($parser, $object) { $parser->__obj = $object; return true; }
function xml_set_element_handler($parser, $start_handler, $end_handler) {
    $parser->__hstart = __xml_handler_norm($parser, $start_handler);
    $parser->__hend = __xml_handler_norm($parser, $end_handler);
    return true;
}
function xml_set_character_data_handler($parser, $handler) {
    $parser->__hcdata = __xml_handler_norm($parser, $handler);
    return true;
}
function xml_set_processing_instruction_handler($parser, $handler) {
    $parser->__hpi = __xml_handler_norm($parser, $handler);
    return true;
}
function xml_set_default_handler($parser, $handler) {
    $parser->__hdefault = __xml_handler_norm($parser, $handler);
    return true;
}
function xml_set_start_namespace_decl_handler($parser, $handler) { return true; }
function xml_set_end_namespace_decl_handler($parser, $handler) { return true; }
function xml_set_notation_decl_handler($parser, $handler) { return true; }
function xml_set_external_entity_ref_handler($parser, $handler) { return true; }
function xml_set_unparsed_entity_decl_handler($parser, $handler) { return true; }
function __xml_fold_name($parser, $name) {
    if ($parser->__tagstart > 0) { $name = (string)substr($name, $parser->__tagstart); }
    return $parser->__fold ? strtoupper($name) : $name;
}
function xml_parse($parser, $data, $is_final = false) {
    $parser->__buf .= (string)$data;
    if (!$is_final) { return 1; }
    $parser->__done = true;
    $events = __xml_tokenize($parser->__buf, $parser->__sep);
    foreach ($events as $e) {
        switch ($e[0]) {
            case 'o':
                if ($parser->__hstart !== null) {
                    $name = __xml_fold_name($parser, $e[1]);
                    $attrs = array();
                    foreach ($e[2] as $k => $v) {
                        $attrs[$parser->__fold ? strtoupper($k) : $k] = $v;
                    }
                    call_user_func($parser->__hstart, $parser, $name, $attrs);
                }
                break;
            case 'c':
                if ($parser->__hend !== null) {
                    call_user_func($parser->__hend, $parser, __xml_fold_name($parser, $e[1]));
                }
                break;
            case 't':
                // XML_OPTION_SKIP_WHITE is a no-op on the libxml compat layer
                // (oracle-probed): whitespace runs are delivered.
                if ($parser->__hcdata !== null) {
                    call_user_func($parser->__hcdata, $parser, $e[1]);
                }
                break;
            case 'p':
                if ($parser->__hpi !== null) {
                    call_user_func($parser->__hpi, $parser, $e[1], $e[2]);
                }
                break;
            case 'x':
                $parser->__err = $e[1];
                $parser->__line = $e[2];
                $parser->__col = $e[3];
                $parser->__byte = $e[4];
                break;
        }
    }
    return $parser->__err === 0 ? 1 : 0;
}
function xml_parse_into_struct($parser, $data, &$values, &$index = null) {
    $values = array();
    $index = array();
    $events = __xml_tokenize((string)$data, $parser->__sep);
    $level = 0;
    $open_idx = array();
    $tag_at = array();
    $ret = 1;
    foreach ($events as $e) {
        switch ($e[0]) {
            case 'o':
                $level++;
                $name = __xml_fold_name($parser, $e[1]);
                $tag_at[$level] = $name;
                $entry = array('tag' => $name, 'type' => 'open', 'level' => $level);
                $has_attrs = false;
                $attrs = array();
                foreach ($e[2] as $k => $v) {
                    $attrs[$parser->__fold ? strtoupper($k) : $k] = $v;
                    $has_attrs = true;
                }
                if ($has_attrs) { $entry['attributes'] = $attrs; }
                $values[] = $entry;
                $i = count($values) - 1;
                $open_idx[$level] = $i;
                $index[$name][] = $i;
                break;
            case 't':
                if ($level < 1) { break; }
                $last = count($values) - 1;
                if ($last === $open_idx[$level]) {
                    if (isset($values[$last]['value'])) { $values[$last]['value'] .= $e[1]; }
                    else { $values[$last]['value'] = $e[1]; }
                } else {
                    $name = $tag_at[$level];
                    $values[] = array('tag' => $name, 'value' => $e[1], 'type' => 'cdata', 'level' => $level);
                    $index[$name][] = count($values) - 1;
                }
                break;
            case 'c':
                $name = __xml_fold_name($parser, $e[1]);
                $last = count($values) - 1;
                if (isset($open_idx[$level]) && $last === $open_idx[$level]) {
                    $values[$last]['type'] = 'complete';
                } else {
                    $values[] = array('tag' => $name, 'type' => 'close', 'level' => $level);
                    $index[$name][] = count($values) - 1;
                }
                unset($open_idx[$level]);
                $level--;
                break;
            case 'x':
                $parser->__err = $e[1];
                $parser->__line = $e[2];
                $parser->__col = $e[3];
                $parser->__byte = $e[4];
                if ($e[1] !== 0) { $ret = 0; }
                break;
        }
    }
    return $ret;
}
function xml_get_error_code($parser) { return $parser->__err; }
function xml_get_current_line_number($parser) { return $parser->__line; }
function xml_get_current_column_number($parser) { return $parser->__col; }
function xml_get_current_byte_index($parser) { return $parser->__byte; }
function xml_error_string($code) {
    // ext/xml/compat.c error_mapping, verbatim (indexed by libxml errNo).
    $strings = array(
        'No error', 'No memory', 'Invalid document start', 'Empty document',
        'Not well-formed (invalid token)', 'Invalid document end',
        'Invalid hexadecimal character reference', 'Invalid decimal character reference',
        'Invalid character reference', 'Invalid character',
        'XML_ERR_CHARREF_AT_EOF', 'XML_ERR_CHARREF_IN_PROLOG', 'XML_ERR_CHARREF_IN_EPILOG',
        'XML_ERR_CHARREF_IN_DTD', 'XML_ERR_ENTITYREF_AT_EOF', 'XML_ERR_ENTITYREF_IN_PROLOG',
        'XML_ERR_ENTITYREF_IN_EPILOG', 'XML_ERR_ENTITYREF_IN_DTD',
        'PEReference at end of document', 'PEReference in prolog', 'PEReference in epilog',
        'PEReference: forbidden within markup decl in internal subset',
        'XML_ERR_ENTITYREF_NO_NAME', "EntityRef: expecting ';'", 'PEReference: no name',
        "PEReference: expecting ';'", 'Undeclared entity error', 'Undeclared entity warning',
        'Unparsed Entity', 'XML_ERR_ENTITY_IS_EXTERNAL', 'XML_ERR_ENTITY_IS_PARAMETER',
        'Unknown encoding', 'Unsupported encoding', "String not started expecting ' or \"",
        "String not closed expecting \" or '", 'Namespace declaration error',
        "EntityValue: \" or ' expected", "EntityValue: \" or ' expected", '< in attribute',
        'Attribute not started', 'Attribute not finished', 'Attribute without value',
        'Attribute redefined', "SystemLiteral \" or ' expected",
        "SystemLiteral \" or ' expected", 'Comment not finished',
        'Processing Instruction not started', 'Processing Instruction not finished',
        'NOTATION: Name expected here', "'>' required to close NOTATION declaration",
        "'(' required to start ATTLIST enumeration",
        "'(' required to start ATTLIST enumeration",
        "MixedContentDecl : '|' or ')*' expected", 'XML_ERR_MIXED_NOT_FINISHED',
        'ELEMENT in DTD not started', 'ELEMENT in DTD not finished',
        'XML declaration not started', 'XML declaration not finished',
        'XML_ERR_CONDSEC_NOT_STARTED', 'XML conditional section not closed',
        'Content error in the external subset', 'DOCTYPE not finished',
        "Sequence ']]>' not allowed in content", 'CDATA not finished', 'Reserved XML Name',
        'Space required', 'XML_ERR_SEPARATOR_REQUIRED',
        'NmToken expected in ATTLIST enumeration', 'XML_ERR_NAME_REQUIRED',
        "MixedContentDecl : '#PCDATA' expected", 'SYSTEM or PUBLIC, the URI is missing',
        'PUBLIC, the Public Identifier is missing', '< required', '> required',
        '</ required', '= required', 'Mismatched tag', 'Tag not finished',
        "standalone accepts only 'yes' or 'no'", 'Invalid XML encoding name',
        "Comment must not contain '--' (double-hyphen)", 'Invalid encoding',
        'external parsed entities cannot be standalone',
        "XML conditional section '[' expected", 'Entity value required',
        'chunk is not well balanced', 'extra content at the end of well balanced chunk',
        'XML_ERR_ENTITY_CHAR_ERROR', 'PEReferences forbidden in internal subset',
        'Detected an entity reference loop', 'XML_ERR_ENTITY_BOUNDARY', 'Invalid URI',
        'Fragment not allowed', 'XML_WAR_CATALOG_PI', 'XML_ERR_NO_DTD',
        'conditional section INCLUDE or IGNORE keyword expected',
        'Version in XML Declaration missing', 'XML_WAR_UNKNOWN_VERSION',
        'XML_WAR_LANG_VALUE', 'XML_WAR_NS_URI', 'XML_WAR_NS_URI_RELATIVE',
        'Missing encoding in text declaration',
    );
    $code = (int)$code;
    return isset($strings[$code]) ? $strings[$code] : 'Unknown';
}
