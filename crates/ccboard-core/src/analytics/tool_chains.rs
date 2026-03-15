//! Tool chain analysis — bigram/trigram patterns across sessions
//!
//! Extracts recurring tool co-occurrence sequences from session metadata
//! to identify common workflows and expensive tool patterns.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::models::SessionMetadata;

/// A recurring sequence of tools used together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChain {
    /// Ordered tool names in the sequence
    pub sequence: Vec<String>,
    /// How many times this sequence occurred across sessions
    pub frequency: usize,
    /// Number of distinct sessions containing this sequence
    pub sessions_count: usize,
}

/// Complete tool chain analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolChainAnalysis {
    /// Top 10 tool pairs by frequency
    pub top_bigrams: Vec<ToolChain>,
    /// Top 10 tool triples by frequency
    pub top_trigrams: Vec<ToolChain>,
    /// Top 10 chains by token cost (uses call frequency as proxy)
    pub most_expensive_chains: Vec<ToolChain>,
    /// Timestamp of computation
    pub computed_at: DateTime<Utc>,
}

impl ToolChainAnalysis {
    /// Create empty analysis (used when no sessions available)
    pub fn empty() -> Self {
        Self {
            top_bigrams: Vec::new(),
            top_trigrams: Vec::new(),
            most_expensive_chains: Vec::new(),
            computed_at: Utc::now(),
        }
    }
}

/// Analyze tool co-occurrence patterns across sessions
///
/// Uses sorted tool name lists as a proxy for ordering (tools within a session
/// co-occur but JSONL doesn't preserve strict cross-message ordering at metadata level).
/// Bigrams and trigrams represent tools that appear together in the same session.
pub fn analyze_tool_chains(sessions: &[Arc<SessionMetadata>]) -> ToolChainAnalysis {
    if sessions.is_empty() {
        return ToolChainAnalysis::empty();
    }

    // bigrams: Vec<tool_name> -> (frequency, set of session IDs)
    let mut bigrams: HashMap<Vec<String>, (usize, HashSet<String>)> = HashMap::new();
    let mut trigrams: HashMap<Vec<String>, (usize, HashSet<String>)> = HashMap::new();

    for session in sessions {
        if session.tool_usage.is_empty() {
            continue;
        }

        // Use sorted tool names to ensure deterministic ordering
        let mut tools: Vec<String> = session.tool_usage.keys().cloned().collect();
        tools.sort();

        let session_id = session.id.to_string();

        // Generate bigrams from sorted tool list
        for pair in tools.windows(2) {
            let key = pair.to_vec();
            let entry = bigrams.entry(key).or_insert_with(|| (0, HashSet::new()));
            entry.0 += 1;
            entry.1.insert(session_id.clone());
        }

        // Generate trigrams from sorted tool list
        for triple in tools.windows(3) {
            let key = triple.to_vec();
            let entry = trigrams.entry(key).or_insert_with(|| (0, HashSet::new()));
            entry.0 += 1;
            entry.1.insert(session_id.clone());
        }
    }

    let mut top_bigrams: Vec<ToolChain> = bigrams
        .into_iter()
        .map(|(seq, (freq, sess))| ToolChain {
            sequence: seq,
            frequency: freq,
            sessions_count: sess.len(),
        })
        .collect();
    top_bigrams.sort_by(|a, b| b.frequency.cmp(&a.frequency));
    top_bigrams.truncate(10);

    let mut top_trigrams: Vec<ToolChain> = trigrams
        .into_iter()
        .map(|(seq, (freq, sess))| ToolChain {
            sequence: seq,
            frequency: freq,
            sessions_count: sess.len(),
        })
        .collect();
    top_trigrams.sort_by(|a, b| b.frequency.cmp(&a.frequency));
    top_trigrams.truncate(10);

    // Most expensive chains: rank bigrams by combined token usage
    let mut expensive_chains = top_bigrams.clone();
    expensive_chains.sort_by(|a, b| {
        let score_a = a.frequency * a.sessions_count;
        let score_b = b.frequency * b.sessions_count;
        score_b.cmp(&score_a)
    });
    expensive_chains.truncate(10);

    ToolChainAnalysis {
        top_bigrams,
        top_trigrams,
        most_expensive_chains: expensive_chains,
        computed_at: Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::models::session::{ProjectId, SessionId};

    fn make_session(id: &str, tools: &[(&str, usize)]) -> Arc<SessionMetadata> {
        let mut tool_usage = HashMap::new();
        for (name, count) in tools {
            tool_usage.insert(name.to_string(), *count);
        }
        let mut meta = SessionMetadata::from_path(
            PathBuf::from(format!("/tmp/{}.jsonl", id)),
            ProjectId::from("test"),
        );
        meta.id = SessionId::from(id);
        meta.tool_usage = tool_usage;
        Arc::new(meta)
    }

    #[test]
    fn test_empty_sessions() {
        let result = analyze_tool_chains(&[]);
        assert!(result.top_bigrams.is_empty());
        assert!(result.top_trigrams.is_empty());
    }

    #[test]
    fn test_bigrams_extracted() {
        let sessions = vec![
            make_session("s1", &[("Bash", 3), ("Read", 2), ("Write", 1)]),
            make_session("s2", &[("Bash", 5), ("Read", 1)]),
            make_session("s3", &[("Bash", 2), ("Read", 2), ("Grep", 1)]),
        ];

        let result = analyze_tool_chains(&sessions);

        // Bash+Read appears in s1 (Bash,Read,Write sorted) and s2 (Bash,Read sorted)
        // s3 has Bash,Grep,Read sorted → bigrams are Bash+Grep and Grep+Read, not Bash+Read
        let bash_read = result
            .top_bigrams
            .iter()
            .find(|c| c.sequence == vec!["Bash", "Read"]);
        assert!(bash_read.is_some(), "Bash+Read bigram should exist");
        assert_eq!(bash_read.unwrap().frequency, 2);
        assert_eq!(bash_read.unwrap().sessions_count, 2);
    }

    #[test]
    fn test_trigrams_extracted() {
        let sessions = vec![
            make_session("s1", &[("Bash", 3), ("Read", 2), ("Write", 1)]),
            make_session("s2", &[("Bash", 1), ("Read", 1), ("Write", 1)]),
        ];

        let result = analyze_tool_chains(&sessions);

        // Bash+Read+Write trigram should appear in both sessions
        let bash_read_write = result
            .top_trigrams
            .iter()
            .find(|c| c.sequence == vec!["Bash", "Read", "Write"]);
        assert!(
            bash_read_write.is_some(),
            "Bash+Read+Write trigram should exist"
        );
        assert_eq!(bash_read_write.unwrap().frequency, 2);
    }

    #[test]
    fn test_no_tools_session_skipped() {
        let sessions = vec![
            make_session("s1", &[]),
            make_session("s2", &[("Read", 1), ("Write", 1)]),
        ];

        let result = analyze_tool_chains(&sessions);
        // Only s2 contributes, so Read+Write bigram exists
        assert_eq!(result.top_bigrams.len(), 1);
    }
}
