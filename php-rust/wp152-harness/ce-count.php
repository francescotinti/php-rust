<?php
// S-152 atto 2 — pesca class_exists 9,7M: k alloc/chiamata sui due rami
// (esiste / non esiste, autoload=false), due N via env CEN, Δ/ΔN elide il
// setup. Eseguito col probe census s151. CONTEGGI, mai tempo.
class CeProbe {}
$n = (int)(getenv('CEN') ?: 100000);
$hit = 0;
for ($i = 0; $i < $n; $i++) {
    if (class_exists('CeProbe', false)) { $hit++; }
    if (class_exists('Nope\Missing', false)) { $hit++; }
}
echo "CE-COUNT-OK n=$n hit=$hit\n";
