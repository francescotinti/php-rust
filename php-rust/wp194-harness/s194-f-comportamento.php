<?php
// S-194 package F behaviour teeth: calls to functions of another file (bound once per call site), by-reference builtins, array_pop.
namespace Lib { function site($s) { return strtoupper_ish($s); } function site_h($s) { return helper($s); } function site_late($s) { return late_fn($s); } }
namespace {
function before_load() { try { return g_add(1, 2); } catch (\Error $e) { return "U:" . $e->getMessage(); } }
echo before_load(), "\n";
require __DIR__ . '/s194-f-lib.php';
echo before_load(), "|", g_add(1), "|", G_ADD(2, 3), "|", \g_add(4, 5), "|", g_var(1, 2, 3), "|", g_rec(10), "\n";
$t = "ab"; $arr2 = []; $arr = ['k' => 'x']; $o = new stdClass; $o->p = 'p';
echo g_ref($t, "cd"), ":", $t, "|", g_ref($arr['k'], "y"), ":", $arr['k'], "|", g_ref($arr['new'], "z"), ":", $arr['new'], "|", g_ref($o->p, "q"), ":", $o->p, "|", g_ref($undef, "u"), ":", $undef, "\n";
echo Lib\add(1, 2), "|", \Lib\add(3, 4), "|", Lib\site("a"), "|", implode(",", Lib\ref($arr2)), "|", implode(",", $arr2), "\n";
// namespace fallback to a GLOBAL user function, then a late namespaced declaration: a site that already ran keeps its binding
function helper($s) { return "gh:" . $s; }
echo Lib\site_h("a"), "|", Lib\caller("b"), "\n";
eval('namespace Lib; function helper($s) { return "lh:" . $s; }');
echo Lib\site_h("a"), "|", Lib\caller("b"), "|", Lib\helper("c"), "\n";
// a site that failed (undefined) is NOT bound: it succeeds once the function exists
for ($i = 0; $i < 3; $i++) { try { echo Lib\site_late($i), ";"; } catch (\Error $e) { echo "U:", $e->getMessage(), ";"; eval('function late_fn($s) { return "late:" . $s; }'); } } echo "\n";
for ($i = 0, $s = 0; $i < 200; $i++) { $s = g_add($s, $i) + Lib\add(1, 1); } echo $s, "\n";
// by-reference builtins
$a = [3, 1, 2]; echo array_push($a, 9, 8), "|", array_pop($a), "|", array_pop($a), "|", count($a), "|"; sort($a); echo implode(",", $a), "|"; $a[] = 7; echo implode(",", array_keys($a)), "\n";
$e = []; var_dump(array_pop($e)); $m = ['x' => 1, 5 => 2, 'y' => 3]; echo array_pop($m), "|", array_pop($m), "|"; $m[] = 4; echo implode(",", array_keys($m)), "|", array_shift($m), "|", array_unshift($m, 0), "\n";
$o->list = [1]; array_push($o->list, 2); $n = ['q' => [5]]; array_push($n['q'], 6); echo implode(",", $o->list), "|", implode(",", $n['q']), "|", array_pop($n['q']), "|", end($a), "|", reset($a), "|", key($a), "\n";
try { $str = "x"; array_pop($str); } catch (\TypeError $x) { echo "T:", $x->getMessage(), "\n"; }
echo "pre|"; $r = [2, 1]; rsort($r); echo implode(",", $r), "|", array_push($r, strlen("abc")), "|", shuffle($e) ? "t" : "f", "\n";
ob_start(); $z = [1]; echo array_pop($z); $buf = ob_get_clean(); echo "ob:", $buf, "\n";
$big = range(1, 50); $c = 0; while (($v = array_pop($big)) !== null) { $c += $v; } echo $c, "|", count($big), "\n";
}
