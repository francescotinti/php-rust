<?php
// S-155 istruttoria ce-count — CONTROLLO attribuzione args-Vec: builtin a
// corpo 0-alloc (function_exists → is_name_callable: find_fn_ci/LcKey/tabelle,
// nessuna alloc al sorgente). Attesa: k=1 (SOLO args-Vec di pop_keys,
// 1 argomento → Vec 16 B). Due N via env FEN. CONTEGGI, mai tempo.
$n = (int)(getenv('FEN') ?: 100000);
$hit = 0;
for ($i = 0; $i < $n; $i++) {
    if (function_exists('strlen')) { $hit++; }
}
echo "FE-COUNT-OK n=$n hit=$hit\n";
