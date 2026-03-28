//! Parser for OpenCode (SST) sessions stored in SQLite.
//!
//! OpenCode stores session data in a SQLite database:
//!   macOS:   ~/Library/Application Support/OpenCode/opencode.db
//!   Linux:   ~/.local/share/opencode/opencode.db
//!   Windows: %APPDATA%\OpenCode\opencode.db
//!
//! Relevant tables:
//!   Session(id TEXT, title TEXT, time_created INTEGER, time_updated INTEGER)
//!   Message(id TEXT, session_id TEXT, role TEXT, model_id TEXT, time_created INTEGER)

use crate::models::session::{ProjectId, SessionId, SessionMetadata, SourceTool};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

pub struct OpenCodeParser;

impl OpenCodeParser {
    /// Returns the default OpenCode database path for the current OS.
    pub fn default_path() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|h| {
                h.join("Library")
                    .join("Application Support")
                    .join("OpenCode")
                    .join("opencode.db")
            })
        }
        #[cfg(target_os = "linux")]
        {
            dirs::data_dir().map(|d| d.join("opencode").join("opencode.db"))
        }
        #[cfg(target_os = "windows")]
        {
            dirs::data_dir().map(|d| d.join("OpenCode").join("opencode.db"))
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }

    /// Scan all OpenCode sessions from the given SQLite database path.
    ///
    /// Returns an empty `Vec` on any error — graceful degradation.
    pub fn scan(db_path: &Path) -> Vec<SessionMetadata> {
        match Self::scan_inner(db_path) {
            Ok(sessions) => {
                debug!(count = sessions.len(), "OpenCode sessions scanned");
                sessions
            }
            Err(e) => {
                warn!(path = %db_path.display(), error = %e, "Failed to scan OpenCode sessions");
                Vec::new()
            }
        }
    }

    fn scan_inner(db_path: &Path) -> anyhow::Result<Vec<SessionMetadata>> {
        use anyhow::Context;
        use rusqlite::Connection;

        let conn = Connection::open(db_path)
            .with_context(|| format!("Cannot open OpenCode db at {}", db_path.display()))?;

        // Check which tables exist — schema may vary across OpenCode versions.
        let has_message_table = table_exists(&conn, "Message")?;

        let mut sessions = Vec::new();

        if has_message_table {
            // Full query joining Session and Message
            let mut stmt = conn
                .prepare(
                    "SELECT s.id, s.time_created, s.time_updated,
                            COUNT(m.id) AS msg_count,
                            GROUP_CONCAT(DISTINCT m.model_id) AS models
                     FROM Session s
                     LEFT JOIN Message m ON m.session_id = s.id
                     GROUP BY s.id",
                )
                .context("Failed to prepare OpenCode session+message query")?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(OpenCodeRow {
                        id: row.get::<_, String>(0)?,
                        time_created: row.get::<_, Option<i64>>(1)?,
                        time_updated: row.get::<_, Option<i64>>(2)?,
                        msg_count: row.get::<_, Option<i64>>(3)?,
                        models: row.get::<_, Option<String>>(4)?,
                    })
                })
                .context("Failed to query OpenCode sessions")?;

            for row in rows {
                match row {
                    Ok(r) => sessions.push(row_to_metadata(r)),
                    Err(e) => warn!(error = %e, "Skipping malformed OpenCode row"),
                }
            }
        } else {
            // Fallback: Session table only
            let mut stmt = conn
                .prepare("SELECT id, time_created, time_updated FROM Session")
                .context("Failed to prepare OpenCode session-only query")?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(OpenCodeRow {
                        id: row.get::<_, String>(0)?,
                        time_created: row.get::<_, Option<i64>>(1)?,
                        time_updated: row.get::<_, Option<i64>>(2)?,
                        msg_count: None,
                        models: None,
                    })
                })
                .context("Failed to query OpenCode sessions (fallback)")?;

            for row in rows {
                match row {
                    Ok(r) => sessions.push(row_to_metadata(r)),
                    Err(e) => warn!(error = %e, "Skipping malformed OpenCode row (fallback)"),
                }
            }
        }

        Ok(sessions)
    }
}

struct OpenCodeRow {
    id: String,
    time_created: Option<i64>,
    time_updated: Option<i64>,
    msg_count: Option<i64>,
    models: Option<String>,
}

/// Convert a Unix timestamp (ms or s) to a UTC `DateTime`.
///
/// OpenCode may store millis or seconds — values above 1e12 are treated as millis.
fn unix_to_datetime(ts: i64) -> chrono::DateTime<chrono::Utc> {
    use chrono::{DateTime, Utc};
    let secs = if ts > 1_000_000_000_000 {
        ts / 1000
    } else {
        ts
    };
    DateTime::<Utc>::from_timestamp(secs, 0).unwrap_or_else(Utc::now)
}

fn row_to_metadata(row: OpenCodeRow) -> SessionMetadata {
    let first_ts = row.time_created.map(unix_to_datetime);
    let last_ts = row.time_updated.map(unix_to_datetime);
    let msg_count = row.msg_count.unwrap_or(0).max(0) as u64;
    let models: Vec<String> = row
        .models
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    // Use a sentinel path so TUI does not try to open it as a JSONL file.
    let sentinel_path = std::path::PathBuf::from(format!("opencode://{}", row.id));

    let mut meta =
        SessionMetadata::from_path(sentinel_path, ProjectId::from("opencode://sessions"));
    meta.id = SessionId::new(format!("opencode:{}", row.id));
    meta.first_timestamp = first_ts;
    meta.last_timestamp = last_ts;
    meta.message_count = msg_count;
    meta.models_used = models;
    meta.source_tool = SourceTool::OpenCode;
    meta
}

/// Check whether a given table exists in the SQLite database.
fn table_exists(conn: &rusqlite::Connection, table: &str) -> anyhow::Result<bool> {
    use anyhow::Context;
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            rusqlite::params![table],
            |row| row.get(0),
        )
        .context("Failed to query sqlite_master")?;
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_to_datetime_millis() {
        // 2024-01-15 00:00:00 UTC in milliseconds
        let ts_ms: i64 = 1_705_276_800_000;
        let dt = unix_to_datetime(ts_ms);
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn test_unix_to_datetime_seconds() {
        // 2024-01-15 00:00:00 UTC in seconds
        let ts_s: i64 = 1705276800;
        let dt = unix_to_datetime(ts_s);
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn test_scan_nonexistent_returns_empty() {
        let sessions = OpenCodeParser::scan(Path::new("/nonexistent/path/opencode.db"));
        assert!(sessions.is_empty());
    }
}
