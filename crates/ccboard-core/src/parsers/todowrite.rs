//! TodoWrite parser for session-to-task mapping
//!
//! Parses TodoWrite/TaskCreate/TaskUpdate tool calls from JSONL sessions
//! to reconstruct task timelines and map sessions to tasks.

use crate::models::session::SessionLine;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Task event extracted from session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    /// Session ID where task was created/updated
    pub session_id: String,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,

    /// Event type (created, updated, completed)
    pub event_type: TaskEventType,

    /// Task ID
    pub task_id: String,

    /// Task subject/title
    pub subject: Option<String>,

    /// Task description
    pub description: Option<String>,

    /// Task status (pending, in_progress, completed)
    pub status: Option<String>,
}

/// Type of task event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEventType {
    /// Task created
    Created,

    /// Task updated
    Updated,

    /// Task completed
    Completed,
}

/// Session-to-task mapping
#[derive(Debug, Clone, Default)]
pub struct SessionTaskMapping {
    /// Map session_id -> list of task events in that session
    pub session_tasks: HashMap<String, Vec<TaskEvent>>,

    /// Map task_id -> list of events for that task (timeline)
    pub task_timeline: HashMap<String, Vec<TaskEvent>>,
}

/// Parser for TodoWrite tool calls
pub struct TodoWriteParser;

impl TodoWriteParser {
    /// Parse a session file and extract task events
    pub fn parse_session(session_path: &Path) -> Result<Vec<TaskEvent>> {
        let file = File::open(session_path)
            .context(format!("Failed to open session file: {:?}", session_path))?;

        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;

            if line.trim().is_empty() {
                continue;
            }

            // Parse JSONL line
            let session_line: SessionLine = match serde_json::from_str(&line) {
                Ok(l) => l,
                Err(_) => continue, // Skip malformed lines
            };

            // Extract task events from this line
            if let Some(line_events) = Self::extract_task_events(&session_line) {
                events.extend(line_events);
            }
        }

        Ok(events)
    }

    /// Extract task events from a session line
    fn extract_task_events(line: &SessionLine) -> Option<Vec<TaskEvent>> {
        let message = line.message.as_ref()?;
        let tool_calls = message.tool_calls.as_ref()?;

        let mut events = Vec::new();

        for tool_call in tool_calls {
            // Parse tool call as serde_json::Value
            if let Some(tool_name) = tool_call.get("name").and_then(|v| v.as_str()) {
                match tool_name {
                    "TaskCreate" | "TodoWrite" => {
                        if let Some(event) = Self::parse_task_create(line, tool_call) {
                            events.push(event);
                        }
                    }
                    "TaskUpdate" => {
                        if let Some(event) = Self::parse_task_update(line, tool_call) {
                            events.push(event);
                        }
                    }
                    _ => {}
                }
            }
        }

        if events.is_empty() {
            None
        } else {
            Some(events)
        }
    }

    /// Parse TaskCreate/TodoWrite tool call
    fn parse_task_create(line: &SessionLine, tool_call: &serde_json::Value) -> Option<TaskEvent> {
        let input = tool_call.get("input")?;

        let session_id = line.session_id.clone().unwrap_or_default();
        let timestamp = line.timestamp.unwrap_or_else(Utc::now);

        let subject = input
            .get("subject")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = input
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Generate task ID from timestamp if not provided
        let task_id = input
            .get("taskId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("task_{}", timestamp.timestamp()));

        Some(TaskEvent {
            session_id,
            timestamp,
            event_type: TaskEventType::Created,
            task_id,
            subject,
            description,
            status: Some("pending".to_string()),
        })
    }

    /// Parse TaskUpdate tool call
    fn parse_task_update(line: &SessionLine, tool_call: &serde_json::Value) -> Option<TaskEvent> {
        let input = tool_call.get("input")?;

        let session_id = line.session_id.clone().unwrap_or_default();
        let timestamp = line.timestamp.unwrap_or_else(Utc::now);

        let task_id = input
            .get("taskId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())?;

        let status = input
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let subject = input
            .get("subject")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let description = input
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Determine event type based on status
        let event_type = match status.as_deref() {
            Some("completed") => TaskEventType::Completed,
            _ => TaskEventType::Updated,
        };

        Some(TaskEvent {
            session_id,
            timestamp,
            event_type,
            task_id,
            subject,
            description,
            status,
        })
    }

    /// Build session-to-task mapping from events
    pub fn build_mapping(events: Vec<TaskEvent>) -> SessionTaskMapping {
        let mut mapping = SessionTaskMapping::default();

        for event in events {
            // Add to session_tasks map
            mapping
                .session_tasks
                .entry(event.session_id.clone())
                .or_default()
                .push(event.clone());

            // Add to task_timeline map
            mapping
                .task_timeline
                .entry(event.task_id.clone())
                .or_default()
                .push(event);
        }

        // Sort timelines by timestamp
        for timeline in mapping.task_timeline.values_mut() {
            timeline.sort_by_key(|e| e.timestamp);
        }

        mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_task_create() {
        let tool_call = json!({
            "name": "TaskCreate",
            "input": {
                "taskId": "task-123",
                "subject": "Implement feature X",
                "description": "Add new functionality"
            }
        });

        let line = SessionLine {
            session_id: Some("session-1".to_string()),
            line_type: "assistant".to_string(),
            timestamp: Some(Utc::now()),
            cwd: None,
            git_branch: None,
            message: Some(crate::models::session::SessionMessage {
                role: Some("assistant".to_string()),
                content: None,
                tool_calls: Some(vec![tool_call]),
                tool_results: None,
                usage: None,
            }),
            model: None,
            usage: None,
            summary: None,
            parent_session_id: None,
        };

        let events = TodoWriteParser::extract_task_events(&line);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].task_id, "task-123");
        assert_eq!(events[0].subject, Some("Implement feature X".to_string()));
        assert_eq!(events[0].event_type, TaskEventType::Created);
    }

    #[test]
    fn test_parse_task_update() {
        let tool_call = json!({
            "name": "TaskUpdate",
            "input": {
                "taskId": "task-123",
                "status": "completed"
            }
        });

        let line = SessionLine {
            session_id: Some("session-1".to_string()),
            line_type: "assistant".to_string(),
            timestamp: Some(Utc::now()),
            cwd: None,
            git_branch: None,
            message: Some(crate::models::session::SessionMessage {
                role: Some("assistant".to_string()),
                content: None,
                tool_calls: Some(vec![tool_call]),
                tool_results: None,
                usage: None,
            }),
            model: None,
            usage: None,
            summary: None,
            parent_session_id: None,
        };

        let events = TodoWriteParser::extract_task_events(&line);
        assert!(events.is_some());

        let events = events.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].task_id, "task-123");
        assert_eq!(events[0].event_type, TaskEventType::Completed);
    }

    #[test]
    fn test_build_mapping() {
        let events = vec![
            TaskEvent {
                session_id: "session-1".to_string(),
                timestamp: Utc::now(),
                event_type: TaskEventType::Created,
                task_id: "task-1".to_string(),
                subject: Some("Task 1".to_string()),
                description: None,
                status: Some("pending".to_string()),
            },
            TaskEvent {
                session_id: "session-1".to_string(),
                timestamp: Utc::now(),
                event_type: TaskEventType::Completed,
                task_id: "task-1".to_string(),
                subject: None,
                description: None,
                status: Some("completed".to_string()),
            },
        ];

        let mapping = TodoWriteParser::build_mapping(events);

        assert_eq!(mapping.session_tasks.len(), 1);
        assert_eq!(mapping.session_tasks.get("session-1").unwrap().len(), 2);

        assert_eq!(mapping.task_timeline.len(), 1);
        assert_eq!(mapping.task_timeline.get("task-1").unwrap().len(), 2);
    }
}
