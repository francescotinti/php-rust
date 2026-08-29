<?php
// S-163 fixture INVARIANZA L-AU1 — divergenza PRE-esistente del pin vs oracle
// scoperta al collaudo fixture (PRIMA della leva): loader che si DE-registra
// DA SOLO durante il lookup CON un successore registrato — l'oracle TERMINA
// la camminata (il successore NON scatta), phpr prosegue sul cursore
// element-stable e lo chiama. Estende il perimetro S-71.2 (che copriva il
// self-unregister SENZA successore). GATE: pin(B) == gemelloA BYTE-ID (la
// leva NON tocca il cursore per costruzione). Catalogo: §3.27.
error_reporting(E_ALL);
class D8 { public function load($c) { spl_autoload_unregister([$this, 'load']); echo "D8 fired\n"; } }
class D8b { public function load($c) { echo "D8b fired\n"; } }
$d8 = new D8(); $d8b = new D8b();
spl_autoload_register([$d8, 'load']);
spl_autoload_register([$d8b, 'load']);
var_dump(class_exists('AuDivMiss'));
spl_autoload_unregister([$d8b, 'load']);
echo "FX-AU-DIV DONE\n";
