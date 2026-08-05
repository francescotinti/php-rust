<?php
// Fixture SCRITTURA IN PLACE SULLA PILA (Matsakis): BinaryAdd fa
// *last_mut()=v — se un prestito H-C1x mettesse in pila un ALIAS dello
// slot proprieta', l'add in place scriverebbe DENTRO l'oggetto.
// ATTESA: le proprieta' NON cambiano per effetto delle espressioni.
class P { public $a = 1; public $b = 2; }
$o = new P;
$r = $o->a + $o->b + $o->a;    // catena con riuso: 1+2+1
echo $r, "\n";                  // 4
echo $o->a, " ", $o->b, "\n";  // 1 2 (INTATTE)
$s = $o->a + 10;
echo $s, " ", $o->a, "\n";     // 11 1
// In un loop stretto (la forma del giudice), con verifica finale:
$sum = 0;
for ($i = 0; $i < 1000; $i++) { $sum += $o->a + $o->b; }
echo $sum, " ", $o->a, " ", $o->b, "\n";  // 3000 1 2
