//! Dashboard tab - Overview with sparkline, stats, model gauges, activity

use crate::theme::ContextSaturationColor;
use ccboard_core::models::StatsCache;
use ccboard_core::parsers::McpConfig;
use ccboard_core::store::DataStore;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
};
use std::sync::Arc;

/// Dashboard tab state
pub struct DashboardTab;

impl DashboardTab {
    pub fn new() -> Self {
        Self
    }

    /// Render the dashboard
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: Option<&StatsCache>,
        mcp_config: Option<&McpConfig>,
        store: Option<&Arc<DataStore>>,
    ) {
        // Check if we should show cache hint
        let show_hint = stats
            .map(|s| s.total_tokens() == 0 && s.session_count() > 0)
            .unwrap_or(false);

        // Main vertical layout
        let mut constraints = vec![
            Constraint::Length(7), // Stats cards row
            Constraint::Length(9), // Sparkline
        ];

        if show_hint {
            constraints.push(Constraint::Length(3)); // Cache hint
        }

        constraints.push(Constraint::Min(12)); // Model gauges

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(area);

        // Stats cards (6 columns now)
        self.render_stats_row(frame, chunks[0], stats, mcp_config, store);

        // Activity sparkline
        self.render_activity(frame, chunks[1], stats);

        // Cache hint (if needed)
        let hint_idx = if show_hint {
            self.render_cache_hint(frame, chunks[2]);
            3
        } else {
            2
        };

        // Model distribution as gauges
        self.render_model_gauges(frame, chunks[hint_idx], stats);
    }

    fn render_stats_row(
        &self,
        frame: &mut Frame,
        area: Rect,
        stats: Option<&StatsCache>,
        mcp_config: Option<&McpConfig>,
        store: Option<&Arc<DataStore>>,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(17), // Tokens (adjusted from 20%)
                Constraint::Percentage(17), // Sessions
                Constraint::Percentage(17), // Messages
                Constraint::Percentage(16), // Cache
                Constraint::Percentage(16), // MCP
                Constraint::Percentage(17), // Context (new)
            ])
            .split(area);

        let (tokens, sessions, messages, cache) = stats
            .map(|s| {
                (
                    Self::format_number(s.total_tokens()),
                    Self::format_number(s.session_count()),
                    Self::format_number(s.message_count()),
                    format!("{:.1}%", s.cache_ratio() * 100.0),
                )
            })
            .unwrap_or_else(|| ("—".into(), "—".into(), "—".into(), "—".into()));

        // MCP servers count
        let mcp_count = mcp_config.map(|config| config.servers.len()).unwrap_or(0);
        let mcp_color = if mcp_count > 0 {
            Color::Green
        } else {
            Color::DarkGray
        };

        // Context window saturation
        let (context_display, context_color) = store
            .map(|s| {
                let ctx_stats = s.context_window_stats();
                let pct = ctx_stats.avg_saturation_pct;
                let color_theme = ContextSaturationColor::from_percentage(pct);
                let icon = color_theme.icon();

                // Format: "68.5%  ⚠️ 3" or "45.2%" (no icon/count if safe)
                let display = if ctx_stats.high_load_count > 0 {
                    format!("{:.1}% {} {}", pct, icon, ctx_stats.high_load_count)
                } else {
                    format!("{:.1}%", pct)
                };

                (display, color_theme.to_color())
            })
            .unwrap_or_else(|| ("—".into(), Color::DarkGray));

        self.render_stat_card(frame, chunks[0], "◆ Tokens", &tokens, Color::Cyan, "total");
        self.render_stat_card(
            frame,
            chunks[1],
            "● Sessions",
            &sessions,
            Color::Green,
            "tracked",
        );
        self.render_stat_card(
            frame,
            chunks[2],
            "▶ Messages",
            &messages,
            Color::Yellow,
            "sent",
        );
        self.render_stat_card(
            frame,
            chunks[3],
            "% Cache Hit",
            &cache,
            Color::Magenta,
            "ratio",
        );
        self.render_stat_card(
            frame,
            chunks[4],
            "◉ MCP",
            &mcp_count.to_string(),
            mcp_color,
            "servers",
        );
        self.render_stat_card(
            frame,
            chunks[5],
            "◐ Context",
            &context_display,
            context_color,
            "avg 30d",
        );
    }

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
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(color).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner area for value and subtitle
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // top padding
                Constraint::Length(2), // value
                Constraint::Length(1), // subtitle
                Constraint::Min(0),    // bottom padding
            ])
            .split(inner);

        // Main value - large and centered
        let value_widget = Paragraph::new(Line::from(Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(value_widget, inner_chunks[1]);

        // Subtitle
        let subtitle_widget = Paragraph::new(Line::from(Span::styled(
            subtitle,
            Style::default().fg(Color::DarkGray),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(subtitle_widget, inner_chunks[2]);
    }

    fn render_activity(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " ≡ 7-Day Activity ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Get activity data
        let (data, labels): (Vec<u64>, Vec<String>) = stats
            .map(|s| {
                let recent = s.recent_daily(7);
                let data: Vec<u64> = recent.iter().map(|d| d.message_count).collect();
                let labels: Vec<String> = recent
                    .iter()
                    .map(|d| {
                        // Extract day from date (YYYY-MM-DD -> DD)
                        d.date.split('-').next_back().unwrap_or("").to_string()
                    })
                    .collect();
                (data, labels)
            })
            .unwrap_or_else(|| (vec![0; 7], vec!["—".to_string(); 7]));

        // Pad data to 7 days if needed
        let mut padded_data = vec![0u64; 7];
        let mut padded_labels = vec!["—".to_string(); 7];
        let start = 7usize.saturating_sub(data.len());
        for (i, (&val, label)) in data.iter().zip(labels.iter()).enumerate() {
            if start + i < 7 {
                padded_data[start + i] = val;
                padded_labels[start + i] = label.clone();
            }
        }

        // Layout: sparkline + labels
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(3),    // sparkline
                Constraint::Length(1), // labels
            ])
            .split(inner);

        // Sparkline with better style - expand data to fill width
        let max_val = padded_data.iter().max().copied().unwrap_or(1).max(1);
        let width = inner_chunks[0].width as usize;
        let expanded_data = Self::expand_sparkline_data(&padded_data, width);
        let sparkline = Sparkline::default()
            .data(&expanded_data)
            .max(max_val)
            .style(Style::default().fg(Color::Cyan))
            .bar_set(symbols::bar::NINE_LEVELS);
        frame.render_widget(sparkline, inner_chunks[0]);

        // Day labels and values
        let label_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, 7); 7])
            .split(inner_chunks[1]);

        for (i, (label, &val)) in padded_labels.iter().zip(padded_data.iter()).enumerate() {
            let display = if val > 0 {
                format!("{} ({})", label, Self::format_short(val))
            } else {
                label.clone()
            };
            let label_widget = Paragraph::new(Line::from(Span::styled(
                display,
                Style::default().fg(Color::DarkGray),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(label_widget, label_chunks[i]);
        }
    }

    fn render_model_gauges(&self, frame: &mut Frame, area: Rect, stats: Option<&StatsCache>) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " ◈ Model Usage ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let models = stats.map(|s| s.top_models(5)).unwrap_or_default();

        if models.is_empty() {
            let no_data = Paragraph::new("No model data available")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(no_data, inner);
            return;
        }

        // Calculate total for percentages
        let total: u64 = models.iter().map(|(_, u)| u.total_tokens()).sum();
        let total = total.max(1);

        // Create gauge for each model
        let constraints: Vec<Constraint> = models.iter().map(|_| Constraint::Length(2)).collect();

        let gauge_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints)
            .split(inner);

        let colors = [
            Color::Magenta,
            Color::Cyan,
            Color::Green,
            Color::Yellow,
            Color::Blue,
        ];

        for (i, ((name, usage), chunk)) in models.iter().zip(gauge_chunks.iter()).enumerate() {
            let tokens = usage.total_tokens();
            let pct = (tokens as f64 / total as f64 * 100.0) as u16;
            let color = colors[i % colors.len()];

            // Model name formatting
            let display_name = Self::format_model_name(name);
            let token_str = Self::format_number(tokens);

            let gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(color).bg(Color::DarkGray))
                .percent(pct.min(100))
                .label(Span::styled(
                    format!(
                        "{:<20} {:>10}  ({:>5.1}%)",
                        display_name, token_str, pct as f64
                    ),
                    Style::default().fg(Color::White).bold(),
                ));

            frame.render_widget(gauge, *chunk);
        }
    }

    fn format_number(n: u64) -> String {
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

    fn format_short(n: u64) -> String {
        if n >= 1_000_000 {
            format!("{}M", n / 1_000_000)
        } else if n >= 1_000 {
            format!("{}K", n / 1_000)
        } else {
            n.to_string()
        }
    }

    /// Expand sparkline data to fill available width
    /// Each original value is repeated to proportionally fill the target width
    fn expand_sparkline_data(data: &[u64], target_width: usize) -> Vec<u64> {
        if data.is_empty() || target_width == 0 {
            return vec![0; target_width.max(1)];
        }

        let data_len = data.len();
        let repeat_factor = target_width / data_len;
        let remainder = target_width % data_len;

        let mut expanded = Vec::with_capacity(target_width);
        for (i, &val) in data.iter().enumerate() {
            // Add one extra repeat for the first 'remainder' items to fill exactly
            let repeats = repeat_factor + if i < remainder { 1 } else { 0 };
            for _ in 0..repeats {
                expanded.push(val);
            }
        }

        expanded
    }

    fn render_cache_hint(&self, frame: &mut Frame, area: Rect) {
        let hint_text = vec![
            Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    "Stats look wrong? Run ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    "ccboard clear-cache",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" to rebuild metadata.", Style::default().fg(Color::Yellow)),
            ]),
        ];

        let hint = Paragraph::new(hint_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .style(Style::default().bg(Color::Black)),
            );

        frame.render_widget(hint, area);
    }

    fn format_model_name(name: &str) -> String {
        // Remove "claude-" prefix and date suffix
        let name = name.strip_prefix("claude-").unwrap_or(name);

        // Handle patterns like "opus-4-5-20251101" -> "Opus 4.5"
        // or "sonnet-4-5-20250929" -> "Sonnet 4.5"
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

        // Try to extract version
        if parts.len() >= 3 {
            // Check if parts[1] and parts[2] are numeric (version like 4-5)
            let p1_numeric = parts[1].chars().all(|c| c.is_ascii_digit());
            let p2_numeric = parts[2].chars().all(|c| c.is_ascii_digit());

            if p1_numeric && p2_numeric {
                return format!("{} {}.{}", capitalized, parts[1], parts[2]);
            } else if p1_numeric {
                return format!("{} {}", capitalized, parts[1]);
            }
        } else if parts.len() >= 2 && parts[1].chars().all(|c| c.is_ascii_digit()) {
            return format!("{} {}", capitalized, parts[1]);
        }

        // For non-claude models, just capitalize first letter and truncate
        if name.len() > 18 {
            format!("{}…", &name[..17])
        } else {
            capitalized
        }
    }
}

impl Default for DashboardTab {
    fn default() -> Self {
        Self::new()
    }
}
