<?php // MUTANTE contatori (S-168 sanatura az.2): stallo BACKEND puro — un miss DRAM per iterazione (array 4M Long, accesso pseudo-casuale); la colonna che schizza vs arith-dq = Processing/backend ⇒ c0 fissata per esclusione
$a=[]; for($j=0;$j<4194304;$j++){ $a[]=$j; }
$s=0; for($i=0;$i<30000000;$i++){ $s += $a[($i*2654435761)&4194303]; } echo $s,"\n";
