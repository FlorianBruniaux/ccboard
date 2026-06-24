---
name: rust-patterns
description: |
  Analyze Rust codebase for design patterns, idioms, and anti-patterns.
  Provides detection, suggestion, and evaluation modes for Rust code quality.
  Keywords: rust, patterns, analysis, idioms, anti-patterns, code-quality
effort: medium
allowed-tools: Read, Grep, Glob, Bash
---

# rust-patterns Skill

Comprehensive Rust pattern analysis tool for detecting, suggesting, and evaluating design patterns in Rust codebases.

## Overview

This skill provides three complementary modes for Rust pattern analysis:

1. **Detection Mode**: Scan codebase to identify existing patterns and anti-patterns
2. **Suggestion Mode**: Recommend patterns to address code smells and improve design
3. **Evaluation Mode**: Score pattern implementation quality against best practices

**Complementary to**: `@RUST_PATTERNS.md` (always-active rules) - this skill provides optional deep analysis.

## Invocation

```bash
# Detection - scan codebase for patterns
/rust-patterns detect [path]

# Suggestion - recommend patterns for code smells
/rust-patterns suggest [path]

# Evaluation - score pattern implementation quality
/rust-patterns evaluate [path] [--pattern=<name>]

# Help
/rust-patterns --help
```

## Mode 1: Detection

**Purpose**: Identify patterns and anti-patterns currently in the codebase.

**Workflow**:
```
1. Scan → Glob for *.rs files in target path
2. Search → Grep for pattern signatures (ownership, newtype, builder, etc.)
3. Detect Anti-patterns → Grep for .clone(), .unwrap(), Arc<Mutex>, blocking ops
4. Classify → Categorize as idiom/pattern/anti-pattern
5. Score Confidence → 0.0-1.0 based on signature strength
6. Report → JSON output with file locations and recommendations
```

**Example**:
```bash
/rust-patterns detect src/
```

**Output Format**:
```json
{
  "scan_summary": {
    "files_scanned": 42,
    "patterns_found": 15,
    "anti_patterns_found": 8,
    "confidence_avg": 0.87
  },
  "patterns": {
    "newtype": [
      {
        "file": "src/models/stats.rs",
        "lines": "12-15",
        "confidence": 0.92,
        "code": "struct SessionId(String);",
        "note": "✅ Good: Zero-cost type safety"
      }
    ],
    "builder": [
      {
        "file": "src/store.rs",
        "lines": "45-78",
        "confidence": 0.88,
        "code": "impl DataStoreBuilder { ... }",
        "note": "✅ Good: Type-state builder pattern"
      }
    ],
    "raii": [
      {
        "file": "src/guard.rs",
        "lines": "23-35",
        "confidence": 0.95,
        "code": "impl Drop for FileGuard { ... }",
        "note": "✅ Good: Automatic cleanup"
      }
    ]
  },
  "anti_patterns": {
    "excessive_clone": [
      {
        "file": "src/app.rs",
        "lines": "45",
        "severity": "high",
        "code": ".clone().process().clone()",
        "suggestion": "Use &DataStore instead of cloning",
        "impact": "Performance: Multiple heap allocations"
      }
    ],
    "unwrap_in_production": [
      {
        "file": "src/parsers/session.rs",
        "lines": "89",
        "severity": "critical",
        "code": "serde_json::from_str(&content).unwrap()",
        "suggestion": "Use ? operator with .context()",
        "impact": "Safety: Will panic on malformed input"
      }
    ],
    "blocking_in_async": [
      {
        "file": "src/api/handlers.rs",
        "lines": "123",
        "severity": "high",
        "code": "std::fs::read(path)",
        "suggestion": "Use tokio::fs::read() or spawn_blocking",
        "impact": "Performance: Blocks async executor"
      }
    ]
  },
  "recommendations": [
    "Fix 3 critical anti-patterns before merge",
    "Consider extracting builder pattern in src/config.rs",
    "Review clone usage in src/app.rs (8 occurrences)"
  ]
}
```

**Detection Signatures**: See `signatures/detection-rules.yaml`

---

## Mode 2: Suggestion

**Purpose**: Recommend specific patterns to address identified code smells.

**Workflow**:
```
1. Code Smell Detection → Identify problems (long functions, switch on type, global state)
2. Pattern Matching → Map smells to applicable Rust patterns
3. Crate-Specific Analysis → tokio/serde/anyhow-specific suggestions
4. Priority Ranking → Score by (impact × feasibility)
5. Report → Markdown with examples and implementation steps
```

**Example**:
```bash
/rust-patterns suggest src/parsers/
```

**Output Format**:
```markdown
# Pattern Suggestions for src/parsers/

## High Priority

### 1. Newtype Pattern for SessionId (Impact: High, Effort: Low)

**Location**: `src/parsers/session.rs:45-89`

**Problem**: Using raw `String` for session IDs allows mixing with other strings

**Current Code**:
```rust
fn get_session(id: String) -> Option<Session> {
    sessions.get(&id)
}
```

**Suggested Pattern**: Newtype

**Refactored Code**:
```rust
struct SessionId(String);

impl SessionId {
    fn new(id: String) -> Self {
        Self(id)
    }
}

fn get_session(id: SessionId) -> Option<Session> {
    sessions.get(&id.0)
}
```

**Benefits**:
- ✅ Type safety: Can't accidentally pass ProjectId
- ✅ Zero-cost: No runtime overhead
- ✅ Self-documenting: Clear what type of ID is expected

---

### 2. Builder Pattern for Config (Impact: High, Effort: Medium)

**Location**: `src/config.rs:12-45`

**Problem**: Constructor with 8 parameters, many optional

**Current Code**:
```rust
Config::new(host, port, timeout, retries, ssl, verbose, log_level, buffer_size)
```

**Suggested Pattern**: Type-State Builder

**Implementation Steps**:
1. Create `ConfigBuilder` struct with `Option<T>` fields
2. Add fluent setter methods returning `Self`
3. Implement `build()` with validation
4. Consider type-state pattern for required fields

**Benefits**:
- ✅ Readable: Clear what each value is
- ✅ Flexible: Optional params with defaults
- ✅ Safe: Compile-time validation with type-state

---

## Medium Priority

### 3. RAII for Resource Cleanup (Impact: Medium, Effort: Low)

**Location**: `src/temp_file.rs:23-56`

**Problem**: Manual cleanup with `defer!` macro, easy to forget

**Suggested Pattern**: RAII with Drop trait

**Benefits**:
- ✅ Automatic: Cleanup guaranteed on scope exit
- ✅ Exception-safe: Works even on panic
- ✅ Idiomatic: Standard Rust pattern

---

## Crate-Specific Suggestions

### tokio Patterns

**Location**: `src/api/server.rs`

**Observations**:
- ❌ Using `std::thread::sleep` in async context (line 89)
- ❌ Blocking I/O with `std::fs::read` (line 123)

**Suggestions**:
1. Replace `std::thread::sleep` → `tokio::time::sleep`
2. Replace `std::fs::read` → `tokio::fs::read` or `spawn_blocking`
3. Add timeouts to all network operations
4. Consider using `tokio::select!` for concurrent operations

### anyhow/thiserror Patterns

**Location**: `src/parsers/`

**Observations**:
- ⚠️ Using `anyhow` in library code (should use `thiserror`)
- ❌ Missing `.context()` on 12 `?` operators

**Suggestions**:
1. Create custom error types with `thiserror::Error` for lib
2. Add `.context()` to all `?` operators for better error messages
3. Consider error chaining for root cause tracking

---

## Summary

**Total Suggestions**: 8
- High Priority: 2
- Medium Priority: 3
- Low Priority: 3

**Estimated Effort**: 4-6 hours for high priority items

**Impact**: Improved type safety, better error handling, cleaner APIs
```

**Suggestion Rules**: See `signatures/anti-patterns.yaml`

---

## Mode 3: Evaluation

**Purpose**: Score the quality of pattern implementations against best practices.

**Workflow**:
```
1. Pattern Identification → Locate target pattern in code
2. Criteria Assessment → Score against 5 quality dimensions
3. Best Practice Check → Compare to reference implementations
4. Report → Detailed scoring with improvement recommendations
```

**Example**:
```bash
/rust-patterns evaluate src/store.rs --pattern=newtype
```

**Scoring Criteria** (0-10 each):

1. **Correctness** (0-10)
   - Pattern structure matches Rust idiom?
   - Proper trait implementations?
   - Follows language conventions?

2. **Ownership Safety** (0-10)
   - Proper borrowing patterns?
   - No unnecessary clones?
   - Optimal reference usage?

3. **Error Handling** (0-10)
   - Uses `Result<T, E>` appropriately?
   - All errors have context?
   - No unwrap/panic in library code?

4. **Performance** (0-10)
   - Zero-copy where possible?
   - No allocations in hot paths?
   - Optimal algorithm choices?

5. **Documentation** (0-10)
   - Clear intent and usage?
   - Lifetimes explained?
   - Examples provided?

**Output Format**:
```markdown
# Pattern Evaluation: Newtype (SessionId)

**File**: `src/models/stats.rs`
**Lines**: 12-25
**Overall Score**: 8.4/10 ⭐⭐⭐⭐

---

## Scoring Breakdown

### 1. Correctness: 9/10 ⭐⭐⭐⭐⭐

✅ **Strengths**:
- Proper tuple struct syntax
- Implements `From<String>` for conversion
- Implements `Display` for formatting

⚠️ **Areas for Improvement**:
- Missing `AsRef<str>` implementation for string access

**Recommendation**: Add `impl AsRef<str> for SessionId`

---

### 2. Ownership Safety: 10/10 ⭐⭐⭐⭐⭐

✅ **Strengths**:
- Zero-cost abstraction (no runtime overhead)
- No unnecessary clones
- Properly borrows in methods

**Excellent**: Optimal ownership patterns

---

### 3. Error Handling: 7/10 ⭐⭐⭐⭐

✅ **Strengths**:
- Returns `Result` from validation

⚠️ **Areas for Improvement**:
- Error messages could be more specific
- Consider using custom error type

**Recommendation**: Add validation with descriptive errors

---

### 4. Performance: 9/10 ⭐⭐⭐⭐⭐

✅ **Strengths**:
- Zero-cost newtype pattern
- No hidden allocations
- Efficient implementation

⚠️ **Minor Note**:
- Could add `#[repr(transparent)]` for guaranteed ABI compatibility

---

### 5. Documentation: 7/10 ⭐⭐⭐⭐

✅ **Strengths**:
- Basic doc comments present
- Usage examples in tests

⚠️ **Areas for Improvement**:
- Missing module-level documentation
- No examples in doc comments
- Type purpose not explained

**Recommendation**: Add comprehensive doc comments with examples

---

## Overall Assessment

**Verdict**: ⭐⭐⭐⭐ Very Good Implementation

This newtype implementation is solid and follows Rust best practices. It provides zero-cost type safety and has proper trait implementations. With minor documentation improvements and additional trait implementations, it would be excellent.

**Priority Improvements**:
1. Add comprehensive documentation with examples
2. Implement `AsRef<str>` for easier string access
3. Consider adding validation in constructor

**Code Example** (Improved):
```rust
/// Unique identifier for a session.
///
/// # Examples
///
/// ```
/// let id = SessionId::new("abc-123".to_string())?;
/// println!("Session: {}", id);
/// ```
#[repr(transparent)]
struct SessionId(String);

impl SessionId {
    /// Creates a new SessionId after validation.
    pub fn new(id: String) -> Result<Self, ValidationError> {
        if id.is_empty() {
            return Err(ValidationError::EmptyId);
        }
        Ok(Self(id))
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
```

---

## Summary

**Overall Score**: 8.4/10 (Very Good)
**Strengths**: Correctness, ownership safety, performance
**Improvement Areas**: Documentation, additional trait implementations
**Effort to Improve**: Low (1-2 hours)
```

**Evaluation Criteria**: See `checklists/pattern-evaluation.md`

---

## Integration with RUST_PATTERNS.md

This skill **complements** the always-active `@RUST_PATTERNS.md` rules:

| Component | Purpose | When to Use |
|-----------|---------|-------------|
| **RUST_PATTERNS.md** | Always-active rules and guidelines | Every Rust development session |
| **rust-patterns skill** | Optional deep analysis | Code review, refactoring, pattern audits |

**Workflow**:
1. `@RUST_PATTERNS.md` guides daily development (always active)
2. `/rust-patterns detect` for codebase health check
3. `/rust-patterns suggest` when refactoring or improving code
4. `/rust-patterns evaluate` before PR or major release

---

## Reference Files

- `reference/idioms.md` - Core Rust idioms reference
- `reference/creational.md` - Creational patterns (Builder, Factory)
- `reference/structural.md` - Structural patterns (Newtype, Trait Objects)
- `reference/behavioral.md` - Behavioral patterns (Strategy, Observer)
- `reference/rust-specific.md` - Rust-specific patterns (Sealed Traits, PhantomData)

## Detection Signatures

- `signatures/detection-rules.yaml` - Pattern detection rules
- `signatures/anti-patterns.yaml` - Anti-pattern detection and mapping
- `signatures/crate-patterns.yaml` - Crate-specific idioms (tokio, serde, anyhow)

## Checklists

- `checklists/pattern-evaluation.md` - Quality scoring criteria

---

**Version**: 1.0.0
**Maintained by**: SuperClaude Framework
