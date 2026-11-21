<?php
class C { const K = 7; public $x = 0; public $y = 1; public static $sp = 3;
  function m($a) { return $a + 1; } static function sm($a) { return $a + 1; } function inc() { $this->x++; } }
class D { function __construct(public int $a) {} }
function f0() {} function f2($a, $b) { return $a + $b; } function ft(int $a, int $b): int { return $a + $b; }
function run($n) {
  $s = 0; $x = 0; $t = ''; $f = 0.0; $o = new C; $u = "hello"; $w = str_repeat("abcdef", 5);
  $a = range(0, 1023); $h = ['alpha' => 1, 'beta' => 2, 'gamma' => 3]; $k = []; for ($j = 0; $j < 1024; $j++) { $k["k$j"] = $j; }
  $c = function ($a) { return $a + 1; }; $r = range(0, 99); $subj = "the quick brown fox jumps over the lazy dog 12345";
  for ($i = 0; $i < $n; $i++) { $s = C::sm($s); }
  echo $s, "|", strlen((string)$t), "|", $x, "\n";
}
run((int)$argv[1]);
