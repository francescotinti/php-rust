<?php
// Trappola (e) — ordine osservabile: effetti del RHS PRIMA, warning
// undef-lhs POI, errore dell'op per ULTIMO.
function eff() { echo "eff-ran\n"; return [1]; }
try { $u += eff(); } catch (\Throwable $e) { echo "caught:", get_class($e), "\n"; }
var_dump(isset($u));
function eff2() { echo "eff2-ran\n"; return 5; }
$w += eff2();
echo "w:", $w, "\n";
