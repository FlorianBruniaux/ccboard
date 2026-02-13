#!/usr/bin/env bash
# Test script for conversation export functionality

set -e

# Find a real session file
SESSION_FILE=$(find ~/.claude/projects -name "*.jsonl" -type f | head -1)

if [ -z "$SESSION_FILE" ]; then
    echo "No session files found in ~/.claude/projects"
    exit 1
fi

# Extract session ID from filename
SESSION_ID=$(basename "$SESSION_FILE" .jsonl)

echo "Testing conversation export with session: $SESSION_ID"
echo "Session file: $SESSION_FILE"
echo

# Create output directory
mkdir -p /tmp/ccboard-export-test

# Test Markdown export
echo "==> Testing Markdown export..."
cargo run --quiet --bin ccboard -- export "$SESSION_ID" -o /tmp/ccboard-export-test/conversation.md -f markdown 2>&1 | tail -5
echo

# Test JSON export
echo "==> Testing JSON export..."
cargo run --quiet --bin ccboard -- export "$SESSION_ID" -o /tmp/ccboard-export-test/conversation.json -f json 2>&1 | tail -5
echo

# Test HTML export
echo "==> Testing HTML export..."
cargo run --quiet --bin ccboard -- export "$SESSION_ID" -o /tmp/ccboard-export-test/conversation.html -f html 2>&1 | tail -5
echo

# Show file sizes
echo "==> Export results:"
ls -lh /tmp/ccboard-export-test/

# Open HTML in browser (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo
    echo "Opening HTML export in browser..."
    open /tmp/ccboard-export-test/conversation.html
fi
