<?php
class P { public int $x { get => 1; } }
class C extends P { public string $x { get => "a"; } }
echo "unreachable";
