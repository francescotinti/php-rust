<?php
class C { public $x = 0; public $y = "5"; }
$o = new C;
$o->x = $o->y + 3;
echo $o->x, "\n";
