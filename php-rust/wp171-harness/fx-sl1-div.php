<?php
// fx-sl1-div.php — fixture pin==stash s166 (INVARIANTE) per la leva L-SL1:
// le forme dei cammini lenti che EMETTONO diagnostici (deprecazione
// float→int, Undefined variable su slot/dst, IncDec su null/bool/stringa
// non numerica). Il pin s166 qui DIVERGE già dall'oracle (riga dei diag
// accodati attribuita all'emit point successivo = §3.13 della stessa
// famiglia; warning assente su `$u2 +=` con dst indefinito = §3.11):
// divergenze PRE-esistenti, a catalogo PHPR_DIVERGENCES_FROM_PHP.md — la
// leva NON deve cambiarle (byte-id contro lo stash s166). Collaudo S-171
// 10:50: pin s166 == stash s166 al byte; vs oracle 18 righe di diff.
function show($label, $v) { echo $label, ': '; var_dump($v); }
$s = 0; $i = 2.5; $s += $i*3 - ($i>>1); show('double-slot-shr', $s);
$s = 0; $s += $undef*3 - ($undef>>1); show('undef-slot', $s);
$u2 += 5*3 - (5>>1); show('undef-dst', $u2);
$n = null; $n--; show('dec-null', $n);
$str = "a"; $str++; show('inc-str', $str);
$str = "Az"; $str++; show('inc-str-carry', $str);
$b = true; $b++; show('inc-bool', $b);
$un++; show('inc-undef', $un);
if ($undef2 < 3) echo "undef-lt true\n";
echo "FX-SL1-DIV DONE\n";
