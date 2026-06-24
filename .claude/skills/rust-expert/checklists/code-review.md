# Rust Code Review Checklist

Use this checklist before committing Rust code to ensure quality and idiomatic patterns.

## 🔴 Critical (Must Fix)

### Error Handling
- [ ] No `.unwrap()` in production code (except documented justification)
- [ ] No `panic` macro (with `!()`) in library code
- [ ] All public functions return `Result<T>` or `Option<T>` where appropriate
- [ ] All `?` operators have `.context()` for meaningful error messages
- [ ] Error messages are actionable and user-friendly

### Safety & Correctness
- [ ] No unsafe blocks without documentation and safety invariants
- [ ] No data races or undefined behavior
- [ ] Integer operations checked for overflow in critical paths
- [ ] Array/slice accesses are bounds-checked or justified
- [ ] No use of deprecated APIs

### Ownership & Borrowing
- [ ] No unnecessary clones (use references where possible)
- [ ] Lifetimes are explicit and correctly bounded
- [ ] Smart pointers (`Arc`, `Rc`, `Box`) used appropriately
- [ ] No dangling references or use-after-free potential

## 🟡 Important (Strong Preference)

### Code Quality
- [ ] `cargo fmt` has been run (consistent formatting)
- [ ] `cargo clippy` passes with zero warnings
- [ ] `cargo test` passes with all tests green
- [ ] No commented-out code (delete or document why)
- [ ] No `TODO` comments for core functionality

### Testing
- [ ] Tests exist in `#[cfg(test)] mod tests`
- [ ] Both success and error paths are tested
- [ ] Public API functions have test coverage
- [ ] Edge cases and boundary conditions tested
- [ ] Integration tests for user-facing features

### Documentation
- [ ] Public API items have `///` doc comments
- [ ] Modules have `//` with `!` for module-level documentation
- [ ] Complex logic has inline comments explaining "why"
- [ ] Examples provided for non-obvious usage

### Naming & Organization
- [ ] Impl blocks placed immediately after type definitions
- [ ] Names follow Rust conventions (snake_case, CamelCase)
- [ ] Functions are focused and have single responsibility
- [ ] Module structure is logical and navigable

### Dependencies
- [ ] New dependencies are justified and necessary
- [ ] Dependencies are up-to-date and maintained
- [ ] Feature flags used to minimize dependency bloat
- [ ] No circular dependencies

## 🟢 Recommended (Apply When Practical)

### Performance
- [ ] Allocations minimized where possible
- [ ] `&str` used over `String` for function parameters
- [ ] `&[T]` used over `Vec<T>` for read-only slices
- [ ] Iterator chains preferred over collecting intermediate vectors
- [ ] Appropriate data structures chosen (HashMap vs BTreeMap, etc.)

### Idiomatic Patterns
- [ ] Standard traits implemented (`Debug`, `Clone`, `Default` where applicable)
- [ ] Builder pattern for complex construction
- [ ] Type-state pattern for state machines
- [ ] Trait objects used for runtime polymorphism where needed
- [ ] Enums with `match` exhaustively handled

### Error Types
- [ ] Library code uses custom error types (thiserror)
- [ ] Application code uses `anyhow::Result<T>`
- [ ] Error types are meaningful and specific
- [ ] Error conversion implemented with `From` trait

### Code Style
- [ ] Functions are reasonably sized (<100 lines ideal)
- [ ] Nesting depth is reasonable (<4 levels)
- [ ] Variables have descriptive names
- [ ] Magic numbers extracted to named constants
- [ ] Consistent error handling strategy throughout

## Async Code Specific

### Tokio Runtime
- [ ] No `std::thread::sleep` in async code (use `tokio::time::sleep`)
- [ ] Blocking operations wrapped in `spawn_blocking`
- [ ] Tasks don't panic without being handled
- [ ] Graceful shutdown implemented for long-running tasks

### Async Patterns
- [ ] Timeouts set for network operations
- [ ] Rate limiting implemented for external APIs
- [ ] Retry logic with exponential backoff where appropriate
- [ ] Proper error propagation in spawned tasks

## CLI-Specific

### Argument Parsing
- [ ] Clap derive API used for type safety
- [ ] Help messages are clear and informative
- [ ] Subcommands logically organized
- [ ] Global flags properly handled

### User Experience
- [ ] Progress indicators for long operations
- [ ] Colored output for better readability
- [ ] Error messages printed to stderr
- [ ] Proper exit codes returned
- [ ] SIGINT/SIGTERM handled gracefully

## Security

### Input Validation
- [ ] User input validated and sanitized
- [ ] File paths validated (no path traversal)
- [ ] Command injection prevented (use `Command` API properly)
- [ ] SQL injection prevented (use parameterized queries)

### Secrets Management
- [ ] No hardcoded secrets or API keys
- [ ] Secrets loaded from environment or secure storage
- [ ] Secrets not logged or printed
- [ ] Credentials not exposed in error messages

## Pre-Commit Commands

Run these before committing:

```bash
# Format code
cargo fmt --all

# Check for common mistakes
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Check documentation
cargo doc --no-deps --all-features

# Security audit (optional but recommended)
cargo audit
```

## Post-Review Actions

After passing checklist:
1. [ ] Create feature branch if not already on one
2. [ ] Commit with descriptive message
3. [ ] Push to remote
4. [ ] Open PR with description and tests
5. [ ] Request review from team

## Exception Notes

Document any exceptions to the checklist with clear justification:

```rust
// EXCEPTION: Using unwrap here because [specific reason]
// If this fails, it indicates a programming error that should be caught in testing
let value = operation().unwrap();
```
