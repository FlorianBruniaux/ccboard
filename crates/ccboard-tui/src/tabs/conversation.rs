//! Conversation viewer - Displays full session content with syntax highlighting

use ccboard_core::models::{ConversationMessage, MessageRole};
use ccboard_core::DataStore;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};
use std::sync::Arc;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

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
        }
    }

    /// Load session content from DataStore (blocking async call)
    pub fn load_session(&mut self, session_id: String, store: &DataStore) {
        self.session_id = Some(session_id.clone());
        self.is_loading = true;
        self.messages.clear();
        self.error = None;
        self.scroll_offset = 0;

        // Block on async operation using tokio runtime
        let runtime = tokio::runtime::Handle::current();
        match runtime.block_on(store.load_session_content(&session_id)) {
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

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) -> bool {
        use crossterm::event::KeyCode;

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

        // Create title with session ID
        let title = format!(
            "💬 Conversation: {} ({} messages)",
            self.session_id.as_ref().unwrap(),
            self.messages.len()
        );

        // Main content area
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate visible messages based on scroll offset
        let visible_messages = self.get_visible_messages(inner.height as usize);

        // Render messages
        let mut y_offset = 0;
        for (_idx, msg) in visible_messages.iter().enumerate() {
            let msg_area = Rect {
                x: inner.x,
                y: inner.y + y_offset as u16,
                width: inner.width,
                height: (inner.height as usize - y_offset).min(10) as u16, // Max 10 lines per message preview
            };

            if y_offset >= inner.height as usize {
                break;
            }

            self.render_message(frame, msg, msg_area);
            y_offset += msg_area.height as usize + 1; // +1 for spacing
        }

        // Render scrollbar
        if self.messages.len() > visible_messages.len() {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray));

            let mut scrollbar_state =
                ScrollbarState::new(self.messages.len()).position(self.scroll_offset);

            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    /// Get visible messages based on scroll offset and available height
    fn get_visible_messages(&self, available_height: usize) -> Vec<&ConversationMessage> {
        let start = self.scroll_offset;
        let end = (start + available_height.saturating_sub(2)).min(self.messages.len());
        self.messages[start..end].iter().collect()
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
