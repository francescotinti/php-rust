<?php
// str-dq.php — giudice F2 h1 (criterio s191-criterio-f2-builtin.md p.3): UNA chiamata builtin a valore per iterazione,
// argomenti costanti, risultato consumato (assegnazione: la stringa precedente muore qui, come nel micro `str`).
// N nel sorgente (`$i<30000000`), letto dagli script con la stessa awk dei driver *-dq. Parità con l'oracle: l'output.
$t = '';
for ($i = 0; $i < 30000000; $i++) { $t = substr('abcdefgh', $i & 3, 4); }
echo $t, "\n";
