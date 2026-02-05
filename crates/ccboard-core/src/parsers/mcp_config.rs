//! Parser for Claude Desktop MCP server configuration
//!
//! Parses `~/.claude/claude_desktop_config.json` to extract MCP server definitions.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// MCP server configuration from claude_desktop_config.json
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfig {
    /// Map of server name to server configuration
    #[serde(rename = "mcpServers")]
    pub servers: HashMap<String, McpServer>,
}

/// Individual MCP server definition
#[derive(Debug, Clone, Deserialize)]
pub struct McpServer {
    /// Server type ("stdio", "http") - optional field from .mcp.json format
    #[serde(default)]
    #[serde(rename = "type")]
    pub server_type: Option<String>,

    /// Command to execute (e.g., "npx", "node") - required for stdio servers
    #[serde(default)]
    pub command: String,

    /// Arguments to pass to the command - optional for stdio servers
    #[serde(default)]
    pub args: Vec<String>,

    /// Optional environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// URL for HTTP servers - optional field from .mcp.json format
    #[serde(default)]
    pub url: Option<String>,

    /// Headers for HTTP servers - optional field from .mcp.json format
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

impl McpServer {
    /// Get display string for this server (command or URL)
    pub fn display_command(&self) -> String {
        if let Some(url) = &self.url {
            // HTTP server
            url.clone()
        } else if !self.command.is_empty() {
            // Stdio server
            if self.args.is_empty() {
                self.command.clone()
            } else {
                format!("{} {}", self.command, self.args.join(" "))
            }
        } else {
            "(unknown)".to_string()
        }
    }

    /// Check if this is an HTTP server
    pub fn is_http(&self) -> bool {
        self.url.is_some()
            || self
                .server_type
                .as_ref()
                .is_some_and(|t| t.to_lowercase() == "http")
    }
}

impl McpConfig {
    /// Load MCP configuration from claude_desktop_config.json and optional .mcp.json
    ///
    /// Merges configuration from:
    /// 1. Global: ~/.claude/claude_desktop_config.json
    /// 2. Project: <project>/.mcp.json (if project_path provided)
    ///
    /// Project config takes precedence for duplicate server names.
    ///
    /// Returns `None` if no config files exist (not an error - MCP is optional).
    /// Returns `Err` only if a file exists but cannot be parsed.
    pub fn load(claude_home: &Path) -> Result<Option<Self>> {
        Self::load_merged(claude_home, None)
    }

    /// Load MCP configuration with optional project-level config
    ///
    /// Merges configuration from:
    /// 1. Global: ~/.claude/claude_desktop_config.json
    /// 2. Project: <project>/.mcp.json (if project_path provided)
    ///
    /// Project config takes precedence for duplicate server names.
    pub fn load_merged(claude_home: &Path, project_path: Option<&Path>) -> Result<Option<Self>> {
        let mut global_config = Self::load_global(claude_home)?;
        let project_config = Self::load_project(project_path)?;

        match (global_config.as_mut(), project_config) {
            (Some(global), Some(project)) => {
                // Merge project servers into global (project takes precedence)
                for (name, server) in project.servers {
                    global.servers.insert(name, server);
                }
                Ok(Some(global.clone()))
            }
            (Some(global), None) => Ok(Some(global.clone())),
            (None, Some(project)) => Ok(Some(project)),
            (None, None) => Ok(None),
        }
    }

    /// Load global MCP config from claude_desktop_config.json
    fn load_global(claude_home: &Path) -> Result<Option<Self>> {
        let config_path = claude_home.join("claude_desktop_config.json");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read claude_desktop_config.json")?;

        let config: McpConfig =
            serde_json::from_str(&content).context("Failed to parse claude_desktop_config.json")?;

        Ok(Some(config))
    }

    /// Load project-level MCP config from .mcp.json
    fn load_project(project_path: Option<&Path>) -> Result<Option<Self>> {
        let Some(project) = project_path else {
            return Ok(None);
        };

        let config_path = project.join(".mcp.json");

        if !config_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&config_path).context("Failed to read .mcp.json")?;

        let config: McpConfig =
            serde_json::from_str(&content).context("Failed to parse .mcp.json")?;

        Ok(Some(config))
    }

    /// Get a formatted command string for display
    pub fn command_display(&self, name: &str) -> Option<String> {
        self.servers
            .get(name)
            .map(|server| server.display_command())
    }

    /// Check if a server has environment variables configured
    pub fn has_env(&self, name: &str) -> bool {
        self.servers
            .get(name)
            .map(|s| !s.env.is_empty())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mcp_config() {
        let json = r#"{
            "mcpServers": {
                "playwright": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-playwright"]
                },
                "serena": {
                    "command": "npx",
                    "args": ["-y", "@serenaai/serena-mcp"],
                    "env": {
                        "SERENA_PROJECT_PATH": "/path/to/project"
                    }
                }
            }
        }"#;

        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 2);
        assert!(config.servers.contains_key("playwright"));
        assert!(config.servers.contains_key("serena"));

        let playwright = &config.servers["playwright"];
        assert_eq!(playwright.command, "npx");
        assert_eq!(playwright.args.len(), 2);
        assert!(playwright.env.is_empty());

        let serena = &config.servers["serena"];
        assert_eq!(serena.env.len(), 1);
    }

    #[test]
    fn test_command_display() {
        let json = r#"{
            "mcpServers": {
                "test": {
                    "command": "node",
                    "args": ["server.js", "--port", "3000"]
                }
            }
        }"#;

        let config: McpConfig = serde_json::from_str(json).unwrap();
        let display = config.command_display("test").unwrap();
        assert_eq!(display, "node server.js --port 3000");
    }

    // Edge cases added for TDD compliance
    #[test]
    fn test_empty_config_parses_with_no_servers() {
        let json = r#"{"mcpServers": {}}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.servers.len(), 0);
    }

    #[test]
    fn test_invalid_json_returns_error() {
        let invalid_json = r#"{ invalid json }"#;
        let result: Result<McpConfig, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_mcpservers_field_returns_error() {
        let json = r#"{"other": "field"}"#;
        let result: Result<McpConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_server_without_command_is_valid() {
        // HTTP servers don't need command field
        let json = r#"{
            "mcpServers": {
                "http-server": {
                    "type": "http",
                    "url": "https://example.com/mcp"
                }
            }
        }"#;
        let result: Result<McpConfig, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.servers.len(), 1);
        let server = &config.servers["http-server"];
        assert!(server.is_http());
        assert_eq!(server.url.as_deref(), Some("https://example.com/mcp"));
    }

    #[test]
    fn test_command_display_returns_none_for_nonexistent_server() {
        let json = r#"{"mcpServers": {}}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert!(config.command_display("nonexistent").is_none());
    }

    #[test]
    fn test_has_env_returns_false_for_server_without_env() {
        let json = r#"{
            "mcpServers": {
                "no-env": {
                    "command": "test",
                    "args": []
                }
            }
        }"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert!(!config.has_env("no-env"));
    }

    #[test]
    fn test_has_env_returns_false_for_nonexistent_server() {
        let json = r#"{"mcpServers": {}}"#;
        let config: McpConfig = serde_json::from_str(json).unwrap();
        assert!(!config.has_env("nonexistent"));
    }

    #[test]
    fn test_load_returns_none_for_missing_file() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = McpConfig::load(temp_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_returns_error_for_invalid_json_file() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("claude_desktop_config.json");
        fs::write(&config_path, "{ invalid json }").unwrap();

        let result = McpConfig::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_merged_combines_global_and_project_configs() {
        use std::fs;
        use tempfile::TempDir;

        // Setup global config
        let claude_home = TempDir::new().unwrap();
        let global_config = r#"{
            "mcpServers": {
                "playwright": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-playwright"]
                },
                "serena": {
                    "command": "npx",
                    "args": ["-y", "@serenaai/serena-mcp"]
                }
            }
        }"#;
        fs::write(
            claude_home.path().join("claude_desktop_config.json"),
            global_config,
        )
        .unwrap();

        // Setup project config
        let project_dir = TempDir::new().unwrap();
        let project_config = r#"{
            "mcpServers": {
                "postgres-staging": {
                    "command": "bash",
                    "args": ["script.sh"]
                },
                "serena": {
                    "command": "uvx",
                    "args": ["--from", "git+https://github.com/oraios/serena"]
                }
            }
        }"#;
        fs::write(project_dir.path().join(".mcp.json"), project_config).unwrap();

        // Load merged config
        let merged = McpConfig::load_merged(claude_home.path(), Some(project_dir.path()))
            .unwrap()
            .unwrap();

        // Should have 3 servers total (playwright from global, postgres-staging from project, serena overridden by project)
        assert_eq!(merged.servers.len(), 3);
        assert!(merged.servers.contains_key("playwright"));
        assert!(merged.servers.contains_key("postgres-staging"));
        assert!(merged.servers.contains_key("serena"));

        // Verify serena was overridden by project config
        let serena = &merged.servers["serena"];
        assert_eq!(serena.command, "uvx");
        assert_eq!(serena.args[0], "--from");
    }

    #[test]
    fn test_load_merged_returns_global_only_when_no_project() {
        use std::fs;
        use tempfile::TempDir;

        let claude_home = TempDir::new().unwrap();
        let global_config = r#"{
            "mcpServers": {
                "playwright": {
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-playwright"]
                }
            }
        }"#;
        fs::write(
            claude_home.path().join("claude_desktop_config.json"),
            global_config,
        )
        .unwrap();

        let merged = McpConfig::load_merged(claude_home.path(), None)
            .unwrap()
            .unwrap();

        assert_eq!(merged.servers.len(), 1);
        assert!(merged.servers.contains_key("playwright"));
    }

    #[test]
    fn test_load_merged_returns_project_only_when_no_global() {
        use std::fs;
        use tempfile::TempDir;

        let claude_home = TempDir::new().unwrap(); // No global config
        let project_dir = TempDir::new().unwrap();
        let project_config = r#"{
            "mcpServers": {
                "postgres": {
                    "command": "bash",
                    "args": ["script.sh"]
                }
            }
        }"#;
        fs::write(project_dir.path().join(".mcp.json"), project_config).unwrap();

        let merged = McpConfig::load_merged(claude_home.path(), Some(project_dir.path()))
            .unwrap()
            .unwrap();

        assert_eq!(merged.servers.len(), 1);
        assert!(merged.servers.contains_key("postgres"));
    }
}
