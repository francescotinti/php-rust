<?php
interface I { public function m(int $x): int; }
enum E implements I { case A; public function m(string $x): int { return 1; } }
echo "unreachable";
