# ccboard Architecture

**Version**: 0.8.0
**Last Updated**: 2026-02-16

This document describes the technical architecture of ccboard, a unified TUI/Web dashboard for Claude Code monitoring.

---

## Table of Contents

- [Overview](#overview)
- [Workspace Structure](#workspace-structure)
- [Core Principles](#core-principles)
- [DataStore: Central State](#datastore-central-state)
- [Concurrency Model](#concurrency-model)
- [Parser Architecture](#parser-architecture)
- [Cache Strategy](#cache-strategy)
- [Event System](#event-system)
- [TUI Architecture](#tui-architecture)
- [Web Architecture](#web-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Data Flow](#data-flow)
- [Error Handling](#error-handling)

---

## Overview

ccboard is a Rust workspace with 4 crates providing dual TUI + Web interfaces for Claude Code monitoring:

```
┌─────────────────────────────────────────────────┐
│              ccboard (binary)                    │
│              CLI entry point                     │
└───────────────┬──────────────┬──────────────────┘
                │              │
        ┌───────▼──────┐  ┌───▼─────────────┐
        │ ccboard-tui  │  │ ccboard-web     │
        │ (Ratatui)    │  │ (Leptos+Axum)   │
        │ 9 tabs       │  │ API backend     │
        └───────┬──────┘  └───┬─────────────┘
                │              │
                └──────┬───────┘
                       │
              ┌────────▼────────┐
              │  ccboard-core   │
              │  (shared lib)   │
              │                 │
              │ • Parsers       │
              │ • Models        │
              │ • DataStore     │
              │ • EventBus      │
              │ • Cache         │
              │ • Analytics     │
              └─────────────────┘
```

**Key Design Goals**:
1. **Single binary, dual frontends**: TUI and Web share thread-safe state
2. **Performance first**: 89x speedup via SQLite cache
3. **Graceful degradation**: Display partial data if files corrupted
4. **Zero config**: Works out of the box with `~/.claude`

---

## Workspace Structure

### Crate Dependency Graph

```
ccboard (bin)
  ├─> ccboard-tui (lib)
  │    └─> ccboard-core (lib)
  │
  └─> ccboard-web (lib)
       └─> ccboard-core (lib)
```

**Dependency flow**: `ccboard` → `{ccboard-tui, ccboard-web}` → `ccboard-core`

### Crate Responsibilities

| Crate | Responsibility | Key Exports |
|-------|---------------|-------------|
| **ccboard** | CLI entry point, mode routing | `main()`, `Cli` struct |
| **ccboard-core** | Data layer, business logic | `DataStore`, `SessionMetadata`, parsers, analytics |
| **ccboard-tui** | Ratatui frontend | `TuiApp`, `DashboardTab`, `SessionsTab`, etc. |
| **ccboard-web** | Axum API backend + Leptos frontend | `create_router()`, `run()` |

---

## Core Principles

### 1. Single Binary, Dual Frontends

Both TUI and Web interfaces share a single `Arc<DataStore>`:

```rust
// main.rs
let store = Arc::new(DataStore::new(config)?);

match cli.command {
    TUI => run_tui(Arc::clone(&store)),
    Web => run_web(Arc::clone(&store)),
    Both => {
        tokio::spawn(run_web(Arc::clone(&store)));
        run_tui(store)
    }
}
```

**Benefits**:
- Zero serialization overhead between frontends
- Consistent state across interfaces
- Single binary deployment (~5.8MB)

### 2. Graceful Degradation

All parsers return `Option<T>` and populate `LoadReport` instead of panicking:

```rust
pub struct LoadReport {
    pub stats_loaded: bool,
    pub settings_loaded: bool,
    pub sessions_scanned: usize,
    pub sessions_failed: usize,
    pub errors: Vec<LoadError>,
}
```

**Example**: If `stats-cache.json` corrupted → display UI with empty stats panel + warning toast.

### 3. Lazy Loading

**Problem**: Parsing 3,550 JSONL sessions (2.5GB) at startup = 20s.

**Solution**: Metadata-only scan at startup, full content on-demand:

```rust
// Startup: Extract metadata from first + last line only
let metadata = SessionIndexParser::extract_metadata(path)?;
store.insert_session(metadata);

// On detail view: Load full content
let content = store.load_session_content(session_id).await?;
```

**Result**: <300ms startup (warm cache).

### 4. Concurrency Safety

**High contention** (sessions): `DashMap<String, Arc<SessionMetadata>>`
- Per-key locking (read session A doesn't block session B)
- 3,550 sessions = 3,550 independent locks

**Low contention** (stats/settings): `parking_lot::RwLock<T>`
- Multiple readers, single writer
- Better fairness than `std::sync::RwLock`

---

## DataStore: Central State

`ccboard-core/src/store.rs` is the single source of truth, thread-safe across TUI and Web.

### Structure

```rust
pub struct DataStore {
    // Configuration
    claude_home: PathBuf,
    project_path: Option<PathBuf>,
    config: DataStoreConfig,

    // Low contention (frequent reads, rare writes)
    stats: RwLock<Option<StatsCache>>,
    settings: RwLock<MergedConfig>,
    mcp_config: RwLock<Option<McpConfig>>,
    rules: RwLock<Rules>,
    invocation_stats: RwLock<InvocationStats>,
    billing_blocks: RwLock<BillingBlockManager>,
    analytics_cache: RwLock<Option<AnalyticsData>>,

    // High contention (many entries, concurrent access)
    sessions: DashMap<String, Arc<SessionMetadata>>,

    // LRU cache (on-demand loading)
    session_content_cache: Cache<String, Vec<String>>,

    // Event bus (cross-frontend notifications)
    event_bus: EventBus,

    // SQLite metadata cache (persistent)
    metadata_cache: Option<Arc<MetadataCache>>,

    // Degraded state tracking
    degraded_state: RwLock<DegradedState>,
}
```

### Key Methods

| Method | Purpose | Concurrency |
|--------|---------|-------------|
| `initial_load()` | Scan `~/.claude`, populate store | Spawns 8 concurrent tasks per project |
| `stats()` | Read-only stats access | RwLock read (non-blocking) |
| `sessions_by_project()` | Filter sessions | DashMap iteration (lock-free) |
| `update_session(path)` | Hot reload on file change | DashMap per-key lock |
| `invalidate_analytics()` | Clear forecast cache | RwLock write |

### Graceful Degradation State

```rust
pub struct DegradedState {
    pub stats_unavailable: bool,
    pub settings_unavailable: bool,
    pub mcp_unavailable: bool,
    pub partial_session_load: bool,
}
```

**Usage**: Display warning toasts if any field `true`.

---

## Concurrency Model

### Initial Scan (Startup)

```rust
// Concurrent project scanning (up to 8 parallel)
let handles: Vec<_> = projects
    .chunks(projects.len() / 8)
    .map(|chunk| {
        tokio::spawn(async move {
            for project in chunk {
                scan_project_sessions(project).await?;
            }
        })
    })
    .collect();
```

**Target**: <2s for 10,000+ sessions (metadata-only with SQLite cache).

### File Watcher (Live Updates)

```rust
// notify-debouncer-mini with adaptive debounce
let debouncer = Debouncer::new(
    Duration::from_millis(500),
    move |events| {
        for event in events {
            match event.kind {
                Modify => store.update_session(&event.path),
                Create => store.add_session(&event.path),
                Remove => store.remove_session(&event.path),
            }
        }
    }
);
```

**Burst detection**: If >10 events in 500ms → wait 2s before processing.

### EventBus (Cross-Frontend Sync)

```rust
// Broadcast events to all subscribers
pub enum DataEvent {
    StatsUpdated,
    SessionCreated(SessionId),
    SessionUpdated(SessionId),
    ConfigChanged(ConfigScope),
    AnalyticsUpdated,
    LoadCompleted,
    WatcherError(String),
}

// Subscribe in TUI
let mut rx = store.event_bus.subscribe();
tokio::spawn(async move {
    while let Ok(event) = rx.recv().await {
        handle_event(event);
    }
});
```

**Capacity**: 256 events buffered per subscriber (tokio broadcast channel).

---

## Parser Architecture

### Parser Pattern

All parsers implement graceful degradation:

```rust
pub trait Parser {
    type Output;

    fn parse(&self, path: &Path) -> Option<Self::Output>;
    fn record_error(&self, error: LoadError);
}
```

**Rule**: Parsers NEVER panic, always return `Option<T>`.

### Parser Inventory (10 modules)

| Module | Input | Strategy | Output |
|--------|-------|----------|--------|
| **StatsParser** | `stats-cache.json` | serde_json + retry (3x 100ms) | `StatsCache` |
| **SettingsParser** | 3 JSON files | Merge with priority | `MergedConfig` |
| **SessionIndexParser** | `*.jsonl` | Streaming, metadata-only | `SessionMetadata` |
| **SessionContentParser** | `*.jsonl` | Full parse on demand | `Vec<SessionMessage>` |
| **HooksParser** | `*.sh` | Read file, detect type | `Vec<Hook>` |
| **McpConfigParser** | `claude_desktop_config.json` | serde_json | `McpConfig` |
| **RulesParser** | `CLAUDE.md` | YAML frontmatter + body | `Rules` |
| **TaskParser** | `tasks/*.json` | serde_json | `Vec<Task>` |
| **InvocationParser** | Scan JSONL | Regex agent/command/skill | `InvocationStats` |
| **filters** | Message text | Pattern matching | `bool` (is_meaningful) |

### Settings Merge Logic

```rust
// 4-level cascade: default < global < project < local
let merged = SettingsParser::merge(&[
    defaults(),                          // Hardcoded
    parse("~/.claude/settings.json"),    // Global
    parse(".claude/settings.json"),      // Project
    parse(".claude/settings.local.json") // Local (highest priority)
]);
```

**Example**: Local `subscriptionPlan: "max20x"` overrides global `"pro"`.

---

## Cache Strategy

### SQLite Metadata Cache

**Problem**: Parsing 3,550 JSONL files = 20s cold start.

**Solution**: SQLite cache with mtime-based invalidation.

#### Schema (v4)

```sql
CREATE TABLE session_metadata (
    path TEXT PRIMARY KEY,
    mtime INTEGER NOT NULL,           -- File modification time
    project TEXT NOT NULL,
    session_id TEXT NOT NULL,
    first_timestamp TEXT,
    last_timestamp TEXT,
    message_count INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    models_used TEXT NOT NULL,        -- JSON array
    has_subagents INTEGER NOT NULL,
    first_user_message TEXT,
    data BLOB NOT NULL                -- bincode serialized SessionMetadata
);

CREATE INDEX idx_project ON session_metadata(project);
CREATE INDEX idx_mtime ON session_metadata(mtime);
```

#### Invalidation Strategy

1. **Startup**: Compare `mtime` → if changed, rescan file
2. **File watcher**: Delete cache entry on `Modify` event
3. **Version mismatch**: Auto-clear on schema change (v3 → v4)

**Cache versioning**:
```rust
const CACHE_VERSION: i32 = 4;
// v1: Initial
// v2: Fixed TokenUsage::total()
// v3: Added token breakdown fields
// v4: Added branch field to SessionMetadata
```

**Result**: 20s → 224ms (89x speedup).

### Moka LRU Cache (Session Content)

```rust
let cache = Cache::builder()
    .max_capacity(100 * 1024 * 1024)  // 100MB
    .time_to_idle(Duration::from_secs(300))
    .build();
```

**Strategy**: Full JSONL content loaded on-demand, cached 5min idle.

**Use case**: Session detail view → load once, cache for subsequent views.

---

## Event System

### EventBus Architecture

```rust
pub struct EventBus {
    tx: broadcast::Sender<DataEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }

    pub fn publish(&self, event: DataEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DataEvent> {
        self.tx.subscribe()
    }
}
```

### Event Types

```rust
pub enum DataEvent {
    StatsUpdated,                    // stats-cache.json changed
    SessionCreated(SessionId),       // New session file
    SessionUpdated(SessionId),       // Session modified
    ConfigChanged(ConfigScope),      // settings.json changed
    AnalyticsUpdated,                // Analytics cache refreshed
    LoadCompleted,                   // Initial load finished
    WatcherError(String),            // File watcher error
}
```

### Usage in TUI

```rust
// Subscribe to events
let mut rx = store.event_bus.subscribe();

// Event loop
loop {
    select! {
        // Handle crossterm events
        Ok(event) = crossterm::event::read() => { ... }

        // Handle data events
        Ok(data_event) = rx.recv() => {
            match data_event {
                SessionUpdated(id) => refresh_session_view(id),
                StatsUpdated => refresh_stats_panel(),
                WatcherError(msg) => show_toast_error(msg),
            }
        }
    }
}
```

---

## TUI Architecture

### Stack

- **Framework**: Ratatui 0.30
- **Input**: Crossterm 0.28
- **Layout**: Immediate-mode rendering (no DOM)

### Tab System

```rust
pub enum ActiveTab {
    Dashboard,
    Sessions,
    Config,
    Hooks,
    Agents,
    Costs,
    History,
    Mcp,
    Analytics,
}

pub struct TuiApp {
    active_tab: ActiveTab,
    tabs: [Box<dyn Tab>; 9],
    store: Arc<DataStore>,
}
```

**Tab trait**:
```rust
pub trait Tab {
    fn render(&mut self, frame: &mut Frame, area: Rect, store: &DataStore);
    fn handle_input(&mut self, key: KeyEvent) -> Action;
}
```

### Event Loop Pattern

```rust
loop {
    // 1. Render
    terminal.draw(|frame| {
        app.render(frame);
    })?;

    // 2. Handle input
    if poll(Duration::from_millis(100))? {
        let event = read()?;
        let action = app.handle_input(event);

        match action {
            Action::Quit => break,
            Action::Refresh => store.reload(),
            Action::SwitchTab(tab) => app.active_tab = tab,
            _ => {}
        }
    }

    // 3. Handle data events
    while let Ok(event) = event_rx.try_recv() {
        app.handle_data_event(event);
    }
}
```

### Component Hierarchy

```
TuiApp
├── Header (breadcrumbs + tabs)
├── ActiveTab
│   ├── Dashboard
│   │   ├── StatsPanel
│   │   ├── ModelUsageChart
│   │   ├── McpServersPanel
│   │   └── ActivitySparkline
│   ├── Sessions
│   │   ├── ProjectTree (left pane)
│   │   ├── SessionList (middle pane)
│   │   └── SessionDetail (right pane)
│   └── ... (7 other tabs)
└── Footer (keybindings)
```

---

## Web Architecture

### Stack

- **Framework**: Leptos 0.7 (CSR)
- **Backend**: Axum 0.8
- **State**: Arc<DataStore> shared with TUI

### API Routes

```rust
Router::new()
    .route("/api/stats", get(stats_handler))
    .route("/api/sessions/recent", get(recent_sessions_handler))
    .route("/api/sessions/live", get(live_sessions_handler))
    .route("/api/sessions", get(sessions_handler))
    .route("/api/config/merged", get(config_handler))
    .route("/api/hooks", get(hooks_handler))
    .route("/api/mcp", get(mcp_handler))
    .route("/api/agents", get(agents_handler))
    .route("/api/commands", get(commands_handler))
    .route("/api/skills", get(skills_handler))
    .route("/api/health", get(health_handler))
    .route("/api/events", get(sse_handler))
```

### SSE Live Updates

```rust
// Server-Sent Events endpoint
async fn sse_handler(
    State(store): State<Arc<DataStore>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = store.event_bus.subscribe();

    let stream = BroadcastStream::new(rx)
        .map(|event| {
            Event::default().json_data(event)
        });

    Sse::new(stream)
}
```

**Usage**: Frontend subscribes to `/api/events` → real-time updates without polling.

### Current Status

- ✅ Axum backend (12 routes + SSE)
- ✅ Leptos frontend (9 pages: Dashboard, Sessions, Analytics, Config, Hooks, MCP, Agents, Costs, History)
- ✅ Full TUI/Web parity (100%)
- ✅ Quota management integration (v0.8.0)

**Architecture**: Leptos WASM frontend (port 3333) communicates with Axum backend (port 8080) via REST API + SSE for live updates. Features include config modal, elevation system, responsive design, and budget tracking with quota gauges.

---

## Performance Optimizations

### Phase 0-3 Results

| Metric | Before | After | Technique |
|--------|--------|-------|-----------|
| **Startup (cold)** | 20.08s | 20.08s | Baseline (unavoidable I/O) |
| **Startup (warm)** | 20.08s | 224ms | **89x** SQLite cache |
| **Memory (sessions)** | 1.4GB | 28MB | **50x** Arc migration |
| **Cache hit rate** | 0% | >99% | mtime invalidation |

### Optimization Techniques Applied

1. **SQLite WAL mode**: Concurrent reads during writes
2. **bincode serialization**: 60% smaller than JSON
3. **Concurrent scanning**: 8 tokio tasks for project dirs
4. **Arc<SessionMetadata>**: 8 bytes vs 400 bytes per clone
5. **Lazy content loading**: Metadata-only scan
6. **DashMap**: Per-key locking vs single global lock
7. **parking_lot::RwLock**: Better fairness than std

### Profiling Tools Used

- **Criterion**: Benchmarks (`cargo bench`)
- **flamegraph**: CPU profiling
- **tokio-console**: Async task monitoring
- **heaptrack**: Memory allocation analysis

---

## Data Flow

### Startup Flow

```
1. main.rs
   └─> DataStore::new(config)
       └─> MetadataCache::new()
           └─> SQLite connection (WAL mode)
       └─> EventBus::new()
       └─> initial_load()
           ├─> StatsParser::parse()
           ├─> SettingsParser::merge()
           ├─> McpConfigParser::parse()
           └─> scan_projects() (8 concurrent tasks)
               └─> For each session file:
                   ├─> Check SQLite cache (mtime)
                   ├─> If stale: SessionIndexParser::extract_metadata()
                   └─> Insert into DashMap

2. TUI/Web launch
   └─> Arc::clone(store)
   └─> Subscribe to EventBus
   └─> Render initial state
```

**Time budget**: <2s for 10,000 sessions (metadata-only, warm cache).

### Update Flow (File Watcher)

```
1. File modified (notify crate)
   └─> Debouncer (500ms adaptive)
       └─> DataStore::update_session(path)
           ├─> SessionIndexParser::extract_metadata()
           ├─> DashMap::insert() (per-key lock)
           ├─> SQLite cache::invalidate(path)
           └─> EventBus::publish(SessionUpdated)

2. EventBus subscribers (TUI + Web)
   └─> Receive SessionUpdated event
   └─> Refresh UI component
```

**Latency**: <50ms from file write to UI update.

### Session Detail View Flow

```
1. User presses Enter on session
   └─> SessionsTab::handle_input()
       └─> store.load_session_content(id)
           ├─> Check Moka cache (5min TTL)
           ├─> If miss: Read JSONL file + parse
           └─> Return Vec<String> (messages)

2. Render detail pane
   └─> Display messages with syntax highlighting
```

**Cache hit rate**: >90% for recently viewed sessions.

---

## Error Handling

### Error Types Hierarchy

```rust
// ccboard-core/src/error.rs
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
```

### Error Handling Pattern

**Binaries** (ccboard, ccboard-tui, ccboard-web):
```rust
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    fs::read_to_string(path)
        .context("Failed to read config file")?
        .parse()
        .context("Failed to parse config")
}
```

**Libraries** (ccboard-core):
```rust
use thiserror::Error;

pub enum CoreError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),
}

fn get_session(id: &str) -> Result<Session, CoreError> {
    sessions.get(id)
        .ok_or_else(|| CoreError::SessionNotFound(id.to_string()))
}
```

### Graceful Degradation Pattern

```rust
// Never panic on parse errors
let stats = match StatsParser::parse(path) {
    Some(s) => s,
    None => {
        load_report.stats_loaded = false;
        load_report.errors.push(LoadError::StatsParseError);
        return; // Continue loading other data
    }
};
```

**UI behavior**: Display warning toast + empty stats panel.

---

## Future Architecture Plans

### Phase 4: Actor Model (20h)

**Goal**: Zero-lock design with message passing.

```rust
// Actor-based DataStore
pub struct DataStoreActor {
    mailbox: mpsc::Receiver<Command>,
    state: State,
}

pub enum Command {
    GetStats(oneshot::Sender<StatsCache>),
    UpdateSession(PathBuf, oneshot::Sender<()>),
    Subscribe(broadcast::Sender<DataEvent>),
}
```

**Benefits**:
- Zero lock contention
- CQRS pattern (separate read/write)
- Easier reasoning about concurrency

**Effort**: 20h (refactor DataStore + update callers).

### Phase 5: Write Operations (10h)

**Goal**: Enable session editing, config updates.

**Challenges**:
- Transaction safety (atomic writes)
- Conflict resolution (concurrent edits)
- Undo/redo support

**Defer to Phase 5**: MVP is read-only.

---

## Contributing to Architecture

When making architectural changes:

1. **Preserve core principles**: Single binary, graceful degradation, lazy loading
2. **Benchmark before/after**: Use Criterion for perf changes
3. **Update this document**: Keep architecture docs synchronized
4. **Add integration tests**: Test cross-crate boundaries
5. **Consider concurrency**: All DataStore methods must be thread-safe

**Architectural Review Triggers**:
- New crate addition
- DataStore structure change
- Parser strategy change
- Cache invalidation logic
- Event system modification

---

## Glossary

Standardized terminology used across ccboard documentation and codebase:

| Term | Definition | Code Location |
|------|-----------|---------------|
| **SessionMetadata** | Lightweight metadata extracted from JSONL first+last lines (timestamps, token counts, models) | `ccboard-core/src/models/session.rs` |
| **SessionContent** | Full parsed JSONL content loaded on-demand for detail views | `SessionContentParser` |
| **DataStore** | Central thread-safe state shared by TUI and Web frontends | `ccboard-core/src/store.rs` |
| **EventBus** | tokio broadcast channel for cross-frontend notifications | `ccboard-core/src/event.rs` |
| **MetadataCache** | SQLite cache storing pre-parsed session metadata with mtime invalidation | `ccboard-core/src/cache/metadata_cache.rs` |
| **LoadReport** | Startup diagnostic tracking which data sources loaded successfully | `ccboard-core/src/error.rs` |
| **DegradedState** | Runtime tracking of unavailable data sources for graceful UI | `ccboard-core/src/error.rs` |
| **MergedConfig** | Result of 4-level settings cascade (default < global < project < local) | `ccboard-core/src/models/config.rs` |
| **BillingBlock** | 5-hour UTC window aggregating token usage for cost estimation | `ccboard-core/src/models/billing_block.rs` |
| **SessionId** | Unique identifier for a Claude Code session (String newtype) | `ccboard-core/src/models/session.rs` |

---

## Testing Strategy

### Test Pyramid

| Level | Count | Location | Purpose |
|-------|-------|----------|---------|
| **Unit** | ~250 | `#[cfg(test)] mod tests` in source files | Parser logic, model validation, formatters |
| **Integration** | ~20 | `tests/` directories | Cross-module, cache + store interactions |
| **Platform** | CI matrix | GitHub Actions | macOS + Linux + Windows builds |
| **Manual** | Pre-release | Checklist in CROSS_PLATFORM.md | TUI navigation, Web UI, CLI commands |

### Coverage by Component

| Component | Test Coverage | Notes |
|-----------|--------------|-------|
| **Parsers** (ccboard-core) | High | Fixtures from real sanitized data |
| **Models** (ccboard-core) | High | Serialization round-trips, field validation |
| **Analytics** (ccboard-types) | Medium | Forecast, anomaly detection, patterns |
| **Cache** (ccboard-core) | Medium | mtime invalidation, version migration |
| **CLI** (ccboard) | Medium | DateFilter, prefix matching, formatters |
| **TUI** (ccboard-tui) | Low | Manual testing (Ratatui TestBackend planned) |
| **Web** (ccboard-web) | Low | Manual testing (Axum TestClient planned) |

### Running Tests

```bash
cargo test --all                    # All 344 tests
cargo test -p ccboard-core          # Core crate only
RUST_LOG=debug cargo test           # With logging
cargo test --all-features           # Integration tests (requires ~/.claude)
```

---

## References

- [PLAN.md](claudedocs/PLAN.md) - Development phases and post-mortems
- [CLAUDE.md](CLAUDE.md) - Project instructions and constraints
- [README.md](README.md) - User-facing documentation
- [Ratatui Book](https://ratatui.rs/)
- [Tokio Docs](https://tokio.rs/)
- [DashMap Docs](https://docs.rs/dashmap/)

---

## v0.8.0 Features (Budget Tracking & Quota Management)

### Quota System Integration

**New DataStore method**:
- `quota_status()` → Returns `Option<QuotaStatus>` with MTD cost, usage %, projection
- Thread-safe via parking_lot::RwLock
- Zero-overhead Arc clones

**Budget calculation**:
- Token-based prorata: `total_cost * (mtd_tokens / total_tokens)`
- Monthly projection: `(MTD cost / current_day) * 30`
- Four-level alerts: Safe (green) / Warning (yellow) / Critical (red) / Exceeded (magenta)

**TUI Integration**:
- Quota gauge in Costs tab Overview (color-coded progress bar)
- Analytics tab budget tracking

**Web Integration**:
- REST API endpoint: `/api/quota`
- Leptos component with Suspense
- CSS progress bar with real-time SSE updates

**Configuration** (`~/.claude/settings.json`):
```json
{
  "budget": {
    "monthlyLimit": 50.0,
    "warningThreshold": 75.0,
    "criticalThreshold": 90.0
  }
}
```

**Testing**: 4 quota module tests covering all alert levels and edge cases

---

**Document Version**: 1.2
**Last Updated**: 2026-02-16
**Maintainer**: Florian Bruniaux
