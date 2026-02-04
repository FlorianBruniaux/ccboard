//! Error types for ccboard-core
//!
//! Provides a comprehensive error hierarchy with thiserror for graceful degradation.

use std::path::PathBuf;
use thiserror::Error;

/// Core error type for ccboard operations
#[derive(Error, Debug)]
pub enum CoreError {
    // ===================
    // IO Errors
    // ===================
    #[error("Failed to read file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file: {path}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Directory not found: {path}")]
    DirectoryNotFound { path: PathBuf },

    #[error("Invalid path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    // ===================
    // Parse Errors
    // ===================
    #[error("Failed to parse JSON in {path}: {message}")]
    JsonParse {
        path: PathBuf,
        message: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("Failed to parse YAML in {path}: {message}")]
    YamlParse {
        path: PathBuf,
        message: String,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("Malformed JSONL line {line_number} in {path}: {message}")]
    JsonlParse {
        path: PathBuf,
        line_number: usize,
        message: String,
    },

    #[error("Invalid frontmatter in {path}: {message}")]
    FrontmatterParse { path: PathBuf, message: String },

    // ===================
    // Watch Errors
    // ===================
    #[error("File watcher error: {message}")]
    WatchError {
        message: String,
        #[source]
        source: Option<notify::Error>,
    },

    // ===================
    // Store Errors
    // ===================
    #[error("Data store not initialized")]
    StoreNotInitialized,

    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Lock acquisition timeout")]
    LockTimeout,

    // ===================
    // Config Errors
    // ===================
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Claude home directory not found")]
    ClaudeHomeNotFound,

    // ===================
    // Circuit Breaker
    // ===================
    #[error("Operation timed out after {timeout_secs}s: {operation}")]
    Timeout {
        operation: String,
        timeout_secs: u64,
    },

    #[error("Circuit breaker open for {operation}: {failures} consecutive failures")]
    CircuitBreakerOpen { operation: String, failures: u32 },
}

/// Severity level for errors during load
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Non-critical, can continue with degraded functionality
    Warning,
    /// Significant but not fatal
    Error,
    /// Cannot continue
    Fatal,
}

/// Individual error entry in load report
#[derive(Debug, Clone)]
pub struct LoadError {
    pub source: String,
    pub message: String,
    pub severity: ErrorSeverity,
    /// Actionable suggestion for user (optional)
    pub suggestion: Option<String>,
}

impl LoadError {
    pub fn warning(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            message: message.into(),
            severity: ErrorSeverity::Warning,
            suggestion: None,
        }
    }

    pub fn error(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
            suggestion: None,
        }
    }

    pub fn fatal(source: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            message: message.into(),
            severity: ErrorSeverity::Fatal,
            suggestion: None,
        }
    }

    /// Add an actionable suggestion to this error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Create user-friendly error from CoreError with context-aware suggestions
    pub fn from_core_error(source: impl Into<String>, error: &CoreError) -> Self {
        let source = source.into();
        let (message, suggestion) = match error {
            CoreError::FileNotFound { path } => (
                format!("File not found: {}", path.display()),
                Some(format!("Check if file exists: ls {}", path.display())),
            ),
            CoreError::FileRead { path, .. } => (
                format!("Cannot read file: {}", path.display()),
                Some(format!("Check permissions: chmod +r {}", path.display())),
            ),
            CoreError::DirectoryNotFound { path } => (
                format!("Directory not found: {}", path.display()),
                Some(format!("Create directory: mkdir -p {}", path.display())),
            ),
            CoreError::JsonParse { path, message, .. } => (
                format!("Invalid JSON in {}: {}", path.display(), message),
                Some("Validate JSON syntax with: jq . <file>".to_string()),
            ),
            CoreError::JsonlParse {
                path,
                line_number,
                message,
            } => (
                format!(
                    "Malformed JSONL line {} in {}: {}",
                    line_number,
                    path.display(),
                    message
                ),
                Some(format!(
                    "Inspect line: sed -n '{}p' {}",
                    line_number,
                    path.display()
                )),
            ),
            CoreError::ClaudeHomeNotFound => (
                "Claude home directory not found".to_string(),
                Some("Run 'claude' CLI at least once to initialize ~/.claude".to_string()),
            ),
            _ => (error.to_string(), None),
        };

        Self {
            source,
            message,
            severity: ErrorSeverity::Error,
            suggestion,
        }
    }
}

/// Report of errors encountered during data loading
///
/// Enables graceful degradation by tracking partial failures
/// instead of failing completely on any error.
#[derive(Debug, Default)]
pub struct LoadReport {
    pub errors: Vec<LoadError>,
    pub stats_loaded: bool,
    pub settings_loaded: bool,
    pub sessions_scanned: usize,
    pub sessions_failed: usize,
}

impl LoadReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: LoadError) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, source: impl Into<String>, message: impl Into<String>) {
        self.errors.push(LoadError::warning(source, message));
    }

    pub fn add_fatal(&mut self, source: impl Into<String>, message: impl Into<String>) {
        self.errors.push(LoadError::fatal(source, message));
    }

    /// Returns true if there are any fatal errors
    pub fn has_fatal_errors(&self) -> bool {
        self.errors
            .iter()
            .any(|e| e.severity == ErrorSeverity::Fatal)
    }

    /// Returns true if there are any errors (including warnings)
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Returns only warnings
    pub fn warnings(&self) -> impl Iterator<Item = &LoadError> {
        self.errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Warning)
    }

    /// Returns count by severity
    pub fn error_count(&self) -> (usize, usize, usize) {
        let warnings = self
            .errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Warning)
            .count();
        let errors = self
            .errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Error)
            .count();
        let fatal = self
            .errors
            .iter()
            .filter(|e| e.severity == ErrorSeverity::Fatal)
            .count();
        (warnings, errors, fatal)
    }

    /// Merge another report into this one
    pub fn merge(&mut self, other: LoadReport) {
        self.errors.extend(other.errors);
        self.stats_loaded = self.stats_loaded || other.stats_loaded;
        self.settings_loaded = self.settings_loaded || other.settings_loaded;
        self.sessions_scanned += other.sessions_scanned;
        self.sessions_failed += other.sessions_failed;
    }
}

/// Degraded state indicator for the data store
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegradedState {
    /// Everything loaded successfully
    Healthy,
    /// Some data missing but functional
    PartialData {
        missing: Vec<String>,
        reason: String,
    },
    /// Read-only mode due to errors
    ReadOnly { reason: String },
}

impl DegradedState {
    pub fn is_healthy(&self) -> bool {
        matches!(self, DegradedState::Healthy)
    }

    pub fn is_degraded(&self) -> bool {
        !self.is_healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_report_severity_counting() {
        let mut report = LoadReport::new();
        report.add_warning("stats", "File not found");
        report.add_error(LoadError::error("settings", "Parse error"));
        report.add_fatal("sessions", "Directory missing");

        let (warnings, errors, fatal) = report.error_count();
        assert_eq!(warnings, 1);
        assert_eq!(errors, 1);
        assert_eq!(fatal, 1);
        assert!(report.has_fatal_errors());
    }

    #[test]
    fn test_load_report_merge() {
        let mut report1 = LoadReport::new();
        report1.stats_loaded = true;
        report1.sessions_scanned = 10;

        let mut report2 = LoadReport::new();
        report2.settings_loaded = true;
        report2.sessions_scanned = 20;
        report2.add_warning("test", "warning");

        report1.merge(report2);

        assert!(report1.stats_loaded);
        assert!(report1.settings_loaded);
        assert_eq!(report1.sessions_scanned, 30);
        assert_eq!(report1.errors.len(), 1);
    }
}
