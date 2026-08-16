<?php
// S-146 az.rev. S-145 #2 — m-dimrmw a DENSITA' 10x (30M iter vs 3M):
// stessa statement di wp138-harness/m-dimrmw.php, solo N scalato per
// risolvere il segnale +0,01 s (1 tick) osservato 3/3 in S-145.
// Risoluzione: 1 tick (0,01 s) / 3e7 = 0,33 ns/iter.
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$e->data['k'] = 0;
for($i=0;$i<30000000;$i++){
  $e->data['k'] += 1;
}
echo $e->data['k'], "\n";
