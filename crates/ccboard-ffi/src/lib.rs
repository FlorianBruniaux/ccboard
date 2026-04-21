uniffi::setup_scaffolding!("ccboard_ffi");

use ccboard_core::{
    event::DataEvent,
    models::session::MessageRole,
    store::DataStore,
};
use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

// ──────────────────────────────────────────────────────────────────────────────
// Tokio runtime singleton
// ──────────────────────────────────────────────────────────────────────────────

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime init failed"))
}

// ──────────────────────────────────────────────────────────────────────────────
// Handle (entry point)
// ──────────────────────────────────────────────────────────────────────────────

/// Main handle — holds one DataStore, shared across all FFI calls.
/// Create with `ccboard_init()`, keep alive for the app lifetime.
#[derive(uniffi::Object)]
pub struct CcboardHandle {
    store: Arc<DataStore>,
}

/// Initialize ccboard with the path to ~/.claude.
/// Call once at app startup. The store is loaded eagerly.
#[uniffi::export]
pub fn ccboard_init(claude_home: String) -> Arc<CcboardHandle> {
    let path = PathBuf::from(claude_home);
    let store = Arc::new(DataStore::with_defaults(path, None));
    runtime().block_on(async { store.initial_load().await });
    Arc::new(CcboardHandle { store })
}

// ──────────────────────────────────────────────────────────────────────────────
// FFI data types — flat structs, no HashMap, no tuples
// ──────────────────────────────────────────────────────────────────────────────

#[derive(uniffi::Record)]
pub struct FfiProject {
    pub id: String,
    pub name: String,
    pub session_count: u64,
    pub total_tokens: u64,
    pub total_messages: u64,
}

#[derive(uniffi::Record)]
pub struct FfiModelSegment {
    pub model: String,
    pub message_count: u64,
}

#[derive(uniffi::Record)]
pub struct FfiToolCount {
    pub name: String,
    pub count: u64,
}

#[derive(uniffi::Record)]
pub struct FfiSessionSummary {
    pub id: String,
    pub project_id: String,
    /// ISO 8601 UTC string, e.g. "2025-04-14T10:30:00Z"
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub message_count: u64,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub models_used: Vec<String>,
    pub model_segments: Vec<FfiModelSegment>,
    pub first_user_message: Option<String>,
    pub has_subagents: bool,
    pub parent_session_id: Option<String>,
    pub duration_seconds: Option<u64>,
    pub branch: Option<String>,
    pub tool_usage: Vec<FfiToolCount>,
    /// "claude_code" | "cursor" | "codex" | "open_code"
    pub source_tool: String,
    pub lines_added: u64,
    pub lines_removed: u64,
}

#[derive(uniffi::Record)]
pub struct FfiTokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

#[derive(uniffi::Record)]
pub struct FfiToolCall {
    pub name: String,
    pub id: String,
    /// JSON-encoded input parameters
    pub input_json: String,
}

#[derive(uniffi::Record)]
pub struct FfiToolResult {
    pub tool_call_id: String,
    pub is_error: bool,
    pub content: String,
}

#[derive(uniffi::Record)]
pub struct FfiMessage {
    /// "user" | "assistant" | "system"
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
    pub model: Option<String>,
    pub tokens: Option<FfiTokenUsage>,
    pub tool_calls: Vec<FfiToolCall>,
    pub tool_results: Vec<FfiToolResult>,
}

#[derive(uniffi::Record)]
pub struct FfiDailyActivity {
    /// "2025-04-14"
    pub date: String,
    pub message_count: u64,
    pub session_count: u64,
    pub tool_call_count: u64,
}

#[derive(uniffi::Record)]
pub struct FfiModelUsage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
}

#[derive(uniffi::Record)]
pub struct FfiStats {
    pub total_sessions: u64,
    pub total_messages: u64,
    pub daily_activity: Vec<FfiDailyActivity>,
    pub model_usage: Vec<FfiModelUsage>,
    pub this_month_cost: f64,
    pub cache_hit_ratio: f64,
}

#[derive(uniffi::Record)]
pub struct FfiHook {
    pub command: String,
    pub is_async: bool,
    pub timeout_secs: Option<u32>,
}

#[derive(uniffi::Record)]
pub struct FfiHookGroup {
    /// "PreToolUse" | "PostToolUse" | "Notification" | etc.
    pub event_name: String,
    pub matcher: Option<String>,
    pub hooks: Vec<FfiHook>,
}

#[derive(uniffi::Record)]
pub struct FfiMcpServer {
    pub name: String,
    /// "stdio" | "http" | ""
    pub server_type: String,
    pub command: String,
    pub args: Vec<String>,
    pub url: Option<String>,
}

#[derive(uniffi::Record)]
pub struct FfiSearchResult {
    pub session_id: String,
    pub project_id: String,
    pub snippet: String,
    pub score: f64,
}

// ──────────────────────────────────────────────────────────────────────────────
// Event callbacks
// ──────────────────────────────────────────────────────────────────────────────

#[uniffi::export(with_foreign)]
pub trait FfiEventListener: Send + Sync {
    fn on_load_completed(&self, session_count: u64);
    fn on_stats_updated(&self);
    fn on_session_changed(&self, id: String);
    fn on_config_changed(&self);
}

// ──────────────────────────────────────────────────────────────────────────────
// CcboardHandle methods
// ──────────────────────────────────────────────────────────────────────────────

#[uniffi::export]
impl CcboardHandle {
    /// Total number of indexed sessions.
    pub fn session_count(&self) -> u64 {
        self.store.session_count() as u64
    }

    /// All projects derived from indexed sessions, sorted by name.
    pub fn get_projects(&self) -> Vec<FfiProject> {
        let by_project = self.store.sessions_by_project();
        let mut projects: Vec<FfiProject> = by_project
            .iter()
            .map(|(project_id, sessions)| {
                let name = project_display_name(project_id);
                let total_tokens: u64 = sessions.iter().map(|s| s.total_tokens).sum();
                let total_messages: u64 = sessions.iter().map(|s| s.message_count).sum();
                FfiProject {
                    id: project_id.clone(),
                    name,
                    session_count: sessions.len() as u64,
                    total_tokens,
                    total_messages,
                }
            })
            .collect();
        projects.sort_by(|a, b| a.name.cmp(&b.name));
        projects
    }

    /// Sessions for a specific project, sorted newest first.
    pub fn get_sessions_for_project(&self, project_id: String) -> Vec<FfiSessionSummary> {
        let by_project = self.store.sessions_by_project();
        by_project
            .get(&project_id)
            .map(|sessions| sessions.iter().map(|s| session_to_ffi(s)).collect())
            .unwrap_or_default()
    }

    /// Most recent sessions across all projects.
    pub fn get_recent_sessions(&self, limit: u32) -> Vec<FfiSessionSummary> {
        self.store
            .recent_sessions(limit as usize)
            .iter()
            .map(|s| session_to_ffi(s))
            .collect()
    }

    /// Full conversation content for a session. Lazy-loaded from JSONL.
    pub fn load_session_content(&self, session_id: String) -> Vec<FfiMessage> {
        runtime().block_on(async {
            match self.store.load_session_content(&session_id).await {
                Ok(messages) => messages
                    .into_iter()
                    .map(|m| {
                        let role = match m.role {
                            MessageRole::User => "user",
                            MessageRole::Assistant => "assistant",
                            MessageRole::System => "system",
                        };
                        FfiMessage {
                            role: role.to_string(),
                            content: m.content,
                            timestamp: m
                                .timestamp
                                .map(|t| t.to_rfc3339()),
                            model: m.model,
                            tokens: m.tokens.map(|t| FfiTokenUsage {
                                input_tokens: t.input_tokens,
                                output_tokens: t.output_tokens,
                                cache_creation_tokens: t.cache_write_tokens,
                                cache_read_tokens: t.cache_read_tokens,
                            }),
                            tool_calls: m
                                .tool_calls
                                .into_iter()
                                .map(|tc| FfiToolCall {
                                    name: tc.name,
                                    id: tc.id,
                                    input_json: tc.input.to_string(),
                                })
                                .collect(),
                            tool_results: m
                                .tool_results
                                .into_iter()
                                .map(|tr| FfiToolResult {
                                    tool_call_id: tr.tool_call_id,
                                    is_error: tr.is_error,
                                    content: tr.content,
                                })
                                .collect(),
                        }
                    })
                    .collect(),
                Err(_) => vec![],
            }
        })
    }

    /// Stats from ~/.claude/stats-cache.json.
    pub fn get_stats(&self) -> Option<FfiStats> {
        let stats = self.store.stats()?;

        let daily_activity = stats
            .daily_activity
            .iter()
            .map(|d| FfiDailyActivity {
                date: d.date.clone(),
                message_count: d.message_count,
                session_count: d.session_count,
                tool_call_count: d.tool_call_count,
            })
            .collect();

        let model_usage = stats
            .model_usage
            .iter()
            .map(|(model, usage)| FfiModelUsage {
                model: model.clone(),
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_read_tokens: usage.cache_read_input_tokens,
                cache_creation_tokens: usage.cache_creation_input_tokens,
            })
            .collect();

        // Cache hit ratio: cache_read / (input + cache_read)
        let total_input: u64 = stats
            .model_usage
            .values()
            .map(|u| u.input_tokens + u.cache_read_input_tokens)
            .sum();
        let total_cache_read: u64 = stats
            .model_usage
            .values()
            .map(|u| u.cache_read_input_tokens)
            .sum();
        let cache_hit_ratio = if total_input > 0 {
            total_cache_read as f64 / total_input as f64
        } else {
            0.0
        };

        Some(FfiStats {
            total_sessions: stats.total_sessions,
            total_messages: stats.total_messages,
            daily_activity,
            model_usage,
            this_month_cost: 0.0, // Claudoscope computes cost via its pricing tables
            cache_hit_ratio,
        })
    }

    /// Hook groups from merged settings.json.
    pub fn get_hooks(&self) -> Vec<FfiHookGroup> {
        let config = self.store.settings();
        let hooks_map = match config.merged.hooks {
            Some(map) => map,
            None => return vec![],
        };

        let mut result = Vec::new();
        for (event_name, groups) in hooks_map {
            for group in groups {
                let hooks = group
                    .hooks
                    .into_iter()
                    .map(|h| FfiHook {
                        command: h.command,
                        is_async: h.r#async.unwrap_or(false),
                        timeout_secs: h.timeout,
                    })
                    .collect();
                result.push(FfiHookGroup {
                    event_name: event_name.clone(),
                    matcher: group.matcher,
                    hooks,
                });
            }
        }
        result.sort_by(|a, b| a.event_name.cmp(&b.event_name));
        result
    }

    /// MCP servers from claude_desktop_config.json / .mcp.json.
    pub fn get_mcp_servers(&self) -> Vec<FfiMcpServer> {
        let config = self.store.mcp_config();
        let mcp = match config {
            Some(c) => c,
            None => return vec![],
        };

        let mut servers: Vec<FfiMcpServer> = mcp
            .servers
            .into_iter()
            .map(|(name, s)| FfiMcpServer {
                name,
                server_type: s.server_type.unwrap_or_default(),
                command: s.command,
                args: s.args,
                url: s.url,
            })
            .collect();
        servers.sort_by(|a, b| a.name.cmp(&b.name));
        servers
    }

    /// FTS5 full-text search across all session content.
    pub fn search_sessions(&self, query: String, limit: u32) -> Vec<FfiSearchResult> {
        self.store
            .search_sessions(&query, limit as usize)
            .into_iter()
            .map(|r| FfiSearchResult {
                session_id: r.session_id,
                project_id: r.project.unwrap_or_default(),
                snippet: r.snippet.unwrap_or_default(),
                score: r.rank,
            })
            .collect()
    }

    /// Subscribe to DataStore events. Fires callbacks on the tokio thread pool —
    /// Swift caller is responsible for dispatching to the main actor.
    pub fn subscribe(&self, listener: Arc<dyn FfiEventListener>) {
        let mut rx = self.store.event_bus().subscribe();
        let count = self.store.session_count() as u64;
        listener.on_load_completed(count);

        runtime().spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    DataEvent::StatsUpdated | DataEvent::AnalyticsUpdated => {
                        listener.on_stats_updated();
                    }
                    DataEvent::SessionCreated(id) | DataEvent::SessionUpdated(id) => {
                        listener.on_session_changed(id.to_string());
                    }
                    DataEvent::ConfigChanged(_) => {
                        listener.on_config_changed();
                    }
                    _ => {}
                }
            }
        });
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Private helpers
// ──────────────────────────────────────────────────────────────────────────────

fn project_display_name(project_id: &str) -> String {
    // project_id is the encoded directory path, e.g.
    // "-Users-florianbruniaux-Sites-perso-ccboard"
    // or the raw path "/Users/florianbruniaux/Sites/perso/ccboard"
    let clean = project_id.trim_start_matches('-').trim_start_matches('/');
    clean
        .rsplit(|c| c == '/' || c == '-')
        .find(|s| !s.is_empty())
        .unwrap_or(project_id)
        .to_string()
}

fn session_to_ffi(s: &ccboard_core::models::session::SessionMetadata) -> FfiSessionSummary {
    FfiSessionSummary {
        id: s.id.to_string(),
        project_id: s.project_path.as_str().to_string(),
        first_timestamp: s.first_timestamp.map(|t| t.to_rfc3339()),
        last_timestamp: s.last_timestamp.map(|t| t.to_rfc3339()),
        message_count: s.message_count,
        total_tokens: s.total_tokens,
        input_tokens: s.input_tokens,
        output_tokens: s.output_tokens,
        cache_creation_tokens: s.cache_creation_tokens,
        cache_read_tokens: s.cache_read_tokens,
        models_used: s.models_used.clone(),
        model_segments: s
            .model_segments
            .iter()
            .map(|(model, count)| FfiModelSegment {
                model: model.clone(),
                message_count: *count as u64,
            })
            .collect(),
        first_user_message: s.first_user_message.clone(),
        has_subagents: s.has_subagents,
        parent_session_id: s.parent_session_id.clone(),
        duration_seconds: s.duration_seconds,
        branch: s.branch.clone(),
        tool_usage: s
            .tool_usage
            .iter()
            .map(|(name, count)| FfiToolCount {
                name: name.clone(),
                count: *count as u64,
            })
            .collect(),
        source_tool: format!("{:?}", s.source_tool).to_lowercase(),
        lines_added: s.lines_added,
        lines_removed: s.lines_removed,
    }
}
