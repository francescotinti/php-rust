<?php
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$map=[]; $s=0; $e = new En(1); $e->data['k']=1;
for($i=0;$i<3000000;$i++){
  $map[$i & 2047] = $e;
  $s += 1;
}
echo $s,"\n";
