//! History tab - Search and filter prompt history

use crate::components::highlight_matches;
use ccboard_core::models::{SessionMetadata, StatsCache};
use chrono::Local;
use std::sync::Arc;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Sparkline,
    },
};

/// History tab state
pub struct HistoryTab {
    /// Search input
    search_query: String,
    /// Is search focused
    search_focused: bool,
    /// Filtered results state
    results_state: ListState,
    /// Cached filtered sessions (Arc for cheap cloning)
    filtered_sessions: Vec<Arc<SessionMetadata>>,
    /// Show stats panel
    show_stats: bool,
    /// Show detail popup
    show_detail: bool,
    /// Error message to display
    error_message: Option<String>,
    /// Show export dialog
    show_export_dialog: bool,
    /// Export success/error message
    export_message: Option<String>,
}

impl Default for HistoryTab {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryTab {
    pub fn new() -> Self {
        let mut results_state = ListState::default();
        results_state.select(Some(0));

        Self {
            search_query: String::new(),
            search_focused: false,
            results_state,
            filtered_sessions: Vec::new(),
            show_stats: true,
            show_detail: false,
            error_message: None,
            show_export_dialog: false,
            export_message: None,
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode, sessions: &[Arc<SessionMetadata>]) {
        use crossterm::event::KeyCode;

        if self.search_focused {
            match key {
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.update_filter(sessions);
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.update_filter(sessions);
                }
                KeyCode::Enter | KeyCode::Esc => {
                    self.search_focused = false;
                }
                _ => {}
            }
        } else if self.show_export_dialog {
            match key {
                KeyCode::Char('1') => {
                    self.export_csv();
                    self.show_export_dialog = false;
                }
                KeyCode::Char('2') => {
                    self.export_json();
                    self.show_export_dialog = false;
                }
                KeyCode::Esc => {
                    self.show_export_dialog = false;
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Char('x') | KeyCode::Char('X') => {
                    self.show_export_dialog = true;
                }
                KeyCode::Char('/') => {
                    self.search_focused = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    let current = self.results_state.selected().unwrap_or(0);
                    self.results_state.select(Some(current.saturating_sub(1)));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let current = self.results_state.selected().unwrap_or(0);
                    let max = self.filtered_sessions.len().saturating_sub(1);
                    self.results_state.select(Some((current + 1).min(max)));
                }
                KeyCode::Enter => {
                    self.show_detail = !self.show_detail;
                }
                KeyCode::Char('e') => {
                    // Open selected session file in editor
                    if let Some(session) = self.get_selected_session() {
                        if let Err(e) = crate::editor::open_in_editor(&session.file_path) {
                            self.error_message = Some(format!("Failed to open editor: {}", e));
                        }
                    }
                }
                KeyCode::Char('o') => {
                    // Reveal session file in file manager
                    if let Some(session) = self.get_selected_session() {
                        if let Err(e) = crate::editor::reveal_in_file_manager(&session.file_path) {
                            self.error_message =
                                Some(format!("Failed to open file manager: {}", e));
                        }
                    }
                }
                KeyCode::Esc => {
                    if self.error_message.is_some() {
                        self.error_message = None;
                    } else if self.export_message.is_some() {
                        self.export_message = None;
                    } else {
                        self.show_detail = false;
                    }
                }
                KeyCode::Tab => {
                    self.show_stats = !self.show_stats;
                }
                KeyCode::Char('c') => {
                    self.search_query.clear();
                    self.update_filter(sessions);
                }
                KeyCode::PageUp => {
                    // Jump up by 10 items
                    let current = self.results_state.selected().unwrap_or(0);
                    self.results_state.select(Some(current.saturating_sub(10)));
                }
                KeyCode::PageDown => {
                    // Jump down by 10 items
                    let current = self.results_state.selected().unwrap_or(0);
                    let max = self.filtered_sessions.len().saturating_sub(1);
                    self.results_state.select(Some((current + 10).min(max)));
                }
                _ => {}
            }
        }
    }

    fn update_filter(&mut self, sessions: &[Arc<SessionMetadata>]) {
        if self.search_query.is_empty() {
            // Show all sessions sorted by date (Arc clone is cheap)
            self.filtered_sessions = sessions.to_vec();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_sessions = sessions
                .iter()
                .filter(|s| {
                    // Search in project path
                    s.project_path.to_lowercase().contains(&query_lower)
                        // Search in first message preview
                        || s.first_user_message
                            .as_ref()
                            .map(|m| m.to_lowercase().contains(&query_lower))
                            .unwrap_or(false)
                        // Search in models
                        || s.models_used
                            .iter()
                            .any(|m| m.to_lowercase().contains(&query_lower))
                })
                .map(|s| Arc::clone(s))
                .collect();
        }

        // Sort by date (newest first)
        self.filtered_sessions
            .sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

        // Reset selection
        if !self.filtered_sessions.is_empty() {
            self.results_state.select(Some(0));
        }
    }

    fn get_selected_session(&self) -> Option<&Arc<SessionMetadata>> {
        let idx = self.results_state.selected()?;
        self.filtered_sessions.get(idx)
    }

    fn export_csv(&mut self) {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("sessions_export_{}.csv", timestamp);
        let export_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".claude/exports")
            .join(filename);

        match ccboard_core::export_sessions_to_csv(&self.filtered_sessions, &export_path) {
            Ok(_) => {
                self.export_message = Some(format!(
                    "✓ Exported {} sessions to {}",
                    self.filtered_sessions.len(),
                    export_path.display()
                ));
            }
            Err(e) => {
                self.export_message = Some(format!("✗ Export failed: {}", e));
            }
        }
    }

    fn export_json(&mut self) {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("sessions_export_{}.json", timestamp);
        let export_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".claude/exports")
            .join(filename);

        match ccboard_core::export_sessions_to_json(&self.filtered_sessions, &export_path) {
            Ok(_) => {
                self.export_message = Some(format!(
                    "✓ Exported {} sessions to {}",
                    self.filtered_sessions.len(),
                    export_path.display()
                ));
            }
            Err(e) => {
                self.export_message = Some(format!("✗ Export failed: {}", e));
            }
        }
    }

    /// Initialize with session data
    pub fn init(&mut self, sessions: &[Arc<SessionMetadata>]) {
        self.update_filter(sessions);
    }

    /// Render the history tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        sessions: &[Arc<SessionMetadata>],
        stats: Option<&StatsCache>,
    ) {
        // Ensure filtered sessions are initialized
        if self.filtered_sessions.is_empty() && !sessions.is_empty() && self.search_query.is_empty()
        {
            self.update_filter(sessions);
        }

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Content
            ])
            .split(area);

        // Search bar
        self.render_search(frame, chunks[0]);

        // Content layout: results | detail (optional) | stats (optional)
        let content_constraints = match (self.show_detail, self.show_stats) {
            (true, true) => vec![
                Constraint::Percentage(40),
                Constraint::Percentage(35),
                Constraint::Percentage(25),
            ],
            (true, false) => vec![Constraint::Percentage(50), Constraint::Percentage(50)],
            (false, true) => vec![Constraint::Percentage(65), Constraint::Percentage(35)],
            (false, false) => vec![Constraint::Percentage(100)],
        };

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(content_constraints)
            .split(chunks[1]);

        // Results list
        self.render_results(frame, content_chunks[0]);

        // Detail popup if open
        let mut chunk_idx = 1;
        if self.show_detail && content_chunks.len() > 1 {
            let selected_session = self
                .results_state
                .selected()
                .and_then(|i| self.filtered_sessions.get(i));
            self.render_detail(frame, content_chunks[chunk_idx], selected_session);
            chunk_idx += 1;
        }

        // Stats panel
        if self.show_stats && content_chunks.len() > chunk_idx {
            self.render_stats_panel(frame, content_chunks[chunk_idx], stats);
        }

        // Render export dialog if open
        if self.show_export_dialog {
            self.render_export_dialog(frame, area);
        }

        // Render export message if present
        if self.export_message.is_some() {
            self.render_export_message(frame, area);
        }

        // Render error popup if present
        if self.error_message.is_some() {
            self.render_error_popup(frame, area);
        }
    }

    fn render_search(&self, frame: &mut Frame, area: Rect) {
        let border_color = if self.search_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let title_text = if self.search_query.is_empty() {
            " / Search (Press / to focus) ".to_string()
        } else {
            format!(" / Search ({} results) ", self.filtered_sessions.len())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title_text,
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let search_display = if self.search_query.is_empty() {
            if self.search_focused {
                Line::from(Span::styled(
                    "Type to search across all sessions...",
                    Style::default().fg(Color::DarkGray),
                ))
            } else {
                Line::from(Span::styled(
                    "Search all messages across sessions",
                    Style::default().fg(Color::DarkGray),
                ))
            }
        } else {
            Line::from(vec![
                Span::styled(&self.search_query, Style::default().fg(Color::White)),
                if self.search_focused {
                    Span::styled("▌", Style::default().fg(Color::Cyan))
                } else {
                    Span::raw("")
                },
            ])
        };

        let results_info = format!(" {} results", self.filtered_sessions.len());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(results_info.len() as u16 + 2),
            ])
            .split(inner);

        let search_widget = Paragraph::new(search_display);
        frame.render_widget(search_widget, chunks[0]);

        let results_widget = Paragraph::new(Span::styled(
            results_info,
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Right);
        frame.render_widget(results_widget, chunks[1]);
    }

    fn render_results(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " History ",
                Style::default().fg(Color::White).bold(),
            ));

        if self.filtered_sessions.is_empty() {
            let empty_lines = if self.search_query.is_empty() {
                vec![
                    Line::from(""),
                    Line::from(Span::styled("📂 No sessions found", Style::default().fg(Color::Yellow))),
                    Line::from(""),
                    Line::from(Span::styled(
                        "No Claude Code sessions detected across all projects",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "💡 Start a new session:",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "   cd <project-dir> && claude",
                        Style::default().fg(Color::Green),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "📁 Sessions stored in: ~/.claude/projects/",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from(Span::styled("🔍 No matching sessions", Style::default().fg(Color::Yellow))),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("No results for: \"{}\"", self.search_query),
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "💡 Try:",
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        "   • Shorter query (single word)",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "   • Different keywords (project, model, message)",
                        Style::default().fg(Color::DarkGray),
                    )),
                    Line::from(Span::styled(
                        "   • Clear filter (press 'c')",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]
            };
            let empty = Paragraph::new(empty_lines)
                .block(block)
                .alignment(Alignment::Left);
            frame.render_widget(empty, area);
            return;
        }

        // Clamp selection
        if let Some(sel) = self.results_state.selected() {
            if sel >= self.filtered_sessions.len() {
                self.results_state
                    .select(Some(self.filtered_sessions.len() - 1));
            }
        }

        let items: Vec<ListItem> = self
            .filtered_sessions
            .iter()
            .enumerate()
            .map(|(i, session)| {
                let is_selected = self.results_state.selected() == Some(i);

                let date_str = session
                    .last_timestamp
                    .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let preview = session
                    .first_user_message
                    .as_ref()
                    .map(|m| {
                        let truncated: String = m.chars().take(50).collect();
                        if m.len() > 50 {
                            format!("{}...", truncated)
                        } else {
                            truncated
                        }
                    })
                    .unwrap_or_else(|| "No preview".to_string());

                let project_short = Self::shorten_path(&session.project_path);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Build preview line with optional highlighting
                let mut preview_line = vec![Span::styled("    ", Style::default())];
                if !self.search_query.is_empty() {
                    let highlighted = highlight_matches(&preview, &self.search_query);
                    preview_line.extend(highlighted);
                } else {
                    preview_line.push(Span::styled(preview, style));
                }

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            if is_selected { "▶ " } else { "  " },
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(date_str, Style::default().fg(Color::Yellow)),
                        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
                        Span::styled(project_short, Style::default().fg(Color::Green)),
                    ]),
                    Line::from(preview_line),
                    Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(
                            format!(
                                "{}msg • {} • {}",
                                session.message_count,
                                Self::format_tokens(session.total_tokens),
                                session.duration_display()
                            ),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                    Line::from(""), // spacing
                ])
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_stateful_widget(list, area, &mut self.results_state);

        // Scrollbar for long result lists
        let result_count = self.filtered_sessions.len();
        if result_count > (area.height as usize - 2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(result_count)
                .position(self.results_state.selected().unwrap_or(0));
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

    fn render_stats_panel(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Activity Overview ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(5), // Summary stats
                Constraint::Length(6), // Hour heatmap
                Constraint::Min(0),    // Recent activity sparkline
            ])
            .split(inner);

        // Summary stats
        self.render_summary_stats(frame, chunks[0], stats);

        // Hour distribution
        self.render_hour_distribution(frame, chunks[1], stats);

        // Recent activity
        self.render_recent_activity(frame, chunks[2], stats);
    }

    fn render_summary_stats(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let (sessions, messages, tokens) = stats
            .map(|s| (s.total_sessions, s.total_messages, s.total_tokens()))
            .unwrap_or((0, 0, 0));

        let lines = vec![
            Line::from(vec![
                Span::styled("Sessions: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    sessions.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Messages: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_number(messages),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Tokens: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    Self::format_tokens(tokens),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let widget = Paragraph::new(lines);
        frame.render_widget(widget, area);
    }

    fn render_hour_distribution(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let title = Paragraph::new(Span::styled(
            "Activity by Hour:",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(title, area);

        let Some(stats) = stats else {
            return;
        };

        // Build hour histogram (24 hours)
        let max_count = stats
            .hour_counts
            .values()
            .max()
            .copied()
            .unwrap_or(1)
            .max(1);

        let bar_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(2),
        };

        // Simple text-based visualization
        let bars: String = (0..24)
            .map(|h| {
                let count = stats.hour_counts.get(&h.to_string()).copied().unwrap_or(0);
                let ratio = count as f64 / max_count as f64;
                if ratio > 0.75 {
                    '█'
                } else if ratio > 0.5 {
                    '▆'
                } else if ratio > 0.25 {
                    '▄'
                } else if ratio > 0.0 {
                    '▂'
                } else {
                    '▁'
                }
            })
            .collect();

        let widget = Paragraph::new(vec![
            Line::from(Span::styled(bars, Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(
                "0    6    12   18   23",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(widget, bar_area);
    }

    fn render_recent_activity(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let title = Paragraph::new(Span::styled(
            "Recent Activity (7 days):",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(title, area);

        let Some(stats) = stats else {
            return;
        };

        let recent = stats.recent_daily(7);
        let data: Vec<u64> = recent.iter().map(|d| d.message_count).collect();

        if data.is_empty() {
            return;
        }

        let sparkline_area = Rect {
            x: area.x,
            y: area.y + 1,
            width: area.width,
            height: area.height.saturating_sub(2),
        };

        let max_val = data.iter().max().copied().unwrap_or(1).max(1);
        let sparkline = Sparkline::default()
            .data(&data)
            .max(max_val)
            .style(Style::default().fg(Color::Green));

        frame.render_widget(sparkline, sparkline_area);
    }

    fn shorten_path(path: &str) -> String {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() <= 2 {
            path.to_string()
        } else {
            parts[parts.len() - 2..].join("/")
        }
    }

    fn format_tokens(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }

    fn format_number(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{:.2}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, session: Option<&Arc<SessionMetadata>>) {
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

        let mut lines = vec![
            Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&session.id, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("File: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    session.file_path.display().to_string(),
                    Style::default().fg(Color::Yellow),
                ),
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
        ];

        // Add highlighted first message if searching
        let first_msg = session
            .first_user_message
            .as_deref()
            .unwrap_or("No preview available");

        if !self.search_query.is_empty() {
            let highlighted = highlight_matches(first_msg, &self.search_query);
            lines.push(Line::from(highlighted));
        } else {
            lines.push(Line::from(Span::styled(
                first_msg,
                Style::default().fg(Color::White),
            )));
        }

        let detail = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(detail, inner);
    }

    fn render_error_popup(&self, frame: &mut Frame, area: Rect) {
        // Center popup (40% width, 30% height)
        let popup_width = (area.width as f32 * 0.4).max(40.0) as u16;
        let popup_height = (area.height as f32 * 0.3).max(8.0) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " Error ",
                Style::default().fg(Color::Red).bold(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");

        let lines = vec![
            Line::from(Span::styled(error_text, Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }

    fn render_export_dialog(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Clear;

        // Center dialog (40% width, 30% height)
        let dialog_width = (area.width as f32 * 0.4).max(40.0) as u16;
        let dialog_height = 12;
        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect {
            x: area.x + dialog_x,
            y: area.y + dialog_y,
            width: dialog_width,
            height: dialog_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Export Sessions ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let inner = block.inner(dialog_area);

        // Clear background
        frame.render_widget(Clear, dialog_area);
        frame.render_widget(block, dialog_area);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Select export format:",
                Style::default().fg(Color::White).bold(),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  1. ", Style::default().fg(Color::Cyan).bold()),
                Span::styled("CSV format", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(Span::styled(
                "     Date, Time, Project, Messages, Tokens",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  2. ", Style::default().fg(Color::Cyan).bold()),
                Span::styled("JSON format", Style::default().fg(Color::White).bold()),
            ]),
            Line::from(Span::styled(
                "     Full session metadata",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press ESC to cancel",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
        frame.render_widget(paragraph, inner);
    }

    fn render_export_message(&mut self, frame: &mut Frame, area: Rect) {
        let Some(msg) = &self.export_message else {
            return;
        };

        // Show message at bottom of screen for 3 seconds
        let msg_height = 3;
        let msg_area = Rect {
            x: area.x + 2,
            y: area.y + area.height.saturating_sub(msg_height + 1),
            width: area.width.saturating_sub(4),
            height: msg_height,
        };

        let is_success = msg.starts_with('✓');
        let border_color = if is_success { Color::Green } else { Color::Red };
        let text_color = if is_success { Color::Green } else { Color::Red };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(msg_area);
        frame.render_widget(block, msg_area);

        let paragraph = Paragraph::new(msg.as_str())
            .style(Style::default().fg(text_color))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);

        // Auto-clear message after showing
        self.export_message = None;
    }
}
