# Phase I-TUI: Sessions Tab Enhancement Progress

## ✅ Completed (Tier 1 & 2)

### Tier 1: Branch Display
- [x] Branch in sessions list (Magenta, truncated to 12 chars)
- [x] Branch in `render_detail()`
- [x] Branch in `render_live_detail()`

### Tier 2: Resume Action
- [x] Handler for 'r' key with `resume_claude_session()`
- [x] Keyboard hint for resume in Sessions panel
- [x] Status bar hint update

**Commit**: `a0bc804` - feat(tui): add branch display and resume functionality to Sessions tab

---

## 📋 Remaining (Tier 3 & 4)

### Tier 3: Cross-Project Search (~60 LOC)

**Goal**: Enable searching across ALL sessions when no project is selected or when search is initiated from Projects focus.

**Implementation**:
1. Add `search_global: bool` field to `SessionsTab` struct
2. Modify search behavior:
   - `/` from focus==1 (Projects) → global search
   - `/` from focus==2 (Sessions) → local search (current)
3. Update `get_filtered_sessions()` to accept all sessions when global
4. Display results with project prefix for context

**Files to modify**:
- `crates/ccboard-tui/src/tabs/sessions.rs`
  - Struct: add `search_global: bool`
  - Handler: detect focus when `/` is pressed
  - Rendering: prefix sessions with project name in global mode

**Testing**:
```bash
cargo run
# Press Tab to Sessions
# Press Tab to Projects focus
# Press '/' and type search query
# → Should see results from ALL projects
```

---

### Tier 4: Date Filter (Optional, ~80 LOC)

**Goal**: Quick date filtering via keyboard cycling.

**Implementation**:
1. Add `DateFilter` enum to sessions.rs (or reuse from CLI)
2. Add 'd' key handler to cycle: All → 24h → 7d → 30d → All
3. Display active filter in panel title or search bar
4. Filter sessions in `get_filtered_sessions()`

**DateFilter logic** (from CLI):
```rust
pub enum DateFilter {
    All,
    Since(DateTime<Utc>),
}

impl DateFilter {
    fn cutoff(&self) -> Option<DateTime<Utc>> {
        match self {
            DateFilter::All => None,
            DateFilter::Since(dt) => Some(*dt),
        }
    }

    fn matches(&self, session: &SessionMetadata) -> bool {
        match self.cutoff() {
            None => true,
            Some(cutoff) => session.first_timestamp
                .map(|ts| ts >= cutoff)
                .unwrap_or(false),
        }
    }
}
```

**Files to modify**:
- `crates/ccboard-tui/src/tabs/sessions.rs`
  - Add DateFilter enum (or import from CLI)
  - Add `date_filter: DateFilter` field
  - Handler for 'd' key to cycle filters
  - Apply filter in session list

**UI Changes**:
- Display current filter in Sessions panel title: "Sessions (Last 7 days)"
- Or in status bar: "24h filter active"

**Testing**:
```bash
cargo run
# Navigate to Sessions
# Press 'd' multiple times
# → Filter should cycle: All → 24h → 7d → 30d → All
# → Sessions list should update accordingly
# → Current filter displayed in UI
```

---

## 🎯 Next Steps

1. **Tier 3 (Recommended)**: Cross-project search is high-value for users with many projects
2. **Tier 4 (Optional)**: Date filter is nice-to-have for large session histories
3. **Testing**: Manual TUI testing after each tier
4. **Documentation**: Update README with new keyboard shortcuts

---

## 📊 Metrics

| Metric | Value |
|--------|-------|
| Tiers completed | 2/4 (50%) |
| LOC added (T1+T2) | ~80 LOC |
| Files modified | 3 core + 3 formatting |
| New functions | 1 (`resume_claude_session`) |
| Tests passing | ✅ (pre-existing failures unrelated) |
| Clippy warnings | 0 new |

---

## 🔧 Commands Reference

```bash
# Build & validate
cargo fmt --all
cargo clippy --all-targets
cargo build --all
cargo test --all

# Run TUI
cargo run

# Sessions tab shortcuts (after implementation)
Tab/Shift+Tab  - Cycle focus (Live → Projects → Sessions)
↑↓ j/k         - Navigate lists
Enter          - Toggle detail view
/              - Search (global from Projects, local from Sessions)
r              - Resume session in Claude CLI
e              - Edit session file
o              - Reveal in file manager
y              - Copy session ID
d              - Cycle date filter (Tier 4)
gg/G           - Jump to top/bottom
```
