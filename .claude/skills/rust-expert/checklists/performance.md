# Performance Optimization Checklist

Systematic approach to optimizing Rust build times and runtime performance.

## Build Performance

### Linker Optimization

- [ ] **Use mold or lld for faster linking**
  ```toml
  # .cargo/config.toml
  [target.x86_64-unknown-linux-gnu]
  linker = "clang"
  rustflags = ["-C", "link-arg=-fuse-ld=mold"]

  # Or for macOS
  [target.x86_64-apple-darwin]
  rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/bin/zld"]
  ```

- [ ] **Parallel codegen enabled**
  ```toml
  [profile.dev]
  codegen-units = 16
  ```

### Incremental Compilation

- [ ] **sccache configured for caching**
  ```bash
  cargo install sccache
  export RUSTC_WRAPPER=sccache
  ```

- [ ] **Incremental compilation enabled**
  ```toml
  [profile.dev]
  incremental = true
  ```

### Dependency Management

- [ ] **Minimal dependency tree**
  - Review `cargo tree` output
  - Use feature flags to disable unused features
  - Consider lighter alternatives

- [ ] **Feature flags optimized**
  ```toml
  [dependencies]
  tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
  # Not: tokio = { version = "1", features = ["full"] }
  ```

- [ ] **Build scripts minimized**
  - Keep `build.rs` simple
  - Cache expensive operations

## Runtime Performance

### Profiling

- [ ] **Profile before optimizing**
  ```bash
  cargo install cargo-flamegraph
  cargo flamegraph --bin myapp
  ```

- [ ] **Benchmark critical paths**
  ```bash
  cargo install cargo-criterion
  cargo criterion
  ```

### Allocations

- [ ] **Minimize heap allocations**
  - Use `&str` instead of `String` where possible
  - Use `&[T]` instead of `Vec<T>` for read-only data
  - Pre-allocate with `Vec::with_capacity`
  - Use `Cow` for conditionally owned data

- [ ] **Avoid unnecessary clones**
  ```rust
  // ❌ Bad: Unnecessary clone
  fn process(data: Vec<String>) -> Vec<String> {
      data.iter().map(|s| s.to_uppercase()).collect()
  }

  // ✅ Good: Use references
  fn process(data: &[String]) -> Vec<String> {
      data.iter().map(|s| s.to_uppercase()).collect()
  }
  ```

- [ ] **Reuse allocations**
  ```rust
  let mut buffer = String::new();
  for item in items {
      buffer.clear();
      write!(&mut buffer, "{}", item)?;
      process(&buffer);
  }
  ```

### Data Structures

- [ ] **Choose appropriate collections**
  - `Vec` for sequential data
  - `HashMap` for fast lookups (random insertion order)
  - `BTreeMap` for sorted keys
  - `HashSet` for membership testing
  - `VecDeque` for queues

- [ ] **Size collections appropriately**
  ```rust
  // Pre-allocate if size known
  let mut vec = Vec::with_capacity(1000);

  // Reserve additional capacity
  vec.reserve(500);
  ```

### Iteration

- [ ] **Use iterator chains**
  ```rust
  // ✅ Good: Iterator chain (lazy)
  items.iter()
      .filter(|x| x.is_valid())
      .map(|x| x.process())
      .collect()

  // ❌ Bad: Multiple intermediate collections
  let valid: Vec<_> = items.iter().filter(|x| x.is_valid()).collect();
  let processed: Vec<_> = valid.iter().map(|x| x.process()).collect();
  ```

- [ ] **Avoid collecting intermediate results**
  ```rust
  // ✅ Good: Count directly
  let count = items.iter().filter(|x| x.is_valid()).count();

  // ❌ Bad: Unnecessary collection
  let count = items.iter().filter(|x| x.is_valid()).collect::<Vec<_>>().len();
  ```

### Async Performance

- [ ] **Spawn tasks for CPU work**
  ```rust
  tokio::task::spawn_blocking(|| {
      // CPU-intensive work
  }).await?;
  ```

- [ ] **Use buffered operations**
  ```rust
  use tokio_stream::StreamExt;

  stream
      .map(|item| async move { process(item).await })
      .buffer_unordered(10) // Process 10 concurrently
      .collect::<Vec<_>>()
      .await;
  ```

- [ ] **Implement proper timeouts**
  ```rust
  tokio::time::timeout(Duration::from_secs(5), operation()).await??;
  ```

## Release Profile Optimization

### Cargo.toml Settings

```toml
[profile.release]
# Enable all optimizations
opt-level = 3

# Link-time optimization (slower build, faster runtime)
lto = true

# Code generation units (1 = best optimization, slower build)
codegen-units = 1

# Strip debug symbols from binary
strip = true

# Panic strategy (abort is slightly faster)
panic = "abort"
```

### Size Optimization

```toml
[profile.release]
# Optimize for size instead of speed
opt-level = "z"

# Or slightly less aggressive
opt-level = "s"

# Strip symbols
strip = true

# Enable LTO
lto = true
```

## Specific Optimizations

### String Operations

- [ ] **Use efficient string building**
  ```rust
  // ✅ Good: Pre-allocate
  let mut s = String::with_capacity(100);
  for item in items {
      write!(s, "{}", item)?;
  }

  // ❌ Bad: Repeated allocations
  let mut s = String::new();
  for item in items {
      s = format!("{}{}", s, item);
  }
  ```

### Pattern Matching

- [ ] **Match expressions fully optimized**
  ```rust
  // ✅ Good: Compiler can optimize
  match value {
      0 => handle_zero(),
      1..=10 => handle_small(),
      _ => handle_large(),
  }
  ```

### Lazy Evaluation

- [ ] **Use lazy_static for expensive initialization**
  ```rust
  use lazy_static::lazy_static;
  use regex::Regex;

  lazy_static! {
      static ref EMAIL_REGEX: Regex = Regex::new(r"^[^@]+@[^@]+$").unwrap();
  }
  ```

## Profiling Commands

### CPU Profiling

```bash
# Flamegraph
cargo flamegraph --bin myapp -- args

# perf (Linux)
cargo build --release
perf record --call-graph=dwarf ./target/release/myapp
perf report
```

### Memory Profiling

```bash
# Valgrind massif
valgrind --tool=massif ./target/release/myapp
ms_print massif.out.*

# Heaptrack (Linux)
heaptrack ./target/release/myapp
heaptrack_gui heaptrack.myapp.*
```

### Benchmarking

```bash
# Criterion
cargo criterion

# Hyperfine (compare binaries)
hyperfine './target/release/myapp' './target/release/other'
```

## Measurement Commands

```bash
# Build time
cargo clean
time cargo build --release

# Binary size
ls -lh target/release/myapp

# Dependency analysis
cargo tree
cargo tree --duplicate

# Unused dependencies
cargo install cargo-udeps
cargo +nightly udeps
```

## Best Practices

### ✅ Do

- Profile before optimizing (measure first)
- Focus on hot paths identified by profiling
- Use release builds for benchmarking
- Benchmark with realistic data
- Consider trade-offs (speed vs memory vs binary size)
- Use `#[inline]` for small hot functions
- Leverage compile-time computation with `const fn`

### ❌ Don't

- Don't optimize prematurely
- Don't optimize cold paths
- Don't sacrifice readability without measurement
- Don't benchmark in debug mode
- Don't trust micro-benchmarks alone
- Don't forget to test after optimizations

## Common Pitfalls

### Debug vs Release

```rust
// Always benchmark in release mode
cargo build --release
hyperfine './target/release/myapp'

// NOT:
cargo build
hyperfine './target/debug/myapp'  // ❌ Wrong!
```

### Unnecessary Bounds Checks

```rust
// ✅ Good: Iterator avoids bounds checks
for item in vec.iter() {
    process(item);
}

// ❌ Bad: Index access requires bounds checks
for i in 0..vec.len() {
    process(&vec[i]);
}
```

### String Concatenation

```rust
// ✅ Good: Single allocation
let result = format!("{} {} {}", a, b, c);

// ❌ Bad: Multiple allocations
let result = a.to_string() + " " + &b + " " + &c;
```
