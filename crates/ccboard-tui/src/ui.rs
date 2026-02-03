//! TUI rendering logic

use crate::app::{App, Tab};
use crate::components::{Breadcrumb, Breadcrumbs};
use crate::tabs::{
    AgentsTab, ConfigTab, CostsTab, DashboardTab, HistoryTab, HooksTab, McpTab, SessionsTab,
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
    mcp: McpTab,
    breadcrumbs: Breadcrumbs,
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
            mcp: McpTab::new(),
            breadcrumbs: Breadcrumbs::new(),
        }
    }

    /// Initialize tabs that need data scan
    pub fn init(
        &mut self,
        claude_home: &std::path::Path,
        project_path: Option<&std::path::Path>,
        invocation_stats: &ccboard_core::models::InvocationStats,
    ) {
        self.agents.scan_directories(claude_home, project_path);
        self.agents.update_invocation_counts(invocation_stats);
        self.config.init(claude_home, project_path);
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
                let hooks_map = app
                    .store
                    .settings()
                    .merged
                    .hooks
                    .clone()
                    .unwrap_or_default();
                self.hooks.handle_key(key, &hooks_map);
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
            Tab::Mcp => {
                let mcp_config = app.store.mcp_config();
                self.mcp.handle_key(key, mcp_config.as_ref());
            }
        }
    }

    /// Render the full UI
    pub fn render(&mut self, frame: &mut Frame, app: &mut App) {
        let size = frame.area();

        // If loading, show loading screen
        if app.is_loading {
            self.render_loading_screen(frame, size, app);
            return;
        }

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header + Tab bar + Breadcrumbs
                Constraint::Min(0),    // Content
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render header with tabs and breadcrumbs
        self.render_header(frame, chunks[0], app.active_tab, app);

        // Render degraded state warning if needed
        let content_area =
            self.render_degraded_banner(frame, chunks[1], &app.store.degraded_state());

        // Render active tab content
        self.render_tab_content(frame, content_area, app);

        // Render status bar
        self.render_status_bar(frame, chunks[2], app);

        // Render command palette (overlay on top of everything)
        app.command_palette.render(frame, size);
    }

    fn render_loading_screen(&mut self, frame: &mut Frame, area: Rect, app: &mut App) {
        // Tick spinner animation
        app.spinner.tick();

        // Center the loading message
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(7),
                Constraint::Percentage(40),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(vertical[1]);

        let loading_area = horizontal[1];

        // Create loading box
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " ccboard ",
                Style::default().fg(Color::Cyan).bold(),
            ));

        let inner = block.inner(loading_area);
        frame.render_widget(block, loading_area);

        // Split inner area for spinner and message
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        // Render spinner with message
        let message = app
            .loading_message
            .as_deref()
            .unwrap_or("Loading...");

        let spinner_line = Line::from(vec![
            Span::raw("  "),
            app.spinner.render(),
            Span::raw("  "),
            Span::styled(message, Style::default().fg(Color::White)),
        ]);

        let spinner_widget = Paragraph::new(spinner_line);
        frame.render_widget(spinner_widget, inner_chunks[2]);

        // Render hint
        let hint = Paragraph::new(Line::from(vec![Span::styled(
            "Press 'q' to quit",
            Style::default().fg(Color::DarkGray),
        )]))
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(hint, inner_chunks[4]);
    }

    fn render_header(&mut self, frame: &mut Frame, area: Rect, active: Tab, app: &App) {
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split header vertically: tab bar + breadcrumbs
        let header_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tab bar
                Constraint::Length(1), // Breadcrumbs
            ])
            .split(inner);

        // Tab bar: split horizontally (logo left, tabs right)
        let tab_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Logo
                Constraint::Min(0),     // Tabs
            ])
            .split(header_rows[0]);

        // Logo
        let logo = Paragraph::new(Line::from(vec![
            Span::styled("◈ ", Style::default().fg(Color::Cyan)),
            Span::styled("ccboard", Style::default().fg(Color::White).bold()),
        ]));
        frame.render_widget(logo, tab_bar_chunks[0]);

        // Tabs with icons
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
                    format!(" {} {} {} ", t.icon(), t.shortcut(), t.name()),
                    style,
                ))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .select(active.index())
            .divider(Span::styled("│", Style::default().fg(Color::DarkGray)));

        frame.render_widget(tabs, tab_bar_chunks[1]);

        // Breadcrumbs (second row)
        let breadcrumbs_path = self.get_breadcrumbs_for_tab(active, app);
        self.breadcrumbs.set_path(breadcrumbs_path);
        self.breadcrumbs.render(frame, header_rows[1]);
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
                let mcp_config = app.store.mcp_config();
                self.dashboard.render(
                    frame,
                    area,
                    stats.as_ref(),
                    mcp_config.as_ref(),
                    Some(&app.store),
                );
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
                let billing_blocks = app.store.billing_blocks();
                self.costs
                    .render(frame, area, stats.as_ref(), Some(&billing_blocks));
            }
            Tab::History => {
                let sessions: Vec<_> = app.store.recent_sessions(10000);
                let stats = app.store.stats();
                self.history.render(frame, area, &sessions, stats.as_ref());
            }
            Tab::Mcp => {
                let mcp_config = app.store.mcp_config();
                self.mcp.render(frame, area, mcp_config.as_ref());
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
                Tab::Config => "←→ columns │ ↑↓ scroll │ e edit │ o reveal",
                Tab::Hooks => "←→ nav │ ↑↓ select",
                Tab::Agents => "Tab switch │ Enter detail",
                Tab::Costs => "Tab/←→/h/l switch views",
                Tab::History => "Enter detail │ / search │ Tab stats │ c clear",
                Tab::Mcp => "←→ focus │ ↑↓ select │ e edit │ o reveal │ r refresh",
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

    /// Get breadcrumbs path for the active tab
    fn get_breadcrumbs_for_tab(&self, tab: Tab, _app: &App) -> Vec<Breadcrumb> {
        let mut path = vec![Breadcrumb::new("Dashboard").with_level(0)];

        match tab {
            Tab::Dashboard => {
                // Just "Dashboard"
            }
            Tab::Sessions => {
                path.push(Breadcrumb::new("Sessions").with_level(1));
            }
            Tab::Config => {
                path.push(Breadcrumb::new("Config").with_level(1));
            }
            Tab::Hooks => {
                path.push(Breadcrumb::new("Hooks").with_level(1));
            }
            Tab::Agents => {
                path.push(Breadcrumb::new("Agents").with_level(1));
            }
            Tab::Costs => {
                path.push(Breadcrumb::new("Costs").with_level(1));
            }
            Tab::History => {
                path.push(Breadcrumb::new("History").with_level(1));
            }
            Tab::Mcp => {
                path.push(Breadcrumb::new("MCP").with_level(1));
            }
        }

        path
    }
}
