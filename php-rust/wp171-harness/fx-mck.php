<?php
// fx-mck.php — S-166 L-MCk (criterio p.5): il cammino k=3 diventa fast; i
// NON ammessi (by-ref, hint) restano sul funnel. Arbitro: A(pin s165, k=3 sul
// funnel) == B(MCk, k=3 fast) BYTE-identici; oracle = fedeltà.
error_reporting(E_ALL);
class T {
    public $v = 0;
    function tre($a, $b, $c) { return $a + $b + $c; }          // ammessa k=3
    function cinque($a, $b, $c, $d, $e) { return "$a$b$c$d$e"; } // ammessa k=5
    function byref3(&$x, $y, $z) { $x += $y + $z; return $x; }  // NON simple_call
    function hint3(int $a, $b, $c) { return $a . $b . $c; }     // hint ⇒ funnel
    function &ret3($a, $b) { return $this->v; }
}
class G3 { public function __get($n) { echo "get:$n "; return 4; } }
$t = new T;
$s = 0;
for ($i = 0; $i < 3; $i++) { $s = $t->tre($s, 1, 2); }  // IC caldo
echo $s, "\n";
echo $t->cinque(1, 2, 3, 4, 5), "\n";
$x = 10; echo $t->byref3($x, 1, 2), " ", $x, "\n";
echo $t->hint3("7", "b", "c"), "\n";
// ArgPlace a k=3 (variabile + __get + letterale) e Err a metà (Append)
$g = new G3; $arr = ['k' => 9];
echo $t->tre($s, $g->p, $arr['k']), "\n";
try { $t->tre($arr[], 1, 2); } catch (\Error $e) { echo "E:", $e->getMessage(), "\n"; }
echo "done\n";
