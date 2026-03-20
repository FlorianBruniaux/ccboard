use crate::theme::{FocusStyle, Palette};
use ccboard_core::models::config::ColorScheme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{
        Block, BorderType, Borders, List, ListItem, ListState, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

/// Reusable list pane component for displaying selectable items
pub struct ListPane<'a> {
    /// Title of the pane
    pub title: String,
    /// Items to display
    pub items: Vec<String>,
    /// List state (selection + scroll)
    pub state: &'a mut ListState,
    /// Whether this pane is focused
    pub focused: bool,
    /// Scrollbar state
    pub scrollbar_state: ScrollbarState,
}

impl<'a> ListPane<'a> {
    /// Create a new list pane
    pub fn new(
        title: impl Into<String>,
        items: Vec<String>,
        state: &'a mut ListState,
        focused: bool,
    ) -> Self {
        let item_count = items.len();
        Self {
            title: title.into(),
            items,
            state,
            focused,
            scrollbar_state: ScrollbarState::new(item_count),
        }
    }

    /// Render the list pane
    pub fn render(&mut self, frame: &mut Frame, area: Rect, scheme: ColorScheme) {
        let p = Palette::new(scheme);
        let border_color = if self.focused {
            FocusStyle::focused_border(scheme)
        } else {
            p.border
        };

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| ListItem::new(item.as_str()))
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color))
                    .style(Style::default().bg(p.surface))
                    .title(Span::styled(
                        format!(" {} ({}) ", self.title, self.items.len()),
                        Style::default().fg(p.fg).add_modifier(Modifier::BOLD),
                    )),
            )
            .highlight_style(
                Style::default()
                    .bg(FocusStyle::focused_bg(scheme))
                    .fg(p.focus)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, self.state);

        // Render scrollbar if items overflow
        if self.items.len() > (area.height as usize).saturating_sub(2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            let scroll_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };

            let selected = self.state.selected().unwrap_or(0);
            self.scrollbar_state = self.scrollbar_state.position(selected);

            frame.render_stateful_widget(scrollbar, scroll_area, &mut self.scrollbar_state);
        }
    }
}
