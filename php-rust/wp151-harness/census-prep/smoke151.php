<?php
// smoke151 — collaudo a esito ESATTO del probe tranche-5 (S-151).
// Gli attesi sono DICHIARATI in census-prep/smoke-atteso.md PRIMA di leggere
// qualunque output (criterio s151-criterio-census.md §3, KS-G1).
class K { public $a = 1; public $b = 2; public $c = 3; }
$keep = new K();          // nascita 1 (viva a fine request)
$tmp = [];
for ($i = 0; $i < 7; $i++) { $tmp[] = new K(); }  // nascite 2..8
$tmp = null;              // 7 morti prima dello snapshot
$drop = new K();          // nascita 9
$drop = null;             // spiazza un OGGETTO -> gc_note(Object) (C4); morte 8
$alias = $keep;           // clone dell'handle (C1)
$keep->a = 42;            // C5: displaced scalar (drop) == 1
$r = $keep->a;            // C5: clone scalare in famiglia-prop (>=1)
echo "SMOKE151 r=$r\n";
