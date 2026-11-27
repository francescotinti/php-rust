<?php class O { function m($a,$b){ return $a+$b; } } $o=new O; $s=0; $t=1;
for($i=0;$i<20000000;$i++){ $s = $o->m($s,$t); } echo $s,"\n";