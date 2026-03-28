#!/bin/bash
# Phase 0 — Test hook for decision:block mechanism
#
# PURPOSE: Validate that Claude Code Stop hooks can:
#   1. Return decision:"block" to prevent session close
#   2. Inject a prompt via additionalContext
#   3. Receive Claude's response and capture it
#
# HOW TO TEST:
#   1. Temporarily register this hook in .claude/settings.json:
#      "Stop": [{"hooks": [{"type": "command", "command": "bash /absolute/path/to/session-stop-test.sh"}]}]
#   2. Run a session, make 3+ edits, then stop
#   3. Observe: does Claude respond with "BRAIN TEST: ..." ?
#   4. Check if a second Stop fires after Claude's response
#
# EXPECTED BEHAVIOR (if decision:block works as in Claude Pulse):
#   - Claude sees the additionalContext prompt
#   - Claude responds with "BRAIN TEST: ..." in the conversation
#   - Session stops only after user explicitly quits again
#
# Log to file for debugging (does NOT interfere with stdout JSON)
LOG="$HOME/.ccboard/hook-test.log"
mkdir -p "$HOME/.ccboard"
echo "[$(date)] session-stop-test.sh invoked" >> "$LOG"

INPUT="$(cat)"
SESSION_ID="$(echo "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
CWD="$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)"

echo "[$(date)] session_id=$SESSION_ID cwd=$CWD" >> "$LOG"
echo "[$(date)] full input: $INPUT" >> "$LOG"

# Guard: prevent infinite loop if called multiple times
GUARD="$HOME/.ccboard/.test_guard_${SESSION_ID}"
if [ -f "$GUARD" ]; then
    echo "[$(date)] Guard hit — second Stop call, parsing response..." >> "$LOG"
    # TODO Phase 2: read JSONL to find PROGRESS/DECISION/BLOCKED response
    rm -f "$GUARD"
    exit 0
fi
touch "$GUARD"

# Return decision:block with a minimal test prompt
printf '{"decision":"block","hookSpecificOutput":{"additionalContext":"BRAIN TEST: Reply with exactly this line: BRAIN TEST: confirmed"}}\n'
echo "[$(date)] Returned decision:block" >> "$LOG"
