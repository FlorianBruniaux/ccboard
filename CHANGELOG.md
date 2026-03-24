# Changelog

All notable changes to ccboard will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.5] - 2026-03-24

### Fixed

- **Dashboard — "API (Pay-as-you-go)" shown for Max/Pro users**: Reworked plan auto-detection priority. `hasOpusPlanDefault: true` now takes precedence over `hasAvailableSubscription` (which can be false even for active Max subscribers when quota is temporarily unavailable). Also reads `oauthAccount.subscriptionCreatedAt` as a last-resort fallback to detect subscribers whose quota is exhausted.

---

## [0.16.4] - 2026-03-23

### Fixed

- **Pricing — Unknown model IDs causing wrong cost estimates**: Added `claude-sonnet-4-6` (current Claude Code default), dot-style aliases (`claude-sonnet-4.5`, `claude-opus-4.5`, `claude-haiku-4.5`, etc.) to the pricing table. Sessions using these IDs fell back to a weighted average instead of exact pricing.
- **Dashboard — "Unknown Plan" shown for all users**: ccboard now auto-detects subscription plan from `~/.claude.json` (`hasAvailableSubscription` + `hasOpusPlanDefault` fields). Pro and Max plans are detected automatically without requiring manual `subscription_plan` config. Manual override via `subscription_plan` in `settings.json` still takes priority.

---

## [0.16.3] - 2026-03-23

### Fixed

- **Web — `cargo install` build error**: `build-placeholder.html` was missing from the crates.io package include list, causing `build.rs` to fail with "No such file or directory" when installing from crates.io

---

## [0.16.2] - 2026-03-23

### Fixed

- **Web — `cargo install` still missing frontend**: `dist/` was excluded from the crates.io package (gitignored by default); added explicit `include` list in `ccboard-web/Cargo.toml` so the pre-built Leptos/WASM assets are bundled with the published crate

---

## [0.16.1] - 2026-03-23

### Fixed

- **Web — Frontend not embedded in released binary**: `trunk build --release` was not run before publishing 0.16.0, leaving the web UI absent from the binary (placeholder warning at startup); fixed by compiling the Leptos/WASM frontend and embedding it via rust-embed before release

---

## [0.16.0] - 2026-03-23

### Fixed

- **TUI — `?` and `:` keybindings broken**: `ToggleHelp` and `ShowCommandPalette` were registered with `KeyModifiers::SHIFT`, but crossterm sends these characters with `KeyModifiers::NONE` on macOS — pressing `?` now correctly opens the help modal
- **Web — Activity page completely unstyled**: all CSS classes (`stat-card`, `severity-badge`, `filter-btn`, `violation-row`, etc.) were missing from style.css; added ~230 lines for the Security Audit / Activity page
- **Web — Analytics Tools tab unstyled**: suggestion cards, tool breakdown, and forecast sections had no CSS; added ~210 lines covering `.suggestion-card`, `.suggestions-grid`, `.forecast-section`, `.breakdown-section`
- **Web — Dashboard session tooltip positioning**: tooltip was a direct child of `<tr>` (invalid HTML — browsers eject it from the table), causing it to appear at the left edge of the viewport; moved inside `<td class="preview--with-tooltip">` with `position: relative` anchor and `right: 0` alignment

### Changed

- **TUI — Activity heatmap responsive layout**: heatmap now fills all available vertical space (layout constraint changed from `Length(12)` to `Min(10)`); cell width scales with terminal width (`cell_w = grid_w / 24`); cell height clamps between 1 and 5 rows (`cell_h = available_height / 7 clamp(1, 5)`)
- **TUI — Heatmap legend redesign**: swatches enlarged from 1 to 2 chars (`██`), labeled with human-readable levels (`No activity / Low / Medium / High / Peak`) for better readability at a glance

### Documentation

- **README.md**: updated tab reference (11 → 12 tabs with correct keyboard shortcuts), added Quick Start section mentioning `?` help and `:` command palette, replaced broken `claudedocs/` links with `docs/GUIDE.md` and `CHANGELOG.md`
- **`docs/GUIDE.md`** (new): complete 700-line user guide covering all 12 tabs, keyboard shortcuts, conversation viewer, live session monitoring, CLI reference, and tips & tricks

---

## [0.15.5] - 2026-03-20

### Added — Phase M: Conversation Viewer Enhancements (complete)

#### MA1 — Tool Call Visualization (commit c213a65)

- **`extract_tool_use_blocks`**: parses `tool_use` content blocks from assistant messages, returns `(id, name, input_json)` triples — real Claude Code format (content array, not legacy `tool_calls` field)
- **`extract_tool_result_blocks`**: parses `tool_result` content blocks from user messages, returns `(tool_use_id, output_text)` pairs
- **`format_tool_input_summary`**: shows most relevant input param per tool (`file_path` for Read/Write/Edit, `command` for Bash, `pattern` for Grep/Glob, `url` for WebFetch)
- Replay viewer: collapsed shows `▶ 2 tool call(s): Read, Bash [Enter]`, expanded shows tool name bold + key param
- `extract_message_content` now skips `tool_use`/`tool_result` blocks — no more `[tool_use]` noise in previews
- 6 new unit tests

#### MA2 — Regex Search in Replay Viewer (commit 11426b8)

- **`/`** inside the replay viewer activates an inline search bar (separate from the session list search)
- Type to search: tries input as a full regex (`(?i)pattern`), falls back to escaped literal on invalid regex
- **`n`/`N`**: navigate to next/previous hit, wraps around, shows `[2/7]` counter in the search bar
- **Esc**: first press clears the query, second press closes the replay viewer
- Matching text highlighted in yellow in message content (reuses `highlight_matches`)
- `rebuild_replay_search_hits`: O(n) rescan on each keystroke, jumps to first hit automatically
- 5 new unit tests

#### MA3 — HTML Export with Syntax Highlighting (commit d87a25d)

- **`render_content_as_html`**: detects fenced code blocks (` ```lang...``` `) in message content via regex, applies syntect syntax highlighting
- Uses **InspiredGitHub** light theme (matches the HTML export's white background design)
- Language badge (`.code-lang`) above each block, `.code-block` container with border
- Supports Rust, Bash, Python, and 40+ languages via syntect's bundled syntaxes
- Fast path for plain-text messages (no regex scan overhead)
- Graceful fallback to `html_escape` on syntect errors
- `syntect 5.2` added to `ccboard-core` dependencies (`regex-fancy` backend)
- 6 new unit tests

#### MA4 — FTS5 Extended + Search Tab UX (commit 4520c2e)

- **`SearchResult`** now includes `first_timestamp` and `message_count` (SQL join on `session_metadata`)
- FTS5 snippet extended from 8 to 12 tokens for more context
- **Results list**: date displayed in yellow + message count per result
- **Detail pane** (40% width, right side): project, date, session ID, message count, full snippet
- **Search-as-you-type**: auto-refresh FTS5 on every char/backspace (min 2 chars threshold)
- ↑/↓ arrow keys navigate results while in input mode
- Enter opens conversation overlay from both input mode and navigation mode
- Cursor `▌` shown when input mode active
- 8 new unit tests

**458 tests total, 0 clippy warnings**

---

## [0.14.0] - 2026-03-19

### Added — Phase Hook-Monitor: Live Session Monitoring

#### `ccboard hook` subcommand

- **`ccboard hook <EventName>`** — reads Claude Code hook JSON from stdin, updates `~/.ccboard/live-sessions.json` with fd-lock file locking and atomic save (`.tmp` + rename). Called by Claude Code hook events: `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Notification`, `Stop`.
- **`HookSessionStatus` enum**: `Running`, `WaitingInput`, `Stopped`, `Unknown` — state machine driven by event type:
  - `PreToolUse` / `PostToolUse` / `UserPromptSubmit` → `Running`
  - `Notification` with `permission_prompt` reason → `WaitingInput`
  - `Stop` → `Stopped`
  - `UserPromptSubmit` on a previously `Stopped` session → revival back to `Running`
- **Pruning**: stopped sessions older than 30 minutes are removed on each hook invocation.
- **macOS notifications**: non-blocking `osascript` spawn on `Stop` events, project name sanitized against AppleScript injection.
- **File watcher**: auto-watches `~/.ccboard/` directory, fires `DataEvent::LiveSessionStatusChanged` on changes to trigger TUI redraw.

#### `ccboard setup` subcommand

- Injects 5 hooks into `~/.claude/settings.json` idempotently: `PreToolUse`, `PostToolUse`, `UserPromptSubmit`, `Notification`, `Stop`.
- Backup written before any write, atomic save via `.tmp` + rename.
- `--dry-run` flag prints planned changes without modifying files.
- Warning printed when running from a Cargo build directory.

#### TUI Sessions tab — live session display

- **`MergedLiveSession`**: hook data and `ps`-based process data merged by session_id → TTY → cwd fallback.
- **`LiveSessionDisplayStatus`** with colored icons: `Running ●`, `WaitingInput ◐`, `Stopped ✓`, `ProcessOnly 🟢`, `Unknown ?`.
- Session list shows idle time for `WaitingInput` sessions and a detail view with status, TTY, and last event.

#### SessionType detection

- **`SessionType` enum**: `Cli`, `VsCode` (stream-json + stdio), `Subagent` (stream-json only) — detected from Claude Code CLI flags.
- `LiveSession` gains `session_type`, `model` (from `--model`), and `resume_id` (from `--resume`).
- Displayed as `IDE` / `Agent` labels in the session list and detail view.

#### `~/.claude.json` parser (`ClaudeGlobalStats`)

- Parses `~/.claude.json` `projects[].lastModelUsage` fields: `costUSD`, `inputTokens`, `outputTokens`, cache tokens.
- **`ClaudeGlobalStats`**: projects sorted by cost descending, `total_last_cost` aggregate across all projects.
- `DataStore` gains `claude_global_stats()` accessor, populated at startup.
- **Costs tab — "6. Per Project" view**: table with last session cost per project, color-coded by magnitude, with model breakdown.

### Fixed — Security & Correctness

- **AppleScript injection**: project name fully escaped before being passed to `osascript`.
- **ps filter**: `is_claude_process_line()` now checks the COMMAND basename exactly, rejecting `claude-desktop`, shell scripts, and `grep` itself.
- Dead code removed from ps TTY block.
- `dirs::home_dir()` used consistently — `std::env::var("HOME")` was previously used in some paths.
- `eprintln!` warning on JSON corruption in `live-sessions.json` (was: silent reset).
- 10 new tests for `is_claude_process_line` and `parse_claude_flags`.

### Tests

- **419 tests passing** (was 405, +14).

---

## [0.13.0] - 2026-03-15

### Added — Phase K: Tool Cost Analytics

- **`tool_token_usage: HashMap<String, u64>`** on `SessionMetadata` — per-tool token attribution via proportional distribution across `tool_use` blocks in each assistant message (JSONL streaming, no full parse). SQLite cache version bumped to v7 for transparent invalidation.

- **`agent_token_stats: HashMap<String, u64>`** on `InvocationStats` — accumulates Task-tool token spend keyed by agent/command name. Updated `compute_invocations()` in `DataStore` to populate from session data.

- **`ccboard-core/src/analytics/tool_chains.rs`** — bigram/trigram analysis over per-session tool sequences:
  - `ToolChain` (sequence, frequency, sessions_count), `ToolChainAnalysis` (top bigrams, trigrams, most expensive chains)
  - Sliding-window extraction, sorted by frequency descending, capped at top-10 per category
  - 4 unit tests covering bigram extraction, deduplication, empty sessions

- **`ccboard-core/src/analytics/optimization.rs`** — cost optimization suggestion engine:
  - `OptimizationCategory` enum: `UnusedPlugin`, `HighCostTool`, `ModelDowngrade`, `RedundantCalls`
  - `CostSuggestion` with title, description, potential_savings ($/mo), action
  - `generate_cost_suggestions()` flags dead plugins (0 invocations) and tools consuming >20% of total tokens with estimated 30% reduction potential
  - Sorted by potential savings descending
  - 5 unit tests

- **`AnalyticsData`** extended with `tool_chains: Option<ToolChainAnalysis>` and `cost_suggestions: Vec<CostSuggestion>`. Both populated in `compute()`, gracefully omitted in `from_sessions_only()`.

- **TUI Analytics tab — Plugins sub-view** (cycle with `[`/`]`):
  - Per-tool token usage bar chart (top 8 tools)
  - Cost suggestions list with category icon, title, savings estimate, and action

- **Web Analytics page — Tools tab**:
  - `GET /api/analytics/suggestions` — aggregates `tool_token_usage` across all sessions, runs dead-plugin and high-cost-tool analysis, returns sorted `CostSuggestion` list as JSON
  - `AnalyticsTools` Leptos component with Suspense-wrapped suggestion cards showing category, savings estimate, description, and action

---

## [0.12.0] - 2026-03-13

### Added — `ccboard discover`

- **`ccboard discover`** — new CLI subcommand that analyzes session history to suggest what to extract as CLAUDE.md rules, skills, or commands, based on recurring n-gram patterns in user messages.
  - N-gram extraction (3–6 grams) with stop-word filtering, subsumption deduplication, and Jaccard similarity clustering
  - Category assignment: >20% of sessions → CLAUDE.md rule, ≥5% → skill, else → command
  - Cross-project patterns get a 1.5× score bonus (`[cross-project]` badge in output)
  - `--since` accepts `7d`, `30d`, `90d`, or a `YYYY-MM-DD` date (default: `90d`)
  - `--min-count` / `--top` to control noise threshold and result count
  - `--all` to scan all projects (default: current project only)
  - `--json` for machine-readable output (pipe to `jq`)
  - `--llm` mode: deduplicates messages, builds a structured prompt, calls `claude --print` as a subprocess for semantic analysis
  - Async loading with bounded concurrency (32-slot semaphore), CPU-bound n-gram work in `tokio::task::spawn_blocking`
  - 6 new unit tests: normalize, n-gram extraction, Jaccard overlap, category thresholds, cross-project bonus

---

## [0.11.2] - 2026-03-09

### Fixed

- **Homebrew build from source**: `cargo build --all` now succeeds even when `dist/` is absent (e.g. after `brew install --build-from-source`). A `build.rs` script in `ccboard-web` detects a missing or empty `dist/` directory at compile time, creates a placeholder `index.html`, and emits a `cargo:warning` to inform the developer. All `/api/*` endpoints remain fully functional; only the WASM frontend is replaced by a minimal HTML page pointing to the pre-built binaries. Full-stack builds (with `trunk build --release`) are unaffected.

---

## [0.11.1] - 2026-03-06

### Fixed

- **Web interface 404 on Linux/installed binary**: `ccboard web` now works out of the box when installed via `cargo install`, Homebrew, or downloaded from a release tarball. The WASM frontend (JS, CSS, WASM assets) is now embedded directly into the binary using `rust-embed` at compile time — no source tree required at runtime.

---

## [0.11.0] - 2026-03-05

### Added — Activity Security Audit + Search Tab

- **Activity tab** (press `a` from any tab) — on-demand per-session security audit with two views toggled via `Tab`:
  - **Sessions view**: list of all sessions with security badges (`✓ OK` / `⚠ N alerts` / `⟳ scanning`). Press `Enter` to analyze one session individually, `r` to batch-scan all sessions (4 concurrent via Semaphore).
  - **Violations view**: consolidated cross-session alert feed, sorted Critical → Warning → Info then newest first within severity. Each item shows: severity icon, category, truncated detail, session + timestamp context, and a per-category remediation hint. Press `j/k` to navigate.

- **`ccboard-core/src/parsers/activity.rs`** — streaming JSONL security audit engine:
  - Single-pass tool_use extraction with duration computation (tool_use → matching tool_result delta)
  - Classification fan-out: `Read/Write/Edit/Glob/Grep` → FileAccess; `Bash` → BashCommand; `WebFetch/WebSearch` → NetworkCall; `mcp__*` → McpCall
  - `is_destructive_command()`: detects `rm -rf`, `git push --force/-f`, `git reset --hard`, `git clean -f`, `DROP TABLE/DATABASE`, `pkill`, `kill -9`. Multi-command aware — splits on `;`, `|`, `&` to avoid false positives across chained commands.
  - `is_sensitive_file()`: matches `.env`, `.pem`, `id_rsa`, `id_ed25519`, `secrets.json`, `credentials.json`, `.npmrc`, `.netrc`. Public key files (`.pub`) excluded.
  - Credential leak detection in bash output: `sk-` (≥20 alphanum chars), `ghp_`, `ghu_`, `ghs_`, `glpat-`, `AKIA`, `xoxb-`, `xoxp-`
  - Scope violation detection via `Path::starts_with()` (component-based, avoids path prefix false positives)
  - 6 alert generation rules across 5 categories: CredentialAccess, DestructiveCommand, ExternalExfil, ScopeViolation, ForcePush

- **`ccboard-core/src/models/activity.rs`** — activity data model:
  - `AlertCategory::action_hint()` — exhaustive match (compile-time enforcement: adding a variant without a hint is a compile error, no `_` fallback)
  - `AlertSeverity` with `PartialOrd` for sort-by-severity across the violations feed
  - Types: `ToolCall`, `FileAccess`, `BashCommand`, `NetworkCall`, `Alert`, `ActivitySummary`

- **SQLite activity tables** in `session-metadata.db`:
  - `activity_cache` — bincode-serialized `ActivitySummary` per session with mtime for invalidation
  - `activity_alerts` — denormalized queryable alerts (severity, category, timestamp, detail) for cross-session queries
  - Atomic writes: `BEGIN IMMEDIATE / COMMIT / ROLLBACK` — no partial state on crash
  - TOCTOU-free: single `tokio::fs::metadata()` call reused for both cache check and write decision

- **`DataStore::all_violations()`** — merge strategy for cross-session alert aggregation:
  - DashMap (in-memory, freshest data) takes priority; SQLite fills gaps for sessions not yet analyzed in-memory
  - Deduplicates by session_id; sorts Critical → Warning → Info, then newest-first within same severity
  - Used by Violations view for zero-SQLite-during-render display (cached per-frame)

- **Concurrency model** for batch scan:
  - `Arc<Semaphore>` with 4 permits — limits concurrent session analysis, avoids I/O saturation
  - `Arc<AtomicUsize>` for live scanning count displayed in Violations stats bar (`⟳ Scanning N sessions…`)
  - `Arc<Mutex<HashSet<String>>>` for failed session tracking shared across async tasks

#### Testing

- **29 unit tests** in `parsers/activity.rs` covering `is_destructive_command`, `is_sensitive_file`, parse fixtures (3 JSONL scenarios: simple, destructive, credential), classify fan-out, alert generation (6 rules), duration computation
- **C1** `test_action_hint_all_variants_non_empty` — verifies exhaustive hint coverage for all `AlertCategory` variants (in `models/activity.rs`)
- **C3** `test_all_violations_dashmap_priority_over_sqlite` — verifies DashMap wins over SQLite for the same session_id; SQLite-only sessions still appear to fill gaps (in `store.rs`)

### Added — Search Tab

- **Search tab** (TUI index 11, accessible via Tab/Shift+Tab) — FTS5 full-text search across all sessions
  - `crates/ccboard-tui/src/tabs/search.rs` — TUI search interface with ranked results and highlighted snippets
  - `crates/ccboard-web/src/pages/search.rs` — Web search page, identical UX
  - `/api/search?q=<query>&limit=N` — backend endpoint backed by SQLite FTS5 index

### Fixed

- Pricing alias `claude-opus-4-5` and `claude-opus-4-6` (without date suffix) missing from embedded pricing table — fell back to default average (3.2) instead of correct price (5.0). Added short-form aliases alongside full versioned keys.
- Axum route syntax: `/api/activity/:session_id` → `/api/activity/{session_id}` (Axum v0.7+ capture group syntax).

---

## [0.10.0] - 2026-02-18

### Added — Phase J: Export Features

- **`ccboard export` subcommands** — bulk data export CLI with 4 subcommands:
  - `export conversation <id> --output <file> --format markdown|json|html` — single session (previously `ccboard export`)
  - `export sessions --output <file> --format csv|json|md [--since 7d]` — sessions list with optional date filter
  - `export stats --output <file> --format csv|json|md` — usage statistics (per-model breakdown + daily activity)
  - `export billing --output <file> --format csv|json|md` — billing blocks (5h windows)
- **Stats export** — 3 new functions in `ccboard-core::export`:
  - `export_stats_to_csv(&StatsCache, path)` — per-model table (input/output/cache/cost)
  - `export_stats_to_json(&StatsCache, path)` — full StatsCache as JSON
  - `export_stats_to_markdown(&StatsCache, path)` — human-readable report with summary + model table + 30-day daily activity
- **Sessions Markdown export** — `export_sessions_to_markdown(sessions, path)` — sessions as Markdown table
- **Billing JSON/Markdown export** — `export_billing_blocks_to_json(&BillingBlockManager, path)` and `export_billing_blocks_to_markdown(&BillingBlockManager, path)`

## [0.9.0] - 2026-02-18

### Added
- **Light mode**: full light theme activated via `Ctrl+T` — 11 tabs + 5 components migrated to a centralized `Palette` system
- **Theme persistence**: selected theme is saved to `~/.claude/cache/ccboard-preferences.json` and restored on startup
- **`Palette` struct** in `theme.rs`: semantic color bundle (`fg`, `bg`, `muted`, `border`, `focus`, `success`, `error`, `warning`, `important`) adapted to the active `ColorScheme`

### Fixed
- Frame background reset on every render (`Clear` + `Block` with `bg(p.bg)`) — without this fix, light mode rendered text invisible (black on black background)

## [0.8.0] - 2026-02-16

### Added - Budget Tracking & Quota Management

#### Core Quota System
- **Month-to-date (MTD) cost calculation** with intelligent token-based prorata
  - Uses `daily_model_tokens` to filter current month activity
  - Prorates total cost based on token proportion (MTD tokens / total tokens)
  - No pricing lookup needed - simple ratio-based calculation
  - Graceful handling of missing daily data
- **Monthly projection** with simple daily average
  - Projects month-end cost: `(MTD cost / current_day) * 30`
  - Calculates projected overage if budget limit set
- **Four-level alert system** with configurable thresholds
  - `Safe` (green): Usage < warning threshold (default 75%)
  - `Warning` (yellow): Usage ≥ warning threshold
  - `Critical` (red): Usage ≥ critical threshold (default 90%)
  - `Exceeded` (magenta): Usage ≥ 100%
- **BudgetConfig** in settings.json
  - `monthlyLimit` (optional): Budget limit in USD (no limit if omitted)
  - `warningThreshold` (default: 75.0): Warning alert trigger %
  - `criticalThreshold` (default: 90.0): Critical alert trigger %

#### TUI Integration
- **Quota gauge in Costs tab Overview**
  - Color-coded progress bar (green/yellow/red/magenta)
  - Displays: MTD cost, budget limit, usage %
  - Shows projected monthly cost and overage
  - Graceful fallback message if budget not configured
  - Position: Between total cost card and token breakdown
- **Analytics tab budget fixes**
  - Updated to use new BudgetConfig field names
  - Fixed `monthly_budget_usd` → `monthly_limit` (Option)
  - Fixed `alert_threshold_pct` → `warning_threshold`

#### Web UI Integration
- **REST API endpoint** `/api/quota`
  - Returns QuotaStatus JSON with current cost, usage %, projection
  - Serializes alert_level as string ("safe", "warning", "critical", "exceeded")
  - Error response if budget not configured or stats unavailable
- **Quota gauge in Costs page Overview**
  - Leptos component with Suspense for async loading
  - CSS progress bar with color-coded fill
  - Displays: MTD cost, budget, usage %, projected cost, overage
  - Graceful error handling and fallback states
  - Real-time updates via SSE (inherited from stats)

#### DataStore Integration
- **quota_status()** method in DataStore
  - Returns `Option<QuotaStatus>` (None if stats/budget unavailable)
  - Follows existing pattern: `stats()`, `settings()`, etc.
  - Zero-overhead: clones are cheap, locks released immediately
  - Thread-safe via parking_lot::RwLock

#### Testing
- **4 quota module tests** covering:
  - Safe, Warning, Critical, Exceeded alert levels
  - MTD calculation with token-based prorata
  - Monthly projection accuracy
  - No-budget scenario (returns Safe with 0% usage)
- **Token-ratio mocking** for predictable test data
  - `mock_stats_with_mtd_ratio(total_cost, ratio, first_date)`
  - Allows testing different MTD scenarios

### Technical Details
- **MVP approach chosen**: Token-based prorata vs precise daily cost aggregation
  - Simpler implementation, no pricing lookup needed
  - Accurate enough for budget alerts (±5% error acceptable)
  - Future optimization: Use pricing module for exact daily costs
- **Core exports** in ccboard-core/lib.rs:
  - `calculate_quota_status`, `AlertLevel`, `QuotaStatus`
- **Zero breaking changes**: Existing budget fields preserved (backward compatible)

## [0.7.0] - 2026-02-13

### Added - Conversation Viewer Enhancements

#### Full-Text Search
- **Interactive search in conversation viewer** with real-time highlighting
  - Press `/` to activate search mode (cyan search bar appears)
  - Type query to search across all messages in current conversation
  - Case-insensitive matching for better usability
  - Real-time results with yellow background highlights
  - Results counter: "Search (X results)" or "Search (no results)"
  - Navigate between matches:
    - `n` → Jump to next match (wraps to start)
    - `N` (shift+n) → Jump to previous match (wraps to end)
  - Press `Enter` to exit search input but keep highlights visible
  - Press `Esc` to clear search and remove highlights
  - Auto-scroll to current match position
  - Performance: <1ms search for 1000+ messages (in-memory, zero I/O)
  - Implementation: `SearchState` in `conversation.rs` with lazy evaluation

#### Dynamic Message Rendering
- **Adaptive message height calculation** (2-20 lines per message)
  - Prevents overflow panics on large messages
  - Intelligent line wrapping for long content
  - Smooth scrolling with dynamic viewport adjustment
  - Graceful handling of edge cases (empty messages, single-line)

### Fixed

#### Critical TUI Stability Fixes
- **Runtime panic in conversation/replay viewers**
  - Root cause: Nested tokio runtime when opening viewers from TUI
  - Error: "Cannot start runtime from within runtime"
  - Solution: Replaced `Runtime::new()` with `tokio::task::block_in_place()`
  - Affected: Conversation viewer ('c' key), Replay viewer ('v' key)
  - Impact: Viewers now open reliably without crashing TUI

- **Overflow panic in message rendering**
  - Root cause: Fixed-height assumption (10 lines) for variable message lengths
  - Error: Integer overflow when calculating render dimensions
  - Solution: Dynamic height calculation with 2-20 line range
  - Impact: Handles messages of any length without panic

- **Esc key not closing viewers**
  - Root cause: Key event routing priority conflict
  - Viewers remained open when pressing Esc
  - Solution: Proper event handling order (close action > search clear)
  - Impact: Esc now consistently closes conversation/replay viewers

### Changed

- **Code quality improvements**
  - Zero compiler warnings (cargo build clean)
  - Zero clippy warnings (cargo clippy clean)
  - Removed unnecessary parentheses in analytics/sessions pages
  - Added `#[allow(dead_code)]` annotations for intentionally unused code
  - Improved code organization in conversation viewer module

### Performance

- **Search optimization**: In-memory string matching with early breaks
  - O(n*m) algorithm with lazy evaluation
  - <1ms for typical conversation (1000 messages)
  - Zero I/O overhead (no disk reads during search)
  - Efficient highlight rendering (only visible portion)

### Developer Experience

- **Testing checklist**: 17 manual test cases documented in `claudedocs/STATUS.md`
  - Runtime fixes (2 tests)
  - Esc key fixes (2 tests)
  - Full-text search (9 tests covering activation, navigation, edge cases)
- **Documentation**: Complete implementation details in `claudedocs/archive/v0.7/`

## [0.6.5] - 2026-02-12

### Added
- **LiteLLM dynamic pricing integration**: Automatic pricing updates from LiteLLM canonical source
  - New CLI commands: `ccboard pricing update` (fetch latest prices) and `ccboard pricing clear` (clear cache)
  - Local caching at `~/.cache/ccboard/pricing.json` with 7-day TTL
  - Fetches 25 Claude models from https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json
  - Embedded pricing as fallback for offline usage
  - Dynamic pricing loaded at startup and merged with embedded prices
  - Benefits: Always up-to-date pricing without code changes, offline support, reduced network calls

### Changed
- Pricing module refactored into modular architecture: `pricing/litellm.rs` (fetch), `pricing/cache.rs` (storage), `pricing/embedded.rs` (fallback), `pricing/mod.rs` (unified API)
- `calculate_cost()` now uses dynamic pricing from cache → embedded fallback
- Added `reqwest` dependency for HTTP fetch from LiteLLM

## [0.6.4] - 2026-02-12

### Fixed
- **CLI panic on emoji truncation**: Fixed `ccboard search` and `ccboard recent` panicking when truncating strings containing emojis or multi-byte Unicode characters
  - Root cause: Byte-based string slicing `&s[..max-1]` panicked when max-1 fell inside a multi-byte character (emojis = 4 bytes)
  - Solution: Character-based truncation using `.chars().take(max-1).collect()` for safe UTF-8 handling
  - Added comprehensive test suite for ASCII, emojis (🔍🚀💡), and Unicode (café, 日本語)

## [0.6.3] - 2026-02-12

### Fixed
- **Web mode startup performance**: Fixed indefinite blocking on startup with large ~/.claude directories (1000+ sessions)
  - Optimized FileWatcher to watch selectively instead of recursively (99% reduction: 26k → ~200 files watched)
  - Moved analytics computation (invocations, billing blocks) to background tasks for instant startup
  - Startup time reduced from indefinite blocking to < 1 second
  - Memory usage reduced by 99% (FileWatcher no longer tracks 26k+ files)
  - Tested with 26,175 files in ~/.claude and 1,825 sessions

### Migration Notes

#### Cache Upgrades (Automatic)

ccboard's SQLite metadata cache auto-upgrades between versions. No manual action needed — on first startup after upgrade, stale cache entries are cleared and repopulated.

| From | To | Change | Action |
|------|-----|--------|--------|
| v1 | v2 | Fixed `TokenUsage::total()` calculation | Auto-clear |
| v2 | v3 | Added token breakdown fields | Auto-clear |
| v3 | v4 | Added `branch` field to SessionMetadata | Auto-clear |

If you experience issues after upgrade, clear the cache manually:
```bash
ccboard clear-cache
```

#### Keybinding Changes (v0.3.0)

- **Costs tab navigation**: Changed from `1-3` keys to `Tab`/`←→`/`h/l` to avoid conflict with main tab `1-9` navigation

## [0.5.2] - 2026-02-10

### Fixed
- **Web frontend API communication**: Fixed URL configuration for proper frontend-backend communication
  - Changed hardcoded `http://localhost:8080` URLs to relative paths (`""`) in 7 frontend modules
  - Fixed Leptos CSR feature configuration (was incorrectly using `hydrate` instead of `csr`)
  - Server now binds to `0.0.0.0` allowing access via both `localhost:3333` and `127.0.0.1:3333`
  - Updated display messages to show `http://localhost:3333` for better user experience
  - Resolves issue where web interface would load initially but hang on "Loading..." when navigating between pages

## [0.5.1] - 2026-02-10

### Added
- **Homebrew formula**: ccboard now available via `brew tap FlorianBruniaux/tap && brew install ccboard`
  - macOS/Linux support
  - Automatic updates via `brew upgrade`
  - Added to [FlorianBruniaux/homebrew-tap](https://github.com/FlorianBruniaux/homebrew-tap)

### Documentation
- **Installation guide**: Prioritize Homebrew (simplest), then cargo install, then pre-built binaries
- **Troubleshooting section**: Common issues + solutions (stats not loading, WASM compilation, port conflicts, Linux file manager, Windows terminal rendering)
- **Platform support transparency**: Clear tier system with emoji indicators (✅ macOS tested, ⚠️ Linux CI-tested, 🧪 Windows experimental)

### Internal
- No code changes (documentation + distribution-only release)

## [0.5.0] - 2026-02-09

### Added - Sprint 1 UX/UI Improvements

#### Visual Design System (60% visual improvement achieved)
- **Elevation system**: 4-level shadows with glow effects
  - `--elevation-1` through `--elevation-4` tokens
  - `--glow-cyan` and `--glow-blue` for accent elements
  - Depth perception and visual hierarchy
- **Hero typography**: Extended font scale for impactful KPI numbers
  - `--text-4xl` (48px) for dashboard statistics
  - `--font-extrabold` (800) for emphasis
  - Gradient text effects on primary numbers
- **Improved contrast**: +16 luminosity boost for better readability
  - `--text-primary`: #f0f0f0 (was #e0e0e0)
  - `--text-secondary`: #b0b0b0 (was #a0a0a0)
  - `--text-muted`: #707070 (was #606060)
  - Better WCAG 2.1 compliance
- **Table spacing**: Breathing room for better scannability
  - Padding: 8px → 16px per cell
  - Zebra striping for row distinction
  - Hover states with cyan accent border
- **Border radius system**: Semantic radius values
  - `--radius-button`: 6px
  - `--radius-card`: 10px
  - `--radius-modal`: 12px
  - Consistent modern look

#### Config Page Enhancements
- **Real-time search**: Filter JSON config with live highlighting
  - Highlights matches with yellow background
  - Case-insensitive search
  - Search results counter
- **Copy buttons**: One-click JSON copy to clipboard
  - Per-column copy functionality
  - Visual feedback on hover
- **Fullscreen modal**: View config without horizontal scrolling
  - Glassmorphism backdrop with blur effect
  - Expand button (📖) next to copy button
  - Scrollable JSON content
  - Click outside to close
- **Diff mode toggle**: Show only overridden settings (planned)

#### Dashboard Interactive Features
- **Clickable KPI cards**: Navigate directly from metrics
  - Total Sessions → Sessions page
  - Total Messages → Sessions page filtered
  - Click hint: "Click to explore →"
- **Session preview tooltips**: Hover to see session details
  - Project path
  - Token stats (input/output)
  - First message snippet
  - Instant preview without modal
  - Glassmorphism styling

#### Design Tokens & CSS Architecture
- **Gradients**: Vertical gradients for depth
  - `--gradient-cyan`, `--gradient-blue`, `--gradient-purple`
  - Chart area fill gradients
  - Surface gradients for cards
- **Opacity scale**: Systematic transparency values
  - `--opacity-5` through `--opacity-80`
  - Consistent layering system
- **400+ lines of new CSS**: Comprehensive design system
  - Component-specific styles
  - Responsive patterns
  - Animation utilities

### Fixed

- **Cost calculation**: Recalculate costs after loading stats
  - Cost analysis was showing $0.00 for all models
  - Added `recalculate_costs()` method to `StatsCache`
  - Applies accurate Anthropic pricing (Opus $15/M, Sonnet $3/M, Haiku $1/M)
  - Calculates cache read/write costs correctly
- **Model display**: Fixed "model unknown" in history page
  - Line 418 in `router.rs` had incorrect String reference
  - Changed `.unwrap_or(&"unknown".to_string())` to `.map(|s| s.as_str()).unwrap_or("unknown")`
  - Proper Option<&String> → &str conversion

### Changed

- **Stats card component**: Added `on_click` handler support
  - Optional click handler parameter
  - Hover states with transform
  - Action hint display
- **Dashboard layout**: Enhanced with navigation callbacks
  - Uses `leptos_router::hooks::use_navigate`
  - Cloned navigate for each closure to avoid move issues

### Technical

- **Files modified**: 7 files, 714 additions, 49 deletions
  - `crates/ccboard-core/src/models/stats.rs`: Add recalculate_costs()
  - `crates/ccboard-core/src/store.rs`: Call cost recalculation
  - `crates/ccboard-web/src/components/stats_card.rs`: Add on_click
  - `crates/ccboard-web/src/pages/config.rs`: Search + modal
  - `crates/ccboard-web/src/pages/dashboard.rs`: Navigation + tooltips
  - `crates/ccboard-web/src/router.rs`: Fix model field bug
  - `crates/ccboard-web/static/style.css`: Design system expansion
- **Merge commit**: feat/web-w1-leptos-spa → main (15,026 additions total)

## [0.4.0] - 2026-02-06

### Added - Quick Wins from Usage Report (QW5-7)

#### Pre-commit Hook (QW5)
- **Cargo check + clippy enforcement**: `.claude/settings.json` hook
  - Automatically runs `cargo check` and `cargo clippy` before every commit
  - Prevents 48% of frictions (buggy code from usage analysis)
  - Tail output (last 20 lines) to avoid overwhelming error messages
  - Applies to all `**/*.rs` files in the workspace

#### /ship Skill (QW6)
- **Automated release workflow**: `.claude/skills/ship/SKILL.md`
  - Build → Test → Commit → Version bump → Push in single command
  - Workflow: `cargo build && clippy && test` → `git add` → conventional commit → push
  - Optional version bump with `bump` argument
  - Dry-run mode with `--dry-run` flag
  - Co-authored commits with Claude attribution

#### CLAUDE.md Enhancements (QW7)
- **Build Verification**: Mandatory `fmt && clippy && test` before commit
  - Addresses #1 friction (buggy code 48% of issues)
  - Zero tolerance policy for clippy warnings
- **Testing Policy**: Manual CLI/TUI testing required
  - Don't rely solely on automated tests
  - Describe what you see when testing UI changes
- **Working Directory Confirmation**: `pwd` + `git branch` before work
  - Prevents wrong directory friction (26% of issues)
  - Never assume which project to work in
- **Avoiding Rabbit Holes**: Max 3-4 exploratory commands before asking
  - Stay focused on task
  - Stop and ask if verification exceeds threshold
- **Plan Execution Protocol**: Sequential execution, commit per step
  - Pattern produces 47% fully-achieved outcomes (vs 12% without)
  - Use TodoWrite for 3+ step plans
- **Language & Communication**: French by default
  - "reprend" = resume previous task
  - Bold Guy style (direct, factual)
- **Graceful Degradation**: Parser error handling
  - Skip malformed entries, continue loading
  - Populate LoadReport instead of panicking

#### Documentation Cleanup
- **Archive obsolete docs**: 28 files moved to organized structure
  - sessions-history/ (4 files)
  - tasks-completed/ (7 files)
  - phase-plans/ (4 files)
  - competitive-old/ (2 files)
  - reference/ (5 files)
- **Active docs reduced**: 36 → 8 focused files
  - PLAN.md, README.md, ACTION_PLAN.md
  - xlaude learnings (3 files, 67KB)
  - performance-benchmark.md, competitive-benchmark.md
- **STATUS.md created**: Single source of truth
  - Production-ready status (v0.4.0)
  - Complete feature matrix (94 features)
  - Web backend/frontend status clarified
  - Known limitations documented

#### Web Status Clarification
- **Backend API**: 100% complete (Axum + SSE + 4 routes)
  - /api/stats, /api/sessions, /api/health
  - SSE streaming at /api/events
  - CORS support, dual TUI+Web mode
- **Frontend UI**: 0% complete (placeholder only)
  - Current: "ccboard web UI - Coming soon"
  - Planned: Phase IV (8-12h Leptos implementation)
  - README corrected to reflect reality

### Added - Phase I-CLI: Session Management Commands

#### Message Filtering (QW2 - xlaude insights)
- **System message filtering**: Filter out protocol/system messages for cleaner previews
  - Excludes: `<local-command`, `<command-`, `<system-reminder>`, `Caveat:`
  - Excludes: `[Request interrupted`, `[Session resumed`, `[Tool output truncated`
  - Applied to `first_user_message` preview extraction in session metadata
- **New module**: `ccboard-core/src/parsers/filters.rs`
  - `is_meaningful_user_message()`: Reusable filter for UI and analytics
  - Comprehensive test coverage (5 test cases)
- **Session parser integration**: Cleaner session previews in TUI and search results
  - Test: `test_message_filtering_excludes_system_messages` validates behavior
  - System commands no longer pollute session previews

#### Environment Variables (QW1 - xlaude insights)
- **`CCBOARD_CLAUDE_HOME`**: Override Claude home directory path
  - Enables testing with isolated configurations
  - Example: `CCBOARD_CLAUDE_HOME=/tmp/test-claude ccboard stats`
- **`CCBOARD_NON_INTERACTIVE`**: Disable interactive prompts for CI/CD
  - Fail fast instead of waiting for user input
  - Example: `CCBOARD_NON_INTERACTIVE=1 ccboard stats`
- **`CCBOARD_FORMAT`**: Force output format (`json` or `table`)
  - Useful for scripting and automation pipelines
  - Example: `CCBOARD_FORMAT=json ccboard recent 10 | jq`
- **`CCBOARD_NO_COLOR`**: Disable ANSI colors for log-friendly output
  - Clean output for file redirects and log aggregators
  - Example: `CCBOARD_NO_COLOR=1 ccboard search "error" > results.log`
- **clap env feature**: Upgraded `clap` with `env` feature for environment variable support
- **Test suite**: Added `tests/env_vars_test.sh` for validation

#### CLI Commands
- **`search` command**: Search sessions by query string (ID, project, message, branch)
  - `--since` filter: `7d`, `30d`, `3m`, `1y`, or `YYYY-MM-DD` format
  - `--limit` option: max results (default: 20)
  - `--json` flag: output as JSON for scripting
  - Example: `ccboard search "implement auth" --since 7d --limit 5`
- **`recent` command**: Show N most recent sessions
  - `--since` date filter support
  - `--json` flag for structured output
  - Example: `ccboard recent 10 --json`
- **`info` command**: Display detailed session metadata
  - Supports full ID or 8+ char prefix matching
  - Shows 17 fields: tokens breakdown, models, duration, branch, etc.
  - `--json` flag for structured output
  - Example: `ccboard info abc123de`
- **`resume` command**: Resume session in Claude CLI
  - Prefix matching (min 8 chars) for convenience
  - Unix: `exec()` replacement for seamless transition
  - Windows: spawn + exit with same status code
  - Example: `ccboard resume 6c93a53e`

#### Core Enhancements
- **Branch extraction**: Added `branch` field to `SessionMetadata`
  - Normalizes git branch names: strips `worktrees/` prefix, `(dirty)` suffix
  - Handles detached HEAD: `HEAD (detached at abc123)` → `HEAD`
  - Extracted from first `gitBranch` in session JSONL
  - Cache version bumped to v4 (auto-invalidation)
- **CLI module** (`crates/ccboard/src/cli.rs`):
  - `DateFilter` parser: `7d`, `3m`, `1y`, `YYYY-MM-DD` formats
  - `CliError` enum: `NoResults`, `AmbiguousId` with helpful messages
  - `find_by_id_or_prefix()`: exact or prefix matching with collision detection
  - `search_sessions()`: multi-field text search with date filtering
  - `format_session_table()`: comfy-table human output or JSON
  - `format_session_info()`: 17-field detailed view (human or JSON)
  - Utilities: `format_tokens()`, `truncate()`, `shorten_project()`

#### Developer Experience
- **Reuses DataStore**: No new `CliStore` - same one-shot pattern as `run_stats()`
- **Zero overhead**: Moka cache not allocated, EventBus has 0 subscribers in CLI mode
- **SQLite cache**: Automatic 200ms warm vs 5s cold startup
- **Unit tests**: 14 tests for DateFilter parsing, prefix matching, formatters
- **Updated CLAUDE.md**: Added CLI examples to "Build & Run" section

## [0.3.0] - 2026-02-05

### Added - Phase H+: UX & Analytics Enhancements

#### UX Improvements

- **Badge Style Keyboard Hints**
  - Modern badge format `[key]` for keyboard shortcuts
  - WCAG AAA compliant contrast (10:1 for keys, 21:1 for descriptions)
  - Black on Cyan background for keys (vs previous Cyan text on DarkGray)
  - White descriptions on DarkGray background (vs previous DarkGray text)
  - Applied to Sessions tab keyboard hints (3 focus states: Live Sessions, Projects, Sessions)
  - Improved scannability (+90%) with visual hierarchy
  - Consistent with modern CLI conventions (GitHub CLI, VS Code, Lazygit)

#### Analytics Enhancements

- **Activity Heatmap**
  - GitHub-style 7 days × 24 hours heatmap in Analytics > Patterns view
  - Color-coded intensity scale: DarkGray → Green → Cyan → Yellow → Magenta
  - Shows session activity patterns by day of week and hour
  - Weekday labels (Mon-Sun) with hour markers (00, 04, 08, 12, 16, 20)
  - Visual legend for intensity levels
  - Helps identify peak productivity hours and work patterns

- **Most Used Tools**
  - Horizontal bar chart showing top 6 most-used tools/models
  - Color-coded bars (Blue, Green, Cyan, Magenta, Yellow, Red)
  - Displays usage count and percentage
  - Bar length proportional to usage (max 40 chars)
  - Located in Analytics > Patterns view
  - MVP implementation: Uses model names as proxy (can be enhanced with real tool_calls parsing)

### Added - Phase 3: UI/UX Quick Wins (Performance & User Experience)

#### Performance Optimization (Phase 0-2)
- **Profiling & Baseline** (Phase 0)
  - Criterion benchmarks for startup performance (`benches/startup_bench.rs`)
  - Performance regression tests with strict time targets (<2s for warm cache)
  - Baseline measured: 20.08s for 3550 sessions
  - Bottleneck identified: JSONL parsing + I/O disk (90% of total time)

- **Security Hardening** (Phase 1)
  - Path traversal protection: `sanitize_project_path()` strips `..` components
  - Symlink rejection in project paths
  - OOM protection: 10MB line size limit for JSONL files
  - Credential masking: `Settings::masked_api_key()` format `sk-ant-••••cdef`
  - Security test suite: 8 tests covering path validation, size limits, masking

- **SQLite Metadata Cache** (Phase 2.1) - **89x Speedup**
  - Cold cache: 20.08s → Warm cache: 0.224s (**89.67x faster**)
  - SQLite with WAL mode for concurrent reads
  - mtime-based cache invalidation
  - bincode serialization for compact storage
  - Background cache population during initial load
  - Cache location: `~/.claude/cache/session-metadata.db`

#### UI/UX Improvements (Phase 3)
- **Loading Spinner** (Task 3.1)
  - Animated Braille dot spinner during startup
  - 4 styles available: Dots, Line, Bounce, Circle
  - 80ms frame rate for smooth animation
  - TUI starts immediately (<10ms) instead of 20s blocking wait
  - Background loading with `tokio::spawn`
  - Can quit with `q` during loading
  - Component: `components/spinner.rs` (+143 LOC)

- **Help Modal** (Task 3.2)
  - Toggle with `?` key (press again to close)
  - Close with `ESC` key
  - Context-aware keybindings per tab
  - Global keybindings section (q, ?, :, F5, Tab, 1-8)
  - Tab-specific shortcuts (Sessions, Config, Hooks, Agents, Costs, History, MCP)
  - Centered overlay (70x24) with cyan borders
  - Component: `components/help_modal.rs` (+293 LOC)

- **Search Highlighting** (Task 3.3)
  - Yellow background highlighting for search matches
  - Case-insensitive matching
  - Works in Sessions tab (preview text)
  - Works in History tab (list + detail popup)
  - Helper function: `highlight_matches()` (+90 LOC, 5 unit tests)
  - Highlights all occurrences in text
  - Black bold text on yellow background for contrast

#### Export & Analysis Features (Phase C)

- **MCP Tab Enhanced Detail Pane** (Task C.1)
  - **Args syntax highlighting**: Color-coded display for better readability
    - Flags (`--flag`, `-f`) → Cyan bold
    - Paths (`/absolute`, `./relative`) → Green
    - URLs (`http://`, `https://`) → Magenta
    - Regular values → White
  - **Environment variables masking**: Auto-detect and mask sensitive values
    - Detects: API_KEY, TOKEN, SECRET, PASSWORD, API patterns
    - Masked format: `abcd••••efgh` (first 4 + last 4 chars)
    - Displayed in gray for masked values
    - Alphabetically sorted for consistency
  - **Server descriptions**: Inline documentation for known MCP servers
    - Auto-detected servers: playwright, serena, sequential, context7, perplexity, claude-in-chrome, filesystem
    - Displayed as italic gray text at top of detail pane
  - **Copy to clipboard**: Press `y` to copy full command
    - Copies: `command arg1 arg2 ...`
    - Success notification with green bottom banner
    - Cross-platform via `arboard` crate
    - ESC to dismiss notification
  - Helper functions: `highlight_arg()`, `mask_sensitive_env()`, `get_server_description()`
  - Module: `tabs/mcp.rs` (+140 LOC)
  - Help modal updated with `y` keybinding
  - New dependency: `arboard = "3"` for clipboard support

- **Billing Blocks CSV Export** (Task C.3)
  - Export billing blocks to CSV format for external analysis
  - Function: `export_billing_blocks_to_csv(&manager, &path)`
  - CSV columns: Date, Block (UTC), Tokens, Sessions, Cost
  - Sorted by date/time (most recent first)
  - Cost formatted with 3 decimal places ($X.XXX)
  - Auto-creates parent directories
  - BufWriter for efficient I/O on large datasets
  - Example: `cargo run --example export_billing_blocks`
  - Module: `ccboard-core/src/export.rs` (+175 LOC)
  - Compatible with Excel, Google Sheets, LibreOffice
  - Tested with 3638 sessions → 104 billing blocks exported

- **History Tab Export CSV/JSON** (Task C.2)
  - Export filtered session results to CSV or JSON format
  - Key binding: `x` in History tab → Format selection dialog
  - CSV export: `export_sessions_to_csv(&sessions, &path)`
    - Columns: Date, Time, Project, Session ID, Messages, Tokens, Models, Duration (min)
    - Duration calculated from first/last timestamps
    - Models joined with `;` separator
  - JSON export: `export_sessions_to_json(&sessions, &path)`
    - Pretty-printed JSON array of SessionMetadata
    - Full metadata serialization (id, timestamps, tokens, models, etc.)
  - Interactive dialog: `1` for CSV, `2` for JSON, `ESC` to cancel
  - Success/error messages with color coding (green/red)
  - Export location: `~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.{csv,json}`
  - Timestamp in filename prevents overwrite
  - Module: `ccboard-core/src/export.rs` (+135 LOC, 2 functions + 5 tests)
  - TUI integration: `tabs/history.rs` (+95 LOC, export dialog + UI)
  - Help modal updated with `x` keybinding documentation
  - Tested: 5 unit tests (CSV empty/data, JSON empty/data, directory creation)
  - Compatible with Excel, Google Sheets, jq, data analysis tools

- **Sessions Tab Live Refresh** (Task C.4)
  - Real-time session update indicators with visual feedback
  - Timestamp tracking with `Instant::now()` for precise elapsed time
  - Added fields to SessionsTab:
    - `last_refresh: Instant` - tracks when data was last refreshed
    - `refresh_message: Option<String>` - notification message
    - `prev_session_count: usize` - detects session count changes
  - Implemented `mark_refreshed()` to track and notify on data updates
    - Detects session count changes: "+3 new", "-2 removed", "refreshed"
    - Shows green notification banner on changes
  - Implemented `format_time_ago()` for human-readable elapsed time
    - Formats: "just now", "5s ago", "2m ago", "1h ago"
  - Implemented `render_refresh_notification()` for bottom banner overlay
    - 60% width, 3 lines height, centered at bottom
    - Green borders and text for success feedback
    - Auto-clears after one render cycle
  - Modified `render_sessions()` title to show timestamp: "Sessions (15) • 2m ago"
  - Call `mark_refreshed()` in ui.rs when Sessions tab renders with fresh data
  - Integrates with FileWatcher EventBus for real-time updates
    - DataEvent::SessionCreated, SessionUpdated, LoadCompleted trigger refresh
  - Module: `tabs/sessions.rs` (+88 LOC)
  - UI integration: `ui.rs` (+3 LOC to call mark_refreshed)
  - All 152 tests passing, zero clippy warnings
  - Test guide: `TEST_GUIDE_PHASE_C4.md` (comprehensive manual testing guide)
  - Test script: `scripts/test_phase_c4.sh` (automated validation script)

#### Arc Migration for Memory Optimization (Phase D)

- **Arc<SessionMetadata> Migration** (Phase D.1-D.3) - **50x Memory Reduction**
  - Replaced `SessionMetadata` clones with `Arc<SessionMetadata>` for massive memory savings
  - Memory per clone: 400 bytes → 8 bytes (**50x reduction**)
  - Clone speed: ~1000ns → ~1ns (**1000x faster**)
  - Heap allocations: Eliminated (100% reduction)
  - Cache pressure: ~50x reduction (smaller working set)
  - **DataStore changes** (`store.rs` +20 LOC):
    - `DashMap<String, Arc<SessionMetadata>>` instead of plain SessionMetadata
    - `get_session()` returns `Option<Arc<SessionMetadata>>`
    - `sessions_by_project()` returns `HashMap<String, Vec<Arc<SessionMetadata>>>`
    - `recent_sessions()` returns `Vec<Arc<SessionMetadata>>`
    - Arc::new() wraps sessions on insertion
    - Arc::clone() in all iterations (cheap: 8 bytes)
  - **Export functions** (`export.rs` +7 LOC):
    - `export_sessions_to_csv(&[Arc<SessionMetadata>], ...)`
    - `export_sessions_to_json(&[Arc<SessionMetadata>], ...)`
    - JSON export: Dereference Arc with `.as_ref()` (Arc doesn't impl Serialize)
    - Tests updated: Arc::new() wrappers in all test fixtures
  - **Sessions Tab** (`sessions.rs` +15 LOC):
    - All methods accept `&HashMap<String, Vec<Arc<SessionMetadata>>>`
    - Filter operations use `Arc::clone()` instead of `.cloned()`
    - Transparent field access via Deref trait
  - **History Tab** (`history.rs` +12 LOC):
    - `filtered_sessions: Vec<Arc<SessionMetadata>>`
    - All methods accept `&[Arc<SessionMetadata>]`
    - `update_filter()` uses Arc::clone in iterations
  - **Benefits**:
    - SessionMetadata cloned only once (at insertion)
    - All subsequent clones are Arc clones (8 bytes pointer copy)
    - No lifetime complexity (Arc = owned type)
    - Thread-safe shared ownership (Arc is Send + Sync)
  - **Tests**: 131 lib tests passing, 0 clippy warnings
  - **Validation**: `TEST_ARC_MIGRATION.md` (comprehensive validation guide)
  - **Duration**: 3.5h (vs 4h estimated - 12.5% faster)

### Changed
- **Startup Flow**: TUI now starts immediately with loading screen instead of blocking
- **Main Binary**: Removed blocking `initial_load()` before TUI start
- **Background Loading**: Initial load runs in tokio task, signals completion via oneshot channel

### Performance
- **Startup Time**: 20.08s → 0.224s warm cache (**89.67x improvement**)
- **Cold Start**: 20s with animated spinner (user feedback)
- **Cache Hit Rate**: >95% after first run
- **Memory (Phase D)**: Arc<SessionMetadata> reduces clone cost by **50x** (400 bytes → 8 bytes)
- **Clone Speed (Phase D)**: **1000x faster** cloning (~1ns vs ~1000ns)
- **Heap Allocations (Phase D)**: Eliminated for session clones (100% reduction)

### Tests
- Phase 0: Performance regression tests (6 tests)
- Phase 1: Security tests (8 tests)
- Phase 2.1: Cache integration tests
- Phase 3: Component tests (10 tests total)
  - Spinner: 3 tests (cycling, styles, custom color)
  - Help Modal: 2 tests (toggle, hide)
  - Search Highlighting: 5 tests (empty query, single/multiple matches, case-insensitive, no match)
- Phase C: Export tests (10 tests total)
  - Billing Blocks CSV: 5 tests (empty manager, with data, parent dir creation, cost formatting, multi-date sorting)
  - Sessions Export: 5 tests (CSV empty/data, JSON empty/data, directory creation)

### Dependencies
- Added `criterion = "0.5"` for benchmarking
- Added `rusqlite = "0.32"` with bundled feature
- Added `bincode = "1"` for cache serialization

### Added - Phase 11: Token Tracking & Invocation Counters

#### Token Tracking
- **Real token extraction** from session JSONL files
  - Fixed `TokenUsage` field mapping (snake_case, cache field aliases)
  - Added `usage` field to `SessionMessage` for proper deserialization
  - Parser checks both `root.usage` and `message.usage` for compatibility
  - Sessions tab now displays actual token counts instead of 0
  - Accumulates input_tokens, output_tokens, cache_read_input_tokens, cache_creation_input_tokens

#### Invocation Statistics
- **Agent/Command/Skill usage tracking** across all sessions
  - New `InvocationStats` model with aggregation support
  - `InvocationParser` with async session scanning and regex-based detection
  - Agents counted by `subagent_type` (e.g., "technical-writer", "debugger")
  - Commands counted by `/name` (e.g., "/commit", "/help")
  - Skills counted by name (e.g., "pdf-generator", "tdd-rust")
  - DataStore integration with `compute_invocations()` method

- **Agents tab visual enhancements**
  - Invocation counts displayed as `(× N)` badges in yellow
  - Automatic sorting by usage (descending) with name as tie-breaker
  - Most-used agents/commands/skills appear first
  - Updated during initial load and shown immediately

### Changed
- **Costs tab keybindings**: Changed from `1-3` to `Tab/←→/h/l` to avoid conflict with main tab navigation
- **Session detail panel**: Added text wrapping for long paths and messages

### Fixed
- **Token display bug**: Sessions now show real token counts from JSONL `message.usage`
- **Field mapping**: TokenUsage correctly deserializes cache_read_input_tokens and cache_creation_input_tokens
- Clippy warnings in editor.rs (unsafe blocks for env var tests)
- Removed unused tempfile dependency from tests
- **Costs tab navigation**: Fixed keybinding conflict where `1-3` switched main tabs instead of Costs views

### Dependencies
- Added `regex = "1"` for command pattern detection

## [0.1.0] - 2026-02-02

### Added - Phase 6: File Opening & MCP UI Integration

#### File Editing & Opening
- **File editor integration** across all tabs with `$EDITOR` support
  - Press `e` to open files in configured editor (Agents, Sessions, History, Hooks, Config)
  - Support for `$VISUAL` > `$EDITOR` > fallback (nano/notepad.exe)
  - Terminal state preservation (alternate screen, raw mode)
  - Cross-platform support (macOS, Linux, Windows)

- **File manager reveal** functionality
  - Press `o` to reveal file in system file manager
  - macOS: Finder with selection (`open -R`)
  - Linux: Default file manager (`xdg-open`)
  - Windows: Explorer with selection (`explorer /select,`)

#### Config Tab Enhancements
- **Cascading config file editing** by column focus
  - Column 0 (Global): Edit `~/.claude/settings.json`
  - Column 1 (Project): Edit `.claude/settings.json`
  - Column 2 (Local): Edit `.claude/settings.local.json`
  - Column 3 (Merged): Read-only view

- **Enhanced MCP section** with multi-line display
  - 3 lines per server (name, command, env vars)
  - Green bullets for configured servers
  - Env count display ("2 vars", "none")
  - Command truncation at 60 chars

- **MCP detail modal** (press `m`)
  - 70% width/height centered overlay
  - Full command display (no truncation)
  - All environment variables with key=value
  - Config file path display
  - Edit config directly with `e` key
  - Auto-close on editor exit

#### Dashboard Improvements
- **MCP servers card** (5th stat card)
  - Changed layout from 4 to 5 columns (20% each)
  - Display MCP server count
  - Green color for active servers, gray for none
  - Real-time count from `claude_desktop_config.json`

#### Hooks Tab
- **File path tracking** for hook scripts
  - Display `.sh` file paths in hook details
  - Edit hook scripts with `e` key
  - Reveal hook files with `o` key
  - Auto-scan `.claude/hooks/bash/*.sh` files

#### Error Handling
- **Consistent error popup pattern** across all tabs
  - Red border with error message
  - Press `Esc` to close
  - User-friendly error messages
  - File existence validation

### Added - Phase 1-3: Core Features

#### Dashboard Tab
- Real-time stats cards (Tokens, Sessions, Messages, Cache Hit Rate)
- 7-day activity sparkline with daily message counts
- Top 5 model usage gauges with percentage breakdown
- Clean layout with color-coded metrics

#### Sessions Tab
- Dual-pane interface (33 projects | sessions list)
- Project tree navigation with expand/collapse
- Session metadata (timestamps, duration, tokens, models)
- First message preview
- Detail popup with full metadata

#### Config Tab
- 4-column cascading view (Global, Project, Local, Merged)
- Configuration visualization with inheritance
- MCP servers section
- Rules (CLAUDE.md) viewer with preview
- Plugins display
- Environment variables section

#### Hooks Tab
- Event-based hook browsing (PreToolUse, PostToolUse, etc.)
- Dual-pane layout (events | hook details)
- Hook code preview with bash scripts
- Match pattern display
- File path tracking

#### Agents Tab
- 3 sub-tabs: Agents (12) | / Commands (5) | Skills (0)
- Frontmatter extraction (name, description)
- File preview (500 chars)
- Recursive directory scanning
- Category icons (◉▶★)

#### Costs Tab
- 3 views: Overview | By Model | Daily Trend
- Token breakdown (input, output, cache read/write)
- Estimated total cost with 2024 pricing
- Model-specific cost analysis
- Per-day cost trends

#### History Tab
- Full-text search across all sessions (press `/`)
- Activity by hour histogram (24h)
- Recent activity sparkline (7 days)
- Session detail popup
- Temporal pattern visualization

### Infrastructure

#### Core Architecture
- **DataStore**: Thread-safe state with DashMap + parking_lot::RwLock
- **EventBus**: tokio broadcast for live updates
- **Graceful degradation**: LoadReport pattern for partial data
- **Metadata-only session scan**: <2s for 1000+ sessions
- **Moka cache**: LRU session content on-demand

#### Parsers
- Stats cache parser (`stats-cache.json`)
- Settings parser with 3-level cascade (global/project/local)
- Session index parser with streaming JSONL
- Frontmatter parser for agents/commands/skills (YAML + Markdown)
- MCP config parser (`claude_desktop_config.json`)
- Rules parser (`CLAUDE.md` global + project)

#### Quality
- Cargo workspace with 4 crates (ccboard, core, tui, web)
- Clippy clean (0 warnings)
- Unit tests for parsers and core logic
- Cross-platform terminal handling
- Binary size: ~2.3MB (release)

### Technical Details

#### Dependencies
- **TUI**: ratatui 0.30, crossterm 0.28
- **Async**: tokio with multi-thread runtime
- **Concurrency**: parking_lot, dashmap, moka cache
- **CLI**: clap 4 with derive macros
- **Serialization**: serde, serde_json, serde_yaml
- **File watching**: notify 7, notify-debouncer-mini
- **Error handling**: anyhow (apps), thiserror (libs)

#### Performance
- Initial load: <2s for 1000+ sessions
- Metadata-only scan: No full JSONL parse at startup
- On-demand content loading: Lazy session body fetch
- Concurrent directory scans: tokio::spawn per project
- Debounced file watcher: 500ms base, 3s burst

### Documentation
- Comprehensive PLAN.md with architecture and phases
- CLAUDE.md with project guidance
- TEST_GUIDE_PHASE6.md with manual test procedures
- Automated test script (test_phase6.sh)

---

## Version History

- **0.1.0** (2026-02-02) - Initial release with Phase 6 features
  - File opening & editing across all tabs
  - MCP UI integration (enhanced section + modal)
  - Dashboard MCP card
  - 7 functional tabs (Dashboard, Sessions, Config, Hooks, Agents, Costs, History)
  - Core parsers and data store
  - Thread-safe concurrency model

---

## Links

- **Repository**: https://github.com/FlorianBruniaux/ccboard
- **Issues**: https://github.com/FlorianBruniaux/ccboard/issues
- **Releases**: https://github.com/FlorianBruniaux/ccboard/releases

---

## Acknowledgments

This project was developed following Test-Driven Development (TDD) principles with guidance from Agent Academy. See `TDD_EVIDENCE.md` for methodology documentation.

**Co-Authored-By**: Claude Sonnet 4.5 <noreply@anthropic.com>
