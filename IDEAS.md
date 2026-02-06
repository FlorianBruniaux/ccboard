# Ideas & Roadmap

> Public roadmap for ccboard feature development. Transparent priorities and community-driven direction.

---

## Done ✅

### Phase 0-E: Foundation & Polish
- ✅ **SQLite metadata cache** (Phase 2) — 89x speedup (20s → 224ms)
- ✅ **9 interactive tabs** (Phases A-E) — Dashboard, Sessions, Config, Hooks, Agents, Costs, History, MCP, Analytics
- ✅ **Analytics pipeline** (Phase E) — Trends, forecasting, patterns, insights
- ✅ **API usage estimation** (Phase I) — Plan-based budgets (Pro/Max5/Max20)
- ✅ **Live monitoring** (Phase C.4) — Process detection, CPU/RAM metrics
- ✅ **Export CSV/JSON** (Phase C.2) — Sessions, billing blocks, history
- ✅ **File watcher** (Phase B) — Adaptive debounce, burst detection
- ✅ **Arc migration** (Phase D) — 50x memory reduction
- ✅ **Production screenshots** (Phase A.6) — 13 screenshots, all tabs documented

---

## High Priority (P1)

### Conversation Viewer
**Status**: Planned for Phase F (1 week effort)

**Justification**: Killer feature, **NO active TUI competitor** does this.
- Sniffly (1.1K stars) had it → STALE 6 months
- Claudelytics had it → STALE 8 months
- vibe-kanban (20.5K stars) = Web UI, different scope

**Implementation**:
- Phase 1: Message list view (role, model, tokens, timestamp) — 2-3 days
- Phase 2: Syntax highlighting (markdown/code blocks via syntect) — 1-2 days
- Phase 3: Tool calls expansion, image preview (sixel/iTerm2) — 3-4 days

**Impact**: Unique in actively-maintained TUI ecosystem.

**Target**: After distribution traction (100+ stars)

---

### GIF Demo (Critical for Distribution)
**Status**: Script ready (`demo.tape`), needs execution

**Effort**: 30 minutes (run `vhs demo.tape`)

**Blocker**: Must have before Reddit/HN posts

**Impact**: Hero animation README, 3x engagement vs static screenshots

---

## Medium Priority (P2)

### Web UI Leptos Frontend
**Status**: Backend ready (Axum + SSE + 4 API routes), frontend placeholder only

**Current Reality**:
- ✅ Backend: Axum server, /api/stats, /api/sessions, /api/health, SSE streaming
- ❌ Frontend: "Coming soon" text, no Leptos UI components

**Justification**: Differentiator dual-mode unique. But not urgent if TUI distribution first.

**Effort**: 8-12 days (focused on high-value tabs)
- Dashboard → Sessions → Analytics (high value, 4-5 days)
- Config → Hooks → Agents (medium value, 3-4 days)
- Costs → History → MCP (completeness, 2-3 days)

**Target**: After Conversation Viewer (Phase G) OR crates.io traction

---

### Plan-Aware Monitoring
**Status**: Researched, not implemented

**What**: Detect user's Claude plan (Pro $18, Max5 $35, Max20 $140) for accurate budget forecasting.

**Reference**: Usage-Monitor (6.4K stars, STALE 7 months) had this feature.

**Challenge**: No API to query plan, must infer from:
- Rate limits (requests/day patterns)
- Billing block analysis
- User manual config (`subscriptionPlan` setting)

**Effort**: 3 days (heuristics + UI)

**Priority**: P2 (nice-to-have, manual config acceptable for MVP)

---

## Lower Priority (P3)

### P90 Predictions (Numpy-like)
**Status**: Linear regression sufficient for MVP

**Current**: Linear regression + R-squared (good enough)

**Upgrade**: P90 percentile predictions (like Usage-Monitor with numpy)

**Effort**: 3 days (integrate `statrs` crate, 192h historical window)

**ROI**: Low (linear reg works, marginal improvement)

**Target**: If user requests (0 so far)

---

### Burn Rate Real-Time
**Status**: Live monitor = process detection, not burn rate

**What**: Tokens/min, tokens/hour, daily forecast in real-time.

**Reference**: Usage-Monitor had this (now stale).

**Effort**: 2 days (token tracking + display)

**Priority**: P3 (post-session analytics sufficient for now)

---

## Watching (Waiting for Demand)

### Menu Bar App (0 requests)
**Status**: Rejected — 5+ competitors, macOS only

**Competitors**:
- CodexBar (4.4K stars)
- CCSeva (748 stars)
- ClaudeBar, BurnRate, Claude Usage Tracker

**Decision**: Not our niche. TUI cross-platform = unique value.

**Reevaluation trigger**: 5+ user requests for menu bar mode.

---

### Multi-Provider Support (0 requests)
**Status**: Rejected — Claude Code = 95% market, complexity not justified

**What**: Support Cursor, Codex, OpenAI Code Interpreter.

**Challenge**: Different session formats, MCP configs, billing models.

**Effort**: 4-6 weeks (parsers, models, UI abstraction)

**Decision**: Focus on Claude Code excellence vs multi-provider mediocrity.

**Reevaluation trigger**: Claude Code market share drops <80%.

---

## Discarded Ideas

| Idea | Reason Discarded |
|------|------------------|
| **GUI Desktop** | claudia 20.4K stars (even if stale), too late to compete |
| **Kanban Workflow** | vibe-kanban 20.5K stars, dominant, different target |
| **Status Line** | 5+ implementations (ccstatusline 2.7K), scope too limited |
| **Browser Extension** | Out of scope, requires Chrome/Firefox APIs |
| **VS Code Extension** | Claude Code = CLI tool, VS Code has native support |
| **Mobile App** | Terminal monitoring on mobile = poor UX |
| **Cloud Sync** | Privacy concerns, adds complexity, local-first philosophy |
| **Team Dashboards** | Enterprise feature, solo dev focus for MVP |
| **AI Recommendations** | Gimmick, users want data not suggestions |

---

## Community Input

**Feature requests welcome!** If you want a feature:

1. **Check this document** — might be planned/discarded
2. **Search existing issues** — avoid duplicates
3. **Open a discussion** — explain use case, not just "add X"
4. **Provide context** — how many users affected, workarounds tried

**What increases priority**:
- 3+ users request same feature
- Clear use case with examples
- Willingness to contribute (code, testing, docs)

**What doesn't**:
- "It would be cool if..."
- Feature parity with unrelated tools
- Hypothetical scenarios without real users

---

## Reevaluation Triggers

| Idea | Current Status | Reevaluate If |
|------|---------------|---------------|
| **Menu bar app** | Watching (0 requests) | 5+ user requests |
| **Multi-provider** | Watching (0 requests) | Claude market share <80% |
| **P90 predictions** | P3 Low | Users complain linear reg inaccurate |
| **Burn rate real-time** | P3 Low | 10+ requests for real-time monitoring |
| **Plan-aware** | P2 Medium | API endpoint becomes available |

---

## Contributing Ideas

Found something that should be here?

1. Open a [GitHub Discussion](../../discussions) with "Idea:" prefix
2. Explain why it matters (use case, users affected)
3. Link to examples if available (competitors, blog posts)
4. Estimate effort if technical (easy/medium/hard)

**We'll add it to this document** with appropriate priority.

---

**Last updated**: 2026-02-06
**Next review**: After distribution (target 100+ stars)

**Recent changes**:
- Web status clarified (backend ready, frontend TODO)
- Quick wins added from usage report (pre-commit hook, /ship skill, CLAUDE.md)
- Documentation cleanup (28 files archived)
