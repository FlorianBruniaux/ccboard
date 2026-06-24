# Async Programming Patterns

Modern async Rust patterns with Tokio runtime, native async traits (1.75+), and production-ready error handling.

## Tokio Runtime Setup

### Basic Application

```rust
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let result = async_operation().await?;
    println!("Result: {}", result);
    Ok(())
}

async fn async_operation() -> Result<String> {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    Ok("Complete".to_string())
}
```

### Multi-threaded Runtime

```rust
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    // Parallel work across 4 threads
    Ok(())
}
```

### Current Thread Runtime (Lightweight)

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Single-threaded async for lightweight apps
    Ok(())
}
```

## Async Functions

### Basic Async Function

```rust
async fn fetch_user(id: u64) -> Result<User> {
    let url = format!("https://api.example.com/users/{}", id);
    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch user")?;

    let user = response
        .json::<User>()
        .await
        .context("Failed to parse user JSON")?;

    Ok(user)
}
```

### Native Async Traits (Rust 1.75+)

```rust
use anyhow::Result;

trait DataStore {
    async fn fetch(&self, key: &str) -> Result<String>;
    async fn store(&mut self, key: &str, value: &str) -> Result<()>;
}

struct RedisStore {
    client: redis::Client,
}

impl DataStore for RedisStore {
    async fn fetch(&self, key: &str) -> Result<String> {
        let mut conn = self.client.get_async_connection().await?;
        let value: String = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(value)
    }

    async fn store(&mut self, key: &str, value: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}
```

## Concurrent Execution

### Spawning Tasks

```rust
use tokio::task;

async fn parallel_operations() -> Result<()> {
    let handle1 = task::spawn(async {
        fetch_data_source_1().await
    });

    let handle2 = task::spawn(async {
        fetch_data_source_2().await
    });

    // Wait for both to complete
    let result1 = handle1.await.context("Task 1 panicked")??;
    let result2 = handle2.await.context("Task 2 panicked")??;

    Ok(())
}
```

### Join Multiple Futures

```rust
use tokio::try_join;

async fn fetch_all_data() -> Result<(User, Posts, Comments)> {
    let user_fut = fetch_user(1);
    let posts_fut = fetch_posts(1);
    let comments_fut = fetch_comments(1);

    // Execute concurrently, fail fast on first error
    let (user, posts, comments) = try_join!(user_fut, posts_fut, comments_fut)?;

    Ok((user, posts, comments))
}
```

### Select (Race Futures)

```rust
use tokio::select;

async fn fetch_with_fallback() -> Result<String> {
    let primary = fetch_from_primary();
    let secondary = fetch_from_secondary();

    select! {
        result = primary => {
            result.context("Primary fetch failed")
        }
        result = secondary => {
            eprintln!("Using secondary source");
            result.context("Secondary fetch failed")
        }
    }
}
```

## Timeouts and Cancellation

### Timeout Pattern

```rust
use tokio::time::{timeout, Duration};
use anyhow::{Context, Result};

async fn fetch_with_timeout(url: &str) -> Result<String> {
    let fetch_future = reqwest::get(url);

    let response = timeout(Duration::from_secs(10), fetch_future)
        .await
        .context("Request timed out after 10 seconds")?
        .context("HTTP request failed")?;

    let body = response.text().await?;
    Ok(body)
}
```

### Graceful Shutdown

```rust
use tokio::signal;
use tokio::sync::mpsc;

async fn run_with_shutdown() -> Result<()> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // Spawn worker task
    let worker = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    println!("Shutting down gracefully");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    println!("Working...");
                }
            }
        }
    });

    // Wait for SIGINT (Ctrl+C)
    signal::ctrl_c().await?;
    println!("Received shutdown signal");

    // Trigger shutdown
    shutdown_tx.send(()).await?;
    worker.await?;

    Ok(())
}
```

## Streams and Iteration

### Async Stream Processing

```rust
use tokio_stream::{StreamExt, Stream};
use std::pin::Pin;

async fn process_stream(
    stream: Pin<Box<dyn Stream<Item = Result<String>> + Send>>
) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let mut stream = stream;

    while let Some(item) = stream.next().await {
        let value = item?;
        results.push(value.to_uppercase());
    }

    Ok(results)
}
```

### Creating Streams

```rust
use tokio_stream::{iter, StreamExt};

async fn process_items(items: Vec<String>) -> Result<()> {
    let stream = iter(items)
        .map(|item| async move {
            process_async(&item).await
        })
        .buffer_unordered(10); // Process 10 concurrently

    tokio::pin!(stream);

    while let Some(result) = stream.next().await {
        let value = result?;
        println!("Processed: {}", value);
    }

    Ok(())
}
```

## Rate Limiting

### Using governor Crate

```rust
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use tokio::time::{sleep, Duration};

struct ApiClient {
    limiter: RateLimiter<
        governor::state::direct::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
    >,
}

impl ApiClient {
    fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }

    async fn fetch(&self, url: &str) -> Result<String> {
        // Wait until rate limit allows
        self.limiter.until_ready().await;

        let response = reqwest::get(url).await?;
        Ok(response.text().await?)
    }
}
```

### Manual Rate Limiting

```rust
use tokio::time::{sleep, Duration, Instant};

struct RateLimiter {
    interval: Duration,
    last_request: Option<Instant>,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        Self {
            interval: Duration::from_secs(1) / requests_per_second,
            last_request: None,
        }
    }

    async fn acquire(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < self.interval {
                sleep(self.interval - elapsed).await;
            }
        }
        self.last_request = Some(Instant::now());
    }
}
```

## Retry Logic with Exponential Backoff

```rust
use tokio::time::{sleep, Duration};
use anyhow::{Context, Result};

async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                let delay = Duration::from_secs(2_u64.pow(attempt));
                eprintln!(
                    "Attempt {} failed: {}. Retrying in {:?}",
                    attempt + 1,
                    e,
                    delay
                );
                sleep(delay).await;
                attempt += 1;
            }
            Err(e) => {
                return Err(e).context(format!(
                    "Operation failed after {} attempts",
                    max_retries + 1
                ));
            }
        }
    }
}

// Usage
async fn fetch_with_retry(url: &str) -> Result<String> {
    retry_with_backoff(
        || async { reqwest::get(url).await?.text().await.map_err(Into::into) },
        3, // max 3 retries
    )
    .await
}
```

## Channels for Communication

### mpsc (Multi-producer, Single-consumer)

```rust
use tokio::sync::mpsc;

async fn worker_pool() -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    // Spawn workers
    for i in 0..5 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(format!("Message from worker {}", i))
                .await
                .unwrap();
        });
    }
    drop(tx); // Close sender so receiver knows when done

    // Receive messages
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }

    Ok(())
}
```

### oneshot (One-time Communication)

```rust
use tokio::sync::oneshot;

async fn fetch_with_notification(url: &str) -> Result<(String, oneshot::Receiver<()>)> {
    let (tx, rx) = oneshot::channel();

    let url = url.to_string();
    tokio::spawn(async move {
        // Long operation
        let result = reqwest::get(&url).await.unwrap();
        let body = result.text().await.unwrap();

        // Notify completion
        let _ = tx.send(());
        body
    });

    Ok(("Started".to_string(), rx))
}
```

## Best Practices

### ✅ Do

- Use `tokio::spawn` for CPU-intensive work to avoid blocking
- Use `try_join` macro (with `!`) for concurrent operations that should fail fast
- Implement timeouts for all network operations
- Use rate limiting for external API calls
- Handle graceful shutdown with `tokio::signal`
- Use async traits (native in 1.75+) for clean abstractions

### ❌ Don't

- Don't use `.unwrap()` in async code (errors harder to debug)
- Don't spawn unlimited tasks (use semaphores or pools)
- Don't block async runtime with synchronous operations
- Don't forget to handle task panics (check `.await` on `JoinHandle`)
- Don't use `std::thread::sleep` in async (use `tokio::time::sleep`)

## Common Pitfalls

### Blocking the Runtime

```rust
// ❌ Bad: Blocks async runtime
async fn bad_blocking() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks thread!
}

// ✅ Good: Async sleep
async fn good_async() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// ✅ Good: Offload blocking work
async fn good_offload() {
    tokio::task::spawn_blocking(|| {
        // Expensive CPU work or blocking IO
        std::thread::sleep(Duration::from_secs(1));
    })
    .await
    .unwrap();
}
```

### Forgetting to .await

```rust
// ❌ Bad: Future created but not executed
async fn bad_no_await() {
    fetch_data(); // Does nothing! Returns Future<T>
}

// ✅ Good: Await the future
async fn good_await() {
    fetch_data().await;
}
```
