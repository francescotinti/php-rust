<?php
// S-194 probe: array_diff over objects with __toString (the shape PHPUnit's dependency sorter produces).
final class Dep {
    private string $c; private string $m;
    function __construct(string $c, string $m) { $this->c = $c; $this->m = $m; }
    function __toString(): string { return $this->c . '::' . $this->m; }
}
$r = []; for ($i = 0; $i < 2000; $i++) { $r[] = new Dep('Class' . ($i % 50), 'test' . $i); }
$l = []; for ($i = 0; $i < 20; $i++) { $l[] = new Dep('Class' . $i, 'test' . ($i * 7)); }
$n = 0; for ($k = 0; $k < 500; $k++) { $n += count(array_diff($l, $r)); }
echo $n, "\n";
