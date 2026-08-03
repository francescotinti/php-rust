<?php
interface A { public function m(int $x): int; }
interface B extends A { public function m(string $x): int; }
echo "unreachable";
