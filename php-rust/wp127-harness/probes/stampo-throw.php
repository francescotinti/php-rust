<?php
// Default che LANCIA (const di classe inesistente): l'errore deve emergere a
// OGNI new (nessuno snapshot su Err — retry come Zend).
class T1 { public $bad = [MISSING_CONST_XYZ]; }
for ($i = 0; $i < 2; $i++) {
    try { $o = new T1; var_dump($o->bad); }
    catch (\Error $e) { echo "catch{$i}: ", $e->getMessage(), "\n"; }
}
echo "fine\n";
