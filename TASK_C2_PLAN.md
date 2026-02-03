# Implementation Plan: C.2 - History Tab Export CSV/JSON

**Date**: 2026-02-03
**Task**: Phase C, Task C.2 - History Tab export CSV/JSON
**Status**: Planning phase - Review required

---

## Executive Summary

**Objective**: Add CSV/JSON export functionality to History tab for filtered session results

**Scope**:
- Export filtered sessions to CSV format
- Export filtered sessions to JSON format
- Key binding in History tab ('x' for export)
- User prompts for format selection
- Auto-create export directory

**Estimated Effort**: 2-3h
**Risk**: Low (reuse export infrastructure from C.3)

---

## Current State Analysis

### History Tab Features (Existing)
- Search/filter sessions by project path, prompt text, model
- Display filtered results in list
- Show session details in popup
- Open in editor (`e` key)
- Reveal in file manager (`o` key)
- Clear search (`c` key)

**Key Data**: `filtered_sessions: Vec<SessionMetadata>`
- Already computed based on search query
- Contains all fields needed for export

### SessionMetadata Structure
```rust
pub struct SessionMetadata {
    pub file_path: PathBuf,
    pub project_path: String,
    pub session_id: String,
    pub first_timestamp: Option<DateTime<Utc>>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub message_count: usize,
    pub total_tokens: u64,
    pub models_used: Vec<String>,
    // ... other fields
}
```

---

## Implementation Plan

### Task 1: Extend Export Module (1h)

**File**: `crates/ccboard-core/src/export.rs`

#### 1.1: Add CSV export for sessions
```rust
/// Export sessions to CSV format
///
/// CSV columns: Date, Time, Project, Session ID, Messages, Tokens, Models, Duration
pub fn export_sessions_to_csv(
    sessions: &[SessionMetadata],
    path: &Path,
) -> Result<()> {
    // Create parent dirs
    // Write header
    // Write each session as row
    // Format timestamps as ISO 8601
    // Join models with ";"
}
```

**CSV Format**:
```csv
Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)
"2026-02-03","14:30:45","/Users/x/project","abc123",25,15000,"sonnet;opus",45
```

#### 1.2: Add JSON export for sessions
```rust
/// Export sessions to JSON format
///
/// Pretty-printed JSON array of session metadata
pub fn export_sessions_to_json(
    sessions: &[SessionMetadata],
    path: &Path,
) -> Result<()> {
    // Serialize sessions to JSON
    // Pretty print with serde_json
    // Write to file
}
```

**JSON Format**:
```json
[
  {
    "session_id": "abc123",
    "project_path": "/Users/x/project",
    "first_timestamp": "2026-02-03T14:30:45Z",
    "last_timestamp": "2026-02-03T15:15:20Z",
    "message_count": 25,
    "total_tokens": 15000,
    "models_used": ["sonnet", "opus"]
  }
]
```

#### 1.3: Tests (5 unit tests)
```rust
#[test]
fn test_export_sessions_csv_empty() { ... }

#[test]
fn test_export_sessions_csv_with_data() { ... }

#[test]
fn test_export_sessions_json_empty() { ... }

#[test]
fn test_export_sessions_json_with_data() { ... }

#[test]
fn test_export_sessions_creates_dirs() { ... }
```

---

### Task 2: Add Export UI to History Tab (0.5h)

**File**: `crates/ccboard-tui/src/tabs/history.rs`

#### 2.1: Add export state
```rust
pub struct HistoryTab {
    // ... existing fields ...

    /// Export format selection (None, Some("csv"), Some("json"))
    export_format: Option<String>,

    /// Export in progress
    exporting: bool,

    /// Export success message
    export_message: Option<String>,
}
```

#### 2.2: Add key binding handler
```rust
KeyCode::Char('x') | KeyCode::Char('X') => {
    // Trigger export dialog
    // Show format selection (CSV / JSON)
    self.show_export_dialog = true;
}

KeyCode::Char('1') if self.show_export_dialog => {
    // Export as CSV
    self.export_format = Some("csv");
    self.trigger_export();
}

KeyCode::Char('2') if self.show_export_dialog => {
    // Export as JSON
    self.export_format = Some("json");
    self.trigger_export();
}
```

#### 2.3: Export logic
```rust
fn trigger_export(&mut self) {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let filename = match self.export_format.as_deref() {
        Some("csv") => format!("sessions_export_{}.csv", timestamp),
        Some("json") => format!("sessions_export_{}.json", timestamp),
        _ => return,
    };

    let export_path = dirs::home_dir()
        .unwrap()
        .join(".claude/exports")
        .join(filename);

    // Call export function
    let result = match self.export_format.as_deref() {
        Some("csv") => ccboard_core::export_sessions_to_csv(&self.filtered_sessions, &export_path),
        Some("json") => ccboard_core::export_sessions_to_json(&self.filtered_sessions, &export_path),
        _ => return,
    };

    match result {
        Ok(_) => {
            self.export_message = Some(format!("✓ Exported {} sessions to {}",
                self.filtered_sessions.len(),
                export_path.display()
            ));
        }
        Err(e) => {
            self.export_message = Some(format!("✗ Export failed: {}", e));
        }
    }
}
```

#### 2.4: Render export dialog
```rust
if self.show_export_dialog {
    let popup = Paragraph::new(vec![
        Line::from("Export Sessions"),
        Line::from(""),
        Line::from("1. CSV format"),
        Line::from("2. JSON format"),
        Line::from(""),
        Line::from("ESC to cancel"),
    ])
    .block(Block::default().borders(Borders::ALL).title("Export"));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}
```

---

### Task 3: Update Help Text (0.5h)

**File**: `crates/ccboard-tui/src/components/help_modal.rs`

Add to History tab section:
```
x       Export filtered sessions (CSV/JSON)
```

---

## Testing Strategy

### Unit Tests (Export Module)
```bash
cargo test -p ccboard-core export::tests::test_export_sessions
```

**Expected**: 5 new tests pass

### Manual Testing

#### Test 1: CSV Export
```bash
cargo run
# Navigate to History tab (Tab to tab 7)
# Search for something: /project
# Press 'x' for export
# Press '1' for CSV
# Check: ~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.csv
```

**Expected CSV**:
```csv
Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)
"2026-02-03","14:30:45","/path/to/project","abc123",25,15000,"sonnet;opus",45
```

#### Test 2: JSON Export
```bash
cargo run
# Navigate to History tab
# Search: /project
# Press 'x', then '2' for JSON
# Check: ~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.json
```

**Expected JSON**:
```json
[
  {
    "session_id": "abc123",
    "project_path": "/path/to/project",
    ...
  }
]
```

#### Test 3: Empty Filter
```bash
cargo run
# History tab, no search
# Press 'x', then '1' for CSV
# Should export ALL sessions
```

**Expected**: CSV with all sessions (3600+)

#### Test 4: Error Handling
```bash
# Make exports dir read-only
chmod 444 ~/.claude/exports
cargo run
# History tab, 'x', '1'
# Should show error message
```

**Expected**: Error message displayed in TUI

---

## File Changes Summary

### New/Modified Files
```
crates/ccboard-core/src/export.rs                (+120 LOC, 2 functions + 5 tests)
crates/ccboard-core/src/lib.rs                   (+2 LOC, re-exports)
crates/ccboard-tui/src/tabs/history.rs           (+80 LOC, export logic + UI)
crates/ccboard-tui/src/components/help_modal.rs  (+1 LOC, help text)
```

**Total**: ~200 LOC

---

## Dependencies

### Already Available
- `serde_json` - JSON serialization
- `chrono` - Timestamp formatting
- `anyhow` - Error handling
- `dirs` - Home directory

### No New Dependencies Required ✅

---

## Success Criteria

C.2 is complete when:

1. ✅ `export_sessions_to_csv()` function implemented
2. ✅ `export_sessions_to_json()` function implemented
3. ✅ 5 unit tests pass (CSV empty/data, JSON empty/data, dirs)
4. ✅ 'x' key binding in History tab triggers export dialog
5. ✅ Format selection dialog (1=CSV, 2=JSON)
6. ✅ Success/error messages displayed
7. ✅ Exported files created in `~/.claude/exports/`
8. ✅ Help text updated
9. ✅ `cargo fmt && cargo clippy && cargo test --all` passes (0 warnings)

---

## Out of Scope (Future)

- ❌ Export format selection via dropdown menu → Phase D
- ❌ Custom export path selection → Phase E
- ❌ Export to other formats (Excel, Markdown) → Phase E
- ❌ Filter/column selection in CSV → Phase E
- ❌ Export from Sessions tab → C.4 or later
- ❌ Batch export all projects → Phase E

---

## Alternative Approaches Considered

### Option 1: Single 'x' key cycles through formats
**Pro**: Fewer keystrokes
**Con**: Unclear which format will be exported

### Option 2: Two separate keys ('x' CSV, 'j' JSON)
**Pro**: Direct export, no dialog
**Con**: Hard to remember which key = which format

### Option 3: Export dialog with format selection (CHOSEN)
**Pro**: Clear, discoverable, extensible
**Con**: Extra keystroke required

---

## Notes

1. **Reuse C.3 patterns**: Directory creation, error handling, BufWriter
2. **SessionMetadata is Clone**: Can pass by value if needed
3. **Filtered sessions already computed**: No extra processing needed
4. **Export path pattern**: `~/.claude/exports/sessions_export_YYYYMMDD_HHMMSS.{csv,json}`
5. **Timestamp in filename**: Prevents overwrite, easy to identify

---

## Questions for Review

1. **CSV columns**: Current proposal is Date, Time, Project, Session ID, Messages, Tokens, Models, Duration. Add/remove columns?
2. **JSON format**: Full SessionMetadata serialization OK, or custom subset?
3. **Key binding**: 'x' for export OK, or different key?
4. **Export dialog**: Inline popup or full modal?
5. **File naming**: `sessions_export_YYYYMMDD_HHMMSS.csv` pattern OK?

---

**Plan Status**: 🔍 **REVIEW REQUESTED**
**Estimated Time**: 2-3h
**Blockers**: None (C.3 infrastructure ready)
**Ready to Start**: After approval ✅
