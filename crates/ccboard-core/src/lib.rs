//! ccboard-core - Core library for ccboard
//!
//! Provides parsers, models, store, and file watcher for Claude Code data.

pub mod analytics;
pub mod cache;
pub mod error;
pub mod event;
pub mod export;
pub mod models;
pub mod parsers;
pub mod store;
pub mod watcher;

pub use analytics::{
    AnalyticsData, ForecastData, Period, TrendDirection, TrendsData, UsagePatterns,
    compute_trends, detect_patterns, forecast_usage, generate_insights,
};
pub use error::{CoreError, DegradedState, LoadReport};
pub use event::{DataEvent, EventBus};
pub use export::{export_billing_blocks_to_csv, export_sessions_to_csv, export_sessions_to_json};
pub use store::DataStore;
pub use watcher::FileWatcher;
