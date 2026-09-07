<?php
// fx-sl2-div.php — forme del cammino prop con DIVERGENZE PRE-esistenti dall'oracle
// (S-172, criterio p.6): gate INVARIANTE pin==stash s171 (non bilaterale). La leva L-SL2
// non le tocca: il probe sigillato scrive solo Long in slot già Long, la coercizione del
// default di classe è a monte (§3.30, PHPR_DIVERGENCES_FROM_PHP.md).
function show($label, $v) { echo $label, ': '; var_dump($v); }
class T2 { public float $f = 1; public int $y = 3; }
// §3.30: default di proprietà `float` scritto come literal int NON coerce a float
// (oracle: float(1); phpr: int(1)). La scrittura runtime coerce correttamente (fx-sl2
// p1-typed-float-from-long: float(4) == oracle).
$t2 = new T2; show('p1-float-default-long-literal', $t2->f);
$t3 = new T2; for ($k = 0; $k < 2; $k++) { $t3->f = $t3->y + 1; } show('p1-typed-float-from-long-div', $t3->f);
echo "FX-SL2-DIV DONE\n";
