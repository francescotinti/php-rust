<?php
class M {
    private $d = ['y' => 4];
    public function __get($n) { echo "get:$n\n"; return $this->d[$n] ?? null; }
    public function __set($n, $v) { echo "set:$n=$v\n"; $this->d[$n] = $v; }
}
$m = new M;
$m->x = $m->y + 1;
echo $m->x, "\n";
