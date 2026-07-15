//! Quota tracking and budget alerts
//!
//! Month-to-date cost sums true per-session cost (via `pricing::calculate_cost`)
//! for sessions started in the current calendar month. Monthly projection uses a
//! simple daily average (no forecasting for v0.8.0 MVP).

use crate::models::{config::BudgetConfig, session::SessionMetadata};
use crate::usage_estimator::days_in_month;
use chrono::{Datelike, Local};
use std::sync::Arc;

/// Alert level based on budget usage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    /// Usage < warning threshold (green)
    Safe,
    /// Usage >= warning threshold (yellow)
    Warning,
    /// Usage >= critical threshold (red)
    Critical,
    /// Usage >= 100% (magenta)
    Exceeded,
}

/// Quota status with current usage and projections
#[derive(Debug, Clone)]
pub struct QuotaStatus {
    /// Current month-to-date cost in USD
    pub current_cost: f64,
    /// Configured monthly budget limit (None = unlimited)
    pub budget_limit: Option<f64>,
    /// Usage percentage (0.0-999.9, clamped for display)
    pub usage_pct: f64,
    /// Projected cost at end of month (simple daily average)
    pub projected_monthly_cost: f64,
    /// Projected overage (None if under budget or no limit)
    pub projected_overage: Option<f64>,
    /// Current alert level
    pub alert_level: AlertLevel,
}

/// Calculate quota status from sessions and budget config
///
/// 1. Calculate MTD cost by summing true per-session cost (via `pricing::calculate_cost`)
///    for sessions started in the current calendar month
/// 2. Project monthly cost as simple daily average * days in the current month
pub fn calculate_quota_status(
    sessions: &[Arc<SessionMetadata>],
    budget: &BudgetConfig,
) -> QuotaStatus {
    // 1. Calculate month-to-date cost (true per-session cost sum)
    let current_cost = calculate_month_to_date_cost(sessions);

    // 2. Calculate usage percentage
    let usage_pct = if let Some(limit) = budget.monthly_limit {
        (current_cost / limit * 100.0).min(999.9)
    } else {
        0.0
    };

    // 3. Determine alert level
    let alert_level = determine_alert_level(usage_pct, budget);

    // 4. Project monthly cost (simple: daily avg * days in month)
    let projected_monthly_cost = project_monthly_cost(current_cost);

    // 5. Calculate projected overage if limit exists
    let projected_overage = if let Some(limit) = budget.monthly_limit {
        if projected_monthly_cost > limit {
            Some(projected_monthly_cost - limit)
        } else {
            None
        }
    } else {
        None
    };

    QuotaStatus {
        current_cost,
        budget_limit: budget.monthly_limit,
        usage_pct,
        projected_monthly_cost,
        projected_overage,
        alert_level,
    }
}

/// Calculate month-to-date cost by summing true per-session cost
///
/// Filters sessions started in the current calendar month (local time) and sums
/// each session's cost via `pricing::calculate_cost`, attributed to
/// `SessionMetadata::primary_model()` instead of a blended average rate.
fn calculate_month_to_date_cost(sessions: &[Arc<SessionMetadata>]) -> f64 {
    let now = Local::now();
    let month_start = now.date_naive().with_day(1).expect("day 1 is always valid");

    sessions
        .iter()
        .filter_map(|session| {
            let ts = session.first_timestamp?;
            if ts.with_timezone(&Local).date_naive() < month_start {
                return None;
            }
            Some(crate::pricing::calculate_cost(
                session.primary_model(),
                session.input_tokens,
                session.output_tokens,
                session.cache_creation_tokens,
                session.cache_read_tokens,
            ))
        })
        .sum()
}

/// Project monthly cost using simple daily average
///
/// Calculates: (MTD cost / days in month so far) * days in the current month
fn project_monthly_cost(mtd_cost: f64) -> f64 {
    let now = Local::now();
    let current_day = now.day() as f64;
    let total_days = days_in_month(now.date_naive()) as f64;

    if current_day < 1.0 {
        return mtd_cost * total_days; // Fallback
    }

    let daily_avg = mtd_cost / current_day;
    daily_avg * total_days
}

/// Determine alert level from usage percentage and thresholds
fn determine_alert_level(usage_pct: f64, budget: &BudgetConfig) -> AlertLevel {
    if usage_pct >= 100.0 {
        AlertLevel::Exceeded
    } else if usage_pct >= budget.critical_threshold {
        AlertLevel::Critical
    } else if usage_pct >= budget.warning_threshold {
        AlertLevel::Warning
    } else {
        AlertLevel::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration};

    /// Mock a session started at `local_ts` with the given token counts.
    fn mock_session(
        local_ts: DateTime<Local>,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Arc<SessionMetadata> {
        let ts = local_ts.with_timezone(&chrono::Utc);
        let mut session = SessionMetadata::from_path(
            std::path::PathBuf::from(format!("/test/{}.jsonl", ts.timestamp_micros())),
            "/test".into(),
        );
        session.first_timestamp = Some(ts);
        session.last_timestamp = Some(ts);
        session.input_tokens = input_tokens;
        session.output_tokens = output_tokens;
        session.total_tokens = input_tokens + output_tokens;
        session.models_used = vec!["claude-sonnet-4-5".to_string()];
        Arc::new(session)
    }

    #[test]
    fn test_quota_status_safe() {
        // Small MTD spend against a generous limit → Safe
        let sessions = vec![mock_session(Local::now(), 10_000, 5_000)];
        let budget = BudgetConfig {
            monthly_limit: Some(1000.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&sessions, &budget);

        assert_eq!(status.alert_level, AlertLevel::Safe);
        assert!(status.usage_pct < 75.0);
    }

    #[test]
    fn test_quota_status_warning() {
        // Large MTD spend against a tight limit → Warning or above
        let sessions = vec![mock_session(Local::now(), 5_000_000, 2_000_000)];
        let budget = BudgetConfig {
            monthly_limit: Some(10.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&sessions, &budget);

        assert!(
            status.alert_level == AlertLevel::Warning
                || status.alert_level == AlertLevel::Critical
                || status.alert_level == AlertLevel::Exceeded
        );
    }

    #[test]
    fn test_quota_excludes_sessions_outside_current_month() {
        // A session from 45 days ago always falls outside the current calendar month
        let sessions = vec![mock_session(
            Local::now() - Duration::days(45),
            5_000_000,
            2_000_000,
        )];
        let budget = BudgetConfig {
            monthly_limit: Some(10.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&sessions, &budget);

        assert_eq!(status.current_cost, 0.0);
        assert_eq!(status.alert_level, AlertLevel::Safe);
    }

    #[test]
    fn test_quota_no_limit() {
        let sessions = vec![mock_session(Local::now(), 1_000_000, 500_000)];
        let budget = BudgetConfig {
            monthly_limit: None,
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&sessions, &budget);

        assert_eq!(status.alert_level, AlertLevel::Safe);
        assert_eq!(status.usage_pct, 0.0);
        assert!(status.projected_overage.is_none());
    }

    #[test]
    fn test_determine_alert_level() {
        let budget = BudgetConfig {
            monthly_limit: Some(100.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };

        assert_eq!(determine_alert_level(50.0, &budget), AlertLevel::Safe);
        assert_eq!(determine_alert_level(75.0, &budget), AlertLevel::Warning);
        assert_eq!(determine_alert_level(90.0, &budget), AlertLevel::Critical);
        assert_eq!(determine_alert_level(100.0, &budget), AlertLevel::Exceeded);
        assert_eq!(determine_alert_level(120.0, &budget), AlertLevel::Exceeded);
    }
}
