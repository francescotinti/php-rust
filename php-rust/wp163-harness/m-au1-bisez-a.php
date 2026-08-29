<?php // bisezione census: loader CLOSURE NON-simple (2 param, default) -> generico con vec![arg]; attesa 1/miss
spl_autoload_register(function ($c, $x = 0) { /* no-op */ });
$cn = 'Doctrine\\Tests\\Models\\CMS\\MissingProbeEntity';
$acc = 0;
for ($i = 0; $i < 200000; $i++) { if (!class_exists($cn)) { $acc++; } }
echo "BZA-OK $acc\n";
