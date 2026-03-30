//! Brain tab — cross-session knowledge base from ~/.ccboard/insights.db
//! + optional claude-mem observations from ~/.claude-mem/claude-mem.db
//!
//! Displays insights captured by session-stop.sh hook and /ccboard-remember,
//! and optionally claude-mem observations when integration is enabled.
//!
//! Keybindings:
//! - j/k or ↑/↓: Navigate list
//! - Enter: Expand/collapse detail
//! - d: Archive (soft-delete) selected insight
//! - ←/→ or h/l: Cycle type filter
//! - r: Reload from DB
//! - M: Toggle claude-mem integration (persists to ~/.ccboard/config.toml)
//! - q: Quit / back to parent

use crate::theme::Palette;
use ccboard_core::cache::InsightsDb;
use ccboard_core::models::claude_mem::ClaudeMemObservation;
use ccboard_core::models::config::ColorScheme;
use ccboard_core::models::insight::{Insight, InsightType};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Active type filter in the Brain tab
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FilterType {
    #[default]
    All,
    Progress,
    Decision,
    Blocked,
    Pattern,
    Fix,
    Context,
}

impl FilterType {
    fn label(self) -> &'static str {
        match self {
            FilterType::All => "All",
            FilterType::Progress => "Progress",
            FilterType::Decision => "Decision",
            FilterType::Blocked => "Blocked",
            FilterType::Pattern => "Pattern",
            FilterType::Fix => "Fix",
            FilterType::Context => "Context",
        }
    }

    fn next(self) -> Self {
        match self {
            FilterType::All => FilterType::Progress,
            FilterType::Progress => FilterType::Decision,
            FilterType::Decision => FilterType::Blocked,
            FilterType::Blocked => FilterType::Pattern,
            FilterType::Pattern => FilterType::Fix,
            FilterType::Fix => FilterType::Context,
            FilterType::Context => FilterType::All,
        }
    }

    fn prev(self) -> Self {
        match self {
            FilterType::All => FilterType::Context,
            FilterType::Progress => FilterType::All,
            FilterType::Decision => FilterType::Progress,
            FilterType::Blocked => FilterType::Decision,
            FilterType::Pattern => FilterType::Blocked,
            FilterType::Fix => FilterType::Pattern,
            FilterType::Context => FilterType::Fix,
        }
    }

    fn insight_type(self) -> Option<InsightType> {
        match self {
            FilterType::All => None,
            FilterType::Progress => Some(InsightType::Progress),
            FilterType::Decision => Some(InsightType::Decision),
            FilterType::Blocked => Some(InsightType::Blocked),
            FilterType::Pattern => Some(InsightType::Pattern),
            FilterType::Fix => Some(InsightType::Fix),
            FilterType::Context => Some(InsightType::Context),
        }
    }
}

/// Brain tab state
pub struct BrainTab {
    /// Loaded insights (filtered)
    insights: Vec<Insight>,
    /// List selection state
    list_state: ListState,
    /// Active type filter
    filter: FilterType,
    /// Expanded detail view for selected insight
    expanded: bool,
    /// Path to insights.db cache dir
    db_dir: PathBuf,
    /// Status message
    pub status: Option<String>,
    /// claude-mem observations (empty when integration is disabled)
    claude_mem_obs: Vec<ClaudeMemObservation>,
    /// Whether claude-mem section is currently visible (runtime only)
    show_claude_mem: bool,
}

impl BrainTab {
    pub fn new() -> Self {
        let db_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".ccboard");
        Self {
            insights: Vec::new(),
            list_state: ListState::default(),
            filter: FilterType::All,
            expanded: false,
            db_dir,
            status: None,
            claude_mem_obs: Vec::new(),
            show_claude_mem: false,
        }
    }

    /// (Re)load insights from disk — called on tab activation and `r`
    pub fn reload(&mut self) {
        match InsightsDb::new(&self.db_dir) {
            Ok(db) => {
                let result = match self.filter.insight_type() {
                    None => db.list_all(500),
                    Some(t) => db.list_by_type_all(t, 500),
                };
                match result {
                    Ok(insights) => {
                        self.insights = insights;
                        self.status = Some(format!("{} insights", self.insights.len()));
                        // Reset selection if out of bounds
                        if self.insights.is_empty() {
                            self.list_state.select(None);
                        } else {
                            let sel = self.list_state.selected().unwrap_or(0);
                            self.list_state
                                .select(Some(sel.min(self.insights.len() - 1)));
                        }
                    }
                    Err(e) => {
                        self.status = Some(format!("DB error: {e}"));
                    }
                }
            }
            Err(_) => {
                // insights.db doesn't exist yet — no hook has fired
                self.insights.clear();
                self.list_state.select(None);
                self.status = Some(
                    "No insights yet. insights.db will be created when session-stop.sh fires."
                        .to_string(),
                );
            }
        }
    }

    /// Sync claude-mem observations from the DataStore and update visibility flag
    pub fn sync_claude_mem(&mut self, store: &Arc<ccboard_core::store::DataStore>) {
        self.show_claude_mem = store.is_claude_mem_enabled();
        if self.show_claude_mem {
            self.claude_mem_obs = store.claude_mem_observations();
        } else {
            self.claude_mem_obs.clear();
        }
    }

    /// Handle keyboard input for this tab. Returns true if consumed.
    pub fn handle_key(
        &mut self,
        key: KeyCode,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) -> bool {
        match key {
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
                true
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.filter = self.filter.next();
                self.expanded = false;
                self.reload();
                true
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.filter = self.filter.prev();
                self.expanded = false;
                self.reload();
                true
            }
            KeyCode::Enter => {
                self.expanded = !self.expanded;
                true
            }
            KeyCode::Char('d') => {
                self.archive_selected();
                true
            }
            KeyCode::Char('r') => {
                self.reload();
                if let Some(s) = store {
                    s.reload_claude_mem_observations();
                    self.sync_claude_mem(s);
                }
                true
            }
            KeyCode::Char('M') => {
                if let Some(s) = store {
                    let new_state = !s.is_claude_mem_enabled();
                    s.toggle_claude_mem(new_state);
                    self.sync_claude_mem(s);
                    let label = if new_state { "enabled" } else { "disabled" };
                    self.status = Some(format!("claude-mem integration {label}"));
                }
                true
            }
            _ => false,
        }
    }

    fn move_selection(&mut self, delta: i64) {
        if self.insights.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as i64;
        let next = (current + delta).max(0).min(self.insights.len() as i64 - 1) as usize;
        self.list_state.select(Some(next));
        self.expanded = false;
    }

    fn archive_selected(&mut self) {
        let Some(idx) = self.list_state.selected() else {
            return;
        };
        let Some(insight) = self.insights.get(idx) else {
            return;
        };
        let id = insight.id;

        match InsightsDb::new(&self.db_dir) {
            Ok(db) => {
                if let Err(e) = db.archive(id) {
                    self.status = Some(format!("Archive failed: {e}"));
                } else {
                    self.status = Some(format!("Archived insight #{id}"));
                    self.reload();
                }
            }
            Err(e) => {
                self.status = Some(format!("DB error: {e}"));
            }
        }
    }

    /// Render the Brain tab (auto-loads on first render)
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        scheme: ColorScheme,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) {
        // Lazy load: if never loaded, fetch from DB now
        if self.status.is_none() {
            self.reload();
            if let Some(s) = store {
                self.sync_claude_mem(s);
            }
        }
        let p = Palette::new(scheme);

        // Header row + main body + footer
        let chunks = Layout::vertical([
            Constraint::Length(3), // filter bar
            Constraint::Min(5),    // list + detail
            Constraint::Length(1), // keybind hint
        ])
        .split(area);

        self.render_filter_bar(frame, chunks[0], &p);
        self.render_body(frame, chunks[1], &p);
        self.render_footer(frame, chunks[2], &p, store);
    }

    fn render_filter_bar(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        let filters = [
            FilterType::All,
            FilterType::Progress,
            FilterType::Decision,
            FilterType::Blocked,
            FilterType::Pattern,
            FilterType::Fix,
            FilterType::Context,
        ];

        let mut spans: Vec<Span> = filters
            .iter()
            .flat_map(|&f| {
                let is_active = f == self.filter;
                let label = format!(" {} ", f.label());
                let style = if is_active {
                    Style::default()
                        .fg(p.bg)
                        .bg(p.focus)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(p.muted)
                };
                [Span::styled(label, style), Span::raw(" ")]
            })
            .collect();

        // claude-mem badge
        if self.show_claude_mem {
            spans.push(Span::raw("  "));
            spans.push(Span::styled(
                " claude-mem ON ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let bar = Paragraph::new(Line::from(spans))
            .block(
                Block::default()
                    .title(" Brain ")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(p.focus)),
            )
            .style(Style::default().bg(p.bg));

        frame.render_widget(bar, area);
    }

    fn render_body(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let has_insights = !self.insights.is_empty();
        let has_mem = self.show_claude_mem && !self.claude_mem_obs.is_empty();

        if !has_insights && !has_mem {
            let msg = self
                .status
                .as_deref()
                .unwrap_or("No insights captured yet.");
            let para = Paragraph::new(msg)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(p.border)),
                )
                .style(Style::default().fg(p.muted).bg(p.bg))
                .wrap(Wrap { trim: true });
            frame.render_widget(para, area);
            return;
        }

        if has_mem {
            // Split: ccboard insights (top) | claude-mem section (bottom)
            let mem_height = (self.claude_mem_obs.len().min(8) + 2) as u16;
            let [insights_area, mem_area] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(mem_height)]).split(area)
                    [..]
            else {
                unreachable!()
            };
            self.render_insights_list(frame, insights_area, p);
            self.render_claude_mem_section(frame, mem_area, p);
        } else {
            self.render_insights_list(frame, area, p);
        }
    }

    fn render_insights_list(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        if self.insights.is_empty() {
            let msg = self
                .status
                .as_deref()
                .unwrap_or("No insights captured yet.");
            let para = Paragraph::new(msg)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(p.border)),
                )
                .style(Style::default().fg(p.muted).bg(p.bg))
                .wrap(Wrap { trim: true });
            frame.render_widget(para, area);
            return;
        }

        // Split body into list | detail when expanded
        let (list_area, detail_area) = if self.expanded {
            let [l, d] =
                Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                    .split(area)[..]
            else {
                unreachable!()
            };
            (l, Some(d))
        } else {
            (area, None)
        };

        // Build list items
        let selected_idx = self.list_state.selected().unwrap_or(usize::MAX);
        let items: Vec<ListItem> = self
            .insights
            .iter()
            .enumerate()
            .map(|(i, insight)| {
                let type_icon = type_icon(&insight.insight_type);
                let type_color = type_color(&insight.insight_type, p);
                let date = insight.created_at.format("%m/%d").to_string();
                let project = basename(&insight.project);
                // Truncate content to fit
                let content: String = insight.content.chars().take(60).collect();
                let content = if insight.content.len() > 60 {
                    format!("{content}…")
                } else {
                    content
                };

                let style = if i == selected_idx {
                    Style::default().fg(p.bg).bg(p.focus)
                } else {
                    Style::default().fg(p.fg).bg(p.bg)
                };

                let line = Line::from(vec![
                    Span::styled(format!("{type_icon} "), Style::default().fg(type_color)),
                    Span::styled(format!("{date}  "), Style::default().fg(p.muted)),
                    Span::styled(format!("{project:<12}  "), Style::default().fg(p.muted)),
                    Span::styled(content, style),
                ]);
                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(p.border)),
            )
            .highlight_style(Style::default().fg(p.bg).bg(p.focus));

        frame.render_stateful_widget(list, list_area, &mut self.list_state);

        // Detail pane
        if let (Some(d_area), Some(insight)) = (detail_area, self.insights.get(selected_idx)) {
            let type_label = format!(
                "{} {}",
                type_icon(&insight.insight_type),
                insight.insight_type.as_str().to_uppercase()
            );
            let date = insight.created_at.format("%Y-%m-%d %H:%M UTC").to_string();
            let project = &insight.project;

            let mut lines = vec![
                Line::from(Span::styled(
                    &type_label,
                    Style::default()
                        .fg(type_color(&insight.insight_type, p))
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(date, Style::default().fg(p.muted))),
                Line::from(Span::styled(project.as_str(), Style::default().fg(p.muted))),
                Line::raw(""),
                Line::from(Span::styled(&insight.content, Style::default().fg(p.fg))),
            ];

            if let Some(reasoning) = &insight.reasoning {
                lines.push(Line::raw(""));
                lines.push(Line::from(Span::styled(
                    "Reasoning:",
                    Style::default().fg(p.muted),
                )));
                lines.push(Line::from(Span::styled(
                    reasoning.as_str(),
                    Style::default().fg(p.fg),
                )));
            }

            let detail = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Detail ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(p.focus)),
                )
                .style(Style::default().bg(p.surface))
                .wrap(Wrap { trim: false });

            frame.render_widget(detail, d_area);
        }
    }

    fn render_claude_mem_section(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        let items: Vec<ListItem> = self
            .claude_mem_obs
            .iter()
            .take(8)
            .map(|obs| {
                let icon = obs.icon();
                let date = format_claude_mem_date(&obs.created_at);
                let project = basename(&obs.project);
                let text: String = obs.display_text().chars().take(60).collect();
                let text = if obs.display_text().len() > 60 {
                    format!("{text}…")
                } else {
                    text
                };

                let line = Line::from(vec![
                    Span::styled(format!("{icon} "), Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{date}  "), Style::default().fg(p.muted)),
                    Span::styled(format!("{project:<12}  "), Style::default().fg(p.muted)),
                    Span::styled(text, Style::default().fg(p.fg)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let title = format!(" claude-mem ({}) ", self.claude_mem_obs.len());
        let list = List::new(items).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(list, area);
    }

    fn render_footer(
        &self,
        frame: &mut Frame,
        area: Rect,
        p: &Palette,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) {
        let mem_hint = if store.map(|s| s.is_claude_mem_enabled()).unwrap_or(false) {
            "  [M] disable claude-mem"
        } else {
            "  [M] enable claude-mem"
        };
        let hints =
            format!("[j/k] nav  [←/→] filter  [Enter] expand  [d] archive  [r] reload{mem_hint}");
        let footer = Paragraph::new(hints.as_str()).style(Style::default().fg(p.muted).bg(p.bg));
        frame.render_widget(footer, area);
    }
}

impl Default for BrainTab {
    fn default() -> Self {
        Self::new()
    }
}

fn type_icon(t: &InsightType) -> &'static str {
    match t {
        InsightType::Progress => "●",
        InsightType::Decision => "◆",
        InsightType::Blocked => "▲",
        InsightType::Pattern => "◉",
        InsightType::Fix => "✦",
        InsightType::Context => "○",
    }
}

fn type_color(t: &InsightType, p: &Palette) -> ratatui::style::Color {
    match t {
        InsightType::Progress => p.success,
        InsightType::Decision => p.focus,
        InsightType::Blocked => p.error,
        InsightType::Pattern => p.important,
        InsightType::Fix => p.warning,
        InsightType::Context => p.muted,
    }
}

fn basename(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string()
}

/// Format a claude-mem ISO timestamp to MM/DD
fn format_claude_mem_date(created_at: &str) -> String {
    // Timestamps are like "2026-03-30T09:42:12.719Z"
    if created_at.len() >= 10 {
        let date = &created_at[5..10]; // "MM-DD"
        date.replace('-', "/")
    } else {
        created_at.to_string()
    }
}
