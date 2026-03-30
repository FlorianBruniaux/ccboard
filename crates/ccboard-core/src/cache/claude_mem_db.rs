//! Read-only reader for claude-mem's SQLite database (`~/.claude-mem/claude-mem.db`)
//!
//! claude-mem is an optional Claude Code plugin. This reader is purely
//! read-only (SQLITE_OPEN_READ_ONLY) and gracefully falls back to an empty
//! result set if the DB is absent, locked, or corrupted.

use crate::models::claude_mem::ClaudeMemSummary;
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use tracing::warn;

/// Read-only access to the claude-mem database
pub struct ClaudeMemDb {
    db_path: PathBuf,
}

impl ClaudeMemDb {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    /// Whether the database file exists on disk
    pub fn is_available(&self) -> bool {
        self.db_path.exists()
    }

    /// Load the most recent session summaries, newest first.
    /// Returns an empty Vec on any error (DB absent, locked, schema mismatch).
    pub fn load_recent_summaries(&self, limit: usize) -> Vec<ClaudeMemSummary> {
        match self.try_load_recent_summaries(limit) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    path = %self.db_path.display(),
                    error = %e,
                    "claude-mem DB read error — returning empty summaries"
                );
                vec![]
            }
        }
    }

    fn try_load_recent_summaries(&self, limit: usize) -> Result<Vec<ClaudeMemSummary>> {
        let conn = Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failed to open claude-mem.db: {}", self.db_path.display()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, memory_session_id, project, request, completed, next_steps,
                        files_edited, created_at
                 FROM session_summaries
                 ORDER BY created_at_epoch DESC
                 LIMIT ?1",
            )
            .context("Failed to prepare claude-mem summaries query")?;

        let summaries = stmt
            .query_map([limit as i64], |row| {
                Ok(ClaudeMemSummary {
                    id: row.get(0)?,
                    memory_session_id: row.get(1)?,
                    project: row.get::<_, String>(2).unwrap_or_default(),
                    request: row.get(3)?,
                    completed: row.get(4)?,
                    next_steps: row.get(5)?,
                    files_edited: row.get(6)?,
                    created_at: row.get::<_, String>(7).unwrap_or_default(),
                })
            })
            .context("Failed to query claude-mem summaries")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(summaries)
    }
}
