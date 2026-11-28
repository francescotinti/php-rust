<?php
// S-195 behaviour tooth: the value of a plain-variable argument is read at the SEND, not at the bind. Expected = the oracle's output.
namespace S195;
class K {
    function val($a, $b = 0) { return "$a,$b"; }
    function mix($a, &$b) { $r = "$a,$b"; $b = 'w'; return $r; }
    function ref(&$a, $b) { $r = "$a,$b"; $a = 'w'; return $r; }
    static function st($a, $b) { return "$a,$b"; }
    function __call($n, $a) { return "__call:" . implode(',', $a); }
    function __invoke($a, $b) { return "inv:$a,$b"; }
    function self2($a, $b) { return $this->val($a, $b); }
    function own() { $i = 1; return $this->val($i, $i = 5); }
    function typed(int $a, int $b): string { return "$a,$b"; }
    function arr($a, $b) { return json_encode($a) . json_encode($b); }
}
class P { public $v; function __construct($a, $b) { $this->v = "$a,$b"; } }
function inc(&$x) { return ++$x; }
function byv($a, $b) { return "$a,$b"; }
function run() {
    $o = new K;
    $i = 1; echo $o->val($i, $i = 5), " $i\n";
    $i = 1; echo $o->val($i, $i++), " $i\n";
    $i = 1; echo $o->val($i, ++$i), " $i\n";
    $i = 1; echo $o->val($i, $i += 3), " $i\n";
    $i = 1; echo $o->val($i, inc($i)), " $i\n";
    $i = 1; echo $o->val($i, $i), " $i\n";
    $i = 1; echo $o->val($i = 2, $i), " $i\n";
    $i = 1; echo $o->val($i, $i = 5, ), $o->val($i, 7), "\n";
    $i = 1; echo $o->mix($i, $i), " $i\n";
    $i = 1; $j = 2; echo $o->mix($i, $j), " $i $j\n";
    $i = 1; echo $o->ref($i, $i = 5), " $i\n";
    $i = 1; echo $o->ref($i, inc($i)), " $i\n";
    $i = 1; echo K::st($i, $i = 5), " $i\n";
    $c = 'S195\\K'; $i = 1; echo $c::st($i, $i = 5), " $i\n";
    $m = 'val'; $i = 1; echo $o->$m($i, $i = 5), " $i\n";
    $i = 1; echo $o->nope($i, $i = 5), " $i\n"; // __call: by value
    $i = 1; echo $o($i, $i = 5), " $i\n";
    $f = fn($a, $b) => "$a,$b"; $i = 1; echo $f($i, $i = 5), " $i\n";
    $g = function ($a, &$b) { $r = "$a,$b"; $b = 9; return $r; }; $i = 1; $k = 2; echo $g($i, $k), " $i $k\n";
    $i = 1; $p = new P($i, $i = 5); echo $p->v, " $i\n";
    $cn = 'S195\\P'; $i = 1; $p = new $cn($i, $i = 5); echo $p->v, " $i\n";
    $i = 1; echo byv($i, $i = 5), " $i\n";
    $i = 1; echo \S195\byv($i, $i = 5), " $i\n";
    $i = 1; echo $o->self2($i, $i = 5), " ", $o->own(), "\n";
    $i = 1; echo $o->typed($i, $i = 5), " $i\n";
    $a = [1]; echo $o->arr($a, $a[] = 2), " ", json_encode($a), "\n";
    $a = [1]; echo $o->arr($a, $a = [9]), " ", json_encode($a), "\n";
    $s = 'x'; echo $o->val($s, $s .= 'y'), " $s\n";
    $r = 1; $rr = &$r; echo $o->val($r, $r = 5), " $r\n";
    echo $o->val($u, $u = 5), " $u\n";
    $i = 1; echo $o->val($i, $o->val($i, $i = 5)), " $i\n";
    $i = 1; echo $o->val($i, [$i, $i = 4][1]), " $i\n";
    $i = 1; echo $o->val($i, (function () use (&$i) { return $i = 6; })()), " $i\n";
    $i = 1; echo call_user_func([$o, 'val'], $i, $i = 5), " $i\n";
    $i = 1; $j = 1; for ($n = 0; $n < 3; $n++) { echo $o->val($i, $j), $o->val($j, $j = $j + 1), ";"; } echo " $i $j\n";
    $i = 1; $b = $i; echo $o->val($i, 2), " "; $b++; echo "$i $b\n";
}
run();
$gi = 1; $go = new K; echo $go->val($gi, $gi = 5), " $gi\n";
