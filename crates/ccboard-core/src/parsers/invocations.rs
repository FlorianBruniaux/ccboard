//! Parser for extracting agent/command/skill invocations from session files

use crate::error::CoreError;
use crate::models::{InvocationStats, SessionLine};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, trace};

/// Regex for detecting command invocations (e.g., /commit, /help)
fn command_regex() -> &'static Regex {
    static COMMAND_RE: OnceLock<Regex> = OnceLock::new();
    COMMAND_RE.get_or_init(|| Regex::new(r"^/([a-z][a-z0-9-]*)").unwrap())
}

/// Parser for invocation statistics
#[derive(Debug)]
pub struct InvocationParser {
    /// Maximum lines to scan per session (circuit breaker)
    max_lines: usize,
}

impl Default for InvocationParser {
    fn default() -> Self {
        Self { max_lines: 50_000 }
    }
}

impl InvocationParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a single session file for invocations
    pub async fn scan_session(&self, path: &Path) -> Result<InvocationStats, CoreError> {
        let mut stats = InvocationStats::new();
        stats.sessions_analyzed = 1;

        let file = File::open(path).await.map_err(|e| {
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

        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut line_number = 0;

        while let Some(line_result) = lines.next_line().await.map_err(|e| CoreError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })? {
            line_number += 1;

            // Circuit breaker
            if line_number > self.max_lines {
                debug!(
                    path = %path.display(),
                    lines = line_number,
                    "Invocation scan hit line limit"
                );
                break;
            }

            // Parse line (skip malformed)
            let session_line: SessionLine = match serde_json::from_str(&line_result) {
                Ok(l) => l,
                Err(_) => continue,
            };

            // Detect agents and skills from assistant messages (tool_use content)
            if let Some(ref message) = session_line.message {
                if let Some(ref content) = message.content {
                    // Content is now Value (can be String or Array)
                    if let Some(content_array) = content.as_array() {
                        for item in content_array {
                            if let Some(obj) = item.as_object() {
                                // Check for Task tool
                                if obj.get("name").and_then(|v| v.as_str()) == Some("Task") {
                                    if let Some(input) =
                                        obj.get("input").and_then(|v| v.as_object())
                                    {
                                        if let Some(agent_type) =
                                            input.get("subagent_type").and_then(|v| v.as_str())
                                        {
                                            *stats
                                                .agents
                                                .entry(agent_type.to_string())
                                                .or_insert(0) += 1;
                                            trace!(agent = agent_type, "Detected agent invocation");
                                        }
                                    }
                                }
                                // Check for Skill tool
                                if obj.get("name").and_then(|v| v.as_str()) == Some("Skill") {
                                    if let Some(input) =
                                        obj.get("input").and_then(|v| v.as_object())
                                    {
                                        if let Some(skill_name) =
                                            input.get("skill").and_then(|v| v.as_str())
                                        {
                                            *stats
                                                .skills
                                                .entry(skill_name.to_string())
                                                .or_insert(0) += 1;
                                            trace!(skill = skill_name, "Detected skill invocation");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Detect commands: user messages starting with /
            if session_line.line_type == "user" {
                if let Some(ref message) = session_line.message {
                    if let Some(ref content) = message.content {
                        // Extract text from Value (String or Array)
                        let text = match content {
                            serde_json::Value::String(s) => s.as_str(),
                            serde_json::Value::Array(blocks) => {
                                // For array, try to get first text block
                                blocks
                                    .first()
                                    .and_then(|block| block.get("text"))
                                    .and_then(|t| t.as_str())
                                    .unwrap_or("")
                            }
                            _ => "",
                        };

                        if let Some(caps) = command_regex().captures(text) {
                            let command = format!("/{}", &caps[1]);
                            *stats.commands.entry(command.clone()).or_insert(0) += 1;
                            trace!(command, "Detected command invocation");
                        }
                    }
                }
            }
        }

        debug!(
            path = %path.display(),
            agents = stats.agents.len(),
            commands = stats.commands.len(),
            skills = stats.skills.len(),
            "Invocation scan complete"
        );

        Ok(stats)
    }

    /// Extract invocations from a content string
    fn extract_invocations(&self, content: &str, stats: &mut InvocationStats) {
        // Detect commands in text
        if let Some(caps) = command_regex().captures(content) {
            let command = format!("/{}", &caps[1]);
            *stats.commands.entry(command).or_insert(0) += 1;
        }
    }

    /// Scan multiple session files and aggregate stats
    pub async fn scan_sessions(&self, paths: &[impl AsRef<Path>]) -> InvocationStats {
        let mut aggregated = InvocationStats::new();

        for path in paths {
            match self.scan_session(path.as_ref()).await {
                Ok(stats) => aggregated.merge(&stats),
                Err(e) => {
                    trace!(
                        path = %path.as_ref().display(),
                        error = %e,
                        "Failed to scan session for invocations"
                    );
                }
            }
        }

        debug!(
            sessions = aggregated.sessions_analyzed,
            total = aggregated.total_invocations(),
            "Aggregated invocation stats"
        );

        aggregated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_detect_agent_invocation() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "message": {{"content": [{{"type":"tool_use","name":"Task","input":{{"subagent_type":"technical-writer","description":"Create docs"}}}}]}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(stats.agents.get("technical-writer"), Some(&1));
        assert_eq!(stats.sessions_analyzed, 1);
    }

    #[tokio::test]
    async fn test_detect_skill_invocation() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "assistant", "message": {{"content": [{{"type":"tool_use","name":"Skill","input":{{"skill":"pdf-generator"}}}}]}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(stats.skills.get("pdf-generator"), Some(&1));
    }

    #[tokio::test]
    async fn test_detect_command_invocation() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type": "user", "message": {{"content": "/commit -m \"Fix bug\""}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type": "user", "message": {{"content": "/help"}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(stats.commands.get("/commit"), Some(&1));
        assert_eq!(stats.commands.get("/help"), Some(&1));
    }

    #[test]
    fn test_command_regex() {
        let re = command_regex();
        assert!(re.is_match("/commit"));
        assert!(re.is_match("/help"));
        assert!(re.is_match("/review-pr"));
        assert!(!re.is_match("not a command"));
        assert!(!re.is_match("/ space"));
    }

    #[tokio::test]
    async fn test_aggregation() {
        let mut file1 = NamedTempFile::new().unwrap();
        writeln!(
            file1,
            r#"{{"type": "user", "message": {{"content": "/commit"}}}}"#
        )
        .unwrap();

        let mut file2 = NamedTempFile::new().unwrap();
        writeln!(
            file2,
            r#"{{"type": "user", "message": {{"content": "/commit"}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let paths = vec![file1.path(), file2.path()];
        let stats = parser.scan_sessions(&paths).await;

        assert_eq!(stats.commands.get("/commit"), Some(&2));
        assert_eq!(stats.sessions_analyzed, 2);
    }
}
