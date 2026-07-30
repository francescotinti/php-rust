<?php
/**
 * G2: Two-requests-same-Vm byte-parity gate (WP-77.5)
 *
 * Verifies that two consecutive HTTP requests on the SAME Vm instance
 * produce byte-identical output. This tests the request_end() reset mechanism:
 * - Request 1: output captured
 * - Request 2: output captured
 * - Compare: must be identical (no residual state leaking)
 *
 * Gate verdict: PASS if both outputs are identical
 * Kill-switch KS-P77-A: triggers if divergence detected
 */

$leaks = [];

// Verify ephemeral state is reset (should NOT exist on second request)
if (isset($GLOBALS['g2_marker'])) {
    $leaks[] = "LEAK:g2_marker";
}

// Verify static variables reset per-request (should be 1 on both requests)
function g2_static_test() {
    static $counter = 0;
    return ++$counter;
}

$c = g2_static_test();
if ($c !== 1) {
    $leaks[] = "LEAK:static_counter=$c";
}

// Create an object and verify ID is consistent
class G2TestClass {
    public $data;
    public function __construct($value) {
        $this->data = $value;
    }
}

$obj1 = new G2TestClass("test_data");
$id1 = spl_object_id($obj1);

// Verify object ID is 1 (reset after request_end)
if ($id1 !== 1) {
    $leaks[] = "LEAK:object_id=$id1";
}

// Deterministic output (identical on both requests)
if (empty($leaks)) {
    // Fixed output for byte-parity check
    echo "G2:PASS:object_id=1\n";
} else {
    echo "G2:FAIL:" . implode(";", $leaks) . "\n";
}

// Stamp state for next request to (not) find
$GLOBALS['g2_marker'] = "marked";
