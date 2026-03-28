//! Anomaly detection for unusual spikes/drops in token usage or session costs
//!
//! Uses Z-score based statistical analysis to flag sessions that deviate
//! significantly from normal behavior patterns.

use crate::models::config::AnomalyThresholds;
use crate::models::session::{SessionId, SessionMetadata};
use chrono::{Local, NaiveDate};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Severity level for anomalies based on standard deviations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalySeverity {
    /// Critical: >3 standard deviations from mean
    Critical,
    /// Warning: >2 standard deviations from mean
    Warning,
}

impl AnomalySeverity {
    /// Icon representation for TUI display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Critical => "🚨",
            Self::Warning => "⚠️",
        }
    }

    /// Color name for TUI styling
    pub fn color_name(&self) -> &'static str {
        match self {
            Self::Critical => "red",
            Self::Warning => "yellow",
        }
    }
}

/// Metric type for anomaly detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyMetric {
    /// Total token usage
    Tokens,
    /// Session cost estimate
    Cost,
}

impl AnomalyMetric {
    /// Display name for the metric
    pub fn name(&self) -> &'static str {
        match self {
            Self::Tokens => "Tokens",
            Self::Cost => "Cost",
        }
    }
}

/// Detected anomaly with context
#[derive(Debug, Clone)]
pub struct Anomaly {
    /// Session ID
    pub session_id: SessionId,
    /// Session date (ISO format)
    pub date: String,
    /// Metric that triggered anomaly
    pub metric: AnomalyMetric,
    /// Actual value
    pub value: f64,
    /// Z-score (number of standard deviations from mean)
    pub z_score: f64,
    /// Deviation percentage from mean
    pub deviation_pct: f64,
    /// Severity level
    pub severity: AnomalySeverity,
}

impl Anomaly {
    /// Format value based on metric type
    pub fn format_value(&self) -> String {
        match self.metric {
            AnomalyMetric::Tokens => format!("{:.0}", self.value),
            AnomalyMetric::Cost => format!("${:.2}", self.value),
        }
    }

    /// Format deviation as percentage with sign
    pub fn format_deviation(&self) -> String {
        let sign = if self.deviation_pct >= 0.0 { "+" } else { "" };
        format!("{}{:.0}%", sign, self.deviation_pct)
    }
}

/// Statistical summary for anomaly detection
#[derive(Debug, Clone)]
struct Statistics {
    mean: f64,
    std_dev: f64,
    #[allow(dead_code)]
    count: usize,
}

impl Statistics {
    /// Calculate statistics from a set of values
    fn compute(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let mean = values.iter().sum::<f64>() / count as f64;

        // Calculate variance
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / count as f64;

        let std_dev = variance.sqrt();

        Some(Self {
            mean,
            std_dev,
            count,
        })
    }

    /// Calculate Z-score for a value
    fn z_score(&self, value: f64) -> f64 {
        if self.std_dev < f64::EPSILON {
            return 0.0; // Avoid division by zero
        }
        (value - self.mean) / self.std_dev
    }

    /// Calculate deviation percentage from mean
    fn deviation_pct(&self, value: f64) -> f64 {
        if self.mean == 0.0 {
            return 0.0;
        }
        ((value - self.mean) / self.mean) * 100.0
    }
}

/// Detect anomalies in session token usage
///
/// # Algorithm
/// - Calculate mean (μ) and standard deviation (σ) for all sessions
/// - Z-score = (x - μ) / σ
/// - Flag if |z| > 2 (warning) or |z| > 3 (critical)
///
/// # Requirements
/// - Minimum 10 sessions for meaningful statistics
/// - Returns empty vec if insufficient data
///
/// # Returns
/// Vec of anomalies sorted by severity (critical first), then by z-score (descending)
pub fn detect_anomalies(sessions: &[Arc<SessionMetadata>]) -> Vec<Anomaly> {
    detect_anomalies_with_thresholds(sessions, &AnomalyThresholds::default())
}

/// Same as [`detect_anomalies`] but uses caller-supplied thresholds.
pub fn detect_anomalies_with_thresholds(
    sessions: &[Arc<SessionMetadata>],
    thresholds: &AnomalyThresholds,
) -> Vec<Anomaly> {
    if sessions.len() < thresholds.min_sessions {
        return vec![];
    }

    let mut anomalies = Vec::new();

    // Token anomalies
    let token_values: Vec<f64> = sessions.iter().map(|s| s.total_tokens as f64).collect();

    if let Some(stats) = Statistics::compute(&token_values) {
        // Only detect anomalies if there's variation (std_dev > 0)
        if stats.std_dev > 0.0 {
            for (idx, session) in sessions.iter().enumerate() {
                let value = token_values[idx];
                let z_score = stats.z_score(value);
                let abs_z = z_score.abs();

                if abs_z > thresholds.warning_z_score {
                    let severity = if abs_z > thresholds.critical_z_score {
                        AnomalySeverity::Critical
                    } else {
                        AnomalySeverity::Warning
                    };

                    anomalies.push(Anomaly {
                        session_id: session.id.clone(),
                        date: session
                            .first_timestamp
                            .as_ref()
                            .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        metric: AnomalyMetric::Tokens,
                        value,
                        z_score,
                        deviation_pct: stats.deviation_pct(value),
                        severity,
                    });
                }
            }
        }
    }

    // TODO: Cost anomalies
    // Cost calculation requires StatsCache for pricing data, which is not available here.
    // For now, focus on token-based anomaly detection only.
    // Future enhancement: Pass pricing data to this function or calculate cost per session.

    // Sort by severity (critical first), then by z-score absolute value (descending)
    anomalies.sort_by(|a, b| {
        match (a.severity, b.severity) {
            (AnomalySeverity::Critical, AnomalySeverity::Warning) => std::cmp::Ordering::Less,
            (AnomalySeverity::Warning, AnomalySeverity::Critical) => std::cmp::Ordering::Greater,
            _ => {
                // Same severity: sort by z-score magnitude
                b.z_score
                    .abs()
                    .partial_cmp(&a.z_score.abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
        }
    });

    anomalies
}

/// A daily cost spike: one day's estimated cost is an outlier vs the recent baseline.
#[derive(Debug, Clone)]
pub struct DailyCostAnomaly {
    /// Date of the spike
    pub date: NaiveDate,
    /// Estimated cost for this day (tokens × $0.01/K)
    pub cost_estimate: f64,
    /// Rolling mean cost per day over the window
    pub avg_cost: f64,
    /// cost_estimate / avg_cost — how many times above average
    pub ratio: f64,
    /// Severity based on ratio
    pub severity: AnomalySeverity,
}

impl DailyCostAnomaly {
    /// Human-readable label, e.g. "3.2× average"
    pub fn format_ratio(&self) -> String {
        format!("{:.1}× avg", self.ratio)
    }

    /// Formatted cost
    pub fn format_cost(&self) -> String {
        format!("${:.3}", self.cost_estimate)
    }
}

/// Detect days where cost is ≥2× the rolling daily average.
///
/// Requires at least 7 days of data. Aggregates sessions by calendar day,
/// flags days above the threshold. Returns results sorted most recent first.
pub fn detect_daily_cost_spikes(
    sessions: &[Arc<SessionMetadata>],
    window_days: usize,
) -> Vec<DailyCostAnomaly> {
    detect_daily_cost_spikes_with_thresholds(sessions, window_days, &AnomalyThresholds::default())
}

/// Same as [`detect_daily_cost_spikes`] but uses caller-supplied thresholds.
pub fn detect_daily_cost_spikes_with_thresholds(
    sessions: &[Arc<SessionMetadata>],
    window_days: usize,
    thresholds: &AnomalyThresholds,
) -> Vec<DailyCostAnomaly> {
    const MIN_DAYS: usize = 7;
    let spike_2x = thresholds.spike_2x;
    let spike_3x = thresholds.spike_3x;
    // Rough cost estimate: $0.01 per 1K tokens
    const COST_PER_1K_TOKENS: f64 = 0.01;

    let cutoff_date = (Local::now() - chrono::Duration::days(window_days as i64)).date_naive();

    // Aggregate estimated cost per day
    let mut daily_costs: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    for session in sessions {
        if let Some(ts) = session.first_timestamp {
            let local_date = ts.with_timezone(&Local).date_naive();
            if local_date < cutoff_date {
                continue;
            }
            let cost = (session.total_tokens as f64 / 1000.0) * COST_PER_1K_TOKENS;
            *daily_costs.entry(local_date).or_default() += cost;
        }
    }

    if daily_costs.len() < MIN_DAYS {
        return vec![];
    }

    let costs: Vec<f64> = daily_costs.values().copied().collect();
    let Some(stats) = Statistics::compute(&costs) else {
        return vec![];
    };

    // Need meaningful cost data (skip if average < $0.01/day)
    if stats.mean < 0.01 {
        return vec![];
    }

    let mut spikes: Vec<DailyCostAnomaly> = daily_costs
        .iter()
        .filter_map(|(date, &cost)| {
            let ratio = cost / stats.mean;
            if ratio >= spike_2x {
                let severity = if ratio >= spike_3x {
                    AnomalySeverity::Critical
                } else {
                    AnomalySeverity::Warning
                };
                Some(DailyCostAnomaly {
                    date: *date,
                    cost_estimate: cost,
                    avg_cost: stats.mean,
                    ratio,
                    severity,
                })
            } else {
                None
            }
        })
        .collect();

    // Most recent first
    spikes.sort_by(|a, b| b.date.cmp(&a.date));
    spikes
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_session(id: &str, tokens: u64) -> Arc<SessionMetadata> {
        Arc::new(SessionMetadata {
            id: id.into(),
            source_tool: None,
            file_path: PathBuf::from(format!("/tmp/{}.jsonl", id)),
            project_path: "test".into(),
            first_timestamp: Some(Utc::now()),
            last_timestamp: Some(Utc::now()),
            message_count: 10,
            total_tokens: tokens,
            input_tokens: tokens / 2,
            output_tokens: tokens / 2,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec!["test-model".to_string()],
            model_segments: Vec::new(),
            file_size_bytes: 1024,
            first_user_message: Some("test message".to_string()),
            has_subagents: false,
            parent_session_id: None,
            duration_seconds: Some(60),
            branch: Some("main".to_string()),
            tool_usage: std::collections::HashMap::new(),
            tool_token_usage: std::collections::HashMap::new(),
        })
    }

    #[test]
    fn test_insufficient_data() {
        let sessions = vec![
            create_test_session("s1", 1000),
            create_test_session("s2", 2000),
        ];

        let anomalies = detect_anomalies(&sessions);
        assert_eq!(
            anomalies.len(),
            0,
            "Should return empty vec for <10 sessions"
        );
    }

    #[test]
    fn test_no_variance() {
        let sessions: Vec<_> = (0..15)
            .map(|i| create_test_session(&format!("s{}", i), 1000))
            .collect();

        let anomalies = detect_anomalies(&sessions);
        assert_eq!(
            anomalies.len(),
            0,
            "Should return empty vec when all values identical"
        );
    }

    #[test]
    fn test_detect_token_outliers() {
        // Create sessions with normal distribution + outliers
        let mut sessions = vec![];

        // 10 normal sessions (1000-1100 tokens) - tight distribution
        for i in 0..10 {
            sessions.push(create_test_session(
                &format!("normal_{}", i),
                1000 + (i * 10) as u64,
            ));
        }

        // 1 critical outlier (20x more tokens) - clearly anomalous
        sessions.push(create_test_session("critical", 20000));

        let anomalies = detect_anomalies(&sessions);

        assert!(
            !anomalies.is_empty(),
            "Should detect at least 1 anomaly with extreme outlier"
        );

        // First should be critical
        assert_eq!(
            anomalies[0].severity,
            AnomalySeverity::Critical,
            "Extreme outlier should be critical"
        );
        assert_eq!(anomalies[0].metric, AnomalyMetric::Tokens);
        assert!(
            anomalies[0].z_score.abs() > 3.0,
            "Critical anomaly should have |z| > 3"
        );
    }

    #[test]
    fn test_severity_sorting() {
        let mut sessions = vec![];

        // 8 normal sessions
        for i in 0..8 {
            sessions.push(create_test_session(
                &format!("normal_{}", i),
                1000 + (i * 10) as u64,
            ));
        }

        // Add outliers in reverse order
        sessions.push(create_test_session("warning", 2000));
        sessions.push(create_test_session("critical", 5000));

        let anomalies = detect_anomalies(&sessions);

        // Critical should come before warning
        let critical_count = anomalies
            .iter()
            .filter(|a| a.severity == AnomalySeverity::Critical)
            .count();

        if critical_count > 0 {
            assert_eq!(
                anomalies[0].severity,
                AnomalySeverity::Critical,
                "Critical anomalies should be sorted first"
            );
        }
    }

    #[test]
    fn test_deviation_percentage() {
        let mut sessions = vec![];

        // 10 sessions all at exactly 1000 tokens
        for i in 0..10 {
            sessions.push(create_test_session(&format!("s{}", i), 1000));
        }

        // Add extreme outlier at 10000 tokens (900% deviation from mean)
        sessions.push(create_test_session("outlier", 10000));

        let anomalies = detect_anomalies(&sessions);

        assert!(
            !anomalies.is_empty(),
            "Should detect anomaly with extreme outlier"
        );

        let outlier = &anomalies[0];
        // With 10 sessions at 1000 and 1 at 10000, mean = ~1818
        // Deviation from mean should be significant
        assert!(
            outlier.deviation_pct.abs() > 100.0,
            "Deviation should be >100% for extreme outlier, got {}%",
            outlier.deviation_pct
        );
    }

    // --- Daily cost spike tests ---

    fn create_session_on_date(id: &str, tokens: u64, days_ago: i64) -> Arc<SessionMetadata> {
        let ts = chrono::Utc::now() - chrono::Duration::days(days_ago);
        Arc::new(SessionMetadata {
            id: id.into(),
            source_tool: None,
            file_path: std::path::PathBuf::from(format!("/tmp/{}.jsonl", id)),
            project_path: "test".into(),
            first_timestamp: Some(ts),
            last_timestamp: Some(ts),
            message_count: 5,
            total_tokens: tokens,
            input_tokens: tokens / 2,
            output_tokens: tokens / 2,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec!["test-model".to_string()],
            model_segments: Vec::new(),
            file_size_bytes: 512,
            first_user_message: Some("test".to_string()),
            has_subagents: false,
            parent_session_id: None,
            duration_seconds: Some(30),
            branch: Some("main".to_string()),
            tool_usage: std::collections::HashMap::new(),
            tool_token_usage: std::collections::HashMap::new(),
        })
    }

    #[test]
    fn test_daily_spikes_insufficient_data() {
        // Only 3 days of data — below MIN_DAYS=7
        let sessions: Vec<_> = (0..3)
            .map(|i| create_session_on_date(&format!("s{}", i), 1000, i as i64))
            .collect();
        let spikes = detect_daily_cost_spikes(&sessions, 30);
        assert!(spikes.is_empty(), "Should return empty for <7 days data");
    }

    #[test]
    fn test_daily_spikes_detects_outlier_day() {
        // 8 normal days (~100K tokens each) + 1 spike day (~1M tokens)
        let mut sessions = vec![];
        for day in 1..=8i64 {
            for j in 0..3 {
                sessions.push(create_session_on_date(
                    &format!("normal_d{}_s{}", day, j),
                    33_000, // ~100K tokens/day total
                    day,
                ));
            }
        }
        // Spike: 1M tokens on day 0 (today)
        sessions.push(create_session_on_date("spike", 1_000_000, 0));

        let spikes = detect_daily_cost_spikes(&sessions, 30);
        assert!(!spikes.is_empty(), "Should detect the spike day");
        assert_eq!(spikes[0].severity, AnomalySeverity::Critical);
        assert!(spikes[0].ratio > 3.0, "ratio should be >3x average");
    }
}
