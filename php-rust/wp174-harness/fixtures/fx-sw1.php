<?php
// fx-sw1 — S-174 leva «Sweep-in-op» (criterio s174-criterio.md p.5): fixture BILATERALE
// (oracle == pin == candidato, byte-id). Ogni riga «etichetta: valore». Le forme
// osservano l'ORDINE tra output dello statement e output dei distruttori dei
// temporanei rilasciati nello STESSO statement (lo Sweep di fine statement deve
// girare), le forme di loop `for` del back-edge fuso (C) e i casi di miss.
class D { public function __construct(public string $n) {} public function __destruct() { echo "dtor-{$this->n}\n"; } }
class P { public int $x = 3; public $y = 4; public $z = 0; }
function mk(string $n, int $v): int { $d = new D($n); return $v; }
function mkobj(string $n): D { return new D($n); }
class Q { public int $v = 2; }
function mkplain(int $v): int { $q = new Q; return $v + $q->v; }
function pressure(int $k): int { $s = 0; for ($i = 0; $i < $k; $i++) { $a = []; $a[] = &$a; $s += 1; } return $s; }

// --- B: BinarySTDst in place con temporaneo morto nel rhs (dtor DEVE stampare prima del blocco seguente)
$s = 10; echo "stdst-dtor#0: $s\n"; $s += mk('a', 5); echo "stdst-dtor#1: $s\n";
$s = 0; for ($i = 0; $i < 3; $i++) { $s += mk("l$i", $i); } echo "stdst-dtor-loop: $s\n";
$t = 1.5; $t += mk('f', 2); echo "stdst-dtor-double: $t\n";
$u = null; $u += mk('n', 2); echo "stdst-dtor-null: $u\n";
$v = 7; $v .= mk('c', 1); echo "stdst-dtor-concat: $v\n";
$w = PHP_INT_MAX; $w += mk('o', 1); echo "stdst-dtor-overflow: $w\n";

// --- B: BinarySCSCDst (forma arith-dq) — solo Long puri: nessun rilascio possibile; parità del valore e loop
$a = 5; $b = 9; $l = 0; for ($i = 0; $i < 4; $i++) { $l = $l + ($a * 3 - ($b >> 1)); } echo "scsc-loop: $l\n";
$l = 0; $a = PHP_INT_MAX; for ($i = 0; $i < 2; $i++) { $l = $l + ($a * 3 - ($b >> 1)); } echo "scsc-overflow: " . var_export($l, true) . "\n";
$l = 0.5; for ($i = 0; $i < 2; $i++) { $l = $l + (5 * 3 - (9 >> 1)); } echo "scsc-double-dst: $l\n";

// --- B: P3 (PropGetSlot+BinarySTDst) e P1/P4 (PropGetSlotRecv+BinaryTCPropSetPop) con rilascio nello stesso statement
$o = new P; $s = 1; $s += $o->x; echo "p3: $s\n";
$o->z = 0; for ($i = 0; $i < 3; $i++) { $o->z = $o->y + 2; } echo "p1-loop: {$o->z}\n";
$o->z = 0; for ($i = 0; $i < 3; $i++) { $o->z = $o->z + 5; } echo "p4-loop: {$o->z}\n";
$d1 = new D('p3'); $s = 1; $s += $o->x + mk('p3t', 0); echo "p3-dtor#0: $s\n"; unset($d1); echo "p3-dtor#1\n";
$o->z = 1; $o->z = $o->z + mk('p4t', 0); echo "p4-dtor: {$o->z}\n";

// --- B: sweep light (dentro funzione) e main con demozioni da ri-esaminare
function light(): int { $q = 0; $q += mk('lt', 1); echo "light-in: $q\n"; return $q; }
echo "light#0: " . light() . "\n"; echo "light#1\n";
function window(): int { $o1 = new D('w1'); $o2 = $o1; unset($o1); $s = 3; $s += 1; return $s; }
$s = 100; $s += window(); echo "window: $s\n";

// --- B: statement eseguito DENTRO un distruttore (IN_DESTRUCTOR: lo sweep è no-op per costruzione; il temporaneo
// è SENZA distruttore: l'ordine dtor-in-dtor è divergenza pre-esistente del pin, fuori perimetro di questa fixture)
class E { public int $acc = 0; public function __destruct() { $this->acc += mkplain(2); echo "e-dtor: {$this->acc}\n"; } }
$e = new E; unset($e); echo "e-after\n";

// --- B: pressione GC (cicli) — la raccolta a valle deve vedere lo stesso stato
gc_disable(); $n = pressure(5000); echo "pressure: $n\n"; echo "collected: " . gc_collect_cycles() . "\n"; gc_enable();
$n = pressure(20000); echo "pressure-on: $n\n"; echo "collected-on: " . gc_collect_cycles() . "\n";

// --- C: back-edge fuso — for canonici e miss (slot non-Long, const non-Int, when, overflow, corpo vuoto, nested)
$c = 0; for ($i = 0; $i < 10; $i++) { $c += $i; } echo "for-lt: $c\n";
$c = 0; for ($i = 10; $i > 0; $i--) { $c += $i; } echo "for-gt-dec: $c\n";
$c = 0; for ($i = 0; $i <= 5; $i++) { $c += 1; } echo "for-le: $c\n";
$c = 0; for ($i = 0; $i != 4; $i++) { $c += 1; } echo "for-ne: $c\n";
$c = 0; for ($i = 0.0; $i < 3; $i++) { $c += 1; } echo "for-double-slot: $c " . var_export($i, true) . "\n";
$c = 0; for ($i = 0; $i < 2.5; $i++) { $c += 1; } echo "for-double-const: $c\n";
$c = 0; for ($i = 0; $i < "3"; $i++) { $c += 1; } echo "for-str-const: $c\n";
$c = 0; for ($i = PHP_INT_MAX - 2; $i < PHP_INT_MAX; $i++) { $c += 1; } echo "for-overflow-edge: $c " . var_export($i, true) . "\n";
$c = 0; for ($i = PHP_INT_MAX - 1; $i <= PHP_INT_MAX; $i++) { $c += 1; if ($c > 4) break; } echo "for-overflow-float: $c " . var_export(is_float($i), true) . "\n";
for ($i = 0; $i < 3; $i++) {} echo "for-empty: $i\n";
$c = 0; for ($i = 0; $i < 3; $i++) { for ($j = 0; $j < 2; $j++) { $c += 1; } } echo "for-nested: $c\n";
$c = 0; $i = null; for (; $i < 2; $i++) { $c += 1; } echo "for-null-init: $c " . var_export($i, true) . "\n";
$c = 0; for ($i = 0; $i < 3; $i++) { $c += mk("fe$i", 1); } echo "for-dtor: $c\n";
$c = 0; for ($i = 0; $i < 5; $i++) { if ($i == 2) continue; $c += 1; } echo "for-continue: $c\n";
$c = 0; $k = 3; for ($i = 0; $i < $k; $i++) { $c += 1; } echo "for-slot-bound: $c\n";
echo "end\n";
