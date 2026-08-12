<?php
// v3: dtor triggers a NESTED gc_collect_cycles(), then writes own declared
// prop leaf then non-leaf. If the nested call detaches $this's props
// (empty-layout Props::new()), the non-leaf write hits the window.
class C3 {
    public $p;
    public $self;
    public function __destruct() {
        echo "dtor C3\n";
        gc_collect_cycles();
        $this->p = ['a' => 1];
        $this->p['b'] = 2;
        var_dump($this->p);
    }
}
$c = new C3;
$c->self = $c;
unset($c);
gc_collect_cycles();
echo "done v3\n";
