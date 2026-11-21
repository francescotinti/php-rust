<?php
// S-194 package F behaviour teeth: functions of ANOTHER file (global and namespaced), loaded by s194-f-comportamento.php.
namespace { function g_add($a, $b = 10) { return $a + $b; } function g_ref(&$x, $y) { $x .= $y; return strlen($x); } function g_var(...$xs) { return count($xs) . ':' . implode(',', $xs); }
  function g_rec($n) { return $n <= 0 ? 0 : $n + g_rec($n - 1); } function strtoupper_ish($s) { return "G:" . $s; } }
namespace Lib { function add($a, $b) { return $a + $b; } function ref(&$x) { $x[] = count($x); return $x; } function strtoupper_ish($s) { return "L:" . $s; } function caller($s) { return helper($s); } }
