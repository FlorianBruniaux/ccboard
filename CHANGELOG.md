# Changelog

All notable changes to ccboard will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive test guide (`TEST_GUIDE_PHASE6.md`) with manual testing procedures
- Automated test verification script (`test_phase6.sh`)
- Enhanced CLI help with detailed descriptions and examples
- **Invocation count field** in AgentEntry structure (prepared for future counting from sessions)
- **Config editing hints** in UI footer ("e edit │ o reveal")

### Changed
- **Costs tab keybindings**: Changed from `1-3` to `Tab/←→/h/l` to avoid conflict with main tab navigation
- **Session detail panel**: Added text wrapping for long paths and messages

### Fixed
- Clippy warnings in editor.rs (unsafe blocks for env var tests)
- Removed unused tempfile dependency from tests
- **Costs tab navigation**: Fixed keybinding conflict where `1-3` switched main tabs instead of Costs views

### Known Issues
- **Tokens display 0**: Claude Code JSONL files don't contain `usage` field. Stats-cache.json only has aggregate stats, not per-session tokens. This is a limitation of Claude Code itself, not ccboard.

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
