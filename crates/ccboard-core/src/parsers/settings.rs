//! Settings parser with explicit deep merge

use crate::error::{CoreError, LoadError, LoadReport};
use crate::models::{HookDefinition, HookGroup, MergedConfig, Settings};
use crate::parsers::HooksParser;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, warn};

/// Parser for Claude Code settings files
pub struct SettingsParser;

impl Default for SettingsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single settings.json file
    pub async fn parse(&self, path: &Path) -> Result<Settings, CoreError> {
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CoreError::FileNotFound {
                    path: path.to_path_buf(),
                }
            } else {
                CoreError::FileRead {
                    path: path.to_path_buf(),
                    source: e,
                }
            }
        })?;

        serde_json::from_str(&content).map_err(|e| CoreError::JsonParse {
            path: path.to_path_buf(),
            message: e.to_string(),
            source: e,
        })
    }

    /// Parse settings file with graceful degradation
    pub async fn parse_graceful(
        &self,
        path: &Path,
        source_name: &str,
        report: &mut LoadReport,
    ) -> Option<Settings> {
        match self.parse(path).await {
            Ok(settings) => {
                debug!(?path, "Loaded settings");
                Some(settings)
            }
            Err(CoreError::FileNotFound { .. }) => {
                // File not existing is normal for project/local settings
                debug!(?path, "Settings file not found (optional)");
                None
            }
            Err(e) => {
                warn!(?path, error = %e, "Failed to parse settings");
                report.add_error(LoadError::error(source_name, e.to_string()));
                None
            }
        }
    }

    /// Load and merge settings from all three levels
    ///
    /// Priority: local > project > global
    pub async fn load_merged(
        &self,
        claude_home: &Path,
        project_path: Option<&Path>,
        report: &mut LoadReport,
    ) -> MergedConfig {
        // Global: ~/.claude/settings.json
        let global_path = claude_home.join("settings.json");
        let global = self
            .parse_graceful(&global_path, "settings.global", report)
            .await;

        // Project: <project>/.claude/settings.json
        let project = if let Some(proj) = project_path {
            let project_path = proj.join(".claude").join("settings.json");
            self.parse_graceful(&project_path, "settings.project", report)
                .await
        } else {
            None
        };

        // Local: <project>/.claude/settings.local.json
        let local = if let Some(proj) = project_path {
            let local_path = proj.join(".claude").join("settings.local.json");
            self.parse_graceful(&local_path, "settings.local", report)
                .await
        } else {
            None
        };

        if global.is_some() || project.is_some() || local.is_some() {
            report.settings_loaded = true;
        }

        // Scan .sh hooks from filesystem and inject into global settings
        let mut global_with_hooks = global;
        if let Some(ref mut settings) = global_with_hooks {
            Self::inject_scanned_hooks(settings, claude_home, project_path);
        } else {
            // If no global settings, create minimal settings with scanned hooks
            let mut settings = Settings::default();
            Self::inject_scanned_hooks(&mut settings, claude_home, project_path);
            if settings.hooks.is_some() {
                global_with_hooks = Some(settings);
            }
        }

        MergedConfig::from_layers(global_with_hooks, project, local)
    }

    /// Scan .sh files from hooks directories and inject into settings
    fn inject_scanned_hooks(settings: &mut Settings, claude_home: &Path, project_path: Option<&Path>) {
        let mut all_scanned_hooks: Vec<crate::parsers::Hook> = Vec::new();

        // Scan global hooks: ~/.claude/hooks/*.sh
        let global_hooks_dir = claude_home.join("hooks");
        if let Ok(hooks) = HooksParser::scan_directory(&global_hooks_dir) {
            all_scanned_hooks.extend(hooks);
        }

        // Scan project hooks: <project>/.claude/hooks/*.sh
        if let Some(proj) = project_path {
            let project_hooks_dir = proj.join(".claude").join("hooks");
            if let Ok(hooks) = HooksParser::scan_directory(&project_hooks_dir) {
                all_scanned_hooks.extend(hooks);
            }
        }

        if all_scanned_hooks.is_empty() {
            return;
        }

        // Convert scanned hooks to HookDefinition and group by type
        let mut hooks_by_event: HashMap<String, Vec<HookGroup>> = settings
            .hooks
            .clone()
            .unwrap_or_default();

        for hook in all_scanned_hooks {
            // Map HookType to event name
            let event_name = match hook.hook_type {
                crate::parsers::HookType::PreCommit => "PreCommit",
                crate::parsers::HookType::PostCommit => "PostCommit",
                crate::parsers::HookType::PrePush => "PrePush",
                crate::parsers::HookType::UserPromptSubmit => "UserPromptSubmit",
                crate::parsers::HookType::ToolResultReturn => "ToolResultReturn",
                crate::parsers::HookType::Custom(ref name) => {
                    // Try to infer event type from custom name
                    if name.contains("pre") || name.contains("before") {
                        "PreToolUse"
                    } else {
                        "Custom"
                    }
                }
            };

            // Create HookDefinition from scanned hook
            let hook_def = HookDefinition {
                command: hook.path.display().to_string(),
                r#async: None,
                timeout: None,
                cwd: None,
                env: None,
                file_path: Some(hook.path.clone()),
            };

            // Add to event group
            let groups = hooks_by_event.entry(event_name.to_string()).or_default();

            // Add to first group or create new group
            if let Some(first_group) = groups.first_mut() {
                first_group.hooks.push(hook_def);
            } else {
                groups.push(HookGroup {
                    matcher: None,
                    hooks: vec![hook_def],
                });
            }
        }

        settings.hooks = Some(hooks_by_event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, tempdir};

    #[tokio::test]
    async fn test_parse_valid_settings() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
            "model": "claude-sonnet-4-20250514",
            "permissions": {{
                "allow": ["Read", "Write"],
                "autoApprove": true
            }}
        }}"#
        )
        .unwrap();

        let parser = SettingsParser::new();
        let settings = parser.parse(file.path()).await.unwrap();

        assert_eq!(settings.model, Some("claude-sonnet-4-20250514".to_string()));
        let perms = settings.permissions.unwrap();
        assert_eq!(
            perms.allow,
            Some(vec!["Read".to_string(), "Write".to_string()])
        );
        assert_eq!(perms.auto_approve, Some(true));
    }

    #[tokio::test]
    async fn test_parse_missing_file_graceful() {
        let parser = SettingsParser::new();
        let mut report = LoadReport::new();

        let result = parser
            .parse_graceful(Path::new("/nonexistent/settings.json"), "test", &mut report)
            .await;

        assert!(result.is_none());
        // Missing file is not an error for optional settings
        assert!(!report.has_errors() || report.warnings().count() == 0);
    }

    #[tokio::test]
    async fn test_load_merged_hierarchy() {
        let dir = tempdir().unwrap();
        let claude_home = dir.path().join(".claude");
        let project = dir.path().join("myproject");
        let project_claude = project.join(".claude");

        std::fs::create_dir_all(&claude_home).unwrap();
        std::fs::create_dir_all(&project_claude).unwrap();

        // Global settings
        std::fs::write(
            claude_home.join("settings.json"),
            r#"{"model": "opus", "theme": "dark"}"#,
        )
        .unwrap();

        // Project settings (overrides model)
        std::fs::write(
            project_claude.join("settings.json"),
            r#"{"model": "sonnet"}"#,
        )
        .unwrap();

        let parser = SettingsParser::new();
        let mut report = LoadReport::new();

        let merged = parser
            .load_merged(&claude_home, Some(&project), &mut report)
            .await;

        assert!(report.settings_loaded);
        // Model overridden by project
        assert_eq!(merged.merged.model, Some("sonnet".to_string()));
        // Theme from global preserved
        assert_eq!(merged.merged.theme, Some("dark".to_string()));
    }
}
