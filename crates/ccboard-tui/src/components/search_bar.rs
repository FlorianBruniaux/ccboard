use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Reusable search bar component for filtering lists
pub struct SearchBar {
    /// Current search query
    pub query: String,
    /// Whether the search bar is active (focused)
    pub active: bool,
    /// Placeholder text when empty
    pub placeholder: String,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchBar {
    /// Create a new search bar
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
            placeholder: "Type to search...".to_string(),
        }
    }

    /// Create a new search bar with custom placeholder
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the query
    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    /// Clear the query
    pub fn clear(&mut self) {
        self.query.clear();
    }

    /// Check if query is empty
    pub fn is_empty(&self) -> bool {
        self.query.is_empty()
    }

    /// Render the search bar
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let (text, style) = if self.query.is_empty() {
            (
                self.placeholder.as_str(),
                Style::default().fg(Color::DarkGray),
            )
        } else {
            (
                self.query.as_str(),
                Style::default().fg(Color::White),
            )
        };

        let border_color = if self.active {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let search_line = Line::from(vec![
            Span::styled("🔍 ", Style::default().fg(Color::Cyan)),
            Span::styled(text, style),
            if self.active {
                Span::styled("_", Style::default().fg(Color::Cyan).add_modifier(Modifier::SLOW_BLINK))
            } else {
                Span::raw("")
            },
        ]);

        let paragraph = Paragraph::new(search_line)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        " Search ",
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    )),
            );

        frame.render_widget(paragraph, area);
    }
}
