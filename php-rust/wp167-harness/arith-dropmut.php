<?php // MUTANTE drop-census (criterio f0 braccio d): +1 stringa temporanea/iter — il conteggio drop-stringhe deve spostarsi di N ESATTO (N=200000)
$s=0; for($i=0;$i<200000;$i++){ $t = $i . "x"; $s += $i*3 - ($i>>2); } echo $s,"\n";
