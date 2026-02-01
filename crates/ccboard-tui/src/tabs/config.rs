//! Config tab - 3-column view with global/project/local + merged result

use ccboard_core::models::{MergedConfig, Settings};
use ccboard_core::parsers::{McpConfig, Rules};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

/// Config tab state
pub struct ConfigTab {
    /// Currently focused column (0=global, 1=project, 2=local, 3=merged)
    focus: usize,
    /// Scroll state for each column
    scroll_states: [ListState; 4],
}

impl Default for ConfigTab {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigTab {
    pub fn new() -> Self {
        Self {
            focus: 3, // Start with merged view
            scroll_states: Default::default(),
        }
    }

    /// Handle key input for this tab
    pub fn handle_key(&mut self, key: crossterm::event::KeyCode) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = self.focus.saturating_sub(1);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = (self.focus + 1).min(3);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let state = &mut self.scroll_states[self.focus];
                let current = state.selected().unwrap_or(0);
                state.select(Some(current.saturating_sub(1)));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let state = &mut self.scroll_states[self.focus];
                let current = state.selected().unwrap_or(0);
                state.select(Some(current + 1));
            }
            _ => {}
        }
    }

    /// Render the config tab
    pub fn render(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        config: &MergedConfig,
        mcp_config: Option<&McpConfig>,
        rules: &Rules,
    ) {
        // 4-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(area);

        self.render_config_column(
            frame,
            chunks[0],
            "Global",
            config.global.as_ref(),
            0,
            None,
            rules,
        );
        self.render_config_column(
            frame,
            chunks[1],
            "Project",
            config.project.as_ref(),
            1,
            None,
            rules,
        );
        self.render_config_column(
            frame,
            chunks[2],
            "Local",
            config.local.as_ref(),
            2,
            None,
            rules,
        );
        self.render_config_column(
            frame,
            chunks[3],
            "Merged",
            Some(&config.merged),
            3,
            mcp_config,
            rules,
        );
    }

    fn render_config_column(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        settings: Option<&Settings>,
        col_index: usize,
        mcp_config: Option<&McpConfig>,
        rules: &Rules,
    ) {
        let is_focused = self.focus == col_index;
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let source_indicator = match col_index {
            0 => "~/.claude/settings.json",
            1 => ".claude/settings.json",
            2 => ".claude/settings.local.json",
            3 => "final result",
            _ => "",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" {} ", title),
                Style::default()
                    .fg(if is_focused {
                        Color::Cyan
                    } else {
                        Color::White
                    })
                    .bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(settings) = settings else {
            let empty = Paragraph::new(vec![
                Line::from(Span::styled(
                    "Not configured",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    source_indicator,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                )),
            ]);
            frame.render_widget(empty, inner);
            return;
        };

        let items = self.settings_to_items(settings, col_index == 3, mcp_config, rules);

        // Clamp scroll state
        if let Some(sel) = self.scroll_states[col_index].selected() {
            if sel >= items.len() && !items.is_empty() {
                self.scroll_states[col_index].select(Some(items.len() - 1));
            }
        }

        let list = List::new(items).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

        frame.render_stateful_widget(list, inner, &mut self.scroll_states[col_index]);
    }

    fn settings_to_items(
        &self,
        settings: &Settings,
        is_merged: bool,
        mcp_config: Option<&McpConfig>,
        rules: &Rules,
    ) -> Vec<ListItem<'static>> {
        let mut items = Vec::new();

        // Model
        if let Some(ref model) = settings.model {
            items.push(self.make_item("model", model, Color::Cyan));
        }

        // Theme
        if let Some(ref theme) = settings.theme {
            items.push(self.make_item("theme", theme, Color::Magenta));
        }

        // API Key (masked)
        if settings.api_key.is_some() {
            items.push(self.make_item("apiKey", "••••••••", Color::Red));
        }

        // Custom instructions
        if let Some(ref instructions) = settings.custom_instructions {
            let preview: String = instructions.chars().take(30).collect();
            let display = if instructions.len() > 30 {
                format!("{}...", preview)
            } else {
                preview
            };
            items.push(self.make_item("customInstructions", &display, Color::Yellow));
        }

        // Permissions section
        if let Some(ref perms) = settings.permissions {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Permissions ───",
                Style::default().fg(Color::DarkGray),
            ))));

            if let Some(ref allow) = perms.allow {
                items.push(self.make_item(
                    "  allow",
                    &format!("[{}]", allow.join(", ")),
                    Color::Green,
                ));
            }
            if let Some(ref deny) = perms.deny {
                items.push(self.make_item("  deny", &format!("[{}]", deny.join(", ")), Color::Red));
            }
            if let Some(ref allow_bash) = perms.allow_bash {
                items.push(self.make_item(
                    "  allowBash",
                    &format!("{} rules", allow_bash.len()),
                    Color::Green,
                ));
            }
            if let Some(ref deny_bash) = perms.deny_bash {
                items.push(self.make_item(
                    "  denyBash",
                    &format!("{} rules", deny_bash.len()),
                    Color::Red,
                ));
            }
            if let Some(auto) = perms.auto_approve {
                items.push(self.make_item(
                    "  autoApprove",
                    if auto { "true" } else { "false" },
                    if auto { Color::Green } else { Color::Yellow },
                ));
            }
            if let Some(trust) = perms.trust_project {
                items.push(self.make_item(
                    "  trustProject",
                    if trust { "true" } else { "false" },
                    if trust { Color::Green } else { Color::Yellow },
                ));
            }
        }

        // Hooks section
        if let Some(ref hooks) = settings.hooks {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Hooks ───",
                Style::default().fg(Color::DarkGray),
            ))));

            for (event, groups) in hooks {
                let hook_count: usize = groups.iter().map(|g| g.hooks.len()).sum();
                items.push(self.make_item(
                    &format!("  {}", event),
                    &format!("{} hooks", hook_count),
                    Color::Yellow,
                ));
            }
        }

        // Env section
        if let Some(ref env) = settings.env {
            if !env.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "─── Environment ───",
                    Style::default().fg(Color::DarkGray),
                ))));

                for (key, value) in env.iter().take(5) {
                    let display_val: String = value.chars().take(20).collect();
                    items.push(self.make_item(&format!("  {}", key), &display_val, Color::Blue));
                }
                if env.len() > 5 {
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("  ... and {} more", env.len() - 5),
                        Style::default().fg(Color::DarkGray),
                    ))));
                }
            }
        }

        // Plugins section
        if let Some(ref plugins) = settings.enabled_plugins {
            if !plugins.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "─── Plugins ───",
                    Style::default().fg(Color::DarkGray),
                ))));

                for (name, enabled) in plugins {
                    let color = if *enabled { Color::Green } else { Color::Red };
                    let status = if *enabled { "enabled" } else { "disabled" };
                    items.push(self.make_item(&format!("  {}", name), status, color));
                }
            }
        }

        // MCP Servers section (only in merged column)
        if is_merged {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── MCP Servers ───",
                Style::default().fg(Color::DarkGray),
            ))));

            if let Some(mcp) = mcp_config {
                if mcp.servers.is_empty() {
                    items.push(ListItem::new(Line::from(Span::styled(
                        "  (No MCP servers configured)",
                        Style::default().fg(Color::DarkGray),
                    ))));
                } else {
                    for (name, server) in &mcp.servers {
                        let cmd_display = format!("{} {}", server.command, server.args.join(" "));
                        // Truncate if too long
                        let cmd_short: String = if cmd_display.len() > 40 {
                            cmd_display.chars().take(37).collect::<String>() + "..."
                        } else {
                            cmd_display
                        };

                        items.push(ListItem::new(Line::from(vec![
                            Span::styled("  ● ", Style::default().fg(Color::Green)),
                            Span::styled(
                                format!("{}: ", name),
                                Style::default().fg(Color::Cyan).bold(),
                            ),
                        ])));
                        items.push(ListItem::new(Line::from(Span::styled(
                            format!("    {}", cmd_short),
                            Style::default().fg(Color::DarkGray),
                        ))));
                    }
                }
            } else {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  (No MCP config found)",
                    Style::default().fg(Color::DarkGray),
                ))));
            }
        }

        // Extra fields
        if !settings.extra.is_empty() && is_merged {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Extra ───",
                Style::default().fg(Color::DarkGray),
            ))));

            for (key, _value) in settings.extra.iter().take(5) {
                items.push(self.make_item(&format!("  {}", key), "...", Color::DarkGray));
            }
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Empty configuration",
                Style::default().fg(Color::DarkGray),
            ))));
        }

        items
    }

    fn make_item(&self, key: &str, value: &str, value_color: Color) -> ListItem<'static> {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{}: ", key), Style::default().fg(Color::DarkGray)),
            Span::styled(value.to_string(), Style::default().fg(value_color)),
        ]))
    }
}
