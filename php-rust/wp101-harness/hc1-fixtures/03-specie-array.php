<?php
// Fixture specie ARRAY (Bak): COW sull'array letto da una proprieta'.
// ATTESA: la copia letta muta indipendentemente; count 2/3; il write-back
// sostituisce per intero.
class A { public $arr = [1, 2]; }
$o = new A;
$c = $o->arr;          // lettura: copia logica
$c[] = 99;             // muta la COPIA
echo count($o->arr), " ", count($c), "\n";   // 2 3
$o->arr[] = 7;         // muta la PROPRIETA' (fetch-dim write path)
echo count($o->arr), " ", count($c), "\n";   // 3 3
echo $o->arr[2], " ", $c[2], "\n";           // 7 99
$o->arr = $c;          // write-back intero
echo implode(",", $o->arr), "\n";            // 1,2,99
