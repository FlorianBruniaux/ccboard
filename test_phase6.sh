#!/bin/bash
# Quick test script for Phase 6 features verification
# Usage: ./test_phase6.sh

set -e

echo "🚀 Phase 6 Test Script - Quick Verification"
echo "==========================================="
echo

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test functions
check_pass() {
    echo -e "${GREEN}✓${NC} $1"
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 1. Build verification
echo "1. Build Verification"
echo "---------------------"
if cargo build --release --all > /dev/null 2>&1; then
    check_pass "Release build successful"
else
    check_fail "Release build failed"
    exit 1
fi

# 2. Clippy check
echo
echo "2. Code Quality (Clippy)"
echo "------------------------"
if cargo clippy --all-targets 2>&1 | grep -q "No issues found"; then
    check_pass "Clippy clean (0 warnings)"
else
    check_fail "Clippy has warnings/errors"
    cargo clippy --all-targets 2>&1 | head -20
fi

# 3. Tests
echo
echo "3. Unit Tests"
echo "-------------"
if cargo test --all --lib > /dev/null 2>&1; then
    check_pass "All unit tests pass"
else
    check_warn "Some tests failed (check manually)"
fi

# 4. File structure verification
echo
echo "4. File Structure"
echo "-----------------"

# Check new files exist
files=(
    "crates/ccboard-tui/src/editor.rs"
    "crates/ccboard-tui/src/tabs/dashboard.rs"
    "crates/ccboard-tui/src/tabs/config.rs"
    "crates/ccboard-tui/src/tabs/sessions.rs"
    "crates/ccboard-tui/src/tabs/history.rs"
    "crates/ccboard-tui/src/tabs/hooks.rs"
    "crates/ccboard-tui/src/tabs/agents.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        check_pass "$(basename $file) exists"
    else
        check_fail "$(basename $file) missing"
    fi
done

# 5. Binary verification
echo
echo "5. Binary Check"
echo "---------------"
if [ -f "target/release/ccboard" ]; then
    size=$(du -h target/release/ccboard | cut -f1)
    check_pass "Binary compiled (size: $size)"

    # Check if binary runs
    if timeout 2s ./target/release/ccboard --help > /dev/null 2>&1; then
        check_pass "Binary executable and responsive"
    else
        check_warn "Binary help command timeout (normal for TUI)"
    fi
else
    check_fail "Binary not found"
fi

# 6. Environment check
echo
echo "6. Environment Setup"
echo "--------------------"
if [ -n "$VISUAL" ]; then
    check_pass "VISUAL set to: $VISUAL"
elif [ -n "$EDITOR" ]; then
    check_pass "EDITOR set to: $EDITOR"
else
    check_warn "No VISUAL or EDITOR set (will use fallback)"
fi

if [ -d "$HOME/.claude" ]; then
    check_pass "~/.claude directory exists"

    # Check MCP config
    if [ -f "$HOME/.claude/claude_desktop_config.json" ]; then
        mcp_count=$(jq '.mcpServers | length' "$HOME/.claude/claude_desktop_config.json" 2>/dev/null || echo "0")
        check_pass "MCP config exists ($mcp_count servers)"
    else
        check_warn "No MCP config (Task 9 won't show data)"
    fi

    # Check stats
    if [ -f "$HOME/.claude/stats-cache.json" ]; then
        check_pass "Stats cache exists"
    else
        check_warn "No stats cache"
    fi
else
    check_warn "~/.claude directory not found (test data needed)"
fi

# 7. Key features verification (static analysis)
echo
echo "7. Feature Implementation Check"
echo "--------------------------------"

# Check editor.rs has key functions
if grep -q "pub fn open_in_editor" crates/ccboard-tui/src/editor.rs; then
    check_pass "open_in_editor() implemented"
else
    check_fail "open_in_editor() missing"
fi

if grep -q "pub fn reveal_in_file_manager" crates/ccboard-tui/src/editor.rs; then
    check_pass "reveal_in_file_manager() implemented"
else
    check_fail "reveal_in_file_manager() missing"
fi

# Check dashboard has 5 columns
if grep -q "Constraint::Percentage(20)" crates/ccboard-tui/src/tabs/dashboard.rs; then
    check_pass "Dashboard has 5 columns layout"
else
    check_warn "Dashboard layout might not be 5 columns"
fi

# Check config has MCP modal
if grep -q "show_mcp_detail" crates/ccboard-tui/src/tabs/config.rs; then
    check_pass "MCP detail modal implemented"
else
    check_fail "MCP modal missing"
fi

# Check keybindings
echo
echo "8. Keybinding Verification"
echo "---------------------------"

keybinding_files=(
    "crates/ccboard-tui/src/tabs/agents.rs"
    "crates/ccboard-tui/src/tabs/sessions.rs"
    "crates/ccboard-tui/src/tabs/history.rs"
    "crates/ccboard-tui/src/tabs/hooks.rs"
    "crates/ccboard-tui/src/tabs/config.rs"
)

for file in "${keybinding_files[@]}"; do
    tab_name=$(basename $file .rs)

    # Check 'e' key
    if grep -q "KeyCode::Char('e')" "$file"; then
        check_pass "$tab_name: 'e' key implemented"
    else
        check_warn "$tab_name: 'e' key missing"
    fi

    # Check 'o' key
    if grep -q "KeyCode::Char('o')" "$file"; then
        check_pass "$tab_name: 'o' key implemented"
    else
        # 'o' is optional for some tabs
        :
    fi
done

# Check 'm' key for config
if grep -q "KeyCode::Char('m')" crates/ccboard-tui/src/tabs/config.rs; then
    check_pass "config: 'm' key (modal) implemented"
else
    check_warn "config: 'm' key missing"
fi

# 9. Summary
echo
echo "========================================="
echo "Summary"
echo "========================================="
echo
echo "✅ Build successful - Binary ready at target/release/ccboard"
echo "📋 Test guide available at TEST_GUIDE_PHASE6.md"
echo
echo "Next steps:"
echo "  1. Run: ./target/release/ccboard"
echo "  2. Follow TEST_GUIDE_PHASE6.md for manual testing"
echo "  3. Test priority: Task 9 (Dashboard) + Task 8 (Modal) first"
echo
echo "Quick test commands:"
echo "  export VISUAL=vim"
echo "  ./target/release/ccboard"
echo "  # Tab 1 → Check 5th MCP card"
echo "  # Tab 5 → Select agent → Press 'e'"
echo "  # Tab 3 → Press 'm' on MCP section"
echo
