#!/usr/bin/env bash
# Rust Patterns Anti-Pattern Checker
# Usage: ./scripts/check-rust-patterns.sh [--strict]
#
# Exits with non-zero if critical anti-patterns are detected.
# Use --strict to fail on any anti-pattern (including medium severity).

set -euo pipefail

# Colors
RED='\033[0;31m'
YELLOW='\033[0;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Flags
STRICT=false
if [[ "${1:-}" == "--strict" ]]; then
    STRICT=true
fi

# Counters
CRITICAL=0
HIGH=0
MEDIUM=0

echo -e "${BLUE}🔍 Scanning for Rust anti-patterns...${NC}\n"

# 1. Critical: unwrap in production code (exclude tests)
echo -e "${YELLOW}[CRITICAL] Checking for .unwrap() in production code...${NC}"
UNWRAP_COUNT=$(grep -r "\.unwrap()" --include="*.rs" crates/ \
    --exclude-dir=tests \
    --exclude="*test*.rs" \
    --exclude="*bench*.rs" \
    | grep -v "unwrap_or" \
    | grep -v "#\[cfg(test)\]" \
    | wc -l | xargs)

if [[ "$UNWRAP_COUNT" -gt 0 ]]; then
    echo -e "${RED}❌ Found $UNWRAP_COUNT .unwrap() calls in production code${NC}"
    echo -e "${YELLOW}Top violators:${NC}"
    grep -r "\.unwrap()" --include="*.rs" crates/ \
        --exclude-dir=tests \
        --exclude="*test*.rs" \
        --exclude="*bench*.rs" \
        | grep -v "unwrap_or" \
        | grep -v "#\[cfg(test)\]" \
        | cut -d':' -f1 \
        | sort | uniq -c | sort -rn | head -5
    CRITICAL=$((CRITICAL + 1))
else
    echo -e "${GREEN}✅ No .unwrap() in production code${NC}"
fi

echo ""

# 2. High: expect in production code
echo -e "${YELLOW}[HIGH] Checking for .expect() in production code...${NC}"
EXPECT_COUNT=$(grep -r "\.expect(" --include="*.rs" crates/ \
    --exclude-dir=tests \
    --exclude="*test*.rs" \
    --exclude="*bench*.rs" \
    | grep -v "#\[cfg(test)\]" \
    | wc -l | xargs)

if [[ "$EXPECT_COUNT" -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  Found $EXPECT_COUNT .expect() calls in production code${NC}"
    HIGH=$((HIGH + 1))
else
    echo -e "${GREEN}✅ No .expect() in production code${NC}"
fi

echo ""

# 3. Medium: Clone chains
echo -e "${YELLOW}[MEDIUM] Checking for .clone().clone() chains...${NC}"
CLONE_CHAIN_COUNT=$(grep -r "\.clone().*\.clone()" --include="*.rs" crates/ | wc -l | xargs)

if [[ "$CLONE_CHAIN_COUNT" -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  Found $CLONE_CHAIN_COUNT clone chains${NC}"
    grep -r "\.clone().*\.clone()" --include="*.rs" crates/ | head -3
    MEDIUM=$((MEDIUM + 1))
else
    echo -e "${GREEN}✅ No clone chains detected${NC}"
fi

echo ""

# 4. Medium: Arc<Mutex> overuse
echo -e "${YELLOW}[MEDIUM] Checking for Arc<Mutex> usage...${NC}"
ARC_MUTEX_COUNT=$(grep -r "Arc<Mutex" --include="*.rs" crates/ | wc -l | xargs)

if [[ "$ARC_MUTEX_COUNT" -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  Found $ARC_MUTEX_COUNT Arc<Mutex> usages (consider RwLock or DashMap)${NC}"
    MEDIUM=$((MEDIUM + 1))
else
    echo -e "${GREEN}✅ No Arc<Mutex> usage (good! using DashMap/RwLock)${NC}"
fi

echo ""

# 5. Critical: std::thread::sleep in async code
echo -e "${YELLOW}[CRITICAL] Checking for std::thread::sleep in async code...${NC}"
BLOCKING_SLEEP_COUNT=$(grep -r "std::thread::sleep" --include="*.rs" crates/ \
    | grep -B 5 "async fn\|async move" \
    | grep "std::thread::sleep" \
    | wc -l | xargs)

if [[ "$BLOCKING_SLEEP_COUNT" -gt 0 ]]; then
    echo -e "${RED}❌ Found $BLOCKING_SLEEP_COUNT blocking sleep in async code${NC}"
    CRITICAL=$((CRITICAL + 1))
else
    echo -e "${GREEN}✅ No blocking sleep in async code${NC}"
fi

echo ""

# 6. Bonus: Check for missing context on ? operators (heuristic)
echo -e "${YELLOW}[HIGH] Checking for ? without .context() (heuristic)...${NC}"
NO_CONTEXT_COUNT=$(grep -r ")?;" --include="*.rs" crates/ccboard-core/src/parsers/ \
    | grep -v "\.context" \
    | grep -v "\.with_context" \
    | wc -l | xargs)

if [[ "$NO_CONTEXT_COUNT" -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  Found ~$NO_CONTEXT_COUNT potential ? without context in parsers/${NC}"
    echo -e "${YELLOW}   (May include false positives)${NC}"
    HIGH=$((HIGH + 1))
else
    echo -e "${GREEN}✅ Most ? operators have context${NC}"
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${RED}Critical issues: $CRITICAL${NC}"
echo -e "${YELLOW}High issues: $HIGH${NC}"
echo -e "${YELLOW}Medium issues: $MEDIUM${NC}"
echo ""

# Exit codes
if [[ "$CRITICAL" -gt 0 ]]; then
    echo -e "${RED}❌ FAIL: Critical anti-patterns detected${NC}"
    echo -e "${YELLOW}Fix these before committing to main branch${NC}"
    exit 1
elif [[ "$HIGH" -gt 0 ]] && [[ "$STRICT" == true ]]; then
    echo -e "${YELLOW}⚠️  FAIL (strict mode): High-severity issues detected${NC}"
    exit 1
elif [[ "$MEDIUM" -gt 0 ]] && [[ "$STRICT" == true ]]; then
    echo -e "${YELLOW}⚠️  FAIL (strict mode): Medium-severity issues detected${NC}"
    exit 1
else
    echo -e "${GREEN}✅ PASS: No critical anti-patterns detected${NC}"
    exit 0
fi
