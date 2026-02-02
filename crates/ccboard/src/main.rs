//! ccboard - Unified Claude Code Management Dashboard

use anyhow::{Context, Result};
use ccboard_core::DataStore;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "ccboard",
    version,
    about = "Unified Claude Code Management Dashboard",
    long_about = "A comprehensive TUI and web dashboard for managing Claude Code data.\n\
                  \n\
                  Visualizes sessions, statistics, configuration, hooks, agents, costs, and history\n\
                  from ~/.claude directories with real-time updates and file editing capabilities.\n\
                  \n\
                  Features:\n\
                    • 7 interactive tabs (Dashboard, Sessions, Config, Hooks, Agents, Costs, History)\n\
                    • File editing with $EDITOR integration (press 'e')\n\
                    • MCP server management and visualization\n\
                    • Real-time cost tracking and analytics\n\
                    • Session search and exploration\n\
                  \n\
                  Examples:\n\
                    ccboard                          # Run TUI (default)\n\
                    ccboard web --port 8080          # Run web interface on port 8080\n\
                    ccboard stats                    # Print stats summary\n\
                    ccboard --project ~/myproject    # Focus on specific project"
)]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,

    /// Path to Claude home directory (default: ~/.claude)
    #[arg(long)]
    claude_home: Option<PathBuf>,

    /// Focus on specific project directory
    #[arg(long)]
    project: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Mode {
    /// Run TUI interface (default)
    Tui,
    /// Run web interface
    Web {
        /// Port for web server
        #[arg(long, default_value = "3333")]
        port: u16,
    },
    /// Run both TUI and web interfaces
    Both {
        /// Port for web server
        #[arg(long, default_value = "3333")]
        port: u16,
    },
    /// Print stats to terminal and exit
    Stats,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let claude_home = cli
        .claude_home
        .or_else(|| dirs::home_dir().map(|h: PathBuf| h.join(".claude")))
        .context("Could not determine Claude home directory")?;

    match cli.mode.unwrap_or(Mode::Tui) {
        Mode::Tui => {
            run_tui(claude_home, cli.project).await?;
        }
        Mode::Web { port } => {
            run_web(claude_home, cli.project, port).await?;
        }
        Mode::Both { port } => {
            run_both(claude_home, cli.project, port).await?;
        }
        Mode::Stats => {
            run_stats(claude_home, cli.project).await?;
        }
    }

    Ok(())
}

async fn run_tui(claude_home: PathBuf, project: Option<PathBuf>) -> Result<()> {
    // Initialize data store
    let store = Arc::new(DataStore::with_defaults(
        claude_home.clone(),
        project.clone(),
    ));

    // Load initial data
    let report = store.initial_load().await;

    if report.has_fatal_errors() {
        eprintln!("Fatal errors during data load:");
        for error in report.errors.iter() {
            eprintln!("  - {}: {}", error.source, error.message);
        }
        return Ok(());
    }

    // Compute invocation statistics (agents/commands/skills usage)
    store.compute_invocations().await;

    // Start file watcher for live updates
    let _watcher = ccboard_core::FileWatcher::start(
        claude_home.clone(),
        project.clone(),
        Arc::clone(&store),
        Default::default(),
    )
    .await
    .context("Failed to start file watcher")?;

    // Run TUI
    ccboard_tui::run(store, claude_home, project).await
}

async fn run_web(claude_home: PathBuf, project: Option<PathBuf>, port: u16) -> Result<()> {
    // Initialize data store
    let store = Arc::new(DataStore::with_defaults(
        claude_home.clone(),
        project.clone(),
    ));

    // Load initial data
    let report = store.initial_load().await;

    if report.has_fatal_errors() {
        eprintln!("Fatal errors during data load:");
        for error in report.errors.iter() {
            eprintln!("  - {}: {}", error.source, error.message);
        }
        return Ok(());
    }

    // Compute invocation statistics (agents/commands/skills usage)
    store.compute_invocations().await;

    // Start file watcher for live updates
    let _watcher = ccboard_core::FileWatcher::start(
        claude_home,
        project,
        Arc::clone(&store),
        Default::default(),
    )
    .await
    .context("Failed to start file watcher")?;

    // Run web server
    ccboard_web::run(store, port).await
}

async fn run_both(claude_home: PathBuf, project: Option<PathBuf>, port: u16) -> Result<()> {
    // Initialize data store
    let store = Arc::new(DataStore::with_defaults(
        claude_home.clone(),
        project.clone(),
    ));

    // Load initial data
    let report = store.initial_load().await;

    if report.has_fatal_errors() {
        eprintln!("Fatal errors during data load:");
        for error in report.errors.iter() {
            eprintln!("  - {}: {}", error.source, error.message);
        }
        return Ok(());
    }

    // Compute invocation statistics (agents/commands/skills usage)
    store.compute_invocations().await;

    // Start file watcher for live updates (shared by TUI and web)
    let _watcher = ccboard_core::FileWatcher::start(
        claude_home.clone(),
        project.clone(),
        Arc::clone(&store),
        Default::default(),
    )
    .await
    .context("Failed to start file watcher")?;

    // Start web server in background
    let web_store = Arc::clone(&store);
    let web_handle = tokio::spawn(async move {
        if let Err(e) = ccboard_web::run(web_store, port).await {
            eprintln!("Web server error: {}", e);
        }
    });

    // Run TUI in foreground
    let tui_result = ccboard_tui::run(store, claude_home, project).await;

    // Clean up web server
    web_handle.abort();

    tui_result
}

async fn run_stats(claude_home: PathBuf, project: Option<PathBuf>) -> Result<()> {
    // Initialize data store
    let store = DataStore::with_defaults(claude_home, project);

    // Load initial data
    let report = store.initial_load().await;

    // Print stats summary
    println!("ccboard - Claude Code Statistics");
    println!("================================");
    println!();

    if let Some(stats) = store.stats() {
        println!("Total Tokens:     {}", format_number(stats.total_tokens()));
        println!(
            "  Input:          {}",
            format_number(stats.total_input_tokens())
        );
        println!(
            "  Output:         {}",
            format_number(stats.total_output_tokens())
        );
        println!(
            "  Cache Read:     {}",
            format_number(stats.total_cache_read_tokens())
        );
        println!(
            "  Cache Write:    {}",
            format_number(stats.total_cache_write_tokens())
        );
        println!();
        println!("Sessions:         {}", stats.session_count());
        println!("Messages:         {}", stats.message_count());
        println!("Cache Hit Ratio:  {:.1}%", stats.cache_ratio() * 100.0);
        println!();

        if !stats.model_usage.is_empty() {
            println!("Models:");
            for (name, usage) in stats.top_models(5) {
                println!(
                    "  {}: {} tokens (in: {}, out: {})",
                    name,
                    format_number(usage.total_tokens()),
                    format_number(usage.input_tokens),
                    format_number(usage.output_tokens)
                );
            }
        }
    } else {
        println!("No stats available");
    }

    println!();
    println!("Sessions indexed: {}", store.session_count());

    if report.has_errors() {
        println!();
        println!("Warnings:");
        for error in report.warnings() {
            println!("  - {}: {}", error.source, error.message);
        }
    }

    Ok(())
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
