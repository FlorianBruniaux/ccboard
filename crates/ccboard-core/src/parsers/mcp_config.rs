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
    /// Command to execute (e.g., "npx", "node")
    pub command: String,

    /// Arguments to pass to the command
    pub args: Vec<String>,

    /// Optional environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl McpConfig {
    /// Load MCP configuration from claude_desktop_config.json
    ///
    /// Returns `None` if the file doesn't exist (not an error - MCP is optional).
    /// Returns `Err` only if the file exists but cannot be parsed.
    pub fn load(claude_home: &Path) -> Result<Option<Self>> {
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

    /// Get a formatted command string for display
    pub fn command_display(&self, name: &str) -> Option<String> {
        self.servers
            .get(name)
            .map(|server| format!("{} {}", server.command, server.args.join(" ")))
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
    fn test_missing_required_command_field_returns_error() {
        let json = r#"{
            "mcpServers": {
                "broken": {
                    "args": ["test"]
                }
            }
        }"#;
        let result: Result<McpConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
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
}
