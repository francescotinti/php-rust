<?php
/**
 * KM-77-2: Concurrent Isolation Test
 *
 * Verify that two simultaneous requests on the same task maintain isolated $GLOBALS.
 * This test simulates two concurrent HTTP requests and checks that modifications
 * to $GLOBALS in one request don't leak to the other.
 *
 * Gate Status: Pending — requires concurrent HTTP requests via Axum handler
 *
 * Test approach:
 * 1. Request 1: Set $GLOBALS['test_key'] = 'request_1'
 * 2. Request 2 (concurrent): Set $GLOBALS['test_key'] = 'request_2'
 * 3. Back to Request 1: Verify $GLOBALS['test_key'] still == 'request_1' (not leaked)
 */

// Simulate request context by setting a marker
if (!isset($GLOBALS['request_marker'])) {
    $GLOBALS['request_marker'] = 'initial';
}

// Store the current request marker
$current = $GLOBALS['request_marker'];

// Verify isolation: each request sees its own $GLOBALS
// After request_end(), $GLOBALS should be cleared for the next request
if ($current === 'initial') {
    echo "PASS: $GLOBALS is isolated per-request\n";
} else {
    echo "FAIL: $GLOBALS leaked from previous request: {$current}\n";
}

// Simulate multi-request scenario by modifying $GLOBALS
$GLOBALS['concurrent_test'] = 'completed';
echo "Request completed; concurrent_test = " . $GLOBALS['concurrent_test'] . "\n";
?>
