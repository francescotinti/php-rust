<?php
// fx-sl1.php — fixture BILATERALE (oracle == pin byte-id) per la leva L-SL1
// «forma sigillata Long» (S-171, criterio p.5): ogni cammino che ESCE dal
// dominio Long del fast path deve cadere al corpo esatto — overflow, shift
// fuori range, Div/Mod/Pow, slot Ref/Double/stringa numerica/null/bool,
// dst==l, typed by-ref, IncDec ai limiti e su tag non-Long, CmpJmpSC nelle
// 8 forme e su tag misti. SENZA forme che emettono diagnostici (quelle
// stanno in fx-sl1-div.php: pin==stash s166, divergenze PRE-esistenti).
function show($label, $v) { echo $label, ': '; var_dump($v); }

// --- (a) BinarySCSCDst: forma del driver `$s += $i*3 - ($i>>2)` ---
$s = 0; for ($i = 0; $i < 100; $i++) { $s += $i*3 - ($i>>2); } show('dq100', $s);
$s = 0; $i = PHP_INT_MAX; $s += $i*3 - ($i>>2); show('mul-overflow', $s);
$s = PHP_INT_MAX; $i = 10; $s += $i*3 - ($i>>2); show('add-overflow-dst', $s);
$s = PHP_INT_MIN; $i = -10; $s += $i*3 - ($i>>2); show('sub-overflow-neg', $s);
$s = 0; $i = 7; $s += $i*3 - ($i>>64); show('shr-64', $s);
$s = 0; $i = -7; $s += $i*3 - ($i>>70); show('shr-70-neg', $s);
$s = 0; $i = 7; $s += $i*3 - ($i<<63); show('shl-63', $s);
$s = 0; $i = 7; $s += $i*3 - ($i<<64); show('shl-64', $s);
try { $s = 0; $i = 7; $s += $i*3 - ($i>>-1); show('shr-neg', $s); } catch (ArithmeticError $e) { echo "shr-neg: ", get_class($e), ' ', $e->getMessage(), "\n"; }
try { $s = 0; $i = 7; $s += $i*3 - ($i<<-1); show('shl-neg', $s); } catch (ArithmeticError $e) { echo "shl-neg: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$s = 0; $i = 9; $s += $i/3 - ($i%4); show('div-exact-mod', $s);
$s = 0; $i = 10; $s += $i/3 - ($i%4); show('div-inexact', $s);
try { $s = 0; $i = 10; $s += $i/0 - ($i%4); show('div-zero', $s); } catch (DivisionByZeroError $e) { echo "div-zero: ", get_class($e), ' ', $e->getMessage(), "\n"; }
try { $s = 0; $i = 10; $s += $i*3 - ($i%0); show('mod-zero', $s); } catch (DivisionByZeroError $e) { echo "mod-zero: ", get_class($e), ' ', $e->getMessage(), "\n"; }
$s = 0; $i = PHP_INT_MIN; $s += $i%-1 - ($i>>2); show('mod-min-neg1', $s);
$s = 0; $i = 3; $s += $i**3 - ($i>>1); show('pow', $s);
$s = 0; $i = 2; $s += $i**63 - ($i>>1); show('pow-overflow', $s);
$s = 0; $i = 6; $s += ($i&3) | ($i^5); show('bitops', $s);
$s = 0; $i = 2.5; $s += $i*3 - ($i*2); show('double-slot', $s);
$s = 1.5; $i = 4; $s += $i*3 - ($i>>1); show('double-dst', $s);
$s = 0; $i = "5"; $s += $i*3 - ($i>>1); show('numeric-string', $s);
$s = "10"; $i = 4; $s += $i*3 - ($i>>1); show('numeric-string-dst', $s);
$s = 0; $i = null; $s += $i*3 - ($i>>1); show('null-slot', $s);
$s = 0; $i = true; $s += $i*3 - ($i>>1); show('bool-slot', $s);
$s = 0; $r = &$s; $i = 4; $s += $i*3 - ($i>>1); show('dst-ref', $s); show('dst-ref-alias', $r); unset($r);
$s = 0; $i = 4; $q = &$i; $s += $i*3 - ($i>>1); show('src-ref', $s); unset($q);
$i = 5; $i += $i*3 - ($i>>1); show('dst==src', $i);
$s = 0; $i = 1; $s = $s + ($i*3 - ($i>>2)); show('assign-form', $s);
function typed(int &$x, int $i) { $x += $i*3 - ($i>>2); }
$t = 1; typed($t, 4); show('typed-ref', $t);
$t2 = PHP_INT_MAX; typed($t2, 4); show('typed-ref-overflow', $t2);

// --- (c) IncDec in place: limiti e tag ---
$i = PHP_INT_MAX; $i++; show('inc-max', $i);
$i = PHP_INT_MIN; $i--; show('dec-min', $i);
$i = PHP_INT_MAX - 1; for ($k = 0; $k < 2; $k++) { $i++; } show('inc-max-loop', $i);
$n = null; $n++; show('inc-null', $n);
$d = 1.5; $d++; show('inc-double', $d);
$str = "9"; $str++; show('inc-numstr', $str);
$str = "1.5"; $str++; show('inc-numstr-float', $str);
$rr = 1; $al = &$rr; $rr++; show('inc-ref', $al); unset($al);
for ($j = 10; $j > 0; $j--) {} show('dec-loop', $j);
for ($j = -3; $j <= 3; $j++) {} show('inc-loop-neg', $j);

// --- (b) CmpJmpSC: 8 forme su Long e su tag misti ---
foreach ([2, 3, 4, 2.5, 3.0, "3", "3.0", "abc", null, true, false] as $v) {
    $o = [];
    if ($v < 3) $o[] = 'lt';
    if ($v <= 3) $o[] = 'le';
    if ($v > 3) $o[] = 'gt';
    if ($v >= 3) $o[] = 'ge';
    if ($v == 3) $o[] = 'eq';
    if ($v != 3) $o[] = 'ne';
    if ($v === 3) $o[] = 'id';
    if ($v !== 3) $o[] = 'nid';
    echo 'cmp(', var_export($v, true), '): ', implode(',', $o), "\n";
}
$c = 0; for ($i = 0; $i < 5; $i++) { $c++; } show('lt-loop', $c);
$c = 0; for ($i = 5; $i >= 0; $i--) { $c++; } show('ge-loop', $c);
$c = 0; for ($i = 0; $i != 4; $i++) { $c++; } show('ne-loop', $c);
$c = 0; for ($i = 0; $i !== 4; $i++) { $c++; } show('nid-loop', $c);
$c = 0; $i = PHP_INT_MAX - 2; while ($i < PHP_INT_MAX) { $i++; $c++; } show('lt-max', $c);
$c = 0; $i = 0.5; while ($i < 3) { $i++; $c++; } show('lt-double-loop', $c);
$c = 0; $i = "1"; while ($i < 3) { $i++; $c++; } show('lt-numstr-loop', $c);
$rv = 2; $ra = &$rv; if ($rv < 3) echo "ref-lt ok\n"; if ($ra >= 2) echo "ref-ge ok\n"; unset($ra);
echo "FX-SL1 DONE\n";
