<?php $s=''; for($i=0;$i<4000000;$i++){ $s = substr($s . "abc", -30); } echo strlen($s),"\n";
