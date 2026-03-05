//! Search tab — FTS5 full-text search across sessions

use ccboard_core::cache::SearchResult;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
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

    pub fn on_char(&mut self, c: char) {
        if self.input_mode {
            self.query.push(c);
        }
    }

    pub fn on_backspace(&mut self) {
        if self.input_mode {
            self.query.pop();
        }
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

    /// Refresh results from data store
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

/// Render the search tab
pub fn render_search_tab(search_tab: &SearchTab, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(0),    // Results list
        ])
        .split(area);

    // Search input box
    let input_style = if search_tab.input_mode {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let mode_hint = if search_tab.input_mode {
        " [ESC to exit, Enter to search] "
    } else {
        " [i to type] "
    };

    let input = Paragraph::new(search_tab.query.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Search{}", mode_hint))
                .border_style(input_style),
        );
    frame.render_widget(input, chunks[0]);

    // Results list
    let results_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Results ({}) ", search_tab.results.len()));

    if search_tab.results.is_empty() {
        let placeholder = if search_tab.query.len() < 2 {
            "Type at least 2 characters to search..."
        } else {
            "No results found"
        };
        let p = Paragraph::new(placeholder)
            .style(Style::default().fg(Color::DarkGray))
            .block(results_block);
        frame.render_widget(p, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = search_tab
        .results
        .iter()
        .map(|r| {
            let project = r.project.as_deref().unwrap_or("unknown");
            let snippet = r.snippet.as_deref().unwrap_or("");
            let id_len = r.session_id.len().min(8);

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
                ]),
                Line::from(Span::styled(
                    if snippet.is_empty() {
                        r.first_user_message
                            .as_deref()
                            .unwrap_or("(no preview)")
                            .to_string()
                    } else {
                        snippet.to_string()
                    },
                    Style::default().fg(Color::Gray),
                )),
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

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}
