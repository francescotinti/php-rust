<?php
/**
 * KM-77-2 (HTTP form, WP-77.4.2): per-request $GLOBALS isolation over HTTP.
 *
 * Unlike the CLI fixture (which sets the marker to 'initial' and checks for
 * 'initial' — a leak would false-PASS), this version stamps each request with
 * a UNIQUE value. Any residue from a previous request is a real bite.
 */

$leaks = [];

// 1. Marker set by a previous request must NOT survive.
if (isset($GLOBALS['km77_marker'])) {
    $leaks[] = "km77_marker={$GLOBALS['km77_marker']}";
}

// 2. Plain global variable from a previous request must NOT survive.
if (isset($GLOBALS['km77_plain'])) {
    $leaks[] = "km77_plain={$GLOBALS['km77_plain']}";
}

// 3. Static-in-function counter: persists within a request, must restart at 0
//    for each new request (per-request statics are ephemeral request state).
function km77_counter() {
    static $n = 0;
    return ++$n;
}
$c1 = km77_counter();
$c2 = km77_counter();
if ($c1 !== 1 || $c2 !== 2) {
    $leaks[] = "static_counter c1=$c1 c2=$c2";
}

$uniq = 'req-' . getmypid() . '-' . bin2hex(random_bytes(6));

if ($leaks === []) {
    echo "PASS uniq=$uniq\n";
} else {
    echo "FAIL uniq=$uniq leaks=" . implode(';', $leaks) . "\n";
}

// Stamp state for the NEXT request to (not) find.
$GLOBALS['km77_marker'] = $uniq;
$km77_plain = $uniq;
$GLOBALS['km77_plain'] = $uniq;
