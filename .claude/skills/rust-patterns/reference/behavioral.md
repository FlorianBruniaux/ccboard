# Behavioral Patterns in Rust

Patterns for communication and responsibility between objects.

## Strategy Pattern

**Purpose**: Encapsulate algorithms and make them interchangeable

### Using Trait Objects

```rust
trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
}

struct GzipCompression;
struct ZstdCompression;
struct NoCompression;

impl CompressionStrategy for GzipCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // gzip compression
    }
}

impl CompressionStrategy for ZstdCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // zstd compression
    }
}

struct Compressor {
    strategy: Box<dyn CompressionStrategy>,
}

impl Compressor {
    fn new(strategy: Box<dyn CompressionStrategy>) -> Self {
        Self { strategy }
    }

    fn compress(&self, data: &[u8]) -> Vec<u8> {
        self.strategy.compress(data)
    }
}

// Usage
let compressor = Compressor::new(Box::new(GzipCompression));
let compressed = compressor.compress(&data);
```

### Using Enums (Preferred)

```rust
enum CompressionStrategy {
    Gzip,
    Zstd,
    None,
}

impl CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Gzip => gzip_compress(data),
            Self::Zstd => zstd_compress(data),
            Self::None => data.to_vec(),
        }
    }
}

// Usage
let strategy = CompressionStrategy::Gzip;
let compressed = strategy.compress(&data);
```

---

## State Pattern

**Purpose**: Object behavior changes based on internal state

### Type-State Pattern (Compile-Time)

```rust
struct Pending;
struct Approved;
struct Rejected;

struct PullRequest<State> {
    id: u64,
    title: String,
    _state: std::marker::PhantomData<State>,
}

impl PullRequest<Pending> {
    fn new(id: u64, title: String) -> Self {
        Self {
            id,
            title,
            _state: std::marker::PhantomData,
        }
    }

    fn approve(self) -> PullRequest<Approved> {
        PullRequest {
            id: self.id,
            title: self.title,
            _state: std::marker::PhantomData,
        }
    }

    fn reject(self) -> PullRequest<Rejected> {
        PullRequest {
            id: self.id,
            title: self.title,
            _state: std::marker::PhantomData,
        }
    }
}

impl PullRequest<Approved> {
    fn merge(self) {
        println!("Merging PR #{}", self.id);
    }
}

// Rejected PRs can't be merged
impl PullRequest<Rejected> {
    fn close(self) {
        println!("Closing PR #{}", self.id);
    }
}

// Usage - compile-time guarantees
let pr = PullRequest::new(1, "Fix bug".to_string());
let approved = pr.approve();
approved.merge(); // ✅ OK

// let rejected = pr.reject();
// rejected.merge(); // ❌ Won't compile!
```

### Runtime State Pattern

```rust
trait State {
    fn handle(self: Box<Self>) -> Box<dyn State>;
}

struct Draft;
struct Published;

impl State for Draft {
    fn handle(self: Box<Self>) -> Box<dyn State> {
        println!("Publishing...");
        Box::new(Published)
    }
}

impl State for Published {
    fn handle(self: Box<Self>) -> Box<dyn State> {
        println!("Already published");
        self
    }
}

struct Document {
    state: Box<dyn State>,
}

impl Document {
    fn new() -> Self {
        Self {
            state: Box::new(Draft),
        }
    }

    fn transition(&mut self) {
        let state = std::mem::replace(&mut self.state, Box::new(Draft));
        self.state = state.handle();
    }
}
```

---

## Observer Pattern

**Purpose**: Notify multiple objects when state changes

### Using Channels (Rust Idiom)

```rust
use std::sync::mpsc;

enum Event {
    UserLoggedIn(String),
    UserLoggedOut(String),
}

struct EventBus {
    subscribers: Vec<mpsc::Sender<Event>>,
}

impl EventBus {
    fn new() -> Self {
        Self {
            subscribers: Vec::new(),
        }
    }

    fn subscribe(&mut self) -> mpsc::Receiver<Event> {
        let (tx, rx) = mpsc::channel();
        self.subscribers.push(tx);
        rx
    }

    fn notify(&self, event: Event) {
        for subscriber in &self.subscribers {
            let _ = subscriber.send(event.clone());
        }
    }
}

// Usage
let mut bus = EventBus::new();
let rx1 = bus.subscribe();
let rx2 = bus.subscribe();

bus.notify(Event::UserLoggedIn("alice".to_string()));

// Subscribers receive event
let event = rx1.recv().unwrap();
```

### Using tokio broadcast

```rust
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel(100);
let mut rx2 = tx.subscribe();

// Send event
tx.send("event").unwrap();

// Receivers get event
let event1 = rx1.recv().await.unwrap();
let event2 = rx2.recv().await.unwrap();
```

---

## Command Pattern

**Purpose**: Encapsulate requests as objects

```rust
trait Command {
    fn execute(&self);
    fn undo(&self);
}

struct InsertTextCommand {
    position: usize,
    text: String,
}

impl Command for InsertTextCommand {
    fn execute(&self) {
        println!("Inserting '{}' at {}", self.text, self.position);
    }

    fn undo(&self) {
        println!("Removing '{}' at {}", self.text, self.position);
    }
}

struct Editor {
    history: Vec<Box<dyn Command>>,
}

impl Editor {
    fn execute(&mut self, command: Box<dyn Command>) {
        command.execute();
        self.history.push(command);
    }

    fn undo(&mut self) {
        if let Some(command) = self.history.pop() {
            command.undo();
        }
    }
}
```

**Rust Alternative**: Use enums for known command types

```rust
enum Command {
    Insert { position: usize, text: String },
    Delete { position: usize, length: usize },
}

impl Command {
    fn execute(&self) {
        match self {
            Command::Insert { position, text } => {
                println!("Inserting '{}' at {}", text, position);
            }
            Command::Delete { position, length } => {
                println!("Deleting {} chars at {}", length, position);
            }
        }
    }

    fn undo(&self) {
        match self {
            Command::Insert { position, text } => {
                println!("Removing '{}' at {}", text, position);
            }
            Command::Delete { position, length } => {
                println!("Restoring {} chars at {}", length, position);
            }
        }
    }
}
```

---

## Iterator Pattern

**Purpose**: Access elements sequentially without exposing structure

```rust
struct MyCollection {
    items: Vec<i32>,
}

impl MyCollection {
    fn iter(&self) -> impl Iterator<Item = &i32> {
        self.items.iter()
    }
}

// Custom iterator
struct MyIterator<'a> {
    collection: &'a MyCollection,
    index: usize,
}

impl<'a> Iterator for MyIterator<'a> {
    type Item = &'a i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.collection.items.len() {
            let item = &self.collection.items[self.index];
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}
```

**Rust Built-in**: Use `IntoIterator` trait

```rust
impl IntoIterator for MyCollection {
    type Item = i32;
    type IntoIter = std::vec::IntoIter<i32>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// Usage
for item in collection {
    println!("{}", item);
}
```

---

## Visitor Pattern

**Purpose**: Add operations to objects without modifying them

```rust
trait Visitor {
    fn visit_file(&mut self, file: &File);
    fn visit_directory(&mut self, dir: &Directory);
}

trait Visitable {
    fn accept(&self, visitor: &mut dyn Visitor);
}

struct File {
    name: String,
    size: u64,
}

struct Directory {
    name: String,
    children: Vec<Box<dyn Visitable>>,
}

impl Visitable for File {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_file(self);
    }
}

impl Visitable for Directory {
    fn accept(&self, visitor: &mut dyn Visitor) {
        visitor.visit_directory(self);
        for child in &self.children {
            child.accept(visitor);
        }
    }
}

// Visitor implementation
struct SizeCalculator {
    total: u64,
}

impl Visitor for SizeCalculator {
    fn visit_file(&mut self, file: &File) {
        self.total += file.size;
    }

    fn visit_directory(&mut self, _dir: &Directory) {
        // Just traverse
    }
}
```

**Rust Alternative**: Use enums + match

```rust
enum Node {
    File { name: String, size: u64 },
    Directory { name: String, children: Vec<Node> },
}

fn calculate_size(node: &Node) -> u64 {
    match node {
        Node::File { size, .. } => *size,
        Node::Directory { children, .. } => {
            children.iter().map(calculate_size).sum()
        }
    }
}
```

---

## Summary

| Pattern | Use Case | Rust Approach |
|---------|----------|---------------|
| Strategy | Interchangeable algorithms | Enum + match (preferred) |
| State | State-dependent behavior | Type-state pattern |
| Observer | Event notification | Channels (mpsc, broadcast) |
| Command | Encapsulate requests | Enum for known types |
| Iterator | Sequential access | Built-in `Iterator` trait |
| Visitor | Add operations | Enum + match (preferred) |

**Key Insight**: Rust's enums + pattern matching often provide simpler, more efficient alternatives to traditional OOP patterns.

**See Also**:
- `idioms.md` for Rust-specific patterns
- `rust-specific.md` for advanced type-level patterns
