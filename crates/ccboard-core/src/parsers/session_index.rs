//! Session index parser with streaming + early termination
//!
//! Key insight from review: first/last line strategy doesn't work because:
//! - First line is often {"type":"file-history-snapshot",...}
//! - Last line is often {"type":"summary",...}
//!
//! Solution: Stream until session_end event, extracting metadata along the way.

use crate::cache::MetadataCache;
use crate::error::{CoreError, LoadError, LoadReport};
use crate::models::{SessionLine, SessionMetadata, session::SessionSummary};
use crate::parsers::filters::is_meaningful_user_message;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, trace, warn};
use walkdir::WalkDir;

/// Maximum characters for first user message preview
const PREVIEW_MAX_CHARS: usize = 200;

/// Maximum lines to scan before giving up (circuit breaker)
const MAX_SCAN_LINES: usize = 10_000;

/// Maximum line size in bytes (10MB) - OOM protection
const MAX_LINE_SIZE: usize = 10 * 1024 * 1024;

/// Parser for discovering and indexing sessions
#[derive(Clone)]
pub struct SessionIndexParser {
    /// Maximum concurrent scans
    max_concurrent: usize,

    /// Optional metadata cache for 90% speedup
    cache: Option<Arc<MetadataCache>>,
}

impl Default for SessionIndexParser {
    fn default() -> Self {
        Self {
            max_concurrent: 8,
            cache: None,
        }
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

    /// Enable metadata caching for 90% startup speedup
    pub fn with_cache(mut self, cache: Arc<MetadataCache>) -> Self {
        self.cache = Some(cache);
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
    ///
    /// SECURITY: Validates path to prevent traversal attacks
    pub fn extract_project_path(&self, session_path: &Path) -> String {
        let raw_path = session_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .and_then(|encoded| Self::sanitize_project_path(encoded).ok())
            .unwrap_or_else(|| "unknown".to_string());

        // Normalize worktrees to their parent repo
        // Example: /path/to/repo/worktrees/feature → /path/to/repo
        Self::normalize_worktree_path(&raw_path)
    }

    /// Normalize worktree paths to their parent repository
    ///
    /// Git worktrees are typically stored in:
    /// - /path/to/repo/worktrees/branch-name
    /// - /path/to/repo/.worktrees/branch-name
    /// - /path/to/worktrees/repo-branch-name
    ///
    /// This function detects common worktree patterns and normalizes them
    /// to the parent repository path for better project grouping.
    fn normalize_worktree_path(path: &str) -> String {
        // Normalize multiple slashes (from -- encoding) to single slash
        // Example: /Users/.../app//worktrees → /Users/.../app/worktrees
        let normalized = path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("/");
        let normalized = if path.starts_with('/') {
            format!("/{}", normalized)
        } else {
            normalized
        };

        // Pattern 1: /path/to/repo/worktrees/branch → /path/to/repo
        if let Some(idx) = normalized.find("/worktrees/") {
            return normalized[..idx].to_string();
        }

        // Pattern 2: /path/to/repo/.worktrees/branch → /path/to/repo
        if let Some(idx) = normalized.find("/.worktrees/") {
            return normalized[..idx].to_string();
        }

        // Pattern 3: /path/to/worktrees/repo-branch → /path/to (heuristic)
        // Only apply if path contains "worktrees" as a directory component
        let components: Vec<&str> = normalized.split('/').collect();
        if let Some(worktree_idx) = components.iter().position(|&c| c == "worktrees") {
            if worktree_idx > 0 {
                // Return everything before "/worktrees"
                return components[..worktree_idx].join("/");
            }
        }

        // No worktree pattern detected, return normalized path
        normalized
    }

    /// Sanitize project path from encoded format
    ///
    /// SECURITY: This function prevents:
    /// - Path traversal via .. components
    /// - Symlink attacks
    /// - Absolute path injection
    pub fn sanitize_project_path(encoded: &str) -> Result<String, CoreError> {
        use std::path::Component;

        let decoded = if encoded.starts_with('-') {
            encoded.replace('-', "/")
        } else {
            encoded.to_string()
        };

        // Strip ".." components to prevent traversal
        let normalized = Path::new(&decoded)
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .collect::<PathBuf>();

        // Validate no symlinks (if path exists)
        #[cfg(unix)]
        if normalized.exists() {
            use std::fs;
            let metadata = fs::symlink_metadata(&normalized).map_err(|e| CoreError::FileRead {
                path: normalized.clone(),
                source: e,
            })?;

            if metadata.is_symlink() {
                return Err(CoreError::InvalidPath {
                    path: normalized,
                    reason: "Symlinks not allowed in project paths".to_string(),
                });
            }
        }

        // Convert to string, preserving leading / for absolute paths
        let path_str = normalized.to_string_lossy().to_string();
        if decoded.starts_with('/') && !path_str.starts_with('/') {
            Ok(format!("/{}", path_str))
        } else {
            Ok(path_str)
        }
    }

    /// Scan a single session file for metadata using streaming
    ///
    /// Streams JSONL lines until session_end or max lines reached.
    /// Extracts: first user message, models used, token counts, timestamps.
    ///
    /// If cache is enabled, checks cache first (90% speedup).
    pub async fn scan_session(&self, path: &Path) -> Result<SessionMetadata, CoreError> {
        let path_buf = path.to_path_buf();

        // Check cache first if available (in blocking task for SQLite)
        if let Some(ref cache) = self.cache {
            if let Ok(metadata_result) = std::fs::metadata(&path_buf) {
                if let Ok(mtime) = metadata_result.modified() {
                    let cache = cache.clone();
                    let path_buf_clone = path_buf.clone();

                    // Try cache in blocking task
                    let cached_result =
                        tokio::task::spawn_blocking(move || cache.get(&path_buf_clone, mtime))
                            .await
                            .ok()
                            .and_then(|r| r.ok())
                            .and_then(|opt| opt);

                    if let Some(cached) = cached_result {
                        trace!(path = %path.display(), "Using cached metadata");
                        return Ok(cached);
                    }
                }
            }
        }

        // Cache miss or no cache: parse from file
        let metadata = self.scan_session_uncached(path).await?;

        // Store in cache if available (in blocking task for SQLite)
        if let Some(ref cache) = self.cache {
            if let Ok(metadata_result) = std::fs::metadata(&path_buf) {
                if let Ok(mtime) = metadata_result.modified() {
                    let cache = cache.clone();
                    let meta_clone = metadata.clone();
                    let path_clone = path_buf.clone();

                    // Store in cache in blocking task (WAIT for completion)
                    if let Ok(Err(e)) = tokio::task::spawn_blocking(move || {
                        cache.put(&path_clone, &meta_clone, mtime)
                    })
                    .await
                    {
                        warn!(path = %path_buf.display(), error = %e, "Failed to cache metadata");
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Scan session without cache (internal)
    async fn scan_session_uncached(&self, path: &Path) -> Result<SessionMetadata, CoreError> {
        let project_path = self.extract_project_path(path);
        let mut metadata = SessionMetadata::from_path(path.to_path_buf(), project_path.into());

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
        let mut input_tokens = 0u64;
        let mut output_tokens = 0u64;
        let mut cache_creation_tokens = 0u64;
        let mut cache_read_tokens = 0u64;
        let mut branch: Option<String> = None;
        let mut tool_usage: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

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

            // SECURITY: OOM protection - skip oversized lines
            if line_result.len() > MAX_LINE_SIZE {
                warn!(
                    path = %path.display(),
                    line = line_number,
                    size_mb = line_result.len() / (1024 * 1024),
                    "Skipping oversized line (potential attack or corruption)"
                );
                continue;
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
                    metadata.id = id.clone().into();
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

            // Extract git branch (first occurrence wins)
            if branch.is_none() {
                if let Some(ref git_branch) = session_line.git_branch {
                    branch = Some(normalize_branch(git_branch));
                }
            }

            // Count user messages and extract first preview (filtered)
            if session_line.line_type == "user" {
                message_count += 1;

                if metadata.first_user_message.is_none() {
                    if let Some(ref msg) = session_line.message {
                        if let Some(ref content) = msg.content {
                            // Content can be String (old format) or Array (new format with content blocks)
                            let text = match content {
                                serde_json::Value::String(s) => s.clone(),
                                serde_json::Value::Array(blocks) => {
                                    // Extract text from content blocks
                                    blocks
                                        .iter()
                                        .filter_map(|block| {
                                            block.get("text").and_then(|t| t.as_str())
                                        })
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                }
                                _ => String::new(),
                            };

                            // Filter out system/protocol messages for cleaner previews
                            if is_meaningful_user_message(&text) {
                                let preview: String =
                                    text.chars().take(PREVIEW_MAX_CHARS).collect();
                                metadata.first_user_message = Some(preview);
                            }
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
                    input_tokens += usage.input_tokens;
                    output_tokens += usage.output_tokens;
                    cache_creation_tokens += usage.cache_write_tokens;
                    cache_read_tokens += usage.cache_read_tokens;
                }

                // Extract tool calls from message
                if let Some(ref msg) = session_line.message {
                    // Try tool_calls field first (if present)
                    if let Some(ref tool_calls) = msg.tool_calls {
                        for tool_call in tool_calls {
                            // Format: {"type": "function", "function": {"name": "Read", ...}}
                            if let Some(function) = tool_call.get("function") {
                                if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                    *tool_usage.entry(name.to_string()).or_default() += 1;
                                }
                            }
                        }
                    }

                    // Also check content array for tool_use blocks (real Claude Code format)
                    if let Some(ref content) = msg.content {
                        if let Some(blocks) = content.as_array() {
                            for block in blocks {
                                if let Some(block_type) = block.get("type").and_then(|t| t.as_str())
                                {
                                    if block_type == "tool_use" {
                                        if let Some(name) =
                                            block.get("name").and_then(|n| n.as_str())
                                        {
                                            *tool_usage.entry(name.to_string()).or_default() += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
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

        // Apply token breakdown (always use counted values, summary doesn't have these)
        metadata.input_tokens = input_tokens;
        metadata.output_tokens = output_tokens;
        metadata.cache_creation_tokens = cache_creation_tokens;
        metadata.cache_read_tokens = cache_read_tokens;

        // Apply branch
        metadata.branch = branch;

        // Apply tool usage
        metadata.tool_usage = tool_usage;

        Ok(metadata)
    }

    /// Apply summary data to metadata
    fn apply_summary(metadata: &mut SessionMetadata, summary: &SessionSummary) {
        // Only use summary values if they are non-zero (summary might be incomplete)
        if summary.total_tokens > 0 {
            metadata.total_tokens = summary.total_tokens;
        }
        if summary.message_count > 0 {
            metadata.message_count = summary.message_count;
        }
        metadata.duration_seconds = summary.duration_seconds;

        if let Some(ref models) = summary.models_used {
            metadata.models_used = models.to_vec();
        }
    }
}

/// Normalize git branch name
///
/// Handles common variations:
/// - Strips `worktrees/` prefix (e.g., "worktrees/feature-x" → "feature-x")
/// - Strips ` (dirty)` suffix (e.g., "main (dirty)" → "main")
/// - Trims whitespace
/// - Handles detached HEAD states (e.g., "HEAD (detached at abc123)" → "HEAD")
fn normalize_branch(raw: &str) -> String {
    let mut normalized = raw.trim();

    // Strip worktrees/ prefix
    if let Some(stripped) = normalized.strip_prefix("worktrees/") {
        normalized = stripped;
    }

    // Strip (dirty) suffix
    if let Some(stripped) = normalized.strip_suffix(" (dirty)") {
        normalized = stripped;
    }

    // Handle detached HEAD: "HEAD (detached at abc123)" → "HEAD"
    if normalized.starts_with("HEAD (detached") {
        normalized = "HEAD";
    }

    normalized.to_string()
}

impl SessionIndexParser {
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
            // CRITICAL: Clone self to preserve cache reference!
            let parser = self.clone();

            let handle = tokio::spawn(async move {
                // Graceful degradation: skip this session if semaphore acquisition fails
                let _permit = match sem.acquire().await {
                    Ok(p) => p,
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Failed to acquire semaphore permit, skipping session");
                        return Err(CoreError::LockTimeout);
                    }
                };
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
    fn test_normalize_branch_plain() {
        assert_eq!(normalize_branch("main"), "main");
        assert_eq!(normalize_branch("feature/auth"), "feature/auth");
    }

    #[test]
    fn test_normalize_branch_worktree_prefix() {
        assert_eq!(normalize_branch("worktrees/feature-x"), "feature-x");
        assert_eq!(normalize_branch("worktrees/main"), "main");
    }

    #[test]
    fn test_normalize_branch_dirty_suffix() {
        assert_eq!(normalize_branch("main (dirty)"), "main");
        assert_eq!(normalize_branch("feature/ui (dirty)"), "feature/ui");
    }

    #[test]
    fn test_normalize_branch_detached_head() {
        assert_eq!(normalize_branch("HEAD (detached at abc123)"), "HEAD");
        assert_eq!(normalize_branch("HEAD (detached at 1a2b3c4)"), "HEAD");
    }

    #[test]
    fn test_normalize_branch_combined() {
        assert_eq!(normalize_branch("worktrees/feature (dirty)"), "feature");
        assert_eq!(normalize_branch("  main (dirty)  "), "main");
    }

    #[test]
    fn test_normalize_branch_whitespace() {
        assert_eq!(normalize_branch("  main  "), "main");
        assert_eq!(normalize_branch("\tmain\n"), "main");
    }

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
    async fn test_scan_session_with_branch() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "test-branch", "gitBranch": "worktrees/feature-cli (dirty)", "message": {{"content": "Test"}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-20250514", "usage": {{"input_tokens": 10, "output_tokens": 5}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 15, "messageCount": 2}}}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(meta.branch, Some("feature-cli".to_string()));
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
        writeln!(file, r#"{{"type": "summary"}}"#).unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        // Should accumulate all token types from both messages
        // Message 1: input(100) + output(50) + cache_read(1000) + cache_write(500) = 1650
        // Message 2: input(200) + output(75) = 275
        // Total = 1650 + 275 = 1925
        assert_eq!(meta.total_tokens, 1925);
    }

    #[test]
    fn test_normalize_worktree_path_pattern1() {
        // Pattern: /path/to/repo/worktrees/branch → /path/to/repo
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users/test/app/worktrees/feature-x"),
            "/Users/test/app"
        );
        assert_eq!(
            SessionIndexParser::normalize_worktree_path(
                "/path/to/MethodeAristote/app/worktrees/bugfixes"
            ),
            "/path/to/MethodeAristote/app"
        );
    }

    #[test]
    fn test_normalize_worktree_path_pattern2() {
        // Pattern: /path/to/repo/.worktrees/branch → /path/to/repo
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users/test/app/.worktrees/feature-x"),
            "/Users/test/app"
        );
    }

    #[test]
    fn test_normalize_worktree_path_pattern3() {
        // Pattern: /path/to/worktrees/repo-branch → /path/to
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users/test/worktrees/bugfixes"),
            "/Users/test"
        );
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/home/user/worktrees/feature-x"),
            "/home/user"
        );
    }

    #[test]
    fn test_normalize_worktree_path_no_worktree() {
        // No worktree pattern → return as-is
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users/test/app"),
            "/Users/test/app"
        );
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/path/to/MethodeAristote/app"),
            "/path/to/MethodeAristote/app"
        );
    }

    #[test]
    fn test_normalize_worktree_path_double_slash() {
        // Handle double slash from -- encoding: app//worktrees → app/worktrees
        // Real case: -Users-...-app--worktrees-bugfixes → /Users/.../app//worktrees/bugfixes
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users/test/app//worktrees/feature-x"),
            "/Users/test/app"
        );
        assert_eq!(
            SessionIndexParser::normalize_worktree_path(
                "/path/to/MethodeAristote/app//worktrees/bugfixes"
            ),
            "/path/to/MethodeAristote/app"
        );
        // Multiple consecutive slashes
        assert_eq!(
            SessionIndexParser::normalize_worktree_path("/Users///test///app//worktrees/fix"),
            "/Users/test/app"
        );
    }

    #[tokio::test]
    async fn test_tool_usage_extraction() {
        let mut file = NamedTempFile::new().unwrap();

        // Create a session with tool calls (in REAL format from Claude Code JSONL)
        // Format: tool_calls are in message.content blocks, not separate tool_calls field
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "tool-test", "message": {{"content": "Read a file"}}}}"#
        )
        .unwrap();

        // REAL Claude Code format: assistant message with tool_use content blocks
        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "message": {{"content": [{{"type": "tool_use", "name": "Read", "id": "call_1"}}], "usage": {{"input_tokens": 100, "output_tokens": 50}}}}}}"#
        )
        .unwrap();

        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "message": {{"content": [{{"type": "tool_use", "name": "Write", "id": "call_2"}}, {{"type": "tool_use", "name": "Grep", "id": "call_3"}}], "usage": {{"input_tokens": 50, "output_tokens": 25}}}}}}"#
        )
        .unwrap();

        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 225, "messageCount": 3}}}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        // Check tool usage was extracted
        assert_eq!(meta.tool_usage.len(), 3, "Should have 3 unique tools");
        assert_eq!(
            *meta.tool_usage.get("Read").unwrap_or(&0),
            1,
            "Read should be called once"
        );
        assert_eq!(
            *meta.tool_usage.get("Write").unwrap_or(&0),
            1,
            "Write should be called once"
        );
        assert_eq!(
            *meta.tool_usage.get("Grep").unwrap_or(&0),
            1,
            "Grep should be called once"
        );
    }

    #[tokio::test]
    async fn test_message_filtering_excludes_system_messages() {
        let mut file = NamedTempFile::new().unwrap();

        // First message is a system command - should be filtered
        writeln!(
            file,
            r#"{{"type": "user", "sessionId": "filter-test", "timestamp": "2025-01-01T10:00:00Z", "message": {{"content": "<local-command>"}}}}"#
        )
        .unwrap();

        // Second message is a system reminder - should be filtered
        writeln!(
            file,
            r#"{{"type": "user", "timestamp": "2025-01-01T10:01:00Z", "message": {{"content": "<system-reminder>"}}}}"#
        )
        .unwrap();

        // Third message is interrupted - should be filtered
        writeln!(
            file,
            r#"{{"type": "user", "timestamp": "2025-01-01T10:02:00Z", "message": {{"content": "[Request interrupted by user]"}}}}"#
        )
        .unwrap();

        // Fourth message is meaningful - should be captured
        writeln!(
            file,
            r#"{{"type": "user", "timestamp": "2025-01-01T10:03:00Z", "message": {{"content": "Fix the bug in authentication"}}}}"#
        )
        .unwrap();

        writeln!(
            file,
            r#"{{"type": "assistant", "model": "claude-sonnet-4-5-20250929", "message": {{"usage": {{"input_tokens": 100, "output_tokens": 50}}}}}}"#
        )
        .unwrap();

        writeln!(
            file,
            r#"{{"type": "summary", "summary": {{"totalTokens": 150, "messageCount": 5}}}}"#
        )
        .unwrap();

        let parser = SessionIndexParser::new();
        let meta = parser.scan_session(file.path()).await.unwrap();

        // Should have 5 messages total (4 user + 1 assistant)
        assert_eq!(meta.message_count, 5);

        // But first_user_message should skip the first 3 system messages
        // and capture the 4th meaningful message
        assert!(meta.first_user_message.is_some());
        let preview = meta.first_user_message.unwrap();
        assert!(
            preview.contains("Fix the bug"),
            "Should capture meaningful message, got: {}",
            preview
        );
        assert!(
            !preview.contains("<local-command>"),
            "Should not contain system commands"
        );
        assert!(
            !preview.contains("[Request interrupted"),
            "Should not contain noise patterns"
        );
    }
}
