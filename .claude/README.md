# ccboard — Claude Code Configuration

## Agents (10)

| Agent | Model | Purpose |
|-------|-------|---------|
| `rust-ccboard` | sonnet | Rust/Leptos implementation — primary coding agent |
| `leptos-designer` | sonnet | Leptos UI components, Ratatui TUI widgets |
| `ui-designer` | sonnet | UI/UX design, accessibility, design systems |
| `backend-architect` | sonnet | Axum backend design, API reliability |
| `system-architect` | opus | System-level architecture, long-term decisions |
| `product-designer` | sonnet | Product design, user flows |
| `code-reviewer` | sonnet | Code quality, security review |
| `architect-review` | sonnet | Architecture review, scalability |
| `debugger` | sonnet | Root cause analysis, error resolution |
| `technical-writer` | sonnet | Documentation, API references |

## Skills (11)

| Skill | Trigger |
|-------|---------|
| `backend-architect` | Backend design requests |
| `ccboard-remember` | Persist key decisions across sessions |
| `code-simplifier` | Code cleanup, DRY/KISS improvements |
| `cybersec` | Security audit, Rust/Axum/WASM |
| `design-patterns` | Architecture patterns |
| `issue-triage` | GitHub issue triage and classification |
| `performance` | Web performance, Core Web Vitals |
| `pr-triage` | PR review and classification |
| `security-guardian` | Security-first code review |
| `ship` | Release and deployment workflows |
| `tdd-rust` | TDD cycle for Rust (red-green-refactor) |

## Commands (2)

| Command | Description |
|---------|-------------|
| `/diagnose` | Diagnose Rust/Cargo build environment |
| `/diagram` | Generate architecture diagrams |

## Hooks

| Event | Script | Purpose |
|-------|--------|---------|
| `SessionStart` | `session-start.sh` | Load Brain context (once per session) |
| `Stop` | `session-stop.sh` | Save session insights to Brain SQLite DB |
| `PreToolUse/Bash` | `pre-commit-format.sh` | Run `cargo fmt` before git commit |
| `Notification` | `notification.sh` | macOS desktop notifications (budget alerts) |

## Rules

- `rust-patterns.md` — idiomatic Rust patterns for this codebase
- `search-strategy.md` — grep/ripgrep strategies for Rust code navigation
