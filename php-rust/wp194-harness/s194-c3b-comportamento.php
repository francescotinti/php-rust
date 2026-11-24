<?php
// S-194 C3b behaviour tooth: method calls whose resolution depends on the calling scope (private / protected methods, private
// homonyms, Closure::bind scopes, __call fallbacks), every site run several times so it is warm. Expected = the oracle's output.
// Generato con assistenza AI (Claude Fable 5.2).
trait T { private function tp($a) { return "T::tp($a)"; } function viaT($a) { return $this->tp($a); } }
class A {
  use T;
  private function f($a = 0) { return "A::f($a)"; }
  protected function g($a = 0) { return static::class . "::A::g($a)"; }
  public function h() { return "A::h"; }
  private static function sp() { return "A::sp"; }
  function callF($a) { return $this->f($a); }
  function callF0() { return $this->f(); }
  function callG($a) { return $this->g($a); }
  function callG0() { return $this->g(); }
  function callOn($o) { return $o->f(1) . '/' . $o->g(2); }
  function callSp() { return $this->sp(); }
  function callH() { return $this->h(); }
}
class B extends A {
  private function f($a = 0) { return "B::f($a)"; }
  protected function g($a = 0) { return "B::g($a)"; }
  function callFB($a) { return $this->f($a); }
}
class C extends A { public function f($a = 0) { return "C::f($a)"; } public function h() { return "C::h"; } }
class M { private function p($a) { return "M::p($a)"; } function __call($n, $a) { return "__call($n," . implode(',', $a) . ")"; } function in($a) { return $this->p($a); } }
class N { protected function q() { return "N::q"; } }
function try_($f) { try { return $f(); } catch (Error $e) { return get_class($e) . ': ' . $e->getMessage(); } }
$a = new A; $b = new B; $c = new C; $m = new M; $n = new N;
for ($i = 0; $i < 3; $i++) {
  foreach ([$a, $b, $c, $a, $b] as $o) {
    echo $o->callF($i), ' ', $o->callF0(), ' ', $o->callG($i), ' ', $o->callG0(), ' ', $o->callSp(), ' ', $o->callH(), ' ', $o->viaT($i), "\n";
  }
  echo $b->callFB($i), ' ', $a->callOn($a), ' ', $a->callOn($b), ' ', $a->callOn($c), ' ', $b->callOn($a), "\n";
  echo $m->in($i), ' ', $m->p($i), ' ', $m->zz(1, 2), "\n";
  foreach ([$a, $b, $c] as $o) { echo try_(fn() => $o->f(9)), ' | ', try_(fn() => $o->g(9)), "\n"; }
  $site = function ($o) { return $o->f(7) . '+' . $o->g(8); };
  foreach ([[null, null], [$a, A::class], [$b, B::class], [$a, A::class], [null, null], [$c, C::class], [$n, N::class]] as [$bindThis, $scope]) {
    $g = $scope === null ? $site : Closure::bind($site, $bindThis, $scope);
    foreach ([$a, $b, $c] as $o) { echo try_(fn() => $g($o)), ' ; '; }
    echo "\n";
  }
  $q = function () { return $this->q(); };
  echo try_(fn() => Closure::bind($q, $n, N::class)()), ' ', try_(fn() => Closure::bind($q, $n, A::class)()), ' ', try_(fn() => Closure::bind($q, $n, N::class)()), "\n";
}
