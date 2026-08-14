<?php
// S-138 — bench leva FD1-ext RMW: SOLO la statement `$e->data['k'] += 1`
// (oggetto singolo, chiave fissa esistente dopo la prima iter; lowering
// atteso FieldAssignOp{base:Local, steps:[Prop,Index], op:Add} — verificato
// con PHPR_DUMP_OPS, s138-criterio-rmw.md).
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$e->data['k'] = 0;
for($i=0;$i<3000000;$i++){
  $e->data['k'] += 1;
}
echo $e->data['k'], "\n";
