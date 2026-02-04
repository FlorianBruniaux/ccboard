//! Usage estimation based on billing blocks and subscription plan
//!
//! Provides estimated usage metrics (today, week, month) with comparison
//! to subscription plan limits.

use crate::models::billing_block::BillingBlockManager;
use chrono::{Datelike, Local, NaiveDate};

/// Subscription plan types with approximate monthly budgets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Unknown,
}

impl SubscriptionPlan {
    /// Parse plan from string (from settings.json)
    pub fn from_str(s: &str) -> Self {
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
    /// Note: These are estimates based on typical usage patterns.
    /// Actual limits may vary based on Anthropic's current pricing.
    pub fn monthly_budget_usd(self) -> Option<f64> {
        match self {
            Self::Pro => Some(20.0),
            Self::Max5x => Some(100.0),
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
}

impl UsageEstimate {
    /// Calculate percentage used for today (if budget exists)
    pub fn percent_today(&self) -> Option<f64> {
        self.budget_usd
            .map(|budget| (self.cost_today / budget * 100.0).min(100.0))
    }

    /// Calculate percentage used for week (if budget exists)
    pub fn percent_week(&self) -> Option<f64> {
        self.budget_usd
            .map(|budget| (self.cost_week / budget * 100.0).min(100.0))
    }

    /// Calculate percentage used for month (if budget exists)
    pub fn percent_month(&self) -> Option<f64> {
        self.budget_usd
            .map(|budget| (self.cost_month / budget * 100.0).min(100.0))
    }
}

impl Default for SubscriptionPlan {
    fn default() -> Self {
        Self::Unknown
    }
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

    UsageEstimate {
        cost_today,
        cost_week,
        cost_month,
        plan,
        budget_usd: plan.monthly_budget_usd(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plan() {
        assert_eq!(SubscriptionPlan::from_str("pro"), SubscriptionPlan::Pro);
        assert_eq!(
            SubscriptionPlan::from_str("max5x"),
            SubscriptionPlan::Max5x
        );
        assert_eq!(
            SubscriptionPlan::from_str("max-20x"),
            SubscriptionPlan::Max20x
        );
        assert_eq!(SubscriptionPlan::from_str("api"), SubscriptionPlan::Api);
        assert_eq!(
            SubscriptionPlan::from_str("unknown"),
            SubscriptionPlan::Unknown
        );
    }

    #[test]
    fn test_monthly_budget() {
        assert_eq!(SubscriptionPlan::Pro.monthly_budget_usd(), Some(20.0));
        assert_eq!(SubscriptionPlan::Max5x.monthly_budget_usd(), Some(100.0));
        assert_eq!(SubscriptionPlan::Max20x.monthly_budget_usd(), Some(200.0));
        assert_eq!(SubscriptionPlan::Api.monthly_budget_usd(), None);
        assert_eq!(SubscriptionPlan::Unknown.monthly_budget_usd(), None);
    }

    #[test]
    fn test_percent_calculation() {
        let estimate = UsageEstimate {
            cost_today: 5.0,
            cost_week: 15.0,
            cost_month: 40.0,
            plan: SubscriptionPlan::Max5x,
            budget_usd: Some(100.0),
        };

        assert_eq!(estimate.percent_today(), Some(5.0));
        assert_eq!(estimate.percent_week(), Some(15.0));
        assert_eq!(estimate.percent_month(), Some(40.0));
    }

    #[test]
    fn test_no_budget() {
        let estimate = UsageEstimate {
            cost_today: 5.0,
            cost_week: 15.0,
            cost_month: 40.0,
            plan: SubscriptionPlan::Api,
            budget_usd: None,
        };

        assert_eq!(estimate.percent_today(), None);
        assert_eq!(estimate.percent_week(), None);
        assert_eq!(estimate.percent_month(), None);
    }
}
