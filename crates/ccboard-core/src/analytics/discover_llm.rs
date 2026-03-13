//! LLM-powered pattern discovery via `claude --print`
//!
//! Deduplicates messages using Jaccard similarity, takes the top 60 by frequency,
//! builds a structured prompt, and calls the local Claude CLI.
//! No API key required — uses the existing Claude subscription.

use serde::{Deserialize, Serialize};

use super::discover::{jaccard_overlap, normalize_text, SessionData};

const LLM_BATCH_SIZE: usize = 60;
const LLM_DEDUP_WINDOW: usize = 300;
const LLM_DEDUP_THRESHOLD: f64 = 0.65;

/// LLM suggestion returned by `claude --print`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmSuggestion {
    pub pattern: String,
    pub category: String,
    pub suggested_name: Option<String>,
    pub rationale: Option<String>,
    pub frequency: Option<String>,
    pub example_messages: Option<Vec<String>>,
    pub suggested_content: Option<String>,
}

/// Deduplicated message entry for LLM analysis
#[derive(Debug, Clone)]
struct DeduplicatedMessage {
    text: String,
    count: usize,
    projects: Vec<String>,
}

/// Deduplicate semantically similar messages using Jaccard similarity.
///
/// Iterates with a bounded window of `LLM_DEDUP_WINDOW` for performance.
/// Returns messages sorted by frequency (highest first), capped at `max_messages`.
fn deduplicate_messages(
    sessions_data: &[SessionData],
    max_messages: usize,
) -> Vec<DeduplicatedMessage> {
    // Collect all messages with project info
    struct RawMsg {
        text: String,
        project: String,
        tokens: Vec<String>,
    }

    let all_msgs: Vec<RawMsg> = sessions_data
        .iter()
        .flat_map(|sd| {
            sd.messages.iter().map(move |msg| RawMsg {
                text: msg.chars().take(500).collect(),
                project: sd.project.clone(),
                tokens: normalize_text(msg),
            })
        })
        .collect();

    if all_msgs.is_empty() {
        return vec![];
    }

    let mut assigned: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut deduped: Vec<DeduplicatedMessage> = Vec::new();

    for i in 0..all_msgs.len() {
        if assigned.contains(&i) {
            continue;
        }
        let mut cluster_projects = vec![all_msgs[i].project.clone()];
        let window_end = (i + 1 + LLM_DEDUP_WINDOW).min(all_msgs.len());

        for j in (i + 1)..window_end {
            if assigned.contains(&j) {
                continue;
            }
            let overlap = jaccard_overlap(&all_msgs[i].tokens, &all_msgs[j].tokens);
            if overlap > LLM_DEDUP_THRESHOLD {
                cluster_projects.push(all_msgs[j].project.clone());
                assigned.insert(j);
            }
        }
        assigned.insert(i);

        // Deduplicate project list
        let mut unique_projects = cluster_projects.clone();
        unique_projects.sort();
        unique_projects.dedup();

        deduped.push(DeduplicatedMessage {
            text: all_msgs[i].text.clone(),
            count: cluster_projects.len(),
            projects: unique_projects,
        });
    }

    deduped.sort_by(|a, b| b.count.cmp(&a.count));
    deduped.truncate(max_messages);
    deduped
}

/// Build the structured analysis prompt for Claude.
fn build_analysis_prompt(messages: &[DeduplicatedMessage]) -> String {
    let mut lines = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        let count_info = if m.count > 1 {
            format!(" (x{})", m.count)
        } else {
            String::new()
        };
        let cross = if m.projects.len() > 1 {
            " [multi-project]"
        } else {
            ""
        };
        let text: String = m
            .text
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ");
        lines.push(format!("{}. {}{}{}", i + 1, text, count_info, cross));
    }

    let messages_block = lines.join("\n");

    format!(
        r#"You are analyzing a developer's Claude Code session history to find recurring patterns worth extracting as reusable configurations.

Below are user messages (deduplicated). Numbers in parentheses show how many times a similar message appeared. [multi-project] means it appeared across different codebases.

MESSAGES:
{messages_block}

Identify recurring patterns and suggest what to extract. For each suggestion, choose the category:
- CLAUDE.md rule: a behavioral instruction that should always be active (broad constraint or guideline)
- skill: specialized expertise loaded on-demand (domain-specific, not always needed)
- command: a repeatable step-by-step workflow with clear inputs/outputs

Return ONLY a JSON array, no prose outside it:
[
  {{
    "pattern": "short description of the recurring intent (max 60 chars)",
    "category": "CLAUDE.md rule",
    "suggested_name": "kebab-case-name",
    "rationale": "one sentence explaining why this should be extracted",
    "frequency": "high",
    "example_messages": ["example 1", "example 2"],
    "suggested_content": "what the skill/command/rule would contain (2-3 sentences)"
  }}
]

Rules:
- Only include genuinely recurring patterns (at least 2 messages with similar intent)
- Prefer specific, actionable suggestions over generic ones
- Maximum 15 suggestions, sorted by impact (most valuable first)"#,
        messages_block = messages_block
    )
}

/// Call `claude --print "<prompt>"` as a subprocess.
///
/// Strips CLAUDECODE and CLAUDE_CODE_ENTRYPOINT from subprocess env
/// to avoid nested-session detection.
pub fn call_claude_cli(
    sessions_data: &[SessionData],
    model: &str,
) -> anyhow::Result<Vec<LlmSuggestion>> {
    let deduped = deduplicate_messages(sessions_data, LLM_DEDUP_WINDOW);
    if deduped.is_empty() {
        return Ok(vec![]);
    }

    let batch = &deduped[..deduped.len().min(LLM_BATCH_SIZE)];

    eprintln!(
        "Sending {} unique messages to claude --print{}...",
        batch.len(),
        if model.is_empty() {
            String::new()
        } else {
            format!(" ({})", model)
        }
    );

    let prompt = build_analysis_prompt(batch);

    let mut cmd = std::process::Command::new("claude");
    cmd.arg("--print");
    if !model.is_empty() {
        cmd.args(["--model", model]);
    }
    // Pass the prompt via stdin to avoid argument length limits and quoting issues
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Remove env vars that block nested Claude sessions
    let mut env: std::collections::HashMap<String, String> = std::env::vars().collect();
    env.remove("CLAUDECODE");
    env.remove("CLAUDE_CODE_ENTRYPOINT");
    cmd.env_clear();
    for (k, v) in &env {
        cmd.env(k, v);
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            anyhow::anyhow!(
                "'claude' CLI not found. Make sure Claude Code is installed and in PATH."
            )
        } else {
            anyhow::anyhow!("Failed to run claude CLI: {}", e)
        }
    })?;

    // Write prompt to stdin
    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write prompt to claude stdin: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| anyhow::anyhow!("Failed to wait for claude CLI: {}", e))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        let stdout_hint = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "claude CLI error (exit {}):\n{}{}",
            output.status.code().unwrap_or(-1),
            &detail[..detail.len().min(500)],
            if !stdout_hint.is_empty() {
                format!("\nstdout: {}", &stdout_hint[..stdout_hint.len().min(300)])
            } else {
                String::new()
            }
        );
    }

    let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Catch runtime errors reported on stdout with exit 0
    if text.to_lowercase().starts_with("execution error")
        || text.to_lowercase().starts_with("error:")
    {
        anyhow::bail!(
            "claude CLI reported an error:\n{}",
            &text[..text.len().min(300)]
        );
    }

    // Strip markdown code fences if present
    if text.starts_with("```") {
        text = text
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .to_string();
        if text.ends_with("```") {
            text = text[..text.len() - 3].to_string();
        }
        text = text.trim().to_string();
    }

    // Extract JSON array if surrounded by prose
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            if end >= start {
                text = text[start..=end].to_string();
            }
        }
    }

    let suggestions: Vec<LlmSuggestion> = serde_json::from_str(&text).map_err(|e| {
        anyhow::anyhow!(
            "Failed to parse CLI response as JSON: {}\nRaw: {}",
            e,
            &text[..text.len().min(500)]
        )
    })?;

    Ok(suggestions)
}
