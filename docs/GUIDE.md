# ccboard User Guide

Complete feature reference for ccboard — the TUI and Web dashboard for Claude Code monitoring.

---

## Table of Contents

- [What ccboard does](#what-ccboard-does)
- [Installation](#installation)
- [First launch](#first-launch)
- [Global navigation](#global-navigation)
- [Tab reference](#tab-reference)
  - [1 — Dashboard](#1--dashboard)
  - [2 — Sessions](#2--sessions)
  - [3 — Analytics](#3--analytics)
  - [4 — Costs](#4--costs)
  - [5 — History](#5--history)
  - [6 — Audit Log](#6--audit-log)
  - [7 — MCP](#7--mcp)
  - [8 — Config](#8--config)
  - [9 — Hooks](#9--hooks)
  - [0 — Tools](#0--tools)
  - [p — Plugins](#p--plugins)
  - [/ — Search](#---search)
  - [13 — Brain](#13--brain)
- [Conversation viewer](#conversation-viewer)
- [Live session monitoring](#live-session-monitoring)
- [CLI reference](#cli-reference)
- [Export reference](#export-reference)
- [Configuration](#configuration)
- [Environment variables](#environment-variables)
- [Tips and tricks](#tips-and-tricks)

---

## What ccboard does

ccboard reads `~/.claude` (and optional project-level `.claude/`) directories to give you a unified view of your Claude Code activity. It works entirely offline — no API calls, no SaaS, no telemetry.

What you get:

- Token usage and cost breakdown by session, project, model, and day
- Full conversation replay with syntax highlighting and inline search
- Settings merge visualization across global/project/local levels
- Bash hooks management with syntax highlighting
- MCP server status and configuration
- Agents, commands, and skills browser
- Security audit: credential access detection, destructive command alerts
- FTS5 full-text search across all sessions
- Live session tracking (Running / WaitingInput / Stopped) via hook injection

---

## Installation

### Homebrew (macOS / Linux, recommended)

```bash
brew tap FlorianBruniaux/tap
brew install ccboard
```

### Cargo

```bash
cargo install ccboard
```

Note: `cargo install` builds without the WASM frontend. The web interface (`ccboard web`) won't serve a UI. Use Homebrew or a pre-built binary for full web support.

### Install script

```bash
curl -sSL https://raw.githubusercontent.com/FlorianBruniaux/ccboard/main/install.sh | bash
```

Downloads the correct pre-built binary for your OS and architecture.

---

## First launch

```bash
# Launch TUI
ccboard

# Enable live session monitoring (run once)
ccboard setup

# Launch web interface
ccboard web --port 3333
```

ccboard looks for `~/.claude` automatically. If your Claude data is elsewhere, use `--claude-home`:

```bash
ccboard --claude-home /path/to/.claude
```

---

## Global navigation

### Tab switching

| Key | Action |
|-----|--------|
| `1` | Dashboard |
| `2` | Sessions |
| `3` | Analytics |
| `4` | Costs |
| `5` | History |
| `6` | Audit Log |
| `7` | MCP |
| `8` | Config |
| `9` | Hooks |
| `0` | Tools |
| `p` | Plugins |
| `/` | Search |
| `b` | Brain |
| `Tab` / `Shift+Tab` | Next / previous tab |

### Universal keys

| Key | Action |
|-----|--------|
| `?` | Show contextual help for the active tab |
| `:` | Open command palette (fuzzy search all commands) |
| `q` | Quit |
| `r` | Refresh data |
| `F5` | Refresh data |
| `Ctrl+R` | Force refresh and clear SQLite cache |
| `Ctrl+T` | Toggle Dark / Light theme (persisted across sessions) |
| `Esc` | Close popup / go back |

### List navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Move left / collapse |
| `l` / `→` | Move right / expand |
| `PgUp` / `PgDn` | Page up / down (10 items) |
| `Enter` | Open detail / select |

### File operations (available where a file is selected)

| Key | Action |
|-----|--------|
| `e` | Edit in `$EDITOR` (falls back to `$VISUAL`, then nano / notepad) |
| `o` | Reveal in file manager (Finder on macOS, xdg-open on Linux, Explorer on Windows) |

### Command palette

Press `:` to open the command palette. Type to fuzzy-search available commands:

```
:dashboard    Jump to Dashboard tab
:sessions     Jump to Sessions tab
:analytics    Jump to Analytics tab
:costs        Jump to Costs tab
:config       Jump to Config tab
:mcp          Jump to MCP tab
:search       Jump to Search tab
:brain        Jump to Brain tab
:quit         Exit application
```

---

## Tab reference

### 1 — Dashboard

**Key**: `1`

Overview of your Claude Code usage at a glance.

**What you see:**

- Token counts for today and this week (input / output / cache read / cache write)
- Estimated API cost vs. your configured subscription plan
- 7-day activity sparkline
- Top models by token consumption
- Active MCP server count
- Monthly projection and budget status

**Budget color coding:**

| Color | Meaning |
|-------|---------|
| Green | Below 60% of monthly budget |
| Yellow | 60–80% of monthly budget |
| Red | Above 80% of monthly budget |

**Configuration**: Add `subscriptionPlan` to `~/.claude/settings.json` to enable accurate budget percentages. Supported values: `"pro"`, `"max5x"`, `"max20x"`, `"api"`.

---

### 2 — Sessions

**Key**: `2`

Three-pane layout: project tree on the left, session list in the middle, detail panel on the right.

**Navigation:**

| Key | Action |
|-----|--------|
| `h` / `l` | Switch focus between project tree, session list, detail panel |
| `j` / `k` | Move up/down in the focused pane |
| `Enter` | Open conversation viewer for selected session |
| `/` | Filter sessions by text |
| `b` | Toggle bookmark on the selected session |
| `B` | Toggle "bookmarked only" filter (show `★` sessions only) |
| `s` | Cycle sort mode (newest / oldest / tokens / duration / messages) |

**Session status indicators:**

| Icon | Meaning |
|------|---------|
| `●` | Running (active Claude process) |
| `◐` | Waiting for input / permission |
| `✓` | Completed |
| `★` | Bookmarked |

Live status requires `ccboard setup` (see [Live session monitoring](#live-session-monitoring)).

**Detail panel** shows:
- Session ID, timestamps, duration
- Token counts (input / output / cache read / cache write)
- Model switching timeline: `Opus 4.5 (8) → Sonnet 4.6 (15)` (computed at scan time)
- Message count, file size
- Subagent tree: `⤵ Subagents (N): X tokens total` with per-child breakdown; or `⤴ Subagent of: <parent_id>` for child sessions
- Bookmark tag and note (if bookmarked)
- AI Summary section (if cached via `ccboard summarize <id>`)
- First user message preview
- CPU / RAM usage for live sessions
- Session type: CLI, IDE, or Agent

**Bookmarks** persist to `~/.ccboard/bookmarks.json`. Each bookmark stores a tag (label), an optional note, and the creation date. Bookmarks survive restarts and are independent of Claude Code's own data.

**AI Summaries** are generated on demand:

```bash
ccboard summarize <session-id>           # Generate and cache
ccboard summarize <session-id> --force   # Regenerate
ccboard summarize <session-id> --model claude-haiku-4-5  # Faster/cheaper model
```

Once cached to `~/.ccboard/summaries/<id>.md`, the summary appears automatically in the detail panel.

**Conversation viewer** opens when you press `Enter` on a session. See [Conversation viewer](#conversation-viewer) for full details.

#### Code metrics

Sessions now show a `+N / -N` badge in the list and detail panel representing lines added and removed across all Edit and Write tool calls in that session. This metric is computed during session indexing and cached in SQLite.

#### Third-party AI tool sessions

ccboard automatically detects and imports sessions from other AI coding tools installed on your system. These sessions appear in the Sessions tab alongside Claude Code sessions, with a colored badge in the source column:

| Tool | Badge | Data source |
|------|-------|-------------|
| Claude Code | (none) | `~/.claude/projects/**/*.jsonl` |
| Cursor | `[Cu]` | `~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb` |
| Codex CLI | `[Cx]` | `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` |
| OpenCode | `[Oc]` | `~/Library/Application Support/OpenCode/opencode.db` (macOS) or `~/.local/share/opencode/opencode.db` (Linux) |

All parsers are opt-in and silent — if the tool is not installed, its parser is skipped without error. Session content from third-party tools is read-only.

---

### 3 — Analytics

**Key**: `3`

Six sub-views, switch with `Tab` / `←` / `→`:

| Sub-view | What it shows |
|----------|---------------|
| **Overview** | Budget status, MTD cost, monthly projection |
| **Trends** | 30-day token and cost trend chart |
| **Patterns** | Hourly heatmap of activity (fills terminal width), day-of-week distribution |
| **Insights** | Actionable suggestions based on usage patterns |
| **Anomalies** | Detected spikes and unusual activity with timestamps |
| **Forecast** | Token usage forecast with confidence bands |

The hourly heatmap is responsive: it uses your full terminal width and adjusts cell size accordingly.

**Budget tracking** configuration (in `~/.claude/settings.json` or `.claude/settings.json`):

```json
{
  "budget": {
    "monthlyBudgetUsd": 50.0,
    "alertThresholdPct": 80.0
  }
}
```

---

### 4 — Costs

**Key**: `4`

Six sub-views, switch with `Tab` / `←` / `→`:

| Sub-view | What it shows |
|----------|---------------|
| **Overview** | Total tokens and estimated cost, cache hit ratio |
| **By Model** | Token and cost breakdown per model |
| **Daily** | Bar chart of daily token consumption |
| **Usage Periods** | 5-hour billing window analysis |
| **Top Sessions** | Most expensive sessions ranked by cost |
| **Per Project** | Cost breakdown by project directory |

**4-level budget alerts** appear in the Overview sub-view:
1. Safe (below threshold)
2. Warning (at threshold)
3. Critical (significantly over)
4. Exceeded (monthly budget surpassed)

---

### 5 — History

**Key**: `5`

Chronological timeline of all sessions with search and export.

**Keys:**

| Key | Action |
|-----|--------|
| `/` | Search within history |
| `x` | Export current view to CSV / JSON / Markdown |
| `Enter` | Open session detail |
| `j` / `k` | Navigate |

**Export** prompts for format (CSV, JSON, Markdown) and destination path.

---

### 6 — Audit Log

**Key**: `6`

Security-focused view of session activity. Scans tool call history for risky patterns.

**What it detects:**

- Credential access: reads to `~/.ssh`, `~/.aws`, `.env` files, API key patterns
- Destructive commands: `rm -rf`, `DROP TABLE`, `git reset --hard`, etc.
- Cross-session violations: same pattern appearing across multiple sessions

**Keys:**

| Key | Action |
|-----|--------|
| `Tab` | Toggle between Sessions list and Violations feed |
| `Enter` | Analyze selected session individually |
| `r` | Batch-scan all sessions (4 concurrent) |
| `j` / `k` | Navigate list |

Each violation includes a remediation hint explaining what to check or change.

---

### 7 — MCP

**Key**: `7`

MCP server configuration and status.

**What you see:**

- Server name and command
- Status: running (detected via process check) or stopped
- Environment variables (sensitive values masked by default)
- Full launch command

**Keys:**

| Key | Action |
|-----|--------|
| `y` | Copy server launch command to clipboard |
| `e` | Edit MCP config file in `$EDITOR` |
| `Enter` | Show server detail |

---

### 8 — Config

**Key**: `8`

Visualizes how Claude Code settings merge across four levels.

**Four-column layout:**

| Column | Source |
|--------|--------|
| Default | Built-in Claude Code defaults |
| Global | `~/.claude/settings.json` |
| Project | `.claude/settings.json` |
| Local | `.claude/settings.local.json` |

The active (merged) value is highlighted. Overrides are visible at a glance.

**Keys:**

| Key | Action |
|-----|--------|
| `h` / `l` | Move focus between columns |
| `e` | Edit the focused config file |
| `o` | Reveal focused file in file manager |
| `m` | Show MCP detail modal |

---

### 9 — Hooks

**Key**: `9`

Bash hooks configured in your Claude Code settings, organized by event type.

**Event types**: `PreToolUse`, `PostToolUse`, `Notification`, `Stop`, `SubagentStop`

**What you see:**

- Hook name and event type
- Bash script content with syntax highlighting
- Badge indicators for hook properties
- File path

**Keys:**

| Key | Action |
|-----|--------|
| `Enter` | Show hook detail |
| `e` | Edit hook script in `$EDITOR` |

---

### 0 — Tools

**Key**: `0`

Browse agents, commands, and skills from your `.claude/` directories.

**Three sub-views** (switch with `Tab`):

| Sub-view | Source | What it shows |
|----------|--------|---------------|
| **Agents** | `.claude/agents/*.md` | Agent name, description, frontmatter YAML |
| **Commands** | `.claude/commands/*.md` | Command name, description, content |
| **Skills** | `.claude/skills/*.md` | Skill name, description, invocation stats |

Frontmatter is parsed and displayed separately from the Markdown body.

---

### p — Plugins

**Key**: `p`

Usage analytics for your skills, MCP servers, and agents.

**What you see:**

- Invocation counts over time
- Token consumption per plugin
- Cost attribution
- Dead code detection: plugins with zero usage in the analysis window

**Sort options** (press `s` to cycle):
- By usage count
- By token cost
- By name (alphabetical)

---

### / — Search

**Key**: `/` (or reach via `Tab`)

Full-text search across all sessions using SQLite FTS5.

**How it works:**

- Results appear as you type (after 2 characters)
- Ranked by relevance (BM25 via FTS5)
- Each result shows a highlighted snippet with the matching context
- Press `Enter` on any result to open the full conversation viewer

**Keys in Search tab:**

| Key | Action |
|-----|--------|
| Type | Search query (live, ≥2 chars) |
| `Enter` | Open selected session in conversation viewer |
| `j` / `k` | Navigate results |
| `Esc` | Clear search |

---

### 13 — Brain

**Key**: `b`

Cross-session knowledge base that captures learning from every session via hooks.

**How it works**

A `session-stop` hook runs when Claude Code finishes a session. It reads the session JSONL, evaluates its significance (sessions under 3KB are skipped as trivial), extracts key information via structured prompting, and stores insights in `~/.ccboard/insights.db` (WAL SQLite).

A `session-start` hook reads the most recent progress, blockers, and knowledge entries from insights.db and injects them into Claude's context at the start of each session, so Claude remembers where you left off without any manual context setting.

**Insight types**

| Type | Icon | Description |
|------|------|-------------|
| `progress` | 📍 | What was accomplished in the session |
| `decision` | 🏛️ | Architectural or design decisions made |
| `blocked` | 🚧 | Blockers encountered, unresolved issues |
| `pattern` | 🔁 | Recurring patterns worth remembering |
| `fix` | 🔧 | Bug fixes with root cause and solution |
| `context` | 📌 | General context and background knowledge |

**Key bindings (Brain tab)**

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate insight list |
| `Enter` | Expand/collapse detail pane |
| `f` | Cycle type filter (All → Progress → Decision → ...) |
| `a` | Archive selected insight |
| `r` | Refresh from database |

**Manual entries**

Use the `/ccboard-remember` skill to store knowledge manually from any session:

```
/ccboard-remember fixed the auth bug by setting SESSION_SECRET in .env
/ccboard-remember decided to use Axum over Actix for the web backend
```

**Database location**: `~/.ccboard/insights.db`

---

## Conversation viewer

Accessible from Sessions tab (press `Enter`) or Search results (press `Enter`).

**What it shows:**

- Full conversation replay: user messages, assistant responses, tool calls
- Tool call summary: `▶ 2 tool call(s): Read, Bash`
- Syntax highlighting for code blocks (40+ languages via syntect)
- Message timestamps and token counts per turn

**Keys inside the viewer:**

| Key | Action |
|-----|--------|
| `j` / `k` or `↓` / `↑` | Scroll through messages |
| `PgUp` / `PgDn` | Scroll faster |
| `/` | Open inline regex search |
| `n` | Jump to next search match |
| `N` | Jump to previous search match |
| `x` | Export conversation to HTML with syntax highlighting |
| `Esc` | Close viewer, return to previous tab |

**Regex search** shows a match counter in the format `[2/7]` (current / total matches). Matches are highlighted in the conversation as you navigate.

**HTML export** produces a self-contained file with full syntax highlighting. Useful for sharing sessions or archiving.

---

## Live session monitoring

By default, ccboard reads static JSONL files. To get real-time status (Running / WaitingInput / Stopped) and macOS notifications when a session ends, inject Claude Code hooks once:

```bash
ccboard setup
```

This adds entries to `~/.claude/settings.json` under the `hooks` key. The hooks write a small status file that ccboard polls via its file watcher.

After setup, the Sessions tab shows live indicators:
- `●` Running: Claude is actively processing
- `◐` Waiting for input: Claude is waiting for your response or permission
- `✓` Stopped: Session ended

CPU, RAM, and live token counts also appear in the detail panel for active sessions.

To remove the hooks, edit `~/.claude/settings.json` and delete the ccboard entries under `hooks`.

---

## CLI reference

All CLI commands support `--claude-home <path>` to override the default `~/.claude` directory.

### TUI and web

```bash
ccboard                          # Launch TUI (default)
ccboard web --port 3333          # Launch web interface
ccboard both --port 3333         # Launch TUI and web simultaneously
ccboard stats                    # Print stats summary and exit
```

### Session commands

```bash
ccboard recent 10                # Show 10 most recent sessions
ccboard recent 5 --json          # JSON output
ccboard info <session-id>        # Show session details
ccboard resume <session-id>      # Resume session in Claude CLI
```

### Search

```bash
ccboard search "query"           # Search sessions (FTS5)
ccboard search "bug" --limit 10  # Limit results
ccboard search "fix" --since 7d  # Last 7 days only
ccboard search "auth" --since 30d
```

### Discovery

```bash
ccboard discover --all           # Analyze all projects (last 90 days)
ccboard discover --all --since 30d --min-count 2 --top 10
ccboard discover --top 20        # Current project only
ccboard discover --all --json    # JSON output
ccboard discover --all --llm     # Semantic analysis via Claude CLI
```

Assigns patterns automatically: >20% of sessions becomes a CLAUDE.md rule suggestion, ≥5% becomes a skill, below 5% becomes a command.

### Setup and maintenance

```bash
ccboard setup                    # Inject live monitoring hooks
ccboard clear-cache              # Clear SQLite session metadata cache
```

---

## Export reference

### From the TUI

- **History tab** (`5`): press `x` to export the current view
- **Conversation viewer**: press `x` to export to HTML

### From the CLI

#### Sessions list

```bash
ccboard export sessions --output sessions.csv
ccboard export sessions --output sessions.json --format json
ccboard export sessions --output sessions.md --format md
ccboard export sessions --output recent.csv --since 7d
```

#### Usage statistics

```bash
ccboard export stats --output stats.csv
ccboard export stats --output stats.json --format json
ccboard export stats --output report.md --format md
```

The Markdown report includes totals, per-model table, and last 30 days of daily activity.

#### Billing blocks

```bash
ccboard export billing --output billing.csv
ccboard export billing --output billing.json --format json
ccboard export billing --output billing.md --format md
```

#### Single conversation

```bash
ccboard export conversation <session-id> --output conv.md
ccboard export conversation <session-id> --output conv.json --format json
ccboard export conversation <session-id> --output conv.html --format html
```

---

## Configuration

ccboard reads configuration from Claude Code's settings files. You can add ccboard-specific keys to any of these files.

### Settings merge order (highest priority first)

1. `.claude/settings.local.json` (project, developer-specific, not committed)
2. `.claude/settings.json` (project, committed to git)
3. `~/.claude/settings.local.json` (global, not committed)
4. `~/.claude/settings.json` (global)

### Subscription plan

Used to calculate budget percentages on the Dashboard.

```json
{
  "subscriptionPlan": "max20x"
}
```

| Value | Plan | Monthly cost |
|-------|------|-------------|
| `"pro"` | Claude Pro | $20 |
| `"max5x"` | Claude Max 5x | $50 |
| `"max20x"` | Claude Max 20x | $200 |
| `"api"` | API pay-as-you-go | — |

### Monthly budget alerts

```json
{
  "budget": {
    "monthlyBudgetUsd": 50.0,
    "alertThresholdPct": 80.0
  }
}
```

When the estimated monthly cost reaches `alertThresholdPct`% of `monthlyBudgetUsd`, the Analytics tab shows a warning. At 100%, it escalates to Exceeded.

---

## Environment variables

| Variable | Effect | Example |
|----------|--------|---------|
| `CCBOARD_CLAUDE_HOME` | Override `~/.claude` path | `CCBOARD_CLAUDE_HOME=/alt/.claude ccboard stats` |
| `CCBOARD_NON_INTERACTIVE` | Disable interactive prompts (CI/CD) | `CCBOARD_NON_INTERACTIVE=1 ccboard stats` |
| `CCBOARD_FORMAT` | Force output format: `json` or `table` | `CCBOARD_FORMAT=json ccboard recent 10` |
| `CCBOARD_NO_COLOR` | Disable ANSI colors | `CCBOARD_NO_COLOR=1 ccboard search "bug"` |

CI/CD example:

```bash
CCBOARD_NON_INTERACTIVE=1 CCBOARD_NO_COLOR=1 CCBOARD_FORMAT=json ccboard stats
```

---

## Tips and tricks

**Jump to a session from search results.** The Search tab (`/`) opens the conversation viewer directly when you press `Enter` on a result. You don't need to navigate to the Sessions tab first.

**Regex search inside conversations.** Once you're in the conversation viewer, press `/` and type a regex pattern. Use `n` and `N` to move between matches. The counter `[2/7]` shows your position.

**Export a conversation for sharing.** Open any session in the conversation viewer, press `x`, choose HTML. The exported file is self-contained with syntax highlighting — no external assets required.

**Find your most expensive sessions.** Go to Costs tab (`4`), switch to the Top Sessions sub-view. Sorted by estimated cost descending.

**Dead code in agents and skills.** The Plugins tab (`p`) shows invocation counts. Anything at zero over your analysis window is a candidate for cleanup.

**Copy an MCP server command.** In the MCP tab (`7`), navigate to a server and press `y`. The full launch command goes to your clipboard — useful for debugging a server that won't start.

**Bulk session analysis for security.** In Audit Log (`6`), press `r` to batch-scan all sessions (4 concurrent). Then switch to the Violations view with `Tab` to see a consolidated feed.

**Force a data refresh without restarting.** Press `Ctrl+R` to clear the SQLite metadata cache and reload from disk. Useful when sessions aren't appearing after a sync or import.

**Use the command palette for tab navigation.** If you forget a key, press `:` and type the tab name. Faster than remembering `p` for Plugins or `0` for Tools.

**Per-project budgets for team work.** Put budget config in `.claude/settings.json` at the repo level (committed). Each developer can override with `.claude/settings.local.json` (gitignored). The Config tab (`8`) shows the merge result in four columns.
