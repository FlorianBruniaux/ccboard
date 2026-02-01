//! Sessions tab - Project tree + session list + detail view

use ccboard_core::models::SessionMetadata;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use std::collections::HashMap;

/// Sessions tab state
pub struct SessionsTab {
    /// Project tree state (selected project index)
    project_state: ListState,
    /// Session list state (selected session index)
    session_state: ListState,
    /// Current focus: Projects (0) or Sessions (1)
    focus: usize,
    /// Cached project list (sorted)
    projects: Vec<String>,
    /// Search filter (TODO: implement search)
    #[allow(dead_code)]
    search_filter: String,
    /// Show detail popup
    show_detail: bool,
}

impl Default for SessionsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionsTab {
    pub fn new() -> Self {
        let mut project_state = ListState::default();
        project_state.select(Some(0));
        let mut session_state = ListState::default();
        session_state.select(Some(0));

        Self {
            project_state,
            session_state,
            focus: 0,
            projects: Vec::new(),
            search_filter: String::new(),
            show_detail: false,
        }
    }

    /// Handle key input for this tab
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyCode,
        _sessions_by_project: &HashMap<String, Vec<SessionMetadata>>,
    ) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == 0 {
                    self.move_project_selection(-1);
                    // Reset session selection when project changes
                    self.session_state.select(Some(0));
                } else {
                    self.move_session_selection(-1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus == 0 {
                    self.move_project_selection(1);
                    self.session_state.select(Some(0));
                } else {
                    self.move_session_selection(1);
                }
            }
            KeyCode::Enter => {
                if self.focus == 1 {
                    self.show_detail = !self.show_detail;
                }
            }
            KeyCode::Esc => {
                self.show_detail = false;
            }
            KeyCode::Char('/') => {
                // Toggle search mode (simplified)
            }
            _ => {}
        }
    }

    fn move_project_selection(&mut self, delta: i32) {
        if self.projects.is_empty() {
            return;
        }
        let current = self.project_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, self.projects.len() as i32 - 1) as usize;
        self.project_state.select(Some(new_idx));
    }

    fn move_session_selection(&mut self, delta: i32) {
        let current = self.session_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).max(0) as usize;
        self.session_state.select(Some(new_idx));
    }

    /// Render the sessions tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions_by_project: &HashMap<String, Vec<SessionMetadata>>,
    ) {
        // Update project cache
        self.projects = sessions_by_project.keys().cloned().collect();
        self.projects.sort();

        // Main layout: projects tree | session list | detail (if open)
        let constraints = if self.show_detail {
            vec![
                Constraint::Percentage(25),
                Constraint::Percentage(35),
                Constraint::Percentage(40),
            ]
        } else {
            vec![Constraint::Percentage(30), Constraint::Percentage(70)]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        // Render projects tree
        self.render_projects(frame, chunks[0]);

        // Get sessions for selected project
        let selected_project = self
            .project_state
            .selected()
            .and_then(|i| self.projects.get(i))
            .cloned();

        let sessions = selected_project
            .as_ref()
            .and_then(|p| sessions_by_project.get(p))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        // Clamp session selection
        if let Some(sel) = self.session_state.selected() {
            if sel >= sessions.len() && !sessions.is_empty() {
                self.session_state.select(Some(sessions.len() - 1));
            }
        }

        // Render session list
        self.render_sessions(frame, chunks[1], sessions);

        // Render detail popup if open
        if self.show_detail && chunks.len() > 2 {
            let selected_session = self.session_state.selected().and_then(|i| sessions.get(i));
            self.render_detail(frame, chunks[2], selected_session);
        }
    }

    fn render_projects(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == 0;
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" Projects ({}) ", self.projects.len()),
                Style::default().fg(Color::White).bold(),
            ));

        let items: Vec<ListItem> = self
            .projects
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let is_selected = self.project_state.selected() == Some(i);
                let display = Self::format_project_path(path);

                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };

                ListItem::new(Line::from(Span::styled(
                    format!(" {} {}", if is_selected { "▶" } else { " " }, display),
                    style,
                )))
            })
            .collect();

        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, area, &mut self.project_state);
    }

    fn render_sessions(&mut self, frame: &mut Frame, area: Rect, sessions: &[SessionMetadata]) {
        let is_focused = self.focus == 1;
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" Sessions ({}) ", sessions.len()),
                Style::default().fg(Color::White).bold(),
            ));

        if sessions.is_empty() {
            let empty = Paragraph::new("No sessions found")
                .block(block)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = self.session_state.selected() == Some(i);

                // Format timestamp
                let date_str = session
                    .last_timestamp
                    .map(|ts| ts.format("%m/%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                // Format preview
                let preview = session
                    .first_user_message
                    .as_ref()
                    .map(|m| {
                        let truncated: String = m.chars().take(40).collect();
                        if m.len() > 40 {
                            format!("{}...", truncated)
                        } else {
                            truncated
                        }
                    })
                    .unwrap_or_else(|| "No preview".to_string());

                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let tokens_str = Self::format_tokens(session.total_tokens);
                let msgs_str = format!("{}msg", session.message_count);

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", if is_selected { "▶" } else { " " }), style),
                    Span::styled(format!("{} ", date_str), Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{:>6} ", tokens_str),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(
                        format!("{:>5} ", msgs_str),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(preview, style),
                ]))
            })
            .collect();

        let list = List::new(items).block(block);

        frame.render_stateful_widget(list, area, &mut self.session_state);

        // Scrollbar
        if sessions.len() > (area.height as usize - 2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(sessions.len())
                .position(self.session_state.selected().unwrap_or(0));
            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, session: Option<&SessionMetadata>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Session Detail ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(session) = session else {
            let empty =
                Paragraph::new("No session selected").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        let lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&session.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&session.project_path, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Started: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session
                        .first_timestamp
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Ended: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session
                        .last_timestamp
                        .map(|ts| ts.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.duration_display(),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Messages: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.message_count.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_tokens(session.total_tokens),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("File Size: ", Style::default().fg(Color::DarkGray)),
                Span::styled(session.size_display(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Models: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    if session.models_used.is_empty() {
                        "unknown".to_string()
                    } else {
                        session.models_used.join(", ")
                    },
                    Style::default().fg(Color::Magenta),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "First message:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                session
                    .first_user_message
                    .as_deref()
                    .unwrap_or("No preview available"),
                Style::default().fg(Color::White),
            )),
        ];

        let detail = Paragraph::new(lines);
        frame.render_widget(detail, inner);
    }

    fn format_project_path(path: &str) -> String {
        // Shorten path for display
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 3 {
            path.to_string()
        } else {
            format!(".../{}", parts[parts.len() - 2..].join("/"))
        }
    }

    fn format_tokens(tokens: u64) -> String {
        if tokens >= 1_000_000 {
            format!("{:.1}M", tokens as f64 / 1_000_000.0)
        } else if tokens >= 1_000 {
            format!("{:.1}K", tokens as f64 / 1_000.0)
        } else {
            tokens.to_string()
        }
    }
}
