//! Activity parser: extracts tool calls, classifies them, and generates security alerts
//!
//! Streams JSONL session files to extract:
//! - tool_use blocks from assistant messages
//! - tool_result blocks from user messages (for duration + output)
//!
//! Then classifies into FileAccess, BashCommand, NetworkCall, and generates Alerts.

use crate::models::activity::{
    ActivitySummary, Alert, AlertCategory, AlertSeverity, BashCommand, FileAccess, FileOperation,
    NetworkCall, NetworkTool, ToolCall,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{trace, warn};

/// Maximum line size in bytes (10MB) - OOM protection, consistent with session_index.rs
const MAX_LINE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum characters to capture for tool output preview
const OUTPUT_PREVIEW_MAX: usize = 500;

/// Parse all tool calls from a session JSONL file.
///
/// Streams the file line by line, extracting tool_use blocks from assistant messages
/// and matching them with tool_result blocks from user messages to compute duration.
///
/// # Graceful degradation
/// - Oversized lines (>10MB) are skipped with a warning
/// - Malformed JSON lines are skipped silently (trace log)
/// - Tool calls without matching results get duration_ms = None
pub async fn parse_tool_calls(session_jsonl: &Path, session_id: &str) -> Result<Vec<ToolCall>> {
    let file = tokio::fs::File::open(session_jsonl)
        .await
        .with_context(|| format!("Failed to open session file: {}", session_jsonl.display()))?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Pending tool calls: id -> (timestamp, tool_name, input)
    let mut pending: HashMap<String, (DateTime<Utc>, String, Value)> = HashMap::new();
    // Results collected: id -> (duration_ms, output)
    let mut results: HashMap<String, (u64, String)> = HashMap::new();
    // Raw calls in order: (id, name, timestamp, input)
    let mut raw_calls: Vec<(String, String, DateTime<Utc>, Value)> = Vec::new();

    let mut line_number = 0usize;
    let mut last_timestamp = Utc::now();

    while let Some(line) = lines
        .next_line()
        .await
        .with_context(|| format!("Failed to read line from {}", session_jsonl.display()))?
    {
        line_number += 1;

        // OOM protection
        if line.len() > MAX_LINE_SIZE {
            warn!(
                path = %session_jsonl.display(),
                line = line_number,
                "Skipping oversized line in activity parser"
            );
            continue;
        }

        // Parse as generic JSON value
        let json: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                trace!(
                    path = %session_jsonl.display(),
                    line = line_number,
                    error = %e,
                    "Skipping malformed JSON line"
                );
                continue;
            }
        };

        let line_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");

        let timestamp = json
            .get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok())
            .unwrap_or(last_timestamp);
        last_timestamp = timestamp;

        match line_type {
            "assistant" => {
                // Extract tool_use blocks from message.content array
                if let Some(content) = json
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) != Some("tool_use") {
                            continue;
                        }
                        let id = block
                            .get("id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();
                        let name = block
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string();
                        let input = block.get("input").cloned().unwrap_or(Value::Null);

                        if id.is_empty() || name.is_empty() {
                            continue;
                        }

                        pending.insert(id.clone(), (timestamp, name.clone(), input.clone()));
                        raw_calls.push((id, name, timestamp, input));
                    }
                }
            }
            "user" => {
                // Extract tool_result blocks from message.content array
                if let Some(content) = json
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array())
                {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) != Some("tool_result") {
                            continue;
                        }
                        let tool_use_id = block
                            .get("tool_use_id")
                            .and_then(|i| i.as_str())
                            .unwrap_or("")
                            .to_string();

                        if tool_use_id.is_empty() {
                            continue;
                        }

                        let output = extract_tool_result_content(block);

                        if let Some((call_ts, _, _)) = pending.get(&tool_use_id) {
                            let duration_ms =
                                (timestamp - *call_ts).num_milliseconds().max(0) as u64;
                            results.insert(tool_use_id, (duration_ms, output));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Build final ToolCall list, merging with results
    let tool_calls = raw_calls
        .into_iter()
        .map(|(id, tool_name, timestamp, input)| {
            let (duration_ms, output) = results
                .remove(&id)
                .map(|(d, o)| (Some(d), Some(o)))
                .unwrap_or((None, None));
            ToolCall {
                id,
                session_id: session_id.to_string(),
                timestamp,
                tool_name,
                input,
                duration_ms,
                output,
            }
        })
        .collect();

    Ok(tool_calls)
}

/// Classify tool calls into FileAccess, BashCommand, NetworkCall, and generate Alerts.
///
/// # Parameters
/// - `calls`: Tool calls from `parse_tool_calls`
/// - `session_id`: Session identifier for attribution
/// - `project_root`: Optional project root path for scope violation detection
pub fn classify_tool_calls(
    calls: Vec<ToolCall>,
    session_id: &str,
    project_root: Option<&str>,
) -> ActivitySummary {
    let mut summary = ActivitySummary::default();

    for call in &calls {
        match call.tool_name.as_str() {
            "Read" | "Write" | "Edit" | "Glob" | "Grep" => {
                if let Some(access) = extract_file_access(call, session_id) {
                    summary.file_accesses.push(access);
                }
            }
            "Bash" => {
                let command = call
                    .input
                    .get("command")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                let is_destructive = is_destructive_command(&command);
                let output_preview: String = call
                    .output
                    .as_deref()
                    .unwrap_or("")
                    .chars()
                    .take(OUTPUT_PREVIEW_MAX)
                    .collect();

                summary.bash_commands.push(BashCommand {
                    session_id: session_id.to_string(),
                    timestamp: call.timestamp,
                    command,
                    is_destructive,
                    output_preview,
                });
            }
            "WebFetch" => {
                let url = call
                    .input
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let domain = extract_domain(&url);
                summary.network_calls.push(NetworkCall {
                    session_id: session_id.to_string(),
                    timestamp: call.timestamp,
                    url,
                    tool: NetworkTool::WebFetch,
                    domain,
                });
            }
            "WebSearch" => {
                let query = call
                    .input
                    .get("query")
                    .and_then(|q| q.as_str())
                    .unwrap_or("")
                    .to_string();
                summary.network_calls.push(NetworkCall {
                    session_id: session_id.to_string(),
                    timestamp: call.timestamp,
                    url: query,
                    tool: NetworkTool::WebSearch,
                    domain: String::new(),
                });
            }
            name if name.starts_with("mcp__") => {
                // Format: mcp__<server>__<tool>
                let server = name
                    .strip_prefix("mcp__")
                    .and_then(|s| s.split("__").next())
                    .unwrap_or("unknown")
                    .to_string();
                let url = call
                    .input
                    .get("url")
                    .or_else(|| call.input.get("uri"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let domain = extract_domain(&url);
                summary.network_calls.push(NetworkCall {
                    session_id: session_id.to_string(),
                    timestamp: call.timestamp,
                    url,
                    tool: NetworkTool::McpCall { server },
                    domain,
                });
            }
            _ => {}
        }
    }

    // Generate security alerts from classified activity
    summary.alerts = generate_alerts(
        &summary.file_accesses,
        &summary.bash_commands,
        &summary.network_calls,
        session_id,
        project_root,
    );

    summary
}

/// Check whether a shell command segment (split on `;`, `&&`, `||`, `|`) that contains
/// `subcommand` also contains `flag` — prevents multi-command false positives like
/// "git push origin main; ls -f /tmp" triggering force-push detection.
fn cmd_segment_has_flag(lower: &str, subcommand: &str, flag: &str) -> bool {
    lower
        .split([';', '|', '&'])
        .any(|seg| seg.contains(subcommand) && seg.contains(flag))
}

/// Check if a bash command is destructive.
///
/// Covers: rm -rf, git push --force/-f, git reset --hard,
/// git clean -f, DROP TABLE/DATABASE, pkill, kill -9
pub fn is_destructive_command(cmd: &str) -> bool {
    // Normalize whitespace for consistent checking
    let normalized: String = cmd.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = normalized.to_lowercase();

    // rm -rf variants
    if lower.contains("rm -rf") || lower.contains("rm -fr") {
        return true;
    }

    // git push --force or -f — check segment-by-segment to avoid false positives in
    // multi-command strings like "git push origin main; ls -f /tmp"
    if cmd_segment_has_flag(&lower, "git push", "--force")
        || cmd_segment_has_flag(&lower, "git push", " -f")
    {
        return true;
    }

    // git reset --hard
    if lower.contains("git reset --hard") {
        return true;
    }

    // git clean -f or --force — same segment-based check
    if cmd_segment_has_flag(&lower, "git clean", " -f")
        || cmd_segment_has_flag(&lower, "git clean", "--force")
    {
        return true;
    }

    // SQL DROP statements (case-insensitive via lower)
    if lower.contains("drop table") || lower.contains("drop database") {
        return true;
    }

    // Process killing
    if lower.contains("pkill") || lower.contains("kill -9") {
        return true;
    }

    false
}

/// Check if a file path refers to a sensitive file.
///
/// Matches on the filename (not full path). Covers:
/// - .env and variants (.env.local, .env.production)
/// - Private keys (id_rsa, id_ed25519, id_ecdsa, id_dsa)
/// - Certificates (.pem, .p12, .pfx)
/// - Credential files (secrets.json, credentials.json)
/// - Auth configs (.npmrc, .netrc)
pub fn is_sensitive_file(path: &str) -> bool {
    let filename = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);

    // .env and all variants (.env.local, .env.production, .env.test, etc.)
    if filename.starts_with(".env") {
        return true;
    }

    // Private SSH/GPG keys (NOT .pub — public keys are intentionally distributed)
    if matches!(filename, "id_rsa" | "id_ed25519" | "id_ecdsa" | "id_dsa") {
        return true;
    }

    // Certificate and key files
    if filename.ends_with(".pem") || filename.ends_with(".p12") || filename.ends_with(".pfx") {
        return true;
    }

    // Credential JSON files
    if matches!(filename, "secrets.json" | "credentials.json") {
        return true;
    }

    // Package manager and network auth
    if matches!(filename, ".npmrc" | ".netrc") {
        return true;
    }

    false
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Extract tool_result content from a tool_result block.
///
/// Handles both string content and array-of-text-blocks format.
fn extract_tool_result_content(block: &Value) -> String {
    match block.get("content") {
        Some(Value::String(s)) => s.chars().take(OUTPUT_PREVIEW_MAX).collect(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter_map(|b| {
                if b.get("type").and_then(|t| t.as_str()) == Some("text") {
                    b.get("text")
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            .chars()
            .take(OUTPUT_PREVIEW_MAX)
            .collect(),
        _ => String::new(),
    }
}

/// Extract a FileAccess from a tool call.
fn extract_file_access(call: &ToolCall, session_id: &str) -> Option<FileAccess> {
    let operation = match call.tool_name.as_str() {
        "Read" => FileOperation::Read,
        "Write" => FileOperation::Write,
        "Edit" => FileOperation::Edit,
        "Glob" => FileOperation::Glob,
        "Grep" => FileOperation::Grep,
        _ => return None,
    };

    // Try various input field names for the path
    let path = call
        .input
        .get("file_path")
        .or_else(|| call.input.get("path"))
        .or_else(|| call.input.get("pattern")) // Glob uses "pattern"
        .and_then(|p| p.as_str())
        .unwrap_or("")
        .to_string();

    if path.is_empty() {
        return None;
    }

    // Extract line range for Read operations
    let line_range = if call.tool_name == "Read" {
        let offset = call.input.get("offset").and_then(|o| o.as_u64());
        let limit = call.input.get("limit").and_then(|l| l.as_u64());
        match (offset, limit) {
            (Some(off), Some(lim)) => Some((off, off + lim)),
            _ => None,
        }
    } else {
        None
    };

    Some(FileAccess {
        session_id: session_id.to_string(),
        timestamp: call.timestamp,
        path,
        operation,
        line_range,
    })
}

/// Extract domain from a URL string.
///
/// Returns empty string if URL is not a valid HTTP/HTTPS URL.
fn extract_domain(url: &str) -> String {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .map(|without_scheme| {
            without_scheme
                .split('/')
                .next()
                .unwrap_or("")
                .split('?')
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default()
}

/// Generate security alerts from classified activity.
fn generate_alerts(
    file_accesses: &[FileAccess],
    bash_commands: &[BashCommand],
    network_calls: &[NetworkCall],
    session_id: &str,
    project_root: Option<&str>,
) -> Vec<Alert> {
    let mut alerts = Vec::new();

    // File access alerts
    for access in file_accesses {
        // CredentialAccess: any operation on a sensitive file
        if is_sensitive_file(&access.path) {
            alerts.push(Alert {
                session_id: session_id.to_string(),
                timestamp: access.timestamp,
                severity: AlertSeverity::Warning,
                category: AlertCategory::CredentialAccess,
                detail: format!("Accessed sensitive file: {}", access.path),
            });
        }

        // ScopeViolation: write/edit outside project root.
        // Use Path::starts_with (component-based) to avoid false passes on paths like
        // "/home/user/project-evil" when root is "/home/user/project".
        if let Some(root) = project_root {
            if matches!(access.operation, FileOperation::Write | FileOperation::Edit)
                && !access.path.is_empty()
                && !std::path::Path::new(&access.path).starts_with(std::path::Path::new(root))
            {
                alerts.push(Alert {
                    session_id: session_id.to_string(),
                    timestamp: access.timestamp,
                    severity: AlertSeverity::Warning,
                    category: AlertCategory::ScopeViolation,
                    detail: format!("Write outside project root '{}': {}", root, access.path),
                });
            }
        }
    }

    // Bash command alerts
    for cmd in bash_commands {
        if cmd.is_destructive {
            let lower = cmd.command.to_lowercase();
            // Distinguish force push from generic destructive
            let category = if lower.contains("git push")
                && (lower.contains("--force") || lower.contains(" -f"))
            {
                AlertCategory::ForcePush
            } else {
                AlertCategory::DestructiveCommand
            };

            alerts.push(Alert {
                session_id: session_id.to_string(),
                timestamp: cmd.timestamp,
                severity: AlertSeverity::Critical,
                category,
                detail: format!("Destructive command: {}", cmd.command),
            });
        }

        // CredentialAccess: credential-like strings in output.
        // "sk-" requires ≥20 alphanumeric chars after prefix to reduce false positives.
        let output = &cmd.output_preview;
        let has_sk_token = output
            .find("sk-")
            .map(|i| {
                output[i + 3..]
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .count()
                    >= 20
            })
            .unwrap_or(false);
        if has_sk_token
            || output.contains("ghp_")
            || output.contains("ghu_")
            || output.contains("ghs_")
            || output.contains("glpat-")
            || output.contains("xoxb-")
            || output.contains("xoxp-")
            || output.contains("AKIA")
            || output.contains("sk-ant-")
        {
            alerts.push(Alert {
                session_id: session_id.to_string(),
                timestamp: cmd.timestamp,
                severity: AlertSeverity::Critical,
                category: AlertCategory::CredentialAccess,
                detail: "Potential credential exposed in bash output".to_string(),
            });
        }
    }

    // Network call alerts
    for net in network_calls {
        if matches!(net.tool, NetworkTool::WebFetch) && !net.domain.is_empty() {
            alerts.push(Alert {
                session_id: session_id.to_string(),
                timestamp: net.timestamp,
                severity: AlertSeverity::Info,
                category: AlertCategory::ExternalExfil,
                detail: format!("External HTTP fetch: {}", net.domain),
            });
        }
    }

    alerts
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── is_destructive_command ───────────────────────────────────────────────

    #[test]
    fn test_destructive_rm_rf() {
        assert!(is_destructive_command("rm -rf /tmp/test"));
        assert!(is_destructive_command("rm -rf ."));
        assert!(is_destructive_command("rm -fr /home/user/data"));
    }

    #[test]
    fn test_destructive_git_push_force() {
        assert!(is_destructive_command("git push --force"));
        assert!(is_destructive_command("git push origin main --force"));
        assert!(is_destructive_command("git push -f"));
        assert!(is_destructive_command("git push origin -f"));
        assert!(is_destructive_command("git push -f origin main"));
    }

    #[test]
    fn test_destructive_git_reset_hard() {
        assert!(is_destructive_command("git reset --hard HEAD~1"));
        assert!(is_destructive_command("git reset --hard origin/main"));
    }

    #[test]
    fn test_destructive_git_clean() {
        assert!(is_destructive_command("git clean -f"));
        assert!(is_destructive_command("git clean --force"));
        assert!(is_destructive_command("git clean -fd"));
    }

    #[test]
    fn test_destructive_sql_drop() {
        assert!(is_destructive_command("DROP TABLE users"));
        assert!(is_destructive_command("drop table users;"));
        assert!(is_destructive_command("DROP DATABASE mydb"));
    }

    #[test]
    fn test_destructive_kill() {
        assert!(is_destructive_command("kill -9 1234"));
        assert!(is_destructive_command("pkill chrome"));
        assert!(is_destructive_command("pkill -f myprocess"));
    }

    #[test]
    fn test_not_destructive_normal_commands() {
        assert!(!is_destructive_command("cargo build"));
        assert!(!is_destructive_command("git status"));
        assert!(!is_destructive_command("git push"));
        assert!(!is_destructive_command("ls -la"));
        assert!(!is_destructive_command("cargo test --all"));
        assert!(!is_destructive_command("rm file.txt")); // non-recursive rm is OK
    }

    // ── is_sensitive_file ────────────────────────────────────────────────────

    #[test]
    fn test_sensitive_env_files() {
        assert!(is_sensitive_file(".env"));
        assert!(is_sensitive_file("/project/.env"));
        assert!(is_sensitive_file("/project/.env.local"));
        assert!(is_sensitive_file("/project/.env.production"));
        assert!(is_sensitive_file(".env.test"));
    }

    #[test]
    fn test_sensitive_ssh_keys() {
        assert!(is_sensitive_file("id_rsa"));
        assert!(is_sensitive_file("/home/user/.ssh/id_rsa"));
        assert!(is_sensitive_file("id_ed25519"));
        assert!(is_sensitive_file("id_ecdsa"));
    }

    #[test]
    fn test_sensitive_certificates() {
        assert!(is_sensitive_file("cert.pem"));
        assert!(is_sensitive_file("server.p12"));
        assert!(is_sensitive_file("keystore.pfx"));
    }

    #[test]
    fn test_sensitive_credential_files() {
        assert!(is_sensitive_file("secrets.json"));
        assert!(is_sensitive_file("credentials.json"));
        assert!(is_sensitive_file(".npmrc"));
        assert!(is_sensitive_file(".netrc"));
    }

    #[test]
    fn test_not_sensitive_normal_files() {
        assert!(!is_sensitive_file("main.rs"));
        assert!(!is_sensitive_file("README.md"));
        assert!(!is_sensitive_file("config.json")); // Not "credentials.json"
        assert!(!is_sensitive_file("package.json"));
        assert!(!is_sensitive_file("src/lib.rs"));
    }

    // ── parse_tool_calls ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_parse_tool_calls_simple_fixture() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/activity/simple_session.jsonl");
        let calls = parse_tool_calls(&fixture, "simple-session").await.unwrap();

        // Should have: Read, Bash, WebFetch
        assert_eq!(calls.len(), 3, "Expected 3 tool calls");

        let names: Vec<&str> = calls.iter().map(|c| c.tool_name.as_str()).collect();
        assert!(names.contains(&"Read"));
        assert!(names.contains(&"Bash"));
        assert!(names.contains(&"WebFetch"));

        // All should have session_id set
        assert!(calls.iter().all(|c| c.session_id == "simple-session"));
    }

    #[tokio::test]
    async fn test_parse_tool_calls_destructive_fixture() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/activity/destructive_session.jsonl");
        let calls = parse_tool_calls(&fixture, "destructive-session")
            .await
            .unwrap();

        assert!(!calls.is_empty(), "Should have tool calls");

        // All Bash
        let bash_calls: Vec<_> = calls.iter().filter(|c| c.tool_name == "Bash").collect();
        assert!(
            bash_calls.len() >= 3,
            "Expected at least 3 bash calls, got {}",
            bash_calls.len()
        );
    }

    #[tokio::test]
    async fn test_parse_tool_calls_credential_fixture() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/activity/credential_session.jsonl");
        let calls = parse_tool_calls(&fixture, "credential-session")
            .await
            .unwrap();

        assert!(!calls.is_empty(), "Should have tool calls");

        // Should have Read calls
        let read_calls: Vec<_> = calls.iter().filter(|c| c.tool_name == "Read").collect();
        assert!(!read_calls.is_empty(), "Should have Read calls");
    }

    #[tokio::test]
    async fn test_parse_duration_calculation() {
        let mut file = NamedTempFile::new().unwrap();

        // Assistant calls a tool at T+0
        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"call_1","name":"Read","input":{{"file_path":"main.rs"}}}}],"usage":{{"input_tokens":10,"output_tokens":5}}}}}}"#).unwrap();

        // User returns result 2 seconds later
        writeln!(file, r#"{{"type":"user","timestamp":"2025-01-15T10:00:02Z","message":{{"content":[{{"type":"tool_result","tool_use_id":"call_1","content":"file content"}}]}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "test").await.unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].duration_ms, Some(2000));
        assert_eq!(calls[0].output.as_deref(), Some("file content"));
    }

    #[tokio::test]
    async fn test_parse_no_duration_without_result() {
        let mut file = NamedTempFile::new().unwrap();

        // Tool call with no matching result
        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"call_no_result","name":"Bash","input":{{"command":"echo hello"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "test").await.unwrap();

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].duration_ms, None);
        assert_eq!(calls[0].output, None);
    }

    // ── classify_tool_calls ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_classify_fan_out() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"r1","name":"Read","input":{{"file_path":"src/main.rs"}}}}],"usage":{{"input_tokens":10,"output_tokens":5}}}}}}"#).unwrap();
        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:01:00Z","message":{{"content":[{{"type":"tool_use","id":"b1","name":"Bash","input":{{"command":"cargo build"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();
        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:02:00Z","message":{{"content":[{{"type":"tool_use","id":"w1","name":"WebFetch","input":{{"url":"https://docs.rs/tokio"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "classify-test")
            .await
            .unwrap();
        let summary = classify_tool_calls(calls, "classify-test", None);

        assert_eq!(summary.file_accesses.len(), 1);
        assert_eq!(summary.bash_commands.len(), 1);
        assert_eq!(summary.network_calls.len(), 1);
    }

    // ── generate_alerts ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_alert_credential_access() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"r1","name":"Read","input":{{"file_path":"/home/user/.env"}}}}],"usage":{{"input_tokens":10,"output_tokens":5}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "alert-test").await.unwrap();
        let summary = classify_tool_calls(calls, "alert-test", None);

        let cred_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::CredentialAccess))
            .collect();
        assert!(
            !cred_alerts.is_empty(),
            "Should have credential access alert"
        );
        assert_eq!(cred_alerts[0].severity, AlertSeverity::Warning);
    }

    #[tokio::test]
    async fn test_alert_destructive_command() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"b1","name":"Bash","input":{{"command":"rm -rf /tmp/test"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "destruct-test")
            .await
            .unwrap();
        let summary = classify_tool_calls(calls, "destruct-test", None);

        let destruct_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::DestructiveCommand))
            .collect();
        assert!(
            !destruct_alerts.is_empty(),
            "Should have destructive command alert"
        );
        assert_eq!(destruct_alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_alert_force_push() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"b1","name":"Bash","input":{{"command":"git push origin main --force"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "fp-test").await.unwrap();
        let summary = classify_tool_calls(calls, "fp-test", None);

        let fp_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::ForcePush))
            .collect();
        assert!(!fp_alerts.is_empty(), "Should have force push alert");
        assert_eq!(fp_alerts[0].severity, AlertSeverity::Critical);
    }

    #[tokio::test]
    async fn test_alert_external_exfil() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"w1","name":"WebFetch","input":{{"url":"https://api.example.com/data"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "exfil-test").await.unwrap();
        let summary = classify_tool_calls(calls, "exfil-test", None);

        let exfil_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::ExternalExfil))
            .collect();
        assert!(!exfil_alerts.is_empty(), "Should have ExternalExfil alert");
        assert_eq!(exfil_alerts[0].severity, AlertSeverity::Info);
        assert!(exfil_alerts[0].detail.contains("api.example.com"));
    }

    #[tokio::test]
    async fn test_alert_scope_violation() {
        let mut file = NamedTempFile::new().unwrap();

        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"e1","name":"Edit","input":{{"file_path":"/etc/hosts"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "scope-test").await.unwrap();
        let summary = classify_tool_calls(calls, "scope-test", Some("/home/user/myproject"));

        let scope_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::ScopeViolation))
            .collect();
        assert!(
            !scope_alerts.is_empty(),
            "Should have scope violation alert"
        );
        assert_eq!(scope_alerts[0].severity, AlertSeverity::Warning);
    }

    #[tokio::test]
    async fn test_alert_credential_in_output() {
        let mut file = NamedTempFile::new().unwrap();

        // Tool call
        writeln!(file, r#"{{"type":"assistant","timestamp":"2025-01-15T10:00:00Z","message":{{"content":[{{"type":"tool_use","id":"b1","name":"Bash","input":{{"command":"env"}}}}],"usage":{{"input_tokens":5,"output_tokens":2}}}}}}"#).unwrap();
        // Result with credential-like token
        writeln!(file, r#"{{"type":"user","timestamp":"2025-01-15T10:00:01Z","message":{{"content":[{{"type":"tool_result","tool_use_id":"b1","content":"GITHUB_TOKEN=ghp_abc123secrettoken"}}]}}}}"#).unwrap();

        let calls = parse_tool_calls(file.path(), "cred-output-test")
            .await
            .unwrap();
        let summary = classify_tool_calls(calls, "cred-output-test", None);

        let cred_alerts: Vec<_> = summary
            .alerts
            .iter()
            .filter(|a| matches!(a.category, AlertCategory::CredentialAccess))
            .collect();
        assert!(
            !cred_alerts.is_empty(),
            "Should detect credential in output"
        );
        // Should be Critical severity
        assert!(cred_alerts
            .iter()
            .any(|a| a.severity == AlertSeverity::Critical));
    }
}
