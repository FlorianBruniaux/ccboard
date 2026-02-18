//! Plugins tab - Plugin usage analytics interface
//!
//! Features:
//! - Three-column layout (Top Usage | Top Cost | Dead Code)
//! - Plugin classification (Skill, MCP, Agent, Command, Native)
//! - Dead code detection
//! - Sort modes (usage, cost, name)
//!
//! Keybindings:
//! - Tab: Cycle between columns
//! - j/k or Up/Down: Navigate within column
//! - s: Toggle sort mode (usage → cost → name)
//! - r: Refresh analytics

use crate::empty_state::EmptyState;
use crate::theme::Palette;
use ccboard_core::analytics::{aggregate_plugin_usage, PluginAnalytics};
use ccboard_core::models::config::ColorScheme;
use ccboard_core::DataStore;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::sync::Arc;

/// Which column has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    TopUsage,
    TopCost,
    DeadCode,
}

/// Sort mode for plugin list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortMode {
    Usage,
    Cost,
    Name,
}

impl SortMode {
    fn label(&self) -> &'static str {
        match self {
            SortMode::Usage => "Usage",
            SortMode::Cost => "Cost",
            SortMode::Name => "Name",
        }
    }

    fn next(&self) -> Self {
        match self {
            SortMode::Usage => SortMode::Cost,
            SortMode::Cost => SortMode::Name,
            SortMode::Name => SortMode::Usage,
        }
    }
}

/// Plugins Tab state
pub struct PluginsTab {
    /// Which column has focus
    focus: Focus,
    /// Current sort mode
    sort_mode: SortMode,
    /// Top usage list state
    top_usage_state: ListState,
    /// Top cost list state
    top_cost_state: ListState,
    /// Dead code list state
    dead_code_state: ListState,
    /// Cached analytics (recomputed on refresh)
    analytics: Option<PluginAnalytics>,
}

impl Default for PluginsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginsTab {
    /// Create a new Plugins tab
    pub fn new() -> Self {
        let mut top_usage_state = ListState::default();
        top_usage_state.select(Some(0));

        Self {
            focus: Focus::TopUsage,
            sort_mode: SortMode::Usage,
            top_usage_state,
            top_cost_state: ListState::default(),
            dead_code_state: ListState::default(),
            analytics: None,
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyCode, store: &Arc<DataStore>) -> bool {
        match key {
            KeyCode::Tab => {
                // Cycle focus between columns
                self.focus = match self.focus {
                    Focus::TopUsage => Focus::TopCost,
                    Focus::TopCost => Focus::DeadCode,
                    Focus::DeadCode => Focus::TopUsage,
                };
                true
            }
            KeyCode::Char('s') => {
                // Toggle sort mode
                self.sort_mode = self.sort_mode.next();
                self.refresh_analytics(store);
                true
            }
            KeyCode::Char('r') => {
                // Refresh analytics
                self.refresh_analytics(store);
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.navigate_up();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.navigate_down();
                true
            }
            _ => false,
        }
    }

    /// Navigate up within current column
    fn navigate_up(&mut self) {
        let state = self.active_list_state_mut();
        if let Some(selected) = state.selected() {
            if selected > 0 {
                state.select(Some(selected - 1));
            }
        }
    }

    /// Navigate down within current column
    fn navigate_down(&mut self) {
        // Calculate length first without holding borrow
        let len = if let Some(analytics) = &self.analytics {
            match self.focus {
                Focus::TopUsage => analytics.top_by_usage.len(),
                Focus::TopCost => analytics.top_by_cost.len(),
                Focus::DeadCode => analytics.dead_plugins.len(),
            }
        } else {
            0
        };

        // Now get mutable state
        let state = self.active_list_state_mut();
        if let Some(selected) = state.selected() {
            if selected < len.saturating_sub(1) {
                state.select(Some(selected + 1));
            }
        }
    }

    /// Get mutable reference to active list state
    fn active_list_state_mut(&mut self) -> &mut ListState {
        match self.focus {
            Focus::TopUsage => &mut self.top_usage_state,
            Focus::TopCost => &mut self.top_cost_state,
            Focus::DeadCode => &mut self.dead_code_state,
        }
    }

    /// Refresh analytics from store
    fn refresh_analytics(&mut self, store: &Arc<DataStore>) {
        // Get all sessions (limit to recent 10000 for performance)
        let sessions = store.recent_sessions(10000);

        // Extract skill and command names from store
        // TODO: Implement proper skill/command extraction from .claude/skills and .claude/commands
        let skills = vec![]; // Placeholder
        let commands = vec![]; // Placeholder

        // Compute analytics
        self.analytics = Some(aggregate_plugin_usage(&sessions, &skills, &commands));

        // Reset selection to first item
        self.top_usage_state.select(Some(0));
        self.top_cost_state.select(Some(0));
        self.dead_code_state.select(Some(0));
    }

    /// Render the plugins tab
    pub fn render(&mut self, frame: &mut Frame, area: Rect, store: &Arc<DataStore>, scheme: ColorScheme) {
        let p = Palette::new(scheme);

        // Lazy load analytics on first render
        if self.analytics.is_none() {
            self.refresh_analytics(store);
        }

        // Check if analytics available (without holding borrow)
        if self.analytics.is_none() {
            let empty = EmptyState::new("No Plugin Data")
                .message("No plugin usage data available")
                .build();
            frame.render_widget(empty, area);
            return;
        }

        // Split layout: header + content
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Stats header
                Constraint::Min(0),    // Content
            ])
            .split(area);

        // Render stats header
        self.render_header(frame, chunks[0]);

        // Render three-column layout
        self.render_columns(frame, chunks[1], &p);
    }

    /// Render stats header
    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let analytics = self.analytics.as_ref().unwrap();
        let active_pct = if analytics.total_plugins > 0 {
            (analytics.active_plugins as f64 / analytics.total_plugins as f64) * 100.0
        } else {
            0.0
        };

        let stats_text = format!(
            "Total: {} | Active: {} ({:.0}%) | Dead Code: {} | Sort: [s] {}",
            analytics.total_plugins,
            analytics.active_plugins,
            active_pct,
            analytics.dead_plugins.len(),
            self.sort_mode.label()
        );

        let stats = Paragraph::new(stats_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("🎁 Plugin Analytics"),
            )
            .alignment(Alignment::Center);

        frame.render_widget(stats, area);
    }

    /// Render three-column layout
    fn render_columns(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        self.render_top_usage(frame, chunks[0], p);
        self.render_top_cost(frame, chunks[1], p);
        self.render_dead_code(frame, chunks[2], p);
    }

    /// Render top usage column
    fn render_top_usage(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let analytics = self.analytics.as_ref().unwrap();
        let focused = self.focus == Focus::TopUsage;

        let items: Vec<ListItem> = analytics
            .top_by_usage
            .iter()
            .enumerate()
            .map(|(i, plugin)| {
                let icon = plugin.plugin_type.icon();
                let text = format!(
                    "{}. {} {} ({} uses)",
                    i + 1,
                    icon,
                    plugin.name,
                    plugin.total_invocations
                );
                ListItem::new(text)
            })
            .collect();

        let title = if focused {
            "Top 10 Most Used [FOCUSED]"
        } else {
            "Top 10 Most Used"
        };

        let border_style = if focused {
            Style::default().fg(p.focus)
        } else {
            Style::default().fg(p.border)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(p.focus),
            );

        frame.render_stateful_widget(list, area, &mut self.top_usage_state);
    }

    /// Render top cost column
    fn render_top_cost(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let analytics = self.analytics.as_ref().unwrap();
        let focused = self.focus == Focus::TopCost;

        let items: Vec<ListItem> = analytics
            .top_by_cost
            .iter()
            .enumerate()
            .map(|(i, plugin)| {
                let text = format!("{}. {} (${:.2})", i + 1, plugin.name, plugin.total_cost);
                ListItem::new(text)
            })
            .collect();

        let title = if focused {
            "Top 10 By Cost [FOCUSED]"
        } else {
            "Top 10 By Cost"
        };

        let border_style = if focused {
            Style::default().fg(p.focus)
        } else {
            Style::default().fg(p.border)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(p.focus),
            );

        frame.render_stateful_widget(list, area, &mut self.top_cost_state);
    }

    /// Render dead code column
    fn render_dead_code(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let analytics = self.analytics.as_ref().unwrap();
        let focused = self.focus == Focus::DeadCode;

        if analytics.dead_plugins.is_empty() {
            let empty = Paragraph::new("No dead code detected!\nAll plugins are being used.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Dead Code (Never Used)"),
                )
                .alignment(Alignment::Center);
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = analytics
            .dead_plugins
            .iter()
            .map(|name| {
                let text = format!("• {} (0 uses)", name);
                ListItem::new(text).style(Style::default().fg(p.muted))
            })
            .collect();

        let title = if focused {
            "Dead Code (Never Used) [FOCUSED]"
        } else {
            "Dead Code (Never Used)"
        };

        let border_style = if focused {
            Style::default().fg(p.focus)
        } else {
            Style::default().fg(p.border)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(border_style),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(p.error));

        frame.render_stateful_widget(list, area, &mut self.dead_code_state);
    }
}
