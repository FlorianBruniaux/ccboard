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
    AnalyticsData, Anomaly, AnomalyMetric, AnomalySeverity, ForecastData, Period, TrendDirection,
    TrendsData, UsagePatterns, SessionDurationStats, Alert,
};

// Re-export model types
pub use models::{
    BillingBlock, BillingBlockUsage, HookDefinition, HookGroup, MergedConfig, Permissions,
    Settings, InvocationStats, SessionLine, SessionMessage, SessionMetadata, SessionSummary,
    ContextWindowStats, DailyActivity, ModelUsage, StatsCache,
};
