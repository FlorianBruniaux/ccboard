//! Activity models for session tool call auditing and security alerting

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A tool call extracted from a session JSONL file
///
/// Combines tool_use (from assistant messages) with tool_result (from user messages)
/// to compute duration and capture output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID (from tool_use block)
    pub id: String,

    /// Session this tool call belongs to
    pub session_id: String,

    /// Timestamp when the tool was called (from assistant message)
    pub timestamp: DateTime<Utc>,

    /// Tool name (e.g., "Read", "Bash", "WebFetch", "mcp__server__tool")
    pub tool_name: String,

    /// Input parameters as JSON
    pub input: Value,

    /// Duration from call to result in milliseconds (None if no result found)
    pub duration_ms: Option<u64>,

    /// Tool output content (from tool_result block, truncated preview)
    pub output: Option<String>,
}

/// File operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Edit,
    Glob,
    Grep,
}

/// A file access event extracted from tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAccess {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub path: String,
    pub operation: FileOperation,
    /// Line range for Read operations: (offset, offset + limit)
    pub line_range: Option<(u64, u64)>,
}

/// A bash command execution event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashCommand {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub is_destructive: bool,
    /// Preview of command output (first 500 chars)
    pub output_preview: String,
}

/// Network tool type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTool {
    WebFetch,
    WebSearch,
    McpCall { server: String },
}

/// A network call event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCall {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    /// URL for WebFetch, query for WebSearch, empty for MCP
    pub url: String,
    pub tool: NetworkTool,
    /// Extracted domain (empty for WebSearch/MCP without URL)
    pub domain: String,
}

/// Alert severity level (ordered: Info < Warning < Critical)
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert category for security audit classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    CredentialAccess,
    DestructiveCommand,
    ExternalExfil,
    ScopeViolation,
    ForcePush,
}

impl AlertCategory {
    /// Remediation hint shown in the Violations view.
    /// Match is exhaustive — adding a new variant requires a hint here.
    pub fn action_hint(&self) -> &'static str {
        match self {
            AlertCategory::CredentialAccess => {
                "Verify the credential wasn't exposed. If it was, rotate it immediately."
            }
            AlertCategory::DestructiveCommand => {
                "Check if deleted files are recoverable (Trash, git stash, backup)."
            }
            AlertCategory::ExternalExfil => {
                "Review what data was sent to this domain and whether it was intentional."
            }
            AlertCategory::ScopeViolation => {
                "Inspect the file written outside the project root. Delete it if unintended."
            }
            AlertCategory::ForcePush => {
                "Run git reflog to find the overwritten commit. Force-push a revert if needed."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_hint_all_variants_non_empty() {
        // Exhaustive: if a new AlertCategory variant is added without a hint,
        // the match in action_hint() will fail to compile.
        let variants = [
            AlertCategory::CredentialAccess,
            AlertCategory::DestructiveCommand,
            AlertCategory::ExternalExfil,
            AlertCategory::ScopeViolation,
            AlertCategory::ForcePush,
        ];
        for variant in &variants {
            let hint = variant.action_hint();
            assert!(!hint.is_empty(), "{:?} has an empty action hint", variant);
        }
    }
}

/// A security alert generated from activity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub detail: String,
}

/// Summary of all activity extracted from a session
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivitySummary {
    pub file_accesses: Vec<FileAccess>,
    pub bash_commands: Vec<BashCommand>,
    pub network_calls: Vec<NetworkCall>,
    pub alerts: Vec<Alert>,
}
