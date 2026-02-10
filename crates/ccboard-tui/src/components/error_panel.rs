//! Error panel component for displaying LoadReport errors

use ccboard_core::error::{ErrorSeverity, LoadError};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render error panel showing LoadReport errors
pub fn render_error_panel(frame: &mut Frame, area: Rect, errors: &[LoadError], title: &str) {
    if errors.is_empty() {
        // No errors to show
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(
                format!(" {} ", title),
                Style::default().fg(Color::Green).bold(),
            ));

        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "✓ All data loaded successfully",
                Style::default().fg(Color::Green),
            )),
        ])
        .block(block);

        frame.render_widget(empty, area);
        return;
    }

    // Determine border color based on worst severity
    let has_fatal = errors.iter().any(|e| e.severity == ErrorSeverity::Fatal);
    let has_error = errors.iter().any(|e| e.severity == ErrorSeverity::Error);
    let border_color = if has_fatal {
        Color::Red
    } else if has_error {
        Color::Yellow
    } else {
        Color::Blue
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" {} ({}) ", title, errors.len()),
            Style::default().fg(border_color).bold(),
        ));

    let items: Vec<ListItem> = errors
        .iter()
        .flat_map(|error| {
            let mut lines = Vec::new();

            // Severity icon + source
            let (icon, color) = match error.severity {
                ErrorSeverity::Fatal => ("✗", Color::Red),
                ErrorSeverity::Error => ("⚠", Color::Yellow),
                ErrorSeverity::Warning => ("ⓘ", Color::Blue),
            };

            lines.push(Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(color).bold()),
                Span::styled(&error.source, Style::default().fg(color)),
            ]));

            // Message
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(&error.message, Style::default().fg(Color::White)),
            ]));

            // Suggestion if available
            if let Some(ref suggestion) = error.suggestion {
                lines.push(Line::from(vec![
                    Span::styled("    💡 ", Style::default().fg(Color::Cyan)),
                    Span::styled(suggestion, Style::default().fg(Color::Cyan)),
                ]));
            }

            // Empty line for spacing
            lines.push(Line::from(""));

            vec![ListItem::new(lines)]
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

/// Render error summary bar (compact, for status bar)
pub fn render_error_summary(errors: &[LoadError]) -> Line<'static> {
    if errors.is_empty() {
        return Line::from(vec![
            Span::styled(" ✓ ", Style::default().fg(Color::Green)),
            Span::styled("No errors", Style::default().fg(Color::DarkGray)),
        ]);
    }

    let fatal = errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Fatal)
        .count();
    let errors_count = errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Error)
        .count();
    let warnings = errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Warning)
        .count();

    let mut spans = Vec::new();

    if fatal > 0 {
        spans.push(Span::styled(" ✗ ", Style::default().fg(Color::Red).bold()));
        spans.push(Span::styled(
            format!("{} fatal", fatal),
            Style::default().fg(Color::Red),
        ));
        spans.push(Span::raw(" "));
    }

    if errors_count > 0 {
        spans.push(Span::styled(" ⚠ ", Style::default().fg(Color::Yellow)));
        spans.push(Span::styled(
            format!("{} errors", errors_count),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::raw(" "));
    }

    if warnings > 0 {
        spans.push(Span::styled(" ⓘ ", Style::default().fg(Color::Blue)));
        spans.push(Span::styled(
            format!("{} warnings", warnings),
            Style::default().fg(Color::Blue),
        ));
    }

    Line::from(spans)
}
