<?php
// Fixture REF NELLO SLOT (Stogov): &$o->x mette un Ref nello slot — la
// lettura DEVE deref-are (mai restituire il wrapper), la scrittura deve
// scrivere DENTRO il ref (gli alias vedono).
// ATTESA: 5 5 / 9 9 / 12 12.
class R { public $x = 5; }
$o = new R;
$r = &$o->x;
echo $o->x, " ", $r, "\n";    // 5 5
$r = 9;
echo $o->x, " ", $r, "\n";    // 9 9
$o->x = 12;
echo $o->x, " ", $r, "\n";    // 12 12
// Lettura in espressione: il valore e' il DEREF, e la mutazione via alias
// dopo la lettura non cambia il letto (niente prestito del wrapper).
$v = $o->x;
$r = 77;
echo $v, " ", $o->x, "\n";    // 12 77
