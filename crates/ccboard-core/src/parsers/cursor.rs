//! Parser for Cursor AI chat sessions stored in workspace-scoped SQLite databases.
//!
//! Cursor stores chat data in per-workspace SQLite files:
//!   macOS: ~/Library/Application Support/Cursor/User/workspaceStorage/*/state.vscdb
//!   Linux: ~/.config/Cursor/User/workspaceStorage/*/state.vscdb
//!   Windows: %APPDATA%\Cursor\User\workspaceStorage\*\state.vscdb
//!
//! Each database has an `ItemTable` with `key TEXT, value TEXT`.  The key
//! `workbench.panel.aichat.view.aichat.chatdata` holds a JSON blob:
//!   { "tabs": [ { "bubbles": [ { "bubbleId": "...", "type": 1|2,
//!                                "createdAt": "2024-...", "text": "..." } ] } ] }

use crate::models::session::{ProjectId, SessionId, SessionMetadata, SourceTool};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

const CHAT_KEY: &str = "workbench.panel.aichat.view.aichat.chatdata";

pub struct CursorParser;

impl CursorParser {
    /// Returns the Cursor `workspaceStorage` directory for the current OS.
    pub fn default_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|h| {
                h.join("Library")
                    .join("Application Support")
                    .join("Cursor")
                    .join("User")
                    .join("workspaceStorage")
            })
        }
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir().map(|c| c.join("Cursor").join("User").join("workspaceStorage"))
        }
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|c| c.join("Cursor").join("User").join("workspaceStorage"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    /// Scan all `state.vscdb` files under `workspace_storage_dir`.
    ///
    /// Returns an empty `Vec` on top-level failure; individual db errors are
    /// logged and skipped.
    pub fn scan(workspace_storage_dir: &Path) -> Vec<SessionMetadata> {
        let vscdb_files = match find_vscdb_files(workspace_storage_dir) {
            Ok(f) => f,
            Err(e) => {
                warn!(
                    path = %workspace_storage_dir.display(),
                    error = %e,
                    "Cannot enumerate Cursor workspaceStorage"
                );
                return Vec::new();
            }
        };

        let mut all_sessions = Vec::new();

        for db_path in &vscdb_files {
            match extract_sessions_from_db(db_path) {
                Ok(sessions) => all_sessions.extend(sessions),
                Err(e) => {
                    warn!(path = %db_path.display(), error = %e, "Skipping Cursor db");
                }
            }
        }

        debug!(count = all_sessions.len(), "Cursor sessions scanned");
        all_sessions
    }
}

/// Recursively find all `state.vscdb` files one level deep inside `dir`
/// (workspaceStorage/<hash>/state.vscdb).
fn find_vscdb_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    use anyhow::Context;

    let mut files = Vec::new();
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("Cannot read dir {}", dir.display()))?;

    for entry in entries.flatten() {
        let workspace_dir = entry.path();
        if !workspace_dir.is_dir() {
            continue;
        }
        let db = workspace_dir.join("state.vscdb");
        if db.exists() {
            files.push(db);
        }
    }

    Ok(files)
}

/// Open a single `state.vscdb` and return one `SessionMetadata` per chat tab.
fn extract_sessions_from_db(db_path: &Path) -> anyhow::Result<Vec<SessionMetadata>> {
    use anyhow::Context;
    use rusqlite::Connection;

    let conn = Connection::open(db_path)
        .with_context(|| format!("Cannot open Cursor db at {}", db_path.display()))?;

    // ItemTable may not exist (older Cursor versions use different storage).
    let table_ok: bool = {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='ItemTable'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count > 0
    };

    if !table_ok {
        return Ok(Vec::new());
    }

    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?1",
            rusqlite::params![CHAT_KEY],
            |row| row.get(0),
        )
        .ok();

    let json_str = match value {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    parse_chat_json(&json_str, db_path).context("Failed to parse Cursor chat JSON")
}

/// Parse the chat JSON blob and build `SessionMetadata` for each tab.
fn parse_chat_json(json_str: &str, db_path: &Path) -> anyhow::Result<Vec<SessionMetadata>> {
    let root: serde_json::Value = serde_json::from_str(json_str)?;

    let tabs = match root.get("tabs").and_then(|v| v.as_array()) {
        Some(t) => t,
        None => return Ok(Vec::new()),
    };

    // Derive a stable workspace identifier from the db directory name.
    let workspace_id = db_path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let mut sessions = Vec::new();

    for (tab_idx, tab) in tabs.iter().enumerate() {
        let bubbles = match tab.get("bubbles").and_then(|v| v.as_array()) {
            Some(b) if !b.is_empty() => b,
            _ => continue,
        };

        let message_count = bubbles.len() as u64;

        // Collect timestamps from bubbles.
        let mut timestamps: Vec<chrono::DateTime<chrono::Utc>> = bubbles
            .iter()
            .filter_map(|b| {
                b.get("createdAt")
                    .and_then(|v| v.as_str())
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .collect();

        timestamps.sort();

        let first_timestamp = timestamps.first().copied();
        let last_timestamp = timestamps.last().copied();

        // Preview from the first user bubble (type == 1).
        let first_user_message = bubbles
            .iter()
            .find(|b| b.get("type").and_then(|v| v.as_i64()) == Some(1))
            .and_then(|b| b.get("text").and_then(|v| v.as_str()))
            .map(|s| {
                let truncated: String = s.chars().take(200).collect();
                truncated
            });

        let session_id = SessionId::new(format!("cursor:{}:{}", workspace_id, tab_idx));
        let sentinel_path = PathBuf::from(format!("cursor://{}:{}", workspace_id, tab_idx));

        let mut meta =
            SessionMetadata::from_path(sentinel_path, ProjectId::from("cursor://sessions"));
        meta.id = session_id;
        meta.first_timestamp = first_timestamp;
        meta.last_timestamp = last_timestamp;
        meta.message_count = message_count;
        meta.first_user_message = first_user_message;
        meta.source_tool = SourceTool::Cursor;

        sessions.push(meta);
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chat_json_empty_tabs() {
        let json = r#"{"tabs": []}"#;
        let result = parse_chat_json(json, Path::new("/fake/state.vscdb"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_chat_json_with_tab() {
        let json = r#"{
            "tabs": [{
                "bubbles": [
                    {"bubbleId": "1", "type": 1, "createdAt": "2024-03-01T10:00:00Z", "text": "Hello"},
                    {"bubbleId": "2", "type": 2, "createdAt": "2024-03-01T10:01:00Z", "text": "Hi back"}
                ]
            }]
        }"#;
        let result = parse_chat_json(json, Path::new("/fake/state.vscdb"));
        assert!(result.is_ok());
        let sessions = result.unwrap();
        assert_eq!(sessions.len(), 1);
        let s = &sessions[0];
        assert_eq!(s.message_count, 2);
        assert_eq!(s.source_tool, SourceTool::Cursor);
        assert_eq!(s.first_user_message.as_deref(), Some("Hello"));
    }

    #[test]
    fn test_scan_nonexistent_returns_empty() {
        let sessions = CursorParser::scan(Path::new("/nonexistent/cursor/workspaceStorage"));
        assert!(sessions.is_empty());
    }
}
