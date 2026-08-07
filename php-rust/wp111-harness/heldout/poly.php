<?php // held-out 1 (S-111): tipi alternati + dispatch megamorfico — CONGELATO
class PA { public function v($x){ return $x + 1; } }
class PB { public function v($x){ return $x + 2; } }
class PC { public function v($x){ return $x * 2; } }
class PD { public function v($x){ return $x - 1; } }
$objs = [new PA, new PB, new PC, new PD];
$vals = [7, 2.5, "3", true];
$s = 0;
for($i=0;$i<18000000;$i++){
  $o = $objs[$i & 3];
  $x = $vals[($i >> 2) & 3];
  $s += $o->v($x) + $x + ($i % 7);
}
echo (int)$s, "\n";
