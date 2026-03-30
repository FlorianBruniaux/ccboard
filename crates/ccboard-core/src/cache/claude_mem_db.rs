//! Read-only reader for claude-mem's SQLite database (`~/.claude-mem/claude-mem.db`)
//!
//! claude-mem is an optional Claude Code plugin. This reader is purely
//! read-only (SQLITE_OPEN_READ_ONLY) and gracefully falls back to an empty
//! result set if the DB is absent, locked, or corrupted.

use crate::models::claude_mem::ClaudeMemObservation;
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags};
use std::path::PathBuf;
use tracing::warn;

/// Read-only access to the claude-mem observations database
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

    /// Load the most recent observations, newest first.
    /// Returns an empty Vec on any error (DB absent, locked, schema mismatch).
    pub fn load_recent(&self, limit: usize) -> Vec<ClaudeMemObservation> {
        match self.try_load_recent(limit) {
            Ok(obs) => obs,
            Err(e) => {
                warn!(
                    path = %self.db_path.display(),
                    error = %e,
                    "claude-mem DB read error — returning empty observations"
                );
                vec![]
            }
        }
    }

    fn try_load_recent(&self, limit: usize) -> Result<Vec<ClaudeMemObservation>> {
        let conn = Connection::open_with_flags(
            &self.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .with_context(|| format!("Failed to open claude-mem.db: {}", self.db_path.display()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, memory_session_id, type, title, narrative, project, created_at
                 FROM observations
                 ORDER BY created_at_epoch DESC
                 LIMIT ?1",
            )
            .context("Failed to prepare claude-mem query")?;

        let observations = stmt
            .query_map([limit as i64], |row| {
                Ok(ClaudeMemObservation {
                    id: row.get(0)?,
                    memory_session_id: row.get(1)?,
                    obs_type: row.get::<_, String>(2).unwrap_or_default(),
                    title: row.get(3)?,
                    narrative: row.get(4)?,
                    project: row.get::<_, String>(5).unwrap_or_default(),
                    created_at: row.get::<_, String>(6).unwrap_or_default(),
                })
            })
            .context("Failed to query claude-mem observations")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(observations)
    }
}
