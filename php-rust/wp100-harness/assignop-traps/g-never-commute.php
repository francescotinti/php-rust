<?php
// Trappola (g) — il fold AssignOp non deve MAI commutare: la POSIZIONE
// degli operandi e' osservabile nei messaggi d'errore (string/array a
// sinistra vs destra) e nei tipi non commutativi (sub, shift, concat-num).
try { $a = [1]; $a += 1; } catch (\TypeError $e) { echo "g1:", $e->getMessage(), "\n"; }
try { $b = 1; $b += [1]; } catch (\TypeError $e) { echo "g2:", $e->getMessage(), "\n"; }
try { echo 3 + []; } catch (\TypeError $e) { echo "g3:", $e->getMessage(), "\n"; }
$s = 10; $s -= 3; echo "g4:", $s, "\n";
$t = 3;  $u = 10; $u -= $t; echo "g5:", $u, "\n";
$v = 1;  $v <<= 3; echo "g6:", $v, "\n";
$w = 8;  $w >>= 2; echo "g7:", $w, "\n";
// by-ref alias (A-ST-100-3): lettura e AssignOp ATTRAVERSO il ref.
$p = 7; $r = &$p;
echo "g8:", $r + 1, "\n";
$r += 2;
echo "g9:", $p, "\n";
