//! Costs tab - Token usage and estimated costs by model

use ccboard_core::models::{BillingBlockManager, StatsCache};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Row, Table},
};

/// Model pricing (per 1M tokens) - approximate as of 2024
#[derive(Debug, Clone, Copy)]
struct ModelPricing {
    input_per_million: f64,
    output_per_million: f64,
    cache_read_per_million: f64,
}

impl ModelPricing {
    fn opus() -> Self {
        Self {
            input_per_million: 15.0,
            output_per_million: 75.0,
            cache_read_per_million: 1.5,
        }
    }

    fn sonnet() -> Self {
        Self {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: 0.3,
        }
    }

    fn haiku() -> Self {
        Self {
            input_per_million: 0.25,
            output_per_million: 1.25,
            cache_read_per_million: 0.03,
        }
    }

    fn for_model(model: &str) -> Self {
        let model_lower = model.to_lowercase();
        if model_lower.contains("opus") {
            Self::opus()
        } else if model_lower.contains("haiku") {
            Self::haiku()
        } else {
            // Default to sonnet pricing
            Self::sonnet()
        }
    }
}

/// Sort mode for cost data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    CostDesc,
    CostAsc,
    TokensDesc,
    TokensAsc,
    NameAsc,
    NameDesc,
}

impl SortMode {
    fn next(&self) -> Self {
        match self {
            Self::CostDesc => Self::CostAsc,
            Self::CostAsc => Self::TokensDesc,
            Self::TokensDesc => Self::TokensAsc,
            Self::TokensAsc => Self::NameAsc,
            Self::NameAsc => Self::NameDesc,
            Self::NameDesc => Self::CostDesc,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::CostDesc => "Cost ↓",
            Self::CostAsc => "Cost ↑",
            Self::TokensDesc => "Tokens ↓",
            Self::TokensAsc => "Tokens ↑",
            Self::NameAsc => "Name A-Z",
            Self::NameDesc => "Name Z-A",
        }
    }
}

/// Costs tab state
pub struct CostsTab {
    /// Selected model index
    model_state: ListState,
    /// View mode (0=Overview, 1=By Model, 2=Daily)
    view_mode: usize,
    /// Sort mode
    sort_mode: SortMode,
}

impl Default for CostsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl CostsTab {
    pub fn new() -> Self {
        let mut model_state = ListState::default();
        model_state.select(Some(0));

        Self {
            model_state,
            view_mode: 0,
            sort_mode: SortMode::CostDesc,
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.view_mode = (self.view_mode + 1) % 4;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.view_mode = (self.view_mode + 3) % 4; // +3 instead of -1 for wrapping
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let current = self.model_state.selected().unwrap_or(0);
                self.model_state.select(Some(current.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let current = self.model_state.selected().unwrap_or(0);
                self.model_state.select(Some(current + 1));
            }
            KeyCode::Char('s') => {
                // Toggle sort mode
                self.sort_mode = self.sort_mode.next();
            }
            _ => {}
        }
    }

    /// Render the costs tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        stats: Option<&StatsCache>,
        billing_blocks: Option<&BillingBlockManager>,
    ) {
        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // View mode tabs
                Constraint::Min(0),    // Content
            ])
            .split(area);

        // Render view mode selector
        self.render_view_tabs(frame, chunks[0]);

        // Render content based on view mode
        match self.view_mode {
            0 => self.render_overview(frame, chunks[1], stats),
            1 => self.render_by_model(frame, chunks[1], stats),
            2 => self.render_daily(frame, chunks[1], stats),
            3 => self.render_billing_blocks(frame, chunks[1], billing_blocks),
            _ => {}
        }
    }

    fn render_view_tabs(&self, frame: &mut Frame, area: Rect) {
        let tabs = [
            "1. Overview",
            "2. By Model",
            "3. Daily",
            "4. Billing Blocks",
        ];

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 4); 4])
            .split(inner);

        for (i, (tab, chunk)) in tabs.iter().zip(chunks.iter()).enumerate() {
            let style = if i == self.view_mode {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let label = Paragraph::new(Span::styled(*tab, style)).alignment(Alignment::Center);
            frame.render_widget(label, *chunk);
        }
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(7),  // Total cost card
                Constraint::Length(10), // Token breakdown
                Constraint::Min(0),     // Model distribution
            ])
            .split(area);

        // Total cost card
        self.render_total_cost(frame, chunks[0], stats);

        // Token breakdown
        self.render_token_breakdown(frame, chunks[1], stats);

        // Model distribution
        self.render_model_distribution(frame, chunks[2], stats);
    }

    fn render_total_cost(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " $ Total Estimated Cost ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let (total_cost, breakdown) = stats
            .map(|s| {
                let mut total = 0.0;
                let mut details = Vec::new();

                for (model, usage) in &s.model_usage {
                    let pricing = ModelPricing::for_model(model);
                    let input_cost =
                        usage.input_tokens as f64 / 1_000_000.0 * pricing.input_per_million;
                    let output_cost =
                        usage.output_tokens as f64 / 1_000_000.0 * pricing.output_per_million;
                    let cache_cost = usage.cache_read_input_tokens as f64 / 1_000_000.0
                        * pricing.cache_read_per_million;
                    let model_total = input_cost + output_cost + cache_cost;
                    total += model_total;
                    details.push((model.clone(), model_total));
                }

                (total, details)
            })
            .unwrap_or((0.0, Vec::new()));

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(inner);

        // Big cost number
        let cost_display = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("${:.2}", total_cost),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "estimated total",
                Style::default().fg(Color::DarkGray),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(cost_display, chunks[0]);

        // Top models by cost
        let mut sorted = breakdown;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_models: Vec<Line> = sorted
            .iter()
            .take(3)
            .map(|(model, cost)| {
                Line::from(vec![
                    Span::styled(
                        Self::format_model_name(model),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(format!(" ${:.2}", cost), Style::default().fg(Color::Yellow)),
                ])
            })
            .collect();

        let top_list = Paragraph::new(top_models);
        frame.render_widget(top_list, chunks[1]);
    }

    fn render_token_breakdown(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Token Breakdown ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let (input, output, cache_read, cache_write) = stats
            .map(|s| {
                let mut input = 0u64;
                let mut output = 0u64;
                let mut cache_read = 0u64;
                let mut cache_write = 0u64;

                for usage in s.model_usage.values() {
                    input += usage.input_tokens;
                    output += usage.output_tokens;
                    cache_read += usage.cache_read_input_tokens;
                    cache_write += usage.cache_creation_input_tokens;
                }

                (input, output, cache_read, cache_write)
            })
            .unwrap_or((0, 0, 0, 0));

        let total = (input + output).max(1);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
                Constraint::Length(2),
            ])
            .split(inner);

        // Input tokens gauge
        let input_pct = (input as f64 / total as f64 * 100.0) as u16;
        let input_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
            .percent(input_pct.min(100))
            .label(format!(
                "Input: {} ({:.1}%)",
                Self::format_tokens(input),
                input_pct
            ));
        frame.render_widget(input_gauge, chunks[0]);

        // Output tokens gauge
        let output_pct = (output as f64 / total as f64 * 100.0) as u16;
        let output_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Magenta).bg(Color::DarkGray))
            .percent(output_pct.min(100))
            .label(format!(
                "Output: {} ({:.1}%)",
                Self::format_tokens(output),
                output_pct
            ));
        frame.render_widget(output_gauge, chunks[1]);

        // Cache read gauge
        let cache_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .percent(50) // Visual only
            .label(format!("Cache Read: {}", Self::format_tokens(cache_read)));
        frame.render_widget(cache_gauge, chunks[2]);

        // Cache write gauge
        let cache_write_gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray))
            .percent(20) // Visual only
            .label(format!("Cache Write: {}", Self::format_tokens(cache_write)));
        frame.render_widget(cache_write_gauge, chunks[3]);
    }

    fn render_model_distribution(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Model Cost Distribution ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(stats) = stats else {
            let empty =
                Paragraph::new("No data available").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        // Calculate costs per model
        let mut model_costs: Vec<(String, f64, u64, u64)> = stats
            .model_usage
            .iter()
            .map(|(model, usage)| {
                let pricing = ModelPricing::for_model(model);
                let cost = usage.input_tokens as f64 / 1_000_000.0 * pricing.input_per_million
                    + usage.output_tokens as f64 / 1_000_000.0 * pricing.output_per_million
                    + usage.cache_read_input_tokens as f64 / 1_000_000.0
                        * pricing.cache_read_per_million;
                (model.clone(), cost, usage.input_tokens, usage.output_tokens)
            })
            .collect();

        self.sort_models(&mut model_costs);

        let rows: Vec<Row> = model_costs
            .iter()
            .take(10)
            .map(|(model, cost, input, output)| {
                Row::new(vec![
                    Self::format_model_name(model),
                    format!("${:.2}", cost),
                    Self::format_tokens(*input),
                    Self::format_tokens(*output),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ],
        )
        .header(
            Row::new(vec!["Model", "Cost", "Input", "Output"]).style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table, inner);
    }

    fn render_by_model(&mut self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let title_text = format!(" Cost by Model • Sort: {} (press 's') ", self.sort_mode.label());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                title_text,
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(stats) = stats else {
            let empty =
                Paragraph::new("No data available").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        let mut model_data: Vec<(String, f64, f64, f64, f64)> = stats
            .model_usage
            .iter()
            .map(|(model, usage)| {
                let pricing = ModelPricing::for_model(model);
                let input_cost =
                    usage.input_tokens as f64 / 1_000_000.0 * pricing.input_per_million;
                let output_cost =
                    usage.output_tokens as f64 / 1_000_000.0 * pricing.output_per_million;
                let cache_cost = usage.cache_read_input_tokens as f64 / 1_000_000.0
                    * pricing.cache_read_per_million;
                let total = input_cost + output_cost + cache_cost;
                (model.clone(), total, input_cost, output_cost, cache_cost)
            })
            .collect();

        self.sort_models_detailed(&mut model_data);

        // Clamp selection
        if let Some(sel) = self.model_state.selected() {
            if sel >= model_data.len() && !model_data.is_empty() {
                self.model_state.select(Some(model_data.len() - 1));
            }
        }

        let items: Vec<ListItem> = model_data
            .iter()
            .enumerate()
            .map(|(i, (model, total, input, output, cache))| {
                let is_selected = self.model_state.selected() == Some(i);
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("{} ", if is_selected { "▶" } else { " " }),
                            Style::default().fg(Color::Cyan),
                        ),
                        Span::styled(Self::format_model_name(model), style),
                        Span::styled(
                            format!("  ${:.2}", total),
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("    Input: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("${:.2}", input), Style::default().fg(Color::Cyan)),
                        Span::styled("  Output: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("${:.2}", output),
                            Style::default().fg(Color::Magenta),
                        ),
                        Span::styled("  Cache: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("${:.2}", cache), Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(""), // spacing
                ])
            })
            .collect();

        let list = List::new(items);
        frame.render_stateful_widget(list, inner, &mut self.model_state);
    }

    fn render_daily(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Daily Token Usage (Last 14 Days) ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(stats) = stats else {
            let empty =
                Paragraph::new("No data available").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        // Get last 14 days of daily model tokens
        let daily = &stats.daily_model_tokens;
        let recent: Vec<_> = daily.iter().rev().take(14).rev().collect();

        if recent.is_empty() {
            let empty = Paragraph::new("No daily data available")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        }

        // Build bar chart data
        let data: Vec<(&str, u64)> = recent
            .iter()
            .map(|d| {
                let total: u64 = d.tokens_by_model.values().sum();
                let label = d.date.split('-').next_back().unwrap_or("");
                (label, total / 1000) // Show in thousands
            })
            .collect();

        let bar_chart = BarChart::default()
            .data(&data)
            .bar_width(3)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::Cyan))
            .value_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .label_style(Style::default().fg(Color::DarkGray));

        frame.render_widget(bar_chart, inner);
    }

    fn format_tokens(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{:.2}B", n as f64 / 1_000_000_000.0)
        } else if n >= 1_000_000 {
            format!("{:.2}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }

    /// Sort model costs according to current sort mode (simple format: name, cost, input_tokens, output_tokens)
    fn sort_models(&self, models: &mut [(String, f64, u64, u64)]) {
        match self.sort_mode {
            SortMode::CostDesc => {
                models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortMode::CostAsc => {
                models.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortMode::TokensDesc => {
                models.sort_by(|a, b| {
                    let a_total = a.2 + a.3;
                    let b_total = b.2 + b.3;
                    b_total.cmp(&a_total)
                });
            }
            SortMode::TokensAsc => {
                models.sort_by(|a, b| {
                    let a_total = a.2 + a.3;
                    let b_total = b.2 + b.3;
                    a_total.cmp(&b_total)
                });
            }
            SortMode::NameAsc => {
                models.sort_by(|a, b| a.0.cmp(&b.0));
            }
            SortMode::NameDesc => {
                models.sort_by(|a, b| b.0.cmp(&a.0));
            }
        }
    }

    /// Sort model costs with detailed breakdown (name, total, input_cost, output_cost, cache_cost)
    fn sort_models_detailed(&self, models: &mut [(String, f64, f64, f64, f64)]) {
        match self.sort_mode {
            SortMode::CostDesc => {
                models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortMode::CostAsc => {
                models.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            }
            SortMode::TokensDesc | SortMode::TokensAsc => {
                // For detailed view, sort by total cost (already computed)
                models.sort_by(|a, b| {
                    if matches!(self.sort_mode, SortMode::TokensDesc) {
                        b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                    }
                });
            }
            SortMode::NameAsc => {
                models.sort_by(|a, b| a.0.cmp(&b.0));
            }
            SortMode::NameDesc => {
                models.sort_by(|a, b| b.0.cmp(&a.0));
            }
        }
    }

    fn format_model_name(name: &str) -> String {
        let name = name.strip_prefix("claude-").unwrap_or(name);
        let parts: Vec<&str> = name.split('-').collect();

        if parts.is_empty() {
            return name.to_string();
        }

        let model_name = parts[0];
        let capitalized = format!(
            "{}{}",
            model_name.chars().next().unwrap_or(' ').to_uppercase(),
            &model_name[1.min(model_name.len())..]
        );

        if parts.len() >= 3 {
            let p1_numeric = parts[1].chars().all(|c| c.is_ascii_digit());
            let p2_numeric = parts[2].chars().all(|c| c.is_ascii_digit());

            if p1_numeric && p2_numeric {
                return format!("{} {}.{}", capitalized, parts[1], parts[2]);
            }
        }

        capitalized
    }

    fn render_billing_blocks(
        &self,
        frame: &mut Frame,
        area: Rect,
        billing_blocks: Option<&BillingBlockManager>,
    ) {
        let Some(blocks_manager) = billing_blocks else {
            let no_data = Paragraph::new("No billing block data available")
                .block(
                    Block::default()
                        .title("Billing Blocks (5h)")
                        .borders(Borders::ALL),
                )
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(no_data, area);
            return;
        };

        let all_blocks = blocks_manager.get_all_blocks();

        if all_blocks.is_empty() {
            let empty_msg = vec![
                Line::from(""),
                Line::from(Span::styled("📊 No cost data available", Style::default().fg(Color::Yellow))),
                Line::from(""),
                Line::from(Span::styled(
                    "No sessions with timestamps found",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "💡 Costs are calculated from session timestamps:",
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::styled(
                    "   • Sessions must have first_timestamp",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "   • Grouped in 5-hour billing blocks",
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            let no_data = Paragraph::new(empty_msg)
                .block(
                    Block::default()
                        .title("Billing Blocks (5h)")
                        .borders(Borders::ALL),
                )
                .alignment(ratatui::layout::Alignment::Left);
            frame.render_widget(no_data, area);
            return;
        }

        // Group by date
        let mut by_date = std::collections::HashMap::new();
        for (block, usage) in &all_blocks {
            by_date
                .entry(block.date)
                .or_insert_with(Vec::new)
                .push((block, usage));
        }

        // Sort dates
        let mut dates: Vec<_> = by_date.keys().collect();
        dates.sort_by(|a, b| b.cmp(a)); // Most recent first

        // Build rows
        let mut rows = Vec::new();
        for date in dates.iter().take(10) {
            // Show last 10 days
            if let Some(blocks) = by_date.get(date) {
                for (block, usage) in blocks {
                    let color = match BillingBlockManager::get_color_for_cost(usage.total_cost) {
                        "green" => Color::Green,
                        "yellow" => Color::Yellow,
                        "red" => Color::Red,
                        _ => Color::White,
                    };

                    let row_data = vec![
                        date.format("%Y-%m-%d").to_string(),
                        block.label(),
                        format!("{:>8}", usage.total_tokens()),
                        format!("{:>6}", usage.session_count),
                        format!("${:>6.2}", usage.total_cost),
                    ];

                    rows.push(
                        Row::new(row_data)
                            .style(Style::default().fg(color))
                            .height(1),
                    );
                }
            }
        }

        // Header
        let header = Row::new(vec!["Date", "Block (UTC)", "Tokens", "Sessions", "Cost"])
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .height(1);

        // Table
        let table = Table::new(
            rows,
            [
                Constraint::Length(12), // Date
                Constraint::Length(15), // Block
                Constraint::Length(10), // Tokens
                Constraint::Length(10), // Sessions
                Constraint::Length(10), // Cost
            ],
        )
        .header(header)
        .block(
            Block::default()
                .title("Billing Blocks (5h UTC) — Last 10 Days")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .column_spacing(2);

        frame.render_widget(table, area);
    }
}
