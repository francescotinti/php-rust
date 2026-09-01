<?php // DERIVATO DICHIARATO di wp164-harness/arith-dq.php (criterio f0 braccio b): SOLO 14 slot dummy PRIMA del loop; corpo IDENTICO
$p01=0;$p02=0;$p03=0;$p04=0;$p05=0;$p06=0;$p07=0;$p08=0;$p09=0;$p10=0;$p11=0;$p12=0;$p13=0;$p14=0;
$s=0; for($i=0;$i<250000000;$i++){ $s += $i*3 - ($i>>2); } echo $s,"\n";
