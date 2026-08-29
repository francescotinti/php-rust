<?php
// S-157 L-AL1 — giudice micro miss/autoload: 1 class_exists MISS per iter con
// autoloader closure NO-OP registrato (modello Composer a miss CACHEATO:
// ritorna senza definire). Esercita il plumbing di try_autoload: guard-key,
// arg del loader, args-Vec della chiamata. N letterale. Parità: conteggio.
spl_autoload_register(function ($c) { /* miss cacheato: no-op */ });
$cn = 'Doctrine\\Tests\\Models\\CMS\\MissingProbeEntity';
$acc = 0;
for ($i = 0; $i < 10000000; $i++) {
    if (!class_exists($cn)) { $acc++; }
}
echo "ML-OK $acc\n";
