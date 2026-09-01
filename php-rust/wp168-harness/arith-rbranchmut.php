<?php // MUTANTE branch (S-168 → S-169): LCG a 31 bit, bit 30 (periodo 2^31): sequenza NON apprendibile dal predittore, a differenza di arith-branchmut.php (periodo ~10)
$s=0; $r=12345; for($i=0;$i<250000000;$i++){ $r=($r*1103515245+12345)&0x7fffffff; if(($r>>30)&1){ $s += $i*3 - ($i>>2); } else { $s -= 1; } } echo $s,"\n";
