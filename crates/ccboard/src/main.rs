//! ccboard - Unified Claude Code Management Dashboard

mod cli;

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
    /// Clear session metadata cache and exit
    ClearCache,
    /// Search sessions by query
    Search {
        /// Query string (searches ID, project, message, branch)
        query: String,
        /// Date filter: 7d, 30d, 3m, 1y, YYYY-MM-DD
        #[arg(long, short = 'd')]
        since: Option<String>,
        /// Max results
        #[arg(long, short = 'n', default_value = "20")]
        limit: usize,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show recent sessions
    Recent {
        /// Number of sessions
        #[arg(default_value = "10")]
        count: usize,
        /// Date filter: 7d, 30d, 3m, 1y, YYYY-MM-DD
        #[arg(long, short = 'd')]
        since: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show detailed session info
    Info {
        /// Session ID or prefix (min 8 chars)
        session_id: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Resume session in Claude CLI
    Resume {
        /// Session ID or prefix (min 8 chars)
        session_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let claude_home = cli
        .claude_home
        .or_else(|| dirs::home_dir().map(|h: PathBuf| h.join(".claude")))
        .context("Could not determine Claude home directory")?;

    // Auto-detect project: if no --project specified, try current directory
    let project = cli.project.or_else(|| {
        let current_dir = std::env::current_dir().ok()?;
        // Check if current directory has a .claude/ subdirectory
        if current_dir.join(".claude").exists() {
            Some(current_dir)
        } else {
            None
        }
    });

    match cli.mode.unwrap_or(Mode::Tui) {
        Mode::Tui => {
            run_tui(claude_home, project).await?;
        }
        Mode::Web { port } => {
            run_web(claude_home, project, port).await?;
        }
        Mode::Both { port } => {
            run_both(claude_home, project, port).await?;
        }
        Mode::Stats => {
            run_stats(claude_home, project).await?;
        }
        Mode::ClearCache => {
            run_clear_cache(claude_home).await?;
        }
        Mode::Search {
            query,
            since,
            limit,
            json,
        } => {
            run_search(claude_home, project, query, since, limit, json).await?;
        }
        Mode::Recent { count, since, json } => {
            run_recent(claude_home, project, count, since, json).await?;
        }
        Mode::Info { session_id, json } => {
            run_info(claude_home, project, session_id, json).await?;
        }
        Mode::Resume { session_id } => {
            run_resume(claude_home, project, session_id).await?;
        }
    }

    Ok(())
}

async fn run_tui(claude_home: PathBuf, project: Option<PathBuf>) -> Result<()> {
    // Initialize data store (without loading data yet - TUI will handle that)
    let store = Arc::new(DataStore::with_defaults(
        claude_home.clone(),
        project.clone(),
    ));

    // Start file watcher for live updates
    let _watcher = ccboard_core::FileWatcher::start(
        claude_home.clone(),
        project.clone(),
        Arc::clone(&store),
        Default::default(),
    )
    .await
    .context("Failed to start file watcher")?;

    // Run TUI (will show loading spinner and load data in background)
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

    // Compute billing blocks (5h usage tracking)
    store.compute_billing_blocks().await;

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

    // Compute billing blocks (5h usage tracking)
    store.compute_billing_blocks().await;

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

async fn run_clear_cache(claude_home: PathBuf) -> Result<()> {
    let cache_dir = claude_home.join("cache");
    let cache_path = cache_dir.join("session-metadata.db");

    if !cache_path.exists() {
        println!("❌ Cache not found at: {}", cache_path.display());
        println!("   Nothing to clear.");
        return Ok(());
    }

    // Get file size before deletion
    let size_bytes = std::fs::metadata(&cache_path)
        .with_context(|| format!("Failed to read cache metadata: {}", cache_path.display()))?
        .len();

    // Delete cache file
    std::fs::remove_file(&cache_path)
        .with_context(|| format!("Failed to delete cache: {}", cache_path.display()))?;

    // Delete WAL files if they exist
    let wal_path = cache_dir.join("session-metadata.db-wal");
    let shm_path = cache_dir.join("session-metadata.db-shm");

    if wal_path.exists() {
        let _ = std::fs::remove_file(&wal_path);
    }
    if shm_path.exists() {
        let _ = std::fs::remove_file(&shm_path);
    }

    println!("✅ Cache cleared successfully");
    println!("   Location: {}", cache_path.display());
    println!("   Freed: {}", format_size(size_bytes));
    println!();
    println!("💡 Next run will rebuild cache with fresh metadata.");

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1}KB", bytes as f64 / 1_024.0)
    } else {
        format!("{}B", bytes)
    }
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

// ============================================================================
// CLI Command Handlers
// ============================================================================

async fn run_search(
    claude_home: PathBuf,
    project: Option<PathBuf>,
    query: String,
    since: Option<String>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let store = DataStore::with_defaults(claude_home, project);

    // Show progress
    if !json {
        eprint!("Scanning sessions... ");
    }

    let report = store.initial_load().await;

    if !json && report.sessions_scanned > 0 {
        eprintln!("✓ {} sessions", report.sessions_scanned);
    }

    // Parse date filter
    let date_filter = if let Some(ref s) = since {
        Some(cli::DateFilter::parse(s).context("Invalid date filter")?)
    } else {
        None
    };

    // Search
    let all = store.recent_sessions(usize::MAX);
    let results = cli::search_sessions(&all, &query, date_filter.as_ref(), limit);

    if results.is_empty() {
        return Err(cli::CliError::NoResults {
            query,
            scanned: all.len(),
        }
        .into());
    }

    println!("{}", cli::format_session_table(&results, json));

    if !json {
        eprintln!("\n{} results from {} sessions", results.len(), all.len());
    }

    Ok(())
}

async fn run_recent(
    claude_home: PathBuf,
    project: Option<PathBuf>,
    count: usize,
    since: Option<String>,
    json: bool,
) -> Result<()> {
    let store = DataStore::with_defaults(claude_home, project);

    if !json {
        eprint!("Loading sessions... ");
    }

    let report = store.initial_load().await;

    if !json && report.sessions_scanned > 0 {
        eprintln!("✓ {} sessions", report.sessions_scanned);
    }

    // Parse date filter
    let date_filter = if let Some(ref s) = since {
        Some(cli::DateFilter::parse(s).context("Invalid date filter")?)
    } else {
        None
    };

    // Get recent sessions
    let mut all = store.recent_sessions(usize::MAX);

    // Apply date filter if specified
    if let Some(filter) = date_filter {
        all.retain(|s| {
            s.first_timestamp
                .map(|ts| filter.matches(&ts))
                .unwrap_or(false)
        });
    }

    let results: Vec<_> = all.into_iter().take(count).collect();

    if results.is_empty() {
        if !json {
            println!("No sessions found.");
        }
        return Ok(());
    }

    println!("{}", cli::format_session_table(&results, json));

    if !json {
        eprintln!(
            "\nShowing {} of {} sessions",
            results.len(),
            report.sessions_scanned
        );
    }

    Ok(())
}

async fn run_info(
    claude_home: PathBuf,
    project: Option<PathBuf>,
    session_id: String,
    json: bool,
) -> Result<()> {
    let store = DataStore::with_defaults(claude_home, project);

    if !json {
        eprint!("Loading sessions... ");
    }

    store.initial_load().await;

    if !json {
        eprintln!("✓");
    }

    let all = store.recent_sessions(usize::MAX);
    let session = cli::find_by_id_or_prefix(&all, &session_id)?;

    println!("{}", cli::format_session_info(&session, json));

    Ok(())
}

async fn run_resume(
    claude_home: PathBuf,
    project: Option<PathBuf>,
    session_id: String,
) -> Result<()> {
    let store = DataStore::with_defaults(claude_home, project);

    eprint!("Loading sessions... ");
    store.initial_load().await;
    eprintln!("✓");

    let all = store.recent_sessions(usize::MAX);
    let session = cli::find_by_id_or_prefix(&all, &session_id)?;

    eprintln!(
        "Resuming session {} in {}",
        &session.id[..8],
        session.project_path
    );

    // Unix: use exec() to replace process (no need to wait)
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = std::process::Command::new("claude")
            .args(["--resume", &session.id])
            .exec();
        anyhow::bail!("Failed to exec claude: {}", err);
    }

    // Windows: spawn and exit with same code
    #[cfg(not(unix))]
    {
        let status = std::process::Command::new("claude")
            .args(["--resume", &session.id])
            .status()
            .context("Failed to spawn claude (is 'claude' in PATH?)")?;
        std::process::exit(status.code().unwrap_or(1));
    }
}
