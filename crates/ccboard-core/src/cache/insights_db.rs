//! SQLite-backed Brain insights database (`~/.ccboard/insights.db`)
//!
//! Separate from `session-metadata.db` to avoid CACHE_VERSION conflicts and allow
//! concurrent writes from bash hooks while Rust holds a read connection.
//!
//! Schema:
//! - insights: cross-session knowledge (progress, decisions, blockers, patterns, fixes, context)
//! - WAL mode for concurrent hook writes

use crate::models::insight::{Insight, InsightType};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;
use tracing::debug;

/// SQLite-backed Brain insights store (thread-safe)
pub struct InsightsDb {
    conn: Mutex<Connection>,
}

impl InsightsDb {
    /// Open (or create) the insights database at `<cache_dir>/insights.db`
    pub fn new(cache_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(cache_dir).with_context(|| {
            format!(
                "Failed to create cache directory: {}",
                cache_dir.display()
            )
        })?;

        let db_path = cache_dir.join("insights.db");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open insights database: {}", db_path.display()))?;

        // WAL for concurrent bash hook writes
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode on insights.db")?;

        conn.pragma_update(None, "foreign_keys", "ON")
            .context("Failed to enable foreign keys")?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS insights (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id  TEXT,
                project     TEXT NOT NULL,
                type        TEXT NOT NULL CHECK (type IN (
                                'progress','decision','blocked','pattern','fix','context')),
                content     TEXT NOT NULL,
                reasoning   TEXT,
                archived    INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_insights_project  ON insights(project);
            CREATE INDEX IF NOT EXISTS idx_insights_type     ON insights(type);
            CREATE INDEX IF NOT EXISTS idx_insights_created  ON insights(created_at);
            CREATE INDEX IF NOT EXISTS idx_insights_archived ON insights(archived);
            "#,
        )
        .context("Failed to initialize insights schema")?;

        debug!("Insights DB opened at {}", db_path.display());
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Insert a new insight. Returns the new row id.
    pub fn insert(
        &self,
        session_id: Option<&str>,
        project: &str,
        insight_type: InsightType,
        content: &str,
        reasoning: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        conn.execute(
            "INSERT INTO insights (session_id, project, type, content, reasoning)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, project, insight_type.as_str(), content, reasoning],
        )
        .context("Failed to insert insight")?;
        Ok(conn.last_insert_rowid())
    }

    /// List non-archived insights for a project, newest first
    pub fn list_for_project(&self, project: &str) -> Result<Vec<Insight>> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        self.query_insights(
            &conn,
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights
             WHERE project = ?1 AND archived = 0
             ORDER BY created_at DESC",
            params![project],
        )
    }

    /// List non-archived insights filtered by project and type
    pub fn list_by_type(&self, project: &str, insight_type: InsightType) -> Result<Vec<Insight>> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        self.query_insights(
            &conn,
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights
             WHERE project = ?1 AND type = ?2 AND archived = 0
             ORDER BY created_at DESC",
            params![project, insight_type.as_str()],
        )
    }

    /// List non-archived insights of a given type across all projects, newest first
    pub fn list_by_type_all(&self, insight_type: InsightType, limit: usize) -> Result<Vec<Insight>> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        self.query_insights(
            &conn,
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights
             WHERE type = ?1 AND archived = 0
             ORDER BY created_at DESC
             LIMIT ?2",
            params![insight_type.as_str(), limit as i64],
        )
    }

    /// List all non-archived insights across all projects, newest first (Brain tab overview)
    pub fn list_all(&self, limit: usize) -> Result<Vec<Insight>> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        self.query_insights(
            &conn,
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights
             WHERE archived = 0
             ORDER BY created_at DESC
             LIMIT ?1",
            params![limit as i64],
        )
    }

    /// Get the most recent insights of specific types for context injection
    pub fn recent_by_types(
        &self,
        project: &str,
        types: &[InsightType],
        limit: usize,
    ) -> Result<Vec<Insight>> {
        if types.is_empty() {
            return Ok(vec![]);
        }
        let placeholders: String = types
            .iter()
            .enumerate()
            .map(|(i, _)| format!("?{}", i + 2))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights
             WHERE project = ?1 AND type IN ({placeholders}) AND archived = 0
             ORDER BY created_at DESC
             LIMIT ?{}",
            types.len() + 2
        );

        let conn = self.conn.lock().expect("insights db mutex poisoned");
        let mut stmt = conn.prepare(&sql).context("Failed to prepare query")?;

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
        params_vec.push(Box::new(project.to_string()));
        for t in types {
            params_vec.push(Box::new(t.as_str().to_string()));
        }
        params_vec.push(Box::new(limit as i64));

        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let insights = stmt
            .query_map(params_refs.as_slice(), Self::row_to_insight)?
            .filter_map(|r| r.ok())
            .collect();

        Ok(insights)
    }

    /// Soft-delete an insight
    pub fn archive(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        conn.execute(
            "UPDATE insights SET archived = 1 WHERE id = ?1",
            params![id],
        )
        .context("Failed to archive insight")?;
        Ok(())
    }

    /// Get a single insight by id
    pub fn get(&self, id: i64) -> Result<Option<Insight>> {
        let conn = self.conn.lock().expect("insights db mutex poisoned");
        conn.query_row(
            "SELECT id, session_id, project, type, content, reasoning, archived, created_at
             FROM insights WHERE id = ?1",
            params![id],
            Self::row_to_insight,
        )
        .optional()
        .context("Failed to get insight")
    }

    // ------ helpers ------

    fn query_insights(
        &self,
        conn: &Connection,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> Result<Vec<Insight>> {
        let mut stmt = conn.prepare(sql).context("Failed to prepare query")?;
        let insights = stmt
            .query_map(params, Self::row_to_insight)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(insights)
    }

    fn row_to_insight(row: &rusqlite::Row<'_>) -> rusqlite::Result<Insight> {
        let type_str: String = row.get(3)?;
        let insight_type = InsightType::from_str(&type_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, e.into()))?;

        let archived_int: i64 = row.get(6)?;
        let created_str: String = row.get(7)?;
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .or_else(|_| {
                // SQLite datetime() produces "YYYY-MM-DD HH:MM:SS" without timezone
                chrono::NaiveDateTime::parse_from_str(&created_str, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| ndt.and_utc())
            })
            .unwrap_or_else(|_| chrono::Utc::now());

        Ok(Insight {
            id: row.get(0)?,
            session_id: row.get(1)?,
            project: row.get(2)?,
            insight_type,
            content: row.get(4)?,
            reasoning: row.get(5)?,
            archived: archived_int != 0,
            created_at,
        })
    }
}
