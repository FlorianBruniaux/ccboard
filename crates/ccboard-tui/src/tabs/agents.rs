//! Agents tab - Browse agents, commands, and skills from .claude directory

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Tabs,
    },
};
use std::path::Path;

/// Agent/command/skill entry
#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub name: String,
    pub file_path: String,
    pub description: Option<String>,
    pub entry_type: AgentType,
    /// Number of times this agent/command/skill has been invoked (TODO: implement counting)
    pub invocation_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Agent,
    Command,
    Skill,
}

impl AgentType {
    fn label(&self) -> &'static str {
        match self {
            AgentType::Agent => "Agents",
            AgentType::Command => "/ Commands",
            AgentType::Skill => "Skills",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            AgentType::Agent => "🤖",   // Robot - AI agents
            AgentType::Command => "⚡", // Lightning - quick commands
            AgentType::Skill => "✨",   // Sparkles - capabilities
        }
    }

    fn color(&self) -> Color {
        match self {
            AgentType::Agent => Color::Cyan,
            AgentType::Command => Color::Green,
            AgentType::Skill => Color::Yellow,
        }
    }
}

/// Agents tab state
pub struct AgentsTab {
    /// Current sub-tab (0=Agents, 1=Commands, 2=Skills)
    sub_tab: usize,
    /// List states for each sub-tab
    list_states: [ListState; 3],
    /// Cached entries
    agents: Vec<AgentEntry>,
    commands: Vec<AgentEntry>,
    skills: Vec<AgentEntry>,
    /// Show detail panel
    show_detail: bool,
    /// Error message to display (if any)
    error_message: Option<String>,
}

impl Default for AgentsTab {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentsTab {
    pub fn new() -> Self {
        let mut list_states: [ListState; 3] = Default::default();
        for state in &mut list_states {
            state.select(Some(0));
        }

        Self {
            sub_tab: 0,
            list_states,
            agents: Vec::new(),
            commands: Vec::new(),
            skills: Vec::new(),
            show_detail: false,
            error_message: None,
        }
    }

    /// Get current sub-tab index
    pub fn current_sub_tab(&self) -> usize {
        self.sub_tab
    }

    /// Get current sub-tab label
    pub fn current_sub_tab_label(&self) -> &'static str {
        match self.sub_tab {
            0 => "Agents",
            1 => "Commands",
            2 => "Skills",
            _ => "Unknown",
        }
    }

    /// Scan directories for agents/commands/skills
    pub fn scan_directories(&mut self, claude_home: &Path, project_path: Option<&Path>) {
        self.agents.clear();
        self.commands.clear();
        self.skills.clear();

        // Scan global and project directories
        let dirs_to_scan: Vec<&Path> = [Some(claude_home), project_path]
            .into_iter()
            .flatten()
            .collect();

        for base_dir in dirs_to_scan {
            self.scan_directory(base_dir, "agents", AgentType::Agent);
            self.scan_directory(base_dir, "commands", AgentType::Command);
            self.scan_directory(base_dir, "skills", AgentType::Skill);
        }

        // Sort all lists by name initially (will be re-sorted by usage later if stats available)
        self.agents.sort_by(|a, b| a.name.cmp(&b.name));
        self.commands.sort_by(|a, b| a.name.cmp(&b.name));
        self.skills.sort_by(|a, b| a.name.cmp(&b.name));
    }

    /// Update invocation counts from stats and sort by usage
    pub fn update_invocation_counts(&mut self, stats: &ccboard_core::models::InvocationStats) {
        // Update agent counts
        for agent in &mut self.agents {
            agent.invocation_count = stats.agents.get(&agent.name).copied().unwrap_or(0);
        }

        // Update command counts (need to add / prefix for matching)
        for command in &mut self.commands {
            let key = format!("/{}", command.name);
            command.invocation_count = stats.commands.get(&key).copied().unwrap_or(0);
        }

        // Update skill counts
        for skill in &mut self.skills {
            skill.invocation_count = stats.skills.get(&skill.name).copied().unwrap_or(0);
        }

        // Sort by usage (descending), then by name (ascending) as tie-breaker
        self.agents.sort_by(|a, b| {
            b.invocation_count
                .cmp(&a.invocation_count)
                .then(a.name.cmp(&b.name))
        });
        self.commands.sort_by(|a, b| {
            b.invocation_count
                .cmp(&a.invocation_count)
                .then(a.name.cmp(&b.name))
        });
        self.skills.sort_by(|a, b| {
            b.invocation_count
                .cmp(&a.invocation_count)
                .then(a.name.cmp(&b.name))
        });
    }

    fn scan_directory(&mut self, base: &Path, subdir: &str, entry_type: AgentType) {
        let dir = base.join(subdir);
        if !dir.exists() {
            return;
        }

        self.scan_recursive(&dir, entry_type);
    }

    fn scan_recursive(&mut self, dir: &Path, entry_type: AgentType) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();

            // Case 1: Direct .md file
            if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                // Skip README files
                if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("README") || n.starts_with("_README"))
                {
                    continue;
                }

                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let description = Self::extract_description(&path);

                let target_list = match entry_type {
                    AgentType::Agent => &mut self.agents,
                    AgentType::Command => &mut self.commands,
                    AgentType::Skill => &mut self.skills,
                };

                target_list.push(AgentEntry {
                    name,
                    file_path: path.display().to_string(),
                    description,
                    entry_type,
                    invocation_count: 0, // TODO: implement counting from sessions
                });
            }
            // Case 2: Directory containing SKILL.md (standard skill format)
            else if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                if skill_md.exists() {
                    let name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let description = Self::extract_description(&skill_md);

                    let target_list = match entry_type {
                        AgentType::Agent => &mut self.agents,
                        AgentType::Command => &mut self.commands,
                        AgentType::Skill => &mut self.skills,
                    };

                    target_list.push(AgentEntry {
                        name,
                        file_path: skill_md.display().to_string(),
                        description,
                        entry_type,
                        invocation_count: 0, // TODO: implement counting from sessions
                    });
                } else {
                    // Case 3: Directory without SKILL.md → scan recursively for .md files
                    self.scan_recursive(&path, entry_type);
                }
            }
        }
    }

    fn extract_description(path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(path).ok()?;

        // Simple frontmatter extraction: look for description field
        if !content.starts_with("---") {
            return None;
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return None;
        }

        let frontmatter = parts[1];
        for line in frontmatter.lines() {
            let line = line.trim();
            if line.starts_with("description:") {
                let desc = line.strip_prefix("description:")?.trim();
                // Remove quotes if present
                let desc = desc.trim_matches('"').trim_matches('\'');
                return Some(desc.to_string());
            }
        }

        None
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.sub_tab = self.sub_tab.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.sub_tab = (self.sub_tab + 1).min(2);
            }
            KeyCode::Tab => {
                self.sub_tab = (self.sub_tab + 1) % 3;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_selection(-1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_selection(1);
            }
            KeyCode::Enter => {
                self.show_detail = !self.show_detail;
            }
            KeyCode::Char('e') => {
                // Open selected file in editor
                if let Some(entry) = self.get_selected_entry() {
                    let path = std::path::Path::new(&entry.file_path);
                    if let Err(e) = crate::editor::open_in_editor(path) {
                        self.error_message = Some(format!("Failed to open editor: {}", e));
                    }
                }
            }
            KeyCode::Char('o') => {
                // Reveal file in file manager
                if let Some(entry) = self.get_selected_entry() {
                    let path = std::path::Path::new(&entry.file_path);
                    if let Err(e) = crate::editor::reveal_in_file_manager(path) {
                        self.error_message = Some(format!("Failed to open file manager: {}", e));
                    }
                }
            }
            KeyCode::Esc => {
                if self.error_message.is_some() {
                    self.error_message = None;
                } else {
                    self.show_detail = false;
                }
            }
            _ => {}
        }
    }

    /// Get currently selected entry
    fn get_selected_entry(&self) -> Option<&AgentEntry> {
        let list = self.current_list();
        let state = &self.list_states[self.sub_tab];
        state.selected().and_then(|idx| list.get(idx))
    }

    fn move_selection(&mut self, delta: i32) {
        let list_len = self.current_list().len();
        if list_len == 0 {
            return;
        }
        let state = &mut self.list_states[self.sub_tab];
        let current = state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, list_len as i32 - 1) as usize;
        state.select(Some(new_idx));
    }

    fn current_list(&self) -> &[AgentEntry] {
        match self.sub_tab {
            0 => &self.agents,
            1 => &self.commands,
            2 => &self.skills,
            _ => &self.agents,
        }
    }

    /// Render the agents tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        _scheme: ccboard_core::models::config::ColorScheme,
    ) {
        // Layout: sub-tabs header (2 lines with padding) | content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(0)])
            .split(area);

        // Render sub-tabs
        self.render_sub_tabs(frame, chunks[0]);

        // Content area
        let content_constraints = if self.show_detail {
            vec![Constraint::Percentage(50), Constraint::Percentage(50)]
        } else {
            vec![Constraint::Percentage(100)]
        };

        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(content_constraints)
            .split(chunks[1]);

        // Render list
        self.render_list(frame, content_chunks[0]);

        // Render detail if open
        if self.show_detail && content_chunks.len() > 1 {
            self.render_detail(frame, content_chunks[1]);
        }

        // Render error popup if present
        if self.error_message.is_some() {
            self.render_error_popup(frame, area);
        }
    }

    fn render_sub_tabs(&self, frame: &mut Frame, area: Rect) {
        let titles: Vec<Line> = [
            (AgentType::Agent, self.agents.len()),
            (AgentType::Command, self.commands.len()),
            (AgentType::Skill, self.skills.len()),
        ]
        .iter()
        .enumerate()
        .map(|(i, (t, count))| {
            let style = if i == self.sub_tab {
                Style::default()
                    .fg(t.color())
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            Line::from(Span::styled(
                format!(" {} {} ({}) ", t.icon(), t.label(), count),
                style,
            ))
        })
        .collect();

        let tabs = Tabs::new(titles)
            .select(self.sub_tab)
            .divider(Span::styled("│", Style::default().fg(Color::DarkGray)))
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(tabs, area);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let entry_type = match self.sub_tab {
            0 => AgentType::Agent,
            1 => AgentType::Command,
            2 => AgentType::Skill,
            _ => AgentType::Agent,
        };

        // Get list length first for selection clamping
        let list_len = self.current_list().len();

        let title_text = if entry_type == AgentType::Command {
            format!(
                " {} - Press / in Claude Code to use • e:edit o:reveal ",
                entry_type.label()
            )
        } else {
            format!(" {} • e:edit o:reveal ", entry_type.label())
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                title_text,
                Style::default().fg(Color::White).bold(),
            ));

        if list_len == 0 {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("No {} found", entry_type.label().to_lowercase()),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    format!(
                        "Create .md files in .claude/{}/",
                        match entry_type {
                            AgentType::Agent => "agents",
                            AgentType::Command => "commands",
                            AgentType::Skill => "skills",
                        }
                    ),
                    Style::default().fg(Color::DarkGray),
                )),
            ])
            .block(block);
            frame.render_widget(empty, area);
            return;
        }

        // Clamp selection
        if let Some(sel) = self.list_states[self.sub_tab].selected() {
            if sel >= list_len {
                self.list_states[self.sub_tab].select(Some(list_len - 1));
            }
        }

        // Now get the list reference for rendering
        let list = self.current_list();
        let items: Vec<ListItem> = list
            .iter()
            .enumerate()
            .map(|(i, entry)| {
                let is_selected = self.list_states[self.sub_tab].selected() == Some(i);

                let style = if is_selected {
                    Style::default()
                        .fg(entry_type.color())
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let desc_preview = entry
                    .description
                    .as_ref()
                    .map(|d| {
                        let truncated: String = d.chars().take(40).collect();
                        if d.len() > 40 {
                            format!(" - {}...", truncated)
                        } else {
                            format!(" - {}", truncated)
                        }
                    })
                    .unwrap_or_default();

                // Build spans with invocation count if > 0
                let mut spans = vec![
                    Span::styled(
                        format!(" {} ", entry_type.icon()),
                        Style::default().fg(entry_type.color()),
                    ),
                    Span::styled(entry.name.clone(), style),
                ];

                // Add invocation count if present
                if entry.invocation_count > 0 {
                    spans.push(Span::styled(
                        format!(" (× {})", entry.invocation_count),
                        Style::default().fg(Color::Yellow),
                    ));
                }

                spans.push(Span::styled(
                    desc_preview,
                    Style::default().fg(Color::DarkGray),
                ));

                ListItem::new(Line::from(spans))
            })
            .collect();

        let widget = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(widget, area, &mut self.list_states[self.sub_tab]);

        // Scrollbar for long lists
        if list_len > (area.height as usize - 2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state = ScrollbarState::new(list_len)
                .position(self.list_states[self.sub_tab].selected().unwrap_or(0));
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

    fn render_detail(&self, frame: &mut Frame, area: Rect) {
        let list = self.current_list();
        let selected = self.list_states[self.sub_tab]
            .selected()
            .and_then(|i| list.get(i));

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Detail ",
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(entry) = selected else {
            let empty = Paragraph::new("Select an item to see details")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        };

        // Read file content for preview
        let content_preview = std::fs::read_to_string(&entry.file_path)
            .map(|c| {
                // Skip frontmatter for preview
                let content = if c.starts_with("---") {
                    let parts: Vec<&str> = c.splitn(3, "---").collect();
                    if parts.len() >= 3 {
                        parts[2].trim().to_string()
                    } else {
                        c
                    }
                } else {
                    c
                };
                // Take first 500 chars
                content.chars().take(500).collect::<String>()
            })
            .unwrap_or_else(|_| "Unable to read file".to_string());

        let lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &entry.name,
                    Style::default().fg(entry.entry_type.color()).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
                Span::styled(entry.entry_type.label(), Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Path: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&entry.file_path, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Description:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                entry.description.as_deref().unwrap_or("No description"),
                Style::default().fg(Color::White),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Content preview:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                if content_preview.len() >= 500 {
                    format!("{}...", content_preview)
                } else {
                    content_preview
                },
                Style::default().fg(Color::Gray),
            )),
        ];

        let detail = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(detail, inner);
    }

    fn render_error_popup(&self, frame: &mut Frame, area: Rect) {
        use ratatui::widgets::Clear;

        // Center popup (40% width, 30% height)
        let popup_width = (area.width as f32 * 0.4).max(40.0) as u16;
        let popup_height = (area.height as f32 * 0.3).max(8.0) as u16;
        let popup_x = (area.width.saturating_sub(popup_width)) / 2;
        let popup_y = (area.height.saturating_sub(popup_height)) / 2;

        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
            width: popup_width,
            height: popup_height,
        };

        // Clear background
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red))
            .title(Span::styled(
                " Error ",
                Style::default().fg(Color::Red).bold(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");

        let lines = vec![
            Line::from(Span::styled(error_text, Style::default().fg(Color::White))),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }
}
