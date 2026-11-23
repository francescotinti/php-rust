<?php
declare(strict_types=1);
// S-194 package C1 behaviour teeth, strict_types unit: the CALL SITE's unit governs argument coercion, the function's own unit its return.
namespace App;
function sri($v): int { return $v; } function srf($v): float { return $v; } function srs($v): string { return $v; } function spi(int $a) { return $a; } function spf(float $a) { return $a; }
t(fn() => pi_("5")); t(fn() => pi_(5)); t(fn() => pf(5)); t(fn() => ps(new S)); t(fn() => pu(1.0)); t(fn() => pn(null)); t(fn() => pc(new B)); t(fn() => pd("7"));
t(fn() => sri("5")); t(fn() => sri(5)); t(fn() => srf(5)); t(fn() => srs(new S)); t(fn() => srs("ok")); t(fn() => ri("5")); t(fn() => spi(5)); t(fn() => spf(1)); t(fn() => (new R)->i("4"));
t(fn() => \array_map('App\spi', ["1"]));
