//! TUI rendering logic

use crate::app::{App, Tab};
use crate::components::{Breadcrumb, Breadcrumbs};
use crate::tabs::{
    AgentsTab, AnalyticsTab, ConfigTab, CostsTab, DashboardTab, HistoryTab, HooksTab, McpTab,
    PluginsTab, SessionsTab,
};
use ccboard_core::DegradedState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
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
    analytics: AnalyticsTab,
    plugins: PluginsTab,
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
            analytics: AnalyticsTab::new(),
            plugins: PluginsTab::new(),
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
    pub fn handle_tab_key(&mut self, key: crossterm::event::KeyCode, app: &mut App) {
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
                self.history
                    .handle_key(key, &sessions, &mut app.search_history);
            }
            Tab::Mcp => {
                let mcp_config = app.store.mcp_config();
                self.mcp.handle_key(key, mcp_config.as_ref());
            }
            Tab::Analytics => {
                use ccboard_core::analytics::Period;
                use crossterm::event::KeyCode;
                match key {
                    KeyCode::F(1) => {
                        self.analytics.set_period(Period::last_7d());
                        let store = app.store.clone();
                        tokio::spawn(async move {
                            store.compute_analytics(Period::last_7d()).await;
                        });
                    }
                    KeyCode::F(2) => {
                        self.analytics.set_period(Period::last_30d());
                        let store = app.store.clone();
                        tokio::spawn(async move {
                            store.compute_analytics(Period::last_30d()).await;
                        });
                    }
                    KeyCode::F(3) => {
                        self.analytics.set_period(Period::last_90d());
                        let store = app.store.clone();
                        tokio::spawn(async move {
                            store.compute_analytics(Period::last_90d()).await;
                        });
                    }
                    KeyCode::F(4) => {
                        self.analytics.set_period(Period::available());
                        let store = app.store.clone();
                        tokio::spawn(async move {
                            store.compute_analytics(Period::available()).await;
                        });
                    }
                    KeyCode::Right | KeyCode::Char('l') => self.analytics.next_view(),
                    KeyCode::Left | KeyCode::Char('h') => self.analytics.prev_view(),
                    KeyCode::Char('j') | KeyCode::Down => {
                        let max_items =
                            app.store.analytics().map(|a| a.insights.len()).unwrap_or(0);
                        self.analytics.scroll_down(max_items);
                    }
                    KeyCode::Char('k') | KeyCode::Up => self.analytics.scroll_up(),
                    KeyCode::Char('s') => {
                        // Cycle sort column for project leaderboard
                        self.analytics.cycle_sort_column();
                    }
                    KeyCode::Char('o') => {
                        // Toggle sort order (ascending/descending) for project leaderboard
                        self.analytics.toggle_sort_order();
                    }
                    KeyCode::Char('r') => {
                        // Recompute analytics with current period (async operation)
                        let store = app.store.clone();
                        let period = self.analytics.period();
                        tokio::spawn(async move {
                            store.compute_analytics(period).await;
                        });
                    }
                    _ => {}
                }
            }
            Tab::Plugins => {
                self.plugins.handle_key(key, &app.store);
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
                Constraint::Length(5), // Header + Tab bar + Breadcrumbs (2 for tabs + 2 for breadcrumbs + 1 for borders)
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

        // Render help modal (overlay on top of command palette)
        app.help_modal
            .render(frame, size, app.active_tab, &app.keybindings);

        // Render toast notifications (overlay on top)
        app.toast_manager.render(frame, size);

        // Render confirmation dialog (overlay on top of toasts)
        app.confirm_dialog.render(frame, size);
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
        let message = app.loading_message.as_deref().unwrap_or("Loading...");

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

        // Split header vertically: tab bar + separator + breadcrumbs
        let header_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Tab bar with padding
                Constraint::Length(2), // Breadcrumbs (with top border = 1 line + content = 1 line)
            ])
            .split(inner);

        // Add vertical padding to tab bar area (empty line at bottom)
        let tab_bar_padded = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Tabs
                Constraint::Length(1), // Bottom padding
            ])
            .split(header_rows[0]);

        // Tab bar: split horizontally (logo left, tabs right)
        let tab_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // Logo
                Constraint::Min(0),     // Tabs
            ])
            .split(tab_bar_padded[0]);

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

        // Breadcrumbs (second row with top border)
        let breadcrumb_block = Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray));

        let breadcrumb_inner = breadcrumb_block.inner(header_rows[1]);
        frame.render_widget(breadcrumb_block, header_rows[1]);

        let breadcrumbs_path = self.get_breadcrumbs_for_tab(active, app);
        self.breadcrumbs.set_path(breadcrumbs_path);
        self.breadcrumbs.render(frame, breadcrumb_inner);
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
        let scheme = app.color_scheme;
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
                    scheme,
                );
            }
            Tab::Sessions => {
                let sessions_by_project = app.store.sessions_by_project();
                let live_sessions = app.live_sessions(); // Use cached live sessions
                                                         // Count total sessions for refresh tracking
                let session_count: usize = sessions_by_project.values().map(|v| v.len()).sum();
                self.sessions.mark_refreshed(session_count);
                self.sessions
                    .render(frame, area, &sessions_by_project, live_sessions, scheme);
            }
            Tab::Config => {
                let config = app.store.settings();
                let mcp_config = app.store.mcp_config();
                let rules = app.store.rules();
                self.config
                    .render(frame, area, &config, mcp_config.as_ref(), &rules, scheme);
            }
            Tab::Hooks => {
                let config = app.store.settings();
                self.hooks.render(frame, area, &config.merged, scheme);
            }
            Tab::Agents => {
                self.agents.render(frame, area, scheme);
            }
            Tab::Costs => {
                let stats = app.store.stats();
                let billing_blocks = app.store.billing_blocks();
                self.costs.render(
                    frame,
                    area,
                    stats.as_ref(),
                    Some(&billing_blocks),
                    scheme,
                    Some(&app.store),
                );
            }
            Tab::History => {
                let sessions: Vec<_> = app.store.recent_sessions(10000);
                let stats = app.store.stats();
                self.history
                    .render(frame, area, &sessions, stats.as_ref(), scheme);
            }
            Tab::Mcp => {
                let mcp_config = app.store.mcp_config();
                self.mcp.render(frame, area, mcp_config.as_ref(), scheme);
            }
            Tab::Analytics => {
                use tracing::debug;
                let analytics = app.store.analytics();
                debug!(
                    has_analytics = analytics.is_some(),
                    "ui.rs: Rendering Analytics tab"
                );
                self.analytics
                    .render(frame, area, analytics.as_ref(), Some(&app.store), scheme);
            }
            Tab::Plugins => {
                self.plugins.render(frame, area, &app.store);
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
                Tab::Sessions => {
                    "←→ nav │ / search │ d date filter │ r resume │ gg/G/Home/End jump"
                }
                Tab::Config => "←→ columns │ ↑↓ scroll │ e edit │ o reveal",
                Tab::Hooks => "←→ nav │ ↑↓ select │ t test │ e edit │ o reveal",
                Tab::Agents => "Tab switch │ Enter detail",
                Tab::Costs => "Tab/←→/h/l switch views",
                Tab::History => "/ search │ gg/G/Home/End jump │ c clear │ x export",
                Tab::Mcp => "←→ focus │ ↑↓ select │ e edit │ o reveal │ r refresh",
                Tab::Analytics => {
                    "F1-F4 period │ ←→/h/l switch views │ j/k scroll │ s sort │ o order │ r refresh"
                }
                Tab::Plugins => "Tab cycle columns │ j/k navigate │ s sort │ r refresh",
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
                path.push(Breadcrumb::new("Capabilities").with_level(1));
                // Add sub-tab to breadcrumbs
                let sub_tab_label = self.agents.current_sub_tab_label();
                path.push(Breadcrumb::new(sub_tab_label).with_level(2));
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
            Tab::Analytics => {
                path.push(Breadcrumb::new("Analytics").with_level(1));
            }
            Tab::Plugins => {
                path.push(Breadcrumb::new("Plugins").with_level(1));
            }
        }

        path
    }
}
