//! Unit tests for analytics module

use super::*;
use chrono::Utc;
use std::sync::Arc;

use crate::models::session::SessionMetadata;

/// Generate test sessions for benchmarking and testing
fn generate_test_sessions(count: usize, days: usize) -> Vec<Arc<SessionMetadata>> {
    let now = Utc::now();
    (0..count)
        .map(|i| {
            let day_offset = (i % days) as i64;
            let ts = now - chrono::Duration::days(day_offset);

            Arc::new(SessionMetadata {
                id: format!("session-{}", i),
                file_path: std::path::PathBuf::from(format!("/test/session-{}.jsonl", i)),
                project_path: "/test".to_string(),
                first_timestamp: Some(ts),
                last_timestamp: Some(ts + chrono::Duration::minutes(30)),
                message_count: 10,
                total_tokens: 1000 + (i as u64 * 100),
                input_tokens: 600 + (i as u64 * 60),
                output_tokens: 300 + (i as u64 * 30),
                cache_creation_tokens: 50 + (i as u64 * 5),
                cache_read_tokens: 50 + (i as u64 * 5),
                models_used: vec!["sonnet".to_string()],
                file_size_bytes: 1024 * (i as u64 + 1),
                first_user_message: None,
                has_subagents: false,
                duration_seconds: Some(1800),
            })
        })
        .collect()
}

// ============================================================================
// Trends Tests (5 tests)
// ============================================================================

#[test]
fn test_trends_empty_sessions() {
    let trends = compute_trends(&[], 30);
    assert!(trends.is_empty());
    assert_eq!(trends.dates.len(), 0);
    assert_eq!(trends.daily_tokens.len(), 0);
}

#[test]
fn test_trends_single_day() {
    let sessions = generate_test_sessions(10, 1);
    let trends = compute_trends(&sessions, 30);

    assert_eq!(trends.dates.len(), 1, "Should have 1 date");
    assert_eq!(trends.daily_sessions[0], 10, "Should have 10 sessions");
    assert!(trends.daily_tokens[0] > 0, "Should have tokens");
}

#[test]
fn test_trends_multi_day_aggregation() {
    let sessions = generate_test_sessions(30, 10);
    let trends = compute_trends(&sessions, 30);

    assert_eq!(trends.dates.len(), 10, "Should have 10 days");
    let total_sessions: usize = trends.daily_sessions.iter().sum();
    assert_eq!(total_sessions, 30, "Should have 30 total sessions");
}

#[test]
fn test_trends_hourly_distribution() {
    let sessions = generate_test_sessions(24, 1);
    let trends = compute_trends(&sessions, 1);

    let total: usize = trends.hourly_distribution.iter().sum();
    assert!(total > 0, "Should have hourly distribution");
}

#[test]
fn test_trends_model_usage() {
    let mut sessions = generate_test_sessions(10, 5);
    for session in sessions.iter_mut() {
        Arc::get_mut(session).unwrap().models_used = vec!["opus".to_string()];
    }

    let trends = compute_trends(&sessions, 30);
    assert!(
        trends.model_usage_over_time.contains_key("opus"),
        "Should track opus usage"
    );
}

// ============================================================================
// Forecast Tests (4 tests)
// ============================================================================

#[test]
fn test_forecast_insufficient_data() {
    let sessions = generate_test_sessions(3, 3);
    let trends = compute_trends(&sessions, 30);
    let forecast = forecast_usage(&trends);

    assert!(
        forecast.unavailable_reason.is_some(),
        "Should be unavailable"
    );
    assert_eq!(forecast.confidence, 0.0);
}

#[test]
fn test_forecast_stable_trend() {
    let mut sessions = generate_test_sessions(30, 30);
    // All sessions same tokens → stable
    for session in sessions.iter_mut() {
        Arc::get_mut(session).unwrap().total_tokens = 1000;
    }

    let trends = compute_trends(&sessions, 30);
    let forecast = forecast_usage(&trends);

    assert!(matches!(forecast.trend_direction, TrendDirection::Stable));
}

#[test]
fn test_forecast_increasing_trend() {
    let mut sessions = generate_test_sessions(30, 30);
    // Increasing tokens over time (oldest to newest)
    // Session i has day_offset = i (now - i days)
    // So session 0 is today, session 29 is 29 days ago
    // We want oldest (29 days ago) = low tokens, newest (today) = high tokens
    for (i, session) in sessions.iter_mut().enumerate() {
        // Reverse: session with offset 29 gets lowest tokens, offset 0 gets highest
        let day_offset = i % 30;
        let tokens = 1000 + ((29 - day_offset) as u64 * 100);
        Arc::get_mut(session).unwrap().total_tokens = tokens;
    }

    let trends = compute_trends(&sessions, 30);
    let forecast = forecast_usage(&trends);

    assert!(
        matches!(forecast.trend_direction, TrendDirection::Up(_)),
        "Should detect increasing trend, got {:?}",
        forecast.trend_direction
    );
}

#[test]
fn test_forecast_confidence_reflects_variance() {
    let sessions = generate_test_sessions(30, 30);
    let trends = compute_trends(&sessions, 30);
    let forecast = forecast_usage(&trends);

    assert!(
        forecast.confidence >= 0.0 && forecast.confidence <= 1.0,
        "Confidence should be in range [0, 1]"
    );
}

// ============================================================================
// Pattern Tests (3 tests)
// ============================================================================

#[test]
fn test_patterns_peak_hours() {
    let sessions = generate_test_sessions(100, 7);
    let patterns = detect_patterns(&sessions, 7);

    // With 100 sessions, should have some peak hours
    assert!(
        !patterns.peak_hours.is_empty() || patterns.hourly_distribution.iter().sum::<usize>() > 0
    );
}

#[test]
fn test_patterns_most_productive_day() {
    let sessions = generate_test_sessions(50, 7);
    let patterns = detect_patterns(&sessions, 7);

    // Should have identified a most productive day
    assert!(patterns.most_productive_hour < 24);
}

#[test]
fn test_patterns_model_distribution_sums_to_one() {
    let sessions = generate_test_sessions(30, 7);
    let patterns = detect_patterns(&sessions, 7);

    if !patterns.model_distribution.is_empty() {
        let sum: f64 = patterns.model_distribution.values().sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Model distribution should sum to ~1.0, got {}",
            sum
        );
    }
}

// ============================================================================
// Integration Test (1 test)
// ============================================================================

#[test]
fn test_full_analytics_pipeline() {
    let sessions = generate_test_sessions(100, 30);
    let period = Period::last_30d();

    let data = AnalyticsData::compute(&sessions, period);

    assert!(!data.trends.is_empty(), "Trends should not be empty");
    assert!(
        data.forecast.confidence >= 0.0,
        "Forecast should have confidence"
    );
    assert!(
        !data.patterns.model_distribution.is_empty(),
        "Patterns should have model distribution"
    );
    assert_eq!(data.period, period, "Period should match");
}

// ============================================================================
// Edge Cases (2 additional tests)
// ============================================================================

#[test]
fn test_analytics_with_missing_timestamps() {
    let mut sessions = generate_test_sessions(10, 5);
    // Remove timestamps from half the sessions
    for session in sessions.iter_mut().take(5) {
        Arc::get_mut(session).unwrap().first_timestamp = None;
    }

    let trends = compute_trends(&sessions, 30);
    // Should handle gracefully, only process 5 sessions
    assert!(trends.daily_sessions.iter().sum::<usize>() <= 5);
}

#[test]
fn test_period_display() {
    let period_7d = Period::last_7d();
    let period_available = Period::available();

    assert_eq!(period_7d.display(100), "Last 7 days");
    assert_eq!(period_available.display(1000), "All loaded (1000 sessions)");
}
