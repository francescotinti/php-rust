<?php
// A-TH-97-1 — IL FALSIFICATORE CHE MORDE (S-96.0).
//
// Il controesempio di Hoare (t1-hoare-exc.php) e' vero ma MASCHERATO nella sua
// forma letterale: l'analisi da' un arco eccezionale a OGNI op della regione
// protetta, quindi se un op senza def precede quello con la def dentro la
// regione, quel primo op ri-inietta il live-set dell'handler e il difetto non
// si vede. Il difetto e' osservabile solo quando l'op che DEFINISCE lo slot e'
// il PRIMO della regione — cioe' quando non gli serve nulla sulla pila.
//
// Tre op che possono stare per prime ed essere una def:
//   unset($m)   -> UnsetPath { nkeys: 0 }  (def dello slot)
//   $m = &$o    -> BindRef   { target }    (def del bersaglio)
//   global $m   -> BindRef   (alias verso la tabella globale)
//
// Contatori attesi (binario di censimento, questo file da solo):
//   PRE-fix : would_take=6 would_take_rc=3 would_take_safe=1 sites_movable=8 sites_safe=4
//   POST-fix: would_take=4 would_take_rc=1 would_take_safe=0 sites_movable=6 sites_safe=3
// L'output del programma e' IDENTICO fra i due (la fase e' di sola misura).
function u1() { $m="v"; echo $m; try { unset($m); throw new Exception(); } catch (\Throwable $e) { echo isset($m)?"S":"U"; } }
function u2($o) { $m="v"; echo $m; try { $m = &$o; throw new Exception(); } catch (\Throwable $e) { echo $m; } }
function u3() { $m="v"; echo $m; try { global $m; throw new Exception(); } catch (\Throwable $e) { echo $m; } }
foreach ([1,2,3] as $i) {
  try { $i==1 ? u1() : ($i==2 ? u2("o") : u3()); } catch (\Throwable $e) { echo "[o]"; }
  echo "\n";
}
