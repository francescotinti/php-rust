<?php
// fx-sl3-div.php — forme della fetta 3 con DIAGNOSTICO e divergenza PRE-esistente (gate pin==stash s172,
// INVARIANTE): la creazione di prop dinamica su classe con __get (senza __set) emette il Deprecated con
// il NUMERO DI RIGA della funzione `show` (riga 11 di fx-sl3) invece della riga dell'assegnazione
// (oracle: riga 92) — divergenza di attribuzione di riga, catalogata S-173 (TODO master), NON toccata da P3/P4.
function show($label, $v) { echo $label, ': '; var_dump($v); }
class G { public $y = 3; private $d = ['x' => 4]; public function __get($n) { return $this->d[$n] ?? null; } }
$g = new G; for ($k = 0; $k < 2; $k++) { $g->z = $g->y + 1; } show('p4-dynamic-on-magic', $g->z);
echo "FX-SL3-DIV DONE\n";
