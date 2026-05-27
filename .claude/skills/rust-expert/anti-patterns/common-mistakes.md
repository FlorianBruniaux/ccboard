# Common Rust Mistakes and Anti-Patterns

Learn from frequent errors to write more idiomatic Rust code.

## Error Handling Anti-Patterns

### Using .unwrap() in Production

```rust
// ❌ Bad: Panics on error
fn read_config() -> Config {
    let content = std::fs::read_to_string("config.toml").unwrap();
    toml::from_str(&content).unwrap()
}

// ✅ Good: Proper error propagation
fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("config.toml")?;
    let config = toml::from_str(&content)?;
    Ok(config)
}
```

### Swallowing Errors Silently

```rust
// ❌ Bad: Error lost
fn process_file(path: &str) {
    if let Ok(content) = std::fs::read_to_string(path) {
        println!("{}", content);
    }
    // Error case ignored!
}

// ✅ Good: Handle errors explicitly
fn process_file(path: &str) -> Result<()> {
    match std::fs::read_to_string(path) {
        Ok(content) => println!("{}", content),
        Err(e) => eprintln!("Failed to read {}: {}", path, e),
    }
    Ok(())
}
```

### Generic Error Messages

```rust
// ❌ Bad: Unhelpful error
fn load_data() -> Result<Data> {
    let file = File::open("data.json")?; // What if this fails?
    let data = serde_json::from_reader(file)?; // What if this fails?
    Ok(data)
}

// ✅ Good: Contextual errors
fn load_data() -> Result<Data> {
    let file = File::open("data.json")
        .context("Failed to open data.json")?;

    let data = serde_json::from_reader(file)
        .context("Failed to parse JSON from data.json")?;

    Ok(data)
}
```

## Ownership Anti-Patterns

### Unnecessary Cloning

```rust
// ❌ Bad: Clone when borrow suffices
fn print_twice(s: String) {
    println!("{}", s.clone());
    println!("{}", s);
}

// ✅ Good: Use reference
fn print_twice(s: &str) {
    println!("{}", s);
    println!("{}", s);
}
```

### String/&str Confusion

```rust
// ❌ Bad: Forces ownership transfer
fn greet(name: String) {
    println!("Hello, {}", name);
}

fn main() {
    let name = String::from("Alice");
    greet(name);
    // println!("{}", name); // Error: name moved
}

// ✅ Good: Accept string slice
fn greet(name: &str) {
    println!("Hello, {}", name);
}

fn main() {
    let name = String::from("Alice");
    greet(&name);
    println!("{}", name); // Works!
}
```

### Fighting the Borrow Checker

```rust
// ❌ Bad: Multiple mutable references
fn broken_example() {
    let mut data = vec![1, 2, 3];
    let first = &mut data[0];
    data.push(4); // Error: can't borrow mutably twice
    *first = 10;
}

// ✅ Good: Sequential borrows
fn fixed_example() {
    let mut data = vec![1, 2, 3];
    data.push(4); // Mutable borrow ends here
    data[0] = 10; // New mutable borrow
}
```

## Collection Anti-Patterns

### Growing Vec Without Capacity

```rust
// ❌ Bad: Multiple reallocations
fn build_large_vec() -> Vec<i32> {
    let mut vec = Vec::new();
    for i in 0..10000 {
        vec.push(i); // Reallocates multiple times
    }
    vec
}

// ✅ Good: Pre-allocate capacity
fn build_large_vec() -> Vec<i32> {
    let mut vec = Vec::with_capacity(10000);
    for i in 0..10000 {
        vec.push(i); // No reallocation
    }
    vec
}
```

### Collecting Intermediate Results

```rust
// ❌ Bad: Unnecessary collection
fn count_valid(items: &[Item]) -> usize {
    let valid: Vec<_> = items.iter().filter(|i| i.is_valid()).collect();
    valid.len()
}

// ✅ Good: Use iterator directly
fn count_valid(items: &[Item]) -> usize {
    items.iter().filter(|i| i.is_valid()).count()
}
```

### String Concatenation in Loop

```rust
// ❌ Bad: Repeated allocations
fn join_strings(items: &[String]) -> String {
    let mut result = String::new();
    for item in items {
        result = result + item + ","; // New allocation each iteration!
    }
    result
}

// ✅ Good: Push to existing string
fn join_strings(items: &[String]) -> String {
    let mut result = String::new();
    for item in items {
        result.push_str(item);
        result.push(',');
    }
    result
}

// ✅ Better: Use join
fn join_strings(items: &[String]) -> String {
    items.join(",")
}
```

## Pattern Matching Anti-Patterns

### Ignoring Match Exhaustiveness

```rust
// ❌ Bad: Catches all errors the same way
fn handle_result(result: Result<i32, MyError>) {
    match result {
        Ok(value) => println!("{}", value),
        Err(_) => println!("Error occurred"), // Lost error information!
    }
}

// ✅ Good: Match specific errors
fn handle_result(result: Result<i32, MyError>) {
    match result {
        Ok(value) => println!("{}", value),
        Err(MyError::NotFound) => println!("Item not found"),
        Err(MyError::PermissionDenied) => println!("Access denied"),
        Err(e) => println!("Other error: {}", e),
    }
}
```

### Using if let for Multiple Cases

```rust
// ❌ Bad: Nested if let
fn process_option(opt: Option<i32>) {
    if let Some(value) = opt {
        if value > 0 {
            println!("Positive");
        } else if value < 0 {
            println!("Negative");
        } else {
            println!("Zero");
        }
    } else {
        println!("None");
    }
}

// ✅ Good: Use match
fn process_option(opt: Option<i32>) {
    match opt {
        Some(value) if value > 0 => println!("Positive"),
        Some(value) if value < 0 => println!("Negative"),
        Some(0) => println!("Zero"),
        None => println!("None"),
    }
}
```

## Async Anti-Patterns

### Blocking the Runtime

```rust
// ❌ Bad: Blocks async executor
async fn bad_async() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks thread!
    fetch_data().await;
}

// ✅ Good: Use async sleep
async fn good_async() {
    tokio::time::sleep(Duration::from_secs(1)).await;
    fetch_data().await;
}
```

### Not Awaiting Futures

```rust
// ❌ Bad: Future created but not executed
async fn broken() {
    fetch_data(); // Does nothing! Returns Future
    println!("Done");
}

// ✅ Good: Await the future
async fn correct() {
    fetch_data().await;
    println!("Done");
}
```

### Sequential Instead of Concurrent

```rust
// ❌ Bad: Sequential (slow)
async fn fetch_all_sequential() -> Result<()> {
    let data1 = fetch_data_1().await?;
    let data2 = fetch_data_2().await?;
    let data3 = fetch_data_3().await?;
    Ok(())
}

// ✅ Good: Concurrent (fast)
async fn fetch_all_concurrent() -> Result<()> {
    let (data1, data2, data3) = tokio::try_join!(
        fetch_data_1(),
        fetch_data_2(),
        fetch_data_3()
    )?;
    Ok(())
}
```

## Type System Anti-Patterns

### Stringly Typed Code

```rust
// ❌ Bad: Magic strings
fn get_config(key: &str) -> String {
    match key {
        "host" => "localhost".to_string(),
        "port" => "8080".to_string(),
        _ => panic!("Unknown key"),
    }
}

// ✅ Good: Use enums
enum ConfigKey {
    Host,
    Port,
}

fn get_config(key: ConfigKey) -> String {
    match key {
        ConfigKey::Host => "localhost".to_string(),
        ConfigKey::Port => "8080".to_string(),
    }
}
```

### Boolean Parameters

```rust
// ❌ Bad: Unclear at call site
fn process_data(data: &[u8], flag: bool) {
    // What does flag mean?
}

// Call site is unclear
process_data(&data, true); // What is true?

// ✅ Good: Use enum
enum ProcessMode {
    Fast,
    Thorough,
}

fn process_data(data: &[u8], mode: ProcessMode) {
    // Clear intent
}

// Clear at call site
process_data(&data, ProcessMode::Thorough);
```

### Primitive Obsession

```rust
// ❌ Bad: Primitive types everywhere
fn create_user(id: u64, name: String, email: String) -> User {
    // Easy to mix up id and other u64s
}

// ✅ Good: Newtype pattern
struct UserId(u64);
struct Email(String);

fn create_user(id: UserId, name: String, email: Email) -> User {
    // Type safety prevents mixing up values
}
```

## Lifetime Anti-Patterns

### Unnecessarily Complex Lifetimes

```rust
// ❌ Bad: Overspecified lifetimes
fn first_word<'a, 'b>(s: &'a str, _other: &'b str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

// ✅ Good: Simplified (lifetime elision)
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

### Returning References to Local Data

```rust
// ❌ Bad: Dangling reference
fn create_string() -> &str {
    let s = String::from("hello");
    &s // Error: returns reference to local variable
}

// ✅ Good: Return owned data
fn create_string() -> String {
    String::from("hello")
}

// ✅ Good: Use static lifetime
fn create_string() -> &'static str {
    "hello" // String literal has static lifetime
}
```

## Performance Anti-Patterns

### Allocating in Hot Loops

```rust
// ❌ Bad: Allocates in every iteration
fn process_items(items: &[Item]) {
    for item in items {
        let buffer = String::new(); // New allocation!
        // process with buffer
    }
}

// ✅ Good: Reuse allocation
fn process_items(items: &[Item]) {
    let mut buffer = String::new();
    for item in items {
        buffer.clear(); // Reuse existing allocation
        // process with buffer
    }
}
```

### Unnecessary Trait Object Boxing

```rust
// ❌ Bad: Runtime dispatch when compile-time possible
fn process(items: Vec<Box<dyn Display>>) {
    for item in items {
        println!("{}", item);
    }
}

// ✅ Good: Use generics (monomorphization)
fn process<T: Display>(items: Vec<T>) {
    for item in items {
        println!("{}", item);
    }
}
```

## Testing Anti-Patterns

### Tests with Side Effects

```rust
// ❌ Bad: Tests depend on filesystem state
#[test]
fn test_file_operations() {
    std::fs::write("test.txt", "data").unwrap();
    let content = read_file("test.txt").unwrap();
    assert_eq!(content, "data");
    // File not cleaned up!
}

// ✅ Good: Clean up after test
#[test]
fn test_file_operations() {
    let temp_file = "test_temp.txt";
    std::fs::write(temp_file, "data").unwrap();
    let content = read_file(temp_file).unwrap();
    assert_eq!(content, "data");
    std::fs::remove_file(temp_file).ok(); // Cleanup
}
```

### Using unwrap() Without Context

```rust
// ❌ Bad: Unclear what failed
#[test]
fn test_parsing() {
    let result = parse("input").unwrap();
    assert_eq!(result.value, 42);
}

// ✅ Good: Clear failure message
#[test]
fn test_parsing() {
    let result = parse("input")
        .expect("Failed to parse valid input");
    assert_eq!(result.value, 42);
}
```

## Quick Detection Commands

```bash
# Find .unwrap() in code
rg "\.unwrap\(\)" --type rust src/

# Find potential panics
rg "panic!" --type rust src/

# Find TODO comments
rg "TODO|FIXME" --type rust src/

# Check for large functions (>100 lines)
# (requires custom script or IDE)
```

## Summary

### Most Common Mistakes

1. Using `.unwrap()` in production code
2. Cloning when borrowing would work
3. Passing `String` instead of `&str`
4. Not pre-allocating Vec capacity
5. Blocking in async code
6. Sequential operations that could be concurrent
7. Swallowing errors silently
8. Stringly typed code (use enums)
9. Not using iterators effectively
10. Fighting the borrow checker instead of redesigning
