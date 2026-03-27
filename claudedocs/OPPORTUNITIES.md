# ccboard — Feature Opportunities Catalog

**Last Updated**: 2026-03-27
**Analysis Date**: 2026-03-24
**Version at Analysis**: v0.17.0 → updated to v0.18.0
**Method**: 3-agent parallel exploration (UX/Product, Technical, Ecosystem angles)

---

## Context

44 opportunities identified and deduplicated from three analysis angles:
- **UX/Product** — workflow gaps, friction points, inspiration from lazygit/k9s/Grafana
- **Technical** — test coverage, performance, architecture, reliability
- **Ecosystem** — unparsed Claude Code data sources, hook events, missing integrations

Deliberately excluded: Phase L (Plugin System), Phase N (Plan-Aware) — already in ROADMAP backlog. Team mode, write operations — post-v1.0 or out of scope.

---

## 🟢 Quick Wins (S — 2-4h each)

### QW1 · Real Invocation Counts (Tools tab)
**Problem**: `invocation_count: 0` hardcoded in Tools tab. Frontmatter parsed but never cross-indexed with sessions.
**Solution**: Aggregate counts from existing `InvocationStats` → display in Tools tab.
**Files**: `ccboard-core/src/analytics/invocations.rs`, `ccboard-tui/src/tabs/agents.rs`
**Impact**: Unlocks dead plugin detection + `ccboard export recommendations` cleanup.
**Status**: 📋 Backlog

---

### QW2 · Settings Hot-Reload (no restart)
**Problem**: Changing `settings.json` (budget limit, thresholds) requires restarting the TUI.
**Solution**: File watcher already detects changes → add `DataEvent::SettingsUpdated` + subscribe in TUI app loop.
**Files**: `ccboard-core/src/watcher.rs`, `ccboard-core/src/store.rs`, `ccboard-tui/src/app.rs`
**Status**: ✅ Done — v0.17.0 (toast "Settings reloaded" on settings.json change)

---

### QW3 · Model Switching Timeline (Sessions detail)
**Problem**: Mid-session model switches (Opus → Sonnet → Haiku) are invisible. No cost impact shown.
**Solution**: Scan `ConversationMessage.model` (already parsed), detect transitions, display in detail pane.
**Files**: `ccboard-core/src/models/session.rs`, `ccboard-tui/src/tabs/sessions.rs`
**Status**: ✅ Done — v0.18.0 (`Opus 4.5 (8) → Sonnet 4.6 (15)` in detail pane, computed inline during JSONL scan)

---

### QW4 · Context Saturation Trend (Dashboard widget)
**Problem**: Context saturation calculated per-session, no 7-day trend or predictive alert.
**Solution**: Linear regression over last 30 sessions → widget "⚠️ Likely to exceed 85% in 3 sessions".
**Files**: `ccboard-core/src/analytics/mod.rs`, `ccboard-tui/src/tabs/dashboard.rs`
**Status**: ✅ Done — v0.18.0 (linear regression → `↑ ~N sessions` / `↓ declining`, `ContextWindowStats.trend_slope`)

---

### QW5 · Hook State Cleanup (stale entries)
**Problem**: Live sessions remain "Running" after crash or kill -9. No TTL on live-sessions.json entries.
**Solution**: Explicit TTL (10min) on session entries + cleanup on startup + heartbeat validation.
**Files**: `ccboard-core/src/hook_state.rs`
**Status**: ✅ Done — v0.18.0 (`prune_stale_running(max_age)` on `LiveSessionFile`, 10-min TTL)

---

### QW6 · Session Replay Complexity Warning
**Problem**: Opening a session with 5000+ tool calls can freeze the viewer. No pre-flight warning.
**Solution**: Complexity score (tool call count) in SessionMetadata → warning modal before loading.
**Files**: `ccboard-tui/src/tabs/sessions.rs`, `ccboard-core/src/models/session.rs`
**Status**: 📋 Backlog

---

### QW7 · Configurable Anomaly Thresholds
**Problem**: Anomaly detection hardcoded (2x spike, 3-sigma outlier). Doesn't fit all usage patterns.
**Solution**: `anomaly_config` section in `settings.json` + TUI modal for live adjustment.
**Files**: `ccboard-core/src/analytics/anomalies.rs`, `ccboard-core/src/models/config.rs`
**Status**: 📋 Backlog

---

## 🟡 Medium Features (M — 4-8h each)

### MF1 · Per-Tool-Call Cost Attribution
**Problem**: Costs aggregated per session. Impossible to know if `WebFetch` costs 10x more than `Read`.
**Solution**: Extend `ToolCall` with estimated token count, cross-reference pricing table. New sub-tab "Tool Cost Breakdown" in Analytics.
**Files**: `ccboard-core/src/parsers/activity.rs`, `ccboard-core/src/analytics/`
**Status**: 📋 Backlog

---

### MF2 · Session Bookmarks & Tags
**Problem**: No way to organize important sessions (breakthroughs, bugs, useful patterns).
**Solution**: Lightweight bookmark system in `~/.ccboard/bookmarks.json` + arbitrary tags + filter in Sessions tab.
**Files**: New `ccboard-core/src/bookmarks.rs`, `ccboard-tui/src/tabs/sessions.rs`
**Status**: ✅ Done — v0.18.0 (`b` toggle, `B` filter starred, `★` icon, `BookmarkStore` persisted to `~/.ccboard/bookmarks.json`)

---

### MF3 · MCP Server Health Dashboard
**Problem**: MCP servers listed but no latency/failure tracking. Impossible to identify which one slows sessions.
**Solution**: Track MCP calls via hook events, aggregate avg latency + failure rate per server. New sub-tab in MCP tab.
**Files**: `ccboard-core/src/hook_event.rs`, `ccboard-tui/src/tabs/mcp.rs`
**Status**: 📋 Backlog

---

### MF4 · TodoWrite Burndown Charts
**Problem**: `todowrite.rs` parser exists but no progress visualization.
**Solution**: Link todos to sessions (via `session_id`), compute velocity, burndown chart per project.
**Files**: `ccboard-core/src/parsers/todowrite.rs` (extend), `ccboard-tui/src/tabs/analytics.rs`
**Status**: 📋 Backlog

---

### MF5 · Session Comparison View (side-by-side)
**Problem**: Cannot compare 2-4 sessions side-by-side (tokens, cost, duration, model, tools).
**Solution**: "Compare" mode in Sessions tab via checkbox multi-select + comparative table.
**Files**: `ccboard-tui/src/tabs/sessions.rs`
**Status**: 📋 Backlog

---

### MF6 · Subagent Session Graph (parent/child tree)
**Problem**: Subagent sessions listed flat. No hierarchy visualization for TeamCreate workflows.
**Solution**: Detect `parent_session_id` in JSONL, render tree view in Sessions tab, aggregate per-agent token costs.
**Files**: `ccboard-core/src/models/session.rs`, `ccboard-tui/src/tabs/sessions.rs`
**Status**: ✅ Done — v0.18.0 (`⤵ Subagents (N)` tree in detail pane, `has_subagents` backfilled at load, `subagent_children()` DataStore API)

---

### MF7 · External Budget Alerts (Slack/Discord webhook)
**Problem**: Budget alerts only visible in-app. No external notification when thresholds hit.
**Solution**: Configurable webhook in `settings.json`, triggered on existing 4-level alert system.
**Files**: `ccboard-core/src/analytics/budget.rs`, new `ccboard-core/src/notifications.rs`
**Status**: 📋 Backlog

---

### MF8 · Session Retry Pattern Analysis
**Problem**: Retry loops (Read → Read → Read → fail) not detected. No reliability analytics.
**Solution**: Analyze `ToolCall.name` sequences, flag repetitive patterns → alert in Activity tab.
**Files**: `ccboard-core/src/parsers/activity.rs`
**Status**: 📋 Backlog

---

### MF9 · TUI Test Coverage (Ratatui TestBackend)
**Problem**: 0 unit tests in `ccboard-tui`. Regression risk on UI refactors.
**Solution**: Snapshot tests for key tabs, keybinding tests for modals, pagination state tests.
**Files**: `crates/ccboard-tui/tests/` (new)
**Status**: 📋 Backlog

---

## 🔴 Strategic Features (L — 8-15h, architectural decisions)

### SF1 · Git Integration (session ↔ commit correlation)
**Problem**: `branch` field parsed but never cross-referenced with git log. Cannot correlate "session X produced commit Y".
**Solution**: Parse git log for commit messages, track changed files per session, diff view in conversation viewer.
**Prerequisite for**: Phase N (Plan-Aware Completion).
**Files**: New `ccboard-core/src/parsers/git.rs`, `ccboard-tui/src/tabs/sessions.rs`
**Status**: 📋 Backlog

---

### SF2 · Activity Timeline Graph (Web UI first)
**Problem**: Hourly heatmap exists but no temporal session flow visualization.
**Solution**: Timeline view in Analytics — X=date, Y=cost/tokens, dots colored by model/project, hover details. Web UI first (SVG easier), TUI sparkline approximation.
**Files**: `crates/ccboard-web/src/pages/analytics.rs`
**Status**: 📋 Backlog

---

### SF3 · Prometheus Metrics Endpoint
**Problem**: Zero external observability. Cannot integrate ccboard into a monitoring stack.
**Solution**: `/metrics` endpoint — parser latencies, cache hit rate, DataStore lock contention, EventBus overflow.
**Files**: `crates/ccboard-web/src/router.rs`, new `ccboard-core/src/metrics.rs`
**Status**: 📋 Backlog

---

### SF4 · LLM Session Summaries (opt-in)
**Problem**: Impossible to understand a session's purpose without reading the full conversation.
**Solution**: `ccboard summarize <session-id>` → calls `claude --print`, caches in `~/.ccboard/summaries/`, displays in detail pane.
**Files**: `crates/ccboard/src/main.rs`, new `ccboard-core/src/summaries.rs`
**Status**: ✅ Done — v0.18.0 (`ccboard summarize <id>`, `SummaryStore` with atomic write, detail pane shows cached summary or hint)

---

## 🏗️ Infrastructure Improvements

These don't add features but improve reliability and maintainability across the board.

| Item | Problem | Files | Complexity | Status |
|---|---|---|---|---|
| **EventBus overflow detection** | 256-cap broadcast → silent event drops under high load | `ccboard-core/src/event.rs` | S | 📋 Backlog |
| **Session content cache eviction** | Moka cache no TTL → unbounded RAM on long sessions | `ccboard-core/src/store.rs` | S | 📋 Backlog |
| **Activity alerts archival (90d TTL)** | Alerts lost after cache eviction, no historical trends | `ccboard-core/src/cache/metadata_cache.rs` | M | 📋 Backlog |
| **Web API test coverage** | 26 endpoints, 0 tests — API contracts unvalidated | `crates/ccboard-web/tests/` | M | 📋 Backlog |
| **Billing health endpoint** | Silent pricing fallback, no audit log | `ccboard-core/src/models/billing_block.rs` | S | 📋 Backlog |
| **Invocation deduplication** | Retry patterns may overcount identical tool calls | `ccboard-core/src/analytics/invocations.rs` | S | 📋 Backlog |

---

## Prioritization Guide

```
✅ Sprint 1 (v0.17.0) — DONE:
  QW2  Settings hot-reload       ✅
  QW5  Hook state cleanup        ✅

✅ Sprint 2 (v0.18.0) — DONE:
  QW3  Model switching timeline  ✅
  QW4  Context saturation trend  ✅
  MF2  Session bookmarks         ✅
  MF6  Subagent graph            ✅
  SF4  LLM summaries             ✅

Sprint 3 (v0.19.x) — candidates:
  QW1  Invocation counts         ← unblocks dead plugin detection
  QW6  Session replay warning    ← safety UX
  QW7  Configurable thresholds   ← low risk, user-requested pattern
  MF1  Per-tool cost attribution  ← highest analytics impact
  MF3  MCP health dashboard       ← differentiator vs competitors
  MF9  TUI test coverage          ← quality infrastructure

Strategic decision:
  SF1  Git integration            ← decide timing relative to Phase N
  SF2  Activity timeline          ← Web UI, significant visual impact
```

---

## What's NOT Here (by design)

| Feature | Reason excluded |
|---|---|
| Phase L (Plugin System) | Already in ROADMAP backlog |
| Phase N (Plan-Aware) | Already in ROADMAP backlog |
| Team / multi-user mode | Post-v1.0, scope too large |
| Write operations | Core principle: read-only |
| Clipboard history parsing | Low value, privacy concerns |
| Prometheus (full APM) | SF3 is a lighter-weight alternative |

---

**Related**: [ROADMAP.md](ROADMAP.md) · [PLAN.md](PLAN.md) · [NEXT_STEPS.md](NEXT_STEPS.md)
**Last Updated**: 2026-03-27
