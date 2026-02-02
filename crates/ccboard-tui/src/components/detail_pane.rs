use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Reusable detail pane component for displaying content
pub struct DetailPane {
    /// Title of the pane
    pub title: String,
    /// Content to display
    pub content: String,
    /// Whether this pane is focused
    pub focused: bool,
}

impl DetailPane {
    /// Create a new detail pane
    pub fn new(title: impl Into<String>, content: impl Into<String>, focused: bool) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            focused,
        }
    }

    /// Render the detail pane
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let paragraph = Paragraph::new(self.content.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(Span::styled(
                        format!(" {} ", self.title),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    )),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(paragraph, area);
    }
}
