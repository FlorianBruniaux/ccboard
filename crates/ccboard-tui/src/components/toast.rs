//! Toast notification component

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::time::{Duration, Instant};

/// Toast notification type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Warning,
    Error,
    Info,
}

impl ToastType {
    pub fn color(&self) -> Color {
        match self {
            Self::Success => Color::Green,
            Self::Warning => Color::Yellow,
            Self::Error => Color::Red,
            Self::Info => Color::Cyan,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error => "✗",
            Self::Info => "ℹ",
        }
    }
}

/// Single toast message
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType) -> Self {
        Self {
            message: message.into(),
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3),
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Success)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Warning)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Error)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastType::Info)
    }
}

/// Toast manager - handles multiple toasts
#[derive(Debug, Default)]
pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn clear_expired(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Clear expired toasts
        self.clear_expired();

        if self.toasts.is_empty() {
            return;
        }

        // Stack toasts from bottom up (max 5 visible)
        let max_visible = 5;
        let visible_toasts: Vec<_> = self
            .toasts
            .iter()
            .rev()
            .take(max_visible)
            .rev()
            .collect();

        let toast_height: u16 = 3;
        let mut y_offset = area.height.saturating_sub((visible_toasts.len() as u16 * toast_height) + 2);

        for toast in visible_toasts {
            let toast_width = (toast.message.len() + 6).min(area.width as usize) as u16;
            let x_offset = (area.width.saturating_sub(toast_width)) / 2;

            let toast_area = Rect {
                x: area.x + x_offset,
                y: area.y + y_offset,
                width: toast_width,
                height: toast_height,
            };

            render_single_toast(frame, toast_area, toast);

            y_offset += toast_height;
        }
    }
}

fn render_single_toast(frame: &mut Frame, area: Rect, toast: &Toast) {
    let color = toast.toast_type.color();
    let icon = toast.toast_type.icon();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = Line::from(vec![
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(&toast.message, Style::default().fg(Color::White)),
    ]);

    let paragraph = Paragraph::new(content).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}
