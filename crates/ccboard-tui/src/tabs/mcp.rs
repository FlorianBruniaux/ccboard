//! MCP tab - Dedicated MCP server management interface
//!
//! Features:
//! - Dual-pane layout (server list | details)
//! - Status detection via process listing (Unix only)
//! - File operations (edit config, reveal file)
//! - Empty state handling
//! - Error popup for failed operations
//!
//! Keybindings:
//! - h/j/k/l or arrows: Navigation
//! - Enter: Focus detail pane
//! - e: Edit claude_desktop_config.json
//! - o: Reveal config file in file manager
//! - r: Refresh status detection
//! - Esc: Close error popup

use crate::empty_state;
use crate::theme::ServerStatusColor;
use ccboard_core::parsers::mcp_config::{McpConfig, McpServer};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashMap;
use std::time::Instant;

/// Which pane has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    List,
    Detail,
}

/// Server runtime status
#[derive(Debug, Clone, PartialEq, Eq)]
enum ServerStatus {
    /// Server is running with given PID
    Running(u32),
    /// Server is not running
    Stopped,
    /// Status detection failed or unsupported platform
    Unknown,
}

impl ServerStatus {
    /// Get display icon and color for this status (using unified theme)
    fn icon(&self) -> (&'static str, Color) {
        let color_status = match self {
            ServerStatus::Running(_) => ServerStatusColor::Running,
            ServerStatus::Stopped => ServerStatusColor::Stopped,
            ServerStatus::Unknown => ServerStatusColor::Unknown,
        };
        (color_status.icon(), color_status.to_color())
    }

    /// Get status text for display
    fn text(&self) -> String {
        match self {
            ServerStatus::Running(pid) => format!("Running (PID: {})", pid),
            ServerStatus::Stopped => "Stopped".to_string(),
            ServerStatus::Unknown => "Unknown".to_string(),
        }
    }
}

/// MCP Tab state
pub struct McpTab {
    /// Server list selection state
    server_list_state: ListState,
    /// Which pane has focus
    focus: Focus,
    /// Cached status for each server
    status_cache: HashMap<String, ServerStatus>,
    /// When status was last refreshed
    last_refresh: Instant,
    /// Error message to display in popup (if any)
    error_message: Option<String>,
}

impl Default for McpTab {
    fn default() -> Self {
        Self::new()
    }
}

impl McpTab {
    /// Create a new MCP tab
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            server_list_state: state,
            focus: Focus::List,
            status_cache: HashMap::new(),
            last_refresh: Instant::now(),
            error_message: None,
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode, mcp_config: Option<&McpConfig>) {
        // Close error popup
        if key == KeyCode::Esc && self.error_message.is_some() {
            self.error_message = None;
            return;
        }

        let server_count = mcp_config.map(|c| c.servers.len()).unwrap_or(0);

        match key {
            // Focus switching (always allowed)
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = Focus::List;
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => {
                self.focus = Focus::Detail;
            }

            // Server selection (when list focused and servers exist)
            KeyCode::Up | KeyCode::Char('k')
                if matches!(self.focus, Focus::List) && server_count > 0 =>
            {
                let current = self.server_list_state.selected().unwrap_or(0);
                let new_idx = current.saturating_sub(1);
                self.server_list_state.select(Some(new_idx));
            }
            KeyCode::Down | KeyCode::Char('j')
                if matches!(self.focus, Focus::List) && server_count > 0 =>
            {
                let current = self.server_list_state.selected().unwrap_or(0);
                let new_idx = (current + 1).min(server_count - 1);
                self.server_list_state.select(Some(new_idx));
            }

            // File operations
            KeyCode::Char('e') => {
                self.handle_edit_config();
            }
            KeyCode::Char('o') => {
                self.handle_reveal_config();
            }

            // Refresh status
            KeyCode::Char('r') => {
                self.refresh_status(mcp_config);
            }

            _ => {}
        }
    }

    /// Render the MCP tab
    pub fn render(&mut self, frame: &mut Frame, area: Rect, mcp_config: Option<&McpConfig>) {
        // Dual-pane layout: 35% list | 65% details
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        // Render server list
        self.render_server_list(frame, chunks[0], mcp_config);

        // Render server detail
        self.render_server_detail(frame, chunks[1], mcp_config);

        // Render error popup if present (overlay)
        if self.error_message.is_some() {
            self.render_error_popup(frame, area);
        }
    }

    /// Render server list pane
    fn render_server_list(&mut self, frame: &mut Frame, area: Rect, mcp_config: Option<&McpConfig>) {
        let is_focused = matches!(self.focus, Focus::List);
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        // Handle no config case
        if mcp_config.is_none() {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " MCP Servers (0) ",
                    Style::default().fg(Color::White).bold(),
                ));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let empty = empty_state::no_mcp_config();
            frame.render_widget(empty, inner);
            return;
        }

        let config = mcp_config.unwrap();
        let server_count = config.servers.len();

        // Handle empty servers case
        if server_count == 0 {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " MCP Servers (0) ",
                    Style::default().fg(Color::White).bold(),
                ));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let empty = empty_state::no_mcp_servers();
            frame.render_widget(empty, inner);
            return;
        }

        // Build server list items
        let mut servers: Vec<(&String, &McpServer)> = config.servers.iter().collect();
        servers.sort_by_key(|(name, _)| *name);

        let items: Vec<ListItem> = servers
            .iter()
            .map(|(name, server)| {
                let status = self
                    .status_cache
                    .get(*name)
                    .cloned()
                    .unwrap_or(ServerStatus::Unknown);
                let (icon, color) = status.icon();

                let cmd_short = format!("{} {}", server.command, server.args.join(" "))
                    .chars()
                    .take(40)
                    .collect::<String>();

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                        Span::styled(name.to_string(), Style::default().fg(Color::White).bold()),
                    ]),
                    Line::from(Span::styled(
                        format!("   {}", cmd_short),
                        Style::default().fg(Color::DarkGray),
                    )),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" MCP Servers ({}) ", server_count),
                Style::default().fg(Color::White).bold(),
            ));

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_stateful_widget(list, area, &mut self.server_list_state);
    }

    /// Render server detail pane
    fn render_server_detail(&mut self, frame: &mut Frame, area: Rect, mcp_config: Option<&McpConfig>) {
        let is_focused = matches!(self.focus, Focus::Detail);
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let selected_server = self.get_selected_server(mcp_config);

        let title = if let Some((name, _)) = selected_server {
            format!(" Server Details: {} ", name)
        } else {
            " Server Details ".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(Color::White).bold()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if selected_server.is_none() {
            let empty = Paragraph::new("Select a server to view details")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            frame.render_widget(empty, inner);
            return;
        }

        let (name, server) = selected_server.unwrap();
        let status = self
            .status_cache
            .get(name)
            .cloned()
            .unwrap_or(ServerStatus::Unknown);

        let mut lines = vec![];

        // Status line
        let (status_icon, status_color) = status.icon();
        let status_text = status.text();
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(
                format!("{} ", status_icon),
                Style::default().fg(status_color).bold(),
            ),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]));
        lines.push(Line::from(""));

        // Command
        lines.push(Line::from(Span::styled(
            "Command:",
            Style::default().fg(Color::Yellow).bold(),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", server.command),
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Arguments
        if !server.args.is_empty() {
            lines.push(Line::from(Span::styled(
                "Arguments:",
                Style::default().fg(Color::Yellow).bold(),
            )));
            for arg in &server.args {
                lines.push(Line::from(Span::styled(
                    format!("  {}", arg),
                    Style::default().fg(Color::White),
                )));
            }
            lines.push(Line::from(""));
        }

        // Environment variables
        lines.push(Line::from(Span::styled(
            "Environment:",
            Style::default().fg(Color::Yellow).bold(),
        )));
        if server.env.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (key, value) in &server.env {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}=", key), Style::default().fg(Color::Cyan)),
                    Span::styled(value.clone(), Style::default().fg(Color::White)),
                ]));
            }
        }
        lines.push(Line::from(""));

        // Config file path
        let config_path = dirs::home_dir()
            .map(|h| h.join(".claude/claude_desktop_config.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.claude/claude_desktop_config.json"));
        lines.push(Line::from(Span::styled(
            "Config File:",
            Style::default().fg(Color::Yellow).bold(),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", config_path.display()),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        // Actions
        lines.push(Line::from(Span::styled(
            "Actions:",
            Style::default().fg(Color::Yellow).bold(),
        )));
        lines.push(Line::from(Span::styled(
            "  [e] Edit config  [o] Reveal file  [r] Refresh status",
            Style::default().fg(Color::DarkGray),
        )));

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false });
        frame.render_widget(paragraph, inner);
    }

    /// Get the currently selected server
    fn get_selected_server<'a>(
        &self,
        mcp_config: Option<&'a McpConfig>,
    ) -> Option<(&'a String, &'a McpServer)> {
        let idx = self.server_list_state.selected()?;
        let config = mcp_config?;
        let servers: Vec<_> = config.servers.iter().collect();
        servers.get(idx).copied()
    }

    /// Refresh status detection for all servers
    fn refresh_status(&mut self, mcp_config: Option<&McpConfig>) {
        let Some(config) = mcp_config else {
            return;
        };

        self.status_cache.clear();

        for (name, server) in &config.servers {
            let status = Self::detect_server_status(server);
            self.status_cache.insert(name.clone(), status);
        }

        self.last_refresh = Instant::now();
    }

    /// Detect if a server is currently running
    #[cfg(unix)]
    fn detect_server_status(server: &McpServer) -> ServerStatus {
        // Extract package name from command
        // Example: npx -y @modelcontextprotocol/server-playwright → server-playwright
        let package = server
            .args
            .iter()
            .find(|arg| arg.starts_with('@') || arg.contains('/'))
            .map(|arg| {
                arg.rsplit('/')
                    .next()
                    .unwrap_or(arg)
                    .split('@')
                    .next()
                    .unwrap_or(arg)
            })
            .unwrap_or(&server.command);

        // Run ps aux | grep <package>
        let output = std::process::Command::new("ps").args(["aux"]).output();

        let Ok(output) = output else {
            return ServerStatus::Unknown;
        };

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Find matching process (exclude grep itself)
        for line in stdout.lines() {
            if line.contains(package) && !line.contains("grep") {
                // Extract PID (second column in ps aux)
                if let Some(pid_str) = line.split_whitespace().nth(1) {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return ServerStatus::Running(pid);
                    }
                }
                return ServerStatus::Running(0); // PID extraction failed but process found
            }
        }

        ServerStatus::Stopped
    }

    /// Detect server status (Windows fallback)
    #[cfg(not(unix))]
    fn detect_server_status(_server: &McpServer) -> ServerStatus {
        ServerStatus::Unknown
    }

    /// Handle 'e' key - edit config file
    fn handle_edit_config(&mut self) {
        let config_path = dirs::home_dir()
            .map(|h| h.join(".claude/claude_desktop_config.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.claude/claude_desktop_config.json"));

        if let Err(e) = crate::editor::open_in_editor(&config_path) {
            self.error_message = Some(format!("Failed to open editor: {}", e));
        }
    }

    /// Handle 'o' key - reveal config file
    fn handle_reveal_config(&mut self) {
        let config_path = dirs::home_dir()
            .map(|h| h.join(".claude/claude_desktop_config.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("~/.claude/claude_desktop_config.json"));

        if let Err(e) = crate::editor::reveal_in_file_manager(&config_path) {
            self.error_message = Some(format!("Failed to reveal file: {}", e));
        }
    }

    /// Render error popup overlay
    fn render_error_popup(&self, frame: &mut Frame, area: Rect) {
        if self.error_message.is_none() {
            return;
        }

        // Center popup (60% width, 30% height)
        let popup_width = (area.width as f32 * 0.6) as u16;
        let popup_height = (area.height as f32 * 0.3).max(8.0) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " Error ",
                Style::default().fg(Color::Red).bold(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");
        let lines = vec![
            Line::from(Span::styled(
                error_text,
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_icon() {
        assert_eq!(ServerStatus::Running(123).icon(), ("●", Color::Green));
        assert_eq!(ServerStatus::Stopped.icon(), ("○", Color::Red));
        assert_eq!(ServerStatus::Unknown.icon(), ("?", Color::DarkGray));
    }

    #[test]
    fn test_new_tab() {
        let tab = McpTab::new();
        assert_eq!(tab.focus, Focus::List);
        assert!(tab.status_cache.is_empty());
        assert!(tab.error_message.is_none());
    }

    #[test]
    fn test_focus_switching() {
        let mut tab = McpTab::new();
        assert_eq!(tab.focus, Focus::List);

        tab.handle_key(KeyCode::Right, None);
        assert_eq!(tab.focus, Focus::Detail);

        tab.handle_key(KeyCode::Left, None);
        assert_eq!(tab.focus, Focus::List);
    }
}
