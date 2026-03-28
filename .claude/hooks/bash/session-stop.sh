#!/bin/bash
# Brain hook — session-stop.sh
# Fires on every session Stop. For meaningful sessions, asks Claude for a structured summary
# then stores the insights in ~/.ccboard/insights.db (via a second Stop pass).
#
# Smart gating: JSONL < 3KB = trivial session, skip silently.
# Guard file prevents infinite loop across double-Stop calls.

set -euo pipefail

# SQL injection protection
sql_escape() { printf '%s' "$1" | sed "s/'/''/g"; }

INPUT="$(cat)"
SESSION_ID="$(echo "$INPUT" | jq -r '.session_id // empty' 2>/dev/null)"
CWD="$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)"
DB_PATH="$HOME/.ccboard/insights.db"
LOG="$HOME/.ccboard/hook-stop.log"

mkdir -p "$HOME/.ccboard"
echo "[$(date)] session-stop.sh: session_id=$SESSION_ID cwd=$CWD" >> "$LOG"

# Bail early if jq/sqlite3 not available
command -v jq >/dev/null 2>&1 || exit 0
command -v sqlite3 >/dev/null 2>&1 || exit 0

# ──────────────────────────────────────────────────────────────
# GUARD: Second Stop call = Claude's response is in the JSONL.
#        Parse it and persist insights, then exit 0 (allow close).
# ──────────────────────────────────────────────────────────────
GUARD="$HOME/.ccboard/.summary_guard_${SESSION_ID}"
if [ -f "$GUARD" ]; then
    echo "[$(date)] Guard hit — second Stop, parsing Claude response..." >> "$LOG"
    rm -f "$GUARD"

    # Find the session JSONL
    JSONL_FILE=$(find "$HOME/.claude/projects" -name "${SESSION_ID}.jsonl" 2>/dev/null | head -1)
    if [ -z "$JSONL_FILE" ]; then
        echo "[$(date)] JSONL not found for $SESSION_ID" >> "$LOG"
        exit 0
    fi

    # Ensure insights table exists
    sqlite3 "$DB_PATH" "
        PRAGMA journal_mode=WAL;
        CREATE TABLE IF NOT EXISTS insights (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id  TEXT,
            project     TEXT NOT NULL,
            type        TEXT NOT NULL CHECK (type IN (
                            'progress','decision','blocked','pattern','fix','context')),
            content     TEXT NOT NULL,
            reasoning   TEXT,
            archived    INTEGER NOT NULL DEFAULT 0,
            created_at  TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_insights_project  ON insights(project);
        CREATE INDEX IF NOT EXISTS idx_insights_type     ON insights(type);
        CREATE INDEX IF NOT EXISTS idx_insights_created  ON insights(created_at);
        CREATE INDEX IF NOT EXISTS idx_insights_archived ON insights(archived);
    " 2>/dev/null || true

    esc_project="$(sql_escape "$CWD")"
    esc_session="$(sql_escape "$SESSION_ID")"

    # Extract the LAST assistant text message from the JSONL
    RESPONSE="$(tail -100 "$JSONL_FILE" 2>/dev/null \
        | jq -r 'select(.type == "assistant") | .message.content[]? | select(.type == "text") | .text' 2>/dev/null \
        | tail -1)"

    echo "[$(date)] Claude response: $RESPONSE" >> "$LOG"

    if [ -z "$RESPONSE" ]; then
        echo "[$(date)] No response found, skipping" >> "$LOG"
        exit 0
    fi

    # Parse PROGRESS / DECISION / BLOCKED lines
    PROGRESS="$(echo "$RESPONSE" | grep -i '^PROGRESS:' | sed 's/^PROGRESS:[[:space:]]*//' | head -1)"
    DECISION="$(echo "$RESPONSE" | grep -i '^DECISION:' | sed 's/^DECISION:[[:space:]]*//' | head -1)"
    BLOCKED="$(echo "$RESPONSE"  | grep -i '^BLOCKED:'  | sed 's/^BLOCKED:[[:space:]]*//'  | head -1)"

    insert_insight() {
        local type="$1"
        local content="$2"
        if [ -n "$content" ]; then
            local esc_content
            esc_content="$(sql_escape "$content")"
            sqlite3 "$DB_PATH" \
                "INSERT INTO insights (session_id, project, type, content, created_at)
                 VALUES ('$esc_session', '$esc_project', '$type', '$esc_content', datetime('now'));" \
                2>> "$LOG" || true
            echo "[$(date)] Inserted $type: $content" >> "$LOG"
        fi
    }

    insert_insight "progress" "$PROGRESS"
    insert_insight "decision" "$DECISION"
    insert_insight "blocked"  "$BLOCKED"

    exit 0
fi

# ──────────────────────────────────────────────────────────────
# FIRST Stop call — smart gating then ask for summary
# ──────────────────────────────────────────────────────────────

# Find session JSONL
JSONL_FILE=$(find "$HOME/.claude/projects" -name "${SESSION_ID}.jsonl" 2>/dev/null | head -1)
if [ -z "$JSONL_FILE" ]; then
    echo "[$(date)] JSONL not found, skipping" >> "$LOG"
    exit 0
fi

# Smart gating: file < 3KB = trivial session (read-only, quick check, etc.)
FILE_SIZE=$(stat -f%z "$JSONL_FILE" 2>/dev/null || stat -c%s "$JSONL_FILE" 2>/dev/null || echo 0)
echo "[$(date)] JSONL size: $FILE_SIZE bytes" >> "$LOG"
if [ "$FILE_SIZE" -lt 3072 ]; then
    echo "[$(date)] Trivial session (<3KB), skipping" >> "$LOG"
    exit 0
fi

# Place guard BEFORE returning decision:block to survive the second pass
touch "$GUARD"

echo "[$(date)] Meaningful session — requesting structured summary" >> "$LOG"

# Return decision:block to keep session open and prompt Claude for summary
printf '{
  "decision": "block",
  "hookSpecificOutput": {
    "additionalContext": "Session ending. Reply ONLY with these exact lines (skip any that do not apply):\nPROGRESS: <what was accomplished, 1 line>\nDECISION: <key choice + why, 1 line>\nBLOCKED: <what is stuck, 1 line>"
  }
}\n'
