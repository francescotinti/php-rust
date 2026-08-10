<?php
class RT { public int $a=1; protected string $b='x'; private array $c=[]; public ?RT $d=null; public $e;
  public function f0(int $x, string $y='s'): int { return $x; }
  public function f1(){ return 1; }
  public function f2($a){ return $a; }
  public function f3(): ?string { return null; }
  public function f4(array $z=[]){ return $z; }
}
$s=0;
for($i=0;$i<400000;$i++){
  $rc = new ReflectionClass('RT');
  $s += count($rc->getMethods()) + count($rc->getProperties());
  $o = $rc->newInstanceWithoutConstructor();
  $m = $rc->getMethod('f0');
  $s += $m->invoke($o, $i & 15);
}
echo $s,"\n";
