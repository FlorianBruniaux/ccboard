//! TUI Application state and event loop

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
        }
    }

    /// Handle keyboard input
    /// Returns true if the key was handled as a global key
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode;

        match key {
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
