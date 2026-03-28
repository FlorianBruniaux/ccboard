//! Parser for GitHub Copilot CLI (copilot-api) session logs
//!
//! Reads `~/.local/share/copilot-api/logs/messages-handler-*.log`.
//!
//! ## Log format
//! ```text
//! [2026-03-14 17:32:05] [debug] [messages-handler] Extracted session ID: <UUID>
//! [2026-03-14 17:32:15] [debug] [messages-handler] Translated Anthropic event: {"type":"message_start","message":{...,"model":"gpt-5.4-2026-03-05",...}}
//! [2026-03-14 17:32:16] [debug] [messages-handler] Translated Anthropic event: {"type":"message_delta",...,"usage":{"input_tokens":52055,"output_tokens":455,"cache_read_input_tokens":0}}
//! ```
//!
//! ## Limitations
//! - No per-project info (all sessions under `copilot:global`)
//! - No cost data (Copilot is subscription-based, no public per-token pricing)
//! - Only the single active log file is parsed (copilot-api rotates daily)

use crate::error::LoadReport;
use crate::models::{ProjectId, SessionId, SessionMetadata};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Source tool identifier for Copilot sessions
pub const COPILOT_SOURCE: &str = "copilot";

/// Raw usage object from Anthropic event JSON
#[derive(Debug, Deserialize)]
struct EventUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

/// Accumulated data for a single Copilot session
#[derive(Debug, Default)]
struct SessionAccum {
    first_timestamp: Option<DateTime<Utc>>,
    last_timestamp: Option<DateTime<Utc>>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    message_count: u64,
    models: HashSet<String>,
}

impl SessionAccum {
    fn update_timestamp(&mut self, ts: DateTime<Utc>) {
        if self.first_timestamp.is_none() {
            self.first_timestamp = Some(ts);
        }
        self.last_timestamp = Some(ts);
    }

    fn add_usage(&mut self, usage: &EventUsage, ts: DateTime<Utc>) {
        // Only count non-zero usage (message_start has all-zero placeholder)
        if usage.input_tokens > 0 || usage.output_tokens > 0 {
            self.input_tokens += usage.input_tokens;
            self.output_tokens += usage.output_tokens;
            self.cache_read_tokens += usage.cache_read_input_tokens;
            self.message_count += 1;
            self.update_timestamp(ts);
        }
    }
}

/// Parser for Copilot CLI (copilot-api) log files
pub struct CopilotParser;

impl CopilotParser {
    /// Scan copilot-api log dir and return sessions as `SessionMetadata`
    pub fn scan_all(copilot_log_dir: &Path, report: &mut LoadReport) -> Vec<SessionMetadata> {
        let log_files = match Self::find_log_files(copilot_log_dir) {
            Ok(f) => f,
            Err(e) => {
                warn!("Failed to read Copilot log dir {}: {}", copilot_log_dir.display(), e);
                return Vec::new();
            }
        };

        let mut sessions: HashMap<String, SessionAccum> = HashMap::new();
        let mut log_file_path: Option<PathBuf> = None;

        for path in &log_files {
            log_file_path = Some(path.clone());
            if let Err(e) = Self::parse_log_file(path, &mut sessions) {
                warn!("Failed to parse Copilot log {}: {}", path.display(), e);
                report.sessions_failed += 1;
            }
        }

        let result: Vec<SessionMetadata> = sessions
            .into_iter()
            .filter(|(_, accum)| accum.message_count > 0)
            .map(|(id, accum)| {
                let total_tokens = accum.input_tokens + accum.output_tokens + accum.cache_read_tokens;
                let duration_seconds = match (accum.first_timestamp, accum.last_timestamp) {
                    (Some(s), Some(e)) => {
                        let diff = e.signed_duration_since(s).num_seconds();
                        if diff > 0 { Some(diff as u64) } else { None }
                    }
                    _ => None,
                };

                let mut models_used: Vec<String> = accum.models.into_iter().collect();
                models_used.sort();

                SessionMetadata {
                    id: SessionId::new(id.clone()),
                    source_tool: Some(COPILOT_SOURCE.to_string()),
                    file_path: log_file_path
                        .clone()
                        .unwrap_or_else(|| PathBuf::from("copilot.log")),
                    project_path: ProjectId::from("copilot:global"),
                    first_timestamp: accum.first_timestamp,
                    last_timestamp: accum.last_timestamp,
                    message_count: accum.message_count,
                    total_tokens,
                    input_tokens: accum.input_tokens,
                    output_tokens: accum.output_tokens,
                    cache_creation_tokens: 0,
                    cache_read_tokens: accum.cache_read_tokens,
                    models_used,
                    file_size_bytes: 0,
                    first_user_message: None,
                    has_subagents: false,
                    duration_seconds,
                    branch: None,
                    tool_usage: HashMap::new(),
                    tool_token_usage: HashMap::new(),
                }
            })
            .collect();

        let count = result.len();
        report.sessions_scanned += count;
        debug!("Copilot: scanned {} sessions", count);
        result
    }

    /// Check if copilot-api logs are available
    pub fn is_available(copilot_log_dir: &Path) -> bool {
        copilot_log_dir.exists()
            && Self::find_log_files(copilot_log_dir)
                .map(|f| !f.is_empty())
                .unwrap_or(false)
    }

    /// Find all `messages-handler-*.log` files, sorted by name (chronological)
    fn find_log_files(log_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
        let mut files: Vec<PathBuf> = std::fs::read_dir(log_dir)?
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("messages-handler-") && n.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();
        files.sort();
        Ok(files)
    }

    /// Parse a single log file, accumulating sessions into the map
    fn parse_log_file(
        path: &Path,
        sessions: &mut HashMap<String, SessionAccum>,
    ) -> std::io::Result<()> {
        let file = std::fs::File::open(path)?;
        let reader = BufReader::new(file);

        let mut current_session_id: Option<String> = None;
        let mut pending_model: Option<String> = None;

        for line in reader.lines() {
            let line = line?;

            // Extract timestamp from line prefix `[YYYY-MM-DD HH:MM:SS]`
            let ts = parse_line_timestamp(&line);

            // Session ID boundary
            if let Some(session_id) = extract_session_id(&line) {
                current_session_id = Some(session_id.to_string());
                if let Some(ts) = ts {
                    sessions.entry(session_id.to_string()).or_default().update_timestamp(ts);
                }
                continue;
            }

            // Anthropic event lines
            if let Some(json_str) = extract_event_json(&line) {
                if let Ok(event) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");

                    match event_type {
                        "message_start" => {
                            // Extract model for this exchange
                            if let Some(model) = event
                                .get("message")
                                .and_then(|m| m.get("model"))
                                .and_then(|v| v.as_str())
                            {
                                pending_model = Some(normalize_model_name(model));
                            }
                        }
                        "message_delta" => {
                            if let Some(ref sid) = current_session_id.clone() {
                                if let Some(Ok(usage)) = event
                                    .get("usage")
                                    .map(|u| serde_json::from_value::<EventUsage>(u.clone()))
                                {
                                    let ts = ts.unwrap_or_else(Utc::now);
                                    let accum = sessions.entry(sid.clone()).or_default();
                                    accum.add_usage(&usage, ts);

                                    // Attach pending model
                                    if let Some(ref model) = pending_model {
                                        accum.models.insert(model.clone());
                                    }
                                    pending_model = None;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }
}

/// Parse `[2026-03-14 17:32:05]` from the start of a log line
fn parse_line_timestamp(line: &str) -> Option<DateTime<Utc>> {
    let line = line.strip_prefix('[')?;
    let end = line.find(']')?;
    let ts_str = &line[..end];
    let naive = NaiveDateTime::parse_from_str(ts_str, "%Y-%m-%d %H:%M:%S").ok()?;
    Some(Utc.from_utc_datetime(&naive))
}

/// Extract session ID from `Extracted session ID: <UUID>` lines
fn extract_session_id(line: &str) -> Option<&str> {
    let marker = "Extracted session ID: ";
    let pos = line.find(marker)?;
    let rest = &line[pos + marker.len()..];
    // UUID is 36 chars: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
    if rest.len() >= 36 {
        Some(&rest[..36])
    } else {
        None
    }
}

/// Extract the JSON blob after `Translated Anthropic event: `
fn extract_event_json(line: &str) -> Option<&str> {
    let marker = "Translated Anthropic event: ";
    let pos = line.find(marker)?;
    Some(&line[pos + marker.len()..])
}

/// Normalize Copilot internal model names to clean identifiers
///
/// - `gpt-5.4-2026-03-05` → `gpt-5.4`
/// - `capi-noe-ptuc-h200-ib-gpt-5-mini-2025-08-07` → `gpt-5-mini`
/// - `gpt-5.3-codex` → `gpt-5.3-codex` (unchanged)
fn normalize_model_name(raw: &str) -> String {
    // Strip `capi-...-ib-` deployment prefix
    let name = if let Some(ib_pos) = raw.find("-ib-") {
        &raw[ib_pos + 4..]
    } else {
        raw
    };

    // Strip trailing date suffix `-YYYY-MM-DD`
    let name = if name.len() > 11 {
        let suffix = &name[name.len() - 11..];
        if suffix.starts_with('-')
            && suffix[1..5].chars().all(|c| c.is_ascii_digit())
            && suffix.as_bytes()[5] == b'-'
        {
            &name[..name.len() - 11]
        } else {
            name
        }
    } else {
        name
    };

    name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const SAMPLE_LOG: &str = r#"[2026-03-14 17:32:05] [debug] [messages-handler] Extracted session ID: 77ff1551-d45d-8b24-f206-74b9d1de6ea9
[2026-03-14 17:32:15] [debug] [messages-handler] Translated Anthropic event: {"type":"message_start","message":{"id":"abc","type":"message","role":"assistant","content":[],"model":"gpt-5.4-2026-03-05","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0}}}
[2026-03-14 17:32:16] [debug] [messages-handler] Translated Anthropic event: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"input_tokens":52055,"output_tokens":455,"cache_read_input_tokens":0}}
[2026-03-14 17:32:45] [debug] [messages-handler] Extracted session ID: 47e7be02-df79-a885-4c4f-e2bf1c83587f
[2026-03-14 17:32:59] [debug] [messages-handler] Translated Anthropic event: {"type":"message_start","message":{"id":"def","type":"message","role":"assistant","content":[],"model":"gpt-5.3-codex","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0}}}
[2026-03-14 17:33:00] [debug] [messages-handler] Translated Anthropic event: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"input_tokens":52067,"output_tokens":582,"cache_read_input_tokens":0}}
[2026-03-14 17:33:42] [debug] [messages-handler] Translated Anthropic event: {"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"input_tokens":404,"output_tokens":2125,"cache_read_input_tokens":51968}}
"#;

    fn write_log(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(normalize_model_name("gpt-5.4-2026-03-05"), "gpt-5.4");
        assert_eq!(normalize_model_name("gpt-5.3-codex"), "gpt-5.3-codex");
        assert_eq!(
            normalize_model_name("capi-noe-ptuc-h200-ib-gpt-5-mini-2025-08-07"),
            "gpt-5-mini"
        );
    }

    #[test]
    fn test_parse_session_id_extraction() {
        let line = "[2026-03-14 17:32:05] [debug] [messages-handler] Extracted session ID: 77ff1551-d45d-8b24-f206-74b9d1de6ea9";
        assert_eq!(
            extract_session_id(line),
            Some("77ff1551-d45d-8b24-f206-74b9d1de6ea9")
        );
    }

    #[test]
    fn test_parse_timestamp() {
        let line = "[2026-03-14 17:32:05] [debug] something";
        let ts = parse_line_timestamp(line).unwrap();
        assert_eq!(ts.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-03-14 17:32:05");
    }

    #[test]
    fn test_scan_all_two_sessions() {
        let dir = tempdir().unwrap();
        write_log(dir.path(), "messages-handler-2026-03-14.log", SAMPLE_LOG);

        let mut report = LoadReport::default();
        let mut sessions = CopilotParser::scan_all(dir.path(), &mut report);
        sessions.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));

        assert_eq!(sessions.len(), 2, "Should find 2 sessions");
        assert_eq!(report.sessions_scanned, 2);

        // Session 47e7be02: 2 message_delta events
        let s2 = sessions.iter().find(|s| s.id.starts_with("47e7be02")).unwrap();
        assert_eq!(s2.message_count, 2);
        assert_eq!(s2.input_tokens, 52067 + 404);
        assert_eq!(s2.output_tokens, 582 + 2125);
        assert_eq!(s2.cache_read_tokens, 51968);
        assert_eq!(s2.models_used, vec!["gpt-5.3-codex"]);
        assert_eq!(s2.source_tool, Some("copilot".to_string()));

        // Session 77ff1551: 1 message_delta event
        let s1 = sessions.iter().find(|s| s.id.starts_with("77ff1551")).unwrap();
        assert_eq!(s1.message_count, 1);
        assert_eq!(s1.input_tokens, 52055);
        assert_eq!(s1.output_tokens, 455);
        assert_eq!(s1.models_used, vec!["gpt-5.4"]);
    }

    #[test]
    fn test_is_available() {
        let dir = tempdir().unwrap();
        assert!(!CopilotParser::is_available(dir.path()));

        write_log(dir.path(), "messages-handler-2026-01-01.log", SAMPLE_LOG);
        assert!(CopilotParser::is_available(dir.path()));
    }
}
