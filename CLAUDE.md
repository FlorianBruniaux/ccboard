# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ccboard** is a unified dashboard for Claude Code management, providing both TUI and web interfaces from a single binary to visualize sessions, stats, configuration, hooks, agents, costs, and history from `~/.claude` directories.

**Stack**: Rust workspace with 4 crates, Ratatui (TUI), Leptos + Axum (Web), DashMap + parking_lot for concurrency.

## Workspace Architecture

This is a Cargo workspace with a layered architecture:

```
ccboard/                     # Root binary - CLI entry point
├─ ccboard-core/             # Shared data layer (parsers, models, store, watcher)
├─ ccboard-tui/              # Ratatui frontend (7 tabs)
└─ ccboard-web/              # Leptos + Axum frontend
```

**Dependency flow**: `ccboard` → `ccboard-tui` + `ccboard-web` → `ccboard-core`

**Core principle**: Single binary, dual frontends sharing a thread-safe `DataStore`.

## Development Commands

### Build & Run

```bash
# Build all crates
cargo build --all

# Run TUI (default mode)
cargo run

# Run web interface
cargo run -- web --port 3333

# Run both TUI and web
cargo run -- both --port 3333

# Print stats and exit
cargo run -- stats

# Session management commands
cargo run -- search "query" --limit 10          # Search sessions
cargo run -- search "bug" --since 7d            # Search last 7 days
cargo run -- recent 10                          # Show 10 most recent sessions
cargo run -- recent 5 --json                    # JSON output
cargo run -- info <session-id>                  # Show session details
cargo run -- resume <session-id>                # Resume session in Claude CLI

# Specify Claude home directory
cargo run -- --claude-home ~/.claude --project /path/to/project
```

### Testing

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p ccboard-core

# Run tests with logging
RUST_LOG=debug cargo test

# Run integration tests (requires real ~/.claude data)
cargo test --all-features
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

### Development Workflow

```bash
# Watch and rebuild TUI
cargo watch -x 'run'

# Watch and rebuild web
cargo watch -x 'run -- web'

# Run with debug logging
RUST_LOG=ccboard=debug cargo run
```

## Core Architecture Patterns

### DataStore: Central Thread-Safe State

`ccboard-core/src/store.rs` is the single source of truth, shared by both TUI and web frontends:

- **DashMap** for sessions (high contention, many entries, per-key locking)
- **parking_lot::RwLock** for stats/settings (low contention, frequent reads, better fairness than std)
- **Moka Cache** for LRU session content (on-demand loading, prevents OOM)
- **EventBus** (tokio broadcast) for live updates across frontends

**Key methods**:
- `initial_load()` → Returns `LoadReport` for graceful degradation
- `reload_stats()` → Called by file watcher
- `update_session(path)` → Called when session file changes
- `stats()`, `settings()`, `sessions_by_project()` → Read accessors

### Graceful Degradation Pattern

All parsers return `Option<T>` and populate `LoadReport` instead of failing fast:

```rust
pub struct LoadReport {
    pub stats_loaded: bool,
    pub settings_loaded: bool,
    pub sessions_scanned: usize,
    pub sessions_failed: usize,
    pub errors: Vec<LoadError>,
}
```

**Rationale**: ccboard should display partial data if some files are corrupted/missing. Only fatal errors prevent UI launch.

### Parsing Strategy

Located in `ccboard-core/src/parsers/`:

- **stats.rs** → `stats-cache.json` (serde_json direct, retry logic for file contention)
- **settings.rs** → JSON merge (global → project → local priority)
- **session_index.rs** → JSONL streaming (BufReader line-by-line, skip malformed)
  - **Lazy metadata extraction**: Only parse first + last line to extract timestamps, message count, models used
  - **Full parse on demand**: Session content loaded when user requests detail view
- **Frontmatter** (agents/commands/skills) → Custom YAML split + serde_yaml

**Performance constraint**: With 1000+ sessions and 2.5GB of JSONL data, full parse at startup is unacceptable. Metadata-only scan targets <2s.

### Concurrency Model

- **Initial scan**: `tokio::spawn` per project directory (up to 8 concurrent)
- **File watcher**: `notify` crate with 500ms debounce (notify-debouncer-mini)
- **EventBus**: `tokio::sync::broadcast` with 100 capacity
- **Cache**: Async-aware Moka cache with 5-minute idle expiry

## Data Sources & Locations

ccboard reads from `~/.claude` and optional project `.claude/`:

| Type | Path | Parser | Format |
|------|------|--------|--------|
| Stats | `~/.claude/stats-cache.json` | `StatsParser` | JSON (serde_json) |
| Global settings | `~/.claude/settings.json` | `SettingsParser` | JSON with merge |
| Project settings | `.claude/settings.json` | `SettingsParser` | JSON with merge |
| Local settings | `.claude/settings.local.json` | `SettingsParser` | JSON (highest priority) |
| MCP config | `~/.claude/claude_desktop_config.json` | TODO | JSON |
| Sessions | `~/.claude/projects/<path>/<id>.jsonl` | `SessionIndexParser` | Streaming JSONL |
| Tasks | `~/.claude/tasks/<list-id>/<task-id>.json` | TODO | JSON |
| Agents/Commands/Skills | `.claude/{agents,commands,skills}/*.md` | Frontmatter | YAML + Markdown |
| Hooks | `.claude/hooks/bash/*.sh` | TODO | Shell scripts |

**Settings merge priority**: local > project > global > defaults

## TUI Structure (Ratatui)

Located in `ccboard-tui/src/`:

- **7 tabs**: Dashboard, Sessions, Config, Hooks, Agents, Costs, History
- **Key bindings**: `Tab`/`Shift+Tab` (nav tabs), `j/k` (nav lists), `Enter` (detail), `/` (search), `r` (refresh), `q` (quit), `1-7` (jump tabs)
- **Event loop**: Crossterm events + DataStore EventBus subscriptions
- **Widgets**: Sparkline, BarChart, Tree, List, Popup, Table (Ratatui components)

**Current implementation status**: Dashboard tab functional, other tabs show "Coming soon" placeholder.

## Web Structure (Leptos + Axum)

Located in `ccboard-web/src/`:

- **Framework**: Leptos (client-side rendering) + Axum (server routing)
- **SSE**: Live updates via Server-Sent Events (`/api/events`)
- **Routes**: `/`, `/sessions`, `/config`, `/hooks`, `/agents`, `/costs`, `/history`
- **API**: `/api/stats`, `/api/sessions`, `/api/config/merged` (JSON endpoints)

**Rationale for Leptos**: Reactive UI with Rust types, no JS build pipeline, compiled to WASM, same binary.

## Error Handling

Follow Rust-specific error handling rules from RULES.md:

- **anyhow::Result** in binaries (`ccboard`, `ccboard-tui`, `ccboard-web`)
- **thiserror** for custom errors in `ccboard-core` (`CoreError`)
- **ALWAYS** use `.context("description")` with `?` operator
- **NO unwrap()** in production code (tests only)
- **Graceful degradation**: Return `Option<T>` + populate `LoadReport` instead of panicking

## Testing Strategy

- **Parsers (core)**: Fixtures from real sanitized JSON/JSONL/MD files
- **Config merge**: Test 3-level priority (global < project < local)
- **JSONL streaming**: 100MB+ file for performance regression
- **TUI**: Ratatui `TestBackend` headless snapshots (planned)
- **Web**: Axum `TestClient` for route assertions (planned)
- **Integration**: `#[cfg(feature = "integration")]` with real `~/.claude` (manual only)

## Implementation Phases (from PLAN.md)

**Phase 1** (CURRENT): Core parsers + Dashboard TUI
- ✅ Stats parser
- ✅ Settings parser + merge
- ✅ Session metadata extraction
- ✅ DataStore initial_load
- ✅ TUI Dashboard tab
- 🚧 Other tabs (skeletons)

**Phase 2**: Sessions + Config tabs (full navigation)
**Phase 3**: Hooks, Agents, Costs, History tabs
**Phase 4**: File watcher + Web UI
**Phase 5**: Polish + Open Source (README, CI, cross-platform)

## Important Constraints

- **Read-only MVP**: No write operations to `~/.claude` in initial release (Phase 6+)
- **Metadata-only scan**: Session content loaded on demand, not at startup
- **Performance target**: Initial load <2s for 1000+ sessions
- **Graceful degradation**: Display partial UI if some data unavailable
- **Shared state**: `Arc<DataStore>` passed to both TUI and web frontends

## Common Pitfalls

- **Don't** parse full JSONL sessions at startup → Use lazy metadata extraction
- **Don't** use `std::sync::RwLock` → Use `parking_lot::RwLock` for better fairness
- **Don't** use single giant lock → DashMap for sessions, separate RwLocks per domain
- **Don't** fail fast on parse errors → Populate LoadReport, continue loading
- **Don't** forget `.context()` on `?` operator → Required for all error propagation