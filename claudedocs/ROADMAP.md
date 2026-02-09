# ccboard Roadmap - Post v0.5.0

## Current Status (2026-02-09)

✅ **Phase G Complete**: Full TUI/Web parity (100%)
- 9 TUI tabs fully functional (Dashboard, Sessions, Config, Hooks, Agents, Costs, History, MCP, Analytics)
- 5 Web pages (Dashboard, Sessions, Analytics, Config, History)
- SSE live updates for real-time monitoring
- Sprint 1 UX improvements (config modal, elevation system, responsive design)
- SQLite metadata cache with 89x speedup (20s → 224ms)
- Arc migration achieving 50x memory reduction (1.4GB → 28MB)

🔄 **Phase A In Progress**: Analytics enhancements
- Token usage forecasting with trend analysis
- Session analytics with model breakdown
- Project leaderboard (planned)
- Session replay (planned)
- Anomaly detection (planned)

---

## Next Phases

### Phase F: Conversation Viewer (Q1 2026)

**Goal**: Full JSONL content display with syntax highlighting

**Features**:
- Display complete session conversations with message threading
- Syntax highlighting for code blocks (Rust, Python, JS, etc.)
- Message filtering (by role, tool usage, timestamps)
- Export conversations (Markdown, JSON, PDF)
- Search within conversation content
- Tool call visualization with input/output display

**Technical**:
- Streaming JSONL parser for large sessions (>100MB)
- Syntax highlighting via `syntect` or `tree-sitter`
- Export pipeline (Markdown → Pandoc → PDF)

---

### Phase H: Plan-Aware (Q2 2026)

**Goal**: Parse and track PLAN.md progress across sessions

**Features**:
- PLAN.md frontmatter parsing (phases, tasks, blockers)
- Task completion tracking across sessions
- Dependency visualization (task graphs)
- Timeline view (planned vs actual completion)
- Phase progress indicators in TUI/Web

**Technical**:
- YAML frontmatter parser for plan metadata
- Task dependency resolution (DAG construction)
- Session-to-task mapping via TodoWrite events
- D3.js integration for web dependency graphs

---

### Phase 11: Tokens & Invocations (Q2 2026)

**Goal**: Deep analytics for token usage and tool invocations

**Features**:
- Token usage per tool (Read, Write, Bash, etc.)
- Token usage per agent (explore, plan, test-runner)
- Invocation patterns analysis (most used tools, chains)
- Cost optimization suggestions (detect expensive patterns)
- Token heatmap per session/project
- Tool efficiency metrics (tokens per invocation)

**Technical**:
- JSONL parser enhancement for `usage` object extraction
- Tool invocation counting via message role analysis
- Cost calculation per tool (using Claude pricing)
- Heatmap visualization (TUI: ratatui charts, Web: Chart.js)

---

### Phase 12: Write Operations (Q3 2026)

**Goal**: Enable configuration editing from TUI/Web

**⚠️ Safety-First Design**:
- Backup before any write operation (`~/.claude/.backups/`)
- Dry-run mode showing diff before applying
- Rollback capability (restore from backup)
- Audit log for all config changes

**Features**:
- Edit settings.json from Config tab (global, project, local)
- Create/edit/delete hooks from Hooks tab
- Create/edit agents/commands/skills from Agents tab
- Toggle MCP servers from MCP tab
- Settings validation before write (JSON schema)

**Technical**:
- JSON schema validation via `jsonschema` crate
- YAML frontmatter validation for agents/commands/skills
- Atomic writes (temp file → rename pattern)
- File watching integration for live reload after edit

---

### Phase 13: Team & Collaboration (Q3 2026)

**Goal**: Multi-user ccboard for team Claude Code monitoring

**Features**:
- Central server aggregating multiple `~/.claude` directories
- Team dashboard (all members' stats aggregated)
- Project sharing (multiple devs working on same project)
- Cost allocation per developer
- Leaderboard (tokens, sessions, projects)

**Technical**:
- Server mode: `ccboard server --port 8080 --team-config team.toml`
- Agent-based directory sync (poll each dev's `~/.claude`)
- User authentication (token-based, no passwords)
- PostgreSQL backend for multi-user data

---

### Phase 14: IDE Integrations (Q4 2026)

**Goal**: Launch ccboard from IDEs (VS Code, Neovim, JetBrains)

**Features**:
- VS Code extension: Show session stats in sidebar
- Neovim plugin: Floating window with dashboard
- JetBrains plugin: Tool window with config viewer
- Quick actions: Resume session, search history, open config

**Technical**:
- VS Code: TypeScript extension + Webview API
- Neovim: Lua plugin + terminal buffer
- JetBrains: Kotlin plugin + Tool Window API
- IPC via `ccboard --json` commands (JSON RPC)

---

### Phase 15: CI/CD Integration (Q4 2026)

**Goal**: Generate reports for CI pipelines

**Features**:
- CLI report generation: `ccboard report --format json --since 1d`
- Token budget enforcement (fail CI if budget exceeded)
- Session quality gates (fail if error rate > 10%)
- Cost tracking per PR/branch
- GitHub Actions integration (automatic reports on PRs)

**Technical**:
- `ccboard report` command with JSON/Markdown/HTML output
- Exit codes for CI failures (non-zero if budget exceeded)
- GitHub Action manifest (`action.yml`)
- Artifact upload for report storage

---

## Archived/Completed Phases

For historical phases (Phase I-VI, Phase G), see:
- `claudedocs/archive/roadmap-phase-i.md` (original roadmap)
- `CHANGELOG.md` (detailed release history)

---

## Contributing to Roadmap

Have ideas for new features? Open a [GitHub Discussion](https://github.com/FlorianBruniaux/ccboard/discussions) or [Issue](https://github.com/FlorianBruniaux/ccboard/issues) with:
- **Use case**: What problem does it solve?
- **Priority**: How critical is it?
- **Complexity**: Rough estimate (hours/days/weeks)
- **Alternatives**: Any existing solutions?

Maintainer will review and potentially add to roadmap with priority/phase assignment.

---

**Last Updated**: 2026-02-09
**Current Version**: v0.5.0
