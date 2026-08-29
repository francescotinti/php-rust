<?php
// S-162 fixture INVARIANZA L-AM2 — forme string-callable con divergenza
// PRE-esistente del pin vs oracle (scoperte al collaudo fixture s162, PRIMA
// della leva): (1) user-fn by-ref via stringa: l'oracle avvisa "must be
// passed by reference" e procede, phpr resta muto; (2) callback undefined:
// oracle TypeError "not a valid callback", phpr Error "undefined function".
// GATE: pin(B) == gemelloA BYTE-ID (la leva NON le tocca per costruzione:
// by-ref non e' simple_call; undefined non risolve). Catalogo: da annotare
// in PHPR_DIVERGENCES_FROM_PHP.md.
error_reporting(E_ALL);
function g4(&$x) { $x++; return $x; }
var_dump(array_map('g4', [1, 2]));
try { array_map('no_such_fn_sm', [1]); } catch (Error $e) { echo get_class($e), ": ", $e->getMessage(), "\n"; }
echo "FX-SM-DIV DONE\n";
