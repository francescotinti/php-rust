<?php
// fx21-args-window (S-105 gate G3, audit-fuga della leva H-D args):
// il CONTENITORE degli argomenti non deve mai essere osservabile fuori
// dalla finestra di chiamata — si osservano solo i VALORI (func_get_args
// = snapshot, by-ref = cella del chiamante, variadic = PhpArray).
// Verdetto: byte-parity oracle<->phpr, PRE e POST leva.

function surplus($a) { $g = func_get_args(); return $a . ':' . implode(',', $g); }
function byref($x, &$y) { $y = $x * 2; return $y + 1; }
function vari($first, ...$rest) { return $first . '|' . implode('-', $rest) . '#' . count($rest); }
function vref(&...$rs) { foreach ($rs as &$r) { $r++; } unset($r); return implode(',', $rs); }
function f0() { return 'z'; }
function f2($p, $q2) { return $p + $q2; }
function exact2($a, $b) { $g = func_get_args(); return implode('+', $g); }

echo surplus('a', 'b', 'c', 'd'), "\n";        // surplus oltre l'arita' dichiarata
$q = 5; echo byref(10, $q), ' ', $q, "\n";     // aliasing by-ref verso il chiamante
echo vari('x'), "\n";                           // variadic vuoto
echo vari('x', 1, 2, 3), "\n";                  // variadic pieno
$m = 1; $n = 2; echo vref($m, $n), ' ', $m, $n, "\n"; // variadic by-ref
echo f0(), f2(3, 4), "\n";                      // arita' 0 e 2 esatte (fast path)
echo exact2(7, 8), "\n";                        // func_get_args senza surplus
// surplus con REFERENCE argomento: il surplus e' by-value, decade a clone
function sref($a) { $g = func_get_args(); $g[1] = 'mut'; return implode('/', $g); }
$r = 'orig'; $rr = &$r;
echo sref('k', $rr), ' ', $r, "\n";
