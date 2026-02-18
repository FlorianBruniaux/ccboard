//! SQLite metadata cache for session files
//!
//! Caches session metadata with mtime-based invalidation for 90% startup speedup.
//!
//! Schema:
//! - session_metadata table: stores parsed metadata + mtime + cache_version
//! - Indexes: project, mtime for fast queries
//!
//! Invalidation:
//! - File watcher detects modification → delete cache entry
//! - Startup: compare mtime → rescan if stale
//! - Startup: compare cache_version → auto-clear if mismatch
//!
//! Cache Version History:
//! - v1: Initial version (pre-TokenUsage fix)
//! - v2: Fixed TokenUsage::total() to include cache_read_tokens + cache_write_tokens
//! - v3: Added token breakdown fields (input_tokens, output_tokens, cache_creation_tokens,
//!   cache_read_tokens) to SessionMetadata + real pricing calculation

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
///
/// Version History:
/// - v1: Initial version
/// - v2: Fixed TokenUsage::total() calculation
/// - v3: Added token breakdown fields
/// - v4: Added branch field to SessionMetadata
const CACHE_VERSION: i32 = 4;

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

                // Clear all session entries
                conn.execute("DELETE FROM session_metadata", [])
                    .context("Failed to clear stale cache")?;

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

        conn.execute(
            r#"
                INSERT OR REPLACE INTO session_metadata
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
                models_used,
                if meta.has_subagents { 1 } else { 0 },
                &meta.first_user_message,
                &data,
            ],
        )
        .context("Failed to insert metadata")?;

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

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub project_count: usize,
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
}
