<?php
// A-BB10: include-heavy census fixture, lib 2/5 — static-heavy class +
// interface + trait (compile-side breadth, deterministic).

interface Heavy2Step {
    public function apply(int $x): int;
}

trait Heavy2Mix {
    public function mix(int $a, int $b): int {
        return (($a << 3) ^ ($b >> 1)) & 0x7FFFFFFF;
    }
}

class Heavy2 implements Heavy2Step {
    use Heavy2Mix;

    public const SALT = 7919;

    public static function step(int $x): int {
        $x = ($x + self::SALT) % 1000003;
        return ($x * $x) % 999983;
    }

    public function apply(int $x): int {
        return $this->mix(self::step($x), $x);
    }
}

class Heavy2Chain {
    /** @var Heavy2Step[] */
    private array $steps = [];

    public function push(Heavy2Step $s): static {
        $this->steps[] = $s;
        return $this;
    }

    public function run(int $seed): int {
        $v = $seed;
        foreach ($this->steps as $s) {
            $v = $s->apply($v);
        }
        return $v;
    }
}

function heavy2_series(int $n): array {
    $out = [];
    for ($i = 0; $i < $n; $i++) {
        $out[] = Heavy2::step($i);
    }
    return $out;
}
