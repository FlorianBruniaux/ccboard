# Testing Patterns

Comprehensive testing strategies for Rust with embedded tests, mocks, property-based testing, and benchmarks.

## Test Organization

### Embedded Unit Tests

```rust
// In src/lib.rs or src/module.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f64, b: f64) -> Result<f64, &'static str> {
    if b == 0.0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[test]
    fn test_divide_success() {
        let result = divide(10.0, 2.0).unwrap();
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_divide_by_zero() {
        let result = divide(10.0, 0.0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Division by zero");
    }
}
```

### Integration Tests

```rust
// In tests/integration_test.rs
use my_crate::public_api_function;

#[test]
fn test_public_api() {
    let result = public_api_function();
    assert!(result.is_ok());
}
```

### Test Module Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions for tests only
    fn setup() -> TestData {
        TestData::new()
    }

    mod unit {
        use super::*;

        #[test]
        fn test_internal_logic() {
            // Unit test
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_component_interaction() {
            // Integration test
        }
    }
}
```

## Assertion Patterns

### Basic Assertions

```rust
#[test]
fn test_assertions() {
    assert!(true);
    assert_eq!(2 + 2, 4);
    assert_ne!(5, 3);

    let result: Result<i32, &str> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}
```

### Custom Error Messages

```rust
#[test]
fn test_with_messages() {
    let x = 10;
    assert!(x > 5, "x should be greater than 5, got {}", x);
    assert_eq!(x, 10, "x should equal 10");
}
```

### Floating Point Comparisons

```rust
#[test]
fn test_floats() {
    let a = 0.1 + 0.2;
    let b = 0.3;

    // ❌ Don't compare floats directly
    // assert_eq!(a, b); // Fails due to precision

    // ✅ Use epsilon comparison
    assert!((a - b).abs() < 1e-10);

    // Or use approx crate
    // use approx::assert_relative_eq;
    // assert_relative_eq!(a, b, epsilon = 1e-10);
}
```

## Test Annotations

### Expected Panics

```rust
#[test]
#[should_panic]
fn test_panic() {
    panic!("This test passes because it panics");
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_panic_with_message() {
    divide_panic(10, 0); // Should panic with specific message
}
```

### Ignored Tests

```rust
#[test]
#[ignore]
fn expensive_test() {
    // Run only with: cargo test -- --ignored
}

#[test]
#[ignore = "waiting for bug fix"]
fn test_known_bug() {
    // Skip until issue resolved
}
```

### Async Tests

```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_operation().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_with_timeout() {
    use tokio::time::{timeout, Duration};

    let result = timeout(
        Duration::from_secs(1),
        slow_async_operation()
    ).await;

    assert!(result.is_ok(), "Operation timed out");
}
```

## Mocking and Dependency Injection

### Trait-Based Mocking

```rust
trait Database {
    fn fetch_user(&self, id: u64) -> Result<User, DbError>;
    fn store_user(&mut self, user: &User) -> Result<(), DbError>;
}

// Production implementation
struct PostgresDb {
    conn: Connection,
}

impl Database for PostgresDb {
    fn fetch_user(&self, id: u64) -> Result<User, DbError> {
        // Real database query
    }

    fn store_user(&mut self, user: &User) -> Result<(), DbError> {
        // Real database write
    }
}

// Test mock
#[cfg(test)]
struct MockDb {
    users: std::collections::HashMap<u64, User>,
}

#[cfg(test)]
impl Database for MockDb {
    fn fetch_user(&self, id: u64) -> Result<User, DbError> {
        self.users.get(&id)
            .cloned()
            .ok_or(DbError::NotFound)
    }

    fn store_user(&mut self, user: &User) -> Result<(), DbError> {
        self.users.insert(user.id, user.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_service() {
        let mut db = MockDb {
            users: std::collections::HashMap::new(),
        };

        let user = User { id: 1, name: "Alice".to_string() };
        db.store_user(&user).unwrap();

        let fetched = db.fetch_user(1).unwrap();
        assert_eq!(fetched.name, "Alice");
    }
}
```

### Dependency Injection Pattern

```rust
struct UserService<D: Database> {
    db: D,
}

impl<D: Database> UserService<D> {
    fn new(db: D) -> Self {
        Self { db }
    }

    fn get_user(&self, id: u64) -> Result<User, DbError> {
        self.db.fetch_user(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_service_with_mock() {
        let mock_db = MockDb::new();
        let service = UserService::new(mock_db);

        let result = service.get_user(1);
        // Assert expected behavior
    }
}
```

## Test Fixtures and Setup

### Setup Function Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TestContext {
        TestContext {
            db: MockDb::new(),
            config: Config::default(),
        }
    }

    #[test]
    fn test_with_setup() {
        let ctx = setup();
        // Use ctx in test
    }
}
```

### Test Data Builders

```rust
#[cfg(test)]
struct UserBuilder {
    id: u64,
    name: String,
    email: String,
}

#[cfg(test)]
impl UserBuilder {
    fn new() -> Self {
        Self {
            id: 1,
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        }
    }

    fn id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    fn build(self) -> User {
        User {
            id: self.id,
            name: self.name,
            email: self.email,
        }
    }
}

#[test]
fn test_with_builder() {
    let user = UserBuilder::new()
        .id(42)
        .name("Alice")
        .build();

    assert_eq!(user.id, 42);
    assert_eq!(user.name, "Alice");
}
```

## Property-Based Testing

### Using proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_add_commutative(a in 0..1000i32, b in 0..1000i32) {
        assert_eq!(add(a, b), add(b, a));
    }

    #[test]
    fn test_string_roundtrip(s in "\\PC*") {
        let encoded = encode(&s);
        let decoded = decode(&encoded)?;
        prop_assert_eq!(s, decoded);
    }
}
```

### Using quickcheck

```rust
#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn test_reverse_twice(xs: Vec<i32>) -> bool {
        let reversed = xs.iter().rev().collect::<Vec<_>>();
        let reversed_twice = reversed.iter().rev().collect::<Vec<_>>();
        xs.iter().collect::<Vec<_>>() == reversed_twice
    }
}
```

## Testing Error Cases

### Result Testing

```rust
#[test]
fn test_error_cases() {
    let result = risky_operation();

    match result {
        Ok(_) => panic!("Expected error"),
        Err(e) => {
            assert_eq!(e.to_string(), "Expected error message");
        }
    }
}

#[test]
fn test_with_question_mark() -> Result<(), Box<dyn std::error::Error>> {
    let value = operation_that_might_fail()?;
    assert_eq!(value, 42);
    Ok(())
}
```

### Testing Panic Recovery

```rust
#[test]
fn test_catch_panic() {
    let result = std::panic::catch_unwind(|| {
        panic!("Expected panic");
    });

    assert!(result.is_err());
}
```

## Benchmarking

### Criterion Benchmarks

```rust
// In benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 20", |b| {
        b.iter(|| fibonacci(black_box(20)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

## Best Practices

### ✅ Do

- Place tests in `#[cfg(test)] mod tests`
- Use `use super::*;` to import parent module items
- Test both success and error cases
- Provide descriptive test names: `test_<what>_<condition>_<expected>`
- Use builders or fixtures for complex test data
- Mock external dependencies with traits
- Run `cargo test` before every commit

### ❌ Don't

- Don't use `.unwrap()` in production code (OK in tests)
- Don't write tests that depend on each other
- Don't test implementation details (test behavior)
- Don't ignore failing tests
- Don't skip error case testing

## Test Coverage

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Open coverage/index.html in browser
```

### Coverage Annotations

```rust
#[cfg(not(tarpaulin_include))]
fn debugging_helper() {
    // Exclude from coverage
}
```

## RTK Project Testing Patterns

### Command Output Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_parsing() {
        let output = "modified: src/main.rs\ndeleted: old.rs";
        let changes = parse_git_status(output);

        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].status, "modified");
    }
}
```

### CLI Argument Testing

```rust
#[cfg(test)]
mod tests {
    use clap::Parser;

    #[test]
    fn test_cli_parsing() {
        let args = Cli::parse_from(vec!["rtk", "diff", "--cached"]);
        assert_eq!(args.command, Command::Diff);
    }
}
```
