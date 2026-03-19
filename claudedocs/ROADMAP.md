# ccboard Roadmap

**Last Updated**: 2026-03-19
**Current Version**: v0.14.0
**Target**: v1.0.0 (Phases K-L complete)

---

## 🎯 Vision

Transform ccboard from a monitoring dashboard into a **complete Claude Code management platform** with analytics, export, advanced insights, and extensibility.

**Core Principles**:
- ✅ Read-only by default (monitoring focus)
- ✅ Performance first (<2s startup maintained)
- ✅ Claude Code-only (no multi-provider scope creep)
- ✅ Graceful degradation (partial data > crashes)

---

## 📍 Current Status (v0.14.0)

### ✅ Production Features

**TUI + Web UI** (11 tabs, 100% parity):
- Dashboard, Sessions (live monitoring with hook-based status), Config, Hooks, Agents
- Costs (5 views + quota + per-project last session cost), History (search + export), MCP, Analytics
- **Activity** — security audit, violations feed, on-demand session analysis, batch scan (4 concurrent)
- **Search** — FTS5 full-text search across all sessions with ranked snippets

**Live Session Monitoring** (v0.14.0):
- `ccboard hook <EventName>` ingests Claude Code hook JSON, updates `~/.ccboard/live-sessions.json` with file locking + atomic save
- `ccboard setup` injects 5 hooks into `~/.claude/settings.json` idempotently (`--dry-run` supported)
- `SessionType` detection (Cli / VsCode / Subagent) from Claude CLI flags
- `~/.claude.json` parser for per-project last session cost (Costs tab "Per Project" view)
- macOS osascript notifications on session stop

**Performance**:
- 89x faster startup (SQLite cache: 20s → 33ms)
- 50x memory reduction (Arc migration: 1.4GB → 28MB)
- 419 tests passing, 0 clippy warnings

**`ccboard discover`** (v0.12.0):
- Analyzes session history to suggest CLAUDE.md rules, skills, or commands
- N-gram extraction (3–6 grams) with stop-word filtering and Jaccard clustering
- `--llm` mode: calls `claude --print` for semantic analysis
- `--since`, `--min-count`, `--top`, `--all`, `--json` flags
- Cross-project pattern detection with 1.5× score bonus

**Export Features** (v0.10.0):
- `ccboard export sessions/stats/billing --format csv|json|md`
- Export conversation to markdown/json/html
- Date filter `--since 7d/30d/...`
- 6 new export functions in `ccboard-core::export`

**Light Mode & Theme Persistence** (v0.9.0):
- Full light/dark theme toggle via `Ctrl+T`
- Theme persisted across sessions (`~/.claude/cache/ccboard-preferences.json`)
- Centralized `Palette` system in `theme.rs`

**Budget Tracking** (v0.8.0):
- Month-to-date cost calculation with token-based prorata
- Monthly projection with configurable limits
- 4-level alerts (Safe/Warning/Critical/Exceeded)
- Visual gauges in TUI + Web UI

### 🐛 Known Issues

None critical. Bug #44 (Web UI non-functional after `cargo install`) resolved in v0.11.1 via `rust-embed`.

---

## 🚀 Upcoming Phases

### ✅ Phase J: Export Features (v0.10.0) - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released 2026-02-18

**Delivered**:
- `ccboard export sessions/stats/billing --format csv|json|md`
- `ccboard export conversation <id> --format markdown|json|html`
- Date filter `--since 7d/30d/...` on sessions export
- 6 new functions in `ccboard-core::export`

---

### ✅ Phase K-Activity + Fixes: v0.11.0 → v0.11.2 - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released

**Delivered**:
- v0.11.0 (2026-03-05): Activity tab (TUI + Web), FTS5 Search tab, SQLite cache v5, 30 new tests
- v0.11.1 (2026-03-06): Bug #44 fixed — WASM assets embedded via `rust-embed` (binary self-contained)
- v0.11.2 (2026-03-09): Homebrew build from source fix via `build.rs` fallback for missing `dist/`

---

### ✅ Phase Discover: `ccboard discover` (v0.12.0) - **DONE**

**Priority**: 🟡 MEDIUM
**Status**: ✅ Released 2026-03-13

**Delivered**:
- `ccboard discover` — CLI subcommand for config optimization suggestions from session history
- N-gram extraction (3–6 grams) with stop-word filtering, subsumption deduplication, Jaccard clustering
- Category assignment: >20% sessions → CLAUDE.md rule, ≥5% → skill, else → command
- Cross-project pattern detection with 1.5× score bonus
- `--llm` mode via `claude --print` subprocess for semantic analysis
- 6 new unit tests

---

### ✅ Phase Hook-Monitor: Live Session Monitoring (v0.14.0) - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released 2026-03-19

**Delivered**:
- `ccboard hook <EventName>` — Claude Code hook receiver, updates `~/.ccboard/live-sessions.json` with fd-lock + atomic save
- `ccboard setup` — idempotent hook injection into `~/.claude/settings.json`, `--dry-run` support
- `HookSessionStatus` state machine: Running / WaitingInput / Stopped / Unknown, with 30-min prune for stale stopped sessions
- `SessionType` detection (Cli / VsCode / Subagent) from Claude CLI flags
- `MergedLiveSession` display in TUI Sessions tab with colored status icons and idle time
- `~/.claude.json` parser (`ClaudeGlobalStats`) + Costs tab "Per Project" view
- File watcher for `~/.ccboard/` firing `DataEvent::LiveSessionStatusChanged` for live TUI redraw
- macOS `osascript` notification on Stop (non-blocking, injection-safe)
- 10 new tests for `is_claude_process_line` and `parse_claude_flags` — 419 total

---

### Phase K-Analytics: Advanced Analytics (v0.15.0) - **NEXT**

**Priority**: 🟡 MEDIUM
**Duration**: 10-12h
**Status**: 📋 Backlog (v0.15.0)

**Goal**: AI-powered insights, anomaly detection, and usage pattern analysis.

**Features**:
- **Anomaly Detection**: Cost spikes > 2x average, unusual activity hours
- **Usage Patterns**: Peak hours, day-of-week trends, model switching patterns
- **Model Recommendations**: Suggest Opus ↔ Sonnet switches based on usage
- **Forecast Accuracy Tracking**: Compare projections vs actual costs

**Value**:
- ✅ Proactive cost management
- ✅ Identify optimization opportunities
- ✅ Understand team productivity patterns

---

### Phase L: Plugin System (v0.13.5)

**Priority**: 🟢 LOW
**Duration**: 12-15h
**Status**: 📋 Backlog

**Goal**: Extensible architecture for community plugins and custom integrations.

**Features**:
- **Plugin API**: Hooks for custom tabs, data sources, metrics
- **Dynamic Loading**: .so/.dylib plugin discovery and loading
- **Example Plugins**:
  - Slack notifications for budget alerts
  - GitHub issue creation for anomalies
  - Custom cost allocation rules

**Value**:
- ✅ Community contributions
- ✅ Team-specific customizations
- ✅ Future-proof architecture

---

### Phase M: Conversation Viewer Enhancements (v0.13.5)

**Priority**: 🟡 MEDIUM
**Duration**: 8-10h
**Status**: 📋 Backlog (extends Phase F)
**GitHub Issues**: #3 (umbrella), #7 (message filtering), #8 (export conversations)

**Goal**: Advanced conversation analysis and visualization.

**Features**:
- **Tool Call Visualization**: Expandable nodes with input/output
- **Message Threading**: Conversation flow graphs
- **Export Enhancements**: HTML reports with syntax highlighting
- **Full-Text Search**: Regex support, multi-session search

**Depends on**: Phase F (Conversation Viewer) completed in v0.7.0

---

### Phase N: Plan-Aware Completion (v0.14.0)

**Priority**: 🟢 LOW
**Duration**: 10-14h
**Status**: 📋 Backlog (extends Phase H)
**GitHub Issues**: #4 (umbrella), #10 (PLAN.md parser), #11 (task DAG), #12 (session-to-task mapping), #13 (D3.js graph)

**Goal**: Complete PLAN.md parsing with dependency graphs and progress tracking.

**Features**:
- **Task Dependency DAG**: Visual dependency graphs (D3.js)
- **Progress Tracking**: Phase completion % across sessions
- **TodoWrite Mapping**: Link sessions to tasks automatically
- **Timeline View**: Gantt-like visualization of planned vs actual

**Depends on**: Phase H (Plan-Aware basics) partially implemented in v0.8.0

---

## 🏁 Milestone: v1.0.0

**Target**: After Phase J, K, L complete
**Criteria**:
- ✅ All major use cases covered (monitoring, export, analytics, plugins)
- ✅ Production stability (>1000 sessions tested, <2% error rate)
- ✅ Documentation complete (user guide, API docs, plugin tutorial)
- ✅ 500+ tests passing, 0 critical bugs

**Timeline**: Q2 2026 (estimated)

---

## 🔄 Future Considerations (Post v1.0.0)

### Write Operations (Safety-Critical)

**Status**: Deprioritized indefinitely
**Reason**: Read-only is safer, simpler, and covers 95% of use cases

**If implemented**:
- JSON schema validation
- Backup/rollback system
- Audit logging
- External code review mandatory

### Team & Collaboration

**Status**: Low priority
**Reason**: Single-user dashboard is core use case

**If implemented**:
- Multi-user server mode
- PostgreSQL backend
- User authentication
- Team cost allocation

### IDE Integrations

**Status**: Low priority
**Reason**: Standalone TUI/Web UI sufficient

**If implemented**:
- VS Code extension
- Neovim plugin
- JetBrains plugin

### CI/CD Integration

**Status**: Low priority
**Reason**: `ccboard stats` CLI already supports automation

**If implemented**:
- GitHub Actions integration
- Token budget enforcement in CI
- Automated reports on PRs

---

## 📊 Phase Comparison

| Phase | Priority | Duration | Version | Focus | GitHub Issues | Status |
|-------|----------|----------|---------|-------|---------------|--------|
| **J** | 🔴 HIGH | 6-8h | v0.10.0 | Export features | — | ✅ Done |
| **K-Activity** | 🔴 HIGH | 8-10h | v0.11.0 | Activity security audit + Search | — | ✅ Done |
| **K-Fixes** | — | — | v0.11.1/0.11.2 | Bug #44 (WASM embed) + Homebrew fix | #44 | ✅ Done |
| **Discover** | 🟡 MEDIUM | 4-6h | v0.12.0 | `ccboard discover` config optimizer | — | ✅ Done |
| **K-Analytics (Tool Cost)** | 🟡 MEDIUM | — | v0.13.0 | Tool token analytics, optimization suggestions | — | ✅ Done |
| **Hook-Monitor** | 🔴 HIGH | — | v0.14.0 | Live session monitoring, hook receiver, setup | — | ✅ Done |
| **K-Analytics** | 🟡 MEDIUM | 10-12h | v0.15.0 | Advanced analytics (anomaly, forecasts) | #14-21 | ⏳ Next |
| **L** | 🟢 LOW | 12-15h | v0.15.5 | Plugin system | — | 📋 Backlog |
| **M** | 🟡 MEDIUM | 8-10h | v0.15.5 | Conversation enhancements | #3, #7, #8 | 📋 Backlog |
| **N** | 🟢 LOW | 10-14h | v0.16.0 | Plan-aware completion | #4, #10-13 | 📋 Backlog |

**Total Estimated**: 46-59h for v1.0.0 completion

---

## 🎯 Success Metrics

| Metric | Baseline (v0.8.0) | Target (v1.0.0) |
|--------|-------------------|-----------------|
| **Startup Time** | 33ms | <50ms |
| **Memory Usage** | 28MB | <50MB |
| **Session Render** | <500ms (1000 msgs) | <500ms |
| **Export Speed** | N/A | <2s for 1000 sessions |
| **Test Coverage** | 344 tests | 500+ tests |
| **Bug Reports** | 0 critical | <5% error rate |

---

## 🤝 Contributing

Interested in implementing a roadmap phase? See:
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [NEXT_STEPS.md](NEXT_STEPS.md) - Current phase details
- [CLAUDE.md](../CLAUDE.md) - Development setup

---

## 📚 Related Documentation

- [VERSION_STATUS.md](VERSION_STATUS.md) - Current version details
- [CHANGELOG.md](../CHANGELOG.md) - Complete version history
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Technical design
- [API.md](../docs/API.md) - Web API documentation

---

**Last Updated**: 2026-03-19
**Maintainer**: @FlorianBruniaux
**Repository**: https://github.com/FlorianBruniaux/ccboard
