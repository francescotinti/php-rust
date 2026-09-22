<?php
// FX-CR1 (S-181): presidio bilaterale della leva L-CR1 «Call/Ret magri» — oracle == phpr byte-id.
// Copre: arità esatta (fast path, ingresso a ip=1), troppo pochi argomenti (cammino lento: ArgumentCountError
// col messaggio Zend), surplus + func_get_args, default, by-ref, ricorsione, dtor a fine frame, metodi (IC),
// closure/dinamica/call_user_func, hint (non simple_call), Ref che decade. I dtor dei locali del callee si
// osservano DOPO lo statement (assegnazione separata): il pin s180 li esegue allo Sweep, l'oracle al ritorno —
// divergenza PRE-esistente (stessa riga `echo "x: ", f(), "\n"` diverge sul pin), fuori perimetro della leva.
function add($a, $b) { return $a + $b; }
function three($a, $b, $c) { return "$a|$b|$c"; }
function withargs($a, $b) { return count(func_get_args()) . ':' . implode(',', func_get_args()); }
function opt($a, $b = 7) { return $a * $b; }
function &byref(array &$arr) { return $arr[0]; }
function rec($n) { return $n <= 0 ? 0 : 1 + rec($n - 1); }
function byval($x) { $x++; return $x; }
function t(int $a, int $b): int { return $a + $b; }
function noargs() { return 'n'; }
function nested($a) { return add($a, add($a, 1)); }
class D { public function __construct(public string $n) {} function __destruct() { echo "dtor {$this->n}\n"; } }
function mk($n) { $d = new D($n); return $n; }
function usesobj($d, $x) { return $d->n . $x; }
class K {
    public $v = 3;
    function m($a, $b) { return $this->v + $a + $b; }
    function one($a) { return $a * 2; }
    function opt2($a, $b = 5) { return $a . $b; }
    static function s($a, $b) { return $a - $b; }
}
class K2 extends K { function m($a, $b) { return "k2:" . parent::m($a, $b); } }
echo "exact: ", add(1, 2), "\n";
echo "exact3: ", three(1, 'x', 2.5), "\n";
echo "noargs: ", noargs(), "\n";
echo "nested: ", nested(3), "\n";
echo "fga exact: ", withargs(5, 6), "\n";
echo "fga surplus: ", withargs(5, 6, 7), "\n";
echo "opt one: ", opt(3), "\n";
echo "opt two: ", opt(3, 4), "\n";
$arr = [10, 20]; $r = &byref($arr); $r = 99; echo "byref: ", $arr[0], "\n";
echo "rec: ", rec(50), "\n";
$mk = mk('a'); echo "mk: $mk\n";
$d = new D('b'); echo "usesobj: ", usesobj($d, '!'), "\n"; unset($d);
$k = new K; echo "m exact: ", $k->m(1, 2), "\n"; echo "one: ", $k->one(4), "\n"; echo "s: ", K::s(9, 4), "\n";
echo "opt2 one: ", $k->opt2('a'), " opt2 two: ", $k->opt2('a', 'b'), "\n";
$k2 = new K2; for ($i = 0; $i < 3; $i++) { echo "loop m: ", $k->m($i, 1), " ", $k2->m($i, 2), "\n"; }
try { add(1); } catch (ArgumentCountError $e) { echo "ACE: ", $e->getMessage(), "\n"; }
try { $k->m(1); } catch (ArgumentCountError $e) { echo "ACE m: ", $e->getMessage(), "\n"; }
try { three(1, 2); } catch (ArgumentCountError $e) { echo "ACE3: ", $e->getMessage(), "\n"; }
try { K::s(1); } catch (ArgumentCountError $e) { echo "ACE s: ", $e->getMessage(), "\n"; }
try { nested(); } catch (ArgumentCountError $e) { echo "ACE nested: ", $e->getMessage(), "\n"; }
$f = 'add'; echo "dyn: ", $f(2, 3), "\n";
echo "cuf: ", call_user_func('add', 4, 5), "\n";
echo "cufa: ", call_user_func_array('three', [1, 2, 3]), "\n";
$c = function ($a, $b) { return $a . $b; }; echo "closure: ", $c('p', 'q'), "\n";
$ref = 5; echo "byval: ", byval($ref), " ", $ref, "\n";
$rr = &$ref; echo "ref arg: ", add($rr, 1), "\n";
echo "typed: ", t(1, 2), "\n";
try { t('x', 2); } catch (TypeError $e) { echo "TE: ", get_class($e), "\n"; }
function inner_dtor() { $d = new D('c'); return usesobj($d, '?'); }
$id = inner_dtor(); echo "inner_dtor: $id\n";
echo "FX-CR1 DONE\n";
