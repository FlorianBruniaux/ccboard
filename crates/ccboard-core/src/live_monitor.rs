//! Live Claude Code session detection
//!
//! Detects running Claude Code processes on the system and provides metadata
//! about active sessions (PID, working directory, duration since start).
//!
//! Also provides `MergedLiveSession` which combines hook-based status data
//! (from ~/.ccboard/live-sessions.json) with ps-based process data.

use crate::hook_state::{HookSessionStatus, LiveSessionFile};
use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use std::process::Command;

/// Type of Claude Code session, detected from CLI flags
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SessionType {
    /// Standard `claude` CLI invocation
    #[default]
    Cli,
    /// IDE integration (--output-format stream-json + --permission-prompt-tool stdio)
    VsCode,
    /// Sub-agent (--output-format stream-json, no --permission-prompt-tool)
    Subagent,
}

impl SessionType {
    pub fn label(&self) -> &'static str {
        match self {
            SessionType::Cli => "CLI",
            SessionType::VsCode => "IDE",
            SessionType::Subagent => "Agent",
        }
    }
}

/// Parsed CLI flags from the claude command line
struct ParsedFlags {
    session_type: SessionType,
    model: Option<String>,
    resume_id: Option<String>,
}

/// Extract the value following a flag from a command string.
/// E.g. `extract_flag_value("claude --model opus-4", "--model")` → `Some("opus-4")`
fn extract_flag_value(command: &str, flag: &str) -> Option<String> {
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i] == flag {
            return Some(tokens[i + 1].to_string());
        }
    }
    None
}

/// Parse CLI flags from a claude command string to determine session type and metadata.
fn parse_claude_flags(command: &str) -> ParsedFlags {
    let has_stream_json = command.contains("stream-json");
    let has_stdio_tool = command.contains("permission-prompt-tool") && command.contains("stdio");

    let session_type = if has_stream_json && has_stdio_tool {
        SessionType::VsCode
    } else if has_stream_json {
        SessionType::Subagent
    } else {
        SessionType::Cli
    };

    let model = extract_flag_value(command, "--model");
    let resume_id = extract_flag_value(command, "--resume");

    ParsedFlags {
        session_type,
        model,
        resume_id,
    }
}

/// Represents a live Claude Code session (running process)
#[derive(Debug, Clone)]
pub struct LiveSession {
    /// Process ID
    pub pid: u32,
    /// Time when the process started
    pub start_time: DateTime<Local>,
    /// Working directory of the process (if detectable)
    pub working_directory: Option<String>,
    /// Full command line
    pub command: String,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Total tokens in active session (if detectable)
    pub tokens: Option<u64>,
    /// Session ID (from JSONL filename)
    pub session_id: Option<String>,
    /// Session name/title (from session_start event)
    pub session_name: Option<String>,
    /// Type of session (CLI / IDE / Agent), detected from CLI flags
    pub session_type: SessionType,
    /// Model in use (from --model flag, if present)
    pub model: Option<String>,
    /// Resume session ID (from --resume flag, may differ from session_id)
    pub resume_id: Option<String>,
}

/// Detect all running Claude Code processes on the system
///
/// Uses platform-specific commands:
/// - Unix (macOS/Linux): `ps aux` to list processes
/// - Windows: `tasklist` with CSV output
///
/// # Returns
/// Vector of LiveSession structs, one per detected Claude process.
/// Returns empty vector on error or if no Claude processes are running.
pub fn detect_live_sessions() -> Result<Vec<LiveSession>> {
    #[cfg(unix)]
    {
        detect_live_sessions_unix()
    }

    #[cfg(windows)]
    {
        detect_live_sessions_windows()
    }
}

#[cfg(unix)]
fn detect_live_sessions_unix() -> Result<Vec<LiveSession>> {
    // Run ps aux to get all processes
    let output = Command::new("ps")
        .args(["aux"])
        .output()
        .context("Failed to run ps command")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<LiveSession> = stdout
        .lines()
        .filter(|line| is_claude_process_line(line))
        .filter_map(parse_ps_line)
        .collect();

    Ok(sessions)
}

/// Returns true if a `ps aux` line belongs to a Claude Code process.
///
/// Checks the COMMAND column (field 10, 0-indexed) basename is exactly `claude`
/// or `claude-code`, avoiding false matches on `claude-desktop`, scripts, grep, etc.
#[cfg(unix)]
fn is_claude_process_line(line: &str) -> bool {
    if line.contains("grep") || line.contains("ccboard") {
        return false;
    }
    // ps aux columns: USER PID %CPU %MEM VSZ RSS TTY STAT START TIME COMMAND…
    // split_whitespace() collapses multiple spaces — skip 10 columns to reach COMMAND
    let mut fields = line.split_whitespace();
    for _ in 0..10 {
        if fields.next().is_none() {
            return false;
        }
    }
    // First token of COMMAND is the binary (possibly a full path)
    let binary = fields.next().unwrap_or("");
    let base = binary.rsplit('/').next().unwrap_or(binary);
    base == "claude" || base == "claude-code"
}

#[cfg(unix)]
fn parse_ps_line(line: &str) -> Option<LiveSession> {
    // ps aux format:
    // USER  PID  %CPU %MEM  VSZ   RSS  TTY  STAT START TIME COMMAND
    // 0     1    2    3     4     5    6    7    8     9    10+
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 11 {
        return None;
    }

    let pid = parts[1].parse::<u32>().ok()?;
    let cpu_percent = parts[2].parse::<f64>().unwrap_or(0.0);
    let memory_mb = parts[5].parse::<u64>().unwrap_or(0) / 1024; // RSS in KB → MB
    let start_str = parts[8]; // START column (HH:MM or MMM DD)
    let command = parts[10..].join(" ");

    // Parse CLI flags (session type, model, resume ID)
    let flags = parse_claude_flags(&command);

    // Parse start time (best effort - format varies by OS and process age)
    let start_time = parse_start_time(start_str).unwrap_or_else(Local::now);

    // Try to get working directory for this PID
    let working_directory = get_cwd_for_pid(pid);

    // Try to extract session metadata (tokens, ID, name)
    let session_metadata = get_session_metadata(&working_directory);

    Some(LiveSession {
        pid,
        start_time,
        working_directory,
        command,
        cpu_percent,
        memory_mb,
        tokens: session_metadata.as_ref().and_then(|m| m.tokens),
        session_id: session_metadata.as_ref().and_then(|m| m.session_id.clone()),
        session_name: session_metadata
            .as_ref()
            .and_then(|m| m.session_name.clone()),
        session_type: flags.session_type,
        model: flags.model,
        resume_id: flags.resume_id,
    })
}

#[cfg(unix)]
fn parse_start_time(start_str: &str) -> Option<DateTime<Local>> {
    // ps START column format varies:
    // - If process started today: "HH:MM" (e.g., "14:30")
    // - If process started earlier: "MMM DD" (e.g., "Feb 04")
    //
    // For simplicity, if it contains ":", assume today's date with that time.
    // Otherwise, fall back to current time (imprecise but acceptable).

    if start_str.contains(':') {
        // Format: "HH:MM" - assume today
        let parts: Vec<&str> = start_str.split(':').collect();
        if parts.len() == 2 {
            if let (Ok(hour), Ok(minute)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                let now = Local::now();
                return now
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .and_then(|dt| Local.from_local_datetime(&dt).single());
            }
        }
    }

    // Fallback: can't parse reliably, return None
    None
}

#[cfg(unix)]
fn get_cwd_for_pid(pid: u32) -> Option<String> {
    // Platform-specific working directory detection
    #[cfg(target_os = "linux")]
    {
        // On Linux: readlink /proc/PID/cwd
        std::fs::read_link(format!("/proc/{}/cwd", pid))
            .ok()
            .and_then(|p| p.to_str().map(String::from))
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS: lsof -p PID -Fn (returns file descriptors, including cwd)
        let output = Command::new("lsof")
            .args(["-p", &pid.to_string(), "-a", "-d", "cwd", "-Fn"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // lsof -Fn output format: "n/path/to/cwd"
        stdout
            .lines()
            .find(|line| line.starts_with('n'))
            .and_then(|line| line.strip_prefix('n'))
            .map(String::from)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        // Other Unix systems: not implemented
        None
    }
}

#[cfg(windows)]
fn detect_live_sessions_windows() -> Result<Vec<LiveSession>> {
    // Run tasklist with CSV output for parsing
    let output = Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq claude.exe", "/FO", "CSV", "/NH"])
        .output()
        .context("Failed to run tasklist command")?;

    if !output.status.success() {
        return Ok(vec![]);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sessions: Vec<LiveSession> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| parse_tasklist_csv(line))
        .collect();

    Ok(sessions)
}

#[cfg(windows)]
fn parse_tasklist_csv(line: &str) -> Option<LiveSession> {
    // CSV format (no header): "ImageName","PID","SessionName","Session#","MemUsage"
    // Example: "claude.exe","12345","Console","1","50,000 K"
    let parts: Vec<&str> = line.split(',').map(|s| s.trim_matches('"')).collect();
    if parts.len() < 2 {
        return None;
    }

    let pid = parts[1].parse::<u32>().ok()?;
    let command = parts[0].to_string();

    // Windows tasklist doesn't provide start time or cwd easily
    // Use current time as approximate start (limitation of Windows API via tasklist)
    let start_time = Local::now();
    let working_directory = None; // Not available via tasklist

    Some(LiveSession {
        pid,
        start_time,
        working_directory,
        command,
        cpu_percent: 0.0,
        memory_mb: 0,
        tokens: None,
        session_id: None,
        session_name: None,
        session_type: SessionType::Cli,
        model: None,
        resume_id: None,
    })
}

/// Session metadata extracted from active JSONL file
struct LiveSessionMetadata {
    tokens: Option<u64>,
    session_id: Option<String>,
    session_name: Option<String>,
}

/// Extract session metadata from active session JSONL file
///
/// Given a working directory (e.g., /Users/foo/myproject), attempts to:
/// 1. Encode the path to match ~/.claude/projects/<encoded>/ format
/// 2. Find the most recent .jsonl file in that directory
/// 3. Parse tokens, session ID, and session name
///
/// Returns None if:
/// - Working directory is None
/// - Session directory doesn't exist
/// - No JSONL files found
/// - Parse errors occur
fn get_session_metadata(working_directory: &Option<String>) -> Option<LiveSessionMetadata> {
    let cwd = working_directory.as_ref()?;

    // Encode path: /Users/foo/myproject → -Users-foo-myproject
    // The leading '/' becomes '-' when replaced, so no need for format!("-{}")
    let encoded = cwd.replace('/', "-");

    // Build sessions directory path
    let home = dirs::home_dir()?;
    let sessions_dir = home.join(".claude").join("projects").join(&encoded);

    if !sessions_dir.exists() {
        return None;
    }

    // Find most recent .jsonl file
    let mut entries: Vec<_> = std::fs::read_dir(&sessions_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s == "jsonl")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());
    let latest = entries.last()?.path();

    // Extract session ID from filename (e.g., "abc123.jsonl" -> "abc123")
    let session_id = latest
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from);

    // Parse JSONL and sum tokens + extract session name
    let file = std::fs::File::open(latest).ok()?;
    let reader = std::io::BufReader::new(file);
    let mut total_tokens = 0u64;
    let mut session_name: Option<String> = None;

    for line in std::io::BufRead::lines(reader) {
        // Skip lines that fail to read (don't fail the entire function)
        let Ok(line) = line else { continue };

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            // Extract session name from session_start event
            if session_name.is_none() {
                if let Some(event_type) = json.get("type").and_then(|v| v.as_str()) {
                    if event_type == "session_start" {
                        session_name = json.get("name").and_then(|v| v.as_str()).map(String::from);
                    }
                }
            }

            // Usage is nested in .message.usage, not at root level
            if let Some(message) = json.get("message") {
                if let Some(usage) = message.get("usage") {
                    // Sum all token types
                    if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        total_tokens += input;
                    }
                    if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        total_tokens += output;
                    }
                    // Note: Field names differ from stats-cache.json:
                    // - cache_creation_input_tokens (not cache_write_tokens)
                    // - cache_read_input_tokens (not cache_read_tokens)
                    if let Some(cache_write) = usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                    {
                        total_tokens += cache_write;
                    }
                    if let Some(cache_read) = usage
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64())
                    {
                        total_tokens += cache_read;
                    }
                }
            }
        }
    }

    Some(LiveSessionMetadata {
        tokens: if total_tokens > 0 {
            Some(total_tokens)
        } else {
            None
        },
        session_id,
        session_name,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Merged live sessions (hook + ps)
// ─────────────────────────────────────────────────────────────────────────────

/// Display status computed from hook data and ps-based fallback
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveSessionDisplayStatus {
    /// Hook says Running
    Running,
    /// Hook says WaitingInput (permission prompt)
    WaitingInput,
    /// Hook says Stopped
    Stopped,
    /// No hooks — detected via ps only
    ProcessOnly,
    /// Unknown (hooks present but status unclear)
    Unknown,
}

impl LiveSessionDisplayStatus {
    /// Short icon for TUI display
    pub fn icon(&self) -> &'static str {
        match self {
            LiveSessionDisplayStatus::Running => "●",
            LiveSessionDisplayStatus::WaitingInput => "◐",
            LiveSessionDisplayStatus::Stopped => "✓",
            LiveSessionDisplayStatus::ProcessOnly => "🟢",
            LiveSessionDisplayStatus::Unknown => "?",
        }
    }
}

/// A merged view of hook data + ps-based process data for one Claude session
#[derive(Debug, Clone)]
pub struct MergedLiveSession {
    /// Session ID (from hook data or ps metadata)
    pub session_id: Option<String>,
    /// Working directory
    pub cwd: String,
    /// TTY device (from hook data)
    pub tty: Option<String>,
    /// Status from hook events (None = no hooks for this session)
    pub hook_status: Option<HookSessionStatus>,
    /// Underlying ps-detected process (None = hook-only, ps couldn't find it yet)
    pub process: Option<LiveSession>,
    /// When the last hook event was received
    pub last_event_at: Option<DateTime<Local>>,
    /// Name of the last hook event
    pub last_event: Option<String>,
}

impl MergedLiveSession {
    /// Compute the effective display status
    pub fn effective_status(&self) -> LiveSessionDisplayStatus {
        match self.hook_status {
            Some(HookSessionStatus::Running) => LiveSessionDisplayStatus::Running,
            Some(HookSessionStatus::WaitingInput) => LiveSessionDisplayStatus::WaitingInput,
            Some(HookSessionStatus::Stopped) => LiveSessionDisplayStatus::Stopped,
            Some(HookSessionStatus::Unknown) | None => {
                if self.process.is_some() {
                    LiveSessionDisplayStatus::ProcessOnly
                } else {
                    LiveSessionDisplayStatus::Unknown
                }
            }
        }
    }

    /// Project name (basename of cwd)
    pub fn project_name(&self) -> &str {
        std::path::Path::new(&self.cwd)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&self.cwd)
    }
}

/// Merge hook file data with ps-based session list.
///
/// Priority order:
/// 1. session_id match (if both have session_id)
/// 2. TTY match
/// 3. cwd basename match (fallback)
///
/// Sessions present only in ps get `hook_status = None` (ProcessOnly).
/// Sessions present only in hooks (not yet visible to ps) are included.
pub fn merge_live_sessions(
    hook_file: &LiveSessionFile,
    ps_sessions: &[LiveSession],
) -> Vec<MergedLiveSession> {
    let mut result: Vec<MergedLiveSession> = Vec::new();
    let mut matched_ps: Vec<bool> = vec![false; ps_sessions.len()];

    // For each hook session, try to find a matching ps session
    for hook_session in hook_file.sessions.values() {
        let mut matched_ps_idx: Option<usize> = None;

        // Match by session_id
        if matched_ps_idx.is_none() {
            for (i, ps) in ps_sessions.iter().enumerate() {
                if matched_ps[i] {
                    continue;
                }
                if ps
                    .session_id
                    .as_deref()
                    .map(|id| id == hook_session.session_id)
                    .unwrap_or(false)
                {
                    matched_ps_idx = Some(i);
                    break;
                }
            }
        }

        // Match by TTY (ps TTY column vs hook tty)
        if matched_ps_idx.is_none() && hook_session.tty != "unknown" {
            let hook_tty_base = std::path::Path::new(&hook_session.tty)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&hook_session.tty);

            for (i, ps) in ps_sessions.iter().enumerate() {
                if matched_ps[i] {
                    continue;
                }

                // Fallback: match by cwd
                if ps
                    .working_directory
                    .as_deref()
                    .map(|wd| wd == hook_session.cwd)
                    .unwrap_or(false)
                {
                    matched_ps_idx = Some(i);
                    break;
                }

                // Try hook_tty_base against session command
                if ps.command.contains(hook_tty_base) {
                    matched_ps_idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = matched_ps_idx {
            matched_ps[idx] = true;
            let ps = &ps_sessions[idx];
            result.push(MergedLiveSession {
                session_id: Some(hook_session.session_id.clone()),
                cwd: hook_session.cwd.clone(),
                tty: Some(hook_session.tty.clone()),
                hook_status: Some(hook_session.status),
                process: Some(ps.clone()),
                last_event_at: Some(hook_session.updated_at.with_timezone(&Local)),
                last_event: Some(hook_session.last_event.clone()),
            });
        } else {
            // Hook-only (ps hasn't picked it up or process ended)
            result.push(MergedLiveSession {
                session_id: Some(hook_session.session_id.clone()),
                cwd: hook_session.cwd.clone(),
                tty: Some(hook_session.tty.clone()),
                hook_status: Some(hook_session.status),
                process: None,
                last_event_at: Some(hook_session.updated_at.with_timezone(&Local)),
                last_event: Some(hook_session.last_event.clone()),
            });
        }
    }

    // Remaining unmatched ps sessions → ProcessOnly
    for (i, ps) in ps_sessions.iter().enumerate() {
        if !matched_ps[i] {
            result.push(MergedLiveSession {
                session_id: ps.session_id.clone(),
                cwd: ps
                    .working_directory
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                tty: None,
                hook_status: None,
                process: Some(ps.clone()),
                last_event_at: None,
                last_event: None,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn test_parse_ps_line() {
        let line = "user  12345  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  /usr/local/bin/claude --session foo";
        let session = parse_ps_line(line).expect("Failed to parse valid ps line");
        assert_eq!(session.pid, 12345);
        assert!(session.command.contains("claude"));
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_ps_line_invalid() {
        let line = "user  invalid  0.0  0.1";
        assert!(parse_ps_line(line).is_none());
    }

    #[test]
    fn test_detect_live_sessions_no_panic() {
        // This test just ensures the function doesn't panic
        // It may return empty vec if no claude processes are running
        let result = detect_live_sessions();
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_start_time_today() {
        let result = parse_start_time("14:30");
        assert!(result.is_some());
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_start_time_fallback() {
        let result = parse_start_time("Feb 04");
        // Should return None for non-HH:MM format (can't reliably parse month/day)
        assert!(result.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_match() {
        let line = "user  12345  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  /usr/local/bin/claude --resume abc";
        assert!(is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_bare_claude() {
        let line = "user  12345  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  claude";
        assert!(is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_rejects_desktop() {
        // claude-desktop should NOT match
        let line = "user  99999  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  /Applications/Claude.app/claude-desktop";
        assert!(!is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_rejects_grep() {
        let line =
            "user  99999  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  grep claude";
        assert!(!is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_rejects_ccboard() {
        let line = "user  99999  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  ccboard hook PreToolUse";
        assert!(!is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_is_claude_process_line_rejects_script_with_claude_in_name() {
        let line = "user  88888  0.0  0.1  123456  78910  ttys001  S+   14:30   0:05.23  python3 claude_runner.py";
        assert!(!is_claude_process_line(line));
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_claude_flags_cli() {
        let flags = parse_claude_flags("/usr/local/bin/claude --resume abc");
        assert_eq!(flags.session_type, SessionType::Cli);
        assert_eq!(flags.resume_id.as_deref(), Some("abc"));
        assert!(flags.model.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_claude_flags_vscode() {
        let flags =
            parse_claude_flags("claude --output-format stream-json --permission-prompt-tool stdio");
        assert_eq!(flags.session_type, SessionType::VsCode);
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_claude_flags_subagent() {
        let flags = parse_claude_flags("claude --output-format stream-json --model claude-opus-4");
        assert_eq!(flags.session_type, SessionType::Subagent);
        assert_eq!(flags.model.as_deref(), Some("claude-opus-4"));
    }

    #[test]
    #[cfg(unix)]
    fn test_parse_claude_flags_no_flags() {
        let flags = parse_claude_flags("claude");
        assert_eq!(flags.session_type, SessionType::Cli);
        assert!(flags.model.is_none());
        assert!(flags.resume_id.is_none());
    }
}
