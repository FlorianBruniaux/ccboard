//! Stats cache model from ~/.claude/stats-cache.json
//!
//! Note: The actual Claude Code stats-cache.json format differs from initial assumptions.
//! Key fields: dailyActivity (array), dailyModelTokens (array), modelUsage (object),
//! totalSessions, totalMessages, hourCounts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level stats cache structure matching actual Claude Code format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsCache {
    /// Version of the stats format
    #[serde(default)]
    pub version: u32,

    /// Last computed date (YYYY-MM-DD)
    #[serde(default)]
    pub last_computed_date: Option<String>,

    /// Daily activity entries
    #[serde(default)]
    pub daily_activity: Vec<DailyActivityEntry>,

    /// Daily model token usage
    #[serde(default)]
    pub daily_model_tokens: Vec<DailyModelTokens>,

    /// Model usage breakdown
    #[serde(default)]
    pub model_usage: HashMap<String, ModelUsage>,

    /// Total sessions
    #[serde(default)]
    pub total_sessions: u64,

    /// Total messages
    #[serde(default)]
    pub total_messages: u64,

    /// Longest session info
    #[serde(default)]
    pub longest_session: Option<LongestSession>,

    /// First session date
    #[serde(default)]
    pub first_session_date: Option<String>,

    /// Hour counts for heatmap (0-23 as strings)
    #[serde(default)]
    pub hour_counts: HashMap<String, u64>,

    /// Total speculation time saved in ms
    #[serde(default)]
    pub total_speculation_time_saved_ms: u64,
}

/// Daily activity entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivityEntry {
    pub date: String,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub session_count: u64,
    #[serde(default)]
    pub tool_call_count: u64,
}

/// Daily model tokens entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyModelTokens {
    pub date: String,
    #[serde(default)]
    pub tokens_by_model: HashMap<String, u64>,
}

/// Per-model usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub web_search_requests: u64,
    #[serde(default)]
    pub cost_usd: f64,
    #[serde(default)]
    pub context_window: u64,
    #[serde(default)]
    pub max_output_tokens: u64,
}

impl ModelUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }

    pub fn total_with_cache(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_input_tokens
            + self.cache_creation_input_tokens
    }
}

/// Longest session info
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LongestSession {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub date: Option<String>,
}

/// Legacy daily activity format for compatibility
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyActivity {
    #[serde(default)]
    pub tokens: u64,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub messages: u64,
    #[serde(default)]
    pub sessions: u64,
}

impl StatsCache {
    /// Calculate total input tokens across all models
    pub fn total_input_tokens(&self) -> u64 {
        self.model_usage.values().map(|m| m.input_tokens).sum()
    }

    /// Calculate total output tokens across all models
    pub fn total_output_tokens(&self) -> u64 {
        self.model_usage.values().map(|m| m.output_tokens).sum()
    }

    /// Calculate total tokens (input + output)
    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens() + self.total_output_tokens()
    }

    /// Calculate total cache read tokens
    pub fn total_cache_read_tokens(&self) -> u64 {
        self.model_usage
            .values()
            .map(|m| m.cache_read_input_tokens)
            .sum()
    }

    /// Calculate total cache write tokens
    pub fn total_cache_write_tokens(&self) -> u64 {
        self.model_usage
            .values()
            .map(|m| m.cache_creation_input_tokens)
            .sum()
    }

    /// Recalculate costs for all models using accurate pricing
    ///
    /// This should be called after loading stats from stats-cache.json to ensure
    /// cost_usd fields are populated with accurate pricing data.
    pub fn recalculate_costs(&mut self) {
        for (model_name, usage) in self.model_usage.iter_mut() {
            usage.cost_usd = crate::pricing::calculate_cost(
                model_name,
                usage.input_tokens,
                usage.output_tokens,
                usage.cache_creation_input_tokens,
                usage.cache_read_input_tokens,
            );
        }
    }

    /// Get session count
    pub fn session_count(&self) -> u64 {
        self.total_sessions
    }

    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.total_messages
    }

    /// Get top N models by token usage
    pub fn top_models(&self, n: usize) -> Vec<(&str, &ModelUsage)> {
        let mut models: Vec<_> = self
            .model_usage
            .iter()
            .filter(|(_, usage)| usage.total_tokens() > 0)
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        models.sort_by(|a, b| b.1.total_tokens().cmp(&a.1.total_tokens()));
        models.truncate(n);
        models
    }

    /// Get recent N days of activity
    pub fn recent_daily(&self, n: usize) -> Vec<&DailyActivityEntry> {
        let len = self.daily_activity.len();
        if len <= n {
            self.daily_activity.iter().collect()
        } else {
            self.daily_activity[len - n..].iter().collect()
        }
    }

    /// Calculate cache hit ratio
    pub fn cache_ratio(&self) -> f64 {
        let cache_read = self.total_cache_read_tokens();
        let total_input = self.total_input_tokens() + cache_read;
        if total_input == 0 {
            return 0.0;
        }
        cache_read as f64 / total_input as f64
    }

    /// Context window size for Sonnet 4.5 (200K tokens)
    pub const CONTEXT_WINDOW: u64 = 200_000;

    /// Calculate context window saturation from session metadata
    ///
    /// NOTE: Requires session metadata to be passed from DataStore
    /// since StatsCache doesn't have direct access to sessions.
    pub fn calculate_context_saturation(
        session_metadata: &[&crate::models::SessionMetadata],
        last_n: usize,
    ) -> ContextWindowStats {
        if session_metadata.is_empty() {
            return ContextWindowStats::default();
        }

        // Sort by last_timestamp descending (most recent first)
        let mut sorted: Vec<_> = session_metadata
            .iter()
            .filter(|s| s.last_timestamp.is_some() && s.total_tokens > 0)
            .collect();
        sorted.sort_by(|a, b| b.last_timestamp.cmp(&a.last_timestamp));

        // Take last N sessions
        let recent: Vec<_> = sorted.into_iter().take(last_n).collect();

        if recent.is_empty() {
            return ContextWindowStats::default();
        }

        // Calculate saturation percentages
        let mut total_pct = 0.0;
        let mut high_load_count = 0;
        let mut peak_pct = 0.0;

        for session in &recent {
            let saturation_pct =
                (session.total_tokens as f64 / Self::CONTEXT_WINDOW as f64) * 100.0;
            total_pct += saturation_pct;

            if saturation_pct > 85.0 {
                high_load_count += 1;
            }

            if saturation_pct > peak_pct {
                peak_pct = saturation_pct;
            }
        }

        ContextWindowStats {
            avg_saturation_pct: total_pct / recent.len() as f64,
            high_load_count,
            peak_saturation_pct: peak_pct,
        }
    }
}

/// Context window saturation statistics
#[derive(Debug, Clone, Default)]
pub struct ContextWindowStats {
    /// Average saturation percentage across last N sessions (0.0-100.0)
    pub avg_saturation_pct: f64,

    /// Count of sessions exceeding 85% saturation (high-load)
    pub high_load_count: usize,

    /// Peak saturation percentage (max session, for future use)
    pub peak_saturation_pct: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_cache_defaults() {
        let stats = StatsCache::default();
        assert_eq!(stats.total_tokens(), 0);
        assert!(stats.model_usage.is_empty());
    }

    #[test]
    fn test_model_usage_total() {
        let usage = ModelUsage {
            input_tokens: 1000,
            output_tokens: 500,
            ..Default::default()
        };
        assert_eq!(usage.total_tokens(), 1500);
    }

    #[test]
    fn test_cache_ratio() {
        let mut stats = StatsCache::default();
        stats.model_usage.insert(
            "test".to_string(),
            ModelUsage {
                input_tokens: 800,
                cache_read_input_tokens: 200,
                ..Default::default()
            },
        );
        assert!((stats.cache_ratio() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_top_models() {
        let mut stats = StatsCache::default();
        stats.model_usage.insert(
            "opus".to_string(),
            ModelUsage {
                input_tokens: 1000,
                output_tokens: 500,
                ..Default::default()
            },
        );
        stats.model_usage.insert(
            "sonnet".to_string(),
            ModelUsage {
                input_tokens: 2000,
                output_tokens: 1000,
                ..Default::default()
            },
        );

        let top = stats.top_models(2);
        assert_eq!(top[0].0, "sonnet");
        assert_eq!(top[1].0, "opus");
    }

    #[test]
    fn test_parse_real_format() {
        let json = r#"{
            "version": 2,
            "lastComputedDate": "2026-01-31",
            "dailyActivity": [
                {"date": "2026-01-30", "messageCount": 100, "sessionCount": 5, "toolCallCount": 20}
            ],
            "modelUsage": {
                "claude-opus-4-5": {
                    "inputTokens": 1000,
                    "outputTokens": 500,
                    "cacheReadInputTokens": 200,
                    "cacheCreationInputTokens": 100
                }
            },
            "totalSessions": 10,
            "totalMessages": 1000,
            "hourCounts": {"10": 50, "14": 100}
        }"#;

        let stats: StatsCache = serde_json::from_str(json).unwrap();
        assert_eq!(stats.version, 2);
        assert_eq!(stats.total_sessions, 10);
        assert_eq!(stats.total_messages, 1000);
        assert_eq!(stats.daily_activity.len(), 1);
        assert_eq!(stats.total_input_tokens(), 1000);
        assert_eq!(stats.total_output_tokens(), 500);
    }

    #[test]
    fn test_context_saturation_calculation() {
        use crate::models::SessionMetadata;
        use chrono::Utc;
        use std::path::PathBuf;

        let mut sessions = vec![];
        let now = Utc::now();

        // Create 5 test sessions with varying token counts
        for (i, tokens) in [50_000u64, 100_000, 150_000, 170_000, 190_000]
            .iter()
            .enumerate()
        {
            let mut meta = SessionMetadata::from_path(
                PathBuf::from(format!("/test{}.jsonl", i)),
                "test".to_string(),
            );
            meta.total_tokens = *tokens;
            meta.last_timestamp = Some(now - chrono::Duration::seconds((4 - i) as i64 * 60));
            sessions.push(meta);
        }

        let refs: Vec<_> = sessions.iter().collect();
        let stats = StatsCache::calculate_context_saturation(&refs, 30);

        // Average: (25% + 50% + 75% + 85% + 95%) / 5 = 66%
        assert!((stats.avg_saturation_pct - 66.0).abs() < 1.0);

        // High-load count (>85%): 1 session (190K tokens = 95%)
        assert_eq!(stats.high_load_count, 1);

        // Peak saturation: 95%
        assert!((stats.peak_saturation_pct - 95.0).abs() < 1.0);
    }

    #[test]
    fn test_context_saturation_empty_sessions() {
        let stats = StatsCache::calculate_context_saturation(&[], 30);
        assert_eq!(stats.avg_saturation_pct, 0.0);
        assert_eq!(stats.high_load_count, 0);
    }

    #[test]
    fn test_context_saturation_fewer_than_requested() {
        use crate::models::SessionMetadata;
        use chrono::Utc;
        use std::path::PathBuf;

        let mut sessions = vec![];
        let now = Utc::now();

        // Only 3 sessions, requesting last 30
        for (i, tokens) in [60_000u64, 80_000, 120_000].iter().enumerate() {
            let mut meta = SessionMetadata::from_path(
                PathBuf::from(format!("/test{}.jsonl", i)),
                "test".to_string(),
            );
            meta.total_tokens = *tokens;
            meta.last_timestamp = Some(now - chrono::Duration::seconds((2 - i) as i64 * 60));
            sessions.push(meta);
        }

        let refs: Vec<_> = sessions.iter().collect();
        let stats = StatsCache::calculate_context_saturation(&refs, 30);

        // Should calculate average of available 3 sessions
        // (30% + 40% + 60%) / 3 = 43.33%
        assert!((stats.avg_saturation_pct - 43.33).abs() < 0.1);
    }
}
