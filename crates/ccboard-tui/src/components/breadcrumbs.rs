use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

/// A single breadcrumb in the navigation path
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    /// Display label
    pub label: String,
    /// Nesting level (0 = root)
    pub level: usize,
    /// Whether this breadcrumb is navigable (future feature)
    pub navigable: bool,
}

impl Breadcrumb {
    /// Create a new breadcrumb
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            level: 0,
            navigable: false,
        }
    }

    /// Set the nesting level
    pub fn with_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    /// Mark as navigable (future feature)
    #[allow(dead_code)]
    pub fn navigable(mut self) -> Self {
        self.navigable = true;
        self
    }
}

/// Breadcrumb navigation trail component
pub struct Breadcrumbs {
    /// Path of breadcrumbs
    path: Vec<Breadcrumb>,
    /// Maximum breadcrumbs to display before truncation
    max_display: usize,
}

impl Default for Breadcrumbs {
    fn default() -> Self {
        Self::new()
    }
}

impl Breadcrumbs {
    /// Create a new empty breadcrumbs trail
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            max_display: 5,
        }
    }

    /// Set the breadcrumb path
    pub fn set_path(&mut self, path: Vec<Breadcrumb>) {
        self.path = path;
    }

    /// Set maximum display count before truncation
    pub fn with_max_display(mut self, max: usize) -> Self {
        self.max_display = max;
        self
    }

    /// Render the breadcrumbs as a paragraph
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.path.is_empty() {
            return;
        }

        let spans = self.build_spans();
        let breadcrumb_line = Line::from(spans);
        let paragraph = Paragraph::new(breadcrumb_line).style(Style::default().fg(Color::DarkGray));

        frame.render_widget(paragraph, area);
    }

    /// Build the display spans for breadcrumbs
    fn build_spans(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();

        // Add location pin icon
        spans.push(Span::styled("📍 ", Style::default().fg(Color::Cyan)));

        // Determine if we need to truncate
        let path_to_display = if self.path.len() > self.max_display {
            self.truncate_path()
        } else {
            self.path.clone()
        };

        // Build breadcrumb trail
        for (idx, crumb) in path_to_display.iter().enumerate() {
            // Add breadcrumb label
            let style = if idx == path_to_display.len() - 1 {
                // Last breadcrumb (current location) is highlighted
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            spans.push(Span::styled(crumb.label.clone(), style));

            // Add separator (except after last item)
            if idx < path_to_display.len() - 1 {
                spans.push(Span::styled(" > ", Style::default().fg(Color::DarkGray)));
            }
        }

        spans
    }

    /// Truncate path to fit max_display limit
    /// Shows "... > Second-to-last > Last"
    fn truncate_path(&self) -> Vec<Breadcrumb> {
        let mut result = Vec::new();

        // Always show first breadcrumb
        if let Some(first) = self.path.first() {
            result.push(first.clone());
        }

        // Add ellipsis indicator
        result.push(Breadcrumb::new("..."));

        // Show last 2 breadcrumbs
        let skip_count = self.path.len().saturating_sub(2);
        result.extend(self.path.iter().skip(skip_count).cloned());

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breadcrumb_creation() {
        let crumb = Breadcrumb::new("Dashboard");
        assert_eq!(crumb.label, "Dashboard");
        assert_eq!(crumb.level, 0);
        assert!(!crumb.navigable);
    }

    #[test]
    fn test_breadcrumb_with_level() {
        let crumb = Breadcrumb::new("Sessions").with_level(1);
        assert_eq!(crumb.level, 1);
    }

    #[test]
    fn test_breadcrumbs_truncation() {
        let mut breadcrumbs = Breadcrumbs::new().with_max_display(5);

        // Create 7 breadcrumbs (exceeds max)
        let path = vec![
            Breadcrumb::new("Dashboard"),
            Breadcrumb::new("Sessions"),
            Breadcrumb::new("Project A"),
            Breadcrumb::new("Session 1"),
            Breadcrumb::new("Details"),
            Breadcrumb::new("Messages"),
            Breadcrumb::new("Message 42"),
        ];

        breadcrumbs.set_path(path);

        let truncated = breadcrumbs.truncate_path();

        // Should have: Dashboard, ..., Messages, Message 42
        assert_eq!(truncated.len(), 4);
        assert_eq!(truncated[0].label, "Dashboard");
        assert_eq!(truncated[1].label, "...");
        assert_eq!(truncated[2].label, "Messages");
        assert_eq!(truncated[3].label, "Message 42");
    }

    #[test]
    fn test_breadcrumbs_no_truncation() {
        let mut breadcrumbs = Breadcrumbs::new().with_max_display(5);

        let path = vec![
            Breadcrumb::new("Dashboard"),
            Breadcrumb::new("Config"),
            Breadcrumb::new("Hooks"),
        ];

        breadcrumbs.set_path(path.clone());

        let spans = breadcrumbs.build_spans();

        // Should contain all 3 labels (plus icon and separators)
        // 📍 Dashboard > Config > Hooks
        // = 1 icon + 3 labels + 2 separators = 6 spans
        assert_eq!(spans.len(), 6);
    }
}
