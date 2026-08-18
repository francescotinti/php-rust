<?php
// S-154 p.2 S8 — decomposizione k: stack di FUNZIONI (nessuna classe),
// limit=2: (k_metodo − k_noclass)/… = gruppo class+type per frame.
function nc3() { return debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 2); }
function nc2() { return nc3(); }
function nc1() { return nc2(); }
$n = (int)(getenv('BTN') ?: 100000);
$acc = 0;
for ($i = 0; $i < $n; $i++) {
    $x = nc1();
    $acc += count($x);
}
echo "BT2NC-OK n=$n frames_tot=$acc\n";
