<?php $n=0; $subj="the quick brown fox jumps over the lazy dog 12345";
for($i=0;$i<8000000;$i++){ if(preg_match('/(\w+)\s+(\d+)/', $subj, $m)) $n++; } echo $n,"\n";
