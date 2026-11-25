<?php
// S-194 C4 behaviour tooth: plain-variable arguments of dynamic calls. Expected = the oracle's output.
namespace C4;
class K {
    public $log = [];
    function val($a, $b = 7) { return "val:" . var_export($a, true) . "," . var_export($b, true); }
    function ref(&$a) { $a = is_int($a) ? $a + 1 : 'made'; return "ref"; }
    function mix($a, &$b, $c) { $b = $a . $c; return "mix"; }
    function typed(int $a, string $s, ?array $x = null): string { return "typed:$a,$s," . gettype($x); }
    function coer(int $a, float $f) { return "coer:" . var_export($a, true) . "," . var_export($f, true); }
    function vr(&...$r) { foreach ($r as &$x) { $x = $x * 2; } return "vr"; }
    function vv(...$r) { return "vv:" . implode('|', $r); }
    static function st($a, &$b) { $b = $a + 100; return "st"; }
    function __call($n, $a) { return "__call:$n:" . json_encode($a); }
    static function __callStatic($n, $a) { return "__callStatic:$n:" . json_encode($a); }
    function __invoke($a, &$b) { $b = "inv$a"; return "invoke"; }
    function gen($a) { yield $a; yield $a * 2; }
    function keep($a) { $this->log[] = $a; return $this; }
}
class P { public $v; function __construct($a, &$b) { $this->v = $a; $b = "ctor$a"; } }
class Q extends P { function __construct($a, &$b) { parent::__construct($a, $b); $this->v .= "q"; } }
function show(...$xs) { foreach ($xs as $x) { echo is_scalar($x) || $x === null ? var_export($x, true) : json_encode($x), " "; } echo "\n"; }
function run() {
    $o = new K;
    $a = 1; $s = "s";
    show($o->val($a), $a);
    show($o->val($a, $s), $a, $s);
    show($o->ref($a), $a);
    $b = $a; $b++; show($a, $b);                 // $a stays a value after the by-value call
    $m = null; show($o->mix($s, $m, $a), $m);
    $nul = null; $ar = [1]; show($o->typed($a, $s), $o->typed($a, $s, $nul), $o->typed($a, $s, $ar));
    $n = "12"; $f = 3; show($o->coer($n, $f), $n, $f);
    $x = 1; $y = 2; show($o->vr($x, $y), $x, $y);
    show($o->vv($x, $y, $s));
    $c = 'C4\\K'; $r = 0; show($c::st($x, $r), $r);
    show($o->nope($x, $s), $c::snope($y));
    $i = 5; $w = null; show($o($i, $w), $w);
    $fn = function ($p, &$q) { $q = $p . "!"; return "clo"; }; $z = 0; show($fn($s, $z), $z);
    $fc = $o->val(...); show($fc($x, $y));
    $fr = $o->ref(...); $t = 10; show($fr($t), $t);
    $u = 0; $p = new P($x, $u); show($p->v, $u);
    $cn = 'C4\\Q'; $u2 = 0; $q = new $cn($y, $u2); show($q->v, $u2);
    foreach ($o->gen($x) as $g) { echo $g, " "; } echo "\n";
    $arr = [3, 1, 2]; sort($arr); show($arr);
    $subj = "ab12"; $mm = null; show(preg_match('/\d+/', $subj, $mm), $mm);
    $ao = new \ArrayObject([1, 2]); $k = 1; show($ao->offsetGet($k), $k);
    static $st = 4; show($o->val($st), $o->ref($st), $st);
    global $gg; $gg = 8; show($o->val($gg), $o->ref($gg), $gg);
    $ra = 1; $rb = &$ra; show($o->val($ra), $o->ref($ra), $rb);
    for ($j = 0, $acc = 0; $j < 5; $j++) { $acc += strlen($o->val($j)); } show($acc, $j);
    $o2 = $o->keep($a)->keep($s)->keep($a); show($o2->log);
    $arr2 = [1, 2]; $cp = $arr2; $o->val($arr2); $arr2[] = 3; show($cp, $arr2);
    $h = fn($v) => $v + 1; $e = 1; show($h($e), $e);
    $name = 'val'; show($o->$name($e), [$o, 'ref']($e), $e);
    show(call_user_func([$o, 'val'], $e), call_user_func_array([$o, 'val'], [$e, $s]));
    $fib = new \Fiber(function ($a) use ($o) { $x = \Fiber::suspend($o->val($a)); return $o->val($x); });
    $v1 = 3; $r1 = $fib->start($v1); $v2 = 4; $fib->resume($v2); show($r1, $fib->getReturn());
}
function undef() {
    $o = new K;
    show($o->val($nope1));
    show($o->ref($nope2), $nope2);
    show($o->typed(1, "x", $nope3));
    $c = 'C4\\K'; show($c::st(1, $nope4), $nope4);
}
set_error_handler(function ($no, $msg) { echo "[$no] $msg\n"; return true; });
run();
undef();
try { $o = new K; $bad = "zz"; $o->typed($bad, $bad); } catch (\TypeError $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
