# CLI Patterns

Idiomatic command-line interface development with clap, configuration management, and user interaction.

## Clap Derive API

### Basic CLI Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "myapp")]
#[command(about = "A CLI tool", long_about = None)]
struct Cli {
    /// Global verbosity flag
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new item
    Add {
        /// Item name
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all items
    List {
        /// Show detailed output
        #[arg(short, long)]
        detailed: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { name, description } => {
            println!("Adding: {}", name);
            if let Some(desc) = description {
                println!("Description: {}", desc);
            }
        }
        Commands::List { detailed } => {
            println!("Listing items (detailed: {})", detailed);
        }
    }
}
```

### Trailing Variable Arguments (RTK Pattern)

Extracted from rtk project for passing arbitrary arguments to underlying commands.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Git diff with custom arguments
    Diff {
        /// Arguments passed directly to git diff
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Git status
    Status {
        /// Additional git status flags
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Diff { args } => {
            // Pass args directly to git
            run_git_command(&["diff".to_string()].iter().chain(&args).collect::<Vec<_>>());
        }
        Commands::Status { args } => {
            run_git_command(&["status".to_string()].iter().chain(&args).collect::<Vec<_>>());
        }
    }
}

fn run_git_command(args: &[String]) {
    use std::process::Command;

    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to execute git");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}
```

### Value Parsers and Validation

```rust
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Json,
    Yaml,
    Toml,
}

#[derive(Parser)]
struct Cli {
    /// Output format
    #[arg(short, long, value_enum, default_value = "json")]
    format: OutputFormat,

    /// Port number (1-65535)
    #[arg(short, long, value_parser = clap::value_parser!(u16).range(1..=65535))]
    port: u16,
}
```

### Argument Groups

```rust
use clap::{ArgGroup, Parser};

#[derive(Parser)]
#[command(group(
    ArgGroup::new("output")
        .required(true)
        .args(&["json", "yaml", "toml"])
))]
struct Cli {
    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Output as YAML
    #[arg(long)]
    yaml: bool,

    /// Output as TOML
    #[arg(long)]
    toml: bool,
}
```

## Configuration Management

### Layered Configuration (CLI > Env > File > Defaults)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    host: String,
    port: u16,
    verbose: bool,
}

impl Config {
    fn load() -> anyhow::Result<Self> {
        // Start with defaults
        let mut config = Self::default();

        // Load from file if exists
        if let Ok(file_config) = Self::from_file("config.toml") {
            config = file_config;
        }

        // Override with environment variables
        if let Ok(host) = std::env::var("APP_HOST") {
            config.host = host;
        }
        if let Ok(port) = std::env::var("APP_PORT") {
            config.port = port.parse()?;
        }

        // CLI args override everything (handled by clap)

        Ok(config)
    }

    fn from_file(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            verbose: false,
        }
    }
}
```

### Config File Location

```rust
use std::path::PathBuf;

fn config_path() -> PathBuf {
    // Try in order: CLI arg > env var > XDG > home
    if let Ok(path) = std::env::var("APP_CONFIG") {
        return PathBuf::from(path);
    }

    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("myapp").join("config.toml");
        if path.exists() {
            return path;
        }
    }

    if let Some(home) = dirs::home_dir() {
        return home.join(".myapp.toml");
    }

    PathBuf::from("config.toml")
}
```

## User Interaction

### Progress Bars with indicatif

```rust
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

fn process_items(items: &[String]) {
    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    for item in items {
        pb.set_message(format!("Processing {}", item));
        // Do work
        std::thread::sleep(Duration::from_millis(100));
        pb.inc(1);
    }

    pb.finish_with_message("Done!");
}
```

### Colored Output

```rust
use colored::Colorize;

fn print_status(success: bool, message: &str) {
    if success {
        println!("{} {}", "✓".green(), message);
    } else {
        eprintln!("{} {}", "✗".red(), message);
    }
}

fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message.yellow());
}
```

### Interactive Prompts

```rust
use dialoguer::{Confirm, Input, Select};

fn interactive_setup() -> anyhow::Result<()> {
    // Text input
    let name: String = Input::new()
        .with_prompt("Enter your name")
        .default("User".to_string())
        .interact_text()?;

    // Confirmation
    let confirm = Confirm::new()
        .with_prompt("Do you want to continue?")
        .default(true)
        .interact()?;

    if !confirm {
        return Ok(());
    }

    // Selection
    let options = &["Option 1", "Option 2", "Option 3"];
    let selection = Select::new()
        .with_prompt("Choose an option")
        .items(options)
        .default(0)
        .interact()?;

    println!("Selected: {}", options[selection]);

    Ok(())
}
```

## Error Handling in CLI

### User-Friendly Error Messages

```rust
use anyhow::{Context, Result};

fn run_cli() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    process_data(&config)
        .context("Failed to process data")?;

    Ok(())
}

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("Error: {}", e);

        // Print chain of causes
        let mut source = e.source();
        while let Some(cause) = source {
            eprintln!("  Caused by: {}", cause);
            source = cause.source();
        }

        std::process::exit(1);
    }
}
```

### Exit Codes

```rust
use std::process::ExitCode;

fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

// Or with custom codes
fn main() -> ExitCode {
    match run_cli() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) if e.is::<ConfigError>() => ExitCode::from(2),
        Err(e) if e.is::<NetworkError>() => ExitCode::from(3),
        Err(_) => ExitCode::FAILURE,
    }
}
```

## Signal Handling

### Graceful Shutdown (Ctrl+C)

```rust
use anyhow::Result;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    // Spawn main work
    let work_handle = tokio::spawn(async {
        loop {
            // Do work
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            println!("Working...");
        }
    });

    // Wait for SIGINT
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        }
        _ = work_handle => {
            println!("Work completed");
        }
    }

    Ok(())
}
```

### Cleanup on Exit

```rust
struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        println!("Cleaning up resources...");
        // Close connections, flush buffers, etc.
    }
}

fn main() {
    let _cleanup = Cleanup;
    // Work happens here
    // Cleanup runs automatically on exit (even on panic)
}
```

## Shell Integration

### Shell Completion Generation

```rust
use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, shells::Shell};

#[derive(Parser)]
struct Cli {
    // CLI definition
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(
        shell,
        &mut cmd,
        "myapp",
        &mut std::io::stdout(),
    );
}

// Generate with: myapp --generate-completion bash > /etc/bash_completion.d/myapp
```

### Environment Variable Loading

```rust
use std::env;

fn load_env_vars() {
    // Load from .env file
    dotenv::dotenv().ok();

    // Access vars
    let api_key = env::var("API_KEY")
        .expect("API_KEY not set");
}
```

## RTK-Specific Patterns

### Git Command Wrapper

```rust
use anyhow::{Context, Result};
use std::process::Command;

pub fn run_git_command(args: &[String]) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
```

### Subcommand Dispatch Pattern

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    List { args: Vec<String> },
    Outdated { args: Vec<String> },
    Install { args: Vec<String> },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { args } => pnpm::list(&args),
        Commands::Outdated { args } => pnpm::outdated(&args),
        Commands::Install { args } => pnpm::install(&args),
    }
}
```

## Best Practices

### ✅ Do

- Use `clap` derive API for type-safe argument parsing
- Provide helpful error messages with context
- Support both CLI args and environment variables
- Use `trailing_var_arg` for proxy commands
- Implement graceful shutdown handling
- Generate shell completions
- Use colored output for better UX
- Exit with appropriate exit codes

### ❌ Don't

- Don't use `std::env::args()` directly (use clap)
- Don't print errors to stdout (use stderr)
- Don't ignore SIGINT/SIGTERM
- Don't use `.unwrap()` for user-facing errors
- Don't hard-code configuration paths
