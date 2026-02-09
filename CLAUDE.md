# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ccboard** is a unified dashboard for Claude Code management, providing both TUI and web interfaces from a single binary to visualize sessions, stats, configuration, hooks, agents, costs, and history from `~/.claude` directories.

**Stack**: Rust workspace with 4 crates, Ratatui (9-tab TUI), Axum (Web API backend), Leptos (WASM frontend), Arc + parking_lot for concurrency.

## Workspace Architecture

This is a Cargo workspace with a layered architecture:

```
ccboard/                     # Root binary - CLI entry point
├─ ccboard-core/             # Shared data layer (parsers, models, store, watcher)
├─ ccboard-tui/              # Ratatui frontend (9 tabs)
└─ ccboard-web/              # Axum API backend + Leptos WASM frontend
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
cargo run -- clear-cache                        # Clear SQLite cache

# Specify Claude home directory
cargo run -- --claude-home ~/.claude --project /path/to/project

# Frontend (Leptos/WASM)
trunk serve --port 3333                             # Serve frontend on http://127.0.0.1:3333
```

**Full stack workflow**:
```bash
# Terminal 1: Backend API
cargo run -- web --port 8080

# Terminal 2: Frontend
trunk serve --port 3333
# Frontend communicates with backend via http://localhost:8080/api/*
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

- **Arc<DataStore>** for shared ownership (replaced DashMap in v0.4.0, ~50x memory reduction)
- **parking_lot::RwLock** for stats/settings (low contention, frequent reads, better fairness than std)
- **SQLite Cache** for session metadata (89x faster than JSONL parsing, on-demand full content loading)
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
- **SQLite Cache**: Thread-safe reads, lazy on-demand content loading

### Performance Optimizations (v0.4.0)

- **SQLite metadata cache**: 89x faster than JSONL parsing (2.9s → 33ms for 1000+ sessions)
- **Arc migration**: ~50x memory reduction vs DashMap for session storage
- **Lazy loading**: Full session content loaded on-demand, only metadata at startup
- **Performance target**: Initial load <2s for 1000+ sessions ✅ achieved (33ms)

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

- **9 tabs**: Dashboard, Sessions, Config, Hooks, Agents/Capabilities, Costs, History, MCP, Analytics
- **Key bindings**: `Tab`/`Shift+Tab` (nav tabs), `j/k` (nav lists), `Enter` (detail), `/` (search), `r` (refresh), `q` (quit), `1-9` (jump tabs)
- **Event loop**: Crossterm events + DataStore EventBus subscriptions
- **Widgets**: Sparkline, BarChart, Tree, List, Popup, Table (Ratatui components)

**Current implementation status**: All 9 tabs fully functional (v0.4.0).

## Web Structure (Axum API + Leptos Frontend)

Located in `ccboard-web/src/`:

**Backend (Axum)**:
- JSON API endpoints: `/api/stats`, `/api/sessions`, `/api/config/merged`
- Server-Sent Events: `/api/events` for live updates
- CORS enabled for local development

**Frontend (Leptos + WASM)**:
- Reactive UI built with Leptos (Rust → WASM, no JavaScript build pipeline)
- Served via `trunk serve` on port 3333
- Pages: Dashboard, Sessions, Analytics, Config, History
- Features: Token usage forecast, session management, real-time updates
- Design: Dark mode with cyan/blue palette

**Architecture**: Frontend (port 3333) communicates with backend (port 8080) via REST API.

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

**Completed Phases (v0.4.0+)**:
- ✅ **Phase I (Infrastructure)**: Stats parser, Settings merge, Session metadata, DataStore, graceful degradation
- ✅ **Phase D (Dashboard TUI)**: Dashboard tab with sparklines, project filters, model breakdown
- ✅ **Phase S (Sessions TUI)**: Sessions tab with search, filters, detail view
- ✅ **Phase C (Config TUI)**: Config tab with merge visualization, setting overrides
- ✅ **Phase H-A (Hooks & Agents TUI)**: Hooks tab (list + detail), Agents/Capabilities tab (frontmatter parsing)
- ✅ **Phase E (Economics TUI)**: Costs tab, History tab with SQLite-backed timelines
- ✅ **Phase G (Leptos Frontend)**: Web UI with Dashboard, Sessions, Analytics, Config, History pages

**Current Phase**:
- 🚧 **Phase A (Analytics)**: Remaining TUI features (project leaderboard, session replay, trend forecasting, anomaly detection)

**Future Phases**:
- **Phase F (Conversation Viewer)**: Full JSONL content display with syntax highlighting
- **Phase H (Plan-Aware)**: PLAN.md parsing, task completion tracking

## Important Constraints

- **Read-only MVP**: No write operations to `~/.claude` in initial release (Phase 6+)
- **Metadata-only scan**: Session content loaded on demand, not at startup
- **Performance target**: Initial load <2s for 1000+ sessions
- **Graceful degradation**: Display partial UI if some data unavailable
- **Shared state**: `Arc<DataStore>` passed to both TUI and web frontends

## Common Pitfalls

- **Don't** parse full JSONL sessions at startup → Use SQLite cache for metadata, lazy full-content loading
- **Don't** use `std::sync::RwLock` → Use `parking_lot::RwLock` for better fairness
- **Don't** use DashMap for large collections → Arc for shared ownership (50x memory reduction)
- **Don't** fail fast on parse errors → Populate LoadReport, continue loading
- **Don't** forget `.context()` on `?` operator → Required for all error propagation

## Build Verification (Mandatory)

**CRITICAL**: After ANY Rust file edits, ALWAYS run the full quality check pipeline before committing:

```bash
cargo fmt --all && cargo clippy --all-targets && cargo test --all
```

**Rules**:
- Never commit code that hasn't passed all 3 checks
- Fix ALL clippy warnings before moving on (zero tolerance)
- If build fails, fix it immediately before continuing to next task
- Pre-commit hook will auto-enforce this (see `.claude/settings.json`)

**Why**: Buggy code was the #1 friction point (48% of issues) in usage analysis - compilation errors, type mismatches, and syntax issues requiring multiple fix rounds.

## Testing Policy

**Manual testing is REQUIRED** for CLI commands and UI changes:

- **For CLI commands**: Actually run them (`cargo run -- <command>`), don't just rely on `cargo test`
- **For TUI changes**: Launch the TUI (`cargo run`) and navigate through affected tabs
- **For backend API changes**: Launch backend (`cargo run -- web --port 8080`) and test endpoints
- **For frontend changes**: Launch full stack (`cargo run -- web` + `trunk serve`) and test in browser at http://127.0.0.1:3333
- **Describe what you see**: When testing UI, describe the actual output/behavior observed

**Anti-pattern**: Running only automated tests (cargo test, cargo clippy) without actually exercising the functionality.

**Example**: If fixing the `search` command, run `cargo run -- search "query"` and verify the output, don't just check that unit tests pass.

## Working Directory Confirmation

**ALWAYS confirm working directory before starting any work**:

```bash
pwd  # Verify you're in /Users/florianbruniaux/Sites/perso/ccboard
git branch  # Verify correct branch
```

**Never assume** which project to work in. If user request is ambiguous, ask before exploring files.

**Context**: ccboard shares directory structure with other projects (RTK, cc-economics). Wrong directory detection was the 2nd most common friction (26% of issues).

## Avoiding Rabbit Holes

**Stay focused on the task**. Do not make excessive operations to verify external APIs, documentation, or edge cases unless explicitly asked.

**Rule**: If verification requires more than 3-4 exploratory commands, STOP and ask the user whether to continue or trust available info.

**Examples of rabbit holes to avoid**:
- Excessive API signature verification across multiple crates
- Deep diving into external crate documentation when stdlib works
- Over-testing edge cases not mentioned in requirements

## Plan Execution Protocol

When user provides a numbered plan (QW1-QW4, Phase 1-5, sprint tasks, etc.):

1. **Execute sequentially**: Follow plan order unless explicitly told otherwise
2. **Commit after each logical step**: One commit per completed phase/task
3. **Never skip or reorder**: If a step is blocked, report it and ask before proceeding
4. **Use TodoWrite**: Track progress for plans with 3+ steps
5. **Validate assumptions**: Before starting, verify all referenced file paths exist and working directory is correct

**Why**: Plan-driven execution produces 47% fully-achieved outcomes vs 12% without structured plans (usage analysis data).

## Language & Communication

- **User communicates in French**: Respond in French unless explicitly writing English content (docs, code comments, READMEs)
- **"reprend"**: Resume previous task where it was left off
- **Be direct**: User prefers direct, factual communication (Bold Guy style from global CLAUDE.md)

## Graceful Degradation for Parsers

When implementing or fixing parsers in `ccboard-core/src/parsers/`:

**Pattern**:
```rust
// ✅ Correct: Skip malformed entries, continue loading
match parse_entry(line) {
    Ok(entry) => sessions.insert(entry.id, entry),
    Err(e) => {
        load_report.errors.push(LoadError::MalformedEntry(line_num, e));
        load_report.sessions_failed += 1;
        continue; // Skip this entry, continue with next
    }
}
```

**Never**:
```rust
// ❌ Wrong: Fail fast on first error
let entry = parse_entry(line)?; // Panics on first malformed entry
```

**Rationale**: ccboard must display partial data if some files are corrupted. Only fatal errors (missing ~/.claude directory, permission denied) should prevent UI launch.