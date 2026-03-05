# ccboard Roadmap

**Last Updated**: 2026-03-05
**Current Version**: v0.11.0
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

## 📍 Current Status (v0.11.0)

### ✅ Production Features

**TUI + Web UI** (11 tabs, 100% parity):
- Dashboard, Sessions (live monitoring), Config, Hooks, Agents
- Costs (4 views + quota), History (search + export), MCP, Analytics
- **Activity** — security audit, violations feed, on-demand session analysis, batch scan (4 concurrent)
- **Search** — FTS5 full-text search across all sessions with ranked snippets

**Performance**:
- 89x faster startup (SQLite cache: 20s → 33ms)
- 50x memory reduction (Arc migration: 1.4GB → 28MB)
- 377 tests passing, 0 clippy warnings

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

- **#44** — Web UI non-functional after `cargo install` (WASM assets not bundled in binary)

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

### ✅ Phase K-Activity: Activity Security Audit + Search (v0.11.0) - **DONE**

**Priority**: 🔴 HIGH
**Status**: ✅ Released 2026-03-05

**Delivered**:
- Activity tab (TUI) + `/activity` page (Web): per-session security audit
- `parse_tool_calls()` + `classify_tool_calls()` — single-pass JSONL streaming engine
- 6 alert rules: CredentialAccess, DestructiveCommand, ForcePush, ExternalExfil, ScopeViolation
- Violations feed: cross-session, sorted Critical → Warning → Info, with remediation hints
- Batch scan: `Arc<Semaphore>` 4 permits, live counter, SQLite persistence
- Search tab (TUI) + `/search` page (Web): FTS5 full-text search with ranked snippets
- SQLite activity tables: `activity_cache` + `activity_alerts` (cache v5)
- 29 new unit tests in `parsers/activity.rs`, 1 in `models/activity.rs`

---

### Phase K-Analytics: Advanced Analytics (v0.12.0) - **NEXT**

**Priority**: 🟡 MEDIUM
**Duration**: 10-12h
**Status**: 📋 Backlog

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

### Phase L: Plugin System (v0.12.0)

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

### Phase M: Conversation Viewer Enhancements (v0.12.5)

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

### Phase N: Plan-Aware Completion (v0.13.0)

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
| **K-Analytics** | 🟡 MEDIUM | 10-12h | v0.12.0 | Advanced analytics | #14-21 | ⏳ Next |
| **L** | 🟢 LOW | 12-15h | v0.13.0 | Plugin system | — | 📋 Backlog |
| **M** | 🟡 MEDIUM | 8-10h | v0.13.5 | Conversation enhancements | #3, #7, #8 | 📋 Backlog |
| **N** | 🟢 LOW | 10-14h | v0.14.0 | Plan-aware completion | #4, #10-13 | 📋 Backlog |

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

**Last Updated**: 2026-03-05
**Maintainer**: @FlorianBruniaux
**Repository**: https://github.com/FlorianBruniaux/ccboard
