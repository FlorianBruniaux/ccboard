//! Help modal component for displaying keybindings

use crate::app::Tab;
use crate::keybindings::{KeyAction, KeyBindings};
use crate::theme::Palette;
use ccboard_core::models::config::ColorScheme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Help modal displaying keybindings
pub struct HelpModal {
    visible: bool,
}

impl Default for HelpModal {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpModal {
    pub fn new() -> Self {
        Self { visible: false }
    }

    /// Toggle help modal visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if modal is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Hide the modal
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Render the help modal as an overlay
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        active_tab: Tab,
        keybindings: &KeyBindings,
        scheme: ColorScheme,
    ) {
        if !self.visible {
            return;
        }

        // Calculate centered modal size
        let modal_width = 70;
        let modal_height = 24;

        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((area.height.saturating_sub(modal_height)) / 2),
                Constraint::Length(modal_height),
                Constraint::Min(0),
            ])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((area.width.saturating_sub(modal_width)) / 2),
                Constraint::Length(modal_width),
                Constraint::Min(0),
            ])
            .split(vertical[1]);

        let modal_area = horizontal[1];

        let p = Palette::new(scheme);

        // Clear the area behind the modal
        frame.render_widget(Clear, modal_area);

        // Create modal block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(p.focus))
            .title(Span::styled(
                " Help - Keybindings ",
                Style::default()
                    .fg(p.focus)
                    .add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center);

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Build help content
        let help_lines = self.build_help_content(active_tab, keybindings, &p);

        let help_text = Paragraph::new(help_lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        frame.render_widget(help_text, inner);
    }

    /// Build help content based on active tab
    fn build_help_content(&self, active_tab: Tab, keybindings: &KeyBindings, p: &Palette) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Global keybindings
        lines.push(Line::from(vec![Span::styled(
            "Global:",
            Style::default()
                .fg(p.warning)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        let focus_color = p.focus;
        // Helper function to add keybinding line
        let add_key_line =
            |lines: &mut Vec<Line<'static>>, action: KeyAction, keybindings: &KeyBindings| {
                if let Some(key_str) = keybindings.get_key_for_action(action) {
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {:12}", key_str),
                            Style::default().fg(focus_color),
                        ),
                        Span::raw(action.description()),
                    ]));
                }
            };

        // Add dynamic keybindings for global actions
        add_key_line(&mut lines, KeyAction::Quit, keybindings);
        add_key_line(&mut lines, KeyAction::ToggleHelp, keybindings);
        add_key_line(&mut lines, KeyAction::ShowCommandPalette, keybindings);
        add_key_line(&mut lines, KeyAction::Refresh, keybindings);
        add_key_line(&mut lines, KeyAction::ForceRefresh, keybindings);
        add_key_line(&mut lines, KeyAction::NextTab, keybindings);
        add_key_line(&mut lines, KeyAction::PrevTab, keybindings);
        add_key_line(&mut lines, KeyAction::ThemeToggle, keybindings);

        // Show tab jump shortcuts
        if let Some(key_str) = keybindings.get_key_for_action(KeyAction::JumpTab0) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {}-...      ", key_str.chars().next().unwrap_or('1')),
                    Style::default().fg(p.focus),
                ),
                Span::raw("Jump to tab (1=Dashboard, 2=Sessions, ...)"),
            ]));
        }

        lines.push(Line::from(""));

        // Tab-specific keybindings
        lines.push(Line::from(vec![Span::styled(
            format!("{}:", active_tab.name()),
            Style::default()
                .fg(p.warning)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        match active_tab {
            Tab::Dashboard => {
                lines.push(Line::from(vec![
                    Span::styled("  F5          ", Style::default().fg(focus_color)),
                    Span::raw("Refresh dashboard"),
                ]));
            }
            Tab::Sessions => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(focus_color)),
                    Span::raw("Navigate between projects and sessions"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(focus_color)),
                    Span::raw("View session details"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  /           ", Style::default().fg(focus_color)),
                    Span::raw("Search sessions"),
                ]));
            }
            Tab::Config => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(focus_color)),
                    Span::raw("Switch between columns"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(focus_color)),
                    Span::raw("Scroll configuration"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  e           ", Style::default().fg(focus_color)),
                    Span::raw("Edit configuration file"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  o           ", Style::default().fg(focus_color)),
                    Span::raw("Reveal file in finder"),
                ]));
            }
            Tab::Hooks => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(focus_color)),
                    Span::raw("Navigate hook types"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(focus_color)),
                    Span::raw("Select hook script"),
                ]));
            }
            Tab::Agents => {
                lines.push(Line::from(vec![
                    Span::styled("  Tab         ", Style::default().fg(focus_color)),
                    Span::raw("Switch between Agents/Commands/Skills"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(focus_color)),
                    Span::raw("View details"),
                ]));
            }
            Tab::Costs => {
                lines.push(Line::from(vec![
                    Span::styled("  Tab/←/→/h/l ", Style::default().fg(focus_color)),
                    Span::raw("Switch between Overview/Billing/Models"),
                ]));
            }
            Tab::History => {
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(focus_color)),
                    Span::raw("View session details"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  /           ", Style::default().fg(focus_color)),
                    Span::raw("Search history"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Tab         ", Style::default().fg(focus_color)),
                    Span::raw("Toggle stats view"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  c           ", Style::default().fg(focus_color)),
                    Span::raw("Clear search"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  x           ", Style::default().fg(focus_color)),
                    Span::raw("Export filtered sessions (CSV/JSON)"),
                ]));
            }
            Tab::Mcp => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(focus_color)),
                    Span::raw("Focus server/detail panes"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(focus_color)),
                    Span::raw("Select server"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  y           ", Style::default().fg(focus_color)),
                    Span::raw("Copy command to clipboard"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  e           ", Style::default().fg(focus_color)),
                    Span::raw("Edit MCP config"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  o           ", Style::default().fg(focus_color)),
                    Span::raw("Reveal config file"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  r           ", Style::default().fg(focus_color)),
                    Span::raw("Refresh server status"),
                ]));
            }
            Tab::Analytics => {
                lines.push(Line::from(vec![
                    Span::styled("  F1-F4       ", Style::default().fg(focus_color)),
                    Span::raw("Select period (7d/30d/90d/All)"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ←→ or h/l   ", Style::default().fg(focus_color)),
                    Span::raw("Switch between sub-views"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  j/k or ↑/↓  ", Style::default().fg(focus_color)),
                    Span::raw("Scroll insights list"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  r           ", Style::default().fg(focus_color)),
                    Span::raw("Recompute analytics"),
                ]));
            }
            Tab::Plugins => {
                lines.push(Line::from(vec![
                    Span::styled("  Tab         ", Style::default().fg(focus_color)),
                    Span::raw("Cycle between columns"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  j/k or ↑/↓  ", Style::default().fg(focus_color)),
                    Span::raw("Navigate within column"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  s           ", Style::default().fg(focus_color)),
                    Span::raw("Toggle sort mode"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  r           ", Style::default().fg(focus_color)),
                    Span::raw("Refresh analytics"),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(p.muted)),
            Span::styled(
                "?",
                Style::default()
                    .fg(focus_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(p.muted)),
            Span::styled(
                "ESC",
                Style::default()
                    .fg(focus_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close", Style::default().fg(p.muted)),
        ]));

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_modal_toggle() {
        let mut modal = HelpModal::new();
        assert!(!modal.is_visible());

        modal.toggle();
        assert!(modal.is_visible());

        modal.toggle();
        assert!(!modal.is_visible());
    }

    #[test]
    fn test_help_modal_hide() {
        let mut modal = HelpModal::new();
        modal.toggle();
        assert!(modal.is_visible());

        modal.hide();
        assert!(!modal.is_visible());
    }
}
