<?php
// S-154 p.2 S8 — decomposizione k: come bt-count.php ma limit=1
// (k2−k1 = costo per-frame; k1−(k2−k1) = costo fisso per chiamata).
class BtProbe {
    function lvl3() { return debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 1); }
    function lvl2() { return $this->lvl3(); }
    function lvl1() { return $this->lvl2(); }
}
$n = (int)(getenv('BTN') ?: 100000);
$p = new BtProbe;
$acc = 0;
for ($i = 0; $i < $n; $i++) {
    $x = $p->lvl1();
    $acc += count($x);
}
echo "BT2L1-OK n=$n frames_tot=$acc\n";
