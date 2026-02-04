//! Analytics tab - Trends, forecasting, patterns, insights with 4 sub-views

use ccboard_core::analytics::{AnalyticsData, Period};
use ccboard_core::store::DataStore;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph,
        Sparkline,
    },
};
use std::sync::Arc;

/// Sub-view selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyticsView {
    Overview,
    Trends,
    Patterns,
    Insights,
}

impl AnalyticsView {
    /// Cycle to next view
    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Trends,
            Self::Trends => Self::Patterns,
            Self::Patterns => Self::Insights,
            Self::Insights => Self::Overview,
        }
    }

    /// Cycle to previous view
    pub fn prev(self) -> Self {
        match self {
            Self::Overview => Self::Insights,
            Self::Trends => Self::Overview,
            Self::Patterns => Self::Trends,
            Self::Insights => Self::Patterns,
        }
    }

    /// Display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Trends => "Trends",
            Self::Patterns => "Patterns",
            Self::Insights => "Insights",
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
    }

    /// Cycle to previous view (Shift+Tab key)
    pub fn prev_view(&mut self) {
        self.current_view = self.current_view.prev();
        self.scroll_offset = 0;
    }

    /// Scroll down (j key)
    pub fn scroll_down(&mut self, max_items: usize) {
        if self.scroll_offset + 10 < max_items {
            self.scroll_offset += 1;
        }
    }

    /// Scroll up (k key)
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Render the analytics tab
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        analytics: Option<&AnalyticsData>,
        store: Option<&Arc<DataStore>>,
    ) {
        use tracing::debug;

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

        self.render_header(frame, chunks[0], analytics, store);

        match analytics {
            Some(data) => {
                debug!(
                    insights_count = data.insights.len(),
                    "Rendering analytics data"
                );
                match self.current_view {
                    AnalyticsView::Overview => self.render_overview(frame, chunks[1], data),
                    AnalyticsView::Trends => self.render_trends(frame, chunks[1], data),
                    AnalyticsView::Patterns => self.render_patterns(frame, chunks[1], data),
                    AnalyticsView::Insights => self.render_insights(frame, chunks[1], data),
                }
            }
            None => {
                debug!("No analytics data available, showing loading");
                self.render_loading(frame, chunks[1])
            }
        }
    }

    /// Render header with period selector and view tabs
    fn render_header(
        &self,
        frame: &mut Frame,
        area: Rect,
        _analytics: Option<&AnalyticsData>,
        store: Option<&Arc<DataStore>>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Period selector (left)
        let session_count = store.map(|s| s.session_count()).unwrap_or(0);
        let period_display = self.current_period.display(session_count);
        let period_text = vec![
            Span::raw("Period: "),
            Span::styled(
                period_display,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "[F1:7d F2:30d F3:90d F4:All]",
                Style::default().fg(Color::DarkGray),
            ),
        ];
        let period_para = Paragraph::new(Line::from(period_text))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Left);
        frame.render_widget(period_para, chunks[0]);

        // View tabs (right)
        let views = [
            AnalyticsView::Overview,
            AnalyticsView::Trends,
            AnalyticsView::Patterns,
            AnalyticsView::Insights,
        ];
        let tabs_text: Vec<Span> = views
            .iter()
            .flat_map(|view| {
                let is_active = *view == self.current_view;
                let style = if is_active {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                vec![Span::styled(view.name(), style), Span::raw(" | ")]
            })
            .collect();

        let tabs_para = Paragraph::new(Line::from(tabs_text))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(tabs_para, chunks[1]);
    }

    /// Render loading state
    fn render_loading(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Analytics")
            .style(Style::default().fg(Color::Gray));

        let para = Paragraph::new(vec![
            Line::from(""),
            Line::from("Computing analytics..."),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'r' to compute or wait for auto-computation",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);

        frame.render_widget(para, area);
    }

    /// Render overview sub-view
    fn render_overview(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7), // Summary cards
                Constraint::Length(9), // Token sparkline
                Constraint::Min(8),    // Top insights
            ])
            .split(area);

        // Summary cards
        self.render_summary_cards(frame, chunks[0], data);

        // Token sparkline
        self.render_token_sparkline(frame, chunks[1], data);

        // Top insights preview
        self.render_insights_preview(frame, chunks[2], data);
    }

    /// Render summary cards
    fn render_summary_cards(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
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
            Color::Cyan,
            "total",
        );

        // Total sessions
        let total_sessions: usize = data.trends.daily_sessions.iter().sum();
        self.render_stat_card(
            frame,
            chunks[1],
            "● Sessions",
            &total_sessions.to_string(),
            Color::Green,
            "count",
        );

        // Monthly cost estimate
        let cost_display = format!("${:.2}", data.forecast.monthly_cost_estimate);
        self.render_stat_card(
            frame,
            chunks[2],
            "$ Cost Est",
            &cost_display,
            Color::Yellow,
            "monthly",
        );

        // Forecast confidence
        let confidence_display = format!("{:.0}%", data.forecast.confidence * 100.0);
        let confidence_color = if data.forecast.confidence > 0.7 {
            Color::Green
        } else if data.forecast.confidence > 0.4 {
            Color::Yellow
        } else {
            Color::Red
        };
        self.render_stat_card(
            frame,
            chunks[3],
            "◉ Confidence",
            &confidence_display,
            confidence_color,
            "forecast",
        );
    }

    /// Render stat card (reused from dashboard)
    fn render_stat_card(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        value: &str,
        color: Color,
        subtitle: &str,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .title(Span::styled(
                title,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                value,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(subtitle, Style::default().fg(Color::DarkGray))),
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
    fn render_token_sparkline(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
        let sparkline_data: Vec<u64> = data.trends.daily_tokens.to_vec();
        let max_val = sparkline_data.iter().max().copied().unwrap_or(1);

        // Outer block with title and borders
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Token Usage Over Time");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Layout: [Y-axis labels (8 chars), Sparkline]
        let chart_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(8),  // Y-axis labels
                Constraint::Min(20),    // Chart
            ])
            .split(inner);

        // Y-axis labels (3 ticks: max, mid, 0)
        let max_label = Self::format_short(max_val);
        let mid_label = Self::format_short(max_val / 2);

        // Calculate vertical spacing to align with sparkline height
        let available_height = chart_layout[0].height as usize;
        let spacing = if available_height >= 3 {
            (available_height - 1) / 2  // Distribute remaining space
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
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(y_axis_widget, chart_layout[0]);

        // Sparkline in remaining area
        let sparkline = Sparkline::default()
            .data(&sparkline_data)
            .style(Style::default().fg(Color::Cyan))
            .max(max_val);

        frame.render_widget(sparkline, chart_layout[1]);
    }

    /// Render insights preview (top 3)
    fn render_insights_preview(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
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
                    .title("Top Insights (Tab to Insights view for all)"),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);
    }

    /// Render trends sub-view (time series charts)
    fn render_trends(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
        let block = Block::default()
            .borders(Borders::ALL)
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

        // Forecast line (from last historical point to 30 days ahead)
        let forecast_data = if data.forecast.unavailable_reason.is_none() && !token_data.is_empty() {
            let last_day = token_data.len() as f64 - 1.0;
            let last_tokens = token_data.last().map(|p| p.1).unwrap_or(0.0);
            let forecast_end_tokens = data.forecast.next_30_days_tokens as f64;

            // Create intermediate points for smoother line
            let mut points = vec![(last_day, last_tokens)];
            for i in 1..=30 {
                let x = last_day + i as f64;
                let progress = i as f64 / 30.0;
                let y = last_tokens + (forecast_end_tokens - last_tokens) * progress;
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
                .name("Tokens")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&token_data),
            Dataset::default()
                .name("Sessions (x100)")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&session_data),
        ];

        // Add forecast dataset if available
        if !forecast_data.is_empty() {
            let forecast_color = if data.forecast.confidence > 0.7 {
                Color::LightGreen
            } else if data.forecast.confidence > 0.4 {
                Color::Yellow
            } else {
                Color::LightRed
            };

            datasets.push(
                Dataset::default()
                    .name(format!("Forecast ({:.0}% conf)", data.forecast.confidence * 100.0))
                    .marker(symbols::Marker::Dot)
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
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
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
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds([0.0, x_max]),
            )
            .y_axis(
                Axis::default()
                    .title("Tokens")
                    .style(Style::default().fg(Color::Gray))
                    .labels(y_labels)
                    .bounds([0.0, max_with_forecast * 1.1]),
            );

        frame.render_widget(chart, area);
    }

    /// Render patterns sub-view (bar charts)
    fn render_patterns(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Hourly distribution
        self.render_hourly_distribution(frame, chunks[0], data);

        // Model distribution
        self.render_model_distribution(frame, chunks[1], data);
    }

    /// Render hourly distribution bar chart
    fn render_hourly_distribution(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
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
                    .title("Hourly Distribution (Session Count)"),
            )
            .data(&bar_data)
            .bar_width(9)
            .bar_gap(2)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(Style::default().fg(Color::White).bg(Color::Cyan));

        frame.render_widget(barchart, area);
    }

    /// Render model distribution
    fn render_model_distribution(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
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
                    .title("Model Distribution (Token %)"),
            )
            .data(&model_data)
            .bar_width(12)
            .bar_gap(2)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().fg(Color::White).bg(Color::Green));

        frame.render_widget(barchart, area);
    }

    /// Render insights sub-view (scrollable list)
    fn render_insights(&self, frame: &mut Frame, area: Rect, data: &AnalyticsData) {
        let insights: Vec<ListItem> = data
            .insights
            .iter()
            .skip(self.scroll_offset)
            .map(|insight| {
                let (icon, color) = if insight.contains("Peak hours") {
                    ("⏰", Color::Yellow)
                } else if insight.contains("Opus") {
                    ("💰", Color::Red)
                } else if insight.contains("Cost") || insight.contains("premium") {
                    ("💸", Color::Magenta)
                } else if insight.contains("Weekend") {
                    ("📅", Color::Cyan)
                } else if insight.contains("confidence") {
                    ("⚠️", Color::Yellow)
                } else {
                    ("💡", Color::Green)
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
                    .title(format!("Actionable Insights{}", scroll_indicator)),
            )
            .style(Style::default().fg(Color::White));

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
}
