//! Search tab — FTS5 full-text search across sessions

use ccboard_core::cache::SearchResult;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

/// State for the search tab
#[derive(Default)]
pub struct SearchTab {
    /// Current search query input
    pub query: String,
    /// FTS5 search results
    pub results: Vec<SearchResult>,
    /// Selected result index
    pub list_state: ListState,
    /// Whether we're in input mode
    pub input_mode: bool,
}

impl SearchTab {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a character and return true if the caller should refresh results.
    pub fn on_char(&mut self, c: char) -> bool {
        if self.input_mode {
            self.query.push(c);
            return true;
        }
        false
    }

    /// Pop a character and return true if the caller should refresh results.
    pub fn on_backspace(&mut self) -> bool {
        if self.input_mode {
            self.query.pop();
            return true;
        }
        false
    }

    pub fn toggle_input(&mut self) {
        self.input_mode = !self.input_mode;
    }

    pub fn next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.results.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn prev(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) | None => self.results.len() - 1,
            Some(i) => i - 1,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_session_id(&self) -> Option<&str> {
        self.list_state
            .selected()
            .and_then(|i| self.results.get(i))
            .map(|r| r.session_id.as_str())
    }

    /// Currently selected result (for detail pane)
    pub fn selected_result(&self) -> Option<&SearchResult> {
        self.list_state.selected().and_then(|i| self.results.get(i))
    }

    /// Refresh results from data store (min 2 chars to trigger)
    pub fn refresh(&mut self, store: &ccboard_core::DataStore) {
        if self.query.len() >= 2 {
            self.results = store.search_sessions(&self.query, 50);
            if self.results.is_empty() {
                self.list_state.select(None);
            } else if self.list_state.selected().is_none() {
                self.list_state.select(Some(0));
            }
        } else {
            self.results.clear();
            self.list_state.select(None);
        }
    }
}

/// Format an ISO timestamp string to a short display form (date + time).
fn fmt_timestamp(ts: &str) -> String {
    // Input: "2026-03-20T14:30:00Z" or "2026-03-20 14:30:00 UTC" etc.
    // Output: "2026-03-20 14:30"
    let s = ts.replace('T', " ");
    let s = s.trim_end_matches('Z');
    // Take up to "YYYY-MM-DD HH:MM"
    s.get(..16).unwrap_or(ts).to_string()
}

/// Render the search tab
pub fn render_search_tab(search_tab: &SearchTab, frame: &mut Frame, area: Rect) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Results area
        ])
        .split(area);

    // ─── Search input box ─────────────────────────────────────────────────
    let input_style = if search_tab.input_mode {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mode_hint = if search_tab.input_mode {
        " [ESC exit | chars auto-search] "
    } else {
        " [i to type | Enter open conversation] "
    };

    let cursor = if search_tab.input_mode { "▌" } else { "" };
    let input = Paragraph::new(format!("{}{}", search_tab.query, cursor))
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search{}", mode_hint))
                .border_style(input_style),
        );
    frame.render_widget(input, vertical[0]);

    let results_area = vertical[1];

    // ─── Empty state ──────────────────────────────────────────────────────
    if search_tab.results.is_empty() {
        let placeholder = if search_tab.query.len() < 2 {
            "Type at least 2 characters to search across all sessions..."
        } else {
            "No results found"
        };
        let p = Paragraph::new(placeholder)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Results ({}) ", search_tab.results.len())),
            );
        frame.render_widget(p, results_area);
        return;
    }

    // ─── Split results area: left = list, right = detail pane ────────────
    let has_detail = search_tab.selected_result().is_some();
    let horizontal = if has_detail {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(results_area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(results_area)
    };

    // ─── Results list ─────────────────────────────────────────────────────
    let results_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Results ({}) ", search_tab.results.len()));

    let items: Vec<ListItem> = search_tab
        .results
        .iter()
        .map(|r| {
            let project = r.project.as_deref().unwrap_or("unknown");
            let id_len = r.session_id.len().min(8);
            let date = r
                .first_timestamp
                .as_deref()
                .map(fmt_timestamp)
                .unwrap_or_else(|| "—".to_string());

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} ", project),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("[{}]", &r.session_id[..id_len]),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("  {} msgs", r.message_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(format!("{} ", date), Style::default().fg(Color::Yellow)),
                    Span::styled(
                        r.snippet
                            .as_deref()
                            .or(r.first_user_message.as_deref())
                            .unwrap_or("(no preview)")
                            .to_string(),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
            ])
        })
        .collect();

    let mut list_state = search_tab.list_state;
    let list = List::new(items)
        .block(results_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, horizontal[0], &mut list_state);

    // ─── Detail pane ──────────────────────────────────────────────────────
    if has_detail {
        if let Some(r) = search_tab.selected_result() {
            let project = r.project.as_deref().unwrap_or("unknown");
            let date = r
                .first_timestamp
                .as_deref()
                .map(fmt_timestamp)
                .unwrap_or_else(|| "—".to_string());
            let snippet = r
                .snippet
                .as_deref()
                .or(r.first_user_message.as_deref())
                .unwrap_or("(no preview)");

            let detail_lines = vec![
                Line::from(vec![
                    Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        project,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Date:    ", Style::default().fg(Color::DarkGray)),
                    Span::styled(&date, Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::styled("Session: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &r.session_id[..r.session_id.len().min(12)],
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Messages:", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!(" {}", r.message_count),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "Snippet:",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(snippet, Style::default().fg(Color::Gray))),
                Line::from(""),
                Line::from(Span::styled(
                    "Enter → open conversation",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            let detail = Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Detail ")
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .wrap(Wrap { trim: true });

            frame.render_widget(detail, horizontal[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_timestamp_iso8601() {
        assert_eq!(fmt_timestamp("2026-03-20T14:30:00Z"), "2026-03-20 14:30");
    }

    #[test]
    fn test_fmt_timestamp_space_format() {
        assert_eq!(fmt_timestamp("2026-03-20 14:30:00 UTC"), "2026-03-20 14:30");
    }

    #[test]
    fn test_fmt_timestamp_short() {
        // If timestamp is shorter than 16 chars, return as-is
        assert_eq!(fmt_timestamp("2026-03-20"), "2026-03-20");
    }

    #[test]
    fn test_on_char_returns_true_in_input_mode() {
        let mut tab = SearchTab::new();
        tab.input_mode = true;
        assert!(tab.on_char('a'));
        assert_eq!(tab.query, "a");
    }

    #[test]
    fn test_on_char_returns_false_outside_input_mode() {
        let mut tab = SearchTab::new();
        tab.input_mode = false;
        assert!(!tab.on_char('a'));
        assert_eq!(tab.query, "");
    }

    #[test]
    fn test_on_backspace_returns_true_in_input_mode() {
        let mut tab = SearchTab::new();
        tab.input_mode = true;
        tab.query = "hello".to_string();
        assert!(tab.on_backspace());
        assert_eq!(tab.query, "hell");
    }

    #[test]
    fn test_on_backspace_returns_false_outside_input_mode() {
        let mut tab = SearchTab::new();
        tab.input_mode = false;
        tab.query = "hello".to_string();
        assert!(!tab.on_backspace());
        assert_eq!(tab.query, "hello"); // unchanged
    }

    #[test]
    fn test_selected_result_none_when_empty() {
        let tab = SearchTab::new();
        assert!(tab.selected_result().is_none());
    }
}
