# Review Comment Template

Use this template to generate GitHub PR review comments. Fill in each section based on the rust-ccboard agent output. Comments are posted in **English** (international audience).

---

## Template

```markdown
## Review

**Scope**: Rust correctness, concurrency safety, error handling, performance, test coverage

### Summary

{1-2 sentences: overall assessment. Be direct -- what's the main takeaway? Does it follow ccboard patterns?}

### Critical Issues (block merge)

{List issues that must be fixed before merge. For each:}
{- `file.rs:42` -- Description of the problem. Why it matters. Suggested fix.}

{If none: "None found."}

### Important Issues (should fix)

{List significant issues that should be addressed. For each:}
{- `file.rs:42` -- Description. Why it matters. Suggested fix.}

{If none: "None found."}

### Suggestions (nice to have)

{List minor improvements and style points. For each:}
{- Description. Context. Optional fix.}

{If none: omit this section.}

### What's Good

{Always include at least 1 positive point. Be specific -- what works well and why.}
{- Description of what's done right.}

---
*Automated review via [ccboard](https://github.com/FlorianBruniaux/ccboard) `/pr-triage`*
```

---

## Formatting Rules

**Citation format**: `crates/ccboard-core/src/parsers/session_index.rs:42` or `` `code snippet` `` for inline references

**Issue severity**:
- Critical (block merge): error handling missing, data safety risk, broken ccboard patterns, no tests for new feature, clippy failures
- Important (should fix): performance regression, scope creep, missing graceful degradation, incorrect concurrency primitive
- Suggestion: naming, DRY opportunity, documentation, style

**ccboard-specific checks to mention if relevant**:

Critical:
- `anyhow::Result` + `.context("msg")` on every `?` -- no bare `?`, no `.unwrap()` in production code
- `parking_lot::RwLock` (not `std::sync::RwLock`) for shared state
- No blocking in async: `std::thread::sleep` -> `tokio::time::sleep`, blocking I/O -> `tokio::task::spawn_blocking`
- Graceful degradation: parsers return `Option<T>`, populate `LoadReport`, never `panic!` on malformed input
- `cargo fmt --all` + `cargo clippy --all-targets` must pass with zero warnings
- Tests use real fixtures from `tests/fixtures/` (not synthetic data)

Important:
- `thiserror` in `ccboard-core` (library), `anyhow` in `ccboard-tui`/`ccboard-web`/`ccboard` (binaries)
- No full JSONL loading at startup -- `BufReader` line-by-line, metadata-only
- `DashMap` for high-contention collections (sessions), `parking_lot::RwLock` for low-contention (stats, config)
- Read-only constraint respected: no writes to `~/.claude/*`
- Startup performance maintained: <2s for 1000+ sessions

Suggestions:
- `Arc<T>` for shared ownership instead of cloning large structs
- `JoinSet` for bounded parallel scanning
- `tokio::sync::broadcast` for EventBus patterns

**Tone**: Professional, constructive, factual. Challenge the code, not the contributor. No superlatives ("great", "amazing", "perfect"). No filler ("as mentioned", "it's worth noting").

**Length**: 200-400 words. Long enough to be useful, short enough to be read.
