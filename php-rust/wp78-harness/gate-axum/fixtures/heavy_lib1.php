<?php
// A-BB10 (Council WP-80): include-heavy census fixture, lib 1/5 — real code
// volume (functions + classes), deterministic, no I/O.

const HEAVY1_PRIME = 977;

function heavy1_calc(int $n): int {
    $acc = $n % HEAVY1_PRIME;
    $acc = ($acc * 31 + 7) % 1000003;
    return $acc ^ ($n & 0xFF);
}

function heavy1_fold(array $xs): int {
    $acc = 1;
    foreach ($xs as $x) {
        $acc = ($acc * 33 + (int)$x) % 1000000007;
    }
    return $acc;
}

function heavy1_render(int $v): string {
    return str_pad(dechex($v), 8, '0', STR_PAD_LEFT);
}

class Heavy1Counter {
    private int $n = 0;
    private array $log = [];

    public function bump(int $by): int {
        $this->n += $by;
        $this->log[] = $by;
        return $this->n;
    }

    public function total(): int {
        return $this->n;
    }

    public function digest(): string {
        return heavy1_render(heavy1_fold($this->log));
    }
}

class Heavy1Grid {
    /** @var int[][] */
    private array $rows = [];

    public function __construct(private int $w, private int $h) {
        for ($y = 0; $y < $h; $y++) {
            $row = [];
            for ($x = 0; $x < $w; $x++) {
                $row[] = heavy1_calc($x * 31 + $y);
            }
            $this->rows[] = $row;
        }
    }

    public function checksum(): int {
        $acc = 0;
        foreach ($this->rows as $row) {
            $acc = ($acc + heavy1_fold($row)) % 1000000007;
        }
        return $acc;
    }
}
