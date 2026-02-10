//! Confirmation dialog component

use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Confirmation dialog result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmResult {
    Yes,
    No,
    Cancel,
}

/// Confirmation dialog state
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    visible: bool,
    title: String,
    message: String,
    default_option: ConfirmResult,
}

impl ConfirmDialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            visible: false,
            title: title.into(),
            message: message.into(),
            default_option: ConfirmResult::No,
        }
    }

    pub fn with_default(mut self, default: ConfirmResult) -> Self {
        self.default_option = default;
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Handle key input, returns Some(result) if a choice was made
    pub fn handle_key(&mut self, key: KeyCode) -> Option<ConfirmResult> {
        if !self.visible {
            return None;
        }

        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.hide();
                Some(ConfirmResult::Yes)
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.hide();
                Some(ConfirmResult::No)
            }
            KeyCode::Esc => {
                self.hide();
                Some(ConfirmResult::Cancel)
            }
            KeyCode::Enter => {
                self.hide();
                Some(self.default_option)
            }
            _ => None,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Center dialog (50% width, auto height)
        let dialog_width = (area.width as f32 * 0.5).max(40.0) as u16;
        let dialog_height = 10;
        let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
        let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect {
            x: area.x + dialog_x,
            y: area.y + dialog_y,
            width: dialog_width,
            height: dialog_height,
        };

        // Clear background
        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(Span::styled(
                format!(" {} ", self.title),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // Layout: message + buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // Message
                Constraint::Length(3), // Buttons
            ])
            .split(inner);

        // Message
        let message_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                &self.message,
                Style::default().fg(Color::White),
            )),
            Line::from(""),
        ];

        let message_widget = Paragraph::new(message_lines)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(message_widget, chunks[0]);

        // Buttons
        let button_lines = vec![
            Line::from(vec![
                Span::styled(
                    "[Y] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Yes",
                    Style::default().fg(if matches!(self.default_option, ConfirmResult::Yes) {
                        Color::Green
                    } else {
                        Color::White
                    }),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    "[N] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "No",
                    Style::default().fg(if matches!(self.default_option, ConfirmResult::No) {
                        Color::Red
                    } else {
                        Color::White
                    }),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(
                    "[Esc] ",
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Cancel", Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                format!(
                    "(Enter = {})",
                    match self.default_option {
                        ConfirmResult::Yes => "Yes",
                        ConfirmResult::No => "No",
                        ConfirmResult::Cancel => "Cancel",
                    }
                ),
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let buttons_widget = Paragraph::new(button_lines).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, chunks[1]);
    }
}
