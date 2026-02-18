//! Tool call widget - Expandable display for tool calls and results

use crate::theme::Palette;
use ccboard_core::models::config::ColorScheme;
use ccboard_core::models::{ToolCall, ToolResult};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Expandable tool call node state
pub struct ToolCallNode {
    /// Tool call information
    pub tool_call: ToolCall,

    /// Matching tool result (if available)
    pub tool_result: Option<ToolResult>,

    /// Whether this node is expanded
    pub expanded: bool,
}

impl ToolCallNode {
    /// Create a new tool call node
    pub fn new(tool_call: ToolCall, tool_result: Option<ToolResult>) -> Self {
        Self {
            tool_call,
            tool_result,
            expanded: false,
        }
    }

    /// Toggle expanded state
    pub fn toggle(&mut self) {
        self.expanded = !self.expanded;
    }

    /// Get status indicator and color
    fn status(&self) -> (&'static str, Color) {
        match &self.tool_result {
            Some(result) if result.is_error => ("❌", Color::Red),
            Some(_) => ("✅", Color::Green),
            None => ("⏳", Color::Yellow),
        }
    }

    /// Render the tool call node
    pub fn render(&self, frame: &mut Frame, area: Rect, p: &Palette) {
        let (status_icon, status_color) = self.status();

        // Header line: status + tool name + expand indicator
        let expand_indicator = if self.expanded { "▼" } else { "▶" };
        let header_spans = vec![
            Span::styled(status_icon, Style::default().fg(status_color)),
            Span::raw(" "),
            Span::styled(
                &self.tool_call.name,
                Style::default().fg(p.focus).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(expand_indicator, Style::default().fg(p.muted)),
        ];

        if !self.expanded {
            // Collapsed: just show header
            let header = Paragraph::new(Line::from(header_spans))
                .style(Style::default().bg(Color::Rgb(30, 30, 40)));
            frame.render_widget(header, area);
        } else {
            // Expanded: show header + details
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(status_color))
                .style(Style::default().bg(Color::Rgb(20, 20, 30)));

            let inner = block.inner(area);
            frame.render_widget(block, area);

            // Split inner: header + input + result
            let constraints = if self.tool_result.is_some() {
                vec![
                    Constraint::Length(1), // Header
                    Constraint::Length(1), // Separator
                    Constraint::Min(3),    // Input
                    Constraint::Length(1), // Separator
                    Constraint::Min(3),    // Result
                ]
            } else {
                vec![
                    Constraint::Length(1), // Header
                    Constraint::Length(1), // Separator
                    Constraint::Min(3),    // Input only
                ]
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(inner);

            // Render header
            let header = Paragraph::new(Line::from(header_spans));
            frame.render_widget(header, chunks[0]);

            // Render input parameters
            let input_text = format!(
                "📥 Input:\n{}",
                serde_json::to_string_pretty(&self.tool_call.input)
                    .unwrap_or_else(|_| "{}".to_string())
            );
            let input_widget = Paragraph::new(input_text)
                .wrap(Wrap { trim: false })
                .style(Style::default().fg(Color::Gray));
            frame.render_widget(input_widget, chunks[2]);

            // Render result if available
            if let Some(result) = &self.tool_result {
                let result_header = if result.is_error {
                    "❌ Error:"
                } else {
                    "📤 Output:"
                };

                let result_text = format!("{}\n{}", result_header, result.content);
                let result_color = if result.is_error {
                    Color::Red
                } else {
                    Color::Green
                };

                let result_widget = Paragraph::new(result_text)
                    .wrap(Wrap { trim: false })
                    .style(Style::default().fg(result_color));

                frame.render_widget(result_widget, chunks[4]);
            }
        }
    }
}

/// Tool calls viewer - Manages multiple tool call nodes
pub struct ToolCallsViewer {
    /// List of tool call nodes
    pub nodes: Vec<ToolCallNode>,

    /// Currently selected node index
    pub selected: usize,
}

impl ToolCallsViewer {
    /// Create a new tool calls viewer from tool calls and results
    pub fn new(tool_calls: Vec<ToolCall>, tool_results: Vec<ToolResult>) -> Self {
        let mut nodes = Vec::new();

        for call in tool_calls {
            // Find matching result by tool_call_id
            let result = tool_results
                .iter()
                .find(|r| r.tool_call_id == call.id)
                .cloned();

            nodes.push(ToolCallNode::new(call, result));
        }

        Self { nodes, selected: 0 }
    }

    /// Toggle selected node
    pub fn toggle_selected(&mut self) {
        if let Some(node) = self.nodes.get_mut(self.selected) {
            node.toggle();
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected < self.nodes.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Render all tool call nodes
    pub fn render(&self, frame: &mut Frame, area: Rect, scheme: ColorScheme) {
        let p = Palette::new(scheme);

        if self.nodes.is_empty() {
            let placeholder = Paragraph::new("No tool calls").style(Style::default().fg(p.muted));
            frame.render_widget(placeholder, area);
            return;
        }

        // Calculate layout for each node
        let mut y_offset = 0;

        for (idx, node) in self.nodes.iter().enumerate() {
            if y_offset >= area.height as usize {
                break;
            }

            let node_height = if node.expanded { 15 } else { 1 };
            let node_area = Rect {
                x: area.x,
                y: area.y + y_offset as u16,
                width: area.width,
                height: node_height.min((area.height as usize - y_offset) as u16),
            };

            // Highlight selected node
            if idx == self.selected {
                let highlight = Block::default()
                    .borders(Borders::LEFT)
                    .border_style(Style::default().fg(p.warning));
                frame.render_widget(highlight, node_area);
            }

            node.render(frame, node_area, &p);
            y_offset += node_height as usize + 1; // +1 for spacing
        }
    }
}
