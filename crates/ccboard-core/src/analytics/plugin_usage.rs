//! Plugin usage analytics module
//!
//! Aggregates tool usage from SessionMetadata to classify and rank plugins:
//! - Skills (.claude/skills/)
//! - Commands (.claude/commands/)
//! - MCP Servers (mcp__server__tool format)
//! - Agents (Task tool with subagent_type)
//! - Native Tools (Read, Write, Edit, Bash, etc.)
//!
//! Provides metrics on total invocations, cost attribution, dead code detection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::models::session::SessionMetadata;
use crate::pricing::calculate_cost;

/// Plugin classification by origin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginType {
    /// User-defined skill from .claude/skills/
    Skill,
    /// User-defined command from .claude/commands/
    Command,
    /// Spawned agent via Task tool
    Agent,
    /// MCP server tool (mcp__server__tool format)
    McpServer,
    /// Built-in Claude Code tool
    NativeTool,
}

impl PluginType {
    /// Icon for TUI/Web display
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Skill => "🎓",
            Self::Command => "⚡",
            Self::Agent => "🤖",
            Self::McpServer => "🔌",
            Self::NativeTool => "🛠️",
        }
    }

    /// Human-readable label
    pub fn label(&self) -> &'static str {
        match self {
            Self::Skill => "Skill",
            Self::Command => "Command",
            Self::Agent => "Agent",
            Self::McpServer => "MCP Server",
            Self::NativeTool => "Native Tool",
        }
    }
}

/// Usage statistics for a single plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUsage {
    /// Plugin identifier (e.g., "rust-expert", "mcp__context7__search")
    pub name: String,
    /// Classification
    pub plugin_type: PluginType,
    /// Total invocations across all sessions
    pub total_invocations: usize,
    /// Session IDs where this plugin was used
    pub sessions_used: Vec<String>,
    /// Total cost attributed to this plugin ($)
    pub total_cost: f64,
    /// Average tokens per invocation
    pub avg_tokens_per_invocation: u64,
    /// First usage timestamp
    pub first_seen: DateTime<Utc>,
    /// Last usage timestamp
    pub last_seen: DateTime<Utc>,
}

impl PluginUsage {
    /// Create new plugin usage record
    fn new(
        name: String,
        plugin_type: PluginType,
        invocations: usize,
        session_id: String,
        cost: f64,
        avg_tokens: u64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            name,
            plugin_type,
            total_invocations: invocations,
            sessions_used: vec![session_id],
            total_cost: cost,
            avg_tokens_per_invocation: avg_tokens,
            first_seen: timestamp,
            last_seen: timestamp,
        }
    }

    /// Merge another usage record into this one
    fn merge(&mut self, other: &Self) {
        self.total_invocations += other.total_invocations;
        if !self.sessions_used.contains(&other.sessions_used[0]) {
            self.sessions_used.push(other.sessions_used[0].clone());
        }
        self.total_cost += other.total_cost;
        if other.first_seen < self.first_seen {
            self.first_seen = other.first_seen;
        }
        if other.last_seen > self.last_seen {
            self.last_seen = other.last_seen;
        }
    }
}

/// Complete plugin analytics data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAnalytics {
    /// Total plugin count (active + dead)
    pub total_plugins: usize,
    /// Active plugin count (used at least once)
    pub active_plugins: usize,
    /// Dead plugins (defined but never used)
    pub dead_plugins: Vec<String>,
    /// All plugin usage records
    pub plugins: Vec<PluginUsage>,
    /// Top 10 by usage
    pub top_by_usage: Vec<PluginUsage>,
    /// Top 10 by cost
    pub top_by_cost: Vec<PluginUsage>,
    /// Timestamp of computation
    pub computed_at: DateTime<Utc>,
}

impl PluginAnalytics {
    /// Create empty analytics
    pub fn empty() -> Self {
        Self {
            total_plugins: 0,
            active_plugins: 0,
            dead_plugins: Vec::new(),
            plugins: Vec::new(),
            top_by_usage: Vec::new(),
            top_by_cost: Vec::new(),
            computed_at: Utc::now(),
        }
    }
}

/// Classify plugin by name and context
fn classify_plugin(name: &str, skills: &[String], commands: &[String]) -> PluginType {
    // MCP server format: mcp__server__tool
    if name.starts_with("mcp__") {
        return PluginType::McpServer;
    }

    // Agent spawning via Task tool
    if name == "Task" {
        return PluginType::Agent;
    }

    // Check against known skills (case-insensitive)
    let lower_name = name.to_lowercase();
    if skills
        .iter()
        .any(|s| lower_name.contains(&s.to_lowercase()))
    {
        return PluginType::Skill;
    }

    // Check against known commands
    if commands
        .iter()
        .any(|c| lower_name.contains(&c.to_lowercase()))
    {
        return PluginType::Command;
    }

    // Known native tools
    const NATIVE_TOOLS: &[&str] = &[
        "Read",
        "Write",
        "Edit",
        "MultiEdit",
        "Bash",
        "Grep",
        "Glob",
        "WebSearch",
        "WebFetch",
        "NotebookEdit",
        "AskUserQuestion",
        "EnterPlanMode",
        "ExitPlanMode",
        "TaskCreate",
        "TaskUpdate",
        "TaskGet",
        "TaskList",
        "TeamCreate",
        "TeamDelete",
        "SendMessage",
        "Skill",
    ];

    if NATIVE_TOOLS.contains(&name) {
        return PluginType::NativeTool;
    }

    // Default: treat as native tool
    PluginType::NativeTool
}

/// Aggregate plugin usage from sessions
///
/// # Arguments
/// - `sessions`: All sessions to analyze
/// - `available_skills`: Skill names from .claude/skills/*.md
/// - `available_commands`: Command names from .claude/commands/*.md
///
/// # Returns
/// Complete plugin analytics with rankings and dead code detection
pub fn aggregate_plugin_usage(
    sessions: &[Arc<SessionMetadata>],
    available_skills: &[String],
    available_commands: &[String],
) -> PluginAnalytics {
    let mut usage_map: HashMap<String, PluginUsage> = HashMap::new();

    // Aggregate tool usage from all sessions
    for session in sessions {
        // Calculate session cost (proportional attribution)
        // Use first model from models_used, or default to sonnet-4.5
        let model = session
            .models_used
            .first()
            .map(|s| s.as_str())
            .unwrap_or("sonnet-4.5");
        let session_cost = calculate_cost(
            model,
            session.input_tokens,
            session.output_tokens,
            session.cache_creation_tokens,
            session.cache_read_tokens,
        );
        let session_tokens = session.total_tokens;

        // Skip sessions with no tool usage
        if session.tool_usage.is_empty() {
            continue;
        }

        // Total tool calls in session (for proportional cost)
        let total_calls: usize = session.tool_usage.values().sum();
        if total_calls == 0 {
            continue;
        }

        for (tool_name, call_count) in &session.tool_usage {
            let plugin_type = classify_plugin(tool_name, available_skills, available_commands);

            // Proportional cost attribution
            let tool_cost = session_cost * (*call_count as f64 / total_calls as f64);

            // Average tokens per call (rough approximation)
            let avg_tokens = if *call_count > 0 {
                session_tokens / *call_count as u64
            } else {
                0
            };

            // Use first/last timestamp from session
            let timestamp = session
                .first_timestamp
                .or(session.last_timestamp)
                .unwrap_or_else(Utc::now);

            usage_map
                .entry(tool_name.clone())
                .and_modify(|usage| {
                    usage.total_invocations += call_count;
                    if !usage.sessions_used.contains(&session.id.to_string()) {
                        usage.sessions_used.push(session.id.to_string());
                    }
                    usage.total_cost += tool_cost;

                    // Update timestamps
                    if let Some(first_ts) = session.first_timestamp {
                        if first_ts < usage.first_seen {
                            usage.first_seen = first_ts;
                        }
                    }
                    if let Some(last_ts) = session.last_timestamp {
                        if last_ts > usage.last_seen {
                            usage.last_seen = last_ts;
                        }
                    }

                    // Recalculate average tokens (weighted by invocations)
                    usage.avg_tokens_per_invocation = (usage.avg_tokens_per_invocation
                        * (usage.total_invocations - call_count) as u64
                        + avg_tokens * *call_count as u64)
                        / usage.total_invocations as u64;
                })
                .or_insert_with(|| {
                    PluginUsage::new(
                        tool_name.clone(),
                        plugin_type,
                        *call_count,
                        session.id.to_string(),
                        tool_cost,
                        avg_tokens,
                        timestamp,
                    )
                });
        }
    }

    // Identify dead code (defined but never used)
    let used_names: HashSet<_> = usage_map.keys().map(|s| s.to_lowercase()).collect();
    let dead_plugins: Vec<String> = available_skills
        .iter()
        .chain(available_commands.iter())
        .filter(|name| !used_names.contains(&name.to_lowercase()))
        .cloned()
        .collect();

    // Convert to sorted vector
    let mut plugins: Vec<_> = usage_map.into_values().collect();
    plugins.sort_by(|a, b| b.total_invocations.cmp(&a.total_invocations));

    // Top 10 by usage
    let top_by_usage = plugins.iter().take(10).cloned().collect();

    // Top 10 by cost
    let mut top_by_cost = plugins.clone();
    top_by_cost.sort_by(|a, b| {
        b.total_cost
            .partial_cmp(&a.total_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_by_cost = top_by_cost.into_iter().take(10).collect();

    PluginAnalytics {
        total_plugins: plugins.len() + dead_plugins.len(),
        active_plugins: plugins.len(),
        dead_plugins,
        plugins,
        top_by_usage,
        top_by_cost,
        computed_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session::SessionId;

    #[test]
    fn test_classify_plugin() {
        let skills = vec!["rust-expert".to_string(), "boldguy-draft".to_string()];
        let commands = vec!["commit".to_string()];

        assert_eq!(
            classify_plugin("rust-expert", &skills, &commands),
            PluginType::Skill
        );
        assert_eq!(
            classify_plugin("mcp__context7__search", &skills, &commands),
            PluginType::McpServer
        );
        assert_eq!(
            classify_plugin("Read", &skills, &commands),
            PluginType::NativeTool
        );
        assert_eq!(
            classify_plugin("Task", &skills, &commands),
            PluginType::Agent
        );
    }

    #[test]
    fn test_aggregate_plugin_usage() {
        use crate::models::session::ProjectId;
        use std::path::PathBuf;

        // Create test sessions with tool_usage
        let mut tool_usage1 = HashMap::new();
        tool_usage1.insert("rust-expert".to_string(), 5);
        tool_usage1.insert("mcp__context7__search".to_string(), 3);

        let mut session1 = SessionMetadata::from_path(
            PathBuf::from("/tmp/test-session.jsonl"),
            "test-project".into(),
        );
        session1.tool_usage = tool_usage1;
        session1.total_tokens = 10000;
        session1.first_timestamp = Some(Utc::now());
        session1.last_timestamp = Some(Utc::now());

        let sessions = vec![Arc::new(session1)];
        let skills = vec!["rust-expert".to_string()];
        let commands = vec![];

        let analytics = aggregate_plugin_usage(&sessions, &skills, &commands);

        assert_eq!(analytics.active_plugins, 2);
        assert_eq!(analytics.plugins[0].name, "rust-expert");
        assert_eq!(analytics.plugins[0].total_invocations, 5);
        assert_eq!(analytics.plugins[0].plugin_type, PluginType::Skill);
        assert_eq!(analytics.plugins[1].name, "mcp__context7__search");
        assert_eq!(analytics.plugins[1].plugin_type, PluginType::McpServer);
    }

    #[test]
    fn test_dead_code_detection() {
        let sessions = vec![];
        let skills = vec!["rust-expert".to_string(), "unused-skill".to_string()];
        let commands = vec!["commit".to_string(), "never-used".to_string()];

        let analytics = aggregate_plugin_usage(&sessions, &skills, &commands);

        assert_eq!(analytics.active_plugins, 0);
        assert_eq!(analytics.dead_plugins.len(), 4); // All skills + commands unused
        assert!(analytics.dead_plugins.contains(&"unused-skill".to_string()));
        assert!(analytics.dead_plugins.contains(&"never-used".to_string()));
    }

    #[test]
    fn test_empty_sessions() {
        let sessions = vec![];
        let skills = vec![];
        let commands = vec![];

        let analytics = aggregate_plugin_usage(&sessions, &skills, &commands);

        assert_eq!(analytics.active_plugins, 0);
        assert_eq!(analytics.total_plugins, 0);
        assert!(analytics.plugins.is_empty());
    }
}
