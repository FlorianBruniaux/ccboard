//! Config tab - 3-column view with global/project/local + merged result

use crate::theme::Palette;
use ccboard_core::models::config::ColorScheme;
use ccboard_core::models::{MergedConfig, Settings};
use ccboard_core::parsers::{McpConfig, Rules};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

/// Config tab state
pub struct ConfigTab {
    /// Currently focused column (0=global, 1=project, 2=local, 3=merged)
    focus: usize,
    /// Scroll state for each column
    scroll_states: [ListState; 4],
    /// Error message to display
    error_message: Option<String>,
    /// Claude home directory (for file paths)
    claude_home: Option<std::path::PathBuf>,
    /// Project directory (for file paths)
    project_path: Option<std::path::PathBuf>,
    /// Show MCP detail modal
    show_mcp_detail: bool,
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
            error_message: None,
            claude_home: None,
            project_path: None,
            show_mcp_detail: false,
        }
    }

    /// Initialize with paths (called from UI init)
    pub fn init(&mut self, claude_home: &std::path::Path, project_path: Option<&std::path::Path>) {
        self.claude_home = Some(claude_home.to_path_buf());
        self.project_path = project_path.map(|p| p.to_path_buf());
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
            KeyCode::Char('e') => {
                if self.show_mcp_detail {
                    // In MCP modal: edit claude_desktop_config.json
                    if let Some(ref claude_home) = self.claude_home {
                        let config_path = claude_home.join("claude_desktop_config.json");
                        if let Err(e) = crate::editor::open_in_editor(&config_path) {
                            self.error_message = Some(format!("Failed to open editor: {}", e));
                        }
                        self.show_mcp_detail = false; // Close modal after opening editor
                    } else {
                        self.error_message = Some("Claude home directory not set".to_string());
                    }
                } else {
                    // Normal mode: Open config file based on focused column
                    if let Some(path) = self.get_focused_file_path() {
                        if let Err(e) = crate::editor::open_in_editor(&path) {
                            self.error_message = Some(format!("Failed to open editor: {}", e));
                        }
                    } else {
                        self.error_message =
                            Some("No config file available for this column".to_string());
                    }
                }
            }
            KeyCode::Char('o') => {
                // Reveal config file in file manager
                if let Some(path) = self.get_focused_file_path() {
                    if let Err(e) = crate::editor::reveal_in_file_manager(&path) {
                        self.error_message = Some(format!("Failed to open file manager: {}", e));
                    }
                } else {
                    self.error_message =
                        Some("No config file available for this column".to_string());
                }
            }
            KeyCode::Char('m') => {
                // Show MCP detail modal (only in merged column)
                if self.focus == 3 {
                    self.show_mcp_detail = true;
                }
            }
            KeyCode::Esc => {
                if self.show_mcp_detail {
                    self.show_mcp_detail = false;
                } else if self.error_message.is_some() {
                    self.error_message = None;
                }
            }
            KeyCode::PageUp => {
                // Jump up by 10 items
                let state = &mut self.scroll_states[self.focus];
                let current = state.selected().unwrap_or(0);
                state.select(Some(current.saturating_sub(10)));
            }
            KeyCode::PageDown => {
                // Jump down by 10 items
                let state = &mut self.scroll_states[self.focus];
                let current = state.selected().unwrap_or(0);
                state.select(Some(current + 10));
            }
            _ => {}
        }
    }

    /// Get the file path for the currently focused column
    fn get_focused_file_path(&self) -> Option<std::path::PathBuf> {
        match self.focus {
            0 => {
                // Global: ~/.claude/settings.json
                self.claude_home.as_ref().map(|p| p.join("settings.json"))
            }
            1 => {
                // Project: .claude/settings.json
                self.project_path
                    .as_ref()
                    .map(|p| p.join(".claude/settings.json"))
            }
            2 => {
                // Local: .claude/settings.local.json
                self.project_path
                    .as_ref()
                    .map(|p| p.join(".claude/settings.local.json"))
            }
            3 => {
                // Merged: no single file, show error
                None
            }
            _ => None,
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
        scheme: ColorScheme,
    ) {
        let p = Palette::new(scheme);

        // Layout: [Help header (2 lines), Content columns]
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Help text
                Constraint::Min(0),    // Content
            ])
            .split(area);

        // Render help header
        let help_text = "Claude Code uses cascading configuration: Local > Project > Global. Merged shows final active configuration.";
        let help = Paragraph::new(help_text)
            .style(Style::default().fg(p.muted))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(help, main_chunks[0]);

        // 4-column layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(main_chunks[1]);

        self.render_config_column(
            frame,
            chunks[0],
            "Global (All projects)",
            config.global.as_ref(),
            0,
            None,
            rules,
            &p,
        );
        self.render_config_column(
            frame,
            chunks[1],
            "Project (This repo)",
            config.project.as_ref(),
            1,
            None,
            rules,
            &p,
        );
        self.render_config_column(
            frame,
            chunks[2],
            "Local (You only)",
            config.local.as_ref(),
            2,
            None,
            rules,
            &p,
        );
        self.render_config_column(
            frame,
            chunks[3],
            "Merged (Active)",
            Some(&config.merged),
            3,
            mcp_config,
            rules,
            &p,
        );

        // Render MCP detail modal if requested
        if self.show_mcp_detail {
            self.render_mcp_detail_modal(frame, area, mcp_config, &p);
        }

        // Render error popup if present
        if self.error_message.is_some() {
            self.render_error_popup(frame, area, &p);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_config_column(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
        settings: Option<&Settings>,
        col_index: usize,
        mcp_config: Option<&McpConfig>,
        rules: &Rules,
        p: &Palette,
    ) {
        let is_focused = self.focus == col_index;
        let border_color = if is_focused { p.focus } else { p.border };

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
                    .fg(if is_focused { p.focus } else { p.fg })
                    .bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let Some(settings) = settings else {
            let message = if col_index == 3 {
                "Using defaults"
            } else {
                "Using defaults ✓"
            };

            let empty = Paragraph::new(vec![
                Line::from(Span::styled(message, Style::default().fg(p.success))),
                Line::from(""),
                Line::from(Span::styled(
                    source_indicator,
                    Style::default().fg(p.muted).add_modifier(Modifier::ITALIC),
                )),
            ]);
            frame.render_widget(empty, inner);
            return;
        };

        let items = self.settings_to_items(settings, col_index == 3, mcp_config, rules, p);

        // Clamp scroll state
        if let Some(sel) = self.scroll_states[col_index].selected() {
            if sel >= items.len() && !items.is_empty() {
                self.scroll_states[col_index].select(Some(items.len() - 1));
            }
        }

        let list = List::new(items)
            .highlight_style(Style::default().bg(p.muted).add_modifier(Modifier::BOLD));

        frame.render_stateful_widget(list, inner, &mut self.scroll_states[col_index]);
    }

    fn settings_to_items(
        &self,
        settings: &Settings,
        is_merged: bool,
        mcp_config: Option<&McpConfig>,
        rules: &Rules,
        p: &Palette,
    ) -> Vec<ListItem<'static>> {
        let mut items = Vec::new();

        // Model
        if let Some(ref model) = settings.model {
            items.push(self.make_item("model", model, p.focus, p));
        }

        // Theme
        if let Some(ref theme) = settings.theme {
            items.push(self.make_item("theme", theme, p.important, p));
        }

        // API Key (masked)
        if settings.api_key.is_some() {
            items.push(self.make_item("apiKey", "••••••••", p.error, p));
        }

        // Custom instructions
        if let Some(ref instructions) = settings.custom_instructions {
            let preview: String = instructions.chars().take(30).collect();
            let display = if instructions.len() > 30 {
                format!("{}...", preview)
            } else {
                preview
            };
            items.push(self.make_item("customInstructions", &display, p.warning, p));
        }

        // Permissions section
        if let Some(ref perms) = settings.permissions {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Permissions ───",
                Style::default().fg(p.muted),
            ))));

            if let Some(ref allow) = perms.allow {
                items.push(self.make_item(
                    "  allow",
                    &format!("[{}]", allow.join(", ")),
                    p.success,
                    p,
                ));
            }
            if let Some(ref deny) = perms.deny {
                items.push(self.make_item("  deny", &format!("[{}]", deny.join(", ")), p.error, p));
            }
            if let Some(ref allow_bash) = perms.allow_bash {
                items.push(self.make_item(
                    "  allowBash",
                    &format!("{} rules", allow_bash.len()),
                    p.success,
                    p,
                ));
            }
            if let Some(ref deny_bash) = perms.deny_bash {
                items.push(self.make_item(
                    "  denyBash",
                    &format!("{} rules", deny_bash.len()),
                    p.error,
                    p,
                ));
            }
            if let Some(auto) = perms.auto_approve {
                items.push(self.make_item(
                    "  autoApprove",
                    if auto { "true" } else { "false" },
                    if auto { p.success } else { p.warning },
                    p,
                ));
            }
            if let Some(trust) = perms.trust_project {
                items.push(self.make_item(
                    "  trustProject",
                    if trust { "true" } else { "false" },
                    if trust { p.success } else { p.warning },
                    p,
                ));
            }
        }

        // Hooks section
        if let Some(ref hooks) = settings.hooks {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Hooks ───",
                Style::default().fg(p.muted),
            ))));

            for (event, groups) in hooks {
                let hook_count: usize = groups.iter().map(|g| g.hooks.len()).sum();
                items.push(self.make_item(
                    &format!("  {}", event),
                    &format!("{} hooks", hook_count),
                    p.warning,
                    p,
                ));
            }
        }

        // Env section
        if let Some(ref env) = settings.env {
            if !env.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "─── Environment ───",
                    Style::default().fg(p.muted),
                ))));

                for (key, value) in env.iter().take(5) {
                    let display_val: String = value.chars().take(20).collect();
                    items.push(self.make_item(&format!("  {}", key), &display_val, Color::Blue, p));
                }
                if env.len() > 5 {
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("  ... and {} more", env.len() - 5),
                        Style::default().fg(p.muted),
                    ))));
                }
            }
        }

        // Plugins section
        if let Some(ref plugins) = settings.enabled_plugins {
            if !plugins.is_empty() {
                items.push(ListItem::new(Line::from(Span::styled(
                    "─── Plugins ───",
                    Style::default().fg(p.muted),
                ))));

                for (name, enabled) in plugins {
                    let color = if *enabled { p.success } else { p.error };
                    let status = if *enabled { "enabled" } else { "disabled" };
                    items.push(self.make_item(&format!("  {}", name), status, color, p));
                }
            }
        }

        // MCP Servers section (only in merged column)
        if is_merged {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── MCP Servers ───",
                Style::default().fg(p.muted),
            ))));

            if let Some(mcp) = mcp_config {
                if mcp.servers.is_empty() {
                    items.push(ListItem::new(Line::from(Span::styled(
                        "  (No MCP servers configured)",
                        Style::default().fg(p.muted),
                    ))));
                } else {
                    for (name, server) in &mcp.servers {
                        let cmd_display = server.display_command();
                        let cmd_short: String = if cmd_display.len() > 60 {
                            cmd_display.chars().take(57).collect::<String>() + "..."
                        } else {
                            cmd_display
                        };

                        items.push(ListItem::new(Line::from(vec![
                            Span::styled("  ● ", Style::default().fg(p.success)),
                            Span::styled(
                                format!("{} (configured)", name),
                                Style::default().fg(p.focus).bold(),
                            ),
                        ])));

                        items.push(ListItem::new(Line::from(Span::styled(
                            format!("    {}", cmd_short),
                            Style::default().fg(p.fg),
                        ))));

                        let env_info = if server.env.is_empty() {
                            "Env: (none)".to_string()
                        } else {
                            format!("Env: {} vars", server.env.len())
                        };
                        items.push(ListItem::new(Line::from(Span::styled(
                            format!("    {}", env_info),
                            Style::default().fg(p.muted),
                        ))));
                    }
                }
            } else {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  (No MCP config found)",
                    Style::default().fg(p.muted),
                ))));
            }
        }

        // Rules section (only in merged column)
        if is_merged {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Rules (CLAUDE.md) ───",
                Style::default().fg(p.muted),
            ))));

            if let Some(ref global) = rules.global {
                let size_kb = global.size as f64 / 1024.0;
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  Global: ", Style::default().fg(p.muted)),
                    Span::styled(
                        format!("~/.claude/CLAUDE.md ({:.1}KB)", size_kb),
                        Style::default().fg(p.focus),
                    ),
                ])));

                let preview = Rules::preview(global, 3);
                for line in preview {
                    let display_line: String = if line.len() > 50 {
                        line.chars().take(47).collect::<String>() + "..."
                    } else {
                        line
                    };
                    items.push(ListItem::new(Line::from(Span::styled(
                        format!("    > {}", display_line),
                        Style::default().fg(p.muted),
                    ))));
                }
            } else {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  Global: (not found)",
                    Style::default().fg(p.muted),
                ))));
            }

            if let Some(ref project) = rules.project {
                let size_kb = project.size as f64 / 1024.0;
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("  Project: ", Style::default().fg(p.muted)),
                    Span::styled(
                        format!(".claude/CLAUDE.md ({:.1}KB)", size_kb),
                        Style::default().fg(p.important),
                    ),
                ])));
            } else {
                items.push(ListItem::new(Line::from(Span::styled(
                    "  Project: (not found)",
                    Style::default().fg(p.muted),
                ))));
            }
        }

        // Extra fields
        if !settings.extra.is_empty() && is_merged {
            items.push(ListItem::new(Line::from(Span::styled(
                "─── Extra ───",
                Style::default().fg(p.muted),
            ))));

            for (key, _value) in settings.extra.iter().take(5) {
                items.push(self.make_item(&format!("  {}", key), "...", p.muted, p));
            }
        }

        if items.is_empty() {
            items.push(ListItem::new(Line::from(Span::styled(
                "Empty configuration",
                Style::default().fg(p.muted),
            ))));
        }

        items
    }

    fn make_item(
        &self,
        key: &str,
        value: &str,
        value_color: Color,
        p: &Palette,
    ) -> ListItem<'static> {
        ListItem::new(Line::from(vec![
            Span::styled(format!("{}: ", key), Style::default().fg(p.muted)),
            Span::styled(value.to_string(), Style::default().fg(value_color)),
        ]))
    }

    fn render_mcp_detail_modal(
        &self,
        frame: &mut Frame,
        area: Rect,
        mcp_config: Option<&McpConfig>,
        p: &Palette,
    ) {
        // Center modal (70% width, 70% height)
        let modal_width = (area.width as f32 * 0.7).max(60.0) as u16;
        let modal_height = (area.height as f32 * 0.7).max(20.0) as u16;
        let modal_x = (area.width.saturating_sub(modal_width)) / 2;
        let modal_y = (area.height.saturating_sub(modal_height)) / 2;

        let modal_area = Rect {
            x: area.x + modal_x,
            y: area.y + modal_y,
            width: modal_width,
            height: modal_height,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.focus))
            .title(Span::styled(
                " MCP Servers Detail ",
                Style::default().fg(p.focus).bold(),
            ));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let mut lines = vec![];

        if let Some(mcp) = mcp_config {
            if mcp.servers.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No MCP servers configured",
                    Style::default().fg(p.muted),
                )));
            } else {
                let config_path = self
                    .claude_home
                    .as_ref()
                    .map(|ch| ch.join("claude_desktop_config.json"))
                    .map(|ch| ch.display().to_string())
                    .unwrap_or_else(|| "~/.claude/claude_desktop_config.json".to_string());

                lines.push(Line::from(Span::styled(
                    format!("Config: {}", config_path),
                    Style::default().fg(p.muted),
                )));
                lines.push(Line::from(""));

                for (name, server) in &mcp.servers {
                    lines.push(Line::from(Span::styled(
                        format!("● {}", name),
                        Style::default().fg(p.success).bold(),
                    )));

                    let full_cmd = server.display_command();
                    lines.push(Line::from(vec![
                        Span::styled("  Command: ", Style::default().fg(p.muted)),
                        Span::styled(full_cmd, Style::default().fg(p.fg)),
                    ]));

                    if server.env.is_empty() {
                        lines.push(Line::from(Span::styled(
                            "  Env: (none)",
                            Style::default().fg(p.muted),
                        )));
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!("  Env: {} variables", server.env.len()),
                            Style::default().fg(p.muted),
                        )));
                        for (key, value) in &server.env {
                            lines.push(Line::from(vec![
                                Span::styled("    ", Style::default()),
                                Span::styled(format!("{}=", key), Style::default().fg(p.warning)),
                                Span::styled(value, Style::default().fg(p.fg)),
                            ]));
                        }
                    }

                    lines.push(Line::from(""));
                }

                if !lines.is_empty() {
                    lines.pop();
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "MCP config not found",
                Style::default().fg(p.muted),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Expected at: ~/.claude/claude_desktop_config.json",
                Style::default().fg(p.muted),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[Esc: close | e: edit config]",
            Style::default().fg(p.muted),
        )));

        let paragraph = Paragraph::new(lines)
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((0, 0));

        frame.render_widget(paragraph, inner);
    }

    fn render_error_popup(&self, frame: &mut Frame, area: Rect, p: &Palette) {
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

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.error))
            .title(Span::styled(" Error ", Style::default().fg(p.error).bold()));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let error_text = self.error_message.as_deref().unwrap_or("Unknown error");

        let lines = vec![
            Line::from(Span::styled(error_text, Style::default().fg(p.fg))),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to close",
                Style::default().fg(p.muted),
            )),
        ];

        let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(paragraph, inner);
    }
}
