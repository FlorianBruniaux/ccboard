# ccboard TODO

Generated from Rust Patterns Audit (2026-02-09)

## 🚨 Critical - v0.5.0 Blockers

### [ ] Fix unwraps in session_index.rs (4h)
**Priority**: URGENT
**File**: `crates/ccboard-core/src/parsers/session_index.rs`
**Issue**: 157 unwraps → panic on malformed JSONL

**Actions**:
- [ ] Replace `sem.acquire().await.unwrap()` line 578 → `?` with context
- [ ] Update scan_all to handle acquire errors gracefully
- [ ] Test with malformed fixtures
- [ ] Verify LoadReport populates errors instead of panic

**Success Criteria**:
- ✅ Malformed session file → LoadReport.errors populated, scan continues
- ✅ No panic on semaphore failures

---

### [ ] Fix unwraps in metadata_cache.rs (2h)
**Priority**: URGENT
**File**: `crates/ccboard-core/src/cache/metadata_cache.rs`
**Issue**: 38 Mutex lock unwraps → panic if lock poisoned

**Actions**:
- [ ] Replace all `conn.lock().unwrap()` → `map_err()` (lines 160, 199, 233, 247, 270, 303, 314)
- [ ] Update DataStore::initial_load to handle cache failures
- [ ] Test with poisoned lock scenarios
- [ ] Verify fallback to JSONL parsing works

**Success Criteria**:
- ✅ Cache lock failure → fallback to JSONL parsing, no crash
- ✅ All cache errors propagate to LoadReport

---

## 🟡 High Priority - Next Sprint

### [ ] Add Newtype for SessionId/ProjectId (1h)
**Priority**: HIGH
**File**: `crates/ccboard-core/src/models/session.rs`
**Issue**: Raw String types allow ID confusion

**Actions**:
- [ ] Create `SessionId(String)` newtype with From/Display traits
- [ ] Create `ProjectId(String)` newtype
- [ ] Update `SessionMetadata { id: SessionId }`
- [ ] Update `DashMap<SessionId, Arc<SessionMetadata>>`
- [ ] Update all parsers to return SessionId
- [ ] Run tests to catch type mismatches

**Success Criteria**:
- ✅ Can't pass ProjectId where SessionId expected (compile error)
- ✅ All tests pass with new types
- ✅ No performance regression

---

### [ ] Add context to all ? operators (2h)
**Priority**: HIGH
**File**: `crates/ccboard-core/src/parsers/*.rs`
**Issue**: Error messages lack context for debugging

**Actions**:
- [ ] Audit all `?` operators in parsers/
- [ ] Add `.context("descriptive message")` to each
- [ ] Ensure error chains include file paths when applicable
- [ ] Test error messages with malformed fixtures

**Success Criteria**:
- ✅ All errors have context explaining WHERE and WHY
- ✅ No cryptic "unexpected EOF" without context

---

## 🟢 Medium Priority - Future Sprints

### [ ] Reduce clone chains in config merge (2h)
**Priority**: MEDIUM
**File**: `crates/ccboard-core/src/models/config.rs`
**Issue**: Double allocations in HashMap merge (lines 262, 285, 299)

**Actions**:
- [ ] Analyze if `source` can be mutable in merge_into()
- [ ] Use `.take()` + `.extend()` instead of `.clone()` if possible
- [ ] If not: Consider Arc<HashMap> for keybindings/env
- [ ] Benchmark before/after

**Success Criteria**:
- ✅ Reduce allocations by 50% during config merge
- ✅ Benchmark shows measurable speedup (>5%)

---

### [ ] Replace unwrap with expect in tests (3h)
**Priority**: MEDIUM
**File**: All test files
**Issue**: Test failures lack descriptive messages

**Actions**:
- [ ] Automated refactor: `.unwrap()` → `.expect("Test fixture should...")`
- [ ] Add descriptive messages explaining WHAT is expected
- [ ] Run full test suite to verify

**Success Criteria**:
- ✅ All test failures have clear context
- ✅ Clippy warning `clippy::unwrap_used` enabled

---

## ⚡ Quick Wins (<1h each)

### [ ] Add .context() to remaining parser ? operators (30 min)
**Files**: `parsers/stats.rs`, `parsers/settings.rs`
**Pattern**: `parse()?` → `parse().context("Failed to parse stats cache")?`

### [ ] Document Arc vs Box trade-off (15 min)
**File**: `store.rs:91`
**Action**: Add comment explaining atomic overhead justification

### [ ] Document parking_lot choice (15 min)
**File**: `store.rs:18`
**Action**: Comment explaining parking_lot > std::sync::RwLock

### [ ] Add RAII Drop impl for MetadataCache (30 min)
**File**: `cache/metadata_cache.rs`
**Action**: WAL checkpoint on drop

### [ ] Enable clippy::unwrap_used lint in CI (15 min)
**File**: `.cargo/config.toml` or `clippy.toml`
**Action**: Add `unwrap_used = "deny"` to lints

---

## 📋 Backlog - Nice to Have

### [ ] Builder for DataStoreConfig
**Priority**: LOW
**Effort**: 2h
**File**: `crates/ccboard-core/src/store.rs`

### [ ] Benchmark DashMap vs Arc<RwLock<HashMap>>
**Priority**: LOW
**Effort**: 3h
**File**: `crates/ccboard-core/src/store.rs`
**Reason**: Profile memory overhead (1000 sessions × 2x = ?)

---

## 📈 Progress Tracking

**Total Effort Estimate**: 14 hours
- Critical: 6h (blockers pour v0.5.0)
- High: 3h (next sprint)
- Medium: 5h (future sprints)

**Quick Wins**: 1.75h (can be done during code reviews)

**Current Health**: 🟡 Good with critical fixes needed

---

**Full Report**: See `claudedocs/rust-patterns-audit-report.md`
**Metrics**: See `claudedocs/rust-patterns-audit-metrics.json`
