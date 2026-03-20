use crate::theme::{FocusStyle, Palette};
use ccboard_core::models::config::ColorScheme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
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
    pub fn render(&self, frame: &mut Frame, area: Rect, scheme: ColorScheme) {
        let p = Palette::new(scheme);
        let border_color = if self.focused {
            FocusStyle::focused_border(scheme)
        } else {
            p.border
        };

        let paragraph = Paragraph::new(self.content.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(p.surface))
                    .title(Span::styled(
                        format!(" {} ", self.title),
                        Style::default().fg(p.fg).add_modifier(Modifier::BOLD),
                    )),
            )
            .wrap(Wrap { trim: false })
            .style(Style::default().fg(p.muted));

        frame.render_widget(paragraph, area);
    }
}
