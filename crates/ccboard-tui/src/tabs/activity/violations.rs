//! Violations feed — consolidated cross-session alert view with remediation hints.
//!
//! Pulls from DataStore::all_violations() (DashMap + SQLite merge, sorted Critical→Info).
//! Renders a scrollable list with severity badge, category, detail, session context,
//! and a per-category action hint.

use crate::theme::Palette;
use ccboard_core::models::activity::{Alert, AlertSeverity};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Violations sub-view state
pub struct ViolationsView {
    pub list_state: ListState,
    /// Cached + sorted alert list — refreshed on DataEvent::AnalyticsUpdated
    pub cache: Vec<Alert>,
    /// True once the initial load has been triggered
    pub loaded: bool,
}

impl Default for ViolationsView {
    fn default() -> Self {
        Self::new()
    }
}

impl ViolationsView {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            cache: Vec::new(),
            loaded: false,
        }
    }

    /// Navigate down in the violations list
    pub fn next(&mut self) {
        if self.cache.is_empty() {
            return;
        }
        let i = self.list_state.selected().unwrap_or(0);
        if i + 1 < self.cache.len() {
            self.list_state.select(Some(i + 1));
        }
    }

    /// Navigate up in the violations list
    pub fn prev(&mut self) {
        let i = self.list_state.selected().unwrap_or(0);
        if i > 0 {
            self.list_state.select(Some(i - 1));
        }
    }

    /// Render the full violations feed
    pub fn render(&mut self, frame: &mut Frame, area: Rect, scanning_count: usize, p: &Palette) {
        // Stats bar (1 line) + list below
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .split(area);

        self.render_stats(frame, rows[0], scanning_count, p);
        self.render_list(frame, rows[1], p);
    }

    fn render_stats(&self, frame: &mut Frame, area: Rect, scanning_count: usize, p: &Palette) {
        let critical = self
            .cache
            .iter()
            .filter(|a| a.severity == AlertSeverity::Critical)
            .count();
        let warning = self
            .cache
            .iter()
            .filter(|a| a.severity == AlertSeverity::Warning)
            .count();
        let info = self
            .cache
            .iter()
            .filter(|a| a.severity == AlertSeverity::Info)
            .count();

        let mut spans = vec![
            Span::styled("🔴 Critical: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{}  ", critical),
                Style::default().fg(p.error).add_modifier(Modifier::BOLD),
            ),
            Span::styled("🟡 Warning: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{}  ", warning),
                Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
            ),
            Span::styled("🔵 Info: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("{}  ", info),
                Style::default().fg(p.focus).add_modifier(Modifier::BOLD),
            ),
        ];

        if scanning_count > 0 {
            spans.push(Span::styled(
                format!("  ⟳ Scanning {} sessions…", scanning_count),
                Style::default().fg(p.focus),
            ));
        } else if self.cache.is_empty() && self.loaded {
            spans.push(Span::styled(
                "  ✓ No violations found",
                Style::default().fg(p.success),
            ));
        } else if !self.loaded {
            spans.push(Span::styled(
                "  Press r to scan all sessions",
                Style::default().fg(p.muted),
            ));
        }

        let block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(p.muted));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(Paragraph::new(Line::from(spans)), inner);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        if self.cache.is_empty() {
            let msg = if self.loaded {
                "✓ No violations across analyzed sessions"
            } else {
                "No sessions analyzed yet — press r to scan all, or switch to Sessions tab and press a"
            };
            frame.render_widget(
                Paragraph::new(Span::styled(msg, Style::default().fg(p.muted))),
                area,
            );
            return;
        }

        let items: Vec<ListItem> = self
            .cache
            .iter()
            .map(|alert| {
                let (icon, sev_style) = match alert.severity {
                    AlertSeverity::Critical => ("🔴", Style::default().fg(p.error)),
                    AlertSeverity::Warning => ("🟡", Style::default().fg(p.warning)),
                    AlertSeverity::Info => ("🔵", Style::default().fg(p.focus)),
                };
                let cat = format!("{:?}", alert.category);

                // Truncate detail — chars() to avoid UTF-8 byte-slice panic
                let detail = if alert.detail.chars().count() > 55 {
                    let t: String = alert.detail.chars().take(52).collect();
                    format!("{}…", t)
                } else {
                    alert.detail.clone()
                };

                let short_session: String = alert.session_id.chars().take(8).collect();
                let hint = alert.category.action_hint();

                ListItem::new(vec![
                    // Line 1: severity icon + category + detail
                    Line::from(vec![
                        Span::raw(format!("{} ", icon)),
                        Span::styled(
                            format!("[{}] ", cat),
                            sev_style.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(detail, Style::default().fg(p.fg)),
                    ]),
                    // Line 2: session context
                    Line::from(vec![
                        Span::styled("   session: ", Style::default().fg(p.muted)),
                        Span::styled(short_session, Style::default().fg(p.focus)),
                        Span::styled(
                            format!(" · {}", alert.timestamp.format("%d %b %H:%M")),
                            Style::default().fg(p.muted),
                        ),
                    ]),
                    // Line 3: action hint
                    Line::from(vec![
                        Span::styled("   → ", Style::default().fg(p.warning)),
                        Span::styled(hint, Style::default().fg(p.muted)),
                    ]),
                    // Separator
                    Line::from(""),
                ])
            })
            .collect();

        let title = format!(" Violations ({}) ", self.cache.len());
        frame.render_stateful_widget(
            List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(p.muted))
                        .title(Span::styled(
                            title,
                            Style::default().fg(p.error).add_modifier(Modifier::BOLD),
                        )),
                )
                .highlight_style(Style::default().bg(p.focus).fg(p.bg))
                .highlight_symbol("▶ "),
            area,
            &mut self.list_state,
        );
    }
}
