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
            (self.query.as_str(), Style::default().fg(Color::White))
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
                Span::styled(
                    "_",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::SLOW_BLINK),
                )
            } else {
                Span::raw("")
            },
        ]);

        let paragraph = Paragraph::new(search_line).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
                .title(Span::styled(
                    " Search ",
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )),
        );

        frame.render_widget(paragraph, area);
    }
}

/// Highlight search matches in text with yellow background
///
/// Takes a text string and a search query, returns a vector of Spans
/// with matches highlighted in yellow.
///
/// # Arguments
/// * `text` - The text to search in
/// * `query` - The search query (case-insensitive)
///
/// # Returns
/// Vector of owned Spans with highlighted matches
pub fn highlight_matches(text: &str, query: &str) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::raw(text.to_string())];
    }

    let query_lower = query.to_lowercase();
    let text_lower = text.to_lowercase();

    let mut spans = Vec::new();
    let mut last_end = 0;

    // Find all matches (case-insensitive)
    for (idx, _) in text_lower.match_indices(&query_lower) {
        // Add text before match
        if idx > last_end {
            spans.push(Span::raw(text[last_end..idx].to_string()));
        }

        // Add highlighted match
        let match_end = idx + query.len();
        spans.push(Span::styled(
            text[idx..match_end].to_string(),
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));

        last_end = match_end;
    }

    // Add remaining text after last match
    if last_end < text.len() {
        spans.push(Span::raw(text[last_end..].to_string()));
    }

    // If no matches found, return original text
    if spans.is_empty() {
        vec![Span::raw(text.to_string())]
    } else {
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_empty_query() {
        let spans = highlight_matches("hello world", "");
        assert_eq!(spans.len(), 1);
    }

    #[test]
    fn test_highlight_single_match() {
        let spans = highlight_matches("hello world", "world");
        assert_eq!(spans.len(), 2); // "hello " + highlighted "world"
    }

    #[test]
    fn test_highlight_multiple_matches() {
        let spans = highlight_matches("test test test", "test");
        assert_eq!(spans.len(), 5); // test + " " + test + " " + test
    }

    #[test]
    fn test_highlight_case_insensitive() {
        let spans = highlight_matches("Hello World", "WORLD");
        assert_eq!(spans.len(), 2);
    }

    #[test]
    fn test_highlight_no_match() {
        let spans = highlight_matches("hello world", "xyz");
        assert_eq!(spans.len(), 1);
    }
}
