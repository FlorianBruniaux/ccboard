# ccboard

**A unified TUI/Web dashboard for Claude Code management**

[![CI](https://github.com/FlorianBruniaux/ccboard/workflows/CI/badge.svg)](https://github.com/FlorianBruniaux/ccboard/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey)](#installation)

> Real-time monitoring of Claude Code sessions, costs, configuration, hooks, agents, and MCP servers from `~/.claude` directories. Single binary, dual interfaces, zero config.

---

## Features

✨ **8 Interactive Tabs**
- **Dashboard**: Overview stats, model usage, MCP servers, 7-day activity
- **Sessions**: Browse all sessions with search and detail view
- **Config**: Cascading configuration editor (global/project/local)
- **Hooks**: Event-based hook management with file editing
- **Agents**: Browse agents, commands, and skills
- **Costs**: Token analytics with estimated costs by model/period
- **History**: Full-text search across sessions with temporal patterns
- **MCP**: Server management with status detection

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

Binary location: `target/release/ccboard` (~2.3MB)

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

## Usage

### Keybindings

#### Global Navigation

| Key | Action |
|-----|--------|
| `q` | Quit application |
| `Tab` / `Shift+Tab` | Navigate tabs forward/backward |
| `1-8` | Jump to specific tab |
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
├── ccboard-tui/             # Ratatui frontend (8 tabs)
└── ccboard-web/             # Leptos + Axum frontend (backend ready)
```

**Dependency flow**: `ccboard` → `ccboard-tui` + `ccboard-web` → `ccboard-core`

### Core Principles

1. **Single binary, dual frontends**: TUI and web share thread-safe `DataStore`
2. **Graceful degradation**: Display partial data if files corrupted/missing
3. **Lazy loading**: Metadata-only scan at startup, content on-demand
4. **Concurrency**: DashMap for sessions (per-key locking), parking_lot::RwLock for stats/settings

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

For detailed architecture documentation, see [PLAN.md](PLAN.md).

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
# Run all tests (88 tests)
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

**Current Status**: 🎉 **PRODUCTION-READY** (v0.2.0)

### Completed Phases ✅

- ✅ **Phase 0**: Profiling & Baseline (4h) - Criterion benchmarks, bottleneck analysis
- ✅ **Phase 1**: Security Hardening (4h) - Path validation, OOM protection, credential masking
- ✅ **Phase 2.1**: SQLite Metadata Cache (12h) - **89x speedup** (20s → 224ms)
- ✅ **Phase 3**: UI/UX Quick Wins (6h) - Loading spinner, help modal, search highlighting
- ✅ **Phase A.5**: crates.io metadata (0.5h) - Release preparation
- ✅ **Phase A.6**: Screenshots & assets (0.5h) - 13 production screenshots

**Total**: 27h development, 139 tests passing, 0 clippy warnings

### In Progress 🚧

**Phase A: Polish & Release** (8-12h remaining)
- ✅ A.5: crates.io metadata
- ✅ A.6: Demo assets (screenshots)
- 🔄 A.1: README.md with screenshots (current)
- ⏳ A.2: CONTRIBUTING.md guide
- ⏳ A.3: CI/CD GitHub Actions
- ⏳ A.4: Cross-platform validation (Linux, Windows)

### Planned 📋

**Phase C: Additional Features** (6-10h)
- MCP Tab enhancements (server details, status)
- History Tab export (CSV, JSON)
- Costs Tab billing blocks integration
- Sessions Tab live refresh

**Phase D: Arc Migration** (2h) - Optional
- Replace clones with Arc<SessionMetadata> (400x less RAM)
- Marginal gain post-cache, deferred

**Phase 4: Actor Model Architecture** (20h) - Post-MVP
- Zero-lock design with message passing
- CQRS pattern for read/write separation
- Write operations (session editing, config updates)
- 100K+ sessions scalability testing

For detailed roadmap, see [PLAN.md](PLAN.md).

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
- **Documentation**: [PLAN.md](PLAN.md) | [CHANGELOG.md](CHANGELOG.md)

---

**Made with ❤️ for the Claude Code community**
