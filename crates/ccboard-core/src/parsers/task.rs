//! Parser for Claude Code task list files
//!
//! Parses `~/.claude/tasks/<list-id>/<task-id>.json` to extract task metadata.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Task status from Claude Code task list
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

/// Task metadata from task list JSON
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Task {
    pub id: String,
    pub status: TaskStatus,
    pub subject: String,
    pub description: Option<String>,
    pub blocked_by: Vec<String>,
}

/// Parser for task JSON files
pub struct TaskParser;

impl TaskParser {
    /// Parse a task from JSON string
    ///
    /// This is the entry point for TDD - we'll build this incrementally
    pub fn parse(json: &str) -> Result<Task> {
        // GREEN: Minimal implementation to pass test_parses_minimal_pending_task
        let task: Task = serde_json::from_str(json).context("Failed to parse task JSON")?;
        Ok(task)
    }

    /// Load a task from file path
    pub fn load(path: &Path) -> Result<Task> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read task file: {}", path.display()))?;

        Self::parse(&content)
            .with_context(|| format!("Failed to parse task from: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TDD Cycle 1: Minimal task parsing
    // RED: This test will fail because parse() is unimplemented
    #[test]
    fn test_parses_minimal_pending_task() {
        let json = r#"{
            "id": "task-1",
            "status": "pending",
            "subject": "Write tests first",
            "blocked_by": []
        }"#;

        let task = TaskParser::parse(json).unwrap();

        assert_eq!(task.id, "task-1");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.subject, "Write tests first");
        assert!(task.blocked_by.is_empty());
        assert!(task.description.is_none());
    }

    // TDD Cycle 2: Task with description and dependencies
    #[test]
    fn test_parses_task_with_description_and_dependencies() {
        let json = r#"{
            "id": "task-2",
            "status": "inprogress",
            "subject": "Implement feature",
            "description": "Detailed implementation steps",
            "blocked_by": ["task-1", "task-3"]
        }"#;

        let task = TaskParser::parse(json).unwrap();

        assert_eq!(task.id, "task-2");
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.subject, "Implement feature");
        assert_eq!(
            task.description,
            Some("Detailed implementation steps".to_string())
        );
        assert_eq!(task.blocked_by, vec!["task-1", "task-3"]);
    }

    // TDD Cycle 3: Completed task
    #[test]
    fn test_parses_completed_task() {
        let json = r#"{
            "id": "task-3",
            "status": "completed",
            "subject": "Done task",
            "blocked_by": []
        }"#;

        let task = TaskParser::parse(json).unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    // TDD Cycle 4: Edge case - Invalid JSON returns error with context
    #[test]
    fn test_invalid_json_returns_error_with_context() {
        let invalid_json = "{ invalid json }";

        let result = TaskParser::parse(invalid_json);

        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Failed to parse task JSON"));
    }

    // TDD Cycle 5: Edge case - Missing required field
    #[test]
    fn test_missing_required_field_returns_error() {
        let json = r#"{
            "id": "task-4",
            "status": "pending"
        }"#;

        let result = TaskParser::parse(json);

        // Should fail because 'subject' is required
        assert!(result.is_err());
    }

    // TDD Cycle 6: Edge case - Unknown status value
    #[test]
    fn test_unknown_status_returns_error() {
        let json = r#"{
            "id": "task-5",
            "status": "invalid_status",
            "subject": "Test",
            "blocked_by": []
        }"#;

        let result = TaskParser::parse(json);

        // Should fail because status is not valid enum variant
        assert!(result.is_err());
    }

    // TDD Cycle 7: Load from file
    #[test]
    fn test_load_from_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let json = r#"{
            "id": "task-file",
            "status": "pending",
            "subject": "Test from file",
            "blocked_by": []
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json.as_bytes()).unwrap();

        let task = TaskParser::load(temp_file.path()).unwrap();

        assert_eq!(task.id, "task-file");
        assert_eq!(task.subject, "Test from file");
    }

    // TDD Cycle 8: Load from non-existent file
    #[test]
    fn test_load_from_missing_file_returns_error() {
        use std::path::PathBuf;

        let path = PathBuf::from("/nonexistent/path/task.json");
        let result = TaskParser::load(&path);

        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("Failed to read task file"));
    }

    // TDD Cycle 9: Real fixture validation
    #[test]
    fn test_parse_real_fixture_pending() {
        let fixture = include_str!("../../tests/fixtures/tasks/task-pending.json");
        let task = TaskParser::parse(fixture).unwrap();

        assert_eq!(task.id, "task-123");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.description.is_some());
        assert!(task.blocked_by.is_empty());
    }

    #[test]
    fn test_parse_real_fixture_with_dependencies() {
        let fixture = include_str!("../../tests/fixtures/tasks/task-inprogress.json");
        let task = TaskParser::parse(fixture).unwrap();

        assert_eq!(task.id, "task-456");
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.blocked_by, vec!["task-123"]);
    }
}
