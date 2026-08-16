<?php
// S-145 sonda-B lato prezzi: il driver passa al builtin le tre specie
// condivise (str/arr/obj) e pretende true. Esiste solo per la build
// sonda-price; l'oracle non lo esegue (builtin assente, DICHIARATO).
class SB { public $x = 1; }
$s = "sonda-b-prezzi-str";
$a = [1, 2, 3];
$o = new SB;
$fn = '__phpr_sonda_b';   // string-callable: instrada su invoke_named,
$ok = $fn($s, $a, $o);     // l'unico sito dove il probe monta il builtin

echo $ok ? "SONDA-OK\n" : "SONDA-FAIL\n";
