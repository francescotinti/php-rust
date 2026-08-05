<?php
// Trappola (b) — typed-ref: la coercizione di tipo fallisce PRIMA di ogni
// scrittura (il valore osservabile resta quello vecchio), nel regime
// strict del FILE ASSEGNANTE.
class T { public int $i = 1; }
$t = new T;
$r = &$t->i;
try { $r += "abc"; } catch (\TypeError $e) { echo "TE1:", $e->getMessage(), "\n"; }
echo "unchanged:", $t->i, "\n";
$r += "41";
echo "coerced:", $t->i, "\n";
try { $r += [1]; } catch (\TypeError $e) { echo "TE2:", $e->getMessage(), "\n"; }
echo "still:", $t->i, "\n";
