//! S-194 C3b (2026-11-24): a method call site caches a private / protected resolution with the
//! calling scope in the key. Behaviour tooth of the step (`wp194-harness/s194-c3b-comportamento.php`):
//! private and protected methods through `$this`, private homonyms in parent and child, the same
//! site run under alternating `Closure::bind` scopes, `__call` fallbacks, mixed receiver classes.
//! The expected string is the oracle's stdout (php 8.5.7 CLI, `php -r`), byte for byte.
use std::process::{Command, Stdio};

#[test]
fn a_scope_dependent_method_resolution_is_cached_per_calling_scope() {
    let script = r#"// S-194 C3b behaviour tooth: method calls whose resolution depends on the calling scope (private / protected methods, private
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
"#;
    let expected = r#"A::f(0) A::f(0) A::A::g(0) A::A::g(0) A::sp A::h T::tp(0)
A::f(0) A::f(0) B::g(0) B::g(0) A::sp A::h T::tp(0)
A::f(0) A::f(0) C::A::g(0) C::A::g(0) A::sp C::h T::tp(0)
A::f(0) A::f(0) A::A::g(0) A::A::g(0) A::sp A::h T::tp(0)
A::f(0) A::f(0) B::g(0) B::g(0) A::sp A::h T::tp(0)
B::f(0) A::f(1)/A::A::g(2) A::f(1)/B::g(2) A::f(1)/C::A::g(2) A::f(1)/A::A::g(2)
M::p(0) __call(p,0) __call(zz,1,2)
Error: Call to private method A::f() from global scope | Error: Call to protected method A::g() from global scope
Error: Call to private method B::f() from global scope | Error: Call to protected method B::g() from global scope
C::f(9) | Error: Call to protected method A::g() from global scope
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope B ; B::f(7)+B::g(8) ; C::f(7)+C::A::g(8) ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
Error: Call to private method A::f() from scope C ; Error: Call to private method B::f() from scope C ; C::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope N ; Error: Call to private method B::f() from scope N ; Error: Call to protected method A::g() from scope N ; 
N::q Error: Call to protected method N::q() from scope A N::q
A::f(1) A::f(0) A::A::g(1) A::A::g(0) A::sp A::h T::tp(1)
A::f(1) A::f(0) B::g(1) B::g(0) A::sp A::h T::tp(1)
A::f(1) A::f(0) C::A::g(1) C::A::g(0) A::sp C::h T::tp(1)
A::f(1) A::f(0) A::A::g(1) A::A::g(0) A::sp A::h T::tp(1)
A::f(1) A::f(0) B::g(1) B::g(0) A::sp A::h T::tp(1)
B::f(1) A::f(1)/A::A::g(2) A::f(1)/B::g(2) A::f(1)/C::A::g(2) A::f(1)/A::A::g(2)
M::p(1) __call(p,1) __call(zz,1,2)
Error: Call to private method A::f() from global scope | Error: Call to protected method A::g() from global scope
Error: Call to private method B::f() from global scope | Error: Call to protected method B::g() from global scope
C::f(9) | Error: Call to protected method A::g() from global scope
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope B ; B::f(7)+B::g(8) ; C::f(7)+C::A::g(8) ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
Error: Call to private method A::f() from scope C ; Error: Call to private method B::f() from scope C ; C::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope N ; Error: Call to private method B::f() from scope N ; Error: Call to protected method A::g() from scope N ; 
N::q Error: Call to protected method N::q() from scope A N::q
A::f(2) A::f(0) A::A::g(2) A::A::g(0) A::sp A::h T::tp(2)
A::f(2) A::f(0) B::g(2) B::g(0) A::sp A::h T::tp(2)
A::f(2) A::f(0) C::A::g(2) C::A::g(0) A::sp C::h T::tp(2)
A::f(2) A::f(0) A::A::g(2) A::A::g(0) A::sp A::h T::tp(2)
A::f(2) A::f(0) B::g(2) B::g(0) A::sp A::h T::tp(2)
B::f(2) A::f(1)/A::A::g(2) A::f(1)/B::g(2) A::f(1)/C::A::g(2) A::f(1)/A::A::g(2)
M::p(2) __call(p,2) __call(zz,1,2)
Error: Call to private method A::f() from global scope | Error: Call to protected method A::g() from global scope
Error: Call to private method B::f() from global scope | Error: Call to protected method B::g() from global scope
C::f(9) | Error: Call to protected method A::g() from global scope
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope B ; B::f(7)+B::g(8) ; C::f(7)+C::A::g(8) ; 
A::f(7)+A::A::g(8) ; A::f(7)+B::g(8) ; A::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from global scope ; Error: Call to private method B::f() from global scope ; Error: Call to protected method A::g() from global scope ; 
Error: Call to private method A::f() from scope C ; Error: Call to private method B::f() from scope C ; C::f(7)+C::A::g(8) ; 
Error: Call to private method A::f() from scope N ; Error: Call to private method B::f() from scope N ; Error: Call to protected method A::g() from scope N ; 
N::q Error: Call to protected method N::q() from scope A N::q"#;
    let out = Command::new(env!("CARGO_BIN_EXE_phpr"))
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .arg("-r")
        .arg(script)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .expect("spawn phpr");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end(), expected);
}
