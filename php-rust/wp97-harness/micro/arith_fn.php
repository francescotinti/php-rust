<?php function loop_(){ $s=0; for($i=0;$i<100000;$i++){ $s += $i*3 - ($i>>2); } return $s; } echo loop_(),"\n";
