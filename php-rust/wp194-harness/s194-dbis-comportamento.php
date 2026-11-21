<?php
// S-194 D-bis behaviour teeth against the oracle: every special path that the direct builtin path must hand over.
class S { function __toString() { echo "[ts]"; return " abc "; } }
class K implements Countable { function count(): int { echo "[cnt]"; return 7; } }
class Dbg { public $a = 1; function __debugInfo() { return ['x' => 2]; } }
function byref(&$x) { return strlen($x) . ':' . strtoupper($x); }
$o = new S; $r = "ref"; $q = &$r; $arr = [3, 1, 2];
echo strlen($o), "|", trim($o), "|", strlen($q), "|", byref($r), "|", count(new K), "|", count($arr), "|", count($arr, COUNT_RECURSIVE), "\n";
var_dump(new Dbg); var_dump(strlen("x"), abs(-3), pi() > 3);
echo implode(",", [new S, 1]), "|", str_replace("a", "b", new S), "|", str_contains(new S, "b") ? "y" : "n", "\n";
echo "before|"; printf("%s|%d\n", [1], 5);
echo "x", strtoupper("ok"), strlen(null), "\n";
echo substr("abc", 5), "|", str_repeat("-", 3), "|", intdiv(7, 2), "\n";
try { echo intdiv(1, 0); } catch (DivisionByZeroError $e) { echo "E:", $e->getMessage(), "\n"; }
try { echo strlen([]); } catch (TypeError $e) { echo "T:", $e->getMessage(), "\n"; }
try { echo count(new S); } catch (TypeError $e) { echo "T:", $e->getMessage(), "\n"; }
$f = 'strlen'; echo $f($o), "|", call_user_func('trim', $o), "|", implode("", array_map('strtoupper', ["a", $q])), "\n";
ob_start(); print_r([1, new Dbg]); echo strrev("cba"); $buf = ob_get_clean(); echo strlen($buf), ":", $buf, "\n";
fwrite(STDOUT, "direct\n"); echo number_format(1234.5, 1), "|", ucfirst("z"), "|", max(1, 2), "|", json_encode(["a" => abs(-1)]), "\n";
$u = "hello"; $s = 0; for ($i = 0; $i < 5; $i++) { $s += strlen($u) + count($arr) + abs(-$i); if ($i == 2) { echo $undefined_var; } } echo $s, "\n";
