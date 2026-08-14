<?php
// S-139 — bench candidato leva dim-read: SOLO la statement `$x = $e->data['k']`
// (oggetto singolo, chiave fissa esistente: niente ctor per-iter, niente
// crescita array). Lowering DA VERIFICARE col dump (PHPR_DUMP_OPS) prima di
// ogni modello: FieldRead{[Prop,Index]}? o PropGet+FetchDim? — istruttoria
// s139-istruttoria-dimread.md.
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$e->data['k'] = 7;
$x = 0;
for($i=0;$i<3000000;$i++){
  $x = $e->data['k'];
}
echo $x, "\n";
