<?php // GEMELLO L1-residente di arith-dcfar.php (S-168 sanatura az.3): stesso array, stesso corpo, maschera 1023 ⇒ 1024 elementi caldi
$a=[]; for($j=0;$j<4194304;$j++){ $a[]=$j; }
$s=0; for($i=0;$i<20000000;$i++){ $s += $a[($i*2654435761)&1023]; } echo $s,"\n";
