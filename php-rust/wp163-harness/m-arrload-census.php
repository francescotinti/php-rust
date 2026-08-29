<?php
// S-163 census L-AU1 — driver d'arbitrato: 200000 miss class_exists con
// loader [$obj,'loadClass'] NO-OP (forma AMMESSA dal fast path). N piccolo
// e MARCATO: il census conta gli eventi vecargs-at-bind per host-call
// (s144/s148/s149) — attesa: Delta A-B = 200000 ESATTO su class_exists
// (il vec![arg] di try_autoload), altri nomi ZERO.
class CensusLoader163 { public function loadClass($c) { /* miss cacheato: no-op */ } }
$l = new CensusLoader163();
spl_autoload_register([$l, 'loadClass']);
$cn = 'Doctrine\\Tests\\Models\\CMS\\MissingProbeEntity';
$acc = 0;
for ($i = 0; $i < 200000; $i++) {
    if (!class_exists($cn)) { $acc++; }
}
echo "MALC-OK $acc\n";
