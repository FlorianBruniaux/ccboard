# KeyAction Reference - Complete List

This document lists all available `KeyAction` values for custom keybindings in ccboard.

## Quick Reference Table

| Action Name | Description | Default Key | Category |
|-------------|-------------|-------------|----------|
| `quit` | Quit application | `q` | Global |
| `force_quit` | Force quit without confirmation | `Ctrl+Q` | Global |
| `refresh` | Refresh data from disk | `F5` | Global |
| `force_refresh` | Force refresh + clear cache | `Ctrl+R` | Global |
| `theme_toggle` | Toggle theme (Dark/Light) | `Ctrl+T` | Global |
| `next_tab` | Navigate to next tab | `Tab` | Navigation |
| `prev_tab` | Navigate to previous tab | `Shift+Tab` | Navigation |
| `jump_tab_0` | Jump to Dashboard | `1` | Navigation |
| `jump_tab_1` | Jump to Sessions | `2` | Navigation |
| `jump_tab_2` | Jump to Config | `3` | Navigation |
| `jump_tab_3` | Jump to Hooks | `4` | Navigation |
| `jump_tab_4` | Jump to Agents | `5` | Navigation |
| `jump_tab_5` | Jump to Costs | `6` | Navigation |
| `jump_tab_6` | Jump to History | `7` | Navigation |
| `jump_tab_7` | Jump to MCP | `8` | Navigation |
| `jump_tab_8` | Jump to Analytics | `9` | Navigation |
| `toggle_help` | Toggle help modal | `?` | UI |
| `show_command_palette` | Show command palette | `:` | UI |
| `close_modal` | Close current modal/dialog | `Esc` | UI |

## Detailed Descriptions

### Global Actions

#### quit
- **Purpose**: Exit ccboard cleanly
- **Default**: `q`
- **Behavior**: Initiates application shutdown
- **Future**: May add confirmation dialog for unsaved work

#### force_quit
- **Purpose**: Exit immediately without prompts
- **Default**: `Ctrl+Q`
- **Behavior**: Bypasses all confirmation dialogs
- **Use Case**: When confirmation dialogs are blocking

#### refresh
- **Purpose**: Reload data from ~/.claude
- **Default**: `F5`
- **Behavior**:
  - Re-reads sessions, stats, settings
  - Preserves session content cache
  - Updates UI with fresh data
- **Use Case**: After external changes to ~/.claude

#### force_refresh
- **Purpose**: Hard refresh with cache clear
- **Default**: `Ctrl+R`
- **Behavior**:
  - Clears session content cache
  - Re-reads all data from disk
  - Shows toast notification
- **Use Case**: When cache may be stale or corrupted

#### theme_toggle
- **Purpose**: Switch between Dark and Light themes
- **Default**: `Ctrl+T`
- **Behavior**:
  - Toggles `app.color_scheme`
  - Shows toast with theme name
  - Updates UI colors immediately
- **Future**: Persist theme preference to settings.json

### Navigation Actions

#### next_tab
- **Purpose**: Move to the next tab
- **Default**: `Tab`
- **Behavior**: Cycles through tabs (wraps to first after last)
- **Cycle Order**: Dashboard → Sessions → Config → Hooks → Agents → Costs → History → MCP → Analytics → Dashboard

#### prev_tab
- **Purpose**: Move to the previous tab
- **Default**: `Shift+Tab`
- **Behavior**: Cycles through tabs in reverse (wraps to last from first)

#### jump_tab_0 through jump_tab_8
- **Purpose**: Jump directly to specific tabs
- **Defaults**: `1` through `9`
- **Mapping**:
  - `jump_tab_0` (1) → Dashboard
  - `jump_tab_1` (2) → Sessions
  - `jump_tab_2` (3) → Config
  - `jump_tab_3` (4) → Hooks
  - `jump_tab_4` (5) → Agents
  - `jump_tab_5` (6) → Costs
  - `jump_tab_6` (7) → History
  - `jump_tab_7` (8) → MCP
  - `jump_tab_8` (9) → Analytics
- **Use Case**: Quick navigation to frequently used tabs

### UI Actions

#### toggle_help
- **Purpose**: Show/hide help modal
- **Default**: `?` (Shift+/)
- **Behavior**:
  - If hidden: shows help modal overlay
  - If visible: hides help modal
- **Content**: Dynamically generated from active keybindings

#### show_command_palette
- **Purpose**: Open command palette (k9s-style)
- **Default**: `:` (Shift+;)
- **Behavior**: Opens fuzzy-search command interface
- **Available Commands**:
  - `:q` or `:quit` → Quit
  - `:refresh` → Refresh data
  - `:goto <tab>` → Navigate to tab
  - `:search <query>` → Search (future)

#### close_modal
- **Purpose**: Close active modal/dialog
- **Default**: `Esc`
- **Behavior**:
  - Closes help modal if visible
  - Closes command palette if visible
  - Future: Close confirmation dialogs
- **Scope**: Only affects modals, not main application

## Usage Examples

### Example 1: Remap Quit to Ctrl+C

```json
{
  "keybindings": {
    "Ctrl+C": "quit"
  }
}
```

**Result**: `Ctrl+C` quits, `q` still works (default preserved)

### Example 2: Swap Tab Navigation

```json
{
  "keybindings": {
    "n": "next_tab",
    "p": "prev_tab"
  }
}
```

**Result**: `n` for next, `p` for previous (Vim-style)

### Example 3: Use Function Keys for Tabs

```json
{
  "keybindings": {
    "F1": "jump_tab_0",
    "F2": "jump_tab_1",
    "F3": "jump_tab_2",
    "F4": "jump_tab_3",
    "F5": "jump_tab_4",
    "F6": "jump_tab_5",
    "F7": "jump_tab_6",
    "F8": "jump_tab_7",
    "F9": "jump_tab_8"
  }
}
```

**Result**: Function keys jump to tabs, number keys freed for other use

### Example 4: Emacs-style Bindings

```json
{
  "keybindings": {
    "Ctrl+X Ctrl+C": "quit",
    "Ctrl+L": "refresh",
    "Ctrl+H": "toggle_help"
  }
}
```

**Note**: Multi-key sequences (e.g., `Ctrl+X Ctrl+C`) not yet supported in Phase 4.

## Implementation Details

### Enum Definition

```rust
pub enum KeyAction {
    Quit,
    ForceQuit,
    Refresh,
    ForceRefresh,
    ThemeToggle,
    NextTab,
    PrevTab,
    JumpTab0,
    JumpTab1,
    JumpTab2,
    JumpTab3,
    JumpTab4,
    JumpTab5,
    JumpTab6,
    JumpTab7,
    JumpTab8,
    ToggleHelp,
    ShowCommandPalette,
    CloseModal,
}
```

### Action Name Parsing

Action names are **case-insensitive** but use **snake_case** convention:

```rust
KeyAction::from_name("quit")        // ✅ Correct
KeyAction::from_name("QUIT")        // ✅ Works (lowercased internally)
KeyAction::from_name("Quit")        // ✅ Works
KeyAction::from_name("force_quit")  // ✅ Correct
KeyAction::from_name("ForceQuit")   // ❌ Fails (use snake_case)
```

### Extensibility

To add new actions (future development):

1. Add enum variant to `KeyAction`
2. Add case to `from_name()` match
3. Add case to `name()` match
4. Add case to `description()` match
5. Add case to `handle_action()` in `app.rs`
6. Update documentation
7. Add unit test

## Future Actions (Planned)

These actions are planned for future phases:

| Action | Description | Phase |
|--------|-------------|-------|
| `search` | Global search across tabs | Phase 5 |
| `filter` | Filter current list | Phase 5 |
| `sort` | Change sort order | Phase 5 |
| `export` | Export current view | Phase 5 |
| `copy` | Copy to clipboard | Phase 5 |
| `edit` | Edit current item | Phase 5 |
| `delete` | Delete current item | Phase 6 |
| `undo` | Undo last action | Phase 6 |
| `redo` | Redo last undone action | Phase 6 |

## Related Documentation

- [KEYBINDINGS.md](./KEYBINDINGS.md) - Full keybindings guide
- [TESTING_KEYBINDINGS.md](./TESTING_KEYBINDINGS.md) - Testing instructions
- [examples/settings-keybindings.json](../examples/settings-keybindings.json) - Default config
- [examples/settings-custom-keys.json](../examples/settings-custom-keys.json) - Custom example
