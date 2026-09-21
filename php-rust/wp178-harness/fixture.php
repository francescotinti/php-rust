<?php
namespace S178;
class MutablePrivate {
    private int $x;
    function set(int $x): void { $this->x = $x; }
    function get(): int { return $this->x; }
}
readonly class ReadOnlyPrivate {
    function __construct(private int $x) {}
    function get(): int { return $this->x; }
}
class PublicMutable { public int $x; }
class BasePrivate {
    private int $x;
    function setBase(int $x): void { $this->x = $x; }
    function getBase(): int { return $this->x; }
}
class ChildPrivate extends BasePrivate {
    private int $x;
    function setChild(int $x): void { $this->x = $x; }
    function getChild(): int { return $this->x; }
}
function publicSet($o, $i): void { $o->x = $i; }
$m = new MutablePrivate;
for ($i=0;$i<10;$i++) $m->set($i);
$total=0;
for ($i=0;$i<10;$i++) { $r=new ReadOnlyPrivate($i); $total += $r->get(); }
$p = new PublicMutable;
for ($i=0;$i<10;$i++) publicSet($p,$i);
$c = new ChildPrivate;
$c->setBase(3); $c->setChild(7);
echo $m->get(),':',$total,':',$p->x,':',$c->getBase(),':',$c->getChild(),"\n";
