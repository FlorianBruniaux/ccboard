//! Session models for JSONL session files

use chrono::{DateTime, Utc};
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::{Borrow, Cow};
use std::fmt;
use std::ops::{Deref, Index, Range, RangeFrom, RangeFull, RangeTo};
use std::path::PathBuf;

/// Newtype for Session ID - zero-cost type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new SessionId
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get reference to inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract inner String, consuming self
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Check if the session ID is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get an iterator over the characters
    pub fn chars(&self) -> std::str::Chars<'_> {
        self.0.chars()
    }

    /// Check if the session ID starts with a given pattern
    pub fn starts_with(&self, pattern: &str) -> bool {
        self.0.starts_with(pattern)
    }

    /// Get the length of the session ID
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<String> for SessionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SessionId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for SessionId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for SessionId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for SessionId {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }
}

impl ToSql for SessionId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.as_str()))
    }
}

impl FromSql for SessionId {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| SessionId::from(s))
    }
}

impl Borrow<str> for SessionId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Deref for SessionId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Index<RangeFull> for SessionId {
    type Output = str;

    fn index(&self, _index: RangeFull) -> &Self::Output {
        &self.0
    }
}

impl Index<Range<usize>> for SessionId {
    type Output = str;

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}

impl Index<RangeFrom<usize>> for SessionId {
    type Output = str;

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.0[index]
    }
}

impl Index<RangeTo<usize>> for SessionId {
    type Output = str;

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> From<&'a SessionId> for Cow<'a, str> {
    fn from(id: &'a SessionId) -> Self {
        Cow::Borrowed(id.as_str())
    }
}

/// Newtype for Project ID - zero-cost type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(String);

impl ProjectId {
    /// Create a new ProjectId
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get reference to inner string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract inner String, consuming self
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Get the length of the project ID
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the project ID is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert to lowercase
    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

impl From<String> for ProjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ProjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ProjectId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToSql for ProjectId {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.as_str()))
    }
}

impl FromSql for ProjectId {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        value.as_str().map(|s| ProjectId::from(s))
    }
}

impl Deref for ProjectId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a ProjectId> for Cow<'a, str> {
    fn from(id: &'a ProjectId) -> Self {
        Cow::Borrowed(id.as_str())
    }
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl Default for MessageRole {
    fn default() -> Self {
        Self::User
    }
}

/// A single line from a session JSONL file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub id: SessionId,

    /// Full path to the JSONL file
    pub file_path: PathBuf,

    /// Project path (extracted from directory structure)
    pub project_path: ProjectId,

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
    pub fn from_path(path: PathBuf, project_path: ProjectId) -> Self {
        let id = SessionId::new(
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
        );

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

/// A single conversation message extracted from session JSONL
///
/// Simplified representation for display in conversation viewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message role (User, Assistant, System)
    pub role: MessageRole,

    /// Text content (extracted from SessionLine.message.content)
    pub content: String,

    /// Timestamp when message was sent
    pub timestamp: Option<DateTime<Utc>>,

    /// Model used (for assistant messages)
    pub model: Option<String>,

    /// Token usage (for assistant messages)
    pub tokens: Option<TokenUsage>,

    /// Tool calls made in this message (if any)
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    /// Tool results received (if any)
    #[serde(default)]
    pub tool_results: Vec<ToolResult>,
}

/// A tool call made by the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool name (e.g., "Read", "Bash", "Edit")
    pub name: String,

    /// Tool call ID for matching with results
    pub id: String,

    /// Input parameters as JSON
    pub input: serde_json::Value,
}

/// Result of a tool call execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID this result corresponds to
    pub tool_call_id: String,

    /// Whether the tool succeeded
    pub is_error: bool,

    /// Output content
    pub content: String,
}

/// Full session content with metadata + messages
///
/// Returned by SessionContentParser for lazy-loaded session display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContent {
    /// Messages in chronological order
    pub messages: Vec<ConversationMessage>,

    /// Session metadata (from cache or index)
    pub metadata: SessionMetadata,
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
        let mut meta =
            SessionMetadata::from_path(PathBuf::from("/test.jsonl"), ProjectId::from("test"));

        meta.duration_seconds = Some(90);
        assert_eq!(meta.duration_display(), "1m 30s");

        meta.duration_seconds = Some(3665);
        assert_eq!(meta.duration_display(), "1h 1m");

        meta.duration_seconds = Some(45);
        assert_eq!(meta.duration_display(), "45s");
    }

    #[test]
    fn test_session_metadata_size_display() {
        let mut meta =
            SessionMetadata::from_path(PathBuf::from("/test.jsonl"), ProjectId::from("test"));

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
