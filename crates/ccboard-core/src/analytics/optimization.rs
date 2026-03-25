//! Cost optimization suggestion engine
//!
//! Analyzes plugin analytics and tool token usage to generate actionable
//! suggestions for reducing Claude Code costs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::analytics::plugin_usage::PluginAnalytics;
use crate::models::session::SessionMetadata;

/// Category of a cost optimization suggestion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationCategory {
    /// A plugin/skill/command that is defined but never invoked
    UnusedPlugin,
    /// A tool consuming a disproportionate share of tokens
    HighCostTool,
    /// Opportunity to downgrade model for simpler tasks
    ModelDowngrade,
    /// Repeated identical tool calls that could be cached/batched
    RedundantCalls,
}

impl OptimizationCategory {
    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::UnusedPlugin => "Unused Plugin",
            Self::HighCostTool => "High-Cost Tool",
            Self::ModelDowngrade => "Model Downgrade",
            Self::RedundantCalls => "Redundant Calls",
        }
    }

    /// Icon for TUI/Web display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::UnusedPlugin => "🗑️",
            Self::HighCostTool => "🔥",
            Self::ModelDowngrade => "⬇️",
            Self::RedundantCalls => "♻️",
        }
    }
}

/// A single actionable cost optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSuggestion {
    /// Category for grouping
    pub category: OptimizationCategory,
    /// Short title (displayed in list)
    pub title: String,
    /// Full description with supporting data
    pub description: String,
    /// Estimated monthly savings in dollars (0 if unknown)
    pub potential_savings: f64,
    /// Concrete action the user can take
    pub action: String,
}

/// Generate cost optimization suggestions from available analytics data
///
/// # Arguments
/// - `plugin_analytics`: Aggregated plugin usage with cost attribution
/// - `tool_token_usage`: Aggregated per-tool token map across all sessions
/// - `total_monthly_cost`: Current MTD or 30-day cost in dollars
///
/// Returns suggestions sorted by potential savings descending.
pub fn generate_cost_suggestions(
    plugin_analytics: &PluginAnalytics,
    tool_token_usage: &HashMap<String, u64>,
    total_monthly_cost: f64,
) -> Vec<CostSuggestion> {
    let mut suggestions: Vec<CostSuggestion> = Vec::new();

    // 1. Dead plugins: defined but zero invocations
    for dead_name in &plugin_analytics.dead_plugins {
        suggestions.push(CostSuggestion {
            category: OptimizationCategory::UnusedPlugin,
            title: format!("Unused plugin: {}", dead_name),
            description: format!(
                "'{}' is defined in .claude/ but has never been invoked across all scanned sessions.",
                dead_name
            ),
            potential_savings: 0.0,
            action: format!(
                "Remove or archive '{}' to reduce cognitive noise in your setup.",
                dead_name
            ),
        });
    }

    // 2. High-cost tools: any single tool consuming >20% of total tokens
    if !tool_token_usage.is_empty() {
        let total_tokens: u64 = tool_token_usage.values().sum();
        if total_tokens > 0 {
            let mut sorted_tools: Vec<(&String, &u64)> = tool_token_usage.iter().collect();
            sorted_tools.sort_by(|a, b| b.1.cmp(a.1));

            for (tool, &tokens) in &sorted_tools {
                let pct = tokens as f64 / total_tokens as f64;
                if pct > 0.20 {
                    let estimated_cost = total_monthly_cost * pct;
                    let savings = estimated_cost * 0.30; // ~30% reduction potential
                    suggestions.push(CostSuggestion {
                        category: OptimizationCategory::HighCostTool,
                        title: format!("High-cost tool: {}", tool),
                        description: format!(
                            "'{}' consumes {:.1}% of your total token budget ({} tokens). \
                             This accounts for an estimated ${:.2}/month.",
                            tool,
                            pct * 100.0,
                            tokens,
                            estimated_cost
                        ),
                        potential_savings: savings,
                        action: format!(
                            "Review usage of '{}' — consider batching calls, caching results, \
                             or reducing call frequency.",
                            tool
                        ),
                    });
                }
            }
        }
    }

    // 3. Sort by potential savings descending, then by category stability
    suggestions.sort_by(|a, b| {
        b.potential_savings
            .partial_cmp(&a.potential_savings)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    suggestions
}

/// Generate model downgrade recommendations based on session history.
///
/// If >60% of sessions use an Opus model AND average tool calls per session < 8,
/// suggests switching to Sonnet for routine work. Estimates monthly savings.
///
/// # Returns
/// Vec of `CostSuggestion` with category `ModelDowngrade`, sorted by savings desc.
pub fn generate_model_recommendations(
    sessions: &[Arc<SessionMetadata>],
    total_monthly_cost: f64,
) -> Vec<CostSuggestion> {
    const MIN_SESSIONS: usize = 5;
    const OPUS_THRESHOLD: f64 = 0.60;
    const LOW_TOOL_CALLS: usize = 8;

    if sessions.len() < MIN_SESSIONS || total_monthly_cost < 0.50 {
        return vec![];
    }

    let mut opus_sessions = 0usize;
    let mut total_tool_calls_opus = 0usize;

    for session in sessions {
        let uses_opus = session
            .models_used
            .iter()
            .any(|m| m.to_lowercase().contains("opus"));
        if uses_opus {
            opus_sessions += 1;
            total_tool_calls_opus += session.tool_usage.values().sum::<usize>();
        }
    }

    if opus_sessions == 0 {
        return vec![];
    }

    let opus_pct = opus_sessions as f64 / sessions.len() as f64;
    let avg_tool_calls = total_tool_calls_opus as f64 / opus_sessions as f64;

    if opus_pct > OPUS_THRESHOLD && avg_tool_calls < LOW_TOOL_CALLS as f64 {
        // Opus is ~5x more expensive than Sonnet; estimate 70% savings on Opus sessions
        let savings = total_monthly_cost * opus_pct * 0.70;
        vec![CostSuggestion {
            category: OptimizationCategory::ModelDowngrade,
            title: format!("Switch {:.0}% of Opus sessions to Sonnet", opus_pct * 100.0),
            description: format!(
                "{:.0}% of sessions use Opus with only ~{:.0} avg tool calls — \
                 Sonnet handles most coding tasks at ~5× lower cost.",
                opus_pct * 100.0,
                avg_tool_calls
            ),
            potential_savings: savings,
            action: "Use claude-sonnet-4-6 for routine tasks, reserve Opus 4.6 for complex \
                     multi-step reasoning."
                .to_string(),
        }]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analytics::plugin_usage::PluginAnalytics;
    use crate::models::session::SessionMetadata;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn empty_analytics() -> PluginAnalytics {
        PluginAnalytics::empty()
    }

    #[test]
    fn test_no_suggestions_empty_data() {
        let analytics = empty_analytics();
        let tool_tokens: HashMap<String, u64> = HashMap::new();
        let suggestions = generate_cost_suggestions(&analytics, &tool_tokens, 0.0);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_unused_plugin_suggestion() {
        let mut analytics = empty_analytics();
        analytics.dead_plugins = vec!["unused-skill".to_string(), "old-command".to_string()];

        let tool_tokens: HashMap<String, u64> = HashMap::new();
        let suggestions = generate_cost_suggestions(&analytics, &tool_tokens, 10.0);

        let unused: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == OptimizationCategory::UnusedPlugin)
            .collect();
        assert_eq!(unused.len(), 2);
    }

    #[test]
    fn test_high_cost_tool_threshold() {
        let analytics = empty_analytics();
        let mut tool_tokens = HashMap::new();
        // Bash uses 50% of tokens — should trigger high-cost suggestion
        tool_tokens.insert("Bash".to_string(), 500u64);
        tool_tokens.insert("Read".to_string(), 500u64);

        let suggestions = generate_cost_suggestions(&analytics, &tool_tokens, 20.0);

        let high_cost: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == OptimizationCategory::HighCostTool)
            .collect();
        // Both tools at 50% each should trigger suggestions
        assert_eq!(high_cost.len(), 2);
        // Potential savings should be non-zero
        assert!(high_cost.iter().all(|s| s.potential_savings > 0.0));
    }

    #[test]
    fn test_below_threshold_no_high_cost() {
        let analytics = empty_analytics();
        let mut tool_tokens = HashMap::new();
        // 5 tools at 20% each — none exceed the 20% threshold (exclusive >)
        for i in 0..5 {
            tool_tokens.insert(format!("Tool{}", i), 200u64);
        }

        let suggestions = generate_cost_suggestions(&analytics, &tool_tokens, 10.0);
        let high_cost: Vec<_> = suggestions
            .iter()
            .filter(|s| s.category == OptimizationCategory::HighCostTool)
            .collect();
        assert!(high_cost.is_empty(), "20% exactly is not above threshold");
    }

    #[test]
    fn test_sorted_by_savings() {
        let analytics = empty_analytics();
        let mut tool_tokens = HashMap::new();
        // One dominant tool at 80%
        tool_tokens.insert("Bash".to_string(), 800u64);
        tool_tokens.insert("Read".to_string(), 200u64);

        let suggestions = generate_cost_suggestions(&analytics, &tool_tokens, 100.0);

        if suggestions.len() >= 2 {
            assert!(
                suggestions[0].potential_savings >= suggestions[1].potential_savings,
                "Should be sorted by potential savings descending"
            );
        }
    }

    // --- generate_model_recommendations tests ---

    fn make_session(id: &str, model: &str, tool_calls: usize) -> Arc<SessionMetadata> {
        let mut tool_usage = HashMap::new();
        if tool_calls > 0 {
            tool_usage.insert("Bash".to_string(), tool_calls);
        }
        Arc::new(SessionMetadata {
            id: id.into(),
            source_tool: None,
            file_path: std::path::PathBuf::from(format!("/tmp/{}.jsonl", id)),
            project_path: "test".into(),
            first_timestamp: Some(chrono::Utc::now()),
            last_timestamp: Some(chrono::Utc::now()),
            message_count: 5,
            total_tokens: 10_000,
            input_tokens: 5_000,
            output_tokens: 5_000,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec![model.to_string()],
            file_size_bytes: 1024,
            first_user_message: None,
            has_subagents: false,
            duration_seconds: Some(60),
            branch: None,
            tool_usage,
            tool_token_usage: HashMap::new(),
        })
    }

    #[test]
    fn test_model_recommendations_too_few_sessions() {
        // Below MIN_SESSIONS=5 — should return empty
        let sessions: Vec<_> = (0..4)
            .map(|i| make_session(&format!("s{}", i), "claude-opus-4-6", 3))
            .collect();
        let recs = generate_model_recommendations(&sessions, 50.0);
        assert!(recs.is_empty(), "Need at least 5 sessions");
    }

    #[test]
    fn test_model_recommendations_low_cost() {
        // Below $0.50/month — should return empty
        let sessions: Vec<_> = (0..10)
            .map(|i| make_session(&format!("s{}", i), "claude-opus-4-6", 3))
            .collect();
        let recs = generate_model_recommendations(&sessions, 0.40);
        assert!(recs.is_empty(), "Cost too low to recommend downgrade");
    }

    #[test]
    fn test_model_recommendations_sonnet_heavy() {
        // Mostly Sonnet — no downgrade recommendation
        let mut sessions: Vec<_> = (0..8)
            .map(|i| make_session(&format!("sonnet_{}", i), "claude-sonnet-4-6", 5))
            .collect();
        sessions.extend((0..2).map(|i| make_session(&format!("opus_{}", i), "claude-opus-4-6", 3)));
        let recs = generate_model_recommendations(&sessions, 50.0);
        assert!(recs.is_empty(), "Mostly Sonnet — no downgrade needed");
    }

    #[test]
    fn test_model_recommendations_opus_heavy_low_tools() {
        // >60% Opus with <8 avg tool calls — should recommend downgrade
        let sessions: Vec<_> = (0..10)
            .map(|i| make_session(&format!("s{}", i), "claude-opus-4-6", 4)) // 4 tool calls avg
            .collect();
        let recs = generate_model_recommendations(&sessions, 100.0);
        assert!(
            !recs.is_empty(),
            "Should recommend Sonnet for low-tool Opus sessions"
        );
        assert_eq!(recs[0].category, OptimizationCategory::ModelDowngrade);
        assert!(recs[0].potential_savings > 0.0);
    }

    #[test]
    fn test_model_recommendations_opus_heavy_high_tools() {
        // >60% Opus but with many tool calls — no recommendation (complex tasks)
        let sessions: Vec<_> = (0..10)
            .map(|i| make_session(&format!("s{}", i), "claude-opus-4-6", 12)) // 12 tool calls avg
            .collect();
        let recs = generate_model_recommendations(&sessions, 100.0);
        assert!(recs.is_empty(), "High tool calls — Opus justified");
    }
}
