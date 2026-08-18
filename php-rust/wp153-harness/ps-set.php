<?php
// S-153 sonda p.1 — PropSetPop IC-hit steady-state: classe plain, slot presente.
// Atteso (piano): PropSetPop.borrow k=1-2, PropSetPop.borrow_mut k=1.
class PsP { public $x = 0; }
$n = (int)(getenv('TDN') ?: 100000);
$o = new PsP;
for ($i = 0; $i < $n; $i++) {
    $o->x = $i;
}
echo "PS-SET-OK n=$n x=" . $o->x . "\n";
