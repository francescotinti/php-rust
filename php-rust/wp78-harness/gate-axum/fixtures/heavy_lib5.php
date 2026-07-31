<?php
// A-BB10: include-heavy census fixture, lib 5/5 — inheritance chain +
// abstract dispatch (method-table compile weight).

abstract class Heavy5Node {
    public function __construct(protected int $v) {}

    abstract public function eval(): int;

    public function label(): string {
        return static::class . '(' . $this->v . ')';
    }
}

class Heavy5Leaf extends Heavy5Node {
    public function eval(): int {
        return $this->v;
    }
}

class Heavy5Double extends Heavy5Leaf {
    public function eval(): int {
        return parent::eval() * 2;
    }
}

class Heavy5Sum extends Heavy5Node {
    /** @var Heavy5Node[] */
    private array $kids = [];

    public function add(Heavy5Node $n): static {
        $this->kids[] = $n;
        return $this;
    }

    public function eval(): int {
        $acc = $this->v;
        foreach ($this->kids as $k) {
            $acc = ($acc + $k->eval()) % 1000000007;
        }
        return $acc;
    }
}

function heavy5_tree(int $depth, int $seed): Heavy5Node {
    if ($depth <= 0) {
        return $seed % 3 === 0 ? new Heavy5Double($seed) : new Heavy5Leaf($seed);
    }
    $sum = new Heavy5Sum($seed);
    for ($i = 0; $i < 3; $i++) {
        $sum->add(heavy5_tree($depth - 1, $seed * 7 + $i + 1));
    }
    return $sum;
}
