<?php
// v5: cycle left alive at script end — dtor runs during teardown dtor-walk;
// writes own declared prop leaf then non-leaf.
class C5 {
    public $p;
    public $self;
    public function __destruct() {
        echo "dtor C5\n";
        $this->p = ['a' => 1];
        $this->p['b'] = 2;
        var_dump($this->p);
    }
}
$c = new C5;
$c->self = $c;
unset($c);
echo "end of script v5\n";
