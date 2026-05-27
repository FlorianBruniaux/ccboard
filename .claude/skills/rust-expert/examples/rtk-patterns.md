# RTK Project Patterns

Real-world patterns extracted from the rtk CLI tool project, demonstrating idiomatic Rust for command-line applications.

## Project Structure

```
rtk/
├── src/
│   ├── main.rs           # Entry point, CLI parsing
│   ├── git.rs            # Git subcommand handlers
│   ├── pnpm.rs           # pnpm subcommand handlers
│   └── lib.rs            # Shared utilities (if needed)
├── Cargo.toml            # Dependencies and metadata
└── README.md             # Documentation
```

**Pattern**: Flat module structure for small CLI tools. Each subcommand domain gets its own module.

## CLI Structure with Clap

### Main Entry Point

```rust
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "rtk")]
#[command(about = "Rust toolkit for common dev operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Git operations with passthrough args
    #[command(subcommand)]
    Git(git::GitCommands),

    /// pnpm operations
    #[command(subcommand)]
    Pnpm(pnpm::PnpmCommands),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Git(cmd) => git::handle_command(cmd),
        Commands::Pnpm(cmd) => pnpm::handle_command(cmd),
    }
}
```

**Patterns**:
- Single `Result<()>` return from main for clean error handling
- Subcommands organized by domain (git, pnpm)
- Delegation to module-specific handlers

## Git Module: Trailing Arguments Pattern

### git.rs Structure

```rust
use clap::Subcommand;
use anyhow::{Context, Result};
use std::process::Command;

#[derive(Subcommand)]
pub enum GitCommands {
    /// Git diff with custom arguments
    Diff {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Git status
    Status {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Git log
    Log {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

pub fn handle_command(cmd: GitCommands) -> Result<()> {
    match cmd {
        GitCommands::Diff { args } => {
            run_git(&["diff"], &args)
        }
        GitCommands::Status { args } => {
            run_git(&["status"], &args)
        }
        GitCommands::Log { args } => {
            run_git(&["log"], &args)
        }
    }
}

fn run_git(base_args: &[&str], user_args: &[String]) -> Result<()> {
    let mut all_args: Vec<String> = base_args.iter().map(|s| s.to_string()).collect();
    all_args.extend_from_slice(user_args);

    let output = Command::new("git")
        .args(&all_args)
        .output()
        .context("Failed to execute git command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
```

**Patterns**:
- `trailing_var_arg = true`: Captures all remaining arguments
- `allow_hyphen_values = true`: Allows flags like `--cached` to pass through
- Unified `run_git` function for DRY principle
- Contextual error messages with `.context()`
- Proper stdout/stderr handling

### Usage Example

```bash
# All these work:
rtk git diff --cached
rtk git status -s
rtk git log --oneline --graph
```

## pnpm Module: Command Execution Pattern

### pnpm.rs Structure

```rust
use clap::Subcommand;
use anyhow::{Context, Result};
use std::process::Command;

#[derive(Subcommand)]
pub enum PnpmCommands {
    /// List installed packages
    List {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Show outdated packages
    Outdated {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Install packages
    Install {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

pub fn handle_command(cmd: PnpmCommands) -> Result<()> {
    match cmd {
        PnpmCommands::List { args } => list(&args),
        PnpmCommands::Outdated { args } => outdated(&args),
        PnpmCommands::Install { args } => install(&args),
    }
}

pub fn list(args: &[String]) -> Result<()> {
    run_pnpm_command(&["list"], args)
}

pub fn outdated(args: &[String]) -> Result<()> {
    run_pnpm_command(&["outdated"], args)
}

pub fn install(args: &[String]) -> Result<()> {
    // Validation before execution
    validate_package_names(args)?;

    run_pnpm_command(&["install"], args)
}

fn validate_package_names(names: &[String]) -> Result<()> {
    for name in names {
        if name.contains("..") || name.starts_with('/') {
            anyhow::bail!("Invalid package name: {}", name);
        }
    }
    Ok(())
}

fn run_pnpm_command(base_args: &[&str], user_args: &[String]) -> Result<()> {
    let mut all_args: Vec<String> = base_args.iter().map(|s| s.to_string()).collect();
    all_args.extend_from_slice(user_args);

    let output = Command::new("pnpm")
        .args(&all_args)
        .output()
        .context("Failed to execute pnpm command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("pnpm command failed:\n{}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
```

**Patterns**:
- Security validation before execution
- Consistent error handling with `anyhow`
- Separate functions for each operation (testable)
- Unified command execution helper

## Error Handling Patterns

### Contextual Errors

```rust
fn read_config_file(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read config file: {}", path))?;

    let config = toml::from_str(&content)
        .context("Failed to parse TOML config")?;

    Ok(config)
}
```

### Early Returns with bail

```rust
fn validate_git_repo() -> Result<()> {
    if !std::path::Path::new(".git").exists() {
        anyhow::bail!("Not a git repository");
    }

    Ok(())
}
```

## Testing Patterns

### Unit Tests Embedded

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_names() {
        assert!(validate_package_names(&["lodash".to_string()]).is_ok());
        assert!(validate_package_names(&["../evil".to_string()]).is_err());
        assert!(validate_package_names(&["/etc/passwd".to_string()]).is_err());
    }

    #[test]
    fn test_command_parsing() {
        use clap::Parser;

        let cli = Cli::parse_from(vec!["rtk", "git", "diff", "--cached"]);
        match cli.command {
            Commands::Git(GitCommands::Diff { args }) => {
                assert_eq!(args, vec!["--cached"]);
            }
            _ => panic!("Wrong command parsed"),
        }
    }
}
```

## Cargo.toml Configuration

```toml
[package]
name = "rtk"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
anyhow = "1"

[profile.release]
opt-level = 3
lto = true
strip = true

[profile.dev]
opt-level = 0
```

**Patterns**:
- Minimal dependencies (clap + anyhow)
- Derive feature for type-safe CLI
- Optimized release profile

## Build Optimization

### .cargo/config.toml

```toml
[build]
# Parallel compilation
jobs = 8

[target.x86_64-unknown-linux-gnu]
# Faster linker
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-apple-darwin]
# macOS alternative
rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/bin/zld"]
```

## Key Takeaways

### ✅ RTK Patterns to Follow

1. **Flat module structure** for small CLI tools
2. **Trailing variable args** for command passthrough
3. **anyhow for application errors** (not libraries)
4. **Contextual error messages** with `.context()`
5. **Security validation** before executing external commands
6. **Embedded unit tests** with `#[cfg(test)]`
7. **Minimal dependencies** (only what's needed)
8. **Optimized release builds** (LTO, strip)

### 🎯 Design Principles

- **DRY**: Unified command execution functions
- **Security**: Validate user input before execution
- **Usability**: Pass flags directly to underlying commands
- **Maintainability**: Clear separation of concerns by module
- **Testability**: Pure functions for business logic

### 📦 Dependency Choices

- **clap**: Type-safe CLI parsing (derive API)
- **anyhow**: Ergonomic error handling for applications
- **No async runtime**: Synchronous is simpler for CLI tools
- **No logging framework**: Use simple eprintln! for errors

### 🚀 Performance Considerations

- Synchronous execution (no async overhead)
- Fast linking with mold/zld
- Release builds optimized for size and speed
- Minimal dependencies reduce compilation time
