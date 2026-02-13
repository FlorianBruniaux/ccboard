//! ccboard-core - Core library for ccboard
//!
//! Provides parsers, models, store, and file watcher for Claude Code data.

pub mod analytics;
pub mod cache;
pub mod error;
pub mod event;
pub mod export;
pub mod graph;
pub mod live_monitor;
pub mod models;
pub mod parsers;
pub mod pricing;
pub mod store;
pub mod usage_estimator;
pub mod watcher;

pub use analytics::{
    compute_trends, detect_patterns, forecast_usage, generate_insights, AnalyticsData,
    ForecastData, Period, TrendDirection, TrendsData, UsagePatterns,
};
pub use error::{CoreError, DegradedState, LoadReport};
pub use event::{DataEvent, EventBus};
pub use export::{export_billing_blocks_to_csv, export_sessions_to_csv, export_sessions_to_json};
pub use live_monitor::{detect_live_sessions, LiveSession};
pub use store::{DataStore, ProjectLeaderboardEntry};
pub use usage_estimator::{calculate_usage_estimate, SubscriptionPlan, UsageEstimate};
pub use watcher::FileWatcher;
