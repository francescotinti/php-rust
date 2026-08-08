<?php
class C { public $x = 0; public $y = 1; }
$o = new C;
$r = &$o->y;
$o->x = $o->y + 1;
$r = 9;
$o->x = $o->y + 1;
echo $o->x, " ", $o->y, "\n";
