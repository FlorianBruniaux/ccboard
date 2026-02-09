//! API client utilities and shared types for frontend

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// API base URL - points to Axum backend
/// In dev: Trunk (3333) → Axum (8080)
/// In prod: Same origin (integrated server)
#[cfg(debug_assertions)]
const API_BASE_URL: &str = "http://localhost:8080";

#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = "";

/// Stats data structure matching backend API response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsData {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub last_computed_date: Option<String>,
    #[serde(default)]
    pub total_sessions: u64,
    #[serde(default)]
    pub total_messages: u64,
    #[serde(default)]
    pub daily_activity: Vec<DailyActivityEntry>,
    #[serde(default)]
    pub model_usage: HashMap<String, ModelUsage>,
    // Analytics extension
    #[serde(default)]
    pub daily_tokens_30d: Vec<u64>,
    #[serde(default)]
    pub forecast_tokens_30d: Vec<u64>,
    #[serde(default)]
    pub forecast_confidence: f64,
    #[serde(default)]
    pub forecast_cost_30d: f64,
    #[serde(default)]
    pub projects_by_cost: Vec<ProjectCost>,
    #[serde(default)]
    pub most_used_model: Option<MostUsedModel>,
    #[serde(default)]
    pub this_month_cost: f64,
    #[serde(default)]
    pub avg_session_cost: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCost {
    pub project: String,
    #[serde(default)]
    pub cost: f64,
    #[serde(default)]
    pub percentage: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MostUsedModel {
    pub name: String,
    #[serde(default)]
    pub count: u64,
}

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
    pub cost_usd: f64,
}

impl ModelUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

impl StatsData {
    /// Calculate total tokens across all models
    pub fn total_tokens(&self) -> u64 {
        self.model_usage.values().map(|m| m.total_tokens()).sum()
    }

    /// Calculate total cost across all models
    pub fn total_cost(&self) -> f64 {
        self.model_usage.values().map(|m| m.cost_usd).sum()
    }

    /// Average cost per session
    pub fn avg_session_cost(&self) -> f64 {
        if self.total_sessions == 0 {
            return 0.0;
        }
        self.total_cost() / self.total_sessions as f64
    }

    /// Sessions count for current month (simplified for WASM)
    pub fn this_month_sessions(&self) -> u64 {
        // Get last 30 days of activity as a proxy for "this month"
        let len = self.daily_activity.len();
        let start = if len > 30 { len - 30 } else { 0 };

        self.daily_activity[start..]
            .iter()
            .map(|entry| entry.session_count)
            .sum()
    }

    /// Token count for current week (simplified for WASM)
    pub fn this_week_tokens(&self) -> u64 {
        // Get last 7 days of activity
        let len = self.daily_activity.len();
        let start = if len > 7 { len - 7 } else { 0 };

        self.daily_activity[start..]
            .iter()
            .map(|entry| {
                // Estimate tokens from message count (rough approximation)
                entry.message_count * 500
            })
            .sum()
    }

    /// Get last 30 days of token activity for sparkline
    pub fn daily_tokens_30d(&self) -> Vec<u64> {
        let mut result = Vec::new();
        let len = self.daily_activity.len();
        let start = if len > 30 { len - 30 } else { 0 };

        for entry in &self.daily_activity[start..] {
            // Estimate tokens from message count
            result.push(entry.message_count * 500);
        }

        // Pad with zeros if less than 30 days
        while result.len() < 30 {
            result.insert(0, 0);
        }

        result
    }
}

/// Session data structure (complete version)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionData {
    pub id: String,
    pub date: Option<String>,
    pub project: String,
    pub model: String,
    pub messages: u64,
    pub tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost: f64,
    pub status: String,
    pub first_timestamp: Option<String>,
    pub duration_seconds: Option<u64>,
    pub preview: Option<String>,
}

/// Recent sessions response from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentSessionsResponse {
    pub sessions: Vec<SessionData>,
    pub total: u64,
}

/// Fetch stats from API
pub async fn fetch_stats() -> Result<StatsData, String> {
    let url = format!("{}/api/stats", API_BASE_URL);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let stats = response
        .json::<StatsData>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(stats)
}

/// Fetch recent sessions from API (for dashboard)
pub async fn fetch_recent_sessions(limit: u32) -> Result<RecentSessionsResponse, String> {
    let url = format!("{}/api/sessions/recent?limit={}", API_BASE_URL, limit);
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.ok() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let sessions = response
        .json::<RecentSessionsResponse>()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(sessions)
}

/// Format large numbers (K, M, B)
pub fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Format cost as USD
pub fn format_cost(cost: f64) -> String {
    format!("${:.2}", cost)
}
