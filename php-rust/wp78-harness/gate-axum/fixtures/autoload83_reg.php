<?php
// A-BB33/A-SK28 (Council WP-84): REGISTER-ONLY twin of autoload82.php —
// the same spl_autoload_register closure, but NO trigger: no unloaded
// class is ever touched, so the autoload never fires. Isolates the
// Δ_register share of the composite "+56 calls/req vs hello" upper bound
// (KB-84-3): include-HIT quota = Δ(autoload82) − Δ(this fixture), never a
// cross-fixture subtraction without this control.
spl_autoload_register(function ($c) { include strtolower($c) . "_al82.php"; });
echo "REG83";
