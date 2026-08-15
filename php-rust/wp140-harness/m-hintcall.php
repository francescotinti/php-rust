<?php
// S-140 — giudice leva HC1 «hint-check senza clone»: SOLO la statement
// `$x = f($e)` con parametro E return tipizzati a CLASSE (il canale del
// deref_clone in coerce_or_check_hint: 2 check/iter, entrambi su Object).
// Niente ctor per-iter, corpo minimo: il canale è il verify, non il lavoro.
class En { public int $id; public function __construct(int $i){ $this->id=$i; } }
function f(En $e): En { return $e; }
$e = new En(1);
$x = null;
for($i=0;$i<3000000;$i++){
  $x = f($e);
}
echo $x->id, "\n";
