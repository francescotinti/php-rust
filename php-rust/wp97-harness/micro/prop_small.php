<?php class C { public $x=0; public $y=1; } $o=new C; $s=0;
for($i=0;$i<100000;$i++){ $o->x = $o->y + 1; $s += $o->x; } echo $s,"
";
