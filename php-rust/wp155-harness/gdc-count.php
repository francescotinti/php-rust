<?php
// S-155 p.3 — conteggio get_declared_classes (4,56M alloc nel census s154,
// CHIAMATE ignote): k alloc/chiamata a DUE N (env GDN) e a DUE popolazioni
// di classi (env GDX = classi extra via eval, costante di setup elisa da
// Δ/ΔN). Eseguito col probe census s155 (post-CE1; il cammino gdc non è
// toccato da CE1). CONTEGGI, mai tempo.
$extra = (int)(getenv('GDX') ?: 0);
for ($i = 0; $i < $extra; $i++) { eval("class GdcExtra$i {}"); }
$n = (int)(getenv('GDN') ?: 100000);
$acc = 0;
for ($i = 0; $i < $n; $i++) {
    $a = get_declared_classes();
    $acc += count($a);
}
$c = count(get_declared_classes());
echo "GDC-COUNT-OK n=$n classi=$c acc_ok=" . (($acc === $n * $c) ? 1 : 0) . "\n";
