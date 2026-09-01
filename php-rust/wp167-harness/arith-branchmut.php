<?php // MUTANTE contatori (criterio f0 braccio c): branch data-dipendente pseudo-casuale — DEVE mostrare branch-miss/iter >=10x di arith-dq
$s=0; for($i=0;$i<250000000;$i++){ if((($i*2654435761)>>13)&1){ $s += $i*3 - ($i>>2); } else { $s -= 1; } } echo $s,"\n";
