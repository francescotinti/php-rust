<?php
// S-153 sonda p.1 — teardown frame con slot oggetto, SENZA $this: funzione vuota.
// Atteso (piano): frame_teardown.borrow k=1 (solo note_slow), Sweep.borrow k=1.
class TdL {}
function f($o) {}
$n = (int)(getenv('TDN') ?: 100000);
$x = new TdL;
for ($i = 0; $i < $n; $i++) {
    f($x);
}
echo "TD-LOCAL-OK n=$n\n";
