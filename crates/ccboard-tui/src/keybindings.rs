//! Custom keybindings system for ccboard TUI
//!
//! Supports:
//! - Customizable keybindings via settings.json
//! - Modifier keys: Ctrl, Shift, Alt, Cmd
//! - Standard keys: a-z, 0-9, F1-F12, Tab, Enter, Esc, etc.
//! - Fallback to defaults if custom bindings missing or malformed
//! - Reverse lookup for help modal display

use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

/// Actions that can be triggered by keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAction {
    /// Quit application (with confirmation if needed)
    Quit,
    /// Force quit without confirmation
    ForceQuit,
    /// Refresh data from disk
    Refresh,
    /// Force refresh and clear caches
    ForceRefresh,
    /// Toggle color scheme (Dark/Light)
    ThemeToggle,
    /// Navigate to next tab
    NextTab,
    /// Navigate to previous tab
    PrevTab,
    /// Jump to specific tab by index (0-8)
    JumpTab0,
    JumpTab1,
    JumpTab2,
    JumpTab3,
    JumpTab4,
    JumpTab5,
    JumpTab6,
    JumpTab7,
    JumpTab8,
    JumpTab9,
    /// Toggle help modal
    ToggleHelp,
    /// Show command palette
    ShowCommandPalette,
    /// Close current modal/dialog
    CloseModal,
}

impl KeyAction {
    /// Get all possible actions
    pub fn all() -> &'static [KeyAction] {
        &[
            KeyAction::Quit,
            KeyAction::ForceQuit,
            KeyAction::Refresh,
            KeyAction::ForceRefresh,
            KeyAction::ThemeToggle,
            KeyAction::NextTab,
            KeyAction::PrevTab,
            KeyAction::JumpTab0,
            KeyAction::JumpTab1,
            KeyAction::JumpTab2,
            KeyAction::JumpTab3,
            KeyAction::JumpTab4,
            KeyAction::JumpTab5,
            KeyAction::JumpTab6,
            KeyAction::JumpTab7,
            KeyAction::JumpTab8,
            KeyAction::JumpTab9,
            KeyAction::ToggleHelp,
            KeyAction::ShowCommandPalette,
            KeyAction::CloseModal,
        ]
    }

    /// Get action name for settings.json
    pub fn name(&self) -> &'static str {
        match self {
            KeyAction::Quit => "quit",
            KeyAction::ForceQuit => "force_quit",
            KeyAction::Refresh => "refresh",
            KeyAction::ForceRefresh => "force_refresh",
            KeyAction::ThemeToggle => "theme_toggle",
            KeyAction::NextTab => "next_tab",
            KeyAction::PrevTab => "prev_tab",
            KeyAction::JumpTab0 => "jump_tab_0",
            KeyAction::JumpTab1 => "jump_tab_1",
            KeyAction::JumpTab2 => "jump_tab_2",
            KeyAction::JumpTab3 => "jump_tab_3",
            KeyAction::JumpTab4 => "jump_tab_4",
            KeyAction::JumpTab5 => "jump_tab_5",
            KeyAction::JumpTab6 => "jump_tab_6",
            KeyAction::JumpTab7 => "jump_tab_7",
            KeyAction::JumpTab8 => "jump_tab_8",
            KeyAction::JumpTab9 => "jump_tab_9",
            KeyAction::ToggleHelp => "toggle_help",
            KeyAction::ShowCommandPalette => "show_command_palette",
            KeyAction::CloseModal => "close_modal",
        }
    }

    /// Get human-readable description for help modal
    pub fn description(&self) -> &'static str {
        match self {
            KeyAction::Quit => "Quit application",
            KeyAction::ForceQuit => "Force quit without confirmation",
            KeyAction::Refresh => "Refresh data",
            KeyAction::ForceRefresh => "Force refresh + clear cache",
            KeyAction::ThemeToggle => "Toggle theme (Dark/Light)",
            KeyAction::NextTab => "Next tab",
            KeyAction::PrevTab => "Previous tab",
            KeyAction::JumpTab0 => "Jump to Dashboard",
            KeyAction::JumpTab1 => "Jump to Sessions",
            KeyAction::JumpTab2 => "Jump to Config",
            KeyAction::JumpTab3 => "Jump to Hooks",
            KeyAction::JumpTab4 => "Jump to Agents",
            KeyAction::JumpTab5 => "Jump to Costs",
            KeyAction::JumpTab6 => "Jump to History",
            KeyAction::JumpTab7 => "Jump to MCP",
            KeyAction::JumpTab8 => "Jump to Analytics",
            KeyAction::JumpTab9 => "Jump to Plugins",
            KeyAction::ToggleHelp => "Toggle help modal",
            KeyAction::ShowCommandPalette => "Show command palette",
            KeyAction::CloseModal => "Close modal/dialog",
        }
    }

    /// Parse action from string (from settings.json)
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "quit" => Some(KeyAction::Quit),
            "force_quit" => Some(KeyAction::ForceQuit),
            "refresh" => Some(KeyAction::Refresh),
            "force_refresh" => Some(KeyAction::ForceRefresh),
            "theme_toggle" => Some(KeyAction::ThemeToggle),
            "next_tab" => Some(KeyAction::NextTab),
            "prev_tab" => Some(KeyAction::PrevTab),
            "jump_tab_0" => Some(KeyAction::JumpTab0),
            "jump_tab_1" => Some(KeyAction::JumpTab1),
            "jump_tab_2" => Some(KeyAction::JumpTab2),
            "jump_tab_3" => Some(KeyAction::JumpTab3),
            "jump_tab_4" => Some(KeyAction::JumpTab4),
            "jump_tab_5" => Some(KeyAction::JumpTab5),
            "jump_tab_6" => Some(KeyAction::JumpTab6),
            "jump_tab_7" => Some(KeyAction::JumpTab7),
            "jump_tab_8" => Some(KeyAction::JumpTab8),
            "jump_tab_9" => Some(KeyAction::JumpTab9),
            "toggle_help" => Some(KeyAction::ToggleHelp),
            "show_command_palette" => Some(KeyAction::ShowCommandPalette),
            "close_modal" => Some(KeyAction::CloseModal),
            _ => None,
        }
    }
}

/// Key with modifiers for lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct KeyWithMods {
    code: KeyCode,
    modifiers: KeyModifiers,
}

/// Keybindings system
pub struct KeyBindings {
    /// Default keybindings (immutable)
    defaults: HashMap<KeyWithMods, KeyAction>,
    /// Custom keybindings from settings.json (override defaults)
    custom: HashMap<KeyWithMods, KeyAction>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBindings {
    /// Create new keybindings with defaults
    pub fn new() -> Self {
        let mut defaults = HashMap::new();

        // Quit
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::Quit,
        );

        // Force quit (Ctrl+Q)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
            },
            KeyAction::ForceQuit,
        );

        // Refresh (F5)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::F(5),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::Refresh,
        );

        // Force refresh (Ctrl+R)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
            },
            KeyAction::ForceRefresh,
        );

        // Theme toggle (Ctrl+T)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('t'),
                modifiers: KeyModifiers::CONTROL,
            },
            KeyAction::ThemeToggle,
        );

        // Next tab (Tab)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::NextTab,
        );

        // Previous tab (Shift+Tab)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::BackTab,
                modifiers: KeyModifiers::SHIFT,
            },
            KeyAction::PrevTab,
        );

        // Jump to tabs (1-9)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('1'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab0,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('2'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab1,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('3'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab2,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('4'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab3,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('5'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab4,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('6'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab5,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('7'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab6,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('8'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab7,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('9'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab8,
        );
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('0'),
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::JumpTab9,
        );

        // Toggle help (?)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char('?'),
                modifiers: KeyModifiers::SHIFT,
            },
            KeyAction::ToggleHelp,
        );

        // Show command palette (:)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Char(':'),
                modifiers: KeyModifiers::SHIFT,
            },
            KeyAction::ShowCommandPalette,
        );

        // Close modal (Esc)
        defaults.insert(
            KeyWithMods {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            },
            KeyAction::CloseModal,
        );

        Self {
            defaults,
            custom: HashMap::new(),
        }
    }

    /// Load custom keybindings from settings.json
    ///
    /// Format: `{"keybindings": {"Ctrl+Q": "quit", "F5": "refresh", ...}}`
    pub fn load_custom(&mut self, keybindings: &HashMap<String, String>) {
        for (key_str, action_str) in keybindings {
            // Parse key string
            let key_with_mods = match parse_key(key_str) {
                Ok(k) => k,
                Err(e) => {
                    eprintln!("Warning: Failed to parse keybinding '{}': {}", key_str, e);
                    continue;
                }
            };

            // Parse action
            let action = match KeyAction::from_name(action_str) {
                Some(a) => a,
                None => {
                    eprintln!(
                        "Warning: Unknown action '{}' for key '{}'",
                        action_str, key_str
                    );
                    continue;
                }
            };

            // Store custom binding
            self.custom.insert(key_with_mods, action);
        }
    }

    /// Get action for a key press
    ///
    /// Returns None if no binding found (key should be passed to active tab)
    pub fn get_action(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<KeyAction> {
        let key = KeyWithMods { code, modifiers };

        // Custom bindings take precedence
        if let Some(action) = self.custom.get(&key) {
            return Some(*action);
        }

        // Fall back to defaults
        self.defaults.get(&key).copied()
    }

    /// Get key string for an action (for help modal)
    ///
    /// Returns the first matching key (custom > default)
    pub fn get_key_for_action(&self, action: KeyAction) -> Option<String> {
        // Check custom first
        for (key, act) in &self.custom {
            if *act == action {
                return Some(format_key(key.code, key.modifiers));
            }
        }

        // Fall back to default
        for (key, act) in &self.defaults {
            if *act == action {
                return Some(format_key(key.code, key.modifiers));
            }
        }

        None
    }
}

/// Parse key string from settings.json
///
/// Examples:
/// - "Ctrl+Q" → KeyCode::Char('q') + CONTROL
/// - "F5" → KeyCode::F(5)
/// - "Tab" → KeyCode::Tab
/// - "shift+tab" → KeyCode::BackTab + SHIFT
/// - "alt+enter" → KeyCode::Enter + ALT
fn parse_key(s: &str) -> Result<KeyWithMods, String> {
    let s = s.trim();
    let parts: Vec<String> = s.split('+').map(|p| p.trim().to_lowercase()).collect();

    if parts.is_empty() {
        return Err("Empty key string".to_string());
    }

    // Parse modifiers
    let mut modifiers = KeyModifiers::NONE;
    let key_part = if parts.len() > 1 {
        for modifier in &parts[..parts.len() - 1] {
            match modifier.as_str() {
                "ctrl" | "control" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                "cmd" | "meta" => {
                    // Map to SUPER on Linux/Windows, CONTROL on macOS
                    #[cfg(target_os = "macos")]
                    {
                        modifiers |= KeyModifiers::SUPER;
                    }
                    #[cfg(not(target_os = "macos"))]
                    {
                        modifiers |= KeyModifiers::CONTROL;
                    }
                }
                _ => return Err(format!("Unknown modifier: {}", modifier)),
            }
        }
        &parts[parts.len() - 1]
    } else {
        &parts[0]
    };

    // Parse key
    let code = match key_part.as_str() {
        // Special keys
        "tab" => {
            // If Shift modifier is present, return BackTab
            if modifiers.contains(KeyModifiers::SHIFT) {
                KeyCode::BackTab
            } else {
                KeyCode::Tab
            }
        }
        "backtab" => {
            // BackTab is Shift+Tab in crossterm
            modifiers |= KeyModifiers::SHIFT;
            KeyCode::BackTab
        }
        "enter" | "return" => KeyCode::Enter,
        "esc" | "escape" => KeyCode::Esc,
        "backspace" => KeyCode::Backspace,
        "delete" | "del" => KeyCode::Delete,
        "space" => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" | "pgup" => KeyCode::PageUp,
        "pagedown" | "pgdn" => KeyCode::PageDown,
        "insert" | "ins" => KeyCode::Insert,

        // Function keys
        k if k.starts_with('f') => {
            let num_str = &k[1..];
            let num: u8 = num_str
                .parse()
                .map_err(|_| format!("Invalid F-key: {}", k))?;
            if !(1..=12).contains(&num) {
                return Err(format!("F-key out of range (1-12): F{}", num));
            }
            KeyCode::F(num)
        }

        // Single character
        k if k.len() == 1 => {
            let ch = k.chars().next().unwrap();
            KeyCode::Char(ch)
        }

        _ => return Err(format!("Unknown key: {}", key_part)),
    };

    Ok(KeyWithMods { code, modifiers })
}

/// Format key for display in help modal
fn format_key(code: KeyCode, modifiers: KeyModifiers) -> String {
    let mut parts = Vec::new();

    // Add modifiers
    if modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }
    if modifiers.contains(KeyModifiers::SUPER) {
        #[cfg(target_os = "macos")]
        parts.push("Cmd");
        #[cfg(not(target_os = "macos"))]
        parts.push("Win");
    }

    // Add key
    let key_str = match code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_uppercase().to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PgUp".to_string(),
        KeyCode::PageDown => "PgDn".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        _ => "Unknown".to_string(),
    };

    parts.push(&key_str);
    parts.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_key() {
        let key = parse_key("q").unwrap();
        assert_eq!(key.code, KeyCode::Char('q'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_key_with_ctrl() {
        let key = parse_key("Ctrl+Q").unwrap();
        assert_eq!(key.code, KeyCode::Char('q'));
        assert_eq!(key.modifiers, KeyModifiers::CONTROL);
    }

    #[test]
    fn test_parse_function_key() {
        let key = parse_key("F5").unwrap();
        assert_eq!(key.code, KeyCode::F(5));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_parse_multiple_modifiers() {
        let key = parse_key("Ctrl+Shift+T").unwrap();
        assert_eq!(key.code, KeyCode::Char('t'));
        assert!(key.modifiers.contains(KeyModifiers::CONTROL));
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_tab() {
        let key = parse_key("Tab").unwrap();
        assert_eq!(key.code, KeyCode::Tab);
    }

    #[test]
    fn test_parse_shift_tab() {
        let key = parse_key("Shift+Tab").unwrap();
        assert_eq!(key.code, KeyCode::BackTab);
        assert!(key.modifiers.contains(KeyModifiers::SHIFT));
    }

    #[test]
    fn test_parse_case_insensitive() {
        let key1 = parse_key("ctrl+q").unwrap();
        let key2 = parse_key("CTRL+Q").unwrap();
        assert_eq!(key1.code, key2.code);
        assert_eq!(key1.modifiers, key2.modifiers);
    }

    #[test]
    fn test_keybindings_defaults() {
        let kb = KeyBindings::new();

        // Test quit
        let action = kb.get_action(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(action, Some(KeyAction::Quit));

        // Test force quit
        let action = kb.get_action(KeyCode::Char('q'), KeyModifiers::CONTROL);
        assert_eq!(action, Some(KeyAction::ForceQuit));

        // Test refresh
        let action = kb.get_action(KeyCode::F(5), KeyModifiers::NONE);
        assert_eq!(action, Some(KeyAction::Refresh));
    }

    #[test]
    fn test_keybindings_custom_override() {
        let mut kb = KeyBindings::new();

        // Verify default
        let action = kb.get_action(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(action, Some(KeyAction::Quit));

        // Load custom binding
        let mut custom = HashMap::new();
        custom.insert("q".to_string(), "refresh".to_string());
        kb.load_custom(&custom);

        // Verify override
        let action = kb.get_action(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(action, Some(KeyAction::Refresh));
    }

    #[test]
    fn test_keybindings_reverse_lookup() {
        let kb = KeyBindings::new();

        let key_str = kb.get_key_for_action(KeyAction::Quit);
        assert_eq!(key_str, Some("Q".to_string()));

        let key_str = kb.get_key_for_action(KeyAction::ForceQuit);
        assert_eq!(key_str, Some("Ctrl+Q".to_string()));
    }

    #[test]
    fn test_action_from_name() {
        assert_eq!(KeyAction::from_name("quit"), Some(KeyAction::Quit));
        assert_eq!(KeyAction::from_name("QUIT"), Some(KeyAction::Quit));
        assert_eq!(
            KeyAction::from_name("force_quit"),
            Some(KeyAction::ForceQuit)
        );
        assert_eq!(KeyAction::from_name("unknown"), None);
    }
}
