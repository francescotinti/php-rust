<?php
// Fixture specie INT (Bak A-BA-102-3): il canale che H-C1a/b tocca per primo.
// ATTESA (scritta prima): somma esatta 60000, x finale 2, y invariato 1.
class C { public $x = 0; public $y = 1; }
$o = new C; $s = 0;
for ($i = 0; $i < 30000; $i++) { $o->x = $o->y + 1; $s += $o->x; }
echo $s, "\n";        // 60000
echo $o->x, "\n";     // 2
echo $o->y, "\n";     // 1
// Overflow del Long dentro una proprieta': promozione a Double, mai wrap.
$o->x = PHP_INT_MAX;
$o->x = $o->x + 1;
var_dump($o->x);      // float(9.2233720368547758E+18)
