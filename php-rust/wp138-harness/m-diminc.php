<?php
// S-138 — bench secondario leva FD1-ext RMW: SOLO `$e->data['k']++`
// (lowering atteso FieldIncDec{base:Local, steps:[Prop,Index], inc, !pre}).
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$e = new En(1);
$e->data['k'] = 0;
for($i=0;$i<3000000;$i++){
  $e->data['k']++;
}
echo $e->data['k'], "\n";
