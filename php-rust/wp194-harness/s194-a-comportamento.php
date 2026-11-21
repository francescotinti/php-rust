<?php
// S-194 package A (A3) behaviour tooth: compare-and-branch forms. Expected: identical to the oracle.
function t($a, $b) {
  $o = [];
  if ($a < $b) $o[] = '<'; if ($a <= $b) $o[] = '<='; if ($a > $b) $o[] = '>'; if ($a >= $b) $o[] = '>=';
  if ($a == $b) $o[] = '=='; if ($a != $b) $o[] = '!='; if ($a === $b) $o[] = '==='; if ($a !== $b) $o[] = '!==';
  $o[] = $a <=> $b;
  return implode(' ', $o);
}
$vals = [0, 1, -1, PHP_INT_MAX, PHP_INT_MIN, 1.0, 0.5, NAN, "1", "1e0", "abc", "", null, true, false, [], [1]];
foreach ($vals as $i => $a) foreach ($vals as $j => $b) echo $i, ',', $j, ': ', t($a, $b), "\n";
function loops($n) {
  $c = 0; for ($i = 0; $i < $n; $i++) { $c++; } echo "a $c $i\n";
  $c = 0; for ($i = 0; $i < $n; $i++) { if ($i == 3) $n = 5; $c++; } echo "b $c $i $n\n";
  $m = 10; $c = 0; for ($i = 0; $i < $m; $i++) { $m = $m - 1; $c++; } echo "c $c $i $m\n";
  $f = 4.5; $c = 0; for ($i = 0; $i < $f; $i++) { $c++; } echo "d $c $i\n";
  $s = "7"; $c = 0; for ($i = 0; $i < $s; $i++) { $c++; } echo "e $c $i\n";
  $r = 6; $q = &$r; $c = 0; for ($i = 0; $i < $r; $i++) { if ($i == 2) $q = 4; $c++; } echo "f $c $i $r\n";
  $c = 0; for ($i = 10; $i > $n; $i--) { $c++; } echo "g $c $i\n";
  $c = 0; for ($i = 0; $i != $n; $i++) { $c++; if ($c > 50) break; } echo "h $c $i\n";
  $c = 0; for ($i = PHP_INT_MAX - 2; $i <= PHP_INT_MAX; $i++) { $c++; if ($c > 5) break; } echo "i $c "; var_dump($i);
  $c = 0; $i = 0; while ($i < $n) { $i++; $c += $i; } echo "j $c\n";
  $c = 0; $i = 0; do { $i++; } while ($i < $n); echo "k $i\n";
  $c = 0; for ($i = 0; $i < $undef; $i++) { $c++; } echo "l $c\n";
  $k = null; for ($i = 0; $i < $k; $i++) { } echo "m $i\n";
}
loops(8); loops(0); loops(-3);
function cond($v) { if ($v) echo "T"; else echo "F"; if (!$v) echo "t"; else echo "f"; echo $v && 1 ? 'A' : 'a', $v || 0 ? 'O' : 'o', "\n"; }
foreach ([true, false, 0, 1, 0.0, "0", "", "a", null, [], [0]] as $v) cond($v);
$g = 3; for ($i = 0; $i < $g; $i++) { echo $i; } echo "\n";
