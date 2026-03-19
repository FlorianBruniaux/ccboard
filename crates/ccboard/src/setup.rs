//! Setup subcommand — injects ccboard hooks into Claude Code settings.json
//!
//! Idempotent: running twice produces the same result.
//! Safe: creates a backup before modifying, validates JSON before saving.

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::path::PathBuf;

/// Claude Code hook events that ccboard monitors
const HOOKS_TO_INJECT: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "UserPromptSubmit",
    "Notification",
    "Stop",
];

/// Run the setup subcommand.
///
/// If `dry_run` is true, prints what would be changed but does not write any files.
pub async fn run_setup(dry_run: bool, claude_home: PathBuf) -> Result<()> {
    // 1. Verify ~/.claude/ exists
    if !claude_home.exists() {
        anyhow::bail!(
            "Claude home directory not found: {}. Run 'claude' at least once to initialize it.",
            claude_home.display()
        );
    }

    let settings_path = claude_home.join("settings.json");

    // 2. Load existing settings (or empty object)
    let settings_content = if settings_path.exists() {
        std::fs::read_to_string(&settings_path)
            .with_context(|| format!("Failed to read {}", settings_path.display()))?
    } else {
        "{}".to_string()
    };

    let mut settings: Map<String, Value> = serde_json::from_str(&settings_content)
        .with_context(|| format!("Failed to parse {}", settings_path.display()))?;

    // 3. Get the binary path (current executable)
    let binary_path = std::env::current_exe()
        .context("Failed to determine ccboard binary path")?
        .to_string_lossy()
        .to_string();

    // Warn if running from a debug/cargo build — the injected path won't survive a `cargo clean`
    if binary_path.contains("/target/debug/") || binary_path.contains("/target/release/") {
        eprintln!(
            "Warning: ccboard is running from a build directory ({}).\n\
             The hook scripts will point to this binary. Install ccboard system-wide\n\
             (e.g. via `cargo install` or Homebrew) for a stable hook path.",
            binary_path
        );
    }

    // 4. Inject hooks (idempotent)
    let hooks_obj = settings
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .context("'hooks' field in settings.json is not an object")?;

    let mut added: Vec<&str> = Vec::new();
    let mut already_present: Vec<&str> = Vec::new();

    for event in HOOKS_TO_INJECT {
        let hook_command = format!("{} hook {}", binary_path, event);

        // Check if hook already present (exact match)
        if hook_already_present(hooks_obj, event, &hook_command) {
            already_present.push(*event);
            continue;
        }

        // Add hook entry — new format: [{matcher, hooks: [{type, command}]}]
        let hook_def = json!([{
            "matcher": "",
            "hooks": [{"type": "command", "command": hook_command}]
        }]);

        hooks_obj.insert(event.to_string(), hook_def);
        added.push(*event);
    }

    // 5. Report changes
    println!();
    if dry_run {
        println!("  ccboard setup --dry-run");
    } else {
        println!("  ccboard setup");
    }
    println!();

    if !added.is_empty() {
        println!("  Hooks to add:");
        for event in &added {
            println!("    + {} → {} hook {}", event, binary_path, event);
        }
        println!();
    }

    if !already_present.is_empty() {
        println!("  Already configured:");
        for event in &already_present {
            println!("    ✓ {}", event);
        }
        println!();
    }

    if added.is_empty() {
        println!("  Nothing to do — all hooks already configured.");
        println!();
        return Ok(());
    }

    if dry_run {
        println!("  (dry-run) No files written.");
        println!();
        return Ok(());
    }

    // 6. Validate final JSON is parseable
    let final_json = serde_json::to_string_pretty(&Value::Object(settings.clone()))
        .context("Failed to serialize updated settings")?;

    // Sanity check: re-parse to confirm valid JSON
    serde_json::from_str::<Value>(&final_json).context("Generated invalid JSON — aborting")?;

    // 7. Backup existing settings
    if settings_path.exists() {
        let backup_path = settings_path.with_extension("json.ccboard-backup");
        std::fs::copy(&settings_path, &backup_path)
            .with_context(|| format!("Failed to create backup at {}", backup_path.display()))?;
        println!("  Backup: {}", backup_path.display());
    }

    // 8. Atomic save
    let tmp_path = settings_path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &final_json)
        .with_context(|| format!("Failed to write tmp file {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &settings_path)
        .with_context(|| format!("Failed to rename tmp to {}", settings_path.display()))?;

    println!("  ✓ Saved {}", settings_path.display());
    println!();
    println!(
        "  {} hook(s) injected. Restart Claude Code for hooks to take effect.",
        added.len()
    );
    println!();

    Ok(())
}

/// Check if a hook command is already present for a given event.
///
/// Supports both formats:
/// - New: `[{matcher, hooks: [{type, command}]}]`
/// - Old: `[{type, command}]`
fn hook_already_present(hooks_obj: &Map<String, Value>, event: &str, command: &str) -> bool {
    let Some(event_hooks) = hooks_obj.get(event) else {
        return false;
    };

    let Some(hooks_array) = event_hooks.as_array() else {
        return false;
    };

    hooks_array.iter().any(|entry| {
        // New format: entry has a "hooks" array
        if let Some(inner) = entry.get("hooks").and_then(|h| h.as_array()) {
            return inner.iter().any(|hook| {
                hook.get("command")
                    .and_then(|c| c.as_str())
                    .map(|c| c == command)
                    .unwrap_or(false)
            });
        }
        // Old format: entry has "command" directly
        entry
            .get("command")
            .and_then(|c| c.as_str())
            .map(|c| c == command)
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hook_already_present_new_format() {
        let mut hooks_obj = Map::new();
        hooks_obj.insert(
            "PreToolUse".to_string(),
            json!([{"matcher": "", "hooks": [{"type": "command", "command": "/usr/local/bin/ccboard hook PreToolUse"}]}]),
        );
        assert!(hook_already_present(
            &hooks_obj,
            "PreToolUse",
            "/usr/local/bin/ccboard hook PreToolUse"
        ));
    }

    #[test]
    fn test_hook_already_present_old_format() {
        let mut hooks_obj = Map::new();
        hooks_obj.insert(
            "PreToolUse".to_string(),
            json!([{"type": "command", "command": "/usr/local/bin/ccboard hook PreToolUse"}]),
        );
        assert!(hook_already_present(
            &hooks_obj,
            "PreToolUse",
            "/usr/local/bin/ccboard hook PreToolUse"
        ));
    }

    #[test]
    fn test_hook_already_present_false() {
        let hooks_obj = Map::new();
        assert!(!hook_already_present(
            &hooks_obj,
            "PreToolUse",
            "/usr/local/bin/ccboard hook PreToolUse"
        ));
    }

    #[test]
    fn test_hook_different_command_not_present() {
        let mut hooks_obj = Map::new();
        hooks_obj.insert(
            "PreToolUse".to_string(),
            json!([{"type": "command", "command": "/other/tool hook PreToolUse"}]),
        );
        assert!(!hook_already_present(
            &hooks_obj,
            "PreToolUse",
            "/usr/local/bin/ccboard hook PreToolUse"
        ));
    }
}
