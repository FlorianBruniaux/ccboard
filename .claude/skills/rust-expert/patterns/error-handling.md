# Error Handling Patterns

Comprehensive guide to Rust error handling using `anyhow`, `thiserror`, and `Result` patterns.

## Application-Level: anyhow

Use `anyhow::Result<T>` for application code where you need quick error propagation with context.

### Basic Pattern

```rust
use anyhow::{Context, Result};

fn read_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config from {}", path))?;

    let config: Config = toml::from_str(&content)
        .context("Failed to parse TOML config")?;

    Ok(config)
}
```

### Context Chaining

```rust
fn process_user_data(user_id: u64) -> Result<()> {
    let user = fetch_user(user_id)
        .context("Database query failed")?;

    validate_user(&user)
        .context(format!("Validation failed for user {}", user_id))?;

    update_user(&user)
        .with_context(|| format!("Failed to update user {} in database", user_id))?;

    Ok(())
}
```

**Note**: Use `with_context(|| ...)` for expensive string formatting to avoid allocation on success path.

### Error Downcast

```rust
use anyhow::{anyhow, Result};

fn handle_api_error(err: anyhow::Error) -> Result<()> {
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            std::io::ErrorKind::NotFound => {
                println!("File not found, creating new one");
                return Ok(());
            }
            _ => return Err(err),
        }
    }
    Err(err)
}
```

## Library-Level: thiserror

Use `thiserror` for libraries to provide structured error types that users can match on.

### Custom Error Type

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    #[error("Invalid configuration: {field}")]
    InvalidConfig { field: String },

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Parse error")]
    Parse(#[from] serde_json::Error),
}
```

### Usage

```rust
fn authenticate(token: &str) -> Result<User, AppError> {
    if token.is_empty() {
        return Err(AppError::Auth("Empty token".to_string()));
    }

    let user = fetch_user_by_token(token)
        .map_err(|e| AppError::Auth(format!("Token validation failed: {}", e)))?;

    Ok(user)
}

// Caller can match on specific errors
match authenticate(token) {
    Ok(user) => println!("Welcome {}", user.name),
    Err(AppError::RateLimited(secs)) => {
        println!("Please wait {} seconds", secs);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## RTK Project Patterns

Extracted from the rtk CLI tool project.

### Command Execution with Context

```rust
use anyhow::{Context, Result};
use std::process::Command;

pub fn run_git_command(args: &[String]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    Ok(())
}
```

### Early Return Pattern

```rust
fn validate_arguments(args: &[String]) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("No arguments provided");
    }

    if args[0].starts_with("--dangerous") {
        anyhow::bail!("Dangerous flag not allowed");
    }

    Ok(())
}
```

### Multiple Error Sources

```rust
use anyhow::Result;

fn complex_operation() -> Result<()> {
    // IO error automatically converted
    let file_content = std::fs::read_to_string("config.toml")?;

    // Parse error automatically converted
    let config: Config = toml::from_str(&file_content)?;

    // Custom context
    validate_config(&config)
        .context("Configuration validation failed")?;

    Ok(())
}
```

## Async Error Handling

### Tokio with anyhow

```rust
use anyhow::{Context, Result};
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(url: &str) -> Result<String> {
    let response = timeout(
        Duration::from_secs(10),
        reqwest::get(url)
    )
    .await
    .context("Request timed out")?
    .context("Failed to send request")?;

    let body = response
        .text()
        .await
        .context("Failed to read response body")?;

    Ok(body)
}
```

### Error Propagation in Spawn

```rust
use anyhow::Result;

async fn parallel_operations() -> Result<()> {
    let handle1 = tokio::spawn(async move {
        operation1().await
    });

    let handle2 = tokio::spawn(async move {
        operation2().await
    });

    // Join and propagate errors
    let result1 = handle1.await
        .context("Task 1 panicked")??;
    let result2 = handle2.await
        .context("Task 2 panicked")??;

    Ok(())
}
```

## Best Practices

### ✅ Do

- Use `.context()` on every `?` to add meaningful information
- Return `Result<()>` from functions that can fail
- Use `anyhow::bail` macro (with `()`) for early returns with errors
- Use `with_context(|| ...)` for expensive error messages
- Provide actionable error messages (what went wrong + why)

### ❌ Don't

- Don't use `.unwrap()` in production code
- Don't use `.expect()` without clear justification
- Don't swallow errors silently
- Don't return generic error strings
- Don't panic in library code

### Exception: Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_config_parsing() {
        let config = parse_config("test.toml").unwrap(); // OK in tests
        assert_eq!(config.port, 8080);
    }
}
```

## Error Recovery Strategies

### Retry with Backoff

```rust
use anyhow::Result;
use tokio::time::{sleep, Duration};

async fn retry_with_backoff<F, T>(
    mut operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut attempt = 0;
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                let delay = Duration::from_secs(2_u64.pow(attempt));
                eprintln!("Attempt {} failed, retrying in {:?}", attempt + 1, delay);
                sleep(delay).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### Fallback Pattern

```rust
fn load_config() -> Result<Config> {
    load_from_file("config.toml")
        .or_else(|_| load_from_env())
        .or_else(|_| Ok(Config::default()))
}
```
