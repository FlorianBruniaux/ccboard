//! Hook subcommand handler
//!
//! Invoked by Claude Code hook scripts. Reads JSON from stdin, updates
//! `~/.ccboard/live-sessions.json` with file locking for concurrent safety.
//!
//! Must complete in <20ms — synchronous only, no tokio runtime initialization.

use anyhow::{Context, Result};
use ccboard_core::hook_event::{status_from_event, HookPayload};
use ccboard_core::hook_state::{make_session_key, LiveSessionFile};
use fd_lock::RwLock as FileLock;
use std::io::{self, Read};
use std::time::Duration;

pub fn run_hook(event_name: String) -> Result<()> {
    // 1. Read stdin (Claude Code pipes JSON to our process)
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context("Failed to read stdin")?;

    // 2. Parse payload — graceful: empty input uses minimal defaults
    let mut payload: HookPayload = if input.trim().is_empty() {
        HookPayload {
            session_id: "unknown".to_string(),
            cwd: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .unwrap_or_else(|| "/tmp".to_string()),
            ..Default::default()
        }
    } else {
        serde_json::from_str(&input).context("Failed to parse hook JSON payload")?
    };
    payload.event_name = event_name.clone();

    // 3. Determine TTY
    let tty = detect_tty();

    // 4. Compute status from event name + payload
    let status = status_from_event(&event_name, &payload);
    let session_key = make_session_key(&payload.session_id, &tty);

    // 5. Ensure ~/.ccboard/ directory exists
    let base_dir = dirs::home_dir()
        .context("Cannot determine home directory")?
        .join(".ccboard");
    std::fs::create_dir_all(&base_dir).context("Failed to create ~/.ccboard/")?;

    let file_path = base_dir.join("live-sessions.json");
    let lock_path = base_dir.join("live-sessions.lock");

    // 6. Acquire file lock (blocks until available)
    let lock_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .context("Failed to open lock file")?;
    let mut file_lock = FileLock::new(lock_file);
    let _lock_guard = file_lock
        .write()
        .context("Failed to acquire write lock on live-sessions")?;

    // 7. Load existing state (Default if file absent)
    let mut session_file = LiveSessionFile::load(&file_path).unwrap_or_else(|e| {
        eprintln!("[ccboard] Warning: failed to parse live-sessions.json: {e}. Starting fresh.");
        LiveSessionFile {
            version: 1,
            ..Default::default()
        }
    });

    // 8. Upsert session entry
    session_file.upsert(
        session_key,
        payload.session_id.clone(),
        payload.cwd.clone(),
        tty,
        status,
        event_name.clone(),
    );

    // 9. Prune stopped sessions older than 30 minutes
    session_file.prune_stopped(Duration::from_secs(30 * 60));
    session_file.updated_at = Some(chrono::Utc::now());

    // 10. Atomic save (write to .tmp then rename)
    session_file
        .save(&file_path)
        .context("Failed to save live-sessions.json")?;

    // 11. Release lock (implicit drop of _lock_guard)
    drop(_lock_guard);

    // 12. macOS notification on Stop (non-blocking spawn)
    #[cfg(target_os = "macos")]
    {
        use ccboard_core::hook_state::HookSessionStatus;
        if status == HookSessionStatus::Stopped {
            let project_name = std::path::Path::new(&payload.cwd)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Sanitize: escape backslashes then double-quotes to prevent AppleScript injection
            let safe_name = project_name.replace('\\', "\\\\").replace('"', "\\\"");

            let _ = std::process::Command::new("osascript")
                .args([
                    "-e",
                    &format!(
                        "display notification \"Session terminée : {}\" with title \"ccboard\"",
                        safe_name
                    ),
                ])
                .spawn(); // non-blocking — hook returns immediately
        }
    }

    Ok(())
}

/// Detect the current TTY.
///
/// Priority:
/// 1. `$TTY` environment variable
/// 2. `tty` command output
/// 3. `"unknown"` fallback
fn detect_tty() -> String {
    // $TTY is set by most shells and is the most reliable source
    if let Ok(tty) = std::env::var("TTY") {
        if !tty.is_empty() {
            return tty;
        }
    }

    // Fallback: run `tty` command
    if let Ok(output) = std::process::Command::new("tty").output() {
        if output.status.success() {
            let tty = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !tty.is_empty() && !tty.contains("not a tty") {
                return tty;
            }
        }
    }

    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tty_fallback() {
        // Just ensure it returns a string without panicking
        let tty = detect_tty();
        assert!(!tty.is_empty());
    }
}
