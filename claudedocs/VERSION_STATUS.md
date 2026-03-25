# ccboard - Version Status

**Last Updated**: 2026-03-24
**Current Version**: v0.17.0
**Status**: 🟢 Production Ready
**Working Directory**: `/Users/florianbruniaux/Sites/perso/ccboard`
**Branch**: `main`
**Next**: Phase L (Plugin System, v0.18.0) or Quick Wins batch from OPPORTUNITIES.md

---

## 📦 Current Release: v0.17.0

**Release Date**: 2026-03-24
**GitHub Tag**: [v0.17.0](https://github.com/FlorianBruniaux/ccboard/releases/tag/v0.17.0)
**Crates.io**: Published ✅
**Homebrew**: Updated ✅
**Tests**: 458 passing · 0 clippy warnings

### Release Highlights

**Waiting Answers Panel + Max 20x tip** — Live monitoring improvements.

#### Features
- Sessions tab: "Waiting Answers" panel — sessions pending user input (WaitingInput status)
- Dashboard: Max 20x plan cost tip surfaced automatically

### Version History (abbreviated)

| Version | Date | Highlights |
|---|---|---|
| v0.17.0 | 2026-03-24 | Waiting Answers panel, Max 20x tip |
| v0.16.4 | 2026-03-23 | Unknown Plan fix, missing model IDs |
| v0.16.0 | 2026-03-22 | Visual redesign, palette system, responsive heatmap |
| v0.15.5 | 2026-03-20 | Phase M: Tool call viz, regex search, HTML export, FTS5 |
| v0.15.0 | 2026-03-20 | Phase K-Analytics: streaks, cost spikes, model recommendations |
| v0.14.0 | 2026-03-19 | Phase Hook-Monitor: live session monitoring, ccboard setup |
| v0.13.0 | — | Tool token analytics, optimization suggestions |
| v0.12.0 | 2026-03-13 | ccboard discover — CLAUDE.md optimizer |
| v0.11.0 | 2026-03-05 | Activity tab, FTS5 Search, SQLite cache v5 |
| v0.10.0 | 2026-02-18 | Export features (CSV/JSON/MD/HTML) |

---

## 📦 Previous Release: v0.9.0

**Release Date**: 2026-02-18
**GitHub Tag**: [v0.9.0](https://github.com/FlorianBruniaux/ccboard/releases/tag/v0.9.0)
**Crates.io**: Published ✅

### Release Highlights

**Light Mode & Theme Persistence** - Full light theme activated via `Ctrl+T`, with automatic persistence across sessions.

#### Core Features
- **Light mode** — 11 tabs + 5 components migrated to centralized `Palette` system
- **Theme persistence** — saved to `~/.claude/cache/ccboard-preferences.json`, reloaded on startup
- **`Palette` struct** in `theme.rs` — semantic color bundle (`fg`, `bg`, `muted`, `border`, `focus`, `success`, `error`, `warning`, `important`) adapated per `ColorScheme`
- **Frame background reset** on each render (`Clear` + `Block` with `bg(p.bg)`) — prevents invisible text in light mode

---

## 📦 Previous Release: v0.8.0

**Release Date**: 2026-02-16
**GitHub Tag**: [v0.8.0](https://github.com/FlorianBruniaux/ccboard/releases/tag/v0.8.0)

### Release Highlights

**Budget Tracking & Quota Management** - Complete cost monitoring system with intelligent prorata calculation and multi-level alerts.

#### Core Features
- **Month-to-Date (MTD) Cost Calculation**
  - Token-based prorata: `total_cost * (mtd_tokens / total_tokens)`
  - No pricing lookup needed - simple ratio-based calculation
  - Graceful handling of missing daily data

- **Monthly Projection & Budget Tracking**
  - Projects month-end cost: `(MTD cost / current_day) * 30`
  - Configurable monthly limits via `settings.json`
  - Four-level alert system (Safe/Warning/Critical/Exceeded)

- **Visual Integration**
  - TUI: Color-coded gauge in Costs tab Overview
  - Web: Progress bar with real-time SSE updates
  - Alert colors: Green/Yellow/Red/Magenta based on usage %

#### Configuration

Add to `~/.claude/settings.json`:

```json
{
  "budget": {
    "monthlyLimit": 50.0,
    "warningThreshold": 75.0,
    "criticalThreshold": 90.0
  }
}
```

### Build Status

- ✅ **Compilation**: 0 errors, 0 warnings
- ✅ **Tests**: 344 passing (all crates)
- ✅ **Clippy**: 0 warnings
- ✅ **Deployment**:
  - Crates.io: `ccboard`, `ccboard-core`, `ccboard-tui`, `ccboard-web`
  - Homebrew: `florianbruniaux/tap/ccboard`

---

## 🎯 Version History

| Version | Date | Highlights | Status |
|---------|------|------------|--------|
| **v0.10.0** | 2026-02-18 | Export features (sessions/stats/billing CSV/JSON/MD) | ✅ Current |
| v0.9.0 | 2026-02-18 | Light mode, theme persistence (Ctrl+T) | ✅ Released |
| v0.8.0 | 2026-02-16 | Budget tracking, quota management | ✅ Released |
| v0.7.0 | 2026-02-13 | Conversation viewer, full-text search | ✅ Released |
| v0.6.5 | 2026-02-12 | Dynamic pricing (LiteLLM) | ✅ Released |
| v0.5.0 | 2026-02-09 | Web UI Sprint 1 (UX improvements) | ✅ Released |
| v0.4.0 | 2026-02-06 | Quick Wins (QW5-7), pre-commit hooks | ✅ Released |
| v0.3.0 | 2026-02-05 | Analytics enhancements, heatmap | ✅ Released |

See [CHANGELOG.md](../CHANGELOG.md) for complete version history.

---

## 📊 Feature Completeness

### TUI (Terminal User Interface) - 100%

**9 Interactive Tabs** - All fully functional:

| Tab | Status | Features |
|-----|--------|----------|
| **Dashboard** | ✅ | KPIs, model usage, 7-day activity, MCP count |
| **Sessions** | ✅ | Project tree, live processes, search, detail view |
| **Config** | ✅ | 4-column merge, MCP detail, edit with `e` |
| **Hooks** | ✅ | Syntax highlighting, test mode, badges |
| **Agents** | ✅ | Frontmatter parsing, invocation stats |
| **Costs** | ✅ | 4 views (Overview + Quota, By Model, Daily, Billing) |
| **History** | ✅ | Full-text search, export CSV/JSON |
| **MCP** | ✅ | Status detection, env masking, copy commands |
| **Analytics** | ✅ | Budget tracking, forecast, heatmap, insights |

### Web UI - 100% TUI Parity ✅

**9 Pages** - Feature-complete:

- Dashboard, Sessions (with live monitoring), Analytics
- Config, Hooks, MCP, Agents, Costs, History
- Real-time updates via Server-Sent Events (SSE)
- Leptos WASM frontend + Axum API backend

### Performance Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| **Startup Time** | <2s | 33ms (warm cache) | ✅ 89x faster |
| **Session Render** | <500ms | <500ms (1000 msgs) | ✅ |
| **Memory Usage** | <100MB | 28MB (Arc migration) | ✅ 50x reduction |
| **Cache Hit Rate** | >95% | >99% | ✅ |
| **Tests** | 100% pass | 344/344 | ✅ |

---

## 🚀 Next Steps

See [NEXT_STEPS.md](NEXT_STEPS.md) for Phase J (Export Features) implementation plan.

### Roadmap Preview

- **v0.10.0** - Phase J: Export Features (sessions, stats, billing → CSV/JSON/MD)
- **v0.11.0** - Phase K: Advanced Analytics (anomaly detection, usage patterns)
- **v0.12.0** - Phase L: Plugin System (extensible architecture)
- **v1.0.0** - Stabilization + production hardening

---

## 📂 Documentation Structure

```
ccboard/
├── README.md              # User documentation
├── CHANGELOG.md           # Complete version history
├── CLAUDE.md              # Development guidelines
├── ARCHITECTURE.md        # Technical architecture
│
└── claudedocs/
    ├── VERSION_STATUS.md  # This file (current state)
    ├── ROADMAP.md         # Future phases roadmap
    ├── NEXT_STEPS.md      # Phase J detailed plan
    │
    └── archive/           # Historical docs
        ├── sessions/      # Session-specific recaps
        ├── versions/      # Version-specific plans
        └── phases/        # Phase implementation details
```

---

## 🔧 Development Quick Start

```bash
# Verify working directory
pwd  # /Users/florianbruniaux/Sites/perso/ccboard
git status
git branch

# Run TUI
cargo run

# Run Web UI (production)
trunk build --release && cargo run -- web

# Run Web UI (dev with hot reload)
cargo run -- web --port 8080  # Terminal 1
trunk serve --port 3333       # Terminal 2

# Quality checks
cargo fmt --all && cargo clippy --all-targets && cargo test --all
```

---

## 🐛 Known Issues

### #44 — Web UI non-functional after `cargo install`

**Symptom**: Running `ccboard web` after `cargo install ccboard` shows "Backend API only" instead of the full frontend.

**Root cause**: The Leptos/WASM frontend must be built with `trunk build --release` and bundled separately — it is not embedded in the Rust binary. Users installing from crates.io don't get the frontend assets.

**Workaround**: Build from source and serve with `trunk serve`, or use the Homebrew tap which includes pre-built assets.

**Status**: Open — requires embedding WASM assets into the binary or providing a separate download step.

Report issues at: https://github.com/FlorianBruniaux/ccboard/issues

---

## 📝 Session Continuity

**For new Claude Code sessions**, use this context:

```
Reprend ccboard à /Users/florianbruniaux/Sites/perso/ccboard

État actuel: v0.10.0 production-ready (2026-02-18)
- Export features déployé (Phase J) : ccboard export sessions/stats/billing --format csv|json|md
- Light mode + theme persistence (Ctrl+T) — v0.9.0
- Budget tracking & quota management — v0.8.0
- 344 tests passing, 0 warnings
- TUI + Web UI 100% feature parity

Prochaine phase: Phase K - Advanced Analytics (v0.11.0)
Voir claudedocs/ROADMAP.md pour le plan.

Documentation centrale: claudedocs/VERSION_STATUS.md (ce fichier)
```

---

**Maintainer**: @FlorianBruniaux
**Repository**: https://github.com/FlorianBruniaux/ccboard
**Crates.io**: https://crates.io/crates/ccboard
