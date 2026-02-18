# ccboard Roadmap

**Last Updated**: 2026-02-16
**Current Version**: v0.8.0
**Target**: v1.0.0 (Phases J-L complete)

---

## 🎯 Vision

Transform ccboard from a monitoring dashboard into a **complete Claude Code management platform** with analytics, export, advanced insights, and extensibility.

**Core Principles**:
- ✅ Read-only by default (monitoring focus)
- ✅ Performance first (<2s startup maintained)
- ✅ Claude Code-only (no multi-provider scope creep)
- ✅ Graceful degradation (partial data > crashes)

---

## 📍 Current Status (v0.8.0)

### ✅ Production Features

**TUI + Web UI** (9 tabs, 100% parity):
- Dashboard, Sessions (live monitoring), Config, Hooks, Agents
- Costs (4 views + quota), History (search + export), MCP, Analytics

**Performance**:
- 89x faster startup (SQLite cache: 20s → 33ms)
- 50x memory reduction (Arc migration: 1.4GB → 28MB)
- 344 tests passing, 0 clippy warnings

**Budget Tracking** (v0.8.0):
- Month-to-date cost calculation with token-based prorata
- Monthly projection with configurable limits
- 4-level alerts (Safe/Warning/Critical/Exceeded)
- Visual gauges in TUI + Web UI

---

## 🚀 Upcoming Phases

### Phase J: Export Features (v0.9.0) - **NEXT**

**Priority**: 🔴 HIGH
**Duration**: 6-8h
**Status**: ⏳ Planned

**Goal**: Export sessions, stats, and billing data to external formats for reporting and analysis.

**Features**:
- Export sessions → CSV/JSON (for Excel, data analysis)
- Export stats → Markdown reports (for team sharing)
- Export billing blocks → CSV (for accounting)
- CLI: `ccboard export sessions --format csv --output sessions.csv`

**Value**:
- ✅ Share data with non-technical stakeholders
- ✅ Integrate with external reporting tools
- ✅ Backup session metadata
- ✅ Quick win with immediate ROI

**See**: [NEXT_STEPS.md](NEXT_STEPS.md) for detailed Phase J plan.

---

### Phase K: Advanced Analytics (v0.10.0)

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

### Phase L: Plugin System (v0.11.0)

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

### Phase M: Conversation Viewer Enhancements (v0.11.5)

**Priority**: 🟡 MEDIUM
**Duration**: 8-10h
**Status**: 📋 Backlog (extends Phase F)

**Goal**: Advanced conversation analysis and visualization.

**Features**:
- **Tool Call Visualization**: Expandable nodes with input/output
- **Message Threading**: Conversation flow graphs
- **Export Enhancements**: HTML reports with syntax highlighting
- **Full-Text Search**: Regex support, multi-session search

**Depends on**: Phase F (Conversation Viewer) completed in v0.7.0

---

### Phase N: Plan-Aware Completion (v0.12.0)

**Priority**: 🟢 LOW
**Duration**: 10-14h
**Status**: 📋 Backlog (extends Phase H)

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

| Phase | Priority | Duration | Version | Focus | Status |
|-------|----------|----------|---------|-------|--------|
| **J** | 🔴 HIGH | 6-8h | v0.9.0 | Export features | ⏳ Next |
| **K** | 🟡 MEDIUM | 10-12h | v0.10.0 | Advanced analytics | 📋 Backlog |
| **L** | 🟢 LOW | 12-15h | v0.11.0 | Plugin system | 📋 Backlog |
| **M** | 🟡 MEDIUM | 8-10h | v0.11.5 | Conversation enhancements | 📋 Backlog |
| **N** | 🟢 LOW | 10-14h | v0.12.0 | Plan-aware completion | 📋 Backlog |

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

**Last Updated**: 2026-02-16
**Maintainer**: @FlorianBruniaux
**Repository**: https://github.com/FlorianBruniaux/ccboard
