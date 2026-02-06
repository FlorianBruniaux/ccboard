//! Session models for JSONL session files

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

/// A single line from a session JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLine {
    /// Session ID
    #[serde(default)]
    pub session_id: Option<String>,

    /// Event type: "user", "assistant", "file-history-snapshot", "session_end", etc.
    #[serde(rename = "type")]
    pub line_type: String,

    /// Timestamp of the event
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,

    /// Current working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Git branch (if available)
    #[serde(default)]
    pub git_branch: Option<String>,

    /// Message content (for user/assistant types)
    #[serde(default)]
    pub message: Option<SessionMessage>,

    /// Model used (for assistant messages)
    #[serde(default)]
    pub model: Option<String>,

    /// Token usage for this message
    #[serde(default)]
    pub usage: Option<TokenUsage>,

    /// Summary data (for session_end type)
    #[serde(default)]
    pub summary: Option<SessionSummary>,

    /// Parent session ID (for subagents)
    #[serde(default)]
    pub parent_session_id: Option<String>,
}

/// Message content in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    /// Role: "user" or "assistant"
    #[serde(default)]
    pub role: Option<String>,

    /// Text content (can be String or Array of content blocks in newer Claude Code versions)
    #[serde(default)]
    pub content: Option<Value>,

    /// Tool calls made
    #[serde(default)]
    pub tool_calls: Option<Vec<serde_json::Value>>,

    /// Tool results
    #[serde(default)]
    pub tool_results: Option<Vec<serde_json::Value>>,

    /// Token usage (for assistant messages)
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

/// Token usage for a message
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub input_tokens: u64,

    #[serde(default)]
    pub output_tokens: u64,

    /// Cache read tokens (from cache_read_input_tokens in JSONL)
    #[serde(default, alias = "cache_read_input_tokens")]
    pub cache_read_tokens: u64,

    /// Cache creation tokens (from cache_creation_input_tokens in JSONL)
    #[serde(default, alias = "cache_creation_input_tokens")]
    pub cache_write_tokens: u64,
}

impl TokenUsage {
    /// Total tokens including cache reads and writes
    ///
    /// This is the sum of all token types:
    /// - input_tokens: Regular input tokens (not cached)
    /// - output_tokens: Generated tokens
    /// - cache_read_tokens: Tokens read from cache (cache hits)
    /// - cache_write_tokens: Tokens written to cache (cache creation)
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_read_tokens + self.cache_write_tokens
    }
}

/// Summary at session end
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    #[serde(default)]
    pub total_tokens: u64,
    #[serde(default)]
    pub total_input_tokens: u64,
    #[serde(default)]
    pub total_output_tokens: u64,
    #[serde(default)]
    pub total_cache_read_tokens: u64,
    #[serde(default)]
    pub total_cache_write_tokens: u64,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub duration_seconds: Option<u64>,
    #[serde(default)]
    pub models_used: Option<Vec<String>>,
}

/// Metadata extracted from a session without full parse
///
/// Created by streaming the JSONL until session_end event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID (from filename or content)
    pub id: String,

    /// Full path to the JSONL file
    pub file_path: PathBuf,

    /// Project path (extracted from directory structure)
    pub project_path: String,

    /// First timestamp in session
    pub first_timestamp: Option<DateTime<Utc>>,

    /// Last timestamp in session
    pub last_timestamp: Option<DateTime<Utc>>,

    /// Total message count (from summary or counted)
    pub message_count: u64,

    /// Total tokens (from summary)
    pub total_tokens: u64,

    /// Token breakdown for precise pricing
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,

    /// Models used in this session
    pub models_used: Vec<String>,

    /// File size in bytes
    pub file_size_bytes: u64,

    /// Preview of first user message (truncated to 200 chars)
    pub first_user_message: Option<String>,

    /// Whether this session spawned subagents
    pub has_subagents: bool,

    /// Duration in seconds (from summary)
    pub duration_seconds: Option<u64>,

    /// Git branch name (normalized, extracted from first gitBranch in session)
    pub branch: Option<String>,

    /// Tool usage statistics: tool name -> call count
    /// Extracted from tool_calls in assistant messages during session scan
    pub tool_usage: std::collections::HashMap<String, usize>,
}

impl SessionMetadata {
    /// Create a minimal metadata from just file path
    pub fn from_path(path: PathBuf, project_path: String) -> Self {
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let file_size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            id,
            file_path: path,
            project_path,
            first_timestamp: None,
            last_timestamp: None,
            message_count: 0,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: Vec::new(),
            file_size_bytes,
            first_user_message: None,
            has_subagents: false,
            duration_seconds: None,
            branch: None,
            tool_usage: std::collections::HashMap::new(),
        }
    }

    /// Human-readable duration
    pub fn duration_display(&self) -> String {
        match self.duration_seconds {
            Some(s) if s >= 3600 => format!("{}h {}m", s / 3600, (s % 3600) / 60),
            Some(s) if s >= 60 => format!("{}m {}s", s / 60, s % 60),
            Some(s) => format!("{}s", s),
            None => "unknown".to_string(),
        }
    }

    /// Human-readable file size
    pub fn size_display(&self) -> String {
        let bytes = self.file_size_bytes;
        if bytes >= 1_000_000 {
            format!("{:.1} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.1} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} B", bytes)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_total() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            ..Default::default()
        };
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn test_session_metadata_duration_display() {
        let mut meta = SessionMetadata::from_path(PathBuf::from("/test.jsonl"), "test".to_string());

        meta.duration_seconds = Some(90);
        assert_eq!(meta.duration_display(), "1m 30s");

        meta.duration_seconds = Some(3665);
        assert_eq!(meta.duration_display(), "1h 1m");

        meta.duration_seconds = Some(45);
        assert_eq!(meta.duration_display(), "45s");
    }

    #[test]
    fn test_session_metadata_size_display() {
        let mut meta = SessionMetadata::from_path(PathBuf::from("/test.jsonl"), "test".to_string());

        meta.file_size_bytes = 500;
        assert_eq!(meta.size_display(), "500 B");

        meta.file_size_bytes = 5_000;
        assert_eq!(meta.size_display(), "5.0 KB");

        meta.file_size_bytes = 2_500_000;
        assert_eq!(meta.size_display(), "2.5 MB");
    }
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn test_real_claude_token_format_deserialization() {
        // CRITICAL: Real format from Claude Code v2.1.29+
        let json = r#"{
            "input_tokens": 10,
            "cache_creation_input_tokens": 64100,
            "cache_read_input_tokens": 19275,
            "cache_creation": {
                "ephemeral_5m_input_tokens": 0,
                "ephemeral_1h_input_tokens": 64100
            },
            "output_tokens": 1,
            "service_tier": "standard"
        }"#;

        let result: Result<TokenUsage, _> = serde_json::from_str(json);

        assert!(
            result.is_ok(),
            "Deserialization MUST succeed for real Claude format. Error: {:?}",
            result.err()
        );

        let usage = result.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 1);
        assert_eq!(usage.cache_read_tokens, 19275);
        assert_eq!(usage.cache_write_tokens, 64100);

        let total = usage.total();
        assert_eq!(total, 83386, "Total should be 10+1+19275+64100 = 83386");
    }
}
