<?php function f($a,$b){ return $a+$b; } $s=0; for($i=0;$i<20000000;$i++){ $s = f($s,1); } echo $s,"\n";
