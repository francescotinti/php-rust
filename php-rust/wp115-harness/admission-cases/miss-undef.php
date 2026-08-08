<?php
class C { public $x = 0; }
$o = new C;
$o->x = $o->zzz + 1;
echo $o->x, "\n";
