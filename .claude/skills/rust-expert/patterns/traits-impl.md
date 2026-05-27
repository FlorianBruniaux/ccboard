# Trait Implementation Patterns

Idiomatic trait design and implementation organization in Rust.

## Standard Traits

### Debug

Always implement `Debug` for custom types.

```rust
// Automatic derive
#[derive(Debug)]
struct User {
    id: u64,
    name: String,
}

// Manual implementation for custom formatting
use std::fmt;

struct Password(String);

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(***)")
    }
}
```

### Clone

```rust
#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
}

// Manual for complex cloning logic
impl Clone for Connection {
    fn clone(&self) -> Self {
        // Custom clone that creates new connection
        Self::new(&self.config)
    }
}
```

### Default

```rust
#[derive(Default)]
struct Options {
    verbose: bool,
    timeout: u32,
}

// Manual with custom defaults
impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            max_connections: 100,
        }
    }
}

// Usage
let opts = Options::default();
let config = Config { port: 3000, ..Default::default() };
```

### Display

Implement `Display` for user-facing output.

```rust
use std::fmt;

struct User {
    name: String,
    email: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

// Usage
println!("User: {}", user); // User: Alice <alice@example.com>
```

### From/Into

```rust
struct UserId(u64);

// Implement From, get Into for free
impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        UserId(id)
    }
}

// Usage
let user_id: UserId = 42.into();
let user_id = UserId::from(42);

// Multiple From implementations
impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        UserId(s.parse().unwrap_or(0))
    }
}
```

## Impl Block Organization

### Pattern: Impl Immediately After Type Definition

```rust
// ✅ Preferred: Keep related code together
struct User {
    id: u64,
    name: String,
}

impl User {
    fn new(id: u64, name: String) -> Self {
        Self { id, name }
    }

    fn display_name(&self) -> &str {
        &self.name
    }
}

// Then trait implementations
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User #{}: {}", self.id, self.name)
    }
}

impl Clone for User {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
        }
    }
}
```

### Grouping Impl Blocks

```rust
// Constructors and conversions
impl Config {
    pub fn new() -> Self { /* ... */ }
    pub fn from_file(path: &str) -> Result<Self> { /* ... */ }
}

// Getters and setters
impl Config {
    pub fn host(&self) -> &str { &self.host }
    pub fn set_host(&mut self, host: String) { self.host = host; }
}

// Business logic
impl Config {
    pub fn validate(&self) -> Result<()> { /* ... */ }
    pub fn merge(&mut self, other: Config) { /* ... */ }
}
```

## Custom Traits

### Basic Trait Definition

```rust
trait Validate {
    fn is_valid(&self) -> bool;
    fn validate(&self) -> Result<(), Vec<String>>;
}

struct Email(String);

impl Validate for Email {
    fn is_valid(&self) -> bool {
        self.0.contains('@')
    }

    fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if !self.0.contains('@') {
            errors.push("Email must contain @".to_string());
        }
        if self.0.len() < 5 {
            errors.push("Email too short".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```

### Trait with Default Implementation

```rust
trait Logger {
    fn log(&self, message: &str) {
        println!("[LOG] {}", message);
    }

    fn error(&self, message: &str) {
        eprintln!("[ERROR] {}", message);
    }
}

// Can use defaults
struct ConsoleLogger;
impl Logger for ConsoleLogger {}

// Or override
struct FileLogger {
    path: String,
}

impl Logger for FileLogger {
    fn log(&self, message: &str) {
        // Custom file logging
        std::fs::write(&self.path, message).ok();
    }
}
```

### Trait with Associated Types

```rust
trait DataStore {
    type Item;
    type Error;

    fn fetch(&self, key: &str) -> Result<Self::Item, Self::Error>;
    fn store(&mut self, key: &str, value: Self::Item) -> Result<(), Self::Error>;
}

struct MemoryStore {
    data: std::collections::HashMap<String, String>,
}

impl DataStore for MemoryStore {
    type Item = String;
    type Error = std::io::Error;

    fn fetch(&self, key: &str) -> Result<String, Self::Error> {
        self.data.get(key).cloned().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found")
        })
    }

    fn store(&mut self, key: &str, value: String) -> Result<(), Self::Error> {
        self.data.insert(key.to_string(), value);
        Ok(())
    }
}
```

## Generic Trait Bounds

### Function with Trait Bounds

```rust
use std::fmt::Display;

// Inline bounds
fn print_item<T: Display>(item: T) {
    println!("Item: {}", item);
}

// Where clause (preferred for multiple bounds)
fn compare_and_print<T>(a: T, b: T)
where
    T: Display + PartialOrd,
{
    if a > b {
        println!("{} is greater", a);
    } else {
        println!("{} is greater or equal", b);
    }
}
```

### Struct with Trait Bounds

```rust
struct Container<T>
where
    T: Clone + Default,
{
    items: Vec<T>,
}

impl<T> Container<T>
where
    T: Clone + Default,
{
    fn new() -> Self {
        Self {
            items: vec![T::default()],
        }
    }

    fn add(&mut self, item: T) {
        self.items.push(item.clone());
    }
}
```

## Trait Objects (Dynamic Dispatch)

### Basic Trait Object

```rust
trait Draw {
    fn draw(&self);
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Draw for Circle {
    fn draw(&self) {
        println!("Drawing circle with radius {}", self.radius);
    }
}

impl Draw for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle {}x{}", self.width, self.height);
    }
}

// Trait object collection
fn draw_shapes(shapes: &[Box<dyn Draw>]) {
    for shape in shapes {
        shape.draw();
    }
}

// Usage
let shapes: Vec<Box<dyn Draw>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 20.0 }),
];
draw_shapes(&shapes);
```

### Trait Object Safety

```rust
// ✅ Object-safe trait
trait Render {
    fn render(&self) -> String;
}

// ❌ Not object-safe (generic method)
trait NotObjectSafe {
    fn process<T>(&self, item: T); // Can't be trait object
}

// ✅ Make it object-safe with associated type
trait ObjectSafe {
    type Item;
    fn process(&self, item: Self::Item);
}
```

## Advanced Patterns

### Extension Trait Pattern

```rust
trait StringExt {
    fn is_numeric(&self) -> bool;
    fn truncate_to(&self, len: usize) -> String;
}

impl StringExt for str {
    fn is_numeric(&self) -> bool {
        self.chars().all(|c| c.is_numeric())
    }

    fn truncate_to(&self, len: usize) -> String {
        self.chars().take(len).collect()
    }
}

// Usage
let s = "12345";
assert!(s.is_numeric());
assert_eq!(s.truncate_to(3), "123");
```

### Sealed Trait (Prevent External Implementation)

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait MyTrait: sealed::Sealed {
    fn operation(&self);
}

struct MyType;
impl sealed::Sealed for MyType {}
impl MyTrait for MyType {
    fn operation(&self) {
        println!("Operation");
    }
}

// External crates cannot implement MyTrait
// because they can't implement sealed::Sealed
```

### Builder Pattern with Traits

```rust
trait Builder {
    type Output;
    fn build(self) -> Self::Output;
}

struct HttpRequestBuilder {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
}

impl HttpRequestBuilder {
    fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_string(),
            headers: Vec::new(),
        }
    }

    fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }

    fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}

impl Builder for HttpRequestBuilder {
    type Output = HttpRequest;

    fn build(self) -> HttpRequest {
        HttpRequest {
            url: self.url,
            method: self.method,
            headers: self.headers,
        }
    }
}
```

## Best Practices

### ✅ Do

- Place impl blocks immediately after type definition
- Group related methods in impl blocks
- Implement standard traits (`Debug`, `Clone`, `Default`)
- Use `where` clauses for complex trait bounds
- Provide default trait implementations when sensible
- Use trait objects for heterogeneous collections

### ❌ Don't

- Don't scatter impl blocks throughout the file
- Don't implement traits that don't make semantic sense
- Don't make traits object-unsafe unnecessarily
- Don't use trait objects when generics suffice (performance)
- Don't forget to derive standard traits when possible
