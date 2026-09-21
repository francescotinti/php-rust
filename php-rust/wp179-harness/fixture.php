<?php
namespace S179;
class MutablePrivate {
 private $x = 0;
 function set($v) { $this->x = $v; }
 function get() { return $this->x; }
}
class ChildPrivate extends MutablePrivate { private $x = 70; }
class ReadonlyPrivate { function __construct(private readonly int $x) {} }
class MagicPrivate {
 private $x = 0;
 function set($v) { $this->x = $v; }
 function __set($k,$v) {}
}
class PublicMutable { public $x = 0; }
class RefPrivate {
 private int $x = 0;
 function &ref() { return $this->x; }
 function set($v) { $this->x = $v; }
}
class RefPublic { public int $y = 0; }
class TypedPrivate {
 private int $x = 0;
 function set($v) { $this->x = $v; }
 function get() { return $this->x; }
}
class UnsetPrivate {
 private $x = 0;
 function run() { unset($this->x); $this->x = 9; $this->x = 10; }
}
class HookPrivate {
 private int $x = 0 { set { $this->x = $value; } }
 function set($v) { $this->x = $v; }
 function get() { return $this->x; }
}
$p = new PublicMutable; $p->x = 1; $p->x = 2;
$rp = new RefPrivate; $rq = new RefPublic; $rq->y =& $rp->ref(); $rp->set(3);
$a = new MutablePrivate; $a->set(1); $a->set(2);
$c = new ChildPrivate; $c->set(3); $c->set(4);
$r = new ReadonlyPrivate(5);
$m = new MagicPrivate; $m->set(6); $m->set(7);
$t = new TypedPrivate; $t->set(8); $t->set(9);
$u = new UnsetPrivate; $u->run();
$h = new HookPrivate; $h->set(11);
$b = \Closure::bind(function ($o) { $o->x = 12; }, null, MutablePrivate::class); $b($a);
echo $a->get(), ':', $c->get(), ':', $t->get(), ':', $h->get(), "\n";
