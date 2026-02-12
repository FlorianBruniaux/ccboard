---
name: rust-patterns
description: |
  Rust design patterns reference - idioms, creational/structural/behavioral patterns,
  Rust-specific patterns, and critical anti-patterns to avoid.
  Keywords: rust, patterns, idioms, ownership, traits, lifetimes, anti-patterns
metadata:
  category: framework
  priority: critical
  version: "1.0.0"
---

# Rust Design Patterns

Comprehensive guide to Rust idioms, design patterns, and anti-patterns for Claude Code operations.

## Priority System

**🔴 CRITICAL**: Always enforce - violation blocks merge/PR
**🟡 IMPORTANT**: Strong preference - deviation requires justification
**🟢 RECOMMENDED**: Apply when practical - optimization opportunities

---

## Core Rust Idioms

### Ownership & Borrowing (🔴 Critical)

**Priority**: 🔴 **Triggers**: Function signatures, data structures, API design

**Prefer References Over Ownership**:

```rust
// ✅ Right: Accepts both &String and &str
fn process_text(text: &str) -> usize {
    text.len()
}

// ❌ Wrong: Forces ownership transfer
fn process_text(text: String) -> usize {
    text.len()
}
```

**Slice Over Vector**:

```rust
// ✅ Right: Flexible, works with Vec, array, slice
fn sum_items(items: &[i32]) -> i32 {
    items.iter().sum()
}

// ❌ Wrong: Forces Vec allocation
fn sum_items(items: Vec<i32>) -> i32 {
    items.iter().sum()
}
```

**Cow for Conditional Ownership**:

```rust
use std::borrow::Cow;

// ✅ Right: Zero-copy when no modification needed
fn normalize<'a>(input: &'a str) -> Cow<'a, str> {
    if input.is_ascii() {
        Cow::Borrowed(input)
    } else {
        Cow::Owned(input.to_lowercase())
    }
}
```

**Decision Tree**:
```
Function parameter needed?
├─ Read-only? → &T (borrow immutably)
├─ Modify temporarily? → &mut T (borrow mutably)
├─ Take ownership? → T (consume)
├─ Conditional clone? → Cow<T>
└─ Shared across threads? → Arc<T>
```

**Detection**: `rg "fn .*\(.*String\)" --type rust src/` (finds String parameters)

---

### Error Handling (🔴 Critical)

**Priority**: 🔴 **Triggers**: All error-prone operations (I/O, parsing, network)

**Result Over Panic**:

```rust
// ✅ Right: Return Result, let caller handle
fn read_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;

    serde_json::from_str(&content)
        .context("Failed to parse config JSON")
}

// ❌ Wrong: Panic in library code
fn read_config(path: &str) -> Config {
    let content = std::fs::read_to_string(path)
        .expect("Config file must exist");

    serde_json::from_str(&content).unwrap()
}
```

**anyhow vs thiserror**:

| Use Case | Library | Example |
|----------|---------|---------|
| **Applications** | `anyhow::Result<T>` | CLI tools, binaries |
| **Libraries** | `thiserror::Error` | Public crates, APIs |

```rust
// ✅ Application error handling
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = read_config("app.json")
        .context("Failed to load application config")?;
    Ok(())
}

// ✅ Library error handling
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found at {path}")]
    NotFound { path: String },

    #[error("Invalid config format: {0}")]
    ParseError(#[from] serde_json::Error),
}
```

**Context Always**:

```rust
// ✅ Right: Context on every ?
let data = fs::read("file.txt")
    .context("Failed to read file.txt")?;

// ❌ Wrong: No context
let data = fs::read("file.txt")?;
```

**Detection**:
```bash
rg "\.unwrap\(\)" --type rust src/        # Find unwraps
rg "\.expect\(" --type rust src/          # Find expects
rg "\?\s*$" --type rust src/              # Find ? without context (partial)
```

---

### Iterator Patterns (🟡 Important)

**Priority**: 🟡 **Triggers**: Loops, collections, data transformations

**Lazy Evaluation**:

```rust
// ✅ Right: Chain iterators, evaluate once
let result: Vec<_> = items.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.value())
    .collect();

// ❌ Wrong: Multiple passes over data
let mut temp = Vec::new();
for item in &items {
    if item.is_valid() {
        temp.push(item);
    }
}
let result: Vec<_> = temp.iter().map(|x| x.value()).collect();
```

**Zero-Copy Chains**:

```rust
// ✅ Right: No intermediate allocations
let total = items.iter()
    .filter(|x| x.active)
    .map(|x| x.value)
    .sum::<i32>();

// ❌ Wrong: Allocates intermediate Vec
let filtered: Vec<_> = items.iter().filter(|x| x.active).collect();
let values: Vec<_> = filtered.iter().map(|x| x.value).collect();
let total: i32 = values.iter().sum();
```

**Common Iterator Methods**:

| Method | Use Case | Example |
|--------|----------|---------|
| `filter` | Keep matching items | `.filter(\|x\| x > 10)` |
| `map` | Transform items | `.map(\|x\| x * 2)` |
| `filter_map` | Filter + transform | `.filter_map(\|x\| x.parse().ok())` |
| `fold` | Accumulate | `.fold(0, \|acc, x\| acc + x)` |
| `collect` | Build collection | `.collect::<Vec<_>>()` |
| `for_each` | Side effects | `.for_each(\|x\| println!("{}", x))` |

**Detection**: `rg "for .* in .*\.clone\(\)" --type rust` (cloning in loops)

---

### Pattern Matching (🟡 Important)

**Priority**: 🟡 **Triggers**: Enum handling, conditional logic

**Exhaustive Matching**:

```rust
enum Status {
    Active,
    Pending,
    Inactive,
}

// ✅ Right: Exhaustive match, compiler enforces
fn handle_status(status: Status) -> &'static str {
    match status {
        Status::Active => "Running",
        Status::Pending => "Waiting",
        Status::Inactive => "Stopped",
    }
}

// ❌ Wrong: Catch-all hides bugs
fn handle_status(status: Status) -> &'static str {
    match status {
        Status::Active => "Running",
        _ => "Other", // Hides Pending and Inactive
    }
}
```

**Destructuring**:

```rust
// ✅ Right: Extract fields directly
let User { id, name, .. } = user;
println!("{}: {}", id, name);

// ✅ Right: Match and extract
match result {
    Ok(User { name, .. }) => println!("Welcome {}", name),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Essential Design Patterns

### Builder Pattern (🟡 Important)

**Priority**: 🟡 **Triggers**: Complex constructors, optional fields

**Type-State Builder** (compile-time safety):

```rust
// ✅ Right: Type-state pattern prevents invalid construction
struct Unset;
struct Set<T>(T);

struct RequestBuilder<Url, Method> {
    url: Url,
    method: Method,
    headers: Vec<(String, String)>,
}

impl RequestBuilder<Unset, Unset> {
    fn new() -> Self {
        Self {
            url: Unset,
            method: Unset,
            headers: Vec::new(),
        }
    }
}

impl<M> RequestBuilder<Unset, M> {
    fn url(self, url: String) -> RequestBuilder<Set<String>, M> {
        RequestBuilder {
            url: Set(url),
            method: self.method,
            headers: self.headers,
        }
    }
}

impl<U> RequestBuilder<U, Unset> {
    fn method(self, method: String) -> RequestBuilder<U, Set<String>> {
        RequestBuilder {
            url: self.url,
            method: Set(method),
            headers: self.headers,
        }
    }
}

impl RequestBuilder<Set<String>, Set<String>> {
    fn build(self) -> Request {
        Request {
            url: self.url.0,
            method: self.method.0,
            headers: self.headers,
        }
    }
}

// Usage - won't compile without url AND method
let request = RequestBuilder::new()
    .url("https://api.example.com".to_string())
    .method("GET".to_string())
    .build();
```

**Simple Builder** (runtime validation):

```rust
// ✅ Right: Simple builder for less critical cases
#[derive(Default)]
struct ConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    timeout: Option<u64>,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn build(self) -> Result<Config, String> {
        Ok(Config {
            host: self.host.ok_or("host is required")?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        })
    }
}
```

**Detection**: `rg "impl.*Builder" --type rust` (find builder implementations)

---

### Newtype Pattern (🔴 Critical)

**Priority**: 🔴 **Triggers**: Type safety for primitives, domain modeling

**Zero-Cost Type Safety**:

```rust
// ✅ Right: Newtype prevents mixing up IDs
struct UserId(u64);
struct SessionId(u64);
struct ProjectId(String);

fn get_user(id: UserId) -> User {
    // Can't accidentally pass SessionId
}

// ❌ Wrong: Easy to mix up IDs
fn get_user(id: u64) -> User {
    // Could pass session_id by mistake
}
```

**With Traits**:

```rust
use std::fmt;

struct UserId(u64);

impl UserId {
    fn new(id: u64) -> Self {
        Self(id)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}
```

**Detection**: `rg "struct \w+\(\w+\);" --type rust` (tuple structs)

---

### RAII Pattern (🔴 Critical)

**Priority**: 🔴 **Triggers**: Resource management, cleanup guarantees

**Automatic Cleanup with Drop**:

```rust
// ✅ Right: RAII ensures cleanup
struct FileGuard {
    path: String,
}

impl FileGuard {
    fn new(path: String) -> std::io::Result<Self> {
        std::fs::write(&path, "")?;
        Ok(Self { path })
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        println!("Cleaned up {}", self.path);
    }
}

// Usage - file automatically deleted when guard dropped
{
    let _guard = FileGuard::new("temp.txt".to_string())?;
    // ... use temp file
} // _guard dropped here, file deleted
```

**Mutex Guards**:

```rust
use std::sync::Mutex;

let data = Mutex::new(vec![1, 2, 3]);

{
    let mut guard = data.lock().unwrap();
    guard.push(4);
} // MutexGuard dropped, lock released automatically

// ❌ Wrong: Manual unlock easy to forget
let mut guard = data.lock().unwrap();
guard.push(4);
// Forgot to release lock!
```

---

### Trait Objects vs Static Dispatch (🟡 Important)

**Priority**: 🟡 **Triggers**: Polymorphism, plugin systems, collections

**Decision Matrix**:

| Factor | Static Dispatch (`impl Trait`) | Dynamic Dispatch (`dyn Trait`) |
|--------|-------------------------------|--------------------------------|
| **Performance** | Zero-cost, inline | Virtual call overhead |
| **Binary Size** | Monomorphization bloat | Smaller binary |
| **Collections** | Not possible | Homogeneous `Vec<Box<dyn Trait>>` |
| **Use Case** | Known at compile-time | Runtime polymorphism |

**Static Dispatch**:

```rust
// ✅ Right: Known types at compile-time
fn process_items(iter: impl Iterator<Item = i32>) -> i32 {
    iter.sum()
}

// Generic version
fn process_items<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.sum()
}
```

**Dynamic Dispatch**:

```rust
// ✅ Right: Heterogeneous collection
trait Processor {
    fn process(&self, data: &str) -> String;
}

struct JsonProcessor;
struct XmlProcessor;

impl Processor for JsonProcessor { /* ... */ }
impl Processor for XmlProcessor { /* ... */ }

// Collection of different processors
let processors: Vec<Box<dyn Processor>> = vec![
    Box::new(JsonProcessor),
    Box::new(XmlProcessor),
];

for processor in &processors {
    println!("{}", processor.process("data"));
}
```

---

## Rust-Specific Patterns

### Sealed Traits (🟢 Recommended)

**Priority**: 🟢 **Triggers**: Library API design, preventing external implementations

**Purpose**: Prevent users from implementing trait outside your crate

```rust
// ✅ Right: Sealed trait pattern
mod sealed {
    pub trait Sealed {}
}

pub trait Operation: sealed::Sealed {
    fn execute(&self);
}

struct Add;
struct Multiply;

impl sealed::Sealed for Add {}
impl sealed::Sealed for Multiply {}

impl Operation for Add {
    fn execute(&self) { println!("Add"); }
}

impl Operation for Multiply {
    fn execute(&self) { println!("Multiply"); }
}

// Users can USE Operation trait but can't IMPLEMENT it
// because they can't implement private sealed::Sealed
```

**Use Cases**:
- Prevent breaking changes when adding trait methods
- Ensure exhaustiveness in pattern matching
- Control trait implementation in public APIs

---

### Extension Traits (🟢 Recommended)

**Priority**: 🟢 **Triggers**: Adding methods to external types

**Purpose**: Add methods to types you don't own

```rust
// ✅ Right: Extension trait
trait StringExt {
    fn truncate_words(&self, max: usize) -> String;
}

impl StringExt for str {
    fn truncate_words(&self, max: usize) -> String {
        self.split_whitespace()
            .take(max)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// Usage
let text = "Hello world from Rust";
let short = text.truncate_words(2); // "Hello world"
```

---

### PhantomData (🟢 Recommended)

**Priority**: 🟢 **Triggers**: Type-state pattern, generic markers

**Purpose**: Mark generic type parameters that aren't used in fields

```rust
use std::marker::PhantomData;

// ✅ Right: PhantomData for type-state
struct Initialized;
struct Uninitialized;

struct Database<State> {
    connection: String,
    _state: PhantomData<State>,
}

impl Database<Uninitialized> {
    fn new() -> Self {
        Self {
            connection: String::new(),
            _state: PhantomData,
        }
    }

    fn connect(self) -> Database<Initialized> {
        Database {
            connection: "connected".to_string(),
            _state: PhantomData,
        }
    }
}

impl Database<Initialized> {
    fn query(&self) -> String {
        // Only initialized DB can query
        "result".to_string()
    }
}
```

---

## Critical Anti-Patterns

### Clone Abuse (🔴 Critical)

**Priority**: 🔴 **Triggers**: Performance issues, memory bloat

**Problem**: Unnecessary cloning kills performance

```rust
// ❌ Wrong: Excessive cloning
fn process_users(users: Vec<User>) -> Vec<String> {
    users.clone()  // Clone 1 - unnecessary
        .iter()
        .filter(|u| u.active)
        .map(|u| u.name.clone())  // Clone 2 - unnecessary
        .collect()
}

// ✅ Right: Borrow and reference
fn process_users(users: &[User]) -> Vec<String> {
    users.iter()
        .filter(|u| u.active)
        .map(|u| u.name.clone())  // Clone only what's needed
        .collect()
}

// ✅ Even better: Return references
fn process_users(users: &[User]) -> Vec<&str> {
    users.iter()
        .filter(|u| u.active)
        .map(|u| u.name.as_str())  // No clone at all
        .collect()
}
```

**Common Violations**:

```rust
// ❌ Cloning in loop
for item in items.clone() {  // Unnecessary clone
    process(item);
}

// ✅ Borrow instead
for item in &items {
    process(item);
}

// ❌ Multiple clones in chain
let result = data.clone().process().clone().finalize();

// ✅ Single clone if ownership needed
let result = data.process().finalize();
```

**Detection**:
```bash
rg "\.clone\(\).*\.clone\(\)" --type rust  # Multiple clones
rg "for .* in .*\.clone\(\)" --type rust   # Cloning in loops
```

---

### Unwrap in Production (🔴 Critical)

**Priority**: 🔴 **Triggers**: Error handling, panic prevention

**Problem**: `.unwrap()` panics on errors, crashes program

```rust
// ❌ Wrong: Panic on error
fn read_config() -> Config {
    let content = std::fs::read_to_string("config.json").unwrap();
    serde_json::from_str(&content).unwrap()
}

// ✅ Right: Propagate errors
fn read_config() -> anyhow::Result<Config> {
    let content = std::fs::read_to_string("config.json")
        .context("Failed to read config.json")?;

    serde_json::from_str(&content)
        .context("Failed to parse config JSON")
}
```

**Acceptable Uses**:

```rust
// ✅ OK: In tests
#[test]
fn test_parsing() {
    let result = parse("input").unwrap();
    assert_eq!(result, expected);
}

// ✅ OK: Documented invariant
fn get_first(items: &[i32]) -> i32 {
    // SAFETY: Caller ensures items is non-empty
    items.first().unwrap()
}

// ✅ Better: Use expect with explanation
let config = load_config().expect("Config must exist for app to run");
```

**Detection**:
```bash
rg "\.unwrap\(\)" --type rust src/       # Find all unwraps
rg "\.expect\(" --type rust src/         # Find all expects
```

---

### Fighting Borrow Checker (🔴 Critical)

**Priority**: 🔴 **Triggers**: Ownership issues, lifetime errors

**Problem**: Adding clones/RefCell to bypass borrow checker

```rust
// ❌ Wrong: Clone to avoid borrow checker
struct Cache {
    data: HashMap<String, Vec<u8>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.get(key).cloned()  // Clone entire Vec!
    }
}

// ✅ Right: Return reference
impl Cache {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.data.get(key)
    }
}

// ✅ Better: Return slice
impl Cache {
    fn get(&self, key: &str) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }
}
```

**RefCell Overuse**:

```rust
// ❌ Wrong: Interior mutability when not needed
struct Stats {
    data: RefCell<HashMap<String, u64>>,
}

// ✅ Right: Redesign for clear ownership
struct Stats {
    data: HashMap<String, u64>,
}

impl Stats {
    fn update(&mut self, key: String, value: u64) {
        self.data.insert(key, value);
    }
}
```

**Detection**: Excessive `RefCell`, `Cell`, unnecessary clones

---

### Arc<Mutex> Overuse (🔴 Critical)

**Priority**: 🔴 **Triggers**: Concurrent data structures

**Problem**: Using `Arc<Mutex>` when simpler alternatives exist

```rust
// ❌ Wrong: Arc<Mutex> for read-heavy data
use std::sync::{Arc, Mutex};

let data = Arc::new(Mutex::new(vec![1, 2, 3]));

// ✅ Right: RwLock for read-heavy
use std::sync::{Arc, RwLock};

let data = Arc::new(RwLock::new(vec![1, 2, 3]));

// Read: Many threads can read concurrently
let read_guard = data.read().unwrap();
println!("{:?}", *read_guard);

// Write: Exclusive access
let mut write_guard = data.write().unwrap();
write_guard.push(4);
```

**Better Yet: parking_lot**:

```rust
// ✅ Best: parking_lot for better performance
use parking_lot::RwLock;
use std::sync::Arc;

let data = Arc::new(RwLock::new(vec![1, 2, 3]));

// No .unwrap() needed, better fairness
let read_guard = data.read();
let mut write_guard = data.write();
```

**Decision Tree**:
```
Shared mutable state needed?
├─ Single-threaded? → RefCell
├─ Read-heavy (90%+ reads)? → RwLock (parking_lot preferred)
├─ Write-heavy? → Mutex (parking_lot preferred)
└─ Lock-free needed? → crossbeam, atomic types
```

---

### Blocking in Async (🔴 Critical)

**Priority**: 🔴 **Triggers**: Async/tokio code

**Problem**: Blocking operations in async context

```rust
// ❌ Wrong: Blocking in async
async fn process() {
    std::thread::sleep(Duration::from_secs(1));  // Blocks executor!
    let data = std::fs::read("file.txt").unwrap();  // Blocks executor!
}

// ✅ Right: Async sleep
async fn process() {
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// ✅ Right: Spawn blocking for CPU/IO work
async fn process() {
    let data = tokio::task::spawn_blocking(|| {
        std::fs::read("file.txt")
    }).await.unwrap().unwrap();
}
```

**Detection**:
```bash
rg "std::thread::sleep" --type rust src/  # Find blocking sleep
rg "std::fs::" --type rust src/           # Find blocking I/O
```

---

## Quick Reference Tables

### Ownership Decision Tree

```
Need data in function?
├─ Just read it?
│  ├─ String/str? → &str (accepts both)
│  ├─ Vec? → &[T] (slice)
│  └─ Other? → &T (borrow)
├─ Modify temporarily? → &mut T
├─ Consume? → T (take ownership)
├─ Share across threads?
│  ├─ Immutable? → Arc<T>
│  ├─ Read-heavy mutable? → Arc<RwLock<T>>
│  └─ Write-heavy mutable? → Arc<Mutex<T>>
└─ Conditional clone? → Cow<'a, T>
```

### Error Handling Matrix

| Context | Type | Example |
|---------|------|---------|
| Application | `anyhow::Result<T>` | CLI, binary |
| Library | `thiserror::Error` | Public crate |
| No error info | `Option<T>` | Pure logic |
| Infallible | Return type directly | Constructor with `new()` |

### Iterator Operations

| Goal | Method | Example |
|------|--------|---------|
| Filter | `.filter()` | `.filter(\|x\| x > 10)` |
| Transform | `.map()` | `.map(\|x\| x * 2)` |
| Filter + transform | `.filter_map()` | `.filter_map(\|s\| s.parse().ok())` |
| Accumulate | `.fold()` | `.fold(0, \|sum, x\| sum + x)` |
| Find first | `.find()` | `.find(\|x\| x.is_valid())` |
| All match | `.all()` | `.all(\|x\| x > 0)` |
| Any match | `.any()` | `.any(\|x\| x < 0)` |
| Count | `.count()` | `.filter(\|x\| x.active).count()` |

### Concurrency Primitives

| Use Case | Type | Crate |
|----------|------|-------|
| Shared immutable | `Arc<T>` | std |
| Shared mutable (read-heavy) | `Arc<RwLock<T>>` | parking_lot |
| Shared mutable (write-heavy) | `Arc<Mutex<T>>` | parking_lot |
| Single-threaded mutation | `RefCell<T>` | std |
| Message passing | `mpsc::channel` | std/tokio |
| Lock-free structures | Various | crossbeam |

---

## Detection Commands

**Find Potential Issues**:

```bash
# Anti-patterns
rg "\.unwrap\(\)" --type rust src/          # Unwraps
rg "\.clone\(\).*\.clone\(\)" --type rust   # Multiple clones
rg "Arc<Mutex<" --type rust                 # Arc<Mutex> (check if RwLock better)
rg "std::thread::sleep" --type rust src/    # Blocking sleep in async?

# Good patterns
rg "impl.*Builder" --type rust              # Builder pattern
rg "struct \w+\(\w+\);" --type rust         # Newtype pattern
rg "impl Drop for" --type rust              # RAII pattern

# Project health
rg "TODO|FIXME" --type rust src/            # TODOs
rg "panic!" --type rust src/                # Explicit panics
```

---

## Integration with RULES.md

This file **complements** the Rust-Specific Rules in `RULES.md`:

- `RULES.md` → **Critical error handling & code organization rules**
- `RUST_PATTERNS.md` → **Comprehensive design patterns & idioms**

Both files use same priority system (🔴 🟡 🟢) for consistency.

---

**File**: `~/.claude/RUST_PATTERNS.md`
**Integration**: Referenced from `RULES.md` section Rust-Specific Rules
**Version**: 1.0.0
## Rust-Specific Rules

> **📚 Full Rust Patterns Reference**: See `@RUST_PATTERNS.md` for comprehensive design patterns, idioms, and anti-patterns guide.

**Priority**: 🔴/🟡 **Triggers**: Rust file operations, code review, refactoring, new Rust code

### Error Handling (🔴 Critical)

- **anyhow for Applications**: Use `anyhow::Result<T>` in binary/CLI applications
- **thiserror for Libraries**: Use `thiserror` for custom error types in libraries
- **Context Always**: Every `?` operator must have `.context("description")` or `.with_context(|| ...)`
- **No unwrap in Production**: Never use `.unwrap()` outside of tests without documented justification
- **No panic in Libraries**: Libraries must never panic, return `Result` instead
- **Actionable Errors**: Error messages must be user-friendly and actionable

✅ **Right**: `fs::read("file")?.context("Failed to read config")`
❌ **Wrong**: `fs::read("file").unwrap()` or `fs::read("file")?` (no context)

### Ownership First (🔴 Critical)

- **Prefer &str**: Use `&str` over `String` for function parameters
- **Prefer &[T]**: Use `&[T]` over `Vec<T>` for read-only slices
- **Minimize Clones**: Only clone when ownership transfer is truly needed
- **Use Cow**: Use `Cow<str>` for conditionally owned strings
- **Smart Pointers**: Use `Arc<T>` for shared ownership, `Arc<Mutex<T>>` for shared mutable state

✅ **Right**: `fn process(data: &str)` accepts both `&String` and `&str`
❌ **Wrong**: `fn process(data: String)` forces ownership transfer

### Impl Organization (🟡 Important)

- **Impl After Type**: Place impl blocks immediately after type definitions
- **Group Related**: Group constructor methods, getters, business logic in separate impl blocks
- **Standard Traits**: Always implement `Debug`, often `Clone`, `Default` where sensible
- **Trait Bounds**: Use `where` clauses for complex trait bounds

✅ **Right**: `struct User {...} impl User {...} impl Display for User {...}`
❌ **Wrong**: Type definition separated from impl by hundreds of lines

### Testing Pattern (🟡 Important)

- **Embedded Tests**: Use `#[cfg(test)] mod tests { use super::*; }`
- **Test Location**: Keep unit tests in same file, integration tests in `tests/`
- **Both Paths**: Test both success and error cases
- **No unwrap**: Use `.expect("message")` in tests for clarity on failure

✅ **Right**: `#[cfg(test)] mod tests { use super::*; #[test] fn test_x() {...} }`
❌ **Wrong**: Tests in separate files for unit tests

### Clippy Clean (🟡 Important)

- **Zero Warnings**: `cargo clippy` must pass with zero warnings before commit
- **Fix Don't Suppress**: Fix issues rather than using `#[allow(...)]` without justification
- **Run Before Commit**: Always run `cargo fmt && cargo clippy && cargo test`

✅ **Right**: Fix clippy suggestions or document why ignored
❌ **Wrong**: Ignore clippy warnings without investigation

### Async Patterns (🟡 Important)

- **No std::thread::sleep**: Use `tokio::time::sleep` in async code
- **Spawn Blocking**: Wrap blocking operations in `tokio::task::spawn_blocking`
- **Timeouts**: Always set timeouts for network operations
- **Rate Limiting**: Implement rate limiting for external API calls
- **Proper Shutdown**: Handle `SIGINT`/`SIGTERM` gracefully with cleanup

✅ **Right**: `tokio::time::sleep(duration).await`
❌ **Wrong**: `std::thread::sleep(duration)` in async context

### Lazy Static (🟢 Recommended)

- **Compiled Regex**: Use `lazy_static!` for compiled regex patterns
- **Static Collections**: Use for expensive-to-compute static data
- **Consider OnceLock**: Rust 1.70+ provides `std::sync::OnceLock` as alternative

✅ **Right**: `lazy_static! { static ref EMAIL_RE: Regex = Regex::new(...).unwrap(); }`
❌ **Wrong**: Compiling regex on every function call

### Pre-Commit Checklist

```bash
cargo fmt --all              # Format code
cargo clippy --all-targets   # Check for issues
cargo test --all-features    # Run tests
```

**Detection Commands**:
```bash
rg "\.unwrap\(\)" --type rust src/          # Find unwraps
rg "panic!" --type rust src/                # Find panics
rg "TODO|FIXME" --type rust src/            # Find TODOs
```
