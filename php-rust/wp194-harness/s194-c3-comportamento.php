<?php
// S-194 C3a behaviour tooth: calls with fewer arguments than parameters (default values), functions and methods,
// every call made twice in a loop so the second one runs on a warm call-site cache. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
class K { const D = 7; }
class P { function __construct(public int $v = 3) {} }
function d1($a, $b = 1) { return $a + $b; }
function d2($a, $b = K::D, $c = [1, 2]) { return $a . '|' . $b . '|' . count($c); }
function d3(int $a, ?string $s = null, float $f = 1) { return var_export([$a, $s, $f], true); }
function d4($a, $b = 2, ...$rest) { return $a . ',' . $b . ',' . count($rest); }
function d5(&$r, $inc = 5) { $r += $inc; }
function d6($a, $b = 10) { return func_num_args() . ':' . implode(',', func_get_args()); }
function d7($a, P $p = new P) { return $a + $p->v; }
function d8(int $a, int $b = 4): int { return $a * $b; }
function d9($a, $b) { return $a . $b; }
function d10($a = 'x', $b = 'y', $c = 'z') { return $a . $b . $c; }
class A {
  function m($a, $b = 'A') { return static::class . ":$a:$b"; }
  function t(int $a, int $b = 2): int { return $a + $b; }
  function n($a, $b = null, $c = K::D) { return var_export($b, true) . "/$c/" . func_num_args(); }
  static function s($a, $b = 'sa') { return "$a$b"; }
  function need($a, $b) { return 1; }
  function self2($x) { return $this->m($x) . '+' . $this->t($x); }
}
class B extends A { function m($a, $b = 'B', $c = 'extra') { return "B:$a:$b:$c"; } }
$a = new A; $b = new B; $r = 0;
for ($i = 0; $i < 2; $i++) {
  echo d1(1), ' ', d1(1, 5), "\n";
  echo d2('p'), ' ', d2('p', 'q'), "\n";
  echo d3(1), ' ', d3('2', 's'), "\n";
  echo d4(1), ' ', d4(1, 2, 3, 4), "\n";
  d5($r); d5($r, 1); echo $r, "\n";
  echo d6(1), ' ', d6(1, 2), ' ', d6(1, 2, 3), "\n";
  echo d7(1), ' ', d7(1, new P(9)), "\n";
  echo d8(3), ' ', d8('3'), ' ', d8(3, 5), "\n";
  echo d10(), d10('1'), d10('1', '2'), d10(c: 'C'), d10('1', c: 'C'), "\n";
  foreach ([$a, $b, $a] as $o) { echo $o->m(1), ' ', $o->m(1, 2), ' ', $o->t(5), ' ', $o->n(0), ' ', $o->n(0, 1), "\n"; }
  echo A::s('q'), B::s('q', 'r'), ' ', $a->self2(4), ' ', $b->self2(4), "\n";
  echo $a->m($undef1), ' ', $a->t(...[6]), ' ', $a->m(b: 'nb', a: 'na'), "\n";
  try { echo d9(1), "\n"; } catch (ArgumentCountError $e) { echo get_class($e), ': ', $e->getMessage(), ' @', $e->getLine(), "\n"; }
  try { echo $a->need(1), "\n"; } catch (ArgumentCountError $e) { echo get_class($e), ': ', $e->getMessage(), ' @', $e->getLine(), "\n"; }
  try { echo d8('x'), "\n"; } catch (TypeError $e) { echo get_class($e), ': ', $e->getMessage(), ' @', $e->getLine(), "\n"; }
  try { echo $a->t('x'), "\n"; } catch (TypeError $e) { echo get_class($e), ': ', $e->getMessage(), ' @', $e->getLine(), "\n"; }
  try { echo d3(), "\n"; } catch (ArgumentCountError $e) { echo $e->getMessage(), "\n"; }
  echo implode(',', array_map('d1', [1, 2])), ' ', call_user_func('d1', 4), ' ', call_user_func([$b, 'm'], 7), "\n";
}
$c = function ($a, $b = 'cl') { return $a . $b; }; echo $c(1), $c(1, 2), "\n";
echo (new ReflectionFunction('d2'))->getNumberOfRequiredParameters(), "\n";
echo d9(1);
