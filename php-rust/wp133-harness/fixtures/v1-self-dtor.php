<?php
// v1: cycle, dtor writes OWN declared prop leaf then non-leaf.
class C1 {
    public $p;
    public $self;
    public function __destruct() {
        echo "dtor C1\n";
        $this->p = ['a' => 1];
        $this->p['b'] = 2;
        var_dump($this->p);
    }
}
$c = new C1;
$c->self = $c;
unset($c);
gc_collect_cycles();
echo "done v1\n";
