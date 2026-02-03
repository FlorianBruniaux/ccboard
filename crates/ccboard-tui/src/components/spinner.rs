//! Animated spinner component for loading states

use ratatui::{
    style::{Color, Style},
    text::Span,
};
use std::time::{Duration, Instant};

/// Animated spinner for loading indicators
#[derive(Debug)]
pub struct Spinner {
    /// Animation frames (Braille patterns)
    frames: &'static [&'static str],
    /// Current frame index
    current_frame: usize,
    /// Last frame update time
    last_update: Instant,
    /// Frame duration
    frame_duration: Duration,
    /// Spinner color
    color: Color,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Create a new spinner with default settings
    pub fn new() -> Self {
        Self {
            frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            current_frame: 0,
            last_update: Instant::now(),
            frame_duration: Duration::from_millis(80),
            color: Color::Cyan,
        }
    }

    /// Create a spinner with custom color
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Create a spinner with custom frame duration
    pub fn with_frame_duration(mut self, duration: Duration) -> Self {
        self.frame_duration = duration;
        self
    }

    /// Update spinner state (call this on each render)
    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
        }
    }

    /// Get current frame as a styled span
    pub fn render(&self) -> Span<'static> {
        Span::styled(
            self.frames[self.current_frame],
            Style::default().fg(self.color),
        )
    }

    /// Get current frame text without styling
    pub fn current_frame(&self) -> &'static str {
        self.frames[self.current_frame]
    }
}

/// Spinner variant for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    /// Standard Braille dots (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏)
    Dots,
    /// Simple line (|/-\)
    Line,
    /// Bouncing bar (⣾⣽⣻⢿⡿⣟⣯⣷)
    Bounce,
    /// Growing circle (◐◓◑◒)
    Circle,
}

impl SpinnerStyle {
    /// Get animation frames for this style
    pub fn frames(&self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Dots => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerStyle::Line => &["|", "/", "-", "\\"],
            SpinnerStyle::Bounce => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            SpinnerStyle::Circle => &["◐", "◓", "◑", "◒"],
        }
    }
}

impl Spinner {
    /// Create spinner with specific style
    pub fn with_style(style: SpinnerStyle) -> Self {
        Self {
            frames: style.frames(),
            current_frame: 0,
            last_update: Instant::now(),
            frame_duration: Duration::from_millis(80),
            color: Color::Cyan,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_cycles() {
        let mut spinner = Spinner::new();
        assert_eq!(spinner.current_frame, 0);

        spinner.tick();
        // Frame may or may not advance depending on timing
        assert!(spinner.current_frame < spinner.frames.len());
    }

    #[test]
    fn test_spinner_styles() {
        let dots = SpinnerStyle::Dots;
        let line = SpinnerStyle::Line;

        assert_eq!(dots.frames().len(), 10);
        assert_eq!(line.frames().len(), 4);
    }

    #[test]
    fn test_custom_color() {
        let spinner = Spinner::new().with_color(Color::Yellow);
        assert_eq!(spinner.color, Color::Yellow);
    }
}
