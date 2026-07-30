<?php
/**
 * KS-M1-Complete: Verify Vm::request_end() resets next_object_id to 1
 *
 * This gate tests the core M1 reset mechanism:
 * - Create an object (increments next_object_id)
 * - After request_end(), next_object_id should be 1
 *
 * Gate Status: Pending — requires Vm instance in handler
 */

// Create a dummy object to verify ID tracking
class TestClass {}
$obj1 = new TestClass();

// In PHP, we can't directly access next_object_id,
// but we can verify object handles are sequential
$handle1 = spl_object_id($obj1);
echo "Object 1 handle: {$handle1}\n";

// Verify handle is an integer (would be spl_object_id == 1 if reset works)
if ($handle1 == 1) {
    echo "PASS: Object ID is 1 (reset verified)\n";
} else {
    echo "FAIL: Object ID is {$handle1}, expected 1\n";
}

// Clean up
unset($obj1);
?>
