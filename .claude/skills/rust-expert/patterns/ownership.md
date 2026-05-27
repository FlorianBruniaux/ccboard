# Ownership & Borrowing Patterns

Master Rust's ownership system with idiomatic patterns for strings, slices, lifetimes, and smart pointers.

## String Types: &str vs String

### Rule of Thumb

- **Input parameters**: Use `&str` (borrowed string slice)
- **Owned data**: Use `String` (heap-allocated, growable)
- **Return values**: Use `String` if creating new data, `&str` if returning reference

### Function Parameters

```rust
// ✅ Preferred: Accept string slice
fn print_greeting(name: &str) {
    println!("Hello, {}", name);
}

// Flexible calling
print_greeting("Alice");           // &str literal
print_greeting(&owned_string);     // String reference
print_greeting(&owned_string[..5]); // Slice of String

// ❌ Avoid: Forces caller to own String
fn print_greeting_bad(name: String) {
    println!("Hello, {}", name);
}
```

### Building Strings

```rust
// Efficient string building
let mut message = String::new();
message.push_str("Hello");
message.push(' ');
message.push_str("world");

// Or use format! for readability
let message = format!("{} {}", "Hello", "world");

// Preallocate capacity if size known
let mut buffer = String::with_capacity(100);
```

### String Conversion

```rust
// &str to String
let owned: String = "hello".to_string();
let owned: String = "hello".to_owned();
let owned: String = String::from("hello");

// String to &str (automatic with Deref)
let s = String::from("hello");
let slice: &str = &s;
let slice: &str = s.as_str();
```

## Cow: Clone-on-Write

Use `Cow<str>` when you might need to modify but usually don't.

### Basic Pattern

```rust
use std::borrow::Cow;

fn normalize_path(path: &str) -> Cow<str> {
    if path.contains('\\') {
        // Need to modify: return Owned
        Cow::Owned(path.replace('\\', "/"))
    } else {
        // No modification: return Borrowed
        Cow::Borrowed(path)
    }
}

// Efficient: no allocation if already normalized
let unix_path = normalize_path("/home/user");
let win_path = normalize_path("C:\\Users\\user");
```

### API Design with Cow

```rust
use std::borrow::Cow;

struct Config {
    database_url: Cow<'static, str>,
}

impl Config {
    fn from_env() -> Self {
        Self {
            // Runtime string: Owned
            database_url: Cow::Owned(std::env::var("DATABASE_URL").unwrap()),
        }
    }

    fn default() -> Self {
        Self {
            // Compile-time string: Borrowed
            database_url: Cow::Borrowed("sqlite::memory:"),
        }
    }
}
```

## Slices vs Vectors

### Function Parameters

```rust
// ✅ Preferred: Accept slice for reading
fn sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

// Flexible calling
sum(&vec![1, 2, 3]);      // Vec reference
sum(&[1, 2, 3]);          // Array reference
sum(&numbers[1..5]);      // Sub-slice

// ❌ Avoid: Unnecessarily restrictive
fn sum_bad(numbers: &Vec<i32>) -> i32 {
    numbers.iter().sum()
}
```

### Returning Slices

```rust
// Return slice into owned data
fn get_first_three(data: &[i32]) -> &[i32] {
    &data[..3.min(data.len())]
}

// Return owned Vec when creating new data
fn doubled(data: &[i32]) -> Vec<i32> {
    data.iter().map(|x| x * 2).collect()
}
```

## Lifetimes

### Basic Lifetime Annotations

```rust
// Lifetime ties output to input
fn first_word(s: &str) -> &str {
    s.split_whitespace()
        .next()
        .unwrap_or("")
}

// Multiple inputs: explicit lifetimes
fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

### Struct Lifetimes

```rust
// Struct holding references needs lifetime
struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Parser { input, position: 0 }
    }

    fn current(&self) -> &'a str {
        &self.input[self.position..]
    }
}
```

### Static Lifetime

```rust
// String literals are 'static
const DEFAULT_NAME: &'static str = "guest";
const DEFAULT_NAME: &str = "guest"; // 'static is implicit

// Owned data outlives function scope
fn leak_string() -> &'static str {
    Box::leak(String::from("leaked").into_boxed_str())
}
```

## Smart Pointers

### Box: Heap Allocation

```rust
// Large data on heap
let large_data = Box::new([0u8; 10_000]);

// Recursive types
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

### Rc: Reference Counting (Single-threaded)

```rust
use std::rc::Rc;

let shared = Rc::new(vec![1, 2, 3]);
let reference1 = Rc::clone(&shared);
let reference2 = Rc::clone(&shared);

println!("References: {}", Rc::strong_count(&shared)); // 3
```

### Arc: Atomic Reference Counting (Thread-safe)

```rust
use std::sync::Arc;
use std::thread;

let shared_data = Arc::new(vec![1, 2, 3, 4, 5]);

let handles: Vec<_> = (0..5)
    .map(|i| {
        let data = Arc::clone(&shared_data);
        thread::spawn(move || {
            println!("Thread {}: {:?}", i, data);
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

### Arc<Mutex<T>>: Shared Mutable State

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0));
let mut handles = vec![];

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    let handle = thread::spawn(move || {
        let mut num = counter.lock().unwrap();
        *num += 1;
    });
    handles.push(handle);
}

for handle in handles {
    handle.join().unwrap();
}

println!("Result: {}", *counter.lock().unwrap()); // 10
```

### RefCell: Interior Mutability (Single-threaded)

```rust
use std::cell::RefCell;

struct Config {
    cache: RefCell<HashMap<String, String>>,
}

impl Config {
    fn get_cached(&self, key: &str) -> Option<String> {
        // Immutable self, but can mutate cache
        let mut cache = self.cache.borrow_mut();
        cache.get(key).cloned()
    }
}
```

## Common Patterns

### Builder Pattern with Owned Strings

```rust
#[derive(Default)]
struct HttpRequest {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
}

impl HttpRequest {
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

// Usage
let req = HttpRequest::new("https://example.com")
    .method("POST")
    .header("Content-Type", "application/json");
```

### Avoiding Clones with References

```rust
// ❌ Unnecessary clone
fn process_data(data: Vec<String>) -> Vec<String> {
    data.iter()
        .map(|s| s.to_uppercase())
        .collect()
}

// ✅ Use references
fn process_data(data: &[String]) -> Vec<String> {
    data.iter()
        .map(|s| s.to_uppercase())
        .collect()
}
```

### Zero-Copy Parsing

```rust
fn parse_command(input: &str) -> (&str, &[&str]) {
    let mut parts = input.split_whitespace();
    let command = parts.next().unwrap_or("");
    let args: Vec<&str> = parts.collect();
    (command, &args)
}

// No allocations: all references into original string
let input = "git commit -m message";
let (cmd, args) = parse_command(input);
```

## Best Practices

### ✅ Do

- Use `&str` for read-only string parameters
- Use `&[T]` for read-only slice parameters
- Use `Cow<str>` when you might need to modify
- Use `Arc<T>` for shared ownership across threads
- Use `Arc<Mutex<T>>` for shared mutable state

### ❌ Don't

- Don't pass `String` when `&str` suffices
- Don't pass `Vec<T>` when `&[T]` suffices
- Don't clone unnecessarily (use references)
- Don't use `Rc` in multi-threaded code (use `Arc`)
- Don't fight the borrow checker (redesign if needed)

## RTK Project Examples

### Command Line Parsing (Zero-Copy)

```rust
fn parse_git_args(args: &[String]) -> (&str, &[String]) {
    if args.is_empty() {
        ("status", &[])
    } else {
        (&args[0], &args[1..])
    }
}
```

### Efficient String Building

```rust
fn format_output(items: &[String]) -> String {
    let mut output = String::with_capacity(items.len() * 50);
    for item in items {
        output.push_str(item);
        output.push('\n');
    }
    output
}
```
