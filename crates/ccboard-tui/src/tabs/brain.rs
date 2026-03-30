//! Brain tab — cross-session knowledge base from ~/.ccboard/insights.db
//! + optional claude-mem session summaries from ~/.claude-mem/claude-mem.db
//!
//! Keybindings:
//! - j/k or ↑/↓: Navigate active section
//! - Tab: Switch focus between ccboard insights and claude-mem summaries
//! - Enter: Expand/collapse detail
//! - d: Archive selected insight (ccboard section only)
//! - ←/→ or h/l: Cycle type filter (ccboard section)
//! - r: Reload both sources
//! - M: Toggle claude-mem integration (persists to ~/.ccboard/config.toml)

use crate::theme::Palette;
use ccboard_core::cache::InsightsDb;
use ccboard_core::models::claude_mem::ClaudeMemSummary;
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

/// Which section has keyboard focus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum FocusSection {
    #[default]
    Insights,
    ClaudeMem,
}

pub struct BrainTab {
    insights: Vec<Insight>,
    list_state: ListState,
    filter: FilterType,
    expanded: bool,
    db_dir: PathBuf,
    pub status: Option<String>,
    /// claude-mem session summaries
    summaries: Vec<ClaudeMemSummary>,
    mem_list_state: ListState,
    mem_expanded: bool,
    focus: FocusSection,
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
            summaries: Vec::new(),
            mem_list_state: ListState::default(),
            mem_expanded: false,
            focus: FocusSection::Insights,
            show_claude_mem: false,
        }
    }

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
                self.insights.clear();
                self.list_state.select(None);
                self.status = Some(
                    "No insights yet. insights.db will be created when session-stop.sh fires."
                        .to_string(),
                );
            }
        }
    }

    pub fn sync_claude_mem(&mut self, store: &Arc<ccboard_core::store::DataStore>) {
        self.show_claude_mem = store.is_claude_mem_enabled();
        if self.show_claude_mem {
            self.summaries = store.claude_mem_summaries();
            if !self.summaries.is_empty() && self.mem_list_state.selected().is_none() {
                self.mem_list_state.select(Some(0));
            }
        } else {
            self.summaries.clear();
            self.mem_list_state.select(None);
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyCode,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) -> bool {
        match key {
            // Tab switches focus between the two sections
            KeyCode::Tab => {
                if self.show_claude_mem {
                    self.focus = match self.focus {
                        FocusSection::Insights => FocusSection::ClaudeMem,
                        FocusSection::ClaudeMem => FocusSection::Insights,
                    };
                }
                true
            }
            KeyCode::Char('j') | KeyCode::Down => {
                match self.focus {
                    FocusSection::Insights => self.move_insight_selection(1),
                    FocusSection::ClaudeMem => self.move_mem_selection(1),
                }
                true
            }
            KeyCode::Char('k') | KeyCode::Up => {
                match self.focus {
                    FocusSection::Insights => self.move_insight_selection(-1),
                    FocusSection::ClaudeMem => self.move_mem_selection(-1),
                }
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.focus == FocusSection::Insights {
                    self.filter = self.filter.next();
                    self.expanded = false;
                    self.reload();
                }
                true
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.focus == FocusSection::Insights {
                    self.filter = self.filter.prev();
                    self.expanded = false;
                    self.reload();
                }
                true
            }
            KeyCode::Enter => {
                match self.focus {
                    FocusSection::Insights => self.expanded = !self.expanded,
                    FocusSection::ClaudeMem => self.mem_expanded = !self.mem_expanded,
                }
                true
            }
            KeyCode::Char('d') => {
                if self.focus == FocusSection::Insights {
                    self.archive_selected();
                }
                true
            }
            KeyCode::Char('r') => {
                self.reload();
                if let Some(s) = store {
                    s.reload_claude_mem_summaries();
                    self.sync_claude_mem(s);
                }
                true
            }
            KeyCode::Char('M') => {
                if let Some(s) = store {
                    let new_state = !s.is_claude_mem_enabled();
                    s.toggle_claude_mem(new_state);
                    self.sync_claude_mem(s);
                    if new_state {
                        self.focus = FocusSection::ClaudeMem;
                    } else {
                        self.focus = FocusSection::Insights;
                    }
                    let label = if new_state { "enabled" } else { "disabled" };
                    self.status = Some(format!("claude-mem {label}"));
                }
                true
            }
            _ => false,
        }
    }

    fn move_insight_selection(&mut self, delta: i64) {
        if self.insights.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as i64;
        let next = (current + delta).max(0).min(self.insights.len() as i64 - 1) as usize;
        self.list_state.select(Some(next));
        self.expanded = false;
    }

    fn move_mem_selection(&mut self, delta: i64) {
        if self.summaries.is_empty() {
            return;
        }
        let current = self.mem_list_state.selected().unwrap_or(0) as i64;
        let next = (current + delta)
            .max(0)
            .min(self.summaries.len() as i64 - 1) as usize;
        self.mem_list_state.select(Some(next));
        self.mem_expanded = false;
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

    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        scheme: ColorScheme,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) {
        if self.status.is_none() {
            self.reload();
            if let Some(s) = store {
                self.sync_claude_mem(s);
            }
        }
        let p = Palette::new(scheme);

        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(1),
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

        if self.show_claude_mem {
            spans.push(Span::raw("  "));
            let mem_style = if self.focus == FocusSection::ClaudeMem {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            };
            spans.push(Span::styled(" claude-mem ON ", mem_style));
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
        if self.show_claude_mem && !self.summaries.is_empty() {
            // Split: insights (top 55%) | claude-mem summaries (bottom 45%)
            let [insights_area, mem_area] =
                Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .split(area)[..]
            else {
                unreachable!()
            };
            self.render_insights_panel(frame, insights_area, p);
            self.render_mem_panel(frame, mem_area, p);
        } else if self.show_claude_mem {
            // claude-mem enabled but no summaries yet
            let [insights_area, mem_area] =
                Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).split(area)[..]
            else {
                unreachable!()
            };
            self.render_insights_panel(frame, insights_area, p);
            let para = Paragraph::new("No session summaries yet in claude-mem DB.")
                .block(
                    Block::default()
                        .title(" claude-mem ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(p.muted).bg(p.bg));
            frame.render_widget(para, mem_area);
        } else {
            self.render_insights_panel(frame, area, p);
        }
    }

    fn render_insights_panel(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let is_focused = self.focus == FocusSection::Insights;
        let border_color = if is_focused { p.focus } else { p.border };

        if self.insights.is_empty() {
            let msg = self
                .status
                .as_deref()
                .unwrap_or("No insights captured yet.");
            let para = Paragraph::new(msg)
                .block(
                    Block::default()
                        .title(" ccboard insights ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(border_color)),
                )
                .style(Style::default().fg(p.muted).bg(p.bg))
                .wrap(Wrap { trim: true });
            frame.render_widget(para, area);
            return;
        }

        let (list_area, detail_area) = if self.expanded && is_focused {
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

        let selected_idx = self.list_state.selected().unwrap_or(usize::MAX);
        let items: Vec<ListItem> = self
            .insights
            .iter()
            .enumerate()
            .map(|(i, insight)| {
                let icon = type_icon(&insight.insight_type);
                let color = type_color(&insight.insight_type, p);
                let date = insight.created_at.format("%m/%d").to_string();
                let project = basename(&insight.project);
                let content: String = insight.content.chars().take(60).collect();
                let content = if insight.content.len() > 60 {
                    format!("{content}…")
                } else {
                    content
                };
                let style = if i == selected_idx && is_focused {
                    Style::default().fg(p.bg).bg(p.focus)
                } else if i == selected_idx {
                    Style::default().fg(p.fg).bg(p.surface)
                } else {
                    Style::default().fg(p.fg).bg(p.bg)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{icon} "), Style::default().fg(color)),
                    Span::styled(format!("{date}  "), Style::default().fg(p.muted)),
                    Span::styled(format!("{project:<12}  "), Style::default().fg(p.muted)),
                    Span::styled(content, style),
                ]))
            })
            .collect();

        let title = if self.show_claude_mem {
            " ccboard insights "
        } else {
            " Brain "
        };
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().fg(p.bg).bg(p.focus));

        frame.render_stateful_widget(list, list_area, &mut self.list_state);

        if let (Some(d_area), Some(insight)) = (detail_area, self.insights.get(selected_idx)) {
            let type_label = format!(
                "{} {}",
                type_icon(&insight.insight_type),
                insight.insight_type.as_str().to_uppercase()
            );
            let date = insight.created_at.format("%Y-%m-%d %H:%M UTC").to_string();
            let mut lines = vec![
                Line::from(Span::styled(
                    &type_label,
                    Style::default()
                        .fg(type_color(&insight.insight_type, p))
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(date, Style::default().fg(p.muted))),
                Line::from(Span::styled(
                    insight.project.as_str(),
                    Style::default().fg(p.muted),
                )),
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

    fn render_mem_panel(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let is_focused = self.focus == FocusSection::ClaudeMem;
        let border_color = if is_focused { Color::Cyan } else { p.border };

        let selected_idx = self.mem_list_state.selected().unwrap_or(usize::MAX);

        // When expanded and focused, show list + detail side by side
        let (list_area, detail_area) = if self.mem_expanded && is_focused {
            let [l, d] =
                Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
                    .split(area)[..]
            else {
                unreachable!()
            };
            (l, Some(d))
        } else {
            (area, None)
        };

        let items: Vec<ListItem> = self
            .summaries
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let date = format_mem_date(&s.created_at);
                let project = basename(&s.project);
                let headline: String = s.headline().chars().take(65).collect();
                let headline = if s.headline().len() > 65 {
                    format!("{headline}…")
                } else {
                    headline
                };
                let style = if i == selected_idx && is_focused {
                    Style::default().fg(Color::Black).bg(Color::Cyan)
                } else if i == selected_idx {
                    Style::default().fg(Color::Cyan).bg(p.surface)
                } else {
                    Style::default().fg(p.fg).bg(p.bg)
                };
                ListItem::new(Line::from(vec![
                    Span::styled("◈ ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{date}  "), Style::default().fg(p.muted)),
                    Span::styled(format!("{project:<12}  "), Style::default().fg(p.muted)),
                    Span::styled(headline, style),
                ]))
            })
            .collect();

        let title = format!(" claude-mem ({}) ", self.summaries.len());
        let list = List::new(items)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(border_color)),
            )
            .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan));

        frame.render_stateful_widget(list, list_area, &mut self.mem_list_state);

        // Detail pane — shows what was asked, what was done, next steps
        if let (Some(d_area), Some(summary)) = (detail_area, self.summaries.get(selected_idx)) {
            let date = format_mem_date_full(&summary.created_at);
            let project = &summary.project;

            let mut lines = vec![
                Line::from(Span::styled(
                    format!("◈  {project}  ·  {date}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::raw(""),
            ];

            if let Some(ref req) = summary.request {
                lines.push(Line::from(Span::styled(
                    "Asked:",
                    Style::default().fg(p.muted).add_modifier(Modifier::BOLD),
                )));
                for line in wrap_text(req, 55) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.fg))));
                }
                lines.push(Line::raw(""));
            }

            if let Some(ref done) = summary.completed {
                lines.push(Line::from(Span::styled(
                    "Done:",
                    Style::default().fg(p.success).add_modifier(Modifier::BOLD),
                )));
                for line in wrap_text(done, 55) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.fg))));
                }
                lines.push(Line::raw(""));
            }

            if let Some(ref next) = summary.next_steps {
                lines.push(Line::from(Span::styled(
                    "Next:",
                    Style::default().fg(p.warning).add_modifier(Modifier::BOLD),
                )));
                for line in wrap_text(next, 55) {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(p.fg))));
                }
            }

            let detail = Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" Session detail ")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().bg(p.surface))
                .wrap(Wrap { trim: false });
            frame.render_widget(detail, d_area);
        }
    }

    fn render_footer(
        &self,
        frame: &mut Frame,
        area: Rect,
        p: &Palette,
        store: Option<&Arc<ccboard_core::store::DataStore>>,
    ) {
        let mem_enabled = store.map(|s| s.is_claude_mem_enabled()).unwrap_or(false);
        let section_hint = if self.show_claude_mem {
            "  [Tab] switch section"
        } else {
            ""
        };
        let mem_hint = if mem_enabled {
            "  [M] disable claude-mem"
        } else {
            "  [M] enable claude-mem"
        };
        let base = match self.focus {
            FocusSection::Insights => "[j/k] nav  [←/→] filter  [Enter] expand  [d] archive",
            FocusSection::ClaudeMem => "[j/k] nav  [Enter] expand detail",
        };
        let hints = format!("{base}{section_hint}  [r] reload{mem_hint}");
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

/// Format ISO timestamp to MM/DD
fn format_mem_date(created_at: &str) -> String {
    if created_at.len() >= 10 {
        created_at[5..10].replace('-', "/")
    } else {
        created_at.to_string()
    }
}

/// Format ISO timestamp to YYYY-MM-DD HH:MM
fn format_mem_date_full(created_at: &str) -> String {
    if created_at.len() >= 16 {
        created_at[..16].replace('T', " ")
    } else {
        created_at.to_string()
    }
}

/// Naive word-wrap: split text into lines of at most `width` chars
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let paragraph = paragraph.trim();
        if paragraph.is_empty() {
            continue;
        }
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current.push_str(word);
            } else if current.len() + 1 + word.len() <= width {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(current.clone());
                current = word.to_string();
            }
        }
        if !current.is_empty() {
            lines.push(current);
        }
    }
    if lines.is_empty() {
        lines.push(text.chars().take(width).collect());
    }
    lines
}
