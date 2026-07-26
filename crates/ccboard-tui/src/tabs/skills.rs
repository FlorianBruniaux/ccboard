//! Skills leaderboard tab - invocation analytics for skills across all sessions
//!
//! Distinct from the "Skills" sub-tab in `tabs/agents.rs`, which lists on-disk skill
//! *definitions* (frontmatter, description, tools). This tab shows the invocation
//! *leaderboard* built from `DataStore::skill_leaderboard()`: how often each skill
//! was actually used, by how many projects, and at what token/cost cost.

use crate::theme::Palette;
use ccboard_core::models::config::ColorScheme;
use ccboard_core::models::SkillLeaderboardEntry;
use ccboard_core::DataStore;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, ListState, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};
use std::collections::HashSet;
use std::sync::Arc;

/// Skills leaderboard tab state
pub struct SkillsTab {
    entries: Vec<SkillLeaderboardEntry>,
    /// Names of skills currently defined on disk (from `.claude/skills/` scan), used to
    /// recompute the `[retired]` badge at render time. Never persisted as a "defined" flag.
    known_skill_names: HashSet<String>,
    list_state: ListState,
    show_detail: bool,
}

impl Default for SkillsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillsTab {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            entries: Vec::new(),
            known_skill_names: HashSet::new(),
            list_state,
            show_detail: false,
        }
    }

    /// Set the on-disk skill catalogue, used to compute the `[retired]` badge.
    /// Names are stored verbatim, never split on `:`.
    pub fn set_known_skills(&mut self, names: impl IntoIterator<Item = String>) {
        self.known_skill_names = names.into_iter().collect();
    }

    /// Refresh the leaderboard from the store. Cheap: reads an already-persisted
    /// SQLite aggregate, not a rescan.
    pub fn refresh(&mut self, store: &Arc<DataStore>) {
        let mut entries = store.skill_leaderboard();
        entries.sort_by(|a, b| {
            b.invocations
                .cmp(&a.invocations)
                .then_with(|| a.skill.cmp(&b.skill))
        });
        self.entries = entries;
    }

    /// A skill is retired if it has invocation history but is no longer defined on disk.
    fn is_retired(&self, skill: &str) -> bool {
        !self.known_skill_names.contains(skill)
    }

    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Char('g') => self.list_state.select(Some(0)),
            KeyCode::Char('G') => {
                if !self.entries.is_empty() {
                    self.list_state.select(Some(self.entries.len() - 1));
                }
            }
            KeyCode::Enter => {
                self.show_detail = !self.show_detail;
            }
            KeyCode::Esc => {
                self.show_detail = false;
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, self.entries.len() as i32 - 1) as usize;
        self.list_state.select(Some(new_idx));
    }

    fn selected_entry(&self) -> Option<&SkillLeaderboardEntry> {
        self.list_state.selected().and_then(|i| self.entries.get(i))
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, scheme: ColorScheme) {
        let p = Palette::new(scheme);

        if self.entries.is_empty() {
            let empty = crate::empty_state::EmptyState::new("No Skill Invocations Yet")
                .message("Skill usage appears here once a Skill tool_use block is recorded in a session.")
                .build();
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(p.border))
                .style(Style::default().bg(p.surface))
                .title(" Skills Leaderboard ");
            let inner = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(empty, inner);
            return;
        }

        // Clamp selection
        if let Some(sel) = self.list_state.selected() {
            if sel >= self.entries.len() {
                self.list_state.select(Some(self.entries.len() - 1));
            }
        }

        let content_constraints = if self.show_detail {
            vec![Constraint::Percentage(65), Constraint::Percentage(35)]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(content_constraints)
            .split(area);

        self.render_table(frame, chunks[0], &p);

        if self.show_detail && chunks.len() > 1 {
            self.render_detail(frame, chunks[1], &p);
        }
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect, p: &Palette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(p.border))
            .style(Style::default().bg(p.surface))
            .title(Span::styled(
                format!(
                    " Skills Leaderboard ({}) • Enter:detail ",
                    self.entries.len()
                ),
                Style::default().fg(p.fg).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let header = Row::new(vec![
            Cell::from("Skill"),
            Cell::from("Uses"),
            Cell::from("Projects"),
            Cell::from("Tokens"),
            Cell::from("Cost"),
            Cell::from("Last seen"),
        ])
        .style(Style::default().fg(p.muted).bold());

        let selected = self.list_state.selected();
        let rows: Vec<Row> = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let retired = self.is_retired(&entry.skill);
                let is_selected = selected == Some(i);

                let name_style = if is_selected {
                    Style::default().fg(p.focus).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(p.fg)
                };

                let name_cell = if retired {
                    Cell::from(Line::from(vec![
                        Span::styled(entry.skill.clone(), name_style),
                        Span::raw(" "),
                        Span::styled(
                            "[retired]",
                            Style::default()
                                .fg(p.warning)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    ]))
                } else {
                    Cell::from(Span::styled(entry.skill.clone(), name_style))
                };

                Row::new(vec![
                    name_cell,
                    Cell::from(entry.invocations.to_string()),
                    Cell::from(entry.projects.to_string()),
                    Cell::from(format_tokens(entry.total_tokens)),
                    Cell::from(format!("${:.2}", entry.estimated_cost_usd)),
                    Cell::from(format_timestamp(entry.last_seen)),
                ])
                .style(if is_selected {
                    Style::default().bg(p.muted).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
            })
            .collect();

        let widths = [
            Constraint::Min(24),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths).header(header).column_spacing(1);
        frame.render_widget(table, inner);

        if self.entries.len() > (inner.height as usize).saturating_sub(1) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state =
                ScrollbarState::new(self.entries.len()).position(selected.unwrap_or(0));
            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(p.focus))
            .style(Style::default().bg(p.surface))
            .title(Span::styled(" Detail ", Style::default().fg(p.fg).bold()));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(entry) = self.selected_entry() else {
            let empty =
                Paragraph::new("Select a skill to see details").style(Style::default().fg(p.muted));
            frame.render_widget(empty, inner);
            return;
        };

        let retired = self.is_retired(&entry.skill);

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Skill: ", Style::default().fg(p.muted)),
                Span::styled(&entry.skill, Style::default().fg(p.fg).bold()),
            ]),
            Line::from(""),
        ];

        if retired {
            lines.push(Line::from(Span::styled(
                "[retired] invocation history exists but this skill is no longer defined on disk",
                Style::default().fg(p.warning).italic(),
            )));
            lines.push(Line::from(""));
        }

        lines.push(Line::from(vec![
            Span::styled("Invocations: ", Style::default().fg(p.muted)),
            Span::styled(entry.invocations.to_string(), Style::default().fg(p.fg)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Projects: ", Style::default().fg(p.muted)),
            Span::styled(entry.projects.to_string(), Style::default().fg(p.fg)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Total tokens: ", Style::default().fg(p.muted)),
            Span::styled(format_tokens(entry.total_tokens), Style::default().fg(p.fg)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Estimated cost: ", Style::default().fg(p.muted)),
            Span::styled(
                format!("${:.2}", entry.estimated_cost_usd),
                Style::default().fg(p.warning),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("First seen: ", Style::default().fg(p.muted)),
            Span::styled(
                format_timestamp(entry.first_seen),
                Style::default().fg(p.fg),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Last seen: ", Style::default().fg(p.muted)),
            Span::styled(format_timestamp(entry.last_seen), Style::default().fg(p.fg)),
        ]));

        let detail = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(detail, inner);
    }
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

fn format_timestamp(ts: Option<chrono::DateTime<chrono::Utc>>) -> String {
    match ts {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => "—".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(skill: &str, invocations: usize) -> SkillLeaderboardEntry {
        SkillLeaderboardEntry {
            skill: skill.to_string(),
            invocations,
            projects: 1,
            total_tokens: 100,
            estimated_cost_usd: 0.01,
            first_seen: None,
            last_seen: None,
        }
    }

    #[test]
    fn retired_badge_fires_only_when_absent_from_disk_catalogue() {
        let mut tab = SkillsTab::new();
        tab.set_known_skills(["superpowers:brainstorming".to_string()]);
        tab.entries = vec![entry("superpowers:brainstorming", 5), entry("old-skill", 3)];

        assert!(!tab.is_retired("superpowers:brainstorming"));
        assert!(tab.is_retired("old-skill"));
    }

    #[test]
    fn namespaced_and_local_skill_names_are_distinct() {
        let mut tab = SkillsTab::new();
        tab.set_known_skills(["brainstorming".to_string()]);

        // A local "brainstorming" being defined does NOT clear the retired
        // badge for the differently-namespaced "superpowers:brainstorming".
        assert!(!tab.is_retired("brainstorming"));
        assert!(tab.is_retired("superpowers:brainstorming"));
    }

    #[test]
    fn refresh_sorts_by_invocations_descending() {
        let mut tab = SkillsTab::new();
        tab.entries = vec![entry("a", 1), entry("b", 10), entry("c", 5)];
        tab.entries.sort_by(|a, b| {
            b.invocations
                .cmp(&a.invocations)
                .then_with(|| a.skill.cmp(&b.skill))
        });

        let names: Vec<&str> = tab.entries.iter().map(|e| e.skill.as_str()).collect();
        assert_eq!(names, vec!["b", "c", "a"]);
    }
}
