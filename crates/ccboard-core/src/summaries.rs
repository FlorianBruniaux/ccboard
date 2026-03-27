//! Session summary store — persisted to ~/.ccboard/summaries/<session_id>.md
//!
//! Summaries are generated once via `ccboard summarize <id>` (calls `claude --print`)
//! and cached. The TUI reads them from cache without re-generating.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Metadata file stored alongside each summary markdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMeta {
    /// Session ID this summary belongs to
    pub session_id: String,
    /// When the summary was generated
    pub generated_at: DateTime<Utc>,
    /// Model used for generation (e.g. "claude-sonnet-4-6")
    pub model: String,
}

/// Read/write access to the summaries cache directory
///
/// Layout: `~/.ccboard/summaries/`
///   `<session_id>.md`    — summary text
///   `<session_id>.json`  — SummaryMeta
pub struct SummaryStore {
    dir: PathBuf,
}

impl SummaryStore {
    /// Create from a base ccboard dir (e.g. `~/.ccboard`).
    /// Initialises the `summaries/` subdirectory on first use.
    pub fn new(ccboard_dir: &Path) -> Self {
        Self {
            dir: ccboard_dir.join("summaries"),
        }
    }

    /// True if a summary exists for this session ID.
    pub fn has_summary(&self, session_id: &str) -> bool {
        self.md_path(session_id).exists()
    }

    /// Load summary text. Returns `None` if not cached.
    pub fn load(&self, session_id: &str) -> Option<String> {
        std::fs::read_to_string(self.md_path(session_id)).ok()
    }

    /// Load metadata. Returns `None` if not cached.
    pub fn load_meta(&self, session_id: &str) -> Option<SummaryMeta> {
        let text = std::fs::read_to_string(self.meta_path(session_id)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Persist summary text + metadata atomically.
    pub fn save(&self, session_id: &str, summary: &str, model: &str) -> Result<()> {
        std::fs::create_dir_all(&self.dir)
            .with_context(|| format!("Failed to create summaries dir: {}", self.dir.display()))?;

        // Write markdown via tmp → rename
        let md = self.md_path(session_id);
        let tmp_md = md.with_extension("md.tmp");
        std::fs::write(&tmp_md, summary)
            .with_context(|| format!("Failed to write summary to {}", tmp_md.display()))?;
        std::fs::rename(&tmp_md, &md)
            .with_context(|| format!("Failed to rename summary to {}", md.display()))?;

        // Write metadata
        let meta = SummaryMeta {
            session_id: session_id.to_string(),
            generated_at: Utc::now(),
            model: model.to_string(),
        };
        let json = serde_json::to_string_pretty(&meta).context("Failed to serialise meta")?;
        let meta_path = self.meta_path(session_id);
        let tmp_meta = meta_path.with_extension("json.tmp");
        std::fs::write(&tmp_meta, &json)
            .with_context(|| format!("Failed to write meta to {}", tmp_meta.display()))?;
        std::fs::rename(&tmp_meta, &meta_path)
            .with_context(|| format!("Failed to rename meta to {}", meta_path.display()))?;

        Ok(())
    }

    /// Delete a cached summary (both md and meta).
    pub fn delete(&self, session_id: &str) -> Result<()> {
        let md = self.md_path(session_id);
        let meta = self.meta_path(session_id);
        if md.exists() {
            std::fs::remove_file(&md)
                .with_context(|| format!("Failed to delete {}", md.display()))?;
        }
        if meta.exists() {
            std::fs::remove_file(&meta)
                .with_context(|| format!("Failed to delete {}", meta.display()))?;
        }
        Ok(())
    }

    fn md_path(&self, session_id: &str) -> PathBuf {
        self.dir.join(format!("{}.md", session_id))
    }

    fn meta_path(&self, session_id: &str) -> PathBuf {
        self.dir.join(format!("{}.json", session_id))
    }
}

/// Call `claude --print` with a summarisation prompt built from session content.
///
/// Returns the raw summary text. Does NOT cache — caller should use `SummaryStore::save`.
pub fn call_claude_summarize(session_text: &str, model: &str) -> Result<String> {
    let prompt = build_summary_prompt(session_text);

    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    if !model.is_empty() {
        cmd.args(["--model", model]);
    }
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Remove env vars that block nested Claude sessions
    let env: Vec<(String, String)> = std::env::vars()
        .filter(|(k, _)| k != "CLAUDECODE" && k != "CLAUDE_CODE_ENTRYPOINT")
        .collect();
    cmd.env_clear();
    for (k, v) in &env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!("'claude' CLI not found. Make sure Claude Code is installed and in PATH.")
        } else {
            anyhow::anyhow!("Failed to run claude CLI: {}", e)
        }
    })?;

    // Write prompt to stdin then close it
    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin
            .write_all(prompt.as_bytes())
            .context("Failed to write prompt to claude stdin")?;
    }

    let output = child.wait_with_output().context("Failed to wait for claude")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("claude --print exited with error: {}", stderr);
    }

    let summary = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if summary.is_empty() {
        anyhow::bail!("claude --print returned empty output");
    }

    Ok(summary)
}

fn build_summary_prompt(session_text: &str) -> String {
    // Truncate to ~20K chars to stay within context limits
    const MAX_CHARS: usize = 20_000;
    let truncated = if session_text.len() > MAX_CHARS {
        let cut = &session_text[..MAX_CHARS];
        format!("{}\n\n[... truncated for length ...]", cut)
    } else {
        session_text.to_string()
    };

    format!(
        r#"You are summarising a Claude Code session log for a developer dashboard.

Produce a concise summary in plain text (no markdown headers, no bullet asterisks).
Structure: 1-2 sentence overview, then "What was accomplished:", then "Key decisions/findings:" (if any).
Keep it under 200 words. Be factual and technical — no filler.

SESSION LOG:
{truncated}

SUMMARY:"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let store = SummaryStore::new(dir.path());
        let id = "abc123";

        assert!(!store.has_summary(id));

        store.save(id, "This is a summary.", "test-model").unwrap();

        assert!(store.has_summary(id));
        assert_eq!(store.load(id).unwrap(), "This is a summary.");

        let meta = store.load_meta(id).unwrap();
        assert_eq!(meta.session_id, id);
        assert_eq!(meta.model, "test-model");
    }

    #[test]
    fn test_delete() {
        let dir = tempdir().unwrap();
        let store = SummaryStore::new(dir.path());
        let id = "del123";

        store.save(id, "content", "model").unwrap();
        assert!(store.has_summary(id));

        store.delete(id).unwrap();
        assert!(!store.has_summary(id));
        assert!(store.load(id).is_none());
    }

    #[test]
    fn test_load_missing_returns_none() {
        let dir = tempdir().unwrap();
        let store = SummaryStore::new(dir.path());
        assert!(store.load("nonexistent").is_none());
        assert!(store.load_meta("nonexistent").is_none());
    }
}
