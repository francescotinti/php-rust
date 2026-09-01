<?php // MUTANTE c0 (S-169 criterio p.3): lavoro RETIRING puro per iterazione — crc32 di 4 KB L1-residenti (tabella e dati caldi, loop predicibile); N=2M
$k=str_repeat("a",4096); $s=0; for($i=0;$i<2000000;$i++){ $s += $i*3 - ($i>>2); $t=crc32($k); } echo $s,"\n";
