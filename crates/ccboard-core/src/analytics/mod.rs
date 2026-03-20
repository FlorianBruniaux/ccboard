//! Advanced analytics module for Claude Code usage analysis
//!
//! Provides time series trends, forecasting, usage pattern detection,
//! and actionable insights to optimize costs and productivity.

use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::models::session::SessionMetadata;

pub mod anomalies;
pub mod discover;
pub mod discover_llm;
pub mod forecasting;
pub mod insights;
pub mod optimization;
pub mod patterns;
pub mod plugin_usage;
pub mod tool_chains;
pub mod trends;

#[cfg(test)]
mod tests;

pub use anomalies::{
    detect_anomalies, detect_daily_cost_spikes, Anomaly, AnomalyMetric, AnomalySeverity,
    DailyCostAnomaly,
};
pub use discover::{
    collect_sessions_data as discover_collect_sessions, discover_patterns, run_discover,
    DiscoverConfig, DiscoverSuggestion, SessionData as DiscoverSessionData, SuggestionCategory,
};
pub use discover_llm::{call_claude_cli as discover_call_llm, LlmSuggestion};
pub use forecasting::{forecast_usage, ForecastData, TrendDirection};
pub use insights::{generate_budget_alerts, generate_insights, Alert};
pub use optimization::{
    generate_cost_suggestions, generate_model_recommendations, CostSuggestion, OptimizationCategory,
};
pub use patterns::{detect_patterns, UsagePatterns};
pub use plugin_usage::{aggregate_plugin_usage, PluginAnalytics, PluginType, PluginUsage};
pub use tool_chains::{analyze_tool_chains, ToolChain, ToolChainAnalysis};
pub use trends::{compute_trends, SessionDurationStats, TrendsData};

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
    /// Tool chain bigram/trigram analysis
    pub tool_chains: Option<ToolChainAnalysis>,
    /// Cost optimization suggestions
    pub cost_suggestions: Vec<optimization::CostSuggestion>,
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

        // Aggregate per-tool token usage across all sessions
        let mut aggregated_tool_tokens: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        for session in sessions {
            for (tool, &tokens) in &session.tool_token_usage {
                *aggregated_tool_tokens.entry(tool.clone()).or_default() += tokens;
            }
        }

        // Estimate period cost from trend data
        let total_cost_estimate: f64 = trends.daily_cost.iter().sum();

        // Generate cost suggestions (plugin_analytics populated with empty data here;
        // full plugin analytics with dead-code detection requires skill/command lists
        // which are provided by DataStore when calling the analytics tab)
        let mut cost_suggestions = optimization::generate_cost_suggestions(
            &plugin_usage::PluginAnalytics::empty(),
            &aggregated_tool_tokens,
            total_cost_estimate,
        );

        // Append model downgrade recommendations
        let model_recs =
            optimization::generate_model_recommendations(sessions, total_cost_estimate);
        cost_suggestions.extend(model_recs);
        // Re-sort by potential savings descending after merge
        cost_suggestions.sort_by(|a, b| {
            b.potential_savings
                .partial_cmp(&a.potential_savings)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Self {
            trends,
            forecast,
            patterns,
            insights,
            tool_chains: Some(analyze_tool_chains(sessions)),
            cost_suggestions,
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
            tool_chains: Some(analyze_tool_chains(sessions)),
            cost_suggestions: Vec::new(),
            computed_at: Utc::now(),
            period,
        }
    }
}
