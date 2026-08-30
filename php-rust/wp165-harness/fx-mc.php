<?php
// fx-mc.php — fixture semantica L-MC1 (B vs oracle vs A): confini del fast path
error_reporting(E_ALL);
class P {
    public $v = 0;
    function add($a,$b){ return $a+$b; }                 // ammessa (k=2)
    function one($a){ return $a*2; }                     // ammessa (k=1)
    function zero(){ return 7; }                         // ammessa (k=0)
    function byref(&$x,$y){ $x += $y; return $x; }       // NON simple_call
    function &retref($k,$_){ return $this->v; }          // by-ref return, value ctx
    function hinted(int $a,$b){ return $a.$b; }          // hint ⇒ NON simple_call
    private function sec($a,$b){ return "P::sec"; }
    function callsec($a,$b){ return $this->sec($a,$b); } // private via $this
}
class Q extends P {
    function add($a,$b){ return ($a+$b)*10; }            // override: IC per-cid
}
class M { function __call($n,$a){ return "call:$n:".count($a); } }
$o = new P; $q = new Q; $m = new M;
$s = 0;
for ($i = 0; $i < 3; $i++) { $s = $o->add($s, 2); }      // riempi IC, poi hit
echo $s, "\n";
echo $o->one(21), " ", $o->zero(), "\n";
$x = 5; echo $o->byref($x, 3), " ", $x, "\n";
$o->v = 9; $r = $o->retref(1, 2); echo $r, "\n";
echo $o->hinted("42", "z"), "\n";
echo $o->callsec(1, 2), "\n";
echo $m->foo(1, 2), "\n";
// stesso sito, classi alternate (IC re-fill / miss)
foreach ([$o, $q, $o, $q] as $t) { echo $t->add(1, 2), " "; }
echo "\n";
// ricevitore Ref
$ro = &$o; echo $ro->add(10, 20), "\n";
// ArgPlace con passi: elemento array e proprietà
$arr = ['k' => 4]; $po = new P; $po->v = 6;
echo $o->add($arr['k'], $po->v), "\n";
// ArgPlace su variabile INDEFINITA: warning alla riga della CALL
echo $o->add(@$undef1, 1), "\n";
echo $o->add($undef2, 1), "\n";
// func_get_args dentro un callee simple_call ad arità esatta
class G { function g($a,$b){ return implode(",", func_get_args()); } }
$gg = new G; echo $gg->g(8, 9), "\n";
// argomento Ref esplicito decade a valore
$vv = 3; $rv = &$vv; echo $o->add($rv, 1), "\n";
echo "done\n";
