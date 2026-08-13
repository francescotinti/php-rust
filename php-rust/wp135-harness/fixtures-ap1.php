<?php
// S-135 AP1 — fixture di fedeltà (criterio s135-criterio-ap1.md p.7).
// Ogni sezione stampa: parità oracle==candidato==stash pena STOP leva.
echo "s1 fast puro\n";
$a = [1, 2]; $a[0] = 9; $a[5] = 'x'; $a['k'] = true; print_r($a);
echo "s2 base Ref\n";
$b = [1]; $r = &$b; $r[0] = 5; $r[1] = 6; print_r($b); unset($r);
echo "s3 vivify da null/assente\n";
unset($v); $v[3] = 'n'; print_r($v);
$w = null; $w['a'] = 1; print_r($w);
echo "s4 stringa-offset\n";
$s = 'abc'; $s[1] = 'X'; var_dump($s);
echo "s5 append\n";
$c = []; $c[] = 1; $c[] = 2; print_r($c);
echo "s6 nested nkeys>1\n";
$d = []; $d[1][2] = 3; $d['x']['y']['z'] = 4; print_r($d);
echo "s7 ArrayAccess\n";
$ao = new ArrayObject([]); $ao[0] = 'aa'; $ao['w'] = 'bb'; print_r($ao->getArrayCopy());
echo "s8 chiavi coerte\n";
$e = []; $e[1.7] = 'f'; $e[true] = 't'; $e[null] = 'n'; $e["5"] = 's5'; $e["05"] = 'lit'; var_dump($e);
echo "s9 chiave illegale\n";
$g = []; try { $g[[]] = 1; } catch (TypeError $t) { echo $t->getMessage(), "\n"; }
echo "s10 scrittura ATTRAVERSO elemento Ref (REF-4)\n";
$h = [0]; $rr = &$h[0]; $h[0] = 9; var_dump($rr); unset($rr);
echo "s11 COW su array condiviso\n";
$i1 = [1]; $i2 = $i1; $i2[0] = 2; print_r($i1); print_r($i2);
echo "s12 base false (deprecation) e oggetto non-AA\n";
$f = false; $f[0] = 1; print_r($f);
$o = new stdClass; try { $o[0] = 1; } catch (Error $t) { echo $t->getMessage(), "\n"; }
echo "s13 global e superglobal\n";
function wg() { global $gv; $gv[2] = 'g'; }
$gv = [0 => 'a']; wg(); print_r($gv);
$_GET['q'] = 'sq'; var_dump($_GET['q']);
echo "s14 valore Ref e overwrite oggetto\n";
$val = 7; $vr = &$val; $j = []; $j[0] = $vr; $val = 8; var_dump($j[0]);
$k = []; $k[0] = new ArrayObject([1]); $k[0] = 2; var_dump($k[0]);
echo "fine\n";
