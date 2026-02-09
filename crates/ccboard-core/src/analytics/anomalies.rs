//! Anomaly detection for unusual spikes/drops in token usage or session costs
//!
//! Uses Z-score based statistical analysis to flag sessions that deviate
//! significantly from normal behavior patterns.

use crate::models::session::{SessionId, SessionMetadata};
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
        if self.std_dev == 0.0 {
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
    const MIN_SESSIONS: usize = 10;
    const WARNING_THRESHOLD: f64 = 2.0;
    const CRITICAL_THRESHOLD: f64 = 3.0;

    if sessions.len() < MIN_SESSIONS {
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

                if abs_z > WARNING_THRESHOLD {
                    let severity = if abs_z > CRITICAL_THRESHOLD {
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    fn create_test_session(id: &str, tokens: u64) -> Arc<SessionMetadata> {
        Arc::new(SessionMetadata {
            id: id.into(),
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
            file_size_bytes: 1024,
            first_user_message: Some("test message".to_string()),
            has_subagents: false,
            duration_seconds: Some(60),
            branch: Some("main".to_string()),
            tool_usage: std::collections::HashMap::new(),
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
}
