<?php
// A-BB10/KB-80-1 (Council WP-80): the SECOND census fixture — include-heavy
// + compute (WP proxy). hello.php's phase b is trivially empty, so ANY fixed
// cost in phase a dominates in percentage; this fixture gives the b channel
// real volume (5 included units + templating + aggregation + tree eval) so
// the a1/a2/a3 split is judged on ABSOLUTES, never on the hello "99%".
// Deterministic output: byte-comparable against the oracle CLI.

require __DIR__ . '/heavy_lib1.php';
require __DIR__ . '/heavy_lib2.php';
require __DIR__ . '/heavy_lib3.php';
require __DIR__ . '/heavy_lib4.php';
require __DIR__ . '/heavy_lib5.php';

// 1. numeric churn over the lib functions
$c = new Heavy1Counter();
$series = heavy2_series(400);
foreach ($series as $i => $v) {
    $c->bump(heavy1_calc($v + $i));
}

// 2. grid + chain (object dispatch)
$grid = new Heavy1Grid(24, 18);
$chain = (new Heavy2Chain())->push(new Heavy2())->push(new Heavy2());
$chained = $chain->run($grid->checksum());

// 3. string/templating volume (the WP-shaped channel)
$html = heavy3_rows(120);
$buf = new Heavy3Buffer();
$buf->add($html);
$buf->add(heavy3_slug('Heavy Fixture Page Title ' . $chained));

// 4. registry + buckets
$reg = new Heavy4Registry();
$reg->on('alpha', fn (int $x): int => heavy1_calc($x));
$reg->on('beta', fn (int $x): int => Heavy2::step($x));
$fired = $reg->fire('alpha', $chained) + $reg->fire('beta', $c->total());
$buckets = heavy4_bucket($series, 7);
$stats = heavy4_stats(heavy4_flatten([$buckets, [$fired]]));

// 5. tree eval (recursion + late binding)
$tree = heavy5_tree(6, 3);

echo 'H:', $c->digest(),
    ';grid=', $grid->checksum(),
    ';chain=', $chained,
    ';html=', strlen($html), ':', $buf->hash(),
    ';stats=', $stats['n'], ':', $stats['sum'], ':', $stats['min'], ':', $stats['max'],
    ';tree=', $tree->eval(),
    ';hooks=', implode(',', $reg->names()),
    "\n";
