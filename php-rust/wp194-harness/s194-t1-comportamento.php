<?php
// S-194 T1 behaviour tooth: class type checks through the success memo. Expected = the oracle's output.
interface I {} interface J extends I {}
class A implements I { function me(): static { return $this; } function same(self $o): self { return $o; } }
class B extends A implements J {}
class C {}
function fa(A $o) { return get_class($o); } function fi(I $o): I { return $o; } function fj(?J $o) { return $o === null ? 'null' : get_class($o); }
function fb(b $o): a { return $o; }                       // case-insensitive names
function late(Later $o) { return get_class($o); }
function t($f) { try { $r = $f(); return is_object($r) ? get_class($r) : $r; } catch (Throwable $e) { return get_class($e) . ': ' . preg_replace('/, called in.*/', '', $e->getMessage()); } }
for ($i = 0; $i < 3; $i++) {
    $a = new A; $b = new B; $c = new C;
    echo t(fn() => fa($a)), ' ', t(fn() => fa($b)), ' ', t(fn() => fa($c)), ' | ', t(fn() => fi($a)), ' ', t(fn() => fi($b)), ' ', t(fn() => fi($c)), ' | ',
         t(fn() => fj($b)), ' ', t(fn() => fj(null)), ' ', t(fn() => fj($a)), ' | ', t(fn() => fb($b)), ' ', t(fn() => fb($a)), ' | ',
         t(fn() => $b->me()), ' ', t(fn() => $a->same($b)), ' ', t(fn() => $b->same($a)), "\n"; // `self` as a parameter type is not enforced by phpr (pre-existing divergence, named aperture): no wrong-class row here
    echo t(fn() => late($a)), ' | ';
    if ($i == 1) { eval('class Later extends A {}'); class_alias('C', 'Alias'); }
    echo $i >= 1 ? t(fn() => late(new Later)) . ' ' . t(fn() => fa(new Later)) . ' ' . t(function () { $f = function (Alias $x) { return get_class($x); }; return $f(new C); }) : '-', "\n";
}
$objs = []; for ($k = 0; $k < 40; $k++) { eval("class K$k extends A {} class U$k {}"); $objs[] = "K$k"; }
$n = 0; foreach ($objs as $cn) { $n += fa(new $cn) === $cn ? 1 : 0; $n += fi(new $cn) instanceof I ? 1 : 0; } echo $n, ' ', t(fn() => fa(new U7)), "\n";
$f = function (Closure $c, iterable $it, callable $cb, object $o) { return 'ok'; }; echo t(fn() => $f(fn() => 1, [1], 'strlen', $a)), ' ', t(fn() => $f($a, [1], 'strlen', $a)), "\n";
