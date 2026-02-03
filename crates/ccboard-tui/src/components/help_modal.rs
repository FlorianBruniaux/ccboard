//! Help modal component for displaying keybindings

use crate::app::Tab;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
    pub fn render(&self, frame: &mut Frame, area: Rect, active_tab: Tab) {
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

        // Clear the area behind the modal
        frame.render_widget(Clear, modal_area);

        // Create modal block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " Help - Keybindings ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .title_alignment(Alignment::Center);

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Build help content
        let help_lines = self.build_help_content(active_tab);

        let help_text = Paragraph::new(help_lines)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left);

        frame.render_widget(help_text, inner);
    }

    /// Build help content based on active tab
    fn build_help_content(&self, active_tab: Tab) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        // Global keybindings
        lines.push(Line::from(vec![Span::styled(
            "Global:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("  q           ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle this help"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  :           ", Style::default().fg(Color::Cyan)),
            Span::raw("Command palette"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  F5          ", Style::default().fg(Color::Cyan)),
            Span::raw("Refresh data"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Tab         ", Style::default().fg(Color::Cyan)),
            Span::raw("Next tab"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  Shift+Tab   ", Style::default().fg(Color::Cyan)),
            Span::raw("Previous tab"),
        ]));
        lines.push(Line::from(vec![
            Span::styled("  1-8         ", Style::default().fg(Color::Cyan)),
            Span::raw("Jump to tab (1=Dashboard, 2=Sessions, ...)"),
        ]));
        lines.push(Line::from(""));

        // Tab-specific keybindings
        lines.push(Line::from(vec![Span::styled(
            format!("{}:", active_tab.name()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]));
        lines.push(Line::from(""));

        match active_tab {
            Tab::Dashboard => {
                lines.push(Line::from(vec![
                    Span::styled("  F5          ", Style::default().fg(Color::Cyan)),
                    Span::raw("Refresh dashboard"),
                ]));
            }
            Tab::Sessions => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Navigate between projects and sessions"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(Color::Cyan)),
                    Span::raw("View session details"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  /           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Search sessions"),
                ]));
            }
            Tab::Config => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Switch between columns"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Scroll configuration"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  e           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Edit configuration file"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  o           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Reveal file in finder"),
                ]));
            }
            Tab::Hooks => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Navigate hook types"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Select hook script"),
                ]));
            }
            Tab::Agents => {
                lines.push(Line::from(vec![
                    Span::styled("  Tab         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Switch between Agents/Commands/Skills"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(Color::Cyan)),
                    Span::raw("View details"),
                ]));
            }
            Tab::Costs => {
                lines.push(Line::from(vec![
                    Span::styled("  Tab/←/→/h/l ", Style::default().fg(Color::Cyan)),
                    Span::raw("Switch between Overview/Billing/Models"),
                ]));
            }
            Tab::History => {
                lines.push(Line::from(vec![
                    Span::styled("  Enter       ", Style::default().fg(Color::Cyan)),
                    Span::raw("View session details"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  /           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Search history"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  Tab         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Toggle stats view"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  c           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Clear search"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  x           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Export filtered sessions (CSV/JSON)"),
                ]));
            }
            Tab::Mcp => {
                lines.push(Line::from(vec![
                    Span::styled("  ←/→         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Focus server/tool columns"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ↑/↓         ", Style::default().fg(Color::Cyan)),
                    Span::raw("Select server/tool"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  e           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Edit MCP config"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  o           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Reveal config file"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  r           ", Style::default().fg(Color::Cyan)),
                    Span::raw("Refresh servers"),
                ]));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "ESC",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close", Style::default().fg(Color::DarkGray)),
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
