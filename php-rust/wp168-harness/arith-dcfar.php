<?php // CONTROLLO POSITIVO D-cache (S-168 sanatura az.3): array 4M Long (64MB), accesso pseudo-casuale ⇒ un miss DRAM per iterazione; gemello arith-dcnear.php (stesso corpo, maschera 1023 ⇒ L1-residente)
$a=[]; for($j=0;$j<4194304;$j++){ $a[]=$j; }
$s=0; for($i=0;$i<20000000;$i++){ $s += $a[($i*2654435761)&4194303]; } echo $s,"\n";
