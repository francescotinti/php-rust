<?php
// KB-82-5 (S-82.0 Phase A): autoload firing AT RUN — the include this
// triggers lands in the a3 window, so the census row bounds the RUN share
// of a3 ("HIT salta a3" stops being ADVISORY for this quota).
spl_autoload_register(function ($c) { include strtolower($c) . "_al82.php"; });
$o = new AutoCls82();
echo $o->v();
