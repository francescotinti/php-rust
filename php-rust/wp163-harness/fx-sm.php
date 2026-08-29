<?php
// S-162 fixture parita' L-AM2 — forme string-callable di array_map, fast E
// pieno: bilaterale oracle==pin BYTE-ID (gate di promozione). Copre le forme
// AMMESSE (utente semplice arita'-1) e le NON-ammesse che DEVONO restare sul
// loop generico invariato.
error_reporting(E_ALL);

function f1($x) { return $x * 2; }                    // ammessa: semplice, 1 param
function f2($x, $y = 10) { return $x + $y; }          // default: n_params!=1 o non-simple, si dichiara
function f3(int $x): int { return $x - 1; }           // hint: NON simple_call
function f4(&$x) { $x++; return $x; }                 // by-ref: NON simple_call
function f5(...$xs) { return count($xs); }            // variadica
class K { public static function sm($x) { return $x . '!'; } }

$a = [3 => 1, 'k' => 2, 5];
var_dump(array_map('f1', $a));                        // chiavi preservate
var_dump(array_map('\\f1', $a));                      // prefisso backslash
var_dump(array_map('F1', $a));                        // case-insensitive
var_dump(array_map('f2', $a));
var_dump(array_map('f3', $a));
var_dump(array_map('f5', $a));
var_dump(array_map('K::sm', [1, 2]));                 // metodo statico stringa
var_dump(array_map('strtoupper', ['a', 'b']));        // builtin: loop generico
var_dump(array_map('trim', ['  x  ', "\ty\n"]));
var_dump(array_map('f1', []));                        // array vuoto
var_dump(array_map('f1', [1, 2], [3, 4]));            // multi-array: ramo N>1
// eccezione DENTRO la callback attraversa il fast path
function boom($x) { if ($x === 2) { throw new RuntimeException('b'); } return $x; }
try { array_map('boom', [1, 2, 3]); } catch (RuntimeException $e) { echo "CAUGHT {$e->getMessage()}\n"; }
// funzione dichiarata da eval (linked): risoluzione linked_functions
eval('function ev1($x) { return $x + 100; }');
var_dump(array_map('ev1', [1, 2]));
// --- S-163 estensioni (rev. S-162 az.1): forme AMMESSE dal fast path ma
// prima non esercitate (l'ammissione simple_call e' piu' larga delle fixture)
function h163($x = 5) { return $x + 1; }              // default su UNICO param: AMMESSA
function r163($x): int { return $x * 3; }             // return-hint (non di param): AMMESSA
var_dump(array_map('h163', $a));
var_dump(array_map('r163', [1, 2]));
// namespaced via stringa (dichiarata via eval per non spezzare il file)
eval('namespace nsx163; function nf($x) { return $x + 7; }');
var_dump(array_map('nsx163\\nf', [1, 2]));            // qualificata
var_dump(array_map('\\nsx163\\nf', [1, 2]));          // fully-qualified
// generator user-fn via stringa: ESCLUSA da simple_call, braccio no-frame
function gen163($x) { yield $x; yield $x * 10; }
foreach (array_map('gen163', [1, 2]) as $k => $g) {
    echo "gen[$k] ", get_class($g), ":";
    foreach ($g as $v) { echo " $v"; }
    echo "\n";
}
echo "FX-SM DONE\n";
// NOTA S-162: le forme by-ref ('f4') e undefined-callback stanno in
// fx-sm-div.php (divergenze PRE-esistenti del pin, gate a INVARIANZA
// pin==gemelloA; la leva non le tocca per costruzione).
