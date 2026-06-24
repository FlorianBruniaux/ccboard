//! Parser for OpenAI Codex CLI sessions.
//!
//! Codex stores sessions under `~/.codex/sessions/YYYY/MM/DD/rollout-<uuid>.jsonl`.
//! Each file is a JSONL stream of event objects. We extract metadata by counting
//! non-empty lines (approximation of message count) and inferring timestamps from
//! the directory structure.

use crate::models::session::{ProjectId, SessionId, SessionMetadata, SourceTool};
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

pub struct CodexParser;

impl CodexParser {
    /// Returns the default Codex sessions directory for this OS.
    pub fn default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".codex").join("sessions"))
    }

    /// Scan all Codex sessions under `sessions_dir`.
    ///
    /// Walks YYYY/MM/DD sub-directories and collects one `SessionMetadata`
    /// per `rollout-*.jsonl` file. Errors are logged and skipped — never
    /// propagated — to honour the graceful-degradation contract.
    pub async fn scan(sessions_dir: &Path) -> Vec<SessionMetadata> {
        let mut results = Vec::new();

        let year_entries = match std::fs::read_dir(sessions_dir) {
            Ok(e) => e,
            Err(e) => {
                warn!(path = %sessions_dir.display(), error = %e, "Cannot read Codex sessions dir");
                return results;
            }
        };

        for year_entry in year_entries.flatten() {
            let year_path = year_entry.path();
            if !year_path.is_dir() {
                continue;
            }
            let month_entries = match std::fs::read_dir(&year_path) {
                Ok(e) => e,
                Err(_) => continue,
            };
            for month_entry in month_entries.flatten() {
                let month_path = month_entry.path();
                if !month_path.is_dir() {
                    continue;
                }
                let day_entries = match std::fs::read_dir(&month_path) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                for day_entry in day_entries.flatten() {
                    let day_path = day_entry.path();
                    if !day_path.is_dir() {
                        continue;
                    }
                    let file_entries = match std::fs::read_dir(&day_path) {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    for file_entry in file_entries.flatten() {
                        let file_path = file_entry.path();
                        if file_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                            continue;
                        }
                        if let Some(meta) = Self::parse_session_file(&file_path) {
                            results.push(meta);
                        }
                    }
                }
            }
        }

        debug!(count = results.len(), "Codex sessions scanned");
        results
    }

    /// Parse a single Codex JSONL file into `SessionMetadata`.
    fn parse_session_file(file_path: &Path) -> Option<SessionMetadata> {
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Build a unique session ID that won't collide with Claude Code UUIDs.
        let session_id = SessionId::new(format!("codex:{}", stem));

        // Infer date from path: .../YYYY/MM/DD/rollout-...jsonl
        // We need four ancestors: file → day → month → year → sessions_root
        let day_dir = file_path.parent()?;
        let month_dir = day_dir.parent()?;
        let year_dir = month_dir.parent()?;

        let year_str = year_dir.file_name()?.to_str()?;
        let month_str = month_dir.file_name()?.to_str()?;
        let day_str = day_dir.file_name()?.to_str()?;

        let timestamp = parse_ymd(year_str, month_str, day_str);

        // Count non-empty lines as a proxy for message count.
        let (message_count, file_size_bytes) = count_lines(file_path);

        let mut meta = SessionMetadata::from_path(
            file_path.to_path_buf(),
            ProjectId::from("codex://sessions"),
        );
        meta.id = session_id;
        meta.first_timestamp = timestamp;
        meta.last_timestamp = timestamp;
        meta.message_count = message_count;
        meta.file_size_bytes = file_size_bytes;
        meta.source_tool = SourceTool::Codex;

        Some(meta)
    }
}

/// Parse YYYY/MM/DD strings into a UTC `DateTime` at midnight.
fn parse_ymd(year: &str, month: &str, day: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    use chrono::{NaiveDate, TimeZone, Utc};
    let y: i32 = year.parse().ok()?;
    let m: u32 = month.parse().ok()?;
    let d: u32 = day.parse().ok()?;
    let naive = NaiveDate::from_ymd_opt(y, m, d)?.and_hms_opt(0, 0, 0)?;
    Some(Utc.from_utc_datetime(&naive))
}

/// Count non-empty lines and return (line_count, file_size_bytes).
fn count_lines(path: &Path) -> (u64, u64) {
    use std::io::{BufRead, BufReader};

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (0, 0),
    };
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let reader = BufReader::new(file);
    let mut count: u64 = 0;
    for line in reader.lines() {
        match line {
            Ok(l) if !l.trim().is_empty() => count += 1,
            Ok(_) => {}
            Err(_) => break,
        }
    }
    (count, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ymd_valid() {
        let dt = parse_ymd("2024", "03", "15");
        assert!(dt.is_some());
        let dt = dt.unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-03-15");
    }

    #[test]
    fn test_parse_ymd_invalid() {
        assert!(parse_ymd("bad", "03", "15").is_none());
        assert!(parse_ymd("2024", "13", "01").is_none());
    }
}
