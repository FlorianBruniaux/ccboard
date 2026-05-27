# Memory and Ownership Pitfalls

Understanding and avoiding common memory management and ownership issues in Rust.

## Reference Lifetime Issues

### Dangling References

```rust
// ❌ Bad: Reference outlives data
fn get_value() -> &String {
    let s = String::from("hello");
    &s // Error: returns reference to local variable
}

// ✅ Good: Return owned data
fn get_value() -> String {
    String::from("hello")
}

// ✅ Good: Use string literal (static lifetime)
fn get_value() -> &'static str {
    "hello"
}
```

### Multiple Mutable References

```rust
// ❌ Bad: Attempting multiple mutable borrows
fn broken() {
    let mut vec = vec![1, 2, 3];
    let first = &mut vec[0];
    vec.push(4); // Error: already borrowed mutably
    *first = 10;
}

// ✅ Good: Use indexing after modification
fn fixed() {
    let mut vec = vec![1, 2, 3];
    vec.push(4);
    vec[0] = 10;
}

// ✅ Good: Split mutable borrows
fn split_borrow() {
    let mut data = vec![1, 2, 3, 4];
    let (left, right) = data.split_at_mut(2);
    left[0] = 10; // OK: different parts of vec
    right[0] = 20;
}
```

### Iterator Invalidation

```rust
// ❌ Bad: Modifying collection while iterating
fn broken() {
    let mut vec = vec![1, 2, 3];
    for item in &vec {
        if *item == 2 {
            vec.push(4); // Error: can't mutate while iterating
        }
    }
}

// ✅ Good: Collect indices first
fn fixed() {
    let mut vec = vec![1, 2, 3];
    let indices: Vec<_> = vec
        .iter()
        .enumerate()
        .filter(|(_, &x)| x == 2)
        .map(|(i, _)| i)
        .collect();

    for _ in indices {
        vec.push(4);
    }
}

// ✅ Good: Use retain or filter
fn better() {
    let mut vec = vec![1, 2, 3];
    vec.retain(|&x| x != 2);
}
```

## Memory Leaks

### Forgetting to Drop

```rust
// ❌ Bad: Memory leak with std::mem::forget
fn leaky() {
    let data = vec![1; 1000000];
    std::mem::forget(data); // Memory never freed!
}

// ✅ Good: Let RAII handle cleanup
fn correct() {
    let data = vec![1; 1000000];
    // Automatically dropped at end of scope
}
```

### Reference Cycles with Rc

```rust
use std::rc::Rc;
use std::cell::RefCell;

// ❌ Bad: Reference cycle causes memory leak
struct Node {
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Rc<RefCell<Node>>>,
}

fn create_cycle() {
    let node1 = Rc::new(RefCell::new(Node {
        next: None,
        prev: None,
    }));

    let node2 = Rc::new(RefCell::new(Node {
        next: Some(Rc::clone(&node1)), // node2 -> node1
        prev: None,
    }));

    node1.borrow_mut().next = Some(Rc::clone(&node2)); // node1 -> node2
    // Cycle! Neither drops when scope ends
}

// ✅ Good: Use Weak for back references
use std::rc::Weak;

struct Node {
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Weak<RefCell<Node>>>, // Weak breaks cycle
}
```

### Thread Leaks

```rust
// ❌ Bad: Spawned thread never joined
fn leak_thread() {
    std::thread::spawn(|| {
        loop {
            // Infinite loop - thread never exits
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    });
    // Thread handle dropped, but thread still running!
}

// ✅ Good: Join or use bounded channels for graceful shutdown
fn clean_thread() {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        loop {
            match rx.try_recv() {
                Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
                _ => {
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        }
    });

    // Send shutdown signal
    tx.send(()).ok();
    handle.join().ok();
}
```

## Use-After-Move Errors

### Accessing Moved Value

```rust
// ❌ Bad: Use after move
fn broken() {
    let s = String::from("hello");
    let s2 = s; // s moved to s2
    println!("{}", s); // Error: s no longer valid
}

// ✅ Good: Clone if both needed
fn with_clone() {
    let s = String::from("hello");
    let s2 = s.clone();
    println!("{} {}", s, s2); // Both valid
}

// ✅ Good: Use reference
fn with_reference() {
    let s = String::from("hello");
    let s2 = &s;
    println!("{} {}", s, s2); // Both valid
}
```

### Moved in Closure

```rust
// ❌ Bad: Move into closure prevents further use
fn broken() {
    let data = vec![1, 2, 3];
    let closure = || {
        println!("{:?}", data); // data moved into closure
    };
    closure();
    println!("{:?}", data); // Error: data moved
}

// ✅ Good: Borrow in closure
fn with_borrow() {
    let data = vec![1, 2, 3];
    let closure = || {
        println!("{:?}", &data); // Borrow, not move
    };
    closure();
    println!("{:?}", data); // OK
}

// ✅ Good: Clone before closure
fn with_clone() {
    let data = vec![1, 2, 3];
    let data_clone = data.clone();
    let closure = move || {
        println!("{:?}", data_clone);
    };
    closure();
    println!("{:?}", data); // OK, original data still valid
}
```

## Unsafe Memory Issues

### Dereferencing Raw Pointers

```rust
// ❌ Bad: Unsafe without proper checks
fn broken_unsafe() {
    let ptr: *const i32 = std::ptr::null();
    unsafe {
        println!("{}", *ptr); // Undefined behavior: null pointer dereference
    }
}

// ✅ Good: Check pointer validity
fn safe_unsafe() {
    let value = 42;
    let ptr: *const i32 = &value;

    unsafe {
        if !ptr.is_null() {
            println!("{}", *ptr);
        }
    }
}
```

### Buffer Overflows

```rust
// ❌ Bad: Unsafe buffer access without bounds check
fn broken_buffer() {
    let mut buffer = [0u8; 10];
    unsafe {
        let ptr = buffer.as_mut_ptr();
        *ptr.offset(20) = 42; // Undefined behavior: out of bounds
    }
}

// ✅ Good: Use safe abstractions
fn safe_buffer() {
    let mut buffer = [0u8; 10];
    if let Some(elem) = buffer.get_mut(9) {
        *elem = 42;
    }
}
```

## Interior Mutability Issues

### RefCell Panic at Runtime

```rust
use std::cell::RefCell;

// ❌ Bad: Multiple mutable borrows cause panic
fn broken_refcell() {
    let data = RefCell::new(vec![1, 2, 3]);
    let borrow1 = data.borrow_mut();
    let borrow2 = data.borrow_mut(); // Panic: already borrowed mutably
}

// ✅ Good: Drop borrow before creating another
fn safe_refcell() {
    let data = RefCell::new(vec![1, 2, 3]);
    {
        let mut borrow1 = data.borrow_mut();
        borrow1.push(4);
    } // borrow1 dropped here
    let borrow2 = data.borrow_mut(); // OK
}
```

### Mutex Poisoning

```rust
use std::sync::Mutex;

// ❌ Bad: Panic while holding lock poisons mutex
fn broken_mutex() {
    let data = Mutex::new(vec![1, 2, 3]);

    let result = std::panic::catch_unwind(|| {
        let mut guard = data.lock().unwrap();
        guard.push(4);
        panic!("Oops"); // Mutex now poisoned
    });

    // Next lock attempt gets poisoned error
    let guard = data.lock(); // Returns Err(PoisonError)
}

// ✅ Good: Handle poison error
fn handle_poison() {
    let data = Mutex::new(vec![1, 2, 3]);

    match data.lock() {
        Ok(guard) => println!("Lock acquired: {:?}", guard),
        Err(poisoned) => {
            eprintln!("Mutex poisoned, recovering data");
            let guard = poisoned.into_inner();
            println!("Recovered: {:?}", guard);
        }
    }
}
```

## Arc and Thread Safety

### Shared Mutable State Without Synchronization

```rust
use std::sync::Arc;
use std::thread;

// ❌ Bad: Compile error - can't mutate through Arc
fn broken_arc() {
    let data = Arc::new(vec![1, 2, 3]);
    let data_clone = Arc::clone(&data);

    thread::spawn(move || {
        data_clone.push(4); // Error: can't mutate through shared reference
    });
}

// ✅ Good: Use Arc<Mutex<T>> for shared mutable state
use std::sync::Mutex;

fn safe_arc() {
    let data = Arc::new(Mutex::new(vec![1, 2, 3]));
    let data_clone = Arc::clone(&data);

    let handle = thread::spawn(move || {
        let mut guard = data_clone.lock().unwrap();
        guard.push(4);
    });

    handle.join().unwrap();
}
```

## Memory Bloat

### Holding onto Large Allocations

```rust
// ❌ Bad: Vec retains capacity after clear
fn memory_bloat() {
    let mut buffer = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
        buffer.push(i);
    }
    buffer.clear(); // Length 0, but capacity still 1,000,000
    // Memory still allocated!
}

// ✅ Good: Shrink or drop
fn memory_efficient() {
    let mut buffer = Vec::with_capacity(1_000_000);
    for i in 0..1_000_000 {
        buffer.push(i);
    }
    buffer.clear();
    buffer.shrink_to_fit(); // Release excess memory
}
```

### String Fragmentation

```rust
// ❌ Bad: Many small allocations
fn fragmented_strings() -> Vec<String> {
    (0..10000)
        .map(|i| format!("Item {}", i)) // 10,000 separate allocations
        .collect()
}

// ✅ Good: Single buffer with references
fn efficient_strings() -> String {
    (0..10000)
        .map(|i| format!("Item {}\n", i))
        .collect() // Single String with all data
}
```

## Detection and Prevention

### Compile-Time Checks

Rust's borrow checker catches most issues at compile time:
- Use-after-move
- Multiple mutable borrows
- Dangling references
- Data races (with Send/Sync)

### Runtime Detection

```bash
# Miri: Undefined behavior detector
cargo +nightly miri test

# Valgrind: Memory leak detector (Linux)
cargo build
valgrind --leak-check=full ./target/debug/myapp

# AddressSanitizer: Memory error detector
RUSTFLAGS="-Z sanitizer=address" cargo +nightly build
./target/debug/myapp
```

### Best Practices

✅ **Do:**
- Prefer owned data over complex lifetimes
- Use `Weak` to break reference cycles
- Drop locks quickly (avoid long-held locks)
- Use safe abstractions over unsafe code
- Profile memory usage in production

❌ **Don't:**
- Use `std::mem::forget` unless absolutely necessary
- Create reference cycles with `Rc<RefCell<T>>`
- Hold locks across await points
- Use unsafe without thorough documentation
- Assume memory is freed without measuring
