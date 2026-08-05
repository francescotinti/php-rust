<?php
// Fixture RIASSEGNAZIONE INTRA-ESPRESSIONE (Stogov): $o->x + ($o->x = ...)
// L'ordine di valutazione PHP e' left-to-right: il primo $o->x si legge
// PRIMA che l'assegnamento lo cambi. Un prestito dello slot valore
// leggerebbe il valore NUOVO — il controesempio del borrow nudo.
// ATTESA: 1+99=100, poi x=99; concat: "a" . assegna "b" => ab, x=b.
class X { public $x = 1; }
$o = new X;
echo $o->x + ($o->x = 99), "\n";   // 100
echo $o->x, "\n";                  // 99
$o->x = "a";
echo $o->x . ($o->x = "b"), "\n";  // ab
echo $o->x, "\n";                  // b
// Variante con lettura DOPO l'assegnamento nello stesso statement:
$o->x = 10;
echo ($o->x = 20) + $o->x, "\n";   // 40
