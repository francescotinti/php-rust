<?php
interface I {
    public function __construct(int $x);
}
class Mid implements I {
    public function __construct(int $x) {}
}
class C extends Mid {
    public function __construct(string $y) {}
}
