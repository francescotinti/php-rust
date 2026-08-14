<?php
// S-136 az.rev. S-135 #5+#1 — fixture AP1 v2: UNO statement per riga (espone
// l'attribuzione di riga dei diag) + sezioni s15-s22 assorbite dai probe del
// revisore S-135. Parità attesa: pin==stash BYTE-ID; vs oracle valgono le
// divergenze a catalogo §3.21 (a/b/c).
echo "s1 fast puro\n";
$a = [1, 2];
$a[0] = 9;
$a[5] = 'x';
$a['k'] = true;
print_r($a);
echo "s2 base Ref\n";
$b = [1];
$r = &$b;
$r[0] = 5;
$r[1] = 6;
print_r($b);
unset($r);
echo "s3 vivify da null/assente\n";
unset($v);
$v[3] = 'n';
print_r($v);
$w = null;
$w['a'] = 1;
print_r($w);
echo "s4 stringa-offset\n";
$s = 'abc';
$s[1] = 'X';
var_dump($s);
echo "s5 append\n";
$c = [];
$c[] = 1;
$c[] = 2;
print_r($c);
echo "s6 nested nkeys>1\n";
$d = [];
$d[1][2] = 3;
$d['x']['y']['z'] = 4;
print_r($d);
echo "s7 ArrayAccess\n";
$ao = new ArrayObject([]);
$ao[0] = 'aa';
$ao['w'] = 'bb';
print_r($ao->getArrayCopy());
echo "s8 chiavi coerte\n";
$e = [];
$e[1.7] = 'f';
$e[true] = 't';
$e[null] = 'n';
$e["5"] = 's5';
$e["05"] = 'lit';
var_dump($e);
echo "s9 chiave illegale\n";
$g = [];
try {
    $g[[]] = 1;
} catch (TypeError $t) {
    echo $t->getMessage(), "\n";
}
echo "s10 scrittura ATTRAVERSO elemento Ref (REF-4)\n";
$h = [0];
$rr = &$h[0];
$h[0] = 9;
var_dump($rr);
unset($rr);
echo "s11 COW su array condiviso\n";
$i1 = [1];
$i2 = $i1;
$i2[0] = 2;
print_r($i1);
print_r($i2);
echo "s12 base false (deprecation) e oggetto non-AA\n";
$f = false;
$f[0] = 1;
print_r($f);
$o = new stdClass;
try {
    $o[0] = 1;
} catch (Error $t) {
    echo $t->getMessage(), "\n";
}
echo "s13 global e superglobal\n";
function wg() {
    global $gv;
    $gv[2] = 'g';
}
$gv = [0 => 'a'];
wg();
print_r($gv);
$_GET['q'] = 'sq';
var_dump($_GET['q']);
echo "s14 valore Ref e overwrite oggetto\n";
$val = 7;
$vr = &$val;
$j = [];
$j[0] = $vr;
$val = 8;
var_dump($j[0]);
$k = [];
$k[0] = new ArrayObject([1]);
$k[0] = 2;
var_dump($k[0]);
echo "s15 distruttore dell'elemento SPOSTATO (timing)\n";
class D15 {
    public $n;
    public function __construct($n) { $this->n = $n; }
    public function __destruct() { echo "dtor {$this->n}\n"; }
}
$m = [];
$m[0] = new D15('primo');
echo "prima dell'overwrite\n";
$m[0] = new D15('secondo');
echo "dopo l'overwrite\n";
unset($m);
echo "s16 valore d'espressione con RHS alias\n";
$av = 1;
$ar = &$av;
$q = [];
var_dump($q['k'] = $ar);
$av = 2;
var_dump($q['k']);
echo "s17 chiave illegale su array CONDIVISO (stato dopo)\n";
$sh1 = [10];
$sh2 = $sh1;
try {
    $sh2[[]] = 1;
} catch (TypeError $t) {
    echo "TypeError\n";
}
print_r($sh1);
print_r($sh2);
echo "s18 next-key dopo int alta e PHP_INT_MAX\n";
$nk = [];
$nk[7] = 'a';
$nk[] = 'b';
print_r($nk);
$mx = [];
$mx[PHP_INT_MAX] = 'max';
try {
    $mx[] = 'over';
} catch (Error $t) {
    echo $t->getMessage(), "\n";
}
print_r($mx);
echo "s19 auto-assegnazione\n";
$sa = ['k' => 1];
$sa['k'] = $sa;
print_r($sa);
echo "s20 chiavi stringa di bordo\n";
$bk = [];
$bk["08"] = 'a';
$bk["-0"] = 'b';
$bk[" 5"] = 'c';
$bk["-7"] = 'd';
var_dump($bk);
echo "s21 GLOBALS\n";
$gg = [1];
$GLOBALS['gg'][0] = 2;
print_r($gg);
echo "s22 ciclo base-in-se-stessa\n";
$cy = [];
$cy[0] = &$cy;
$cy[0] = 7;
var_dump($cy);
echo "fine\n";
