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
use crate::theme::{Palette, ServerStatusColor};
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
    fn icon(&self, scheme: ccboard_core::models::config::ColorScheme) -> (&'static str, Color) {
        let color_status = match self {
            ServerStatus::Running(_) => ServerStatusColor::Running,
            ServerStatus::Stopped => ServerStatusColor::Stopped,
            ServerStatus::Unknown => ServerStatusColor::Unknown,
        };
        (color_status.icon(), color_status.to_color(scheme))
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
    /// Copy success message to display
    copy_message: Option<String>,
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
            copy_message: None,
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode, mcp_config: Option<&McpConfig>) {
        // Close error/copy message popup
        if key == KeyCode::Esc {
            if self.error_message.is_some() {
                self.error_message = None;
                return;
            }
            if self.copy_message.is_some() {
                self.copy_message = None;
                return;
            }
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

            // Copy command to clipboard
            KeyCode::Char('y') => {
                self.handle_copy_command(mcp_config);
            }

            // Refresh status
            KeyCode::Char('r') => {
                self.refresh_status(mcp_config);
            }

            // Page navigation
            KeyCode::PageUp if matches!(self.focus, Focus::List) && server_count > 0 => {
                let current = self.server_list_state.selected().unwrap_or(0);
                let new_idx = current.saturating_sub(10);
                self.server_list_state.select(Some(new_idx));
            }
            KeyCode::PageDown if matches!(self.focus, Focus::List) && server_count > 0 => {
                let current = self.server_list_state.selected().unwrap_or(0);
                let new_idx = (current + 10).min(server_count - 1);
                self.server_list_state.select(Some(new_idx));
            }

            _ => {}
        }
    }

    /// Render the MCP tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        mcp_config: Option<&McpConfig>,
        scheme: ccboard_core::models::config::ColorScheme,
    ) {
        let p = Palette::new(scheme);

        // Dual-pane layout: 35% list | 65% details
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        // Render server list
        self.render_server_list(frame, chunks[0], mcp_config, scheme, &p);

        // Render server detail
        self.render_server_detail(frame, chunks[1], mcp_config, scheme, &p);

        // Render copy message if present (overlay)
        if self.copy_message.is_some() {
            self.render_copy_message(frame, area, &p);
        }

        // Render error popup if present (overlay)
        if self.error_message.is_some() {
            self.render_error_popup(frame, area, &p);
        }
    }

    /// Render server list pane
    fn render_server_list(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        mcp_config: Option<&McpConfig>,
        scheme: ccboard_core::models::config::ColorScheme,
        p: &Palette,
    ) {
        let is_focused = matches!(self.focus, Focus::List);
        let border_color = if is_focused { p.focus } else { p.border };

        // Handle no config case
        if mcp_config.is_none() {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " MCP Servers (0) ",
                    Style::default().fg(p.fg).bold(),
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
                    Style::default().fg(p.fg).bold(),
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
                let (icon, color) = status.icon(scheme);

                let cmd_short = server
                    .display_command()
                    .chars()
                    .take(40)
                    .collect::<String>();

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                        Span::styled(name.to_string(), Style::default().fg(p.fg).bold()),
                    ]),
                    Line::from(Span::styled(
                        format!("   {}", cmd_short),
                        Style::default().fg(p.muted),
                    )),
                ])
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" MCP Servers ({}) ", server_count),
                Style::default().fg(p.fg).bold(),
            ));

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(p.muted));

        frame.render_stateful_widget(list, area, &mut self.server_list_state);
    }

    /// Render server detail pane
    fn render_server_detail(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        mcp_config: Option<&McpConfig>,
        scheme: ccboard_core::models::config::ColorScheme,
        p: &Palette,
    ) {
        let is_focused = matches!(self.focus, Focus::Detail);
        let border_color = if is_focused { p.focus } else { p.border };

        let selected_server = self.get_selected_server(mcp_config);

        let title = if let Some((name, _)) = selected_server {
            format!(" Server Details: {} ", name)
        } else {
            " Server Details ".to_string()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title,
                Style::default().fg(p.fg).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if selected_server.is_none() {
            let empty = Paragraph::new("Select a server to view details")
                .style(Style::default().fg(p.muted))
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

        // Description (if known server type)
        if let Some(desc) = Self::get_server_description(name, server) {
            lines.push(Line::from(Span::styled(
                desc,
                Style::default().fg(p.muted).italic(),
            )));
            lines.push(Line::from(""));
        }

        // Status line
        let (status_icon, status_color) = status.icon(scheme);
        let status_text = status.text();
        lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().fg(p.warning).bold()),
            Span::styled(
                format!("{} ", status_icon),
                Style::default().fg(status_color).bold(),
            ),
            Span::styled(status_text, Style::default().fg(status_color)),
        ]));
        lines.push(Line::from(""));

        // Command or URL
        if server.is_http() {
            lines.push(Line::from(Span::styled(
                "Type:",
                Style::default().fg(p.warning).bold(),
            )));
            lines.push(Line::from(Span::styled(
                "  HTTP Server",
                Style::default().fg(p.focus),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "URL:",
                Style::default().fg(p.warning).bold(),
            )));
            lines.push(Line::from(Span::styled(
                format!("  {}", server.url.as_deref().unwrap_or("(unknown)")),
                Style::default().fg(p.fg),
            )));
            lines.push(Line::from(""));

            // Headers
            if let Some(headers) = &server.headers {
                if !headers.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "Headers:",
                        Style::default().fg(p.warning).bold(),
                    )));
                    for (key, value) in headers {
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("  {} = ", key),
                                Style::default().fg(p.focus).bold(),
                            ),
                            Span::styled(value.clone(), Style::default().fg(p.fg)),
                        ]));
                    }
                    lines.push(Line::from(""));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "Type:",
                Style::default().fg(p.warning).bold(),
            )));
            lines.push(Line::from(Span::styled(
                "  Stdio Server",
                Style::default().fg(p.focus),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "Command:",
                Style::default().fg(p.warning).bold(),
            )));
            lines.push(Line::from(Span::styled(
                format!(
                    "  {}",
                    if server.command.is_empty() {
                        "(unknown)"
                    } else {
                        &server.command
                    }
                ),
                Style::default().fg(p.fg),
            )));
            lines.push(Line::from(""));
        }

        // Arguments with syntax highlighting (only for stdio servers)
        if !server.is_http() && !server.args.is_empty() {
            lines.push(Line::from(Span::styled(
                "Arguments:",
                Style::default().fg(p.warning).bold(),
            )));
            for arg in &server.args {
                let spans = Self::highlight_arg(arg);
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
        }

        // Environment variables with masking for sensitive values
        lines.push(Line::from(Span::styled(
            "Environment:",
            Style::default().fg(p.warning).bold(),
        )));
        if server.env.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(p.muted),
            )));
        } else {
            // Sort env vars for consistent display
            let mut env_vars: Vec<_> = server.env.iter().collect();
            env_vars.sort_by_key(|(k, _)| *k);

            for (key, value) in env_vars {
                let masked_value = Self::mask_sensitive_env(key, value);
                let value_color = if masked_value.contains("••••") {
                    p.muted // Masked values in muted
                } else {
                    p.fg
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {} = ", key),
                        Style::default().fg(p.focus).bold(),
                    ),
                    Span::styled(masked_value, Style::default().fg(value_color)),
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
            Style::default().fg(p.warning).bold(),
        )));
        lines.push(Line::from(Span::styled(
            format!("  {}", config_path.display()),
            Style::default().fg(p.muted),
        )));
        lines.push(Line::from(""));

        // Actions
        lines.push(Line::from(Span::styled(
            "Actions:",
            Style::default().fg(p.warning).bold(),
        )));
        lines.push(Line::from(Span::styled(
            "  [y] Copy command  [e] Edit config  [o] Reveal file  [r] Refresh",
            Style::default().fg(p.muted),
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

    /// Handle 'y' key - copy command to clipboard
    fn handle_copy_command(&mut self, mcp_config: Option<&McpConfig>) {
        let Some((name, server)) = self.get_selected_server(mcp_config) else {
            self.error_message = Some("No server selected".to_string());
            return;
        };

        // Build full command string
        let command = server.display_command();

        // Copy to clipboard
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&command) {
                    self.error_message = Some(format!("Failed to copy: {}", e));
                } else {
                    self.copy_message = Some(format!("✓ Copied to clipboard: {}", name));
                }
            }
            Err(e) => {
                self.error_message = Some(format!("Clipboard error: {}", e));
            }
        }
    }

    /// Render error popup overlay
    fn render_error_popup(&self, frame: &mut Frame, area: Rect, p: &Palette) {
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
            .border_style(Style::default().fg(p.error))
            .title(Span::styled(
                " Error ",
                Style::default().fg(p.error).bold(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");
        let lines = vec![
            Line::from(Span::styled(error_text, Style::default().fg(p.fg))),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(p.muted),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }

    /// Render copy success message overlay
    fn render_copy_message(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        if self.copy_message.is_none() {
            return;
        }

        // Bottom notification (80% width, 3 lines height)
        let msg_width = (area.width as f32 * 0.8) as u16;
        let msg_height = 3;
        let msg_x = (area.width.saturating_sub(msg_width)) / 2;
        let msg_y = area.height.saturating_sub(msg_height + 2);

        let msg_area = Rect {
            x: area.x + msg_x,
            y: area.y + msg_y,
            width: msg_width,
            height: msg_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.success));

        let inner = block.inner(msg_area);
        frame.render_widget(block, msg_area);

        let msg_text = self.copy_message.as_deref().unwrap_or("");
        let paragraph = Paragraph::new(msg_text)
            .style(Style::default().fg(p.success))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, inner);
    }

    /// Highlight argument with syntax coloring
    ///
    /// Detects:
    /// - Flags: --flag, -f (Cyan)
    /// - Paths: /absolute, ./relative (Green)
    /// - URLs: http://, https:// (Magenta)
    /// - Values: normal (White)
    fn highlight_arg(arg: &str) -> Vec<Span<'static>> {
        let mut spans = vec![Span::raw("  ")];

        // Flag detection (--flag or -f)
        if arg.starts_with("--")
            || (arg.starts_with('-') && !arg.starts_with("--") && arg.len() == 2)
        {
            spans.push(Span::styled(
                arg.to_string(),
                Style::default().fg(Color::Cyan).bold(),
            ));
        }
        // URL detection
        else if arg.starts_with("http://") || arg.starts_with("https://") {
            spans.push(Span::styled(
                arg.to_string(),
                Style::default().fg(Color::Magenta),
            ));
        }
        // Path detection (absolute or relative)
        else if arg.starts_with('/') || arg.starts_with("./") || arg.starts_with("../") {
            spans.push(Span::styled(
                arg.to_string(),
                Style::default().fg(Color::Green),
            ));
        }
        // Regular value
        else {
            spans.push(Span::styled(
                arg.to_string(),
                Style::default().fg(Color::White),
            ));
        }

        spans
    }

    /// Mask sensitive environment variable values
    ///
    /// Detects common patterns like API_KEY, TOKEN, SECRET, PASSWORD
    /// and masks the value showing only first 4 and last 4 characters
    fn mask_sensitive_env(key: &str, value: &str) -> String {
        let key_lower = key.to_lowercase();
        let is_sensitive = key_lower.contains("key")
            || key_lower.contains("token")
            || key_lower.contains("secret")
            || key_lower.contains("password")
            || key_lower.contains("api");

        if is_sensitive && value.len() > 8 {
            format!("{}••••{}", &value[..4], &value[value.len() - 4..])
        } else {
            value.to_string()
        }
    }

    /// Detect known MCP server types and return description
    fn get_server_description(name: &str, server: &McpServer) -> Option<String> {
        // Check command/args for known servers
        let cmd_str = server.display_command();

        if name.contains("playwright") || cmd_str.contains("playwright") {
            Some("Browser automation and web testing server".to_string())
        } else if name.contains("serena") || cmd_str.contains("serena") {
            Some("Code search and semantic analysis server".to_string())
        } else if name.contains("filesystem") || cmd_str.contains("filesystem") {
            Some("File system operations server".to_string())
        } else if name.contains("sequential") || cmd_str.contains("sequential") {
            Some("Multi-step reasoning and analysis server".to_string())
        } else if name.contains("context7") || cmd_str.contains("context7") {
            Some("Documentation and context retrieval server".to_string())
        } else if name.contains("perplexity") || cmd_str.contains("perplexity") {
            Some("Web search and research server".to_string())
        } else if name.contains("claude-in-chrome") || cmd_str.contains("claude-in-chrome") {
            Some("Browser automation via Chrome extension".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_icon() {
        use ccboard_core::models::config::ColorScheme;
        assert_eq!(
            ServerStatus::Running(123).icon(ColorScheme::Dark),
            ("●", Color::Green)
        );
        assert_eq!(
            ServerStatus::Stopped.icon(ColorScheme::Dark),
            ("○", Color::Red)
        );
        assert_eq!(
            ServerStatus::Unknown.icon(ColorScheme::Dark),
            ("?", Color::DarkGray)
        );
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
