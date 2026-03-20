//! Reusable empty state components with actionable hints
//!
//! Provides consistent empty state patterns across all tabs,
//! inspired by lazygit's informative empty states.

use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

// Palette-aligned Rgb values (matching theme::Palette Dark scheme)
const TITLE_COLOR: Color = Color::Rgb(220, 175, 60); // warning amber
const MSG_COLOR: Color = Color::Rgb(90, 95, 120); // muted
const ACTIONS_LABEL_COLOR: Color = Color::Rgb(80, 190, 210); // focus cyan
const KEY_COLOR: Color = Color::Rgb(80, 200, 120); // success green
const DESC_COLOR: Color = Color::Rgb(220, 220, 235); // fg soft white
const BRACKET_COLOR: Color = Color::Rgb(90, 95, 120); // muted

/// Builder for empty state messages
pub struct EmptyState {
    title: String,
    message: Vec<String>,
    actions: Vec<(String, String)>, // (key, description)
}

impl EmptyState {
    /// Create new empty state with title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: Vec::new(),
            actions: Vec::new(),
        }
    }

    /// Add a message line
    pub fn message(mut self, msg: impl Into<String>) -> Self {
        self.message.push(msg.into());
        self
    }

    /// Add an action hint
    pub fn action(mut self, key: impl Into<String>, description: impl Into<String>) -> Self {
        self.actions.push((key.into(), description.into()));
        self
    }

    /// Build the paragraph widget
    pub fn build(self) -> Paragraph<'static> {
        let mut lines = Vec::new();

        // Empty line for spacing
        lines.push(Line::from(""));

        // Title
        lines.push(Line::from(Span::styled(
            self.title,
            Style::default().fg(TITLE_COLOR),
        )));

        // Empty line after title
        lines.push(Line::from(""));

        // Message lines
        for msg in self.message {
            lines.push(Line::from(Span::styled(
                msg,
                Style::default().fg(MSG_COLOR),
            )));
        }

        // Empty line before actions
        if !self.actions.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Actions:",
                Style::default().fg(ACTIONS_LABEL_COLOR),
            )));

            for (key, desc) in self.actions {
                lines.push(Line::from(vec![
                    Span::styled("  [", Style::default().fg(BRACKET_COLOR)),
                    Span::styled(key, Style::default().fg(KEY_COLOR)),
                    Span::styled("] ", Style::default().fg(BRACKET_COLOR)),
                    Span::styled(desc, Style::default().fg(DESC_COLOR)),
                ]));
            }
        }

        Paragraph::new(lines).alignment(Alignment::Center)
    }
}

/// Predefined empty states for common scenarios
pub fn no_mcp_config() -> Paragraph<'static> {
    EmptyState::new("No MCP Config Found")
        .message("Claude Desktop config not found")
        .message("")
        .message("Expected location:")
        .message("~/.claude/claude_desktop_config.json")
        .action("e", "Edit/Create config file")
        .action("r", "Refresh")
        .build()
}

pub fn no_mcp_servers() -> Paragraph<'static> {
    EmptyState::new("No MCP Servers Configured")
        .message("Add MCP servers to claude_desktop_config.json")
        .message("")
        .message("Example:")
        .message(r#"  "mcpServers": { "myserver": { "command": "..." } }"#)
        .action("e", "Edit config")
        .action("r", "Refresh")
        .build()
}

pub fn no_sessions(project: Option<&str>) -> Paragraph<'static> {
    let mut state = EmptyState::new("No Sessions Found");

    if let Some(proj) = project {
        state = state.message(format!("No sessions in project: {}", proj));
    } else {
        state = state.message("No sessions in any project yet");
    }

    state
        .message("")
        .message("Get started:")
        .message("• Use Claude Code to create a session")
        .message("• Or specify different project:")
        .message("  ccboard --project ~/path/to/project")
        .action("r", "Refresh")
        .action("c", "Change project")
        .build()
}

pub fn no_agents() -> Paragraph<'static> {
    EmptyState::new("No Agents Found")
        .message("No agent definitions in .claude/agents/")
        .message("")
        .message("Create an agent:")
        .message("  1. Create .claude/agents/myagent.md")
        .message("  2. Add YAML frontmatter with agent config")
        .message("  3. Define agent behavior in markdown")
        .action("r", "Refresh")
        .build()
}

pub fn no_hooks() -> Paragraph<'static> {
    EmptyState::new("No Hooks Found")
        .message("No hook scripts in .claude/hooks/bash/")
        .message("")
        .message("Create a hook:")
        .message("  1. Create .claude/hooks/bash/pre-tool-use.sh")
        .message("  2. Make it executable (chmod +x)")
        .message("  3. Test with Claude Code")
        .action("r", "Refresh")
        .build()
}

pub fn no_history() -> Paragraph<'static> {
    EmptyState::new("No History Available")
        .message("No session history found")
        .message("")
        .message("History is populated from session JSONL files")
        .action("r", "Refresh")
        .build()
}

pub fn no_insights(period_label: impl Into<String>) -> Paragraph<'static> {
    EmptyState::new("No Insights Yet")
        .message(format!(
            "Not enough data for period: {}",
            period_label.into()
        ))
        .message("Insights are computed from session patterns over time")
        .action("F1", "Last 7 days")
        .action("F2", "Last 30 days")
        .action("F3", "Last 90 days")
        .action("r", "Refresh")
        .build()
}

pub fn no_server_selected() -> Paragraph<'static> {
    EmptyState::new("No Server Selected")
        .message("Select a server from the list to view details")
        .action("j/k", "Navigate servers")
        .action("r", "Refresh status")
        .build()
}

pub fn no_search_results(query: &str) -> Paragraph<'static> {
    EmptyState::new("No Results Found")
        .message(format!("No matches for query: {}", query))
        .message("")
        .message("Try:")
        .message("• Different search terms")
        .message("• Broader keywords")
        .message("• Check spelling")
        .action("Esc", "Clear search")
        .build()
}

pub fn loading() -> Paragraph<'static> {
    EmptyState::new("Loading...")
        .message("Fetching data...")
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_state_builder() {
        let _state = EmptyState::new("Test Title")
            .message("Test message")
            .action("r", "Refresh")
            .build();

        // Just verify it builds without panicking
    }

    #[test]
    fn test_predefined_states() {
        // Verify predefined states build without panicking
        let _ = no_mcp_config();
        let _ = no_mcp_servers();
        let _ = no_sessions(Some("test-project"));
        let _ = no_agents();
        let _ = no_hooks();
        let _ = no_history();
        let _ = no_search_results("test query");
        let _ = loading();
    }
}
