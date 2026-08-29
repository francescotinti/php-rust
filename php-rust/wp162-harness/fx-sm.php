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
echo "FX-SM DONE\n";
// NOTA S-162: le forme by-ref ('f4') e undefined-callback stanno in
// fx-sm-div.php (divergenze PRE-esistenti del pin, gate a INVARIANZA
// pin==gemelloA; la leva non le tocca per costruzione).
