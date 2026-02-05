# Custom Keybindings

ccboard supports custom keyboard shortcuts via `settings.json` configuration.

## Configuration

Add a `keybindings` object to your settings.json file:

```json
{
  "keybindings": {
    "key_string": "action_name"
  }
}
```

### Settings Hierarchy

Keybindings follow the 4-layer settings merge:

1. **Global**: `~/.claude/settings.json`
2. **Global Local**: `~/.claude/settings.local.json`
3. **Project**: `.claude/settings.json`
4. **Project Local**: `.claude/settings.local.json` (highest priority)

Custom bindings override defaults. Bindings from higher priority layers override lower ones.

## Key String Format

### Basic Keys

```json
"q": "quit",
"r": "refresh",
"5": "jump_tab_4"
```

### Modifiers

Supported modifiers: `Ctrl`, `Shift`, `Alt`, `Cmd` (macOS)

```json
"Ctrl+Q": "force_quit",
"Shift+Tab": "prev_tab",
"Alt+Enter": "some_action",
"Ctrl+Shift+T": "some_action"
```

**Note**: Case-insensitive (`ctrl+q` = `CTRL+Q`)

### Special Keys

```json
"Tab": "next_tab",
"Esc": "close_modal",
"Enter": "confirm",
"Backspace": "delete",
"Delete": "remove",
"Space": "toggle",
"F5": "refresh",
"F1": "jump_tab_0"
```

Supported special keys:
- `Tab`, `BackTab`
- `Enter`, `Return`
- `Esc`, `Escape`
- `Backspace`
- `Delete`, `Del`
- `Space`
- Arrow keys: `Up`, `Down`, `Left`, `Right`
- Navigation: `Home`, `End`, `PageUp`, `PageDown`
- Function keys: `F1`-`F12`
- `Insert`

## Available Actions

### Global Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `quit` | Quit application | `q` |
| `force_quit` | Force quit without confirmation | `Ctrl+Q` |
| `refresh` | Refresh data from disk | `F5` |
| `force_refresh` | Force refresh + clear cache | `Ctrl+R` |
| `theme_toggle` | Toggle theme (Dark/Light) | `Ctrl+T` |
| `next_tab` | Navigate to next tab | `Tab` |
| `prev_tab` | Navigate to previous tab | `Shift+Tab` |
| `toggle_help` | Toggle help modal | `?` |
| `show_command_palette` | Show command palette | `:` |
| `close_modal` | Close current modal/dialog | `Esc` |

### Tab Jump Actions

| Action | Description | Default Key |
|--------|-------------|-------------|
| `jump_tab_0` | Jump to Dashboard | `1` |
| `jump_tab_1` | Jump to Sessions | `2` |
| `jump_tab_2` | Jump to Config | `3` |
| `jump_tab_3` | Jump to Hooks | `4` |
| `jump_tab_4` | Jump to Agents | `5` |
| `jump_tab_5` | Jump to Costs | `6` |
| `jump_tab_6` | Jump to History | `7` |
| `jump_tab_7` | Jump to MCP | `8` |
| `jump_tab_8` | Jump to Analytics | `9` |

## Examples

### Default Keybindings

```json
{
  "keybindings": {
    "q": "quit",
    "Ctrl+Q": "force_quit",
    "F5": "refresh",
    "Ctrl+R": "force_refresh",
    "Ctrl+T": "theme_toggle",
    "Tab": "next_tab",
    "Shift+Tab": "prev_tab",
    "1": "jump_tab_0",
    "2": "jump_tab_1",
    "3": "jump_tab_2",
    "4": "jump_tab_3",
    "5": "jump_tab_4",
    "6": "jump_tab_5",
    "7": "jump_tab_6",
    "8": "jump_tab_7",
    "9": "jump_tab_8",
    "?": "toggle_help",
    ":": "show_command_palette",
    "Esc": "close_modal"
  }
}
```

### Custom Keybindings (Vim-style)

```json
{
  "keybindings": {
    "Ctrl+Q": "quit",
    "Ctrl+Shift+Q": "force_quit",
    "r": "refresh",
    "R": "force_refresh",
    "t": "theme_toggle",
    "n": "next_tab",
    "p": "prev_tab",
    "F1": "jump_tab_0",
    "F2": "jump_tab_1",
    "F3": "jump_tab_2",
    "F4": "jump_tab_3",
    "F5": "jump_tab_4",
    "F6": "jump_tab_5",
    "F7": "jump_tab_6",
    "F8": "jump_tab_7",
    "F9": "jump_tab_8",
    "h": "toggle_help",
    "Ctrl+P": "show_command_palette",
    "Esc": "close_modal"
  }
}
```

### Emacs-style Navigation

```json
{
  "keybindings": {
    "Ctrl+X Ctrl+C": "quit",
    "Ctrl+G": "close_modal",
    "Ctrl+L": "refresh",
    "Ctrl+X t": "theme_toggle",
    "Ctrl+X n": "next_tab",
    "Ctrl+X p": "prev_tab",
    "Ctrl+H": "toggle_help"
  }
}
```

**Note**: Multi-key sequences like `Ctrl+X Ctrl+C` are not yet supported in Phase 4. Planned for Phase 6.

## Testing Your Configuration

1. Create or edit `~/.claude/settings.json` or `.claude/settings.json`
2. Add your `keybindings` object
3. Restart ccboard
4. Press `?` (or your custom help key) to see active keybindings

## Troubleshooting

### Keybinding Not Working

1. **Check for typos**: Keys are case-insensitive but must be valid
2. **Verify action name**: Action names are case-sensitive (`quit` not `Quit`)
3. **Check conflicts**: Some keys may be captured by terminal emulator
4. **View logs**: ccboard warns about malformed bindings on startup

### Common Mistakes

```json
// ❌ Wrong: Invalid action name
"q": "exit"  // Should be "quit"

// ❌ Wrong: Invalid key format
"ctrl-q": "quit"  // Should use "+" not "-"

// ❌ Wrong: Unsupported key
"F13": "refresh"  // Only F1-F12 supported

// ✅ Correct
"Ctrl+Q": "quit"
```

### Terminal Compatibility

Some key combinations may be captured by your terminal emulator:
- `Ctrl+S` (XOFF - flow control)
- `Ctrl+Z` (suspend)
- `Cmd+Q` (macOS quit)

Use alternative bindings if conflicts occur.

## Help Modal

The help modal (`?` by default) dynamically displays your custom keybindings. This ensures documentation always matches your active configuration.

## Implementation Status

**Phase 4 (Current)**:
- ✅ Custom keybindings via settings.json
- ✅ HashMap configuration format
- ✅ Support for modifiers (Ctrl, Shift, Alt, Cmd)
- ✅ Support for standard keys (a-z, 0-9, F1-F12, special keys)
- ✅ Fallback to defaults
- ✅ Dynamic help modal
- ✅ Settings merge across 4 layers

**Future Enhancements (Phase 6+)**:
- Multi-key sequences (e.g., `Ctrl+X Ctrl+C`)
- Tab-specific keybindings
- Contextual keybindings (modal-specific)
- Key recording wizard
- Conflict detection and warnings

## Related Documentation

- [Configuration](./CONFIGURATION.md) - Full settings.json guide
- [Architecture](../ARCHITECTURE.md) - Settings merge implementation
- [Examples](../examples/) - Example configurations
