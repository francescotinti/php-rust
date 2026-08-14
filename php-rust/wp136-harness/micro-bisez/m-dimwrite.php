<?php
// S-136 — bench del modello tempo FieldAssign: SOLO la statement
// `$e->data['k'] = $i` (oggetto singolo, chiave fissa: niente ctor per-iter,
// niente crescita array). Il lowering è FieldAssign{base:Local, steps:[Prop,Index]}
// (verificato col dump, s136-istruttoria-dimwrite.md).
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$s = 0;
for($i=0;$i<3000000;$i++){
  $e->data['k'] = $i;
  $s += 1;
}
echo $s, ",", $e->data['k'], "\n";
