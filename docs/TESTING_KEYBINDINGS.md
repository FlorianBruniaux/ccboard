# Testing Custom Keybindings - Phase 4

This document provides manual testing instructions for the custom keybindings feature.

## Setup

### 1. Build ccboard

```bash
cd /Users/florianbruniaux/Sites/perso/ccboard
cargo build --release
```

### 2. Prepare Test Settings

Copy one of the example configurations:

```bash
# Default keybindings (documents the defaults)
cp examples/settings-keybindings.json ~/.claude/settings.json

# OR custom keybindings (Vim-style)
cp examples/settings-custom-keys.json ~/.claude/settings.json
```

## Test Plan

### Test 1: Default Keybindings (No Configuration)

**Setup**: Remove or rename `~/.claude/settings.json` if it exists

**Steps**:
1. Run `./target/release/ccboard`
2. Verify default keybindings work:
   - `q` → Quit
   - `Ctrl+Q` → Force quit
   - `F5` → Refresh
   - `Ctrl+R` → Force refresh + toast
   - `Ctrl+T` → Toggle theme (Dark ↔ Light)
   - `Tab` → Next tab
   - `Shift+Tab` → Previous tab
   - `1-9` → Jump to tabs
   - `?` → Toggle help modal
   - `:` → Show command palette
   - `Esc` → Close modal

**Expected**: All default keys work as documented

---

### Test 2: Custom Keybindings Override

**Setup**: Create `~/.claude/settings.json`:

```json
{
  "keybindings": {
    "Ctrl+C": "quit",
    "r": "refresh"
  }
}
```

**Steps**:
1. Run `./target/release/ccboard`
2. Test custom bindings:
   - `Ctrl+C` → Should quit (overrides default)
   - `r` → Should refresh (new binding)
3. Test remaining defaults:
   - `F5` → Should still refresh (default not overridden)
   - `Tab` → Should still work (default preserved)

**Expected**: Custom bindings work, defaults remain for unmodified keys

---

### Test 3: Help Modal Shows Custom Keys

**Setup**: Use settings from Test 2

**Steps**:
1. Run `./target/release/ccboard`
2. Press `?` to open help modal
3. Verify "Global" section shows:
   - `Ctrl+C` for "Quit application" (not `q`)
   - `R` for "Refresh data" (showing both `r` and `F5`)

**Expected**: Help modal dynamically reflects custom keybindings

---

### Test 4: Invalid Keybindings Graceful Failure

**Setup**: Create `~/.claude/settings.json` with errors:

```json
{
  "keybindings": {
    "Ctrl+Q": "quit",
    "INVALID_KEY": "refresh",
    "F5": "unknown_action",
    "Ctrl-X": "quit"
  }
}
```

**Steps**:
1. Run `./target/release/ccboard`
2. Check stderr for warnings:
   - Warning about `INVALID_KEY`
   - Warning about `unknown_action`
   - Warning about `Ctrl-X` (should use `+` not `-`)
3. Verify valid binding works:
   - `Ctrl+Q` → Should quit

**Expected**: Application starts, logs warnings, skips malformed entries

---

### Test 5: Settings Hierarchy (4-Layer Merge)

**Setup**: Create layered settings:

```bash
# Global
echo '{"keybindings": {"q": "quit", "r": "refresh"}}' > ~/.claude/settings.json

# Global Local (overrides global)
echo '{"keybindings": {"r": "force_refresh"}}' > ~/.claude/settings.local.json

# Project (overrides global layers)
mkdir -p /tmp/test-project/.claude
echo '{"keybindings": {"q": "force_quit"}}' > /tmp/test-project/.claude/settings.json

# Project Local (highest priority)
echo '{"keybindings": {"Ctrl+T": "quit"}}' > /tmp/test-project/.claude/settings.local.json
```

**Steps**:
1. Run `./target/release/ccboard --project /tmp/test-project`
2. Test merged bindings:
   - `q` → Force quit (project overrides global)
   - `r` → Force refresh (global local overrides global)
   - `Ctrl+T` → Quit (project local highest priority)

**Expected**: Settings merge correctly with proper precedence

---

### Test 6: All Supported Keys

**Setup**: Create comprehensive test configuration:

```json
{
  "keybindings": {
    "a": "quit",
    "Z": "refresh",
    "5": "jump_tab_4",
    "F1": "jump_tab_0",
    "F12": "refresh",
    "Tab": "next_tab",
    "Shift+Tab": "prev_tab",
    "Esc": "close_modal",
    "Enter": "toggle_help",
    "Space": "refresh",
    "Backspace": "prev_tab",
    "Delete": "close_modal",
    "Up": "prev_tab",
    "Down": "next_tab",
    "Home": "jump_tab_0",
    "End": "jump_tab_8",
    "Ctrl+A": "quit",
    "Alt+B": "refresh",
    "Shift+C": "theme_toggle"
  }
}
```

**Steps**:
1. Run `./target/release/ccboard`
2. Test each key type:
   - Letters: `a`, `Z`
   - Numbers: `5`
   - Function keys: `F1`, `F12`
   - Special keys: `Tab`, `Esc`, `Enter`, `Space`, etc.
   - Arrow keys: `Up`, `Down`
   - Modifiers: `Ctrl+A`, `Alt+B`, `Shift+C`

**Expected**: All supported key types work correctly

---

### Test 7: Case Insensitivity

**Setup**: Create settings with mixed case:

```json
{
  "keybindings": {
    "ctrl+q": "quit",
    "CTRL+R": "refresh",
    "Shift+TAB": "prev_tab",
    "f5": "refresh"
  }
}
```

**Steps**:
1. Run `./target/release/ccboard`
2. Test all bindings work regardless of case

**Expected**: Case-insensitive parsing works correctly

---

### Test 8: Modifier Combinations

**Setup**: Test all modifier combinations:

```json
{
  "keybindings": {
    "Ctrl+Q": "quit",
    "Shift+Q": "refresh",
    "Alt+Q": "theme_toggle",
    "Ctrl+Shift+Q": "force_quit",
    "Ctrl+Alt+Q": "close_modal"
  }
}
```

**Steps**:
1. Run `./target/release/ccboard`
2. Test each modifier combination separately

**Expected**: All modifier combinations work without conflicts

---

### Test 9: Theme Toggle Persistence

**Setup**: Default keybindings

**Steps**:
1. Run `./target/release/ccboard`
2. Press `Ctrl+T` to toggle theme
3. Verify toast shows "Theme: Light"
4. Verify UI switches to light theme
5. Press `Ctrl+T` again
6. Verify toast shows "Theme: Dark"
7. Verify UI switches back to dark theme

**Expected**: Theme toggles work with toast feedback

---

### Test 10: Modal Close Behavior

**Setup**: Default keybindings

**Steps**:
1. Run `./target/release/ccboard`
2. Press `?` to open help modal
3. Press `Esc` to close
4. Verify help modal closed
5. Press `:` to open command palette
6. Press `Esc` to close
7. Verify command palette closed

**Expected**: `Esc` (CloseModal action) works for all modals

---

## Performance Test

**Setup**: Large keybindings configuration (100+ entries)

**Steps**:
1. Create settings.json with 100+ keybindings
2. Run `./target/release/ccboard`
3. Measure startup time
4. Test key lookup performance

**Expected**: No noticeable performance impact (<50ms startup overhead)

---

## Edge Cases

### Empty Keybindings

```json
{
  "keybindings": {}
}
```

**Expected**: Falls back to all defaults

### Null Keybindings

```json
{
  "keybindings": null
}
```

**Expected**: Falls back to all defaults

### Duplicate Keys

```json
{
  "keybindings": {
    "q": "quit",
    "q": "refresh"
  }
}
```

**Expected**: Last definition wins (JSON parsing behavior)

### Conflicting Bindings

Bind `Tab` to two actions in different layers:
- Global: `Tab` → `next_tab`
- Project: `Tab` → `prev_tab`

**Expected**: Project binding takes precedence

---

## Regression Tests

Ensure existing functionality still works:

1. **Command Palette**: `:` still opens palette
2. **Help Modal**: `?` still shows help
3. **Tab Navigation**: Number keys still jump to tabs
4. **Tab-specific keys**: Keys in active tabs still work

---

## Automated Test Suite

Run unit tests:

```bash
cargo test -p ccboard-tui keybindings
```

Expected output:
```
running 11 tests
test keybindings::tests::test_action_from_name ... ok
test keybindings::tests::test_parse_case_insensitive ... ok
test keybindings::tests::test_keybindings_defaults ... ok
test keybindings::tests::test_parse_function_key ... ok
test keybindings::tests::test_keybindings_reverse_lookup ... ok
test keybindings::tests::test_keybindings_custom_override ... ok
test keybindings::tests::test_parse_key_with_ctrl ... ok
test keybindings::tests::test_parse_multiple_modifiers ... ok
test keybindings::tests::test_parse_shift_tab ... ok
test keybindings::tests::test_parse_simple_key ... ok
test keybindings::tests::test_parse_tab ... ok

test result: ok. 11 passed; 0 failed
```

---

## Sign-Off Checklist

- [ ] All default keybindings work without configuration
- [ ] Custom keybindings override defaults correctly
- [ ] Help modal shows custom keys dynamically
- [ ] Invalid entries fail gracefully with warnings
- [ ] Settings merge across 4 layers correctly
- [ ] All supported key types work
- [ ] Case-insensitive parsing works
- [ ] All modifier combinations work
- [ ] Theme toggle works with toast feedback
- [ ] Modal close (Esc) works for all modals
- [ ] No performance degradation
- [ ] All unit tests pass
- [ ] No regressions in existing features

---

## Known Limitations (Phase 4)

- **No multi-key sequences**: `Ctrl+X Ctrl+C` not supported yet
- **No tab-specific bindings**: Global keybindings only
- **No conflict detection**: Application won't warn about conflicting bindings
- **No visual key recording**: Users must manually edit settings.json

These limitations are planned for Phase 6+ enhancements.
