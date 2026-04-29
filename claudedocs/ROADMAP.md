# ccboard Roadmap

**Last Updated**: 2026-04-21
**Current Version**: v0.22.0
**Target**: v1.0.0 (Phases L + N complete)

---

## 🎯 Vision

Transform ccboard from a monitoring dashboard into a **complete Claude Code management platform** with analytics, export, advanced insights, and extensibility.

**Core Principles**:
- ✅ Read-only by default (monitoring focus)
- ✅ Performance first (<2s startup maintained)
- ✅ Claude Code-only (no multi-provider scope creep)
- ✅ Graceful degradation (partial data > crashes)

---

## 📍 Current Status (v0.22.0)

### ✅ Production Features

**TUI + Web UI** (13 tabs, 100% parity):
- Dashboard, Sessions (live monitoring + context pressure bar), Config, Hooks, Agents
- Costs (5 views + quota + per-project last session cost), History (search + export), MCP, Analytics
- **Activity** — security audit, violations feed, on-demand session analysis, batch scan (4 concurrent)
- **Search** — FTS5 full-text search with date, message count, detail pane, search-as-you-type
- **Brain** — cross-session knowledge base, hook auto-extract, claude-mem integration
- **Visual redesign (v0.16.0)**: palette system, rounded borders, responsive heatmap, sub-tabs

**Live Context Window Monitoring (v0.22.0)** ✅:
- **Context pressure bar** — `[████░░░░░░] 42% ⚡` inline in Sessions list for active processes
- **Incremental JSONL parsing** (`LiveMonitorState`) — O(delta) parsing per tick; offset-based, inode+mtime file rotation detection; eliminates full re-parse on 50MB+ files
- **Compaction detection** — context drop >30% between turns → `compaction_count++`, `⚡` indicator
- **Context detail panel** — 20-char color-coded bar (green/orange/red), turn count, current tool
- **ccboard-ffi crate** — UniFFI bindings exposing `ccboard-core` to Swift/iOS via staticlib/cdylib
- **uniffi-bindgen crate** — binding generator binary for Swift/Kotlin targets
- 492 tests passing, 0 clippy warnings

**Brain Tab (v0.21.0)** ✅:
- Cross-session knowledge base via `session-stop.sh` + `session-start.sh` hooks, stored in `~/.ccboard/insights.db`
- Filterable by type (`←`/`→`), expandable detail pane, archive (`d`), refresh (`r`)
- Optional claude-mem integration (toggle `m`) — surfaces observations from claude-mem alongside insights
- `/ccboard-remember` skill for manual insight capture from any Claude session
- Web Brain page (Leptos): filter tabs, list + detail split panel, type icons and colors
- `GET /api/insights` Axum route with project/type/limit filters

**Third-party Session Imports (v0.21.0)** ✅:
- Cursor (`workspaceStorage/*/state.vscdb`), Codex CLI (`rollout-*.jsonl`), OpenCode (`opencode.db`)
- `SourceTool` enum; yellow badge (`[Cu]`, `[Cx]`, `[Oc]`) in Sessions tab; opt-in, silently skipped if not installed

**Code Metrics (v0.21.0)** ✅:
- `lines_added` / `lines_removed` computed from Edit/Write tool inputs during session indexing
- `+N/-N` badge in Sessions list, detail panel, and web session table

**Analytics Enhancements (v0.20.0)** ✅:
- Per-tool cost breakdown (`Costs` sub-tab): Tool | Calls | Tokens | % Total | Est. Cost | $/Call with red/yellow hotspot coloring
- Pattern Discovery (`Discover` sub-tab): n-gram analysis, Jaccard clustering, CLAUDE.md/Skill/Command suggestions
- MCP server usage stats: Server | Calls | Sessions | Last Seen
- TUI smoke tests: 12 tests, one per tab — 492 total

**Session Intelligence (v0.18.0–v0.19.0)** ✅:
- **Bookmarks** — `b` toggles, `B` filters to starred only, persisted to `~/.ccboard/bookmarks.json`
- **Subagent graph** — parent/child tree in Sessions detail pane with token/msg breakdown
- **LLM summaries** — `ccboard summarize <id>` via `claude --print`, cached to `~/.ccboard/summaries/`
- **Model switching timeline** — `Opus 4.5 (8) → Sonnet 4.6 (15)` computed inline during JSONL scan
- **Context saturation trend** — linear regression predicts sessions until 85% breach
- **Hook state TTL** — stale Running/WaitingInput sessions pruned after 10 min
- **Live sessions panel (Web Dashboard)** — polls `/api/sessions/live` every 5s, animated green dot

**Live Session Monitoring** (v0.14.0):
- `ccboard hook <EventName>` ingests Claude Code hook JSON, updates `~/.ccboard/live-sessions.json` with file locking + atomic save
- `ccboard setup` injects 5 hooks into `~/.claude/settings.json` idempotently (`--dry-run` supported)
- `SessionType` detection (Cli / VsCode / Subagent) from Claude CLI flags
- `~/.claude.json` parser for per-project last session cost (Costs tab "Per Project" view)
- macOS osascript notifications on session stop

**Advanced Analytics (v0.15.0)**:
- Streak detection (current + longest) in UsagePatterns
- Daily cost spikes (DailyCostAnomaly, Z-score based)
- Model downgrade recommendations (Opus heavy + low tools → Sonnet)
- AnalyticsData caching anomalies/daily_spikes (no per-frame recomputation)

**Phase M — Conversation Viewer (v0.15.5)** ✅:
- MA1 ✅ Tool call visualization — expandable nodes, input params (c213a65)
- MA2 ✅ Regex search in replay viewer — `/`, `n`/`N` nav, highlights (11426b8)
- MA3 ✅ Export HTML enrichi — syntect syntax highlighting, code-lang badge (d87a25d)
- MA4 ✅ FTS5 extended + Search tab — date, detail pane, search-as-you-type (4520c2e)

**Waiting Answers Panel + Max 20x tip (v0.17.0)**:
- Sessions tab: "Waiting Answers" panel showing sessions pending user input (WaitingInput status)
- Dashboard: Max 20x plan cost tip surfaced automatically

**Bug Fixes & Polish (v0.16.0 → v0.16.5)**:
- TUI keybindings `?` / `:` fixed on macOS (KeyModifiers::NONE vs SHIFT)
- Web Activity + Analytics Tools pages fully styled (440+ CSS lines added)
- Web session tooltip positioning fixed (invalid HTML in `<tr>` → `<td>`)
- Pricing table extended: `claude-sonnet-4-6`, dot-style aliases (`claude-sonnet-4.5`, etc.)
- Dashboard plan auto-detection from `~/.claude.json` (`hasAvailableSubscription` / `hasOpusPlanDefault`)
- `cargo install` web embedding fixed (dist/ included in crates.io package)
- Complete user guide: `docs/GUIDE.md` (700 lines, all 12 tabs + CLI reference)

**Performance**:
- 89x faster startup (SQLite cache: 20s → 33ms)
- 50x memory reduction (Arc migration: 1.4GB → 28MB)
- O(delta) live session parsing (vs O(file_size) per tick previously)
- 492 tests passing, 0 clippy warnings

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

### ✅ Tab Bar UX Redesign (v0.14.x) - **DONE**

**Status**: ✅ Done 2026-03-20

Onglets inactifs : icône seule (4 cols). Onglet actif : icône blanc + `[k]` muted + nom cyan+bold. Barre passe de 193 cols à ~85 cols max — plus d'overflow sur terminal standard.
Fichier modifié : `crates/ccboard-tui/src/ui.rs` `render_header()`.

---

### ✅ Phase K-Analytics: Advanced Analytics (v0.15.0) - **DONE**

**Priority**: 🟡 MEDIUM
**Status**: ✅ Released 2026-03-20
**Commits**: f17e747 (implémentation) + c2d315b (10 code review fixes)

Delivered:
- Streak detection, daily cost spikes, model downgrade recommendations
- AnalyticsData owns anomalies/daily_spikes (computed once, cached)
- 11 nouveaux tests — 433 total

---

### ✅ Live Context Window Monitoring + ccboard-ffi (v0.22.0) - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released 2026-04-21

**Delivered**:
- `LiveMonitorState` — stateful incremental JSONL parsing, `HashMap<session_id, TranscriptCache>`, persisted across 2s TUI ticks via `parking_lot::Mutex` in `DataStore`
- Context pressure bar `[████░░░░░░] 42% ⚡` inline in Sessions list; 20-char color-coded detail panel (green/orange/red)
- Compaction detection (context drop >30% → `compaction_count++`, `⚡` marker)
- `context_window_for_model()` heuristic (200K default, 1M for `claude-*-1m` or `max_context_tokens > 200_000`)
- File rotation detection via `(inode, mtime_nanos)` identity tuple
- `ccboard-ffi` crate — UniFFI `staticlib`/`cdylib` for Swift/iOS native dashboards
- `uniffi-bindgen` crate — binding generator binary

---

### ✅ Brain Tab + Third-party Sessions + Code Metrics (v0.21.0) - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released 2026-03-28

**Delivered**:
- Brain tab (key `b`): `session-stop.sh` auto-extracts insights into `~/.ccboard/insights.db`; `session-start.sh` injects context at session start; filterable by type, expandable detail, archive
- Optional claude-mem integration (`m` toggle in Brain tab)
- Third-party session imports: Cursor, Codex CLI, OpenCode — `SourceTool` enum, yellow badge
- Code Metrics: `lines_added`/`lines_removed` from Edit/Write tool inputs; `+N/-N` badge
- `/ccboard-remember` skill for manual insight capture

---

### ✅ Analytics Costs + Pattern Discovery + MCP Stats (v0.20.0) - **DONE**

**Priority**: 🟡 MEDIUM
**Status**: ✅ Released 2026-03-28

**Delivered**:
- Analytics `Costs` sub-tab: per-tool token tracking with red/yellow hotspot coloring
- `Discover` sub-tab: n-gram analysis + Jaccard clustering → CLAUDE.md/Skill/Command suggestions
- MCP server usage stats: Server | Calls | Sessions | Last Seen
- 12 TUI smoke tests (one per tab) — 492 total

---

### ✅ Session Intelligence + Web Live Panel (v0.18.0–v0.19.0) - **DONE**

**Priority**: 🟡 MEDIUM
**Status**: ✅ Released 2026-03-24 → 2026-03-27

**Delivered**:
- Bookmarks, subagent graph, LLM summaries, model switching timeline, context saturation trend
- Waiting Answers panel, Max 20x tip surfacing
- Web Dashboard live sessions panel (polls `/api/sessions/live` every 5s, animated dot)
- Brain tab initial claude-mem integration

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

### ✅ Phase M: Conversation Viewer Enhancements (v0.15.5) - **DONE**

**Priority**: 🟡 MEDIUM
**Status**: ✅ Released 2026-03-20
**Commits**: c213a65, 11426b8, d87a25d, 4520c2e

**Delivered**:
- MA1: Tool call visualization (expandable blocks, 6 tests)
- MA2: Regex search in replay viewer (`/`, `n`/`N`, yellow highlights, 5 tests)
- MA3: HTML export with syntect syntax highlighting (InspiredGitHub theme, 6 tests)
- MA4: FTS5 extended + Search tab detail pane, search-as-you-type, 8 tests

**458 tests total, 0 clippy warnings**

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
| **K-Analytics** | 🟡 MEDIUM | 10-12h | v0.15.0 | Advanced analytics (anomaly, forecasts) | #14-21 | ✅ Done 2026-03-20 |
| **M** | 🟡 MEDIUM | 8-10h | v0.15.5 | Conversation enhancements | #3, #7, #8 | ✅ Done 2026-03-20 |
| **v0.16.x fixes** | — | — | v0.16.0–0.16.4 | Visual redesign, keybindings, web CSS, pricing, plan detection | — | ✅ Done 2026-03-23 |
| **v0.17.0** | — | — | v0.17.0 | Waiting Answers panel + Max 20x tip | — | ✅ Done 2026-03-24 |
| **v0.18.0–v0.19.0** | 🟡 MEDIUM | — | v0.18.0–v0.19.0 | Session intelligence + Web live panel | — | ✅ Done 2026-03-27 |
| **v0.20.0** | 🟡 MEDIUM | — | v0.20.0 | Analytics costs + Pattern Discovery + MCP stats | — | ✅ Done 2026-03-28 |
| **v0.21.0** | 🔴 HIGH | — | v0.21.0 | Brain tab + Third-party sessions + Code metrics | — | ✅ Done 2026-03-28 |
| **v0.22.0** | 🔴 HIGH | — | v0.22.0 | Live context window monitoring + ccboard-ffi | — | ✅ Done 2026-04-21 |
| **L** | 🟢 LOW | 12-15h | v0.23.0 | Plugin system | — | 📋 Backlog |
| **N** | 🟢 LOW | 10-14h | v0.24.0 | Plan-aware completion | #4, #10-13 | 📋 Backlog |

**Total Estimated**: 46-59h for v1.0.0 completion

---

## 🎯 Success Metrics

| Metric | Baseline (v0.8.0) | Target (v1.0.0) |
|--------|-------------------|-----------------|
| **Startup Time** | 33ms | <50ms |
| **Memory Usage** | 28MB | <50MB |
| **Session Render** | <500ms (1000 msgs) | <500ms |
| **Export Speed** | N/A | <2s for 1000 sessions |
| **Test Coverage** | 492 tests | 500+ tests |
| **Bug Reports** | 0 critical | <5% error rate |

---

## 🔭 Opportunities Backlog (v0.17.x+)

A full feature opportunity analysis was conducted on 2026-03-24. See **[OPPORTUNITIES.md](OPPORTUNITIES.md)** for the complete catalog (44 opportunities across 3 categories).

**Quick summary**:

| Category | Count | Examples |
|---|---|---|
| 🟢 Quick Wins (S, 2-4h) | 7 | Invocation counts, settings hot-reload, model switching timeline, context saturation trend |
| 🟡 Medium Features (M, 4-8h) | 9 | Per-tool cost attribution, session bookmarks, MCP health dashboard, subagent graph |
| 🔴 Strategic (L, 8-15h) | 4 | Git integration (Phase N prereq), activity timeline, Prometheus metrics, LLM summaries |

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

**Last Updated**: 2026-04-21
**Maintainer**: @FlorianBruniaux
**Repository**: https://github.com/FlorianBruniaux/ccboard
