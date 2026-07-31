<?php
// A-DS8 fixture (S-78.1.5, Council WP-79 — Stogov c.5c): the RetainSet parking
// surface was exercised by NO gate — and the first gate run CAUGHT a real
// leak (per-worker arena grew 1/include/request, KS-DS-78-4; fixed with the
// per-request RetainSet pin). This script requires+includes two units: output
// must be byte-identical to the oracle on EVERY request (statics in included
// units restart per request); the leak-freedom side is asserted by the cargo
// test include_units_pinned_per_request_not_leaked.
require __DIR__ . '/inc_lib.php';
include __DIR__ . '/inc_lib2.php';
echo "INC:" . lib_tick() . lib_tick() . ";" . inc2_val() . "\n";
