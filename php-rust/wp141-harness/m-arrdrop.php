<?php
// m-arrdrop — giudice teardown array PACKED (criterio s141-criterio-rd1.md):
// costruzione+teardown per iterazione di un array di 6 elementi misti
// (2 long vivi, 3 str dal pool costanti, 1 double). $s ancora l'array
// (niente DCE) e l'azzeramento esplicito colloca il teardown NEL loop.
$s = 0;
for ($i = 0; $i<10000000; $i++) {
    $a = [$i, "alpha", 3.5, "beta", $i + 1, "gamma"];
    $s += $a[0];
    $a = null;
}
echo $s, "\n";
