<?php
// A-BB10: include-heavy census fixture, lib 4/5 — array/aggregation code
// (hook-table shape: registries keyed by name, WP-style).

class Heavy4Registry {
    /** @var array<string, callable> */
    private array $hooks = [];

    public function on(string $name, callable $fn): void {
        $this->hooks[$name] = $fn;
    }

    public function fire(string $name, int $arg): int {
        if (!isset($this->hooks[$name])) {
            return 0;
        }
        return ($this->hooks[$name])($arg);
    }

    public function names(): array {
        $ns = array_keys($this->hooks);
        sort($ns);
        return $ns;
    }
}

function heavy4_bucket(array $xs, int $buckets): array {
    $out = array_fill(0, $buckets, 0);
    foreach ($xs as $x) {
        $out[((int)$x) % $buckets] += 1;
    }
    return $out;
}

function heavy4_stats(array $xs): array {
    $n = count($xs);
    if ($n === 0) {
        return ['n' => 0, 'sum' => 0, 'min' => 0, 'max' => 0];
    }
    return [
        'n' => $n,
        'sum' => array_sum($xs),
        'min' => min($xs),
        'max' => max($xs),
    ];
}

function heavy4_flatten(array $nested): array {
    $out = [];
    array_walk_recursive($nested, function ($v) use (&$out) {
        $out[] = $v;
    });
    return $out;
}
