<?php
// S-140 — giudice HC1 v2 a DENSITÀ 6 check/iter (criterio v2: il tentativo r5
// ha dato D=+3,3 < pavimento 4,0 su 2 check/iter — lo strumento non risolve
// il prezzo per-check ~1,65 ns; qui 5 parametri tipizzati + return = 6 check
// Object/iter, D atteso ≈ 6×prezzo, sopra il pavimento se il canale è reale).
class En { public int $id; public function __construct(int $i){ $this->id=$i; } }
function f(En $a, En $b, En $c, En $d, En $e): En { return $a; }
$e = new En(1);
$x = null;
for($i=0;$i<3000000;$i++){
  $x = f($e, $e, $e, $e, $e);
}
echo $x->id, "\n";
