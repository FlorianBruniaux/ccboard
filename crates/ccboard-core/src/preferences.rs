//! User preferences persistence for ccboard
//!
//! Stores UI preferences (theme, etc.) in `~/.claude/cache/ccboard-preferences.json`.

use crate::models::config::ColorScheme;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// ccboard-specific user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcboardPreferences {
    /// Color scheme (dark / light)
    pub color_scheme: ColorScheme,
}

impl Default for CcboardPreferences {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Dark,
        }
    }
}

impl CcboardPreferences {
    /// Load preferences from `<cache_dir>/ccboard-preferences.json`.
    /// Returns defaults on any I/O or parse error (graceful degradation).
    pub fn load(cache_dir: &Path) -> Self {
        let path = cache_dir.join("ccboard-preferences.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist preferences to `<cache_dir>/ccboard-preferences.json`.
    pub fn save(&self, cache_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(cache_dir)
            .context("Failed to create cache directory for preferences")?;
        let path = cache_dir.join("ccboard-preferences.json");
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize preferences")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write preferences to {}", path.display()))
    }
}
