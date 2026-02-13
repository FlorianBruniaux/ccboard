//! Conversation viewer - Displays full session content with syntax highlighting

use crate::widgets::ToolCallsViewer;
use ccboard_core::models::{ConversationMessage, MessageRole};
use ccboard_core::DataStore;
use chrono::{DateTime, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use std::collections::HashMap;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Message filtering options
#[derive(Debug, Clone)]
pub struct MessageFilter {
    /// Show user messages
    pub show_user: bool,
    /// Show assistant messages
    pub show_assistant: bool,
    /// Show system messages
    pub show_system: bool,
    /// Show only messages with tool calls
    pub tools_only: bool,
    /// Filter by timestamp range (optional)
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self {
            show_user: true,
            show_assistant: true,
            show_system: true,
            tools_only: false,
            start_time: None,
            end_time: None,
        }
    }
}

impl MessageFilter {
    /// Check if a message passes all active filters
    pub fn matches(&self, msg: &ConversationMessage) -> bool {
        // Role filter
        let role_match = match msg.role {
            MessageRole::User => self.show_user,
            MessageRole::Assistant => self.show_assistant,
            MessageRole::System => self.show_system,
        };

        if !role_match {
            return false;
        }

        // Tools filter (only applies to assistant messages)
        if self.tools_only {
            // Check if content contains tool usage indicators
            let has_tools = msg.content.contains("tool_use")
                || msg.content.contains("<function_calls>")
                || msg.content.contains("<invoke");

            if !has_tools {
                return false;
            }
        }

        // Timestamp range filter
        if let Some(ref ts) = msg.timestamp {
            if let Some(ref start) = self.start_time {
                if ts < start {
                    return false;
                }
            }

            if let Some(ref end) = self.end_time {
                if ts > end {
                    return false;
                }
            }
        }

        true
    }

    /// Check if any filters are active (non-default)
    pub fn is_active(&self) -> bool {
        !self.show_user
            || !self.show_assistant
            || !self.show_system
            || self.tools_only
            || self.start_time.is_some()
            || self.end_time.is_some()
    }

    /// Reset all filters to default
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Conversation viewer state
pub struct ConversationTab {
    /// Current session ID being viewed
    session_id: Option<String>,

    /// Loaded conversation messages
    messages: Vec<ConversationMessage>,

    /// Vertical scroll offset (message index)
    scroll_offset: usize,

    /// Search query (for future implementation)
    search_query: String,

    /// Error message if load failed
    error: Option<String>,

    /// Loading state
    is_loading: bool,

    /// Syntax highlighting resources
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,

    /// Message filtering
    filter: MessageFilter,

    /// Show filter panel
    show_filter_panel: bool,

    /// Tool call viewers per message (message_index -> viewer)
    tool_viewers: HashMap<usize, ToolCallsViewer>,

    /// Currently viewing tool calls for message index
    viewing_tools: Option<usize>,
}

impl ConversationTab {
    pub fn new() -> Self {
        Self {
            session_id: None,
            messages: Vec::new(),
            scroll_offset: 0,
            search_query: String::new(),
            error: None,
            is_loading: false,
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            filter: MessageFilter::default(),
            show_filter_panel: false,
            tool_viewers: HashMap::new(),
            viewing_tools: None,
        }
    }

    /// Load session content from DataStore (blocking async call)
    pub fn load_session(&mut self, session_id: String, store: &DataStore) {
        self.session_id = Some(session_id.clone());
        self.is_loading = true;
        self.messages.clear();
        self.error = None;
        self.scroll_offset = 0;

        // Use block_in_place to safely block within existing runtime
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(store.load_session_content(&session_id))
        });

        match result {
            Ok(messages) => {
                self.messages = messages;
                self.is_loading = false;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load session: {}", e));
                self.is_loading = false;
                self.messages.clear();
            }
        }
    }

    /// Close conversation view
    pub fn close(&mut self) {
        self.session_id = None;
        self.messages.clear();
        self.scroll_offset = 0;
        self.error = None;
        self.is_loading = false;
        self.tool_viewers.clear();
        self.viewing_tools = None;
    }

    /// Check if a conversation is currently loaded
    pub fn is_open(&self) -> bool {
        self.session_id.is_some()
    }

    /// Scroll down by n messages
    pub fn scroll_down(&mut self, n: usize) {
        if !self.messages.is_empty() {
            self.scroll_offset =
                (self.scroll_offset + n).min(self.messages.len().saturating_sub(1));
        }
    }

    /// Scroll up by n messages
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.scroll_offset = self.messages.len().saturating_sub(1);
        }
    }

    /// Toggle filter panel visibility
    pub fn toggle_filter_panel(&mut self) {
        self.show_filter_panel = !self.show_filter_panel;
    }

    /// Toggle user message filter
    pub fn toggle_user_filter(&mut self) {
        self.filter.show_user = !self.filter.show_user;
        self.scroll_offset = 0; // Reset scroll when filter changes
    }

    /// Toggle assistant message filter
    pub fn toggle_assistant_filter(&mut self) {
        self.filter.show_assistant = !self.filter.show_assistant;
        self.scroll_offset = 0;
    }

    /// Toggle system message filter
    pub fn toggle_system_filter(&mut self) {
        self.filter.show_system = !self.filter.show_system;
        self.scroll_offset = 0;
    }

    /// Toggle tools-only filter
    pub fn toggle_tools_filter(&mut self) {
        self.filter.tools_only = !self.filter.tools_only;
        self.scroll_offset = 0;
    }

    /// Reset all filters
    pub fn reset_filters(&mut self) {
        self.filter.reset();
        self.scroll_offset = 0;
    }

    /// Show tool calls for current message
    pub fn show_tool_calls_for_current(&mut self) {
        // Get filtered messages to find the actual message at scroll_offset
        let filtered: Vec<(usize, &ConversationMessage)> = self
            .messages
            .iter()
            .enumerate()
            .filter(|(_, msg)| self.filter.matches(msg))
            .collect();

        if let Some((original_idx, msg)) = filtered.get(self.scroll_offset) {
            // Check if message has tool calls
            if !msg.tool_calls.is_empty() || !msg.tool_results.is_empty() {
                // Create viewer if not exists
                if !self.tool_viewers.contains_key(original_idx) {
                    let viewer =
                        ToolCallsViewer::new(msg.tool_calls.clone(), msg.tool_results.clone());
                    self.tool_viewers.insert(*original_idx, viewer);
                }

                self.viewing_tools = Some(*original_idx);
            }
        }
    }

    /// Close tool calls viewer
    pub fn close_tool_calls(&mut self) {
        self.viewing_tools = None;
    }

    /// Toggle tool call expansion (when viewing)
    pub fn toggle_tool_call(&mut self) {
        if let Some(msg_idx) = self.viewing_tools {
            if let Some(viewer) = self.tool_viewers.get_mut(&msg_idx) {
                viewer.toggle_selected();
            }
        }
    }

    /// Navigate tool calls
    pub fn tool_call_up(&mut self) {
        if let Some(msg_idx) = self.viewing_tools {
            if let Some(viewer) = self.tool_viewers.get_mut(&msg_idx) {
                viewer.move_up();
            }
        }
    }

    pub fn tool_call_down(&mut self) {
        if let Some(msg_idx) = self.viewing_tools {
            if let Some(viewer) = self.tool_viewers.get_mut(&msg_idx) {
                viewer.move_down();
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode;

        // Tool calls viewer bindings (when viewing tools)
        if self.viewing_tools.is_some() {
            match key {
                KeyCode::Esc => {
                    self.close_tool_calls();
                    return true;
                }
                KeyCode::Enter => {
                    self.toggle_tool_call();
                    return true;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.tool_call_down();
                    return true;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.tool_call_up();
                    return true;
                }
                _ => return false,
            }
        }

        // Filter panel bindings (when panel is open)
        if self.show_filter_panel {
            match key {
                KeyCode::Char('f') | KeyCode::Esc => {
                    self.toggle_filter_panel();
                    return true;
                }
                KeyCode::Char('u') => {
                    self.toggle_user_filter();
                    return true;
                }
                KeyCode::Char('a') => {
                    self.toggle_assistant_filter();
                    return true;
                }
                KeyCode::Char('s') => {
                    self.toggle_system_filter();
                    return true;
                }
                KeyCode::Char('t') => {
                    self.toggle_tools_filter();
                    return true;
                }
                KeyCode::Char('r') => {
                    self.reset_filters();
                    return true;
                }
                _ => return false,
            }
        }

        // Normal navigation bindings
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down(1);
                true
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up(1);
                true
            }
            KeyCode::Char('d') => {
                self.scroll_down(10);
                true
            }
            KeyCode::Char('u') => {
                self.scroll_up(10);
                true
            }
            KeyCode::Char('g') => {
                self.scroll_to_top();
                true
            }
            KeyCode::Char('G') => {
                self.scroll_to_bottom();
                true
            }
            KeyCode::Char('f') => {
                self.toggle_filter_panel();
                true
            }
            KeyCode::Char('t') => {
                // Show tool calls for current message
                self.show_tool_calls_for_current();
                true
            }
            KeyCode::PageDown => {
                self.scroll_down(20);
                true
            }
            KeyCode::PageUp => {
                self.scroll_up(20);
                true
            }
            _ => false,
        }
    }

    /// Render the conversation view
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // If loading, show loading message
        if self.is_loading {
            let loading_text = Paragraph::new("Loading conversation...\n\nPlease wait...")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("💬 Conversation")
                        .style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: true });
            frame.render_widget(loading_text, area);
            return;
        }

        // If error, show error message
        if let Some(ref err) = self.error {
            let error_text = Paragraph::new(err.as_str())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("❌ Error")
                        .style(Style::default().fg(Color::Red)),
                )
                .wrap(Wrap { trim: true });
            frame.render_widget(error_text, area);
            return;
        }

        // If no session loaded, show placeholder
        if self.session_id.is_none() {
            let placeholder = Paragraph::new("No conversation loaded\n\nPress 'c' on a session in the Sessions tab to view its conversation.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("💬 Conversation")
                )
                .style(Style::default().fg(Color::DarkGray))
                .wrap(Wrap { trim: true });
            frame.render_widget(placeholder, area);
            return;
        }

        // Create title with session ID and filter status
        let filtered_count = self.get_filtered_count();
        let filter_indicator = if self.filter.is_active() {
            format!(" [Filtered: {}/{}]", filtered_count, self.messages.len())
        } else {
            format!(" ({} messages)", self.messages.len())
        };

        let title = format!(
            "💬 Conversation: {}{}",
            self.session_id.as_ref().unwrap(),
            filter_indicator
        );

        // Main content area (split if filter panel is shown)
        let (content_area, filter_area) = if self.show_filter_panel {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(40), Constraint::Length(35)])
                .split(area);
            (chunks[0], Some(chunks[1]))
        } else {
            (area, None)
        };

        // Render main content block
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Cyan));

        let inner = block.inner(content_area);
        frame.render_widget(block, content_area);

        // Calculate visible messages based on scroll offset
        let visible_messages = self.get_visible_messages(inner.height as usize);

        // Render messages
        let mut y_offset = 0;
        for (_idx, msg) in visible_messages.iter().enumerate() {
            // Guard FIRST - prevent any operation if out of space
            if y_offset >= inner.height as usize {
                break;
            }

            // Safe subtraction: we know y_offset < inner.height
            let available_height = inner.height as usize - y_offset;
            let msg_height = self
                .calculate_message_height(msg, inner.width as usize)
                .min(available_height);

            let msg_area = Rect {
                x: inner.x,
                y: inner.y + y_offset as u16,
                width: inner.width,
                height: msg_height as u16,
            };

            self.render_message(frame, msg, msg_area);

            // Use saturating_add to prevent overflow
            y_offset = y_offset.saturating_add(msg_height).saturating_add(1);
        }

        // Render scrollbar
        let filtered_count = self.get_filtered_count();
        if filtered_count > visible_messages.len() {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray));

            let mut scrollbar_state =
                ScrollbarState::new(filtered_count).position(self.scroll_offset);

            frame.render_stateful_widget(scrollbar, content_area, &mut scrollbar_state);
        }

        // Render filter panel if visible
        if let Some(filter_area) = filter_area {
            self.render_filter_panel(frame, filter_area);
        }

        // Render tool calls viewer as overlay if viewing
        if let Some(msg_idx) = self.viewing_tools {
            if let Some(viewer) = self.tool_viewers.get(&msg_idx) {
                // Create centered overlay (80% width, 80% height)
                let overlay_width = (area.width as f32 * 0.8) as u16;
                let overlay_height = (area.height as f32 * 0.8) as u16;
                let overlay_x = (area.width - overlay_width) / 2;
                let overlay_y = (area.height - overlay_height) / 2;

                let overlay_area = Rect {
                    x: area.x + overlay_x,
                    y: area.y + overlay_y,
                    width: overlay_width,
                    height: overlay_height,
                };

                // Render backdrop
                let backdrop = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(
                        "🔧 Tool Calls [Esc to close, Enter to expand/collapse, j/k to navigate]",
                    )
                    .style(Style::default().bg(Color::Rgb(15, 15, 25)));

                let inner = backdrop.inner(overlay_area);
                frame.render_widget(backdrop, overlay_area);

                // Render tool calls viewer
                viewer.render(frame, inner);
            }
        }
    }

    /// Render the filter panel
    fn render_filter_panel(&self, frame: &mut Frame, area: Rect) {
        let mut filter_lines = vec![
            Line::from(Span::styled(
                "Filters",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Role Filters:",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(format!(
                "[u] User: {}",
                if self.filter.show_user { "✓" } else { " " }
            )),
            Line::from(format!(
                "[a] Assistant: {}",
                if self.filter.show_assistant {
                    "✓"
                } else {
                    " "
                }
            )),
            Line::from(format!(
                "[s] System: {}",
                if self.filter.show_system { "✓" } else { " " }
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Content Filters:",
                Style::default().fg(Color::Cyan),
            )),
            Line::from(format!(
                "[t] Tools only: {}",
                if self.filter.tools_only { "✓" } else { " " }
            )),
            Line::from(""),
            Line::from(Span::styled("Actions:", Style::default().fg(Color::Cyan))),
            Line::from("[r] Reset filters"),
            Line::from("[f/Esc] Close panel"),
            Line::from(""),
            Line::from(Span::styled(
                if self.filter.is_active() {
                    "⚠ Filters active"
                } else {
                    "No filters"
                },
                Style::default().fg(if self.filter.is_active() {
                    Color::Yellow
                } else {
                    Color::DarkGray
                }),
            )),
        ];

        // Add timestamp filter info if set
        if self.filter.start_time.is_some() || self.filter.end_time.is_some() {
            filter_lines.push(Line::from(""));
            filter_lines.push(Line::from(Span::styled(
                "Time Range:",
                Style::default().fg(Color::Cyan),
            )));
            if let Some(ref start) = self.filter.start_time {
                filter_lines.push(Line::from(format!(
                    "From: {}",
                    start.format("%Y-%m-%d %H:%M")
                )));
            }
            if let Some(ref end) = self.filter.end_time {
                filter_lines.push(Line::from(format!("To: {}", end.format("%Y-%m-%d %H:%M"))));
            }
        }

        let filter_text = Paragraph::new(filter_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🔍 Filters")
                    .style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(filter_text, area);
    }

    /// Get visible messages based on scroll offset, filters, and available height
    fn get_visible_messages(&self, available_height: usize) -> Vec<&ConversationMessage> {
        // Apply filters first
        let filtered_messages: Vec<&ConversationMessage> = self
            .messages
            .iter()
            .filter(|msg| self.filter.matches(msg))
            .collect();

        // Then apply scroll offset
        let start = self
            .scroll_offset
            .min(filtered_messages.len().saturating_sub(1));
        let end = (start + available_height.saturating_sub(2)).min(filtered_messages.len());

        filtered_messages[start..end].to_vec()
    }

    /// Get total count of filtered messages
    fn get_filtered_count(&self) -> usize {
        self.messages
            .iter()
            .filter(|msg| self.filter.matches(msg))
            .count()
    }

    /// Calculate message height based on content
    fn calculate_message_height(&self, msg: &ConversationMessage, width: usize) -> usize {
        // Header takes 1 line
        let header_height = 1;

        // Count content lines with word wrapping
        let content_lines = msg.content.lines().count();
        let wrapped_lines: usize = msg
            .content
            .lines()
            .map(|line| {
                if line.len() > width.saturating_sub(2) {
                    // Account for borders
                    (line.len() / width.saturating_sub(2)) + 1
                } else {
                    1
                }
            })
            .sum();

        // Total: header + content, min 2, max 20
        (header_height + wrapped_lines.max(content_lines)).clamp(2, 20)
    }

    /// Render a single message with syntax highlighting
    fn render_message(&self, frame: &mut Frame, msg: &ConversationMessage, area: Rect) {
        let (role_label, role_color, bg_color) = match msg.role {
            MessageRole::User => ("👤 User", Color::Blue, Color::Rgb(20, 30, 60)),
            MessageRole::Assistant => ("🤖 Assistant", Color::Green, Color::Rgb(20, 50, 30)),
            MessageRole::System => ("⚙️ System", Color::Yellow, Color::Rgb(60, 50, 20)),
        };

        // Build header line with role + timestamp + model
        let mut header_spans = vec![
            Span::styled(
                role_label,
                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ];

        if let Some(ref ts) = msg.timestamp {
            header_spans.push(Span::styled(
                format!("{}", ts.format("%H:%M:%S")),
                Style::default().fg(Color::DarkGray),
            ));
            header_spans.push(Span::raw(" "));
        }

        if let Some(ref model) = msg.model {
            header_spans.push(Span::styled(
                format!("[{}]", model),
                Style::default().fg(Color::Magenta),
            ));
            header_spans.push(Span::raw(" "));
        }

        if let Some(ref tokens) = msg.tokens {
            header_spans.push(Span::styled(
                format!("{}t", tokens.total()),
                Style::default().fg(Color::Cyan),
            ));
        }

        // Split area: header line + content
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(area);

        // Render header
        let header = Paragraph::new(Line::from(header_spans)).style(Style::default().bg(bg_color));
        frame.render_widget(header, layout[0]);

        // Render content with syntax highlighting
        let content_text = self.highlight_content(&msg.content);
        let content = Paragraph::new(content_text)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(bg_color));

        frame.render_widget(content, layout[1]);
    }

    /// Apply syntax highlighting to message content
    fn highlight_content(&self, content: &str) -> Text<'static> {
        // Check if content contains code blocks (```language)
        if content.contains("```") {
            self.highlight_code_blocks(content)
        } else {
            // Plain text
            Text::raw(content.to_string())
        }
    }

    /// Highlight code blocks in markdown-style content
    fn highlight_code_blocks(&self, content: &str) -> Text<'static> {
        let mut lines = Vec::new();
        let mut in_code_block = false;
        let mut current_lang = String::new();
        let mut code_lines = Vec::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block - apply highlighting
                    let highlighted = self.highlight_code(&code_lines.join("\n"), &current_lang);
                    lines.extend(highlighted);
                    code_lines.clear();
                    in_code_block = false;
                } else {
                    // Start of code block
                    current_lang = line[3..].trim().to_string();
                    in_code_block = true;
                }
            } else if in_code_block {
                code_lines.push(line);
            } else {
                // Regular text line
                lines.push(Line::from(line.to_string()));
            }
        }

        // Handle unclosed code block
        if in_code_block && !code_lines.is_empty() {
            let highlighted = self.highlight_code(&code_lines.join("\n"), &current_lang);
            lines.extend(highlighted);
        }

        Text::from(lines)
    }

    /// Highlight code snippet with syntect
    fn highlight_code(&self, code: &str, lang: &str) -> Vec<Line<'static>> {
        // Find syntax by language name
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        // Use base16-ocean.dark theme
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();
        for line_text in LinesWithEndings::from(code) {
            let ranges = highlighter
                .highlight_line(line_text, &self.syntax_set)
                .unwrap_or_default();

            let spans: Vec<Span> = ranges
                .into_iter()
                .map(|(style, text)| {
                    let fg = syntect_to_ratatui_color(style.foreground);
                    Span::styled(text.to_string(), Style::default().fg(fg))
                })
                .collect();

            lines.push(Line::from(spans));
        }

        lines
    }
}

/// Convert syntect color to ratatui color
fn syntect_to_ratatui_color(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

impl Default for ConversationTab {
    fn default() -> Self {
        Self::new()
    }
}
