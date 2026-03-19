//! ccboard-core - Core library for ccboard
//!
//! Provides parsers, models, store, and file watcher for Claude Code data.

pub mod analytics;
pub mod cache;
pub mod error;
pub mod event;
pub mod export;
pub mod graph;
pub mod hook_event;
pub mod hook_state;
pub mod live_monitor;
pub mod models;
pub mod parsers;
pub mod preferences;
pub mod pricing;
pub mod quota;
pub mod store;
pub mod usage_estimator;
pub mod watcher;

pub use analytics::{
    compute_trends, detect_patterns, discover_call_llm, discover_collect_sessions,
    discover_patterns, forecast_usage, generate_insights, run_discover, AnalyticsData,
    DiscoverConfig, DiscoverSessionData, DiscoverSuggestion, ForecastData, LlmSuggestion, Period,
    SuggestionCategory, TrendDirection, TrendsData, UsagePatterns,
};
pub use cache::{AggregateStats, SearchResult, StoredAlert};
pub use error::{CoreError, DegradedState, LoadReport};
pub use event::{DataEvent, EventBus};
pub use export::{
    export_billing_blocks_to_csv, export_billing_blocks_to_json, export_billing_blocks_to_markdown,
    export_sessions_to_csv, export_sessions_to_json, export_sessions_to_markdown,
    export_stats_to_csv, export_stats_to_json, export_stats_to_markdown,
};
pub use hook_event::{status_from_event, HookPayload};
pub use hook_state::{
    make_session_key, HookSession, HookSessionStatus, LiveSessionFile, SessionKey,
};
pub use live_monitor::{
    detect_live_sessions, merge_live_sessions, LiveSession, LiveSessionDisplayStatus,
    MergedLiveSession, SessionType,
};
pub use models::activity::{
    ActivitySummary, Alert, AlertCategory, AlertSeverity, BashCommand, FileAccess, FileOperation,
    NetworkCall, NetworkTool, ToolCall as ActivityToolCall,
};
pub use quota::{calculate_quota_status, AlertLevel, QuotaStatus};
pub use store::{DataStore, ProjectLeaderboardEntry};
pub use usage_estimator::{calculate_usage_estimate, SubscriptionPlan, UsageEstimate};
pub use watcher::FileWatcher;
