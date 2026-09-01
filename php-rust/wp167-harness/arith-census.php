<?php // DERIVATO DICHIARATO di arith-dq.php (criterio f0 braccio d): SOLO N 250000000→2000000 (i conteggi census sono per-iter, scalano esatti)
$s=0; for($i=0;$i<2000000;$i++){ $s += $i*3 - ($i>>2); } echo $s,"\n";
