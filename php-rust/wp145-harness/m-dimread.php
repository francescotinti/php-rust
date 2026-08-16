<?php
// S-145 — bench leva FR1 «dim-read fuso»: SOLO la lettura `$x = $e->data['k']`
// (oggetto singolo, chiave fissa esistente; lowering atteso PropGetSlot+
// PushConst+FetchDim — verificato con PHPR_DUMP_OPS in sessione). Stessa
// forma-famiglia di m-dimrmw/m-dimwrite (S-136/S-138).
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$e->data['k'] = 41;
$x = 0;
for($i=0;$i<3000000;$i++){
  $x = $e->data['k'];
}
echo $x + 1, "\n";
