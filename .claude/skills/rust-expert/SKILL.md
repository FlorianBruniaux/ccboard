---
name: rust-expert
description: Expert Rust idiomatique pour développement CLI/système. Ownership, error handling avec anyhow/thiserror, traits, async Tokio, testing. Utiliser pour coder, reviewer ou refactorer du Rust.
allowed-tools: Read, Grep, Glob, Bash
effort: medium
tags: [rust, expert, ownership, async, error-handling]
---

# Rust Expert Skill

Expert in idiomatic Rust 1.75+ development with focus on CLI/system tools, ownership patterns, error handling, traits, async programming, and testing.

## Core Competencies

### 1. Ownership & Borrowing Mastery
- Prefer `&str` over `String` for function parameters
- Use slices (`&[T]`) over `Vec<T>` when ownership not needed
- Leverage `Cow<str>` for conditionally owned strings
- Apply `Arc<T>` and `Arc<Mutex<T>>` for thread-safe sharing
- Design lifetimes explicitly when necessary

### 2. Error Handling Excellence
- **Default**: `anyhow::Result<T>` for applications
- **Libraries**: `thiserror` for custom error types
- **Context**: Always use `.context("operation description")` with `?`
- **No unwrap**: Use `?` operator or `expect()` with justification
- **Recovery**: Provide actionable error messages

### 3. Trait-Driven Design
- Implement standard traits: `Debug`, `Clone`, `Default`, `Display`
- Use trait objects (`dyn Trait`) for runtime polymorphism
- Leverage trait bounds for generic constraints
- Organize impl blocks immediately after type definitions
- Apply async traits (native in Rust 1.75+)

### 4. Async Programming Patterns
- **Runtime**: Tokio for production async
- **Patterns**: `async fn`, `tokio::spawn`, `tokio::select` macro (with `!`)
- **Streams**: `tokio_stream` for async iteration
- **Rate limiting**: `governor` crate for API throttling
- **Retries**: Exponential backoff with `tokio::time::sleep`

### 5. CLI Development Best Practices
- **Argument parsing**: `clap` with derive macros
- **Config**: `serde` + `toml`/`yaml` for structured config
- **Output**: `indicatif` for progress, `colored` for styling
- **Terminal**: Handle `SIGINT` gracefully, cleanup on exit

### 6. Testing Strategy
- Embedded tests: `#[cfg(test)] mod tests { use super::*; }`
- Unit tests alongside code, integration tests in `tests/`
- Use `#[should_panic]` for expected panics
- Mock external dependencies with traits
- Property-based testing with `proptest` or `quickcheck`

### 7. Build Optimization
- **Linker**: Use `mold` or `lld` for faster linking
- **Cache**: `sccache` for incremental builds
- **Profile**: `cargo build --release` with LTO
- **Dependencies**: Minimize with feature flags
- **Workspace**: Organize large projects with `workspace`

### 8. Code Quality Standards
- **Before commit**: `cargo fmt`, `cargo clippy`, `cargo test`
- **Clippy**: Zero warnings policy
- **Documentation**: `///` for public APIs, `//` with `!` for modules
- **Examples**: Provide runnable examples in docs

### 9. Project Structure Patterns
- Flat module hierarchy when possible (rtk style)
- `src/main.rs` for binaries, `src/lib.rs` for libraries
- Group related functionality in modules
- Keep functions focused and small
- Avoid deep nesting (max 3-4 levels)

## Reference Files

This skill includes detailed patterns and checklists:

- **Patterns**:
  - `patterns/error-handling.md` - anyhow, thiserror, Result patterns
  - `patterns/ownership.md` - &str vs String, Cow, Arc, lifetimes
  - `patterns/async-patterns.md` - Tokio, futures, async traits
  - `patterns/traits-impl.md` - Trait implementation organization
  - `patterns/testing.md` - Test organization and assertions
  - `patterns/cli-patterns.md` - clap derive, subcommands

- **Checklists**:
  - `checklists/code-review.md` - Pre-commit review checklist
  - `checklists/performance.md` - Build optimization checklist

- **Anti-Patterns**:
  - `anti-patterns/common-mistakes.md` - Frequent Rust mistakes
  - `anti-patterns/memory-pitfalls.md` - Ownership pitfalls

- **Examples**:
  - `examples/rtk-patterns.md` - Real-world patterns from rtk project

## When to Use This Skill

✅ **Use for**:
- Writing new Rust code (CLI tools, libraries, async services)
- Code review of Rust PRs
- Refactoring to idiomatic patterns
- Performance optimization
- Error handling improvements
- Test organization

❌ **Don't use for**:
- Simple syntax questions (use native knowledge)
- Non-Rust languages
- Project setup without coding

## Integration with SuperClaude Framework

- **Works with**: backend-architect (system design), TDD (test-first), code-reviewer
- **Flags**: Responds to `--think` for architectural analysis
- **Mode**: Compatible with `--uc` token efficiency mode
- **Language**: English for shareability, French instructions supported

## Quick Reference

### Error Handling
```rust
use anyhow::{Context, Result};

fn process_file(path: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read file")?;
    Ok(())
}
```

### Ownership Patterns
```rust
// Prefer borrowing
fn print_name(name: &str) { println!("{}", name); }

// Conditionally owned
use std::borrow::Cow;
fn maybe_modify(input: &str, modify: bool) -> Cow<str> {
    if modify {
        Cow::Owned(input.to_uppercase())
    } else {
        Cow::Borrowed(input)
    }
}
```

### Async Patterns
```rust
#[tokio::main]
async fn main() -> Result<()> {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        fetch_data()
    ).await??;
    Ok(())
}
```

### Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculation() {
        assert_eq!(calculate(2, 3), 5);
    }
}
```
