<?php
// fx-sw1 — S-174 leva «Sweep-in-op» (criterio s174-criterio.md p.5): fixture BILATERALE
// (oracle == pin == candidato, byte-id). Blocchi «etichetta: begin» … «post …»: le righe
// senza etichetta (distruttori, post) appartengono al blocco che le precede, così l'ORDINE
// tra statement, distruttori dei temporanei morti nello STESSO statement (lo Sweep di fine
// statement DEVE girare) e riga «post» è giudicato per blocco. Poi le forme `for` del
// back-edge fuso (C) coi casi di miss. Marcatore finale FX-SW1 DONE.
class D { public function __construct(public string $n) {} public function __destruct() { echo "dtor-{$this->n}\n"; } }
class P { public int $x = 3; public $y = 4; public $z = 0; }
class Q { public int $v = 2; }
class P2 { public $y = 4; public $z = 0; }   // PLAIN (nessuna prop tipizzata): perimetro del probe P1/P4 (IC set solo su classi plain, S-173)
function mk(string $n, int $v): int { $d = new D($n); return $v; }
function mkplain(int $v): int { $q = new Q; return $v + $q->v; }
function pressure(int $k): int { $s = 0; for ($i = 0; $i < $k; $i++) { $a = []; $a[] = &$a; $s += 1; } return $s; }

// --- B: BinarySTDst in place (dst Long, rhs Long) con temporaneo morto nel rhs: dtor PRIMA di «post»
echo "stdst-dtor: begin\n"; $s = 10; $s += mk('a', 5); echo "post $s\n";
echo "stdst-dtor-twice: begin\n"; $s = 10; $s += mk('a1', 5); $s += mk('a2', 5); echo "post $s\n";
echo "stdst-dtor-loop: begin\n"; $s = 0; for ($i = 0; $i < 3; $i++) { $s += mk("l$i", $i); } echo "post $s\n";
// --- B fuori dominio (dst non-Long / op non i64 / overflow): sentiero storico, Sweep gira come sempre
echo "stdst-dtor-double: begin\n"; $t = 1.5; $t += mk('f', 2); echo "post $t\n";
echo "stdst-dtor-null: begin\n"; $u = null; $u += mk('n', 2); echo "post $u\n";
echo "stdst-dtor-concat: begin\n"; $v = 7; $v .= mk('c', 1); echo "post $v\n";
echo "stdst-dtor-overflow: begin\n"; $w = PHP_INT_MAX; $w += mk('o', 1); echo "post $w\n";
echo "stdst-dtor-div: begin\n"; $x = 8; $x /= mk('d', 2); echo "post $x\n";

// --- B: BinarySCSCDst (forma arith-dq: `$s += $a*3 - ($b>>1)`, assegnazione COMPOSTA) — Long puri: nessun rilascio; parità valore e loop
$a = 5; $b = 9; $l = 0; for ($i = 0; $i < 4; $i++) { $l += $a * 3 - ($b >> 1); } echo "scsc-loop: $l\n";
$l = 0; $a = PHP_INT_MAX; for ($i = 0; $i < 2; $i++) { $l += $a * 3 - ($b >> 1); } echo "scsc-overflow: " . var_export($l, true) . "\n";
$l = 0.5; $a = 5; for ($i = 0; $i < 2; $i++) { $l += $a * 3 - ($b >> 1); } echo "scsc-double-dst: $l\n";

// --- B: P3 (PropGetSlot+BinarySTDst) e P1/P4 (PropGetSlotRecv+BinaryTCPropSetPop): forme pure e con rilascio a monte
$o = new P; $s = 1; $s += $o->x; echo "p3: $s\n";
$o->z = 0; $c = 0; for ($i = 0; $i < 3; $i++) { $o->z = $o->y + 2; $c += 1; } echo "p1-loop: {$o->z} $c\n";
$o->z = 0; for ($i = 0; $i < 3; $i++) { $o->z = $o->z + 5; } echo "p4-loop: {$o->z}\n";
echo "p3-dtor: begin\n"; $s = 1; $s += $o->x + mk('p3t', 0); echo "post $s\n";
echo "p3-chain: begin\n"; $s = 0; for ($i = 0; $i < 2; $i++) { $s += mk("pc$i", 1); $s += $o->x; } echo "post $s\n";
$q = new P2;
echo "p1-chain: begin\n"; $q->z = 0; for ($i = 0; $i < 2; $i++) { $s += mk("qc$i", 1); $q->z = $q->y + 2; } echo "post {$q->z}\n";
echo "p4-chain: begin\n"; $q->z = 0; for ($i = 0; $i < 2; $i++) { $s += mk("rc$i", 1); $q->z = $q->z + 5; } echo "post {$q->z}\n";
echo "p4-dtor: begin\n"; $o->z = 1; $o->z = $o->z + mk('p4t', 0); echo "post {$o->z}\n";
// --- v3 (replica alla revisione S-174, rilievo 2): forme a LOOP osservabili per ITERAZIONE (echo nel corpo: il
// dtor del temporaneo deve precedere «k$i»/«i$i») e coppie per SITO (…-ctl: statement intermedio NON fuso ⇒ resta intatto)
echo "stdst-dtor-loop-echo: begin\n"; $s = 0; for ($i = 0; $i < 3; $i++) { $s += mk("le$i", $i); echo "i$i\n"; } echo "post $s\n";
echo "for-dtor-echo: begin\n"; $c = 0; for ($i = 0; $i < 3; $i++) { $c += mk("fd$i", 1); echo "i$i\n"; } echo "post $c\n";
echo "p3-site: begin\n"; $s = 0; for ($i = 0; $i < 2; $i++) { $s += mk("p3s$i", 1); $s += $o->x; echo "k$i\n"; } echo "post $s\n";
echo "p3-site-ctl: begin\n"; $s = 0; for ($i = 0; $i < 2; $i++) { $s += mk("p3c$i", 1); $u = 0; echo "k$i\n"; } echo "post $s\n";
echo "p1-site: begin\n"; $s = 0; $q->z = 0; for ($i = 0; $i < 2; $i++) { $s += mk("p1s$i", 1); $q->z = $q->y + 2; echo "k$i\n"; } echo "post {$q->z}\n";
echo "p1-site-typed: begin\n"; $s = 0; $o->z = 0; for ($i = 0; $i < 2; $i++) { $s += mk("p1t$i", 1); $o->z = $o->y + 2; echo "k$i\n"; } echo "post {$o->z}\n";
echo "p4-site: begin\n"; $s = 0; $q->z = 0; for ($i = 0; $i < 2; $i++) { $s += mk("p4s$i", 1); $q->z = $q->z + 5; echo "k$i\n"; } echo "post {$q->z}\n";
echo "scsc-site: begin\n"; $s = 0; $a = 5; $b = 9; $l = 0; for ($i = 0; $i < 2; $i++) { $s += mk("sc$i", 1); $l += $a * 3 - ($b >> 1); echo "k$i\n"; } echo "post $l\n";

// --- B: sweep light (dentro funzione) e main con demozioni da ri-esaminare
function light(): int { $q = 0; $q += mk('lt', 1); echo "post-in $q\n"; return $q; }
echo "light: begin\n"; $r = light(); echo "post $r\n";
function window(): int { $o1 = new D('w1'); $o2 = $o1; unset($o1); $s = 3; $s += 1; return $s; }
echo "window: begin\n"; $s = 100; $s += window(); echo "post $s\n";

// --- B: statement eseguito DENTRO un distruttore (IN_DESTRUCTOR: sweep no-op per costruzione; temporaneo SENZA
// distruttore — l'ordine dtor-in-dtor è divergenza pre-esistente del pin, fuori perimetro)
class E { public int $acc = 0; public function __destruct() { $this->acc += mkplain(2); echo "post-in {$this->acc}\n"; } }
echo "e-dtor: begin\n"; $e = new E; unset($e); echo "post\n";

// --- B: pressione GC (cicli) — la raccolta a valle deve vedere lo stesso stato
gc_disable(); $n = pressure(5000); echo "pressure: $n\n"; echo "collected: " . gc_collect_cycles() . "\n"; gc_enable();
$n = pressure(20000); echo "pressure-on: $n\n"; echo "collected-on: " . gc_collect_cycles() . "\n";

// --- C: back-edge fuso — for canonici (Long/Int) e miss (slot non-Long, const non-Int, when, overflow, corpo vuoto, nested)
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
echo "for-dtor: begin\n"; $c = 0; for ($i = 0; $i < 3; $i++) { $c += mk("fe$i", 1); } echo "post $c\n";
$c = 0; for ($i = 0; $i < 5; $i++) { if ($i == 2) continue; $c += 1; } echo "for-continue: $c\n";
$c = 0; $k = 3; for ($i = 0; $i < $k; $i++) { $c += 1; } echo "for-slot-bound: $c\n";
$c = 0; for ($i = 0; 3 > $i; $i++) { $c += 1; } echo "for-const-lhs: $c\n";
$c = 0; $i = 0; while ($i < 3) { $c += 1; $i++; } echo "while-lt: $c\n";
$c = 0; for ($i = 0; $i < 3; $i += 2) { $c += 1; } echo "for-step2: $c\n";
echo "FX-SW1 DONE\n";
