<?php
namespace Pdo;
class Sqlite extends \PDO {
    public const DETERMINISTIC = 2048;
    public const OPEN_READONLY = 1;
    public const OPEN_READWRITE = 2;
    public const OPEN_CREATE = 4;
    public const ATTR_OPEN_FLAGS = 1000;
    public const ATTR_READONLY_STATEMENT = 1001;
    public const ATTR_EXTENDED_RESULT_CODES = 1002;
    public const ATTR_BUSY_STATEMENT = 1003;
    public const ATTR_EXPLAIN_STATEMENT = 1004;
    public const ATTR_TRANSACTION_MODE = 1005;
    public const TRANSACTION_MODE_DEFERRED = 0;
    public const TRANSACTION_MODE_IMMEDIATE = 1;
    public const TRANSACTION_MODE_EXCLUSIVE = 2;
    public const EXPLAIN_MODE_PREPARED = 0;
    public const EXPLAIN_MODE_EXPLAIN = 1;
    public const EXPLAIN_MODE_EXPLAIN_QUERY_PLAN = 2;
    public const OK = 0;
    public const DENY = 1;
    public const IGNORE = 2;
}

namespace Dom;

// ext/dom "new DOM" API subset (PHP 8.4+): the surface symfony/dom-crawler's
// HTML5 parse path consumes. Handles carry (docId, nodeId) into the same
// arena as the legacy DOM* classes; HTML parsing is host-side
// (__dom_load_html, an HTML5-lite tree builder). The companion constant
// `Dom\HTML_NO_DEFAULT_NS` is seeded host-side (vm/mod.rs) — prelude
// top-level statements never execute.

class Node {
    public $__d = -1;
    public $__n = -1;
    public static function __wrapNew($d, $n) {
        if ($d < 0 || $n < 0) { return null; }
        $i = \__dom_info($d, $n);
        switch ($i[0]) {
            case 1: $c = 'Dom\\Element'; break;
            case 3: $c = 'Dom\\Text'; break;
            case 4: $c = 'Dom\\CDATASection'; break;
            case 7: $c = 'Dom\\ProcessingInstruction'; break;
            case 8: $c = 'Dom\\Comment'; break;
            case 9: $c = 'Dom\\HTMLDocument'; break;
            case 10: $c = 'Dom\\DocumentType'; break;
            default: $c = 'Dom\\Node';
        }
        $r = new \ReflectionClass($c);
        $o = $r->newInstanceWithoutConstructor();
        $o->__d = $d;
        $o->__n = $n;
        return $o;
    }
    public function __get($name) {
        switch ($name) {
            case 'nodeType': return \__dom_info($this->__d, $this->__n)[0];
            case 'nodeName': return \__dom_info($this->__d, $this->__n)[1];
            case 'nodeValue':
                $i = \__dom_info($this->__d, $this->__n);
                if ($i[0] === 1 || $i[0] === 11) { return \__dom_text($this->__d, $this->__n); }
                return $i[2];
            case 'textContent': return \__dom_text($this->__d, $this->__n);
            case 'parentNode': return Node::__wrapNew($this->__d, \__dom_nav($this->__d, $this->__n, 0));
            case 'firstChild': return Node::__wrapNew($this->__d, \__dom_nav($this->__d, $this->__n, 1));
            case 'lastChild': return Node::__wrapNew($this->__d, \__dom_nav($this->__d, $this->__n, 2));
            case 'nextSibling': return Node::__wrapNew($this->__d, \__dom_nav($this->__d, $this->__n, 3));
            case 'previousSibling': return Node::__wrapNew($this->__d, \__dom_nav($this->__d, $this->__n, 4));
            case 'ownerDocument':
                $i = \__dom_info($this->__d, $this->__n);
                return $i[0] === 9 ? null : Node::__wrapNew($this->__d, 0);
            case 'childNodes':
                $items = array();
                foreach (\__dom_children($this->__d, $this->__n) as $c) {
                    $items[] = Node::__wrapNew($this->__d, $c);
                }
                return NodeList::__make($items);
            // Subclass magic props share this one __get (data/tagName/… read
            // as null on node kinds that lack them, a documented shortcut).
            case 'data': return \__dom_info($this->__d, $this->__n)[2];
            case 'target': return \__dom_info($this->__d, $this->__n)[1];
            case 'tagName': return \__dom_info($this->__d, $this->__n)[1];
            case 'attributes':
                if (\__dom_info($this->__d, $this->__n)[0] !== 1) { return null; }
                $items = array();
                foreach (\__dom_attr($this->__d, $this->__n, 4, '', '') as $an) {
                    $items[$an] = Attr::__wrapAttr($this->__d, $this->__n, $an);
                }
                return NamedNodeMap::__make($items);
            case 'documentElement': return Node::__wrapNew($this->__d, \__dom_doc_element($this->__d));
            case 'inputEncoding':
                $m = \__dom_doc_meta($this->__d);
                return isset($m[2]) ? $m[2] : 'UTF-8';
        }
        return null;
    }
    public function hasChildNodes() { return \__dom_nav($this->__d, $this->__n, 1) >= 0; }
}

class CharacterData extends Node {}
class Text extends CharacterData {}
class CDATASection extends Text {}
class Comment extends CharacterData {}
class ProcessingInstruction extends CharacterData {}
class DocumentType extends Node {}
class Element extends Node {
    public function getAttribute($qualifiedName) {
        $v = \__dom_attr($this->__d, $this->__n, 0, (string)$qualifiedName, '');
        return $v === false ? null : $v;
    }
    public function hasAttribute($qualifiedName) {
        return \__dom_attr($this->__d, $this->__n, 2, (string)$qualifiedName, '');
    }
    public function getElementsByTagName($qualifiedName) {
        $items = array();
        foreach (\__dom_by_tag($this->__d, $this->__n, (string)$qualifiedName) as $n) {
            $items[] = Node::__wrapNew($this->__d, $n);
        }
        return NodeList::__make($items);
    }
}
class Attr extends Node {
    public $__e = -1; // owner element node id
    public $__a = '';
    public static function __wrapAttr($d, $e, $name) {
        $a = new Attr();
        $a->__d = $d;
        $a->__e = $e;
        $a->__a = (string)$name;
        return $a;
    }
    public function __get($name) {
        switch ($name) {
            case 'name': case 'nodeName': case 'localName': return $this->__a;
            case 'value': case 'nodeValue': case 'textContent':
                $v = \__dom_attr($this->__d, $this->__e, 0, $this->__a, '');
                return $v === false ? '' : $v;
            case 'nodeType': return 2;
            case 'specified': return true;
            case 'ownerElement': return Node::__wrapNew($this->__d, $this->__e);
        }
        return null;
    }
}
class Document extends Node {}
class HTMLDocument extends Document {
    public static function createFromString(string $source, int $options = 0, ?string $overrideEncoding = null): HTMLDocument {
        $d = \__dom_load_html($source, $overrideEncoding);
        $r = new \ReflectionClass('Dom\\HTMLDocument');
        $o = $r->newInstanceWithoutConstructor();
        $o->__d = $d;
        $o->__n = 0;
        return $o;
    }
}
class NodeList implements \IteratorAggregate, \Countable {
    public $length = 0;
    public $__items = array();
    public static function __make($items) {
        $l = new NodeList();
        $l->__items = $items;
        $l->length = count($items);
        return $l;
    }
    public function item($index) { return isset($this->__items[$index]) ? $this->__items[$index] : null; }
    public function count(): int { return $this->length; }
    public function getIterator(): \Iterator { return new \ArrayIterator($this->__items); }
}
class NamedNodeMap implements \IteratorAggregate, \Countable {
    public $length = 0;
    public $__items = array(); // name => Attr
    public static function __make($items) {
        $m = new NamedNodeMap();
        $m->__items = $items;
        $m->length = count($items);
        return $m;
    }
    public function getNamedItem($name) { return isset($this->__items[$name]) ? $this->__items[$name] : null; }
    public function item($index) {
        $i = 0;
        foreach ($this->__items as $v) { if ($i === (int)$index) { return $v; } $i++; }
        return null;
    }
    public function count(): int { return $this->length; }
    public function getIterator(): \Iterator { return new \ArrayIterator($this->__items); }
}
