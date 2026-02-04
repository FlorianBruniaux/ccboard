//! Data store with DashMap + parking_lot::RwLock
//!
//! Uses DashMap for sessions (per-entry locking) and parking_lot::RwLock
//! for stats/settings (better fairness than std::sync::RwLock).

use crate::analytics::{AnalyticsData, Period};
use crate::cache::MetadataCache;
use crate::error::{DegradedState, LoadReport};
use crate::event::{ConfigScope, DataEvent, EventBus};
use crate::models::{
    BillingBlockManager, InvocationStats, MergedConfig, SessionMetadata, StatsCache,
};
use crate::parsers::{
    InvocationParser, McpConfig, Rules, SessionIndexParser, SettingsParser, StatsParser,
};
use dashmap::DashMap;
use moka::future::Cache;
use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Configuration for the data store
#[derive(Debug, Clone)]
pub struct DataStoreConfig {
    /// Maximum session metadata entries to keep
    pub max_session_metadata_count: usize,

    /// Maximum size for session content cache in MB
    pub max_session_content_cache_mb: usize,

    /// Maximum concurrent session scans
    pub max_concurrent_scans: usize,

    /// Stats parser retry count
    pub stats_retry_count: u32,

    /// Stats parser retry delay
    pub stats_retry_delay: Duration,
}

impl Default for DataStoreConfig {
    fn default() -> Self {
        Self {
            max_session_metadata_count: 10_000,
            max_session_content_cache_mb: 100,
            max_concurrent_scans: 8,
            stats_retry_count: 3,
            stats_retry_delay: Duration::from_millis(100),
        }
    }
}

/// Central data store for ccboard
///
/// Thread-safe access to all Claude Code data.
/// Uses DashMap for sessions (high contention) and RwLock for stats/settings (low contention).
pub struct DataStore {
    /// Path to Claude home directory
    claude_home: PathBuf,

    /// Current project path (if focused)
    project_path: Option<PathBuf>,

    /// Configuration
    config: DataStoreConfig,

    /// Stats cache (low contention, frequent reads)
    stats: RwLock<Option<StatsCache>>,

    /// Merged settings
    settings: RwLock<MergedConfig>,

    /// MCP server configuration
    mcp_config: RwLock<Option<McpConfig>>,

    /// Rules from CLAUDE.md files
    rules: RwLock<Rules>,

    /// Invocation statistics (agents, commands, skills)
    invocation_stats: RwLock<InvocationStats>,

    /// Billing blocks (5h usage tracking)
    billing_blocks: RwLock<BillingBlockManager>,

    /// Analytics data cache (invalidated on stats/sessions update)
    analytics_cache: RwLock<Option<AnalyticsData>>,

    /// Session metadata (high contention with many entries)
    /// Arc<SessionMetadata> for cheap cloning (8 bytes vs ~400 bytes)
    sessions: DashMap<String, Arc<SessionMetadata>>,

    /// Session content cache (LRU for on-demand loading)
    #[allow(dead_code)]
    session_content_cache: Cache<String, Vec<String>>,

    /// Event bus for notifying subscribers
    event_bus: EventBus,

    /// Current degraded state
    degraded_state: RwLock<DegradedState>,

    /// Metadata cache for 90% startup speedup (optional)
    metadata_cache: Option<Arc<MetadataCache>>,
}

impl DataStore {
    /// Create a new data store
    pub fn new(
        claude_home: PathBuf,
        project_path: Option<PathBuf>,
        config: DataStoreConfig,
    ) -> Self {
        let session_content_cache = Cache::builder()
            .max_capacity((config.max_session_content_cache_mb * 1024 * 1024 / 1000) as u64) // Rough estimate
            .time_to_idle(Duration::from_secs(300)) // 5 min idle expiry
            .build();

        // Create metadata cache in ~/.claude/cache/
        let metadata_cache = {
            let cache_dir = claude_home.join("cache");
            match MetadataCache::new(&cache_dir) {
                Ok(cache) => {
                    debug!(path = %cache_dir.display(), "Metadata cache enabled");
                    Some(Arc::new(cache))
                }
                Err(e) => {
                    warn!(error = %e, "Failed to create metadata cache, running without cache");
                    None
                }
            }
        };

        Self {
            claude_home,
            project_path,
            config,
            stats: RwLock::new(None),
            settings: RwLock::new(MergedConfig::default()),
            mcp_config: RwLock::new(None),
            rules: RwLock::new(Rules::default()),
            invocation_stats: RwLock::new(InvocationStats::new()),
            billing_blocks: RwLock::new(BillingBlockManager::new()),
            analytics_cache: RwLock::new(None),
            sessions: DashMap::new(),
            session_content_cache,
            event_bus: EventBus::default_capacity(),
            degraded_state: RwLock::new(DegradedState::Healthy),
            metadata_cache,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(claude_home: PathBuf, project_path: Option<PathBuf>) -> Self {
        Self::new(claude_home, project_path, DataStoreConfig::default())
    }

    /// Get the event bus for subscribing to updates
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get current degraded state
    pub fn degraded_state(&self) -> DegradedState {
        self.degraded_state.read().clone()
    }

    /// Initial load of all data with LoadReport for graceful degradation
    pub async fn initial_load(&self) -> LoadReport {
        let mut report = LoadReport::new();

        info!(claude_home = %self.claude_home.display(), "Starting initial data load");

        // Load stats
        self.load_stats(&mut report).await;

        // Load settings
        self.load_settings(&mut report).await;

        // Load MCP configuration
        self.load_mcp_config(&mut report).await;

        // Load rules
        self.load_rules(&mut report).await;

        // Scan sessions
        self.scan_sessions(&mut report).await;

        // Determine degraded state
        self.update_degraded_state(&report);

        // Notify subscribers
        self.event_bus.publish(DataEvent::LoadCompleted);

        info!(
            stats_loaded = report.stats_loaded,
            settings_loaded = report.settings_loaded,
            sessions_scanned = report.sessions_scanned,
            sessions_failed = report.sessions_failed,
            errors = report.errors.len(),
            "Initial load complete"
        );

        report
    }

    /// Load stats cache
    async fn load_stats(&self, report: &mut LoadReport) {
        let stats_path = self.claude_home.join("stats-cache.json");
        let parser = StatsParser::new()
            .with_retries(self.config.stats_retry_count, self.config.stats_retry_delay);

        if let Some(stats) = parser.parse_graceful(&stats_path, report).await {
            let mut guard = self.stats.write();
            *guard = Some(stats);
            debug!("Stats loaded successfully");
        }
    }

    /// Load and merge settings
    async fn load_settings(&self, report: &mut LoadReport) {
        let parser = SettingsParser::new();
        let merged = parser
            .load_merged(&self.claude_home, self.project_path.as_deref(), report)
            .await;

        let mut guard = self.settings.write();
        *guard = merged;
        debug!("Settings loaded and merged");
    }

    /// Load MCP server configuration
    async fn load_mcp_config(&self, report: &mut LoadReport) {
        match McpConfig::load(&self.claude_home) {
            Ok(Some(config)) => {
                let server_count = config.servers.len();
                let mut guard = self.mcp_config.write();
                *guard = Some(config);
                debug!(server_count, "MCP config loaded successfully");
            }
            Ok(None) => {
                debug!("No MCP config found (optional)");
            }
            Err(e) => {
                use crate::error::LoadError;
                report.add_error(LoadError::error(
                    "mcp_config",
                    format!("Failed to parse MCP config: {}", e),
                ));
            }
        }
    }

    /// Load rules from CLAUDE.md files
    async fn load_rules(&self, report: &mut LoadReport) {
        match Rules::load(&self.claude_home, self.project_path.as_deref()) {
            Ok(rules) => {
                let has_global = rules.global.is_some();
                let has_project = rules.project.is_some();
                let mut guard = self.rules.write();
                *guard = rules;
                debug!(has_global, has_project, "Rules loaded");
            }
            Err(e) => {
                use crate::error::LoadError;
                report.add_error(LoadError::error(
                    "rules",
                    format!("Failed to load rules: {}", e),
                ));
            }
        }
    }

    /// Scan all sessions
    async fn scan_sessions(&self, report: &mut LoadReport) {
        let projects_dir = self.claude_home.join("projects");

        if !projects_dir.exists() {
            report.add_warning(
                "sessions",
                format!("Projects directory not found: {}", projects_dir.display()),
            );
            return;
        }

        let mut parser =
            SessionIndexParser::new().with_concurrency(self.config.max_concurrent_scans);

        // Enable metadata cache if available (90% speedup)
        if let Some(ref cache) = self.metadata_cache {
            parser = parser.with_cache(cache.clone());
        }

        let sessions = parser.scan_all(&projects_dir, report).await;

        // Enforce max count limit
        let sessions_to_add: Vec<_> = if sessions.len() > self.config.max_session_metadata_count {
            warn!(
                total = sessions.len(),
                limit = self.config.max_session_metadata_count,
                "Session count exceeds limit, keeping most recent"
            );

            let mut sorted = sessions;
            sorted.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
            sorted.truncate(self.config.max_session_metadata_count);
            sorted
        } else {
            sessions
        };

        // Insert into DashMap (wrap in Arc for cheap cloning)
        for session in sessions_to_add {
            self.sessions.insert(session.id.clone(), Arc::new(session));
        }

        debug!(count = self.sessions.len(), "Sessions indexed");
    }

    /// Update degraded state based on load report
    fn update_degraded_state(&self, report: &LoadReport) {
        let mut state = self.degraded_state.write();

        if report.has_fatal_errors() {
            *state = DegradedState::ReadOnly {
                reason: "Fatal errors during load".to_string(),
            };
            return;
        }

        let mut missing = Vec::new();

        if !report.stats_loaded {
            missing.push("stats".to_string());
        }
        if !report.settings_loaded {
            missing.push("settings".to_string());
        }
        if report.sessions_failed > 0 {
            missing.push(format!("{} sessions", report.sessions_failed));
        }

        if missing.is_empty() {
            *state = DegradedState::Healthy;
        } else {
            *state = DegradedState::PartialData {
                missing: missing.clone(),
                reason: format!("Missing: {}", missing.join(", ")),
            };
        }
    }

    // ===================
    // Read accessors
    // ===================

    /// Get a clone of stats
    pub fn stats(&self) -> Option<StatsCache> {
        self.stats.read().clone()
    }

    /// Calculate context window saturation from current sessions
    pub fn context_window_stats(&self) -> crate::models::ContextWindowStats {
        // Clone Arc (cheap) to avoid lifetime issues with DashMap iterators
        let sessions: Vec<_> = self
            .sessions
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();
        // Dereference Arc to get &SessionMetadata
        let refs: Vec<_> = sessions.iter().map(|s| s.as_ref()).collect();
        crate::models::StatsCache::calculate_context_saturation(&refs, 30)
    }

    /// Get merged settings
    pub fn settings(&self) -> MergedConfig {
        self.settings.read().clone()
    }

    /// Get MCP server configuration
    pub fn mcp_config(&self) -> Option<McpConfig> {
        self.mcp_config.read().clone()
    }

    /// Get rules
    pub fn rules(&self) -> Rules {
        self.rules.read().clone()
    }

    /// Get invocation statistics
    pub fn invocation_stats(&self) -> InvocationStats {
        self.invocation_stats.read().clone()
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get session by ID
    /// Returns Arc<SessionMetadata> for cheap cloning
    pub fn get_session(&self, id: &str) -> Option<Arc<SessionMetadata>> {
        self.sessions.get(id).map(|r| Arc::clone(r.value()))
    }

    /// Get analytics data for a period (cached)
    ///
    /// Returns cached analytics if available, otherwise None.
    /// Call `compute_analytics()` to compute and cache.
    pub fn analytics(&self) -> Option<AnalyticsData> {
        let analytics = self.analytics_cache.read().clone();
        debug!(
            has_analytics = analytics.is_some(),
            "analytics() getter called"
        );
        analytics
    }

    /// Compute and cache analytics data for a period
    ///
    /// This is a CPU-intensive operation (trends, forecasting, patterns).
    /// For 1000+ sessions, this may take 100-300ms, so it's offloaded
    /// to a blocking task.
    ///
    /// Cache is invalidated on stats reload or session updates (EventBus pattern).
    pub async fn compute_analytics(&self, period: Period) {
        let sessions: Vec<_> = self
            .sessions
            .iter()
            .map(|r| Arc::clone(r.value()))
            .collect();

        info!(
            session_count = sessions.len(),
            period = ?period,
            "compute_analytics() ENTRY"
        );

        // Offload to blocking task for CPU-intensive computation
        let analytics =
            tokio::task::spawn_blocking(move || AnalyticsData::compute(&sessions, period)).await;

        match analytics {
            Ok(data) => {
                info!(
                    insights_count = data.insights.len(),
                    "compute_analytics() computed data"
                );
                let mut guard = self.analytics_cache.write();
                *guard = Some(data);
                self.event_bus.publish(DataEvent::AnalyticsUpdated);
                info!("compute_analytics() EXIT - cached and event published");
            }
            Err(e) => {
                warn!(error = %e, "Failed to compute analytics (task panicked)");
            }
        }
    }

    /// Invalidate analytics cache (called on data changes)
    ///
    /// Note: Currently unused to prevent aggressive invalidation.
    /// Kept for future use if smart invalidation is needed.
    #[allow(dead_code)]
    fn invalidate_analytics_cache(&self) {
        let mut guard = self.analytics_cache.write();
        *guard = None;
        debug!("Analytics cache invalidated");
    }

    /// Get all session IDs
    pub fn session_ids(&self) -> Vec<String> {
        self.sessions.iter().map(|r| r.key().clone()).collect()
    }

    /// Clear session content cache (for memory optimization on F5)
    pub fn clear_session_content_cache(&self) {
        self.session_content_cache.invalidate_all();
        debug!("Session content cache cleared");
    }

    /// Get sessions grouped by project
    /// Returns Arc<SessionMetadata> for cheap cloning
    pub fn sessions_by_project(
        &self,
    ) -> std::collections::HashMap<String, Vec<Arc<SessionMetadata>>> {
        let mut by_project = std::collections::HashMap::new();

        for entry in self.sessions.iter() {
            let session = Arc::clone(entry.value());
            by_project
                .entry(session.project_path.clone())
                .or_insert_with(Vec::new)
                .push(session);
        }

        // Sort sessions within each project by timestamp (newest first)
        for sessions in by_project.values_mut() {
            sessions.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
        }

        by_project
    }

    /// Get recent sessions (sorted by last timestamp, newest first)
    /// Returns Arc<SessionMetadata> for cheap cloning
    pub fn recent_sessions(&self, limit: usize) -> Vec<Arc<SessionMetadata>> {
        let mut sessions: Vec<_> = self
            .sessions
            .iter()
            .map(|r| Arc::clone(r.value()))
            .collect();
        sessions.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));
        sessions.truncate(limit);
        sessions
    }

    // ===================
    // Update methods (called by watcher)
    // ===================

    /// Reload stats (called on file change)
    pub async fn reload_stats(&self) {
        let stats_path = self.claude_home.join("stats-cache.json");
        let parser = StatsParser::new()
            .with_retries(self.config.stats_retry_count, self.config.stats_retry_delay);

        let mut report = LoadReport::new();
        if let Some(stats) = parser.parse_graceful(&stats_path, &mut report).await {
            let mut guard = self.stats.write();
            *guard = Some(stats);

            // Don't invalidate analytics - it will auto-recompute if needed
            // Instead, just publish the event so UI can decide whether to recompute
            self.event_bus.publish(DataEvent::StatsUpdated);
            debug!("Stats reloaded");
        }
    }

    /// Reload settings from files (called when settings change)
    pub async fn reload_settings(&self) {
        let parser = SettingsParser::new();
        let merged = parser
            .load_merged(
                &self.claude_home,
                self.project_path.as_deref(),
                &mut LoadReport::new(),
            )
            .await;

        {
            let mut guard = self.settings.write();
            *guard = merged;
        }

        self.event_bus
            .publish(DataEvent::ConfigChanged(ConfigScope::Global));
        debug!("Settings reloaded");
    }

    /// Add or update a session (called when session file changes)
    pub async fn update_session(&self, path: &Path) {
        let parser = SessionIndexParser::new();

        match parser.scan_session(path).await {
            Ok(meta) => {
                let id = meta.id.clone();
                let is_new = !self.sessions.contains_key(&id);

                self.sessions.insert(id.clone(), Arc::new(meta));

                // Don't invalidate analytics on every session update - too aggressive
                // Analytics will be recomputed on demand or periodically
                // Only invalidate on significant changes (detected by UI)

                if is_new {
                    self.event_bus.publish(DataEvent::SessionCreated(id));
                } else {
                    self.event_bus.publish(DataEvent::SessionUpdated(id));
                }
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to update session");
            }
        }
    }

    /// Compute invocation statistics from all sessions
    ///
    /// This scans all session files to count agent/command/skill invocations.
    /// Should be called after initial load or when sessions are updated.
    pub async fn compute_invocations(&self) {
        let paths: Vec<_> = self
            .sessions
            .iter()
            .map(|r| r.value().file_path.clone())
            .collect();

        debug!(session_count = paths.len(), "Computing invocation stats");

        let parser = InvocationParser::new();
        let stats = parser.scan_sessions(&paths).await;

        let mut guard = self.invocation_stats.write();
        *guard = stats;

        debug!(
            agents = guard.agents.len(),
            commands = guard.commands.len(),
            skills = guard.skills.len(),
            total = guard.total_invocations(),
            "Invocation stats computed"
        );

        // Note: Using LoadCompleted as there's no specific invocation stats event
        self.event_bus.publish(DataEvent::LoadCompleted);
    }

    /// Compute billing blocks from all sessions
    ///
    /// This scans all sessions with timestamps and aggregates usage into 5-hour billing blocks.
    /// Uses real model pricing based on token breakdown for accurate cost calculation.
    pub async fn compute_billing_blocks(&self) {
        debug!("Computing billing blocks from sessions with real pricing");

        let mut manager = BillingBlockManager::new();
        let mut sessions_with_timestamps = 0;
        let mut sessions_without_timestamps = 0;

        for session in self.sessions.iter() {
            let metadata = session.value();

            // Skip sessions without timestamps
            let Some(timestamp) = &metadata.first_timestamp else {
                sessions_without_timestamps += 1;
                continue;
            };

            sessions_with_timestamps += 1;

            // Get model for this session (use first model, or "unknown")
            let model = metadata
                .models_used
                .first()
                .map(|s| s.as_str())
                .unwrap_or("unknown");

            // Calculate real cost using pricing table
            let cost = crate::pricing::calculate_cost(
                model,
                metadata.input_tokens,
                metadata.output_tokens,
                metadata.cache_creation_tokens,
                metadata.cache_read_tokens,
            );

            manager.add_usage(
                timestamp,
                metadata.input_tokens,
                metadata.output_tokens,
                metadata.cache_creation_tokens,
                metadata.cache_read_tokens,
                cost,
            );
        }

        debug!(
            sessions_with_timestamps,
            sessions_without_timestamps,
            blocks = manager.get_all_blocks().len(),
            "Billing blocks computed with real pricing"
        );

        let mut guard = self.billing_blocks.write();
        *guard = manager;

        self.event_bus.publish(DataEvent::LoadCompleted);
    }

    /// Get billing blocks (read-only access)
    pub fn billing_blocks(&self) -> parking_lot::RwLockReadGuard<'_, BillingBlockManager> {
        self.billing_blocks.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_data_store_creation() {
        let dir = tempdir().unwrap();
        let store = DataStore::with_defaults(dir.path().to_path_buf(), None);

        assert_eq!(store.session_count(), 0);
        assert!(store.stats().is_none());
        assert!(store.degraded_state().is_healthy());
    }

    #[tokio::test]
    async fn test_initial_load_missing_dir() {
        let dir = tempdir().unwrap();
        let store = DataStore::with_defaults(dir.path().join("nonexistent"), None);

        let report = store.initial_load().await;

        // Should have warnings but not crash
        assert!(report.has_errors());
        assert!(store.degraded_state().is_degraded());
    }

    #[tokio::test]
    async fn test_initial_load_with_stats() {
        let dir = tempdir().unwrap();
        let claude_home = dir.path();

        // Create stats file with new format
        std::fs::write(
            claude_home.join("stats-cache.json"),
            r#"{"version": 2, "totalSessions": 5, "totalMessages": 100, "modelUsage": {"test": {"inputTokens": 600, "outputTokens": 400}}}"#,
        )
        .unwrap();

        // Create projects dir
        std::fs::create_dir_all(claude_home.join("projects")).unwrap();

        let store = DataStore::with_defaults(claude_home.to_path_buf(), None);
        let report = store.initial_load().await;

        assert!(report.stats_loaded);
        let stats = store.stats().unwrap();
        assert_eq!(stats.total_tokens(), 1000);
        assert_eq!(stats.session_count(), 5);
    }

    #[tokio::test]
    async fn test_event_bus_subscription() {
        let dir = tempdir().unwrap();
        let store = DataStore::with_defaults(dir.path().to_path_buf(), None);

        let mut rx = store.event_bus().subscribe();

        // Trigger load completed
        store.event_bus().publish(DataEvent::StatsUpdated);

        let event = rx.recv().await.unwrap();
        assert!(matches!(event, DataEvent::StatsUpdated));
    }

    #[tokio::test]
    async fn test_analytics_cache_and_invalidation() {
        use crate::models::session::SessionMetadata;
        use chrono::Utc;

        let dir = tempdir().unwrap();
        let store = DataStore::with_defaults(dir.path().to_path_buf(), None);

        // Add test sessions
        let now = Utc::now();
        for i in 0..10 {
            let total_tokens = 1000 * (i as u64 + 1);
            let session = SessionMetadata {
                id: format!("test-{}", i),
                file_path: std::path::PathBuf::from(format!("/test-{}.jsonl", i)),
                project_path: "/test".to_string(),
                first_timestamp: Some(now - chrono::Duration::days(i)),
                last_timestamp: Some(now),
                message_count: 10,
                total_tokens,
                input_tokens: total_tokens / 2,
                output_tokens: total_tokens / 3,
                cache_creation_tokens: total_tokens / 10,
                cache_read_tokens: total_tokens - (total_tokens / 2 + total_tokens / 3 + total_tokens / 10),
                models_used: vec!["sonnet".to_string()],
                file_size_bytes: 1024,
                first_user_message: None,
                has_subagents: false,
                duration_seconds: Some(1800),
            };
            store.sessions.insert(session.id.clone(), Arc::new(session));
        }

        // Initially no analytics
        assert!(store.analytics().is_none());

        // Compute analytics
        store.compute_analytics(Period::last_7d()).await;

        // Analytics should be cached
        let analytics1 = store.analytics().expect("Analytics should be cached");
        assert!(!analytics1.trends.is_empty());
        assert_eq!(analytics1.period, Period::last_7d());

        // Invalidate by reloading stats
        store.invalidate_analytics_cache();
        assert!(store.analytics().is_none(), "Cache should be invalidated");

        // Re-compute with different period
        store.compute_analytics(Period::last_30d()).await;
        let analytics2 = store.analytics().expect("Analytics should be re-cached");
        assert_eq!(analytics2.period, Period::last_30d());
    }
}
