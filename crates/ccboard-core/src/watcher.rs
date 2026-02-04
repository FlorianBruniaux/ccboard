//! File watcher for Claude Code data changes
//!
//! Uses notify with adaptive debouncing for efficient change detection.

use crate::event::{ConfigScope, DataEvent, EventBus};
use crate::store::DataStore;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, trace};

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    /// Base debounce delay
    pub debounce_delay: Duration,

    /// Maximum debounce delay during burst
    pub max_debounce_delay: Duration,

    /// Burst detection threshold (events per second)
    pub burst_threshold: u32,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_delay: Duration::from_millis(500),
            max_debounce_delay: Duration::from_secs(3),
            burst_threshold: 10,
        }
    }
}

/// File watcher that monitors Claude Code directories
pub struct FileWatcher {
    /// Notify watcher instance
    _watcher: RecommendedWatcher,

    /// Shutdown signal
    shutdown_tx: mpsc::Sender<()>,
}

impl FileWatcher {
    /// Start watching Claude Code directories
    pub async fn start(
        claude_home: PathBuf,
        project_path: Option<PathBuf>,
        store: Arc<DataStore>,
        config: WatcherConfig,
    ) -> Result<Self, notify::Error> {
        let (event_tx, mut event_rx) = mpsc::channel::<notify::Result<Event>>(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Create watcher
        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = event_tx.blocking_send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        let mut file_watcher = Self {
            _watcher: watcher,
            shutdown_tx,
        };

        // Watch paths
        file_watcher.watch_path(&claude_home, RecursiveMode::Recursive)?;

        if let Some(ref proj) = project_path {
            let claude_dir = proj.join(".claude");
            if claude_dir.exists() {
                file_watcher.watch_path(&claude_dir, RecursiveMode::Recursive)?;
            }
        }

        info!(claude_home = %claude_home.display(), "File watcher started");

        // Spawn event processor
        let event_bus = store.event_bus().clone();
        tokio::spawn(async move {
            let mut debounce_state = DebounceState::new(config);

            loop {
                tokio::select! {
                    Some(result) = event_rx.recv() => {
                        match result {
                            Ok(event) => {
                                if let Some((data_event, path)) = Self::process_event(&event, &claude_home, project_path.as_deref()) {
                                    if debounce_state.should_emit(&data_event) {
                                        debug!(?data_event, "Emitting file change event");
                                        Self::handle_event(data_event, Some(&path), &store, &event_bus).await;
                                    }
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "File watcher error");
                                event_bus.publish(DataEvent::WatcherError(e.to_string()));
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("File watcher shutting down");
                        break;
                    }
                }
            }
        });

        Ok(file_watcher)
    }

    fn watch_path(&mut self, path: &Path, mode: RecursiveMode) -> Result<(), notify::Error> {
        self._watcher.watch(path, mode)?;
        debug!(path = %path.display(), "Watching path");
        Ok(())
    }

    /// Process a notify event into a DataEvent with its path
    fn process_event(
        event: &Event,
        claude_home: &Path,
        project_path: Option<&Path>,
    ) -> Option<(DataEvent, PathBuf)> {
        // Only process create/modify events
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {}
            _ => return None,
        }

        let path = event.paths.first()?;
        let path_str = path.to_string_lossy();

        trace!(path = %path_str, "Processing file event");

        // Stats cache
        if path
            .file_name()
            .map(|n| n == "stats-cache.json")
            .unwrap_or(false)
        {
            return Some((DataEvent::StatsUpdated, path.clone()));
        }

        // Session files
        if path.extension().map(|e| e == "jsonl").unwrap_or(false) && path_str.contains("projects")
        {
            let session_id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            return Some((DataEvent::SessionUpdated(session_id), path.clone()));
        }

        // Global settings
        if *path == claude_home.join("settings.json") {
            return Some((DataEvent::ConfigChanged(ConfigScope::Global), path.clone()));
        }

        // Project settings
        if let Some(proj) = project_path {
            if *path == proj.join(".claude").join("settings.json") {
                return Some((
                    DataEvent::ConfigChanged(ConfigScope::Project(
                        proj.to_string_lossy().to_string(),
                    )),
                    path.clone(),
                ));
            }
            if *path == proj.join(".claude").join("settings.local.json") {
                return Some((
                    DataEvent::ConfigChanged(ConfigScope::Local(
                        proj.to_string_lossy().to_string(),
                    )),
                    path.clone(),
                ));
            }
        }

        // MCP config
        if path
            .file_name()
            .map(|n| n == "claude_desktop_config.json")
            .unwrap_or(false)
        {
            return Some((DataEvent::ConfigChanged(ConfigScope::Mcp), path.clone()));
        }

        None
    }

    /// Handle a data event by updating the store
    async fn handle_event(
        event: DataEvent,
        path: Option<&Path>,
        store: &DataStore,
        event_bus: &EventBus,
    ) {
        match &event {
            DataEvent::StatsUpdated => {
                store.reload_stats().await;
            }
            DataEvent::SessionUpdated(_id) | DataEvent::SessionCreated(_id) => {
                // Update session with path
                if let Some(p) = path {
                    store.update_session(p).await;
                }
            }
            DataEvent::ConfigChanged(_scope) => {
                // Reload settings
                store.reload_settings().await;
            }
            _ => {}
        }

        event_bus.publish(event);
    }

    /// Stop the watcher
    pub async fn stop(&self) {
        let _ = self.shutdown_tx.send(()).await;
    }
}

/// Debounce state for adaptive debouncing
struct DebounceState {
    config: WatcherConfig,
    last_events: std::collections::HashMap<String, std::time::Instant>,
    event_count_window: std::collections::VecDeque<std::time::Instant>,
}

impl DebounceState {
    fn new(config: WatcherConfig) -> Self {
        Self {
            config,
            last_events: std::collections::HashMap::new(),
            event_count_window: std::collections::VecDeque::new(),
        }
    }

    fn should_emit(&mut self, event: &DataEvent) -> bool {
        let now = std::time::Instant::now();
        let key = Self::event_key(event);

        // Track event rate for burst detection
        self.event_count_window.push_back(now);
        while self
            .event_count_window
            .front()
            .map(|t| now.duration_since(*t) > Duration::from_secs(1))
            .unwrap_or(false)
        {
            self.event_count_window.pop_front();
        }

        // Calculate adaptive delay
        let delay = if self.event_count_window.len() as u32 > self.config.burst_threshold {
            self.config.max_debounce_delay
        } else {
            self.config.debounce_delay
        };

        // Check if enough time has passed
        if let Some(last) = self.last_events.get(&key) {
            if now.duration_since(*last) < delay {
                trace!(key = %key, "Debouncing event");
                return false;
            }
        }

        self.last_events.insert(key, now);
        true
    }

    fn event_key(event: &DataEvent) -> String {
        match event {
            DataEvent::StatsUpdated => "stats".to_string(),
            DataEvent::SessionCreated(id) | DataEvent::SessionUpdated(id) => {
                format!("session:{}", id)
            }
            DataEvent::ConfigChanged(scope) => format!("config:{:?}", scope),
            DataEvent::AnalyticsUpdated => "analytics".to_string(),
            DataEvent::LoadCompleted => "load".to_string(),
            DataEvent::WatcherError(_) => "error".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounce_state_basic() {
        let config = WatcherConfig {
            debounce_delay: Duration::from_millis(100),
            max_debounce_delay: Duration::from_millis(500),
            burst_threshold: 5,
        };
        let mut state = DebounceState::new(config);

        // First event should pass
        assert!(state.should_emit(&DataEvent::StatsUpdated));

        // Immediate second should be debounced
        assert!(!state.should_emit(&DataEvent::StatsUpdated));

        // Different event type should pass
        assert!(state.should_emit(&DataEvent::SessionUpdated("test".to_string())));
    }

    #[test]
    fn test_process_event_stats() {
        let claude_home = PathBuf::from("/home/user/.claude");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from("/home/user/.claude/stats-cache.json")],
            ..Default::default()
        };

        let result = FileWatcher::process_event(&event, &claude_home, None);
        assert!(matches!(result, Some((DataEvent::StatsUpdated, _))));
    }

    #[test]
    fn test_process_event_session() {
        let claude_home = PathBuf::from("/home/user/.claude");
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content,
            )),
            paths: vec![PathBuf::from(
                "/home/user/.claude/projects/-test/abc123.jsonl",
            )],
            ..Default::default()
        };

        let result = FileWatcher::process_event(&event, &claude_home, None);
        assert!(matches!(result, Some((DataEvent::SessionUpdated(id), _)) if id == "abc123"));
    }
}
