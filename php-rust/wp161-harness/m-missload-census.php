<?php
// S-161 arbitrato AL2 — driver CONTEGGI census: m-missload.php (wp158) con
// N 10000000→200000 (adattamento DICHIARATO; i conteggi scalano esatti).
// Parità: ML-OK 200000.
spl_autoload_register(function ($c) { /* miss cacheato: no-op */ });
$cn = 'Doctrine\\Tests\\Models\\CMS\\MissingProbeEntity';
$acc = 0;
for ($i = 0; $i < 200000; $i++) {
    if (!class_exists($cn)) { $acc++; }
}
echo "ML-OK $acc\n";
