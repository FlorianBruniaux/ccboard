//! Session index parser with streaming + early termination
//!
//! Key insight from review: first/last line strategy doesn't work because:
//! - First line is often {"type":"file-history-snapshot",...}
//! - Last line is often {"type":"summary",...}
//!
//! Solution: Stream until session_end event, extracting metadata along the way.

use crate::error::{CoreError, LoadError, LoadReport};
use crate::models::{SessionLine, SessionMetadata, session::SessionSummary};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, trace, warn};
use walkdir::WalkDir;

/// Maximum characters for first user message preview
const PREVIEW_MAX_CHARS: usize = 200;

/// Maximum lines to scan before giving up (circuit breaker)
const MAX_SCAN_LINES: usize = 10_000;

/// Parser for discovering and indexing sessions
pub struct SessionIndexParser {
    /// Maximum concurrent scans
    max_concurrent: usize,
}

impl Default for SessionIndexParser {
    fn default() -> Self {
        Self { max_concurrent: 8 }
    }
}

impl SessionIndexParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_concurrency(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Discover all session files under a projects directory
    pub fn discover_sessions(&self, projects_dir: &Path) -> Vec<PathBuf> {
        let mut sessions = Vec::new();

        for entry in WalkDir::new(projects_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                sessions.push(path.to_path_buf());
            }
        }

        debug!(count = sessions.len(), "Discovered session files");
        sessions
    }

    /// Extract project path from session file path
    ///
    /// Format: ~/.claude/projects/-Users-foo-myproject/<session>.jsonl
    /// Returns: /Users/foo/myproject
    pub fn extract_project_path(&self, session_path: &Path) -> String {
        session_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|encoded| {
                // Decode path: -Users-foo-bar -> /Users/foo/bar
                if encoded.starts_with('-') {
                    encoded.replace('-', "/")
                } else {
                    encoded.to_string()
                }
            })
            .unwrap_or_else(|| "unknown".to_string())
    }

    /// Scan a single session file for metadata using streaming
    ///
    /// Streams JSONL lines until session_end or max lines reached.
    /// Extracts: first user message, models used, token counts, timestamps.
    pub async fn scan_session(&self, path: &Path) -> Result<SessionMetadata, CoreError> {
        let project_path = self.extract_project_path(path);
        let mut metadata = SessionMetadata::from_path(path.to_path_buf(), project_path);

        let file = File::open(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                CoreError::FileRead {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })?;

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut line_number = 0;
        let mut models_seen: HashSet<String> = HashSet::new();
        let mut first_timestamp = None;
        let mut last_timestamp = None;
        let mut message_count = 0u64;
        let mut total_tokens = 0u64;

        while let Some(line_result) = lines.next_line().await.map_err(|e| CoreError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })? {
            line_number += 1;

            // Circuit breaker: stop if file is unexpectedly large
            if line_number > MAX_SCAN_LINES {
                warn!(
                    path = %path.display(),
                    lines = line_number,
                    "Session scan hit line limit, terminating early"
                );
                break;
            }

            // Parse line (skip malformed)
            let session_line: SessionLine = match serde_json::from_str(&line_result) {
                Ok(l) => l,
                Err(e) => {
                    trace!(
                        path = %path.display(),
                        line = line_number,
                        error = %e,
                        "Skipping malformed JSONL line"
                    );
                    continue;
                }
            };

            // Track timestamps
            if let Some(ts) = session_line.timestamp {
                if first_timestamp.is_none() {
                    first_timestamp = Some(ts);
                }
                last_timestamp = Some(ts);
            }

            // Extract session ID from first line with it
            // Prefer sessionId from content over filename-derived ID
            if let Some(ref id) = session_line.session_id {
                if metadata.id.is_empty()
                    || metadata.id == "unknown"
                    || !metadata.id.chars().all(|c| c.is_alphanumeric() || c == '-')
                    || metadata.id.starts_with(".tmp")
                {
                    metadata.id = id.clone();
                }
            }

            // Track models
            if let Some(ref model) = session_line.model {
                models_seen.insert(model.clone());
            }

            // Track subagents
            if session_line.parent_session_id.is_some() {
                metadata.has_subagents = true;
            }

            // Count user messages and extract first preview
            if session_line.line_type == "user" {
                message_count += 1;

                if metadata.first_user_message.is_none() {
                    if let Some(ref msg) = session_line.message {
                        if let Some(ref content) = msg.content {
                            let preview: String = content.chars().take(PREVIEW_MAX_CHARS).collect();
                            metadata.first_user_message = Some(preview);
                        }
                    }
                }
            }

            // Count assistant messages
            if session_line.line_type == "assistant" {
                message_count += 1;

                // Accumulate tokens from either root usage or message.usage
                let usage_opt = session_line
                    .usage
                    .as_ref()
                    .or_else(|| session_line.message.as_ref().and_then(|m| m.usage.as_ref()));

                if let Some(usage) = usage_opt {
                    total_tokens += usage.total();
                }
            }

            // Early termination on session_end
            if session_line.line_type == "summary" || session_line.line_type == "session_end" {
                if let Some(summary) = session_line.summary {
                    // Use summary data which is more accurate
                    Self::apply_summary(&mut metadata, &summary);
                }
                break;
            }
        }

        // Apply collected data
        metadata.first_timestamp = first_timestamp;
        metadata.last_timestamp = last_timestamp;
        metadata.models_used = models_seen.into_iter().collect();

        // Only use counted values if summary didn't provide them
        if metadata.message_count == 0 {
            metadata.message_count = message_count;
        }
        if metadata.total_tokens == 0 {
            metadata.total_tokens = total_tokens;
        }

        Ok(metadata)
    }

    /// Apply summary data to metadata
    fn apply_summary(metadata: &mut SessionMetadata, summary: &SessionSummary) {
        metadata.total_tokens = summary.total_tokens;
        metadata.message_count = summary.message_count;
        metadata.duration_seconds = summary.duration_seconds;

        if let Some(ref models) = summary.models_used {
            metadata.models_used = models.to_vec();
        }
    }

    /// Scan session with graceful degradation
    pub async fn scan_session_graceful(
        &self,
        path: &Path,
        report: &mut LoadReport,
    ) -> Option<SessionMetadata> {
        match self.scan_session(path).await {
            Ok(meta) => {
                report.sessions_scanned += 1;
                Some(meta)
            }
            Err(e) => {
                report.sessions_failed += 1;
                report.add_warning(
                    format!("session:{}", path.display()),
                    format!("Failed to scan: {}", e),
                );
                None
            }
        }
    }

    /// Scan all sessions in a directory with parallel processing
    pub async fn scan_all(
        &self,
        projects_dir: &Path,
        report: &mut LoadReport,
    ) -> Vec<SessionMetadata> {
        let paths = self.discover_sessions(projects_dir);
        let mut results = Vec::with_capacity(paths.len());

        // Use semaphore for bounded concurrency
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let mut handles = Vec::new();

        for path in paths {
            let sem = semaphore.clone();
            let parser = SessionIndexParser::new();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                parser.scan_session(&path).await
            });

            handles.push(handle);
        }

        for handle in handles {
            match handle.await {
                Ok(Ok(meta)) => {
                    report.sessions_scanned += 1;
                    results.push(meta);
                }
                Ok(Err(e)) => {
                    report.sessions_failed += 1;
                    report.add_warning("session_scan", e.to_string());
                }
                Err(e) => {
                    report.sessions_failed += 1;
                    report.add_error(LoadError::error(
                        "session_scan",
                        format!("Task panic: {}", e),
                    ));
                }
            }
        }

        debug!(
            scanned = report.sessions_scanned,
            failed = report.sessions_failed,
            "Session scan complete"
        );

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, tempdir};

    #[test]
    fn test_extract_project_path() {
        let parser = SessionIndexParser::new();

        let path = Path::new("/home/user/.claude/projects/-Users-foo-myproject/abc123.jsonl");
        let project = parser.extract_project_path(path);
        assert_eq!(project, "/Users/foo/myproject");
    }

    #[tokio::test]
    async fn test_scan_session_basic() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "file-history-snapshot", "timestamp": "2025-01-15T10:00:00Z"}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "test-123", "timestamp": "2025-01-15T10:01:00Z", "message": {{"content": "Hello, can you help me?"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-20250514", "timestamp": "2025-01-15T10:02:00Z", "usage": {{"input_tokens": 100, "output_tokens": 50}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 150, "messageCount": 2, "durationSeconds": 60}}}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(meta.id, "test-123");
        assert_eq!(meta.total_tokens, 150);
        assert_eq!(meta.message_count, 2);
        assert_eq!(meta.duration_seconds, Some(60));
        assert!(meta.first_user_message.is_some());
        assert!(meta.first_user_message.unwrap().contains("Hello"));
        assert!(
            meta.models_used
                .contains(&"claude-sonnet-4-20250514".to_string())
        );
    }

    #[tokio::test]
    async fn test_scan_session_early_termination() {
        let mut file = NamedTempFile::new().unwrap();

        // Write a session_end early, followed by more content
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "early", "message": {{"content": "First"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 100, "messageCount": 1}}}}"#
        )
        .unwrap();
        // This should not be processed
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "early", "message": {{"content": "Should not appear"}}}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        // Should only count 1 message (from summary, not 2)
        assert_eq!(meta.message_count, 1);
        assert!(meta.first_user_message.unwrap().contains("First"));
    }

    #[test]
    fn test_discover_sessions() {
        let dir = tempdir().unwrap();
        let sessions_dir = dir.path().join("projects").join("-test-project");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        std::fs::write(sessions_dir.join("session1.jsonl"), "{}").unwrap();
        std::fs::write(sessions_dir.join("session2.jsonl"), "{}").unwrap();
        std::fs::write(sessions_dir.join("other.txt"), "not a session").unwrap();

        let parser = SessionIndexParser::new();
        let sessions = parser.discover_sessions(dir.path().join("projects").as_path());

        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().all(|p| p.extension().unwrap() == "jsonl"));
    }

    #[tokio::test]
    async fn test_token_extraction_with_cache() {
        let mut file = NamedTempFile::new().unwrap();

        // Real format from Claude Code JSONL with cache tokens in message.usage
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "cache-test", "message": {{"content": "Test"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "message": {{"usage": {{"input_tokens": 100, "output_tokens": 50, "cache_read_input_tokens": 1000, "cache_creation_input_tokens": 500}}}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "message": {{"usage": {{"input_tokens": 200, "output_tokens": 75}}}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "summary"}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        // Should accumulate all token types from both messages
        // total = (100 + 50) + (200 + 75) = 150 + 275 = 425
        assert_eq!(meta.total_tokens, 425);
    }
}
