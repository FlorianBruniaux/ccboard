//! Hook payload deserialization and status mapping
//!
//! Claude Code fires hooks by writing JSON to stdin of the hook command.
//! The format uses camelCase except for `session_id`.

use crate::hook_state::HookSessionStatus;
use serde::Deserialize;

/// JSON payload received from Claude Code hooks via stdin
///
/// Claude Code sends all fields in snake_case. All optional fields use
/// `#[serde(default)]` for forward compatibility.
#[derive(Debug, Deserialize, Default)]
pub struct HookPayload {
    /// Session ID
    #[serde(default)]
    pub session_id: String,

    /// Working directory of the Claude Code process
    #[serde(default)]
    pub cwd: String,

    /// Type of notification (e.g. "permission_prompt") — only set on Notification hook
    #[serde(default)]
    pub notification_type: Option<String>,

    /// Path to the session transcript file
    #[serde(default)]
    pub transcript_path: Option<String>,

    /// Hook event name — populated by the binary, NOT from JSON
    #[serde(skip)]
    pub event_name: String,
}

/// Map a hook event name and payload to the resulting session status
pub fn status_from_event(event: &str, payload: &HookPayload) -> HookSessionStatus {
    match event {
        "PreToolUse" | "PostToolUse" | "UserPromptSubmit" => HookSessionStatus::Running,
        "Notification" => {
            if payload.notification_type.as_deref() == Some("permission_prompt") {
                HookSessionStatus::WaitingInput
            } else {
                HookSessionStatus::Running
            }
        }
        "Stop" => HookSessionStatus::Stopped,
        _ => HookSessionStatus::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_tool_use_is_running() {
        let payload = HookPayload::default();
        assert_eq!(
            status_from_event("PreToolUse", &payload),
            HookSessionStatus::Running
        );
    }

    #[test]
    fn test_stop_is_stopped() {
        let payload = HookPayload::default();
        assert_eq!(
            status_from_event("Stop", &payload),
            HookSessionStatus::Stopped
        );
    }

    #[test]
    fn test_notification_permission_prompt_is_waiting() {
        let payload = HookPayload {
            notification_type: Some("permission_prompt".to_string()),
            ..Default::default()
        };
        assert_eq!(
            status_from_event("Notification", &payload),
            HookSessionStatus::WaitingInput
        );
    }

    #[test]
    fn test_notification_other_is_running() {
        let payload = HookPayload {
            notification_type: Some("other".to_string()),
            ..Default::default()
        };
        assert_eq!(
            status_from_event("Notification", &payload),
            HookSessionStatus::Running
        );
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{
            "session_id": "abc-123",
            "cwd": "/Users/alice/project",
            "notification_type": "permission_prompt",
            "transcript_path": "/path/to/transcript"
        }"#;
        let payload: HookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.session_id, "abc-123");
        assert_eq!(payload.cwd, "/Users/alice/project");
        assert_eq!(
            payload.notification_type.as_deref(),
            Some("permission_prompt")
        );
        assert_eq!(
            payload.transcript_path.as_deref(),
            Some("/path/to/transcript")
        );
    }

    #[test]
    fn test_json_minimal() {
        // Only session_id and cwd (minimum required fields)
        let json = r#"{"session_id":"x","cwd":"/tmp"}"#;
        let payload: HookPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.session_id, "x");
        assert!(payload.notification_type.is_none());
    }
}
