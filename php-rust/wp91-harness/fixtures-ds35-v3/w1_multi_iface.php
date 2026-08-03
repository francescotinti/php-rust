<?php
interface I1 { public function m(): int; }
interface I2 { public function m(): string; }
class C implements I1, I2 { public function m(): int { return 1; } }
echo "unreachable";
