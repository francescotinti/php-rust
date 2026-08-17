<?php
// S-152 atto 2 — istruttoria debug_backtrace 21,3M (pesca outlier per NOME).
// Replica la forma dominante Doctrine: IGNORE_ARGS + limit=2, chiamata da
// dentro metodi (stack ≥4). Eseguito col probe census s151 a DUE valori di N
// (env BTN): k alloc/chiamata = Δconteggio/ΔN — le costanti di setup si
// elidono nella differenza. CONTEGGI, mai tempo.
class BtProbe {
    function lvl3() { return debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 2); }
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
echo "BT-COUNT-OK n=$n frames_tot=$acc\n";
