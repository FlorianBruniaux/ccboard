# Structural Patterns in Rust

Patterns for organizing code and relationships between entities.

## Newtype Pattern

**Purpose**: Zero-cost type safety wrapper around primitives

```rust
struct UserId(u64);
struct SessionId(String);
struct Email(String);

impl UserId {
    fn new(id: u64) -> Self {
        Self(id)
    }

    fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

// Usage - prevents mixing IDs
fn get_user(id: UserId) -> User {
    // Can't accidentally pass SessionId
}
```

**Benefits**:
- Zero runtime cost
- Type safety
- Clear intent
- Can't mix up IDs

---

## Trait Objects (Dynamic Dispatch)

**Purpose**: Runtime polymorphism

```rust
trait Processor {
    fn process(&self, data: &str) -> String;
}

struct JsonProcessor;
struct XmlProcessor;

impl Processor for JsonProcessor { /* ... */ }
impl Processor for XmlProcessor { /* ... */ }

// Heterogeneous collection
let processors: Vec<Box<dyn Processor>> = vec![
    Box::new(JsonProcessor),
    Box::new(XmlProcessor),
];

for processor in &processors {
    let result = processor.process(data);
}
```

**Trade-offs**:
- ✅ Runtime flexibility
- ✅ Smaller binary size
- ❌ Virtual call overhead
- ❌ Can't use with generics

---

## Extension Traits

**Purpose**: Add methods to external types

```rust
trait StringExt {
    fn truncate_words(&self, max: usize) -> String;
    fn is_palindrome(&self) -> bool;
}

impl StringExt for str {
    fn truncate_words(&self, max: usize) -> String {
        self.split_whitespace()
            .take(max)
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn is_palindrome(&self) -> bool {
        let clean: String = self.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_lowercase().next().unwrap())
            .collect();
        clean == clean.chars().rev().collect::<String>()
    }
}

// Usage
let text = "Hello world from Rust";
let short = text.truncate_words(2); // "Hello world"
```

**Use Cases**:
- Add methods to standard library types
- Domain-specific extensions
- Testing utilities

---

## Adapter Pattern

**Purpose**: Make incompatible interfaces work together

```rust
// External library interface
struct ExternalLogger {
    fn log_message(&self, level: i32, msg: &str);
}

// Your interface
trait Logger {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
}

// Adapter
struct LoggerAdapter {
    external: ExternalLogger,
}

impl Logger for LoggerAdapter {
    fn info(&self, msg: &str) {
        self.external.log_message(1, msg);
    }

    fn error(&self, msg: &str) {
        self.external.log_message(3, msg);
    }
}
```

**Rust Idiom**: Often use newtype + `From`/`Into`

```rust
struct Adapter(ExternalType);

impl From<ExternalType> for Adapter {
    fn from(ext: ExternalType) -> Self {
        Self(ext)
    }
}

impl MyTrait for Adapter {
    // Implement your interface
}
```

---

## Facade Pattern

**Purpose**: Simplified interface to complex subsystem

```rust
// Complex subsystem
struct Database { /* ... */ }
struct Cache { /* ... */ }
struct Logger { /* ... */ }

// Facade
struct DataService {
    db: Database,
    cache: Cache,
    logger: Logger,
}

impl DataService {
    fn get_user(&self, id: UserId) -> Result<User> {
        self.logger.info("Fetching user");

        if let Some(user) = self.cache.get(&id) {
            return Ok(user);
        }

        let user = self.db.query_user(id)?;
        self.cache.set(id, &user);
        Ok(user)
    }
}
```

**Benefits**:
- Hides complexity
- Single entry point
- Decouples client from subsystem

---

## Delegation Pattern

**Purpose**: Forward method calls to another object

```rust
struct Inner {
    data: Vec<u8>,
}

impl Inner {
    fn process(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

struct Wrapper {
    inner: Inner,
}

impl Wrapper {
    // Delegate to inner
    fn process(&self) -> String {
        self.inner.process()
    }

    // Additional functionality
    fn process_with_prefix(&self, prefix: &str) -> String {
        format!("{}{}", prefix, self.inner.process())
    }
}
```

**Rust Alternative**: Use `Deref` trait for automatic delegation

```rust
use std::ops::Deref;

impl Deref for Wrapper {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// Now Wrapper can use Inner's methods directly
let wrapper = Wrapper { inner };
wrapper.process(); // Calls Inner::process
```

---

## Composition Over Inheritance

**Principle**: Rust doesn't have inheritance - use composition

```rust
// ❌ Can't do this in Rust
// class Manager extends Employee { }

// ✅ Composition
struct Employee {
    name: String,
    id: u64,
}

struct Manager {
    employee: Employee,  // Composition
    team: Vec<Employee>,
}

impl Manager {
    fn name(&self) -> &str {
        &self.employee.name
    }
}
```

**Alternative: Trait-based**

```rust
trait Employee {
    fn name(&self) -> &str;
    fn id(&self) -> u64;
}

struct Developer {
    name: String,
    id: u64,
    language: String,
}

struct Manager {
    name: String,
    id: u64,
    team_size: usize,
}

impl Employee for Developer {
    fn name(&self) -> &str { &self.name }
    fn id(&self) -> u64 { self.id }
}

impl Employee for Manager {
    fn name(&self) -> &str { &self.name }
    fn id(&self) -> u64 { self.id }
}
```

---

## Summary

| Pattern | Use Case | Rust Approach |
|---------|----------|---------------|
| Newtype | Type safety | Zero-cost wrapper |
| Trait Objects | Runtime polymorphism | `Box<dyn Trait>` |
| Extension Traits | Add methods | Implement trait for external type |
| Adapter | Interface compatibility | Newtype + `From` |
| Facade | Simplify complex system | Struct with methods |
| Delegation | Forward calls | `Deref` or explicit |

**See Also**:
- `idioms.md` for core patterns
- `rust-specific.md` for advanced trait patterns
