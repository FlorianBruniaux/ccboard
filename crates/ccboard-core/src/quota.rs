//! Quota tracking and budget alerts
//!
//! MVP approach: Uses total cost from model_usage with prorata for month-to-date.
//! Simple projection based on daily average (no forecasting for v0.8.0 MVP).

use crate::models::{config::BudgetConfig, stats::StatsCache};
use chrono::{Datelike, Local};

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

/// Calculate quota status from stats and budget config
///
/// MVP approach:
/// 1. Calculate MTD cost from total model_usage.cost_usd + prorata for current month
/// 2. Project monthly cost as simple daily average * 30 days
pub fn calculate_quota_status(stats: &StatsCache, budget: &BudgetConfig) -> QuotaStatus {
    // 1. Calculate month-to-date cost (simple prorata from total)
    let current_cost = calculate_month_to_date_cost(stats);

    // 2. Calculate usage percentage
    let usage_pct = if let Some(limit) = budget.monthly_limit {
        (current_cost / limit * 100.0).min(999.9)
    } else {
        0.0
    };

    // 3. Determine alert level
    let alert_level = determine_alert_level(usage_pct, budget);

    // 4. Project monthly cost (simple: daily avg * 30)
    let projected_monthly_cost = project_monthly_cost(stats, current_cost);

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

/// Calculate month-to-date cost using token-based prorata
///
/// Uses daily_model_tokens to compute the proportion of tokens in current month vs total.
fn calculate_month_to_date_cost(stats: &StatsCache) -> f64 {
    // Total cost across all models
    let total_cost: f64 = stats.model_usage.values().map(|m| m.cost_usd).sum();

    if total_cost == 0.0 {
        return 0.0;
    }

    // Get current month prefix (e.g., "2026-02")
    let now = Local::now();
    let month_prefix = format!("{}-{:02}", now.year(), now.month());

    // Calculate total tokens in current month from daily_model_tokens
    let mtd_tokens: u64 = stats
        .daily_model_tokens
        .iter()
        .filter(|d| d.date.starts_with(&month_prefix))
        .flat_map(|d| d.tokens_by_model.values())
        .sum();

    // Calculate total tokens across all models
    let total_tokens: u64 = stats
        .model_usage
        .values()
        .map(|m| m.input_tokens + m.output_tokens)
        .sum();

    if total_tokens == 0 {
        return 0.0;
    }

    // MTD cost = total_cost * (mtd_tokens / total_tokens)
    total_cost * (mtd_tokens as f64 / total_tokens as f64)
}

/// Project monthly cost using simple daily average
///
/// Calculates: (MTD cost / days in month so far) * 30
fn project_monthly_cost(_stats: &StatsCache, mtd_cost: f64) -> f64 {
    let now = Local::now();
    let current_day = now.day() as f64;

    if current_day < 1.0 {
        return mtd_cost * 30.0; // Fallback
    }

    // Daily average * 30 days
    let daily_avg = mtd_cost / current_day;
    daily_avg * 30.0
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
    use crate::models::stats::ModelUsage;
    use std::collections::HashMap;

    /// Mock stats with specified MTD ratio
    ///
    /// `mtd_ratio`: proportion of tokens in current month (0.0-1.0)
    /// Example: mtd_ratio=0.2 means 20% of tokens are in current month
    fn mock_stats_with_mtd_ratio(total_cost: f64, mtd_ratio: f64, first_date: &str) -> StatsCache {
        use crate::models::stats::DailyModelTokens;

        let total_tokens = 15000u64;
        let mtd_tokens = (total_tokens as f64 * mtd_ratio) as u64;

        let mut model_usage = HashMap::new();
        model_usage.insert(
            "claude-sonnet-4".to_string(),
            ModelUsage {
                input_tokens: total_tokens * 2 / 3,
                output_tokens: total_tokens / 3,
                cost_usd: total_cost,
                ..Default::default()
            },
        );

        // Mock daily_model_tokens for current month
        let now = Local::now();
        let current_day = now.day();
        let mut daily_model_tokens = Vec::new();

        // Add token data for each day of current month so far
        if mtd_tokens > 0 {
            for day in 1..=current_day {
                let mut tokens_by_model = HashMap::new();
                tokens_by_model.insert(
                    "claude-sonnet-4".to_string(),
                    mtd_tokens / current_day as u64,
                );

                daily_model_tokens.push(DailyModelTokens {
                    date: format!("{}-{:02}-{:02}", now.year(), now.month(), day),
                    tokens_by_model,
                });
            }
        }

        StatsCache {
            model_usage,
            daily_model_tokens,
            first_session_date: Some(first_date.to_string()),
            ..Default::default()
        }
    }

    // Convenience wrapper for backward compatibility
    fn mock_stats_with_total_cost(total_cost: f64, first_date: &str) -> StatsCache {
        mock_stats_with_mtd_ratio(total_cost, 1.0, first_date)
    }

    #[test]
    fn test_quota_status_safe() {
        // $100 total cost, 20% in current month → ~$20 MTD
        // With $50 limit → 40% usage → Safe
        let stats = mock_stats_with_mtd_ratio(100.0, 0.2, "2024-01-01");
        let budget = BudgetConfig {
            monthly_limit: Some(50.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&stats, &budget);

        assert_eq!(status.alert_level, AlertLevel::Safe);
        assert!(status.usage_pct < 75.0);
    }

    #[test]
    fn test_quota_status_warning() {
        // High total cost → high MTD
        let stats = mock_stats_with_total_cost(500.0, "2024-01-01");
        let budget = BudgetConfig {
            monthly_limit: Some(100.0),
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&stats, &budget);

        // Should be warning or critical or exceeded
        assert!(
            status.alert_level == AlertLevel::Warning
                || status.alert_level == AlertLevel::Critical
                || status.alert_level == AlertLevel::Exceeded
        );
    }

    #[test]
    fn test_quota_no_limit() {
        let stats = mock_stats_with_total_cost(1000.0, "2024-01-01");
        let budget = BudgetConfig {
            monthly_limit: None,
            warning_threshold: 75.0,
            critical_threshold: 90.0,
        };
        let status = calculate_quota_status(&stats, &budget);

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
