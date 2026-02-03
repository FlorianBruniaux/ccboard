//! TUI Application state and event loop

use crate::components::{CommandPalette, Spinner};
use ccboard_core::{DataEvent, DataStore};
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
            _ => Tab::Dashboard,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Sessions => "Sessions",
            Tab::Config => "Config",
            Tab::Hooks => "Hooks",
            Tab::Agents => "Agents",
            Tab::Costs => "Costs",
            Tab::History => "History",
            Tab::Mcp => "MCP",
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
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tab::Dashboard => "◆",
            Tab::Sessions => "●",
            Tab::Config => "⚙",
            Tab::Hooks => "▣",
            Tab::Agents => "◉",
            Tab::Costs => "💰",
            Tab::History => "⏱",
            Tab::Mcp => "◈",
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

    /// Loading state (true during initial_load)
    pub is_loading: bool,

    /// Loading message to display
    pub loading_message: Option<String>,

    /// Loading spinner
    pub spinner: Spinner,
}

impl App {
    pub fn new(store: Arc<DataStore>) -> Self {
        let event_rx = store.event_bus().subscribe();

        Self {
            store,
            event_rx,
            active_tab: Tab::Dashboard,
            should_quit: false,
            needs_refresh: true,
            status_message: None,
            command_palette: CommandPalette::new(),
            is_loading: true,
            loading_message: Some("Loading sessions...".to_string()),
            spinner: Spinner::new(),
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
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> bool {
        use crate::components::command_palette::CommandAction;
        use crossterm::event::KeyCode;

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

        // Global keybindings (when palette is not active)
        match key {
            KeyCode::Char(':') => {
                // Show command palette
                self.command_palette.show();
                true
            }
            KeyCode::Char('q') => {
                self.should_quit = true;
                true
            }
            KeyCode::F(5) => {
                // F5 for refresh (was 'r', but 'r' conflicts with search input)
                self.needs_refresh = true;
                true
            }
            KeyCode::Tab => {
                self.next_tab();
                true
            }
            KeyCode::BackTab => {
                self.prev_tab();
                true
            }
            KeyCode::Char(c) if ('1'..='8').contains(&c) => {
                let idx = (c as usize) - ('1' as usize);
                self.active_tab = Tab::from_index(idx);
                true
            }
            _ => false,
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

    /// Check for data events (non-blocking)
    pub fn poll_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                DataEvent::StatsUpdated
                | DataEvent::SessionCreated(_)
                | DataEvent::SessionUpdated(_)
                | DataEvent::ConfigChanged(_) => {
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
}
