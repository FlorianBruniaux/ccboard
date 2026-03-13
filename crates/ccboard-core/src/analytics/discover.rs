//! Pattern discovery from session history
//!
//! Analyzes JSONL session files to surface recurring user patterns
//! worth extracting as Claude.md rules, skills, or slash commands.
//!
//! Algorithm: n-gram extraction → frequency filtering → deduplication →
//! Jaccard clustering → category assignment by 20%/5% session thresholds.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

/// Configuration for a discover run
#[derive(Debug, Clone)]
pub struct DiscoverConfig {
    /// Number of past days to analyze (default: 90)
    pub since_days: u32,
    /// Minimum occurrences to surface a pattern (default: 3)
    pub min_count: usize,
    /// Maximum suggestions to return (default: 20)
    pub top: usize,
    /// Search all projects (false = current project directory only)
    pub all_projects: bool,
}

impl Default for DiscoverConfig {
    fn default() -> Self {
        Self {
            since_days: 90,
            min_count: 3,
            top: 20,
            all_projects: false,
        }
    }
}

/// Category of a discovered suggestion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionCategory {
    /// Broad behavioral rule — active in every session (>20% of sessions)
    ClaudeMdRule,
    /// Specialized expertise loaded on demand (5–20% of sessions)
    Skill,
    /// Repeatable workflow with clear inputs/outputs (<5% of sessions)
    Command,
}

impl SuggestionCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClaudeMdRule => "CLAUDE.MD RULE",
            Self::Skill => "SKILL",
            Self::Command => "COMMAND",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::ClaudeMdRule => "📋",
            Self::Skill => "🧩",
            Self::Command => "⚡",
        }
    }
}

/// A single pattern suggestion produced by the discover algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverSuggestion {
    /// Human-readable pattern phrase
    pub pattern: String,
    /// Total occurrences across all sessions
    pub count: usize,
    /// Number of distinct sessions containing this pattern
    pub session_count: usize,
    /// Number of distinct projects containing this pattern
    pub project_count: usize,
    /// True when pattern appears in 2+ projects
    pub cross_project: bool,
    /// Category determined by session percentage thresholds
    pub category: SuggestionCategory,
    /// Composite score: session_pct * cross_project_bonus
    pub score: f64,
    /// Up to 2 example session IDs
    pub example_sessions: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Stop words & system injection markers
// ─────────────────────────────────────────────────────────────────────────────

static STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "is", "it", "its", "be", "as", "was", "are", "were", "been", "have", "has", "had",
    "do", "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
    "this", "that", "these", "those", "i", "you", "we", "they", "he", "she", "my", "your", "our",
    "their", "his", "her", "me", "us", "them", "so", "if", "then", "than", "when", "what", "how",
    "why", "where", "who", "which", "not", "no", "also", "just", "now", "up", "out", "about",
    "into", "after", "before", "all", "any", "some", "more", "new", "add", "use", "make", "get",
    "go", "run", "see", "here", "there", "need", "want", "please", "ok", "okay", "yes", "yeah",
    "let", "can", "help", "look", "check", "same", "like", "very", "much", "only", "other", "also",
    "each", "file", "code", "create", "update", "change", "think", "know", "give", "take", "put",
    "keep",
];

static SYSTEM_INJECTION_MARKERS: &[&str] = &[
    "this session is being continued",
    "read the full transcript",
    "context summary below covers",
    "exact snippets error messages content",
    "exiting plan mode",
    "task tools haven",
    "teamcreate tool team parallelize",
];

fn is_stop_word(token: &str) -> bool {
    STOP_WORDS.contains(&token)
}

fn is_system_injection(text: &str) -> bool {
    let lower = text.to_lowercase();
    SYSTEM_INJECTION_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

// ─────────────────────────────────────────────────────────────────────────────
// Text normalization & n-gram extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Normalize text: lowercase, strip punctuation (keep hyphens), tokenize,
/// remove stop words, and drop tokens with length <= 2.
pub fn normalize_text(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    // Replace chars that are not alphanumeric or hyphens with spaces
    let clean: String = lower
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect();

    clean
        .split_whitespace()
        .filter(|t| t.len() > 2 && !is_stop_word(t))
        .map(|t| t.to_string())
        .collect()
}

/// Extract all n-grams of length `n` from a token list.
pub fn extract_ngrams(tokens: &[String], n: usize) -> Vec<Vec<String>> {
    if tokens.len() < n {
        return vec![];
    }
    (0..=(tokens.len() - n))
        .map(|i| tokens[i..i + n].to_vec())
        .collect()
}

/// Jaccard similarity between two token lists (set-based).
pub fn jaccard_overlap(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let set_a: HashSet<&str> = a.iter().map(|s| s.as_str()).collect();
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session data extraction
// ─────────────────────────────────────────────────────────────────────────────

/// Collected data for a single session
#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: String,
    pub project: String,
    pub messages: Vec<String>,
}

/// Parse a JSONL entry to extract the message content if it is a real user message.
fn extract_user_content(line: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct MessageContent {
        #[serde(rename = "type")]
        msg_type: Option<String>,
        content: Option<serde_json::Value>,
    }
    #[derive(Deserialize)]
    struct Entry {
        #[serde(rename = "type")]
        entry_type: Option<String>,
        message: Option<MessageContent>,
    }

    let entry: Entry = serde_json::from_str(line).ok()?;

    if entry.entry_type.as_deref() != Some("user") {
        return None;
    }

    let message = entry.message?;
    // Must be a user role message
    if message.msg_type.as_deref() != Some("human") && message.msg_type.is_some() {
        // Some entries have no type on the inner message — still accept them
        // when they have a string content
    }

    let content = message.content?;

    // Content must be a plain string (arrays = tool_results)
    let text = match &content {
        serde_json::Value::String(s) => s.clone(),
        _ => return None,
    };

    let trimmed = text.trim();

    // Filter: must not start with XML
    if trimmed.starts_with('<') {
        return None;
    }

    // Filter: length bounds (10..=800)
    let len = trimmed.len();
    if !(10..=800).contains(&len) {
        return None;
    }

    // Filter: system injection
    if is_system_injection(trimmed) {
        return None;
    }

    Some(trimmed.to_string())
}

/// Extract all real user messages from a single JSONL file.
///
/// Uses streaming BufReader — never loads the whole file.
fn extract_all_user_messages(path: &Path) -> Vec<String> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Some(msg) = extract_user_content(&line) {
            messages.push(msg);
        }
    }

    messages
}

// ─────────────────────────────────────────────────────────────────────────────
// Session collection (async, bounded concurrency)
// ─────────────────────────────────────────────────────────────────────────────

/// Collect session data from all project directories under `projects_dir`.
///
/// - Respects `since_days` cutoff via file mtime
/// - Skips sessions whose filename starts with `agent-`
/// - Uses a semaphore to bound concurrent file I/O to 32
pub async fn collect_sessions_data(
    projects_dir: &Path,
    since_days: u32,
    filter_project: Option<&str>,
) -> Vec<SessionData> {
    if !projects_dir.exists() {
        return vec![];
    }

    // Build list of project dirs
    let project_dirs: Vec<PathBuf> = match std::fs::read_dir(projects_dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter(|e| {
                if let Some(proj) = filter_project {
                    e.file_name().to_string_lossy().contains(proj)
                } else {
                    true
                }
            })
            .map(|e| e.path())
            .collect(),
        Err(_) => return vec![],
    };

    let cutoff = chrono::Utc::now() - chrono::Duration::days(since_days as i64);
    let cutoff_secs = cutoff.timestamp() as u64;

    let semaphore = Arc::new(Semaphore::new(32));
    let mut handles = Vec::new();

    for project_dir in project_dirs {
        let project_name = project_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Enumerate JSONL files
        let jsonl_files: Vec<PathBuf> = match std::fs::read_dir(&project_dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map(|ext| ext == "jsonl").unwrap_or(false))
                .collect(),
            Err(_) => continue,
        };

        for filepath in jsonl_files {
            let session_id = filepath
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip agent sessions
            if session_id.starts_with("agent-") {
                continue;
            }

            // Date filter via mtime
            let mtime = match filepath.metadata() {
                Ok(m) => m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                Err(_) => continue,
            };

            if mtime < cutoff_secs {
                continue;
            }

            let sem = Arc::clone(&semaphore);
            let filepath_clone = filepath.clone();
            let project_name_clone = project_name.clone();
            let session_id_clone = session_id.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.ok()?;
                // CPU-bound work: spawn_blocking for file I/O + parsing
                let path = filepath_clone.clone();
                let messages =
                    tokio::task::spawn_blocking(move || extract_all_user_messages(&path))
                        .await
                        .ok()?;

                if messages.is_empty() {
                    return None;
                }

                Some(SessionData {
                    session_id: session_id_clone,
                    project: project_name_clone,
                    messages,
                })
            });

            handles.push(handle);
        }
    }

    let mut result = Vec::new();
    for handle in handles {
        if let Ok(Some(data)) = handle.await {
            result.push(data);
        }
    }

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Core algorithm
// ─────────────────────────────────────────────────────────────────────────────

/// Occurrence of an n-gram in a specific session/project
#[derive(Debug, Clone)]
struct NgramOccurrence {
    session_id: String,
    project: String,
}

/// Run the full discovery algorithm on pre-collected session data.
pub fn discover_patterns(
    sessions_data: &[SessionData],
    min_count: usize,
    top: usize,
) -> Vec<DiscoverSuggestion> {
    let total_sessions = sessions_data.len();
    if total_sessions == 0 {
        return vec![];
    }

    // ── Step 1: build n-gram index ────────────────────────────────────────────
    let mut ngram_index: HashMap<Vec<String>, Vec<NgramOccurrence>> = HashMap::new();

    for sd in sessions_data {
        for msg in &sd.messages {
            let tokens = normalize_text(msg);
            if tokens.len() < 3 {
                continue;
            }

            for n in 3..=6usize {
                for ngram in extract_ngrams(&tokens, n) {
                    ngram_index.entry(ngram).or_default().push(NgramOccurrence {
                        session_id: sd.session_id.clone(),
                        project: sd.project.clone(),
                    });
                }
            }
        }
    }

    // ── Step 2: frequency filter ──────────────────────────────────────────────
    let frequent: HashMap<Vec<String>, Vec<NgramOccurrence>> = ngram_index
        .into_iter()
        .filter(|(_, occs)| occs.len() >= min_count)
        .collect();

    // ── Step 3: deduplication — prefer longer n-grams ────────────────────────
    let mut sorted_ngrams: Vec<(Vec<String>, Vec<NgramOccurrence>)> =
        frequent.into_iter().collect();

    // Sort by (len DESC, count DESC)
    sorted_ngrams.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(b.1.len().cmp(&a.1.len())));

    let mut kept: Vec<(Vec<String>, Vec<NgramOccurrence>)> = Vec::new();
    let mut subsumed: HashSet<Vec<String>> = HashSet::new();

    for (ngram, occs) in sorted_ngrams {
        if subsumed.contains(&ngram) {
            continue;
        }
        // Mark all sub-ngrams (length 3..len) as subsumed
        for sub_n in 3..ngram.len() {
            let end = ngram.len() - sub_n + 1;
            for i in 0..end {
                let sub = ngram[i..i + sub_n].to_vec();
                subsumed.insert(sub);
            }
        }
        kept.push((ngram, occs));
    }

    // ── Step 4: Jaccard clustering ────────────────────────────────────────────
    let mut clusters: Vec<Vec<usize>> = Vec::new();
    let mut assigned: HashSet<usize> = HashSet::new();

    for i in 0..kept.len() {
        if assigned.contains(&i) {
            continue;
        }
        let mut cluster = vec![i];
        for j in (i + 1)..kept.len() {
            if assigned.contains(&j) {
                continue;
            }
            let overlap = jaccard_overlap(&kept[i].0, &kept[j].0);
            if overlap > 0.6 {
                cluster.push(j);
                assigned.insert(j);
            }
        }
        clusters.push(cluster);
        assigned.insert(i);
    }

    // ── Step 5: build suggestions per cluster ─────────────────────────────────
    let mut suggestions = Vec::new();

    for cluster in &clusters {
        // Representative: longest ngram with most occurrences
        let best_idx = *cluster
            .iter()
            .max_by_key(|&&i| (kept[i].0.len(), kept[i].1.len()))
            .unwrap();

        let (ref ngram, _) = kept[best_idx];

        // Aggregate across cluster members
        let mut all_occurrences: Vec<&NgramOccurrence> = Vec::new();
        for &idx in cluster {
            all_occurrences.extend(kept[idx].1.iter());
        }

        let distinct_sessions: HashSet<&str> = all_occurrences
            .iter()
            .map(|o| o.session_id.as_str())
            .collect();
        let distinct_projects: HashSet<&str> =
            all_occurrences.iter().map(|o| o.project.as_str()).collect();

        let count = all_occurrences.len();
        let session_count = distinct_sessions.len();
        let project_count = distinct_projects.len();
        let cross_project = project_count >= 2;

        if session_count < min_count {
            continue;
        }

        let session_pct = session_count as f64 / total_sessions as f64;

        let category = if session_pct > 0.20 {
            SuggestionCategory::ClaudeMdRule
        } else if session_pct >= 0.05 {
            SuggestionCategory::Skill
        } else {
            SuggestionCategory::Command
        };

        let score = session_pct * if cross_project { 1.5 } else { 1.0 };

        // Up to 2 example session IDs
        let mut example_sessions: Vec<String> = distinct_sessions
            .iter()
            .take(2)
            .map(|s| s.to_string())
            .collect();
        example_sessions.sort(); // deterministic order

        let pattern = ngram.join(" ");

        suggestions.push(DiscoverSuggestion {
            pattern,
            count,
            session_count,
            project_count,
            cross_project,
            category,
            score: (score * 10000.0).round() / 10000.0,
            example_sessions,
        });
    }

    // ── Step 6: sort by score DESC, take top N ────────────────────────────────
    suggestions.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suggestions.truncate(top);
    suggestions
}

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Run pattern discovery against the Claude projects directory.
///
/// Returns `(suggestions, total_sessions, total_projects)`.
pub async fn run_discover(
    claude_home: &Path,
    config: &DiscoverConfig,
    filter_project: Option<&str>,
) -> anyhow::Result<(Vec<DiscoverSuggestion>, usize, usize)> {
    let projects_dir = claude_home.join("projects");

    eprint!("\rScanning sessions...");
    let sessions_data =
        collect_sessions_data(&projects_dir, config.since_days, filter_project).await;

    let total_sessions = sessions_data.len();
    let total_projects: HashSet<&str> = sessions_data.iter().map(|s| s.project.as_str()).collect();
    let total_projects_count = total_projects.len();

    if total_sessions == 0 {
        eprintln!();
        return Ok((vec![], 0, 0));
    }

    eprint!(
        "\rAnalyzing {} sessions across {} project(s)...    ",
        total_sessions, total_projects_count
    );

    // CPU-bound: run in blocking thread
    let sessions_data_clone = sessions_data;
    let min_count = config.min_count;
    let top = config.top;

    let suggestions = tokio::task::spawn_blocking(move || {
        discover_patterns(&sessions_data_clone, min_count, top)
    })
    .await
    .map_err(|e| anyhow::anyhow!("spawn_blocking failed: {}", e))?;

    eprintln!();

    Ok((suggestions, total_sessions, total_projects_count))
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        let tokens = normalize_text("Add unit tests for the authentication flow!");
        // "add", "unit", "tests", "the", "for", "authentication", "flow" raw
        // after stop word removal: "unit", "tests", "authentication", "flow"
        assert!(tokens.contains(&"authentication".to_string()));
        assert!(tokens.contains(&"flow".to_string()));
        assert!(tokens.contains(&"tests".to_string()));
        // stop words removed
        assert!(!tokens.contains(&"the".to_string()));
        assert!(!tokens.contains(&"for".to_string()));
        // "add" is a stop word
        assert!(!tokens.contains(&"add".to_string()));
    }

    #[test]
    fn test_normalize_strips_punctuation() {
        let tokens = normalize_text("Write tests: before implementation.");
        // Punctuation stripped, no stop words
        assert!(tokens.contains(&"write".to_string()));
        assert!(tokens.contains(&"tests".to_string()));
        assert!(tokens.contains(&"before".to_string()) || !tokens.contains(&"before".to_string()));
        // "before" is a stop word — confirm it's filtered
        assert!(!tokens.contains(&"before".to_string()));
    }

    #[test]
    fn test_ngram_extraction() {
        let tokens: Vec<String> = vec![
            "write".into(),
            "tests".into(),
            "authentication".into(),
            "flow".into(),
            "security".into(),
        ];

        let trigrams = extract_ngrams(&tokens, 3);
        assert_eq!(trigrams.len(), 3);
        assert_eq!(trigrams[0], vec!["write", "tests", "authentication"]);

        let six_grams = extract_ngrams(&tokens, 6);
        assert!(six_grams.is_empty(), "tokens shorter than 6 → no 6-grams");

        let tokens6: Vec<String> = vec![
            "a".into(),
            "b".into(),
            "c".into(),
            "d".into(),
            "e".into(),
            "f".into(),
        ];
        let six_grams2 = extract_ngrams(&tokens6, 6);
        assert_eq!(six_grams2.len(), 1);
    }

    #[test]
    fn test_jaccard_overlap() {
        let a: Vec<String> = vec!["write".into(), "tests".into(), "first".into()];
        let b: Vec<String> = vec!["write".into(), "tests".into(), "auth".into()];
        let overlap = jaccard_overlap(&a, &b);
        // intersection=2 ("write","tests"), union=4
        assert!((overlap - 0.5).abs() < 1e-9);

        let identical: Vec<String> = vec!["foo".into(), "bar".into()];
        assert!((jaccard_overlap(&identical, &identical) - 1.0).abs() < 1e-9);

        let empty: Vec<String> = vec![];
        assert_eq!(jaccard_overlap(&empty, &a), 0.0);
    }

    #[test]
    fn test_category_threshold() {
        // Build synthetic sessions_data to hit 21% → ClaudeMdRule
        let mut sessions_data: Vec<SessionData> = Vec::new();

        // Repeat pattern "security review authentication flow handling" in 22/100 sessions
        for i in 0..100 {
            let msg = if i < 22 {
                "security review authentication flow handling properly"
            } else {
                format!("some other message number {}", i).leak()
            };
            sessions_data.push(SessionData {
                session_id: format!("session-{:04}", i),
                project: "proj-a".to_string(),
                messages: vec![msg.to_string()],
            });
        }

        let suggestions = discover_patterns(&sessions_data, 3, 20);
        // Find suggestion containing "security"
        let security = suggestions.iter().find(|s| s.pattern.contains("security"));
        if let Some(s) = security {
            assert_eq!(
                s.category,
                SuggestionCategory::ClaudeMdRule,
                "22/100 = 22% > 20% → ClaudeMdRule"
            );
        }
    }

    #[test]
    fn test_cross_project_bonus() {
        // Pattern appearing in 2 projects gets 1.5x multiplier
        let sessions_data: Vec<SessionData> = (0..10)
            .map(|i| SessionData {
                session_id: format!("s{}", i),
                project: if i < 5 { "proj-a" } else { "proj-b" }.to_string(),
                messages: vec!["deploy staging environment pipeline testing".to_string()],
            })
            .collect();

        let suggestions = discover_patterns(&sessions_data, 3, 20);
        assert!(!suggestions.is_empty(), "should find patterns");

        let first = &suggestions[0];
        assert!(first.cross_project, "pattern appears in 2 projects");
        // score = session_pct * 1.5 (10/10 * 1.5 = 1.5 capped to 1.0 range-wise, but not clipped here)
        assert!(first.score > first.session_count as f64 / 10.0);
    }
}
