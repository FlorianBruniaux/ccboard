//! SQLite metadata cache for session files
//!
//! Caches session metadata with mtime-based invalidation for 90% startup speedup.
//! Also caches activity analysis (tool calls, alerts) per session on demand.
//!
//! Schema:
//! - session_metadata: parsed metadata + mtime + cache_version
//! - activity_cache: serialized ActivitySummary + mtime per session file
//! - activity_alerts: searchable alert records (severity/category) across all sessions
//! - Indexes: project, mtime, session_id, severity for fast queries
//!
//! Invalidation:
//! - File watcher detects modification → delete session + activity cache entries
//! - Startup: compare mtime → rescan if stale
//! - Startup: compare cache_version → auto-clear ALL tables if mismatch
//!
//! Cache Version History:
//! - v1: Initial version (pre-TokenUsage fix)
//! - v2: Fixed TokenUsage::total() to include cache_read_tokens + cache_write_tokens
//! - v3: Added token breakdown fields (input_tokens, output_tokens, cache_creation_tokens,
//!   cache_read_tokens) to SessionMetadata + real pricing calculation
//! - v4: Added branch field to SessionMetadata
//! - v5: Added activity_cache + activity_alerts tables for Phase 2 activity module
//! - v6: Added aggregate_stats table with triggers + FTS5 session_fts table
//! - v7: Added tool_token_usage field to SessionMetadata (Phase K analytics)

use crate::models::activity::ActivitySummary;
use crate::models::SessionMetadata;
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use tracing::{debug, warn};

/// Current cache version
///
/// **IMPORTANT**: Increment this version when changing how metadata is calculated:
/// - TokenUsage fields added/removed
/// - SessionMetadata structure changed
/// - Parsing logic modified (e.g., token accumulation)
///
/// This triggers automatic cache invalidation on startup, preventing stale data bugs.
/// All tables (session_metadata + activity) are cleared on version mismatch.
///
/// Version History:
/// - v1: Initial version
/// - v2: Fixed TokenUsage::total() calculation
/// - v3: Added token breakdown fields
/// - v4: Added branch field to SessionMetadata
/// - v5: Added activity_cache + activity_alerts tables
/// - v6: Added aggregate_stats table with triggers + FTS5 session_fts table
/// - v7: Added tool_token_usage field to SessionMetadata (Phase K analytics)
/// - v8: Added source_tool field to SessionMetadata (multi-LLM support)
const CACHE_VERSION: i32 = 8;

/// SQLite-based metadata cache (thread-safe)
pub struct MetadataCache {
    conn: Mutex<Connection>,
    #[allow(dead_code)]
    cache_path: PathBuf,
}

impl MetadataCache {
    /// Create or open cache database
    pub fn new(cache_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(cache_dir).with_context(|| {
            format!("Failed to create cache directory: {}", cache_dir.display())
        })?;

        let cache_path = cache_dir.join("session-metadata.db");
        let conn = Connection::open(&cache_path)
            .with_context(|| format!("Failed to open cache database: {}", cache_path.display()))?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode")?;

        // Initialize schema
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS cache_metadata (
                key TEXT PRIMARY KEY,
                value INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS session_metadata (
                path TEXT PRIMARY KEY,
                mtime INTEGER NOT NULL,
                project TEXT NOT NULL,
                session_id TEXT NOT NULL,
                first_timestamp TEXT,
                last_timestamp TEXT,
                message_count INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                models_used TEXT NOT NULL,
                has_subagents INTEGER NOT NULL,
                first_user_message TEXT,
                data BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_project ON session_metadata(project);
            CREATE INDEX IF NOT EXISTS idx_mtime ON session_metadata(mtime);
            CREATE INDEX IF NOT EXISTS idx_session_id ON session_metadata(session_id);

            CREATE TABLE IF NOT EXISTS activity_cache (
                session_path TEXT PRIMARY KEY,
                mtime INTEGER NOT NULL,
                session_id TEXT NOT NULL,
                tool_call_count INTEGER NOT NULL DEFAULT 0,
                alert_count INTEGER NOT NULL DEFAULT 0,
                data BLOB NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_activity_session_id ON activity_cache(session_id);
            CREATE INDEX IF NOT EXISTS idx_activity_mtime ON activity_cache(mtime);

            CREATE TABLE IF NOT EXISTS activity_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_path TEXT NOT NULL,
                severity TEXT NOT NULL,
                category TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                detail TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_alerts_session ON activity_alerts(session_path);
            CREATE INDEX IF NOT EXISTS idx_alerts_severity ON activity_alerts(severity);

            CREATE TABLE IF NOT EXISTS aggregate_stats (
                key   TEXT PRIMARY KEY,
                value INTEGER NOT NULL DEFAULT 0
            );

            INSERT OR IGNORE INTO aggregate_stats (key, value) VALUES
                ('total_sessions', 0),
                ('total_messages', 0);

            CREATE TRIGGER IF NOT EXISTS stats_ai
            AFTER INSERT ON session_metadata BEGIN
                UPDATE aggregate_stats SET value = value + 1 WHERE key = 'total_sessions';
                UPDATE aggregate_stats SET value = value + new.message_count WHERE key = 'total_messages';
            END;

            CREATE TRIGGER IF NOT EXISTS stats_ad
            AFTER DELETE ON session_metadata BEGIN
                UPDATE aggregate_stats SET value = MAX(0, value - 1) WHERE key = 'total_sessions';
                UPDATE aggregate_stats SET value = MAX(0, value - old.message_count) WHERE key = 'total_messages';
            END;

            CREATE VIRTUAL TABLE IF NOT EXISTS session_fts USING fts5(
                session_id  UNINDEXED,
                project     UNINDEXED,
                first_user_message,
                models_used,
                content='session_metadata',
                content_rowid='rowid',
                tokenize='unicode61'
            );

            CREATE TRIGGER IF NOT EXISTS session_fts_ai
            AFTER INSERT ON session_metadata BEGIN
                INSERT INTO session_fts(rowid, session_id, project, first_user_message, models_used)
                VALUES (new.rowid, new.session_id, new.project, new.first_user_message, new.models_used);
            END;

            CREATE TRIGGER IF NOT EXISTS session_fts_ad
            AFTER DELETE ON session_metadata BEGIN
                INSERT INTO session_fts(session_fts, rowid, session_id, project, first_user_message, models_used)
                VALUES ('delete', old.rowid, old.session_id, old.project, old.first_user_message, old.models_used);
            END;

            CREATE TRIGGER IF NOT EXISTS session_fts_au
            AFTER UPDATE ON session_metadata BEGIN
                INSERT INTO session_fts(session_fts, rowid, session_id, project, first_user_message, models_used)
                VALUES ('delete', old.rowid, old.session_id, old.project, old.first_user_message, old.models_used);
                INSERT INTO session_fts(rowid, session_id, project, first_user_message, models_used)
                VALUES (new.rowid, new.session_id, new.project, new.first_user_message, new.models_used);
            END;
            "#,
        )
        .context("Failed to create schema")?;

        // Check cache version and auto-invalidate if mismatch
        let stored_version: Option<i32> = conn
            .query_row(
                "SELECT value FROM cache_metadata WHERE key = 'version'",
                [],
                |row| row.get(0),
            )
            .optional()
            .context("Failed to query cache version")?;

        match stored_version {
            Some(v) if v != CACHE_VERSION => {
                warn!(
                    stored = v,
                    current = CACHE_VERSION,
                    "Cache version mismatch detected, clearing stale cache"
                );

                // Clear all session and activity entries
                conn.execute("DELETE FROM session_metadata", [])
                    .context("Failed to clear stale session cache")?;
                conn.execute("DELETE FROM activity_cache", [])
                    .context("Failed to clear stale activity cache")?;
                conn.execute("DELETE FROM activity_alerts", [])
                    .context("Failed to clear stale activity alerts")?;
                conn.execute("DELETE FROM aggregate_stats", [])
                    .context("Failed to clear stale aggregate stats")?;
                conn.execute(
                    "INSERT OR IGNORE INTO aggregate_stats (key, value) VALUES ('total_sessions', 0)",
                    [],
                )
                .context("Failed to reinitialize total_sessions")?;
                conn.execute(
                    "INSERT OR IGNORE INTO aggregate_stats (key, value) VALUES ('total_messages', 0)",
                    [],
                )
                .context("Failed to reinitialize total_messages")?;

                // Update version
                conn.execute(
                    "INSERT OR REPLACE INTO cache_metadata (key, value) VALUES ('version', ?)",
                    params![CACHE_VERSION],
                )
                .context("Failed to update cache version")?;

                debug!("Cache cleared and version updated to {}", CACHE_VERSION);
            }
            None => {
                // First run, set version
                conn.execute(
                    "INSERT INTO cache_metadata (key, value) VALUES ('version', ?)",
                    params![CACHE_VERSION],
                )
                .context("Failed to initialize cache version")?;

                debug!("Cache version initialized to {}", CACHE_VERSION);
            }
            Some(_) => {
                debug!("Cache version {} matches current", CACHE_VERSION);
            }
        }

        let cache = Self {
            conn: Mutex::new(conn),
            cache_path: cache_path.clone(),
        };

        debug!(path = %cache_path.display(), "Metadata cache initialized");

        Ok(cache)
    }

    /// Get cached metadata if fresh, otherwise None
    pub fn get(&self, path: &Path, current_mtime: SystemTime) -> Result<Option<SessionMetadata>> {
        let path_str = path.to_string_lossy();
        let mtime_secs = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_secs();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM session_metadata WHERE path = ? AND mtime = ?",
                params![path_str.as_ref(), mtime_secs as i64],
                |row| row.get(0),
            )
            .optional()
            .context("Failed to query cache")?;

        match result {
            Some(bytes) => {
                let meta: SessionMetadata = bincode::deserialize(&bytes)
                    .context("Failed to deserialize cached metadata")?;
                debug!(path = %path.display(), "Cache hit");
                Ok(Some(meta))
            }
            None => {
                debug!(path = %path.display(), "Cache miss");
                Ok(None)
            }
        }
    }

    /// Store metadata in cache
    pub fn put(&self, path: &Path, meta: &SessionMetadata, mtime: SystemTime) -> Result<()> {
        let path_str = path.to_string_lossy();
        let mtime_secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_secs();

        let data = bincode::serialize(meta).context("Failed to serialize metadata")?;

        // Extract searchable fields
        let models_used =
            serde_json::to_string(&meta.models_used).context("Failed to serialize models")?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        // 1. Try to insert new row (only fires stats_ai trigger on NEW rows)
        conn.execute(
            r#"
            INSERT OR IGNORE INTO session_metadata
            (path, mtime, project, session_id, first_timestamp, last_timestamp,
             message_count, total_tokens, models_used, has_subagents, first_user_message, data)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                path_str.as_ref(),
                mtime_secs as i64,
                meta.project_path.as_str(),
                meta.id.as_str(),
                meta.first_timestamp.as_ref().map(|t| t.to_rfc3339()),
                meta.last_timestamp.as_ref().map(|t| t.to_rfc3339()),
                meta.message_count as i64,
                meta.total_tokens as i64,
                models_used.as_str(),
                if meta.has_subagents { 1 } else { 0 },
                &meta.first_user_message,
                &data,
            ],
        )
        .context("Failed to insert metadata")?;

        // 2. Update if row already existed (no INSERT trigger fires, no stats double-count)
        conn.execute(
            r#"
            UPDATE session_metadata
            SET mtime = ?, project = ?, session_id = ?, first_timestamp = ?, last_timestamp = ?,
                message_count = ?, total_tokens = ?, models_used = ?, has_subagents = ?,
                first_user_message = ?, data = ?
            WHERE path = ? AND mtime != ?
            "#,
            params![
                mtime_secs as i64,
                meta.project_path.as_str(),
                meta.id.as_str(),
                meta.first_timestamp.as_ref().map(|t| t.to_rfc3339()),
                meta.last_timestamp.as_ref().map(|t| t.to_rfc3339()),
                meta.message_count as i64,
                meta.total_tokens as i64,
                models_used.as_str(),
                if meta.has_subagents { 1 } else { 0 },
                &meta.first_user_message,
                &data,
                path_str.as_ref(),
                mtime_secs as i64,
            ],
        )
        .context("Failed to update metadata")?;

        debug!(path = %path.display(), "Metadata cached");
        Ok(())
    }

    /// Invalidate cache entry for a path
    pub fn invalidate(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        conn.execute(
            "DELETE FROM session_metadata WHERE path = ?",
            params![path_str.as_ref()],
        )
        .context("Failed to delete cache entry")?;

        debug!(path = %path.display(), "Cache entry invalidated");
        Ok(())
    }

    /// Get all cached paths for a project
    pub fn get_project_paths(&self, project: &str) -> Result<Vec<PathBuf>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT path FROM session_metadata WHERE project = ?")
            .context("Failed to prepare query")?;

        let rows = stmt
            .query_map(params![project], |row| {
                let path_str: String = row.get(0)?;
                Ok(PathBuf::from(path_str))
            })
            .context("Failed to query project paths")?;

        let mut paths = Vec::new();
        for row in rows {
            paths.push(row.context("Failed to read row")?);
        }

        Ok(paths)
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let total_entries: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_metadata", [], |row| {
                row.get(0)
            })
            .context("Failed to count entries")?;

        let total_size: i64 = conn
            .query_row(
                "SELECT SUM(LENGTH(data)) FROM session_metadata",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let project_count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT project) FROM session_metadata",
                [],
                |row| row.get(0),
            )
            .context("Failed to count projects")?;

        Ok(CacheStats {
            total_entries: total_entries as usize,
            total_size_bytes: total_size as usize,
            project_count: project_count as usize,
        })
    }

    /// Clear all cache entries (for testing or rebuild)
    pub fn clear(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        conn.execute("DELETE FROM session_metadata", [])
            .context("Failed to clear cache")?;

        debug!("Cache cleared");
        Ok(())
    }

    /// Vacuum database to reclaim space
    pub fn vacuum(&self) -> Result<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        conn.execute("VACUUM", []).context("Failed to vacuum")?;

        debug!("Database vacuumed");
        Ok(())
    }

    // ─── Aggregate stats + FTS5 search methods ───────────────────────────────

    /// Get aggregate session stats from O(1) table (total sessions + messages)
    pub fn get_aggregate_stats(&self) -> Result<AggregateStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let mut stmt = conn
            .prepare("SELECT key, value FROM aggregate_stats")
            .context("Failed to prepare aggregate_stats query")?;

        let mut total_sessions = 0usize;
        let mut total_messages = 0usize;

        let rows = stmt
            .query_map([], |row| {
                let key: String = row.get(0)?;
                let value: i64 = row.get(1)?;
                Ok((key, value))
            })
            .context("Failed to query aggregate_stats")?;

        for row in rows {
            let (key, value) = row.context("Failed to read aggregate_stats row")?;
            match key.as_str() {
                "total_sessions" => total_sessions = value.max(0) as usize,
                "total_messages" => total_messages = value.max(0) as usize,
                _ => {}
            }
        }

        Ok(AggregateStats {
            total_sessions,
            total_messages,
        })
    }

    /// Search sessions using FTS5 full-text search.
    ///
    /// Returns up to `limit` results ranked by relevance (BM25).
    /// Returns empty vec (not error) if FTS5 index doesn't exist yet (graceful degradation).
    pub fn search_sessions(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        // Check if FTS5 table exists (graceful degradation for old cache DBs)
        let fts_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='session_fts'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            > 0;

        if !fts_exists {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                r#"
                SELECT
                    sm.path,
                    sm.session_id,
                    sm.project,
                    sm.first_user_message,
                    snippet(session_fts, 2, '[', ']', '...', 12) AS snippet,
                    session_fts.rank,
                    sm.first_timestamp,
                    sm.message_count
                FROM session_fts
                JOIN session_metadata sm ON session_fts.rowid = sm.rowid
                WHERE session_fts MATCH ?
                ORDER BY session_fts.rank
                LIMIT ?
                "#,
            )
            .context("Failed to prepare FTS5 search query")?;

        let limit_i64 = limit as i64;
        let rows = stmt
            .query_map(params![query, limit_i64], |row| {
                Ok(SearchResult {
                    path: PathBuf::from(row.get::<_, String>(0)?),
                    session_id: row.get(1)?,
                    project: row.get(2)?,
                    first_user_message: row.get(3)?,
                    snippet: row.get(4)?,
                    rank: row.get(5)?,
                    first_timestamp: row.get(6)?,
                    message_count: row.get::<_, Option<i64>>(7)?.unwrap_or(0) as u64,
                })
            })
            .context("Failed to execute FTS5 search")?;

        let mut results = Vec::new();
        for row in rows {
            match row {
                Ok(r) => results.push(r),
                Err(e) => {
                    warn!("FTS5 search row error: {}", e);
                }
            }
        }

        Ok(results)
    }

    /// Rebuild FTS5 index from existing session_metadata rows.
    ///
    /// Called once after cache version bump to populate FTS5 for existing sessions.
    pub fn rebuild_fts_index(&self) -> Result<usize> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        // Trigger FTS5 content table rebuild
        conn.execute("INSERT INTO session_fts(session_fts) VALUES('rebuild')", [])
            .context("Failed to trigger FTS5 rebuild")?;

        // Count sessions indexed
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM session_metadata", [], |row| {
                row.get(0)
            })
            .context("Failed to count sessions")?;

        debug!("FTS5 index rebuilt for {} sessions", count);
        Ok(count as usize)
    }

    // ─── Activity cache methods ───────────────────────────────────────────────

    /// Get cached ActivitySummary if the session file mtime matches.
    ///
    /// Returns None on cache miss or mtime mismatch (session was modified).
    pub fn get_activity(
        &self,
        path: &Path,
        current_mtime: SystemTime,
    ) -> Result<Option<ActivitySummary>> {
        let path_str = path.to_string_lossy();
        let mtime_secs = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_secs();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM activity_cache WHERE session_path = ? AND mtime = ?",
                params![path_str.as_ref(), mtime_secs as i64],
                |row| row.get(0),
            )
            .optional()
            .context("Failed to query activity cache")?;

        match result {
            Some(bytes) => {
                let summary: ActivitySummary = bincode::deserialize(&bytes)
                    .context("Failed to deserialize activity summary")?;
                debug!(path = %path.display(), "Activity cache hit");
                Ok(Some(summary))
            }
            None => {
                debug!(path = %path.display(), "Activity cache miss");
                Ok(None)
            }
        }
    }

    /// Store an ActivitySummary in cache, keyed by session file path + mtime.
    ///
    /// Also populates the `activity_alerts` table for cross-session alert queries.
    pub fn put_activity(
        &self,
        path: &Path,
        session_id: &str,
        summary: &ActivitySummary,
        mtime: SystemTime,
    ) -> Result<()> {
        let path_str = path.to_string_lossy();
        let mtime_secs = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_secs();

        let data = bincode::serialize(summary).context("Failed to serialize activity summary")?;

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        // Wrap all writes in a transaction for atomicity.
        // Without this, a crash between DELETE and re-insert leaves stale alerts.
        conn.execute_batch("BEGIN IMMEDIATE")
            .context("Failed to begin activity cache transaction")?;

        let result = (|| -> anyhow::Result<()> {
            // Upsert activity_cache
            conn.execute(
                r#"
                INSERT OR REPLACE INTO activity_cache
                (session_path, mtime, session_id, tool_call_count, alert_count, data)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
                params![
                    path_str.as_ref(),
                    mtime_secs as i64,
                    session_id,
                    (summary.file_accesses.len()
                        + summary.bash_commands.len()
                        + summary.network_calls.len()) as i64,
                    summary.alerts.len() as i64,
                    &data,
                ],
            )
            .context("Failed to insert activity cache entry")?;

            // Refresh activity_alerts for this session (delete + re-insert)
            conn.execute(
                "DELETE FROM activity_alerts WHERE session_path = ?",
                params![path_str.as_ref()],
            )
            .context("Failed to delete old activity alerts")?;

            for alert in &summary.alerts {
                let severity = format!("{:?}", alert.severity);
                let category = format!("{:?}", alert.category);
                conn.execute(
                    r#"
                    INSERT INTO activity_alerts (session_path, severity, category, timestamp, detail)
                    VALUES (?, ?, ?, ?, ?)
                    "#,
                    params![
                        path_str.as_ref(),
                        severity,
                        category,
                        alert.timestamp.to_rfc3339(),
                        &alert.detail,
                    ],
                )
                .context("Failed to insert activity alert")?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT")
                .context("Failed to commit activity cache transaction")?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }

        debug!(
            path = %path.display(),
            alerts = summary.alerts.len(),
            "Activity summary cached"
        );
        Ok(())
    }

    /// Invalidate activity cache entry for a session file.
    ///
    /// Called by file watcher when a session file is modified.
    pub fn invalidate_activity(&self, path: &Path) -> Result<()> {
        let path_str = path.to_string_lossy();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        conn.execute(
            "DELETE FROM activity_cache WHERE session_path = ?",
            params![path_str.as_ref()],
        )
        .context("Failed to delete activity cache entry")?;

        conn.execute(
            "DELETE FROM activity_alerts WHERE session_path = ?",
            params![path_str.as_ref()],
        )
        .context("Failed to delete activity alerts")?;

        debug!(path = %path.display(), "Activity cache invalidated");
        Ok(())
    }

    /// Get all stored alerts, optionally filtered by minimum severity.
    ///
    /// Useful for a global alert view across all analyzed sessions.
    pub fn get_all_alerts(&self, min_severity: Option<&str>) -> Result<Vec<StoredAlert>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        // Build a query that respects severity hierarchy: Critical > Warning > Info.
        // "Warning" means Warning + Critical; "Critical" means Critical only; None means all.
        let query = match min_severity {
            Some("Critical") => "SELECT session_path, severity, category, timestamp, detail \
                 FROM activity_alerts WHERE severity = 'Critical' ORDER BY timestamp DESC",
            Some("Warning") => "SELECT session_path, severity, category, timestamp, detail \
                 FROM activity_alerts WHERE severity IN ('Warning', 'Critical') ORDER BY timestamp DESC",
            _ => "SELECT session_path, severity, category, timestamp, detail \
                 FROM activity_alerts ORDER BY timestamp DESC",
        };

        let mut stmt = conn
            .prepare(query)
            .context("Failed to prepare alert query")?;

        let rows = stmt
            .query_map([], |row| {
                Ok(StoredAlert {
                    session_path: row.get(0)?,
                    severity: row.get(1)?,
                    category: row.get(2)?,
                    timestamp: row.get(3)?,
                    detail: row.get(4)?,
                })
            })
            .context("Failed to query alerts")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect alerts")?;

        Ok(rows)
    }

    /// Get activity cache statistics
    pub fn activity_stats(&self) -> Result<ActivityCacheStats> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let analyzed_sessions: i64 = conn
            .query_row("SELECT COUNT(*) FROM activity_cache", [], |row| row.get(0))
            .context("Failed to count activity cache entries")?;

        let total_alerts: i64 = conn
            .query_row("SELECT COUNT(*) FROM activity_alerts", [], |row| row.get(0))
            .context("Failed to count alerts")?;

        let critical_alerts: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM activity_alerts WHERE severity = 'Critical'",
                [],
                |row| row.get(0),
            )
            .context("Failed to count critical alerts")?;

        Ok(ActivityCacheStats {
            analyzed_sessions: analyzed_sessions as usize,
            total_alerts: total_alerts as usize,
            critical_alerts: critical_alerts as usize,
        })
    }
}

impl Drop for MetadataCache {
    fn drop(&mut self) {
        // WAL checkpoint on drop to ensure all data is flushed to main database file
        // and WAL file doesn't grow unbounded across restarts
        if let Ok(conn) = self.conn.lock() {
            if let Err(e) = conn.pragma_update(None, "wal_checkpoint", "TRUNCATE") {
                warn!("Failed to checkpoint WAL on MetadataCache drop: {}", e);
            } else {
                debug!("WAL checkpoint completed on MetadataCache drop");
            }
        }
    }
}

/// Session metadata cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub project_count: usize,
}

/// Aggregate statistics from O(1) table
#[derive(Debug, Clone, Default)]
pub struct AggregateStats {
    pub total_sessions: usize,
    pub total_messages: usize,
}

/// A single FTS5 search result
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub session_id: String,
    pub project: Option<String>,
    pub first_user_message: Option<String>,
    pub snippet: Option<String>,
    pub rank: f64,
    /// ISO 8601 timestamp string from session_metadata (e.g. "2026-03-20T14:30:00Z")
    pub first_timestamp: Option<String>,
    /// Total message count for this session
    pub message_count: u64,
}

/// Activity cache statistics
#[derive(Debug, Clone)]
pub struct ActivityCacheStats {
    pub analyzed_sessions: usize,
    pub total_alerts: usize,
    pub critical_alerts: usize,
}

/// A single alert record from the activity_alerts table
#[derive(Debug, Clone)]
pub struct StoredAlert {
    pub session_path: String,
    pub severity: String,
    pub category: String,
    pub timestamp: String,
    pub detail: String,
}

impl CacheStats {
    pub fn hit_rate(&self, scanned: usize) -> f64 {
        if scanned == 0 {
            return 0.0;
        }
        (self.total_entries as f64) / (scanned as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionMetadata;
    use chrono::Utc;
    use tempfile::tempdir;

    #[test]
    fn test_cache_creation() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 0);
    }

    #[test]
    fn test_cache_put_get() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/test.jsonl");
        let mut meta = SessionMetadata::from_path(path.clone(), "/test".into());
        meta.id = "test-123".into();
        meta.message_count = 42;
        meta.total_tokens = 1000;
        meta.models_used = vec!["sonnet".to_string()].into_iter().collect();
        meta.first_timestamp = Some(Utc::now());

        let mtime = SystemTime::now();

        // Put
        cache.put(&path, &meta, mtime).unwrap();

        // Get with same mtime (hit)
        let cached = cache.get(&path, mtime).unwrap();
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.id, "test-123");
        assert_eq!(cached.message_count, 42);

        // Get with different mtime (miss)
        let old_mtime = mtime - std::time::Duration::from_secs(3600);
        let cached = cache.get(&path, old_mtime).unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_invalidate() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/test.jsonl");
        let meta = SessionMetadata::from_path(path.clone(), "/test".into());
        let mtime = SystemTime::now();

        cache.put(&path, &meta, mtime).unwrap();

        // Invalidate
        cache.invalidate(&path).unwrap();

        // Should be gone
        let cached = cache.get(&path, mtime).unwrap();
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_project_paths() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let mtime = SystemTime::now();

        // Add sessions for two projects
        for i in 0..3 {
            let path = PathBuf::from(format!("/tmp/project1/session{}.jsonl", i));
            let meta = SessionMetadata::from_path(path.clone(), "/project1".into());
            cache.put(&path, &meta, mtime).unwrap();
        }

        for i in 0..2 {
            let path = PathBuf::from(format!("/tmp/project2/session{}.jsonl", i));
            let meta = SessionMetadata::from_path(path.clone(), "/project2".into());
            cache.put(&path, &meta, mtime).unwrap();
        }

        // Get project1 paths
        let paths = cache.get_project_paths("/project1").unwrap();
        assert_eq!(paths.len(), 3);

        // Get project2 paths
        let paths = cache.get_project_paths("/project2").unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_cache_stats() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let mtime = SystemTime::now();

        // Add some entries
        for i in 0..10 {
            let path = PathBuf::from(format!("/tmp/session{}.jsonl", i));
            let meta = SessionMetadata::from_path(path.clone(), "/test".into());
            cache.put(&path, &meta, mtime).unwrap();
        }

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 10);
        assert!(stats.total_size_bytes > 0);
        assert_eq!(stats.project_count, 1);
    }

    #[test]
    fn test_cache_clear() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/test.jsonl");
        let meta = SessionMetadata::from_path(path.clone(), "/test".into());
        cache.put(&path, &meta, SystemTime::now()).unwrap();

        assert_eq!(cache.stats().unwrap().total_entries, 1);

        cache.clear().unwrap();

        assert_eq!(cache.stats().unwrap().total_entries, 0);
    }

    // ── activity cache tests ─────────────────────────────────────────────────

    fn make_summary_with_alerts() -> ActivitySummary {
        use crate::models::activity::{Alert, AlertCategory, AlertSeverity};
        use chrono::Utc;

        ActivitySummary {
            file_accesses: vec![],
            bash_commands: vec![],
            network_calls: vec![],
            alerts: vec![
                Alert {
                    session_id: "test-session".to_string(),
                    timestamp: Utc::now(),
                    severity: AlertSeverity::Critical,
                    category: AlertCategory::DestructiveCommand,
                    detail: "rm -rf /tmp".to_string(),
                },
                Alert {
                    session_id: "test-session".to_string(),
                    timestamp: Utc::now(),
                    severity: AlertSeverity::Warning,
                    category: AlertCategory::CredentialAccess,
                    detail: "Accessed .env".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_activity_put_get_hit() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();

        let cached = cache.get_activity(&path, mtime).unwrap();
        assert!(cached.is_some(), "Should be a cache hit");
        let cached = cached.unwrap();
        assert_eq!(cached.alerts.len(), 2);
    }

    #[test]
    fn test_activity_get_miss_on_mtime_change() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();

        // Different mtime → miss
        let stale_mtime = mtime - std::time::Duration::from_secs(60);
        let cached = cache.get_activity(&path, stale_mtime).unwrap();
        assert!(cached.is_none(), "Should be a cache miss on mtime change");
    }

    #[test]
    fn test_activity_invalidate() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();

        // Verify stored
        assert!(cache.get_activity(&path, mtime).unwrap().is_some());

        // Invalidate
        cache.invalidate_activity(&path).unwrap();

        // Should be gone
        assert!(
            cache.get_activity(&path, mtime).unwrap().is_none(),
            "Should be gone after invalidation"
        );

        // Alerts should also be cleared
        let alerts = cache.get_all_alerts(None).unwrap();
        assert!(
            alerts.is_empty(),
            "Alerts should be cleared with activity cache"
        );
    }

    #[test]
    fn test_get_all_alerts_returns_stored_alerts() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();

        let alerts = cache.get_all_alerts(None).unwrap();
        assert_eq!(alerts.len(), 2, "Should return both alerts");

        let critical: Vec<_> = alerts.iter().filter(|a| a.severity == "Critical").collect();
        assert_eq!(critical.len(), 1);
        assert!(critical[0].detail.contains("rm -rf"));
    }

    #[test]
    fn test_get_all_alerts_filter_by_severity() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();

        let critical_only = cache.get_all_alerts(Some("Critical")).unwrap();
        assert_eq!(critical_only.len(), 1);
        assert_eq!(critical_only[0].severity, "Critical");
    }

    #[test]
    fn test_activity_stats() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let stats = cache.activity_stats().unwrap();
        assert_eq!(stats.analyzed_sessions, 0);
        assert_eq!(stats.total_alerts, 0);

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts();
        cache
            .put_activity(&path, "test-session", &summary, SystemTime::now())
            .unwrap();

        let stats = cache.activity_stats().unwrap();
        assert_eq!(stats.analyzed_sessions, 1);
        assert_eq!(stats.total_alerts, 2);
        assert_eq!(stats.critical_alerts, 1);
    }

    #[test]
    fn test_activity_put_replaces_stale_alerts() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/session.jsonl");
        let summary = make_summary_with_alerts(); // 2 alerts
        let mtime = SystemTime::now();

        cache
            .put_activity(&path, "test-session", &summary, mtime)
            .unwrap();
        assert_eq!(cache.get_all_alerts(None).unwrap().len(), 2);

        // Re-put with a clean summary (0 alerts)
        let empty_summary = ActivitySummary::default();
        let new_mtime = mtime + std::time::Duration::from_secs(1);
        cache
            .put_activity(&path, "test-session", &empty_summary, new_mtime)
            .unwrap();

        // Old alerts should be gone
        let alerts = cache.get_all_alerts(None).unwrap();
        assert_eq!(alerts.len(), 0, "Stale alerts should be replaced");
    }
}
