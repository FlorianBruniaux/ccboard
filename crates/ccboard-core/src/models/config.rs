//! Configuration models for Claude Code settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Claude Code settings (from settings.json)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// Permission settings
    #[serde(default)]
    pub permissions: Option<Permissions>,

    /// Hook configurations by event name
    #[serde(default)]
    pub hooks: Option<HashMap<String, Vec<HookGroup>>>,

    /// Default model
    #[serde(default)]
    pub model: Option<String>,

    /// Environment variables to inject
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// Enabled plugins/features
    #[serde(default)]
    pub enabled_plugins: Option<HashMap<String, bool>>,

    /// API key (masked in display)
    #[serde(default)]
    pub api_key: Option<String>,

    /// Custom instructions
    #[serde(default)]
    pub custom_instructions: Option<String>,

    /// Theme settings
    #[serde(default)]
    pub theme: Option<String>,

    /// Additional untyped fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Settings {
    /// Get masked API key for display (SECURITY: never expose full key)
    ///
    /// Returns a masked version showing only prefix and suffix:
    /// "sk-ant-1234567890abcdef" → "sk-ant-••••cdef"
    pub fn masked_api_key(&self) -> Option<String> {
        self.api_key.as_ref().map(|key| {
            if key.len() <= 10 {
                // Short key: mask everything except first 3 chars
                format!("{}••••", &key.chars().take(3).collect::<String>())
            } else {
                // Standard key: show prefix and suffix
                let prefix = key.chars().take(7).collect::<String>();
                let suffix = key.chars().skip(key.len() - 4).collect::<String>();
                format!("{}••••{}", prefix, suffix)
            }
        })
    }
}

/// Permission configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    /// Allowed tools
    #[serde(default)]
    pub allow: Option<Vec<String>>,

    /// Denied tools
    #[serde(default)]
    pub deny: Option<Vec<String>>,

    /// Allowed Bash commands/patterns
    #[serde(default)]
    pub allow_bash: Option<Vec<String>>,

    /// Denied Bash commands/patterns
    #[serde(default)]
    pub deny_bash: Option<Vec<String>>,

    /// Auto-approve mode
    #[serde(default)]
    pub auto_approve: Option<bool>,

    /// Trust project settings
    #[serde(default)]
    pub trust_project: Option<bool>,
}

/// Hook group configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookGroup {
    /// Matcher pattern (glob or regex)
    #[serde(default)]
    pub matcher: Option<String>,

    /// Hooks in this group
    #[serde(default)]
    pub hooks: Vec<HookDefinition>,
}

/// Individual hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookDefinition {
    /// Command to execute
    pub command: String,

    /// Run asynchronously
    #[serde(default)]
    pub r#async: Option<bool>,

    /// Timeout in seconds
    #[serde(default)]
    pub timeout: Option<u32>,

    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,

    /// Source file path (not from JSON, populated during scanning)
    #[serde(skip)]
    pub file_path: Option<std::path::PathBuf>,
}

/// Merged configuration from all levels
#[derive(Debug, Clone, Default)]
pub struct MergedConfig {
    /// Source of each field for debugging
    pub global: Option<Settings>,
    pub project: Option<Settings>,
    pub local: Option<Settings>,

    /// Final merged result
    pub merged: Settings,
}

impl MergedConfig {
    /// Create merged config from three levels
    /// Priority: local > project > global
    pub fn from_layers(
        global: Option<Settings>,
        project: Option<Settings>,
        local: Option<Settings>,
    ) -> Self {
        let mut merged = Settings::default();

        // Merge global first
        if let Some(ref g) = global {
            Self::merge_into(&mut merged, g);
        }

        // Then project
        if let Some(ref p) = project {
            Self::merge_into(&mut merged, p);
        }

        // Finally local (highest priority)
        if let Some(ref l) = local {
            Self::merge_into(&mut merged, l);
        }

        Self {
            global,
            project,
            local,
            merged,
        }
    }

    /// Explicit field-by-field merge (not shallow copy)
    fn merge_into(target: &mut Settings, source: &Settings) {
        // Scalar fields: override if present
        if source.model.is_some() {
            target.model = source.model.clone();
        }
        if source.api_key.is_some() {
            target.api_key = source.api_key.clone();
        }
        if source.custom_instructions.is_some() {
            target.custom_instructions = source.custom_instructions.clone();
        }
        if source.theme.is_some() {
            target.theme = source.theme.clone();
        }

        // Permissions: deep merge
        if let Some(ref src_perms) = source.permissions {
            let target_perms = target.permissions.get_or_insert_with(Permissions::default);
            Self::merge_permissions(target_perms, src_perms);
        }

        // Hooks: merge by event name, then extend hook lists
        if let Some(ref src_hooks) = source.hooks {
            let target_hooks = target.hooks.get_or_insert_with(HashMap::new);
            for (event, groups) in src_hooks {
                let target_groups = target_hooks.entry(event.clone()).or_default();
                target_groups.extend(groups.clone());
            }
        }

        // Env: merge maps
        if let Some(ref src_env) = source.env {
            let target_env = target.env.get_or_insert_with(HashMap::new);
            for (k, v) in src_env {
                target_env.insert(k.clone(), v.clone());
            }
        }

        // Plugins: merge maps
        if let Some(ref src_plugins) = source.enabled_plugins {
            let target_plugins = target.enabled_plugins.get_or_insert_with(HashMap::new);
            for (k, v) in src_plugins {
                target_plugins.insert(k.clone(), *v);
            }
        }

        // Extra fields: merge
        for (k, v) in &source.extra {
            target.extra.insert(k.clone(), v.clone());
        }
    }

    /// Deep merge permissions
    fn merge_permissions(target: &mut Permissions, source: &Permissions) {
        // Lists: extend (not replace)
        if let Some(ref src_allow) = source.allow {
            let target_allow = target.allow.get_or_insert_with(Vec::new);
            for item in src_allow {
                if !target_allow.contains(item) {
                    target_allow.push(item.clone());
                }
            }
        }
        if let Some(ref src_deny) = source.deny {
            let target_deny = target.deny.get_or_insert_with(Vec::new);
            for item in src_deny {
                if !target_deny.contains(item) {
                    target_deny.push(item.clone());
                }
            }
        }
        if let Some(ref src_allow_bash) = source.allow_bash {
            let target_allow_bash = target.allow_bash.get_or_insert_with(Vec::new);
            for item in src_allow_bash {
                if !target_allow_bash.contains(item) {
                    target_allow_bash.push(item.clone());
                }
            }
        }
        if let Some(ref src_deny_bash) = source.deny_bash {
            let target_deny_bash = target.deny_bash.get_or_insert_with(Vec::new);
            for item in src_deny_bash {
                if !target_deny_bash.contains(item) {
                    target_deny_bash.push(item.clone());
                }
            }
        }

        // Booleans: override if present
        if source.auto_approve.is_some() {
            target.auto_approve = source.auto_approve;
        }
        if source.trust_project.is_some() {
            target.trust_project = source.trust_project;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_scalar_override() {
        let global = Settings {
            model: Some("opus".to_string()),
            ..Default::default()
        };
        let project = Settings {
            model: Some("sonnet".to_string()),
            ..Default::default()
        };

        let merged = MergedConfig::from_layers(Some(global), Some(project), None);
        assert_eq!(merged.merged.model, Some("sonnet".to_string()));
    }

    #[test]
    fn test_merge_env_combines() {
        let mut global_env = HashMap::new();
        global_env.insert("A".to_string(), "1".to_string());
        global_env.insert("B".to_string(), "2".to_string());

        let mut project_env = HashMap::new();
        project_env.insert("B".to_string(), "override".to_string());
        project_env.insert("C".to_string(), "3".to_string());

        let global = Settings {
            env: Some(global_env),
            ..Default::default()
        };
        let project = Settings {
            env: Some(project_env),
            ..Default::default()
        };

        let merged = MergedConfig::from_layers(Some(global), Some(project), None);
        let env = merged.merged.env.unwrap();

        assert_eq!(env.get("A"), Some(&"1".to_string()));
        assert_eq!(env.get("B"), Some(&"override".to_string()));
        assert_eq!(env.get("C"), Some(&"3".to_string()));
    }

    #[test]
    fn test_merge_permissions_extend() {
        let global = Settings {
            permissions: Some(Permissions {
                allow: Some(vec!["Read".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let project = Settings {
            permissions: Some(Permissions {
                allow: Some(vec!["Write".to_string()]),
                deny: Some(vec!["Bash".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let merged = MergedConfig::from_layers(Some(global), Some(project), None);
        let perms = merged.merged.permissions.unwrap();

        let allow = perms.allow.unwrap();
        assert!(allow.contains(&"Read".to_string()));
        assert!(allow.contains(&"Write".to_string()));

        let deny = perms.deny.unwrap();
        assert!(deny.contains(&"Bash".to_string()));
    }
}
