<?php
// S-145 smoke conteggi: ogni chiave attesa dello smoke DEVE valere >=1
// (esito ESATTO per chiave, lezione tranche-2). Le classi ref/rcother NON
// sono chiavi di smoke (in uno script minimo possono legittimamente
// restare a 0, dichiarato nel modello).
$n = 1; $m = $n;            // clone_scalar
$s = "x"; $t = $s . "y";    // clone_str (il concat legge la base)
$a = [1, 2]; $b = $a;       // clone_arr
class C { public $p = 1; }
$o = new C; $q = $o;        // clone_obj + gcnote_cont (nota su container)
$q->p = 2;
echo "ok\n";
