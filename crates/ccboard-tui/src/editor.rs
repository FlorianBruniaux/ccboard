use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::process::Command;

/// Opens a file in the user's preferred editor.
///
/// Editor selection priority:
/// 1. $VISUAL environment variable
/// 2. $EDITOR environment variable
/// 3. Platform default (nano on Unix, notepad.exe on Windows)
///
/// This function temporarily exits the alternate screen and disables raw mode
/// to allow the editor to take over the terminal, then restores the TUI state
/// after the editor exits.
///
/// # Errors
///
/// Returns error if:
/// - File path is invalid or doesn't exist
/// - Editor command fails to spawn
/// - Terminal state cannot be restored
pub fn open_in_editor(file_path: &Path) -> Result<()> {
    // Validate file exists
    if !file_path.exists() {
        anyhow::bail!("File does not exist: {}", file_path.display());
    }

    // Get editor command
    let editor = get_editor_command();

    // Exit alternate screen and disable raw mode
    use crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
    };
    use std::io::stdout;

    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(stdout(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;

    // Spawn editor (blocking)
    let status = Command::new(&editor)
        .arg(file_path)
        .status()
        .with_context(|| format!("Failed to spawn editor: {}", editor))?;

    // Re-enter alternate screen and enable raw mode
    use crossterm::terminal::EnterAlternateScreen;
    execute!(stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;
    enable_raw_mode().context("Failed to enable raw mode")?;

    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status: {:?}", status.code());
    }

    Ok(())
}

/// Opens the file's parent directory in the system file manager.
///
/// Platform-specific behavior:
/// - macOS: Uses `open -R` to reveal file in Finder
/// - Windows: Uses `explorer /select,` to select file in Explorer
/// - Linux: Uses `xdg-open` to open parent directory
///
/// This is a non-blocking operation (fire and forget).
///
/// # Errors
///
/// Returns error if:
/// - File path has no parent directory
/// - File manager command fails to spawn
pub fn reveal_in_file_manager(file_path: &Path) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(file_path)
            .spawn()
            .context("Failed to spawn 'open' command (macOS)")?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg("/select,")
            .arg(file_path)
            .spawn()
            .context("Failed to spawn 'explorer' command (Windows)")?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let parent = file_path
            .parent()
            .context("File has no parent directory")?;

        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .context("Failed to spawn 'xdg-open' command (Linux)")?;
    }

    Ok(())
}

/// Gets the editor command to use, checking environment variables.
fn get_editor_command() -> String {
    env::var("VISUAL")
        .or_else(|_| env::var("EDITOR"))
        .unwrap_or_else(|_| get_default_editor())
}

/// Returns the platform-specific default editor.
#[cfg(unix)]
fn get_default_editor() -> String {
    "nano".to_string()
}

#[cfg(windows)]
fn get_default_editor() -> String {
    "notepad.exe".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_editor_command_visual() {
        unsafe {
            env::set_var("VISUAL", "vim");
            env::remove_var("EDITOR");
        }
        assert_eq!(get_editor_command(), "vim");
    }

    #[test]
    fn test_get_editor_command_editor() {
        unsafe {
            env::remove_var("VISUAL");
            env::remove_var("EDITOR");
            env::set_var("EDITOR", "emacs");
        }
        assert_eq!(get_editor_command(), "emacs");
    }

    #[test]
    fn test_get_editor_command_default() {
        unsafe {
            env::remove_var("VISUAL");
            env::remove_var("EDITOR");
        }
        let default = get_editor_command();
        #[cfg(unix)]
        assert_eq!(default, "nano");
        #[cfg(windows)]
        assert_eq!(default, "notepad.exe");
    }

    #[test]
    fn test_open_in_editor_nonexistent_file() {
        let result = open_in_editor(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not exist"));
    }

    #[test]
    fn test_reveal_in_file_manager_no_parent() {
        // Root path has no parent
        let result = reveal_in_file_manager(Path::new("/"));
        // On some systems, root has no parent, but behavior varies
        // This test mainly checks we don't panic
        let _ = result; // Allow success or failure
    }
}
