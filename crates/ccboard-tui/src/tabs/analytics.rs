//! Analytics tab - Trends, forecasting, patterns, insights, anomalies with 5 sub-views

use crate::empty_state;
use crate::theme::Palette;
use ccboard_core::analytics::{AnalyticsData, AnomalySeverity, Period};
use ccboard_core::store::DataStore;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, BarChart, Block, BorderType, Borders, Cell, Chart, Dataset, GraphType, List,
        ListItem, Paragraph, Row, Sparkline, Table,
    },
    Frame,
};
use std::sync::Arc;

/// Sub-view selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticsView {
    Overview,
    Trends,
    Patterns,
    Insights,
    Anomalies,
    /// Per-tool token usage and cost optimization suggestions
    Costs,
    /// Pattern discovery from session history
    Discover,
}

impl AnalyticsView {
    /// Cycle to next view
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Trends,
            Self::Trends => Self::Patterns,
            Self::Patterns => Self::Insights,
            Self::Insights => Self::Anomalies,
            Self::Anomalies => Self::Costs,
            Self::Costs => Self::Discover,
            Self::Discover => Self::Overview,
        }
    }

    /// Cycle to previous view
    pub fn prev(self) -> Self {
        match self {
            Self::Overview => Self::Discover,
            Self::Trends => Self::Overview,
            Self::Patterns => Self::Trends,
            Self::Insights => Self::Patterns,
            Self::Anomalies => Self::Insights,
            Self::Costs => Self::Anomalies,
            Self::Discover => Self::Costs,
        }
    }

    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Trends => "Trends",
            Self::Patterns => "Patterns",
            Self::Insights => "Summary",
            Self::Anomalies => "Anomalies",
            Self::Costs => "Costs",
            Self::Discover => "Discover",
        }
    }
}

/// State for the Discover sub-view
#[derive(Debug, Clone)]
struct DiscoverState {
    scroll: usize,
    /// Shared flag: true while async discover task is running.
    /// Arc<AtomicBool> so the spawned task can clear it without needing &mut self.
    loading: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Default for DiscoverState {
    fn default() -> Self {
        Self {
            scroll: 0,
            loading: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

/// Sort column for project leaderboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaderboardSortColumn {
    ProjectName,
    TotalSessions,
    TotalTokens,
    TotalCost,
    AvgSessionCost,
}

impl LeaderboardSortColumn {
    /// Cycle to next column
    pub fn next(self) -> Self {
        match self {
            Self::ProjectName => Self::TotalSessions,
            Self::TotalSessions => Self::TotalTokens,
            Self::TotalTokens => Self::TotalCost,
            Self::TotalCost => Self::AvgSessionCost,
            Self::AvgSessionCost => Self::ProjectName,
        }
    }

    /// Column header name
    pub fn header(&self) -> &'static str {
        match self {
            Self::ProjectName => "Project",
            Self::TotalSessions => "Sessions",
            Self::TotalTokens => "Tokens",
            Self::TotalCost => "Cost",
            Self::AvgSessionCost => "Avg Cost",
        }
    }
}

/// Analytics tab state
pub struct AnalyticsTab {
    /// Current period selection
    current_period: Period,
    /// Current sub-view
    current_view: AnalyticsView,
    /// Scroll offset for insights list
    scroll_offset: usize,
    /// Leaderboard sort column
    leaderboard_sort: LeaderboardSortColumn,
    /// Leaderboard sort descending
    leaderboard_sort_desc: bool,
    /// Scroll offset for tool cost breakdown table (Costs view)
    tool_cost_scroll: usize,
    /// Discover sub-view state
    discover: DiscoverState,
}

impl Default for AnalyticsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalyticsTab {
    pub fn new() -> Self {
        Self {
            current_period: Period::last_30d(),
            current_view: AnalyticsView::Overview,
            scroll_offset: 0,
            leaderboard_sort: LeaderboardSortColumn::TotalCost,
            leaderboard_sort_desc: true,
            tool_cost_scroll: 0,
            discover: DiscoverState::default(),
        }
    }

    /// Get current period
    pub fn period(&self) -> Period {
        self.current_period
    }

    /// Set period (F1-F4 keys)
    pub fn set_period(&mut self, period: Period) {
        self.current_period = period;
    }

    /// Cycle to next view (Tab key)
    pub fn next_view(&mut self) {
        self.current_view = self.current_view.next();
        self.scroll_offset = 0;
        self.tool_cost_scroll = 0;
    }

    /// Cycle to previous view (Shift+Tab key)
    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
        self.scroll_offset = 0;
        self.tool_cost_scroll = 0;
    }

    /// Scroll down (j key)
    pub fn scroll_down(&mut self, max_items: usize) {
        if self.current_view == AnalyticsView::Costs {
            if self.tool_cost_scroll + 1 < max_items {
                self.tool_cost_scroll += 1;
            }
        } else if self.current_view == AnalyticsView::Discover {
            self.discover.scroll = self.discover.scroll.saturating_add(1);
        } else if self.scroll_offset + 10 < max_items {
            self.scroll_offset += 1;
        }
    }

    /// Scroll up (k key)
    pub fn scroll_up(&mut self) {
        if self.current_view == AnalyticsView::Costs {
            if self.tool_cost_scroll > 0 {
                self.tool_cost_scroll -= 1;
            }
        } else if self.current_view == AnalyticsView::Discover {
            self.discover.scroll = self.discover.scroll.saturating_sub(1);
        } else if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Cycle sort column (s key)
    pub fn cycle_sort_column(&mut self) {
        self.leaderboard_sort = self.leaderboard_sort.next();
    }

    /// Toggle sort order (r key)
    pub fn toggle_sort_order(&mut self) {
        self.leaderboard_sort_desc = !self.leaderboard_sort_desc;
    }

    /// Get current view
    pub fn current_view(&self) -> AnalyticsView {
        self.current_view
    }

    /// Mark discover as loading and return a cloned Arc to the loading flag.
    ///
    /// The spawned async task uses the returned Arc to clear the flag on completion
    /// without requiring `&mut self`.
    pub fn begin_discover_loading(&mut self) -> std::sync::Arc<std::sync::atomic::AtomicBool> {
        self.discover
            .loading
            .store(true, std::sync::atomic::Ordering::Relaxed);
        std::sync::Arc::clone(&self.discover.loading)
    }

    /// Render the analytics tab
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        analytics: Option<&AnalyticsData>,
        store: Option<&Arc<DataStore>>,
        _scheme: ccboard_core::models::config::ColorScheme,
    ) {
        use tracing::debug;

        let p = Palette::new(_scheme);

        debug!(
            has_analytics = analytics.is_some(),
            "AnalyticsTab::render() called"
        );

        // Main layout: header + content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header (period selector + view tabs)
                Constraint::Min(10),   // Content
            ])
            .split(area);

        self.render_header(frame, chunks[0], analytics, store, &p);

        // Discover view renders independently of analytics data
        if self.current_view == AnalyticsView::Discover {
            self.render_discover(frame, chunks[1], store, &p);
            return;
        }

        match analytics {
            Some(data) => {
                debug!(
                    insights_count = data.insights.len(),
                    "Rendering analytics data"
                );
                match self.current_view {
                    AnalyticsView::Overview => {
                        self.render_overview(frame, chunks[1], data, store, &p)
                    }
                    AnalyticsView::Trends => self.render_trends(frame, chunks[1], data, &p),
                    AnalyticsView::Patterns => self.render_patterns(frame, chunks[1], data, &p),
                    AnalyticsView::Insights => self.render_insights(frame, chunks[1], data, &p),
                    AnalyticsView::Anomalies => self.render_anomalies(frame, chunks[1], data, &p),
                    AnalyticsView::Costs => self.render_costs(frame, chunks[1], data, &p),
                    AnalyticsView::Discover => unreachable!("handled above"),
                }
            }
            None => {
                debug!("No analytics data available, showing loading");
                self.render_loading(frame, chunks[1], &p)
            }
        }
    }

    /// Render header with period selector and view tabs
    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        _analytics: Option<&AnalyticsData>,
        _store: Option<&Arc<DataStore>>,
        p: &Palette,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(35), Constraint::Min(0)])
            .split(area);

        // Period selector (left)
        let periods: &[(&str, bool)] = &[
            ("F1:7d", matches!(self.current_period, Period::Days(7))),
            ("F2:30d", matches!(self.current_period, Period::Days(30))),
            ("F3:90d", matches!(self.current_period, Period::Days(90))),
            ("F4:All", matches!(self.current_period, Period::Available)),
        ];
        let mut period_text = vec![Span::styled(" ", Style::default())];
        for (label, is_active) in periods {
            if *is_active {
                period_text.push(Span::styled(
                    format!(" {} ", label),
                    Style::default()
                        .fg(p.bg)
                        .bg(p.focus)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                period_text.push(Span::styled(
                    format!(" {} ", label),
                    Style::default().fg(p.muted),
                ));
            }
            period_text.push(Span::raw(" "));
        }
        let period_para = Paragraph::new(Line::from(period_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface)),
            )
            .alignment(Alignment::Left);
        frame.render_widget(period_para, chunks[0]);

        // View tabs (right)
        let views = [
            AnalyticsView::Overview,
            AnalyticsView::Trends,
            AnalyticsView::Patterns,
            AnalyticsView::Insights,
            AnalyticsView::Anomalies,
            AnalyticsView::Costs,
            AnalyticsView::Discover,
        ];
        let mut tab_spans: Vec<Span> = Vec::new();
        for (i, view) in views.iter().enumerate() {
            if i > 0 {
                tab_spans.push(Span::styled("  │  ", Style::default().fg(p.border)));
            }
            let is_active = *view == self.current_view;
            if is_active {
                tab_spans.push(Span::styled(
                    view.name(),
                    Style::default()
                        .fg(p.focus)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ));
            } else {
                tab_spans.push(Span::styled(view.name(), Style::default().fg(p.muted)));
            }
        }
        let tab_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(p.border));
        let tab_inner = tab_block.inner(chunks[1]);
        frame.render_widget(tab_block, chunks[1]);
        frame.render_widget(
            Paragraph::new(Line::from(tab_spans)).alignment(Alignment::Center),
            tab_inner,
        );
    }

    /// Render loading state
    fn render_loading(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Analytics")
            .style(Style::default().fg(p.muted).bg(p.surface));

        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from("Computing analytics..."),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'r' to compute or wait for auto-computation",
                Style::default().fg(p.muted),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);

        frame.render_widget(para, area);
    }

    /// Render overview sub-view
    fn render_overview(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        store: Option<&Arc<DataStore>>,
        p: &Palette,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),  // Summary cards
                Constraint::Length(6),  // Budget status
                Constraint::Length(9),  // Token sparkline
                Constraint::Length(12), // Project leaderboard
                Constraint::Min(5),     // Top insights
            ])
            .split(area);

        // Summary cards
        self.render_summary_cards(frame, chunks[0], data, p);

        // Budget status (if configured)
        if let Some(store) = store {
            self.render_budget_status(frame, chunks[1], data, store, p);
        }

        // Token sparkline
        self.render_token_sparkline(frame, chunks[2], data, p);

        // Project leaderboard
        if let Some(store) = store {
            self.render_project_leaderboard(frame, chunks[3], store, p);
        }

        // Top insights preview
        self.render_insights_preview(frame, chunks[4], data, p);
    }

    /// Render summary cards
    fn render_summary_cards(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        // Total tokens
        let total_tokens: u64 = data.trends.daily_tokens.iter().sum();
        let tokens_display = Self::format_number(total_tokens);
        self.render_stat_card(
            frame,
            chunks[0],
            "◆ Tokens",
            &tokens_display,
            p.focus,
            "total",
            p,
        );

        // Total sessions
        let total_sessions: usize = data.trends.daily_sessions.iter().sum();
        self.render_stat_card(
            frame,
            chunks[1],
            "● Sessions",
            &total_sessions.to_string(),
            p.success,
            "count",
            p,
        );

        // Monthly cost estimate
        let cost_display = format!("${:.2}", data.forecast.monthly_cost_estimate);
        self.render_stat_card(
            frame,
            chunks[2],
            "$ Cost Est",
            &cost_display,
            p.warning,
            "monthly",
            p,
        );

        // Forecast confidence
        let confidence_display = format!("{:.0}%", data.forecast.confidence * 100.0);
        let confidence_color = if data.forecast.confidence > 0.7 {
            p.success
        } else if data.forecast.confidence > 0.4 {
            p.warning
        } else {
            p.error
        };
        self.render_stat_card(
            frame,
            chunks[3],
            "◉ Confidence",
            &confidence_display,
            confidence_color,
            "forecast",
            p,
        );
    }

    /// Render budget status with alerts
    fn render_budget_status(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        store: &Arc<DataStore>,
        p: &Palette,
    ) {
        use ccboard_core::analytics::generate_budget_alerts;

        let settings = store.settings();
        let budget_config = settings.merged.budget.as_ref();

        if let Some(config) = budget_config {
            let alerts = generate_budget_alerts(
                &data.trends,
                &data.forecast,
                config.monthly_limit,
                config.warning_threshold,
            );

            let current_cost = data.forecast.monthly_cost_estimate;
            let budget = config.monthly_limit.unwrap_or(0.0);
            let pct = if budget > 0.0 {
                (current_cost / budget * 100.0).min(100.0)
            } else {
                0.0
            };
            let remaining = (budget - current_cost).max(0.0);

            // Progress bar
            let bar_len = (pct / 5.0) as usize; // 20 chars max
            let bar = "━".repeat(bar_len.min(20));

            // Color based on percentage
            let (bar_color, status_icon) = if pct >= config.warning_threshold {
                (p.error, "⚠️ ")
            } else if pct >= 60.0 {
                (p.warning, "")
            } else {
                (p.success, "")
            };

            let mut lines = vec![
                Line::from(vec![
                    Span::styled("Monthly Est: ", Style::default().fg(p.muted)),
                    Span::styled(
                        format!("${:.2}", current_cost),
                        Style::default().fg(p.focus).bold(),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Budget:      ", Style::default().fg(p.muted)),
                    Span::styled(format!("${:.2}", budget), Style::default().fg(p.fg)),
                    Span::raw(" "),
                    Span::styled(bar, Style::default().fg(bar_color)),
                    Span::raw(" "),
                    Span::styled(
                        format!("{:.0}%", pct),
                        Style::default().fg(bar_color).bold(),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Remaining:   ", Style::default().fg(p.muted)),
                    Span::styled(
                        format!("${:.2} ({:.0}%)", remaining, 100.0 - pct),
                        Style::default().fg(p.fg),
                    ),
                ]),
            ];

            // Add alerts
            for alert in &alerts {
                use ccboard_core::analytics::Alert;
                match alert {
                    Alert::BudgetWarning { pct, .. } => {
                        lines.push(Line::from(vec![Span::raw("")]));
                        lines.push(Line::from(vec![Span::styled(
                            format!(
                                "{}WARNING: Approaching budget limit ({:.0}%)",
                                status_icon, pct
                            ),
                            Style::default().fg(p.error).bold(),
                        )]));
                    }
                    Alert::ProjectedOverage { overage, .. } => {
                        lines.push(Line::from(vec![Span::styled(
                            format!(
                                "💡 TIP: Projected overage: ${:.2} if trend continues",
                                overage
                            ),
                            Style::default().fg(p.warning),
                        )]));
                    }
                    _ => {}
                }
            }

            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(p.surface))
                        .title("Budget Status"),
                )
                .alignment(Alignment::Left);

            frame.render_widget(paragraph, area);
        } else {
            // No budget configured
            let text = vec![
                Line::from(Span::styled(
                    "No budget configured",
                    Style::default().fg(p.muted),
                )),
                Line::from(Span::styled(
                    "Add \"budget\": {\"monthlyBudgetUsd\": 50} to settings.json",
                    Style::default().fg(p.muted).italic(),
                )),
            ];

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(p.surface))
                        .title("Budget Status"),
                )
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
        }
    }

    /// Render stat card (reused from dashboard)
    #[allow(clippy::too_many_arguments)]
    fn render_stat_card(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        value: &str,
        color: Color,
        subtitle: &str,
        p: &Palette,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(p.surface))
            .title(Span::styled(
                title,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                value,
                Style::default().fg(p.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(subtitle, Style::default().fg(p.muted))),
        ];

        let para = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(para, area);
    }

    /// Format large numbers with K/M/B suffixes
    fn format_number(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{:.1}B", n as f64 / 1_000_000_000.0)
        } else if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }

    /// Render token sparkline
    fn render_token_sparkline(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let sparkline_data: Vec<u64> = data.trends.daily_tokens.to_vec();
        let max_val = sparkline_data.iter().max().copied().unwrap_or(1);

        // Outer block with title and borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(p.surface))
            .title("Token Usage Over Time");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Layout: [Y-axis labels (8 chars), Sparkline]
        let chart_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8), // Y-axis labels
                Constraint::Min(20),   // Chart
            ])
            .split(inner);

        // Y-axis labels (3 ticks: max, mid, 0)
        let max_label = Self::format_short(max_val);
        let mid_label = Self::format_short(max_val / 2);

        // Calculate vertical spacing to align with sparkline height
        let available_height = chart_layout[0].height as usize;
        let spacing = if available_height >= 3 {
            (available_height - 1) / 2 // Distribute remaining space
        } else {
            0
        };

        let mut y_labels = vec![max_label];
        for _ in 0..spacing {
            y_labels.push(String::new());
        }
        if available_height >= 2 {
            y_labels.push(mid_label);
        }
        for _ in 0..spacing {
            y_labels.push(String::new());
        }
        if available_height >= 3 {
            y_labels.push("0".to_string());
        }

        let y_axis_widget = Paragraph::new(y_labels.join("\n"))
            .alignment(ratatui::layout::Alignment::Right)
            .style(Style::default().fg(p.muted));
        frame.render_widget(y_axis_widget, chart_layout[0]);

        // Sparkline in remaining area
        let sparkline = Sparkline::default()
            .data(&sparkline_data)
            .style(Style::default().fg(p.focus))
            .max(max_val);

        frame.render_widget(sparkline, chart_layout[1]);
    }

    /// Render insights preview (top 3)
    fn render_insights_preview(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let insights: Vec<ListItem> = data
            .insights
            .iter()
            .take(3)
            .map(|insight| {
                let icon = if insight.contains("Peak hours") {
                    "⏰"
                } else if insight.contains("Opus") || insight.contains("Cost") {
                    "💰"
                } else if insight.contains("Weekend") {
                    "📅"
                } else {
                    "💡"
                };
                ListItem::new(format!("{} {}", icon, insight))
            })
            .collect();

        let list = List::new(insights)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Top Insights (Tab to Insights view for all)"),
            )
            .style(Style::default().fg(p.fg));

        frame.render_widget(list, area);
    }

    /// Render trends sub-view (time series charts)
    fn render_trends(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData, p: &Palette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(p.surface))
            .title("Trends - Token & Session Activity Over Time");

        // Prepare data points for chart
        let token_data: Vec<(f64, f64)> = data
            .trends
            .daily_tokens
            .iter()
            .enumerate()
            .map(|(i, &tokens)| (i as f64, tokens as f64))
            .collect();

        let session_data: Vec<(f64, f64)> = data
            .trends
            .daily_sessions
            .iter()
            .enumerate()
            .map(|(i, &sessions)| (i as f64, sessions as f64 * 100.0)) // Scale for visibility
            .collect();

        // Forecast line using linear regression (30 days ahead)
        let forecast_data = if data.forecast.unavailable_reason.is_none() && !token_data.is_empty()
        {
            let last_day = token_data.len() as f64 - 1.0;
            let last_tokens = token_data.last().map(|p| p.1).unwrap_or(0.0);

            // Compute linear regression: y = slope * x + intercept
            let n = token_data.len() as f64;
            let sum_x: f64 = (0..token_data.len()).map(|i| i as f64).sum();
            let sum_y: f64 = data.trends.daily_tokens.iter().map(|&t| t as f64).sum();
            let sum_xx: f64 = (0..token_data.len()).map(|i| (i as f64).powi(2)).sum();
            let sum_xy: f64 = token_data.iter().map(|(x, y)| x * y).sum();

            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
            let intercept = (sum_y - slope * sum_x) / n;

            // Generate forecast points (30 days ahead using regression line)
            let mut points = vec![(last_day, last_tokens)];
            for i in 1..=30 {
                let x = last_day + i as f64;
                let y = (slope * x + intercept).max(0.0); // Linear projection
                points.push((x, y));
            }
            points
        } else {
            vec![]
        };

        let max_tokens = data.trends.daily_tokens.iter().max().copied().unwrap_or(1) as f64;
        let max_with_forecast = if !forecast_data.is_empty() {
            forecast_data.iter().map(|p| p.1).fold(max_tokens, f64::max)
        } else {
            max_tokens
        };

        let mut datasets = vec![
            Dataset::default()
                .name("Historical Tokens")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(p.focus))
                .data(&token_data),
            Dataset::default()
                .name("Sessions (×100)")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(p.success))
                .data(&session_data),
        ];

        // Add 30d forecast dataset (orange dashed line)
        if !forecast_data.is_empty() {
            let forecast_color = if data.forecast.confidence > 0.4 {
                p.warning // Medium/high confidence
            } else {
                p.error // Low confidence
            };

            datasets.push(
                Dataset::default()
                    .name(format!(
                        "30d Forecast ({:.0}% conf)",
                        data.forecast.confidence * 100.0
                    ))
                    .marker(symbols::Marker::Dot) // Dotted line for forecast
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(forecast_color))
                    .data(&forecast_data),
            );
        }

        // X-axis bounds: historical + 30 days forecast
        let x_max = if !forecast_data.is_empty() {
            data.trends.dates.len() as f64 + 30.0
        } else {
            data.trends.dates.len() as f64
        };

        let x_labels = vec![
            Span::raw("0"),
            Span::raw(format!("{}", data.trends.dates.len() / 2)),
            Span::raw(format!("{}", data.trends.dates.len())),
            Span::styled(
                "+30d",
                Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
            ),
        ];

        let y_labels = vec![
            Span::raw("0"),
            Span::raw(Self::format_number((max_with_forecast / 2.0) as u64)),
            Span::raw(Self::format_number(max_with_forecast as u64)),
        ];

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(
                Axis::default()
                    .title("Days")
                    .style(Style::default().fg(p.muted))
                    .labels(x_labels)
                    .bounds([0.0, x_max]),
            )
            .y_axis(
                Axis::default()
                    .title("Tokens")
                    .style(Style::default().fg(p.muted))
                    .labels(y_labels)
                    .bounds([0.0, max_with_forecast * 1.1]),
            );

        frame.render_widget(chart, area);
    }

    /// Render patterns sub-view (bar charts)
    fn render_patterns(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData, p: &Palette) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(10),    // Activity Heatmap — takes all available space
                Constraint::Length(12), // Most Used Tools — fixed
                Constraint::Length(12), // Model Distribution + Duration — fixed
            ])
            .split(area);

        // Activity Heatmap (GitHub-style)
        self.render_activity_heatmap(frame, chunks[0], data, p);

        // Most Used Tools (horizontal bar chart)
        self.render_most_used_tools(frame, chunks[1], data, p);

        // Model distribution & duration stats (side by side)
        let bottom_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);

        self.render_model_distribution(frame, bottom_chunks[0], data, p);
        self.render_duration_stats(frame, bottom_chunks[1], data, p);
    }

    /// Render activity heatmap (GitHub-style 7 days x 24 hours)
    /// Cells are sized dynamically to fill the available area.
    fn render_activity_heatmap(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let heatmap = &data.patterns.activity_heatmap;

        // Find max value for color scaling
        let max_activity = heatmap
            .iter()
            .flat_map(|row| row.iter())
            .max()
            .copied()
            .unwrap_or(1);

        let weekday_labels = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

        // --- Calculate responsive cell dimensions ---
        let label_w = 4usize; // "Mon "
        let inner_w = area.width.saturating_sub(2) as usize; // minus block borders
        let grid_w = inner_w.saturating_sub(label_w);
        let cell_w = (grid_w / 24).max(1);

        let inner_h = area.height.saturating_sub(2) as usize; // minus block borders
                                                              // Reserve: 1 header + 2 blank + 1 legend = 4 fixed rows
        let available_for_days = inner_h.saturating_sub(4);
        // Cap cell height at 5 lines so cells stay compact and readable
        let cell_h = (available_for_days / 7).clamp(1, 5);

        let mut lines = vec![];

        // --- Header: hour labels positioned at correct column offsets ---
        let mut header_chars: Vec<char> = vec![' '; label_w + 24 * cell_w];
        for (hour, label) in [
            (0, "00"),
            (4, "04"),
            (8, "08"),
            (12, "12"),
            (16, "16"),
            (20, "20"),
        ] {
            let pos = label_w + hour * cell_w;
            if pos + 2 <= header_chars.len() {
                header_chars[pos] = label.chars().next().unwrap_or(' ');
                header_chars[pos + 1] = label.chars().nth(1).unwrap_or(' ');
            }
        }
        lines.push(Line::from(Span::styled(
            header_chars.into_iter().collect::<String>(),
            Style::default().fg(p.muted),
        )));

        // --- Heatmap rows: each day occupies cell_h terminal lines ---
        for (day_idx, day_label) in weekday_labels.iter().enumerate() {
            let row_activity: Vec<u8> = heatmap[day_idx]
                .iter()
                .map(|&activity| {
                    if max_activity > 0 {
                        (activity as f64 / max_activity as f64 * 4.0) as u8
                    } else {
                        0
                    }
                })
                .collect();

            for sub_row in 0..cell_h {
                // Show day label on the middle sub-row only
                let prefix = if sub_row == cell_h / 2 {
                    Span::styled(
                        format!("{:<width$}", day_label, width = label_w),
                        Style::default().fg(p.muted),
                    )
                } else {
                    Span::raw(format!("{:width$}", "", width = label_w))
                };

                let mut row_spans = vec![prefix];
                for &intensity in &row_activity {
                    let color = match intensity {
                        0 => p.muted,
                        1 => p.success,
                        2 => p.focus,
                        3 => p.warning,
                        _ => p.important,
                    };
                    row_spans.push(Span::styled("█".repeat(cell_w), Style::default().fg(color)));
                }
                lines.push(Line::from(row_spans));
            }
        }

        // --- Legend (2 lines for breathing room) ---
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("██", Style::default().fg(p.muted)),
            Span::styled("  No activity    ", Style::default().fg(p.muted)),
            Span::styled("██", Style::default().fg(p.success)),
            Span::styled("  Low    ", Style::default().fg(p.muted)),
            Span::styled("██", Style::default().fg(p.focus)),
            Span::styled("  Medium    ", Style::default().fg(p.muted)),
            Span::styled("██", Style::default().fg(p.warning)),
            Span::styled("  High    ", Style::default().fg(p.muted)),
            Span::styled("██", Style::default().fg(p.important)),
            Span::styled("  Peak", Style::default().fg(p.muted)),
        ]));

        // Streak info in title
        let streak = data.patterns.current_streak_days;
        let longest = data.patterns.longest_streak_days;
        let streak_label = if streak > 0 {
            format!(" 🔥 {} day streak (best: {})", streak, longest)
        } else {
            format!(" best: {} days", longest)
        };
        let heatmap_title = format!("Activity Heatmap{}", streak_label);

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title(heatmap_title),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render most used tools (horizontal bar chart)
    fn render_most_used_tools(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        // Sort tools by usage count (descending)
        let mut tools: Vec<(&String, &usize)> = data.patterns.tool_usage.iter().collect();
        tools.sort_by(|a, b| b.1.cmp(a.1));

        // Take top 6 tools
        let top_tools: Vec<(&str, usize)> = tools
            .iter()
            .take(6)
            .map(|(name, count)| (name.as_str(), **count))
            .collect();

        if top_tools.is_empty() {
            // No tool data available
            let text = vec![Line::from(Span::styled(
                "No tool usage data available",
                Style::default().fg(p.muted),
            ))];
            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(p.surface))
                        .title("Most Used Tools"),
                )
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        // Calculate percentages
        let total: usize = tools.iter().map(|(_, count)| **count).sum();
        let max_count = top_tools.iter().map(|(_, count)| *count).max().unwrap_or(1);

        let mut lines = vec![];
        let colors = [p.focus, p.success, p.focus, p.important, p.warning, p.error];

        for (i, (tool_name, count)) in top_tools.iter().enumerate() {
            let pct = if total > 0 {
                *count as f64 / total as f64 * 100.0
            } else {
                0.0
            };

            // Bar length proportional to count (max 40 chars)
            let bar_len = ((*count as f64 / max_count as f64) * 40.0) as usize;
            let bar = "━".repeat(bar_len);

            let color = colors[i % colors.len()];

            lines.push(Line::from(vec![
                Span::styled(format!("{:<15}", tool_name), Style::default().fg(p.fg)),
                Span::styled(bar, Style::default().fg(color)),
                Span::raw(" "),
                Span::styled(format!("{} ", count), Style::default().fg(color).bold()),
                Span::styled(format!("{:.1}%", pct), Style::default().fg(p.muted)),
            ]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Most Used Tools"),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render hourly distribution bar chart
    fn _render_hourly_distribution(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        // Group hours into 6 blocks (4-hour chunks)
        let mut hour_blocks = [0; 6];
        for (hour, count) in data.patterns.hourly_distribution.iter().enumerate() {
            let block_idx = hour / 4;
            hour_blocks[block_idx] += count;
        }

        let bar_data: Vec<(&str, u64)> = vec![
            ("00-04", hour_blocks[0] as u64),
            ("04-08", hour_blocks[1] as u64),
            ("08-12", hour_blocks[2] as u64),
            ("12-16", hour_blocks[3] as u64),
            ("16-20", hour_blocks[4] as u64),
            ("20-24", hour_blocks[5] as u64),
        ];

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Hourly Distribution (Session Count)"),
            )
            .data(&bar_data)
            .bar_width(9)
            .bar_gap(2)
            .bar_style(Style::default().fg(p.focus))
            .value_style(Style::default().fg(p.bg).bg(p.focus));

        frame.render_widget(barchart, area);
    }

    /// Render model distribution
    fn render_model_distribution(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let mut model_data: Vec<(&str, u64)> = data
            .patterns
            .model_distribution
            .iter()
            .map(|(model, pct)| (model.as_str(), (*pct * 1000.0) as u64)) // Scale for bar height
            .collect();

        model_data.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by percentage desc
        model_data.truncate(5); // Top 5 models

        let barchart = BarChart::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Model Distribution (Token %)"),
            )
            .data(&model_data)
            .bar_width(12)
            .bar_gap(2)
            .bar_style(Style::default().fg(p.success))
            .value_style(Style::default().fg(p.bg).bg(p.success));

        frame.render_widget(barchart, area);
    }

    /// Render session duration statistics
    fn render_duration_stats(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let stats = &data.trends.duration_stats;

        // Format duration as minutes/seconds
        let format_duration = |secs: f64| {
            let mins = (secs / 60.0).floor() as u64;
            let secs = (secs % 60.0) as u64;
            if mins > 0 {
                format!("{}m {:02}s", mins, secs)
            } else {
                format!("{}s", secs)
            }
        };

        let text = vec![
            Line::from(vec![
                Span::styled("Avg: ", Style::default().fg(p.muted)),
                Span::styled(
                    format_duration(stats.avg_duration_secs),
                    Style::default().fg(p.focus).bold(),
                ),
                Span::styled("  (median: ", Style::default().fg(p.muted)),
                Span::styled(
                    format_duration(stats.median_duration_secs),
                    Style::default().fg(p.focus),
                ),
                Span::styled(")", Style::default().fg(p.muted)),
            ]),
            Line::from(vec![
                Span::styled("P95: ", Style::default().fg(p.muted)),
                Span::styled(
                    format_duration(stats.p95_duration_secs),
                    Style::default().fg(p.warning).bold(),
                ),
                Span::styled("  (95% sessions < this)", Style::default().fg(p.muted)),
            ]),
            Line::from(vec![
                Span::styled("Range: ", Style::default().fg(p.muted)),
                Span::styled(
                    format_duration(stats.shortest_session_secs as f64),
                    Style::default().fg(p.success),
                ),
                Span::styled(" → ", Style::default().fg(p.muted)),
                Span::styled(
                    format_duration(stats.longest_session_secs as f64),
                    Style::default().fg(p.error),
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Distribution:",
                Style::default().fg(p.muted).bold(),
            )]),
        ];

        // Distribution bars
        let total: usize = stats.distribution.iter().sum();
        let mut distrib_lines = vec![];
        let labels = ["0-5m", "5-15m", "15-30m", "30-60m", "60m+"];
        for (i, &count) in stats.distribution.iter().enumerate() {
            if total > 0 {
                let pct = (count as f64 / total as f64 * 100.0) as usize;
                let bar_len = (pct / 5).min(20); // Max 20 chars
                let bar = "█".repeat(bar_len);
                distrib_lines.push(Line::from(vec![
                    Span::styled(format!("{:6}", labels[i]), Style::default().fg(p.muted)),
                    Span::raw(" "),
                    Span::styled(bar, Style::default().fg(p.focus)),
                    Span::raw(" "),
                    Span::styled(format!("{}%", pct), Style::default().fg(p.fg)),
                ]));
            }
        }

        let all_lines = [text, distrib_lines].concat();

        let paragraph = Paragraph::new(all_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Session Duration Statistics"),
            )
            .alignment(Alignment::Left);

        frame.render_widget(paragraph, area);
    }

    /// Render insights sub-view (scrollable list)
    fn render_insights(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData, p: &Palette) {
        // Empty state when no insights computed
        if data.insights.is_empty() {
            let period_label = self.current_period.display(0);
            let empty = empty_state::no_insights(period_label);
            frame.render_widget(empty, area);
            return;
        }

        let insights: Vec<ListItem> = data
            .insights
            .iter()
            .skip(self.scroll_offset)
            .map(|insight| {
                let (icon, color) = if insight.contains("Peak hours") {
                    ("⏰", p.warning)
                } else if insight.contains("Opus") {
                    ("💰", p.error)
                } else if insight.contains("Cost") || insight.contains("premium") {
                    ("💸", p.important)
                } else if insight.contains("Weekend") {
                    ("📅", p.focus)
                } else if insight.contains("confidence") {
                    ("⚠️", p.warning)
                } else {
                    ("💡", p.success)
                };

                ListItem::new(Line::from(vec![
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(insight.clone(), Style::default().fg(color)),
                ]))
            })
            .collect();

        let scroll_indicator = if data.insights.len() > 10 {
            format!(
                " (scroll: {}/{})",
                self.scroll_offset + 1,
                data.insights.len()
            )
        } else {
            String::new()
        };

        let list = List::new(insights)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title(format!("Actionable Insights{}", scroll_indicator)),
            )
            .style(Style::default().fg(p.fg));

        frame.render_widget(list, area);
    }

    /// Format large numbers with K/M/B suffixes
    fn format_short(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{}B", n / 1_000_000_000)
        } else if n >= 1_000_000 {
            format!("{}M", n / 1_000_000)
        } else if n >= 1_000 {
            format!("{}K", n / 1_000)
        } else {
            n.to_string()
        }
    }

    /// Render project leaderboard table
    fn render_project_leaderboard(
        &self,
        frame: &mut Frame,
        area: Rect,
        store: &Arc<DataStore>,
        p: &Palette,
    ) {
        // Get leaderboard data
        let mut entries = store.projects_leaderboard();

        // Sort based on current selection
        match self.leaderboard_sort {
            LeaderboardSortColumn::ProjectName => {
                entries.sort_by(|a, b| {
                    if self.leaderboard_sort_desc {
                        b.project_name.cmp(&a.project_name)
                    } else {
                        a.project_name.cmp(&b.project_name)
                    }
                });
            }
            LeaderboardSortColumn::TotalSessions => {
                entries.sort_by(|a, b| {
                    if self.leaderboard_sort_desc {
                        b.total_sessions.cmp(&a.total_sessions)
                    } else {
                        a.total_sessions.cmp(&b.total_sessions)
                    }
                });
            }
            LeaderboardSortColumn::TotalTokens => {
                entries.sort_by(|a, b| {
                    if self.leaderboard_sort_desc {
                        b.total_tokens.cmp(&a.total_tokens)
                    } else {
                        a.total_tokens.cmp(&b.total_tokens)
                    }
                });
            }
            LeaderboardSortColumn::TotalCost => {
                entries.sort_by(|a, b| {
                    let cmp = b
                        .total_cost
                        .partial_cmp(&a.total_cost)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if self.leaderboard_sort_desc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            LeaderboardSortColumn::AvgSessionCost => {
                entries.sort_by(|a, b| {
                    let cmp = b
                        .avg_session_cost
                        .partial_cmp(&a.avg_session_cost)
                        .unwrap_or(std::cmp::Ordering::Equal);
                    if self.leaderboard_sort_desc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
        }

        // Create header with sort indicators
        let sort_indicator = if self.leaderboard_sort_desc {
            "↓"
        } else {
            "↑"
        };

        let header_cells = [
            LeaderboardSortColumn::ProjectName,
            LeaderboardSortColumn::TotalSessions,
            LeaderboardSortColumn::TotalTokens,
            LeaderboardSortColumn::TotalCost,
            LeaderboardSortColumn::AvgSessionCost,
        ]
        .iter()
        .map(|col| {
            let header = if *col == self.leaderboard_sort {
                format!("{} {}", col.header(), sort_indicator)
            } else {
                col.header().to_string()
            };

            Cell::from(header).style(Style::default().fg(p.warning).add_modifier(Modifier::BOLD))
        });

        let header = Row::new(header_cells).height(1).bottom_margin(1);

        // Create rows (top 5 projects)
        let rows: Vec<Row> = entries
            .iter()
            .take(5)
            .enumerate()
            .map(|(idx, entry)| {
                // Highlight top 3 with different colors
                let row_color = match idx {
                    0 => p.success,
                    1 => p.focus,
                    2 => p.warning,
                    _ => p.fg,
                };

                let cells = vec![
                    Cell::from(entry.project_name.clone()).style(Style::default().fg(row_color)),
                    Cell::from(entry.total_sessions.to_string())
                        .style(Style::default().fg(row_color)),
                    Cell::from(Self::format_number(entry.total_tokens))
                        .style(Style::default().fg(row_color)),
                    Cell::from(format!("${:.2}", entry.total_cost))
                        .style(Style::default().fg(row_color)),
                    Cell::from(format!("${:.2}", entry.avg_session_cost))
                        .style(Style::default().fg(row_color)),
                ];

                Row::new(cells).height(1)
            })
            .collect();

        let widths = [
            Constraint::Percentage(40), // Project name
            Constraint::Percentage(15), // Sessions
            Constraint::Percentage(15), // Tokens
            Constraint::Percentage(15), // Cost
            Constraint::Percentage(15), // Avg Cost
        ];

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title("Project Leaderboard (Top 5) - [s] sort column | [o] order"),
            )
            .column_spacing(1);

        frame.render_widget(table, area);
    }

    /// Render anomalies sub-view (Z-score based anomaly detection)
    fn render_anomalies(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData, p: &Palette) {
        let anomalies = &data.anomalies;
        let daily_spikes = &data.daily_spikes;
        let session_count = data.sessions_in_period;
        let thresholds = &data.anomaly_thresholds;

        // Split area: thresholds hint (1 line) + daily cost spikes (6 lines) + anomaly table (rest)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(area);

        // Render active thresholds hint
        let is_custom = thresholds.warning_z_score != 2.0
            || thresholds.critical_z_score != 3.0
            || thresholds.spike_2x != 2.0
            || thresholds.spike_3x != 3.0;
        let hint_style = if is_custom {
            Style::default().fg(p.important)
        } else {
            Style::default().fg(p.muted)
        };
        let hint = format!(
            " Thresholds: warn >{:.1}σ  crit >{:.1}σ  spike ≥{:.1}x/≥{:.1}x  min {} sessions{}",
            thresholds.warning_z_score,
            thresholds.critical_z_score,
            thresholds.spike_2x,
            thresholds.spike_3x,
            thresholds.min_sessions,
            if is_custom { "  [custom]" } else { "" },
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(Span::styled(hint, hint_style))),
            chunks[0],
        );

        let chunks = &chunks[1..];

        // Render daily cost spikes panel
        {
            let spike_title = format!(" Daily Cost Spikes ({} detected) ", daily_spikes.len());
            let spike_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(spike_title)
                .style(Style::default().fg(p.border).bg(p.surface));

            if daily_spikes.is_empty() {
                let para = Paragraph::new(vec![Line::from(Span::styled(
                    "✅  No daily cost spikes — usage within normal range",
                    Style::default().fg(p.success),
                ))])
                .block(spike_block)
                .alignment(Alignment::Center);
                frame.render_widget(para, chunks[0]);
            } else {
                let items: Vec<ListItem> = daily_spikes
                    .iter()
                    .take(3)
                    .map(|s| {
                        let color = match s.severity {
                            AnomalySeverity::Critical => p.error,
                            AnomalySeverity::Warning => p.warning,
                        };
                        ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("{} ", s.severity.icon()),
                                Style::default().fg(color),
                            ),
                            Span::styled(
                                s.date.format("%Y-%m-%d").to_string(),
                                Style::default().fg(p.fg),
                            ),
                            Span::styled(
                                format!("  {}  (avg ${:.3})", s.format_cost(), s.avg_cost),
                                Style::default().fg(p.muted),
                            ),
                            Span::styled(
                                format!("  {}", s.format_ratio()),
                                Style::default().fg(color).bold(),
                            ),
                        ]))
                    })
                    .collect();
                let list = List::new(items).block(spike_block);
                frame.render_widget(list, chunks[0]);
            }
        }

        let area = chunks[1];

        // Check minimum data requirement
        if session_count < 10 {
            let text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Insufficient data for anomaly detection",
                    Style::default().fg(p.warning).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!("Found {} sessions (minimum 10 required)", session_count),
                    Style::default().fg(p.muted),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Anomaly detection uses Z-score statistical analysis",
                    Style::default().fg(p.muted).italic(),
                )),
                Line::from(Span::styled(
                    "to identify sessions with unusual token usage or costs.",
                    Style::default().fg(p.muted).italic(),
                )),
            ];

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(p.surface))
                        .title("Anomaly Detection"),
                )
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        // Check if no anomalies found
        if anomalies.is_empty() {
            let text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "✅ No anomalies detected",
                    Style::default().fg(p.success).bold(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!(
                        "Analyzed {} sessions - All within normal range",
                        session_count
                    ),
                    Style::default().fg(p.muted),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Anomalies are flagged when tokens or cost exceed 2σ from mean",
                    Style::default().fg(p.muted).italic(),
                )),
            ];

            let paragraph = Paragraph::new(text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(p.surface))
                        .title("Anomaly Detection"),
                )
                .alignment(Alignment::Center);

            frame.render_widget(paragraph, area);
            return;
        }

        // Render anomaly table
        let header = Row::new(vec![
            Cell::from(Span::styled("Severity", Style::default().bold())),
            Cell::from(Span::styled("Date", Style::default().bold())),
            Cell::from(Span::styled("Session ID", Style::default().bold())),
            Cell::from(Span::styled("Metric", Style::default().bold())),
            Cell::from(Span::styled("Value", Style::default().bold())),
            Cell::from(Span::styled("Z-Score", Style::default().bold())),
            Cell::from(Span::styled("Deviation", Style::default().bold())),
        ])
        .style(Style::default().fg(p.warning))
        .height(1);

        // Take top 10 anomalies (they're already sorted by severity)
        let rows: Vec<Row> = anomalies
            .iter()
            .take(10)
            .skip(self.scroll_offset)
            .map(|anomaly| {
                // Severity column with emoji and color
                let (severity_color, severity_icon) = match anomaly.severity {
                    AnomalySeverity::Critical => (p.error, "🚨"),
                    AnomalySeverity::Warning => (p.warning, "⚠️"),
                };

                let cells = vec![
                    Cell::from(Span::styled(
                        severity_icon,
                        Style::default().fg(severity_color).bold(),
                    )),
                    Cell::from(anomaly.date.clone()),
                    Cell::from(Span::styled(
                        anomaly
                            .session_id
                            .as_str()
                            .get(..8)
                            .unwrap_or(anomaly.session_id.as_str()),
                        Style::default().fg(p.focus),
                    )),
                    Cell::from(anomaly.metric.name()),
                    Cell::from(Span::styled(
                        anomaly.format_value(),
                        Style::default().fg(severity_color).bold(),
                    )),
                    Cell::from(Span::styled(
                        format!("{:.2}", anomaly.z_score),
                        Style::default().fg(p.fg),
                    )),
                    Cell::from(Span::styled(
                        anomaly.format_deviation(),
                        Style::default().fg(severity_color),
                    )),
                ];

                Row::new(cells).height(1)
            })
            .collect();

        let widths = [
            Constraint::Length(4),      // Severity emoji
            Constraint::Length(16),     // Date
            Constraint::Length(10),     // Session ID (truncated)
            Constraint::Length(8),      // Metric
            Constraint::Length(12),     // Value
            Constraint::Length(8),      // Z-Score
            Constraint::Percentage(15), // Deviation
        ];

        let scroll_indicator = if anomalies.len() > 10 {
            format!(
                " (showing {} of {}) - [j/k] scroll",
                (self.scroll_offset + 10).min(anomalies.len()),
                anomalies.len()
            )
        } else {
            format!(" (showing all {} anomalies)", anomalies.len())
        };

        let critical_count = anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .count();

        let title = format!(
            "Anomaly Detection - {} critical, {} total{}",
            critical_count,
            anomalies.len(),
            scroll_indicator
        );

        let table = Table::new(rows, widths)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(Style::default().bg(p.surface))
                    .title(title),
            )
            .column_spacing(1);

        frame.render_widget(table, area);
    }

    /// Render Plugins sub-view: per-tool token usage + cost optimization suggestions
    fn render_costs(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData, p: &Palette) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(55), // Tool token bar chart
                Constraint::Percentage(45), // Cost suggestions
            ])
            .split(area);

        self.render_tool_token_chart(frame, chunks[0], data, p);
        self.render_cost_suggestions(frame, chunks[1], data, p);
    }

    /// Render per-tool token usage as a bar chart
    fn render_tool_token_chart(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        // Aggregate tool_token_usage across available data
        // AnalyticsData doesn't store per-tool tokens directly, so we show tool chains
        // as a proxy. If tool_chains is present, show top tools from bigrams.
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Tool Token Distribution (by call frequency) ")
            .style(Style::default().fg(p.border).bg(p.surface));

        if let Some(ref chains) = data.tool_chains {
            if chains.top_bigrams.is_empty() {
                let para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        "No tool chain data available.",
                        Style::default().fg(p.muted),
                    )),
                    Line::from(Span::styled(
                        "Run more sessions with tool calls to see patterns.",
                        Style::default().fg(p.muted),
                    )),
                ])
                .block(block)
                .alignment(Alignment::Center);
                frame.render_widget(para, area);
                return;
            }

            // Build bar data from top bigrams (frequency as proxy for cost)
            let bar_data: Vec<(&str, u64)> = chains
                .top_bigrams
                .iter()
                .take(15)
                .map(|c| {
                    // Use static string slices from sequence — take first tool name
                    (c.sequence[0].as_str(), c.frequency as u64)
                })
                .collect();

            // Deduplicate by tool name (keep highest frequency)
            let mut seen_tools: std::collections::HashMap<&str, u64> =
                std::collections::HashMap::new();
            for (tool, freq) in &bar_data {
                let entry = seen_tools.entry(tool).or_insert(0);
                if *freq > *entry {
                    *entry = *freq;
                }
            }

            let mut sorted_tools: Vec<(&str, u64)> = seen_tools.into_iter().collect();
            sorted_tools.sort_by(|a, b| b.1.cmp(&a.1));
            sorted_tools.truncate(10);

            if sorted_tools.is_empty() {
                let para = Paragraph::new("No data").block(block);
                frame.render_widget(para, area);
                return;
            }

            let chart_data: Vec<(&str, u64)> = sorted_tools;

            let bar_chart = BarChart::default()
                .block(block)
                .data(&chart_data)
                .bar_width(7)
                .bar_gap(1)
                .bar_style(Style::default().fg(p.focus))
                .value_style(Style::default().fg(p.bg).add_modifier(Modifier::BOLD))
                .label_style(Style::default().fg(p.fg));

            frame.render_widget(bar_chart, area);
        } else {
            let para = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Tool analytics not yet computed.",
                    Style::default().fg(p.muted),
                )),
                Line::from(Span::styled(
                    "Press 'r' to refresh.",
                    Style::default().fg(p.muted),
                )),
            ])
            .block(block)
            .alignment(Alignment::Center);
            frame.render_widget(para, area);
        }
    }

    /// Render cost optimization suggestions list
    fn render_cost_suggestions(
        &self,
        frame: &mut Frame,
        area: Rect,
        data: &AnalyticsData,
        p: &Palette,
    ) {
        let title = format!(
            " Cost Optimization ({} suggestions) ",
            data.cost_suggestions.len()
        );
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .style(Style::default().fg(p.border).bg(p.surface));

        if data.cost_suggestions.is_empty() {
            let para = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No optimization suggestions — your setup looks efficient.",
                    Style::default().fg(p.success),
                )),
            ])
            .block(block)
            .alignment(Alignment::Center);
            frame.render_widget(para, area);
            return;
        }

        let visible = (area.height as usize).saturating_sub(2);
        let start = self
            .scroll_offset
            .min(data.cost_suggestions.len().saturating_sub(1));

        let items: Vec<ListItem> = data
            .cost_suggestions
            .iter()
            .skip(start)
            .take(visible)
            .map(|s| {
                let savings_str = if s.potential_savings > 0.01 {
                    format!(" [-${:.2}/mo]", s.potential_savings)
                } else {
                    String::new()
                };
                let line1 = Line::from(vec![
                    Span::styled(
                        format!("{} {} ", s.category.icon(), s.category.label()),
                        Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("{}{}", s.title, savings_str),
                        Style::default().fg(p.fg),
                    ),
                ]);
                let line2 = Line::from(Span::styled(
                    format!("   {} {}", "→", s.action),
                    Style::default().fg(p.muted),
                ));
                ListItem::new(vec![line1, line2])
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }

    /// Render Discover sub-view — pattern discovery from session history
    fn render_discover(
        &self,
        frame: &mut Frame,
        area: Rect,
        store: Option<&Arc<DataStore>>,
        p: &Palette,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let suggestions = store.and_then(|s| s.discover());
        let suggestion_count = suggestions.as_ref().map(|v| v.len()).unwrap_or(0);
        let is_loading = self
            .discover
            .loading
            .load(std::sync::atomic::Ordering::Relaxed);

        let status_text = if is_loading {
            " Analyzing session history for patterns... ".to_string()
        } else if suggestion_count > 0 {
            format!(
                " {} suggestions found  [r] re-run  [j/k] scroll ",
                suggestion_count
            )
        } else {
            " Press [r] to discover recurring patterns in your session history ".to_string()
        };

        let status_style = if is_loading {
            Style::default().fg(p.warning)
        } else {
            Style::default().fg(p.muted)
        };

        let status = Paragraph::new(status_text).style(status_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Pattern Discovery ")
                .style(Style::default().fg(p.border).bg(p.surface)),
        );
        frame.render_widget(status, chunks[0]);

        let has_results = suggestion_count > 0;
        if !has_results {
            let msg = if is_loading {
                "Scanning session history for recurring patterns..."
            } else {
                "No patterns discovered yet. Press [r] to analyze your session history."
            };
            let para = Paragraph::new(msg)
                .style(Style::default().fg(p.muted))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(Style::default().fg(p.border).bg(p.surface)),
                );
            frame.render_widget(para, chunks[1]);
        } else if let Some(ref suggestions) = suggestions {
            let items: Vec<ListItem> = suggestions
                .iter()
                .skip(self.discover.scroll)
                .map(|s| {
                    let icon = s.category.icon();
                    let cat = s.category.as_str();
                    let score_pct = (s.score * 100.0) as u32;
                    let line = Line::from(vec![
                        Span::styled(format!("{} ", icon), Style::default().fg(p.focus)),
                        Span::styled(format!("{:<14} ", cat), Style::default().fg(p.muted)),
                        Span::styled(
                            s.pattern.clone(),
                            Style::default().fg(p.fg).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  ({} sessions, score {}%)", s.session_count, score_pct),
                            Style::default().fg(p.muted),
                        ),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(
                            " Suggestions ({}) — [r] re-run  [j/k] scroll ",
                            suggestion_count
                        ))
                        .style(Style::default().fg(p.border).bg(p.surface)),
                )
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            frame.render_widget(list, chunks[1]);
        }
    }
}
