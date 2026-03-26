//! Session bookmark store — persisted to ~/.ccboard/bookmarks.json
//!
//! Sessions can be tagged with a short label (e.g. "important", "bug-fix") and an
//! optional free-text note. The store is a thin wrapper around a HashMap serialised
//! to JSON via atomic write (tmp → rename).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single bookmark entry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BookmarkEntry {
    /// Short label chosen by the user (e.g. "important", "bug", "reference")
    pub tag: String,

    /// Optional free-text note
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// When the bookmark was created
    pub created_at: DateTime<Utc>,
}

/// Persisted bookmark store
///
/// Backed by `~/.ccboard/bookmarks.json`.  All mutating methods persist to disk
/// immediately (atomic write).  Read methods operate on the in-memory map and
/// are O(1) / O(n).
#[derive(Debug, Default)]
pub struct BookmarkStore {
    path: PathBuf,
    entries: HashMap<String, BookmarkEntry>, // key = session_id
}

impl BookmarkStore {
    /// Load from `path`.  If the file does not exist, an empty store is returned.
    pub fn load(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let entries = if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read {}", path.display()))?;
            serde_json::from_str::<HashMap<String, BookmarkEntry>>(&raw)
                .with_context(|| format!("Failed to parse {}", path.display()))?
        } else {
            HashMap::new()
        };
        Ok(Self { path, entries })
    }

    /// Return the filesystem path backing this store
    pub fn path(&self) -> &Path {
        &self.path
    }

    // ── Write operations ────────────────────────────────────────────────────

    /// Add or update a bookmark.  Persists immediately.
    pub fn upsert(
        &mut self,
        session_id: &str,
        tag: impl Into<String>,
        note: Option<String>,
    ) -> Result<()> {
        self.entries.insert(
            session_id.to_string(),
            BookmarkEntry {
                tag: tag.into(),
                note,
                created_at: Utc::now(),
            },
        );
        self.save()
    }

    /// Remove a bookmark.  Persists immediately.  Returns `true` if something
    /// was removed.
    pub fn remove(&mut self, session_id: &str) -> Result<bool> {
        let removed = self.entries.remove(session_id).is_some();
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Toggle: remove if present, add with `tag` if absent.
    /// Returns `true` if the session is now bookmarked.
    pub fn toggle(&mut self, session_id: &str, tag: impl Into<String>) -> Result<bool> {
        if self.entries.contains_key(session_id) {
            self.remove(session_id)?;
            Ok(false)
        } else {
            self.upsert(session_id, tag, None)?;
            Ok(true)
        }
    }

    // ── Read operations (no I/O) ────────────────────────────────────────────

    /// Return `true` if the session has a bookmark
    pub fn is_bookmarked(&self, session_id: &str) -> bool {
        self.entries.contains_key(session_id)
    }

    /// Return the entry for a session, if any
    pub fn get(&self, session_id: &str) -> Option<&BookmarkEntry> {
        self.entries.get(session_id)
    }

    /// All bookmarked session IDs
    pub fn all_ids(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }

    /// Number of bookmarks
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    // ── Persistence ─────────────────────────────────────────────────────────

    fn save(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialise bookmarks")?;
        // Atomic write: tmp file → rename
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, &json)
            .with_context(|| format!("Failed to write {}", tmp.display()))?;
        std::fs::rename(&tmp, &self.path)
            .with_context(|| format!("Failed to rename {} → {}", tmp.display(), self.path.display()))?;
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn temp_store() -> (BookmarkStore, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        // Remove the file so BookmarkStore::load() starts empty
        let path = f.path().to_path_buf();
        std::fs::remove_file(&path).ok();
        let store = BookmarkStore::load(&path).unwrap();
        (store, f)
    }

    #[test]
    fn test_upsert_and_is_bookmarked() {
        let (mut store, _f) = temp_store();
        assert!(!store.is_bookmarked("sess-1"));

        store.upsert("sess-1", "important", None).unwrap();
        assert!(store.is_bookmarked("sess-1"));
        assert_eq!(store.get("sess-1").unwrap().tag, "important");
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_remove() {
        let (mut store, _f) = temp_store();
        store.upsert("sess-2", "bug", None).unwrap();
        let removed = store.remove("sess-2").unwrap();
        assert!(removed);
        assert!(!store.is_bookmarked("sess-2"));
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn test_toggle() {
        let (mut store, _f) = temp_store();
        let now_bookmarked = store.toggle("sess-3", "ref").unwrap();
        assert!(now_bookmarked);
        assert!(store.is_bookmarked("sess-3"));

        let now_bookmarked = store.toggle("sess-3", "ref").unwrap();
        assert!(!now_bookmarked);
        assert!(!store.is_bookmarked("sess-3"));
    }

    #[test]
    fn test_persist_and_reload() {
        let f = NamedTempFile::new().unwrap();
        let path = f.path().to_path_buf();
        std::fs::remove_file(&path).ok();

        {
            let mut store = BookmarkStore::load(&path).unwrap();
            store.upsert("sess-persist", "keep", Some("a note".into())).unwrap();
        }

        // Reload from disk
        let store2 = BookmarkStore::load(&path).unwrap();
        assert!(store2.is_bookmarked("sess-persist"));
        let entry = store2.get("sess-persist").unwrap();
        assert_eq!(entry.tag, "keep");
        assert_eq!(entry.note.as_deref(), Some("a note"));
    }

    #[test]
    fn test_empty_store_if_file_missing() {
        let (store, _f) = temp_store();
        assert_eq!(store.len(), 0);
    }
}
