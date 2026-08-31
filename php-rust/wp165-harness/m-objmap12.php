<?php
// S-165 — guardia objmap RI-RISOLUTA (REGOLE §3, az.rev. S-154: tick ≤
// soglia/4; a N=3M il tick era 3,3 ns ⇒ N=12000000, tick 10 ms/N ≈ 0,83 ns).
// DERIVATO DICHIARATO di wp127-harness/micro-orm/objmap.php: SOLO 3000000→12000000.
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$map=[]; $s=0; $e = new En(1); $e->data['k']=1;
for($i=0;$i<12000000;$i++){
  $map[$i & 2047] = $e;
  $s += 1;
}
echo $s,"\n";
