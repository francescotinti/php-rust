<?php
class En { public int $id; public string $name; public array $data=[]; public ?En $ref=null;
  public function __construct(int $i){ $this->id=$i; $this->name="n$i"; }
}
$s=0;
for($i=0;$i<3000000;$i++){
  $e = new En($i);
  $a = [];
  $a['k'] = $i;
  $s += $e->id & 1;
}
echo $s,"\n";
