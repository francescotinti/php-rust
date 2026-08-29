<?php
// S-163 L-AU1 — giudice micro miss/autoload ARRAY-callable: 1 class_exists
// MISS per iter con loader [$obj,'loadClass'] NO-OP (modello Composer REALE:
// spl_autoload_register([$loader,'loadClass']) a miss CACHEATO). Esercita il
// bundle per-miss del cammino generico: args-Vec (try_autoload) + elems-Vec
// (invoke_array_callable) + to_vec del nome metodo. N letterale. Parità:
// conteggio. Gemello di m-missload.php (S-157) con la SOLA forma del loader
// cambiata: closure → array-callable.
class ProbeLoader163 { public function loadClass($c) { /* miss cacheato: no-op */ } }
$l = new ProbeLoader163();
spl_autoload_register([$l, 'loadClass']);
$cn = 'Doctrine\\Tests\\Models\\CMS\\MissingProbeEntity';
$acc = 0;
for ($i = 0; $i < 10000000; $i++) {
    if (!class_exists($cn)) { $acc++; }
}
echo "MAL-OK $acc\n";
