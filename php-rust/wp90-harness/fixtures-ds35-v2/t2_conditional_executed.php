<?php
echo "pre|";
if (true) {
    class C extends P { public function m(int $x) { return $x; } }
}
class P { public function m(int|string $x) { return $x; } }
echo "post";
