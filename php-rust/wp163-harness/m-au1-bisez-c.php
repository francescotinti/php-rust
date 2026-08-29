<?php // bisezione census: chiamata DINAMICA di array-callable FUORI autoload ($cb($x)); conta vecargs del solo dispatch array
class BzC { public function m($x) { return $x; } }
$l = new BzC();
$cb = [$l, 'm'];
$acc = 0;
for ($i = 0; $i < 200000; $i++) { $acc += strlen($cb('ab')); }
echo "BZC-OK $acc\n";
