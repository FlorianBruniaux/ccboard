#!/usr/bin/env bash
# notification.sh — Hook Notification: budget alerts + task completion
set -euo pipefail

INPUT=$(cat)
TITLE=$(echo "$INPUT" | jq -r '.title // "Claude Code"' 2>/dev/null || echo "Claude Code")
MESSAGE=$(echo "$INPUT" | jq -r '.message // ""' 2>/dev/null || echo "")

[[ -z "$MESSAGE" ]] && exit 0

# macOS: osascript notification (silent fail on non-macOS)
osascript -e "display notification \"${MESSAGE}\" with title \"${TITLE}\"" 2>/dev/null || true
exit 0
