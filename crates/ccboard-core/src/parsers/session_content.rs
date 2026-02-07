//! Full session content parser for message replay
//!
//! Parses entire JSONL session files to extract chronological message stream.
//! Used for session replay viewer in TUI.

use crate::models::SessionLine;
use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::warn;

/// Maximum line size in bytes (10MB) - OOM protection
const MAX_LINE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum lines to parse (circuit breaker for malformed/infinite files)
const MAX_LINES: usize = 50_000;

/// Parser for full session content
pub struct SessionContentParser;

impl SessionContentParser {
    /// Parse all lines from a session JSONL file
    ///
    /// Returns messages in chronological order (as they appear in file).
    /// Filters out non-message events (file-history-snapshot, etc) in the future if needed.
    ///
    /// # Performance
    /// - Streaming parse: doesn't load entire file into memory
    /// - Early termination: stops at MAX_LINES circuit breaker
    /// - Line size limit: skips lines >10MB to prevent OOM
    ///
    /// # Errors
    /// Returns CoreError::SessionParseFailed if file cannot be read or contains invalid JSON.
    pub async fn parse_session(session_path: &Path) -> Result<Vec<SessionLine>> {
        let file = File::open(session_path)
            .await
            .context("Failed to open session file")?;

        let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer
        let mut lines_stream = reader.lines();
        let mut messages = Vec::new();
        let mut line_num = 0;

        while let Some(line) = lines_stream
            .next_line()
            .await
            .context("Failed to read line from session file")?
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

    /// Filter messages to only user/assistant interactions
    ///
    /// Removes system events like "file-history-snapshot", "session_end", etc.
    /// Useful for replay viewer to focus on conversation flow.
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

    #[tokio::test]
    async fn test_parse_empty_file() {
        // Create temp empty file
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("empty.jsonl");
        tokio::fs::write(&file_path, "").await.unwrap();

        let result = SessionContentParser::parse_session(&file_path).await;
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

        let result = SessionContentParser::parse_session(&file_path).await;
        assert!(result.is_ok());
        // Should skip malformed line, parse valid one
        assert_eq!(result.unwrap().len(), 1);
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
