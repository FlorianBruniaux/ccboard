#!/bin/bash
# Brain hook — session-start.sh
# Fires on every new session start. Injects recent Brain insights as context
# so Claude is aware of active blockers, last progress, and known patterns.
#
# Only fires if ~/.ccboard/insights.db exists (no-op otherwise).
# Context is injected via hookSpecificOutput.additionalContext.

set -euo pipefail

sql_escape() { printf '%s' "$1" | sed "s/'/''/g"; }

INPUT="$(cat)"
SESSION_ID="$(echo "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
PROJECT="$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)"
DB_PATH="$HOME/.ccboard/insights.db"
LOG="$HOME/.ccboard/hook-start.log"

mkdir -p "$HOME/.ccboard"

# Bail early if no DB, jq, or sqlite3
[ -f "$DB_PATH" ] || exit 0
command -v jq >/dev/null 2>&1 || exit 0
command -v sqlite3 >/dev/null 2>&1 || exit 0
[ -z "$PROJECT" ] && exit 0

# Guard: inject context only once per session (first PreToolUse call)
GUARD="$HOME/.ccboard/.ctx_injected_${SESSION_ID}"
if [ -f "$GUARD" ]; then
    exit 0  # Already injected for this session
fi
touch "$GUARD"

echo "[$(date)] session-start.sh: project=$PROJECT" >> "$LOG"

esc_project="$(sql_escape "$PROJECT")"

# Last progress (1 line)
last_progress="$(sqlite3 "$DB_PATH" \
    "SELECT content FROM insights
     WHERE project='$esc_project' AND type='progress' AND archived=0
     ORDER BY created_at DESC LIMIT 1" 2>/dev/null || true)"

# Active blockers (up to 2)
blockers="$(sqlite3 "$DB_PATH" \
    "SELECT content FROM insights
     WHERE project='$esc_project' AND type='blocked' AND archived=0
     ORDER BY created_at DESC LIMIT 2" 2>/dev/null || true)"

# Known patterns & fixes (up to 3)
knowledge="$(sqlite3 "$DB_PATH" \
    "SELECT type || ': ' || content FROM insights
     WHERE project='$esc_project' AND type IN ('pattern','fix','context')
     AND archived=0 ORDER BY created_at DESC LIMIT 3" 2>/dev/null || true)"

# Build context string
context=""
[ -n "$last_progress" ] && context="${context}## Last session\n${last_progress}\n\n"
[ -n "$blockers" ]      && context="${context}## Active blockers\n${blockers}\n\n"
[ -n "$knowledge" ]     && context="${context}## Project knowledge\n${knowledge}\n"

if [ -z "$context" ]; then
    echo "[$(date)] No context found for project" >> "$LOG"
    exit 0
fi

echo "[$(date)] Injecting context" >> "$LOG"

# Escape context for JSON string (newlines → \n, quotes → \")
escaped="$(printf '%s' "$context" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))' 2>/dev/null \
    || printf '%s' "$context" | sed 's/\\/\\\\/g; s/"/\\"/g' | tr '\n' ' ')"

printf '{"hookSpecificOutput":{"additionalContext":%s}}\n' "$escaped"
