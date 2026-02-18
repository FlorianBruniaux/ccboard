//! TUI Application state and event loop

use crate::components::{CommandPalette, ConfirmDialog, HelpModal, Spinner, ToastManager};
use crate::keybindings::{KeyAction, KeyBindings};
use ccboard_core::models::config::ColorScheme;
use ccboard_core::{DataEvent, DataStore};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Active tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Dashboard,
    Sessions,
    Config,
    Hooks,
    Agents,
    Costs,
    History,
    Mcp,
    Analytics,
    Plugins,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Dashboard,
            Tab::Sessions,
            Tab::Config,
            Tab::Hooks,
            Tab::Agents,
            Tab::Costs,
            Tab::History,
            Tab::Mcp,
            Tab::Analytics,
            Tab::Plugins,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Dashboard => 0,
            Tab::Sessions => 1,
            Tab::Config => 2,
            Tab::Hooks => 3,
            Tab::Agents => 4,
            Tab::Costs => 5,
            Tab::History => 6,
            Tab::Mcp => 7,
            Tab::Analytics => 8,
            Tab::Plugins => 9,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Tab::Dashboard,
            1 => Tab::Sessions,
            2 => Tab::Config,
            3 => Tab::Hooks,
            4 => Tab::Agents,
            5 => Tab::Costs,
            6 => Tab::History,
            7 => Tab::Mcp,
            8 => Tab::Analytics,
            9 => Tab::Plugins,
            _ => Tab::Dashboard,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Sessions => "Sessions",
            Tab::Config => "Config",
            Tab::Hooks => "Hooks",
            Tab::Agents => "Capabilities", // Changed from "Agents"
            Tab::Costs => "Costs",
            Tab::History => "History",
            Tab::Mcp => "MCP",
            Tab::Analytics => "Analytics",
            Tab::Plugins => "Plugins",
        }
    }

    pub fn shortcut(&self) -> char {
        match self {
            Tab::Dashboard => '1',
            Tab::Sessions => '2',
            Tab::Config => '3',
            Tab::Hooks => '4',
            Tab::Agents => '5',
            Tab::Costs => '6',
            Tab::History => '7',
            Tab::Mcp => '8',
            Tab::Analytics => '9',
            Tab::Plugins => '0',
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Dashboard => "📊",
            Tab::Sessions => "💬",
            Tab::Config => "⚙️",
            Tab::Hooks => "🪝",
            Tab::Agents => "🤖",
            Tab::Costs => "💰",
            Tab::History => "📜",
            Tab::Mcp => "🔌",
            Tab::Analytics => "📈",
            Tab::Plugins => "🎁", // Gift box for plugins
        }
    }
}

/// TUI Application state
pub struct App {
    /// Data store reference
    pub store: Arc<DataStore>,

    /// Event receiver for data updates
    pub event_rx: broadcast::Receiver<DataEvent>,

    /// Currently active tab
    pub active_tab: Tab,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Whether data needs refresh
    pub needs_refresh: bool,

    /// Error/warning message to display
    pub status_message: Option<String>,

    /// Command palette (k9s-style `:` prefix)
    pub command_palette: CommandPalette,

    /// Help modal (toggle with `?`)
    pub help_modal: HelpModal,

    /// Loading state (true during initial_load)
    pub is_loading: bool,

    /// Loading message to display
    pub loading_message: Option<String>,

    /// Loading spinner
    pub spinner: Spinner,

    /// Toast notification manager
    pub toast_manager: ToastManager,

    /// Confirmation dialog (for future destructive actions)
    pub confirm_dialog: ConfirmDialog,

    /// Cached live sessions (refreshed every 2s)
    pub live_sessions_cache: Vec<ccboard_core::LiveSession>,

    /// Last time live sessions were refreshed
    pub last_live_refresh: std::time::Instant,

    /// Search history for History tab (last 50 searches, newest first)
    pub search_history: VecDeque<String>,

    /// Current color scheme (Dark/Light)
    pub color_scheme: ColorScheme,

    /// Custom keybindings
    pub keybindings: KeyBindings,
}

impl App {
    pub fn new(store: Arc<DataStore>) -> Self {
        let event_rx = store.event_bus().subscribe();

        // Load keybindings from settings
        let mut keybindings = KeyBindings::new();
        let settings = store.settings();
        if let Some(custom_keybindings) = &settings.merged.keybindings {
            keybindings.load_custom(custom_keybindings);
        }

        // Load persisted color scheme (fallback to Dark if missing)
        let prefs = store.load_preferences();

        Self {
            store,
            event_rx,
            active_tab: Tab::Dashboard,
            should_quit: false,
            needs_refresh: true,
            status_message: None,
            command_palette: CommandPalette::new(),
            help_modal: HelpModal::new(),
            is_loading: true,
            loading_message: Some("Loading sessions...".to_string()),
            spinner: Spinner::new(),
            toast_manager: ToastManager::new(),
            confirm_dialog: ConfirmDialog::new("Confirm", "Are you sure?"),
            live_sessions_cache: Vec::new(),
            last_live_refresh: std::time::Instant::now(),
            search_history: VecDeque::with_capacity(50),
            color_scheme: prefs.color_scheme,
            keybindings,
        }
    }

    /// Update loading message
    pub fn set_loading_message(&mut self, message: impl Into<String>) {
        self.loading_message = Some(message.into());
    }

    /// Mark loading as complete
    pub fn complete_loading(&mut self) {
        self.is_loading = false;
        self.loading_message = None;
    }

    /// Handle keyboard input
    /// Returns true if the key was handled as a global key
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> bool {
        use crate::components::command_palette::CommandAction;

        // If command palette is visible, handle keys there first
        if self.command_palette.is_visible() {
            if let Some(action) = self.command_palette.handle_key(key) {
                // Execute the command action
                match action {
                    CommandAction::Quit => self.should_quit = true,
                    CommandAction::RefreshData => self.needs_refresh = true,
                    CommandAction::GoToTab(tab) => self.active_tab = tab,
                    CommandAction::Search(query) => {
                        // TODO: Implement search functionality when History/Sessions support it
                        self.status_message = Some(format!("Search: {}", query));
                    }
                    CommandAction::ShowHelp => {
                        // Show palette with empty query to list all commands
                        self.command_palette.show();
                    }
                }
            }
            return true;
        }

        // Try to match key to action via keybindings
        if let Some(action) = self.keybindings.get_action(key, modifiers) {
            self.handle_action(action);
            return true;
        }

        // Key not handled by global keybindings
        false
    }

    /// Handle a keybinding action
    fn handle_action(&mut self, action: KeyAction) {
        match action {
            KeyAction::Quit | KeyAction::ForceQuit => {
                self.should_quit = true;
            }
            KeyAction::Refresh => {
                self.needs_refresh = true;
            }
            KeyAction::ForceRefresh => {
                self.needs_refresh = true;
                self.store.clear_session_content_cache();
                self.info_toast("♻ Reloading data...");
            }
            KeyAction::ThemeToggle => {
                self.color_scheme = match self.color_scheme {
                    ColorScheme::Dark => ColorScheme::Light,
                    ColorScheme::Light => ColorScheme::Dark,
                };
                let theme_name = match self.color_scheme {
                    ColorScheme::Dark => "Dark",
                    ColorScheme::Light => "Light",
                };
                // Persist the new color scheme
                let prefs = ccboard_core::preferences::CcboardPreferences {
                    color_scheme: self.color_scheme,
                };
                if let Err(e) = self.store.save_preferences(&prefs) {
                    tracing::warn!(error = %e, "Failed to persist color scheme preference");
                }
                self.info_toast(format!("Theme: {}", theme_name));
            }
            KeyAction::NextTab => {
                self.next_tab();
            }
            KeyAction::PrevTab => {
                self.prev_tab();
            }
            KeyAction::JumpTab0 => self.active_tab = Tab::from_index(0),
            KeyAction::JumpTab1 => self.active_tab = Tab::from_index(1),
            KeyAction::JumpTab2 => self.active_tab = Tab::from_index(2),
            KeyAction::JumpTab3 => self.active_tab = Tab::from_index(3),
            KeyAction::JumpTab4 => self.active_tab = Tab::from_index(4),
            KeyAction::JumpTab5 => self.active_tab = Tab::from_index(5),
            KeyAction::JumpTab6 => self.active_tab = Tab::from_index(6),
            KeyAction::JumpTab7 => self.active_tab = Tab::from_index(7),
            KeyAction::JumpTab8 => self.active_tab = Tab::from_index(8),
            KeyAction::JumpTab9 => self.active_tab = Tab::from_index(9),
            KeyAction::ToggleHelp => {
                self.help_modal.toggle();
            }
            KeyAction::ShowCommandPalette => {
                self.command_palette.show();
            }
            KeyAction::CloseModal => {
                if self.help_modal.is_visible() {
                    self.help_modal.hide();
                } else if self.command_palette.is_visible() {
                    self.command_palette.hide();
                }
            }
        }
    }

    fn next_tab(&mut self) {
        let idx = self.active_tab.index();
        self.active_tab = Tab::from_index((idx + 1) % Tab::all().len());
    }

    fn prev_tab(&mut self) {
        let idx = self.active_tab.index();
        self.active_tab = Tab::from_index((idx + Tab::all().len() - 1) % Tab::all().len());
    }

    /// Add success toast notification
    pub fn success_toast(&mut self, message: impl Into<String>) {
        self.toast_manager
            .push(crate::components::Toast::success(message));
    }

    /// Add error toast notification
    pub fn error_toast(&mut self, message: impl Into<String>) {
        self.toast_manager
            .push(crate::components::Toast::error(message));
    }

    /// Add warning toast notification
    pub fn warning_toast(&mut self, message: impl Into<String>) {
        self.toast_manager
            .push(crate::components::Toast::warning(message));
    }

    /// Add info toast notification
    pub fn info_toast(&mut self, message: impl Into<String>) {
        self.toast_manager
            .push(crate::components::Toast::info(message));
    }

    /// Check for data events (non-blocking)
    pub fn poll_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DataEvent::StatsUpdated
                | DataEvent::SessionCreated(_)
                | DataEvent::SessionUpdated(_)
                | DataEvent::ConfigChanged(_)
                | DataEvent::AnalyticsUpdated => {
                    self.needs_refresh = true;
                }
                DataEvent::WatcherError(msg) => {
                    self.status_message = Some(format!("Watcher error: {}", msg));
                }
                DataEvent::LoadCompleted => {
                    self.needs_refresh = true;
                }
            }
        }
    }

    /// Refresh live sessions cache if needed (every 2 seconds when on Sessions tab)
    pub fn refresh_live_sessions_if_needed(&mut self) {
        // Only refresh if on Sessions tab
        if self.active_tab != Tab::Sessions {
            return;
        }

        // Check if 2 seconds have elapsed since last refresh
        let now = std::time::Instant::now();
        if now.duration_since(self.last_live_refresh).as_secs() >= 2 {
            self.live_sessions_cache = self.store.live_sessions();
            self.last_live_refresh = now;
        }
    }

    /// Get cached live sessions
    pub fn live_sessions(&self) -> &[ccboard_core::LiveSession] {
        &self.live_sessions_cache
    }
}
