# ccboard

**A unified TUI/Web dashboard for Claude Code management**

<p align="center">
  <a href="https://github.com/FlorianBruniaux/ccboard/stargazers"><img src="https://img.shields.io/github/stars/FlorianBruniaux/ccboard?style=for-the-badge" alt="GitHub stars"/></a>
  <a href="https://crates.io/crates/ccboard"><img src="https://img.shields.io/crates/v/ccboard?style=for-the-badge&logo=rust" alt="crates.io"/></a>
  <a href="https://crates.io/crates/ccboard"><img src="https://img.shields.io/crates/d/ccboard?style=for-the-badge" alt="Downloads"/></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tests-156_passing-success?style=for-the-badge&logo=github-actions" alt="Tests"/>
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

> **The only actively-maintained Rust TUI** combining Claude Code monitoring, config management, hooks, agents, and MCP servers in a single 5.8MB binary. 89x faster startup with SQLite cache, 156 tests, 0 clippy warnings.

---

## Stats

| Metric | Value |
|--------|-------|
| **Stars** | ![GitHub stars](https://img.shields.io/github/stars/FlorianBruniaux/ccboard?style=social) |
| **Language** | Rust 1.85+ |
| **Binary Size** | 5.8MB (release) |
| **Startup Time** | <300ms (warm cache) |
| **Tests** | 156 passing |
| **Clippy** | 0 warnings |
| **Tabs** | 9 (TUI) + Web API |
| **Cache Speedup** | 89x (20s → 224ms) |
| **License** | MIT OR Apache-2.0 |

---

## Features

✨ **9 Interactive Tabs**
- **Dashboard**: Overview stats, model usage, MCP servers, 7-day activity, **API usage estimation** with plan-based budgets
- **Sessions**: Browse all sessions with search, live Claude processes with CPU/RAM/Tokens, and detail view
- **Config**: Cascading configuration editor (global/project/local)
- **Hooks**: Event-based hook management with file editing
- **Agents**: Browse agents, commands, and skills
- **Costs**: Token analytics with estimated costs by model/period
- **History**: Full-text search across sessions with temporal patterns
- **MCP**: Server management with status detection
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

ccboard vs other Claude Code monitoring tools (verified 2026-02-04):

| Feature | **ccboard** | vibe-kanban | ccusage | Usage-Monitor | Sniffly |
|---------|-------------|-------------|---------|---------------|---------|
| **Status** | ✅ Active | ✅ Active | ✅ Active | 🔴 Stale 7m | 🔴 Stale 6m |
| **Stars** | 0 | 20,478 | 10,361 | 6,412 | 1,131 |
| **Language** | Rust | TypeScript | TypeScript | Python | Python |
| **Type** | TUI+Web | Web UI | CLI | Terminal | Web UI |
| | | | | | |
| **TUI Dashboard** | ✅ 9 tabs | ❌ | ❌ | ✅ 1 view | ❌ |
| **Config Viewer (3-level merge)** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Hooks Viewer + Test** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Agents/Commands/Skills Browser** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **MCP Server Status Detection** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **SQLite Cache (89x speedup)** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Export CSV/JSON** | ✅ | ❌ | ✅ JSON | ❌ | ❌ |
| **Live File Watcher** | ✅ | ❌ | ❌ | ⚠️ Poll 3s | ❌ |
| **Advanced Analytics (Forecast, Budget)** | ✅ 4 views | ❌ | ❌ | ⚠️ Basic | ❌ |
| **Single Binary (no runtime)** | ✅ 5.8MB | ❌ npm | ❌ npm | ❌ pip | ❌ pip |
| | | | | | |
| **P90 Predictions** | ❌ | ❌ | ❌ | ✅ | ❌ |
| **Conversation Viewer** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Kanban Workflow** | ❌ | ✅ | ❌ | ❌ | ❌ |

**Unique to ccboard**:
- Only Rust TUI actively maintained (4/6 competitors stale since Aug-Sep 2025)
- Config 3-level merge viewer (global/project/local)
- Hooks syntax highlighting + test mode
- Agents/Commands/Skills browser with invocation stats
- MCP process detection (cross-platform)
- SQLite metadata cache (89x faster startup)
- **Advanced Analytics**: 30-day forecasting, budget alerts, session duration stats, usage patterns
- Dual TUI + Web single binary

**References**:
- **vibe-kanban**: Multi-provider kanban (broader scope, different target)
- **ccusage**: CLI cost tracker (reference for pricing, no dashboard)
- **Usage-Monitor**: Stale since Sep 2025 (7 months, 74 open issues)
- **Sniffly**: Stale since Aug 2025 (6 months)

**Complementary tools**:
- **[xlaude](https://github.com/Xuanwo/xlaude)** (171 ⭐): Git worktree manager for Claude sessions
  - **Complementarity**: xlaude focuses on workspace isolation (PTY sessions, branch management), ccboard on analytics/monitoring
  - **Performance comparison**: ccboard lazy loading 15x faster (4.8s vs 72s for 3000 sessions)
  - **Use cases**: Use xlaude for session isolation, ccboard for historical analysis and cost tracking
  - **Learnings applied**: Environment variables (QW1), message filtering (QW2), performance validation (QW3)

---

## Screenshots

### Dashboard - Key Metrics & Model Usage
![Dashboard](assets/screenshots/dashboard.png)

### Sessions - Project Tree & Search
![Sessions](assets/screenshots/sessions.png)

### Search Highlighting
![Search](assets/screenshots/recherche.png)

### Help Modal - Keybindings
![Help](assets/screenshots/aide.png)

<details>
<summary>📸 More Screenshots (click to expand)</summary>

#### Configuration Viewer
![Config](assets/screenshots/config.png)

#### Hooks Management
![Hooks](assets/screenshots/hooks.png)

#### Agents & Commands
![Agents](assets/screenshots/agents.png)
![Commands](assets/screenshots/commands.png)

#### Cost Analytics
![Costs](assets/screenshots/costs.png)
![Cost by Model](assets/screenshots/cost%20by%20model.png)

#### History Search
![History](assets/screenshots/history.png)

#### MCP Servers
![MCP](assets/screenshots/mcp.png)

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

### From crates.io

```bash
cargo install ccboard
```

### From source

```bash
git clone https://github.com/FlorianBruniaux/ccboard.git
cd ccboard
cargo build --release
```

Binary location: `target/release/ccboard` (~5.8MB)

### Requirements

- Rust 1.85+ (for development)
- Claude Code installed with `~/.claude` directory

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
└── ccboard-web/             # Axum API backend (Leptos frontend TODO Phase G)
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

**Current Status**: 🎉 **PRODUCTION-READY** (v0.4.0)

### Completed Phases ✅

- ✅ **Phase I (Infrastructure)**: Stats parser, Settings merge, Session metadata, DataStore
- ✅ **Phase D (Dashboard TUI)**: Dashboard tab with sparklines, project filters, model breakdown
- ✅ **Phase S (Sessions TUI)**: Sessions tab with search, filters, detail view
- ✅ **Phase C (Config TUI)**: Config tab with merge visualization, setting overrides
- ✅ **Phase H-A (Hooks & Agents TUI)**: Hooks tab (list + detail), Agents/Capabilities tab
- ✅ **Phase E (Economics TUI)**: Costs tab, History tab with SQLite-backed timelines
- ✅ **Phase 0-3**: Profiling, security hardening, SQLite cache (89x speedup), UI/UX quick wins

**Total**: 156 tests passing, 0 clippy warnings

### In Progress 🚧

**Phase A (Analytics TUI)** - Completion (2-4h remaining)
- 🔄 A.1-A.4: Project leaderboard, session replay, trend forecasting, anomaly detection

### Planned 📋

**Phase F (Conversation Viewer)** (8-12h)
- Full JSONL content display with syntax highlighting
- Message navigation, search within conversations
- Export selected messages

**Phase G (Leptos Frontend)** (20-30h)
- Web UI matching TUI feature parity
- Reactive components with WASM compilation
- SSE live updates integration

**Phase H (Plan-Aware)** (6-10h)
- PLAN.md parsing and visualization
- Task completion tracking
- Progress indicators per phase

For detailed roadmap, see [PLAN.md](claudedocs/PLAN.md).

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

## Links

- **Repository**: https://github.com/FlorianBruniaux/ccboard
- **Issues**: https://github.com/FlorianBruniaux/ccboard/issues
- **Releases**: https://github.com/FlorianBruniaux/ccboard/releases
- **Crates.io**: https://crates.io/crates/ccboard (coming soon)
- **Documentation**: [PLAN.md](claudedocs/PLAN.md) | [CHANGELOG.md](CHANGELOG.md)

---

**Made with ❤️ for the Claude Code community**
