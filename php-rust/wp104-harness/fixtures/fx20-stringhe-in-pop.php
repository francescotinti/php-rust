<?php
// fx20 stringhe-in-Pop (KS-BA-105-1, Concilio WP-105): l'arbitro del leak
// che il giudice prop non può vedere. Ogni iterazione fa morire stringhe
// HEAP (non-interned) sui DUE sentieri della leva H-C2:
//   - Pop: un expression-statement il cui risultato stringa viene scartato;
//   - overwrite di slot: $s riassegnato a ogni giro.
// Il VERDETTO di leak NON è in-script (memory_get_usage in phpr è uno stub
// costante — scoperto dal mutation-check KS-MA-105-1, rosso archiviato in
// wp104-harness/denti-rossi/): lo emette il GATE misurando il peak RSS del
// processo (~1M stringhe trattenute ≈ +250 MiB, inconfondibile).
// Qui solo output deterministico byte-identico all'oracle.
$base = str_repeat("x", 64);
for ($i = 0; $i < 1000000; $i++) {
    $s = $base . $i;
    $s . "tail";
}
var_dump($s === $base . "999999");
echo "fx20 done\n";
