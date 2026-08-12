<?php
// v7: two-object cycle, both with dtors. First dtor runs a NESTED gc (which may
// run the peer's dtor early), then writes the peer's declared prop leaf then
// non-leaf. Probes both the double-destruct discipline and the detach window.
class P7 {
    public $p;
    public $peer;
    public $name;
    public function __construct($n) { $this->name = $n; }
    public function __destruct() {
        echo "dtor {$this->name}\n";
        if ($this->name === 'first') {
            gc_collect_cycles();
            $this->peer->p = ['a' => 1];
            $this->peer->p['b'] = 2;
            var_dump($this->peer->p);
        }
    }
}
$x = new P7('first');
$y = new P7('second');
$x->peer = $y; $y->peer = $x;
unset($x, $y);
gc_collect_cycles();
echo "done v7\n";
