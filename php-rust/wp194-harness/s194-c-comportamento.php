<?php
// S-194 package C1 behaviour teeth: declared parameter and return types on user calls (general call path, typed return). Run on the oracle and on the pin BEFORE the code.
namespace App;
interface I {} interface J extends I {}
class A implements J { public $n = 1; function __destruct() { echo "[~A{$this->n}]"; } }
class B extends A {} class Z {} class S { function __toString() { return "str"; } }
\class_alias(A::class, 'App\AA');
function t($f) { try { $r = $f(); echo \is_object($r) ? \get_class($r) : \var_export($r, true), "\n"; } catch (\Throwable $e) { echo \get_class($e), ": ", $e->getMessage(), "\n"; } }
// parameters
function pi_(int $a) { return $a; } function pf(float $a) { return $a; } function ps(string $a) { return $a; } function pb(bool $a) { return $a; }
function pn(?int $a) { return $a; } function pa(array $a) { return \count($a); } function pc(A $a) { return $a->n; } function pIf(I $a) { return 1; }
function plc(a $x) { return 2; } function pq(\App\AA $x) { return 3; } function pu(int|string $a) { return $a; } function pun(A|Z|null $a) { return $a === null ? 'null' : \get_class($a); }
function pcl(\Closure $c) { return $c(); } function pcb(callable $c) { return $c(); } function pit(iterable $i) { return 4; } function po(object $o) { return \get_class($o); }
function pd(int $a = 5, ?A $b = null, string ...$rest) { return $a . ':' . ($b ? 'A' : '-') . ':' . \implode(',', $rest); }
function pr(int &$a) { $a++; return $a; } function pself(A $a, A $b) { return $a === $b; }
t(fn() => pi_(5)); t(fn() => pi_("5")); t(fn() => pi_(5.0)); t(fn() => pi_("x")); t(fn() => pi_(null)); t(fn() => pi_(true)); t(fn() => pi_(new Z));
t(fn() => pf(5)); t(fn() => pf("1.5")); t(fn() => ps(5)); t(fn() => ps(new S)); t(fn() => ps(new Z)); t(fn() => pb(0)); t(fn() => pb("a"));
t(fn() => pn(null)); t(fn() => pn(3)); t(fn() => pa([1, 2])); t(fn() => pa(1));
t(fn() => pc(new A)); t(fn() => pc(new B)); t(fn() => pc(new Z)); t(fn() => pc(null)); t(fn() => pc(1)); t(fn() => pIf(new B)); t(fn() => pIf(new Z));
t(fn() => plc(new A)); t(fn() => pq(new B)); t(fn() => pq(new Z)); t(fn() => pu(1.0)); t(fn() => pu("s")); t(fn() => pu([])); t(fn() => pun(null)); t(fn() => pun(new Z)); t(fn() => pun(new S));
t(fn() => pcl(fn() => 9)); t(fn() => pcl('strlen')); t(fn() => pcb('App\pit')); t(fn() => pcb('nosuch')); t(fn() => pit([1])); t(fn() => pit(new \ArrayIterator([]))); t(fn() => pit(1)); t(fn() => po(new Z)); t(fn() => po(fn() => 1)); t(fn() => po("x"));
t(fn() => pd()); t(fn() => pd("7")); t(fn() => pd(1, new A, "a", "b")); t(fn() => pd(1, null, "a", 2)); t(fn() => pd(1, null, [])); t(fn() => pd(b: new B)); t(fn() => pd(1, new Z));
$v = 1; t(fn() => pr($v)); echo $v, "\n"; $w = "x"; t(fn() => pr($w)); $o = new A; $q = &$o; t(fn() => pc($q)); t(fn() => pself($o, $q));
// returns
function ri($v): int { return $v; } function rf($v): float { return $v; } function rs($v): string { return $v; } function rb($v): bool { return $v; }
function rn($v): ?int { return $v; } function ra($v): array { return $v; } function rc($v): A { return $v; } function rIf($v): I { return $v; } function rnc($v): ?A { return $v; }
function ru($v): int|string { return $v; } function rv(): void { return; } function rit($v): iterable { return $v; } function ro($v): object { return $v; } function rcb($v): callable { return $v; }
function rloc(): A { $a = new A; $a->n = 7; $tmp = new A; $tmp->n = 8; return $a; } function &rref(array &$a): int { return $a[0]; }
function rgen(): \Generator { yield 1; return 2; } function rfe(array $a): int { foreach ($a as $x) { if ($x > 1) { return $x; } } return 0; }
function rnv(): never { throw new \RuntimeException("never"); }
class R { function self_(): self { return $this; } function st(): static { return $this; } function bad(): static { return new R; } function i(int $a): int { return $a + 1; } static function si($v): int { return $v; }
  function mixed_($v): mixed { return $v; } function __toString(): string { return "R!"; } }
class R2 extends R {}
t(fn() => ri(5)); t(fn() => ri("5")); t(fn() => ri(5.0)); t(fn() => ri(5.5)); t(fn() => ri("x")); t(fn() => ri(null)); t(fn() => ri(new Z)); t(fn() => rf(5)); t(fn() => rf("2")); t(fn() => rs(5)); t(fn() => rs(new S)); t(fn() => rb(1)); t(fn() => rb([]));
t(fn() => rn(null)); t(fn() => rn(2)); t(fn() => rn("q")); t(fn() => ra([1])); t(fn() => ra(1)); t(fn() => rc(new A)); t(fn() => rc(new B)); t(fn() => rc(new Z)); t(fn() => rc(null)); t(fn() => rIf(new B)); t(fn() => rIf(new Z)); t(fn() => rnc(null)); t(fn() => rnc(new B)); t(fn() => rnc(3));
t(fn() => ru(1)); t(fn() => ru("a")); t(fn() => ru(1.5)); t(fn() => ru([])); t(fn() => rv()); t(fn() => rit([1])); t(fn() => rit(2)); t(fn() => ro(new Z)); t(fn() => ro(1)); t(fn() => rcb('strlen')); t(fn() => rcb('nosuch'));
echo "loc:"; $k = rloc(); echo "|", $k->n, "\n"; unset($k); echo "\n"; $arr = [4]; $rr = &rref($arr); $rr = 9; echo $arr[0], "\n";
foreach (rgen() as $g) { echo $g; } echo "\n"; t(fn() => rfe([1, 2, 3])); t(fn() => rfe([])); t(fn() => rnv());
t(fn() => (new R)->self_()); t(fn() => (new R2)->st()); t(fn() => (new R2)->bad()); t(fn() => (new R)->i("4")); t(fn() => (new R)->i("z")); t(fn() => R::si("8")); t(fn() => R::si([])); t(fn() => (new R)->mixed_(null)); echo \strlen(new R), "\n";
$ref = 5; $al = &$ref; t(fn() => ri($al)); $os = new A; $os->n = 3; $or = &$os; t(fn() => rc($or)->n);
echo \array_sum(\array_map('App\ri', ["1", 2, 3.0])), "|", \implode(",", \array_map(fn(int $x): int => $x * 2, [1, "2"])), "\n";
require __DIR__ . '/s194-c-comportamento-strict.php';
echo "end\n";
