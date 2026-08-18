<?php
// S-153 sonda p.1 — teardown frame con $this: metodo VUOTO su oggetto stabile.
// Atteso (piano): frame_teardown.borrow k=2 (id-probe + note_slow), Sweep.borrow k=1.
class TdProbe {
    function m() {}
}
$n = (int)(getenv('TDN') ?: 100000);
$p = new TdProbe;
for ($i = 0; $i < $n; $i++) {
    $p->m();
}
echo "TD-THIS-OK n=$n\n";
