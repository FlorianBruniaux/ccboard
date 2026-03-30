//! ccboard-specific configuration (`~/.ccboard/config.toml`)
//!
//! Separate from Claude Code's `settings.json` to avoid polluting
//! the Claude settings namespace with ccboard-only options.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::warn;

fn default_claude_mem_limit() -> usize {
    200
}

/// ccboard runtime configuration stored at `~/.ccboard/config.toml`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcboardConfig {
    /// Enable the claude-mem integration (reads `~/.claude-mem/claude-mem.db`)
    #[serde(default)]
    pub claude_mem_enabled: bool,

    /// Override the default path to claude-mem.db
    /// Defaults to `~/.claude-mem/claude-mem.db` when None
    #[serde(default)]
    pub claude_mem_db_path: Option<String>,

    /// Max number of observations to load from claude-mem (default: 200)
    #[serde(default = "default_claude_mem_limit")]
    pub claude_mem_limit: usize,
}

impl Default for CcboardConfig {
    fn default() -> Self {
        Self {
            claude_mem_enabled: false,
            claude_mem_db_path: None,
            claude_mem_limit: 200,
        }
    }
}

impl CcboardConfig {
    /// Load config from `<ccboard_dir>/config.toml`, returning defaults if absent or invalid
    pub fn load(ccboard_dir: &Path) -> Self {
        let path = ccboard_dir.join("config.toml");
        match std::fs::read_to_string(&path) {
            Ok(content) => match toml::from_str::<Self>(&content) {
                Ok(cfg) => cfg,
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "Failed to parse ccboard config.toml, using defaults");
                    Self::default()
                }
            },
            Err(_) => Self::default(), // File absent is fine — first run
        }
    }

    /// Persist config to `<ccboard_dir>/config.toml`
    pub fn save(&self, ccboard_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(ccboard_dir)
            .with_context(|| format!("Failed to create ccboard dir: {}", ccboard_dir.display()))?;
        let content = toml::to_string_pretty(self).context("Failed to serialize CcboardConfig")?;
        let path = ccboard_dir.join("config.toml");
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write {}", path.display()))
    }

    /// Resolve the effective path to claude-mem.db
    pub fn db_path(&self) -> PathBuf {
        if let Some(ref override_path) = self.claude_mem_db_path {
            PathBuf::from(override_path)
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".claude-mem/claude-mem.db")
        }
    }
}
