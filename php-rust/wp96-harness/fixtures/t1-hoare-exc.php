<?php
// A-TH-97-1 (Hoare, Concilio WP-97) — il controesempio, reso eseguibile.
//
// $m e' vivo; `echo $m` sta FUORI da ogni regione protetta; dentro il try c'e'
// una def di $m che NON avviene, perche' l'op precedente lancia. Il catch
// legge $m e deve vedere il VECCHIO valore.
//
// Il difetto: l'analisi dava a ogni op della regione un arco eccezionale, poi
// sottraeva le def di quell'op ANCHE dal contributo di quell'arco. Sullo
// `StoreSlot($m)` la def uccideva $m pur venendo dal ramo in cui la def non e'
// avvenuta: `live_out` dell'`echo` perdeva $m, il sito diventava «movibile», e
// con TakeSlot il catch avrebbe letto Undef dove Zend stampa il valore.
function hoare_exc($d) {
    $m = "valore-che-il-catch-deve-vedere";
    echo $m, "\n";
    try {
        $m = 10 / $d;                 // DivisionByZeroError: la def non avviene
    } catch (\DivisionByZeroError $e) {
        echo $m, "\n";                // Zend stampa il valore, non Undef
    }
}
hoare_exc(0);
