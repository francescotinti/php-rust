<?php
// S-194 package N behaviour teeth: unqualified builtin calls inside a namespace (CallNsFallback). Run on the oracle and on the pin BEFORE the code.
namespace App;
class S { function __toString() { echo "[ts]"; return " abc "; } }
class K implements \Countable { function count(): int { echo "[cnt]"; return 7; } }
class Dbg { public $a = 1; function __debugInfo() { return ['x' => 2]; } }
function site($v) { return strlen($v); }
function site2($v) { return strtoupper($v); }
function byref(&$x) { return strlen($x) . ':' . strtoupper($x) . ':' . count([$x]); }
$d = new Dbg; $s = "hello"; $q = &$s; $a = [3, 1, 2]; // $d first: its object id must not depend on the lifetime of the temporaries below (pre-existing divergence, out/n-id-sonda.out)
echo strlen($s), "|", strlen($q), "|", count($a), "|", abs(-4), "|", str_repeat("ab", 2), "|", substr($s, 1, 3), "|", byref($s), "|", pi() > 3, "|", max(1, 2, 3, 4, 5, 6), "|", implode(",", $a), "\n";
echo strlen(new S), "|", trim(new S), "|", count(new K), "|", sizeof($a), "|", count($a, COUNT_RECURSIVE), "|", str_replace("a", "b", new S), "\n";
var_dump($d); var_dump(strlen("x"));
echo "before|"; printf("%s|%d\n", [1], 5);
try { echo strlen([]); } catch (\TypeError $e) { echo "T:", $e->getMessage(), "\n"; }
try { echo count(5); } catch (\TypeError $e) { echo "T:", $e->getMessage(), "\n"; }
try { echo intdiv(1, 0); } catch (\DivisionByZeroError $e) { echo "E:", $e->getMessage(), "\n"; }
try { echo nosuchfn(1); } catch (\Error $e) { echo "U:", $e->getMessage(), "\n"; }
ob_start(); print_r([1, 2]); echo strrev("cba"); $buf = ob_get_clean(); echo strlen($buf), ":", $buf, "\n";
echo strlen(...[$s]), "|", count(...[$a]), "|", strlen(string: $s), "|", \strlen($s), "|", namespace\site($s), "\n";
// late shadowing: a namespaced function declared AFTER a call site has already run
echo site("abc"), "|", site2("abc"), "\n";
eval('namespace App; function strlen($x) { return "shadow:" . $x; }');
echo site("abc"), "|", strlen("abc"), "|", site2("abc"), "\n";
if (true) { function strtoupper($x) { return "late:" . $x; } }
echo site2("abc"), "|", strtoupper("abc"), "\n";
for ($i = 0, $t = 0; $i < 300; $i++) { $t += count($a) + abs(-$i); if ($i == 100) { eval('namespace App; function abs($x) { return 1000; }'); } } echo $t, "|", abs(-5), "\n";
