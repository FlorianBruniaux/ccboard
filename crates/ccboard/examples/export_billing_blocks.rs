//! Example: Export billing blocks to CSV
//!
//! This example demonstrates how to use the CSV export functionality
//! with real data from ~/.claude directory.
//!
//! Run with: cargo run --example export_billing_blocks

use ccboard_core::{DataStore, export_billing_blocks_to_csv};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize DataStore with real ~/.claude data
    let claude_home = dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".claude");

    let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    println!("Loading data from: {}", claude_home.display());
    println!("Project: {}", project_dir.display());

    let store = Arc::new(DataStore::with_defaults(
        claude_home.clone(),
        Some(project_dir),
    ));

    // Load initial data
    let report = store.initial_load().await;

    println!("\n=== Load Report ===");
    println!("Stats loaded: {}", report.stats_loaded);
    println!("Settings loaded: {}", report.settings_loaded);
    println!("Sessions scanned: {}", report.sessions_scanned);
    println!("Sessions failed: {}", report.sessions_failed);

    if report.has_fatal_errors() {
        eprintln!("\nFatal errors:");
        for error in report.errors.iter() {
            eprintln!("  - {}: {}", error.source, error.message);
        }
        std::process::exit(1);
    }

    // Compute billing blocks
    println!("\nComputing billing blocks...");
    store.compute_billing_blocks().await;

    let billing_blocks = store.billing_blocks();
    let all_blocks = billing_blocks.get_all_blocks();

    println!("Total billing blocks: {}", all_blocks.len());
    if !all_blocks.is_empty() {
        println!("\nFirst 5 blocks (sorted by date, most recent last):");
        for (block, usage) in all_blocks.iter().rev().take(5) {
            println!(
                "  {} {} - Tokens: {}, Sessions: {}, Cost: ${:.3}",
                block.date.format("%Y-%m-%d"),
                block.label(),
                usage.total_tokens(),
                usage.session_count,
                usage.total_cost
            );
        }
    }

    // Export to CSV
    let export_path = claude_home.join("exports/billing-blocks-test.csv");
    println!("\nExporting to: {}", export_path.display());

    export_billing_blocks_to_csv(&billing_blocks, &export_path).expect("Failed to export CSV");

    println!("✓ Export successful!");

    // Display CSV content (first 10 lines)
    println!("\n=== CSV Content (first 10 lines) ===");
    let content = std::fs::read_to_string(&export_path).expect("Failed to read CSV");
    for (i, line) in content.lines().take(10).enumerate() {
        println!("{}: {}", i + 1, line);
    }

    if content.lines().count() > 10 {
        println!("... ({} more lines)", content.lines().count() - 10);
    }

    println!("\n✓ Test completed successfully!");
    println!("CSV file available at: {}", export_path.display());
}
