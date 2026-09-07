<?php
// S-163 fixture parità L-AU1 — forme di loader autoload array-callable, fast
// E pieno: bilaterale oracle==pin BYTE-ID (gate di promozione). Copre la forma
// AMMESSA ([obj, metodo-utente PUBBLICO non-static simple arità-1]) e le
// NON-ammesse che DEVONO restare sul cammino generico invariato. Ogni loader
// si de-registra per non inquinare i casi successivi.
error_reporting(E_ALL);

function probe($tag, $cls) {
    if (class_exists($cls)) { echo "$tag: defined\n"; } else { echo "$tag: miss\n"; }
}

// 1. AMMESSA: metodo pubblico d'istanza, 1 param, definisce la classe
class L1 { public function load($c) { if ($c === 'AuC1') { eval('class AuC1 {}'); } } }
$l1 = new L1();
spl_autoload_register([$l1, 'load']);
probe('t1-def', 'AuC1');
probe('t1-miss', 'AuMiss1');
spl_autoload_unregister([$l1, 'load']);

// 2. AMMESSA: case-insensitive sul nome metodo + miss no-op
class L2 { public function loadClass($c) { /* no-op */ } }
$l2 = new L2();
spl_autoload_register([$l2, 'LOADclass']);
probe('t2-miss', 'AuMiss2');
spl_autoload_unregister([$l2, 'LOADclass']);

// 3. AMMESSA: metodo EREDITATO dal parent (risoluzione lungo la catena)
class L3p { public function load($c) { if ($c === 'AuC3') { eval('class AuC3 {}'); } } }
class L3 extends L3p {}
$l3 = new L3();
spl_autoload_register([$l3, 'load']);
probe('t3-def', 'AuC3');
spl_autoload_unregister([$l3, 'load']);

// 4. NON ammessa: forma STATICA per stringa ['Cls','m'] (resta sul generico)
class L4 { public static function sload($c) { if ($c === 'AuC4') { eval('class AuC4 {}'); } } }
spl_autoload_register(['L4', 'sload']);
probe('t4-def', 'AuC4');
spl_autoload_unregister(['L4', 'sload']);

// 5. NON ammessa: metodo STATICO via [obj,'m'] (is_static esclude)
$l5 = new L4();
spl_autoload_register([$l5, 'sload']);
probe('t5-miss', 'AuMiss5');
spl_autoload_unregister([$l5, 'sload']);

// 6. NON ammessa: default param (n_params!=1) e variadica
class L6 { public function load($c, $x = 5) { /* no-op */ } }
$l6 = new L6();
spl_autoload_register([$l6, 'load']);
probe('t6-miss', 'AuMiss6');
spl_autoload_unregister([$l6, 'load']);

// 7. eccezione DENTRO il loader attraversa il fast path
class L7 { public function load($c) { if ($c === 'AuBoom') { throw new RuntimeException('au'); } } }
$l7 = new L7();
spl_autoload_register([$l7, 'load']);
try { class_exists('AuBoom'); } catch (RuntimeException $e) { echo "CAUGHT {$e->getMessage()}\n"; }
spl_autoload_unregister([$l7, 'load']);

// 8. lista LIVE: un loader ne DE-registra un ALTRO (successivo) durante il
// lookup — il successivo non scatta (S-71.2). Il caso «si de-registra DA SOLO
// con successore» è divergente PRE-esistente: sta in fx-au-div.php (§3.27).
class L8 {
    public $peer;
    public function load($c) { spl_autoload_unregister([$this->peer, 'load']); echo "L8 fired\n"; }
}
$l8 = new L8();
class L8b { public function load($c) { echo "L8b fired\n"; } }
$l8b = new L8b();
$l8->peer = $l8b;
spl_autoload_register([$l8, 'load']);
spl_autoload_register([$l8b, 'load']);
probe('t8-miss', 'AuMiss8');
spl_autoload_unregister([$l8, 'load']);

// 9. DUE loader array-callable in catena (k=2: il secondo definisce)
class L9a { public function load($c) { /* no-op */ } }
class L9b { public function load($c) { if ($c === 'AuC9') { eval('class AuC9 {}'); } } }
$l9a = new L9a(); $l9b = new L9b();
spl_autoload_register([$l9a, 'load']);
spl_autoload_register([$l9b, 'load']);
probe('t9-def', 'AuC9');
spl_autoload_unregister([$l9a, 'load']);
spl_autoload_unregister([$l9b, 'load']);

// 10. AMMESSA: return-hint (non di parametro) resta simple
class L10 { public function load($c): void { /* no-op */ } }
$l10 = new L10();
spl_autoload_register([$l10, 'load']);
probe('t10-miss', 'AuMiss10');
spl_autoload_unregister([$l10, 'load']);

echo "FX-AU DONE\n";
