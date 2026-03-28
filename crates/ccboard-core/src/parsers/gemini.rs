//! Parser for Gemini CLI session files
//!
//! Reads `~/.gemini/tmp/{projectHash}/chats/session-*.json` files.
//!
//! Format (stable since Gemini CLI v0.28+):
//! ```json
//! {
//!   "sessionId": "UUID",
//!   "projectHash": "sha256-hex",
//!   "startTime": "ISO-8601",
//!   "lastUpdated": "ISO-8601",
//!   "messages": [
//!     { "id": "UUID", "timestamp": "ISO-8601", "type": "user|gemini|info|error|tool_call|tool_result", "content": "..." }
//!   ]
//! }
//! ```
//!
//! Limitations: no token counts, no cost data (not stored locally by Gemini CLI).

use crate::error::LoadReport;
use crate::models::{ProjectId, SessionId, SessionMetadata};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, warn};

/// Source tool identifier for Gemini sessions
pub const GEMINI_SOURCE: &str = "gemini";

/// Raw Gemini session JSON structure
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiSession {
    session_id: String,
    project_hash: String,
    start_time: Option<DateTime<Utc>>,
    last_updated: Option<DateTime<Utc>>,
    messages: Vec<GeminiMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: Option<serde_json::Value>,
    #[serde(default)]
    display_content: Option<serde_json::Value>,
}

/// Parser for Gemini CLI sessions
pub struct GeminiParser;

impl GeminiParser {
    /// Scan `~/.gemini/tmp/` and return all sessions as `SessionMetadata`
    pub fn scan_all(gemini_home: &Path, report: &mut LoadReport) -> Vec<SessionMetadata> {
        let tmp_dir = gemini_home.join("tmp");
        if !tmp_dir.exists() {
            debug!("No Gemini tmp dir found at {}", tmp_dir.display());
            return Vec::new();
        }

        let mut sessions = Vec::new();

        let project_dirs = match std::fs::read_dir(&tmp_dir) {
            Ok(d) => d,
            Err(e) => {
                warn!("Failed to read Gemini tmp dir: {}", e);
                return Vec::new();
            }
        };

        for entry in project_dirs.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let chats_dir = path.join("chats");
            if !chats_dir.exists() {
                continue;
            }

            let chat_files = match std::fs::read_dir(&chats_dir) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to read Gemini chats dir {}: {}", chats_dir.display(), e);
                    continue;
                }
            };

            for file_entry in chat_files.flatten() {
                let file_path = file_entry.path();
                if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }

                match Self::parse_session_file(&file_path) {
                    Ok(meta) => sessions.push(meta),
                    Err(e) => {
                        warn!("Failed to parse Gemini session {}: {}", file_path.display(), e);
                        report.sessions_failed += 1;
                    }
                }
            }
        }

        report.sessions_scanned += sessions.len();
        debug!("Gemini: scanned {} sessions", sessions.len());
        sessions
    }

    /// Parse a single Gemini session JSON file into `SessionMetadata`
    fn parse_session_file(path: &Path) -> Result<SessionMetadata> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;

        let session: GeminiSession = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

        let file_size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        // Count only real conversation messages (user + gemini), skip info/error
        let message_count = session
            .messages
            .iter()
            .filter(|m| m.msg_type == "user" || m.msg_type == "gemini")
            .count() as u64;

        // Extract first user message for preview
        let first_user_message = session
            .messages
            .iter()
            .find(|m| m.msg_type == "user")
            .and_then(|m| {
                // Use displayContent if available (shorter, user-typed), else content
                let val = m.display_content.as_ref().or(m.content.as_ref())?;
                extract_text_preview(val, 200)
            });

        // project_path: use the project hash as identifier (no reverse mapping available)
        let project_path = ProjectId::from(format!("gemini:{}", &session.project_hash[..8]));

        Ok(SessionMetadata {
            id: SessionId::new(session.session_id.clone()),
            source_tool: Some(GEMINI_SOURCE.to_string()),
            file_path: path.to_path_buf(),
            project_path,
            first_timestamp: session.start_time,
            last_timestamp: session.last_updated,
            message_count,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec!["gemini".to_string()],
            file_size_bytes,
            first_user_message,
            has_subagents: false,
            duration_seconds: compute_duration(session.start_time, session.last_updated),
            branch: None,
            tool_usage: std::collections::HashMap::new(),
            tool_token_usage: std::collections::HashMap::new(),
        })
    }

    /// Check if Gemini CLI is installed (i.e. `~/.gemini/tmp/` exists)
    pub fn is_available(gemini_home: &Path) -> bool {
        gemini_home.join("tmp").exists()
    }
}

/// Extract text preview from a Gemini message content value (String or Array)
fn extract_text_preview(val: &serde_json::Value, max_len: usize) -> Option<String> {
    let text = match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|item| item.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
            .collect::<Vec<_>>()
            .join(" "),
        _ => return None,
    };

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.len() <= max_len {
        Some(trimmed.to_string())
    } else {
        let truncated = &trimmed[..max_len];
        // Truncate at last space to avoid cutting words
        let end = truncated.rfind(' ').unwrap_or(max_len);
        Some(format!("{}…", &trimmed[..end]))
    }
}

fn compute_duration(
    start: Option<DateTime<Utc>>,
    end: Option<DateTime<Utc>>,
) -> Option<u64> {
    let s = start?;
    let e = end?;
    let diff = e.signed_duration_since(s);
    if diff.num_seconds() > 0 {
        Some(diff.num_seconds() as u64)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_session(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    const SAMPLE_SESSION: &str = r#"{
  "sessionId": "test-session-id",
  "projectHash": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
  "startTime": "2026-01-01T10:00:00Z",
  "lastUpdated": "2026-01-01T10:30:00Z",
  "messages": [
    { "id": "m1", "timestamp": "2026-01-01T10:00:00Z", "type": "info", "content": "Session started" },
    { "id": "m2", "timestamp": "2026-01-01T10:01:00Z", "type": "user", "content": [{"text": "Hello Gemini"}], "displayContent": [{"text": "Hello Gemini"}] },
    { "id": "m3", "timestamp": "2026-01-01T10:02:00Z", "type": "gemini", "content": "Hi there!" }
  ]
}"#;

    #[test]
    fn test_parse_session_file() {
        let dir = tempdir().unwrap();
        let path = write_session(dir.path(), "session-2026-01-01T10-00-test.json", SAMPLE_SESSION);

        let meta = GeminiParser::parse_session_file(&path).unwrap();

        assert_eq!(meta.id, "test-session-id");
        assert_eq!(meta.source_tool, Some("gemini".to_string()));
        assert_eq!(meta.message_count, 2); // user + gemini, not info
        assert_eq!(meta.models_used, vec!["gemini"]);
        assert_eq!(meta.project_path.as_ref(), "gemini:abcdef12");
        assert_eq!(meta.input_tokens, 0);
        assert_eq!(meta.first_user_message.as_deref(), Some("Hello Gemini"));
        assert_eq!(meta.duration_seconds, Some(1800)); // 30 minutes
    }

    #[test]
    fn test_scan_all_structure() {
        let dir = tempdir().unwrap();

        // Create ~/.gemini/tmp/{hash}/chats/session.json structure
        let hash = "aaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaa7777bbbb8888";
        let chats_dir = dir.path().join("tmp").join(hash).join("chats");
        fs::create_dir_all(&chats_dir).unwrap();
        write_session(&chats_dir, "session-2026-01-01T10-00-abc.json", SAMPLE_SESSION);

        let mut report = LoadReport::default();
        let sessions = GeminiParser::scan_all(dir.path(), &mut report);

        assert_eq!(sessions.len(), 1);
        assert_eq!(report.sessions_scanned, 1);
        assert_eq!(report.sessions_failed, 0);
    }

    #[test]
    fn test_is_available() {
        let dir = tempdir().unwrap();
        assert!(!GeminiParser::is_available(dir.path()));

        fs::create_dir_all(dir.path().join("tmp")).unwrap();
        assert!(GeminiParser::is_available(dir.path()));
    }

    #[test]
    fn test_malformed_json_skipped() {
        let dir = tempdir().unwrap();
        let hash = "aaaa1111bbbb2222cccc3333dddd4444eeee5555ffff6666aaaa7777bbbb8888";
        let chats_dir = dir.path().join("tmp").join(hash).join("chats");
        fs::create_dir_all(&chats_dir).unwrap();

        write_session(&chats_dir, "session-good.json", SAMPLE_SESSION);
        write_session(&chats_dir, "session-bad.json", "{ invalid json }");

        let mut report = LoadReport::default();
        let sessions = GeminiParser::scan_all(dir.path(), &mut report);

        assert_eq!(sessions.len(), 1);
        assert_eq!(report.sessions_failed, 1);
    }
}
