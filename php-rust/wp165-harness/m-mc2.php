<?php class C { function f($a,$b){ return $a+$b; } } $o=new C; $s=0; for($i=0;$i<20000000;$i++){ $s=$o->f($s,1); } echo $s,"\n";
