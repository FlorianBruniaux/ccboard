# Rust-Specific Patterns

Advanced patterns unique to Rust's type system and ownership model.

## Sealed Traits

**Purpose**: Prevent external implementations of a trait

```rust
mod sealed {
    pub trait Sealed {}
}

pub trait Operation: sealed::Sealed {
    fn execute(&self) -> i32;
}

struct Add;
struct Multiply;

impl sealed::Sealed for Add {}
impl sealed::Sealed for Multiply {}

impl Operation for Add {
    fn execute(&self) -> i32 { 2 + 2 }
}

impl Operation for Multiply {
    fn execute(&self) -> i32 { 2 * 2 }
}

// Users can USE Operation but can't IMPLEMENT it
// because sealed::Sealed is private
```

**Use Cases**:
- Prevent breaking changes when adding trait methods
- Ensure exhaustiveness in pattern matching
- Control trait implementation in public APIs

---

## Type-State Pattern

**Purpose**: Encode state in the type system for compile-time guarantees

```rust
use std::marker::PhantomData;

struct Locked;
struct Unlocked;

struct Door<State> {
    _state: PhantomData<State>,
}

impl Door<Locked> {
    fn new() -> Self {
        Self { _state: PhantomData }
    }

    fn unlock(self, key: &Key) -> Door<Unlocked> {
        if key.is_valid() {
            Door { _state: PhantomData }
        } else {
            panic!("Invalid key");
        }
    }
}

impl Door<Unlocked> {
    fn open(&self) {
        println!("Door opened");
    }

    fn lock(self) -> Door<Locked> {
        Door { _state: PhantomData }
    }
}

// Usage - compile-time safety
let door = Door::new(); // Locked
// door.open(); // ❌ Won't compile!

let door = door.unlock(&key); // Unlocked
door.open(); // ✅ OK
```

**Advanced**: Zero-sized Types (ZST) for states

```rust
struct Connecting;
struct Connected;
struct Disconnected;

struct Connection<State = Disconnected> {
    _state: PhantomData<State>,
    socket: TcpStream,
}

impl Connection<Disconnected> {
    fn connect(address: &str) -> Result<Connection<Connecting>> {
        // Start connecting...
    }
}

impl Connection<Connecting> {
    fn wait(self) -> Result<Connection<Connected>> {
        // Wait for connection...
    }
}

impl Connection<Connected> {
    fn send(&self, data: &[u8]) -> Result<()> {
        // Only connected can send
    }

    fn disconnect(self) -> Connection<Disconnected> {
        // Return to disconnected state
    }
}
```

---

## PhantomData Pattern

**Purpose**: Mark generic type parameters that aren't used in fields

```rust
use std::marker::PhantomData;

struct Initialized;
struct Uninitialized;

struct Database<State> {
    connection_string: String,
    _state: PhantomData<State>,
}

impl Database<Uninitialized> {
    fn new(connection_string: String) -> Self {
        Self {
            connection_string,
            _state: PhantomData,
        }
    }

    fn initialize(self) -> Database<Initialized> {
        // Perform initialization...
        Database {
            connection_string: self.connection_string,
            _state: PhantomData,
        }
    }
}

impl Database<Initialized> {
    fn query(&self, sql: &str) -> Result<Vec<Row>> {
        // Only initialized DB can query
    }
}
```

**Use Cases**:
- Type-state pattern
- Variance markers
- Generic lifetimes not stored in struct

---

## RAII (Resource Acquisition Is Initialization)

**Purpose**: Automatic resource cleanup through Drop trait

```rust
struct FileGuard {
    path: PathBuf,
}

impl FileGuard {
    fn new(path: PathBuf) -> std::io::Result<Self> {
        std::fs::write(&path, "")?;
        Ok(Self { path })
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        println!("Cleaned up {}", self.path.display());
    }
}

// Usage - automatic cleanup
{
    let _guard = FileGuard::new(PathBuf::from("temp.txt"))?;
    // Use temp file...
} // _guard dropped, file deleted automatically
```

**Common Uses**:
- Temporary files
- Lock guards (`MutexGuard`, `RwLockGuard`)
- Database transactions
- Network connections

---

## Interior Mutability

**Purpose**: Mutate data behind shared reference

### RefCell (Single-threaded)

```rust
use std::cell::RefCell;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.borrow().get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.borrow_mut().insert(key, value);
    }
}
```

### Mutex (Multi-threaded)

```rust
use std::sync::{Arc, Mutex};

struct SharedCounter {
    count: Arc<Mutex<u64>>,
}

impl SharedCounter {
    fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }
}
```

### RwLock (Read-heavy workloads)

```rust
use std::sync::{Arc, RwLock};

struct SharedCache {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl SharedCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.write().unwrap().insert(key, value);
    }
}
```

**Decision Tree**:
```
Interior mutability needed?
├─ Single-threaded? → RefCell
├─ Read-heavy (90%+ reads)? → RwLock
├─ Write-heavy? → Mutex
└─ Lock-free? → Atomic types
```

---

## Newtype Pattern (Advanced)

**Purpose**: Zero-cost type safety with additional semantics

```rust
struct UserId(u64);

impl UserId {
    fn new(id: u64) -> Result<Self, ValidationError> {
        if id == 0 {
            return Err(ValidationError::InvalidId);
        }
        Ok(Self(id))
    }
}

// Implement traits for convenience
impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "UserId({})", self.0)
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl AsRef<u64> for UserId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

// Transparent representation for FFI
#[repr(transparent)]
struct TransparentId(u64);
```

---

## Prefer Small Crates

**Principle**: Compose functionality from small, focused crates

```rust
// ✅ Good: Small, focused dependencies
[dependencies]
serde = "1.0"
serde_json = "1.0"
anyhow = "1.0"

// ❌ Avoid: Large, monolithic frameworks
// that pull in unnecessary dependencies
```

**Benefits**:
- Faster compile times
- Smaller binaries
- Easier to audit
- More flexible

---

## Zero-Cost Abstractions

**Principle**: Abstractions with no runtime overhead

```rust
// Newtype - zero cost
struct Meters(f64);

// Iterator chains - zero allocations
let sum: i32 = (1..100)
    .filter(|x| x % 2 == 0)
    .map(|x| x * 2)
    .sum();

// Const generics - compile-time
struct Array<T, const N: usize> {
    data: [T; N],
}
```

---

## Functional Error Handling

**Principle**: Use combinators for error handling

```rust
// ✅ Good: Functional style
fn process() -> Result<Value> {
    read_file("config.json")
        .context("Failed to read config")?
        .parse()
        .context("Failed to parse config")?
        .validate()
        .context("Invalid config")?
}

// ✅ Good: map_err for transformation
fn get_user(id: UserId) -> Result<User, ApiError> {
    database.query(id)
        .map_err(|e| ApiError::DatabaseError(e))?
        .ok_or(ApiError::NotFound)?
}
```

---

## Summary

| Pattern | Purpose | Use Case |
|---------|---------|----------|
| Sealed Traits | Prevent external impl | Public APIs |
| Type-State | Compile-time state | State machines |
| PhantomData | Mark unused generics | Type-level programming |
| RAII | Automatic cleanup | Resource management |
| Interior Mutability | Mutate behind & | Caching, sharing |
| Newtype | Type safety | Domain modeling |
| Zero-Cost Abstractions | No overhead | Performance |

**Key Principle**: Leverage Rust's type system for compile-time guarantees rather than runtime checks.

**See Also**:
- `RUST_PATTERNS.md` for comprehensive pattern guide
- `idioms.md` for core Rust idioms
- `structural.md` for code organization patterns
