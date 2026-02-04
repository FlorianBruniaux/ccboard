//! Time series trends analysis
//!
//! Aggregates session data by day, hour, and weekday to identify usage patterns over time.

use chrono::{Datelike, Local, Timelike};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use crate::models::session::SessionMetadata;

/// Session duration statistics
#[derive(Debug, Clone)]
pub struct SessionDurationStats {
    /// Average duration in seconds
    pub avg_duration_secs: f64,
    /// Median duration in seconds
    pub median_duration_secs: f64,
    /// 95th percentile duration in seconds
    pub p95_duration_secs: f64,
    /// Shortest session in seconds
    pub shortest_session_secs: u64,
    /// Longest session in seconds
    pub longest_session_secs: u64,
    /// Distribution buckets (0-5m, 5-15m, 15-30m, 30-60m, 60m+)
    pub distribution: [usize; 5],
}

impl SessionDurationStats {
    /// Empty placeholder (no sessions with duration data)
    pub fn empty() -> Self {
        Self {
            avg_duration_secs: 0.0,
            median_duration_secs: 0.0,
            p95_duration_secs: 0.0,
            shortest_session_secs: 0,
            longest_session_secs: 0,
            distribution: [0; 5],
        }
    }
}

/// Time series trends data
#[derive(Debug, Clone)]
pub struct TrendsData {
    /// Dates in "YYYY-MM-DD" format (sorted chronologically)
    pub dates: Vec<String>,
    /// Daily token counts (aligned with dates)
    pub daily_tokens: Vec<u64>,
    /// Daily session counts (aligned with dates)
    pub daily_sessions: Vec<usize>,
    /// Daily cost estimates (aligned with dates)
    pub daily_cost: Vec<f64>,
    /// Hourly distribution (0-23)
    pub hourly_distribution: [usize; 24],
    /// Weekday distribution (0=Monday, 6=Sunday)
    pub weekday_distribution: [usize; 7],
    /// Model usage over time (aligned with dates)
    pub model_usage_over_time: HashMap<String, Vec<usize>>,
    /// Session duration statistics
    pub duration_stats: SessionDurationStats,
}

impl TrendsData {
    /// Check if empty (no data in period)
    pub fn is_empty(&self) -> bool {
        self.dates.is_empty()
    }

    /// Get tokens at specific date index
    pub fn get_tokens_at(&self, idx: usize) -> Option<(&str, u64)> {
        Some((self.dates.get(idx)?, self.daily_tokens[idx]))
    }

    /// Empty placeholder for no data
    pub fn empty() -> Self {
        Self {
            dates: Vec::new(),
            daily_tokens: Vec::new(),
            daily_sessions: Vec::new(),
            daily_cost: Vec::new(),
            hourly_distribution: [0; 24],
            weekday_distribution: [0; 7],
            model_usage_over_time: HashMap::new(),
            duration_stats: SessionDurationStats::empty(),
        }
    }
}

/// Daily aggregate helper
#[derive(Default)]
struct DailyAggregate {
    tokens: u64,
    sessions: usize,
    cost: f64,
}

/// Estimate cost from session
///
/// TODO: Integrate with StatsCache.model_pricing when available
/// Currently uses placeholder: $0.01 per 1K tokens
fn estimate_cost(session: &SessionMetadata) -> f64 {
    (session.total_tokens as f64 / 1000.0) * 0.01
}

/// Compute trends from sessions
///
/// Aggregates sessions by local date, hour, weekday and model.
/// Converts UTC timestamps to local timezone for grouping.
///
/// # Performance
/// Target: <40ms for 1000 sessions
///
/// # Graceful Degradation
/// - Missing timestamps: Session skipped with warning
/// - Empty models_used: Counted but not tracked per-model
pub fn compute_trends(sessions: &[Arc<SessionMetadata>], days: usize) -> TrendsData {
    let mut daily_map: BTreeMap<String, DailyAggregate> = BTreeMap::new();
    let mut hourly_counts = [0usize; 24];
    let mut weekday_counts = [0usize; 7];
    let mut model_usage: HashMap<String, BTreeMap<String, usize>> = HashMap::new();
    let mut durations_secs: Vec<u64> = Vec::new();

    let now = Local::now();
    let cutoff = now - chrono::Duration::days(days as i64);

    for session in sessions {
        let Some(ts) = session.first_timestamp else {
            tracing::warn!("Session {} missing timestamp, skipping", session.id);
            continue;
        };

        // Convert UTC → Local for grouping
        let local_ts = ts.with_timezone(&Local);

        // Filter by period
        if local_ts < cutoff {
            continue;
        }

        let date_key = local_ts.format("%Y-%m-%d").to_string();

        // Aggregate daily
        let agg = daily_map.entry(date_key.clone()).or_default();
        agg.tokens += session.total_tokens;
        agg.sessions += 1;
        agg.cost += estimate_cost(session);

        // Hourly distribution
        hourly_counts[local_ts.hour() as usize] += 1;

        // Weekday distribution (0 = Monday, 6 = Sunday)
        weekday_counts[local_ts.weekday().num_days_from_monday() as usize] += 1;

        // Model usage over time
        for model in &session.models_used {
            model_usage
                .entry(model.clone())
                .or_default()
                .entry(date_key.clone())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        // Session duration
        if let (Some(start), Some(end)) = (session.first_timestamp, session.last_timestamp) {
            if let Ok(duration) = (end - start).num_seconds().try_into() {
                durations_secs.push(duration);
            }
        }
    }

    // Extract sorted dates + values
    let dates: Vec<String> = daily_map.keys().cloned().collect();
    let daily_tokens: Vec<u64> = daily_map.values().map(|a| a.tokens).collect();
    let daily_sessions: Vec<usize> = daily_map.values().map(|a| a.sessions).collect();
    let daily_cost: Vec<f64> = daily_map.values().map(|a| a.cost).collect();

    // Align model usage with dates
    let model_usage_over_time: HashMap<String, Vec<usize>> = model_usage
        .into_iter()
        .map(|(model, date_map)| {
            let counts = dates
                .iter()
                .map(|d| *date_map.get(d).unwrap_or(&0))
                .collect();
            (model, counts)
        })
        .collect();

    // Compute session duration statistics
    let duration_stats = compute_duration_stats(&durations_secs);

    TrendsData {
        dates,
        daily_tokens,
        daily_sessions,
        daily_cost,
        hourly_distribution: hourly_counts,
        weekday_distribution: weekday_counts,
        model_usage_over_time,
        duration_stats,
    }
}

/// Compute session duration statistics
fn compute_duration_stats(durations: &[u64]) -> SessionDurationStats {
    if durations.is_empty() {
        return SessionDurationStats::empty();
    }

    let mut sorted = durations.to_vec();
    sorted.sort_unstable();

    let shortest = sorted[0];
    let longest = sorted[sorted.len() - 1];

    // Average
    let sum: u64 = sorted.iter().sum();
    let avg = sum as f64 / sorted.len() as f64;

    // Median
    let median = if sorted.len() % 2 == 0 {
        let mid = sorted.len() / 2;
        (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
    } else {
        sorted[sorted.len() / 2] as f64
    };

    // P95 (95th percentile)
    let p95_idx = ((sorted.len() as f64) * 0.95).ceil() as usize - 1;
    let p95 = sorted.get(p95_idx).copied().unwrap_or(longest) as f64;

    // Distribution buckets: 0-5m, 5-15m, 15-30m, 30-60m, 60m+
    let mut distribution = [0usize; 5];
    for &duration in durations {
        let minutes = duration / 60;
        let bucket = match minutes {
            0..=4 => 0,      // 0-5m
            5..=14 => 1,     // 5-15m
            15..=29 => 2,    // 15-30m
            30..=59 => 3,    // 30-60m
            _ => 4,          // 60m+
        };
        distribution[bucket] += 1;
    }

    SessionDurationStats {
        avg_duration_secs: avg,
        median_duration_secs: median,
        p95_duration_secs: p95,
        shortest_session_secs: shortest,
        longest_session_secs: longest,
        distribution,
    }
}
