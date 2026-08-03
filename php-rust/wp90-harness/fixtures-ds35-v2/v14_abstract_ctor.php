<?php
abstract class P { abstract public function __construct(int $x); }
class C extends P { public function __construct(string $y) {} }
echo "alive";
