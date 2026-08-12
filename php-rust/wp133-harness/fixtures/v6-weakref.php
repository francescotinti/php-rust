<?php
// v6: WeakReference to a cycle object (no dtor). After gc, get() must be null;
// if an engine hands back a detached shell, the leaf+non-leaf write probes it.
class A6 { public $p; public $self; }
$a = new A6;
$a->self = $a;
$w = WeakReference::create($a);
unset($a);
gc_collect_cycles();
$o = $w->get();
if ($o === null) {
    echo "null\n";
} else {
    echo "ALIVE\n";
    $o->p = ['a' => 1];
    $o->p['b'] = 2;
    var_dump($o->p);
}
echo "done v6\n";
