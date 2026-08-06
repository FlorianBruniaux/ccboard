//! SQLite metadata cache for session files
//!
//! Caches session metadata with mtime-based invalidation for 90% startup speedup.
//! Also caches activity analysis (tool calls, alerts) per session on demand.
//!
//! Schema:
//! - session_metadata: parsed metadata + mtime + cache_version
//! - activity_cache: serialized ActivitySummary + mtime per session file
//! - activity_alerts: searchable alert records (severity/category) across all sessions
//! - skill_scan_inventory: per-path scan bookkeeping (mtime, parser_revision) for skill events
//! - skill_events: one row per real `Skill` tool_use invocation, additive since CACHE_VERSION 9
//!   (see `CACHE_VERSION` doc: these two tables are `CREATE TABLE IF NOT EXISTS`, unconditional
//!   on every open, and are never gated by a version bump)
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
use crate::models::{SessionMetadata, SkillInvocationEvent, SkillLeaderboardEntry};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
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
/// - v9: Added lines_added/lines_removed fields to SessionMetadata (code metrics)
const CACHE_VERSION: i32 = 9;

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

            CREATE TABLE IF NOT EXISTS skill_scan_inventory (
                path TEXT PRIMARY KEY, mtime INTEGER NOT NULL, scanned_at INTEGER NOT NULL,
                event_count INTEGER NOT NULL DEFAULT 0, parser_revision INTEGER NOT NULL DEFAULT 1
            );
            CREATE INDEX IF NOT EXISTS idx_skill_inv_mtime ON skill_scan_inventory(mtime);

            CREATE TABLE IF NOT EXISTS skill_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT, path TEXT NOT NULL, skill TEXT NOT NULL,
                args TEXT, ts TEXT, day TEXT, project TEXT NOT NULL, is_subagent INTEGER NOT NULL DEFAULT 0,
                model TEXT, tokens_total INTEGER NOT NULL DEFAULT 0, tokens_input INTEGER NOT NULL DEFAULT 0,
                tokens_output INTEGER NOT NULL DEFAULT 0, tokens_cache_read INTEGER NOT NULL DEFAULT 0,
                tokens_cache_write INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_skill_events_path ON skill_events(path);
            CREATE INDEX IF NOT EXISTS idx_skill_events_skill ON skill_events(skill);
            CREATE INDEX IF NOT EXISTS idx_skill_events_day ON skill_events(day);
            CREATE INDEX IF NOT EXISTS idx_skill_events_project ON skill_events(project);
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
                // Orphan-prevention only: skill tables are additive and not part
                // of CACHE_VERSION's contract (see module docs), but a future
                // unrelated version bump must not leave stale skill rows behind.
                conn.execute("DELETE FROM skill_scan_inventory", [])
                    .context("Failed to clear stale skill scan inventory")?;
                conn.execute("DELETE FROM skill_events", [])
                    .context("Failed to clear stale skill events")?;
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
        let mtime_nanos = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_nanos() as i64;

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM session_metadata WHERE path = ? AND mtime = ?",
                params![path_str.as_ref(), mtime_nanos],
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
        let mtime_nanos = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_nanos() as i64;

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
                mtime_nanos,
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
                mtime_nanos,
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
                mtime_nanos,
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
        conn.execute("DELETE FROM skill_scan_inventory", [])
            .context("Failed to clear skill scan inventory")?;
        conn.execute("DELETE FROM skill_events", [])
            .context("Failed to clear skill events")?;

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
        let mtime_nanos = current_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_nanos() as i64;

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let result: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM activity_cache WHERE session_path = ? AND mtime = ?",
                params![path_str.as_ref(), mtime_nanos],
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
        let mtime_nanos = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_nanos() as i64;

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
                    mtime_nanos,
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

    // ─── Skill invocation cache methods (Phase 1) ─────────────────────────────

    /// Persist skill invocation events for a session file.
    ///
    /// Idempotent: deletes any prior events for the same path and re-inserts,
    /// in the same transaction as the `skill_scan_inventory` upsert, so a
    /// re-scan of an unchanged file never double-counts.
    pub fn put_skill_events(
        &self,
        path: &Path,
        mtime: SystemTime,
        events: &[SkillInvocationEvent],
    ) -> Result<()> {
        let path_str = path.to_string_lossy();
        let mtime_nanos = mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Invalid mtime")?
            .as_nanos() as i64;
        let scanned_at = Utc::now().timestamp();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        conn.execute_batch("BEGIN IMMEDIATE")
            .context("Failed to begin skill events transaction")?;

        let result = (|| -> anyhow::Result<()> {
            conn.execute(
                r#"
                INSERT INTO skill_scan_inventory (path, mtime, scanned_at, event_count, parser_revision)
                VALUES (?, ?, ?, ?, 1)
                ON CONFLICT(path) DO UPDATE SET
                    mtime = excluded.mtime,
                    scanned_at = excluded.scanned_at,
                    event_count = excluded.event_count,
                    parser_revision = excluded.parser_revision
                "#,
                params![
                    path_str.as_ref(),
                    mtime_nanos,
                    scanned_at,
                    events.len() as i64
                ],
            )
            .context("Failed to upsert skill scan inventory")?;

            conn.execute(
                "DELETE FROM skill_events WHERE path = ?",
                params![path_str.as_ref()],
            )
            .context("Failed to clear stale skill events")?;

            for event in events {
                let ts = event.timestamp.map(|t| t.to_rfc3339());
                let day = event.timestamp.map(|t| t.format("%Y-%m-%d").to_string());
                conn.execute(
                    r#"
                    INSERT INTO skill_events
                    (path, skill, args, ts, day, project, is_subagent, model,
                     tokens_total, tokens_input, tokens_output, tokens_cache_read, tokens_cache_write)
                    VALUES (?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                    params![
                        path_str.as_ref(),
                        event.skill,
                        ts,
                        day,
                        event.project.as_str(),
                        if event.is_subagent { 1 } else { 0 },
                        event.model,
                        event.tokens as i64,
                        event.tokens_input as i64,
                        event.tokens_output as i64,
                        event.tokens_cache_read as i64,
                        event.tokens_cache_write as i64,
                    ],
                )
                .context("Failed to insert skill event")?;
            }

            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT")
                .context("Failed to commit skill events transaction")?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }

        debug!(path = %path.display(), events = events.len(), "Skill events cached");
        Ok(())
    }

    /// Read back all persisted skill events for a single session path.
    pub fn get_skill_events(&self, path: &Path) -> Result<Vec<SkillInvocationEvent>> {
        let path_str = path.to_string_lossy();

        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let mut stmt = conn
            .prepare(
                r#"
                SELECT skill, ts, project, is_subagent, model,
                       tokens_total, tokens_input, tokens_output, tokens_cache_read, tokens_cache_write
                FROM skill_events WHERE path = ? ORDER BY id
                "#,
            )
            .context("Failed to prepare skill events query")?;

        let path_for_rows = path.to_path_buf();
        let rows = stmt
            .query_map(params![path_str.as_ref()], move |row| {
                let ts: Option<String> = row.get(1)?;
                let is_subagent: i64 = row.get(3)?;
                Ok(SkillInvocationEvent {
                    skill: row.get(0)?,
                    timestamp: ts
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    project: row.get::<_, String>(2)?.into(),
                    session_path: path_for_rows.clone(),
                    is_subagent: is_subagent != 0,
                    model: row.get(4)?,
                    tokens: row.get::<_, i64>(5)? as u64,
                    tokens_input: row.get::<_, i64>(6)? as u64,
                    tokens_output: row.get::<_, i64>(7)? as u64,
                    tokens_cache_read: row.get::<_, i64>(8)? as u64,
                    tokens_cache_write: row.get::<_, i64>(9)? as u64,
                })
            })
            .context("Failed to query skill events")?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row.context("Failed to read skill event row")?);
        }
        Ok(events)
    }

    /// Compute the skill invocation leaderboard from persisted `skill_events`.
    ///
    /// Cost is never stored; it's computed here at read time via
    /// `calculate_cost(model, ...)` per (skill, model) token breakdown, since
    /// different invocations of the same skill can carry different models.
    pub fn skill_leaderboard(
        &self,
        filter: &SkillLeaderboardFilter,
    ) -> Result<Vec<SkillLeaderboardEntry>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| anyhow::anyhow!("Metadata cache lock poisoned: {}", e))?;

        let mut where_clauses: Vec<String> = Vec::new();
        let mut query_params: Vec<String> = Vec::new();

        if !filter.include_subagents {
            where_clauses.push("is_subagent = 0".to_string());
        }
        if let Some(since) = filter.since {
            where_clauses.push("ts >= ?".to_string());
            query_params.push(since.to_rfc3339());
        }
        if let Some(ref project) = filter.project {
            where_clauses.push("project = ?".to_string());
            query_params.push(project.clone());
        }
        if let Some(ref skill) = filter.skill {
            where_clauses.push("skill = ?".to_string());
            query_params.push(skill.clone());
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let summary_sql = format!(
            "SELECT skill, COUNT(*), COUNT(DISTINCT project), SUM(tokens_total), MIN(ts), MAX(ts) \
             FROM skill_events {where_sql} GROUP BY skill"
        );

        let mut stmt = conn
            .prepare(&summary_sql)
            .context("Failed to prepare skill leaderboard summary query")?;

        let mut entries: std::collections::HashMap<String, SkillLeaderboardEntry> =
            std::collections::HashMap::new();
        let rows = stmt
            .query_map(params_from_iter(query_params.iter()), |row| {
                let skill: String = row.get(0)?;
                let invocations: i64 = row.get(1)?;
                let projects: i64 = row.get(2)?;
                let total_tokens: i64 = row.get(3)?;
                let first_seen: Option<String> = row.get(4)?;
                let last_seen: Option<String> = row.get(5)?;
                Ok((
                    skill,
                    invocations,
                    projects,
                    total_tokens,
                    first_seen,
                    last_seen,
                ))
            })
            .context("Failed to query skill leaderboard summary")?;

        for row in rows {
            let (skill, invocations, projects, total_tokens, first_seen, last_seen) =
                row.context("Failed to read skill leaderboard summary row")?;
            entries.insert(
                skill.clone(),
                SkillLeaderboardEntry {
                    skill,
                    invocations: invocations.max(0) as usize,
                    projects: projects.max(0) as usize,
                    total_tokens: total_tokens.max(0) as u64,
                    estimated_cost_usd: 0.0,
                    first_seen: first_seen
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    last_seen: last_seen
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                },
            );
        }

        let cost_sql = format!(
            "SELECT skill, model, SUM(tokens_input), SUM(tokens_output), SUM(tokens_cache_write), SUM(tokens_cache_read) \
             FROM skill_events {where_sql} GROUP BY skill, model"
        );
        let mut cost_stmt = conn
            .prepare(&cost_sql)
            .context("Failed to prepare skill leaderboard cost query")?;
        let cost_rows = cost_stmt
            .query_map(params_from_iter(query_params.iter()), |row| {
                let skill: String = row.get(0)?;
                let model: Option<String> = row.get(1)?;
                let input: i64 = row.get(2)?;
                let output: i64 = row.get(3)?;
                let cache_write: i64 = row.get(4)?;
                let cache_read: i64 = row.get(5)?;
                Ok((skill, model, input, output, cache_write, cache_read))
            })
            .context("Failed to query skill leaderboard cost breakdown")?;

        for row in cost_rows {
            let (skill, model, input, output, cache_write, cache_read) =
                row.context("Failed to read skill leaderboard cost row")?;
            let model = model.unwrap_or_else(|| "unknown".to_string());
            let cost = crate::pricing::calculate_cost(
                &model,
                input.max(0) as u64,
                output.max(0) as u64,
                cache_write.max(0) as u64,
                cache_read.max(0) as u64,
            );
            if let Some(entry) = entries.get_mut(&skill) {
                entry.estimated_cost_usd += cost;
            }
        }

        let mut result: Vec<SkillLeaderboardEntry> = entries.into_values().collect();
        result.sort_by_key(|e| std::cmp::Reverse(e.total_tokens));
        Ok(result)
    }
}

/// Filter for querying the skill invocation leaderboard from `skill_events`.
///
/// `include_subagents` defaults to `false`: subagent-triggered skill
/// invocations (transcripts under a `subagents/` directory) are excluded
/// unless explicitly requested, matching the `ccboard skills` CLI default.
#[derive(Debug, Clone, Default)]
pub struct SkillLeaderboardFilter {
    pub since: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub skill: Option<String>,
    pub include_subagents: bool,
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

    // ── Skill invocation cache tests (Phase 1) ──────────────────────────────

    fn make_skill_event(
        skill: &str,
        session_path: &Path,
        project: &str,
        tokens: u64,
        model: &str,
        timestamp: DateTime<Utc>,
    ) -> SkillInvocationEvent {
        SkillInvocationEvent {
            skill: skill.to_string(),
            timestamp: Some(timestamp),
            project: project.into(),
            session_path: session_path.to_path_buf(),
            is_subagent: false,
            tokens,
            model: Some(model.to_string()),
            tokens_input: tokens / 2,
            tokens_output: tokens / 2,
            tokens_cache_read: 0,
            tokens_cache_write: 0,
        }
    }

    #[test]
    fn test_skill_invocations_roundtrip() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();

        let path = PathBuf::from("/tmp/roundtrip/session.jsonl");
        let mtime = SystemTime::now();
        let ts = Utc::now();
        let events = vec![
            make_skill_event("pdf-generator", &path, "/roundtrip", 100, "sonnet-4", ts),
            make_skill_event(
                "superpowers:brainstorming",
                &path,
                "/roundtrip",
                42,
                "opus-4",
                ts,
            ),
        ];

        cache.put_skill_events(&path, mtime, &events).unwrap();

        let read_back = cache.get_skill_events(&path).unwrap();
        assert_eq!(read_back.len(), 2, "no events lost on roundtrip");
        assert_eq!(read_back[0].skill, "pdf-generator");
        assert_eq!(read_back[0].tokens, 100);
        assert_eq!(read_back[0].model.as_deref(), Some("sonnet-4"));
        assert_eq!(read_back[1].skill, "superpowers:brainstorming");
        assert_eq!(read_back[1].tokens, 42);
        assert_eq!(read_back[1].tokens_input, 21);
        assert_eq!(read_back[1].tokens_output, 21);
    }

    #[test]
    fn test_skill_invocations_migration_from_populated_v9_db() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("session-metadata.db");

        // Build a realistic pre-existing v9 database by hand: the schema as
        // it existed before this feature, no skill_scan_inventory/skill_events.
        {
            let conn = Connection::open(&db_path).unwrap();
            conn.execute_batch(
                r#"
                CREATE TABLE cache_metadata (key TEXT PRIMARY KEY, value INTEGER NOT NULL);
                CREATE TABLE session_metadata (
                    path TEXT PRIMARY KEY, mtime INTEGER NOT NULL, project TEXT NOT NULL,
                    session_id TEXT NOT NULL, first_timestamp TEXT, last_timestamp TEXT,
                    message_count INTEGER NOT NULL, total_tokens INTEGER NOT NULL,
                    models_used TEXT NOT NULL, has_subagents INTEGER NOT NULL,
                    first_user_message TEXT, data BLOB NOT NULL
                );
                INSERT INTO cache_metadata (key, value) VALUES ('version', 9);
                "#,
            )
            .unwrap();

            for i in 0..5 {
                let path = format!("/tmp/v9-project/session{i}.jsonl");
                let meta = SessionMetadata::from_path(PathBuf::from(&path), "/v9-project".into());
                let data = bincode::serialize(&meta).unwrap();
                conn.execute(
                    "INSERT INTO session_metadata (path, mtime, project, session_id, \
                     first_timestamp, last_timestamp, message_count, total_tokens, models_used, \
                     has_subagents, first_user_message, data) \
                     VALUES (?, ?, ?, ?, NULL, NULL, ?, ?, ?, 0, NULL, ?)",
                    params![
                        path,
                        1_000_000_000_i64 + i,
                        "/v9-project",
                        format!("session-{i}"),
                        i * 3,
                        i * 100,
                        "[]",
                        &data,
                    ],
                )
                .unwrap();
            }
        }

        // Reopen with the new code, which unconditionally creates
        // skill_scan_inventory + skill_events alongside the existing tables.
        let cache = MetadataCache::new(dir.path()).unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(
            stats.total_entries, 5,
            "pre-existing v9 session_metadata rows must survive opening with the new code"
        );

        // New tables must exist and be queryable (empty, no error).
        let leaderboard = cache
            .skill_leaderboard(&SkillLeaderboardFilter::default())
            .unwrap();
        assert!(leaderboard.is_empty());
    }

    #[test]
    fn test_additive_schema_preserves_existing_rows() {
        let dir = tempdir().unwrap();
        let mtime = SystemTime::now();

        {
            let cache = MetadataCache::new(dir.path()).unwrap();
            for i in 0..8u64 {
                let path = PathBuf::from(format!("/tmp/additive/session{i}.jsonl"));
                let mut meta = SessionMetadata::from_path(path.clone(), "/additive".into());
                meta.id = format!("session-{i}").into();
                meta.message_count = i * 7;
                meta.total_tokens = i * 1000;
                cache.put(&path, &meta, mtime).unwrap();
            }
        } // cache dropped here: connection closed, WAL checkpointed

        // Reopen, simulating a fresh process start with the new code.
        let cache = MetadataCache::new(dir.path()).unwrap();
        assert_eq!(cache.stats().unwrap().total_entries, 8);

        for i in 0..8u64 {
            let path = PathBuf::from(format!("/tmp/additive/session{i}.jsonl"));
            let cached = cache
                .get(&path, mtime)
                .unwrap()
                .unwrap_or_else(|| panic!("session {i} missing after reopen"));
            assert_eq!(cached.id.as_str(), format!("session-{i}"));
            assert_eq!(cached.message_count, i * 7);
            assert_eq!(cached.total_tokens, i * 1000);
        }
    }

    #[test]
    fn test_skill_leaderboard_aggregation() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();
        let mtime = SystemTime::now();

        let t1 = Utc::now() - chrono::Duration::days(2);
        let t2 = Utc::now() - chrono::Duration::days(1);
        let t3 = Utc::now();

        let session_a = PathBuf::from("/tmp/proj-a/session1.jsonl");
        let session_b = PathBuf::from("/tmp/proj-b/session2.jsonl");
        let session_c = PathBuf::from("/tmp/proj-a/session3.jsonl");

        cache
            .put_skill_events(
                &session_a,
                mtime,
                &[make_skill_event(
                    "pdf-generator",
                    &session_a,
                    "/proj-a",
                    100,
                    "sonnet-4",
                    t1,
                )],
            )
            .unwrap();
        cache
            .put_skill_events(
                &session_b,
                mtime,
                &[make_skill_event(
                    "pdf-generator",
                    &session_b,
                    "/proj-b",
                    200,
                    "sonnet-4",
                    t2,
                )],
            )
            .unwrap();
        cache
            .put_skill_events(
                &session_c,
                mtime,
                &[make_skill_event(
                    "superpowers:brainstorming",
                    &session_c,
                    "/proj-a",
                    50,
                    "opus-4",
                    t3,
                )],
            )
            .unwrap();

        let leaderboard = cache
            .skill_leaderboard(&SkillLeaderboardFilter::default())
            .unwrap();

        let pdf_entry = leaderboard
            .iter()
            .find(|e| e.skill == "pdf-generator")
            .expect("pdf-generator entry present");
        assert_eq!(pdf_entry.invocations, 2, "aggregated across two sessions");
        assert_eq!(pdf_entry.projects, 2, "aggregated across two projects");
        assert_eq!(pdf_entry.total_tokens, 300);
        assert!(pdf_entry.estimated_cost_usd > 0.0);
        assert_eq!(
            pdf_entry.first_seen.map(|t| t.date_naive()),
            Some(t1.date_naive())
        );
        assert_eq!(
            pdf_entry.last_seen.map(|t| t.date_naive()),
            Some(t2.date_naive())
        );

        let brainstorm_entry = leaderboard
            .iter()
            .find(|e| e.skill == "superpowers:brainstorming")
            .expect("namespaced skill entry present, not split on ':'");
        assert_eq!(brainstorm_entry.invocations, 1);
        assert_eq!(brainstorm_entry.projects, 1);
        assert_eq!(brainstorm_entry.total_tokens, 50);
    }

    #[test]
    fn test_skill_leaderboard_filters_by_project_and_skill() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();
        let mtime = SystemTime::now();
        let now = Utc::now();

        let session_a = PathBuf::from("/tmp/proj-a/session1.jsonl");
        let session_b = PathBuf::from("/tmp/proj-b/session2.jsonl");

        cache
            .put_skill_events(
                &session_a,
                mtime,
                &[make_skill_event(
                    "skill-x", &session_a, "/proj-a", 10, "sonnet-4", now,
                )],
            )
            .unwrap();
        cache
            .put_skill_events(
                &session_b,
                mtime,
                &[make_skill_event(
                    "skill-y", &session_b, "/proj-b", 20, "sonnet-4", now,
                )],
            )
            .unwrap();

        let filtered = cache
            .skill_leaderboard(&SkillLeaderboardFilter {
                project: Some("/proj-a".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].skill, "skill-x");

        let by_skill = cache
            .skill_leaderboard(&SkillLeaderboardFilter {
                skill: Some("skill-y".to_string()),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(by_skill.len(), 1);
        assert_eq!(by_skill[0].skill, "skill-y");
    }

    #[test]
    fn test_skill_leaderboard_excludes_subagents_by_default() {
        let dir = tempdir().unwrap();
        let cache = MetadataCache::new(dir.path()).unwrap();
        let mtime = SystemTime::now();
        let now = Utc::now();

        let path = PathBuf::from("/tmp/proj/subagents/agent.jsonl");
        let mut event = make_skill_event("skill-z", &path, "/proj", 30, "sonnet-4", now);
        event.is_subagent = true;

        cache.put_skill_events(&path, mtime, &[event]).unwrap();

        let default_filter = cache
            .skill_leaderboard(&SkillLeaderboardFilter::default())
            .unwrap();
        assert!(
            default_filter.is_empty(),
            "subagent invocations must be excluded unless include_subagents is set"
        );

        let with_subagents = cache
            .skill_leaderboard(&SkillLeaderboardFilter {
                include_subagents: true,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(with_subagents.len(), 1);
        assert_eq!(with_subagents[0].skill, "skill-z");
    }
}
