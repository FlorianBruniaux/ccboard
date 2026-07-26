//! Parser for extracting agent/command/skill invocations from session files

use crate::error::CoreError;
use crate::models::{InvocationStats, ProjectId, SessionLine, SkillInvocationEvent};
use crate::parsers::session_index::SessionIndexParser;
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

/// A session file is a subagent transcript when it lives under a `subagents/`
/// directory next to its parent session file. Subagent transcripts reuse their
/// parent's sessionId in their JSON content, so path (not sessionId) is the
/// only reliable identity signal for deduplication.
fn is_subagent_path(path: &Path) -> bool {
    path.components().any(|c| c.as_os_str() == "subagents")
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

        let project = ProjectId::from(SessionIndexParser::new().extract_project_path(path));
        let is_subagent = is_subagent_path(path);

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

                        // Proportional token attribution for Skill blocks, mirroring the
                        // base/remainder split in session_index.rs::scan_session_uncached
                        // exactly (see test_parsers_agree_on_token_totals). Only the key
                        // differs: credit goes to the skill name from `input.skill`,
                        // never the literal "Skill".
                        let message_tools: Vec<&str> = content_array
                            .iter()
                            .filter(|item| {
                                item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                            })
                            .filter_map(|item| item.get("name").and_then(|n| n.as_str()))
                            .collect();

                        if !message_tools.is_empty() {
                            let usage_opt = session_line.usage.as_ref().or(message.usage.as_ref());
                            let effective_model =
                                session_line.model.clone().or_else(|| message.model.clone());

                            let n = message_tools.len() as u64;
                            let (total, input, output, cache_read, cache_write) = usage_opt
                                .map(|u| {
                                    (
                                        u.total(),
                                        u.input_tokens,
                                        u.output_tokens,
                                        u.cache_read_tokens,
                                        u.cache_write_tokens,
                                    )
                                })
                                .unwrap_or((0, 0, 0, 0, 0));

                            let split = |value: u64, index: u64| {
                                (value / n) + if index == 0 { value % n } else { 0 }
                            };

                            let tool_use_blocks = content_array.iter().filter(|item| {
                                item.get("type").and_then(|t| t.as_str()) == Some("tool_use")
                            });

                            for (i, block) in tool_use_blocks.enumerate() {
                                let i = i as u64;
                                let Some(obj) = block.as_object() else {
                                    continue;
                                };
                                if obj.get("name").and_then(|v| v.as_str()) != Some("Skill") {
                                    continue;
                                }
                                let Some(skill_name) = obj
                                    .get("input")
                                    .and_then(|v| v.as_object())
                                    .and_then(|input| input.get("skill"))
                                    .and_then(|v| v.as_str())
                                else {
                                    continue;
                                };

                                stats.skill_events.push(SkillInvocationEvent {
                                    skill: skill_name.to_string(),
                                    timestamp: session_line.timestamp,
                                    project: project.clone(),
                                    session_path: path.to_path_buf(),
                                    is_subagent,
                                    tokens: split(total, i),
                                    model: effective_model.clone(),
                                    tokens_input: split(input, i),
                                    tokens_output: split(output, i),
                                    tokens_cache_read: split(cache_read, i),
                                    tokens_cache_write: split(cache_write, i),
                                });
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
    #[allow(dead_code)]
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
    use tempfile::{tempdir, NamedTempFile};

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

    // ── Skill invocation event tests (Phase 1) ─────────────────────────────

    #[tokio::test]
    async fn test_skill_event_records_tokens() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Skill","input":{{"skill":"pdf-generator"}}}}]}},"usage":{{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":10,"cache_creation_input_tokens":5}},"timestamp":"2026-01-01T00:00:00Z"}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(stats.skill_events.len(), 1);
        let event = &stats.skill_events[0];
        assert_eq!(event.skill, "pdf-generator");
        assert_eq!(event.tokens, 165);
        assert_eq!(event.tokens_input, 100);
        assert_eq!(event.tokens_output, 50);
        assert_eq!(event.tokens_cache_read, 10);
        assert_eq!(event.tokens_cache_write, 5);
        assert!(!event.is_subagent);
        assert!(event.timestamp.is_some());
    }

    #[tokio::test]
    async fn test_prose_mention_not_counted() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"text","text":"utilise le skill flow-lean"}}]}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert!(
            stats.skill_events.is_empty(),
            "textual mentions must never produce skill events"
        );
        assert!(stats.skills.is_empty());
    }

    #[tokio::test]
    async fn test_subagent_path_dedup() {
        let dir = tempdir().unwrap();
        let parent_path = dir.path().join("session-abc.jsonl");
        let subagent_dir = dir.path().join("subagents");
        std::fs::create_dir_all(&subagent_dir).unwrap();
        let subagent_path = subagent_dir.join("agent-x.jsonl");

        let line = r#"{"type":"assistant","sessionId":"shared-session-id","message":{"role":"assistant","content":[{"type":"tool_use","name":"Skill","input":{"skill":"pdf-generator"}}]},"usage":{"input_tokens":10,"output_tokens":10}}"#;
        std::fs::write(&parent_path, format!("{line}\n")).unwrap();
        std::fs::write(&subagent_path, format!("{line}\n")).unwrap();

        let parser = InvocationParser::new();
        let parent_stats = parser.scan_session(&parent_path).await.unwrap();
        let subagent_stats = parser.scan_session(&subagent_path).await.unwrap();

        assert_eq!(parent_stats.skill_events.len(), 1);
        assert_eq!(subagent_stats.skill_events.len(), 1);
        assert!(
            !parent_stats.skill_events[0].is_subagent,
            "parent transcript must not be flagged as subagent"
        );
        assert!(
            subagent_stats.skill_events[0].is_subagent,
            "file under subagents/ must be flagged as subagent"
        );
        assert_ne!(
            parent_stats.skill_events[0].session_path, subagent_stats.skill_events[0].session_path,
            "path is the identity key, never the shared sessionId in content"
        );
    }

    #[tokio::test]
    async fn test_multi_skill_same_message() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Skill","input":{{"skill":"skill-a"}}}},{{"type":"tool_use","name":"Skill","input":{{"skill":"skill-b"}}}}]}},"usage":{{"input_tokens":100,"output_tokens":1}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        // total = 101 tokens, n = 2 tools -> base = 50, remainder = 1 to block 0
        assert_eq!(stats.skill_events.len(), 2);
        assert_eq!(stats.skill_events[0].skill, "skill-a");
        assert_eq!(stats.skill_events[0].tokens, 51);
        assert_eq!(stats.skill_events[1].skill, "skill-b");
        assert_eq!(stats.skill_events[1].tokens, 50);
        assert_eq!(
            stats.skill_events[0].tokens + stats.skill_events[1].tokens,
            101
        );
    }

    #[tokio::test]
    async fn test_namespaced_skill_name() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Skill","input":{{"skill":"superpowers:brainstorming"}}}}]}}}}"#
        )
        .unwrap();

        let parser = InvocationParser::new();
        let stats = parser.scan_session(file.path()).await.unwrap();

        assert_eq!(stats.skill_events.len(), 1);
        assert_eq!(stats.skill_events[0].skill, "superpowers:brainstorming");
        assert_eq!(stats.skills.get("superpowers:brainstorming"), Some(&1));
        assert!(
            !stats.skills.contains_key("brainstorming"),
            "namespaced skill must never be split on ':'"
        );
    }

    #[tokio::test]
    async fn test_parsers_agree_on_token_totals() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Read","input":{{}}}},{{"type":"tool_use","name":"Skill","input":{{"skill":"skill-a"}}}}]}},"usage":{{"input_tokens":100,"output_tokens":1}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"role":"assistant","content":[{{"type":"tool_use","name":"Skill","input":{{"skill":"skill-b"}}}},{{"type":"tool_use","name":"Skill","input":{{"skill":"skill-c"}}}},{{"type":"tool_use","name":"Bash","input":{{}}}}]}},"usage":{{"input_tokens":250,"output_tokens":7}}}}"#
        )
        .unwrap();

        let invocation_parser = InvocationParser::new();
        let invocation_stats = invocation_parser.scan_session(file.path()).await.unwrap();
        let skill_events_total: u64 = invocation_stats.skill_events.iter().map(|e| e.tokens).sum();

        let session_parser = SessionIndexParser::new();
        let metadata = session_parser.scan_session(file.path()).await.unwrap();
        let session_index_skill_total =
            metadata.tool_token_usage.get("Skill").copied().unwrap_or(0);

        assert!(
            session_index_skill_total > 0,
            "sanity check: fixture must produce non-trivial Skill token usage"
        );
        assert_eq!(
            skill_events_total, session_index_skill_total,
            "sum of per-skill token shares must equal session_index's tool_token_usage[\"Skill\"]"
        );
    }
}
