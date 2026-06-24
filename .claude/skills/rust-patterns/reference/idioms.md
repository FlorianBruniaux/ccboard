# Rust Idioms Reference

Core Rust idioms that every Rust developer should know and apply.

## 1. Use Borrowed Types for Arguments

**Principle**: Accept `&str` instead of `&String`, `&[T]` instead of `&Vec<T>`

```rust
// ✅ Good: Flexible, accepts both owned and borrowed
fn process(text: &str) { }

// ❌ Bad: Forces String allocation
fn process(text: String) { }
```

**Rationale**: More flexible, works with string literals, `&String`, and `&str`.

---

## 2. Return Owned Types

**Principle**: Return `String`, not `&str` for owned data

```rust
// ✅ Good: Clear ownership
fn get_name() -> String {
    "Alice".to_string()
}

// ⚠️ Complex: Lifetime management
fn get_name<'a>() -> &'a str {
    // Harder to use correctly
}
```

---

## 3. Constructor Conventions

**Principle**: Use `new()` for constructors, `with_*()` for variants

```rust
struct Config {
    host: String,
    port: u16,
}

impl Config {
    // Main constructor
    fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    // Variant with default
    fn with_defaults() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
        }
    }
}
```

---

## 4. Use `mem::take` and `mem::replace`

**Principle**: Avoid cloning when you can move

```rust
use std::mem;

struct Task {
    data: Vec<u8>,
}

impl Task {
    fn process(&mut self) {
        // ✅ Good: Move out without cloning
        let data = mem::take(&mut self.data);
        // Process data...

        // ❌ Bad: Unnecessary clone
        let data = self.data.clone();
    }
}
```

---

## 5. Iterating by Value vs Reference

**Principle**: Know when to use `for item in &collection` vs `for item in collection`

```rust
let items = vec![1, 2, 3];

// ✅ Borrow - items still usable after
for item in &items {
    println!("{}", item);
}
println!("Items: {:?}", items);

// ✅ Move - consumes items
for item in items {
    // items no longer usable
}
```

---

## 6. Prefer `unwrap_or` Over `match`

**Principle**: Use combinator methods for cleaner code

```rust
// ✅ Good: Concise
let value = option.unwrap_or(default);

// ❌ Verbose
let value = match option {
    Some(v) => v,
    None => default,
};
```

---

## 7. Use `?` for Error Propagation

**Principle**: `?` operator for clean error handling

```rust
// ✅ Good: Clean and idiomatic
fn read_config() -> Result<Config, Error> {
    let content = fs::read_to_string("config.json")?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

// ❌ Verbose
fn read_config() -> Result<Config, Error> {
    let content = match fs::read_to_string("config.json") {
        Ok(c) => c,
        Err(e) => return Err(e.into()),
    };
    // ...
}
```

---

## 8. Iterator Chains Over Loops

**Principle**: Prefer iterator methods over manual loops

```rust
// ✅ Good: Functional, composable
let result: Vec<_> = items.iter()
    .filter(|x| x.is_valid())
    .map(|x| x.value())
    .collect();

// ❌ Imperative, more verbose
let mut result = Vec::new();
for item in &items {
    if item.is_valid() {
        result.push(item.value());
    }
}
```

---

## 9. Use `Default` Trait

**Principle**: Implement `Default` for sensible defaults

```rust
#[derive(Default)]
struct Config {
    host: String,     // defaults to ""
    port: u16,        // defaults to 0
    timeout: u64,     // defaults to 0
}

// Usage
let config = Config::default();
let config = Config { port: 8080, ..Default::default() };
```

---

## 10. RAII with Drop Trait

**Principle**: Automatic cleanup with `Drop`

```rust
struct Guard {
    resource: Resource,
}

impl Drop for Guard {
    fn drop(&mut self) {
        // Automatic cleanup
        self.resource.cleanup();
    }
}
```

---

## 11. Use `Cow` for Clone-on-Write

**Principle**: Avoid cloning unless necessary

```rust
use std::borrow::Cow;

fn process(input: &str) -> Cow<str> {
    if needs_modification(input) {
        Cow::Owned(modify(input))
    } else {
        Cow::Borrowed(input)
    }
}
```

---

## 12. Destructuring in Match

**Principle**: Extract values directly in patterns

```rust
match result {
    Ok(User { id, name, .. }) => {
        println!("User {}: {}", id, name);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## 13. Entry API for HashMap

**Principle**: Use `entry()` to avoid double lookups

```rust
// ✅ Good: Single lookup
*map.entry(key).or_insert(0) += 1;

// ❌ Bad: Two lookups
if map.contains_key(&key) {
    *map.get_mut(&key).unwrap() += 1;
} else {
    map.insert(key, 1);
}
```

---

## 14. Use `From` Instead of `Into`

**Principle**: Implement `From`, get `Into` for free

```rust
// ✅ Good: Implement From
impl From<String> for UserId {
    fn from(s: String) -> Self {
        UserId(s)
    }
}

// Auto-implemented Into
let id: UserId = "abc".to_string().into();
```

---

## 15. Closure Type Inference

**Principle**: Let compiler infer closure types

```rust
// ✅ Good: Inferred
items.iter().map(|x| x * 2).collect()

// ❌ Verbose: Explicit types rarely needed
items.iter().map(|x: &i32| -> i32 { x * 2 }).collect()
```

---

## Quick Reference

| Idiom | Instead Of | Benefit |
|-------|------------|---------|
| `&str` | `String` | Flexibility |
| `&[T]` | `&Vec<T>` | Generality |
| `?` operator | `match` | Conciseness |
| Iterator chains | `for` loops | Composability |
| `Default` trait | Custom defaults | Standard |
| `Drop` trait | Manual cleanup | Safety |
| `Cow<T>` | `.clone()` | Performance |
| `entry()` API | Double lookup | Efficiency |
| `From` trait | `Into` trait | Idiomatic |

---

**See Also**:
- `RUST_PATTERNS.md` for comprehensive patterns
- `creational.md` for object creation patterns
- `structural.md` for code organization patterns
