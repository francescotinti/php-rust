<?php
// A-BB10: include-heavy census fixture, lib 3/5 — string machinery
// (the WP-shaped workload: templating-ish concat + parsing).

function heavy3_slug(string $s): string {
    $s = strtolower($s);
    $s = preg_replace('/[^a-z0-9]+/', '-', $s);
    return trim($s, '-');
}

function heavy3_tpl(string $tpl, array $vars): string {
    $out = $tpl;
    foreach ($vars as $k => $v) {
        $out = str_replace('{' . $k . '}', (string)$v, $out);
    }
    return $out;
}

function heavy3_rows(int $n): string {
    $buf = '';
    for ($i = 0; $i < $n; $i++) {
        $buf .= heavy3_tpl('<li id="{id}" class="{cls}">{txt}</li>', [
            'id' => 'row-' . $i,
            'cls' => $i % 2 === 0 ? 'even' : 'odd',
            'txt' => heavy3_slug('Item Number ' . $i),
        ]);
    }
    return $buf;
}

class Heavy3Buffer {
    private array $parts = [];

    public function add(string $p): void {
        $this->parts[] = $p;
    }

    public function join(string $glue = ''): string {
        return implode($glue, $this->parts);
    }

    public function hash(): string {
        return substr(md5($this->join('|')), 0, 12);
    }
}
