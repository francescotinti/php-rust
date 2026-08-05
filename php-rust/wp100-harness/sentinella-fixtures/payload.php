<?php
// Fixture payload della sentinella estesa (A-PE-101-3): calcolo
// deterministico + invarianti di reset per-richiesta (static, object id).
function tally() { static $n = 0; $n++; return $n; }
$s = 0;
for ($i = 0; $i < 2000; $i++) { $s = $s + $i + $i + $i; $s -= 2 * $i; }
$a = [];
for ($i = 0; $i < 50; $i++) { $a["k$i"] = $i * 7; }
$o = new stdClass;
echo "P1:sum=", $s, ":tally=", tally(), ":oid=", spl_object_id($o),
     ":arr=", array_sum($a), ":glob=", isset($GLOBALS['p1_marker']) ? 'LEAK' : 'clean', "\n";
$GLOBALS['p1_marker'] = 1;
