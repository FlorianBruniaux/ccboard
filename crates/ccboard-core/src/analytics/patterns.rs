//! Usage pattern detection
//!
//! Identifies behavioral patterns: peak hours, productive days,
//! model distribution, and session duration analytics.

use chrono::{Datelike, NaiveDate, Timelike, Weekday};
use std::collections::{BTreeSet, HashMap};
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
    /// Current consecutive-day usage streak (days ending today or yesterday)
    pub current_streak_days: u32,
    /// Longest consecutive-day streak across all loaded sessions
    pub longest_streak_days: u32,
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
            current_streak_days: 0,
            longest_streak_days: 0,
        }
    }
}

/// Estimate cost from session (same as trends.rs)
fn estimate_cost(session: &SessionMetadata) -> f64 {
    (session.total_tokens as f64 / 1000.0) * 0.01
}

/// Compute current and longest consecutive-day streaks across all sessions.
///
/// "Current streak" counts backward from today (or yesterday if no session today).
/// "Longest streak" scans all active days.
fn compute_streaks(sessions: &[Arc<SessionMetadata>]) -> (u32, u32) {
    use chrono::Local;

    let mut active_days: BTreeSet<NaiveDate> = BTreeSet::new();
    for session in sessions {
        if let Some(ts) = session.first_timestamp {
            active_days.insert(ts.with_timezone(&Local).date_naive());
        }
    }

    if active_days.is_empty() {
        return (0, 0);
    }

    // Current streak: walk backward from today (or yesterday if no session today)
    let today = Local::now().date_naive();
    let yesterday = today - chrono::Duration::days(1);
    let start = if active_days.contains(&today) {
        today
    } else if active_days.contains(&yesterday) {
        yesterday
    } else {
        // No activity today or yesterday — streak is 0
        let longest = {
            let days_vec: Vec<NaiveDate> = active_days.into_iter().collect();
            let mut longest = 0u32;
            let mut streak = 0u32;
            let mut prev: Option<NaiveDate> = None;
            for day in &days_vec {
                if let Some(p) = prev {
                    if *day == p + chrono::Duration::days(1) {
                        streak += 1;
                    } else {
                        streak = 1;
                    }
                } else {
                    streak = 1;
                }
                longest = longest.max(streak);
                prev = Some(*day);
            }
            longest
        };
        return (0, longest);
    };
    let mut current = 0u32;
    let mut check = start;
    loop {
        if active_days.contains(&check) {
            current += 1;
            check -= chrono::Duration::days(1);
        } else {
            break;
        }
    }

    // Longest streak: scan sorted days
    let days_vec: Vec<NaiveDate> = active_days.into_iter().collect();
    let mut longest = 0u32;
    let mut streak = 0u32;
    let mut prev: Option<NaiveDate> = None;
    for day in &days_vec {
        if let Some(p) = prev {
            if *day == p + chrono::Duration::days(1) {
                streak += 1;
            } else {
                streak = 1;
            }
        } else {
            streak = 1;
        }
        longest = longest.max(streak);
        prev = Some(*day);
    }
    // current streak cannot exceed longest
    let current = current.min(longest);

    (current, longest)
}

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
    let mut model_tokens: HashMap<String, f64> = HashMap::new();
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

        // Tool usage stats - extracted from session metadata
        for (tool_name, count) in &session.tool_usage {
            *tool_usage.entry(tool_name.clone()).or_default() += count;
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
            *model_tokens.entry("unknown".to_string()).or_default() += session.total_tokens as f64;
            *model_costs.entry("unknown".to_string()).or_default() += estimate_cost(session);
        } else {
            let models_count = session.models_used.len() as f64;
            let tokens_per_model = session.total_tokens as f64 / models_count;
            let cost = estimate_cost(session);
            let cost_per_model = cost / models_count;

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
    let total_tokens: f64 = model_tokens.values().sum();
    let model_distribution: HashMap<String, f64> = if total_tokens > 0.0 {
        model_tokens
            .into_iter()
            .map(|(model, tokens)| (model, tokens / total_tokens))
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

    let (current_streak_days, longest_streak_days) = compute_streaks(sessions);

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
        current_streak_days,
        longest_streak_days,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_on_days_ago(days_ago: i64) -> Arc<SessionMetadata> {
        let ts = chrono::Utc::now() - chrono::Duration::days(days_ago);
        Arc::new(SessionMetadata {
            id: format!("s{}", days_ago).into(),
            file_path: std::path::PathBuf::from(format!("/tmp/s{}.jsonl", days_ago)),
            project_path: "test".into(),
            first_timestamp: Some(ts),
            last_timestamp: Some(ts),
            message_count: 1,
            total_tokens: 100,
            input_tokens: 50,
            output_tokens: 50,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec![],
            file_size_bytes: 256,
            first_user_message: None,
            has_subagents: false,
            duration_seconds: Some(10),
            branch: None,
            tool_usage: std::collections::HashMap::new(),
            tool_token_usage: std::collections::HashMap::new(),
        })
    }

    #[test]
    fn test_streak_empty_sessions() {
        let (current, longest) = compute_streaks(&[]);
        assert_eq!(current, 0);
        assert_eq!(longest, 0);
    }

    #[test]
    fn test_streak_single_today() {
        let sessions = vec![session_on_days_ago(0)];
        let (current, longest) = compute_streaks(&sessions);
        assert_eq!(current, 1);
        assert_eq!(longest, 1);
    }

    #[test]
    fn test_streak_yesterday_only() {
        // No session today, but one yesterday — current streak should be 1
        let sessions = vec![session_on_days_ago(1)];
        let (current, longest) = compute_streaks(&sessions);
        assert_eq!(current, 1);
        assert_eq!(longest, 1);
    }

    #[test]
    fn test_streak_gap_breaks_current() {
        // Sessions 2+ days ago only — current streak is 0
        let sessions = vec![session_on_days_ago(3), session_on_days_ago(4)];
        let (current, longest) = compute_streaks(&sessions);
        assert_eq!(current, 0);
        assert_eq!(longest, 2);
    }

    #[test]
    fn test_streak_consecutive_days() {
        // Sessions for 5 consecutive days ending today
        let sessions: Vec<_> = (0..5).map(session_on_days_ago).collect();
        let (current, longest) = compute_streaks(&sessions);
        assert_eq!(current, 5);
        assert_eq!(longest, 5);
    }

    #[test]
    fn test_streak_longer_historical_than_current() {
        // Historical 7-day streak (10-16 days ago) + current 2-day streak (0-1 days ago)
        let mut sessions: Vec<_> = (0..=1).map(session_on_days_ago).collect();
        sessions.extend((10..=16).map(session_on_days_ago));
        let (current, longest) = compute_streaks(&sessions);
        assert_eq!(current, 2);
        assert_eq!(longest, 7);
    }
}
