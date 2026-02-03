# TUI Polish & Completion - Phase E

**Objectif**: Finaliser le TUI avec polish UI/UX, error handling, et keyboard shortcuts

**Durée estimée**: 6-8h
**Temps écoulé**: ~2.5h
**Progression**: 40%

## 📊 Résumé de Progression

| Priorité | Section | Status | Temps | Commits |
|----------|---------|--------|-------|---------|
| **1** | Polish UI/UX (Quick Wins) | 🟡 Partiel | 30min | `04f365f` |
| **2** | Hooks Tab | ✅ **Complete** | **1.5h** | `40bc04e` |
| **3** | Error Handling | ⏳ À faire | 1h | - |
| **4** | Keyboard Shortcuts | 🟡 Partiel | 30min | `580c3e8` |
| **5** | Performance | ⏳ À faire | 1h | - |
| **6** | Status Messages | ⏳ À faire | 1h | - |

**Dernière mise à jour**: 2026-02-03 (commit `40bc04e`)

---

## 1. Polish UI/UX (2h)

### 1.1 Visual Improvements
- [ ] Add separators/borders between sections
- [ ] Improve color consistency across tabs
- [x] Better alignment and spacing (empty states)
- [ ] Loading states for slow operations (not needed - sync operations)
- [x] Empty state messages (when no data) - Sessions, History, Costs tabs

### 1.2 Dashboard Enhancements
- [ ] Add tooltips/descriptions for metrics
- [ ] Highlight current day in activity chart
- [ ] Show trend indicators (↑↓) for metrics

### 1.3 Sessions Tab Polish
- [ ] Add session age indicators (e.g., "2 days ago")
- [ ] Show file size in human-readable format
- [ ] Better preview truncation with "..."
- [ ] Add session count per project in tree

### 1.4 History Tab Polish
- [ ] Better date grouping (Today, Yesterday, This Week, etc.)
- [ ] Add filters (by project, by model, by date range)
- [ ] Highlight search terms in results

---

## 2. Hooks Tab Improvements (1.5h) ✅ COMPLETE

### 2.1 Current State Analysis
- [x] Read hooks.rs to see current implementation
- [x] Identify missing features

### 2.2 Enhancements
- [ ] Add hook execution count/statistics (no data available in sessions)
- [ ] Show last execution timestamp (no data available in sessions)
- [x] Syntax highlighting for bash scripts (comments, strings, variables, keywords)
- [x] Add "test hook" functionality (keyboard shortcut 't')
- [x] Show hook parameters/environment (badges: async, timeout, env vars)
- [x] Better error display when hook fails (test result popup with color coding)

### 2.3 UI Improvements
- [x] Clearer hook type indicators (badges: async, ⏱timeout, env:N)
- [x] Better organization (visual badges for properties)
- [ ] Add hook enabled/disabled status (not in data model)

---

## 3. Error Handling (1h)

### 3.1 Error Categories
- [ ] File not found errors
- [ ] Permission errors
- [ ] Parse errors (malformed JSONL)
- [ ] Network/timeout errors (for future)

### 3.2 Error Display
- [ ] User-friendly error messages (no stack traces)
- [ ] Actionable suggestions (e.g., "Try: chmod +x file")
- [ ] Error context (which file, which operation)
- [ ] Recovery options (Retry, Skip, Ignore)

### 3.3 Error Logging
- [ ] Optional debug mode (RUST_LOG=debug)
- [ ] Error log file (~/.claude/ccboard-errors.log)
- [ ] Show last N errors in status bar

---

## 4. Keyboard Shortcuts (1.5h)

### 4.1 Global Shortcuts
- [x] `q` - Quit
- [x] `?` - Help modal
- [x] `Tab` - Next tab
- [x] `Shift+Tab` - Previous tab
- [x] `1-8` - Jump to tab
- [x] `F5` - Refresh
- [ ] `Ctrl+R` - Force reload all data
- [ ] `Ctrl+Q` - Quit without confirmation
- [ ] `Esc` - Cancel/Close current dialog

### 4.2 Navigation Shortcuts
- [x] `g` + `g` - Go to top (vim-style) - Sessions + History tabs
- [x] `G` - Go to bottom - Sessions + History tabs
- [ ] `Ctrl+D` - Page down (PageDown already works)
- [ ] `Ctrl+U` - Page up (PageUp already works)
- [x] `Home` / `End` - First/Last item - Sessions + History tabs

### 4.3 Tab-Specific Shortcuts
**Sessions:**
- [x] `/` - Search
- [x] `Enter` - View detail
- [ ] `d` - Delete session (with confirmation)
- [ ] `y` - Copy session ID to clipboard
- [ ] `t` - Toggle tree collapse/expand

**History:**
- [x] `/` - Search
- [x] `c` - Clear search
- [x] `x` - Export
- [ ] `f` - Focus filter dialog

**Costs:**
- [x] `Tab/h/l` - Switch views
- [ ] `s` - Sort by cost/tokens/date

**Hooks:**
- [x] `t` - Test hook (execute and show result)
- [x] `e` - Edit hook file
- [x] `o` - Reveal hook file

**MCP:**
- [x] `y` - Copy command
- [x] `e` - Edit config
- [x] `o` - Reveal file
- [x] `r` - Refresh

### 4.4 Command Palette Enhancements
- [ ] Add more commands to palette
- [ ] Fuzzy search in palette
- [ ] Show keyboard shortcuts in palette

---

## 5. Performance (1h)

### 5.1 Rendering Optimization
- [x] Arc<SessionMetadata> (Phase D - DONE)
- [ ] Virtualized scrolling for large lists (>1000 items)
- [ ] Debounced search input (300ms delay)
- [ ] Lazy load session details (on Enter, not on select)

### 5.2 Data Loading
- [x] Background loading with spinner (Phase 3.1 - DONE)
- [x] SQLite cache (Phase 2.1 - DONE)
- [ ] Incremental session loading (load visible range first)
- [ ] Cancel in-flight operations on tab switch

### 5.3 Memory Optimization
- [ ] Limit cached session content (keep only recent N)
- [ ] Clear old data on F5 refresh
- [ ] Monitor memory usage in debug mode

---

## 6. Status Messages & Feedback (1h)

### 6.1 Status Bar Enhancements
- [ ] Show current operation (e.g., "Loading sessions...")
- [ ] Progress percentage for long operations
- [ ] Success/error indicators with color
- [ ] Auto-clear after 3 seconds

### 6.2 Toast Notifications
- [ ] Success messages (green) - "Exported 50 sessions"
- [ ] Warning messages (yellow) - "Cache outdated, refreshing..."
- [ ] Error messages (red) - "Failed to load config"
- [ ] Info messages (blue) - "Tip: Press ? for help"

### 6.3 Confirmation Dialogs
- [ ] Delete confirmations
- [ ] Quit confirmation (if data not saved)
- [ ] Overwrite file confirmations

### 6.4 Progress Indicators
- [ ] Spinner for async operations
- [ ] Progress bar for exports
- [ ] File count during scans

---

## Implementation Order

1. **Quick Wins** (30min)
   - Empty state messages
   - Loading states
   - Status bar messages

2. **Keyboard Shortcuts** (1h)
   - Add missing global shortcuts
   - Vim-style navigation
   - Tab-specific shortcuts

3. **Error Handling** (1h)
   - User-friendly error messages
   - Error recovery options
   - Error logging

4. **Hooks Tab** (1.5h)
   - Read current implementation
   - Add missing features
   - UI improvements

5. **UI Polish** (2h)
   - Visual improvements
   - Better spacing/colors
   - Enhanced widgets

6. **Performance** (1h)
   - Virtualized scrolling
   - Debounced search
   - Memory optimization

7. **Status Messages** (1h)
   - Toast notifications
   - Confirmation dialogs
   - Progress indicators

---

## Testing Checklist

After each section:
- [ ] Manual testing of new features
- [ ] Verify no regressions
- [ ] Update help modal with new shortcuts
- [ ] Update documentation

---

## Success Criteria

- [ ] All 7 tabs polished and consistent
- [ ] Error messages helpful and actionable
- [ ] Keyboard shortcuts comprehensive
- [ ] Performance smooth with 5000+ sessions
- [ ] User feedback clear and timely
- [ ] Zero clippy warnings
- [ ] All tests passing
