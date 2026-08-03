<?php
interface I { public function __construct(int $x); }
class C implements I { public function __construct(string $y) {} }
echo "alive";
