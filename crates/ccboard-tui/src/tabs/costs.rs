//! Costs tab - Token usage and estimated costs by model

use ccboard_core::models::StatsCache;
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

/// Costs tab state
pub struct CostsTab {
    /// Selected model index
    model_state: ListState,
    /// View mode (0=Overview, 1=By Model, 2=Daily)
    view_mode: usize,
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
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.view_mode = (self.view_mode + 1) % 3;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.view_mode = (self.view_mode + 2) % 3; // +2 instead of -1 for wrapping
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let current = self.model_state.selected().unwrap_or(0);
                self.model_state.select(Some(current.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let current = self.model_state.selected().unwrap_or(0);
                self.model_state.select(Some(current + 1));
            }
            _ => {}
        }
    }

    /// Render the costs tab
    pub fn render(&mut self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
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
            _ => {}
        }
    }

    fn render_view_tabs(&self, frame: &mut Frame, area: Rect) {
        let tabs = ["1. Overview", "2. By Model", "3. Daily Trend"];

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 3); 3])
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

        model_costs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Cost by Model ",
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

        model_data.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
}
