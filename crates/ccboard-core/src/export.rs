//! CSV/JSON export functionality for billing blocks and sessions
//!
//! Provides simple, testable export with proper error handling.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

use crate::models::{BillingBlockManager, SessionMetadata};

/// Export billing blocks to CSV format matching TUI table display
///
/// CSV columns: Date, Block (UTC), Tokens, Sessions, Cost
/// Rows sorted by date/time (most recent first)
///
/// # Arguments
/// * `manager` - Reference to BillingBlockManager
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if file creation or write operations fail
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::BillingBlockManager;
/// use ccboard_core::export::export_billing_blocks_to_csv;
/// use std::path::Path;
///
/// let manager = BillingBlockManager::new();
/// let path = Path::new("billing-blocks.csv");
/// export_billing_blocks_to_csv(&manager, &path).unwrap();
/// ```
pub fn export_billing_blocks_to_csv(manager: &BillingBlockManager, path: &Path) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(writer, "Date,Block (UTC),Tokens,Sessions,Cost")
        .context("Failed to write CSV header")?;

    // Get blocks sorted ascending, then reverse for descending (most recent first)
    let mut blocks = manager.get_all_blocks();
    blocks.reverse(); // Most recent first

    // Write data rows
    for (block, usage) in blocks {
        writeln!(
            writer,
            "\"{}\",\"{}\",{},{},\"${:.3}\"",
            block.date.format("%Y-%m-%d"), // "2026-02-03"
            block.label(),                 // "10:00-14:59"
            usage.total_tokens(),
            usage.session_count,
            usage.total_cost
        )
        .with_context(|| format!("Failed to write row for block {:?}", block))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

/// Export sessions to CSV format
///
/// CSV columns: Date, Time, Project, Session ID, Messages, Tokens, Models, Duration (min)
/// Rows sorted by date/time (most recent first in input)
///
/// # Arguments
/// * `sessions` - Slice of SessionMetadata to export
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if file creation or write operations fail
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::SessionMetadata;
/// use ccboard_core::export::export_sessions_to_csv;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// let sessions: Vec<Arc<SessionMetadata>> = vec![]; // Load sessions
/// let path = Path::new("sessions.csv");
/// export_sessions_to_csv(&sessions, &path).unwrap();
/// ```
pub fn export_sessions_to_csv(sessions: &[Arc<SessionMetadata>], path: &Path) -> Result<()> {
    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let file = File::create(path)
        .with_context(|| format!("Failed to create CSV file: {}", path.display()))?;

    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)"
    )
    .context("Failed to write CSV header")?;

    // Write data rows
    for session in sessions {
        let date = session
            .first_timestamp
            .map(|ts| ts.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let time = session
            .first_timestamp
            .map(|ts| ts.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let models = session.models_used.join(";");

        let duration = if let (Some(first), Some(last)) =
            (&session.first_timestamp, &session.last_timestamp)
        {
            let diff = last.signed_duration_since(*first);
            diff.num_minutes()
        } else {
            0
        };

        writeln!(
            writer,
            "\"{}\",\"{}\",\"{}\",\"{}\",{},{},\"{}\",{}",
            date,
            time,
            session.project_path,
            session.id,
            session.message_count,
            session.total_tokens,
            models,
            duration
        )
        .with_context(|| format!("Failed to write row for session {}", session.id))?;
    }

    writer.flush().context("Failed to flush CSV writer")?;

    Ok(())
}

/// Export sessions to JSON format
///
/// Pretty-printed JSON array of session metadata
///
/// # Arguments
/// * `sessions` - Slice of SessionMetadata to export
/// * `path` - Destination file path (created/overwritten)
///
/// # Errors
/// Returns error if serialization or file write fails
///
/// # Examples
///
/// ```no_run
/// use ccboard_core::models::SessionMetadata;
/// use ccboard_core::export::export_sessions_to_json;
/// use std::path::Path;
/// use std::sync::Arc;
///
/// let sessions: Vec<Arc<SessionMetadata>> = vec![]; // Load sessions
/// let path = Path::new("sessions.json");
/// export_sessions_to_json(&sessions, &path).unwrap();
/// ```
pub fn export_sessions_to_json(sessions: &[Arc<SessionMetadata>], path: &Path) -> Result<()> {
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Dereference Arc to get &SessionMetadata for serialization
    let sessions_ref: Vec<&SessionMetadata> = sessions.iter().map(|s| s.as_ref()).collect();

    // Serialize to JSON (pretty print)
    let json = serde_json::to_string_pretty(&sessions_ref)
        .context("Failed to serialize sessions to JSON")?;

    // Write to file
    std::fs::write(path, json)
        .with_context(|| format!("Failed to write JSON file: {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BillingBlockManager;
    use chrono::{TimeZone, Utc};
    use tempfile::TempDir;

    #[test]
    fn test_export_empty_manager() {
        let manager = BillingBlockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("test.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert_eq!(contents, "Date,Block (UTC),Tokens,Sessions,Cost\n");
    }

    #[test]
    fn test_export_with_data() {
        let mut manager = BillingBlockManager::new();

        // Add sample data (2 blocks)
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 3, 14, 30, 0).unwrap();
        manager.add_usage(&ts1, 5000, 1500, 200, 100, 0.015);

        let ts2 = Utc.with_ymd_and_hms(2026, 2, 3, 20, 15, 0).unwrap();
        manager.add_usage(&ts2, 3000, 1000, 100, 50, 0.010);

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("billing.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 3); // Header + 2 blocks
        assert_eq!(lines[0], "Date,Block (UTC),Tokens,Sessions,Cost");
        assert!(lines[1].contains("2026-02-03"));
        assert!(lines[1].contains("20:00-23:59")); // Later block first (reversed)
        assert!(lines[2].contains("10:00-14:59"));
    }

    #[test]
    fn test_creates_parent_directory() {
        let manager = BillingBlockManager::new();
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("exports/nested/test.csv");

        export_billing_blocks_to_csv(&manager, &nested_path).unwrap();

        assert!(nested_path.exists());
    }

    #[test]
    fn test_cost_formatting() {
        let mut manager = BillingBlockManager::new();

        // Test various cost values to verify 3 decimal places
        let ts = Utc.with_ymd_and_hms(2026, 2, 3, 10, 0, 0).unwrap();
        manager.add_usage(&ts, 1000, 500, 50, 25, 1.23456); // Should be $1.235

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("cost.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert!(contents.contains("\"$1.235\""));
    }

    #[test]
    fn test_multiple_dates_sorted() {
        let mut manager = BillingBlockManager::new();

        // Add blocks for multiple dates
        let ts1 = Utc.with_ymd_and_hms(2026, 2, 1, 10, 0, 0).unwrap();
        manager.add_usage(&ts1, 1000, 500, 0, 0, 0.5);

        let ts2 = Utc.with_ymd_and_hms(2026, 2, 3, 5, 0, 0).unwrap();
        manager.add_usage(&ts2, 2000, 1000, 0, 0, 1.0);

        let ts3 = Utc.with_ymd_and_hms(2026, 2, 2, 15, 0, 0).unwrap();
        manager.add_usage(&ts3, 1500, 750, 0, 0, 0.75);

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sorted.csv");

        export_billing_blocks_to_csv(&manager, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 4); // Header + 3 blocks
        // Most recent first
        assert!(lines[1].contains("2026-02-03")); // Feb 3
        assert!(lines[2].contains("2026-02-02")); // Feb 2
        assert!(lines[3].contains("2026-02-01")); // Feb 1
    }

    // Session export tests
    use crate::models::SessionMetadata;
    use std::path::PathBuf;

    fn create_test_session(id: &str, project: &str, messages: u64, tokens: u64) -> SessionMetadata {
        let timestamp = Utc.with_ymd_and_hms(2026, 2, 3, 14, 30, 0).unwrap();
        SessionMetadata {
            id: id.into(),
            file_path: PathBuf::from(format!("/test/{}.jsonl", id)),
            project_path: project.into(),
            first_timestamp: Some(timestamp),
            last_timestamp: Some(timestamp + chrono::Duration::minutes(45)),
            message_count: messages,
            total_tokens: tokens,
            input_tokens: tokens / 2,
            output_tokens: tokens / 3,
            cache_creation_tokens: tokens / 10,
            cache_read_tokens: tokens - (tokens / 2 + tokens / 3 + tokens / 10),
            models_used: vec!["sonnet".to_string(), "opus".to_string()],
            file_size_bytes: 1024,
            first_user_message: Some("Test message".to_string()),
            has_subagents: false,
            duration_seconds: Some(2700), // 45 minutes
            branch: None,
            tool_usage: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_export_sessions_csv_empty() {
        let sessions: Vec<Arc<SessionMetadata>> = vec![];
        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sessions.csv");

        super::export_sessions_to_csv(&sessions, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        assert_eq!(
            contents,
            "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)\n"
        );
    }

    #[test]
    fn test_export_sessions_csv_with_data() {
        let sessions = vec![
            Arc::new(create_test_session(
                "abc123",
                "/Users/test/project1",
                25,
                15000,
            )),
            Arc::new(create_test_session(
                "def456",
                "/Users/test/project2",
                10,
                5000,
            )),
        ];

        let temp_dir = TempDir::new().unwrap();
        let csv_path = temp_dir.path().join("sessions.csv");

        super::export_sessions_to_csv(&sessions, &csv_path).unwrap();

        let contents = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = contents.lines().collect();

        assert_eq!(lines.len(), 3); // Header + 2 sessions
        assert_eq!(
            lines[0],
            "Date,Time,Project,Session ID,Messages,Tokens,Models,Duration (min)"
        );
        assert!(lines[1].contains("2026-02-03"));
        assert!(lines[1].contains("14:30:00"));
        assert!(lines[1].contains("abc123"));
        assert!(lines[1].contains("25"));
        assert!(lines[1].contains("15000"));
        assert!(lines[1].contains("sonnet;opus"));
        assert!(lines[1].contains("45")); // duration in minutes
    }

    #[test]
    fn test_export_sessions_json_empty() {
        let sessions: Vec<Arc<SessionMetadata>> = vec![];
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("sessions.json");

        super::export_sessions_to_json(&sessions, &json_path).unwrap();

        let contents = std::fs::read_to_string(&json_path).unwrap();
        assert_eq!(contents, "[]");
    }

    #[test]
    fn test_export_sessions_json_with_data() {
        let sessions = vec![Arc::new(create_test_session(
            "abc123",
            "/Users/test/project1",
            25,
            15000,
        ))];

        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("sessions.json");

        super::export_sessions_to_json(&sessions, &json_path).unwrap();

        let contents = std::fs::read_to_string(&json_path).unwrap();

        // Verify it's valid JSON
        let parsed: Vec<SessionMetadata> = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "abc123");
        assert_eq!(parsed[0].message_count, 25);
        assert_eq!(parsed[0].total_tokens, 15000);
    }

    #[test]
    fn test_export_sessions_creates_dirs() {
        let sessions = vec![Arc::new(create_test_session("test", "/test", 1, 100))];
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("exports/nested/sessions.csv");

        super::export_sessions_to_csv(&sessions, &nested_path).unwrap();

        assert!(nested_path.exists());
    }
}
