//! CSV export functionality for billing blocks
//!
//! Provides simple, testable CSV export with proper error handling.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::models::BillingBlockManager;

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
}
