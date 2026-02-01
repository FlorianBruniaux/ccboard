//! Stats cache parser with retry on parse failure

use crate::error::{CoreError, LoadError, LoadReport};
use crate::models::StatsCache;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Parser for stats-cache.json
pub struct StatsParser {
    /// Maximum retry attempts
    max_retries: u32,
    /// Delay between retries
    retry_delay: Duration,
}

impl Default for StatsParser {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
        }
    }
}

impl StatsParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_retries(mut self, max_retries: u32, retry_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = retry_delay;
        self
    }

    /// Parse stats-cache.json with retry logic
    ///
    /// Retries on parse failure as the file might be mid-write by Claude Code.
    pub async fn parse(&self, path: &Path) -> Result<StatsCache, CoreError> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                debug!(attempt, "Retrying stats parse after delay");
                sleep(self.retry_delay).await;
            }

            match self.try_parse(path).await {
                Ok(stats) => return Ok(stats),
                Err(e) => {
                    warn!(attempt, error = %e, "Stats parse attempt failed");
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| CoreError::FileNotFound {
            path: path.to_path_buf(),
        }))
    }

    /// Single parse attempt
    async fn try_parse(&self, path: &Path) -> Result<StatsCache, CoreError> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                CoreError::FileRead {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })?;

        serde_json::from_str(&content).map_err(|e| CoreError::JsonParse {
            path: path.to_path_buf(),
            message: e.to_string(),
            source: e,
        })
    }

    /// Parse with graceful degradation, recording errors in LoadReport
    pub async fn parse_graceful(&self, path: &Path, report: &mut LoadReport) -> Option<StatsCache> {
        match self.parse(path).await {
            Ok(stats) => {
                report.stats_loaded = true;
                Some(stats)
            }
            Err(CoreError::FileNotFound { .. }) => {
                report.add_warning("stats", format!("Stats file not found: {}", path.display()));
                None
            }
            Err(e) => {
                report.add_error(LoadError::error("stats", e.to_string()));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_parse_valid_stats() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "version": 2,
            "totalSessions": 5,
            "totalMessages": 20,
            "modelUsage": {{
                "test-model": {{
                    "inputTokens": 600,
                    "outputTokens": 400
                }}
            }}
        }}"#
        )
        .unwrap();

        let parser = StatsParser::new();
        let stats = parser.parse(file.path()).await.unwrap();

        assert_eq!(stats.total_tokens(), 1000);
        assert_eq!(stats.total_input_tokens(), 600);
        assert_eq!(stats.session_count(), 5);
    }

    #[tokio::test]
    async fn test_parse_missing_file() {
        let parser = StatsParser::new();
        let result = parser.parse(Path::new("/nonexistent/stats.json")).await;

        assert!(matches!(result, Err(CoreError::FileNotFound { .. })));
    }

    #[tokio::test]
    async fn test_parse_invalid_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "not valid json").unwrap();

        let parser = StatsParser::new().with_retries(1, Duration::from_millis(10));
        let result = parser.parse(file.path()).await;

        assert!(matches!(result, Err(CoreError::JsonParse { .. })));
    }

    #[tokio::test]
    async fn test_parse_graceful_records_errors() {
        let parser = StatsParser::new();
        let mut report = LoadReport::new();

        let result = parser
            .parse_graceful(Path::new("/nonexistent/stats.json"), &mut report)
            .await;

        assert!(result.is_none());
        assert!(!report.stats_loaded);
        assert!(report.has_errors());
    }
}
