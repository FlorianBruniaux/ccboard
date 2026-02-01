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
}
