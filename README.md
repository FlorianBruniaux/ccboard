# ccboard

**A unified TUI/Web dashboard for Claude Code management**

<p align="center">
  <a href="https://github.com/FlorianBruniaux/ccboard/stargazers"><img src="https://img.shields.io/github/stars/FlorianBruniaux/ccboard?style=for-the-badge" alt="GitHub stars"/></a>
  <a href="https://crates.io/crates/ccboard"><img src="https://img.shields.io/crates/v/ccboard?style=for-the-badge&logo=rust" alt="crates.io"/></a>
  <a href="https://crates.io/crates/ccboard"><img src="https://img.shields.io/crates/d/ccboard?style=for-the-badge" alt="Downloads"/></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tests-234_passing-success?style=for-the-badge&logo=github-actions" alt="Tests"/>
  <img src="https://img.shields.io/badge/Clippy-0_warnings-success?style=for-the-badge&logo=rust" alt="Clippy"/>
  <img src="https://img.shields.io/badge/Binary-5.8MB-blue?style=for-the-badge" alt="Binary Size"/>
  <img src="https://img.shields.io/badge/Cache_Speedup-89x-orange?style=for-the-badge&logo=sqlite" alt="Speedup"/>
</p>

<p align="center">
  <a href="./LICENSE-MIT"><img src="https://img.shields.io/badge/License-MIT_OR_Apache--2.0-blue.svg?style=flat-square" alt="License"/></a>
  <a href="https://github.com/FlorianBruniaux/ccboard/actions"><img src="https://img.shields.io/github/actions/workflow/status/FlorianBruniaux/ccboard/ci.yml?branch=main&style=flat-square&logo=github-actions" alt="CI"/></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85%2B-orange.svg?style=flat-square&logo=rust" alt="Rust"/></a>
  <a href="#installation"><img src="https://img.shields.io/badge/platform-macOS_|_Linux_|_Windows-lightgrey?style=flat-square" alt="Platform"/></a>
</p>

<p align="center">
  <img src="assets/demo.gif" alt="ccboard demo" width="800"/>
</p>

> **The only actively-maintained Rust TUI** combining Claude Code monitoring, config management, hooks, agents, and MCP servers in a single 5.8MB binary. 89x faster startup with SQLite cache, 234 tests, 0 clippy warnings.

---

## Stats

| Metric | Value |
|--------|-------|
| **Stars** | ![GitHub stars](https://img.shields.io/github/stars/FlorianBruniaux/ccboard?style=social) |
| **Language** | Rust 1.85+ |
| **Binary Size** | 5.8MB (release) |
| **Startup Time** | <300ms (warm cache) |
| **Tests** | 234 passing |
| **Clippy** | 0 warnings |
| **Tabs** | 9 (TUI) + Web API |
| **Cache Speedup** | 89x (20s → 224ms) |
| **License** | MIT OR Apache-2.0 |

---

## Features

✨ **9 Interactive Tabs** (TUI + Web)
- **Dashboard**: Overview stats, model usage, MCP servers, 7-day activity, **API usage estimation** with plan-based budgets
- **Sessions**: Browse all sessions with search, **live Claude processes with CPU/RAM/Tokens** 🆕, and detail view
- **Config**: Cascading configuration editor (global/project/local) with 4-column diff view
- **Hooks**: Event-based hook management with bash syntax highlighting 🆕
- **Agents**: Browse agents, commands, and skills with frontmatter YAML parsing 🆕
- **Costs**: Token analytics with 4 tabs 🆕
  - **Overview**: Total cost, token breakdown bar, model distribution table
  - **By Model**: Detailed cost breakdown (input/output/cache per model)
  - **Daily**: 14-day bar chart with cost visualization
  - **Billing Blocks**: 5-hour billing windows with estimated costs
- **History**: Full-text search across sessions with temporal patterns
- **MCP**: Server management with status detection and env vars display 🆕
- **Analytics**: Advanced analytics with 4 sub-views
  - **Overview**: Monthly budget tracking with visual alerts (⚠️ warnings at threshold)
  - **Trends**: Time series charts with 30-day forecasting (confidence-based coloring)
  - **Patterns**: Activity heatmap (7 days × 24h, GitHub-style), Most Used Tools bar chart, model distribution, session duration stats
  - **Insights**: Actionable recommendations for cost optimization

🚀 **Performance First**
- **89x faster startup** (20s → 224ms) with SQLite metadata cache
- Handles 10,000+ sessions effortlessly (warm cache <300ms)
- Lazy metadata extraction, on-demand content loading
- Real-time file watching with 500ms debounce
- >99% cache hit rate after first run

🎨 **Polished UX** (k9s/lazygit-inspired)
- Command palette with fuzzy matching (`:` prefix)
- Breadcrumbs navigation trail
- Tab icons for quick identification
- Vim keybindings (hjkl) + arrow keys
- PgUp/PgDn page navigation
- Scrollbar indicators on long lists

📊 **Live Updates**
- File watcher monitors `~/.claude` changes
- Auto-refresh stats, sessions, config
- Server-Sent Events for web interface

🔧 **File Operations**
- Edit any file with `$EDITOR` integration (press `e`)
- Reveal in file manager (press `o`)
- Cross-platform support (macOS/Linux/Windows)

📦 **Zero Config**
- Works out of the box with `~/.claude`
- Single binary, no dependencies
- Cross-platform (macOS, Linux, Windows)

---

## Why ccboard Exists

**Problem**: Claude Code has no built-in visualization/analysis tools beyond basic CLI commands (`/history`, `/stats`). Users are left scripting with `jq`, `grep`, or manually opening JSON files.

**Solution**: ccboard is the **only tool** dedicated to Claude Code monitoring and management:
- **Zero direct competitors** for Claude Code dashboard (verified 2026-02-04)
- **Not competing with LangSmith/W&B** (they trace LangChain API calls, not local Claude sessions)
- **Fills the gap** between CLI commands and full observability

### Unique Position

1. **All-local**: Reads `~/.claude` files, no SaaS/API required
2. **Unified Dashboard**: 9 tabs (config, hooks, agents, MCP, analytics) vs basic CLI
3. **Performance**: SQLite cache (89x speedup), handles 10K+ sessions
4. **Dual Interface**: TUI + Web in single 5.8MB binary

**Risk**: Anthropic could integrate dashboard into Claude Code CLI. But currently, nothing exists.

---

## Competitive Landscape

ccboard vs other Claude Code monitoring tools (verified 2026-02-06):

| Feature | **ccboard** | agtrace | claudelytics | ccusage |
|---------|-------------|---------|--------------|---------|
| **Status** | ✅ Active | ✅ Active | 🔴 Stale 6m | ✅ Active |
| **Stars** | 0 | 23 | 62 | 10,361 |
| **Language** | Rust | Rust | Rust | TypeScript |
| **Type** | TUI+Web | TUI | TUI | CLI |
| | | | | |
| **TUI Dashboard** | ✅ 9 tabs | ✅ Single view | ✅ 8 tabs | ❌ |
| **Config Viewer (3-level merge)** | ✅ | ❌ | ❌ | ❌ |
| **Hooks Viewer + Test** | ✅ | ❌ | ❌ | ❌ |
| **Agents/Commands/Skills Browser** | ✅ | ❌ | ❌ | ❌ |
| **MCP Server Status Detection** | ✅ | ❌ | ❌ | ❌ |
| **SQLite Cache (89x speedup)** | ✅ | ✅ Pointer-based | ❌ | ❌ |
| **Export CSV/JSON** | ✅ | ❌ | ✅ | ✅ JSON |
| **Live File Watcher** | ✅ | ✅ Poll 1s | ❌ | ❌ |
| **Advanced Analytics (Forecast, Budget)** | ✅ 4 views | ❌ | ⚠️ Burn rate | ❌ |
| **Single Binary (no runtime)** | ✅ 5.8MB | ✅ Rust | ✅ Rust | ❌ npm |
| | | | | |
| **MCP Server Mode** | ⏳ Soon | ✅ 6 tools | ❌ | ❌ |
| **Billing Blocks (5h)** | ⏳ Soon | ❌ | ✅ | ❌ |
| **Conversation Viewer** | ⏳ Soon | ❌ | ✅ | ❌ |
| **Multi-provider** | ❌ | ✅ 3 providers | ❌ | ❌ |

**Unique to ccboard**:
- Only **multi-concern dashboard** (config + hooks + agents + MCP + analytics)
- Config 3-level merge viewer (global/project/local)
- Hooks syntax highlighting + test mode
- Agents/Commands/Skills browser with invocation stats
- MCP server **status** detection (vs agtrace = MCP server mode)
- SQLite metadata cache (89x faster startup)
- **Advanced Analytics**: 30-day forecasting, budget alerts, session duration stats, usage patterns
- Dual TUI + Web single binary

**References**:
- **agtrace** (23⭐): Observability-focused, MCP self-reflection (6 tools), multi-provider
- **claudelytics** (62⭐, STALE 6m): Feature-rich TUI (8 tabs, billing blocks, conversation viewer)
- **ccusage** (10K⭐): CLI cost tracker (reference for pricing, no dashboard)

**Complementary tools**:
- **[xlaude](https://github.com/Xuanwo/xlaude)** (171 ⭐): Git worktree manager for Claude sessions
  - **Complementarity**: xlaude focuses on workspace isolation (PTY sessions, branch management), ccboard on analytics/monitoring
  - **Performance comparison**: ccboard lazy loading 15x faster (4.8s vs 72s for 3000 sessions)
  - **Use cases**: Use xlaude for session isolation, ccboard for historical analysis and cost tracking
  - **Learnings applied**: Environment variables (QW1), message filtering (QW2), performance validation (QW3)

---

## Screenshots

### TUI (Terminal)

#### Dashboard - Key Metrics & Model Usage
![Dashboard](assets/screenshots/tui/tui-01-dashboard.png)

#### Sessions - Project Tree & Search
![Sessions](assets/screenshots/tui/tui-02-sessions-list.png)

#### Sessions - Detail View
![Session Detail](assets/screenshots/tui/tui-02-sessions-detail.png)

#### Sessions - Live Process Monitoring
![Live Sessions](assets/screenshots/tui/tui-02-sessions-live.png)

<details>
<summary>More TUI Screenshots (click to expand)</summary>

#### Configuration - 4-Column Merge View
![Config](assets/screenshots/tui/tui-03-config.png)

#### Hooks Management
![Hooks](assets/screenshots/tui/tui-04-hooks.png)

#### Agents, Commands & Skills
![Agents](assets/screenshots/tui/tui-05-agents.png)
![Commands](assets/screenshots/tui/tui-05-commands.png)
![Skills](assets/screenshots/tui/tui-05-skills.png)

#### Cost Analytics
![Costs Overview](assets/screenshots/tui/tui-06-costs-overview.png)
![Costs By Model](assets/screenshots/tui/tui-06-costs-by-model.png)
![Costs Daily](assets/screenshots/tui/tui-06-costs-daily.png)
![Costs Billing Blocks](assets/screenshots/tui/tui-06-costs-billing-blocks.png)
![Costs Leaderboard](assets/screenshots/tui/tui-06-costs-leaderboard.png)

#### History Search
![History](assets/screenshots/tui/tui-07-history.png)

#### MCP Servers
![MCP](assets/screenshots/tui/tui-08-mcp.png)

#### Analytics
![Analytics Overview](assets/screenshots/tui/tui-09-analytics-overview.png)
![Analytics Trends](assets/screenshots/tui/tui-09-analytics-trends.png)
![Analytics Patterns](assets/screenshots/tui/tui-09-analytics-patterns.png)
![Analytics Insights](assets/screenshots/tui/tui-09-analytics-insights.png)
![Analytics Anomalies](assets/screenshots/tui/tui-09-analytics-anomalies.png)

</details>

---

### Web Interface

#### Dashboard
![Web Dashboard](assets/screenshots/web/web-01-dashboard.png)

#### Sessions - Browse & Live Monitoring
![Web Sessions](assets/screenshots/web/web-02-sessions-list.png)
![Web Sessions Active](assets/screenshots/web/web-02-sessions-active.png)

<details>
<summary>More Web Screenshots (click to expand)</summary>

#### Configuration
![Web Config](assets/screenshots/web/web-03-config.png)
![Web Config Modal](assets/screenshots/web/web-03-config-modal.png)

#### Hooks
![Web Hooks](assets/screenshots/web/web-04-hooks.png)

#### Agents, Commands & Skills
![Web Agents](assets/screenshots/web/web-05-agents.png)
![Web Commands](assets/screenshots/web/web-05-commands.png)
![Web Skills](assets/screenshots/web/web-05-skills.png)

#### Cost Analytics
![Web Costs Overview](assets/screenshots/web/web-06-costs-overview.png)
![Web Costs By Model](assets/screenshots/web/web-06-costs-by-model.png)
![Web Costs Daily](assets/screenshots/web/web-06-costs-daily.png)
![Web Costs Billing Blocks](assets/screenshots/web/web-06-costs-billing-blocks.png)

#### History
![Web History](assets/screenshots/web/web-07-history.png)

#### MCP Servers
![Web MCP](assets/screenshots/web/web-08-mcp.png)

#### Analytics
![Web Analytics Overview](assets/screenshots/web/web-09-analytics-overview.png)
![Web Analytics Trends](assets/screenshots/web/web-09-analytics-trends.png)
![Web Analytics Patterns](assets/screenshots/web/web-09-analytics-patterns.png)
![Web Analytics Insights](assets/screenshots/web/web-09-analytics-insights.png)

</details>

---

## Learning Paths

Choose your path based on your goal:

<details>
<summary><strong>Quick Start</strong> — Get running in 5 minutes</summary>

1. **Install**: `cargo install ccboard`
2. **Launch**: `ccboard`
3. **Navigate tabs**: Press `1-9` to jump between tabs
4. **Search sessions**: Press `2` (Sessions tab) then `/` to search
5. **Check costs**: Press `6` (Costs tab) to see token costs

**You're ready!** Press `?` anytime for keybindings help.

</details>

<details>
<summary><strong>For Monitoring</strong> — Track costs and sessions (10 minutes)</summary>

**Goal**: Monitor Claude Code usage and costs in real-time.

1. **Dashboard overview** (Tab 1)
   - Total tokens, sessions, costs
   - 7-day activity sparkline
   - API usage estimation with plan budgets

2. **Live sessions** (Tab 2)
   - Process detection (CPU, RAM, tokens)
   - 3-pane browser (projects → sessions → detail)
   - Search with `/`

3. **Cost tracking** (Tab 6)
   - Daily costs, cost by model, billing blocks
   - Export CSV for accounting

4. **Analytics trends** (Tab 9)
   - **Overview**: Monthly budget tracking with visual progress bars and alerts
   - **Trends**: Time series charts with 30-day forecasting (confidence-coded)
   - **Patterns**: Peak hours, model distribution, session duration statistics
   - **Insights**: Actionable cost optimization recommendations

**Next**: Configure budget alerts in `.claude/settings.json` or export data with Tab 7 (History).

</details>

<details>
<summary><strong>For Configuration</strong> — Manage setup and tools (15 minutes)</summary>

**Goal**: View and manage Claude Code configuration.

1. **Config 3-level merge** (Tab 3)
   - See default → global → project → local cascade
   - 4-column diff view
   - Press `e` to edit in `$EDITOR`

2. **Hooks management** (Tab 4)
   - Syntax highlighting for `.sh` scripts
   - Test mode: press `t` to dry-run hook
   - Badge indicators (PreToolUse, PostSessionEnd, etc.)

3. **Agents browser** (Tab 5)
   - 3 sub-tabs: Agents / Commands / Skills
   - Frontmatter parsing (YAML metadata)
   - Invocation stats

4. **MCP servers** (Tab 8)
   - Process detection (running/stopped)
   - Server descriptions from config
   - Env vars masking (security)

**Next**: Press `r` to refresh all data after config changes.

</details>

<details>
<summary><strong>For Power Users</strong> — Advanced features (30 minutes)</summary>

**Goal**: Master all ccboard capabilities.

1. **Export workflows** (Tab 7)
   - CSV: Sessions, billing blocks (5h UTC windows)
   - JSON: Structured session metadata
   - Filters: Date range, project, model

2. **SQLite cache internals** (Architecture)
   - Read `ARCHITECTURE.md` for cache strategy
   - 89x speedup explained (WAL mode, mtime invalidation)
   - Clear cache: `ccboard clear-cache`

3. **File watcher config**
   - Adaptive debounce (500ms default, burst detection)
   - EventBus (7 event types, 256 capacity)
   - See `ARCHITECTURE.md` Event System section

4. **Custom hooks development**
   - Examples: `examples/hooks/bash/`
   - PreToolUse, PostToolUse, PostSessionEnd
   - See `CONTRIBUTING.md` for hook patterns

5. **Dual mode: TUI + Web**
   - Run both: `ccboard both --port 3333`
   - Web API: `http://localhost:3333/api/stats`
   - SSE live updates: `/api/events`

**Next**: Contribute! See `CONTRIBUTING.md`.

</details>

---

## Installation

### Recommended: Homebrew (macOS/Linux)

```bash
brew tap FlorianBruniaux/tap
brew install ccboard
```

**Why Homebrew?** Simple one-command install, automatic updates via `brew upgrade`, no manual Rust setup required.

### Alternative: cargo install (requires Rust 1.85+)

```bash
cargo install ccboard
```

**Why cargo?** ccboard's target audience (Claude Code users) often has Rust installed. Ensures compatibility and always installs latest crates.io version.

### Alternative: Pre-built binaries

Download from [GitHub Releases](https://github.com/FlorianBruniaux/ccboard/releases/latest):

| Platform | Status | Download |
|----------|--------|----------|
| **macOS** (x86_64/ARM64) | ✅ Fully tested | [ccboard-macos-*.tar.gz](https://github.com/FlorianBruniaux/ccboard/releases) |
| **Linux** (x86_64/ARM64) | ⚠️ Community-tested | [ccboard-linux-*.tar.gz](https://github.com/FlorianBruniaux/ccboard/releases) |
| **Windows** (x86_64) | 🧪 Experimental | [ccboard-windows-*.exe.zip](https://github.com/FlorianBruniaux/ccboard/releases) |

**Platform support:**
- ✅ **macOS**: Manually tested on every release
- ⚠️ **Linux**: CI-tested, community validation welcome
- 🧪 **Windows**: Best-effort support (feedback appreciated)

---

## Usage

### TUI Mode (Default)

```bash
ccboard              # Launch TUI dashboard
ccboard stats        # Print stats and exit
ccboard search "query"   # Search sessions
ccboard recent 10    # Show 10 most recent sessions
```

### Web Mode

ccboard has **2 web workflows** depending on your use case:

#### Option 1: Production (Single Command) ⭐ Recommended

**For**: Running the full stack (API + Frontend) in production or for general use.

```bash
# Step 1: Compile frontend once (run in ccboard repo root)
trunk build --release

# Step 2: Start server (serves API + static frontend)
ccboard web
```

**Output**:
```
⠋ Loading sessions and statistics...
✓ Ready in 2.34s (1,247 sessions loaded)

🌐 Backend API + Frontend: http://127.0.0.1:3333
   API endpoints:          http://127.0.0.1:3333/api/*
```

**Features**:
- ✅ Single process, single port
- ✅ Serves backend API (`/api/*`) + frontend static files
- ✅ Real-time data updates via Server-Sent Events (SSE)
- ❌ No hot reload (need `trunk build` + F5 after code changes)

**When to use**: Daily use, demos, production, or when you just want the web interface running.

---

#### Option 2: Development (Hot Reload) 🔧

**For**: Developing the frontend with automatic recompilation and browser refresh.

```bash
# Terminal 1: Start backend API
ccboard web --port 8080

# Terminal 2: Start frontend dev server (run in ccboard repo root)
trunk serve --port 3333
```

**Output Terminal 1**:
```
🌐 Backend API only:       http://127.0.0.1:8080/api/*
   💡 Run 'trunk build' to compile frontend
```

**Output Terminal 2**:
```
📦 Starting build...
✅ Success! App is being served at: http://127.0.0.1:3333
```

**Features**:
- ✅ Real-time data updates via SSE
- ✅ **Hot reload**: Frontend code changes auto-recompile and refresh browser
- ✅ Separate logs for backend and frontend
- ❌ Two terminals required

**When to use**: When developing the Leptos frontend (editing `crates/ccboard-web/src/**/*.rs`).

**Note**: `trunk serve` automatically proxies `/api/*` requests to `http://localhost:8080` via Trunk.toml config.

---

### Dual Mode (TUI + Web)

Run both TUI and web server simultaneously:

```bash
ccboard both --port 3333
```

- Web server runs in background
- TUI runs in foreground
- Shared DataStore (same data, live updates)
- Press `q` in TUI to exit both

---

## Troubleshooting

### "Stats not loading" or "No sessions found"

Run Claude Code at least once to generate `~/.claude` directory:

```bash
claude  # Or use Claude Code via IDE
```

Then relaunch ccboard.

### "WASM compilation failed" (Web mode)

Ensure trunk is installed:

```bash
cargo install trunk
trunk --version  # Should be 0.18+
```

Then rebuild:

```bash
cd ccboard-web
trunk build --release
```

### "Connection refused" (Web mode)

Check if backend port is in use:

```bash
lsof -i :8080  # macOS/Linux
netstat -ano | findstr :8080  # Windows
```

Change port if needed:

```bash
ccboard web --port 3333
```

### Linux: "File manager not opening"

Install xdg-utils:

```bash
sudo apt install xdg-utils  # Debian/Ubuntu
sudo dnf install xdg-utils  # Fedora
```

### Windows: Terminal rendering issues

Use Windows Terminal (not cmd.exe) for proper Unicode support:
- Download: [Windows Terminal](https://aka.ms/terminal)
- Braille spinners `⠋⠙⠹` render correctly in Windows Terminal

---

## Quick Start

### TUI (Default)

```bash
# Launch dashboard
ccboard

# Focus on specific project
ccboard --project ~/myproject

# Use custom Claude home
ccboard --claude-home ~/custom/.claude
```

### Web Interface

```bash
# Run web server on port 3333
ccboard web --port 3333

# Open http://localhost:3333
```

**Available Pages** (100% TUI parity) ✅:
- `/` - Dashboard with KPIs and forecast
- `/sessions` - Sessions browser with **live CPU/RAM monitoring** 🔥
- `/analytics` - Analytics with budget tracking
- `/config` - 4-column configuration viewer
- `/hooks` - Hooks with syntax highlighting
- `/mcp` - MCP servers with status
- `/agents` - Agents/Commands/Skills browser
- `/costs` - 4 tabs (Overview, By Model, Daily, Billing Blocks)
- `/history` - History search and filters

### Both TUI + Web

```bash
# Run both interfaces simultaneously
ccboard both --port 3333
```

### Stats Only

```bash
# Print stats summary and exit
ccboard stats
```

**Output example:**
```
ccboard - Claude Code Statistics
================================

Total Tokens:     12.5M
  Input:          8.2M
  Output:         3.1M
  Cache Read:     890K
  Cache Write:    310K

Sessions:         2,340
Messages:         18,450
Cache Hit Ratio:  28.7%

Models:
  claude-sonnet-4.5: 9.8M tokens (in: 6.5M, out: 2.3M)
  claude-opus-4: 1.2M tokens (in: 800K, out: 400K)
```

---

## Configuration

### API Usage Estimation

ccboard displays estimated API costs in the Dashboard with plan-based budget tracking. Configure your subscription plan to see accurate percentages and budget limits.

**Add to `~/.claude/settings.json`** (global) **or** `.claude/settings.json` (per-project):

```json
{
  "subscriptionPlan": "max20x"
}
```

**Available plans:**

| Plan | Subscription Cost | Config Value |
|------|-------------------|--------------|
| Claude Pro | $20/month | `"pro"` |
| Claude Max 5x | $50/month | `"max5x"` |
| Claude Max 20x | $200/month | `"max20x"` |
| API (Pay-as-you-go) | No fixed cost | `"api"` |

**Important**: Max plans have **rate limits** (requests/day), not fixed spending limits. The costs shown are subscription prices used as reference points for budget estimation.

**Dashboard display:**

```
┌─ 💰 API Usage (Est.) - Claude Max 20x ─┐
│ Today:      $ 2.45 / $200.00  (  1.2%)│
│ This week:  $ 8.12 / $200.00  (  4.1%)│
│ This month: $78.40 / $200.00  ( 39.2%)│
└──────────────────────────────────────────┘
```

**Color coding:**
- 🟢 **Green**: < 60% of monthly budget
- 🟡 **Yellow**: 60-80% of monthly budget
- 🔴 **Red**: > 80% of monthly budget

**Note**: This is a **local estimation** calculated from your billing blocks, not real-time API data. For actual limits, use `:usage` in Claude Code or the Anthropic dashboard.

### Budget Alerts & Tracking

Configure custom monthly budgets with automatic alerts in the **Analytics tab** (Tab 9 → Overview). Get visual warnings when approaching your spending limit.

**Add to `~/.claude/settings.json`** (global) **or** `.claude/settings.json` (per-project):

```json
{
  "budget": {
    "monthlyBudgetUsd": 50.0,
    "alertThresholdPct": 80.0
  }
}
```

**Configuration:**

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `monthlyBudgetUsd` | number | Your monthly spending limit in USD | Required |
| `alertThresholdPct` | number | Alert threshold percentage (0-100) | `80.0` |

**Analytics Overview display:**

```
┌─ Budget Status ─────────────────────────────┐
│ Monthly Est: $42.50                         │
│ Budget:      $50.00  ━━━━━━━━━━━━━━━━  85% │
│ Remaining:   $7.50 (15%)                    │
│                                              │
│ ⚠️  WARNING: Approaching budget limit (85%) │
│ 💡 TIP: Projected overage: $5.20 if trend…  │
└──────────────────────────────────────────────┘
```

**Visual indicators:**

- 🟢 **Green bar**: < 60% of budget (safe zone)
- 🟡 **Yellow bar**: 60-80% of budget (caution)
- 🔴 **Red bar + ⚠️**: ≥ 80% of budget (warning)

**Alert types:**

1. **Budget Warning**: Current cost approaching threshold
2. **Projected Overage**: Forecast predicts budget exceeded if trend continues
3. **Usage Spike**: Daily tokens > 2x average (anomaly detection)

**4-level priority** (higher overrides lower):
1. `~/.claude/settings.json` (global)
2. `~/.claude/settings.local.json` (global, not committed to git)
3. `.claude/settings.json` (project, committed)
4. `.claude/settings.local.json` (project, developer-specific)

**Example workflows:**

- **Solo developer**: Set global budget in `~/.claude/settings.json`
- **Team project**: Set team budget in `.claude/settings.json` (committed), override personally in `.claude/settings.local.json`
- **Multiple projects**: Different budgets per project in each `.claude/settings.json`

---

## Usage

### Keybindings

#### Global Navigation

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Tab` / `Shift+Tab` | Navigate tabs forward/backward |
| `1-9` | Jump to specific tab |
| `:` | Open command palette |
| `r` | Refresh data |
| `Esc` | Close popup / Go back |

#### List Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `h` / `←` | Move left / Collapse |
| `l` / `→` | Move right / Expand |
| `PgUp` / `PgDn` | Page up/down (10 items) |
| `Enter` | Show detail / Select |

#### File Operations

| Key | Action |
|-----|--------|
| `e` | Edit file in `$EDITOR` |
| `o` | Reveal file in file manager |

#### Tab-Specific

**Sessions**
- `/` - Search sessions
- `Enter` - Show session detail

**Config**
- `m` - Show MCP detail modal
- `e` - Edit config file (based on column focus)

**History**
- `/` - Full-text search across sessions

**Costs**
- `Tab` / `←` / `→` - Switch cost views (Overview/By Model/Daily)

### Environment Variables

ccboard supports environment variables for automation and CI/CD workflows:

| Variable | Description | Example |
|----------|-------------|---------|
| `CCBOARD_CLAUDE_HOME` | Override Claude home directory | `CCBOARD_CLAUDE_HOME=/custom/path ccboard stats` |
| `CCBOARD_NON_INTERACTIVE` | Disable interactive prompts (CI/CD mode) | `CCBOARD_NON_INTERACTIVE=1 ccboard stats` |
| `CCBOARD_FORMAT` | Force output format: `json` or `table` | `CCBOARD_FORMAT=json ccboard recent 10` |
| `CCBOARD_NO_COLOR` | Disable ANSI colors (log-friendly) | `CCBOARD_NO_COLOR=1 ccboard search "bug"` |

**Use cases**:

```bash
# CI/CD: JSON output without colors
CCBOARD_NON_INTERACTIVE=1 CCBOARD_NO_COLOR=1 CCBOARD_FORMAT=json ccboard stats

# Testing: Isolated configuration
CCBOARD_CLAUDE_HOME=/tmp/test-claude ccboard stats

# Automation: Pipe JSON to other tools
CCBOARD_FORMAT=json ccboard sessions search "error" | jq '.[] | .id'

# Log-friendly: No colors for file redirects
CCBOARD_NO_COLOR=1 ccboard recent 50 > sessions.log
```

### Command Palette

Press `:` to open the command palette with fuzzy matching:

```
:dashboard    → Jump to Dashboard tab
:sessions     → Jump to Sessions tab
:config       → Jump to Config tab
:mcp          → Jump to MCP tab
:quit         → Exit application
```

### File Editing

ccboard integrates with your configured editor:

1. Navigate to any file (agent, session, hook, config)
2. Press `e` to edit
3. Editor opens in terminal (terminal state preserved)
4. Changes detected automatically via file watcher

**Editor priority**: `$VISUAL` > `$EDITOR` > fallback (nano/notepad.exe)

---

## Architecture

### Stack

```
ccboard/                     # Binary CLI entry point
├── ccboard-core/            # Data layer (parsers, models, store, watcher)
├── ccboard-tui/             # Ratatui frontend (9 tabs)
└── ccboard-web/             # Axum API backend + Leptos WASM frontend
```

**Dependency flow**: `ccboard` → `ccboard-tui` + `ccboard-web` → `ccboard-core`

### Core Principles

1. **Single binary, dual frontends**: TUI and web share thread-safe `DataStore`
2. **Graceful degradation**: Display partial data if files corrupted/missing
3. **Lazy loading**: SQLite metadata cache (89x speedup), content on-demand
4. **Concurrency**: Arc for sessions (50x memory reduction), parking_lot::RwLock for stats/settings

### Data Sources

ccboard reads from `~/.claude` and optional project `.claude/`:

| Type | Path | Format |
|------|------|--------|
| Stats | `~/.claude/stats-cache.json` | JSON |
| Global settings | `~/.claude/settings.json` | JSON |
| Project settings | `.claude/settings.json` | JSON |
| Local settings | `.claude/settings.local.json` | JSON (highest priority) |
| MCP config | `~/.claude/claude_desktop_config.json` | JSON |
| Sessions | `~/.claude/projects/<path>/<id>.jsonl` | Streaming JSONL |
| Agents/Commands/Skills | `.claude/{agents,commands,skills}/*.md` | YAML frontmatter + Markdown |
| Hooks | `.claude/hooks/bash/*.sh` | Shell scripts |

**Settings merge priority**: local > project > global > defaults

### Performance

Startup performance improvements from profiling and optimization (Phases 0-3):

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Cold cache** | 20.08s | 20.08s | Baseline (first run) |
| **Warm cache** | 20.08s | **224ms** | **89.67x faster** ✅ |
| **Cache hit rate** | 0% | >99% | After first run |
| **Sessions** | 3550 | 3550 | Handles 10K+ |

**Optimization techniques**:
- SQLite metadata cache with WAL mode
- mtime-based invalidation
- bincode serialization for compact storage
- Concurrent directory scanning with tokio::spawn
- Lazy session content loading (metadata-only scan)

For detailed architecture documentation, see [PLAN.md](claudedocs/PLAN.md).

---

## Development

### Prerequisites

- Rust 1.85+ (`rustup install stable`)
- Claude Code with `~/.claude` directory

### Build & Run

```bash
# Clone repository
git clone https://github.com/FlorianBruniaux/ccboard.git
cd ccboard

# Build all crates
cargo build --all

# Run TUI (default)
cargo run

# Run web interface
cargo run -- web --port 3333

# Run with debug logging
RUST_LOG=ccboard=debug cargo run
```

### Testing

```bash
# Run all tests (156 tests)
cargo test --all

# Run tests for specific crate
cargo test -p ccboard-core

# Run with logging
RUST_LOG=debug cargo test
```

### Quality Checks

```bash
# Format code (REQUIRED before commit)
cargo fmt --all

# Clippy (MUST pass with zero warnings)
cargo clippy --all-targets

# Pre-commit checklist
cargo fmt --all && cargo clippy --all-targets && cargo test --all
```

### Watch Mode

```bash
# Auto-rebuild TUI on changes
cargo watch -x 'run'

# Auto-rebuild web
cargo watch -x 'run -- web'
```

### Error Handling Standards

ccboard follows strict Rust error handling practices:

- **anyhow::Result** in binaries (`ccboard`, `ccboard-tui`, `ccboard-web`)
- **thiserror** for custom errors in `ccboard-core`
- **Always** use `.context("description")` with `?` operator
- **No unwrap()** in production code (tests only)
- **Graceful degradation**: Return `Option<T>` + populate `LoadReport`

### Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

**Development workflow:**
1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Make changes with tests
4. Run quality checks (`cargo fmt && cargo clippy && cargo test`)
5. Commit with descriptive message
6. Push and open a Pull Request

---

## Roadmap

**Current Status**: 🎉 **PRODUCTION-READY** (v0.5.0)

### Completed ✅

- ✅ **Infrastructure**: Stats parser, Settings merge, Session metadata, DataStore, SQLite cache (89x speedup)
- ✅ **TUI Dashboard**: 9 interactive tabs (Dashboard, Sessions, Config, Hooks, Agents, Costs, History, MCP, Analytics)
- ✅ **Web Frontend**: Full Leptos/WASM UI with 100% TUI parity (9 pages)
- ✅ **Live Monitoring**: CPU/RAM/Tokens tracking for active Claude processes
- ✅ **Cost Analytics**: 4 views (Overview, By Model, Daily, Billing Blocks) + budget alerts
- ✅ **Advanced Analytics**: 30-day forecasting, usage patterns, actionable insights

**Total**: 234 tests passing, 0 clippy warnings

### Coming Soon 🚧

**Conversation Viewer**
- Full JSONL content display with syntax highlighting
- Message navigation, search within conversations
- Export selected messages

**Plan-Aware Dashboard**
- PLAN.md parsing and visualization
- Task completion tracking
- Progress indicators

**MCP Server Mode**
- Expose ccboard data as MCP tools for Claude Code

For detailed roadmap, see [ROADMAP.md](claudedocs/ROADMAP.md) and [PLAN.md](claudedocs/PLAN.md).

**Documentation**:
- [API Documentation](docs/API.md) - REST API and SSE endpoints for Web UI
- [Architecture](ARCHITECTURE.md) - Technical architecture and design patterns
- [Contributing](CONTRIBUTING.md) - Development guidelines and workflow

---


## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

---

## Acknowledgments

This project was developed following Test-Driven Development (TDD) principles with guidance from Agent Academy.

**Co-Authored-By**: Claude Sonnet 4.5 <noreply@anthropic.com>

---

## Author

**Florian Bruniaux** - Independent Developer
🌐 [florian.bruniaux.com](https://florian.bruniaux.com/) | 💼 [LinkedIn](https://www.linkedin.com/in/florianbruniaux/) | 🐙 [GitHub](https://github.com/FlorianBruniaux)

---

## Links

### Project

- **Repository**: https://github.com/FlorianBruniaux/ccboard
- **Issues**: https://github.com/FlorianBruniaux/ccboard/issues
- **Releases**: https://github.com/FlorianBruniaux/ccboard/releases
- **Crates.io**: https://crates.io/crates/ccboard (coming soon)
- **Documentation**: [PLAN.md](claudedocs/PLAN.md) | [CHANGELOG.md](CHANGELOG.md)

### Related Projects

- **[RTK (Rust Token Killer)](https://github.com/FlorianBruniaux/rtk)** - CLI proxy for 60-90% token reduction on dev operations
- **[Claude Code Ultimate Guide](https://cc.bruniaux.com/)** - Comprehensive guide to Claude Code CLI
- **[ccbridge](https://ccbridge.bruniaux.com/)** - Claude Code bridge for team collaboration
- **[Cowork Guide](https://cowork.bruniaux.com/)** - Technical coworking patterns and practices

---

**Made with ❤️ for the Claude Code community**
