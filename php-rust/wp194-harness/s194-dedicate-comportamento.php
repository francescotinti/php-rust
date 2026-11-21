<?php
// S-194 dedicated strlen/count ops: behaviour teeth. Gate = output identical to the pin (reference recorded before the code).
require __DIR__ . '/s194-dedicate-strict.php'; require __DIR__ . '/s194-dedicate-ns.php';
class S { function __toString() { echo "[ts]"; return "abcde"; } }
class N { }
class K implements Countable { function count(): int { echo "[cnt]"; return 7; } }
function byref(&$x) { return strlen($x) . ':' . count([$x]); }
function arr() { echo "[arr]"; return [1, 2, 3]; }
$s = "hello"; $q = &$s; $a = [1, [2, 3], 4]; $b = &$a; $e = []; $bin = "a\0b\xff";
echo strlen($s), "|", strlen($q), "|", strlen(""), "|", strlen($bin), "|", strlen("lit"), "|", strlen($s . "x"), "|", byref($s), "\n";
echo strlen(12345), "|", strlen(1.5), "|", strlen(true), "|", strlen(false), "|", strlen(new S), "\n";
echo "pre|", strlen(null), "|post\n";
try { echo strlen([]); } catch (TypeError $x) { echo "T:", $x->getMessage(), "\n"; }
try { echo strlen(new N); } catch (Throwable $x) { echo get_class($x), ":", $x->getMessage(), "\n"; }
echo count($a), "|", count($b), "|", count($e), "|", sizeof($a), "|", count($a, COUNT_RECURSIVE), "|", count($a, 1), "|", count(arr()), "|", count(new K), "\n";
foreach ([null, 5, "str", 1.5, new N] as $v) { try { echo count($v), "\n"; } catch (TypeError $x) { echo "T:", $x->getMessage(), "\n"; } }
echo "u1|", strlen($undef1), "|u2\n";
try { echo "c1|", count($undef2), "\n"; } catch (TypeError $x) { echo "T:", $x->getMessage(), "\n"; }
echo "w|", strlen($s) + count($a) + strlen($undef3), "|", $undef4, strlen("zz"), "\n";
ob_start(); echo strlen($s), count($a); $buf = ob_get_clean(); echo "ob:", $buf, "\n";
$f = 'strlen'; $g = 'count'; echo $f($s), "|", $g($a), "|", call_user_func('strlen', $s), "|", implode(",", array_map('strlen', ["a", "bb"])), "|", implode(",", array_map('count', [[1], []])), "\n";
echo strlen(...[$s]), "|", count(...[$a]), "|", strlen(string: $s), "|", count(value: $a), "\n";
strict_probe(); \Shadow\probe(); \Plain\probe();
$t = 0; for ($i = 0; $i < 1000; $i++) { $t += strlen($s) + count($a); if ($i == 500) { $s .= "!"; $a[] = 9; } } echo $t, "\n";
$m = ['x' => 'abc']; $o = new stdClass; $o->p = [1, 2]; echo strlen($m['x']), "|", count($o->p), "|", strlen($m['x'][0]), "|", count($m), "\n";
