# Creational Patterns in Rust

Patterns for object creation and initialization.

## Builder Pattern

**Purpose**: Construct complex objects step by step with fluent API

### Simple Builder (Runtime Validation)

```rust
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

    fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    fn build(self) -> Result<Config, BuildError> {
        Ok(Config {
            host: self.host.ok_or(BuildError::MissingHost)?,
            port: self.port.unwrap_or(8080),
            timeout: self.timeout.unwrap_or(30),
        })
    }
}

// Usage
let config = ConfigBuilder::new()
    .host("localhost")
    .port(3000)
    .build()?;
```

### Type-State Builder (Compile-Time Safety)

```rust
use std::marker::PhantomData;

// State markers
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
    fn url(self, url: impl Into<String>) -> RequestBuilder<Set<String>, M> {
        RequestBuilder {
            url: Set(url.into()),
            method: self.method,
            headers: self.headers,
        }
    }
}

impl<U> RequestBuilder<U, Unset> {
    fn method(self, method: impl Into<String>) -> RequestBuilder<U, Set<String>> {
        RequestBuilder {
            url: self.url,
            method: Set(method.into()),
            headers: self.headers,
        }
    }
}

impl<U, M> RequestBuilder<U, M> {
    fn header(mut self, key: String, value: String) -> Self {
        self.headers.push((key, value));
        self
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

// Usage - won't compile without required fields
let req = RequestBuilder::new()
    .url("https://api.example.com")
    .method("GET")
    .header("Authorization".to_string(), "Bearer token".to_string())
    .build();
```

**When to Use**:
- Complex objects with many parameters
- Optional configuration
- Fluent, readable API
- Type-state when compile-time safety needed

---

## Factory Method Pattern

**Purpose**: Define interface for creating objects, let subclasses decide type

```rust
trait Parser {
    fn parse(&self, input: &str) -> Result<Value, ParseError>;
}

struct JsonParser;
struct XmlParser;
struct TomlParser;

impl Parser for JsonParser {
    fn parse(&self, input: &str) -> Result<Value, ParseError> {
        // JSON parsing
    }
}

impl Parser for XmlParser {
    fn parse(&self, input: &str) -> Result<Value, ParseError> {
        // XML parsing
    }
}

// Factory
fn create_parser(format: &str) -> Box<dyn Parser> {
    match format {
        "json" => Box::new(JsonParser),
        "xml" => Box::new(XmlParser),
        "toml" => Box::new(TomlParser),
        _ => Box::new(JsonParser), // default
    }
}

// Usage
let parser = create_parser("json");
let value = parser.parse(input)?;
```

**Rust Alternative**: Often use enums instead of trait objects

```rust
enum Parser {
    Json(JsonParser),
    Xml(XmlParser),
    Toml(TomlParser),
}

impl Parser {
    fn parse(&self, input: &str) -> Result<Value, ParseError> {
        match self {
            Parser::Json(p) => p.parse(input),
            Parser::Xml(p) => p.parse(input),
            Parser::Toml(p) => p.parse(input),
        }
    }
}
```

---

## Prototype Pattern (Clone)

**Purpose**: Create new objects by cloning existing ones

```rust
#[derive(Clone)]
struct Template {
    config: Config,
    defaults: HashMap<String, String>,
}

impl Template {
    fn customize(&self, overrides: HashMap<String, String>) -> Self {
        let mut clone = self.clone();
        clone.defaults.extend(overrides);
        clone
    }
}

// Usage
let base = Template::new(config);
let variant1 = base.customize(overrides1);
let variant2 = base.customize(overrides2);
```

**Rust Idiom**: Use `derive(Clone)` and explicit `.clone()`

---

## Singleton Pattern

**Purpose**: Ensure only one instance exists

### Using `lazy_static`

```rust
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref INSTANCE: Mutex<Database> = Mutex::new(Database::new());
}

fn get_database() -> std::sync::MutexGuard<'static, Database> {
    INSTANCE.lock().unwrap()
}
```

### Using `OnceLock` (Rust 1.70+)

```rust
use std::sync::OnceLock;

static INSTANCE: OnceLock<Database> = OnceLock::new();

fn get_database() -> &'static Database {
    INSTANCE.get_or_init(|| Database::new())
}
```

**Rust Alternatives**:
- Pass dependencies explicitly (preferred)
- Use dependency injection
- Only use global state when truly necessary

---

## Summary

| Pattern | Use Case | Rust Idiom |
|---------|----------|------------|
| Builder | Complex construction | Type-state or simple builder |
| Factory | Runtime type selection | Enum instead of trait objects |
| Prototype | Clone and customize | `derive(Clone)` |
| Singleton | Global state | `OnceLock` or explicit passing |

**See Also**:
- `idioms.md` for Rust-specific construction idioms
- `RUST_PATTERNS.md` for detailed builder examples
