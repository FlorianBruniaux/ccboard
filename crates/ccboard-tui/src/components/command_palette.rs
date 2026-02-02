use crate::app::Tab;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Action triggered by a command
#[derive(Debug, Clone)]
pub enum CommandAction {
    /// Navigate to a specific tab
    GoToTab(Tab),
    /// Search with a query string
    Search(String),
    /// Refresh all data from ~/.claude
    RefreshData,
    /// Quit the application
    Quit,
    /// Show help/all commands
    ShowHelp,
}

/// A single command definition
#[derive(Debug, Clone)]
pub struct Command {
    /// Primary command name (e.g., "quit", "dashboard")
    pub name: String,
    /// Keyboard shortcut (e.g., "q", "1", "r")
    pub shortcut: String,
    /// Human-readable description
    pub description: String,
    /// Action to execute
    pub action: CommandAction,
    /// Tags for fuzzy matching (e.g., ["exit", "close"] for quit)
    pub tags: Vec<String>,
}

impl Command {
    /// Check if command matches the query (fuzzy substring matching)
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Match against name, shortcut, description, or tags
        self.name.to_lowercase().contains(&query_lower)
            || self.shortcut.to_lowercase().contains(&query_lower)
            || self.description.to_lowercase().contains(&query_lower)
            || self.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
    }
}

/// k9s-style command palette with `:` prefix and fuzzy matching
pub struct CommandPalette {
    /// User's search query
    query: String,
    /// Filtered commands matching the query
    results: Vec<Command>,
    /// All available commands
    commands: Vec<Command>,
    /// Currently selected result index
    selected: usize,
    /// Whether the palette is visible
    visible: bool,
    /// List state for rendering
    list_state: ListState,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPalette {
    /// Create a new command palette with all available commands
    pub fn new() -> Self {
        let commands = Self::build_command_registry();
        let results = commands.clone();

        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            query: String::new(),
            results,
            commands,
            selected: 0,
            visible: false,
            list_state,
        }
    }

    /// Build the registry of all available commands
    fn build_command_registry() -> Vec<Command> {
        vec![
            // Quit
            Command {
                name: "quit".to_string(),
                shortcut: "q".to_string(),
                description: "Quit ccboard".to_string(),
                action: CommandAction::Quit,
                tags: vec!["exit".to_string(), "close".to_string()],
            },
            // Refresh
            Command {
                name: "refresh".to_string(),
                shortcut: "r".to_string(),
                description: "Reload data from ~/.claude".to_string(),
                action: CommandAction::RefreshData,
                tags: vec!["reload".to_string(), "update".to_string()],
            },
            // Help
            Command {
                name: "help".to_string(),
                shortcut: "?".to_string(),
                description: "Show all commands".to_string(),
                action: CommandAction::ShowHelp,
                tags: vec!["commands".to_string(), "list".to_string()],
            },
            // Tab navigation
            Command {
                name: "dashboard".to_string(),
                shortcut: "1".to_string(),
                description: "Go to Dashboard tab".to_string(),
                action: CommandAction::GoToTab(Tab::Dashboard),
                tags: vec!["home".to_string(), "overview".to_string()],
            },
            Command {
                name: "sessions".to_string(),
                shortcut: "2".to_string(),
                description: "Go to Sessions tab".to_string(),
                action: CommandAction::GoToTab(Tab::Sessions),
                tags: vec!["projects".to_string(), "conversations".to_string()],
            },
            Command {
                name: "config".to_string(),
                shortcut: "3".to_string(),
                description: "Go to Config tab".to_string(),
                action: CommandAction::GoToTab(Tab::Config),
                tags: vec!["settings".to_string(), "configuration".to_string()],
            },
            Command {
                name: "hooks".to_string(),
                shortcut: "4".to_string(),
                description: "Go to Hooks tab".to_string(),
                action: CommandAction::GoToTab(Tab::Hooks),
                tags: vec!["scripts".to_string(), "automation".to_string()],
            },
            Command {
                name: "agents".to_string(),
                shortcut: "5".to_string(),
                description: "Go to Agents tab".to_string(),
                action: CommandAction::GoToTab(Tab::Agents),
                tags: vec!["commands".to_string(), "skills".to_string()],
            },
            Command {
                name: "costs".to_string(),
                shortcut: "6".to_string(),
                description: "Go to Costs tab".to_string(),
                action: CommandAction::GoToTab(Tab::Costs),
                tags: vec!["billing".to_string(), "usage".to_string(), "money".to_string()],
            },
            Command {
                name: "history".to_string(),
                shortcut: "7".to_string(),
                description: "Go to History tab".to_string(),
                action: CommandAction::GoToTab(Tab::History),
                tags: vec!["timeline".to_string(), "activity".to_string()],
            },
            Command {
                name: "mcp".to_string(),
                shortcut: "8".to_string(),
                description: "Go to MCP tab".to_string(),
                action: CommandAction::GoToTab(Tab::Mcp),
                tags: vec!["servers".to_string(), "plugins".to_string()],
            },
        ]
    }

    /// Show the command palette
    pub fn show(&mut self) {
        self.visible = true;
        self.query.clear();
        self.results = self.commands.clone();
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    /// Hide the command palette
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the currently selected command, if any
    pub fn selected_command(&self) -> Option<&Command> {
        self.results.get(self.selected)
    }

    /// Handle key input for the command palette
    pub fn handle_key(&mut self, key: KeyCode) -> Option<CommandAction> {
        match key {
            KeyCode::Esc => {
                self.hide();
                None
            }
            KeyCode::Enter => {
                let action = self.selected_command().map(|cmd| cmd.action.clone());
                self.hide();
                action
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.list_state.select(Some(self.selected));
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected < self.results.len().saturating_sub(1) {
                    self.selected += 1;
                    self.list_state.select(Some(self.selected));
                }
                None
            }
            KeyCode::Char(c) => {
                self.query.push(c);
                self.filter_results();
                None
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.filter_results();
                None
            }
            _ => None,
        }
    }

    /// Filter results based on current query
    fn filter_results(&mut self) {
        if self.query.is_empty() {
            self.results = self.commands.clone();
        } else {
            self.results = self.commands
                .iter()
                .filter(|cmd| cmd.matches(&self.query))
                .cloned()
                .collect();
        }

        // Reset selection to first result
        self.selected = 0;
        self.list_state.select(Some(0));
    }

    /// Render the command palette as a centered overlay
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Create centered overlay (60% width, 50% height)
        let popup_area = Self::centered_rect(60, 50, area);

        // Split into input and results areas
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input box
                Constraint::Min(0),     // Results list
            ])
            .split(popup_area);

        // Render input box
        let input_text = format!(":{}", self.query);
        let input = Paragraph::new(input_text)
            .style(Style::default().fg(Color::Cyan))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette")
                    .border_style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(input, chunks[0]);

        // Render results list
        let items: Vec<ListItem> = self
            .results
            .iter()
            .map(|cmd| {
                let shortcut = Span::styled(
                    format!(":{} ", cmd.shortcut),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
                let name = Span::styled(
                    format!("{:<15}", cmd.name),
                    Style::default().fg(Color::White),
                );
                let desc = Span::styled(
                    &cmd.description,
                    Style::default().fg(Color::Gray),
                );

                ListItem::new(Line::from(vec![shortcut, name, desc]))
            })
            .collect();

        let results_widget = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("{} commands", self.results.len()))
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(results_widget, chunks[1], &mut self.list_state);
    }

    /// Create a centered rectangle with given percentage width/height
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_matches() {
        let cmd = Command {
            name: "dashboard".to_string(),
            shortcut: "1".to_string(),
            description: "Go to Dashboard tab".to_string(),
            action: CommandAction::GoToTab(Tab::Dashboard),
            tags: vec!["home".to_string(), "overview".to_string()],
        };

        assert!(cmd.matches("dash"));
        assert!(cmd.matches("board"));
        assert!(cmd.matches("1"));
        assert!(cmd.matches("home"));
        assert!(cmd.matches("overview"));
        assert!(!cmd.matches("xyz"));
    }

    #[test]
    fn test_filter_results() {
        let mut palette = CommandPalette::new();

        // Initially all commands shown
        assert!(!palette.results.is_empty());

        // Filter by "dash"
        palette.query = "dash".to_string();
        palette.filter_results();
        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].name, "dashboard");

        // Filter by "quit"
        palette.query = "quit".to_string();
        palette.filter_results();
        assert_eq!(palette.results.len(), 1);
        assert_eq!(palette.results[0].name, "quit");
    }

    #[test]
    fn test_show_hide() {
        let mut palette = CommandPalette::new();

        assert!(!palette.is_visible());

        palette.show();
        assert!(palette.is_visible());
        assert_eq!(palette.query, "");

        palette.hide();
        assert!(!palette.is_visible());
    }
}
