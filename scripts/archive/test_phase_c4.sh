#!/bin/bash
# Test script for Phase C.4: Sessions Tab Live Refresh
# Usage: ./scripts/test_phase_c4.sh

set -e

echo "🧪 Phase C.4 Test Suite"
echo "======================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0

# Test 1: Build
echo "📦 Test 1: Building project..."
if cargo build --all > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Build successful${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ Build failed${NC}"
    ((FAILED++))
    exit 1
fi
echo ""

# Test 2: All tests pass
echo "🧪 Test 2: Running all tests..."
if cargo test --all 2>&1 | grep -q "test result: ok"; then
    TOTAL=$(cargo test --all 2>&1 | grep "test result: ok" | awk '{print $4}')
    echo -e "${GREEN}✅ All tests passing ($TOTAL tests)${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ Some tests failed${NC}"
    ((FAILED++))
fi
echo ""

# Test 3: Clippy clean
echo "🔍 Test 3: Checking clippy warnings..."
CLIPPY_OUTPUT=$(cargo clippy --all-targets 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -q "warning:"; then
    WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:")
    echo -e "${RED}❌ Found $WARNINGS clippy warnings${NC}"
    ((FAILED++))
else
    echo -e "${GREEN}✅ Zero clippy warnings${NC}"
    ((PASSED++))
fi
echo ""

# Test 4: Check key files exist and have expected code
echo "📄 Test 4: Verifying key implementations..."

# Check sessions.rs has mark_refreshed
if grep -q "pub fn mark_refreshed" crates/ccboard-tui/src/tabs/sessions.rs; then
    echo -e "${GREEN}  ✅ mark_refreshed() found in sessions.rs${NC}"
    ((PASSED++))
else
    echo -e "${RED}  ❌ mark_refreshed() NOT found${NC}"
    ((FAILED++))
fi

# Check sessions.rs has format_time_ago
if grep -q "fn format_time_ago" crates/ccboard-tui/src/tabs/sessions.rs; then
    echo -e "${GREEN}  ✅ format_time_ago() found${NC}"
    ((PASSED++))
else
    echo -e "${RED}  ❌ format_time_ago() NOT found${NC}"
    ((FAILED++))
fi

# Check sessions.rs has render_refresh_notification
if grep -q "fn render_refresh_notification" crates/ccboard-tui/src/tabs/sessions.rs; then
    echo -e "${GREEN}  ✅ render_refresh_notification() found${NC}"
    ((PASSED++))
else
    echo -e "${RED}  ❌ render_refresh_notification() NOT found${NC}"
    ((FAILED++))
fi

# Check ui.rs calls mark_refreshed
if grep -q "mark_refreshed" crates/ccboard-tui/src/ui.rs; then
    echo -e "${GREEN}  ✅ mark_refreshed() called in ui.rs${NC}"
    ((PASSED++))
else
    echo -e "${RED}  ❌ mark_refreshed() NOT called in ui.rs${NC}"
    ((FAILED++))
fi

# Check Instant import
if grep -q "use std::time::Instant" crates/ccboard-tui/src/tabs/sessions.rs; then
    echo -e "${GREEN}  ✅ Instant imported${NC}"
    ((PASSED++))
else
    echo -e "${RED}  ❌ Instant NOT imported${NC}"
    ((FAILED++))
fi
echo ""

# Test 5: Documentation exists
echo "📚 Test 5: Checking documentation..."
if [ -f "TEST_GUIDE_PHASE_C4.md" ]; then
    echo -e "${GREEN}✅ Test guide exists${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ Test guide missing${NC}"
    ((FAILED++))
fi

if grep -q "Sessions Tab Live Refresh" CHANGELOG.md; then
    echo -e "${GREEN}✅ CHANGELOG updated${NC}"
    ((PASSED++))
else
    echo -e "${RED}❌ CHANGELOG not updated${NC}"
    ((FAILED++))
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All automated tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run manual tests from TEST_GUIDE_PHASE_C4.md"
    echo "2. Test the TUI with: cargo run"
    echo "3. Create a new session to test notifications"
    echo ""
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Review output above.${NC}"
    exit 1
fi
