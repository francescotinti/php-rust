<?php
// s195-fetta-comportamento.php — S-195, register slice (criterion s195-criterio-fetta-registri.md p.7): the semantics that the
// direct return, the call result delivered to a slot and the last argument read from a slot must not move. Expected = oracle stdout
// (-d log_errors=0 -d display_errors=1). Generato con assistenza AI (Claude Fable 5.2).
class D { public function __construct(public string $n) {} public function __destruct() { echo "dtor {$this->n}\n"; } }
function r_const() { return 7; }
function r_null() { return null; }
function r_var($a) { $b = $a; return $b; }
function r_undef() { return $nope; }
function r_param($a) { return $a; }
function &r_ref(array &$arr) { return $arr[0]; }
function r_refvar() { $x = 1; $y = &$x; $y = 9; return $x; }
function r_typed_int($a): int { return $a; }
function r_typed_str(): string { return 5; }
function r_void(): void { return; }
function r_finally() { try { return 1; } finally { echo "finally\n"; } }
function r_finally_override() { try { return 1; } finally { return 2; } }
function r_dtor_order() { $d = new D("local"); $v = "value"; return $v; }
function r_dtor_self() { $d = new D("returned"); return $d; }
function r_static() { static $n = 0; $n++; return $n; }
function r_gen() { $x = yield 1; return $x; }
function r_global() { global $g; return $g; }
class C {
    public $p = 3; private static $s = 4;
    public function getP() { return $this->p; }
    public function getThis() { return $this; }
    public function k() { return 11; }
    public static function st() { $v = self::$s; return $v; }
    public function arg($a) { return $a; }
}
echo "A1 ", r_const(), " ", var_export(r_null(), true), " ", r_var("v"), "\n";
echo "A2 "; var_dump(r_undef());
echo "A3 ", r_param([1, 2])[1], "\n";
$arr = [1, 2]; $q = &r_ref($arr); $q = 50; echo "A4 ", $arr[0], "\n";
echo "A5 ", r_refvar(), "\n";
echo "A6 "; var_dump(r_typed_int("12"));
try { r_typed_int("x"); } catch (TypeError $e) { echo "A7 ", $e->getMessage(), "\n"; }
echo "A8 "; var_dump(r_typed_str());
echo "A9 "; var_dump(r_void());
echo "A10 ", r_finally(), " ", r_finally_override(), "\n";
echo "A11 start\n"; $v = r_dtor_order(); echo "A11 got $v\n";
echo "A12 start\n"; $o = r_dtor_self(); echo "A12 got {$o->n}\n"; $o = null; echo "A12 end\n";
echo "A13 start\n"; r_dtor_self(); echo "A13 end\n";
echo "A14 ", r_static(), r_static(), r_static(), "\n";
$gen = r_gen(); $gen->current(); $gen->send("sent"); echo "A15 ", $gen->getReturn(), "\n";
$g = "glob"; echo "A16 ", r_global(), "\n";
$c = new C;
echo "A17 ", $c->getP(), " ", $c->k(), " ", C::st(), " ", get_class($c->getThis()), "\n";
// B: call result delivered to a slot
$x = r_const(); echo "B1 $x\n";
$x = new D("old"); echo "B2 before\n"; $x = r_const(); echo "B2 after $x\n";
$t = 1; $u = &$t; $t = r_var("through-ref"); echo "B3 $u\n";
function b_typed() { static $acc = []; $acc[] = r_const(); return count($acc); }
echo "B4 ", b_typed(), b_typed(), "\n";
function b_self($x) { $x = r_param($x); $x = r_param($x); return $x; }
echo "B5 ", b_self("same"), "\n";
function b_discard() { r_const(); r_dtor_self(); echo "B6 after discard\n"; }
b_discard();
$x = $c->getP(); $y = $c->arg($x); echo "B7 $x $y\n";
function b_loop() { $s = 0; for ($i = 0; $i < 4; $i++) { $s = r_var($s) + r_const(); } return $s; }
echo "B8 ", b_loop(), "\n";
$k = r_undef(); echo "B9 "; var_dump($k);
// C: last argument read from a slot
$s = "abc"; echo "C1 ", strlen($s), " ", strtoupper($s), " ", count([$s, $s]), "\n";
function c_undef() { return strlen($nope2); }
try { echo "C2 "; var_dump(c_undef()); } catch (TypeError $e) { echo get_class($e), "\n"; }
$a2 = [3, 1, 2]; sort($a2); echo "C3 ", implode(",", $a2), "\n";
$r1 = "ref"; $r2 = &$r1; echo "C4 ", strlen($r2), " ", $c->arg($r2), "\n";
$m = "m"; echo "C5 ", $c->arg($m), $c->arg($m), "\n";
function c_this() { $o = new C; $v = "pv"; return $o->arg($v); }
echo "C6 ", c_this(), "\n";
$big = new D("arg"); echo "C7 ", $c->arg($big)->n, "\n"; $big = null; echo "C7 end\n";
$n = null; echo "C8 "; var_dump(is_null($n), $c->arg($n));
function c_two($a, $b) { return "$a-$b"; }
$p1 = "one"; $p2 = "two"; echo "C9 ", c_two($p1, $p2), " ", str_replace($p1, $p2, "one one"), "\n";
echo "end\n";
