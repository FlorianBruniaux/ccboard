//! Full session content parser for message replay
//!
//! Parses entire JSONL session files to extract chronological message stream.
//! Used for session replay viewer in TUI.

use crate::error::CoreError;
use crate::models::{ConversationMessage, MessageRole, SessionLine, SessionMetadata};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{trace, warn};

/// Maximum line size in bytes (10MB) - OOM protection
const MAX_LINE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum lines to parse (circuit breaker for malformed/infinite files)
const MAX_LINES: usize = 50_000;

/// Parser for full session content
pub struct SessionContentParser;

impl SessionContentParser {
    /// Parse full session content with conversation messages
    ///
    /// Returns ConversationMessage list extracted from JSONL session file.
    /// Converts SessionLine events into user/assistant/system messages for display.
    ///
    /// # Performance
    /// - Streaming parse: doesn't load entire file into memory
    /// - Early termination: stops at MAX_LINES circuit breaker
    /// - Line size limit: skips lines >10MB to prevent OOM
    /// - Graceful degradation: skips malformed lines
    ///
    /// # Errors
    /// Returns CoreError if file cannot be read (not found, permissions, etc).
    /// Malformed JSON lines are skipped with warnings (graceful degradation).
    pub async fn parse_conversation(
        session_path: &Path,
        _metadata: SessionMetadata,
    ) -> Result<Vec<ConversationMessage>, CoreError> {
        let lines = Self::parse_session_lines(session_path).await?;

        let messages = lines
            .into_iter()
            .filter_map(|line| Self::convert_to_message(line))
            .collect();

        Ok(messages)
    }

    /// Parse all lines from a session JSONL file
    ///
    /// Returns raw SessionLine structs in chronological order.
    /// Used internally by parse_conversation and by TUI for legacy display.
    pub async fn parse_session_lines(session_path: &Path) -> Result<Vec<SessionLine>, CoreError> {
        let file = File::open(session_path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::FileNotFound {
                    path: session_path.to_path_buf(),
                }
            } else {
                CoreError::FileRead {
                    path: session_path.to_path_buf(),
                    source: e,
                }
            }
        })?;

        let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer
        let mut lines_stream = reader.lines();
        let mut messages = Vec::new();
        let mut line_num = 0;

        while let Some(line) = lines_stream
            .next_line()
            .await
            .map_err(|e| CoreError::FileRead {
                path: session_path.to_path_buf(),
                source: e,
            })?
        {
            line_num += 1;

            // Circuit breaker: prevent infinite loops on malformed files
            if line_num > MAX_LINES {
                warn!(
                    path = %session_path.display(),
                    "Session file exceeds MAX_LINES ({}), stopping parse",
                    MAX_LINES
                );
                break;
            }

            // OOM protection: skip oversized lines
            if line.len() > MAX_LINE_SIZE {
                warn!(
                    path = %session_path.display(),
                    line_num,
                    size = line.len(),
                    "Skipping oversized line (>10MB)"
                );
                continue;
            }

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Parse JSON line
            match serde_json::from_str::<SessionLine>(&line) {
                Ok(session_line) => {
                    messages.push(session_line);
                }
                Err(e) => {
                    // Log but don't fail on malformed lines (graceful degradation)
                    warn!(
                        path = %session_path.display(),
                        line_num,
                        error = %e,
                        "Failed to parse session line, skipping"
                    );
                    continue;
                }
            }
        }

        Ok(messages)
    }

    /// Convert SessionLine to ConversationMessage
    ///
    /// Extracts role, content, timestamp, model from SessionLine.
    /// Returns None for non-message events (file-history-snapshot, session_end).
    fn convert_to_message(line: SessionLine) -> Option<ConversationMessage> {
        // Only convert user/assistant message types
        let role = match line.line_type.as_str() {
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "system" => MessageRole::System,
            _ => {
                // Skip non-message events
                trace!(line_type = %line.line_type, "Skipping non-message event");
                return None;
            }
        };

        // Extract content from message (handle both String and Array formats)
        let content = line.message.as_ref().and_then(|msg| {
            msg.content.as_ref().and_then(|c| match c {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Array(blocks) => {
                    // Extract text from content blocks (format: [{type: "text", text: "..."}])
                    let text = blocks
                        .iter()
                        .filter_map(|block| {
                            if let Some("text") = block.get("type").and_then(|t| t.as_str()) {
                                block.get("text").and_then(|t| t.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    if text.is_empty() {
                        None
                    } else {
                        Some(text)
                    }
                }
                _ => None,
            })
        });

        // Skip messages without content
        let content = content.unwrap_or_default();
        if content.is_empty() {
            trace!(role = ?role, "Skipping message with empty content");
            return None;
        }

        // Extract tokens (prefer message.usage over root usage)
        let tokens = line
            .message
            .as_ref()
            .and_then(|m| m.usage.clone())
            .or(line.usage);

        // Extract tool calls from message
        let tool_calls = line
            .message
            .as_ref()
            .and_then(|m| m.tool_calls.as_ref())
            .map(|calls| {
                calls
                    .iter()
                    .filter_map(|call| Self::parse_tool_call(call))
                    .collect()
            })
            .unwrap_or_default();

        // Extract tool results from message
        let tool_results = line
            .message
            .as_ref()
            .and_then(|m| m.tool_results.as_ref())
            .map(|results| {
                results
                    .iter()
                    .filter_map(|result| Self::parse_tool_result(result))
                    .collect()
            })
            .unwrap_or_default();

        Some(ConversationMessage {
            role,
            content,
            timestamp: line.timestamp,
            model: line.model,
            tokens,
            tool_calls,
            tool_results,
        })
    }

    /// Parse a tool call from JSON value
    fn parse_tool_call(value: &serde_json::Value) -> Option<crate::models::ToolCall> {
        let name = value.get("name")?.as_str()?.to_string();
        let id = value.get("id")?.as_str()?.to_string();
        let input = value.get("input")?.clone();

        Some(crate::models::ToolCall { name, id, input })
    }

    /// Parse a tool result from JSON value
    fn parse_tool_result(value: &serde_json::Value) -> Option<crate::models::ToolResult> {
        let tool_call_id = value.get("tool_use_id")?.as_str()?.to_string();
        let is_error = value
            .get("is_error")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let content = value
            .get("content")
            .and_then(|c| match c {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Array(blocks) => {
                    // Extract text from content blocks
                    let text = blocks
                        .iter()
                        .filter_map(|block| {
                            if let Some("text") = block.get("type").and_then(|t| t.as_str()) {
                                block.get("text").and_then(|t| t.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Some(text)
                }
                _ => None,
            })
            .unwrap_or_default();

        Some(crate::models::ToolResult {
            tool_call_id,
            is_error,
            content,
        })
    }

    /// Filter messages to only user/assistant interactions (legacy method)
    ///
    /// Removes system events like "file-history-snapshot", "session_end", etc.
    /// Useful for replay viewer to focus on conversation flow.
    ///
    /// NOTE: Prefer parse_conversation() for new code, which returns ConversationMessage.
    pub fn filter_messages(lines: Vec<SessionLine>) -> Vec<SessionLine> {
        lines
            .into_iter()
            .filter(|line| {
                matches!(
                    line.line_type.as_str(),
                    "user" | "assistant" | "tool_use" | "tool_result"
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProjectId;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_parse_empty_file() {
        // Create temp empty file
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.jsonl");
        tokio::fs::write(&file_path, "").await.unwrap();

        let result = SessionContentParser::parse_session_lines(&file_path).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_parse_malformed_json() {
        // Create temp file with malformed JSON
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("malformed.jsonl");
        tokio::fs::write(&file_path, "not json\n{\"type\":\"user\"}\n")
            .await
            .unwrap();

        let result = SessionContentParser::parse_session_lines(&file_path).await;
        assert!(result.is_ok());
        // Should skip malformed line, parse valid one
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_parse_conversation_full() {
        // Create temp file with user + assistant messages
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "file-history-snapshot", "timestamp": "2025-01-15T10:00:00Z"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "test-123", "timestamp": "2025-01-15T10:01:00Z", "message": {{"content": "Hello, can you help me?"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "timestamp": "2025-01-15T10:02:00Z", "message": {{"content": "Sure! How can I help?", "usage": {{"input_tokens": 100, "output_tokens": 50}}}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 150, "messageCount": 2}}}}"#
        )
        .unwrap();

        let metadata =
            SessionMetadata::from_path(file.path().to_path_buf(), ProjectId::from("test-project"));

        let messages = SessionContentParser::parse_conversation(file.path(), metadata)
            .await
            .unwrap();

        // Should extract 2 messages (user + assistant), skip file-history-snapshot + summary
        assert_eq!(messages.len(), 2);

        assert_eq!(messages[0].role, MessageRole::User);
        assert!(messages[0].content.contains("Hello"));
        assert!(messages[0].timestamp.is_some());

        assert_eq!(messages[1].role, MessageRole::Assistant);
        assert!(messages[1].content.contains("Sure"));
        assert_eq!(
            messages[1].model,
            Some("claude-sonnet-4-5-20250929".to_string())
        );
        assert!(messages[1].tokens.is_some());
        assert_eq!(messages[1].tokens.as_ref().unwrap().input_tokens, 100);
    }

    #[test]
    fn test_filter_messages() {
        let lines = vec![
            SessionLine {
                line_type: "user".to_string(),
                ..Default::default()
            },
            SessionLine {
                line_type: "file-history-snapshot".to_string(),
                ..Default::default()
            },
            SessionLine {
                line_type: "assistant".to_string(),
                ..Default::default()
            },
            SessionLine {
                line_type: "session_end".to_string(),
                ..Default::default()
            },
        ];

        let filtered = SessionContentParser::filter_messages(lines);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].line_type, "user");
        assert_eq!(filtered[1].line_type, "assistant");
    }
}
