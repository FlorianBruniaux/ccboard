//! Usage forecasting with linear regression
//!
//! Predicts future token usage and costs based on historical trends,
//! with R² confidence metric to assess prediction reliability.

use super::trends::TrendsData;

/// Forecast data with predictions
#[derive(Debug, Clone)]
pub struct ForecastData {
    /// Predicted tokens for next 30 days
    pub next_30_days_tokens: u64,
    /// Predicted cost for next 30 days
    pub next_30_days_cost: f64,
    /// Monthly cost estimate (extrapolated)
    pub monthly_cost_estimate: f64,
    /// Confidence (R² coefficient, 0.0-1.0)
    pub confidence: f64,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Reason if unavailable
    pub unavailable_reason: Option<String>,
}

/// Trend direction with percentage change
#[derive(Debug, Clone)]
pub enum TrendDirection {
    /// Increasing trend (percentage)
    Up(f64),
    /// Decreasing trend (percentage)
    Down(f64),
    /// Stable trend (<30% change over 30 days)
    Stable,
}

impl ForecastData {
    /// Create unavailable forecast with reason
    pub fn unavailable(reason: &str) -> Self {
        Self {
            next_30_days_tokens: 0,
            next_30_days_cost: 0.0,
            monthly_cost_estimate: 0.0,
            confidence: 0.0,
            trend_direction: TrendDirection::Stable,
            unavailable_reason: Some(reason.to_string()),
        }
    }
}

/// Forecast usage with linear regression
///
/// Uses simple linear regression (y = slope * x + intercept) to extrapolate
/// token usage 30 days into the future. Confidence is measured using R²
/// (coefficient of determination), not sample size.
///
/// # Performance
/// Target: <20ms
///
/// # Returns
/// - `ForecastData::unavailable()` if <7 days of data
/// - `ForecastData` with R² confidence otherwise
pub fn forecast_usage(trends: &TrendsData) -> ForecastData {
    let days_in_period = calendar_span_days(&trends.dates);
    if days_in_period < 7 {
        return ForecastData::unavailable("Insufficient data (<7 days)");
    }

    // Prepare data points (x = calendar day offset from first date, y = tokens)
    let offsets = calendar_offsets(&trends.dates);
    let points: Vec<_> = offsets
        .iter()
        .zip(trends.daily_tokens.iter())
        .map(|(&x, &tokens)| (x, tokens as f64))
        .collect();

    // Linear regression: y = slope * x + intercept
    let (slope, intercept, r_squared) = linear_regression(&points);

    // R² = coefficient of determination (0.0-1.0)
    // 1.0 = perfect fit, 0.0 = no correlation
    let confidence = r_squared.clamp(0.0, 1.0);

    // Extrapolate 30 days ahead from the most recent calendar day covered
    let last_x = offsets.last().copied().unwrap_or(0.0);
    let next_30_x = last_x + 30.0;
    let next_30_days_tokens = (slope * next_30_x + intercept).max(0.0) as u64;

    // Estimate cost (using current avg cost/token)
    let total_cost: f64 = trends.daily_cost.iter().sum();
    let total_tokens: u64 = trends.daily_tokens.iter().sum();
    let cost_per_token = if total_tokens > 0 {
        total_cost / total_tokens as f64
    } else {
        0.01 / 1000.0 // Default: $0.01/1K tokens
    };
    let next_30_days_cost = next_30_days_tokens as f64 * cost_per_token;

    // Monthly estimate (extrapolate to 30 days from the calendar window average)
    let monthly_cost_estimate = (total_cost / days_in_period as f64) * 30.0;

    // Trend direction: compare the 30-day projected change against a stable
    // baseline (average of the first half of the window) instead of the
    // regression intercept, which can sit near zero and blow up the ratio.
    let half = (points.len() / 2).max(1);
    let baseline: f64 = points.iter().take(half).map(|p| p.1).sum::<f64>() / half as f64;
    let projected_change = slope * 30.0;

    let trend_direction = if baseline.abs() < 1e-6 || projected_change.abs() < 1e-9 {
        TrendDirection::Stable
    } else {
        let change_pct = (projected_change / baseline * 100.0).abs();
        if change_pct < 30.0 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Up(change_pct)
        } else {
            TrendDirection::Down(change_pct)
        }
    };

    ForecastData {
        next_30_days_tokens,
        next_30_days_cost,
        monthly_cost_estimate,
        confidence,
        trend_direction,
        unavailable_reason: None,
    }
}

/// Calendar span in days covered by a sorted list of "YYYY-MM-DD" dates
///
/// Falls back to the entry count if dates fail to parse, so callers still
/// degrade gracefully instead of panicking on malformed data.
fn calendar_span_days(dates: &[String]) -> i64 {
    let (Some(first), Some(last)) = (dates.first(), dates.last()) else {
        return 0;
    };
    match (
        chrono::NaiveDate::parse_from_str(first, "%Y-%m-%d"),
        chrono::NaiveDate::parse_from_str(last, "%Y-%m-%d"),
    ) {
        (Ok(first), Ok(last)) => (last - first).num_days() + 1,
        _ => dates.len() as i64,
    }
}

/// Calendar day offset of each date relative to the first date
///
/// Falls back to the sequential index for entries that fail to parse.
fn calendar_offsets(dates: &[String]) -> Vec<f64> {
    let base = dates
        .first()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    dates
        .iter()
        .enumerate()
        .map(
            |(i, d)| match (base, chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok()) {
                (Some(base), Some(current)) => (current - base).num_days() as f64,
                _ => i as f64,
            },
        )
        .collect()
}

/// Simple linear regression with R² calculation
///
/// Computes the best-fit line y = slope * x + intercept and R² coefficient.
///
/// # Returns
/// (slope, intercept, r_squared)
fn linear_regression(points: &[(f64, f64)]) -> (f64, f64, f64) {
    let n = points.len() as f64;
    let sum_x: f64 = points.iter().map(|p| p.0).sum();
    let sum_y: f64 = points.iter().map(|p| p.1).sum();
    let sum_xx: f64 = points.iter().map(|p| p.0 * p.0).sum();
    let sum_xy: f64 = points.iter().map(|p| p.0 * p.1).sum();

    // Slope and intercept
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    // R² (coefficient of determination)
    let mean_y = sum_y / n;
    let ss_tot: f64 = points.iter().map(|p| (p.1 - mean_y).powi(2)).sum();
    let ss_res: f64 = points
        .iter()
        .map(|p| {
            let predicted = slope * p.0 + intercept;
            (p.1 - predicted).powi(2)
        })
        .sum();

    let r_squared = if ss_tot > 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    (slope, intercept, r_squared)
}
