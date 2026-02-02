//! Hooks tab - View hooks by event type

use ccboard_core::models::{HookDefinition, HookGroup, Settings};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
};
use std::collections::HashMap;

/// Hooks tab state
pub struct HooksTab {
    /// Selected event type index
    event_state: ListState,
    /// Selected hook index within event
    hook_state: ListState,
    /// Focus: events (0) or hooks (1)
    focus: usize,
    /// Cached event names (sorted)
    event_names: Vec<String>,
    /// Error message to display
    error_message: Option<String>,
}

impl Default for HooksTab {
    fn default() -> Self {
        Self::new()
    }
}

impl HooksTab {
    pub fn new() -> Self {
        let mut event_state = ListState::default();
        event_state.select(Some(0));
        let mut hook_state = ListState::default();
        hook_state.select(Some(0));

        Self {
            event_state,
            hook_state,
            focus: 0,
            event_names: Vec::new(),
            error_message: None,
        }
    }

    /// Handle key input
    pub fn handle_key(
        &mut self,
        key: crossterm::event::KeyCode,
        hooks_map: &HashMap<String, Vec<HookGroup>>,
    ) {
        use crossterm::event::KeyCode;

        match key {
            KeyCode::Left | KeyCode::Char('h') => {
                self.focus = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.focus = 1;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focus == 0 {
                    self.move_event_selection(-1);
                    self.hook_state.select(Some(0));
                } else {
                    self.move_hook_selection(-1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focus == 0 {
                    self.move_event_selection(1);
                    self.hook_state.select(Some(0));
                } else {
                    self.move_hook_selection(1);
                }
            }
            KeyCode::Char('e') => {
                // Open selected hook file in editor
                if self.focus == 1 {
                    if let Some(hook) = self.get_selected_hook(hooks_map) {
                        if let Some(ref path) = hook.file_path {
                            if let Err(e) = crate::editor::open_in_editor(path) {
                                self.error_message = Some(format!("Failed to open editor: {}", e));
                            }
                        } else {
                            self.error_message =
                                Some("No file path available for this hook".to_string());
                        }
                    }
                }
            }
            KeyCode::Char('o') => {
                // Reveal hook file in file manager
                if self.focus == 1 {
                    if let Some(hook) = self.get_selected_hook(hooks_map) {
                        if let Some(ref path) = hook.file_path {
                            if let Err(e) = crate::editor::reveal_in_file_manager(path) {
                                self.error_message =
                                    Some(format!("Failed to open file manager: {}", e));
                            }
                        } else {
                            self.error_message =
                                Some("No file path available for this hook".to_string());
                        }
                    }
                }
            }
            KeyCode::Esc => {
                if self.error_message.is_some() {
                    self.error_message = None;
                }
            }
            KeyCode::PageUp => {
                // Jump up by 10 items
                if self.focus == 0 {
                    self.move_event_selection(-10);
                    self.hook_state.select(Some(0));
                } else {
                    self.move_hook_selection(-10);
                }
            }
            KeyCode::PageDown => {
                // Jump down by 10 items
                if self.focus == 0 {
                    self.move_event_selection(10);
                    self.hook_state.select(Some(0));
                } else {
                    self.move_hook_selection(10);
                }
            }
            _ => {}
        }
    }

    fn move_event_selection(&mut self, delta: i32) {
        if self.event_names.is_empty() {
            return;
        }
        let current = self.event_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).clamp(0, self.event_names.len() as i32 - 1) as usize;
        self.event_state.select(Some(new_idx));
    }

    fn move_hook_selection(&mut self, delta: i32) {
        let current = self.hook_state.selected().unwrap_or(0) as i32;
        let new_idx = (current + delta).max(0) as usize;
        self.hook_state.select(Some(new_idx));
    }

    fn get_selected_hook<'a>(
        &self,
        hooks_map: &'a HashMap<String, Vec<HookGroup>>,
    ) -> Option<&'a HookDefinition> {
        let event_idx = self.event_state.selected()?;
        let event_name = self.event_names.get(event_idx)?;
        let groups = hooks_map.get(event_name)?;

        // Flatten all hooks from all groups
        let all_hooks: Vec<&HookDefinition> = groups.iter().flat_map(|g| &g.hooks).collect();

        let hook_idx = self.hook_state.selected()?;
        all_hooks.get(hook_idx).copied()
    }

    /// Render the hooks tab
    pub fn render(&mut self, frame: &mut Frame, area: Rect, settings: &Settings) {
        let hooks = settings.hooks.as_ref();

        // Update event names cache
        self.event_names = hooks
            .map(|h| {
                let mut names: Vec<_> = h.keys().cloned().collect();
                names.sort();
                names
            })
            .unwrap_or_default();

        // Layout: event list | hook details
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        self.render_events(frame, chunks[0], hooks);

        // Get hooks for selected event
        let selected_event = self
            .event_state
            .selected()
            .and_then(|i| self.event_names.get(i))
            .cloned();

        let hook_groups = selected_event
            .as_ref()
            .and_then(|e| hooks.and_then(|h| h.get(e)))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        self.render_hook_details(frame, chunks[1], &selected_event, hook_groups);

        // Render error popup if present
        if self.error_message.is_some() {
            self.render_error_popup(frame, area);
        }
    }

    fn render_events(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        hooks: Option<&HashMap<String, Vec<HookGroup>>>,
    ) {
        let is_focused = self.focus == 0;
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" Events ({}) ", self.event_names.len()),
                Style::default().fg(Color::White).bold(),
            ));

        if self.event_names.is_empty() {
            let empty = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No hooks configured",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Add hooks in settings.json:",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(Span::styled(
                    "  \"hooks\": {",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(Span::styled(
                    "    \"PreToolUse\": [...]",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(Span::styled("  }", Style::default().fg(Color::Yellow))),
            ])
            .block(block);
            frame.render_widget(empty, area);
            return;
        }

        let items: Vec<ListItem> = self
            .event_names
            .iter()
            .enumerate()
            .map(|(i, event)| {
                let is_selected = self.event_state.selected() == Some(i);
                let hook_count: usize = hooks
                    .and_then(|h| h.get(event))
                    .map(|groups| groups.iter().map(|g| g.hooks.len()).sum())
                    .unwrap_or(0);

                let (icon, color) = Self::event_style(event);
                let style = if is_selected && is_focused {
                    Style::default().fg(color).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::Gray)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", icon), Style::default().fg(color)),
                    Span::styled(event.clone(), style),
                    Span::styled(
                        format!(" ({})", hook_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_stateful_widget(list, area, &mut self.event_state);

        // Scrollbar for long event lists
        let event_count = self.event_names.len();
        if event_count > (area.height as usize - 2) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state =
                ScrollbarState::new(event_count).position(self.event_state.selected().unwrap_or(0));
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

    fn render_hook_details(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        event: &Option<String>,
        groups: &[HookGroup],
    ) {
        let is_focused = self.focus == 1;
        let border_color = if is_focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let title = event
            .as_ref()
            .map(|e| format!(" {} Hooks ", e))
            .unwrap_or_else(|| " Hook Details ".to_string());

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                title,
                Style::default().fg(Color::White).bold(),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if groups.is_empty() {
            let empty = Paragraph::new("Select an event to see hooks")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, inner);
            return;
        }

        // Flatten all hooks with their group context
        let mut all_hooks: Vec<(&Option<String>, &HookDefinition)> = Vec::new();
        for group in groups {
            for hook in &group.hooks {
                all_hooks.push((&group.matcher, hook));
            }
        }

        // Clamp selection
        if let Some(sel) = self.hook_state.selected() {
            if sel >= all_hooks.len() && !all_hooks.is_empty() {
                self.hook_state.select(Some(all_hooks.len() - 1));
            }
        }

        let items: Vec<ListItem> = all_hooks
            .iter()
            .enumerate()
            .map(|(i, (matcher, hook))| {
                let is_selected = self.hook_state.selected() == Some(i);

                let mut lines = vec![Line::from(vec![
                    Span::styled(
                        if is_selected { "▶ " } else { "  " },
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled("$ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        hook.command.clone(),
                        if is_selected {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    ),
                ])];

                // Show matcher if present
                if let Some(m) = matcher {
                    lines.push(Line::from(vec![
                        Span::styled("    match: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(m.clone(), Style::default().fg(Color::Yellow)),
                    ]));
                }

                // Show file path if present
                if let Some(ref path) = hook.file_path {
                    lines.push(Line::from(vec![
                        Span::styled("    file: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            path.display().to_string(),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }

                // Show async/timeout if present
                let mut attrs = Vec::new();
                if hook.r#async == Some(true) {
                    attrs.push("async".to_string());
                }
                if let Some(timeout) = hook.timeout {
                    attrs.push(format!("timeout: {}s", timeout));
                }
                if !attrs.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("    [{}]", attrs.join(", ")),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                lines.push(Line::from("")); // spacing

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items);
        frame.render_stateful_widget(list, inner, &mut self.hook_state);

        // Scrollbar for long hook lists
        let hook_count = all_hooks.len();
        if hook_count > (inner.height as usize) {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None);
            let mut scrollbar_state =
                ScrollbarState::new(hook_count).position(self.hook_state.selected().unwrap_or(0));
            frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
        }
    }

    fn event_style(event: &str) -> (&'static str, Color) {
        match event {
            "PreToolUse" => ("⚡", Color::Yellow),
            "PostToolUse" => ("✓", Color::Green),
            "PrePromptSubmit" | "PreSubmit" => ("→", Color::Cyan),
            "PostPromptSubmit" | "PostSubmit" => ("←", Color::Blue),
            "Notification" => ("!", Color::Magenta),
            "Stop" => ("■", Color::Red),
            _ => ("●", Color::Gray),
        }
    }

    fn render_error_popup(&self, frame: &mut Frame, area: Rect) {
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
