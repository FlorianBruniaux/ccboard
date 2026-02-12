//! CLI commands for session management
//!
//! Provides search, recent, info, and resume commands using DataStore directly.

use anyhow::{Context, Result};
use ccboard_core::models::SessionMetadata;
use chrono::{DateTime, Utc};
use comfy_table::{Cell, Color, ContentArrangement, Row, Table};
use std::sync::Arc;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug)]
pub enum CliError {
    NoResults {
        query: String,
        scanned: usize,
    },
    AmbiguousId {
        prefix: String,
        count: usize,
        suggestions: String,
    },
    Core(ccboard_core::error::CoreError),
    Other(anyhow::Error),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::NoResults { query, scanned } => {
                write!(
                    f,
                    "No sessions match '{}' ({} sessions scanned)",
                    query, scanned
                )
            }
            CliError::AmbiguousId {
                prefix,
                count,
                suggestions,
            } => {
                write!(
                    f,
                    "Ambiguous ID prefix '{}': matches {} sessions\n{}",
                    prefix, count, suggestions
                )
            }
            CliError::Core(e) => write!(f, "{}", e),
            CliError::Other(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CliError {}

impl From<ccboard_core::error::CoreError> for CliError {
    fn from(e: ccboard_core::error::CoreError) -> Self {
        CliError::Core(e)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::Other(e)
    }
}

// ============================================================================
// Date Filter
// ============================================================================

/// Date filter for queries
pub enum DateFilter {
    Days(u32),
    Months(u32),
    Years(u32),
    Since(DateTime<Utc>),
}

impl DateFilter {
    /// Parse from string: "7d", "30d", "3m", "1y", "YYYY-MM-DD"
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(stripped) = s.strip_suffix('d') {
            let days = stripped
                .parse::<u32>()
                .context("Invalid days format (expected: 7d)")?;
            return Ok(DateFilter::Days(days));
        }

        if let Some(stripped) = s.strip_suffix('m') {
            let months = stripped
                .parse::<u32>()
                .context("Invalid months format (expected: 3m)")?;
            return Ok(DateFilter::Months(months));
        }

        if let Some(stripped) = s.strip_suffix('y') {
            let years = stripped
                .parse::<u32>()
                .context("Invalid years format (expected: 1y)")?;
            return Ok(DateFilter::Years(years));
        }

        // Try parsing as date
        let date =
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").context("Invalid date format")?;
        let datetime = date.and_hms_opt(0, 0, 0).context("Invalid time")?.and_utc();
        Ok(DateFilter::Since(datetime))
    }

    /// Get cutoff timestamp
    pub fn cutoff(&self) -> DateTime<Utc> {
        let now = Utc::now();
        match self {
            DateFilter::Days(d) => now - chrono::Duration::days(*d as i64),
            DateFilter::Months(m) => now - chrono::Duration::days((*m as i64) * 30),
            DateFilter::Years(y) => now - chrono::Duration::days((*y as i64) * 365),
            DateFilter::Since(dt) => *dt,
        }
    }

    /// Check if timestamp matches filter
    pub fn matches(&self, timestamp: &DateTime<Utc>) -> bool {
        *timestamp >= self.cutoff()
    }
}

// ============================================================================
// Query Helpers
// ============================================================================

/// Find session by exact ID or unique prefix
pub fn find_by_id_or_prefix(
    sessions: &[Arc<SessionMetadata>],
    id: &str,
) -> Result<Arc<SessionMetadata>, CliError> {
    // Try exact match first
    if let Some(session) = sessions.iter().find(|s| s.id == id) {
        return Ok(Arc::clone(session));
    }

    // Try prefix match (minimum 8 chars for safety)
    if id.len() < 8 {
        return Err(CliError::NoResults {
            query: id.to_string(),
            scanned: sessions.len(),
        });
    }

    let matches: Vec<_> = sessions.iter().filter(|s| s.id.starts_with(id)).collect();

    match matches.len() {
        0 => Err(CliError::NoResults {
            query: id.to_string(),
            scanned: sessions.len(),
        }),
        1 => Ok(Arc::clone(matches[0])),
        count => {
            let suggestions = matches
                .iter()
                .take(5)
                .map(|s| format!("  - {}", &s.id[..16.min(s.id.len())]))
                .collect::<Vec<_>>()
                .join("\n");
            Err(CliError::AmbiguousId {
                prefix: id.to_string(),
                count,
                suggestions,
            })
        }
    }
}

/// Search sessions by query string
pub fn search_sessions(
    sessions: &[Arc<SessionMetadata>],
    query: &str,
    date_filter: Option<&DateFilter>,
    limit: usize,
) -> Vec<Arc<SessionMetadata>> {
    let query_lower = query.to_lowercase();

    sessions
        .iter()
        .filter(|s| {
            // Date filter
            if let Some(filter) = date_filter {
                if let Some(ts) = s.first_timestamp {
                    if !filter.matches(&ts) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Text search: ID, project path, first message
            s.id.to_lowercase().contains(&query_lower)
                || s.project_path.to_lowercase().contains(&query_lower)
                || s.first_user_message
                    .as_ref()
                    .map(|m| m.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
                || s.branch
                    .as_ref()
                    .map(|b| b.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
        })
        .take(limit)
        .cloned()
        .collect()
}

// ============================================================================
// Formatters
// ============================================================================

/// Format sessions as table (human) or JSON
pub fn format_session_table(
    sessions: &[Arc<SessionMetadata>],
    json: bool,
    no_color: bool,
) -> String {
    if json {
        return serde_json::to_string_pretty(sessions).unwrap_or_else(|_| "[]".to_string());
    }

    if sessions.is_empty() {
        return "No sessions found.".to_string();
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    // Apply colors only if enabled
    if no_color {
        table.set_header(vec![
            "ID", "Project", "Branch", "Date", "Msgs", "Tokens", "Duration", "Preview",
        ]);
    } else {
        table.set_header(vec![
            Cell::new("ID").fg(Color::Cyan),
            Cell::new("Project").fg(Color::Cyan),
            Cell::new("Branch").fg(Color::Cyan),
            Cell::new("Date").fg(Color::Cyan),
            Cell::new("Msgs").fg(Color::Cyan),
            Cell::new("Tokens").fg(Color::Cyan),
            Cell::new("Duration").fg(Color::Cyan),
            Cell::new("Preview").fg(Color::Cyan),
        ]);
    }

    for session in sessions {
        let id_short = &session.id[..8.min(session.id.len())];
        let project = shorten_project(&session.project_path);
        let branch = session
            .branch
            .as_ref()
            .map(|b| truncate(b, 15))
            .unwrap_or_else(|| "-".to_string());
        let date = session
            .first_timestamp
            .map(|ts| ts.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let msgs = session.message_count.to_string();
        let tokens = format_tokens(session.total_tokens);
        let duration = session.duration_display();
        let preview = session
            .first_user_message
            .as_ref()
            .map(|m| truncate(m, 40))
            .unwrap_or_else(|| "".to_string());

        table.add_row(Row::from(vec![
            id_short, &project, &branch, &date, &msgs, &tokens, &duration, &preview,
        ]));
    }

    table.to_string()
}

/// Format single session info (human or JSON)
pub fn format_session_info(session: &SessionMetadata, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(session).unwrap_or_else(|_| "{}".to_string());
    }

    let mut lines = vec![];
    lines.push(format!("Session ID:       {}", session.id));
    lines.push(format!("Project:          {}", session.project_path));
    lines.push(format!(
        "Branch:           {}",
        session.branch.as_deref().unwrap_or("-")
    ));
    lines.push(format!("File:             {}", session.file_path.display()));
    lines.push(format!(
        "First timestamp:  {}",
        session
            .first_timestamp
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!(
        "Last timestamp:   {}",
        session
            .last_timestamp
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "-".to_string())
    ));
    lines.push(format!("Messages:         {}", session.message_count));
    lines.push(format!(
        "Total tokens:     {}",
        format_tokens(session.total_tokens)
    ));
    lines.push(format!(
        "  Input:          {}",
        format_tokens(session.input_tokens)
    ));
    lines.push(format!(
        "  Output:         {}",
        format_tokens(session.output_tokens)
    ));
    lines.push(format!(
        "  Cache creation: {}",
        format_tokens(session.cache_creation_tokens)
    ));
    lines.push(format!(
        "  Cache read:     {}",
        format_tokens(session.cache_read_tokens)
    ));
    lines.push(format!(
        "Models:           {}",
        session.models_used.join(", ")
    ));
    lines.push(format!("File size:        {}", session.size_display()));
    lines.push(format!("Duration:         {}", session.duration_display()));
    lines.push(format!("Has subagents:    {}", session.has_subagents));
    lines.push(format!(
        "First message:    {}",
        session.first_user_message.as_deref().unwrap_or("-")
    ));

    lines.join("\n")
}

// ============================================================================
// Utilities
// ============================================================================

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn truncate(s: &str, max: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        // Use char-based truncation to avoid panicking on multi-byte characters (emojis)
        s.chars().take(max - 1).collect::<String>() + "…"
    }
}

fn shorten_project(path: &str) -> String {
    // Extract last 2 path components
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() > 2 {
        format!("…/{}", parts[parts.len() - 2..].join("/"))
    } else {
        path.to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_date_filter_parse_days() {
        let filter = DateFilter::parse("7d").unwrap();
        if let DateFilter::Days(d) = filter {
            assert_eq!(d, 7);
        } else {
            panic!("Expected Days variant");
        }
    }

    #[test]
    fn test_date_filter_parse_months() {
        let filter = DateFilter::parse("3m").unwrap();
        if let DateFilter::Months(m) = filter {
            assert_eq!(m, 3);
        } else {
            panic!("Expected Months variant");
        }
    }

    #[test]
    fn test_date_filter_parse_years() {
        let filter = DateFilter::parse("1y").unwrap();
        if let DateFilter::Years(y) = filter {
            assert_eq!(y, 1);
        } else {
            panic!("Expected Years variant");
        }
    }

    #[test]
    fn test_date_filter_parse_date() {
        let filter = DateFilter::parse("2025-06-15").unwrap();
        if let DateFilter::Since(dt) = filter {
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2025-06-15");
        } else {
            panic!("Expected Since variant");
        }
    }

    #[test]
    fn test_date_filter_parse_invalid() {
        assert!(DateFilter::parse("invalid").is_err());
    }

    #[test]
    fn test_truncate_ascii() {
        assert_eq!(truncate("hello world", 20), "hello world");
        assert_eq!(truncate("hello world", 5), "hell…");
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_emoji() {
        // Emoji = 4 bytes but 1 char
        assert_eq!(truncate("🔍 test", 10), "🔍 test");
        assert_eq!(truncate("🔍 test", 4), "🔍 t…");
        assert_eq!(truncate("hello 🔍 world", 8), "hello 🔍…");
    }

    #[test]
    fn test_truncate_multi_emoji() {
        // Multiple emojis
        assert_eq!(truncate("🚀🔍💡", 5), "🚀🔍💡");
        assert_eq!(truncate("🚀🔍💡test", 4), "🚀🔍💡…");
        assert_eq!(truncate("test🚀🔍💡", 5), "test…");
    }

    #[test]
    fn test_truncate_unicode() {
        // Various unicode characters
        assert_eq!(truncate("café", 10), "café");
        assert_eq!(truncate("café", 3), "ca…");
        assert_eq!(truncate("日本語", 5), "日本語");
        assert_eq!(truncate("日本語テスト", 4), "日本語…");
    }

    #[test]
    fn test_date_filter_cutoff_approximate() {
        let filter = DateFilter::Days(7);
        let cutoff = filter.cutoff();
        let now = Utc::now();
        let diff = (now - cutoff).num_days();
        assert!((diff - 7).abs() <= 1); // Allow 1 day variance for test timing
    }

    #[test]
    fn test_date_filter_matches() {
        let filter = DateFilter::Days(7);
        let recent = Utc::now() - chrono::Duration::days(3);
        let old = Utc::now() - chrono::Duration::days(10);

        assert!(filter.matches(&recent));
        assert!(!filter.matches(&old));
    }

    fn create_test_session(id: &str) -> Arc<SessionMetadata> {
        Arc::new(SessionMetadata {
            id: id.into(),
            file_path: std::path::PathBuf::from(format!("/{}.jsonl", id)),
            project_path: "/test".into(),
            first_timestamp: Some(Utc::now()),
            last_timestamp: Some(Utc::now()),
            message_count: 10,
            total_tokens: 1000,
            input_tokens: 500,
            output_tokens: 500,
            cache_creation_tokens: 0,
            cache_read_tokens: 0,
            models_used: vec!["sonnet".to_string()],
            file_size_bytes: 1024,
            first_user_message: Some("Test message".to_string()),
            has_subagents: false,
            duration_seconds: Some(1800),
            branch: Some("main".to_string()),
            tool_usage: std::collections::HashMap::new(),
        })
    }

    #[test]
    fn test_find_by_id_exact_match() {
        let sessions = vec![
            create_test_session("abc123def456"),
            create_test_session("xyz789ghi012"),
        ];

        let result = find_by_id_or_prefix(&sessions, "abc123def456").unwrap();
        assert_eq!(result.id, "abc123def456");
    }

    #[test]
    fn test_find_by_id_prefix_match() {
        let sessions = vec![
            create_test_session("abc123def456"),
            create_test_session("xyz789ghi012"),
        ];

        let result = find_by_id_or_prefix(&sessions, "abc123de").unwrap();
        assert_eq!(result.id, "abc123def456");
    }

    #[test]
    fn test_find_by_id_ambiguous() {
        let sessions = vec![
            create_test_session("abc123de"),
            create_test_session("abc123dx"),
        ];

        let result = find_by_id_or_prefix(&sessions, "abc123d");
        // Should return NoResults because prefix < 8 chars
        assert!(matches!(result, Err(CliError::NoResults { .. })));
    }

    #[test]
    fn test_find_by_id_not_found() {
        let sessions = vec![create_test_session("abc123def456")];

        let result = find_by_id_or_prefix(&sessions, "notfound");
        assert!(matches!(result, Err(CliError::NoResults { .. })));
    }

    #[test]
    fn test_format_session_table_empty() {
        let sessions: Vec<Arc<SessionMetadata>> = vec![];
        let output = format_session_table(&sessions, false, false);
        assert!(output.contains("No sessions found"));
    }

    #[test]
    fn test_format_session_table_json() {
        let sessions = vec![create_test_session("abc123def456")];
        let output = format_session_table(&sessions, true, false);
        assert!(output.contains("abc123def456"));
        assert!(output.starts_with('['));
    }

    #[test]
    fn test_format_session_info_json() {
        let session = create_test_session("abc123def456");
        let output = format_session_info(&session, true);
        assert!(output.contains("abc123def456"));
        assert!(output.starts_with('{'));
    }
}
