<?php
// fx-mc2.php — S-166 az.rev.1 (revisione S-165, lente semantica): error-path
// e casi scoperti del fast path L-MC1d. Arbitro: pin s165 (fast path) vs
// stash s163 (solo funnel) BYTE-identici; oracle = fedeltà (diff solo se NUOVE).
error_reporting(E_ALL);

class P {
    public $v = 0;
    function add($a, $b) { return $a + $b; }
    function &retref($k, $_) { return $this->v; }
    function take($a, $b) { return $b; }
    function two($a, $b) { return 0; }
}
class Q { // ricevitore temporaneo con dtor parlante
    function __destruct() { echo "dtor:Q "; }
    function add($a, $b) { return $a + $b; }
}
class R { function __destruct() { echo "dtor:R "; } } // argomento temporaneo
class D {
    public $t;
    function __construct($t) { $this->t = $t; }
    function __destruct() { echo "dtor:D{$this->t} "; }
}
class G {
    public function __get($n) {
        echo "get:$n ";
        if ($n === 'boom') { throw new \Exception('boom'); }
        return 5;
    }
}

$o = new P;
for ($i = 0; $i < 3; $i++) { $o->add(1, 2); } // IC caldo: fast path attivo

// ── caso 1: Err a metà materializzazione (Append su param by-value) ──
// ordine dei drop recv/args osservabile via __destruct + riuso handle-id.
echo "C1: ";
$a = [1, 2];
try { (new Q)->add($a[], new R); } catch (\Error $e) { echo "E1:", $e->getMessage(), " "; }
$h1 = new P; $h2 = new P;
echo "ids:", spl_object_id($h1), ",", spl_object_id($h2), "\n";

// ── caso 2: ArgPlace con passo Prop → __get (anche che lancia) ──
echo "C2: ";
$g = new G;
echo $o->add($g->p, 2), " ";
try { $o->add($g->boom, 2); } catch (\Exception $e) { echo "E2:", $e->getMessage(); }
echo "\n";

// ── caso 3: deref=false — $x =& $o->retref(…) deve ALIASARE ──
echo "C3: ";
$o->v = 7;
$x = &$o->retref(1, 2);
$x = 42;
echo $o->v, "\n";

// ── caso 4: __destruct su last-ref di argomento (ordine slot al ritorno) ──
echo "C4: ";
echo $o->take(new D('a'), 9), " ";
echo $o->two(new D('x'), new D('y')), " ";
echo "\n";

echo "done\n";
