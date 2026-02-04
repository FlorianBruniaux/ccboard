//! Actionable insights generation
//!
//! Rule-based recommendations to optimize costs and productivity.

use super::forecasting::{ForecastData, TrendDirection};
use super::patterns::UsagePatterns;
use super::trends::TrendsData;

/// Generate actionable insights
///
/// Uses rule-based thresholds to identify optimization opportunities:
/// - Peak hours >30% → batch work suggestion
/// - Opus >20% → review necessity
/// - Cost imbalance (token vs cost ratio >1.5x) → expensive model warning
/// - Cost trend >+20% with confidence >0.5 → budget alert
/// - Weekend usage <10% → weekday optimization
/// - Low confidence (<0.5) → unreliable forecast warning
///
/// # Performance
/// Target: <10ms
pub fn generate_insights(
    _trends: &TrendsData,
    patterns: &UsagePatterns,
    forecast: &ForecastData,
) -> Vec<String> {
    let mut insights = Vec::new();

    // 1. Peak hours insight (>30% of sessions)
    let total_sessions: usize = patterns.hourly_distribution.iter().sum();
    if !patterns.peak_hours.is_empty() && total_sessions > 0 {
        let peak_count: usize = patterns
            .peak_hours
            .iter()
            .map(|&h| patterns.hourly_distribution[h as usize])
            .sum();

        if peak_count > total_sessions * 3 / 10 {
            insights.push(format!(
                "Peak hours: {:02}h-{:02}h ({:.0}% of sessions). Consider batching work.",
                patterns.peak_hours.first().unwrap_or(&0),
                patterns.peak_hours.last().unwrap_or(&23),
                peak_count as f64 / total_sessions as f64 * 100.0
            ));
        }
    }

    // 2. Expensive model warning (Opus >20% usage)
    if let Some(&opus_pct) = patterns.model_distribution.get("opus") {
        if opus_pct > 0.2 {
            insights.push(format!(
                "Opus usage: {:.0}% tokens. Costs 3x more than Sonnet. Review necessity.",
                opus_pct * 100.0
            ));
        }
    }

    // 3. Cost imbalance (tokens vs cost distribution mismatch)
    for (model, &token_pct) in &patterns.model_distribution {
        if let Some(&cost_pct) = patterns.model_cost_distribution.get(model) {
            let cost_premium = cost_pct / token_pct;
            if cost_premium > 1.5 && cost_pct > 0.2 {
                insights.push(format!(
                    "{}: {:.0}% tokens but {:.0}% cost. Cost premium: {:.1}x.",
                    model,
                    token_pct * 100.0,
                    cost_pct * 100.0,
                    cost_premium
                ));
            }
        }
    }

    // 4. Cost trend warning (>20% increase)
    if let TrendDirection::Up(pct) = forecast.trend_direction {
        if pct > 20.0 && forecast.confidence > 0.5 {
            insights.push(format!(
                "Cost trend: +{:.0}% over period. Monthly estimate: ${:.2} (confidence: {:.0}%).",
                pct,
                forecast.monthly_cost_estimate,
                forecast.confidence * 100.0
            ));
        }
    }

    // 5. Weekend optimization (usage <10%)
    let weekday_sum: usize = patterns.weekday_distribution[0..5].iter().sum();
    let weekend_sum: usize = patterns.weekday_distribution[5..7].iter().sum();
    let total = weekday_sum + weekend_sum;

    if total > 0 {
        let weekend_pct = weekend_sum as f64 / total as f64;
        if weekend_pct < 0.1 {
            insights.push(format!(
                "Weekend usage: {:.0}%. Consider weekday-focused workflows.",
                weekend_pct * 100.0
            ));
        }
    }

    // 6. Low confidence warning
    if forecast.confidence < 0.5 {
        insights.push(format!(
            "Forecast confidence low ({:.0}%). Predictions may be unreliable.",
            forecast.confidence * 100.0
        ));
    }

    insights
}
