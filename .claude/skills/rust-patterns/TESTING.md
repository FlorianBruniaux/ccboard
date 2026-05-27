# Testing Scenarios for rust-patterns Skill

Test scenarios to validate all three modes of the rust-patterns skill.

## Test Environment

**Test Codebase**: ccboard (`/Users/florianbruniaux/Sites/perso/ccboard`)

**Rationale**: Real-world Rust workspace with:
- Multiple crates (ccboard, ccboard-core, ccboard-tui, ccboard-web)
- Known patterns (Arc, RwLock, Newtype, RAII, Builder)
- Known anti-patterns (some clones, potential unwraps)
- ~10K lines of Rust code

---

## Test 1: Detection Mode - Full Codebase Scan

**Command**:
```bash
/rust-patterns detect /Users/florianbruniaux/Sites/perso/ccboard/crates
```

**Expected Patterns to Find**:

### Newtype Pattern
- `SessionId` in `ccboard-core/src/models/`
- `ProjectId`, `UserId` (if present)
- **Expected**: 3-5 instances, confidence >0.85

### RAII Pattern
- `DataStore` cleanup in guards
- File/lock guards
- **Expected**: 2-4 instances, confidence >0.90

### Builder Pattern (if present)
- Config builders
- Request builders
- **Expected**: 0-2 instances, confidence >0.80

### Arc Usage
- `Arc<DataStore>` in store.rs
- Shared state across threads
- **Expected**: 5-10 instances, confidence >0.95

### RwLock Usage
- `parking_lot::RwLock` for stats/settings
- **Expected**: 2-4 instances, confidence >0.90

**Expected Anti-Patterns to Find**:

### Excessive Clone
- `.clone()` in iteration or chains
- **Expected**: 3-8 instances, severity: medium-high
- **Location**: TUI event loops, data transformations

### Unwrap in Production
- `.unwrap()` outside test modules
- **Expected**: 0-3 instances (should be low, ccboard follows rules)
- **Severity**: critical

### Arc<Mutex> Instead of RwLock
- Check if any Arc<Mutex> for read-heavy data
- **Expected**: 0 instances (ccboard uses RwLock correctly)

**Success Criteria**:
- ✅ Finds all 4-6 Newtype instances
- ✅ Identifies Arc<RwLock> pattern in store.rs
- ✅ Detects any unwrap/clone violations
- ✅ Confidence scores >0.80 for major patterns
- ✅ JSON output is well-formed and complete

---

## Test 2: Detection Mode - Single Crate

**Command**:
```bash
/rust-patterns detect /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core
```

**Focus**: Core library patterns only

**Expected**:
- Newtype patterns in models/
- Parser patterns in parsers/
- Error handling with anyhow in store.rs
- No blocking I/O (all std::fs usage should be flagged if in async context)

**Success Criteria**:
- ✅ Correctly scopes analysis to single crate
- ✅ Doesn't include TUI/web crate patterns
- ✅ Identifies library-specific patterns

---

## Test 3: Suggestion Mode - Parser Refactoring

**Command**:
```bash
/rust-patterns suggest /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core/src/parsers
```

**Expected Suggestions**:

### Pattern: Newtype for File Paths
- **If found**: Raw `PathBuf` or `String` for paths
- **Suggest**: `struct ConfigPath(PathBuf);` for type safety

### Pattern: Builder for Complex Parsers
- **If found**: Parsers with many optional parameters
- **Suggest**: Builder pattern with fluent API

### Pattern: Error Context
- **If found**: `?` without `.context()`
- **Suggest**: Add descriptive context to all error propagations

### Pattern: Iterator Chains
- **If found**: Multiple pass processing, intermediate Vec allocations
- **Suggest**: Single iterator chain with lazy evaluation

**Success Criteria**:
- ✅ Provides 2-5 actionable suggestions
- ✅ Each suggestion has code examples
- ✅ Prioritized by impact × effort
- ✅ Markdown output is readable and well-formatted

---

## Test 4: Suggestion Mode - Async Code

**Command**:
```bash
/rust-patterns suggest /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-web/src
```

**Expected Crate-Specific Suggestions**:

### tokio Patterns
- Check for `std::thread::sleep` → suggest `tokio::time::sleep`
- Check for blocking I/O → suggest `spawn_blocking`
- Check for missing timeouts → suggest timeout wrappers

### axum Patterns
- Handler error conversion patterns
- State management with Arc

**Success Criteria**:
- ✅ Identifies tokio-specific issues
- ✅ Provides tokio idiomatic alternatives
- ✅ Distinguishes between crate ecosystems

---

## Test 5: Evaluation Mode - DataStore Pattern

**Command**:
```bash
/rust-patterns evaluate /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core/src/store.rs --pattern=arc-rwlock
```

**Target Pattern**: `Arc<RwLock<T>>` usage in DataStore

**Expected Scoring**:

### 1. Correctness (9-10/10)
- ✅ Proper Arc for shared ownership
- ✅ RwLock for read-heavy access
- ✅ Correct lock/unlock usage

### 2. Ownership Safety (9-10/10)
- ✅ No unnecessary clones
- ✅ Minimal lock duration
- ✅ Clear ownership boundaries

### 3. Error Handling (7-9/10)
- ✅ Uses anyhow::Result
- ⚠️ Some .unwrap() on locks (acceptable for Mutex/RwLock)

### 4. Performance (8-10/10)
- ✅ parking_lot::RwLock (faster than std)
- ✅ Read locks for frequent access
- ✅ Write locks only when needed

### 5. Documentation (6-8/10)
- ⚠️ Could improve doc comments
- ⚠️ Thread-safety guarantees not explicit

**Expected Overall Score**: 8.0-9.5/10

**Success Criteria**:
- ✅ Identifies correct pattern usage
- ✅ Scores each dimension with justification
- ✅ Provides specific improvement suggestions
- ✅ Code examples in recommendations

---

## Test 6: Evaluation Mode - Newtype Pattern

**Command**:
```bash
/rust-patterns evaluate /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core/src/models/stats.rs --pattern=newtype
```

**Target**: SessionId or similar newtype

**Expected Scoring**:

### 1. Correctness (8-10/10)
- Check for proper tuple struct syntax
- Check for trait implementations (Display, From, etc.)

### 2. Ownership Safety (9-10/10)
- Zero-cost abstraction
- No hidden allocations

### 3. Error Handling (Variable)
- Depends on validation presence

### 4. Performance (10/10)
- Zero-cost newtype

### 5. Documentation (Variable)
- Check doc comments quality

**Expected Overall Score**: 7.5-9.0/10

**Success Criteria**:
- ✅ Correctly analyzes newtype implementation
- ✅ Identifies zero-cost nature
- ✅ Suggests documentation/trait improvements

---

## Test 7: Edge Cases

### Empty Directory
**Command**: `/rust-patterns detect /tmp/empty`
**Expected**: Graceful handling, "No Rust files found" message

### Non-Rust Directory
**Command**: `/rust-patterns detect /Users/florianbruniaux/.claude/plans`
**Expected**: "No Rust files found" or skips non-.rs files

### Invalid Pattern Name
**Command**: `/rust-patterns evaluate src/store.rs --pattern=invalid`
**Expected**: Error message listing valid pattern names

### Mixed Codebase (Rust + Other)
**Command**: `/rust-patterns detect .`
**Expected**: Only analyzes .rs files, ignores others

---

## Integration Test: Full Workflow

**Scenario**: Code review workflow

```bash
# 1. Detect patterns in PR diff files
/rust-patterns detect crates/ccboard-core/src/store.rs

# 2. If anti-patterns found, get suggestions
/rust-patterns suggest crates/ccboard-core/src/store.rs

# 3. Evaluate critical patterns before merge
/rust-patterns evaluate crates/ccboard-core/src/store.rs --pattern=arc-rwlock
```

**Success Criteria**:
- ✅ All three modes work sequentially
- ✅ Output from one mode informs next
- ✅ Provides actionable PR review feedback

---

## Performance Tests

### Large Codebase
**Target**: Entire ccboard workspace (~10K lines)
**Command**: `/rust-patterns detect /Users/florianbruniaux/Sites/perso/ccboard`

**Expectations**:
- Scan completes in <30 seconds
- Memory usage <500MB
- No crashes or hangs

### Deep Directory Tree
**Target**: Nested crate structure
**Command**: `/rust-patterns detect /Users/florianbruniaux/Sites/perso/ccboard/crates`

**Expectations**:
- Recursively finds all .rs files
- Handles symlinks gracefully
- Reports progress for long scans

---

## Output Quality Tests

### JSON Format Validation
**Test**: Parse detection output as JSON
**Expected**: Valid JSON, all required fields present

### Markdown Readability
**Test**: Render suggestion/evaluation output
**Expected**: Proper headers, code blocks, formatting

### Error Messages
**Test**: Trigger various error conditions
**Expected**: Clear, actionable error messages

---

## Regression Tests

After each skill update, run:

```bash
# Full detection on ccboard
/rust-patterns detect /Users/florianbruniaux/Sites/perso/ccboard/crates

# Suggestion on parsers (known code smells)
/rust-patterns suggest /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core/src/parsers

# Evaluation on store (known good pattern)
/rust-patterns evaluate /Users/florianbruniaux/Sites/perso/ccboard/crates/ccboard-core/src/store.rs --pattern=arc-rwlock
```

**Compare**: Results should be consistent across versions unless intentional changes.

---

## Success Metrics

**Detection Mode**:
- ✅ >90% accuracy on known patterns
- ✅ <5% false positives
- ✅ Confidence scores correlate with correctness

**Suggestion Mode**:
- ✅ >80% of suggestions are actionable
- ✅ Priorities align with actual impact
- ✅ Code examples compile and work

**Evaluation Mode**:
- ✅ Scores align with manual code review
- ✅ Improvement suggestions are specific
- ✅ Overall scores within ±1 point of expert assessment

---

**Test Coverage**: All 3 modes × 7 test scenarios = 21 test cases minimum
