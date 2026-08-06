<?php
// fx20 stringhe-in-Pop (KS-BA-105-1, Concilio WP-105): l'arbitro del leak
// che il giudice prop non può vedere. Ogni iterazione fa morire stringhe
// HEAP (non-interned) sui DUE sentieri della leva H-C2:
//   - Pop: un expression-statement il cui risultato stringa viene scartato;
//   - overwrite di slot (StoreSlot/reg_store_slot): $s riassegnato a ogni giro.
// Se il fast-out saltasse il glue delle Str (il difetto del predicato v1),
// ~200k stringhe da ~70 B resterebbero vive: growth >> soglia e il dente
// morde. Attesa: growth stabile ⇒ bool(true), byte-identico all'oracle.
$base = str_repeat("x", 64);
$mem0 = 0;
$mem1 = 0;
for ($i = 0; $i < 200000; $i++) {
    $s = $base . $i;      // il vecchio $s (Str) muore nell'overwrite
    $s . "tail";          // la Str temporanea muore via Pop
    if ($i === 50000) {
        $mem0 = memory_get_usage();
    }
    if ($i === 150000) {
        $mem1 = memory_get_usage();
    }
}
var_dump($s === $base . "199999");
$growth = $mem1 - $mem0;
var_dump($growth < 64 * 1024);
echo "fx20 done\n";
