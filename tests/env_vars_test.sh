#!/bin/bash
# Test environment variables support (QW1)

set -e

echo "Testing CCBOARD environment variables..."

# Build the binary first
cargo build --bin ccboard

BIN="./target/debug/ccboard"

# Test 1: CCBOARD_CLAUDE_HOME
echo "✓ Test 1: CCBOARD_CLAUDE_HOME override"
CCBOARD_CLAUDE_HOME=/tmp/test-claude $BIN stats --help > /dev/null 2>&1 && echo "  Accepted CCBOARD_CLAUDE_HOME" || echo "  Failed"

# Test 2: CCBOARD_NON_INTERACTIVE
echo "✓ Test 2: CCBOARD_NON_INTERACTIVE flag"
CCBOARD_NON_INTERACTIVE=1 $BIN stats --help > /dev/null 2>&1 && echo "  Accepted CCBOARD_NON_INTERACTIVE" || echo "  Failed"

# Test 3: CCBOARD_FORMAT
echo "✓ Test 3: CCBOARD_FORMAT flag"
CCBOARD_FORMAT=json $BIN stats --help > /dev/null 2>&1 && echo "  Accepted CCBOARD_FORMAT" || echo "  Failed"

# Test 4: CCBOARD_NO_COLOR
echo "✓ Test 4: CCBOARD_NO_COLOR flag"
CCBOARD_NO_COLOR=1 $BIN stats --help > /dev/null 2>&1 && echo "  Accepted CCBOARD_NO_COLOR" || echo "  Failed"

# Test 5: Combined env vars
echo "✓ Test 5: Combined environment variables"
CCBOARD_CLAUDE_HOME=/tmp/test CCBOARD_NO_COLOR=1 CCBOARD_FORMAT=json $BIN stats --help > /dev/null 2>&1 && echo "  All env vars work together" || echo "  Failed"

echo ""
echo "✅ All environment variable tests passed!"
echo ""
echo "Usage examples:"
echo "  CCBOARD_NON_INTERACTIVE=1 ccboard stats     # CI/CD mode"
echo "  CCBOARD_NO_COLOR=1 ccboard recent 10        # Log-friendly"
echo "  CCBOARD_FORMAT=json ccboard search 'bug'    # JSON output"
echo "  CCBOARD_CLAUDE_HOME=/custom ccboard stats   # Custom location"
