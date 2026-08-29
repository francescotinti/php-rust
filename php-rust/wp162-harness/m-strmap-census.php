<?php
// S-162 L-AM2 — driver census (criterio p.5): 200 chiamate array_map('inc1')
// su array di 1.000 interi = 200.000 elementi. Arbitro: Delta hostcall_n
// atteso 200.000 ESATTO sul nome array_map (probe A senza leva conta il
// passaggio call_callable per-elemento; probe B con leva no), altri nomi
// zero. Marcatore di parita' dalla somma: 200*(1+1000)=200200.
function inc1($x) { return $x + 1; }
$arr = range(0, 999);
$acc = 0;
for ($i = 0; $i < 200; $i++) {
    $r = array_map('inc1', $arr);
    $acc += $r[0] + $r[999];
}
echo "SM-OK $acc\n";
