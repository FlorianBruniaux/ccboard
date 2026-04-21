---
name: ccboard-remember
description: Store a fix, pattern, or context note in Brain knowledge base (~/.ccboard/insights.db). Use when you want to remember a fix, pattern, decision, or context note across sessions.
allowed-tools: Read, Write, Bash
effort: low
tags: [brain, knowledge, memory, ccboard]
---

# /ccboard-remember — Store Knowledge in Brain

Saves a piece of knowledge to `~/.ccboard/insights.db` for future session context injection.

## Usage

```
/ccboard-remember fix: <description>
/ccboard-remember pattern: <description>
/ccboard-remember context: <description>
/ccboard-remember decision: <description>
```

## Workflow

1. Parse the user's input prefix (`fix:`, `pattern:`, `context:`, `decision:`)
   - Default type is `context` if no prefix provided
2. Get the current project path with `pwd`
3. Distill the input into a clean 1-line summary (≤ 120 chars)
4. Run this exact command (substituting TYPE, SUMMARY, and ORIGINAL):

```bash
sqlite3 "$HOME/.ccboard/insights.db" \
  "PRAGMA journal_mode=WAL;
   CREATE TABLE IF NOT EXISTS insights (
     id INTEGER PRIMARY KEY AUTOINCREMENT,
     session_id TEXT, project TEXT NOT NULL,
     type TEXT NOT NULL, content TEXT NOT NULL,
     reasoning TEXT, archived INTEGER NOT NULL DEFAULT 0,
     created_at TEXT NOT NULL DEFAULT (datetime('now'))
   );
   INSERT INTO insights (project, type, content, reasoning, created_at)
   VALUES ('$(pwd)', 'TYPE', 'SUMMARY', 'ORIGINAL', datetime('now'));"
```

5. Confirm: `Stored [TYPE] — SUMMARY`

## Type Guide

| Prefix | Type | Use for |
|--------|------|---------|
| `fix:` | fix | A recurring bug and its solution |
| `pattern:` | pattern | A code pattern that works well in this project |
| `context:` | context | Background info Claude should know |
| `decision:` | decision | An architectural decision and why |
| (none) | context | Default when no prefix is given |

## Examples

```
/ccboard-remember fix: SQLite WAL mode required when hooks write concurrently with Rust reads
/ccboard-remember pattern: Use Arc<DataStore> + parking_lot::RwLock for all shared state
/ccboard-remember context: This project targets macOS only, no Windows support needed
/ccboard-remember decision: insights.db is separate from session-metadata.db to avoid CACHE_VERSION conflicts
```

## Safeguards

- NEVER run sqlite3 if the user input contains shell metacharacters (`;`, `|`, `` ` ``, `$()`)
  that weren't in the original input — escape properly
- sql_escape the content: replace `'` with `''` before inserting
- If `sqlite3` is not available, report the error and show the manual SQL to run
- Max content length: 500 chars (truncate with `…` if longer)

## Expected Outcome

One new row in `~/.ccboard/insights.db`, visible in the Brain tab (`cargo run`) and Brain web page (`/brain`).
