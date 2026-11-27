<?php class O { function m($a,$b){ return $a+$b; } } $o=new O; $s=0; $t=1; $u=0;
for($i=0;$i<20000000;$i++){ $u = $o->m($t,1); } echo $u,"\n";