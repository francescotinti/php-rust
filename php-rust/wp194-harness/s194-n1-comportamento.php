<?php
// S-194 N1 behaviour tooth: substring search on short and long haystacks, object ids reused. Expected = the oracle's output.
function b($v) { return var_export($v, true); }
$h = str_repeat("ab", 100) . "needle" . str_repeat("cd", 100);
$long = str_repeat("xy", 200) . "needle" . str_repeat("zz", 50);
foreach ([["hello world", "o"], ["hello world", "world"], ["hello", "hello!"], ["hello", "l"], ["", "a"], ["aaa", "aa"], ["abcabcabd", "abd"], ["abc", "c"], ["abc", "abc"],
          [$h, "needle"], [$h, "dc"], [$h, "x"], [$long, "needle"], [$long, "zzz"], [$long, "q"], ["a\0b", "\0b"], ["été", "t"], ["aXbXc", "X"]] as [$s, $n]) {
    echo b(strpos($s, $n)), ' ', b(str_contains($s, $n)), ' ', b(strstr($s, $n) === false ? false : strlen(strstr($s, $n))), ' ', b(strrpos($s, $n)), ' ', b(substr_count($s, $n)), ' ', b(stripos($s, strtoupper($n))), ' ', b(str_starts_with($s, $n)), ' ', b(str_ends_with($s, $n)), "\n";
}
echo b(strpos("abcabc", "c", 3)), ' ', b(strpos("abcabc", "c", -1)), ' ', b(strpos("abcabc", "a", -3)), ' ', b(str_contains("abc", "")), ' ', b(strpos("abc", "")), ' ', b(strstr("user@example.com", "@", true)), ' ', b(explode(",", "a,b,,c")), "\n";
try { strpos("abc", "b", 9); } catch (ValueError $e) { echo get_class($e), ': ', $e->getMessage(), "\n"; }
class E1 extends Exception {} class P1 { public $v = 1; function __destruct() { echo "d"; } }
for ($i = 0; $i < 3; $i++) { $p = new P1; $e = new E1("m$i"); echo ' ', $e->getLine(), ':', basename($e->getFile()) === basename(__FILE__) ? 'f' : 'x', ':', count($e->getTrace()), ' ', spl_object_id($p) > 0 ? 'id' : 'no'; $p = null; }
echo "\n";
function mk() { return new E1("in"); } $x = mk(); echo $x->getLine(), ' ', $x->getTrace()[0]['function'], "\n";
