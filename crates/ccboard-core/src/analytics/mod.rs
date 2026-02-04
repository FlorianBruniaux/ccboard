//! Advanced analytics module for Claude Code usage analysis
//!
//! Provides time series trends, forecasting, usage pattern detection,
//! and actionable insights to optimize costs and productivity.

use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::models::session::SessionMetadata;

pub mod forecasting;
pub mod insights;
pub mod patterns;
pub mod trends;

#[cfg(test)]
mod tests;

pub use forecasting::{ForecastData, TrendDirection, forecast_usage};
pub use insights::{Alert, generate_budget_alerts, generate_insights};
pub use patterns::{UsagePatterns, detect_patterns};
pub use trends::{TrendsData, SessionDurationStats, compute_trends};

/// Period selection for analytics computation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Period {
    /// Last N days from now
    Days(usize),
    /// All loaded sessions (honest: not "all time", limited by DataStore)
    Available,
}

impl Period {
    /// Last 7 days
    pub fn last_7d() -> Self {
        Self::Days(7)
    }

    /// Last 30 days
    pub fn last_30d() -> Self {
        Self::Days(30)
    }

    /// Last 90 days
    pub fn last_90d() -> Self {
        Self::Days(90)
    }

    /// All available sessions
    pub fn available() -> Self {
        Self::Available
    }

    /// Convert to days (for filtering)
    pub fn days(&self) -> usize {
        match self {
            Period::Days(n) => *n,
            Period::Available => 36500, // 100 years (effectively all)
        }
    }

    /// Display label (shows loaded count for Available)
    pub fn display(&self, total_loaded: usize) -> String {
        match self {
            Period::Days(n) => format!("Last {} days", n),
            Period::Available => format!("All loaded ({} sessions)", total_loaded),
        }
    }
}

/// Complete analytics data for a period
#[derive(Debug, Clone)]
pub struct AnalyticsData {
    /// Time series trends
    pub trends: TrendsData,
    /// Usage forecasting
    pub forecast: ForecastData,
    /// Behavioral patterns
    pub patterns: UsagePatterns,
    /// Actionable insights
    pub insights: Vec<String>,
    /// Timestamp of computation
    pub computed_at: DateTime<Utc>,
    /// Period analyzed
    pub period: Period,
}

impl AnalyticsData {
    /// Compute analytics from sessions (sync function)
    ///
    /// This is a sync function for simplicity. If computation exceeds 16ms
    /// (render deadline), caller should offload to `tokio::task::spawn_blocking`.
    ///
    /// # Performance
    /// Target: <100ms for 1000 sessions over 30 days
    pub fn compute(sessions: &[Arc<SessionMetadata>], period: Period) -> Self {
        let trends = compute_trends(sessions, period.days());
        let forecast = forecast_usage(&trends);
        let patterns = detect_patterns(sessions, period.days());
        let insights = generate_insights(&trends, &patterns, &forecast);

        Self {
            trends,
            forecast,
            patterns,
            insights,
            computed_at: Utc::now(),
            period,
        }
    }

    /// Graceful fallback if stats-cache.json missing
    ///
    /// Cost forecasting requires pricing data from StatsCache.
    /// If unavailable, returns limited analytics with warning.
    pub fn from_sessions_only(sessions: &[Arc<SessionMetadata>], period: Period) -> Self {
        tracing::warn!("Stats cache missing, computing analytics from sessions only");

        Self {
            trends: compute_trends(sessions, period.days()),
            forecast: ForecastData::unavailable("Stats cache required for cost forecasting"),
            patterns: detect_patterns(sessions, period.days()),
            insights: vec!["Limited insights: stats cache unavailable".to_string()],
            computed_at: Utc::now(),
            period,
        }
    }
}
