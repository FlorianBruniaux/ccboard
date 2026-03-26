//! Hook-based live session state
//!
//! Tracks Claude Code session status via hook events, written to
//! ~/.ccboard/live-sessions.json with file locking for concurrent safety.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Composite key: `"{session_id}:{tty}"` — unique per session per terminal
pub type SessionKey = String;

/// Build a session key from session_id and tty
pub fn make_session_key(session_id: &str, tty: &str) -> SessionKey {
    format!("{}:{}", session_id, tty)
}

/// Status of a Claude Code session as observed via hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HookSessionStatus {
    /// Claude is actively processing (PreToolUse, PostToolUse, UserPromptSubmit)
    Running,
    /// Waiting for user permission (Notification with permission_prompt)
    WaitingInput,
    /// Session has ended (Stop hook received)
    Stopped,
    /// Unknown status
    #[default]
    Unknown,
}

/// A Claude Code session tracked via hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSession {
    /// Claude session ID (from hook payload)
    pub session_id: String,
    /// Working directory of the Claude process
    pub cwd: String,
    /// TTY device path (e.g. "/dev/ttys001")
    pub tty: String,
    /// Current status
    pub status: HookSessionStatus,
    /// When this session was first seen
    pub created_at: DateTime<Utc>,
    /// When this session was last updated
    pub updated_at: DateTime<Utc>,
    /// Name of the last hook event received
    pub last_event: String,
}

/// Contents of ~/.ccboard/live-sessions.json
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiveSessionFile {
    /// Schema version — always 1 for now, used for future migrations
    pub version: u8,
    /// Active sessions keyed by "{session_id}:{tty}"
    pub sessions: HashMap<SessionKey, HookSession>,
    /// When this file was last written
    pub updated_at: Option<DateTime<Utc>>,
}

/// Errors produced by hook state operations
#[derive(Debug, Error)]
pub enum HookStateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No home directory found")]
    NoHome,
}

impl LiveSessionFile {
    /// Default path: `~/.ccboard/live-sessions.json`
    pub fn default_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".ccboard").join("live-sessions.json"))
    }

    /// Lock file path: `~/.ccboard/live-sessions.lock`
    pub fn lock_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".ccboard").join("live-sessions.lock"))
    }

    /// Load from disk. Returns `Default` if file does not exist; errors only on parse failure.
    pub fn load(path: &Path) -> Result<Self, HookStateError> {
        if !path.exists() {
            return Ok(Self {
                version: 1,
                ..Default::default()
            });
        }
        let data = std::fs::read(path)?;
        let mut file: Self = serde_json::from_slice(&data)?;
        // Ensure version is set on old files
        if file.version == 0 {
            file.version = 1;
        }
        Ok(file)
    }

    /// Atomic write: write to `.tmp`, then rename (APFS/ext4-safe)
    pub fn save(&self, path: &Path) -> Result<(), HookStateError> {
        let tmp_path = path.with_extension("tmp");
        let data = serde_json::to_vec_pretty(self)?;
        std::fs::write(&tmp_path, &data)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Remove `Stopped` sessions older than `max_age`
    pub fn prune_stopped(&mut self, max_age: std::time::Duration) {
        let cutoff =
            Utc::now() - Duration::from_std(max_age).unwrap_or_else(|_| Duration::minutes(30));
        self.sessions
            .retain(|_, s| s.status != HookSessionStatus::Stopped || s.updated_at >= cutoff);
    }

    /// Remove `Running` / `WaitingInput` sessions not updated within `max_age`.
    ///
    /// Handles Claude processes that crashed or were killed without sending a Stop hook.
    /// Default recommended value: 10 minutes.
    pub fn prune_stale_running(&mut self, max_age: std::time::Duration) {
        let cutoff =
            Utc::now() - Duration::from_std(max_age).unwrap_or_else(|_| Duration::minutes(10));
        self.sessions.retain(|_, s| {
            // Stopped sessions are handled by prune_stopped — leave them alone here
            s.status == HookSessionStatus::Stopped || s.updated_at >= cutoff
        });
    }

    /// Upsert a session: create if new, update status/timestamp if existing.
    /// Special rule: `UserPromptSubmit` on a `Stopped` session revives it to `Running`.
    pub fn upsert(
        &mut self,
        key: SessionKey,
        session_id: String,
        cwd: String,
        tty: String,
        new_status: HookSessionStatus,
        event_name: String,
    ) {
        let now = Utc::now();

        // If the session was Stopped and we get a Running event → revive it
        let effective_status = if new_status == HookSessionStatus::Running
            && self
                .sessions
                .get(&key)
                .map(|s| s.status == HookSessionStatus::Stopped)
                .unwrap_or(false)
        {
            HookSessionStatus::Running
        } else {
            new_status
        };

        if let Some(existing) = self.sessions.get_mut(&key) {
            existing.status = effective_status;
            existing.updated_at = now;
            existing.last_event = event_name;
        } else {
            self.sessions.insert(
                key,
                HookSession {
                    session_id,
                    cwd,
                    tty,
                    status: effective_status,
                    created_at: now,
                    updated_at: now,
                    last_event: event_name,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_prune_stopped_removes_old() {
        let mut file = LiveSessionFile {
            version: 1,
            ..Default::default()
        };

        let old_time = Utc::now() - Duration::from_secs(31 * 60); // 31 minutes ago
        file.sessions.insert(
            "s1:tty1".to_string(),
            HookSession {
                session_id: "s1".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty1".to_string(),
                status: HookSessionStatus::Stopped,
                created_at: old_time,
                updated_at: old_time,
                last_event: "Stop".to_string(),
            },
        );

        // Running session should survive
        file.sessions.insert(
            "s2:tty2".to_string(),
            HookSession {
                session_id: "s2".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty2".to_string(),
                status: HookSessionStatus::Running,
                created_at: old_time,
                updated_at: old_time,
                last_event: "PreToolUse".to_string(),
            },
        );

        file.prune_stopped(Duration::from_secs(30 * 60));

        assert!(!file.sessions.contains_key("s1:tty1"));
        assert!(file.sessions.contains_key("s2:tty2"));
    }

    #[test]
    fn test_upsert_revives_stopped() {
        let mut file = LiveSessionFile {
            version: 1,
            ..Default::default()
        };

        let key = "s1:tty1".to_string();
        let old_time = Utc::now() - chrono::Duration::seconds(5);

        file.sessions.insert(
            key.clone(),
            HookSession {
                session_id: "s1".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty1".to_string(),
                status: HookSessionStatus::Stopped,
                created_at: old_time,
                updated_at: old_time,
                last_event: "Stop".to_string(),
            },
        );

        file.upsert(
            key.clone(),
            "s1".to_string(),
            "/tmp".to_string(),
            "tty1".to_string(),
            HookSessionStatus::Running,
            "UserPromptSubmit".to_string(),
        );

        assert_eq!(file.sessions[&key].status, HookSessionStatus::Running);
    }

    #[test]
    fn test_prune_stale_running_removes_stale() {
        let mut file = LiveSessionFile {
            version: 1,
            ..Default::default()
        };

        let stale_time = Utc::now() - Duration::from_secs(11 * 60);
        let recent_time = Utc::now() - Duration::from_secs(60);

        // Stale Running session — should be removed
        file.sessions.insert(
            "s1:tty1".to_string(),
            HookSession {
                session_id: "s1".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty1".to_string(),
                status: HookSessionStatus::Running,
                created_at: stale_time,
                updated_at: stale_time,
                last_event: "PreToolUse".to_string(),
            },
        );

        // Stale WaitingInput session — should be removed
        file.sessions.insert(
            "s2:tty2".to_string(),
            HookSession {
                session_id: "s2".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty2".to_string(),
                status: HookSessionStatus::WaitingInput,
                created_at: stale_time,
                updated_at: stale_time,
                last_event: "Notification".to_string(),
            },
        );

        // Recent Running session — should survive
        file.sessions.insert(
            "s3:tty3".to_string(),
            HookSession {
                session_id: "s3".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty3".to_string(),
                status: HookSessionStatus::Running,
                created_at: recent_time,
                updated_at: recent_time,
                last_event: "PreToolUse".to_string(),
            },
        );

        // Old Stopped session — prune_stale_running should NOT touch it
        file.sessions.insert(
            "s4:tty4".to_string(),
            HookSession {
                session_id: "s4".to_string(),
                cwd: "/tmp".to_string(),
                tty: "tty4".to_string(),
                status: HookSessionStatus::Stopped,
                created_at: stale_time,
                updated_at: stale_time,
                last_event: "Stop".to_string(),
            },
        );

        file.prune_stale_running(Duration::from_secs(10 * 60));

        assert!(!file.sessions.contains_key("s1:tty1"), "stale Running should be pruned");
        assert!(!file.sessions.contains_key("s2:tty2"), "stale WaitingInput should be pruned");
        assert!(file.sessions.contains_key("s3:tty3"), "recent Running should survive");
        assert!(file.sessions.contains_key("s4:tty4"), "Stopped handled by prune_stopped, not touched here");
    }

    #[test]
    fn test_make_session_key() {
        assert_eq!(
            make_session_key("abc-123", "/dev/ttys001"),
            "abc-123:/dev/ttys001"
        );
    }
}
