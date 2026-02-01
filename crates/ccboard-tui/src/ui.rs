//! TUI rendering logic

use crate::app::{App, Tab};
use crate::tabs::{
    AgentsTab, ConfigTab, CostsTab, DashboardTab, HistoryTab, HooksTab, SessionsTab,
};
use ccboard_core::DegradedState;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
};

/// Main UI renderer
pub struct Ui {
    dashboard: DashboardTab,
    sessions: SessionsTab,
    config: ConfigTab,
    hooks: HooksTab,
    agents: AgentsTab,
    costs: CostsTab,
    history: HistoryTab,
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Ui {
    pub fn new() -> Self {
        Self {
            dashboard: DashboardTab::new(),
            sessions: SessionsTab::new(),
            config: ConfigTab::new(),
            hooks: HooksTab::new(),
            agents: AgentsTab::new(),
            costs: CostsTab::new(),
            history: HistoryTab::new(),
        }
    }

    /// Initialize tabs that need data scan
    pub fn init(&mut self, claude_home: &std::path::Path, project_path: Option<&std::path::Path>) {
        self.agents.scan_directories(claude_home, project_path);
    }

    /// Handle key input for the active tab
    pub fn handle_tab_key(&mut self, key: crossterm::event::KeyCode, app: &App) {
        match app.active_tab {
            Tab::Dashboard => {
                // Dashboard has no interactive elements yet
            }
            Tab::Sessions => {
                let sessions_by_project = app.store.sessions_by_project();
                self.sessions.handle_key(key, &sessions_by_project);
            }
            Tab::Config => {
                self.config.handle_key(key);
            }
            Tab::Hooks => {
                self.hooks.handle_key(key);
            }
            Tab::Agents => {
                self.agents.handle_key(key);
            }
            Tab::Costs => {
                self.costs.handle_key(key);
            }
            Tab::History => {
                let sessions: Vec<_> = app.store.recent_sessions(10000);
                self.history.handle_key(key, &sessions);
            }
        }
    }

    /// Render the full UI
    pub fn render(&mut self, frame: &mut Frame, app: &App) {
        let size = frame.area();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header + Tab bar
                Constraint::Min(0),    // Content
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render header with tabs
        self.render_header(frame, chunks[0], app.active_tab);

        // Render degraded state warning if needed
        let content_area =
            self.render_degraded_banner(frame, chunks[1], &app.store.degraded_state());

        // Render active tab content
        self.render_tab_content(frame, content_area, app);

        // Render status bar
        self.render_status_bar(frame, chunks[2], app);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect, active: Tab) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split header: logo left, tabs right
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Logo
                Constraint::Min(0),     // Tabs
            ])
            .split(inner);

        // Logo
        let logo = Paragraph::new(Line::from(vec![
            Span::styled("◈ ", Style::default().fg(Color::Cyan)),
            Span::styled("ccboard", Style::default().fg(Color::White).bold()),
        ]));
        frame.render_widget(logo, header_chunks[0]);

        // Tabs
        let titles: Vec<Line> = Tab::all()
            .iter()
            .map(|t| {
                let style = if *t == active {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                Line::from(Span::styled(
                    format!(" {} {} ", t.shortcut(), t.name()),
                    style,
                ))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .select(active.index())
            .divider(Span::styled("│", Style::default().fg(Color::DarkGray)));

        frame.render_widget(tabs, header_chunks[1]);
    }

    fn render_degraded_banner(&self, frame: &mut Frame, area: Rect, state: &DegradedState) -> Rect {
        match state {
            DegradedState::Healthy => area,
            DegradedState::PartialData { reason, .. } => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                let banner = Paragraph::new(Line::from(vec![
                    Span::styled(" ⚠ ", Style::default().fg(Color::Yellow).bold()),
                    Span::styled(reason, Style::default().fg(Color::Yellow)),
                ]))
                .style(Style::default().bg(Color::DarkGray));

                frame.render_widget(banner, chunks[0]);
                chunks[1]
            }
            DegradedState::ReadOnly { reason } => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                let banner = Paragraph::new(Line::from(vec![
                    Span::styled(" ⊘ READ-ONLY ", Style::default().fg(Color::Red).bold()),
                    Span::styled(reason, Style::default().fg(Color::Red)),
                ]))
                .style(Style::default().bg(Color::DarkGray));

                frame.render_widget(banner, chunks[0]);
                chunks[1]
            }
        }
    }

    fn render_tab_content(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        match app.active_tab {
            Tab::Dashboard => {
                let stats = app.store.stats();
                self.dashboard.render(frame, area, stats.as_ref());
            }
            Tab::Sessions => {
                let sessions_by_project = app.store.sessions_by_project();
                self.sessions.render(frame, area, &sessions_by_project);
            }
            Tab::Config => {
                let config = app.store.settings();
                let mcp_config = app.store.mcp_config();
                let rules = app.store.rules();
                self.config
                    .render(frame, area, &config, mcp_config.as_ref(), &rules);
            }
            Tab::Hooks => {
                let config = app.store.settings();
                self.hooks.render(frame, area, &config.merged);
            }
            Tab::Agents => {
                self.agents.render(frame, area);
            }
            Tab::Costs => {
                let stats = app.store.stats();
                self.costs.render(frame, area, stats.as_ref());
            }
            Tab::History => {
                let sessions: Vec<_> = app.store.recent_sessions(10000);
                let stats = app.store.stats();
                self.history.render(frame, area, &sessions, stats.as_ref());
            }
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let session_count = app.store.session_count();

        let status = if let Some(ref msg) = app.status_message {
            Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(Color::Yellow).bold()),
                Span::styled(msg.as_str(), Style::default().fg(Color::Yellow)),
            ])
        } else {
            // Tab-specific hints
            let hint = match app.active_tab {
                Tab::Dashboard => "F5 refresh",
                Tab::Sessions => "←→ nav │ Enter detail │ / search",
                Tab::Config => "←→ columns │ ↑↓ scroll",
                Tab::Hooks => "←→ nav │ ↑↓ select",
                Tab::Agents => "Tab switch │ Enter detail",
                Tab::Costs => "Tab view │ 1-3 switch",
                Tab::History => "Enter detail │ / search │ Tab stats │ c clear",
            };

            Line::from(vec![
                Span::styled(
                    format!(" ● {} sessions ", session_count),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("│", Style::default().fg(Color::DarkGray)),
                Span::styled(" q", Style::default().fg(Color::Cyan).bold()),
                Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
                Span::styled("│", Style::default().fg(Color::DarkGray)),
                Span::styled(format!(" {}", hint), Style::default().fg(Color::DarkGray)),
            ])
        };

        let bar = Paragraph::new(status).style(Style::default().bg(Color::DarkGray));
        frame.render_widget(bar, area);
    }
}
