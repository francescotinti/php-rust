<?php
// v2: two-object cycle; B's dtor writes A's declared prop leaf then non-leaf (A has no dtor).
class A2 { public $p; public $peer; }
class B2 {
    public $peer;
    public function __destruct() {
        echo "dtor B2\n";
        $this->peer->p = ['a' => 1];
        $this->peer->p['b'] = 2;
        var_dump($this->peer->p);
    }
}
$a = new A2; $b = new B2;
$a->peer = $b; $b->peer = $a;
unset($a, $b);
gc_collect_cycles();
echo "done v2\n";
