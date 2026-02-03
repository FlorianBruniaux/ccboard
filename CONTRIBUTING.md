# Contributing to ccboard

Thank you for your interest in contributing to ccboard! This guide will help you get started with development, testing, and submitting contributions.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Testing](#testing)
- [Pull Request Process](#pull-request-process)
- [Architecture Guidelines](#architecture-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

---

## Getting Started

### Prerequisites

- **Rust 1.85+** (uses edition 2024)
- **Git** for version control
- **Claude Code** installed with `~/.claude` directory (for testing)

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/ccboard.git
cd ccboard

# Add upstream remote
git remote add upstream https://github.com/FlorianBruniaux/ccboard.git
```

---

## Development Setup

### Build

```bash
# Build all crates
cargo build --all

# Build in release mode
cargo build --release --all
```

### Run

```bash
# Run TUI (default)
cargo run

# Run web interface
cargo run -- web --port 3333

# Run with debug logging
RUST_LOG=ccboard=debug cargo run
```

### Watch Mode (Development)

```bash
# Install cargo-watch
cargo install cargo-watch

# Auto-rebuild on changes
cargo watch -x 'run'

# Auto-rebuild web
cargo watch -x 'run -- web'
```

---

## Code Style

ccboard follows strict Rust code quality standards.

### Formatting

**REQUIRED** before every commit:

```bash
# Format all code
cargo fmt --all

# Check formatting without modifying
cargo fmt --all -- --check
```

**Rule**: All code must be formatted with `rustfmt`. CI will reject PRs with formatting issues.

### Linting

**REQUIRED** before every commit:

```bash
# Run clippy with strict warnings
cargo clippy --all-targets -- -D warnings
```

**Rules**:
- Zero clippy warnings allowed
- Fix issues, don't suppress with `#[allow(...)]` unless justified in PR description
- Common violations:
  - Unused imports
  - Nested `if let` (use collapsed pattern)
  - Missing documentation on public items
  - Unnecessary `.clone()` calls

### Error Handling Standards

ccboard follows idiomatic Rust error handling:

#### Binary Crates (ccboard, ccboard-tui, ccboard-web)

Use **anyhow** for application errors:

```rust
use anyhow::{Context, Result};

fn load_session(path: &Path) -> Result<Session> {
    let file = File::open(path)
        .context("Failed to open session file")?;  // ✅ ALWAYS add context

    // ...
}
```

**Rules**:
- Every `?` operator MUST have `.context("description")` or `.with_context(|| ...)`
- Context messages should be actionable and user-friendly
- Never use `.unwrap()` in production code (tests are OK)

#### Library Crates (ccboard-core)

Use **thiserror** for custom error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid JSONL format at line {line}")]
    InvalidFormat { line: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Rules**:
- Libraries NEVER panic (except for contract violations in debug mode)
- Always return `Result<T, E>` for fallible operations
- Use `#[from]` for automatic error conversion

#### Graceful Degradation

ccboard displays partial data if some files are corrupted:

```rust
pub struct LoadReport {
    pub stats_loaded: bool,
    pub sessions_scanned: usize,
    pub sessions_failed: usize,
    pub errors: Vec<LoadError>,
}

// ✅ Good: Collect errors, continue loading
fn load_all() -> LoadReport {
    let mut report = LoadReport::default();

    for path in paths {
        match parse_session(&path) {
            Ok(session) => { /* store */ }
            Err(e) => {
                report.errors.push(e);
                report.sessions_failed += 1;
                // Continue with other sessions
            }
        }
    }

    report
}
```

---

## Testing

### Running Tests

```bash
# Run all tests (139 tests)
cargo test --all

# Run tests for specific crate
cargo test -p ccboard-core

# Run specific test
cargo test test_name

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Test Requirements

**All PRs must include tests** for:
- New features (unit tests + integration tests where applicable)
- Bug fixes (regression test demonstrating the fix)
- Public API changes (documentation tests)

### Test Organization

```rust
// ✅ Good: Tests in same file as implementation
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_session() {
        // Arrange
        let input = r#"{"id":"123","type":"session"}"#;

        // Act
        let session = parse_session(input).unwrap();

        // Assert
        assert_eq!(session.id, "123");
    }

    #[test]
    fn test_parse_handles_malformed_json() {
        let input = "invalid json";
        assert!(parse_session(input).is_err());
    }
}
```

**Test Fixtures**:
- Located in `crates/ccboard-core/tests/fixtures/`
- Use real sanitized data when possible
- Keep fixtures minimal (only necessary fields)

### Benchmarks

Performance-critical code should include benchmarks:

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench startup_bench
```

**Location**: `crates/ccboard-core/benches/`

---

## Pull Request Process

### 1. Create a Feature Branch

```bash
# Update main
git checkout main
git pull upstream main

# Create feature branch
git checkout -b feat/my-feature
# or
git checkout -b fix/issue-123
```

**Branch naming**:
- `feat/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation only
- `refactor/description` - Code refactoring
- `test/description` - Test additions/fixes
- `chore/description` - Maintenance tasks

### 2. Make Changes

Follow the [Pre-Commit Checklist](#pre-commit-checklist) for every commit.

### 3. Write Tests

- Add unit tests in the same file (`#[cfg(test)] mod tests`)
- Add integration tests in `tests/` if needed
- Update documentation tests if API changed

### 4. Update Documentation

- Update README.md if user-facing changes
- Add/update doc comments (`///`) for public APIs
- Update CHANGELOG.md with notable changes

### 5. Pre-Commit Checklist

**REQUIRED before pushing**:

```bash
# 1. Format code
cargo fmt --all

# 2. Check clippy (MUST pass with 0 warnings)
cargo clippy --all-targets -- -D warnings

# 3. Run all tests
cargo test --all

# 4. Build in release mode (catches some edge cases)
cargo build --release --all
```

All checks must pass before submitting PR.

### 6. Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <description>

feat(tui): add search highlighting in Sessions tab
fix(core): handle symlinks in project path validation
docs(readme): update installation instructions
refactor(cache): replace clones with Arc<T>
test(parser): add regression test for #123
chore(deps): update rusqlite to 0.32
```

**Types**:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation only
- `refactor` - Code refactoring (no behavior change)
- `test` - Test additions/fixes
- `perf` - Performance improvements
- `chore` - Maintenance (deps, tooling)
- `ci` - CI/CD changes

**Scopes** (optional):
- `tui`, `web`, `core`, `cli`
- `cache`, `parser`, `store`, `watcher`
- `deps`, `config`, `readme`

### 7. Push and Create PR

```bash
# Push to your fork
git push origin feat/my-feature

# Create PR on GitHub
# Fill in the PR template with:
# - Description of changes
# - Related issue (if any)
# - Testing performed
# - Screenshots (if UI changes)
```

### 8. PR Review Process

1. **CI checks** must pass (formatting, clippy, tests)
2. **Code review** by maintainer
3. **Address feedback** if requested
4. **Squash commits** if needed (maintainer will guide)
5. **Merge** once approved

---

## Architecture Guidelines

### Workspace Structure

ccboard is a Cargo workspace with 4 crates:

```
ccboard/                     # Binary CLI entry point
├─ ccboard-core/             # Shared data layer (NO UI)
│  ├─ parsers/               # JSONL, JSON, YAML parsers
│  ├─ models/                # Domain models
│  ├─ store.rs               # Thread-safe DataStore
│  └─ cache/                 # SQLite metadata cache
├─ ccboard-tui/              # Ratatui frontend
│  ├─ tabs/                  # 7 tab implementations
│  ├─ components/            # Reusable UI components
│  └─ ui.rs                  # Main rendering logic
└─ ccboard-web/              # Leptos + Axum frontend (future)
```

**Dependency Rules**:
- `ccboard-core` NEVER depends on `tui` or `web` (pure business logic)
- `tui` and `web` depend on `core` (UI → logic, not logic → UI)
- `ccboard` binary depends on all crates (entry point)

### Where to Add Features

| Feature Type | Location |
|--------------|----------|
| New parser (JSONL, JSON, etc.) | `ccboard-core/src/parsers/` |
| New domain model | `ccboard-core/src/models/` |
| Cache logic | `ccboard-core/src/cache/` |
| New TUI tab | `ccboard-tui/src/tabs/` |
| Reusable UI component | `ccboard-tui/src/components/` |
| CLI command | `ccboard/src/main.rs` |
| Web page | `ccboard-web/src/pages/` |

### Concurrency Patterns

ccboard uses thread-safe concurrency:

```rust
use dashmap::DashMap;
use parking_lot::RwLock;

pub struct DataStore {
    // ✅ DashMap for high-contention collections (per-key locking)
    sessions: DashMap<String, Arc<SessionMetadata>>,

    // ✅ RwLock for low-contention data (many readers, rare writers)
    stats: RwLock<Option<Stats>>,
    settings: RwLock<Option<Settings>>,
}
```

**Rules**:
- Use `Arc<T>` for shared ownership, minimize `.clone()` of large structs
- Use `DashMap` for collections with many concurrent writes
- Use `parking_lot::RwLock` for data with many reads, rare writes
- NEVER use `std::sync::RwLock` (parking_lot is faster and fairer)
- Avoid nested locks (deadlock risk)

### File Watching

File changes are detected automatically:

```rust
use notify::Watcher;

// Debounced watcher (500ms)
let watcher = notify_debouncer_mini::new_debouncer(
    Duration::from_millis(500),
    |event| { /* handle change */ }
);
```

**Patterns**:
- Stats cache: Reload on `stats-cache.json` change
- Sessions: Update specific session on `.jsonl` change
- Config: Reload cascade on any `settings.json` change

---

## Commit Message Guidelines

### Good Examples

```
feat(cache): implement SQLite metadata cache with 89x speedup

- Add MetadataCache with mtime-based invalidation
- Use WAL mode for concurrent reads
- Background cache population during initial load
- Benchmarks show 20s → 224ms improvement

Closes #45
```

```
fix(parser): handle symlinks in project path sanitization

Previously, symlinks in project paths could bypass path validation,
allowing directory traversal attacks. Now explicitly reject symlinks
with is_symlink() check.

Fixes #78
```

```
docs(readme): add 13 production screenshots

- Dashboard, Sessions, Config, Hooks, Agents, Costs, History, MCP
- Search highlighting and Help modal screenshots
- Collapsible section for additional screenshots
```

### Bad Examples

```
❌ update stuff
❌ fix bug
❌ WIP
❌ changes
```

---

## Questions?

- **Bugs**: [Open an issue](https://github.com/FlorianBruniaux/ccboard/issues/new)
- **Features**: [Start a discussion](https://github.com/FlorianBruniaux/ccboard/discussions)
- **Security**: See [SECURITY.md](SECURITY.md)

---

Thank you for contributing to ccboard! 🎉
