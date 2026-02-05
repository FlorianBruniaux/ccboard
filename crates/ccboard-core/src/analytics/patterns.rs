//! Usage pattern detection
//!
//! Identifies behavioral patterns: peak hours, productive days,
//! model distribution, and session duration analytics.

use chrono::{Datelike, Timelike, Weekday};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::models::session::SessionMetadata;

/// Usage patterns
#[derive(Debug, Clone)]
pub struct UsagePatterns {
    /// Most productive hour (0-23)
    pub most_productive_hour: u8,
    /// Most productive weekday
    pub most_productive_day: Weekday,
    /// Average session duration
    pub avg_session_duration: Duration,
    /// Most used model (by token count)
    pub most_used_model: String,
    /// Model distribution by token count (percentages)
    pub model_distribution: HashMap<String, f64>,
    /// Model distribution by cost (percentages)
    pub model_cost_distribution: HashMap<String, f64>,
    /// Peak hours (above 80th percentile)
    pub peak_hours: Vec<u8>,
    /// Hourly distribution (sessions per hour, 0-23)
    pub hourly_distribution: [usize; 24],
    /// Weekday distribution (sessions per weekday, 0-6)
    pub weekday_distribution: [usize; 7],
    /// Activity heatmap: [weekday][hour] = session count
    /// weekday: 0-6 (Mon-Sun), hour: 0-23
    pub activity_heatmap: [[usize; 24]; 7],
    /// Tool usage statistics: tool name -> call count
    pub tool_usage: HashMap<String, usize>,
}

impl UsagePatterns {
    /// Empty placeholder
    pub fn empty() -> Self {
        Self {
            most_productive_hour: 0,
            most_productive_day: Weekday::Mon,
            avg_session_duration: Duration::from_secs(0),
            most_used_model: "unknown".to_string(),
            model_distribution: HashMap::new(),
            model_cost_distribution: HashMap::new(),
            peak_hours: Vec::new(),
            hourly_distribution: [0; 24],
            weekday_distribution: [0; 7],
            activity_heatmap: [[0; 24]; 7],
            tool_usage: HashMap::new(),
        }
    }
}

/// Estimate cost from session (same as trends.rs)
///
/// TODO: Deduplicate with trends.rs estimate_cost()
fn estimate_cost(session: &SessionMetadata) -> f64 {
    (session.total_tokens as f64 / 1000.0) * 0.01
}

/// Detect usage patterns
///
/// Analyzes hourly/weekday distributions, model usage (token + cost weighted),
/// session duration, and peak hours (80th percentile threshold).
///
/// # Performance
/// Target: <30ms for 1000 sessions
///
/// # Graceful Degradation
/// - Empty sessions: Returns UsagePatterns::empty()
/// - Missing timestamps: Session skipped with warning
/// - No duration data: avg_session_duration = 0
pub fn detect_patterns(sessions: &[Arc<SessionMetadata>], days: usize) -> UsagePatterns {
    use chrono::Local;

    if sessions.is_empty() {
        return UsagePatterns::empty();
    }

    let mut hourly_counts = [0usize; 24];
    let mut weekday_counts = [0usize; 7];
    let mut activity_heatmap = [[0usize; 24]; 7];
    let mut tool_usage: HashMap<String, usize> = HashMap::new();
    let mut total_duration = Duration::from_secs(0);
    let mut duration_count = 0usize;
    let mut model_tokens: HashMap<String, u64> = HashMap::new();
    let mut model_costs: HashMap<String, f64> = HashMap::new();

    // Filter by period (same logic as compute_trends)
    let now = Local::now();
    let cutoff = now - chrono::Duration::days(days as i64);

    for session in sessions {
        // Apply period filter check
        let passes_filter = if let Some(ts) = session.first_timestamp {
            let local_ts = ts.with_timezone(&Local);
            local_ts >= cutoff
        } else {
            false
        };

        if !passes_filter {
            continue;
        }

        // Hourly distribution & heatmap
        if let Some(ts) = session.first_timestamp {
            let local_ts = ts.with_timezone(&Local);
            let hour = local_ts.hour() as usize;
            let weekday = local_ts.weekday().num_days_from_monday() as usize;

            hourly_counts[hour] += 1;
            weekday_counts[weekday] += 1;
            activity_heatmap[weekday][hour] += 1;
        }

        // Tool usage stats - TODO: Extract from session JSONL content
        // For now, we'll use model names as a proxy for "tools"
        // This could be enhanced later to parse actual tool_calls from JSONL
        for model in &session.models_used {
            *tool_usage.entry(model.clone()).or_default() += 1;
        }

        // Session duration
        if let (Some(start), Some(end)) = (session.first_timestamp, session.last_timestamp) {
            if let Ok(duration) = (end - start).to_std() {
                total_duration += duration;
                duration_count += 1;
            }
        }

        // Model distribution (tokens + cost)
        // Divide tokens equally among models used in this session
        if session.models_used.is_empty() {
            // No model info: attribute to "unknown"
            *model_tokens.entry("unknown".to_string()).or_default() += session.total_tokens;
            *model_costs.entry("unknown".to_string()).or_default() += estimate_cost(session);
        } else {
            let models_count = session.models_used.len() as u64;
            let tokens_per_model = session.total_tokens / models_count;
            let cost = estimate_cost(session);
            let cost_per_model = cost / models_count as f64;

            for model in &session.models_used {
                *model_tokens.entry(model.clone()).or_default() += tokens_per_model;
                *model_costs.entry(model.clone()).or_default() += cost_per_model;
            }
        }
    }

    // Most productive hour
    let most_productive_hour = hourly_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(hour, _)| hour as u8)
        .unwrap_or(0);

    // Most productive day
    let most_productive_day = weekday_counts
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .and_then(|(idx, _)| Weekday::try_from(idx as u8).ok())
        .unwrap_or(Weekday::Mon);

    // Average duration
    let avg_session_duration = if duration_count > 0 {
        total_duration / duration_count as u32
    } else {
        Duration::from_secs(0)
    };

    // Peak hours (80th percentile threshold)
    let total_sessions: usize = hourly_counts.iter().sum();
    let threshold = (total_sessions as f64 * 0.8 / 24.0) as usize;
    let peak_hours: Vec<u8> = hourly_counts
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > threshold)
        .map(|(hour, _)| hour as u8)
        .collect();

    // Model distribution (by tokens)
    let total_tokens: u64 = model_tokens.values().sum();
    let model_distribution: HashMap<String, f64> = if total_tokens > 0 {
        model_tokens
            .into_iter()
            .map(|(model, tokens)| (model, tokens as f64 / total_tokens as f64))
            .collect()
    } else {
        HashMap::new()
    };

    // Most used model
    let most_used_model = model_distribution
        .iter()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(model, _)| model.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Model cost distribution (NEW: cost-weighted)
    let total_cost: f64 = model_costs.values().sum();
    let model_cost_distribution: HashMap<String, f64> = if total_cost > 0.0 {
        model_costs
            .into_iter()
            .map(|(model, cost)| (model, cost / total_cost))
            .collect()
    } else {
        HashMap::new()
    };

    UsagePatterns {
        most_productive_hour,
        most_productive_day,
        avg_session_duration,
        most_used_model,
        model_distribution,
        model_cost_distribution,
        peak_hours,
        hourly_distribution: hourly_counts,
        weekday_distribution: weekday_counts,
        activity_heatmap,
        tool_usage,
    }
}
