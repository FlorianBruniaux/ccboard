//! Invocation statistics for agents, commands, and skills

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics about agent/command/skill invocations across all sessions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvocationStats {
    /// Agent invocations (subagent_type -> count)
    /// Example: "technical-writer" -> 5
    pub agents: HashMap<String, usize>,

    /// Command invocations (/command -> count)
    /// Example: "/commit" -> 12
    pub commands: HashMap<String, usize>,

    /// Skill invocations (skill name -> count)
    /// Example: "pdf-generator" -> 3
    pub skills: HashMap<String, usize>,

    /// When stats were last computed
    pub last_computed: DateTime<Utc>,

    /// Number of sessions analyzed
    pub sessions_analyzed: usize,
}

impl InvocationStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            commands: HashMap::new(),
            skills: HashMap::new(),
            last_computed: Utc::now(),
            sessions_analyzed: 0,
        }
    }

    /// Get total number of invocations across all types
    pub fn total_invocations(&self) -> usize {
        self.agents.values().sum::<usize>()
            + self.commands.values().sum::<usize>()
            + self.skills.values().sum::<usize>()
    }

    /// Merge another InvocationStats into this one
    pub fn merge(&mut self, other: &InvocationStats) {
        for (name, count) in &other.agents {
            *self.agents.entry(name.clone()).or_insert(0) += count;
        }
        for (name, count) in &other.commands {
            *self.commands.entry(name.clone()).or_insert(0) += count;
        }
        for (name, count) in &other.skills {
            *self.skills.entry(name.clone()).or_insert(0) += count;
        }
        self.sessions_analyzed += other.sessions_analyzed;
        // Keep the most recent timestamp
        if other.last_computed > self.last_computed {
            self.last_computed = other.last_computed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_invocations() {
        let mut stats = InvocationStats::new();
        stats.agents.insert("technical-writer".to_string(), 5);
        stats.commands.insert("/commit".to_string(), 12);
        stats.skills.insert("pdf-generator".to_string(), 3);

        assert_eq!(stats.total_invocations(), 20);
    }

    #[test]
    fn test_merge() {
        let mut stats1 = InvocationStats::new();
        stats1.agents.insert("debugger".to_string(), 3);
        stats1.commands.insert("/commit".to_string(), 5);
        stats1.sessions_analyzed = 10;

        let mut stats2 = InvocationStats::new();
        stats2.agents.insert("debugger".to_string(), 2);
        stats2.agents.insert("code-reviewer".to_string(), 1);
        stats2.commands.insert("/commit".to_string(), 7);
        stats2.sessions_analyzed = 5;

        stats1.merge(&stats2);

        assert_eq!(stats1.agents.get("debugger"), Some(&5));
        assert_eq!(stats1.agents.get("code-reviewer"), Some(&1));
        assert_eq!(stats1.commands.get("/commit"), Some(&12));
        assert_eq!(stats1.sessions_analyzed, 15);
    }
}
