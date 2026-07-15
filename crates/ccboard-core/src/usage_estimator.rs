//! Usage estimation based on billing blocks and subscription plan
//!
//! Provides estimated usage metrics (today, week, month) with comparison
//! to subscription plan limits.

use crate::models::billing_block::BillingBlockManager;
use chrono::{Datelike, Local, NaiveDate};

/// Subscription plan types with approximate monthly budgets
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SubscriptionPlan {
    /// Claude Pro (~$20/month)
    Pro,
    /// Claude Max 5x (~$100/month)
    Max5x,
    /// Claude Max 20x (~$200/month)
    Max20x,
    /// API usage (pay-as-you-go, no fixed limit)
    Api,
    /// Unknown/unset plan
    #[default]
    Unknown,
}

impl SubscriptionPlan {
    /// Parse plan from string (from settings.json)
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pro" => Self::Pro,
            "max5x" | "max-5x" | "max_5x" => Self::Max5x,
            "max20x" | "max-20x" | "max_20x" => Self::Max20x,
            "api" => Self::Api,
            _ => Self::Unknown,
        }
    }

    /// Get approximate monthly budget in USD
    ///
    /// Note: These are subscription costs, not spending limits.
    /// Max plans have rate limits (requests/day), not fixed monthly budgets.
    /// Use these values as reference points for cost estimation.
    pub fn monthly_budget_usd(self) -> Option<f64> {
        match self {
            Self::Pro => Some(20.0),
            Self::Max5x => Some(50.0), // Updated from $100 to $50 (2025 pricing)
            Self::Max20x => Some(200.0),
            Self::Api => None, // Pay-as-you-go, no fixed limit
            Self::Unknown => None,
        }
    }

    /// Get display name
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Pro => "Claude Pro",
            Self::Max5x => "Claude Max 5x",
            Self::Max20x => "Claude Max 20x",
            Self::Api => "API (Pay-as-you-go)",
            Self::Unknown => "Unknown Plan",
        }
    }
}

/// Estimated usage metrics
#[derive(Debug, Clone, Default)]
pub struct UsageEstimate {
    /// Cost today in USD
    pub cost_today: f64,
    /// Cost this week in USD
    pub cost_week: f64,
    /// Cost this month in USD
    pub cost_month: f64,
    /// Subscription plan
    pub plan: SubscriptionPlan,
    /// Monthly budget (if applicable)
    pub budget_usd: Option<f64>,
    /// Monthly budget prorated to a single day of the current month (if applicable)
    pub daily_budget_usd: Option<f64>,
    /// Monthly budget prorated to a week of the current month (if applicable)
    pub weekly_budget_usd: Option<f64>,
}

/// Display cap for usage percentages, matching quota.rs's QuotaStatus::usage_pct.
/// 100.0 would silently hide how far over budget a period is; a high cap keeps
/// the magnitude visible while still bounding the value for display.
const MAX_DISPLAY_PERCENT: f64 = 999.9;

impl UsageEstimate {
    /// Calculate percentage used for today against the prorated daily budget
    pub fn percent_today(&self) -> Option<f64> {
        self.daily_budget_usd
            .map(|budget| (self.cost_today / budget * 100.0).min(MAX_DISPLAY_PERCENT))
    }

    /// Calculate percentage used for the week against the prorated weekly budget
    pub fn percent_week(&self) -> Option<f64> {
        self.weekly_budget_usd
            .map(|budget| (self.cost_week / budget * 100.0).min(MAX_DISPLAY_PERCENT))
    }

    /// Calculate percentage used for month against the full monthly budget
    pub fn percent_month(&self) -> Option<f64> {
        self.budget_usd
            .map(|budget| (self.cost_month / budget * 100.0).min(MAX_DISPLAY_PERCENT))
    }
}

/// Number of days in the month containing `date`
pub(crate) fn days_in_month(date: NaiveDate) -> i64 {
    let (next_year, next_month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    let next_month_start = NaiveDate::from_ymd_opt(next_year, next_month, 1).unwrap();
    let this_month_start = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap();
    (next_month_start - this_month_start).num_days()
}

/// Calculate usage estimate from billing blocks
pub fn calculate_usage_estimate(
    billing_blocks: &BillingBlockManager,
    plan: SubscriptionPlan,
) -> UsageEstimate {
    let now = Local::now();
    let today = now.date_naive();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);
    let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();

    let mut cost_today = 0.0;
    let mut cost_week = 0.0;
    let mut cost_month = 0.0;

    // Sum costs from billing blocks
    for (block, usage) in billing_blocks.get_all_blocks() {
        let block_date = block.date;
        let cost = usage.total_cost;

        if block_date == today {
            cost_today += cost;
        }
        if block_date >= week_start {
            cost_week += cost;
        }
        if block_date >= month_start {
            cost_month += cost;
        }
    }

    // The plan's monthly_budget_usd() is a single monthly figure (e.g. $200 for
    // Max20x). Using it as-is for "today" and "this week" made both periods
    // saturate at 100% almost immediately, since a full month's budget is a huge
    // denominator for a single day. Prorate it instead.
    let budget_usd = plan.monthly_budget_usd();
    let days = days_in_month(today) as f64;
    let daily_budget_usd = budget_usd.map(|b| b / days);
    let weekly_budget_usd = budget_usd.map(|b| b * 7.0 / days);

    UsageEstimate {
        cost_today,
        cost_week,
        cost_month,
        plan,
        budget_usd,
        daily_budget_usd,
        weekly_budget_usd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plan() {
        assert_eq!(SubscriptionPlan::parse("pro"), SubscriptionPlan::Pro);
        assert_eq!(SubscriptionPlan::parse("max5x"), SubscriptionPlan::Max5x);
        assert_eq!(SubscriptionPlan::parse("max-20x"), SubscriptionPlan::Max20x);
        assert_eq!(SubscriptionPlan::parse("api"), SubscriptionPlan::Api);
        assert_eq!(
            SubscriptionPlan::parse("unknown"),
            SubscriptionPlan::Unknown
        );
    }

    #[test]
    fn test_monthly_budget() {
        assert_eq!(SubscriptionPlan::Pro.monthly_budget_usd(), Some(20.0));
        assert_eq!(SubscriptionPlan::Max5x.monthly_budget_usd(), Some(50.0));
        assert_eq!(SubscriptionPlan::Max20x.monthly_budget_usd(), Some(200.0));
        assert_eq!(SubscriptionPlan::Api.monthly_budget_usd(), None);
        assert_eq!(SubscriptionPlan::Unknown.monthly_budget_usd(), None);
    }

    #[test]
    fn test_percent_calculation_uses_prorated_budgets() {
        // Each period is checked against its own prorated budget, not the flat
        // monthly figure: $5 against a $2 daily budget, $15 against a $10 weekly
        // budget, $40 against the full $50 monthly budget.
        let estimate = UsageEstimate {
            cost_today: 5.0,
            cost_week: 15.0,
            cost_month: 40.0,
            plan: SubscriptionPlan::Max5x,
            budget_usd: Some(50.0),
            daily_budget_usd: Some(2.0),
            weekly_budget_usd: Some(10.0),
        };

        assert_eq!(estimate.percent_today(), Some(250.0));
        assert_eq!(estimate.percent_week(), Some(150.0));
        assert_eq!(estimate.percent_month(), Some(80.0));
    }

    #[test]
    fn test_percent_caps_at_max_display_percent_instead_of_100() {
        // Regression test: today/week/month percentages used to be clamped at
        // 100.0, which made any period spending more than its budget
        // indistinguishable from one spending exactly its budget. They should
        // still be bounded (so extreme values don't blow up widget layouts),
        // but the cap must be well above 100 to keep an "over budget" signal.
        let estimate = UsageEstimate {
            cost_today: 1000.0,
            cost_week: 1000.0,
            // Unlike cost_today/cost_week, $1000 against the full $200 monthly
            // budget is only 500% -- under the cap -- so use a value that
            // actually exceeds it here too.
            cost_month: 3000.0,
            plan: SubscriptionPlan::Max20x,
            budget_usd: Some(200.0),
            daily_budget_usd: Some(200.0 / 31.0),
            weekly_budget_usd: Some(200.0 * 7.0 / 31.0),
        };

        assert_eq!(estimate.percent_today(), Some(MAX_DISPLAY_PERCENT));
        assert_eq!(estimate.percent_week(), Some(MAX_DISPLAY_PERCENT));
        assert_eq!(estimate.percent_month(), Some(MAX_DISPLAY_PERCENT));
    }

    #[test]
    fn test_no_budget() {
        let estimate = UsageEstimate {
            cost_today: 5.0,
            cost_week: 15.0,
            cost_month: 40.0,
            plan: SubscriptionPlan::Api,
            budget_usd: None,
            daily_budget_usd: None,
            weekly_budget_usd: None,
        };

        assert_eq!(estimate.percent_today(), None);
        assert_eq!(estimate.percent_week(), None);
        assert_eq!(estimate.percent_month(), None);
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(
            days_in_month(NaiveDate::from_ymd_opt(2026, 2, 15).unwrap()),
            28
        );
        assert_eq!(
            days_in_month(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()), // leap year
            29
        );
        assert_eq!(
            days_in_month(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()),
            31
        );
        assert_eq!(
            days_in_month(NaiveDate::from_ymd_opt(2026, 12, 25).unwrap()),
            31
        );
    }

    #[test]
    fn test_calculate_usage_estimate_prorates_budget_from_plan() {
        let manager = BillingBlockManager::new();
        let estimate = calculate_usage_estimate(&manager, SubscriptionPlan::Max20x);

        let today = Local::now().date_naive();
        let expected_days = days_in_month(today) as f64;

        assert_eq!(estimate.budget_usd, Some(200.0));
        assert_eq!(estimate.daily_budget_usd, Some(200.0 / expected_days));
        assert_eq!(
            estimate.weekly_budget_usd,
            Some(200.0 * 7.0 / expected_days)
        );
    }
}
