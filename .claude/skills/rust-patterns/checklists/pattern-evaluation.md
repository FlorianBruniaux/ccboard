# Pattern Evaluation Checklist

Scoring criteria for evaluating Rust pattern implementations (0-10 scale).

## 1. Correctness (0-10)

**Question**: Does the pattern implementation follow Rust idioms correctly?

### Scoring Guide

**9-10 points**: Excellent
- ✅ Pattern structure matches Rust idiom exactly
- ✅ All trait implementations are correct
- ✅ Follows language conventions precisely
- ✅ No logical errors or edge cases missed

**7-8 points**: Good
- ✅ Pattern mostly correct
- ⚠️ Minor deviation from idiom
- ⚠️ One or two trait implementations could be improved

**5-6 points**: Acceptable
- ✅ Basic pattern structure correct
- ⚠️ Several deviations from idiom
- ⚠️ Missing some standard trait implementations

**3-4 points**: Needs Improvement
- ⚠️ Pattern structure has issues
- ❌ Incorrect trait implementations
- ❌ Misses the point of the pattern

**0-2 points**: Poor
- ❌ Pattern implemented incorrectly
- ❌ Fundamental misunderstanding
- ❌ Does not follow Rust conventions

---

## 2. Ownership Safety (0-10)

**Question**: Does the implementation use optimal ownership patterns?

### Scoring Guide

**9-10 points**: Excellent
- ✅ Proper borrowing everywhere
- ✅ No unnecessary clones
- ✅ Optimal reference usage (&str, &[T], etc.)
- ✅ Clear ownership boundaries
- ✅ Zero-cost abstractions

**7-8 points**: Good
- ✅ Mostly good borrowing
- ⚠️ A few unnecessary clones
- ⚠️ Could use more references

**5-6 points**: Acceptable
- ✅ Basic ownership correct
- ⚠️ Several unnecessary clones
- ⚠️ Some owned types where refs would work

**3-4 points**: Needs Improvement
- ⚠️ Many unnecessary clones
- ❌ Fights borrow checker with clones
- ❌ Poor ownership design

**0-2 points**: Poor
- ❌ Excessive cloning everywhere
- ❌ Fundamental ownership issues
- ❌ Performance problems from allocations

### Checklist
- [ ] Function parameters use &T instead of T where possible
- [ ] Uses &str instead of String for parameters
- [ ] Uses &[T] instead of Vec<T> for parameters
- [ ] Minimal use of .clone()
- [ ] No unnecessary ownership transfers
- [ ] Proper use of Cow<> for conditional cloning

---

## 3. Error Handling (0-10)

**Question**: Are errors handled properly with context and propagation?

### Scoring Guide

**9-10 points**: Excellent
- ✅ Uses Result<T, E> appropriately
- ✅ All errors have descriptive context
- ✅ No unwrap/panic in production code
- ✅ Proper error propagation with ?
- ✅ Custom error types where appropriate

**7-8 points**: Good
- ✅ Mostly good error handling
- ⚠️ Missing context in a few places
- ⚠️ One or two unwraps (with justification)

**5-6 points**: Acceptable
- ✅ Basic Result usage
- ⚠️ Many errors lack context
- ⚠️ Several unwraps

**3-4 points**: Needs Improvement
- ⚠️ Inconsistent error handling
- ❌ Many unwraps without justification
- ❌ Poor error messages

**0-2 points**: Poor
- ❌ Unwrap/panic everywhere
- ❌ No error context
- ❌ Errors swallowed or ignored

### Checklist
- [ ] All ? operators have .context()
- [ ] No .unwrap() in production code (except justified)
- [ ] Error messages are actionable
- [ ] Uses anyhow for applications
- [ ] Uses thiserror for libraries
- [ ] Proper error type design

---

## 4. Performance (0-10)

**Question**: Is the implementation optimized for performance?

### Scoring Guide

**9-10 points**: Excellent
- ✅ Zero-copy where possible
- ✅ No allocations in hot paths
- ✅ Optimal algorithm choices
- ✅ Proper use of iterators (lazy evaluation)
- ✅ Memory-efficient data structures

**7-8 points**: Good
- ✅ Mostly efficient
- ⚠️ A few unnecessary allocations
- ⚠️ Could use iterators more

**5-6 points**: Acceptable
- ✅ Basic performance OK
- ⚠️ Several allocations that could be avoided
- ⚠️ Suboptimal algorithm in places

**3-4 points**: Needs Improvement
- ⚠️ Many unnecessary allocations
- ❌ Inefficient algorithms
- ❌ Performance issues likely

**0-2 points**: Poor
- ❌ Serious performance problems
- ❌ Excessive allocations
- ❌ O(n²) where O(n) possible

### Checklist
- [ ] Uses iterator chains instead of collecting intermediate results
- [ ] No .clone() in hot paths
- [ ] Appropriate data structures (HashMap, BTreeMap, Vec, etc.)
- [ ] No unnecessary String allocations
- [ ] Uses capacity hints (with_capacity) where appropriate
- [ ] Considers cache locality for large data

---

## 5. Documentation (0-10)

**Question**: Is the pattern well-documented and self-explanatory?

### Scoring Guide

**9-10 points**: Excellent
- ✅ Comprehensive doc comments
- ✅ Clear intent and usage explained
- ✅ Lifetimes explained when complex
- ✅ Examples in doc comments
- ✅ Module-level documentation
- ✅ Safety invariants documented

**7-8 points**: Good
- ✅ Basic doc comments present
- ⚠️ Could use more examples
- ⚠️ Missing some explanations

**5-6 points**: Acceptable
- ✅ Some doc comments
- ⚠️ Missing examples
- ⚠️ Intent not always clear

**3-4 points**: Needs Improvement
- ⚠️ Minimal documentation
- ❌ No examples
- ❌ Intent unclear

**0-2 points**: Poor
- ❌ No documentation
- ❌ Code is confusing
- ❌ Lifetimes unexplained

### Checklist
- [ ] Public items have doc comments
- [ ] Doc comments include examples
- [ ] Complex lifetimes are explained
- [ ] Safety requirements documented (for unsafe)
- [ ] Module-level documentation exists
- [ ] README or usage guide for complex patterns

---

## Overall Scoring

**Total Score**: Sum of all 5 dimensions (max 50, report as X/10 by dividing by 5)

### Score Interpretation

| Score | Rating | Description |
|-------|--------|-------------|
| 9.0-10.0 | ⭐⭐⭐⭐⭐ Excellent | Production-ready, exemplary implementation |
| 8.0-8.9 | ⭐⭐⭐⭐ Very Good | Strong implementation, minor improvements possible |
| 7.0-7.9 | ⭐⭐⭐⭐ Good | Solid implementation, some areas for improvement |
| 6.0-6.9 | ⭐⭐⭐ Acceptable | Works but needs improvement |
| 5.0-5.9 | ⭐⭐⭐ Fair | Has issues, refactoring recommended |
| 0.0-4.9 | ⭐⭐ Poor | Significant problems, requires rework |

---

## Evaluation Report Template

```markdown
# Pattern Evaluation: [Pattern Name] ([Type])

**File**: `path/to/file.rs`
**Lines**: X-Y
**Overall Score**: N.N/10 ⭐⭐⭐⭐

---

## Scoring Breakdown

### 1. Correctness: X/10 ⭐⭐⭐⭐⭐

✅ **Strengths**:
- [List strengths]

⚠️ **Areas for Improvement**:
- [List improvements]

**Recommendation**: [Specific advice]

---

### 2. Ownership Safety: X/10 ⭐⭐⭐⭐⭐

[Same format]

---

### 3. Error Handling: X/10 ⭐⭐⭐⭐

[Same format]

---

### 4. Performance: X/10 ⭐⭐⭐⭐⭐

[Same format]

---

### 5. Documentation: X/10 ⭐⭐⭐

[Same format]

---

## Overall Assessment

**Verdict**: ⭐⭐⭐⭐ [Rating]

[Summary paragraph]

**Priority Improvements**:
1. [Highest priority]
2. [Next priority]
3. [Optional improvement]

**Code Example** (Improved):
```rust
// Suggested improvements
```

---

## Summary

**Overall Score**: N.N/10 ([Rating])
**Strengths**: [Brief list]
**Improvement Areas**: [Brief list]
**Effort to Improve**: [Low/Medium/High] ([X hours])
```
