//! TUI rendering logic

use crate::app::{App, Tab};
// Breadcrumbs removed — navigation is now shown in the header tab bar
use crate::tabs::render_search_tab;
use crate::tabs::{
    ActivityTab, AgentsTab, AnalyticsTab, ConfigTab, ConversationTab, CostsTab, DashboardTab,
    HistoryTab, HooksTab, McpTab, PluginsTab, SessionsTab,
};
use crate::theme::Palette;
use ccboard_core::DegradedState;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
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
    activity: ActivityTab,
    conversation: ConversationTab,
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
            activity: ActivityTab::new(),
            conversation: ConversationTab::new(),
        }
    }

    /// Check if conversation viewer is currently open
    pub fn is_conversation_open(&self) -> bool {
        self.conversation.is_open()
    }

    /// Check if replay viewer is currently open (in Sessions tab)
    pub fn is_replay_open(&self) -> bool {
        self.sessions.is_replay_open()
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
        use crossterm::event::KeyCode;

        // If conversation is open, handle keys there first
        if self.conversation.is_open() {
            match key {
                KeyCode::Esc => {
                    self.conversation.close();
                }
                _ => {
                    self.conversation.handle_key(key);
                }
            }
            return;
        }

        match app.active_tab {
            Tab::Dashboard => {
                // Dashboard has no interactive elements yet
            }
            Tab::Sessions => {
                let sessions_by_project = app.store.sessions_by_project();

                // Check if 'c' key pressed to open conversation
                if let KeyCode::Char('c') = key {
                    if let Some(session_id) =
                        self.sessions.selected_session_id(&sessions_by_project)
                    {
                        self.conversation.load_session(session_id, &app.store);
                        return;
                    }
                }

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
            Tab::Activity => {
                let sessions = app.store.recent_sessions(50);
                self.activity.handle_key(key, &sessions, &app.store);
            }
            Tab::Search => {
                use crossterm::event::KeyCode;

                if app.search_tab.input_mode {
                    match key {
                        KeyCode::Esc => {
                            app.search_tab.toggle_input();
                        }
                        KeyCode::Enter => {
                            // Open selected result's conversation
                            if let Some(session_id) = app.search_tab.selected_session_id() {
                                self.conversation
                                    .load_session(session_id.to_string(), &app.store);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.search_tab.on_backspace() {
                                app.search_tab.refresh(app.store.as_ref());
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.search_tab.on_char(c) {
                                app.search_tab.refresh(app.store.as_ref());
                            }
                        }
                        // Allow navigating results while in input mode
                        KeyCode::Down => app.search_tab.next(),
                        KeyCode::Up => app.search_tab.prev(),
                        _ => {}
                    }
                } else {
                    match key {
                        KeyCode::Char('i') => {
                            app.search_tab.toggle_input();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.search_tab.next();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.search_tab.prev();
                        }
                        KeyCode::Enter => {
                            if let Some(session_id) = app.search_tab.selected_session_id() {
                                self.conversation
                                    .load_session(session_id.to_string(), &app.store);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Render the full UI
    pub fn render(&mut self, frame: &mut Frame, app: &mut App) {
        let size = frame.area();

        // Update notification timeouts every frame (even if tab not visible)
        self.sessions.update_notification_timeout();

        // If loading, show loading screen
        if app.is_loading {
            self.render_loading_screen(frame, size, app);
            return;
        }

        // Fill entire frame with theme background (critical for light mode)
        let p = Palette::new(app.color_scheme);
        frame.render_widget(Clear, size);
        frame.render_widget(
            Block::default().style(Style::default().bg(p.bg).fg(p.fg)),
            size,
        );

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header: logo + tabs on one line + separator
                Constraint::Min(0),    // Content
                Constraint::Length(1), // Status bar
            ])
            .split(size);

        // Render header with tabs and breadcrumbs
        self.render_header(frame, chunks[0], app.active_tab, app);

        // Render degraded state warning if needed
        let content_area = self.render_degraded_banner(
            frame,
            chunks[1],
            &app.store.degraded_state(),
            app.color_scheme,
        );

        // Render active tab content
        self.render_tab_content(frame, content_area, app);

        // Render status bar
        self.render_status_bar(frame, chunks[2], app);

        // Render command palette (overlay on top of everything)
        app.command_palette.render(frame, size, app.color_scheme);

        // Render help modal (overlay on top of command palette)
        app.help_modal.render(
            frame,
            size,
            app.active_tab,
            &app.keybindings,
            app.color_scheme,
        );

        // Render toast notifications (overlay on top)
        app.toast_manager.render(frame, size, app.color_scheme);

        // Render confirmation dialog (overlay on top of toasts)
        app.confirm_dialog.render(frame, size, app.color_scheme);
    }

    fn render_loading_screen(&mut self, frame: &mut Frame, area: Rect, app: &mut App) {
        let p = Palette::new(app.color_scheme);
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
            .border_style(Style::default().fg(p.focus))
            .style(Style::default().bg(p.bg))
            .title(Span::styled(
                " ccboard ",
                Style::default().fg(p.focus).bold(),
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
            Span::styled(message, Style::default().fg(p.fg)),
        ]);

        let spinner_widget = Paragraph::new(spinner_line);
        frame.render_widget(spinner_widget, inner_chunks[2]);

        // Render hint
        let hint = Paragraph::new(Line::from(vec![Span::styled(
            "Press 'q' to quit",
            Style::default().fg(p.muted),
        )]))
        .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(hint, inner_chunks[4]);
    }

    fn render_header(&mut self, frame: &mut Frame, area: Rect, active: Tab, app: &App) {
        let p = Palette::new(app.color_scheme);

        // Separator border at bottom only
        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(p.border));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Horizontal layout: logo | tabs | hints
        let tab_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(12), // "◈ ccboard"
                Constraint::Min(0),     // Tab bar
                Constraint::Length(20), // "[?] help  [q] quit"
            ])
            .split(inner);

        // Logo
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("◈ ", Style::default().fg(p.focus)),
                Span::styled("ccboard", Style::default().fg(p.fg).bold()),
            ])),
            tab_bar_chunks[0],
        );

        // Tabs — all show their full name, active gets highlight background
        let active_bg = match app.color_scheme {
            ccboard_core::models::config::ColorScheme::Dark => Color::Rgb(20, 40, 55),
            ccboard_core::models::config::ColorScheme::Light => Color::Rgb(200, 220, 240),
        };
        let titles: Vec<Line> = Tab::all()
            .iter()
            .map(|t| {
                let sc = t.shortcut();
                if *t == active {
                    let label = if sc.is_ascii_digit() {
                        format!("{} {}", sc, t.name())
                    } else {
                        t.name().to_string()
                    };
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            label,
                            Style::default()
                                .fg(p.focus)
                                .bg(active_bg)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(" "),
                    ])
                } else {
                    let inactive_label = if sc.is_ascii_digit() {
                        format!(" {} {} ", sc, t.name())
                    } else {
                        format!(" {} ", t.name())
                    };
                    Line::from(Span::styled(inactive_label, Style::default().fg(p.muted)))
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .select(active.index())
            .highlight_style(Style::default()) // disable Ratatui's default REVERSED highlight
            .divider(Span::styled("│", Style::default().fg(p.border)));

        frame.render_widget(tabs, tab_bar_chunks[1]);

        // Hints (right-aligned)
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("[?]", Style::default().fg(p.focus)),
                Span::styled(" help  ", Style::default().fg(p.muted)),
                Span::styled("[q]", Style::default().fg(p.focus)),
                Span::styled(" quit", Style::default().fg(p.muted)),
            ]))
            .alignment(Alignment::Right),
            tab_bar_chunks[2],
        );
    }

    fn render_degraded_banner(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &DegradedState,
        scheme: ccboard_core::models::config::ColorScheme,
    ) -> Rect {
        let p = Palette::new(scheme);
        match state {
            DegradedState::Healthy => area,
            DegradedState::PartialData { reason, .. } => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                let banner = Paragraph::new(Line::from(vec![
                    Span::styled(" ⚠ ", Style::default().fg(p.warning).bold()),
                    Span::styled(reason, Style::default().fg(p.warning)),
                ]))
                .style(Style::default().bg(p.muted));

                frame.render_widget(banner, chunks[0]);
                chunks[1]
            }
            DegradedState::ReadOnly { reason } => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(area);

                let banner = Paragraph::new(Line::from(vec![
                    Span::styled(" ⊘ READ-ONLY ", Style::default().fg(p.error).bold()),
                    Span::styled(reason, Style::default().fg(p.error)),
                ]))
                .style(Style::default().bg(p.muted));

                frame.render_widget(banner, chunks[0]);
                chunks[1]
            }
        }
    }

    fn render_tab_content(&mut self, frame: &mut Frame, area: Rect, app: &App) {
        let scheme = app.color_scheme;

        // If conversation is open, render it as overlay
        if self.conversation.is_open() {
            self.conversation.render(frame, area, scheme);
            return;
        }

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
                self.plugins.render(frame, area, &app.store, scheme);
            }
            Tab::Activity => {
                let sessions = app.store.recent_sessions(50);
                self.activity
                    .render(frame, area, &sessions, &app.store, scheme);
            }
            Tab::Search => {
                render_search_tab(&app.search_tab, frame, area, scheme);
            }
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: Rect, app: &App) {
        let p = Palette::new(app.color_scheme);
        let session_count = app.store.session_count();

        let status = if let Some(ref msg) = app.status_message {
            Line::from(vec![
                Span::styled(" ⚠ ", Style::default().fg(p.warning).bold()),
                Span::styled(msg.as_str(), Style::default().fg(p.warning)),
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
                Tab::Activity => "j/k navigate │ a analyze session │ Tab/Shift+Tab switch tabs",
                Tab::Search => "i type query │ Enter search/open │ j/k navigate │ ESC exit input",
            };

            Line::from(vec![
                Span::styled(
                    format!(" ● {} sessions ", session_count),
                    Style::default().fg(p.muted),
                ),
                Span::styled("│", Style::default().fg(p.muted)),
                Span::styled(" q", Style::default().fg(p.focus).bold()),
                Span::styled(" quit ", Style::default().fg(p.muted)),
                Span::styled("│", Style::default().fg(p.muted)),
                Span::styled(format!(" {}", hint), Style::default().fg(p.muted)),
            ])
        };

        let bar = Paragraph::new(status).style(Style::default().bg(p.muted));
        frame.render_widget(bar, area);
    }
}
