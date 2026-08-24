<?php
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$map=[]; $s=0;
for($i=0;$i<12000000;$i++){
  $e = new En($i);
  $e->data['k'] = $i;
  $map[$i & 2047] = $e;
  $s += $e->id & 1;
}
echo $s,"\n";
