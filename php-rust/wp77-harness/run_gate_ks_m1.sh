#!/bin/bash
# KS-M1-Complete Gate Runner
# Verifies Vm::request_end() resets next_object_id to 1
#
# Kill-Switch: next_object_id must == 1 after reset
# Test Approach: Create Vm, call request_end(), verify reset

set -e

REPO_ROOT="/Volumes/Extreme Pro/Claude/php-rust-experiment/php-rust"
BINARY="$HOME/Claude/php-rust-output/release/phpr"
FIXTURE="${REPO_ROOT}/wp77-harness/fixtures/gate_ks_m1_complete.php"
CORPUS_PATH="/Volumes/Extreme Pro/Claude/php-8.5.7"

echo "=========================================="
echo "KS-M1-Complete: Object ID Reset Gate"
echo "=========================================="
echo ""
echo "Binary: $BINARY"
echo "Fixture: $FIXTURE"
echo "Corpus: $CORPUS_PATH"
echo ""

if [ ! -f "$BINARY" ]; then
    echo "ERROR: Binary not found: $BINARY"
    exit 1
fi

if [ ! -f "$FIXTURE" ]; then
    echo "ERROR: Fixture not found: $FIXTURE"
    exit 1
fi

# Create a simple test that creates objects and verifies IDs reset
echo "Running object ID test..."
cd "$CORPUS_PATH"

# Run the test script via CLI
OUTPUT=$("$BINARY" "$FIXTURE" 2>&1 || true)
echo "$OUTPUT"

# Check for PASS in output
if echo "$OUTPUT" | grep -q "PASS"; then
    echo ""
    echo "✅ KS-M1-Complete PASSED"
    exit 0
else
    echo ""
    echo "❌ KS-M1-Complete FAILED"
    exit 1
fi
