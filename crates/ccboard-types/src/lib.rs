//! ccboard-types - Shared data types for ccboard
//!
//! This crate contains pure data structures without heavy dependencies.
//! No tokio, no async runtime - just serde-serializable types.
//!
//! Used by:
//! - ccboard-core (backend logic)
//! - ccboard-web (frontend WASM)
//! - ccboard-tui (terminal UI)

pub mod analytics;
pub mod models;

// Re-export analytics types
pub use analytics::{
    Alert, AnalyticsData, Anomaly, AnomalyMetric, AnomalySeverity, ForecastData, Period,
    SessionDurationStats, TrendDirection, TrendsData, UsagePatterns,
};

// Re-export model types
pub use models::{
    BillingBlock, BillingBlockUsage, ContextWindowStats, DailyActivity, HookDefinition, HookGroup,
    InvocationStats, MergedConfig, ModelUsage, Permissions, SessionLine, SessionMessage,
    SessionMetadata, SessionSummary, Settings, StatsCache,
};
