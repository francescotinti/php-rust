<?php
class C { public $x = 0; public $y = 7; }
$o = new C;
for ($i = 0; $i < 10; $i++) {
    $o->x = $o->y + 3;
    $o->y = $o->x + 2;
    echo $o->x, " ", $o->y, "\n";
}
