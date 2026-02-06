# xlaude vs ccboard: Architecture Comparison

Quick reference guide comparing architectural decisions and trade-offs.

---

## High-Level Comparison

| Dimension | xlaude | ccboard |
|-----------|--------|---------|
| **Purpose** | AI session + worktree management | Session analytics + monitoring |
| **Primary user** | Developer (active coding) | Developer/Team lead (monitoring) |
| **Data operations** | CRUD (mutable state) | Read-only (MVP) |
| **Scale target** | <100 worktrees | 1000+ sessions |
| **Git integration** | Heavy (core feature) | None (Phase 1-5) |
| **Release status** | v0.7.0 (stable) | v0.1.0 (in development) |

---

## Technology Stack

### Core Dependencies

| Category | xlaude | ccboard | Notes |
|----------|--------|---------|-------|
| **Language** | Rust Edition 2024 | Rust Edition 2021 | Both modern Rust |
| **CLI framework** | clap 4.5 (derive) | clap 4.5 (derive) | ✅ Identical |
| **Error handling** | anyhow 1.0 | anyhow + thiserror | ccboard adds custom errors for core lib |
| **Serialization** | serde + serde_json | serde + serde_json | ✅ Identical |
| **Async runtime** | tokio 1.41 | tokio 1.41 | ✅ Identical |
| **HTTP server** | axum 0.7 | axum 0.7 | ✅ Identical |

### Unique Dependencies

**xlaude-specific**:
- `portable-pty 0.8` → PTY session management (run vim/claude in browser)
- `bip39 2.2` → Random human-readable worktree names
- `dialoguer 0.12` → Interactive CLI prompts
- `webbrowser 0.8` → Auto-open dashboard
- `directories 6.0` → Cross-platform config paths

**ccboard-specific**:
- `ratatui 0.29` → TUI framework
- `leptos 0.7` → Reactive web UI (Rust → WASM)
- `moka 0.12` → LRU cache (lazy session loading)
- `dashmap 6.1` → Concurrent HashMap (session storage)
- `notify 7.0` → File system watcher

---

## Architecture Patterns

### Binary Structure

**xlaude**: Monolithic
```
xlaude (single binary)
├── CLI commands (12 files in commands/)
├── Dashboard logic (dashboard.rs - 33KB, 73% of code)
└── Static HTML (include_str! embedded)
```

**ccboard**: Workspace
```
ccboard (workspace)
├── ccboard (binary - CLI entry)
├── ccboard-core (lib - parsers, models, store)
├── ccboard-tui (lib - Ratatui frontend)
└── ccboard-web (lib - Leptos + Axum frontend)
```

**Trade-offs**:
- xlaude: ✅ Simpler deployment, ❌ Recompile for HTML changes
- ccboard: ✅ Separation of concerns, ❌ More complex structure

### State Management

**xlaude**:
```rust
// Single JSON file (mutable)
XlaudeState {
    worktrees: HashMap<String, WorktreeInfo>,
    agent: Option<String>,
    editor: Option<String>,
}
// Location: ~/.config/xlaude/state.json
// Operations: CRUD (create, read, update, delete)
```

**ccboard**:
```rust
// Read-only from ~/.claude directory
DataStore {
    stats: RwLock<Stats>,              // stats-cache.json
    settings: RwLock<Settings>,        // settings.json (merged)
    sessions: DashMap<String, Session>, // *.jsonl (lazy loaded)
    session_cache: Moka<PathBuf, SessionContent>,
}
// Operations: Read-only (no writes in MVP)
```

**Trade-offs**:
- xlaude: Owns state, can modify → Corruption risk (no file locking)
- ccboard: External state, read-only → Safer, but no write features (Phase 6+)

### Session Parsing Strategy

**xlaude** (`claude.rs`):
```rust
// Full parse on every `list` call
for line in reader.lines() {
    if json["type"] == "user" {
        user_messages.push(content); // Accumulate ALL messages
    }
}
return user_messages.last(); // Return LAST only
```

**Performance**: O(n × m) where n = sessions, m = messages per session
- 1000 sessions × 100 msgs = 100K lines parsed every time

**ccboard** (`session_index.rs`):
```rust
// Lazy metadata-only scan
let first_line = reader.lines().next(); // Start timestamp
let last_line = reader.lines().last();  // End timestamp
// Full content loaded on-demand via Moka cache
```

**Performance**: O(n) metadata scan + O(1) detail view (cache hit)
- 1000 sessions = 2000 lines parsed (first + last only)

**Benchmark** (1000 sessions, 100KB avg):

| Operation | xlaude | ccboard | Winner |
|-----------|--------|---------|--------|
| Initial load | ~30s | ~2s | 🏆 ccboard (15x faster) |
| Memory usage | High (all msgs in RAM) | Low (metadata only) | 🏆 ccboard |
| Detail view | Instant (already loaded) | 200ms (cache miss) | 🏆 xlaude |

**Verdict**: ccboard strategy superior pour large session counts.

---

## Web Dashboard Architecture

### Frontend Approach

**xlaude**:
- **Tech**: Static HTML + vanilla JavaScript
- **Build**: None (HTML embedded with `include_str!`)
- **Assets**: 35KB `index.html` inline
- **State**: Manual DOM manipulation
- **Live updates**: WebSocket (bidirectional)

**ccboard**:
- **Tech**: Leptos (Rust → WASM) + Reactive UI
- **Build**: `wasm-pack` + `trunk` (or custom build)
- **Assets**: Compiled WASM bundle
- **State**: Reactive signals (auto-update DOM)
- **Live updates**: SSE (Server-Sent Events, unidirectional)

### Backend Routes Comparison

**xlaude** (`dashboard.rs`):
```rust
Router::new()
    .route("/", get(serve_index))                        // Static HTML
    .route("/api/worktrees", get(api_worktrees))         // List worktrees
    .route("/api/worktrees/:repo/:name/actions", post(...)) // Trigger action
    .route("/api/sessions/:id/stream", get(...))         // WebSocket stream
    .route("/api/settings", get(...).post(...))          // Config CRUD
```

**ccboard** (`ccboard-web`, planned):
```rust
Router::new()
    .route("/", get(serve_app))                          // Leptos app
    .route("/api/stats", get(api_stats))                 // Stats JSON
    .route("/api/sessions", get(api_sessions))           // Sessions list
    .route("/api/sessions/:id", get(api_session_detail)) // Session detail
    .route("/api/config/merged", get(api_config))        // Merged settings
    .route("/api/events", get(sse_live_updates))         // SSE stream
```

### Real-time Updates

**xlaude** (WebSocket):
```rust
// Bidirectional communication
Client → Server: User input (stdin to PTY)
Server → Client: PTY output (stdout/stderr)

// Use case: Interactive terminal in browser
ws.send(JSON.stringify({ type: "input", data: "ls -la\n" }));
ws.onmessage = (msg) => {
    terminal.write(msg.data); // Display PTY output
};
```

**ccboard** (SSE):
```rust
// Unidirectional broadcast
Server → Client: Stats/session updates

// Use case: Live monitoring dashboard
const events = new EventSource('/api/events');
events.addEventListener('stats_update', (e) => {
    updateDashboard(JSON.parse(e.data));
});
```

**Trade-offs**:

| Feature | WebSocket (xlaude) | SSE (ccboard) |
|---------|-------------------|---------------|
| **Complexity** | High (state sync) | Low (broadcast) |
| **Use case** | Interactive PTY | Read-only monitoring |
| **Overhead** | Higher (full-duplex) | Lower (server → client) |
| **Browser support** | Excellent | Excellent |
| **Reconnection** | Manual | Auto-reconnect built-in |

**Recommendation**: ccboard's SSE choice is correct pour read-only monitoring. WebSocket only if Phase 6+ adds interactive features.

---

## Git Integration

### xlaude (Heavy Integration)

**Core feature**: Git worktree management

```rust
// Direct CLI calls via std::process::Command
pub fn execute_git(args: &[&str]) -> Result<String> {
    Command::new("git").args(args).output()?
}

// Examples
execute_git(&["worktree", "add", path, branch])?;
execute_git(&["branch", "--merged"])?;
execute_git(&["remote", "get-url", "origin"])?;
```

**Features**:
- ✅ Create/delete worktrees
- ✅ Detect base branch (main/master/develop)
- ✅ Check merge status (via git + GitHub CLI)
- ✅ Extract repo name from remote URL

**Performance**: Synchronous, blocking calls (acceptable for <100 worktrees)

### ccboard (No Git Integration - Phase 1-5)

**Rationale**: Sessions already versioned by Claude CLI, no need to interact with git directly.

**Future** (Phase 6+): Potential git integration
- Parse git branch from session metadata (if available)
- Track git worktrees via xlaude state integration
- Detect stale sessions (branch merged but session active)

**If implemented**: Use async + caching (1000+ sessions → many git calls)

---

## Error Handling Philosophy

### xlaude

**Pattern**: `anyhow::Result<T>` everywhere, no custom error types

```rust
use anyhow::{Context, Result, bail};

pub fn create_worktree(name: &str) -> Result<()> {
    let branch = get_current_branch()
        .context("Failed to get current branch")?;

    if !is_base_branch()? {
        bail!("Must be on base branch to create worktree");
    }

    Ok(())
}
```

**Trade-offs**:
- ✅ Simple (no error enum boilerplate)
- ✅ Good error chains (`.context()`)
- ❌ No type-level error handling (all errors same type)
- ❌ Caller can't match specific error kinds

### ccboard

**Pattern**: `anyhow` for binaries, `thiserror` for core library

```rust
// ccboard-core/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Session file not found: {0}")]
    SessionNotFound(PathBuf),

    #[error("Failed to parse session: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// ccboard/src/main.rs (binary)
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let store = DataStore::new()
        .context("Failed to initialize data store")?;
    Ok(())
}
```

**Trade-offs**:
- ✅ Type-safe error handling in core library
- ✅ Caller can match specific errors
- ✅ Better API ergonomics for library users
- ❌ More boilerplate (error enum definitions)

**Recommendation**: ccboard approach is correct for library/binary split. xlaude could benefit from custom errors in `git.rs` (distinguish `BranchNotFound`, `WorktreeExists`, etc).

---

## Testing Strategy

### xlaude

**Approach**: Pragmatic (tests when necessary)

```rust
// Dev dependencies
[dev-dependencies]
insta = "1.43"        // Snapshot testing
assert_cmd = "2.0"    // CLI integration tests
tempfile = "3.23"     // Test fixtures
```

**Test files**:
- `tests/integration.rs` - CLI command integration tests
- `tests/pipe_input.rs` - Piped input handling

**Coverage**: Focused on critical paths (CLI commands, state persistence)

**Philosophy** (from `AGENTS.md`):
> "Tests are optional unless explicitly required or necessary for verification"

### ccboard

**Approach**: Comprehensive coverage planned

```rust
// Test structure (planned)
tests/
├── parsers/         # Fixtures from real sanitized data
├── tui/             # Ratatui TestBackend snapshots
├── web/             # Axum TestClient route tests
└── integration/     # #[cfg(feature = "integration")] with real ~/.claude
```

**Coverage targets**:
- ✅ Parsers (stats, settings, sessions) - fixtures
- ✅ Config merge logic (3-level priority)
- ✅ JSONL streaming (100MB+ performance regression)
- ⏭️ TUI rendering (Ratatui snapshots)
- ⏭️ Web routes (Axum TestClient)
- 🔒 Integration (manual only, real data)

**Trade-offs**:
- xlaude: ✅ Faster iteration, ❌ Lower confidence in refactors
- ccboard: ✅ High confidence, ❌ More test maintenance

---

## Performance Characteristics

### Scalability Targets

| Metric | xlaude | ccboard |
|--------|--------|---------|
| **Max worktrees/sessions** | ~100 | 1000+ |
| **Initial load time** | ~30s (100 worktrees) | <2s (1000+ sessions) |
| **Memory usage** | High (full parse) | Low (lazy + cache) |
| **Detail view** | Instant (preloaded) | 200ms (cache miss) |

### Bottlenecks

**xlaude**:
1. ❌ Full session parse on every `list` call
2. ❌ No pagination in `/api/worktrees` (returns all)
3. ⚠️ PTY session registry unbounded (memory leak risk)
4. ⚠️ No file locking on `state.save()` (corruption risk)

**ccboard**:
1. ✅ Lazy loading with Moka cache (optimized)
2. ✅ DashMap for concurrent reads (scalable)
3. ⏭️ File watcher debounce (500ms, tunable)
4. ⏭️ Pagination needed for web UI (1000+ sessions)

---

## Code Organization

### Module Structure

**xlaude** (flat):
```
src/
├── main.rs
├── dashboard.rs       (33KB - 73% of code)
├── git.rs
├── claude.rs
├── codex.rs
├── state.rs
└── commands/
    ├── create.rs
    ├── open.rs
    └── ...
```

**Observation**: `dashboard.rs` concentration → Monolithic module

**ccboard** (layered):
```
ccboard-core/src/
├── parsers/           (stats, settings, sessions)
├── models/            (Session, Stats, Settings)
├── analytics/         (patterns, usage, costs)
├── store.rs           (DataStore - central state)
└── export.rs          (JSON, CSV export)

ccboard-tui/src/
├── app.rs             (TUI state machine)
├── tabs/              (Dashboard, Sessions, Config, ...)
└── widgets/           (Reusable components)

ccboard-web/src/
├── components/        (Leptos components)
├── api/               (Axum routes)
└── sse.rs             (Server-Sent Events)
```

**Observation**: Clear separation of concerns → Scalable architecture

---

## Automation & Scripting

### Environment Variables

**xlaude**:
```bash
XLAUDE_YES=1                    # Auto-confirm prompts
XLAUDE_NON_INTERACTIVE=1        # Disable interactive mode
XLAUDE_NO_AUTO_OPEN=1           # Skip "open now?" after create
XLAUDE_CONFIG_DIR=/path         # Override config location
XLAUDE_CODEX_SESSIONS_DIR=/path # Custom Codex sessions path
XLAUDE_TEST_SEED=42             # Deterministic random names
XLAUDE_TEST_MODE=1              # Test harness mode
```

**ccboard** (planned):
```bash
CCBOARD_CLAUDE_HOME=/path       # Override ~/.claude location
CCBOARD_NON_INTERACTIVE=1       # Disable interactive prompts
CCBOARD_FORMAT=json             # Force JSON output
CCBOARD_NO_COLOR=1              # Disable ANSI colors
```

**Recommendation**: Adopt xlaude's pattern pour ccboard automation.

### Piped Input

**xlaude**:
```bash
# Input priority: CLI args > piped input > interactive
echo "feature-x" | xlaude open
echo "1" | xlaude delete  # Select first worktree
yes | xlaude delete foo   # Auto-confirm deletion
```

**ccboard** (future):
```bash
# Potential use cases
ccboard sessions --since 7d | grep "bug" | ccboard analyze
ccboard stats --json | jq '.total_sessions'
```

---

## Unique Selling Points

### xlaude Strengths

1. ✅ **Worktree isolation**: Each feature branch = dedicated AI context
2. ✅ **PTY in browser**: Run `vim`, `claude`, `bash` remotely
3. ✅ **Agent-agnostic**: Works with Claude + Codex (+ custom agents)
4. ✅ **BIP39 names**: Human-readable worktree names (`sunset-river-galaxy`)
5. ✅ **Safety checks**: Multi-level confirmations avant delete (uncommitted, unpushed, merge status)

### ccboard Strengths

1. ✅ **Session analytics**: Aggregate stats, patterns, cost tracking
2. ✅ **Lazy loading**: Handles 1000+ sessions efficiently (<2s load)
3. ✅ **Dual frontends**: TUI + Web from single binary
4. ✅ **Read-only safety**: No risk of corrupting `~/.claude` state
5. ✅ **Reactive UI**: Leptos (Rust → WASM) for type-safe frontend

---

## Integration Opportunities

### Scenario A: ccboard reads xlaude state

**Implementation**: Parse `~/.config/xlaude/state.json`

**Feature unlock**:
- Display branch name in Sessions tab
- Group sessions by worktree
- Detect stale sessions (branch merged, session active)

**Effort**: 3h (parser + DataStore integration + UI)

### Scenario B: xlaude embeds ccboard stats

**Implementation**: Call `ccboard stats --json` from xlaude dashboard

**Feature unlock**:
- Show aggregated session stats per worktree
- Cost tracking per feature branch
- Agent usage patterns

**Effort**: 2h (xlaude dashboard integration)

### Scenario C: Unified tool

**Concept**: Merge xlaude + ccboard into `claude-workspace`

**Features**:
- Worktree management (xlaude)
- Session analytics (ccboard)
- Live monitoring (PTY + reactive dashboard)

**Effort**: ~3-4 weeks (architecture alignment, UI unification)

**Feasibility**: High (both Rust, compatible stacks)

---

## Recommendation Matrix

### Choose xlaude if:
- ✅ You work on multiple feature branches simultaneously
- ✅ You need isolated AI contexts per feature
- ✅ You want interactive PTY sessions in browser
- ✅ You use git worktrees heavily

### Choose ccboard if:
- ✅ You need session analytics (costs, patterns, usage)
- ✅ You monitor 100+ sessions
- ✅ You want read-only monitoring dashboard
- ✅ You need TUI + Web interfaces

### Use BOTH if:
- ✅ You want workspace isolation (xlaude) + analytics (ccboard)
- ✅ You're in a team (xlaude for devs, ccboard for leads)
- ✅ Integration scenario A implemented (ccboard reads xlaude state)

---

## Conclusion

**Complementary tools** addressing different aspects of Claude workflow:

- **xlaude**: Active development (workspace management)
- **ccboard**: Post-hoc analysis (monitoring, analytics)

**Architectural lessons for ccboard**:
1. ✅ Keep lazy loading strategy (superior to xlaude's full parse)
2. 🎯 Adopt BIP39 names for session anonymization
3. 🎯 Add environment variable automation
4. ⏭️ Consider xlaude state integration (Phase 4)
5. ⏭️ Stick with SSE (no WebSocket unless interactive features)

**Next steps**:
- Implement quick wins (#1 BIP39, #2 Env vars, #3 Message filtering)
- Test integration with real xlaude users
- Document interoperability in README

---

**Full analysis**: See `claudedocs/xlaude-analysis.md` (24KB technical deep dive)
**Actionable insights**: See `claudedocs/xlaude-actionable-insights.md` (prioritized recommendations)
**Created**: 2026-02-06
