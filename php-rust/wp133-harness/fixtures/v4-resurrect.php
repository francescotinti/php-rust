<?php
// v4: dtor resurrects $this into a global; second GC destroys it without
// re-running the dtor; then normal code writes the declared prop leaf then
// non-leaf through the surviving handle.
class C4 {
    public $p;
    public $self;
    public function __destruct() {
        echo "dtor C4\n";
        $GLOBALS['keep'] = $this;
    }
}
$c = new C4;
$c->self = $c;
unset($c);
gc_collect_cycles();
echo "after gc1\n";
$k = $GLOBALS['keep'];
$k->self = $k;
unset($GLOBALS['keep'], $k);
gc_collect_cycles();
echo "after gc2\n";
echo isset($GLOBALS['keep']) ? "kept\n" : "gone\n";
echo "done v4\n";
